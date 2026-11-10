//! Cut order and direction: the route optimizer.
//!
//! The objects to cut are reordered, and inside an object its subpaths,
//! to shorten the travel with the knife up between cuts:
//!
//! 1. With `inner_first`, an outline lying inside a closed outline of
//!    another object (or of the same object) is cut before it, so a part
//!    does not move once it is free of the sheet. "Inside" means its box
//!    lies in the other outline's box, the box is smaller, and its first
//!    node lies in the other outline.
//! 2. Nearest neighbour: from `from` (the plotter's origin), the next
//!    object is the one whose nearest allowed start is closest.
//! 3. 2-opt on the order of the objects: a stretch of the order is
//!    reversed when that shortens the travel and keeps rule 1.
//! 4. Start points: a closed outline starts at its node nearest to where
//!    the knife is (`StartAt::Nearest`), at its node nearest the
//!    bottom-left corner of its box (`StartAt::BottomLeft`), or where it
//!    starts now (`StartAt::Keep`). An open path is cut from its nearer end
//!    unless the direction is kept.
//! 5. Direction: closed outlines keep theirs, or all run clockwise or all
//!    counter-clockwise (as seen on the page, y up).
//!
//! Objects marked not editable keep their subpaths, starts and
//! directions (groups, text); only their place in the order changes.
//! Travel is the straight distance between the end of one cut and the
//! start of the next, from `from` to the first start; the return to the
//! origin is not counted.

use super::{finite, finite_path, inside, polys, signed_area, FLATTEN_TOLERANCE};
use tracedraw_core::geometry::{BezPath, ParamCurve, PathEl, PathSeg, Point, Rect, Shape as _};

/// The largest number of objects 2-opt works on (above it the nearest
/// neighbour order is kept).
pub const MAX_TWO_OPT: usize = 2000;

/// Passes of 2-opt over the order at most.
const TWO_OPT_PASSES: usize = 20;

/// Direction of closed outlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Direction {
    #[default]
    Keep,
    Clockwise,
    CounterClockwise,
}

/// Where a closed outline starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum StartAt {
    Keep,
    /// At the node nearest to the end of the previous cut.
    #[default]
    Nearest,
    /// At the node nearest to the bottom-left corner of its box.
    BottomLeft,
}

/// Settings of [`plan`].
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct RouteSpec {
    pub direction: Direction,
    pub start: StartAt,
    /// Outlines inside others are cut first.
    pub inner_first: bool,
    /// Where the knife is before the first cut (page space).
    pub from: Point,
}

impl Default for RouteSpec {
    fn default() -> Self {
        RouteSpec {
            direction: Direction::Keep,
            start: StartAt::Nearest,
            inner_first: true,
            from: Point::ZERO,
        }
    }
}

/// One object to cut: its outlines in page space, and whether its
/// subpaths may be reordered, turned and reversed.
#[derive(Debug, Clone)]
pub struct RouteItem {
    pub paths: Vec<BezPath>,
    pub editable: bool,
}

/// The planned route.
#[derive(Debug, Clone, Default)]
pub struct Route {
    /// Item indices in cut order.
    pub order: Vec<usize>,
    /// For each item (by its index), its subpaths in cut order, each a
    /// path of its own with its new start and direction.
    pub cuts: Vec<Vec<BezPath>>,
    /// For each item, whether its subpaths changed order, start or
    /// direction.
    pub changed: Vec<bool>,
    /// Travel in the given order, everything as drawn (mm).
    pub travel_before: f64,
    /// Travel of the planned route (mm).
    pub travel_after: f64,
    /// The moves with the knife up, in order.
    pub moves: Vec<(Point, Point)>,
    /// The moves with the knife up in the given order, everything as
    /// drawn.
    pub moves_before: Vec<(Point, Point)>,
    /// Where the last cut ends (`from` when nothing is cut).
    pub end: Point,
}

