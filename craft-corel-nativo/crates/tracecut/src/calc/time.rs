//! Cutting time: the outlines cut at the cutting speed, the moves between
//! them at the travel speed, a lift and lower per outline, all slowed by
//! an acceleration factor and repeated for every pass.

use crate::subpaths;
use tracedraw_core::geometry::{BezPath, PathEl, Point};

/// A machine's speeds, kept by name.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeProfile {
    pub name: String,
    /// Speed while cutting (mm/s).
    pub cut_speed: f64,
    /// Speed of the moves with the tool up (mm/s).
    pub travel_speed: f64,
    /// Time to lift and lower the tool once per outline (s).
    pub lift_time: f64,
    /// Cutting and travel times are multiplied by this (the head never
    /// runs at full speed through corners and starts).
    pub accel_factor: f64,
    /// Every outline is cut this many times.
    pub passes: u32,
}

/// Slowest and fastest speeds accepted (mm/s).
pub const SPEED_RANGE: (f64, f64) = (0.1, 100_000.0);
/// Longest lift and lower time accepted (s).
pub const MAX_LIFT_TIME: f64 = 60.0;
/// Acceleration factors accepted.
pub const ACCEL_RANGE: (f64, f64) = (1.0, 10.0);
/// Most passes accepted.
pub const MAX_PASSES: u32 = 100;

impl Default for TimeProfile {
    fn default() -> Self {
        TimeProfile {
            name: "Standard".into(),
            cut_speed: 300.0,
            travel_speed: 600.0,
            lift_time: 0.2,
            accel_factor: 1.2,
            passes: 1,
        }
    }
}

impl TimeProfile {
    /// The profile with every value in its range; a value that is not a
    /// number takes the default.
    pub fn sanitized(&self) -> TimeProfile {
        let d = TimeProfile::default();
        let fix = |v: f64, def: f64, lo: f64, hi: f64| {
            if v.is_finite() {
                v.clamp(lo, hi)
            } else {
                def
            }
        };
        TimeProfile {
            name: self.name.clone(),
            cut_speed: fix(self.cut_speed, d.cut_speed, SPEED_RANGE.0, SPEED_RANGE.1),
            travel_speed: fix(
                self.travel_speed,
                d.travel_speed,
                SPEED_RANGE.0,
                SPEED_RANGE.1,
            ),
            lift_time: fix(self.lift_time, d.lift_time, 0.0, MAX_LIFT_TIME),
            accel_factor: fix(
                self.accel_factor,
                d.accel_factor,
                ACCEL_RANGE.0,
                ACCEL_RANGE.1,
            ),
            passes: self.passes.clamp(1, MAX_PASSES),
        }
    }
}

/// The order the head visits the outlines in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TravelOrder {
    /// The stacking order, each outline from its first node.
    AsDrawn,
    /// Always the nearest outline next: a closed outline entered at its
    /// nearest node (and left there), an open one at its nearer end (and
    /// left at the other).
    #[default]
    Nearest,
}

/// The head's moves with the tool up.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Travel {
    /// Length of the moves between outlines, from the start point on
    /// (mm). The way back is not counted.
    pub travel_length: f64,
    /// Outlines (subpaths) cut.
    pub outlines: usize,
    /// The job was too large for [`TravelOrder::Nearest`] and was taken
    /// as drawn.
    pub as_drawn_fallback: bool,
}

/// Above this many node distances to compare, Nearest falls back to the
/// stacking order (the estimate must stay quick).
pub const NEAREST_WORK_LIMIT: usize = 20_000_000;

/// One outline as the head sees it.
struct Stop {
    /// Nodes it can start at.
    entries: Vec<Point>,
    /// Closed: it ends where it starts.
    closed: bool,
    first: Point,
    last: Point,
}

