//! The password asked for when a protected PDF (or AI) file is opened or
//! imported and does not open without one.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::ui::dialogs::{ok_cancel, window};
use egui::{Color32, Context, RichText};
use std::path::PathBuf;

const ERROR: Color32 = Color32::from_rgb(0xC4, 0x2B, 0x1C);

/// The dialog's state.
#[derive(Clone, PartialEq)]
pub struct PdfPasswordState {
    pub path: PathBuf,
    /// Import into the drawing (File > Import) rather than open.
    pub import: bool,
    pub password: String,
    /// The last password tried did not open the file.
    pub wrong: bool,
    /// The field has been given the keyboard once.
    pub focused_once: bool,
}

impl std::fmt::Debug for PdfPasswordState {
    // The password stays out of any log.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfPasswordState")
            .field("path", &self.path)
            .field("import", &self.import)
            .field("wrong", &self.wrong)
            .finish_non_exhaustive()
    }
}

impl App {
    /// Ask for the password of the protected PDF `path`; `wrong` when the
    /// one given did not open it.
    pub fn ask_pdf_password(&mut self, path: PathBuf, import: bool, wrong: bool) {
        let name = file_name(&path);
        self.status = trf("status.pdf_needs_password", &[("name", &name)]);
        self.dialog = crate::ui::dialogs::Dialog::PdfPassword(PdfPasswordState {
            path,
            import,
            password: String::new(),
            wrong,
            focused_once: false,
        });
    }
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn dialog(app: &mut App, ctx: &Context, st: &mut PdfPasswordState, close: &mut bool) {
    window(ctx, tr("pdf_password.title")).show(ctx, |ui| {
        ui.set_max_width(380.0);
        ui.label(trf(
            "pdf_password.prompt",
            &[("name", &file_name(&st.path))],
        ));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(tr("publish.password"));
            let field = ui.add_sized(
                [240.0, 20.0],
                egui::TextEdit::singleline(&mut st.password)
                    .password(true)
                    .id(egui::Id::new("pdf_password_field")),
            );
            // The field takes the keyboard once the window has it (asking
            // before the window is shown can stall some X11 setups).
            if !st.focused_once && ui.input(|i| i.focused) {
                field.request_focus();
                st.focused_once = true;
            }
        });
        if st.wrong {
            ui.label(RichText::new(tr("pdf_password.wrong")).color(ERROR));
        }
        if ok_cancel(ui, close) {
            let path = st.path.clone();
            let password = std::mem::take(&mut st.password);
            if st.import {
                app.import_pdf_with_password(&path, &password);
            } else {
                app.open_path_with_password(path, &password);
            }
        } else if *close {
            app.status = trf("status.pdf_not_opened", &[("name", &file_name(&st.path))]);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{vec2, Pos2, Rect};

    /// A drawing published with a password to open it.
    fn protected_pdf(dir: &std::path::Path) -> PathBuf {
        let app = App::headless();
        let mut st = crate::ui::publish_pdf::PublishState::new(&app);
        st.lock_open = true;
        st.open_password = "abc".into();
        let path = dir.join("locked.pdf");
        crate::ui::publish_pdf::publish(&app, &st, &path).expect("published");
        path
    }

    #[test]
    fn protected_pdfs_ask_for_their_password() {
        let dir = std::env::temp_dir().join(format!("tracedraw-pdfpw-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = protected_pdf(&dir);
        let mut app = App::headless();
        let docs = app.docs.len();
        app.open_path(path.clone());
        let crate::ui::dialogs::Dialog::PdfPassword(st) = &app.dialog else {
            panic!("no password asked: {:?}", app.dialog);
        };
        assert!(!st.wrong && !st.import);
        assert_eq!(app.docs.len(), docs);
        // A wrong password asks again, saying so.
        app.open_path_with_password(path.clone(), "nope");
        let crate::ui::dialogs::Dialog::PdfPassword(st) = &app.dialog else {
            panic!("no second question");
        };
        assert!(st.wrong);
        // The dialog draws.
        let ctx = crate::theme::ui_context();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
            ..Default::default()
        };
        for _ in 0..2 {
            let mut out = ctx.run_ui(input.clone(), |ui| {
                crate::ui::dialogs::show(&mut app, ui.ctx());
            });
            out.textures_delta.clear();
        }
        // The right one opens it.
        app.dialog = crate::ui::dialogs::Dialog::None;
        app.open_path_with_password(path.clone(), "abc");
        assert!(matches!(app.dialog, crate::ui::dialogs::Dialog::None));
        assert_eq!(app.docs.len(), docs + 1);
        // Importing asks the same way.
        app.import_pdf(&path);
        let crate::ui::dialogs::Dialog::PdfPassword(st) = &app.dialog else {
            panic!("no password asked on import");
        };
        assert!(st.import);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
