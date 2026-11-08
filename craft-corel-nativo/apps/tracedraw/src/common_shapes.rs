//! The Common Shapes tool's shapes, in five sets: basic shapes, arrows,
//! flowchart symbols, banners and callouts. Each is drawn into the box
//! dragged (`CommonShape::path`), as one curve object that fills even-odd
//! like every curve: parts that should look separate are separate
//! subpaths side by side, never overlapping, and inner details are
//! straight two-point lines.

use crate::i18n::tr;
use tracedraw_core::geometry::{BezPath, Point, Rect};

/// The sets of shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Basic,
    Arrows,
    Flowchart,
    Banners,
    Callouts,
}

impl Category {
    pub const ALL: [Category; 5] = [
        Category::Basic,
        Category::Arrows,
        Category::Flowchart,
        Category::Banners,
        Category::Callouts,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Category::Basic => "shapes.basic",
            Category::Arrows => "shapes.arrows",
            Category::Flowchart => "shapes.flowchart",
            Category::Banners => "shapes.banners",
            Category::Callouts => "shapes.callouts",
        }
    }
}

macro_rules! shapes {
    ($( $cat:ident => [ $( $v:ident = $key:literal ),* $(,)? ] ),* $(,)?) => {
        /// One shape of the Common Shapes tool.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum CommonShape { $( $( $v, )* )* }

        impl CommonShape {
            /// Every shape, set by set.
            pub const ALL: &'static [CommonShape] = &[ $( $( CommonShape::$v, )* )* ];

            pub fn category(self) -> Category {
                match self { $( $( CommonShape::$v => Category::$cat, )* )* }
            }

            /// The i18n key of its name.
            pub fn key(self) -> &'static str {
                match self { $( $( CommonShape::$v => $key, )* )* }
            }
        }
    };
}

shapes! {
    Basic => [
        Parallelogram = "shape.parallelogram",
        Trapezoid = "shape.trapezoid",
        Diamond = "shape.diamond",
        Triangle = "shape.triangle",
        RightTriangle = "shape.right_triangle",
        Hexagon = "shape.hexagon",
        Octagon = "shape.octagon",
        Cross = "shape.cross",
        Heart = "shape.heart",
        Lightning = "shape.lightning",
        Crescent = "shape.crescent",
        Ring = "shape.ring",
        Cylinder = "shape.cylinder",
        Cube = "shape.cube",
        Frame = "shape.frame",
        Teardrop = "shape.teardrop",
        Smiley = "shape.smiley",
        NoSymbol = "shape.no_symbol",
    ],
    Arrows => [
        RightArrow = "shape.arrow",
        LeftArrow = "shape.left_arrow",
        UpArrow = "shape.up_arrow",
        DownArrow = "shape.down_arrow",
        LeftRightArrow = "shape.left_right_arrow",
        UpDownArrow = "shape.up_down_arrow",
        QuadArrow = "shape.quad_arrow",
        Chevron = "shape.chevron",
        Pointer = "shape.pointer",
        NotchedArrow = "shape.notched_arrow",
        StripedArrow = "shape.striped_arrow",
        BentArrow = "shape.bent_arrow",
    ],
    Flowchart => [
        Process = "shape.process",
        AlternateProcess = "shape.alternate_process",
        Decision = "shape.decision",
        Data = "shape.data",
        PredefinedProcess = "shape.predefined_process",
        InternalStorage = "shape.internal_storage",
        Document = "shape.document",
        Multidocument = "shape.multidocument",
        Terminator = "shape.terminator",
        Preparation = "shape.preparation",
        ManualInput = "shape.manual_input",
        ManualOperation = "shape.manual_operation",
        Connector = "shape.connector",
        OffPageConnector = "shape.off_page_connector",
        Card = "shape.card",
        PunchedTape = "shape.punched_tape",
        SummingJunction = "shape.summing_junction",
        Or = "shape.or",
        Collate = "shape.collate",
        Sort = "shape.sort",
        Extract = "shape.extract",
        Merge = "shape.merge",
        StoredData = "shape.stored_data",
        Delay = "shape.delay",
        MagneticDisk = "shape.magnetic_disk",
        DirectAccessStorage = "shape.direct_access_storage",
        Display = "shape.display",
    ],
    Banners => [
        Banner = "shape.banner",
        FoldedRibbon = "shape.folded_ribbon",
        Wave = "shape.wave",
        DoubleWave = "shape.double_wave",
        Scroll = "shape.scroll",
        Explosion = "shape.explosion",
        Burst = "shape.burst",
        Pennant = "shape.pennant",
    ],
    Callouts => [
        Callout = "shape.callout",
        RoundedCallout = "shape.rounded_callout",
        OvalCallout = "shape.oval_callout",
        CloudCallout = "shape.cloud_callout",
        LineCallout = "shape.line_callout",
        BentLineCallout = "shape.bent_line_callout",
    ],
}

