//! Import PDF: how a PDF's (or AI file's) text is read in, asked when one
//! is opened or imported (#386): as editable text in the fonts installed
//! here, or as curves drawn from the fonts the file embeds, which keeps
//! its look exactly.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::ui::dialogs::{ok_cancel, window};
use crate::ui::tips::RichTip;
use egui::Context;
use std::path::PathBuf;

/// The dialog's state.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfTextState {
    pub path: PathBuf,
    /// Import into the drawing (File > Import) rather than open.
    pub import: bool,
    pub curves: bool,
    /// Reduce very large pictures (#1835).
    pub reduce_pictures: bool,
    /// Keep the choice and stop asking (Options > File Formats).
    pub dont_ask: bool,
}

impl App {
    /// Before a PDF is read: ask how to read its text, when the
    /// preferences say so and it has not been asked for this file yet.
    /// True when the dialog was opened (the caller stops there).
    pub fn ask_pdf_text(&mut self, path: &std::path::Path, import: bool) -> bool {
        if !self.settings.formats.pdf_ask_text || std::mem::take(&mut self.pdf_text_chosen) {
            return false;
        }
        self.dialog = crate::ui::dialogs::Dialog::PdfText(PdfTextState {
            path: path.to_path_buf(),
            import,
            curves: self.settings.formats.pdf_text_as_curves,
            reduce_pictures: self.settings.formats.pdf_reduce_pictures,
            dont_ask: false,
        });
        true
    }

    /// The import options the preferences give. Pictures come in at
    /// their own size up to the memory guard, unless Reduce very large
    /// pictures is on; the browser and Android always reduce them, having
    /// less memory to give (#1835).
    pub fn pdf_options(&self) -> tracedraw_io::pdf_import::ImportOptions {
        use tracedraw_io::pdf_import::{FULL_PICTURE_PIXELS, REDUCED_PICTURE_PIXELS};
        let reduce = self.settings.formats.pdf_reduce_pictures || always_reduce_pictures();
        tracedraw_io::pdf_import::ImportOptions {
            text_as_curves: self.settings.formats.pdf_text_as_curves,
            max_picture_pixels: if reduce {
                REDUCED_PICTURE_PIXELS
            } else {
                FULL_PICTURE_PIXELS
            },
        }
    }
}

/// The browser and Android reduce very large pictures whatever the
/// choice (#1835).
pub fn always_reduce_pictures() -> bool {
    crate::files::WEB || crate::files::ANDROID
}

/// The Reduce very large pictures check box, ticked and greyed where
/// pictures are always reduced.
pub fn reduce_pictures_checkbox(ui: &mut egui::Ui, reduce: &mut bool) {
    if always_reduce_pictures() {
        let mut on = true;
        ui.add_enabled(
            false,
            egui::Checkbox::new(&mut on, tr("pdf_import.reduce_pictures")),
        )
        .rich_tip(crate::ui::tips::pair(
            "pdf_import.reduce_pictures",
            "pdf_import.reduce_pictures_always_tip",
        ));
    } else {
        ui.checkbox(reduce, tr("pdf_import.reduce_pictures"))
            .rich_tip(crate::ui::tips::pair(
                "pdf_import.reduce_pictures",
                "pdf_import.reduce_pictures_tip",
            ));
    }
}

/// Text whose font the file does not embed, left as text by the import:
/// as curves too, in the font that stands in for it here.
pub fn remaining_text_to_curves(pages: &mut [tracedraw_io::pdf_import::ImportedPage]) {
    let mut ids = tracedraw_core::id::IdSource::default();
    let mut next = || ids.shape();
    for p in pages {
        for s in &mut p.shapes {
            to_curves(s, &mut next);
        }
    }
}

fn to_curves(s: &mut tracedraw_core::Shape, next: &mut dyn FnMut() -> tracedraw_core::ShapeId) {
    use tracedraw_core::ShapeKind;
    match &mut s.kind {
        ShapeKind::Text { .. } => {
            if let Some(kind) = s.curves_kind(&[], next) {
                s.kind = kind;
            }
        }
        ShapeKind::Group { children } => {
            for c in children {
                to_curves(c, next);
            }
        }
        ShapeKind::ClipFrame { contents, .. } => {
            for c in contents {
                to_curves(c, next);
            }
        }
        _ => {}
    }
}

