//! Table model operations: per-edge cell borders, cell margins, separated
//! borders, inserting several rows or columns, splitting cells into N,
//! moving rows and columns, resizing by their dividing lines and hit
//! testing those lines. The app turns each edit into one `SetShapeKind`
//! command, so all of it undoes as one step.

use crate::document::{Table, TableCell, TextAlign};
use crate::geometry::{BezPath, Point, Rect};
use crate::style::Stroke;
use serde::{Deserialize, Serialize};

/// Smallest row height or column width a drag or a split leaves, mm.
pub const MIN_CELL: f64 = 1.0;

/// One side of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl CellSide {
    pub const ALL: [CellSide; 4] = [
        CellSide::Top,
        CellSide::Right,
        CellSide::Bottom,
        CellSide::Left,
    ];

    pub fn opposite(self) -> CellSide {
        match self {
            CellSide::Top => CellSide::Bottom,
            CellSide::Bottom => CellSide::Top,
            CellSide::Left => CellSide::Right,
            CellSide::Right => CellSide::Left,
        }
    }

    fn horizontal(self) -> bool {
        matches!(self, CellSide::Top | CellSide::Bottom)
    }
}

/// The border on one side of a cell.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum EdgeBorder {
    /// The table's border (`Table::border`).
    #[default]
    Table,
    /// No border on this side.
    None,
    /// This side's own outline.
    Stroke(Box<Stroke>),
}

impl EdgeBorder {
    fn is_table(&self) -> bool {
        matches!(self, EdgeBorder::Table)
    }
}

/// The borders of a cell's four sides.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CellBorders {
    pub top: EdgeBorder,
    pub right: EdgeBorder,
    pub bottom: EdgeBorder,
    pub left: EdgeBorder,
}

impl CellBorders {
    pub fn is_default(&self) -> bool {
        *self == CellBorders::default()
    }

    pub fn side(&self, s: CellSide) -> &EdgeBorder {
        match s {
            CellSide::Top => &self.top,
            CellSide::Right => &self.right,
            CellSide::Bottom => &self.bottom,
            CellSide::Left => &self.left,
        }
    }

    pub fn side_mut(&mut self, s: CellSide) -> &mut EdgeBorder {
        match s {
            CellSide::Top => &mut self.top,
            CellSide::Right => &mut self.right,
            CellSide::Bottom => &mut self.bottom,
            CellSide::Left => &mut self.left,
        }
    }
}

/// Space between a cell's edge and its text, per side, mm.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CellMargins {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl CellMargins {
    pub fn uniform(v: f64) -> Self {
        CellMargins {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }
}

/// Which borders of the selected cells a border change applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderSet {
    #[default]
    All,
    Outside,
    Inside,
    Top,
    Bottom,
    Left,
    Right,
    InnerHorizontal,
    InnerVertical,
}

impl BorderSet {
    pub const ALL: [BorderSet; 9] = [
        BorderSet::All,
        BorderSet::Outside,
        BorderSet::Inside,
        BorderSet::Top,
        BorderSet::Bottom,
        BorderSet::Left,
        BorderSet::Right,
        BorderSet::InnerHorizontal,
        BorderSet::InnerVertical,
    ];

    /// i18n key of the choice's name.
    pub fn key(self) -> &'static str {
        match self {
            BorderSet::All => "table.borders_all",
            BorderSet::Outside => "table.borders_outside",
            BorderSet::Inside => "table.borders_inside",
            BorderSet::Top => "table.borders_top",
            BorderSet::Bottom => "table.borders_bottom",
            BorderSet::Left => "table.borders_left",
            BorderSet::Right => "table.borders_right",
            BorderSet::InnerHorizontal => "table.borders_inner_h",
            BorderSet::InnerVertical => "table.borders_inner_v",
        }
    }

    /// Whether the side `s` of a selected cell is in the set; `outer`
    /// tells whether that side lies on the selection's outline.
    fn takes(self, s: CellSide, outer: bool) -> bool {
        match self {
            BorderSet::All => true,
            BorderSet::Outside => outer,
            BorderSet::Inside => !outer,
            BorderSet::Top => outer && s == CellSide::Top,
            BorderSet::Bottom => outer && s == CellSide::Bottom,
            BorderSet::Left => outer && s == CellSide::Left,
            BorderSet::Right => outer && s == CellSide::Right,
            BorderSet::InnerHorizontal => !outer && s.horizontal(),
            BorderSet::InnerVertical => !outer && !s.horizontal(),
        }
    }
}

/// What lies under the pointer near a table, for the Table tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableHit {
    /// Just outside the top-left corner: every cell.
    Corner,
    /// Just left of row `r`: that row.
    RowHeader(u32),
    /// Just above column `c`: that column.
    ColHeader(u32),
    /// The vertical line at the right of column `k - 1` (1..=cols).
    ColLine(u32),
    /// The horizontal line under row `k - 1` (1..=rows).
    RowLine(u32),
}

pub fn blank_cell(row: u32, col: u32) -> TableCell {
    TableCell {
        row,
        col,
        row_span: 1,
        col_span: 1,
        text: Vec::new(),
        fill: None,
        align: TextAlign::Left,
        borders: CellBorders::default(),
    }
}

