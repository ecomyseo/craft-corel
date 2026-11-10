//! Shape Recognition: a freehand stroke becomes the shape it looks like,
//! as drawn (tilted shapes stay tilted): a straight line, a rectangle, an
//! ellipse, a triangle, a four-sided shape with its own corners, else a
//! smoothed curve. How readily strokes are taken for shapes and how much
//! free curves are smoothed are the property bar's two levels.

use crate::app::App;
use serde::{Deserialize, Serialize};
use tracedraw_core::{
    document::{Shape, ShapeKind},
    geometry::{self, Affine, BezPath, Point, Rect, Vec2},
    Command,
};

/// The bar's levels, 0 (off) to 5 (highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RecognizeSettings {
    /// How readily a stroke is taken for a shape.
    pub recognition: u8,
    /// How much a free curve is smoothed.
    pub smoothing: u8,
}

impl Default for RecognizeSettings {
    fn default() -> Self {
        RecognizeSettings {
            recognition: 3,
            smoothing: 3,
        }
    }
}

/// The level names, off to highest.
pub const LEVEL_KEYS: [&str; 6] = [
    "level.none",
    "level.lowest",
    "level.low",
    "level.medium",
    "level.high",
    "level.highest",
];

impl RecognizeSettings {
    fn level(v: u8) -> usize {
        (v as usize).min(5)
    }

    /// Free curves: how far the stroke may be simplified, mm.
    pub fn smoothing_tolerance(&self) -> f64 {
        [0.05, 0.15, 0.3, 0.6, 1.0, 1.6][Self::level(self.smoothing)]
    }

    /// Corners: simplification as a share of the stroke's size.
    fn corner_share(&self) -> f64 {
        [0.0, 0.03, 0.045, 0.06, 0.08, 0.11][Self::level(self.recognition)]
    }

    /// A straight line: the farthest point from the chord as a share of it.
    fn straight_share(&self) -> f64 {
        [0.0, 0.025, 0.04, 0.06, 0.08, 0.11][Self::level(self.recognition)]
    }

    /// Right angles: how far off 90 degrees a corner may be.
    fn right_angle_slack(&self) -> f64 {
        [0.0, 8.0, 12.0, 16.0, 20.0, 25.0][Self::level(self.recognition)]
    }

    /// Ellipses: how far the area may be from the fitted ellipse's.
    fn ellipse_slack(&self) -> f64 {
        [0.0, 0.05, 0.08, 0.12, 0.16, 0.2][Self::level(self.recognition)]
    }
}

/// Sketch: how a stroke joins the selected curve and how much it is
/// smoothed.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SketchSettings {
    /// A stroke starting this close to the selected open curve's end (mm)
    /// continues it.
    pub join_distance: f64,
    /// Join strokes to the selected curve at all.
    pub join: bool,
    /// Smoothing level, 0 (off) to 5.
    pub smoothing: u8,
}

impl Default for SketchSettings {
    fn default() -> Self {
        SketchSettings {
            join_distance: 5.0,
            join: true,
            smoothing: 3,
        }
    }
}

impl SketchSettings {
    /// How far a stroke may be simplified, mm.
    pub fn tolerance(&self) -> f64 {
        RecognizeSettings {
            recognition: 0,
            smoothing: self.smoothing,
        }
        .smoothing_tolerance()
    }
}

/// What a stroke becomes: a shape in its own space and where it goes.
#[derive(Debug, Clone, PartialEq)]
pub struct Recognized {
    pub kind: ShapeKind,
    pub transform: Affine,
}

fn plain(kind: ShapeKind) -> Recognized {
    Recognized {
        kind,
        transform: Affine::IDENTITY,
    }
}

/// An angle near a multiple of 90 degrees snapped onto it (within `slack`
/// degrees), so nearly level shapes come out level.
fn snap_angle(a: f64, slack: f64) -> f64 {
    let quarter = std::f64::consts::FRAC_PI_2;
    let k = (a / quarter).round();
    if (a - k * quarter).abs().to_degrees() <= slack {
        k * quarter
    } else {
        a
    }
}

