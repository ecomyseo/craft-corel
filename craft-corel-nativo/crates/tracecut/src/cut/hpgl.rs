//! Plotter output: HPGL for cutting plotters, with knife offset
//! compensation and overcut.
//!
//! The outlines are flattened (0.02 mm), turned by 90 degrees
//! counter-clockwise when `rotate` is on, mirrored left to right when
//! `mirror` is on, and placed so the chosen corner of the job's box is the
//! plotter's 0,0 with both axes counting into the job. Coordinates are
//! whole plotter units (`units_per_mm`, 40 for the usual 0.025 mm unit).
//!
//! A swivel knife's tip trails the holder's axis by the knife offset `d`.
//! To cut a point `P` while moving in direction `u`, the holder goes to
//! `P + d u`. At a node where the direction turns by more than
//! `corner_limit` degrees the holder draws an arc of radius `d` around
//! the node from the old direction to the new one (steps of at most 10
//! degrees), so the blade turns on the spot; smaller turns go straight
//! on. Closed outlines are cut on past their start by `overcut` so the
//! first and last cut meet. Each outline is cut `passes` times. With
//! `weed_frame` a rectangle around everything, `weed_margin` away, is
//! cut last.

use super::{fin, finite, polys, Poly, FLATTEN_TOLERANCE};
use tracedraw_core::geometry::{BezPath, Point, Rect, Vec2};

/// Largest arc step at compensated corners, degrees.
const ARC_STEP_DEG: f64 = 10.0;

/// Points per pen-down command at most.
const POINTS_PER_COMMAND: usize = 64;

/// Which corner of the job is the plotter's 0,0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Origin {
    #[default]
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl Origin {
    pub const ALL: [Origin; 4] = [
        Origin::BottomLeft,
        Origin::BottomRight,
        Origin::TopLeft,
        Origin::TopRight,
    ];
}

/// A cutting plotter's settings (kept as named presets by the app).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Profile {
    pub origin: Origin,
    /// Plotter units per millimetre (40: 0.025 mm).
    pub units_per_mm: f64,
    /// Pen (knife) up and down commands.
    pub pen_up: String,
    pub pen_down: String,
    /// Sent before and after the job, as typed.
    pub init: String,
    pub end: String,
    /// Speed (sent as `VS`), 0: not sent.
    pub velocity: f64,
    /// Knife force (sent as `FS`), 0: not sent.
    pub force: f64,
    /// Knife offset (mm), 0: no compensation.
    pub knife_offset: f64,
    /// Turns sharper than this get a compensation arc (degrees).
    pub corner_limit: f64,
    /// Closed outlines are cut on past their start by this (mm).
    pub overcut: f64,
    pub mirror: bool,
    pub rotate: bool,
    /// Times each outline is cut (1 to 10).
    pub passes: u32,
    /// Cut a frame around the job last.
    pub weed_frame: bool,
    pub weed_margin: f64,
    /// A line break after each command.
    pub line_breaks: bool,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            origin: Origin::BottomLeft,
            units_per_mm: 40.0,
            pen_up: "PU".into(),
            pen_down: "PD".into(),
            init: "IN;".into(),
            end: "PU0,0;".into(),
            velocity: 0.0,
            force: 0.0,
            knife_offset: 0.25,
            corner_limit: 15.0,
            overcut: 1.0,
            mirror: false,
            rotate: false,
            passes: 1,
            weed_frame: false,
            weed_margin: 5.0,
            line_breaks: false,
        }
    }
}

/// The most passes.
pub const MAX_PASSES: u32 = 10;

/// A plotter job.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Plot {
    /// The commands to send.
    pub text: String,
    /// Outlines cut (each pass counted once).
    pub outlines: usize,
    /// Length cut, all passes, overcut and compensation arcs left out (mm).
    pub cut_length: f64,
    /// Travel with the knife up, all passes, from 0,0 (mm).
    pub travel: f64,
    /// Width and height of the job along the plotter's axes (mm).
    pub width: f64,
    pub height: f64,
}

