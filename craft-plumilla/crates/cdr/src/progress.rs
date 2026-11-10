//! Optional, measured progress for opening a drawing. No clock-based estimate.

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenPhase {
    Streams,
    Bitmaps,
    Document,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenProgress {
    pub phase: OpenPhase,
    pub percent: u8,
    pub completed: u64,
    pub total: u64,
}

pub type OpenProgressCallback = Arc<dyn Fn(OpenProgress) + Send + Sync>;

struct State {
    callback: OpenProgressCallback,
    last: Mutex<Option<OpenProgress>>,
}

#[derive(Clone, Default)]
pub(crate) struct Progress(Option<Arc<State>>);

impl Progress {
    pub fn new(callback: Option<OpenProgressCallback>) -> Self {
        let progress = Self(callback.map(|callback| {
            Arc::new(State {
                callback,
                last: Mutex::new(None),
            })
        }));
        progress.report(OpenPhase::Streams, 0, 0, 0);
        progress
    }

    pub fn enabled(&self) -> bool {
        self.0.is_some()
    }

    pub fn report(&self, phase: OpenPhase, percent: u8, completed: u64, total: u64) {
        let Some(state) = &self.0 else { return };
        let Ok(mut last) = state.last.lock() else {
            return;
        };
        let percent = percent.min(if phase == OpenPhase::Complete {
            100
        } else {
            99
        });
        let percent = last.map_or(percent, |p| p.percent.max(percent));
        if last.is_some_and(|p| p.percent == percent && p.phase == phase) {
            return;
        }
        let event = OpenProgress {
            phase,
            percent,
            completed,
            total,
        };
        *last = Some(event);
        // Keep ordering even when several small pictures finish on workers.
        (state.callback)(event);
    }

    pub fn streams(&self, completed: u64, total: u64) {
        let fraction = if total == 0 {
            0
        } else {
            completed.min(total).saturating_mul(60) / total
        };
        self.report(OpenPhase::Streams, fraction as u8, completed, total);
    }

    pub fn bitmap_work(&self, total: u64) -> BitmapWork {
        self.report(OpenPhase::Bitmaps, 60, 0, total);
        BitmapWork {
            progress: self.clone(),
            completed: std::sync::atomic::AtomicU64::new(0),
            total,
        }
    }
}

pub(crate) struct BitmapWork {
    progress: Progress,
    completed: std::sync::atomic::AtomicU64,
    total: u64,
}

impl BitmapWork {
    pub fn advance(&self, pixels: u64) {
        if !self.progress.enabled() {
            return;
        }
        let completed = self
            .completed
            .fetch_add(pixels, std::sync::atomic::Ordering::Relaxed)
            .saturating_add(pixels);
        let fraction = if self.total == 0 {
            0
        } else {
            completed.min(self.total).saturating_mul(35) / self.total
        };
        self.progress.report(
            OpenPhase::Bitmaps,
            60 + fraction as u8,
            completed,
            self.total,
        );
    }

    pub fn finish(&self) {
        self.progress.report(OpenPhase::Document, 95, 0, 2);
    }
}

pub(crate) struct Timing {
    #[cfg(not(target_arch = "wasm32"))]
    clock: Option<(std::time::Instant, std::time::Instant)>,
    label: &'static str,
}

impl Timing {
    pub fn new(label: &'static str) -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            clock: std::env::var_os("CRAFT_PLUMILLA_OPEN_TIMING")
                .filter(|value| value != "0")
                .map(|_| {
                    let now = std::time::Instant::now();
                    (now, now)
                }),
            label,
        }
    }

    pub fn lap(&mut self, phase: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some((start, last)) = &mut self.clock {
            let now = std::time::Instant::now();
            eprintln!(
                "craft-cdr timing scope={} phase={} elapsed_ms={} total_ms={}",
                self.label,
                phase,
                now.duration_since(*last).as_millis(),
                now.duration_since(*start).as_millis()
            );
            *last = now;
        }
        #[cfg(target_arch = "wasm32")]
        let _ = (self.label, phase);
    }
}
