//! Export (vector and raster formats), print (through a PDF handed to
//! the system viewer), print merge, find and replace.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::ui::dialogs::{
    EXPORT_FORMATS, ExportColorMode, ExportState, FindReplaceState, PrintState, PropertyFind,
};
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tracedraw_core::find::{self as objfind, ColorQuery, Hit, ObjectQuery, ObjectType};
use tracedraw_core::{
    Command, Document, ShapeId, TextSpan,
    document::ShapeKind,
    geometry::{Affine, Rect, Size},
};

pub const CANCELLED: &str = "export cancelled";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportPhase {
    Render,
    Encode,
    Write,
    Complete,
}
#[derive(Clone, Copy, Debug)]
pub struct ExportProgress {
    pub phase: ExportPhase,
    pub percent: u8,
    pub completed: u64,
    pub total: u64,
}
pub type ExportProgressCallback = Arc<dyn Fn(ExportProgress) + Send + Sync>;

// Exact memo of final quantized RGB/Gray bytes. The prepared job's colour
// configuration is immutable, so entries cannot cross profiles/intents/BPC.
const RGB_MEMO_MIN_PIXELS: u64 = 8 * 1024 * 1024;
const RGB_MEMO_BYTES: usize = (1 << 24) * std::mem::size_of::<u32>();
struct ExactRgbMemo {
    slots: Vec<u32>,
}
impl ExactRgbMemo {
    fn try_new(byte_budget: usize) -> Option<Self> {
        if byte_budget < RGB_MEMO_BYTES {
            return None;
        }
        let mut slots = Vec::new();
        slots.try_reserve_exact(1 << 24).ok()?;
        slots.resize(1 << 24, 0);
        Some(Self { slots })
    }
    fn key(rgb: [u8; 3]) -> usize {
        usize::from(rgb[0]) | (usize::from(rgb[1]) << 8) | (usize::from(rgb[2]) << 16)
    }
    fn get(&self, rgb: [u8; 3]) -> Option<[u8; 3]> {
        let value = self.slots[Self::key(rgb)];
        (value & 0x8000_0000 != 0).then_some([value as u8, (value >> 8) as u8, (value >> 16) as u8])
    }
    fn put(&mut self, rgb: [u8; 3], converted: [u8; 3]) {
        self.slots[Self::key(rgb)] = 0x8000_0000
            | u32::from(converted[0])
            | (u32::from(converted[1]) << 8)
            | (u32::from(converted[2]) << 16);
    }
}
#[derive(Default)]
struct ExactRgbMemoSlot {
    attempted: bool,
    table: Option<ExactRgbMemo>,
}
#[derive(Clone, Copy)]
enum OutputConversion {
    Auto,
    #[cfg(test)]
    Reference,
    #[cfg(test)]
    Memo {
        byte_budget: usize,
    },
}
impl OutputConversion {
    fn identity(self) -> bool {
        matches!(self, Self::Auto)
    }
    fn memo_budget(self, pixels: u64) -> Option<usize> {
        match self {
            Self::Auto => (pixels >= RGB_MEMO_MIN_PIXELS).then_some(RGB_MEMO_BYTES),
            #[cfg(test)]
            Self::Reference => None,
            #[cfg(test)]
            Self::Memo { byte_budget } => Some(byte_budget),
        }
    }
}

/// All document, settings and profile ownership crosses the worker boundary;
/// no App or UI context is retained by the export operation.
pub struct ExportJob {
    doc: Document,
    state: ExportState,
    path: PathBuf,
    ext: &'static str,
    numbers: Vec<usize>,
    title: String,
    cmyk: Option<Vec<u8>>,
    rgb: Option<Vec<u8>>,
    gray: Option<Vec<u8>>,
    proof: Option<Vec<u8>>,
    pdf_profile: Option<Vec<u8>>,
    screen: Option<Arc<tracedraw_core::color::IccEngine>>,
    map_gray: bool,
    profile_name: String,
    base_sizes: Vec<(f64, f64)>,
    intent: tracedraw_core::icc::Intent,
    bpc: bool,
    rgb_memo: std::cell::RefCell<ExactRgbMemoSlot>,
}
pub struct PreviewJob {
    job: ExportJob,
    max_px: u32,
    region: Option<((u32, u32), (u32, u32))>,
}
pub struct ExportPreviewResult {
    pub image: tiny_skia::Pixmap,
    pub estimated_bytes: Option<usize>,
    pub original: tiny_skia::Pixmap,
    pub output_size: (u32, u32),
    pub profile_name: String,
    pub profile_bytes: usize,
    pub base_size_mm: (f64, f64),
}

// Cropping follows render visibility without dropping clip geometry or changing
// the editable source snapshot. A hidden child cannot enlarge a visible group.
fn visible_export_shape(shape: &tracedraw_core::Shape) -> Option<tracedraw_core::Shape> {
    if !shape.visible {
        return None;
    }
    let mut copy = shape.clone();
    match &mut copy.kind {
        ShapeKind::Group { children } => {
            *children = children.iter().filter_map(visible_export_shape).collect();
            if children.is_empty() {
                return None;
            }
        }
        ShapeKind::ClipFrame { contents, .. } => {
            *contents = contents.iter().filter_map(visible_export_shape).collect()
        }
        _ => {}
    }
    Some(copy)
}

pub fn prepare_job(app: &App, path: &Path, st: &ExportState) -> Result<ExportJob, String> {
    let (_, ext) = EXPORT_FORMATS
        .get(st.format)
        .ok_or("invalid export format")?;
    let mut normalized = st.clone();
    if *ext != "jpg" {
        // A JPEG minimum of zero can remain in the shared dialog state when
        // switching format. Other raster encoders retain their valid minimum.
        normalized.quality = normalized.quality.max(1);
        normalized.keep_ratio = true;
        normalized.scale_y = normalized.scale;
        normalized.crop_to_page = true;
        normalized.blur = 0.0;
        normalized.anti_alias = true;
        normalized.simulate_overprints = false;
        normalized.overprint_black = false;
        normalized.use_proof_profile = false;
        normalized.matte = [255; 3];
    }
    let st = &normalized;
    let (mut doc, _) = export_document(app, st);
    if doc.pages.is_empty() {
        return Err("no page to export".into());
    }
    let raster = matches!(
        *ext,
        "jpg" | "png" | "tiff" | "tif" | "bmp" | "gif" | "webp" | "psd"
    );
    // Materialize applicable master layers in drawing order before per-page
    // cropping or non-uniform scaling. The immutable source document stays
    // untouched, and page-number fields retain the source page index.
    if raster && !doc.master.is_empty() {
        let masters = std::mem::take(&mut doc.master);
        for (export_index, page) in doc.pages.iter_mut().enumerate() {
            let source_index = app
                .doc()
                .pages
                .iter()
                .position(|source| source.id == page.id)
                .unwrap_or(export_index);
            let make_masters = |front| {
                masters
                    .iter()
                    .filter(move |layer| {
                        layer.scope.applies(source_index) && layer.in_front == front
                    })
                    .map(|layer| {
                        let mut layer = layer.clone();
                        for shape in &mut layer.shapes {
                            shape.show_page_number(source_index);
                        }
                        layer
                    })
            };
            let mut layers: Vec<_> = make_masters(false).collect();
            layers.append(&mut page.layers);
            layers.extend(make_masters(true));
            page.layers = layers;
        }
    }
    if raster && st.quality > 100 {
        return Err("raster quality exceeds 100".into());
    }
    let mut bases = Vec::new();
    for page in &mut doc.pages {
        if raster && !st.selection_only && !st.crop_to_page {
            let visible_shapes: Vec<_> = page
                .layers
                .iter()
                .filter(|layer| layer.visible)
                .flat_map(|layer| &layer.shapes)
                .filter_map(visible_export_shape)
                .collect();
            if let Some(bounds) = print_area(visible_shapes.iter()) {
                let shift = Affine::translate((-bounds.x0, -bounds.y0));
                for shape in page.layers.iter_mut().flat_map(|layer| &mut layer.shapes) {
                    shape.transform_by(shift);
                    carry_blend_ends(shape, shift);
                }
                page.size = Size::new(bounds.width().max(0.1), bounds.height().max(0.1));
            }
        }
        bases.push((page.size.width, page.size.height));
    }
    if raster {
        if !st.dpi.is_finite()
            || st.dpi <= 0.0
            || !st.scale.is_finite()
            || st.scale <= 0.0
            || !st.scale_y.is_finite()
            || st.scale_y <= 0.0
        {
            return Err("export DPI and scales must be positive finite values".into());
        }
        let ratio = if st.keep_ratio {
            1.0
        } else {
            st.scale_y / st.scale
        };
        if ratio != 1.0 {
            let transform = Affine::scale_non_uniform(1.0, ratio);
            for page in &mut doc.pages {
                page.size.height *= ratio;
                for shape in page.layers.iter_mut().flat_map(|layer| &mut layer.shapes) {
                    shape.transform_by(transform);
                    carry_blend_ends(shape, transform);
                }
            }
        }
    }
    let mut colors = crate::color_policy::prepare_screen_colors(&doc, &app.settings.color);
    let cmyk = export_cmyk_profile(app);
    let rgb = doc
        .rgb_profile
        .as_ref()
        .map(|profile| profile.data.to_vec())
        .or_else(|| crate::files::read(&app.settings.color.rgb_profile_path).ok());
    let gray = doc
        .gray_profile
        .as_ref()
        .map(|profile| profile.data.to_vec())
        .or_else(|| crate::files::read(&app.settings.color.gray_profile_path).ok());
    let intent = tracedraw_core::icc::Intent::from_name(&doc.metadata.rendering_intent)
        .or_else(|| tracedraw_core::icc::Intent::from_name(&app.settings.color.intent))
        .unwrap_or_default();
    let bpc = app.settings.color.black_point_compensation;
    if doc.rgb_profile.is_some() && !app.settings.color.builtin_engine {
        colors.icc = Some(tracedraw_core::color::IccEngine::new(
            rgb.as_deref()
                .and_then(|bytes| tracedraw_core::icc::Profile::parse(bytes).ok()),
            cmyk.as_deref()
                .and_then(|bytes| tracedraw_core::icc::Profile::parse(bytes).ok()),
            intent,
            bpc,
        ));
    }
    let proof = if st.use_proof_profile {
        resolve_proof_profile(app, &doc)?
    } else {
        None
    };
    let numbers = if exports_spreads(app, st) {
        (1..=doc.pages.len()).collect()
    } else {
        export_page_indexes(app, st)
            .into_iter()
            .map(|n| n + 1)
            .collect()
    };
    let title = if st.title.trim().is_empty() {
        app.document_title().trim_end_matches('*').to_string()
    } else {
        st.title.trim().to_string()
    };
    if *ext == "ai" && st.classic_ai() {
        doc.title = title.clone();
    }
    let profile_name = if st.use_proof_profile {
        app.settings.color.proof_profile.clone()
    } else {
        match st.color_mode_for(ext) {
            ExportColorMode::Cmyk => doc
                .cmyk_profile
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| app.settings.color.cmyk_profile.clone()),
            ExportColorMode::Gray => doc
                .gray_profile
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| app.settings.color.gray_profile.clone()),
            _ => doc
                .rgb_profile
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| app.settings.color.rgb_profile.clone()),
        }
    };
    if st.overprint_black && st.color_mode_for(ext) == ExportColorMode::Cmyk {
        for page in &mut doc.pages {
            for shape in page.layers.iter_mut().flat_map(|layer| &mut layer.shapes) {
                overprint_black(shape);
            }
        }
        for shape in doc.master.iter_mut().flat_map(|layer| &mut layer.shapes) {
            overprint_black(shape);
        }
        for symbol in &mut doc.symbols {
            overprint_black(&mut symbol.shape);
        }
    }
    Ok(ExportJob {
        doc,
        state: st.clone(),
        path: path.to_path_buf(),
        ext,
        numbers,
        title,
        cmyk,
        rgb,
        gray,
        proof,
        intent,
        bpc,
        pdf_profile: pdfx_profile(
            app,
            match st.pdf_standard {
                1 => tracedraw_io::pdf::PdfStandard::X1a,
                2 => tracedraw_io::pdf::PdfStandard::X3,
                3 => tracedraw_io::pdf::PdfStandard::X4,
                _ => tracedraw_io::pdf::PdfStandard::None,
            },
        ),
        screen: colors.icc.map(Arc::new),
        map_gray: colors.map_gray_to_black,
        profile_name,
        base_sizes: bases,
        rgb_memo: std::cell::RefCell::new(ExactRgbMemoSlot::default()),
    })
}
pub fn prepare_preview_job(app: &App, st: &ExportState, max_px: u32) -> Result<PreviewJob, String> {
    Ok(PreviewJob {
        job: prepare_job(app, Path::new("preview.jpg"), st)?,
        max_px: max_px.clamp(16, 2048),
        region: None,
    })
}
/// Native-resolution detail, with a bounded tile instead of stretching a fit preview.
pub fn prepare_preview_region_job(
    app: &App,
    st: &ExportState,
    origin_px: (u32, u32),
    size_px: (u32, u32),
) -> Result<PreviewJob, String> {
    Ok(PreviewJob {
        job: prepare_job(app, Path::new("preview.jpg"), st)?,
        max_px: 2048,
        region: Some((
            origin_px,
            (size_px.0.clamp(1, 2048), size_px.1.clamp(1, 2048)),
        )),
    })
}

impl ExportJob {
    pub fn run(
        self,
        progress: Option<ExportProgressCallback>,
        cancel: Arc<AtomicBool>,
    ) -> Result<String, String> {
        let scoped = self.screen.clone();
        let converter: tracedraw_core::color::engine::RgbConverter =
            std::rc::Rc::new(move |color| {
                scoped
                    .as_ref()
                    .and_then(|icc| icc.to_rgb_f32(color.process()))
                    .unwrap_or_else(|| color.to_rgb_f32_builtin())
            });
        tracedraw_core::color::engine::with_icc_scope(self.screen.clone(), self.map_gray, || {
            tracedraw_core::color::engine::with_rgb_converter(converter, || {
                self.run_inner(progress, cancel)
            })
        })
    }
    fn run_inner(
        self,
        progress: Option<ExportProgressCallback>,
        cancel: Arc<AtomicBool>,
    ) -> Result<String, String> {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        let streaming = matches!(self.ext, "jpg" | "png" | "tif" | "tiff");
        let total = if streaming {
            (0..self.doc.pages.len())
                .map(|k| {
                    self.raster_options(k)
                        .map(|opts| self.raster_work_units(&opts))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .sum()
        } else if self.state.one_file_holds_the_pages(self.ext) {
            1
        } else {
            self.doc.pages.len() as u64
        };
        let work = ExportWork {
            callback: progress.as_ref(),
            cancel: &cancel,
            done: std::cell::Cell::new(0),
            total,
        };
        work.report(ExportPhase::Render);
        let mut written = Vec::new();
        if self.state.one_file_holds_the_pages(self.ext) {
            work.report(ExportPhase::Encode);
            let bytes = document_file_prepared(
                &self.doc,
                self.ext,
                &self.state,
                self.pdf_profile.clone(),
                self.title.clone(),
            )?;
            work.done.set(1);
            work.report(ExportPhase::Write);
            write_export_bytes(&self.path, &bytes, &cancel)?;
            written.push(self.path.clone());
        } else {
            for k in 0..self.doc.pages.len() {
                if cancel.load(Ordering::Relaxed) {
                    return Err(CANCELLED.into());
                }
                let target = if self.doc.pages.len() > 1 {
                    numbered_path(
                        &self.path,
                        self.numbers.get(k).copied().unwrap_or(k + 1),
                        self.ext,
                    )
                } else {
                    self.path.clone()
                };
                if streaming {
                    self.write_raster_page(k, &target, &work)?;
                } else {
                    work.report(ExportPhase::Encode);
                    let bytes =
                        page_file(&self.doc, k, self.ext, &self.state, self.cmyk.as_deref())?;
                    work.done.set(work.done.get() + 1);
                    work.report(ExportPhase::Write);
                    write_export_bytes(&target, &bytes, &cancel)?;
                }
                written.push(target);
            }
        }
        if let Some(callback) = &progress {
            callback(ExportProgress {
                phase: ExportPhase::Complete,
                percent: 100,
                completed: total,
                total,
            });
        }
        Ok(match (written.first(), written.last()) {
            (Some(first), Some(last)) if written.len() > 1 => trf(
                "status.exported_pages",
                &[
                    ("count", &written.len().to_string()),
                    ("first", &first.display().to_string()),
                    ("last", &last.display().to_string()),
                ],
            ),
            _ => trf(
                "status.exported",
                &[("path", &self.path.display().to_string())],
            ),
        })
    }
}

fn write_export_bytes(path: &Path, bytes: &[u8], cancel: &AtomicBool) -> Result<(), String> {
    if crate::files::WEB || crate::files::is_document_path(path) {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        return crate::files::write(path, bytes).map_err(|error| error.to_string());
    }
    tracedraw_io::raster_export::atomic_file(path, cancel, |file| {
        use std::io::Write;
        for part in bytes.chunks(64 * 1024) {
            if cancel.load(Ordering::Relaxed) {
                return Err(tracedraw_io::Error::Format(CANCELLED.into()));
            }
            file.write_all(part)?;
        }
        Ok(())
    })
    .map_err(|error| {
        if cancel.load(Ordering::Relaxed) {
            CANCELLED.into()
        } else {
            error.to_string()
        }
    })
}

fn resolve_proof_profile(app: &App, doc: &Document) -> Result<Option<Vec<u8>>, String> {
    let name = app.settings.color.proof_profile.trim();
    if name.is_empty() || name.starts_with("Generic ") {
        return Ok(None);
    }
    for profile in [&doc.rgb_profile, &doc.cmyk_profile, &doc.gray_profile]
        .into_iter()
        .flatten()
    {
        if profile.name.eq_ignore_ascii_case(name) {
            return Ok(Some(profile.data.to_vec()));
        }
    }
    if let Ok(bytes) = crate::files::read(name) {
        return Ok(Some(bytes));
    }
    if let Some(profile) = crate::color_policy::system_profiles()
        .into_iter()
        .find(|profile| profile.name.eq_ignore_ascii_case(name))
    {
        return crate::files::read(profile.path)
            .map(Some)
            .map_err(|error| error.to_string());
    }
    Err(format!("proof profile is not available: {name}"))
}

struct ExportWork<'a> {
    callback: Option<&'a ExportProgressCallback>,
    cancel: &'a AtomicBool,
    done: std::cell::Cell<u64>,
    total: u64,
}
impl ExportWork<'_> {
    fn check(&self) -> Result<(), String> {
        if self.cancel.load(Ordering::Relaxed) {
            Err(CANCELLED.into())
        } else {
            Ok(())
        }
    }
    fn report(&self, phase: ExportPhase) {
        if let Some(callback) = self.callback {
            callback(ExportProgress {
                phase,
                percent: (self.done.get().saturating_mul(99) / self.total.max(1)).min(99) as u8,
                completed: self.done.get(),
                total: self.total,
            });
        }
    }
    fn rows(&self, phase: ExportPhase, width: u32) -> impl Fn(u32) + '_ {
        let previous = std::cell::Cell::new(0u32);
        move |rows: u32| {
            let old = previous.get();
            if rows > old {
                previous.set(rows);
                self.done.set(
                    self.done
                        .get()
                        .saturating_add(u64::from(rows - old) * u64::from(width)),
                );
                self.report(phase);
            }
        }
    }
}

