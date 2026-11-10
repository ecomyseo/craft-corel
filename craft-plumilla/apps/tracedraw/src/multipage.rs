//! Multipage View (View > Multipage View): every page of the drawing laid
//! out in a grid on the desktop. The current page is edited as always;
//! the others are drawn beside it, and a click on one of them makes it
//! the current page without moving anything on screen.

use crate::app::App;
use crate::view::View;
use egui::{Color32, Painter, Pos2, Rect as ERect};
use tracedraw_core::document::{Guide, GuideLine};
use tracedraw_core::geometry::{Rect, Size};
use tracedraw_core::spreads::{self, PageSide};
use tracedraw_core::PageId;

/// The spacings Page > Page Spacing offers, in millimetres; Custom...
/// takes any other.
pub const SPACING_PRESETS: [(f64, &str); 6] = [
    (0.0, "menu.layout.page_spacing_0"),
    (5.0, "menu.layout.page_spacing_5"),
    (10.0, "menu.layout.page_spacing_10"),
    (20.0, "menu.layout.page_spacing_20"),
    (30.0, "menu.layout.page_spacing_30"),
    (50.0, "menu.layout.page_spacing_50"),
];

/// Columns of the grid: always this many (fewer pages fill one row), so
/// a page added goes at the next place and the pages before it stay
/// where they were (#1018).
pub const COLUMNS: usize = 4;

/// The pages' places in the current page's coordinates (mm, Y up): a grid
/// of [`COLUMNS`] columns, left to right then down, each page at the top
/// left of its cell, cells `gap` mm apart (a negative or non-finite gap
/// counts as none).
/// The current page is at the origin; an unknown `active` keeps the grid
/// where the first page is at the origin.
pub fn layout(pages: &[(PageId, Size)], active: PageId, gap: f64) -> Vec<(PageId, Rect)> {
    let gap = if gap.is_finite() { gap.max(0.0) } else { 0.0 };
    let n = pages.len();
    if n == 0 {
        return Vec::new();
    }
    let cols = n.clamp(1, COLUMNS);
    let rows = n.div_ceil(cols);
    let mut col_w = vec![0.0f64; cols];
    let mut row_h = vec![0.0f64; rows];
    for (i, (_, size)) in pages.iter().enumerate() {
        let (r, c) = (i / cols, i % cols);
        if let Some(w) = col_w.get_mut(c) {
            *w = w.max(size.width.max(0.0));
        }
        if let Some(h) = row_h.get_mut(r) {
            *h = h.max(size.height.max(0.0));
        }
    }
    let mut placed: Vec<(PageId, Rect)> = Vec::with_capacity(n);
    for (i, (id, size)) in pages.iter().enumerate() {
        let (r, c) = (i / cols, i % cols);
        let x0: f64 = col_w.iter().take(c).sum::<f64>() + c as f64 * gap;
        let top: f64 = -(row_h.iter().take(r).sum::<f64>() + r as f64 * gap);
        let w = size.width.max(0.0);
        let h = size.height.max(0.0);
        placed.push((*id, Rect::new(x0, top - h, x0 + w, top)));
    }
    let (dx, dy) = placed
        .iter()
        .find(|(id, _)| *id == active)
        .or(placed.first())
        .map(|(_, r)| (r.x0, r.y0))
        .unwrap_or((0.0, 0.0));
    placed
        .into_iter()
        .map(|(id, r)| (id, Rect::new(r.x0 - dx, r.y0 - dy, r.x1 - dx, r.y1 - dy)))
        .collect()
}

/// Spreads per row of the Multipage View with facing pages: two, so the
/// grid is as wide as the four columns of single pages.
pub const SPREAD_COLUMNS: usize = 2;

/// The pages' places with facing pages (#1972, #1660), in the current
/// page's coordinates (mm, Y up): the spreads (see
/// [`tracedraw_core::spreads`]) in a grid of [`SPREAD_COLUMNS`], left to
/// right then down, `gap` mm apart; the two pages of a spread touch at
/// the spine. Every slot is as wide as the widest page, so a page added
/// never moves the pages before it; a left page ends at the spine, a
/// right page starts there, tops lined up in each row.
pub fn facing_layout(
    pages: &[(PageId, Size)],
    active: PageId,
    gap: f64,
    start: PageSide,
) -> Vec<(PageId, Rect)> {
    let gap = if gap.is_finite() { gap.max(0.0) } else { 0.0 };
    let n = pages.len();
    if n == 0 {
        return Vec::new();
    }
    let slot = pages
        .iter()
        .map(|(_, s)| s.width.max(0.0))
        .fold(0.0f64, f64::max);
    let row_of = |i: usize| spreads::spread_number(i, start) / SPREAD_COLUMNS;
    let rows = row_of(n - 1) + 1;
    let mut row_h = vec![0.0f64; rows];
    for (i, (_, size)) in pages.iter().enumerate() {
        if let Some(h) = row_h.get_mut(row_of(i)) {
            *h = h.max(size.height.max(0.0));
        }
    }
    let mut placed: Vec<(PageId, Rect)> = Vec::with_capacity(n);
    for (i, (id, size)) in pages.iter().enumerate() {
        let k = spreads::spread_number(i, start);
        let (r, c) = (k / SPREAD_COLUMNS, k % SPREAD_COLUMNS);
        let spine = c as f64 * (2.0 * slot + gap) + slot;
        let top: f64 = -(row_h.iter().take(r).sum::<f64>() + r as f64 * gap);
        let w = size.width.max(0.0);
        let h = size.height.max(0.0);
        let x0 = match spreads::side_of(i, start) {
            PageSide::Left => spine - w,
            PageSide::Right => spine,
        };
        placed.push((*id, Rect::new(x0, top - h, x0 + w, top)));
    }
    let (dx, dy) = placed
        .iter()
        .find(|(id, _)| *id == active)
        .or(placed.first())
        .map(|(_, r)| (r.x0, r.y0))
        .unwrap_or((0.0, 0.0));
    placed
        .into_iter()
        .map(|(id, r)| (id, Rect::new(r.x0 - dx, r.y0 - dy, r.x1 - dx, r.y1 - dy)))
        .collect()
}

