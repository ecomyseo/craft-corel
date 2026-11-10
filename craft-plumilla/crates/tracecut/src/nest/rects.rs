//! Rectangle nesting: objects placed by their bounding boxes with the
//! MaxRects method.
//!
//! Each sheet keeps the list of its largest free rectangles (they may
//! overlap). A box goes into the free rectangle that leaves the shortest
//! side over (best short side fit; on a roll, the place whose top is
//! lowest, then the leftmost), then every free rectangle it covers is
//! split into the parts around it and rectangles inside others are
//! dropped. Boxes go largest area first; each takes the first sheet it
//! fits on, a new sheet when none has room. The spacing is kept by
//! growing every box and the usable area by it.

use super::{finite_rect, Frame, Layout, Placement, SheetSpec};
use tracedraw_core::geometry::{Affine, Point, Rect, Vec2};

/// What rectangle nesting does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectSettings {
    pub sheet: SheetSpec,
    /// Boxes may turn 90 degrees.
    pub rotate: bool,
}

impl Default for RectSettings {
    fn default() -> Self {
        RectSettings {
            sheet: SheetSpec::default(),
            rotate: true,
        }
    }
}

/// One object: its page-space bounds and its net area (for the
/// utilisation; a negative or non-finite area counts its box).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectItem {
    pub bounds: Rect,
    pub area: f64,
}

const EPS: f64 = 1e-9;

/// A box's size as placed and whether it is turned.
type Turn = (f64, f64, bool);

#[derive(Debug, Clone)]
struct Bin {
    free: Vec<Rect>,
}

impl Bin {
    fn new(w: f64, h: f64) -> Bin {
        Bin {
            free: vec![Rect::new(0.0, 0.0, w, h)],
        }
    }

    /// The best place for a `w` x `h` box and its score (lower is better).
    fn find(&self, w: f64, h: f64, lowest: bool) -> Option<(Point, (f64, f64))> {
        let mut best: Option<(Point, (f64, f64))> = None;
        for f in &self.free {
            if f.width() + EPS < w || f.height() + EPS < h {
                continue;
            }
            let score = if lowest {
                (f.y0 + h, f.x0)
            } else {
                let dw = f.width() - w;
                let dh = f.height() - h;
                (dw.min(dh), dw.max(dh))
            };
            if best.is_none_or(|(_, s)| score < s) {
                best = Some((Point::new(f.x0, f.y0), score));
            }
        }
        best
    }

    /// Take `used` out of the free rectangles.
    fn take(&mut self, used: Rect) {
        let mut pieces = Vec::new();
        self.free.retain(|f| {
            let meets = f.x0 < used.x1 - EPS
                && used.x0 < f.x1 - EPS
                && f.y0 < used.y1 - EPS
                && used.y0 < f.y1 - EPS;
            if !meets {
                return true;
            }
            if used.x0 > f.x0 + EPS {
                pieces.push(Rect::new(f.x0, f.y0, used.x0, f.y1));
            }
            if used.x1 < f.x1 - EPS {
                pieces.push(Rect::new(used.x1, f.y0, f.x1, f.y1));
            }
            if used.y0 > f.y0 + EPS {
                pieces.push(Rect::new(f.x0, f.y0, f.x1, used.y0));
            }
            if used.y1 < f.y1 - EPS {
                pieces.push(Rect::new(f.x0, used.y1, f.x1, f.y1));
            }
            false
        });
        self.free.extend(pieces);
        // Drop rectangles inside others (of equal ones, keep the first).
        let contains = |a: &Rect, b: &Rect| {
            a.x0 <= b.x0 + EPS && a.y0 <= b.y0 + EPS && a.x1 >= b.x1 - EPS && a.y1 >= b.y1 - EPS
        };
        let n = self.free.len();
        let mut keep = vec![true; n];
        for i in 0..n {
            for j in 0..n {
                if i == j || !keep[j] {
                    continue;
                }
                if contains(&self.free[j], &self.free[i]) {
                    let equal = contains(&self.free[i], &self.free[j]);
                    if !equal || j < i {
                        keep[i] = false;
                        break;
                    }
                }
            }
        }
        let mut k = keep.iter();
        self.free.retain(|_| k.next().copied().unwrap_or(true));
    }
}

