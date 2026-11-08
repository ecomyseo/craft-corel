//! Geometry primitives. We re-export kurbo so the rest of the workspace
//! shares one set of types, and add the few helpers the editor needs.

pub use kurbo::{
    Affine, Arc, BezPath, Circle, CubicBez, Ellipse, Line, ParamCurve, ParamCurveArclen,
    ParamCurveDeriv, ParamCurveNearest, PathEl, PathSeg, Point, Rect, RoundedRect, Shape, Size,
    Vec2,
};

/// Flatten a path into lines within `tolerance`, calling `f` with each
/// move, line and close element.
pub fn flatten(path: &BezPath, tolerance: f64, f: &mut dyn FnMut(PathEl)) {
    kurbo::flatten(path.elements().iter().copied(), tolerance, f);
}

/// Build a closed rectangle path, optionally with rounded corners.
pub fn rect_path(rect: Rect, radius: f64) -> BezPath {
    if radius > 0.0 {
        RoundedRect::from_rect(rect, radius).to_path(0.01)
    } else {
        rect.to_path(0.01)
    }
}

/// Build a closed ellipse path inscribed in `rect`: four curves between
/// nodes at the right, top, left and bottom (in that order, starting at
/// the right), each node with equal handles along the tangent, and the
/// last curve ending on the start, so it is one closed curve of four
/// nodes when converted to curves (#625).
pub fn ellipse_path(rect: Rect) -> BezPath {
    // Handle length of a quarter circle as a cubic, as a share of the
    // radius: 4/3 (sqrt 2 - 1).
    const K: f64 = 0.552_284_749_830_793_4;
    let c = rect.center();
    let (rx, ry) = (rect.width() / 2.0, rect.height() / 2.0);
    let (kx, ky) = (rx * K, ry * K);
    let right = Point::new(c.x + rx, c.y);
    let top = Point::new(c.x, c.y + ry);
    let left = Point::new(c.x - rx, c.y);
    let bottom = Point::new(c.x, c.y - ry);
    let mut path = BezPath::new();
    path.move_to(right);
    let at = |x: f64, y: f64| Point::new(x, y);
    path.curve_to(at(right.x, right.y + ky), at(top.x + kx, top.y), top);
    path.curve_to(at(top.x - kx, top.y), at(left.x, left.y + ky), left);
    path.curve_to(at(left.x, left.y - ky), at(bottom.x - kx, bottom.y), bottom);
    path.curve_to(
        at(bottom.x + kx, bottom.y),
        at(right.x, right.y - ky),
        right,
    );
    path.close_path();
    path
}

/// Arc or pie of the ellipse inscribed in `rect`, angles in degrees CCW from +x.
pub fn ellipse_arc_path(rect: Rect, start_deg: f64, end_deg: f64, pie: bool) -> BezPath {
    let c = rect.center();
    let radii = kurbo::Vec2::new(rect.width() / 2.0, rect.height() / 2.0);
    let mut sweep = (end_deg - start_deg).rem_euclid(360.0);
    if sweep == 0.0 {
        sweep = 360.0;
    }
    let arc = kurbo::Arc::new(c, radii, start_deg.to_radians(), sweep.to_radians(), 0.0);
    let mut path = BezPath::new();
    let start = Point::new(
        c.x + radii.x * start_deg.to_radians().cos(),
        c.y + radii.y * start_deg.to_radians().sin(),
    );
    if pie {
        path.move_to(c);
        path.line_to(start);
    } else {
        path.move_to(start);
    }
    arc.append_iter(0.1).for_each(|el| path.push(el));
    if pie {
        path.close_path();
    }
    path
}

/// Build a regular polygon or star inscribed in the ellipse that fills
/// `rect`, with its first vertex at the top.
///
/// `points` is the number of vertices; `sharpness` in `0.0..1.0` pulls every
/// other vertex toward the centre, turning a polygon into a star, and a
/// negative one puts those points past the centre: an inverted star (see
/// [`star_inner_ratio`]).
pub fn polygon_path(rect: Rect, points: u32, sharpness: f64) -> BezPath {
    twisted_polygon_path(rect, points, sharpness, 0.0)
}

