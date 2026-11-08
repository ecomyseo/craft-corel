//! Repeat Copies' live preview (#1197): while the Repeat Copies
//! panel is being worked in (the pointer on it, one of its fields being
//! typed in, or the last click was in it), the copies an Apply would make
//! are outlined on the canvas, dashed in the selection colour as the other
//! panels' previews. Nothing is added to the drawing until Apply, which
//! stays one undo step; the preview goes when the panel is closed or
//! hidden, when the work moves elsewhere (a click outside the panel) and
//! when nothing is selected.
//!
//! Cost: the selected objects' outlines are flattened once, in page
//! space, and kept until the selection, the drawing, the values or the
//! zoom change. Each frame the outlines are moved to the screen once and
//! every copy is drawn by adding its screen offset, as line segments;
//! copies outside the drawing window are skipped, and past
//! [`OUTLINE_BUDGET`] points in all the copies show as their boxes.
//!
//! Fill page (#1255) previews the same way: every place of the grid,
//! the selection's outlines turned once when the copies stand turned,
//! and the trim marks as plain lines.

use crate::app::App;
use crate::fill_page::{FillGrid, FillSettings, Turn};
use crate::i18n::{tr, trf};
use crate::ops2::StepRepeat;
use crate::theme::Tokens;
use egui::{Painter, Pos2};
use tracedraw_core::geometry::{Affine, BezPath, PathEl, Point, Rect, Size, Vec2};
use tracedraw_core::{Color, Command, Fill, Shape, ShapeId, ShapeKind, Stroke};

/// The most outline points drawn for all the copies in a frame; past it
/// each copy shows as its box.
pub const OUTLINE_BUDGET: usize = 20_000;

/// What the cached outlines were worked out for.
#[derive(Debug, Clone, PartialEq)]
pub struct StepPreviewKey {
    settings: StepRepeat,
    revision: u64,
    drawing: Option<u64>,
    selection: Vec<ShapeId>,
    zoom: f32,
}

/// The selection's outlines (page space, flattened; turned for a Fill
/// page of turned copies), their box, the step from one copy to the next
/// and where each copy goes.
#[derive(Debug, Clone)]
pub struct StepPreview {
    key: StepPreviewKey,
    pub outlines: Vec<(Vec<Point>, bool)>,
    pub bounds: Rect,
    pub step: Vec2,
    pub copies: u32,
    /// How far copy `k` (1 the first) lies from `bounds`.
    pub offsets: Vec<Vec2>,
    /// A Fill page's trim marks.
    pub marks: Vec<(Point, Point)>,
    points: usize,
}

impl StepPreview {
    /// Each copy as its box rather than its outlines.
    pub fn as_boxes(&self) -> bool {
        self.points.saturating_mul(self.copies as usize) > OUTLINE_BUDGET
    }

    /// The box of copy `k` (1 is the first copy).
    pub fn copy_bounds(&self, k: u32) -> Rect {
        let d = self.offset(k);
        Rect::new(
            self.bounds.x0 + d.x,
            self.bounds.y0 + d.y,
            self.bounds.x1 + d.x,
            self.bounds.y1 + d.y,
        )
    }

    /// How far copy `k` (1 the first) lies from the selection.
    pub fn offset(&self, k: u32) -> Vec2 {
        (k as usize)
            .checked_sub(1)
            .and_then(|i| self.offsets.get(i))
            .copied()
            .unwrap_or(self.step * k as f64)
    }
}

/// The trim marks' outline: black on every plate, 0.2 mm.
fn mark_stroke() -> Stroke {
    Stroke::new(Color::Registration, 0.2)
}

impl App {
    /// The page or area a Fill page tiles (#1255).
    pub fn fill_area(&self) -> Rect {
        let sr = &self.step_repeat;
        let a = sr.area;
        if sr.area_custom
            && a.width().is_finite()
            && a.height().is_finite()
            && a.width() > 0.0
            && a.height() > 0.0
        {
            a
        } else {
            self.page_rect()
        }
    }