/// Builds a path in the unit square (u right, v up, both 0 to 1) and
/// places it in a box.
struct Pen {
    r: Rect,
    path: BezPath,
}

impl Pen {
    fn new(r: Rect) -> Pen {
        Pen {
            r,
            path: BezPath::new(),
        }
    }

    fn at(&self, u: f64, v: f64) -> Point {
        Point::new(
            self.r.x0 + u * self.r.width(),
            self.r.y0 + v * self.r.height(),
        )
    }

    fn poly(&mut self, pts: &[(f64, f64)]) {
        for (i, &(u, v)) in pts.iter().enumerate() {
            let p = self.at(u, v);
            if i == 0 {
                self.path.move_to(p);
            } else {
                self.path.line_to(p);
            }
        }
        self.path.close_path();
    }

    /// A straight detail line (a two-point subpath: it is stroked, never
    /// filled).
    fn line(&mut self, a: (f64, f64), b: (f64, f64)) {
        let (a, b) = (self.at(a.0, a.1), self.at(b.0, b.1));
        self.path.move_to(a);
        self.path.line_to(b);
    }

    fn move_to(&mut self, u: f64, v: f64) {
        let p = self.at(u, v);
        self.path.move_to(p);
    }

    fn line_to(&mut self, u: f64, v: f64) {
        let p = self.at(u, v);
        self.path.line_to(p);
    }

    fn curve_to(&mut self, c1: (f64, f64), c2: (f64, f64), to: (f64, f64)) {
        let (a, b, c) = (
            self.at(c1.0, c1.1),
            self.at(c2.0, c2.1),
            self.at(to.0, to.1),
        );
        self.path.curve_to(a, b, c);
    }

    fn close(&mut self) {
        self.path.close_path();
    }

    /// An elliptical arc from angle `a0` to `a1` (radians, counter-
    /// clockwise from +u) around (cu, cv) with radii (ru, rv), as cubic
    /// pieces of at most a quarter turn; starts with a line from the
    /// current point (or a move when `start` is set).
    fn arc(&mut self, c: (f64, f64), rad: (f64, f64), a0: f64, a1: f64, start: bool) {
        let point = |a: f64| (c.0 + rad.0 * a.cos(), c.1 + rad.1 * a.sin());
        let p0 = point(a0);
        if start {
            self.move_to(p0.0, p0.1);
        } else {
            self.line_to(p0.0, p0.1);
        }
        let n = ((a1 - a0).abs() / std::f64::consts::FRAC_PI_2)
            .ceil()
            .max(1.0) as usize;
        let step = (a1 - a0) / n as f64;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        for i in 0..n {
            let (s, e) = (a0 + step * i as f64, a0 + step * (i + 1) as f64);
            let (ps, pe) = (point(s), point(e));
            let c1 = (ps.0 - k * rad.0 * s.sin(), ps.1 + k * rad.1 * s.cos());
            let c2 = (pe.0 + k * rad.0 * e.sin(), pe.1 - k * rad.1 * e.cos());
            self.curve_to(c1, c2, pe);
        }
    }

    fn ellipse(&mut self, c: (f64, f64), rad: (f64, f64)) {
        self.arc(c, rad, 0.0, std::f64::consts::TAU, true);
        self.close();
    }

    /// A rectangle with corners rounded by `radius` box units (the same
    /// size across and up, whatever the box's shape).
    fn rounded(&mut self, u0: f64, v0: f64, u1: f64, v1: f64, radius: f64) {
        let (w, h) = (self.r.width().max(1e-9), self.r.height().max(1e-9));
        let rmax = ((u1 - u0) * w).min((v1 - v0) * h) / 2.0;
        let r = radius.min(rmax).max(0.0);
        let (ru, rv) = (r / w, r / h);
        use std::f64::consts::{FRAC_PI_2, PI};
        self.arc((u1 - ru, v0 + rv), (ru, rv), -FRAC_PI_2, 0.0, true);
        self.arc((u1 - ru, v1 - rv), (ru, rv), 0.0, FRAC_PI_2, false);
        self.arc((u0 + ru, v1 - rv), (ru, rv), FRAC_PI_2, PI, false);
        self.arc((u0 + ru, v0 + rv), (ru, rv), PI, PI + FRAC_PI_2, false);
        self.close();
    }