impl Table {
    /// Index of the cell anchored at (row, col).
    pub fn cell_index(&self, row: u32, col: u32) -> Option<usize> {
        self.cells.iter().position(|c| c.row == row && c.col == col)
    }

    /// The anchor (top-left grid position) of the cell covering a grid
    /// position.
    pub fn anchor_of(&self, row: u32, col: u32) -> Option<(u32, u32)> {
        self.cells
            .iter()
            .find(|c| {
                row >= c.row && row < c.row + c.row_span && col >= c.col && col < c.col + c.col_span
            })
            .map(|c| (c.row, c.col))
    }

    /// The cells of the rectangular block spanned by two cells, grown until
    /// no merged cell sticks out of it; anchors in row-major order.
    pub fn block_cells(&self, a: (u32, u32), b: (u32, u32)) -> Vec<(u32, u32)> {
        let (mut r0, mut r1) = (a.0.min(b.0), a.0.max(b.0));
        let (mut c0, mut c1) = (a.1.min(b.1), a.1.max(b.1));
        // Take in the far ends of the corner cells.
        for (r, c) in [a, b] {
            if let Some(cell) = self.cell_index(r, c).and_then(|i| self.cells.get(i)) {
                r1 = r1.max(cell.row + cell.row_span - 1);
                c1 = c1.max(cell.col + cell.col_span - 1);
            }
        }
        loop {
            let mut grown = false;
            for c in &self.cells {
                let (cr1, cc1) = (c.row + c.row_span - 1, c.col + c.col_span - 1);
                let meets = c.row <= r1 && cr1 >= r0 && c.col <= c1 && cc1 >= c0;
                if !meets {
                    continue;
                }
                if c.row < r0 || cr1 > r1 || c.col < c0 || cc1 > c1 {
                    r0 = r0.min(c.row);
                    r1 = r1.max(cr1);
                    c0 = c0.min(c.col);
                    c1 = c1.max(cc1);
                    grown = true;
                }
            }
            if !grown {
                break;
            }
        }
        let mut out: Vec<(u32, u32)> = self
            .cells
            .iter()
            .filter(|c| c.row >= r0 && c.row <= r1 && c.col >= c0 && c.col <= c1)
            .map(|c| (c.row, c.col))
            .collect();
        out.sort_unstable();
        out
    }

    /// Every cell, row-major.
    pub fn all_cells(&self) -> Vec<(u32, u32)> {
        let mut out: Vec<(u32, u32)> = self.cells.iter().map(|c| (c.row, c.col)).collect();
        out.sort_unstable();
        out
    }

    /// First and last grid row and column the cells cover.
    pub fn span_of(&self, cells: &[(u32, u32)]) -> Option<(u32, u32, u32, u32)> {
        let mut out: Option<(u32, u32, u32, u32)> = None;
        for &(r, c) in cells {
            let Some(cell) = self.cell_index(r, c).and_then(|i| self.cells.get(i)) else {
                continue;
            };
            let (r1, c1) = (cell.row + cell.row_span - 1, cell.col + cell.col_span - 1);
            out = Some(match out {
                None => (cell.row, r1, cell.col, c1),
                Some((a, b, x, y)) => (a.min(cell.row), b.max(r1), x.min(cell.col), y.max(c1)),
            });
        }
        out
    }

    /// The margins between a cell's edges and its text.
    pub fn cell_margins(&self) -> CellMargins {
        self.margins
            .unwrap_or_else(|| CellMargins::uniform(self.padding))
    }

    /// The rectangle a cell is drawn in: its grid rectangle, made smaller
    /// by half the spacing on each side when the borders are separated.
    pub fn cell_box(&self, cell: &TableCell) -> Rect {
        let r = self.cell_rect(cell);
        match self.spacing {
            Some([h, v]) => {
                let (h, v) = ((h / 2.0).max(0.0), (v / 2.0).max(0.0));
                let x0 = (r.x0 + h).min(r.center().x);
                let x1 = (r.x1 - h).max(r.center().x);
                let y0 = (r.y0 + v).min(r.center().y);
                let y1 = (r.y1 - v).max(r.center().y);
                Rect::new(x0, y0, x1, y1)
            }
            None => r,
        }
    }

    /// Where a cell's text goes: its box less the margins.
    pub fn cell_inner(&self, cell: &TableCell) -> Rect {
        let b = self.cell_box(cell);
        let m = self.cell_margins();
        let x0 = b.x0 + m.left.max(0.0);
        let x1 = (b.x1 - m.right.max(0.0)).max(x0);
        let y1 = b.y1 - m.top.max(0.0);
        let y0 = (b.y0 + m.bottom.max(0.0)).min(y1);
        Rect::new(x0, y0, x1, y1)
    }

    fn resolve(&self, e: &EdgeBorder) -> Option<Stroke> {
        match e {
            EdgeBorder::Table => self.border.clone(),
            EdgeBorder::None => None,
            EdgeBorder::Stroke(s) => Some(s.as_ref().clone()),
        }
    }

