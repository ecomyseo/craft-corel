//! Registration marks for print and cut, and crop marks.
//!
//! Registration marks are filled shapes an optical sensor finds: one at
//! each corner of a box and, optionally, extra ones along the sides.
//! Around artwork (`inside` off) each corner mark sits outside the box,
//! its nearest edge `offset` away from the box's corner in x and in y; on a
//! page (`inside` on) it sits inside the box, `offset` in from both edges.
//! Styles: a square of side `size`, a circle of diameter `size`, or an L
//! whose arms (`corner_length` long, `corner_width` wide) run along the
//! two sides of the corner. Extra marks: when a side is longer than
//! `spacing`, it is split into `ceil(length / spacing)` equal parts and a
//! mark (a square for the L style) sits at each division, in line with
//! the corner marks, on both opposite sides.
//!
//! Crop marks: at each corner of a box two lines, `length` long, along
//! the box's sides produced, starting `offset` away from the corner.

use super::{fin, finite_rect, line_path, polygon_path, rect_path};
use tracedraw_core::geometry::{BezPath, Circle, Point, Rect, Shape as _};

/// Shape of the registration marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum MarkStyle {
    #[default]
    Square,
    Circle,
    Corner,
}

/// Settings of [`registration_marks`], lengths in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct RegSpec {
    pub style: MarkStyle,
    /// Side of a square, diameter of a circle.
    pub size: f64,
    /// Distance between the box and the marks.
    pub offset: f64,
    /// Extra marks along the sides every so far; 0: none.
    pub spacing: f64,
    pub corner_length: f64,
    pub corner_width: f64,
    /// The marks go inside the box (a page) instead of around it.
    pub inside: bool,
}

impl Default for RegSpec {
    fn default() -> Self {
        RegSpec {
            style: MarkStyle::Square,
            size: 5.0,
            offset: 10.0,
            spacing: 400.0,
            corner_length: 20.0,
            corner_width: 1.0,
            inside: false,
        }
    }
}

/// The most extra marks along one side.
pub const MAX_EXTRA: usize = 100;

/// The four corners of a box with their outward directions, from the
/// bottom-left one counter-clockwise.
fn corners(r: Rect) -> [(Point, f64, f64); 4] {
    [
        (Point::new(r.x0, r.y0), -1.0, -1.0),
        (Point::new(r.x1, r.y0), 1.0, -1.0),
        (Point::new(r.x1, r.y1), 1.0, 1.0),
        (Point::new(r.x0, r.y1), -1.0, 1.0),
    ]
}

fn square_or_circle(c: Point, size: f64, circle: bool) -> BezPath {
    if circle {
        Circle::new(c, size / 2.0).to_path(0.01)
    } else {
        rect_path(Rect::new(
            c.x - size / 2.0,
            c.y - size / 2.0,
            c.x + size / 2.0,
            c.y + size / 2.0,
        ))
    }
}