/// A polygon or star whose points between the vertices are turned by
/// `twist` radians (counter-clockwise, within half a turn) about the
/// centre: a twisted star. A polygon without sharpness gains points
/// between its vertices, where the middles of its sides were.
pub fn twisted_polygon_path(rect: Rect, points: u32, sharpness: f64, twist: f64) -> BezPath {
    let c = rect.center();
    let rx = rect.width() / 2.0;
    let ry = rect.height() / 2.0;
    let mut path = BezPath::new();
    for (i, (x, y)) in polygon_unit_vertices(points, sharpness, twist).enumerate() {
        let p = Point::new(c.x + rx * x, c.y + ry * y);
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close_path();
    path
}

/// Whether `sharpness` makes a polygon a star: any finite value but 0
/// (a negative one an inverted star).
pub fn is_star_sharpness(sharpness: f64) -> bool {
    sharpness.is_finite() && sharpness != 0.0
}

/// The distance of a star's inner points from its centre, as a part of
/// the vertices' distance, for a star `sharpness` (not 0). From 0 to 1
/// the points come in from the rim (`1 - sharpness`) to the centre; from
/// 0 to -1 they lie past the centre, on the far side (#1331): `-0.5`
/// puts them half way out on the opposite side, `-1` at the centre again.
/// The ratio is then negative: `-(1 - |sharpness|)`.
pub fn star_inner_ratio(sharpness: f64) -> f64 {
    if !sharpness.is_finite() {
        return 1.0;
    }
    let s = sharpness.clamp(-1.0, 1.0);
    if s < 0.0 {
        -(1.0 + s)
    } else {
        1.0 - s
    }
}

/// The star sharpness whose inner points sit at `ratio` of the vertices'
/// distance from the centre, negative past it: the inverse of
/// [`star_inner_ratio`], kept a star (its size between 0.001 and 0.999)
/// on either side.
pub fn star_sharpness_for_ratio(ratio: f64) -> f64 {
    let r = if ratio.is_finite() { ratio } else { 0.0 };
    if r < 0.0 {
        -(1.0 + r).clamp(0.001, 0.999)
    } else {
        (1.0 - r).clamp(0.001, 0.999)
    }
}

/// The fewest points a complex star has.
pub const COMPLEX_STAR_MIN_POINTS: u32 = 5;

/// The largest sharpness a complex star of `points` points takes: its
/// sides join vertices at most (points - 1) / 2 apart.
pub fn complex_star_max_sharpness(points: u32) -> u32 {
    let n = points.max(COMPLEX_STAR_MIN_POINTS);
    ((n - 1) / 2).saturating_sub(1).max(1)
}

/// A complex star: `points` vertices on the ellipse in `rect` (the first
/// at the top, then clockwise), each joined to the one `sharpness + 1`
/// further on, so the sides cross. When that step shares a factor with
/// the count the star is several closed subpaths (two triangles for six
/// points). Filled even-odd, the middle stays empty.
pub fn complex_star_path(rect: Rect, points: u32, sharpness: u32) -> BezPath {
    let n = points.clamp(COMPLEX_STAR_MIN_POINTS, 500) as usize;
    let step = sharpness.clamp(1, complex_star_max_sharpness(n as u32)) as usize + 1;
    let verts: Vec<Point> = {
        let c = rect.center();
        let (rx, ry) = (rect.width() / 2.0, rect.height() / 2.0);
        polygon_unit_vertices(n as u32, 0.0, 0.0)
            .map(|(x, y)| Point::new(c.x + rx * x, c.y + ry * y))
            .collect()
    };
    let g = gcd(n, step);
    let mut path = BezPath::new();
    for start in 0..g {
        let mut i = start;
        path.move_to(verts[i]);
        for _ in 1..n / g {
            i = (i + step) % n;
            path.line_to(verts[i]);
        }
        path.close_path();
    }
    path
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a.max(1)
    } else {
        gcd(b, a % b)
    }
}

