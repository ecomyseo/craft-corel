//! Cutting: plotter output, weeding, bridges, cut order, registration and crop marks.
//!
//! Pure geometry over page-space outlines in millimetres, y up. The
//! application turns the results into commands
//! (TraceCut's Cut tools).
//!
//! - [`weed`]: weeding lines, weeding boxes and the weeding frame.
//! - [`bridges`]: uncut bridges (tabs) on closed outlines.
//! - [`order`]: cut order, start points and directions (route optimizer).
//! - [`hpgl`]: plotter output with knife offset compensation and overcut.
//! - [`marks`]: registration marks for print and cut, and crop marks.

pub mod bridges;
pub mod hpgl;
pub mod marks;
pub mod order;
pub mod weed;

use tracedraw_core::geometry::{BezPath, PathEl, Point, Rect, Shape as _};

/// Tolerance used to flatten curves into polygons, in millimetres.
pub const FLATTEN_TOLERANCE: f64 = 0.02;

/// A flattened subpath: its points (a closed one does not repeat its
/// first point at the end) and whether it closes.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Poly {
    pub pts: Vec<Point>,
    pub closed: bool,
}

/// True when both coordinates are finite.
pub(crate) fn finite(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

/// True when every point of the path is finite.
pub(crate) fn finite_path(path: &BezPath) -> bool {
    path.elements().iter().all(|el| match *el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => finite(p),
        PathEl::QuadTo(a, p) => finite(a) && finite(p),
        PathEl::CurveTo(a, b, p) => finite(a) && finite(b) && finite(p),
        PathEl::ClosePath => true,
    })
}

/// `v` when it is finite, else `fallback`.
pub(crate) fn fin(v: f64, fallback: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// True when every side of the rectangle is finite.
pub(crate) fn finite_rect(r: Rect) -> bool {
    r.x0.is_finite() && r.y0.is_finite() && r.x1.is_finite() && r.y1.is_finite()
}

/// Flatten a path into polygons, one per subpath with segments. Subpaths
/// with a non-finite point are left out; repeated points are dropped.
pub(crate) fn polys(path: &BezPath, tolerance: f64) -> Vec<Poly> {
    let mut out = Vec::new();
    for sp in crate::subpaths(path) {
        if !finite_path(&sp.path) {
            continue;
        }
        let mut pts: Vec<Point> = Vec::new();
        tracedraw_core::geometry::flatten(&sp.path, tolerance, &mut |el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p)
                if pts.last().is_none_or(|q| (*q - p).hypot() > 1e-9) =>
            {
                pts.push(p);
            }
            _ => {}
        });
        if sp.closed && pts.len() > 1 {
            if let (Some(a), Some(b)) = (pts.first(), pts.last()) {
                if (*a - *b).hypot() <= 1e-9 {
                    pts.pop();
                }
            }
        }
        if pts.len() >= 2 {
            out.push(Poly {
                pts,
                closed: sp.closed,
            });
        }
    }
    out
}

/// The bounding box of the finite paths, if any.
pub(crate) fn bounds(paths: &[BezPath]) -> Option<Rect> {
    paths
        .iter()
        .filter(|p| finite_path(p) && !p.elements().is_empty())
        .map(|p| p.bounding_box())
        .filter(|b| finite_rect(*b))
        .reduce(|a, b| a.union(b))
}

/// A closed rectangle path, counter-clockwise from the bottom-left corner.
pub(crate) fn rect_path(r: Rect) -> BezPath {
    let mut p = BezPath::new();
    p.move_to((r.x0, r.y0));
    p.line_to((r.x1, r.y0));
    p.line_to((r.x1, r.y1));
    p.line_to((r.x0, r.y1));
    p.close_path();
    p
}

/// An open straight line.
pub(crate) fn line_path(a: Point, b: Point) -> BezPath {
    let mut p = BezPath::new();
    p.move_to(a);
    p.line_to(b);
    p
}

/// A closed polygon path through `pts`.
pub(crate) fn polygon_path(pts: &[Point]) -> BezPath {
    let mut p = BezPath::new();
    for (i, q) in pts.iter().enumerate() {
        if i == 0 {
            p.move_to(*q);
        } else {
            p.line_to(*q);
        }
    }
    if !pts.is_empty() {
        p.close_path();
    }
    p
}

/// Signed area of a polygon (positive: counter-clockwise, y up).
pub(crate) fn signed_area(pts: &[Point]) -> f64 {
    let n = pts.len();
    if n < 3 {
        return 0.0;
    }
    let mut a = 0.0;
    for i in 0..n {
        let p = pts[i];
        let q = pts[(i + 1) % n];
        a += p.x * q.y - q.x * p.y;
    }
    a / 2.0
}

/// True when `p` lies inside the polygon (even-odd, boundary undecided).
pub(crate) fn inside(p: Point, pts: &[Point]) -> bool {
    let n = pts.len();
    if n < 3 {
        return false;
    }
    let mut c = false;
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (pts[i], pts[j]);
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x {
                c = !c;
            }
        }
        j = i;
    }
    c
}

/// The bounding box of a polygon.
pub(crate) fn poly_bounds(pts: &[Point]) -> Option<Rect> {
    let first = pts.first()?;
    let mut r = Rect::from_points(*first, *first);
    for p in pts {
        r = r.union_pt(*p);
    }
    Some(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygons_drop_repeats_and_non_finite_subpaths() {
        let mut p = rect_path(Rect::new(0.0, 0.0, 10.0, 5.0));
        p.move_to((f64::NAN, 0.0));
        p.line_to((1.0, 1.0));
        let ps = polys(&p, FLATTEN_TOLERANCE);
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].pts.len(), 4);
        assert!(ps[0].closed);
        assert!((signed_area(&ps[0].pts) - 50.0).abs() < 1e-9);
        assert!(inside(Point::new(5.0, 2.0), &ps[0].pts));
        assert!(!inside(Point::new(15.0, 2.0), &ps[0].pts));
        assert!(bounds(&[BezPath::new()]).is_none());
    }
}
