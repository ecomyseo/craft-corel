use super::*;
use crate::settings::ColorPrefs;

fn distinct_screen_intents_profile() -> Vec<u8> {
    use tracedraw_core::icc::test_profiles as tp;
    fn input_tag(bytes: &[u8]) -> Vec<u8> {
        let count = u32::from_be_bytes(bytes[128..132].try_into().unwrap()) as usize;
        let entry = bytes[132..132 + count * 12]
            .chunks_exact(12)
            .find(|entry| &entry[..4] == b"A2B0")
            .expect("synthetic profile has an input table");
        let offset = u32::from_be_bytes(entry[4..8].try_into().unwrap()) as usize;
        let size = u32::from_be_bytes(entry[8..12].try_into().unwrap()) as usize;
        bytes[offset..offset + size].to_vec()
    }
    tp::Builder::new(b"prtr", b"CMYK", b"Lab ", 4)
        .tag(b"A2B1", input_tag(&tp::intent_cmyk_profile(false)))
        .tag(b"A2B2", input_tag(&tp::intent_cmyk_profile(true)))
        .build()
}

#[test]
fn undo_redo_refreshes_screen_intent_with_the_same_carried_profile_arc() {
    use tracedraw_core::{Document, DocumentProfile, Rect};
    let mut doc = Document::default();
    let profile_bytes = distinct_screen_intents_profile();
    doc.cmyk_profile = Some(DocumentProfile::new(
        "Intent fixture",
        profile_bytes.clone(),
    ));
    doc.metadata.rendering_intent = "Relative colorimetric".into();
    let paint = Color::Cmyk {
        c: 0.2,
        m: 0.4,
        y: 0.6,
        k: 0.1,
    };
    let area = Rect::new(0.0, 0.0, 10.0, 10.0);
    let mut shape = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Rect {
            rect: area,
            radius: 0.0,
            corners: None,
        },
    );
    shape.fill = Fill::Solid(paint);
    shape.stroke = None;
    doc.pages[0].layers[0].shapes.push(shape);
    let carried = doc.cmyk_profile.as_ref().unwrap().data.clone();
    let mut app = App::headless();
    app.settings.color.black_point_compensation = false;
    app.engine.replace(doc);
    app.apply_color_settings();
    let installed = tracedraw_core::color::engine::icc().unwrap();
    app.sync_document_colors();
    assert!(
        std::sync::Arc::ptr_eq(&installed, &tracedraw_core::color::engine::icc().unwrap(),),
        "an unchanged document must reuse the installed screen engine"
    );
    let prefs = app.settings.color.clone();
    let pixel = |app: &App| {
        let image =
            tracedraw_render::render_area_image(app.doc(), app.doc().pages[0].id, area, 72.0)
                .expect("render a CMYK swatch through the installed screen engine");
        let offset = ((image.height() / 2 * image.width() + image.width() / 2) * 4) as usize;
        <[u8; 4]>::try_from(&image.data()[offset..offset + 4]).unwrap()
    };
    let expected = |intent| {
        let engine = tracedraw_core::color::IccEngine::new(
            None,
            Some(Profile::parse(&profile_bytes).unwrap()),
            intent,
            false,
        );
        let rgb = engine.to_rgb_f32(paint).expect("CMYK input transform");
        [
            (rgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (rgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (rgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
            255,
        ]
    };
    let relative = expected(Intent::RelativeColorimetric);
    let saturation = expected(Intent::Saturation);
    assert_ne!(
        relative, saturation,
        "the fixture must distinguish screen intents"
    );
    assert_eq!(pixel(&app), relative);
    let mut metadata = app.doc().metadata.clone();
    metadata.rendering_intent = "Saturation".into();
    app.engine
        .run_with_label(&Command::SetMetadata { metadata }, "Color Management")
        .unwrap();
    app.apply_color_settings();
    let changed = app.doc().clone();
    assert_eq!(pixel(&app), saturation);
    assert!(std::sync::Arc::ptr_eq(
        &carried,
        &app.doc().cmyk_profile.as_ref().unwrap().data
    ));
    app.undo();
    assert_eq!(app.doc().metadata.rendering_intent, "Relative colorimetric");
    assert!(std::sync::Arc::ptr_eq(
        &carried,
        &app.doc().cmyk_profile.as_ref().unwrap().data
    ));
    assert_eq!(
        pixel(&app),
        relative,
        "undo must refresh the screen despite the same profile Arc"
    );
    app.redo();
    assert_eq!(app.doc(), &changed);
    assert!(std::sync::Arc::ptr_eq(
        &carried,
        &app.doc().cmyk_profile.as_ref().unwrap().data
    ));
    assert_eq!(
        pixel(&app),
        saturation,
        "redo must refresh the screen despite the same profile Arc"
    );
    assert_eq!(app.settings.color, prefs);
}

#[test]
fn newly_created_drawings_keep_the_effective_intent_after_cdr_roundtrip() {
    let mut failures = Vec::new();
    for route in ["direct", "dialog"] {
        let mut app = App::headless();
        app.settings.color.intent = "Saturation".into();
        if route == "direct" {
            app.new_document();
        } else {
            let mut state = crate::new_document::NewDocState::from_settings(
                &app.settings.new_document,
                "Intent drawing".into(),
            );
            state.found = Some(Vec::new());
            crate::new_document::create(&mut app, &state);
        }
        let prefs = app.settings.color.clone();
        let original = app.doc().clone();
        let before = Intent::from_name(&original.metadata.rendering_intent)
            .or_else(|| Intent::from_name(&prefs.intent))
            .unwrap();
        let bytes = tracedraw_cdr::write::document_to_cdr_version(&original, 21);
        let (reopened, _) = tracedraw_cdr::open_bytes(&bytes, "new drawing intent").unwrap();
        let after = Intent::from_name(&reopened.metadata.rendering_intent).unwrap();
        eprintln!(
            "{route}: stored={:?}, effective={before:?}, reopened={after:?}",
            original.metadata.rendering_intent
        );
        assert_eq!(before, Intent::Saturation);
        if before != after {
            failures.push(route);
        }
        assert_eq!(app.doc(), &original);
        assert_eq!(app.settings.color, prefs);
    }
    assert!(
        failures.is_empty(),
        "new drawings changed intent after CDR roundtrip: {failures:?}"
    );
}

fn check_screen_intent(document_intent: &str, preference_intent: &str, expected: Intent) {
    for bpc in [false, true] {
        let (mut doc, _, _, original_samples) = tests::bitmap_fixture();
        doc.metadata.rendering_intent = document_intent.into();
        let color = ColorPrefs {
            intent: preference_intent.into(),
            black_point_compensation: bpc,
            ..Default::default()
        };
        let original_doc = doc.clone();
        let original_color = color.clone();
        let prepared = prepare_screen_colors(&doc, &color);
        let engine = prepared
            .icc
            .expect("embedded CMYK profile enables calibration");
        assert_eq!(
            engine.intent(),
            expected,
            "{document_intent:?}/{preference_intent}"
        );
        assert_eq!(engine.black_point_compensation(), bpc);
        assert_eq!(prepared.map_gray_to_black, color.map_gray_to_black);
        assert!(prepared.profile_key.is_some());
        assert_eq!(doc, original_doc);
        assert_eq!(color, original_color);
        for shape in tests::collect_bitmaps(&doc) {
            assert_eq!(tests::decoded_samples(shape), original_samples);
        }
    }
}

#[test]
fn screen_intent_prefers_the_document_in_both_directions_without_changing_values() {
    check_screen_intent(
        "Relative colorimetric",
        "Saturation",
        Intent::RelativeColorimetric,
    );
    check_screen_intent("Saturation", "Relative colorimetric", Intent::Saturation);
    check_screen_intent("Perceptual", "Saturation", Intent::Perceptual);
    check_screen_intent(
        "Absolute colorimetric",
        "Saturation",
        Intent::AbsoluteColorimetric,
    );
}

#[test]
fn screen_intent_falls_back_for_empty_or_unknown_document_metadata() {
    for absent in ["", " ", "Unknown application intent"] {
        check_screen_intent(absent, "Saturation", Intent::Saturation);
        check_screen_intent(
            absent,
            "Relative colorimetric",
            Intent::RelativeColorimetric,
        );
    }
    check_screen_intent("", "Unknown preference intent", Intent::default());
}

#[test]
fn cdr_roundtrip_calibration_uses_the_drawing_intent_and_keeps_preference_bpc() {
    for (document_intent, preference_intent, expected) in [
        (
            "Relative colorimetric",
            "Saturation",
            Intent::RelativeColorimetric,
        ),
        ("Saturation", "Relative colorimetric", Intent::Saturation),
    ] {
        let (mut source, _, _, _) = tests::bitmap_fixture();
        source.metadata.rendering_intent = document_intent.into();
        let original_source = source.clone();
        let bytes = tracedraw_cdr::write::document_to_cdr_version(&source, 21);
        let (doc, _) = tracedraw_cdr::open_bytes(&bytes, "screen intent integration")
            .expect("small native CDR opens");
        assert_eq!(source, original_source);
        assert_eq!(doc.metadata.rendering_intent, document_intent);
        let original_doc = doc.clone();
        for bpc in [false, true] {
            let color = ColorPrefs {
                intent: preference_intent.into(),
                black_point_compensation: bpc,
                ..Default::default()
            };
            let original_color = color.clone();
            let prepared = prepare_screen_colors(&doc, &color);
            let engine = prepared
                .icc
                .expect("CDR preserves its embedded CMYK profile");
            assert_eq!(engine.intent(), expected);
            assert_eq!(engine.black_point_compensation(), bpc);
            assert_eq!(doc, original_doc);
            assert_eq!(color, original_color);
        }
    }
}
