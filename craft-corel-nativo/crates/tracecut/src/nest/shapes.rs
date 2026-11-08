//! Shape nesting: objects placed by their true outlines, holes allowed.
//!
//! Each object becomes a region (its closed outlines flattened, open
//! curves by their boxes, holes filled unless parts may go into them),
//! grown by half the spacing with round corners and simplified within a
//! quarter of the tolerance; the growth carries 0.3 x the tolerance more
//! so the simplification never brings two objects closer than the
//! spacing. Each allowed turn of each region is drawn on a grid of cells
//! (a cell is set when the region touches it).
//!
//! Bottom-left fill: objects go largest first. For each sheet in turn and
//! each turn of the object, the grid gives the lowest, then leftmost,
//! cell position where no set cells meet; from there the object slides
//! down and left in true geometry (exact polygon overlap tests, a
//! binary search over up to three cells) until it touches. The turn whose
//! top ends lowest wins; an object that fits on no sheet opens a new one.
//!
//! The improvement loop then swaps two objects of the order at random
//! (seeded, so the same seed gives the same layout), nests again and keeps
//! the new order when the layout is no worse: fewer objects left out,
//! fewer sheets, less length on the last sheet, then a lower sum of tops.
//! It runs a set number of rounds and stops early when the caller says
//! time is up.

use super::poly::{bounds, overlap, Grid, Region};
use super::{Frame, Layout, Placement, Rng, SheetSpec};
use crate::subpaths;
use tracedraw_core::geometry::{flatten, Affine, BezPath, PathEl, Point, Rect, Shape as _, Vec2};
use tracedraw_core::shaping;

/// The turns an object may take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Rotations {
    /// As it is.
    None,
    /// 0 and 180 degrees.
    Half,
    /// Every 90 degrees.
    #[default]
    Quarter,
    /// Every 45 degrees.
    Eighth,
}

impl Rotations {
    pub const ALL: [Rotations; 4] = [
        Rotations::None,
        Rotations::Half,
        Rotations::Quarter,
        Rotations::Eighth,
    ];

    /// The angles, degrees counter-clockwise.
    pub fn angles(self) -> Vec<f64> {
        match self {
            Rotations::None => vec![0.0],
            Rotations::Half => vec![0.0, 180.0],
            Rotations::Quarter => vec![0.0, 90.0, 180.0, 270.0],
            Rotations::Eighth => (0..8).map(|i| i as f64 * 45.0).collect(),
        }
    }
}

/// What shape nesting does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeSettings {
    pub sheet: SheetSpec,
    pub rotations: Rotations,
    /// Smaller objects may go into the holes of larger ones.
    pub holes: bool,
    /// Accuracy of the outlines (mm), 0.01 to 50.
    pub tolerance: f64,
    /// Rounds of the improvement loop (0: none).
    pub rounds: usize,
    pub seed: u64,
}

impl Default for ShapeSettings {
    fn default() -> Self {
        ShapeSettings {
            sheet: SheetSpec::default(),
            rotations: Rotations::Quarter,
            holes: true,
            tolerance: 0.5,
            rounds: 100,
            seed: 1,
        }
    }
}

/// One turn of an object.
#[derive(Debug, Clone)]
struct Orient {
    /// The grown region with its box's corner at (0, 0).
    region: Region,
    w: f64,
    h: f64,
    runs: Vec<Vec<(usize, usize)>>,
    /// Page to this local frame.
    local: Affine,
    angle: f64,
}

#[derive(Debug, Clone)]
struct Part {
    area: f64,
    key: f64,
    orients: Vec<Orient>,
}

/// Where one object went: sheet, turn and position in the usable area.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Spot {
    sheet: usize,
    orient: usize,
    pos: Vec2,
}

/// How good a layout is: lower is better, compared in order.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Score {
    left_out: usize,
    sheets: usize,
    last_length: f64,
    tops: f64,
}

impl Score {
    fn better_or_equal(&self, o: &Score) -> bool {
        use std::cmp::Ordering::*;
        let ord = self
            .left_out
            .cmp(&o.left_out)
            .then(self.sheets.cmp(&o.sheets))
            .then(self.last_length.total_cmp(&o.last_length))
            .then(self.tops.total_cmp(&o.tops));
        matches!(ord, Less | Equal)
    }
}