/// A subpath ready for routing.
#[derive(Debug, Clone)]
struct Sub {
    segs: Vec<PathSeg>,
    closed: bool,
    /// Closed: the start of every segment; open: both ends.
    nodes: Vec<Point>,
    start: Point,
    end: Point,
    bbox: Rect,
    poly: Vec<Point>,
    area: f64,
    /// The node nearest the bottom-left corner of the box.
    bottom_left: usize,
}

impl Sub {
    fn new(path: &BezPath, closed: bool) -> Option<Sub> {
        if !finite_path(path) {
            return None;
        }
        let segs: Vec<PathSeg> = path.segments().collect();
        let start = segs.first()?.start();
        let end = segs.last()?.end();
        let nodes: Vec<Point> = if closed {
            segs.iter().map(|s| s.start()).collect()
        } else {
            vec![start, end]
        };
        let poly = polys(path, FLATTEN_TOLERANCE)
            .into_iter()
            .next()
            .map(|p| p.pts)
            .unwrap_or_else(|| nodes.clone());
        let bbox = path.bounding_box();
        let corner = Point::new(bbox.x0, bbox.y0);
        let bottom_left = nodes
            .iter()
            .enumerate()
            .min_by(|a, b| (*a.1 - corner).hypot().total_cmp(&(*b.1 - corner).hypot()))
            .map_or(0, |(i, _)| i);
        Some(Sub {
            area: if closed { signed_area(&poly) } else { 0.0 },
            segs,
            closed,
            nodes,
            start,
            end,
            bbox,
            poly,
            bottom_left,
        })
    }

    /// The cost to start this subpath from `p`, the node it starts at,
    /// whether it runs backwards, and where it ends.
    fn entry(&self, p: Point, spec: &RouteSpec) -> (f64, usize, bool, Point) {
        if self.closed {
            let k = match spec.start {
                StartAt::Keep => 0,
                StartAt::BottomLeft => self.bottom_left,
                StartAt::Nearest => nearest(&self.nodes, p),
            };
            let reverse = match spec.direction {
                Direction::Keep => false,
                Direction::Clockwise => self.area > 0.0,
                Direction::CounterClockwise => self.area < 0.0,
            };
            let at = self.nodes.get(k).copied().unwrap_or(self.start);
            ((at - p).hypot(), k, reverse, at)
        } else {
            let ds = (self.start - p).hypot();
            let de = (self.end - p).hypot();
            if spec.direction != Direction::Keep && de < ds {
                (de, 0, true, self.start)
            } else {
                (ds, 0, false, self.end)
            }
        }
    }

    /// The subpath as a path, starting at node `k`, maybe reversed.
    fn rebuild(&self, k: usize, reverse: bool) -> BezPath {
        let mut segs: Vec<PathSeg> = self.segs.clone();
        if self.closed && k < segs.len() {
            segs.rotate_left(k);
        }
        if reverse {
            segs.reverse();
            for s in &mut segs {
                *s = s.reverse();
            }
        }
        let mut out = BezPath::new();
        if let Some(first) = segs.first() {
            out.move_to(first.start());
        }
        for s in &segs {
            out.push(match s {
                PathSeg::Line(l) => PathEl::LineTo(l.p1),
                PathSeg::Quad(q) => PathEl::QuadTo(q.p1, q.p2),
                PathSeg::Cubic(c) => PathEl::CurveTo(c.p1, c.p2, c.p3),
            });
        }
        if self.closed {
            out.close_path();
        }
        out
    }

    /// True when `other` lies inside this closed subpath.
    fn holds(&self, other: &Sub) -> bool {
        self.closed
            && contains_rect(self.bbox, other.bbox)
            && other.bbox.area() < self.bbox.area()
            && other.nodes.first().is_some_and(|p| inside(*p, &self.poly))
    }
}

