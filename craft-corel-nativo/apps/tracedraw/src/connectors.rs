//! Connectors that stay attached. A connector is an open path between an
//! anchor of one object and an anchor of another; what it joins is in its
//! object data (`connector`): the two objects, each anchor as fractions of
//! its object's bounds (as custom anchors are), how it runs (straight, at
//! right angles, at right angles with rounded corners and their radius)
//! and where its ends were last put.
//!
//! When the drawing changes, each connector looks at its objects: moved,
//! scaled or reshaped, the connector runs again between their anchors, in
//! the same undo step. A connector moved on its own (or pasted elsewhere)
//! no longer meets the ends it was given and is let go: it stays where it
//! is as a plain line. Moved together with its objects it stays attached.

use crate::app::App;
use serde::{Deserialize, Serialize};
use tracedraw_core::{
    document::{Shape, ShapeKind},
    geometry::{BezPath, PathEl, Point, Rect},
    Command, Fill, ShapeId,
};

/// The object data key holding a connection.
pub const DATA_KEY: &str = "connector";

/// How a connector runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
    #[default]
    Straight,
    RightAngle,
    Rounded,
}

fn default_radius() -> f64 {
    3.0
}

/// What a connector joins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    pub from: ShapeId,
    /// Fractions of `from`'s bounds.
    pub from_anchor: (f64, f64),
    pub to: ShapeId,
    pub to_anchor: (f64, f64),
    #[serde(default)]
    pub kind: ConnectorKind,
    /// Corner radius of rounded connectors, mm.
    #[serde(default = "default_radius")]
    pub radius: f64,
    /// Where the ends were last put (page space).
    pub ends: (Point, Point),
}

impl Connection {
    /// The connection a shape holds.
    pub fn of(s: &Shape) -> Option<Connection> {
        if !matches!(s.kind, ShapeKind::Path { .. }) {
            return None;
        }
        let (_, v) = s.data.iter().find(|(k, _)| k == DATA_KEY)?;
        let c: Connection = serde_json::from_str(v).ok()?;
        let ok = [
            c.from_anchor.0,
            c.from_anchor.1,
            c.to_anchor.0,
            c.to_anchor.1,
            c.radius,
        ]
        .iter()
        .all(|v| v.is_finite())
            && [c.ends.0, c.ends.1]
                .iter()
                .all(|p| p.x.is_finite() && p.y.is_finite());
        ok.then_some(c)
    }
}

/// Object data with the connection `c` (none: without one).
pub fn with_connection(
    mut data: Vec<(String, String)>,
    c: Option<&Connection>,
) -> Vec<(String, String)> {
    data.retain(|(k, _)| k != DATA_KEY);
    if let Some(v) = c.and_then(|c| serde_json::to_string(c).ok()) {
        data.push((DATA_KEY.into(), v));
    }
    data
}

/// The point at fractions `f` of `b`.
fn at(b: Rect, f: (f64, f64)) -> Point {
    Point::new(b.x0 + f.0 * b.width(), b.y0 + f.1 * b.height())
}

/// The fractions of `b` at `p` (the middle across a side of no size).
fn fractions(b: Rect, p: Point) -> (f64, f64) {
    let f = |v: f64, lo: f64, size: f64| {
        if size.abs() < 1e-9 {
            0.5
        } else {
            ((v - lo) / size).clamp(0.0, 1.0)
        }
    };
    (f(p.x, b.x0, b.width()), f(p.y, b.y0, b.height()))
}

