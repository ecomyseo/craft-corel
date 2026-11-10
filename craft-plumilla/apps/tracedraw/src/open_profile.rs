//! Windowless diagnosis of the real background opening and progress overlay.
//! Requires an explicit isolated settings folder and never saves the drawing.

use crate::app::App;
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};
use tracedraw_core::document::ShapeKind;

fn emit(value: Value) {
    println!("{value}");
}

fn collect_text(shape: &egui::Shape, output: &mut Vec<String>) {
    match shape {
        egui::Shape::Text(text) => output.push(text.galley.text().to_owned()),
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                collect_text(shape, output);
            }
        }
        _ => {}
    }
}

pub(crate) fn run(path: &Path) -> Result<(), String> {
    if std::env::var_os("CRAFT_PLUMILLA_CONFIG_DIR").is_none()
        && std::env::var_os("TRACEDRAW_CONFIG_DIR").is_none()
    {
        return Err("opening diagnosis requires an isolated settings directory".into());
    }
    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cdr"))
    {
        return Err("opening diagnosis requires a CDR file".into());
    }
    let startup = Instant::now();
    let context = egui::Context::default();
    crate::theme::install_fonts(&context);
    let mut app = App::headless();
    app.color_import_context = Some(context.clone());
    let tabs_before = app.docs.len();
    emit(json!({"event":"initialized", "startup_ms":startup.elapsed().as_millis()}));
    let started = Instant::now();
    app.open_path(path.to_owned());
    emit(
        json!({"event":"scheduled", "elapsed_ms":started.elapsed().as_millis(),
                "background":app.cdr_open_job.is_some()}),
    );
    let mut last_status = String::new();
    let mut last_painted = Vec::<String>::new();
    let mut frames = 0u64;
    let mut max_frame_ms = 0u128;
    while app.cdr_open_job.is_some() {
        if started.elapsed() > Duration::from_secs(300) {
            return Err("background opening exceeded the 300-second diagnostic limit".into());
        }
        let frame_started = Instant::now();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 700.0),
            )),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            app.poll_cdr_open(ui.ctx());
            app.show_cdr_open_progress(ui.ctx());
        });
        max_frame_ms = max_frame_ms.max(frame_started.elapsed().as_millis());
        frames += 1;
        let mut painted = Vec::new();
        for shape in &output.shapes {
            collect_text(&shape.shape, &mut painted);
        }
        // This diagnosis consumes text geometry rather than uploading textures
        // to a GPU. A headless egui integration must still consume the delta.
        output.textures_delta.clear();
        if app.status != last_status || painted != last_painted {
            emit(
                json!({"event":"progress", "elapsed_ms":started.elapsed().as_millis(),
                        "status":app.status, "painted_text":painted,
                        "loading":app.cdr_open_job.is_some()}),
            );
            last_status = app.status.clone();
            last_painted = painted;
        }
        if app.cdr_open_job.is_some() {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    let ready_ms = started.elapsed().as_millis();
    if app.docs.len() <= tabs_before {
        return Err(format!(
            "opening did not publish a document: {}",
            app.status
        ));
    }
    let document = app.doc();
    let cmyk_crc32 = document.cmyk_profile.as_ref().map(|profile| {
        let mut crc = flate2::Crc::new();
        crc.update(profile.data.as_slice());
        format!("{:08x}", crc.sum())
    });
    let mut pending: Vec<_> = document
        .all_layers()
        .flat_map(|layer| layer.shapes.iter())
        .collect();
    let mut bitmaps = Vec::new();
    let mut objects = 0u64;
    while let Some(shape) = pending.pop() {
        objects += 1;
        match &shape.kind {
            ShapeKind::Group { children } => pending.extend(children.iter()),
            ShapeKind::ClipFrame { frame, contents } => {
                pending.push(frame);
                pending.extend(contents.iter());
            }
            ShapeKind::Bitmap { width_px, height_px, png, .. } => bitmaps.push(json!({
                "shape_id":shape.id.raw(), "width_px":width_px, "height_px":height_px,
                "png_bytes":png.len(), "cmyk":shape.cmyk_bitmap,
                "ink_profile_bytes":shape.picture_inks.as_ref().and_then(|ink| ink.profile.as_ref()).map(|p|p.data.len())
            })),
            _ => {}
        }
    }
    emit(
        json!({"event":"complete", "ready_ms":ready_ms, "frames":frames,
        "max_progress_frame_ms":max_frame_ms, "title":document.title,
        "pages":document.pages.len(), "objects_including_groups_and_clips":objects,
        "page_sizes_mm":document.pages.iter().map(|page| json!({"width":page.size.width,"height":page.size.height})).collect::<Vec<_>>(),
        "bitmaps":bitmaps, "palette_colours":document.palette.len(),
        "embedded_fonts":document.fonts.len(),
        "cmyk_profile_bytes":document.cmyk_profile.as_ref().map(|p|p.data.len()),
        "cmyk_profile_crc32":cmyk_crc32,
        "rgb_profile_bytes":document.rgb_profile.as_ref().map(|p|p.data.len()),
        "gray_profile_bytes":document.gray_profile.as_ref().map(|p|p.data.len()),
        "color_context":document.color_context,
        "status":app.status, "source_saved":false, "visible_window_created":false}),
    );
    Ok(())
}
