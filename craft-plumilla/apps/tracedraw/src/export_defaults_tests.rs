use super::App;
use crate::ui::dialogs::ExportState;
use tracedraw_core::document::PrimaryColorMode;

#[test]
fn a_first_jpeg_uses_centimetres_and_the_document_colour_mode() {
    let mut app = App::headless();
    app.settings.formats.remember_last = false;
    for (mode, expected) in [(PrimaryColorMode::Rgb, 0), (PrimaryColorMode::Cmyk, 1)] {
        let mut doc = app.doc().clone();
        doc.metadata.primary_color_mode = mode;
        app.engine.replace(doc);
        let mut state = ExportState { remember: true, size_unit: 0, ..Default::default() };
        app.recall_export_format(&mut state, 3);
        assert_eq!((state.size_unit, state.color_mode), (2, expected));
    }
}

#[test]
fn a_remembered_jpeg_keeps_explicit_pixels_and_colour_choices() {
    let mut app = App::headless();
    app.settings.formats.remember_last = false;
    let saved = ExportState { remember: true, format: 3, size_unit: 0, color_mode: 2, ..Default::default() };
    app.keep_export_choices(&saved);
    let mut state = ExportState { remember: true, size_unit: 1, color_mode: 0, ..Default::default() };
    app.recall_export_format(&mut state, 3);
    assert_eq!((state.size_unit, state.color_mode), (0, 2));
}

#[test]
fn a_temporary_web_export_keeps_its_pixel_units() {
    let app = App::headless();
    let mut state = ExportState { remember: false, size_unit: 0, color_mode: 0, ..Default::default() };
    app.recall_export_format(&mut state, 3);
    assert_eq!((state.size_unit, state.color_mode), (0, 0));
}
