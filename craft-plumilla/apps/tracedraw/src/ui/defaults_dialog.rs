//! The question asked when a fill or outline is chosen with nothing
//! selected: which kinds of new objects take it (`defaults.rs`).

use crate::app::App;
use crate::defaults::{kind_key, DefaultChange};
use crate::i18n::tr;
use crate::ui::dialogs::{ok_cancel, window};
use egui::Context;
use tracedraw_core::DefaultKind;

/// The dialog's state: the change and the kinds ticked.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub change: DefaultChange,
    /// Graphic, artistic text, paragraph text.
    pub kinds: [bool; 3],
    /// Use the kinds ticked from now on without asking.
    pub dont_ask: bool,
}

pub fn dialog(app: &mut App, ctx: &Context, st: &mut State, close: &mut bool) {
    window(ctx, tr("defaults.title")).show(ctx, |ui| {
        ui.set_max_width(360.0);
        ui.label(tr("defaults.question"));
        ui.add_space(4.0);
        for (i, kind) in DefaultKind::ALL.iter().enumerate() {
            ui.checkbox(&mut st.kinds[i], tr(kind_key(*kind)));
        }
        ui.add_space(6.0);
        ui.checkbox(&mut st.dont_ask, tr("defaults.dont_ask"));
        if ok_cancel(ui, close) {
            app.settings.defaults.set_kinds(st.kinds);
            if st.dont_ask {
                app.settings.defaults.ask = false;
            }
            app.change_defaults(&st.change, st.kinds);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Pos2, Rect};
    use tracedraw_core::style::Fill;

    #[test]
    fn the_question_draws_and_keeps_the_choice() {
        let mut app = App::headless();
        app.selection.clear();
        app.request_default_change(DefaultChange::Fill(Fill::None));
        let ctx = crate::theme::ui_context();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
            ..Default::default()
        };
        for _ in 0..2 {
            let mut out = ctx.run_ui(input.clone(), |ui| {
                crate::ui::dialogs::show(&mut app, ui.ctx())
            });
            out.textures_delta.clear();
        }
        assert!(matches!(
            app.dialog,
            crate::ui::dialogs::Dialog::ChangeDefaults(_)
        ));
    }
}
