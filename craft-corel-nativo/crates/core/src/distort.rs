//! Distortions of an outline about a centre: Push and Pull, Zipper and
//! Twister. They work on the path's own nodes and segments, so every
//! subpath stays a subpath and an open curve stays open.
//!
//! - Push and Pull moves each node along the line from the centre by
//!   `amount` percent of its distance (away for a positive amount, in for
//!   a negative one) and the control points of the segments the other way
//!   by the same share, so the segments bow against the nodes: pushed
//!   nodes give spikes with hollow sides, pulled nodes give petals.
//!   Straight segments become curves whose control points start on their
//!   nodes, so a side bows from its middle and each control point moves
//!   along the line from the centre through its node: pushed corners
//!   become spikes, pulled corners curl.
//! - Zipper puts `frequency` teeth on every segment: the segment is cut
//!   into `frequency + 1` equal steps of its parameter and the points
//!   alternate `amplitude` to one side of the segment and to the other.
//!   The tips (the points on the side the handle is on) lean: they sit
//!   on the segment's normal turned by `tilt_deg`, still `amplitude`
//!   from the segment, while the bases stay straight out on the other
//!   side, so a lean neither moves the bases nor changes the teeth's
//!   height (#1671, #1736). Random scales each tooth by a fixed
//!   pseudo-random share; Local scales them by how near they are to the
//!   centre (full at the centre, none at the farthest node); Smooth
//!   draws a wave through the tips, a sine's shape, which never crosses
//!   itself however high the teeth are. With `shifted_tips` (the teeth
//!   `.cdr` files hold, #1715) the outline's first point is a tip, on
//!   the normal's side, and every tip is moved by a fifth of the way
//!   from the middle of the outline's box to the centre instead of
//!   leaning by `tilt_deg`: the handle's place sets the lean.
//! - Twister turns every point about the centre by `angle` times one
//!   less its distance over the farthest distance, so the points at the
//!   centre turn by the whole angle and the farthest point stays where it
//!   is: the outline keeps its place and its outer shape, the inside
//!   swirls (#1674). The outline is first cut into short steps so the
//!   turn bends it smoothly.

use crate::geometry::{
    BezPath, CubicBez, Line, ParamCurve, ParamCurveArclen, ParamCurveDeriv, ParamCurveNearest,
    PathEl, PathSeg, Point, Rect, Shape as _, Vec2,
};
use kurbo::QuadBez;

/// A distortion and its centre (in the space of the path it is applied
/// to; `None` for the middle of the path's bounds).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum Distort {
    /// Nodes pushed away from the centre (positive) or pulled in
    /// (negative), percent of their distance, -200 to 200.
    PushPull {
        amount: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center: Option<Point>,
    },
    /// Teeth `amplitude` mm high, `frequency` on each segment, leaning
    /// `tilt_deg` from the outline's normal (-180 to 180, counterclockwise
    /// when positive; older files have none, upright teeth).
    Zipper {
        amplitude: f64,
        frequency: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center: Option<Point>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        random: bool,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        smooth: bool,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        local: bool,
        #[serde(default, skip_serializing_if = "is_zero")]
        tilt_deg: f64,
        /// The teeth as `.cdr` files hold them (#1715): the first point
        /// of the outline is a tip, on the normal's side, and the tips
        /// are moved by [`SHIFT_SHARE`] of the way from the middle of the
        /// outline's box to the centre; `tilt_deg` is not used.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        shifted_tips: bool,
    },
    /// Turned about the centre, `angle_deg` at the centre and none at
    /// the farthest point (counterclockwise when positive).
    Twister {
        angle_deg: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center: Option<Point>,
    },
}

impl Distort {
    /// The centre it was given, if any.
    pub fn center(&self) -> Option<Point> {
        match *self {
            Distort::PushPull { center, .. }
            | Distort::Zipper { center, .. }
            | Distort::Twister { center, .. } => center,
        }
    }

    /// The same distortion about another centre.
    pub fn with_center(self, at: Option<Point>) -> Distort {
        match self {
            Distort::PushPull { amount, .. } => Distort::PushPull { amount, center: at },
            Distort::Zipper {
                amplitude,
                frequency,
                random,
                smooth,
                local,
                tilt_deg,
                shifted_tips,
                ..
            } => Distort::Zipper {
                amplitude,
                frequency,
                center: at,
                random,
                smooth,
                local,
                tilt_deg,
                shifted_tips,
            },
            Distort::Twister { angle_deg, .. } => Distort::Twister {
                angle_deg,
                center: at,
            },
        }
    }
}

/// The highest Zipper frequency and amplitude, the largest Twister
/// angle (degrees) and Push and Pull amount taken, and the farthest a
/// Zipper's teeth lean either way (degrees).
pub const MAX_FREQUENCY: u32 = 100;
pub const MAX_AMPLITUDE: f64 = 1000.0;
pub const MAX_ANGLE: f64 = 3600.0;
pub const MAX_AMOUNT: f64 = 200.0;
pub const MAX_TILT: f64 = 180.0;

/// How far shifted tips move (see `Distort::Zipper::shifted_tips`): this
/// share of the way from the middle of the outline's box to the centre.
/// The `.cdr` files of #1715 and #1671 (Zipper on a line, the centre
/// moved about between them) have every tip of the normal's side moved
/// so, whatever the amplitude, and the other points not at all.
pub const SHIFT_SHARE: f64 = 0.2;

