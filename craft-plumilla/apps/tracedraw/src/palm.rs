//! Palm rejection on Android (#1832): while a pen touches the screen or
//! hovers over it, the hand resting on the screen does not draw, pick or
//! pan. Android reports each pointer with its tool (finger, stylus,
//! eraser, mouse, a palm it recognised), which winit drops; the patched
//! copy of winit asks [`PalmRejection::judge`] about every pointer of a
//! motion event first (`vendor/winit/src/pointer_filter.rs`), and
//! delivers a touch only when it says so.
//!
//! The rules, with Options > Tools > Curves > "Ignore the hand while
//! drawing with a pen" on (the default):
//! - A pen (stylus or eraser) always gets through.
//! - A finger that comes down while a pen is down or hovering is ignored
//!   until it lifts, whatever the pen does meanwhile.
//! - A pen that comes down while fingers are down (the hand landed
//!   first) ends those touches (`Cancelled`) and they are ignored until
//!   they lift.
//! - With no pen near, fingers work as before: a tap, a drag, two
//!   fingers to pan and pinch to zoom.
//! - A touch the system itself calls a palm never gets through.
//!
//! Off, every touch gets through as before. The mouse is never touched.
//! See docs/behavior/android.md.

// Outside Android nothing reports pointers with their tools: only the
// tests run the filter there.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "android")]
pub use winit::platform::android::{PointerAction, PointerReport, PointerTool, Verdict};
#[cfg(not(target_os = "android"))]
pub use winit_pointer_filter::{PointerAction, PointerReport, PointerTool, Verdict};

/// The patched winit's filter types, outside Android (the tests and the
/// desktop builds, which have no such events): the same file.
#[cfg(not(target_os = "android"))]
#[path = "../../../vendor/winit/src/pointer_filter.rs"]
#[allow(dead_code)]
mod winit_pointer_filter;

/// Options > Tools > Curves > "Ignore the hand while drawing with a pen",
/// as the editor last drew a frame (the filter runs on the input's turn).
static ON: AtomicBool = AtomicBool::new(true);

