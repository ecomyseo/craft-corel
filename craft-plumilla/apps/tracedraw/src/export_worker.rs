//! Export execution belongs to a worker; the UI keeps a cancellable snapshot.

use super::App;
use crate::export::{ExportJob, ExportPhase, ExportProgress};
use crate::i18n::{tr, trf};
use crate::ui::dialogs::ExportState;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};

pub(crate) struct ExportWorker {
    path: PathBuf,
    receiver: Receiver<Result<String, String>>,
    progress: Arc<Mutex<Option<ExportProgress>>>,
    cancel: Arc<AtomicBool>,
}

impl Drop for ExportWorker {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl ExportWorker {
    fn snapshot(&self) -> Option<ExportProgress> {
        self.progress.lock().ok().and_then(|value| value.clone())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn(path: PathBuf, prepared: ExportJob, context: egui::Context) -> Result<Self, String> {
        Self::spawn_task(path, context, move |callback, cancel| prepared.run(Some(callback), cancel))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn_task(
        path: PathBuf,
        context: egui::Context,
        run: impl FnOnce(Arc<dyn Fn(ExportProgress) + Send + Sync>, Arc<AtomicBool>) -> Result<String, String>
            + Send + 'static,
    ) -> Result<Self, String> {
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(Mutex::new(None));
        let worker_cancel = Arc::clone(&cancel);
        let worker_progress = Arc::clone(&progress);
        let wake = context.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("document-export".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let callback: Arc<dyn Fn(ExportProgress) + Send + Sync> = Arc::new(move |event| {
                    if let Ok(mut last) = worker_progress.lock() {
                        *last = Some(event);
                    }
                    wake.request_repaint();
                });
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run(callback, worker_cancel)
                }))
                .unwrap_or_else(|_| Err("export worker failed unexpectedly".into()));
                let _ = sender.send(result);
                context.request_repaint();
            })
            .map_err(|error| error.to_string())?;
        Ok(Self { path, receiver, progress, cancel })
    }
}

fn progress_label(progress: Option<&ExportProgress>, cancelling: bool) -> String {
    if cancelling {
        return tr("status.export_cancelling");
    }
    let phase = match progress.map(|event| &event.phase) {
        Some(ExportPhase::Encode) => tr("status.export_encoding"),
        Some(ExportPhase::Write | ExportPhase::Complete) => tr("status.export_writing"),
        _ => tr("status.export_rendering"),
    };
    let percent = progress.map_or(0, |event| event.percent.min(99));
    trf("status.export_progress", &[("phase", &phase), ("percent", &percent.to_string())])
}

impl App {
    pub(crate) fn start_export_job(&mut self, path: PathBuf, state: ExportState) -> Result<(), String> {
        if self.export_job.is_some() {
            return Err(tr("status.export_busy"));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let prepared = crate::export::prepare_job(self, &path, &state)?;
            let context = self.color_import_context.clone().unwrap_or_default();
            self.export_job = Some(ExportWorker::spawn(path, prepared, context)?);
            self.status = progress_label(None, false);
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            // Browser downloads must stay in their user-gesture integration.
            self.status = crate::export::export(self, &path, &state)?;
            Ok(())
        }
    }

    pub(crate) fn cancel_export_job(&mut self) {
        if let Some(job) = &self.export_job {
            job.cancel.store(true, Ordering::Relaxed);
            self.status = tr("status.export_cancelling");
        }
    }

    pub(crate) fn export_progress(&self) -> Option<ExportProgress> {
        self.export_job.as_ref().and_then(ExportWorker::snapshot)
    }

    pub(crate) fn poll_export_job(&mut self, context: &egui::Context) {
        let Some(job) = self.export_job.take() else { return };
        match job.receiver.try_recv() {
            Ok(Ok(message)) => self.status = message,
            Ok(Err(error)) => {
                self.status = if error == "export cancelled" {
                    tr("status.export_cancelled")
                } else {
                    format!("{}: {error}", tr("dialog.export_failed"))
                };
            }
            Err(TryRecvError::Disconnected) => self.status = tr("status.export_worker_failed"),
            Err(TryRecvError::Empty) => {
                let current = job.snapshot();
                self.status = progress_label(current.as_ref(), job.cancel.load(Ordering::Relaxed));
                self.export_job = Some(job);
                context.request_repaint_after(std::time::Duration::from_millis(50));
            }
        }
    }

    pub(crate) fn show_export_progress(&mut self, context: &egui::Context) {
        let Some(job) = self.export_job.as_ref() else { return };
        let progress = job.snapshot();
        let cancelling = job.cancel.load(Ordering::Relaxed);
        let name = job.path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
        let label = progress_label(progress.as_ref(), cancelling);
        let mut cancel_pressed = false;
        let width = (context.content_rect().width() - 32.0).clamp(64.0, 320.0);
        egui::Area::new(egui::Id::new("document_export_progress"))
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -48.0))
            .order(egui::Order::Foreground)
            .show(context, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(width);
                    ui.horizontal_wrapped(|ui| { ui.spinner(); ui.label(label); });
                    ui.label(name);
                    let value = progress.as_ref().map_or(0, |event| event.percent.min(99));
                    ui.add(egui::ProgressBar::new(f32::from(value) / 100.0).desired_width(width));
                    cancel_pressed = ui.add_enabled(!cancelling, egui::Button::new(tr("dialog.cancel"))).clicked();
                });
            });
        if cancel_pressed { self.cancel_export_job(); }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[path = "export_worker_tests.rs"]
mod tests;
