//! Import transaction refutation: edited destinations, resources and failures.

use super::color_import_tests as fixture;
use crate::app::App;
use tracedraw_core::{Command, Document, LayerId, PageId, ShapeId};

#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    document: Document,
    instance: u64,
    revision: u64,
    history: (Vec<&'static str>, Vec<&'static str>),
    last_step: Option<u64>,
    dirty: bool,
    selection: Vec<ShapeId>,
    page: PageId,
    layer: Option<LayerId>,
    missing_fonts: Vec<String>,
    color_preferences: serde_json::Value,
}

fn snapshot(app: &App) -> Snapshot {
    Snapshot {
        document: app.doc().clone(),
        instance: app.engine.instance(),
        revision: app.engine.revision(),
        history: app.engine.history_labels(),
        last_step: app.engine.last_step_id(),
        dirty: app.engine.is_dirty(),
        selection: app.selection.clone(),
        page: app.page,
        layer: app.active_layer(),
        missing_fonts: app.missing_fonts.clone(),
        color_preferences: serde_json::to_value(&app.settings.color).unwrap(),
    }
}

fn edited_destination() -> App {
    let b = fixture::profile(true);
    let mut doc = fixture::document(Some(&b));
    fixture::add_picture(&mut doc, Some(&b));
    fixture::add_vector(&mut doc);
    let selected = doc.pages[0].layers[0].shapes[1].id;
    let page = doc.pages[0].id;
    let mut app = App::headless();
    app.engine.replace(doc);
    app.page = page;
    app.selection = vec![selected];
    app.settings.color.intent = "Saturation".into();
    app.settings.color.black_point_compensation = false;
    app.settings.color.preserve_pure_black = false;
    app.settings.color.cmyk_profile_path.clear();
    app.settings.color.cmyk_profile = "Generic CMYK (open)".into();
    app.engine
        .run(&Command::SetTitle {
            title: "Edited destination survives import".into(),
        })
        .unwrap();
    assert_eq!(app.engine.history_labels().0.len(), 1);
    app
}

