//! CDR opening for the desktop UI: one worker, real progress and atomic delivery.
//! Automation continues to use App::try_open_path and its synchronous result.

use super::*;
use crate::color_policy::{prepare_open_policy, prepare_screen_colors, PreparedScreenColors};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CdrOpenMode {
    Open,
    Reopen,
    Template,
}

struct CdrOpenRequest {
    path: PathBuf,
    mode: CdrOpenMode,
}

pub(crate) struct CdrOpenJob {
    path: PathBuf,
    receiver: Receiver<Result<PreparedCdrOpen, String>>,
    progress: Arc<AtomicU8>,
    discarded: Arc<AtomicBool>,
    pending: VecDeque<CdrOpenRequest>,
    mode: CdrOpenMode,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "document_unit_tests/ruler_capture.rs"]
mod ui_capture;

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "cdr_open_tests.rs"]
mod tests;

struct PreparedCdrOpen {
    engine: Engine,
    report: tracedraw_cdr::ParseReport,
    colors: PreparedScreenColors,
    color_notice: Option<String>,
    color_error: Option<String>,
    thumbnail: Option<Vec<u8>>,
}

fn advance(progress: &AtomicU8, percent: u8) {
    // Only publication on the UI thread may set 100.
    progress.fetch_max(percent.min(99), Ordering::Relaxed);
}

fn same_open_file(a: &std::path::Path, b: &std::path::Path) -> bool {
    crate::files::same_path(&crate::files::canonical(a), &crate::files::canonical(b))
}

#[cfg(not(target_arch = "wasm32"))]
fn prepare_cdr_open(
    path: &std::path::Path,
    color: crate::settings::ColorPrefs,
    progress: Arc<AtomicU8>,
) -> Result<PreparedCdrOpen, String> {
    let started = web_time::Instant::now();
    let bytes = crate::files::read(path).map_err(|error| error.to_string())?;
    let title = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled");
    let reported = Arc::clone(&progress);
    let callback: tracedraw_cdr::OpenProgressCallback = Arc::new(move |event| {
        advance(&reported, (u16::from(event.percent) * 88 / 100) as u8);
    });
    let (mut document, report, raw) =
        tracedraw_cdr::open_bytes_keeping_with_progress(&bytes, title, Some(callback))
            .map_err(|error| error.to_string())?;
    // The editor historically discards RawTable. Release the inflated streams
    // and input before allocating the rendered thumbnail or preparing policy.
    drop(raw);
    drop(bytes);
    log::info!(
        "CDR read completed after {} ms",
        started.elapsed().as_millis()
    );
    for warning in &report.warnings {
        log::warn!("cdr: {warning}");
    }
    advance(&progress, 90);
    let (color_notice, color_error) = match prepare_open_policy(&document, &color) {
        Ok(prepared) => {
            if let Some(updated) = prepared.document {
                document = updated;
            }
            (prepared.notice, None)
        }
        // Opening previously kept the original document when its requested
        // profile could not be applied, reporting that failure on the status.
        Err(error) => (None, Some(error)),
    };
    advance(&progress, 93);
    let mut colors = prepare_screen_colors(&document, &color);
    let _ = tracedraw_text::fonts().add_document_fonts(&document.fonts);
    let engine = Engine::new(document);
    advance(&progress, 96);

    // A worker must not install a process-wide profile while another drawing
    // remains editable. The existing scoped converter restores even on unwind.
    let icc = colors.icc.take().map(Arc::new);
    let scoped_icc = icc.clone();
    let converter: tracedraw_core::color::engine::RgbConverter = std::rc::Rc::new(move |color| {
        let color = color.process();
        scoped_icc
            .as_ref()
            .and_then(|engine| engine.to_rgb_f32(color))
            .unwrap_or_else(|| color.to_rgb_f32_builtin())
    });
    let thumbnail = tracedraw_core::color::engine::with_rgb_converter(converter, || {
        crate::thumbnails::document_thumbnail_png(engine.document())
    });
    colors.icc = icc
        .map(|engine| {
            Arc::try_unwrap(engine)
                .map_err(|_| "thumbnail colour converter is still borrowed".to_string())
        })
        .transpose()?;
    advance(&progress, 99);
    log::info!(
        "CDR worker prepared engine, policy and thumbnail after {} ms",
        started.elapsed().as_millis()
    );
    Ok(PreparedCdrOpen {
        engine,
        report,
        colors,
        color_notice,
        color_error,
        thumbnail,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn spawn_cdr_open(
    path: PathBuf,
    color: crate::settings::ColorPrefs,
    context: egui::Context,
    load: impl FnOnce(
            &std::path::Path,
            crate::settings::ColorPrefs,
            Arc<AtomicU8>,
        ) -> Result<PreparedCdrOpen, String>
        + Send
        + 'static,
) -> Result<CdrOpenJob, String> {
    let progress = Arc::new(AtomicU8::new(0));
    let discarded = Arc::new(AtomicBool::new(false));
    let worker_progress = Arc::clone(&progress);
    let worker_discarded = Arc::clone(&discarded);
    let worker_path = path.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("cdr-open".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let mut result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                load(&worker_path, color, worker_progress)
            }))
            .unwrap_or_else(|_| Err("CDR opening worker failed unexpectedly".into()));
            if worker_discarded.load(Ordering::Relaxed) {
                // Release superseded document allocations on the worker.
                drop(result);
                result = Err("CDR opening request was superseded".into());
            }
            let _ = sender.send(result);
            context.request_repaint();
        })
        .map_err(|error| error.to_string())?;
    Ok(CdrOpenJob {
        path,
        receiver,
        progress,
        discarded,
        pending: VecDeque::new(),
        mode: CdrOpenMode::Open,
    })
}

