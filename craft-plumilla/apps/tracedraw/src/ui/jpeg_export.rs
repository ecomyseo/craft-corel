//! JPEG export: comparison previews and operative encoding/colour/size controls.
use super::dialogs::{self, ExportState, EXPORT_SIZE_UNITS};
use super::tips::RichTip;
use crate::app::App;
use crate::i18n::{tr, trf};
use egui::{Context, Ui, Vec2};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Pane {
    choices: Option<ExportState>,
    texture: Option<egui::TextureHandle>,
    original_texture: Option<egui::TextureHandle>,
    show_original: bool,
    key: Option<u64>,
    size: (u32, u32),
    output_size: (u32, u32),
    base_size_mm: (f64, f64),
    bytes: Option<usize>,
    profile_name: String,
    profile_bytes: usize,
    zoom: f32,
    pan: Vec2,
    detail: bool,
    detail_scale: f32,
    region_origin: (u32, u32),
    region_size: (u32, u32),
    shown_origin: Option<(u32, u32)>,
}
impl Default for Pane {
    fn default() -> Self {
        Self {
            choices: None,
            texture: None,
            original_texture: None,
            show_original: false,
            key: None,
            size: (1, 1),
            output_size: (1, 1),
            base_size_mm: (0.0, 0.0),
            bytes: None,
            profile_name: String::new(),
            profile_bytes: 0,
            zoom: 1.0,
            pan: Vec2::ZERO,
            detail: false,
            detail_scale: 1.0,
            region_origin: (0, 0),
            region_size: (640, 640),
            shown_origin: None,
        }
    }
}
struct Running {
    pane: usize,
    key: u64,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Result<crate::export::ExportPreviewResult, String>>,
    origin: Option<(u32, u32)>,
    choices: ExportState,
}
struct PreviewCache {
    panes: Vec<Pane>,
    active: usize,
    layout: usize,
    requested_key: Option<u64>,
    changed: Instant,
    running: Option<Running>,
    error: Option<String>,
    document_key: u64,
    download_bps: f64,
}
impl Default for PreviewCache {
    fn default() -> Self {
        Self {
            panes: vec![Pane::default(); 4],
            active: 0,
            layout: 0,
            requested_key: None,
            changed: Instant::now(),
            running: None,
            error: None,
            document_key: 0,
            download_bps: 56_000.0,
        }
    }
}
fn cache(ctx: &Context) -> Arc<Mutex<PreviewCache>> {
    let id = egui::Id::new("jpeg_export_preview_worker");
    ctx.data_mut(|d| {
        if let Some(cache) = d.get_temp::<Arc<Mutex<PreviewCache>>>(id) {
            return cache;
        }
        let cache = Arc::new(Mutex::new(PreviewCache::default()));
        d.insert_temp(id, cache.clone());
        cache
    })
}

/// Whether the actual currently requested preview has been published.
pub(crate) fn diagnostic_preview_ready(ctx: &Context) -> bool {
    let shared = cache(ctx);
    let state = shared.lock().unwrap_or_else(|e| e.into_inner());
    let pane = &state.panes[state.active];
    state.running.is_none()
        && state.error.is_none()
        && pane.texture.is_some()
        && pane.key.is_some()
        && pane.key == state.requested_key
}

/// Release only this context's diagnostic previews. Call after readiness
/// before starting another large job; cancellation itself is cooperative.
pub(crate) fn cancel_diagnostic_previews(ctx: &Context) {
    let shared = cache(ctx);
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    cancel(&mut state);
    state.running = None;
    state.panes = vec![Pane::default(); 4];
    state.requested_key = None;
    state.active = 0;
    state.error = None;
}
fn cancel(cache: &mut PreviewCache) {
    if let Some(run) = cache.running.as_ref() {
        run.cancel.store(true, Ordering::Relaxed);
    }
}
pub(super) fn cancel_preview(ctx: &Context) {
    let shared = cache(ctx);
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    cancel(&mut state);
    state.requested_key = None;
}
fn document_key(app: &App) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    app.engine.instance().hash(&mut hash);
    app.engine.revision().hash(&mut hash);
    app.page.raw().hash(&mut hash);
    hash.finish()
}
fn preview_key(app: &App, st: &ExportState, pane: &Pane) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    dialogs::export_preview_key(app, st).hash(&mut hash);
    pane.detail.hash(&mut hash);
    if pane.detail {
        pane.region_origin.hash(&mut hash);
        pane.region_size.hash(&mut hash);
    }
    hash.finish()
}

