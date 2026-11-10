//! The object model VBA macros for drawing suites use, for JavaScript
//! compiled from VBA (`scripting::run_vgcore`): `Application`,
//! `ActiveDocument`, `ActivePage`, `ActiveLayer`, `ActiveSelection`,
//! 1-based `Shapes` and `ShapeRange`, `Shape` with its `Fill`, `Outline`,
//! `Curve` and `Text`, `Color`, lengths in `ActiveDocument.Unit` and the
//! `cdr*` constants, under the names and values those macros expect.
//!
//! The model is JavaScript ([`PRELUDE`]); it reads and changes the
//! drawing through the `vg_*` methods of the script host below, which
//! work in page millimetres with y up from the page's lower left corner.
//! See docs/behavior/vba.md, "Object model".

use super::{HostInner, id_arg, num};
use serde_json::{Value, json};
use tracedraw_core::{
    Color, Command, Fill, LayerId, PageId, ShapeId, Stroke,
    document::{Shape, ShapeKind},
    geometry::Affine,
};

/// What the app shows that macros read but a drawing does not hold.
#[derive(Debug, Clone, Default)]
pub(super) struct AppView {
    /// The app's units, by name ("Millimeters").
    pub units: String,
    /// Effective axis defaults for absent persisted ruler preferences.
    pub ruler_units: [u32; 2],
    /// The centre of the drawing window, page millimetres.
    pub center: (f64, f64),
    /// The nodes selected with the Shape tool (object, node index).
    pub nodes: Vec<(u64, usize)>,
    /// The colour profiles in use: RGB, CMYK, grayscale.
    pub profiles: [String; 3],
}

impl AppView {
    pub fn of(app: &crate::app::App) -> AppView {
        let c = app.view.visible_page_rect(app.canvas_rect).center();
        let p = &app.settings.color;
        let doc_cmyk = app.doc().cmyk_profile.as_ref().map(|d| d.name.clone());
        AppView {
            units: format!("{:?}", app.units),
            ruler_units: [
                app.ruler_unit(true).script_unit(),
                app.ruler_unit(false).script_unit(),
            ],
            center: (c.x, c.y),
            nodes: app
                .node_selection
                .iter()
                .map(|(s, i)| (s.raw(), *i))
                .collect(),
            profiles: [
                p.rgb_profile.clone(),
                doc_cmyk.unwrap_or_else(|| p.cmyk_profile.clone()),
                p.gray_profile.clone(),
            ],
        }
    }
}

/// The JavaScript constructors validate before serialization. Keep the same
/// finite rectangle invariant at the typed boundary for direct host calls too.
fn validate_rectangle_kind(kind: &ShapeKind) -> Result<(), String> {
    if let ShapeKind::Rect {
        rect,
        radius,
        corners,
    } = kind
    {
        let coordinates = [
            rect.x0,
            rect.y0,
            rect.x1,
            rect.y1,
            rect.width(),
            rect.height(),
        ];
        if coordinates.iter().any(|value| !value.is_finite()) {
            return Err("rectangle coordinates and dimensions must be finite".into());
        }
        if !radius.is_finite() || *radius < 0.0 {
            return Err("rectangle radius must be finite and nonnegative".into());
        }
        if corners.as_ref().is_some_and(|corners| {
            corners
                .radii
                .iter()
                .any(|value| !value.is_finite() || *value < 0.0)
        }) {
            return Err("rectangle corner radii must be finite and nonnegative".into());
        }
    }
    Ok(())
}

fn ids(a: &Value, key: &str) -> Vec<ShapeId> {
    a.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_u64()).map(ShapeId).collect())
        .unwrap_or_default()
}

fn kind_tag(k: &ShapeKind) -> &'static str {
    match k {
        ShapeKind::Rect { .. } => "rect",
        ShapeKind::Ellipse { .. } => "ellipse",
        ShapeKind::Polygon { .. } => "polygon",
        ShapeKind::Path { .. } => "path",
        ShapeKind::Text { .. } => "text",
        ShapeKind::Group { .. } => "group",
        ShapeKind::ClipFrame { .. } => "clipframe",
        ShapeKind::Table(_) => "table",
        ShapeKind::SymbolInstance { .. } => "symbol",
        ShapeKind::Bitmap { .. } => "bitmap",
    }
}

impl HostInner {
    /// The top-level layer holding `id` (nested objects: their outermost
    /// group's layer).
    pub(super) fn layer_of(&self, id: ShapeId) -> Result<LayerId, String> {
        let top = self.doc.ancestors(id).first().copied().unwrap_or(id);
        self.doc
            .locate(top)
            .map(|(l, _)| l)
            .map_err(|e| e.to_string())
    }

    /// The page a layer belongs to; master layers count as the active
    /// page's.
    fn page_of_layer(&self, layer: LayerId) -> PageId {
        self.doc
            .pages
            .iter()
            .find(|p| p.layers.iter().any(|l| l.id == layer))
            .map(|p| p.id)
            .unwrap_or(self.page)
    }

    /// A copy of `s` with new ids for it and everything inside it, and no
    /// identity of its own yet (the engine gives one).
    pub(super) fn fresh_ids(&mut self, mut s: Shape) -> Shape {
        match &mut s.kind {
            ShapeKind::Group { children } => {
                let kids = std::mem::take(children);
                *children = kids.into_iter().map(|c| self.fresh_ids(c)).collect();
            }
            ShapeKind::ClipFrame { frame, contents } => {
                let f = std::mem::replace(
                    frame.as_mut(),
                    Shape::new(ShapeId(0), ShapeKind::Group { children: vec![] }),
                );
                **frame = self.fresh_ids(f);
                let kids = std::mem::take(contents);
                *contents = kids.into_iter().map(|c| self.fresh_ids(c)).collect();
            }
            _ => {}
        }
        s.id = self.new_id();
        s.uid = None;
        s
    }