fn directory(name: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../evidencias/craft-corel-nativo/importacion-color/generated")
        .join(format!(
            "{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn source_document() -> Document {
    let a = fixture::profile(false);
    let mut incoming = fixture::document(Some(&a));
    fixture::add_picture(&mut incoming, Some(&a));
    fixture::add_vector(&mut incoming);
    incoming
}

#[test]
fn native_memory_clipboard_preserves_real_icc_and_obeys_all_three_policies_atomically() {
    use crate::clipboard::NativeClipboardPayload;
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    let mut source = source_document();
    let selected = source.pages[0].layers[0].shapes.clone();
    fixture::add_vector(&mut source); // Deliberately outside the copied selection.
    let original = source.clone();
    let captured = NativeClipboardPayload::capture(&source, &selected, 123456).unwrap();
    let payload: NativeClipboardPayload =
        serde_json::from_value(serde_json::to_value(&captured).unwrap()).unwrap();
    assert_eq!(payload.version, 1);
    assert_eq!(payload.source_instance, 123456);
    assert_eq!(payload.document.cmyk_profile, source.cmyk_profile);
    assert_eq!(payload.document.pages[0].layers[0].shapes.len(), 2);
    for policy in [0, 1, 2] {
        let mut app = edited_destination();
        app.settings.color.import_cmyk = policy;
        let before = snapshot(&app);
        let original_id = fixture::bitmaps(app.doc())[0].id;
        let imported = app.try_paste_native(&payload).unwrap();
        assert_eq!(app.selection, imported);
        assert_eq!(
            app.engine.history_labels().0.len(),
            before.history.0.len() + 1
        );
        assert_eq!(app.engine.history_labels().0.last(), Some(&"Paste"));
        let target = if policy == 2 { &a } else { &b };
        assert_eq!(app.doc().cmyk_profile.as_ref().unwrap().data, target.data);
        assert_eq!(
            fixture::bitmaps(app.doc()).len(),
            2,
            "keep the existing picture and paste the selected one"
        );
        for picture in fixture::bitmaps(app.doc()) {
            let expected = if picture.id == original_id {
                if policy == 2 {
                    fixture::expected_samples(
                        &fixture::CMYKA,
                        Some(&b),
                        Some(&a),
                        fixture::options(),
                    )
                } else {
                    fixture::CMYKA.to_vec()
                }
            } else if policy == 0 {
                fixture::expected_samples(&fixture::CMYKA, Some(&a), Some(&b), fixture::options())
            } else {
                fixture::CMYKA.to_vec()
            };
            fixture::assert_picture(picture, &expected, Some(target));
        }
        let after = app.doc().clone();
        app.undo();
        assert_doc_after_undo(app.doc(), &before.document);
        assert_eq!(app.selection, before.selection);
        app.redo();
        assert_eq!(app.doc(), &after);
        assert_eq!(app.selection, imported);
        assert_eq!(source, original);
    }
    for fault in ["version", "bitmap", "symbol"] {
        let mut invalid = payload.clone();
        match fault {
            "version" => invalid.version = 99,
            "bitmap" => {
                invalid.document.pages[0].layers[0].shapes[0]
                    .picture_inks
                    .as_mut()
                    .unwrap()
                    .png = vec![5, 7, 11].into()
            }
            "symbol" => {
                let shape = tracedraw_core::Shape::new(
                    invalid.document.ids_mut().shape(),
                    tracedraw_core::ShapeKind::SymbolInstance { index: 777 },
                );
                invalid
                    .document
                    .symbols
                    .push(tracedraw_core::document::Symbol {
                        name: "Late invalid clipboard symbol".into(),
                        shape,
                    });
            }
            _ => unreachable!(),
        }
        let mut app = edited_destination();
        app.settings.color.import_cmyk = 0;
        let before = snapshot(&app);
        assert!(app.try_paste_native(&invalid).is_err(), "invalid {fault}");
        assert_eq!(snapshot(&app), before, "failed {fault} must be atomic");
    }
}

#[test]
fn clipboard_capture_remaps_only_transitive_symbols_and_rejects_cycles() {
    use crate::clipboard::NativeClipboardPayload;
    use tracedraw_core::{
        document::{DocumentFont, Symbol},
        Shape, ShapeKind,
    };
    let mut source = source_document();
    source.fonts.push(DocumentFont::new(
        "Unused carried font",
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").to_vec(),
    ));
    for index in 0..5 {
        let mut shape = fixture::picture(source.cmyk_profile.as_ref());
        shape.id = source.ids_mut().shape();
        if index == 0 {
            shape.picture_inks.as_mut().unwrap().png = vec![1, 3, 9].into();
        }
        if index == 4 {
            let child = Shape::new(
                source.ids_mut().shape(),
                ShapeKind::SymbolInstance { index: 2 },
            );
            shape = Shape::new(
                source.ids_mut().shape(),
                ShapeKind::Group {
                    children: vec![child],
                },
            );
        }
        source.symbols.push(Symbol {
            name: format!("Owned symbol {index}"),
            shape,
        });
    }
    let selected = Shape::new(
        source.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 4 },
    );
    let original = source.clone();
    let payload = NativeClipboardPayload::capture(&source, &[selected.clone()], 987654).unwrap();
    assert_eq!(payload.document.symbols.len(), 2);
    assert_eq!(payload.document.symbols[0].name, "Owned symbol 2");
    assert_eq!(payload.document.symbols[1].name, "Owned symbol 4");
    assert!(payload.document.fonts.is_empty());
    assert!(payload.document.master.is_empty());
    assert_eq!(fixture::bitmaps(&payload.document).len(), 1);
    let mut root_indices = Vec::new();
    symbol_indices(
        &payload.document.pages[0].layers[0].shapes[0],
        &mut root_indices,
    );
    assert_eq!(root_indices, vec![1]);
    let mut nested_indices = Vec::new();
    symbol_indices(&payload.document.symbols[1].shape, &mut nested_indices);
    assert_eq!(nested_indices, vec![0]);
    assert_eq!(source, original);
    source.symbols[2].shape.kind = ShapeKind::SymbolInstance { index: 4 };
    assert!(NativeClipboardPayload::capture(&source, &[selected], 987654).is_err());
}

#[test]
fn untagged_clipboard_records_source_default_a_before_pasting_into_default_b() {
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    let root = directory("clipboard-defaults");
    let a_path = root.join("source-a.icc");
    let b_path = root.join("destination-b.icc");
    std::fs::write(&a_path, a.data.as_slice()).unwrap();
    std::fs::write(&b_path, b.data.as_slice()).unwrap();
    let mut source_doc = fixture::document(None);
    fixture::add_picture(&mut source_doc, None);
    fixture::add_vector(&mut source_doc);
    let mut source = edited_destination();
    source.page = source_doc.pages[0].id;
    source.engine.replace(source_doc);
    source.settings.color.cmyk_profile_path = a_path.display().to_string();
    source.settings.color.cmyk_profile = a.name.clone();
    let source_before = snapshot(&source);
    let payload = source
        .native_payload_for_selection(&source.doc().pages[0].layers[0].shapes)
        .unwrap();
    assert_eq!(payload.document.cmyk_profile.as_ref().unwrap().data, a.data);
    assert_eq!(snapshot(&source), source_before);
    for policy in [0, 1, 2] {
        let mut destination_doc = fixture::document(None);
        fixture::add_picture(&mut destination_doc, None);
        fixture::add_vector(&mut destination_doc);
        let mut destination = edited_destination();
        destination.page = destination_doc.pages[0].id;
        destination.selection = vec![destination_doc.pages[0].layers[0].shapes[1].id];
        let old_id = destination_doc.pages[0].layers[0].shapes[0].id;
        destination.engine.replace(destination_doc);
        destination.settings.color.cmyk_profile_path = b_path.display().to_string();
        destination.settings.color.cmyk_profile = b.name.clone();
        destination.settings.color.import_cmyk = policy;
        let before = snapshot(&destination);
        destination.try_paste_native(&payload).unwrap();
        let target = if policy == 2 { &a } else { &b };
        assert_eq!(
            destination.doc().cmyk_profile.as_ref().unwrap().data,
            target.data
        );
        let pictures = fixture::bitmaps(destination.doc());
        assert_eq!(pictures.len(), 2);
        for picture in pictures {
            let expected = if picture.id == old_id && policy == 2 {
                fixture::expected_samples(&fixture::CMYKA, Some(&b), Some(&a), fixture::options())
            } else if picture.id != old_id && policy == 0 {
                fixture::expected_samples(&fixture::CMYKA, Some(&a), Some(&b), fixture::options())
            } else {
                fixture::CMYKA.to_vec()
            };
            fixture::assert_picture(picture, &expected, Some(target));
        }
        let after = destination.doc().clone();
        destination.undo();
        assert_doc_after_undo(destination.doc(), &before.document);
        assert!(destination.doc().cmyk_profile.is_none());
        destination.redo();
        assert_eq!(destination.doc(), &after);
        assert_eq!(snapshot(&source), source_before);
    }
    assert_eq!(std::fs::read(a_path).unwrap().as_slice(), a.data.as_slice());
    assert_eq!(std::fs::read(b_path).unwrap().as_slice(), b.data.as_slice());
}

#[test]
fn borrowed_clipboard_depth_is_rejected_before_recursive_clones_or_identity_checks() {
    use crate::clipboard::NativeClipboardPayload;
    use tracedraw_core::{Shape, ShapeKind};
    fn dismantle(document: &mut Document) {
        let mut pending = std::mem::take(&mut document.pages[0].layers[0].shapes);
        while let Some(mut shape) = pending.pop() {
            if let ShapeKind::Group { children } = std::mem::replace(
                &mut shape.kind,
                ShapeKind::Group {
                    children: Vec::new(),
                },
            ) {
                pending.extend(children);
            }
        }
    }
    let mut document = fixture::document(Some(&fixture::profile(false)));
    fixture::add_vector(&mut document);
    let mut tree = document.pages[0].layers[0].shapes.pop().unwrap();
    for _ in 0..140 {
        tree = Shape::new(
            document.ids_mut().shape(),
            ShapeKind::Group {
                children: vec![tree],
            },
        );
    }
    document.pages[0].layers[0].shapes.push(tree);
    let capture_error = match NativeClipboardPayload::capture(
        &document,
        &document.pages[0].layers[0].shapes,
        123456,
    ) {
        Ok(mut captured) => {
            dismantle(&mut captured.document);
            None
        }
        Err(error) => Some(error),
    };
    let mut payload = NativeClipboardPayload {
        version: 1,
        document,
        source_instance: 123456,
    };
    let mut app = edited_destination();
    let before = snapshot(&app);
    let paste_error = app.try_paste_native(&payload).err();
    // Remove one group at a time even when an assertion below fails.
    dismantle(&mut payload.document);
    assert!(
        capture_error
            .as_deref()
            .is_some_and(|error| error.contains("colour tree exceeds its traversal budget")),
        "capture must reject the traversal budget before copying: {capture_error:?}"
    );
    assert!(
        paste_error
            .as_deref()
            .is_some_and(|error| error.contains("colour tree exceeds its traversal budget")),
        "paste must reject the traversal budget before copying: {paste_error:?}"
    );
    assert_eq!(snapshot(&app), before);
}

#[test]
fn editable_dimension_keeps_converted_colours_after_redraw_and_corrupt_metadata_is_atomic() {
    use crate::dimensions::{Dimension, DATA_KEY};
    use tracedraw_core::{geometry::Point, Color, Fill, ShapeKind};
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    let mut app = edited_destination();
    app.dimension_style.line_color = Color::cmyk_pct(17.0, 33.0, 49.0, 65.0);
    app.dimension_style.text_color = Color::cmyk_pct(51.0, 13.0, 27.0, 42.0);
    app.create_dimension(
        Point::new(2.0, 3.0),
        Point::new(32.0, 3.0),
        Point::new(2.0, 12.0),
    );
    let dimension_id = app.selection[0];
    let original = Dimension::of(app.doc().find_shape(dimension_id).unwrap()).unwrap();
    let convert = |colour: Color| {
        let Color::Cmyk { c, m, y, k } = colour else {
            panic!("CMYK fixture");
        };
        let [c, m, y, k] =
            fixture::expected_pixel([c, m, y, k], Some(&b), Some(&a), fixture::options());
        Color::Cmyk { c, m, y, k }
    };
    let expected_line = convert(original.style.line_color);
    let expected_text = convert(original.style.text_color);
    let mut destination = app.doc().clone();
    let group = destination.find_shape_mut(dimension_id).unwrap();
    let encoded = &mut group
        .data
        .iter_mut()
        .find(|(key, _)| key == DATA_KEY)
        .unwrap()
        .1;
    let mut original_metadata: serde_json::Value = serde_json::from_str(encoded).unwrap();
    original_metadata["owned_extension"] =
        serde_json::json!({"keep":"anchors, custom payload and measurement"});
    *encoded = serde_json::to_string(&original_metadata).unwrap();
    app.engine.replace(destination);
    app.settings.color.import_cmyk = 2;
    let before = snapshot(&app);
    let prepared = app.prepare_color_import(source_document(), None).unwrap();
    app.commit_color_import(prepared).unwrap();
    let after_import = app.doc().clone();
    let group = app.doc().find_shape(dimension_id).unwrap();
    let encoded = &group
        .data
        .iter()
        .find(|(key, _)| key == DATA_KEY)
        .unwrap()
        .1;
    let mut actual_metadata: serde_json::Value = serde_json::from_str(encoded).unwrap();
    for key in ["line_color", "text_color"] {
        actual_metadata["style"][key] = original_metadata["style"][key].clone();
    }
    assert_eq!(
        actual_metadata, original_metadata,
        "only persistent colour fields may change during import"
    );
    let converted = Dimension::of(group).unwrap();
    assert_eq!(converted.style.line_color, expected_line);
    assert_eq!(converted.style.text_color, expected_text);
    assert_eq!(converted.measure, original.measure);
    app.selection = vec![dimension_id];
    app.set_dimension_style(|style| style.precision = 3);
    let redrawn = app.doc().find_shape(dimension_id).unwrap();
    let ShapeKind::Group { children } = &redrawn.kind else {
        panic!("editable dimension group");
    };
    assert_eq!(children[0].stroke.as_ref().unwrap().color, expected_line);
    assert_eq!(children[2].fill, Fill::Solid(expected_text));
    let edited = Dimension::of(redrawn).unwrap();
    assert_eq!(edited.style.precision, 3);
    assert_eq!(edited.style.line_color, expected_line);
    assert_eq!(edited.style.text_color, expected_text);
    app.undo(); // Only the style edit.
    assert_doc_after_undo(app.doc(), &after_import);
    app.undo(); // Then the one import transaction.
    assert_doc_after_undo(app.doc(), &before.document);
    assert_eq!(app.selection, before.selection);

    let mut incoming = source_document();
    let mut broken = before.document.find_shape(dimension_id).unwrap().clone();
    broken.id = incoming.ids_mut().shape();
    if let ShapeKind::Group { children } = &mut broken.kind {
        for child in children {
            child.id = incoming.ids_mut().shape();
        }
    }
    broken
        .data
        .iter_mut()
        .find(|(key, _)| key == DATA_KEY)
        .unwrap()
        .1 = "{late corrupt dimension JSON".into();
    incoming.pages[0].layers[0].shapes.push(broken);
    app.settings.color.import_cmyk = 0;
    let before_error = snapshot(&app);
    let error = app.prepare_color_import(incoming, None).err().unwrap();
    assert!(
        error.contains("key must be a string"),
        "must reach the corrupt dimension JSON, not an unrelated validation: {error}"
    );
    assert_eq!(
        snapshot(&app),
        before_error,
        "late dimension error must preserve redo, IDs, selection and document"
    );
}

#[test]
#[ignore = "measures a real 512-square worker conversion; run separately"]
fn large_cmyka_worker_records_virtual_frames_and_commit_separately() {
    use crate::import_document::ColorImportPlacement;
    use std::time::{Duration, Instant};
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Info)
        .try_init();
    let root = directory("worker-512");
    let generation = Instant::now();
    let a = fixture::profile(false);
    let mut pixels = Vec::with_capacity(512 * 512 * 5);
    for index in 0..512 * 512usize {
        pixels.extend_from_slice(&[
            index as u8,
            (index >> 8) as u8,
            (((index * 37) >> 4) + (index >> 16) * 61) as u8,
            ((index * 19) >> 7) as u8,
            [0, 1, 127, 255][index % 4],
        ]);
    }
    let bytes =
        tracedraw_io::cmyk_bitmap::cmyka_tiff(512, 512, &pixels, 300.0, Some(a.data.as_slice()))
            .unwrap();
    let path = root.join("owned-512-cmyka.tif");
    std::fs::write(&path, &bytes).unwrap();
    let generation_ms = generation.elapsed().as_secs_f64() * 1000.0;
    let mut app = edited_destination();
    let mut generic = fixture::document(None);
    fixture::add_vector(&mut generic);
    app.page = generic.pages[0].id;
    app.selection = vec![generic.pages[0].layers[0].shapes[0].id];
    app.engine.replace(generic);
    app.settings.color.import_cmyk = 0;
    let snapshot_began = Instant::now();
    let before = snapshot(&app);
    let independent_snapshot_ms = snapshot_began.elapsed().as_secs_f64() * 1000.0;
    let enqueue = Instant::now();
    app.start_color_import(&path, ColorImportPlacement::InFile)
        .unwrap();
    let enqueue_ms = enqueue.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(snapshot(&app), before);
    let ctx = egui::Context::default();
    let waiting = Instant::now();
    let mut worker_frames_ms = Vec::new();
    let mut commit_frame_ms = None;
    while app.color_import_job.is_some() && waiting.elapsed() < Duration::from_secs(180) {
        let frame = Instant::now();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("Independent 512-square import worker measurement");
            app.poll_color_import();
        });
        output.textures_delta.clear();
        let elapsed = frame.elapsed().as_secs_f64() * 1000.0;
        if app.color_import_job.is_some() {
            worker_frames_ms.push(elapsed);
            assert_eq!(app.engine.history_labels(), before.history);
        } else {
            commit_frame_ms = Some(elapsed);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        app.color_import_job.is_none(),
        "worker did not settle in 180 seconds"
    );
    assert!(
        worker_frames_ms.len() >= 2,
        "must observe several frames while conversion runs"
    );
    assert_eq!(
        app.engine.history_labels().0.len(),
        before.history.0.len() + 1
    );
    let bitmap = fixture::bitmaps(app.doc())[0];
    let (samples, embedded, rgba) = fixture::raw_picture(bitmap);
    assert!(
        embedded.is_none(),
        "generic destination must stay explicitly untagged"
    );
    assert_eq!(samples.len(), pixels.len());
    for index in [0usize, 1, 127, 65536, 512 * 512 - 1] {
        let offset = index * 5;
        let expected = fixture::expected_samples(
            &pixels[offset..offset + 5],
            Some(&a),
            None,
            fixture::options(),
        );
        assert_eq!(&samples[offset..offset + 5], expected.as_slice());
        assert_eq!(rgba.as_raw()[index * 4 + 3], pixels[offset + 4]);
    }
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    worker_frames_ms.sort_by(f64::total_cmp);
    let result = serde_json::json!({
        "width":512,"height":512,"pixels":512*512,
        "source":"owned ICC A CMYKA", "target":"explicit generic CMYK press",
        "fixture_generation_ms":generation_ms,
        "independent_snapshot_ms":independent_snapshot_ms,
        "enqueue_including_production_snapshot_ms":enqueue_ms,
        "worker_and_poll_wall_ms":waiting.elapsed().as_secs_f64()*1000.0,
        "virtual_frames_during_worker":worker_frames_ms.len(),
        "virtual_frame_median_ms":worker_frames_ms[worker_frames_ms.len()/2],
        "virtual_frame_p95_ms":worker_frames_ms[worker_frames_ms.len()*95/100],
        "virtual_frame_max_ms":worker_frames_ms.last(),
        "commit_virtual_frame_ms":commit_frame_ms,
        "scope":"minimal egui label and poll only; no desktop, GPU or full editor smoothness claim",
        "checks":"five independent ink/alpha samples, source intact, one history step"
    });
    std::fs::write(
        root.join("timings.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("LUMEN_WORKER_512 {}", result);
}

fn source_files(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
    let source = source_document();
    let native = directory.join("owned-source.tdraw");
    let tiff = directory.join("owned-source.tif");
    let cdr = directory.join("owned-source-v27.cdr");
    std::fs::write(&native, serde_json::to_vec_pretty(&source).unwrap()).unwrap();
    std::fs::write(&tiff, fixture::tiff(source.cmyk_profile.as_ref())).unwrap();
    std::fs::write(
        &cdr,
        tracedraw_cdr::write::document_to_cdr_version(&source, 27),
    )
    .unwrap();
    vec![native, tiff, cdr]
}

fn assert_doc_after_undo(actual: &Document, before: &Document) {
    // Successful undo preserves the monotonic allocator by the engine's public
    // contract. A failed preparation/commit must preserve it exactly instead.
    let mut expected = before.clone();
    expected.set_ids(actual.ids().clone());
    assert_eq!(actual, &expected);
}

#[test]
fn actual_tiff_cdr_and_native_import_are_one_reversible_step_in_an_edited_document() {
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    let root = directory("file-matrix");
    let sources = source_files(&root);
    std::fs::write(
        root.join("owned-destination.tdraw"),
        serde_json::to_vec_pretty(edited_destination().doc()).unwrap(),
    )
    .unwrap();
    let mut evidence = Vec::new();
    for policy in [0, 1, 2] {
        for source in &sources {
            let original_file = std::fs::read(source).unwrap();
            let mut app = edited_destination();
            app.settings.color.import_cmyk = policy;
            let before = snapshot(&app);
            let original_picture = fixture::bitmaps(&before.document)[0].id;
            let imported = app.try_import_path(source, None).unwrap();
            assert!(!imported.is_empty());
            assert_eq!(app.selection, imported);
            assert!(imported
                .iter()
                .all(|id| id.raw() >= before.document.ids().peek_next()));
            assert_eq!(app.doc().title, before.document.title);
            assert_eq!(app.doc().pages.len(), before.document.pages.len());
            assert_eq!(
                app.engine.history_labels().0.len(),
                before.history.0.len() + 1
            );
            let target = if policy == 2 { &a } else { &b };
            assert_eq!(app.doc().cmyk_profile.as_ref().unwrap().data, target.data);
            let incoming_picture = fixture::bitmaps(app.doc())
                .into_iter()
                .find(|shape| shape.id != original_picture)
                .unwrap();
            let expected_incoming = if policy == 0 {
                fixture::expected_samples(&fixture::CMYKA, Some(&a), Some(&b), fixture::options())
            } else {
                fixture::CMYKA.to_vec()
            };
            fixture::assert_picture(incoming_picture, &expected_incoming, Some(target));
            let expected_existing = if policy == 2 {
                fixture::expected_samples(&fixture::CMYKA, Some(&b), Some(&a), fixture::options())
            } else {
                fixture::CMYKA.to_vec()
            };
            fixture::assert_picture(
                app.doc().find_shape(original_picture).unwrap(),
                &expected_existing,
                Some(target),
            );
            if source.extension().unwrap() == "tif" {
                let link = incoming_picture
                    .data
                    .iter()
                    .find(|(key, _)| key == "bitmap.link")
                    .unwrap();
                assert!(std::path::Path::new(&link.1).is_absolute());
                assert_eq!(
                    std::fs::canonicalize(&link.1).unwrap(),
                    std::fs::canonicalize(source).unwrap()
                );
                let tracedraw_core::ShapeKind::Bitmap {
                    rect,
                    width_px,
                    height_px,
                    ..
                } = &incoming_picture.kind
                else {
                    unreachable!()
                };
                assert_eq!((*width_px, *height_px), (2, 2));
                assert!((rect.width() - 2.0 * 25.4 / 300.0).abs() < 1e-7);
                assert!((rect.height() - 2.0 * 25.4 / 300.0).abs() < 1e-7);
            }
            let after = app.doc().clone();
            let after_next_id = after.ids().peek_next();
            app.undo();
            assert_doc_after_undo(app.doc(), &before.document);
            assert!(
                app.doc().ids().peek_next() >= after_next_id,
                "undo never reuses allocated ids"
            );
            assert_eq!(
                app.selection, before.selection,
                "undo restores the prior selection"
            );
            assert_eq!(app.engine.history_labels().0.len(), before.history.0.len());
            app.redo();
            assert_eq!(
                app.doc(),
                &after,
                "redo restores all document resources and pixels"
            );
            assert_eq!(
                app.selection, imported,
                "redo restores the imported selection"
            );
            assert_eq!(
                std::fs::read(source).unwrap(),
                original_file,
                "source file is read-only"
            );
            evidence.push(serde_json::json!({
                "policy": policy, "source": source, "source_bytes": original_file.len(),
                "imported_ids": imported, "undo_redo_exact_except_monotonic_ids": true,
            }));
        }
    }
    std::fs::write(
        root.join("file-matrix.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    std::fs::write(root.join("oracles.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "source_a": a, "destination_b": b, "original_cmyka": fixture::CMYKA,
        "a_to_b_cmyka": fixture::expected_samples(&fixture::CMYKA, Some(&a), Some(&b), fixture::options()),
        "b_to_a_cmyka": fixture::expected_samples(&fixture::CMYKA, Some(&b), Some(&a), fixture::options()),
        "width_px": 2, "height_px": 2, "dpi": 300,
    })).unwrap()).unwrap();
}

