//! Classic Illustrator file writer: the PostScript-based `.ai` of
//! version 8, for programs that read only those files (#1728).
//!
//! The layout follows the public "Adobe Illustrator File Format
//! Specification" (see `CLEANROOM.md`): header comments with the
//! bounding box and the `%AI5_`/`%AI8_` markers, an empty prolog and
//! setup (the file names no procedure set, as the format allows), then
//! the artwork in the format's own operators: layers (`Lb`, `Ln`,
//! `LB`), groups (`u`, `U`), clipping groups (`q`, `W`, `Q`), compound
//! paths (`*u`, `*U`), path construction (`m`, `L`, `C`), painting
//! (`f`, `F`, `s`, `S`, `b`, `B`, `n`, `N`), process colours (`k`,
//! `K`) and outline settings (`w`, `j`, `J`, `M`, `d`).
//!
//! What the format of that version cannot hold is turned into what it
//! can: text as filled outlines, fountain fills as bands of solid
//! colour clipped by the object, pattern, texture and mesh fills as
//! their average colour, live effects as the objects they draw. Every
//! colour is written as a process colour (RGB through the built-in
//! conversion). Pictures and bitmap effects are left out: the vector
//! object is written without them. One artboard per file: the page,
//! whose box is the file's bounding box.

use std::fmt::Write as _;
use tracedraw_core::{
    document::{Shape, ShapeKind},
    geometry::{Affine, BezPath, PathEl, Point, Shape as _},
    Color, Document, Fill, Fountain, FountainKind, LineCap, LineJoin,
};

const MM_PT: f64 = 72.0 / 25.4;

/// How many bands of solid colour a fountain fill is cut into.
const FOUNTAIN_BANDS: usize = 64;

/// More subpaths than this are written as they are, without turning
/// them so a non-zero fill matches the even-odd one (the check compares
/// every pair).
const MAX_ORIENTED_SUBPATHS: usize = 400;

fn f(v: f64) -> String {
    if !v.is_finite() {
        return "0".into();
    }
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.to_string()
    }
}

/// A PostScript string literal: brackets and backslashes escaped,
/// bytes outside printable ASCII as octal escapes.
fn ps_string(s: &str) -> String {
    let mut out = String::from("(");
    for b in s.bytes() {
        match b {
            b'(' | b')' | b'\\' => {
                out.push('\\');
                out.push(b as char);
            }
            0x20..=0x7e => out.push(b as char),
            _ => {
                let _ = write!(out, "\\{b:03o}");
            }
        }
    }
    out.push(')');
    out
}

/// The process colour (cyan, magenta, yellow, black in 0..1) a colour
/// is written as.
fn cmyk(c: Color) -> [f64; 4] {
    let q = |v: f32| {
        if v.is_finite() {
            f64::from(v).clamp(0.0, 1.0)
        } else {
            0.0
        }
    };
    match c.process() {
        Color::Cmyk { c, m, y, k } => [q(c), q(m), q(y), q(k)],
        Color::Registration => [1.0, 1.0, 1.0, 1.0],
        Color::Gray { v } => [0.0, 0.0, 0.0, 1.0 - q(v)],
        other => {
            let [r, g, b] = other.to_rgb_f32().map(q);
            let k = 1.0 - r.max(g).max(b);
            if k >= 1.0 - 1e-6 {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [
                    (1.0 - r - k) / (1.0 - k),
                    (1.0 - g - k) / (1.0 - k),
                    (1.0 - b - k) / (1.0 - k),
                    k,
                ]
            }
        }
    }
}

/// The operands and operator that set the fill (`k`) or outline (`K`)
/// colour.
fn color_op(c: Color, op: &str) -> String {
    let [c, m, y, k] = cmyk(c);
    format!("{} {} {} {} {op}", f(c), f(m), f(y), f(k))
}

