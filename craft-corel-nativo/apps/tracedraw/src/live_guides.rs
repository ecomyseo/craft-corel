//! Alignment guides and dynamic guides: temporary lines drawn while
//! objects are moved, drawn or resized (see `docs/behavior/live-guides.md`).
//!
//! Everything here is pure geometry in page millimetres: the app collects
//! the boxes of the visible objects once per drawing change
//! ([`AlignTargets`]), asks for a correction and the lines to show, and the
//! canvas draws them in screen space.

use serde::{Deserialize, Serialize};
use tracedraw_core::geometry::{Point, Rect, Vec2};

/// Below this, two values are the same line.
const SAME: f64 = 1e-6;

/// Line style of a guide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GuideStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

impl GuideStyle {
    pub const ALL: [GuideStyle; 3] = [GuideStyle::Solid, GuideStyle::Dashed, GuideStyle::Dotted];

    pub fn key(self) -> &'static str {
        match self {
            GuideStyle::Solid => "options.guide_style_solid",
            GuideStyle::Dashed => "options.guide_style_dashed",
            GuideStyle::Dotted => "options.guide_style_dotted",
        }
    }
}

/// Options > Alignment Guides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AlignOptions {
    /// Each object of the selection aligns by its own box (else the
    /// selection's box as one).
    pub each_object: bool,
    /// Align edges and centres.
    pub edges: bool,
    pub centers: bool,
    /// The page's edges and centre are targets too.
    pub page: bool,
    /// Margin lines at `offset_mm` outside and `inset_mm` inside the other
    /// objects' and the page's edges.
    pub margins: bool,
    pub offset_mm: f64,
    pub inset_mm: f64,
    /// A move snaps to equal gaps between neighbours and shows them.
    pub equal_spacing: bool,
    /// A box being drawn or resized snaps to the width or height of
    /// another object and marks both.
    pub equal_size: bool,
    pub color: [u8; 3],
    pub style: GuideStyle,
}

impl Default for AlignOptions {
    fn default() -> Self {
        AlignOptions {
            each_object: false,
            edges: true,
            centers: true,
            page: true,
            margins: false,
            offset_mm: 5.0,
            inset_mm: 5.0,
            equal_spacing: true,
            equal_size: true,
            color: [230, 40, 150],
            style: GuideStyle::Solid,
        }
    }
}

/// Options > Dynamic Guides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DynamicOptions {
    /// The guides' angles in degrees, counter-clockwise from the x axis
    /// (each guide runs both ways, so 0 and 180 are the same line).
    pub angles: Vec<f64>,
    /// The distance from the anchor and the angle beside the pointer.
    pub tooltip: bool,
    /// Tick marks along the guide every `tick_mm`, which the point snaps
    /// to when `snap_ticks` is on.
    pub ticks: bool,
    pub tick_mm: f64,
    pub snap_ticks: bool,
    pub color: [u8; 3],
    pub style: GuideStyle,
}

impl Default for DynamicOptions {
    fn default() -> Self {
        DynamicOptions {
            angles: vec![0.0, 45.0, 90.0, 135.0],
            tooltip: true,
            ticks: false,
            tick_mm: 5.0,
            snap_ticks: false,
            color: [40, 150, 90],
            style: GuideStyle::Dashed,
        }
    }
}

impl DynamicOptions {
    /// The angles to use: finite, in [0, 180), without repeats (at most
    /// 36). A file edited by hand may hold anything.
    pub fn clean_angles(&self) -> Vec<f64> {
        let mut out: Vec<f64> = Vec::new();
        for a in self.angles.iter().copied().filter(|a| a.is_finite()) {
            let a = a.rem_euclid(180.0);
            if !out.iter().any(|b| (b - a).abs() < 1e-6) && out.len() < 36 {
                out.push(a);
            }
        }
        out
    }
}

/// A line segment to draw (page coordinates).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seg {
    pub a: Point,
    pub b: Point,
}

/// A candidate value on one axis and the box it comes from (the page's
/// or an object's), to draw the line from the moving box to it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub v: f64,
    pub from: Rect,
}

/// The other objects' boxes and the values they offer, built once per
/// drawing change and view (only the visible objects).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AlignTargets {
    pub boxes: Vec<Rect>,
    /// The boxes' widths and heights (sorted), for equal sizes.
    pub widths: Vec<Target>,
    pub heights: Vec<Target>,
    /// Sorted by value, for a quick search.
    pub xs: Vec<Target>,
    pub ys: Vec<Target>,
}