/// Vertices of a polygon or star on the unit circle, first at the top
/// (y up), then clockwise.
fn polygon_unit_vertices(
    points: u32,
    sharpness: f64,
    twist: f64,
) -> impl Iterator<Item = (f64, f64)> {
    let points = points.max(3) as usize;
    let twist = if twist.is_finite() {
        twist.clamp(-std::f64::consts::PI, std::f64::consts::PI)
    } else {
        0.0
    };
    let star = is_star_sharpness(sharpness) || twist != 0.0;
    // A plain polygon turned keeps its sides' middles as the inner points.
    let inner = if is_star_sharpness(sharpness) {
        star_inner_ratio(sharpness)
    } else {
        (std::f64::consts::PI / points as f64).cos()
    };
    let n = if star { points * 2 } else { points };
    (0..n).map(move |i| {
        let mut t = std::f64::consts::FRAC_PI_2 - i as f64 * std::f64::consts::TAU / n as f64;
        let k = if star && i % 2 == 1 {
            t += twist;
            inner
        } else {
            1.0
        };
        (k * t.cos(), k * t.sin())
    })
}

/// The ellipse rectangle (the `rect` of a polygon shape) whose polygon or
/// star exactly fills `bounds`. Drawing tools drag the visible box of the
/// polygon; the model keeps the ellipse its outer vertices lie on.
pub fn polygon_rect_for_bounds(bounds: Rect, points: u32, sharpness: f64) -> Rect {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (x, y) in polygon_unit_vertices(points, sharpness, 0.0) {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
    }
    let rx = bounds.width() / (x1 - x0).max(1e-9);
    let ry = bounds.height() / (y1 - y0).max(1e-9);
    let cx = bounds.x0 - x0 * rx;
    let cy = bounds.y0 - y0 * ry;
    Rect::new(cx - rx, cy - ry, cx + rx, cy + ry)
}

/// Straight segments through the points.
pub fn polyline_path(points: &[Point], closed: bool) -> BezPath {
    let mut path = BezPath::new();
    for (i, p) in points.iter().enumerate() {
        if i == 0 {
            path.move_to(*p);
        } else {
            path.line_to(*p);
        }
    }
    if closed && points.len() > 2 {
        path.close_path();
    }
    path
}

/// Smooth cubic curve through the points (Catmull-Rom converted to Bezier),
/// the way a freehand stroke is turned into a curve. A closed curve wraps
/// around: the last point joins the first with a curve too, and the
/// tangents run on through the join, so it has no corner there. A closing
/// point equal to the first is dropped.
pub fn smooth_path(points: &[Point], closed: bool) -> BezPath {
    let mut pts = points;
    if closed && pts.len() > 3 {
        if let (Some(first), Some(last)) = (pts.first(), pts.last()) {
            if (*first - *last).hypot() < 1e-9 {
                pts = &pts[..pts.len() - 1];
            }
        }
    }
    let n = pts.len();
    if n < 3 {
        return polyline_path(pts, closed);
    }
    let mut path = BezPath::new();
    path.move_to(pts[0]);
    let get = |i: isize| -> Point {
        let idx = if closed {
            i.rem_euclid(n as isize) as usize
        } else {
            i.clamp(0, n as isize - 1) as usize
        };
        pts[idx]
    };
    let segments = if closed { n } else { n - 1 };
    // The tangent at a point runs from the point before to the one after
    // and the handles along it are a third of the segment long, so a long
    // segment next to a short one neither overshoots nor flattens.
    let handle = |from: Point, along: Vec2, len: f64| -> Point {
        let l = along.hypot();
        if l < 1e-12 {
            from
        } else {
            from + along * (len / (3.0 * l))
        }
    };
    for i in 0..segments {
        let p0 = get(i as isize - 1);
        let p1 = get(i as isize);
        let p2 = get(i as isize + 1);
        let p3 = get(i as isize + 2);
        let len = (p2 - p1).hypot();
        let c1 = handle(p1, p2 - p0, len);
        let c2 = handle(p2, p1 - p3, len);
        path.curve_to(c1, c2, p2);
    }
    if closed {
        path.close_path();
    }
    path
}