/// The page under a page-space point, if any.
pub fn page_at(placed: &[(PageId, Rect)], p: tracedraw_core::geometry::Point) -> Option<PageId> {
    placed
        .iter()
        .find(|(_, r)| p.x >= r.x0 && p.x <= r.x1 && p.y >= r.y0 && p.y <= r.y1)
        .map(|(id, _)| *id)
}

/// Everything the grid covers, for Zoom to Fit in this view.
pub fn bounds(placed: &[(PageId, Rect)]) -> Option<Rect> {
    let mut it = placed.iter().map(|(_, r)| *r);
    let first = it.next()?;
    Some(it.fold(first, |a, b| a.union(b)))
}

impl App {
    /// The grid of pages around the current one (in spreads with facing
    /// pages).
    pub fn multipage_layout(&self) -> Vec<(PageId, Rect)> {
        let doc = self.doc();
        let pages: Vec<(PageId, Size)> = doc.pages.iter().map(|p| (p.id, p.size)).collect();
        match doc.facing() {
            Some(start) => facing_layout(&pages, self.page, doc.metadata.page_spacing(), start),
            None => layout(&pages, self.page, doc.metadata.page_spacing()),
        }
    }

    /// The pages on the desktop and where, in the current page's
    /// coordinates: every page in the Multipage View, else the current
    /// page's spread with facing pages (#1972), else the current page.
    pub fn shown_pages(&self) -> Vec<(PageId, Rect)> {
        if self.multipage {
            return self.multipage_layout();
        }
        let doc = self.doc();
        if doc.facing().is_some() {
            let spread = doc.spread_at(self.page_index());
            let ids: Vec<PageId> = spread
                .pages()
                .filter_map(|i| doc.pages.get(i).map(|p| p.id))
                .collect();
            if ids.len() > 1 {
                return self
                    .multipage_layout()
                    .into_iter()
                    .filter(|(id, _)| ids.contains(id))
                    .collect();
            }
        }
        vec![(self.page, self.page_rect())]
    }

    /// Pages other than the current one are on the desktop: the
    /// Multipage View, or the current page's spread with facing pages.
    pub fn shows_other_pages(&self) -> bool {
        self.multipage || self.shown_pages().len() > 1
    }

    /// The box of the spread page `page` is in, where it is shown (the
    /// page's own box without facing pages); `None` when the page is not
    /// shown.
    pub fn shown_spread_rect(&self, page: PageId) -> Option<Rect> {
        let shown = self.shown_pages();
        let doc = self.doc();
        let index = doc.pages.iter().position(|p| p.id == page)?;
        let spread = doc.spread_at(index);
        let ids: Vec<PageId> = spread
            .pages()
            .filter_map(|i| doc.pages.get(i).map(|p| p.id))
            .collect();
        bounds(
            &shown
                .into_iter()
                .filter(|(id, _)| ids.contains(id) || *id == page)
                .collect::<Vec<_>>(),
        )
    }

    /// Layout > Facing Pages and Document Options > Layout: show the
    /// pages in spreads (`on`), the first page on `start`, as one undo
    /// step (a document property saved with the drawing). The view fits
    /// the spread anew. False when nothing changed.
    pub fn set_facing_pages(&mut self, on: bool, start: PageSide) -> bool {
        if !self.has_document() {
            return false;
        }
        let m = &self.doc().metadata;
        if m.facing_pages == on && m.facing_start == start {
            return false;
        }
        let mut metadata = m.clone();
        metadata.facing_pages = on;
        metadata.facing_start = start;
        self.run(tracedraw_core::Command::SetMetadata { metadata });
        self.page_rasters.borrow_mut().clear();
        self.fit_pending = true;
        true
    }

    /// The space between the pages of the Multipage View, mm.
    pub fn page_spacing(&self) -> f64 {
        self.doc().metadata.page_spacing()
    }

    /// Page > Page Spacing and Document Setup > Layout: set the space
    /// between the pages of the Multipage View, in mm, as one undo step
    /// (a document property, saved with the drawing). The current page
    /// stays where it is on screen. False when nothing changed.
    pub fn set_page_spacing(&mut self, mm: f64) -> bool {
        if !self.has_document() || !mm.is_finite() {
            return false;
        }
        let mm = mm.clamp(0.0, tracedraw_core::document::MAX_PAGE_SPACING);
        if (mm - self.page_spacing()).abs() < 1e-9 {
            return false;
        }
        let mut metadata = self.doc().metadata.clone();
        metadata.page_spacing = Some(mm);
        self.run(tracedraw_core::Command::SetMetadata { metadata });
        true
    }

    /// Turn the Multipage View on or off; turning it on shows every page.
    pub fn set_multipage(&mut self, on: bool) {
        self.multipage = on;
        if !on {
            self.page_rasters.borrow_mut().clear();
            self.page_snapshot.borrow_mut().take();
        }
        if on {
            self.page_sorter = false;
        }
        self.fit_pending = true;
    }

    /// Make `page` the current page and keep every page where it is on
    /// screen: the view moves by the page's place in the grid.
    pub fn multipage_activate(&mut self, page: PageId) {
        if page == self.page {
            return;
        }
        let Some(r) = self
            .shown_pages()
            .into_iter()
            .find(|(id, _)| *id == page)
            .map(|(_, r)| r)
        else {
            return;
        };
        self.view.origin = self
            .view
            .to_screen(tracedraw_core::geometry::Point::new(r.x0, r.y0));
        self.page = page;
        self.selection.clear();
        self.focus.clear();
        self.picked_layer = None;
    }

