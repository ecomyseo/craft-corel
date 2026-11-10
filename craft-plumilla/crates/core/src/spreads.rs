//! Facing pages (#1972, #1660): the pages of a drawing shown and drawn
//! in spreads, two pages side by side with the spine between them. The
//! first page stands alone on the right (or starts a spread on the left),
//! then pages 2-3, 4-5 ... pair up. The pages keep their own coordinates;
//! a spread only says which pages sit beside which and where. Objects
//! drawn across the spine belong to one page and reach over onto the
//! other.

use crate::document::{Document, Layer, Page, Shape, ShapeKind};
use crate::geometry::{Affine, Point, Rect, Size};
use serde::{Deserialize, Serialize};

/// The side of a spread a page sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PageSide {
    Left,
    /// The first page alone on the right, as a book opens.
    #[default]
    Right,
}

impl PageSide {
    pub fn is_right(&self) -> bool {
        *self == PageSide::Right
    }
}

/// The pages of one spread, by index (0-based): a left page, a right
/// page, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Spread {
    pub left: Option<usize>,
    pub right: Option<usize>,
}

impl Spread {
    /// The pages of the spread, left first.
    pub fn pages(&self) -> impl Iterator<Item = usize> {
        self.left.into_iter().chain(self.right)
    }

    pub fn contains(&self, index: usize) -> bool {
        self.left == Some(index) || self.right == Some(index)
    }

    /// The other page of the spread, if `index` has one beside it.
    pub fn partner(&self, index: usize) -> Option<usize> {
        match (self.left, self.right) {
            (Some(l), Some(r)) if l == index => Some(r),
            (Some(l), Some(r)) if r == index => Some(l),
            _ => None,
        }
    }
}

/// How many slots come before the first page: one empty left slot when
/// the first page stands on the right.
fn lead(start: PageSide) -> usize {
    match start {
        PageSide::Right => 1,
        PageSide::Left => 0,
    }
}

/// The side page `index` sits on when the first page is on `start`.
pub fn side_of(index: usize, start: PageSide) -> PageSide {
    if (index + lead(start)).is_multiple_of(2) {
        PageSide::Left
    } else {
        PageSide::Right
    }
}

/// The number of the spread page `index` is in (0-based).
pub fn spread_number(index: usize, start: PageSide) -> usize {
    (index + lead(start)) / 2
}

/// The spread holding page `index` of `count` pages.
pub fn spread_of(index: usize, count: usize, start: PageSide) -> Spread {
    let n = spread_number(index, start);
    let slot = |s: usize| {
        (s >= lead(start))
            .then(|| s - lead(start))
            .filter(|i| *i < count)
    };
    Spread {
        left: slot(2 * n),
        right: slot(2 * n + 1),
    }
}

/// Every spread of `count` pages, in order. Adding or removing pages
/// reflows them: the spreads are worked out from the page order alone.
pub fn spreads(count: usize, start: PageSide) -> Vec<Spread> {
    if count == 0 {
        return Vec::new();
    }
    let last = spread_number(count - 1, start);
    (0..=last)
        .map(|n| spread_of((2 * n).saturating_sub(lead(start)), count, start))
        .collect()
}

/// Where the pages of a spread go, with the spine at `x = left width`:
/// the left page from x 0, the right page from the spine, their tops
/// lined up. Each page's bottom left corner, by page index, and the box
/// of the whole spread.
pub fn place(spread: Spread, size: impl Fn(usize) -> Size) -> (Vec<(usize, Point)>, Rect) {
    let dim = |i: Option<usize>| {
        i.map(&size)
            .map(|s| (s.width.max(0.0), s.height.max(0.0)))
            .unwrap_or((0.0, 0.0))
    };
    let (wl, hl) = dim(spread.left);
    let (wr, hr) = dim(spread.right);
    let h = hl.max(hr);
    let mut out = Vec::with_capacity(2);
    if let Some(l) = spread.left {
        out.push((l, Point::new(0.0, h - hl)));
    }
    if let Some(r) = spread.right {
        out.push((r, Point::new(wl, h - hr)));
    }
    (out, Rect::new(0.0, 0.0, wl + wr, h))
}