/// The path the knife holder follows so the tip cuts `pts`.
pub fn knife_path(
    pts: &[Point],
    closed: bool,
    offset: f64,
    corner_limit_deg: f64,
    overcut: f64,
) -> Vec<Point> {
    let mut p: Vec<Point> = Vec::with_capacity(pts.len() + 2);
    for q in pts.iter().copied().filter(|q| finite(*q)) {
        if p.last().is_none_or(|l| (*l - q).hypot() > 1e-9) {
            p.push(q);
        }
    }
    if closed && p.len() > 2 {
        if let (Some(a), Some(b)) = (p.first(), p.last()) {
            if (*a - *b).hypot() <= 1e-9 {
                p.pop();
            }
        }
    }
    if p.len() < 2 {
        return p;
    }
    if closed {
        let n = p.len();
        let mut ext = p.clone();
        ext.push(p[0]);
        let mut left = fin(overcut, 0.0).max(0.0);
        let mut i = 0;
        while left > 1e-9 && i < n {
            let a = p[i];
            let b = p[(i + 1) % n];
            let l = (b - a).hypot();
            if l >= left {
                ext.push(a.lerp(b, left / l));
                left = 0.0;
            } else {
                ext.push(b);
                left -= l;
            }
            i += 1;
        }
        p = ext;
    }
    let d = fin(offset, 0.0);
    if d <= 0.0 {
        return p;
    }
    let dirs: Vec<Vec2> = p
        .windows(2)
        .map(|w| {
            let v = w[1] - w[0];
            v / v.hypot()
        })
        .collect();
    let limit = fin(corner_limit_deg, 0.0).max(0.0).to_radians();
    let step = ARC_STEP_DEG.to_radians();
    let mut out = Vec::with_capacity(p.len() * 2);
    out.push(p[0] + dirs[0] * d);
    for (i, u) in dirs.iter().enumerate() {
        let node = p[i + 1];
        out.push(node + *u * d);
        if let Some(v) = dirs.get(i + 1) {
            let turn = u.cross(*v).atan2(u.dot(*v));
            if turn.abs() > limit {
                let steps = (turn.abs() / step).ceil().max(1.0) as usize;
                for k in 1..=steps {
                    let a = turn * k as f64 / steps as f64;
                    let (s, c) = a.sin_cos();
                    let w = Vec2::new(u.x * c - u.y * s, u.x * s + u.y * c);
                    out.push(node + w * d);
                }
            }
        }
    }
    out
}

fn poly_length(pts: &[Point], closed: bool) -> f64 {
    let open: f64 = pts.windows(2).map(|w| (w[1] - w[0]).hypot()).sum();
    match (closed, pts.first(), pts.last()) {
        (true, Some(a), Some(b)) => open + (*a - *b).hypot(),
        _ => open,
    }
}

/// Number formatting for HPGL parameters: whole numbers without a point.
fn num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.3}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// One outline of a job as the plotter cuts it: millimetres, turned and
/// mirrored, y up, before the job is moved to the plotter's origin.
#[derive(Debug, Clone, PartialEq)]
pub struct JobCut {
    pub pts: Vec<Point>,
    pub closed: bool,
}

/// A job laid out for the plotter: its outlines in cut order (the weeding
/// frame last), turned and mirrored; the box of the job (with the frame);
/// and the corner of that box that is the plotter's 0,0.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    pub cuts: Vec<JobCut>,
    /// The weeding frame, when the profile asks for one.
    pub frame: Option<Rect>,
    /// The job's box; `None` when there is nothing to cut.
    pub bbox: Option<Rect>,
    pub corner: Origin,
    /// The corner of `bbox` that is the plotter's 0,0.
    pub origin: Point,
}

impl Layout {
    /// Where `q` lands on the plotter (mm from its 0,0, both axes counting
    /// into the job).
    pub fn place(&self, q: Point) -> Point {
        let Some(b) = self.bbox else { return q };
        let (x, y) = match self.corner {
            Origin::BottomLeft => (q.x - b.x0, q.y - b.y0),
            Origin::BottomRight => (b.x1 - q.x, q.y - b.y0),
            Origin::TopLeft => (q.x - b.x0, b.y1 - q.y),
            Origin::TopRight => (b.x1 - q.x, b.y1 - q.y),
        };
        Point::new(x, y)
    }
}