#[test]
fn late_malformed_native_symbol_preserves_history_redo_ids_selection_and_sources() {
    use tracedraw_core::document::{DocumentFont, Symbol};
    let root = directory("late-native-error");
    let mut incoming = source_document();
    incoming.fonts.push(DocumentFont::new(
        "TestSymbol",
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").to_vec(),
    ));
    let mut late = fixture::picture(incoming.cmyk_profile.as_ref());
    late.id = incoming.ids_mut().shape();
    late.picture_inks.as_mut().unwrap().png = vec![2, 3, 5, 7].into();
    incoming.symbols.push(Symbol {
        name: "Late malformed bitmap definition".into(),
        shape: late,
    });
    let path = root.join("malformed-owned.tdraw");
    let bytes = serde_json::to_vec_pretty(&incoming).unwrap();
    std::fs::write(&path, &bytes).unwrap();
    let mut app = edited_destination();
    app.settings.color.import_cmyk = 0;
    app.engine
        .run(&Command::SetTitle {
            title: "A step held on the redo stack".into(),
        })
        .unwrap();
    app.undo();
    assert!(!app.engine.history_labels().1.is_empty());
    let before = snapshot(&app);
    assert!(app.try_import_path(&path, None).is_err());
    assert_eq!(
        snapshot(&app),
        before,
        "late errors cannot leave state or resource changes"
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    app.settings.color.import_cmyk = 255;
    let before = snapshot(&app);
    assert!(app.prepare_color_import(source_document(), None).is_err());
    assert_eq!(
        snapshot(&app),
        before,
        "unknown selector is an error, never a fallback"
    );
}

#[test]
fn invalid_destination_icc_and_nonfinite_placement_never_begin_a_transaction() {
    let root = directory("invalid-input");
    let path = &source_files(&root)[0];
    let mut app = edited_destination();
    app.settings.color.import_cmyk = 0;
    for point in [
        tracedraw_core::geometry::Point::new(f64::NAN, 0.0),
        tracedraw_core::geometry::Point::new(0.0, f64::INFINITY),
    ] {
        let before = snapshot(&app);
        assert!(app.try_import_path(path, Some(point)).is_err());
        assert_eq!(snapshot(&app), before);
    }
    let invalid =
        tracedraw_core::document::DocumentProfile::new("Invalid destination", vec![11, 22, 33]);
    app.engine
        .run(&Command::SetCmykProfile {
            profile: Some(invalid),
        })
        .unwrap();
    let before = snapshot(&app);
    assert!(app.try_import_path(path, None).is_err());
    assert_eq!(snapshot(&app), before);
}

#[test]
fn prepared_import_is_rejected_if_instance_revision_page_layer_selection_or_preferences_changed() {
    use tracedraw_core::geometry::Size;
    for change in [
        "instance",
        "revision",
        "page",
        "layer",
        "selection",
        "preferences",
    ] {
        let mut app = edited_destination();
        app.settings.color.import_cmyk = 2;
        if change == "page" {
            app.engine
                .run(&Command::AddPage {
                    name: Some("Other existing page".into()),
                    size: Size::new(100.0, 70.0),
                    layer: None,
                })
                .unwrap();
            app.selection.clear();
        }
        if change == "layer" {
            app.engine
                .run(&Command::AddLayer {
                    page: app.page,
                    name: "Other existing layer".into(),
                })
                .unwrap();
            app.selection.clear();
        }
        let prepared = app.prepare_color_import(source_document(), None).unwrap();
        match change {
            "instance" => {
                let revision = app.engine.revision();
                let mut replacement = tracedraw_core::Engine::new(app.doc().clone());
                while replacement.revision() < revision {
                    replacement
                        .run(&Command::SetTitle {
                            title: app.doc().title.clone(),
                        })
                        .unwrap();
                }
                assert_eq!(
                    replacement.revision(),
                    revision,
                    "identity must be checked independently of revision"
                );
                app.engine = replacement;
            }
            "revision" => {
                app.engine
                    .run(&Command::SetOpacity {
                        shapes: app.selection.clone(),
                        opacity: 0.37,
                    })
                    .unwrap();
            }
            "page" => {
                app.page = app.doc().pages[1].id;
            }
            "layer" => {
                app.picked_layer = Some(app.doc().pages[0].layers[0].id);
            }
            "selection" => {
                app.selection.clear();
            }
            "preferences" => {
                app.settings.color.intent = "Perceptual".into();
            }
            _ => unreachable!(),
        }
        let current = snapshot(&app);
        assert!(app.commit_color_import(prepared).is_err(), "stale {change}");
        assert_eq!(
            snapshot(&app),
            current,
            "stale {change} cannot change the current drawing"
        );
    }
}

#[test]
fn new_default_converts_but_an_explicit_saved_assignment_preference_survives() {
    let settings = crate::settings::Settings::default();
    assert_eq!(settings.color.import_cmyk, 0);
    let mut encoded = serde_json::to_value(settings).unwrap();
    encoded["color"]["import_cmyk"] = serde_json::json!(1);
    let restored: crate::settings::Settings = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(
        restored.color.import_cmyk, 1,
        "preserve the user's explicit setting"
    );
    encoded["color"]
        .as_object_mut()
        .unwrap()
        .remove("import_cmyk");
    let missing: crate::settings::Settings = serde_json::from_value(encoded).unwrap();
    assert_eq!(
        missing.color.import_cmyk, 0,
        "missing fields use the new default"
    );
}

#[test]
fn background_import_cancel_and_stale_revision_do_not_commit_in_a_virtual_egui_frame() {
    use crate::import_document::ColorImportPlacement;
    let root = directory("worker");
    let path = &source_files(&root)[0];
    for cancel in [false, true] {
        let mut app = edited_destination();
        app.settings.color.import_cmyk = 2;
        let before = snapshot(&app);
        app.start_color_import(path, ColorImportPlacement::InFile)
            .unwrap();
        assert_eq!(
            snapshot(&app),
            before,
            "starting work has no document side effects"
        );
        if cancel {
            app.cancel_color_import();
        } else {
            app.engine
                .run(&Command::SetOpacity {
                    shapes: app.selection.clone(),
                    opacity: 0.37,
                })
                .unwrap();
        }
        let expected = snapshot(&app);
        let ctx = egui::Context::default();
        let start = std::time::Instant::now();
        while app.color_import_job.is_some() && start.elapsed() < std::time::Duration::from_secs(5)
        {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.label("Independent import cancellation check");
                app.poll_color_import();
            });
            output.textures_delta.clear();
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(
            app.color_import_job.is_none(),
            "worker must settle after cancel or stale rejection"
        );
        assert_eq!(
            snapshot(&app),
            expected,
            "cancel/stale must preserve intervening user work"
        );
    }
}

