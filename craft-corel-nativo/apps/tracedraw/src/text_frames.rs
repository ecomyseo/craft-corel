//! Paragraph text frames with the Pick tool: scaling a paragraph text with
//! the handles (or the property bar's size and scale) changes its frame and
//! the text flows anew in it, the letters keeping their size; turning,
//! slanting, mirroring and moving act on the whole text as before. While
//! a handle is dragged the text flows anew in the frame it would get, so
//! the outline shows the lines the release makes (#1327). Frames show as
//! dashed lines (Text > Show Text Frames). See docs/behavior/text.md.

use crate::app::App;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use tracedraw_core::document::text_outline::TextRequest;
use tracedraw_core::geometry::{Affine, BezPath, Point, Rect, Size};
use tracedraw_core::{Command, Shape, ShapeId, ShapeKind, TextSpan};

/// The longest a text's layout may take, in seconds, for it to flow anew
/// in its frame on every frame of a drag: a quarter of a 60 Hz frame, so
/// the drag keeps its pace with the rest of the frame's work. A text that
/// takes longer three times in a row shows its frame only, and flows on
/// release, until the drawing changes.
pub const LIVE_REFLOW_BUDGET: f64 = 0.004;

/// How many layouts in a row may take longer than the budget before a
/// text stops flowing live.
pub const LIVE_REFLOW_OVER: u32 = 3;

/// The longest text (in bytes of UTF-8) whose glyphs a drag shapes to
/// flow it live, when the text engine has not kept them: a few
/// milliseconds of shaping.
pub const LIVE_SHAPE_BYTES: usize = 4000;

/// A paragraph text flowed in the frame a drag would give it.
#[derive(Debug, Clone)]
pub struct LiveFrame {
    /// The frame laid out (the text's own space).
    pub frame: Size,
    /// The letters' outline, page space, as the release places them.
    pub outline: BezPath,
    /// The lines that show in the frame.
    #[cfg(test)]
    pub lines: usize,
    /// The text does not fit (and has no frame to flow on into).
    pub overflow: bool,
}

/// Live layouts of paragraph texts whose frames are being dragged: one
/// layout per text and frame size (a frame draws them all from the same
/// one, so there is at most one layout a frame), kept per document
/// revision and font generation.
#[derive(Debug)]
pub struct LiveReflow {
    key: (u64, u64),
    /// [`LIVE_REFLOW_BUDGET`] (tests change it; below zero the live
    /// reflow is off, the frame following alone as before).
    pub budget: f64,
    frames: HashMap<ShapeId, Rc<LiveFrame>>,
    /// Layouts made per text: the first one shapes the glyphs, which the
    /// text engine keeps, so it does not count against the budget.
    layouts: HashMap<ShapeId, u32>,
    /// Layouts in a row per text that took longer than the budget: one
    /// alone may be the machine busy with something else.
    over: HashMap<ShapeId, u32>,
    /// Texts whose layout took longer than [`LIVE_REFLOW_BUDGET`].
    slow: HashSet<ShapeId>,
}

impl Default for LiveReflow {
    fn default() -> Self {
        LiveReflow {
            key: (0, 0),
            budget: LIVE_REFLOW_BUDGET,
            frames: HashMap::new(),
            layouts: HashMap::new(),
            over: HashMap::new(),
            slow: HashSet::new(),
        }
    }
}

impl LiveReflow {
    /// Layouts made so far for `id` in this revision (for tests).
    #[cfg(test)]
    pub fn layouts_of(&self, id: ShapeId) -> u32 {
        self.layouts.get(&id).copied().unwrap_or(0)
    }
}

/// The lines of a text's layout that show in its frame.
#[cfg(test)]
pub fn shown_lines(layout: &tracedraw_text::TextLayout) -> usize {
    layout
        .lines
        .iter()
        .filter(|l| l.start_char < layout.fitted_chars)
        .count()
}

