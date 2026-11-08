//! Time Tracker: the time spent on a drawing, task by task, saved with
//! the drawing (`Document::time_tracker`).
//!
//! One clock serves the editor. While it runs, time goes to the active
//! task of the active drawing, and only while someone is at work: time
//! counts up to the idle limit after the last input (pointer, keys,
//! wheel), so a break pauses the clock on its own and the next input
//! starts it again. Counted time is written to the drawing every few
//! seconds, when the clock stops, when the active task changes, and
//! before the drawing is saved, closed or left for another tab.
//!
//! Recording time does not count as a change of the drawing (no undo step,
//! no unsaved marker): the records are saved with the drawing's next save.
//! Editing the records by hand (a name, a time, a date) does mark the
//! drawing changed.

use crate::app::App;
use jiff::tz::TimeZone;
use tracedraw_core::{Command, TimeTracker, TrackedTask};

/// Counted time is written to the drawing once this many seconds gather.
pub const FLUSH_SECONDS: f64 = 10.0;

/// The clock.
#[derive(Debug, Clone, Default)]
pub struct Clock {
    /// Recording: the active task gathers time while the user works.
    pub running: bool,
    /// When the last frame was counted (seconds on the frame clock).
    last_tick: Option<f64>,
    /// When the user last did something (seconds on the frame clock).
    last_input: f64,
    /// Seconds counted and not yet written to the drawing.
    pub pending: f64,
}

impl Clock {
    /// Seconds to count for the frame at `now` (seconds on a monotonic
    /// clock); `input` tells whether the user did something this frame.
    /// The time since the last frame counts up to `idle` seconds after the
    /// input before this frame, so a long break adds nothing.
    pub fn tick(&mut self, now: f64, input: bool, idle: f64) -> f64 {
        let counted = match self.last_tick {
            Some(prev) if self.running && now > prev => {
                let until = now.min(self.last_input + idle.max(0.0));
                (until - prev).clamp(0.0, now - prev)
            }
            _ => 0.0,
        };
        self.last_tick = Some(now);
        if input {
            self.last_input = now;
        }
        counted
    }

    /// True when the last input is more than `idle` seconds old.
    pub fn idle(&self, now: f64, idle: f64) -> bool {
        now - self.last_input > idle
    }
}

/// True for the events that show someone at work.
pub fn is_user_input(events: &[egui::Event]) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            egui::Event::Key { .. }
                | egui::Event::Text(_)
                | egui::Event::PointerMoved(_)
                | egui::Event::MouseMoved(_)
                | egui::Event::PointerButton { .. }
                | egui::Event::MouseWheel { .. }
                | egui::Event::Zoom(_)
                | egui::Event::Touch { .. }
                | egui::Event::Copy
                | egui::Event::Cut
                | egui::Event::Paste(_)
                | egui::Event::Ime(_)
        )
    })
}

/// Now, in seconds since 1970-01-01 UTC.
pub fn unix_now() -> i64 {
    jiff::Timestamp::now().as_second()
}

/// The time zone dates are shown and typed in.
pub fn local_zone() -> TimeZone {
    TimeZone::try_system().unwrap_or(TimeZone::UTC)
}

