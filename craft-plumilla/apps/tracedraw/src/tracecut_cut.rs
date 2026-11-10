//! TraceCut's Cut tools: plotter output (with the test cut), weeding,
//! bridges, cut order, CutContour and registration marks. The geometry
//! is `tracedraw_tracecut::cut`; this module keeps the tools' settings,
//! works their results out for the preview and turns them into commands.
//! See docs/behavior/tracecut.md.
//!
//! Adapted from the Sign & Cut cutting tools of #170 by LaMka21.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::tracecut_send::Destination;
use serde::{Deserialize, Serialize};
use tracedraw_core::geometry::{Affine, BezPath, PathEl, Point, Rect, Shape as _};
use tracedraw_core::{Color, Command, Fill, LayerId, Shape, ShapeId, ShapeKind, Stroke};
use tracedraw_tracecut::cut::bridges::{self, BridgeCount, BridgeSpec};
use tracedraw_tracecut::cut::hpgl::{self, Plot, Profile};
use tracedraw_tracecut::cut::marks::{self, RegSpec};
use tracedraw_tracecut::cut::order::{self, Direction, RouteItem, RouteSpec, StartAt};
use tracedraw_tracecut::cut::weed::{self, WeedSpec};

/// The name of the spot ink that marks cut lines.
pub const CUT_CONTOUR: &str = "CutContour";

/// The process stand-in of the CutContour ink: 100 % magenta, so the
/// lines show on screen and stand out in a proof.
pub const CUT_CONTOUR_CMYK: [f32; 4] = [0.0, 1.0, 0.0, 0.0];

/// Plotter Output: the cutter's profile and where the job goes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlotterSettings {
    pub profile: Profile,
    /// Cut in the planned order (nearest next, inner outlines first)
    /// instead of the stacking order.
    pub optimize: bool,
    pub inner_first: bool,
    pub destination: Destination,
    /// The port path: `/dev/ttyUSB0`, `COM3`.
    pub port: String,
    /// The cutter on the network: `host` or `host:port`.
    pub host: String,
    /// The side of the test cut's square (mm).
    pub test_size: f64,
}

impl Default for PlotterSettings {
    fn default() -> Self {
        PlotterSettings {
            profile: Profile::default(),
            optimize: true,
            inner_first: true,
            destination: Destination::File,
            port: crate::tracecut_send::default_port(),
            host: String::new(),
            test_size: 20.0,
        }
    }
}

impl PlotterSettings {
    /// The port path or the host the job goes to.
    pub fn target(&self) -> String {
        match self.destination.usable() {
            Destination::Port => self.port.trim().to_string(),
            Destination::Network => self.host.trim().to_string(),
            Destination::File => String::new(),
        }
    }
}

/// Bridges, as the window shows them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BridgeSettings {
    /// Length of each bridge (mm).
    pub length: f64,
    /// A count per outline (else one every `every` mm).
    pub per_outline: bool,
    pub count: u32,
    pub every: f64,
    /// Only outlines longer than this get bridges (mm).
    pub min_outline: f64,
}

impl Default for BridgeSettings {
    fn default() -> Self {
        BridgeSettings {
            length: 2.0,
            per_outline: true,
            count: 2,
            every: 100.0,
            min_outline: 0.0,
        }
    }
}

impl BridgeSettings {
    pub fn spec(&self) -> BridgeSpec {
        BridgeSpec {
            length: self.length,
            count: if self.per_outline {
                BridgeCount::PerOutline(self.count)
            } else {
                BridgeCount::Every(self.every)
            },
            min_outline: self.min_outline,
        }
    }
}

/// Cut Order: start points, directions and inner outlines first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderSettings {
    pub direction: Direction,
    pub start: StartAt,
    pub inner_first: bool,
}

impl Default for OrderSettings {
    fn default() -> Self {
        OrderSettings {
            direction: Direction::Keep,
            start: StartAt::Nearest,
            inner_first: true,
        }
    }
}

/// CutContour: the outline width (0: a hairline) and whether the fill
/// goes.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CutContourSettings {
    pub width: f64,
    pub remove_fill: bool,
}