    /// Group objects lying on layers into one group placed where the
    /// lowest of them was, without the drawing minting the group's id
    /// itself (so the id is the same when the batch replays).
    pub(super) fn group_shapes(&mut self, members: &[ShapeId]) -> Result<ShapeId, String> {
        let mut located: Vec<(ShapeId, LayerId, usize)> = members
            .iter()
            .map(|id| {
                self.doc
                    .locate(*id)
                    .map(|(l, i)| (*id, l, i))
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<_, _>>()?;
        if located.is_empty() {
            return Err("nothing to group".into());
        }
        let rank = |doc: &tracedraw_core::Document, l: LayerId| {
            doc.all_layers()
                .position(|x| x.id == l)
                .unwrap_or(usize::MAX)
        };
        located.sort_by_key(|(_, l, i)| (rank(&self.doc, *l), *i));
        let (layer, insert_at) = (located[0].1, located[0].2);
        let children: Vec<Shape> = located
            .iter()
            .filter_map(|(id, _, _)| self.doc.find_shape(*id).cloned())
            .collect();
        self.run(Command::DeleteShapes {
            shapes: located.iter().map(|(id, _, _)| *id).collect(),
        })?;
        let gid = self.new_id();
        let mut group = Shape::new(gid, ShapeKind::Group { children });
        group.stroke = None;
        group.fill = Fill::None;
        self.run(Command::AddShape {
            layer,
            shape: group,
        })?;
        let n = self.doc.layer(layer).map(|l| l.shapes.len()).unwrap_or(1);
        self.run(Command::Reorder {
            shape: gid,
            layer,
            index: insert_at.min(n - 1),
        })?;
        Ok(gid)
    }

    fn info(&self, id: ShapeId) -> Result<Value, String> {
        let s = self
            .doc
            .find_shape(id)
            .ok_or_else(|| format!("no shape {}", id.raw()))?;
        let on_page = self.doc.shape_on_page(id).unwrap_or_else(|| s.clone());
        let rotation = on_page.orientation_angle();
        let parent = self.doc.ancestors(id).last().map(|p| p.raw());
        let (children, frame, contents): (Vec<u64>, Option<u64>, Vec<u64>) = match &s.kind {
            ShapeKind::Group { children } => {
                (children.iter().map(|c| c.id.raw()).collect(), None, vec![])
            }
            ShapeKind::ClipFrame { frame, contents } => (
                vec![],
                Some(frame.id.raw()),
                contents.iter().map(|c| c.id.raw()).collect(),
            ),
            _ => (vec![], None, vec![]),
        };
        let layer = self.layer_of(id).ok();
        let page = layer.map(|l| self.page_of_layer(l)).unwrap_or(self.page);
        let effects: Vec<Value> = s
            .effects
            .iter()
            .filter_map(|e| {
                serde_json::to_value(e)
                    .ok()
                    .and_then(|v| v.get("effect").cloned())
            })
            .collect();
        Ok(json!({
            "id": id.raw(),
            "kind": kind_tag(&s.kind),
            "qr": s.qr.is_some(),
            "name": s.name.clone().unwrap_or_default(),
            "locked": s.locked,
            "visible": s.visible,
            "opacity": s.opacity,
            "overprint_fill": s.overprint_fill,
            "overprint_outline": s.overprint_outline,
            "layer": layer.map(|l| l.raw()),
            "page": page.raw(),
            "parent": parent,
            "children": children,
            "frame": frame,
            "contents": contents,
            "data": s.data,
            "fill": s.fill,
            "stroke": s.stroke,
            "rotation": rotation,
            "transform": s.transform.as_coeffs(),
            "page_transform": on_page.transform.as_coeffs(),
            "effects": effects,
            "uid": s.uid,
            "transparency_mask": s.effects.iter().find_map(|e| match e {
                tracedraw_core::live::Effect::Transparency { mask, .. } => Some(match mask {
                    Fill::Fountain(_) => "fountain",
                    Fill::Pattern(_) => "pattern",
                    Fill::Texture(_) => "texture",
                    _ => "uniform",
                }),
                _ => None,
            }),
        }))
    }

    fn bounds_of(&self, list: &[ShapeId]) -> Option<tracedraw_core::geometry::Rect> {
        list.iter()
            .filter_map(|id| self.doc.shape_on_page(*id))
            .map(|s| s.bounds())
            .reduce(|a, b| a.union(b))
    }

    /// The `vg_*` methods; `None` for a name that is not one of them.
    pub(super) fn vg_call(&mut self, method: &str, a: &Value) -> Result<Option<Value>, String> {
        Ok(Some(match method {
            "vg_doc" => json!({
                "title": self.doc.title,
                "file": self.file,
                "page": self.page.raw(),
                "layer": self.layer.map(|l| l.raw()),
                "pages": self.doc.pages.iter().map(|p| p.id.raw()).collect::<Vec<_>>(),
                "selection": self.selection.iter().map(|s| s.raw()).collect::<Vec<_>>(),
                "changed": !self.commands.is_empty(),
                "dpi": self.doc.metadata.resolution_dpi,
                "units": self.view.units,
                "script_unit": self.doc.metadata.script_unit,
                "ruler_units": [self.doc.metadata.rulers.horizontal_unit, self.doc.metadata.rulers.vertical_unit],
                "ruler_default_units": self.view.ruler_units,
                "view": [self.view.center.0, self.view.center.1],
                "profiles": self.view.profiles,
                "master": self.doc.master.iter().map(|l| l.id.raw()).collect::<Vec<_>>(),
            }),
            "vg_set_unit" | "vg_restore_unit" => {
                let unit = match a.get("unit") {
                    Some(Value::Null) if method == "vg_restore_unit" => None,
                    Some(value) => Some(
                        value
                            .as_u64()
                            .filter(|unit| *unit <= 16)
                            .ok_or("script unit must be an integer between 0 and 16")?
                            as u32,
                    ),
                    None => return Err("missing script unit".into()),
                };
                if self.doc.metadata.script_unit != unit {
                    let mut metadata = self.doc.metadata.clone();
                    metadata.script_unit = unit;
                    self.run(Command::SetMetadata { metadata })?;
                }
                json!({"script_unit": unit})
            }
            "vg_set_ruler_unit" => {
                let horizontal = a
                    .get("horizontal")
                    .and_then(Value::as_bool)
                    .ok_or("ruler axis must be a boolean")?;
                let unit = a
                    .get("unit")
                    .and_then(Value::as_u64)
                    .filter(|unit| *unit <= 16)
                    .ok_or("ruler unit must be an integer between 0 and 16")?
                    as u32;
                let mut metadata = self.doc.metadata.clone();
                let axis = if horizontal {
                    &mut metadata.rulers.horizontal_unit
                } else {
                    &mut metadata.rulers.vertical_unit
                };
                if *axis != Some(unit) {
                    *axis = Some(unit);
                    self.run(Command::SetMetadata { metadata })?;
                }
                json!({"unit": unit})
            }
            "vg_info" => self.info(ShapeId(id_arg(a, "id")?))?,
            "vg_default_stroke" => {
                serde_json::to_value(Stroke::default()).map_err(|e| e.to_string())?
            }
            "vg_exists" => json!({"yes": self.doc.find_shape(ShapeId(id_arg(a, "id")?)).is_some()}),
            "vg_bounds" => {
                let list = ids(a, "ids");
                match self.bounds_of(&list) {
                    Some(b) => json!({"x0": b.x0, "y0": b.y0, "x1": b.x1, "y1": b.y1}),
                    None => json!({"x0": 0.0, "y0": 0.0, "x1": 0.0, "y1": 0.0}),
                }
            }
            "vg_transform" => {
                let list = ids(a, "ids");
                let m: Vec<f64> = a
                    .get("m")
                    .and_then(|m| m.as_array())
                    .map(|m| m.iter().filter_map(|v| v.as_f64()).collect())
                    .unwrap_or_default();
                if m.len() != 6 || m.iter().any(|v| !v.is_finite()) {
                    return Err("bad transform".into());
                }
                if !list.is_empty() {
                    self.run(Command::TransformShapes {
                        shapes: list,
                        transform: Affine::new([m[0], m[1], m[2], m[3], m[4], m[5]]),
                    })?;
                }
                json!({})
            }
            "vg_kind" => {
                let s = self.shape(id_arg(a, "id")?)?;
                serde_json::to_value(&s.kind).map_err(|e| e.to_string())?
            }
            "vg_set_kind" => {
                let shape = ShapeId(id_arg(a, "id")?);
                let kind: ShapeKind =
                    serde_json::from_value(a.get("kind").cloned().unwrap_or(Value::Null))
                        .map_err(|e| format!("not a shape: {e}"))?;
                validate_rectangle_kind(&kind)?;
                self.run(Command::SetShapeKind { shape, kind })?;
                json!({})
            }
            "vg_set_fill" => {
                let fill: Fill =
                    serde_json::from_value(a.get("fill").cloned().unwrap_or(Value::Null))
                        .map_err(|e| format!("not a fill: {e}"))?;
                let shapes = ids(a, "ids");
                if !shapes.is_empty() {
                    self.run(Command::SetFill { shapes, fill })?;
                }
                json!({})
            }
            "vg_set_stroke" => {
                let stroke: Option<Stroke> = match a.get("stroke") {
                    None | Some(Value::Null) => None,
                    Some(v) => Some(
                        serde_json::from_value(v.clone())
                            .map_err(|e| format!("not an outline: {e}"))?,
                    ),
                };
                let shapes = ids(a, "ids");
                if !shapes.is_empty() {
                    self.run(Command::SetStroke { shapes, stroke })?;
                }
                json!({})
            }
            "vg_props" => {
                let shapes = ids(a, "ids");
                if shapes.is_empty() {
                    return Ok(Some(json!({})));
                }
                let flag = |k: &str| a.get(k).and_then(|v| v.as_bool());
                if let Some(locked) = flag("locked") {
                    self.run(Command::SetLocked {
                        shapes: shapes.clone(),
                        locked,
                    })?;
                }
                if let Some(visible) = flag("visible") {
                    self.run(Command::SetVisible {
                        shapes: shapes.clone(),
                        visible,
                    })?;
                }
                if flag("overprint_fill").is_some() || flag("overprint_outline").is_some() {
                    self.run(Command::SetOverprint {
                        shapes: shapes.clone(),
                        fill: flag("overprint_fill"),
                        outline: flag("overprint_outline"),
                    })?;
                }
                if let Some(o) = a.get("opacity").and_then(|v| v.as_f64()) {
                    self.run(Command::SetOpacity {
                        shapes: shapes.clone(),
                        opacity: o.clamp(0.0, 1.0),
                    })?;
                }
                json!({})
            }
            "vg_set_data" => {
                let shape = ShapeId(id_arg(a, "id")?);
                let data: Vec<(String, String)> =
                    serde_json::from_value(a.get("data").cloned().unwrap_or(Value::Null))
                        .map_err(|e| e.to_string())?;
                self.run(Command::SetObjectData { shape, data })?;
                json!({})
            }
            "vg_order" => {
                let list = ids(a, "ids");
                let op = a.get("op").and_then(|v| v.as_str()).unwrap_or("front");
                for id in list {
                    let (layer, i) = self.doc.locate(id).map_err(|e| e.to_string())?;
                    let n = self.doc.layer(layer).map(|l| l.shapes.len()).unwrap_or(1);
                    match op {
                        "front_of" | "back_of" => {
                            let reference = ShapeId(id_arg(a, "ref")?);
                            self.run(Command::OrderRelative {
                                shapes: vec![id],
                                reference,
                                in_front: op == "front_of",
                            })?;
                        }
                        _ => {
                            let index = match op {
                                "front" => n - 1,
                                "back" => 0,
                                "forward" => (i + 1).min(n - 1),
                                _ => i.saturating_sub(1),
                            };
                            self.run(Command::Reorder {
                                shape: id,
                                layer,
                                index,
                            })?;
                        }
                    }
                }
                json!({})
            }
            "vg_group" => {
                let list = ids(a, "ids");
                if list.len() < 2 {
                    return Err("grouping needs two objects or more".into());
                }
                json!({"id": self.group_shapes(&list)?.raw()})
            }
            "vg_ungroup" => {
                let all = a.get("all").and_then(|v| v.as_bool()).unwrap_or(false);
                let mut out = Vec::new();
                let mut todo = vec![ShapeId(id_arg(a, "id")?)];
                while let Some(g) = todo.pop() {
                    let s = self.shape(g.raw())?;
                    let ShapeKind::Group { children } = &s.kind else {
                        out.push(g.raw());
                        continue;
                    };
                    let kids: Vec<ShapeId> = children.iter().map(|c| c.id).collect();
                    self.run(Command::Ungroup { group: g })?;
                    for k in kids {
                        let nested = matches!(
                            self.doc.find_shape(k).map(|s| &s.kind),
                            Some(ShapeKind::Group { .. })
                        );
                        if all && nested {
                            todo.push(k);
                        } else {
                            out.push(k.raw());
                        }
                    }
                }
                json!({"ids": out})
            }
            "vg_combine" => {
                let list = ids(a, "ids");
                if list.is_empty() {
                    return Err("nothing to combine".into());
                }
                let keep = list[0];
                self.run(Command::Combine {
                    shapes: list.clone(),
                })?;
                let id = if self.doc.find_shape(keep).is_some() {
                    keep
                } else {
                    list.iter()
                        .copied()
                        .find(|id| self.doc.find_shape(*id).is_some())
                        .ok_or("combine left nothing")?
                };
                json!({"id": id.raw()})
            }
            "vg_break_apart" => {
                let id = ShapeId(id_arg(a, "id")?);
                let s = self.shape(id.raw())?;
                let path = match &s.kind {
                    ShapeKind::Path { path, .. } => path.clone(),
                    _ => s.local_path(),
                };
                // One curve per subpath: the first keeps the object.
                let mut parts: Vec<tracedraw_core::BezPath> = Vec::new();
                for el in path.elements() {
                    if matches!(el, tracedraw_core::geometry::PathEl::MoveTo(_)) || parts.is_empty()
                    {
                        parts.push(tracedraw_core::BezPath::new());
                    }
                    if let Some(p) = parts.last_mut() {
                        p.push(*el);
                    }
                }
                let closed = |p: &tracedraw_core::BezPath| {
                    p.elements()
                        .iter()
                        .any(|e| matches!(e, tracedraw_core::geometry::PathEl::ClosePath))
                };
                let mut out = vec![id.raw()];
                if parts.len() > 1 {
                    let layer = self.layer_of(id)?;
                    let first = parts.remove(0);
                    self.run(Command::SetShapeKind {
                        shape: id,
                        kind: ShapeKind::Path {
                            closed: closed(&first),
                            path: first,
                        },
                    })?;
                    for p in parts {
                        let mut piece = s.clone();
                        piece.kind = ShapeKind::Path {
                            closed: closed(&p),
                            path: p,
                        };
                        let piece = self.fresh_ids(piece);
                        out.push(piece.id.raw());
                        self.run(Command::AddShape {
                            layer,
                            shape: piece,
                        })?;
                    }
                }
                json!({"ids": out})
            }
            "vg_boolean" => {
                use tracedraw_core::shaping::{Op, overlay};
                let target = ShapeId(id_arg(a, "target")?);
                let others = ids(a, "others");
                let op = a.get("op").and_then(|v| v.as_str()).unwrap_or("weld");
                let t = self.doc.shape_on_page(target).ok_or("no target")?;
                let mut result = t.page_path();
                for o in &others {
                    let p = self.doc.shape_on_page(*o).ok_or("no shape")?.page_path();
                    result = overlay(
                        &result,
                        &p,
                        match op {
                            "trim" => Op::Trim,
                            "intersect" => Op::Intersect,
                            _ => Op::Weld,
                        },
                    );
                }
                if result.elements().is_empty() {
                    return Ok(Some(json!({"id": Value::Null})));
                }
                let layer = self.layer_of(target)?;
                let id = self.new_id();
                let mut shape = Shape::new(
                    id,
                    ShapeKind::Path {
                        path: result,
                        closed: true,
                    },
                );
                shape.fill = t.fill.clone();
                shape.stroke = t.stroke.clone();
                shape.opacity = t.opacity;
                self.run(Command::AddShape { layer, shape })?;
                let leave_target = a
                    .get("leave_target")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let leave_source = a
                    .get("leave_source")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let mut gone: Vec<ShapeId> = Vec::new();
                if !leave_target {
                    gone.push(target);
                }
                if !leave_source {
                    gone.extend(others.iter().copied());
                }
                if !gone.is_empty() {
                    self.run(Command::DeleteShapes {
                        shapes: gone.clone(),
                    })?;
                    self.selection.retain(|s| !gone.contains(s));
                }
                json!({"id": id.raw()})
            }
            "vg_duplicate" => {
                let (dx, dy) = (num(a, "dx", 0.0), num(a, "dy", 0.0));
                let mut out = Vec::new();
                for id in ids(a, "ids") {
                    let layer = self.layer_of(id)?;
                    let Some(s) = self.doc.shape_on_page(id) else {
                        continue;
                    };
                    let mut copy = self.fresh_ids(s);
                    copy.transform_by(Affine::translate((dx, dy)));
                    out.push(copy.id.raw());
                    self.run(Command::AddShape { layer, shape: copy })?;
                }
                json!({"ids": out})
            }
            "vg_delete" => {
                let list: Vec<ShapeId> = ids(a, "ids")
                    .into_iter()
                    .filter(|id| self.doc.find_shape(*id).is_some())
                    .collect();
                if !list.is_empty() {
                    self.run(Command::DeleteShapes {
                        shapes: list.clone(),
                    })?;
                    self.selection.retain(|s| !list.contains(s));
                }
                json!({})
            }
            "vg_add_shape" => {
                let layer = self.layer_for(a)?;
                let kind: ShapeKind =
                    serde_json::from_value(a.get("kind").cloned().unwrap_or(Value::Null))
                        .map_err(|e| format!("not a shape: {e}"))?;
                validate_rectangle_kind(&kind)?;
                let mut s = Shape::new(ShapeId(0), kind);
                if let Some(f) = a.get("fill").filter(|v| !v.is_null()) {
                    s.fill = serde_json::from_value(f.clone()).map_err(|e| e.to_string())?;
                }
                match a.get("stroke") {
                    Some(Value::Null) => s.stroke = None,
                    Some(v) => {
                        s.stroke =
                            Some(serde_json::from_value(v.clone()).map_err(|e| e.to_string())?)
                    }
                    None => {}
                }
                self.add(layer, s)?
            }
            "vg_page" if id_arg(a, "page")? == 0 => {
                // The master page: the layers drawn on every page.
                let size = self.doc.page(self.page).map(|p| p.size).unwrap_or_default();
                let master: Vec<u64> = self.doc.master.iter().map(|l| l.id.raw()).collect();
                json!({"name": "Master Page", "w": size.width, "h": size.height, "index": 0,
                       "layers": master, "master": master})
            }
            "vg_page" => {
                let pid = PageId(id_arg(a, "page")?);
                let (i, p) = self
                    .doc
                    .pages
                    .iter()
                    .enumerate()
                    .find(|(_, p)| p.id == pid)
                    .ok_or("no page")?;
                json!({
                    "name": p.name, "w": p.size.width, "h": p.size.height, "index": i + 1,
                    "layers": p.layers.iter().map(|l| l.id.raw()).collect::<Vec<_>>(),
                    "master": self.doc.master.iter().map(|l| l.id.raw()).collect::<Vec<_>>(),
                })
            }
            "vg_rename_page" => {
                let page = PageId(id_arg(a, "page")?);
                let name = a
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();
                self.run(Command::RenamePage { page, name })?;
                json!({})
            }
            "vg_delete_page" => {
                let page = PageId(id_arg(a, "page")?);
                if self.doc.pages.len() < 2 {
                    return Err("a drawing keeps one page at least".into());
                }
                self.run(Command::DeletePage { page })?;
                if self.page == page {
                    self.page = self.doc.pages[0].id;
                    self.layer = self.doc.pages[0].layers.last().map(|l| l.id);
                }
                json!({})
            }
            "vg_insert_pages" => {
                let count = num(a, "count", 1.0).max(1.0) as usize;
                // 0-based place for the first new page; at the end by default.
                let at = a
                    .get("at")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(self.doc.pages.len());
                let size = self
                    .doc
                    .page(self.page)
                    .map(|p| p.size)
                    .unwrap_or(tracedraw_core::document::paper::A4);
                let mut out = Vec::new();
                for k in 0..count {
                    self.run(Command::AddPage {
                        name: None,
                        size,
                        layer: None,
                    })?;
                    let page = self.doc.pages.last().map(|p| p.id).ok_or("no page")?;
                    let to = (at + k).min(self.doc.pages.len() - 1);
                    if to != self.doc.pages.len() - 1 {
                        self.run(Command::MovePage { page, to })?;
                    }
                    out.push(page.raw());
                }
                json!({"ids": out})
            }
            "vg_layer" => {
                let id = LayerId(id_arg(a, "layer")?);
                let l = self.doc.layer(id).map_err(|e| e.to_string())?;
                let master = self.doc.master.iter().any(|m| m.id == id);
                json!({"name": l.name, "visible": l.visible, "printable": l.printable,
                       "locked": l.locked, "master": master,
                       "page": self.page_of_layer(id).raw()})
            }
            "vg_layer_set" => {
                let layer = LayerId(id_arg(a, "layer")?);
                if let Some(printable) = a.get("printable").and_then(|v| v.as_bool()) {
                    self.run(Command::SetLayerPrintable { layer, printable })?;
                }
                if let Some(locked) = a.get("locked").and_then(|v| v.as_bool()) {
                    self.run(Command::SetLayerLocked { layer, locked })?;
                }
                json!({})
            }
            "vg_delete_layer" => {
                let layer = LayerId(id_arg(a, "layer")?);
                self.run(Command::DeleteLayer { layer })?;
                if self.layer == Some(layer) {
                    self.layer = self
                        .doc
                        .page(self.page)
                        .ok()
                        .and_then(|p| p.layers.last().map(|l| l.id));
                }
                json!({})
            }
            "vg_length" => {
                use kurbo::ParamCurveArclen;
                let s = self
                    .doc
                    .shape_on_page(ShapeId(id_arg(a, "id")?))
                    .ok_or("no shape")?;
                let path = s.page_path();
                let len: f64 = path.segments().map(|seg| seg.arclen(1e-4)).sum();
                json!({"length": len})
            }
            "vg_color" => {
                let c: Color = a
                    .get("color")
                    .cloned()
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or(Color::BLACK);
                let model = a.get("model").and_then(|m| m.as_str()).unwrap_or("RGB");
                let out = c.convert_to(model);
                let [r, g, b] = c.to_rgb8();
                json!({"color": out, "rgb": [r, g, b], "hex": c.to_hex()})
            }
            "vg_save" => {
                if self.trust == super::Trust::Untrusted {
                    return Err("saving is not available here".into());
                }
                let path = a
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or("missing path")?;
                let p = std::path::Path::new(path);
                let ext = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let data = match ext.as_str() {
                    "cdr" => tracedraw_cdr::write::document_to_cdr(&self.doc),
                    _ => tracedraw_text::fonts()
                        .with_carried_fonts(&self.doc)
                        .to_json()
                        .map_err(|e| e.to_string())?
                        .into_bytes(),
                };
                crate::files::write(p, data).map_err(|e| e.to_string())?;
                self.file = Some(path.to_string());
                json!({})
            }
            "vg_export" => {
                if self.trust == super::Trust::Untrusted {
                    return Err("export is not available here: no file access".into());
                }
                let path = a
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or("missing path")?;
                let p = std::path::Path::new(path);
                let ext = a
                    .get("format")
                    .and_then(|f| f.as_str())
                    .filter(|f| !f.is_empty())
                    .map(|f| f.to_ascii_lowercase())
                    .or_else(|| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.to_ascii_lowercase())
                    })
                    .unwrap_or_default();
                let page = a
                    .get("page")
                    .and_then(|v| v.as_u64())
                    .map(PageId)
                    .unwrap_or(self.page);
                let idx = self
                    .doc
                    .pages
                    .iter()
                    .position(|pg| pg.id == page)
                    .unwrap_or(0);
                let dpi = num(a, "dpi", 0.0);
                let size_px = (num(a, "w", 0.0), num(a, "h", 0.0));
                let raster = |doc: &tracedraw_core::Document| -> Result<image::RgbaImage, String> {
                    let pg = doc.page(page).map_err(|e| e.to_string())?;
                    // A width in pixels wins over the resolution.
                    let dpi = if size_px.0 > 0.0 {
                        size_px.0 * 25.4 / pg.size.width.max(1e-9)
                    } else if size_px.1 > 0.0 {
                        size_px.1 * 25.4 / pg.size.height.max(1e-9)
                    } else if dpi > 0.0 {
                        dpi
                    } else {
                        96.0
                    };
                    let pm = tracedraw_render::render_page_image(doc, page, dpi)
                        .ok_or("render failed")?;
                    let img =
                        image::RgbaImage::from_raw(pm.width(), pm.height(), pm.data().to_vec())
                            .ok_or("render failed")?;
                    Ok(if size_px.0 > 0.0 && size_px.1 > 0.0 {
                        image::imageops::resize(
                            &img,
                            size_px.0 as u32,
                            size_px.1 as u32,
                            image::imageops::FilterType::Lanczos3,
                        )
                    } else {
                        img
                    })
                };
                let encode =
                    |img: &image::RgbaImage, fmt: image::ImageFormat| -> Result<Vec<u8>, String> {
                        let mut out = std::io::Cursor::new(Vec::new());
                        if fmt == image::ImageFormat::Jpeg {
                            image::DynamicImage::ImageRgba8(img.clone())
                                .to_rgb8()
                                .write_to(&mut out, fmt)
                                .map_err(|e| e.to_string())?;
                        } else {
                            img.write_to(&mut out, fmt).map_err(|e| e.to_string())?;
                        }
                        Ok(out.into_inner())
                    };
                let data = match ext.as_str() {
                    "svg" => tracedraw_io::svg::page_to_svg(&self.doc, idx).into_bytes(),
                    "pdf" => tracedraw_io::pdf::document_to_pdf(&self.doc),
                    "eps" | "ps" => tracedraw_io::eps::document_to_ps(&self.doc).into_bytes(),
                    "cdr" => tracedraw_cdr::write::document_to_cdr(&self.doc),
                    "tdraw" => tracedraw_text::fonts()
                        .with_carried_fonts(&self.doc)
                        .to_json()
                        .map_err(|e| e.to_string())?
                        .into_bytes(),
                    "png" => encode(&raster(&self.doc)?, image::ImageFormat::Png)?,
                    "jpg" | "jpeg" => encode(&raster(&self.doc)?, image::ImageFormat::Jpeg)?,
                    "bmp" => encode(&raster(&self.doc)?, image::ImageFormat::Bmp)?,
                    "tif" | "tiff" => encode(&raster(&self.doc)?, image::ImageFormat::Tiff)?,
                    "gif" => encode(&raster(&self.doc)?, image::ImageFormat::Gif)?,
                    "webp" => encode(&raster(&self.doc)?, image::ImageFormat::WebP)?,
                    other => return Err(format!("this format cannot be exported: {other}")),
                };
                crate::files::write(p, data).map_err(|e| e.to_string())?;
                json!({})
            }
            "vg_move_to_layer" => {
                let shape = ShapeId(id_arg(a, "id")?);
                let layer = LayerId(id_arg(a, "layer")?);
                let index = self
                    .doc
                    .layer(layer)
                    .map(|l| l.shapes.len())
                    .map_err(|e| e.to_string())?;
                self.run(Command::Reorder {
                    shape,
                    layer,
                    index,
                })?;
                json!({})
            }
            "vg_zorder" => {
                let id = ShapeId(id_arg(a, "id")?);
                let top = self.doc.ancestors(id).first().copied().unwrap_or(id);
                let (layer, i) = self.doc.locate(top).map_err(|e| e.to_string())?;
                let rank = self
                    .doc
                    .all_layers()
                    .position(|l| l.id == layer)
                    .unwrap_or(0);
                json!({"rank": rank as f64 * 1e6 + i as f64})
            }
            "vg_place_inside" => {
                let contents = ids(a, "ids");
                let frame = ShapeId(id_arg(a, "frame")?);
                self.run(Command::PlaceInside { contents, frame })?;
                json!({})
            }
            "vg_clear_shadow" => {
                self.run(Command::SetShadow {
                    shapes: vec![ShapeId(id_arg(a, "id")?)],
                    shadow: None,
                })?;
                json!({})
            }
            "vg_status" => {
                self.status = a.get("text").and_then(|t| t.as_str()).map(str::to_string);
                json!({})
            }
            "vg_node_selection" => {
                let id = id_arg(a, "id")?;
                let list: Vec<usize> = self
                    .view
                    .nodes
                    .iter()
                    .filter(|(s, _)| *s == id)
                    .map(|(_, i)| *i)
                    .collect();
                json!({"indices": list})
            }
            "vg_guides" => {
                use tracedraw_core::document::GuideLine;
                let page = PageId(id_arg(a, "page")?);
                let p = self.doc.page(page).map_err(|e| e.to_string())?;
                let list: Vec<Value> = p
                    .guides
                    .iter()
                    .map(|g| match g.line {
                        GuideLine::Horizontal { y } => json!({"x": 0.0, "y": y, "angle": 0.0}),
                        GuideLine::Vertical { x } => json!({"x": x, "y": 0.0, "angle": 90.0}),
                        GuideLine::Angled { x, y, angle } => {
                            json!({"x": x, "y": y, "angle": angle})
                        }
                    })
                    .collect();
                json!({"guides": list})
            }
            "vg_add_guide" | "vg_set_guide" => {
                use tracedraw_core::document::GuideLine;
                let page = PageId(id_arg(a, "page")?);
                let (x, y) = (num(a, "x", 0.0), num(a, "y", 0.0));
                let angle = num(a, "angle", 0.0).rem_euclid(180.0);
                let line = if angle.abs() < 1e-9 {
                    GuideLine::Horizontal { y }
                } else if (angle - 90.0).abs() < 1e-9 {
                    GuideLine::Vertical { x }
                } else {
                    GuideLine::Angled { x, y, angle }
                };
                if method == "vg_add_guide" {
                    self.run(Command::AddGuide {
                        page,
                        guide: line.into(),
                    })?;
                    let n = self.doc.page(page).map(|p| p.guides.len()).unwrap_or(1);
                    json!({"index": n - 1})
                } else {
                    let index = id_arg(a, "index")? as usize;
                    let mut guide = self
                        .doc
                        .page(page)
                        .ok()
                        .and_then(|p| p.guides.get(index).cloned())
                        .ok_or("no such guideline")?;
                    guide.line = line;
                    self.run(Command::MoveGuide { page, index, guide })?;
                    json!({})
                }
            }
            "vg_unlock_all" => {
                let page = id_arg(a, "page")?;
                let layers: Vec<&tracedraw_core::document::Layer> = if page == 0 {
                    self.doc.master.iter().collect()
                } else {
                    self.doc
                        .page(PageId(page))
                        .map(|p| p.layers.iter().collect())
                        .unwrap_or_default()
                };
                let shapes: Vec<ShapeId> = layers
                    .iter()
                    .flat_map(|l| l.shapes.iter().filter(|s| s.locked).map(|s| s.id))
                    .collect();
                if !shapes.is_empty() {
                    self.run(Command::SetLocked {
                        shapes,
                        locked: false,
                    })?;
                }
                json!({})
            }
            "vg_area" => {
                use kurbo::Shape as _;
                let s = self
                    .doc
                    .shape_on_page(ShapeId(id_arg(a, "id")?))
                    .ok_or("no shape")?;
                let path = s.page_path();
                // Each closed subpath's area; holes (opposite winding) count
                // against the total, as the even-odd fill draws them.
                let mut parts: Vec<tracedraw_core::BezPath> = Vec::new();
                for el in path.elements() {
                    if matches!(el, tracedraw_core::geometry::PathEl::MoveTo(_)) || parts.is_empty()
                    {
                        parts.push(tracedraw_core::BezPath::new());
                    }
                    if let Some(p) = parts.last_mut() {
                        p.push(*el);
                    }
                }
                let areas: Vec<f64> = parts.iter().map(|p| p.area().abs()).collect();
                let total = path.area().abs();
                json!({"total": total, "subpaths": areas})
            }
            "vg_bitmap" => {
                let s = self.shape(id_arg(a, "id")?)?;
                let ShapeKind::Bitmap {
                    width_px,
                    height_px,
                    png,
                    ..
                } = &s.kind
                else {
                    return Err("not a bitmap".into());
                };
                let on_page = self.doc.shape_on_page(s.id).unwrap_or_else(|| s.clone());
                let b = on_page.bounds();
                // The PNG header's colour type: 0 gray, 2 RGB, 3 palette,
                // 4 gray with alpha, 6 RGB with alpha.
                let ctype = png.get(25).copied().unwrap_or(6);
                let img = crate::bitmap_fx::decode(png);
                let transparent = img
                    .as_ref()
                    .map(|i| i.pixels().any(|p| p.0[3] < 255))
                    .unwrap_or(false);
                let gray_only = img
                    .as_ref()
                    .map(|i| i.pixels().all(|p| p.0[0] == p.0[1] && p.0[1] == p.0[2]))
                    .unwrap_or(false);
                let bw = gray_only
                    && img
                        .as_ref()
                        .map(|i| i.pixels().all(|p| p.0[0] == 0 || p.0[0] == 255))
                        .unwrap_or(false);
                let mode = match ctype {
                    3 => "paletted",
                    0 | 4 if bw => "bw",
                    0 | 4 => "gray",
                    _ if bw => "bw",
                    _ if gray_only => "gray",
                    _ => "rgb",
                };
                json!({"w": width_px, "h": height_px, "width_mm": b.width(), "height_mm": b.height(),
                       "mode": mode, "transparent": transparent})
            }
            "vg_bitmap_resample" | "vg_bitmap_mode" => {
                let id = ShapeId(id_arg(a, "id")?);
                let s = self.shape(id.raw())?;
                let ShapeKind::Bitmap { rect, png, .. } = &s.kind else {
                    return Err("not a bitmap".into());
                };
                let img = crate::bitmap_fx::decode(png).ok_or("the bitmap cannot be read")?;
                let out = if method == "vg_bitmap_resample" {
                    let w = num(a, "w", img.width() as f64).round().max(1.0) as u32;
                    let h = num(a, "h", img.height() as f64).round().max(1.0) as u32;
                    let how = if a
                        .get("anti_alias")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
                    {
                        crate::resample::Method::Smooth
                    } else {
                        crate::resample::Method::Pixels
                    };
                    crate::resample::resample(&img, w, h, how)
                } else {
                    use crate::bitmap_modes::{ConvertMode, ConvertOptions};
                    let mode = match a.get("mode").and_then(|m| m.as_str()).unwrap_or("rgb") {
                        "bw" => ConvertMode::BlackWhite,
                        "gray" => ConvertMode::Grayscale,
                        "paletted" => ConvertMode::Paletted,
                        "cmyk" => ConvertMode::Cmyk,
                        _ => ConvertMode::Rgb,
                    };
                    crate::bitmap_modes::convert_pixels(
                        &img,
                        &ConvertOptions {
                            mode,
                            dithered: a.get("dithered").and_then(|v| v.as_bool()).unwrap_or(false),
                            ..Default::default()
                        },
                    )
                };
                let encoded =
                    crate::bitmap_fx::encode(&out).ok_or("the bitmap cannot be written")?;
                self.run(Command::SetShapeKind {
                    shape: id,
                    kind: ShapeKind::Bitmap {
                        rect: *rect,
                        width_px: out.width(),
                        height_px: out.height(),
                        png: encoded.into(),
                        fx: None,
                    },
                })?;
                json!({})
            }
            "vg_to_bitmap" => {
                let list = ids(a, "ids");
                let shapes: Vec<Shape> = list
                    .iter()
                    .filter_map(|id| self.doc.shape_on_page(*id))
                    .collect();
                let Some(bounds) = shapes.iter().map(|s| s.bounds()).reduce(|x, y| x.union(y))
                else {
                    return Err("nothing to convert".into());
                };
                let dpi = num(a, "dpi", 300.0).clamp(36.0, 2400.0);
                let transparent = a
                    .get("transparent")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let anti_alias = a
                    .get("anti_alias")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let mut tmp = tracedraw_core::Document::new(
                    "tmp",
                    tracedraw_core::Size::new(bounds.width().max(0.1), bounds.height().max(0.1)),
                );
                tmp.symbols = self.doc.symbols.clone();
                let l = tmp.pages[0].layers[0].id;
                for s in &shapes {
                    let mut c = s.clone();
                    c.transform_by(Affine::translate((-bounds.x0, -bounds.y0)));
                    if let Ok(layer) = tmp.layer_mut(l) {
                        layer.shapes.push(c);
                    }
                }
                let page = tmp.pages[0].id;
                let zoom = dpi / 25.4;
                let w = (bounds.width() * zoom).ceil().max(1.0) as u32;
                let h = (bounds.height() * zoom).ceil().max(1.0) as u32;
                let pm = tracedraw_render::render_page(
                    &tmp,
                    page,
                    &tracedraw_render::RenderOptions {
                        width: w,
                        height: h,
                        view: tracedraw_render::ViewTransform {
                            zoom,
                            origin_x: 0.0,
                            origin_y: h as f64,
                        },
                        anti_alias,
                        ..tracedraw_render::RenderOptions::default()
                    },
                )
                .ok_or("render failed")?;
                let mut img =
                    image::RgbaImage::from_raw(w, h, pm.data().to_vec()).ok_or("render failed")?;
                // Premultiplied pixels back to plain ones, on white unless
                // the bitmap keeps its transparency.
                for p in img.pixels_mut() {
                    let al = p.0[3] as f32 / 255.0;
                    for k in 0..3 {
                        let c = if al > 0.0 { p.0[k] as f32 / al } else { 0.0 };
                        p.0[k] = if transparent {
                            c.round().min(255.0) as u8
                        } else {
                            (c * al + 255.0 * (1.0 - al)).round().min(255.0) as u8
                        };
                    }
                    if !transparent {
                        p.0[3] = 255;
                    }
                }
                {
                    use crate::bitmap_modes::{ConvertMode, ConvertOptions};
                    let mode = match a.get("mode").and_then(|m| m.as_str()).unwrap_or("rgb") {
                        "bw" => Some(ConvertMode::BlackWhite),
                        "gray" => Some(ConvertMode::Grayscale),
                        "paletted" => Some(ConvertMode::Paletted),
                        _ => None,
                    };
                    if let Some(mode) = mode {
                        img = crate::bitmap_modes::convert_pixels(
                            &img,
                            &ConvertOptions {
                                mode,
                                dithered: a
                                    .get("dithered")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false),
                                ..Default::default()
                            },
                        );
                    }
                }
                let png = crate::bitmap_fx::encode(&img).ok_or("encode failed")?;
                let layer = self.layer_of(list[0])?;
                let id = self.new_id();
                let mut bitmap = Shape::new(
                    id,
                    ShapeKind::Bitmap {
                        rect: bounds,
                        width_px: w,
                        height_px: h,
                        png: png.into(),
                        fx: None,
                    },
                );
                bitmap.fill = Fill::None;
                bitmap.stroke = None;
                bitmap.overprint_fill = a
                    .get("overprint")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                self.run(Command::DeleteShapes {
                    shapes: list.clone(),
                })?;
                self.selection.retain(|s| !list.contains(s));
                self.run(Command::AddShape {
                    layer,
                    shape: bitmap,
                })?;
                json!({"id": id.raw()})
            }
            "vg_overflow" => {
                let s = self
                    .doc
                    .shape_on_page(ShapeId(id_arg(a, "id")?))
                    .ok_or("no shape")?;
                let over = crate::text_editing::layout_of_text_shape(&s, Affine::IDENTITY)
                    .map(|(l, _)| l.overflow)
                    .unwrap_or(false);
                json!({"overflow": over})
            }
            "vg_effects" => {
                let s = self.shape(id_arg(a, "id")?)?;
                let mut list: Vec<Value> = s
                    .effects
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let kind = serde_json::to_value(e)
                            .ok()
                            .and_then(|v| v.get("effect").cloned())
                            .unwrap_or(Value::Null);
                        json!({"kind": kind, "index": i})
                    })
                    .collect();
                if let Some(sh) = &s.shadow {
                    list.push(
                        json!({"kind": "shadow", "color": sh.color, "opacity": sh.opacity,
                                     "blur": sh.blur, "dx": sh.offset.x, "dy": sh.offset.y}),
                    );
                }
                if let ShapeKind::Text {
                    on_path: Some(_), ..
                } = &s.kind
                {
                    list.push(json!({"kind": "textonpath"}));
                }
                json!({"effects": list})
            }
            "vg_separate" => {
                // The object and its effects become plain objects, as
                // Flatten Effects makes them; a drop shadow becomes a
                // curve of its colour and opacity under the object (its
                // blur is not kept).
                let id = ShapeId(id_arg(a, "id")?);
                let mut s = self.shape(id.raw())?;
                let layer = self.layer_of(id)?;
                let mut fresh = Vec::new();
                if let Some(sh) = s.shadow {
                    let on_page = self.doc.shape_on_page(id).ok_or("no shape")?;
                    let path = Affine::translate(sh.offset) * on_page.page_path();
                    let sid = self.new_id();
                    let mut shadow = Shape::new(sid, ShapeKind::Path { path, closed: true });
                    shadow.fill = Fill::Solid(sh.color);
                    shadow.stroke = None;
                    shadow.opacity = sh.opacity.clamp(0.0, 1.0);
                    self.run(Command::SetShadow {
                        shapes: vec![id],
                        shadow: None,
                    })?;
                    self.run(Command::AddShape {
                        layer,
                        shape: shadow,
                    })?;
                    self.run(Command::OrderRelative {
                        shapes: vec![sid],
                        reference: id,
                        in_front: false,
                    })?;
                    fresh.push(sid.raw());
                    s.shadow = None;
                    if s.effects.is_empty() {
                        fresh.push(id.raw());
                        return Ok(Some(json!({"ids": fresh})));
                    }
                }
                let next = self.doc.ids().peek_next();
                let mut k = 0u64;
                let parts = tracedraw_core::live::break_apart(&s, || {
                    let id = ShapeId(next + k);
                    k += 1;
                    id
                });
                self.run(Command::DeleteShapes { shapes: vec![id] })?;
                for p in parts {
                    let p = self.fresh_ids(p);
                    fresh.push(p.id.raw());
                    self.run(Command::AddShape { layer, shape: p })?;
                }
                self.selection.retain(|x| *x != id);
                json!({"ids": fresh})
            }
            _ => return Ok(None),
        }))
    }
}

/// The JavaScript object model.
pub(super) const PRELUDE: &str = include_str!("vg_model.js");

#[cfg(test)]
#[path = "vg_tests.rs"]
mod tests;