/// A paragraph text frame as shown on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShownFrame {
    pub id: tracedraw_core::ShapeId,
    /// The frame flows into another (its tab shows a link).
    pub linked: bool,
    /// Another frame flows into this one (its top tab shows a link).
    pub receives: bool,
    /// Page-space corners, the frame's lower left first.
    pub corners: [Point; 4],
    /// The text is selected or being edited.
    pub chosen: bool,
    /// The text is being edited (its frame is drawn black).
    pub editing: bool,
    /// The text does not fit (drawn red, with a tab below).
    pub overflow: bool,
}

/// The factors of an axis-aligned scale by positive amounts (with any
/// move), else `None`.
fn axis_scale(a: Affine) -> Option<(f64, f64)> {
    let [sx, b, c, sy, e, f] = a.as_coeffs();
    let finite = [sx, b, c, sy, e, f].iter().all(|v| v.is_finite());
    (finite && b.abs() < 1e-12 && c.abs() < 1e-12 && sx > 1e-9 && sy > 1e-9).then_some((sx, sy))
}

/// The page-space box of a paragraph text's frame.
fn frame_box(s: &Shape) -> Option<Rect> {
    let c = s.text_frame_corners()?;
    Some(
        c.iter()
            .skip(1)
            .fold(Rect::from_points(c[0], c[0]), |r, p| r.union_pt(*p)),
    )
}

/// The text kind of `s` (a paragraph text whose transform is an axis-
/// aligned scale) with its frame made the image of its page-space frame
/// under `t`.
fn resized_kind(s: &Shape, t: Affine) -> Option<ShapeKind> {
    let (tx, ty) = axis_scale(s.transform)?;
    let ShapeKind::Text {
        spans,
        align,
        para,
        on_path: None,
        frame: Some(_),
        ..
    } = &s.kind
    else {
        return None;
    };
    let old = frame_box(s)?;
    let a = t * Point::new(old.x0, old.y0);
    let b = t * Point::new(old.x1, old.y1);
    let new = Rect::from_points(a, b);
    let origin = s.transform.inverse() * Point::new(new.x0, new.y0);
    Some(ShapeKind::Text {
        spans: spans.clone(),
        origin,
        frame: Some(Size::new(
            (new.width() / tx).max(0.5),
            (new.height() / ty).max(0.5),
        )),
        align: *align,
        para: para.clone(),
        on_path: None,
    })
}

impl App {
    /// `t` (page space) resizes the frame of `id` instead of scaling it: a
    /// paragraph text lying on a layer, neither turned nor slanted, and `t`
    /// an axis-aligned scale by positive amounts that is not just a move.
    pub fn resizes_text_frame(&self, id: ShapeId, t: Affine) -> bool {
        let Some((sx, sy)) = axis_scale(t) else {
            return false;
        };
        if (sx - 1.0).abs() < 1e-9 && (sy - 1.0).abs() < 1e-9 {
            return false;
        }
        self.doc()
            .shape(id)
            .ok()
            .is_some_and(|(_, s)| resized_kind(s, t).is_some())
    }

    /// Apply `t` to the selection as one undo step: paragraph texts that
    /// take it as a new frame get it so, the rest are transformed.
    /// Returns false when no paragraph frame was involved (the caller
    /// transforms as usual).
    pub(crate) fn transform_with_frames(&mut self, shapes: &[ShapeId], t: Affine) -> bool {
        let frames: Vec<ShapeId> = shapes
            .iter()
            .copied()
            .filter(|id| self.resizes_text_frame(*id, t))
            .collect();
        if frames.is_empty() {
            return false;
        }
        let mut cmds = Vec::new();
        let others: Vec<ShapeId> = shapes
            .iter()
            .copied()
            .filter(|id| !frames.contains(id))
            .collect();
        if !others.is_empty() {
            cmds.push(Command::TransformShapes {
                shapes: others,
                transform: t,
            });
        }
        for id in &frames {
            let Ok((_, s)) = self.doc().shape(*id) else {
                continue;
            };
            if let Some(kind) = resized_kind(s, t) {
                cmds.push(Command::SetShapeKind { shape: *id, kind });
            }
            // A frame's symmetry goes with it, as an object's does in a
            // transform (not the one being edited).
            if self.symmetry_edit != Some(*id)
                && s.effect_stack()
                    .iter()
                    .any(|(e, _)| matches!(e, tracedraw_core::live::Effect::Symmetry { .. }))
            {
                let mut moved = s.clone();
                moved.carry_effects(t);
                cmds.push(Command::SetEffectStack {
                    shape: *id,
                    effects: moved.effects,
                    parked: moved.parked_effects,
                });
            }
        }
        if let Err(e) = self.run_batch_keeping_lines("Transform", &cmds) {
            self.status = e.to_string();
        }
        true
    }

