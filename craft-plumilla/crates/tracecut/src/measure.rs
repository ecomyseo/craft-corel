//! Outline length and area of objects, for pricing and cutting time.
//!
//! Length is the arc length of every segment, the closing segment of a
//! closed subpath included. Area follows the even-odd rule inside each
//! object: a closed subpath inside an odd number of others is a hole, so
//! the letter O counts its ring only. Open subpaths have length but no
//! area. Objects are measured on their own: an object lying on another is
//! a separate piece and adds its own area.

use crate::subpaths;
use tracedraw_core::geometry::{BezPath, ParamCurveArclen, PathEl, Point, Rect, Shape as _};

/// Accuracy of the arc length of curves, in millimetres.
pub const ARCLEN_ACCURACY: f64 = 1e-4;

/// What [`measure`] found, in millimetres and square millimetres.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Measurement {
    /// Number of objects measured.
    pub objects: usize,
    /// Total outline length.
    pub length: f64,
    /// Material area: outer areas minus holes.
    pub area: f64,
    /// Area of the outer subpaths (and islands inside holes).
    pub outer_area: f64,
    /// Area of the holes.
    pub hole_area: f64,
    /// Area of the bounding box of everything measured.
    pub bbox_area: f64,
    /// The bounding box, when anything was measured.
    pub bbox: Option<Rect>,
    /// Closed and open subpaths.
    pub closed_paths: usize,
    pub open_paths: usize,
    /// Nodes (segment ends and subpath starts).
    pub nodes: usize,
}

impl Measurement {
    /// Add another measurement to this one.
    pub fn add(&mut self, o: &Measurement) {
        self.objects += o.objects;
        self.length += o.length;
        self.area += o.area;
        self.outer_area += o.outer_area;
        self.hole_area += o.hole_area;
        self.closed_paths += o.closed_paths;
        self.open_paths += o.open_paths;
        self.nodes += o.nodes;
        self.bbox = match (self.bbox, o.bbox) {
            (Some(a), Some(b)) => Some(a.union(b)),
            (a, b) => a.or(b),
        };
        self.bbox_area = self.bbox.map_or(0.0, |b| b.area().abs());
    }
}

/// Outline length of a path.
pub fn length(path: &BezPath) -> f64 {
    path.segments()
        .map(|s| s.arclen(ARCLEN_ACCURACY))
        .filter(|l| l.is_finite())
        .sum()
}

/// Measure one object given by its outlines (see [`crate::outlines`]).
pub fn measure_object(paths: &[BezPath]) -> Measurement {
    let mut m = Measurement {
        objects: 1,
        ..Default::default()
    };
    // Every path of the object is a piece of its own (a group's members).
    for path in paths {
        m.length += length(path);
        m.nodes += path
            .elements()
            .iter()
            .filter(|e| !matches!(e, PathEl::ClosePath))
            .count();
        let bb = path.bounding_box();
        if bb.x0.is_finite() && bb.y0.is_finite() {
            m.bbox = Some(m.bbox.map_or(bb, |b| b.union(bb)));
        }
        let subs = subpaths(path);
        let mut closed: Vec<(BezPath, f64, Point)> = Vec::new();
        for s in subs {
            if s.closed {
                m.closed_paths += 1;
                let mut p = s.path;
                if !matches!(p.elements().last(), Some(PathEl::ClosePath)) {
                    p.close_path();
                }
                let a = p.area();
                if !a.is_finite() || a.abs() < 1e-12 {
                    continue;
                }
                let probe = probe_point(&p);
                closed.push((p, a.abs(), probe));
            } else {
                m.open_paths += 1;
            }
        }
        for (i, (_, area, probe)) in closed.iter().enumerate() {
            let depth = closed
                .iter()
                .enumerate()
                .filter(|(j, (other, oa, _))| *j != i && *oa > *area && other.winding(*probe) != 0)
                .count();
            if depth % 2 == 0 {
                m.outer_area += area;
            } else {
                m.hole_area += area;
            }
        }
    }
    m.area = m.outer_area - m.hole_area;
    m.bbox_area = m.bbox.map_or(0.0, |b| b.area().abs());
    m
}

/// Measure several objects and add them up.
pub fn measure(objects: &[Vec<BezPath>]) -> Measurement {
    let mut total = Measurement::default();
    for o in objects {
        total.add(&measure_object(o));
    }
    total
}

/// A point on the subpath away from its nodes (the middle of its first
/// segment), to test which other subpaths hold it.
fn probe_point(p: &BezPath) -> Point {
    use tracedraw_core::geometry::ParamCurve;
    p.segments()
        .next()
        .map(|s| s.eval(0.5))
        .unwrap_or(Point::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Circle, Rect};

    fn square(x: f64, y: f64, s: f64) -> BezPath {
        Rect::new(x, y, x + s, y + s).to_path(0.01)
    }

    #[test]
    fn a_square_has_its_perimeter_and_area() {
        let m = measure_object(&[square(0.0, 0.0, 10.0)]);
        assert!((m.length - 40.0).abs() < 1e-9);
        assert!((m.area - 100.0).abs() < 1e-9);
        assert_eq!(m.closed_paths, 1);
        assert_eq!(m.nodes, 4);
    }

    #[test]
    fn a_circle_measures_two_pi_r() {
        let c = Circle::new((0.0, 0.0), 10.0).to_path(1e-4);
        let m = measure_object(&[c]);
        assert!((m.length - 2.0 * std::f64::consts::PI * 10.0).abs() < 1e-3);
        assert!((m.area - std::f64::consts::PI * 100.0).abs() < 1e-2);
    }

    #[test]
    fn holes_are_taken_away_and_islands_added_back() {
        let mut p = square(0.0, 0.0, 30.0);
        // A hole, drawn in the same direction (even-odd decides, not winding).
        p.extend(square(5.0, 5.0, 20.0));
        // An island in the hole.
        p.extend(square(10.0, 10.0, 10.0));
        let m = measure_object(&[p]);
        assert!((m.outer_area - (900.0 + 100.0)).abs() < 1e-9);
        assert!((m.hole_area - 400.0).abs() < 1e-9);
        assert!((m.area - 600.0).abs() < 1e-9);
        assert!((m.length - (120.0 + 80.0 + 40.0)).abs() < 1e-9);
    }

    #[test]
    fn open_lines_have_length_only() {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((3.0, 4.0));
        let m = measure_object(&[p]);
        assert!((m.length - 5.0).abs() < 1e-12);
        assert_eq!(m.area, 0.0);
        assert_eq!(m.open_paths, 1);
    }

    #[test]
    fn objects_add_up_and_share_a_bounding_box() {
        let m = measure(&[vec![square(0.0, 0.0, 10.0)], vec![square(20.0, 0.0, 10.0)]]);
        assert_eq!(m.objects, 2);
        assert!((m.area - 200.0).abs() < 1e-9);
        assert!((m.bbox_area - 300.0).abs() < 1e-9);
    }

    #[test]
    fn empty_input_measures_nothing() {
        let m = measure(&[]);
        assert_eq!(m, Measurement::default());
        let m = measure_object(&[BezPath::new()]);
        assert_eq!(m.area, 0.0);
    }
}
