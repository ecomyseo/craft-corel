use super::*;
use std::time::{Duration, Instant};
use tracedraw_core::Document;

fn fixture(title: &str) -> Document {
    let mut doc = Document::default();
    doc.title = title.into();
    let mut ids = doc.ids().clone();
    let image = image::RgbaImage::from_pixel(2, 2, image::Rgba([11, 33, 77, 255]));
    let png = crate::bitmap_fx::encode(&image).expect("small bitmap");
    for index in 0..4 {
        if index > 0 {
            let mut page = doc.pages[0].clone();
            page.id = ids.page();
            page.layers[0].id = ids.layer();
            page.layers[0].shapes.clear();
            doc.pages.push(page);
        }
        doc.pages[index].name = format!("page-{}", index + 1);
        doc.pages[index].layers[0].shapes.push(Shape::new(
            ids.shape(),
            ShapeKind::Bitmap {
                rect: Rect::new(10.0, 10.0, 20.0, 20.0),
                width_px: 2,
                height_px: 2,
                png: png.clone().into(),
                fx: None,
            },
        ));
    }
    doc.set_ids(ids);
    doc
}

fn prepared(title: &str) -> PreparedCdrOpen {
    let document = fixture(title);
    PreparedCdrOpen {
        colors: prepare_screen_colors(&document, &Default::default()),
        engine: Engine::new(document),
        report: tracedraw_cdr::ParseReport {
            pages: 4,
            shapes: 4,
            ..Default::default()
        },
        color_notice: None,
        color_error: None,
        thumbnail: None,
    }
}

fn settle(app: &mut App, context: &egui::Context) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.cdr_open_job.is_some() {
        app.poll_cdr_open(context);
        assert!(
            Instant::now() < deadline,
            "small opening worker must finish"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn normalized(document: &Document) -> serde_json::Value {
    fn visit(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(object) => {
                object.remove("uid");
                for item in object.values_mut() {
                    visit(item);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    visit(item);
                }
            }
            _ => {}
        }
    }
    let mut value = serde_json::to_value(document).expect("document JSON");
    visit(&mut value);
    value
}

#[test]
fn worker_payload_is_send_and_progress_never_finishes_before_publication() {
    fn send<T: Send>() {}
    send::<PreparedCdrOpen>();
    send::<PreparedScreenColors>();
    send::<Engine>();
    let progress = AtomicU8::new(0);
    advance(&progress, 42);
    advance(&progress, 12);
    assert_eq!(progress.load(Ordering::Relaxed), 42);
    advance(&progress, 100);
    assert_eq!(progress.load(Ordering::Relaxed), 99);
}

#[test]
fn held_worker_paints_real_overlay_at_42_and_keeps_ui_events_and_old_document() {
    crate::i18n::with_thread_language("en", || {
        let context = crate::theme::ui_context();
        let mut app = App::headless();
        app.color_import_context = Some(context.clone());
        let old = app.doc().clone();
        let old_history = app.engine.history_labels();
        let tabs = app.docs.len();
        let (release, wait) = std::sync::mpsc::channel();
        let (announced, announcement) = std::sync::mpsc::channel();
        app.cdr_open_job = Some(
            spawn_cdr_open(
                PathBuf::from("held.cdr"),
                app.settings.color.clone(),
                context.clone(),
                move |_, _, progress| {
                    advance(&progress, 42);
                    announced.send(()).map_err(|error| error.to_string())?;
                    wait.recv().map_err(|error| error.to_string())?;
                    Ok(prepared("complete"))
                },
            )
            .expect("worker"),
        );
        announcement
            .recv_timeout(Duration::from_secs(5))
            .expect("real worker progress");
        let progress = Arc::clone(&app.cdr_open_job.as_ref().unwrap().progress);
        let mut capture = ui_capture::Capture::default();
        let previous_directory = std::env::var_os("CRAFT_PLUMILLA_RULER_CAPTURE_DIR");
        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match &self.0 {
                    Some(value) => std::env::set_var("CRAFT_PLUMILLA_RULER_CAPTURE_DIR", value),
                    None => std::env::remove_var("CRAFT_PLUMILLA_RULER_CAPTURE_DIR"),
                }
            }
        }
        let _restore = Restore(previous_directory.clone());
        if let Some(parent) = previous_directory {
            let directory = PathBuf::from(parent).join("cdr-open-progress");
            std::fs::create_dir_all(&directory).unwrap();
            std::env::set_var("CRAFT_PLUMILLA_RULER_CAPTURE_DIR", directory);
        }
        let mut clicked = 0usize;
        let mut button = egui::Rect::NOTHING;
        let frame = |app: &mut App,
                     events: Vec<egui::Event>,
                     time: f64,
                     clicked: &mut usize,
                     button: &mut egui::Rect| {
            context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ui| {
                    app.poll_cdr_open(ui.ctx());
                    let response = ui.button("Continue editing");
                    *button = response.rect;
                    if response.clicked() {
                        *clicked += 1;
                    }
                    app.show_cdr_open_progress(ui.ctx());
                },
            )
        };
        let mut last = None;
        for index in 0..4 {
            let mut out = frame(
                &mut app,
                Vec::new(),
                index as f64 * 0.1,
                &mut clicked,
                &mut button,
            );
            capture.frame(&context, &out, false);
            out.textures_delta.clear();
            last = Some(out);
        }
        fn texts(shape: &egui::Shape, out: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(text) => out.push(text.galley.text().to_string()),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        texts(shape, out);
                    }
                }
                _ => {}
            }
        }
        let output = last.as_ref().unwrap();
        let mut painted = Vec::new();
        for clipped in &output.shapes {
            texts(&clipped.shape, &mut painted);
        }
        assert!(
            painted
                .iter()
                .any(|text| text.contains("Reading") && text.contains("42")),
            "{painted:?}"
        );
        assert_eq!(app.doc(), &old);
        assert_eq!(app.docs.len(), tabs);
        assert_eq!(app.engine.history_labels(), old_history);
        assert_eq!(progress.load(Ordering::Relaxed), 42);
        capture.frame(&context, output, true);
        let at = button.center();
        for (index, pressed) in [true, false].into_iter().enumerate() {
            let mut out = frame(
                &mut app,
                vec![
                    egui::Event::PointerMoved(at),
                    egui::Event::PointerButton {
                        pos: at,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                0.5 + index as f64 * 0.1,
                &mut clicked,
                &mut button,
            );
            capture.frame(&context, &out, false);
            out.textures_delta.clear();
        }
        assert_eq!(
            clicked, 1,
            "UI still consumes pointer events while reader is held"
        );
        release.send(()).unwrap();
        settle(&mut app, &context);
        assert_eq!(app.doc().title, "complete");
        assert_eq!(app.doc().pages.len(), 4);
        assert_eq!(
            app.doc()
                .all_layers()
                .map(|layer| layer.shapes.len())
                .sum::<usize>(),
            4
        );
        assert_eq!(app.docs.len(), tabs + 1);
        assert!(!app.engine.is_dirty());
        assert_eq!(progress.load(Ordering::Relaxed), 100);
    });
}

