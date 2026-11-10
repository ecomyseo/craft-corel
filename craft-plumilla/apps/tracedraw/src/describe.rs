//! Descriptions of the document as JSON, for the programs that edit it
//! without seeing it: the MCP server (`mcp.rs`) and AI Drawing (`ai.rs`).

use crate::app::App;
use serde_json::{Value, json};
use tracedraw_core::{Fill, document::ShapeKind};

fn fill_summary(f: &Fill) -> Value {
    match f {
        Fill::None => json!("none"),
        Fill::Solid(c) => json!({"solid": crate::app::color_description(*c)}),
        Fill::Fountain(ft) => {
            json!({"fountain": format!("{:?}", ft.kind), "stops": ft.stops.len()})
        }
        Fill::Pattern(_) => json!("pattern"),
        Fill::Texture(_) => json!("texture"),
        Fill::Mesh(_) => json!("mesh"),
    }
}

/// One object: id, type, name, bounds in mm, fill, outline, opacity,
/// text and font, group children, effect count.
pub fn shape_json(s: &tracedraw_core::document::Shape) -> Value {
    enum Work<'a> {
        Shape(&'a tracedraw_core::document::Shape),
        Group(Value, usize),
    }
    let mut pending = vec![Work::Shape(s)];
    let mut values = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Shape(shape) => {
                let value = shape_header(shape);
                if let ShapeKind::Group { children } = &shape.kind {
                    pending.push(Work::Group(value, children.len()));
                    pending.extend(children.iter().rev().map(Work::Shape));
                } else {
                    values.push(value);
                }
            }
            Work::Group(mut value, count) => {
                let children = values.split_off(values.len() - count);
                value["children"] = Value::Array(children);
                values.push(value);
            }
        }
    }
    values.pop().expect("one root description")
}

fn shape_header(s: &tracedraw_core::document::Shape) -> Value {
    let b = s.bounds();
    let kind = match &s.kind {
        ShapeKind::Rect { .. } => "rectangle",
        ShapeKind::Ellipse { .. } => "ellipse",
        ShapeKind::Polygon { .. } => "polygon",
        ShapeKind::Path { .. } => "curve",
        ShapeKind::Text { .. } => "text",
        ShapeKind::Group { .. } => "group",
        ShapeKind::ClipFrame { .. } => "clip_frame",
        ShapeKind::Table(_) => "table",
        ShapeKind::SymbolInstance { .. } => "symbol",
        ShapeKind::Bitmap { .. } => "bitmap",
    };
    let mut v = json!({
        "id": s.id.0,
        "type": kind,
        "name": s.name,
        "bounds_mm": {"x0": round(b.x0), "y0": round(b.y0), "x1": round(b.x1), "y1": round(b.y1)},
        "fill": fill_summary(&s.fill),
        "outline": s.stroke.as_ref().map(|st| json!({
            "colour": crate::app::color_description(st.color),
            "width_mm": round(st.width),
        })),
        "opacity": s.opacity,
    });
    if let ShapeKind::Text { spans, .. } = &s.kind {
        v["text"] = json!(spans.iter().map(|sp| sp.text.as_str()).collect::<String>());
        if let Some(sp) = spans.first() {
            v["font"] = json!({"family": sp.font_family, "size_pt": sp.size_pt, "bold": sp.bold, "italic": sp.italic});
        }
    }
    if !s.effects.is_empty() {
        v["effects"] = json!(s.effects.len());
    }
    v
}