/// The connector's path from `pa` to `pb`: straight, or at right angles
/// with an elbow halfway along the longer span (rounded: corners of up to
/// `radius`, never more than a quarter of the distance).
pub fn route(kind: ConnectorKind, radius: f64, pa: Point, pb: Point) -> BezPath {
    let mut path = BezPath::new();
    path.move_to(pa);
    if kind == ConnectorKind::Straight {
        path.line_to(pb);
        return path;
    }
    let horizontal = (pb.x - pa.x).abs() > (pb.y - pa.y).abs();
    let c1 = if horizontal {
        Point::new((pa.x + pb.x) / 2.0, pa.y)
    } else {
        Point::new(pa.x, (pa.y + pb.y) / 2.0)
    };
    let c2 = if horizontal {
        Point::new(c1.x, pb.y)
    } else {
        Point::new(pb.x, c1.y)
    };
    let r = radius.max(0.0).min((pb - pa).hypot() / 4.0);
    let corner = |path: &mut BezPath, from: Point, c: Point, to: Point| {
        let (d1, d2) = (c - from, to - c);
        let (l1, l2) = (d1.hypot(), d2.hypot());
        if kind == ConnectorKind::Rounded && r > 1e-9 && l1 > 1e-9 && l2 > 1e-9 {
            let rr = r.min(l1 / 2.0).min(l2 / 2.0);
            path.line_to(c - d1 / l1 * rr);
            path.quad_to(c, c + d2 / l2 * rr);
        } else {
            path.line_to(c);
        }
    };
    corner(&mut path, pa, c1, c2);
    corner(&mut path, c1, c2, pb);
    path.line_to(pb);
    path
}

/// The first and last points of a path.
fn ends_of(path: &BezPath) -> Option<(Point, Point)> {
    let first = match path.elements().first()? {
        PathEl::MoveTo(p) => *p,
        _ => return None,
    };
    let last = path.elements().iter().rev().find_map(|e| match e {
        PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => Some(*p),
        _ => None,
    })?;
    Some((first, last))
}

fn near(a: Point, b: Point) -> bool {
    (a - b).hypot() < 1e-4
}

impl App {
    /// The connectors copies bring along (#476): each connector of the
    /// page that joins a copied object (`originals[i]`, copied as
    /// `copies[i]`) to one not copied, and is not copied itself, gives a
    /// connector of the same kind, outline and anchors from that object to
    /// the copy. Their ids come from the engine; add them with the copies.
    pub fn connector_copies(&mut self, originals: &[ShapeId], copies: &[Shape]) -> Vec<Shape> {
        let copy_of = |id: ShapeId| {
            originals
                .iter()
                .position(|o| *o == id)
                .and_then(|i| copies.get(i))
        };
        let connectors: Vec<(Shape, Connection)> = self
            .page_layers()
            .iter()
            .flat_map(|l| &l.shapes)
            .filter(|s| !originals.contains(&s.id))
            .filter_map(|s| Connection::of(s).map(|c| (s.clone(), c)))
            .collect();
        let mut out = Vec::new();
        for (s, c) in connectors {
            // Exactly one end copied: the other end stays where it is.
            let (from, to, fb, tb) = match (copy_of(c.from), copy_of(c.to)) {
                (Some(f), None) => match self.anchor_box(c.to) {
                    Some(tb) => (f.id, c.to, f.bounds(), tb),
                    None => continue,
                },
                (None, Some(t)) => match self.anchor_box(c.from) {
                    Some(fb) => (c.from, t.id, fb, t.bounds()),
                    None => continue,
                },
                _ => continue,
            };
            let (pa, pb) = (at(fb, c.from_anchor), at(tb, c.to_anchor));
            let joined = Connection {
                from,
                to,
                ends: (pa, pb),
                ..c.clone()
            };
            let id = self.engine.new_shape_id();
            let mut copy = crate::app::reid_pub(s.clone(), id, &mut self.engine);
            copy.transform = tracedraw_core::geometry::Affine::IDENTITY;
            copy.kind = ShapeKind::Path {
                path: route(c.kind, c.radius, pa, pb),
                closed: false,
            };
            copy.data = with_connection(s.data.clone(), Some(&joined));
            out.push(copy);
        }
        out
    }

    /// An object's bounds on the page, for its anchors.
    fn anchor_box(&self, id: ShapeId) -> Option<Rect> {
        self.doc().shape_on_page(id).map(|s| s.bounds())
    }