    /// The Fill page settings of the panel.
    pub fn fill_settings(&self) -> FillSettings {
        let sr = &self.step_repeat;
        FillSettings {
            margins: if sr.margins_same {
                [sr.margins[0]; 4]
            } else {
                sr.margins
            },
            gap: if sr.gap_same {
                (sr.gap.0, sr.gap.0)
            } else {
                sr.gap
            },
            row_shift: sr.row_shift,
            turn: Turn::from_index(sr.turn),
        }
    }

    /// The grids of a Fill page of the selection: as it stands, turned,
    /// and the one chosen; `None` with nothing selected.
    pub fn fill_page_layout(&self) -> Option<(FillGrid, FillGrid, FillGrid)> {
        let b = self.selection_bounds()?;
        Some(crate::fill_page::layout(
            Size::new(b.width(), b.height()),
            self.fill_area(),
            &self.fill_settings(),
        ))
    }

    /// The count line of a grid: "4 x 6 = 24 copies", with the copies of
    /// a shifted row when they are fewer.
    pub fn fill_count_text(g: &FillGrid) -> String {
        let n = crate::fill_page::count(g).to_string();
        let (cols, rows) = (g.cols.to_string(), g.rows.to_string());
        if g.rows > 1 && g.shifted_cols != g.cols {
            trf(
                "docker.fillpage_count_shifted",
                &[
                    ("cols", &cols),
                    ("shifted", &g.shifted_cols.to_string()),
                    ("rows", &rows),
                    ("n", &n),
                ],
            )
        } else {
            trf(
                "docker.fillpage_count",
                &[("cols", &cols), ("rows", &rows), ("n", &n)],
            )
        }
    }

    /// The map from the selection to each place of a grid: turned about
    /// the selection's middle when the copies stand turned, then moved.
    fn fill_transforms(bounds: Rect, g: &FillGrid) -> Vec<Affine> {
        let c = bounds.center();
        let turn = if g.turned {
            Affine::rotate_about(std::f64::consts::FRAC_PI_2, c)
        } else {
            Affine::IDENTITY
        };
        g.cells
            .iter()
            .map(|cell| Affine::translate(cell.center() - c) * turn)
            .collect()
    }

    /// Fill page (#1255): the selection moves to the first place of the
    /// grid and a copy goes to every other one, with the trim marks when
    /// asked, as one undo step; everything made stays selected.
    pub fn fill_page_apply(&mut self) {
        let shapes = self.selected_shapes();
        if shapes.is_empty() {
            return;
        }
        let (Some(bounds), Some((_, _, g))) = (self.selection_bounds(), self.fill_page_layout())
        else {
            return;
        };
        if crate::fill_page::count(&g) == 0 {
            self.status = tr("status.fill_page_none");
            return;
        }
        if g.cells.is_empty() {
            self.status = trf(
                "status.fill_page_too_many",
                &[("max", &crate::fill_page::MAX_COPIES.to_string())],
            );
            return;
        }
        let Some(layer) = self.active_layer() else {
            return;
        };
        let sr = self.step_repeat.clone();
        let ids: Vec<ShapeId> = shapes.iter().map(|s| s.id).collect();
        let mut cmds = Vec::new();
        let mut new_sel = ids.clone();
        for (i, t) in Self::fill_transforms(bounds, &g).into_iter().enumerate() {
            if i == 0 {
                cmds.push(Command::TransformShapes {
                    shapes: ids.clone(),
                    transform: t,
                });
                continue;
            }
            for s in &shapes {
                let mut c = s.clone();
                c.id = self.engine.new_shape_id();
                c.transform_by(t);
                new_sel.push(c.id);
                cmds.push(Command::AddShape { layer, shape: c });
            }
        }
        if sr.marks {
            let mut path = BezPath::new();
            for (a, b) in crate::fill_page::trim_marks(&g.cells, sr.mark_length, sr.mark_offset) {
                path.move_to(a);
                path.line_to(b);
            }
            if !path.elements().is_empty() {
                let mut m = Shape::new(
                    self.engine.new_shape_id(),
                    ShapeKind::Path {
                        path,
                        closed: false,
                    },
                );
                m.fill = Fill::None;
                m.stroke = Some(mark_stroke());
                m.name = Some(tr("docker.fillpage_marks_name"));
                new_sel.push(m.id);
                cmds.push(Command::AddShape { layer, shape: m });
            }
        }
        if let Err(e) = self.engine.run_batch("Fill Page", &cmds) {
            self.status = e.to_string();
            return;
        }
        self.selection = new_sel;
    }
}

