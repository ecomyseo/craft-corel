//! 3-point callouts. A callout is a group of its leader (an open path
//! from the tip through the elbow to the text, with a head at the tip),
//! the shape around its text when its style has one, and its text, with
//! the tip, the elbow and the style in the group's object data
//! (`callout`). After every change of the drawing the leader's end and
//! the shape are fitted to the text again, in the same undo step: typed
//! text, a new font size or the text moved on its own take them along,
//! and a tip or an elbow moved with the Shape tool stays where it was
//! put.
//!
//! The text sits on the side of the elbow the third click was on: to its
//! right, left aligned, or to its left, right aligned, so it grows away
//! from the leader. The leader meets the middle of the text's side, the
//! underline's near end, or the middle of the shape's side.

use crate::app::App;
use crate::i18n::tr;
use serde::{Deserialize, Serialize};
use tracedraw_core::{
    document::{Shape, ShapeKind, TextAlign, TextSpan},
    geometry::{ellipse_path, rect_path, Affine, BezPath, PathEl, Point, Rect},
    Arrowhead, Color, Command, Fill, ShapeId, Stroke,
};

/// The object data key holding a callout (on the group).
pub const DATA_KEY: &str = "callout";
/// The object data key naming a part of a callout (on its children).
pub const PART_KEY: &str = "callout.part";
const LEADER: &str = "leader";
const SHAPE: &str = "shape";

/// What surrounds the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalloutShape {
    #[default]
    None,
    Underline,
    Rectangle,
    Rounded,
    Ellipse,
}

impl CalloutShape {
    pub const ALL: [CalloutShape; 5] = [
        CalloutShape::None,
        CalloutShape::Underline,
        CalloutShape::Rectangle,
        CalloutShape::Rounded,
        CalloutShape::Ellipse,
    ];

    pub fn key(self) -> &'static str {
        match self {
            CalloutShape::None => "callout.shape_none",
            CalloutShape::Underline => "callout.shape_underline",
            CalloutShape::Rectangle => "callout.shape_rectangle",
            CalloutShape::Rounded => "callout.shape_rounded",
            CalloutShape::Ellipse => "callout.shape_ellipse",
        }
    }
}

/// How a callout's leader and shape look.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalloutStyle {
    /// Outline width of the leader and the shape, mm.
    pub width: f64,
    /// Dash pattern in multiples of the width; empty for a solid line.
    pub dash: Vec<f64>,
    /// The head at the tip.
    pub head: Arrowhead,
    pub shape: CalloutShape,
    /// Room between the text and the leader's end, the underline or the
    /// shape, mm.
    pub gap: f64,
}

impl Default for CalloutStyle {
    fn default() -> Self {
        CalloutStyle {
            width: Stroke::HAIRLINE,
            dash: Vec::new(),
            head: Arrowhead::Arrow,
            shape: CalloutShape::None,
            gap: 1.0,
        }
    }
}

impl CalloutStyle {
    /// The same style with its numbers in range (a file or a preferences
    /// file edited by hand).
    pub fn clamped(&self) -> CalloutStyle {
        let d = CalloutStyle::default();
        let fin = |v: f64, lo: f64, hi: f64, or: f64| {
            if v.is_finite() {
                v.clamp(lo, hi)
            } else {
                or
            }
        };
        let dash_ok = self.dash.len() <= 16
            && self.dash.iter().all(|v| v.is_finite() && *v >= 0.0)
            && self.dash.iter().sum::<f64>() > 0.0;
        CalloutStyle {
            width: fin(self.width, Stroke::HAIRLINE, 100.0, d.width),
            dash: if dash_ok {
                self.dash.clone()
            } else {
                Vec::new()
            },
            head: self.head.clone(),
            shape: self.shape,
            gap: fin(self.gap, 0.0, 100.0, d.gap),
        }
    }
}

/// A callout: where its leader starts and bends, and how it looks. The
/// points are in the space of the callout's group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Callout {
    pub tip: Point,
    pub elbow: Point,
    #[serde(default)]
    pub style: CalloutStyle,
}