/// An object moved by `t`, the copies blends keep of their end objects
/// moved along.
fn shift(s: &mut Shape, t: Affine) {
    s.transform_by(t);
    carry_blend_ends(s, t);
}

fn carry_blend_ends(s: &mut Shape, t: Affine) {
    for e in s.effects.iter_mut() {
        if let crate::live::Effect::Blend { end, .. } = e {
            end.transform_by(t);
        }
    }
    match &mut s.kind {
        ShapeKind::Group { children } => children.iter_mut().for_each(|c| carry_blend_ends(c, t)),
        ShapeKind::ClipFrame { frame, contents } => {
            carry_blend_ends(frame, t);
            contents.iter_mut().for_each(|c| carry_blend_ends(c, t));
        }
        _ => {}
    }
}

impl Document {
    /// The side the first page starts on when the drawing shows facing
    /// pages, else `None`.
    pub fn facing(&self) -> Option<PageSide> {
        self.metadata
            .facing_pages
            .then_some(self.metadata.facing_start)
    }

    /// The spread holding page `index`: with facing pages off, the page
    /// alone (on the right).
    pub fn spread_at(&self, index: usize) -> Spread {
        match self.facing() {
            Some(start) => spread_of(index, self.pages.len(), start),
            None => Spread {
                left: None,
                right: (index < self.pages.len()).then_some(index),
            },
        }
    }

    /// Where the pages of `spread` sit in it (see [`place`]).
    pub fn place_spread(&self, spread: Spread) -> (Vec<(usize, Point)>, Rect) {
        place(spread, |i| {
            self.pages
                .get(i)
                .map(|p| p.size)
                .unwrap_or(Size::new(0.0, 0.0))
        })
    }

    /// The drawing with each page also carrying the parts of its facing
    /// page's objects that reach over the spine onto it (or within
    /// `margin` mm of it, for the bleed), so a page exported or printed
    /// alone shows all that lies on it. The copies go beneath the page's
    /// own layers, in a layer of their own, with ids of their own. A
    /// drawing without facing pages comes back as it is.
    pub fn with_spread_overhangs(&self, margin: f64) -> Document {
        let mut out = self.clone();
        let Some(start) = self.facing() else {
            return out;
        };
        let margin = if margin.is_finite() {
            margin.max(0.0)
        } else {
            0.0
        };
        let count = self.pages.len();
        for (index, page) in self.pages.iter().enumerate() {
            let spread = spread_of(index, count, start);
            let Some(other) = spread.partner(index) else {
                continue;
            };
            let (placed, _) = self.place_spread(spread);
            let at = |i: usize| {
                placed
                    .iter()
                    .find(|(j, _)| *j == i)
                    .map(|(_, p)| *p)
                    .unwrap_or(Point::new(0.0, 0.0))
            };
            let (mine, theirs) = (at(index), at(other));
            // The other page's coordinates in this page's.
            let t = Affine::translate((theirs.x - mine.x, theirs.y - mine.y));
            let reach = page.rect().inflate(margin, margin);
            let Some(src) = self.pages.get(other) else {
                continue;
            };
            let mut copies = Vec::new();
            for layer in src.layers.iter().filter(|l| l.visible && l.printable) {
                for s in layer.shapes.iter().filter(|s| s.visible) {
                    let b = s.visual_bounds();
                    let moved = t.transform_rect_bbox(b);
                    if moved.x0 < reach.x1
                        && moved.x1 > reach.x0
                        && moved.y0 < reach.y1
                        && moved.y1 > reach.y0
                    {
                        let mut c = s.clone();
                        shift(&mut c, t);
                        copies.push(c);
                    }
                }
            }
            if copies.is_empty() {
                continue;
            }
            let ids = out.ids_mut();
            let copies: Vec<Shape> = copies
                .iter()
                .map(|c| c.with_fresh_ids(&mut || ids.shape()))
                .collect();
            let mut layer = Layer::new(ids.layer(), "Facing page");
            layer.shapes = copies;
            if let Some(p) = out.pages.get_mut(index) {
                p.layers.insert(0, layer);
            }
        }
        out
    }