fn nearest(nodes: &[Point], p: Point) -> usize {
    nodes
        .iter()
        .enumerate()
        .min_by(|a, b| (*a.1 - p).hypot().total_cmp(&(*b.1 - p).hypot()))
        .map_or(0, |(i, _)| i)
}

fn contains_rect(outer: Rect, inner: Rect) -> bool {
    const E: f64 = 1e-9;
    inner.x0 >= outer.x0 - E
        && inner.y0 >= outer.y0 - E
        && inner.x1 <= outer.x1 + E
        && inner.y1 <= outer.y1 + E
}

/// Distance from `p` to a rectangle (0 inside).
fn rect_distance(p: Point, r: Rect) -> f64 {
    let dx = (r.x0 - p.x).max(0.0).max(p.x - r.x1);
    let dy = (r.y0 - p.y).max(0.0).max(p.y - r.y1);
    dx.hypot(dy)
}

struct Item {
    subs: Vec<Sub>,
    editable: bool,
    bbox: Rect,
    /// For each subpath, the subpaths of the same item cut before it.
    inner: Vec<Vec<usize>>,
}

/// The steps of one item: (subpath, node, reversed).
type Steps = Vec<(usize, usize, bool)>;

impl Item {
    /// Cut this item starting from `p`: travel, end point, steps, moves.
    fn run(&self, mut p: Point, spec: &RouteSpec) -> (f64, Point, Steps, Vec<(Point, Point)>) {
        let mut travel = 0.0;
        let mut steps = Vec::with_capacity(self.subs.len());
        let mut moves = Vec::new();
        if !self.editable {
            for (i, s) in self.subs.iter().enumerate() {
                travel += (s.start - p).hypot();
                moves.push((p, s.start));
                steps.push((i, 0, false));
                p = s.end;
            }
            return (travel, p, steps, moves);
        }
        let mut done = vec![false; self.subs.len()];
        for _ in 0..self.subs.len() {
            let mut best: Option<(f64, usize, usize, bool, Point, Point)> = None;
            for (i, s) in self.subs.iter().enumerate() {
                if done[i] || !self.inner[i].iter().all(|j| done[*j]) {
                    continue;
                }
                let (c, k, r, exit) = s.entry(p, spec);
                if best.is_none_or(|b| c < b.0) {
                    let at = if s.closed {
                        exit
                    } else if r {
                        s.end
                    } else {
                        s.start
                    };
                    best = Some((c, i, k, r, at, exit));
                }
            }
            let Some((c, i, k, r, at, exit)) = best.or_else(|| {
                // A cycle (cannot happen with strict containment): take
                // the first one left.
                let i = done.iter().position(|d| !d)?;
                let s = &self.subs[i];
                Some(((s.start - p).hypot(), i, 0, false, s.start, s.end))
            }) else {
                break;
            };
            done[i] = true;
            travel += c;
            moves.push((p, at));
            steps.push((i, k, r));
            p = exit;
        }
        (travel, p, steps, moves)
    }

    /// The cost of the cheapest first cut of this item from `p`.
    fn entry_cost(&self, p: Point, spec: &RouteSpec) -> f64 {
        if !self.editable {
            return self.subs.first().map_or(0.0, |s| (s.start - p).hypot());
        }
        self.subs
            .iter()
            .enumerate()
            .filter(|(i, _)| self.inner[*i].is_empty())
            .map(|(_, s)| s.entry(p, spec).0)
            .fold(f64::INFINITY, f64::min)
    }
}