impl ExportJob {
    fn raster_options(&self, k: usize) -> Result<tracedraw_render::RenderOptions, String> {
        let page = self.doc.pages.get(k).ok_or("no page")?;
        // Scaling changes the requested pixel extent, not the file's DPI tag.
        // Reject an oversized result; never clamp it to a smaller DPI.
        let zoom = (self.state.dpi * self.state.scale) / 25.4;
        let width =
            tracedraw_render::pixel_extent(page.size.width * zoom).ok_or("invalid output width")?;
        let height = tracedraw_render::pixel_extent(page.size.height * zoom)
            .ok_or("invalid output height")?;
        tracedraw_io::raster_export::checked_layout(width, height, 4)
            .map_err(|error| error.to_string())?;
        Ok(tracedraw_render::RenderOptions {
            width,
            height,
            view: tracedraw_render::ViewTransform {
                zoom,
                origin_x: 0.0,
                origin_y: f64::from(height),
            },
            anti_alias: self.state.anti_alias,
            simulate_overprints: self.state.simulate_overprints || self.state.overprint_black,
            ..Default::default()
        })
    }
    fn raster_work_units(&self, opts: &tracedraw_render::RenderOptions) -> u64 {
        let render = if self.state.color_mode_for(self.ext) == ExportColorMode::Cmyk {
            4
        } else {
            1
        };
        let blur = if self.ext == "jpg" && self.state.blur > 0.0 {
            2
        } else {
            0
        };
        u64::from(opts.width) * u64::from(opts.height) * (render + blur + 2)
    }
    fn render_cmyk_rows(
        &self,
        k: usize,
        opts: &tracedraw_render::RenderOptions,
        control: &tracedraw_io::raster_export::Control<'_>,
    ) -> Result<Vec<u8>, String> {
        let mut page = self.doc.pages.get(k).ok_or("no page")?.clone();
        for shape in page.layers.iter_mut().flat_map(|layer| &mut layer.shapes) {
            map_colors(shape, &spot_as_cmyk);
        }
        let mut one = self.doc.clone();
        for shape in one.master.iter_mut().flat_map(|layer| &mut layer.shapes) {
            map_colors(shape, &spot_as_cmyk);
        }
        for symbol in &mut one.symbols {
            map_colors(&mut symbol.shape, &spot_as_cmyk);
        }
        let master = one.master.clone();
        let symbols = one.symbols.clone();
        one.pages = vec![page.clone()];
        let mut out = tracedraw_io::raster_export::allocate(opts.width, opts.height, 4)
            .map_err(|error| error.to_string())?;
        for ink in 0..4 {
            one.pages[0] =
                plate_page_checked_with_cancel(&page, Plate::Process(ink), control.cancelled)?;
            let mut master_page = page.clone();
            master_page.layers = master.clone();
            one.master = plate_page_checked_with_cancel(
                &master_page,
                Plate::Process(ink),
                control.cancelled,
            )?
            .layers;
            one.symbols = symbols.clone();
            for symbol in &mut one.symbols {
                let mut symbol_page = page.clone();
                symbol_page.layers.truncate(1);
                if let Some(layer) = symbol_page.layers.first_mut() {
                    layer.shapes = vec![symbol.shape.clone()];
                    symbol.shape = plate_page_checked_with_cancel(
                        &symbol_page,
                        Plate::Process(ink),
                        control.cancelled,
                    )?
                    .layers[0]
                        .shapes
                        .remove(0);
                }
            }

            let band = tracedraw_render::export_band_rows(&one, page.id, opts);
            for y in (0..opts.height).step_by(band as usize) {
                if control.cancelled.load(Ordering::Relaxed) {
                    return Err(CANCELLED.into());
                }
                let end = y.saturating_add(band).min(opts.height);
                let part = tracedraw_render::render_page_rows(&one, page.id, opts, y, end)
                    .ok_or("CMYK band render failed")?;
                let start = y as usize * opts.width as usize * 4;
                for (target, pixel) in out
                    [start..start + (end - y) as usize * opts.width as usize * 4]
                    .chunks_exact_mut(4)
                    .zip(part.pixels())
                {
                    target[ink as usize] = pixel.alpha().saturating_sub(pixel.red());
                }
                (control.row_done)(ink as u32 * opts.height + end);
            }
        }
        Ok(out)
    }
    fn make_raster(
        &self,
        k: usize,
        opts: &tracedraw_render::RenderOptions,
        work: &ExportWork<'_>,
    ) -> Result<
        (
            Vec<u8>,
            tracedraw_io::raster_export::Samples,
            Option<Vec<u8>>,
        ),
        String,
    > {
        use tracedraw_io::raster_export::{self as output, Samples};
        work.check()?;
        let mode = self.state.color_mode_for(self.ext);
        let row_done = work.rows(ExportPhase::Render, opts.width);
        let control = output::Control {
            cancelled: work.cancel,
            row_done: &row_done,
        };
        let input = if mode == ExportColorMode::Cmyk {
            Samples::Cmyk
        } else if self.state.transparent && keeps_alpha(self.ext) {
            Samples::Rgba
        } else {
            Samples::Rgb
        };
        let mut data = if input == Samples::Cmyk {
            self.render_cmyk_rows(k, opts, &control)?
        } else {
            output::render_samples(
                &self.doc,
                self.doc.pages[k].id,
                opts,
                input,
                self.state.matte,
                &control,
            )
            .map_err(|error| error.to_string())?
        };
        drop(row_done);
        let (samples, icc) =
            self.convert_output(&mut data, opts.width, opts.height, input, work)?;
        if self.ext == "jpg" && self.state.blur > 0.0 {
            let row_done = work.rows(ExportPhase::Render, opts.width);
            blur_export_samples(
                &mut data,
                opts.width,
                opts.height,
                samples.channels(),
                self.state.blur,
                work.cancel,
                &row_done,
            )?;
        }
        Ok((data, samples, if self.state.embed_icc { icc } else { None }))
    }
    fn encode_raster_to<W: std::io::Write + std::io::Seek>(
        &self,
        out: W,
        opts: &tracedraw_render::RenderOptions,
        data: &[u8],
        samples: tracedraw_io::raster_export::Samples,
        icc: Option<&[u8]>,
        work: &ExportWork<'_>,
    ) -> Result<(), String> {
        use tracedraw_io::raster_export as output;
        let row_done = work.rows(ExportPhase::Encode, opts.width);
        let control = output::Control {
            cancelled: work.cancel,
            row_done: &row_done,
        };
        let result = match self.ext {
            "jpg" => output::jpeg_to(
                out,
                opts.width,
                opts.height,
                data,
                samples,
                &output::JpegOptions {
                    quality: self.state.quality,
                    subsampling: self.state.jpeg_subsampling,
                    progressive: self.state.jpeg_progressive,
                    optimize_huffman: self.state.jpeg_optimize_huffman,
                    dpi: self.state.dpi,
                },
                icc,
                &control,
            ),
            "png" => output::png_to(
                out,
                opts.width,
                opts.height,
                data,
                samples,
                self.state.dpi,
                icc,
                &control,
            ),
            "tif" | "tiff" => output::tiff_to(
                out,
                opts.width,
                opts.height,
                data,
                samples,
                self.state.dpi,
                icc,
                &control,
            ),
            _ => return Err("not a streaming raster format".into()),
        };
        result.map_err(|error| {
            if work.cancel.load(Ordering::Relaxed) {
                CANCELLED.into()
            } else {
                error.to_string()
            }
        })
    }
    fn write_raster_page(
        &self,
        k: usize,
        path: &Path,
        work: &ExportWork<'_>,
    ) -> Result<(), String> {
        let opts = self.raster_options(k)?;
        let (data, samples, icc) = self.make_raster(k, &opts, work)?;
        if crate::files::WEB || crate::files::is_document_path(path) {
            let mut out = std::io::Cursor::new(Vec::new());
            self.encode_raster_to(&mut out, &opts, &data, samples, icc.as_deref(), work)?;
            work.check()?;
            work.report(ExportPhase::Write);
            return crate::files::write(path, out.into_inner()).map_err(|error| error.to_string());
        }
        tracedraw_io::raster_export::atomic_file(path, work.cancel, |file| {
            self.encode_raster_to(file, &opts, &data, samples, icc.as_deref(), work)
                .map_err(tracedraw_io::Error::Format)?;
            work.report(ExportPhase::Write);
            Ok(())
        })
        .map_err(|error| {
            if work.cancel.load(Ordering::Relaxed) {
                CANCELLED.into()
            } else {
                error.to_string()
            }
        })
    }
    fn convert_output(
        &self,
        data: &mut Vec<u8>,
        width: u32,
        height: u32,
        input: tracedraw_io::raster_export::Samples,
        work: &ExportWork<'_>,
    ) -> Result<(tracedraw_io::raster_export::Samples, Option<Vec<u8>>), String> {
        self.convert_output_with(data, width, height, input, work, OutputConversion::Auto)
    }
    fn convert_output_with(
        &self,
        data: &mut Vec<u8>,
        width: u32,
        height: u32,
        input: tracedraw_io::raster_export::Samples,
        work: &ExportWork<'_>,
        policy: OutputConversion,
    ) -> Result<(tracedraw_io::raster_export::Samples, Option<Vec<u8>>), String> {
        use tracedraw_core::icc::{ColorSpace, Profile, Transform};
        use tracedraw_io::raster_export::{self as output, Samples};
        work.check()?;
        let expected = output::checked_layout(width, height, input.channels())
            .map_err(|error| error.to_string())?;
        if data.len() != expected {
            return Err("raster sample length does not match output dimensions".into());
        }
        let mode = self.state.color_mode_for(self.ext);
        let source_cmyk = self
            .cmyk
            .as_deref()
            .and_then(|bytes| Profile::parse(bytes).ok());
        let proof = self
            .proof
            .as_deref()
            .map(Profile::parse)
            .transpose()
            .map_err(|error| error.to_string())?;
        let proof_space = proof.as_ref().map(Profile::color_space);
        let srgb = Profile::srgb();
        let soft_proof = self.state.use_proof_profile
            && mode != ExportColorMode::Cmyk
            && proof_space != Some(ColorSpace::Rgb);
        let target_bytes = match mode {
            ExportColorMode::Cmyk if self.state.use_proof_profile => {
                if proof_space.is_some_and(|space| space != ColorSpace::Cmyk) {
                    return Err("CMYK output needs a CMYK proof profile".into());
                }
                self.proof.clone()
            }
            ExportColorMode::Cmyk => self.cmyk.clone(),
            ExportColorMode::Gray => self.gray.clone(),
            ExportColorMode::Rgb if soft_proof => Some(output::srgb_profile_bytes()),
            ExportColorMode::Rgb if self.state.use_proof_profile => self
                .proof
                .clone()
                .or_else(|| Some(output::srgb_profile_bytes())),
            ExportColorMode::Rgb => self
                .rgb
                .clone()
                .or_else(|| Some(output::srgb_profile_bytes())),
        };
        let target = target_bytes
            .as_deref()
            .map(Profile::parse)
            .transpose()
            .map_err(|error| error.to_string())?;
        let desired_space = match mode {
            ExportColorMode::Cmyk => ColorSpace::Cmyk,
            ExportColorMode::Gray => ColorSpace::Gray,
            ExportColorMode::Rgb => ColorSpace::Rgb,
        };
        if target
            .as_ref()
            .is_some_and(|profile| profile.color_space() != desired_space)
        {
            return Err("output ICC colour space does not match encoded samples".into());
        }
        let transform = match mode {
            ExportColorMode::Cmyk if self.state.use_proof_profile => source_cmyk
                .as_ref()
                .zip(target.as_ref())
                .and_then(|(source, target)| Transform::new(source, target, self.intent, self.bpc)),
            ExportColorMode::Rgb
                if !soft_proof && (self.rgb.is_some() || self.state.use_proof_profile) =>
            {
                target
                    .as_ref()
                    .and_then(|target| Transform::new(&srgb, target, self.intent, self.bpc))
            }
            ExportColorMode::Gray => target
                .as_ref()
                .and_then(|target| Transform::new(&srgb, target, self.intent, self.bpc)),
            _ => None,
        };
        let to_proof = proof
            .as_ref()
            .and_then(|profile| Transform::new(&srgb, profile, self.intent, self.bpc));
        let from_proof = proof
            .as_ref()
            .and_then(|profile| Transform::new(profile, &srgb, self.intent, self.bpc));
        let output = if mode == ExportColorMode::Gray {
            Samples::Gray
        } else {
            input
        };
        let input_channels = input.channels();
        let output_channels = output.channels();
        let row_done = work.rows(ExportPhase::Render, width);
        // An absent transform and absent proof mean the already prepared
        // samples are the result. No ICC resemblance is used as an identity.
        if policy.identity()
            && !self.state.use_proof_profile
            && transform.is_none()
            && ((mode == ExportColorMode::Rgb && matches!(input, Samples::Rgb | Samples::Rgba))
                || (mode == ExportColorMode::Cmyk && input == Samples::Cmyk))
        {
            work.check()?;
            row_done(height);
            work.check()?;
            return Ok((output, target_bytes));
        }
        let memo_budget = if matches!(mode, ExportColorMode::Rgb | ExportColorMode::Gray)
            && matches!(input, Samples::Rgb | Samples::Rgba)
        {
            policy.memo_budget(u64::from(width) * u64::from(height))
        } else {
            None
        };
        let mut memo_guard = memo_budget.map(|_| self.rgb_memo.borrow_mut());
        if let Some(slot) = memo_guard.as_mut() {
            if !slot.attempted {
                work.check()?;
                slot.attempted = true;
                slot.table = ExactRgbMemo::try_new(memo_budget.unwrap_or(0));
                work.check()?;
            }
        }
        let mut memo = memo_guard.as_mut().and_then(|slot| slot.table.as_mut());
        for y in 0..height {
            work.check()?;
            for x in 0..width {
                let pixel = y as usize * width as usize + x as usize;
                let from = pixel * input_channels;
                let original = [
                    data[from],
                    *data.get(from + 1).unwrap_or(&0),
                    *data.get(from + 2).unwrap_or(&0),
                    *data.get(from + 3).unwrap_or(&255),
                ];
                let rgb_key = [original[0], original[1], original[2]];
                if let Some(converted) = memo.as_deref().and_then(|table| table.get(rgb_key)) {
                    let to = pixel * output_channels;
                    let channels = output_channels.min(3);
                    data[to..to + channels].copy_from_slice(&converted[..channels]);
                    if output == Samples::Rgba {
                        data[to + 3] = original[3];
                    }
                    continue;
                }
                let mut rgb = [original[0], original[1], original[2]].map(|v| f32::from(v) / 255.0);
                if soft_proof {
                    if let (Some(to), Some(from)) = (&to_proof, &from_proof) {
                        let mut proof_values = [0.0; 4];
                        if !to.transform_into(&rgb, &mut proof_values)
                            || !from.transform_into(&proof_values, &mut rgb)
                        {
                            return Err("proof profile transform failed".into());
                        }
                    } else if proof.is_none() {
                        rgb = tracedraw_core::press::cmyk_to_srgb(
                            tracedraw_core::press::srgb_to_cmyk(rgb),
                        );
                    } else {
                        return Err("proof profile has no supported bidirectional transform".into());
                    }
                }
                let mut values = [0.0f32; 4];
                match mode {
                    ExportColorMode::Cmyk if self.state.use_proof_profile => {
                        if let Some(transform) = &transform {
                            if !transform.transform_into(
                                &original.map(|v| f32::from(v) / 255.0),
                                &mut values,
                            ) {
                                return Err("CMYK profile transform failed".into());
                            }
                        } else {
                            let source = original.map(|v| f32::from(v) / 255.0);
                            let rgb = self
                                .screen
                                .as_ref()
                                .and_then(|engine| {
                                    engine.to_rgb_f32(tracedraw_core::Color::Cmyk {
                                        c: source[0],
                                        m: source[1],
                                        y: source[2],
                                        k: source[3],
                                    })
                                })
                                .unwrap_or_else(|| tracedraw_core::press::cmyk_to_srgb(source));
                            values = to_proof
                                .as_ref()
                                .and_then(|transform| transform.rgb_to_cmyk(rgb))
                                .unwrap_or_else(|| tracedraw_core::press::srgb_to_cmyk(rgb));
                        }
                    }
                    ExportColorMode::Cmyk => values = original.map(|v| f32::from(v) / 255.0),
                    ExportColorMode::Gray => {
                        if let Some(transform) = &transform {
                            if !transform.transform_into(&rgb, &mut values) {
                                return Err("gray profile transform failed".into());
                            }
                        } else {
                            let gray = tracedraw_core::Color::rgb8(
                                (rgb[0] * 255.0).round() as u8,
                                (rgb[1] * 255.0).round() as u8,
                                (rgb[2] * 255.0).round() as u8,
                            )
                            .convert_to("Grayscale");
                            values[0] = match gray {
                                tracedraw_core::Color::Gray { v } => v,
                                _ => 0.0,
                            };
                        }
                    }
                    ExportColorMode::Rgb => {
                        if let Some(transform) = &transform {
                            if !transform.transform_into(&rgb, &mut values) {
                                return Err("RGB profile transform failed".into());
                            }
                        } else {
                            values[..3].copy_from_slice(&rgb);
                        }
                    }
                };
                let to = pixel * output_channels;
                for channel in 0..output_channels.min(values.len()) {
                    data[to + channel] = (values[channel].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                if output == Samples::Rgba {
                    data[to + 3] = original[3];
                }
                if let Some(table) = memo.as_deref_mut() {
                    table.put(
                        rgb_key,
                        [
                            data[to],
                            *data
                                .get(to + 1)
                                .filter(|_| output_channels > 1)
                                .unwrap_or(&0),
                            *data
                                .get(to + 2)
                                .filter(|_| output_channels > 2)
                                .unwrap_or(&0),
                        ],
                    );
                }
            }
            row_done(y + 1);
        }
        data.truncate(width as usize * height as usize * output_channels);
        Ok((output, target_bytes))
    }
}

fn blur_export_samples(
    data: &mut [u8],
    width: u32,
    height: u32,
    channels: usize,
    sigma: f32,
    cancel: &AtomicBool,
    row_done: &dyn Fn(u32),
) -> Result<(), String> {
    if !sigma.is_finite() || !(0.0..=2.0).contains(&sigma) {
        return Err("JPEG blur must be between 0 and 2 pixels".into());
    }
    if sigma <= 0.0 {
        return Ok(());
    }
    let radius = (sigma * 3.0).ceil() as i32;
    let mut kernel: Vec<f32> = (-radius..=radius)
        .map(|x| (-(x * x) as f32 / (2.0 * sigma * sigma)).exp())
        .collect();
    let sum: f32 = kernel.iter().sum();
    for value in &mut kernel {
        *value /= sum;
    }
    let row_bytes = width as usize * channels;
    let mut row = Vec::new();
    row.try_reserve_exact(row_bytes)
        .map_err(|_| "not enough memory for blur row")?;
    row.resize(row_bytes, 0);
    for y in 0..height as usize {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        row.copy_from_slice(&data[y * row_bytes..(y + 1) * row_bytes]);
        for x in 0..width as usize {
            for c in 0..channels {
                let value: f32 = kernel
                    .iter()
                    .enumerate()
                    .map(|(k, weight)| {
                        f32::from(
                            row[(x as i32 + k as i32 - radius).clamp(0, width as i32 - 1) as usize
                                * channels
                                + c],
                        ) * weight
                    })
                    .sum();
                data[y * row_bytes + x * channels + c] = value.round().clamp(0.0, 255.0) as u8;
            }
        }
        row_done(y as u32 + 1);
    }
    let mut rows: std::collections::VecDeque<(usize, Vec<u8>)> = std::collections::VecDeque::new();
    let mut loaded = 0usize;
    for y in 0..height as usize {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        let end = (y + radius as usize).min(height as usize - 1);
        while loaded <= end {
            let mut row = Vec::new();
            row.try_reserve_exact(row_bytes)
                .map_err(|_| "not enough memory for blur strips")?;
            row.extend_from_slice(&data[loaded * row_bytes..(loaded + 1) * row_bytes]);
            rows.push_back((loaded, row));
            loaded += 1;
        }
        let inputs: Vec<(&[u8], f32)> = kernel
            .iter()
            .enumerate()
            .map(|(k, weight)| {
                let at = (y as i32 + k as i32 - radius).clamp(0, height as i32 - 1) as usize;
                (
                    rows.iter()
                        .find(|(index, _)| *index == at)
                        .expect("retained blur row")
                        .1
                        .as_slice(),
                    *weight,
                )
            })
            .collect();
        for byte in 0..row_bytes {
            let value: f32 = inputs
                .iter()
                .map(|(row, weight)| f32::from(row[byte]) * weight)
                .sum();
            data[y * row_bytes + byte] = value.round().clamp(0.0, 255.0) as u8;
        }
        let next_start = (y + 1).saturating_sub(radius as usize);
        while rows.front().is_some_and(|(index, _)| *index < next_start) {
            rows.pop_front();
        }
        row_done(height + y as u32 + 1);
    }
    Ok(())
}

#[cfg(test)]
fn samples_for_preview(
    data: &[u8],
    width: u32,
    height: u32,
    samples: tracedraw_io::raster_export::Samples,
    icc: Option<&[u8]>,
    intent: tracedraw_core::icc::Intent,
    bpc: bool,
) -> Result<tiny_skia::Pixmap, String> {
    samples_for_preview_checked(data, width, height, samples, icc, intent, bpc, &|| Ok(()))
}
fn samples_for_preview_with_cancel(
    data: &[u8],
    width: u32,
    height: u32,
    samples: tracedraw_io::raster_export::Samples,
    icc: Option<&[u8]>,
    intent: tracedraw_core::icc::Intent,
    bpc: bool,
    cancel: &AtomicBool,
) -> Result<tiny_skia::Pixmap, String> {
    samples_for_preview_checked(data, width, height, samples, icc, intent, bpc, &|| {
        check_export_cancel(Some(cancel))
    })
}
fn samples_for_preview_checked(
    data: &[u8],
    width: u32,
    height: u32,
    samples: tracedraw_io::raster_export::Samples,
    icc: Option<&[u8]>,
    intent: tracedraw_core::icc::Intent,
    bpc: bool,
    check: &dyn Fn() -> Result<(), String>,
) -> Result<tiny_skia::Pixmap, String> {
    use tracedraw_io::raster_export::Samples;
    check()?;
    let profile = icc
        .map(tracedraw_core::icc::Profile::parse)
        .transpose()
        .map_err(|error| error.to_string())?;
    let display = tracedraw_core::icc::Profile::srgb();
    let transform = profile
        .as_ref()
        .and_then(|profile| tracedraw_core::icc::Transform::new(profile, &display, intent, bpc));
    let mut image = tiny_skia::Pixmap::new(width, height).ok_or("preview allocation failed")?;
    check()?;
    for (targets, sources) in image
        .pixels_mut()
        .chunks_mut(width as usize)
        .zip(data.chunks_exact(width as usize * samples.channels()))
    {
        check()?;
        for (target, source) in targets
            .iter_mut()
            .zip(sources.chunks_exact(samples.channels()))
        {
            let mut rgb = [0.0f32; 3];
            if let Some(transform) = &transform {
                let mut input = [0.0f32; 4];
                for (to, from) in input.iter_mut().zip(source) {
                    *to = f32::from(*from) / 255.0;
                }
                if !transform.transform_into(&input[..samples.channels().min(4)], &mut rgb) {
                    return Err("preview ICC conversion failed".into());
                }
            } else {
                rgb = match samples {
                    Samples::Gray => [f32::from(source[0]) / 255.0; 3],
                    Samples::Cmyk => tracedraw_core::press::cmyk_to_srgb(
                        [source[0], source[1], source[2], source[3]].map(|v| f32::from(v) / 255.0),
                    ),
                    _ => [source[0], source[1], source[2]].map(|v| f32::from(v) / 255.0),
                };
            }
            let [r, g, b] = rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
            let alpha = if samples == Samples::Rgba {
                source[3]
            } else {
                255
            };
            *target = tiny_skia::ColorU8::from_rgba(r, g, b, alpha).premultiply();
        }
    }
    check()?;
    Ok(image)
}

impl PreviewJob {
    pub fn run(self, cancel: Arc<AtomicBool>) -> Result<ExportPreviewResult, String> {
        let screen = self.job.screen.clone();
        let scoped = screen.clone();
        let converter: tracedraw_core::color::engine::RgbConverter =
            std::rc::Rc::new(move |color| {
                scoped
                    .as_ref()
                    .and_then(|engine| engine.to_rgb_f32(color.process()))
                    .unwrap_or_else(|| color.to_rgb_f32_builtin())
            });
        tracedraw_core::color::engine::with_icc_scope(screen, self.job.map_gray, || {
            tracedraw_core::color::engine::with_rgb_converter(converter, || self.run_inner(cancel))
        })
    }
    fn run_inner(self, cancel: Arc<AtomicBool>) -> Result<ExportPreviewResult, String> {
        let mut opts = self.job.raster_options(0)?;
        let output_size = (opts.width, opts.height);
        if let Some((origin, size)) = self.region {
            let x = origin.0.min(opts.width - 1);
            let y = origin.1.min(opts.height - 1);
            opts.width = size.0.min(opts.width - x);
            opts.height = size.1.min(opts.height - y);
            opts.view.origin_x -= f64::from(x);
            opts.view.origin_y -= f64::from(y);
        } else {
            let ratio = f64::from(self.max_px) / f64::from(opts.width.max(opts.height));
            let ratio = ratio.min(1.0);
            opts.width = (f64::from(opts.width) * ratio).ceil().max(1.0) as u32;
            opts.height = (f64::from(opts.height) * ratio).ceil().max(1.0) as u32;
            opts.view.zoom *= ratio;
            opts.view.origin_y = f64::from(opts.height);
        }
        let work = ExportWork {
            callback: None,
            cancel: &cancel,
            done: std::cell::Cell::new(0),
            total: self.job.raster_work_units(&opts),
        };
        let (data, samples, icc) = self.job.make_raster(0, &opts, &work)?;
        let original = samples_for_preview_with_cancel(
            &data,
            opts.width,
            opts.height,
            samples,
            icc.as_deref(),
            self.job.intent,
            self.job.bpc,
            &cancel,
        )?;
        let mut encoded = std::io::Cursor::new(Vec::new());
        let encoded = if matches!(self.job.ext, "jpg" | "png" | "tif" | "tiff") {
            self.job.encode_raster_to(
                &mut encoded,
                &opts,
                &data,
                samples,
                icc.as_deref(),
                &work,
            )?;
            encoded.into_inner()
        } else {
            let mut bytes = Vec::new();
            original
                .encode_png()
                .map(|encoded| bytes = encoded)
                .map_err(|error| error.to_string())?;
            bytes
        };
        work.check()?;
        let image = if samples == tracedraw_io::raster_export::Samples::Cmyk {
            let (w, h, inks) =
                tracedraw_io::raster_export::decode_cmyk_preview(&encoded, self.job.ext)
                    .map_err(|error| error.to_string())?;
            samples_for_preview_with_cancel(
                &inks,
                w,
                h,
                samples,
                icc.as_deref(),
                self.job.intent,
                self.job.bpc,
                &cancel,
            )?
        } else {
            let decoded = image::load_from_memory(&encoded).map_err(|error| error.to_string())?;
            match samples {
                tracedraw_io::raster_export::Samples::Gray => samples_for_preview_with_cancel(
                    decoded.to_luma8().as_raw(),
                    opts.width,
                    opts.height,
                    samples,
                    icc.as_deref(),
                    self.job.intent,
                    self.job.bpc,
                    &cancel,
                )?,
                _ => {
                    let rgba = decoded.into_rgba8();
                    samples_for_preview_with_cancel(
                        rgba.as_raw(),
                        opts.width,
                        opts.height,
                        tracedraw_io::raster_export::Samples::Rgba,
                        icc.as_deref(),
                        self.job.intent,
                        self.job.bpc,
                        &cancel,
                    )?
                }
            }
        };
        let full_pixels = u64::from(output_size.0) * u64::from(output_size.1);
        let small_pixels = u64::from(opts.width) * u64::from(opts.height);
        let estimated = if self.region.is_none() {
            estimate_preview_bytes(&encoded, self.job.ext, full_pixels, small_pixels)
        } else {
            None
        };
        let profile_name = icc
            .as_deref()
            .and_then(|bytes| tracedraw_core::icc::Profile::parse(bytes).ok())
            .and_then(|profile| profile.description().map(str::to_owned))
            .unwrap_or_else(|| {
                if icc.is_some() {
                    self.job.profile_name.clone()
                } else {
                    String::new()
                }
            });
        Ok(ExportPreviewResult {
            image,
            original,
            estimated_bytes: estimated,
            output_size,
            profile_name,
            profile_bytes: icc.as_ref().map_or(0, Vec::len),
            base_size_mm: self.job.base_sizes[0],
        })
    }
}

/// Count the actual APP2 ICC containers written by our JPEG encoder, including
/// marker, length, signature and sequence fields. Entropy data is not scanned.
fn jpeg_fixed_icc_bytes(jpeg: &[u8]) -> Option<usize> {
    if jpeg.get(..2)? != b"\xff\xd8" {
        return None;
    }
    let mut at = 2usize;
    let mut fixed = 0usize;
    loop {
        let start = at;
        if *jpeg.get(at)? != 0xff {
            return None;
        }
        while jpeg.get(at) == Some(&0xff) {
            at = at.checked_add(1)?;
        }
        let marker = *jpeg.get(at)?;
        at = at.checked_add(1)?;
        match marker {
            0xda | 0xd9 => return Some(fixed), // SOS/EOI: all generated ICC precedes the scan
            0x01 => continue,                  // standalone TEM
            0x00 | 0xd0..=0xd8 => return None,
            _ => {}
        }
        let size_bytes = jpeg.get(at..at.checked_add(2)?)?;
        let size = usize::from(u16::from_be_bytes([size_bytes[0], size_bytes[1]]));
        if size < 2 {
            return None;
        }
        let end = at.checked_add(size)?;
        let payload = jpeg.get(at.checked_add(2)?..end)?;
        if marker == 0xe2 && payload.starts_with(b"ICC_PROFILE\0") {
            if payload.len() < 14 {
                return None;
            }
            fixed = fixed.checked_add(end.checked_sub(start)?)?;
        }
        at = end;
    }
}

/// A size estimate, not an encoding of the full image. Keep embedded ICC
/// containers fixed; retain the historical area estimate for all other bytes.
fn estimate_preview_bytes(
    encoded: &[u8],
    ext: &str,
    full_pixels: u64,
    small_pixels: u64,
) -> Option<usize> {
    let fixed = if matches!(ext, "jpg" | "jpeg") {
        jpeg_fixed_icc_bytes(encoded)?
    } else {
        0
    };
    let scalable = u64::try_from(encoded.len().checked_sub(fixed)?).ok()?;
    let fixed = u64::try_from(fixed).ok()?;
    usize::try_from(
        (scalable.saturating_mul(full_pixels) / small_pixels.max(1)).saturating_add(fixed),
    )
    .ok()
}

/// Document limited to the selection or the current page, for exports.
pub(crate) fn export_document(app: &App, st: &ExportState) -> (Document, Rect) {
    let doc = app.engine.document();
    if st.selection_only {
        let bounds = app.selection_bounds().unwrap_or(app.page_rect());
        // A selected page is still in its document's colour/font/style context.
        // Cloning shares immutable picture/font/profile bytes; no full raster
        // copies are made while preparing the worker's owned document.
        let mut tmp = doc.clone();
        let mut page = doc
            .page(app.page)
            .map(Clone::clone)
            .unwrap_or_else(|_| doc.pages[0].clone());
        page.size = Size::new(bounds.width().max(0.1), bounds.height().max(0.1));
        for layer in &mut page.layers {
            layer.shapes.clear();
        }
        tmp.pages = vec![page];
        tmp.master.clear();
        let l = tmp.pages[0].layers[0].id;
        for s in app.selected_shapes() {
            let mut c = s.clone();
            let shift = Affine::translate((-bounds.x0, -bounds.y0));
            c.transform_by(shift);
            carry_blend_ends(&mut c, shift);
            if let Ok(layer) = tmp.layer_mut(l) {
                layer.shapes.push(c);
            }
        }
        (tmp, bounds)
    } else {
        // Facing pages (#1972): each spread as one page, or each page
        // with what its facing page holds across the spine (and in the
        // bleed beside it).
        if exports_spreads(app, st) {
            return (
                doc.spreads_document(&export_page_indexes(app, st)),
                app.page_rect(),
            );
        }
        // The pages chosen, in the order chosen (#1382).
        let mut tmp = doc.with_spread_overhangs(st.bleed_mm.max(doc.metadata.bleed));
        let mut all: Vec<_> = std::mem::take(&mut tmp.pages)
            .into_iter()
            .map(Some)
            .collect();
        tmp.pages = export_page_indexes(app, st)
            .into_iter()
            .filter_map(|i| all.get_mut(i).and_then(Option::take))
            .collect();
        (tmp, app.page_rect())
    }
}

/// `st` exports the spreads of facing pages as pages (#1972).
pub(crate) fn exports_spreads(app: &App, st: &ExportState) -> bool {
    st.spreads && !st.selection_only && app.engine.document().facing().is_some()
}

/// The indexes of the pages `st` exports from the drawing (#1382).
pub(crate) fn export_page_indexes(app: &App, st: &ExportState) -> Vec<usize> {
    let doc = app.engine.document();
    let current = doc.pages.iter().position(|p| p.id == app.page).unwrap_or(0);
    st.page_indexes(doc.pages.len(), current)
}

/// Where page `number` of a export to `path` that writes one file per
/// page goes: "name-3.png" for page 3 (#1382).
pub fn numbered_path(path: &Path, number: usize, ext: &str) -> std::path::PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map_or(ext.to_string(), |e| e.to_string_lossy().to_string());
    path.with_file_name(format!("{stem}-{number}.{ext}"))
}

pub fn export(app: &mut App, path: &Path, st: &ExportState) -> Result<String, String> {
    prepare_job(app, path, st)?.run(None, Arc::new(AtomicBool::new(false)))
}

/// The file of a format that holds every page (PDF, AI, PostScript, PRN,
/// HTML) for the prepared document and captured profile.
fn document_file_prepared(
    doc: &Document,
    ext: &str,
    st: &ExportState,
    profile: Option<Vec<u8>>,
    title: String,
) -> Result<Vec<u8>, String> {
    Ok(match ext {
        "pdf" | "ai" => {
            use tracedraw_io::pdf::{PdfOptions, PdfStandard};
            let standard = match (ext, st.pdf_standard) {
                ("pdf", 1) => PdfStandard::X1a,
                ("pdf", 2) => PdfStandard::X3,
                ("pdf", 3) => PdfStandard::X4,
                _ => PdfStandard::None,
            };
            let opts = PdfOptions {
                standard,
                output_profile: profile,
                output_condition: st.output_condition.clone(),
                bleed_mm: if ext == "pdf" { st.bleed_mm } else { 0.0 },
                title: Some(title),
                keywords: (!st.tags.trim().is_empty()).then(|| st.tags.trim().to_string()),
                flatten_dpi: st.dpi.max(72.0),
                text: tracedraw_io::pdf::PdfText {
                    as_curves: ext == "pdf" && st.text_as_curves,
                    ..Default::default()
                },
                ..Default::default()
            };
            tracedraw_io::pdf::document_to_pdf_with(doc, &opts)
        }
        // Every exported page; a print file is the same PostScript.
        "ps" | "prn" => tracedraw_io::eps::document_to_ps(doc).into_bytes(),
        "html" => tracedraw_io::html::document_to_html(doc).into_bytes(),
        other => return Err(format!("{other}: not a format of several pages")),
    })
}

/// The file of page `k` of `doc` in a format that holds one page; `icc`
/// is the profile a CMYK bitmap carries.
fn page_file(
    doc: &Document,
    k: usize,
    ext: &str,
    st: &ExportState,
    icc: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    Ok(match ext {
        "svg" => tracedraw_io::svg::page_to_svg(doc, k).into_bytes(),
        "eps" => tracedraw_io::eps::page_to_eps(doc, k).into_bytes(),
        // The classic AI file: one artboard, so one page per file (#1728).
        "ai" if st.classic_ai() => tracedraw_io::ai::page_to_ai8(doc, k).into_bytes(),
        "dxf" => tracedraw_io::dxf::page_to_dxf(doc, k).into_bytes(),
        "emf" => tracedraw_io::emf::page_to_emf(doc, k),
        "wmf" => tracedraw_io::emf::page_to_wmf(doc, k),
        "plt" => tracedraw_io::plt::page_to_plt(doc, k).into_bytes(),
        "psd" => tracedraw_io::psd::page_to_psd(doc, k, st.render_dpi()).ok_or("render failed")?,
        _ => {
            let pid = doc.pages.get(k).ok_or("no page")?.id;
            match st.color_mode_for(ext) {
                // The inks of the page, as its separations print them
                // (#1763).
                ExportColorMode::Cmyk => {
                    let (w, h, ink) =
                        cmyk_pixels(doc, pid, st.render_dpi()).ok_or("render failed")?;
                    let bytes = if ext == "jpg" {
                        tracedraw_io::cmyk_bitmap::cmyk_jpeg(w, h, &ink, st.quality, st.dpi, icc)
                    } else {
                        tracedraw_io::cmyk_bitmap::cmyk_tiff(w, h, &ink, st.dpi, icc)
                    };
                    return bytes.ok_or_else(|| "CMYK encoding failed".to_string());
                }
                ExportColorMode::Gray => {
                    let pm = tracedraw_render::render_page_image(doc, pid, st.render_dpi())
                        .ok_or("render failed")?;
                    let bytes =
                        encode_gray(&on_white(&pm).ok_or("render failed")?, ext, st.quality)?;
                    return Ok(record_dpi(bytes, ext, st.dpi));
                }
                ExportColorMode::Rgb => {}
            }
            let mut pm = tracedraw_render::render_page_image(doc, pid, st.render_dpi())
                .ok_or("render failed")?;
            if st.transparent && matches!(ext, "png" | "webp" | "gif") {
                // Re-render on a transparent background.
                let p = doc.page(pid).map_err(|e| e.to_string())?;
                let zoom = st.render_dpi() / 25.4;
                let w = (p.size.width * zoom).ceil() as u32;
                let h = (p.size.height * zoom).ceil() as u32;
                pm = tracedraw_render::render_page(
                    doc,
                    pid,
                    &tracedraw_render::RenderOptions {
                        width: w,
                        height: h,
                        view: tracedraw_render::ViewTransform {
                            zoom,
                            origin_x: 0.0,
                            origin_y: h as f64,
                        },
                        preview: None,
                        wireframe: false,
                        ..tracedraw_render::RenderOptions::default()
                    },
                )
                .ok_or("render failed")?;
            }
            let bytes = encode_raster_with(&pm, ext, st.quality, st.webp_lossless)?;
            // The file says its resolution, so it opens at the size
            // chosen in the dialog (#617).
            record_dpi(bytes, ext, st.dpi)
        }
    })
}

/// The output intent's profile for a PDF/X file: the CMYK profile picked
/// under Colour Management, when there is one and it can be read.
pub(crate) fn pdfx_profile(app: &App, standard: tracedraw_io::pdf::PdfStandard) -> Option<Vec<u8>> {
    if standard == tracedraw_io::pdf::PdfStandard::None {
        return None;
    }
    let p = app.settings.color.cmyk_profile_path.trim().to_string();
    (!p.is_empty())
        .then(|| crate::files::read(&p).ok())
        .flatten()
}

/// The CMYK profile a CMYK bitmap export carries (#1763): the one the
/// colour management converts its colours with, the drawing's own while
/// it is used, else the one picked under Colour Management; none with
/// the built-in conversions.
pub(crate) fn export_cmyk_profile(app: &App) -> Option<Vec<u8>> {
    let c = &app.settings.color;
    if c.builtin_engine {
        return None;
    }
    if c.open_cmyk == 0 && crate::app::carried_cmyk_profile(app.doc()).is_some() {
        return app.doc().cmyk_profile.as_ref().map(|p| p.data.to_vec());
    }
    app.settings_cmyk_profile().map(|p| p.data.to_vec())
}

/// `pm` drawn over white, as a page is printed.
fn on_white(pm: &tiny_skia::Pixmap) -> Option<tiny_skia::Pixmap> {
    let mut white = tiny_skia::Pixmap::new(pm.width(), pm.height())?;
    white.fill(tiny_skia::Color::WHITE);
    white.draw_pixmap(
        0,
        0,
        pm.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    Some(white)
}

/// Page `pid` of `doc` at `dpi` as ink coverages (#1763): cyan, magenta,
/// yellow and black of each pixel, 255 = full ink, with its size in
/// pixels. Each process ink is the page's separation plate for it
/// ([`plate_page`]) drawn on white: CMYK colours keep their exact
/// values, other colours go through the drawing's colour management as
/// on the plates, pictures give the inks they keep (#1659) or their
/// pixels converted, and spot colours print as their process mix.
pub(crate) fn cmyk_pixels(
    doc: &Document,
    pid: tracedraw_core::PageId,
    dpi: f64,
) -> Option<(u32, u32, Vec<u8>)> {
    let mut page = doc.page(pid).ok()?.clone();
    for layer in page.layers.iter_mut() {
        for s in layer.shapes.iter_mut() {
            map_colors(s, &spot_as_cmyk);
        }
    }
    let mut one = doc.clone();
    one.pages.retain(|p| p.id == pid);
    let mut out: Option<(u32, u32, Vec<u8>)> = None;
    for ink in 0..4 {
        *one.pages.first_mut()? = plate_page(&page, Plate::Process(ink));
        let pm = on_white(&tracedraw_render::render_page_image(&one, pid, dpi)?)?;
        let (w, h) = (pm.width(), pm.height());
        let (ow, oh, buf) =
            out.get_or_insert_with(|| (w, h, vec![0u8; w as usize * h as usize * 4]));
        if (*ow, *oh) != (w, h) {
            return None;
        }
        for (px, p) in buf.chunks_exact_mut(4).zip(pm.pixels()) {
            // The plate is grey: black is full ink.
            px[ink] = 255 - p.red();
        }
    }
    out
}

/// The grey of each pixel of an opaque `pm`, through the colour
/// management's Grayscale conversion, each distinct colour once.
fn gray_pixels(pm: &tiny_skia::Pixmap) -> Vec<u8> {
    let mut known: std::collections::HashMap<[u8; 3], u8> = std::collections::HashMap::new();
    pm.pixels()
        .iter()
        .map(|p| {
            let rgb = [p.red(), p.green(), p.blue()];
            *known.entry(rgb).or_insert_with(|| {
                match tracedraw_core::Color::rgb8(rgb[0], rgb[1], rgb[2]).convert_to("Grayscale") {
                    tracedraw_core::Color::Gray { v } => (v.clamp(0.0, 1.0) * 255.0).round() as u8,
                    _ => 0,
                }
            })
        })
        .collect()
}

/// An opaque rendered page as a one-channel JPEG (at `quality`) or TIFF.
fn encode_gray(pm: &tiny_skia::Pixmap, ext: &str, quality: u8) -> Result<Vec<u8>, String> {
    let img = image::GrayImage::from_raw(pm.width(), pm.height(), gray_pixels(pm))
        .ok_or("image buffer")?;
    let mut out = std::io::Cursor::new(Vec::new());
    if ext == "jpg" {
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100))
            .encode_image(&img)
            .map_err(|e| e.to_string())?;
    } else {
        img.write_to(&mut out, image::ImageFormat::Tiff)
            .map_err(|e| e.to_string())?;
    }
    Ok(out.into_inner())
}

/// The ink coverages of an opaque `pm` through the colour management's
/// CMYK conversion, each distinct colour once: for the estimate of a
/// CMYK file's size from the preview.
fn ink_pixels(pm: &tiny_skia::Pixmap) -> Vec<u8> {
    let mut known: std::collections::HashMap<[u8; 3], [u8; 4]> = std::collections::HashMap::new();
    pm.pixels()
        .iter()
        .flat_map(|p| {
            let rgb = [p.red(), p.green(), p.blue()];
            *known.entry(rgb).or_insert_with(|| {
                match tracedraw_core::Color::rgb8(rgb[0], rgb[1], rgb[2]).convert_to("CMYK") {
                    tracedraw_core::Color::Cmyk { c, m, y, k } => {
                        [c, m, y, k].map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
                    }
                    _ => [0; 4],
                }
            })
        })
        .collect()
}

/// `bytes`, a PNG, JPEG or BMP file, saying it was made at `dpi` dots per
/// inch: a PNG gets a `pHYs` chunk after its header, a JPEG's JFIF header
/// and a BMP's header get the density (#617). Other formats, and files not
/// laid out as the encoder writes them, come back unchanged.
pub fn record_dpi(bytes: Vec<u8>, ext: &str, dpi: f64) -> Vec<u8> {
    tracedraw_io::raster::record_dpi(bytes, ext, dpi)
}

/// True when the export format `ext` keeps transparency.
pub fn keeps_alpha(ext: &str) -> bool {
    matches!(ext, "png" | "webp" | "gif")
}

/// What the export will look like: the page (or the selection) of `st`
/// rendered to fit `max_px` on its longer side, on a transparent
/// background when the format keeps one, else on white.
pub fn preview(app: &App, st: &ExportState, max_px: u32) -> Option<tiny_skia::Pixmap> {
    let (_, ext) = EXPORT_FORMATS[st.format.min(EXPORT_FORMATS.len() - 1)];
    let (doc, _) = export_document(app, st);
    let page = doc.pages.first()?;
    let longest = page.size.width.max(page.size.height).max(0.1);
    let zoom = max_px.max(16) as f64 / longest;
    let w = (page.size.width * zoom).ceil().max(1.0) as u32;
    let h = (page.size.height * zoom).ceil().max(1.0) as u32;
    let mut pm = tracedraw_render::render_page(
        &doc,
        page.id,
        &tracedraw_render::RenderOptions {
            width: w,
            height: h,
            view: tracedraw_render::ViewTransform {
                zoom,
                origin_x: 0.0,
                origin_y: h as f64,
            },
            preview: None,
            wireframe: false,
            ..tracedraw_render::RenderOptions::default()
        },
    )?;
    if !(st.transparent && keeps_alpha(ext)) {
        let mut white = tiny_skia::Pixmap::new(w, h)?;
        white.fill(tiny_skia::Color::WHITE);
        white.draw_pixmap(
            0,
            0,
            pm.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            tiny_skia::Transform::identity(),
            None,
        );
        pm = white;
    }
    // A CMYK file shows as it will print, a grey one in grey (#1763).
    match st.color_mode_for(ext) {
        ExportColorMode::Rgb => {}
        ExportColorMode::Cmyk => {
            let mut known: std::collections::HashMap<[u8; 3], [u8; 3]> =
                std::collections::HashMap::new();
            for p in pm.pixels_mut() {
                let rgb = [p.red(), p.green(), p.blue()];
                let [r, g, b] = *known.entry(rgb).or_insert_with(|| {
                    tracedraw_core::color::proof_rgb(rgb.map(|v| f32::from(v) / 255.0))
                        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
                });
                *p = tiny_skia::PremultipliedColorU8::from_rgba(r, g, b, 255)?;
            }
        }
        ExportColorMode::Gray => {
            let grey = gray_pixels(&pm);
            for (p, v) in pm.pixels_mut().iter_mut().zip(grey) {
                *p = tiny_skia::PremultipliedColorU8::from_rgba(v, v, v, 255)?;
            }
        }
    }
    Some(pm)
}

