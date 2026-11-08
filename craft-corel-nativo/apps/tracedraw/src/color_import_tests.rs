//! Independent import colour fixtures and mathematical oracles.
//! The expected ink bytes never call the import planner or picture reprofiler.

use tracedraw_core::document::DocumentProfile;
use tracedraw_core::geometry::{Affine, Size};
use tracedraw_core::icc::{test_profiles, Intent, Profile, Transform};
use tracedraw_core::{Color, Document, Fill, Rect, Shape, ShapeKind};
use tracedraw_io::picture_profile::Options;

pub(crate) const CMYKA: [u8; 20] = [
    17, 33, 49, 65, 0, 251, 127, 3, 9, 1, 0, 0, 0, 85, 127, 11, 29, 57, 113, 255,
];

pub(crate) fn profile(darker: bool) -> DocumentProfile {
    DocumentProfile::new(
        if darker {
            "Independent B"
        } else {
            "Independent A"
        },
        test_profiles::intent_cmyk_profile(darker),
    )
}

pub(crate) fn options() -> Options {
    Options {
        intent: Intent::Saturation,
        black_point_compensation: false,
        preserve_pure_black: false,
    }
}

pub(crate) fn tiff(profile: Option<&DocumentProfile>) -> Vec<u8> {
    tracedraw_io::cmyk_bitmap::cmyka_tiff(
        2,
        2,
        &CMYKA,
        300.0,
        profile.map(|profile| profile.data.as_slice()),
    )
    .unwrap()
}

pub(crate) fn picture(profile: Option<&DocumentProfile>) -> Shape {
    let doc =
        tracedraw_io::raster::document_from_bytes(&tiff(profile), "Owned CMYKA fixture").unwrap();
    let mut shape = doc.pages[0].layers[0].shapes[0].clone();
    shape.name = Some("Independent CMYKA source".into());
    shape.transform = Affine::translate((7.0, 9.0));
    shape
        .data
        .push(("bitmap.link".into(), "owned-foreign.tif".into()));
    shape
}

pub(crate) fn document(profile: Option<&DocumentProfile>) -> Document {
    let mut doc = Document::new("Independent colour import", Size::new(100.0, 70.0));
    doc.cmyk_profile = profile.cloned();
    doc.metadata.cmyk_profile = profile
        .map(|profile| profile.name.clone())
        .unwrap_or_else(|| "Generic CMYK (open)".into());
    doc
}

pub(crate) fn add_picture(doc: &mut Document, own_profile: Option<&DocumentProfile>) {
    let mut bitmap = picture(own_profile);
    bitmap.id = doc.ids_mut().shape();
    doc.pages[0].layers[0].shapes.push(bitmap);
}

pub(crate) fn add_vector(doc: &mut Document) {
    let mut shape = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Rect {
            rect: Rect::new(10.0, 15.0, 30.0, 35.0),
            radius: 0.0,
            corners: None,
        },
    );
    shape.fill = Fill::Solid(Color::cmyk_pct(17.0, 33.0, 49.0, 65.0));
    doc.pages[0].layers[0].shapes.push(shape);
}

pub(crate) fn raw_picture(shape: &Shape) -> (Vec<u8>, Option<Vec<u8>>, image::RgbaImage) {
    let bytes = tracedraw_io::picture::original_cmyk_tiff(shape, 300.0).unwrap();
    let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(bytes)).unwrap();
    let embedded = decoder.get_tag_u8_vec(tiff::tags::Tag::IccProfile).ok();
    let tiff::decoder::DecodingResult::U8(samples) = decoder.read_image().unwrap() else {
        panic!("expected original eight-bit ink planes");
    };
    let ShapeKind::Bitmap { png, .. } = &shape.kind else {
        panic!("expected bitmap");
    };
    (
        samples,
        embedded,
        tracedraw_io::picture::read(png).unwrap().rgba,
    )
}

