//! Repeat Copies' Fill page (#1255): the selection tiled in a grid over
//! the page (or an area of it) inside its margins, with a gap between
//! the copies each way, every other row shifted if asked, turned a
//! quarter turn when that fits more, and trim marks at every cut line
//! outside the grid. The grid is centred in the room inside the margins.
//!
//! This module only works out where the copies go; the app makes them
//! (`App::fill_page_apply`, one undo step) and outlines them while the
//! panel is worked in (`step_repeat`).

use tracedraw_core::geometry::{Point, Rect, Size};

/// The most copies one Fill page makes; a layout of more is not applied.
pub const MAX_COPIES: usize = 10_000;

/// Which way the copies stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Turn {
    /// As the selection stands.
    #[default]
    AsIs,
    /// Turned a quarter turn (90 degrees anticlockwise).
    Turned,
    /// Whichever of the two fits more copies (as it stands on a tie).
    Best,
}

impl Turn {
    /// The choice a panel index stands for (0 as is).
    pub fn from_index(i: u8) -> Self {
        match i {
            1 => Turn::Turned,
            2 => Turn::Best,
            _ => Turn::AsIs,
        }
    }
}

/// The settings of a Fill page, in mm.
#[derive(Debug, Clone, PartialEq)]
pub struct FillSettings {
    /// Room kept clear inside the area: left, right, top, bottom.
    pub margins: [f64; 4],
    /// The gap between copies side by side, and between rows.
    pub gap: (f64, f64),
    /// How far every second row (the second, fourth, ... from the top)
    /// moves right; 0 for none.
    pub row_shift: f64,
    pub turn: Turn,
}

/// Where the copies of one Fill page go.
#[derive(Debug, Clone, PartialEq)]
pub struct FillGrid {
    /// Copies across in a row, and in a shifted row.
    pub cols: usize,
    pub shifted_cols: usize,
    pub rows: usize,
    /// The copies stand turned a quarter turn.
    pub turned: bool,
    /// Each copy's box, row by row from the top, left to right.
    pub cells: Vec<Rect>,
}

#[cfg(test)]
impl FillGrid {
    /// The number of copies placed.
    pub fn count(&self) -> usize {
        self.cells.len()
    }
}

/// A length kept finite and not negative.
fn len(v: f64) -> f64 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

/// How many pieces of `size` with `gap` between them fit in `room`.
fn fits(room: f64, size: f64, gap: f64) -> usize {
    if size <= 1e-9 || room + 1e-9 < size {
        return 0;
    }
    let n = ((room - size) / (size + gap) + 1e-9).floor() + 1.0;
    // Kept to a count the grid can hold, whatever was typed.
    n.min(MAX_COPIES as f64 * 10.0) as usize
}

/// The grid of copies of an object of `item` (its box, as it stands) in
/// `area`, standing as it does or turned (`turned`).
pub fn grid(item: Size, area: Rect, s: &FillSettings, turned: bool) -> FillGrid {
    let (w, h) = if turned {
        (item.height, item.width)
    } else {
        (item.width, item.height)
    };
    let [ml, mr, mt, mb] = s.margins.map(len);
    let (gx, gy) = (len(s.gap.0), len(s.gap.1));
    let shift = len(s.row_shift);
    let room = Rect::new(area.x0 + ml, area.y0 + mb, area.x1 - mr, area.y1 - mt);
    let (rw, rh) = (room.width(), room.height());
    let empty = |turned| FillGrid {
        cols: 0,
        shifted_cols: 0,
        rows: 0,
        turned,
        cells: Vec::new(),
    };
    if !(w.is_finite() && h.is_finite()) || rw <= 0.0 || rh <= 0.0 {
        return empty(turned);
    }
    let cols = fits(rw, w, gx);
    let shifted_cols = if shift > 0.0 {
        fits(rw - shift, w, gx)
    } else {
        cols
    };
    let rows = fits(rh, h, gy);
    if cols == 0 || rows == 0 {
        return empty(turned);
    }
    let row_cols = |r: usize| if r % 2 == 1 { shifted_cols } else { cols };
    let total: usize = (0..rows.min(MAX_COPIES + 1)).map(row_cols).sum();
    // The block's size, to centre it: its widest row and its rows.
    let row_w = |n: usize| n as f64 * w + n.saturating_sub(1) as f64 * gx;
    let block_w = if rows > 1 && shifted_cols > 0 {
        row_w(cols).max(shift + row_w(shifted_cols))
    } else {
        row_w(cols)
    };
    let block_h = rows as f64 * h + (rows - 1) as f64 * gy;
    let x0 = room.x0 + (rw - block_w) / 2.0;
    let top = room.y1 - (rh - block_h) / 2.0;
    let mut cells = Vec::new();
    // Past the limit the count is reported but no boxes are made.
    if total <= MAX_COPIES {
        cells.reserve(total);
        for r in 0..rows {
            let y1 = top - r as f64 * (h + gy);
            let left = x0 + if r % 2 == 1 { shift } else { 0.0 };
            for c in 0..row_cols(r) {
                let cx = left + c as f64 * (w + gx);
                cells.push(Rect::new(cx, y1 - h, cx + w, y1));
            }
        }
    }
    FillGrid {
        cols,
        shifted_cols,
        rows,
        turned,
        cells,
    }
}

