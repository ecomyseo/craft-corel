//! The Coordinates docker's work (Window > Dockers > Coordinates): draw or
//! replace rectangles, squares, ellipses, circles, polygons, regular
//! polygons, stars, complex stars, two-point lines and multipoint curves
//! from typed coordinates, with a live preview on the drawing. The "Set ... interactively" buttons take the
//! next click or drag on the drawing instead of a typed value.

use crate::app::App;
use crate::i18n::tr;
use egui::{PointerButton, Response};
use tracedraw_core::{
    document::{axis_scale, ShapeKind},
    geometry::{
        complex_star_max_sharpness, polyline_path, Affine, BezPath, PathEl, Point, Rect,
        COMPLEX_STAR_MIN_POINTS,
    },
    live::Effect,
    Command, Shape, ShapeId,
};

/// The object the docker draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoordKind {
    #[default]
    Rectangle,
    Square,
    Ellipse,
    Circle,
    Polygon,
    RegularPolygon,
    Star,
    ComplexStar,
    Line,
    MultiPoint,
}

impl CoordKind {
    #[cfg(test)]
    pub const ALL: [CoordKind; 10] = [
        CoordKind::Rectangle,
        CoordKind::Square,
        CoordKind::Ellipse,
        CoordKind::Circle,
        CoordKind::Polygon,
        CoordKind::RegularPolygon,
        CoordKind::Star,
        CoordKind::ComplexStar,
        CoordKind::Line,
        CoordKind::MultiPoint,
    ];

    /// Rectangles, squares, ellipses and circles: a box with an origin
    /// point.
    pub fn is_boxed(self) -> bool {
        matches!(
            self,
            CoordKind::Rectangle | CoordKind::Square | CoordKind::Ellipse | CoordKind::Circle
        )
    }

    /// Polygons and stars: a centre and a bounding ellipse.
    pub fn is_polygon(self) -> bool {
        matches!(
            self,
            CoordKind::Polygon
                | CoordKind::RegularPolygon
                | CoordKind::Star
                | CoordKind::ComplexStar
        )
    }

    /// One size (a side or a diameter) instead of two.
    pub fn is_uniform(self) -> bool {
        matches!(
            self,
            CoordKind::Square | CoordKind::Circle | CoordKind::RegularPolygon
        )
    }

    /// The i18n key of its name.
    pub fn key(self) -> &'static str {
        match self {
            CoordKind::Rectangle => "coords.rectangle",
            CoordKind::Square => "coords.square",
            CoordKind::Ellipse => "coords.ellipse",
            CoordKind::Circle => "coords.circle",
            CoordKind::Polygon => "coords.polygon",
            CoordKind::RegularPolygon => "coords.regular_polygon",
            CoordKind::Star => "coords.star",
            CoordKind::ComplexStar => "coords.complex_star",
            CoordKind::Line => "coords.line",
            CoordKind::MultiPoint => "coords.multipoint",
        }
    }
}

/// What a "Set ... interactively" button waits for on the drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordPick {
    /// A click places the origin point (a polygon's centre).
    Origin,
    /// A drag's length sets the width (a side, a diameter).
    Width,
    /// A drag's length sets the height.
    Height,
    /// A drag's direction sets the angle of rotation.
    Angle,
    /// A click places the box's lower-left or upper-right corner.
    LowerLeft,
    UpperRight,
    /// A drag's length sets a regular polygon's side.
    SideLength,
    /// A click places a line's start or end point.
    Start,
    End,
    /// A drag's length sets a line's length.
    LineLength,
    /// A click places the chosen point of a multipoint curve (a new one
    /// when none is chosen).
    Point,
}

impl CoordPick {
    /// Picks made by dragging rather than clicking.
    pub fn is_drag(self) -> bool {
        matches!(
            self,
            CoordPick::Width
                | CoordPick::Height
                | CoordPick::Angle
                | CoordPick::SideLength
                | CoordPick::LineLength
        )
    }
}

/// The docker's fields; positions in page millimetres (the docker shows
/// them from the ruler origin), angles in degrees counter-clockwise.
#[derive(Debug, Clone, PartialEq)]
pub struct CoordsState {
    pub kind: CoordKind,
    /// The origin point of a box: 0 to 8, row by row from the top left.
    pub origin: usize,
    /// Where the origin point (a polygon's centre) is.
    pub at: Point,
    /// Width and height; a polygon's bounding ellipse diameters. Squares,
    /// circles and regular polygons use `w` only.
    pub w: f64,
    pub h: f64,
    /// Changing one size changes the other in proportion.
    pub proportional: bool,
    pub angle: f64,
    pub points: u32,
    /// A star's sharpness, 0.01 to 0.99, or -0.01 to -0.99 inverted.
    pub sharpness: f64,
    /// A complex star's sharpness, 1 up to what its points allow.
    pub complex_sharpness: u32,
    pub start: Point,
    pub end: Point,
    /// A multipoint curve's points, the chosen one, and whether it closes.
    pub curve: Vec<Point>,
    pub current: Option<usize>,
    pub closed: bool,
    /// The control points of the segment that ends at each point of
    /// `curve` (the first point's: the closing segment's), `None` for a
    /// straight one: a curve read from the drawing keeps its bends.
    /// Shorter than `curve` when the last points are straight.
    pub curve_ctrl: Vec<Option<(Point, Point)>>,
    /// The element index of each point of `curve` in the path it was read
    /// from, so a node chosen on the drawing chooses its point (#1889).
    /// Empty once points are added or deleted.
    pub curve_el: Vec<usize>,
    /// The selected object the fields were read from.
    pub loaded: Option<ShapeId>,
    /// Its outline and placement when they were read, so an edit of the
    /// object on the drawing reads it again.
    pub loaded_from: Option<(ShapeKind, Affine)>,
    /// The node chosen on the drawing that last chose a point.
    pub followed_node: Option<(ShapeId, usize)>,
    /// The fields were placed on the page once.
    pub placed: bool,
}