/// Registration marks: their look, and whether they go inside the page
/// (else around the selection).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MarksSettings {
    pub spec: RegSpec,
    pub page: bool,
}

/// A change of the drawing worked out by a Cut tool.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// A new group of `paths` named by the i18n key `name`, drawn as
    /// `style` says, on the active layer.
    AddGroup {
        name: &'static str,
        paths: Vec<BezPath>,
        style: AddStyle,
    },
    /// A command as it is.
    Cmd(Box<Command>),
}

/// How new objects look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddStyle {
    /// A hairline, no fill (weeding lines).
    Hairline,
    /// Filled black, no outline (registration marks).
    Filled,
}

/// What a Cut tool worked out: the changes for OK in one undo step
/// named `label`, what the preview adds, and the status line.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Changes {
    pub label: &'static str,
    pub changes: Vec<Change>,
    /// Outlines to draw over the objects in the preview.
    pub preview: Vec<BezPath>,
    /// Places to ring in the preview.
    pub points: Vec<Point>,
    /// Moves with the knife up, for the preview.
    pub moves: Vec<(Point, Point)>,
    pub summary: String,
    /// The preview shows the result in place of the objects (nesting).
    pub hide_objects: bool,
}

/// A plotter job worked out for the preview and for OK.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub plot: Plot,
    /// The outlines in cut order (page space).
    pub cuts: Vec<BezPath>,
    pub moves: Vec<(Point, Point)>,
}

/// The inverse of `t`, when it has one.
fn invert(t: Affine) -> Option<Affine> {
    let det = t.determinant();
    (det.is_finite() && det.abs() > 1e-12).then(|| t.inverse())
}

/// True when every subpath of `path` is closed.
fn all_closed(path: &BezPath) -> bool {
    let els = path.elements();
    let starts = els
        .iter()
        .filter(|e| matches!(e, PathEl::MoveTo(_)))
        .count();
    let closes = els
        .iter()
        .filter(|e| matches!(e, PathEl::ClosePath))
        .count();
    starts > 0 && starts == closes
}

/// Each subpath of `path` as a path of its own.
fn split_subpaths(path: &BezPath) -> Vec<BezPath> {
    let mut out: Vec<BezPath> = Vec::new();
    for el in path.elements() {
        if matches!(el, PathEl::MoveTo(_)) || out.is_empty() {
            out.push(BezPath::new());
        }
        if let Some(last) = out.last_mut() {
            last.push(*el);
        }
    }
    out.retain(|p| p.elements().len() > 1);
    out
}

/// The moves with the knife up between outlines cut in this order, from
/// `from`.
fn moves_between(cuts: &[BezPath], from: Point) -> Vec<(Point, Point)> {
    let mut at = from;
    let mut out = Vec::new();
    for c in cuts {
        let els = c.elements();
        let (Some(start), Some(end)) = (
            els.first().and_then(|e| e.end_point()),
            els.iter().rev().find_map(|e| e.end_point()),
        ) else {
            continue;
        };
        let closed = matches!(els.last(), Some(PathEl::ClosePath));
        if (start - at).hypot() > 1e-9 {
            out.push((at, start));
        }
        at = if closed { start } else { end };
    }
    out
}

/// The box of the finite, non-empty paths.
fn bounds_of(paths: &[BezPath]) -> Option<Rect> {
    paths
        .iter()
        .filter(|p| !p.elements().is_empty())
        .map(|p| p.bounding_box())
        .filter(|b| b.x0.is_finite() && b.y0.is_finite() && b.x1.is_finite() && b.y1.is_finite())
        .reduce(|a, b| a.union(b))
}

/// A length for the status line, in the document's units.
pub fn fmt_len(u: crate::app::Units, mm: f64) -> String {
    format!("{:.1} {}", u.world_from_mm(mm), u.short())
}

