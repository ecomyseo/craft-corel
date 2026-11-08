//! Bridges (tabs): short stretches of a closed outline left uncut so the
//! part stays in the sheet until it is broken out.
//!
//! A closed subpath of length `L` longer than `min_outline` gets `n`
//! bridges of `length` each: `n` is the count asked for, or `L / distance`
//! rounded (at least 1). They are spread evenly by arc length, the first
//! half a spacing from the start node (`L / 2n`, `3L / 2n`, ...), and each
//! one moves to the nearest place in the middle half of a straight
//! segment (at least a bridge length from its ends) when there is one
//! within a quarter spacing. The outline is then cut into
//! open pieces between the bridges; curves keep their exact shape. An
//! outline where the bridges would take more than nine tenths of its
//! length is left whole. Open subpaths stay as they are.

use super::{fin, finite_path};
use crate::measure::ARCLEN_ACCURACY;
use tracedraw_core::geometry::{BezPath, ParamCurve, ParamCurveArclen, PathEl, PathSeg, Point};

/// The most bridges one outline gets.
pub const MAX_BRIDGES: u32 = 1000;

/// How many bridges an outline gets.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BridgeCount {
    /// This many per closed outline.
    PerOutline(u32),
    /// One every so many millimetres of outline (at least one).
    Every(f64),
}

/// Settings of [`bridge_path`], lengths in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct BridgeSpec {
    /// Length of each bridge.
    pub length: f64,
    pub count: BridgeCount,
    /// Only outlines longer than this get bridges.
    pub min_outline: f64,
}

impl Default for BridgeSpec {
    fn default() -> Self {
        BridgeSpec {
            length: 2.0,
            count: BridgeCount::PerOutline(2),
            min_outline: 0.0,
        }
    }
}

/// A path with bridges: the new path, how many bridges it got and where
/// their centres are.
#[derive(Debug, Clone)]
pub struct Bridged {
    pub path: BezPath,
    pub bridges: usize,
    pub centres: Vec<Point>,
}

/// Put bridges into the closed subpaths of `path`; `None` when no
/// subpath changed.
pub fn bridge_path(path: &BezPath, spec: &BridgeSpec) -> Option<Bridged> {
    let b = fin(spec.length, 0.0);
    if b <= 0.0 {
        return None;
    }
    let min = fin(spec.min_outline, 0.0).max(0.0);
    let mut out = BezPath::new();
    let mut total = 0usize;
    let mut centres = Vec::new();
    for sp in crate::subpaths(path) {
        let done = if sp.closed && finite_path(&sp.path) {
            bridge_subpath(&sp.path, b, min, spec.count)
        } else {
            None
        };
        match done {
            Some((pieces, n, at)) => {
                out.extend(pieces.elements().iter().copied());
                total += n;
                centres.extend(at);
            }
            None => out.extend(sp.path.elements().iter().copied()),
        }
    }
    (total > 0).then_some(Bridged {
        path: out,
        bridges: total,
        centres,
    })
}

/// Bridge positions (centres, by arc length from the start) for an
/// outline of segments `segs` with lengths `lens`.
fn positions(segs: &[PathSeg], lens: &[f64], b: f64, n: usize) -> Vec<f64> {
    let total: f64 = lens.iter().sum();
    let step = total / n as f64;
    // Places on straight segments where a bridge fits.
    let mut ranges: Vec<(f64, f64)> = Vec::new();
    let mut s = 0.0;
    for (seg, l) in segs.iter().zip(lens) {
        if matches!(seg, PathSeg::Line(_)) && *l >= 2.0 * b {
            let keep = b.max(l / 4.0);
            ranges.push((s + keep, s + l - keep));
        }
        s += l;
    }
    let window = step / 4.0;
    let ideal: Vec<f64> = (0..n).map(|i| (i as f64 + 0.5) * step).collect();
    let mut placed: Vec<f64> = ideal
        .iter()
        .map(|&c| {
            let mut best: Option<(f64, f64)> = None;
            for &(lo, hi) in &ranges {
                for shift in [-total, 0.0, total] {
                    let at = c.clamp(lo + shift, hi + shift);
                    let d = (at - c).abs();
                    if d <= window && best.is_none_or(|(bd, _)| d < bd) {
                        best = Some((d, at - shift));
                    }
                }
            }
            best.map_or(c, |(_, at)| at.rem_euclid(total))
        })
        .collect();
    // Snapping must not make neighbours overlap: those go back.
    for i in 0..n {
        if n < 2 {
            break;
        }
        let j = (i + 1) % n;
        let mut d = placed[j] - placed[i];
        if j == 0 {
            d += total;
        }
        if d < b * 1.5 {
            placed[i] = ideal[i];
            placed[j] = ideal[j];
        }
    }
    placed.sort_by(f64::total_cmp);
    placed
}