impl Default for CoordsState {
    fn default() -> Self {
        CoordsState {
            kind: CoordKind::Rectangle,
            origin: 4,
            at: Point::new(100.0, 100.0),
            w: 50.0,
            h: 30.0,
            proportional: false,
            angle: 0.0,
            points: 5,
            sharpness: 0.5,
            complex_sharpness: 1,
            start: Point::new(50.0, 100.0),
            end: Point::new(150.0, 100.0),
            curve: Vec::new(),
            current: None,
            closed: false,
            curve_ctrl: Vec::new(),
            curve_el: Vec::new(),
            loaded: None,
            loaded_from: None,
            followed_node: None,
            placed: false,
        }
    }
}

/// Point `i` of the nine origin points of `r` (row by row from the top
/// left; Y is up, so row 0 is `y1`).
pub fn origin_in(r: Rect, i: usize) -> Point {
    let (col, row) = ((i % 3).min(2), (i / 3).min(2));
    let x = [r.x0, r.center().x, r.x1][col];
    let y = [r.y1, r.center().y, r.y0][row];
    Point::new(x, y)
}

fn finite_pos(v: f64) -> bool {
    v.is_finite() && v > 1e-9
}

/// An angle brought into (-180, 180].
fn norm_deg(a: f64) -> f64 {
    if !a.is_finite() {
        return 0.0;
    }
    let mut a = a % 360.0;
    if a <= -180.0 {
        a += 360.0;
    } else if a > 180.0 {
        a -= 360.0;
    }
    a
}

impl CoordsState {
    /// Put the defaults on `page` (its middle) the first time.
    pub fn place_on(&mut self, page: Rect) {
        if self.placed {
            return;
        }
        self.placed = true;
        let c = page.center();
        self.at = c;
        self.start = Point::new(c.x - 50.0, c.y);
        self.end = Point::new(c.x + 50.0, c.y);
    }

    /// Width and height as drawn.
    pub fn size(&self) -> (f64, f64) {
        if self.kind.is_uniform() {
            (self.w, self.w)
        } else {
            (self.w, self.h)
        }
    }

    /// Choose what to draw; sizes and points are kept where they fit.
    pub fn set_kind(&mut self, kind: CoordKind) {
        self.kind = kind;
        if kind == CoordKind::ComplexStar {
            self.points = self.points.max(COMPLEX_STAR_MIN_POINTS);
            self.complex_sharpness = self
                .complex_sharpness
                .clamp(1, complex_star_max_sharpness(self.points));
        }
        if kind == CoordKind::MultiPoint && self.curve.is_empty() {
            self.curve = vec![self.start, self.end];
            self.current = Some(1);
        }
    }

    pub fn set_width(&mut self, v: f64) {
        if !finite_pos(v) {
            return;
        }
        if self.proportional && !self.kind.is_uniform() && finite_pos(self.w) {
            self.h *= v / self.w;
        }
        self.w = v;
    }

    pub fn set_height(&mut self, v: f64) {
        if !finite_pos(v) {
            return;
        }
        if self.proportional && !self.kind.is_uniform() && finite_pos(self.h) {
            self.w *= v / self.h;
        }
        self.h = v;
    }

    /// A regular polygon's side: its points lie on a circle of diameter
    /// side / sin(180° / points).
    pub fn side_length(&self) -> f64 {
        let n = self.points.max(3) as f64;
        self.w * (std::f64::consts::PI / n).sin()
    }

    pub fn set_side_length(&mut self, s: f64) {
        let n = self.points.max(3) as f64;
        self.set_width(s / (std::f64::consts::PI / n).sin());
    }

    pub fn line_length(&self) -> f64 {
        (self.end - self.start).hypot()
    }

    /// A line's angle from its start to its end.
    pub fn line_angle(&self) -> f64 {
        let d = self.end - self.start;
        if d.hypot() < 1e-12 {
            return 0.0;
        }
        norm_deg(d.y.atan2(d.x).to_degrees())
    }

    pub fn set_line_length(&mut self, l: f64) {
        if !l.is_finite() || l < 0.0 {
            return;
        }
        let a = self.line_angle().to_radians();
        self.end = self.start + l * tracedraw_core::geometry::Vec2::new(a.cos(), a.sin());
    }

    pub fn set_line_angle(&mut self, deg: f64) {
        if !deg.is_finite() {
            return;
        }
        let l = self.line_length();
        let a = deg.to_radians();
        self.end = self.start + l * tracedraw_core::geometry::Vec2::new(a.cos(), a.sin());
    }

    /// The box's lower-left and upper-right corners, when it is not
    /// rotated.
    pub fn bounding_box(&self) -> Option<Rect> {
        if !self.kind.is_boxed() || self.angle.abs() > 1e-9 {
            return None;
        }
        let (w, h) = self.size();
        let local = Rect::new(0.0, 0.0, w, h);
        let anchor = origin_in(local, self.origin);
        let x0 = self.at.x - anchor.x;
        let y0 = self.at.y - anchor.y;
        Some(Rect::new(x0, y0, x0 + w, y0 + h))
    }

