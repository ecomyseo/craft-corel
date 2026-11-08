//! One undo step for a run of steps from one number field.
//!
//! The mouse wheel over a number field, a click on its spin arrows and a
//! drag on them step the value, and each step applies at once as a
//! command: turning the wheel from 1 to 100 made about a hundred undo
//! steps. The steps one field makes in a row are joined here into one,
//! the way a drag on the canvas is one step: with the engine's groups
//! ([`Engine::join_group`](tracedraw_core::Engine::join_group)), after
//! each frame in which the field stepped. Undo then puts back the value
//! from before the first step, Redo the value where the steps stopped.
//!
//! A step joins the one before when it comes from the same field, to the
//! same selection, with the same kind of command (the same undo name),
//! within [`MERGE_SECONDS`] of the field's last step (or while one drag
//! on its arrows goes on), and nothing else changed the drawing in
//! between: another command, an Undo or Redo, a preview or another
//! drawing ends the run.

use crate::app::App;
use tracedraw_core::id::ShapeId;

/// The longest pause between two steps of a field that still joins them.
pub const MERGE_SECONDS: f64 = 0.8;

/// A run of steps from one field, joined into one undo step.
#[derive(Debug, Clone)]
pub struct FieldRun {
    /// The field (its wheel memory id, see `ui::field::note_step`).
    field: egui::Id,
    /// The drawing's engine (see `Engine::instance`).
    engine: u64,
    /// The undo step before the run, the group's mark.
    before: Option<u64>,
    /// The run's step, on top of the undo stack.
    step: Option<u64>,
    label: &'static str,
    selection: Vec<ShapeId>,
    /// The input time of the run's last step, seconds.
    time: f64,
    /// The last step came from a drag still held.
    held: bool,
}

/// How the history stood when a frame began.
#[derive(Debug, Clone, Copy)]
pub struct FrameStart {
    engine: u64,
    revision: u64,
    last: Option<u64>,
    at_step: bool,
    preview: bool,
}

impl App {
    /// Note how the history stands before a frame's controls run.
    pub fn step_merge_start(&self) -> FrameStart {
        // A widget drawn outside the frame (tests) says nothing about it.
        let _ = crate::ui::field::take_step();
        let e = &self.engine;
        FrameStart {
            engine: e.instance(),
            revision: e.revision(),
            last: e.last_step_id(),
            at_step: e.at_recorded_step(),
            preview: e.in_preview(),
        }
    }

    /// After a frame: the steps a number field made in it join its run
    /// of steps when they continue it, or begin a run. `time` is the
    /// frame's input time in seconds.
    pub fn step_merge_end(&mut self, start: FrameStart, time: f64) {
        let stepped = crate::ui::field::take_step();
        let e = &self.engine;
        let same_engine = e.instance() == start.engine;
        let recorded = same_engine && e.at_recorded_step() && e.last_step_id() != start.last;
        let Some((field, held)) = stepped else {
            // Anything else that changed the drawing ends the run.
            if !same_engine || e.revision() != start.revision {
                self.field_run = None;
            }
            return;
        };
        if !recorded {
            // A step that changed nothing (the value at its limit) keeps
            // the run; any other change ends it.
            if !same_engine || e.revision() != start.revision {
                self.field_run = None;
            }
            return;
        }
        let Some(label) = e.undo_label() else {
            self.field_run = None;
            return;
        };
        if start.preview || e.in_preview() {
            // A preview keeps its own steps apart.
            self.field_run = None;
            return;
        }
        let continues = self.field_run.as_ref().is_some_and(|r| {
            r.field == field
                && r.engine == start.engine
                && start.at_step
                && start.last == r.step
                && r.label == label
                && r.selection == self.selection
                && (time - r.time <= MERGE_SECONDS || (r.held && held))
        });
        let before = match &self.field_run {
            Some(r) if continues => r.before,
            _ => start.last,
        };
        // The frame's steps join the run's (or each other, when a frame
        // made several).
        self.engine.join_group(before, label);
        self.field_run = Some(FieldRun {
            field,
            engine: start.engine,
            before,
            step: self.engine.last_step_id(),
            label,
            selection: self.selection.clone(),
            time,
            held,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Rect;
    use tracedraw_core::ShapeKind;

    /// A drawing window with a rectangle 47 mm wide selected, and the
    /// input clock its frames run on.
    struct Window {
        ctx: egui::Context,
        app: App,
        id: ShapeId,
        time: f64,
    }

    impl Window {
        fn new() -> Self {
            let mut app = App::headless();
            app.settings.autocorrect.enabled = false;
            let id = app
                .new_shape(ShapeKind::Rect {
                    rect: Rect::new(13.0, 10.0, 60.0, 40.0),
                    radius: 0.0,
                    corners: None,
                })
                .expect("rect");
            app.select(vec![id]);
            let mut w = Window {
                ctx: crate::theme::ui_context(),
                app,
                id,
                time: 0.0,
            };
            w.frame(Vec::new());
            w.frame(Vec::new());
            w
        }

        /// One frame, 1/60 s after the last; the texts it drew and where.
        fn frame(&mut self, events: Vec<egui::Event>) -> Vec<(String, egui::Rect)> {
            self.time += 1.0 / 60.0;
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 1000.0),
                )),
                events,
                time: Some(self.time),
                ..Default::default()
            };
            let app = &mut self.app;
            let mut out = self.ctx.run_ui(input, |ui| crate::ui::root(app, ui));
            out.textures_delta.clear();
            out.shapes
                .iter()
                .filter_map(|c| match &c.shape {
                    egui::Shape::Text(t) => {
                        Some((t.galley.text().to_string(), t.visual_bounding_rect()))
                    }
                    _ => None,
                })
                .collect()
        }

