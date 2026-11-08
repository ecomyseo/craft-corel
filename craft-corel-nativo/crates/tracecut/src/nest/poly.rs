//! Polygons for shape nesting: rings with holes (even-odd), their
//! overlap test and a conservative raster of them on a grid of cells.

use tracedraw_core::geometry::{Point, Rect, Vec2};

/// A region bounded by closed rings, even-odd: a ring inside an odd
/// number of others is a hole.
#[derive(Debug, Clone, Default)]
pub(crate) struct Region {
    pub rings: Vec<Vec<Point>>,
    pub bbox: Rect,
}

impl Region {
    pub(crate) fn new(rings: Vec<Vec<Point>>) -> Region {
        let rings: Vec<Vec<Point>> = rings
            .into_iter()
            .filter(|r| r.len() >= 3 && r.iter().all(|p| p.x.is_finite() && p.y.is_finite()))
            .collect();
        let bbox = bounds(rings.iter().flatten());
        Region { rings, bbox }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rings.is_empty()
    }

    /// Net area (outer rings minus holes).
    pub(crate) fn area(&self) -> f64 {
        let mut a = 0.0;
        for (i, r) in self.rings.iter().enumerate() {
            let depth = self
                .rings
                .iter()
                .enumerate()
                .filter(|(j, o)| *j != i && inside_ring(o, r[0]))
                .count();
            let ra = ring_area(r).abs();
            if depth % 2 == 0 {
                a += ra;
            } else {
                a -= ra;
            }
        }
        a.max(0.0)
    }

    /// The rings that are not holes (holes filled in).
    pub(crate) fn outer_only(&self) -> Region {
        let rings = self
            .rings
            .iter()
            .enumerate()
            .filter(|(i, r)| {
                let depth = self
                    .rings
                    .iter()
                    .enumerate()
                    .filter(|(j, o)| j != i && inside_ring(o, r[0]))
                    .count();
                depth % 2 == 0
            })
            .map(|(_, r)| r.clone())
            .collect();
        Region::new(rings)
    }

    /// The region moved by `d`.
    pub(crate) fn moved(&self, d: Vec2) -> Region {
        Region {
            rings: self
                .rings
                .iter()
                .map(|r| r.iter().map(|p| *p + d).collect())
                .collect(),
            bbox: self.bbox + d,
        }
    }

    /// Whether `p` is inside (even-odd over all rings).
    pub(crate) fn contains(&self, p: Point) -> bool {
        self.rings.iter().filter(|r| inside_ring(r, p)).count() % 2 == 1
    }
}

/// The box around `points` (zero at the origin for none).
pub(crate) fn bounds<'a>(points: impl Iterator<Item = &'a Point>) -> Rect {
    let mut r: Option<Rect> = None;
    for p in points {
        r = Some(match r {
            None => Rect::from_points(*p, *p),
            Some(r) => r.union_pt(*p),
        });
    }
    r.unwrap_or(Rect::ZERO)
}

/// Signed area of a ring (counter-clockwise positive).
pub(crate) fn ring_area(r: &[Point]) -> f64 {
    let n = r.len();
    let mut a = 0.0;
    for i in 0..n {
        let p = r[i];
        let q = r[(i + 1) % n];
        a += p.x * q.y - q.x * p.y;
    }
    a / 2.0
}

