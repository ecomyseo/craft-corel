//! The control line of an interactive effect (the fill or transparency
//! line, the drop shadow's line, the extrusion's line to its vanishing
//! point, the blend's line, the distortion's handle line): a press on it
//! within the pick distance (Options > Nodes and Handles), away from its
//! handles and sliders, takes the whole effect, and the drag moves it by
//! as much as the pointer moves, keeping the line's length and angle.
//! Where one end belongs to the object (a drop shadow's start, an
//! extrusion's middle), that end stays and the other moves by the drag.
//! Ctrl keeps that move horizontal or vertical, whichever is longer, as
//! it keeps an object's move straight. The tools' pages in docs/behavior
//! (gradient-fill, transparency, drop-shadow, extrude, blend, distort) say
//! the same.

use crate::app::App;
use tracedraw_core::geometry::{Point, Vec2};

/// `p` lies within `tol` of the segment from `a` to `b`, its ends
/// included.
pub fn near_segment(a: Point, b: Point, p: Point, tol: f64) -> bool {
    let d = b - a;
    let l2 = d.hypot2();
    let t = if l2 < 1e-12 {
        0.0
    } else {
        ((p - a).dot(d) / l2).clamp(0.0, 1.0)
    };
    let dist = (a + d * t - p).hypot();
    dist.is_finite() && dist <= tol
}

/// The pointer's way `d` since the press, kept horizontal or vertical
/// (whichever is longer) with `ctrl`.
pub fn constrain_move(d: Vec2, ctrl: bool) -> Vec2 {
    if !ctrl {
        d
    } else if d.x.abs() >= d.y.abs() {
        Vec2::new(d.x, 0.0)
    } else {
        Vec2::new(0.0, d.y)
    }
}

impl App {
    /// The pick distance (Options > Nodes and Handles) in page millimetres
    /// at the current zoom: how near a press must be to a control line.
    pub fn line_pick_tol(&self) -> f64 {
        self.settings.nodes.pick_px() / f64::from(self.view.zoom.max(1e-6))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_a_segment_and_its_ends() {
        let (a, b) = (Point::new(0.0, 0.0), Point::new(10.0, 0.0));
        assert!(near_segment(a, b, Point::new(5.0, 0.9), 1.0));
        assert!(!near_segment(a, b, Point::new(5.0, 1.1), 1.0));
        assert!(near_segment(a, b, Point::new(10.5, 0.0), 1.0));
        assert!(!near_segment(a, b, Point::new(11.5, 0.0), 1.0));
        // A line of no length is its point.
        assert!(near_segment(a, a, Point::new(0.5, 0.0), 1.0));
    }

    #[test]
    fn ctrl_keeps_the_move_straight() {
        let d = Vec2::new(5.0, 2.0);
        assert_eq!(constrain_move(d, false), d);
        assert_eq!(constrain_move(d, true), Vec2::new(5.0, 0.0));
        assert_eq!(
            constrain_move(Vec2::new(1.0, -3.0), true),
            Vec2::new(0.0, -3.0)
        );
    }
}