fn finite(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

fn with_data(mut data: Vec<(String, String)>, c: &Callout) -> Vec<(String, String)> {
    data.retain(|(k, _)| k != DATA_KEY);
    if let Ok(v) = serde_json::to_string(c) {
        data.push((DATA_KEY.into(), v));
    }
    data
}

fn with_part(mut data: Vec<(String, String)>, part: &str) -> Vec<(String, String)> {
    data.retain(|(k, _)| k != PART_KEY);
    data.push((PART_KEY.into(), part.into()));
    data
}

fn part_of(s: &Shape) -> Option<&str> {
    s.data
        .iter()
        .find(|(k, _)| k == PART_KEY)
        .map(|(_, v)| v.as_str())
}

/// The on-curve points of a path object, in its parent's space.
fn points_of(s: &Shape) -> Vec<Point> {
    let ShapeKind::Path { path, .. } = &s.kind else {
        return Vec::new();
    };
    path.elements()
        .iter()
        .filter_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            PathEl::QuadTo(_, p) => Some(*p),
            PathEl::CurveTo(_, _, p) => Some(*p),
            PathEl::ClosePath => None,
        })
        .map(|p| s.transform * p)
        .filter(|p| finite(*p))
        .collect()
}

fn polyline(points: &[Point]) -> BezPath {
    let mut path = BezPath::new();
    for (i, p) in points.iter().enumerate() {
        if i == 0 {
            path.move_to(*p);
        } else {
            path.line_to(*p);
        }
    }
    path
}

/// The box of a text object in its parent's space: its lines, from the
/// left of the first character to the right of the last and from the
/// lowest descent to the highest ascent (an estimate from the size when
/// no font gives outlines).
pub fn text_box(t: &Shape) -> Option<Rect> {
    let (layout, to_parent) = crate::text_editing::layout_of_text_shape(t, Affine::IDENTITY)?;
    let mut b: Option<Rect> = None;
    for l in &layout.lines {
        if l.edges.is_empty() {
            continue;
        }
        // The line's left and right ends (in right-to-left text the
        // edges run the other way, #629).
        let (x0, x1) = l.x_range();
        let r = Rect::new(x0, l.baseline - l.descent, x1, l.baseline + l.ascent);
        b = Some(b.map_or(r, |b| b.union(r)));
    }
    let ok = |r: &Rect| {
        r.x0.is_finite()
            && r.y0.is_finite()
            && r.x1.is_finite()
            && r.y1.is_finite()
            && r.height() > 0.0
    };
    let b = match b.filter(ok) {
        Some(b) => b,
        None => {
            // No layout: from the size and the longest line.
            let ShapeKind::Text { spans, align, .. } = &t.kind else {
                return None;
            };
            let size = spans
                .first()
                .map(|s| s.size_pt)
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(12.0);
            let h = size * 25.4 / 72.0;
            let text = tracedraw_core::spans_text(spans);
            let lines = text.split('\n').count().max(1) as f64;
            let longest = text
                .split('\n')
                .map(|l| l.chars().count())
                .max()
                .unwrap_or(0) as f64;
            let w = h * 0.55 * longest;
            let x0 = match align {
                TextAlign::Right => -w,
                TextAlign::Center => -w / 2.0,
                _ => 0.0,
            };
            Rect::new(x0, -0.25 * h - (lines - 1.0) * 1.2 * h, x0 + w, 0.75 * h)
        }
    };
    let r = to_parent.transform_rect_bbox(b);
    ok(&r).then_some(r)
}

impl Callout {
    /// The callout a shape holds (a group made by the 3-point callout
    /// tool).
    pub fn of(shape: &Shape) -> Option<Callout> {
        if !matches!(shape.kind, ShapeKind::Group { .. }) {
            return None;
        }
        let (_, v) = shape.data.iter().find(|(k, _)| k == DATA_KEY)?;
        let mut c: Callout = serde_json::from_str(v).ok()?;
        if !(finite(c.tip) && finite(c.elbow)) {
            return None;
        }
        c.style = c.style.clamped();
        Some(c)
    }

