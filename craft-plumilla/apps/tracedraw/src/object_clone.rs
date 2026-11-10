//! Edit > Clone: a copy linked to its master. Until the clone's own value
//! of a property is edited, it follows the master's fill, outline and
//! shape (for a group or a frame: everything it holds). The clone keeps
//! its own place, size and turn.
//!
//! The link lives in the clone's object data, so it survives save and
//! load: `clone.master` holds the master's id and `clone.state` one entry
//! per followed property, `name:<master hash>/<clone hash>` with the hashes
//! of both values at the last update, or `name:own` once the clone took
//! its own value. After each recorded step [`App::sync_object_clones`]
//! compares the hashes: a clone whose value changed has overridden it; a
//! master whose value changed passes the new value on. The update is
//! added to that same undo step, so one Undo takes both back.

use crate::app::App;
use crate::i18n::tr;
use tracedraw_core::{
    document::{Shape, ShapeKind},
    Command, ShapeId,
};

const MASTER: &str = "clone.master";
const STATE: &str = "clone.state";

/// A property a clone follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Fill,
    Outline,
    Shape,
}

impl Prop {
    fn name(self) -> &'static str {
        match self {
            Prop::Fill => "fill",
            Prop::Outline => "outline",
            Prop::Shape => "shape",
        }
    }

    fn parse(s: &str) -> Option<Prop> {
        match s {
            "fill" => Some(Prop::Fill),
            "outline" => Some(Prop::Outline),
            "shape" => Some(Prop::Shape),
            _ => None,
        }
    }
}

/// Where a followed property stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Follow {
    /// Hashes of the master's and the clone's value at the last update.
    Linked(u64, u64),
    /// The clone was given its own value.
    Own,
}

/// The properties an object of this kind follows: a group or a frame as
/// a whole (its contents carry their own fills), anything else its fill,
/// outline and shape.
fn props_of(s: &Shape) -> &'static [Prop] {
    match s.kind {
        ShapeKind::Group { .. } | ShapeKind::ClipFrame { .. } => &[Prop::Shape],
        _ => &[Prop::Fill, Prop::Outline, Prop::Shape],
    }
}

/// FNV-1a over the JSON form: stable across runs, unlike the std hasher.
fn hash_json<T: serde::Serialize>(v: &T) -> u64 {
    let text = serde_json::to_string(v).unwrap_or_default();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// A kind with the ids of the objects inside it cleared, so a copy with
/// new ids hashes the same.
fn canonical(kind: &ShapeKind) -> ShapeKind {
    fn clear(s: &mut Shape) {
        s.id = ShapeId(0);
        s.uid = None;
        match &mut s.kind {
            ShapeKind::Group { children } => children.iter_mut().for_each(clear),
            ShapeKind::ClipFrame { frame, contents } => {
                clear(frame);
                contents.iter_mut().for_each(clear);
            }
            _ => {}
        }
    }
    let mut k = kind.clone();
    match &mut k {
        ShapeKind::Group { children } => children.iter_mut().for_each(clear),
        ShapeKind::ClipFrame { frame, contents } => {
            clear(frame);
            contents.iter_mut().for_each(clear);
        }
        _ => {}
    }
    k
}

fn hash_prop(s: &Shape, p: Prop) -> u64 {
    match p {
        Prop::Fill => hash_json(&s.fill),
        Prop::Outline => hash_json(&s.stroke),
        Prop::Shape => hash_json(&canonical(&s.kind)),
    }
}

/// The master and the state of each property, from object data.
pub fn clone_link(data: &[(String, String)]) -> Option<(ShapeId, Vec<(Prop, Follow)>)> {
    let master = data.iter().find(|(k, _)| k == MASTER)?;
    let master = ShapeId(master.1.parse().ok()?);
    let state = data
        .iter()
        .find(|(k, _)| k == STATE)
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    let mut out = Vec::new();
    for part in state.split(';').filter(|p| !p.is_empty()) {
        let Some((name, v)) = part.split_once(':') else {
            continue;
        };
        let Some(p) = Prop::parse(name) else {
            continue;
        };
        let f = if v == "own" {
            Follow::Own
        } else {
            let Some((m, c)) = v.split_once('/') else {
                continue;
            };
            match (u64::from_str_radix(m, 16), u64::from_str_radix(c, 16)) {
                (Ok(m), Ok(c)) => Follow::Linked(m, c),
                _ => continue,
            }
        };
        out.push((p, f));
    }
    Some((master, out))
}

fn link_data(
    data: &[(String, String)],
    master: ShapeId,
    state: &[(Prop, Follow)],
) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = data
        .iter()
        .filter(|(k, _)| k != MASTER && k != STATE)
        .cloned()
        .collect();
    let text: Vec<String> = state
        .iter()
        .map(|(p, f)| match f {
            Follow::Own => format!("{}:own", p.name()),
            Follow::Linked(m, c) => format!("{}:{m:x}/{c:x}", p.name()),
        })
        .collect();
    out.push((MASTER.into(), master.raw().to_string()));
    out.push((STATE.into(), text.join(";")));
    out
}

