//! Remove From Group (the object's right-click menu on the page and in the
//! Objects docker) and Convert Connector (see docs/behavior/pick-tool.md
//! and docs/behavior/connectors-and-anchors.md).

use crate::app::App;
use crate::connectors::ConnectorKind;
use tracedraw_core::{Command, ShapeId, ShapeKind};

impl App {
    /// The group directly holding `id` when that group lies on a layer:
    /// the one Remove From Group takes the object out of.
    pub fn group_to_leave(&self, id: ShapeId) -> Option<ShapeId> {
        let anc = self.doc().ancestors(id);
        let [g] = anc.as_slice() else {
            return None;
        };
        matches!(self.doc().find_shape(*g)?.kind, ShapeKind::Group { .. }).then_some(*g)
    }

    /// Remove From Group is offered: a chosen object lies in a group on a
    /// layer.
    pub fn can_remove_from_group(&self, ids: &[ShapeId]) -> bool {
        ids.iter().any(|id| self.group_to_leave(*id).is_some())
    }

    /// Remove From Group: each of `ids` inside a group on a layer comes
    /// out of it, just in front of the group and where it was on the page,
    /// keeping their order; a group left empty goes. One undo step; the
    /// objects come out selected.
    pub fn remove_from_group(&mut self, ids: &[ShapeId]) -> bool {
        // By group, each group's leavers in the order they are drawn.
        let mut by_group: Vec<(ShapeId, Vec<ShapeId>)> = Vec::new();
        for id in ids {
            let Some(g) = self.group_to_leave(*id) else {
                continue;
            };
            match by_group.iter_mut().find(|(x, _)| *x == g) {
                Some((_, v)) => v.push(*id),
                None => by_group.push((g, vec![*id])),
            }
        }
        if by_group.is_empty() {
            return false;
        }
        let mut cmds = Vec::new();
        let mut out = Vec::new();
        for (g, mut leavers) in by_group {
            let Some(group) = self.doc().find_shape(g).cloned() else {
                continue;
            };
            let ShapeKind::Group { children } = &group.kind else {
                continue;
            };
            let place = |id: &ShapeId| children.iter().position(|c| c.id == *id).unwrap_or(0);
            leavers.sort_by_key(place);
            let emptied = children.len() == leavers.len();
            let Ok((layer, index)) = self.doc().locate(g) else {
                continue;
            };
            let copies: Vec<_> = leavers
                .iter()
                .filter_map(|id| self.doc().shape_on_page(*id))
                .collect();
            if copies.len() != leavers.len() {
                continue;
            }
            cmds.push(Command::DeleteShapes {
                shapes: leavers.clone(),
            });
            for (k, shape) in copies.into_iter().enumerate() {
                let id = shape.id;
                cmds.push(Command::AddShape { layer, shape });
                if emptied {
                    // The group went with its last member: the objects take
                    // its place.
                    cmds.push(Command::Reorder {
                        shape: id,
                        layer,
                        index: index + k,
                    });
                }
            }
            if !emptied {
                cmds.push(Command::OrderRelative {
                    shapes: leavers.clone(),
                    reference: g,
                    in_front: true,
                });
            }
            out.extend(leavers);
        }
        if cmds.is_empty() {
            return false;
        }
        if let Err(e) = self.engine.run_batch("Remove From Group", &cmds) {
            self.status = e.to_string();
            return false;
        }
        self.select(out);
        true
    }

    /// Convert Connector: the selected connectors run straight, at right
    /// angles or at rounded right angles (as the property bar's buttons).
    pub fn convert_connectors(&mut self, kind: ConnectorKind) {
        self.set_connectors(Some(kind), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{geometry::Rect, Point, Vec2};

    fn square(app: &mut App, x: f64) -> ShapeId {
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(x, 0.0, x + 10.0, 10.0),
            radius: 0.0,
            corners: None,
        })
        .expect("square")
    }

    fn grouped() -> (App, ShapeId, [ShapeId; 3]) {
        let mut app = App::headless();
        let ids = [square(&mut app, 0.0), square(&mut app, 20.0), square(&mut app, 40.0)];
        app.select(ids.to_vec());
        app.group_selection();
        let g = app.selection[0];
        // A moved group: its members are placed through it.
        app.nudge(Vec2::new(5.0, 7.0));
        (app, g, ids)
    }

    fn page_box(app: &App, id: ShapeId) -> Rect {
        app.doc().shape_on_page(id).expect("object").bounds()
    }

    /// One member comes out in front of the group, where it was; the
    /// group keeps the others; one undo step.
    #[test]
    fn a_member_comes_out_in_front_of_its_group() {
        let (mut app, g, [a, b, c]) = grouped();
        let before = page_box(&app, b);
        assert_eq!(app.group_to_leave(b), Some(g));
        assert_eq!(app.group_to_leave(g), None);
        assert!(app.remove_from_group(&[b]));
        assert_eq!(app.selection, vec![b]);
        assert!(app.doc().shape(b).is_ok(), "on the layer");
        assert_eq!(app.group_to_leave(b), None);
        let r = page_box(&app, b);
        assert!((r.x0 - before.x0).abs() < 1e-9 && (r.y0 - before.y0).abs() < 1e-9, "{r:?} {before:?}");
        assert!(app.stacked_below(g, b), "in front of the group");
        match &app.doc().find_shape(g).expect("group").kind {
            ShapeKind::Group { children } => {
                assert_eq!(children.iter().map(|s| s.id).collect::<Vec<_>>(), vec![a, c]);
            }
            k => panic!("{k:?}"),
        }
        app.undo();
        assert_eq!(app.group_to_leave(b), Some(g));
        assert_eq!(page_box(&app, b), before);
    }

    /// All the members out: the group goes and they take its place in
    /// their order.
    #[test]
    fn the_last_members_out_take_the_group_s_place() {
        let (mut app, g, [a, b, c]) = grouped();
        let top = square(&mut app, 80.0);
        assert!(app.remove_from_group(&[c, a, b]));
        assert!(app.doc().find_shape(g).is_none());
        let order: Vec<ShapeId> = app.page_layers()[0].shapes.iter().map(|s| s.id).collect();
        let pos = |id| order.iter().position(|x| *x == id).expect("on the layer");
        assert!(pos(a) < pos(b) && pos(b) < pos(c) && pos(c) < pos(top), "{order:?}");
        assert!((page_box(&app, a).x0 - 5.0).abs() < 1e-9);
    }

    /// The right-click menu's Convert Connector runs the connectors
    /// another way.
    #[test]
    fn convert_connector_changes_how_it_runs() {
        let mut app = App::headless();
        let a = square(&mut app, 0.0);
        let b = square(&mut app, 50.0);
        let id = app
            .connect(a, Point::new(10.0, 5.0), b, Point::new(50.0, 5.0), ConnectorKind::Straight)
            .expect("connector");
        app.select(vec![id]);
        app.convert_connectors(ConnectorKind::RightAngle);
        assert_eq!(app.selected_connectors()[0].1.kind, ConnectorKind::RightAngle);
        app.convert_connectors(ConnectorKind::Straight);
        assert_eq!(app.selected_connectors()[0].1.kind, ConnectorKind::Straight);
    }
}
