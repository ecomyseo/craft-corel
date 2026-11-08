//! Drawing helpers for the toolbox icons: a 24 unit design grid mapped
//! onto a rectangle, paths written in the SVG path syntax (M, L, H, V,
//! C, S, Q, A and Z, absolute and relative), strokes with round caps and
//! joins, dashes, fills of concave shapes, and the motifs the icon style
//! shares (nodes, handles, the pointer, arrowheads, ribbons). See
//! docs/behavior/icons.md for the style rules.

use egui::epaint::{self, CornerRadius};
use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

/// The standard stroke of the icon style, grid units (design width).
pub const LINE: f32 = 2.0;
/// Guides and handles.
pub const THIN: f32 = 1.4;
/// Every stroke is drawn at this share of its design width: the standard
/// line is 1.25 px in a 22 px box at 1x (#1682, thinner icons).
pub const WEIGHT: f32 = 0.625;
/// Filled dots (brush dabs, drops, the blend circle) at this share of
/// their design radius.
pub const DOT: f32 = 0.85;
/// The dash and gap of a ghost (something proposed or gone), grid units.
pub const DASH: (f32, f32) = (2.4, 2.2);
/// The alpha of an object body's ink wash.
pub const WASH: f32 = 0.14;

/// One subpath: its flattened points on the grid, which of them are
/// corners (the end of a path command, where a round join goes) and
/// whether it closes.
#[derive(Clone, Debug, Default)]
pub struct Sub {
    pub pts: Vec<Pos2>,
    pub corner: Vec<bool>,
    pub closed: bool,
}

impl Sub {
    fn push(&mut self, p: Pos2, corner: bool) {
        self.pts.push(p);
        self.corner.push(corner);
    }
}

enum Tok {
    Cmd(char),
    Num(f32),
}

/// The tokens of an SVG path: command letters and numbers, where a
/// number ends at a second dot or a sign ("-3.5.5-4" is -3.5, 0.5, -4).
fn lex(d: &str) -> Vec<Tok> {
    let b = d.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_ascii_alphabetic() {
            out.push(Tok::Cmd(c));
            i += 1;
        } else if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' {
            let start = i;
            if c == '-' || c == '+' {
                i += 1;
            }
            let mut dot = false;
            while i < b.len() {
                let ch = b[i] as char;
                if ch.is_ascii_digit() {
                    i += 1;
                } else if ch == '.' && !dot {
                    dot = true;
                    i += 1;
                } else {
                    break;
                }
            }
            if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
                i += 1;
                if i < b.len() && (b[i] == b'-' || b[i] == b'+') {
                    i += 1;
                }
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let v: f32 = d[start..i].parse().unwrap_or(0.0);
            out.push(Tok::Num(v));
        } else {
            i += 1;
        }
    }
    out
}