    /// Set the box from its two corners (not rotated).
    pub fn set_bounding_box(&mut self, lower_left: Point, upper_right: Point) {
        let r = Rect::from_points(lower_left, upper_right);
        if !finite_pos(r.width()) || !finite_pos(r.height()) {
            return;
        }
        self.angle = 0.0;
        self.w = r.width();
        self.h = if self.kind.is_uniform() {
            r.width()
        } else {
            r.height()
        };
        let local = Rect::new(0.0, 0.0, self.w, self.h);
        let anchor = origin_in(local, self.origin);
        self.at = Point::new(r.x0 + anchor.x, r.y0 + anchor.y);
    }

    /// Choose another origin point: the object stays where it is.
    pub fn set_origin(&mut self, i: usize) {
        if !self.kind.is_boxed() {
            self.origin = i.min(8);
            return;
        }
        let (w, h) = self.size();
        let local = Rect::new(0.0, 0.0, w, h);
        let t = self.placement(local);
        self.origin = i.min(8);
        self.at = t * origin_in(local, self.origin);
    }

    /// Where a box of this size goes: rotated about its origin point,
    /// which sits on `at`.
    fn placement(&self, local: Rect) -> Affine {
        let anchor = origin_in(local, self.origin);
        Affine::translate(self.at.to_vec2())
            * Affine::rotate(self.angle.to_radians())
            * Affine::translate(-anchor.to_vec2())
    }

    /// The object the fields describe, its kind and transform; `None`
    /// while it has no size.
    pub fn shape_kind(&self) -> Option<(ShapeKind, Affine)> {
        let (w, h) = self.size();
        match self.kind {
            CoordKind::Rectangle | CoordKind::Square | CoordKind::Ellipse | CoordKind::Circle => {
                if !finite_pos(w) || !finite_pos(h) {
                    return None;
                }
                let rect = Rect::new(0.0, 0.0, w, h);
                let kind = if matches!(self.kind, CoordKind::Rectangle | CoordKind::Square) {
                    ShapeKind::Rect {
                        rect,
                        radius: 0.0,
                        corners: None,
                    }
                } else {
                    ShapeKind::Ellipse { rect, arc: None }
                };
                Some((kind, self.placement(rect)))
            }
            CoordKind::Polygon
            | CoordKind::RegularPolygon
            | CoordKind::Star
            | CoordKind::ComplexStar => {
                if !finite_pos(w) || !finite_pos(h) {
                    return None;
                }
                let rect = Rect::new(-w / 2.0, -h / 2.0, w / 2.0, h / 2.0);
                let t =
                    Affine::translate(self.at.to_vec2()) * Affine::rotate(self.angle.to_radians());
                let kind = match self.kind {
                    CoordKind::Star => ShapeKind::Polygon {
                        rect,
                        points: self.points.clamp(3, 500),
                        sharpness: crate::ui::propbar::sharpness_from_pct(
                            self.sharpness * 100.0,
                            self.sharpness,
                        ),
                        complex: None,
                        twist: 0.0,
                    },
                    CoordKind::ComplexStar => {
                        let n = self.points.clamp(COMPLEX_STAR_MIN_POINTS, 500);
                        ShapeKind::Polygon {
                            rect,
                            points: n,
                            sharpness: 0.0,
                            complex: Some(
                                self.complex_sharpness
                                    .clamp(1, complex_star_max_sharpness(n)),
                            ),
                            twist: 0.0,
                        }
                    }
                    _ => ShapeKind::Polygon {
                        rect,
                        points: self.points.clamp(3, 500),
                        sharpness: 0.0,
                        complex: None,
                        twist: 0.0,
                    },
                };
                Some((kind, t))
            }
            CoordKind::Line => {
                if self.line_length() < 1e-9 {
                    return None;
                }
                Some((
                    ShapeKind::Path {
                        path: polyline_path(&[self.start, self.end], false),
                        closed: false,
                    },
                    Affine::IDENTITY,
                ))
            }
            CoordKind::MultiPoint => {
                if self.curve.len() < 2 {
                    return None;
                }
                let closed = self.closed && self.curve.len() > 2;
                Some((
                    ShapeKind::Path {
                        path: self.curve_path(closed),
                        closed,
                    },
                    Affine::IDENTITY,
                ))
            }
        }
    }

    /// The outline the preview shows, page space.
    pub fn preview(&self) -> Option<BezPath> {
        let (kind, t) = self.shape_kind()?;
        let mut s = Shape::new(ShapeId(0), kind);
        s.transform = t;
        Some(s.page_path())
    }

    /// The point the preview marks: the origin point, a polygon's centre,
    /// a line's start, the chosen point of a curve.
    pub fn marked_point(&self) -> Option<Point> {
        match self.kind {
            CoordKind::Line => Some(self.start),
            CoordKind::MultiPoint => self.current.and_then(|i| self.curve.get(i).copied()),
            _ => Some(self.at),
        }
    }