/// About how many bytes the export of `st` takes: a vector format is
/// written in memory; a bitmap format is estimated from `preview` (the
/// output of [`preview`]) encoded the same way and scaled to the export's
/// pixel count.
pub fn estimated_size(app: &App, st: &ExportState, preview: &tiny_skia::Pixmap) -> Option<usize> {
    let (_, ext) = EXPORT_FORMATS[st.format.min(EXPORT_FORMATS.len() - 1)];
    let (doc, bounds) = export_document(app, st);
    // One file per page: the first page's size times the pages (#1382).
    let pages = doc.pages.len().max(1);
    match ext {
        "svg" => Some(tracedraw_io::svg::page_to_svg(&doc, 0).len() * pages),
        "eps" => Some(tracedraw_io::eps::page_to_eps(&doc, 0).len() * pages),
        "ps" | "prn" => Some(tracedraw_io::eps::document_to_ps(&doc).len()),
        "ai" if st.classic_ai() => Some(tracedraw_io::ai::page_to_ai8(&doc, 0).len() * pages),
        "pdf" | "ai" => Some(tracedraw_io::pdf::document_to_pdf(&doc).len()),
        "png" | "jpg" | "bmp" | "tif" | "webp" | "gif" | "psd" => {
            let enc = if ext == "psd" { "tif" } else { ext };
            let (w, h) = (preview.width(), preview.height());
            let bytes = match st.color_mode_for(ext) {
                ExportColorMode::Rgb => {
                    encode_raster_with(preview, enc, st.quality, st.webp_lossless).ok()?
                }
                ExportColorMode::Gray => encode_gray(preview, ext, st.quality).ok()?,
                ExportColorMode::Cmyk if ext == "jpg" => tracedraw_io::cmyk_bitmap::cmyk_jpeg(
                    w,
                    h,
                    &ink_pixels(preview),
                    st.quality,
                    st.dpi,
                    None,
                )?,
                ExportColorMode::Cmyk => {
                    tracedraw_io::cmyk_bitmap::cmyk_tiff(w, h, &ink_pixels(preview), st.dpi, None)?
                }
            }
            .len() as f64;
            let dpi = st.render_dpi();
            let full = (bounds.width() / 25.4 * dpi) * (bounds.height() / 25.4 * dpi);
            let shown = (preview.width() as f64 * preview.height() as f64).max(1.0);
            Some((bytes * full / shown * pages as f64) as usize)
        }
        _ => None,
    }
}

/// A byte count for people: "820 KB", "1.4 MB".
pub fn human_size(bytes: usize) -> String {
    let b = bytes as f64;
    if b < 1000.0 * 1024.0 {
        format!("{:.0} KB", (b / 1024.0).max(1.0))
    } else {
        format!("{:.1} MB", b / (1024.0 * 1024.0))
    }
}

/// Test convenience: the same shared encoder as the CLI and the desktop.
#[cfg(test)]
fn encode_raster(pm: &tiny_skia::Pixmap, ext: &str, quality: u8) -> Result<Vec<u8>, String> {
    encode_raster_with(pm, ext, quality, true)
}

/// Encode straight RGBA through the shared IO codecs. In lossless WebP,
/// quality never rounds RGB samples; the caller stamps its export DPI.
fn encode_raster_with(
    pm: &tiny_skia::Pixmap,
    ext: &str,
    quality: u8,
    lossless: bool,
) -> Result<Vec<u8>, String> {
    let (w, h) = (pm.width(), pm.height());
    let mut rgba = image::RgbaImage::new(w, h);
    for (out, p) in rgba.pixels_mut().zip(pm.pixels()) {
        let p = p.demultiply();
        *out = image::Rgba([p.red(), p.green(), p.blue(), p.alpha()]);
    }
    tracedraw_io::raster::encode_rgba(
        &rgba,
        ext,
        &tracedraw_io::raster::RasterOptions {
            quality,
            webp_lossless: lossless,
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
}

/// Print: build a PDF with the print options and send it to the chosen
/// system printer, or open it in the system viewer, which owns the
/// printer dialog.
pub fn print(app: &mut App, st: &PrintState) -> Result<String, String> {
    let mut doc = crate::print_this_sheet::sheets_to_print(app, st);
    if st.separations {
        doc = separations_doc_without(&doc, &st.plates_off);
    }
    let stem = app.document_title().trim_end_matches('*').to_string();
    // Files get their copies inside; the printer (lp) or the system
    // viewer handles the count.
    if st.destination != 0 && st.copies > 1 {
        repeat_copies(&mut doc, st.copies.min(999) as usize, st.collate);
    }
    match st.destination {
        1 | 2 => {
            let ps = st.destination == 2;
            let (ext, filter) = if ps {
                ("ps", "PostScript")
            } else {
                ("pdf", "PDF")
            };
            let bytes = if ps {
                postscript_level(tracedraw_io::eps::document_to_ps(&doc), st.ps_level).into_bytes()
            } else {
                tracedraw_io::pdf::document_to_pdf(&doc)
            };
            crate::files::Dialog::new()
                .add_filter(filter, &[ext])
                .set_file_name(format!("{stem}.{ext}"))
                .save_file(app, move |app, path| {
                    app.status = match crate::files::write(&path, bytes) {
                        Ok(()) => trf("status.exported", &[("path", &path.display().to_string())]),
                        Err(e) => format!("{}: {e}", tr("dialog.print_failed")),
                    };
                });
            Ok(String::new())
        }
        _ => {
            let name = format!("{stem}.pdf");
            if let (Some(printer), false) = (&st.printer, crate::files::WEB) {
                // The sheets are drawn and spooled off the interface's
                // thread: a slow or stuck printer leaves the window usable
                // (#810).
                let (printer, copies, collate) = (printer.clone(), st.copies, st.collate);
                // Print as bitmap: the sheets are already pictures of
                // that resolution, sent as they are.
                let bitmap_dpi = st.as_bitmap.then(|| st.bitmap_dpi.clamp(36.0, 1200.0));
                let msg = trf("status.printing_to", &[("name", &printer)]);
                app.start_print_job(move || {
                    print_on(&printer, &name, &doc, copies, collate, &stem, bitmap_dpi)
                });
                return Ok(msg);
            }
            let pdf = tracedraw_io::pdf::document_to_pdf(&doc);
            if crate::files::WEB {
                // In a browser the PDF is downloaded; its viewer prints it.
                let path = std::path::PathBuf::from("/download").join(&name);
                crate::files::write(&path, pdf).map_err(|e| e.to_string())?;
                return Ok(trf("status.print_sent", &[("path", &name)]));
            }
            print_through_viewer(&name, pdf)
        }
    }
}

/// The print PDF sent to the system printer `printer` (`lp`), with the
/// copies the dialog asked for.
#[cfg(not(target_os = "windows"))]
fn print_on(
    printer: &str,
    name: &str,
    doc: &Document,
    copies: u32,
    collate: bool,
    title: &str,
    _bitmap_dpi: Option<f64>,
) -> Result<String, String> {
    let pdf = tracedraw_io::pdf::document_to_pdf(doc);
    let dir = std::env::temp_dir().join("tracedraw-print");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(name);
    crate::files::write(&path, pdf).map_err(|e| e.to_string())?;
    crate::printers::print_pdf(printer, &path, copies, collate, title)?;
    Ok(trf("status.print_sent_printer", &[("name", printer)]))
}

/// The sheets drawn at the printer's own resolution and sent to the
/// Windows printer `printer`, local or on the network (#614), with the
/// copies the dialog asked for. Each sheet goes as strips of pixels with
/// its blank parts left out, so fine lines and small text come out as
/// sharp as from a PDF sent through `lp` and blank paper costs nothing in
/// the job (#1404); `bitmap_dpi` (Print as bitmap) sends that resolution
/// instead.
#[cfg(target_os = "windows")]
fn print_on(
    printer: &str,
    _name: &str,
    doc: &Document,
    copies: u32,
    collate: bool,
    title: &str,
    bitmap_dpi: Option<f64>,
) -> Result<String, String> {
    let dpi = bitmap_dpi
        .unwrap_or_else(|| windows_print_dpi(crate::printers::printer_resolution_windows(printer)));
    let sheets: Vec<crate::printers::Sheet> = doc
        .pages
        .iter()
        .map(|p| crate::printers::Sheet {
            width: p.size.width,
            height: p.size.height,
            tiles: sheet_tiles(doc, p.id, dpi, PRINT_TILE_PIXELS).unwrap_or_default(),
        })
        .collect();
    crate::printers::print_sheets_windows(printer, &sheets, dpi, copies, collate, title)?;
    Ok(trf("status.print_sent_printer", &[("name", printer)]))
}

/// The resolution sheets are drawn at for a Windows printer that says
/// `reported` dots per inch: its own, kept between 300 and 1200 (a
/// higher one would only make the job larger), 600 when it does not say.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn windows_print_dpi(reported: Option<f64>) -> f64 {
    match reported {
        Some(d) if d.is_finite() && d > 0.0 => d.clamp(300.0, 1200.0),
        _ => 600.0,
    }
}

/// The most pixels drawn at once for a printed sheet (64 MB of RGBA).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
const PRINT_TILE_PIXELS: u64 = 1 << 24;

/// A sheet drawn on white at `dpi` as strips of at most `max_pixels`
/// pixels each, every strip cut to the part of it that is not blank
/// (white) and left out when all of it is: the PNGs with where they go on
/// the sheet. The strips put together give the same pixels as the whole
/// sheet drawn at once, with no sheet-sized picture in memory.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn sheet_tiles(
    doc: &Document,
    id: tracedraw_core::PageId,
    dpi: f64,
    max_pixels: u64,
) -> Option<Vec<crate::printers::Tile>> {
    let page = doc.page(id).ok()?;
    let zoom = dpi / 25.4;
    let w = ((page.size.width * zoom).ceil() as u32).max(1);
    let h = ((page.size.height * zoom).ceil() as u32).max(1);
    // Whole multiples of 16 rows, as the coarse grid of procedural fills.
    let rows = ((max_pixels / w as u64) / 16 * 16).clamp(16, (h as u64).div_ceil(16) * 16) as u32;
    let opts = tracedraw_render::RenderOptions {
        width: w,
        height: h,
        view: tracedraw_render::ViewTransform {
            zoom,
            origin_x: 0.0,
            origin_y: h as f64,
        },
        ..Default::default()
    };
    let mut tiles = Vec::new();
    let mut y0 = 0u32;
    while y0 < h {
        let band = rows.min(h - y0);
        let objects = tracedraw_render::render_page_rows(doc, id, &opts, y0, y0 + band)?;
        let mut strip = tiny_skia::Pixmap::new(w, band)?;
        strip.fill(tiny_skia::Color::WHITE);
        strip.draw_pixmap(
            0,
            0,
            objects.as_ref(),
            &Default::default(),
            tiny_skia::Transform::identity(),
            None,
        );
        if let Some((x, y, cw, ch)) = inked_area(&strip) {
            let part =
                strip.clone_rect(tiny_skia::IntRect::from_xywh(x as i32, y as i32, cw, ch)?)?;
            tiles.push(crate::printers::Tile {
                png: part.encode_png().ok()?,
                x: x as f64 / zoom,
                y: (y0 + y) as f64 / zoom,
                width: cw as f64 / zoom,
                height: ch as f64 / zoom,
            });
        }
        y0 += band;
    }
    Some(tiles)
}

/// The smallest box (x, y, width, height in pixels) holding every pixel of
/// `pm` that is not white; `None` when all of them are.
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
fn inked_area(pm: &tiny_skia::Pixmap) -> Option<(u32, u32, u32, u32)> {
    let w = pm.width() as usize;
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for (y, row) in pm.data().chunks_exact(w * 4).enumerate() {
        let mut first = None;
        let mut last = 0;
        for (x, px) in row.chunks_exact(4).enumerate() {
            if px != [255, 255, 255, 255] {
                first.get_or_insert(x);
                last = x;
            }
        }
        if let Some(f) = first {
            x0 = x0.min(f);
            x1 = x1.max(last);
            y0 = y0.min(y);
            y1 = y;
        }
    }
    (x0 != usize::MAX).then(|| {
        (
            x0 as u32,
            y0 as u32,
            (x1 - x0 + 1) as u32,
            (y1 - y0 + 1) as u32,
        )
    })
}

/// Prints sent to the system viewer in this session.
#[cfg(not(target_arch = "wasm32"))]
static VIEWER_PRINTS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Where the `count`th print of `name` to the viewer goes in `dir`: the
/// first takes the name itself, later ones "name (2).pdf" and on, so a
/// viewer still showing (and holding) an earlier print opens the new one
/// in a page of its own (#809). Prints of the name before the last one
/// are removed once no viewer holds them.
#[cfg(not(target_arch = "wasm32"))]
fn viewer_print_path(dir: &Path, name: &str, count: u32) -> std::path::PathBuf {
    let p = Path::new(name);
    let stem = p.file_stem().map_or(name.into(), |s| s.to_string_lossy());
    let ext = p.extension().map_or("pdf".into(), |s| s.to_string_lossy());
    let nth = |n: u32| match n {
        0 | 1 => dir.join(name),
        n => dir.join(format!("{stem} ({n}).{ext}")),
    };
    for n in 1..count.saturating_sub(1) {
        let _ = std::fs::remove_file(nth(n));
    }
    // A file of that name left by another session and still held.
    let mut n = count.max(1);
    while n < count.max(1) + 100 && nth(n).exists() && std::fs::remove_file(nth(n)).is_err() {
        n += 1;
    }
    nth(n)
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn print_through_viewer(name: &str, pdf: Vec<u8>) -> Result<String, String> {
    let dir = std::env::temp_dir().join("tracedraw-print");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let count = VIEWER_PRINTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    let path = viewer_print_path(&dir, name, count);
    crate::files::write(&path, pdf).map_err(|e| e.to_string())?;
    open_with_system(&path)?;
    Ok(trf(
        "status.print_sent",
        &[("path", &path.display().to_string())],
    ))
}

/// On Android the print PDF goes to the system's print service, whose
/// dialog picks the printer, the copies and the paper (#1398). The file
/// stays in the app's temporary folder: the service reads it later.
#[cfg(target_os = "android")]
fn print_through_viewer(name: &str, pdf: Vec<u8>) -> Result<String, String> {
    let dir = std::env::temp_dir().join("tracedraw-print");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(shared_file_name(name));
    std::fs::write(&path, pdf).map_err(|e| e.to_string())?;
    let job = name.trim_end_matches(".pdf");
    if crate::android::outside::print_pdf(&path, job) {
        Ok(trf("status.print_sent_android", &[("name", job)]))
    } else {
        Err(tr("status.print_service_failed"))
    }
}

#[cfg(target_arch = "wasm32")]
fn print_through_viewer(name: &str, _pdf: Vec<u8>) -> Result<String, String> {
    Ok(trf("status.print_sent", &[("path", name)]))
}

/// The sheets repeated for `copies` copies: whole sets one after the other
/// (collated) or each sheet `copies` times.
fn repeat_copies(doc: &mut Document, copies: usize, collate: bool) {
    let original = std::mem::take(&mut doc.pages);
    let mut ids = doc.ids().clone();
    let mut push = |p: &tracedraw_core::document::Page, out: &mut Vec<_>| {
        let mut q = p.clone();
        q.id = ids.page();
        out.push(q);
    };
    let mut out = Vec::with_capacity(original.len() * copies);
    if collate {
        for _ in 0..copies {
            for p in &original {
                push(p, &mut out);
            }
        }
    } else {
        for p in &original {
            for _ in 0..copies {
                push(p, &mut out);
            }
        }
    }
    doc.set_ids(ids);
    doc.pages = out;
}

/// The PostScript file's language level in its header comments.
fn postscript_level(ps: String, level: u8) -> String {
    let level = level.clamp(2, 3);
    match ps.find('\n') {
        Some(i) => format!("{}\n%%LanguageLevel: {level}{}", &ps[..i], &ps[i..]),
        None => ps,
    }
}

/// The sheets the printer gets for the Print dialog's settings: the pages
/// imposed with their marks, mirrored and inverted when asked. Print and
/// Print Preview both start here, so the preview shows what prints.
/// The objects `page` shows (every one with `all`) scaled down and moved
/// as one to fit inside it, centred, when they run off it; objects wholly
/// off the page stay out of it.
fn fit_contents_to_page(page: &mut tracedraw_core::document::Page, all: bool) {
    const EPS: f64 = 1e-6;
    let area = page.rect();
    let shows = |b: &Rect| {
        all || (b.x0 <= area.x1 && b.x1 >= area.x0 && b.y0 <= area.y1 && b.y1 >= area.y0)
    };
    let Some(b) = page
        .layers
        .iter()
        .flat_map(|l| &l.shapes)
        .map(|s| s.bounds())
        .filter(|b| b.width().is_finite() && b.height().is_finite() && shows(b))
        .reduce(|a, b| a.union(b))
    else {
        return;
    };
    let inside = b.x0 >= area.x0 - EPS
        && b.y0 >= area.y0 - EPS
        && b.x1 <= area.x1 + EPS
        && b.y1 <= area.y1 + EPS;
    if inside {
        return;
    }
    let k = (area.width() / b.width().max(EPS))
        .min(area.height() / b.height().max(EPS))
        .min(1.0);
    let t = Affine::translate(area.center().to_vec2())
        * Affine::scale(k)
        * Affine::translate(-b.center().to_vec2());
    for layer in page.layers.iter_mut() {
        for s in layer.shapes.iter_mut() {
            if shows(&s.bounds()) {
                s.transform_by(t);
            }
        }
    }
}

/// The area `shapes` print on: their outlines (half the outline's width
/// beyond the path) and live effects included, at least a sliver each
/// way (a line has no height). `None` for no shapes.
pub fn print_area<'a>(shapes: impl Iterator<Item = &'a tracedraw_core::Shape>) -> Option<Rect> {
    const MIN: f64 = 0.01;
    let area = shapes
        .map(|s| {
            let half = s.stroke.as_ref().map_or(0.0, |st| st.width.max(0.0) / 2.0);
            s.visual_bounds().inflate(half, half)
        })
        .filter(|b| b.x0.is_finite() && b.y0.is_finite() && b.x1.is_finite() && b.y1.is_finite())
        .reduce(|a, b| a.union(b))?;
    let c = area.center();
    let (w, h) = (area.width().max(MIN), area.height().max(MIN));
    Some(Rect::new(
        c.x - w / 2.0,
        c.y - h / 2.0,
        c.x + w / 2.0,
        c.y + h / 2.0,
    ))
}

/// The copies of their end objects that the blends of `s` (and of the
/// objects inside it) keep, in page space, moved by `t` along with `s`:
/// a page-space transformation of the start object alone, with no end
/// object to bring the copy up to date from (the end was not exported
/// with the start), would otherwise leave the steps running to where the
/// end used to be (#1541).
fn carry_blend_ends(s: &mut tracedraw_core::Shape, t: Affine) {
    for e in s.effects.iter_mut() {
        if let tracedraw_core::live::Effect::Blend { end, .. } = e {
            end.transform_by(t);
        }
    }
    match &mut s.kind {
        ShapeKind::Group { children } => children.iter_mut().for_each(|c| carry_blend_ends(c, t)),
        ShapeKind::ClipFrame { frame, contents } => {
            carry_blend_ends(frame, t);
            contents.iter_mut().for_each(|c| carry_blend_ends(c, t));
        }
        _ => {}
    }
}

/// The copies of their end objects that the blends of `doc` keep brought
/// up to date from the end objects themselves, as the editor does after
/// every change (`App::refresh_blends`). The sheets are made from a copy
/// of the drawing whose objects are moved, shrunk and recoloured outside
/// the editor (Fit to page, the selection as the page, a negative, text
/// in black, spot inks as CMYK), and a blend evaluated from a stale copy
/// ran its steps to where the end object used to be, in the colour it
/// used to have (#1541).
fn refresh_blend_ends(doc: &mut Document) {
    use tracedraw_core::live::Effect;
    fn starts(s: &tracedraw_core::Shape, out: &mut Vec<(ShapeId, ShapeId)>) {
        for e in &s.effects {
            if let Effect::Blend {
                end_id: Some(end), ..
            } = e
            {
                out.push((s.id, *end));
            }
        }
        match &s.kind {
            ShapeKind::Group { children } => children.iter().for_each(|c| starts(c, out)),
            ShapeKind::ClipFrame { frame, contents } => {
                starts(frame, out);
                contents.iter().for_each(|c| starts(c, out));
            }
            _ => {}
        }
    }
    let mut pairs = Vec::new();
    for layer in doc.all_layers() {
        for s in &layer.shapes {
            starts(s, &mut pairs);
        }
    }
    for (start, end) in pairs {
        // An end object that is not printed (not selected, say) leaves the
        // copy as it is, as in the editor.
        let Some(mut copy) = doc.shape_on_page(end) else {
            continue;
        };
        copy.effects.clear();
        let Some(s) = doc.find_shape_mut(start) else {
            continue;
        };
        for e in s.effects.iter_mut() {
            if let Effect::Blend {
                end: kept,
                end_id: Some(id),
                ..
            } = e
            {
                if *id == end {
                    **kept = copy.clone();
                }
            }
        }
    }
}

/// The area the objects left on `page` and in the `master` layers cover
/// (what prints of them: visible objects of visible, printable layers,
/// their outlines and live effects included) made the page: the page
/// takes its size and the objects move with it, so they print as they
/// are placed among themselves. Nothing changes when nothing prints.
fn selection_as_page(
    page: &mut tracedraw_core::document::Page,
    master: &mut [tracedraw_core::Layer],
) {
    let printed = |l: &tracedraw_core::Layer| l.visible && l.printable;
    let shapes = page
        .layers
        .iter()
        .chain(master.iter())
        .filter(|l| printed(l))
        .flat_map(|l| &l.shapes)
        .filter(|s| s.visible);
    let Some(area) = print_area(shapes) else {
        return;
    };
    let (w, h) = (area.width(), area.height());
    let to_page = Affine::translate((-area.x0, -area.y0));
    for layer in page.layers.iter_mut().chain(master.iter_mut()) {
        for s in layer.shapes.iter_mut() {
            s.transform_by(to_page);
        }
    }
    page.size = Size::new(w, h);
}

/// The indexes of the drawing's pages the Print dialog's settings print,
/// in printing order: the whole drawing, the current page, the pages
/// listed or the selection's page, odd or even ones only when asked
/// (`PrintState::page_indexes`). `print_sheets` imposes these pages, and
/// the Layout sketch with the booklet summary under it counts the same
/// ones, so the counts match what prints (#788).
pub fn printed_pages(app: &App, st: &PrintState) -> Vec<usize> {
    let doc = app.engine.document();
    let current = doc.pages.iter().position(|p| p.id == app.page).unwrap_or(0);
    st.page_indexes(doc.pages.len(), current)
}

pub fn print_sheets(app: &App, st: &PrintState) -> Document {
    let src = app.engine.document();
    // Facing pages: each page printed with what its facing page holds
    // across the spine and in the bleed beside it (#1972); not for the
    // selection alone, nor when Fit to page shrinks what runs off.
    let mut doc = if (st.range == 3 && !app.selection.is_empty()) || st.fit_to_page {
        src.clone()
    } else {
        src.with_spread_overhangs(src.metadata.bleed)
    };
    let keep = printed_pages(app, st);
    let mut i = 0;
    doc.pages.retain(|_| {
        i += 1;
        keep.contains(&(i - 1))
    });
    // Selection: only the selected objects of the current page (none of
    // the master layers' others, no page background), and the area they
    // cover is the page printed: the layout, the sheet the size of the
    // page, Keep size and the marks work on it (#1001).
    if st.range == 3 && !app.selection.is_empty() {
        let picked = |s: &tracedraw_core::Shape| app.selection.contains(&s.id);
        for layer in doc.master.iter_mut() {
            layer.shapes.retain(picked);
        }
        for page in doc.pages.iter_mut() {
            page.background = None;
            for layer in page.layers.iter_mut() {
                layer.shapes.retain(picked);
            }
            selection_as_page(page, &mut doc.master);
        }
    }
    // Fit to page: objects running off the page (the selection, when
    // that is printed) shrink to fit inside it, centred, rather than
    // being cut at its edge (#534).
    if st.fit_to_page {
        for page in doc.pages.iter_mut() {
            fit_contents_to_page(page, st.range == 3);
        }
    }
    for page in doc.pages.iter_mut() {
        for layer in page.layers.iter_mut() {
            for s in layer.shapes.iter_mut() {
                if st.spot_to_cmyk {
                    map_colors(s, &spot_as_cmyk);
                }
                if st.text_black {
                    text_in_black(s);
                }
                if st.overprint_black {
                    overprint_black(s);
                }
            }
        }
    }
    // The blends' steps follow the objects as printed (#1541).
    refresh_blend_ends(&mut doc);
    let options = print_layout(app, st, &doc);
    let mut doc = crate::imposition::impose(&doc, &options);
    for page in doc.pages.iter_mut() {
        let w = page.size.width;
        for layer in page.layers.iter_mut() {
            for s in layer.shapes.iter_mut() {
                // Emulsion down: the whole sheet, marks too, mirrored.
                if st.mirror {
                    s.transform = Affine::translate((w, 0.0))
                        * Affine::new([-1.0, 0.0, 0.0, 1.0, 0.0, 0.0])
                        * s.transform;
                }
                if st.invert {
                    invert_colors(s);
                }
            }
        }
    }
    // A negative turns the paper black too, so that what was black ink
    // (now white) shows as the clear part of the film.
    if st.invert {
        let mut ids = doc.ids().clone();
        for page in doc.pages.iter_mut() {
            let size = page.size;
            let mut ground = tracedraw_core::Shape::new(
                ids.shape(),
                ShapeKind::Rect {
                    rect: tracedraw_core::geometry::Rect::new(0.0, 0.0, size.width, size.height),
                    radius: 0.0,
                    corners: None,
                },
            );
            ground.fill = tracedraw_core::Fill::Solid(tracedraw_core::Color::BLACK);
            ground.stroke = None;
            if page.layers.is_empty() {
                page.layers
                    .push(tracedraw_core::Layer::new(ids.layer(), "Negative"));
            }
            if let Some(layer) = page.layers.first_mut() {
                layer.shapes.insert(0, ground);
            }
        }
        doc.set_ids(ids);
    }
    // Grayscale composite output (#1657): every colour and picture of
    // the sheets, marks included, as its grey.
    if st.grayscale && !st.separations {
        grayscale_doc(&mut doc);
    }
    // Reverse order (#1394, #1571): the last sheet first, so a stack
    // printed face up, or the backs of printed sheets, come out in order.
    if st.reverse_order {
        doc.pages.reverse();
    }
    if st.as_bitmap {
        sheets_as_bitmaps(&mut doc, st.bitmap_dpi.clamp(36.0, 1200.0));
    }
    doc
}

/// The grey a colour prints as in grayscale output: its luminance
/// (`0.299 R + 0.587 G + 0.114 B`, as Bitmaps > Mode > Grayscale);
/// the registration colour stays, it prints on every plate.
fn gray_of(c: tracedraw_core::Color) -> tracedraw_core::Color {
    use tracedraw_core::Color;
    match c {
        Color::Gray { .. } | Color::Registration => c,
        _ => {
            let [r, g, b] = c.to_rgb8();
            let v = (0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b)) / 255.0;
            Color::Gray {
                v: v.clamp(0.0, 1.0),
            }
        }
    }
}

/// Grayscale output (#1657): every colour the sheets hold (fills,
/// outlines, text, gradients, meshes, patterns, effects, shadows and
/// lenses) turned to its grey, and every picture to its grey pixels (a
/// CMYK picture's own inks are left out, so it prints from the grey
/// pixels). The colours are found wherever the document keeps one, by
/// walking the objects' serialized form, so no kind of object is missed.
pub fn grayscale_doc(doc: &mut Document) {
    fn walk(v: &mut serde_json::Value) {
        use serde_json::Value;
        match v {
            Value::Array(a) => a.iter_mut().for_each(walk),
            Value::Object(m) => {
                if m.get("model").is_some_and(Value::is_string) {
                    if let Ok(c) =
                        serde_json::from_value::<tracedraw_core::Color>(Value::Object(m.clone()))
                    {
                        // A colour can share its object with the tag of
                        // what holds it (a solid fill is `{"fill":
                        // "solid", "model": ...}`): only its own keys go.
                        if let (Ok(Value::Object(was)), Ok(Value::Object(gray))) =
                            (serde_json::to_value(c), serde_json::to_value(gray_of(c)))
                        {
                            for k in was.keys() {
                                m.remove(k);
                            }
                            m.extend(gray);
                        }
                        return;
                    }
                }
                m.remove("picture_inks");
                m.remove("cmyk_bitmap");
                if let Some(Value::String(b64)) = m.get_mut("png") {
                    use base64::Engine;
                    let engine = base64::engine::general_purpose::STANDARD;
                    let gray = engine
                        .decode(b64.as_bytes())
                        .ok()
                        .and_then(|png| crate::bitmap_fx::decode(&png))
                        .and_then(|img| crate::bitmap_fx::encode_gray(&img));
                    if let Some(gray) = gray {
                        *b64 = engine.encode(gray);
                        // The picture's effects stay as printed, not
                        // worked out again from the colour original.
                        m.remove("fx");
                    }
                }
                m.values_mut().for_each(walk);
            }
            _ => {}
        }
    }
    fn gray<T: serde::Serialize + serde::de::DeserializeOwned>(x: &mut T) {
        let Ok(mut v) = serde_json::to_value(&*x) else {
            return;
        };
        walk(&mut v);
        match serde_json::from_value(v) {
            Ok(back) => *x = back,
            Err(e) => log::warn!("grayscale print: {e}"),
        }
    }
    for page in doc.pages.iter_mut() {
        gray(page);
    }
    gray(&mut doc.master);
}

/// A spot ink's colour as its CMYK mix at the tint.
fn spot_as_cmyk(c: tracedraw_core::Color) -> tracedraw_core::Color {
    match c {
        tracedraw_core::Color::Spot {
            tint, c, m, y, k, ..
        } => tracedraw_core::Color::Cmyk {
            c: c * tint,
            m: m * tint,
            y: y * tint,
            k: k * tint,
        },
        other => other,
    }
}

/// Composite > Print text in black: every text, in groups too, filled and
/// outlined in black.
fn text_in_black(s: &mut tracedraw_core::Shape) {
    use tracedraw_core::{Color, Fill};
    match &mut s.kind {
        ShapeKind::Text { spans, .. } => {
            if !matches!(s.fill, Fill::None) {
                s.fill = Fill::Solid(Color::BLACK);
            }
            if let Some(st) = &mut s.stroke {
                st.color = Color::BLACK;
            }
            for sp in spans.iter_mut() {
                if sp.fill.is_some() {
                    sp.fill = Some(Fill::Solid(Color::BLACK));
                }
            }
        }
        ShapeKind::Group { children } => children.iter_mut().for_each(text_in_black),
        ShapeKind::ClipFrame { contents, .. } => contents.iter_mut().for_each(text_in_black),
        _ => {}
    }
}

/// Whether `c` is black ink only (100 % black, no other ink).
fn is_pure_black(c: &tracedraw_core::Color) -> bool {
    match c.convert_to("CMYK") {
        tracedraw_core::Color::Cmyk { c, m, y, k } => {
            k >= 0.999 && c <= 0.001 && m <= 0.001 && y <= 0.001
        }
        _ => false,
    }
}

/// Composite > Always overprint black: black fills and outlines overprint.
fn overprint_black(s: &mut tracedraw_core::Shape) {
    use tracedraw_core::Fill;
    if let Fill::Solid(c) = &s.fill {
        if is_pure_black(c) {
            s.overprint_fill = true;
        }
    }
    if s.stroke.as_ref().is_some_and(|st| is_pure_black(&st.color)) {
        s.overprint_outline = true;
    }
    match &mut s.kind {
        ShapeKind::Group { children } => children.iter_mut().for_each(overprint_black),
        ShapeKind::ClipFrame { frame, contents } => {
            overprint_black(frame);
            contents.iter_mut().for_each(overprint_black);
        }
        _ => {}
    }
}

/// A sheet drawn on white at `dpi`: its PNG and size in pixels.
fn sheet_pixels(
    doc: &Document,
    id: tracedraw_core::PageId,
    dpi: f64,
) -> Option<(Vec<u8>, u32, u32)> {
    let pm = tracedraw_render::render_page_image(doc, id, dpi)?;
    let mut white = tiny_skia::Pixmap::new(pm.width(), pm.height())?;
    white.fill(tiny_skia::Color::WHITE);
    white.draw_pixmap(
        0,
        0,
        pm.as_ref(),
        &Default::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    let png = white.encode_png().ok()?;
    Some((png, white.width(), white.height()))
}

/// General > Print as bitmap: each sheet becomes one picture of itself.
fn sheets_as_bitmaps(doc: &mut Document, dpi: f64) {
    let pages: Vec<_> = doc.pages.iter().map(|p| (p.id, p.size)).collect();
    for (id, size) in pages {
        let Some((png, w, h)) = sheet_pixels(doc, id, dpi) else {
            continue;
        };
        let shape_id = doc.ids_mut().shape();
        let mut shape = tracedraw_core::Shape::new(
            shape_id,
            ShapeKind::Bitmap {
                rect: tracedraw_core::geometry::Rect::new(0.0, 0.0, size.width, size.height),
                width_px: w,
                height_px: h,
                png: png.into(),
                fx: None,
            },
        );
        shape.fill = tracedraw_core::Fill::None;
        shape.stroke = None;
        if let Ok(page) = doc.page_mut(id) {
            if let Some(first) = page.layers.first_mut() {
                first.shapes = vec![shape];
            }
            page.layers.truncate(1);
        }
    }
}

/// A separation plate: a process ink (0 cyan to 3 black) or a spot ink by
/// its id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plate {
    Process(usize),
    Spot(u32),
}

/// The plates of a drawing: the four process inks, then its spot inks,
/// each with its name.
pub fn plates(doc: &Document) -> Vec<(Plate, String)> {
    let mut plates: Vec<(Plate, String)> = vec![
        (Plate::Process(0), "Cyan".into()),
        (Plate::Process(1), "Magenta".into()),
        (Plate::Process(2), "Yellow".into()),
        (Plate::Process(3), "Black".into()),
    ];
    plates.extend(doc.inks.iter().map(|i| (Plate::Spot(i.id), i.name.clone())));
    plates
}

/// One plate of a page: every colour turned into the grey of the ink's
/// coverage (black = full ink); registration colour prints on all plates.
/// Pictures show the coverage of the plate's ink, pixel by pixel, with
/// their alpha (#1659, `picture_plates`).

