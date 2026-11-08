//! Which objects a marquee (Pick tool) or a lasso (Freehand Pick) takes:
//! those lying wholly inside it, or with Alt every object it touches. See
//! docs/behavior/pick-tool.md and docs/behavior/free-transform.md.

use tracedraw_core::geometry::{rect_path, BezPath, PathEl, Point, Rect, Shape as _};
use tracedraw_core::{Shape, ShapeKind};

/// Flattening tolerance for outlines, page millimetres.
const FLAT_MM: f64 = 0.05;

/// The region a selection drag drew.
#[derive(Debug, Clone, Copy)]
pub enum Region<'a> {
    Rect(Rect),
    /// A closed polygon (the lasso), even-odd.
    Polygon(&'a [Point]),
}

impl Region<'_> {
    fn contains(&self, p: Point) -> bool {
        match self {
            Region::Rect(r) => p.x >= r.x0 && p.x <= r.x1 && p.y >= r.y0 && p.y <= r.y1,
            Region::Polygon(pts) => point_in_polygon(pts, p),
        }
    }

    fn bounds(&self) -> Rect {
        match self {
            Region::Rect(r) => *r,
            Region::Polygon(pts) => pts
                .iter()
                .fold(None::<Rect>, |acc, p| {
                    let r = Rect::from_points(*p, *p);
                    Some(acc.map_or(r, |a| a.union(r)))
                })
                .unwrap_or(Rect::ZERO),
        }
    }

    /// The region's edges, closed.
    fn edges(&self) -> Vec<(Point, Point)> {
        let pts: Vec<Point> = match self {
            Region::Rect(r) => vec![
                Point::new(r.x0, r.y0),
                Point::new(r.x1, r.y0),
                Point::new(r.x1, r.y1),
                Point::new(r.x0, r.y1),
            ],
            Region::Polygon(pts) => pts.to_vec(),
        };
        let n = pts.len();
        (0..n).map(|i| (pts[i], pts[(i + 1) % n])).collect()
    }

    /// A point of the region, to tell whether the region lies in a fill.
    fn sample(&self) -> Option<Point> {
        match self {
            Region::Rect(r) => Some(r.center()),
            Region::Polygon(pts) => pts.first().copied(),
        }
    }
}

