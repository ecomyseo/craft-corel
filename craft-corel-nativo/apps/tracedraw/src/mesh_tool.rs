//! The Mesh Fill tool. A click on an object gives it a mesh fill of the
//! property bar's rows and columns (or selects an object that has one);
//! its nodes are clicked, dragged and coloured from the colour palette;
//! a double-click adds a grid line through the nearest node and Delete
//! removes the lines through the selected node. The property bar sets the
//! grid, Smooth colour, the selected node's transparency and Clear Mesh.

use crate::app::App;
use crate::i18n::tr;
use egui::{PointerButton, Response};
use tracedraw_core::{
    geometry::{Point, Shape as _},
    Color, Command, Fill, Mesh, ShapeId,
};

/// Mesh nodes count from 200 in the effect node numbers (see
/// `interaction2::effect_nodes`).
pub const MESH_NODE_BASE: usize = 200;

impl App {
    /// The selected mesh node of a selected object: (object, node).
    pub fn selected_mesh_node(&self) -> Option<(ShapeId, usize)> {
        let (id, k) = self.selected_effect_node?;
        let i = k.checked_sub(MESH_NODE_BASE)?;
        if !self.selection.contains(&id) {
            return None;
        }
        match &self.doc().find_shape(id)?.fill {
            Fill::Mesh(m) if i < m.nodes.len() => Some((id, i)),
            _ => None,
        }
    }

    /// The first selected object with a mesh fill, and its mesh.
    pub fn selected_mesh(&self) -> Option<(ShapeId, Mesh)> {
        self.selection
            .iter()
            .find_map(|id| match &self.doc().find_shape(*id)?.fill {
                Fill::Mesh(m) => Some((*id, m.clone())),
                _ => None,
            })
    }

    /// Give object `id` a mesh fill of the bar's grid over its bounds, its
    /// fill's colour in every node; one undo step.
    pub fn apply_mesh_fill(&mut self, id: ShapeId) {
        let Some(s) = self.doc().find_shape(id).cloned() else {
            return;
        };
        let b = s.local_path().bounding_box();
        if !(b.width() > 0.0 && b.height() > 0.0) {
            return;
        }
        let base = s.fill.preview_color().unwrap_or(Color::WHITE);
        let mesh = Mesh::new(
            b,
            self.mesh_rows.clamp(1, 50),
            self.mesh_cols.clamp(1, 50),
            base,
        );
        self.run(Command::SetFill {
            shapes: vec![id],
            fill: Fill::Mesh(mesh),
        });
        self.selected_effect_node = None;
    }

    /// The Mesh Fill tool's clicks and drags.
    pub fn mesh_tool_input(&mut self, response: &Response, p: Point) {
        if self.effect_node_input(response, p) {
            return;
        }
        if response.clicked_by(PointerButton::Primary) {
            // Inside an object counts, filled or not: the mesh fills it.
            match self.hit_test_inside(p) {
                Some(id) => {
                    self.select(vec![id]);
                    self.selected_effect_node = None;
                    let meshed = self
                        .doc()
                        .find_shape(id)
                        .is_some_and(|s| matches!(s.fill, Fill::Mesh(_)));
                    if !meshed {
                        self.apply_mesh_fill(id);
                    }
                }
                None => {
                    self.select(Vec::new());
                    self.selected_effect_node = None;
                }
            }
        }
    }

    /// Change the selected meshes with `f`, one undo step named `label`.
    fn edit_meshes(&mut self, label: &'static str, f: impl Fn(ShapeId, &mut Mesh)) -> bool {
        let mut cmds = Vec::new();
        for id in self.selection.clone() {
            let Some(s) = self.doc().find_shape(id) else {
                continue;
            };
            let Fill::Mesh(m) = &s.fill else {
                continue;
            };
            let mut m2 = m.clone();
            f(id, &mut m2);
            if &m2 != m {
                cmds.push(Command::SetFill {
                    shapes: vec![id],
                    fill: Fill::Mesh(m2),
                });
            }
        }
        if cmds.is_empty() {
            return false;
        }
        if let Err(e) = self.engine.run_batch(label, &cmds) {
            self.status = e.to_string();
            return false;
        }
        true
    }

    /// The bar's grid: new meshes take it and the selected ones change to
    /// it, keeping their colours where the old grid had them.
    pub fn set_mesh_grid(&mut self, rows: u32, cols: u32) {
        self.mesh_rows = rows.clamp(1, 50);
        self.mesh_cols = cols.clamp(1, 50);
        let (r, c) = (self.mesh_rows, self.mesh_cols);
        if self.edit_meshes("Mesh Grid", |_, m| {
            if (m.rows, m.cols) != (r, c) {
                *m = m.regrid(r, c);
            }
        }) {
            self.selected_effect_node = None;
        }
    }