    /// The drawing with each spread of the pages at `indexes` (0-based,
    /// in order; facing pages decide the spreads, else each page stands
    /// alone) turned into one page as wide as its pages side by side:
    /// Export > Spreads. The master layers are drawn into each half (with
    /// that page's number in page number fields), as are the pages'
    /// backgrounds and guidelines; the result has no master layers.
    pub fn spreads_document(&self, indexes: &[usize]) -> Document {
        let mut out = self.clone();
        out.pages.clear();
        out.master.clear();
        out.master_guides.clear();
        let mut seen: Vec<Spread> = Vec::new();
        for &i in indexes {
            if i >= self.pages.len() {
                continue;
            }
            let full = self.spread_at(i);
            if seen.contains(&full) {
                continue;
            }
            seen.push(full);
            // Only the chosen pages of the spread.
            let spread = Spread {
                left: full.left.filter(|l| indexes.contains(l)),
                right: full.right.filter(|r| indexes.contains(r)),
            };
            let (placed, rect) = self.place_spread(spread);
            let mut layers: Vec<Layer> = Vec::new();
            let mut guides = Vec::new();
            let mut names = Vec::new();
            let mut first_id = None;
            for (index, at) in placed {
                let Some(page) = self.pages.get(index) else {
                    continue;
                };
                first_id.get_or_insert(page.id);
                names.push(if page.name.is_empty() {
                    (index + 1).to_string()
                } else {
                    page.name.clone()
                });
                let t = Affine::translate((at.x, at.y));
                let drawn = self.layers_drawn_for_page(page.id).unwrap_or_default();
                let ids = out.ids_mut();
                if let Some(bg) = &page.background {
                    let mut l = Layer::new(ids.layer(), "Background");
                    let mut s = Shape::new(
                        ids.shape(),
                        ShapeKind::Rect {
                            rect: page.rect(),
                            radius: 0.0,
                            corners: None,
                        },
                    );
                    s.fill = bg.clone();
                    s.stroke = None;
                    shift(&mut s, t);
                    l.shapes.push(s);
                    layers.push(l);
                }
                for l in drawn {
                    let mut l: Layer = l.into_owned();
                    l.id = ids.layer();
                    l.scope = crate::document::MasterScope::All;
                    l.in_front = false;
                    l.shapes = l
                        .shapes
                        .iter()
                        .map(|s| {
                            let mut c = s.with_fresh_ids(&mut || ids.shape());
                            shift(&mut c, t);
                            c
                        })
                        .collect();
                    layers.push(l);
                }
                for g in self.master_guides.iter().chain(page.guides.iter()) {
                    guides.push(g.with_line(shift_line(g.line, at.x, at.y)));
                }
            }
            let Some(id) = first_id else {
                continue;
            };
            out.pages.push(Page {
                id,
                name: names.join("-"),
                size: Size::new(rect.width(), rect.height()),
                layers,
                guides,
                background: None,
            });
        }
        if out.pages.is_empty() {
            // Never a drawing without pages.
            return self.clone();
        }
        out
    }
}