fn finite(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

fn stops(paths: &[BezPath]) -> Vec<Stop> {
    let mut out = Vec::new();
    for path in paths {
        for s in subpaths(path) {
            let nodes: Vec<Point> = s
                .path
                .elements()
                .iter()
                .filter(|e| !matches!(e, PathEl::ClosePath))
                .filter_map(|e| e.end_point())
                .filter(|p| finite(*p))
                .collect();
            let (Some(first), Some(last)) = (nodes.first().copied(), nodes.last().copied()) else {
                continue;
            };
            out.push(Stop {
                entries: if s.closed { nodes } else { vec![first, last] },
                closed: s.closed,
                first,
                last,
            });
        }
    }
    out
}

/// The moves between the outlines of `paths` (page space, mm), starting
/// at `start`.
pub fn travel(paths: &[BezPath], start: Point, order: TravelOrder) -> Travel {
    walk(paths, start, order, |_, _| {})
}

/// [`travel`] and each move of the head with the tool up, from where it
/// is to where the next outline starts, in the order they are made.
pub fn travel_moves(
    paths: &[BezPath],
    start: Point,
    order: TravelOrder,
) -> (Travel, Vec<(Point, Point)>) {
    let mut moves = Vec::new();
    let t = walk(paths, start, order, |a, b| moves.push((a, b)));
    (t, moves)
}

fn walk(
    paths: &[BezPath],
    start: Point,
    order: TravelOrder,
    mut on_move: impl FnMut(Point, Point),
) -> Travel {
    let stops = stops(paths);
    let start = if finite(start) { start } else { Point::ZERO };
    let nodes: usize = stops.iter().map(|s| s.entries.len()).sum();
    let mut t = Travel {
        outlines: stops.len(),
        ..Default::default()
    };
    let nearest = order == TravelOrder::Nearest;
    if nearest && stops.len().saturating_mul(nodes) > NEAREST_WORK_LIMIT {
        t.as_drawn_fallback = true;
    }
    let mut pos = start;
    if !nearest || t.as_drawn_fallback {
        for s in &stops {
            t.travel_length += (s.first - pos).hypot();
            on_move(pos, s.first);
            pos = if s.closed { s.first } else { s.last };
        }
        return t;
    }
    let mut left: Vec<usize> = (0..stops.len()).collect();
    while !left.is_empty() {
        // (position in `left`, entry point, its distance, exit point)
        let mut best: Option<(usize, f64, Point, Point)> = None;
        for (k, &i) in left.iter().enumerate() {
            let Some(s) = stops.get(i) else { continue };
            for &e in &s.entries {
                let d = (e - pos).hypot();
                if best.is_none_or(|(_, bd, _, _)| d < bd) {
                    let exit = if s.closed {
                        e
                    } else if e == s.first {
                        s.last
                    } else {
                        s.first
                    };
                    best = Some((k, d, e, exit));
                }
            }
        }
        let Some((k, d, entry, exit)) = best else {
            break;
        };
        t.travel_length += d;
        on_move(pos, entry);
        pos = exit;
        left.swap_remove(k);
    }
    t
}

/// How long a job takes, every part over all passes (seconds).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TimeEstimate {
    /// Length cut in one pass (mm).
    pub cut_length: f64,
    pub travel: Travel,
    pub passes: u32,
    pub cut_s: f64,
    pub travel_s: f64,
    pub lift_s: f64,
    pub total_s: f64,
}

/// The time to cut `cut_length` mm with the moves of `travel`, with the
/// values of `profile` (taken in their ranges, see
/// [`TimeProfile::sanitized`]):
///
/// - cutting = cut length / cutting speed x acceleration factor
/// - travel = travel length / travel speed x acceleration factor
/// - lifts = outlines x lift time
/// - total = (cutting + travel + lifts) x passes
pub fn estimate(cut_length: f64, travel: &Travel, profile: &TimeProfile) -> TimeEstimate {
    let p = profile.sanitized();
    let cut_length = super::non_negative(cut_length);
    let travel_length = super::non_negative(travel.travel_length);
    let passes = f64::from(p.passes);
    let cut_s = cut_length / p.cut_speed * p.accel_factor * passes;
    let travel_s = travel_length / p.travel_speed * p.accel_factor * passes;
    let lift_s = travel.outlines as f64 * p.lift_time * passes;
    TimeEstimate {
        cut_length,
        travel: *travel,
        passes: p.passes,
        cut_s,
        travel_s,
        lift_s,
        total_s: cut_s + travel_s + lift_s,
    }
}