impl AlignTargets {
    /// The targets of `boxes` and of `pages` (the page in view, and the
    /// other pages of a multipage view, as they are displayed).
    pub fn new(boxes: Vec<Rect>, pages: &[Rect], o: &AlignOptions) -> Self {
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut widths: Vec<Target> = boxes
            .iter()
            .map(|b| Target {
                v: b.width(),
                from: *b,
            })
            .filter(|t| t.v.is_finite() && t.v > SAME)
            .collect();
        let mut heights: Vec<Target> = boxes
            .iter()
            .map(|b| Target {
                v: b.height(),
                from: *b,
            })
            .filter(|t| t.v.is_finite() && t.v > SAME)
            .collect();
        widths.sort_by(|a, b| a.v.total_cmp(&b.v));
        heights.sort_by(|a, b| a.v.total_cmp(&b.v));
        let all = boxes.iter().chain(pages).copied();
        for r in all {
            if o.edges {
                xs.extend([r.x0, r.x1].map(|v| Target { v, from: r }));
                ys.extend([r.y0, r.y1].map(|v| Target { v, from: r }));
            }
            if o.centers {
                let c = r.center();
                xs.push(Target { v: c.x, from: r });
                ys.push(Target { v: c.y, from: r });
            }
            if o.margins {
                let (out, inn) = (o.offset_mm.max(0.0), o.inset_mm.max(0.0));
                xs.extend(
                    [r.x0 - out, r.x1 + out, r.x0 + inn, r.x1 - inn].map(|v| Target { v, from: r }),
                );
                ys.extend(
                    [r.y0 - out, r.y1 + out, r.y0 + inn, r.y1 - inn].map(|v| Target { v, from: r }),
                );
            }
        }
        xs.retain(|t| t.v.is_finite());
        ys.retain(|t| t.v.is_finite());
        xs.sort_by(|a, b| a.v.total_cmp(&b.v));
        ys.sort_by(|a, b| a.v.total_cmp(&b.v));
        AlignTargets {
            boxes,
            widths,
            heights,
            xs,
            ys,
        }
    }
}

/// The targets whose value lies within `tol` of `v` (a binary search in
/// the sorted list).
fn near(list: &[Target], v: f64, tol: f64) -> &[Target] {
    let lo = list.partition_point(|t| t.v < v - tol);
    let hi = list.partition_point(|t| t.v <= v + tol);
    list.get(lo..hi).unwrap_or(&[])
}

/// The values of a box that align: its edges and centre on one axis.
fn features(r: Rect, horizontal: bool, o: &AlignOptions) -> Vec<f64> {
    let (a, b) = if horizontal {
        (r.x0, r.x1)
    } else {
        (r.y0, r.y1)
    };
    let mut out = Vec::new();
    if o.edges {
        out.extend([a, b]);
    }
    if o.centers {
        out.push((a + b) / 2.0);
    }
    out
}

/// The smallest correction within `tol` that puts one of `values` on a
/// target.
fn best_shift(values: &[f64], list: &[Target], tol: f64) -> Option<f64> {
    let mut best: Option<f64> = None;
    for v in values {
        for t in near(list, *v, tol) {
            let d = t.v - v;
            if best.map(|b| d.abs() < b.abs()).unwrap_or(true) {
                best = Some(d);
            }
        }
    }
    best
}

/// The lines between the moving boxes and the targets they now line up
/// with (vertical lines for x values, horizontal for y values), each from
/// one box to the other along the other axis.
pub fn lines_for(moving: &[Rect], t: &AlignTargets, o: &AlignOptions) -> Vec<Seg> {
    let mut out: Vec<Seg> = Vec::new();
    for m in moving {
        for x in features(*m, true, o) {
            for tg in near(&t.xs, x, SAME) {
                let y0 = m.y0.min(tg.from.y0);
                let y1 = m.y1.max(tg.from.y1);
                push_unique(
                    &mut out,
                    Seg {
                        a: Point::new(tg.v, y0),
                        b: Point::new(tg.v, y1),
                    },
                );
            }
        }
        for y in features(*m, false, o) {
            for tg in near(&t.ys, y, SAME) {
                let x0 = m.x0.min(tg.from.x0);
                let x1 = m.x1.max(tg.from.x1);
                push_unique(
                    &mut out,
                    Seg {
                        a: Point::new(x0, tg.v),
                        b: Point::new(x1, tg.v),
                    },
                );
            }
        }
    }
    out
}

/// Lines on the same value merge into one spanning both.
fn push_unique(out: &mut Vec<Seg>, s: Seg) {
    let vertical = (s.a.x - s.b.x).abs() < SAME;
    for o in out.iter_mut() {
        let ov = (o.a.x - o.b.x).abs() < SAME;
        if vertical && ov && (o.a.x - s.a.x).abs() < SAME {
            o.a.y = o.a.y.min(s.a.y);
            o.b.y = o.b.y.max(s.b.y);
            return;
        }
        if !vertical && !ov && (o.a.y - s.a.y).abs() < SAME {
            o.a.x = o.a.x.min(s.a.x);
            o.b.x = o.b.x.max(s.b.x);
            return;
        }
    }
    out.push(s);
}

/// What a move with alignment guides gives: the correction to add to the
/// translation on each axis (`None` where nothing aligned) and the lines
/// and equal gaps to draw at the corrected place.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AlignMove {
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub lines: Vec<Seg>,
    pub gaps: Vec<Seg>,
}