    /// Smooth colour on or off for the selected meshes.
    pub fn set_mesh_smooth(&mut self, on: bool) {
        self.edit_meshes("Smooth Mesh Colour", |_, m| m.smooth = on);
    }

    /// The selected node's opacity (1 opaque).
    pub fn set_mesh_node_alpha(&mut self, alpha: f32) {
        let Some((sid, i)) = self.selected_mesh_node() else {
            return;
        };
        let alpha = if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self.edit_meshes("Mesh Node Transparency", |id, m| {
            if id == sid {
                if let Some(n) = m.nodes.get_mut(i) {
                    n.alpha = alpha;
                }
            }
        });
    }

    /// A palette colour for the selected mesh node (with the Mesh Fill or
    /// Shape tool). False when no node is selected.
    pub fn color_mesh_node(&mut self, c: Color) -> bool {
        let Some((sid, i)) = self.selected_mesh_node() else {
            return false;
        };
        self.edit_meshes("Mesh Node Colour", |id, m| {
            if id == sid {
                if let Some(n) = m.nodes.get_mut(i) {
                    n.color = c;
                }
            }
        });
        true
    }

    /// Delete with a mesh node selected: the grid lines through it go
    /// (not those on the edge).
    pub fn delete_mesh_lines(&mut self) -> bool {
        let Some((sid, i)) = self.selected_mesh_node() else {
            return false;
        };
        let removed = self.edit_meshes("Delete Mesh Lines", |id, m| {
            if id == sid {
                m.remove_lines_through(i);
            }
        });
        if removed {
            self.selected_effect_node = None;
        } else {
            self.status = tr("status.mesh_edge_stays");
        }
        true
    }

    /// Clear Mesh: the selected objects lose their mesh fills (no fill).
    pub fn clear_mesh(&mut self) {
        let shapes: Vec<ShapeId> = self
            .selection
            .iter()
            .copied()
            .filter(|id| {
                self.doc()
                    .find_shape(*id)
                    .is_some_and(|s| matches!(s.fill, Fill::Mesh(_)))
            })
            .collect();
        if shapes.is_empty() {
            return;
        }
        self.run(Command::SetFill {
            shapes,
            fill: Fill::None,
        });
        self.selected_effect_node = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{Rect, ShapeKind};

    fn square(app: &mut App) -> ShapeId {
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 40.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        id
    }

    fn mesh(app: &App, id: ShapeId) -> Mesh {
        match &app.doc().find_shape(id).expect("shape").fill {
            Fill::Mesh(m) => m.clone(),
            other => panic!("{other:?}"),
        }
    }

    /// The tool's grid makes the mesh; a palette colour paints the selected
    /// node (the fill stays a mesh); transparency, smooth colour, Delete
    /// and Clear Mesh act on it.
    #[test]
    fn the_mesh_tool_builds_colours_and_clears_meshes() {
        let mut app = App::headless();
        let id = square(&mut app);
        app.tool = crate::tools::Tool::MeshFill;
        app.mesh_rows = 3;
        app.mesh_cols = 2;
        app.apply_mesh_fill(id);
        let m = mesh(&app, id);
        assert_eq!((m.rows, m.cols, m.nodes.len()), (3, 2, 12));
        // No node chosen: a palette colour fills the object as usual.
        assert!(!app.color_mesh_node(Color::rgb8(255, 0, 0)));
        // Node 4 (row 1, column 1) chosen: only it turns red.
        app.selected_effect_node = Some((id, MESH_NODE_BASE + 4));
        app.apply_fill(Fill::Solid(Color::rgb8(255, 0, 0)));
        let m = mesh(&app, id);
        assert_eq!(m.nodes[4].color, Color::rgb8(255, 0, 0));
        assert_ne!(m.nodes[3].color, Color::rgb8(255, 0, 0));
        app.set_mesh_node_alpha(0.25);
        assert_eq!(mesh(&app, id).nodes[4].alpha, 0.25);
        app.set_mesh_smooth(true);
        assert!(mesh(&app, id).smooth);
        // A new grid keeps the colours where they were.
        app.set_mesh_grid(6, 4);
        let m = mesh(&app, id);
        assert_eq!((m.rows, m.cols), (6, 4));
        assert_eq!(m.node(2, 2).expect("node").color, Color::rgb8(255, 0, 0));
        assert!(m.smooth);
        // Delete: the lines through an inner node go; an edge node's stay.
        app.selected_effect_node = Some((id, MESH_NODE_BASE + 6)); // row 1, column 1
        assert!(app.delete_mesh_lines());
        let m = mesh(&app, id);
        assert_eq!((m.rows, m.cols), (5, 3));
        app.selected_effect_node = Some((id, MESH_NODE_BASE));
        assert!(app.delete_mesh_lines());
        assert_eq!((mesh(&app, id).rows, mesh(&app, id).cols), (5, 3));
        app.clear_mesh();
        assert_eq!(app.doc().find_shape(id).expect("shape").fill, Fill::None);
    }
}
