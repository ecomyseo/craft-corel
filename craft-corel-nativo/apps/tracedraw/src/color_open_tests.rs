//! Opening policies keep a clean document and change all retained ink planes.

use super::tests::{bitmap_fixture, collect_bitmaps, decoded_samples, target_file};
use super::*;

fn configured(policy: u8) -> (App, Vec<u8>, Vec<u8>, Vec<u8>, std::path::PathBuf) {
    let (doc, source, target, samples) = bitmap_fixture();
    let (root, found) = target_file(&target);
    let mut app = App::headless();
    app.engine.replace(doc);
    app.settings.color.open_cmyk = policy;
    app.settings.color.cmyk_profile = found.name().to_string();
    app.settings.color.cmyk_profile_path = found.path().unwrap().to_string();
    app.settings.color.warn_open_missing = false;
    app.settings.color.warn_open_mismatch = false;
    app.settings.color.intent = "Saturation".into();
    app.settings.color.black_point_compensation = false;
    app.settings.color.preserve_pure_black = false;
    (app, source, target, samples, root)
}

#[test]
fn keeping_tagged_input_does_not_read_a_missing_default_profile() {
    let (mut app, _, _, _, root) = configured(0);
    app.settings.color.cmyk_profile_path = root.join("missing.icc").display().to_string();
    let before = app.doc().clone();
    assert!(app.apply_open_policy().unwrap().is_none());
    assert_eq!(app.doc(), &before);
    assert_eq!(app.engine.history_labels(), (vec![], vec![]));
    assert!(!app.engine.is_dirty());
}

