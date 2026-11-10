//! Object > Rollover: make a rollover of the selection, edit one of its
//! states (Normal, Over, Down), finish editing, break it apart, and the
//! live preview that shows a rollover's Over state while the pointer is
//! over it and its Down state while it is pressed (see `docs/behavior/
//! rollovers.md`).

use crate::app::App;
use tracedraw_core::{Command, Document, ShapeId, ShapeKind, ROLLOVER_STATES};

/// The i18n keys of the states, in order.
pub const STATE_KEYS: [&str; ROLLOVER_STATES] =
    ["rollover.state_normal", "rollover.state_over", "rollover.state_down"];

impl App {
    /// The rollover selected: the one object selected, when it is one.
    pub fn selected_rollover(&self) -> Option<ShapeId> {
        let [id] = self.selection.as_slice() else {
            return None;
        };
        self.doc()
            .find_shape(*id)
            .filter(|s| s.rollover.is_some())
            .map(|s| s.id)
    }

    /// The rollover whose states are being edited, while it still is one
    /// in the drawing shown.
    pub fn edited_rollover(&self) -> Option<ShapeId> {
        let id = self.editing_rollover?;
        self.doc()
            .find_shape(id)
            .filter(|s| s.rollover.is_some())
            .map(|s| s.id)
    }

    /// The state the edited rollover shows (0 Normal, 1 Over, 2 Down).
    pub fn edited_rollover_state(&self) -> Option<usize> {
        let id = self.edited_rollover()?;
        let s = self.doc().find_shape(id)?;
        s.rollover.as_ref().map(|r| r.shown_state())
    }

    /// Create Rollover is offered for objects on a layer (not inside a
    /// group), none of them a rollover, while no rollover is edited.
    pub fn can_create_rollover(&self) -> bool {
        self.edited_rollover().is_none()
            && !self.selection.is_empty()
            && self.selection.iter().all(|id| {
                self.doc().locate(*id).is_ok()
                    && self
                        .doc()
                        .find_shape(*id)
                        .is_some_and(|s| s.rollover.is_none())
            })
    }

    /// Object > Rollover > Create Rollover: the selection becomes one
    /// rollover (one undo step), selected.
    pub fn create_rollover(&mut self) {
        if !self.can_create_rollover() {
            return;
        }
        let id = ShapeId(self.doc().ids().peek_next());
        let shapes = self.selection.clone();
        self.run(Command::CreateRollover { shapes });
        if self.doc().find_shape(id).is_some_and(|s| s.rollover.is_some()) {
            self.select(vec![id]);
        }
    }

    /// Object > Rollover > Edit Rollover: the selected rollover's states
    /// are edited; it shows the state chosen, Normal first.
    pub fn edit_rollover(&mut self) {
        if self.edited_rollover().is_some() {
            return;
        }
        let Some(id) = self.selected_rollover() else {
            return;
        };
        self.editing_rollover = Some(id);
        self.rollover_hover = None;
        self.select_rollover_objects(id);
    }

    /// Choose the state of the edited rollover to change: its objects are
    /// shown and selected.
    pub fn set_edited_rollover_state(&mut self, state: usize) {
        let Some(id) = self.edited_rollover() else {
            return;
        };
        if self.edited_rollover_state() != Some(state) {
            self.run(Command::SetRolloverState {
                shape: id,
                state: state.min(ROLLOVER_STATES - 1) as u8,
            });
        }
        self.select_rollover_objects(id);
    }

    /// Object > Rollover > Finish Editing Rollover: Normal shown again and
    /// the rollover selected as a whole.
    pub fn finish_rollover(&mut self) {
        let Some(id) = self.edited_rollover() else {
            self.editing_rollover = None;
            return;
        };
        if self.edited_rollover_state() != Some(0) {
            self.run(Command::SetRolloverState { shape: id, state: 0 });
        }
        self.editing_rollover = None;
        self.select(vec![id]);
    }

    /// Object > Rollover > Break Apart Rollover: each state becomes a
    /// plain group, all of them selected.
    pub fn extract_rollover(&mut self) {
        let Some(id) = self.selected_rollover() else {
            return;
        };
        if self.edited_rollover().is_some() {
            return;
        }
        let first = self.doc().ids().peek_next();
        self.run(Command::ExtractRollover { shape: id });
        let made: Vec<ShapeId> = (first..first + ROLLOVER_STATES as u64)
            .map(ShapeId)
            .filter(|g| self.doc().find_shape(*g).is_some())
            .collect();
        if !made.is_empty() {
            self.select(made);
        }
    }