fn round(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

/// The whole document: pages, layers, every object (MCP `document_info`).
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub fn document(app: &App) -> Value {
    let doc = app.engine.document();
    let mut value = json!({
        "title": doc.title,
        "file": app.file.as_ref().map(|p| p.display().to_string()),
        "modified": app.engine.is_dirty(),
        "selection": app.selection.iter().map(|id| id.0).collect::<Vec<_>>(),
    });
    value["pages"] = Value::Array(
        doc.pages
            .iter()
            .map(|p| {
                let mut page = json!({
                    "id": p.id.0,
                    "name": p.name,
                    "size_mm": {"width": round(p.size.width), "height": round(p.size.height)},
                });
                page["layers"] = Value::Array(
                    p.layers
                        .iter()
                        .map(|l| {
                            let mut layer = json!({
                                "id": l.id.0,
                                "name": l.name,
                                "visible": l.visible,
                                "locked": l.locked,
                            });
                            layer["objects"] =
                                Value::Array(l.shapes.iter().map(shape_json).collect());
                            layer
                        })
                        .collect(),
                );
                page
            })
            .collect(),
    );
    value
}

/// The active page for AI Drawing: its size, its layers (the active one
/// marked) with their objects, and the selection. At most `max_objects`
/// objects are described (group children count); the rest are counted.
/// File paths are left out.
pub fn active_page(app: &App, max_objects: usize) -> Value {
    let doc = app.engine.document();
    let Some(page) = doc
        .pages
        .iter()
        .find(|p| p.id == app.page)
        .or(doc.pages.first())
    else {
        return json!({});
    };
    let active = app.active_layer();
    let mut budget = max_objects;
    let mut omitted = 0usize;
    let layers: Vec<Value> = page
        .layers
        .iter()
        .map(|l| {
            json!({
                "id": l.id.0,
                "name": l.name,
                "visible": l.visible,
                "locked": l.locked,
                "active": Some(l.id) == active,
                "objects": l
                    .shapes
                    .iter()
                    .filter_map(|s| capped_shape_json(s, &mut budget, &mut omitted))
                    .collect::<Vec<_>>(),
            })
        })
        .collect();
    let mut v = json!({
        "page": {
            "name": page.name,
            "width_mm": round(page.size.width),
            "height_mm": round(page.size.height),
        },
        "layers": layers,
        "selection": app.selection.iter().map(|id| id.0).collect::<Vec<_>>(),
    });
    if omitted > 0 {
        v["objects_not_listed"] = json!(omitted);
    }
    v
}

/// `shape_json` while `budget` lasts; objects past it are counted in
/// `omitted` (with their children).
fn capped_shape_json(
    s: &tracedraw_core::document::Shape,
    budget: &mut usize,
    omitted: &mut usize,
) -> Option<Value> {
    if *budget == 0 {
        *omitted += count(s);
        return None;
    }
    *budget -= 1;
    let mut v = shape_header(s);
    if let ShapeKind::Group { children } = &s.kind {
        v["children"] = json!(
            children
                .iter()
                .filter_map(|c| capped_shape_json(c, budget, omitted))
                .collect::<Vec<_>>()
        );
    }
    Some(v)
}

/// An object and every object inside it.
fn count(s: &tracedraw_core::document::Shape) -> usize {
    match &s.kind {
        ShapeKind::Group { children } => 1 + children.iter().map(count).sum::<usize>(),
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Rect;

    #[test]
    fn the_active_page_is_described_up_to_a_number_of_objects() {
        let mut app = App::headless();
        for i in 0..5 {
            let x = 10.0 * i as f64;
            app.new_shape(ShapeKind::Rect {
                rect: Rect::new(x, 0.0, x + 5.0, 5.0),
                radius: 0.0,
                corners: None,
            })
            .expect("a layer");
        }
        let all = active_page(&app, 100);
        let layer = &all["layers"][0];
        assert_eq!(layer["active"], true);
        assert_eq!(layer["objects"].as_array().map(Vec::len), Some(5));
        assert_eq!(layer["objects"][0]["type"], "rectangle");
        assert_eq!(all["page"]["width_mm"], 210.0);
        assert!(all.get("objects_not_listed").is_none());
        assert!(all.get("file").is_none());
        let some = active_page(&app, 2);
        assert_eq!(
            some["layers"][0]["objects"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(some["objects_not_listed"], 3);
    }
}