/// A duration as hours, minutes and seconds ("1:05:09"); whole seconds,
/// rounded down.
pub fn hms(seconds: f64) -> String {
    let s = if seconds.is_finite() {
        seconds.max(0.0).floor() as u64
    } else {
        0
    };
    format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// Hours with two decimals ("1.09"), for spreadsheets.
pub fn decimal_hours(seconds: f64) -> String {
    let h = if seconds.is_finite() {
        seconds.max(0.0) / 3600.0
    } else {
        0.0
    };
    format!("{h:.2}")
}

/// A typed duration: "h:mm:ss", "h:mm" or hours alone ("1.5"); `None`
/// for anything else or a negative time.
pub fn parse_hms(text: &str) -> Option<f64> {
    let parts: Vec<&str> = text.trim().split(':').map(str::trim).collect();
    let num = |s: &str| -> Option<f64> {
        let v: f64 = s.replace(',', ".").parse().ok()?;
        (v.is_finite() && v >= 0.0).then_some(v)
    };
    let whole = |s: &str| -> Option<f64> {
        let v: u64 = s.parse().ok()?;
        Some(v as f64)
    };
    match parts.as_slice() {
        [h] if !h.is_empty() => num(h).map(|h| h * 3600.0),
        [h, m] => {
            let (h, m) = (whole(h)?, whole(m)?);
            (m < 60.0).then_some(h * 3600.0 + m * 60.0)
        }
        [h, m, s] => {
            let (h, m, s) = (whole(h)?, whole(m)?, num(s)?);
            (m < 60.0 && s < 60.0).then_some(h * 3600.0 + m * 60.0 + s)
        }
        _ => None,
    }
}

/// A moment as a local date and time ("2026-10-11 14:05").
pub fn date_time(seconds: i64, tz: &TimeZone) -> String {
    jiff::Timestamp::from_second(seconds)
        .map(|t| {
            t.to_zoned(tz.clone())
                .strftime("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_default()
}

/// A typed local date and time: "YYYY-MM-DD HH:MM" or a date alone (its
/// midnight). An empty text is no date (`Some(None)`); `None` when the
/// text cannot be read.
pub fn parse_date_time(text: &str, tz: &TimeZone) -> Option<Option<i64>> {
    let text = text.trim();
    if text.is_empty() {
        return Some(None);
    }
    let dt = jiff::civil::DateTime::strptime("%Y-%m-%d %H:%M", text)
        .or_else(|_| jiff::civil::DateTime::strptime("%Y-%m-%d %H:%M:%S", text))
        .or_else(|_| {
            jiff::civil::Date::strptime("%Y-%m-%d", text)
                .map(|d| d.to_datetime(jiff::civil::Time::midnight()))
        })
        .ok()?;
    let z = dt.to_zoned(tz.clone()).ok()?;
    Some(Some(z.timestamp().as_second()))
}

/// The name a new task gets: "Task n" with the first number not taken.
pub fn new_task_name(tracker: &TimeTracker) -> String {
    (1..)
        .map(|n| crate::i18n::trf("tracker.task_n", &[("n", &n.to_string())]))
        .find(|name| !tracker.tasks.iter().any(|t| &t.name == name))
        .unwrap_or_default()
}

/// One field of a CSV line, quoted when it has to be.
fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// The records as CSV: task, time, hours, start and finish (local time),
/// then the total. `extra` is time counted and not yet written, which
/// belongs to the active task.
pub fn csv(tracker: &TimeTracker, extra: f64, tz: &TimeZone) -> String {
    use crate::i18n::tr;
    let mut out = String::new();
    let header = [
        tr("tracker.task"),
        tr("tracker.time"),
        tr("tracker.hours"),
        tr("tracker.start"),
        tr("tracker.finish"),
    ];
    out.push_str(&header.map(|h| csv_field(&h)).join(","));
    out.push_str("\r\n");
    for (i, t) in tracker.tasks.iter().enumerate() {
        let secs = task_seconds(tracker, i, extra);
        let row = [
            csv_field(&t.name),
            hms(secs),
            decimal_hours(secs),
            t.start.map(|s| date_time(s, tz)).unwrap_or_default(),
            t.finish.map(|s| date_time(s, tz)).unwrap_or_default(),
        ];
        out.push_str(&row.join(","));
        out.push_str("\r\n");
    }
    let total = tracker.total_seconds() + active_extra(tracker, extra);
    out.push_str(&format!(
        "{},{},{},,\r\n",
        csv_field(&tr("tracker.total")),
        hms(total),
        decimal_hours(total)
    ));
    out
}

/// The records as a plain text report: the drawing's name, when it was
/// written, then a table of the tasks and the total.
pub fn text_report(
    title: &str,
    tracker: &TimeTracker,
    extra: f64,
    tz: &TimeZone,
    now: i64,
) -> String {
    use crate::i18n::{tr, trf};
    let mut rows: Vec<[String; 4]> = vec![[
        tr("tracker.task"),
        tr("tracker.time"),
        tr("tracker.start"),
        tr("tracker.finish"),
    ]];
    for (i, t) in tracker.tasks.iter().enumerate() {
        rows.push([
            t.name.clone(),
            hms(task_seconds(tracker, i, extra)),
            t.start.map(|s| date_time(s, tz)).unwrap_or_default(),
            t.finish.map(|s| date_time(s, tz)).unwrap_or_default(),
        ]);
    }
    let total = tracker.total_seconds() + active_extra(tracker, extra);
    rows.push([
        tr("tracker.total"),
        hms(total),
        String::new(),
        String::new(),
    ]);
    let width = |c: usize| rows.iter().map(|r| r[c].chars().count()).max().unwrap_or(0);
    let widths = [width(0), width(1), width(2), width(3)];
    let mut out = format!(
        "{}\n{}\n\n",
        trf("tracker.report_title", &[("name", title)]),
        trf("tracker.report_written", &[("date", &date_time(now, tz))])
    );
    let last = rows.len() - 1;
    for (r, row) in rows.iter().enumerate() {
        if r == last {
            let rule: usize = widths.iter().sum::<usize>() + 6;
            out.push_str(&"-".repeat(rule));
            out.push('\n');
        }
        let mut line = String::new();
        for (c, cell) in row.iter().enumerate() {
            let pad = widths[c].saturating_sub(cell.chars().count());
            // The time column lines up on the right.
            if c == 1 {
                line.push_str(&" ".repeat(pad));
                line.push_str(cell);
            } else {
                line.push_str(cell);
                if c < 3 {
                    line.push_str(&" ".repeat(pad));
                }
            }
            if c < 3 {
                line.push_str("  ");
            }
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// Task `i`'s time with the unwritten `extra` when it is the active one.
pub fn task_seconds(tracker: &TimeTracker, i: usize, extra: f64) -> f64 {
    let base = tracker
        .tasks
        .get(i)
        .map(|t| t.seconds.max(0.0))
        .unwrap_or(0.0);
    if tracker.active == Some(i) {
        base + extra.max(0.0)
    } else {
        base
    }
}

fn active_extra(tracker: &TimeTracker, extra: f64) -> f64 {
    if tracker.active_task().is_some() {
        extra.max(0.0)
    } else {
        0.0
    }
}

impl App {
    /// The clock's work for one frame at `now` (seconds on the frame
    /// clock); `input` tells whether the user did something. Returns true
    /// while the clock is counting, so the caller repaints to keep it
    /// going.
    pub fn time_tracker_frame(&mut self, now: f64, input: bool) -> bool {
        let idle = self.settings.time_tracker.idle_seconds();
        let counted = self.tracker.tick(now, input, idle);
        if counted > 0.0 && self.can_track() {
            self.tracker.pending += counted;
        }
        if self.tracker.pending >= FLUSH_SECONDS {
            self.flush_tracked_time();
        }
        self.tracker.running && !self.tracker.idle(now, idle)
    }

    /// True when time can go somewhere: a drawing is open and has an
    /// active task.
    pub fn can_track(&self) -> bool {
        self.has_document() && self.doc().time_tracker.active_task().is_some()
    }

    /// Write the counted time to the active task (a no-op when there is
    /// none: the time is dropped).
    pub fn flush_tracked_time(&mut self) {
        let secs = std::mem::take(&mut self.tracker.pending);
        if secs <= 0.0 || !self.can_track() {
            return;
        }
        let mut t = self.doc().time_tracker.clone();
        t.record(secs, unix_now());
        if let Err(e) = self.engine.run(&Command::SetTimeTracker { tracker: t }) {
            log::warn!("time tracker: {e}");
        }
    }

    /// Replace the records by hand: counted time goes to the task it was
    /// counted for first, and the drawing is marked changed.
    pub fn edit_time_tracker(&mut self, edit: impl FnOnce(&mut TimeTracker)) {
        if !self.has_document() {
            return;
        }
        self.flush_tracked_time();
        let mut t = self.doc().time_tracker.clone();
        edit(&mut t);
        // An index past the end is no active task.
        if t.active.is_some_and(|i| i >= t.tasks.len()) {
            t.active = None;
        }
        if t == self.doc().time_tracker {
            return;
        }
        match self.engine.run(&Command::SetTimeTracker { tracker: t }) {
            Ok(()) => self.engine.mark_changed(),
            Err(e) => log::warn!("time tracker: {e}"),
        }
    }

    /// Add a task named "Task n" and make it the active one.
    pub fn add_tracked_task(&mut self) {
        self.edit_time_tracker(|t| {
            let name = new_task_name(t);
            t.tasks.push(TrackedTask {
                name,
                ..Default::default()
            });
            t.active = Some(t.tasks.len() - 1);
        });
    }

    /// Start or pause the clock. Starting a drawing without tasks adds
    /// one; with tasks but none active, the first becomes active.
    pub fn toggle_time_clock(&mut self) {
        if self.tracker.running {
            self.flush_tracked_time();
            self.tracker.running = false;
            return;
        }
        if !self.has_document() {
            return;
        }
        let tracker = &self.doc().time_tracker;
        if tracker.tasks.is_empty() {
            self.add_tracked_task();
        } else if tracker.active_task().is_none() {
            self.edit_time_tracker(|t| t.active = Some(0));
        }
        self.tracker.running = true;
    }

    /// The active task's time, with what is counted and not yet written.
    pub fn active_task_seconds(&self) -> Option<f64> {
        if !self.has_document() {
            return None;
        }
        let t = &self.doc().time_tracker;
        t.active
            .filter(|&i| i < t.tasks.len())
            .map(|i| task_seconds(t, i, self.tracker.pending))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_counts_work_and_stops_in_a_break() {
        let mut c = Clock {
            running: true,
            ..Default::default()
        };
        let idle = 300.0;
        assert_eq!(c.tick(0.0, true, idle), 0.0);
        // A second of work.
        assert_eq!(c.tick(1.0, false, idle), 1.0);
        // Frames without input count until the idle limit...
        assert_eq!(c.tick(200.0, false, idle), 199.0);
        assert_eq!(c.tick(400.0, false, idle), 100.0);
        assert!(c.idle(400.0, idle));
        // ...and nothing after it, even when the user comes back.
        assert_eq!(c.tick(1000.0, true, idle), 0.0);
        assert!(!c.idle(1000.0, idle));
        assert_eq!(c.tick(1001.5, false, idle), 1.5);
        // A paused clock counts nothing; time never runs backwards.
        c.running = false;
        assert_eq!(c.tick(1010.0, true, idle), 0.0);
        c.running = true;
        assert_eq!(c.tick(1005.0, true, idle), 0.0);
    }

    #[test]
    fn durations_read_and_write() {
        assert_eq!(hms(0.0), "0:00:00");
        assert_eq!(hms(3909.9), "1:05:09");
        assert_eq!(hms(-5.0), "0:00:00");
        assert_eq!(hms(f64::NAN), "0:00:00");
        assert_eq!(hms(100.0 * 3600.0), "100:00:00");
        assert_eq!(decimal_hours(5400.0), "1.50");
        assert_eq!(parse_hms("1:05:09"), Some(3909.0));
        assert_eq!(parse_hms(" 2:30 "), Some(9000.0));
        assert_eq!(parse_hms("1.5"), Some(5400.0));
        assert_eq!(parse_hms("1,25"), Some(4500.0));
        assert_eq!(parse_hms("0:00:30.5"), Some(30.5));
        for bad in ["", "abc", "1:60", "1:00:60", "-1", "1:2:3:4", "1::2"] {
            assert_eq!(parse_hms(bad), None, "{bad}");
        }
    }

    #[test]
    fn dates_read_and_write_in_a_time_zone() {
        let utc = TimeZone::UTC;
        assert_eq!(date_time(0, &utc), "1970-01-01 00:00");
        assert_eq!(date_time(1_760_000_000, &utc), "2025-10-09 08:53");
        assert_eq!(
            parse_date_time("2025-10-09 08:53", &utc),
            Some(Some(1_759_999_980))
        );
        assert_eq!(parse_date_time("1970-01-02", &utc), Some(Some(86_400)));
        assert_eq!(parse_date_time("  ", &utc), Some(None));
        assert_eq!(parse_date_time("yesterday", &utc), None);
        assert_eq!(parse_date_time("2025-13-01 10:00", &utc), None);
        // A fixed offset shifts both ways alike.
        let plus2 = TimeZone::fixed(jiff::tz::offset(2));
        assert_eq!(date_time(0, &plus2), "1970-01-01 02:00");
        assert_eq!(parse_date_time("1970-01-01 02:00", &plus2), Some(Some(0)));
    }

    fn sample() -> TimeTracker {
        TimeTracker {
            tasks: vec![
                TrackedTask {
                    name: "Sketch, \"first\" pass".into(),
                    seconds: 3909.0,
                    start: Some(0),
                    finish: Some(3909),
                },
                TrackedTask {
                    name: "Colour".into(),
                    seconds: 60.0,
                    start: None,
                    finish: None,
                },
            ],
            active: Some(1),
        }
    }

    #[test]
    fn csv_quotes_names_and_adds_the_unwritten_time() {
        let csv = csv(&sample(), 30.0, &TimeZone::UTC);
        let lines: Vec<&str> = csv.split("\r\n").collect();
        assert_eq!(lines[0], "Task,Time,Hours,Start,Finish");
        assert_eq!(
            lines[1],
            "\"Sketch, \"\"first\"\" pass\",1:05:09,1.09,1970-01-01 00:00,1970-01-01 01:05"
        );
        // The active task carries the 30 s not yet written.
        assert_eq!(lines[2], "Colour,0:01:30,0.03,,");
        assert_eq!(lines[3], "Total,1:06:39,1.11,,");
        assert_eq!(lines[4], "");
    }

    #[test]
    fn the_text_report_lines_up_its_columns() {
        let report = text_report("poster", &sample(), 0.0, &TimeZone::UTC, 86_400);
        let lines: Vec<&str> = report.lines().collect();
        assert_eq!(lines[0], "Time records: poster");
        assert_eq!(lines[1], "Written 1970-01-02 00:00");
        assert_eq!(lines[2], "");
        assert!(lines[3].starts_with("Task "));
        let time_end = |l: &str| {
            l.find(":00:")
                .or_else(|| l.find(":05:"))
                .or_else(|| l.find(":06:"))
        };
        // Times end in the same column.
        let a = lines[4].find("1:05:09").map(|i| i + 7);
        let b = lines[5].find("0:01:00").map(|i| i + 7);
        assert!(a.is_some() && a == b, "{report}");
        assert!(lines[6].chars().all(|c| c == '-'));
        assert!(lines[7].starts_with("Total"));
        assert!(time_end(lines[7]).is_some());
    }

    #[test]
    fn new_tasks_take_the_first_free_number() {
        let mut t = TimeTracker::default();
        assert_eq!(new_task_name(&t), "Task 1");
        t.tasks.push(TrackedTask {
            name: "Task 1".into(),
            ..Default::default()
        });
        t.tasks.push(TrackedTask {
            name: "Task 3".into(),
            ..Default::default()
        });
        assert_eq!(new_task_name(&t), "Task 2");
    }

    #[test]
    fn recording_goes_to_the_active_task_without_an_undo_step() {
        let mut app = App::headless();
        app.open_document(
            tracedraw_core::Document::new("t", tracedraw_core::geometry::Size::new(100.0, 100.0)),
            None,
        );
        assert!(!app.can_track());
        // Starting adds "Task 1" (a change of the drawing).
        app.toggle_time_clock();
        assert!(app.tracker.running);
        assert_eq!(app.doc().time_tracker.tasks.len(), 1);
        assert_eq!(app.doc().time_tracker.active, Some(0));
        app.engine.mark_saved();
        let steps = app.engine.history_labels().0.len();
        // Work for 25 s in frames a second apart.
        let mut now = 0.0;
        app.time_tracker_frame(now, true);
        for _ in 0..25 {
            now += 1.0;
            assert!(app.time_tracker_frame(now, true));
        }
        // Written every 10 s; the rest is still pending.
        assert_eq!(app.doc().time_tracker.tasks[0].seconds, 20.0);
        assert_eq!(app.active_task_seconds(), Some(25.0));
        // Pausing writes the rest; recording is no change and no step.
        app.toggle_time_clock();
        assert!(!app.tracker.running);
        assert_eq!(app.doc().time_tracker.tasks[0].seconds, 25.0);
        assert!(app.doc().time_tracker.tasks[0].finish.is_some());
        assert!(!app.engine.is_dirty());
        assert_eq!(app.engine.history_labels().0.len(), steps);
        // A paused clock counts nothing.
        assert!(!app.time_tracker_frame(now + 5.0, true));
        assert_eq!(app.active_task_seconds(), Some(25.0));
        // A hand edit marks the drawing changed.
        app.edit_time_tracker(|t| t.tasks[0].name = "Layout".into());
        assert!(app.engine.is_dirty());
        assert_eq!(app.doc().time_tracker.tasks[0].name, "Layout");
    }

    #[test]
    fn leaving_a_drawing_writes_its_time_first() {
        let mut app = App::headless();
        let size = tracedraw_core::geometry::Size::new(100.0, 100.0);
        app.open_document(tracedraw_core::Document::new("a", size), None);
        let a = app.active_doc;
        app.toggle_time_clock();
        app.time_tracker_frame(0.0, true);
        app.time_tracker_frame(4.0, true);
        assert_eq!(app.tracker.pending, 4.0);
        // Opening another drawing writes the 4 s to the first one.
        app.open_document(tracedraw_core::Document::new("b", size), None);
        assert_eq!(app.tracker.pending, 0.0);
        // The new drawing has no task: its time goes nowhere.
        app.time_tracker_frame(6.0, true);
        assert_eq!(app.tracker.pending, 0.0);
        app.switch_document(a);
        assert_eq!(app.doc().title, "a");
        assert_eq!(
            app.doc().time_tracker.tasks.first().map(|t| t.seconds),
            Some(4.0)
        );
    }
}