/// The subpaths of `path`, each starting at its move; a move with
/// nothing after it is dropped.
fn subpaths(path: &BezPath) -> Vec<BezPath> {
    let mut out: Vec<BezPath> = Vec::new();
    let mut cur = BezPath::new();
    for el in path.elements() {
        if let PathEl::MoveTo(_) = el {
            if cur.elements().len() > 1 {
                out.push(std::mem::take(&mut cur));
            } else {
                cur = BezPath::new();
            }
        } else if cur.elements().is_empty() {
            // A segment with no move before it: start where it is.
            continue;
        }
        cur.push(*el);
    }
    if cur.elements().len() > 1 {
        out.push(cur);
    }
    out
}

/// The subpaths of an even-odd filled object turned so that each runs
/// the other way from the ones around it. The format fills a compound
/// path by its winding, so nested outlines (a ring, the hole in a
/// letter) must alternate their direction to stay holes. Outlines that
/// cross themselves or each other may still fill differently.
fn orient_for_nonzero(subs: Vec<BezPath>) -> Vec<BezPath> {
    if subs.len() < 2 || subs.len() > MAX_ORIENTED_SUBPATHS {
        return subs;
    }
    let closed: Vec<BezPath> = subs
        .iter()
        .map(|sp| {
            let mut c = sp.clone();
            if !matches!(c.elements().last(), Some(PathEl::ClosePath)) {
                c.close_path();
            }
            c
        })
        .collect();
    let start = |sp: &BezPath| match sp.elements().first() {
        Some(PathEl::MoveTo(p)) => Some(*p),
        _ => None,
    };
    subs.into_iter()
        .enumerate()
        .map(|(i, sp)| {
            let Some(p) = start(&sp) else {
                return sp;
            };
            let depth = closed
                .iter()
                .enumerate()
                .filter(|(j, c)| *j != i && c.winding(p) != 0)
                .count();
            let area = closed[i].area();
            let want_positive = depth % 2 == 0;
            if area != 0.0 && (area > 0.0) != want_positive {
                sp.reverse_subpaths()
            } else {
                sp
            }
        })
        .collect()
}

/// A closed polygon through `pts` (mm).
fn polygon(pts: &[Point]) -> BezPath {
    let mut p = BezPath::new();
    for (i, q) in pts.iter().enumerate() {
        if i == 0 {
            p.move_to(*q);
        } else {
            p.line_to(*q);
        }
    }
    p.close_path();
    p
}

/// A circle of four cubic arcs (mm).
fn circle(c: Point, r: f64) -> BezPath {
    let k = 0.552_284_749_830_793_6 * r;
    let mut p = BezPath::new();
    p.move_to((c.x + r, c.y));
    p.curve_to((c.x + r, c.y + k), (c.x + k, c.y + r), (c.x, c.y + r));
    p.curve_to((c.x - k, c.y + r), (c.x - r, c.y + k), (c.x - r, c.y));
    p.curve_to((c.x - r, c.y - k), (c.x - k, c.y - r), (c.x, c.y - r));
    p.curve_to((c.x + k, c.y - r), (c.x + r, c.y - k), (c.x + r, c.y));
    p.close_path();
    p
}

