//! Private, windowless execution of the actual export dialog and export worker.
//! Only explicit isolated settings and previously nonexistent outputs are used.

use crate::{app::App, ui::dialogs::{Dialog, ExportState, EXPORT_FORMATS}};
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    input: Option<PathBuf>,
    output: PathBuf,
    format: Option<String>,
    dpi: Option<f64>,
    quality: Option<u8>,
    color_mode: Option<usize>,
    width_mm: Option<f64>,
    height_mm: Option<f64>,
    #[serde(default)] selection_ids: Vec<u64>,
    #[serde(default)] page: usize,
    jpeg_subsampling: Option<usize>,
    jpeg_progressive: Option<bool>,
    jpeg_optimize_huffman: Option<bool>,
    embed_icc: Option<bool>,
    matte: Option<[u8; 3]>,
    anti_alias: Option<bool>,
    blur: Option<f32>,
    simulate_overprints: Option<bool>,
    overprint_black: Option<bool>,
    use_proof_profile: Option<bool>,
    crop_to_page: Option<bool>,
    scale: Option<f64>,
    scale_y: Option<f64>,
    keep_ratio: Option<bool>,
    keep_size: Option<bool>,
    capture: Option<PathBuf>,
    capture_size: Option<[u32; 2]>,
    capture_wait_ms: Option<u64>,
    #[serde(default)] only_dialog: bool,
    cancel_after_percent: Option<u8>,
}

fn emit(value: serde_json::Value) { println!("{value}"); }

fn text(shape: &egui::Shape, into: &mut Vec<String>) {
    match shape {
        egui::Shape::Text(value) => into.push(value.galley.text().to_owned()),
        egui::Shape::Vec(shapes) => for shape in shapes { text(shape, into); },
        _ => {},
    }
}

fn input(size: [u32; 2]) -> egui::RawInput {
    egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO,
        egui::vec2(size[0] as f32, size[1] as f32))), ..Default::default() }
}