fn bridge_subpath(
    path: &BezPath,
    b: f64,
    min: f64,
    count: BridgeCount,
) -> Option<(BezPath, usize, Vec<Point>)> {
    let segs: Vec<PathSeg> = path.segments().collect();
    let lens: Vec<f64> = segs
        .iter()
        .map(|s| s.arclen(ARCLEN_ACCURACY))
        .map(|l| if l.is_finite() { l.max(0.0) } else { 0.0 })
        .collect();
    let total: f64 = lens.iter().sum();
    if total.partial_cmp(&min) != Some(std::cmp::Ordering::Greater) || total <= 0.0 {
        return None;
    }
    let n = match count {
        BridgeCount::PerOutline(c) => c.min(MAX_BRIDGES) as usize,
        BridgeCount::Every(d) => {
            let d = fin(d, 0.0);
            if d <= 0.0 {
                1
            } else {
                ((total / d).round().max(1.0)).min(f64::from(MAX_BRIDGES)) as usize
            }
        }
    };
    if n == 0 || n as f64 * b > 0.9 * total {
        return None;
    }
    let centres = positions(&segs, &lens, b, n);
    let mut out = BezPath::new();
    for (i, c) in centres.iter().enumerate() {
        let next = match centres.get(i + 1) {
            Some(x) => *x,
            None => centres.first().map_or(*c, |f| f + total),
        };
        piece(&segs, &lens, c + b / 2.0, next - b / 2.0, &mut out);
    }
    let at = centres
        .iter()
        .filter_map(|c| point_at(&segs, &lens, *c))
        .collect();
    Some((out, n, at))
}

/// The point at arc length `at` along the outline (wrapping round).
fn point_at(segs: &[PathSeg], lens: &[f64], at: f64) -> Option<Point> {
    let total: f64 = lens.iter().sum();
    if total <= 0.0 || !at.is_finite() {
        return None;
    }
    let at = at.rem_euclid(total);
    let mut s = 0.0;
    for (seg, l) in segs.iter().zip(lens) {
        if *l > 0.0 && at <= s + l {
            return Some(seg.eval(seg.inv_arclen(at - s, ARCLEN_ACCURACY)));
        }
        s += l;
    }
    segs.last().map(|seg| seg.end())
}