impl App {
    /// The objects among `ids` whose outline a Cut tool may rewrite:
    /// shapes and curves, groups opened, unlocked, without live effects.
    pub fn tracecut_leaves(&self, ids: &[ShapeId]) -> Vec<ShapeId> {
        fn walk(s: &Shape, out: &mut Vec<ShapeId>) {
            if s.locked {
                return;
            }
            match &s.kind {
                ShapeKind::Group { children } => {
                    for c in children {
                        walk(c, out);
                    }
                }
                ShapeKind::Rect { .. }
                | ShapeKind::Ellipse { .. }
                | ShapeKind::Polygon { .. }
                | ShapeKind::Path { .. }
                    if s.effects.is_empty() =>
                {
                    out.push(s.id);
                }
                _ => {}
            }
        }
        let doc = self.doc();
        let mut out = Vec::new();
        for id in ids {
            if let Some(s) = doc.find_shape(*id) {
                walk(s, &mut out);
            }
        }
        out
    }

    /// The plotter job of the selection: its outlines in stacking order,
    /// or planned (nearest next, inner outlines first) from the job's
    /// bottom-left corner.
    pub fn tracecut_job(&self, s: &PlotterSettings) -> Job {
        let ids = self.tracecut_targets();
        let outlines = self.tracecut_outlines(&ids);
        let all: Vec<BezPath> = outlines.iter().flatten().cloned().collect();
        let corner = bounds_of(&all)
            .map(|b| Point::new(b.x0, b.y0))
            .unwrap_or(Point::ZERO);
        let cuts: Vec<BezPath> = if s.optimize {
            let items: Vec<RouteItem> = outlines
                .iter()
                .map(|paths| RouteItem {
                    paths: paths.clone(),
                    editable: true,
                })
                .collect();
            let spec = RouteSpec {
                direction: Direction::Keep,
                start: StartAt::Nearest,
                inner_first: s.inner_first,
                from: corner,
            };
            let route = order::plan(&items, &spec);
            route
                .order
                .iter()
                .filter_map(|i| route.cuts.get(*i))
                .flatten()
                .cloned()
                .collect()
        } else {
            all.iter().flat_map(split_subpaths).collect()
        };
        let plot = hpgl::plot(&cuts, &s.profile);
        let moves = moves_between(&cuts, corner);
        Job { plot, cuts, moves }
    }

    /// The test cut as a job.
    pub fn tracecut_test_job(&self, s: &PlotterSettings) -> Plot {
        hpgl::plot(&hpgl::test_cut(s.test_size), &s.profile)
    }

    /// Send or save `plot`: a `.plt` file where the user says, or the
    /// port or the network on a thread of its own.
    pub fn tracecut_output(
        &mut self,
        s: &PlotterSettings,
        plot: &Plot,
        ctx: Option<egui::Context>,
    ) {
        let bytes = plot.text.clone().into_bytes();
        match s.destination.usable() {
            Destination::File => {
                crate::files::Dialog::new()
                    .add_filter("PLT (HPGL)", &["plt"])
                    .set_file_name("cut.plt")
                    .save_file(self, move |app, path| {
                        app.status = match crate::files::write(&path, bytes) {
                            Ok(()) => {
                                trf("tracecut.saved", &[("path", &path.display().to_string())])
                            }
                            Err(e) => e.to_string(),
                        };
                    });
            }
            dest => {
                let target = s.target();
                if target.is_empty() {
                    self.status = tr("tracecut.no_target");
                    return;
                }
                self.status = trf("tracecut.sending", &[("to", &target)]);
                crate::tracecut_send::start(dest, target, bytes, ctx);
            }
        }
    }

    /// The job's line for the status strip.
    pub fn tracecut_job_summary(&self, job: &Job) -> String {
        let u = self.units;
        if job.plot.outlines == 0 {
            return tr("tracecut.nothing_to_cut");
        }
        trf(
            "tracecut.job_summary",
            &[
                ("n", &job.plot.outlines.to_string()),
                ("cut", &fmt_len(u, job.plot.cut_length)),
                ("travel", &fmt_len(u, job.plot.travel)),
                ("w", &fmt_len(u, job.plot.width)),
                ("h", &fmt_len(u, job.plot.height)),
            ],
        )
    }