/// Lay out `cuts` (page-space outlines in cut order) for a plotter set up
/// as `profile`: flattened, turned by 90 degrees counter-clockwise with
/// `rotate`, mirrored left to right with `mirror`, the weeding frame added.
pub fn layout(cuts: &[BezPath], profile: &Profile) -> Layout {
    let mut list: Vec<JobCut> = cuts
        .iter()
        .flat_map(|c| polys(c, FLATTEN_TOLERANCE))
        .map(|poly: Poly| {
            let pts = poly
                .pts
                .into_iter()
                .map(|q| {
                    let mut q = if profile.rotate {
                        Point::new(-q.y, q.x)
                    } else {
                        q
                    };
                    if profile.mirror {
                        q.x = -q.x;
                    }
                    q
                })
                .collect();
            JobCut {
                pts,
                closed: poly.closed,
            }
        })
        .collect();
    let all: Vec<Point> = list.iter().flat_map(|p| p.pts.iter().copied()).collect();
    let Some(mut bbox) = super::poly_bounds(&all) else {
        return Layout {
            corner: profile.origin,
            ..Default::default()
        };
    };
    let mut frame = None;
    if profile.weed_frame {
        let m = fin(profile.weed_margin, 0.0).max(0.0);
        bbox = bbox.inflate(m, m);
        frame = Some(bbox);
        list.push(JobCut {
            pts: vec![
                Point::new(bbox.x0, bbox.y0),
                Point::new(bbox.x1, bbox.y0),
                Point::new(bbox.x1, bbox.y1),
                Point::new(bbox.x0, bbox.y1),
            ],
            closed: true,
        });
    }
    let origin = match profile.origin {
        Origin::BottomLeft => Point::new(bbox.x0, bbox.y0),
        Origin::BottomRight => Point::new(bbox.x1, bbox.y0),
        Origin::TopLeft => Point::new(bbox.x0, bbox.y1),
        Origin::TopRight => Point::new(bbox.x1, bbox.y1),
    };
    Layout {
        cuts: list,
        frame,
        bbox: Some(bbox),
        corner: profile.origin,
        origin,
    }
}

/// The job for `cuts` (page-space outlines in cut order) on a plotter set
/// up as `profile`.
pub fn plot(cuts: &[BezPath], profile: &Profile) -> Plot {
    plot_layout(&layout(cuts, profile), profile)
}

/// The job for a laid-out set of cuts (see [`layout`]).
pub fn plot_layout(job: &Layout, profile: &Profile) -> Plot {
    let upm = {
        let u = fin(profile.units_per_mm, 40.0);
        if u > 0.0 {
            u.min(100_000.0)
        } else {
            40.0
        }
    };
    let passes = profile.passes.clamp(1, MAX_PASSES) as usize;
    let Some(bbox) = job.bbox else {
        return Plot {
            text: format!("{}{}", profile.init, profile.end),
            ..Default::default()
        };
    };
    let place = |q: Point| job.place(q);
    let nl = if profile.line_breaks { "\n" } else { "" };
    let mut text = String::new();
    text.push_str(&profile.init);
    text.push_str(nl);
    if fin(profile.velocity, 0.0) > 0.0 {
        text.push_str(&format!("VS{};{nl}", num(profile.velocity)));
    }
    if fin(profile.force, 0.0) > 0.0 {
        text.push_str(&format!("FS{};{nl}", num(profile.force)));
    }
    let unit = |q: Point| -> (i64, i64) {
        (
            (q.x * upm).round().clamp(-1e15, 1e15) as i64,
            (q.y * upm).round().clamp(-1e15, 1e15) as i64,
        )
    };
    let mut plot = Plot {
        width: bbox.width(),
        height: bbox.height(),
        ..Default::default()
    };
    let mut at = Point::ZERO;
    for _ in 0..passes {
        for poly in &job.cuts {
            let placed: Vec<Point> = poly.pts.iter().map(|q| place(*q)).collect();
            let path = knife_path(
                &placed,
                poly.closed,
                profile.knife_offset,
                profile.corner_limit,
                profile.overcut,
            );
            let mut units: Vec<(i64, i64)> = Vec::with_capacity(path.len());
            for q in &path {
                let u = unit(*q);
                if units.last() != Some(&u) {
                    units.push(u);
                }
            }
            let Some(first) = path.first() else { continue };
            let Some(&(x0, y0)) = units.first() else {
                continue;
            };
            plot.travel += (*first - at).hypot();
            plot.cut_length += poly_length(&placed, poly.closed);
            plot.outlines += 1;
            at = path.last().copied().unwrap_or(*first);
            text.push_str(&format!("{}{x0},{y0};{nl}", profile.pen_up));
            for chunk in units[1..].chunks(POINTS_PER_COMMAND) {
                let coords: Vec<String> = chunk.iter().map(|(x, y)| format!("{x},{y}")).collect();
                text.push_str(&format!("{}{};{nl}", profile.pen_down, coords.join(",")));
            }
        }
    }
    plot.outlines /= passes;
    text.push_str(&profile.end);
    text.push_str(nl);
    plot.text = text;
    plot
}