/// Seconds as hours, minutes and whole seconds (rounded).
pub fn hms(seconds: f64) -> (u64, u64, u64) {
    let s = if seconds.is_finite() && seconds > 0.0 {
        // Saturating: an absurd time shows as the largest one.
        seconds.round().min(u64::MAX as f64) as u64
    } else {
        0
    };
    (s / 3600, (s / 60) % 60, s % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Rect, Shape as _};

    fn square(x: f64, y: f64, s: f64) -> BezPath {
        Rect::new(x, y, x + s, y + s).to_path(0.01)
    }

    fn line(a: (f64, f64), b: (f64, f64)) -> BezPath {
        let mut p = BezPath::new();
        p.move_to(a);
        p.line_to(b);
        p
    }

    #[test]
    fn the_default_profile_is_the_documented_one() {
        let p = TimeProfile::default();
        assert_eq!(p.cut_speed, 300.0);
        assert_eq!(p.travel_speed, 600.0);
        assert_eq!(p.lift_time, 0.2);
        assert_eq!(p.accel_factor, 1.2);
        assert_eq!(p.passes, 1);
    }

    #[test]
    fn time_adds_cutting_travel_and_lifts_times_passes() {
        let travel = Travel {
            travel_length: 600.0,
            outlines: 5,
            as_drawn_fallback: false,
        };
        let p = TimeProfile {
            passes: 2,
            ..Default::default()
        };
        let e = estimate(3000.0, &travel, &p);
        // 3000 / 300 x 1.2 x 2 = 24; 600 / 600 x 1.2 x 2 = 2.4; 5 x 0.2 x 2 = 2.
        assert!((e.cut_s - 24.0).abs() < 1e-9);
        assert!((e.travel_s - 2.4).abs() < 1e-9);
        assert!((e.lift_s - 2.0).abs() < 1e-9);
        assert!((e.total_s - 28.4).abs() < 1e-9);
    }

    #[test]
    fn values_out_of_range_are_brought_in() {
        let bad = TimeProfile {
            name: "x".into(),
            cut_speed: 0.0,
            travel_speed: f64::NAN,
            lift_time: -1.0,
            accel_factor: f64::INFINITY,
            passes: 0,
        };
        let s = bad.sanitized();
        assert_eq!(s.cut_speed, SPEED_RANGE.0);
        assert_eq!(s.travel_speed, 600.0);
        assert_eq!(s.lift_time, 0.0);
        assert_eq!(s.accel_factor, 1.2);
        assert_eq!(s.passes, 1);
        let e = estimate(f64::NAN, &Travel::default(), &bad);
        assert!(e.total_s.is_finite());
        assert_eq!(e.cut_s, 0.0);
    }

    #[test]
    fn nearest_order_takes_the_closest_outline_next() {
        // Drawn far one first; the near one is 10 mm from the origin.
        let far = square(100.0, 0.0, 10.0);
        let near = square(10.0, 0.0, 10.0);
        let paths = [far, near];
        let drawn = travel(&paths, Point::ZERO, TravelOrder::AsDrawn);
        // 100 to the far one, back 90 to the near one.
        assert!((drawn.travel_length - 190.0).abs() < 1e-9);
        let near = travel(&paths, Point::ZERO, TravelOrder::Nearest);
        // 10 to the near one, then from (10,0) to the far one's (100,0).
        assert!((near.travel_length - 100.0).abs() < 1e-9);
        assert_eq!(near.outlines, 2);
    }

    #[test]
    fn closed_outlines_start_at_their_nearest_node_and_lines_at_either_end() {
        // The square's nearest node to (50,50) is (30,30).
        let sq = square(0.0, 0.0, 30.0);
        let t = travel(
            std::slice::from_ref(&sq),
            Point::new(50.0, 30.0),
            TravelOrder::Nearest,
        );
        assert!((t.travel_length - 20.0).abs() < 1e-9);
        // A line entered at its far end leaves at its near end.
        let l = line((100.0, 0.0), (10.0, 0.0));
        let t = travel(
            &[square(101.0, 0.0, 1.0), l],
            Point::ZERO,
            TravelOrder::Nearest,
        );
        // 10 to (10,0), cut to (100,0), then 1 to the square at (101,0).
        assert!((t.travel_length - 11.0).abs() < 1e-9);
    }

    #[test]
    fn subpaths_are_outlines_of_their_own() {
        let mut p = square(0.0, 0.0, 10.0);
        p.extend(square(2.0, 2.0, 2.0));
        let t = travel(&[p], Point::ZERO, TravelOrder::AsDrawn);
        assert_eq!(t.outlines, 2);
        assert!((t.travel_length - 8.0_f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn nothing_to_cut_takes_no_time() {
        let t = travel(&[], Point::new(f64::NAN, 0.0), TravelOrder::Nearest);
        assert_eq!(t, Travel::default());
        let e = estimate(0.0, &t, &TimeProfile::default());
        assert_eq!(e.total_s, 0.0);
    }

    #[test]
    fn non_finite_nodes_are_left_out() {
        let mut p = BezPath::new();
        p.move_to((f64::NAN, f64::NAN));
        p.line_to((f64::INFINITY, 0.0));
        let t = travel(
            &[p, line((3.0, 4.0), (6.0, 8.0))],
            Point::ZERO,
            TravelOrder::Nearest,
        );
        assert_eq!(t.outlines, 1);
        assert!((t.travel_length - 5.0).abs() < 1e-9);
    }

    #[test]
    fn the_moves_add_up_to_the_travel() {
        let paths = [square(100.0, 0.0, 10.0), square(10.0, 0.0, 10.0)];
        for order in [TravelOrder::Nearest, TravelOrder::AsDrawn] {
            let (t, moves) = travel_moves(&paths, Point::ZERO, order);
            assert_eq!(t, travel(&paths, Point::ZERO, order));
            assert_eq!(moves.len(), 2);
            let sum: f64 = moves.iter().map(|(a, b)| (*b - *a).hypot()).sum();
            assert!((sum - t.travel_length).abs() < 1e-9);
            assert_eq!(moves[0].0, Point::ZERO);
        }
        let (t, moves) = travel_moves(&[], Point::ZERO, TravelOrder::Nearest);
        assert_eq!(t.outlines, 0);
        assert!(moves.is_empty());
    }

    #[test]
    fn hours_minutes_and_seconds() {
        assert_eq!(hms(3725.4), (1, 2, 5));
        assert_eq!(hms(59.6), (0, 1, 0));
        assert_eq!(hms(f64::NAN), (0, 0, 0));
        assert_eq!(hms(-5.0), (0, 0, 0));
        let (h, _, _) = hms(f64::MAX);
        assert!(h > 0);
    }
}