    /// Weeding lines, boxes and frame around the selection.
    pub fn tracecut_weeding(&self, spec: &WeedSpec) -> Changes {
        let outlines = self.tracecut_outlines(&self.tracecut_targets());
        let paths = weed::weeding(&outlines, spec);
        let summary = if paths.is_empty() {
            tr("tracecut.nothing_to_cut")
        } else {
            trf("tracecut.weeding_found", &[("n", &paths.len().to_string())])
        };
        Changes {
            label: "Weeding",
            preview: paths.clone(),
            changes: if paths.is_empty() {
                Vec::new()
            } else {
                vec![Change::AddGroup {
                    name: "tracecut.weeding_group",
                    paths,
                    style: AddStyle::Hairline,
                }]
            },
            summary,
            ..Default::default()
        }
    }

    /// Bridges in the closed outlines of the selected shapes and curves.
    pub fn tracecut_bridges(&self, s: &BridgeSettings) -> Changes {
        let spec = s.spec();
        let doc = self.doc();
        let mut out = Changes {
            label: "Bridges",
            ..Default::default()
        };
        let mut n = 0;
        for id in self.tracecut_leaves(&self.tracecut_targets()) {
            let Some(shape) = doc.shape_on_page(id) else {
                continue;
            };
            let page = shape.page_path();
            let Some(b) = bridges::bridge_path(&page, &spec) else {
                continue;
            };
            let Some(inv) = invert(shape.transform) else {
                continue;
            };
            n += b.bridges;
            out.points.extend(b.centres.iter().copied());
            out.preview.push(b.path.clone());
            out.changes
                .push(Change::Cmd(Box::new(Command::SetShapeKind {
                    shape: id,
                    kind: ShapeKind::Path {
                        path: inv * b.path,
                        closed: false,
                    },
                })));
        }
        out.summary = if n == 0 {
            tr("tracecut.no_bridges")
        } else {
            trf("tracecut.bridges_found", &[("n", &n.to_string())])
        };
        out
    }

    /// The cut order of the selection, layer by layer: the objects
    /// reordered in their layers, the subpaths of shapes and curves given
    /// their new order, start and direction.
    pub fn tracecut_cut_order(&self, s: &OrderSettings) -> Changes {
        let targets = self.tracecut_targets();
        let doc = self.doc();
        let mut out = Changes {
            label: "Cut Order",
            ..Default::default()
        };
        let mut from = Point::ZERO;
        let (mut before, mut after) = (0.0, 0.0);
        for layer in self.page_layers() {
            if layer.locked {
                continue;
            }
            let ids: Vec<ShapeId> = layer
                .shapes
                .iter()
                .filter(|sh| !sh.locked && targets.contains(&sh.id))
                .map(|sh| sh.id)
                .collect();
            if ids.is_empty() {
                continue;
            }
            let items: Vec<RouteItem> = ids
                .iter()
                .map(|id| {
                    let sh = doc.shape_on_page(*id);
                    let editable = sh.as_ref().is_some_and(|sh| {
                        sh.effects.is_empty()
                            && matches!(
                                sh.kind,
                                ShapeKind::Rect { .. }
                                    | ShapeKind::Ellipse { .. }
                                    | ShapeKind::Polygon { .. }
                                    | ShapeKind::Path { .. }
                            )
                    });
                    RouteItem {
                        paths: sh
                            .map(|sh| tracedraw_tracecut::outlines(&sh))
                            .unwrap_or_default(),
                        editable,
                    }
                })
                .collect();
            let spec = RouteSpec {
                direction: s.direction,
                start: s.start,
                inner_first: s.inner_first,
                from,
            };
            let route = order::plan(&items, &spec);
            from = route.end;
            before += route.travel_before;
            after += route.travel_after;
            out.moves.extend(route.moves.iter().copied());
            // The layer's list with the targets' places taken in route
            // order, then one move per object out of place.
            let wanted: Vec<ShapeId> = route
                .order
                .iter()
                .filter_map(|i| ids.get(*i))
                .copied()
                .collect();
            let mut next = wanted.iter();
            let final_order: Vec<ShapeId> = layer
                .shapes
                .iter()
                .map(|sh| {
                    if ids.contains(&sh.id) {
                        next.next().copied().unwrap_or(sh.id)
                    } else {
                        sh.id
                    }
                })
                .collect();
            let mut now: Vec<ShapeId> = layer.shapes.iter().map(|sh| sh.id).collect();
            for (i, id) in final_order.iter().enumerate() {
                if now.get(i) == Some(id) {
                    continue;
                }
                if let Some(from_i) = now.iter().position(|x| x == id) {
                    let moved = now.remove(from_i);
                    now.insert(i, moved);
                    out.changes.push(Change::Cmd(Box::new(Command::Reorder {
                        shape: *id,
                        layer: layer.id,
                        index: i,
                    })));
                }
            }
            for (i, changed) in route.changed.iter().enumerate() {
                let (Some(id), Some(cuts)) = (ids.get(i), route.cuts.get(i)) else {
                    continue;
                };
                out.preview.extend(cuts.iter().cloned());
                if !changed {
                    continue;
                }
                let Some(inv) = doc.shape_on_page(*id).and_then(|sh| invert(sh.transform)) else {
                    continue;
                };
                let mut path = BezPath::new();
                for c in cuts {
                    path.extend(c.elements().iter().copied());
                }
                let closed = all_closed(&path);
                out.changes
                    .push(Change::Cmd(Box::new(Command::SetShapeKind {
                        shape: *id,
                        kind: ShapeKind::Path {
                            path: inv * path,
                            closed,
                        },
                    })));
            }
        }
        let u = self.units;
        out.summary = if out.preview.is_empty() {
            tr("tracecut.nothing_to_cut")
        } else {
            trf(
                "tracecut.order_found",
                &[
                    ("before", &fmt_len(u, before)),
                    ("after", &fmt_len(u, after)),
                ],
            )
        };
        out
    }

