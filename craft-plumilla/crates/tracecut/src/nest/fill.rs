//! Fill Sheet: fill a sheet with copies of one box in a grid.
//!
//! The grid runs from the origin corner, `gap` between copies and the
//! margin kept along the edges. With turning allowed, six plans are
//! compared and the one with the most copies wins (fewer turned copies,
//! then the earlier plan, on a tie): every copy upright, every copy
//! turned 90 degrees, and either grid with the strip left over at its
//! far side or its far end filled by copies the other way round.

use super::{finite_rect, Frame, SheetSpec};
use tracedraw_core::geometry::{Affine, Rect, Vec2};

/// Copies made at most, whatever the limit.
pub const MAX_COPIES: usize = 10_000;

/// What Fill Sheet does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillSettings {
    /// The sheet: width, height, margin and corner (a roll is taken as
    /// a sheet of the width and height given).
    pub sheet: SheetSpec,
    /// Distance between copies (mm).
    pub gap: f64,
    /// Copies may turn 90 degrees where more fit.
    pub rotate: bool,
    /// At most this many copies, the original's place included (0: as
    /// many as fit).
    pub limit: usize,
}

impl Default for FillSettings {
    fn default() -> Self {
        FillSettings {
            sheet: SheetSpec::default(),
            gap: 2.0,
            rotate: true,
            limit: 0,
        }
    }
}

/// One copy: the page-space transform for the original object and
/// whether it is turned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Copy {
    pub transform: Affine,
    pub turned: bool,
}

const EPS: f64 = 1e-9;

/// Cells of a grid of `w` x `h` boxes in `big_w` x `big_h` from
/// `(x0, y0)`, row by row, at most `cap` of them; with the width and
/// height the whole grid takes.
#[allow(clippy::too_many_arguments)]
fn grid(
    x0: f64,
    y0: f64,
    big_w: f64,
    big_h: f64,
    (w, h): (f64, f64),
    g: f64,
    turned: bool,
    cap: usize,
) -> (Vec<(f64, f64, bool)>, f64, f64) {
    if big_w + EPS < w || big_h + EPS < h {
        return (Vec::new(), 0.0, 0.0);
    }
    let nx = (((big_w + g) / (w + g)) + EPS).floor().clamp(0.0, 1e9);
    let ny = (((big_h + g) / (h + g)) + EPS).floor().clamp(0.0, 1e9);
    let (nx, ny) = (nx as usize, ny as usize);
    let mut cells = Vec::new();
    'rows: for j in 0..ny {
        for i in 0..nx {
            if cells.len() >= cap {
                break 'rows;
            }
            cells.push((x0 + i as f64 * (w + g), y0 + j as f64 * (h + g), turned));
        }
    }
    (cells, nx as f64 * (w + g), ny as f64 * (h + g))
}

