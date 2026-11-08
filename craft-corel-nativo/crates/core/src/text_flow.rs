//! Paragraph text flowing around other objects (Wrap Paragraph Text) and
//! baselines snapped to the page's baseline grid.
//!
//! An object with [`Shape::wrap_text`] pushes the lines of every paragraph
//! frame beneath it in the stacking order of its page (all layers of the
//! page, bottom to top; master layers among themselves) out of its way.
//! [`Document::refresh_text_flow`] works out, for each frame, the outlines
//! of those objects in the frame's own layout space and where the page's
//! baseline grid falls, and keeps them in the frame's [`FrameFlow`]. The
//! engine refreshes it after every change, and readers after loading, so
//! the text layout (drawing, hit testing, carets, exports) only needs the
//! text object itself. The flow is never saved: it is made again from the
//! drawing.

use crate::document::{Document, Layer, Shape, ShapeKind};
use crate::geometry::{flatten, Affine, BezPath, PathEl, Point, Rect, Shape as _};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Whether text keeps away from an object's outline or from its bounding
/// box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapMode {
    /// Along the outline: each line stops where the object's shape is at
    /// its height.
    #[default]
    Contour,
    /// Along the bounding box.
    Square,
}

/// Where the text goes around an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WrapSide {
    /// On the object's left only.
    Left,
    /// On its right only.
    Right,
    /// On both sides: a line may continue past the object.
    #[default]
    Straddle,
    /// Above and below it only: no text beside it (with the bounding
    /// box).
    AboveBelow,
}

/// How paragraph text wraps around an object (its wrap style and the
/// text wrap offset).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextWrap {
    pub mode: WrapMode,
    pub side: WrapSide,
    /// The room kept free around the object, mm.
    pub offset: f64,
}

impl Default for TextWrap {
    fn default() -> Self {
        TextWrap {
            mode: WrapMode::Contour,
            side: WrapSide::Straddle,
            offset: 2.0,
        }
    }
}

impl TextWrap {
    /// The wrap styles in the order the lists show them.
    pub const STYLES: [(WrapMode, WrapSide); 7] = [
        (WrapMode::Contour, WrapSide::Left),
        (WrapMode::Contour, WrapSide::Right),
        (WrapMode::Contour, WrapSide::Straddle),
        (WrapMode::Square, WrapSide::Left),
        (WrapMode::Square, WrapSide::Right),
        (WrapMode::Square, WrapSide::Straddle),
        (WrapMode::Square, WrapSide::AboveBelow),
    ];

    pub fn is_default(&self) -> bool {
        *self == TextWrap::default()
    }

    /// The offset, kept finite and not negative.
    fn margin(&self) -> f64 {
        if self.offset.is_finite() {
            self.offset.clamp(0.0, 1000.0)
        } else {
            0.0
        }
    }
}

/// One object a frame's text keeps away from, in the frame's layout space
/// (x from the frame's left edge, y = 0 at its top, y up).
#[derive(Debug, Clone, PartialEq)]
pub struct WrapRegion {
    pub wrap: TextWrap,
    /// The bounding box of the outline.
    pub bbox: Rect,
    /// The outline as line segments.
    pub segments: Vec<(Point, Point)>,
}

impl WrapRegion {
    /// The region of an outline given in layout space; `None` for an
    /// empty or broken one.
    pub fn from_path(path: &BezPath, wrap: TextWrap) -> Option<WrapRegion> {
        let mut segments = Vec::new();
        let mut start: Option<Point> = None;
        let mut last: Option<Point> = None;
        flatten(path, 0.1, &mut |el| match el {
            PathEl::MoveTo(p) => {
                if let (Some(a), Some(b)) = (last, start) {
                    segments.push((a, b));
                }
                start = Some(p);
                last = Some(p);
            }
            PathEl::LineTo(p) => {
                if let Some(a) = last {
                    segments.push((a, p));
                }
                last = Some(p);
            }
            PathEl::ClosePath => {
                if let (Some(a), Some(b)) = (last, start) {
                    segments.push((a, b));
                }
                last = start;
            }
            _ => {}
        });
        // Open subpaths count as closed: the outline is a region.
        if let (Some(a), Some(b)) = (last, start) {
            if a != b {
                segments.push((a, b));
            }
        }
        segments.retain(|(a, b)| {
            a.x.is_finite() && a.y.is_finite() && b.x.is_finite() && b.y.is_finite()
        });
        let mut bbox: Option<Rect> = None;
        for (a, b) in &segments {
            let r = Rect::from_points(*a, *b);
            bbox = Some(bbox.map_or(r, |x| x.union(r)));
        }
        Some(WrapRegion {
            wrap,
            bbox: bbox?,
            segments,
        })
    }