impl App {
    /// The distance from one copy to the next for the selection: the
    /// offsets as typed, or with Spacing the selection's size plus them.
    pub fn step_and_repeat_step(&self) -> Option<Vec2> {
        let b = self.selection_bounds()?;
        let sr = &self.step_repeat;
        let dx = if sr.mode_x == 1 {
            b.width() + sr.dx
        } else {
            sr.dx
        };
        let dy = if sr.mode_y == 1 {
            b.height() + sr.dy
        } else {
            sr.dy
        };
        Some(Vec2::new(dx, dy))
    }

    /// The preview of the copies an Apply would make, worked out again
    /// only when its key changed; `None` with nothing selected.
    pub fn step_and_repeat_preview(&mut self) -> Option<&StepPreview> {
        if self.selection.is_empty() {
            self.step_preview = None;
            return None;
        }
        let key = StepPreviewKey {
            settings: self.step_repeat.clone(),
            revision: self.engine.revision(),
            drawing: self.docs.get(self.active_doc).map(|d| d.drawing),
            selection: self.selection.clone(),
            zoom: self.view.zoom,
        };
        if self.step_preview.as_ref().is_none_or(|p| p.key != key) {
            self.step_preview = self.build_step_preview(key);
        }
        self.step_preview.as_ref()
    }

    fn build_step_preview(&self, key: StepPreviewKey) -> Option<StepPreview> {
        let step = self.step_and_repeat_step()?;
        let sel = self.selection_bounds()?;
        // A Fill page: the places of the grid, the outlines turned once
        // when the copies stand turned, and the trim marks.
        let fill = if key.settings.fill {
            self.fill_page_layout().map(|(_, _, g)| g)
        } else {
            None
        };
        let turn = match &fill {
            Some(g) if g.turned => Affine::rotate_about(std::f64::consts::FRAC_PI_2, sel.center()),
            _ => Affine::IDENTITY,
        };
        let bounds = if turn == Affine::IDENTITY {
            sel
        } else {
            let c = sel.center();
            let (hw, hh) = (sel.height() / 2.0, sel.width() / 2.0);
            Rect::new(c.x - hw, c.y - hh, c.x + hw, c.y + hh)
        };
        let (offsets, marks): (Vec<Vec2>, Vec<(Point, Point)>) = match &fill {
            Some(g) => (
                g.cells
                    .iter()
                    .map(|c| c.center() - bounds.center())
                    .collect(),
                if key.settings.marks {
                    crate::fill_page::trim_marks(
                        &g.cells,
                        key.settings.mark_length,
                        key.settings.mark_offset,
                    )
                } else {
                    Vec::new()
                },
            ),
            None => (
                (1..=key.settings.copies.max(1))
                    .map(|k| step * k as f64)
                    .collect(),
                Vec::new(),
            ),
        };
        // A quarter of a screen pixel, as the canvas flattens.
        let tol = 0.25 / self.view.zoom.max(0.05) as f64;
        let mut outlines: Vec<(Vec<Point>, bool)> = Vec::new();
        for s in self.selected_shapes() {
            let mut path = s.page_path();
            path.apply_affine(turn);
            let mut cur: Vec<Point> = Vec::new();
            let mut closed = false;
            tracedraw_core::geometry::flatten(&path, tol, &mut |el| match el {
                PathEl::MoveTo(p) => {
                    if cur.len() > 1 {
                        outlines.push((std::mem::take(&mut cur), closed));
                    }
                    cur.clear();
                    closed = false;
                    cur.push(p);
                }
                PathEl::LineTo(p) => cur.push(p),
                PathEl::ClosePath => closed = true,
                _ => {}
            });
            if cur.len() > 1 {
                outlines.push((cur, closed));
            }
        }
        let points = outlines.iter().map(|(p, c)| p.len() + *c as usize).sum();
        Some(StepPreview {
            copies: offsets.len() as u32,
            key,
            outlines,
            bounds,
            step,
            offsets,
            marks,
            points,
        })
    }
}