/// Even-odd point in polygon.
pub fn point_in_polygon(pts: &[Point], p: Point) -> bool {
    let n = pts.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (pts[i], pts[j]);
        if (a.y > p.y) != (b.y > p.y) {
            let x = (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x;
            if p.x < x {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// The flattened subpaths of `path`, each closed one ending on its first
/// point.
fn polylines(path: &BezPath) -> Vec<Vec<Point>> {
    let mut out: Vec<Vec<Point>> = Vec::new();
    let mut cur: Vec<Point> = Vec::new();
    tracedraw_core::geometry::flatten(path, FLAT_MM, &mut |el| match el {
        PathEl::MoveTo(p) => {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            cur.push(p);
        }
        PathEl::LineTo(p) => cur.push(p),
        PathEl::ClosePath => {
            if let Some(first) = cur.first().copied() {
                cur.push(first);
            }
        }
        _ => {}
    });
    if !cur.is_empty() {
        out.push(cur);
    }
    out.retain(|l| l.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
    out
}

/// The segments `a`-`b` and `c`-`d` meet (touching ends count).
fn segments_meet(a: Point, b: Point, c: Point, d: Point) -> bool {
    let cross =
        |o: Point, p: Point, q: Point| (p.x - o.x) * (q.y - o.y) - (p.y - o.y) * (q.x - o.x);
    let on = |o: Point, p: Point, q: Point| {
        q.x >= o.x.min(p.x) && q.x <= o.x.max(p.x) && q.y >= o.y.min(p.y) && q.y <= o.y.max(p.y)
    };
    let (d1, d2) = (cross(c, d, a), cross(c, d, b));
    let (d3, d4) = (cross(a, b, c), cross(a, b, d));
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    (d1 == 0.0 && on(c, d, a))
        || (d2 == 0.0 && on(c, d, b))
        || (d3 == 0.0 && on(a, b, c))
        || (d4 == 0.0 && on(a, b, d))
}

fn overlap(a: Rect, b: Rect) -> bool {
    a.x0 <= b.x1 && b.x0 <= a.x1 && a.y0 <= b.y1 && b.y0 <= a.y1
}

/// Kinds that count anywhere inside their box.
fn boxed(s: &Shape) -> bool {
    matches!(
        s.kind,
        ShapeKind::Text { .. }
            | ShapeKind::Table(_)
            | ShapeKind::Bitmap { .. }
            | ShapeKind::ClipFrame { .. }
            | ShapeKind::SymbolInstance { .. }
    )
}

/// The children of a group (`g` in page space), each in page space.
fn children_on_page(g: &Shape) -> Vec<Shape> {
    match &g.kind {
        ShapeKind::Group { children } => children
            .iter()
            .filter(|c| c.visible)
            .map(|c| {
                let mut c = c.clone();
                c.transform_by(g.transform);
                c
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The region takes `s` (in page space): wholly inside it, or with
/// `touching` when it crosses or holds part of the object's outline, or
/// lies in an object whose inside counts (`fill_counts`: filled, or
/// "Treat all objects as filled"; text, bitmaps and the like always).
pub fn takes(
    region: Region,
    s: &Shape,
    touching: bool,
    fill_counts: &dyn Fn(&Shape) -> bool,
) -> bool {
    if !s.effects.is_empty() {
        // What the object draws: its warped outline and the objects its
        // effects add; wholly inside means all of them.
        let ev = tracedraw_core::live::evaluate(s);
        let parts: Vec<Shape> = std::iter::once(ev.main)
            .chain(ev.below)
            .chain(ev.above)
            .map(|mut part| {
                part.effects.clear();
                part
            })
            .collect();
        return if touching {
            parts.iter().any(|q| takes(region, q, true, fill_counts))
        } else {
            parts.iter().all(|q| takes(region, q, false, fill_counts))
        };
    }
    let b = s.bounds();
    if !b.x0.is_finite() || !b.y0.is_finite() || !b.x1.is_finite() || !b.y1.is_finite() {
        return false;
    }
    if !touching {
        return match region {
            Region::Rect(r) => r.contains_rect(b),
            Region::Polygon(_) => {
                if matches!(s.kind, ShapeKind::Group { .. }) {
                    let kids = children_on_page(s);
                    return !kids.is_empty()
                        && kids.iter().all(|c| takes(region, c, false, fill_counts));
                }
                let outline = if boxed(s) {
                    rect_path(b, 0.0)
                } else {
                    s.page_path()
                };
                let lines = polylines(&outline);
                !lines.is_empty() && lines.iter().flatten().all(|p| region.contains(*p))
            }
        };
    }
    if !overlap(region.bounds(), b) {
        return false;
    }
    if matches!(s.kind, ShapeKind::Group { .. }) {
        return children_on_page(s)
            .iter()
            .any(|c| takes(region, c, true, fill_counts));
    }
    let (outline, inside) = if boxed(s) {
        (rect_path(b, 0.0), true)
    } else {
        (s.page_path(), fill_counts(s))
    };
    let lines = polylines(&outline);
    if lines.iter().flatten().any(|p| region.contains(*p)) {
        return true;
    }
    let edges = region.edges();
    for line in &lines {
        for seg in line.windows(2) {
            if edges
                .iter()
                .any(|(c, d)| segments_meet(seg[0], seg[1], *c, *d))
            {
                return true;
            }
        }
    }
    // The whole region lies inside the object.
    inside && region.sample().is_some_and(|p| outline.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{Color, Fill, ShapeId};

    fn rect_shape(id: u64, r: Rect, filled: bool) -> Shape {
        let mut s = Shape::new(
            ShapeId(id),
            ShapeKind::Rect {
                rect: r,
                radius: 0.0,
                corners: None,
            },
        );
        if filled {
            s.fill = Fill::Solid(Color::rgb8(255, 0, 0));
        }
        s
    }

    fn filled(s: &Shape) -> bool {
        !matches!(s.fill, Fill::None)
    }

    #[test]
    fn a_marquee_takes_what_is_inside_or_with_alt_what_it_touches() {
        let s = rect_shape(1, Rect::new(10.0, 10.0, 30.0, 30.0), false);
        let around = Region::Rect(Rect::new(0.0, 0.0, 40.0, 40.0));
        let across = Region::Rect(Rect::new(20.0, 0.0, 40.0, 20.0));
        let inside = Region::Rect(Rect::new(15.0, 15.0, 25.0, 25.0));
        let apart = Region::Rect(Rect::new(50.0, 50.0, 60.0, 60.0));
        assert!(takes(around, &s, false, &filled));
        assert!(!takes(across, &s, false, &filled));
        assert!(takes(across, &s, true, &filled));
        assert!(!takes(apart, &s, true, &filled));
        // Inside an outline-only rectangle nothing is touched; inside a
        // filled one the fill is.
        assert!(!takes(inside, &s, true, &filled));
        let f = rect_shape(2, Rect::new(10.0, 10.0, 30.0, 30.0), true);
        assert!(takes(inside, &f, true, &filled));
    }

    #[test]
    fn a_lasso_takes_objects_wholly_inside_or_touched() {
        let s = rect_shape(1, Rect::new(10.0, 10.0, 20.0, 20.0), false);
        // A diamond around the square.
        let diamond = [
            Point::new(15.0, -5.0),
            Point::new(35.0, 15.0),
            Point::new(15.0, 35.0),
            Point::new(-5.0, 15.0),
        ];
        assert!(takes(Region::Polygon(&diamond), &s, false, &filled));
        // A triangle over its right half.
        let half = [
            Point::new(15.0, 0.0),
            Point::new(40.0, 0.0),
            Point::new(40.0, 30.0),
            Point::new(15.0, 30.0),
        ];
        assert!(!takes(Region::Polygon(&half), &s, false, &filled));
        assert!(takes(Region::Polygon(&half), &s, true, &filled));
        // Degenerate lassos take nothing.
        assert!(!takes(Region::Polygon(&diamond[..2]), &s, false, &filled));
    }

    #[test]
    fn groups_count_by_their_objects() {
        let a = rect_shape(1, Rect::new(0.0, 0.0, 10.0, 10.0), false);
        let b = rect_shape(2, Rect::new(30.0, 0.0, 40.0, 10.0), false);
        let g = Shape::new(
            ShapeId(3),
            ShapeKind::Group {
                children: vec![a, b],
            },
        );
        // Between the two rectangles: inside the group's box, touching
        // neither.
        let gap = Region::Rect(Rect::new(15.0, 2.0, 25.0, 8.0));
        assert!(!takes(gap, &g, true, &filled));
        let over_b = Region::Rect(Rect::new(35.0, -5.0, 50.0, 5.0));
        assert!(takes(over_b, &g, true, &filled));
        assert!(!takes(over_b, &g, false, &filled));
    }

    #[test]
    fn crossing_segments_meet() {
        let p = Point::new;
        assert!(segments_meet(
            p(0.0, 0.0),
            p(10.0, 10.0),
            p(0.0, 10.0),
            p(10.0, 0.0)
        ));
        assert!(!segments_meet(
            p(0.0, 0.0),
            p(1.0, 1.0),
            p(5.0, 0.0),
            p(6.0, 1.0)
        ));
        // Touching ends.
        assert!(segments_meet(
            p(0.0, 0.0),
            p(5.0, 0.0),
            p(5.0, 0.0),
            p(5.0, 5.0)
        ));
    }
}