/// Registration marks for the box `area` (closed paths to fill).
pub fn registration_marks(area: Rect, spec: &RegSpec) -> Vec<BezPath> {
    let mut out = Vec::new();
    if !finite_rect(area) {
        return out;
    }
    let r = area.abs();
    let size = fin(spec.size, 0.0);
    let offset = fin(spec.offset, 0.0).max(0.0);
    if size <= 0.0 {
        return out;
    }
    // Outward (1) or inward (-1).
    let k = if spec.inside { -1.0 } else { 1.0 };
    // Centre of the square or circle at each corner.
    let centre = |p: Point, sx: f64, sy: f64| {
        let d = offset + size / 2.0;
        Point::new(p.x + k * sx * d, p.y + k * sy * d)
    };
    for (p, sx, sy) in corners(r) {
        match spec.style {
            MarkStyle::Square | MarkStyle::Circle => out.push(square_or_circle(
                centre(p, sx, sy),
                size,
                spec.style == MarkStyle::Circle,
            )),
            MarkStyle::Corner => {
                let len = fin(spec.corner_length, 0.0).max(0.0);
                let w = fin(spec.corner_width, 0.0).max(0.0);
                if len <= 0.0 || w <= 0.0 {
                    continue;
                }
                let w = w.min(len);
                // The L's outer corner and the arms' direction (towards
                // the other corners of the box).
                let (ax, ay) = (-sx, -sy);
                let outer = if spec.inside {
                    Point::new(p.x + ax * offset, p.y + ay * offset)
                } else {
                    Point::new(p.x + sx * (offset + w), p.y + sy * (offset + w))
                };
                let at = |u: f64, v: f64| Point::new(outer.x + u * ax, outer.y + v * ay);
                out.push(polygon_path(&[
                    at(0.0, 0.0),
                    at(len, 0.0),
                    at(len, w),
                    at(w, w),
                    at(w, len),
                    at(0.0, len),
                ]));
            }
        }
    }
    let spacing = fin(spec.spacing, 0.0);
    if spacing > 0.0 {
        let circle = spec.style == MarkStyle::Circle;
        let bl = centre(Point::new(r.x0, r.y0), -1.0, -1.0);
        let tr = centre(Point::new(r.x1, r.y1), 1.0, 1.0);
        let mut along = |from: f64, to: f64, fixed: [f64; 2], horizontal: bool| {
            let span = (to - from).abs();
            if span <= spacing {
                return;
            }
            let n = ((span / spacing).ceil() as usize).min(MAX_EXTRA + 1);
            for i in 1..n {
                let t = from + (to - from) * i as f64 / n as f64;
                for f in fixed {
                    let c = if horizontal {
                        Point::new(t, f)
                    } else {
                        Point::new(f, t)
                    };
                    out.push(square_or_circle(c, size, circle));
                }
            }
        };
        along(bl.x, tr.x, [bl.y, tr.y], true);
        along(bl.y, tr.y, [bl.x, tr.x], false);
    }
    out
}

/// Settings of [`crop_marks`], in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct CropSpec {
    pub length: f64,
    pub offset: f64,
}

impl Default for CropSpec {
    fn default() -> Self {
        CropSpec {
            length: 5.0,
            offset: 3.0,
        }
    }
}

