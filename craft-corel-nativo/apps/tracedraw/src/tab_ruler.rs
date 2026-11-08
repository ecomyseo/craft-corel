//! The tab ruler: a strip along the top edge of the paragraph frame being
//! edited with the Text tool, showing the tab stops of the paragraphs at
//! the caret (#1852). A click on it adds a left stop, a drag moves a stop,
//! a drag away from the strip removes it, and a right click picks a
//! stop's kind. Each change is one undo step. While a stop is dragged
//! the text flows to it as it moves (at most [`LIVE_EVERY`] apart, so a
//! long text keeps the drag smooth), and the whole drag is one undo step.
//! See docs/behavior/text.md, Tab ruler.

use crate::app::App;
use crate::theme::Tokens;
use crate::tools::Tool;
use egui::{Color32, Painter, Pos2, Stroke, Vec2};
use tracedraw_core::{
    default_tab_step, document::ShapeKind, normalized_tabs, ShapeId, TabKind, TabStop,
};

/// The gap between the frame's top edge and the strip, beyond the
/// selection handles, interface points.
const GAP: f32 = 4.0;
/// The strip's height, interface points.
pub const HEIGHT: f32 = 14.0;
/// How near a marker a press must be to take it, interface points.
const GRAB: f32 = 6.0;
/// How far from the strip a stop must be dragged to be removed.
const OFF: f32 = 18.0;
/// The shortest time between two reflows of the text while a stop is
/// dragged, seconds (#1852, #1912).
pub const LIVE_EVERY: f64 = 1.0 / 30.0;

/// The ruler of the frame being edited, as it stands on screen.
#[derive(Debug, Clone, PartialEq)]
pub struct TabRuler {
    pub shape: ShapeId,
    /// The left and right ends of the frame's top edge on screen.
    pub a: Pos2,
    pub b: Pos2,
    /// The unit vector pointing out of the frame, across the edge.
    pub up: Vec2,
    /// The frame's width, mm (the length the stops are measured along).
    pub width_mm: f64,
    /// The stops shown: the paragraph's own, or the default grid.
    pub stops: Vec<TabStop>,
    /// The paragraph has no stops of its own: `stops` is the default grid
    /// (the setting "Default tab spacing", 12.7 mm unless set).
    pub defaults: bool,
    /// The selection handles' size, interface points.
    pub handle: f32,
    /// The frame's height on screen, interface points.
    pub depth: f32,
}

/// What a point of the strip is over: a position along it, mm, and the
/// stop there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RulerHit {
    pub shape: ShapeId,
    pub pos: f64,
    pub stop: Option<usize>,
}

/// A stop being dragged along the ruler: the stops as they will be, the
/// index of the one moving, and whether it is off the strip (to go).
#[derive(Debug, Clone, PartialEq)]
pub struct TabDrag {
    pub shape: ShapeId,
    pub stops: Vec<TabStop>,
    pub index: usize,
    pub off: bool,
    /// The undo step before the drag: the drag's live steps join into one
    /// after it.
    pub before: Option<u64>,
    /// The stops the text was last laid out with during the drag, and
    /// when (input time, seconds); `None` until the first reflow.
    pub shown: Option<(Vec<TabStop>, f64)>,
}

impl TabDrag {
    /// The stops the paragraph takes if the drag ends now: the dragged
    /// one left out when it is off the strip.
    pub fn result(&self) -> Vec<TabStop> {
        let mut stops = self.stops.clone();
        if self.off && self.index < stops.len() {
            stops.remove(self.index);
        }
        stops
    }
}

/// The default grid of a frame `width` mm wide: a left stop every
/// [`default_tab_step`] mm inside it (the setting "Default tab spacing").
pub fn default_stops(width: f64) -> Vec<TabStop> {
    let mut out = Vec::new();
    let step = default_tab_step();
    let mut x = step;
    while x < width - 1e-6 && out.len() < 2000 {
        out.push(TabStop::left(x));
        x += step;
    }
    out
}

impl TabRuler {
    fn len(&self) -> f32 {
        (self.b - self.a).length().max(1e-3)
    }