    /// The CutContour ink of the drawing, and the command that adds it
    /// when the drawing has none.
    pub fn tracecut_contour_ink(&self) -> (Color, Option<Command>) {
        let inks = &self.doc().inks;
        if let Some(i) = inks.iter().find(|i| i.name == CUT_CONTOUR) {
            return (i.color(1.0), None);
        }
        let id = inks.iter().map(|i| i.id).max().unwrap_or(0) + 1;
        let ink = tracedraw_core::document::Ink {
            id,
            name: CUT_CONTOUR.into(),
            cmyk: CUT_CONTOUR_CMYK,
        };
        let color = ink.color(1.0);
        let mut all = inks.clone();
        all.push(ink);
        (color, Some(Command::SetInks { inks: all }))
    }

    /// Mark the selection's outlines as cut lines: a CutContour outline
    /// (the ink added to the drawing when it has none), the fill removed
    /// when asked.
    pub fn tracecut_cut_contour(&self, s: &CutContourSettings) -> Changes {
        let ids: Vec<ShapeId> = self
            .tracecut_targets()
            .into_iter()
            .filter(|id| self.doc().find_shape(*id).is_some_and(|sh| !sh.locked))
            .collect();
        let mut out = Changes {
            label: "CutContour",
            ..Default::default()
        };
        if ids.is_empty() {
            out.summary = tr("tracecut.select_first");
            return out;
        }
        let (color, add) = self.tracecut_contour_ink();
        if let Some(c) = add {
            out.changes.push(Change::Cmd(Box::new(c)));
        }
        // Its swatch joins the document palette in the same step, as a
        // colour applied from the Properties docker does (#1927).
        if self.settings.palette.auto_update_document && !self.doc().palette.contains(&color) {
            let mut colors = self.doc().palette.clone();
            colors.push(color);
            out.changes
                .push(Change::Cmd(Box::new(Command::SetDocumentPalette {
                    colors,
                })));
        }
        let width = if s.width.is_finite() && s.width > Stroke::HAIRLINE {
            s.width.min(10.0)
        } else {
            Stroke::HAIRLINE
        };
        let mut stroke = Stroke::hairline(color);
        stroke.width = width;
        out.changes.push(Change::Cmd(Box::new(Command::SetStroke {
            shapes: ids.clone(),
            stroke: Some(stroke),
        })));
        if s.remove_fill {
            out.changes.push(Change::Cmd(Box::new(Command::SetFill {
                shapes: ids.clone(),
                fill: Fill::None,
            })));
        }
        let outlines = self.tracecut_outlines(&ids);
        out.preview = outlines.into_iter().flatten().collect();
        out.summary = trf("tracecut.contour_found", &[("n", &ids.len().to_string())]);
        out
    }