    /// The x range the region takes from a line whose band runs from `y0`
    /// up to `y1`: `None` when it leaves the line alone. Infinite ends
    /// stand for the whole line on that side.
    pub fn blocked(&self, y0: f64, y1: f64) -> Option<(f64, f64)> {
        let m = self.wrap.margin();
        let (lo, hi) = (y0.min(y1) - m, y0.max(y1) + m);
        if self.bbox.y1 < lo || self.bbox.y0 > hi {
            return None;
        }
        let square = self.wrap.mode == WrapMode::Square || self.wrap.side == WrapSide::AboveBelow;
        let (a, b) = if square {
            (self.bbox.x0, self.bbox.x1)
        } else {
            // The outline's extent inside the band.
            let mut range: Option<(f64, f64)> = None;
            let mut add = |x: f64| {
                range = Some(range.map_or((x, x), |(a, b)| (a.min(x), b.max(x))));
            };
            for (p, q) in &self.segments {
                let (p, q) = if p.y <= q.y { (*p, *q) } else { (*q, *p) };
                if q.y < lo || p.y > hi {
                    continue;
                }
                let at = |y: f64| {
                    if (q.y - p.y).abs() < 1e-12 {
                        p.x
                    } else {
                        p.x + (q.x - p.x) * (y - p.y) / (q.y - p.y)
                    }
                };
                add(at(p.y.max(lo)));
                add(at(q.y.min(hi)));
            }
            range?
        };
        let (a, b) = (a - m, b + m);
        Some(match self.wrap.side {
            WrapSide::Straddle => (a, b),
            WrapSide::Left => (a, f64::INFINITY),
            WrapSide::Right => (f64::NEG_INFINITY, b),
            WrapSide::AboveBelow => (f64::NEG_INFINITY, f64::INFINITY),
        })
    }
}

/// The parts of the line `x0..x1` whose band runs from `y0` up to `y1`
/// that no region takes, left to right; parts narrower than `min_width`
/// are left out.
pub fn free_segments(
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    regions: &[WrapRegion],
    min_width: f64,
) -> Vec<(f64, f64)> {
    let mut free = vec![(x0, x1)];
    for r in regions {
        let Some((a, b)) = r.blocked(y0, y1) else {
            continue;
        };
        let mut next = Vec::new();
        for (s, e) in free {
            if b <= s || a >= e {
                next.push((s, e));
                continue;
            }
            if a > s {
                next.push((s, a));
            }
            if b < e {
                next.push((b, e));
            }
        }
        free = next;
    }
    free.retain(|(s, e)| e - s >= min_width.max(1e-6));
    free
}

/// What a paragraph frame takes from the drawing around it: the objects
/// its text wraps around and the page's baseline grid, in its layout
/// space. Worked out by [`Document::refresh_text_flow`], never saved.
#[derive(Clone, Default)]
pub struct FrameFlow {
    pub wrap: Arc<Vec<WrapRegion>>,
    /// The page's baseline grid: the y of one of its lines and the
    /// distance between lines, in layout space. `None`: the grid is
    /// measured from the frame top (a turned frame, or no page).
    pub grid: Option<(f64, f64)>,
    key: u64,
}