fn visible_panes(layout: usize) -> usize {
    match layout {
        0 => 1,
        3 => 4,
        _ => 2,
    }
}
fn wanted_choices(cache: &PreviewCache, current: &ExportState, index: usize) -> ExportState {
    if index == cache.active {
        current.clone()
    } else {
        cache.panes[index]
            .choices
            .clone()
            .unwrap_or_else(|| current.clone())
    }
}
fn update_preview(app: &App, ctx: &Context, st: &ExportState, cache: &mut PreviewCache) {
    let active = cache.active;
    let key = preview_key(app, st, &cache.panes[active]);
    if cache.requested_key != Some(key) {
        cache.requested_key = Some(key);
        cache.changed = Instant::now();
        cache.error = None;
        if cache.panes[active].key != Some(key)
            && cache
                .running
                .as_ref()
                .is_some_and(|r| r.pane != active || r.key != key)
        {
            cancel(cache);
        }
    }
    if let Some(run) = cache.running.take() {
        let expected = preview_key(
            app,
            &wanted_choices(cache, st, run.pane),
            &cache.panes[run.pane],
        );
        match run.receiver.try_recv() {
            Ok(Ok(result)) if !run.cancel.load(Ordering::Relaxed) && run.key == expected => {
                let image = egui::ColorImage::from_rgba_premultiplied(
                    [
                        result.image.width() as usize,
                        result.image.height() as usize,
                    ],
                    result.image.data(),
                );
                let pane = &mut cache.panes[run.pane];
                pane.size = (result.image.width(), result.image.height());
                pane.shown_origin = run.origin;
                let original = egui::ColorImage::from_rgba_premultiplied(
                    [
                        result.original.width() as usize,
                        result.original.height() as usize,
                    ],
                    result.original.data(),
                );
                pane.original_texture = Some(ctx.load_texture(
                    format!("jpeg_original_{}", run.pane),
                    original,
                    egui::TextureOptions::LINEAR,
                ));
                pane.texture = Some(ctx.load_texture(
                    format!("jpeg_preview_{}", run.pane),
                    image,
                    egui::TextureOptions::LINEAR,
                ));
                pane.key = Some(run.key);
                pane.output_size = result.output_size;
                pane.base_size_mm = result.base_size_mm;
                pane.bytes = result.estimated_bytes;
                pane.profile_name = result.profile_name;
                pane.profile_bytes = result.profile_bytes;
                pane.choices = Some(run.choices);
            }
            Ok(Ok(_)) => {}
            Ok(Err(error)) if !run.cancel.load(Ordering::Relaxed) => {
                cache.error = Some(error);
                cache.panes[run.pane].key = Some(run.key);
            }
            Ok(Err(_)) => {}
            Err(mpsc::TryRecvError::Empty) => cache.running = Some(run),
            Err(mpsc::TryRecvError::Disconnected) => {
                cache.error = Some(tr("jpeg.preview_failed"));
                cache.panes[run.pane].key = Some(run.key);
            }
        }
    }
    let pending = std::iter::once(active)
        .chain((0..visible_panes(cache.layout)).filter(|i| *i != active))
        .find_map(|index| {
            let choices = wanted_choices(cache, st, index);
            let key = preview_key(app, &choices, &cache.panes[index]);
            (cache.panes[index].key != Some(key)).then_some((index, key, choices))
        });
    if cache.running.is_none() && cache.changed.elapsed() >= Duration::from_millis(120) {
        if let Some((index, key, choices)) = pending.as_ref() {
            let pane = &cache.panes[*index];
            let origin = pane.detail.then_some(pane.region_origin);
            let prepared = if pane.detail {
                crate::export::prepare_preview_region_job(
                    app,
                    choices,
                    pane.region_origin,
                    pane.region_size,
                )
            } else {
                crate::export::prepare_preview_job(app, choices, 1024)
            };
            match prepared {
                Ok(job) => {
                    let (sender, receiver) = mpsc::channel();
                    let flag = Arc::new(AtomicBool::new(false));
                    let worker_flag = flag.clone();
                    match std::thread::Builder::new()
                        .name("jpeg-export-preview".into())
                        .spawn(move || {
                            let _ = sender.send(job.run(worker_flag));
                        }) {
                        Ok(_) => {
                            cache.running = Some(Running {
                                pane: *index,
                                key: *key,
                                cancel: flag,
                                receiver,
                                origin,
                                choices: choices.clone(),
                            })
                        }
                        Err(error) => {
                            cache.error = Some(error.to_string());
                            cache.panes[*index].key = Some(*key);
                        }
                    }
                }
                Err(error) => {
                    cache.error = Some(error);
                    cache.panes[*index].key = Some(*key);
                }
            }
        }
    }
    if cache.running.is_some() || pending.is_some() {
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

/// A DPI change with Keep size on keeps the pixel dimensions, while the
/// physical dimensions change inversely. The JPEG density metadata still changes.
pub(crate) fn change_dpi(st: &mut ExportState, new_dpi: f64) {
    if !new_dpi.is_finite() {
        return;
    }
    let old = if st.dpi.is_finite() {
        st.dpi.clamp(1.0, 2400.0)
    } else {
        300.0
    };
    let new = new_dpi.clamp(1.0, 2400.0);
    if st.keep_size {
        st.scale = (st.scale_or_one() * old / new).clamp(0.01, 100.0);
        st.scale_y = (valid_scale(st.scale_y) * old / new).clamp(0.01, 100.0);
    }
    st.dpi = new;
}
fn valid_scale(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.01, 100.0)
    } else {
        1.0
    }
}
pub(crate) fn change_scale(st: &mut ExportState, vertical: bool, value: f64) {
    let value = valid_scale(value);
    if vertical {
        st.scale_y = value;
        if st.keep_ratio {
            st.scale = value;
        }
    } else {
        st.scale = value;
        if st.keep_ratio {
            st.scale_y = value;
        }
    }
}
pub(crate) fn displayed_size(base: (f64, f64), st: &ExportState) -> (f64, f64) {
    let sx = st.scale_or_one();
    let sy = if st.keep_ratio {
        sx
    } else {
        valid_scale(st.scale_y)
    };
    let unit = EXPORT_SIZE_UNITS[st.size_unit.min(3)].1;
    if unit == 0.0 {
        (
            (base.0 * sx * st.dpi / 25.4).ceil(),
            (base.1 * sy * st.dpi / 25.4).ceil(),
        )
    } else {
        (base.0 * sx / unit, base.1 * sy / unit)
    }
}
fn typed_size(st: &mut ExportState, base: f64, vertical: bool, value: f64) {
    if !(base > 0.0 && value.is_finite() && value > 0.0) {
        return;
    }
    let unit = EXPORT_SIZE_UNITS[st.size_unit.min(3)].1;
    let scale = if unit == 0.0 {
        value * (1.0 - 1e-9) * 25.4 / (base * st.dpi)
    } else {
        value * unit / base
    };
    change_scale(st, vertical, scale);
}
fn check(ui: &mut Ui, value: &mut bool, key: &str) {
    ui.checkbox(value, tr(key))
        .rich_tip(super::tips::keyed(key));
}
fn colour_mode(ui: &mut Ui, st: &mut ExportState) {
    ui.horizontal(|ui| {
        ui.label(tr("dialog.color_mode"));
        let keys = ["jpeg.rgb", "jpeg.cmyk", "jpeg.gray"];
        super::keynav::ComboBox::from_id_salt("jpeg_colour")
            .selected_text(tr(keys[st.color_mode.min(2)]))
            .show_ui(ui, |ui| {
                for (i, key) in keys.iter().enumerate() {
                    ui.selectable_value(&mut st.color_mode, i, tr(key));
                }
            })
            .response
            .rich_tip(super::tips::keyed("dialog.color_mode"));
    });
}
fn quality_label(quality: u8) -> String {
    if quality == 50 {
        tr("jpeg.medium")
    } else {
        format!("{quality} %")
    }
}
fn settings(ui: &mut Ui, st: &mut ExportState) {
    colour_mode(ui, st);
    ui.add_enabled_ui(st.color_mode == 1, |ui| {
        check(ui, &mut st.overprint_black, "jpeg.overprint_black")
    });
    ui.horizontal(|ui| {
        ui.label(tr("jpeg.matte"));
        ui.color_edit_button_srgb(&mut st.matte)
            .rich_tip(super::tips::keyed("jpeg.matte"));
    });
    ui.horizontal(|ui| {
        ui.label(tr("dialog.quality"));
        super::keynav::ComboBox::from_id_salt("jpeg_quality")
            .selected_text(quality_label(st.quality))
            .width(120.0)
            .show_ui(ui, |ui| {
                for quality in [100, 90, 75, 50, 20] {
                    ui.selectable_value(&mut st.quality, quality, quality_label(quality));
                }
            })
            .response
            .rich_tip(super::tips::keyed("dialog.quality"));
        ui.add(
            super::field::NumField::new(&mut st.quality)
                .range(0..=100)
                .suffix(" %")
                .width(55.0),
        )
        .rich_tip(super::tips::keyed("dialog.quality"));
    });
    ui.horizontal(|ui| {
        ui.label(tr("jpeg.subformat"));
        super::keynav::ComboBox::from_id_salt("jpeg_subsampling")
            .selected_text(
                [
                    "4:4:4".to_string(),
                    tr("jpeg.subformat_standard"),
                    "4:2:0".into(),
                ][st.jpeg_subsampling.min(2)]
                .clone(),
            )
            .show_ui(ui, |ui| {
                for (i, label) in [
                    "4:4:4".to_string(),
                    tr("jpeg.subformat_standard"),
                    "4:2:0".into(),
                ]
                .into_iter()
                .enumerate()
                {
                    ui.selectable_value(&mut st.jpeg_subsampling, i, label);
                }
            })
            .response
            .rich_tip(super::tips::keyed("jpeg.subformat"));
    });
    let mut percent = (st.blur * 50.0).clamp(0.0, 100.0);
    let response = ui.add(
        super::Slider::new(&mut percent, 0.0..=100.0)
            .text(tr("jpeg.blur"))
            .suffix(" %"),
    );
    if response.changed() {
        st.blur = percent / 50.0;
    }
    response.rich_tip(super::tips::keyed("jpeg.blur"));
}
fn profile_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    }
}
fn advanced(ui: &mut Ui, st: &mut ExportState, pane: &Pane) {
    check(ui, &mut st.anti_alias, "jpeg.antialias");
    ui.label(tr("jpeg.colour_settings"));
    ui.radio_value(&mut st.use_proof_profile, false, tr("jpeg.document_colour"))
        .rich_tip(super::tips::keyed("jpeg.document_colour"));
    ui.radio_value(&mut st.use_proof_profile, true, tr("jpeg.proof_colour"))
        .rich_tip(super::tips::keyed("jpeg.proof_colour"));
    check(ui, &mut st.embed_icc, "jpeg.embed_icc");
    if pane.profile_bytes > 0 && !pane.profile_name.is_empty() {
        ui.label(format!(
            "{} · {}",
            pane.profile_name,
            profile_size(pane.profile_bytes)
        ));
    } else {
        ui.label(tr(if pane.key.is_some() {
            "jpeg.no_profile"
        } else {
            "jpeg.profile_pending"
        }));
    }
    check(ui, &mut st.jpeg_progressive, "jpeg.progressive");
    check(ui, &mut st.jpeg_optimize_huffman, "jpeg.optimize");
}
fn dimension_field<'a>(value: &'a mut f64, unit_index: usize) -> super::field::NumField<'a, f64> {
    let (label, unit) = EXPORT_SIZE_UNITS[unit_index.min(3)];
    let field = super::field::NumField::new(value)
        .range(0.01..=200_000.0)
        .max_decimals(if unit == 0.0 { 0 } else { 2 })
        .decimal_comma(crate::i18n::language().starts_with("es"))
        .width(92.0);
    if unit > 0.0 {
        field.suffix(format!(" {label}")).unit_mm(unit)
    } else {
        field
    }
}
fn unit_label(index: usize) -> String {
    crate::app::Units::from_id(EXPORT_SIZE_UNITS[index.min(3)].0).label()
}
fn transforms(ui: &mut Ui, st: &mut ExportState, base: (f64, f64)) {
    ui.horizontal(|ui| {
        ui.label(tr("jpeg.units"));
        super::keynav::ComboBox::from_id_salt("jpeg_size_unit")
            .selected_text(unit_label(st.size_unit))
            .show_ui(ui, |ui| {
                for i in 0..EXPORT_SIZE_UNITS.len() {
                    ui.selectable_value(&mut st.size_unit, i, unit_label(i));
                }
            })
            .response
            .rich_tip(super::tips::keyed("jpeg.units"));
    });
    let (mut width, mut height) = displayed_size(base, st);
    egui::Grid::new("jpeg_dimensions")
        .num_columns(3)
        .spacing([8.0, 7.0])
        .show(ui, |ui| {
            ui.label(tr("dialog.width"));
            let response = ui.add(dimension_field(&mut width, st.size_unit));
            if response.changed() {
                typed_size(st, base.0, false, width);
            }
            response.rich_tip(super::tips::keyed("dialog.width"));
            let mut percent = st.scale_or_one() * 100.0;
            let response = ui.add(
                super::field::NumField::new(&mut percent)
                    .range(1.0..=10_000.0)
                    .suffix(" %")
                    .width(68.0),
            );
            if response.changed() {
                change_scale(st, false, percent / 100.0);
            }
            response.rich_tip(super::tips::keyed("jpeg.width_percent"));
            ui.end_row();
            ui.label(tr("dialog.height"));
            let response = ui.add(dimension_field(&mut height, st.size_unit));
            if response.changed() {
                typed_size(st, base.1, true, height);
            }
            response.rich_tip(super::tips::keyed("dialog.height"));
            let mut percent = if st.keep_ratio {
                st.scale_or_one()
            } else {
                valid_scale(st.scale_y)
            } * 100.0;
            let response = ui.add(
                super::field::NumField::new(&mut percent)
                    .range(1.0..=10_000.0)
                    .suffix(" %")
                    .width(68.0),
            );
            if response.changed() {
                change_scale(st, true, percent / 100.0);
            }
            response.rich_tip(super::tips::keyed("jpeg.height_percent"));
            ui.end_row();
        });
    let old = st.keep_ratio;
    check(ui, &mut st.keep_ratio, "jpeg.keep_ratio");
    if st.keep_ratio && !old {
        st.scale_y = st.scale_or_one();
    }
    ui.horizontal(|ui| {
        ui.label(tr("dialog.resolution"));
        let mut dpi = st.dpi;
        let response = ui.add(
            super::field::NumField::new(&mut dpi)
                .range(1.0..=2400.0)
                .suffix(" dpi")
                .width(100.0),
        );
        if response.changed() {
            change_dpi(st, dpi);
        }
        response.rich_tip(super::tips::keyed("dialog.resolution"));
        if ui
            .small_button("300")
            .rich_tip_with(|| "300 dpi".into())
            .clicked()
        {
            change_dpi(st, 300.0);
        }
    });
    ui.checkbox(&mut st.keep_size, tr("jpeg.keep_size"))
        .rich_tip_with(|| tr("jpeg.keep_size_help").into());
    check(ui, &mut st.crop_to_page, "jpeg.crop_to_page");
}
fn estimated_download_seconds(bytes: Option<usize>, bps: f64) -> Option<f64> {
    if !bps.is_finite() || bps <= 0.0 {
        return None;
    }
    bytes.map(|bytes| bytes as f64 * 8.0 / bps)
}
fn speed_controls(ui: &mut Ui, cache: &mut PreviewCache) {
    ui.horizontal_wrapped(|ui| {
        ui.label(tr("jpeg.speed"));
        let label = if cache.download_bps == 56_000.0 {
            tr("jpeg.modem_56")
        } else {
            format!("{:.0} kbps", cache.download_bps / 1000.0)
        };
        super::keynav::ComboBox::from_id_salt("jpeg_download_speed")
            .selected_text(label)
            .width(210.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut cache.download_bps, 56_000.0, tr("jpeg.modem_56"));
                for (bps, label) in [
                    (1_000_000.0, "1 Mbps"),
                    (10_000_000.0, "10 Mbps"),
                    (100_000_000.0, "100 Mbps"),
                ] {
                    ui.selectable_value(&mut cache.download_bps, bps, label);
                }
            })
            .response
            .rich_tip(super::tips::keyed("jpeg.speed"));
        let mut kbps = cache.download_bps / 1000.0;
        let response = ui.add(
            super::field::NumField::new(&mut kbps)
                .range(1.0..=10_000_000.0)
                .max_decimals(1)
                .suffix(" kbps")
                .width(95.0),
        );
        if response.changed() && kbps.is_finite() {
            cache.download_bps = kbps * 1000.0;
        }
        response.rich_tip(super::tips::keyed("jpeg.speed"));
        let seconds =
            estimated_download_seconds(cache.panes[cache.active].bytes, cache.download_bps);
        let time = seconds.map_or_else(|| "—".to_owned(), |value| format!("{value:.2} s"));
        ui.label(trf("jpeg.transfer_time", &[("time", &time)]));
    });
}
fn jpeg_ok_cancel(ui: &mut Ui, close: &mut bool) -> bool {
    let gaps = 10.0 + 2.0 * ui.spacing().item_spacing.x;
    let left = (ui.available_width() - super::chrome::row_width(2) - gaps).max(0.0);
    let ok_label = tr("jpeg.ok");
    let cancel_label = tr("dialog.cancel");
    let picked = super::chrome::button_row_after(
        ui,
        &[ok_label.as_str(), cancel_label.as_str()],
        false,
        left,
    );
    let mut accepted = picked == Some(0);
    if picked.is_some() {
        *close = true;
    }
    if !accepted && dialogs::enter_pressed(ui) {
        accepted = true;
        *close = true;
    }
    accepted
}
fn pane_rects(rect: egui::Rect, layout: usize) -> Vec<egui::Rect> {
    let (cols, rows) = match layout {
        1 => (2, 1),
        2 => (1, 2),
        3 => (2, 2),
        _ => (1, 1),
    };
    let gap = 5.0;
    let size = Vec2::new(
        (rect.width() - gap * (cols - 1) as f32) / cols as f32,
        (rect.height() - gap * (rows - 1) as f32) / rows as f32,
    );
    (0..rows)
        .flat_map(|row| {
            (0..cols).map(move |col| {
                egui::Rect::from_min_size(
                    rect.min + Vec2::new(col as f32 * (size.x + gap), row as f32 * (size.y + gap)),
                    size,
                )
            })
        })
        .collect()
}
fn paint_pane(ui: &mut Ui, rect: egui::Rect, pane: &mut Pane, active: bool, index: usize) -> bool {
    let response = ui.interact(
        rect,
        ui.id().with(("jpeg_pane", index)),
        egui::Sense::click_and_drag(),
    );
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, crate::theme::Tokens::panel_dark());
    if response.dragged() {
        pane.pan += response.drag_delta();
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            if pane.detail {
                pane.detail_scale = (pane.detail_scale * (scroll * 0.004).exp()).clamp(0.25, 64.0);
            } else {
                pane.zoom = (pane.zoom * (scroll * 0.004).exp()).clamp(0.05, 64.0);
            }
        }
    }
    if let Some(texture) = pane
        .original_texture
        .as_ref()
        .filter(|_| pane.show_original)
        .or(pane.texture.as_ref())
    {
        let fit = ((rect.width() - 12.0) / pane.output_size.0.max(1) as f32)
            .min((rect.height() - 30.0) / pane.output_size.1.max(1) as f32)
            .max(0.00001);
        let scale = if pane.detail {
            pane.detail_scale
        } else {
            fit * pane.zoom
        };
        let full_size = Vec2::new(
            pane.output_size.0.max(1) as f32,
            pane.output_size.1.max(1) as f32,
        ) * scale;
        let full_origin = rect.center() + pane.pan - full_size * 0.5;
        if pane.detail {
            pane.region_origin = (
                ((rect.left() - full_origin.x) / scale)
                    .floor()
                    .clamp(0.0, (pane.output_size.0.max(1) - 1) as f32) as u32,
                ((rect.top() - full_origin.y) / scale)
                    .floor()
                    .clamp(0.0, (pane.output_size.1.max(1) - 1) as f32) as u32,
            );
            pane.region_size = (
                (rect.width() / scale).ceil().clamp(1.0, 2048.0) as u32,
                (rect.height() / scale).ceil().clamp(1.0, 2048.0) as u32,
            );
        }
        let image = if let Some(origin) = pane.shown_origin {
            egui::Rect::from_min_size(
                full_origin + Vec2::new(origin.0 as f32, origin.1 as f32) * scale,
                Vec2::new(pane.size.0 as f32, pane.size.1 as f32) * scale,
            )
        } else {
            egui::Rect::from_min_size(full_origin, full_size)
        };
        painter.image(
            texture.id(),
            image,
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        let label = format!(
            "{} × {} px · {}",
            pane.output_size.0,
            pane.output_size.1,
            pane.bytes
                .map(crate::export::human_size)
                .unwrap_or_default()
        );
        painter.text(
            rect.left_bottom() + Vec2::new(8.0, -7.0),
            egui::Align2::LEFT_BOTTOM,
            label,
            egui::FontId::proportional(11.0),
            crate::theme::Tokens::text(),
        );
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            tr("jpeg.preview_pending"),
            egui::FontId::proportional(13.0),
            crate::theme::Tokens::text_dim(),
        );
    }
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(
            if active { 2.0 } else { 1.0 },
            if active {
                ui.visuals().selection.bg_fill
            } else {
                crate::theme::Tokens::window_border()
            },
        ),
        egui::StrokeKind::Inside,
    );
    response
        .clone()
        .rich_tip(super::tips::keyed("jpeg.preview_pan"));
    response.clicked()
}