fn prepare(items: &[RouteItem], spec: &RouteSpec) -> Vec<Option<Item>> {
    items
        .iter()
        .map(|it| {
            let mut subs = Vec::new();
            let mut all_finite = true;
            for path in &it.paths {
                for sp in crate::subpaths(path) {
                    match Sub::new(&sp.path, sp.closed) {
                        Some(s) => subs.push(s),
                        None => all_finite = false,
                    }
                }
            }
            let bbox = subs.iter().map(|s| s.bbox).reduce(|a, b| a.union(b))?;
            if !super::finite_rect(bbox) {
                return None;
            }
            let inner = subs
                .iter()
                .map(|s| {
                    if !spec.inner_first {
                        return Vec::new();
                    }
                    subs.iter()
                        .enumerate()
                        .filter(|(_, o)| s.holds(o))
                        .map(|(j, _)| j)
                        .collect()
                })
                .collect();
            Some(Item {
                subs,
                editable: it.editable && all_finite,
                bbox,
                inner,
            })
        })
        .collect()
}

/// For each item, the items that must be cut before it.
fn precedence(items: &[Option<Item>], spec: &RouteSpec) -> Vec<Vec<usize>> {
    let n = items.len();
    let mut preds = vec![Vec::new(); n];
    if !spec.inner_first {
        return preds;
    }
    for (b, ib) in items.iter().enumerate() {
        let Some(ib) = ib else { continue };
        for (a, ia) in items.iter().enumerate() {
            let Some(ia) = ia else { continue };
            if a == b || !contains_rect(ib.bbox, ia.bbox) {
                continue;
            }
            let Some(first) = ia.subs.first() else {
                continue;
            };
            if ib.subs.iter().any(|s| {
                s.closed
                    && contains_rect(s.bbox, ia.bbox)
                    && ia.bbox.area() < s.bbox.area()
                    && inside(first.start, &s.poly)
            }) {
                preds[b].push(a);
            }
        }
    }
    preds
}

/// Exact travel of an order: total, steps per item, moves, and each
/// item's entry and exit points.
type Evaluation = (f64, Vec<Steps>, Vec<(Point, Point)>, Vec<(Point, Point)>);

/// Exact travel of an order (see [`Evaluation`]).
fn evaluate(items: &[Option<Item>], order: &[usize], spec: &RouteSpec) -> Evaluation {
    let mut p = spec.from;
    let mut total = 0.0;
    let mut steps = vec![Vec::new(); items.len()];
    let mut moves = Vec::new();
    let mut ends = vec![(Point::ZERO, Point::ZERO); items.len()];
    for &i in order {
        let Some(Some(it)) = items.get(i) else {
            continue;
        };
        let entry_from = p;
        let (t, exit, s, m) = it.run(p, spec);
        let entry = m.first().map_or(entry_from, |mv| mv.1);
        ends[i] = (entry, exit);
        total += t;
        steps[i] = s;
        moves.extend(m);
        p = exit;
    }
    (total, steps, moves, ends)
}

fn greedy(items: &[Option<Item>], preds: &[Vec<usize>], spec: &RouteSpec) -> Vec<usize> {
    let live: Vec<usize> = (0..items.len()).filter(|i| items[*i].is_some()).collect();
    let mut done = vec![false; items.len()];
    let mut order = Vec::with_capacity(live.len());
    let mut p = spec.from;
    for _ in 0..live.len() {
        let mut best: Option<(f64, usize)> = None;
        for &i in &live {
            if done[i] || !preds[i].iter().all(|j| done[*j] || items[*j].is_none()) {
                continue;
            }
            let Some(it) = &items[i] else { continue };
            if best.is_some_and(|(c, _)| rect_distance(p, it.bbox) >= c) {
                continue;
            }
            let c = it.entry_cost(p, spec);
            if best.is_none_or(|(bc, _)| c < bc) {
                best = Some((c, i));
            }
        }
        let pick = best
            .map(|(_, i)| i)
            .or_else(|| live.iter().copied().find(|i| !done[*i]));
        let Some(i) = pick else { break };
        done[i] = true;
        order.push(i);
        if let Some(it) = &items[i] {
            p = it.run(p, spec).1;
        }
    }
    order
}