fn unlinked_data(data: &[(String, String)]) -> Vec<(String, String)> {
    data.iter()
        .filter(|(k, _)| k != MASTER && k != STATE)
        .cloned()
        .collect()
}

fn walk<'a>(s: &'a Shape, out: &mut Vec<&'a Shape>) {
    out.push(s);
    match &s.kind {
        ShapeKind::Group { children } => children.iter().for_each(|c| walk(c, out)),
        ShapeKind::ClipFrame { frame, contents } => {
            walk(frame, out);
            contents.iter().for_each(|c| walk(c, out));
        }
        _ => {}
    }
}

/// A copy of a container's kind with new ids for everything inside.
fn fresh_kind(kind: &ShapeKind, engine: &mut tracedraw_core::Engine) -> ShapeKind {
    let mut tmp = Shape::new(ShapeId(0), kind.clone());
    tmp = crate::app::reid_pub(tmp, ShapeId(0), engine);
    tmp.kind
}

impl App {
    /// Every (clone, master) pair in the drawing.
    pub fn object_clones(&self) -> Vec<(ShapeId, ShapeId)> {
        let mut all = Vec::new();
        for l in self.doc().all_layers() {
            for s in &l.shapes {
                walk(s, &mut all);
            }
        }
        all.iter()
            .filter_map(|s| clone_link(&s.data).map(|(m, _)| (s.id, m)))
            .collect()
    }

    /// The master of a clone.
    pub fn clone_master_of(&self, id: ShapeId) -> Option<ShapeId> {
        let s = self.doc().find_shape(id)?;
        clone_link(&s.data).map(|(m, _)| m)
    }

    /// Edit > Clone: a linked copy of each selected object, at the
    /// duplicate offset, selected. Clones of clones are not made (a
    /// selected clone is left out, with a note).
    pub fn clone_selection(&mut self) {
        let shapes = self.selected_shapes();
        let Some(layer) = self.active_layer() else {
            return;
        };
        let mut ids = Vec::new();
        let mut cmds = Vec::new();
        let mut skipped = false;
        for s in shapes {
            if clone_link(&s.data).is_some() {
                skipped = true;
                continue;
            }
            let id = self.engine.new_shape_id();
            let mut shape = crate::app::reid_pub(s.clone(), id, &mut self.engine);
            shape.transform =
                tracedraw_core::Affine::translate(self.duplicate_offset) * shape.transform;
            let state: Vec<(Prop, Follow)> = props_of(&s)
                .iter()
                .map(|p| (*p, Follow::Linked(hash_prop(&s, *p), hash_prop(&shape, *p))))
                .collect();
            shape.data = link_data(&shape.data, s.id, &state);
            shape.uid = None;
            ids.push(id);
            cmds.push(Command::AddShape { layer, shape });
        }
        if skipped {
            self.status = tr("status.clone_of_clone");
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Clone", &cmds) {
            self.status = e.to_string();
            return;
        }
        self.clone_sync_revision = self.engine.revision();
        self.select(ids);
    }