    /// A star around (0.5, 0.5) with the given radii, point by point
    /// (fractions of the half box), the first point straight up.
    fn star(&mut self, radii: &[f64]) {
        let n = radii.len();
        let pts: Vec<(f64, f64)> = radii
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let a = std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU * i as f64 / n as f64;
                (0.5 + 0.5 * r * a.cos(), 0.5 + 0.5 * r * a.sin())
            })
            .collect();
        self.poly(&pts);
    }

    /// A band whose bottom and top edges are waves of `periods` periods
    /// and height `amp`.
    fn wave_band(&mut self, v0: f64, v1: f64, amp: f64, periods: usize) {
        let n = periods.max(1);
        self.move_to(0.0, v0);
        for i in 0..n {
            let (a, b) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
            let d = (b - a) / 3.0;
            self.curve_to((a + d, v0 - amp), (b - d, v0 + amp), (b, v0));
        }
        self.line_to(1.0, v1);
        for i in (0..n).rev() {
            let (a, b) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
            let d = (b - a) / 3.0;
            self.curve_to((b - d, v1 + amp), (a + d, v1 - amp), (a, v1));
        }
        self.close();
    }

    /// A cylinder standing in the box: the top ellipse and the body under
    /// its front edge, side by side.
    fn cylinder(&mut self) {
        use std::f64::consts::PI;
        let rv = 0.12;
        self.ellipse((0.5, 1.0 - rv), (0.5, rv));
        // Body: front half of the top ellipse down to the bottom.
        self.arc((0.5, 1.0 - rv), (0.5, rv), PI, 2.0 * PI, true);
        self.arc((0.5, rv), (0.5, rv), 0.0, -PI, false);
        self.close();
    }

    /// A cylinder lying down: the right end's ellipse and the body.
    fn lying_cylinder(&mut self) {
        use std::f64::consts::{FRAC_PI_2, PI};
        let ru = 0.12;
        self.ellipse((1.0 - ru, 0.5), (ru, 0.5));
        self.arc((1.0 - ru, 0.5), (ru, 0.5), FRAC_PI_2, PI + FRAC_PI_2, true);
        self.arc((ru, 0.5), (ru, 0.5), PI + FRAC_PI_2, FRAC_PI_2, false);
        self.close();
    }

    /// A hole: drawn by `build` like any part, then turned to run against
    /// the outer outlines (which run counter-clockwise), so it stays open
    /// under either fill rule.
    fn hole(&mut self, build: impl FnOnce(&mut Pen)) {
        use tracedraw_core::geometry::Shape as _;
        let mut part = Pen::new(self.r);
        build(&mut part);
        let mut path = part.finish();
        if path.area() > 0.0 {
            path = path.reverse_subpaths();
        }
        self.path.extend(path);
    }

    fn finish(self) -> BezPath {
        self.path
    }
}

/// The right arrow's outline: shaft 40 % of the height, head 40 % of
/// the width.
const RIGHT_ARROW: [(f64, f64); 7] = [
    (0.0, 0.3),
    (0.6, 0.3),
    (0.6, 0.0),
    (1.0, 0.5),
    (0.6, 1.0),
    (0.6, 0.7),
    (0.0, 0.7),
];

const LEFT_RIGHT_ARROW: [(f64, f64); 10] = [
    (0.0, 0.5),
    (0.3, 1.0),
    (0.3, 0.7),
    (0.7, 0.7),
    (0.7, 1.0),
    (1.0, 0.5),
    (0.7, 0.0),
    (0.7, 0.3),
    (0.3, 0.3),
    (0.3, 0.0),
];

fn mapped(pts: &[(f64, f64)], f: impl Fn(f64, f64) -> (f64, f64)) -> Vec<(f64, f64)> {
    pts.iter().map(|&(u, v)| f(u, v)).collect()
}