/// Flatten `d` into subpaths on the grid.
pub fn parse(d: &str) -> Vec<Sub> {
    let toks = lex(d);
    let mut subs: Vec<Sub> = Vec::new();
    let mut cur = Pos2::ZERO;
    let mut start = Pos2::ZERO;
    // The last control point, reflected by S and T.
    let mut ctrl2: Option<Pos2> = None;
    let mut ctrl1: Option<Pos2> = None;
    let mut i = 0;
    let mut cmd = 'M';
    let num = |toks: &[Tok], i: &mut usize| -> Option<f32> {
        match toks.get(*i) {
            Some(Tok::Num(v)) => {
                *i += 1;
                Some(*v)
            }
            _ => None,
        }
    };
    while i < toks.len() {
        if let Tok::Cmd(c) = toks[i] {
            cmd = c;
            i += 1;
        }
        let rel = cmd.is_ascii_lowercase();
        let base = |rel: bool, cur: Pos2| if rel { cur.to_vec2() } else { Vec2::ZERO };
        match cmd.to_ascii_uppercase() {
            'M' => {
                let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) else {
                    break;
                };
                cur = Pos2::new(x, y) + base(rel, cur);
                start = cur;
                let mut s = Sub::default();
                s.push(cur, true);
                subs.push(s);
                ctrl1 = None;
                ctrl2 = None;
                // Further pairs are implicit line-tos.
                cmd = if rel { 'l' } else { 'L' };
            }
            'L' | 'H' | 'V' => {
                let next = match cmd.to_ascii_uppercase() {
                    'L' => {
                        let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) else {
                            break;
                        };
                        Pos2::new(x, y) + base(rel, cur)
                    }
                    'H' => {
                        let Some(x) = num(&toks, &mut i) else { break };
                        Pos2::new(x + if rel { cur.x } else { 0.0 }, cur.y)
                    }
                    _ => {
                        let Some(y) = num(&toks, &mut i) else { break };
                        Pos2::new(cur.x, y + if rel { cur.y } else { 0.0 })
                    }
                };
                cur = next;
                if let Some(s) = subs.last_mut() {
                    s.push(cur, true);
                }
                ctrl1 = None;
                ctrl2 = None;
            }
            'C' | 'S' => {
                let p1 = if cmd.eq_ignore_ascii_case(&'C') {
                    let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) else {
                        break;
                    };
                    Pos2::new(x, y) + base(rel, cur)
                } else {
                    ctrl2.map(|c| cur + (cur - c)).unwrap_or(cur)
                };
                let (Some(x2), Some(y2), Some(x), Some(y)) = (
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                ) else {
                    break;
                };
                let p2 = Pos2::new(x2, y2) + base(rel, cur);
                let p3 = Pos2::new(x, y) + base(rel, cur);
                if let Some(s) = subs.last_mut() {
                    let n = 12;
                    for j in 1..=n {
                        let t = j as f32 / n as f32;
                        s.push(crate::ui::brand::cubic_point(&[cur, p1, p2, p3], t), j == n);
                    }
                }
                ctrl2 = Some(p2);
                ctrl1 = None;
                cur = p3;
            }
            'Q' | 'T' => {
                let p1 = if cmd.eq_ignore_ascii_case(&'Q') {
                    let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) else {
                        break;
                    };
                    Pos2::new(x, y) + base(rel, cur)
                } else {
                    ctrl1.map(|c| cur + (cur - c)).unwrap_or(cur)
                };
                let (Some(x), Some(y)) = (num(&toks, &mut i), num(&toks, &mut i)) else {
                    break;
                };
                let p2 = Pos2::new(x, y) + base(rel, cur);
                if let Some(s) = subs.last_mut() {
                    let n = 10;
                    for j in 1..=n {
                        let t = j as f32 / n as f32;
                        let u = 1.0 - t;
                        let p = Pos2::new(
                            u * u * cur.x + 2.0 * u * t * p1.x + t * t * p2.x,
                            u * u * cur.y + 2.0 * u * t * p1.y + t * t * p2.y,
                        );
                        s.push(p, j == n);
                    }
                }
                ctrl1 = Some(p1);
                ctrl2 = None;
                cur = p2;
            }
            'A' => {
                let (Some(rx), Some(ry), Some(rot), Some(large), Some(sweep), Some(x), Some(y)) = (
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                    num(&toks, &mut i),
                ) else {
                    break;
                };
                let end = Pos2::new(x, y) + base(rel, cur);
                if let Some(s) = subs.last_mut() {
                    for (p, last) in arc_points(cur, end, rx, ry, rot, large != 0.0, sweep != 0.0) {
                        s.push(p, last);
                    }
                }
                ctrl1 = None;
                ctrl2 = None;
                cur = end;
            }
            'Z' => {
                if let Some(s) = subs.last_mut() {
                    s.closed = true;
                }
                cur = start;
                ctrl1 = None;
                ctrl2 = None;
                // A command letter must follow; stray numbers are skipped.
                while let Some(Tok::Num(_)) = toks.get(i) {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
        }
    }
    subs
}