/// Drop points closer than `tolerance` to the line between their
/// neighbours (Ramer-Douglas-Peucker), used to thin freehand input.
pub fn simplify(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }
    fn rdp(pts: &[Point], tol: f64, out: &mut Vec<Point>) {
        let (a, b) = (pts[0], pts[pts.len() - 1]);
        let ab = b - a;
        let len2 = ab.hypot2();
        let mut max_d = 0.0;
        let mut idx = 0;
        for (i, p) in pts.iter().enumerate().skip(1).take(pts.len() - 2) {
            let d = if len2 < 1e-12 {
                (*p - a).hypot()
            } else {
                let t = ((*p - a).dot(ab) / len2).clamp(0.0, 1.0);
                (*p - (a + ab * t)).hypot()
            };
            if d > max_d {
                max_d = d;
                idx = i;
            }
        }
        if max_d > tol && idx > 0 {
            rdp(&pts[..=idx], tol, out);
            out.pop();
            rdp(&pts[idx..], tol, out);
        } else {
            out.push(a);
            out.push(b);
        }
    }
    let mut out = Vec::new();
    rdp(points, tolerance, &mut out);
    out
}

/// Axis-aligned bounds of a path after an affine transform.
pub fn transformed_bounds(path: &BezPath, transform: Affine) -> Rect {
    (transform * path.clone()).bounding_box()
}

/// A cubic B-spline over the control points `points` as Bezier
/// segments (quadratic with three points, a line with two). Open: the
/// clamped uniform spline, which starts at the first point and ends at
/// the last, its ends tangent to the control polygon. Closed: the
/// periodic spline, smooth all round. Each knot span becomes one segment,
/// its control points the blossoms of the span's ends.
/// [`bspline_path`] with the control points `fixed` says (one flag a
/// point, missing ones floating) made fixed: the curve passes through a
/// fixed point, with a corner there between the lines of the control
/// polygon, as a point taken three times by the cubic spline does. The
/// ends of an open spline are fixed already (#1758).
pub fn bspline_path_fixed(points: &[Point], fixed: &[bool], closed: bool) -> BezPath {
    if !fixed.contains(&true) {
        return bspline_path(points, closed);
    }
    let n = points.len();
    let mut all = Vec::with_capacity(n + 2 * fixed.len());
    for (i, p) in points.iter().enumerate() {
        let end = !closed && (i == 0 || i + 1 == n);
        let times = if fixed.get(i).copied().unwrap_or(false) && !end {
            3
        } else {
            1
        };
        all.extend(std::iter::repeat_n(*p, times));
    }
    bspline_path(&all, closed)
}