    /// A connector from `from` to `to`, from the drag that began at `start`
    /// and ended at `end` (the anchors are chosen as the Connector tools
    /// choose them); one undo step. The new connector is selected.
    pub fn connect(
        &mut self,
        from: ShapeId,
        start: Point,
        to: ShapeId,
        end: Point,
        kind: ConnectorKind,
    ) -> Option<ShapeId> {
        let (pa, pb) = self.connector_anchors(from, start, to, end)?;
        let (ba, bb) = (self.anchor_box(from)?, self.anchor_box(to)?);
        let c = Connection {
            from,
            from_anchor: fractions(ba, pa),
            to,
            to_anchor: fractions(bb, pb),
            kind,
            radius: self.connector_radius,
            ends: (pa, pb),
        };
        let layer = self.active_layer()?;
        let id = self.engine.new_shape_id();
        let mut s = Shape::new(
            id,
            ShapeKind::Path {
                path: route(kind, c.radius, pa, pb),
                closed: false,
            },
        );
        self.doc().defaults.apply_to(&mut s);
        s.fill = Fill::None;
        s.name = Some("Connector".into());
        s.data = with_connection(Vec::new(), Some(&c));
        if let Err(e) = self
            .engine
            .run_batch("Connector", &[Command::AddShape { layer, shape: s }])
        {
            self.status = e.to_string();
            return None;
        }
        self.select(vec![id]);
        Some(id)
    }

    /// The selected connectors.
    pub fn selected_connectors(&self) -> Vec<(ShapeId, Connection)> {
        self.selection
            .iter()
            .filter_map(|id| {
                let s = self.doc().find_shape(*id)?;
                Connection::of(s).map(|c| (*id, c))
            })
            .collect()
    }

    /// Change the selected connectors (how they run, their corner radius),
    /// one undo step; the bar's values for the next ones change too.
    pub fn set_connectors(&mut self, kind: Option<ConnectorKind>, radius: Option<f64>) {
        if let Some(r) = radius.filter(|r| r.is_finite()) {
            self.connector_radius = r.clamp(0.0, 1000.0);
        }
        let mut cmds = Vec::new();
        for (id, mut c) in self.selected_connectors() {
            if let Some(k) = kind {
                c.kind = k;
            }
            if radius.is_some() {
                c.radius = self.connector_radius;
            }
            let Some(s) = self.doc().shape_on_page(id) else {
                continue;
            };
            cmds.push(Command::SetShapeKind {
                shape: id,
                kind: ShapeKind::Path {
                    path: s.transform.inverse() * route(c.kind, c.radius, c.ends.0, c.ends.1),
                    closed: false,
                },
            });
            cmds.push(Command::SetObjectData {
                shape: id,
                data: with_connection(s.data.clone(), Some(&c)),
            });
        }
        if cmds.is_empty() {
            return;
        }
        // A radius dragged or typed on: one step while nothing else
        // changes in between.
        let label = if kind.is_none() {
            "Connector Radius"
        } else {
            "Connector"
        };
        let ids: Vec<ShapeId> = self.selection.clone();
        if label == "Connector Radius"
            && self.engine.undo_label() == Some(label)
            && self.connector_radius_step.as_ref() == Some(&(self.engine.revision(), ids.clone()))
        {
            let _ = self.engine.undo();
        }
        if let Err(e) = self.engine.run_batch(label, &cmds) {
            self.status = e.to_string();
            return;
        }
        self.connector_radius_step =
            (label == "Connector Radius").then(|| (self.engine.revision(), ids));
    }