/// 2-opt on the order: reverse stretches while that shortens the travel
/// between the items' ends (an item in a reversed stretch is taken
/// backwards) and keeps the precedence.
fn two_opt(order: &mut [usize], ends: &mut [(Point, Point)], preds: &[Vec<usize>], from: Point) {
    let n = order.len();
    if !(3..=MAX_TWO_OPT).contains(&n) {
        return;
    }
    let mut pos = vec![0usize; ends.len()];
    for (k, &i) in order.iter().enumerate() {
        pos[i] = k;
    }
    for _ in 0..TWO_OPT_PASSES {
        let mut improved = false;
        for i in 0..n {
            let before = if i == 0 { from } else { ends[order[i - 1]].1 };
            let ei = ends[order[i]].0;
            for j in i + 1..n {
                let xj = ends[order[j]].1;
                let (old, new) = match order.get(j + 1) {
                    Some(&next) => {
                        let en = ends[next].0;
                        (
                            (ei - before).hypot() + (en - xj).hypot(),
                            (xj - before).hypot() + (en - ei).hypot(),
                        )
                    }
                    None => ((ei - before).hypot(), (xj - before).hypot()),
                };
                if new + 1e-9 >= old {
                    continue;
                }
                // Precedence inside the stretch would turn round.
                let breaks = order[i..=j]
                    .iter()
                    .any(|b| preds[*b].iter().any(|a| (i..=j).contains(&pos[*a])));
                if breaks {
                    continue;
                }
                order[i..=j].reverse();
                for (k, &it) in order.iter().enumerate().take(j + 1).skip(i) {
                    pos[it] = k;
                    let (e, x) = ends[it];
                    ends[it] = (x, e);
                }
                improved = true;
                break;
            }
        }
        if !improved {
            break;
        }
    }
}