/// Align boxes moved by `d`: their edges or centres onto the targets, or
/// (with equal spacing) into equal gaps between neighbours, whichever is
/// nearer, within `tol`.
pub fn align_move(
    moving: &[Rect],
    d: Vec2,
    t: &AlignTargets,
    o: &AlignOptions,
    tol: f64,
) -> AlignMove {
    let moved: Vec<Rect> = moving.iter().map(|r| *r + d).collect();
    let xs: Vec<f64> = moved.iter().flat_map(|r| features(*r, true, o)).collect();
    let ys: Vec<f64> = moved.iter().flat_map(|r| features(*r, false, o)).collect();
    let mut dx = best_shift(&xs, &t.xs, tol);
    let mut dy = best_shift(&ys, &t.ys, tol);
    // Equal gaps, for the boxes as one.
    let whole = moved.iter().copied().reduce(|a, b| a.union(b));
    if let (true, Some(w)) = (o.equal_spacing, whole) {
        if let Some(s) = spacing_shift(w, &t.boxes, true, tol) {
            if dx.map(|b| s.abs() < b.abs()).unwrap_or(true) {
                dx = Some(s);
            }
        }
        if let Some(s) = spacing_shift(w, &t.boxes, false, tol) {
            if dy.map(|b| s.abs() < b.abs()).unwrap_or(true) {
                dy = Some(s);
            }
        }
    }
    let fix = Vec2::new(dx.unwrap_or(0.0), dy.unwrap_or(0.0));
    let at: Vec<Rect> = moved.iter().map(|r| *r + fix).collect();
    let lines = lines_for(&at, t, o);
    let gaps = match (
        o.equal_spacing,
        at.iter().copied().reduce(|a, b| a.union(b)),
    ) {
        (true, Some(w)) => {
            let mut g = equal_gaps(w, &t.boxes, true);
            g.extend(equal_gaps(w, &t.boxes, false));
            g
        }
        _ => Vec::new(),
    };
    AlignMove {
        dx,
        dy,
        lines,
        gaps,
    }
}

/// Snap a point (the pointer of a drawing tool, a scale handle) so its x
/// or y lies on a target within `tol`; the lines from the point to what
/// it aligns with.
pub fn align_point(p: Point, t: &AlignTargets, tol: f64) -> (Option<f64>, Option<f64>, Vec<Seg>) {
    let pick = |list: &[Target], v: f64| {
        near(list, v, tol)
            .iter()
            .min_by(|a, b| (a.v - v).abs().total_cmp(&(b.v - v).abs()))
            .copied()
    };
    let tx = pick(&t.xs, p.x);
    let ty = pick(&t.ys, p.y);
    let q = Point::new(
        tx.map(|t| t.v).unwrap_or(p.x),
        ty.map(|t| t.v).unwrap_or(p.y),
    );
    let mut lines = Vec::new();
    if let Some(tx) = tx {
        lines.push(Seg {
            a: Point::new(tx.v, q.y.min(tx.from.y0)),
            b: Point::new(tx.v, q.y.max(tx.from.y1)),
        });
    }
    if let Some(ty) = ty {
        lines.push(Seg {
            a: Point::new(q.x.min(ty.from.x0), ty.v),
            b: Point::new(q.x.max(ty.from.x1), ty.v),
        });
    }
    (tx.map(|t| t.v), ty.map(|t| t.v), lines)
}

/// The span of a box on one axis and on the other.
fn span(r: Rect, horizontal: bool) -> ((f64, f64), (f64, f64)) {
    if horizontal {
        ((r.x0, r.x1), (r.y0, r.y1))
    } else {
        ((r.y0, r.y1), (r.x0, r.x1))
    }
}

/// The boxes that share the moving box's band (they overlap it across the
/// axis), sorted along the axis.
fn band(m: Rect, boxes: &[Rect], horizontal: bool) -> Vec<(f64, f64)> {
    let (_, (c0, c1)) = span(m, horizontal);
    let mut out: Vec<(f64, f64)> = boxes
        .iter()
        .filter_map(|b| {
            let (along, (d0, d1)) = span(*b, horizontal);
            (d0 < c1 && d1 > c0).then_some(along)
        })
        .collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    out
}

/// The nearest neighbours before and after the moving box along the axis
/// (they do not overlap it), and the gap each has to its own neighbour on
/// the far side.
struct Neighbours {
    before: Option<(f64, f64)>,
    after: Option<(f64, f64)>,
    before_gap: Option<f64>,
    after_gap: Option<f64>,
}

fn neighbours(m: Rect, boxes: &[Rect], horizontal: bool) -> Neighbours {
    let ((a, b), _) = span(m, horizontal);
    let row = band(m, boxes, horizontal);
    let before = row
        .iter()
        .filter(|s| s.1 <= a + SAME && s.0 < a)
        .max_by(|x, y| x.1.total_cmp(&y.1))
        .copied();
    let after = row
        .iter()
        .filter(|s| s.0 >= b - SAME && s.1 > b)
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .copied();
    // Gaps between neighbours further out, in the same band.
    let before_gap = before.and_then(|n| {
        row.iter()
            .filter(|s| s.1 <= n.0 + SAME && s.0 < n.0)
            .map(|s| n.0 - s.1)
            .min_by(|x, y| x.total_cmp(y))
    });
    let after_gap = after.and_then(|n| {
        row.iter()
            .filter(|s| s.0 >= n.1 - SAME && s.1 > n.1)
            .map(|s| s.0 - n.1)
            .min_by(|x, y| x.total_cmp(y))
    });
    Neighbours {
        before,
        after,
        before_gap,
        after_gap,
    }
}