    /// Connectors follow their objects (see the module notes). Runs once
    /// per change of the drawing; the changes join the step that made it.
    pub fn refresh_connectors(&mut self) {
        let revision = self.engine.revision();
        if revision == self.connectors_checked {
            return;
        }
        fn walk(s: &Shape, out: &mut Vec<ShapeId>) {
            if Connection::of(s).is_some() {
                out.push(s.id);
            }
            if let ShapeKind::Group { children } = &s.kind {
                children.iter().for_each(|c| walk(c, out));
            }
        }
        let mut ids = Vec::new();
        for layer in self.doc().all_layers() {
            for s in &layer.shapes {
                walk(s, &mut ids);
            }
        }
        for id in ids {
            let Some(s) = self.doc().shape_on_page(id) else {
                continue;
            };
            let (Some(c), ShapeKind::Path { path, .. }) = (Connection::of(&s), &s.kind) else {
                continue;
            };
            let Some((qa, qb)) = ends_of(&(s.transform * path.clone())) else {
                continue;
            };
            let anchors = self
                .anchor_box(c.from)
                .zip(self.anchor_box(c.to))
                .map(|(ba, bb)| (at(ba, c.from_anchor), at(bb, c.to_anchor)));
            let mut cmds = Vec::new();
            match anchors {
                // In place: nothing to do but remember where.
                Some((pa, pb)) if near(qa, pa) && near(qb, pb) => {
                    if !(near(c.ends.0, pa) && near(c.ends.1, pb)) {
                        let c2 = Connection {
                            ends: (pa, pb),
                            ..c.clone()
                        };
                        cmds.push(Command::SetObjectData {
                            shape: id,
                            data: with_connection(s.data.clone(), Some(&c2)),
                        });
                    }
                }
                // Its objects moved, the connector did not: it follows.
                Some((pa, pb)) if near(qa, c.ends.0) && near(qb, c.ends.1) => {
                    // The path in the connector's own space.
                    let local = s.transform.inverse();
                    cmds.push(Command::SetShapeKind {
                        shape: id,
                        kind: ShapeKind::Path {
                            path: local * route(c.kind, c.radius, pa, pb),
                            closed: false,
                        },
                    });
                    let c2 = Connection {
                        ends: (pa, pb),
                        ..c.clone()
                    };
                    cmds.push(Command::SetObjectData {
                        shape: id,
                        data: with_connection(s.data.clone(), Some(&c2)),
                    });
                }
                // Moved on its own, or an object is gone: let go.
                Some(_) => cmds.push(Command::SetObjectData {
                    shape: id,
                    data: with_connection(s.data.clone(), None),
                }),
                None => {}
            }
            for cmd in cmds {
                if let Err(e) = self.engine.amend(&cmd) {
                    self.status = e.to_string();
                    break;
                }
            }
        }
        self.connectors_checked = self.engine.revision();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Affine, Vec2};

