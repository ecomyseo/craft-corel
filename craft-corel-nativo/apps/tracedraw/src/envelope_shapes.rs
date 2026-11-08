//! The Envelope docker's preset shapes: outlines in the unit square
//! (y up), each turned into an envelope frame around an object's box. A
//! frame's corners are the outline nodes the box's corners map to.

use tracedraw_core::envelope::Frame;
use tracedraw_core::geometry::{Point, Rect};

/// An outline being built: nodes, the handles of the segment from each
/// node to the next, and the corner nodes.
#[derive(Debug, Clone, Default)]
pub struct Outline {
    pub nodes: Vec<Point>,
    pub handles: Vec<[Point; 2]>,
    pub corners: Vec<usize>,
}

fn p(x: f64, y: f64) -> Point {
    Point::new(x, y)
}

impl Outline {
    fn start(x: f64, y: f64) -> Outline {
        Outline {
            nodes: vec![p(x, y)],
            ..Default::default()
        }
    }

    fn last(&self) -> Point {
        self.nodes.last().copied().unwrap_or_default()
    }

    fn line(mut self, x: f64, y: f64) -> Self {
        let a = self.last();
        let b = p(x, y);
        self.handles
            .push([a.lerp(b, 1.0 / 3.0), a.lerp(b, 2.0 / 3.0)]);
        self.nodes.push(b);
        self
    }

    fn curve(mut self, h1: (f64, f64), h2: (f64, f64), to: (f64, f64)) -> Self {
        self.handles.push([p(h1.0, h1.1), p(h2.0, h2.1)]);
        self.nodes.push(p(to.0, to.1));
        self
    }

    /// An elliptic arc about `c` with radii `r`, from the current point
    /// (at angle `a0`, degrees) to `a1` in `n` pieces.
    fn arc(mut self, c: (f64, f64), r: (f64, f64), a0: f64, a1: f64, n: usize) -> Self {
        let n = n.max(1);
        let step = (a1 - a0) / n as f64;
        let k = 4.0 / 3.0 * (step.to_radians() / 4.0).tan();
        let at = |a: f64| {
            let t = a.to_radians();
            (c.0 + r.0 * t.cos(), c.1 + r.1 * t.sin())
        };
        let tangent = |a: f64| {
            let t = a.to_radians();
            (-r.0 * t.sin(), r.1 * t.cos())
        };
        for i in 0..n {
            let (s, e) = (a0 + step * i as f64, a0 + step * (i + 1) as f64);
            let (ps, pe) = (at(s), at(e));
            let (ts, te) = (tangent(s), tangent(e));
            self = self.curve(
                (ps.0 + k * ts.0, ps.1 + k * ts.1),
                (pe.0 - k * te.0, pe.1 - k * te.1),
                pe,
            );
        }
        self
    }

    /// The current node is a corner.
    fn corner(mut self) -> Self {
        self.corners.push(self.nodes.len() - 1);
        self
    }

    /// Close the outline: a last segment that ended on the first node
    /// replaces it; otherwise a straight one goes back.
    fn close(mut self) -> Self {
        let first = self.nodes.first().copied().unwrap_or_default();
        if self.nodes.len() > 1 && (self.last() - first).hypot() < 1e-9 {
            self.nodes.pop();
            self.corners.retain(|c| *c < self.nodes.len());
        } else {
            let a = self.last();
            self.handles
                .push([a.lerp(first, 1.0 / 3.0), a.lerp(first, 2.0 / 3.0)]);
        }
        self
    }

    /// The frame around `b`.
    pub fn frame(&self, b: Rect) -> Option<Frame> {
        let map = |q: Point| p(b.x0 + q.x * b.width(), b.y0 + q.y * b.height());
        let nodes: Vec<Point> = self.nodes.iter().map(|q| map(*q)).collect();
        let handles: Vec<[Point; 2]> = self
            .handles
            .iter()
            .map(|[a, c]| [map(*a), map(*c)])
            .collect();
        Frame::from_parts(&nodes, &handles, &self.corners)
    }

    /// The outline as points for a picture, `steps` per segment, y up.
    pub fn points(&self, steps: usize) -> Vec<Point> {
        use tracedraw_core::geometry::{CubicBez, ParamCurve};
        let n = self.nodes.len();
        let mut out = Vec::new();
        for i in 0..n {
            let a = self.nodes[i];
            let b = self.nodes[(i + 1) % n];
            let [h1, h2] = self.handles.get(i).copied().unwrap_or([a, b]);
            let c = CubicBez::new(a, h1, h2, b);
            for k in 0..steps {
                out.push(c.eval(k as f64 / steps as f64));
            }
        }
        out
    }
}