    /// The leader's points and the shape's outline around a text box `tb`
    /// lying to the right of the elbow (`right`) or to its left.
    pub fn parts_on(&self, tb: Rect, right: bool) -> (Vec<Point>, Option<BezPath>) {
        let g = self.style.gap.max(0.0);
        let mut leader = vec![self.tip, self.elbow];
        let shape = match self.style.shape {
            CalloutShape::None => {
                let x = if right { tb.x0 - g } else { tb.x1 + g };
                leader.push(Point::new(x, tb.center().y));
                None
            }
            CalloutShape::Underline => {
                let y = tb.y0 - g;
                let (near, far) = if right {
                    (tb.x0 - g, tb.x1 + g)
                } else {
                    (tb.x1 + g, tb.x0 - g)
                };
                leader.push(Point::new(near, y));
                leader.push(Point::new(far, y));
                None
            }
            CalloutShape::Rectangle | CalloutShape::Rounded => {
                let fb = tb.inflate(g, g);
                let x = if right { fb.x0 } else { fb.x1 };
                leader.push(Point::new(x, fb.center().y));
                let radius = if self.style.shape == CalloutShape::Rounded {
                    (fb.height().min(fb.width()) * 0.3).min(3.0)
                } else {
                    0.0
                };
                Some(rect_path(fb, radius))
            }
            CalloutShape::Ellipse => {
                // The ellipse through the corners of the padded box.
                let fb = tb.inflate(g, g);
                let c = fb.center();
                let (a, b) = (
                    fb.width() / 2.0 * std::f64::consts::SQRT_2,
                    fb.height() / 2.0 * std::f64::consts::SQRT_2,
                );
                let x = if right { c.x - a } else { c.x + a };
                leader.push(Point::new(x, c.y));
                Some(ellipse_path(Rect::new(c.x - a, c.y - b, c.x + a, c.y + b)))
            }
        };
        (leader, shape)
    }

    /// The parts around a text box, on the side of the elbow it is on.
    pub fn parts(&self, tb: Rect) -> (Vec<Point>, Option<BezPath>) {
        self.parts_on(tb, tb.center().x >= self.elbow.x)
    }

    /// The group's children fitted to its text: the leader (its tip and
    /// elbow taken from where the leader has them now), the shape when
    /// the style has one, any other children, then the text. `None` when
    /// the group has no leader or no text to fit.
    pub fn fitted(
        &self,
        children: &[Shape],
        new_id: &mut dyn FnMut() -> ShapeId,
    ) -> Option<(Callout, Vec<Shape>)> {
        let leader_at = children
            .iter()
            .position(|s| part_of(s) == Some(LEADER))
            .or_else(|| {
                children
                    .first()
                    .filter(|s| matches!(s.kind, ShapeKind::Path { .. }))
                    .map(|_| 0)
            })?;
        let text_at = children
            .iter()
            .position(|s| matches!(s.kind, ShapeKind::Text { .. }))?;
        let shape_at = children
            .iter()
            .enumerate()
            .position(|(i, s)| i != leader_at && part_of(s) == Some(SHAPE));
        let mut c = self.clone();
        let pts = points_of(&children[leader_at]);
        if let [tip, elbow, ..] = pts[..] {
            c.tip = tip;
            c.elbow = elbow;
        }
        let tb = text_box(&children[text_at])?;
        let (points, outline) = c.parts(tb);
        let mut leader = children[leader_at].clone();
        leader.kind = ShapeKind::Path {
            path: polyline(&points),
            closed: false,
        };
        leader.transform = Affine::IDENTITY;
        leader.data = with_part(leader.data, LEADER);
        let mut out = vec![leader.clone()];
        if let Some(outline) = outline {
            let mut s = match shape_at {
                Some(i) => children[i].clone(),
                None => {
                    // A new shape takes the leader's outline, without heads.
                    let mut s = Shape::new(
                        new_id(),
                        ShapeKind::Path {
                            path: outline.clone(),
                            closed: true,
                        },
                    );
                    s.fill = Fill::None;
                    s.stroke = leader.stroke.clone().map(|st| Stroke {
                        start_arrow: Arrowhead::None,
                        end_arrow: Arrowhead::None,
                        ..st
                    });
                    s
                }
            };
            s.kind = ShapeKind::Path {
                path: outline,
                closed: true,
            };
            s.transform = Affine::IDENTITY;
            s.data = with_part(s.data, SHAPE);
            out.push(s);
        }
        for (i, s) in children.iter().enumerate() {
            if i != leader_at && i != text_at && Some(i) != shape_at {
                out.push(s.clone());
            }
        }
        out.push(children[text_at].clone());
        Some((c, out))
    }
}

/// The stroke a leader of `style` has, from `base` (its colour and the
/// rest are kept).
fn leader_stroke(base: Option<Stroke>, style: &CalloutStyle) -> Stroke {
    let base = base.unwrap_or_else(|| Stroke::hairline(Color::BLACK));
    Stroke {
        width: style.width,
        dash: style.dash.clone(),
        start_arrow: style.head.clone(),
        end_arrow: Arrowhead::None,
        ..base
    }
}