pub fn dialog(app: &mut App, ctx: &Context, st: &mut PdfTextState, close: &mut bool) {
    window(ctx, tr("pdf_import.title")).show(ctx, |ui| {
        ui.set_max_width(420.0);
        let name = st
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        ui.label(trf("pdf_import.prompt", &[("name", &name)]));
        ui.add_space(6.0);
        ui.label(tr("pdf_import.text_as"));
        ui.radio_value(&mut st.curves, false, tr("pdf_import.text"))
            .rich_tip(tr("pdf_import.text_tip"));
        ui.radio_value(&mut st.curves, true, tr("pdf_import.curves"))
            .rich_tip(tr("pdf_import.curves_tip"));
        ui.add_space(4.0);
        reduce_pictures_checkbox(ui, &mut st.reduce_pictures);
        ui.add_space(4.0);
        ui.checkbox(&mut st.dont_ask, tr("pdf_import.dont_ask"));
        if ok_cancel(ui, close) {
            app.settings.formats.pdf_text_as_curves = st.curves;
            app.settings.formats.pdf_reduce_pictures = st.reduce_pictures;
            if st.dont_ask {
                app.settings.formats.pdf_ask_text = false;
            }
            app.pdf_text_chosen = true;
            let path = st.path.clone();
            if st.import {
                app.import_pdf(&path);
            } else {
                app.open_path(path);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Opening a PDF asks how to read its text once; the answer is kept,
    /// and Do not ask again stops the question.
    #[test]
    fn opening_a_pdf_asks_how_to_read_its_text() {
        let mut app = App::headless();
        app.settings.formats.pdf_ask_text = true;
        let p = std::path::Path::new("x.pdf");
        assert!(app.ask_pdf_text(p, false));
        assert!(matches!(
            app.dialog,
            crate::ui::dialogs::Dialog::PdfText(PdfTextState { import: false, .. })
        ));
        // Answered: the next read goes through without asking.
        app.pdf_text_chosen = true;
        assert!(!app.ask_pdf_text(p, false));
        assert!(app.ask_pdf_text(p, true));
        app.settings.formats.pdf_ask_text = false;
        assert!(!app.ask_pdf_text(p, true));
        app.settings.formats.pdf_text_as_curves = true;
        assert!(app.pdf_options().text_as_curves);
    }

    /// Pictures come in at their own size on the desktop unless Reduce
    /// very large pictures is ticked, which is off by default (#1835).
    #[test]
    fn pictures_are_reduced_only_when_asked_on_the_desktop() {
        use tracedraw_io::pdf_import::{FULL_PICTURE_PIXELS, REDUCED_PICTURE_PIXELS};
        let mut app = App::headless();
        assert!(!app.settings.formats.pdf_reduce_pictures);
        assert!(!always_reduce_pictures());
        assert_eq!(app.pdf_options().max_picture_pixels, FULL_PICTURE_PIXELS);
        app.settings.formats.pdf_reduce_pictures = true;
        assert_eq!(app.pdf_options().max_picture_pixels, REDUCED_PICTURE_PIXELS);
    }

    /// A picture that came in smaller is told after opening, in a message
    /// of its own, whatever the warnings say (#1835).
    #[test]
    fn a_shrunk_picture_is_told_after_opening() {
        use crate::ui::dialogs::Dialog;
        let mut app = App::headless();
        app.settings.warnings.unread_objects = false;
        app.settings.warnings.missing_fonts = false;
        app.pdf_shrunk = vec![tracedraw_io::pdf_import::ShrunkPicture {
            page: 0,
            width: 11_160,
            height: 14_794,
            new_width: 5_580,
            new_height: 7_397,
        }];
        app.show_open_notes("test_file.pdf".into(), 0);
        let Dialog::OpenNotes(notes) = &app.dialog else {
            panic!("no message: {:?}", app.dialog);
        };
        let text = notes.text();
        assert!(
            text.contains("test_file.pdf") && text.contains("11160 x 14794 to 5580 x 7397"),
            "{text}"
        );
        assert!(app.pdf_shrunk.is_empty(), "told once");
        // Nothing shrunk: no message.
        app.dialog = Dialog::None;
        app.show_open_notes("other.pdf".into(), 0);
        assert!(matches!(app.dialog, Dialog::None));
    }
}