/// Area, centroid and the central second moments (xx, yy, xy) of a
/// polygon.
fn moments(pts: &[Point]) -> Option<(f64, Point, f64, f64, f64)> {
    let n = pts.len();
    if n < 3 {
        return None;
    }
    let (mut a, mut cx, mut cy) = (0.0, 0.0, 0.0);
    let (mut ixx, mut iyy, mut ixy) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (p, q) = (pts[i], pts[(i + 1) % n]);
        let cross = p.x * q.y - q.x * p.y;
        a += cross;
        cx += (p.x + q.x) * cross;
        cy += (p.y + q.y) * cross;
        // Second moments about the origin.
        iyy += (p.x * p.x + p.x * q.x + q.x * q.x) * cross;
        ixx += (p.y * p.y + p.y * q.y + q.y * q.y) * cross;
        ixy += (p.x * q.y + 2.0 * p.x * p.y + 2.0 * q.x * q.y + q.x * p.y) * cross;
    }
    a /= 2.0;
    if a.abs() < 1e-9 {
        return None;
    }
    let (cx, cy) = (cx / (6.0 * a), cy / (6.0 * a));
    // Per unit area, about the centroid (x^2, y^2 and xy averages).
    let mxx = iyy / (12.0 * a) - cx * cx;
    let myy = ixx / (12.0 * a) - cy * cy;
    let mxy = ixy / (24.0 * a) - cx * cy;
    Some((a.abs(), Point::new(cx, cy), mxx, myy, mxy))
}

/// The corners of a closed stroke: simplified from its point farthest from
/// the middle, the closing duplicate dropped.
fn corners(points: &[Point], tolerance: f64) -> Vec<Point> {
    let c = points.iter().fold(Vec2::ZERO, |acc, p| acc + p.to_vec2()) / points.len().max(1) as f64;
    let start = points
        .iter()
        .enumerate()
        .max_by(|a, b| {
            (a.1.to_vec2() - c)
                .hypot()
                .total_cmp(&(b.1.to_vec2() - c).hypot())
        })
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut loop_pts: Vec<Point> = points[start..]
        .iter()
        .chain(points[..start].iter())
        .copied()
        .collect();
    if let Some(first) = loop_pts.first().copied() {
        loop_pts.push(first);
    }
    let mut out = geometry::simplify(&loop_pts, tolerance);
    if out.len() > 1
        && out
            .first()
            .zip(out.last())
            .is_some_and(|(a, b)| (*a - *b).hypot() < tolerance)
    {
        out.pop();
    }
    out
}

/// The inside angle at each corner, degrees.
fn angles(c: &[Point]) -> Vec<f64> {
    let n = c.len();
    (0..n)
        .map(|i| {
            let (p, q, r) = (c[(i + n - 1) % n], c[i], c[(i + 1) % n]);
            let (u, v) = (p - q, r - q);
            let d = u.hypot() * v.hypot();
            if d < 1e-12 {
                0.0
            } else {
                (u.dot(v) / d).clamp(-1.0, 1.0).acos().to_degrees()
            }
        })
        .collect()
}

