//! TraceDraw: the pressure of a graphics tablet's pen in the browser
//! (#1410). The published web layer reads a force only from touch events
//! (a finger, or a stylus the browser reports as a touch) and drops the
//! `pressure` of pointer events, so a pen on a tablet drew with no
//! pressure. A pen's pointer events now also send its pressure the way the
//! patched winit copy does on X11 and macOS (see `vendor/README.md`): an
//! `egui::Event::Touch` of phase `Move` only, with the reading as `force`,
//! which TraceDraw's raw input hook reads and then takes out before egui
//! sees it (`apps/tracedraw/src/pen.rs`). The pen's pointer events also
//! move the pointer, and its touches are left out (#1655): Safari on iPad
//! sends an Apple Pencil no mouse events (`pen_moved`, `is_stylus_touch`,
//! `pen_cancelled`).
//!
//! The file uses nothing of the crate but egui, so that TraceDraw's tests
//! can include and run it.

/// The `pointerType` of a pen's pointer events.
pub(crate) const PEN: &str = "pen";

/// The touch device id of the pen's moving touches; the touch events of
/// the published layer use 0.
pub(crate) const PEN_DEVICE: egui::TouchDeviceId = egui::TouchDeviceId(1);

/// `buttons` bits that mean the pen touches the surface: its tip (the
/// primary button) or its eraser end.
const CONTACT_BUTTONS: u16 = 1 | 32;

/// What the pen's pointer events said about pressure so far.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct PenPressure {
    /// A pen gave a reading other than 0.5 while pressed: its pressure is
    /// real. Until then 0.5 counts as no pressure, since that is what a
    /// browser reports, while pressed, for a pen that cannot measure it.
    measured: bool,
}

impl PenPressure {
    /// The force to send for a pointer event of `pointer_type` with
    /// `pressure` (0 to 1) and `buttons` (the bit mask of the buttons
    /// held). `None` sends nothing: a mouse (which never has pressure,
    /// and whose 0.5 while pressed means none), a touch (whose force
    /// still comes from the touch events, as before) and any other kind.
    /// `Some(None)` is a pen with no pressure: hovering, lifted, a reading
    /// of zero or not a number, or 0.5 before the pen gave any other
    /// reading; it clears the pressure, so none is ever stale.
    /// `Some(Some(p))` is the pen pressed at `p`.
    pub(crate) fn reading(
        &mut self,
        pointer_type: &str,
        pressure: f32,
        buttons: u16,
    ) -> Option<Option<f32>> {
        if pointer_type != PEN {
            return None;
        }
        if buttons & CONTACT_BUTTONS == 0 || !pressure.is_finite() || pressure <= 0.0 {
            return Some(None);
        }
        let pressure = pressure.min(1.0);
        if pressure == 0.5 && !self.measured {
            return Some(None);
        }
        self.measured = true;
        Some(Some(pressure))
    }
}

/// The event that carries a pen's `force` at `pos`: a moving touch of the
/// pen's device, with the pointer's id, never a start or an end (those
/// would press the primary button a second time; the pen's pointer
/// events press it already).
pub(crate) fn pen_touch(pointer_id: i32, pos: egui::Pos2, force: Option<f32>) -> egui::Event {
    egui::Event::Touch {
        device_id: PEN_DEVICE,
        id: egui::TouchId::from(pointer_id),
        phase: egui::TouchPhase::Move,
        pos,
        force,
    }
}

/// The `touchType` Safari gives the touches of an Apple Pencil.
pub(crate) const STYLUS: &str = "stylus";

/// Whether a touch event's touch is a stylus's (#1655). Safari on iPad
/// sends an Apple Pencil both as touches and as pen pointer events; the
/// pen's position, press and pressure come from its pointer events, so
/// its touches are left out, or the press would come twice and egui
/// would take the Pencil for a finger. A finger has the `touchType`
/// "direct", and browsers without the property give none.
pub(crate) fn is_stylus_touch(touch_type: Option<&str>) -> bool {
    touch_type == Some(STYLUS)
}

/// Where a pen's pointer event puts the pointer (#1655): Safari on iPad
/// sends an Apple Pencil no mouse events at all, so the pen's own
/// `pointerdown` and `pointermove` move the pointer, before the press
/// they may carry. Nothing for a mouse or a touch, whose position comes
/// from their mouse and touch events as before.
pub(crate) fn pen_moved(pointer_type: &str, pos: egui::Pos2) -> Option<egui::Event> {
    (pointer_type == PEN).then_some(egui::Event::PointerMoved(pos))
}

