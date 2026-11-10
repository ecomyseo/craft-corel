use super::*;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracedraw_core::{geometry::Rect, Color, Fill, ShapeKind};

fn output_path(label: &str) -> PathBuf {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors().nth(3).expect("workspace").join(".build/export-worker-tests");
    std::fs::create_dir_all(&root).expect("test directory");
    root.join(format!("{label}-{}.png", SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()))
}

fn settle(app: &mut App, context: &egui::Context) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while app.export_job.is_some() {
        assert!(Instant::now() < deadline, "export never completed: {}", app.status);
        let mut output = context.run_ui(egui::RawInput::default(), |ui| {
            app.poll_export_job(ui.ctx());
            app.show_export_progress(ui.ctx());
        });
        output.textures_delta.clear();
        std::thread::yield_now();
    }
}

fn overlay_frame(app: &mut App, context: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1000.0,700.0))),
        events,
        ..Default::default()
    };
    let mut output = context.run_ui(input, |ui| {
        app.poll_export_job(ui.ctx());
        app.show_export_progress(ui.ctx());
    });
    output.textures_delta.clear();
    output
}

fn text_centre(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
    match shape {
        egui::Shape::Text(text) if text.galley.text() == label => Some(text.pos + text.galley.rect.center().to_vec2()),
        egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| text_centre(shape,label)),
        _ => None,
    }
}

#[test]
fn export_snapshot_runs_off_ui_and_survives_later_document_edits() {
    let context = egui::Context::default();
    let mut app = App::headless();
    app.color_import_context = Some(context.clone());
    app.new_shape_styled(ShapeKind::Rect { rect: Rect::new(0.0, 0.0, 30.0, 30.0), radius: 0.0, corners: None }, |shape| {
        shape.fill = Fill::Solid(Color::rgb8(255, 0, 0));
        shape.stroke = None;
    });
    let state = ExportState { format: 2, dpi: 72.0, transparent: false, ..Default::default() };
    let path = output_path("snapshot");
    app.start_export_job(path.clone(), state.clone()).expect("start");
    assert!(app.export_job.is_some(), "result must be published by UI polling");
    assert!(app.start_export_job(output_path("busy"), state).is_err(), "no overlapping writer");
    let mut changed = app.doc().clone();
    changed.pages[0].layers[0].shapes[0].fill = Fill::Solid(Color::rgb8(0, 0, 255));
    app.engine.replace(changed);
    settle(&mut app, &context);
    let image = image::open(&path).expect("actual PNG").to_rgb8();
    // Document coordinates grow upwards from the page's lower-left corner.
    let pixel = image.get_pixel(10, image.height() - 10).0;
    assert!(pixel[0] > 240 && pixel[1] < 15 && pixel[2] < 15, "snapshot changed: {pixel:?}");
    assert_eq!(app.doc().pages[0].layers[0].shapes[0].fill, Fill::Solid(Color::rgb8(0, 0, 255)));
    assert!(app.status.contains(path.file_name().expect("name").to_string_lossy().as_ref()), "completion status: {}", app.status);
}

#[test]
fn cancelling_keeps_the_worker_reserved_until_it_acknowledges() {
    let context = egui::Context::default();
    let mut app = App::headless();
    let (started, receiver) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel();
    app.export_job = Some(ExportWorker::spawn_task(output_path("cancel"), context.clone(), move |_, cancel| {
        started.send(()).expect("started");
        released.recv_timeout(Duration::from_secs(10)).expect("release");
        assert!(cancel.load(Ordering::Relaxed));
        Err("export cancelled".into())
    }).expect("worker"));
    receiver.recv_timeout(Duration::from_secs(10)).expect("running");
    let mut position = None;
    for _ in 0..3 {
        let output = overlay_frame(&mut app, &context, Vec::new());
        position = output.shapes.iter().find_map(|shape| text_centre(&shape.shape,&tr("dialog.cancel"))).or(position);
    }
    let position = position.expect("the actual export overlay must paint its Cancel button");
    for pressed in [true,false] {
        overlay_frame(&mut app, &context, vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {pos:position,button:egui::PointerButton::Primary,pressed,modifiers:egui::Modifiers::default()},
        ]);
    }
    assert!(app.export_job.as_ref().expect("reserved worker").cancel.load(Ordering::Relaxed), "clicking Cancel must signal its worker");
    app.poll_export_job(&context);
    assert!(app.export_job.is_some(), "cancel must not release a still-running exporter");
    assert!(app.start_export_job(output_path("cancel-busy"), ExportState::default()).is_err());
    release.send(()).expect("release");
    settle(&mut app, &context);
    assert_eq!(app.status, tr("status.export_cancelled"));
}

#[test]
fn a_completed_commit_wins_over_late_cancellation() {
    let context = egui::Context::default();
    let mut app = App::headless();
    let (finished, receiver) = std::sync::mpsc::channel();
    app.export_job = Some(ExportWorker::spawn_task(output_path("committed"), context.clone(), move |_, _| {
        finished.send(()).expect("finished");
        Ok("committed output".into())
    }).expect("worker"));
    receiver.recv_timeout(Duration::from_secs(10)).expect("finished");
    app.cancel_export_job();
    settle(&mut app, &context);
    assert_eq!(app.status, "committed output");
}

#[test]
fn dropping_the_ui_state_cancels_its_owned_worker() {
    let context = egui::Context::default();
    let (started, receiver) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel();
    let (finished, done) = std::sync::mpsc::channel();
    let worker = ExportWorker::spawn_task(output_path("drop"), context, move |_, cancel| {
        started.send(()).expect("started");
        released.recv_timeout(Duration::from_secs(10)).expect("release");
        finished.send(cancel.load(Ordering::Relaxed)).expect("finished");
        Err("export cancelled".into())
    }).expect("worker");
    receiver.recv_timeout(Duration::from_secs(10)).expect("started");
    drop(worker);
    release.send(()).expect("release");
    assert!(done.recv_timeout(Duration::from_secs(10)).expect("cancelled"));
}