pub fn bspline_path(points: &[Point], closed: bool) -> BezPath {
    let mut path = BezPath::new();
    let n = points.len();
    if n == 0 {
        return path;
    }
    if n == 1 {
        path.move_to(points[0]);
        return path;
    }
    let deg = if closed { 3.min(n) } else { 3.min(n - 1) };
    if deg <= 1 {
        path.move_to(points[0]);
        for p in &points[1..] {
            path.line_to(*p);
        }
        if closed {
            path.close_path();
        }
        return path;
    }
    // Control points and knots.
    let (ctrl, knots): (Vec<Point>, Vec<f64>) = if closed {
        let mut c = points.to_vec();
        c.extend_from_slice(&points[..deg]);
        let k = (0..c.len() + deg + 1).map(|i| i as f64).collect();
        (c, k)
    } else {
        let mut k = vec![0.0; deg + 1];
        k.extend((1..n - deg).map(|i| i as f64));
        k.extend(std::iter::repeat_n((n - deg) as f64, deg + 1));
        (points.to_vec(), k)
    };
    let spans: Vec<usize> = if closed {
        (deg..deg + n).collect()
    } else {
        (deg..n).collect()
    };
    // The blossom f(us) on span `k` (de Boor with one parameter a level).
    let blossom = |k: usize, us: &[f64]| -> Point {
        let mut d: Vec<Point> = (0..=deg).map(|j| ctrl[j + k - deg]).collect();
        for r in 1..=deg {
            for j in (r..=deg).rev() {
                let lo = knots[j + k - deg];
                let hi = knots[j + 1 + k - r];
                let a = if hi > lo {
                    (us[r - 1] - lo) / (hi - lo)
                } else {
                    0.0
                };
                d[j] = d[j - 1].lerp(d[j], a);
            }
        }
        d[deg]
    };
    let mut first = true;
    for k in spans {
        let (a, b) = (knots[k], knots[k + 1]);
        if b <= a {
            continue;
        }
        if deg == 3 {
            let p0 = blossom(k, &[a, a, a]);
            if first {
                path.move_to(p0);
                first = false;
            }
            path.curve_to(
                blossom(k, &[a, a, b]),
                blossom(k, &[a, b, b]),
                blossom(k, &[b, b, b]),
            );
        } else {
            let p0 = blossom(k, &[a, a]);
            if first {
                path.move_to(p0);
                first = false;
            }
            path.quad_to(blossom(k, &[a, b]), blossom(k, &[b, b]));
        }
    }
    if closed {
        path.close_path();
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The corner points of a closed polygon path, in order.
    fn corners(p: &BezPath) -> Vec<Point> {
        p.elements()
            .iter()
            .filter_map(|e| match e {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
                _ => None,
            })
            .collect()
    }

    /// An ellipse is one closed curve of four nodes at the right, top,
    /// left and bottom, each with equal handles, whatever its size (#625).
    #[test]
    fn an_ellipse_is_closed_with_four_nodes_on_its_axes() {
        for rect in [
            Rect::new(10.0, 20.0, 70.0, 50.0),
            Rect::new(0.0, 0.0, 400.0, 900.0),
            Rect::new(-3.0, -3.0, 3.0, 3.0),
        ] {
            let p = ellipse_path(rect);
            assert_eq!(p.elements().last(), Some(&PathEl::ClosePath), "{p:?}");
            let nodes = crate::nodes::nodes(&p);
            assert_eq!(nodes.len(), 4, "{p:?}");
            let c = rect.center();
            let (rx, ry) = (rect.width() / 2.0, rect.height() / 2.0);
            let at: Vec<Point> = nodes.iter().map(|n| n.pos).collect();
            assert_eq!(
                at,
                vec![
                    Point::new(c.x + rx, c.y),
                    Point::new(c.x, c.y + ry),
                    Point::new(c.x - rx, c.y),
                    Point::new(c.x, c.y - ry),
                ]
            );
            for n in &nodes {
                let (i, o) = (n.ctrl_in.expect("in"), n.ctrl_out.expect("out"));
                assert!(((i - n.pos) + (o - n.pos)).hypot() < 1e-9, "{n:?}");
            }
            // On the ellipse within a hair everywhere.
            for seg in p.segments() {
                for k in 0..=10 {
                    let q = seg.eval(k as f64 / 10.0);
                    let r = ((q.x - c.x) / rx).powi(2) + ((q.y - c.y) / ry).powi(2);
                    assert!((r.sqrt() - 1.0).abs() < 3e-4, "{q:?}");
                }
            }
        }
    }

    /// A fixed control point is on the curve, a floating one is not; an
    /// open spline's ends are on it either way (#1758).
    #[test]
    fn a_fixed_bspline_point_is_on_the_curve() {
        let pts = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 20.0),
            Point::new(30.0, 20.0),
            Point::new(40.0, 0.0),
            Point::new(60.0, 10.0),
        ];
        let dist = |path: &BezPath, q: Point| {
            path.segments()
                .flat_map(|s| (0..=200).map(move |k| s.eval(k as f64 / 200.0)))
                .map(|p| (p - q).hypot())
                .fold(f64::INFINITY, f64::min)
        };
        let floating = bspline_path(&pts, false);
        assert!(dist(&floating, pts[2]) > 1.0);
        let fixed = bspline_path_fixed(&pts, &[false, false, true], false);
        assert!(dist(&fixed, pts[2]) < 1e-6, "through the fixed point");
        assert!(dist(&fixed, pts[1]) > 1.0, "not the floating one");
        assert!((fixed.segments().next().expect("seg").start() - pts[0]).hypot() < 1e-9);
        // No flag set: the plain spline.
        assert_eq!(bspline_path_fixed(&pts, &[false; 5], false), floating);
        // Closed too.
        let closed = bspline_path_fixed(&pts, &[false, true], true);
        assert!(dist(&closed, pts[1]) < 1e-6);
        assert!(dist(&bspline_path(&pts, true), pts[1]) > 1.0);
    }

    #[test]
    fn bsplines_start_and_end_on_their_ends_and_stay_inside() {
        let pts = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 20.0),
            Point::new(30.0, 20.0),
            Point::new(40.0, 0.0),
            Point::new(60.0, 10.0),
        ];
        let p = bspline_path(&pts, false);
        let segs: Vec<_> = p.segments().collect();
        assert_eq!(segs.len(), 2, "one cubic a knot span");
        assert!((segs[0].start() - pts[0]).hypot() < 1e-9);
        assert!((segs[1].end() - pts[4]).hypot() < 1e-9);
        // Smooth where the segments meet, and inside the control points' box.
        let b = p.bounding_box();
        assert!(b.x0 >= -1e-9 && b.x1 <= 60.0 + 1e-9 && b.y0 >= -1e-9 && b.y1 <= 20.0 + 1e-9);
        if let (PathSeg::Cubic(c0), PathSeg::Cubic(c1)) = (segs[0], segs[1]) {
            let t0 = c0.p3 - c0.p2;
            let t1 = c1.p1 - c1.p0;
            assert!(
                (t0.x * t1.y - t0.y * t1.x).abs() < 1e-6,
                "tangent continuity"
            );
        } else {
            panic!("cubic segments");
        }
        // Four points: one cubic with those control points.
        let p = bspline_path(&pts[..4], false);
        let segs: Vec<_> = p.segments().collect();
        assert_eq!(segs.len(), 1);
        if let PathSeg::Cubic(c) = segs[0] {
            assert!((c.p1 - pts[1]).hypot() < 1e-9 && (c.p2 - pts[2]).hypot() < 1e-9);
        }
        // Closed: one segment a point, ending where it began.
        let p = bspline_path(&pts, true);
        let segs: Vec<_> = p.segments().collect();
        assert_eq!(segs.len(), 5);
        assert!((segs[4].end() - segs[0].start()).hypot() < 1e-9);
        // Two points: a line; three: a quadratic.
        assert_eq!(bspline_path(&pts[..2], false).segments().count(), 1);
        assert!(matches!(
            bspline_path(&pts[..3], false).segments().next(),
            Some(PathSeg::Quad(_))
        ));
    }

    #[test]
    fn twisting_turns_the_points_between_the_vertices() {
        let r = Rect::new(-10.0, -10.0, 10.0, 10.0);
        let star = corners(&polygon_path(r, 5, 0.5));
        let turn = 0.3;
        let twisted = corners(&twisted_polygon_path(r, 5, 0.5, turn));
        assert_eq!((star.len(), twisted.len()), (10, 10));
        for (i, (a, b)) in star.iter().zip(&twisted).enumerate() {
            if i % 2 == 0 {
                // The vertices stay.
                assert!((*a - *b).hypot() < 1e-9, "{i}");
            } else {
                // The points between them keep their distance and turn.
                assert!((a.to_vec2().hypot() - b.to_vec2().hypot()).abs() < 1e-9);
                let da = b.y.atan2(b.x) - a.y.atan2(a.x);
                let da = (da + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                    - std::f64::consts::PI;
                assert!((da - turn).abs() < 1e-9, "{i}: {da}");
            }
        }
        // A plain polygon turned gains points where its sides' middles were.
        let plain = corners(&twisted_polygon_path(r, 6, 0.0, 0.2));
        assert_eq!(plain.len(), 12);
        let mid = (std::f64::consts::PI / 6.0).cos() * 10.0;
        assert!((plain[1].to_vec2().hypot() - mid).abs() < 1e-9);
        // Untwisted is the polygon as before; nonsense counts as untwisted.
        assert_eq!(corners(&twisted_polygon_path(r, 6, 0.0, 0.0)).len(), 6);
        assert_eq!(corners(&twisted_polygon_path(r, 5, 0.5, f64::NAN)), star);
    }

    #[test]
    fn rect_path_has_bounds() {
        let r = Rect::new(10.0, 20.0, 110.0, 70.0);
        let p = rect_path(r, 0.0);
        let b = p.bounding_box();
        assert!((b.x0 - 10.0).abs() < 1e-9 && (b.y1 - 70.0).abs() < 1e-9);
    }

    #[test]
    fn simplify_drops_collinear_points() {
        let pts: Vec<Point> = (0..10).map(|i| Point::new(i as f64, 0.0)).collect();
        assert_eq!(simplify(&pts, 0.1).len(), 2);
        let smooth = smooth_path(&pts, false);
        assert_eq!(smooth.elements().len(), 10);
    }

    #[test]
    fn closed_smooth_curves_wrap_around_without_a_corner() {
        let pts = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let p = smooth_path(&pts, true);
        // Four curves (the last one back to the start) and the close.
        let curves: Vec<(Point, Point, Point)> = p
            .elements()
            .iter()
            .filter_map(|e| match e {
                PathEl::CurveTo(a, b, c) => Some((*a, *b, *c)),
                _ => None,
            })
            .collect();
        assert_eq!(curves.len(), 4);
        assert_eq!(curves[3].2, pts[0]);
        // The tangent into the start equals the tangent out of it.
        let into = pts[0] - curves[3].1;
        let out = curves[0].0 - pts[0];
        assert!((into.x - out.x).abs() < 1e-9 && (into.y - out.y).abs() < 1e-9);
        // A closing point equal to the first changes nothing.
        let mut again = pts.to_vec();
        again.push(pts[0]);
        assert_eq!(smooth_path(&again, true), p);
    }

    #[test]
    fn star_has_twice_the_points() {
        let p = polygon_path(Rect::new(0.0, 0.0, 10.0, 10.0), 5, 0.5);
        let n = p
            .elements()
            .iter()
            .filter(|e| matches!(e, PathEl::LineTo(_)))
            .count();
        assert_eq!(n, 9);
    }

    /// #1331: a negative sharpness puts the inner points past the centre.
    #[test]
    fn negative_sharpness_inverts_the_star() {
        let r = Rect::new(-10.0, -10.0, 10.0, 10.0);
        let pts = |s: f64| -> Vec<Point> {
            polygon_path(r, 5, s)
                .elements()
                .iter()
                .filter_map(|e| e.end_point())
                .collect()
        };
        let (plain, inverted) = (pts(0.7), pts(-0.7));
        assert_eq!(inverted.len(), 10);
        for i in 0..10 {
            if i % 2 == 0 {
                // The vertices stay.
                assert!((plain[i] - inverted[i]).hypot() < 1e-9);
            } else {
                // Each inner point mirrored through the centre.
                assert!((plain[i].to_vec2() + inverted[i].to_vec2()).hypot() < 1e-9);
            }
        }
        assert!((star_inner_ratio(-0.7) + 0.3).abs() < 1e-12);
        assert!((star_inner_ratio(0.7) - 0.3).abs() < 1e-12);
        assert_eq!(star_inner_ratio(-1.0), 0.0);
        for s in [0.001, 0.3, 0.999, -0.001, -0.3, -0.999] {
            assert!((star_sharpness_for_ratio(star_inner_ratio(s)) - s).abs() < 1e-12);
        }
        // Kept a star on both sides, never 0.
        assert_eq!(star_sharpness_for_ratio(2.0), 0.001);
        assert_eq!(star_sharpness_for_ratio(-2.0), -0.001);
        assert_eq!(star_sharpness_for_ratio(0.0), 0.999);
        assert!(is_star_sharpness(-0.5) && !is_star_sharpness(0.0));
        assert!(!is_star_sharpness(f64::NAN));
    }

    #[test]
    fn polygons_point_up() {
        // First vertex at the top centre (y up).
        let p = polygon_path(Rect::new(0.0, 0.0, 10.0, 10.0), 5, 0.0);
        match p.elements()[0] {
            PathEl::MoveTo(q) => assert!((q.x - 5.0).abs() < 1e-9 && (q.y - 10.0).abs() < 1e-9),
            ref e => panic!("{e:?}"),
        }
    }

    #[test]
    fn complex_stars_cross_their_sides() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        let count =
            |p: &BezPath, f: fn(&PathEl) -> bool| p.elements().iter().filter(|e| f(e)).count();
        // A pentagram: one subpath of five vertices, every second one.
        let p = complex_star_path(r, 5, 1);
        assert_eq!(count(&p, |e| matches!(e, PathEl::MoveTo(_))), 1);
        assert_eq!(count(&p, |e| matches!(e, PathEl::LineTo(_))), 4);
        let pts: Vec<Point> = p
            .elements()
            .iter()
            .filter_map(|e| match e {
                PathEl::MoveTo(q) | PathEl::LineTo(q) => Some(*q),
                _ => None,
            })
            .collect();
        let verts = polygon_path(r, 5, 0.0);
        let PathEl::LineTo(v2) = verts.elements()[2] else {
            panic!("{verts:?}");
        };
        assert!((pts[1] - v2).hypot() < 1e-9, "{pts:?}");
        // Filled even-odd, the middle pentagon is a hole.
        assert!(p.winding(r.center()).abs() == 2);
        // Six points: two triangles.
        let p = complex_star_path(r, 6, 1);
        assert_eq!(count(&p, |e| matches!(e, PathEl::MoveTo(_))), 2);
        // Sharpness follows the point count.
        assert_eq!(complex_star_max_sharpness(5), 1);
        assert_eq!(complex_star_max_sharpness(9), 3);
        assert_eq!(complex_star_max_sharpness(12), 4);
        assert_eq!(complex_star_max_sharpness(0), 1);
        // Out of range values are brought in.
        assert_eq!(complex_star_path(r, 9, 50), complex_star_path(r, 9, 3));
        assert_eq!(complex_star_path(r, 1, 1), complex_star_path(r, 5, 1));
    }

    #[test]
    fn polygon_fills_the_dragged_box() {
        let drag = Rect::new(20.0, 10.0, 70.0, 40.0);
        for (n, sharp) in [(3, 0.0), (5, 0.0), (6, 0.0), (5, 0.5), (3, 0.2), (12, 0.3)] {
            let r = polygon_rect_for_bounds(drag, n, sharp);
            let b = polygon_path(r, n, sharp).bounding_box();
            assert!(
                (b.x0 - drag.x0).abs() < 1e-9
                    && (b.y0 - drag.y0).abs() < 1e-9
                    && (b.x1 - drag.x1).abs() < 1e-9
                    && (b.y1 - drag.y1).abs() < 1e-9,
                "{n} {sharp}: {b:?}"
            );
        }
    }
}