    fn dir(&self) -> Vec2 {
        (self.b - self.a) / self.len()
    }

    /// The near and far sides of the strip from the edge.
    fn band(&self) -> (f32, f32) {
        let near = self.handle + GAP;
        (near, near + HEIGHT)
    }

    /// The screen point `along` mm from the left end, `out` points out
    /// from the edge.
    pub fn at(&self, along: f64, out: f32) -> Pos2 {
        let t = (along / self.width_mm.max(1e-6)) as f32;
        self.a + (self.b - self.a) * t + self.up * out
    }

    /// The middle of the strip, `along` mm from its left end.
    #[cfg(test)]
    pub fn on_strip(&self, along: f64) -> Pos2 {
        let (near, far) = self.band();
        self.at(along, (near + far) / 2.0)
    }

    /// Where a stop's marker stands: on the strip's bottom line.
    pub fn marker(&self, along: f64) -> Pos2 {
        self.at(along, self.band().0 + 2.0)
    }

    /// The position along the edge (mm) and the distance out from it
    /// (points) of screen point `p`.
    pub fn locate(&self, p: Pos2) -> (f64, f32) {
        let r = p - self.a;
        let along = r.dot(self.dir()) / self.len();
        (along as f64 * self.width_mm, r.dot(self.up))
    }

    /// What screen point `p` is over, if it is on the strip.
    pub fn hit(&self, p: Pos2) -> Option<RulerHit> {
        let (pos, out) = self.locate(p);
        let (near, far) = self.band();
        let px = self.len() as f64 / self.width_mm.max(1e-6);
        let slack = 3.0 / px;
        if out < near - 2.0 || out > far + 2.0 || pos < -slack || pos > self.width_mm + slack {
            return None;
        }
        let stop = self
            .stops
            .iter()
            .enumerate()
            .map(|(i, s)| (i, ((s.pos - pos) * px).abs()))
            .filter(|(_, d)| *d <= GRAB as f64)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i);
        Some(RulerHit {
            shape: self.shape,
            pos: pos.clamp(0.0, self.width_mm),
            stop,
        })
    }

    /// Whether screen point `p` is far enough from the strip for a
    /// dragged stop to go.
    pub fn is_off(&self, p: Pos2) -> bool {
        let (_, out) = self.locate(p);
        let (near, far) = self.band();
        out < near - OFF || out > far + OFF
    }
}

impl App {
    /// The tab ruler of the paragraph frame being edited with the Text
    /// tool; `None` for artistic text, text on a path, vertical text and
    /// other tools.
    pub fn tab_ruler(&self) -> Option<TabRuler> {
        if self.tool != Tool::Text {
            return None;
        }
        let id = self.text_edit.as_ref()?.shape;
        let s = self.doc().find_shape(id)?;
        let ShapeKind::Text {
            frame: Some(frame),
            on_path: None,
            ..
        } = &s.kind
        else {
            return None;
        };
        let (_, para) = self.paragraph_style_of(s)?;
        if para.vertical || frame.width <= 0.0 {
            return None;
        }
        let c = s.text_frame_corners()?;
        let pts = c.map(|p| self.view.to_screen(p));
        let (a, b) = (pts[3], pts[2]);
        let edge = b - a;
        if edge.length() < 8.0 {
            return None;
        }
        let mut up = Vec2::new(edge.y, -edge.x).normalized();
        if (a - pts[0]).dot(up) < 0.0 {
            up = -up;
        }
        let (stops, defaults) = match &self.tab_drag {
            Some(d) if d.shape == id => (d.stops.clone(), false),
            _ if para.tabs.is_empty() => (default_stops(frame.width), true),
            _ => (para.tabs.clone(), false),
        };
        Some(TabRuler {
            shape: id,
            a,
            b,
            up,
            depth: (pts[3] - pts[0]).length(),
            width_mm: frame.width,
            stops,
            defaults,
            handle: self.handle_size(),
        })
    }

    /// Give the paragraphs at the caret of text `id` the stops `tabs`
    /// (sorted, doubles dropped): one undo step.
    pub fn set_tab_stops(&mut self, id: ShapeId, tabs: Vec<TabStop>) {
        let tabs = normalized_tabs(&tabs);
        self.edit_paragraph_style(id, |p| p.tabs = tabs);
    }