/// The copies of an object (or selection) with page-space `bounds`
/// filling the sheet; the first is where the original goes.
pub fn fill_sheet(bounds: Rect, s: &FillSettings) -> Vec<Copy> {
    let mut spec = s.sheet;
    spec.roll = false;
    let Some(spec) = spec.sane() else {
        return Vec::new();
    };
    let Some(b) = finite_rect(bounds) else {
        return Vec::new();
    };
    let g = if s.gap.is_finite() {
        s.gap.max(0.0)
    } else {
        0.0
    };
    let frame = Frame::new(&spec);
    let norm = frame.to_norm();
    let nb = norm.transform_rect_bbox(b);
    let (w, h) = (nb.width(), nb.height());
    if w + g <= EPS || h + g <= EPS {
        return Vec::new();
    }
    let m = spec.margin;
    let (big_w, big_h) = (spec.width - 2.0 * m, spec.height - 2.0 * m);
    if big_w < -EPS || big_h < -EPS {
        return Vec::new();
    }
    let mut plans: Vec<Vec<(f64, f64, bool)>> = Vec::new();
    let turns: &[(f64, f64, bool)] = if s.rotate && (w - h).abs() > EPS {
        &[(w, h, false), (h, w, true)]
    } else {
        &[(w, h, false)]
    };
    let cap = if s.limit > 0 {
        s.limit.min(MAX_COPIES)
    } else {
        MAX_COPIES
    };
    for &(a, c, t) in turns {
        let (cells, used_w, used_h) = grid(0.0, 0.0, big_w, big_h, (a, c), g, t, cap);
        plans.push(cells.clone());
        if turns.len() == 2 {
            let (oa, oc, ot) = (c, a, !t);
            // The strip at the far side, then the one at the far end.
            let (side, _, _) = grid(used_w, 0.0, big_w - used_w, big_h, (oa, oc), g, ot, cap);
            let mut p = cells.clone();
            p.extend(side);
            plans.push(p);
            let (end, _, _) = grid(0.0, used_h, big_w, big_h - used_h, (oa, oc), g, ot, cap);
            let mut p = cells;
            p.extend(end);
            plans.push(p);
        }
    }
    let turned = |p: &Vec<(f64, f64, bool)>| p.iter().filter(|c| c.2).count();
    let mut best: Vec<(f64, f64, bool)> = Vec::new();
    for p in plans {
        if p.len() > best.len() || (p.len() == best.len() && turned(&p) < turned(&best)) {
            best = p;
        }
    }
    best.truncate(cap);
    best.into_iter()
        .map(|(x, y, t)| {
            let rot = if t {
                Affine::rotate(std::f64::consts::FRAC_PI_2)
            } else {
                Affine::IDENTITY
            };
            let rb = (rot * norm).transform_rect_bbox(b);
            let local = Affine::translate(-rb.origin().to_vec2()) * rot * norm;
            Copy {
                transform: frame.place(0, m, Vec2::new(x, y), local),
                turned: t,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::Corner;
    use super::*;

    fn settings(w: f64, h: f64, gap: f64, margin: f64) -> FillSettings {
        FillSettings {
            sheet: SheetSpec {
                width: w,
                height: h,
                margin,
                ..Default::default()
            },
            gap,
            rotate: false,
            limit: 0,
        }
    }

    #[test]
    fn a_grid_fills_the_sheet_inside_its_margin() {
        let b = Rect::new(300.0, 300.0, 20.0 + 300.0, 10.0 + 300.0);
        let s = settings(100.0, 50.0, 2.0, 5.0);
        let c = fill_sheet(b, &s);
        // (90 + 2) / 22 = 4 across, (40 + 2) / 12 = 3 up.
        assert_eq!(c.len(), 12);
        let first = c[0].transform.transform_rect_bbox(b);
        assert!((first.x0 - 5.0).abs() < 1e-9 && (first.y0 - 5.0).abs() < 1e-9);
        let last = c[11].transform.transform_rect_bbox(b);
        assert!((last.x0 - (5.0 + 3.0 * 22.0)).abs() < 1e-9);
        assert!((last.y1 - (5.0 + 2.0 * 12.0 + 10.0)).abs() < 1e-9);
        assert!(last.x1 <= 95.0 + 1e-9 && last.y1 <= 45.0 + 1e-9);
    }

    #[test]
    fn turned_copies_fill_the_strip_left_over() {
        // 30 x 20 on 80 x 50: upright 2 x 2; turned (20 x 30) 4 x 1 with
        // the 80 x 20 strip above taking 2 upright: 6.
        let b = Rect::new(0.0, 0.0, 30.0, 20.0);
        let mut s = settings(80.0, 50.0, 0.0, 0.0);
        assert_eq!(fill_sheet(b, &s).len(), 4);
        s.rotate = true;
        let c = fill_sheet(b, &s);
        assert_eq!(c.len(), 6);
        assert_eq!(c.iter().filter(|c| c.turned).count(), 4);
        let last = c[5].transform.transform_rect_bbox(b);
        assert!(!c[5].turned && (last.y0 - 30.0).abs() < 1e-9 && (last.x0 - 30.0).abs() < 1e-9);
        // Upright 2 x 2 with one turned in the 20 wide strip: 5 on 80 x 40,
        // where turned ones give 4 x 1 and nothing above.
        let s = FillSettings {
            rotate: true,
            ..settings(80.0, 40.0, 0.0, 0.0)
        };
        let c = fill_sheet(b, &s);
        assert_eq!(c.len(), 5);
        assert_eq!(c.iter().filter(|c| c.turned).count(), 1);
    }

    #[test]
    fn the_limit_and_the_corner_apply() {
        let b = Rect::new(0.0, 0.0, 10.0, 10.0);
        let mut s = settings(100.0, 100.0, 0.0, 0.0);
        s.limit = 5;
        s.sheet.corner = Corner::TopRight;
        let c = fill_sheet(b, &s);
        assert_eq!(c.len(), 5);
        let first = c[0].transform.transform_rect_bbox(b);
        assert!((first.x1 - 100.0).abs() < 1e-9 && (first.y1 - 100.0).abs() < 1e-9);
        let fifth = c[4].transform.transform_rect_bbox(b);
        assert!((fifth.x0 - 50.0).abs() < 1e-9 && (fifth.y1 - 100.0).abs() < 1e-9);
    }

    #[test]
    fn nothing_fits_or_bad_input_gives_no_copies() {
        let s = settings(100.0, 100.0, 2.0, 0.0);
        assert!(fill_sheet(Rect::new(0.0, 0.0, 200.0, 10.0), &s).is_empty());
        assert!(fill_sheet(Rect::new(f64::NAN, 0.0, 1.0, 1.0), &s).is_empty());
        assert!(fill_sheet(
            Rect::new(0.0, 0.0, 0.0, 0.0),
            &settings(100.0, 100.0, 0.0, 0.0)
        )
        .is_empty());
        assert!(fill_sheet(
            Rect::new(0.0, 0.0, 1.0, 1.0),
            &settings(f64::NAN, 100.0, 2.0, 0.0)
        )
        .is_empty());
        assert!(fill_sheet(
            Rect::new(0.0, 0.0, 1.0, 1.0),
            &settings(10.0, 10.0, 2.0, 6.0)
        )
        .is_empty());
        // Far too many tiny copies: no more than the most made.
        let tiny = fill_sheet(
            Rect::new(0.0, 0.0, 0.01, 0.01),
            &settings(10_000.0, 10_000.0, 0.0, 0.0),
        );
        assert_eq!(tiny.len(), MAX_COPIES);
        // A flat line still repeats across the gap.
        let line = fill_sheet(
            Rect::new(0.0, 0.0, 10.0, 0.0),
            &settings(10.0, 10.0, 2.0, 0.0),
        );
        assert_eq!(line.len(), 6);
    }
}