impl App {
    /// A 3-point callout: the leader from `tip` through `elbow` to `at`,
    /// the text beside `at` on the side of the elbow it is on, ready for
    /// typing (its sample text selected). One undo step.
    pub fn create_callout(&mut self, tip: Point, elbow: Point, at: Point) {
        let Some(layer) = self.active_layer() else {
            return;
        };
        if !(finite(tip) && finite(elbow) && finite(at)) {
            return;
        }
        let c = Callout {
            tip,
            elbow,
            style: self.callout_style.clamped(),
        };
        let right = at.x >= elbow.x;
        let sample = tr("tool.callout_text");
        let tid = self.engine.new_shape_id();
        let mut text = Shape::new(
            tid,
            ShapeKind::Text {
                spans: vec![TextSpan {
                    bold: self.text_bold,
                    italic: self.text_italic,
                    ..TextSpan::new(sample.clone(), self.text_font.clone(), self.text_size_pt)
                }],
                origin: Point::ZERO,
                frame: None,
                align: if right {
                    TextAlign::Left
                } else {
                    TextAlign::Right
                },
                para: Default::default(),
                on_path: None,
            },
        );
        text.fill = Fill::Solid(Color::BLACK);
        text.stroke = None;
        // Where the leader would end with the text at the origin, and the
        // shift that puts that end on `at`.
        if let Some(tb) = text_box(&text) {
            let (points, _) = c.parts_on(tb, right);
            if let Some(end) = points.get(2) {
                if let ShapeKind::Text { origin, .. } = &mut text.kind {
                    *origin = Point::ZERO + (at - *end);
                }
            }
        }
        let lid = self.engine.new_shape_id();
        let mut leader = Shape::new(
            lid,
            ShapeKind::Path {
                path: polyline(&[tip, elbow, at]),
                closed: false,
            },
        );
        leader.fill = Fill::None;
        leader.stroke = Some(leader_stroke(None, &c.style));
        leader.data = with_part(Vec::new(), LEADER);
        let (c, children) = {
            let engine = &mut self.engine;
            let mut new_id = || engine.new_shape_id();
            match c.fitted(&[leader.clone(), text.clone()], &mut new_id) {
                Some(f) => f,
                None => (c.clone(), vec![leader, text]),
            }
        };
        let gid = self.engine.new_shape_id();
        let mut g = Shape::new(gid, ShapeKind::Group { children });
        g.name = Some("Callout".into());
        g.data = with_data(Vec::new(), &c);
        if let Err(e) = self
            .engine
            .run_batch("Callout", &[Command::AddShape { layer, shape: g }])
        {
            self.status = e.to_string();
            return;
        }
        self.begin_text_edit(tid);
        if let Some(te) = self.text_edit.as_mut() {
            te.anchor = 0;
            te.caret = sample.chars().count();
        }
    }

    /// The callouts the selection is or is in (a callout's text being
    /// typed counts as its callout), each once.
    pub fn selected_callouts(&self) -> Vec<(ShapeId, Callout)> {
        let mut out: Vec<(ShapeId, Callout)> = Vec::new();
        for id in &self.selection {
            let mut candidates = vec![*id];
            if let Some(parent) = self.doc().ancestors(*id).last() {
                candidates.push(*parent);
            }
            for cid in candidates {
                if out.iter().any(|(o, _)| *o == cid) {
                    continue;
                }
                if let Some(c) = self.doc().find_shape(cid).and_then(Callout::of) {
                    out.push((cid, c));
                    break;
                }
            }
        }
        out
    }

    /// The style the bar shows: the first selected callout's, else the one
    /// new callouts take.
    pub fn callout_style_shown(&self) -> CalloutStyle {
        self.selected_callouts()
            .into_iter()
            .next()
            .map(|(_, c)| c.style)
            .unwrap_or_else(|| self.callout_style.clamped())
    }

