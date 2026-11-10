//! Weeding: extra cuts in the waste around and between objects so it
//! peels off the backing in small pieces.
//!
//! Weeding lines: from each object, straight lines run out of its
//! bounding box (`dividers` lines on each side, evenly spaced along that
//! side) until they meet a neighbouring object or the weeding frame. The
//! lines never cross an object: every object is grown by `gap` (its
//! closed outlines offset outward, its open ones widened) and the lines
//! stop where they reach a grown object. A line starts where it leaves
//! its own grown object, and each end that stops at an object is pulled
//! back by `shorten`; ends on the frame touch it, so the waste comes apart.
//! Weeding boxes: a rectangle around each object, its bounding box grown
//! by `box_x` left and right and `box_y` below and above; boxes that
//! overlap are welded into one outline. The weeding frame is the bounding
//! box of everything (the objects, or the boxes) grown by `margin`.

use super::{bounds, fin, finite, line_path, polys, rect_path, Poly, FLATTEN_TOLERANCE};
use tracedraw_core::geometry::{BezPath, PathEl, Point, Rect};
use tracedraw_core::shaping;

/// Weeding lines are dropped when shorter than this (mm).
pub const MIN_LINE: f64 = 0.5;

/// Lines whose ends lie this close to another line's ends are the same
/// line (mm).
const SAME_LINE: f64 = 0.5;

/// What [`weeding`] draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum WeedMode {
    /// Lines out of each object to its neighbours and the frame.
    #[default]
    Lines,
    /// A rectangle around each object.
    Boxes,
}

/// Settings of [`weeding`], in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct WeedSpec {
    pub mode: WeedMode,
    /// Distance the lines keep from the objects.
    pub gap: f64,
    /// Each end of a line that stops at an object is pulled back by this.
    pub shorten: f64,
    /// Lines on each side of an object's box (0 to 20).
    pub dividers: u32,
    /// Draw the weeding frame.
    pub frame: bool,
    /// Distance from the objects (or boxes) to the frame.
    pub margin: f64,
    /// Weeding boxes: growth left and right, below and above.
    pub box_x: f64,
    pub box_y: f64,
}

impl Default for WeedSpec {
    fn default() -> Self {
        WeedSpec {
            mode: WeedMode::Lines,
            gap: 2.0,
            shorten: 1.0,
            dividers: 1,
            frame: true,
            margin: 5.0,
            box_x: 2.0,
            box_y: 2.0,
        }
    }
}

/// The largest number of dividers per side.
pub const MAX_DIVIDERS: u32 = 20;

/// An object grown by the gap: regions, each a set of contours read with
/// the even-odd rule.
struct Obstacle {
    regions: Vec<Vec<Vec<Point>>>,
    bounds: Rect,
}

impl Obstacle {
    fn contains(&self, p: Point) -> bool {
        if !(p.x >= self.bounds.x0
            && p.x <= self.bounds.x1
            && p.y >= self.bounds.y0
            && p.y <= self.bounds.y1)
        {
            return false;
        }
        self.regions
            .iter()
            .any(|contours| contours.iter().filter(|c| super::inside(p, c)).count() % 2 == 1)
    }

    /// Parameters (0 to 1) where segment `a`-`b` crosses the outline.
    fn crossings(&self, a: Point, b: Point, out: &mut Vec<f64>) {
        let seg = Rect::from_points(a, b);
        if seg.x1 < self.bounds.x0
            || seg.x0 > self.bounds.x1
            || seg.y1 < self.bounds.y0
            || seg.y0 > self.bounds.y1
        {
            return;
        }
        for contours in &self.regions {
            for c in contours {
                let n = c.len();
                for i in 0..n {
                    let p = c[i];
                    let q = c[(i + 1) % n];
                    if let Some(t) = cross(a, b, p, q) {
                        out.push(t);
                    }
                }
            }
        }
    }
}