/// What `points` (page space) become.
pub fn recognize(points: &[Point], s: &RecognizeSettings) -> Option<Recognized> {
    if points.len() < 3 {
        return None;
    }
    let b = Rect::from_points(points[0], points[0]);
    let b = points.iter().fold(b, |r, p| r.union_pt(*p));
    let diag = (b.width().powi(2) + b.height().powi(2)).sqrt();
    if diag.is_nan() || diag <= 0.5 {
        return None;
    }
    let (first, last) = (points[0], points[points.len() - 1]);
    let closed = (first - last).hypot() < 0.2 * diag;
    let free = |closed: bool| {
        let pts = geometry::simplify(points, s.smoothing_tolerance());
        plain(ShapeKind::Path {
            path: geometry::smooth_path(&pts, closed),
            closed,
        })
    };
    if s.recognition == 0 {
        return Some(free(closed));
    }
    if !closed {
        // Straight: nothing far from the line between the ends.
        let chord = last - first;
        let len = chord.hypot();
        if len > 1e-9 {
            let n = Vec2::new(-chord.y, chord.x) / len;
            let far = points
                .iter()
                .map(|p| (*p - first).dot(n).abs())
                .fold(0.0, f64::max);
            if far / len <= s.straight_share() {
                let mut path = BezPath::new();
                path.move_to(first);
                path.line_to(last);
                return Some(plain(ShapeKind::Path {
                    path,
                    closed: false,
                }));
            }
        }
        return Some(free(false));
    }
    let c = corners(points, s.corner_share() * diag);
    let slack = s.right_angle_slack();
    let ang = angles(&c);
    // A rectangle: four corners near right angles.
    if c.len() == 4 && ang.iter().all(|a| (a - 90.0).abs() <= slack) {
        let mut w = ((c[1] - c[0]).hypot() + (c[3] - c[2]).hypot()) / 2.0;
        let mut h = ((c[2] - c[1]).hypot() + (c[0] - c[3]).hypot()) / 2.0;
        let middle = Point::new(
            (c[0].x + c[1].x + c[2].x + c[3].x) / 4.0,
            (c[0].y + c[1].y + c[2].y + c[3].y) / 4.0,
        );
        // A rectangle turned a quarter turn is the same rectangle with its
        // sides swapped: keep the angle between -45 and 45 degrees.
        let mut angle = (c[1] - c[0]).atan2();
        let quarter = std::f64::consts::FRAC_PI_2;
        while angle > quarter / 2.0 {
            angle -= quarter;
            std::mem::swap(&mut w, &mut h);
        }
        while angle <= -quarter / 2.0 {
            angle += quarter;
            std::mem::swap(&mut w, &mut h);
        }
        let angle = snap_angle(angle, slack);
        return Some(Recognized {
            kind: ShapeKind::Rect {
                rect: Rect::new(-w / 2.0, -h / 2.0, w / 2.0, h / 2.0),
                radius: 0.0,
                corners: None,
            },
            transform: Affine::translate(middle.to_vec2()) * Affine::rotate(angle),
        });
    }
    // An ellipse: as much area as the ellipse of the same spread, any tilt.
    if c.len() >= 5 || c.len() < 3 {
        if let Some((area, centroid, mxx, myy, mxy)) = moments(points) {
            let mid = (mxx + myy) / 2.0;
            let diff = (((mxx - myy) / 2.0).powi(2) + mxy * mxy).sqrt();
            let (l1, l2) = (mid + diff, (mid - diff).max(0.0));
            let (ra, rb) = (2.0 * l1.sqrt(), 2.0 * l2.sqrt());
            let fitted = std::f64::consts::PI * ra * rb;
            if fitted > 1e-9 && (area / fitted - 1.0).abs() <= s.ellipse_slack() {
                let tilt = 0.5 * (2.0 * mxy).atan2(mxx - myy);
                // A near circle needs no tilt; a nearly level one is level.
                let angle = if (ra - rb).abs() < 0.08 * ra {
                    0.0
                } else {
                    snap_angle(tilt, slack)
                };
                return Some(Recognized {
                    kind: ShapeKind::Ellipse {
                        rect: Rect::new(-ra, -rb, ra, rb),
                        arc: None,
                    },
                    transform: Affine::translate(centroid.to_vec2()) * Affine::rotate(angle),
                });
            }
        }
    }
    // Triangles and other four-sided shapes keep their corners.
    if c.len() == 3 || c.len() == 4 {
        return Some(plain(ShapeKind::Path {
            path: geometry::polyline_path(&c, true),
            closed: true,
        }));
    }
    Some(free(true))
}