    /// Object > Rollover > Live Preview of Rollovers.
    pub fn toggle_rollover_live_preview(&mut self) {
        self.rollover_live = !self.rollover_live;
        self.rollover_hover = None;
    }

    /// The rollover objects of the state shown, selected.
    fn select_rollover_objects(&mut self, id: ShapeId) {
        let ids: Vec<ShapeId> = match self.doc().find_shape(id).map(|s| &s.kind) {
            Some(ShapeKind::Group { children }) => children.iter().map(|c| c.id).collect(),
            _ => Vec::new(),
        };
        if !ids.is_empty() {
            self.select(ids);
        }
    }

    /// The live preview: the rollover under the pointer at `p` (page
    /// space), with Over, or Down while `pressed`; `None` when the
    /// preview is off, a rollover is edited or the pointer is not over
    /// one. Kept so the canvas draws that state.
    pub fn update_rollover_hover(&mut self, p: Option<tracedraw_core::Point>, pressed: bool) {
        let next = if !self.rollover_live || self.edited_rollover().is_some() {
            None
        } else {
            p.and_then(|p| self.rollover_at(p))
                .map(|id| (id, if pressed { 2u8 } else { 1u8 }))
        };
        self.rollover_hover = next;
    }

    /// The top rollover whose shown objects cover `p`.
    fn rollover_at(&self, p: tracedraw_core::Point) -> Option<ShapeId> {
        let page = self.doc().page(self.page).ok()?;
        page.layers
            .iter()
            .rev()
            .filter(|l| l.visible)
            .flat_map(|l| l.shapes.iter().rev())
            .filter(|s| s.visible && s.rollover.is_some())
            .find(|s| self.visual_bounds(s).contains(p))
            .map(|s| s.id)
    }

    /// The drawing as the live preview shows it: the rollover under the
    /// pointer in its Over or Down state. Made once per change of the
    /// drawing or of the state previewed, not every frame.
    pub fn rollover_preview_doc(&self) -> Option<std::rc::Rc<Document>> {
        let (id, state) = self.rollover_hover?;
        let rev = self.engine.revision();
        if let Some((r, key, doc)) = self.rollover_doc.borrow().as_ref() {
            if *r == rev && *key == (id, state) {
                return Some(doc.clone());
            }
        }
        let mut doc = self.doc().clone();
        Command::SetRolloverState { shape: id, state }
            .apply(&mut doc)
            .ok()?;
        let doc = std::rc::Rc::new(doc);
        *self.rollover_doc.borrow_mut() = Some((rev, (id, state), doc.clone()));
        Some(doc)
    }