    /// In the Multipage View and a spread of facing pages, a press on
    /// another page makes it current before the tool sees the press.
    /// True when the page changed. With facing pages an object reaching
    /// over the spine is picked where it lies (#1972): a press on an
    /// object of the current page keeps the page, and a press on the
    /// current page where only an object of another page of the spread
    /// lies makes that page current.
    pub fn multipage_press(&mut self, s: Pos2) -> bool {
        if !self.shows_other_pages() {
            return false;
        }
        let p = self.view.to_page(s);
        let shown = self.shown_pages();
        let facing = self.doc().facing().is_some();
        let under = page_at(&shown, p);
        if facing && self.hit_test(p).is_some() {
            return false;
        }
        let target = match under {
            Some(id) if id != self.page => Some(id),
            _ if facing => self.spread_object_page_at(&shown, p),
            // Off every page, or on the current page where it has no
            // object: an object of another page lying there (one left on
            // the desktop beside its page) makes its page current, so the
            // press then picks it instead of doing nothing (#2055).
            _ if self.multipage && self.hit_test(p).is_none() => {
                self.shown_object_page_at(&shown, p)
            }
            _ => None,
        };
        match target {
            Some(id) => {
                self.multipage_activate(id);
                true
            }
            None => false,
        }
    }

    /// Another shown page whose visible object lies at `p` (the current
    /// page's coordinates), going by the objects' boxes; the pages in
    /// their drawing order, so the frontmost page's object wins (#2055).
    fn shown_object_page_at(
        &self,
        shown: &[(PageId, Rect)],
        p: tracedraw_core::geometry::Point,
    ) -> Option<PageId> {
        let doc = self.doc();
        shown
            .iter()
            .rev()
            .filter(|(id, _)| *id != self.page)
            .filter_map(|(id, r)| doc.page(*id).ok().map(|page| (page, *r)))
            .find(|(page, r)| {
                let q = tracedraw_core::geometry::Point::new(p.x - r.x0, p.y - r.y0);
                page.layers
                    .iter()
                    .filter(|l| l.visible && !l.locked)
                    .flat_map(|l| l.shapes.iter())
                    .filter(|sh| sh.visible)
                    .any(|sh| {
                        let b = sh.visual_bounds();
                        q.x >= b.x0 && q.x <= b.x1 && q.y >= b.y0 && q.y <= b.y1
                    })
            })
            .map(|(page, _)| page.id)
    }

    /// Another page of the current spread whose visible object lies at
    /// `p` (the current page's coordinates), going by the objects' boxes.
    fn spread_object_page_at(
        &self,
        shown: &[(PageId, Rect)],
        p: tracedraw_core::geometry::Point,
    ) -> Option<PageId> {
        let doc = self.doc();
        let spread = doc.spread_at(self.page_index());
        spread
            .pages()
            .filter_map(|i| doc.pages.get(i))
            .filter(|page| page.id != self.page)
            .find(|page| {
                let Some((_, r)) = shown.iter().find(|(id, _)| *id == page.id) else {
                    return false;
                };
                let q = tracedraw_core::geometry::Point::new(p.x - r.x0, p.y - r.y0);
                page.layers
                    .iter()
                    .filter(|l| l.visible && !l.locked)
                    .flat_map(|l| l.shapes.iter())
                    .filter(|sh| sh.visible)
                    .any(|sh| {
                        let b = sh.visual_bounds();
                        q.x >= b.x0 && q.x <= b.x1 && q.y >= b.y0 && q.y <= b.y1
                    })
            })
            .map(|page| page.id)
    }

    /// The guidelines of the other pages in the Multipage View, each moved
    /// to where its page sits in the grid (the current page's
    /// coordinates) and given the page's box there, so they can be drawn
    /// within their page (#1448); the facing page's in a spread of facing
    /// pages too (#1972). None when no other page is shown; the current
    /// page's guidelines are drawn as always.
    pub fn other_page_guides(&self) -> Vec<(Guide, Rect)> {
        if !self.shows_other_pages() {
            return Vec::new();
        }
        let doc = self.doc();
        let mut out = Vec::new();
        for (id, r) in self.shown_pages() {
            if id == self.page {
                continue;
            }
            let Some(page) = doc.pages.iter().find(|p| p.id == id) else {
                continue;
            };
            // The page's own guidelines and the master page's (#1928).
            for g in doc.master_guides.iter().chain(page.guides.iter()) {
                let mut g = *g;
                g.line = shifted(g.line, r.x0, r.y0);
                out.push((g, r));
            }
        }
        out
    }

    /// What Zoom to Fit and a new view fit: the page, or every page in the
    /// Multipage View.
    pub fn fit_rect(&self) -> Rect {
        if self.shows_other_pages() {
            if let Some(b) = bounds(&self.shown_pages()) {
                return b;
            }
        }
        self.page_rect()
    }
}