/// The points of an SVG elliptical arc from `from` to `to`, the last
/// one flagged (SVG 1.1, F.6.5 and F.6.6).
fn arc_points(
    from: Pos2,
    to: Pos2,
    rx: f32,
    ry: f32,
    rot_deg: f32,
    large: bool,
    sweep: bool,
) -> Vec<(Pos2, bool)> {
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx < 1e-6 || ry < 1e-6 || from == to {
        return vec![(to, true)];
    }
    let phi = rot_deg.to_radians();
    let (sp, cp) = phi.sin_cos();
    let dx = (from.x - to.x) / 2.0;
    let dy = (from.y - to.y) / 2.0;
    let x1 = cp * dx + sp * dy;
    let y1 = -sp * dx + cp * dy;
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }
    let num = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut coef = if den > 0.0 { (num / den).sqrt() } else { 0.0 };
    if large == sweep {
        coef = -coef;
    }
    let cx1 = coef * rx * y1 / ry;
    let cy1 = -coef * ry * x1 / rx;
    let cx = cp * cx1 - sp * cy1 + (from.x + to.x) / 2.0;
    let cy = sp * cx1 + cp * cy1 + (from.y + to.y) / 2.0;
    let ang = |ux: f32, uy: f32, vx: f32, vy: f32| {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        let mut a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 {
            a = -a;
        }
        a
    };
    let theta1 = ang(1.0, 0.0, (x1 - cx1) / rx, (y1 - cy1) / ry);
    let mut dtheta = ang(
        (x1 - cx1) / rx,
        (y1 - cy1) / ry,
        (-x1 - cx1) / rx,
        (-y1 - cy1) / ry,
    );
    if !sweep && dtheta > 0.0 {
        dtheta -= std::f32::consts::TAU;
    } else if sweep && dtheta < 0.0 {
        dtheta += std::f32::consts::TAU;
    }
    let n = ((dtheta.abs() / (std::f32::consts::PI / 12.0)).ceil() as usize).max(3);
    (1..=n)
        .map(|j| {
            let t = theta1 + dtheta * j as f32 / n as f32;
            let (st, ct) = t.sin_cos();
            let x = cp * rx * ct - sp * ry * st + cx;
            let y = sp * rx * ct + cp * ry * st + cy;
            (if j == n { to } else { Pos2::new(x, y) }, j == n)
        })
        .collect()
}