    /// The text of paragraph frame `id` flowed anew in the frame `t`
    /// gives it while a handle is dragged, laid out at most once per frame
    /// size; `None` when `t` does not resize its frame, or when its layout
    /// is too slow for a frame ([`LIVE_REFLOW_BUDGET`]): then the frame
    /// follows alone and the text flows on release, as before (#1327).
    pub fn live_frame(&self, id: ShapeId, t: Affine) -> Option<Rc<LiveFrame>> {
        if !self.resizes_text_frame(id, t) {
            return None;
        }
        let s = self.doc().find_shape(id)?;
        let kind = resized_kind(s, t)?;
        let ShapeKind::Text {
            spans,
            origin,
            frame: Some(frame),
            align,
            para,
            ..
        } = &kind
        else {
            return None;
        };
        let key = (self.engine.revision(), tracedraw_text::fonts().generation());
        let mut live = self.live_reflow.borrow_mut();
        if live.key != key {
            *live = LiveReflow {
                key,
                budget: live.budget,
                ..Default::default()
            };
        }
        if live.budget < 0.0 || live.slow.contains(&id) {
            return None;
        }
        if let Some(f) = live.frames.get(&id).filter(|f| f.frame == *frame) {
            return Some(f.clone());
        }
        // A frame of a linked chain keeps what fits of its text and of
        // the frames after it, as the release flows it (a copy dragged
        // off with a right click is not linked).
        let next = self.next_frame(id).filter(|_| !self.drag_copy);
        let flowing: Vec<TextSpan>;
        let spans = if next.is_some() {
            let chain = self.chain_of(id);
            let at = chain.iter().position(|c| *c == id).unwrap_or(0);
            flowing = spans
                .iter()
                .cloned()
                .chain(chain.iter().skip(at + 1).flat_map(|c| {
                    match self.doc().find_shape(*c).map(|s| &s.kind) {
                        Some(ShapeKind::Text { spans, .. }) => spans.clone(),
                        _ => Vec::new(),
                    }
                }))
                .collect();
            &flowing
        } else {
            spans
        };
        let req = TextRequest {
            spans,
            frame: Some(*frame),
            align: *align,
            para,
            on_path: None,
        };
        let fonts = tracedraw_text::fonts();
        // Shaping a long text anew would hold the drag up at its start:
        // only a short one is shaped here (the page's picture shaped the
        // text when it was drawn, and the text engine keeps that).
        if live.layouts.get(&id).is_none()
            && !fonts.is_shaped(&req)
            && spans.iter().map(|s| s.text.len()).sum::<usize>() > LIVE_SHAPE_BYTES
        {
            live.slow.insert(id);
            return None;
        }
        let started = web_time::Instant::now();
        let layout = fonts.layout_fitted(&req);
        let took = started.elapsed().as_secs_f64();
        let count = {
            let n = live.layouts.entry(id).or_insert(0);
            *n += 1;
            *n
        };
        let over = if count > 1 && took > live.budget {
            let n = live.over.entry(id).or_insert(0);
            *n += 1;
            *n
        } else {
            live.over.remove(&id);
            0
        };
        if over >= LIVE_REFLOW_OVER {
            log::debug!("text {id:?} takes {took:.4} s to lay out: it flows on release");
            live.slow.insert(id);
            live.frames.remove(&id);
            return None;
        }
        let place = s.transform
            * tracedraw_core::document::text_layout_shift(
                *origin,
                Some(*frame),
                false,
                para.vertical,
            );
        let f = Rc::new(LiveFrame {
            frame: *frame,
            #[cfg(test)]
            lines: shown_lines(&layout),
            overflow: layout.overflow && next.is_none(),
            outline: place * layout.path,
        });
        live.frames.insert(id, f.clone());
        Some(f)
    }