/// A guideline moved by `(dx, dy)`: a page's guideline where the page
/// sits in the grid.
fn shifted(line: GuideLine, dx: f64, dy: f64) -> GuideLine {
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

/// How long one frame may spend rendering the page and the pages beside
/// it, seconds; the pages left wait for the next frames, showing their
/// last picture meanwhile (#903). One render a frame always runs.
pub const RENDER_BUDGET: f64 = 0.010;

/// The part of a page to render, on screen: the whole paper while it
/// covers no more of the screen than the canvas does, so the picture
/// stays whole while the view moves; else only its visible part.
pub fn render_rect(paper: ERect, canvas: ERect) -> ERect {
    let clip = paper.intersect(canvas);
    let area = |r: ERect| r.width().max(0.0) * r.height().max(0.0);
    if area(paper) <= area(canvas) {
        paper
    } else {
        clip
    }
}

/// The pages other than the current one on the desktop, on screen
/// (rounded to pixels), with their index in the drawing.
fn other_papers(app: &App, rect: ERect) -> Vec<(usize, PageId, Rect, ERect)> {
    let doc = app.doc();
    app.shown_pages()
        .into_iter()
        .filter_map(|(id, r)| {
            let index = doc.pages.iter().position(|p| p.id == id)?;
            let paper = app.view.rect_to_screen(r);
            let paper = ERect::from_min_max(paper.min.round(), paper.max.round());
            paper
                .expand(30.0)
                .intersects(rect)
                .then_some((index, id, r, paper))
        })
        .collect()
}

/// Paints the shadows and the paper of the pages other than the current
/// one (the Multipage View, a spread of facing pages): all the shadows
/// first, then the papers, so no shadow falls on a page beside it.
/// Painted before the current page; their objects come after it (see
/// [`draw_other_pages`]), so an object of theirs that reaches over the
/// spine shows on the current page too (#1972).
pub fn draw_other_papers(app: &App, painter: &Painter, rect: ERect) {
    let papers: Vec<ERect> = other_papers(app, rect)
        .into_iter()
        .filter(|(_, id, _, _)| *id != app.page)
        .map(|(_, _, _, paper)| paper)
        .collect();
    if app.show_page_shadow {
        for paper in &papers {
            painter.rect_filled(
                paper
                    .expand(1.0)
                    .translate(crate::theme::Tokens::PAGE_SHADOW_OFFSET),
                0.0,
                crate::theme::Tokens::page_shadow(),
            );
        }
    }
    for paper in &papers {
        painter.rect_filled(*paper, 0.0, crate::theme::Tokens::PAGE);
    }
}

/// Paints the objects of the pages other than the current one (each
/// page kept as its own texture), their borders and the page names
/// below. While the view moves each page shows its last picture scaled
/// into place, and the pages render anew once it rests, as many a frame
/// as [`RENDER_BUDGET`] allows, counted from `started` (before the
/// current page's render; `rendered` when that one rendered) (#902,
/// #903). With facing pages a page renders over its whole spread, so
/// what it holds across the spine shows (#1972).
pub fn draw_other_pages(
    app: &App,
    painter: &Painter,
    rect: ERect,
    started: web_time::Instant,
    rendered: bool,
) {
    let placed = other_papers(app, rect);
    let doc = app.doc();
    let ctx = painter.ctx();
    let facing = doc.facing().is_some();
    let mut rasters = app.page_rasters.borrow_mut();
    rasters.retain(|id, _| doc.pages.iter().any(|p| p.id == *id));
    let mut rendered = rendered;
    for (index, id, _, paper) in placed.iter().copied() {
        if id != app.page {
            // The part of the desktop the page's objects show on: the
            // page, or its spread.
            let area = if facing {
                app.shown_spread_rect(id)
                    .map(|r| {
                        let r = app.view.rect_to_screen(r);
                        ERect::from_min_max(r.min.round(), r.max.round())
                    })
                    .unwrap_or(paper)
            } else {
                paper
            };
            let clip = area.intersect(rect);
            if clip.width() >= 1.0 && clip.height() >= 1.0 {
                let view = View {
                    zoom: app.view.zoom,
                    origin: paper.left_bottom(),
                };
                let raster = rasters.entry(id).or_default();
                raster.follows_zoom = true;
                raster.defer = rendered && started.elapsed().as_secs_f64() > RENDER_BUDGET;
                // A render on its thread takes the drawing shared by all
                // the pages, copied once a change (the browser build draws
                // on the one thread).
                if cfg!(not(target_arch = "wasm32")) && !raster.defer && raster.has_picture() {
                    let revision = app.engine.revision();
                    let mut snap = app.page_snapshot.borrow_mut();
                    if snap.as_ref().is_none_or(|(r, _)| *r != revision) {
                        *snap = Some((revision, std::sync::Arc::new(doc.clone())));
                    }
                    raster.snapshot = snap.as_ref().map(|(_, d)| d.clone());
                }
                let before = raster.render_count();
                let shown = raster.texture(
                    ctx,
                    doc,
                    id,
                    &view,
                    render_rect(area, rect),
                    None,
                    app.engine.revision(),
                    app.wireframe,
                    app.simulate_overprints,
                    app.rasterize_complex_effects,
                    app.proof_colors,
                    None,
                    None,
                );
                rendered |= raster.render_count() != before;
                raster.defer = false;
                raster.snapshot = None;
                // The picture where its part of the page is, cut to the
                // page's visible part: an older picture shown while the
                // view moves covers another part of the page than now.
                if let Some((tex, at)) = shown {
                    painter.with_clip_rect(clip).image(
                        tex,
                        at,
                        ERect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            }
            if app.show_page_border {
                painter.rect_stroke(
                    paper,
                    0.0,
                    egui::Stroke::new(1.0, crate::theme::Tokens::page_border()),
                    egui::StrokeKind::Outside,
                );
            }
        }
        let name = doc
            .pages
            .get(index)
            .map(|p| p.name.clone())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| crate::i18n::trf("doc.page_n", &[("n", &(index + 1).to_string())]));
        let color = if id == app.page {
            crate::theme::Tokens::text()
        } else {
            crate::theme::Tokens::text_dim()
        };
        painter.text(
            paper.left_bottom() + egui::vec2(0.0, 6.0),
            egui::Align2::LEFT_TOP,
            format!("{}: {name}", index + 1),
            egui::FontId::proportional(12.0),
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The space between pages of a drawing that sets none.
    const GAP_MM: f64 = tracedraw_core::document::DEFAULT_PAGE_SPACING;
    use tracedraw_core::geometry::Point;

    fn pages(n: usize) -> Vec<(PageId, Size)> {
        (0..n)
            .map(|i| (PageId(i as u64 + 1), Size::new(100.0, 50.0)))
            .collect()
    }

    #[test]
    fn pages_sit_in_a_grid_with_the_current_one_at_the_origin() {
        let ps = pages(6);
        let placed = layout(&ps, PageId(1), GAP_MM);
        // Four columns, 30 mm apart, then the next row going down.
        assert_eq!(placed[0].1, Rect::new(0.0, 0.0, 100.0, 50.0));
        assert_eq!(placed[1].1, Rect::new(130.0, 0.0, 230.0, 50.0));
        assert_eq!(placed[3].1, Rect::new(390.0, 0.0, 490.0, 50.0));
        assert_eq!(placed[4].1, Rect::new(0.0, -80.0, 100.0, -30.0));
        // Another page current: the same grid, moved so it is at the origin.
        let placed = layout(&ps, PageId(6), GAP_MM);
        assert_eq!(placed[5].1, Rect::new(0.0, 0.0, 100.0, 50.0));
        assert_eq!(placed[0].1, Rect::new(-130.0, 80.0, -30.0, 130.0));
        assert_eq!(page_at(&placed, Point::new(-50.0, 100.0)), Some(PageId(1)));
        assert_eq!(page_at(&placed, Point::new(-20.0, 100.0)), None);
        assert_eq!(bounds(&placed), Some(Rect::new(-130.0, 0.0, 360.0, 130.0)));
        assert!(layout(&[], PageId(1), GAP_MM).is_empty());
    }

    /// Pages added one by one go at the next place of a grid of four
    /// columns; the pages before never move (#1018). The grid used to have
    /// as many columns as the square root of the page count, so the 4th
    /// and the 10th page rearranged every page.
    #[test]
    fn adding_pages_never_moves_the_pages_before() {
        for n in 1..=13 {
            let before = layout(&pages(n), PageId(1), GAP_MM);
            let after = layout(&pages(n + 1), PageId(1), GAP_MM);
            assert_eq!(after[..n], before[..], "page {} added", n + 1);
            let (r, c) = (n / COLUMNS, n % COLUMNS);
            let x0 = c as f64 * 130.0;
            let y0 = -(r as f64) * 80.0;
            assert_eq!(after[n].1, Rect::new(x0, y0, x0 + 100.0, y0 + 50.0));
        }
    }

    #[test]
    fn pages_of_other_sizes_take_the_widest_and_tallest_cell() {
        let ps = vec![
            (PageId(1), Size::new(100.0, 50.0)),
            (PageId(2), Size::new(40.0, 80.0)),
            (PageId(3), Size::new(150.0, 30.0)),
        ];
        let placed = layout(&ps, PageId(1), GAP_MM);
        // One row: each column as wide as its page, every top lined up
        // with page 1's; the next row (page 5 on) would start below the
        // tallest, page 2.
        assert_eq!(placed[1].1, Rect::new(130.0, -30.0, 170.0, 50.0));
        assert_eq!(placed[2].1, Rect::new(200.0, 20.0, 350.0, 50.0));
        let mut five = ps.clone();
        five.push((PageId(4), Size::new(10.0, 10.0)));
        five.push((PageId(5), Size::new(150.0, 30.0)));
        let placed = layout(&five, PageId(1), GAP_MM);
        // Column 0 is now as wide as page 5, which sits a row below page
        // 2's bottom.
        assert_eq!(placed[1].1, Rect::new(180.0, -30.0, 220.0, 50.0));
        assert_eq!(placed[4].1, Rect::new(0.0, -90.0, 150.0, -60.0));
    }

    #[test]
    fn a_small_page_renders_whole_and_a_large_one_its_visible_part() {
        let canvas = ERect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        // Small and partly off the canvas: the whole paper, so the picture
        // stays whole while the view moves.
        let small = ERect::from_min_size(egui::pos2(-50.0, 100.0), egui::vec2(200.0, 150.0));
        assert_eq!(render_rect(small, canvas), small);
        // Larger than the canvas: only what shows.
        let large = ERect::from_min_size(egui::pos2(-500.0, -500.0), egui::vec2(2000.0, 1500.0));
        assert_eq!(render_rect(large, canvas), canvas);
    }

    /// One frame of the whole window at egui time `time`.
    fn frame_at(ctx: &egui::Context, app: &mut App, time: f64) {
        let input = egui::RawInput {
            screen_rect: Some(ERect::from_min_size(Pos2::ZERO, egui::vec2(1600.0, 1000.0))),
            time: Some(time),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| crate::ui::root(app, ui));
        out.textures_delta.clear();
    }

    fn renders(app: &App) -> u64 {
        app.page_rasters
            .borrow()
            .values()
            .map(|r| r.render_count())
            .sum()
    }

    /// While the view zooms, the pages beside the current one show their
    /// last pictures scaled into place and render nothing; once the view
    /// rests they render anew (#902, #903).
    #[test]
    fn other_pages_render_once_the_zoom_rests() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.show_welcome = false;
        for _ in 0..3 {
            app.add_page();
            app.new_shape(tracedraw_core::ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            });
        }
        app.page = app.doc().pages[0].id;
        app.set_multipage(true);
        let mut t = 0.0;
        for _ in 0..20 {
            t += 0.02;
            frame_at(&ctx, &mut app, t);
        }
        let at_rest = renders(&app);
        assert!(at_rest >= 3, "every other page rendered: {at_rest}");
        // Still at rest: nothing rendered again.
        t += 0.02;
        frame_at(&ctx, &mut app, t);
        assert_eq!(renders(&app), at_rest);
        // A zoom, frame by frame: nothing rendered.
        for _ in 0..5 {
            app.view.zoom_at(egui::pos2(800.0, 500.0), 0.9);
            t += 0.016;
            frame_at(&ctx, &mut app, t);
        }
        assert_eq!(renders(&app), at_rest, "no render while zooming");
        // At rest: each page rendered again, one render on a thread at a
        // time in the whole program (other tests' renders may hold the
        // turn for a moment).
        for _ in 0..400 {
            t += 0.02;
            frame_at(&ctx, &mut app, t);
            if renders(&app) >= at_rest + 3 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(renders(&app) >= at_rest + 3);
    }

    /// The other pages' guidelines show in the Multipage View, each
    /// where its page sits in the grid and within that page; they used
    /// to show only while their page was the current one (#1448).
    #[test]
    fn the_other_pages_guidelines_show_where_their_pages_sit() {
        let mut app = App::headless();
        app.add_page();
        let first = app.doc().pages[0].id;
        let second = app.doc().pages[1].id;
        app.page = second;
        app.add_guide(Guide::horizontal(10.0));
        app.add_guide(Guide::vertical(20.0));
        app.add_guide(
            GuideLine::Angled {
                x: 5.0,
                y: 6.0,
                angle: 30.0,
            }
            .into(),
        );
        app.page = first;
        app.add_guide(Guide::horizontal(40.0));
        // Off: nothing beyond the current page's own guidelines.
        app.set_multipage(false);
        assert!(app.other_page_guides().is_empty());
        app.set_multipage(true);
        let r = app
            .multipage_layout()
            .into_iter()
            .find(|(id, _)| *id == second)
            .map(|(_, r)| r)
            .expect("the second page's place");
        assert!(r.x0 > 0.0, "beside the first page: {r:?}");
        let shown = app.other_page_guides();
        assert_eq!(shown.len(), 3, "the second page's three, not the first's");
        assert!(shown.iter().all(|(_, page)| *page == r));
        let lines: Vec<GuideLine> = shown.iter().map(|(g, _)| g.line).collect();
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(matches!(lines[0], GuideLine::Horizontal { y } if near(y, 10.0 + r.y0)));
        assert!(matches!(lines[1], GuideLine::Vertical { x } if near(x, 20.0 + r.x0)));
        assert!(matches!(
            lines[2],
            GuideLine::Angled { x, y, angle }
                if near(x, 5.0 + r.x0) && near(y, 6.0 + r.y0) && near(angle, 30.0)
        ));
        // The second page current: the first page's guideline, where
        // that page now sits (to the left of the origin).
        app.multipage_activate(second);
        let shown = app.other_page_guides();
        assert_eq!(shown.len(), 1);
        assert!(shown[0].1.x0 < 0.0);
        assert!(
            matches!(shown[0].0.line, GuideLine::Horizontal { y } if near(y, 40.0 + shown[0].1.y0))
        );
        // A frame draws them within their page without trouble.
        let ctx = crate::theme::ui_context();
        app.show_welcome = false;
        app.show_guides = true;
        frame_at(&ctx, &mut app, 0.1);
        frame_at(&ctx, &mut app, 0.2);
    }

    /// Facing pages starting on the right: page 1 alone on the right of
    /// its spread, then 2-3, 4-5 touching at the spine, two spreads a
    /// row; a page added never moves the pages before it (#1972).
    #[test]
    fn facing_pages_sit_in_spreads_touching_at_the_spine() {
        let ps = pages(6);
        let placed = facing_layout(&ps, PageId(1), GAP_MM, PageSide::Right);
        // Page 1 on the right of the first spread, at the origin.
        assert_eq!(placed[0].1, Rect::new(0.0, 0.0, 100.0, 50.0));
        // Pages 2 and 3 form the second spread, touching.
        assert_eq!(placed[1].1, Rect::new(130.0, 0.0, 230.0, 50.0));
        assert_eq!(placed[2].1, Rect::new(230.0, 0.0, 330.0, 50.0));
        // Pages 4-5 start the next row; page 6 on the left of a spread.
        assert_eq!(placed[3].1, Rect::new(-100.0, -80.0, 0.0, -30.0));
        assert_eq!(placed[4].1, Rect::new(0.0, -80.0, 100.0, -30.0));
        assert_eq!(placed[5].1, Rect::new(130.0, -80.0, 230.0, -30.0));
        for n in 1..=9 {
            let before = facing_layout(&pages(n), PageId(1), GAP_MM, PageSide::Right);
            let after = facing_layout(&pages(n + 1), PageId(1), GAP_MM, PageSide::Right);
            assert_eq!(after[..n], before[..], "page {} added", n + 1);
        }
        // Starting on the left: 1-2 is the first spread.
        let left = facing_layout(&ps, PageId(1), GAP_MM, PageSide::Left);
        assert_eq!(left[0].1, Rect::new(0.0, 0.0, 100.0, 50.0));
        assert_eq!(left[1].1, Rect::new(100.0, 0.0, 200.0, 50.0));
        assert_eq!(left[2].1, Rect::new(230.0, 0.0, 330.0, 50.0));
        assert!(facing_layout(&[], PageId(1), GAP_MM, PageSide::Left).is_empty());
    }

    /// An app with four pages, a rectangle on page 2 reaching over the
    /// spine onto page 3, page 2 current.
    fn facing_app() -> App {
        let mut app = App::headless();
        app.show_welcome = false;
        for _ in 0..3 {
            app.add_page();
        }
        app.page = app.doc().pages[1].id;
        let w = app.doc().pages[1].size.width;
        app.new_shape(tracedraw_core::ShapeKind::Rect {
            rect: Rect::new(w - 20.0, 10.0, w + 30.0, 40.0),
            radius: 0.0,
            corners: None,
        });
        app
    }

    /// Facing pages on: the desktop shows the current page's spread, the
    /// facing page beside it at the spine; Zoom to Fit takes the spread;
    /// switching it is one undo step, saved with the drawing (#1972).
    #[test]
    fn facing_pages_show_the_spread_beside_the_current_page() {
        let mut app = facing_app();
        let (p2, p3) = (app.doc().pages[1].id, app.doc().pages[2].id);
        let w = app.doc().pages[1].size.width;
        assert_eq!(app.shown_pages().len(), 1);
        assert!(!app.shows_other_pages());
        let undo = app.engine.undo_label();
        assert!(app.set_facing_pages(true, PageSide::Right));
        assert!(!app.set_facing_pages(true, PageSide::Right), "no change");
        assert!(app.doc().metadata.facing_pages);
        let shown = app.shown_pages();
        assert_eq!(shown.len(), 2);
        assert!(app.shows_other_pages());
        assert_eq!(shown[0], (p2, app.page_rect()));
        assert_eq!(shown[1].0, p3);
        assert!((shown[1].1.x0 - w).abs() < 1e-9, "page 3 at the spine");
        assert!((app.fit_rect().width() - 2.0 * w).abs() < 1e-9);
        assert_eq!(app.shown_spread_rect(p3), Some(app.fit_rect()));
        // Page 1 stands alone.
        app.goto_page(0);
        assert_eq!(app.shown_pages().len(), 1);
        // Starting on the left, page 1 pairs with page 2.
        assert!(app.set_facing_pages(true, PageSide::Left));
        assert_eq!(app.shown_pages().len(), 2);
        assert!(app.shown_pages()[1].1.x0 > 0.0);
        // Back to where it was in one undo step each.
        app.undo();
        app.undo();
        assert!(!app.doc().metadata.facing_pages);
        assert_eq!(app.engine.undo_label(), undo);
    }

    /// The other page of the spread becomes current where it is, from
    /// the page tabs and navigator or a press on it, the view not moving;
    /// a press on the current page's object where it reaches over the
    /// spine keeps the page, so the object can be picked there; adding
    /// a page reflows the spreads (#1972).
    #[test]
    fn the_facing_page_becomes_current_where_it_is() {
        let mut app = facing_app();
        app.set_facing_pages(true, PageSide::Right);
        app.fit_pending = false;
        let (p2, p3) = (app.doc().pages[1].id, app.doc().pages[2].id);
        let w = app.doc().pages[1].size.width;
        let corner = app.view.to_screen(Point::new(w, 0.0));
        // On the rectangle, over page 3: page 2 stays current.
        let on_rect = app.view.to_screen(Point::new(w + 10.0, 20.0));
        assert!(!app.multipage_press(on_rect));
        assert_eq!(app.page, p2);
        // On page 3 away from it: page 3 current, nothing moved.
        let on_p3 = app.view.to_screen(Point::new(w + 50.0, 100.0));
        assert!(app.multipage_press(on_p3));
        assert_eq!(app.page, p3);
        let now = app.view.to_screen(Point::new(0.0, 0.0));
        assert!((now - corner).length() < 1e-3);
        // Page 2's rectangle over page 3 (now current): page 2 again.
        let p = app.view.to_page(on_rect);
        assert!(p.x > 0.0 && app.hit_test(p).is_none());
        assert!(app.multipage_press(on_rect));
        assert_eq!(app.page, p2);
        // The navigator to page 3: the same place, no fit.
        app.goto_page(2);
        assert_eq!(app.page, p3);
        assert!(!app.fit_pending);
        // To page 4 (another spread): fitted to that spread.
        app.goto_page(3);
        assert!(app.fit_pending);
        assert_eq!(app.shown_pages().len(), 1, "page 4 alone on the left");
        app.add_page();
        assert_eq!(app.doc().pages.len(), 5);
        app.page = app.doc().pages[3].id;
        assert_eq!(app.shown_pages().len(), 2, "pages 4 and 5 now");
        // Off: nothing beside the page, presses do nothing.
        app.set_facing_pages(false, PageSide::Right);
        assert!(!app.multipage_press(on_p3));
        assert_eq!(app.fit_rect(), app.page_rect());
    }

    /// The facing page's guidelines show within it, snapping finds its
    /// objects, and frames draw the spread (objects across the spine,
    /// the bleed around the spread, Mask Outside Page) without trouble.
    #[test]
    fn the_spread_keeps_guides_snapping_and_drawing() {
        let ctx = crate::theme::ui_context();
        let mut app = facing_app();
        let p3 = app.doc().pages[2].id;
        app.page = p3;
        app.add_guide(Guide::vertical(20.0));
        app.page = app.doc().pages[1].id;
        assert!(app.other_page_guides().is_empty());
        app.set_facing_pages(true, PageSide::Right);
        let w = app.doc().pages[1].size.width;
        let shown = app.other_page_guides();
        assert_eq!(shown.len(), 1);
        assert!(
            matches!(shown[0].0.line, GuideLine::Vertical { x } if (x - (w + 20.0)).abs() < 1e-9)
        );
        app.show_guides = true;
        app.show_bleed = true;
        let mut m = app.doc().metadata.clone();
        m.bleed = 3.0;
        app.run(tracedraw_core::Command::SetMetadata { metadata: m });
        let mut t = 0.0;
        for _ in 0..10 {
            t += 0.05;
            frame_at(&ctx, &mut app, t);
        }
        app.mask_outside_page = true;
        frame_at(&ctx, &mut app, t + 0.05);
        app.set_multipage(true);
        assert_eq!(app.multipage_layout().len(), 4);
        frame_at(&ctx, &mut app, t + 0.1);
        frame_at(&ctx, &mut app, t + 0.15);
    }

    #[test]
    fn the_cells_are_as_far_apart_as_the_spacing() {
        let ps = pages(6);
        for gap in [0.0, 5.0, 12.5, 50.0] {
            let placed = layout(&ps, PageId(1), gap);
            assert_eq!(placed[0].1, Rect::new(0.0, 0.0, 100.0, 50.0));
            assert_eq!(placed[1].1, Rect::new(100.0 + gap, 0.0, 200.0 + gap, 50.0));
            assert_eq!(
                placed[3].1,
                Rect::new(300.0 + 3.0 * gap, 0.0, 400.0 + 3.0 * gap, 50.0)
            );
            assert_eq!(placed[4].1, Rect::new(0.0, -50.0 - gap, 100.0, -gap));
            assert_eq!(
                bounds(&placed),
                Some(Rect::new(0.0, -50.0 - gap, 400.0 + 3.0 * gap, 50.0))
            );
        }
        // A negative or broken spacing counts as none.
        assert_eq!(layout(&ps, PageId(1), -10.0), layout(&ps, PageId(1), 0.0));
        assert_eq!(
            layout(&ps, PageId(1), f64::NAN),
            layout(&ps, PageId(1), 0.0)
        );
    }

    /// The drawing's spacing drives the grid, hit testing and Zoom to
    /// Fit; changing it is one undo step and keeps the current page in
    /// place.
    #[test]
    fn the_drawing_s_page_spacing_moves_the_pages_apart() {
        let mut app = App::headless();
        app.add_page();
        let first = app.doc().pages[0].id;
        let second = app.doc().pages[1].id;
        app.page = first;
        app.set_multipage(true);
        let w = app.doc().pages[0].size.width;
        assert_eq!(app.page_spacing(), GAP_MM);
        assert_eq!(app.multipage_layout()[1].1.x0, w + 30.0);
        let fit_before = app.fit_rect().width();
        let undo_before = app.engine.undo_label();
        assert!(app.set_page_spacing(5.0));
        assert!(!app.set_page_spacing(5.0), "no change, no undo step");
        assert_eq!(app.doc().metadata.page_spacing, Some(5.0));
        let placed = app.multipage_layout();
        assert_eq!(placed[0].1.x0, 0.0, "the current page stays put");
        assert_eq!(placed[1].1.x0, w + 5.0);
        assert!((app.fit_rect().width() - (fit_before - 25.0)).abs() < 1e-9);
        // A point in the old gap is on the second page now.
        let p = Point::new(w + 10.0, 1.0);
        assert_eq!(page_at(&placed, p), Some(second));
        let s = app.view.to_screen(p);
        assert!(app.multipage_press(s));
        assert_eq!(app.page, second);
        app.multipage_activate(first);
        // One undo step back to the default.
        assert_ne!(app.engine.undo_label(), undo_before);
        app.undo();
        assert_eq!(app.page_spacing(), 30.0);
        assert_eq!(app.multipage_layout()[1].1.x0, w + 30.0);
        // Out of range values are kept in range.
        assert!(app.set_page_spacing(-3.0));
        assert_eq!(app.page_spacing(), 0.0);
        assert!(!app.set_page_spacing(f64::INFINITY));
    }

    #[test]
    fn a_click_on_another_page_makes_it_current_without_moving_the_view() {
        let mut app = App::headless();
        app.add_page();
        let first = app.doc().pages[0].id;
        let second = app.doc().pages[1].id;
        app.page = first;
        app.set_multipage(true);
        app.fit_pending = false;
        let placed = app.multipage_layout();
        let r = placed[1].1;
        let on_second = app.view.to_screen(Point::new(r.x0 + 5.0, r.y0 + 5.0));
        let before = app.view.to_screen(Point::new(r.x0, r.y0));
        // A press on the current page changes nothing.
        let on_first = app.view.to_screen(Point::new(5.0, 5.0));
        assert!(!app.multipage_press(on_first));
        assert!(app.multipage_press(on_second));
        assert_eq!(app.page, second);
        // The second page's corner is now the origin, at the same place.
        let after = app.view.to_screen(Point::new(0.0, 0.0));
        assert!((after - before).length() < 1e-3);
        let p = app.view.to_page(on_second);
        assert!((p.x - 5.0).abs() < 1e-3 && (p.y - 5.0).abs() < 1e-3);
        // Fit covers both pages.
        assert!(app.fit_rect().width() > app.page_rect().width());
        // Off: a press does nothing, Fit is the page again.
        app.set_multipage(false);
        assert!(!app.multipage_press(on_first));
        assert_eq!(app.fit_rect(), app.page_rect());
    }

    /// Each open drawing keeps its own pictures of its other pages: the
    /// pages of a drawing never show another drawing's pages, not even
    /// for a moment after switching, though page ids repeat between
    /// drawings (#1663).
    #[test]
    fn each_drawing_keeps_its_own_other_page_pictures() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.show_welcome = false;
        let fill = |app: &mut App, w: f64| {
            for _ in 0..2 {
                app.add_page();
                app.new_shape(tracedraw_core::ShapeKind::Rect {
                    rect: Rect::new(10.0, 10.0, w, 40.0),
                    radius: 0.0,
                    corners: None,
                });
            }
            app.page = app.doc().pages[0].id;
        };
        fill(&mut app, 60.0);
        let a_pages: Vec<PageId> = app.doc().pages.iter().map(|p| p.id).collect();
        app.set_multipage(true);
        let mut t = 0.0;
        let mut rest = |app: &mut App| {
            for _ in 0..400 {
                t += 0.02;
                frame_at(&ctx, app, t);
                if renders(app) >= 2 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        rest(&mut app);
        let a_renders = renders(&app);
        assert!(a_renders >= 2, "the first drawing's other pages rendered");

        app.open_document(
            tracedraw_core::Document::new("Second", Size::new(123.0, 45.0)),
            None,
        );
        fill(&mut app, 90.0);
        let b_pages: Vec<PageId> = app.doc().pages.iter().map(|p| p.id).collect();
        // The ids the cache was keyed by repeat in the new drawing.
        assert!(b_pages.iter().any(|id| a_pages.contains(id)));
        // Nothing of the first drawing's pages is there to show.
        assert!(app.page_rasters.borrow().is_empty());
        assert!(app.page_snapshot.borrow().is_none());
        rest(&mut app);
        assert!(
            renders(&app) >= 2,
            "the second drawing's other pages rendered"
        );

        // Back to the first drawing: its own pictures, none of the
        // second's, and no render needed to show them.
        app.switch_document(0);
        assert_eq!(renders(&app), a_renders);
        let a_width = app.doc().pages[0].size.width;
        assert_ne!(a_width, 123.0);
        assert!(app
            .page_snapshot
            .borrow()
            .as_ref()
            .is_none_or(|(_, d)| d.pages[0].size.width == a_width));
        // And to the second again: its own pictures came along.
        app.switch_document(1);
        assert!(renders(&app) >= 2);
    }
}
