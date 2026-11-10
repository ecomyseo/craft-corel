// TraceDraw: windows are centred on the primary monitor where it lies on
// the desktop, not from the desktop's top-left corner (see
// vendor/README.md, #1800). Kept apart so that TraceDraw's own tests
// compile and run it (`splash::eframe_monitor_place::tests`): this copy
// is outside the workspace and its own tests run in no gate.
//
// A monitor's position is where its top-left corner lies on the desktop
// that all monitors share. Centring a window by the monitor's size alone
// puts it on whatever monitor holds the desktop's top-left corner: with
// the primary monitor below and right of two others (X11, four
// monitors, #1800), the window opened on the top-left one.

use std::sync::OnceLock;

/// The primary monitor as `[x, y, width, height]` in system points
/// (physical pixels over the monitor's scale factor).
static PRIMARY: OnceLock<[f32; 4]> = OnceLock::new();

/// The primary monitor (or the first one the system lists) as `[x, y,
/// width, height]` in system points: its top-left corner on the desktop
/// and its size. `None` before the first window is made, and where the
/// system does not tell (Wayland, the browser).
#[allow(dead_code)]
pub fn primary_monitor() -> Option<[f32; 4]> {
    PRIMARY.get().copied()
}

/// Keep the primary monitor's rectangle for [`primary_monitor`]. Set
/// once; later calls are ignored.
#[allow(dead_code)]
pub(crate) fn remember_primary(rect: [f32; 4]) {
    if rect[2] > 0.0 && rect[3] > 0.0 && rect.iter().all(|v| v.is_finite()) {
        let _ = PRIMARY.set(rect);
    }
}

/// The top-left corner of a window of `inner` (width, height) centred on
/// `monitor` (`[x, y, width, height]`, the same unit), never above or left
/// of the monitor.
#[allow(dead_code)]
pub fn centred_on(monitor: [f32; 4], inner: [f32; 2]) -> [f32; 2] {
    [
        monitor[0] + ((monitor[2] - inner[0]) / 2.0).max(0.0),
        monitor[1] + ((monitor[3] - inner[1]) / 2.0).max(0.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The layout of #1800 at 100 %: the primary monitor (5120 x 2880)
    /// lies at (1544, 2160), below two 3840 x 2160 monitors at (0, 0) and
    /// (3840, 0). A window is centred on it, not on the desktop's
    /// top-left corner.
    #[test]
    fn a_window_is_centred_where_the_monitor_lies() {
        let primary = [1544.0, 2160.0, 5120.0, 2880.0];
        assert_eq!(
            centred_on(primary, [800.0, 600.0]),
            [1544.0 + 2160.0, 2160.0 + 1140.0]
        );
        // At the desktop's corner nothing changes.
        assert_eq!(
            centred_on([0.0, 0.0, 1920.0, 1080.0], [1600.0, 900.0]),
            [160.0, 90.0]
        );
        // Larger than the monitor: at its own top-left corner.
        assert_eq!(centred_on(primary, [6000.0, 3000.0]), [1544.0, 2160.0]);
        // A monitor left of or above the desktop's origin.
        assert_eq!(
            centred_on([-1920.0, -200.0, 1920.0, 1080.0], [1600.0, 900.0]),
            [-1760.0, -110.0]
        );
    }

    #[test]
    fn a_monitor_without_size_is_not_kept() {
        remember_primary([0.0, 0.0, 0.0, 1080.0]);
        remember_primary([f32::NAN, 0.0, 1920.0, 1080.0]);
        assert_eq!(primary_monitor(), None);
    }
}