    /// The outline of each border, grouped by stroke: per-edge settings
    /// first, the table's border where a side has none of its own. A line
    /// shared by two cells takes the setting of the cell that has one (the
    /// cell below or to the right when both do). With separated borders
    /// every cell draws its own box and the table its outline.
    pub fn border_paths(&self) -> Vec<(Stroke, BezPath)> {
        let mut groups: Vec<(Stroke, BezPath)> = Vec::new();
        let mut add = |s: Option<Stroke>, a: Point, b: Point| {
            let Some(s) = s else {
                return;
            };
            if let Some((_, p)) = groups.iter_mut().find(|(g, _)| *g == s) {
                p.move_to(a);
                p.line_to(b);
            } else {
                let mut p = BezPath::new();
                p.move_to(a);
                p.line_to(b);
                groups.push((s, p));
            }
        };
        if self.spacing.is_some() {
            if let Some(b) = &self.border {
                let r = self.rect;
                let corners = [
                    Point::new(r.x0, r.y1),
                    Point::new(r.x1, r.y1),
                    Point::new(r.x1, r.y0),
                    Point::new(r.x0, r.y0),
                ];
                for i in 0..4 {
                    add(Some(b.clone()), corners[i], corners[(i + 1) % 4]);
                }
            }
            for cell in &self.cells {
                if self.covered(cell.row, cell.col) {
                    continue;
                }
                let r = self.cell_box(cell);
                let (tl, tr) = (Point::new(r.x0, r.y1), Point::new(r.x1, r.y1));
                let (bl, br) = (Point::new(r.x0, r.y0), Point::new(r.x1, r.y0));
                add(self.resolve(&cell.borders.top), tl, tr);
                add(self.resolve(&cell.borders.right), tr, br);
                add(self.resolve(&cell.borders.bottom), br, bl);
                add(self.resolve(&cell.borders.left), bl, tl);
            }
            return groups;
        }
        let cell_at = |r: u32, c: u32| {
            self.anchor_of(r, c)
                .and_then(|(ar, ac)| self.cell_index(ar, ac))
                .and_then(|i| self.cells.get(i))
        };
        let pick = |before: Option<&TableCell>, after: Option<&TableCell>, s: CellSide| {
            // `after` is below or to the right of the line; its side facing
            // the line is `s`, the other cell's is the opposite one.
            let a = after.map(|c| c.borders.side(s));
            let b = before.map(|c| c.borders.side(s.opposite()));
            match (a, b) {
                (Some(e), _) if !e.is_table() => self.resolve(e),
                (_, Some(e)) if !e.is_table() => self.resolve(e),
                _ => self.border.clone(),
            }
        };
        let (rows, cols) = (self.rows(), self.cols());
        // Horizontal lines: line k is the top of row k.
        for k in 0..=rows {
            let y = self.row_top(k);
            let mut run: Option<(Option<Stroke>, f64)> = None;
            for j in 0..cols {
                let above = if k > 0 { cell_at(k - 1, j) } else { None };
                let below = if k < rows { cell_at(k, j) } else { None };
                let inside = match (above, below) {
                    (Some(a), Some(b)) => a.row == b.row && a.col == b.col,
                    _ => false,
                };
                let s = if inside {
                    None
                } else {
                    pick(above, below, CellSide::Top)
                };
                let x = self.col_x(j);
                match &run {
                    Some((rs, _)) if *rs == s => {}
                    Some((rs, x0)) => {
                        add(rs.clone(), Point::new(*x0, y), Point::new(x, y));
                        run = Some((s, x));
                    }
                    None => run = Some((s, x)),
                }
            }
            if let Some((rs, x0)) = run {
                add(rs, Point::new(x0, y), Point::new(self.col_x(cols), y));
            }
        }
        // Vertical lines: line k is the left of column k.
        for k in 0..=cols {
            let x = self.col_x(k);
            let mut run: Option<(Option<Stroke>, f64)> = None;
            for i in 0..rows {
                let left = if k > 0 { cell_at(i, k - 1) } else { None };
                let right = if k < cols { cell_at(i, k) } else { None };
                let inside = match (left, right) {
                    (Some(a), Some(b)) => a.row == b.row && a.col == b.col,
                    _ => false,
                };
                let s = if inside {
                    None
                } else {
                    pick(left, right, CellSide::Left)
                };
                let y = self.row_top(i);
                match &run {
                    Some((rs, _)) if *rs == s => {}
                    Some((rs, y0)) => {
                        add(rs.clone(), Point::new(x, *y0), Point::new(x, y));
                        run = Some((s, y));
                    }
                    None => run = Some((s, y)),
                }
            }
            if let Some((rs, y0)) = run {
                add(rs, Point::new(x, y0), Point::new(x, self.row_top(rows)));
            }
        }
        groups
    }