fn finite(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// Apply `how` to `path`.
pub fn distort(path: &BezPath, how: Distort) -> BezPath {
    distort_in(path, path.bounding_box(), how)
}

/// As [`distort`], about the box `b` instead of the path's own (the
/// objects of a group, each distorted about the box of them all).
pub fn distort_in(path: &BezPath, b: Rect, how: Distort) -> BezPath {
    if !(b.x0.is_finite() && b.y0.is_finite() && b.x1.is_finite() && b.y1.is_finite()) {
        return path.clone();
    }
    // A centre far off the outline (a damaged file) is not taken.
    let reach = b.inflate(1e5, 1e5);
    let c = how
        .center()
        .filter(|p| finite(*p) && reach.contains(*p))
        .unwrap_or(b.center());
    let out = match how {
        Distort::PushPull { amount, .. } => {
            let s = if amount.is_finite() {
                amount.clamp(-MAX_AMOUNT, MAX_AMOUNT) / 100.0
            } else {
                0.0
            };
            push_pull(path, c, s)
        }
        Distort::Zipper {
            amplitude,
            frequency,
            random,
            smooth,
            local,
            tilt_deg,
            shifted_tips,
            ..
        } => {
            let a = if amplitude.is_finite() {
                amplitude.clamp(0.0, MAX_AMPLITUDE)
            } else {
                0.0
            };
            let tilt = if tilt_deg.is_finite() {
                tilt_deg.clamp(-MAX_TILT, MAX_TILT).to_radians()
            } else {
                0.0
            };
            zipper(
                path,
                c,
                Teeth {
                    amplitude: a,
                    frequency: frequency.min(MAX_FREQUENCY),
                    random,
                    smooth,
                    local,
                    tilt,
                    shift: shifted_tips.then(|| (c - b.center()) * SHIFT_SHARE),
                },
            )
        }
        Distort::Twister { angle_deg, .. } => {
            let a = if angle_deg.is_finite() {
                angle_deg.clamp(-MAX_ANGLE, MAX_ANGLE)
            } else {
                0.0
            };
            twister(path, c, a.to_radians())
        }
    };
    let ob = out.bounding_box();
    if ob.x0.is_finite() && ob.y0.is_finite() && ob.x1.is_finite() && ob.y1.is_finite() {
        out
    } else {
        path.clone()
    }
}

/// One subpath: its segments and whether it closes.
struct Sub {
    segs: Vec<PathSeg>,
    closed: bool,
    start: Point,
}

/// The subpaths of `path` as segments, the closing line of a closed one
/// included as a segment when its last point is not its first.
fn subpaths(path: &BezPath) -> Vec<Sub> {
    let mut out: Vec<Sub> = Vec::new();
    let mut cur: Option<Sub> = None;
    let mut last = Point::ZERO;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                if let Some(s) = cur.take() {
                    out.push(s);
                }
                cur = Some(Sub {
                    segs: Vec::new(),
                    closed: false,
                    start: p,
                });
                last = p;
            }
            PathEl::LineTo(p) => {
                let s = cur.get_or_insert(Sub {
                    segs: Vec::new(),
                    closed: false,
                    start: last,
                });
                s.segs.push(PathSeg::Line(Line::new(last, p)));
                last = p;
            }
            PathEl::QuadTo(q, p) => {
                let s = cur.get_or_insert(Sub {
                    segs: Vec::new(),
                    closed: false,
                    start: last,
                });
                s.segs.push(PathSeg::Quad(QuadBez::new(last, q, p)));
                last = p;
            }
            PathEl::CurveTo(a, b, p) => {
                let s = cur.get_or_insert(Sub {
                    segs: Vec::new(),
                    closed: false,
                    start: last,
                });
                s.segs.push(PathSeg::Cubic(CubicBez::new(last, a, b, p)));
                last = p;
            }
            PathEl::ClosePath => {
                if let Some(mut s) = cur.take() {
                    if (last - s.start).hypot() > 1e-9 {
                        s.segs.push(PathSeg::Line(Line::new(last, s.start)));
                    }
                    s.closed = true;
                    last = s.start;
                    out.push(s);
                }
            }
        }
    }
    if let Some(s) = cur.take() {
        out.push(s);
    }
    // A subpath that ends where it started is closed (an ellipse has no
    // closing element).
    for s in &mut out {
        if !s.closed
            && s.segs.len() > 1
            && s.segs
                .last()
                .is_some_and(|g| (g.end() - s.start).hypot() < 1e-6)
        {
            s.closed = true;
        }
    }
    out
}

/// A segment as a cubic: lines with their control points at a third and
/// two thirds, quadratics raised.
fn as_cubic(seg: &PathSeg) -> (Point, Point, Point, Point) {
    match *seg {
        PathSeg::Line(l) => (
            l.p0,
            l.p0.lerp(l.p1, 1.0 / 3.0),
            l.p0.lerp(l.p1, 2.0 / 3.0),
            l.p1,
        ),
        PathSeg::Quad(q) => (
            q.p0,
            q.p0 + (q.p1 - q.p0) * (2.0 / 3.0),
            q.p2 + (q.p1 - q.p2) * (2.0 / 3.0),
            q.p2,
        ),
        PathSeg::Cubic(c) => (c.p0, c.p1, c.p2, c.p3),
    }
}

fn push_pull(path: &BezPath, c: Point, s: f64) -> BezPath {
    let node = |p: Point| c + (p - c) * (1.0 + s);
    let handle = |p: Point| c + (p - c) * (1.0 - s);
    let mut out = BezPath::new();
    for sub in subpaths(path) {
        out.move_to(node(sub.start));
        for seg in &sub.segs {
            // A straight side's control points start on its nodes, so
            // they go the other way along the nodes' own lines from the
            // centre: the side bows from its middle and the corners turn
            // into spikes or curls (#1264). Control points at a third and
            // two thirds made the bend start at the corners.
            let (_, a, b, p) = match *seg {
                PathSeg::Line(l) => (l.p0, l.p0, l.p1, l.p1),
                _ => as_cubic(seg),
            };
            out.curve_to(handle(a), handle(b), node(p));
        }
        if sub.closed {
            out.close_path();
        }
    }
    out
}