    fn rect(app: &mut App, x0: f64, y0: f64) -> ShapeId {
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(x0, y0, x0 + 20.0, y0 + 20.0),
            radius: 0.0,
            corners: None,
        })
        .expect("shape")
    }

    fn ends(app: &App, id: ShapeId) -> (Point, Point) {
        let s = app.doc().shape_on_page(id).expect("connector");
        match &s.kind {
            ShapeKind::Path { path, .. } => ends_of(&(s.transform * path.clone())).expect("ends"),
            other => panic!("{other:?}"),
        }
    }

    /// A copy of a connected object comes with a copy of its connector,
    /// joining the object at the other end to the copy, of the same kind
    /// and in the same undo step; copied with its connector, or pasted
    /// from elsewhere, nothing more comes (#476).
    #[test]
    fn copies_of_connected_objects_bring_their_connector() {
        let mut app = App::headless();
        app.view.zoom = 1.0;
        let a = rect(&mut app, 0.0, 0.0);
        let b = rect(&mut app, 60.0, 0.0);
        let c = app
            .connect(
                a,
                Point::new(20.0, 10.0),
                b,
                Point::new(60.0, 10.0),
                ConnectorKind::RightAngle,
            )
            .expect("connector");
        app.refresh_connectors();
        let connectors = |app: &App| -> Vec<(ShapeId, Connection)> {
            app.page_layers()
                .iter()
                .flat_map(|l| &l.shapes)
                .filter_map(|s| Connection::of(s).map(|k| (s.id, k)))
                .collect()
        };
        let steps = app.engine.history_labels().0.len();
        app.duplicate_offset = tracedraw_core::geometry::Vec2::new(0.0, 40.0);
        app.select(vec![b]);
        app.duplicate();
        let copy = app.selection[0];
        assert_eq!(app.engine.history_labels().0.len(), steps + 1, "one step");
        let all = connectors(&app);
        assert_eq!(all.len(), 2);
        let (nid, n) = all
            .iter()
            .find(|(id, _)| *id != c)
            .cloned()
            .expect("a new connector");
        assert_eq!((n.from, n.to, n.kind), (a, copy, ConnectorKind::RightAngle));
        app.refresh_connectors();
        assert_eq!(
            ends(&app, nid),
            (Point::new(20.0, 10.0), Point::new(60.0, 50.0)),
            "from a to the copy's left side"
        );
        // The object and its connector copied together: no third one.
        app.select(vec![b, c]);
        app.duplicate();
        assert_eq!(connectors(&app).len(), 3);
    }

    /// A connector follows the objects it joins; moved with them it stays
    /// attached; moved on its own it is let go.
    #[test]
    fn connectors_follow_their_objects() {
        let mut app = App::headless();
        app.view.zoom = 1.0;
        let a = rect(&mut app, 0.0, 0.0);
        let b = rect(&mut app, 60.0, 0.0);
        // From a's right side to b's left side.
        let c = app
            .connect(
                a,
                Point::new(20.0, 10.0),
                b,
                Point::new(60.0, 10.0),
                ConnectorKind::Straight,
            )
            .expect("connector");
        app.refresh_connectors();
        assert_eq!(
            ends(&app, c),
            (Point::new(20.0, 10.0), Point::new(60.0, 10.0))
        );
        // b moves up: the connector follows in the same undo step.
        app.run(Command::TransformShapes {
            shapes: vec![b],
            transform: Affine::translate(Vec2::new(0.0, 30.0)),
        });
        let depth = app.engine.history_labels().0.len();
        app.refresh_connectors();
        assert_eq!(app.engine.history_labels().0.len(), depth);
        assert_eq!(
            ends(&app, c),
            (Point::new(20.0, 10.0), Point::new(60.0, 40.0))
        );
        // All three moved together: still attached.
        app.run(Command::TransformShapes {
            shapes: vec![a, b, c],
            transform: Affine::translate(Vec2::new(5.0, 0.0)),
        });
        app.refresh_connectors();
        let s = app.doc().find_shape(c).cloned().expect("connector");
        assert!(Connection::of(&s).is_some());
        app.run(Command::TransformShapes {
            shapes: vec![b],
            transform: Affine::translate(Vec2::new(10.0, 0.0)),
        });
        app.refresh_connectors();
        assert_eq!(ends(&app, c).1, Point::new(75.0, 40.0));
        // The connector alone moved: let go, it stays where it was put.
        app.run(Command::TransformShapes {
            shapes: vec![c],
            transform: Affine::translate(Vec2::new(0.0, -50.0)),
        });
        app.refresh_connectors();
        let s = app.doc().find_shape(c).cloned().expect("connector");
        assert!(Connection::of(&s).is_none());
        assert_eq!(ends(&app, c).0, Point::new(25.0, -40.0));
        // Undo brings the attachment back.
        app.undo();
        app.refresh_connectors();
        let s = app.doc().find_shape(c).cloned().expect("connector");
        assert!(Connection::of(&s).is_some());
    }

    #[test]
    fn right_angle_and_rounded_routes() {
        let (pa, pb) = (Point::new(0.0, 0.0), Point::new(40.0, 20.0));
        let straight = route(ConnectorKind::Straight, 3.0, pa, pb);
        assert_eq!(straight.elements().len(), 2);
        let elbow = route(ConnectorKind::RightAngle, 3.0, pa, pb);
        let pts: Vec<Point> = elbow
            .elements()
            .iter()
            .filter_map(|e| match e {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
                _ => None,
            })
            .collect();
        assert_eq!(
            pts,
            vec![pa, Point::new(20.0, 0.0), Point::new(20.0, 20.0), pb]
        );
        let rounded = route(ConnectorKind::Rounded, 3.0, pa, pb);
        assert_eq!(
            rounded
                .elements()
                .iter()
                .filter(|e| matches!(e, PathEl::QuadTo(..)))
                .count(),
            2
        );
        assert_eq!(ends_of(&rounded), Some((pa, pb)));
    }

    /// The bar changes how a selected connector runs and its radius.
    #[test]
    fn selected_connectors_change_kind() {
        let mut app = App::headless();
        let a = rect(&mut app, 0.0, 0.0);
        let b = rect(&mut app, 60.0, 40.0);
        let c = app
            .connect(
                a,
                Point::new(20.0, 10.0),
                b,
                Point::new(60.0, 50.0),
                ConnectorKind::Straight,
            )
            .expect("connector");
        app.select(vec![c]);
        app.set_connectors(Some(ConnectorKind::Rounded), Some(5.0));
        let s = app.doc().find_shape(c).cloned().expect("connector");
        let conn = Connection::of(&s).expect("connection");
        assert_eq!((conn.kind, conn.radius), (ConnectorKind::Rounded, 5.0));
        match &s.kind {
            ShapeKind::Path { path, .. } => {
                assert!(path
                    .elements()
                    .iter()
                    .any(|e| matches!(e, PathEl::QuadTo(..))))
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(app.connector_radius, 5.0);
    }
}