impl App {
    /// The Shape Recognition tool's stroke ends: its shape on the active
    /// layer, selected, one undo step.
    pub fn finish_shape_recognition(&mut self, points: Vec<Point>) {
        let Some(r) = recognize(&points, &self.recognize) else {
            return;
        };
        let Some(layer) = self.active_layer() else {
            return;
        };
        let id = self.engine.new_shape_id();
        let mut shape = Shape::new(id, r.kind);
        shape.transform = r.transform;
        self.doc().defaults.apply_to(&mut shape);
        if matches!(shape.kind, ShapeKind::Path { closed: false, .. }) {
            shape.fill = tracedraw_core::Fill::None;
        }
        self.run(Command::AddShape { layer, shape });
        self.select(vec![id]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Shape as _;

    fn wobble(i: usize) -> f64 {
        ((i as f64) * 2.399).sin() * 0.3
    }

    /// Points around a rectangle (`w` by `h`, turned by `angle` about
    /// `c`), slightly wobbly, closed by hand.
    fn rect_stroke(c: Point, w: f64, h: f64, angle: f64) -> Vec<Point> {
        let corners = [
            (-w / 2.0, -h / 2.0),
            (w / 2.0, -h / 2.0),
            (w / 2.0, h / 2.0),
            (-w / 2.0, h / 2.0),
        ];
        let t = Affine::translate(c.to_vec2()) * Affine::rotate(angle);
        let mut pts = Vec::new();
        for k in 0..4 {
            let (a, b) = (corners[k], corners[(k + 1) % 4]);
            for i in 0..20 {
                let f = i as f64 / 20.0;
                let p = Point::new(a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f);
                pts.push(t * (p + Vec2::new(wobble(pts.len()), wobble(pts.len() + 7))));
            }
        }
        pts.push(pts[0] + Vec2::new(0.4, 0.2));
        pts
    }

    fn ellipse_stroke(c: Point, a: f64, b: f64, angle: f64) -> Vec<Point> {
        let t = Affine::translate(c.to_vec2()) * Affine::rotate(angle);
        (0..=80)
            .map(|i| {
                let th = i as f64 / 80.0 * std::f64::consts::TAU;
                t * Point::new(a * th.cos() + wobble(i), b * th.sin() + wobble(i + 3))
            })
            .collect()
    }

    #[test]
    fn strokes_become_the_shapes_they_look_like() {
        let s = RecognizeSettings::default();
        // A level rectangle.
        let r =
            recognize(&rect_stroke(Point::new(50.0, 50.0), 40.0, 20.0, 0.05), &s).expect("shape");
        match r.kind {
            ShapeKind::Rect { rect, .. } => {
                assert!(
                    (rect.width() - 40.0).abs() < 1.5 && (rect.height() - 20.0).abs() < 1.5,
                    "{rect:?}"
                );
            }
            other => panic!("{other:?}"),
        }
        // Nearly level: snapped level.
        assert!((r.transform.as_coeffs()[1]).abs() < 1e-9);
        // A tilted rectangle stays tilted.
        let r =
            recognize(&rect_stroke(Point::new(50.0, 50.0), 40.0, 20.0, 0.6), &s).expect("shape");
        match r.kind {
            ShapeKind::Rect { rect, .. } => {
                assert!((rect.width() - 40.0).abs() < 1.5, "{rect:?}");
            }
            other => panic!("{other:?}"),
        }
        let c = r.transform.as_coeffs();
        assert!((c[1].atan2(c[0]) - 0.6).abs() < 0.08, "{c:?}");
        // A quarter turn more: the same rectangle on its side.
        let r = recognize(
            &rect_stroke(
                Point::new(50.0, 50.0),
                40.0,
                20.0,
                0.6 + std::f64::consts::FRAC_PI_2,
            ),
            &s,
        )
        .expect("shape");
        match r.kind {
            ShapeKind::Rect { rect, .. } => assert!((rect.width() - 20.0).abs() < 1.5, "{rect:?}"),
            other => panic!("{other:?}"),
        }
        // A circle and a tilted ellipse.
        let r =
            recognize(&ellipse_stroke(Point::new(0.0, 0.0), 20.0, 20.0, 0.0), &s).expect("shape");
        match r.kind {
            ShapeKind::Ellipse { rect, .. } => {
                assert!((rect.width() - 40.0).abs() < 2.0, "{rect:?}")
            }
            other => panic!("{other:?}"),
        }
        let r =
            recognize(&ellipse_stroke(Point::new(0.0, 0.0), 30.0, 12.0, 0.7), &s).expect("shape");
        match r.kind {
            ShapeKind::Ellipse { rect, .. } => {
                assert!(
                    (rect.width() - 60.0).abs() < 3.0 && (rect.height() - 24.0).abs() < 3.0,
                    "{rect:?}"
                );
                let c = r.transform.as_coeffs();
                assert!((c[1].atan2(c[0]) - 0.7).abs() < 0.08, "{c:?}");
            }
            other => panic!("{other:?}"),
        }
        // A triangle keeps its corners.
        let tri: Vec<Point> = {
            let v = [
                Point::new(0.0, 0.0),
                Point::new(40.0, 0.0),
                Point::new(10.0, 30.0),
            ];
            let mut pts = Vec::new();
            for k in 0..3 {
                for i in 0..20 {
                    pts.push(v[k].lerp(v[(k + 1) % 3], i as f64 / 20.0));
                }
            }
            pts.push(v[0]);
            pts
        };
        match recognize(&tri, &s).expect("shape").kind {
            ShapeKind::Path { closed: true, path } => {
                let b = path.bounding_box();
                assert!((b.width() - 40.0).abs() < 1.0 && (b.height() - 30.0).abs() < 1.0);
            }
            other => panic!("{other:?}"),
        }
        // A nearly straight stroke: a line from end to end.
        let line: Vec<Point> = (0..30)
            .map(|i| Point::new(i as f64, wobble(i) * 0.5))
            .collect();
        match recognize(&line, &s).expect("shape").kind {
            ShapeKind::Path {
                closed: false,
                path,
            } => assert_eq!(path.elements().len(), 2),
            other => panic!("{other:?}"),
        }
        // Recognition off: the rectangle stroke stays a smoothed curve.
        let off = RecognizeSettings {
            recognition: 0,
            ..s
        };
        let r =
            recognize(&rect_stroke(Point::new(50.0, 50.0), 40.0, 20.0, 0.0), &off).expect("shape");
        assert!(matches!(r.kind, ShapeKind::Path { closed: true, .. }));
        // Too small to be anything.
        assert!(recognize(
            &[Point::ZERO, Point::new(0.1, 0.0), Point::new(0.2, 0.1)],
            &s
        )
        .is_none());
    }
}
