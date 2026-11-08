//! Places where outlines cross: an outline crossing itself (a figure
//! eight, a loop at a sharp node) cuts badly and fails offsets, and two
//! objects crossing each other usually want a weld before cutting.
//!
//! Outlines are flattened to lines within `tol / 4`; neighbouring
//! segments of the same subpath, which meet at their shared node, do not
//! count. Lines that run along each other (an overlap, see
//! [`crate::cleanup`]) do not count either.

use crate::subpaths;
use tracedraw_core::geometry::{BezPath, Point};

/// One crossing: where, and which objects (the same index twice for an
/// outline crossing itself).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crossing {
    pub point: Point,
    pub a: usize,
    pub b: usize,
}

struct Seg {
    p0: Point,
    p1: Point,
    obj: usize,
    sub: usize,
    idx: usize,
    sub_len: usize,
    closed: bool,
    min_x: f64,
    max_x: f64,
}

/// The crossings of the objects' outlines: each object's own, and with
/// `between` also those between different objects. Crossings closer than
/// `tol` to one already found for the same objects are one.
pub fn find_crossings(objects: &[Vec<BezPath>], between: bool, tol: f64) -> Vec<Crossing> {
    let tol = tol.max(1e-6);
    let mut segs: Vec<Seg> = Vec::new();
    let mut sub_id = 0usize;
    for (obj, paths) in objects.iter().enumerate() {
        for path in paths {
            for sub in subpaths(path) {
                let mut pts: Vec<Point> = Vec::new();
                tracedraw_core::geometry::flatten(&sub.path, tol * 0.25, &mut |el| {
                    if let Some(p) = el.end_point() {
                        if pts.last().is_none_or(|q: &Point| (*q - p).hypot() > 1e-9) {
                            pts.push(p);
                        }
                    }
                });
                if sub.closed {
                    if let (Some(a), Some(b)) = (pts.first().copied(), pts.last().copied()) {
                        if (a - b).hypot() > 1e-9 {
                            pts.push(a);
                        }
                    }
                }
                if !pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()) {
                    continue;
                }
                let n = pts.len().saturating_sub(1);
                for (k, w) in pts.windows(2).enumerate() {
                    segs.push(Seg {
                        p0: w[0],
                        p1: w[1],
                        obj,
                        sub: sub_id,
                        idx: k,
                        sub_len: n,
                        closed: sub.closed,
                        min_x: w[0].x.min(w[1].x),
                        max_x: w[0].x.max(w[1].x),
                    });
                }
                sub_id += 1;
            }
        }
    }
    segs.sort_by(|a, b| a.min_x.total_cmp(&b.min_x));
    let mut found: Vec<Crossing> = Vec::new();
    for i in 0..segs.len() {
        let s = &segs[i];
        for t in &segs[i + 1..] {
            if t.min_x > s.max_x + 1e-9 {
                break;
            }
            if s.obj != t.obj && !between {
                continue;
            }
            if neighbours(s, t) {
                continue;
            }
            let Some(p) = intersect(s.p0, s.p1, t.p0, t.p1) else {
                continue;
            };
            let (a, b) = if s.obj <= t.obj {
                (s.obj, t.obj)
            } else {
                (t.obj, s.obj)
            };
            if found
                .iter()
                .any(|c| c.a == a && c.b == b && (c.point - p).hypot() < tol)
            {
                continue;
            }
            found.push(Crossing { point: p, a, b });
        }
    }
    found
}

/// Segments that follow each other in one subpath (the last and first of
/// a closed one too).
fn neighbours(s: &Seg, t: &Seg) -> bool {
    if s.sub != t.sub {
        return false;
    }
    let (i, j) = if s.idx < t.idx {
        (s.idx, t.idx)
    } else {
        (t.idx, s.idx)
    };
    j == i + 1 || (s.closed && i == 0 && j + 1 == s.sub_len)
}

/// Where two segments meet, ends included; `None` when they are parallel.
fn intersect(a0: Point, a1: Point, b0: Point, b1: Point) -> Option<Point> {
    let r = a1 - a0;
    let s = b1 - b0;
    let den = r.cross(s);
    let scale = r.hypot() * s.hypot();
    if scale < 1e-18 || den.abs() <= 1e-12 * scale {
        return None;
    }
    let q = b0 - a0;
    let t = q.cross(s) / den;
    let u = q.cross(r) / den;
    const E: f64 = 1e-9;
    if (-E..=1.0 + E).contains(&t) && (-E..=1.0 + E).contains(&u) {
        Some(a0 + r * t)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Rect, Shape as _};

    #[test]
    fn a_figure_eight_crosses_itself_once() {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((10.0, 10.0));
        p.line_to((10.0, 0.0));
        p.line_to((0.0, 10.0));
        p.close_path();
        let c = find_crossings(&[vec![p]], false, 0.01);
        assert_eq!(c.len(), 1);
        assert!((c[0].point - Point::new(5.0, 5.0)).hypot() < 1e-9);
        assert_eq!((c[0].a, c[0].b), (0, 0));
    }

    #[test]
    fn simple_shapes_do_not_cross_themselves() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.01);
        let e = tracedraw_core::geometry::Circle::new((30.0, 5.0), 5.0).to_path(0.01);
        assert!(find_crossings(&[vec![r], vec![e]], false, 0.01).is_empty());
    }

    #[test]
    fn overlapping_objects_cross_only_when_asked() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.01);
        let b = Rect::new(5.0, 5.0, 15.0, 15.0).to_path(0.01);
        let objects = [vec![a], vec![b]];
        assert!(find_crossings(&objects, false, 0.01).is_empty());
        let c = find_crossings(&objects, true, 0.01);
        assert_eq!(c.len(), 2);
        assert!(c.iter().all(|c| c.a == 0 && c.b == 1));
    }

    #[test]
    fn an_open_line_through_a_node_twice_touches_itself() {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((10.0, 0.0));
        p.line_to((10.0, 10.0));
        p.line_to((5.0, -5.0));
        let c = find_crossings(&[vec![p]], false, 0.01);
        assert_eq!(c.len(), 1);
    }
}