impl App {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn open_cdr_in_background(&mut self, _path: PathBuf) -> bool {
        false
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn reopen_cdr_in_background(&mut self, _path: PathBuf, _template: bool) -> bool {
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn open_cdr_in_background(&mut self, path: PathBuf) -> bool {
        self.queue_cdr_open(path, CdrOpenMode::Open)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn reopen_cdr_in_background(&mut self, path: PathBuf, template: bool) -> bool {
        self.queue_cdr_open(
            path,
            if template {
                CdrOpenMode::Template
            } else {
                CdrOpenMode::Reopen
            },
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn queue_cdr_open(&mut self, path: PathBuf, mode: CdrOpenMode) -> bool {
        if crate::files::WEB || self.color_import_context.is_none() {
            return false;
        }
        // Do not call file_kind for this guard: it reads an entire AI file.
        let cdr = path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| {
                matches!(value.to_ascii_lowercase().as_str(), "cdr" | "cdt" | "cdrt")
            });
        if mode == CdrOpenMode::Open && !is_template_path(&path) {
            if let Some(index) = self.tab_with_file(&path) {
                // Activating an existing tab must not discard another file
                // already reading in an instance-inbox batch [new, old, new].
                self.switch_document(index);
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default();
                self.status = crate::i18n::trf("status.already_open", &[("name", &name)]);
                self.settings.touch_recent(&path);
                self.settings.save();
                return true;
            }
        }
        if let Some(job) = self.cdr_open_job.as_mut() {
            if mode == CdrOpenMode::Open
                && ((same_open_file(&job.path, &path)
                    && job.mode == mode
                    && !job.discarded.load(Ordering::Relaxed))
                    || job.pending.iter().any(|request| {
                        request.mode == mode && same_open_file(&request.path, &path)
                    }))
            {
                return true;
            }
            job.pending.push_back(CdrOpenRequest { path, mode });
            return true;
        }
        if !cdr {
            return false;
        }
        if let Err(error) = self.start_cdr_open(path.clone(), mode) {
            self.cdr_open_error(&path, &error);
        }
        true
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_cdr_open(&mut self, path: PathBuf, mode: CdrOpenMode) -> Result<(), String> {
        let context = self
            .color_import_context
            .clone()
            .ok_or("CDR UI context is missing")?;
        self.cdr_open_job = Some(spawn_cdr_open(
            path,
            self.settings.color.clone(),
            context,
            prepare_cdr_open,
        )?);
        if let Some(job) = self.cdr_open_job.as_mut() {
            job.mode = mode;
        }
        self.status = crate::i18n::trf("status.reading_cdr_progress", &[("percent", "0")]);
        Ok(())
    }

    pub(crate) fn discard_pending_cdr_open(&mut self) {
        if let Some(job) = self.cdr_open_job.as_mut() {
            job.discarded.store(true, Ordering::Relaxed);
            job.pending.clear();
        }
    }

    fn cdr_open_error(&mut self, path: &std::path::Path, error: &str) {
        self.status = crate::i18n::trf(
            "status.could_not_open",
            &[("p", &path.display().to_string()), ("e", error)],
        );
    }

    pub(crate) fn poll_cdr_open(&mut self, context: &egui::Context) {
        let Some(mut job) = self.cdr_open_job.take() else {
            return;
        };
        match job.receiver.try_recv() {
            Err(TryRecvError::Empty) => {
                if !job.discarded.load(Ordering::Relaxed) || !job.pending.is_empty() {
                    let percent = if job.discarded.load(Ordering::Relaxed) {
                        0
                    } else {
                        job.progress.load(Ordering::Relaxed).min(99)
                    };
                    self.status = crate::i18n::trf(
                        "status.reading_cdr_progress",
                        &[("percent", &percent.to_string())],
                    );
                    context.request_repaint_after(std::time::Duration::from_millis(50));
                }
                self.cdr_open_job = Some(job);
            }
            outcome => {
                if !job.discarded.load(Ordering::Relaxed) {
                    match outcome {
                        Ok(Ok(prepared)) => {
                            self.publish_cdr_open(&job.path, job.mode, prepared);
                            job.progress.store(100, Ordering::Relaxed);
                        }
                        Ok(Err(error)) => self.cdr_open_error(&job.path, &error),
                        Err(TryRecvError::Disconnected) => {
                            self.cdr_open_error(&job.path, "CDR opening worker stopped")
                        }
                        Err(TryRecvError::Empty) => {
                            self.cdr_open_job = Some(job);
                            return;
                        }
                    }
                }
                if let Some(next) = job.pending.pop_front() {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.continue_cdr_queue(next, job.pending);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn continue_cdr_queue(&mut self, first: CdrOpenRequest, mut pending: VecDeque<CdrOpenRequest>) {
        let mut next = Some(first);
        while let Some(request) = next.take() {
            // A previous queued item may have opened this file under an alias.
            // Preserve normal Open's existing-tab semantics before reading it.
            if request.mode == CdrOpenMode::Open
                && !is_template_path(&request.path)
                && self.tab_with_file(&request.path).is_some()
            {
                self.queue_cdr_open(request.path, request.mode);
                next = pending.pop_front();
                continue;
            }
            let cdr = request
                .path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| {
                    matches!(value.to_ascii_lowercase().as_str(), "cdr" | "cdt" | "cdrt")
                });
            if cdr {
                match self.start_cdr_open(request.path.clone(), request.mode) {
                    Ok(()) => {
                        if let Some(job) = self.cdr_open_job.as_mut() {
                            job.pending = pending;
                        }
                        return;
                    }
                    Err(error) => self.cdr_open_error(&request.path, &error),
                }
            } else {
                // Mixed inbox batches keep their order too. Only CDR needs
                // this reader worker; other formats retain their existing API.
                match request.mode {
                    CdrOpenMode::Open => {
                        let _ = self.try_open_path(request.path);
                    }
                    CdrOpenMode::Reopen => {
                        let _ = self.try_open_path_with_password(request.path, "");
                    }
                    CdrOpenMode::Template => self.open_as_template(request.path),
                }
            }
            next = pending.pop_front();
        }
    }

    fn publish_cdr_open(
        &mut self,
        path: &std::path::Path,
        mode: CdrOpenMode,
        prepared: PreparedCdrOpen,
    ) {
        let PreparedCdrOpen {
            engine,
            report,
            colors,
            color_notice,
            color_error,
            thumbnail,
        } = prepared;
        let cdr_version = report
            .version
            .map(|version| writable_cdr_version(version.0));
        let opened_version = report.version.map(|version| version.0);
        let version = report
            .version
            .map(|version| version.name())
            .unwrap_or_default();
        let mut message = crate::i18n::trf(
            "status.opened_cdr",
            &[
                ("version", &version.to_string()),
                ("pages", &report.pages.to_string()),
                ("objects", &report.shapes.to_string()),
                ("skipped", &report.skipped_objects.to_string()),
            ],
        );
        if let (Some(old), Some(written)) = (report.version, cdr_version) {
            if old.0 != written {
                message.push(' ');
                message.push_str(&crate::i18n::trf(
                    "status.cdr_saves_as",
                    &[("version", &written.to_string())],
                ));
            }
        }
        self.open_prepared_document(engine, None);
        self.install_prepared_screen_colors(colors);
        self.opened_from =
            (!crate::files::WEB && !is_template_path(path)).then(|| crate::files::canonical(path));
        if mode == CdrOpenMode::Template {
            self.opened_from = None;
        }
        self.cdr_version = cdr_version;
        self.cdr_unsaved_version =
            opened_version.filter(|version| *version < tracedraw_cdr::write::CHECKED_SINCE);
        self.tdraw_version = None;
        if is_template_path(path) {
            self.cdr_version = None;
            self.cdr_unsaved_version = None;
        }
        self.file = None;
        self.status = format!("{}: {message}", path.display());
        if let Some(notice) = color_notice {
            self.status.push_str(&format!(" | {notice}"));
        }
        if let Some(error) = color_error {
            self.status.push_str(&format!(
                " | {}",
                crate::i18n::trf("color_mgmt.apply_failed", &[("e", &error)])
            ));
        }
        self.fit_pending = true;
        self.settings.touch_recent(path);
        self.settings.last_dir = path.parent().map(|parent| parent.to_path_buf());
        self.settings.save();
        self.store_prepared_thumbnail(path, thumbnail);
        self.show_welcome = false;
        self.check_missing_fonts();
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        self.pdf_shrunk.clear();
        self.show_open_notes(name, report.skipped_objects);
    }

    pub(crate) fn show_cdr_open_progress(&self, context: &egui::Context) {
        let Some(job) = &self.cdr_open_job else {
            return;
        };
        if job.discarded.load(Ordering::Relaxed) && job.pending.is_empty() {
            return;
        }
        let percent = if job.discarded.load(Ordering::Relaxed) {
            0
        } else {
            job.progress.load(Ordering::Relaxed).min(99)
        };
        let label = crate::i18n::trf(
            "status.reading_cdr_progress",
            &[("percent", &percent.to_string())],
        );
        egui::Area::new(egui::Id::new("cdr_open_progress"))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 76.0))
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(280.0);
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(label);
                    });
                    let path = if job.discarded.load(Ordering::Relaxed) {
                        job.pending
                            .front()
                            .map(|request| &request.path)
                            .unwrap_or(&job.path)
                    } else {
                        &job.path
                    };
                    if let Some(name) = path.file_name() {
                        ui.label(name.to_string_lossy());
                    }
                    ui.add(egui::ProgressBar::new(f32::from(percent) / 100.0).desired_width(280.0));
                });
            });
    }
}