/// Map actual fill samples, retaining geometry and alpha. Failed resources are
/// omitted with explicit diagnostics for historical print APIs; export rejects
/// the same diagnostics before committing a destination.
fn check_export_cancel(cancel: Option<&AtomicBool>) -> Result<(), String> {
    if cancel.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        Err(CANCELLED.into())
    } else {
        Ok(())
    }
}
fn map_print_fill(
    fill: &mut tracedraw_core::Fill,
    colour: &dyn Fn(tracedraw_core::Color) -> tracedraw_core::Color,
    picture: &dyn Fn(&tracedraw_core::Shape) -> Result<Option<tracedraw_core::SharedBytes>, String>,
    warnings: &mut Vec<String>,
    cancel: Option<&AtomicBool>,
) -> Result<(), String> {
    check_export_cancel(cancel)?;
    use tracedraw_core::{Color, Fill, Pattern};
    match fill {
        Fill::None => {}
        Fill::Solid(c) => *c = colour(*c),
        Fill::Fountain(g) => {
            for stop in &mut g.stops {
                stop.color = colour(stop.color);
            }
        }
        Fill::Mesh(m) => {
            for node in &mut m.nodes {
                node.color = colour(node.color);
            }
        }
        Fill::Texture(t) => {
            t.color_a = colour(t.color_a);
            t.color_b = colour(t.color_b);
        }
        Fill::Pattern(Pattern::TwoColor { front, back, .. }) => {
            *front = colour(*front);
            *back = colour(*back);
        }
        Fill::Pattern(Pattern::Vector { shapes, .. }) => {
            for shape in shapes {
                map_print_shape(shape, colour, picture, warnings, false, cancel)?;
            }
        }
        Fill::Pattern(Pattern::Bitmap {
            png,
            width_px,
            height_px,
            ..
        }) => {
            check_export_cancel(cancel)?;
            let decoded = crate::bitmap_fx::decode(png);
            check_export_cancel(cancel)?;
            let converted = if let Some(mut pixels) =
                decoded.filter(|p| p.dimensions() == (*width_px, *height_px))
            {
                for row in pixels.rows_mut() {
                    check_export_cancel(cancel)?;
                    for pixel in row {
                        let rgb = colour(Color::rgb8(pixel[0], pixel[1], pixel[2])).to_rgb8();
                        pixel[0] = rgb[0];
                        pixel[1] = rgb[1];
                        pixel[2] = rgb[2];
                    }
                }
                check_export_cancel(cancel)?;
                let encoded = if let Some(flag) = cancel {
                    crate::picture_plates::encode_rgba_with_cancel(&pixels, flag)?
                } else {
                    crate::bitmap_fx::encode(&pixels)
                };
                check_export_cancel(cancel)?;
                encoded
            } else {
                None
            };
            if let Some(bytes) = converted {
                *png = bytes.into();
            } else {
                warnings.push(
                    "cannot decode or encode bitmap pattern at its original dimensions".into(),
                );
                *fill = Fill::None;
            }
        }
    }
    check_export_cancel(cancel)?;
    Ok(())
}
fn map_print_shape(
    shape: &mut tracedraw_core::Shape,
    colour: &dyn Fn(tracedraw_core::Color) -> tracedraw_core::Color,
    picture: &dyn Fn(&tracedraw_core::Shape) -> Result<Option<tracedraw_core::SharedBytes>, String>,
    warnings: &mut Vec<String>,
    required: bool,
    cancel: Option<&AtomicBool>,
) -> Result<(), String> {
    check_export_cancel(cancel)?;
    if !shape.visible && !required {
        return Ok(());
    }
    // The renderer consumes the baked picture instead of these original
    // paints. Normalize the export copy once, then separate what is drawn.
    if let Some(drawn) = shape.drawn_as() {
        *shape = drawn;
    }
    map_print_fill(&mut shape.fill, colour, picture, warnings, cancel)?;
    for fill in &mut shape.extra_fills {
        map_print_fill(fill, colour, picture, warnings, cancel)?;
    }
    if let Some(stroke) = &mut shape.stroke {
        stroke.color = colour(stroke.color);
    }
    for stroke in &mut shape.extra_strokes {
        stroke.color = colour(stroke.color);
    }
    for effect in &mut shape.effects {
        if let tracedraw_core::live::Effect::Blend { end, .. } = effect {
            map_print_shape(end, colour, picture, warnings, true, cancel)?;
        }
    }
    if matches!(shape.kind, ShapeKind::Bitmap { .. }) {
        if let Some(bytes) = picture(shape)? {
            if let ShapeKind::Bitmap { png, fx, .. } = &mut shape.kind {
                *png = bytes;
                *fx = None;
            }
            shape.cmyk_bitmap = false;
            shape.picture_inks = None;
        } else {
            warnings.push(format!("cannot separate bitmap resource {:?}", shape.id));
            shape.visible = false;
        }
        return Ok(());
    }
    let opacity_mask = shape.opacity_mask.is_some();
    match &mut shape.kind {
        ShapeKind::Group { children } => {
            for child in children {
                map_print_shape(child, colour, picture, warnings, false, cancel)?;
            }
        }
        ShapeKind::ClipFrame { frame, contents } => {
            // The frame's luminance is alpha, not a printable paint.
            if !opacity_mask {
                map_print_shape(frame, colour, picture, warnings, false, cancel)?;
            }
            for child in contents {
                map_print_shape(child, colour, picture, warnings, false, cancel)?;
            }
        }
        ShapeKind::Text { spans, .. } => {
            for span in spans {
                if let Some(fill) = &mut span.fill {
                    map_print_fill(fill, colour, picture, warnings, cancel)?;
                }
            }
        }
        _ => {}
    }
    check_export_cancel(cancel)?;
    Ok(())
}
fn map_print_page(
    page: &tracedraw_core::document::Page,
    colour: &dyn Fn(tracedraw_core::Color) -> tracedraw_core::Color,
    picture: &dyn Fn(&tracedraw_core::Shape) -> Result<Option<tracedraw_core::SharedBytes>, String>,
    cancel: Option<&AtomicBool>,
) -> Result<(tracedraw_core::document::Page, Vec<String>), String> {
    check_export_cancel(cancel)?;
    let mut page = page.clone();
    let mut warnings = Vec::new();
    for layer in &mut page.layers {
        if !layer.visible {
            continue;
        }
        for shape in &mut layer.shapes {
            map_print_shape(shape, colour, picture, &mut warnings, false, cancel)?;
        }
    }
    check_export_cancel(cancel)?;
    Ok((page, warnings))
}
fn plate_colour(c: tracedraw_core::Color, plate: Plate) -> tracedraw_core::Color {
    use tracedraw_core::Color;
    if matches!(c, Color::Registration) {
        return Color::Gray { v: 0.0 };
    }
    let value = match (plate, c) {
        (Plate::Spot(id), Color::Spot { ink, tint, .. }) if id == ink => tint,
        (Plate::Spot(_), _) | (Plate::Process(_), Color::Spot { .. }) => 0.0,
        (Plate::Process(i), _) => match c.convert_to("CMYK") {
            Color::Cmyk { c, m, y, k } => [c, m, y, k].get(i).copied().unwrap_or(0.0),
            _ => 0.0,
        },
    };
    Color::Gray {
        v: 1.0 - value.clamp(0.0, 1.0),
    }
}
fn mapped_plate_page(
    page: &tracedraw_core::document::Page,
    plate: Plate,
) -> (tracedraw_core::document::Page, Vec<String>) {
    map_print_page(
        page,
        &|colour| plate_colour(colour, plate),
        &|shape| Ok(crate::picture_plates::plate_picture(shape, plate)),
        None,
    )
    .unwrap_or_else(|error| (page.clone(), vec![error]))
}
fn plate_page_checked(
    page: &tracedraw_core::document::Page,
    plate: Plate,
) -> Result<tracedraw_core::document::Page, String> {
    let (page, warnings) = mapped_plate_page(page, plate);
    if warnings.is_empty() {
        Ok(page)
    } else {
        Err(warnings.join("; "))
    }
}

fn plate_page_checked_with_cancel(
    page: &tracedraw_core::document::Page,
    plate: Plate,
    cancel: &AtomicBool,
) -> Result<tracedraw_core::document::Page, String> {
    let (page, warnings) = map_print_page(
        page,
        &|colour| plate_colour(colour, plate),
        &|shape| crate::picture_plates::plate_picture_with_cancel(shape, plate, cancel),
        Some(cancel),
    )?;
    if warnings.is_empty() {
        Ok(page)
    } else {
        Err(warnings.join("; "))
    }
}

/// Historical print API: preserve every readable fill; explicitly omit only a
/// resource which cannot be decoded, recording the diagnostic.
pub fn plate_page(
    page: &tracedraw_core::document::Page,
    plate: Plate,
) -> tracedraw_core::document::Page {
    let (page, warnings) = mapped_plate_page(page, plate);
    for warning in warnings {
        eprintln!("print colour resource omitted: {warning}");
    }
    page
}

/// A page as printed with only some inks (#571): process colours keep
/// the channels of the inks on, spot colours show when their ink is on
/// and are paper otherwise; registration colour prints with every ink.
/// Pictures keep the inks on, pixel by pixel (#1659); `inks` are the
/// drawing's spot inks, for the colour a picture's spot channel shows.
pub fn inks_page(
    page: &tracedraw_core::document::Page,
    process: [bool; 4],
    spots: &[u32],
    inks: &[tracedraw_core::document::Ink],
) -> tracedraw_core::document::Page {
    use tracedraw_core::Color;
    let spot_colors: Vec<(u32, [u8; 3])> = inks
        .iter()
        .filter(|i| spots.contains(&i.id))
        .map(|i| (i.id, i.color(1.0).to_rgb8()))
        .collect();
    let spots = spots.to_vec();
    let keep = move |c: Color| -> Color {
        let on = |i: usize, v: f32| if process[i] { v } else { 0.0 };
        match c {
            Color::Registration => Color::Cmyk {
                c: on(0, 1.0),
                m: on(1, 1.0),
                y: on(2, 1.0),
                k: on(3, 1.0),
            },
            Color::Spot { ink, .. } if spots.contains(&ink) => c,
            Color::Spot { .. } => Color::Gray { v: 1.0 },
            other => match other.convert_to("CMYK") {
                Color::Cmyk { c, m, y, k } => Color::Cmyk {
                    c: on(0, c),
                    m: on(1, m),
                    y: on(2, y),
                    k: on(3, k),
                },
                same => same,
            },
        }
    };
    let (page, warnings) = map_print_page(
        page,
        &keep,
        &|picture| {
            Ok(crate::picture_plates::inks_picture(
                picture,
                process,
                &spot_colors,
            ))
        },
        None,
    )
    .unwrap_or_else(|error| (page.clone(), vec![error]));
    for warning in warnings {
        eprintln!("print colour resource omitted: {warning}");
    }
    page
}

/// The print layout for the Print dialog's settings: the imposition, the
/// sheet, the bleed and the marks.
pub fn print_layout(app: &App, st: &PrintState, doc: &Document) -> crate::imposition::Options {
    let bleed = if st.bleed {
        st.bleed_limit.clamp(0.0, 50.0)
    } else {
        0.0
    };
    let first = doc
        .pages
        .first()
        .map(|p| p.size)
        .unwrap_or(Size::new(210.0, 297.0));
    let tz = crate::time_tracker::local_zone();
    crate::imposition::Options {
        layout: st.layout(),
        sheet: st.sheet_size_for(first),
        gutter: st.gutter.max(0.0),
        gutter_y: Some(st.gutter_y.max(0.0)),
        order: crate::imposition::PageOrder::from_index(st.page_order),
        bleed,
        scale: if st.fit_to_page {
            1.0
        } else {
            st.scale / 100.0
        },
        back_side: st.back_side,
        keep_size: st.keep_size,
        info_offset: st.info_offset,
        paper_margins: st.margins,
        // Moved and stretched in Print Preview, the whole page with its
        // clip, clear of the printer's margins (#536, #595).
        page_offset: st.art_offset,
        page_stretch: st.art_stretch,
        stretch_fit: st.stretch_fit,
        booklet: st.booklet(),
        marks: crate::imposition::Marks {
            crop: st.crop_marks,
            registration: st.registration_marks,
            color_bars: st.color_bars,
            densitometer: st.densitometer,
            page_info: st.page_info,
            page_numbers: st.page_numbers,
        },
        title: if st.info_text.trim().is_empty() {
            app.document_name()
        } else {
            st.info_text.trim().to_string()
        },
        when: crate::time_tracker::date_time(crate::time_tracker::unix_now(), &tz),
    }
}

/// Map every colour of `s` and what it holds through `f`: fills of every
/// kind, outlines, character fills, groups and clip frames.
pub fn map_colors(
    s: &mut tracedraw_core::Shape,
    f: &dyn Fn(tracedraw_core::Color) -> tracedraw_core::Color,
) {
    fn fill(
        x: &mut tracedraw_core::Fill,
        f: &dyn Fn(tracedraw_core::Color) -> tracedraw_core::Color,
    ) {
        use tracedraw_core::{Fill, Pattern};
        match x {
            Fill::Solid(c) => *c = f(*c),
            Fill::Fountain(g) => g.stops.iter_mut().for_each(|s| s.color = f(s.color)),
            Fill::Mesh(m) => m.nodes.iter_mut().for_each(|n| n.color = f(n.color)),
            Fill::Pattern(Pattern::TwoColor { front, back, .. }) => {
                *front = f(*front);
                *back = f(*back);
            }
            Fill::Texture(texture) => {
                texture.color_a = f(texture.color_a);
                texture.color_b = f(texture.color_b);
            }
            Fill::Pattern(Pattern::Vector { shapes, .. }) => {
                for shape in shapes {
                    map_colors(shape, f);
                }
            }
            _ => {}
        }
    }
    fill(&mut s.fill, f);
    for extra in &mut s.extra_fills {
        fill(extra, f);
    }
    if let Some(st) = &mut s.stroke {
        st.color = f(st.color);
    }
    for st in &mut s.extra_strokes {
        st.color = f(st.color);
    }
    // The copy of its end object a blend keeps: the steps run to the
    // end's colour as mapped (#1541).
    for e in s.effects.iter_mut() {
        if let tracedraw_core::live::Effect::Blend { end, .. } = e {
            map_colors(end, f);
        }
    }
    let opacity_mask = s.opacity_mask.is_some();
    match &mut s.kind {
        ShapeKind::Group { children } => children.iter_mut().for_each(|c| map_colors(c, f)),
        ShapeKind::ClipFrame { frame, contents } => {
            if !opacity_mask {
                map_colors(frame, f);
            }
            contents.iter_mut().for_each(|c| map_colors(c, f));
        }
        ShapeKind::Text { spans, .. } => {
            for sp in spans.iter_mut() {
                if let Some(x) = &mut sp.fill {
                    fill(x, f);
                }
            }
        }
        _ => {}
    }
}

fn invert_colors(s: &mut tracedraw_core::Shape) {
    use tracedraw_core::Color;
    map_colors(s, &|c: Color| {
        let [r, g, b] = c.to_rgb8();
        Color::rgb8(255 - r, 255 - g, 255 - b)
    });
}

/// Colour separations with every plate.
#[cfg(test)]
fn separations_doc(doc: &Document) -> Document {
    separations_doc_without(doc, &[])
}

/// Colour separations: one grey page per sheet and plate (the four
/// process inks, then each spot ink), each the grey of that ink's
/// coverage; a spot colour prints on its own plate only, at its tint.
/// The plates at the indices `off` (in [`plates`] order), those unticked
/// in Print Preview, are left out (#846).
fn separations_doc_without(doc: &Document, off: &[usize]) -> Document {
    let size = doc
        .pages
        .first()
        .map(|p| p.size)
        .unwrap_or(Size::new(210.0, 297.0));
    let mut sep = Document::new(format!("{} (separations)", doc.title), size);
    sep.pages.clear();
    sep.symbols = doc.symbols.clone();
    let plates: Vec<(Plate, String)> = plates(doc)
        .into_iter()
        .enumerate()
        .filter(|(i, _)| !off.contains(i))
        .map(|(_, p)| p)
        .collect();
    for page in &doc.pages {
        for (plate, name) in &plates {
            let mut p = plate_page(page, *plate);
            p.name = format!("{} {}", page.name, name);
            let mut ids = sep.ids().clone();
            p.id = ids.page();
            sep.set_ids(ids);
            sep.pages.push(p);
        }
    }
    sep
}

/// A file name that stays one name inside a folder: no folder
/// separators, no leading dots, never empty.
pub fn shared_file_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | '\0') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let clean = clean.trim().trim_start_matches('.').to_string();
    if clean.is_empty() {
        "document".into()
    } else {
        clean
    }
}

/// Whether `target` is an address with a scheme (`https:`, `mailto:`,
/// `market:`) rather than a file path.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn is_address(target: &str) -> bool {
    target.split_once(':').is_some_and(|(scheme, _)| {
        scheme.len() > 1
            && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

/// Open a file or a web address in the app the system has for it. On
/// Android (#1398) an address opens in its app, a file of the app (or a
/// document the picker returned) in the app picked for its type; a
/// folder cannot be shown there.
#[cfg(target_os = "android")]
pub fn open_with_system(path: &Path) -> Result<(), String> {
    let target = path.to_string_lossy();
    let opened = if is_address(&target) {
        crate::android::outside::view(&target, "")
    } else {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let mime = crate::files::mime_for_extension(ext);
        match crate::files::document_uri(path) {
            Some(uri) => crate::android::outside::view(&uri, mime),
            None if path.is_dir() => return Err(tr("status.no_folders_on_android")),
            None => crate::android::outside::view(&target, mime),
        }
    };
    if opened {
        Ok(())
    } else {
        Err(tr("status.could_not_open_viewer"))
    }
}

#[cfg(test)]
thread_local! {
    /// What the tests asked the system to open: kept here instead of
    /// opening a browser or a viewer on the desktop of whoever runs them.
    pub static OPENED_IN_TESTS: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(not(target_os = "android"))]
#[allow(unreachable_code)]
pub fn open_with_system(path: &Path) -> Result<(), String> {
    #[cfg(test)]
    {
        OPENED_IN_TESTS.with(|o| o.borrow_mut().push(path.display().to_string()));
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("cmd")
        .args(["/C", "start", "", &path.display().to_string()])
        .spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(path).spawn();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let r = std::process::Command::new("xdg-open").arg(path).spawn();
    r.map(|_| ())
        .map_err(|e| format!("{}: {e}", tr("status.could_not_open_viewer")))
}

/// The program and arguments that open `path` in `editor`: the
/// program itself, or on macOS an application bundle through `open -a`.
pub fn editor_command(editor: &str, path: &Path) -> (String, Vec<String>) {
    let editor = editor.trim().trim_matches('"');
    let file = path.display().to_string();
    if cfg!(target_os = "macos") && editor.trim_end_matches('/').ends_with(".app") {
        return ("open".into(), vec!["-a".into(), editor.into(), file]);
    }
    (editor.into(), vec![file])
}

/// Open `path` in the bitmap editor of the preferences, or with the
/// system's program when none is set (#593).
#[cfg(not(target_arch = "wasm32"))]
pub fn open_in_editor(editor: &str, path: &Path) -> Result<(), String> {
    if editor.trim().is_empty() {
        return open_with_system(path);
    }
    let (program, args) = editor_command(editor, path);
    std::process::Command::new(&program)
        .args(&args)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("{}: {program}: {e}", tr("status.could_not_open_viewer")))
}

impl App {
    /// Open a web page in the system browser (a new tab in the browser
    /// build).
    pub fn open_url(&mut self, url: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::web::open_url(url);
            return;
        }
        #[allow(unreachable_code)]
        if let Err(e) = open_with_system(Path::new(url)) {
            self.status = e;
        }
    }

    /// File name for a PDF sent somewhere: the document title, cleaned.
    fn send_to_file_name(&self) -> String {
        let title = self.document_title();
        let stem: String = title
            .trim_end_matches('*')
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let stem = stem.trim();
        let stem = if stem.is_empty() { "Graphic1" } else { stem };
        format!("{stem}.pdf")
    }

    /// File > Send To: write the document as PDF into `target` and report
    /// where it went.
    pub fn send_to(&mut self, target: SendTarget) {
        if crate::files::ANDROID {
            self.send_to_android(target);
            return;
        }
        if crate::files::WEB {
            // No folders in a browser: every target is a PDF download.
            let name = self.send_to_file_name();
            let pdf = tracedraw_io::pdf::document_to_pdf(self.engine.document());
            let path = std::path::PathBuf::from("/download").join(&name);
            self.status = match crate::files::write(&path, pdf) {
                Ok(()) => trf("status.sent_to", &[("path", &name)]),
                Err(e) => trf("status.export_failed", &[("e", &e.to_string())]),
            };
            let _ = target;
            return;
        }
        let Some(dir) = target.dir() else {
            self.status = tr("status.send_to_no_folder");
            return;
        };
        let path = dir.join(self.send_to_file_name());
        match tracedraw_io::save_pdf(self.engine.document(), &path) {
            Ok(()) => {
                self.status = trf("status.sent_to", &[("path", &path.display().to_string())]);
                if target == SendTarget::Mail {
                    // The platform's mail client gets the PDF as an
                    // attachment; without a mailto attachment standard,
                    // open the file's folder so it can be dragged in.
                    let _ = open_with_system(&dir);
                }
            }
            Err(e) => self.status = trf("status.export_failed", &[("e", &e.to_string())]),
        }
    }
}

impl App {
    /// Send To on Android (#1398): Documents saves the PDF where the
    /// system's picker says, Mail Recipient hands it to a mail app through
    /// the share sheet; there is no desktop folder ([`SendTarget::why_not`]).
    fn send_to_android(&mut self, target: SendTarget) {
        if let Some(why) = target.why_not() {
            self.status = tr(why);
            return;
        }
        let name = self.send_to_file_name();
        let pdf = tracedraw_io::pdf::document_to_pdf(self.engine.document());
        if target == SendTarget::Documents {
            crate::files::Dialog::new()
                .add_filter("PDF", &["pdf"])
                .set_file_name(name)
                .save_file(self, move |app, path| {
                    app.status = match crate::files::write(&path, pdf) {
                        Ok(()) => trf("status.sent_to", &[("path", &path.display().to_string())]),
                        Err(e) => trf("status.export_failed", &[("e", &e.to_string())]),
                    };
                });
            return;
        }
        let dir = std::env::temp_dir().join("tracedraw-send");
        let path = dir.join(shared_file_name(&name));
        self.status = match std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, pdf))
        {
            Err(e) => trf("status.export_failed", &[("e", &e.to_string())]),
            Ok(()) if share_pdf(&path, name.trim_end_matches(".pdf")) => {
                trf("status.shared", &[("name", &name)])
            }
            Ok(()) => tr("status.share_failed"),
        };
    }
}

/// Hand the PDF at `path` to a mail app through the share sheet (Android);
/// false where there is none.
fn share_pdf(path: &Path, subject: &str) -> bool {
    #[cfg(target_os = "android")]
    return crate::android::outside::share(path, "application/pdf", subject, true);
    #[cfg(not(target_os = "android"))]
    {
        let _ = (path, subject);
        false
    }
}

/// Destinations of File > Send To.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendTarget {
    Desktop,
    Documents,
    Mail,
}

impl SendTarget {
    /// Why this destination cannot be used here, as an i18n key for the
    /// greyed menu row's tip: Android has no desktop folder.
    pub fn why_not(self) -> Option<&'static str> {
        why_not_send_to(self, crate::files::ANDROID)
    }

    /// The folder the PDF is written to; the mail recipient gets it from
    /// the system's temporary folder.
    pub fn dir(self) -> Option<std::path::PathBuf> {
        match self {
            SendTarget::Desktop => user_dir("XDG_DESKTOP_DIR", "Desktop"),
            SendTarget::Documents => user_dir("XDG_DOCUMENTS_DIR", "Documents"),
            SendTarget::Mail => {
                let d = std::env::temp_dir().join("tracedraw-send");
                std::fs::create_dir_all(&d).ok()?;
                Some(d)
            }
        }
    }
}

/// [`SendTarget::why_not`] on Android (`android`) or elsewhere.
pub fn why_not_send_to(target: SendTarget, android: bool) -> Option<&'static str> {
    (android && target == SendTarget::Desktop).then_some("menu.file.send_to_desktop_android")
}

/// `$XDG_*_DIR`, then `~/<name>` (created when missing), then the home
/// folder itself.
fn user_dir(env: &str, name: &str) -> Option<std::path::PathBuf> {
    if let Some(p) = std::env::var_os(env) {
        let p = std::path::PathBuf::from(p);
        if p.is_dir() {
            return Some(p);
        }
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)?;
    let p = home.join(name);
    if p.is_dir() || std::fs::create_dir_all(&p).is_ok() {
        Some(p)
    } else {
        Some(home)
    }
}

// ----- print merge ---------------------------------------------------------------

/// The text of a data file as spreadsheets save it: UTF-8 (with or
/// without the byte order mark), UTF-16 with its mark (Excel's Unicode
/// Text), else code page 1252 (Excel's CSV on Windows), which failed to
/// read as UTF-8 (#495).
pub fn decode_text(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    let utf16 = |rest: &[u8], le: bool| {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|c| {
                if le {
                    u16::from_le_bytes([c[0], c[1]])
                } else {
                    u16::from_be_bytes([c[0], c[1]])
                }
            })
            .collect();
        String::from_utf16_lossy(&units)
    };
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        return utf16(rest, true);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        return utf16(rest, false);
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes
            .iter()
            .map(|&b| crate::ui::glyphs_docker::from_cp1252(b).unwrap_or('\u{FFFD}'))
            .collect(),
    }
}