/// Place `items` by their bounding boxes. Boxes that fit on no sheet
/// (or across no roll) are left out.
pub fn nest_rects(items: &[RectItem], s: &RectSettings) -> Layout {
    let mut layout = Layout {
        placements: vec![None; items.len()],
        ..Default::default()
    };
    let Some(spec) = s.sheet.sane() else {
        return layout;
    };
    let frame = Frame::new(&spec);
    let norm = frame.to_norm();
    let (across, along) = spec.across_along();
    let (m, sp) = (spec.margin, spec.spacing);
    // Each box and the usable area grow by the spacing.
    let bin_w = across - 2.0 * m + sp;
    let boxes: Vec<Option<Rect>> = items
        .iter()
        .map(|it| finite_rect(it.bounds).map(|b| norm.transform_rect_bbox(b)))
        .collect();
    let bin_h = if spec.roll {
        boxes
            .iter()
            .flatten()
            .map(|b| b.width().max(b.height()) + sp)
            .sum::<f64>()
            + 1.0
    } else {
        along - 2.0 * m + sp
    };
    if !(bin_w > EPS && bin_h > EPS) {
        return layout;
    }
    let mut order: Vec<usize> = (0..items.len()).filter(|i| boxes[*i].is_some()).collect();
    let area = |i: usize| boxes[i].map_or(0.0, |b| b.area());
    order.sort_by(|a, b| area(*b).total_cmp(&area(*a)).then(a.cmp(b)));

    let mut bins: Vec<Bin> = Vec::new();
    let mut extents: Vec<(usize, f64)> = Vec::new();
    for i in order {
        let Some(b) = boxes[i] else { continue };
        let (w, h) = (b.width(), b.height());
        let mut turns: Vec<Turn> = vec![(w, h, false)];
        if s.rotate && (w - h).abs() > EPS {
            turns.push((h, w, true));
        }
        let fits_empty = turns
            .iter()
            .any(|(tw, th, _)| tw + sp <= bin_w + EPS && th + sp <= bin_h + EPS);
        if !fits_empty {
            continue;
        }
        let mut spot: Option<(usize, Point, f64, f64, bool)> = None;
        for _attempt in 0..2 {
            for (bi, bin) in bins.iter().enumerate() {
                let mut best: Option<(Point, (f64, f64), Turn)> = None;
                for t in &turns {
                    if let Some((p, score)) = bin.find(t.0 + sp, t.1 + sp, spec.roll) {
                        if best.as_ref().is_none_or(|(_, s, _)| score < *s) {
                            best = Some((p, score, *t));
                        }
                    }
                }
                if let Some((p, _, (tw, th, turned))) = best {
                    spot = Some((bi, p, tw, th, turned));
                    break;
                }
            }
            if spot.is_some() || (spec.roll && !bins.is_empty()) {
                break;
            }
            bins.push(Bin::new(bin_w, bin_h));
        }
        let Some((bi, p, tw, th, turned)) = spot else {
            continue;
        };
        if let Some(bin) = bins.get_mut(bi) {
            bin.take(Rect::new(p.x, p.y, p.x + tw + sp, p.y + th + sp));
        }
        let rot = if turned {
            Affine::rotate(std::f64::consts::FRAC_PI_2)
        } else {
            Affine::IDENTITY
        };
        let rb = (rot * norm).transform_rect_bbox(finite_rect(items[i].bounds).unwrap_or(b));
        let local = Affine::translate(-rb.origin().to_vec2()) * rot * norm;
        let transform = frame.place(bi, m, Vec2::new(p.x, p.y), local);
        layout.placements[i] = Some(Placement {
            sheet: bi,
            transform,
            angle: if turned { 90.0 } else { 0.0 },
        });
        extents.push((bi, m + p.y + th));
        let a = items[i].area;
        layout.parts_area += if a.is_finite() && a >= 0.0 { a } else { w * h };
    }
    layout.finish(&spec, &extents);
    layout
}