    /// Paragraph text `s` does not fit its frame (the last frame of a
    /// linked chain included); cached per document revision.
    pub fn text_overflows(&self, s: &Shape) -> bool {
        let rev = self.engine.revision();
        let mut cache = self.overflow_cache.borrow_mut();
        if cache.0 != rev {
            *cache = (rev, Default::default());
        }
        if let Some(v) = cache.1.get(&s.id) {
            return *v;
        }
        let v = match &s.kind {
            ShapeKind::Text {
                spans,
                frame: Some(f),
                align,
                para,
                on_path: None,
                ..
            } => {
                let req = tracedraw_core::document::text_outline::TextRequest {
                    spans,
                    frame: Some(*f),
                    align: *align,
                    para,
                    on_path: None,
                };
                tracedraw_text::fonts().layout(&req).overflow
            }
            _ => false,
        };
        cache.1.insert(s.id, v);
        v
    }

    /// The paragraph text frames to show, in page space, each with whether
    /// it is chosen (selected or edited): every frame on the page with
    /// Text > Show Text Frames, else those of the selected texts and the
    /// text being edited; the selected frames being dragged take `preview`
    /// (resized for a scale, else moved, turned or slanted with the
    /// selection).
    pub fn shown_text_frames(&self, preview: Option<Affine>) -> Vec<ShownFrame> {
        let doc = self.doc();
        let Ok(page) = doc.page(self.page) else {
            return Vec::new();
        };
        let editing = self.text_edit.as_ref().map(|te| te.shape);
        let mut out = Vec::new();
        for s in page
            .layers
            .iter()
            .filter(|l| l.visible)
            .flat_map(|l| &l.shapes)
            .filter(|s| s.visible)
        {
            // Only text frames: asking every object whether it is selected
            // cost a search of the selection each (#367).
            let Some(mut c) = s.text_frame_corners() else {
                continue;
            };
            let selected = self.selection.contains(&s.id);
            let chosen = selected || editing == Some(s.id);
            if !(self.settings.show_text_frames || chosen) {
                continue;
            }
            // A frame being resized tells whether its text fits as it
            // flows in it now.
            let mut live = None;
            if let Some(t) = preview.filter(|t| selected && self.resizes_text_frame(s.id, *t)) {
                if let Some(kind) = resized_kind(s, t) {
                    let mut moved = s.clone();
                    moved.kind = kind;
                    c = moved.text_frame_corners().unwrap_or(c);
                }
                live = self.live_frame(s.id, t);
            } else if let Some(t) = preview.filter(|_| selected) {
                c = c.map(|p| t * p);
            }
            out.push(ShownFrame {
                id: s.id,
                linked: self.next_frame(s.id).is_some(),
                receives: self.prev_frame(s.id).is_some(),
                corners: c,
                chosen,
                editing: editing == Some(s.id),
                overflow: live.map_or_else(|| self.text_overflows(s), |l| l.overflow),
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paragraph(app: &mut App) -> ShapeId {
        // The real text engine, so the words wrap in the frame.
        tracedraw_text::install();
        app.start_text(Point::new(20.0, 20.0), Some(Size::new(60.0, 30.0)));
        for c in "Some words to flow in the frame".chars() {
            app.text_insert(&c.to_string());
        }
        let id = app.selection[0];
        app.finish_text();
        app.select(vec![id]);
        id
    }

    fn frame_of(app: &App, id: ShapeId) -> (Point, Size) {
        match app.doc().shape(id).map(|(_, s)| s.kind.clone()) {
            Ok(ShapeKind::Text {
                origin,
                frame: Some(f),
                ..
            }) => (origin, f),
            other => panic!("not paragraph text: {other:?}"),
        }
    }

    #[test]
    fn scaling_paragraph_text_resizes_its_frame_and_keeps_the_letters() {
        let mut app = App::headless();
        let id = paragraph(&mut app);
        let size_before = match app.doc().shape(id).map(|(_, s)| s.kind.clone()) {
            Ok(ShapeKind::Text { spans, .. }) => spans[0].size_pt,
            _ => panic!("text"),
        };
        // The frame counts in the bounds, empty space and all.
        let b = app.selection_bounds().expect("bounds");
        assert!(
            (b.width() - 60.0).abs() < 1e-6 && (b.height() - 30.0).abs() < 1e-6,
            "{b:?}"
        );
        // Twice as wide from the left edge: the frame doubles, the letters
        // stay, the transform stays the identity.
        let t = Affine::translate((20.0, 0.0))
            * Affine::scale_non_uniform(2.0, 1.0)
            * Affine::translate((-20.0, 0.0));
        app.transform_selection(t);
        let (origin, f) = frame_of(&app, id);
        assert!((f.width - 120.0).abs() < 1e-6 && (f.height - 30.0).abs() < 1e-6);
        assert!((origin.x - 20.0).abs() < 1e-6 && (origin.y - 20.0).abs() < 1e-6);
        let s = app.doc().shape(id).expect("shape").1.clone();
        assert_eq!(s.transform, Affine::IDENTITY);
        match &s.kind {
            ShapeKind::Text { spans, .. } => assert_eq!(spans[0].size_pt, size_before),
            _ => panic!("text"),
        }
        // One undo step brings the old frame back.
        app.undo();
        assert!((frame_of(&app, id).1.width - 60.0).abs() < 1e-6);
        // A turn still turns the whole text; a move moves it.
        let turn = Affine::rotate(0.3);
        app.select(vec![id]);
        app.transform_selection(turn);
        let s = app.doc().shape(id).expect("shape").1.clone();
        assert_eq!(s.transform, turn);
        // Once turned, a scale scales the text as a whole.
        assert!(!app.resizes_text_frame(id, Affine::scale(2.0)));
    }

    /// A frame being resized lays its text out once per frame size (the
    /// canvas asks for it twice a frame), and a text whose layout takes
    /// longer than the budget goes back to flowing on release (#1327).
    #[test]
    fn the_live_reflow_lays_out_once_per_size_and_gives_way_to_a_slow_text() {
        let mut app = App::headless();
        let id = paragraph(&mut app);
        let narrow = |k: f64| {
            Affine::translate((20.0, 0.0))
                * Affine::scale_non_uniform(k, 1.0)
                * Affine::translate((-20.0, 0.0))
        };
        let a = app.live_frame(id, narrow(0.5)).expect("flowed");
        let again = app.live_frame(id, narrow(0.5)).expect("flowed");
        assert!(Rc::ptr_eq(&a, &again));
        assert_eq!(app.live_reflow.borrow().layouts_of(id), 1);
        assert!(app.shown_text_frames(Some(narrow(0.5)))[0].chosen);
        assert_eq!(app.live_reflow.borrow().layouts_of(id), 1, "one a size");
        let wide = app.live_frame(id, narrow(2.0)).expect("flowed");
        assert!(wide.lines < a.lines, "{} {}", wide.lines, a.lines);
        assert_eq!(app.live_reflow.borrow().layouts_of(id), 2);
        // A move or a turn is no frame resize.
        assert!(app.live_frame(id, Affine::translate((5.0, 0.0))).is_none());
        // Slower than the budget: the frame follows alone from then on.
        app.live_reflow.borrow_mut().budget = 0.0;
        assert!(
            app.live_frame(id, narrow(0.7)).is_some(),
            "once may be chance"
        );
        assert!(app.live_frame(id, narrow(0.6)).is_some());
        assert!(app.live_frame(id, narrow(0.8)).is_none());
        assert!(app.live_frame(id, narrow(0.5)).is_none());
        // The drawing changed: tried again.
        app.live_reflow.borrow_mut().budget = 1.0;
        app.transform_selection(narrow(0.9));
        assert!(app.live_frame(id, narrow(0.5)).is_some());
    }

    /// A frame of a linked chain flowed live takes the text of the frames
    /// after it that fits, as the release flows it, and does not show
    /// red for the rest, which flows on.
    #[test]
    fn a_linked_frame_flows_live_with_the_text_after_it() {
        let mut app = App::headless();
        tracedraw_text::install();
        let mk = |app: &mut App, text: &str, y: f64| {
            app.new_shape(ShapeKind::Text {
                spans: vec![tracedraw_core::TextSpan::new(text, "DejaVu Sans", 12.0)],
                origin: Point::new(20.0, y),
                frame: Some(Size::new(40.0, 12.0)),
                align: tracedraw_core::TextAlign::Left,
                para: Default::default(),
                on_path: None,
            })
            .expect("frame")
        };
        let a = mk(
            &mut app,
            "one two three four five six seven eight nine ten eleven twelve",
            100.0,
        );
        let b = mk(&mut app, "", 50.0);
        assert!(app.link_frames(a, b));
        assert_eq!(app.next_frame(a), Some(b), "{}", app.status);
        app.select(vec![a]);
        // Twice as tall from the top: more lines of the chain's text.
        let t = Affine::translate((0.0, 100.0))
            * Affine::scale_non_uniform(1.0, 2.0)
            * Affine::translate((0.0, -100.0));
        let live = app.live_frame(a, t).expect("flowed");
        assert!(!live.overflow);
        app.transform_selection(t);
        let s = app.doc().find_shape(a).expect("a").clone();
        let ShapeKind::Text {
            spans,
            frame,
            align,
            para,
            ..
        } = &s.kind
        else {
            panic!("text");
        };
        let after = tracedraw_text::fonts().layout(&TextRequest {
            spans,
            frame: *frame,
            align: *align,
            para,
            on_path: None,
        });
        assert_eq!(live.lines, shown_lines(&after));
        assert_eq!(live.outline, s.page_path());
    }

    #[test]
    fn a_frame_too_small_for_its_text_overflows() {
        let mut app = App::headless();
        let id = paragraph(&mut app);
        let s = app.doc().shape(id).expect("shape").1.clone();
        assert!(!app.text_overflows(&s));
        // A tenth of the height: the words no longer fit.
        app.transform_selection(Affine::scale_non_uniform(1.0, 0.1));
        let s = app.doc().shape(id).expect("shape").1.clone();
        assert!(app.text_overflows(&s));
        assert!(app.shown_text_frames(None)[0].overflow);
    }

    #[test]
    fn the_frame_being_edited_is_drawn_black_and_a_selected_one_blue() {
        let mut app = App::headless();
        app.settings.autocorrect.enabled = false;
        tracedraw_text::install();
        app.start_text(Point::new(20.0, 20.0), Some(Size::new(60.0, 30.0)));
        app.text_insert("Words");
        let f = app.shown_text_frames(None);
        assert!(f[0].editing && f[0].chosen);
        assert_eq!(crate::canvas::text_frame_color(&f[0]), egui::Color32::BLACK);
        let id = app.selection[0];
        app.finish_text();
        app.select(vec![id]);
        let f = app.shown_text_frames(None);
        assert!(!f[0].editing && f[0].chosen);
        assert_eq!(
            crate::canvas::text_frame_color(&f[0]),
            crate::theme::Tokens::SELECTION
        );
        // The selected characters' shade is a see-through grey.
        let [r, g, b, a] = crate::canvas::TEXT_SELECTION.to_array();
        assert!(r == g && g == b && a > 0 && a < 255);
        // Plain to see on the white page: a mid grey there (#632).
        let on_white = 255.0 * (1.0 - a as f32 / 255.0) + r as f32;
        assert!(on_white < 170.0, "{on_white}");
    }

    #[test]
    fn frames_show_dashed_and_follow_a_scale_being_dragged() {
        let mut app = App::headless();
        let id = paragraph(&mut app);
        assert!(app.settings.show_text_frames);
        assert_eq!(app.shown_text_frames(None).len(), 1);
        let t = Affine::scale_non_uniform(1.5, 1.0);
        let c = app.shown_text_frames(Some(t));
        assert!((c[0].corners[1].x - 120.0).abs() < 1e-6, "{c:?}");
        assert!(c[0].chosen && !c[0].overflow);
        // Off: only the selected text's frame.
        app.settings.show_text_frames = false;
        assert_eq!(app.shown_text_frames(None).len(), 1);
        app.select(Vec::new());
        assert!(app.shown_text_frames(None).is_empty());
        assert!(app.doc().shape(id).is_ok());
    }
}
