//! Nesting: placing objects on sheets and rolls with the least waste.
//!
//! Pure geometry over page-space outlines in millimetres, y up. The
//! application turns the results into commands
//! (TraceCut's Nesting tools).
//!
//! - [`rects`]: rectangle nesting by bounding boxes (MaxRects).
//! - [`shapes`]: nesting by true outlines, holes allowed, with an
//!   improvement loop.
//! - [`fill`]: Fill Sheet, a grid of copies filling a sheet.
//!
//! Every layout is worked out in a *normalised frame*: `u` across the
//! sheet (or roll) from the origin corner, `v` along it, the usable area
//! starting at (0, 0). `Frame` maps it back to the page. Sheets are
//! laid side by side to the right of the page's origin, [`SHEET_GAP`]
//! apart; a roll starts at the page's origin.

pub mod fill;
pub(crate) mod poly;
pub mod rects;
pub mod shapes;

use tracedraw_core::geometry::{Affine, Point, Rect, Vec2};

/// The gap between sheets laid side by side, in millimetres.
pub const SHEET_GAP: f64 = 20.0;

/// The corner the layout starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Corner {
    #[default]
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl Corner {
    pub const ALL: [Corner; 4] = [
        Corner::BottomLeft,
        Corner::BottomRight,
        Corner::TopLeft,
        Corner::TopRight,
    ];

    fn right(self) -> bool {
        matches!(self, Corner::BottomRight | Corner::TopRight)
    }

    fn top(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::TopRight)
    }
}

/// Which way a roll's length runs on the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum RollDir {
    /// The length runs to the right, the roll width up the page.
    #[default]
    Right,
    /// The length runs up the page, the roll width to the right.
    Up,
}

/// The material: sheets of `width` x `height`, or a roll of `width`
/// with no end.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SheetSpec {
    /// Sheet width (along x), or the roll width.
    pub width: f64,
    /// Sheet height (along y); not used for a roll.
    pub height: f64,
    pub roll: bool,
    pub roll_dir: RollDir,
    /// Free border kept along the sheet's edges (mm).
    pub margin: f64,
    /// Distance kept between objects (mm).
    pub spacing: f64,
    pub corner: Corner,
}

impl Default for SheetSpec {
    fn default() -> Self {
        SheetSpec {
            width: 1220.0,
            height: 2440.0,
            roll: false,
            roll_dir: RollDir::Right,
            margin: 0.0,
            spacing: 2.0,
            corner: Corner::BottomLeft,
        }
    }
}

impl SheetSpec {
    /// The spec with every value made usable, or `None` when the sheet
    /// has no size.
    pub(crate) fn sane(&self) -> Option<SheetSpec> {
        let pos = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
        let s = SheetSpec {
            width: pos(self.width),
            height: pos(self.height),
            margin: pos(self.margin),
            spacing: pos(self.spacing),
            ..*self
        };
        let ok = s.width > 1e-6 && (s.roll || s.height > 1e-6);
        ok.then_some(s)
    }

    /// The size across (`u`) and along (`v`) the material; along is
    /// infinite for a roll.
    pub(crate) fn across_along(&self) -> (f64, f64) {
        if self.roll {
            (self.width, f64::INFINITY)
        } else {
            (self.width, self.height)
        }
    }

    /// The page-space rectangle of sheet `i`; for a roll, the stretch of
    /// `length` from its start.
    pub fn sheet_rect(&self, i: usize, length: f64) -> Rect {
        let length = if length.is_finite() {
            length.max(0.0)
        } else {
            0.0
        };
        if self.roll {
            return match self.roll_dir {
                RollDir::Right => Rect::new(0.0, 0.0, length, self.width),
                RollDir::Up => Rect::new(0.0, 0.0, self.width, length),
            };
        }
        let x = i as f64 * (self.width + SHEET_GAP);
        Rect::new(x, 0.0, x + self.width, self.height)
    }
}

/// The map from the normalised frame of a sheet to the page.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Frame {
    /// Axis map: page = origin + linear * n.
    linear: Affine,
    spec: SheetSpec,
}

impl Frame {
    pub(crate) fn new(spec: &SheetSpec) -> Frame {
        let sx = if spec.corner.right() { -1.0 } else { 1.0 };
        let sy = if spec.corner.top() { -1.0 } else { 1.0 };
        // Columns: where u and v point on the page.
        let linear = if spec.roll {
            match spec.roll_dir {
                // u across the width (up or down), v along x.
                RollDir::Right => Affine::new([0.0, sy, 1.0, 0.0, 0.0, 0.0]),
                RollDir::Up => Affine::new([sx, 0.0, 0.0, 1.0, 0.0, 0.0]),
            }
        } else {
            Affine::new([sx, 0.0, 0.0, sy, 0.0, 0.0])
        };
        Frame {
            linear,
            spec: *spec,
        }
    }

    /// The page point of sheet `i`'s origin corner.
    fn origin(&self, i: usize) -> Point {
        let s = &self.spec;
        if s.roll {
            let w = s.width;
            return match s.roll_dir {
                RollDir::Right => Point::new(0.0, if s.corner.top() { w } else { 0.0 }),
                RollDir::Up => Point::new(if s.corner.right() { w } else { 0.0 }, 0.0),
            };
        }
        let r = s.sheet_rect(i, 0.0);
        Point::new(
            if s.corner.right() { r.x1 } else { r.x0 },
            if s.corner.top() { r.y1 } else { r.y0 },
        )
    }

    /// Page to normalised directions (no translation).
    pub(crate) fn to_norm(self) -> Affine {
        // The axis maps are their own inverses' transposes.
        let c = self.linear.as_coeffs();
        Affine::new([c[0], c[2], c[1], c[3], 0.0, 0.0])
    }