impl FrameFlow {
    pub fn new(wrap: Vec<WrapRegion>, grid: Option<(f64, f64)>) -> FrameFlow {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for r in &wrap {
            (r.wrap.mode as u8, r.wrap.side as u8).hash(&mut h);
            r.wrap.offset.to_bits().hash(&mut h);
            for (a, b) in &r.segments {
                for v in [a.x, a.y, b.x, b.y] {
                    v.to_bits().hash(&mut h);
                }
            }
        }
        if let Some((o, p)) = grid {
            o.to_bits().hash(&mut h);
            p.to_bits().hash(&mut h);
        }
        FrameFlow {
            wrap: Arc::new(wrap),
            grid,
            key: h.finish(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.wrap.is_empty() && self.grid.is_none()
    }
}

impl PartialEq for FrameFlow {
    fn eq(&self, other: &Self) -> bool {
        self.is_empty() && other.is_empty() || self.key == other.key && self.grid == other.grid
    }
}

impl std::fmt::Debug for FrameFlow {
    /// Short: it goes into the text outline cache key.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "FrameFlow")
        } else {
            write!(f, "FrameFlow({:x})", self.key)
        }
    }
}

/// One object of a page in stacking order, with the transform of the
/// groups it is in.
struct Item<'a> {
    shape: &'a Shape,
    parent: Affine,
    visible: bool,
}

fn collect<'a>(shapes: &'a [Shape], parent: Affine, visible: bool, out: &mut Vec<Item<'a>>) {
    for s in shapes {
        let visible = visible && s.visible;
        out.push(Item {
            shape: s,
            parent,
            visible,
        });
        if let ShapeKind::Group { children } = &s.kind {
            collect(children, parent * s.transform, visible, out);
        }
    }
}

fn layers_items(layers: &[Layer]) -> Vec<Item<'_>> {
    let mut out = Vec::new();
    for l in layers {
        collect(&l.shapes, Affine::IDENTITY, l.visible, &mut out);
    }
    out
}

/// The outline text keeps away from, on the page: a paragraph frame's
/// rectangle, any other object's outline.
fn obstacle_path(item: &Item) -> BezPath {
    let s = item.shape;
    match &s.kind {
        ShapeKind::Text {
            origin,
            frame: Some(f),
            on_path: None,
            ..
        } => {
            item.parent
                * s.transform
                * Rect::new(origin.x, origin.y, origin.x + f.width, origin.y + f.height)
                    .to_path(0.01)
        }
        _ => item.parent * s.page_path(),
    }
}