/// A built-in machine: the command set of a family of cutters, by what
/// it sends (none is named after a maker). Choosing one sets the
/// commands, the unit and the line breaks of a profile and keeps its knife
/// settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum Machine {
    /// `IN;` at the start, `PU`/`PD` with absolute coordinates in
    /// 0.025 mm units, back to 0,0 at the end.
    #[default]
    Hpgl,
    /// The same with absolute plotting and pen 1 selected at the start
    /// (`IN;PA;SP1;`) and the pen put away at the end (`PU0,0;SP0;`).
    HpglPenSelect,
    /// The generic commands, one command per line, for cutters and
    /// spoolers that read a line at a time.
    HpglLines,
}

impl Machine {
    pub const ALL: [Machine; 3] = [Machine::Hpgl, Machine::HpglPenSelect, Machine::HpglLines];

    /// `profile` with this machine's commands, unit and line breaks.
    pub fn apply(self, profile: &Profile) -> Profile {
        let base = Profile {
            units_per_mm: 40.0,
            pen_up: "PU".into(),
            pen_down: "PD".into(),
            init: "IN;".into(),
            end: "PU0,0;".into(),
            line_breaks: false,
            ..profile.clone()
        };
        match self {
            Machine::Hpgl => base,
            Machine::HpglPenSelect => Profile {
                init: "IN;PA;SP1;".into(),
                end: "PU0,0;SP0;".into(),
                ..base
            },
            Machine::HpglLines => Profile {
                line_breaks: true,
                ..base
            },
        }
    }

    /// The built-in machine whose commands `profile` has, if any.
    pub fn of(profile: &Profile) -> Option<Machine> {
        Machine::ALL
            .into_iter()
            .find(|m| m.apply(profile) == *profile)
    }
}