/// A classic Illustrator (version 8, PostScript-based) file of page
/// `page_index` of `doc`; empty when there is no such page.
pub fn page_to_ai8(doc: &Document, page_index: usize) -> String {
    let Some(page) = doc.pages.get(page_index) else {
        return String::new();
    };
    let w = page.size.width * MM_PT;
    let h = page.size.height * MM_PT;
    let layers: Vec<_> = doc
        .layers_drawn_for_page(page.id)
        .unwrap_or_default()
        .into_iter()
        .filter(|l| l.visible && l.printable)
        .collect();
    let mut out = String::new();
    let _ = writeln!(out, "%!PS-Adobe-3.0");
    let _ = writeln!(out, "%%Creator: TraceDraw");
    let _ = writeln!(out, "%%AI8_CreatorVersion: 8.0");
    let _ = writeln!(
        out,
        "%%Title: {}",
        ps_string(&doc.title.replace(['\n', '\r'], " "))
    );
    let _ = writeln!(
        out,
        "%%BoundingBox: 0 0 {} {}",
        w.ceil() as i64,
        h.ceil() as i64
    );
    let _ = writeln!(out, "%%HiResBoundingBox: 0 0 {} {}", f(w), f(h));
    let _ = writeln!(out, "%%DocumentProcessColors: Cyan Magenta Yellow Black");
    let _ = writeln!(out, "%AI5_FileFormat 4.0");
    let _ = writeln!(out, "%AI3_ColorUsage: Color");
    let _ = writeln!(
        out,
        "%AI3_TemplateBox: {} {} {} {}",
        f(w / 2.0),
        f(h / 2.0),
        f(w / 2.0),
        f(h / 2.0)
    );
    let _ = writeln!(out, "%AI3_TileBox: 0 0 {} {}", f(w), f(h));
    let _ = writeln!(out, "%AI3_DocumentPreview: None");
    let _ = writeln!(out, "%AI5_ArtSize: {} {}", f(w), f(h));
    let _ = writeln!(out, "%AI5_NumLayers: {}", layers.len().max(1));
    let _ = writeln!(out, "%%EndComments");
    let _ = writeln!(out, "%%BeginProlog");
    let _ = writeln!(out, "%%EndProlog");
    let _ = writeln!(out, "%%BeginSetup");
    let _ = writeln!(out, "%%EndSetup");
    let mut wr = Writer {
        out: &mut out,
        symbols: &doc.symbols,
    };
    let names: Vec<String> = if layers.is_empty() {
        vec!["Layer 1".into()]
    } else {
        layers.iter().map(|l| l.name.clone()).collect()
    };
    for (i, name) in names.iter().enumerate() {
        wr.line("%AI5_BeginLayer");
        // Visible, preview, enabled, printing, not dimmed, no masks,
        // colour index and the layer's colour.
        wr.line("1 1 1 1 0 0 0 79 128 255 Lb");
        wr.line(&format!("{} Ln", ps_string(name)));
        if i == 0 {
            if let Some(bg) = &page.background {
                let mut s = Shape::new(
                    tracedraw_core::ShapeId(0),
                    ShapeKind::Rect {
                        rect: page.rect(),
                        radius: 0.0,
                        corners: None,
                    },
                );
                s.fill = bg.clone();
                s.stroke = None;
                wr.shape(&s, Affine::IDENTITY);
            }
        }
        if let Some(layer) = layers.get(i) {
            for s in &layer.shapes {
                wr.shape(s, Affine::IDENTITY);
            }
        }
        wr.line("LB");
        wr.line("%AI5_EndLayer--");
    }
    let _ = writeln!(out, "%%PageTrailer");
    let _ = writeln!(out, "%%Trailer");
    let _ = writeln!(out, "%%EOF");
    out
}

struct Writer<'a> {
    out: &'a mut String,
    symbols: &'a [tracedraw_core::Symbol],
}