/// Crop marks for the box `r`: one path per corner, each with its two
/// lines.
pub fn crop_marks(r: Rect, spec: &CropSpec) -> Vec<BezPath> {
    if !finite_rect(r) {
        return Vec::new();
    }
    let len = fin(spec.length, 0.0);
    let off = fin(spec.offset, 0.0).max(0.0);
    if len <= 0.0 {
        return Vec::new();
    }
    corners(r.abs())
        .iter()
        .map(|(p, sx, sy)| {
            let mut path = line_path(
                Point::new(p.x + sx * off, p.y),
                Point::new(p.x + sx * (off + len), p.y),
            );
            path.extend(line_path(
                Point::new(p.x, p.y + sy * off),
                Point::new(p.x, p.y + sy * (off + len)),
            ));
            path
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxes(paths: &[BezPath]) -> Vec<Rect> {
        paths.iter().map(|p| p.bounding_box()).collect()
    }

    fn near(a: Rect, b: Rect) -> bool {
        (a.x0 - b.x0).abs() < 1e-6
            && (a.y0 - b.y0).abs() < 1e-6
            && (a.x1 - b.x1).abs() < 1e-6
            && (a.y1 - b.y1).abs() < 1e-6
    }

    #[test]
    fn square_marks_sit_outside_the_corners() {
        let m = registration_marks(Rect::new(0.0, 0.0, 100.0, 50.0), &RegSpec::default());
        assert_eq!(m.len(), 4);
        let b = boxes(&m);
        assert!(
            near(b[0], Rect::new(-15.0, -15.0, -10.0, -10.0)),
            "{:?}",
            b[0]
        );
        assert!(
            near(b[2], Rect::new(110.0, 60.0, 115.0, 65.0)),
            "{:?}",
            b[2]
        );
        // On a page they go inside.
        let page = registration_marks(
            Rect::new(0.0, 0.0, 210.0, 297.0),
            &RegSpec {
                inside: true,
                ..Default::default()
            },
        );
        let b = boxes(&page);
        assert!(near(b[0], Rect::new(10.0, 10.0, 15.0, 15.0)), "{:?}", b[0]);
        assert!(
            near(b[2], Rect::new(195.0, 282.0, 200.0, 287.0)),
            "{:?}",
            b[2]
        );
    }

    #[test]
    fn long_sides_get_extra_marks() {
        let spec = RegSpec {
            style: MarkStyle::Circle,
            ..Default::default()
        };
        // Centres 25 mm outside: 1050 x 125 between centres; 1050 / 400
        // gives 3 parts, 2 marks on top and 2 below.
        let m = registration_marks(Rect::new(0.0, 0.0, 1025.0, 100.0), &spec);
        assert_eq!(m.len(), 4 + 4);
        let c = m[4].bounding_box().center();
        assert!((c.x - (-12.5 + 1050.0 / 3.0)).abs() < 1e-6, "{c:?}");
        assert!((c.y + 12.5).abs() < 1e-6);
        assert!((m[4].bounding_box().width() - 5.0).abs() < 1e-3);
        let none = RegSpec {
            spacing: 0.0,
            ..spec
        };
        assert_eq!(
            registration_marks(Rect::new(0.0, 0.0, 1025.0, 100.0), &none).len(),
            4
        );
    }

    #[test]
    fn corner_marks_are_ls_along_the_sides() {
        let spec = RegSpec {
            style: MarkStyle::Corner,
            ..Default::default()
        };
        let m = registration_marks(Rect::new(0.0, 0.0, 100.0, 100.0), &spec);
        assert_eq!(m.len(), 4);
        // Bottom-left: outer corner at (-11, -11), arms 20 long to the
        // right and up.
        assert!(near(m[0].bounding_box(), Rect::new(-11.0, -11.0, 9.0, 9.0)));
        let area = m[0].area().abs();
        assert!((area - (20.0 + 19.0)).abs() < 1e-9, "{area}");
        assert!(near(
            m[2].bounding_box(),
            Rect::new(91.0, 91.0, 111.0, 111.0)
        ));
    }

    #[test]
    fn crop_marks_at_each_corner() {
        let m = crop_marks(Rect::new(0.0, 0.0, 50.0, 20.0), &CropSpec::default());
        assert_eq!(m.len(), 4);
        assert_eq!(crate::subpaths(&m[0]).len(), 2);
        assert!(near(m[0].bounding_box(), Rect::new(-8.0, -8.0, 0.0, 0.0)));
        assert!(near(m[2].bounding_box(), Rect::new(50.0, 20.0, 58.0, 28.0)));
    }

    #[test]
    fn bad_input_gives_no_marks() {
        let nan = Rect::new(f64::NAN, 0.0, 1.0, 1.0);
        assert!(registration_marks(nan, &RegSpec::default()).is_empty());
        assert!(crop_marks(nan, &CropSpec::default()).is_empty());
        let r = Rect::new(0.0, 0.0, 1.0, 1.0);
        let bad = RegSpec {
            size: f64::NAN,
            ..Default::default()
        };
        assert!(registration_marks(r, &bad).is_empty());
        let huge = RegSpec {
            spacing: 1e-9,
            offset: f64::INFINITY,
            ..Default::default()
        };
        assert!(
            registration_marks(Rect::new(0.0, 0.0, 1e6, 1.0), &huge).len() <= 4 + 4 * MAX_EXTRA
        );
        assert!(crop_marks(
            r,
            &CropSpec {
                length: -1.0,
                offset: 0.0
            }
        )
        .is_empty());
        // A flat box still gets marks.
        assert_eq!(
            registration_marks(Rect::new(0.0, 0.0, 10.0, 0.0), &RegSpec::default()).len(),
            4
        );
    }
}