/// Plan the cut order of `items`.
pub fn plan(items: &[RouteItem], spec: &RouteSpec) -> Route {
    let spec = RouteSpec {
        from: if finite(spec.from) {
            spec.from
        } else {
            Point::ZERO
        },
        ..*spec
    };
    let prepared = prepare(items, &spec);
    let mut moves_before = Vec::new();
    let travel_before = {
        let mut p = spec.from;
        let mut t = 0.0;
        for it in prepared.iter().flatten() {
            for s in &it.subs {
                t += (s.start - p).hypot();
                if (s.start - p).hypot() > 1e-9 {
                    moves_before.push((p, s.start));
                }
                p = s.end;
            }
        }
        t
    };
    let preds = precedence(&prepared, &spec);
    let first = greedy(&prepared, &preds, &spec);
    let (t1, steps1, moves1, ends1) = evaluate(&prepared, &first, &spec);
    let mut ends = ends1.clone();
    let mut second = first.clone();
    two_opt(&mut second, &mut ends, &preds, spec.from);
    let (t2, steps2, moves2, ends2) = evaluate(&prepared, &second, &spec);
    let (mut order, travel_after, steps, moves, ends) = if t2 + 1e-9 < t1 {
        (second, t2, steps2, moves2, ends2)
    } else {
        (first, t1, steps1, moves1, ends1)
    };
    let end = order
        .last()
        .and_then(|i| ends.get(*i))
        .map_or(spec.from, |e| e.1);
    // Items with nothing to cut go last, in their order.
    for (i, it) in prepared.iter().enumerate() {
        if it.is_none() {
            order.push(i);
        }
    }
    let mut cuts = Vec::with_capacity(items.len());
    let mut changed = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        match prepared.get(i).and_then(|p| p.as_ref()) {
            Some(it) if it.editable => {
                let st = steps.get(i).cloned().unwrap_or_default();
                let moved = st
                    .iter()
                    .enumerate()
                    .any(|(n, (s, k, r))| *s != n || *k != 0 || *r);
                cuts.push(
                    st.iter()
                        .filter_map(|(s, k, r)| it.subs.get(*s).map(|sub| sub.rebuild(*k, *r)))
                        .collect(),
                );
                changed.push(moved);
            }
            _ => {
                cuts.push(
                    item.paths
                        .iter()
                        .flat_map(crate::subpaths)
                        .map(|s| s.path)
                        .collect(),
                );
                changed.push(false);
            }
        }
    }
    Route {
        end,
        order,
        cuts,
        changed,
        travel_before,
        travel_after,
        moves: moves
            .into_iter()
            .filter(|(a, b)| (*a - *b).hypot() > 1e-9)
            .collect(),
        moves_before,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cut::rect_path;

    fn sq(x: f64, y: f64, s: f64) -> BezPath {
        rect_path(Rect::new(x, y, x + s, y + s))
    }

    fn item(p: BezPath) -> RouteItem {
        RouteItem {
            paths: vec![p],
            editable: true,
        }
    }

    fn first_point(p: &BezPath) -> Point {
        match p.elements().first() {
            Some(PathEl::MoveTo(p)) => *p,
            _ => Point::new(f64::NAN, f64::NAN),
        }
    }

    #[test]
    fn nearest_neighbour_shortens_a_zigzag() {
        // Drawn far, near, far, near: cut near ones in a row.
        let items: Vec<RouteItem> = [0.0, 300.0, 10.0, 310.0, 20.0, 320.0]
            .iter()
            .map(|x| item(sq(*x, 0.0, 5.0)))
            .collect();
        let r = plan(&items, &RouteSpec::default());
        assert_eq!(r.order, vec![0, 2, 4, 1, 3, 5]);
        assert!(r.travel_after < r.travel_before / 3.0, "{r:?}");
        assert_eq!(r.cuts.len(), 6);
        // The moves as drawn add up to the travel before.
        assert_eq!(r.moves_before.len(), 5);
        let sum: f64 = r.moves_before.iter().map(|(a, b)| (*b - *a).hypot()).sum();
        assert!((sum - r.travel_before).abs() < 1e-9);
    }

    #[test]
    fn inner_outlines_go_first() {
        // An object inside a frame drawn before it; and a hole drawn after
        // its outer outline in one object.
        let mut letter = sq(100.0, 0.0, 50.0);
        letter.extend(sq(110.0, 10.0, 10.0));
        let items = vec![
            item(sq(0.0, 0.0, 50.0)),
            item(sq(20.0, 20.0, 5.0)),
            item(letter),
        ];
        let r = plan(&items, &RouteSpec::default());
        let p0 = r.order.iter().position(|i| *i == 0).expect("frame");
        let p1 = r.order.iter().position(|i| *i == 1).expect("inner");
        assert!(p1 < p0, "{:?}", r.order);
        // The letter's hole is cut before its outline.
        let cuts = &r.cuts[2];
        assert_eq!(cuts.len(), 2);
        assert!(cuts[0].bounding_box().width() < 11.0);
        assert!(r.changed[2]);
        // Without the rule the frame may come first.
        let r = plan(
            &items,
            &RouteSpec {
                inner_first: false,
                ..Default::default()
            },
        );
        assert_eq!(r.order[0], 0);
    }

    #[test]
    fn closed_outlines_start_at_the_nearest_node() {
        let items = vec![item(sq(0.0, 0.0, 10.0)), item(sq(20.0, 0.0, 10.0))];
        let r = plan(&items, &RouteSpec::default());
        // The second square starts at its bottom-left node (20, 0),
        // nearest to where the first ended (0, 0).
        assert_eq!(first_point(&r.cuts[1][0]), Point::new(20.0, 0.0));
        assert!((r.travel_after - 20.0).abs() < 1e-9);
        assert_eq!(r.end, Point::new(20.0, 0.0));
        let from_corner = RouteSpec {
            from: Point::new(30.0, 10.0),
            ..Default::default()
        };
        let r = plan(&items, &from_corner);
        assert_eq!(r.order, vec![1, 0]);
        assert_eq!(first_point(&r.cuts[1][0]), Point::new(30.0, 10.0));
        assert_eq!(first_point(&r.cuts[0][0]), Point::new(10.0, 10.0));
        assert!(r.changed[1]);
        let bl = RouteSpec {
            start: StartAt::BottomLeft,
            ..from_corner
        };
        let r = plan(&items, &bl);
        assert_eq!(first_point(&r.cuts[1][0]), Point::new(20.0, 0.0));
    }

    #[test]
    fn directions_follow_the_option() {
        let items = vec![item(sq(0.0, 0.0, 10.0))];
        let cw = plan(
            &items,
            &RouteSpec {
                direction: Direction::Clockwise,
                ..Default::default()
            },
        );
        let pts: Vec<Point> = cw.cuts[0][0]
            .elements()
            .iter()
            .filter_map(|e| e.end_point())
            .collect();
        assert!(signed_area(&pts[..4]) < 0.0, "{pts:?}");
        assert!(cw.changed[0]);
        let ccw = plan(
            &items,
            &RouteSpec {
                direction: Direction::CounterClockwise,
                ..Default::default()
            },
        );
        assert!(!ccw.changed[0], "already counter-clockwise from its start");
        // Open paths are cut from the nearer end unless kept.
        let mut line = BezPath::new();
        line.move_to((50.0, 0.0));
        line.line_to((1.0, 0.0));
        let items = vec![item(line)];
        let kept = plan(&items, &RouteSpec::default());
        assert!(!kept.changed[0]);
        assert!((kept.travel_after - 50.0).abs() < 1e-9);
        let free = plan(
            &items,
            &RouteSpec {
                direction: Direction::Clockwise,
                ..Default::default()
            },
        );
        assert!(free.changed[0]);
        assert!((free.travel_after - 1.0).abs() < 1e-9);
        assert_eq!(first_point(&free.cuts[0][0]), Point::new(1.0, 0.0));
    }

    #[test]
    fn fixed_items_keep_their_shape_and_two_opt_untangles() {
        let mut fixed = sq(0.0, 0.0, 10.0);
        fixed.extend(sq(100.0, 100.0, 10.0));
        let items = vec![RouteItem {
            paths: vec![fixed.clone()],
            editable: false,
        }];
        let r = plan(&items, &RouteSpec::default());
        assert!(!r.changed[0]);
        assert_eq!(r.cuts[0].len(), 2);
        assert_eq!(first_point(&r.cuts[0][1]), Point::new(100.0, 100.0));
        // A ring of points: the route never gets longer than as drawn.
        let items: Vec<RouteItem> = (0..24)
            .map(|k| {
                let a = f64::from(k * 7 % 24) / 24.0 * std::f64::consts::TAU;
                item(sq(200.0 + 100.0 * a.cos(), 200.0 + 100.0 * a.sin(), 2.0))
            })
            .collect();
        let r = plan(&items, &RouteSpec::default());
        assert!(r.travel_after <= r.travel_before);
        let mut sorted = r.order.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..24).collect::<Vec<_>>());
        assert_eq!(r.moves.len(), 24);
    }

    #[test]
    fn empty_and_non_finite_items_do_not_panic() {
        let r = plan(&[], &RouteSpec::default());
        assert!(r.order.is_empty() && r.travel_after == 0.0);
        let mut bad = BezPath::new();
        bad.move_to((f64::NAN, 0.0));
        bad.line_to((1.0, f64::INFINITY));
        let items = vec![item(bad), item(BezPath::new()), item(sq(5.0, 5.0, 1.0))];
        let r = plan(
            &items,
            &RouteSpec {
                from: Point::new(f64::NAN, 0.0),
                ..Default::default()
            },
        );
        assert_eq!(r.order, vec![2, 0, 1]);
        assert!(r.travel_after.is_finite());
        assert_eq!(r.cuts.len(), 3);
    }
}