#[test]
fn worker_error_leaves_document_tabs_and_history_intact() {
    let context = crate::theme::ui_context();
    let mut app = App::headless();
    let old = app.doc().clone();
    let history = app.engine.history_labels();
    let tabs = app.docs.len();
    app.cdr_open_job = Some(
        spawn_cdr_open(
            PathBuf::from("failed.cdr"),
            app.settings.color.clone(),
            context.clone(),
            |_, _, progress| {
                advance(&progress, 31);
                Err("reader sentinel error".into())
            },
        )
        .unwrap(),
    );
    settle(&mut app, &context);
    assert_eq!(app.doc(), &old);
    assert_eq!(app.docs.len(), tabs);
    assert_eq!(app.engine.history_labels(), history);
    assert!(app.status.contains("reader sentinel error"));
}

#[test]
fn opening_queue_keeps_one_worker_and_publishes_every_file_in_fifo_order() {
    let context = crate::theme::ui_context();
    let mut app = App::headless();
    app.color_import_context = Some(context.clone());
    let directory = std::env::temp_dir().join(format!("cdr-open-fifo-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let second = directory.join("second.cdr");
    let third = directory.join("third.cdr");
    let bytes = tracedraw_cdr::write::document_to_cdr_version(&fixture("small"), 12);
    std::fs::write(&second, &bytes).unwrap();
    std::fs::write(&third, &bytes).unwrap();
    let (release, wait) = std::sync::mpsc::channel();
    let (begun, begin) = std::sync::mpsc::channel();
    let tabs = app.docs.len();
    app.cdr_open_job = Some(
        spawn_cdr_open(
            PathBuf::from("first.cdr"),
            app.settings.color.clone(),
            context.clone(),
            move |_, _, progress| {
                advance(&progress, 42);
                begun.send(()).map_err(|error| error.to_string())?;
                wait.recv().map_err(|error| error.to_string())?;
                Ok(prepared("first"))
            },
        )
        .unwrap(),
    );
    begin.recv_timeout(Duration::from_secs(5)).unwrap();
    let reader = app.cdr_open_job.as_ref().unwrap().progress.clone();
    let existing = directory.join("already-open.cdr");
    std::fs::write(&existing, &bytes).unwrap();
    app.opened_from = Some(crate::files::canonical(&existing));
    app.open_path(existing);
    assert!(!app
        .cdr_open_job
        .as_ref()
        .unwrap()
        .discarded
        .load(Ordering::Relaxed));
    app.open_path(second.clone());
    app.open_path(third.clone());
    app.open_path(second.clone());
    let job = app.cdr_open_job.as_ref().unwrap();
    assert!(Arc::ptr_eq(&reader, &job.progress));
    assert_eq!(job.path, PathBuf::from("first.cdr"));
    assert_eq!(
        job.pending
            .iter()
            .map(|request| request.path.clone())
            .collect::<Vec<_>>(),
        vec![second, third]
    );
    assert_eq!(app.docs.len(), tabs);
    release.send(()).unwrap();
    settle(&mut app, &context);
    assert_eq!(app.docs.len(), tabs + 3);
    let mut titles = Vec::new();
    for index in tabs..tabs + 3 {
        app.switch_document(index);
        titles.push(app.doc().title.clone());
    }
    assert_eq!(titles, ["first", "second", "third"]);
    assert_eq!(reader.load(Ordering::Relaxed), 100);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn opening_queue_deduplicates_canonical_aliases_and_rechecks_before_reading() {
    let context = crate::theme::ui_context();
    let mut app = App::headless();
    app.color_import_context = Some(context.clone());
    let directory = std::env::temp_dir().join(format!("cdr-open-alias-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("subdir")).unwrap();
    let first = directory.join("first.cdr");
    let second = directory.join("second.cdr");
    let third = directory.join("third.cdr");
    let first_alias = directory.join("subdir").join("..").join("first.cdr");
    let second_alias = directory.join("subdir").join("..").join("second.cdr");
    let bytes = tracedraw_cdr::write::document_to_cdr_version(&fixture("small"), 12);
    for path in [&first, &second, &third] {
        std::fs::write(path, &bytes).unwrap();
    }
    assert_ne!(second, second_alias);
    assert!(same_open_file(&second, &second_alias));
    let (release, wait) = std::sync::mpsc::channel();
    let (begun, begin) = std::sync::mpsc::channel();
    let tabs = app.docs.len();
    app.cdr_open_job = Some(
        spawn_cdr_open(
            first.clone(),
            app.settings.color.clone(),
            context.clone(),
            move |_, _, progress| {
                advance(&progress, 42);
                begun.send(()).map_err(|error| error.to_string())?;
                wait.recv().map_err(|error| error.to_string())?;
                Ok(prepared("first"))
            },
        )
        .unwrap(),
    );
    begin.recv_timeout(Duration::from_secs(5)).unwrap();
    let reader = app.cdr_open_job.as_ref().unwrap().progress.clone();
    app.open_path(first_alias);
    app.open_path(second.clone());
    app.open_path(second_alias.clone());
    app.open_path(third.clone());
    let job = app.cdr_open_job.as_ref().unwrap();
    assert!(Arc::ptr_eq(&reader, &job.progress));
    assert_eq!(job.path, first);
    assert_eq!(
        job.pending
            .iter()
            .map(|request| request.path.clone())
            .collect::<Vec<_>>(),
        vec![second.clone(), third.clone()]
    );
    release.send(()).unwrap();
    settle(&mut app, &context);
    assert_eq!(app.docs.len(), tabs + 3);
    let mut titles = Vec::new();
    for index in tabs..tabs + 3 {
        app.switch_document(index);
        titles.push(app.doc().title.clone());
    }
    assert_eq!(titles, ["first", "second", "third"]);
    assert_eq!(reader.load(Ordering::Relaxed), 100);
    // Private consumer case: deliberately inject an alias that now identifies
    // an open, edited tab. Public enqueue dedup above is tested separately.
    app.switch_document(tabs + 1);
    app.engine
        .run(&tracedraw_core::Command::SetTitle {
            title: "edited second".into(),
        })
        .unwrap();
    let edited = app.doc().clone();
    let history = app.engine.history_labels();
    assert!(!history.0.is_empty());
    app.switch_document(tabs + 2);
    app.continue_cdr_queue(
        CdrOpenRequest {
            path: second_alias,
            mode: CdrOpenMode::Open,
        },
        VecDeque::new(),
    );
    assert!(
        app.cdr_open_job.is_none(),
        "consumer must not read the alias again"
    );
    assert_eq!(app.active_doc, tabs + 1);
    assert_eq!(app.docs.len(), tabs + 3);
    assert_eq!(app.doc(), &edited);
    assert_eq!(app.engine.history_labels(), history);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn small_cdr_sync_and_async_keep_identical_document_pixels_profiles_and_clean_history() {
    let context = crate::theme::ui_context();
    let directory = std::env::temp_dir().join(format!("cdr-open-parity-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("same.cdr");
    std::fs::write(
        &path,
        tracedraw_cdr::write::document_to_cdr_version(&fixture("small"), 12),
    )
    .unwrap();
    let mut sync = App::headless();
    sync.try_open_path(path.clone()).unwrap();
    let mut asynchronous = App::headless();
    asynchronous.color_import_context = Some(context.clone());
    asynchronous.open_path(path.clone());
    assert!(asynchronous.cdr_open_job.is_some());
    settle(&mut asynchronous, &context);
    assert_eq!(normalized(sync.doc()), normalized(asynchronous.doc()));
    assert_eq!(sync.cdr_version, asynchronous.cdr_version);
    assert_eq!(sync.opened_from, asynchronous.opened_from);
    assert_eq!(sync.file, asynchronous.file);
    assert!(!asynchronous.engine.is_dirty());
    assert!(asynchronous.engine.history_labels().0.is_empty());
    assert_eq!(asynchronous.doc().pages.len(), 4);
    assert!(asynchronous
        .doc()
        .all_layers()
        .flat_map(|layer| &layer.shapes)
        .all(|shape| matches!(
            shape.kind,
            ShapeKind::Bitmap {
                width_px: 2,
                height_px: 2,
                ..
            }
        )));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn cdr_reopen_and_template_also_use_worker_and_keep_their_origin_contracts() {
    let context = crate::theme::ui_context();
    let directory = std::env::temp_dir().join(format!("cdr-open-modes-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("modes.cdr");
    std::fs::write(
        &path,
        tracedraw_cdr::write::document_to_cdr_version(&fixture("small"), 12),
    )
    .unwrap();
    let mut app = App::headless();
    app.try_open_path(path.clone()).unwrap();
    let version = app.cdr_version;
    let tabs = app.docs.len();
    app.color_import_context = Some(context.clone());
    app.open_path_with_password(path.clone(), "");
    assert!(app.cdr_open_job.is_some());
    assert_eq!(app.docs.len(), tabs);
    settle(&mut app, &context);
    assert_eq!(app.docs.len(), tabs + 1);
    assert_eq!(app.opened_from, Some(crate::files::canonical(&path)));
    app.open_as_template(path);
    assert!(app.cdr_open_job.is_some());
    settle(&mut app, &context);
    assert_eq!(app.docs.len(), tabs + 2);
    assert_eq!(app.opened_from, None);
    assert_eq!(app.file, None);
    assert_eq!(app.cdr_version, version);
    assert_eq!(app.doc().pages.len(), 4);
    assert!(!app.engine.is_dirty());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn automation_open_stays_synchronous_and_discards_a_late_ui_result() {
    let context = crate::theme::ui_context();
    let directory =
        std::env::temp_dir().join(format!("cdr-open-automation-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("automation.cdr");
    std::fs::write(
        &path,
        tracedraw_cdr::write::document_to_cdr_version(&fixture("small"), 12),
    )
    .unwrap();
    let mut app = App::headless();
    app.color_import_context = Some(context.clone());
    let (release, wait) = std::sync::mpsc::channel();
    let (announced, announcement) = std::sync::mpsc::channel();
    app.cdr_open_job = Some(
        spawn_cdr_open(
            PathBuf::from("late.cdr"),
            app.settings.color.clone(),
            context.clone(),
            move |_, _, progress| {
                advance(&progress, 42);
                announced.send(()).map_err(|error| error.to_string())?;
                wait.recv().map_err(|error| error.to_string())?;
                Ok(prepared("late"))
            },
        )
        .unwrap(),
    );
    announcement.recv_timeout(Duration::from_secs(5)).unwrap();
    app.try_open_path(path.clone()).unwrap();
    assert_eq!(
        app.doc().pages.len(),
        4,
        "automation returns only after installation"
    );
    assert_eq!(app.doc().title, "automation");
    let delivered = app.doc().clone();
    let tabs = app.docs.len();
    release.send(()).unwrap();
    settle(&mut app, &context);
    assert_eq!(app.doc(), &delivered);
    assert_eq!(
        app.docs.len(),
        tabs,
        "late UI completion cannot override automation"
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn async_cdr_keeps_embedded_cmyk_icc_alpha_planes_and_clipframe_like_sync() {
    let context = crate::theme::ui_context();
    let profile = tracedraw_core::icc::test_profiles::lut16_cmyk_profile();
    let samples = [
        17, 33, 49, 65, 127, 24, 48, 72, 96, 255, 0, 0, 0, 128, 63, 32, 64, 96, 128, 0,
    ];
    let tiff =
        tracedraw_io::cmyk_bitmap::cmyka_tiff(2, 2, &samples, 300.0, Some(&profile)).unwrap();
    let (png, inks) = tracedraw_io::picture::read(&tiff)
        .unwrap()
        .into_png()
        .unwrap();
    let expected_alpha: Vec<u8> = image::load_from_memory(&png)
        .unwrap()
        .to_rgba8()
        .pixels()
        .map(|pixel| pixel[3])
        .collect();
    let mut document = fixture("profiled");
    document.cmyk_profile = Some(tracedraw_core::DocumentProfile::new(
        "small CMYK",
        profile.clone(),
    ));
    let mut bitmap = document.pages[0].layers[0].shapes.remove(0);
    if let ShapeKind::Bitmap { png: bytes, .. } = &mut bitmap.kind {
        *bytes = png.into();
    }
    bitmap.cmyk_bitmap = true;
    bitmap.picture_inks = inks;
    let mut ids = document.ids().clone();
    let frame = Shape::new(
        ids.shape(),
        ShapeKind::Rect {
            rect: Rect::new(10.0, 10.0, 20.0, 20.0),
            radius: 0.0,
            corners: None,
        },
    );
    let clipped = Shape::new(
        ids.shape(),
        ShapeKind::ClipFrame {
            frame: Box::new(frame),
            contents: vec![bitmap],
        },
    );
    document.pages[0].layers[0].shapes.push(clipped);
    document.set_ids(ids);
    let directory =
        std::env::temp_dir().join(format!("cdr-open-profile-alpha-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let path = directory.join("profiled.cdr");
    std::fs::write(
        &path,
        tracedraw_cdr::write::document_to_cdr_version(&document, 17),
    )
    .unwrap();
    let mut sync = App::headless();
    sync.try_open_path(path.clone()).unwrap();
    let mut asynchronous = App::headless();
    asynchronous.color_import_context = Some(context.clone());
    asynchronous.open_path(path);
    settle(&mut asynchronous, &context);
    assert_eq!(normalized(sync.doc()), normalized(asynchronous.doc()));
    assert_eq!(
        asynchronous
            .doc()
            .cmyk_profile
            .as_ref()
            .unwrap()
            .data
            .as_slice(),
        profile
    );
    let mut pending: Vec<&Shape> = asynchronous
        .doc()
        .all_layers()
        .flat_map(|layer| &layer.shapes)
        .collect();
    let mut clips = 0;
    let mut retained = 0;
    while let Some(shape) = pending.pop() {
        match &shape.kind {
            ShapeKind::Group { children } => pending.extend(children),
            ShapeKind::ClipFrame { frame, contents } => {
                clips += 1;
                pending.push(frame);
                pending.extend(contents);
            }
            ShapeKind::Bitmap {
                png,
                width_px,
                height_px,
                ..
            } if shape.cmyk_bitmap => {
                retained += 1;
                assert_eq!((*width_px, *height_px), (2, 2));
                let inks = shape.picture_inks.as_ref().unwrap();
                assert!(inks.fits(png));
                assert_eq!(inks.profile.as_ref().unwrap().data.as_slice(), profile);
                let actual: Vec<u8> = image::load_from_memory(png)
                    .unwrap()
                    .to_rgba8()
                    .pixels()
                    .map(|pixel| pixel[3])
                    .collect();
                assert_eq!(actual, expected_alpha);
            }
            _ => {}
        }
    }
    assert_eq!(clips, 1, "clip frame retained as editable structure");
    assert_eq!(
        retained, 1,
        "CMYK bitmap and its retained planes are present"
    );
    assert!(!asynchronous.engine.is_dirty());
    assert!(asynchronous.engine.history_labels().0.is_empty());
    std::fs::remove_dir_all(directory).unwrap();
}