/// The smallest shift within `tol` that makes the gaps around the moving
/// box equal: centred between its two neighbours, or as far from one
/// neighbour as that neighbour is from the next.
pub fn spacing_shift(m: Rect, boxes: &[Rect], horizontal: bool, tol: f64) -> Option<f64> {
    let ((a, b), _) = span(m, horizontal);
    let n = neighbours(m, boxes, horizontal);
    let mut wanted = Vec::new();
    if let (Some(l), Some(r)) = (n.before, n.after) {
        let free = r.0 - l.1 - (b - a);
        if free >= 0.0 {
            wanted.push(l.1 + free / 2.0 - a);
        }
    }
    if let (Some(l), Some(g)) = (n.before, n.before_gap) {
        wanted.push(l.1 + g - a);
    }
    if let (Some(r), Some(g)) = (n.after, n.after_gap) {
        wanted.push(r.0 - g - b);
    }
    wanted
        .into_iter()
        .filter(|s| s.abs() <= tol && s.is_finite())
        .min_by(|x, y| x.abs().total_cmp(&y.abs()))
}

/// The gaps to mark when the moving box sits at equal spacing: the gap on
/// each side when both are equal, or the gap on one side and the equal one
/// beyond its neighbour. Each is a segment across the gap, through the
/// middle of the band.
pub fn equal_gaps(m: Rect, boxes: &[Rect], horizontal: bool) -> Vec<Seg> {
    let ((a, b), (c0, c1)) = span(m, horizontal);
    let mid = (c0 + c1) / 2.0;
    let n = neighbours(m, boxes, horizontal);
    let seg = |s: f64, e: f64| {
        if horizontal {
            Seg {
                a: Point::new(s, mid),
                b: Point::new(e, mid),
            }
        } else {
            Seg {
                a: Point::new(mid, s),
                b: Point::new(mid, e),
            }
        }
    };
    let eq = |x: f64, y: f64| (x - y).abs() < 1e-4 && x > SAME;
    let mut out = Vec::new();
    if let (Some(l), Some(r)) = (n.before, n.after) {
        if eq(a - l.1, r.0 - b) {
            out.push(seg(l.1, a));
            out.push(seg(b, r.0));
        }
    }
    if let (Some(l), Some(g)) = (n.before, n.before_gap) {
        if eq(a - l.1, g) {
            out.push(seg(l.1, a));
            out.push(seg(l.0 - g, l.0));
        }
    }
    if let (Some(r), Some(g)) = (n.after, n.after_gap) {
        if eq(r.0 - b, g) {
            out.push(seg(b, r.0));
            out.push(seg(r.1, r.1 + g));
        }
    }
    out.dedup();
    out
}

/// A box being drawn or resized from the fixed point `anchor` to `p`
/// (its size is `factor` times the distance on each axis: 2 when it grows
/// from its centre): on the axes asked for, the size snaps to another
/// object's width or height within `tol` (of the pointer). Returns the
/// snapped coordinates and the size marks: the box's side and the other
/// object's, for each axis that matched (bottom sides for widths, left
/// sides for heights).
pub fn size_snap(
    p: Point,
    anchor: Point,
    factor: f64,
    axes: (bool, bool),
    t: &AlignTargets,
    tol: f64,
) -> (Option<f64>, Option<f64>, Vec<Seg>) {
    let factor = if factor.is_finite() && factor > SAME {
        factor
    } else {
        1.0
    };
    let pick = |list: &[Target], from: f64, to: f64| -> Option<(f64, Rect)> {
        let size = (to - from).abs() * factor;
        if size <= SAME {
            return None;
        }
        let m = near(list, size, tol * factor)
            .iter()
            .min_by(|a, b| (a.v - size).abs().total_cmp(&(b.v - size).abs()))?;
        let sign = if to >= from { 1.0 } else { -1.0 };
        Some((from + sign * m.v / factor, m.from))
    };
    let wx = axes.0.then(|| pick(&t.widths, anchor.x, p.x)).flatten();
    let hy = axes.1.then(|| pick(&t.heights, anchor.y, p.y)).flatten();
    let q = Point::new(
        wx.map(|w| w.0).unwrap_or(p.x),
        hy.map(|h| h.0).unwrap_or(p.y),
    );
    // The box as it now is.
    let half = factor > 1.5;
    let span = |a: f64, b: f64| {
        if half {
            (a - (b - a), b)
        } else {
            (a, b)
        }
    };
    let (x0, x1) = span(anchor.x, q.x);
    let (y0, y1) = span(anchor.y, q.y);
    let me = Rect::new(x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1));
    let mut marks = Vec::new();
    if let Some((_, o)) = wx {
        for b in [me, o] {
            marks.push(Seg {
                a: Point::new(b.x0, b.y0),
                b: Point::new(b.x1, b.y0),
            });
        }
    }
    if let Some((_, o)) = hy {
        for b in [me, o] {
            marks.push(Seg {
                a: Point::new(b.x0, b.y0),
                b: Point::new(b.x0, b.y1),
            });
        }
    }
    (wx.map(|w| w.0), hy.map(|h| h.0), marks)
}

/// What the canvas draws this frame: alignment lines, equal gaps and the
/// dynamic guide the point snapped to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Overlay {
    pub lines: Vec<Seg>,
    pub gaps: Vec<Seg>,
    /// Equal sizes: a side of the box being made and of the object it
    /// matches, marked outside the box (below a width, left of a height).
    pub sizes: Vec<Seg>,
    pub dynamic: Option<DynHit>,
}

impl Overlay {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
            && self.gaps.is_empty()
            && self.sizes.is_empty()
            && self.dynamic.is_none()
    }
}

// ----- Dynamic guides ------------------------------------------------