/// The test cut: a square of side `size` (mm) with a triangle inside,
/// the triangle first so the square does not move while it is cut. The
/// square's bottom-left corner is at 0,0; the triangle's base runs a
/// quarter of the side in from the square's, its apex a quarter below
/// the top. A size that is not a positive number gives 20 mm.
pub fn test_cut(size: f64) -> Vec<BezPath> {
    let s = if size.is_finite() && size > 0.0 {
        size.min(1000.0)
    } else {
        20.0
    };
    let mut triangle = BezPath::new();
    triangle.move_to((0.25 * s, 0.25 * s));
    triangle.line_to((0.75 * s, 0.25 * s));
    triangle.line_to((0.5 * s, 0.75 * s));
    triangle.close_path();
    vec![triangle, super::rect_path(Rect::new(0.0, 0.0, s, s))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cut::rect_path;
    use tracedraw_core::geometry::Shape as _;

    #[test]
    fn machines_set_the_commands_and_keep_the_knife() {
        let mine = Profile {
            knife_offset: 0.45,
            passes: 2,
            ..Default::default()
        };
        assert_eq!(Machine::of(&mine), Some(Machine::Hpgl));
        let pen = Machine::HpglPenSelect.apply(&mine);
        assert_eq!(pen.init, "IN;PA;SP1;");
        assert_eq!(pen.end, "PU0,0;SP0;");
        assert_eq!(pen.knife_offset, 0.45);
        assert_eq!(pen.passes, 2);
        assert_eq!(Machine::of(&pen), Some(Machine::HpglPenSelect));
        let lines = Machine::HpglLines.apply(&pen);
        assert!(lines.line_breaks);
        assert_eq!(lines.init, "IN;");
        let text = plot(
            &test_cut(10.0),
            &Profile {
                knife_offset: 0.0,
                overcut: 0.0,
                ..lines
            },
        )
        .text;
        assert!(text.lines().count() > 3, "{text}");
        let odd = Profile {
            init: "XX;".into(),
            ..Default::default()
        };
        assert_eq!(Machine::of(&odd), None);
    }

    #[test]
    fn the_test_cut_is_a_square_with_a_triangle_cut_first() {
        let cuts = test_cut(20.0);
        assert_eq!(cuts.len(), 2);
        let b0 = cuts[0].bounding_box();
        let b1 = cuts[1].bounding_box();
        assert_eq!(b1, Rect::new(0.0, 0.0, 20.0, 20.0));
        assert!(b0.x0 > b1.x0 && b0.x1 < b1.x1 && b0.y0 > b1.y0 && b0.y1 < b1.y1);
        let p = plot(&cuts, &Profile::default());
        assert_eq!(p.outlines, 2);
        assert!((p.width - 20.0).abs() < 1e-9 && (p.height - 20.0).abs() < 1e-9);
        // Nonsense sizes give the default square.
        assert_eq!(test_cut(f64::NAN)[1].bounding_box().width(), 20.0);
        assert_eq!(test_cut(-3.0)[1].bounding_box().width(), 20.0);
    }

    fn bare() -> Profile {
        Profile {
            knife_offset: 0.0,
            overcut: 0.0,
            ..Default::default()
        }
    }

    #[test]
    fn a_square_in_plotter_units() {
        let sq = rect_path(Rect::new(100.0, 100.0, 110.0, 110.0));
        let p = plot(std::slice::from_ref(&sq), &bare());
        assert_eq!(p.text, "IN;PU0,0;PD400,0,400,400,0,400,0,0;PU0,0;");
        assert_eq!(p.outlines, 1);
        assert!((p.cut_length - 40.0).abs() < 1e-9);
        assert!((p.width - 10.0).abs() < 1e-9);
        // Overcut 1 mm runs on along the first side.
        let p = plot(
            std::slice::from_ref(&sq),
            &Profile {
                overcut: 1.0,
                ..bare()
            },
        );
        assert!(
            p.text.contains("PD400,0,400,400,0,400,0,0,40,0;"),
            "{}",
            p.text
        );
        // Speed, force, passes and line breaks.
        let p = plot(
            &[sq],
            &Profile {
                velocity: 20.0,
                force: 80.5,
                passes: 2,
                line_breaks: true,
                ..bare()
            },
        );
        assert!(
            p.text.starts_with("IN;\nVS20;\nFS80.5;\nPU0,0;\n"),
            "{}",
            p.text
        );
        assert_eq!(p.text.matches("PU0,0;").count(), 3);
        assert_eq!(p.outlines, 1);
        assert!((p.cut_length - 80.0).abs() < 1e-9);
    }

    #[test]
    fn origin_mirror_and_rotation_place_the_job() {
        let mut path = rect_path(Rect::new(0.0, 0.0, 20.0, 10.0));
        path.move_to((20.0, 10.0));
        path.line_to((25.0, 10.0));
        let tr = plot(
            &[path.clone()],
            &Profile {
                origin: Origin::TopRight,
                ..bare()
            },
        );
        // The open line's end (25, 10) is the top-right corner: 0,0.
        assert!(tr.text.contains("PU200,0;PD0,0;"), "{}", tr.text);
        let rot = plot(
            &[path.clone()],
            &Profile {
                rotate: true,
                ..bare()
            },
        );
        assert!((rot.width - 10.0).abs() < 1e-9 && (rot.height - 25.0).abs() < 1e-9);
        let mir = plot(
            &[path],
            &Profile {
                mirror: true,
                ..bare()
            },
        );
        // Mirrored, the line runs from x = 5 to x = 0.
        assert!(mir.text.contains("PU200,400;PD0,400;"), "{}", mir.text);
        for p in [&tr, &rot, &mir] {
            for n in p.text.split(|c: char| !c.is_ascii_digit() && c != '-') {
                if let Ok(v) = n.parse::<i64>() {
                    assert!(v >= 0, "{}", p.text);
                }
            }
        }
    }

    #[test]
    fn knife_offset_adds_arcs_at_sharp_corners_only() {
        let line = knife_path(
            &[Point::new(0.0, 0.0), Point::new(10.0, 0.0)],
            false,
            0.25,
            15.0,
            1.0,
        );
        assert_eq!(line, vec![Point::new(0.25, 0.0), Point::new(10.25, 0.0)]);
        let sq = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];
        let k = knife_path(&sq, true, 0.25, 15.0, 0.0);
        // Start, then per side its end and 9 arc points at three corners.
        assert_eq!(k.len(), 1 + 4 + 3 * 9);
        assert_eq!(k[0], Point::new(0.25, 0.0));
        let arc_end = k[1 + 9];
        assert!(
            (arc_end - Point::new(10.0, 0.25)).hypot() < 1e-9,
            "{arc_end:?}"
        );
        for q in &k[1..=10] {
            assert!(((*q - Point::new(10.0, 0.0)).hypot() - 0.25).abs() < 1e-9);
        }
        // With overcut the start corner is turned too and the cut runs on.
        let k = knife_path(&sq, true, 0.25, 15.0, 1.0);
        assert_eq!(k.len(), 1 + 5 + 4 * 9);
        assert_eq!(k.last().copied(), Some(Point::new(1.25, 0.0)));
        // A shallow turn goes straight on.
        let shallow = knife_path(
            &[
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(20.0, 1.0),
            ],
            false,
            0.25,
            15.0,
            0.0,
        );
        assert_eq!(shallow.len(), 3);
    }

    #[test]
    fn the_layout_marks_the_origin_corner_and_the_frame() {
        let sq = rect_path(Rect::new(10.0, 20.0, 30.0, 30.0));
        let l = layout(
            std::slice::from_ref(&sq),
            &Profile {
                origin: Origin::TopRight,
                weed_frame: true,
                weed_margin: 5.0,
                ..bare()
            },
        );
        assert_eq!(l.bbox, Some(Rect::new(5.0, 15.0, 35.0, 35.0)));
        assert_eq!(l.frame, l.bbox);
        assert_eq!(l.origin, Point::new(35.0, 35.0));
        assert_eq!(l.cuts.len(), 2);
        assert_eq!(l.place(l.origin), Point::ZERO);
        assert_eq!(l.place(Point::new(5.0, 15.0)), Point::new(30.0, 20.0));
        // Turned: (x, y) becomes (-y, x).
        let r = layout(
            std::slice::from_ref(&sq),
            &Profile {
                rotate: true,
                ..bare()
            },
        );
        assert_eq!(r.bbox, Some(Rect::new(-30.0, 10.0, -20.0, 30.0)));
        assert_eq!(r.frame, None);
        let empty = layout(&[], &bare());
        assert!(empty.bbox.is_none() && empty.cuts.is_empty());
        assert_eq!(empty.place(Point::new(1.0, 2.0)), Point::new(1.0, 2.0));
    }

    #[test]
    fn weed_frame_is_cut_last() {
        let sq = rect_path(Rect::new(0.0, 0.0, 10.0, 10.0));
        let p = plot(
            &[sq],
            &Profile {
                weed_frame: true,
                weed_margin: 5.0,
                ..bare()
            },
        );
        assert_eq!(p.outlines, 2);
        assert!(
            p.text.ends_with("PU0,0;PD800,0,800,800,0,800,0,0;PU0,0;"),
            "{}",
            p.text
        );
        assert!((p.width - 20.0).abs() < 1e-9);
    }

    #[test]
    fn nothing_or_non_finite_input_does_not_panic() {
        let p = plot(&[], &Profile::default());
        assert_eq!(p.text, "IN;PU0,0;");
        let mut bad = BezPath::new();
        bad.move_to((f64::NAN, 1.0));
        bad.line_to((1.0, f64::INFINITY));
        let wild = Profile {
            units_per_mm: f64::NAN,
            knife_offset: f64::INFINITY,
            corner_limit: f64::NAN,
            overcut: f64::NEG_INFINITY,
            passes: 0,
            velocity: f64::NAN,
            ..Default::default()
        };
        let p = plot(&[bad, rect_path(Rect::new(0.0, 0.0, 1.0, 1.0))], &wild);
        assert_eq!(p.outlines, 1);
        assert!(!p.text.contains("NaN") && !p.text.contains("inf"));
        assert!(knife_path(&[], true, 1.0, 15.0, 1.0).is_empty());
        assert_eq!(
            knife_path(&[Point::new(1.0, 1.0); 3], true, 1.0, 15.0, 1.0).len(),
            1
        );
    }
}