    /// Page to sheet `i`'s frame: across and along from its origin corner
    /// (no inset).
    pub(crate) fn to_sheet(self, i: usize) -> Affine {
        self.to_norm() * Affine::translate(-self.origin(i).to_vec2())
    }

    /// The page transform that takes an object whose normalised form,
    /// after `local` (page to the part's local frame), lands at `pos`
    /// on sheet `i`, `inset` in from the corner.
    pub(crate) fn place(&self, i: usize, inset: f64, pos: Vec2, local: Affine) -> Affine {
        let o = self.origin(i);
        Affine::translate(o.to_vec2())
            * self.linear
            * Affine::translate(pos + Vec2::new(inset, inset))
            * local
    }
}

/// Where one object goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The sheet (0 for a roll).
    pub sheet: usize,
    /// The page-space transform to apply to the object.
    pub transform: Affine,
    /// The turn it was given, in degrees.
    pub angle: f64,
}

/// A nesting result.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    /// One entry per object given, `None` when it does not fit.
    pub placements: Vec<Option<Placement>>,
    /// Sheets used (1 for a roll with anything on it).
    pub sheets: usize,
    /// Roll: the length used from the roll's start (the farthest object
    /// plus the margin). Sheets: the same on the last sheet.
    pub length: f64,
    /// Material used: sheets x sheet area, or roll width x length (mm2).
    pub used_area: f64,
    /// Net area of the objects placed (mm2).
    pub parts_area: f64,
    /// The improvement loop ran out of time before its last round.
    pub stopped_by_time: bool,
    /// Rounds of the improvement loop run.
    pub rounds: usize,
}

impl Layout {
    /// Objects placed.
    pub fn placed(&self) -> usize {
        self.placements.iter().filter(|p| p.is_some()).count()
    }

    /// Material used by the objects, 0 to 1.
    pub fn utilisation(&self) -> f64 {
        if self.used_area > 1e-9 {
            (self.parts_area / self.used_area).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Fill in sheets, length and used area from per-object extents:
    /// `(sheet, far edge along v in the normalised frame)`.
    pub(crate) fn finish(&mut self, spec: &SheetSpec, extents: &[(usize, f64)]) {
        let sheets = extents.iter().map(|e| e.0 + 1).max().unwrap_or(0);
        let last = sheets.saturating_sub(1);
        let far = extents
            .iter()
            .filter(|e| e.0 == last)
            .map(|e| e.1)
            .fold(0.0_f64, f64::max);
        self.sheets = sheets;
        self.length = if sheets == 0 { 0.0 } else { far + spec.margin };
        self.used_area = if spec.roll {
            spec.width * self.length
        } else {
            sheets as f64 * spec.width * spec.height
        };
    }
}

/// A small deterministic random generator (SplitMix64).
#[derive(Debug, Clone)]
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x2545_f491_4f6c_dd1d)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A number below `n` (0 when `n` is 0).
    pub(crate) fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }
}

/// `r` with its corners in order, when every coordinate is finite.
pub(crate) fn finite_rect(r: Rect) -> Option<Rect> {
    let ok = [r.x0, r.y0, r.x1, r.y1].iter().all(|v| v.is_finite());
    ok.then_some(r.abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_put_the_origin_corner_first() {
        let spec = SheetSpec {
            width: 100.0,
            height: 50.0,
            corner: Corner::TopRight,
            ..Default::default()
        };
        let f = Frame::new(&spec);
        // A point 10 across and 5 along from the corner of sheet 1.
        let t = f.place(1, 0.0, Vec2::new(10.0, 5.0), Affine::IDENTITY);
        let p = t * Point::ZERO;
        let x1 = 100.0 + SHEET_GAP + 100.0;
        assert!((p.x - (x1 - 10.0)).abs() < 1e-9 && (p.y - 45.0).abs() < 1e-9);
        // to_norm undoes the axis map.
        let back = f.to_norm() * f.linear;
        assert!((back.as_coeffs()[0] - 1.0).abs() < 1e-12);
        assert!((back.as_coeffs()[3] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_roll_to_the_right_runs_along_x() {
        let spec = SheetSpec {
            width: 600.0,
            roll: true,
            ..Default::default()
        };
        let f = Frame::new(&spec);
        let p = f.place(0, 0.0, Vec2::new(10.0, 300.0), Affine::IDENTITY) * Point::ZERO;
        assert!((p.x - 300.0).abs() < 1e-9 && (p.y - 10.0).abs() < 1e-9);
        let r = spec.sheet_rect(0, 1000.0);
        assert_eq!(r, Rect::new(0.0, 0.0, 1000.0, 600.0));
        assert_eq!(spec.sheet_rect(0, f64::NAN).width(), 0.0);
    }

    #[test]
    fn specs_without_a_size_are_refused() {
        let mut s = SheetSpec {
            width: f64::NAN,
            ..Default::default()
        };
        assert!(s.sane().is_none());
        s.width = 10.0;
        s.height = 0.0;
        assert!(s.sane().is_none());
        s.roll = true;
        s.margin = f64::INFINITY;
        assert_eq!(s.sane().map(|s| s.margin), Some(0.0));
    }

    #[test]
    fn the_generator_repeats_for_a_seed() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        let mut c = Rng::new(8);
        let xa: Vec<u64> = (0..5).map(|_| a.next_u64()).collect();
        let xb: Vec<u64> = (0..5).map(|_| b.next_u64()).collect();
        let xc: Vec<u64> = (0..5).map(|_| c.next_u64()).collect();
        assert_eq!(xa, xb);
        assert_ne!(xa, xc);
        assert_eq!(a.below(0), 0);
        assert!(a.below(3) < 3);
    }
}