/// A pseudo-random share in 0..1, the same for the same tooth.
fn share(seg: usize, k: usize) -> f64 {
    let mut h = (seg as u64)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add((k as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 29;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// The unit normal of a segment at `t` (left of its direction).
fn normal(seg: &PathSeg, t: f64) -> Vec2 {
    let e = 1e-4;
    let (t0, t1) = ((t - e).max(0.0), (t + e).min(1.0));
    let d = seg.eval(t1) - seg.eval(t0);
    let len = d.hypot();
    if len < 1e-12 {
        return Vec2::ZERO;
    }
    Vec2::new(-d.y / len, d.x / len)
}

/// The unit normal between two normals (the direction a node moves
/// where two segments meet); `b` alone when they cancel out.
fn between(a: Vec2, b: Vec2) -> Vec2 {
    let m = a + b;
    let len = m.hypot();
    if len < 1e-9 {
        b
    } else {
        m / len
    }
}

/// The unit normal of `path` at its point nearest to `c` (left of the
/// outline's direction there): the direction a Zipper's teeth stand in
/// with no tilt, the one its handle line shows (#1671). To the right
/// when the path has no segments.
pub fn zipper_zero_direction(path: &BezPath, c: Point) -> Vec2 {
    if !finite(c) {
        return Vec2::new(1.0, 0.0);
    }
    let mut best: Option<(f64, Vec2)> = None;
    for seg in path.segments() {
        let near = seg.nearest(c, 1e-6);
        if !near.distance_sq.is_finite() {
            continue;
        }
        if best.is_none_or(|(d, _)| near.distance_sq < d) {
            let n = normal(&seg, near.t.clamp(0.0, 1.0));
            if n.hypot() > 0.5 {
                best = Some((near.distance_sq, n));
            }
        }
    }
    best.map(|(_, n)| n).unwrap_or(Vec2::new(1.0, 0.0))
}

/// A Zipper's teeth: their height, how many on each segment, the
/// options and the lean of the teeth from the normal (radians).
struct Teeth {
    amplitude: f64,
    frequency: u32,
    random: bool,
    smooth: bool,
    local: bool,
    tilt: f64,
    /// Shifted tips: the first point is a tip and every tip is moved by
    /// this, `tilt` left aside.
    shift: Option<Vec2>,
}

/// How far a leaning tip goes along the outline at most, in teeth
/// heights: the shift of a lean of about 84 degrees, so a lean of a
/// quarter turn (its tips would go along the outline for ever) keeps
/// them within ten teeth heights of their place (#1736).
const MAX_LEAN_SHIFT: f64 = 10.0;

/// The handles of a wave's cubic between two tips, a share of the
/// distance between the tips along the outline: the cubic nearest a
/// sine's half wave, through the tips with the outline's direction.
const WAVE_HANDLE: f64 = 0.3642;

/// A wave through the tips `pts` (each with the outline's unit direction
/// there and the distance along the outline to the next tip): one cubic
/// from tip to tip, leaving and arriving along the outline, with its
/// handles `WAVE_HANDLE` of that distance long. The handles never reach
/// past the middle, so the wave goes on along the outline between two
/// tips however high they are and never crosses itself (#1671).
fn wave_path(pts: &[(Point, Vec2, f64)], closed: bool) -> BezPath {
    let mut out = BezPath::new();
    let Some(first) = pts.first() else {
        return out;
    };
    out.move_to(first.0);
    let count = if closed { pts.len() } else { pts.len() - 1 };
    for i in 0..count {
        let (p1, t1, along) = pts[i];
        let (p2, t2, _) = pts[(i + 1) % pts.len()];
        let h = WAVE_HANDLE * along;
        out.curve_to(p1 + t1 * h, p2 - t2 * h, p2);
    }
    if closed {
        out.close_path();
    }
    out
}

fn zipper(path: &BezPath, c: Point, teeth: Teeth) -> BezPath {
    let Teeth {
        amplitude,
        frequency,
        random,
        smooth,
        local,
        tilt,
        shift: tip_shift,
    } = teeth;
    if frequency == 0 || amplitude <= 0.0 {
        return path.clone();
    }
    let (tilt_sin, tilt_cos) = tilt.sin_cos();
    // The side of the outline the handle is on: the normal's side for a
    // lean under a quarter turn, the other side past it.
    let side = if tilt_cos >= 0.0 { 1.0 } else { -1.0 };
    // How far along the outline a tip goes for each unit of its height,
    // counterclockwise of the normal when positive: the tangent of the
    // lean, so the tip lies on the normal turned by the tilt (#1736).
    // Near a quarter turn it is held at `MAX_LEAN_SHIFT`.
    let shift = (tilt_sin / tilt_cos.abs().max(1e-12)).clamp(-MAX_LEAN_SHIFT, MAX_LEAN_SHIFT);
    let subs = subpaths(path);
    // The farthest node from the centre, for Local.
    let far = subs
        .iter()
        .flat_map(|s| std::iter::once(s.start).chain(s.segs.iter().map(|g| g.end())))
        .map(|p| (p - c).hypot())
        .fold(0.0f64, f64::max)
        .max(1e-9);
    // `frequency + 1` lines on every segment, so a line with frequency 5
    // is six lines between seven tips (#1122).
    let steps = frequency as usize + 1;
    let mut out = BezPath::new();
    let mut seg_index = 0usize;
    for sub in subs {
        let count = sub.segs.len();
        if count == 0 {
            out.move_to(sub.start);
            continue;
        }
        // A closed outline needs an even number of tips for the last to
        // alternate with the first: its last segment takes one line more
        // when the count would be odd.
        let odd_closed = sub.closed && (count * steps) % 2 == 1;
        // Each tip, the outline's direction there and the distance along
        // the outline to the next tip (for the wave).
        let mut pts: Vec<(Point, Vec2, f64)> = Vec::with_capacity(count * steps + 2);
        let mut last_base: Option<Point> = None;
        let mut first_base = sub.start;
        for (i, seg) in sub.segs.iter().enumerate() {
            let last_seg = i + 1 == count;
            let steps = if odd_closed && last_seg {
                steps + 1
            } else {
                steps
            };
            // The teeth run through the nodes: every point of the zigzag,
            // the nodes included, is a tooth's tip, so the ends of an open
            // curve are tips too and the pattern goes on unbroken from
            // one segment to the next (#1108). An open curve's last node
            // comes with its last segment; a closed one's start is its
            // end.
            let last = if !sub.closed && last_seg {
                steps
            } else {
                steps - 1
            };
            for k in 0..=last {
                let t = k as f64 / steps as f64;
                let p = seg.eval(t);
                let mut n = normal(seg, t);
                if k == 0 {
                    // A node moves between the normals of its two segments.
                    let before = if i > 0 {
                        sub.segs.get(i - 1)
                    } else if sub.closed {
                        sub.segs.last()
                    } else {
                        None
                    };
                    if let Some(b) = before {
                        n = between(normal(b, 1.0), n);
                    }
                }
                // Tips alternate along the whole subpath; shifted tips
                // start with a tip.
                let first_tip = tip_shift.is_some();
                let sign = if (pts.len() % 2 == 1) != first_tip {
                    1.0
                } else {
                    -1.0
                };
                let mut a = amplitude;
                if random {
                    a *= share(seg_index, k);
                }
                if local {
                    a *= (1.0 - (p - c).hypot() / far).clamp(0.0, 1.0);
                }
                if let (Some(prev), Some(base)) = (pts.last_mut(), last_base) {
                    prev.2 = (p - base).hypot();
                }
                if pts.is_empty() {
                    first_base = p;
                }
                last_base = Some(p);
                // Only the tips lean (#1736): a base stays where the
                // upright teeth have it, `a` straight out on the side
                // away from the handle; a tip (on the handle's side) is
                // `a` out too, so the teeth keep their height, and moved
                // along the outline by `shift` times that, onto the normal
                // turned by `tilt`. The outline's direction is the normal
                // turned back a quarter.
                let across = Vec2::new(-n.y, n.x);
                let moved = match tip_shift {
                    // The teeth of `.cdr` files: tips on the normal's
                    // side moved by the shift, bases straight out on the
                    // other (#1715).
                    Some(v) if sign > 0.0 => n * a + v,
                    Some(_) => n * -a,
                    None if sign > 0.0 => (n * side + across * shift) * a,
                    None => n * (-side * a),
                };
                pts.push((p + moved, Vec2::new(n.y, -n.x), 0.0));
            }
            seg_index += 1;
        }
        if let (Some(last), Some(base)) = (pts.last_mut(), last_base) {
            last.2 = (first_base - base).hypot();
        }
        if smooth {
            out.extend(wave_path(&pts, sub.closed).elements().iter().copied());
        } else {
            let tips: Vec<Point> = pts.iter().map(|t| t.0).collect();
            out.extend(
                crate::geometry::polyline_path(&tips, sub.closed)
                    .elements()
                    .iter()
                    .copied(),
            );
        }
    }
    out
}

/// The most a point's turn changes from one step of a twisted segment
/// to the next (radians): 24 steps to a turn keep a cubic through the
/// steps, with their true directions, within a few hundredths of a
/// percent of the size of the true curve.
const TWIST_STEP: f64 = std::f64::consts::PI / 12.0;

/// The most steps one segment is cut into.
const MAX_STEPS: usize = 2048;

fn twister(path: &BezPath, c: Point, angle: f64) -> BezPath {
    if angle == 0.0 {
        return path.clone();
    }
    let subs = subpaths(path);
    let b = path.bounding_box();
    let size = b.width().max(b.height()).max(1e-6);
    let step = size / 64.0;
    // The farthest point from the centre stays where it is and the turn
    // grows toward the centre, where it is the whole angle (#1674): the
    // outline keeps its outer shape and swirls inside. The distance is
    // measured on points along the segments; the little a curve may
    // reach beyond them does not turn either.
    let far = subs
        .iter()
        .flat_map(|s| {
            std::iter::once(s.start).chain(s.segs.iter().flat_map(|g| {
                let n = ((g.arclen(1e-3) / step).ceil() as usize).clamp(1, 64);
                (1..=n).map(move |k| g.eval(k as f64 / n as f64))
            }))
        })
        .map(|p| (p - c).hypot())
        .fold(0.0f64, f64::max)
        .max(1e-9);
    // The turn falls by `rate` for every unit of distance from the centre.
    let rate = -angle / far;
    // A point and the direction a curve through it goes after the turn:
    // `p` turned about `c` by `angle * (1 - |p - c| / far)`, and its
    // tangent `d` turned the same with the extra swing of the turn
    // changing with the distance (the derivative of the turn along the
    // curve).
    let turn = |p: Point, d: Vec2| -> (Point, Vec2) {
        let v = p - c;
        let r = v.hypot();
        let a = if r > far {
            0.0
        } else {
            (angle + rate * r).clamp(-angle.abs(), angle.abs())
        };
        let (sin, cos) = a.sin_cos();
        let rot = |w: Vec2| Vec2::new(w.x * cos - w.y * sin, w.x * sin + w.y * cos);
        let swing = if r < 1e-12 || r > far {
            Vec2::ZERO
        } else {
            Vec2::new(-v.y, v.x) * (rate * v.dot(d) / r)
        };
        (c + rot(v), rot(d + swing))
    };
    let mut out = BezPath::new();
    for sub in &subs {
        let (p0, _) = turn(sub.start, Vec2::ZERO);
        out.move_to(p0);
        for seg in &sub.segs {
            let cubic = match *seg {
                PathSeg::Line(l) => {
                    let (a, b, c2, d) = as_cubic(&PathSeg::Line(l));
                    CubicBez::new(a, b, c2, d)
                }
                PathSeg::Quad(q) => q.raise(),
                PathSeg::Cubic(cb) => cb,
            };
            // Steps along which the turn changes by at most `TWIST_STEP`
            // (the distance from the centre, so the turn, changes by at
            // most the length walked). A cubic through each step's ends
            // with their true directions follows the turned curve; where
            // the turn does not change (a circle about its middle) the
            // segment turns whole, exactly.
            let len = seg.arclen(1e-3);
            let n = ((rate.abs() * len / TWIST_STEP).ceil() as usize).clamp(1, MAX_STEPS);
            let h = 1.0 / n as f64;
            let deriv = cubic.deriv();
            let mut prev = turn(cubic.eval(0.0), deriv.eval(0.0).to_vec2());
            for k in 1..=n {
                let t = k as f64 * h;
                let next = turn(cubic.eval(t), deriv.eval(t).to_vec2());
                // A cubic through the two points with their directions
                // (Hermite), the handles a third of the step along them.
                let c1 = prev.0 + prev.1 * (h / 3.0);
                let c2 = next.0 - next.1 * (h / 3.0);
                out.curve_to(c1, c2, next.0);
                prev = next;
            }
        }
        if sub.closed {
            out.close_path();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    fn square(x: f64, y: f64, s: f64) -> BezPath {
        let mut p = BezPath::new();
        p.move_to((x, y));
        p.line_to((x + s, y));
        p.line_to((x + s, y + s));
        p.line_to((x, y + s));
        p.close_path();
        p
    }

    fn nodes(p: &BezPath) -> Vec<Point> {
        p.elements()
            .iter()
            .filter_map(|e| match e {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
                PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => Some(*p),
                PathEl::ClosePath => None,
            })
            .collect()
    }

    fn subpath_count(p: &BezPath) -> usize {
        p.elements()
            .iter()
            .filter(|e| matches!(e, PathEl::MoveTo(_)))
            .count()
    }

    fn closes(p: &BezPath) -> usize {
        p.elements()
            .iter()
            .filter(|e| matches!(e, PathEl::ClosePath))
            .count()
    }

    #[test]
    fn push_and_pull_move_nodes_and_bow_the_sides() {
        let sq = square(-10.0, -10.0, 20.0);
        // Pushed 50 %: the corners go out by half their distance, the
        // sides bow in: the middle of a side comes nearer the centre.
        let pushed = distort(
            &sq,
            Distort::PushPull {
                amount: 50.0,
                center: None,
            },
        );
        let n = nodes(&pushed);
        assert!(n
            .iter()
            .any(|p| (p.x - 15.0).abs() < 1e-9 && (p.y - 15.0).abs() < 1e-9));
        let side_mid = |p: &BezPath| {
            p.segments()
                .map(|s| s.eval(0.5))
                .map(|q| q.to_vec2().hypot())
                .fold(f64::INFINITY, f64::min)
        };
        assert!(side_mid(&pushed) < 10.0, "{}", side_mid(&pushed));
        // Pulled 50 %: corners in, sides bow out past the old sides.
        let pulled = distort(
            &sq,
            Distort::PushPull {
                amount: -50.0,
                center: None,
            },
        );
        assert!(nodes(&pulled)
            .iter()
            .any(|p| (p.x - 5.0).abs() < 1e-9 && (p.y - 5.0).abs() < 1e-9));
        let reach = pulled.bounding_box();
        assert!(reach.width() > 10.0);
        assert_eq!((subpath_count(&pulled), closes(&pulled)), (1, 1));
        // About a centre of its own: a node at the centre stays.
        let off = distort(
            &sq,
            Distort::PushPull {
                amount: 80.0,
                center: Some(Point::new(-10.0, -10.0)),
            },
        );
        assert!(nodes(&off)
            .iter()
            .any(|p| (p.x + 10.0).abs() < 1e-9 && (p.y + 10.0).abs() < 1e-9));
        // An open curve stays open; two squares stay two subpaths.
        let mut open = BezPath::new();
        open.move_to((0.0, 0.0));
        open.line_to((10.0, 0.0));
        open.line_to((10.0, 10.0));
        let o = distort(
            &open,
            Distort::PushPull {
                amount: 30.0,
                center: None,
            },
        );
        assert_eq!((subpath_count(&o), closes(&o)), (1, 0));
        let mut two = square(0.0, 0.0, 10.0);
        two.extend(square(20.0, 0.0, 10.0).elements().iter().copied());
        let t = distort(
            &two,
            Distort::PushPull {
                amount: 30.0,
                center: None,
            },
        );
        assert_eq!((subpath_count(&t), closes(&t)), (2, 2));
    }

    /// #1264: a straight side bends from its middle. Its control points
    /// lie on the lines from the centre through its nodes, so pulled in,
    /// a side's lobe reaches past its corners (they curl) and pushed out,
    /// the corners are spikes along those lines.
    #[test]
    fn straight_sides_bow_from_their_middle() {
        let sq = square(-10.0, -10.0, 20.0);
        for amount in [-45.0, 30.0] {
            let out = distort(
                &sq,
                Distort::PushPull {
                    amount,
                    center: None,
                },
            );
            for seg in out.segments() {
                let PathSeg::Cubic(c) = seg else {
                    panic!("sides become curves");
                };
                for (node, handle) in [(c.p0, c.p1), (c.p3, c.p2)] {
                    let cross = node.x * handle.y - node.y * handle.x;
                    assert!(cross.abs() < 1e-9, "{node:?} {handle:?} at {amount}");
                }
            }
        }
        // Pulled 45 %: corners at 5.5, each side's lobe 12.25 out and
        // wider than the corners it runs between.
        let pulled = distort(
            &sq,
            Distort::PushPull {
                amount: -45.0,
                center: None,
            },
        );
        let b = pulled.bounding_box();
        assert!((b.x1 - 12.25).abs() < 0.05, "{b:?}");
        let top = pulled
            .segments()
            .find(|s| s.eval(0.5).y > 10.0)
            .expect("the top side");
        let left_most = (0..=100)
            .map(|i| top.eval(i as f64 / 100.0).x)
            .fold(f64::INFINITY, f64::min);
        assert!(left_most < -5.5 - 0.5, "{left_most}");
    }

    fn zip(random: bool, smooth: bool, local: bool) -> Distort {
        Distort::Zipper {
            amplitude: 2.0,
            frequency: 3,
            center: None,
            random,
            smooth,
            local,
            tilt_deg: 0.0,
            shifted_tips: false,
        }
    }

    fn zipper_on_line(amplitude: f64, frequency: u32, smooth: bool, tilt_deg: f64) -> BezPath {
        let mut line = BezPath::new();
        line.move_to((0.0, 0.0));
        line.line_to((130.0, 0.0));
        distort(
            &line,
            Distort::Zipper {
                amplitude,
                frequency,
                center: None,
                random: false,
                smooth,
                local: false,
                tilt_deg,
                shifted_tips: false,
            },
        )
    }

    /// #1671: the teeth lean by the tilt from the outline's normal, so a
    /// zipper is not always upright: tilted 60 degrees on a line, every
    /// tip lies on the normal turned by 60 degrees from its point on the
    /// line; the tips' direction is the one `zipper_zero_direction`
    /// gives turned by the tilt. Since #1736 only the tips lean, the
    /// amplitude high: the bases stay upright on the other side.
    #[test]
    fn the_teeth_lean_by_the_tilt() {
        let out = zipper_on_line(8.0, 15, false, 60.0);
        let n = nodes(&out);
        assert_eq!(n.len(), 17);
        // The line runs right, its left normal is up; turned 60 degrees
        // it is (-sin 60, cos 60).
        let lean = Vec2::new(-(60f64.to_radians().sin()), 60f64.to_radians().cos());
        for (i, p) in n.iter().enumerate() {
            let base = Point::new(130.0 * i as f64 / 16.0, 0.0);
            let want = if i % 2 == 1 {
                base + lean * (8.0 / 60f64.to_radians().cos())
            } else {
                base + Vec2::new(0.0, -8.0)
            };
            assert!(
                (*p - want).hypot() < 1e-9,
                "tip {i}: {p:?}, wanted {want:?}"
            );
        }
        // The non-tilted teeth stand on the normal (the old test).
        let up = zipper_on_line(8.0, 15, false, 0.0);
        for (i, p) in nodes(&up).iter().enumerate() {
            assert!((p.x - 130.0 * i as f64 / 16.0).abs() < 1e-9, "{i}: {p:?}");
            assert!((p.y.abs() - 8.0).abs() < 1e-9, "{i}: {p:?}");
        }
        // Leaning 180 degrees is the same zigzag with its sides swapped.
        let back = zipper_on_line(8.0, 15, false, 180.0);
        for (p, q) in nodes(&up).iter().zip(nodes(&back)) {
            assert!((p.x - q.x).abs() < 1e-9 && (p.y + q.y).abs() < 1e-9);
        }
        // The zero direction: the normal at the outline's nearest point.
        let sq = square(0.0, 0.0, 20.0);
        let d = zipper_zero_direction(&sq, Point::new(10.0, 2.0));
        assert!(d.x.abs() < 1e-9 && (d.y.abs() - 1.0).abs() < 1e-9, "{d:?}");
        let d = zipper_zero_direction(&sq, Point::new(30.0, 11.0));
        assert!(d.y.abs() < 1e-9 && (d.x.abs() - 1.0).abs() < 1e-9, "{d:?}");
        assert_eq!(
            zipper_zero_direction(&BezPath::new(), Point::ZERO),
            Vec2::new(1.0, 0.0)
        );
        // Older files without a tilt read as upright teeth; a tilt of
        // zero is not written.
        let z: Distort = serde_json::from_str(
            r#"{"mode":"zipper","amplitude":2.0,"frequency":12,"tilt_deg":30.0}"#,
        )
        .expect("a tilted zipper");
        assert!(matches!(z, Distort::Zipper { tilt_deg, .. } if tilt_deg == 30.0));
        let s = serde_json::to_string(&zip(false, false, false)).expect("json");
        assert!(!s.contains("tilt"), "{s}");
    }

    /// #1736: only the tips lean. Tilted either way, the bases (the
    /// points on the side away from the handle) stay where the upright
    /// teeth have them, the amplitude straight out from the outline, and
    /// the tips keep the amplitude's height on the handle's side, moved
    /// along the outline so they lie on the leaned normal from their
    /// point on it: the zigzag is as high as the upright one, half to
    /// each side of the outline, however far it leans.
    #[test]
    fn tilted_teeth_keep_their_bases_and_their_height() {
        let step = 130.0 / 16.0;
        for tilt in [30.0f64, -30.0, 60.0, -75.0] {
            let out = zipper_on_line(8.0, 15, false, tilt);
            let n = nodes(&out);
            assert_eq!(n.len(), 17);
            let shift = -8.0 * tilt.to_radians().tan();
            for (i, p) in n.iter().enumerate() {
                let base = Point::new(step * i as f64, 0.0);
                let want = if i % 2 == 1 {
                    base + Vec2::new(shift, 8.0)
                } else {
                    base + Vec2::new(0.0, -8.0)
                };
                assert!(
                    (*p - want).hypot() < 1e-9,
                    "tilt {tilt}, point {i}: {p:?}, wanted {want:?}"
                );
            }
            // As high as the upright zigzag, half to each side.
            let b = out.bounding_box();
            assert!(
                (b.y0 + 8.0).abs() < 1e-9 && (b.y1 - 8.0).abs() < 1e-9,
                "{b:?}"
            );
        }
        // The handle on the other side (a tilt past a quarter turn): the
        // tips are the points on that side and lean, the others stay.
        let out = zipper_on_line(8.0, 15, false, 150.0);
        let shift = 8.0 * 30f64.to_radians().tan();
        for (i, p) in nodes(&out).iter().enumerate() {
            let base = Point::new(step * i as f64, 0.0);
            let want = if i % 2 == 1 {
                base + Vec2::new(-shift, -8.0)
            } else {
                base + Vec2::new(0.0, 8.0)
            };
            assert!((*p - want).hypot() < 1e-9, "{i}: {p:?}, wanted {want:?}");
        }
        // A tilt of a quarter turn does not throw the tips off to
        // infinity: they stay within reach of the outline.
        let flat = zipper_on_line(8.0, 15, false, 90.0).bounding_box();
        assert!(flat.width() < 130.0 + 8.0 * 20.0, "{flat:?}");
        assert!((flat.y0 + 8.0).abs() < 1e-9, "{flat:?}");
    }

    /// What #1736 keeps of #1671: the teeth lean the way the handle
    /// line goes (each tip lies on the leaned normal from its point on
    /// the outline, the direction `zipper_zero_direction` turned by the
    /// tilt), upright at no tilt, the sides swapped at 180 degrees, and
    /// smooth teeth stay sine waves through the points that never cross
    /// themselves or join, leaning or not.
    #[test]
    fn tilted_teeth_still_follow_the_handle_and_smooth_ones_never_cross() {
        let step = 130.0 / 16.0;
        let tilt = 40f64;
        let line_dir = Vec2::from_angle(
            zipper_zero_direction(
                &{
                    let mut l = BezPath::new();
                    l.move_to((0.0, 0.0));
                    l.line_to((130.0, 0.0));
                    l
                },
                Point::new(65.0, 0.0),
            )
            .atan2()
                + tilt.to_radians(),
        );
        for (i, p) in nodes(&zipper_on_line(8.0, 15, false, tilt))
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 1)
        {
            let d = *p - Point::new(step * i as f64, 0.0);
            assert!(
                d.cross(line_dir).abs() < 1e-9 && d.dot(line_dir) > 0.0,
                "{i}: {d:?}"
            );
        }
        // Smooth and leaning: cubic waves through the same points, each
        // going on along the line (never turning back), within the
        // amplitude of it, crossing it once between two points.
        let sharp = nodes(&zipper_on_line(10.0, 15, false, 25.0));
        let out = zipper_on_line(10.0, 15, true, 25.0);
        assert_eq!(nodes(&out), sharp);
        assert!(out
            .elements()
            .iter()
            .skip(1)
            .all(|e| matches!(e, PathEl::CurveTo(..))));
        for (i, seg) in out.segments().enumerate() {
            let mut last_x = f64::NEG_INFINITY;
            let mut crossings = 0;
            let mut last_y = seg.eval(0.0).y;
            for k in 0..=40 {
                let p = seg.eval(k as f64 / 40.0);
                assert!(p.x >= last_x - 1e-9, "wave {i} turns back at {p:?}");
                assert!(p.y.abs() <= 10.0 + 1e-9, "wave {i} overshoots at {p:?}");
                if k > 0 && (p.y > 0.0) != (last_y > 0.0) {
                    crossings += 1;
                }
                last_x = p.x;
                last_y = p.y;
            }
            assert_eq!(crossings, 1, "wave {i}");
        }
    }

    /// #1671: a smooth zipper is a wave through the tips the shape of a
    /// sine: between two tips it goes on along the outline, so the
    /// waves never cross or join however high the teeth are (10 mm
    /// teeth every 8 mm), and it crosses the outline midway between the
    /// tips.
    #[test]
    fn smooth_teeth_are_waves_that_never_cross() {
        let out = zipper_on_line(10.0, 15, true, 0.0);
        let segs: Vec<PathSeg> = out.segments().collect();
        assert_eq!(segs.len(), 16);
        let step = 130.0 / 16.0;
        for (i, seg) in segs.iter().enumerate() {
            let mut last_x = f64::NEG_INFINITY;
            for k in 0..=20 {
                let p = seg.eval(k as f64 / 20.0);
                assert!(p.x >= last_x - 1e-9, "wave {i} turns back at {p:?}");
                assert!(p.y.abs() <= 10.0 + 1e-9, "wave {i} overshoots at {p:?}");
                last_x = p.x;
            }
            // Within its own stretch of the line, crossing it midway.
            assert!((seg.eval(0.0).x - step * i as f64).abs() < 1e-9);
            assert!((seg.eval(1.0).x - step * (i + 1) as f64).abs() < 1e-9);
            let mid = seg.eval(0.5);
            assert!(mid.y.abs() < 1e-9 && (mid.x - step * (i as f64 + 0.5)).abs() < 1e-9);
            // A sine's shape: a quarter of the way the wave is at
            // cos(pi / 4) of the amplitude, within a percent.
            let q = seg.eval(0.25);
            let want = (std::f64::consts::PI * (q.x - step * i as f64) / step).cos() * 10.0;
            assert!((q.y.abs() - want.abs()).abs() < 0.1, "{q:?} vs {want}");
        }
        // Leaving and arriving along the line: the tips are the crests.
        for seg in &segs {
            let d = seg.eval(0.001) - seg.eval(0.0);
            assert!(d.y.abs() < d.x.abs() * 0.01, "{d:?}");
        }
        // A closed outline's wave closes without a corner.
        let sq = square(0.0, 0.0, 40.0);
        let out = distort(
            &sq,
            Distort::Zipper {
                amplitude: 6.0,
                frequency: 4,
                center: None,
                random: false,
                smooth: true,
                local: false,
                tilt_deg: 0.0,
                shifted_tips: false,
            },
        );
        assert_eq!((subpath_count(&out), closes(&out)), (1, 1));
        assert!(out
            .elements()
            .iter()
            .all(|e| !matches!(e, PathEl::LineTo(_))));
    }

    #[test]
    fn zipper_teeth_on_every_segment() {
        let sq = square(0.0, 0.0, 20.0);
        let out = distort(&sq, zip(false, false, false));
        // 4 sides of 4 lines (frequency 3 plus one), one tip each, closed.
        assert_eq!(nodes(&out).len(), 16);
        assert_eq!(closes(&out), 1);
        // The teeth reach 2 mm out and in along the sides; the corners
        // are tips too, moved between the normals of their sides.
        let b = out.bounding_box();
        assert!(
            (b.x0 + 2.0).abs() < 1e-6 && (b.x1 - 22.0).abs() < 1e-6,
            "{b:?}"
        );
        let corner = Point::new(20.0, 0.0);
        assert!(!nodes(&out).contains(&corner));
        let moved = nodes(&out)
            .into_iter()
            .map(|p| (p - corner).hypot())
            .fold(f64::INFINITY, f64::min);
        assert!((moved - 2.0).abs() < 1e-6, "{moved}");
        // No frequency: unchanged.
        let none = Distort::Zipper {
            amplitude: 2.0,
            frequency: 0,
            center: None,
            random: false,
            smooth: false,
            local: false,
            tilt_deg: 0.0,
            shifted_tips: false,
        };
        assert_eq!(distort(&sq, none), sq);
        // Random: smaller teeth, the same each time.
        let a = distort(&sq, zip(true, false, false));
        assert_eq!(a, distort(&sq, zip(true, false, false)));
        let ab = a.bounding_box();
        assert!(ab.width() <= 24.0 + 1e-9 && ab != b);
        // Local: smaller teeth away from the centre (none at the corners).
        let lb = distort(&sq, zip(false, false, true)).bounding_box();
        assert!(lb.width() < b.width());
        // Smooth: curves through the teeth.
        assert!(distort(&sq, zip(false, true, false))
            .elements()
            .iter()
            .any(|e| matches!(e, PathEl::CurveTo(..))));
    }

    /// A straight line with the Zipper (#1108, #1122): every point of
    /// the zigzag, the two ends included, is a tooth's tip, alternately
    /// on each side; no node is left on the line; frequency 6 gives
    /// seven lines.
    #[test]
    fn a_zipped_line_starts_and_ends_on_tips() {
        let mut line = BezPath::new();
        line.move_to((10.0, 0.0));
        line.line_to((10.0, 70.0));
        let out = distort(
            &line,
            Distort::Zipper {
                amplitude: 3.0,
                frequency: 6,
                center: None,
                random: false,
                smooth: false,
                local: false,
                tilt_deg: 0.0,
                shifted_tips: false,
            },
        );
        let n = nodes(&out);
        assert_eq!(n.len(), 8, "seven lines");
        assert_eq!((subpath_count(&out), closes(&out)), (1, 0));
        for (i, p) in n.iter().enumerate() {
            assert!(((p.x - 10.0).abs() - 3.0).abs() < 1e-9, "{i}: {p:?}");
            assert!((p.y - 10.0 * i as f64).abs() < 1e-9, "{i}: {p:?}");
            if i > 0 {
                assert!((p.x - 10.0) * (n[i - 1].x - 10.0) < 0.0, "alternate");
            }
        }
        // Two segments: the pattern goes on through the middle node, no
        // two neighbouring tips on the same side.
        let mut bent = BezPath::new();
        bent.move_to((0.0, 0.0));
        bent.line_to((30.0, 0.0));
        bent.line_to((30.0, 30.0));
        let out = distort(
            &bent,
            Distort::Zipper {
                amplitude: 1.0,
                frequency: 3,
                center: None,
                random: false,
                smooth: true,
                local: false,
                tilt_deg: 0.0,
                shifted_tips: false,
            },
        );
        let n = nodes(&out);
        assert_eq!(n.len(), 9, "two segments of four lines");
        let side = |p: &Point| -> f64 {
            // Signed distance from the nearer side of the bent line,
            // outside positive.
            if p.y.abs() <= (p.x - 30.0).abs() {
                -p.y
            } else {
                p.x - 30.0
            }
        };
        for i in 1..n.len() {
            assert!(
                side(&n[i]) * side(&n[i - 1]) < 0.0,
                "{i}: {:?}",
                &n[i - 1..=i]
            );
        }
        // A closed outline with an odd count of tips (a triangle with
        // frequency 2: three sides of three lines) takes one line more
        // on its last side, so the last tip alternates with the first.
        let mut tri = BezPath::new();
        tri.move_to((0.0, 0.0));
        tri.line_to((40.0, 0.0));
        tri.line_to((20.0, 30.0));
        tri.close_path();
        let out = distort(
            &tri,
            Distort::Zipper {
                amplitude: 1.0,
                frequency: 2,
                center: None,
                random: false,
                smooth: false,
                local: false,
                tilt_deg: 0.0,
                shifted_tips: false,
            },
        );
        let n = nodes(&out);
        assert_eq!(n.len(), 10);
        let c = Point::new(20.0, 10.0);
        let outward = |p: &Point| -> bool {
            // Farther from the middle than the nearest point of the
            // triangle's own outline: the tip points out.
            let (on, _) = tri
                .segments()
                .map(|s| {
                    let q = s.nearest(*p, 1e-6);
                    (s.eval(q.t), q.distance_sq)
                })
                .fold((Point::ZERO, f64::INFINITY), |best, cur| {
                    if cur.1 < best.1 {
                        cur
                    } else {
                        best
                    }
                });
            (*p - c).hypot() > (on - c).hypot()
        };
        for i in 0..n.len() {
            let j = (i + 1) % n.len();
            assert_ne!(outward(&n[i]), outward(&n[j]), "{i}: {:?} {:?}", n[i], n[j]);
        }
    }

    /// A straight line with the Twister (#1108) is a smooth spiral from
    /// end to end: every step lies on the true turned curve and the ends
    /// leave along its direction, not along a straight tail. The ends,
    /// the farthest points, stay where they are and the middle turns by
    /// the whole angle (#1674).
    #[test]
    fn a_twisted_line_is_a_smooth_spiral_to_its_ends() {
        let mut line = BezPath::new();
        line.move_to((0.0, -50.0));
        line.line_to((0.0, 50.0));
        let turns = 4.0;
        let out = distort(
            &line,
            Distort::Twister {
                angle_deg: -360.0 * turns,
                center: None,
            },
        );
        assert_eq!((subpath_count(&out), closes(&out)), (1, 0));
        // Fine enough: a step at most a 24th of a turn, each half of
        // the line turning four times.
        let segs: Vec<PathSeg> = out.segments().collect();
        assert!(segs.len() >= 2 * 24 * turns as usize, "{}", segs.len());
        // The true curve: a point at distance r from the middle turned
        // by 1 - r / 50 of the angle, so the ends stay.
        let angle = (-360.0f64 * turns).to_radians();
        let truth = |y: f64| -> Point {
            let a = angle * (1.0 - y.abs() / 50.0);
            Point::new(-y * a.sin(), y * a.cos())
        };
        let n = nodes(&out);
        assert!((n[0] - Point::new(0.0, -50.0)).hypot() < 1e-9, "{:?}", n[0]);
        assert!(
            (n[n.len() - 1] - Point::new(0.0, 50.0)).hypot() < 1e-9,
            "{:?}",
            n[n.len() - 1]
        );
        for seg in &segs {
            for k in 0..=4 {
                let p = seg.eval(k as f64 / 4.0);
                let r = p.to_vec2().hypot();
                // The point is on the spiral: at its distance the curve
                // passes through the same angle.
                let want = truth(r);
                let want2 = truth(-r);
                let off = (p - want).hypot().min((p - want2).hypot());
                assert!(off < 0.03, "{p:?} off by {off}");
            }
        }
        // The end leaves tangent to the spiral (the derivative of the
        // turn), not along the chord to the next step.
        let first = segs[0];
        let d0 = (first.eval(1e-3) - first.eval(0.0)).normalize();
        let want = (truth(-50.0 + 1e-4) - truth(-50.0)).normalize();
        assert!(d0.dot(want) > 0.999, "{d0:?} vs {want:?}");
        let last = segs[segs.len() - 1];
        let d1 = (last.eval(1.0) - last.eval(1.0 - 1e-3)).normalize();
        let want = (truth(50.0) - truth(50.0 - 1e-4)).normalize();
        assert!(d1.dot(want) > 0.999, "{d1:?} vs {want:?}");
        // No two neighbouring steps meet at a corner.
        for w in segs.windows(2) {
            let a = (w[0].eval(1.0) - w[0].eval(0.999)).normalize();
            let b = (w[1].eval(0.001) - w[1].eval(0.0)).normalize();
            assert!(a.dot(b) > 0.99, "corner between steps: {a:?} {b:?}");
        }
    }

    /// #1674: the farthest points keep their places and the turn grows
    /// toward the centre, so the outline does not look rotated as a
    /// whole: a square's corners stay at the corners and its sides swirl.
    #[test]
    fn twister_keeps_the_far_points_and_turns_the_inside() {
        let sq = square(-10.0, -10.0, 20.0);
        let t = distort(
            &sq,
            Distort::Twister {
                angle_deg: 90.0,
                center: None,
            },
        );
        // The corners (farthest) stay; the middle of the top side, at
        // 10 of the corners' 14.14, turns by 90 * (1 - 10 / 14.14).
        let n = nodes(&t);
        for corner in [(10.0, 10.0), (-10.0, 10.0), (-10.0, -10.0), (10.0, -10.0)] {
            let c = Point::new(corner.0, corner.1);
            assert!(n.iter().any(|p| (*p - c).hypot() < 1e-6), "{c:?} in {n:?}");
        }
        let a = (90.0f64 * (1.0 - 10.0 / 200f64.sqrt())).to_radians();
        let mid = Point::new(-10.0 * a.sin(), 10.0 * a.cos());
        let off = t
            .segments()
            .map(|s| s.nearest(mid, 1e-9).distance_sq.sqrt())
            .fold(f64::INFINITY, f64::min);
        assert!(off < 0.02, "{mid:?} off by {off}");
        assert_eq!((subpath_count(&t), closes(&t)), (1, 1));
        // The bounds of a quarter turn of a square about its middle stay
        // near the square's.
        let b = t.bounding_box();
        assert!(b.width() < 30.0 && b.width() > 18.0, "{b:?}");
        // A star turned two whole turns about its own middle keeps its
        // tips in place while its inner nodes turn by 1 - 16 / 40 of the
        // angle.
        let mut star = BezPath::new();
        let tip = |i: usize| {
            let a = std::f64::consts::FRAC_PI_2 + i as f64 * std::f64::consts::PI / 5.0;
            let r = if i % 2 == 0 { 40.0 } else { 16.0 };
            Point::new(r * a.cos(), r * a.sin())
        };
        star.move_to(tip(0));
        for i in 1..10 {
            star.line_to(tip(i));
        }
        star.close_path();
        let ts = distort(
            &star,
            Distort::Twister {
                angle_deg: -720.0,
                center: Some(Point::ZERO),
            },
        );
        let n = nodes(&ts);
        let inner_turn = (-720.0f64 * (1.0 - 16.0 / 40.0)).to_radians();
        for i in 0..10 {
            let want = if i % 2 == 0 {
                tip(i)
            } else {
                let p = tip(i);
                let (s, c) = inner_turn.sin_cos();
                Point::new(p.x * c - p.y * s, p.x * s + p.y * c)
            };
            assert!(
                n.iter().any(|p| (*p - want).hypot() < 1e-6),
                "node {i}: {want:?} not among the nodes"
            );
        }
        let sb = ts.bounding_box();
        let ob = star.bounding_box();
        assert!((sb.y1 - ob.y1).abs() < 1e-6, "the top tip stays: {sb:?}");
        // An ellipse (no closing element, but closed) stays closed.
        let e = crate::geometry::ellipse_path(Rect::new(0.0, 0.0, 30.0, 20.0));
        let te = distort(
            &e,
            Distort::Twister {
                angle_deg: 60.0,
                center: None,
            },
        );
        assert_eq!((subpath_count(&te), closes(&te)), (1, 1));
        // Zero: unchanged; an odd centre falls back to the middle.
        let z = Distort::Twister {
            angle_deg: 0.0,
            center: None,
        };
        assert_eq!(distort(&sq, z), sq);
        let odd = Distort::Twister {
            angle_deg: 90.0,
            center: Some(Point::new(f64::NAN, 0.0)),
        };
        assert_eq!(distort(&sq, odd), t);
    }

    #[test]
    fn older_files_and_odd_values_read() {
        let d: Distort = serde_json::from_str(r#"{"mode":"twister","angle_deg":140.0}"#)
            .expect("an older twister");
        assert_eq!(
            d,
            Distort::Twister {
                angle_deg: 140.0,
                center: None
            }
        );
        let z: Distort =
            serde_json::from_str(r#"{"mode":"zipper","amplitude":2.0,"frequency":12}"#)
                .expect("an older zipper");
        assert!(matches!(
            z,
            Distort::Zipper {
                random: false,
                smooth: false,
                local: false,
                tilt_deg: 0.0,
                center: None,
                ..
            }
        ));
        // Written without the defaults.
        let s = serde_json::to_string(&z).expect("json");
        assert!(!s.contains("random") && !s.contains("center"), "{s}");
        // Out of range and not a number: no panic, finite output.
        let sq = square(0.0, 0.0, 10.0);
        for how in [
            Distort::PushPull {
                amount: f64::NAN,
                center: None,
            },
            Distort::PushPull {
                amount: 1e9,
                center: None,
            },
            Distort::Zipper {
                amplitude: f64::INFINITY,
                frequency: u32::MAX,
                center: None,
                random: true,
                smooth: true,
                local: true,
                tilt_deg: 0.0,
                shifted_tips: false,
            },
            Distort::Twister {
                angle_deg: 1e12,
                center: Some(Point::new(1e300, 0.0)),
            },
        ] {
            let out = distort(&sq, how);
            let b = out.bounding_box();
            assert!(b.x0.is_finite() && b.y1.is_finite(), "{how:?} {b:?}");
        }
        assert_eq!(
            distort(
                &BezPath::new(),
                Distort::PushPull {
                    amount: 10.0,
                    center: None
                }
            ),
            BezPath::new()
        );
        let _ = Rect::ZERO;
    }
    /// Shifted tips, the teeth of `.cdr` files (#1715), as the Zipper
    /// files of #1736 draw them on a 133.39 mm line: the points alternate
    /// from a tip on the normal's side (up for a line to the right), the
    /// tips moved by a fifth of the way from the line's middle to the
    /// centre whatever the amplitude, the bases straight down. With the
    /// centre 60.33 mm left of the middle and 1.86 mm under the line, the
    /// first tip lies 12.07 mm left of the line's start and the tips
    /// reach 0.37 mm less high than the bases go low ("Zipper Traning
    /// Data 17" to "21"). Older files have no such teeth.
    #[test]
    fn shifted_tips_start_with_a_tip_moved_toward_the_centre() {
        let mut line = BezPath::new();
        line.move_to((0.0, 0.0));
        line.line_to((133.39, 0.0));
        let mid = Point::new(66.695, 0.0);
        let center = mid + Vec2::new(-60.33, -1.86);
        for a in [16.674, 6.836] {
            let z = Distort::Zipper {
                amplitude: a,
                frequency: 15,
                center: Some(center),
                random: false,
                smooth: false,
                local: false,
                tilt_deg: 30.0,
                shifted_tips: true,
            };
            let out = distort(&line, z);
            let pts: Vec<Point> = out
                .elements()
                .iter()
                .filter_map(|e| match *e {
                    PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(p),
                    _ => None,
                })
                .collect();
            assert_eq!(pts.len(), 17);
            let shift = Vec2::new(-60.33, -1.86) * SHIFT_SHARE;
            for (k, p) in pts.iter().enumerate() {
                let base = Point::new(133.39 * k as f64 / 16.0, 0.0);
                let want = if k % 2 == 0 {
                    base + Vec2::new(0.0, a) + shift
                } else {
                    base + Vec2::new(0.0, -a)
                };
                assert!((*p - want).hypot() < 1e-6, "{k}: {p:?} {want:?}");
            }
            let b = out.bounding_box();
            assert!((b.x0 + 12.066).abs() < 1e-6, "{b:?}");
            assert!(
                (b.y1 - (a - 0.372)).abs() < 1e-6 && (b.y0 + a).abs() < 1e-6,
                "{b:?}"
            );
        }
        let old: Distort =
            serde_json::from_str(r#"{"mode":"zipper","amplitude":5.0,"frequency":3}"#)
                .expect("old");
        assert!(matches!(
            old,
            Distort::Zipper {
                shifted_tips: false,
                ..
            }
        ));
        let json = serde_json::to_string(&Distort::Zipper {
            amplitude: 5.0,
            frequency: 3,
            center: None,
            random: false,
            smooth: false,
            local: false,
            tilt_deg: 0.0,
            shifted_tips: true,
        })
        .expect("json");
        assert!(json.contains(r#""shifted_tips":true"#), "{json}");
    }
}