/// Triangles of a simple polygon by ear clipping; a fan when the polygon
/// is degenerate.
pub fn triangulate(pts: &[Pos2]) -> Vec<[usize; 3]> {
    let n = pts.len();
    let mut tris = Vec::new();
    if n < 3 {
        return tris;
    }
    let area: f32 = (0..n)
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    let sign = if area >= 0.0 { 1.0 } else { -1.0 };
    let cross = |a: Pos2, b: Pos2, c: Pos2| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let inside = |p: Pos2, a: Pos2, b: Pos2, c: Pos2| {
        let eps = -1e-5;
        cross(a, b, p) * sign >= eps && cross(b, c, p) * sign >= eps && cross(c, a, p) * sign >= eps
    };
    let mut idx: Vec<usize> = (0..n).collect();
    let mut guard = 0;
    while idx.len() > 3 && guard < 4 * n {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for k in 0..m {
            let (i0, i1, i2) = (idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]);
            let (a, b, c) = (pts[i0], pts[i1], pts[i2]);
            if cross(a, b, c) * sign <= 1e-7 {
                continue;
            }
            let blocked = idx
                .iter()
                .any(|&j| j != i0 && j != i1 && j != i2 && inside(pts[j], a, b, c));
            if blocked {
                continue;
            }
            tris.push([i0, i1, i2]);
            idx.remove(k);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if idx.len() == 3 {
        tris.push([idx[0], idx[1], idx[2]]);
    } else {
        for k in 1..idx.len().saturating_sub(1) {
            tris.push([idx[0], idx[k], idx[k + 1]]);
        }
    }
    tris
}

/// Point `(cx, cy)` moved by `r` at `deg` degrees (clockwise on screen).
pub fn polar(cx: f32, cy: f32, r: f32, deg: f32) -> (f32, f32) {
    let a = deg.to_radians();
    (cx + r * a.cos(), cy + r * a.sin())
}

/// The SVG path of a rounded rectangle.
pub fn rrect(x: f32, y: f32, w: f32, h: f32, rx: f32) -> String {
    let r = rx.min(w / 2.0).min(h / 2.0);
    if r <= 0.0 {
        return format!("M{x} {y}h{w}v{h}h{}z", -w);
    }
    format!(
        "M{} {y}h{}a{r} {r} 0 0 1 {r} {r}v{}a{r} {r} 0 0 1 {} {r}h{}a{r} {r} 0 0 1 {} {}v{}a{r} {r} 0 0 1 {r} {}z",
        x + r,
        w - 2.0 * r,
        h - 2.0 * r,
        -r,
        -(w - 2.0 * r),
        -r,
        -r,
        -(h - 2.0 * r),
        -r
    )
}

/// The points of a star with `n` points, outer radius `r1`, inner `r2`.
pub fn star(cx: f32, cy: f32, r1: f32, r2: f32, n: usize, rot: f32) -> Vec<Pos2> {
    (0..2 * n)
        .map(|i| {
            let a = (rot + i as f32 * 180.0 / n as f32).to_radians();
            let r = if i % 2 == 0 { r1 } else { r2 };
            Pos2::new(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// The points of a spiral of `turns` turns from radius `r0` to `r1`,
/// starting at `rot` degrees.
pub fn spiral(cx: f32, cy: f32, turns: f32, r0: f32, r1: f32, n: usize, rot: f32) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let a = rot.to_radians() + t * turns * std::f32::consts::TAU;
            let r = r0 + (r1 - r0) * t;
            Pos2::new(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// The closed outline of a stroke of width `w(t)` (half width each side)
/// along the cubic `p`.
pub fn ribbon(p: [Pos2; 4], w: impl Fn(f32) -> f32, n: usize) -> Vec<Pos2> {
    let mut left = Vec::with_capacity(n + 1);
    let mut right = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let t = i as f32 / n as f32;
        let at = crate::ui::brand::cubic_point(&p, t);
        let tan = crate::ui::brand::cubic_tangent(&p, t);
        let len = tan.length().max(1e-6);
        let nrm = Vec2::new(-tan.y, tan.x) / len;
        let hw = w(t);
        left.push(at + nrm * hw);
        right.push(at - nrm * hw);
    }
    right.reverse();
    left.extend(right);
    left
}

/// The brush for one icon: the grid, the three colours of the style and
/// a transform for parts drawn turned.
#[derive(Clone)]
pub struct Pen {
    painter: Painter,
    /// Pixels (points) per grid unit.
    k: f32,
    /// Ink draws the object, the accent the action, the background the
    /// opaque bodies that sit in front of something.
    pub ink: Color32,
    pub acc: Color32,
    pub bg: Color32,
    /// Rotation applied to grid coordinates: angle and centre.
    rot: Option<(f32, Pos2)>,
    /// Where grid (0, 0) lands, points: the grid centred on the box and
    /// snapped so integer units sit on pixel centres (see `new`).
    origin: Pos2,
    /// Physical pixels per point.
    ppp: f32,
}

/// The points per grid unit of an icon drawn in a box `width` points
/// wide at `ppp` pixels per point. A box within a twelfth of a whole
/// number of pixels per unit takes that whole number (a 22 px box draws
/// the 24 unit grid at 1 px per unit, at 2x at 2 px), so every integer
/// unit is on a pixel; any other box keeps its own size, the grid's side
/// rounded to whole pixels (16, 20, 32 and 40 px boxes: a pixel every
/// 3, 6, 3 and 3 units), so a larger box always draws a larger icon.
/// Boxes under 0.85 px per unit keep the fractional scale.
pub fn grid_scale(width: f32, ppp: f32) -> f32 {
    let ppp = if ppp.is_finite() && ppp > 0.0 {
        ppp
    } else {
        1.0
    };
    let kp = width * ppp / 24.0;
    if !kp.is_finite() || kp < 0.85 {
        return if kp.is_finite() { kp / ppp } else { 1.0 / ppp };
    }
    let whole = kp.round().max(1.0);
    if (kp / whole - 1.0).abs() <= 1.0 / 12.0 + 1e-4 {
        whole / ppp
    } else {
        (kp * 24.0).round() / 24.0 / ppp
    }
}

impl Pen {
    /// The grid is `r.width() / 24` points per unit, snapped to whole
    /// physical pixels (see [`grid_scale`]). The grid is centred on `r`
    /// with its origin on a pixel edge, moved half a pixel when the
    /// standard line's pixel width is odd (or under 2 px), so that
    /// straight strokes on integer units are crisp.
    pub fn new(painter: &Painter, r: Rect, ink: Color32, acc: Color32, bg: Color32) -> Self {
        let ppp = painter.pixels_per_point();
        let k = grid_scale(r.width(), ppp);
        let o = r.center() - Vec2::splat(12.0 * k);
        let mut origin = Pos2::new((o.x * ppp).round() / ppp, (o.y * ppp).round() / ppp);
        let wpx = LINE * k * WEIGHT * ppp;
        if wpx < 1.9 || (wpx.round() as i64) % 2 == 1 {
            origin += Vec2::splat(0.5 / ppp);
        }
        Pen {
            painter: painter.clone(),
            k,
            ink,
            acc,
            bg,
            rot: None,
            origin,
            ppp,
        }
    }

    /// The same pen drawing everything turned `deg` degrees around
    /// `(cx, cy)`.
    pub fn rotated(&self, deg: f32, cx: f32, cy: f32) -> Pen {
        let mut p = self.clone();
        p.rot = Some((deg.to_radians(), Pos2::new(cx, cy)));
        p
    }

    /// `color` at `alpha` (0 to 1).
    pub fn alpha(color: Color32, alpha: f32) -> Color32 {
        color.gamma_multiply(alpha)
    }

    /// The ink wash of an object body.
    pub fn wash(&self) -> Color32 {
        Self::alpha(self.ink, WASH)
    }

    /// Pixels of `v` grid units.
    pub fn px(&self, v: f32) -> f32 {
        v * self.k
    }

    /// The stroke width in points at the icon's weight, never thinner
    /// than a physical pixel.
    fn width(&self, w: f32) -> f32 {
        (w * self.k * WEIGHT).max(1.0 / self.ppp)
    }

    /// The design width (grid units) that draws exactly one physical
    /// pixel: the frames of hollow nodes and handles.
    fn hairline(&self) -> f32 {
        1.0 / (self.ppp * self.k * WEIGHT)
    }

    fn map(&self, p: Pos2) -> Pos2 {
        let p = match self.rot {
            Some((a, c)) => {
                let (s, co) = a.sin_cos();
                let d = p - c;
                c + Vec2::new(d.x * co - d.y * s, d.x * s + d.y * co)
            }
            None => p,
        };
        self.origin + p.to_vec2() * self.k
    }

    /// Grid point `(x, y)` on the screen.
    pub fn at(&self, x: f32, y: f32) -> Pos2 {
        self.map(Pos2::new(x, y))
    }

    fn screen(&self, pts: &[Pos2]) -> Vec<Pos2> {
        pts.iter().map(|&p| self.map(p)).collect()
    }

    /// Stroke `d` (an SVG path) `w` units wide with round caps and joins.
    pub fn stroke(&self, d: &str, w: f32, color: Color32) {
        for s in parse(d) {
            self.stroke_sub(&s, w, color);
        }
    }

    pub fn line(&self, d: &str, color: Color32) {
        self.stroke(d, LINE, color);
    }

    pub fn thin(&self, d: &str, color: Color32) {
        self.stroke(d, THIN, color);
    }

    /// Stroke grid points, every one a corner.
    pub fn stroke_pts(&self, pts: &[Pos2], closed: bool, w: f32, color: Color32) {
        let s = Sub {
            pts: pts.to_vec(),
            corner: vec![true; pts.len()],
            closed,
        };
        self.stroke_sub(&s, w, color);
    }

    fn stroke_sub(&self, s: &Sub, w: f32, color: Color32) {
        if s.pts.len() < 2 || color.a() == 0 {
            return;
        }
        let wpx = self.width(w);
        let pts = self.screen(&s.pts);
        let stroke = Stroke::new(wpx, color);
        if s.closed {
            self.painter
                .add(epaint::PathShape::closed_line(pts.clone(), stroke));
        } else {
            self.painter
                .add(epaint::PathShape::line(pts.clone(), stroke));
        }
        // Round caps and joins: a dot at each open end and corner of a
        // stroke at least three pixels wide (on thinner lines the dots
        // only fatten the corners). A translucent stroke keeps the plain
        // ends, the dots would darken them.
        if color.a() < 255 || wpx * self.ppp < 3.0 {
            return;
        }
        let rad = wpx / 2.0;
        let n = pts.len();
        for (i, &p) in pts.iter().enumerate() {
            let end = !s.closed && (i == 0 || i + 1 == n);
            if end || s.corner.get(i).copied().unwrap_or(false) {
                self.painter.circle_filled(p, rad, color);
            }
        }
    }

    /// Stroke `d` dashed (`dash` on, `gap` off, grid units).
    pub fn dashed(&self, d: &str, w: f32, color: Color32, dash: f32, gap: f32) {
        for s in parse(d) {
            let mut pts = self.screen(&s.pts);
            if s.closed {
                if let Some(&f) = pts.first() {
                    pts.push(f);
                }
            }
            let shapes = epaint::Shape::dashed_line(
                &pts,
                Stroke::new(self.width(w), color),
                self.px(dash),
                self.px(gap),
            );
            self.painter.extend(shapes);
        }
    }

    /// Stroke `d` as a ghost: the standard dashes.
    pub fn ghost(&self, d: &str, color: Color32) {
        self.dashed(d, LINE, color, DASH.0, DASH.1);
    }

    /// Fill `d`, concave shapes included; `edge` adds a hairline of the
    /// same colour so the edge is smoothed like a stroke's.
    pub fn fill(&self, d: &str, color: Color32, edge: bool) {
        for s in parse(d) {
            self.fill_pts(&s.pts, color, edge);
        }
    }

    pub fn fill_pts(&self, pts: &[Pos2], color: Color32, edge: bool) {
        if pts.len() < 3 || color.a() == 0 {
            return;
        }
        let mut clean: Vec<Pos2> = Vec::with_capacity(pts.len());
        for &p in pts {
            if clean.last().is_none_or(|&l| (l - p).length_sq() > 1e-8) {
                clean.push(p);
            }
        }
        if clean.len() > 1 && (clean[0] - clean[clean.len() - 1]).length_sq() <= 1e-8 {
            clean.pop();
        }
        if clean.len() < 3 {
            return;
        }
        let scr = self.screen(&clean);
        let mut mesh = egui::Mesh::default();
        for &p in &scr {
            mesh.colored_vertex(p, color);
        }
        for [a, b, c] in triangulate(&scr) {
            mesh.add_triangle(a as u32, b as u32, c as u32);
        }
        self.painter.add(mesh);
        if edge {
            // One physical pixel: a real anti-aliased edge (a thinner
            // stroke is faded by the tessellator into a 2 px halo).
            self.painter.add(epaint::PathShape::closed_line(
                scr,
                Stroke::new(1.0 / self.ppp, color),
            ));
        }
    }

    /// A disc whose fill fades from `from` at its left to `to` at its
    /// right (a fan with vertex colours, premultiplied).
    pub fn fade_circle(&self, cx: f32, cy: f32, rad: f32, from: Color32, to: Color32) {
        let mix = |t: f32| {
            let a = from.to_array().map(|b| b as f32);
            let b = to.to_array().map(|b| b as f32);
            let c = |i: usize| (a[i] + (b[i] - a[i]) * t).round() as u8;
            Color32::from_rgba_premultiplied(c(0), c(1), c(2), c(3))
        };
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(self.at(cx, cy), mix(0.5));
        let n = 48u32;
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            let (x, y) = (cx + rad * a.cos(), cy + rad * a.sin());
            mesh.colored_vertex(self.at(x, y), mix((x - (cx - rad)) / (2.0 * rad)));
        }
        for i in 0..n {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        self.painter.add(mesh);
    }

    /// A convex fill (nodes, arrowheads), smoothed by the tessellator.
    pub fn convex(&self, pts: &[Pos2], color: Color32) {
        self.painter.add(epaint::PathShape::convex_polygon(
            self.screen(pts),
            color,
            Stroke::NONE,
        ));
    }

    /// An object body: the ink wash under an ink line.
    pub fn body(&self, d: &str) {
        self.fill(d, self.wash(), false);
        self.line(d, self.ink);
    }

    /// An opaque body in front of something: the background under an
    /// ink line.
    pub fn solid(&self, d: &str) {
        self.fill(d, self.bg, false);
        self.line(d, self.ink);
    }

    /// A circle (`fill` transparent for a ring), stroke `w` units.
    pub fn circle(&self, cx: f32, cy: f32, rad: f32, fill: Color32, w: f32, stroke: Color32) {
        let c = self.at(cx, cy);
        // A filled dot is drawn smaller (DOT); rings keep their radius.
        let dot = w <= 0.0 && fill.a() == 255;
        let rpx = self.px(rad) * if dot { DOT } else { 1.0 };
        if fill.a() > 0 {
            self.painter.circle_filled(c, rpx, fill);
        }
        if w > 0.0 && stroke.a() > 0 {
            self.painter
                .circle_stroke(c, rpx, Stroke::new(self.width(w), stroke));
        }
    }

    /// A rounded rectangle, `w` the stroke width (0 for none).
    pub fn rect(
        &self,
        x: f32,
        y: f32,
        wd: f32,
        ht: f32,
        rx: f32,
        fill: Color32,
        w: f32,
        stroke: Color32,
    ) {
        if self.rot.is_some() {
            let d = rrect(x, y, wd, ht, rx);
            if fill.a() > 0 {
                self.fill(&d, fill, false);
            }
            if w > 0.0 {
                self.stroke(&d, w, stroke);
            }
            return;
        }
        let r = Rect::from_min_max(self.at(x, y), self.at(x + wd, y + ht));
        let cr = CornerRadius::same(self.px(rx).round().clamp(0.0, 255.0) as u8);
        if fill.a() > 0 {
            self.painter.rect_filled(r, cr, fill);
        }
        if w > 0.0 && stroke.a() > 0 {
            self.painter.rect_stroke(
                r,
                cr,
                Stroke::new(self.width(w), stroke),
                epaint::StrokeKind::Middle,
            );
        }
    }

    /// The node motif: a small square on the line. Blue marks the live
    /// point the tool acts on; hollow (background with an ink frame) a
    /// plain anchor.
    pub fn node(&self, x: f32, y: f32, live: bool) {
        let s = 3.0;
        let h = s / 2.0;
        if live {
            self.rect(x - h, y - h, s, s, 0.8, self.acc, 0.0, Color32::TRANSPARENT);
        } else {
            self.rect(x - h, y - h, s, s, 0.8, self.bg, self.hairline(), self.ink);
        }
    }

    /// A handle end: a hollow accent circle, a pixel's frame.
    pub fn handle(&self, x: f32, y: f32, rad: f32) {
        self.circle(x, y, rad, self.bg, self.hairline(), self.acc);
    }

    /// The pick arrow (slanted tail), scaled `s`, its tip at `(ox, oy)`,
    /// with the live node at its tail when `live`.
    pub fn pointer(&self, ox: f32, oy: f32, s: f32, live: bool) {
        let pts: Vec<Pos2> = [
            (0.0, 0.0),
            (0.0, 14.9),
            (4.0, 11.2),
            (6.9, 17.4),
            (9.5, 16.2),
            (6.6, 10.1),
            (12.1, 10.1),
        ]
        .iter()
        .map(|&(x, y)| Pos2::new(ox + x * s, oy + y * s))
        .collect();
        self.fill_pts(&pts, self.wash(), false);
        let w = if s >= 0.8 { LINE } else { 1.7 };
        self.stroke_pts(&pts, true, w, self.ink);
        if live {
            self.rect(
                ox + 13.6 * s - 1.5,
                oy + 15.6 * s - 1.5,
                3.0,
                3.0,
                1.0,
                self.acc,
                0.0,
                Color32::TRANSPARENT,
            );
        }
    }

    /// A filled arrowhead, its tip at `(x, y)`, pointing along `(dx, dy)`.
    pub fn arrow_head(&self, x: f32, y: f32, dx: f32, dy: f32, size: f32, color: Color32) {
        let l = (dx * dx + dy * dy).sqrt().max(1e-6);
        let (dx, dy) = (dx / l, dy / l);
        let (nx, ny) = (-dy, dx);
        let (bx, by) = (x - dx * size, y - dy * size);
        let w = size * 0.55;
        self.convex(
            &[
                Pos2::new(x, y),
                Pos2::new(bx + nx * w, by + ny * w),
                Pos2::new(bx - nx * w, by - ny * w),
            ],
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The grid grows with its box: whole pixels per unit near 1x and
    /// 2x, the box's own size elsewhere (the button sizes' 16, 20, 24,
    /// 32 and 40 px boxes each draw a larger icon than the one before).
    #[test]
    fn the_grid_grows_with_its_box() {
        assert_eq!(grid_scale(22.0, 1.0), 1.0);
        assert_eq!(grid_scale(24.0, 1.0), 1.0);
        assert_eq!(grid_scale(22.0, 2.0), 1.0);
        assert_eq!(grid_scale(48.0, 1.0), 2.0);
        assert!((grid_scale(18.0, 1.0) - 0.75).abs() < 1e-6);
        let sides: Vec<f32> = [16.0, 20.0, 24.0, 32.0, 40.0]
            .iter()
            .map(|w| grid_scale(*w, 1.0) * 24.0)
            .collect();
        for (side, w) in sides.iter().zip([16.0, 20.0, 24.0, 32.0, 40.0]) {
            assert!((side - w).abs() < 1e-4, "{side} for {w}");
        }
        for ppp in [1.0, 1.25, 1.5, 2.0] {
            let ks: Vec<f32> = [16.0, 20.0, 24.0, 32.0, 40.0]
                .iter()
                .map(|w| grid_scale(*w, ppp))
                .collect();
            assert!(ks.windows(2).all(|w| w[1] > w[0]), "{ppp}: {ks:?}");
            // The grid's side is whole pixels.
            for k in ks {
                let px = k * 24.0 * ppp;
                assert!((px - px.round()).abs() < 1e-3, "{ppp}: {px}");
            }
        }
        assert!(grid_scale(f32::NAN, 1.0).is_finite());
        assert!(grid_scale(22.0, 0.0).is_finite());
    }

    #[test]
    fn numbers_split_at_signs_and_second_dots() {
        let toks = lex("c-1.5-.5-3.5.5-4 2.5");
        let nums: Vec<f32> = toks
            .iter()
            .filter_map(|t| match t {
                Tok::Num(v) => Some(*v),
                _ => None,
            })
            .collect();
        assert_eq!(nums, vec![-1.5, -0.5, -3.5, 0.5, -4.0, 2.5]);
    }

    #[test]
    fn paths_flatten_with_relative_and_implicit_commands() {
        let subs = parse("M3 19L7 6 13 15 20 4");
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].pts.len(), 4);
        assert_eq!(subs[0].pts[3], Pos2::new(20.0, 4.0));
        assert!(!subs[0].closed);
        let subs = parse("M4 9v10a1.5 1.5 0 0 0 1.5 1.5h13z");
        assert!(subs[0].closed);
        let last = subs[0].pts[subs[0].pts.len() - 1];
        assert!((last.x - 18.5).abs() < 1e-4 && (last.y - 20.5).abs() < 1e-4);
        // Two subpaths from one M each.
        assert_eq!(parse("M7 7h10M7 10.5h6").len(), 2);
    }

    #[test]
    fn arcs_pass_through_their_midpoint() {
        // The small arc of radius 9.5 from (3,18) to (21,18), clockwise
        // on screen: its centre is 3.04 below the chord (the 3-point
        // curve icon), so its apex is at y = 21.04 - 9.5.
        let subs = parse("M3 18A9.5 9.5 0 0 1 21 18");
        let pts = &subs[0].pts;
        let top = pts.iter().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!((top - 11.54).abs() < 0.2, "{top}");
        // The half circle: its apex is the radius above the chord.
        let subs = parse("M3 18A9 9 0 0 1 21 18");
        let top = subs[0].pts.iter().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!((top - 9.0).abs() < 0.2, "{top}");
        assert_eq!(pts[pts.len() - 1], Pos2::new(21.0, 18.0));
    }

    #[test]
    fn concave_shapes_triangulate_fully() {
        // An L shape: 6 vertices, 4 triangles, every one inside it.
        let l = [
            Pos2::new(0.0, 0.0),
            Pos2::new(4.0, 0.0),
            Pos2::new(4.0, 1.0),
            Pos2::new(1.0, 1.0),
            Pos2::new(1.0, 4.0),
            Pos2::new(0.0, 4.0),
        ];
        let tris = triangulate(&l);
        assert_eq!(tris.len(), 4);
        let area: f32 = tris
            .iter()
            .map(|&[a, b, c]| {
                let (p, q, r) = (l[a], l[b], l[c]);
                ((q.x - p.x) * (r.y - p.y) - (q.y - p.y) * (r.x - p.x)).abs() / 2.0
            })
            .sum();
        assert!((area - 7.0).abs() < 1e-4, "{area}");
        // The star is concave with 10 vertices.
        let s = star(12.0, 13.0, 9.5, 4.3, 5, -90.0);
        assert_eq!(triangulate(&s).len(), 8);
    }

    #[test]
    fn ribbons_close_around_the_curve() {
        let pts = ribbon(
            [
                Pos2::new(3.0, 18.0),
                Pos2::new(7.0, 6.0),
                Pos2::new(17.0, 6.0),
                Pos2::new(21.0, 18.0),
            ],
            |t| 0.5 + 2.6 * (std::f32::consts::PI * t).sin(),
            28,
        );
        assert_eq!(pts.len(), 58);
        assert!((pts[0] - pts[57]).length() < 1.1);
    }
}
