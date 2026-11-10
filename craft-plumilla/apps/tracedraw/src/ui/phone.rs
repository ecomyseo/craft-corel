//! The phone layout (#1301, #1497, #1691): on a screen whose smaller
//! side is under [`PHONE_DP`] density-independent pixels (Android's
//! `smallestScreenWidthDp`, the same at any density and in either
//! orientation) the window is laid out for a phone. The Touch
//! workspace's larger size is not applied (it made a 360 dp phone a
//! 257 point wide window, too narrow for any bar), the menu titles that
//! do not fit go under a "»" title at the end of the menu bar, and an
//! open docker takes the whole room beside its tab strip, as a sheet,
//! instead of a column beside the drawing. The bars, the toolbox and the
//! palette slide as on the desktop (`overflow.rs`). A tablet (600 dp and
//! more) keeps the layout it had. See docs/behavior/android.md.

use std::cell::Cell;

/// The smaller screen side below which the screen is a phone's, in
/// density-independent pixels (system points): Android's own line
/// between phone and tablet layouts.
pub const PHONE_DP: f32 = 600.0;

thread_local! {
    /// Whether the frame being drawn is laid out for a phone (set once a
    /// frame by [`update`], on the thread that draws).
    static PHONE: Cell<bool> = const { Cell::new(false) };
}

/// The screen's size in system points: the monitor's when the system
/// gives it (the whole screen on Android and in a browser that knows
/// it), else the window's. egui reports both in interface points, which
/// shrink as the interface grows.
pub fn screen_points(ctx: &egui::Context) -> egui::Vec2 {
    let zoom = ctx.zoom_factor();
    let monitor = ctx.input(|i| i.viewport().monitor_size);
    let size = match monitor {
        Some(m) if m.x > 0.0 && m.y > 0.0 => m,
        _ => ctx.content_rect().size(),
    };
    size * zoom
}

/// Whether a screen of `size` system points is a phone's.
pub fn is_phone_size(size: egui::Vec2) -> bool {
    size.x.min(size.y) < PHONE_DP
}

/// Decide, for this frame, whether the window is laid out for a phone.
pub fn update(ctx: &egui::Context) {
    let phone = is_phone_size(screen_points(ctx));
    PHONE.with(|p| p.set(phone));
}

/// Whether this frame is laid out for a phone.
pub fn phone() -> bool {
    PHONE.with(Cell::get)
}

/// Lay out the frames drawn on this thread as for a phone (tests).
#[cfg(test)]
pub fn set_phone(phone: bool) {
    PHONE.with(|p| p.set(phone));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Android's phone sizes are phones in both orientations; a 7 or 10
    /// inch tablet (600 dp and more) is not.
    #[test]
    fn phones_are_told_by_their_smaller_side() {
        for (w, h) in [
            (360.0, 640.0),
            (640.0, 360.0),
            (412.0, 915.0),
            (393.0, 873.0),
        ] {
            assert!(is_phone_size(egui::vec2(w, h)), "{w} x {h}");
        }
        for (w, h) in [
            (800.0, 1280.0),
            (1280.0, 800.0),
            (600.0, 960.0),
            (1920.0, 1080.0),
        ] {
            assert!(!is_phone_size(egui::vec2(w, h)), "{w} x {h}");
        }
    }
}
