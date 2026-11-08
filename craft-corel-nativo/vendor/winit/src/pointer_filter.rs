//! TraceDraw: the program's say on each pointer of Android's motion
//! events, with the tool that made it (finger, stylus, eraser, mouse),
//! for palm rejection (#1832, see `vendor/README.md`). winit hands every
//! touch to the program as a `Touch` with no word of its tool, and drops
//! the hover events of a pen near the screen. With a filter set, the
//! Android backend reports each pointer (and each hover) here first and
//! delivers its touch only when the filter says so; before that it ends
//! the touches the filter names (a hand that landed before the pen) with
//! `Cancelled`. With no filter set every touch is delivered, as winit does
//! without the change. The file has no dependency on the rest of the
//! crate, so that TraceDraw's tests can include and run it.

use std::sync::{Mutex, MutexGuard};

/// What made a pointer (Android's `MotionEvent.TOOL_TYPE_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerTool {
    Unknown,
    Finger,
    Stylus,
    Mouse,
    Eraser,
    /// A palm the system recognised as one.
    Palm,
}

/// What a pointer did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerAction {
    Down,
    Move,
    Up,
    Cancel,
    /// A pen came near the screen, moved over it or went away, without
    /// touching it.
    HoverEnter,
    HoverMove,
    HoverExit,
}

/// One pointer of a motion event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerReport {
    /// The input device (a touch screen and its pen can be one device).
    pub device: i32,
    /// The pointer's id, the `id` of its `Touch`.
    pub pointer: u64,
    pub tool: PointerTool,
    pub action: PointerAction,
}

/// The filter's answer for one pointer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Verdict {
    /// Deliver the pointer's touch.
    pub deliver: bool,
    /// Touches delivered before, of the same device, to end first with
    /// `Cancelled` (and never deliver again).
    pub cancel: Vec<u64>,
}

impl Verdict {
    /// Deliver the touch, cancel nothing.
    pub fn deliver() -> Self {
        Verdict {
            deliver: true,
            cancel: Vec::new(),
        }
    }

    /// Drop the touch.
    pub fn drop() -> Self {
        Verdict::default()
    }
}

type Filter = Box<dyn FnMut(&PointerReport) -> Verdict + Send>;

static FILTER: Mutex<Option<Filter>> = Mutex::new(None);

fn filter() -> MutexGuard<'static, Option<Filter>> {
    FILTER.lock().unwrap_or_else(|e| e.into_inner())
}

/// Set the filter asked about every pointer. It must not set a filter
/// itself.
pub fn set_filter(f: Filter) {
    *filter() = Some(f);
}

/// The filter's verdict on `report`: with no filter set, a touch is
/// delivered (a hover has no touch to deliver either way).
pub fn judge(report: &PointerReport) -> Verdict {
    match filter().as_mut() {
        Some(f) => f(report),
        None => Verdict::deliver(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // One test: the filter is one static for the whole process.
    #[test]
    fn without_a_filter_everything_is_delivered_and_with_one_it_decides() {
        let finger = PointerReport {
            device: 3,
            pointer: 0,
            tool: PointerTool::Finger,
            action: PointerAction::Down,
        };
        assert_eq!(judge(&finger), Verdict::deliver());
        let asked = Arc::new(AtomicUsize::new(0));
        let count = asked.clone();
        set_filter(Box::new(move |r| {
            count.fetch_add(1, Ordering::SeqCst);
            if r.tool == PointerTool::Finger {
                Verdict::drop()
            } else {
                Verdict {
                    deliver: true,
                    cancel: vec![0],
                }
            }
        }));
        assert_eq!(judge(&finger), Verdict::drop());
        let pen = PointerReport {
            tool: PointerTool::Stylus,
            pointer: 1,
            ..finger
        };
        assert_eq!(judge(&pen).cancel, vec![0]);
        assert_eq!(asked.load(Ordering::SeqCst), 2);
    }
}