        /// The middle of the property bar's field showing `width`.
        fn field_showing(&mut self, value: f64) -> egui::Pos2 {
            let texts = self.frame(Vec::new());
            let want = format!("{value:.0}");
            texts
                .iter()
                .filter(|(t, _)| {
                    t.ends_with(" mm")
                        && t.split(['.', ' '])
                            .next()
                            .is_some_and(|whole| whole == want)
                })
                .map(|(_, r)| r.center())
                .min_by(|a, b| a.y.total_cmp(&b.y))
                .unwrap_or_else(|| panic!("a field showing {want} mm in {texts:?}"))
        }

        /// `n` wheel notches up over the point, a frame each.
        fn notches(&mut self, at: egui::Pos2, n: usize) {
            self.frame(vec![egui::Event::PointerMoved(at)]);
            for _ in 0..n {
                self.frame(vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::vec2(0.0, 1.0),
                    phase: egui::TouchPhase::Move,
                    modifiers: egui::Modifiers::NONE,
                }]);
            }
        }

        fn width(&self) -> f64 {
            self.app
                .doc()
                .find_shape(self.id)
                .map(|s| s.bounds().width())
                .unwrap_or(f64::NAN)
        }

        fn steps(&self) -> usize {
            self.app.engine.history_labels().0.len()
        }
    }

    /// Ten notches of the wheel over the property bar's width field are
    /// one undo step: Undo puts back the width from before the first
    /// notch, Redo the width where the wheel stopped.
    #[test]
    fn a_turn_of_the_wheel_over_the_width_field_is_one_undo_step() {
        let mut w = Window::new();
        let steps = w.steps();
        let at = w.field_showing(47.0);
        w.notches(at, 10);
        assert!((w.width() - 57.0).abs() < 1e-6, "width {}", w.width());
        assert_eq!(w.steps(), steps + 1, "{:?}", w.app.engine.history_labels());
        w.app.engine.undo().expect("undo");
        assert!((w.width() - 47.0).abs() < 1e-6, "width {}", w.width());
        assert_eq!(w.steps(), steps);
        w.app.engine.redo().expect("redo");
        assert!((w.width() - 57.0).abs() < 1e-6, "width {}", w.width());
    }

    /// A pause longer than [`MERGE_SECONDS`] between notches, a notch on
    /// another field, or a command between them makes another step.
    #[test]
    fn a_pause_another_field_or_a_command_ends_the_step() {
        let mut w = Window::new();
        let steps = w.steps();
        let at = w.field_showing(47.0);
        w.notches(at, 3);
        assert_eq!(w.steps(), steps + 1);
        // A pause on the input clock.
        w.time += MERGE_SECONDS + 0.5;
        w.notches(at, 2);
        assert!((w.width() - 52.0).abs() < 1e-6, "width {}", w.width());
        assert_eq!(w.steps(), steps + 2);
        // The height field, at once.
        let h = w.field_showing(30.0);
        w.notches(h, 2);
        assert_eq!(w.steps(), steps + 3);
        // Back on the width field at once: another field came between.
        let at = w.field_showing(52.0);
        w.notches(at, 2);
        assert_eq!(w.steps(), steps + 4);
        // A command from the canvas (a nudge) between notches.
        w.app
            .transform_selection(tracedraw_core::geometry::Affine::translate(
                tracedraw_core::geometry::Vec2::new(1.0, 0.0),
            ));
        assert_eq!(w.steps(), steps + 5);
        w.notches(at, 2);
        assert_eq!(w.steps(), steps + 6);
        assert!((w.width() - 56.0).abs() < 1e-6, "width {}", w.width());
        // Each Undo takes back one of them: the last two notches first.
        w.app.engine.undo().expect("undo");
        assert!((w.width() - 54.0).abs() < 1e-6, "width {}", w.width());
    }
}