fn settle_worker(app: &mut App) {
    let ctx = egui::Context::default();
    let start = std::time::Instant::now();
    while app.color_import_job.is_some() && start.elapsed() < std::time::Duration::from_secs(5) {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("Independent queued import check");
            app.poll_color_import();
        });
        output.textures_delta.clear();
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(
        app.color_import_job.is_none(),
        "background queue must settle"
    );
}

#[test]
fn background_queue_preserves_order_and_cancel_discards_all_waiting_imports() {
    use crate::import_document::ColorImportPlacement;
    let root = directory("queue");
    let paths = source_files(&root);
    let mut app = edited_destination();
    app.settings.color.import_cmyk = 0;
    let history = app.engine.history_labels().0.len();
    app.start_color_import(&paths[0], ColorImportPlacement::InFile)
        .unwrap();
    app.start_color_import(&paths[1], ColorImportPlacement::InFile)
        .unwrap();
    assert_eq!(app.color_import_queue.len(), 1);
    settle_worker(&mut app);
    assert_eq!(app.engine.history_labels().0.len(), history + 2);
    assert_eq!(
        app.selection.len(),
        1,
        "the last queued source is one TIFF picture"
    );
    let selected = app.doc().find_shape(app.selection[0]).unwrap();
    let link = selected
        .data
        .iter()
        .find(|(key, _)| key == "bitmap.link")
        .unwrap();
    assert_eq!(
        std::fs::canonicalize(&link.1).unwrap(),
        std::fs::canonicalize(&paths[1]).unwrap()
    );
    assert!(app.color_import_queue.is_empty());

    let mut app = edited_destination();
    app.settings.color.import_cmyk = 0;
    let before = snapshot(&app);
    app.start_color_import(&paths[0], ColorImportPlacement::InFile)
        .unwrap();
    for _ in 0..32 {
        app.start_color_import(&paths[1], ColorImportPlacement::InFile)
            .unwrap();
    }
    assert_eq!(app.color_import_queue.len(), 32);
    assert!(app
        .start_color_import(&paths[2], ColorImportPlacement::InFile)
        .is_err());
    assert_eq!(snapshot(&app), before);
    app.cancel_color_import();
    settle_worker(&mut app);
    assert!(app.color_import_queue.is_empty());
    assert_eq!(
        snapshot(&app),
        before,
        "cancel before poll cannot commit the active or queued job"
    );
}