pub(crate) fn bitmaps(doc: &Document) -> Vec<&Shape> {
    fn visit_fill<'a>(fill: &'a Fill, result: &mut Vec<&'a Shape>) {
        if let Fill::Pattern(tracedraw_core::style::Pattern::Vector { shapes, .. }) = fill {
            for shape in shapes {
                visit(shape, result);
            }
        }
    }
    fn visit<'a>(shape: &'a Shape, result: &mut Vec<&'a Shape>) {
        visit_fill(&shape.fill, result);
        for fill in &shape.extra_fills {
            visit_fill(fill, result);
        }
        match &shape.kind {
            ShapeKind::Bitmap { .. } => result.push(shape),
            ShapeKind::Group { children } => children.iter().for_each(|child| visit(child, result)),
            ShapeKind::ClipFrame { frame, contents } => {
                visit(frame, result);
                contents.iter().for_each(|child| visit(child, result));
            }
            _ => {}
        }
    }
    let mut result = Vec::new();
    for layer in doc.all_layers() {
        for shape in &layer.shapes {
            visit(shape, &mut result);
        }
    }
    for symbol in &doc.symbols {
        visit(&symbol.shape, &mut result);
    }
    result
}

fn same_calibration(source: Option<&DocumentProfile>, target: Option<&DocumentProfile>) -> bool {
    match (source, target) {
        (None, None) => true,
        (Some(source), Some(target)) => source.data == target.data,
        _ => false,
    }
}

pub(crate) fn expected_pixel(
    value: [f32; 4],
    source: Option<&DocumentProfile>,
    target: Option<&DocumentProfile>,
    options: Options,
) -> [f32; 4] {
    if same_calibration(source, target)
        || (options.preserve_pure_black && value[..3] == [0.0, 0.0, 0.0])
    {
        return value;
    }
    let srgb = Profile::srgb();
    match (source, target) {
        (Some(source), Some(target)) => {
            let source = Profile::parse(&source.data).unwrap();
            let target = Profile::parse(&target.data).unwrap();
            Transform::new(
                &source,
                &target,
                options.intent,
                options.black_point_compensation,
            )
            .unwrap()
            .transform(&value)
            .try_into()
            .unwrap()
        }
        (None, Some(target)) => {
            let rgb = tracedraw_core::press::cmyk_to_srgb(value);
            let target = Profile::parse(&target.data).unwrap();
            Transform::new(
                &srgb,
                &target,
                options.intent,
                options.black_point_compensation,
            )
            .unwrap()
            .transform(&rgb)
            .try_into()
            .unwrap()
        }
        (Some(source), None) => {
            let source = Profile::parse(&source.data).unwrap();
            let rgb = Transform::new(
                &source,
                &srgb,
                options.intent,
                options.black_point_compensation,
            )
            .unwrap()
            .transform(&value);
            tracedraw_core::press::srgb_to_cmyk(rgb.try_into().unwrap())
        }
        (None, None) => value,
    }
}