/// What a pen's `pointercancel` (the browser took the pen over, or it
/// left the screen without a `pointerup`) tells egui: no pressure, the
/// tip let go, the pointer gone, so that no drag is left pressed.
pub(crate) fn pen_cancelled(
    pointer_id: i32,
    pos: egui::Pos2,
    modifiers: egui::Modifiers,
) -> [egui::Event; 3] {
    [
        pen_touch(pointer_id, pos, None),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers,
        },
        egui::Event::PointerGone,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An Apple Pencil's touches are left out; a finger's, or those of a
    /// browser that gives no touch type, are not.
    #[test]
    fn only_a_stylus_touch_is_left_out() {
        assert!(is_stylus_touch(Some("stylus")));
        assert!(!is_stylus_touch(Some("direct")));
        assert!(!is_stylus_touch(None));
    }

    /// A pen's pointer events move the pointer; a mouse's and a touch's
    /// do not (their own events do).
    #[test]
    fn a_pens_pointer_events_move_the_pointer() {
        let pos = egui::pos2(5.0, 6.0);
        assert_eq!(pen_moved("pen", pos), Some(egui::Event::PointerMoved(pos)));
        assert_eq!(pen_moved("mouse", pos), None);
        assert_eq!(pen_moved("touch", pos), None);
    }

    /// A cancelled pen lets the tip go and leaves no pressure behind.
    #[test]
    fn a_cancelled_pen_lets_go() {
        let pos = egui::pos2(1.0, 2.0);
        let [touch, release, gone] = pen_cancelled(4, pos, egui::Modifiers::NONE);
        assert_eq!(touch, pen_touch(4, pos, None));
        assert!(matches!(
            release,
            egui::Event::PointerButton {
                button: egui::PointerButton::Primary,
                pressed: false,
                ..
            }
        ));
        assert_eq!(gone, egui::Event::PointerGone);
    }

    #[test]
    fn only_a_pen_gives_pressure() {
        let mut p = PenPressure::default();
        // The mouse: nothing at all, pressed (0.5) or not (0).
        assert_eq!(p.reading("mouse", 0.5, 1), None);
        assert_eq!(p.reading("mouse", 0.0, 0), None);
        // A touch keeps its force from the touch events.
        assert_eq!(p.reading("touch", 0.7, 1), None);
        assert_eq!(p.reading("", 0.7, 1), None);
        // A pen pressed on its tip or its eraser.
        assert_eq!(p.reading("pen", 0.3, 1), Some(Some(0.3)));
        assert_eq!(p.reading("pen", 0.8, 32), Some(Some(0.8)));
        assert_eq!(p.reading("pen", 0.8, 1 | 2), Some(Some(0.8)));
        // Readings past 1 are clamped.
        assert_eq!(p.reading("pen", 1.5, 1), Some(Some(1.0)));
    }

    #[test]
    fn a_pen_without_contact_or_reading_clears_the_pressure() {
        let mut p = PenPressure::default();
        // Hovering, the barrel button alone, lifted.
        assert_eq!(p.reading("pen", 0.0, 0), Some(None));
        assert_eq!(p.reading("pen", 0.0, 2), Some(None));
        assert_eq!(p.reading("pen", 0.4, 0), Some(None));
        // Pressed with no reading, or not a number.
        assert_eq!(p.reading("pen", 0.0, 1), Some(None));
        assert_eq!(p.reading("pen", f32::NAN, 1), Some(None));
        assert_eq!(p.reading("pen", -0.2, 1), Some(None));
    }

    /// A pen that cannot measure pressure reports 0.5 while pressed: no
    /// pressure, so it draws like the mouse, until a pen gives another
    /// reading, after which 0.5 is a reading like any other.
    #[test]
    fn a_constant_half_is_no_pressure_until_a_pen_measures() {
        let mut p = PenPressure::default();
        for _ in 0..3 {
            assert_eq!(p.reading("pen", 0.5, 1), Some(None));
        }
        assert_eq!(p.reading("pen", 0.42, 1), Some(Some(0.42)));
        assert_eq!(p.reading("pen", 0.5, 1), Some(Some(0.5)));
        // Hovering still clears it.
        assert_eq!(p.reading("pen", 0.0, 0), Some(None));
    }

    #[test]
    fn the_pen_touch_only_moves() {
        let pos = egui::pos2(3.0, 4.0);
        assert_eq!(
            pen_touch(2, pos, Some(0.25)),
            egui::Event::Touch {
                device_id: PEN_DEVICE,
                id: egui::TouchId(2),
                phase: egui::TouchPhase::Move,
                pos,
                force: Some(0.25),
            }
        );
        assert_ne!(PEN_DEVICE, egui::TouchDeviceId(0));
    }
}