/// Turn palm rejection on or off (each frame, from the options).
pub fn set_enabled(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// Whether palm rejection is on.
pub fn enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// What the filter knows of the pointers now down or near.
#[derive(Debug, Default)]
pub struct PalmRejection {
    /// Pens touching the screen: (device, pointer).
    pens_down: HashSet<(i32, u64)>,
    /// Pens hovering near the screen.
    pens_near: HashSet<(i32, u64)>,
    /// Fingers delivered and not lifted yet.
    fingers: HashSet<(i32, u64)>,
    /// Fingers being ignored until they lift.
    ignored: HashSet<(i32, u64)>,
}

impl PalmRejection {
    /// Whether a pen is down or hovering.
    pub fn pen_near(&self) -> bool {
        !self.pens_down.is_empty() || !self.pens_near.is_empty()
    }

    /// The verdict on one pointer, with palm rejection `on` or off.
    pub fn judge(&mut self, r: &PointerReport, on: bool) -> Verdict {
        let key = (r.device, r.pointer);
        let lifted = matches!(r.action, PointerAction::Up | PointerAction::Cancel);
        match r.tool {
            PointerTool::Stylus | PointerTool::Eraser => {
                match r.action {
                    PointerAction::HoverEnter | PointerAction::HoverMove => {
                        self.pens_near.insert(key);
                    }
                    PointerAction::HoverExit => {
                        self.pens_near.remove(&key);
                    }
                    PointerAction::Down => {
                        self.pens_near.remove(&key);
                        self.pens_down.insert(key);
                        if on && !self.fingers.is_empty() {
                            // The hand landed first: its touches end.
                            let cancel: Vec<u64> = self
                                .fingers
                                .iter()
                                .filter(|(d, _)| *d == r.device)
                                .map(|(_, p)| *p)
                                .collect();
                            for p in &cancel {
                                self.fingers.remove(&(r.device, *p));
                                self.ignored.insert((r.device, *p));
                            }
                            return Verdict {
                                deliver: true,
                                cancel,
                            };
                        }
                    }
                    PointerAction::Move => {
                        self.pens_down.insert(key);
                    }
                    PointerAction::Up | PointerAction::Cancel => {
                        self.pens_down.remove(&key);
                    }
                }
                Verdict::deliver()
            }
            PointerTool::Finger | PointerTool::Unknown | PointerTool::Palm => {
                if matches!(
                    r.action,
                    PointerAction::HoverEnter | PointerAction::HoverMove | PointerAction::HoverExit
                ) {
                    return Verdict::drop();
                }
                if self.ignored.contains(&key) {
                    if lifted {
                        self.ignored.remove(&key);
                    }
                    return Verdict::drop();
                }
                let palm = r.tool == PointerTool::Palm;
                if on && r.action == PointerAction::Down && (palm || self.pen_near()) {
                    self.ignored.insert(key);
                    return Verdict::drop();
                }
                if r.action == PointerAction::Down {
                    self.fingers.insert(key);
                } else if lifted {
                    self.fingers.remove(&key);
                }
                Verdict::deliver()
            }
            PointerTool::Mouse => Verdict::deliver(),
        }
    }
}

/// Have the patched winit ask about every pointer from now on.
#[cfg(target_os = "android")]
pub fn install() {
    let mut state = PalmRejection::default();
    winit::platform::android::set_pointer_filter(Box::new(move |r| state.judge(r, enabled())));
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: i32 = 4;

    fn report(pointer: u64, tool: PointerTool, action: PointerAction) -> PointerReport {
        PointerReport {
            device: SCREEN,
            pointer,
            tool,
            action,
        }
    }

    fn finger(f: &mut PalmRejection, pointer: u64, action: PointerAction) -> Verdict {
        f.judge(&report(pointer, PointerTool::Finger, action), true)
    }

    fn pen(f: &mut PalmRejection, action: PointerAction) -> Verdict {
        f.judge(&report(9, PointerTool::Stylus, action), true)
    }

    use PointerAction::*;

    /// A hand resting on the screen while the pen draws: the pen's
    /// strokes get through, the hand's touches do not, until it lifts.
    #[test]
    fn the_hand_does_not_draw_while_the_pen_is_down() {
        let mut f = PalmRejection::default();
        assert!(pen(&mut f, Down).deliver);
        assert!(!finger(&mut f, 0, Down).deliver);
        assert!(pen(&mut f, Move).deliver);
        assert!(!finger(&mut f, 0, Move).deliver);
        assert!(pen(&mut f, Up).deliver);
        // The pen lifted, the hand still rests: still ignored.
        assert!(!finger(&mut f, 0, Move).deliver);
        assert!(!finger(&mut f, 0, Up).deliver);
        // A new touch with no pen near draws again.
        assert!(finger(&mut f, 1, Down).deliver);
        assert!(finger(&mut f, 1, Up).deliver);
    }

    /// A pen hovering over the screen (Android's hover events of a
    /// stylus) is enough: the hand coming down then is ignored.
    #[test]
    fn a_hovering_pen_keeps_the_hand_out() {
        let mut f = PalmRejection::default();
        pen(&mut f, HoverEnter);
        pen(&mut f, HoverMove);
        assert!(f.pen_near());
        assert!(!finger(&mut f, 0, Down).deliver);
        pen(&mut f, HoverExit);
        assert!(!f.pen_near());
        assert!(!finger(&mut f, 0, Up).deliver);
        assert!(finger(&mut f, 2, Down).deliver);
    }

    /// The hand landed before the pen: its touch is ended (cancelled) as
    /// the pen comes down, and nothing more of it gets through.
    #[test]
    fn a_hand_that_landed_first_is_cancelled_when_the_pen_comes_down() {
        let mut f = PalmRejection::default();
        assert!(finger(&mut f, 0, Down).deliver);
        assert!(finger(&mut f, 0, Move).deliver);
        let v = pen(&mut f, Down);
        assert!(v.deliver);
        assert_eq!(v.cancel, vec![0]);
        assert!(!finger(&mut f, 0, Move).deliver);
        // Only once.
        assert!(pen(&mut f, Move).cancel.is_empty());
        assert!(!finger(&mut f, 0, Up).deliver);
    }

    /// With no pen, two fingers pan and pinch as before; the eraser end
    /// of a pen counts as a pen; a system-recognised palm never gets
    /// through; the mouse always does.
    #[test]
    fn fingers_alone_work_as_before() {
        let mut f = PalmRejection::default();
        assert!(finger(&mut f, 0, Down).deliver);
        assert!(finger(&mut f, 1, Down).deliver);
        assert!(finger(&mut f, 0, Move).deliver);
        assert!(finger(&mut f, 1, Move).deliver);
        assert!(finger(&mut f, 1, Up).deliver);
        assert!(finger(&mut f, 0, Up).deliver);
        let eraser = f.judge(&report(5, PointerTool::Eraser, Down), true);
        assert!(eraser.deliver);
        assert!(!finger(&mut f, 3, Down).deliver);
        f.judge(&report(5, PointerTool::Eraser, Up), true);
        assert!(!f.judge(&report(6, PointerTool::Palm, Down), true).deliver);
        assert!(f.judge(&report(7, PointerTool::Mouse, Down), true).deliver);
    }

    /// Off: every touch gets through, also with the pen down.
    #[test]
    fn off_lets_the_hand_through() {
        let mut f = PalmRejection::default();
        assert!(
            f.judge(&report(9, PointerTool::Stylus, Down), false)
                .deliver
        );
        let v = f.judge(&report(0, PointerTool::Finger, Down), false);
        assert!(v.deliver && v.cancel.is_empty());
        assert!(f.judge(&report(6, PointerTool::Palm, Down), false).deliver);
    }

    /// The setting turns the filter on and off; it is on at first.
    #[test]
    fn the_option_is_on_at_first() {
        assert!(crate::prefs::CurvePrefs::default().pen_ignores_hand);
    }
}