/// Records of a CSV text: the first row names the fields; the separator
/// is the comma, semicolon or tab the first row holds most of.
pub fn parse_csv(text: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let first = text.lines().next().unwrap_or_default();
    let delim = b",;\t"
        .iter()
        .copied()
        .max_by_key(|d| first.matches(*d as char).count())
        .filter(|d| first.contains(*d as char))
        .unwrap_or(b',');
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .flexible(true)
        .from_reader(text.as_bytes());
    let headers: Vec<String> = rdr
        .headers()
        .map(|h| h.iter().map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    let rows: Vec<Vec<String>> = rdr
        .records()
        .filter_map(|r| r.ok())
        .map(|r| r.iter().map(|s| s.to_string()).collect())
        .collect();
    (headers, rows)
}

// ----- find and replace ----------------------------------------------------------

fn matches_text(hay: &str, needle: &str, match_case: bool, whole_word: bool) -> bool {
    if needle.is_empty() {
        return false;
    }
    let (h, n) = if match_case {
        (hay.to_string(), needle.to_string())
    } else {
        (hay.to_lowercase(), needle.to_lowercase())
    };
    if !whole_word {
        return h.contains(&n);
    }
    h.split(|c: char| !c.is_alphanumeric()).any(|w| w == n)
}

/// Find text objects containing `needle` (kind 0) or objects whose kind or
/// name matches it (kind 1).
pub fn find_shapes(
    app: &App,
    needle: &str,
    kind: usize,
    match_case: bool,
    whole_word: bool,
) -> Vec<ShapeId> {
    let Ok(page) = app.doc().page(app.page) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    fn walk(
        s: &tracedraw_core::Shape,
        needle: &str,
        kind: usize,
        mc: bool,
        ww: bool,
        out: &mut Vec<ShapeId>,
    ) {
        let hit = match (kind, &s.kind) {
            (0, ShapeKind::Text { spans, .. }) => {
                let t: String = spans.iter().map(|x| x.text.as_str()).collect();
                matches_text(&t, needle, mc, ww)
            }
            (1, k) => {
                let name = s.name.clone().unwrap_or_default();
                let kind_name = crate::ui::dockers::kind_name(k);
                matches_text(&name, needle, mc, ww) || matches_text(&kind_name, needle, mc, false)
            }
            _ => false,
        };
        if hit {
            out.push(s.id);
        }
        match &s.kind {
            ShapeKind::Group { children } => children
                .iter()
                .for_each(|c| walk(c, needle, kind, mc, ww, out)),
            ShapeKind::ClipFrame { frame, contents } => {
                walk(frame, needle, kind, mc, ww, out);
                contents
                    .iter()
                    .for_each(|c| walk(c, needle, kind, mc, ww, out));
            }
            _ => {}
        }
    }
    for s in page.layers.iter().flat_map(|l| &l.shapes) {
        walk(s, needle, kind, match_case, whole_word, &mut out);
    }
    // Only top-level ids are selectable; keep those.
    out.retain(|id| app.doc().shape(*id).is_ok());
    out
}

pub fn replace_text(
    app: &mut App,
    needle: &str,
    replacement: &str,
    match_case: bool,
    whole_word: bool,
    all: bool,
) {
    let ids = find_shapes(app, needle, 0, match_case, whole_word);
    let ids: Vec<ShapeId> = if all {
        ids
    } else {
        ids.into_iter().take(1).collect()
    };
    let mut cmds = Vec::new();
    for id in ids {
        let Some(s) = app.doc().find_shape(id).cloned() else {
            continue;
        };
        if let ShapeKind::Text {
            spans,
            origin,
            frame,
            align,
            para,
            on_path,
        } = s.kind
        {
            let new_spans: Vec<TextSpan> = spans
                .into_iter()
                .map(|sp| TextSpan {
                    text: replace_in(&sp.text, needle, replacement, match_case, whole_word),
                    ..sp
                })
                .collect();
            cmds.push(Command::SetShapeKind {
                shape: id,
                kind: ShapeKind::Text {
                    spans: new_spans,
                    origin,
                    frame,
                    align,
                    para,
                    on_path,
                },
            });
        }
    }
    if !cmds.is_empty() {
        if let Err(e) = app.engine.run_batch("Replace", &cmds) {
            app.status = e.to_string();
        }
    }
}

fn replace_in(
    text: &str,
    needle: &str,
    replacement: &str,
    match_case: bool,
    whole_word: bool,
) -> String {
    if needle.is_empty() {
        return text.to_string();
    }
    if whole_word {
        let mut out = String::new();
        let mut word = String::new();
        let flush = |word: &mut String, out: &mut String| {
            if !word.is_empty() {
                let eq = if match_case {
                    word == needle
                } else {
                    word.to_lowercase() == needle.to_lowercase()
                };
                out.push_str(if eq { replacement } else { word });
                word.clear();
            }
        };
        for c in text.chars() {
            if c.is_alphanumeric() {
                word.push(c);
            } else {
                flush(&mut word, &mut out);
                out.push(c);
            }
        }
        flush(&mut word, &mut out);
        return out;
    }
    if match_case {
        return text.replace(needle, replacement);
    }
    let lower = text.to_lowercase();
    let nl = needle.to_lowercase();
    let mut out = String::new();
    let mut i = 0;
    while let Some(pos) = lower[i..].find(&nl) {
        out.push_str(&text[i..i + pos]);
        out.push_str(replacement);
        i += pos + nl.len();
    }
    out.push_str(&text[i..]);
    out
}

// ----- find and replace by properties -----------------------------------------

impl PropertyFind {
    /// The criteria ticked.
    pub fn query(&self) -> ObjectQuery {
        let color = |none: bool, c| {
            if none {
                ColorQuery::Nothing
            } else {
                ColorQuery::Color(c)
            }
        };
        let font = self.font.trim();
        ObjectQuery {
            object_type: self
                .use_type
                .then(|| ObjectType::ALL.get(self.object_type).copied())
                .flatten(),
            fill: self.use_fill.then(|| color(self.fill_none, self.fill)),
            outline: self
                .use_outline
                .then(|| color(self.outline_none, self.outline)),
            outline_width: self.use_width.then_some(self.width),
            font: (self.use_font && !font.is_empty()).then(|| font.to_string()),
        }
    }

    /// The replacements ticked.
    pub fn replacement(&self) -> objfind::Replacement {
        let font = self.new_font.trim();
        objfind::Replacement {
            fill: self.set_fill.then_some(self.new_fill),
            outline: self.set_outline.then_some(self.new_outline),
            outline_width: self.set_width.then_some(self.new_width),
            font: (self.set_font && !font.is_empty()).then(|| font.to_string()),
        }
    }

    /// Fill in the criteria from `s` (From Selection): its type, fill,
    /// outline, outline width and font, ticking the ones it has.
    pub fn take_from(&mut self, s: &tracedraw_core::Shape) {
        let q = objfind::query_from(s);
        self.use_type = false;
        if let Some(t) = q.object_type {
            self.object_type = ObjectType::ALL.iter().position(|x| *x == t).unwrap_or(0);
        }
        self.use_fill = q.fill.is_some();
        match q.fill {
            Some(ColorQuery::Nothing) => self.fill_none = true,
            Some(ColorQuery::Color(c)) => {
                self.fill_none = false;
                self.fill = c;
            }
            None => {}
        }
        self.use_outline = q.outline.is_some();
        match q.outline {
            Some(ColorQuery::Nothing) => self.outline_none = true,
            Some(ColorQuery::Color(c)) => {
                self.outline_none = false;
                self.outline = c;
            }
            None => {}
        }
        self.use_width = false;
        if let Some(w) = q.outline_width {
            self.width = w;
        }
        self.use_font = q.font.is_some();
        if let Some(f) = q.font {
            self.font = f;
        }
    }
}

/// The objects that meet the ticked criteria, on the current page or in
/// the whole drawing.
pub fn find_by_properties(app: &App, pf: &PropertyFind) -> Vec<Hit> {
    let scope = if pf.whole_drawing {
        objfind::Scope::Document
    } else {
        objfind::Scope::Page
    };
    objfind::find(app.doc(), &pf.query(), scope, app.page_index())
}

/// Find All by properties: select the objects found on the current page.
/// Returns how many of them are on it.
pub fn select_found(app: &mut App, st: &mut FindReplaceState) -> usize {
    st.props.hits = find_by_properties(app, &st.props);
    // No object shown alone yet: Next starts at the first one.
    st.cursor = st.props.hits.len();
    let here = Some(app.page_index());
    let ids: Vec<ShapeId> = st
        .props
        .hits
        .iter()
        .filter(|h| h.page.is_none_or(|p| Some(p) == here))
        .map(|h| h.id)
        .collect();
    let n = ids.len();
    app.select(ids);
    n
}

/// The next (or previous) object found: its page comes into view and it
/// is selected alone. The search runs again first, so it follows edits.
pub fn step_found(app: &mut App, st: &mut FindReplaceState, forward: bool) {
    let before = st.props.hits.get(st.cursor).copied();
    st.props.hits = find_by_properties(app, &st.props);
    let n = st.props.hits.len();
    if n == 0 {
        return;
    }
    // Carry on from the object shown last, wherever it is now.
    let at = before.and_then(|b| st.props.hits.iter().position(|h| h.id == b.id));
    st.cursor = match (at, forward) {
        (None, true) => 0,
        (None, false) => n - 1,
        (Some(i), true) => (i + 1) % n,
        (Some(i), false) => (i + n - 1) % n,
    };
    let Some(hit) = st.props.hits.get(st.cursor).copied() else {
        return;
    };
    if let Some(p) = hit.page {
        if p != app.page_index() {
            app.goto_page(p);
        }
    }
    app.select(vec![hit.id]);
}

/// Replace All by properties, as one undo step. Returns how many
/// objects changed.
pub fn replace_properties(app: &mut App, st: &mut FindReplaceState) -> usize {
    st.props.hits = find_by_properties(app, &st.props);
    let cmds = objfind::replace_commands(
        app.doc(),
        &st.props.hits,
        &st.props.query(),
        &st.props.replacement(),
    );
    if cmds.is_empty() {
        return 0;
    }
    let changed: std::collections::HashSet<ShapeId> = cmds
        .iter()
        .filter_map(|c| match c {
            Command::SetFill { shapes, .. } | Command::SetStroke { shapes, .. } => {
                shapes.first().copied()
            }
            Command::SetShapeKind { shape, .. } => Some(*shape),
            _ => None,
        })
        .collect();
    match app.engine.run_batch("Replace", &cmds) {
        Ok(()) => changed.len(),
        Err(e) => {
            app.status = e.to_string();
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SendTarget, is_address, shared_file_name, why_not_send_to};

    #[test]
    fn prepared_export_job_is_send_and_owns_its_snapshot() {
        use super::*;
        fn assert_send<T: Send>() {}
        assert_send::<ExportJob>();
        assert_send::<PreviewJob>();
        let mut app = App::headless();
        let st = ExportState {
            format: 3,
            dpi: 25.4,
            matte: [30, 50, 70],
            crop_to_page: true,
            ..Default::default()
        };
        let job = prepare_preview_region_job(&app, &st, (4, 5), (20, 18)).unwrap();
        let mut changed = app.doc().clone();
        changed.pages[0].size = Size::new(900.0, 700.0);
        app.engine.replace(changed);
        let result = job.run(Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(result.output_size, (210, 297));
        assert_eq!((result.image.width(), result.image.height()), (20, 18));
        assert_eq!(&result.original.data()[..4], &[30, 50, 70, 255]);
        assert!(
            result.image.data()[..3]
                .iter()
                .zip([30u8, 50, 70])
                .all(|(a, b)| a.abs_diff(b) <= 3)
        );
        assert!(result.profile_bytes > 128);
        assert_eq!(result.profile_name, "Craft Plumilla sRGB");
    }
    #[test]
    fn selection_export_keeps_metadata_fonts_and_all_profile_context() {
        use super::*;
        let mut app = App::headless();
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(2.0, 3.0, 12.0, 14.0),
            radius: 0.0,
            corners: None,
        });
        let mut original = app.doc().clone();
        original.metadata.author = "Export fixture".into();
        original.metadata.rendering_intent = "Relative Colorimetric".into();
        original.rgb_profile = Some(tracedraw_core::DocumentProfile::new(
            "Fixture RGB",
            tracedraw_io::raster_export::srgb_profile_bytes(),
        ));
        original.gray_profile = Some(tracedraw_core::DocumentProfile::new(
            "Fixture Gray",
            vec![10, 11, 12],
        ));
        app.engine.replace(original.clone());
        let id = app.doc().pages[0].layers[0].shapes[0].id;
        app.select(vec![id]);
        let (selected, _) = export_document(
            &app,
            &ExportState {
                selection_only: true,
                ..Default::default()
            },
        );
        assert_eq!(selected.metadata, original.metadata);
        assert_eq!(selected.rgb_profile, original.rgb_profile);
        assert_eq!(selected.gray_profile, original.gray_profile);
        assert_eq!(selected.cmyk_profile, original.cmyk_profile);
        assert_eq!(selected.fonts, original.fonts);
        assert_eq!(selected.color_context, original.color_context);
        assert_eq!(selected.pages.len(), 1);
        assert_eq!(selected.pages[0].layers[0].shapes.len(), 1);
    }
    #[test]
    fn output_blur_changes_neighbours_and_cancellation_leaves_destination() {
        use super::*;
        let cancel = AtomicBool::new(false);
        let mut data = vec![0u8; 9 * 9 * 3];
        data[(4 * 9 + 4) * 3..(4 * 9 + 4) * 3 + 3].fill(255);
        blur_export_samples(&mut data, 9, 9, 3, 1.0, &cancel, &|_| {}).unwrap();
        assert!(data[(4 * 9 + 4) * 3] < 255);
        assert!(data[(4 * 9 + 3) * 3] > 0);
        assert_eq!(data[0], 0);
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            blur_export_samples(&mut data, 9, 9, 3, 1.0, &cancel, &|_| {}).unwrap_err(),
            CANCELLED
        );
    }

    #[test]
    fn jpeg_crop_and_unlocked_scale_transform_master_layers_with_the_page() {
        use super::*;
        use tracedraw_core::{Color, Fill};
        let mut app = App::headless();
        app.new_shape_styled(
            ShapeKind::Rect {
                rect: Rect::new(100.0, 100.0, 120.0, 110.0),
                radius: 0.0,
                corners: None,
            },
            |shape| {
                shape.fill = Fill::Solid(Color::rgb8(0, 0, 255));
                shape.stroke = None;
            },
        );
        let mut doc = app.doc().clone();
        let mut master = doc.pages[0].layers[0].clone();
        master.in_front = true;
        let mut object = master.shapes[0].clone();
        object.transform_by(Affine::translate((20.0, 0.0)));
        object.fill = Fill::Solid(Color::rgb8(255, 0, 0));
        master.shapes = vec![object];
        let mut hidden_object = master.shapes[0].clone();
        hidden_object.id = doc.ids_mut().shape();
        hidden_object.visible = false;
        hidden_object.transform_by(Affine::translate((500.0, 500.0)));
        doc.pages[0].layers[0].shapes.push(hidden_object);
        let mut hidden_layer = master.clone();
        hidden_layer.id = doc.ids_mut().layer();
        hidden_layer.visible = false;
        hidden_layer.shapes[0].transform_by(Affine::translate((1000.0, 1000.0)));
        doc.pages[0].layers.push(hidden_layer);
        doc.master = vec![master];
        app.engine.replace(doc);
        let st = ExportState {
            format: 3,
            dpi: 25.4,
            crop_to_page: false,
            keep_ratio: false,
            scale: 1.0,
            scale_y: 2.0,
            ..Default::default()
        };
        let result = prepare_preview_job(&app, &st, 200)
            .unwrap()
            .run(Arc::new(AtomicBool::new(false)))
            .unwrap();
        assert_eq!(result.output_size, (40, 20));
        assert_eq!(result.base_size_mm, (40.0, 10.0));
        let at = |x: usize, y: usize| {
            &result.original.data()[((y * 40 + x) * 4)..((y * 40 + x) * 4 + 3)]
        };
        assert_eq!(at(10, 10), &[0, 0, 255]);
        assert_eq!(at(30, 10), &[255, 0, 0]);
        assert_eq!(
            app.doc().master[0].shapes[0].bounds(),
            Rect::new(120.0, 100.0, 140.0, 110.0)
        );
    }
    #[test]
    fn jpeg_gray_proof_profile_really_changes_output_pixels_without_attaching_gray_icc_to_rgb() {
        use super::*;
        use tracedraw_core::icc::test_profiles::{Builder, curv_gamma};
        use tracedraw_core::{Color, Fill};
        let mut app = App::headless();
        app.new_shape_styled(
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 20.0, 20.0),
                radius: 0.0,
                corners: None,
            },
            |shape| {
                shape.fill = Fill::Solid(Color::rgb8(240, 30, 30));
                shape.stroke = None;
            },
        );
        let gray = Builder::new(b"mntr", b"GRAY", b"XYZ ", 2)
            .tag(b"kTRC", curv_gamma(1.8))
            .build();
        let mut doc = app.doc().clone();
        doc.gray_profile = Some(tracedraw_core::DocumentProfile::new("Proof Gray", gray));
        app.engine.replace(doc);
        app.settings.color.proof_profile = "Proof Gray".into();
        let st = ExportState {
            format: 3,
            dpi: 25.4,
            crop_to_page: false,
            use_proof_profile: true,
            ..Default::default()
        };
        let result = prepare_preview_job(&app, &st, 100)
            .unwrap()
            .run(Arc::new(AtomicBool::new(false)))
            .unwrap();
        let pixel = &result.original.data()[0..3];
        assert!(
            pixel[0].abs_diff(pixel[1]) < 3 && pixel[1].abs_diff(pixel[2]) < 3,
            "gray proof left coloured pixels: {pixel:?}"
        );
        assert_ne!(pixel, &[240, 30, 30]);
        assert_eq!(result.profile_name, "Craft Plumilla sRGB");
        assert!(result.profile_bytes > 128);
    }

    #[test]
    fn preview_screen_conversion_observes_cancel_between_rows_and_preserves_exact_samples() {
        use std::cell::Cell;
        use tracedraw_io::raster_export::Samples;
        for samples in [Samples::Rgb, Samples::Rgba, Samples::Gray, Samples::Cmyk] {
            let source: Vec<u8> = (0..7 * 5 * samples.channels())
                .map(|n| (n * 37 % 256) as u8)
                .collect();
            let original = source.clone();
            let reference = samples_for_preview(
                &source,
                7,
                5,
                samples,
                None,
                tracedraw_core::icc::Intent::RelativeColorimetric,
                false,
            )
            .unwrap();
            let flag = AtomicBool::new(false);
            let actual = samples_for_preview_with_cancel(
                &source,
                7,
                5,
                samples,
                None,
                tracedraw_core::icc::Intent::RelativeColorimetric,
                false,
                &flag,
            )
            .unwrap();
            assert_eq!(actual.data(), reference.data());
            let visits = Cell::new(0);
            let result = samples_for_preview_checked(
                &source,
                7,
                5,
                samples,
                None,
                tracedraw_core::icc::Intent::RelativeColorimetric,
                false,
                &|| {
                    visits.set(visits.get() + 1);
                    if visits.get() == 4 {
                        flag.store(true, Ordering::Relaxed);
                    }
                    check_export_cancel(Some(&flag))
                },
            );
            assert!(matches!(result, Err(ref error) if error == CANCELLED));
            assert_eq!(visits.get(), 4, "one row completed before cancellation");
            assert_eq!(source, original);
            assert!(
                matches!(samples_for_preview_with_cancel(&source, 7, 5, samples, None,
                tracedraw_core::icc::Intent::RelativeColorimetric, false, &flag), Err(ref error) if error == CANCELLED)
            );
        }
    }

    #[test]
    fn cancelled_plate_traversal_propagates_error_and_preserves_source_and_legacy_pixels() {
        let doc = picture_doc(
            2,
            &[
                255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 37, 255, 255, 255, 0,
            ],
            None,
        );
        let page = &doc.pages[0];
        let before = page.clone();
        let flag = AtomicBool::new(false);
        let legacy = plate_page_checked(page, Plate::Process(0)).unwrap();
        let actual = plate_page_checked_with_cancel(page, Plate::Process(0), &flag).unwrap();
        assert_eq!(actual, legacy);
        let result = map_print_page(
            page,
            &|colour| colour,
            &|_| {
                flag.store(true, Ordering::Relaxed);
                Err(CANCELLED.into())
            },
            Some(&flag),
        );
        assert!(
            matches!(result, Err(ref error) if error == CANCELLED),
            "cancel must not become an omitted-resource warning"
        );
        assert!(
            matches!(plate_page_checked_with_cancel(page, Plate::Process(0), &flag), Err(ref error) if error == CANCELLED)
        );
        assert_eq!(page, &before);
    }

    #[test]
    fn jpeg_preview_estimate_scales_image_bytes_but_keeps_real_icc_app2_fixed_at_two_area_factors()
    {
        use tracedraw_io::raster_export::{self as output, Samples};
        let mut profile = output::srgb_profile_bytes();
        // Valid RGB ICC with harmless trailing padding, large enough for three APP2 chunks.
        profile.resize(150_000, 0);
        let size = profile.len() as u32;
        profile[..4].copy_from_slice(&size.to_be_bytes());
        assert!(tracedraw_core::icc::Profile::parse(&profile).is_ok());
        let pixels: Vec<u8> = (0..21 * 17 * 3).map(|i| (i * 43 % 256) as u8).collect();
        let cancelled = AtomicBool::new(false);
        let control = output::Control {
            cancelled: &cancelled,
            row_done: &|_| {},
        };
        for progressive in [false, true] {
            let options = output::JpegOptions {
                quality: 50,
                subsampling: 1,
                progressive,
                optimize_huffman: true,
                dpi: 300.0,
            };
            let mut embedded = Vec::new();
            let mut plain = Vec::new();
            output::jpeg_to(
                &mut embedded,
                21,
                17,
                &pixels,
                Samples::Rgb,
                &options,
                Some(&profile),
                &control,
            )
            .unwrap();
            output::jpeg_to(
                &mut plain,
                21,
                17,
                &pixels,
                Samples::Rgb,
                &options,
                None,
                &control,
            )
            .unwrap();
            // Independent inspection of real APP2 length fields and reassembled ICC.
            let mut measured = 0usize;
            let mut recovered = Vec::new();
            let mut segments = 0;
            for (offset, signature) in embedded.windows(12).enumerate() {
                if signature != b"ICC_PROFILE\0" {
                    continue;
                }
                assert_eq!(&embedded[offset - 4..offset - 2], &[0xff, 0xe2]);
                let length = usize::from(u16::from_be_bytes([
                    embedded[offset - 2],
                    embedded[offset - 1],
                ]));
                segments += 1;
                assert_eq!(embedded[offset + 12] as usize, segments);
                recovered.extend_from_slice(&embedded[offset + 14..offset + length - 2]);
                measured += length + 2;
            }
            assert_eq!(segments, 3);
            assert_eq!(recovered, profile);
            assert_eq!(jpeg_fixed_icc_bytes(&embedded), Some(measured));
            assert_eq!(jpeg_fixed_icc_bytes(&plain), Some(0));
            assert_eq!(embedded.len() - plain.len(), measured);
            let decoded = |bytes: &[u8]| image::load_from_memory(bytes).unwrap().into_rgb8();
            assert_eq!(decoded(&embedded), decoded(&plain));
            for factor in [4u64, 137] {
                let small = 21 * 17;
                let without = estimate_preview_bytes(&plain, "jpg", small * factor, small).unwrap();
                let with = estimate_preview_bytes(&embedded, "jpg", small * factor, small).unwrap();
                assert_eq!(
                    without,
                    plain.len() * factor as usize,
                    "embed=false must retain the old estimate"
                );
                assert_eq!(
                    with - without,
                    measured,
                    "ICC must not be multiplied by the image area"
                );
                assert_ne!(
                    with,
                    embedded.len() * factor as usize,
                    "the old causal overestimate must be rejected"
                );
            }
            assert_eq!(
                estimate_preview_bytes(&embedded, "jpg", 21 * 17, 21 * 17),
                Some(embedded.len())
            );
        }
    }

    #[test]
    fn jpeg_icc_size_reader_handles_truncated_segments_and_preserves_other_format_estimates() {
        for bytes in [
            &[][..],
            &[0xff],
            &[0xff, 0xd8, 0xff, 0xe2],
            &[0xff, 0xd8, 0xff, 0xe2, 0, 1],
            &[0xff, 0xd8, 0xff, 0xe2, 0, 20, 1, 2, 3],
        ] {
            assert!(jpeg_fixed_icc_bytes(bytes).is_none());
            assert!(estimate_preview_bytes(bytes, "jpg", 100, 10).is_none());
        }
        let non_icc_app2 = [0xff, 0xd8, 0xff, 0xe2, 0, 4, 42, 43, 0xff, 0xd9];
        assert_eq!(jpeg_fixed_icc_bytes(&non_icc_app2), Some(0));
        assert_eq!(
            estimate_preview_bytes(&non_icc_app2, "jpg", 100, 10),
            Some(100)
        );
        for ext in ["png", "tif"] {
            assert_eq!(
                estimate_preview_bytes(b"unmodified-other-format", ext, 1370, 10),
                Some(b"unmodified-other-format".len() * 137)
            );
        }
    }

    #[test]
    fn cmyk_export_keeps_texture_and_all_pattern_details_at_original_tile_resolution() {
        use super::*;
        use tracedraw_core::{Color, Fill, Pattern, PatternTile, Shape, Texture, TextureKind};
        let cyan = Color::cmyk8(255, 0, 0, 0);
        let white = Color::cmyk8(0, 0, 0, 0);
        let mut tile = tiny_skia::Pixmap::new(2, 2).unwrap();
        tile.data_mut().copy_from_slice(&[
            0, 174, 239, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 174, 239, 255,
        ]);
        let png: tracedraw_core::SharedBytes = tile.encode_png().unwrap().into();
        let mut front = Shape::new(
            tracedraw_core::ShapeId(80001),
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 1.0, 2.0),
                radius: 0.0,
                corners: None,
            },
        );
        front.fill = Fill::Solid(cyan);
        front.stroke = None;
        let mut back = front.clone();
        back.id = tracedraw_core::ShapeId(80002);
        back.kind = ShapeKind::Rect {
            rect: Rect::new(1.0, 0.0, 2.0, 2.0),
            radius: 0.0,
            corners: None,
        };
        back.fill = Fill::Solid(white);
        let fills = [
            Fill::Pattern(Pattern::TwoColor {
                tile: PatternTile::Checker,
                front: cyan,
                back: white,
                size_mm: 4.0,
            }),
            Fill::Texture(Texture {
                kind: TextureKind::Noise,
                color_a: cyan,
                color_b: white,
                scale: 2.0,
                seed: 123,
            }),
            Fill::Pattern(Pattern::Bitmap {
                png: png.clone(),
                width_px: 2,
                height_px: 2,
                size_mm: 4.0,
            }),
            Fill::Pattern(Pattern::Vector {
                shapes: vec![front, back],
                tile: Size::new(2.0, 2.0),
            }),
        ];
        for fill in fills {
            let name = format!("{fill:?}");
            let native_inks = !matches!(fill, Fill::Pattern(Pattern::Bitmap { .. }));
            let mut app = App::headless();
            app.new_shape_styled(
                ShapeKind::Rect {
                    rect: Rect::new(0.0, 0.0, 20.0, 20.0),
                    radius: 0.0,
                    corners: None,
                },
                |shape| {
                    shape.fill = fill;
                    shape.stroke = None;
                },
            );
            let st = ExportState {
                format: 3,
                color_mode: 1,
                dpi: 25.4,
                crop_to_page: false,
                ..Default::default()
            };
            let job = prepare_job(&app, Path::new("unused.jpg"), &st).unwrap();
            let cancel = AtomicBool::new(false);
            let work = ExportWork {
                callback: None,
                cancel: &cancel,
                done: std::cell::Cell::new(0),
                total: 2400,
            };
            let opts = job.raster_options(0).unwrap();
            let (data, samples, _) = job.make_raster(0, &opts, &work).unwrap();
            assert_eq!(samples, tracedraw_io::raster_export::Samples::Cmyk);
            assert_eq!((opts.width, opts.height), (20, 20));
            let mut low = 255u8;
            let mut high = 0u8;
            for pixel in data.chunks_exact(4) {
                low = low.min(pixel[0]);
                high = high.max(pixel[0]);
            }
            assert!(
                high.saturating_sub(low) > 40,
                "separation flattened fill {name}: cyan range{low}..{high}"
            );
            if native_inks {
                assert!(
                    data.chunks_exact(4)
                        .all(|pixel| pixel[1..].iter().all(|ink| *ink <= 2)),
                    "unwanted ink in {name}"
                );
            }
            for page in [
                plate_page(&job.doc.pages[0], Plate::Process(0)),
                inks_page(&job.doc.pages[0], [true, false, false, false], &[], &[]),
            ] {
                let mut printed = job.doc.clone();
                let id = page.id;
                printed.pages = vec![page];
                let raster = tracedraw_render::render_page(&printed, id, &opts).unwrap();
                let low = raster.data().chunks_exact(4).map(|p| p[0]).min().unwrap();
                let high = raster.data().chunks_exact(4).map(|p| p[0]).max().unwrap();
                assert!(high.saturating_sub(low) > 40, "print flattened {name}");
            }

            if let Fill::Pattern(Pattern::Bitmap {
                width_px,
                height_px,
                ..
            }) = &job.doc.pages[0].layers[0].shapes[0].fill
            {
                assert_eq!((*width_px, *height_px), (2, 2));
            }
        }
    }

    #[test]
    fn bitmap_pattern_separation_keeps_every_pixel_dimension_and_alpha() {
        use super::*;
        use tracedraw_core::{Fill, Pattern};
        let mut app = App::headless();
        let mut image = tiny_skia::Pixmap::new(2, 2).unwrap();
        image
            .data_mut()
            .copy_from_slice(&[0, 0, 0, 0, 0, 30, 50, 64, 0, 70, 100, 128, 0, 174, 239, 255]);
        let png = image.encode_png().unwrap();
        app.new_shape_styled(
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            },
            |shape| {
                shape.fill = Fill::Pattern(Pattern::Bitmap {
                    png: png.into(),
                    width_px: 2,
                    height_px: 2,
                    size_mm: 4.0,
                });
                shape.stroke = None;
            },
        );
        let plate = plate_page_checked(&app.doc().pages[0], Plate::Process(0)).unwrap();
        let Fill::Pattern(Pattern::Bitmap {
            png,
            width_px,
            height_px,
            ..
        }) = &plate.layers[0].shapes[0].fill
        else {
            panic!("pattern replaced");
        };
        assert_eq!((*width_px, *height_px), (2, 2));
        let decoded = crate::bitmap_fx::decode(png).unwrap();
        assert_eq!(decoded.dimensions(), (2, 2));
        assert_eq!(
            decoded.pixels().map(|pixel| pixel[3]).collect::<Vec<_>>(),
            vec![0, 64, 128, 255]
        );
        assert!(
            decoded
                .pixels()
                .all(|pixel| pixel[0] == pixel[1] && pixel[1] == pixel[2])
        );
    }
    #[test]
    fn cmyk_export_rejects_unreadable_pattern_without_replacing_destination() {
        use super::*;
        use tracedraw_core::{Fill, Pattern};
        let mut app = App::headless();
        app.new_shape_styled(
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            },
            |shape| {
                shape.fill = Fill::Pattern(Pattern::Bitmap {
                    png: vec![1, 2, 3].into(),
                    width_px: 2,
                    height_px: 2,
                    size_mm: 4.0,
                });
                shape.stroke = None;
            },
        );
        let path =
            std::env::temp_dir().join(format!("craft-invalid-pattern-{}.jpg", std::process::id()));
        std::fs::write(&path, b"preserve destination").unwrap();
        let state = ExportState {
            format: 3,
            color_mode: 1,
            dpi: 25.4,
            ..Default::default()
        };
        let result = prepare_job(&app, &path, &state)
            .unwrap()
            .run(None, Arc::new(AtomicBool::new(false)));
        assert!(result.unwrap_err().contains("bitmap pattern"));
        assert_eq!(std::fs::read(&path).unwrap(), b"preserve destination");
        std::fs::remove_file(path).unwrap();
        let print = plate_page(&app.doc().pages[0], Plate::Process(0));
        assert!(matches!(print.layers[0].shapes[0].fill, Fill::None));
    }

    fn memo_job(
        mode: usize,
        proof_kind: u8,
        intent: tracedraw_core::icc::Intent,
        bpc: bool,
    ) -> super::ExportJob {
        use super::*;
        use tracedraw_core::icc::test_profiles::{
            Builder, curv_gamma, intent_cmyk_profile, matrix_trc_profile,
        };
        let app = App::headless();
        let state = ExportState {
            format: 3,
            color_mode: mode,
            use_proof_profile: false,
            ..Default::default()
        };
        let mut job = prepare_job(&app, Path::new("memo-unused.jpg"), &state).unwrap();
        job.rgb = Some(matrix_trc_profile(
            curv_gamma(1.8),
            curv_gamma(2.2),
            curv_gamma(2.6),
        ));
        job.gray = Some(
            Builder::new(b"mntr", b"GRAY", b"XYZ ", 2)
                .tag(b"kTRC", curv_gamma(1.8))
                .build(),
        );
        job.intent = intent;
        job.bpc = bpc;
        job.state.use_proof_profile = proof_kind != 0;
        job.proof = match proof_kind {
            1 => Some(intent_cmyk_profile(false)),
            2 => job.gray.clone(),
            3 => job.rgb.clone(),
            _ => None,
        };
        job
    }
    fn memo_samples(channels: usize) -> Vec<u8> {
        let colours = [
            [0, 0, 0],
            [255, 255, 255],
            [255, 0, 255],
            [0, 255, 254],
            [254, 208, 20],
            [127, 128, 129],
            [1, 2, 3],
            [255, 254, 253],
            [64, 64, 64],
        ];
        (0..37 * 5)
            .flat_map(|n| {
                let rgb = colours[n % colours.len()];
                let mut pixel = rgb.to_vec();
                if channels == 4 {
                    pixel.push([0, 1, 128, 255][(n / colours.len()) % 4]);
                }
                pixel
            })
            .collect()
    }
    #[test]
    fn output_rgb24_memo_matches_uncached_every_byte_across_profiles_intents_bpc_and_alpha() {
        use super::*;
        use tracedraw_core::icc::Intent;
        use tracedraw_io::raster_export::Samples;
        let scoped: tracedraw_core::color::engine::RgbConverter =
            std::rc::Rc::new(|colour| colour.to_rgb_f32_builtin());
        tracedraw_core::color::engine::with_icc_scope(None, false, || {
            tracedraw_core::color::engine::with_rgb_converter(scoped, || {
                for mode in [0, 2] {
                    for proof in [0, 1, 2, 3] {
                        for intent in [
                            Intent::Perceptual,
                            Intent::RelativeColorimetric,
                            Intent::Saturation,
                            Intent::AbsoluteColorimetric,
                        ] {
                            for bpc in [false, true] {
                                let job = memo_job(mode, proof, intent, bpc);
                                for input in [Samples::Rgb, Samples::Rgba] {
                                    let source = memo_samples(input.channels());
                                    let mut reference = source.clone();
                                    let mut cached = source.clone();
                                    let cancel = AtomicBool::new(false);
                                    let work = || ExportWork {
                                        callback: None,
                                        cancel: &cancel,
                                        done: std::cell::Cell::new(0),
                                        total: 185,
                                    };
                                    let expected = job
                                        .convert_output_with(
                                            &mut reference,
                                            37,
                                            5,
                                            input,
                                            &work(),
                                            OutputConversion::Reference,
                                        )
                                        .unwrap();
                                    let actual = job
                                        .convert_output_with(
                                            &mut cached,
                                            37,
                                            5,
                                            input,
                                            &work(),
                                            OutputConversion::Memo {
                                                byte_budget: RGB_MEMO_BYTES,
                                            },
                                        )
                                        .unwrap();
                                    assert_eq!(actual, expected);
                                    assert_eq!(
                                        cached, reference,
                                        "mode{mode} proof{proof} intent{intent:?} bpc{bpc} input{input:?}"
                                    );
                                    if actual.0 == Samples::Rgba {
                                        assert_eq!(
                                            cached
                                                .chunks_exact(4)
                                                .map(|p| p[3])
                                                .collect::<Vec<_>>(),
                                            source
                                                .chunks_exact(4)
                                                .map(|p| p[3])
                                                .collect::<Vec<_>>()
                                        );
                                    }
                                    let slot = job.rgb_memo.borrow();
                                    assert_eq!(
                                        slot.table.as_ref().unwrap().slots.len() * 4,
                                        RGB_MEMO_BYTES
                                    );
                                }
                            }
                        }
                    }
                }
            })
        });
    }
    #[test]
    fn output_identity_preserves_all_bytes_allocation_icc_and_stage_progress() {
        use super::*;
        use tracedraw_io::raster_export::Samples;
        let app = App::headless();
        for (mode, input) in [(0, Samples::Rgb), (0, Samples::Rgba), (1, Samples::Cmyk)] {
            let state = ExportState {
                format: 3,
                color_mode: mode,
                ..Default::default()
            };
            let mut job = prepare_job(&app, Path::new("identity-unused.jpg"), &state).unwrap();
            job.rgb = None;
            job.proof = None;
            if mode == 1 {
                job.cmyk = Some(tracedraw_core::icc::test_profiles::lut16_cmyk_profile());
            }
            let source: Vec<u8> = (0..256 * input.channels()).map(|n| n as u8).collect();
            let mut actual = source.clone();
            let mut reference = source.clone();
            let ptr = actual.as_ptr();
            let cap = actual.capacity();
            let cancel = AtomicBool::new(false);
            let work = ExportWork {
                callback: None,
                cancel: &cancel,
                done: std::cell::Cell::new(0),
                total: 256,
            };
            let expected = job
                .convert_output_with(
                    &mut reference,
                    16,
                    16,
                    input,
                    &work,
                    OutputConversion::Reference,
                )
                .unwrap();
            work.done.set(0);
            let result = job
                .convert_output(&mut actual, 16, 16, input, &work)
                .unwrap();
            assert_eq!(result, expected);
            assert_eq!(actual, source);
            assert_eq!(actual, reference);
            assert_eq!(actual.as_ptr(), ptr);
            assert_eq!(actual.capacity(), cap);
            assert_eq!(work.done.get(), 256);
            assert!(!job.rgb_memo.borrow().attempted);
        }
    }
    #[test]
    fn output_memo_is_bounded_optional_and_identity_still_validates_profile_and_input() {
        use super::*;
        use tracedraw_io::raster_export::Samples;
        assert!(
            OutputConversion::Auto
                .memo_budget(RGB_MEMO_MIN_PIXELS - 1)
                .is_none()
        );
        assert_eq!(
            OutputConversion::Auto.memo_budget(RGB_MEMO_MIN_PIXELS),
            Some(64 << 20)
        );
        assert!(ExactRgbMemo::try_new(RGB_MEMO_BYTES - 1).is_none());
        let job = memo_job(0, 1, tracedraw_core::icc::Intent::Saturation, true);
        let source = memo_samples(4);
        let mut actual = source.clone();
        let mut expected = source;
        let cancel = AtomicBool::new(false);
        let work = ExportWork {
            callback: None,
            cancel: &cancel,
            done: std::cell::Cell::new(0),
            total: 370,
        };
        job.convert_output_with(
            &mut expected,
            37,
            5,
            Samples::Rgba,
            &work,
            OutputConversion::Reference,
        )
        .unwrap();
        job.convert_output_with(
            &mut actual,
            37,
            5,
            Samples::Rgba,
            &work,
            OutputConversion::Memo { byte_budget: 0 },
        )
        .unwrap();
        assert_eq!(actual, expected);
        assert!(job.rgb_memo.borrow().attempted);
        assert!(job.rgb_memo.borrow().table.is_none());
        let app = App::headless();
        let mut bad = prepare_job(
            &app,
            Path::new("invalid-identity.jpg"),
            &ExportState {
                format: 3,
                color_mode: 1,
                ..Default::default()
            },
        )
        .unwrap();
        bad.cmyk = Some(tracedraw_io::raster_export::srgb_profile_bytes());
        let mut bytes = vec![0; 16];
        assert!(
            bad.convert_output(&mut bytes, 2, 2, Samples::Cmyk, &work)
                .is_err()
        );
        assert_eq!(bytes, vec![0; 16]);
        bad.cmyk = None;
        assert!(
            bad.convert_output(&mut bytes, 1, 1, Samples::Cmyk, &work)
                .is_err()
        );
    }
    #[test]
    fn explicit_srgb_profile_is_not_assumed_identity_and_small_images_do_not_allocate_memo() {
        use super::*;
        use tracedraw_io::raster_export::Samples;
        let app = App::headless();
        let mut job = prepare_job(
            &app,
            Path::new("explicit-srgb.jpg"),
            &ExportState {
                format: 3,
                ..Default::default()
            },
        )
        .unwrap();
        job.rgb = Some(tracedraw_io::raster_export::srgb_profile_bytes());
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let seen = calls.clone();
        let callback: ExportProgressCallback = Arc::new(move |_| {
            seen.fetch_add(1, Ordering::Relaxed);
        });
        let cancel = AtomicBool::new(false);
        let work = ExportWork {
            callback: Some(&callback),
            cancel: &cancel,
            done: std::cell::Cell::new(0),
            total: 16,
        };
        let mut data = vec![90; 16 * 3];
        job.convert_output(&mut data, 4, 4, Samples::Rgb, &work)
            .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 4);
        assert!(!job.rgb_memo.borrow().attempted);
    }
    #[test]
    fn output_memo_warm_hits_and_reference_cancel_on_the_same_row_without_changing_alpha() {
        use super::*;
        use tracedraw_io::raster_export::Samples;
        let job = memo_job(
            0,
            1,
            tracedraw_core::icc::Intent::AbsoluteColorimetric,
            true,
        );
        let source = memo_samples(4);
        let warm_cancel = AtomicBool::new(false);
        let warm = ExportWork {
            callback: None,
            cancel: &warm_cancel,
            done: std::cell::Cell::new(0),
            total: 185,
        };
        let mut bytes = source.clone();
        job.convert_output_with(
            &mut bytes,
            37,
            5,
            Samples::Rgba,
            &warm,
            OutputConversion::Memo {
                byte_budget: RGB_MEMO_BYTES,
            },
        )
        .unwrap();
        let mut results = Vec::new();
        for policy in [
            OutputConversion::Reference,
            OutputConversion::Memo {
                byte_budget: RGB_MEMO_BYTES,
            },
        ] {
            let cancel = Arc::new(AtomicBool::new(false));
            let flag = cancel.clone();
            let callback: ExportProgressCallback = Arc::new(move |p| {
                if p.completed >= 37 {
                    flag.store(true, Ordering::Relaxed);
                }
            });
            let work = ExportWork {
                callback: Some(&callback),
                cancel: &cancel,
                done: std::cell::Cell::new(0),
                total: 185,
            };
            let mut bytes = source.clone();
            let error = job
                .convert_output_with(&mut bytes, 37, 5, Samples::Rgba, &work, policy)
                .unwrap_err();
            assert_eq!(error, CANCELLED);
            assert_eq!(work.done.get(), 37);
            results.push(bytes);
        }
        assert_eq!(results[0], results[1]);
        let cancel = AtomicBool::new(true);
        let work = ExportWork {
            callback: None,
            cancel: &cancel,
            done: std::cell::Cell::new(0),
            total: 185,
        };
        let mut bytes = source.clone();
        assert_eq!(
            job.convert_output(&mut bytes, 37, 5, Samples::Rgba, &work)
                .unwrap_err(),
            CANCELLED
        );
        assert_eq!(bytes, source);
    }
    #[test]
    fn jpeg_zero_quality_then_png_keeps_pixels_dpi_and_original_dialog_preference() {
        use super::*;
        use tracedraw_core::{Color, Fill};
        let mut app = App::headless();
        let size = Size::new(5.0 * 25.4 / 300.0, 4.0 * 25.4 / 300.0);
        let mut doc = app.doc().clone();
        doc.pages[0].size = size;
        app.engine.replace(doc);
        app.new_shape_styled(
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, size.width, size.height),
                radius: 0.0,
                corners: None,
            },
            |shape| {
                shape.fill = Fill::Solid(Color::rgb8(17, 84, 203));
                shape.stroke = None;
            },
        );
        let mut state = ExportState {
            format: 3,
            quality: 0,
            dpi: 300.0,
            ..Default::default()
        };
        let folder =
            std::env::temp_dir().join(format!("craft-quality-zero-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let jpg = folder.join("zero.jpg");
        let job = prepare_job(&app, &jpg, &state).unwrap();
        assert_eq!(job.state.quality, 0);
        job.run(None, Arc::new(AtomicBool::new(false))).unwrap();
        assert_eq!(image::image_dimensions(&jpg).unwrap(), (5, 4));
        state.format = 2;
        let png = folder.join("after-zero.png");
        let job = prepare_job(&app, &png, &state).unwrap();
        assert_eq!(state.quality, 0);
        assert_eq!(job.state.quality, 1);
        job.run(None, Arc::new(AtomicBool::new(false))).unwrap();
        state.quality = 1;
        let one = folder.join("one.png");
        export(&mut app, &one, &state).unwrap();
        assert_eq!(std::fs::read(&png).unwrap(), std::fs::read(&one).unwrap());
        assert_eq!(image::image_dimensions(&png).unwrap(), (5, 4));
        assert!(
            image::open(&png)
                .unwrap()
                .to_rgb8()
                .pixels()
                .all(|p| p.0 == [17, 84, 203])
        );
        assert_eq!(
            crate::image_dpi::resolution(&std::fs::read(&png).unwrap())
                .map(|(x, y)| (x.round() as u32, y.round() as u32)),
            Some((300, 300))
        );
        state.quality = 255;
        assert!(prepare_job(&app, &folder.join("bad.png"), &state).is_err());
        state.format = 3;
        assert!(prepare_job(&app, &folder.join("bad.jpg"), &state).is_err());
        for path in [jpg, png, one] {
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir(folder).unwrap();
    }
    /// Android (#1398): Send To has no desktop folder there (the row is
    /// greyed with the reason), while Documents and Mail stay; every
    /// destination works elsewhere. The file handed to another app keeps
    /// one name inside its folder, and addresses are told from paths.
    #[test]
    fn android_sends_and_opens_through_the_system() {
        assert_eq!(
            why_not_send_to(SendTarget::Desktop, true),
            Some("menu.file.send_to_desktop_android")
        );
        assert_eq!(why_not_send_to(SendTarget::Documents, true), None);
        assert_eq!(why_not_send_to(SendTarget::Mail, true), None);
        for t in [SendTarget::Desktop, SendTarget::Documents, SendTarget::Mail] {
            assert_eq!(why_not_send_to(t, false), None);
            // Not on Android here.
            assert_eq!(t.why_not(), None);
        }
        assert!(crate::i18n::has_english(
            "menu.file.send_to_desktop_android"
        ));
        assert_eq!(shared_file_name("Poster.pdf"), "Poster.pdf");
        assert_eq!(shared_file_name("a/b\\c.pdf"), "a_b_c.pdf");
        assert_eq!(shared_file_name("../x.pdf"), "_x.pdf");
        assert_eq!(shared_file_name(" .. "), "document");
        for a in [
            "https://tracedraw.dev/help",
            "mailto:someone@example.com",
            "market://details?id=dev.tracedraw.app",
            "content://com.android.providers/document/1",
        ] {
            assert!(is_address(a), "{a}");
        }
        for p in [
            "/data/user/0/dev.tracedraw.app/files/tmp/a.pdf",
            "C:\\Users\\a.pdf",
            "notes.txt",
            "",
        ] {
            assert!(!is_address(p), "{p}");
        }
        // The Print dialog names the destination after the system's
        // service: the PDF viewer here, the print service on Android.
        assert_eq!(crate::ui::dialogs::print_service_key(), "print.dest_viewer");
        for key in [
            "print.dest_android",
            "status.print_sent_android",
            "status.print_service_failed",
            "status.shared",
            "status.share_failed",
            "status.bitmap_sent_to_app",
            "status.no_app_edits_pictures",
            "status.no_folders_on_android",
            "options.bitmap_editor_android",
            "options.open_folder_android",
        ] {
            assert!(crate::i18n::has_english(key), "{key}");
        }
    }

    /// Each print to the viewer gets a name of its own, so the viewer
    /// opens it in a new page while the last one is still shown; older
    /// ones are cleared away (#809).
    #[test]
    fn each_print_to_the_viewer_gets_its_own_file() {
        let dir = std::env::temp_dir().join(format!("tracedraw-viewer-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut paths = Vec::new();
        for count in 1..=3 {
            let p = super::viewer_print_path(&dir, "Poster.pdf", count);
            std::fs::write(&p, b"%PDF").unwrap();
            paths.push(p);
        }
        let names: Vec<_> = paths
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["Poster.pdf", "Poster (2).pdf", "Poster (3).pdf"]);
        assert!(!paths[0].exists(), "the oldest is cleared away");
        assert!(paths[1].exists() && paths[2].exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    use super::*;

    /// A spot colour on a rectangle: printed with separations, the sheets
    /// give the four process plates and one for the spot ink, where the
    /// rectangle is full ink; the print preview offers its box (#779).
    #[test]
    fn separations_print_a_plate_for_each_spot_ink() {
        use tracedraw_core::Fill;
        let mut app = App::headless();
        app.run(tracedraw_core::Command::SetInks {
            inks: vec![tracedraw_core::document::Ink {
                id: 7,
                name: "Spot Red".into(),
                cmyk: [0.0, 1.0, 0.8, 0.0],
            }],
        });
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(20.0, 20.0, 80.0, 60.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.run(tracedraw_core::Command::SetFill {
            shapes: vec![id],
            fill: Fill::Solid(tracedraw_core::Color::Spot {
                ink: 7,
                tint: 1.0,
                c: 0.0,
                m: 1.0,
                y: 0.8,
                k: 0.0,
            }),
        });
        let st = PrintState {
            separations: true,
            ..PrintState::default()
        };
        let sheets = print_sheets(&app, &st);
        let names: Vec<String> = plates(&sheets).into_iter().map(|(_, n)| n).collect();
        assert_eq!(names, ["Cyan", "Magenta", "Yellow", "Black", "Spot Red"]);
        let seps = separations_doc(&sheets);
        assert_eq!(seps.pages.len(), 5);
        // Plates unticked in Print Preview do not print (#846).
        let some = separations_doc_without(&sheets, &[0, 2]);
        let kept: Vec<&str> = some.pages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(kept.len(), 3, "{kept:?}");
        assert!(
            kept[0].ends_with("Magenta") && kept[2].ends_with("Spot Red"),
            "{kept:?}"
        );
        let spot = &seps.pages[4];
        assert!(spot.name.ends_with("Spot Red"), "{}", spot.name);
        // The rectangle is full ink on its plate and absent on cyan.
        let fills = |page: &tracedraw_core::document::Page| -> Vec<Fill> {
            let mut out = Vec::new();
            fn walk(s: &tracedraw_core::Shape, out: &mut Vec<Fill>) {
                if let ShapeKind::Group { children } = &s.kind {
                    children.iter().for_each(|c| walk(c, out));
                } else if let ShapeKind::ClipFrame { contents, .. } = &s.kind {
                    contents.iter().for_each(|c| walk(c, out));
                } else {
                    out.push(s.fill.clone());
                }
            }
            for l in &page.layers {
                l.shapes.iter().for_each(|s| walk(s, &mut out));
            }
            out
        };
        let full = Fill::Solid(tracedraw_core::Color::Gray { v: 0.0 });
        assert!(fills(spot).contains(&full), "{:?}", fills(spot));
        assert!(!fills(&seps.pages[0]).contains(&full));
    }

    /// A Windows print goes at the printer's resolution in strips, blank
    /// ones left out and each cut to its inked part; put back together
    /// they are the sheet drawn whole, pixel for pixel (#1404).
    #[test]
    fn printed_sheet_strips_put_together_are_the_whole_sheet() {
        use tracedraw_core::Fill;
        let mut doc = Document::new("s", tracedraw_core::Size::new(50.0, 40.0));
        let layer = doc.pages[0].layers[0].id;
        let page = doc.pages[0].id;
        for (rect, colour) in [
            (
                tracedraw_core::Rect::new(3.0, 2.0, 21.5, 9.3),
                tracedraw_core::Color::rgb8(200, 0, 0),
            ),
            (
                tracedraw_core::Rect::new(10.2, 25.0, 47.0, 38.0),
                tracedraw_core::Color::rgb8(0, 90, 200),
            ),
        ] {
            let id = doc.ids_mut().shape();
            let mut s = tracedraw_core::Shape::new(id, ShapeKind::Ellipse { rect, arc: None });
            s.fill = Fill::Solid(colour);
            doc.layer_mut(layer).expect("layer").shapes.push(s);
        }
        let dpi = 200.0;
        let whole = tracedraw_render::render_page_image(&doc, page, dpi).expect("whole");
        let (w, h) = (whole.width(), whole.height());
        // Strips of 16 rows: many, some blank, some cut.
        let tiles = sheet_tiles(&doc, page, dpi, w as u64 * 16).expect("tiles");
        assert!(tiles.len() > 4, "{}", tiles.len());
        assert!(
            tiles.len() < h.div_ceil(16) as usize,
            "the blank strips between the two shapes are left out"
        );
        let mut joined = tiny_skia::Pixmap::new(w, h).expect("joined");
        joined.fill(tiny_skia::Color::WHITE);
        let zoom = dpi / 25.4;
        for t in &tiles {
            assert!(t.width < 50.0 - 1.0, "cut to its inked part");
            let pm = tiny_skia::Pixmap::decode_png(&t.png).expect("png");
            assert!(pm.height() <= 16);
            assert!(((t.width * zoom) - pm.width() as f64).abs() < 1e-6);
            assert!(((t.height * zoom) - pm.height() as f64).abs() < 1e-6);
            joined.draw_pixmap(
                (t.x * zoom).round() as i32,
                (t.y * zoom).round() as i32,
                pm.as_ref(),
                &Default::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }
        // The same pixels, but for an edge's smoothing now and then by a
        // step of its sixteen samples (each strip's shapes are placed a
        // whole number of rows apart, which rounds a coordinate the other
        // way at times): no marks where the strips meet, where outlines
        // cut at a strip's edge used to leave caps.
        let differ: Vec<u8> = whole
            .data()
            .iter()
            .zip(joined.data())
            .map(|(a, b)| a.abs_diff(*b))
            .filter(|d| *d > 0)
            .collect();
        let worst = differ.iter().copied().max().unwrap_or(0);
        assert!(worst <= 40, "{worst}");
        assert!(differ.len() < whole.data().len() / 200, "{}", differ.len());
        // A blank sheet sends nothing but the paper.
        let blank = Document::new("b", tracedraw_core::Size::new(50.0, 40.0));
        let tiles = sheet_tiles(&blank, blank.pages[0].id, 300.0, 1 << 24).expect("blank");
        assert!(tiles.is_empty());
    }

    /// The printer's own resolution, kept within reason (#1404).
    #[test]
    fn windows_prints_at_the_printers_resolution() {
        assert_eq!(windows_print_dpi(Some(600.0)), 600.0);
        assert_eq!(windows_print_dpi(Some(1200.0)), 1200.0);
        assert_eq!(windows_print_dpi(Some(2400.0)), 1200.0);
        assert_eq!(windows_print_dpi(Some(150.0)), 300.0);
        assert_eq!(windows_print_dpi(None), 600.0);
        assert_eq!(windows_print_dpi(Some(f64::NAN)), 600.0);
    }

    #[test]
    fn spot_inks_get_plates_of_their_own() {
        use tracedraw_core::{Fill, document::Ink};
        let mut doc = Document::new("s", tracedraw_core::Size::new(50.0, 50.0));
        doc.inks.push(Ink {
            id: 7,
            name: "Gold".into(),
            cmyk: [0.0, 0.2, 0.8, 0.1],
        });
        let layer = doc.pages[0].layers[0].id;
        let id = doc.ids_mut().shape();
        let mut s = tracedraw_core::Shape::new(
            id,
            ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            },
        );
        s.fill = Fill::Solid(doc.inks[0].color(0.6));
        s.stroke = None;
        doc.layer_mut(layer).expect("layer").shapes.push(s);
        let pdf = tracedraw_io::pdf::document_to_pdf(&separations_doc(&doc));
        let text = String::from_utf8_lossy(&pdf);
        // C, M, Y, K and the spot plate.
        assert!(
            text.contains("/Count 5"),
            "{}",
            &text[..text.len().min(400)]
        );
    }

    /// A drawing of one picture of `rgba` pixels (`w` wide), with `inks`
    /// as its ink channels when given, and a spot ink with id 7.
    fn picture_doc(
        w: u32,
        rgba: &[u8],
        inks: Option<(Vec<tracedraw_core::picture_inks::PictureInk>, Vec<u8>)>,
    ) -> Document {
        use tracedraw_core::picture_inks::PictureInks;
        let h = (rgba.len() / 4) as u32 / w;
        let img = image::RgbaImage::from_raw(w, h, rgba.to_vec()).expect("pixels");
        let png: tracedraw_core::SharedBytes = crate::bitmap_fx::encode(&img).expect("png").into();
        let mut doc = Document::new("p", tracedraw_core::Size::new(50.0, 50.0));
        doc.inks.push(tracedraw_core::document::Ink {
            id: 7,
            name: "Gold".into(),
            cmyk: [0.0, 0.2, 0.8, 0.1],
        });
        let id = doc.ids_mut().shape();
        let mut s = tracedraw_core::Shape::new(
            id,
            ShapeKind::Bitmap {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                width_px: w,
                height_px: h,
                png: png.clone(),
                fx: None,
            },
        );
        s.fill = tracedraw_core::Fill::None;
        s.stroke = None;
        if let Some((inks, planes)) = inks {
            let n = inks.len() as u32;
            let gray = image::GrayImage::from_raw(w, h * n, planes).expect("planes");
            let mut out = std::io::Cursor::new(Vec::new());
            gray.write_to(&mut out, image::ImageFormat::Png)
                .expect("png");
            s.cmyk_bitmap = true;
            s.picture_inks = Some(Box::new(PictureInks {
                inks,
                png: out.into_inner().into(),
                of: tracedraw_core::picture_inks::fingerprint(&png),
                profile: None,
            }));
        }
        doc.pages[0].layers[0].shapes.push(s);
        doc
    }

    /// The pixels of the picture on each plate (C, M, Y, K, then the spot
    /// inks), found inside groups and clip frames.
    fn plate_pixels(doc: &Document) -> Vec<Vec<[u8; 4]>> {
        fn find(s: &tracedraw_core::Shape) -> Option<&tracedraw_core::SharedBytes> {
            match &s.kind {
                ShapeKind::Bitmap { png, .. } => Some(png),
                ShapeKind::Group { children } => children.iter().find_map(find),
                ShapeKind::ClipFrame { contents, .. } => contents.iter().find_map(find),
                _ => None,
            }
        }
        separations_doc(doc)
            .pages
            .iter()
            .map(|p| {
                let png = p.layers[0].shapes.iter().find_map(find).expect("picture");
                let img = crate::bitmap_fx::decode(png).expect("decodes");
                img.pixels().map(|p| p.0).collect()
            })
            .collect()
    }

    /// Separations split pictures into their inks (#1659): a CMYK
    /// picture prints each ink it holds on its plate, an RGB or grey one
    /// is converted to CMYK, a spot channel goes to its spot plate, and
    /// the picture's alpha stays on every plate.
    #[test]
    fn separations_split_pictures_into_their_inks() {
        use tracedraw_core::picture_inks::PictureInk;
        let ink = |p: &[u8; 4]| 255 - p[0];
        // A CMYK picture of cyan ink alone (its screen pixels a cyan
        // that converts back to some magenta too): only the cyan plate.
        let process: Vec<PictureInk> = (0..4).map(PictureInk::Process).collect();
        let doc = picture_doc(
            1,
            &[0, 174, 239, 255],
            Some((process.clone(), vec![255, 0, 0, 0])),
        );
        let plates = plate_pixels(&doc);
        assert_eq!(plates.len(), 5, "C, M, Y, K and the spot plate");
        let inks: Vec<u8> = plates.iter().map(|p| ink(&p[0])).collect();
        assert_eq!(inks, [255, 0, 0, 0, 0]);
        // Black ink alone stays on the black plate.
        let doc = picture_doc(1, &[35, 31, 32, 255], Some((process, vec![0, 0, 0, 255])));
        let inks: Vec<u8> = plate_pixels(&doc).iter().map(|p| ink(&p[0])).collect();
        assert_eq!(inks, [0, 0, 0, 255, 0]);
        // RGB red: magenta and yellow.
        let doc = picture_doc(1, &[255, 0, 0, 255], None);
        let inks: Vec<u8> = plate_pixels(&doc).iter().map(|p| ink(&p[0])).collect();
        assert!(inks[0] < 26 && inks[3] < 26 && inks[4] == 0, "{inks:?}");
        assert!(inks[1] > 230 && inks[2] > 230, "{inks:?}");
        // A grey picture lands on the black plate.
        let doc = picture_doc(2, &[128, 128, 128, 255, 0, 0, 0, 255], None);
        let plates = plate_pixels(&doc);
        for (i, plate) in plates.iter().enumerate() {
            let inks: Vec<u8> = plate.iter().map(ink).collect();
            if i == 3 {
                assert!(inks[0] > 100 && inks[1] == 255, "{inks:?}");
            } else {
                assert_eq!(inks, [0, 0], "plate {i}");
            }
        }
        // Transparency: the alpha of every pixel on every plate.
        let doc = picture_doc(3, &[255, 0, 0, 0, 255, 0, 0, 128, 0, 0, 0, 255], None);
        for plate in plate_pixels(&doc) {
            assert_eq!(
                plate.iter().map(|p| p[3]).collect::<Vec<_>>(),
                [0, 128, 255]
            );
        }
        // A spot channel prints on its spot plate only.
        let doc = picture_doc(
            1,
            &[200, 160, 60, 255],
            Some((vec![PictureInk::Spot(7)], vec![204])),
        );
        let inks: Vec<u8> = plate_pixels(&doc).iter().map(|p| ink(&p[0])).collect();
        assert_eq!(inks, [0, 0, 0, 0, 204]);
        // Channels that no longer fit the pixels (the picture changed)
        // give way to the pixels: this red is magenta and yellow.
        let mut doc = picture_doc(
            1,
            &[255, 0, 0, 255],
            Some((
                (0..4).map(PictureInk::Process).collect(),
                vec![255, 0, 0, 0],
            )),
        );
        if let Some(inks) = &mut doc.pages[0].layers[0].shapes[0].picture_inks {
            inks.of ^= 1;
        }
        let inks: Vec<u8> = plate_pixels(&doc).iter().map(|p| ink(&p[0])).collect();
        assert!(inks[0] < 26 && inks[1] > 230, "{inks:?}");
        // Inside a clip frame (clipped, as it shows) the picture splits too.
        let mut doc = picture_doc(1, &[255, 0, 0, 255], None);
        let pic = doc.pages[0].layers[0].shapes.remove(0);
        let id = doc.ids_mut().shape();
        let frame = tracedraw_core::Shape::new(
            id,
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 5.0, 5.0),
                radius: 0.0,
                corners: None,
            },
        );
        let id = doc.ids_mut().shape();
        doc.pages[0].layers[0]
            .shapes
            .push(tracedraw_core::Shape::new(
                id,
                ShapeKind::ClipFrame {
                    frame: Box::new(frame),
                    contents: vec![pic],
                },
            ));
        let inks: Vec<u8> = plate_pixels(&doc).iter().map(|p| ink(&p[0])).collect();
        assert!(inks[0] < 26 && inks[2] > 230, "{inks:?}");
    }

    /// File > Export of the current page, every page or a list of pages:
    /// one numbered file per page for the formats of one page, one file
    /// with every page chosen for PDF, its text as curves when asked
    /// (#1382).
    #[test]
    fn export_writes_the_pages_chosen() {
        let mut app = App::headless();
        // Three pages, each with a rectangle of its own width.
        for k in 0..3 {
            if k > 0 {
                app.add_page();
            }
            app.new_shape(ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 20.0 + 10.0 * k as f64, 30.0),
                radius: 0.0,
                corners: None,
            });
        }
        let ids: Vec<_> = app.doc().pages.iter().map(|p| p.id).collect();
        app.page = ids[1];
        let dir = std::env::temp_dir().join(format!("tracedraw-1382-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let files = |dir: &Path| {
            let mut names: Vec<String> = std::fs::read_dir(dir)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
                .collect();
            names.sort();
            names
        };
        let png = |range: (bool, bool, &str)| ExportState {
            format: 2,
            dpi: 25.4,
            all_pages: range.0,
            page_list: range.1,
            pages: range.2.into(),
            transparent: false,
            ..ExportState::default()
        };
        // The current page: one file under the name given.
        let one = dir.join("one");
        std::fs::create_dir_all(&one).unwrap();
        export(&mut app, &one.join("a.png"), &png((false, false, ""))).expect("current");
        assert_eq!(files(&one), ["a.png"]);
        let img = image::open(one.join("a.png")).unwrap();
        assert_eq!(img.width(), 210, "page 2, 25.4 dpi: one pixel per mm");
        // Every page: numbered files.
        let all = dir.join("all");
        std::fs::create_dir_all(&all).unwrap();
        let msg = export(&mut app, &all.join("a.png"), &png((true, false, ""))).expect("all");
        assert_eq!(files(&all), ["a-1.png", "a-2.png", "a-3.png"]);
        assert!(msg.contains('3'), "{msg}");
        // A list: the pages listed, numbered by their page.
        let list = dir.join("list");
        std::fs::create_dir_all(&list).unwrap();
        export(&mut app, &list.join("a.png"), &png((true, true, "3, 1"))).expect("list");
        assert_eq!(files(&list), ["a-1.png", "a-3.png"]);
        // Page 3's rectangle is 40 mm wide, page 1's 20 mm: each file
        // shows its own page.
        let dark = |p: &Path| {
            let img = image::open(p).unwrap().to_rgb8();
            let y = img.height() - 20;
            let inked: Vec<u32> = (0..img.width())
                .filter(|x| img.get_pixel(*x, y).0 != [255, 255, 255])
                .collect();
            inked.last().unwrap_or(&0) - inked.first().unwrap_or(&0)
        };
        assert!(dark(&list.join("a-3.png")) > dark(&list.join("a-1.png")) + 10);
        // SVG too writes one file per page.
        let svg = dir.join("svg");
        std::fs::create_dir_all(&svg).unwrap();
        let st = ExportState {
            format: 0,
            ..png((true, true, "2-3"))
        };
        export(&mut app, &svg.join("b.svg"), &st).expect("svg");
        assert_eq!(files(&svg), ["b-2.svg", "b-3.svg"]);
        // PDF: one file holding the pages chosen.
        let pdf = dir.join("pdf");
        std::fs::create_dir_all(&pdf).unwrap();
        let st = ExportState {
            format: 1,
            ..png((true, true, "1, 3"))
        };
        export(&mut app, &pdf.join("c.pdf"), &st).expect("pdf");
        assert_eq!(files(&pdf), ["c.pdf"]);
        let text = String::from_utf8_lossy(&std::fs::read(pdf.join("c.pdf")).unwrap()).to_string();
        assert!(text.contains("/Count 2"), "two pages");
        let st = ExportState {
            format: 1,
            ..png((true, false, ""))
        };
        export(&mut app, &pdf.join("c.pdf"), &st).expect("pdf");
        let text = String::from_utf8_lossy(&std::fs::read(pdf.join("c.pdf")).unwrap()).to_string();
        assert!(text.contains("/Count 3"), "every page");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Facing pages (#1972): Export > Spreads makes each spread one page
    /// as wide as both, page 1 alone; every page on its own carries
    /// what its facing page holds across the spine, in exports and in
    /// print; a PDF of the spreads has a page per spread.
    #[test]
    fn facing_pages_export_spreads_or_pages_with_the_spine_crossing() {
        let mut app = App::headless();
        for _ in 0..2 {
            app.add_page();
        }
        app.page = app.doc().pages[1].id;
        let size = app.doc().pages[1].size;
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(size.width - 20.0, 10.0, size.width + 30.0, 40.0),
            radius: 0.0,
            corners: None,
        });
        let st = ExportState {
            format: 1,
            all_pages: true,
            ..ExportState::default()
        };
        let objects = |d: &Document, k: usize| -> usize {
            d.pages[k].layers.iter().map(|l| l.shapes.len()).sum()
        };
        // Facing pages off: as before.
        let (plain, _) = export_document(&app, &st);
        assert_eq!(plain.pages.len(), 3);
        assert_eq!(objects(&plain, 2), 0);
        app.set_facing_pages(true, tracedraw_core::spreads::PageSide::Right);
        // Every page alone: page 3 carries the rectangle's part on it.
        let (pages, _) = export_document(&app, &st);
        assert_eq!(pages.pages.len(), 3);
        assert_eq!(objects(&pages, 2), 1);
        assert_eq!(objects(&pages, 0), 0);
        let printed = print_sheets(&app, &PrintState::default());
        assert_eq!(printed.pages.len(), 3);
        // Spreads: page 1, then pages 2-3 as one page twice as wide.
        let spreads = ExportState {
            spreads: true,
            ..st.clone()
        };
        let (doc, _) = export_document(&app, &spreads);
        assert_eq!(doc.pages.len(), 2);
        assert_eq!(doc.pages[0].size, size);
        assert!((doc.pages[1].size.width - 2.0 * size.width).abs() < 1e-9);
        assert_eq!(objects(&doc, 1), 1);
        let dir = std::env::temp_dir().join(format!("td-spreads-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s.pdf");
        export(&mut app, &path, &spreads).expect("pdf");
        let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).to_string();
        assert!(text.contains("/Count 2"), "a page per spread");
        // One file per spread for a format of one page.
        let png = ExportState {
            format: 2,
            dpi: 20.0,
            ..spreads.clone()
        };
        export(&mut app, &dir.join("s.png"), &png).expect("png");
        assert!(dir.join("s-1.png").exists() && dir.join("s-2.png").exists());
        assert!(!dir.join("s-3.png").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The pages an export takes: the current one, every page, or the
    /// list in its order (every page when it names none); the selection
    /// comes from the current page (#1382).
    #[test]
    fn export_page_choices_pick_pages() {
        let mut st = ExportState::default();
        assert_eq!(st.page_indexes(4, 2), vec![2]);
        st.all_pages = true;
        assert_eq!(st.page_indexes(4, 2), vec![0, 1, 2, 3]);
        st.page_list = true;
        st.pages = "4, 1-2".into();
        assert_eq!(st.page_indexes(4, 2), vec![3, 0, 1]);
        st.pages = "9".into();
        assert_eq!(st.page_indexes(4, 2), vec![0, 1, 2, 3]);
        st.selection_only = true;
        assert_eq!(st.page_indexes(4, 2), vec![2]);
        assert!(ExportState::one_file_for_all_pages("pdf"));
        assert!(!ExportState::one_file_for_all_pages("svg"));
        assert!(!ExportState::one_file_for_all_pages("png"));
        assert_eq!(
            numbered_path(Path::new("/x/name.jpeg"), 3, "jpg"),
            Path::new("/x/name-3.jpeg")
        );
        assert_eq!(
            numbered_path(Path::new("/x/name"), 2, "png"),
            Path::new("/x/name-2.png")
        );
    }

    /// "AI" writes the PDF-compatible file by default, and the classic
    /// version 8 file (PostScript, one page per file) when chosen, for
    /// programs that read only those (#1728).
    #[test]
    fn export_ai_writes_pdf_compatible_or_classic_version_8() {
        let mut app = App::headless();
        for k in 0..2 {
            if k > 0 {
                app.add_page();
            }
            app.new_shape(ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 40.0, 30.0),
                radius: 0.0,
                corners: None,
            });
        }
        let dir = std::env::temp_dir().join(format!("tracedraw-1728-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let ai = EXPORT_FORMATS.iter().position(|f| f.1 == "ai").unwrap();
        let st = |version: usize| ExportState {
            format: ai,
            all_pages: true,
            ai_version: version,
            title: "Logo (curves)".into(),
            ..ExportState::default()
        };
        // The default: one PDF-compatible file with both pages, as before.
        assert!(!ExportState::default().classic_ai());
        export(&mut app, &dir.join("pdf.ai"), &st(0)).expect("pdf-compatible");
        let bytes = std::fs::read(dir.join("pdf.ai")).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        // Classic: one PostScript file per page, each with the AI header.
        assert!(st(1).classic_ai() && !st(1).one_file_holds_the_pages("ai"));
        export(&mut app, &dir.join("old.ai"), &st(1)).expect("classic");
        for name in ["old-1.ai", "old-2.ai"] {
            let text = std::fs::read_to_string(dir.join(name)).expect(name);
            assert!(text.starts_with("%!PS-Adobe-3.0\n"), "{name}");
            assert!(text.contains("\n%%AI8_CreatorVersion: 8.0\n"));
            assert!(text.contains("\n%AI5_FileFormat 4.0\n"));
            assert!(text.contains("\n%%BoundingBox: 0 0 596 842\n"));
            assert!(text.contains("\n%%Title: (Logo \\(curves\\))\n"), "{text}");
        }
        // It opens again as a PostScript-based .ai, with the rectangle.
        assert_eq!(App::file_kind(&dir.join("old-1.ai")), "eps");
        let mut ids = tracedraw_core::id::IdSource::default();
        let bytes = std::fs::read(dir.join("old-1.ai")).unwrap();
        let back = tracedraw_io::eps_import::parse(&bytes, &mut ids).expect("reads");
        assert!(!back.shapes.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Export to PDF with text as curves holds no font (#1382).
    #[test]
    fn export_pdf_text_as_curves_holds_no_font() {
        tracedraw_text::install();
        let fonts = tracedraw_text::fonts();
        let Some(family) = ["DejaVu Sans", "Liberation Sans", "Noto Sans"]
            .into_iter()
            .find(|f| fonts.has_family(f))
        else {
            return;
        };
        let mut app = App::headless();
        app.new_shape(ShapeKind::Text {
            spans: vec![TextSpan::new("Searchable words", family, 24.0)],
            origin: tracedraw_core::Point::new(20.0, 50.0),
            frame: None,
            align: Default::default(),
            para: Default::default(),
            on_path: None,
        });
        let dir = std::env::temp_dir().join(format!("tracedraw-1382c-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.pdf");
        let mut st = ExportState {
            format: 1,
            ..ExportState::default()
        };
        export(&mut app, &path, &st).expect("pdf");
        let live = std::fs::read(&path).unwrap();
        st.text_as_curves = true;
        export(&mut app, &path, &st).expect("pdf");
        let curves = std::fs::read(&path).unwrap();
        let words = |bytes: &[u8]| {
            lopdf::Document::load_mem(bytes)
                .ok()
                .and_then(|d| d.extract_text(&[1]).ok())
                .unwrap_or_default()
        };
        assert!(String::from_utf8_lossy(&live).contains("/FontDescriptor"));
        assert!(words(&live).contains("Searchable"), "live text");
        assert!(
            !String::from_utf8_lossy(&curves).contains("/Font"),
            "no font"
        );
        assert!(!words(&curves).contains("Searchable"), "curves");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_export_preview_shows_the_page_and_estimates_the_size() {
        let mut app = App::headless();
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(10.0, 10.0, 100.0, 60.0),
            radius: 0.0,
            corners: None,
        });
        // PNG at 300 dpi: an A4 preview keeps the page's proportions.
        let st = ExportState {
            format: 2,
            dpi: 300.0,
            ..ExportState::default()
        };
        let pm = preview(&app, &st, 200).expect("preview");
        assert_eq!(pm.height(), 200);
        assert!((pm.width() as f64 - 200.0 * 210.0 / 297.0).abs() <= 1.0);
        let png = estimated_size(&app, &st, &pm).expect("estimate");
        assert!(png > 0);
        // The same page at 72 dpi is estimated smaller.
        let small = ExportState {
            dpi: 72.0,
            ..st.clone()
        };
        assert!(estimated_size(&app, &small, &pm).unwrap_or(usize::MAX) < png);
        // A vector format is measured, not estimated.
        let svg = ExportState {
            format: 0,
            ..st.clone()
        };
        assert!(estimated_size(&app, &svg, &pm).is_some());
        assert!(keeps_alpha("png") && !keeps_alpha("jpg"));
    }

    /// A bitmap export records the resolution of the dialog, and the
    /// scale changes how many pixels it has but not that resolution, so it
    /// opens at the size chosen (#617).
    #[test]
    fn bitmap_exports_record_their_resolution_and_scale() {
        let mut app = App::headless();
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(10.0, 10.0, 100.0, 60.0),
            radius: 0.0,
            corners: None,
        });
        let b = app.page_rect();
        let dir = std::env::temp_dir().join(format!("tracedraw-617-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (format, ext) in [(2, "png"), (3, "jpg"), (4, "bmp")] {
            let st = ExportState {
                format,
                dpi: 50.0,
                scale: 2.0,
                transparent: false,
                ..ExportState::default()
            };
            let path = dir.join(format!("out.{ext}"));
            export(&mut app, &path, &st).expect(ext);
            let bytes = std::fs::read(&path).unwrap();
            let (x, y) = crate::image_dpi::resolution(&bytes).expect(ext);
            assert!(
                (x - 50.0).abs() < 0.1 && (y - 50.0).abs() < 0.1,
                "{ext}: {x} {y}"
            );
            // Still a file every reader opens, twice the pixels of 50 dpi.
            let img = image::load_from_memory(&bytes).expect(ext);
            let want = crate::ui::dialogs::export_pixels(b.width(), b.height(), 100.0);
            assert_eq!((img.width(), img.height()), want, "{ext}");
        }
        let _ = std::fs::remove_dir_all(&dir);
        // Files not laid out as expected come back as they were.
        assert_eq!(record_dpi(vec![1, 2, 3], "png", 300.0), vec![1, 2, 3]);
        assert_eq!(record_dpi(b"BM".to_vec(), "bmp", 300.0), b"BM".to_vec());
        assert_eq!(
            record_dpi(vec![0xFF, 0xD8], "jpg", f64::NAN),
            vec![0xFF, 0xD8]
        );
    }

    /// The pixel at `(x, y)` of an uncompressed chunky TIFF, read with
    /// the fields of TIFF 6.0 (image width, strip offsets, rows per
    /// strip, samples per pixel); also the photometric interpretation.
    fn tiff_pixel(b: &[u8], x: usize, y: usize) -> (Vec<u8>, u32) {
        let le16 = |at: usize| u16::from_le_bytes([b[at], b[at + 1]]) as u32;
        let le32 = |at: usize| u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]]);
        assert_eq!(&b[..4], b"II*\0");
        let ifd = le32(4) as usize;
        let field = |tag: u32| -> Vec<u32> {
            for i in 0..le16(ifd) as usize {
                let e = ifd + 2 + 12 * i;
                if le16(e) == tag {
                    let (kind, n) = (le16(e + 2), le32(e + 4) as usize);
                    let size = if kind == 3 { 2 } else { 4 };
                    let at = if size * n <= 4 {
                        e + 8
                    } else {
                        le32(e + 8) as usize
                    };
                    return (0..n)
                        .map(|k| {
                            if size == 2 {
                                le16(at + 2 * k)
                            } else {
                                le32(at + 4 * k)
                            }
                        })
                        .collect();
                }
            }
            panic!("no tag {tag}");
        };
        let (w, spp, rows) = (field(256)[0], field(277)[0], field(278)[0]);
        let offsets = field(273);
        let row = (w * spp) as usize;
        let strip = y / rows as usize;
        let at = offsets[strip] as usize + (y % rows as usize) * row + x * spp as usize;
        (b[at..at + spp as usize].to_vec(), field(262)[0])
    }

    /// File > Export to JPEG or TIFF in CMYK (#1763): a rectangle filled
    /// with pure cyan comes out as cyan ink alone, the paper as no ink,
    /// an RGB red through the colour management as magenta and yellow;
    /// RGB exports are as before, Grayscale writes one channel, and the
    /// drawing's CMYK profile goes into the files.
    #[test]
    fn cmyk_bitmap_exports_keep_the_inks() {
        use tracedraw_core::{Color, Fill};
        let mut app = App::headless();
        for (rect, color) in [
            (
                Rect::new(10.0, 10.0, 100.0, 100.0),
                Color::cmyk8(255, 0, 0, 0),
            ),
            (Rect::new(110.0, 10.0, 200.0, 100.0), Color::rgb8(255, 0, 0)),
        ] {
            app.new_shape_styled(
                ShapeKind::Rect {
                    rect,
                    radius: 0.0,
                    corners: None,
                },
                |s| {
                    s.fill = Fill::Solid(color);
                    s.stroke = None;
                },
            );
        }
        let mut doc = app.doc().clone();
        let profile = tracedraw_core::icc::test_profiles::lut16_cmyk_profile();
        doc.cmyk_profile = Some(tracedraw_core::DocumentProfile::new(
            "Press",
            profile.clone(),
        ));
        app.engine.replace(doc);
        let dir = std::env::temp_dir().join(format!("tracedraw-1763-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let st = |format: usize, color_mode: usize| ExportState {
            format,
            dpi: 25.4,
            quality: 95,
            color_mode,
            transparent: false,
            ..ExportState::default()
        };
        // Page coordinates run up from the bottom: the rectangles lie
        // near the bottom of the 297 mm page, one pixel per mm.
        let (cyan_at, red_at, paper_at) = ((55, 297 - 55), (155, 297 - 55), (55, 150));

        // CMYK JPEG, read by the decoder Craft Plumilla opens files with.
        let jpg = dir.join("cmyk.jpg");
        export(&mut app, &jpg, &st(3, 1)).expect("cmyk jpeg");
        let bytes = std::fs::read(&jpg).unwrap();
        assert!(bytes.windows(5).any(|w| w == b"Adobe"), "Adobe marker");
        let sof = bytes
            .windows(2)
            .position(|w| w == [0xFF, 0xC0])
            .expect("SOF0");
        assert_eq!(bytes[sof + 9], 4, "four components");
        let icc_at = bytes
            .windows(12)
            .position(|w| w == b"ICC_PROFILE\0")
            .expect("profile embedded");
        assert_eq!(&bytes[icc_at + 14..icc_at + 14 + 64], &profile[..64]);
        assert_eq!(
            crate::image_dpi::resolution(&bytes).map(|(x, _)| x.round()),
            Some(25.0)
        );
        let img = image::load_from_memory(&bytes).expect("decodes").to_rgb8();
        assert_eq!((img.width(), img.height()), (210, 297));
        // The decoder shows (255 - ink) x (255 - black): only no red, full
        // green and blue is cyan 100 with magenta, yellow and black 0.
        let p = img.get_pixel(cyan_at.0, cyan_at.1).0;
        assert!(p[0] < 6 && p[1] > 249 && p[2] > 249, "cyan: {p:?}");
        let p = img.get_pixel(paper_at.0, paper_at.1).0;
        assert!(p.iter().all(|v| *v > 249), "paper: {p:?}");

        // CMYK TIFF: the inks as they are.
        let tif = dir.join("cmyk.tif");
        export(&mut app, &tif, &st(5, 1)).expect("cmyk tiff");
        let bytes = std::fs::read(&tif).unwrap();
        let (cyan, photometric) = tiff_pixel(&bytes, cyan_at.0 as usize, cyan_at.1 as usize);
        assert_eq!(photometric, 5, "separated");
        assert!(
            cyan[0] > 252 && cyan[1..].iter().all(|v| *v < 3),
            "{cyan:?}"
        );
        let (paper, _) = tiff_pixel(&bytes, paper_at.0 as usize, paper_at.1 as usize);
        assert_eq!(paper, [0, 0, 0, 0]);
        let (red, _) = tiff_pixel(&bytes, red_at.0 as usize, red_at.1 as usize);
        let want = match Color::rgb8(255, 0, 0).convert_to("CMYK") {
            Color::Cmyk { c, m, y, k } => [c, m, y, k].map(|v| (v * 255.0).round() as i32),
            other => panic!("{other:?}"),
        };
        for (got, want) in red.iter().zip(want) {
            assert!((*got as i32 - want).abs() <= 2, "red: {red:?} {want:?}");
        }
        assert!(
            red[1] > 200 && red[2] > 200,
            "red prints magenta and yellow"
        );
        assert!(bytes.windows(64).any(|w| w == &profile[..64]), "profile");
        assert_eq!(
            crate::image_dpi::resolution(&bytes).map(|(x, _)| x.round()),
            Some(25.0)
        );

        // RGB, the default: JFIF with three components, as before.
        assert_eq!(ExportState::default().color_mode, 0);
        let rgb = dir.join("rgb.jpg");
        export(&mut app, &rgb, &st(3, 0)).expect("rgb jpeg");
        let bytes = std::fs::read(&rgb).unwrap();
        assert_eq!(&bytes[6..11], b"JFIF\0");
        assert!(!bytes.windows(5).any(|w| w == b"Adobe"));
        let img = image::load_from_memory(&bytes).unwrap();
        assert_eq!(img.color(), image::ColorType::Rgb8);
        let p = img.to_rgb8().get_pixel(cyan_at.0, cyan_at.1).0;
        // This document explicitly carries Synthetic CMYK. Independent
        // Pillow/LCMS2.19 conversion of C=255 gives [80,205,183] for all
        // four intents and either BPC setting; a generic profile expectation
        // would incorrectly require the worker to ignore the document ICC.
        let want = [80, 205, 183];
        for (a, b) in p.iter().zip(want) {
            assert!((*a as i32 - b as i32).abs() < 6, "{p:?} {want:?}");
        }
        let rgb_tif = dir.join("rgb.tif");
        export(&mut app, &rgb_tif, &st(5, 0)).expect("rgb tiff");
        let (_, photometric) = tiff_pixel(&std::fs::read(&rgb_tif).unwrap(), 0, 0);
        assert_eq!(photometric, 2, "RGB TIFF as before");

        // Grayscale: one channel, the colour's grey.
        for (format, ext) in [(3, "jpg"), (5, "tif")] {
            let path = dir.join(format!("gray.{ext}"));
            export(&mut app, &path, &st(format, 2)).expect(ext);
            let img = image::open(&path).unwrap();
            assert_eq!(img.color(), image::ColorType::L8, "{ext}");
            let v = img.to_luma8().get_pixel(cyan_at.0, cyan_at.1).0[0];
            let want = Color::rgb8(80, 205, 183).convert_to("Grayscale");
            let want = match want {
                Color::Gray { v } => (v * 255.0).round() as i32,
                other => panic!("{other:?}"),
            };
            assert!((v as i32 - want).abs() < 6, "{ext}: {v} {want}");
        }
        // Other formats ignore the colour mode.
        let png = dir.join("x.png");
        export(&mut app, &png, &st(2, 1)).expect("png");
        assert!(matches!(
            image::open(&png).unwrap().color(),
            image::ColorType::Rgb8 | image::ColorType::Rgba8
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The bitmap size in the Export dialog: in pixels it sets the
    /// resolution as before, in a length it sets the scale; the other side
    /// follows and nonsense is ignored (#617).
    #[test]
    fn the_export_size_is_typed_in_pixels_or_lengths() {
        use crate::ui::dialogs::{export_size_in_unit, set_export_size};
        let mut st = ExportState {
            dpi: 100.0,
            size_unit: 3,
            ..ExportState::default()
        };
        let near =
            |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6;
        assert!(near(export_size_in_unit(254.0, 127.0, &st), (10.0, 5.0)));
        set_export_size(&mut st, 254.0, 20.0);
        assert!((st.scale - 2.0).abs() < 1e-9 && st.dpi == 100.0);
        assert!(near(export_size_in_unit(254.0, 127.0, &st), (20.0, 10.0)));
        assert!((st.render_dpi() - 200.0).abs() < 1e-9);
        st.size_unit = 1;
        assert!(near(export_size_in_unit(254.0, 127.0, &st), (508.0, 254.0)));
        st.size_unit = 0;
        assert!(near(
            export_size_in_unit(254.0, 127.0, &st),
            (2000.0, 1000.0)
        ));
        set_export_size(&mut st, 254.0, 1000.0);
        assert!((st.dpi - 50.0).abs() < 1e-3 && (st.scale - 2.0).abs() < 1e-9);
        assert!(near(
            export_size_in_unit(254.0, 127.0, &st),
            (1000.0, 500.0)
        ));
        // Never past the highest resolution, and nonsense changes nothing.
        st.size_unit = 3;
        set_export_size(&mut st, 254.0, 1.0e9);
        assert!(st.render_dpi() <= 2400.0 + 1e-6);
        let before = st.clone();
        for typed in [0.0, -5.0, f64::NAN, f64::INFINITY] {
            set_export_size(&mut st, 254.0, typed);
        }
        set_export_size(&mut st, 0.0, 10.0);
        assert_eq!(st, before);
        st.scale = f64::NAN;
        assert_eq!(st.scale_or_one(), 1.0);
    }

    #[test]
    fn colours_are_mapped_inside_groups_frames_and_text() {
        use tracedraw_core::{Color, Fill, Rect, Shape, ShapeId, document::TextSpan};
        let red = Color::Rgb {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        };
        let mut inner = Shape::new(
            ShapeId(2),
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                radius: 0.0,
                corners: None,
            },
        );
        inner.fill = Fill::Solid(red);
        let mut span = TextSpan::new("a", "Inter", 10.0);
        span.fill = Some(Fill::Solid(red));
        let text = Shape::new(
            ShapeId(3),
            ShapeKind::Text {
                spans: vec![span],
                origin: tracedraw_core::Point::ZERO,
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            },
        );
        let group = Shape::new(
            ShapeId(4),
            ShapeKind::Group {
                children: vec![inner, text],
            },
        );
        let frame = Shape::new(
            ShapeId(5),
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 2.0, 2.0),
                radius: 0.0,
                corners: None,
            },
        );
        let mut clip = Shape::new(
            ShapeId(1),
            ShapeKind::ClipFrame {
                frame: Box::new(frame),
                contents: vec![group],
            },
        );
        let green = Color::Gray { v: 0.5 };
        map_colors(&mut clip, &|_| green);
        let ShapeKind::ClipFrame { contents, frame } = &clip.kind else {
            panic!("clip frame");
        };
        assert_eq!(frame.stroke.as_ref().map(|s| s.color), Some(green));
        let ShapeKind::Group { children } = &contents[0].kind else {
            panic!("group");
        };
        assert_eq!(children[0].fill, Fill::Solid(green));
        let ShapeKind::Text { spans, .. } = &children[1].kind else {
            panic!("text");
        };
        assert_eq!(spans[0].fill, Some(Fill::Solid(green)));
    }

    #[test]
    fn page_lists_and_ranges_pick_pages() {
        use crate::ui::dialogs::parse_page_list;
        assert_eq!(parse_page_list("1-3, 5", 6), vec![0, 1, 2, 4]);
        assert_eq!(parse_page_list("4-", 5), vec![3, 4]);
        assert_eq!(parse_page_list("9, x, 2", 3), vec![1]);
        let mut st = PrintState::default();
        assert_eq!(st.page_indexes(3, 1), vec![0, 1, 2]);
        st.range = 1;
        assert_eq!(st.page_indexes(3, 1), vec![1]);
        st.range = 2;
        st.pages = "3".into();
        assert_eq!(st.page_indexes(3, 0), vec![2]);
        // Odd or even pages of the document or the list (#951); the
        // current page stays, and a choice leaving no page is let go.
        st.parity = 1;
        assert_eq!(st.page_indexes(3, 0), vec![2]);
        st.pages = "1-6".into();
        assert_eq!(st.page_indexes(6, 0), vec![0, 2, 4]);
        st.parity = 2;
        assert_eq!(st.page_indexes(6, 0), vec![1, 3, 5]);
        st.pages = "3".into();
        assert_eq!(st.page_indexes(6, 0), vec![2], "nothing left: as listed");
        st.range = 0;
        assert_eq!(st.page_indexes(5, 0), vec![1, 3]);
        st.range = 1;
        assert_eq!(st.page_indexes(5, 0), vec![0]);
    }

    /// A drop shadow prints: the print PDF holds its picture with its soft
    /// mask, as the exported PDF does (#1029).
    #[test]
    fn a_drop_shadow_prints() {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(20.0, 20.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        let pdf_text = |app: &App| {
            let bytes =
                tracedraw_io::pdf::document_to_pdf(&print_sheets(app, &PrintState::default()));
            String::from_utf8_lossy(&bytes).into_owned()
        };
        assert!(!pdf_text(&app).contains("/SMask"));
        app.run(tracedraw_core::Command::SetShadow {
            shapes: vec![id],
            shadow: Some(tracedraw_core::Shadow::default()),
        });
        let text = pdf_text(&app);
        assert!(text.contains("/Subtype /Image"), "the shadow's picture");
        assert!(text.contains("/SMask"), "with its transparency");
        assert!(text.contains("/BM /Multiply"), "multiplied");
    }

    /// Black text prints black, and a negative (Invert) prints it as the
    /// clear part of a black sheet rather than white on white paper.
    #[test]
    fn black_text_prints_black_and_a_negative_blackens_the_sheet() {
        use tracedraw_core::{Color, Fill};
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Text {
                spans: vec![TextSpan::new("Card", "Sans", 24.0)],
                origin: tracedraw_core::geometry::Point::new(20.0, 100.0),
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            })
            .expect("text");
        let k100 = Color::cmyk_pct(0.0, 0.0, 0.0, 100.0);
        app.run(tracedraw_core::Command::SetFill {
            shapes: vec![id],
            fill: Fill::Solid(k100),
        });
        // The sheet may hold the page in groups or clip frames.
        fn texts(s: &tracedraw_core::Shape, out: &mut Vec<Fill>) {
            match &s.kind {
                ShapeKind::Text { .. } => out.push(s.fill.clone()),
                ShapeKind::Group { children } => children.iter().for_each(|c| texts(c, out)),
                ShapeKind::ClipFrame { frame, contents } => {
                    texts(frame, out);
                    contents.iter().for_each(|c| texts(c, out));
                }
                _ => {}
            }
        }
        let text_fill = |doc: &Document| -> Vec<Fill> {
            let mut out = Vec::new();
            for l in doc.all_layers() {
                l.shapes.iter().for_each(|s| texts(s, &mut out));
            }
            out
        };
        let st = PrintState::default();
        let sheets = print_sheets(&app, &st);
        assert_eq!(text_fill(&sheets), vec![Fill::Solid(k100)]);
        // The PDF as text with its zlib streams inflated.
        let bytes = tracedraw_io::pdf::document_to_pdf(&sheets);
        let mut pdf = String::from_utf8_lossy(&bytes).into_owned();
        let mut rest: &[u8] = &bytes;
        while let Some(i) = rest.windows(7).position(|w| w == b"stream\n") {
            let data = &rest[i + 7..];
            use std::io::Read;
            let mut out = Vec::new();
            if flate2::read::ZlibDecoder::new(data)
                .read_to_end(&mut out)
                .is_ok()
            {
                pdf.push_str(&String::from_utf8_lossy(&out));
            }
            rest = data;
        }
        assert!(pdf.contains("0 0 0 1 k"), "black ink in the PDF");
        let st = PrintState {
            invert: true,
            ..PrintState::default()
        };
        let sheets = print_sheets(&app, &st);
        match text_fill(&sheets).as_slice() {
            [Fill::Solid(c)] => assert!(c.to_rgb8().iter().all(|v| *v > 180), "{c:?}"),
            other => panic!("{other:?}"),
        }
        let first = &sheets.pages[0].layers[0].shapes[0];
        assert_eq!(first.fill, Fill::Solid(Color::BLACK));
        let b = first.bounds();
        let size = sheets.pages[0].size;
        assert!(b.x0 <= 0.0 && b.y0 <= 0.0 && b.x1 >= size.width && b.y1 >= size.height);
    }

    #[test]
    fn print_colour_options_change_the_sheets_objects() {
        use tracedraw_core::{Color, Fill};
        let spot = Color::Spot {
            ink: 9,
            tint: 0.5,
            c: 1.0,
            m: 0.0,
            y: 0.0,
            k: 0.0,
        };
        assert_eq!(
            spot_as_cmyk(spot),
            Color::Cmyk {
                c: 0.5,
                m: 0.0,
                y: 0.0,
                k: 0.0
            }
        );
        let mut doc = Document::new("t", Size::new(50.0, 50.0));
        let id = doc.ids_mut().shape();
        let mut text = tracedraw_core::Shape::new(
            id,
            ShapeKind::Text {
                spans: vec![TextSpan::new("a", "Arial", 12.0)],
                origin: tracedraw_core::geometry::Point::ZERO,
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            },
        );
        text.fill = Fill::Solid(Color::cmyk_pct(0.0, 100.0, 0.0, 0.0));
        text_in_black(&mut text);
        assert_eq!(text.fill, Fill::Solid(Color::BLACK));
        let mut black = text.clone();
        black.fill = Fill::Solid(Color::cmyk_pct(0.0, 0.0, 0.0, 100.0));
        overprint_black(&mut black);
        assert!(black.overprint_fill);
        let mut red = text;
        red.fill = Fill::Solid(Color::cmyk_pct(0.0, 100.0, 100.0, 0.0));
        overprint_black(&mut red);
        assert!(!red.overprint_fill);
    }

    #[test]
    fn copies_repeat_sheets_collated_or_not_and_ps_gets_its_level() {
        let mut doc = Document::new("t", Size::new(50.0, 50.0));
        let mut ids = doc.ids().clone();
        let mut second = doc.pages[0].clone();
        second.id = ids.page();
        second.name = "B".into();
        doc.set_ids(ids);
        doc.pages[0].name = "A".into();
        doc.pages.push(second);
        let mut a = doc.clone();
        repeat_copies(&mut a, 2, true);
        let names: Vec<_> = a.pages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["A", "B", "A", "B"]);
        repeat_copies(&mut doc, 2, false);
        let names: Vec<_> = doc.pages.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["A", "A", "B", "B"]);
        let ids: std::collections::BTreeSet<_> = doc.pages.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), 4);
        let ps = postscript_level("%!PS-Adobe-3.0\n%%EOF\n".into(), 3);
        assert!(ps.starts_with("%!PS-Adobe-3.0\n%%LanguageLevel: 3\n"));
    }

    /// Grayscale output (#1657): every colour on the sheets, an effect's
    /// too, prints as its grey and a picture as grey pixels; the drawing
    /// keeps its colours. Reverse order (#1394, #1571): the last sheet
    /// comes first.
    #[test]
    fn grayscale_and_reverse_order_change_the_printed_sheets() {
        use tracedraw_core::{Color, Fill};
        let mut app = App::headless();
        let size = app.doc().pages[0].size;
        let red = Color::rgb8(220, 20, 20);
        app.new_shape_styled(
            ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(10.0, 10.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            },
            |s| {
                s.name = Some("first-page".into());
                s.fill = Fill::Solid(red);
                s.effects.push(tracedraw_core::live::Effect::BlockShadow {
                    offset: tracedraw_core::geometry::Vec2::new(2.0, -2.0),
                    color: Color::cmyk_pct(100.0, 0.0, 0.0, 0.0),
                    gap: 0.0,
                });
            },
        )
        .expect("rect");
        let img = image::RgbaImage::from_pixel(3, 2, image::Rgba([220, 20, 20, 255]));
        app.new_shape(ShapeKind::Bitmap {
            rect: tracedraw_core::geometry::Rect::new(70.0, 10.0, 100.0, 30.0),
            width_px: 3,
            height_px: 2,
            png: crate::bitmap_fx::encode(&img).expect("png").into(),
            fx: None,
        })
        .expect("bitmap");
        app.run(tracedraw_core::Command::AddPage {
            name: None,
            size,
            layer: None,
        });
        app.page = app.doc().pages[1].id;
        app.new_shape_styled(
            ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(10.0, 10.0, 30.0, 30.0),
                radius: 0.0,
                corners: None,
            },
            |s| {
                s.name = Some("second-page".into());
                s.fill = Fill::Solid(Color::rgb8(20, 200, 20));
            },
        )
        .expect("rect");

        let json = |doc: &Document| serde_json::to_string(&doc.pages).expect("json");
        let models = |doc: &Document| {
            let text = json(doc);
            let mut found: Vec<String> = text
                .match_indices("\"model\":\"")
                .map(|(i, m)| {
                    text[i + m.len()..]
                        .split('"')
                        .next()
                        .unwrap_or_default()
                        .to_string()
                })
                .collect();
            found.sort();
            found.dedup();
            found
        };
        let pictures = |doc: &Document| {
            use base64::Engine;
            let text = json(doc);
            text.match_indices("\"png\":\"")
                .filter_map(|(i, m)| {
                    let b64 = text[i + m.len()..].split('"').next()?;
                    base64::engine::general_purpose::STANDARD.decode(b64).ok()
                })
                .collect::<Vec<_>>()
        };

        let plain = print_sheets(&app, &PrintState::default());
        assert!(
            models(&plain)
                .iter()
                .any(|m| m != "gray" && m != "registration")
        );
        let names: Vec<bool> = plain
            .pages
            .iter()
            .map(|p| {
                serde_json::to_string(p)
                    .expect("json")
                    .contains("first-page")
            })
            .collect();
        assert_eq!(names, [true, false], "in the drawing's order");

        let st = PrintState {
            grayscale: true,
            crop_marks: true,
            color_bars: true,
            ..PrintState::default()
        };
        let gray = print_sheets(&app, &st);
        for m in models(&gray) {
            assert!(m == "gray" || m == "registration", "{m} left in grayscale");
        }
        let pics = pictures(&gray);
        assert_eq!(pics.len(), 1);
        assert!(crate::bitmap_fx::png_is_gray(&pics[0]), "a grey picture");
        let px = image::load_from_memory(&pics[0]).expect("png").to_luma8();
        let want = (0.299f32 * 220.0 + 0.587 * 20.0 + 0.114 * 20.0).round() as i32;
        assert!((i32::from(px.get_pixel(1, 1).0[0]) - want).abs() <= 1);
        // The red fill as its luminance.
        assert!(json(&gray).contains("first-page"));
        assert!(
            models(&app.doc().clone()).iter().any(|m| m == "rgb"),
            "the drawing keeps its colours"
        );
        // Separations print from the colours, whatever the box says.
        let sep = print_sheets(
            &app,
            &PrintState {
                grayscale: true,
                separations: true,
                ..PrintState::default()
            },
        );
        assert!(models(&sep).iter().any(|m| m == "rgb"));

        let st = PrintState {
            reverse_order: true,
            ..PrintState::default()
        };
        let rev = print_sheets(&app, &st);
        let names: Vec<bool> = rev
            .pages
            .iter()
            .map(|p| {
                serde_json::to_string(p)
                    .expect("json")
                    .contains("first-page")
            })
            .collect();
        assert_eq!(names, [false, true], "the last page first");
        let both = print_sheets(
            &app,
            &PrintState {
                reverse_order: true,
                grayscale: true,
                ..PrintState::default()
            },
        );
        assert_eq!(both.pages.len(), 2);
        assert!(
            serde_json::to_string(&both.pages[1])
                .expect("json")
                .contains("first-page")
        );
    }

    #[test]
    fn the_print_dialog_settings_make_the_layout() {
        let app = App::headless();
        let mut st = PrintState {
            layout_kind: 3,
            cols: 4,
            rows: 0,
            sheet: 2,
            landscape: true,
            crop_marks: true,
            ..Default::default()
        };
        let o = print_layout(&app, &st, app.doc());
        assert_eq!(
            o.layout,
            crate::imposition::Layout::Repeat { cols: 4, rows: 1 }
        );
        assert_eq!(o.sheet, Some(Size::new(297.0, 210.0)));
        assert!(o.marks.crop && !o.marks.registration);
        // Bleed only when asked for, as far as the bleed limit.
        st.bleed_limit = 3.0;
        assert_eq!(print_layout(&app, &st, app.doc()).bleed, 0.0);
        st.bleed = true;
        assert_eq!(print_layout(&app, &st, app.doc()).bleed, 3.0);
        // Orientation overrides the Landscape box.
        st.orientation = 1;
        assert_eq!(
            print_layout(&app, &st, app.doc()).sheet,
            Some(Size::new(210.0, 297.0))
        );
        st.orientation = 0;
        st.sheet = 0;
        assert_eq!(print_layout(&app, &st, app.doc()).sheet, None);
    }

    #[test]
    fn sizes_read_in_kilobytes_and_megabytes() {
        assert_eq!(human_size(10), "1 KB");
        assert_eq!(human_size(820 * 1024), "820 KB");
        assert_eq!(human_size(3 * 1024 * 1024 + 400 * 1024), "3.4 MB");
    }

    #[test]
    fn csv_parses_headers_and_rows() {
        let (h, r) = parse_csv("Name,City\nAna,Porto\nBob,Lisboa\n");
        assert_eq!(h, vec!["Name", "City"]);
        assert_eq!(r.len(), 2);
        assert_eq!(r[1][1], "Lisboa");
        let (h2, _) = parse_csv("a;b\n1;2\n");
        assert_eq!(h2, vec!["a", "b"]);
        let (h3, r3) = parse_csv("a\tb, c\n1\t2\n");
        assert_eq!(h3, vec!["a", "b, c"]);
        assert_eq!(r3[0], vec!["1", "2"]);
    }

    /// Data files as spreadsheets save them read: UTF-8 with its mark (no
    /// stray mark in the first field), UTF-16 (Unicode Text) and code page
    /// 1252, which Excel writes CSV in on Windows (#495).
    #[test]
    fn data_files_in_every_encoding_read() {
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend("Nome,Cidade\nJoão,Évora\n".as_bytes());
        let (h, r) = parse_csv(&decode_text(&bom));
        assert_eq!(h, vec!["Nome", "Cidade"]);
        assert_eq!(r[0], vec!["João", "Évora"]);
        let mut le = vec![0xFF, 0xFE];
        for u in "Name\tCity\nAna\tPorto\n".encode_utf16() {
            le.extend(u.to_le_bytes());
        }
        let (h, r) = parse_csv(&decode_text(&le));
        assert_eq!(h, vec!["Name", "City"]);
        assert_eq!(r[0], vec!["Ana", "Porto"]);
        // "João" and the euro sign in code page 1252 are not UTF-8.
        let ansi = b"Name;Price\nJo\xe3o;5 \x80\n";
        let (h, r) = parse_csv(&decode_text(ansi));
        assert_eq!(h, vec!["Name", "Price"]);
        assert_eq!(r[0], vec!["João", "5 €"]);
    }

    #[test]
    fn replace_variants() {
        assert_eq!(replace_in("Hello hello", "hello", "x", false, false), "x x");
        assert_eq!(
            replace_in("Hello hello", "hello", "x", true, false),
            "Hello x"
        );
        assert_eq!(
            replace_in("cat concat", "cat", "dog", false, true),
            "dog concat"
        );
    }

    /// Both opaque odd RGB samples and partial alpha must survive lossless
    /// encoding at every quality. Quality is never a quantization switch.
    #[test]
    fn lossless_webp_keeps_rgb_and_alpha_below_quality_100() {
        let mut pm = tiny_skia::Pixmap::new(3, 1).expect("pixmap");
        pm.pixels_mut()[0] =
            tiny_skia::PremultipliedColorU8::from_rgba(37, 21, 39, 255).expect("pixel");
        pm.pixels_mut()[1] =
            tiny_skia::PremultipliedColorU8::from_rgba(31, 17, 59, 127).expect("pixel");
        let expected: Vec<_> = pm
            .pixels()
            .iter()
            .map(|p| {
                let p = p.demultiply();
                [p.red(), p.green(), p.blue(), p.alpha()]
            })
            .collect();
        for quality in [40, 90, 100] {
            let bytes = encode_raster_with(&pm, "webp", quality, true).expect("encode");
            let back = image::load_from_memory(&bytes)
                .expect("decode")
                .into_rgba8();
            let got: Vec<_> = back.pixels().map(|p| p.0).collect();
            assert_eq!(got, expected, "lossless at quality {quality}");
        }
    }

    #[test]
    fn webp_quality_shrinks_the_file() {
        let mut pm = tiny_skia::Pixmap::new(256, 128).expect("pixmap");
        for (i, p) in pm.pixels_mut().iter_mut().enumerate() {
            let (x, y) = ((i % 256) as u8, (i / 256) as u8);
            let n = ((x as u32 * 7 + y as u32 * 13) % 5) as u8;
            *p = tiny_skia::PremultipliedColorU8::from_rgba(x, y.wrapping_mul(2), x ^ y ^ n, 255)
                .expect("pixel");
        }
        // Lossy: far smaller at a low quality, and still a WebP (#592).
        let lossy = encode_raster_with(&pm, "webp", 10, false).expect("webp");
        let lossy_high = encode_raster_with(&pm, "webp", 90, false).expect("webp");
        assert!(
            lossy.len() < lossy_high.len(),
            "{} vs {}",
            lossy.len(),
            lossy_high.len()
        );
        assert!(image::load_from_memory(&lossy).is_ok());
        let full = encode_raster(&pm, "webp", 100).expect("webp");
        assert!(
            lossy.len() * 3 < full.len(),
            "{} vs {}",
            lossy.len(),
            full.len()
        );
        let low = encode_raster(&pm, "webp", 40).expect("webp");
        // The old assertion required a smaller lossless file by rounding
        // colours. That was data loss, not a compression-quality setting.
        assert_eq!(
            image::load_from_memory(&low)
                .expect("decode low")
                .into_rgba8(),
            image::load_from_memory(&full)
                .expect("decode full")
                .into_rgba8(),
        );
        let back = image::load_from_memory(&full).expect("decode").to_rgba8();
        assert_eq!(back.get_pixel(37, 21).0, {
            let d = pm.pixel(37, 21).expect("pixel").demultiply();
            [d.red(), d.green(), d.blue(), d.alpha()]
        });
    }

    /// The artwork moved in Print Preview prints moved by that much on
    /// the sheet; at first it stays (#536).
    #[test]
    fn the_artwork_moved_in_print_preview_prints_moved() {
        let mut app = App::headless();
        app.new_shape(ShapeKind::Rect {
            rect: tracedraw_core::geometry::Rect::new(20.0, 20.0, 60.0, 50.0),
            radius: 0.0,
            corners: None,
        })
        .expect("rect");
        let st = PrintState {
            fit_to_page: false,
            ..PrintState::default()
        };
        // The page goes on the sheet in a frame that clips it: the
        // rectangle is inside, with the frame's transform.
        let bounds = |doc: &Document| {
            doc.pages[0]
                .layers
                .iter()
                .flat_map(|l| &l.shapes)
                .find_map(|s| match &s.kind {
                    ShapeKind::ClipFrame { contents, .. } => contents
                        .iter()
                        .find(|c| matches!(c.kind, ShapeKind::Rect { .. }))
                        .map(|c| {
                            let mut c = c.clone();
                            c.transform = s.transform * c.transform;
                            c.bounds()
                        }),
                    _ => None,
                })
                .expect("the rectangle")
        };
        let at_first = bounds(&print_sheets(&app, &st));
        let moved = PrintState {
            art_offset: (5.0, -3.0),
            ..st.clone()
        };
        let b = bounds(&print_sheets(&app, &moved));
        assert!(
            (b.x0 - at_first.x0 - 5.0).abs() < 1e-6,
            "{b:?} {at_first:?}"
        );
        assert!((b.y0 - at_first.y0 + 3.0).abs() < 1e-6);
    }

    /// The bitmap editor of the preferences gets the file as its argument;
    /// a macOS application bundle opens through `open -a` (#593).
    #[test]
    fn the_bitmap_editor_gets_the_file() {
        let p = Path::new("/tmp/bitmap-7.png");
        let (prog, args) = editor_command(" \"/usr/bin/gimp\" ", p);
        assert_eq!(prog, "/usr/bin/gimp");
        assert_eq!(args, vec!["/tmp/bitmap-7.png".to_string()]);
        let (prog, args) = editor_command("/Applications/GIMP.app", p);
        if cfg!(target_os = "macos") {
            assert_eq!(prog, "open");
            assert_eq!(args[0], "-a");
        } else {
            assert_eq!(prog, "/Applications/GIMP.app");
            assert_eq!(args.len(), 1);
        }
    }

    /// Fit to page shrinks objects running off the page to fit inside it,
    /// centred; objects wholly off the page stay out (#534).
    /// A drawing on an A4 page: two selected squares (10 x 10 mm, a 1 mm
    /// outline) 40 mm apart, one not selected, an object on a master
    /// layer and a page background.
    fn selection_drawing() -> (App, [tracedraw_core::ShapeId; 2]) {
        use tracedraw_core::{Color, Fill, Layer, Shape, Stroke};
        let mut doc = Document::new("sel", Size::new(210.0, 297.0));
        let square = |doc: &mut Document, x: f64, y: f64| {
            let mut s = Shape::new(
                doc.ids_mut().shape(),
                ShapeKind::Rect {
                    rect: Rect::new(x, y, x + 10.0, y + 10.0),
                    radius: 0.0,
                    corners: None,
                },
            );
            s.fill = Fill::Solid(Color::rgb8(200, 30, 30));
            s.stroke = Some(Stroke {
                width: 1.0,
                ..Stroke::default()
            });
            s
        };
        let a = square(&mut doc, 50.0, 100.0);
        let b = square(&mut doc, 90.0, 120.0);
        let other = square(&mut doc, 150.0, 200.0);
        let ids = [a.id, b.id];
        doc.pages[0].layers[0].shapes.extend([a, b, other]);
        doc.pages[0].background = Some(Fill::Solid(Color::rgb8(0, 0, 255)));
        let master_shape = square(&mut doc, 5.0, 5.0);
        let mut master = Layer::master(doc.ids_mut().layer(), "Master", Default::default());
        master.shapes.push(master_shape);
        doc.master.push(master);
        let mut app = App::headless();
        app.engine = tracedraw_core::Engine::new(doc);
        if let Some(p) = app.doc().pages.first() {
            app.page = p.id;
        }
        app.selection = ids.to_vec();
        (app, ids)
    }

    /// The shapes a sheet prints (what its page clips hold) and their
    /// bounds on the sheet.
    fn printed(sheet: &tracedraw_core::document::Page) -> Vec<Rect> {
        sheet
            .layers
            .iter()
            .flat_map(|l| &l.shapes)
            .filter_map(|c| match &c.kind {
                ShapeKind::ClipFrame { contents, .. } => Some(
                    contents
                        .iter()
                        .map(|s| {
                            let mut s = s.clone();
                            s.transform_by(c.transform);
                            s.bounds()
                        })
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            })
            .flatten()
            .collect()
    }

    /// Print range Selection prints only the selected objects (no other
    /// object, nothing of the master layers, no page background) on a
    /// page the size of the area they cover: the sheet sized to the page
    /// is that area, a fixed sheet gets it centred at its size, and the
    /// layouts repeat it (#1001).
    #[test]
    fn the_selection_prints_alone_on_a_page_of_its_size() {
        let (app, _) = selection_drawing();
        // The two squares with their outlines: from (49.5, 99.5) to
        // (100.5, 130.5).
        let (w, h) = (51.0, 31.0);
        let st = PrintState {
            range: 3,
            ..PrintState::default()
        };
        let sheets = print_sheets(&app, &st);
        assert_eq!(sheets.pages.len(), 1);
        let sheet = &sheets.pages[0];
        assert!(
            (sheet.size.width - w).abs() < 1e-6 && (sheet.size.height - h).abs() < 1e-6,
            "{:?}",
            sheet.size
        );
        let shapes = printed(sheet);
        assert_eq!(shapes.len(), 2, "only the two selected squares: {shapes:?}");
        // As they are placed among themselves, the area's corner at the
        // sheet's corner.
        let mut x0: Vec<f64> = shapes.iter().map(|r| r.x0).collect();
        x0.sort_by(f64::total_cmp);
        assert!(
            (x0[0] - 0.5).abs() < 1e-6 && (x0[1] - 40.5).abs() < 1e-6,
            "{x0:?}"
        );
        // On an A4 sheet: centred, at its size.
        let st = PrintState {
            range: 3,
            sheet: 2,
            orientation: 1,
            ..PrintState::default()
        };
        let sheets = print_sheets(&app, &st);
        let sheet = &sheets.pages[0];
        assert_eq!(sheet.size, Size::new(210.0, 297.0));
        let area = printed(sheet)
            .into_iter()
            .reduce(|a, b| a.union(b))
            .expect("printed");
        assert!((area.width() - 50.0).abs() < 1e-6, "{area:?}");
        assert!((area.center().x - 105.0).abs() < 1e-6, "{area:?}");
        // Repeated 2 x 3 on the sheet: six copies of the area.
        let st = PrintState {
            range: 3,
            sheet: 2,
            orientation: 1,
            layout_kind: 3,
            cols: 2,
            rows: 3,
            ..PrintState::default()
        };
        let sheets = print_sheets(&app, &st);
        assert_eq!(printed(&sheets.pages[0]).len(), 12);
        let plan = crate::imposition::plan(&[Size::new(w, h)], &print_layout(&app, &st, &sheets));
        assert!(
            plan[0]
                .cells
                .iter()
                .all(|c| (c.trim.width() - w).abs() < 1e-6)
        );
    }

    /// Keep size: the selection never prints smaller than it is, even on
    /// a sheet too small for it; without it, it shrinks to fit (#1001).
    #[test]
    fn keep_size_keeps_the_selection_at_its_size() {
        let (app, _) = selection_drawing();
        let a6ish = |keep_size| PrintState {
            range: 3,
            sheet: crate::ui::dialogs::CUSTOM_SHEET,
            custom_sheet: Size::new(30.0, 30.0),
            orientation: 1,
            keep_size,
            ..PrintState::default()
        };
        let width = |st: &PrintState| {
            printed(&print_sheets(&app, st).pages[0])
                .into_iter()
                .reduce(|a, b| a.union(b))
                .map(|r| r.width())
                .expect("printed")
        };
        assert!((width(&a6ish(true)) - 50.0).abs() < 1e-6);
        assert!(width(&a6ish(false)) < 30.0);
    }

    #[test]
    fn fit_to_page_shrinks_what_runs_off_the_page() {
        let mut app = App::headless();
        let size = app.page_size();
        let rect = |app: &mut App, r: Rect| {
            app.new_shape(ShapeKind::Rect {
                rect: r,
                radius: 0.0,
                corners: None,
            })
            .expect("rect")
        };
        // Twice the page's width, off its right edge; another wholly off.
        let wide = rect(
            &mut app,
            Rect::new(10.0, 10.0, 10.0 + size.width * 2.0, 60.0),
        );
        let away = rect(&mut app, Rect::new(-300.0, 0.0, -250.0, 50.0));
        let mut page = app.doc().pages[0].clone();
        fit_contents_to_page(&mut page, false);
        let bounds = |page: &tracedraw_core::document::Page, id| {
            page.layers
                .iter()
                .flat_map(|l| &l.shapes)
                .find(|s| s.id == id)
                .map(|s| s.bounds())
                .expect("shape")
        };
        let b = bounds(&page, wide);
        assert!(b.x0 >= -1e-6 && b.x1 <= size.width + 1e-6, "{b:?}");
        assert!((b.width() - size.width).abs() < 1e-6, "as wide as the page");
        assert!((b.center().y - size.height / 2.0).abs() < 1e-6, "centred");
        assert_eq!(bounds(&page, away), Rect::new(-300.0, 0.0, -250.0, 50.0));
        // Printing the selection fits it whatever it covers.
        let mut page = app.doc().pages[0].clone();
        fit_contents_to_page(&mut page, true);
        assert!(bounds(&page, away).x0 >= -1e-6);
        // Objects inside the page stay as they are.
        let mut page = app.doc().pages[0].clone();
        page.layers
            .iter_mut()
            .for_each(|l| l.shapes.retain(|s| s.id != wide && s.id != away));
        let before = page.clone();
        fit_contents_to_page(&mut page, false);
        assert_eq!(page, before);
    }

    /// A blend prints as it is drawn when the print moves or recolours
    /// its objects: with Fit to page (the end object running off the
    /// page), with the selection printed as the page, and as a negative,
    /// the steps lie between the start and the end as printed and their
    /// colours run between the printed colours. The steps were evaluated
    /// from the copy of the end object the blend keeps, which the print
    /// left where (and as) it was on the page (#1541).
    #[test]
    fn a_blend_prints_between_its_objects_as_printed() {
        use tracedraw_core::{Color, Fill, live::Effect};
        let mut app = App::headless();
        let square = |app: &mut App, r: Rect, c: Color| {
            let id = app
                .new_shape(ShapeKind::Rect {
                    rect: r,
                    radius: 0.0,
                    corners: None,
                })
                .expect("square");
            app.run(Command::SetFill {
                shapes: vec![id],
                fill: Fill::Solid(c),
            });
            id
        };
        // On an A4 page: the start inside, the end running off the right.
        let a = square(
            &mut app,
            Rect::new(20.0, 100.0, 40.0, 120.0),
            Color::rgb8(255, 0, 0),
        );
        let b = square(
            &mut app,
            Rect::new(180.0, 100.0, 260.0, 180.0),
            Color::rgb8(0, 0, 255),
        );
        app.select(vec![a, b]);
        app.blend_selection(3, 0.0, 0.0, 0.0);
        app.refresh_blends();
        app.selection = vec![a, b];
        // The start, the end and the steps of the first sheet, on the
        // sheet: (bounds, colour) each.
        let printed_blend = |doc: &Document| {
            let mut start = None;
            let mut end = None;
            let mut steps = Vec::new();
            for c in doc.pages[0].layers.iter().flat_map(|l| &l.shapes) {
                let ShapeKind::ClipFrame { contents, .. } = &c.kind else {
                    continue;
                };
                for s in contents {
                    let on_sheet = |s: &tracedraw_core::Shape| {
                        let mut s = s.clone();
                        s.transform_by(c.transform);
                        (s.bounds(), s.fill.preview_color().expect("a colour"))
                    };
                    if s.effects.iter().any(|e| matches!(e, Effect::Blend { .. })) {
                        let ev = tracedraw_core::live::evaluate(s);
                        start = Some(on_sheet(s));
                        steps.extend(ev.below.iter().chain(&ev.above).map(on_sheet));
                    } else if s.effects.is_empty() {
                        end = Some(on_sheet(s));
                    }
                }
            }
            (start.expect("start"), end.expect("end"), steps)
        };
        let between = |(start, end, steps): &((Rect, Color), (Rect, Color), Vec<(Rect, Color)>)| {
            assert_eq!(steps.len(), 3);
            let (s, e) = (start.0.center(), end.0.center());
            // From the start to the end, the middle step half way.
            for (i, (b, _)) in steps.iter().enumerate() {
                let want = s.lerp(e, (i + 1) as f64 / 4.0);
                let got = b.center();
                assert!((got - want).hypot() < 1e-6, "{i}: {got:?}, not {want:?}");
            }
            let [r0, g0, b0] = start.1.to_rgb8();
            let [r1, g1, b1] = end.1.to_rgb8();
            let [r, g, bl] = steps[1].1.to_rgb8();
            let mid = |x: u8, y: u8| (x as i32 + y as i32) / 2;
            assert!(
                (r as i32 - mid(r0, r1)).abs() <= 1
                    && (g as i32 - mid(g0, g1)).abs() <= 1
                    && (bl as i32 - mid(b0, b1)).abs() <= 1,
                "{:?} between {:?} and {:?}",
                steps[1].1,
                start.1,
                end.1
            );
        };
        let fit = PrintState {
            fit_to_page: true,
            ..PrintState::default()
        };
        let p = printed_blend(&print_sheets(&app, &fit));
        assert!(
            p.1.0.x1 <= 210.0 + 1e-6,
            "shrunk onto the page: {:?}",
            p.1.0
        );
        between(&p);
        let selection = PrintState {
            range: 3,
            ..PrintState::default()
        };
        between(&printed_blend(&print_sheets(&app, &selection)));
        let negative = PrintState {
            invert: true,
            ..PrintState::default()
        };
        let p = printed_blend(&print_sheets(&app, &negative));
        assert_eq!(p.0.1.to_rgb8(), [0, 255, 255]);
        between(&p);
        // Exported alone, the start keeps its steps running to the end's
        // place beside it.
        app.selection = vec![a];
        let (doc, bounds) = export_document(
            &app,
            &ExportState {
                selection_only: true,
                ..ExportState::default()
            },
        );
        let s = doc.pages[0].layers[0].shapes[0].clone();
        let ev = tracedraw_core::live::evaluate(&s);
        let last = ev
            .above
            .last()
            .or(ev.below.first())
            .expect("steps")
            .bounds()
            .center();
        // Moved with the start: the last step three quarters of the way
        // from the start's centre (30, 110) to the end's (220, 140).
        let want = tracedraw_core::geometry::Point::new(172.5 - bounds.x0, 132.5 - bounds.y0);
        assert!((last - want).hypot() < 1e-6, "{last:?}, not {want:?}");
    }
}