struct Attempt {
    spots: Vec<Option<Spot>>,
    score: Score,
}

const EPS: f64 = 1e-7;

/// The usable area of every sheet and the grid cell.
#[derive(Debug, Clone, Copy)]
struct Area {
    cw: f64,
    /// Infinite for a roll.
    ch: f64,
    cell: f64,
}

impl Area {
    fn new_grid(&self) -> Grid {
        let cols = (self.cw / self.cell).ceil().clamp(1.0, 1e6) as usize + 1;
        let rows = if self.ch.is_finite() {
            (self.ch / self.cell).ceil().clamp(1.0, 1e6) as usize + 2
        } else {
            usize::MAX
        };
        Grid::new(self.cell, cols, rows)
    }
}

/// The closed rings of one path in the normalised frame: closed
/// subpaths flattened, open ones by their boxes (grown a little so they
/// have an area).
fn rings_of(path: &BezPath, norm: Affine, tol: f64) -> Vec<Vec<Point>> {
    let mut rings = Vec::new();
    for sp in subpaths(path) {
        let p = norm * sp.path.clone();
        if sp.closed {
            let mut ring: Vec<Point> = Vec::new();
            flatten(&p, tol, &mut |el| match el {
                PathEl::MoveTo(q) | PathEl::LineTo(q) => ring.push(q),
                _ => {}
            });
            if ring.len() > 1 && ring.first() == ring.last() {
                ring.pop();
            }
            rings.push(ring);
        } else {
            let b = p.bounding_box().inflate(0.01, 0.01);
            rings.push(vec![
                Point::new(b.x0, b.y0),
                Point::new(b.x1, b.y0),
                Point::new(b.x1, b.y1),
                Point::new(b.x0, b.y1),
            ]);
        }
    }
    rings
}

fn rings_to_path(rings: &[Vec<Point>]) -> BezPath {
    let mut p = BezPath::new();
    for r in rings {
        let mut it = r.iter();
        if let Some(first) = it.next() {
            p.move_to(*first);
            for q in it {
                p.line_to(*q);
            }
            p.close_path();
        }
    }
    p
}