    /// Change the style new callouts take and the selected ones have; one
    /// undo step for the selected ones. Only what changed is applied to
    /// their outlines, so a leader's colour stays.
    pub fn set_callout_style(&mut self, f: impl Fn(&mut CalloutStyle)) {
        f(&mut self.callout_style);
        self.callout_style = self.callout_style.clamped();
        let mut cmds = Vec::new();
        for (id, c) in self.selected_callouts() {
            let mut style = c.style.clone();
            f(&mut style);
            let style = style.clamped();
            if style == c.style {
                continue;
            }
            let Some(ShapeKind::Group { children }) =
                self.doc().find_shape(id).map(|g| g.kind.clone())
            else {
                continue;
            };
            let Some(data) = self.doc().find_shape(id).map(|g| g.data.clone()) else {
                continue;
            };
            let restyle = |st: Option<Stroke>, heads: bool| {
                let mut st = st.unwrap_or_else(|| Stroke::hairline(Color::BLACK));
                if style.width != c.style.width {
                    st.width = style.width;
                }
                if style.dash != c.style.dash {
                    st.dash = style.dash.clone();
                }
                if heads && style.head != c.style.head {
                    st.start_arrow = style.head.clone();
                }
                Some(st)
            };
            let children: Vec<Shape> = children
                .into_iter()
                .enumerate()
                .map(|(i, mut s)| {
                    match part_of(&s) {
                        Some(LEADER) => s.stroke = restyle(s.stroke.clone(), true),
                        Some(SHAPE) => s.stroke = restyle(s.stroke.clone(), false),
                        None if i == 0 && matches!(s.kind, ShapeKind::Path { .. }) => {
                            s.stroke = restyle(s.stroke.clone(), true)
                        }
                        _ => {}
                    }
                    s
                })
                .collect();
            let next = Callout { style, ..c.clone() };
            let engine = &mut self.engine;
            let mut new_id = || engine.new_shape_id();
            let (next, children) = next
                .fitted(&children, &mut new_id)
                .unwrap_or((next.clone(), children));
            cmds.push(Command::SetShapeKind {
                shape: id,
                kind: ShapeKind::Group { children },
            });
            cmds.push(Command::SetObjectData {
                shape: id,
                data: with_data(data, &next),
            });
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Callout Style", &cmds) {
            self.status = e.to_string();
        }
    }

    /// Callouts follow their text (see the module notes). Runs once per
    /// change of the drawing; the changes join the step that made it.
    pub fn refresh_callouts(&mut self) {
        let revision = self.engine.revision();
        if revision == self.callouts_checked {
            return;
        }
        fn walk(s: &Shape, out: &mut Vec<Shape>) {
            if Callout::of(s).is_some() {
                out.push(s.clone());
            }
            match &s.kind {
                ShapeKind::Group { children } => children.iter().for_each(|c| walk(c, out)),
                ShapeKind::ClipFrame { frame, contents } => {
                    walk(frame, out);
                    contents.iter().for_each(|c| walk(c, out));
                }
                _ => {}
            }
        }
        let mut groups = Vec::new();
        for layer in self.doc().all_layers() {
            for s in &layer.shapes {
                walk(s, &mut groups);
            }
        }
        for g in groups {
            let (Some(c), ShapeKind::Group { children }) = (Callout::of(&g), &g.kind) else {
                continue;
            };
            let engine = &mut self.engine;
            let mut new_id = || engine.new_shape_id();
            let Some((next, fitted)) = c.fitted(children, &mut new_id) else {
                continue;
            };
            if fitted == *children && next == c {
                continue;
            }
            let cmds = [
                Command::SetShapeKind {
                    shape: g.id,
                    kind: ShapeKind::Group { children: fitted },
                },
                Command::SetObjectData {
                    shape: g.id,
                    data: with_data(g.data.clone(), &next),
                },
            ];
            for cmd in &cmds {
                if let Err(e) = self.engine.amend(cmd) {
                    self.status = e.to_string();
                    break;
                }
            }
        }
        self.callouts_checked = self.engine.revision();
    }

    /// The 3-point callout tool's clicks and drags: a drag from the tip to
    /// the elbow (or a click on each), then a click where the text goes.
    pub fn callout_input(&mut self, response: &egui::Response, p: Point) {
        use egui::PointerButton;
        let p = self.snap_point(p);
        if response.drag_started_by(PointerButton::Primary) && self.dimension_points.is_empty() {
            // Where the button went down, not where the drag was seen.
            let origin = response
                .ctx
                .input(|i| i.pointer.press_origin())
                .map(|o| self.snap_point(self.view.to_page(o)))
                .unwrap_or(p);
            self.dimension_points.push(origin);
            self.dimension_dragging = true;
            return;
        }
        if response.drag_stopped() && self.dimension_dragging {
            self.dimension_dragging = false;
            let start = self.dimension_points.first().copied();
            if start.is_some_and(|s| (p - s).hypot() > 1e-6) {
                self.dimension_points.push(p);
            } else {
                self.dimension_points.clear();
            }
            return;
        }
        if response.clicked_by(PointerButton::Primary) {
            self.callout_click(p);
        }
    }