    /// The drawing cannot be closed while one of its rollovers is edited
    /// (finish editing first); the reason goes to the status bar.
    pub fn rollover_blocks_close(&mut self) -> bool {
        if self.edited_rollover().is_some() {
            self.status = crate::i18n::tr("status.rollover_editing");
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::menus::run_command;
    use tracedraw_core::{Color, Fill, Rect};

    const RED: Fill = Fill::Solid(Color::Rgb {
        r: 200.0 / 255.0,
        g: 0.0,
        b: 0.0,
    });

    fn two_boxes() -> (App, egui::Context, ShapeId, ShapeId) {
        crate::i18n::set_language("en");
        let mut app = App::headless();
        let mut boxed = |x: f64, c: Color| {
            app.new_shape_styled(
                ShapeKind::Rect {
                    rect: Rect::new(x, 10.0, x + 30.0, 30.0),
                    radius: 0.0,
                    corners: None,
                },
                |s| s.fill = Fill::Solid(c),
            )
            .expect("a box")
        };
        let a = boxed(10.0, Color::rgb8(200, 0, 0));
        let b = boxed(60.0, Color::rgb8(0, 0, 200));
        let ctx = crate::theme::ui_context();
        // Fonts and styles, as after the first frame.
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        (app, ctx, a, b)
    }

    fn fill_of_state(app: &App, id: ShapeId, state: usize) -> Fill {
        app.doc().find_shape(id).expect("rollover").rollover_state(state).expect("state")[0]
            .fill
            .clone()
    }

    /// Object > Rollover through its menu rows: Create, Edit, a state,
    /// a change to that state only, Finish Editing; each row enabled only
    /// when it applies.
    #[test]
    fn the_rollover_menu_creates_edits_and_finishes() {
        let (mut app, ctx, a, b) = two_boxes();
        assert!(!run_command(&mut app, &ctx, "menu.object.rollover_create"), "nothing selected");
        app.select(vec![a, b]);
        assert!(!run_command(&mut app, &ctx, "menu.object.rollover_edit"), "not a rollover yet");
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_create"));
        let r = app.selected_rollover().expect("the new rollover is selected");
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_edit"));
        assert_eq!(app.edited_rollover(), Some(r));
        assert_eq!(app.edited_rollover_state(), Some(0));
        assert!(run_command(&mut app, &ctx, "rollover.state_over"));
        assert_eq!(app.edited_rollover_state(), Some(1));
        // The Over objects are selected: a fill changes them only.
        assert_eq!(app.selection.len(), 2);
        assert!(!app.selection.contains(&a));
        let first = app.selection[0];
        app.run(Command::SetFill {
            shapes: vec![first],
            fill: Fill::Solid(Color::rgb8(0, 160, 0)),
        });
        assert!(!run_command(&mut app, &ctx, "menu.object.rollover_create"), "while editing");
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_finish"));
        assert_eq!(app.edited_rollover(), None);
        assert_eq!(app.selection, vec![r]);
        assert_eq!(fill_of_state(&app, r, 0), RED);
        assert_eq!(fill_of_state(&app, r, 1), Fill::Solid(Color::rgb8(0, 160, 0)));
        assert_eq!(fill_of_state(&app, r, 2), RED);
        // The drawing shows Normal again.
        let ShapeKind::Group { children } = &app.doc().find_shape(r).expect("r").kind else {
            panic!("a group");
        };
        assert_eq!(children[0].id, a);
    }

    /// Break Apart Rollover: three plain groups in its place, selected.
    #[test]
    fn breaking_apart_leaves_a_group_a_state() {
        let (mut app, ctx, a, b) = two_boxes();
        app.select(vec![a, b]);
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_create"));
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_extract"));
        assert_eq!(app.selection.len(), 3);
        for id in &app.selection {
            let s = app.doc().find_shape(*id).expect("group");
            assert!(s.rollover.is_none());
            assert!(matches!(s.kind, ShapeKind::Group { .. }));
        }
        app.undo();
        assert!(app.doc().pages[0].layers[0].shapes[0].rollover.is_some());
    }

    /// The drawing is not closed while a rollover is edited, and is once
    /// editing is finished.
    #[test]
    fn a_drawing_with_a_rollover_being_edited_stays_open() {
        let (mut app, ctx, a, b) = two_boxes();
        app.select(vec![a, b]);
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_create"));
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_edit"));
        let docs = app.docs.len();
        app.close_document();
        assert_eq!(app.docs.len(), docs);
        assert!(app.has_document());
        assert_eq!(app.status, crate::i18n::tr("status.rollover_editing"));
        app.finish_rollover();
        assert!(!app.rollover_blocks_close());
    }

    /// Live Preview of Rollovers: Over while the pointer is over the
    /// rollover and Down while pressed, in the picture only; the drawing
    /// and its history do not change.
    #[test]
    fn the_live_preview_shows_over_and_down_without_changing_the_drawing() {
        let (mut app, ctx, a, b) = two_boxes();
        app.select(vec![a, b]);
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_create"));
        let r = app.selected_rollover().expect("rollover");
        let over_first = app.doc().find_shape(r).expect("r").rollover_state(1).expect("over")[0].id;
        let before = app.doc().clone();
        let steps = app.engine.history_labels().0.len();
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_live_preview"));
        assert!(app.rollover_live);
        let inside = tracedraw_core::Point::new(20.0, 20.0);
        app.update_rollover_hover(Some(inside), false);
        assert_eq!(app.rollover_hover, Some((r, 1)));
        let shown = app.rollover_preview_doc().expect("preview");
        let ShapeKind::Group { children } = &shown.find_shape(r).expect("r").kind else {
            panic!("a group");
        };
        assert_eq!(children[0].id, over_first, "Over is drawn");
        app.update_rollover_hover(Some(inside), true);
        assert_eq!(app.rollover_hover, Some((r, 2)));
        app.update_rollover_hover(Some(tracedraw_core::Point::new(150.0, 150.0)), false);
        assert_eq!(app.rollover_hover, None);
        assert!(app.rollover_preview_doc().is_none());
        assert_eq!(app.doc(), &before);
        assert_eq!(app.engine.history_labels().0.len(), steps);
        // Off: no preview anywhere.
        assert!(run_command(&mut app, &ctx, "menu.object.rollover_live_preview"));
        app.update_rollover_hover(Some(inside), false);
        assert_eq!(app.rollover_hover, None);
    }
}