#[cfg(test)]
mod tests {
    use super::super::{Corner, RollDir, SHEET_GAP};
    use super::*;

    fn item(x: f64, y: f64, w: f64, h: f64) -> RectItem {
        RectItem {
            bounds: Rect::new(x, y, x + w, y + h),
            area: w * h,
        }
    }

    fn sheet(w: f64, h: f64, spacing: f64) -> RectSettings {
        RectSettings {
            sheet: SheetSpec {
                width: w,
                height: h,
                spacing,
                ..Default::default()
            },
            rotate: true,
        }
    }

    fn placed(items: &[RectItem], l: &Layout) -> Vec<Option<Rect>> {
        items
            .iter()
            .zip(&l.placements)
            .map(|(it, p)| p.map(|p| p.transform.transform_rect_bbox(it.bounds)))
            .collect()
    }

    fn apart(rects: &[Rect], gap: f64) -> bool {
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                let dx = (b.x0 - a.x1).max(a.x0 - b.x1);
                let dy = (b.y0 - a.y1).max(a.y0 - b.y1);
                if dx.max(dy) < gap - 1e-6 {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn four_squares_fill_a_sheet() {
        let items: Vec<RectItem> = (0..4)
            .map(|i| item(i as f64 * 70.0, 300.0, 50.0, 50.0))
            .collect();
        let l = nest_rects(&items, &sheet(100.0, 100.0, 0.0));
        assert_eq!(l.placed(), 4);
        assert_eq!(l.sheets, 1);
        assert!((l.utilisation() - 1.0).abs() < 1e-9);
        let r: Vec<Rect> = placed(&items, &l).into_iter().flatten().collect();
        assert!(apart(&r, 0.0));
        for b in &r {
            assert!(b.x0 > -1e-9 && b.y0 > -1e-9 && b.x1 < 100.0 + 1e-9 && b.y1 < 100.0 + 1e-9);
        }
    }

    #[test]
    fn spacing_is_kept_between_boxes() {
        let items = vec![item(0.0, 0.0, 40.0, 40.0), item(0.0, 0.0, 40.0, 40.0)];
        let l = nest_rects(&items, &sheet(82.0, 40.0, 2.0));
        assert_eq!(l.placed(), 2);
        let r: Vec<Rect> = placed(&items, &l).into_iter().flatten().collect();
        assert!(apart(&r, 2.0));
        // One more millimetre of spacing and they no longer fit together.
        let l = nest_rects(&items, &sheet(82.0, 40.0, 3.0));
        assert_eq!(l.sheets, 2);
    }

    #[test]
    fn a_box_turns_when_it_only_fits_turned() {
        let items = vec![item(0.0, 0.0, 30.0, 90.0)];
        let l = nest_rects(&items, &sheet(100.0, 40.0, 0.0));
        let p = l.placements[0].expect("placed turned");
        assert_eq!(p.angle, 90.0);
        let b = p.transform.transform_rect_bbox(items[0].bounds);
        assert!((b.width() - 90.0).abs() < 1e-9 && (b.height() - 30.0).abs() < 1e-9);
        let mut s = sheet(100.0, 40.0, 0.0);
        s.rotate = false;
        assert_eq!(nest_rects(&items, &s).placed(), 0);
    }

    #[test]
    fn extra_sheets_go_to_the_right() {
        let items: Vec<RectItem> = (0..3).map(|_| item(0.0, 0.0, 60.0, 60.0)).collect();
        let l = nest_rects(&items, &sheet(100.0, 100.0, 0.0));
        assert_eq!(l.sheets, 3);
        assert!((l.used_area - 30_000.0).abs() < 1e-9);
        let r = placed(&items, &l);
        let xs: Vec<f64> = r.iter().flatten().map(|b| b.x0).collect();
        assert!(xs.contains(&0.0));
        assert!(xs.iter().any(|x| (x - (100.0 + SHEET_GAP)).abs() < 1e-9));
        assert!(xs
            .iter()
            .any(|x| (x - 2.0 * (100.0 + SHEET_GAP)).abs() < 1e-9));
    }

    #[test]
    fn a_roll_uses_the_least_length() {
        let items: Vec<RectItem> = (0..10).map(|_| item(0.0, 0.0, 30.0, 30.0)).collect();
        let mut s = sheet(100.0, 0.0, 0.0);
        s.sheet.roll = true;
        let l = nest_rects(&items, &s);
        assert_eq!(l.placed(), 10);
        assert_eq!(l.sheets, 1);
        assert!((l.length - 120.0).abs() < 1e-9, "{}", l.length);
        // The length runs to the right.
        let r: Vec<Rect> = placed(&items, &l).into_iter().flatten().collect();
        let x1 = r.iter().map(|b| b.x1).fold(0.0, f64::max);
        let y1 = r.iter().map(|b| b.y1).fold(0.0, f64::max);
        assert!((x1 - 120.0).abs() < 1e-9 && y1 <= 100.0 + 1e-9);
        assert!(apart(&r, 0.0));
        s.sheet.roll_dir = RollDir::Up;
        let l = nest_rects(&items, &s);
        let r: Vec<Rect> = placed(&items, &l).into_iter().flatten().collect();
        let y1 = r.iter().map(|b| b.y1).fold(0.0, f64::max);
        assert!((y1 - 120.0).abs() < 1e-9);
    }

    #[test]
    fn the_origin_corner_and_margin_place_the_first_box() {
        let items = vec![item(500.0, 500.0, 10.0, 10.0)];
        let mut s = sheet(100.0, 100.0, 0.0);
        s.sheet.corner = Corner::TopRight;
        s.sheet.margin = 5.0;
        let l = nest_rects(&items, &s);
        let b = placed(&items, &l)[0].expect("placed");
        assert!((b.x1 - 95.0).abs() < 1e-9 && (b.y1 - 95.0).abs() < 1e-9);
        // Larger than the sheet within its margins: left out.
        let big = vec![item(0.0, 0.0, 95.0, 20.0)];
        assert_eq!(nest_rects(&big, &s).placed(), 0);
    }

    #[test]
    fn bad_input_is_left_out_without_panicking() {
        let items = vec![
            RectItem {
                bounds: Rect::new(f64::NAN, 0.0, 1.0, 1.0),
                area: 1.0,
            },
            RectItem {
                bounds: Rect::new(0.0, 0.0, 10.0, 10.0),
                area: f64::NAN,
            },
        ];
        let l = nest_rects(&items, &sheet(100.0, 100.0, 2.0));
        assert!(l.placements[0].is_none());
        assert!(l.placements[1].is_some());
        assert!((l.parts_area - 100.0).abs() < 1e-9);
        let l = nest_rects(&items, &sheet(f64::INFINITY, 100.0, 2.0));
        assert_eq!(l.placed(), 0);
        assert_eq!(nest_rects(&[], &sheet(100.0, 100.0, 2.0)).sheets, 0);
    }

    #[test]
    fn the_layout_repeats() {
        let items: Vec<RectItem> = (0..12)
            .map(|i| {
                item(
                    0.0,
                    0.0,
                    10.0 + (i * 7 % 23) as f64,
                    5.0 + (i * 5 % 17) as f64,
                )
            })
            .collect();
        let s = sheet(60.0, 60.0, 1.0);
        let a = nest_rects(&items, &s);
        let b = nest_rects(&items, &s);
        assert_eq!(a, b);
        let r: Vec<Rect> = placed(&items, &a).into_iter().flatten().collect();
        // Apart on the same sheet (sheets are 20 mm apart, so all are).
        assert!(apart(&r, 1.0));
    }
}