/// Where segment `a`-`b` crosses segment `p`-`q`, as a parameter along
/// `a`-`b`.
fn cross(a: Point, b: Point, p: Point, q: Point) -> Option<f64> {
    let r = b - a;
    let s = q - p;
    let den = r.x * s.y - r.y * s.x;
    if den.abs() < 1e-12 {
        return None;
    }
    let w = p - a;
    let t = (w.x * s.y - w.y * s.x) / den;
    let u = (w.x * r.y - w.y * r.x) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some(t)
}

fn contour_points(path: &BezPath) -> Vec<Vec<Point>> {
    shaping::contours(path)
        .into_iter()
        .map(|c| c.into_iter().map(|[x, y]| Point::new(x, y)).collect())
        .collect()
}

/// An object (its outlines) grown by `gap`.
fn obstacle(paths: &[BezPath], gap: f64) -> Option<Obstacle> {
    let mut closed = BezPath::new();
    let mut opens: Vec<Poly> = Vec::new();
    for path in paths {
        for poly in polys(path, FLATTEN_TOLERANCE) {
            if poly.closed && poly.pts.len() >= 3 {
                closed.extend(super::polygon_path(&poly.pts));
            } else {
                opens.push(poly);
            }
        }
    }
    let mut regions = Vec::new();
    if !closed.elements().is_empty() {
        let grown = if gap > 1e-9 {
            shaping::offset(&closed, gap)
        } else {
            closed
        };
        let c = contour_points(&grown);
        if !c.is_empty() {
            regions.push(c);
        }
    }
    for o in opens {
        let band = shaping::stroke_band(&o.pts, (2.0 * gap).max(0.02), false);
        let c = contour_points(&band);
        if !c.is_empty() {
            regions.push(c);
        }
    }
    let all: Vec<Point> = regions.iter().flatten().flatten().copied().collect();
    let b = super::poly_bounds(&all)?;
    Some(Obstacle { regions, bounds: b })
}

/// One piece of a ray: from `t0` to `t1`, inside obstacle `owner` (if any).
#[derive(Debug, Clone, Copy)]
struct Span {
    t0: f64,
    t1: f64,
    owner: Option<usize>,
}

/// The weeding cuts for the objects (each given by its outlines in page
/// space): open lines, closed boxes and the frame, as separate paths.
/// Objects without finite outlines are left out.
pub fn weeding(objects: &[Vec<BezPath>], spec: &WeedSpec) -> Vec<BezPath> {
    let gap = fin(spec.gap, 0.0).max(0.0);
    let margin = fin(spec.margin, 0.0).max(0.0);
    let boxes: Vec<Option<Rect>> = objects.iter().map(|o| bounds(o)).collect();
    let mut out = Vec::new();
    let all = boxes.iter().flatten().copied().reduce(|a, b| a.union(b));
    let Some(all) = all else {
        return out;
    };
    match spec.mode {
        WeedMode::Lines => {
            let frame = all.inflate(margin, margin);
            out.extend(lines(objects, &boxes, frame, gap, spec));
            if spec.frame {
                out.push(rect_path(frame));
            }
        }
        WeedMode::Boxes => {
            let bx = fin(spec.box_x, 0.0).max(0.0);
            let by = fin(spec.box_y, 0.0).max(0.0);
            let grown: Vec<Rect> = boxes.iter().flatten().map(|b| b.inflate(bx, by)).collect();
            let mut welded = BezPath::new();
            for r in &grown {
                welded = if welded.elements().is_empty() {
                    rect_path(*r)
                } else {
                    shaping::overlay(&welded, &rect_path(*r), shaping::Op::Weld)
                };
            }
            out.extend(split_subpaths(&welded));
            if spec.frame {
                let around = grown
                    .iter()
                    .copied()
                    .reduce(|a, b| a.union(b))
                    .unwrap_or(all);
                out.push(rect_path(around.inflate(margin, margin)));
            }
        }
    }
    out
}