/// The flow of every paragraph frame of one stacking order, in order.
fn flows(items: &[Item], page_height: Option<f64>, baseline_start: f64) -> Vec<Option<FrameFlow>> {
    let obstacles: Vec<(usize, BezPath, crate::text_flow::TextWrap)> = items
        .iter()
        .enumerate()
        .filter(|(_, it)| it.visible && it.shape.wrap_text)
        .map(|(i, it)| (i, obstacle_path(it), it.shape.wrap_style))
        .collect();
    items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let ShapeKind::Text {
                origin,
                frame: Some(f),
                on_path: None,
                para,
                ..
            } = &it.shape.kind
            else {
                return None;
            };
            let to_page = it.parent * it.shape.transform;
            let det = to_page.determinant();
            let mut wrap = Vec::new();
            if det.is_finite() && det.abs() > 1e-12 {
                let to_layout =
                    Affine::translate((-origin.x, -(origin.y + f.height))) * to_page.inverse();
                let frame_box = Rect::new(0.0, -f.height, f.width, 0.0);
                for (j, path, style) in &obstacles {
                    if *j <= i {
                        continue;
                    }
                    let local = to_layout * path.clone();
                    let Some(region) = WrapRegion::from_path(&local, *style) else {
                        continue;
                    };
                    // Only objects over the frame (with their offset) count.
                    let m = style.margin();
                    if !region.bbox.inflate(m, m).overlaps(frame_box) {
                        continue;
                    }
                    wrap.push(region);
                }
            }
            // The page's baseline grid, for a frame that is not turned.
            let grid = match page_height {
                Some(h) if para.baseline_grid_mm > 1e-6 => {
                    let [a, b, c, d, _, ty] = to_page.as_coeffs();
                    if b.abs() < 1e-9 && c.abs() < 1e-9 && a > 1e-9 && d > 1e-9 {
                        let first = h - baseline_start.max(0.0);
                        Some((
                            (first - ty) / d - (origin.y + f.height),
                            para.baseline_grid_mm / d,
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            Some(FrameFlow::new(wrap, grid))
        })
        .collect()
}

fn store(shapes: &mut [Shape], flows: &mut std::vec::IntoIter<Option<FrameFlow>>) {
    for s in shapes {
        let flow = flows.next().flatten();
        if let ShapeKind::Text {
            frame: Some(_),
            on_path: None,
            para,
            ..
        } = &mut s.kind
        {
            let flow = flow.unwrap_or_default();
            if para.flow != flow {
                para.flow = flow;
            }
        }
        if let ShapeKind::Group { children } = &mut s.kind {
            store(children, flows);
        }
    }
}

impl Document {
    /// Work out again what every paragraph frame's text wraps around and
    /// where the page's baseline grid falls for it (see the module
    /// documentation). Cheap when nothing wraps.
    pub fn refresh_text_flow(&mut self) {
        let start = self.metadata.grid.baseline_start;
        let mut all: Vec<Vec<Option<FrameFlow>>> = Vec::new();
        for page in &self.pages {
            all.push(flows(
                &layers_items(&page.layers),
                Some(page.size.height),
                start,
            ));
        }
        let master = flows(&layers_items(&self.master), None, start);
        for (page, f) in self.pages.iter_mut().zip(all) {
            let mut it = f.into_iter();
            for l in &mut page.layers {
                store(&mut l.shapes, &mut it);
            }
        }
        let mut it = master.into_iter();
        for l in &mut self.master {
            store(&mut l.shapes, &mut it);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{ParagraphStyle, TextAlign, TextSpan};
    use crate::geometry::Size;
    use crate::id::ShapeId;

    fn square(x0: f64, y0: f64, x1: f64, y1: f64, wrap: TextWrap) -> WrapRegion {
        WrapRegion::from_path(&Rect::new(x0, y0, x1, y1).to_path(0.01), wrap).unwrap()
    }

    #[test]
    fn a_square_in_the_middle_splits_the_line_in_two() {
        let w = TextWrap {
            mode: WrapMode::Square,
            side: WrapSide::Straddle,
            offset: 1.0,
        };
        let r = square(40.0, -30.0, 60.0, -10.0, w);
        let free = free_segments(0.0, 100.0, -22.0, -18.0, std::slice::from_ref(&r), 2.0);
        assert_eq!(free, vec![(0.0, 39.0), (61.0, 100.0)]);
        // Above the object (and its offset) the line is whole.
        assert_eq!(
            free_segments(0.0, 100.0, -8.0, -5.0, &[r], 2.0),
            vec![(0.0, 100.0)]
        );
    }

    #[test]
    fn the_sides_keep_the_text_on_one_side_or_above_and_below() {
        let mk = |side| TextWrap {
            mode: WrapMode::Square,
            side,
            offset: 0.0,
        };
        let band = |side| {
            free_segments(
                0.0,
                100.0,
                -21.0,
                -19.0,
                &[square(40.0, -30.0, 60.0, -10.0, mk(side))],
                1.0,
            )
        };
        assert_eq!(band(WrapSide::Left), vec![(0.0, 40.0)]);
        assert_eq!(band(WrapSide::Right), vec![(60.0, 100.0)]);
        assert!(band(WrapSide::AboveBelow).is_empty());
    }

    #[test]
    fn contour_follows_the_outline_at_the_line_height() {
        // A triangle pointing up: narrow at the top, wide at the bottom.
        let mut tri = BezPath::new();
        tri.move_to((50.0, 0.0));
        tri.line_to((100.0, -100.0));
        tri.line_to((0.0, -100.0));
        tri.close_path();
        let w = TextWrap {
            mode: WrapMode::Contour,
            side: WrapSide::Straddle,
            offset: 0.0,
        };
        let r = WrapRegion::from_path(&tri, w).unwrap();
        let (a, b) = r.blocked(-20.0, -10.0).unwrap();
        assert!(
            (a - 40.0).abs() < 1e-6 && (b - 60.0).abs() < 1e-6,
            "{a} {b}"
        );
        let (a, b) = r.blocked(-90.0, -80.0).unwrap();
        assert!((a - 5.0).abs() < 1e-6 && (b - 95.0).abs() < 1e-6, "{a} {b}");
        // The bounding box takes the whole width at any height.
        let sq = WrapRegion::from_path(
            &tri,
            TextWrap {
                mode: WrapMode::Square,
                ..w
            },
        )
        .unwrap();
        assert_eq!(sq.blocked(-20.0, -10.0), Some((0.0, 100.0)));
    }

    #[test]
    fn frames_take_the_wrap_objects_above_them_on_their_page() {
        let mut doc = Document::new("t", Size::new(210.0, 297.0));
        let frame = Shape::new(
            ShapeId(1),
            ShapeKind::Text {
                spans: vec![TextSpan::new("text", "Sans", 12.0)],
                origin: Point::new(10.0, 100.0),
                frame: Some(Size::new(100.0, 50.0)),
                align: TextAlign::Left,
                para: ParagraphStyle::default(),
                on_path: None,
            },
        );
        let mut below = Shape::new(
            ShapeId(2),
            ShapeKind::Rect {
                rect: Rect::new(20.0, 110.0, 30.0, 120.0),
                radius: 0.0,
                corners: None,
            },
        );
        below.wrap_text = true;
        let mut above = below.clone();
        above.id = ShapeId(3);
        let mut off = above.clone();
        off.id = ShapeId(4);
        off.kind = ShapeKind::Rect {
            rect: Rect::new(150.0, 10.0, 160.0, 20.0),
            radius: 0.0,
            corners: None,
        };
        let mut not_wrapping = above.clone();
        not_wrapping.id = ShapeId(5);
        not_wrapping.wrap_text = false;
        let layer = &mut doc.pages[0].layers[0].shapes;
        layer.extend([below, frame, above, off, not_wrapping]);
        doc.refresh_text_flow();
        let flow = |doc: &Document| match &doc.pages[0].layers[0].shapes[1].kind {
            ShapeKind::Text { para, .. } => para.flow.clone(),
            _ => unreachable!(),
        };
        let f = flow(&doc);
        assert_eq!(f.wrap.len(), 1);
        // In layout space: x from the frame's left, y down from its top.
        let b = f.wrap[0].bbox;
        assert!(
            (b.x0 - 10.0).abs() < 1e-6 && (b.y1 - (120.0 - 150.0)).abs() < 1e-6,
            "{b:?}"
        );
        // Wrap switched off: nothing left.
        doc.pages[0].layers[0].shapes[2].wrap_text = false;
        doc.refresh_text_flow();
        assert!(flow(&doc).is_empty());
    }

    #[test]
    fn the_baseline_grid_is_measured_from_the_page() {
        let mut doc = Document::new("t", Size::new(210.0, 297.0));
        let para = ParagraphStyle {
            baseline_grid_mm: 5.0,
            ..ParagraphStyle::default()
        };
        doc.pages[0].layers[0].shapes.push(Shape::new(
            ShapeId(1),
            ShapeKind::Text {
                spans: vec![TextSpan::new("text", "Sans", 12.0)],
                origin: Point::new(10.0, 100.0),
                frame: Some(Size::new(100.0, 50.0)),
                align: TextAlign::Left,
                para,
                on_path: None,
            },
        ));
        doc.metadata.grid.baseline_start = 7.0;
        doc.refresh_text_flow();
        let ShapeKind::Text { para, .. } = &doc.pages[0].layers[0].shapes[0].kind else {
            unreachable!()
        };
        let (origin, pitch) = para.flow.grid.unwrap();
        // The first grid line is 7 mm under the page top (297 mm), the
        // frame top is at 150 mm.
        assert!((origin - (290.0 - 150.0)).abs() < 1e-9);
        assert_eq!(pitch, 5.0);
    }
}