impl Writer<'_> {
    fn line(&mut self, s: &str) {
        self.out.push_str(s);
        self.out.push('\n');
    }

    /// The construction of one subpath; true when it is closed.
    fn subpath_ops(&mut self, path: &BezPath) -> bool {
        let mut cur: Option<Point> = None;
        let mut closed = false;
        let pt = |p: Point| (f(p.x * MM_PT), f(p.y * MM_PT));
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    let (x, y) = pt(*p);
                    let _ = writeln!(self.out, "{x} {y} m");
                    cur = Some(*p);
                }
                PathEl::LineTo(p) => {
                    let (x, y) = pt(*p);
                    let _ = writeln!(self.out, "{x} {y} L");
                    cur = Some(*p);
                }
                PathEl::QuadTo(c, p) => {
                    // The cubic that draws the same curve.
                    let s = cur.unwrap_or(*c);
                    let c1 = s + (*c - s) * (2.0 / 3.0);
                    let c2 = *p + (*c - *p) * (2.0 / 3.0);
                    let ((ax, ay), (bx, by), (x, y)) = (pt(c1), pt(c2), pt(*p));
                    let _ = writeln!(self.out, "{ax} {ay} {bx} {by} {x} {y} C");
                    cur = Some(*p);
                }
                PathEl::CurveTo(a, b, p) => {
                    let ((ax, ay), (bx, by), (x, y)) = (pt(*a), pt(*b), pt(*p));
                    let _ = writeln!(self.out, "{ax} {ay} {bx} {by} {x} {y} C");
                    cur = Some(*p);
                }
                PathEl::ClosePath => closed = true,
            }
        }
        closed
    }

    /// One path object: each subpath built and painted, a compound path
    /// when there are several. `clip` marks it as the clipping path of
    /// the group it starts.
    fn path_object(&mut self, subs: &[BezPath], fill: bool, stroke: bool, clip: bool) {
        if subs.is_empty() {
            return;
        }
        let compound = subs.len() > 1;
        if compound {
            self.line("*u");
        }
        for sp in subs {
            let closed = self.subpath_ops(sp);
            if clip {
                self.line("W");
            }
            let op = match (fill, stroke) {
                (true, true) => "b",
                (true, false) => "f",
                (false, true) => "s",
                (false, false) => "n",
            };
            if closed {
                self.line(op);
            } else {
                self.line(&op.to_ascii_uppercase());
            }
        }
        if compound {
            self.line("*U");
        }
    }

    /// A fountain fill as bands of solid colour inside a clipping group
    /// of the object's outline.
    fn fountain(&mut self, subs: &[BezPath], ft: &Fountain, bounds: tracedraw_core::Rect) {
        let ft = ft.to_plain(256);
        self.line("q");
        self.path_object(subs, false, false, true);
        let c = bounds.center();
        let diag = (bounds.width().hypot(bounds.height())).max(1e-3);
        let n = FOUNTAIN_BANDS;
        // Neighbouring bands of the same colour are written as one.
        let mut runs: Vec<(usize, usize, String)> = Vec::new();
        for i in 0..n {
            let col = color_op(ft.color_at((i as f64 + 0.5) / n as f64), "k");
            match runs.last_mut() {
                Some((_, end, last)) if *last == col => *end = i + 1,
                _ => runs.push((i, i + 1, col)),
            }
        }
        match ft.kind {
            FountainKind::Radial => {
                let centre = Point::new(
                    c.x + ft.offset.x * bounds.width() / 2.0,
                    c.y + ft.offset.y * bounds.height() / 2.0,
                );
                let r = bounds.width().max(bounds.height()) / 2.0 * std::f64::consts::SQRT_2;
                // Beyond the last circle: the end colour over everything.
                let outer = color_op(ft.color_at(1.0), "k");
                self.line(&outer);
                let far = diag + centre.distance(c);
                let all = polygon(&[
                    Point::new(c.x - far, c.y - far),
                    Point::new(c.x + far, c.y - far),
                    Point::new(c.x + far, c.y + far),
                    Point::new(c.x - far, c.y + far),
                ]);
                self.path_object(&[all], true, false, false);
                // Discs from the largest in, each to the end of its run.
                for (_, end, col) in runs.iter().rev() {
                    self.line(col);
                    let disc = circle(centre, r * *end as f64 / n as f64);
                    self.path_object(&[disc], true, false, false);
                }
            }
            _ => {
                let a = ft.angle.to_radians();
                let dir = (a.cos(), a.sin());
                let perp = (-dir.1, dir.0);
                let half = (bounds.width() * a.cos().abs() + bounds.height() * a.sin().abs()) / 2.0;
                let at = |s: f64, e: f64| {
                    Point::new(c.x + dir.0 * s + perp.0 * e, c.y + dir.1 * s + perp.1 * e)
                };
                let last = runs.len().saturating_sub(1);
                for (k, (start, end, col)) in runs.iter().enumerate() {
                    let mut s0 = -half + 2.0 * half * *start as f64 / n as f64;
                    let mut s1 = -half + 2.0 * half * *end as f64 / n as f64;
                    // The first and last bands reach past the object.
                    if k == 0 {
                        s0 -= diag;
                    }
                    if k == last {
                        s1 += diag;
                    }
                    self.line(col);
                    let band = polygon(&[at(s0, -diag), at(s1, -diag), at(s1, diag), at(s0, diag)]);
                    self.path_object(&[band], true, false, false);
                }
            }
        }
        self.line("Q");
    }

    fn shape(&mut self, shape: &Shape, parent: Affine) {
        if !shape.visible {
            return;
        }
        if !shape.effects.is_empty() {
            let ev = tracedraw_core::live::evaluate(shape);
            let mut parts: Vec<Shape> = Vec::new();
            parts.extend(ev.below.iter().cloned());
            parts.push(ev.main.clone());
            parts.extend(ev.above.iter().cloned());
            let group = parts.len() > 1;
            if group {
                self.line("u");
            }
            for mut p in parts {
                p.effects.clear();
                self.shape(&p, parent);
            }
            if group {
                self.line("U");
            }
            return;
        }
        // An outline set behind the fill: the outline first, then the
        // fill over it, as the drawing window draws it.
        if shape.stroke.as_ref().is_some_and(|st| st.behind_fill)
            && !matches!(shape.fill, Fill::None)
        {
            let mut outline = shape.clone();
            outline.fill = Fill::None;
            if let Some(st) = outline.stroke.as_mut() {
                st.behind_fill = false;
            }
            let mut fill = shape.clone();
            fill.stroke = None;
            self.line("u");
            self.shape(&outline, parent);
            self.shape(&fill, parent);
            self.line("U");
            return;
        }
        let transform = parent * shape.transform;
        match &shape.kind {
            ShapeKind::Group { children } => {
                self.line("u");
                for c in children {
                    self.shape(c, transform);
                }
                self.line("U");
                return;
            }
            ShapeKind::Table(_) | ShapeKind::SymbolInstance { .. } => {
                self.line("u");
                for c in shape.expand(self.symbols) {
                    self.shape(&c, transform);
                }
                self.line("U");
                return;
            }
            ShapeKind::ClipFrame { frame, contents } => {
                self.line("u");
                let mut fill_only = (**frame).clone();
                fill_only.stroke = None;
                self.shape(&fill_only, transform);
                let clip = subpaths(&(transform * frame.page_path()));
                if !clip.is_empty() {
                    self.line("q");
                    self.path_object(&clip, false, false, true);
                    for c in contents {
                        self.shape(c, transform);
                    }
                    self.line("Q");
                }
                let mut outline = (**frame).clone();
                outline.fill = Fill::None;
                self.shape(&outline, transform);
                self.line("U");
                return;
            }
            // Pictures have no place in this version of the format.
            ShapeKind::Bitmap { .. } => return,
            _ => {}
        }
        let path = transform * shape.local_path();
        let subs = subpaths(&path);
        if subs.is_empty() {
            return;
        }
        let bounds = path.bounding_box();
        let filled = !matches!(shape.fill, Fill::None);
        // Text is filled by winding, the rest even-odd.
        let subs = if filled && !matches!(shape.kind, ShapeKind::Text { .. }) {
            orient_for_nonzero(subs)
        } else {
            subs
        };
        let fill_color = match &shape.fill {
            Fill::None => None,
            Fill::Solid(c) => Some(*c),
            Fill::Fountain(ft) => {
                if shape.stroke.is_some() {
                    self.line("u");
                }
                self.fountain(&subs, ft, bounds);
                None
            }
            other => Some(other.preview_color().unwrap_or(Color::Gray { v: 0.5 })),
        };
        let fountain_group = matches!(shape.fill, Fill::Fountain(_)) && shape.stroke.is_some();
        if let Some(c) = fill_color {
            self.line(&color_op(c, "k"));
        }
        if let Some(s) = &shape.stroke {
            let width = if s.width <= tracedraw_core::Stroke::HAIRLINE + 1e-9 {
                0.0
            } else {
                s.width * MM_PT
            };
            self.line(&color_op(s.color, "K"));
            let _ = writeln!(
                self.out,
                "{} w {} j {} J {} M",
                f(width),
                match s.join {
                    LineJoin::Miter => 0,
                    LineJoin::Round => 1,
                    LineJoin::Bevel => 2,
                },
                match s.cap {
                    LineCap::Butt => 0,
                    LineCap::Round => 1,
                    LineCap::Square => 2,
                },
                f(s.miter_ratio().max(1.0))
            );
            let unit = if width > 0.0 { width } else { 1.0 };
            let dash: Vec<String> = s.dash.iter().map(|d| f(d * unit)).collect();
            let _ = writeln!(self.out, "[{}] 0 d", dash.join(" "));
        }
        if fill_color.is_some() || shape.stroke.is_some() {
            self.path_object(&subs, fill_color.is_some(), shape.stroke.is_some(), false);
        }
        if fountain_group {
            self.line("U");
        }
        if let Some(s) = &shape.stroke {
            // Arrowheads: the page path already carries the shape's own
            // transform, so only the parent applies.
            for head in tracedraw_core::arrowhead_paths(&shape.page_path(), s) {
                self.line(&color_op(s.color, "k"));
                let subs = subpaths(&(parent * head));
                self.path_object(&subs, true, false, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Rect;

    fn rect_shape(id: u64, r: Rect) -> Shape {
        Shape::new(
            tracedraw_core::ShapeId(id),
            ShapeKind::Rect {
                rect: r,
                radius: 0.0,
                corners: None,
            },
        )
    }

    /// The operators of the artwork, one per line's last word, after
    /// the header comments.
    fn ops(ai: &str) -> Vec<&str> {
        ai.lines()
            .filter(|l| !l.starts_with('%'))
            .filter_map(|l| l.split_whitespace().last())
            .collect()
    }

    /// A small drawing: a filled and outlined rectangle and a group of a
    /// ring (an ellipse with a hole) and a line.
    fn small_drawing() -> Document {
        let mut doc = Document::default();
        doc.title = "Small (test)".into();
        let layer = doc.pages[0].layers[0].id;
        let mut r = rect_shape(1, Rect::new(10.0, 20.0, 60.0, 40.0));
        r.fill = Fill::Solid(Color::cmyk_pct(100.0, 0.0, 0.0, 0.0));
        r.stroke = Some(tracedraw_core::Stroke::new(Color::rgb8(255, 0, 0), 1.0));
        let mut ring = BezPath::new();
        ring.extend(circle(Point::new(100.0, 100.0), 30.0));
        ring.extend(circle(Point::new(100.0, 100.0), 10.0));
        let mut ring = Shape::new(
            tracedraw_core::ShapeId(2),
            ShapeKind::Path {
                path: ring,
                closed: true,
            },
        );
        ring.fill = Fill::Solid(Color::Gray { v: 0.25 });
        ring.stroke = None;
        let mut line = Shape::new(
            tracedraw_core::ShapeId(3),
            ShapeKind::Path {
                path: {
                    let mut p = BezPath::new();
                    p.move_to((0.0, 0.0));
                    p.line_to((50.0, 50.0));
                    p
                },
                closed: false,
            },
        );
        line.fill = Fill::None;
        line.stroke = Some(tracedraw_core::Stroke::new(Color::BLACK, 0.5));
        let group = Shape::new(
            tracedraw_core::ShapeId(4),
            ShapeKind::Group {
                children: vec![ring, line],
            },
        );
        let l = doc.layer_mut(layer).unwrap();
        l.shapes.push(r);
        l.shapes.push(group);
        doc
    }

    #[test]
    fn classic_ai_starts_with_the_header_the_format_asks_for() {
        let ai = page_to_ai8(&small_drawing(), 0);
        let mut lines = ai.lines();
        assert_eq!(lines.next(), Some("%!PS-Adobe-3.0"));
        assert!(ai.contains("\n%%Creator: TraceDraw\n"));
        assert!(ai.contains("\n%%AI8_CreatorVersion: 8.0\n"));
        assert!(ai.contains("\n%AI5_FileFormat 4.0\n"));
        assert!(ai.contains("\n%%Title: (Small \\(test\\))\n"), "{ai}");
        // A4 portrait: the page is the bounding box.
        assert!(ai.contains("\n%%BoundingBox: 0 0 596 842\n"));
        assert!(ai.contains("\n%%HiResBoundingBox: 0 0 595.276 841.89\n"));
        assert!(ai.contains("\n%AI5_ArtSize: 595.276 841.89\n"));
        // Comments, empty prolog and setup, then the artwork.
        let order = [
            "%%EndComments",
            "%%BeginProlog",
            "%%EndProlog",
            "%%BeginSetup",
            "%%EndSetup",
            "%AI5_BeginLayer",
            "%AI5_EndLayer--",
            "%%Trailer",
            "%%EOF",
        ];
        let at: Vec<usize> = order
            .iter()
            .map(|m| ai.find(&format!("\n{m}\n")).expect(m))
            .collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]), "{at:?}");
        // No PostScript drawing operators: the artwork is in the
        // format's own operators only.
        for ps in [
            "moveto",
            "lineto",
            "fill",
            "stroke",
            "setcmykcolor",
            "gsave",
        ] {
            assert!(!ops(&ai).contains(&ps), "{ps} in {ai}");
        }
    }

    #[test]
    fn classic_ai_groups_paths_and_colours_balance() {
        let ai = page_to_ai8(&small_drawing(), 0);
        let ops = ops(&ai);
        let count = |o: &str| ops.iter().filter(|x| **x == o).count();
        assert_eq!(count("u"), count("U"));
        assert_eq!(count("u"), 1, "{ai}");
        assert_eq!(count("*u"), count("*U"));
        assert_eq!(count("*u"), 1, "the ring is one compound path: {ai}");
        assert_eq!(count("Lb"), 1);
        assert_eq!(count("LB"), 1);
        // Groups nest properly.
        let mut depth = 0i32;
        for o in &ops {
            match *o {
                "u" | "*u" | "q" => depth += 1,
                "U" | "*U" | "Q" => depth -= 1,
                _ => {}
            }
            assert!(depth >= 0);
        }
        assert_eq!(depth, 0);
        // The rectangle: cyan fill, red outline (converted to process
        // colour), closed, filled and stroked.
        assert!(ai.contains("\n1 0 0 0 k\n"), "{ai}");
        assert!(ai.contains("\n0 1 1 0 K\n"), "{ai}");
        assert!(ai.contains(&format!("\n{} w 0 j 0 J", f(MM_PT))), "{ai}");
        assert_eq!(count("b"), 1);
        // The ring: 25 % grey as 75 % black, two closed subpaths filled.
        assert!(ai.contains("\n0 0 0 0.75 k\n"), "{ai}");
        assert_eq!(count("f"), 2);
        // The line: open, stroked only.
        assert_eq!(count("S"), 1);
        assert!(ai.contains(&format!(
            "\n0 0 m\n{} {} L\nS\n",
            f(50.0 * MM_PT),
            f(50.0 * MM_PT)
        )));
    }

    #[test]
    fn a_ring_s_hole_runs_the_other_way() {
        // Both circles drawn the same way: the hole must be turned so a
        // winding fill leaves it empty.
        let mut p = circle(Point::new(0.0, 0.0), 30.0);
        p.extend(circle(Point::new(0.0, 0.0), 10.0));
        let subs = orient_for_nonzero(subpaths(&p));
        assert_eq!(subs.len(), 2);
        assert!(subs[0].area() > 0.0 && subs[1].area() < 0.0);
        let mut all = subs[0].clone();
        all.extend(subs[1].clone());
        assert_eq!(all.winding(Point::new(0.0, 0.0)), 0);
        assert_ne!(all.winding(Point::new(20.0, 0.0)), 0);
    }

    #[test]
    fn fountains_become_clipped_bands_and_pictures_are_left_out() {
        let mut doc = Document::default();
        let layer = doc.pages[0].layers[0].id;
        let mut s = rect_shape(1, Rect::new(10.0, 10.0, 60.0, 40.0));
        s.fill = Fill::linear(Color::BLACK, Color::WHITE, 0.0);
        s.stroke = Some(tracedraw_core::Stroke::new(Color::BLACK, 1.0));
        let bmp = Shape::new(
            tracedraw_core::ShapeId(2),
            ShapeKind::Bitmap {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                width_px: 1,
                height_px: 1,
                png: Default::default(),
                fx: None,
            },
        );
        let l = doc.layer_mut(layer).unwrap();
        l.shapes.push(s);
        l.shapes.push(bmp);
        let ai = page_to_ai8(&doc, 0);
        let ops = ops(&ai);
        let count = |o: &str| ops.iter().filter(|x| **x == o).count();
        assert_eq!(count("q"), 1);
        assert_eq!(count("Q"), 1);
        assert_eq!(count("W"), 1);
        // Bands from black to white, then the outline over them.
        assert!(count("f") > 10, "{ai}");
        assert!(ai.contains("\n0 0 0 0.992 k\n"), "{ai}");
        assert!(ai.contains("\n0 0 0 0.008 k\n"), "{ai}");
        assert_eq!(count("s"), 1);
        assert!(!ai.contains("colorimage"));
    }

    #[test]
    fn classic_ai_reads_back_with_its_page_and_objects() {
        let doc = small_drawing();
        let ai = page_to_ai8(&doc, 0);
        let mut ids = tracedraw_core::id::IdSource::default();
        let back = crate::eps_import::parse(ai.as_bytes(), &mut ids).expect("reads");
        // The bounding box is the page.
        assert!((back.size.width - 210.0).abs() < 0.01, "{:?}", back.size);
        assert!((back.size.height - 297.0).abs() < 0.01, "{:?}", back.size);
        let page = |s: &Shape| s.page_path().bounding_box();
        let all: Vec<&Shape> = back.shapes.iter().collect();
        // The rectangle, its fill cyan and its outline there.
        let rect = all
            .iter()
            .find(|s| matches!(s.fill, Fill::Solid(Color::Cmyk { c, .. }) if c > 0.99))
            .expect("the cyan rectangle");
        let b = page(rect);
        assert!(
            (b.x0 - 10.0).abs() < 0.01 && (b.y0 - 20.0).abs() < 0.01,
            "{b:?}"
        );
        assert!(
            (b.x1 - 60.0).abs() < 0.01 && (b.y1 - 40.0).abs() < 0.01,
            "{b:?}"
        );
        // The ring: one object of two subpaths, its hole empty.
        let ring = all
            .iter()
            .find(
                |s| matches!(s.fill, Fill::Solid(Color::Cmyk { k, .. }) if (k - 0.75).abs() < 0.01),
            )
            .expect("the ring");
        let rp = ring.page_path();
        assert_eq!(subpaths(&rp).len(), 2);
        let b = rp.bounding_box();
        assert!(
            (b.x0 - 70.0).abs() < 0.01 && (b.x1 - 130.0).abs() < 0.01,
            "{b:?}"
        );
        // The line, stroked.
        assert!(all.iter().any(|s| s.stroke.is_some()
            && matches!(s.fill, Fill::None)
            && (page(s).x1 - 50.0).abs() < 0.01));
        assert!(
            back.warnings.iter().all(|w| !w.contains("undefined")),
            "{:?}",
            back.warnings
        );
    }

    #[test]
    fn no_page_writes_nothing() {
        assert!(page_to_ai8(&Document::default(), 3).is_empty());
    }
}