    /// A drag from `a` to `b` for a drag pick.
    pub fn apply_drag(&mut self, pick: CoordPick, a: Point, b: Point) {
        let d = b - a;
        let len = d.hypot();
        let ang = norm_deg(d.y.atan2(d.x).to_degrees());
        match pick {
            CoordPick::Width => self.set_width(len),
            CoordPick::Height => self.set_height(len),
            CoordPick::Angle => {
                if len > 1e-9 {
                    if self.kind == CoordKind::Line {
                        self.set_line_angle(ang);
                    } else {
                        self.angle = ang;
                    }
                }
            }
            CoordPick::SideLength => {
                if finite_pos(len) {
                    self.set_side_length(len);
                }
            }
            CoordPick::LineLength => self.set_line_length(len),
            _ => self.apply_point(pick, b),
        }
    }

    /// A click at `p` for a point pick.
    pub fn apply_point(&mut self, pick: CoordPick, p: Point) {
        if !p.x.is_finite() || !p.y.is_finite() {
            return;
        }
        match pick {
            CoordPick::Origin => self.at = p,
            CoordPick::LowerLeft => {
                if let Some(b) = self.bounding_box() {
                    self.set_bounding_box(p, Point::new(b.x1, b.y1));
                }
            }
            CoordPick::UpperRight => {
                if let Some(b) = self.bounding_box() {
                    self.set_bounding_box(Point::new(b.x0, b.y0), p);
                }
            }
            CoordPick::Start => self.start = p,
            CoordPick::End => self.end = p,
            CoordPick::Point => match self.current {
                Some(i) if i < self.curve.len() => self.set_curve_point(i, p),
                _ => {
                    self.curve.push(p);
                    self.current = Some(self.curve.len() - 1);
                }
            },
            _ => {}
        }
    }

    /// Add a point after the chosen one (at the end when none is), a
    /// little past it, and choose it.
    pub fn add_point(&mut self) {
        let at = match self.current {
            Some(i) if i < self.curve.len() => i + 1,
            _ => self.curve.len(),
        };
        let p = match (
            at.checked_sub(1).and_then(|i| self.curve.get(i)),
            at.checked_sub(2).and_then(|i| self.curve.get(i)),
        ) {
            (Some(a), Some(b)) => *a + (*a - *b),
            (Some(a), None) => *a + tracedraw_core::geometry::Vec2::new(10.0, 0.0),
            _ => self.at,
        };
        self.curve.insert(at, p);
        if at <= self.curve_ctrl.len() {
            self.curve_ctrl.insert(at, None);
        }
        self.curve_el.clear();
        self.current = Some(at);
    }

    /// The control points of the segment ending at point `i`.
    fn ctrl(&self, i: usize) -> Option<(Point, Point)> {
        self.curve_ctrl.get(i).copied().flatten()
    }

    /// The multipoint curve's outline: straight segments, and the bends
    /// of a curve read from the drawing.
    fn curve_path(&self, closed: bool) -> BezPath {
        if self.curve_ctrl.iter().all(Option::is_none) {
            return polyline_path(&self.curve, closed);
        }
        let mut path = BezPath::new();
        for (i, p) in self.curve.iter().enumerate() {
            match (i, self.ctrl(i)) {
                (0, _) => path.move_to(*p),
                (_, Some((c1, c2))) => path.curve_to(c1, c2, *p),
                (_, None) => path.line_to(*p),
            }
        }
        if closed {
            if let (Some((c1, c2)), Some(first)) = (self.ctrl(0), self.curve.first()) {
                path.curve_to(c1, c2, *first);
            }
            path.close_path();
        }
        path
    }

    /// Move point `i` to `p`; the bends next to it move along, as a node
    /// moved on the drawing takes its handles.
    pub fn set_curve_point(&mut self, i: usize, p: Point) {
        let Some(old) = self.curve.get(i).copied() else {
            return;
        };
        let d = p - old;
        self.curve[i] = p;
        let n = self.curve.len();
        if let Some(Some((_, c2))) = self.curve_ctrl.get_mut(i) {
            *c2 += d;
        }
        let next = if i + 1 < n { i + 1 } else { 0 };
        if next != i {
            if let Some(Some((c1, _))) = self.curve_ctrl.get_mut(next) {
                *c1 += d;
            }
        }
    }

    /// Delete the chosen point; the next one (or the last) is chosen.
    pub fn delete_point(&mut self) {
        let Some(i) = self.current else {
            return;
        };
        if i >= self.curve.len() {
            self.current = None;
            return;
        }
        self.curve.remove(i);
        if i < self.curve_ctrl.len() {
            self.curve_ctrl.remove(i);
        }
        self.curve_el.clear();
        self.current = if self.curve.is_empty() {
            None
        } else {
            Some(i.min(self.curve.len() - 1))
        };
    }