pub(crate) fn show(app: &mut App, ctx: &Context, st: &mut ExportState, close: &mut bool) {
    let shared = cache(ctx);
    let mut cache = shared.lock().unwrap_or_else(|e| e.into_inner());
    let source = document_key(app);
    if cache.document_key != source {
        cancel(&mut cache);
        cache.document_key = source;
        cache.panes = vec![Pane::default(); 4];
        cache.active = 0;
        cache.requested_key = None;
    }
    update_preview(app, ctx, st, &mut cache);
    let screen = (ctx.content_rect().size() - Vec2::splat(24.0)).max(Vec2::splat(180.0));
    super::chrome::window(tr("jpeg.title"))
        .resizable(
            Vec2::new(1120.0, 760.0).min(screen),
            Vec2::new(380.0, 280.0).min(screen),
        )
        .show(ctx, |ui| {
            let room = ui.available_size();
            let gap = 12.0;
            let stacked = room.x < 660.0;
            let panel_width = if stacked { room.x } else { 330.0 };
            let height = (room.y - 48.0).max(90.0);
            let preview_height = if stacked {
                (height * 0.42)
                    .max(166.0)
                    .min((height - gap - 40.0).max(70.0))
            } else {
                height
            };
            let panel_height = if stacked {
                (height - preview_height - gap).max(40.0)
            } else {
                height
            };
            let layout = if stacked {
                egui::Layout::top_down(egui::Align::Min)
            } else {
                egui::Layout::left_to_right(egui::Align::Min)
            };
            let (content_rect, _) =
                ui.allocate_exact_size(Vec2::new(room.x, height), egui::Sense::hover());
            ui.scope_builder(
                egui::UiBuilder::new().max_rect(content_rect).layout(layout),
                |ui| {
                    ui.set_clip_rect(ui.clip_rect().intersect(content_rect));
                    let preview_width = if stacked {
                        room.x
                    } else {
                        (room.x - panel_width - gap).max(100.0)
                    };
                    ui.allocate_ui_with_layout(
                        Vec2::new(preview_width, preview_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            ui.horizontal_wrapped(|ui| {
                                let active = cache.active;
                                let pane = &mut cache.panes[active];
                                ui.add_enabled_ui(pane.original_texture.is_some(), |ui| {
                                    ui.selectable_value(
                                        &mut pane.show_original,
                                        true,
                                        tr("jpeg.before"),
                                    )
                                    .rich_tip(super::tips::keyed("jpeg.before"));
                                    ui.selectable_value(
                                        &mut pane.show_original,
                                        false,
                                        tr("jpeg.after"),
                                    )
                                    .rich_tip(super::tips::keyed("jpeg.after"));
                                });
                                if ui
                                    .small_button(tr("jpeg.fit"))
                                    .rich_tip(super::tips::keyed("jpeg.fit"))
                                    .clicked()
                                {
                                    pane.zoom = 1.0;
                                    pane.detail = false;
                                    pane.pan = Vec2::ZERO;
                                }
                                if ui
                                    .small_button("1:1")
                                    .rich_tip(super::tips::keyed("jpeg.actual_pixels"))
                                    .clicked()
                                {
                                    pane.detail = true;
                                    pane.detail_scale = 1.0 / ctx.pixels_per_point();
                                    pane.pan = Vec2::ZERO;
                                }
                                if ui
                                    .small_button("−")
                                    .rich_tip(super::tips::keyed("jpeg.zoom_out"))
                                    .clicked()
                                {
                                    if pane.detail {
                                        pane.detail_scale = (pane.detail_scale / 1.25).max(0.25);
                                    } else {
                                        pane.zoom = (pane.zoom / 1.25).max(0.05);
                                    }
                                }
                                if ui
                                    .small_button("+")
                                    .rich_tip(super::tips::keyed("jpeg.zoom_in"))
                                    .clicked()
                                {
                                    if pane.detail {
                                        pane.detail_scale = (pane.detail_scale * 1.25).min(64.0);
                                    } else {
                                        pane.zoom = (pane.zoom * 1.25).min(64.0);
                                    }
                                }
                                ui.label(format!(
                                    "{:.0} %",
                                    if pane.detail {
                                        pane.detail_scale * ctx.pixels_per_point() * 100.0
                                    } else {
                                        pane.zoom * 100.0
                                    }
                                ));
                                for (value, label) in [(0, "1"), (1, "2 ↔"), (2, "2 ↕"), (3, "4")]
                                {
                                    if ui
                                        .selectable_value(&mut cache.layout, value, label)
                                        .rich_tip(super::tips::keyed("jpeg.compare_views"))
                                        .changed()
                                    {
                                        let pane = cache.panes[cache.active].clone();
                                        for other in &mut cache.panes {
                                            if other.choices.is_none() {
                                                *other = pane.clone();
                                                other.choices = Some(
                                                    pane.choices
                                                        .clone()
                                                        .unwrap_or_else(|| st.clone()),
                                                );
                                            }
                                        }
                                        if cache.active >= visible_panes(cache.layout) {
                                            let previous = cache.active;
                                            cache.panes[previous].choices = Some(st.clone());
                                            cache.active = 0;
                                            if let Some(choices) = cache.panes[0].choices.clone() {
                                                *st = choices;
                                            }
                                            cache.requested_key = None;
                                        }
                                    }
                                }
                            });
                            let speed_height = if preview_width < 650.0 { 52.0 } else { 28.0 };
                            let (rect, _) = ui.allocate_exact_size(
                                Vec2::new(
                                    preview_width,
                                    (preview_height - 55.0 - speed_height).max(24.0),
                                ),
                                egui::Sense::hover(),
                            );
                            let mut selected = None;
                            let rectangles = pane_rects(rect, cache.layout);
                            #[cfg(test)]
                            ctx.data_mut(|d| {
                                d.insert_temp(
                                    egui::Id::new("jpeg_test_pane_rects"),
                                    rectangles.clone(),
                                )
                            });
                            for (index, rect) in rectangles.into_iter().enumerate() {
                                let active = cache.active == index;
                                if paint_pane(ui, rect, &mut cache.panes[index], active, index) {
                                    selected = Some(index);
                                }
                            }
                            if let Some(index) = selected.filter(|i| *i != cache.active) {
                                let previous = cache.active;
                                cache.panes[previous].choices = Some(st.clone());
                                if let Some(choices) = cache.panes[index].choices.clone() {
                                    *st = choices;
                                }
                                cache.active = index;
                                cache.requested_key = None;
                                cancel(&mut cache);
                            }
                            speed_controls(ui, &mut cache);
                            if cache.running.is_some() {
                                ui.label(tr("jpeg.preview_pending"));
                            }
                            if let Some(error) = cache.error.as_ref() {
                                ui.colored_label(ui.visuals().error_fg_color, error);
                            }
                        },
                    );
                    ui.add_space(gap);
                    ui.allocate_ui_with_layout(
                        Vec2::new(panel_width, panel_height),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .id_salt("jpeg_controls_scroll")
                                .max_height(panel_height)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width((panel_width - 16.0).max(170.0));
                                    dialogs::export_presets_row(app, ui, st);
                                    ui.label(tr("dialog.format"));
                                    super::keynav::ComboBox::from_id_salt("jpeg_export_format")
                                        .selected_text(dialogs::export_format_label(st.format))
                                        .show_ui(ui, |ui| {
                                            for entry in
                                                dialogs::export_format_order().into_iter().flatten()
                                            {
                                                if ui
                                                    .selectable_label(
                                                        st.format == entry,
                                                        dialogs::export_format_label(entry),
                                                    )
                                                    .clicked()
                                                {
                                                    app.recall_export_format(st, entry);
                                                }
                                            }
                                        })
                                        .response
                                        .rich_tip(super::tips::keyed("dialog.format"));
                                    egui::CollapsingHeader::new(tr("jpeg.settings"))
                                        .default_open(true)
                                        .show(ui, |ui| settings(ui, st))
                                        .header_response
                                        .rich_tip(super::tips::keyed("jpeg.settings"));
                                    let pane = &cache.panes[cache.active];
                                    egui::CollapsingHeader::new(tr("jpeg.advanced"))
                                        .default_open(true)
                                        .show(ui, |ui| advanced(ui, st, pane))
                                        .header_response
                                        .rich_tip(super::tips::keyed("jpeg.advanced"));
                                    let base = if pane.base_size_mm.0 > 0.0 {
                                        pane.base_size_mm
                                    } else {
                                        let b = if st.selection_only {
                                            app.selection_bounds().unwrap_or(app.page_rect())
                                        } else {
                                            app.page_rect()
                                        };
                                        (b.width(), b.height())
                                    };
                                    egui::CollapsingHeader::new(tr("jpeg.transforms"))
                                        .default_open(true)
                                        .show(ui, |ui| transforms(ui, st, base))
                                        .header_response
                                        .rich_tip(super::tips::keyed("jpeg.transforms"));
                                    check(ui, &mut st.selection_only, "dialog.selected_only");
                                    if !st.selection_only {
                                        dialogs::export_range_fields(app, ui, st);
                                    }
                                });
                        },
                    );
                },
            );
            ui.separator();
            ui.horizontal(|ui| {
                if jpeg_ok_cancel(ui, close) {
                    dialogs::export_with_file_dialog(app, st);
                    app.keep_export_choices(st);
                }
            });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        *close = true;
    }
    if cache.requested_key != Some(preview_key(app, st, &cache.panes[cache.active])) {
        ctx.request_repaint();
    }
    if *close || st.format != 3 {
        cancel(&mut cache);
        cache.requested_key = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(
        ctx: &Context,
        app: &mut App,
        st: &mut ExportState,
        size: Vec2,
        events: Vec<egui::Event>,
    ) -> (egui::FullOutput, bool) {
        let mut closed = false;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ui| show(app, ui.ctx(), st, &mut closed),
        );
        // These interaction fixtures inspect shapes and cached texture handles;
        // the separate profile capture applies the actual pixel deltas.
        out.textures_delta.clear();
        (out, closed)
    }
    fn text_shapes(shape: &egui::Shape, out: &mut Vec<(String, egui::Rect)>) {
        match shape {
            egui::Shape::Text(text) => out.push((
                text.galley.text().to_string(),
                egui::Rect::from_min_size(text.pos, text.galley.size()),
            )),
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    text_shapes(shape, out)
                }
            }
            _ => {}
        }
    }
    fn painted(out: &egui::FullOutput) -> Vec<(String, egui::Rect)> {
        let mut result = Vec::new();
        for clipped in &out.shapes {
            text_shapes(&clipped.shape, &mut result);
        }
        result
    }
    fn at(out: &egui::FullOutput, label: &str) -> egui::Pos2 {
        painted(out)
            .iter()
            .find(|(text, _)| text == label)
            .unwrap_or_else(|| panic!("missing {label}: {:?}", painted(out)))
            .1
            .center()
    }
    fn click(
        ctx: &Context,
        app: &mut App,
        st: &mut ExportState,
        size: Vec2,
        pos: egui::Pos2,
    ) -> (egui::FullOutput, bool) {
        let mut last = None;
        for pressed in [true, false] {
            last = Some(frame(
                ctx,
                app,
                st,
                size,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            ));
        }
        last.unwrap()
    }
    fn open_combo(
        ctx: &Context,
        app: &mut App,
        st: &mut ExportState,
        size: Vec2,
        previous: &egui::FullOutput,
        selected: &str,
        row: &str,
    ) -> egui::FullOutput {
        let pos = at(previous, selected);
        frame(ctx, app, st, size, vec![egui::Event::PointerMoved(pos)]);
        let mut out = click(ctx, app, st, size, pos).0;
        assert!(
            egui::Popup::is_any_open(ctx),
            "the real {selected} combo click must open its popup"
        );
        // A newly opened egui Area first measures itself invisibly. Keep
        // drawing actual frames, as the window does, before locating a row.
        for _ in 0..4 {
            if painted(&out).iter().any(|(text, _)| text == row) {
                return out;
            }
            out = frame(ctx, app, st, size, Vec::new()).0;
        }
        panic!(
            "open combo {selected} never painted {row}: {:?}",
            painted(&out)
        );
    }
    fn fixture() -> (App, ExportState) {
        let mut app = App::headless();
        let mut doc =
            tracedraw_core::Document::new("JPEG UI fixture", tracedraw_core::Size::new(25.4, 12.7));
        let mut shape = tracedraw_core::Shape::new(
            doc.ids_mut().shape(),
            tracedraw_core::ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(0.0, 0.0, 25.4, 12.7),
                radius: 0.0,
                corners: None,
            },
        );
        shape.fill = tracedraw_core::Fill::Solid(tracedraw_core::Color::rgb8(220, 30, 20));
        shape.stroke = None;
        doc.pages[0].layers[0].shapes.push(shape);
        app.engine.replace(doc);
        app.page = app.doc().pages[0].id;
        (
            app,
            ExportState {
                format: 3,
                dpi: 72.0,
                size_unit: 2,
                ..Default::default()
            },
        )
    }
    #[test]
    fn jpeg_transformations_keep_pixels_or_physical_size_and_unlock_axes() {
        let mut st = ExportState {
            format: 3,
            dpi: 300.0,
            size_unit: 0,
            ..Default::default()
        };
        let base = (254.0, 127.0);
        assert_eq!(displayed_size(base, &st), (3000.0, 1500.0));
        st.keep_size = true;
        change_dpi(&mut st, 600.0);
        assert_eq!(displayed_size(base, &st), (3000.0, 1500.0));
        st.size_unit = 2;
        assert_eq!(displayed_size(base, &st), (12.7, 6.35));
        st.keep_size = false;
        change_dpi(&mut st, 300.0);
        assert_eq!(displayed_size(base, &st), (12.7, 6.35));
        change_scale(&mut st, false, 1.5);
        assert_eq!(st.scale_y, 1.5);
        st.keep_ratio = false;
        change_scale(&mut st, true, 2.0);
        assert_eq!(displayed_size(base, &st), (38.1, 25.4));
        let before = st.clone();
        change_dpi(&mut st, f64::NAN);
        assert_eq!(st, before);
        typed_size(&mut st, 254.0, false, 30.0);
        assert!((st.scale - 300.0 / 254.0).abs() < 1e-10);
        assert_eq!(st.scale_y, 2.0);
        st.size_unit = 0;
        st.dpi = 300.0;
        typed_size(&mut st, 210.0, false, 1001.0);
        assert_eq!(
            displayed_size((210.0, 297.0), &st).0,
            1001.0,
            "a typed integer pixel width must not round up to the next pixel"
        );
        crate::i18n::with_thread_language("es", || {
            assert_eq!(crate::i18n::language(), "es");
            let ctx = crate::theme::ui_context();
            let base = (573.4, 200.0);
            let mut localized = ExportState {
                format: 3,
                size_unit: 2,
                keep_ratio: false,
                ..Default::default()
            };
            let before = localized.clone();
            let draw = |state: &mut ExportState, events: Vec<egui::Event>| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1000.0, 600.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| transforms(ui, state, base),
                );
                output.textures_delta.clear();
                output
            };
            let mut output = draw(&mut localized, Vec::new());
            for _ in 0..3 {
                output = draw(&mut localized, Vec::new());
            }
            let position = at(&output, "57,34 cm");
            assert!(painted(&output)
                .iter()
                .any(|(text, _)| text == "centímetros"));
            draw(&mut localized, vec![egui::Event::PointerMoved(position)]);
            for pressed in [true, false] {
                draw(
                    &mut localized,
                    vec![egui::Event::PointerButton {
                        pos: position,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    }],
                );
            }
            draw(
                &mut localized,
                vec![
                    egui::Event::ModifiersChanged(egui::Modifiers::COMMAND),
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::COMMAND,
                    },
                ],
            );
            draw(
                &mut localized,
                vec![
                    egui::Event::ModifiersChanged(egui::Modifiers::NONE),
                    egui::Event::Key {
                        key: egui::Key::A,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::Text("1in".into()),
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            let output = draw(&mut localized, Vec::new());
            assert!((displayed_size(base, &localized).0 - 2.54).abs() < 1e-9);
            assert!((localized.scale - 25.4 / base.0).abs() < 1e-9);
            assert!(painted(&output).iter().any(|(text, _)| text == "2,54 cm"));
            assert_eq!(localized.scale_y, before.scale_y);
            assert_eq!(
                (localized.dpi, localized.quality, localized.color_mode),
                (before.dpi, before.quality, before.color_mode)
            );
            crate::i18n::with_thread_language("en", || {
                assert_eq!(crate::i18n::language(), "en");
                let output = draw(&mut localized, Vec::new());
                assert!(painted(&output).iter().any(|(text, _)| text == "2.54 cm"));
            });
            assert_eq!(crate::i18n::language(), "es");
        });
    }
    #[test]
    fn jpeg_options_survive_presets_json_and_old_settings() {
        assert_eq!(profile_size(3144), "3.07 KB");
        let st = ExportState {
            format: 3,
            quality: 43,
            dpi: 173.0,
            size_unit: 2,
            scale: 1.4,
            scale_y: 0.7,
            keep_ratio: false,
            keep_size: true,
            crop_to_page: false,
            jpeg_subsampling: 2,
            jpeg_progressive: true,
            jpeg_optimize_huffman: true,
            embed_icc: false,
            matte: [22, 33, 44],
            anti_alias: false,
            blur: 1.7,
            simulate_overprints: true,
            overprint_black: true,
            use_proof_profile: true,
            ..Default::default()
        };
        let preset = crate::prefs::ExportPreset::from_state("Client print", &st);
        let encoded = serde_json::to_string(&preset).unwrap();
        let restored: crate::prefs::ExportPreset = serde_json::from_str(&encoded).unwrap();
        let mut out = ExportState::default();
        restored.apply(&mut out);
        assert_eq!(
            crate::settings::JpegExportPrefs::from_state(&out),
            crate::settings::JpegExportPrefs::from_state(&st)
        );
        assert_eq!(
            (
                out.dpi,
                out.quality,
                out.scale,
                out.color_mode,
                out.size_unit
            ),
            (173.0, 43, 1.4, 0, 2)
        );
        let mut prefs = crate::prefs::FormatPrefs::default();
        prefs.remember(&st);
        let json = serde_json::to_string(&prefs).unwrap();
        let loaded: crate::prefs::FormatPrefs = serde_json::from_str(&json).unwrap();
        assert_eq!(
            crate::settings::JpegExportPrefs::from_state(&loaded.state()),
            crate::settings::JpegExportPrefs::from_state(&st)
        );
        let old: crate::prefs::FormatPrefs =
            serde_json::from_str(r#"{"export_format":3,"export_quality":67}"#).unwrap();
        assert_eq!(old.state().quality, 67);
        assert!(old.state().anti_alias && old.state().embed_icc && old.state().keep_ratio);
        assert_eq!(old.state().scale_y, 1.0);
        let minimum = ExportState {
            quality: 0,
            ..st.clone()
        };
        let minimum_preset = crate::prefs::ExportPreset::from_state("JPEG minimum", &minimum);
        let mut minimum_out = ExportState::default();
        minimum_preset.apply(&mut minimum_out);
        assert_eq!(minimum_out.quality, 0);
        let mut minimum_prefs = crate::prefs::FormatPrefs::default();
        minimum_prefs.remember(&minimum);
        let json = serde_json::to_string(&minimum_prefs).unwrap();
        let loaded: crate::prefs::FormatPrefs = serde_json::from_str(&json).unwrap();
        assert_eq!(
            loaded.state().quality,
            0,
            "the JPEG minimum must survive a settings roundtrip"
        );
        let other = ExportState {
            format: 0,
            quality: 0,
            ..Default::default()
        };
        crate::prefs::ExportPreset::from_state("Other format", &other).apply(&mut minimum_out);
        assert_eq!(
            minimum_out.quality, 1,
            "the existing range of other formats stays unchanged"
        );
        minimum_prefs.remember(&other);
        assert_eq!(minimum_prefs.state().quality, 1);
        crate::i18n::with_thread_language("es", || {
            assert_eq!(quality_label(50), "Medio");
            assert_eq!(quality_label(75), "75 %");
            assert_eq!(quality_label(90), "90 %");
        });
    }
    #[test]
    fn jpeg_download_speed_uses_real_events_and_estimates_only_the_active_view() {
        crate::i18n::with_thread_language("en", || {
            let ctx = crate::theme::ui_context();
            let mut state = PreviewCache::default();
            state.requested_key = Some(9);
            state.panes[0].key = Some(9);
            state.panes[0].bytes = Some(7_000);
            state.panes[1].bytes = Some(14_000);
            let draw = |state: &mut PreviewCache, events: Vec<egui::Event>| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(900.0, 260.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| speed_controls(ui, state),
                );
                output.textures_delta.clear();
                output
            };
            let click_speed = |state: &mut PreviewCache, pos: egui::Pos2| {
                draw(state, vec![egui::Event::PointerMoved(pos)]);
                let mut output = None;
                for pressed in [true, false] {
                    output = Some(draw(
                        state,
                        vec![egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::NONE,
                        }],
                    ));
                }
                output.unwrap()
            };
            let mut output = draw(&mut state, Vec::new());
            for _ in 0..3 {
                output = draw(&mut state, Vec::new());
            }
            assert_eq!(state.download_bps, 56_000.0);
            assert_eq!(
                estimated_download_seconds(Some(7_000), state.download_bps),
                Some(1.0)
            );
            assert!(painted(&output)
                .iter()
                .any(|(text, _)| { text == &trf("jpeg.transfer_time", &[("time", "1.00 s")]) }));
            output = click_speed(&mut state, at(&output, &tr("jpeg.modem_56")));
            assert!(egui::Popup::is_any_open(&ctx));
            for _ in 0..4 {
                if painted(&output).iter().any(|(text, _)| text == "1 Mbps") {
                    break;
                }
                output = draw(&mut state, Vec::new());
            }
            click_speed(&mut state, at(&output, "1 Mbps"));
            let output = draw(&mut state, Vec::new());
            assert_eq!(state.download_bps, 1_000_000.0);
            assert!(
                (estimated_download_seconds(Some(7_000), state.download_bps).unwrap() - 0.056)
                    .abs()
                    < 1e-12
            );
            assert!(painted(&output)
                .iter()
                .any(|(text, _)| { text == &trf("jpeg.transfer_time", &[("time", "0.06 s")]) }));
            assert_eq!(state.requested_key, Some(9));
            assert_eq!(
                state.panes[0].key,
                Some(9),
                "speed must not invalidate or recode the image"
            );
            state.active = 1;
            let output = draw(&mut state, Vec::new());
            assert!(painted(&output)
                .iter()
                .any(|(text, _)| { text == &trf("jpeg.transfer_time", &[("time", "0.11 s")]) }));
            assert_eq!(estimated_download_seconds(None, 56_000.0), None);
            assert_eq!(estimated_download_seconds(Some(7_000), f64::NAN), None);
        });
    }
    #[test]
    fn jpeg_real_controls_reach_the_encoded_file() {
        crate::i18n::with_thread_language("en", || {
            let (mut app, mut st) = fixture();
            let profile_ctx = crate::theme::ui_context();
            let missing_profile = Pane {
                key: Some(1),
                profile_name: "Generic Gray".into(),
                profile_bytes: 0,
                ..Default::default()
            };
            let mut profile_out = profile_ctx.run_ui(egui::RawInput::default(), |ui| {
                advanced(ui, &mut st, &missing_profile);
            });
            profile_out.textures_delta.clear();
            let labels = painted(&profile_out);
            assert!(labels
                .iter()
                .any(|(text, _)| text == &tr("jpeg.no_profile")));
            assert!(labels
                .iter()
                .all(|(text, _)| !text.contains("Generic Gray")));
            let old = app.doc().clone();
            let ctx = crate::theme::ui_context();
            let size = Vec2::new(1280.0, 900.0);
            let mut out = frame(&ctx, &mut app, &mut st, size, Vec::new()).0;
            for _ in 0..3 {
                out = frame(&ctx, &mut app, &mut st, size, Vec::new()).0;
            }
            let pos = at(&out, &tr("jpeg.progressive"));
            out = click(&ctx, &mut app, &mut st, size, pos).0;
            assert!(
                st.jpeg_progressive,
                "real progressive checkbox changes output options"
            );
            out = open_combo(
                &ctx,
                &mut app,
                &mut st,
                size,
                &out,
                &tr("jpeg.rgb"),
                &tr("jpeg.gray"),
            );
            let pos = at(&out, &tr("jpeg.gray"));
            click(&ctx, &mut app, &mut st, size, pos);
            assert_eq!(st.color_mode, 2, "the real list selects grayscale");
            let path = std::env::temp_dir().join(format!(
                "craft-jpeg-ui-{}-{}.jpg",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            crate::export::prepare_job(&app, &path, &st)
                .unwrap()
                .run(None, Arc::new(AtomicBool::new(false)))
                .unwrap();
            let bytes = std::fs::read(&path).unwrap();
            std::fs::remove_file(&path).unwrap();
            let mut offset = 2;
            let mut sof = None;
            while offset + 3 < bytes.len() {
                assert_eq!(bytes[offset], 0xff);
                let marker = bytes[offset + 1];
                if marker == 0xc0 || marker == 0xc2 {
                    sof = Some((marker, bytes[offset + 9]));
                    break;
                }
                if marker == 0xda || marker == 0xd9 {
                    break;
                }
                let length = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;
                offset += 2 + length;
            }
            assert_eq!(
                sof,
                Some((0xc2, 1)),
                "checkbox/list choices must produce progressive one-channel JPEG"
            );
            let decoded = image::load_from_memory(&bytes).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (72, 36));
            assert_eq!(app.doc(), &old);
            let shared = cache(&ctx);
            let mut cache = shared.lock().unwrap_or_else(|e| e.into_inner());
            cancel(&mut cache);
        });
    }
    #[test]
    fn jpeg_dialog_stays_within_small_and_large_viewports() {
        crate::i18n::with_thread_language("en", || {
            for size in [
                Vec2::new(1280.0, 900.0),
                Vec2::new(800.0, 600.0),
                Vec2::new(640.0, 360.0),
                Vec2::new(390.0, 844.0),
            ] {
                let (mut app, mut st) = fixture();
                let ctx = crate::theme::ui_context();
                let mut out = frame(&ctx, &mut app, &mut st, size, Vec::new()).0;
                for _ in 0..3 {
                    out = frame(&ctx, &mut app, &mut st, size, Vec::new()).0;
                }
                let id = egui::Id::new(("chrome_dialog", tr("jpeg.title")));
                let rect = ctx.memory(|m| m.area_rect(id)).unwrap();
                assert!(
                    rect.left() >= -0.5
                        && rect.top() >= -0.5
                        && rect.right() <= size.x + 0.5
                        && rect.bottom() <= size.y + 0.5,
                    "{size:?}: {rect:?}"
                );
                assert!(painted(&out)
                    .iter()
                    .any(|(text, _)| text == &tr("dialog.cancel")));
                let point = at(&out, &tr("dialog.cancel"));
                assert!(egui::Rect::from_min_size(egui::Pos2::ZERO, size).contains(point));
                assert!(
                    click(&ctx, &mut app, &mut st, size, point).1,
                    "Cancel remains reachable at {size:?}"
                );
                let shared = cache(&ctx);
                let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                cancel(&mut state);
            }
            crate::i18n::with_thread_language("es", || {
                let (mut app, mut st) = fixture();
                let ctx = crate::theme::ui_context();
                let mut output = frame(
                    &ctx,
                    &mut app,
                    &mut st,
                    Vec2::new(1280.0, 900.0),
                    Vec::new(),
                )
                .0;
                for _ in 0..3 {
                    output = frame(
                        &ctx,
                        &mut app,
                        &mut st,
                        Vec2::new(1280.0, 900.0),
                        Vec::new(),
                    )
                    .0;
                }
                let labels = painted(&output);
                assert!(labels.iter().any(|(text, _)| text == "OK"));
                assert!(!labels.iter().any(|(text, _)| text == "Aceptar"));
                assert!(labels.iter().any(|(text, _)| text == "centímetros"));
                cancel_diagnostic_previews(&ctx);
            });
        });
    }
    #[test]
    fn cancelling_the_preview_via_egui_signals_its_worker_without_changing_the_document() {
        crate::i18n::with_thread_language("en", || {
            let (mut app, mut st) = fixture();
            let old = app.doc().clone();
            let history = app.engine.history_labels();
            let ctx = crate::theme::ui_context();
            let size = Vec2::new(1280.0, 900.0);
            let flag = Arc::new(AtomicBool::new(false));
            let worker_flag = flag.clone();
            let (release, wait) = mpsc::channel();
            let (sender, receiver) = mpsc::channel();
            let worker = std::thread::spawn(move || {
                wait.recv().unwrap();
                assert!(worker_flag.load(Ordering::Relaxed));
                sender
                    .send(Err::<crate::export::ExportPreviewResult, _>(
                        "cancelled".into(),
                    ))
                    .unwrap();
            });
            let shared = cache(&ctx);
            {
                let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                state.document_key = document_key(&app);
                let key = preview_key(&app, &st, &state.panes[0]);
                state.requested_key = Some(key);
                state.running = Some(Running {
                    pane: 0,
                    key,
                    cancel: flag.clone(),
                    receiver,
                    origin: None,
                    choices: st.clone(),
                });
            }
            let mut out = frame(&ctx, &mut app, &mut st, size, Vec::new()).0;
            for _ in 0..3 {
                out = frame(&ctx, &mut app, &mut st, size, Vec::new()).0;
            }
            assert!(
                !flag.load(Ordering::Relaxed),
                "the worker must remain live until the actual Cancel click"
            );
            let pos = at(&out, &tr("dialog.cancel"));
            assert!(click(&ctx, &mut app, &mut st, size, pos).1);
            assert!(flag.load(Ordering::Relaxed));
            release.send(()).unwrap();
            worker.join().unwrap();
            assert_eq!(app.doc(), &old);
            assert_eq!(app.engine.history_labels(), history);
            assert!(app.file.is_none());
            let (_, closed) = frame(&ctx, &mut app, &mut st, size, Vec::new());
            assert!(!closed);
        });
    }
    #[test]
    fn closing_the_jpeg_title_bar_cancels_its_retained_preview_and_preserves_the_document() {
        crate::i18n::with_thread_language("en", || {
            let (mut app, st) = fixture();
            let original = app.doc().clone();
            let history = app.engine.history_labels();
            let selection = app.selection.clone();
            let ctx = crate::theme::ui_context();
            let flag = Arc::new(AtomicBool::new(false));
            let worker_flag = flag.clone();
            let (release, held) = mpsc::channel();
            let (sender, receiver) = mpsc::channel();
            let worker = std::thread::spawn(move || {
                if held.recv().is_err() {
                    return false;
                }
                let cancelled = worker_flag.load(Ordering::Relaxed);
                let _ = sender.send(Err::<crate::export::ExportPreviewResult, String>(
                    "cancelled".into(),
                ));
                cancelled
            });
            let shared = cache(&ctx);
            {
                let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
                state.document_key = document_key(&app);
                let key = preview_key(&app, &st, &state.panes[0]);
                state.requested_key = Some(key);
                state.running = Some(Running {
                    pane: 0,
                    key,
                    cancel: flag.clone(),
                    receiver,
                    origin: None,
                    choices: st.clone(),
                });
            }
            app.dialog = dialogs::Dialog::Export(st);
            let draw = |app: &mut App, events: Vec<egui::Event>| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1280.0, 900.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| dialogs::show(app, ui.ctx()),
                );
                output.textures_delta.clear();
            };
            for _ in 0..4 {
                draw(&mut app, Vec::new());
            }
            assert!(
                !flag.load(Ordering::Relaxed),
                "the retained worker must be live before X"
            );
            let window_id = egui::Id::new(("chrome_dialog", tr("jpeg.title")));
            let name = format!("{window_id:?}");
            let close_id = egui::Id::new((name.as_str(), "close"));
            let point = ctx
                .viewport(|viewport| {
                    for (_, widgets) in viewport.prev_pass.widgets.layers() {
                        for widget in widgets {
                            if widget.id == close_id {
                                return Some(widget.interact_rect.center());
                            }
                        }
                    }
                    None
                })
                .expect("the actual JPEG title-bar X is drawn");
            draw(&mut app, vec![egui::Event::PointerMoved(point)]);
            for pressed in [true, false] {
                draw(
                    &mut app,
                    vec![egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    }],
                );
            }
            assert!(matches!(app.dialog, dialogs::Dialog::None));
            assert!(
                flag.load(Ordering::Relaxed),
                "X must signal the same cancellation as Cancel"
            );
            release.send(()).unwrap();
            assert!(worker.join().unwrap());
            assert_eq!(app.doc(), &original);
            assert_eq!(app.engine.history_labels(), history);
            assert_eq!(app.selection, selection);
            assert!(app.file.is_none());
            draw(&mut app, Vec::new());
            assert!(matches!(app.dialog, dialogs::Dialog::None));
        });
    }
    #[test]
    fn jpeg_comparison_views_keep_independent_choices_and_one_to_one_uses_native_tiles() {
        crate::i18n::with_thread_language("en", || {
            let (mut app, mut st) = fixture();
            let mut doc = app.doc().clone();
            doc.pages[0].size = tracedraw_core::Size::new(254.0, 127.0);
            app.engine.replace(doc);
            app.page = app.doc().pages[0].id;
            st.dpi = 300.0;
            let ctx = crate::theme::ui_context();
            let size = Vec2::new(1280.0, 900.0);
            let settle = |app: &mut App, st: &mut ExportState| {
                let deadline = Instant::now() + Duration::from_secs(15);
                loop {
                    let out = frame(&ctx, app, st, size, Vec::new()).0;
                    let shared = cache(&ctx);
                    let state = shared.lock().unwrap_or_else(|e| e.into_inner());
                    if state.running.is_none()
                        && state.panes[state.active].key.is_some()
                        && state.panes[state.active].texture.is_some()
                        && state.panes[state.active].key == state.requested_key
                    {
                        assert!(state.error.is_none(), "{:?}", state.error);
                        return out;
                    }
                    assert!(
                        Instant::now() < deadline,
                        "small real preview must complete"
                    );
                    drop(state);
                    std::thread::sleep(Duration::from_millis(10));
                }
            };
            let out = settle(&mut app, &mut st);
            click(&ctx, &mut app, &mut st, size, at(&out, "1:1"));
            settle(&mut app, &mut st);
            let shared = cache(&ctx);
            let first_origin = {
                let state = shared.lock().unwrap_or_else(|e| e.into_inner());
                let pane = &state.panes[0];
                assert!(pane.detail && pane.shown_origin.is_some());
                assert_eq!(pane.detail_scale, 1.0);
                assert_eq!(pane.output_size, (3000, 1500));
                assert!(pane.size.0 < 3000 && pane.size.0 <= 2048 && pane.size.1 <= 2048);
                pane.shown_origin.unwrap()
            };
            let rectangles = ctx
                .data(|d| d.get_temp::<Vec<egui::Rect>>(egui::Id::new("jpeg_test_pane_rects")))
                .unwrap();
            let start = rectangles[0].center();
            let end = start + Vec2::new(30.0, 0.0);
            frame(
                &ctx,
                &mut app,
                &mut st,
                size,
                vec![
                    egui::Event::PointerMoved(start),
                    egui::Event::PointerButton {
                        pos: start,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            frame(
                &ctx,
                &mut app,
                &mut st,
                size,
                vec![egui::Event::PointerMoved(end)],
            );
            frame(
                &ctx,
                &mut app,
                &mut st,
                size,
                vec![egui::Event::PointerButton {
                    pos: end,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            let out = settle(&mut app, &mut st);
            {
                let state = shared.lock().unwrap_or_else(|e| e.into_inner());
                assert!(state.panes[0].pan.x > 0.0);
                assert!(
                    state.panes[0].shown_origin.unwrap().0 < first_origin.0,
                    "pan must request a different native region"
                );
            }
            let out = click(&ctx, &mut app, &mut st, size, at(&out, "4")).0;
            let out = open_combo(
                &ctx,
                &mut app,
                &mut st,
                size,
                &out,
                "90 %",
                &tr("jpeg.medium"),
            );
            let pos = at(&out, &tr("jpeg.medium"));
            click(&ctx, &mut app, &mut st, size, pos);
            assert_eq!(st.quality, 50);
            let rectangles = ctx
                .data(|d| d.get_temp::<Vec<egui::Rect>>(egui::Id::new("jpeg_test_pane_rects")))
                .unwrap();
            assert_eq!(rectangles.len(), 4);
            click(&ctx, &mut app, &mut st, size, rectangles[1].center());
            assert_eq!(
                st.quality, 90,
                "the second view retains its independent original settings"
            );
            click(&ctx, &mut app, &mut st, size, rectangles[0].center());
            assert_eq!(
                st.quality, 50,
                "the first view retains the quality selected through its real list"
            );
            let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
            cancel(&mut state);
        });
    }
}
