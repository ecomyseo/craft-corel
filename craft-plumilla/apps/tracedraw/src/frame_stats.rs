//! Frame statistics for the log (#616): what the first frame found (the
//! window's size in pixels, the interface scale, the monitor, how long
//! the start took), and at debug level every 10 s while frames happen
//! how many there were, how many had input, what building them cost, the
//! gaps between them and why they were asked for. An idle window logs
//! nothing, and with the debug level off nothing is measured past the
//! first frame. `RUST_LOG=debug` (or `RUST_LOG=info,tracedraw=debug`)
//! turns it on; see docs/behavior/about-and-startup.md.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;
use web_time::Instant;

/// How often a summary is logged while frames happen.
pub const PERIOD: Duration = Duration::from_secs(10);

/// When the process started (as the log began).
static START: OnceLock<Instant> = OnceLock::new();

pub fn mark_start() {
    let _ = START.set(Instant::now());
}

/// The frames of one period.
#[derive(Default)]
pub struct Stats {
    since: Option<Instant>,
    frames: u32,
    with_input: u32,
    build: Duration,
    build_max: Duration,
    gap: Duration,
    gap_max: Duration,
    last: Option<Instant>,
    causes: HashMap<String, u32>,
}

impl Stats {
    /// A frame that began at `began` and was built by `end`, with or
    /// without input, asked for by `causes`. The summary when the period
    /// is over (and a new one begins).
    pub fn frame(
        &mut self,
        began: Instant,
        end: Instant,
        input: bool,
        causes: impl IntoIterator<Item = String>,
    ) -> Option<String> {
        let since = *self.since.get_or_insert(began);
        if let Some(last) = self.last {
            let gap = began.saturating_duration_since(last);
            self.gap += gap;
            self.gap_max = self.gap_max.max(gap);
        }
        self.last = Some(began);
        let build = end.saturating_duration_since(began);
        self.frames += 1;
        self.with_input += u32::from(input);
        self.build += build;
        self.build_max = self.build_max.max(build);
        for c in causes {
            *self.causes.entry(c).or_default() += 1;
        }
        if end.saturating_duration_since(since) < PERIOD {
            return None;
        }
        let text = self.summary(end.saturating_duration_since(since));
        *self = Stats {
            last: self.last,
            ..Default::default()
        };
        Some(text)
    }

    fn summary(&self, span: Duration) -> String {
        let n = self.frames.max(1);
        let gaps = self.frames.saturating_sub(1).max(1);
        let mut causes: Vec<(&String, &u32)> = self.causes.iter().collect();
        causes.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let causes: Vec<String> = causes
            .iter()
            .take(5)
            .map(|(c, k)| format!("{c} x{k}"))
            .collect();
        format!(
            "frames: {} in {:.1} s ({} with input), built in {:.2} ms mean, {:.2} ms max; {:.1} ms apart mean, {:.1} ms max; asked for by: {}",
            self.frames,
            span.as_secs_f64(),
            self.with_input,
            self.build.as_secs_f64() * 1000.0 / n as f64,
            self.build_max.as_secs_f64() * 1000.0,
            self.gap.as_secs_f64() * 1000.0 / gaps as f64,
            self.gap_max.as_secs_f64() * 1000.0,
            if causes.is_empty() {
                "input".to_string()
            } else {
                causes.join(", ")
            }
        )
    }
}

thread_local! {
    static STATS: RefCell<Stats> = RefCell::new(Stats::default());
    /// Whether frames are measured (the debug level is on for this
    /// module), found at the first frame; none yet before it.
    static ON: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// At the start of building a frame: when it began, if anything is
/// measured.
pub fn begin(ctx: &egui::Context) -> Option<Instant> {
    let on = ON.with(|on| {
        on.get().unwrap_or_else(|| {
            first_frame(ctx);
            let yes = log::log_enabled!(log::Level::Debug);
            on.set(Some(yes));
            yes
        })
    });
    on.then(Instant::now)
}

/// Once the frame is built.
pub fn end(ctx: &egui::Context, began: Option<Instant>) {
    let Some(began) = began else {
        return;
    };
    let input = ctx.input(|i| !i.events.is_empty());
    let causes = ctx
        .repaint_causes()
        .into_iter()
        .map(|c| short_cause(&c.to_string()));
    let summary = STATS.with(|s| s.borrow_mut().frame(began, Instant::now(), input, causes));
    if let Some(text) = summary {
        log::debug!("{text}");
    }
}

/// A repaint cause as the log shows it: the crate and file of a
/// dependency without the registry's folders, no trailing blanks.
fn short_cause(cause: &str) -> String {
    let cause = cause.trim_end();
    match cause.find("index.crates.io-") {
        Some(i) => match cause[i..].find('/') {
            Some(j) => cause[i + j + 1..].to_string(),
            None => cause.to_string(),
        },
        None => cause.to_string(),
    }
}

fn first_frame(ctx: &egui::Context) {
    let ppp = ctx.pixels_per_point();
    let size = ctx.content_rect().size() * ppp;
    let monitor = ctx.input(|i| i.viewport().monitor_size);
    let after = START.get().map(|t| t.elapsed()).unwrap_or_default();
    log::info!(
        "first frame {:.2} s after the start: window {:.0} x {:.0} px, {ppp} px per point, monitor {}",
        after.as_secs_f64(),
        size.x,
        size.y,
        monitor
            .map(|m| format!("{:.0} x {:.0} points", m.x, m.y))
            .unwrap_or_else(|| "unknown".into())
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A summary every 10 s of frames: their count, how many had input,
    /// the build times and gaps, the causes most frequent first; then the
    /// next period starts afresh (#616).
    #[test]
    fn frames_are_summed_up_every_ten_seconds() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut s = Stats::default();
        let mut out = None;
        // A frame every 100 ms, built in 4 ms (8 ms every tenth), every
        // other one with input; the others asked for by an animation.
        for i in 0..=100u32 {
            let began = t0 + ms(100) * i;
            let build = if i % 10 == 0 { ms(8) } else { ms(4) };
            let causes = (i % 2 == 1).then(|| "app.rs:1 animation".to_string());
            if let Some(t) = s.frame(began, began + build, i % 2 == 0, causes) {
                assert!(out.is_none(), "one summary in 10 s");
                out = Some((i, t));
            }
        }
        let (at, text) = out.expect("a summary after 10 s");
        assert_eq!(at, 100);
        assert!(
            text.starts_with("frames: 101 in 10.0 s (51 with input)"),
            "{text}"
        );
        assert!(text.contains("8.00 ms max"), "{text}");
        assert!(text.contains("100.0 ms apart mean, 100.0 ms max"), "{text}");
        assert!(text.contains("app.rs:1 animation x50"), "{text}");
        // The next period counts from nothing, its first gap included.
        let next = t0 + ms(10_200);
        assert_eq!(s.frame(next, next + ms(1), false, None::<String>), None);
        assert_eq!(s.frames, 1);
        assert_eq!(s.gap_max, ms(200));
        assert_eq!(
            short_cause(
                "/home/u/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/egui-0.36.2/src/context.rs:538  "
            ),
            "egui-0.36.2/src/context.rs:538"
        );
        assert_eq!(
            short_cause("apps/tracedraw/src/app.rs:3061 "),
            "apps/tracedraw/src/app.rs:3061"
        );
    }
}