/// Append the open piece of the outline between arc lengths `from` and
/// `to` (`to` may pass the end once: it goes on from the start).
fn piece(segs: &[PathSeg], lens: &[f64], from: f64, to: f64, out: &mut BezPath) {
    if to - from <= 1e-9 {
        return;
    }
    let total: f64 = lens.iter().sum();
    let mut started = false;
    let mut s = 0.0;
    for _ in 0..3 {
        for (seg, l) in segs.iter().zip(lens) {
            let (a, b) = (s, s + l);
            s = b;
            if *l <= 0.0 || b <= from || a >= to {
                continue;
            }
            let t0 = if from > a {
                seg.inv_arclen(from - a, ARCLEN_ACCURACY)
            } else {
                0.0
            };
            let t1 = if to < b {
                seg.inv_arclen(to - a, ARCLEN_ACCURACY)
            } else {
                1.0
            };
            if t1 - t0 <= 1e-12 {
                continue;
            }
            let part = seg.subsegment(t0..t1);
            if !started {
                out.move_to(part.start());
                started = true;
            }
            out.push(match part {
                PathSeg::Line(l) => PathEl::LineTo(l.p1),
                PathSeg::Quad(q) => PathEl::QuadTo(q.p1, q.p2),
                PathSeg::Cubic(c) => PathEl::CurveTo(c.p1, c.p2, c.p3),
            });
        }
        if s >= to || total <= 0.0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cut::rect_path;
    use crate::measure::length;
    use tracedraw_core::geometry::{Circle, Point, Rect, Shape as _};

    fn pieces(p: &BezPath) -> Vec<BezPath> {
        crate::subpaths(p).into_iter().map(|s| s.path).collect()
    }

    #[test]
    fn a_square_gets_two_bridges_in_the_middle_of_its_sides() {
        let sq = rect_path(Rect::new(0.0, 0.0, 100.0, 100.0));
        let r = bridge_path(&sq, &BridgeSpec::default()).expect("bridged");
        assert_eq!(r.bridges, 2);
        let ps = pieces(&r.path);
        assert_eq!(ps.len(), 2);
        assert!(crate::subpaths(&r.path).iter().all(|s| !s.closed));
        // 400 mm of outline less two bridges of 2 mm.
        assert!((length(&r.path) - 396.0).abs() < 1e-6);
        // Ideal centres at 100 and 300 are corners: moved onto the sides.
        let ends: Vec<Point> = ps
            .iter()
            .flat_map(|p| {
                let e = p.elements();
                [
                    e.first().and_then(|e| e.end_point()),
                    e.last().and_then(|e| e.end_point()),
                ]
            })
            .flatten()
            .collect();
        for e in ends {
            let on_corner = [(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]
                .iter()
                .any(|(x, y)| (e - Point::new(*x, *y)).hypot() < 2.0);
            assert!(!on_corner, "{e:?}");
        }
        // The centres lie on the sides, away from the corners.
        assert_eq!(r.centres.len(), 2);
        for c in &r.centres {
            let on_side = c.x.abs() < 1e-6
                || (c.x - 100.0).abs() < 1e-6
                || c.y.abs() < 1e-6
                || (c.y - 100.0).abs() < 1e-6;
            assert!(on_side, "{c:?}");
            assert!(c.x > 2.0 && c.x < 98.0 || c.y > 2.0 && c.y < 98.0, "{c:?}");
        }
    }

    #[test]
    fn bridges_by_distance_and_minimum_length() {
        let sq = rect_path(Rect::new(0.0, 0.0, 100.0, 100.0));
        let spec = BridgeSpec {
            length: 3.0,
            count: BridgeCount::Every(100.0),
            min_outline: 0.0,
        };
        let r = bridge_path(&sq, &spec).expect("bridged");
        assert_eq!(r.bridges, 4);
        assert!((length(&r.path) - 388.0).abs() < 1e-6);
        let small = BridgeSpec {
            min_outline: 500.0,
            ..spec
        };
        assert!(bridge_path(&sq, &small).is_none());
    }

    #[test]
    fn curves_keep_their_shape_between_bridges() {
        let c = Circle::new((0.0, 0.0), 50.0).to_path(1e-4);
        let r = bridge_path(
            &c,
            &BridgeSpec {
                count: BridgeCount::PerOutline(4),
                ..Default::default()
            },
        )
        .expect("bridged");
        assert_eq!(r.bridges, 4);
        let full = length(&c);
        assert!((length(&r.path) - (full - 8.0)).abs() < 1e-3);
        for el in r.path.elements() {
            if let Some(p) = el.end_point() {
                assert!(((p - Point::ZERO).hypot() - 50.0).abs() < 0.01);
            }
        }
        assert_eq!(pieces(&r.path).len(), 4);
    }

    #[test]
    fn open_paths_and_crowded_outlines_stay() {
        let mut open = BezPath::new();
        open.move_to((0.0, 0.0));
        open.line_to((100.0, 0.0));
        assert!(bridge_path(&open, &BridgeSpec::default()).is_none());
        let tiny = rect_path(Rect::new(0.0, 0.0, 1.0, 1.0));
        assert!(bridge_path(&tiny, &BridgeSpec::default()).is_none());
        // A ring: both outlines get bridges.
        let mut ring = rect_path(Rect::new(0.0, 0.0, 50.0, 50.0));
        ring.extend(rect_path(Rect::new(10.0, 10.0, 40.0, 40.0)));
        ring.move_to((70.0, 0.0));
        ring.line_to((80.0, 0.0));
        let r = bridge_path(&ring, &BridgeSpec::default()).expect("bridged");
        assert_eq!(r.bridges, 4);
        assert_eq!(pieces(&r.path).len(), 5);
    }

    #[test]
    fn non_finite_input_and_settings_do_not_panic() {
        let sq = rect_path(Rect::new(0.0, 0.0, 10.0, 10.0));
        for spec in [
            BridgeSpec {
                length: f64::NAN,
                ..Default::default()
            },
            BridgeSpec {
                length: -1.0,
                ..Default::default()
            },
            BridgeSpec {
                count: BridgeCount::Every(f64::NAN),
                ..Default::default()
            },
            BridgeSpec {
                count: BridgeCount::PerOutline(u32::MAX),
                ..Default::default()
            },
            BridgeSpec {
                min_outline: f64::INFINITY,
                ..Default::default()
            },
        ] {
            let _ = bridge_path(&sq, &spec);
        }
        let mut bad = BezPath::new();
        bad.move_to((0.0, 0.0));
        bad.line_to((f64::INFINITY, 0.0));
        bad.line_to((0.0, 5.0));
        bad.close_path();
        assert!(bridge_path(&bad, &BridgeSpec::default()).is_none());
        assert!(bridge_path(&BezPath::new(), &BridgeSpec::default()).is_none());
    }
}