    /// The canvas input for the tab ruler (Text tool): true when the
    /// ruler took it.
    pub fn tab_ruler_input(&mut self, response: &egui::Response, screen: Pos2) -> bool {
        use egui::PointerButton;
        // A stop being dragged follows the pointer until the release.
        if let Some(mut d) = self.tab_drag.take() {
            let Some(r) = self.tab_ruler().filter(|r| r.shape == d.shape) else {
                return true;
            };
            let now = response
                .ctx
                .pointer_latest_pos()
                .or(response.hover_pos())
                .unwrap_or(screen);
            let held = response.ctx.input(|i| i.pointer.primary_down());
            let (pos, _) = r.locate(now);
            if let Some(s) = d.stops.get_mut(d.index) {
                s.pos = ((pos.clamp(0.0, r.width_mm)) * 10.0).round() / 10.0;
            }
            d.off = r.is_off(now);
            let time = response.ctx.input(|i| i.time);
            let ended = !(held && !response.drag_stopped());
            let stops = d.result();
            let changed = d.shown.as_ref().is_none_or(|(s, _)| *s != stops);
            let due = d
                .shown
                .as_ref()
                .is_none_or(|(_, at)| time - at >= LIVE_EVERY);
            if changed && (due || ended) {
                // The text flows to the stops as they are now; the drag's
                // steps join into one undo step.
                self.set_tab_stops(d.shape, stops.clone());
                let label = self.engine.undo_label().unwrap_or("Tab Stops");
                self.engine.join_group(d.before, label);
                d.shown = Some((stops, time));
            } else if changed {
                // Laid out again once the pause is over, even if the
                // pointer stops here.
                let wait = d
                    .shown
                    .as_ref()
                    .map_or(0.0, |(_, at)| LIVE_EVERY - (time - at));
                response
                    .ctx
                    .request_repaint_after(std::time::Duration::from_secs_f64(wait.max(0.0)));
            }
            if !ended {
                self.tab_drag = Some(d);
            }
            response.ctx.request_repaint();
            return true;
        }
        let Some(r) = self.tab_ruler() else {
            return false;
        };
        if response.secondary_clicked() {
            let Some(hit) = r.hit(screen) else {
                return false;
            };
            self.tab_ruler_menu = Some(hit);
            self.context_menu = Some((screen, self.view.to_page(screen)));
            return true;
        }
        let press = response.ctx.input(|i| i.pointer.press_origin());
        if response.drag_started_by(PointerButton::Primary) {
            let Some(hit) = r.hit(press.unwrap_or(screen)) else {
                return false;
            };
            let mut stops = r.stops.clone();
            let index = match hit.stop {
                Some(i) => i,
                None => {
                    stops.push(TabStop::left(hit.pos));
                    stops.len() - 1
                }
            };
            self.tab_drag = Some(TabDrag {
                shape: r.shape,
                stops,
                index,
                off: false,
                before: self.engine.last_step_id(),
                shown: None,
            });
            return true;
        }
        if response.clicked_by(PointerButton::Primary) {
            let Some(hit) = r.hit(screen) else {
                return false;
            };
            if hit.stop.is_none() {
                let mut stops = r.stops.clone();
                stops.push(TabStop::left((hit.pos * 10.0).round() / 10.0));
                self.set_tab_stops(r.shape, stops);
            }
            return true;
        }
        // A press on the strip waits for its click or drag; the text
        // under it does not take it.
        response.is_pointer_button_down_on() && press.is_some_and(|p| r.hit(p).is_some())
    }

    /// The ruler's right-click menu picked `kind` (or `None`: remove) for
    /// what `hit` was over: the stop there, or a new stop at that place.
    pub fn tab_ruler_pick(&mut self, hit: RulerHit, kind: Option<TabKind>) {
        let Some(r) = self.tab_ruler().filter(|r| r.shape == hit.shape) else {
            return;
        };
        let mut stops = r.stops.clone();
        match (hit.stop, kind) {
            (Some(i), Some(k)) => {
                if let Some(s) = stops.get_mut(i) {
                    s.kind = k;
                }
            }
            (Some(i), None) => {
                if i < stops.len() {
                    stops.remove(i);
                }
            }
            (None, Some(k)) => stops.push(TabStop::new((hit.pos * 10.0).round() / 10.0, k)),
            (None, None) => return,
        }
        self.set_tab_stops(hit.shape, stops);
    }
}