    /// Edit > Clone is offered: something selected that is not a clone.
    pub fn can_clone(&self) -> bool {
        self.selected_shapes()
            .iter()
            .any(|s| clone_link(&s.data).is_none())
    }

    /// Context menu > Select Master: the masters of the selected clones.
    pub fn select_clone_masters(&mut self) {
        let doc = self.doc();
        let mut masters: Vec<ShapeId> = Vec::new();
        for id in &self.selection {
            if let Some(m) = self.clone_master_of(*id) {
                if doc.find_shape(m).is_some() && !masters.contains(&m) {
                    masters.push(m);
                }
            }
        }
        if !masters.is_empty() {
            self.select(masters);
        }
    }

    /// Context menu > Select Clones: the clones of the selected masters.
    pub fn select_clones_of_selection(&mut self) {
        let clones: Vec<ShapeId> = self
            .object_clones()
            .into_iter()
            .filter(|(_, m)| self.selection.contains(m))
            .map(|(c, _)| c)
            .collect();
        if !clones.is_empty() {
            self.select(clones);
        }
    }

    /// Something selected is a clone whose master still exists.
    pub fn selection_has_clone(&self) -> bool {
        let doc = self.doc();
        self.selection.iter().any(|id| {
            self.clone_master_of(*id)
                .is_some_and(|m| doc.find_shape(m).is_some())
        })
    }

    /// Something selected has clones.
    pub fn selection_has_clones(&self) -> bool {
        self.object_clones()
            .iter()
            .any(|(_, m)| self.selection.contains(m))
    }

    /// Context menu > Revert to Master: the selected clones take the
    /// master's fill, outline and shape again and follow them from now on.
    pub fn revert_clones_to_master(&mut self) {
        let mut cmds = Vec::new();
        for id in self.selection.clone() {
            let Some(clone) = self.doc().find_shape(id).cloned() else {
                continue;
            };
            let Some((master_id, _)) = clone_link(&clone.data) else {
                continue;
            };
            let Some(master) = self.doc().find_shape(master_id).cloned() else {
                continue;
            };
            let mut after = clone.clone();
            for p in props_of(&master) {
                self.push_prop(&master, &mut after, *p, &mut cmds);
            }
            let state: Vec<(Prop, Follow)> = props_of(&master)
                .iter()
                .map(|p| {
                    (
                        *p,
                        Follow::Linked(hash_prop(&master, *p), hash_prop(&after, *p)),
                    )
                })
                .collect();
            cmds.push(Command::SetObjectData {
                shape: id,
                data: link_data(&clone.data, master_id, &state),
            });
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Revert to Master", &cmds) {
            self.status = e.to_string();
        }
        self.clone_sync_revision = self.engine.revision();
    }

    /// The command that gives `clone` the master's value of `p`; `clone`
    /// is updated to match.
    fn push_prop(&mut self, master: &Shape, clone: &mut Shape, p: Prop, cmds: &mut Vec<Command>) {
        match p {
            Prop::Fill => {
                clone.fill = master.fill.clone();
                cmds.push(Command::SetFill {
                    shapes: vec![clone.id],
                    fill: master.fill.clone(),
                });
            }
            Prop::Outline => {
                clone.stroke = master.stroke.clone();
                cmds.push(Command::SetStroke {
                    shapes: vec![clone.id],
                    stroke: master.stroke.clone(),
                });
            }
            Prop::Shape => {
                let kind = match master.kind {
                    ShapeKind::Group { .. } | ShapeKind::ClipFrame { .. } => {
                        fresh_kind(&master.kind, &mut self.engine)
                    }
                    _ => master.kind.clone(),
                };
                clone.kind = kind.clone();
                cmds.push(Command::SetShapeKind {
                    shape: clone.id,
                    kind,
                });
                if clone.node_types != master.node_types {
                    clone.node_types = master.node_types.clone();
                    cmds.push(Command::SetNodeTypes {
                        shape: clone.id,
                        types: master.node_types.clone(),
                    });
                }
            }
        }
    }

