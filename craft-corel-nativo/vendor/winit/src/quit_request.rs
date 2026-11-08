//! TraceDraw: the program's say on a quit asked of the whole application
//! (macOS: the Quit item of the application menu and its Command+Q, the
//! Dock icon's Quit, a log-out or shut-down; see `vendor/README.md`).
//! AppKit asks the application delegate whether it may terminate; the
//! delegate asks here. With a handler set, the handler decides: `true`
//! means the program takes the request over (it asks about unsaved work
//! and closes its window itself), so the delegate cancels the terminate;
//! `false` lets it go ahead. With no handler set the terminate goes ahead,
//! as winit does without the change. The file has no dependency on the
//! rest of the crate, so that TraceDraw's tests can include and run it.

use std::sync::{Mutex, MutexGuard};

type Handler = Box<dyn Fn() -> bool + Send>;

static HANDLER: Mutex<Option<Handler>> = Mutex::new(None);

fn handler() -> MutexGuard<'static, Option<Handler>> {
    HANDLER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Set the handler asked when the system wants the application to quit.
/// It must not set a handler itself.
pub fn set_handler(h: Handler) {
    *handler() = Some(h);
}

/// The system asks to quit: true when the program takes the request over
/// and the terminate is to be cancelled, false when it may go ahead.
pub fn program_takes_it() -> bool {
    handler().as_ref().is_some_and(|h| h())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    // One test: the handler is one static for the whole process.
    #[test]
    fn without_a_handler_the_quit_goes_ahead_and_with_one_the_handler_decides() {
        assert!(!program_takes_it());
        let unsaved = Arc::new(AtomicBool::new(false));
        let asked = Arc::new(AtomicUsize::new(0));
        let (u, a) = (unsaved.clone(), asked.clone());
        set_handler(Box::new(move || {
            a.fetch_add(1, Ordering::SeqCst);
            u.load(Ordering::SeqCst)
        }));
        assert!(!program_takes_it());
        unsaved.store(true, Ordering::SeqCst);
        assert!(program_takes_it());
        assert_eq!(asked.load(Ordering::SeqCst), 2);
    }
}