/// How many preset shapes there are.
pub const COUNT: usize = 24;

/// Preset `i`'s outline (0 to [`COUNT`] - 1; others give the square).
pub fn outline(i: usize) -> Outline {
    match i {
        // Square.
        0 => Outline::start(0.0, 0.0)
            .corner()
            .line(1.0, 0.0)
            .corner()
            .line(1.0, 1.0)
            .corner()
            .line(0.0, 1.0)
            .corner()
            .close(),
        // Arched top.
        1 => Outline::start(0.0, 0.0)
            .corner()
            .line(1.0, 0.0)
            .corner()
            .line(1.0, 0.8)
            .corner()
            .curve((1.0, 1.07), (0.0, 1.07), (0.0, 0.8))
            .corner()
            .close(),
        // Circle.
        2 => Outline::start(
            0.5 + 0.5 * (225f64).to_radians().cos(),
            0.5 + 0.5 * (225f64).to_radians().sin(),
        )
        .corner()
        .arc((0.5, 0.5), (0.5, 0.5), 225.0, 315.0, 2)
        .corner()
        .arc((0.5, 0.5), (0.5, 0.5), 315.0, 405.0, 2)
        .corner()
        .arc((0.5, 0.5), (0.5, 0.5), 45.0, 135.0, 2)
        .corner()
        .arc((0.5, 0.5), (0.5, 0.5), 135.0, 225.0, 2)
        .close(),
        // Pill.
        3 => Outline::start(0.25, 0.0)
            .corner()
            .line(0.75, 0.0)
            .corner()
            .arc((0.75, 0.5), (0.25, 0.5), -90.0, 90.0, 2)
            .corner()
            .line(0.25, 1.0)
            .corner()
            .arc((0.25, 0.5), (0.25, 0.5), 90.0, 270.0, 2)
            .close(),
        // Parallelogram.
        4 => polygon(
            &[(0.0, 0.0), (0.75, 0.0), (1.0, 1.0), (0.25, 1.0)],
            [0, 1, 2, 3],
        ),
        // Trapezoid.
        5 => polygon(
            &[(0.0, 0.0), (1.0, 0.0), (0.75, 1.0), (0.25, 1.0)],
            [0, 1, 2, 3],
        ),
        // Hexagon.
        6 => polygon(
            &[
                (0.25, 0.0),
                (0.75, 0.0),
                (1.0, 0.5),
                (0.75, 1.0),
                (0.25, 1.0),
                (0.0, 0.5),
            ],
            [0, 1, 3, 4],
        ),
        // Heptagon-like rounded octagon.
        7 => polygon(
            &[
                (0.35, 0.0),
                (0.65, 0.0),
                (1.0, 0.3),
                (1.0, 0.7),
                (0.65, 1.0),
                (0.35, 1.0),
                (0.0, 0.7),
                (0.0, 0.3),
            ],
            [0, 2, 4, 6],
        ),
        // Octagon.
        8 => polygon(
            &[
                (0.29, 0.0),
                (0.71, 0.0),
                (1.0, 0.29),
                (1.0, 0.71),
                (0.71, 1.0),
                (0.29, 1.0),
                (0.0, 0.71),
                (0.0, 0.29),
            ],
            [0, 2, 4, 6],
        ),
        // Long hexagon.
        9 => polygon(
            &[
                (0.15, 0.0),
                (0.85, 0.0),
                (1.0, 0.5),
                (0.85, 1.0),
                (0.15, 1.0),
                (0.0, 0.5),
            ],
            [0, 1, 3, 4],
        ),
        // Flag.
        10 => Outline::start(0.0, 0.0)
            .corner()
            .curve((0.33, 0.15), (0.66, -0.15), (1.0, 0.0))
            .corner()
            .line(1.0, 1.0)
            .corner()
            .curve((0.66, 0.85), (0.33, 1.15), (0.0, 1.0))
            .corner()
            .close(),
        // Heart.
        11 => Outline::start(0.5, 0.0)
            .corner()
            .curve((0.65, 0.15), (1.0, 0.4), (1.0, 0.65))
            .corner()
            .curve((1.0, 0.88), (0.88, 1.0), (0.75, 1.0))
            .curve((0.6, 1.0), (0.5, 0.92), (0.5, 0.8))
            .corner()
            .curve((0.5, 0.92), (0.4, 1.0), (0.25, 1.0))
            .curve((0.12, 1.0), (0.0, 0.88), (0.0, 0.65))
            .corner()
            .curve((0.0, 0.4), (0.35, 0.15), (0.5, 0.0))
            .close(),
        // Pinched square.
        12 => Outline::start(0.0, 0.0)
            .corner()
            .curve((0.33, 0.2), (0.66, 0.2), (1.0, 0.0))
            .corner()
            .curve((0.8, 0.33), (0.8, 0.66), (1.0, 1.0))
            .corner()
            .curve((0.66, 0.8), (0.33, 0.8), (0.0, 1.0))
            .corner()
            .curve((0.2, 0.66), (0.2, 0.33), (0.0, 0.0))
            .close(),
        // Four-point star.
        13 => Outline::start(0.38, 0.38)
            .corner()
            .curve((0.45, 0.25), (0.48, 0.1), (0.5, 0.0))
            .curve((0.52, 0.1), (0.55, 0.25), (0.62, 0.38))
            .corner()
            .curve((0.75, 0.45), (0.9, 0.48), (1.0, 0.5))
            .curve((0.9, 0.52), (0.75, 0.55), (0.62, 0.62))
            .corner()
            .curve((0.55, 0.75), (0.52, 0.9), (0.5, 1.0))
            .curve((0.48, 0.9), (0.45, 0.75), (0.38, 0.62))
            .corner()
            .curve((0.25, 0.55), (0.1, 0.52), (0.0, 0.5))
            .curve((0.1, 0.48), (0.25, 0.45), (0.38, 0.38))
            .close(),
        // Barrel: top and bottom bowed out.
        14 => Outline::start(0.0, 0.0)
            .corner()
            .curve((0.33, -0.12), (0.66, -0.12), (1.0, 0.0))
            .corner()
            .line(1.0, 1.0)
            .corner()
            .curve((0.66, 1.12), (0.33, 1.12), (0.0, 1.0))
            .corner()
            .close(),
        // Rounded rectangle: the corners are the middles of the arcs.
        15 => {
            let r = 0.2;
            let d = r * (1.0 - std::f64::consts::FRAC_1_SQRT_2);
            Outline::start(d, d)
                .corner()
                .arc((r, r), (r, r), 225.0, 270.0, 1)
                .line(1.0 - r, 0.0)
                .arc((1.0 - r, r), (r, r), -90.0, -45.0, 1)
                .corner()
                .arc((1.0 - r, r), (r, r), -45.0, 0.0, 1)
                .line(1.0, 1.0 - r)
                .arc((1.0 - r, 1.0 - r), (r, r), 0.0, 45.0, 1)
                .corner()
                .arc((1.0 - r, 1.0 - r), (r, r), 45.0, 90.0, 1)
                .line(r, 1.0)
                .arc((r, 1.0 - r), (r, r), 90.0, 135.0, 1)
                .corner()
                .arc((r, 1.0 - r), (r, r), 135.0, 180.0, 1)
                .line(0.0, r)
                .arc((r, r), (r, r), 180.0, 225.0, 1)
                .close()
        }
        // Cushion: bowed out top and bottom, sides in.
        16 => Outline::start(0.0, 0.0)
            .corner()
            .curve((0.33, -0.15), (0.66, -0.15), (1.0, 0.0))
            .corner()
            .curve((0.88, 0.33), (0.88, 0.66), (1.0, 1.0))
            .corner()
            .curve((0.66, 1.15), (0.33, 1.15), (0.0, 1.0))
            .corner()
            .curve((0.12, 0.66), (0.12, 0.33), (0.0, 0.0))
            .close(),
        // Diamond.
        17 => polygon(
            &[(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)],
            [0, 1, 2, 3],
        ),
        // Lens.
        18 => Outline::start(0.0, 0.5)
            .corner()
            .curve((0.2, 0.2), (0.35, 0.0), (0.5, 0.0))
            .curve((0.65, 0.0), (0.8, 0.2), (1.0, 0.5))
            .corner()
            .curve((0.8, 0.8), (0.65, 1.0), (0.5, 1.0))
            .corner()
            .curve((0.35, 1.0), (0.2, 0.8), (0.0, 0.5))
            .close()
            .with_corners([0, 1, 2, 3]),
        // Badge: corners cut in; the corners are the middles of the cuts.
        19 => Outline::start(0.07, 0.07)
            .corner()
            .curve((0.12, 0.07), (0.15, 0.05), (0.15, 0.0))
            .line(0.85, 0.0)
            .curve((0.85, 0.05), (0.88, 0.07), (0.93, 0.07))
            .corner()
            .curve((0.95, 0.1), (0.95, 0.15), (1.0, 0.15))
            .line(1.0, 0.85)
            .curve((0.95, 0.85), (0.95, 0.9), (0.93, 0.93))
            .corner()
            .curve((0.88, 0.93), (0.85, 0.95), (0.85, 1.0))
            .line(0.15, 1.0)
            .curve((0.15, 0.95), (0.12, 0.93), (0.07, 0.93))
            .corner()
            .curve((0.05, 0.9), (0.05, 0.85), (0.0, 0.85))
            .line(0.0, 0.15)
            .curve((0.05, 0.15), (0.05, 0.1), (0.07, 0.07))
            .close(),
        // Wavy rectangle.
        20 => Outline::start(0.0, 0.0)
            .corner()
            .curve((0.17, 0.08), (0.33, 0.08), (0.5, 0.0))
            .curve((0.67, -0.08), (0.83, -0.08), (1.0, 0.0))
            .corner()
            .line(1.0, 1.0)
            .corner()
            .curve((0.83, 0.92), (0.67, 0.92), (0.5, 1.0))
            .curve((0.33, 1.08), (0.17, 1.08), (0.0, 1.0))
            .corner()
            .close(),
        // Cross.
        21 => polygon(
            &[
                (0.3, 0.3),
                (0.3, 0.0),
                (0.7, 0.0),
                (0.7, 0.3),
                (1.0, 0.3),
                (1.0, 0.7),
                (0.7, 0.7),
                (0.7, 1.0),
                (0.3, 1.0),
                (0.3, 0.7),
                (0.0, 0.7),
                (0.0, 0.3),
            ],
            [0, 3, 6, 9],
        ),
        // Cloud: eight bumps.
        22 => {
            let at = |a: f64| {
                let t = a.to_radians();
                (0.5 + 0.42 * t.cos(), 0.5 + 0.42 * t.sin())
            };
            let first = at(225.0);
            let mut o = Outline::start(first.0, first.1);
            for k in 0..8 {
                let (a0, a1) = (225.0 + 45.0 * k as f64, 225.0 + 45.0 * (k + 1) as f64);
                let e = at(a1);
                let mid = (a0 + a1) / 2.0;
                let bump = |a: f64| {
                    let t = a.to_radians();
                    (0.5 + 0.62 * t.cos(), 0.5 + 0.62 * t.sin())
                };
                let h1 = bump(mid - 14.0);
                let h2 = bump(mid + 14.0);
                if k % 2 == 0 {
                    o = o.corner();
                }
                o = o.curve(h1, h2, e);
            }
            o.close()
        }
        // Speech bubble.
        23 => Outline::start(0.0, 0.25)
            .corner()
            .line(0.2, 0.25)
            .line(0.1, 0.0)
            .line(0.4, 0.25)
            .line(1.0, 0.25)
            .corner()
            .line(1.0, 1.0)
            .corner()
            .line(0.0, 1.0)
            .corner()
            .close(),
        _ => outline(0),
    }
}