    /// After a recorded step: clones whose own value changed keep it from
    /// now on; masters whose value changed pass it on to their clones. The
    /// changes join the step just recorded. Nothing happens after an undo
    /// or a redo (the snapshot already holds clones and masters together).
    pub fn sync_object_clones(&mut self) {
        let revision = self.engine.revision();
        if revision == self.clone_sync_revision {
            return;
        }
        self.clone_sync_revision = revision;
        if !self.engine.at_recorded_step() {
            return;
        }
        let pairs = self.object_clones();
        if pairs.is_empty() {
            return;
        }
        let mut cmds = Vec::new();
        for (clone_id, master_id) in pairs {
            let Some(clone) = self.doc().find_shape(clone_id).cloned() else {
                continue;
            };
            let Some((_, state)) = clone_link(&clone.data) else {
                continue;
            };
            let Some(master) = self.doc().find_shape(master_id).cloned() else {
                continue;
            };
            let mut after = clone.clone();
            let mut new_state = Vec::with_capacity(state.len());
            let mut changed = false;
            for (p, f) in state {
                let next = match f {
                    Follow::Own => Follow::Own,
                    Follow::Linked(m, c) => {
                        if hash_prop(&clone, p) != c {
                            changed = true;
                            Follow::Own
                        } else if hash_prop(&master, p) != m {
                            changed = true;
                            self.push_prop(&master, &mut after, p, &mut cmds);
                            Follow::Linked(hash_prop(&master, p), hash_prop(&after, p))
                        } else {
                            f
                        }
                    }
                };
                new_state.push((p, next));
            }
            if changed {
                cmds.push(Command::SetObjectData {
                    shape: clone_id,
                    data: link_data(&clone.data, master_id, &new_state),
                });
            }
        }
        for c in &cmds {
            if let Err(e) = self.engine.amend(c) {
                log::warn!("clone update: {e}");
            }
        }
        self.clone_sync_revision = self.engine.revision();
    }