/// The dynamic guide a point snapped to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DynHit {
    /// The snap point the guide comes from.
    pub anchor: Point,
    /// The guide's angle (degrees, [0, 180)).
    pub angle: f64,
    /// Where the point landed on the guide.
    pub point: Point,
    /// A second guide crossing there, when the point sits on both.
    pub cross: Option<(Point, f64)>,
}

impl DynHit {
    /// The distance from the anchor.
    pub fn distance(&self) -> f64 {
        (self.point - self.anchor).hypot()
    }

    /// The direction from the anchor to the point, degrees in [0, 360).
    pub fn bearing(&self) -> f64 {
        let v = self.point - self.anchor;
        if v.hypot() < SAME {
            return self.angle;
        }
        v.y.atan2(v.x).to_degrees().rem_euclid(360.0)
    }
}

fn dir(angle_deg: f64) -> Vec2 {
    let a = angle_deg.to_radians();
    Vec2::new(a.cos(), a.sin())
}

/// The foot of `p` on the guide through `anchor` at `angle`, and its
/// distance from `p`.
fn foot(p: Point, anchor: Point, angle: f64) -> (Point, f64) {
    let u = dir(angle);
    let t = (p - anchor).dot(u);
    let q = anchor + u * t;
    (q, (p - q).hypot())
}

/// The point where two guides cross, if they are not parallel.
fn crossing(a: Point, angle_a: f64, b: Point, angle_b: f64) -> Option<Point> {
    let (u, v) = (dir(angle_a), dir(angle_b));
    let den = u.x * v.y - u.y * v.x;
    if den.abs() < 1e-9 {
        return None;
    }
    let w = b - a;
    let t = (w.x * v.y - w.y * v.x) / den;
    Some(a + u * t)
}

/// Snap `p` onto the nearest guide within `tol` from one of `anchors` at
/// one of `angles`; on two crossing guides, onto their crossing. A point
/// on the anchor itself (closer than `tol`) is left to the point snap.
pub fn dynamic_snap(p: Point, anchors: &[Point], angles: &[f64], tol: f64) -> Option<DynHit> {
    let mut hits: Vec<(f64, Point, f64, Point)> = Vec::new();
    for a in anchors {
        if (p - *a).hypot() <= tol {
            continue;
        }
        for ang in angles {
            let (q, d) = foot(p, *a, *ang);
            if d <= tol {
                hits.push((d, *a, *ang, q));
            }
        }
    }
    hits.sort_by(|x, y| x.0.total_cmp(&y.0));
    let (_, anchor, angle, point) = *hits.first()?;
    // A second guide (another anchor, or the same at another angle does
    // not count: they cross at the anchor) crossing near the pointer.
    let cross = hits.iter().skip(1).find_map(|(_, b, bang, _)| {
        if (*b - anchor).hypot() < SAME {
            return None;
        }
        let c = crossing(anchor, angle, *b, *bang)?;
        ((c - p).hypot() <= tol).then_some((c, *bang))
    });
    let point = cross.map(|c| c.0).unwrap_or(point);
    Some(DynHit {
        anchor,
        angle,
        point,
        cross,
    })
}

/// The point moved along its guide onto the nearest tick (every `step`
/// from the anchor) when that is within `tol`.
pub fn snap_to_tick(hit: DynHit, step: f64, tol: f64) -> DynHit {
    if step.is_nan() || step <= SAME || hit.cross.is_some() {
        return hit;
    }
    let u = dir(hit.angle);
    let t = (hit.point - hit.anchor).dot(u);
    let k = (t / step).round() * step;
    if (k - t).abs() <= tol {
        DynHit {
            point: hit.anchor + u * k,
            ..hit
        }
    } else {
        hit
    }
}

/// Tick marks between the anchor and the point, every `step` (at most
/// `max`).
pub fn ticks(anchor: Point, point: Point, step: f64, max: usize) -> Vec<Point> {
    let v = point - anchor;
    let len = v.hypot();
    if step.is_nan() || step <= SAME || len < step {
        return Vec::new();
    }
    let u = v * (1.0 / len);
    let n = ((len / step).floor() as usize).min(max);
    (1..=n).map(|i| anchor + u * (step * i as f64)).collect()
}

/// Add an acquired snap point to the recent anchors (the newest first, no
/// repeats, at most `max`).
pub fn acquire(anchors: &mut Vec<Point>, p: Point, max: usize) {
    anchors.retain(|a| (*a - p).hypot() > SAME);
    anchors.insert(0, p);
    anchors.truncate(max);
}