pub(crate) fn run(path: &Path) -> Result<(), String> {
    if std::env::var_os("TRACEDRAW_CONFIG_DIR").is_none() {
        return Err("export diagnosis requires an explicit isolated settings directory".into());
    }
    if std::fs::metadata(path).map_err(|e| e.to_string())?.len() > 64 * 1024 {
        return Err("export diagnosis specification exceeds its budget".into());
    }
    let spec: Spec = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let capture_wait_ms = spec.capture_wait_ms.unwrap_or(12_000);
    if !(100..=120_000).contains(&capture_wait_ms) {
        return Err("diagnostic capture wait must be between 100 and 120000 milliseconds".into());
    }
    if spec.capture_wait_ms.is_some() && spec.capture.is_none() {
        return Err("diagnostic capture wait requires a capture path".into());
    }
    if !spec.output.is_absolute() || spec.output.exists() {
        return Err("export diagnosis requires a new absolute output path".into());
    }
    if !spec.output.parent().is_some_and(Path::is_dir) {
        return Err("export diagnosis output directory must already exist".into());
    }
    let context = egui::Context::default();
    crate::theme::install_fonts(&context);
    let mut app = App::headless();
    App::setup_context(&context);
    crate::ui::apply_theme(&context, &app.settings.theme);
    tracedraw_render::set_pixel_effects(crate::fx::lens_effects);
    let mut capture = crate::profile_capture::Capture::default();
    app.color_import_context = Some(context.clone());
    let clock = Instant::now();
    emit(json!({"event":"initialized","visible_window_created":false}));
    if let Some(source) = &spec.input {
        if !source.is_absolute() { return Err("input path must be absolute".into()); }
        let old_tabs = app.docs.len();
        app.open_path(source.clone());
        while app.cdr_open_job.is_some() {
            if clock.elapsed() > Duration::from_secs(300) { return Err("opening exceeded diagnostic deadline".into()); }
            let mut output = context.run_ui(input([1000, 700]), |ui| {
                app.poll_cdr_open(ui.ctx()); app.show_cdr_open_progress(ui.ctx());
            });
            if spec.capture.is_some() { capture.consume(&mut output)?; }
            output.textures_delta.clear();
            std::thread::sleep(Duration::from_millis(20));
        }
        if app.docs.len() <= old_tabs { return Err(format!("opening failed: {}", app.status)); }
    } else {
        let width = spec.width_mm.unwrap_or(100.0);
        let height = spec.height_mm.unwrap_or(70.0);
        if !(width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0) {
            return Err("invalid synthetic page size".into());
        }
        app.open_document(tracedraw_core::Document::new("Export diagnosis", tracedraw_core::geometry::Size::new(width, height)), None);
        app.new_shape_styled(tracedraw_core::ShapeKind::Rect {
            rect: tracedraw_core::geometry::Rect::new(0.0,0.0,width * 0.5,height), radius:0.0,corners:None,
        }, |shape| { shape.fill=tracedraw_core::Fill::Solid(tracedraw_core::Color::rgb8(220,30,20)); shape.stroke=None; });
    }
    app.page = app.doc().pages.get(spec.page).ok_or("page index out of range")?.id;
    if !spec.selection_ids.is_empty() {
        let page = app.doc().page(app.page).map_err(|e| e.to_string())?;
        let mut pending: Vec<_> = page.layers.iter().flat_map(|layer| &layer.shapes).collect();
        let mut selected = Vec::new();
        while let Some(shape) = pending.pop() {
            if spec.selection_ids.contains(&shape.id.raw()) { selected.push(shape.id); }
            match &shape.kind {
                tracedraw_core::ShapeKind::Group { children } => pending.extend(children),
                tracedraw_core::ShapeKind::ClipFrame { frame, contents } => { pending.push(frame); pending.extend(contents); },
                _ => {},
            }
        }
        if selected.len() != spec.selection_ids.len() { return Err("selection IDs do not match the requested page".into()); }
        app.selection = selected;
    }
    let format = spec.format.as_deref().unwrap_or("jpg");
    let mut state = ExportState { format:EXPORT_FORMATS.iter().position(|(_, ext)| *ext == format).ok_or("unsupported diagnosis format")?, ..Default::default() };
    if format == "jpg" {
        state.size_unit = 2;
        state.color_mode = match app.doc().metadata.primary_color_mode {
            tracedraw_core::document::PrimaryColorMode::Cmyk => 1,
            tracedraw_core::document::PrimaryColorMode::Rgb => 0,
        };
    }
    state.selection_only = !spec.selection_ids.is_empty();
    macro_rules! choice { ($($name:ident),+ $(,)?) => { $(if let Some(value) = spec.$name { state.$name = value; })+ }; }
    choice!(dpi,quality,color_mode,jpeg_subsampling,jpeg_progressive,jpeg_optimize_huffman,embed_icc,matte,anti_alias,blur,simulate_overprints,overprint_black,use_proof_profile,crop_to_page,scale,scale_y,keep_ratio,keep_size);
    emit(json!({"event":"opened","open_ms":clock.elapsed().as_millis(),"page":spec.page,
        "selection_ids":spec.selection_ids,"choices":format!("{state:?}"),"status":app.status,
        "page_size_mm":app.doc().pages.get(spec.page).map(|page| [page.size.width,page.size.height]),
        "rgb_profile_bytes":app.doc().rgb_profile.as_ref().map(|p| p.data.len()),
        "cmyk_profile_bytes":app.doc().cmyk_profile.as_ref().map(|p| p.data.len()),
        "gray_profile_bytes":app.doc().gray_profile.as_ref().map(|p| p.data.len())}));
    if let Some(capture_path) = &spec.capture {
        if capture_path.exists() || !capture_path.is_absolute() { return Err("dialog capture must have a new absolute path".into()); }
        let size = spec.capture_size.unwrap_or([1400, 900]);
        if size[0] < 320 || size[1] < 240 || size[0] > 2560 || size[1] > 1600 { return Err("dialog capture size out of range".into()); }
        app.dialog = Dialog::Export(state.clone());
        let mut painted = Vec::new();
        let preview_started = Instant::now();
        let until = preview_started + Duration::from_millis(capture_wait_ms);
        let mut next_notice = preview_started + Duration::from_secs(1);
        let mut frames = 0;
        let (preview_ready, preview_wait_ms) = loop {
            let mut output = context.run_ui(input(size), |ui| crate::ui::dialogs::show(&mut app, ui.ctx()));
            capture.consume(&mut output)?;
            painted.clear();
            for shape in &output.shapes { text(&shape.shape, &mut painted); }
            frames += 1;
            let ready = crate::ui::jpeg_export::diagnostic_preview_ready(&context);
            if frames >= 10 && (ready || Instant::now() >= until) {
                let wait_ms = preview_started.elapsed().as_millis();
                capture.save(&context, &output, size, capture_path)?;
                break (ready, wait_ms);
            }
            if Instant::now() >= next_notice {
                emit(json!({"event":"preview_wait","elapsed_ms":preview_started.elapsed().as_millis(),
                    "frames":frames,"preview_ready":ready,"visible_window_created":false}));
                next_notice = Instant::now() + Duration::from_secs(1);
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        crate::ui::jpeg_export::cancel_diagnostic_previews(&context);
        app.dialog = Dialog::None;
        emit(json!({"event":"dialog","painted_text":painted,"path":capture_path,
            "preview_ready":preview_ready,"preview_wait_ms":preview_wait_ms,
            "preview_wait_limit_ms":capture_wait_ms,
            "capture_ms":preview_started.elapsed().as_millis(),
            "capture_backend":"CPU meshes and textures from actual egui output","gpu_tested":false,"visible_window_created":false}));
        if !preview_ready {
            return Err("preview did not finish within its diagnostic deadline".into());
        }
    }
    if spec.only_dialog { return Ok(()); }
    let started = Instant::now();
    app.start_export_job(spec.output.clone(), state)?;
    emit(json!({"event":"scheduled","background":app.export_job.is_some(),
        "prepare_ms":started.elapsed().as_millis()}));
    let mut last = String::new();
    let mut frames = 0u64;
    let mut max_frame_ms = 0u128;
    let mut cancelled = false;
    while app.export_job.is_some() {
        if started.elapsed() > Duration::from_secs(600) { app.cancel_export_job(); return Err("export exceeded diagnostic deadline".into()); }
        let frame_start = Instant::now();
        let mut output = context.run_ui(input([1000,700]), |ui| {
            app.poll_export_job(ui.ctx()); app.show_export_progress(ui.ctx());
        });
        max_frame_ms = max_frame_ms.max(frame_start.elapsed().as_millis()); frames += 1;
        let mut painted = Vec::new();
        for shape in &output.shapes { text(&shape.shape,&mut painted); }
        output.textures_delta.clear();
        if app.status != last {
            let progress = app.export_progress();
            emit(json!({"event":"progress","elapsed_ms":started.elapsed().as_millis(),"status":app.status,"painted_text":painted,
                "percent":progress.as_ref().map(|p|p.percent),"completed":progress.as_ref().map(|p|p.completed),"total":progress.as_ref().map(|p|p.total)}));
            last = app.status.clone();
        }
        if !cancelled && spec.cancel_after_percent.is_some_and(|limit| app.export_progress().is_some_and(|p|p.percent >= limit)) {
            app.cancel_export_job(); cancelled = true;
        }
        if app.export_job.is_some() { std::thread::sleep(Duration::from_millis(20)); }
    }
    let bytes = std::fs::metadata(&spec.output).ok().map(|value|value.len());
    emit(json!({"event":"complete","elapsed_ms":started.elapsed().as_millis(),"frames":frames,"max_progress_frame_ms":max_frame_ms,
        "output":spec.output,"output_bytes":bytes,"cancel_requested":cancelled,"status":app.status,"source_saved":false,"visible_window_created":false}));
    if cancelled && bytes.is_some() { return Err("cancelled export unexpectedly committed a file".into()); }
    if !cancelled && bytes.is_none() { return Err(format!("export produced no file: {}",app.status)); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_capture_deadlines_are_rejected_before_opening_any_document() {
        let base = PathBuf::from(std::env::var_os("TRACEDRAW_CONFIG_DIR").expect("isolated test profile"));
        let specification = base.join("invalid-capture-wait.json");
        let output = base.join("unexpected-output.jpg");
        let missing_input = base.join("missing-source.cdr");
        for (wait, capture) in [(99, Some(base.join("unused.png"))),
                                (120_001, Some(base.join("unused.png"))), (100, None)] {
            let request = json!({"input":missing_input,"output":output,"capture":capture,
                                 "capture_wait_ms":wait,"only_dialog":true});
            std::fs::write(&specification, serde_json::to_vec(&request).expect("specification")).expect("write specification");
            let error = run(&specification).expect_err("invalid capture deadline");
            assert!(error.contains("capture wait"), "wrong validation order: {error}");
            assert!(!output.exists());
        }
    }

    #[test]
    fn actual_dialog_capture_consumes_textures_and_paints_the_document_without_exporting() {
        let base = PathBuf::from(std::env::var_os("TRACEDRAW_CONFIG_DIR").expect("isolated test profile"));
        let folder = base.join("capture-regression");
        std::fs::create_dir_all(&folder).expect("capture directory");
        let capture = folder.join("dialog.png");
        let output = folder.join("unused.jpg");
        let specification = folder.join("spec.json");
        let json = json!({"output":output,"capture":capture,"capture_size":[800,600],
            "format":"jpg","dpi":72,"quality":90,"color_mode":0,"capture_wait_ms":20_000,
            "width_mm":100,"height_mm":70,"only_dialog":true});
        std::fs::write(&specification, serde_json::to_vec(&json).expect("specification")).expect("specification file");
        run(&specification).expect("actual dialog capture");
        assert!(!output.exists(), "a dialog-only diagnostic must not export");
        let image = image::open(&capture).expect("complete capture PNG").to_rgb8();
        assert_eq!(image.dimensions(), (800,600));
        let red = image.pixels().filter(|p| p[0] > 150 && p[1] < 80 && p[2] < 80).count();
        assert!(red > 1000, "document preview was not painted: {red} red pixels");
    }
}