    /// Read the fields from an object; false when the docker cannot draw
    /// its kind (curves need one subpath; their bends are kept).
    pub fn load(&mut self, s: &Shape) -> bool {
        let t = s.transform;
        let (sx, sy) = axis_scale(t);
        let [a, b, ..] = t.as_coeffs();
        let angle = norm_deg(b.atan2(a).to_degrees());
        let same = |w: f64, h: f64| (w - h).abs() <= 1e-6 * w.abs().max(h.abs()).max(1.0);
        match &s.kind {
            ShapeKind::Rect { rect, .. } | ShapeKind::Ellipse { rect, arc: None } => {
                let is_rect = matches!(s.kind, ShapeKind::Rect { .. });
                let (w, h) = (rect.width() * sx, rect.height() * sy);
                let uniform_now = self.kind.is_uniform();
                self.kind = match (is_rect, uniform_now && same(w, h)) {
                    (true, true) => CoordKind::Square,
                    (true, false) => CoordKind::Rectangle,
                    (false, true) => CoordKind::Circle,
                    (false, false) => CoordKind::Ellipse,
                };
                self.w = w;
                self.h = h;
                self.angle = angle;
                self.at = t * origin_in(*rect, self.origin);
            }
            ShapeKind::Polygon {
                rect,
                points,
                sharpness,
                complex,
                ..
            } => {
                let (w, h) = (rect.width() * sx, rect.height() * sy);
                self.kind = match complex {
                    Some(k) => {
                        self.complex_sharpness = *k;
                        CoordKind::ComplexStar
                    }
                    None if tracedraw_core::geometry::is_star_sharpness(*sharpness) => {
                        self.sharpness = *sharpness;
                        CoordKind::Star
                    }
                    None if self.kind == CoordKind::RegularPolygon && same(w, h) => {
                        CoordKind::RegularPolygon
                    }
                    None => CoordKind::Polygon,
                };
                self.points = *points;
                self.w = w;
                self.h = h;
                self.angle = angle;
                self.at = t * rect.center();
            }
            ShapeKind::Path { path, closed } => {
                let mut pts = Vec::new();
                let mut ctrl: Vec<Option<(Point, Point)>> = Vec::new();
                let mut els = Vec::new();
                let mut ends = false;
                let mut last = Point::ZERO;
                for (i, el) in path.elements().iter().enumerate() {
                    // One subpath only: nothing after its end.
                    if ends {
                        return false;
                    }
                    match el {
                        PathEl::MoveTo(p) if i == 0 => {
                            pts.push(t * *p);
                            ctrl.push(None);
                            last = *p;
                        }
                        PathEl::LineTo(p) => {
                            pts.push(t * *p);
                            ctrl.push(None);
                            last = *p;
                        }
                        PathEl::QuadTo(q, p) => {
                            let c1 = last + (*q - last) * (2.0 / 3.0);
                            let c2 = *p + (*q - *p) * (2.0 / 3.0);
                            pts.push(t * *p);
                            ctrl.push(Some((t * c1, t * c2)));
                            last = *p;
                        }
                        PathEl::CurveTo(c1, c2, p) => {
                            pts.push(t * *p);
                            ctrl.push(Some((t * *c1, t * *c2)));
                            last = *p;
                        }
                        PathEl::ClosePath => ends = true,
                        PathEl::MoveTo(_) => return false,
                    }
                    if !matches!(el, PathEl::ClosePath) {
                        els.push(i);
                    }
                }
                if pts.len() < 2 {
                    return false;
                }
                let closed = *closed || ends;
                if closed && pts.len() > 2 && (pts[0] - pts[pts.len() - 1]).hypot() < 1e-9 {
                    // The closing segment ends at the first point.
                    pts.pop();
                    els.pop();
                    if let Some(c) = ctrl.pop() {
                        ctrl[0] = c;
                    }
                }
                if pts.len() == 2 && !closed && ctrl.iter().all(Option::is_none) {
                    self.kind = CoordKind::Line;
                    self.start = pts[0];
                    self.end = pts[1];
                } else {
                    self.kind = CoordKind::MultiPoint;
                    self.current = Some(pts.len() - 1);
                    self.curve = pts;
                    self.curve_ctrl = ctrl;
                    self.curve_el = els;
                    self.closed = closed;
                }
            }
            _ => return false,
        }
        self.loaded = Some(s.id);
        true
    }

    /// Choose the point of the node at element `el` of the curve the
    /// fields were read from.
    pub fn choose_node(&mut self, el: usize) -> bool {
        if self.kind != CoordKind::MultiPoint {
            return false;
        }
        let Some(i) = self.curve_el.iter().position(|e| *e == el) else {
            return false;
        };
        self.current = Some(i);
        true
    }
}

impl App {
    /// The single selected object the docker can replace.
    pub fn coords_target(&self) -> Option<Shape> {
        let [id] = self.selection.as_slice() else {
            return None;
        };
        let s = self.doc().find_shape(*id)?.clone();
        matches!(
            s.kind,
            ShapeKind::Rect { .. }
                | ShapeKind::Ellipse { .. }
                | ShapeKind::Polygon { .. }
                | ShapeKind::Path { .. }
        )
        .then_some(s)
    }

    /// Read the fields from a newly selected object, again when it was
    /// changed on the drawing, and choose the point of a node chosen on
    /// the drawing (#1889).
    pub fn coords_follow_selection(&mut self) {
        let Some(s) = self.coords_target() else {
            self.coords.loaded = None;
            self.coords.loaded_from = None;
            return;
        };
        let now = (s.kind.clone(), s.transform);
        if self.coords.loaded != Some(s.id) || self.coords.loaded_from.as_ref() != Some(&now) {
            let mut c = self.coords.clone();
            if c.load(&s) {
                self.coords = c;
            } else {
                self.coords.loaded = Some(s.id);
            }
            self.coords.loaded_from = Some(now);
            self.coords.followed_node = None;
        }
        let node = match self.node_selection.as_slice() {
            [(id, el)] if *id == s.id => Some((*id, *el)),
            _ => None,
        };
        if node != self.coords.followed_node {
            self.coords.followed_node = node;
            if let Some((_, el)) = node {
                self.coords.choose_node(el);
            }
        }
    }