#[cfg(test)]
mod scratch_191 {
    use super::*;
    use tracedraw_core::{Color, Fill, Stroke, TextSpan};

    #[test]
    #[ignore]
    fn scratch_black_text_outputs() {
        tracedraw_text::install();
        let dir = std::path::PathBuf::from(
            "/tmp/claude-0/-home-claude/ecd2ad2c-4fd8-5343-ab00-e2a436bfbe5e/scratchpad/p191",
        );
        std::fs::create_dir_all(&dir).unwrap();
        let mut report = String::new();
        let colors: Vec<(&str, Color)> = vec![
            ("k100", Color::cmyk_pct(0.0, 0.0, 0.0, 100.0)),
            ("rich", Color::cmyk_pct(60.0, 40.0, 40.0, 100.0)),
            ("reg", Color::Registration),
            ("gray0", Color::Gray { v: 0.0 }),
            ("rgb0", Color::BLACK),
            (
                "lab0",
                Color::Lab {
                    l: 0.0,
                    a: 0.0,
                    b: 0.0,
                },
            ),
            (
                "spot",
                Color::Spot {
                    ink: 1,
                    tint: 1.0,
                    c: 0.0,
                    m: 0.0,
                    y: 0.0,
                    k: 1.0,
                },
            ),
        ];
        for (name, color) in &colors {
            for mode in ["fill", "outline", "paragraph", "group"] {
                let mut app = App::headless();
                app.run(tracedraw_core::Command::SetInks {
                    inks: vec![tracedraw_core::document::Ink {
                        id: 1,
                        name: "Black".into(),
                        cmyk: [0.0, 0.0, 0.0, 1.0],
                    }],
                });
                let id = app
                    .new_shape(ShapeKind::Text {
                        spans: vec![TextSpan::new("WWWW", "Sans", 72.0)],
                        origin: tracedraw_core::geometry::Point::new(20.0, 100.0),
                        frame: (mode == "paragraph")
                            .then(|| tracedraw_core::geometry::Size::new(150.0, 60.0)),
                        align: Default::default(),
                        para: Default::default(),
                        on_path: None,
                    })
                    .expect("text");
                let outline = mode == "outline";
                app.run(tracedraw_core::Command::SetFill {
                    shapes: vec![id],
                    fill: if outline {
                        Fill::None
                    } else {
                        Fill::Solid(*color)
                    },
                });
                app.run(tracedraw_core::Command::SetStroke {
                    shapes: vec![id],
                    stroke: outline.then(|| Stroke::new(*color, 1.0)),
                });
                if mode == "group" {
                    let r = app
                        .new_shape(ShapeKind::Rect {
                            rect: tracedraw_core::geometry::Rect::new(5.0, 5.0, 10.0, 10.0),
                            radius: 0.0,
                            corners: None,
                        })
                        .unwrap();
                    app.select(vec![id, r]);
                    app.group_selection();
                }
                let tag = format!("{name}_{mode}");
                let variants: Vec<(&str, Box<dyn Fn(&mut PrintState)>)> = vec![
                    ("plain", Box::new(|_| {})),
                    ("opb", Box::new(|s| s.overprint_black = true)),
                    ("tb", Box::new(|s| s.text_black = true)),
                    ("s2c", Box::new(|s| s.spot_to_cmyk = true)),
                    ("bmp", Box::new(|s| s.as_bitmap = true)),
                    ("mirror", Box::new(|s| s.mirror = true)),
                    ("invert", Box::new(|s| s.invert = true)),
                    (
                        "marks",
                        Box::new(|s| {
                            s.crop_marks = true;
                            s.registration_marks = true;
                            s.color_bars = true;
                        }),
                    ),
                ];
                for (v, set) in &variants {
                    let mut st = PrintState::default();
                    set(&mut st);
                    let doc = print_sheets(&app, &st);
                    let pdf = tracedraw_io::pdf::document_to_pdf(&doc);
                    std::fs::write(dir.join(format!("print_{tag}_{v}.pdf")), pdf).unwrap();
                }
                let st = PrintState {
                    separations: true,
                    ..PrintState::default()
                };
                let doc = separations_doc(&print_sheets(&app, &st));
                std::fs::write(
                    dir.join(format!("print_{tag}_seps.pdf")),
                    tracedraw_io::pdf::document_to_pdf(&doc),
                )
                .unwrap();
                for (k, std_) in [
                    tracedraw_io::pdf::PdfStandard::None,
                    tracedraw_io::pdf::PdfStandard::X1a,
                    tracedraw_io::pdf::PdfStandard::X3,
                    tracedraw_io::pdf::PdfStandard::X4,
                ]
                .into_iter()
                .enumerate()
                {
                    let opts = tracedraw_io::pdf::PdfOptions {
                        standard: std_,
                        output_condition: "FOGRA39".into(),
                        flatten_dpi: 150.0,
                        ..Default::default()
                    };
                    let pdf = tracedraw_io::pdf::document_to_pdf_with(app.doc(), &opts);
                    std::fs::write(dir.join(format!("publish_{tag}_x{k}.pdf")), pdf).unwrap();
                }
                // Through `.cdr` and back.
                for ver in [14u16, 17, 20, 24, 27] {
                    let bytes = tracedraw_cdr::write::document_to_cdr_version(app.doc(), ver);
                    let Ok((doc, _)) = tracedraw_cdr::open_bytes(&bytes, "t") else {
                        report.push_str(&format!("{tag} v{ver}: unreadable\n"));
                        continue;
                    };
                    let mut texts = Vec::new();
                    fn walk(s: &tracedraw_core::Shape, out: &mut Vec<String>) {
                        match &s.kind {
                            ShapeKind::Text { spans, .. } => out.push(format!(
                                "fill {:?} stroke {:?} span_fills {:?}",
                                s.fill,
                                s.stroke.as_ref().map(|k| k.color),
                                spans.iter().map(|sp| sp.fill.clone()).collect::<Vec<_>>()
                            )),
                            ShapeKind::Group { children } => {
                                children.iter().for_each(|c| walk(c, out))
                            }
                            _ => {}
                        }
                    }
                    for s in doc
                        .pages
                        .iter()
                        .flat_map(|p| &p.layers)
                        .flat_map(|l| &l.shapes)
                    {
                        walk(s, &mut texts);
                    }
                    report.push_str(&format!("{tag} v{ver}: {texts:?}\n"));
                    let mut back = App::headless();
                    back.open_document(doc, None);
                    let pdf = tracedraw_io::pdf::document_to_pdf(&print_sheets(
                        &back,
                        &PrintState::default(),
                    ));
                    std::fs::write(dir.join(format!("cdr{ver}_{tag}.pdf")), pdf).unwrap();
                }
            }
        }
        std::fs::write(dir.join("report.txt"), report).unwrap();
    }
}