/// The number of copies a grid holds, also past [`MAX_COPIES`].
pub fn count(g: &FillGrid) -> usize {
    (0..g.rows)
        .map(|r| if r % 2 == 1 { g.shifted_cols } else { g.cols })
        .sum()
}

/// The grids as the object stands and turned, and the one the settings
/// choose.
pub fn layout(item: Size, area: Rect, s: &FillSettings) -> (FillGrid, FillGrid, FillGrid) {
    let as_is = grid(item, area, s, false);
    let turned = grid(item, area, s, true);
    let chosen = match s.turn {
        Turn::AsIs => as_is.clone(),
        Turn::Turned => turned.clone(),
        Turn::Best => {
            if count(&turned) > count(&as_is) {
                turned.clone()
            } else {
                as_is.clone()
            }
        }
    };
    (as_is, turned, chosen)
}

/// Trim marks for copies in `cells`: at every cut line (each copy's
/// edges), a line of `length` starting `offset` outside the grid on both
/// sides, so no mark falls on a copy. Each mark is a pair of ends.
pub fn trim_marks(cells: &[Rect], length: f64, offset: f64) -> Vec<(Point, Point)> {
    let mut out = Vec::new();
    let Some(all) = cells.iter().copied().reduce(|a, b| a.union(b)) else {
        return out;
    };
    let (length, offset) = (len(length), len(offset));
    if length <= 0.0 {
        return out;
    }
    let mut xs: Vec<f64> = cells.iter().flat_map(|r| [r.x0, r.x1]).collect();
    let mut ys: Vec<f64> = cells.iter().flat_map(|r| [r.y0, r.y1]).collect();
    for v in [&mut xs, &mut ys] {
        v.sort_by(|a, b| a.total_cmp(b));
        v.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    }
    for x in xs {
        out.push((
            Point::new(x, all.y1 + offset),
            Point::new(x, all.y1 + offset + length),
        ));
        out.push((
            Point::new(x, all.y0 - offset),
            Point::new(x, all.y0 - offset - length),
        ));
    }
    for y in ys {
        out.push((
            Point::new(all.x0 - offset, y),
            Point::new(all.x0 - offset - length, y),
        ));
        out.push((
            Point::new(all.x1 + offset, y),
            Point::new(all.x1 + offset + length, y),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page of the report's video: 12 x 18 inches.
    fn page() -> Rect {
        Rect::new(0.0, 0.0, 304.8, 457.2)
    }

    fn settings(margin: f64, gap: f64, turn: Turn) -> FillSettings {
        FillSettings {
            margins: [margin; 4],
            gap: (gap, gap),
            row_shift: 0.0,
            turn,
        }
    }

    /// The report's video: a label 3 inches wide and a little over 2
    /// high on a 12 x 18 inch page fits 4 x 8 = 32 as it stands and
    /// 5 x 6 = 30 turned; with 3 mm margins 3 x 8 = 24 and 5 x 5 = 25,
    /// and whichever fits more takes the turned 25.
    #[test]
    fn the_counts_match_the_report() {
        let label = Size::new(76.2, 51.0);
        let (a, t, _) = layout(label, page(), &settings(0.0, 0.0, Turn::AsIs));
        assert_eq!((a.cols, a.rows, a.count()), (4, 8, 32));
        assert_eq!((t.cols, t.rows, t.count()), (5, 6, 30));
        let (a, t, best) = layout(label, page(), &settings(3.0, 0.0, Turn::Best));
        assert_eq!((a.cols, a.rows, a.count()), (3, 8, 24));
        assert_eq!((t.cols, t.rows, t.count()), (5, 5, 25));
        assert!(best.turned);
        assert_eq!(best.count(), 25);
        // The turned copies stand 51 wide and 76.2 high.
        let c = best.cells[0];
        assert!((c.width() - 51.0).abs() < 1e-9 && (c.height() - 76.2).abs() < 1e-9);
    }

    /// The copies touch with no gap, keep the gap given each way, sit
    /// inside the margins and are centred in the room there.
    #[test]
    fn the_grid_keeps_the_gaps_and_margins_and_is_centred() {
        let item = Size::new(50.0, 30.0);
        let s = FillSettings {
            margins: [10.0, 20.0, 5.0, 15.0],
            gap: (4.0, 6.0),
            row_shift: 0.0,
            turn: Turn::AsIs,
        };
        let area = Rect::new(0.0, 0.0, 210.0, 297.0);
        let g = grid(item, area, &s, false);
        // Room 180 x 277: (180 + 4) / 54 = 3 across, (277 + 6) / 36 = 7 down.
        assert_eq!((g.cols, g.rows, g.count()), (3, 7, 21));
        let (a, b) = (g.cells[0], g.cells[1]);
        assert!((b.x0 - a.x1 - 4.0).abs() < 1e-9);
        let below = g.cells[3];
        assert!((a.y0 - below.y1 - 6.0).abs() < 1e-9);
        let all = g.cells.iter().copied().reduce(|a, b| a.union(b)).unwrap();
        assert!(all.x0 >= 10.0 - 1e-9 && all.x1 <= 190.0 + 1e-9, "{all:?}");
        assert!(all.y0 >= 15.0 - 1e-9 && all.y1 <= 292.0 + 1e-9, "{all:?}");
        let room_mid = ((10.0 + 190.0) / 2.0, (15.0 + 292.0) / 2.0);
        assert!((all.center().x - room_mid.0).abs() < 1e-9);
        assert!((all.center().y - room_mid.1).abs() < 1e-9);
        // The first copy is at the top left.
        assert!(g
            .cells
            .iter()
            .all(|c| c.y1 <= a.y1 + 1e-9 && c.x0 >= a.x0 - 1e-9));
    }

    /// Every second row moves right by the shift, holding fewer copies
    /// when the last one would run past the margin.
    #[test]
    fn alternate_rows_shift() {
        let item = Size::new(40.0, 20.0);
        let mut s = settings(0.0, 0.0, Turn::AsIs);
        s.row_shift = 20.0;
        let area = Rect::new(0.0, 0.0, 160.0, 60.0);
        let g = grid(item, area, &s, false);
        assert_eq!((g.cols, g.shifted_cols, g.rows), (4, 3, 3));
        assert_eq!(g.count(), 4 + 3 + 4);
        assert!((g.cells[4].x0 - (g.cells[0].x0 + 20.0)).abs() < 1e-9);
        // Room for the shifted row too: as many in each row.
        let wide = Rect::new(0.0, 0.0, 180.0, 60.0);
        let g = grid(item, wide, &s, false);
        assert_eq!((g.cols, g.shifted_cols), (4, 4));
    }

    /// Nothing fits: no copies; a bad value is no copies, not a panic;
    /// a huge count is reported without making the boxes.
    #[test]
    fn nothing_fits_and_bad_values() {
        let s = settings(0.0, 0.0, Turn::Best);
        let (a, t, c) = layout(Size::new(400.0, 500.0), page(), &s);
        assert_eq!((a.count(), t.count(), c.count()), (0, 0, 0));
        let g = grid(Size::new(f64::NAN, 10.0), page(), &s, false);
        assert_eq!(g.count(), 0);
        let mut bad = s.clone();
        bad.margins = [f64::INFINITY, -5.0, f64::NAN, 0.0];
        bad.gap = (f64::NAN, -1.0);
        let _ = layout(Size::new(10.0, 10.0), page(), &bad);
        let tiny = grid(Size::new(0.5, 0.5), page(), &s, false);
        assert!(count(&tiny) > MAX_COPIES);
        assert!(tiny.cells.is_empty());
    }

    /// Trim marks stand at every cut line on all four sides of the grid,
    /// outside it, as long as asked.
    #[test]
    fn trim_marks_mark_every_cut() {
        let item = Size::new(50.0, 30.0);
        let s = settings(0.0, 0.0, Turn::AsIs);
        let g = grid(item, Rect::new(0.0, 0.0, 150.0, 60.0), &s, false);
        assert_eq!(g.count(), 6);
        let marks = trim_marks(&g.cells, 3.0, 1.0);
        // 4 cut lines across and 3 down, each marked at both ends.
        assert_eq!(marks.len(), (4 + 3) * 2);
        let all = g.cells.iter().copied().reduce(|a, b| a.union(b)).unwrap();
        for (a, b) in &marks {
            assert!(((*a - *b).hypot() - 3.0).abs() < 1e-9);
            let outside = |p: &Point| {
                p.x <= all.x0 - 1.0 + 1e-9
                    || p.x >= all.x1 + 1.0 - 1e-9
                    || p.y <= all.y0 - 1.0 + 1e-9
                    || p.y >= all.y1 + 1.0 - 1e-9
            };
            assert!(outside(a) && outside(b), "{a:?} {b:?}");
        }
        // With a gap each copy's two edges are cut.
        let mut gapped = s.clone();
        gapped.gap = (5.0, 5.0);
        let g = grid(item, Rect::new(0.0, 0.0, 160.0, 70.0), &gapped, false);
        assert_eq!(trim_marks(&g.cells, 3.0, 0.0).len(), (6 + 4) * 2);
        assert!(trim_marks(&g.cells, 0.0, 0.0).is_empty());
    }
}