    /// Break the link of the selected clones: they become plain objects.
    pub fn unlink_clones(&mut self) {
        let mut cmds = Vec::new();
        for id in &self.selection {
            if let Some(s) = self.doc().find_shape(*id) {
                if clone_link(&s.data).is_some() {
                    cmds.push(Command::SetObjectData {
                        shape: *id,
                        data: unlinked_data(&s.data),
                    });
                }
            }
        }
        if !cmds.is_empty() {
            let _ = self.engine.run_batch("Break Clone Link", &cmds);
            self.clone_sync_revision = self.engine.revision();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{geometry::Rect, Color, Fill, Stroke};

    fn rect(app: &mut App, x: f64) -> ShapeId {
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(x, 10.0, x + 20.0, 30.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.sync_object_clones();
        id
    }

    fn fill_of(app: &App, id: ShapeId) -> Fill {
        app.doc().find_shape(id).expect("shape").fill.clone()
    }

    fn red() -> Fill {
        Fill::Solid(Color::rgb8(255, 0, 0))
    }

    fn blue() -> Fill {
        Fill::Solid(Color::rgb8(0, 0, 255))
    }

    fn cloned(app: &mut App, master: ShapeId) -> ShapeId {
        app.select(vec![master]);
        app.clone_selection();
        let clone = app.selection[0];
        assert_ne!(clone, master);
        assert_eq!(app.clone_master_of(clone), Some(master));
        clone
    }

    #[test]
    fn a_clone_follows_its_master_until_it_is_edited() {
        let mut app = App::headless();
        let master = rect(&mut app, 10.0);
        let clone = cloned(&mut app, master);
        // The master's fill, outline and shape pass on.
        app.run(Command::SetFill {
            shapes: vec![master],
            fill: red(),
        });
        assert_eq!(fill_of(&app, clone), red());
        let thick = Some(Stroke::hairline(Color::rgb8(0, 128, 0)));
        app.run(Command::SetStroke {
            shapes: vec![master],
            stroke: thick.clone(),
        });
        assert_eq!(app.doc().find_shape(clone).expect("clone").stroke, thick);
        app.run(Command::SetShapeKind {
            shape: master,
            kind: ShapeKind::Ellipse {
                rect: Rect::new(10.0, 10.0, 30.0, 40.0),
                arc: None,
            },
        });
        assert!(matches!(
            app.doc().find_shape(clone).map(|s| &s.kind),
            Some(ShapeKind::Ellipse { .. })
        ));
        // One Undo takes the master's change and the clone's back.
        app.undo();
        assert!(matches!(
            app.doc().find_shape(clone).map(|s| &s.kind),
            Some(ShapeKind::Rect { .. })
        ));
        assert!(matches!(
            app.doc().find_shape(master).map(|s| &s.kind),
            Some(ShapeKind::Rect { .. })
        ));
        // The clone's own fill stops it following the master's fill only.
        app.run(Command::SetFill {
            shapes: vec![clone],
            fill: blue(),
        });
        app.run(Command::SetFill {
            shapes: vec![master],
            fill: red(),
        });
        app.run(Command::SetFill {
            shapes: vec![master],
            fill: Fill::None,
        });
        assert_eq!(fill_of(&app, clone), blue());
        let other = Some(Stroke::hairline(Color::BLACK));
        app.run(Command::SetStroke {
            shapes: vec![master],
            stroke: other.clone(),
        });
        assert_eq!(app.doc().find_shape(clone).expect("clone").stroke, other);
        // Moving the master leaves the clone where it is.
        let before = app.doc().find_shape(clone).expect("clone").bounds();
        app.run(Command::TransformShapes {
            shapes: vec![master],
            transform: tracedraw_core::Affine::translate((50.0, 0.0)),
        });
        assert_eq!(app.doc().find_shape(clone).expect("clone").bounds(), before);
        // Revert to Master: the master's fill again, and following again.
        app.select(vec![clone]);
        assert!(app.selection_has_clone());
        app.revert_clones_to_master();
        assert_eq!(fill_of(&app, clone), Fill::None);
        app.run(Command::SetFill {
            shapes: vec![master],
            fill: red(),
        });
        assert_eq!(fill_of(&app, clone), red());
    }

    #[test]
    fn select_master_and_clones() {
        let mut app = App::headless();
        let master = rect(&mut app, 10.0);
        let a = cloned(&mut app, master);
        let b = cloned(&mut app, master);
        app.select(vec![a]);
        app.select_clone_masters();
        assert_eq!(app.selection, vec![master]);
        assert!(app.selection_has_clones());
        app.select_clones_of_selection();
        let mut got = app.selection.clone();
        got.sort();
        assert_eq!(got, vec![a, b]);
        // No clone of a clone.
        app.select(vec![a]);
        assert!(!app.can_clone());
        app.clone_selection();
        assert_eq!(app.selection, vec![a]);
        assert_eq!(app.status, tr("status.clone_of_clone"));
    }

    #[test]
    fn a_group_clone_follows_the_group_with_ids_of_its_own() {
        let mut app = App::headless();
        let a = rect(&mut app, 10.0);
        let b = rect(&mut app, 40.0);
        app.select(vec![a, b]);
        app.group_selection();
        let group = app.selection[0];
        app.sync_object_clones();
        let clone = cloned(&mut app, group);
        app.run(Command::SetFill {
            shapes: vec![a],
            fill: red(),
        });
        let doc = app.doc();
        let ShapeKind::Group { children } = &doc.find_shape(clone).expect("clone").kind else {
            panic!("a group");
        };
        assert_eq!(children.len(), 2);
        assert!(children.iter().any(|c| c.fill == red()));
        assert!(children.iter().all(|c| c.id != a && c.id != b));
    }

    #[test]
    fn broken_link_data_is_ignored() {
        let data = vec![
            (MASTER.to_string(), "x".to_string()),
            (STATE.to_string(), "fill:zz".to_string()),
        ];
        assert!(clone_link(&data).is_none());
        let data = vec![
            (MASTER.to_string(), "7".to_string()),
            (
                STATE.to_string(),
                "fill:zz/1;bogus:own;outline;shape:own".to_string(),
            ),
        ];
        let (m, st) = clone_link(&data).expect("a link");
        assert_eq!(m, ShapeId(7));
        assert_eq!(st, vec![(Prop::Shape, Follow::Own)]);
    }
}
