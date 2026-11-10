//! Cut-file clean-up: objects lying on top of each other and edges that
//! two outlines share, both of which a cutter or router would cut twice.

use crate::subpaths;
use tracedraw_core::geometry::{
    BezPath, Line, ParamCurve, ParamCurveNearest, PathSeg, Point, Rect, Shape as _, Vec2,
};

/// Flattening accuracy as a share of the tolerance.
const FLATTEN_SHARE: f64 = 0.25;

// ----- duplicate objects ----------------------------------------------------

/// The objects that repeat an earlier one: for each, its index and the
/// index of the earlier object it repeats. Two objects are the same when
/// every point of either outline lies within `tol` of the other's (and
/// their bounding boxes agree within `tol`). The order of `objects` is
/// the stacking order; the first of equal objects is the original.
pub fn find_duplicates(objects: &[Vec<BezPath>], tol: f64) -> Vec<(usize, usize)> {
    let tol = tol.max(1e-6);
    let prepared: Vec<Option<(Rect, Vec<Vec<Point>>)>> =
        objects.iter().map(|o| prepare(o, tol)).collect();
    let mut order: Vec<usize> = (0..objects.len())
        .filter(|i| prepared[*i].is_some())
        .collect();
    order.sort_by(|a, b| {
        let ax = prepared[*a].as_ref().map_or(0.0, |p| p.0.x0);
        let bx = prepared[*b].as_ref().map_or(0.0, |p| p.0.x0);
        ax.total_cmp(&bx)
    });
    let mut original: Vec<Option<usize>> = vec![None; objects.len()];
    for (k, &i) in order.iter().enumerate() {
        let Some((bi, pi)) = &prepared[i] else {
            continue;
        };
        for &j in order[k + 1..].iter() {
            let Some((bj, pj)) = &prepared[j] else {
                continue;
            };
            if bj.x0 - bi.x0 > tol {
                break;
            }
            if !boxes_match(*bi, *bj, tol) {
                continue;
            }
            let (first, later) = if i < j { (i, j) } else { (j, i) };
            if original[later].is_some() {
                continue;
            }
            if covered(pi, pj, tol) && covered(pj, pi, tol) {
                // Point to the first original, not to another duplicate.
                original[later] = Some(original[first].unwrap_or(first));
            }
        }
    }
    let mut out: Vec<(usize, usize)> = original
        .iter()
        .enumerate()
        .filter_map(|(i, o)| o.map(|o| (i, o)))
        .collect();
    out.sort();
    out
}

fn boxes_match(a: Rect, b: Rect, tol: f64) -> bool {
    (a.x0 - b.x0).abs() <= tol
        && (a.y0 - b.y0).abs() <= tol
        && (a.x1 - b.x1).abs() <= tol
        && (a.y1 - b.y1).abs() <= tol
}

/// The object's bounds and its outlines as polylines with points no more
/// than `tol` apart.
fn prepare(paths: &[BezPath], tol: f64) -> Option<(Rect, Vec<Vec<Point>>)> {
    let mut polys = Vec::new();
    let mut bbox: Option<Rect> = None;
    for p in paths {
        for sub in subpaths(p) {
            let mut pts = polyline(&sub.path, tol * FLATTEN_SHARE);
            if sub.closed {
                if let (Some(a), Some(b)) = (pts.first().copied(), pts.last().copied()) {
                    if (a - b).hypot() > 1e-9 {
                        pts.push(a);
                    }
                }
            }
            let pts = densify(&pts, tol);
            if pts.is_empty() {
                continue;
            }
            for q in &pts {
                let r = Rect::from_points(*q, *q);
                bbox = Some(bbox.map_or(r, |b| b.union(r)));
            }
            polys.push(pts);
        }
    }
    let bbox = bbox?;
    if !(bbox.x0.is_finite() && bbox.y0.is_finite() && bbox.x1.is_finite() && bbox.y1.is_finite()) {
        return None;
    }
    Some((bbox, polys))
}

fn polyline(path: &BezPath, accuracy: f64) -> Vec<Point> {
    let mut pts = Vec::new();
    tracedraw_core::geometry::flatten(path, accuracy.max(1e-6), &mut |el| {
        if let Some(p) = el.end_point() {
            pts.push(p);
        }
    });
    pts
}