/// A straight-sided outline through `pts` with the given corner nodes.
fn polygon(pts: &[(f64, f64)], corners: [usize; 4]) -> Outline {
    let mut o = Outline::start(pts[0].0, pts[0].1);
    for q in &pts[1..] {
        o = o.line(q.0, q.1);
    }
    o.close().with_corners(corners)
}

impl Outline {
    fn with_corners(mut self, c: [usize; 4]) -> Self {
        self.corners = c.to_vec();
        self
    }
}

/// Preset `i`'s frame around `b`.
pub fn frame(i: usize, b: Rect) -> Option<Frame> {
    outline(i).frame(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_makes_a_frame_with_four_increasing_corners() {
        let b = Rect::new(10.0, 20.0, 110.0, 70.0);
        for i in 0..COUNT {
            let o = outline(i);
            assert_eq!(o.nodes.len(), o.handles.len(), "preset {i}");
            assert_eq!(o.corners.len(), 4, "preset {i}: {:?}", o.corners);
            assert!(
                o.corners.windows(2).all(|w| w[0] < w[1]),
                "preset {i}: {:?}",
                o.corners
            );
            let f = frame(i, b).unwrap_or_else(|| panic!("preset {i} has a frame"));
            assert_eq!(f.corners.to_vec(), o.corners, "preset {i}");
            // The frame stays near the box.
            let fb = f.bounds();
            assert!(fb.x0 > -10.0 && fb.x1 < 130.0, "preset {i}: {fb:?}");
        }
        // The square leaves the box as it is.
        let sq = frame(0, b).expect("square");
        assert_eq!(sq.nodes[0], Point::new(10.0, 20.0));
        assert_eq!(sq.nodes[2], Point::new(110.0, 70.0));
        // The circle's nodes lie on the ellipse in the box.
        let c = frame(2, b).expect("circle");
        for n in &c.nodes {
            let (x, y) = ((n.x - 60.0) / 50.0, (n.y - 45.0) / 25.0);
            assert!((x * x + y * y - 1.0).abs() < 1e-6, "{n:?}");
        }
    }
}