/// Draw the preview of the copies when the panel asked for it this frame.
pub fn draw_preview(app: &App, painter: &Painter) {
    if !app.step_preview_shown {
        return;
    }
    let Some(pv) = app.step_preview.as_ref() else {
        return;
    };
    let view = &app.view;
    let stroke = egui::Stroke::new(1.0, Tokens::SELECTION);
    let clip = painter.clip_rect();
    let origin = view.to_screen(Point::ZERO);
    let boxes = pv.as_boxes();
    // The outlines on screen once; each copy adds its offset.
    let screen: Vec<(Vec<Pos2>, bool)> = if boxes {
        Vec::new()
    } else {
        pv.outlines
            .iter()
            .map(|(pts, closed)| (pts.iter().map(|p| view.to_screen(*p)).collect(), *closed))
            .collect()
    };
    let mut buf: Vec<Pos2> = Vec::new();
    for k in 1..=pv.copies {
        let b = pv.copy_bounds(k);
        let r = egui::Rect::from_two_pos(
            view.to_screen(Point::new(b.x0, b.y0)),
            view.to_screen(Point::new(b.x1, b.y1)),
        );
        if !r.expand(1.0).intersects(clip) {
            continue;
        }
        if boxes {
            buf.clear();
            buf.extend([
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
                r.left_top(),
            ]);
            crate::canvas::dashed_polyline(painter, &buf, stroke, 4.0, 3.0);
            continue;
        }
        let o = pv.offset(k);
        let d = view.to_screen(Point::new(o.x, o.y)) - origin;
        for (pts, closed) in &screen {
            buf.clear();
            buf.extend(pts.iter().map(|p| *p + d));
            if *closed {
                if let Some(first) = buf.first().copied() {
                    buf.push(first);
                }
            }
            crate::canvas::dashed_polyline(painter, &buf, stroke, 4.0, 3.0);
        }
    }
    // A Fill page's trim marks, as plain segments.
    for (a, b) in &pv.marks {
        let (a, b) = (view.to_screen(*a), view.to_screen(*b));
        if clip.intersects(egui::Rect::from_two_pos(a, b).expand(1.0)) {
            painter.line_segment([a, b], stroke);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::ShapeKind;

    fn app_with_rect() -> (App, ShapeId) {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 30.0, 20.0),
                radius: 0.0,
                corners: None,
            })
            .expect("a rectangle");
        app.select(vec![id]);
        (app, id)
    }

    /// #1197: the preview has a copy for each one Apply makes, at the
    /// same places, and adds nothing to the drawing; Apply is one undo
    /// step and makes exactly the copies previewed.
    #[test]
    fn the_preview_shows_the_copies_apply_makes() {
        let (mut app, _) = app_with_rect();
        app.step_repeat.copies = 3;
        app.step_repeat.dx = 25.0;
        app.step_repeat.dy = 0.0;
        let before = app.engine.revision();
        let shapes = app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        let pv = app.step_and_repeat_preview().expect("a preview").clone();
        assert_eq!(app.engine.revision(), before, "the preview changes nothing");
        assert_eq!(pv.copies, 3);
        assert!(!pv.as_boxes());
        assert_eq!(pv.outlines.len(), 1);
        let previewed: Vec<Rect> = (1..=3).map(|k| pv.copy_bounds(k)).collect();
        app.step_and_repeat_apply();
        let after = app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        assert_eq!(after, shapes + 3);
        let made: Vec<Rect> = app
            .selected_shapes()
            .iter()
            .map(|s| tracedraw_core::geometry::Shape::bounding_box(&s.page_path()))
            .collect();
        for (p, m) in previewed.iter().zip(&made) {
            assert!(
                (p.x0 - m.x0).abs() < 1e-6 && (p.y0 - m.y0).abs() < 1e-6,
                "{p:?} {m:?}"
            );
        }
        app.undo();
        let undone = app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        assert_eq!(undone, shapes, "one undo takes every copy away");
    }

    /// Spacing mode steps by the selection's size plus the distance; the
    /// preview follows a changed value at once and is kept while nothing
    /// changes.
    #[test]
    fn the_preview_follows_the_values_and_is_cached() {
        let (mut app, _) = app_with_rect();
        app.step_repeat.copies = 2;
        app.step_repeat.mode_x = 1;
        app.step_repeat.dx = 5.0;
        app.step_repeat.dy = 0.0;
        let step = app.step_and_repeat_preview().expect("a preview").step;
        assert!((step.x - 25.0).abs() < 1e-9, "{step:?}");
        let ptr = app.step_preview.as_ref().map(|p| p.outlines.as_ptr());
        app.step_and_repeat_preview();
        assert_eq!(
            app.step_preview.as_ref().map(|p| p.outlines.as_ptr()),
            ptr,
            "kept while nothing changes"
        );
        app.step_repeat.dy = -7.0;
        let step = app.step_and_repeat_preview().expect("a preview").step;
        assert!((step.y + 7.0).abs() < 1e-9);
        app.select(Vec::new());
        assert!(app.step_and_repeat_preview().is_none());
    }

    /// Many copies of a detailed outline show as boxes.
    #[test]
    fn many_copies_show_as_boxes() {
        let (mut app, _) = app_with_rect();
        app.step_repeat.copies = 999;
        let pv = app.step_and_repeat_preview().expect("a preview");
        // A rectangle has 5 points: 999 copies stay under the budget.
        assert!(!pv.as_boxes());
        let mut big = pv.clone();
        big.points = OUTLINE_BUDGET;
        big.copies = 2;
        assert!(big.as_boxes());
    }

    fn shape_count(app: &App) -> usize {
        app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum()
    }

    /// #1255: Fill page tiles the selection over the page: the original
    /// takes the first place and copies the others, as many as the count
    /// shown, at the places previewed, with the trim marks; one undo
    /// takes it all back.
    #[test]
    fn fill_page_makes_the_copies_counted_and_previewed() {
        let (mut app, id) = app_with_rect();
        app.step_repeat.fill = true;
        app.step_repeat.margins = [10.0; 4];
        app.step_repeat.gap = (5.0, 5.0);
        app.step_repeat.marks = true;
        let b = app.selection_bounds().expect("bounds");
        let (as_is, _, g) = app.fill_page_layout().expect("a layout");
        assert_eq!(g, as_is, "as it stands by default");
        let n = crate::fill_page::count(&g);
        // A4 inside 10 mm margins, 5 mm gaps.
        let fits = |room: f64, size: f64| ((room + 5.0) / (size + 5.0)).floor() as usize;
        assert_eq!(g.cols, fits(190.0, b.width()));
        assert_eq!(g.rows, fits(277.0, b.height()));
        assert_eq!(n, g.cols * g.rows);
        let text = App::fill_count_text(&g);
        assert!(
            text.contains(&format!("{} x {}", g.cols, g.rows)) && text.contains(&n.to_string()),
            "{text}"
        );
        let pv = app.step_and_repeat_preview().expect("a preview").clone();
        assert_eq!(pv.copies as usize, n);
        assert!(!pv.marks.is_empty());
        let previewed: Vec<Rect> = (1..=pv.copies).map(|k| pv.copy_bounds(k)).collect();
        let before = shape_count(&app);
        app.step_and_repeat_apply();
        // The copies and one object of trim marks.
        assert_eq!(shape_count(&app), before + n - 1 + 1);
        assert_eq!(app.selection.len(), n + 1);
        assert_eq!(app.selection[0], id, "the original stays, moved");
        let boxes = app.selected_bounds();
        for (p, m) in previewed.iter().zip(boxes.iter()) {
            assert!(
                (p.x0 - m.x0).abs() < 1e-6 && (p.y1 - m.y1).abs() < 1e-6,
                "{p:?} {m:?}"
            );
        }
        let marks = app.selected_shapes().last().cloned().expect("the marks");
        assert!(matches!(marks.kind, ShapeKind::Path { closed: false, .. }));
        assert!(marks.stroke.is_some());
        app.undo();
        assert_eq!(shape_count(&app), before, "one undo takes it all back");
        let back = app.doc().pages[0]
            .layers
            .iter()
            .flat_map(|l| l.shapes.iter())
            .find(|s| s.id == id)
            .map(|s| tracedraw_core::geometry::Shape::bounding_box(&s.page_path()))
            .expect("the original");
        assert!((back.x0 - 10.0).abs() < 1e-6 && (back.y0 - 10.0).abs() < 1e-6);
    }

    /// Turned copies stand a quarter turn round, and "whichever fits
    /// more" turns them when that fits more; nothing fits, nothing is
    /// made.
    #[test]
    fn fill_page_turns_the_copies_to_fit_more() {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 120.0, 80.0),
                radius: 0.0,
                corners: None,
            })
            .expect("a rectangle");
        app.select(vec![id]);
        app.step_repeat.fill = true;
        app.step_repeat.turn = 2;
        let (a, t, g) = app.fill_page_layout().expect("a layout");
        // 120 x 80 (and its outline) on A4: 1 x 3 as it stands, 2 x 2
        // turned.
        assert_eq!(crate::fill_page::count(&a), 3);
        assert_eq!(crate::fill_page::count(&t), 4);
        assert!(g.turned);
        let before = shape_count(&app);
        app.step_and_repeat_apply();
        assert_eq!(shape_count(&app), before + 3);
        for b in app.selected_bounds().iter() {
            assert!(b.height() > b.width(), "turned: {b:?}");
        }
        app.undo();
        // Too big for the page: nothing made, and the panel says so.
        app.select(vec![id]);
        app.step_repeat.margins = [100.0; 4];
        let (_, _, g) = app.fill_page_layout().expect("a layout");
        assert_eq!(crate::fill_page::count(&g), 0);
        let rev = app.engine.revision();
        app.step_and_repeat_apply();
        assert_eq!(app.engine.revision(), rev);
        assert_eq!(app.status, tr("status.fill_page_none"));
    }

    /// An area of the page can be filled instead of the whole page.
    #[test]
    fn fill_page_fills_a_chosen_area() {
        let (mut app, _) = app_with_rect();
        app.step_repeat.fill = true;
        app.step_repeat.area_custom = true;
        app.step_repeat.area = Rect::new(50.0, 100.0, 150.0, 200.0);
        let (_, _, g) = app.fill_page_layout().expect("a layout");
        assert!(crate::fill_page::count(&g) > 0);
        for c in &g.cells {
            assert!(c.x0 >= 50.0 - 1e-9 && c.x1 <= 150.0 + 1e-9, "{c:?}");
            assert!(c.y0 >= 100.0 - 1e-9 && c.y1 <= 200.0 + 1e-9, "{c:?}");
        }
    }
}