fn shift_line(line: crate::document::GuideLine, dx: f64, dy: f64) -> crate::document::GuideLine {
    use crate::document::GuideLine;
    match line {
        GuideLine::Horizontal { y } => GuideLine::Horizontal { y: y + dy },
        GuideLine::Vertical { x } => GuideLine::Vertical { x: x + dx },
        GuideLine::Angled { x, y, angle } => GuideLine::Angled {
            x: x + dx,
            y: y + dy,
            angle,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::ShapeKind;
    use crate::geometry::Rect;

    fn pages_of(spreads: &[Spread]) -> Vec<Vec<usize>> {
        spreads.iter().map(|s| s.pages().collect()).collect()
    }

    /// Starting on the right, page 1 stands alone, then 2-3, 4-5 ...;
    /// starting on the left, 1-2, 3-4 ...
    #[test]
    fn pages_pair_up_after_the_first_page() {
        let right = spreads(6, PageSide::Right);
        assert_eq!(
            pages_of(&right),
            vec![vec![0], vec![1, 2], vec![3, 4], vec![5]]
        );
        assert_eq!(
            right[0],
            Spread {
                left: None,
                right: Some(0)
            }
        );
        assert_eq!(
            right[3],
            Spread {
                left: Some(5),
                right: None
            }
        );
        assert_eq!(
            pages_of(&spreads(5, PageSide::Left)),
            vec![vec![0, 1], vec![2, 3], vec![4]]
        );
        assert!(spreads(0, PageSide::Right).is_empty());
        assert_eq!(side_of(0, PageSide::Right), PageSide::Right);
        assert_eq!(side_of(1, PageSide::Right), PageSide::Left);
        assert_eq!(side_of(0, PageSide::Left), PageSide::Left);
        assert_eq!(spread_of(2, 6, PageSide::Right).partner(2), Some(1));
        assert_eq!(spread_of(0, 6, PageSide::Right).partner(0), None);
    }

    /// Adding or removing a page reflows the spreads from the page order.
    #[test]
    fn adding_and_removing_pages_reflows_the_spreads() {
        for n in 1..12 {
            for start in [PageSide::Left, PageSide::Right] {
                let all = spreads(n, start);
                let flat: Vec<usize> = all.iter().flat_map(|s| s.pages()).collect();
                assert_eq!(flat, (0..n).collect::<Vec<_>>(), "{n} {start:?}");
                for i in 0..n {
                    assert!(all[spread_number(i, start)].contains(i));
                }
            }
        }
        let mut doc = Document::new("d", Size::new(100.0, 200.0));
        doc.metadata.facing_pages = true;
        for _ in 0..3 {
            let id = doc.ids_mut().page();
            let layer = doc.ids_mut().layer();
            doc.pages.push(Page {
                id,
                name: String::new(),
                size: Size::new(100.0, 200.0),
                layers: vec![Layer::new(layer, "L")],
                guides: Vec::new(),
                background: None,
            });
        }
        assert_eq!(doc.spread_at(3).pages().collect::<Vec<_>>(), vec![3]);
        doc.pages.remove(0);
        assert_eq!(doc.spread_at(0).pages().collect::<Vec<_>>(), vec![0]);
        assert_eq!(doc.spread_at(2).pages().collect::<Vec<_>>(), vec![1, 2]);
        doc.metadata.facing_start = PageSide::Left;
        assert_eq!(doc.spread_at(0).pages().collect::<Vec<_>>(), vec![0, 1]);
        doc.metadata.facing_pages = false;
        assert_eq!(doc.spread_at(1).pages().collect::<Vec<_>>(), vec![1]);
        assert_eq!(doc.facing(), None);
    }

    #[test]
    fn a_spread_lines_up_the_tops_with_the_spine_between() {
        let sizes = [Size::new(100.0, 200.0), Size::new(120.0, 150.0)];
        let (placed, rect) = place(
            Spread {
                left: Some(0),
                right: Some(1),
            },
            |i| sizes[i],
        );
        assert_eq!(
            placed,
            vec![(0, Point::new(0.0, 0.0)), (1, Point::new(100.0, 50.0))]
        );
        assert_eq!(rect, Rect::new(0.0, 0.0, 220.0, 200.0));
    }

    fn facing_doc() -> (Document, crate::ShapeId) {
        let mut doc = Document::new("d", Size::new(100.0, 200.0));
        doc.metadata.facing_pages = true;
        for _ in 0..2 {
            let id = doc.ids_mut().page();
            let layer = doc.ids_mut().layer();
            doc.pages.push(Page {
                id,
                name: String::new(),
                size: Size::new(100.0, 200.0),
                layers: vec![Layer::new(layer, "L")],
                guides: Vec::new(),
                background: None,
            });
        }
        // On page 2 (the left page of spread 2-3), across the spine.
        let sid = doc.ids_mut().shape();
        let mut s = Shape::new(
            sid,
            ShapeKind::Rect {
                rect: Rect::new(80.0, 10.0, 130.0, 50.0),
                radius: 0.0,
                corners: None,
            },
        );
        s.stroke = None;
        doc.pages[1].layers[0].shapes.push(s);
        (doc, sid)
    }

    /// The right page exported alone carries the part of the left
    /// page's object that reaches over the spine onto it, moved into its
    /// own coordinates, beneath its own layers and with ids of its own.
    #[test]
    fn a_page_alone_carries_what_reaches_over_the_spine() {
        let (doc, sid) = facing_doc();
        let out = doc.with_spread_overhangs(0.0);
        let right = &out.pages[2];
        assert_eq!(right.layers.len(), 2);
        let copy = &right.layers[0].shapes[0];
        assert_ne!(copy.id, sid);
        let b = copy.visual_bounds();
        assert!(
            (b.x0 + 20.0).abs() < 1e-9 && (b.x1 - 30.0).abs() < 1e-9,
            "{b:?}"
        );
        // The left page itself and page 1 (alone) are as they were.
        assert_eq!(out.pages[1], doc.pages[1]);
        assert_eq!(out.pages[0], doc.pages[0]);
        // Facing pages off: nothing added.
        let mut off = doc.clone();
        off.metadata.facing_pages = false;
        assert_eq!(off.with_spread_overhangs(0.0), off);
        // An object clear of the spine (and of the bleed) is not copied.
        let mut clear = doc.clone();
        if let Some(s) = clear.pages[1].layers[0].shapes.first_mut() {
            s.transform_by(Affine::translate((-60.0, 0.0)));
        }
        assert_eq!(clear.with_spread_overhangs(3.0).pages[2].layers.len(), 1);
        assert_eq!(clear.with_spread_overhangs(35.0).pages[2].layers.len(), 2);
    }

    /// Export > Spreads: each spread one page as wide as its two pages,
    /// the right page's objects moved past the spine, the master layers
    /// drawn into both halves; page 1 alone stays a page of its own.
    #[test]
    fn spreads_become_pages_as_wide_as_both() {
        let (mut doc, _) = facing_doc();
        let sid = doc.ids_mut().shape();
        let master = doc.ids_mut().layer();
        let mut l = Layer::master(master, "M", crate::MasterScope::All);
        l.shapes.push(Shape::new(
            sid,
            ShapeKind::Rect {
                rect: Rect::new(1.0, 1.0, 2.0, 2.0),
                radius: 0.0,
                corners: None,
            },
        ));
        doc.master.push(l);
        let sid = doc.ids_mut().shape();
        doc.pages[2].layers[0].shapes.push(Shape::new(
            sid,
            ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 20.0, 20.0),
                radius: 0.0,
                corners: None,
            },
        ));
        let out = doc.spreads_document(&[0, 1, 2]);
        assert_eq!(out.pages.len(), 2);
        assert!(out.master.is_empty());
        assert_eq!(out.pages[0].size, Size::new(100.0, 200.0));
        assert_eq!(out.pages[1].size, Size::new(200.0, 200.0));
        assert_eq!(out.pages[1].name, "2-3");
        let all: Vec<&Shape> = out.pages[1]
            .layers
            .iter()
            .flat_map(|l| l.shapes.iter())
            .collect();
        // Master copy twice, the spine-crossing rect, the right page's rect.
        assert_eq!(all.len(), 4);
        let mut ids: Vec<_> = all.iter().map(|s| s.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 4, "ids of their own");
        let moved = all
            .iter()
            .map(|s| s.visual_bounds())
            .any(|b| (b.x0 - 110.0).abs() < 1e-9 && (b.y0 - 10.0).abs() < 1e-9);
        assert!(moved, "the right page's rect past the spine");
        // Only page 3 chosen: it stands alone.
        let one = doc.spreads_document(&[2]);
        assert_eq!(one.pages.len(), 1);
        assert_eq!(one.pages[0].size, Size::new(100.0, 200.0));
        // Nothing chosen: the drawing as it is, never without pages.
        assert_eq!(doc.spreads_document(&[]).pages.len(), 3);
    }

    /// Facing pages are saved with the drawing and left out while off.
    #[test]
    fn facing_pages_round_trip_in_the_file() {
        let mut m = crate::Metadata::default();
        let off = serde_json::to_string(&m).expect("json");
        assert!(!off.contains("facing"));
        m.facing_pages = true;
        m.facing_start = PageSide::Left;
        let on = serde_json::to_string(&m).expect("json");
        assert!(on.contains("\"facing_pages\":true") && on.contains("\"facing_start\":\"left\""));
        let back: crate::Metadata = serde_json::from_str(&on).expect("read");
        assert_eq!(back, m);
        let old: crate::Metadata = serde_json::from_str("{}").expect("read");
        assert!(!old.facing_pages && old.facing_start == PageSide::Right);
    }
}
