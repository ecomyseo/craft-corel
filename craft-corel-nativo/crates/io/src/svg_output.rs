//! Output buffers shared by a single bounded SVG export operation.
//!
//! The byte budget covers strings emitted by the writer, not geometric work
//! performed before emission (font outlines, live effects and rasterization).

use std::cell::{Cell, RefCell};
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

struct Budget {
    per_buffer: usize,
    total_appends: usize,
    appended: Cell<usize>,
    failed: Cell<bool>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Rc<Budget>>> = const { RefCell::new(None) };
}

pub(super) fn limited(max_bytes: usize, render: impl FnOnce() -> String) -> Result<String, String> {
    let total_appends = max_bytes
        .checked_mul(8)
        .ok_or("SVG output budget overflows")?;
    let budget = Rc::new(Budget {
        per_buffer: max_bytes,
        total_appends,
        appended: Cell::new(0),
        failed: Cell::new(false),
    });
    struct Restore(Option<Rc<Budget>>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let removed = ACTIVE.with(|slot| slot.replace(self.0.take()));
            drop(removed);
        }
    }
    let _restore = Restore(ACTIVE.with(|slot| slot.replace(Some(Rc::clone(&budget)))));
    let output = render();
    if budget.failed.get() || output.len() > max_bytes {
        Err("SVG exceeds its output or intermediate append budget".into())
    } else {
        Ok(output)
    }
}

pub(super) struct Output {
    text: String,
    budget: Option<Rc<Budget>>,
}

impl Output {
    pub(super) fn new() -> Self {
        Self {
            text: String::new(),
            budget: ACTIVE.with(|slot| slot.borrow().clone()),
        }
    }

    pub(super) fn exhausted(&self) -> bool {
        self.budget
            .as_ref()
            .is_some_and(|budget| budget.failed.get())
    }

    fn append(&mut self, text: &str) -> fmt::Result {
        if let Some(budget) = &self.budget {
            let fits_buffer = text.len() <= budget.per_buffer.saturating_sub(self.text.len());
            let fits_work =
                text.len() <= budget.total_appends.saturating_sub(budget.appended.get());
            if budget.failed.get()
                || !fits_buffer
                || !fits_work
                || self.text.try_reserve(text.len()).is_err()
            {
                budget.failed.set(true);
                return Err(fmt::Error);
            }
            budget.appended.set(budget.appended.get() + text.len());
        }
        self.text.push_str(text);
        Ok(())
    }

    pub(super) fn push_str(&mut self, text: &str) {
        let _ = self.append(text);
    }

    pub(super) fn push(&mut self, ch: char) {
        let mut bytes = [0; 4];
        self.push_str(ch.encode_utf8(&mut bytes));
    }

    pub(super) fn into_string(self) -> String {
        self.text
    }
}

impl fmt::Write for Output {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.append(text)
    }
}

impl Deref for Output {
    type Target = str;
    fn deref(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl<S: AsRef<str>> FromIterator<S> for Output {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut output = Self::new();
        for item in iter {
            output.push_str(item.as_ref());
            if output.exhausted() {
                break;
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_oversized_append_never_allocates_or_returns_truncated_success() {
        let result = limited(8, || {
            let mut output = Output::new();
            output.push_str("exactly8");
            output.push_str("does not fit");
            assert_eq!(&*output, "exactly8");
            assert!(output.exhausted());
            output.into_string()
        });
        assert!(result.is_err());
        let mut ordinary = Output::new();
        ordinary.push_str("an unlimited operation after a failed limited one");
        assert!(!ordinary.exhausted());
    }

    #[test]
    fn intermediate_buffers_share_an_operation_budget() {
        let result = limited(8, || {
            for _ in 0..8 {
                let mut buffer = Output::new();
                buffer.push_str("exactly8");
                assert!(!buffer.exhausted());
            }
            let mut output = Output::new();
            output.push('x');
            assert!(output.exhausted());
            output.into_string()
        });
        assert!(result.is_err());
    }

    #[test]
    fn nested_export_scopes_restore_their_enclosing_budget() {
        assert_eq!(
            limited(16, || {
                assert!(limited(0, || "too much".into()).is_err());
                let mut output = Output::new();
                output.push_str("outer survives");
                output.into_string()
            })
            .unwrap(),
            "outer survives"
        );
    }
}