pub(crate) fn expected_samples(
    original: &[u8],
    source: Option<&DocumentProfile>,
    target: Option<&DocumentProfile>,
    options: Options,
) -> Vec<u8> {
    let mut expected = original.to_vec();
    for pixel in expected.chunks_exact_mut(5) {
        let output = expected_pixel(
            std::array::from_fn(|index| f32::from(pixel[index]) / 255.0),
            source,
            target,
            options,
        );
        for (byte, value) in pixel[..4].iter_mut().zip(output) {
            *byte = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    expected
}

pub(crate) fn assert_picture(shape: &Shape, samples: &[u8], target: Option<&DocumentProfile>) {
    let (actual, icc, rgba) = raw_picture(shape);
    assert_eq!(
        actual, samples,
        "all ink bytes, including transparent pixels"
    );
    assert_eq!(
        icc.as_deref(),
        target.map(|profile| profile.data.as_slice())
    );
    let srgb = Profile::srgb();
    let target = target.map(|profile| Profile::parse(&profile.data).unwrap());
    let transform = target.as_ref().map(|profile| {
        Transform::new(profile, &srgb, Intent::RelativeColorimetric, false).unwrap()
    });
    for (actual, original) in rgba.pixels().zip(samples.chunks_exact(5)) {
        let values = std::array::from_fn(|index| f32::from(original[index]) / 255.0);
        let rgb: [f32; 3] = transform
            .as_ref()
            .map(|transform| transform.transform(&values).try_into().unwrap())
            .unwrap_or_else(|| tracedraw_core::press::cmyk_to_srgb(values));
        let expected: [u8; 4] = std::array::from_fn(|index| {
            if index == 3 {
                original[4]
            } else {
                (rgb[index].clamp(0.0, 1.0) * 255.0).round() as u8
            }
        });
        assert_eq!(actual.0, expected, "unassociated display pixels and alpha");
    }
}

fn import_options(policy: u8) -> crate::color_import::ImportColorOptions {
    crate::color_import::ImportColorOptions {
        policy,
        default_cmyk: None,
        bitmap: options(),
        warn_missing: true,
        warn_mismatch: true,
    }
}

fn prepared_destination(
    original: &Document,
    plan: &crate::color_import::ImportColorPlan,
) -> Document {
    let mut result = original.clone();
    for command in &plan.destination_commands {
        command.apply(&mut result).unwrap();
    }
    result
}

#[test]
fn independent_three_policy_matrix_keeps_alpha_and_changes_the_correct_document() {
    let a = profile(false);
    let b = profile(true);
    for policy in [0, 1, 2] {
        let mut destination = document(Some(&b));
        add_picture(&mut destination, Some(&b));
        add_vector(&mut destination);
        let mut incoming = document(Some(&a));
        add_picture(&mut incoming, Some(&a));
        add_vector(&mut incoming);
        let old_destination = destination.clone();
        let old_incoming = incoming.clone();
        let plan = crate::color_import::prepare_import_color(
            &destination,
            incoming.clone(),
            &import_options(policy),
        )
        .unwrap();
        assert_eq!(
            destination, old_destination,
            "prepare cannot edit destination"
        );
        assert_eq!(
            incoming, old_incoming,
            "prepare consumes only its owned copy"
        );
        let changed_destination = prepared_destination(&destination, &plan);
        match policy {
            0 => {
                assert_eq!(changed_destination, old_destination);
                let expected = expected_samples(&CMYKA, Some(&a), Some(&b), options());
                assert_ne!(expected, CMYKA, "the oracle must detect wrong identity");
                assert_picture(bitmaps(&plan.incoming)[0], &expected, Some(&b));
            }
            1 => {
                assert_eq!(changed_destination, old_destination);
                assert_picture(bitmaps(&plan.incoming)[0], &CMYKA, Some(&b));
                assert_ne!(
                    raw_picture(bitmaps(&plan.incoming)[0]).2,
                    raw_picture(bitmaps(&old_incoming)[0]).2,
                    "assignment changes appearance rather than ink values"
                );
            }
            2 => {
                assert_eq!(
                    plan.incoming, old_incoming,
                    "embedded incoming values survive"
                );
                assert_eq!(
                    changed_destination.cmyk_profile.as_ref().unwrap().data,
                    a.data
                );
                let expected = expected_samples(&CMYKA, Some(&b), Some(&a), options());
                assert_ne!(expected, CMYKA);
                assert_picture(bitmaps(&changed_destination)[0], &expected, Some(&a));
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn own_picture_icc_overrides_a_matching_document_icc_and_names_are_not_identity() {
    let a = profile(false);
    let b = profile(true);
    let destination = document(Some(&a));
    let mut incoming = document(Some(&a));
    add_picture(&mut incoming, Some(&b));
    let plan =
        crate::color_import::prepare_import_color(&destination, incoming, &import_options(0))
            .unwrap();
    let expected = expected_samples(&CMYKA, Some(&b), Some(&a), options());
    assert_ne!(expected, CMYKA);
    assert_picture(bitmaps(&plan.incoming)[0], &expected, Some(&a));

    let mut same_name_b = b.clone();
    same_name_b.name = a.name.clone();
    let destination = document(Some(&same_name_b));
    let mut incoming = document(Some(&a));
    add_picture(&mut incoming, Some(&a));
    let plan =
        crate::color_import::prepare_import_color(&destination, incoming, &import_options(0))
            .unwrap();
    assert!(
        plan.notice.is_some(),
        "equal names cannot hide a data mismatch"
    );
    assert_picture(
        bitmaps(&plan.incoming)[0],
        &expected_samples(&CMYKA, Some(&a), Some(&b), options()),
        Some(&b),
    );

    let mut renamed_a = a.clone();
    renamed_a.name = "Other label, identical calibration".into();
    let destination = document(Some(&renamed_a));
    let mut incoming = document(Some(&a));
    add_picture(&mut incoming, Some(&a));
    let original = incoming.clone();
    let plan =
        crate::color_import::prepare_import_color(&destination, incoming, &import_options(0))
            .unwrap();
    assert!(
        plan.notice.is_none(),
        "labels alone are not a calibration mismatch"
    );
    assert_eq!(raw_picture(bitmaps(&plan.incoming)[0]).0, CMYKA);
    assert_eq!(
        raw_picture(bitmaps(&plan.incoming)[0]).2,
        raw_picture(bitmaps(&original)[0]).2,
        "identical ICC data must not introduce a round trip"
    );
}

#[test]
fn untagged_and_generic_profiles_use_real_press_conversion_both_directions() {
    let a = profile(false);
    let b = profile(true);
    for default in [None, Some(b.clone())] {
        let destination = document(Some(&a));
        let mut incoming = document(None);
        add_picture(&mut incoming, None);
        let mut options = import_options(0);
        options.default_cmyk = default.clone();
        let plan =
            crate::color_import::prepare_import_color(&destination, incoming, &options).unwrap();
        assert!(plan.notice.is_some(), "missing source profile warning");
        let expected = expected_samples(&CMYKA, default.as_ref(), Some(&a), options.bitmap);
        assert_ne!(expected, CMYKA);
        assert_picture(bitmaps(&plan.incoming)[0], &expected, Some(&a));
    }

    let destination = document(None);
    let mut incoming = document(Some(&a));
    add_picture(&mut incoming, Some(&a));
    let plan =
        crate::color_import::prepare_import_color(&destination, incoming, &import_options(0))
            .unwrap();
    let expected = expected_samples(&CMYKA, Some(&a), None, options());
    assert_ne!(expected, CMYKA);
    assert_picture(bitmaps(&plan.incoming)[0], &expected, None);
    assert!(prepared_destination(&destination, &plan)
        .cmyk_profile
        .is_none());
}

#[test]
fn picture_conversion_honours_intent_bpc_and_pure_black_even_under_transparency() {
    let a = profile(false);
    let b = profile(true);
    let mut outputs = std::collections::HashSet::new();
    let mut relative_bpc = Vec::new();
    for intent in [
        Intent::RelativeColorimetric,
        Intent::AbsoluteColorimetric,
        Intent::Perceptual,
        Intent::Saturation,
    ] {
        for bpc in [false, true] {
            for black in [false, true] {
                let destination = document(Some(&a));
                let mut incoming = document(Some(&a));
                add_picture(&mut incoming, Some(&b));
                let mut options = import_options(0);
                options.bitmap.intent = intent;
                options.bitmap.black_point_compensation = bpc;
                options.bitmap.preserve_pure_black = black;
                let plan =
                    crate::color_import::prepare_import_color(&destination, incoming, &options)
                        .unwrap();
                let expected = expected_samples(&CMYKA, Some(&b), Some(&a), options.bitmap);
                let actual = raw_picture(bitmaps(&plan.incoming)[0]).0;
                assert_eq!(actual, expected);
                assert_eq!(
                    actual
                        .chunks_exact(5)
                        .map(|pixel| pixel[4])
                        .collect::<Vec<_>>(),
                    [0, 1, 127, 255]
                );
                assert_ne!(
                    actual[..4],
                    CMYKA[..4],
                    "transparent ink values also convert"
                );
                if black {
                    assert_eq!(actual[10..14], CMYKA[10..14], "K-only option");
                }
                if !bpc && !black {
                    outputs.insert(actual.clone());
                }
                if intent == Intent::RelativeColorimetric && !black {
                    relative_bpc.push(actual);
                }
            }
        }
    }
    assert_eq!(
        outputs.len(),
        4,
        "each intent must have an observable oracle"
    );
    assert_ne!(relative_bpc[0], relative_bpc[1], "BPC must be consumed");
}

#[test]
fn late_invalid_picture_and_unknown_policy_return_no_partial_destination_change() {
    let a = profile(false);
    let b = profile(true);
    let mut destination = document(Some(&a));
    add_picture(&mut destination, Some(&a));
    let original = destination.clone();
    let mut incoming = document(Some(&b));
    add_picture(&mut incoming, Some(&b));
    let mut bad = picture(Some(&b));
    bad.id = incoming.ids_mut().shape();
    bad.picture_inks.as_mut().unwrap().png = vec![1, 2, 3].into();
    incoming.symbols.push(tracedraw_core::document::Symbol {
        name: "Malformed late definition".into(),
        shape: bad,
    });
    assert!(crate::color_import::prepare_import_color(
        &destination,
        incoming.clone(),
        &import_options(0),
    )
    .is_err());
    assert_eq!(destination, original);
    assert!(crate::color_import::prepare_import_color(
        &destination,
        incoming,
        &import_options(255),
    )
    .is_err());
    assert_eq!(destination, original);
}

#[test]
fn destination_conversion_reaches_groups_clips_master_and_unused_symbols_without_changing_rgb() {
    use tracedraw_core::document::{Layer, Symbol};
    let a = profile(false);
    let b = profile(true);
    let mut destination = document(Some(&b));
    add_picture(&mut destination, Some(&b));
    let mut child = picture(Some(&b));
    child.id = destination.ids_mut().shape();
    child.uid = None;
    let group = Shape::new(
        destination.ids_mut().shape(),
        ShapeKind::Group {
            children: vec![child],
        },
    );
    destination.pages[0].layers[0].shapes.push(group);
    let frame = Shape::new(
        destination.ids_mut().shape(),
        ShapeKind::Rect {
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            radius: 0.0,
            corners: None,
        },
    );
    let mut child = picture(Some(&b));
    child.id = destination.ids_mut().shape();
    child.uid = None;
    let clip = Shape::new(
        destination.ids_mut().shape(),
        ShapeKind::ClipFrame {
            frame: Box::new(frame),
            contents: vec![child],
        },
    );
    destination.pages[0].layers[0].shapes.push(clip);
    let mut child = picture(Some(&b));
    child.id = destination.ids_mut().shape();
    child.uid = None;
    destination.symbols.push(Symbol {
        name: "Unused CMYKA definition".into(),
        shape: child,
    });
    let mut child = picture(Some(&b));
    child.id = destination.ids_mut().shape();
    child.uid = None;
    let mut master = Layer::new(destination.ids_mut().layer(), "CMYKA master");
    master.shapes.push(child);
    destination.master.push(master);
    add_vector(&mut destination);
    let mut pattern_child = picture(Some(&b));
    pattern_child.id = destination.ids_mut().shape();
    pattern_child.uid = None;
    destination.pages[0].layers[0]
        .shapes
        .last_mut()
        .unwrap()
        .extra_fills
        .push(Fill::Pattern(tracedraw_core::style::Pattern::Vector {
            shapes: vec![pattern_child],
            tile: Size::new(3.0, 3.0),
        }));
    let rgba = image::RgbaImage::from_pixel(2, 2, image::Rgba([24, 78, 139, 61]));
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let mut rgb = Shape::new(
        destination.ids_mut().shape(),
        ShapeKind::Bitmap {
            rect: Rect::new(50.0, 10.0, 55.0, 15.0),
            width_px: 2,
            height_px: 2,
            png: png.into_inner().into(),
            fx: None,
        },
    );
    rgb.name = Some("RGB must not be converted to ink mode".into());
    destination.pages[0].layers[0].shapes.push(rgb.clone());
    let mut incoming = document(Some(&a));
    add_vector(&mut incoming);
    let plan =
        crate::color_import::prepare_import_color(&destination, incoming, &import_options(2))
            .unwrap();
    let updated = prepared_destination(&destination, &plan);
    let expected = expected_samples(&CMYKA, Some(&b), Some(&a), options());
    let cmyk: Vec<_> = bitmaps(&updated)
        .into_iter()
        .filter(|shape| shape.cmyk_bitmap)
        .collect();
    assert_eq!(
        cmyk.len(),
        6,
        "all definitions, master, vector-pattern and nested pictures"
    );
    for bitmap in cmyk {
        assert_picture(bitmap, &expected, Some(&a));
    }
    assert_eq!(
        updated.find_shape(rgb.id).unwrap(),
        &rgb,
        "RGB pixels and mode are unchanged"
    );
}

fn collect_cmyk(value: &serde_json::Value, output: &mut Vec<[f32; 4]>) {
    if let Ok(Color::Cmyk { c, m, y, k }) = serde_json::from_value::<Color>(value.clone()) {
        output.push([c, m, y, k]);
        return;
    }
    match value {
        serde_json::Value::Object(fields) => fields
            .values()
            .for_each(|value| collect_cmyk(value, output)),
        serde_json::Value::Array(values) => {
            values.iter().for_each(|value| collect_cmyk(value, output))
        }
        _ => {}
    }
}

#[test]
fn destination_conversion_covers_text_spans_extra_paints_patterns_palette_and_colour_styles() {
    use tracedraw_core::document::{ColorStyle, TextSpan};
    use tracedraw_core::geometry::Point;
    use tracedraw_core::style::{Fountain, FountainKind, Pattern, PatternTile, Stroke};
    let a = profile(false);
    let b = profile(true);
    let value = Color::cmyk_pct(17.0, 33.0, 49.0, 65.0);
    let Color::Cmyk { c, m, y, k } = value else {
        unreachable!()
    };
    let expected = expected_pixel([c, m, y, k], Some(&b), Some(&a), options());
    assert_ne!(expected, [c, m, y, k]);
    let mut destination = document(Some(&b));
    add_vector(&mut destination);
    let shape = &mut destination.pages[0].layers[0].shapes[0];
    shape.shadow = Some(tracedraw_core::document::Shadow {
        color: value,
        ..Default::default()
    });
    shape
        .effects
        .push(tracedraw_core::live::Effect::BlockShadow {
            offset: tracedraw_core::geometry::Vec2::new(1.25, -2.5),
            color: value,
            gap: 0.2,
        });
    shape.extra_fills.push(Fill::Fountain(Fountain::two(
        FountainKind::Linear,
        value,
        value,
        31.0,
    )));
    shape.extra_fills.push(Fill::Pattern(Pattern::TwoColor {
        tile: PatternTile::Checker,
        front: value,
        back: Color::rgb8(10, 60, 130),
        size_mm: 4.5,
    }));
    shape.extra_strokes.push(Stroke {
        color: value,
        ..Default::default()
    });
    let mut span = TextSpan::new("Independent text span", "Selawik", 18.0);
    span.fill = Some(Fill::Solid(value));
    span.background = Some(Fill::Solid(value));
    span.outline = Some(Stroke {
        color: value,
        ..Default::default()
    });
    let text = Shape::new(
        destination.ids_mut().shape(),
        ShapeKind::Text {
            spans: vec![span],
            origin: Point::new(10.0, 50.0),
            frame: None,
            align: Default::default(),
            para: Default::default(),
            on_path: None,
        },
    );
    destination.pages[0].layers[0].shapes.push(text);
    destination.palette.push(value);
    destination.color_styles.push(ColorStyle {
        name: "Independent CMYK style".into(),
        color: value,
        harmony: None,
    });
    let mut original_values = Vec::new();
    collect_cmyk(
        &serde_json::to_value(&destination).unwrap(),
        &mut original_values,
    );
    assert!(
        original_values.len() >= 10,
        "the fixture exercises several colour-bearing fields"
    );
    assert_eq!(
        original_values
            .iter()
            .filter(|channels| **channels == [c, m, y, k])
            .count(),
        12,
        "all twelve injected fields use the independent sample"
    );
    assert!(
        original_values
            .iter()
            .any(|channels| *channels == [0.0, 0.0, 0.0, 1.0]),
        "the fixture also covers the document's default black paints"
    );
    let mut incoming = document(Some(&a));
    add_vector(&mut incoming);
    let plan =
        crate::color_import::prepare_import_color(&destination, incoming, &import_options(2))
            .unwrap();
    let updated = prepared_destination(&destination, &plan);
    let mut converted = Vec::new();
    collect_cmyk(&serde_json::to_value(&updated).unwrap(), &mut converted);
    assert_eq!(
        converted.len(),
        original_values.len(),
        "keep the CMYK model and every colour-bearing field"
    );
    for (channels, original) in converted.into_iter().zip(original_values) {
        let expected = expected_pixel(original, Some(&b), Some(&a), options());
        for (actual, expected) in channels.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() < 2e-6,
                "all CMYK fields use the document transform"
            );
        }
    }
    let ShapeKind::Rect {
        rect: before_rect, ..
    } = destination.pages[0].layers[0].shapes[0].kind
    else {
        unreachable!()
    };
    let ShapeKind::Rect {
        rect: after_rect, ..
    } = updated.pages[0].layers[0].shapes[0].kind
    else {
        unreachable!()
    };
    assert_eq!(before_rect, after_rect);
    assert_eq!(
        updated.pages[0].layers[0].shapes[0].transform,
        destination.pages[0].layers[0].shapes[0].transform
    );
}

#[test]
fn ignoring_embedded_source_can_repair_a_bad_tag_but_conversion_cannot_guess_it() {
    let a = profile(false);
    let b = profile(true);
    let bad = DocumentProfile::new("Malformed embedded source", vec![1, 2, 3, 4]);
    let destination = document(Some(&a));
    let mut incoming = document(Some(&b));
    add_picture(&mut incoming, Some(&b));
    incoming.pages[0].layers[0].shapes[0]
        .picture_inks
        .as_mut()
        .unwrap()
        .profile = Some(bad.clone());
    let assigned = crate::color_import::prepare_import_color(
        &destination,
        incoming.clone(),
        &import_options(1),
    )
    .unwrap();
    assert_picture(bitmaps(&assigned.incoming)[0], &CMYKA, Some(&a));
    assert!(
        crate::color_import::prepare_import_color(&destination, incoming, &import_options(0),)
            .is_err()
    );
    let invalid_destination = document(Some(&bad));
    let mut valid_incoming = document(Some(&a));
    add_picture(&mut valid_incoming, Some(&a));
    for policy in [0, 1] {
        assert!(crate::color_import::prepare_import_color(
            &invalid_destination,
            valid_incoming.clone(),
            &import_options(policy),
        )
        .is_err());
    }
}

#[test]
fn materializing_a_real_destination_default_also_calibrates_its_untagged_picture_preview() {
    let a = profile(false);
    let b = profile(true);
    for policy in [0, 1] {
        let mut destination = document(None);
        add_picture(&mut destination, None);
        add_picture(&mut destination, Some(&a));
        let own_before = destination.pages[0].layers[0].shapes[1].clone();
        let mut incoming = document(Some(&a));
        add_picture(&mut incoming, Some(&a));
        let mut options = import_options(policy);
        options.default_cmyk = Some(b.clone());
        let plan =
            crate::color_import::prepare_import_color(&destination, incoming, &options).unwrap();
        let updated = prepared_destination(&destination, &plan);
        assert_eq!(updated.cmyk_profile.as_ref().unwrap().data, b.data);
        let actual = raw_picture(bitmaps(&updated)[0]);
        assert_eq!(
            actual.0, CMYKA,
            "materialization assigns interpretation, not new ink values"
        );
        let expected_display = raw_picture(&picture(Some(&b))).2;
        assert_ne!(
            raw_picture(bitmaps(&destination)[0]).2,
            expected_display,
            "the fixture distinguishes B from the generic preview"
        );
        assert_eq!(actual.2, expected_display);
        assert_eq!(
            updated.find_shape(own_before.id).unwrap(),
            &own_before,
            "a picture with its own ICC A remains exactly intact"
        );
        assert!(
            !plan.destination_commands.is_empty(),
            "the destination change is part of the import transaction"
        );
        assert!(
            destination.cmyk_profile.is_none(),
            "preparation does not materialize the live destination"
        );
    }
}