/// The guide through `anchor` at `angle`, clipped to `view`.
pub fn guide_in_view(anchor: Point, angle: f64, view: Rect) -> Option<Seg> {
    let u = dir(angle);
    // Parametric clip of the infinite line to the box (Liang-Barsky).
    let (mut t0, mut t1) = (f64::NEG_INFINITY, f64::INFINITY);
    for (p, q) in [
        (-u.x, anchor.x - view.x0),
        (u.x, view.x1 - anchor.x),
        (-u.y, anchor.y - view.y0),
        (u.y, view.y1 - anchor.y),
    ] {
        if p.abs() < 1e-12 {
            if q < 0.0 {
                return None;
            }
            continue;
        }
        let r = q / p;
        if p < 0.0 {
            t0 = t0.max(r);
        } else {
            t1 = t1.min(r);
        }
    }
    (t0.is_finite() && t1.is_finite() && t0 <= t1).then(|| Seg {
        a: anchor + u * t0,
        b: anchor + u * t1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x0: f64, y0: f64, x1: f64, y1: f64) -> Rect {
        Rect::new(x0, y0, x1, y1)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn a_moved_box_lines_up_with_another_boxs_edge_and_shows_the_line() {
        let o = AlignOptions::default();
        let t = AlignTargets::new(vec![r(100.0, 100.0, 140.0, 120.0)], &[], &o);
        // The moving box's top edge comes within 1 mm of the other's top
        // (its bottom 3 mm from the other's bottom, its centre 2 mm).
        let m = align_move(
            &[r(10.0, 10.0, 30.0, 28.0)],
            Vec2::new(0.0, 93.0),
            &t,
            &o,
            3.0,
        );
        assert!(close(m.dy.unwrap_or(0.0), -1.0), "{m:?}");
        assert_eq!(m.dx, None);
        // One horizontal line at y = 120 from the moved box to the other.
        assert_eq!(m.lines.len(), 1, "{:?}", m.lines);
        let l = m.lines[0];
        assert!(close(l.a.y, 120.0) && close(l.b.y, 120.0));
        assert!(close(l.a.x, 10.0) && close(l.b.x, 140.0));
    }

    #[test]
    fn centres_and_edges_can_be_turned_off() {
        let mut o = AlignOptions::default();
        let other = r(100.0, 100.0, 140.0, 120.0);
        // Moving centre x = 21 near the other's centre x = 120 after the
        // move; edges far from 100 and 140.
        let moving = [r(10.0, 0.0, 32.0, 5.0)];
        let d = Vec2::new(98.0, 0.0);
        let t = AlignTargets::new(vec![other], &[], &o);
        assert!(close(
            align_move(&moving, d, &t, &o, 2.0).dx.unwrap_or(9.0),
            1.0
        ));
        o.centers = false;
        let t = AlignTargets::new(vec![other], &[], &o);
        assert_eq!(align_move(&moving, d, &t, &o, 2.0).dx, None);
        // Edges only: the left edge at 108 is not near 100 or 140; at 99.
        let d = Vec2::new(89.0, 0.0);
        assert!(close(
            align_move(&moving, d, &t, &o, 2.0).dx.unwrap_or(9.0),
            1.0
        ));
        o.edges = false;
        o.centers = true;
        let t = AlignTargets::new(vec![other], &[], &o);
        assert_eq!(align_move(&moving, d, &t, &o, 2.0).dx, None);
    }

    #[test]
    fn each_object_of_the_selection_aligns_by_its_own_box() {
        let o = AlignOptions {
            centers: false,
            equal_spacing: false,
            ..Default::default()
        };
        let t = AlignTargets::new(vec![r(200.0, 20.0, 210.0, 30.0)], &[], &o);
        // Two boxes moving together; the inner edge 150 of the first is
        // the only one near 199 after a move of 49 + 1.
        let a = r(100.0, 0.0, 150.0, 5.0);
        let b = r(155.0, 0.0, 170.0, 5.0);
        let d = Vec2::new(49.0, 0.0);
        // As one box (100..170) nothing is near 200 or 210.
        let whole = [a.union(b)];
        assert_eq!(align_move(&whole, d, &t, &o, 2.0).dx, None);
        // Each on its own: 150 + 49 = 199 goes to 200.
        let m = align_move(&[a, b], d, &t, &o, 2.0);
        assert!(close(m.dx.unwrap_or(0.0), 1.0), "{m:?}");
        assert_eq!(m.lines.len(), 1);
    }

    #[test]
    fn the_page_and_margins_are_targets() {
        let mut o = AlignOptions::default();
        let page = r(0.0, 0.0, 210.0, 297.0);
        let t = AlignTargets::new(Vec::new(), &[page], &o);
        // The page centre.
        let (x, _, lines) = align_point(Point::new(104.0, 50.0), &t, 2.0);
        assert_eq!(x, Some(105.0));
        assert_eq!(lines.len(), 1);
        // A 10 mm inset of the page's left edge.
        o.margins = true;
        o.inset_mm = 10.0;
        let t = AlignTargets::new(Vec::new(), &[page], &o);
        let (x, _, _) = align_point(Point::new(11.0, 50.0), &t, 2.0);
        assert_eq!(x, Some(10.0));
        // And an offset outside another object.
        o.offset_mm = 4.0;
        let t = AlignTargets::new(vec![r(50.0, 50.0, 60.0, 60.0)], &[], &o);
        let (x, y, _) = align_point(Point::new(64.5, 45.5), &t, 1.0);
        assert_eq!((x, y), (Some(64.0), Some(46.0)));
    }

    #[test]
    fn a_point_far_from_everything_is_left_alone() {
        let o = AlignOptions::default();
        let t = AlignTargets::new(vec![r(0.0, 0.0, 10.0, 10.0)], &[], &o);
        let (x, y, l) = align_point(Point::new(30.0, 40.0), &t, 2.0);
        assert_eq!((x, y), (None, None));
        assert!(l.is_empty());
        // Garbage options from a hand-edited file do not break anything.
        let bad = AlignOptions {
            margins: true,
            offset_mm: f64::NAN,
            ..Default::default()
        };
        let t = AlignTargets::new(vec![r(0.0, 0.0, 10.0, 10.0)], &[], &bad);
        assert!(t.xs.iter().all(|t| t.v.is_finite()));
    }

    #[test]
    fn a_box_between_two_neighbours_snaps_to_the_middle_and_marks_the_gaps() {
        let boxes = [r(0.0, 0.0, 10.0, 10.0), r(50.0, 0.0, 60.0, 10.0)];
        // A 10 mm box at 21..31: centred would be 25..35.
        let m = r(21.0, 2.0, 31.0, 8.0);
        let s = spacing_shift(m, &boxes, true, 5.0).expect("a shift");
        assert!(close(s, 4.0), "{s}");
        let g = equal_gaps(m + Vec2::new(s, 0.0), &boxes, true);
        assert_eq!(g.len(), 2, "{g:?}");
        assert!(close(g[0].a.x, 10.0) && close(g[0].b.x, 25.0));
        assert!(close(g[1].a.x, 35.0) && close(g[1].b.x, 50.0));
        // Out of reach: no shift. Not in the same band: no neighbours.
        assert_eq!(spacing_shift(m, &boxes, true, 1.0), None);
        let high = r(21.0, 30.0, 31.0, 40.0);
        assert_eq!(spacing_shift(high, &boxes, true, 5.0), None);
    }

    #[test]
    fn a_box_after_a_row_repeats_the_rows_gap() {
        // Two boxes 5 mm apart; the third should come 5 mm after the
        // second.
        let boxes = [r(0.0, 0.0, 10.0, 10.0), r(15.0, 0.0, 25.0, 10.0)];
        let m = r(31.5, 0.0, 41.5, 10.0);
        let s = spacing_shift(m, &boxes, true, 2.0).expect("a shift");
        assert!(close(s, -1.5), "{s}");
        let g = equal_gaps(m + Vec2::new(s, 0.0), &boxes, true);
        assert_eq!(g.len(), 2);
        // Vertically too.
        let col = [r(0.0, 0.0, 10.0, 10.0), r(0.0, 20.0, 10.0, 30.0)];
        let m = r(0.0, 41.0, 10.0, 51.0);
        assert!(close(
            spacing_shift(m, &col, false, 2.0).unwrap_or(0.0),
            -1.0
        ));
    }

    #[test]
    fn equal_spacing_takes_part_in_a_move() {
        let o = AlignOptions {
            edges: false,
            centers: false,
            ..Default::default()
        };
        let boxes = vec![r(0.0, 0.0, 10.0, 10.0), r(50.0, 0.0, 60.0, 10.0)];
        let t = AlignTargets::new(boxes, &[], &o);
        let m = align_move(
            &[r(0.0, 20.0, 10.0, 28.0)],
            Vec2::new(23.0, -18.0),
            &t,
            &o,
            3.0,
        );
        assert!(close(m.dx.unwrap_or(0.0), 2.0), "{m:?}");
        assert_eq!(m.gaps.len(), 2);
        let off = AlignOptions {
            equal_spacing: false,
            ..o
        };
        let m = align_move(
            &[r(0.0, 20.0, 10.0, 28.0)],
            Vec2::new(23.0, -18.0),
            &t,
            &off,
            3.0,
        );
        assert_eq!(m.dx, None);
        assert!(m.gaps.is_empty());
    }

    #[test]
    fn a_box_being_drawn_takes_another_objects_width_and_height() {
        let o = AlignOptions::default();
        let t = AlignTargets::new(vec![r(100.0, 100.0, 140.0, 120.0)], &[], &o);
        // From (0, 0) to (41, 19): 40 wide, 20 high within 2 mm.
        let (x, y, marks) = size_snap(
            Point::new(41.0, 19.0),
            Point::new(0.0, 0.0),
            1.0,
            (true, true),
            &t,
            2.0,
        );
        assert_eq!((x, y), (Some(40.0), Some(20.0)));
        // A width mark under each box, a height mark left of each.
        assert_eq!(marks.len(), 4, "{marks:?}");
        assert_eq!(
            marks[0],
            Seg {
                a: Point::new(0.0, 0.0),
                b: Point::new(40.0, 0.0)
            }
        );
        assert_eq!(
            marks[1],
            Seg {
                a: Point::new(100.0, 100.0),
                b: Point::new(140.0, 100.0)
            }
        );
        // Drawn leftwards and downwards: the same sizes, the other way.
        let (x, y, _) = size_snap(
            Point::new(-39.0, -21.5),
            Point::new(0.0, 0.0),
            1.0,
            (true, true),
            &t,
            2.0,
        );
        assert_eq!((x, y), (Some(-40.0), Some(-20.0)));
        // From the centre the pointer goes half as far.
        let (x, _, _) = size_snap(
            Point::new(20.5, 3.0),
            Point::new(0.0, 0.0),
            2.0,
            (true, false),
            &t,
            2.0,
        );
        assert_eq!(x, Some(20.0));
        // Only the axes asked for (a side handle), and nothing far off.
        let (x, y, m) = size_snap(
            Point::new(41.0, 19.0),
            Point::new(0.0, 0.0),
            1.0,
            (false, true),
            &t,
            2.0,
        );
        assert_eq!((x, y, m.len()), (None, Some(20.0), 2));
        let (x, y, m) = size_snap(
            Point::new(60.0, 50.0),
            Point::new(0.0, 0.0),
            1.0,
            (true, true),
            &t,
            2.0,
        );
        assert_eq!((x, y), (None, None));
        assert!(m.is_empty());
    }

    #[test]
    fn other_pages_are_targets_as_displayed() {
        let o = AlignOptions::default();
        // A second page to the right of the first, 30 mm apart.
        let pages = [r(0.0, 0.0, 210.0, 297.0), r(240.0, 0.0, 450.0, 297.0)];
        let t = AlignTargets::new(Vec::new(), &pages, &o);
        let (x, _, _) = align_point(Point::new(238.5, 50.0), &t, 2.0);
        assert_eq!(x, Some(240.0));
    }

    #[test]
    fn a_point_snaps_onto_a_guide_from_an_anchor_at_the_chosen_angles() {
        let angles = DynamicOptions::default().clean_angles();
        let a = Point::new(0.0, 0.0);
        // Near the 45 degree guide.
        let h = dynamic_snap(Point::new(30.0, 31.0), &[a], &angles, 2.0).expect("a hit");
        assert!(close(h.angle, 45.0));
        assert!(close(h.point.x, 30.5) && close(h.point.y, 30.5));
        assert!(close(h.distance(), 30.5 * 2f64.sqrt()));
        assert!(close(h.bearing(), 45.0));
        // The other way along the same guide reads 225 degrees.
        let h = dynamic_snap(Point::new(-30.0, -29.0), &[a], &angles, 2.0).expect("a hit");
        assert!(close(h.bearing(), 225.0));
        // Not near any guide; on the anchor itself.
        assert!(dynamic_snap(Point::new(30.0, 12.0), &[a], &angles, 2.0).is_none());
        assert!(dynamic_snap(Point::new(1.0, 0.5), &[a], &angles, 2.0).is_none());
        // Only 30 degrees.
        let h = dynamic_snap(Point::new(30.0, 18.0), &[a], &[30.0], 2.0).expect("30");
        assert!(close(h.angle, 30.0));
    }

    #[test]
    fn two_guides_from_two_anchors_catch_their_crossing() {
        let anchors = [Point::new(0.0, 0.0), Point::new(50.0, 40.0)];
        // The horizontal from the first and the vertical from the second
        // cross at (50, 0).
        let h = dynamic_snap(Point::new(49.0, 1.0), &anchors, &[0.0, 90.0], 2.0).expect("a hit");
        assert!(close(h.point.x, 50.0) && close(h.point.y, 0.0), "{h:?}");
        assert!(h.cross.is_some());
    }

    #[test]
    fn ticks_mark_the_guide_and_the_point_can_snap_to_them() {
        let t = ticks(Point::new(0.0, 0.0), Point::new(12.0, 0.0), 5.0, 100);
        assert_eq!(t, vec![Point::new(5.0, 0.0), Point::new(10.0, 0.0)]);
        assert!(ticks(Point::new(0.0, 0.0), Point::new(3.0, 0.0), 5.0, 100).is_empty());
        assert!(ticks(Point::new(0.0, 0.0), Point::new(30.0, 0.0), 0.0, 100).is_empty());
        assert_eq!(
            ticks(Point::new(0.0, 0.0), Point::new(1e9, 0.0), 1.0, 50).len(),
            50
        );
        let h = dynamic_snap(Point::new(11.0, 0.5), &[Point::new(0.0, 0.0)], &[0.0], 2.0)
            .expect("a hit");
        let s = snap_to_tick(h, 5.0, 1.5);
        assert_eq!(s.point, Point::new(10.0, 0.0));
        assert_eq!(snap_to_tick(h, 5.0, 0.5).point, Point::new(11.0, 0.0));
    }

    #[test]
    fn anchors_are_kept_newest_first_without_repeats() {
        let mut a = Vec::new();
        for p in [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (0.0, 0.0)] {
            acquire(&mut a, Point::new(p.0, p.1), 3);
        }
        assert_eq!(
            a,
            vec![
                Point::new(0.0, 0.0),
                Point::new(2.0, 2.0),
                Point::new(1.0, 1.0)
            ]
        );
        acquire(&mut a, Point::new(5.0, 5.0), 3);
        assert_eq!(a.len(), 3);
        assert_eq!(a[0], Point::new(5.0, 5.0));
    }

    #[test]
    fn angles_from_a_file_are_cleaned() {
        let o = DynamicOptions {
            angles: vec![0.0, 180.0, f64::NAN, -45.0, 135.0, 400.0],
            ..Default::default()
        };
        assert_eq!(o.clean_angles(), vec![0.0, 135.0, 40.0]);
    }

    #[test]
    fn a_guide_is_clipped_to_the_view() {
        let v = r(0.0, 0.0, 100.0, 50.0);
        let s = guide_in_view(Point::new(10.0, 10.0), 0.0, v).expect("in view");
        assert!(close(s.a.x, 0.0) && close(s.b.x, 100.0) && close(s.a.y, 10.0));
        let s = guide_in_view(Point::new(10.0, 10.0), 45.0, v).expect("in view");
        assert!(close(s.a.x, 0.0) && close(s.a.y, 0.0));
        assert!(close(s.b.x, 50.0) && close(s.b.y, 50.0));
        assert!(guide_in_view(Point::new(10.0, 80.0), 0.0, v).is_none());
    }
}