    /// Create object: a new object from the fields, selected.
    pub fn create_from_coords(&mut self) -> Option<ShapeId> {
        let (kind, transform) = self.coords.shape_kind()?;
        let layer = self.active_layer()?;
        let id = self.engine.new_shape_id();
        let mut shape = Shape::new(id, kind);
        shape.fill = self.default_fill();
        shape.stroke = self.default_stroke();
        shape.transform = transform;
        if let Err(e) = self
            .engine
            .run_with_label(&Command::AddShape { layer, shape }, "Create Object")
        {
            self.status = e.to_string();
            return None;
        }
        self.select(vec![id]);
        self.coords.loaded = Some(id);
        Some(id)
    }

    /// Replace object: the selected object becomes the one the fields
    /// describe; it keeps its fill, outline and place in the stacking
    /// order, and loses envelopes and perspective. One undo step.
    pub fn replace_from_coords(&mut self) -> bool {
        let Some(old) = self.coords_target() else {
            return false;
        };
        let Some((kind, transform)) = self.coords.shape_kind() else {
            return false;
        };
        let det = old.transform.determinant();
        if !det.is_finite() || det.abs() < 1e-12 {
            return false;
        }
        let mut cmds = vec![
            Command::SetShapeKind {
                shape: old.id,
                kind,
            },
            Command::TransformShapes {
                shapes: vec![old.id],
                transform: transform * old.transform.inverse(),
            },
        ];
        // The effects as they were (the new transform does not carry a
        // symmetry's lines), without envelopes and perspective.
        let raw = self
            .doc()
            .find_shape(old.id)
            .cloned()
            .unwrap_or(old.clone());
        let keep = |e: &Effect| !matches!(e, Effect::Envelope { .. } | Effect::Perspective { .. });
        cmds.push(Command::SetEffectStack {
            shape: old.id,
            effects: raw.effects.iter().filter(|e| keep(e)).cloned().collect(),
            parked: raw.parked_effects.clone(),
        });
        if let Err(e) = self.engine.run_batch("Replace Object", &cmds) {
            self.status = e.to_string();
            return false;
        }
        self.coords.loaded = Some(old.id);
        true
    }