/// Diagnostic, run by hand: the inks of a CMYK export of a drawing set in
/// `TRACEDRAW_PROBE_FILE` (objects in `TRACEDRAW_PROBE_IDS`, comma
/// separated) against the inks of its RGB export converted by the same
/// colour management, overall and where the RGB export is green.
#[cfg(test)]
mod cmyk_probe {
    use super::*;

    #[test]
    #[ignore]
    fn cmyk_export_against_converted_rgb_export() {
        let Ok(path) = std::env::var("TRACEDRAW_PROBE_FILE") else {
            return;
        };
        let ids: Vec<tracedraw_core::ShapeId> = std::env::var("TRACEDRAW_PROBE_IDS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .map(tracedraw_core::ShapeId)
            .collect();
        let dpi: f64 = std::env::var("TRACEDRAW_PROBE_DPI")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(12.0);
        let hide: Vec<u64> = std::env::var("TRACEDRAW_PROBE_HIDE")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        let (mut doc, _) = tracedraw_cdr::open(&path).expect("open");
        for id in &hide {
            if let Some(s) = doc.find_shape_mut(tracedraw_core::ShapeId(*id)) {
                s.visible = false;
            }
        }
        let mut app = App::headless();
        app.open_document(doc, None);
        app.select(ids.clone());
        let raster = |mode: usize| {
            let st = ExportState {
                format: 3,
                color_mode: mode,
                dpi,
                selection_only: !ids.is_empty(),
                crop_to_page: false,
                ..Default::default()
            };
            let job = prepare_job(&app, Path::new("probe.jpg"), &st).expect("job");
            let screen = job.screen.clone();
            let map_gray = job.map_gray;
            tracedraw_core::color::engine::with_icc_scope(screen, map_gray, || {
                let cancel = AtomicBool::new(false);
                let work = ExportWork {
                    callback: None,
                    cancel: &cancel,
                    done: std::cell::Cell::new(0),
                    total: u64::MAX,
                };
                let opts = job.raster_options(0).expect("options");
                let (data, samples, _) = job.make_raster(0, &opts, &work).expect("raster");
                let channels = samples.channels();
                let converted: Vec<[f64; 4]> = data
                    .chunks_exact(channels)
                    .map(|p| match samples {
                        tracedraw_io::raster_export::Samples::Cmyk => {
                            [p[0], p[1], p[2], p[3]].map(|v| v as f64 / 255.0)
                        }
                        _ => match tracedraw_core::Color::rgb8(p[0], p[1], p[2]).convert_to("CMYK")
                        {
                            tracedraw_core::Color::Cmyk { c, m, y, k } => [c, m, y, k].map(f64::from),
                            _ => [f64::NAN; 4],
                        },
                    })
                    .collect();
                let rgb: Vec<[u8; 3]> = data
                    .chunks_exact(channels)
                    .map(|p| match samples {
                        tracedraw_io::raster_export::Samples::Cmyk => [0; 3],
                        _ => [p[0], p[1], p[2]],
                    })
                    .collect();
                (opts.width, opts.height, converted, rgb, format!("{samples:?}"))
            })
        };
        let (w1, h1, cmyk, _, s1) = raster(1);
        let (w0, h0, from_rgb, rgb, s0) = raster(0);
        println!("cmyk export {w1}x{h1} {s1}; rgb export {w0}x{h0} {s0}");
        assert_eq!((w0, h0), (w1, h1), "same grid");
        let mean = |px: &[[f64; 4]], pick: &dyn Fn(usize) -> bool| {
            let mut sum = [0.0; 4];
            let mut n: f64 = 0.0;
            for (i, p) in px.iter().enumerate() {
                if pick(i) {
                    for c in 0..4 {
                        sum[c] += p[c];
                    }
                    n += 1.0;
                }
            }
            (sum.map(|s| s / n.max(1.0)), n)
        };
        let green = |i: usize| rgb[i][1] > 110 && rgb[i][0] < 90 && rgb[i][2] < 90;
        let dark = |i: usize| rgb[i].iter().all(|v| *v < 60);
        for (name, pick) in [
            ("all", &(|_| true) as &dyn Fn(usize) -> bool),
            ("green", &green),
            ("dark", &dark),
        ] {
            let (a, n) = mean(&cmyk, pick);
            let (b, _) = mean(&from_rgb, pick);
            println!(
                "{name:>5} n={n:>8} cmyk export C{:.3} M{:.3} Y{:.3} K{:.3} | rgb converted C{:.3} M{:.3} Y{:.3} K{:.3}",
                a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]
            );
        }
    }
}