fn symbol_indices(shape: &tracedraw_core::Shape, output: &mut Vec<usize>) {
    use tracedraw_core::{Fill, ShapeKind};
    match &shape.kind {
        ShapeKind::SymbolInstance { index } => output.push(*index),
        ShapeKind::Group { children } => children
            .iter()
            .for_each(|child| symbol_indices(child, output)),
        ShapeKind::ClipFrame { frame, contents } => {
            symbol_indices(frame, output);
            contents
                .iter()
                .for_each(|child| symbol_indices(child, output));
        }
        _ => {}
    }
    for fill in std::iter::once(&shape.fill).chain(shape.extra_fills.iter()) {
        if let Fill::Pattern(tracedraw_core::style::Pattern::Vector { shapes, .. }) = fill {
            shapes
                .iter()
                .for_each(|shape| symbol_indices(shape, output));
        }
    }
}

#[test]
fn foreign_symbols_and_odd_master_do_not_bind_to_destination_definitions_or_global_masters() {
    use tracedraw_core::document::{DocumentFont, Layer, MasterScope, Symbol};
    use tracedraw_core::geometry::Size;
    use tracedraw_core::{Fill, Shape, ShapeKind};
    let mut app = edited_destination();
    let mut destination = app.doc().clone();
    let mut existing = destination.pages[0].layers[0].shapes[1].clone();
    existing.id = destination.ids_mut().shape();
    existing.uid = None;
    existing.fill = Fill::Solid(tracedraw_core::Color::rgb8(201, 19, 43));
    destination.symbols.push(Symbol {
        name: "Existing destination definition".into(),
        shape: existing,
    });
    app.engine.replace(destination);
    app.engine
        .run(&Command::SetTitle {
            title: "Edited destination with existing resources".into(),
        })
        .unwrap();
    app.engine
        .run(&Command::AddPage {
            name: Some("Existing even destination page".into()),
            size: Size::new(100.0, 70.0),
            layer: None,
        })
        .unwrap();
    app.page = app.doc().pages[1].id;
    app.selection.clear();
    app.settings.color.import_cmyk = 1;
    let before = snapshot(&app);

    let mut source = source_document();
    let mut inner = fixture::picture(source.cmyk_profile.as_ref());
    inner.id = source.ids_mut().shape();
    inner.uid = None;
    let nested = Shape::new(
        source.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 1 },
    );
    let mut outer = Shape::new(
        source.ids_mut().shape(),
        ShapeKind::Group {
            children: vec![nested],
        },
    );
    let pattern_reference = Shape::new(
        source.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 1 },
    );
    outer
        .extra_fills
        .push(Fill::Pattern(tracedraw_core::style::Pattern::Vector {
            shapes: vec![pattern_reference],
            tile: Size::new(2.0, 2.0),
        }));
    source.symbols.push(Symbol {
        name: "Foreign outer".into(),
        shape: outer,
    });
    source.symbols.push(Symbol {
        name: "Foreign inner".into(),
        shape: inner,
    });
    let instance = Shape::new(
        source.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 0 },
    );
    source.pages[0].layers[0].shapes.push(instance);
    let mut master = Layer::new(source.ids_mut().layer(), "Source background layer");
    master.scope = MasterScope::Odd;
    let mut master_shape = source.pages[0].layers[0].shapes[1].clone();
    master_shape.id = source.ids_mut().shape();
    master_shape.uid = None;
    master.shapes.push(master_shape);
    source.master.push(master);
    source.fonts.push(DocumentFont::new(
        "TestSymbol",
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").to_vec(),
    ));
    let original_source = source.clone();
    let prepared = app.prepare_color_import(source.clone(), None).unwrap();
    assert_eq!(snapshot(&app), before);
    let selected = app.commit_color_import(prepared).unwrap();
    assert_eq!(source, original_source);
    assert_eq!(
        app.doc().pages[0],
        before.document.pages[0],
        "combined import cannot change earlier destination pages"
    );
    assert_eq!(
        app.doc().master,
        before.document.master,
        "foreign masters cannot become global destination masters"
    );
    assert_eq!(app.doc().symbols[0], before.document.symbols[0]);
    assert_eq!(app.doc().fonts, original_source.fonts);
    let outer_index = app
        .doc()
        .symbols
        .iter()
        .position(|symbol| symbol.name == "Foreign outer")
        .unwrap();
    let inner_index = app
        .doc()
        .symbols
        .iter()
        .position(|symbol| symbol.name == "Foreign inner")
        .unwrap();
    assert!(outer_index > 0 && inner_index > 0 && outer_index != inner_index);
    let mut references = Vec::new();
    symbol_indices(&app.doc().symbols[outer_index].shape, &mut references);
    assert_eq!(
        references.len(),
        2,
        "nested and vector-pattern references are both present"
    );
    assert!(references.iter().all(|index| *index == inner_index));
    let mut root_references = Vec::new();
    for id in &selected {
        symbol_indices(app.doc().find_shape(*id).unwrap(), &mut root_references);
    }
    assert!(root_references.contains(&outer_index));
    assert!(
        root_references.len() >= 2,
        "odd source master must appear even on the even destination page"
    );
    let master = app
        .doc()
        .symbols
        .iter()
        .find(|symbol| {
            symbol.name.contains("Source background layer")
                || symbol.shape.name.as_deref() == Some("Source background layer")
        })
        .unwrap();
    assert!(
        master
            .shape
            .data
            .iter()
            .any(|(key, value)| key.to_ascii_lowercase().contains("scope")
                && value.eq_ignore_ascii_case("odd")),
        "retain the source master scope as metadata"
    );
    let after = app.doc().clone();
    let high_water = after.ids().peek_next();
    app.undo();
    assert_doc_after_undo(app.doc(), &before.document);
    assert!(app.doc().ids().peek_next() >= high_water);
    assert_eq!(app.selection, before.selection);
    app.redo();
    assert_eq!(app.doc(), &after);
    assert_eq!(app.selection, selected);
}

#[test]
fn unresolved_foreign_symbol_is_an_atomic_error_even_when_other_resources_are_valid() {
    use tracedraw_core::{Shape, ShapeKind};
    let mut app = edited_destination();
    app.settings.color.import_cmyk = 0;
    let mut incoming = source_document();
    let unresolved = Shape::new(
        incoming.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 777 },
    );
    incoming.pages[0].layers[0].shapes.push(unresolved);
    let before = snapshot(&app);
    assert!(app.prepare_color_import(incoming, None).is_err());
    assert_eq!(snapshot(&app), before);
}