/// Each subpath of a path as a path of its own.
fn split_subpaths(path: &BezPath) -> Vec<BezPath> {
    let mut out: Vec<BezPath> = Vec::new();
    for el in path.elements() {
        if let PathEl::MoveTo(_) = el {
            out.push(BezPath::new());
        }
        if let Some(cur) = out.last_mut() {
            cur.push(*el);
        }
    }
    out.retain(|p| p.elements().len() > 2);
    out
}

fn lines(
    objects: &[Vec<BezPath>],
    boxes: &[Option<Rect>],
    frame: Rect,
    gap: f64,
    spec: &WeedSpec,
) -> Vec<BezPath> {
    let shorten = fin(spec.shorten, 0.0).max(0.0);
    let n = spec.dividers.min(MAX_DIVIDERS);
    // Obstacles by object index (None: nothing to cut).
    let obstacles: Vec<Option<Obstacle>> = objects.iter().map(|o| obstacle(o, gap)).collect();
    let present: Vec<(usize, &Obstacle)> = obstacles
        .iter()
        .enumerate()
        .filter_map(|(i, o)| o.as_ref().map(|o| (i, o)))
        .collect();
    let mut found: Vec<(Point, Point)> = Vec::new();
    for (i, b) in boxes.iter().enumerate() {
        let Some(b) = b else { continue };
        let Some(own_k) = present.iter().position(|(j, _)| *j == i) else {
            continue;
        };
        let c = b.center();
        for k in 1..=n {
            let f = f64::from(k) / f64::from(n + 1);
            let y = b.y0 + b.height() * f;
            let x = b.x0 + b.width() * f;
            // Inner point (on the box's centre line), side point, frame point.
            let rays = [
                (
                    Point::new(c.x, y),
                    Point::new(b.x1, y),
                    Point::new(frame.x1, y),
                ),
                (
                    Point::new(c.x, y),
                    Point::new(b.x0, y),
                    Point::new(frame.x0, y),
                ),
                (
                    Point::new(x, c.y),
                    Point::new(x, b.y1),
                    Point::new(x, frame.y1),
                ),
                (
                    Point::new(x, c.y),
                    Point::new(x, b.y0),
                    Point::new(x, frame.y0),
                ),
            ];
            for (from, side, to) in rays {
                if let Some(l) = ray(from, side, to, own_k, &present, shorten) {
                    found.push(l);
                }
            }
        }
    }
    let mut out: Vec<(Point, Point)> = Vec::new();
    for (a, b) in found {
        let same = out.iter().any(|(p, q)| {
            ((*p - a).hypot() < SAME_LINE && (*q - b).hypot() < SAME_LINE)
                || ((*p - b).hypot() < SAME_LINE && (*q - a).hypot() < SAME_LINE)
        });
        if !same {
            out.push((a, b));
        }
    }
    out.into_iter().map(|(a, b)| line_path(a, b)).collect()
}