/// The document symbol's outline (straight top, wavy bottom) in the
/// part (u0..u1, v0..v1) of the box.
fn document(p: &mut Pen, u0: f64, v0: f64, u1: f64, v1: f64) {
    let (w, h) = (u1 - u0, v1 - v0);
    let at = |u: f64, v: f64| (u0 + u * w, v0 + v * h);
    let s = at(0.0, 0.12);
    p.move_to(s.0, s.1);
    p.curve_to(at(0.3, -0.12), at(0.6, 0.36), at(1.0, 0.08));
    let t = at(1.0, 1.0);
    p.line_to(t.0, t.1);
    let t = at(0.0, 1.0);
    p.line_to(t.0, t.1);
    p.close();
}

impl CommonShape {
    pub fn name(self) -> String {
        tr(self.key())
    }

    /// The shapes of one set.
    pub fn of(category: Category) -> impl Iterator<Item = CommonShape> {
        CommonShape::ALL
            .iter()
            .copied()
            .filter(move |s| s.category() == category)
    }

    /// The shape drawn into `r`.
    pub fn path(self, r: Rect) -> BezPath {
        use std::f64::consts::{FRAC_PI_2, PI, TAU};
        let mut p = Pen::new(r);
        let min_side = r.width().abs().min(r.height().abs());
        match self {
            // ----- Basic shapes -----
            CommonShape::Parallelogram => {
                p.poly(&[(0.0, 0.0), (0.75, 0.0), (1.0, 1.0), (0.25, 1.0)])
            }
            CommonShape::Trapezoid => p.poly(&[(0.0, 0.0), (1.0, 0.0), (0.75, 1.0), (0.25, 1.0)]),
            CommonShape::Diamond | CommonShape::Decision => {
                p.poly(&[(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)])
            }
            CommonShape::Triangle | CommonShape::Extract => {
                p.poly(&[(0.5, 1.0), (1.0, 0.0), (0.0, 0.0)])
            }
            CommonShape::Merge => p.poly(&[(0.0, 1.0), (1.0, 1.0), (0.5, 0.0)]),
            CommonShape::RightTriangle => p.poly(&[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]),
            CommonShape::Hexagon => p.poly(&[
                (0.25, 0.0),
                (0.75, 0.0),
                (1.0, 0.5),
                (0.75, 1.0),
                (0.25, 1.0),
                (0.0, 0.5),
            ]),
            CommonShape::Octagon => {
                let c = 0.2929;
                p.poly(&[
                    (c, 0.0),
                    (1.0 - c, 0.0),
                    (1.0, c),
                    (1.0, 1.0 - c),
                    (1.0 - c, 1.0),
                    (c, 1.0),
                    (0.0, 1.0 - c),
                    (0.0, c),
                ])
            }
            CommonShape::Cross => p.poly(&[
                (0.35, 0.0),
                (0.65, 0.0),
                (0.65, 0.35),
                (1.0, 0.35),
                (1.0, 0.65),
                (0.65, 0.65),
                (0.65, 1.0),
                (0.35, 1.0),
                (0.35, 0.65),
                (0.0, 0.65),
                (0.0, 0.35),
                (0.35, 0.35),
            ]),
            CommonShape::Heart => {
                p.move_to(0.5, 0.0);
                p.curve_to((0.1, 0.35), (0.0, 0.55), (0.0, 0.72));
                p.curve_to((0.0, 0.95), (0.25, 1.05), (0.5, 0.78));
                p.curve_to((0.75, 1.05), (1.0, 0.95), (1.0, 0.72));
                p.curve_to((1.0, 0.55), (0.9, 0.35), (0.5, 0.0));
                p.close();
            }
            CommonShape::Lightning => p.poly(&[
                (0.57, 1.0),
                (0.0, 0.45),
                (0.43, 0.45),
                (0.21, 0.0),
                (1.0, 0.6),
                (0.57, 0.6),
                (0.79, 1.0),
            ]),
            CommonShape::Crescent => {
                // Out along a wide half ellipse, back along a narrower one
                // with the same ends.
                p.arc((1.0, 0.5), (1.0, 0.5), FRAC_PI_2, PI + FRAC_PI_2, true);
                p.arc((1.0, 0.5), (0.6, 0.5), PI + FRAC_PI_2, FRAC_PI_2, false);
                p.close();
            }
            CommonShape::Ring => {
                p.ellipse((0.5, 0.5), (0.5, 0.5));
                p.hole(|p| p.ellipse((0.5, 0.5), (0.3, 0.3)));
            }
            CommonShape::Cylinder | CommonShape::MagneticDisk => p.cylinder(),
            CommonShape::Cube => {
                let d = 0.25;
                // Front, top and side faces, edge to edge.
                p.poly(&[
                    (0.0, 0.0),
                    (1.0 - d, 0.0),
                    (1.0 - d, 1.0 - d),
                    (0.0, 1.0 - d),
                ]);
                p.poly(&[(0.0, 1.0 - d), (1.0 - d, 1.0 - d), (1.0, 1.0), (d, 1.0)]);
                p.poly(&[(1.0 - d, 0.0), (1.0, d), (1.0, 1.0), (1.0 - d, 1.0 - d)]);
            }
            CommonShape::Frame => {
                p.poly(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
                p.hole(|p| p.poly(&[(0.15, 0.15), (0.85, 0.15), (0.85, 0.85), (0.15, 0.85)]));
            }
            CommonShape::Teardrop => {
                // Round at the bottom, the sides running straight up to
                // the point (tangent where they meet the round part).
                let a = 1.0;
                p.arc(
                    (0.5, 0.37),
                    (0.45, 0.37),
                    FRAC_PI_2 + a,
                    TAU + FRAC_PI_2 - a,
                    true,
                );
                p.line_to(0.5, 1.0);
                p.close();
            }
            CommonShape::Smiley => {
                p.ellipse((0.5, 0.5), (0.5, 0.5));
                p.hole(|p| p.ellipse((0.35, 0.65), (0.07, 0.09)));
                p.hole(|p| p.ellipse((0.65, 0.65), (0.07, 0.09)));
                // The mouth: a crescent under the middle.
                p.hole(|p| {
                    p.arc((0.5, 0.45), (0.25, 0.2), PI + 0.35, TAU - 0.35, true);
                    p.arc((0.5, 0.38), (0.21, 0.1), -0.35, -PI + 0.35, false);
                    p.close();
                });
            }
            CommonShape::NoSymbol => {
                p.ellipse((0.5, 0.5), (0.5, 0.5));
                // The inner circle less the bar from top left to bottom
                // right: two openings.
                let a = 0.32_f64;
                for start in [-PI / 4.0, 3.0 * PI / 4.0] {
                    p.hole(|p| {
                        p.arc((0.5, 0.5), (0.36, 0.36), start + a, start + PI - a, true);
                        p.close();
                    });
                }
            }
            // ----- Arrows -----
            CommonShape::RightArrow => p.poly(&RIGHT_ARROW),
            CommonShape::LeftArrow => p.poly(&mapped(&RIGHT_ARROW, |u, v| (1.0 - u, v))),
            CommonShape::UpArrow => p.poly(&mapped(&RIGHT_ARROW, |u, v| (v, u))),
            CommonShape::DownArrow => p.poly(&mapped(&RIGHT_ARROW, |u, v| (v, 1.0 - u))),
            CommonShape::LeftRightArrow => p.poly(&LEFT_RIGHT_ARROW),
            CommonShape::UpDownArrow => p.poly(&mapped(&LEFT_RIGHT_ARROW, |u, v| (v, u))),
            CommonShape::QuadArrow => p.poly(&[
                (0.5, 1.0),
                (0.68, 0.8),
                (0.58, 0.8),
                (0.58, 0.58),
                (0.8, 0.58),
                (0.8, 0.68),
                (1.0, 0.5),
                (0.8, 0.32),
                (0.8, 0.42),
                (0.58, 0.42),
                (0.58, 0.2),
                (0.68, 0.2),
                (0.5, 0.0),
                (0.32, 0.2),
                (0.42, 0.2),
                (0.42, 0.42),
                (0.2, 0.42),
                (0.2, 0.32),
                (0.0, 0.5),
                (0.2, 0.68),
                (0.2, 0.58),
                (0.42, 0.58),
                (0.42, 0.8),
                (0.32, 0.8),
            ]),
            CommonShape::Chevron => p.poly(&[
                (0.0, 0.0),
                (0.7, 0.0),
                (1.0, 0.5),
                (0.7, 1.0),
                (0.0, 1.0),
                (0.3, 0.5),
            ]),
            CommonShape::Pointer => {
                p.poly(&[(0.0, 0.0), (0.7, 0.0), (1.0, 0.5), (0.7, 1.0), (0.0, 1.0)])
            }
            CommonShape::NotchedArrow => p.poly(&[
                (0.0, 0.3),
                (0.6, 0.3),
                (0.6, 0.0),
                (1.0, 0.5),
                (0.6, 1.0),
                (0.6, 0.7),
                (0.0, 0.7),
                (0.15, 0.5),
            ]),
            CommonShape::StripedArrow => {
                p.poly(&[(0.0, 0.3), (0.05, 0.3), (0.05, 0.7), (0.0, 0.7)]);
                p.poly(&[(0.1, 0.3), (0.18, 0.3), (0.18, 0.7), (0.1, 0.7)]);
                p.poly(&mapped(&RIGHT_ARROW, |u, v| (0.23 + 0.77 * u, v)));
            }
            CommonShape::BentArrow => p.poly(&[
                (0.0, 0.0),
                (0.65, 0.0),
                (0.65, 0.65),
                (0.85, 0.65),
                (0.5, 1.0),
                (0.15, 0.65),
                (0.35, 0.65),
                (0.35, 0.3),
                (0.0, 0.3),
            ]),
            // ----- Flowchart -----
            CommonShape::Process => p.poly(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]),
            CommonShape::AlternateProcess => p.rounded(0.0, 0.0, 1.0, 1.0, min_side * 0.15),
            CommonShape::Data => p.poly(&[(0.0, 0.0), (0.8, 0.0), (1.0, 1.0), (0.2, 1.0)]),
            CommonShape::PredefinedProcess => {
                p.poly(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
                p.line((0.1, 0.0), (0.1, 1.0));
                p.line((0.9, 0.0), (0.9, 1.0));
            }
            CommonShape::InternalStorage => {
                p.poly(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
                p.line((0.15, 0.0), (0.15, 1.0));
                p.line((0.0, 0.85), (1.0, 0.85));
            }
            CommonShape::Document => document(&mut p, 0.0, 0.0, 1.0, 1.0),
            CommonShape::Multidocument => {
                document(&mut p, 0.0, 0.0, 0.8, 0.8);
                // The two pages behind: the strips that show.
                p.poly(&[
                    (0.1, 0.8),
                    (0.1, 0.9),
                    (0.9, 0.9),
                    (0.9, 0.16),
                    (0.8, 0.16),
                    (0.8, 0.8),
                ]);
                p.poly(&[
                    (0.2, 0.9),
                    (0.2, 1.0),
                    (1.0, 1.0),
                    (1.0, 0.26),
                    (0.9, 0.26),
                    (0.9, 0.9),
                ]);
            }
            CommonShape::Terminator => p.rounded(0.0, 0.0, 1.0, 1.0, min_side / 2.0),
            CommonShape::Preparation => p.poly(&[
                (0.0, 0.5),
                (0.2, 1.0),
                (0.8, 1.0),
                (1.0, 0.5),
                (0.8, 0.0),
                (0.2, 0.0),
            ]),
            CommonShape::ManualInput => p.poly(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 0.7)]),
            CommonShape::ManualOperation => {
                p.poly(&[(0.0, 1.0), (1.0, 1.0), (0.8, 0.0), (0.2, 0.0)])
            }
            CommonShape::Connector => p.ellipse((0.5, 0.5), (0.5, 0.5)),
            CommonShape::OffPageConnector => {
                p.poly(&[(0.0, 1.0), (1.0, 1.0), (1.0, 0.35), (0.5, 0.0), (0.0, 0.35)])
            }
            CommonShape::Card => {
                p.poly(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.2, 1.0), (0.0, 0.8)])
            }
            CommonShape::PunchedTape => p.wave_band(0.12, 0.88, 0.16, 1),
            CommonShape::SummingJunction => {
                p.ellipse((0.5, 0.5), (0.5, 0.5));
                let d = 0.5 * std::f64::consts::FRAC_1_SQRT_2;
                p.line((0.5 - d, 0.5 - d), (0.5 + d, 0.5 + d));
                p.line((0.5 - d, 0.5 + d), (0.5 + d, 0.5 - d));
            }
            CommonShape::Or => {
                p.ellipse((0.5, 0.5), (0.5, 0.5));
                p.line((0.0, 0.5), (1.0, 0.5));
                p.line((0.5, 0.0), (0.5, 1.0));
            }
            CommonShape::Collate => {
                p.poly(&[(0.0, 1.0), (1.0, 1.0), (0.5, 0.5)]);
                p.poly(&[(0.0, 0.0), (1.0, 0.0), (0.5, 0.5)]);
            }
            CommonShape::Sort => {
                p.poly(&[(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)]);
                p.line((0.0, 0.5), (1.0, 0.5));
            }
            CommonShape::StoredData => {
                p.move_to(0.15, 1.0);
                p.line_to(1.0, 1.0);
                p.curve_to((0.88, 0.8), (0.88, 0.2), (1.0, 0.0));
                p.line_to(0.15, 0.0);
                p.curve_to((-0.05, 0.2), (-0.05, 0.8), (0.15, 1.0));
                p.close();
            }
            CommonShape::Delay => {
                p.move_to(0.0, 0.0);
                p.arc((0.5, 0.5), (0.5, 0.5), -FRAC_PI_2, FRAC_PI_2, false);
                p.line_to(0.0, 1.0);
                p.close();
            }
            CommonShape::DirectAccessStorage => p.lying_cylinder(),
            CommonShape::Display => {
                p.move_to(0.0, 0.5);
                p.line_to(0.2, 1.0);
                p.line_to(0.8, 1.0);
                p.curve_to((1.07, 0.85), (1.07, 0.15), (0.8, 0.0));
                p.line_to(0.2, 0.0);
                p.close();
            }
            // ----- Banners -----
            CommonShape::Banner => p.poly(&[
                (0.0, 0.0),
                (0.15, 0.5),
                (0.0, 1.0),
                (1.0, 1.0),
                (0.85, 0.5),
                (1.0, 0.0),
            ]),
            CommonShape::FoldedRibbon => {
                p.poly(&[(0.15, 0.25), (0.85, 0.25), (0.85, 1.0), (0.15, 1.0)]);
                let tail = [
                    (0.0, 0.0),
                    (0.25, 0.0),
                    (0.25, 0.25),
                    (0.15, 0.25),
                    (0.15, 0.7),
                    (0.0, 0.7),
                    (0.08, 0.35),
                ];
                p.poly(&tail);
                p.poly(&mapped(&tail, |u, v| (1.0 - u, v)));
            }
            CommonShape::Wave => p.wave_band(0.15, 0.85, 0.2, 1),
            CommonShape::DoubleWave => p.wave_band(0.1, 0.9, 0.16, 2),
            CommonShape::Scroll => {
                p.poly(&[(0.1, 0.12), (0.9, 0.12), (0.9, 0.88), (0.1, 0.88)]);
                p.rounded(0.0, 0.04, 0.1, 0.96, min_side * 0.05);
                p.rounded(0.9, 0.04, 1.0, 0.96, min_side * 0.05);
            }
            CommonShape::Explosion => p.star(&[
                1.0, 0.55, 0.85, 0.5, 0.95, 0.6, 0.75, 0.45, 1.0, 0.55, 0.8, 0.5, 0.9, 0.45, 0.85,
                0.6,
            ]),
            CommonShape::Burst => {
                let radii: Vec<f64> = (0..48)
                    .map(|i| if i % 2 == 0 { 1.0 } else { 0.82 })
                    .collect();
                p.star(&radii)
            }
            CommonShape::Pennant => {
                p.poly(&[(0.0, 0.0), (0.04, 0.0), (0.04, 1.0), (0.0, 1.0)]);
                p.poly(&[(0.04, 1.0), (1.0, 0.75), (0.04, 0.5)]);
            }
            // ----- Callouts -----
            CommonShape::Callout => {
                p.move_to(0.0, 0.35);
                p.line_to(0.0, 1.0);
                p.line_to(1.0, 1.0);
                p.line_to(1.0, 0.25);
                p.line_to(0.35, 0.25);
                p.line_to(0.15, 0.0);
                p.line_to(0.2, 0.25);
                p.line_to(0.0, 0.25);
                p.close();
            }
            CommonShape::RoundedCallout => {
                let (w, h) = (r.width().max(1e-9), r.height().max(1e-9));
                let rad = (min_side * 0.12).min(0.75 * h * 0.5);
                let (ru, rv) = (rad / w, rad / h);
                p.arc((1.0 - ru, 0.25 + rv), (ru, rv), -FRAC_PI_2, 0.0, true);
                p.arc((1.0 - ru, 1.0 - rv), (ru, rv), 0.0, FRAC_PI_2, false);
                p.arc((ru, 1.0 - rv), (ru, rv), FRAC_PI_2, PI, false);
                p.arc((ru, 0.25 + rv), (ru, rv), PI, PI + FRAC_PI_2, false);
                p.line_to(0.2, 0.25);
                p.line_to(0.15, 0.0);
                p.line_to(0.35, 0.25);
                p.close();
            }
            CommonShape::OvalCallout => {
                // The ellipse open where the tail leaves it.
                let (a0, a1) = (PI + 0.85, PI + 1.25);
                p.arc((0.5, 0.6), (0.5, 0.4), a1, a0 + TAU, true);
                p.line_to(0.12, 0.0);
                p.close();
            }
            CommonShape::CloudCallout => {
                // Bumps around an ellipse, then two small puffs to the tail.
                let n = 10;
                let (cu, cv, eu, ev) = (0.55, 0.6, 0.33, 0.27);
                let pts: Vec<(f64, f64)> = (0..n)
                    .map(|i| {
                        let a = TAU * i as f64 / n as f64;
                        (cu + eu * a.cos(), cv + ev * a.sin())
                    })
                    .collect();
                p.move_to(pts[0].0, pts[0].1);
                for i in 0..n {
                    let (a, b) = (pts[i], pts[(i + 1) % n]);
                    let mid = TAU * (i as f64 + 0.5) / n as f64;
                    let (nu, nv) = (mid.cos() * 0.1, mid.sin() * 0.1);
                    p.curve_to((a.0 + nu, a.1 + nv), (b.0 + nu, b.1 + nv), b);
                }
                p.close();
                p.ellipse((0.22, 0.17), (0.07, 0.06));
                p.ellipse((0.08, 0.05), (0.04, 0.035));
            }
            CommonShape::LineCallout => {
                p.poly(&[(0.3, 0.4), (1.0, 0.4), (1.0, 1.0), (0.3, 1.0)]);
                p.line((0.3, 0.55), (0.0, 0.0));
            }
            CommonShape::BentLineCallout => {
                p.poly(&[(0.3, 0.4), (1.0, 0.4), (1.0, 1.0), (0.3, 1.0)]);
                p.line((0.3, 0.55), (0.15, 0.55));
                p.line((0.15, 0.55), (0.0, 0.0));
            }
        }
        p.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Shape as _;

    #[test]
    fn every_shape_fills_its_box() {
        let r = Rect::new(10.0, 20.0, 50.0, 40.0);
        let mut bad = Vec::new();
        for shape in CommonShape::ALL {
            let path = shape.path(r);
            assert!(!path.elements().is_empty(), "{shape:?}");
            let b = path.bounding_box();
            // Within the box (curves may round off a corner), and filling
            // most of it both ways.
            let inside = b.x0 >= r.x0 - 0.5
                && b.x1 <= r.x1 + 0.5
                && b.y0 >= r.y0 - 0.5
                && b.y1 <= r.y1 + 0.5;
            let fills = b.width() > r.width() * 0.8 && b.height() > r.height() * 0.8;
            if !(inside && fills) {
                bad.push(format!("{shape:?} {b:?}"));
            }
            // Every name is in the string table.
            assert_ne!(shape.name(), shape.key(), "{shape:?}");
        }
        assert!(bad.is_empty(), "{bad:#?}");
        for c in Category::ALL {
            assert!(CommonShape::of(c).count() >= 6, "{c:?}");
            assert_ne!(tr(c.key()), c.key());
        }
        assert_eq!(CommonShape::ALL.len(), 71);
    }

    #[test]
    fn rounded_corners_keep_their_size_in_long_boxes() {
        // A terminator in a wide box: half-circle ends as high as the box.
        let path = CommonShape::Terminator.path(Rect::new(0.0, 0.0, 100.0, 20.0));
        assert!((path.bounding_box().height() - 20.0).abs() < 1e-6);
        let area = path.area().abs();
        let expected = 80.0 * 20.0 + std::f64::consts::PI * 100.0;
        assert!((area - expected).abs() < 2.0, "{area} {expected}");
        // Holes run against the outline, so they stay open under either
        // fill rule: the ring is the outer disk less the inner one.
        let ring = CommonShape::Ring.path(Rect::new(0.0, 0.0, 10.0, 10.0));
        let expected = std::f64::consts::PI * (25.0 - 9.0);
        assert!(
            (ring.area().abs() - expected).abs() < 0.2,
            "{}",
            ring.area()
        );
        let frame = CommonShape::Frame.path(Rect::new(0.0, 0.0, 10.0, 10.0));
        assert!((frame.area().abs() - (100.0 - 49.0)).abs() < 1e-6);
    }
}