fn path_to_rings(p: &BezPath) -> Vec<Vec<Point>> {
    let mut out = Vec::new();
    let mut cur: Vec<Point> = Vec::new();
    for el in p.elements() {
        match *el {
            PathEl::MoveTo(q) => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(q);
            }
            PathEl::LineTo(q) => cur.push(q),
            PathEl::QuadTo(_, q) | PathEl::CurveTo(_, _, q) => cur.push(q),
            PathEl::ClosePath => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Thin a closed ring within `tol`.
fn simplify_ring(r: &[Point], tol: f64) -> Vec<Point> {
    let Some(first) = r.first() else {
        return Vec::new();
    };
    let mut closed = r.to_vec();
    closed.push(*first);
    let mut s = tracedraw_core::geometry::simplify(&closed, tol);
    s.pop();
    s
}

/// The grown region of an object, before turning: each path a piece of
/// its own (even-odd inside it), the pieces welded.
fn grown_region(
    paths: &[BezPath],
    norm: Affine,
    s: &ShapeSettings,
    d: f64,
    tol: f64,
) -> Option<Region> {
    let mut all: Vec<Point> = Vec::new();
    let mut union: Option<BezPath> = None;
    for path in paths {
        let rings = rings_of(path, norm, tol / 20.0);
        all.extend(
            rings
                .iter()
                .flatten()
                .copied()
                .filter(|p| p.x.is_finite() && p.y.is_finite()),
        );
        let mut region = Region::new(rings);
        if !s.holes {
            region = region.outer_only();
        }
        if region.is_empty() {
            continue;
        }
        let piece = rings_to_path(&region.rings);
        union = Some(match union {
            None => piece,
            Some(u) => shaping::overlay(&u, &piece, shaping::Op::Weld),
        });
    }
    if all.is_empty() {
        return None;
    }
    let union = union.unwrap_or_default();
    let grown = shaping::offset(&union, d);
    let simple: Vec<Vec<Point>> = path_to_rings(&grown)
        .iter()
        .map(|r| simplify_ring(r, tol / 4.0))
        .collect();
    let region = Region::new(simple);
    if !region.is_empty() && region.area() > 1e-9 {
        return Some(region);
    }
    // Flat or empty outlines: their box, grown.
    let b = bounds(all.iter()).inflate(d.max(EPS), d.max(EPS));
    Some(Region::new(vec![vec![
        Point::new(b.x0, b.y0),
        Point::new(b.x1, b.y0),
        Point::new(b.x1, b.y1),
        Point::new(b.x0, b.y1),
    ]]))
}

fn prepare(
    paths: &[BezPath],
    norm: Affine,
    s: &ShapeSettings,
    d: f64,
    tol: f64,
    cell: f64,
) -> Option<Part> {
    let base = grown_region(paths, norm, s, d, tol)?;
    let key = base.area();
    let area = paths
        .iter()
        .map(|p| crate::measure::measure_object(std::slice::from_ref(p)).area)
        .filter(|a| a.is_finite())
        .sum::<f64>();
    let mut orients = Vec::new();
    for angle in s.rotations.angles() {
        let rot = Affine::rotate(angle.to_radians());
        let rings: Vec<Vec<Point>> = base
            .rings
            .iter()
            .map(|r| r.iter().map(|p| rot * *p).collect())
            .collect();
        let b = bounds(rings.iter().flatten());
        let shift = -b.origin().to_vec2();
        let region = Region::new(rings).moved(shift);
        if region.is_empty() {
            continue;
        }
        let (w, h) = (b.width(), b.height());
        let mut g = Grid::new(
            cell,
            (w / cell).ceil().clamp(1.0, 1e6) as usize + 2,
            usize::MAX,
        );
        g.paint(&region);
        orients.push(Orient {
            region,
            w,
            h,
            runs: g.runs(),
            local: Affine::translate(shift) * rot * norm,
            angle,
        });
    }
    (!orients.is_empty()).then_some(Part { area, key, orients })
}

#[derive(Debug, Clone)]
struct SheetState {
    grid: Grid,
    placed: Vec<Region>,
}

fn fits(o: &Orient, pos: Vec2, sheet: &SheetState, area: &Area) -> bool {
    if pos.x < -EPS || pos.y < -EPS || pos.x + o.w > area.cw + EPS || pos.y + o.h > area.ch + EPS {
        return false;
    }
    let bb = o.region.bbox + pos;
    let near: Vec<&Region> = sheet
        .placed
        .iter()
        .filter(|r| {
            r.bbox.x0 <= bb.x1 && bb.x0 <= r.bbox.x1 && r.bbox.y0 <= bb.y1 && bb.y0 <= r.bbox.y1
        })
        .collect();
    if near.is_empty() {
        return true;
    }
    let moved = o.region.moved(pos);
    !near.iter().any(|r| overlap(r, &moved))
}

/// The farthest `d` in `0..=max` that `ok` accepts (0 when none is).
fn slide(max: f64, ok: &mut dyn FnMut(f64) -> bool) -> f64 {
    if max <= EPS {
        return 0.0;
    }
    if ok(max) {
        return max;
    }
    let (mut lo, mut hi) = (0.0, max);
    for _ in 0..12 {
        let mid = (lo + hi) / 2.0;
        if ok(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// The lowest, then leftmost, place for `o` on `sheet`, slid down and
/// left until it touches.
fn find_spot(o: &Orient, sheet: &SheetState, area: &Area) -> Option<Vec2> {
    if o.w > area.cw + EPS || o.h > area.ch + EPS {
        return None;
    }
    let c = area.cell;
    let max_kx = ((area.cw - o.w) / c + 1e-9).floor().max(0.0) as usize;
    let max_ky = if area.ch.is_finite() {
        ((area.ch - o.h) / c + 1e-9).floor().max(0.0) as usize
    } else {
        sheet.grid.used_rows()
    };
    let mut found = None;
    'rows: for ky in 0..=max_ky {
        let mut kx = 0usize;
        while kx <= max_kx {
            match sheet.grid.conflict(&o.runs, kx, ky) {
                Some(next) => kx = next.max(kx + 1),
                None => {
                    let pos = Vec2::new(kx as f64 * c, ky as f64 * c);
                    if fits(o, pos, sheet, area) {
                        found = Some(pos);
                        break 'rows;
                    }
                    kx += 1;
                }
            }
        }
    }
    let mut pos = found?;
    // Slide down and left in true geometry.
    let reach = 3.0 * c;
    for _ in 0..8 {
        let mut moved = false;
        let d = slide(reach.min(pos.y), &mut |d| {
            fits(o, pos - Vec2::new(0.0, d), sheet, area)
        });
        if d > 1e-6 {
            pos.y -= d;
            moved = true;
        }
        let d = slide(reach.min(pos.x), &mut |d| {
            fits(o, pos - Vec2::new(d, 0.0), sheet, area)
        });
        if d > 1e-6 {
            pos.x -= d;
            moved = true;
        }
        if !moved {
            break;
        }
    }
    Some(pos)
}

/// The best place for `part` on `sheet`: its turn and position.
fn best_on(part: &Part, sheet: &SheetState, area: &Area) -> Option<(usize, Vec2)> {
    let mut best: Option<(usize, Vec2, (f64, f64))> = None;
    for (oi, o) in part.orients.iter().enumerate() {
        if let Some(pos) = find_spot(o, sheet, area) {
            let score = (pos.y + o.h, pos.x);
            let better = best.as_ref().is_none_or(|(_, _, s)| {
                score.0 < s.0 - EPS || (score.0 <= s.0 + EPS && score.1 < s.1 - EPS)
            });
            if better {
                best = Some((oi, pos, score));
            }
        }
    }
    best.map(|(oi, pos, _)| (oi, pos))
}

/// Nest the parts in `order`. `None` when `out_of_time` says stop.
fn run(
    parts: &[Option<Part>],
    order: &[usize],
    area: &Area,
    roll: bool,
    out_of_time: &mut Option<&mut dyn FnMut() -> bool>,
) -> Option<Attempt> {
    let mut sheets: Vec<SheetState> = Vec::new();
    let mut spots: Vec<Option<Spot>> = vec![None; parts.len()];
    for &i in order {
        if let Some(stop) = out_of_time.as_mut() {
            if stop() {
                return None;
            }
        }
        let Some(Some(part)) = parts.get(i) else {
            continue;
        };
        let mut spot = None;
        for (si, sheet) in sheets.iter().enumerate() {
            if let Some((oi, pos)) = best_on(part, sheet, area) {
                spot = Some((si, oi, pos));
                break;
            }
        }
        if spot.is_none() && (!roll || sheets.is_empty()) {
            let fresh = SheetState {
                grid: area.new_grid(),
                placed: Vec::new(),
            };
            if let Some((oi, pos)) = best_on(part, &fresh, area) {
                sheets.push(fresh);
                spot = Some((sheets.len() - 1, oi, pos));
            }
        }
        let Some((si, oi, pos)) = spot else {
            continue;
        };
        let (Some(sheet), Some(o)) = (sheets.get_mut(si), part.orients.get(oi)) else {
            continue;
        };
        let placed = o.region.moved(pos);
        sheet.grid.paint(&placed);
        sheet.placed.push(placed);
        if let Some(s) = spots.get_mut(i) {
            *s = Some(Spot {
                sheet: si,
                orient: oi,
                pos,
            });
        }
    }
    let n_sheets = sheets.len();
    let mut last_length: f64 = 0.0;
    let mut tops = 0.0;
    let mut left_out = 0;
    for (i, s) in spots.iter().enumerate() {
        let Some(Some(part)) = parts.get(i) else {
            continue;
        };
        match s {
            None => left_out += 1,
            Some(s) => {
                let h = part.orients.get(s.orient).map_or(0.0, |o| o.h);
                tops += s.pos.y + h;
                if s.sheet + 1 == n_sheets {
                    last_length = last_length.max(s.pos.y + h);
                }
            }
        }
    }
    Some(Attempt {
        spots,
        score: Score {
            left_out,
            sheets: n_sheets,
            last_length,
            tops,
        },
    })
}

/// Place each object, given by its page-space outlines, by its true
/// shape. `out_of_time` is asked between objects during the improvement
/// loop; the first layout is always finished.
pub fn nest_shapes(
    parts: &[Vec<BezPath>],
    s: &ShapeSettings,
    out_of_time: &mut dyn FnMut() -> bool,
) -> Layout {
    let mut layout = Layout {
        placements: vec![None; parts.len()],
        ..Default::default()
    };
    let Some(spec) = s.sheet.sane() else {
        return layout;
    };
    let tol = if s.tolerance.is_finite() {
        s.tolerance.clamp(0.01, 50.0)
    } else {
        0.5
    };
    let d = spec.spacing / 2.0 + 0.3 * tol;
    // The grown regions hold each outline grown by at least half the
    // spacing (the rest of `d` covers flattening and simplifying).
    let inset = spec.margin - spec.spacing / 2.0;
    let (across, along) = spec.across_along();
    let cw = across - 2.0 * inset;
    let ch = along - 2.0 * inset;
    if !(cw > EPS && ch > EPS) {
        return layout;
    }
    let area = Area {
        cw,
        ch,
        cell: tol.max(cw / 256.0),
    };
    let frame = Frame::new(&spec);
    let norm = frame.to_norm();
    let prepared: Vec<Option<Part>> = parts
        .iter()
        .map(|p| prepare(p, norm, s, d, tol, area.cell))
        .collect();
    let mut order: Vec<usize> = (0..parts.len())
        .filter(|i| prepared[*i].is_some())
        .collect();
    let key = |i: usize| {
        prepared
            .get(i)
            .and_then(|p| p.as_ref())
            .map_or(0.0, |p| p.key)
    };
    order.sort_by(|a, b| key(*b).total_cmp(&key(*a)).then(a.cmp(b)));

    let Some(mut best) = run(&prepared, &order, &area, spec.roll, &mut None) else {
        return layout;
    };
    let n = order.len();
    let mut rng = Rng::new(s.seed);
    if n >= 2 {
        for _ in 0..s.rounds {
            if out_of_time() {
                layout.stopped_by_time = true;
                break;
            }
            let mut cand = order.clone();
            let i = rng.below(n);
            let mut j = rng.below(n - 1);
            if j >= i {
                j += 1;
            }
            if rng.below(3) == 0 {
                // Move one object to another place in the order.
                let x = cand.remove(i);
                cand.insert(j.min(cand.len()), x);
            } else {
                cand.swap(i, j);
            }
            let attempt = run(
                &prepared,
                &cand,
                &area,
                spec.roll,
                &mut Some(&mut *out_of_time),
            );
            layout.rounds += 1;
            let Some(attempt) = attempt else {
                layout.stopped_by_time = true;
                break;
            };
            if attempt.score.better_or_equal(&best.score) {
                best = attempt;
                order = cand;
            }
        }
    }

    let mut extents = Vec::new();
    for (i, spot) in best.spots.iter().enumerate() {
        let (Some(spot), Some(Some(part))) = (spot, prepared.get(i)) else {
            continue;
        };
        let Some(o) = part.orients.get(spot.orient) else {
            continue;
        };
        let transform = frame.place(spot.sheet, inset, spot.pos, o.local);
        if let Some(p) = layout.placements.get_mut(i) {
            *p = Some(Placement {
                sheet: spot.sheet,
                transform,
                angle: o.angle,
            });
        }
        let to_sheet = frame.to_sheet(spot.sheet) * transform;
        let far = parts
            .get(i)
            .and_then(|p| placed_bounds(p, to_sheet))
            .map_or(inset + spot.pos.y + o.h, |b| b.y1);
        extents.push((spot.sheet, far));
        layout.parts_area += part.area;
    }
    layout.finish(&spec, &extents);
    layout
}

/// The page-space box of `paths` after `t` (for callers and tests).
pub fn placed_bounds(paths: &[BezPath], t: Affine) -> Option<Rect> {
    paths
        .iter()
        .map(|p| (t * p.clone()).bounding_box())
        .reduce(|a, b| a.union(b))
}

#[cfg(test)]
mod tests {
    use super::super::rects::{nest_rects, RectItem, RectSettings};
    use super::*;
    use tracedraw_core::geometry::{Circle, Rect};

    fn poly(pts: &[(f64, f64)]) -> BezPath {
        let mut p = BezPath::new();
        for (i, q) in pts.iter().enumerate() {
            if i == 0 {
                p.move_to(*q);
            } else {
                p.line_to(*q);
            }
        }
        p.close_path();
        p
    }

    fn square(x: f64, y: f64, s: f64) -> BezPath {
        poly(&[(x, y), (x + s, y), (x + s, y + s), (x, y + s)])
    }

    fn settings(w: f64, h: f64, spacing: f64) -> ShapeSettings {
        ShapeSettings {
            sheet: SheetSpec {
                width: w,
                height: h,
                spacing,
                ..Default::default()
            },
            rounds: 0,
            ..Default::default()
        }
    }

    fn never() -> impl FnMut() -> bool {
        || false
    }

    /// The objects where they were put, one path each.
    fn moved(parts: &[Vec<BezPath>], l: &Layout) -> Vec<BezPath> {
        parts
            .iter()
            .zip(&l.placements)
            .filter_map(|(p, pl)| {
                let pl = pl.as_ref()?;
                let mut all = BezPath::new();
                for q in p {
                    for el in (pl.transform * q.clone()).elements() {
                        all.push(*el);
                    }
                }
                Some(all)
            })
            .collect()
    }

    fn shared_area(a: &BezPath, b: &BezPath) -> f64 {
        let i = shaping::overlay(a, b, shaping::Op::Intersect);
        crate::measure::measure_object(&[i]).area
    }

    fn no_overlaps(paths: &[BezPath]) {
        for (i, a) in paths.iter().enumerate() {
            for b in &paths[i + 1..] {
                assert!(shared_area(a, b) < 1e-6, "objects overlap");
            }
        }
    }

    #[test]
    fn two_triangles_share_a_square_their_boxes_cannot() {
        let tri = vec![poly(&[(0.0, 0.0), (50.0, 0.0), (0.0, 50.0)])];
        let parts = vec![tri.clone(), tri.clone()];
        let l = nest_shapes(&parts, &settings(52.0, 52.0, 0.0), &mut never());
        assert_eq!(l.placed(), 2);
        assert_eq!(l.sheets, 1);
        let angles: Vec<f64> = l.placements.iter().flatten().map(|p| p.angle).collect();
        assert!(angles.contains(&180.0), "{angles:?}");
        let m = moved(&parts, &l);
        no_overlaps(&m);
        for p in &m {
            let b = p.bounding_box();
            assert!(
                b.x0 > -1e-6 && b.y0 > -1e-6 && b.x1 < 52.0 + 1e-6 && b.y1 < 52.0 + 1e-6,
                "{b:?} {:?}",
                l.placements
            );
        }
        // By their boxes they need two sheets.
        let items: Vec<RectItem> = parts
            .iter()
            .map(|p| RectItem {
                bounds: p[0].bounding_box(),
                area: 1250.0,
            })
            .collect();
        let r = nest_rects(
            &items,
            &RectSettings {
                sheet: settings(52.0, 52.0, 0.0).sheet,
                rotate: true,
            },
        );
        assert_eq!(r.sheets, 2);
        assert!(l.utilisation() > r.utilisation());
    }

    #[test]
    fn a_small_part_goes_into_a_hole_when_allowed() {
        let mut ring = square(0.0, 0.0, 60.0);
        ring.extend(square(10.0, 10.0, 40.0));
        let ring = vec![ring];
        let small = vec![square(100.0, 100.0, 20.0)];
        let parts = vec![ring, small];
        let mut s = settings(62.0, 62.0, 2.0);
        let l = nest_shapes(&parts, &s, &mut never());
        assert_eq!((l.placed(), l.sheets), (2, 1));
        let m = moved(&parts, &l);
        no_overlaps(&m);
        // The ring's net area (2000) and the square's (400).
        assert!((l.parts_area - 2400.0).abs() < 1e-6);
        s.holes = false;
        let l = nest_shapes(&parts, &s, &mut never());
        assert_eq!((l.placed(), l.sheets), (2, 2));
    }

    #[test]
    fn spacing_and_margin_are_kept() {
        let parts = vec![vec![square(0.0, 0.0, 20.0)], vec![square(0.0, 0.0, 20.0)]];
        let mut s = settings(100.0, 30.0, 4.0);
        s.sheet.margin = 3.0;
        let l = nest_shapes(&parts, &s, &mut never());
        assert_eq!(l.placed(), 2);
        let b: Vec<Rect> = moved(&parts, &l).iter().map(|p| p.bounding_box()).collect();
        let (a, c) = if b[0].x0 < b[1].x0 {
            (b[0], b[1])
        } else {
            (b[1], b[0])
        };
        let gap = c.x0 - a.x1;
        assert!((4.0 - 1e-6..=5.0).contains(&gap), "gap {gap}");
        for r in &b {
            assert!(r.x0 >= 3.0 - 1e-6 && r.y0 >= 3.0 - 1e-6 && r.y1 <= 27.0 + 1e-6);
            assert!(r.y0 <= 3.0 + 1.0, "slid down to the margin");
        }
    }

    #[test]
    fn curved_outlines_keep_the_spacing_too() {
        let c = Circle::new((0.0, 0.0), 10.0).to_path(0.01);
        let parts = vec![vec![c.clone()], vec![c]];
        let l = nest_shapes(&parts, &settings(100.0, 25.0, 3.0), &mut never());
        assert_eq!((l.placed(), l.sheets), (2, 1));
        let centres: Vec<Point> = l
            .placements
            .iter()
            .flatten()
            .map(|p| p.transform * Point::ZERO)
            .collect();
        let dist = (centres[0] - centres[1]).hypot();
        assert!((23.0 - 1e-6..=24.0).contains(&dist), "{dist}");
    }

    /// A larger job for timing by hand:
    /// `cargo test --release -p tracedraw-tracecut timing -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn timing_of_a_larger_job() {
        let parts: Vec<Vec<BezPath>> = (0..150)
            .map(|i| {
                let k = (i % 7) as f64;
                let mut p = Circle::new((0.0, 0.0), 40.0 + 10.0 * k).to_path(0.01);
                if i % 3 == 0 {
                    p.extend(Circle::new((0.0, 0.0), 20.0 + 5.0 * k).to_path(0.01));
                }
                let mut q = poly(&[(0.0, 0.0), (150.0 + 20.0 * k, 0.0), (70.0, 90.0 + 10.0 * k)]);
                if i % 2 == 0 {
                    q = p;
                }
                vec![q]
            })
            .collect();
        let s = ShapeSettings {
            rounds: 20,
            ..Default::default()
        };
        let t = std::time::Instant::now();
        let l = nest_shapes(&parts, &s, &mut never());
        println!(
            "{} placed on {} sheets, {:.1} %, last length {:.0}, {} rounds in {:?}",
            l.placed(),
            l.sheets,
            l.utilisation() * 100.0,
            l.length,
            l.rounds,
            t.elapsed()
        );
        no_overlaps(&moved(&parts, &l));
    }

    #[test]
    fn a_roll_is_filled_along_its_length() {
        let parts: Vec<Vec<BezPath>> = (0..6).map(|_| vec![square(0.0, 0.0, 20.0)]).collect();
        let mut s = settings(50.0, 0.0, 0.0);
        s.sheet.roll = true;
        let l = nest_shapes(&parts, &s, &mut never());
        assert_eq!((l.placed(), l.sheets), (6, 1));
        assert!(l.length > 60.0 - 1e-6 && l.length < 62.0, "{}", l.length);
        let m = moved(&parts, &l);
        no_overlaps(&m);
        let x1 = m.iter().map(|p| p.bounding_box().x1).fold(0.0, f64::max);
        assert!((x1 - l.length).abs() < 1.0);
        assert!((l.used_area - 50.0 * l.length).abs() < 1e-6);
    }

    #[test]
    fn eight_turns_fit_a_bar_across_the_diagonal() {
        // A 90 x 6 bar.
        let parts = [vec![poly(&[
            (0.0, 0.0),
            (90.0, 0.0),
            (90.0, 6.0),
            (0.0, 6.0),
        ])]];
        let mut s = settings(80.0, 80.0, 0.0);
        let l = nest_shapes(&parts[..1], &s, &mut never());
        assert_eq!(l.placed(), 0);
        s.rotations = Rotations::Eighth;
        let l = nest_shapes(&parts[..1], &s, &mut never());
        let p = l.placements[0].expect("fits turned 45 degrees");
        assert!(p.angle % 90.0 != 0.0);
    }

    #[test]
    fn the_same_seed_gives_the_same_layout_and_rounds_never_make_it_worse() {
        let parts: Vec<Vec<BezPath>> = (0..7)
            .map(|i| {
                let k = i as f64;
                vec![poly(&[
                    (0.0, 0.0),
                    (12.0 + 3.0 * k, 0.0),
                    (6.0 + k, 9.0 + (i % 3) as f64 * 4.0),
                ])]
            })
            .collect();
        let mut s = settings(40.0, 40.0, 1.0);
        s.tolerance = 1.0;
        let first = nest_shapes(&parts, &s, &mut never());
        s.rounds = 6;
        s.seed = 3;
        let a = nest_shapes(&parts, &s, &mut never());
        let b = nest_shapes(&parts, &s, &mut never());
        assert_eq!(a, b);
        assert_eq!(a.rounds, 6);
        assert!(a.placed() >= first.placed());
        assert!(a.sheets <= first.sheets);
        if a.sheets == first.sheets {
            assert!(a.length <= first.length + 1e-9);
        }
        no_overlaps(&moved(&parts, &a));
    }

    #[test]
    fn running_out_of_time_keeps_the_first_layout() {
        let parts: Vec<Vec<BezPath>> = (0..3).map(|_| vec![square(0.0, 0.0, 10.0)]).collect();
        let mut s = settings(40.0, 40.0, 1.0);
        s.rounds = 50;
        let l = nest_shapes(&parts, &s, &mut || true);
        assert!(l.stopped_by_time);
        assert_eq!(l.rounds, 0);
        assert_eq!(l.placed(), 3);
        // Time running out within a round drops that round.
        let mut calls = 0;
        let l = nest_shapes(&parts, &s, &mut || {
            calls += 1;
            calls > 2
        });
        assert!(l.stopped_by_time && l.rounds == 1 && l.placed() == 3);
    }

    #[test]
    fn curves_open_lines_and_bad_input() {
        let circle = Circle::new((0.0, 0.0), 10.0).to_path(0.01);
        let mut line = BezPath::new();
        line.move_to((0.0, 0.0));
        line.line_to((30.0, 0.0));
        let mut bad = BezPath::new();
        bad.move_to((f64::NAN, 0.0));
        bad.line_to((1.0, f64::INFINITY));
        bad.line_to((2.0, 2.0));
        bad.close_path();
        let parts = vec![vec![circle], vec![line], vec![bad], vec![]];
        let l = nest_shapes(&parts, &settings(100.0, 100.0, 2.0), &mut never());
        assert!(l.placements[0].is_some());
        assert!(l.placements[1].is_some(), "an open line by its box");
        assert!(l.placements[3].is_none());
        assert!(l.parts_area > 300.0);
        no_overlaps(&moved(
            &parts[..2],
            &Layout {
                placements: l.placements[..2].to_vec(),
                ..Default::default()
            },
        ));
        let mut s = settings(f64::NAN, 100.0, 2.0);
        assert_eq!(nest_shapes(&parts, &s, &mut never()).placed(), 0);
        s = settings(100.0, 100.0, f64::NAN);
        s.tolerance = f64::NAN;
        assert!(nest_shapes(&parts, &s, &mut never()).placed() >= 1);
        assert!(placed_bounds(&[], Affine::IDENTITY).is_none());
    }
}