    /// A click of the 3-point callout tool at `p` (page space).
    pub fn callout_click(&mut self, p: Point) {
        self.finish_text();
        self.dimension_points.push(p);
        if let [tip, elbow, at] = self.dimension_points[..] {
            self.dimension_points.clear();
            self.create_callout(tip, elbow, at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Tool;

    /// The box of a callout's text and a character background reach
    /// over the whole of a line mixing directions, Arabic after a Latin
    /// word included, not only to its first run (#629).
    #[test]
    fn text_boxes_and_backgrounds_cover_mixed_direction_text() {
        tracedraw_text::install();
        let fonts = tracedraw_text::fonts();
        // A face without Arabic sets the Arabic word apart, as the default
        // font on Linux does.
        let Some(family) = ["Carlito", "Liberation Sans", "DejaVu Sans"]
            .into_iter()
            .find(|f| fonts.has_family(f))
        else {
            return;
        };
        for t in [
            "Asasa \u{645}\u{631}\u{62D}\u{628}\u{627}",
            "\u{645}\u{631}\u{62D}\u{628}\u{627} Asasa",
        ] {
            let mut span = tracedraw_core::TextSpan::new(t, family, 24.0);
            span.background = Some(tracedraw_core::Fill::Solid(tracedraw_core::Color::rgb8(
                255, 255, 0,
            )));
            let kind = ShapeKind::Text {
                spans: vec![span],
                origin: Point::new(0.0, 0.0),
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            };
            let ink = Shape::new(ShapeId(1), kind.clone()).bounds();
            if ink.width() <= 0.0 {
                continue;
            }
            let tb = text_box(&Shape::new(ShapeId(1), kind.clone())).expect("box");
            assert!(
                tb.x0 <= ink.x0 + 0.5 && tb.x1 >= ink.x1 - 0.5,
                "{t}: {tb:?} {ink:?}"
            );
            let bg = tracedraw_core::document::text_backgrounds(&kind);
            let x0 = bg.iter().map(|(r, _)| r.x0).fold(f64::MAX, f64::min);
            let x1 = bg.iter().map(|(r, _)| r.x1).fold(f64::MIN, f64::max);
            assert!(
                x0 <= ink.x0 + 0.5 && x1 >= ink.x1 - 0.5,
                "{t}: {bg:?} {ink:?}"
            );
        }
    }

    fn parts_of(app: &App, g: ShapeId) -> (Vec<Point>, Option<Shape>, Shape) {
        let Some(ShapeKind::Group { children }) = app.doc().find_shape(g).map(|s| s.kind.clone())
        else {
            panic!("no callout group");
        };
        let leader = children
            .iter()
            .find(|s| part_of(s) == Some(LEADER))
            .cloned()
            .expect("leader");
        let shape = children.iter().find(|s| part_of(s) == Some(SHAPE)).cloned();
        let text = children
            .iter()
            .find(|s| matches!(s.kind, ShapeKind::Text { .. }))
            .cloned()
            .expect("text");
        (points_of(&leader), shape, text)
    }

    fn the_callout(app: &App) -> ShapeId {
        let (id, _) = app
            .doc()
            .all_layers()
            .flat_map(|l| &l.shapes)
            .find_map(|s| Callout::of(s).map(|c| (s.id, c)))
            .expect("a callout");
        id
    }

    fn near(a: Point, b: Point) -> bool {
        (a - b).hypot() < 1e-6
    }

    fn set_text(app: &mut App, id: ShapeId, s: &str) {
        let Some(mut t) = app.doc().find_shape(id).cloned() else {
            panic!("no text");
        };
        if let ShapeKind::Text { spans, .. } = &mut t.kind {
            spans.truncate(1);
            spans[0].text = s.into();
        }
        app.run(Command::SetShapeKind {
            shape: id,
            kind: t.kind,
        });
    }

    #[test]
    fn a_callout_ends_at_its_text_and_follows_it() {
        let mut app = App::headless();
        app.tool = Tool::Callout;
        app.callout_click(Point::new(10.0, 10.0));
        app.callout_click(Point::new(30.0, 30.0));
        app.callout_click(Point::new(50.0, 30.0));
        let g = the_callout(&app);
        let (pts, shape, text) = parts_of(&app, g);
        // Tip, elbow, and the end where the third click was.
        assert_eq!(pts.len(), 3);
        assert!(near(pts[0], Point::new(10.0, 10.0)) && near(pts[1], Point::new(30.0, 30.0)));
        assert!(near(pts[2], Point::new(50.0, 30.0)), "{pts:?}");
        assert!(shape.is_none());
        // The text is to the right of the end, a gap away, its middle on
        // the leader's end; its sample text is selected for typing.
        let tb = text_box(&text).expect("box");
        assert!((tb.x0 - 51.0).abs() < 1e-6, "{tb:?}");
        assert!((tb.center().y - 30.0).abs() < 1e-6);
        let te = app.text_edit.clone().expect("typing");
        assert_eq!(te.shape, text.id);
        assert_eq!(te.selection(), (0, tr("tool.callout_text").chars().count()));
        app.finish_text();
        // One undo step made it.
        let steps = app.engine.history_labels().0.len();
        // A longer text keeps its left side where it was.
        set_text(&mut app, text.id, "A much longer callout text");
        app.refresh_callouts();
        let (pts2, _, text2) = parts_of(&app, g);
        let tb2 = text_box(&text2).expect("box");
        assert!(tb2.width() > tb.width());
        assert!(near(pts2[2], pts[2]));
        // The text moved on its own takes the leader's end along, in the
        // same step.
        let before = app.engine.history_labels().0.len();
        app.run(Command::TransformShapes {
            shapes: vec![text.id],
            transform: Affine::translate((5.0, -8.0)),
        });
        app.refresh_callouts();
        assert_eq!(app.engine.history_labels().0.len(), before + 1);
        let (pts3, _, _) = parts_of(&app, g);
        assert!(near(
            pts3[2],
            pts2[2] + tracedraw_core::geometry::Vec2::new(5.0, -8.0)
        ));
        assert!(near(pts3[1], pts[1]));
        // Undo puts both back and leaves Redo there: the restored drawing
        // needs no fitting.
        app.undo();
        app.refresh_callouts();
        let (pts4, _, _) = parts_of(&app, g);
        assert!(near(pts4[2], pts2[2]));
        assert!(app.engine.redo_label().is_some());
        assert!(steps >= 1);
    }

    #[test]
    fn a_tip_moved_with_the_shape_tool_stays() {
        let mut app = App::headless();
        app.create_callout(
            Point::new(0.0, 0.0),
            Point::new(20.0, 20.0),
            Point::new(40.0, 20.0),
        );
        app.finish_text();
        let g = the_callout(&app);
        let Some(ShapeKind::Group { children }) = app.doc().find_shape(g).map(|s| s.kind.clone())
        else {
            panic!("group");
        };
        let leader = children[0].clone();
        assert_eq!(part_of(&leader), Some(LEADER));
        // The tip and the end dragged: the tip stays, the end goes back to
        // the text.
        let mut pts = points_of(&leader);
        pts[0] = Point::new(-5.0, 3.0);
        pts[2] = Point::new(90.0, 90.0);
        app.run(Command::SetShapeKind {
            shape: leader.id,
            kind: ShapeKind::Path {
                path: polyline(&pts),
                closed: false,
            },
        });
        app.refresh_callouts();
        let (after, _, _) = parts_of(&app, g);
        assert!(near(after[0], Point::new(-5.0, 3.0)));
        assert!(near(after[2], Point::new(40.0, 20.0)), "{after:?}");
        let c = app
            .doc()
            .find_shape(g)
            .and_then(Callout::of)
            .expect("callout");
        assert!(near(c.tip, Point::new(-5.0, 3.0)));
    }

    #[test]
    fn text_left_of_the_elbow_is_right_aligned_and_shapes_fit() {
        let mut app = App::headless();
        app.callout_style.shape = CalloutShape::Rectangle;
        app.callout_style.gap = 2.0;
        app.create_callout(
            Point::new(50.0, 0.0),
            Point::new(40.0, 20.0),
            Point::new(20.0, 20.0),
        );
        app.finish_text();
        let g = the_callout(&app);
        let (pts, shape, text) = parts_of(&app, g);
        let ShapeKind::Text { align, .. } = &text.kind else {
            panic!("text");
        };
        assert_eq!(*align, TextAlign::Right);
        // The rectangle is 2 mm round the text, the leader meets the
        // middle of its right side at the third click.
        let tb = text_box(&text).expect("box");
        let frame = shape.expect("frame").local_path();
        let fb = tracedraw_core::geometry::Shape::bounding_box(&frame);
        assert!((fb.x1 - (tb.x1 + 2.0)).abs() < 1e-6 && (fb.y0 - (tb.y0 - 2.0)).abs() < 1e-6);
        assert!(near(pts[2], Point::new(20.0, 20.0)), "{pts:?} {fb:?}");
        // Style changes go to the selected callout, one step: an ellipse,
        // a wider dashed outline, a round head.
        app.select(vec![g]);
        let before = app.engine.history_labels().0.len();
        app.set_callout_style(|s| {
            s.shape = CalloutShape::Ellipse;
            s.width = 0.5;
            s.dash = vec![4.0, 2.0];
            s.head = Arrowhead::Circle;
        });
        assert_eq!(app.engine.history_labels().0.len(), before + 1);
        let (pts2, shape2, text2) = parts_of(&app, g);
        let shape2 = shape2.expect("ellipse");
        let eb = tracedraw_core::geometry::Shape::bounding_box(&shape2.local_path());
        let tb2 = text_box(&text2).expect("box");
        assert!(eb.width() > tb2.width() + 4.0);
        assert!((pts2[2].x - eb.x1).abs() < 1e-3, "{pts2:?} {eb:?}");
        let Some(ShapeKind::Group { children }) = app.doc().find_shape(g).map(|s| s.kind.clone())
        else {
            panic!("group");
        };
        let st = children[0].stroke.clone().expect("leader outline");
        assert_eq!((st.width, st.dash.clone()), (0.5, vec![4.0, 2.0]));
        assert_eq!(st.start_arrow, Arrowhead::Circle);
        let sst = shape2.stroke.clone().expect("shape outline");
        assert_eq!(sst.start_arrow, Arrowhead::None);
        assert_eq!(sst.width, 0.5);
        // None takes the shape away; Underline runs under the text.
        app.set_callout_style(|s| s.shape = CalloutShape::Underline);
        let (pts3, shape3, text3) = parts_of(&app, g);
        assert!(shape3.is_none());
        let tb3 = text_box(&text3).expect("box");
        assert_eq!(pts3.len(), 4);
        assert!((pts3[2].y - (tb3.y0 - 2.0)).abs() < 1e-6);
        assert!((pts3[3].x - (tb3.x0 - 2.0)).abs() < 1e-6);
        // The tool keeps the last style for the next callout.
        assert_eq!(app.callout_style.shape, CalloutShape::Underline);
        assert_eq!(app.callout_style_shown().shape, CalloutShape::Underline);
    }

    #[test]
    fn odd_callout_data_is_left_alone() {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Group { children: vec![] })
            .expect("group");
        for v in [
            "garbage",
            r#"{"tip":{"x":1,"y":2}}"#,
            r#"{"tip":{"x":1e999,"y":2},"elbow":{"x":0,"y":0}}"#,
        ] {
            app.run(Command::SetObjectData {
                shape: id,
                data: vec![(DATA_KEY.into(), v.into())],
            });
            assert!(app.doc().find_shape(id).and_then(Callout::of).is_none());
            app.refresh_callouts();
        }
        // A callout group with no text, or no leader: nothing to fit.
        // Numbers out of range are brought in.
        app.run(Command::SetObjectData {
            shape: id,
            data: vec![(
                DATA_KEY.into(),
                r#"{"tip":{"x":0,"y":0},"elbow":{"x":1,"y":1},
                    "style":{"gap":-5,"width":-3,"dash":[0,0],"shape":"ellipse"}}"#
                    .into(),
            )],
        });
        let read = app
            .doc()
            .find_shape(id)
            .and_then(Callout::of)
            .expect("read");
        assert_eq!(read.style.gap, 0.0);
        assert_eq!(read.style.shape, CalloutShape::Ellipse);
        assert_eq!(read.style.width, Stroke::HAIRLINE);
        assert!(read.style.dash.is_empty());
        let before = app.engine.history_labels().0.len();
        app.refresh_callouts();
        assert_eq!(app.engine.history_labels().0.len(), before);
        // Odd clicks do not panic either.
        app.create_callout(Point::new(f64::NAN, 0.0), Point::ZERO, Point::ZERO);
        assert_eq!(app.engine.history_labels().0.len(), before);
    }
}