/// Separations draw every paint the composite draws: the extra fills and
/// outlines, the picture of bitmap effects instead of the paints under it,
/// and an opacity mask's luminance left the same on every plate.
#[cfg(test)]
mod print_paint_tests {
    use super::*;
    use tracedraw_core::{Color, Fill, Shape};

    const PLATES: [Plate; 4] = [
        Plate::Process(0),
        Plate::Process(1),
        Plate::Process(2),
        Plate::Process(3),
    ];

    fn doc_with(build: impl FnOnce(&mut Document) -> Vec<Shape>) -> Document {
        let mut doc = Document::new("p", tracedraw_core::Size::new(40.0, 20.0));
        let layer = doc.pages[0].layers[0].id;
        let shapes = build(&mut doc);
        doc.layer_mut(layer).expect("layer").shapes.extend(shapes);
        doc
    }

    fn rect(doc: &mut Document, r: Rect) -> Shape {
        let mut s = Shape::new(
            doc.ids_mut().shape(),
            ShapeKind::Rect {
                rect: r,
                radius: 0.0,
                corners: None,
            },
        );
        s.stroke = None;
        s
    }

    fn plate_pixels(doc: &Document, plate: Plate) -> Vec<u8> {
        let (page, warnings) = mapped_plate_page(&doc.pages[0], plate);
        assert!(warnings.is_empty(), "{warnings:?}");
        let id = page.id;
        let mut printed = doc.clone();
        printed.pages = vec![page];
        tracedraw_render::render_page_image(&printed, id, 50.8)
            .expect("plate")
            .data()
            .to_vec()
    }

    fn grey_only(pixels: &[u8]) -> bool {
        pixels.chunks_exact(4).all(|p| p[0] == p[1] && p[1] == p[2])
    }

    #[test]
    fn extra_fills_and_outlines_are_separated_like_the_main_ones() {
        let cyan = Color::cmyk8(255, 0, 0, 0);
        let magenta = Color::cmyk8(0, 255, 0, 0);
        let outline = tracedraw_core::Stroke {
            color: magenta,
            width: 2.0,
            ..Default::default()
        };
        let main = doc_with(|doc| {
            let mut s = rect(doc, Rect::new(5.0, 5.0, 35.0, 15.0));
            s.fill = Fill::Solid(cyan);
            s.stroke = Some(outline.clone());
            vec![s]
        });
        let extra = doc_with(|doc| {
            let mut s = rect(doc, Rect::new(5.0, 5.0, 35.0, 15.0));
            s.extra_fills = vec![Fill::Solid(cyan)];
            s.extra_strokes = vec![outline.clone()];
            vec![s]
        });
        for plate in PLATES {
            let want = plate_pixels(&main, plate);
            let got = plate_pixels(&extra, plate);
            assert!(grey_only(&got), "{plate:?}: an extra paint kept its colour");
            assert_eq!(got, want, "{plate:?}: extra paints differ from the main ones");
        }
        // The cyan plate carries the fill and the magenta plate the outline.
        let c = plate_pixels(&extra, Plate::Process(0));
        let m = plate_pixels(&extra, Plate::Process(1));
        assert!(c.chunks_exact(4).any(|p| p[0] < 10));
        assert!(m.chunks_exact(4).any(|p| p[0] < 10));
    }

    #[test]
    fn an_opacity_mask_hides_the_same_part_on_every_plate() {
        for invert in [false, true] {
            let doc = doc_with(|doc| {
                // Left half white (shows), right half black (hides).
                let mut frame = rect(doc, Rect::new(0.0, 0.0, 40.0, 20.0));
                frame.fill = Fill::Fountain(tracedraw_core::Fountain {
                    stops: vec![
                        tracedraw_core::Stop::new(0.0, Color::WHITE),
                        tracedraw_core::Stop::new(0.5, Color::WHITE),
                        tracedraw_core::Stop::new(0.501, Color::BLACK),
                        tracedraw_core::Stop::new(1.0, Color::BLACK),
                    ],
                    ..tracedraw_core::Fountain::two(
                        tracedraw_core::FountainKind::Linear,
                        Color::WHITE,
                        Color::BLACK,
                        0.0,
                    )
                });
                // The same amount of every ink, so each plate is the same
                // picture when the mask is the same.
                let mut contents = rect(doc, Rect::new(0.0, 0.0, 40.0, 20.0));
                contents.fill = Fill::Solid(Color::cmyk8(128, 128, 128, 128));
                let mut clip = Shape::new(
                    doc.ids_mut().shape(),
                    ShapeKind::ClipFrame {
                        frame: Box::new(frame),
                        contents: vec![contents],
                    },
                );
                clip.stroke = None;
                clip.opacity_mask = Some(tracedraw_core::document::OpacityMask { invert });
                vec![clip]
            });
            let plates: Vec<Vec<u8>> = PLATES.iter().map(|p| plate_pixels(&doc, *p)).collect();
            for (plate, pixels) in PLATES.iter().zip(&plates) {
                assert!(grey_only(pixels), "{plate:?}");
                assert_eq!(pixels, &plates[3], "{plate:?} masked another part (invert {invert})");
            }
            // The mask both shows and hides something.
            assert!(plates[0].chunks_exact(4).any(|p| p[0] < 160));
            assert!(plates[0].chunks_exact(4).any(|p| p[0] > 250));
        }
    }

    #[test]
    fn a_bitmap_effect_picture_is_separated_instead_of_the_paints_under_it() {
        // The effect's picture is green; the object under it is cyan.
        let mut green = tiny_skia::Pixmap::new(4, 4).expect("pixmap");
        for p in green.data_mut().chunks_exact_mut(4) {
            p.copy_from_slice(&[0, 160, 60, 255]);
        }
        let png: tracedraw_core::SharedBytes = green.encode_png().expect("png").into();
        let doc = doc_with(|doc| {
            let mut s = rect(doc, Rect::new(5.0, 5.0, 35.0, 15.0));
            s.fill = Fill::Solid(Color::cmyk8(255, 0, 0, 0));
            s.raster_fx = Some(Box::new(tracedraw_core::document::RasterFx {
                effects: vec![tracedraw_core::document::BitmapEffect {
                    id: "gaussian_blur".into(),
                    params: Default::default(),
                    visible: true,
                }],
                rect: Rect::new(0.0, 0.0, 30.0, 10.0),
                width_px: 4,
                height_px: 4,
                png,
                source: 0,
            }));
            vec![s]
        });
        for plate in PLATES {
            let pixels = plate_pixels(&doc, plate);
            assert!(grey_only(&pixels), "{plate:?}: the picture kept its colour");
        }
        // Green has yellow ink and no magenta: the picture, not the cyan
        // fill under it, is what the plates print.
        let y = plate_pixels(&doc, Plate::Process(2));
        let m = plate_pixels(&doc, Plate::Process(1));
        assert!(y.chunks_exact(4).any(|p| p[0] < 200), "no yellow from the picture");
        assert!(m.chunks_exact(4).all(|p| p[0] > 200), "magenta where green has none");
    }
}