/// Draw the tab ruler of the frame being edited: the strip with a tick
/// every millimetre (every 5 when they would crowd), a longer one every
/// 10, the stops as markers of their kind (the default grid lighter) and,
/// while a stop is dragged, a line down the frame where it would go.
pub fn draw(app: &App, painter: &Painter) {
    let Some(r) = app.tab_ruler() else {
        return;
    };
    let (near, far) = (r.handle + GAP, r.handle + GAP + HEIGHT);
    let fill = Tokens::ruler_bg();
    let ink = Tokens::text();
    let dim = Tokens::ruler_tick();
    painter.add(egui::Shape::convex_polygon(
        vec![
            r.at(0.0, near),
            r.at(r.width_mm, near),
            r.at(r.width_mm, far),
            r.at(0.0, far),
        ],
        fill,
        Stroke::new(1.0, dim),
    ));
    let px = r.len() as f64 / r.width_mm.max(1e-6);
    let step = if px >= 3.0 { 1.0 } else { 5.0 };
    let mut x = 0.0;
    let mut n = 0usize;
    while x <= r.width_mm + 1e-6 && n < 5000 {
        let major = (x / 10.0 - (x / 10.0).round()).abs() < 1e-6;
        let mid = (x / 5.0 - (x / 5.0).round()).abs() < 1e-6;
        let h = if major {
            HEIGHT * 0.6
        } else if mid {
            HEIGHT * 0.4
        } else {
            HEIGHT * 0.2
        };
        painter.line_segment([r.at(x, far), r.at(x, far - h)], Stroke::new(1.0, dim));
        x += step;
        n += 1;
    }
    let dragging = app.tab_drag.as_ref().filter(|d| d.shape == r.shape);
    for (i, s) in r.stops.iter().enumerate() {
        let moving = dragging.is_some_and(|d| d.index == i);
        let color = if moving && dragging.is_some_and(|d| d.off) {
            dim.gamma_multiply(0.4)
        } else if moving {
            Tokens::SELECTION
        } else if r.defaults {
            dim
        } else {
            ink
        };
        marker(painter, &r, s, color);
    }
    // Where the dragged stop would go: a line down the frame.
    if let Some(s) = dragging.and_then(|d| d.stops.get(d.index).filter(|_| !d.off)) {
        let top = r.at(s.pos, near);
        painter.line_segment(
            [top, top - r.up * (near + r.depth)],
            Stroke::new(1.0, Tokens::SELECTION.gamma_multiply(0.6)),
        );
    }
}

/// One stop's marker: a corner opening to the right (left stop), to the
/// left (right stop), an upturned T (centre stop), and the T with a dot
/// (decimal stop).
fn marker(painter: &Painter, r: &TabRuler, s: &TabStop, color: Color32) {
    let base = r.marker(s.pos);
    let dir = r.dir();
    let up = r.up;
    let p = |x: f32, y: f32| base + dir * x + up * y;
    let st = Stroke::new(2.0, color);
    let stem = [p(0.0, 0.0), p(0.0, 7.0)];
    painter.line_segment(stem, st);
    match s.kind {
        TabKind::Left => {
            painter.line_segment([p(0.0, 0.0), p(5.0, 0.0)], st);
        }
        TabKind::Right => {
            painter.line_segment([p(-5.0, 0.0), p(0.0, 0.0)], st);
        }
        TabKind::Center => {
            painter.line_segment([p(-4.0, 0.0), p(4.0, 0.0)], st);
        }
        TabKind::Decimal => {
            painter.line_segment([p(-4.0, 0.0), p(4.0, 0.0)], st);
            painter.circle_filled(p(3.0, 4.0), 1.3, color);
        }
    }
}