    /// Registration marks around the selection, or inside the page.
    pub fn tracecut_marks(&self, s: &MarksSettings) -> Changes {
        let area = if s.page {
            Some(self.page_rect())
        } else {
            bounds_of(
                &self
                    .tracecut_outlines(&self.tracecut_targets())
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>(),
            )
        };
        let spec = RegSpec {
            inside: s.page,
            ..s.spec
        };
        let paths = area
            .map(|a| marks::registration_marks(a, &spec))
            .unwrap_or_default();
        Changes {
            label: "Registration Marks",
            preview: paths.clone(),
            summary: if paths.is_empty() {
                tr("tracecut.select_first")
            } else {
                trf("tracecut.marks_found", &[("n", &paths.len().to_string())])
            },
            changes: if paths.is_empty() {
                Vec::new()
            } else {
                vec![Change::AddGroup {
                    name: "tracecut.marks_group",
                    paths,
                    style: AddStyle::Filled,
                }]
            },
            ..Default::default()
        }
    }

    /// Apply `c` in one undo step. Returns whether anything changed; the
    /// status line says what.
    pub fn tracecut_apply_changes(&mut self, c: &Changes) -> bool {
        if c.changes.is_empty() {
            self.status = c.summary.clone();
            return false;
        }
        let mut cmds = Vec::new();
        let mut added = Vec::new();
        for ch in &c.changes {
            match ch {
                Change::Cmd(cmd) => cmds.push((**cmd).clone()),
                Change::AddGroup { name, paths, style } => {
                    let Some(layer) = self.active_layer() else {
                        continue;
                    };
                    let g = self.tracecut_group(layer, name, paths, *style);
                    added.push(g.id);
                    cmds.push(Command::AddShape { layer, shape: g });
                }
            }
        }
        if cmds.is_empty() {
            self.status = tr("tracecut.no_layer");
            return false;
        }
        if let Err(e) = self.engine.run_batch(c.label, &cmds) {
            self.status = e.to_string();
            return false;
        }
        if !added.is_empty() {
            self.select(added);
        }
        self.status = c.summary.clone();
        true
    }