/// Point in ring by crossings (points on the boundary may go either way).
pub(crate) fn inside_ring(r: &[Point], p: Point) -> bool {
    let n = r.len();
    let mut inside = false;
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let (a, b) = (r[i], r[j % n]);
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

fn cross(o: Point, a: Point, b: Point) -> f64 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

fn on_box(a: Point, b: Point, p: Point) -> bool {
    p.x >= a.x.min(b.x) && p.x <= a.x.max(b.x) && p.y >= a.y.min(b.y) && p.y <= a.y.max(b.y)
}

/// Whether segments `ab` and `cd` meet (touching counts).
pub(crate) fn segments_meet(a: Point, b: Point, c: Point, d: Point) -> bool {
    let d1 = cross(c, d, a);
    let d2 = cross(c, d, b);
    let d3 = cross(a, b, c);
    let d4 = cross(a, b, d);
    if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
    {
        return true;
    }
    (d1 == 0.0 && on_box(c, d, a))
        || (d2 == 0.0 && on_box(c, d, b))
        || (d3 == 0.0 && on_box(a, b, c))
        || (d4 == 0.0 && on_box(a, b, d))
}

fn seg_box(a: Point, b: Point) -> Rect {
    Rect::from_points(a, b)
}

fn boxes_meet(a: Rect, b: Rect) -> bool {
    a.x0 <= b.x1 && b.x0 <= a.x1 && a.y0 <= b.y1 && b.y0 <= a.y1
}

/// The edges of `r` whose boxes meet `window`.
fn edges_in(r: &Region, window: Rect) -> Vec<(Point, Point)> {
    let mut out = Vec::new();
    for ring in &r.rings {
        let n = ring.len();
        for i in 0..n {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            if boxes_meet(seg_box(a, b), window) {
                out.push((a, b));
            }
        }
    }
    out
}

/// Whether two regions share any area or touch. Two regions overlap when
/// their edges meet, or a ring of one lies inside the other's area.
pub(crate) fn overlap(a: &Region, b: &Region) -> bool {
    if a.is_empty() || b.is_empty() || !boxes_meet(a.bbox, b.bbox) {
        return false;
    }
    let window = a.bbox.intersect(b.bbox);
    let ea = edges_in(a, window);
    let eb = edges_in(b, window);
    for (p, q) in &ea {
        let bp = seg_box(*p, *q);
        for (r, s) in &eb {
            if boxes_meet(bp, seg_box(*r, *s)) && segments_meet(*p, *q, *r, *s) {
                return true;
            }
        }
    }
    // No edges meet: each ring is wholly inside or outside the other.
    a.rings.iter().any(|r| b.contains(r[0])) || b.rings.iter().any(|r| a.contains(r[0]))
}

/// A grid of cells `cell` mm wide from (0, 0), each set or free, rows
/// as bit sets. Rows grow on demand.
#[derive(Debug, Clone)]
pub(crate) struct Grid {
    pub cell: f64,
    pub cols: usize,
    words: usize,
    rows: Vec<Vec<u64>>,
    /// Rows the grid may hold (the sheet's), `usize::MAX` for a roll.
    pub max_rows: usize,
}

const EPS: f64 = 1e-7;

impl Grid {
    pub(crate) fn new(cell: f64, cols: usize, max_rows: usize) -> Grid {
        let cols = cols.max(1);
        Grid {
            cell,
            cols,
            words: cols.div_ceil(64),
            rows: Vec::new(),
            max_rows,
        }
    }

    /// Rows in use (above them every cell is free).
    pub(crate) fn used_rows(&self) -> usize {
        self.rows.len()
    }

    fn ensure(&mut self, rows: usize) {
        let rows = rows.min(self.max_rows);
        while self.rows.len() < rows {
            self.rows.push(vec![0; self.words]);
        }
    }

    fn set_span(&mut self, row: usize, c0: usize, c1: usize) {
        if row >= self.max_rows || c0 > c1 || c0 >= self.cols {
            return;
        }
        self.ensure(row + 1);
        let c1 = c1.min(self.cols - 1);
        if let Some(r) = self.rows.get_mut(row) {
            for c in c0..=c1 {
                if let Some(w) = r.get_mut(c / 64) {
                    *w |= 1u64 << (c % 64);
                }
            }
        }
    }

    /// The last set cell of `row` in `c0..c1`.
    pub(crate) fn last_set(&self, row: usize, c0: usize, c1: usize) -> Option<usize> {
        let r = self.rows.get(row)?;
        let c1 = c1.min(self.cols);
        if c0 >= c1 {
            return None;
        }
        let mut w = (c1 - 1) / 64;
        loop {
            let mut bits = *r.get(w)?;
            let lo = w * 64;
            if c1 < lo + 64 {
                bits &= (1u64 << (c1 - lo)) - 1;
            }
            if c0 > lo {
                bits &= !((1u64 << (c0 - lo)) - 1);
            }
            if bits != 0 {
                return Some(lo + 63 - bits.leading_zeros() as usize);
            }
            if w == 0 || lo <= c0 {
                return None;
            }
            w -= 1;
        }
    }

    /// Mark every cell `region` touches (a cell is set when any of the
    /// region's area or boundary lies in it, so the raster always covers
    /// the region).
    pub(crate) fn paint(&mut self, region: &Region) {
        if region.is_empty() {
            return;
        }
        let c = self.cell;
        let cell_of = |v: f64| -> Option<usize> {
            if !v.is_finite() {
                return None;
            }
            let k = (v / c).floor();
            if k < 0.0 {
                Some(0)
            } else if k > 1e9 {
                None
            } else {
                Some(k as usize)
            }
        };
        // Edges: each row band the edge crosses, the cells of its span.
        for ring in &region.rings {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                let (ylo, yhi) = (a.y.min(b.y), a.y.max(b.y));
                if yhi < -EPS {
                    continue;
                }
                let (Some(r0), Some(r1)) = (cell_of(ylo - EPS), cell_of(yhi + EPS)) else {
                    continue;
                };
                for row in r0..=r1.min(self.max_rows.saturating_sub(1)) {
                    let y0 = row as f64 * c - EPS;
                    let y1 = (row + 1) as f64 * c + EPS;
                    let (x0, x1) = if (b.y - a.y).abs() < 1e-12 {
                        (a.x.min(b.x), a.x.max(b.x))
                    } else {
                        let t0 = ((y0 - a.y) / (b.y - a.y)).clamp(0.0, 1.0);
                        let t1 = ((y1 - a.y) / (b.y - a.y)).clamp(0.0, 1.0);
                        let xa = a.x + (b.x - a.x) * t0;
                        let xb = a.x + (b.x - a.x) * t1;
                        (xa.min(xb), xa.max(xb))
                    };
                    if x1 < -EPS {
                        continue;
                    }
                    if let (Some(c0), Some(c1)) = (cell_of(x0 - EPS), cell_of(x1 + EPS)) {
                        self.set_span(row, c0, c1);
                    }
                }
            }
        }
        // Inside: cells whose centre is inside (even-odd scanline).
        let b = region.bbox;
        let (Some(r0), Some(r1)) = (cell_of(b.y0), cell_of(b.y1)) else {
            return;
        };
        let mut xs: Vec<f64> = Vec::new();
        for row in r0..=r1.min(self.max_rows.saturating_sub(1)) {
            let y = (row as f64 + 0.5) * c;
            xs.clear();
            for ring in &region.rings {
                let n = ring.len();
                for i in 0..n {
                    let (a, b) = (ring[i], ring[(i + 1) % n]);
                    if (a.y > y) != (b.y > y) {
                        xs.push(a.x + (y - a.y) / (b.y - a.y) * (b.x - a.x));
                    }
                }
            }
            xs.sort_by(|a, b| a.total_cmp(b));
            for pair in xs.chunks_exact(2) {
                // Cells with centres in [pair[0], pair[1]].
                let first = ((pair[0] / c) - 0.5).ceil();
                let last = ((pair[1] / c) - 0.5).floor();
                if last < 0.0 || last < first || !first.is_finite() || !last.is_finite() {
                    continue;
                }
                let first = first.max(0.0);
                if first > 1e9 {
                    continue;
                }
                self.set_span(row, first as usize, last.min(1e9) as usize);
            }
        }
    }

    /// The runs of set cells of each row: `(start, end)` exclusive.
    pub(crate) fn runs(&self) -> Vec<Vec<(usize, usize)>> {
        self.rows
            .iter()
            .map(|r| {
                let mut out = Vec::new();
                let mut start: Option<usize> = None;
                for c in 0..self.cols {
                    let on = r.get(c / 64).is_some_and(|w| w & (1u64 << (c % 64)) != 0);
                    match (on, start) {
                        (true, None) => start = Some(c),
                        (false, Some(s)) => {
                            out.push((s, c));
                            start = None;
                        }
                        _ => {}
                    }
                }
                if let Some(s) = start {
                    out.push((s, self.cols));
                }
                out
            })
            .collect()
    }

    /// Whether the shape given by its `runs` fits with its cell (0, 0) at
    /// `(kx, ky)`; when not, the next `kx` worth trying on that row.
    pub(crate) fn conflict(
        &self,
        runs: &[Vec<(usize, usize)>],
        kx: usize,
        ky: usize,
    ) -> Option<usize> {
        for (r, row_runs) in runs.iter().enumerate() {
            let row = ky + r;
            if row >= self.rows.len() {
                break;
            }
            for &(a, b) in row_runs {
                if let Some(q) = self.last_set(row, kx + a, kx + b) {
                    return Some(q + 1 - a);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f64, y: f64, s: f64) -> Vec<Point> {
        vec![
            Point::new(x, y),
            Point::new(x + s, y),
            Point::new(x + s, y + s),
            Point::new(x, y + s),
        ]
    }

    #[test]
    fn a_ring_with_a_hole_has_its_net_area() {
        let r = Region::new(vec![square(0.0, 0.0, 10.0), square(2.0, 2.0, 6.0)]);
        assert!((r.area() - 64.0).abs() < 1e-9);
        assert!(r.contains(Point::new(1.0, 1.0)));
        assert!(!r.contains(Point::new(5.0, 5.0)));
        assert!((r.outer_only().area() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn overlap_sees_crossings_containment_and_holes() {
        let ring = Region::new(vec![square(0.0, 0.0, 10.0), square(2.0, 2.0, 6.0)]);
        let small = Region::new(vec![square(3.0, 3.0, 2.0)]);
        assert!(!overlap(&ring, &small), "inside the hole");
        assert!(overlap(&ring.outer_only(), &small), "hole filled");
        let crossing = Region::new(vec![square(8.0, 8.0, 5.0)]);
        assert!(overlap(&ring, &crossing));
        let apart = Region::new(vec![square(20.0, 0.0, 5.0)]);
        assert!(!overlap(&ring, &apart));
        let touching = Region::new(vec![square(10.0, 0.0, 5.0)]);
        assert!(overlap(&ring, &touching), "touching counts");
        let big = Region::new(vec![square(-5.0, -5.0, 30.0)]);
        assert!(overlap(&ring, &big), "wholly inside");
        assert!(!overlap(&Region::default(), &big));
    }

    #[test]
    fn rings_with_bad_points_are_dropped() {
        let r = Region::new(vec![
            vec![
                Point::new(f64::NAN, 0.0),
                Point::new(1.0, 0.0),
                Point::new(0.0, 1.0),
            ],
            vec![Point::new(0.0, 0.0), Point::new(1.0, 0.0)],
        ]);
        assert!(r.is_empty());
        assert_eq!(r.area(), 0.0);
    }

    #[test]
    fn the_raster_covers_the_shape_and_keeps_holes_free() {
        let mut g = Grid::new(1.0, 20, usize::MAX);
        let ring = Region::new(vec![square(0.5, 0.5, 9.0), square(3.0, 3.0, 4.0)]);
        g.paint(&ring);
        let runs = g.runs();
        // Row 0 is crossed by the bottom edge: cells 0..=9.
        assert_eq!(runs[0], vec![(0, 10)]);
        // Row 4 (y 4..5) lies in the hole between x 3 and 7: cells 4 and 5 free.
        assert_eq!(runs[4], vec![(0, 4), (6, 10)]);
        assert_eq!(g.used_rows(), 10);
        assert_eq!(g.last_set(4, 0, 6), Some(3));
        assert_eq!(g.last_set(4, 4, 6), None);
        assert_eq!(g.last_set(99, 0, 6), None);
    }

    #[test]
    fn conflicts_jump_past_the_blocking_cell() {
        let mut g = Grid::new(1.0, 100, 10);
        g.paint(&Region::new(vec![square(0.2, 0.2, 9.6)]));
        let mut p = Grid::new(1.0, 3, usize::MAX);
        p.paint(&Region::new(vec![square(0.2, 0.2, 2.6)]));
        let runs = p.runs();
        // The first blocked run's last cell is 2: try from 3 next.
        assert_eq!(g.conflict(&runs, 0, 0), Some(3));
        assert_eq!(g.conflict(&runs, 7, 0), Some(10));
        assert_eq!(g.conflict(&runs, 10, 0), None);
        // Above the painted rows everything is free.
        assert_eq!(g.conflict(&runs, 0, 10), None);
        // Bits across word boundaries.
        let mut w = Grid::new(1.0, 200, 1);
        w.set_span(0, 60, 130);
        assert_eq!(w.last_set(0, 0, 200), Some(130));
        assert_eq!(w.last_set(0, 0, 64), Some(63));
        assert_eq!(w.last_set(0, 131, 200), None);
    }
}