    /// The canvas while a "Set ... interactively" button waits: a click
    /// or a drag sets the value; a right click cancels. True when it took
    /// the input.
    pub fn coords_pick_input(&mut self, response: &Response, p: Point) -> bool {
        let Some(pick) = self.coord_pick else {
            return false;
        };
        if response.secondary_clicked() {
            self.coord_pick = None;
            self.coord_drag = None;
            self.status = tr("status.cancelled");
            return true;
        }
        let p = self.snap_point(p);
        if pick.is_drag() {
            if response.drag_started_by(PointerButton::Primary) {
                self.coord_drag = Some(p);
            }
            if response.drag_stopped() {
                if let Some(a) = self.coord_drag.take() {
                    self.coords.apply_drag(pick, a, p);
                    self.coord_pick = None;
                }
            }
        } else if response.clicked_by(PointerButton::Primary) {
            self.coords.apply_point(pick, p);
            self.coord_pick = None;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Shape as _;

    fn close(a: Point, b: Point) -> bool {
        (a - b).hypot() < 1e-6
    }

    #[test]
    fn a_rectangle_turns_about_its_origin_point() {
        let mut c = CoordsState {
            at: Point::new(10.0, 20.0),
            w: 40.0,
            h: 20.0,
            ..Default::default()
        };
        // The bottom-left origin point stays put.
        c.set_origin(6);
        assert!(close(c.at, Point::new(-10.0, 10.0)), "{:?}", c.at);
        assert_eq!(c.bounding_box(), Some(Rect::new(-10.0, 10.0, 30.0, 30.0)));
        c.angle = 90.0;
        assert_eq!(c.bounding_box(), None);
        let p = c.preview().expect("an outline").bounding_box();
        assert!(
            (p.x0 - -30.0).abs() < 1e-9 && (p.x1 - -10.0).abs() < 1e-9,
            "{p:?}"
        );
        assert!(
            (p.y0 - 10.0).abs() < 1e-9 && (p.y1 - 50.0).abs() < 1e-9,
            "{p:?}"
        );
        // Typed corners set the size and the place again.
        c.angle = 0.0;
        c.set_bounding_box(Point::new(0.0, 0.0), Point::new(30.0, 10.0));
        assert_eq!((c.w, c.h), (30.0, 10.0));
        assert!(close(c.at, Point::new(0.0, 0.0)));
        // Proportional sizes.
        c.proportional = true;
        c.set_width(60.0);
        assert_eq!((c.w, c.h), (60.0, 20.0));
        c.set_height(5.0);
        assert_eq!((c.w, c.h), (15.0, 5.0));
        // Nothing to draw without a size.
        c.w = 0.0;
        assert!(c.shape_kind().is_none());
    }

    #[test]
    fn polygons_stars_and_lines_from_their_fields() {
        let mut c = CoordsState {
            at: Point::new(50.0, 50.0),
            w: 20.0,
            h: 20.0,
            points: 6,
            ..Default::default()
        };
        c.set_kind(CoordKind::RegularPolygon);
        // A hexagon's side is its radius.
        assert!((c.side_length() - 10.0).abs() < 1e-9);
        c.set_side_length(15.0);
        assert!((c.w - 30.0).abs() < 1e-9);
        let (k, t) = c.shape_kind().expect("a polygon");
        assert!(matches!(
            k,
            ShapeKind::Polygon {
                points: 6,
                complex: None,
                ..
            }
        ));
        assert!(close(t * Point::ZERO, Point::new(50.0, 50.0)));
        // A complex star needs five points at least.
        c.points = 3;
        c.complex_sharpness = 9;
        c.set_kind(CoordKind::ComplexStar);
        assert_eq!((c.points, c.complex_sharpness), (5, 1));
        let (k, _) = c.shape_kind().expect("a complex star");
        assert!(matches!(
            k,
            ShapeKind::Polygon {
                complex: Some(1),
                ..
            }
        ));
        // A line: length and angle.
        c.set_kind(CoordKind::Line);
        c.start = Point::new(0.0, 0.0);
        c.end = Point::new(10.0, 0.0);
        c.set_line_angle(90.0);
        assert!(close(c.end, Point::new(0.0, 10.0)));
        c.set_line_length(4.0);
        assert!(close(c.end, Point::new(0.0, 4.0)));
        assert!((c.line_angle() - 90.0).abs() < 1e-9);
        // Interactive picks: a drag sets the angle, a click the end.
        c.apply_drag(CoordPick::Angle, Point::ZERO, Point::new(-3.0, 0.0));
        assert!(close(c.end, Point::new(-4.0, 0.0)), "{:?}", c.end);
        c.apply_point(CoordPick::End, Point::new(1.0, 1.0));
        assert_eq!(c.end, Point::new(1.0, 1.0));
    }

    #[test]
    fn multipoint_curves_add_delete_and_close() {
        let mut c = CoordsState::default();
        c.set_kind(CoordKind::MultiPoint);
        assert_eq!(c.curve.len(), 2);
        c.add_point();
        assert_eq!(c.curve.len(), 3);
        assert_eq!(c.current, Some(2));
        c.apply_point(CoordPick::Point, Point::new(1.0, 2.0));
        assert_eq!(c.curve[2], Point::new(1.0, 2.0));
        c.closed = true;
        let (k, _) = c.shape_kind().expect("a curve");
        assert!(matches!(k, ShapeKind::Path { closed: true, .. }));
        c.delete_point();
        assert_eq!((c.curve.len(), c.current), (2, Some(1)));
        // Two points do not close.
        let (k, _) = c.shape_kind().expect("a curve");
        assert!(matches!(k, ShapeKind::Path { closed: false, .. }));
        c.delete_point();
        c.delete_point();
        assert!(c.curve.is_empty() && c.current.is_none());
        assert!(c.shape_kind().is_none());
        c.delete_point();
    }

    #[test]
    fn objects_load_into_the_fields() {
        let mut c = CoordsState::default();
        let mut s = Shape::new(
            ShapeId(7),
            ShapeKind::Ellipse {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                arc: None,
            },
        );
        s.transform =
            Affine::translate((5.0, 5.0)) * Affine::rotate(30f64.to_radians()) * Affine::scale(2.0);
        assert!(c.load(&s));
        assert_eq!(c.kind, CoordKind::Ellipse);
        assert!((c.w - 20.0).abs() < 1e-9 && (c.h - 20.0).abs() < 1e-9);
        assert!((c.angle - 30.0).abs() < 1e-9);
        // Drawn again from the fields, it is the same outline.
        let a = c.preview().expect("an ellipse");
        let b = s.page_path();
        assert!((a.area() - b.area()).abs() < 1e-6);
        assert!((a.bounding_box().center() - b.bounding_box().center()).hypot() < 1e-6);
        // Straight curves load as lines or multipoint curves; others not.
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((3.0, 4.0));
        let line = Shape::new(
            ShapeId(8),
            ShapeKind::Path {
                path: p.clone(),
                closed: false,
            },
        );
        assert!(c.load(&line));
        assert_eq!((c.kind, c.line_length()), (CoordKind::Line, 5.0));
        p.line_to((6.0, 0.0));
        p.close_path();
        let tri = Shape::new(
            ShapeId(9),
            ShapeKind::Path {
                path: p.clone(),
                closed: true,
            },
        );
        assert!(c.load(&tri));
        assert_eq!(
            (c.kind, c.curve.len(), c.closed),
            (CoordKind::MultiPoint, 3, true)
        );
        p.curve_to((1.0, 1.0), (2.0, 2.0), (3.0, 3.0));
        let curvy = Shape::new(
            ShapeId(10),
            ShapeKind::Path {
                path: p,
                closed: false,
            },
        );
        assert!(!c.load(&curvy));
        let star = Shape::new(
            ShapeId(11),
            ShapeKind::Polygon {
                rect: Rect::new(-5.0, -5.0, 5.0, 5.0),
                points: 9,
                sharpness: 0.0,
                complex: Some(3),
                twist: 0.0,
            },
        );
        assert!(c.load(&star));
        assert_eq!(
            (c.kind, c.points, c.complex_sharpness),
            (CoordKind::ComplexStar, 9, 3)
        );
    }

    /// A curve with bends reads into the multipoint curve's points with
    /// its bends kept: drawn again from the fields it is the same outline;
    /// a point moved takes the bends next to it along; a node chosen on
    /// the drawing chooses its point (#1889).
    #[test]
    fn a_curve_with_bends_loads_and_follows_its_nodes() {
        let mut app = App::headless();
        let mut p = BezPath::new();
        p.move_to((10.0, 10.0));
        p.curve_to((20.0, 0.0), (30.0, 0.0), (40.0, 10.0));
        p.line_to((40.0, 40.0));
        p.curve_to((30.0, 50.0), (20.0, 50.0), (10.0, 10.0));
        p.close_path();
        let id = app
            .new_shape(ShapeKind::Path {
                path: p,
                closed: true,
            })
            .expect("curve");
        app.select(vec![id]);
        app.coords_follow_selection();
        let c = &app.coords;
        assert_eq!(c.kind, CoordKind::MultiPoint);
        assert_eq!(c.curve.len(), 3, "{:?}", c.curve);
        assert!(c.closed);
        assert!(close(c.curve[1], Point::new(40.0, 10.0)));
        let page = app.doc().find_shape(id).expect("shape").page_path();
        let drawn = c.preview().expect("outline");
        assert!((drawn.area() - page.area()).abs() < 1e-6);
        assert!((drawn.bounding_box().center() - page.bounding_box().center()).hypot() < 1e-6);
        // Node 2 (element 2, the corner at 40, 40) chosen on the drawing.
        app.node_selection = vec![(id, 2)];
        app.coords_follow_selection();
        assert_eq!(app.coords.current, Some(2));
        // A click in the list chooses another point; the same node choice
        // does not take it back.
        app.coords.current = Some(0);
        app.coords_follow_selection();
        assert_eq!(app.coords.current, Some(0));
        // The closing segment's end chooses the first point.
        app.node_selection = vec![(id, 3)];
        app.coords_follow_selection();
        assert_eq!(app.coords.current, Some(0));
        // Moving the first point takes its bends along.
        let mut c = app.coords.clone();
        c.set_curve_point(0, Point::new(10.0, 20.0));
        assert_eq!(
            c.curve_ctrl[1],
            Some((Point::new(20.0, 10.0), Point::new(30.0, 0.0)))
        );
        assert_eq!(
            c.curve_ctrl[0],
            Some((Point::new(30.0, 50.0), Point::new(20.0, 60.0)))
        );
        // Replace keeps the bends.
        app.coords = c;
        assert!(app.replace_from_coords());
        let s = app.doc().find_shape(id).expect("shape");
        let ShapeKind::Path { path, .. } = &s.kind else {
            panic!("a curve");
        };
        assert!(path
            .elements()
            .iter()
            .any(|e| matches!(e, PathEl::CurveTo(..))));
        // An edit on the drawing reads the curve again.
        app.run(Command::TransformShapes {
            shapes: vec![id],
            transform: Affine::translate((5.0, 0.0)),
        });
        app.coords_follow_selection();
        assert!(close(app.coords.curve[1], Point::new(45.0, 10.0)));
    }

    #[test]
    fn create_and_replace_are_one_step_each() {
        let mut app = App::headless();
        app.coords = CoordsState {
            at: Point::new(20.0, 20.0),
            w: 10.0,
            h: 10.0,
            ..Default::default()
        };
        let depth = app.engine.history_labels().0.len();
        let id = app.create_from_coords().expect("created");
        assert_eq!(app.selection, vec![id]);
        assert_eq!(app.engine.history_labels().0.len(), depth + 1);
        // Replace it with a circle somewhere else.
        app.coords.set_kind(CoordKind::Circle);
        app.coords.at = Point::new(50.0, 60.0);
        assert!(app.replace_from_coords());
        assert_eq!(app.engine.history_labels().0.len(), depth + 2);
        let s = app.doc().find_shape(id).cloned().expect("still there");
        assert!(matches!(s.kind, ShapeKind::Ellipse { .. }));
        let c = s.bounds().center();
        assert!((c - Point::new(50.0, 60.0)).hypot() < 1e-6, "{c:?}");
        // The fields follow the selection.
        app.coords = CoordsState::default();
        app.coords_follow_selection();
        assert_eq!(app.coords.loaded, Some(id));
        assert!((app.coords.w - 10.0).abs() < 1e-9);
        // Nothing selected: nothing to replace.
        app.select(vec![]);
        assert!(!app.replace_from_coords());
    }

    #[test]
    fn every_object_draws_from_the_defaults() {
        for k in CoordKind::ALL {
            let mut c = CoordsState::default();
            c.set_kind(k);
            let (kind, _) = c.shape_kind().unwrap_or_else(|| panic!("{k:?}"));
            assert!(!Shape::new(ShapeId(1), kind)
                .local_path()
                .elements()
                .is_empty());
            assert!(c.preview().is_some() && c.marked_point().is_some(), "{k:?}");
            assert!(!tr(k.key()).is_empty());
        }
    }

    #[test]
    fn picks_wait_for_a_click_or_a_drag() {
        assert!(CoordPick::Width.is_drag());
        assert!(!CoordPick::Origin.is_drag());
        let mut c = CoordsState::default();
        c.apply_drag(CoordPick::Width, Point::ZERO, Point::new(3.0, 4.0));
        assert_eq!(c.w, 5.0);
        // A drag without length leaves the angle.
        c.angle = 12.0;
        c.apply_drag(CoordPick::Angle, Point::ZERO, Point::ZERO);
        assert_eq!(c.angle, 12.0);
        c.apply_point(CoordPick::Origin, Point::new(f64::NAN, 0.0));
        assert!(c.at.x.is_finite());
    }
}