/// Insert points so that neighbours are at most `step` apart.
fn densify(pts: &[Point], step: f64) -> Vec<Point> {
    let mut out = Vec::with_capacity(pts.len());
    for (k, p) in pts.iter().enumerate() {
        if k > 0 {
            let a = pts[k - 1];
            let d = (*p - a).hypot();
            let n = (d / step).ceil().min(100_000.0) as usize;
            for s in 1..n {
                out.push(a.lerp(*p, s as f64 / n as f64));
            }
        }
        out.push(*p);
    }
    out
}

/// Every point of `a` lies within `tol` of a polyline of `b`.
fn covered(a: &[Vec<Point>], b: &[Vec<Point>], tol: f64) -> bool {
    a.iter()
        .flatten()
        .all(|p| b.iter().any(|poly| near_polyline(*p, poly, tol)))
}

fn near_polyline(p: Point, poly: &[Point], tol: f64) -> bool {
    if poly.len() == 1 {
        return (poly[0] - p).hypot() <= tol;
    }
    poly.windows(2)
        .any(|w| dist_to_segment(p, w[0], w[1]) <= tol)
}

fn dist_to_segment(p: Point, a: Point, b: Point) -> f64 {
    let ab = b - a;
    let len2 = ab.hypot2();
    if len2 < 1e-24 {
        return (p - a).hypot();
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    (p - (a + ab * t)).hypot()
}

// ----- duplicate edges ------------------------------------------------------

/// What [`remove_duplicate_edges`] did to one object.
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeResult {
    /// Nothing of it lay on an earlier edge.
    Unchanged,
    /// Its outline without the shared stretches (open where they were cut).
    Changed { path: BezPath, removed_length: f64 },
    /// All of it lay on earlier edges.
    Removed { removed_length: f64 },
}

/// A segment already kept, against which later ones are checked.
struct Kept {
    seg: PathSeg,
    bbox: Rect,
}

/// Remove the stretches of outline that run along an edge met earlier,
/// so that every edge is cut once: two letters sharing a side keep the
/// side in the first only. Straight segments lose just the shared part;
/// a curved segment goes when it repeats an earlier curve or line from
/// end to end within `tol`. Objects are taken in the order given, and a
/// stretch of an object can also repeat an earlier stretch of the same
/// object. Pieces shorter than `tol` are dropped.
pub fn remove_duplicate_edges(objects: &[Vec<BezPath>], tol: f64) -> Vec<EdgeResult> {
    let tol = tol.max(1e-6);
    let mut kept: Vec<Kept> = Vec::new();
    let mut grid = Grid::new(cell_size(objects, tol));
    let mut results = Vec::with_capacity(objects.len());
    for paths in objects {
        let mut pieces_total: Vec<Vec<PathSeg>> = Vec::new();
        let mut removed = 0.0;
        let mut any_removed = false;
        let mut any_left = false;
        for path in paths {
            for sub in subpaths(path) {
                let segs: Vec<PathSeg> =
                    sub.path.segments().filter(|s| seg_len(s) > 1e-9).collect();
                // Pieces of this subpath, in order; a piece is a run of
                // segments that stays connected.
                let mut chains: Vec<Vec<PathSeg>> = Vec::new();
                let mut broken = false;
                let mut current: Vec<PathSeg> = Vec::new();
                for seg in &segs {
                    let left = remaining(seg, &kept, &grid, tol);
                    let full = left.len() == 1 && same_seg(&left[0], seg);
                    if !full {
                        any_removed = true;
                        broken = true;
                        removed += seg_len(seg) - left.iter().map(seg_len).sum::<f64>();
                    }
                    for piece in left {
                        let joins = current
                            .last()
                            .is_some_and(|l| (l.end() - piece.start()).hypot() < 1e-9);
                        if !joins && !current.is_empty() {
                            chains.push(std::mem::take(&mut current));
                        }
                        current.push(piece);
                    }
                    if !full {
                        // A gap ends the run even when a later piece of
                        // the same segment follows.
                        if !current.is_empty()
                            && !current
                                .last()
                                .is_some_and(|l| (l.end() - seg.end()).hypot() < 1e-9)
                        {
                            chains.push(std::mem::take(&mut current));
                        }
                    }
                    // Kept once checked, so a segment never meets itself
                    // but later ones (of this object too) meet it.
                    let bbox = seg.bounding_box().inflate(tol, tol);
                    grid.insert(kept.len(), bbox);
                    kept.push(Kept { seg: *seg, bbox });
                }
                if !current.is_empty() {
                    chains.push(current);
                }
                // A closed outline cut open: its last run continues into
                // its first.
                if sub.closed && broken && chains.len() > 1 {
                    let first_start = chains[0][0].start();
                    let last_end = chains.last().and_then(|c| c.last()).map(|s| s.end());
                    if last_end.is_some_and(|e| (e - first_start).hypot() < 1e-9) {
                        let first = chains.remove(0);
                        if let Some(last) = chains.last_mut() {
                            last.extend(first);
                        }
                    }
                }
                if !chains.is_empty() {
                    any_left = true;
                }
                if sub.closed && !broken {
                    // Whole and closed: keep it closed.
                    pieces_total.push(Vec::new());
                    if let Some(c) = chains.pop() {
                        pieces_total.push(c);
                        pieces_total.push(Vec::new());
                    }
                } else {
                    pieces_total.extend(chains);
                }
            }
        }
        if !any_removed {
            results.push(EdgeResult::Unchanged);
        } else if !any_left {
            results.push(EdgeResult::Removed {
                removed_length: removed,
            });
        } else {
            results.push(EdgeResult::Changed {
                path: build_path(&pieces_total),
                removed_length: removed,
            });
        }
    }
    results
}

/// Build a path from runs; a run between two empty markers is closed.
fn build_path(runs: &[Vec<PathSeg>]) -> BezPath {
    let mut p = BezPath::new();
    let mut close_next = false;
    let mut k = 0;
    while k < runs.len() {
        let run = &runs[k];
        if run.is_empty() {
            close_next = !close_next;
            k += 1;
            continue;
        }
        p.move_to(run[0].start());
        for s in run {
            match *s {
                PathSeg::Line(l) => p.line_to(l.p1),
                PathSeg::Quad(q) => p.quad_to(q.p1, q.p2),
                PathSeg::Cubic(c) => p.curve_to(c.p1, c.p2, c.p3),
            }
        }
        if close_next {
            p.close_path();
        }
        k += 1;
    }
    p
}

fn seg_len(s: &PathSeg) -> f64 {
    use tracedraw_core::geometry::ParamCurveArclen;
    s.arclen(1e-4)
}

fn same_seg(a: &PathSeg, b: &PathSeg) -> bool {
    (a.start() - b.start()).hypot() < 1e-12 && (a.end() - b.end()).hypot() < 1e-12
}

/// What is left of `seg` after taking away its stretches along kept
/// segments.
fn remaining(seg: &PathSeg, kept: &[Kept], grid: &Grid, tol: f64) -> Vec<PathSeg> {
    let bbox = seg.bounding_box().inflate(tol, tol);
    let near = grid.query(bbox);
    match seg {
        PathSeg::Line(line) => {
            let len = (line.p1 - line.p0).hypot();
            let mut left: Vec<(f64, f64)> = vec![(0.0, 1.0)];
            for &i in &near {
                let Some(k) = kept.get(i) else { continue };
                if !k.bbox.overlaps(bbox) {
                    continue;
                }
                if let Some((a, b)) = shared_stretch(line, &k.seg, tol) {
                    left = subtract(&left, a, b);
                    if left.is_empty() {
                        break;
                    }
                }
            }
            left.into_iter()
                .filter(|(a, b)| (b - a) * len > tol)
                .map(|(a, b)| {
                    if a <= 0.0 && b >= 1.0 {
                        *seg
                    } else {
                        PathSeg::Line(Line::new(line.eval(a), line.eval(b)))
                    }
                })
                .collect()
        }
        _ => {
            let repeated = near.iter().any(|&i| {
                kept.get(i)
                    .is_some_and(|k| k.bbox.overlaps(bbox) && curve_repeats(seg, &k.seg, tol))
            });
            if repeated {
                Vec::new()
            } else {
                vec![*seg]
            }
        }
    }
}

/// The stretch of `line` (as parameters a < b) that runs along `other`
/// within `tol`, when `other` is straight and lies on the same line.
fn shared_stretch(line: &Line, other: &PathSeg, tol: f64) -> Option<(f64, f64)> {
    let o = match other {
        PathSeg::Line(l) => *l,
        // A curve that is straight in fact (its handles on the chord).
        s => {
            let l = Line::new(s.start(), s.end());
            let flat = [0.25, 0.5, 0.75]
                .iter()
                .all(|t| dist_to_segment(s.eval(*t), l.p0, l.p1) <= tol * 0.5);
            if !flat {
                return None;
            }
            l
        }
    };
    let d: Vec2 = line.p1 - line.p0;
    let len2 = d.hypot2();
    if len2 < 1e-24 {
        return None;
    }
    let len = len2.sqrt();
    // Both ends of the other segment lie on this segment's line.
    let off = |p: Point| ((p - line.p0).cross(d) / len).abs();
    if off(o.p0) > tol || off(o.p1) > tol {
        return None;
    }
    let t0 = (o.p0 - line.p0).dot(d) / len2;
    let t1 = (o.p1 - line.p0).dot(d) / len2;
    let (lo, hi) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
    let a = lo.max(0.0);
    let b = hi.min(1.0);
    if (b - a) * len <= tol {
        return None;
    }
    Some((a, b))
}

/// Take `[a, b]` out of the sorted intervals.
fn subtract(left: &[(f64, f64)], a: f64, b: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(left.len() + 1);
    for &(x, y) in left {
        if b <= x || a >= y {
            out.push((x, y));
            continue;
        }
        if a > x {
            out.push((x, a));
        }
        if b < y {
            out.push((b, y));
        }
    }
    out
}

/// `seg` (a curve) follows `other` from end to end, in either direction.
fn curve_repeats(seg: &PathSeg, other: &PathSeg, tol: f64) -> bool {
    let same =
        (seg.start() - other.start()).hypot() <= tol && (seg.end() - other.end()).hypot() <= tol;
    let reversed =
        (seg.start() - other.end()).hypot() <= tol && (seg.end() - other.start()).hypot() <= tol;
    if !same && !reversed {
        return false;
    }
    (1..8).all(|k| {
        let p = seg.eval(k as f64 / 8.0);
        let n = other.nearest(p, 1e-6);
        n.distance_sq.sqrt() <= tol
    })
}

fn cell_size(objects: &[Vec<BezPath>], tol: f64) -> f64 {
    let mut bbox: Option<Rect> = None;
    for p in objects.iter().flatten() {
        let b = p.bounding_box();
        if b.x0.is_finite() && b.y0.is_finite() && b.x1.is_finite() && b.y1.is_finite() {
            bbox = Some(bbox.map_or(b, |a| a.union(b)));
        }
    }
    let side = bbox.map_or(100.0, |b| b.width().max(b.height()));
    (side / 64.0).max(tol * 4.0).max(0.5)
}

/// A uniform grid of segment boxes, to find nearby segments quickly.
struct Grid {
    cell: f64,
    cells: std::collections::HashMap<(i64, i64), Vec<usize>>,
    /// Segments spanning too many cells, checked always.
    big: Vec<usize>,
}

impl Grid {
    const MAX_CELLS: i64 = 4096;

    fn new(cell: f64) -> Grid {
        Grid {
            cell,
            cells: std::collections::HashMap::new(),
            big: Vec::new(),
        }
    }

    fn range(&self, r: Rect) -> Option<(i64, i64, i64, i64)> {
        let f = |v: f64| (v / self.cell).floor();
        let (x0, y0, x1, y1) = (f(r.x0), f(r.y0), f(r.x1), f(r.y1));
        if ![x0, y0, x1, y1]
            .iter()
            .all(|v| v.is_finite() && v.abs() < 1e12)
        {
            return None;
        }
        Some((x0 as i64, y0 as i64, x1 as i64, y1 as i64))
    }

    fn insert(&mut self, id: usize, r: Rect) {
        match self.range(r) {
            Some((x0, y0, x1, y1)) if (x1 - x0 + 1) * (y1 - y0 + 1) <= Self::MAX_CELLS => {
                for x in x0..=x1 {
                    for y in y0..=y1 {
                        self.cells.entry((x, y)).or_default().push(id);
                    }
                }
            }
            _ => self.big.push(id),
        }
    }

    fn query(&self, r: Rect) -> Vec<usize> {
        let mut out = self.big.clone();
        match self.range(r) {
            Some((x0, y0, x1, y1)) if (x1 - x0 + 1) * (y1 - y0 + 1) <= Self::MAX_CELLS => {
                for x in x0..=x1 {
                    for y in y0..=y1 {
                        if let Some(v) = self.cells.get(&(x, y)) {
                            out.extend_from_slice(v);
                        }
                    }
                }
            }
            _ => out.extend(self.cells.values().flatten().copied()),
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::length;
    use tracedraw_core::geometry::{Circle, Rect};

    fn square(x: f64, y: f64, s: f64) -> BezPath {
        Rect::new(x, y, x + s, y + s).to_path(0.01)
    }

    #[test]
    fn a_repeated_object_is_found_whatever_its_start_node() {
        let a = square(0.0, 0.0, 10.0);
        // The same square drawn from another corner and the other way round.
        let mut b = BezPath::new();
        b.move_to((10.0, 10.0));
        b.line_to((10.0, 0.0));
        b.line_to((0.0, 0.0));
        b.line_to((0.0, 10.0));
        b.close_path();
        let c = square(0.05, 0.0, 10.0);
        let d = square(30.0, 0.0, 10.0);
        let found = find_duplicates(&[vec![a], vec![b], vec![c], vec![d]], 0.1);
        assert_eq!(found, vec![(1, 0), (2, 0)]);
    }

    #[test]
    fn a_smaller_object_inside_is_not_a_duplicate() {
        let found = find_duplicates(
            &[vec![square(0.0, 0.0, 10.0)], vec![square(0.0, 0.0, 9.0)]],
            0.1,
        );
        assert!(found.is_empty());
    }

    #[test]
    fn circles_match_within_the_tolerance() {
        let a = Circle::new((0.0, 0.0), 5.0).to_path(0.01);
        let b = Circle::new((0.02, 0.0), 5.0).to_path(0.01);
        let c = Circle::new((0.0, 0.0), 5.5).to_path(0.01);
        let found = find_duplicates(&[vec![a], vec![b], vec![c]], 0.05);
        assert_eq!(found, vec![(1, 0)]);
    }

    #[test]
    fn two_squares_sharing_a_side_cut_it_once() {
        let a = square(0.0, 0.0, 10.0);
        let b = square(10.0, 0.0, 10.0);
        let r = remove_duplicate_edges(&[vec![a], vec![b]], 0.01);
        assert_eq!(r[0], EdgeResult::Unchanged);
        let EdgeResult::Changed {
            path,
            removed_length,
        } = &r[1]
        else {
            panic!("second square should change: {:?}", r[1]);
        };
        assert!((removed_length - 10.0).abs() < 1e-9);
        assert!((length(path) - 30.0).abs() < 1e-9);
        // One open run of three sides.
        let subs = subpaths(path);
        assert_eq!(subs.len(), 1);
        assert!(!subs[0].closed);
    }

    #[test]
    fn a_partly_shared_side_keeps_its_free_part() {
        let a = square(0.0, 0.0, 10.0);
        let b = Rect::new(10.0, 5.0, 20.0, 25.0).to_path(0.01);
        let r = remove_duplicate_edges(&[vec![a], vec![b]], 0.01);
        let EdgeResult::Changed {
            path,
            removed_length,
        } = &r[1]
        else {
            panic!("expected a change");
        };
        assert!((removed_length - 5.0).abs() < 1e-9);
        assert!((length(path) - 55.0).abs() < 1e-9);
    }

    #[test]
    fn an_exact_copy_is_removed_entirely() {
        let a = Circle::new((0.0, 0.0), 5.0).to_path(0.01);
        let r = remove_duplicate_edges(&[vec![a.clone()], vec![a]], 0.01);
        assert_eq!(r[0], EdgeResult::Unchanged);
        assert!(matches!(r[1], EdgeResult::Removed { .. }));
    }

    #[test]
    fn separate_objects_are_left_alone() {
        let r = remove_duplicate_edges(
            &[vec![square(0.0, 0.0, 10.0)], vec![square(10.5, 0.0, 10.0)]],
            0.1,
        );
        assert_eq!(r, vec![EdgeResult::Unchanged, EdgeResult::Unchanged]);
    }

    #[test]
    fn a_line_drawn_back_over_itself_is_cut_once() {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((10.0, 0.0));
        p.move_to((10.0, 0.0));
        p.line_to((0.0, 0.0));
        let r = remove_duplicate_edges(&[vec![p]], 0.01);
        let EdgeResult::Changed { path, .. } = &r[0] else {
            panic!("expected a change");
        };
        assert!((length(path) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn bad_numbers_do_not_panic() {
        let mut p = BezPath::new();
        p.move_to((f64::NAN, 0.0));
        p.line_to((1.0, f64::INFINITY));
        let _ = remove_duplicate_edges(&[vec![p.clone()], vec![p.clone()]], 0.01);
        let _ = find_duplicates(&[vec![p.clone()], vec![p]], 0.01);
    }
}