    /// A group of new objects drawn from `paths`.
    fn tracecut_group(
        &mut self,
        _layer: LayerId,
        name: &str,
        paths: &[BezPath],
        style: AddStyle,
    ) -> Shape {
        let children: Vec<Shape> = paths
            .iter()
            .map(|p| {
                let id = self.engine.new_shape_id();
                let closed = all_closed(p);
                let mut s = Shape::new(
                    id,
                    ShapeKind::Path {
                        path: p.clone(),
                        closed,
                    },
                );
                match style {
                    AddStyle::Hairline => {
                        s.fill = Fill::None;
                        s.stroke = Some(Stroke::hairline(Color::BLACK));
                    }
                    AddStyle::Filled => {
                        s.fill = Fill::Solid(Color::BLACK);
                        s.stroke = None;
                    }
                }
                s
            })
            .collect();
        let id = self.engine.new_shape_id();
        let mut g = Shape::new(id, ShapeKind::Group { children });
        g.fill = Fill::None;
        g.stroke = None;
        g.name = Some(tr(name));
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(app: &mut App, x: f64, y: f64, w: f64, h: f64) -> ShapeId {
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(x, y, x + w, y + h),
            radius: 0.0,
            corners: None,
        })
        .expect("rect")
    }

    fn undo_len(app: &App) -> usize {
        app.engine.history_labels().0.len()
    }

    #[test]
    fn the_job_cuts_inner_outlines_first_and_goes_to_hpgl() {
        let mut app = App::headless();
        let outer = rect(&mut app, 0.0, 0.0, 100.0, 100.0);
        let inner = rect(&mut app, 40.0, 40.0, 20.0, 20.0);
        app.select(vec![outer, inner]);
        let s = PlotterSettings {
            profile: Profile {
                knife_offset: 0.0,
                overcut: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let job = app.tracecut_job(&s);
        assert_eq!(job.cuts.len(), 2);
        // The inner square first.
        assert!(job.cuts[0].bounding_box().width() < 30.0);
        assert!(job.plot.text.starts_with("IN;"));
        assert_eq!(job.plot.outlines, 2);
        assert!((job.plot.width - 100.0).abs() < 1e-6);
        // In stacking order the outer square comes first.
        let plain = app.tracecut_job(&PlotterSettings {
            optimize: false,
            ..s.clone()
        });
        assert!(plain.cuts[0].bounding_box().width() > 90.0);
        assert!(app.tracecut_job_summary(&job).contains('2'));
        // Nothing selected: nothing to cut.
        app.select(Vec::new());
        assert_eq!(app.tracecut_job(&s).plot.outlines, 0);
    }

    #[test]
    fn the_test_cut_is_a_small_job() {
        let app = App::headless();
        let p = app.tracecut_test_job(&PlotterSettings::default());
        assert_eq!(p.outlines, 2);
        assert!((p.width - 20.0).abs() < 1e-9);
    }

    #[test]
    fn weeding_adds_one_group_of_hairlines_in_one_step() {
        let mut app = App::headless();
        let a = rect(&mut app, 0.0, 0.0, 20.0, 20.0);
        let b = rect(&mut app, 40.0, 0.0, 20.0, 20.0);
        app.select(vec![a, b]);
        let c = app.tracecut_weeding(&WeedSpec::default());
        assert!(!c.preview.is_empty());
        let before = undo_len(&app);
        assert!(app.tracecut_apply_changes(&c));
        assert_eq!(undo_len(&app), before + 1);
        let g = app
            .doc()
            .find_shape(app.selection[0])
            .expect("group")
            .clone();
        let ShapeKind::Group { children } = &g.kind else {
            panic!("a group");
        };
        assert_eq!(children.len(), c.preview.len());
        assert!(children
            .iter()
            .all(|s| s.fill == Fill::None && s.stroke.is_some()));
        app.undo();
        assert!(app.doc().find_shape(g.id).is_none());
    }

    #[test]
    fn bridges_open_the_outlines_in_one_step() {
        let mut app = App::headless();
        let a = rect(&mut app, 0.0, 0.0, 50.0, 50.0);
        app.select(vec![a]);
        let c = app.tracecut_bridges(&BridgeSettings::default());
        assert_eq!(c.points.len(), 2);
        let before = undo_len(&app);
        assert!(app.tracecut_apply_changes(&c));
        assert_eq!(undo_len(&app), before + 1);
        let s = app.doc().find_shape(a).expect("shape");
        let ShapeKind::Path { path, closed } = &s.kind else {
            panic!("a curve");
        };
        assert!(!closed);
        assert_eq!(
            path.elements()
                .iter()
                .filter(|e| matches!(e, PathEl::MoveTo(_)))
                .count(),
            2
        );
        // The page stays where it was: the cut pieces lie on the square.
        let b = s.page_path().bounding_box();
        assert!((b.x0 - 0.0).abs() < 1e-6 && (b.x1 - 50.0).abs() < 1e-6);
    }

    #[test]
    fn cut_order_reorders_the_layer_in_one_step() {
        let mut app = App::headless();
        let far = rect(&mut app, 200.0, 0.0, 10.0, 10.0);
        let near = rect(&mut app, 0.0, 0.0, 10.0, 10.0);
        let mid = rect(&mut app, 100.0, 0.0, 10.0, 10.0);
        app.select(vec![far, near, mid]);
        let c = app.tracecut_cut_order(&OrderSettings::default());
        let before = undo_len(&app);
        assert!(app.tracecut_apply_changes(&c));
        assert_eq!(undo_len(&app), before + 1);
        let layer = app.active_layer().expect("layer");
        let order: Vec<ShapeId> = app
            .doc()
            .layer(layer)
            .expect("layer")
            .shapes
            .iter()
            .map(|s| s.id)
            .collect();
        assert_eq!(order, vec![near, mid, far]);
    }

    #[test]
    fn cut_contour_adds_the_ink_once_and_marks_the_outlines() {
        let mut app = App::headless();
        let a = rect(&mut app, 0.0, 0.0, 20.0, 20.0);
        app.select(vec![a]);
        let s = CutContourSettings {
            width: 0.0,
            remove_fill: true,
        };
        let c = app.tracecut_cut_contour(&s);
        let before = undo_len(&app);
        assert!(app.tracecut_apply_changes(&c));
        assert_eq!(undo_len(&app), before + 1);
        let inks = app.doc().inks.clone();
        assert_eq!(inks.iter().filter(|i| i.name == CUT_CONTOUR).count(), 1);
        let sh = app.doc().find_shape(a).expect("shape");
        assert_eq!(sh.fill, Fill::None);
        let stroke = sh.stroke.clone().expect("outline");
        assert!(matches!(stroke.color, Color::Spot { .. }));
        assert!((stroke.width - Stroke::HAIRLINE).abs() < 1e-12);
        // Its swatch is in the document palette, once, and the undo step
        // takes it out again with the ink (#1927).
        assert_eq!(app.doc().palette, vec![stroke.color]);
        // Again: the same ink, no second one, no second swatch.
        let (_, add) = app.tracecut_contour_ink();
        assert!(add.is_none());
        let again = app.tracecut_cut_contour(&s);
        assert!(app.tracecut_apply_changes(&again));
        assert_eq!(app.doc().palette, vec![stroke.color]);
        app.undo();
        app.undo();
        assert!(app.doc().palette.is_empty());
        assert!(app.doc().inks.iter().all(|i| i.name != CUT_CONTOUR));
        // With the document palette not following the colours used, it
        // is left as it is.
        let mut app = App::headless();
        app.settings.palette.auto_update_document = false;
        let a = rect(&mut app, 0.0, 0.0, 20.0, 20.0);
        app.select(vec![a]);
        let c = app.tracecut_cut_contour(&s);
        assert!(app.tracecut_apply_changes(&c));
        assert!(app.doc().palette.is_empty());
        // Kept by name in PDF, EPS and SVG.
        let pdf = tracedraw_io::pdf::document_to_pdf(app.doc());
        assert!(String::from_utf8_lossy(&pdf).contains("/Separation /CutContour"));
        let eps = tracedraw_io::eps::page_to_eps(app.doc(), 0);
        assert!(eps.contains("(CutContour) cvn"), "{eps}");
        assert!(tracedraw_io::svg::page_to_svg(app.doc(), 0)
            .contains("data-spot-stroke=\"CutContour\""));
    }

    #[test]
    fn registration_marks_go_around_the_selection_or_in_the_page() {
        let mut app = App::headless();
        let a = rect(&mut app, 50.0, 50.0, 40.0, 30.0);
        app.select(vec![a]);
        let c = app.tracecut_marks(&MarksSettings::default());
        assert_eq!(c.preview.len(), 4);
        let around = bounds_of(&c.preview).expect("marks");
        assert!(around.x0 < 50.0 && around.x1 > 90.0);
        let inside = app.tracecut_marks(&MarksSettings {
            page: true,
            ..Default::default()
        });
        let page = app.page_rect();
        let b = bounds_of(&inside.preview).expect("marks");
        assert!(b.x0 >= page.x0 && b.x1 <= page.x1 && b.y0 >= page.y0 && b.y1 <= page.y1);
        assert!(app.tracecut_apply_changes(&c));
        let g = app.doc().find_shape(app.selection[0]).expect("group");
        let ShapeKind::Group { children } = &g.kind else {
            panic!("a group");
        };
        assert!(children
            .iter()
            .all(|s| s.fill == Fill::Solid(Color::BLACK) && s.stroke.is_none()));
    }

    #[test]
    fn a_send_without_a_target_says_so() {
        let mut app = App::headless();
        let a = rect(&mut app, 0.0, 0.0, 10.0, 10.0);
        app.select(vec![a]);
        let s = PlotterSettings::default();
        let job = app.tracecut_job(&s);
        let net = PlotterSettings {
            destination: Destination::Network,
            host: String::new(),
            ..s
        };
        app.tracecut_output(&net, &job.plot, None);
        if crate::tracecut_send::CAN_SEND {
            assert_eq!(app.status, tr("tracecut.no_target"));
        }
    }
}