    /// The cells across side `s` of a cell (anchors, without repeats).
    fn neighbours(&self, cell: &TableCell, s: CellSide) -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        let mut push = |r: u32, c: u32| {
            if let Some(a) = self.anchor_of(r, c) {
                if !out.contains(&a) {
                    out.push(a);
                }
            }
        };
        match s {
            CellSide::Top if cell.row > 0 => {
                (cell.col..cell.col + cell.col_span).for_each(|c| push(cell.row - 1, c))
            }
            CellSide::Bottom => {
                (cell.col..cell.col + cell.col_span).for_each(|c| push(cell.row + cell.row_span, c))
            }
            CellSide::Left if cell.col > 0 => {
                (cell.row..cell.row + cell.row_span).for_each(|r| push(r, cell.col - 1))
            }
            CellSide::Right => {
                (cell.row..cell.row + cell.row_span).for_each(|r| push(r, cell.col + cell.col_span))
            }
            _ => {}
        }
        out
    }

    /// Set the borders `set` of the selected cells to `border`. A side on
    /// the selection's outline also changes the facing side of the cell
    /// across it when that one had a setting of its own, so the shared
    /// line shows the new border.
    pub fn apply_borders(&mut self, selected: &[(u32, u32)], set: BorderSet, border: EdgeBorder) {
        let mut changes: Vec<((u32, u32), CellSide)> = Vec::new();
        for &(r, c) in selected {
            let Some(cell) = self.cell_index(r, c).and_then(|i| self.cells.get(i)) else {
                continue;
            };
            for s in CellSide::ALL {
                let across = self.neighbours(cell, s);
                let outer = across.is_empty() || across.iter().any(|a| !selected.contains(a));
                if !set.takes(s, outer) {
                    continue;
                }
                changes.push(((r, c), s));
                for a in across {
                    if !selected.contains(&a) {
                        changes.push((a, s.opposite()));
                    }
                }
            }
        }
        for ((r, c), s) in changes {
            let Some(i) = self.cell_index(r, c) else {
                continue;
            };
            let Some(cell) = self.cells.get_mut(i) else {
                continue;
            };
            let own = selected.contains(&(r, c));
            let side = cell.borders.side_mut(s);
            if own || !side.is_table() {
                *side = border.clone();
            }
        }
    }

    /// Insert `count` rows before row `at` (0..=rows), each as tall as row
    /// `like`. Merged cells crossing the insertion line grow.
    pub fn insert_rows(&mut self, at: u32, count: u32, like: u32) {
        let at = at.min(self.rows());
        let h = self.row_heights.get(like as usize).copied().unwrap_or(10.0);
        for _ in 0..count {
            self.row_heights.insert(at as usize, h);
        }
        for c in self.cells.iter_mut() {
            if c.row >= at {
                c.row += count;
            } else if c.row + c.row_span > at {
                c.row_span += count;
            }
        }
        for r in at..at + count {
            for col in 0..self.cols() {
                if !self.covered(r, col) && self.cell_index(r, col).is_none() {
                    self.cells.push(blank_cell(r, col));
                }
            }
        }
    }

    /// Insert `count` columns before column `at` (0..=cols), each as wide
    /// as column `like`.
    pub fn insert_cols(&mut self, at: u32, count: u32, like: u32) {
        let at = at.min(self.cols());
        let w = self.col_widths.get(like as usize).copied().unwrap_or(20.0);
        for _ in 0..count {
            self.col_widths.insert(at as usize, w);
        }
        for c in self.cells.iter_mut() {
            if c.col >= at {
                c.col += count;
            } else if c.col + c.col_span > at {
                c.col_span += count;
            }
        }
        for row in 0..self.rows() {
            for col in at..at + count {
                if !self.covered(row, col) && self.cell_index(row, col).is_none() {
                    self.cells.push(blank_cell(row, col));
                }
            }
        }
    }

    /// Remove `count` rows from row `at` on, at least one row staying.
    /// Cells wholly inside the band go; a merged cell reaching into it
    /// keeps the rows it has outside.
    pub fn remove_rows(&mut self, at: u32, count: u32) {
        let end = at.saturating_add(count).min(self.rows());
        if at >= end || end - at >= self.rows() {
            return;
        }
        let count = end - at;
        self.row_heights.drain(at as usize..end as usize);
        self.cells
            .retain(|c| !(c.row >= at && c.row + c.row_span <= end));
        for c in self.cells.iter_mut() {
            let (r0, r1) = (c.row, c.row + c.row_span);
            let before = r1.min(at).saturating_sub(r0);
            let after = r1.saturating_sub(r0.max(end));
            c.row = if r0 < at {
                r0
            } else if r0 >= end {
                r0 - count
            } else {
                at
            };
            c.row_span = (before + after).max(1);
        }
    }

    /// [`Table::remove_rows`] for columns.
    pub fn remove_cols(&mut self, at: u32, count: u32) {
        let end = at.saturating_add(count).min(self.cols());
        if at >= end || end - at >= self.cols() {
            return;
        }
        let count = end - at;
        self.col_widths.drain(at as usize..end as usize);
        self.cells
            .retain(|c| !(c.col >= at && c.col + c.col_span <= end));
        for c in self.cells.iter_mut() {
            let (c0, c1) = (c.col, c.col + c.col_span);
            let before = c1.min(at).saturating_sub(c0);
            let after = c1.saturating_sub(c0.max(end));
            c.col = if c0 < at {
                c0
            } else if c0 >= end {
                c0 - count
            } else {
                at
            };
            c.col_span = (before + after).max(1);
        }
    }

    /// Split the cells into `n` rows each. The cells must lie in one band
    /// (the same first row and row span); others in the band stay whole
    /// (they span the new rows). A merged band of `k` rows splits without
    /// new grid rows when `n` divides `k`. Returns false (nothing changed)
    /// when the split is not possible.
    pub fn split_rows(&mut self, cells: &[(u32, u32)], n: u32) -> bool {
        self.split(cells, n, true)
    }

    /// [`Table::split_rows`] for columns.
    pub fn split_cols(&mut self, cells: &[(u32, u32)], n: u32) -> bool {
        self.split(cells, n, false)
    }

    fn split(&mut self, cells: &[(u32, u32)], n: u32, rows: bool) -> bool {
        if !(2..=100).contains(&n) {
            return false;
        }
        let picked: Vec<usize> = cells
            .iter()
            .filter_map(|&(r, c)| self.cell_index(r, c))
            .collect();
        let Some(&first) = picked.first() else {
            return false;
        };
        let band = |c: &TableCell| {
            if rows {
                (c.row, c.row_span)
            } else {
                (c.col, c.col_span)
            }
        };
        let Some(b) = self.cells.get(first).map(band) else {
            return false;
        };
        let picked: Vec<usize> = picked
            .into_iter()
            .filter(|&i| self.cells.get(i).map(band) == Some(b))
            .collect();
        let (start, span) = b;
        let sizes = if rows {
            &self.row_heights
        } else {
            &self.col_widths
        };
        let total: f64 = sizes.iter().skip(start as usize).take(span as usize).sum();
        // Pieces thinner than a tenth of a millimetre are refused.
        // (NaN sizes are refused too.)
        if total / (n as f64) < 0.1 || total.is_nan() {
            return false;
        }
        let originals: Vec<TableCell> = picked
            .iter()
            .filter_map(|&i| self.cells.get(i).cloned())
            .collect();
        let piece = |orig: &TableCell, k: u32, at: u32, len: u32| {
            let mut c = orig.clone();
            if k > 0 {
                c.text = Vec::new();
            }
            if rows {
                c.row = at;
                c.row_span = len;
                if k > 0 {
                    c.borders.top = EdgeBorder::Table;
                }
                if k + 1 < n {
                    c.borders.bottom = EdgeBorder::Table;
                }
            } else {
                c.col = at;
                c.col_span = len;
                if k > 0 {
                    c.borders.left = EdgeBorder::Table;
                }
                if k + 1 < n {
                    c.borders.right = EdgeBorder::Table;
                }
            }
            c
        };
        if span % n == 0 {
            let per = span / n;
            let mut out = Vec::new();
            for o in &originals {
                for k in 0..n {
                    out.push(piece(o, k, start + k * per, per));
                }
            }
            self.cells
                .retain(|c| !originals.iter().any(|o| o.row == c.row && o.col == c.col));
            self.cells.extend(out);
            return true;
        }
        if span != 1 {
            return false;
        }
        // One grid row (column) becomes n equal ones.
        let s = start as usize;
        let size = total / n as f64;
        let list = if rows {
            &mut self.row_heights
        } else {
            &mut self.col_widths
        };
        if let Some(x) = list.get_mut(s) {
            *x = size;
        }
        for _ in 1..n {
            list.insert(s + 1, size);
        }
        let extra = n - 1;
        for c in self.cells.iter_mut() {
            let is_picked = originals.iter().any(|o| o.row == c.row && o.col == c.col);
            let (pos, len) = if rows {
                (&mut c.row, &mut c.row_span)
            } else {
                (&mut c.col, &mut c.col_span)
            };
            if *pos > start {
                *pos += extra;
            } else if *pos + *len > start && !is_picked {
                *len += extra;
            }
        }
        let mut out = Vec::new();
        for o in &originals {
            for k in 0..n {
                out.push(piece(o, k, start + k, 1));
            }
        }
        self.cells
            .retain(|c| !originals.iter().any(|o| o.row == c.row && o.col == c.col));
        self.cells.extend(out);
        true
    }

    /// Move rows `first..first + count` so they come before row `to` of
    /// the current order (0..=rows). Returns false, changing nothing, when
    /// the move would cut a merged cell or goes nowhere.
    pub fn move_rows(&mut self, first: u32, count: u32, to: u32) -> bool {
        self.move_lines(first, count, to, true)
    }

    /// [`Table::move_rows`] for columns.
    pub fn move_cols(&mut self, first: u32, count: u32, to: u32) -> bool {
        self.move_lines(first, count, to, false)
    }

    fn move_lines(&mut self, first: u32, count: u32, to: u32, rows: bool) -> bool {
        let len = if rows { self.rows() } else { self.cols() };
        let end = first.saturating_add(count);
        if count == 0 || end > len || to > len || (to >= first && to <= end) {
            return false;
        }
        // New order of the old indices.
        let moving: Vec<u32> = (first..end).collect();
        let rest: Vec<u32> = (0..len).filter(|i| !moving.contains(i)).collect();
        let insert_at = rest.iter().filter(|&&i| i < to).count();
        let mut order = rest.clone();
        for (k, m) in moving.iter().enumerate() {
            order.insert(insert_at + k, *m);
        }
        let mut new_of = vec![0u32; len as usize];
        for (new, &old) in order.iter().enumerate() {
            if let Some(slot) = new_of.get_mut(old as usize) {
                *slot = new as u32;
            }
        }
        // Merged cells must stay in one piece and in order.
        for c in &self.cells {
            let (pos, span) = if rows {
                (c.row, c.row_span)
            } else {
                (c.col, c.col_span)
            };
            let Some(&base) = new_of.get(pos as usize) else {
                return false;
            };
            for k in 1..span {
                if new_of.get((pos + k) as usize) != Some(&(base + k)) {
                    return false;
                }
            }
        }
        let sizes = if rows {
            &mut self.row_heights
        } else {
            &mut self.col_widths
        };
        let old = sizes.clone();
        *sizes = order
            .iter()
            .filter_map(|&i| old.get(i as usize).copied())
            .collect();
        for c in self.cells.iter_mut() {
            let pos = if rows { &mut c.row } else { &mut c.col };
            if let Some(&n) = new_of.get(*pos as usize) {
                *pos = n;
            }
        }
        true
    }

    /// Drag the vertical line `k` (1..=cols) by `dx` mm: an inner line
    /// trades width between its two columns (the table keeps its width),
    /// the right edge resizes the last column. No column gets narrower
    /// than [`MIN_CELL`].
    pub fn drag_col_line(&mut self, k: u32, dx: f64) {
        Self::drag_line(&mut self.col_widths, k, dx);
    }

    /// Drag the horizontal line `k` (1..=rows) down by `dy` mm (the row
    /// above grows); the bottom edge resizes the last row.
    pub fn drag_row_line(&mut self, k: u32, dy: f64) {
        Self::drag_line(&mut self.row_heights, k, dy);
    }

    fn drag_line(sizes: &mut [f64], k: u32, d: f64) {
        if !d.is_finite() || k == 0 {
            return;
        }
        let i = k as usize - 1;
        let n = sizes.len();
        if i >= n {
            return;
        }
        if i + 1 == n {
            if let Some(a) = sizes.get_mut(i) {
                *a = (*a + d).max(MIN_CELL);
            }
            return;
        }
        let (a, b) = (sizes[i], sizes[i + 1]);
        let lo = MIN_CELL - a;
        let hi = b - MIN_CELL;
        let d = if lo > hi { 0.0 } else { d.clamp(lo, hi) };
        sizes[i] = a + d;
        sizes[i + 1] = b - d;
    }

    /// What is at a local point near the table: `tol` is the reach in mm.
    /// The header zones lie just outside the left and top edges.
    pub fn hit(&self, p: Point, tol: f64) -> Option<TableHit> {
        let r = self.rect;
        let reach = tol * 3.0;
        let left = p.x >= r.x0 - reach && p.x <= r.x0 + tol * 0.5;
        let top = p.y <= r.y1 + reach && p.y >= r.y1 - tol * 0.5;
        if left && top {
            return Some(TableHit::Corner);
        }
        let in_x = p.x >= r.x0 && p.x <= r.x1;
        let in_y = p.y >= r.y0 && p.y <= r.y1;
        if top && in_x {
            return (0..self.cols())
                .find(|&c| p.x < self.col_x(c + 1))
                .or(self.cols().checked_sub(1))
                .map(TableHit::ColHeader);
        }
        if left && in_y {
            return (0..self.rows())
                .find(|&k| p.y > self.row_top(k + 1))
                .or(self.rows().checked_sub(1))
                .map(TableHit::RowHeader);
        }
        if p.y >= r.y0 - tol && p.y <= r.y1 {
            if let Some(k) = (1..=self.cols()).find(|&k| (p.x - self.col_x(k)).abs() <= tol) {
                return Some(TableHit::ColLine(k));
            }
        }
        if p.x >= r.x0 && p.x <= r.x1 + tol {
            if let Some(k) = (1..=self.rows()).find(|&k| (p.y - self.row_top(k)).abs() <= tol) {
                return Some(TableHit::RowLine(k));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    fn grid(rows: u32, cols: u32) -> Table {
        Table::new(
            Rect::new(0.0, 0.0, cols as f64 * 10.0, rows as f64 * 10.0),
            rows,
            cols,
            Some(Stroke::hairline(Color::BLACK)),
        )
    }

    fn red() -> Stroke {
        let mut s = Stroke::hairline(Color::rgb8(255, 0, 0));
        s.width = 0.5;
        s
    }

    /// Rows and columns removed from the end (the property bar's counts,
    /// #1680) and from the middle: the cells in them go, a merged cell
    /// reaching into them keeps the rest, and the last row or column
    /// cannot be removed.
    #[test]
    fn remove_rows_and_columns_keeps_the_rest_of_the_table() {
        let mut t = grid(4, 3);
        // A cell spanning rows 1..=2 of column 0.
        t.cells.retain(|c| !(c.col == 0 && c.row == 2));
        if let Some(i) = t.cell_index(1, 0) {
            t.cells[i].row_span = 2;
        }
        t.remove_rows(2, 2);
        assert_eq!(t.rows(), 2);
        assert_eq!(t.cells.len(), 6);
        assert!(t.cells.iter().all(|c| c.row + c.row_span <= 2));
        let merged = t.cells[t.cell_index(1, 0).unwrap()].clone();
        assert_eq!(merged.row_span, 1);
        // Columns from the middle: the merged cell spanning columns 0..=2
        // of row 0 loses the one removed and stays at column 0.
        let mut t = grid(2, 4);
        t.cells
            .retain(|c| !(c.row == 0 && (1..=2).contains(&c.col)));
        if let Some(i) = t.cell_index(0, 0) {
            t.cells[i].col_span = 3;
        }
        t.remove_cols(1, 1);
        assert_eq!(t.cols(), 3);
        // Row 0: the merged cell and the last one; row 1: three cells.
        assert_eq!(t.cells.len(), 5);
        let merged = t.cells[t.cell_index(0, 0).unwrap()].clone();
        assert_eq!((merged.col, merged.col_span), (0, 2));
        assert!(t.cell_index(1, 2).is_some());
        // Never below one row or column; nothing past the end.
        let mut t = grid(2, 2);
        t.remove_rows(0, 2);
        t.remove_cols(5, 1);
        assert_eq!((t.rows(), t.cols()), (2, 2));
    }

    fn total(p: &BezPath) -> f64 {
        use kurbo::ParamCurveArclen as _;
        p.segments().map(|s| s.arclen(1e-6)).sum()
    }

    #[test]
    fn default_borders_draw_every_grid_line_once() {
        let t = grid(2, 3);
        let g = t.border_paths();
        assert_eq!(g.len(), 1);
        // 3 horizontal lines of 30 mm and 4 vertical lines of 20 mm.
        assert!((total(&g[0].1) - (90.0 + 80.0)).abs() < 1e-6);
    }

    #[test]
    fn outside_borders_of_a_selection_take_their_own_stroke() {
        let mut t = grid(3, 3);
        let sel = t.block_cells((0, 0), (1, 1));
        assert_eq!(sel, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
        t.apply_borders(
            &sel,
            BorderSet::Outside,
            EdgeBorder::Stroke(Box::new(red())),
        );
        let g = t.border_paths();
        let red_len = g
            .iter()
            .find(|(s, _)| *s == red())
            .map(|(_, p)| total(p))
            .unwrap_or(0.0);
        // The outline of a 20 x 20 block.
        assert!((red_len - 80.0).abs() < 1e-6, "{red_len}");
        // Inner lines of the block keep the table's border.
        let c = &t.cells[t.cell_index(0, 0).unwrap()];
        assert_eq!(c.borders.right, EdgeBorder::Table);
        assert_eq!(c.borders.top, EdgeBorder::Stroke(Box::new(red())));
    }

    #[test]
    fn each_border_choice_picks_its_sides() {
        let lens = |set: BorderSet| {
            let mut t = grid(3, 3);
            let sel = t.block_cells((0, 0), (1, 1));
            t.apply_borders(&sel, set, EdgeBorder::Stroke(Box::new(red())));
            t.border_paths()
                .iter()
                .find(|(s, _)| *s == red())
                .map(|(_, p)| total(p))
                .unwrap_or(0.0)
        };
        assert!((lens(BorderSet::All) - 120.0).abs() < 1e-6);
        assert!((lens(BorderSet::Inside) - 40.0).abs() < 1e-6);
        assert!((lens(BorderSet::Top) - 20.0).abs() < 1e-6);
        assert!((lens(BorderSet::Bottom) - 20.0).abs() < 1e-6);
        assert!((lens(BorderSet::Left) - 20.0).abs() < 1e-6);
        assert!((lens(BorderSet::Right) - 20.0).abs() < 1e-6);
        assert!((lens(BorderSet::InnerHorizontal) - 20.0).abs() < 1e-6);
        assert!((lens(BorderSet::InnerVertical) - 20.0).abs() < 1e-6);
    }

    #[test]
    fn no_border_removes_the_line_shared_with_a_neighbour() {
        let mut t = grid(1, 2);
        t.apply_borders(
            &[(0, 0)],
            BorderSet::Right,
            EdgeBorder::Stroke(Box::new(red())),
        );
        // The neighbour had its own setting later: the new one replaces it.
        t.apply_borders(&[(0, 1)], BorderSet::Left, EdgeBorder::None);
        let g = t.border_paths();
        assert!(g.iter().all(|(s, _)| *s != red()));
        let black: f64 = g.iter().map(|(_, p)| total(p)).sum();
        // Outline 20 + 20 + 10 + 10, no middle line.
        assert!((black - 60.0).abs() < 1e-6, "{black}");
    }

    #[test]
    fn separated_borders_shrink_cells_and_draw_the_outline() {
        let mut t = grid(1, 2);
        t.spacing = Some([2.0, 4.0]);
        let c = t.cells[0].clone();
        let b = t.cell_box(&c);
        assert_eq!(b, Rect::new(1.0, 2.0, 9.0, 8.0));
        let g = t.border_paths();
        let len: f64 = g.iter().map(|(_, p)| total(p)).sum();
        // Table outline 60 + two boxes of 8 x 6.
        assert!((len - (60.0 + 2.0 * 28.0)).abs() < 1e-6, "{len}");
    }

    #[test]
    fn margins_per_side_place_the_text() {
        let mut t = grid(1, 1);
        t.margins = Some(CellMargins {
            left: 1.0,
            right: 2.0,
            top: 3.0,
            bottom: 4.0,
        });
        let c = t.cells[0].clone();
        assert_eq!(t.cell_inner(&c), Rect::new(1.0, 4.0, 8.0, 7.0));
        t.margins = None;
        assert_eq!(t.cell_inner(&c), Rect::new(1.0, 1.0, 9.0, 9.0));
    }

    #[test]
    fn insert_several_rows_and_columns() {
        let mut t = grid(2, 2);
        t.row_heights[1] = 7.0;
        t.insert_rows(2, 3, 1);
        assert_eq!(t.rows(), 5);
        assert_eq!(t.cells.len(), 10);
        assert!(t.row_heights[2..].iter().all(|h| (*h - 7.0).abs() < 1e-9));
        t.insert_cols(0, 2, 0);
        assert_eq!(t.cols(), 4);
        assert_eq!(t.cells.len(), 20);
        // The old first cell moved two columns right.
        assert!(t.cell_index(0, 2).is_some());
    }

    #[test]
    fn split_a_cell_into_three_rows_and_columns() {
        let mut t = grid(2, 2);
        assert!(t.split_rows(&[(0, 0)], 3));
        assert_eq!(t.rows(), 4);
        // The other cell of the row spans the new rows.
        let other = &t.cells[t.cell_index(0, 1).unwrap()];
        assert_eq!(other.row_span, 3);
        assert!(t.row_heights[..3]
            .iter()
            .all(|h| (*h - 10.0 / 3.0).abs() < 1e-9));
        assert!(t.cell_index(2, 0).is_some());
        // A merged band of three rows splits into three without new rows.
        assert!(t.split_rows(&[(0, 1)], 3));
        assert_eq!(t.rows(), 4);
        assert_eq!(t.cell_index(1, 1).map(|i| t.cells[i].row_span), Some(1));
        assert!(t.split_cols(&[(3, 1)], 4));
        assert_eq!(t.cols(), 5);
        assert!(!t.split_cols(&[(3, 1)], 1));
    }

    #[test]
    fn rows_and_columns_move_unless_a_merged_cell_would_break() {
        let mut t = grid(3, 1);
        t.row_heights = vec![1.0, 2.0, 3.0];
        t.cells[0].text = vec![crate::TextSpan::new("a", "Sans", 10.0)];
        assert!(t.move_rows(0, 1, 3));
        assert_eq!(t.row_heights, vec![2.0, 3.0, 1.0]);
        let a = t.cells.iter().find(|c| !c.text.is_empty()).unwrap();
        assert_eq!(a.row, 2);
        assert!(!t.move_rows(1, 1, 2), "a move onto itself does nothing");
        let mut m = grid(2, 3);
        m.cells.retain(|c| !(c.row == 0 && c.col == 1));
        m.cells[0].col_span = 2;
        assert!(!m.move_cols(2, 1, 1), "would cut the merged cell");
        assert!(m.move_cols(2, 1, 0));
    }

    #[test]
    fn dragging_lines_resizes_with_a_minimum() {
        let mut t = grid(2, 3);
        t.drag_col_line(1, 4.0);
        assert_eq!(t.col_widths, vec![14.0, 6.0, 10.0]);
        t.drag_col_line(1, 100.0);
        assert_eq!(t.col_widths, vec![19.0, 1.0, 10.0]);
        t.drag_col_line(3, -2.5);
        assert_eq!(t.col_widths[2], 7.5);
        t.drag_row_line(2, 5.0);
        assert_eq!(t.row_heights, vec![10.0, 15.0]);
        t.drag_row_line(1, f64::NAN);
        assert_eq!(t.row_heights, vec![10.0, 15.0]);
    }

    #[test]
    fn hit_finds_headers_corner_and_lines() {
        let t = grid(2, 3);
        assert_eq!(t.hit(Point::new(-1.0, 21.0), 1.0), Some(TableHit::Corner));
        assert_eq!(
            t.hit(Point::new(15.0, 21.5), 1.0),
            Some(TableHit::ColHeader(1))
        );
        assert_eq!(
            t.hit(Point::new(-2.0, 5.0), 1.0),
            Some(TableHit::RowHeader(1))
        );
        assert_eq!(
            t.hit(Point::new(10.4, 5.0), 1.0),
            Some(TableHit::ColLine(1))
        );
        assert_eq!(
            t.hit(Point::new(25.0, 10.3), 1.0),
            Some(TableHit::RowLine(1))
        );
        assert_eq!(t.hit(Point::new(15.0, 15.0), 1.0), None);
    }

    #[test]
    fn a_block_grows_over_merged_cells() {
        let mut t = grid(3, 3);
        // Merge (1,1)-(2,2).
        t.cells
            .retain(|c| !(c.row >= 1 && c.col >= 1 && !(c.row == 1 && c.col == 1)));
        let i = t.cell_index(1, 1).unwrap();
        t.cells[i].row_span = 2;
        t.cells[i].col_span = 2;
        let b = t.block_cells((0, 0), (1, 1));
        assert_eq!(b, vec![(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (2, 0)]);
        assert_eq!(t.span_of(&b), Some((0, 2, 0, 2)));
    }
}