/// One weeding line along the ray `from` to `to` for the object `own`
/// (an index into `present`), or `None`.
fn ray(
    from: Point,
    side: Point,
    to: Point,
    own: usize,
    present: &[(usize, &Obstacle)],
    shorten: f64,
) -> Option<(Point, Point)> {
    if !finite(from) || !finite(to) {
        return None;
    }
    let len = (to - from).hypot();
    if len < 1e-9 {
        return None;
    }
    let obstacles: Vec<&Obstacle> = present.iter().map(|(_, o)| *o).collect();
    let mut ts = vec![0.0, 1.0];
    for o in &obstacles {
        o.crossings(from, to, &mut ts);
    }
    ts.retain(|t| t.is_finite());
    ts.sort_by(f64::total_cmp);
    ts.dedup_by(|x, y| (*x - *y).abs() < 1e-9);
    let spans: Vec<Span> = ts
        .windows(2)
        .map(|w| {
            let m = from.lerp(to, (w[0] + w[1]) / 2.0);
            Span {
                t0: w[0],
                t1: w[1],
                owner: obstacles.iter().position(|o| o.contains(m)),
            }
        })
        .collect();
    let t_side = (side - from).hypot() / len;
    // Leave the own object where it is left for the last time.
    let start = spans
        .iter()
        .filter(|s| s.owner == Some(own))
        .map(|s| s.t1)
        .fold(None, |m: Option<f64>, t| Some(m.map_or(t, |m| m.max(t))))
        .unwrap_or(t_side);
    let mut end = None;
    let mut at_object = false;
    for s in spans.iter().filter(|s| s.t1 > start + 1e-12) {
        if s.owner.is_some() {
            if s.t0 <= start + 1e-12 {
                // Starts inside another object: no line.
                return None;
            }
            end = Some(s.t0);
            at_object = true;
            break;
        }
        end = Some(s.t1);
    }
    let end = end?;
    let mut a = start * len + shorten;
    let mut b = end * len;
    if at_object {
        b -= shorten;
    }
    a = a.max(0.0);
    b = b.min(len);
    if b - a < MIN_LINE {
        return None;
    }
    let dir = (to - from) / len;
    Some((from + dir * a, from + dir * b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Shape as _;

    fn square(x: f64, y: f64, s: f64) -> Vec<BezPath> {
        vec![rect_path(Rect::new(x, y, x + s, y + s))]
    }

    fn open_lines(out: &[BezPath]) -> Vec<(Point, Point)> {
        out.iter()
            .filter(|p| p.elements().len() == 2)
            .filter_map(|p| match (p.elements()[0], p.elements()[1]) {
                (PathEl::MoveTo(a), PathEl::LineTo(b)) => Some((a, b)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn one_object_gets_four_lines_and_a_frame() {
        let spec = WeedSpec::default();
        let out = weeding(&[square(0.0, 0.0, 10.0)], &spec);
        let lines = open_lines(&out);
        assert_eq!(lines.len(), 4);
        assert_eq!(out.len(), 5);
        // The frame: the box grown by the margin.
        let frame = out.last().map(|p| p.bounding_box()).expect("frame");
        assert!((frame.x0 + 5.0).abs() < 1e-9 && (frame.x1 - 15.0).abs() < 1e-9);
        // The right line starts gap + shorten from the square and ends on the frame.
        let right = lines
            .iter()
            .find(|(a, b)| a.x > 10.0 && b.x > 10.0)
            .expect("right line");
        assert!((right.0.x - 13.0).abs() < 0.05, "{right:?}");
        assert!((right.1.x - 15.0).abs() < 1e-6, "{right:?}");
        assert!((right.0.y - 5.0).abs() < 1e-9);
    }

    #[test]
    fn lines_between_neighbours_stop_short_of_both() {
        let spec = WeedSpec {
            frame: false,
            ..Default::default()
        };
        let out = weeding(&[square(0.0, 0.0, 10.0), square(30.0, 0.0, 10.0)], &spec);
        let lines = open_lines(&out);
        // The two lines between the squares are the same line, kept once.
        let between: Vec<_> = lines
            .iter()
            .filter(|(a, b)| a.x > 10.0 && a.x < 30.0 && b.x > 10.0 && b.x < 30.0)
            .collect();
        assert_eq!(between.len(), 1, "{lines:?}");
        let (a, b) = between[0];
        let (l, r) = if a.x < b.x { (a, b) } else { (b, a) };
        assert!(
            (l.x - 13.0).abs() < 0.05 && (r.x - 27.0).abs() < 0.05,
            "{l:?} {r:?}"
        );
        assert_eq!(lines.len(), 7);
    }

    #[test]
    fn lines_never_cross_an_object() {
        // A wide bar above a small square: the square's upward line stops
        // under the bar.
        let bar = vec![rect_path(Rect::new(-20.0, 20.0, 40.0, 25.0))];
        let spec = WeedSpec {
            frame: false,
            dividers: 3,
            ..Default::default()
        };
        let objects = [square(0.0, 0.0, 10.0), bar];
        let out = weeding(&objects, &spec);
        let grown: Vec<Rect> = objects
            .iter()
            .map(|o| bounds(o).expect("box").inflate(1.99, 1.99))
            .collect();
        for (a, b) in open_lines(&out) {
            for k in 0..=100 {
                let p = a.lerp(b, f64::from(k) / 100.0);
                for g in &grown {
                    assert!(!g.contains(p), "{a:?}-{b:?} enters {g:?}");
                }
            }
        }
        assert!(!out.is_empty());
    }

    #[test]
    fn a_ring_starts_its_lines_outside_the_ring() {
        let mut ring = rect_path(Rect::new(0.0, 0.0, 20.0, 20.0));
        ring.extend(rect_path(Rect::new(5.0, 5.0, 15.0, 15.0)));
        let spec = WeedSpec {
            frame: false,
            ..Default::default()
        };
        let lines = open_lines(&weeding(&[vec![ring]], &spec));
        assert_eq!(lines.len(), 4);
        for (a, _) in lines {
            assert!(
                a.x < -2.5 || a.x > 22.5 || a.y < -2.5 || a.y > 22.5,
                "{a:?}"
            );
        }
    }

    #[test]
    fn boxes_grow_and_weld() {
        let spec = WeedSpec {
            mode: WeedMode::Boxes,
            frame: false,
            box_x: 3.0,
            box_y: 1.0,
            ..Default::default()
        };
        let out = weeding(&[square(0.0, 0.0, 10.0), square(12.0, 0.0, 10.0)], &spec);
        assert_eq!(out.len(), 1, "the overlapping boxes are one outline");
        let b = out[0].bounding_box();
        assert!((b.x0 + 3.0).abs() < 1e-6 && (b.x1 - 25.0).abs() < 1e-6);
        assert!((b.y0 + 1.0).abs() < 1e-6 && (b.y1 - 11.0).abs() < 1e-6);
        let apart = weeding(&[square(0.0, 0.0, 10.0), square(50.0, 0.0, 10.0)], &spec);
        assert_eq!(apart.len(), 2);
        let framed = weeding(
            &[square(0.0, 0.0, 10.0)],
            &WeedSpec {
                frame: true,
                ..spec
            },
        );
        assert_eq!(framed.len(), 2);
        let f = framed[1].bounding_box();
        assert!((f.x0 + 8.0).abs() < 1e-6 && (f.y0 + 6.0).abs() < 1e-6);
    }

    #[test]
    fn open_curves_are_obstacles_too() {
        let line = vec![line_path(Point::new(0.0, 0.0), Point::new(0.0, 10.0))];
        let spec = WeedSpec {
            frame: false,
            ..Default::default()
        };
        let lines = open_lines(&weeding(&[line], &spec));
        assert_eq!(lines.len(), 4);
        let right = lines.iter().find(|(a, _)| a.x > 0.5).expect("right");
        assert!((right.0.x - 3.0).abs() < 0.1, "{right:?}");
    }

    #[test]
    fn degenerate_and_non_finite_input_gives_nothing_or_stays_finite() {
        let spec = WeedSpec::default();
        assert!(weeding(&[], &spec).is_empty());
        assert!(weeding(&[vec![BezPath::new()]], &spec).is_empty());
        let mut bad = BezPath::new();
        bad.move_to((f64::NAN, 0.0));
        bad.line_to((f64::INFINITY, 1.0));
        assert!(weeding(&[vec![bad.clone()]], &spec).is_empty());
        let wild = WeedSpec {
            gap: f64::NAN,
            shorten: f64::INFINITY,
            margin: f64::NEG_INFINITY,
            dividers: 1000,
            ..Default::default()
        };
        let out = weeding(&[square(0.0, 0.0, 10.0), vec![bad]], &wild);
        for p in &out {
            assert!(super::super::finite_path(p));
        }
        // A point-like object.
        let dot = vec![line_path(Point::new(1.0, 1.0), Point::new(1.0, 1.0))];
        let _ = weeding(&[dot], &spec);
    }
}