#[test]
fn assign_and_convert_open_all_picture_containers_without_edit_history() {
    for policy in [1, 2] {
        let (mut app, source, target, samples, _) = configured(policy);
        let mut expected = samples.clone();
        if policy == 2 {
            let transform = Transform::new(
                &Profile::parse(&source).unwrap(),
                &Profile::parse(&target).unwrap(),
                Intent::Saturation,
                false,
            )
            .unwrap();
            for pixel in expected.chunks_exact_mut(5) {
                let output = transform.transform(
                    &pixel[..4]
                        .iter()
                        .map(|v| f32::from(*v) / 255.0)
                        .collect::<Vec<_>>(),
                );
                for (byte, value) in pixel[..4].iter_mut().zip(output) {
                    *byte = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
            assert_ne!(expected, samples);
        }
        assert!(app.apply_open_policy().unwrap().is_none());
        assert_eq!(
            app.doc().cmyk_profile.as_ref().unwrap().data.as_slice(),
            target
        );
        let bitmaps = collect_bitmaps(app.doc());
        assert_eq!(bitmaps.len(), 5);
        for bitmap in bitmaps {
            assert_eq!(decoded_samples(bitmap), expected);
            assert_eq!(
                bitmap
                    .picture_inks
                    .as_ref()
                    .unwrap()
                    .profile
                    .as_ref()
                    .unwrap()
                    .data
                    .as_slice(),
                target
            );
        }
        assert_eq!(app.engine.history_labels(), (vec![], vec![]));
        assert!(!app.engine.is_dirty());
    }
}

#[test]
fn a_late_invalid_picture_or_default_leaves_the_opened_data_untouched() {
    let (mut app, _, _, _, root) = configured(1);
    let mut broken = app.doc().clone();
    broken.symbols[0].shape.picture_inks.as_mut().unwrap().png = vec![1, 2, 3].into();
    app.engine.replace(broken);
    let before = app.doc().clone();
    assert!(app.apply_open_policy().is_err());
    assert_eq!(app.doc(), &before);
    assert!(!app.engine.is_dirty());
    let (valid, _, _, _) = bitmap_fixture();
    app.engine.replace(valid);
    let before = app.doc().clone();
    std::fs::write(root.join("target.icc"), b"bad ICC").unwrap();
    assert!(app.apply_open_policy().is_err());
    assert_eq!(app.doc(), &before);
    assert_eq!(app.engine.history_labels(), (vec![], vec![]));
}

#[test]
fn equal_names_with_different_profile_bytes_still_warn_when_kept() {
    let (mut app, _, _, _, _) = configured(0);
    app.settings.color.cmyk_profile = app.doc().cmyk_profile.as_ref().unwrap().name.clone();
    app.settings.color.warn_open_mismatch = true;
    let before = app.doc().clone();
    assert!(app.apply_open_policy().unwrap().is_some());
    assert_eq!(app.doc(), &before);
    assert!(!app.engine.is_dirty());
}

#[test]
fn an_unknown_open_policy_is_rejected_without_changes() {
    let (mut app, _, _, _, _) = configured(255);
    let before = app.doc().clone();
    assert!(app.apply_open_policy().is_err());
    assert_eq!(app.doc(), &before);
    assert_eq!(app.engine.history_labels(), (vec![], vec![]));
}

#[test]
fn opening_tagged_tiff_applies_assignment_and_preserves_original_file() {
    let (mut app, source, target, samples, root) = configured(1);
    let bytes =
        tracedraw_io::cmyk_bitmap::cmyka_tiff(2, 2, &samples, 300.0, Some(&source)).unwrap();
    let input = root.join("owned-source.tif");
    std::fs::write(&input, &bytes).unwrap();
    app.try_open_path(input.clone()).unwrap();
    assert_eq!(
        app.doc().cmyk_profile.as_ref().unwrap().data.as_slice(),
        target
    );
    let bitmaps = collect_bitmaps(app.doc());
    assert_eq!(bitmaps.len(), 1);
    assert_eq!(decoded_samples(bitmaps[0]), samples);
    assert_eq!(std::fs::read(input).unwrap(), bytes);
    assert!(app.engine.history_labels().0.is_empty());
    assert!(!app.engine.is_dirty());
}

#[test]
fn untagged_tiff_is_assigned_default_inks_for_each_open_policy_and_keeps_notice() {
    for policy in [0, 1, 2] {
        let (mut app, _, target, samples, root) = configured(policy);
        app.settings.color.warn_open_missing = true;
        let bytes = tracedraw_io::cmyk_bitmap::cmyka_tiff(2, 2, &samples, 300.0, None).unwrap();
        let input = root.join("untagged-source.tif");
        std::fs::write(&input, &bytes).unwrap();
        app.try_open_path(input.clone()).unwrap();
        assert_eq!(
            app.doc().cmyk_profile.as_ref().unwrap().data.as_slice(),
            target
        );
        let bitmaps = collect_bitmaps(app.doc());
        assert_eq!(bitmaps.len(), 1);
        assert_eq!(decoded_samples(bitmaps[0]), samples);
        assert!(app
            .status
            .contains(&crate::i18n::tr("color_mgmt.warn_missing")));
        assert_eq!(std::fs::read(input).unwrap(), bytes);
        assert!(!app.engine.is_dirty());
        assert!(app.engine.history_labels().0.is_empty());
    }
}

#[test]
fn conversion_to_a_missing_icc_reports_failure_without_relabeling_original_inks() {
    let (mut app, _, _, _, _) = configured(2);
    app.settings.color.cmyk_profile_path.clear();
    let before = app.doc().clone();
    assert!(app.apply_open_policy().is_err());
    assert_eq!(app.doc(), &before);
    assert!(!app.engine.is_dirty());
    assert_eq!(app.engine.history_labels(), (vec![], vec![]));
}

#[test]
fn opening_generated_cdr_uses_the_same_clean_assignment_policy() {
    let (mut app, source, target, samples, root) = configured(1);
    let tiff = tracedraw_io::cmyk_bitmap::cmyka_tiff(2, 2, &samples, 300.0, Some(&source)).unwrap();
    let doc = tracedraw_io::raster::document_from_bytes(&tiff, "Owned CDR fixture").unwrap();
    let bytes = tracedraw_cdr::write::document_to_cdr_version(&doc, 27);
    let input = root.join("owned-generated.cdr");
    std::fs::write(&input, &bytes).unwrap();
    app.try_open_path(input.clone()).unwrap();
    assert_eq!(
        app.doc().cmyk_profile.as_ref().unwrap().data.as_slice(),
        target
    );
    let bitmaps = collect_bitmaps(app.doc());
    assert_eq!(bitmaps.len(), 1);
    assert_eq!(decoded_samples(bitmaps[0]), samples);
    assert_eq!(std::fs::read(input).unwrap(), bytes);
    assert_eq!(app.engine.history_labels(), (vec![], vec![]));
    assert!(!app.engine.is_dirty());
}

#[test]
fn a_failed_default_profile_still_opens_tagged_source_without_losing_its_inks() {
    let (mut app, source, _, samples, root) = configured(1);
    let bytes =
        tracedraw_io::cmyk_bitmap::cmyka_tiff(2, 2, &samples, 300.0, Some(&source)).unwrap();
    let input = root.join("tagged-invalid-default.tif");
    std::fs::write(&input, &bytes).unwrap();
    std::fs::write(root.join("target.icc"), b"bad ICC").unwrap();
    let error = named_profile(
        &app.settings.color.cmyk_profile,
        &app.settings.color.cmyk_profile_path,
    )
    .unwrap_err();
    let notice = crate::i18n::trf("color_mgmt.apply_failed", &[("e", &error)]);
    app.try_open_path(input.clone()).unwrap();
    assert_eq!(
        app.doc().cmyk_profile.as_ref().unwrap().data.as_slice(),
        source
    );
    assert_eq!(decoded_samples(collect_bitmaps(app.doc())[0]), samples);
    assert!(app.status.contains(&notice));
    assert_eq!(std::fs::read(input).unwrap(), bytes);
    assert_eq!(app.engine.history_labels(), (vec![], vec![]));
    assert!(!app.engine.is_dirty());
}

#[test]
fn screen_calibration_uses_retained_document_icc_after_default_failure_and_sync() {
    use tracedraw_core::color::{engine, IccEngine};
    for policy in [1, 2] {
        let (mut app, source, _, _, root) = configured(policy);
        app.settings.color.rgb_profile_path.clear();
        app.settings.color.builtin_engine = false;
        std::fs::write(root.join("target.icc"), b"bad ICC").unwrap();
        let before = app.doc().clone();
        assert!(app.apply_open_policy().is_err());
        assert_eq!(app.doc(), &before);
        let local = IccEngine::new(
            None,
            Some(Profile::parse(&source).unwrap()),
            Intent::Saturation,
            false,
        );
        let probe = Color::cmyk8(17, 33, 49, 65);
        let expected_probe = local
            .to_rgb_f32(probe)
            .unwrap()
            .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
        engine::clear();
        assert_ne!(probe.to_rgb8(), expected_probe);
        app.apply_color_settings();
        for color in [
            Color::cmyk8(17, 33, 49, 65),
            Color::cmyk8(251, 127, 3, 9),
            Color::cmyk8(0, 85, 255, 11),
        ] {
            let expected = local
                .to_rgb_f32(color)
                .unwrap()
                .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
            assert_eq!(color.to_rgb8(), expected);
        }
        assert!(engine::icc().unwrap().cmyk_profile().is_some());
        let mut another = app.doc().clone();
        another.cmyk_profile = Some(tracedraw_core::DocumentProfile::new(
            "Retained source",
            source,
        ));
        app.engine.replace(another);
        app.sync_document_colors();
        let color = Color::cmyk8(17, 33, 49, 65);
        let expected = local
            .to_rgb_f32(color)
            .unwrap()
            .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
        assert_eq!(color.to_rgb8(), expected);
        engine::clear();
    }
}
