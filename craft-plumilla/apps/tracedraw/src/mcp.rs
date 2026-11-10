//! `tracedraw --mcp`: a Model Context Protocol server on stdin/stdout.
//!
//! The editor runs headless and answers JSON-RPC 2.0 messages (one per
//! line) with the MCP methods `initialize`, `ping`, `tools/list` and
//! `tools/call`. The tools open and save files in every format the
//! editor knows, export pages, run automation scripts (the same
//! JavaScript object model as Tools > Scripts), describe the document
//! and undo or redo. An agent connected to this server edits documents
//! the way a user does: every change is a command with undo.

use crate::app::App;
use crate::describe::document as describe;
use serde_json::{Value, json};
use std::io::{BufRead, Write};

const PROTOCOL_VERSION: &str = "2024-11-05";

/// Run the server until stdin closes.
pub fn serve() {
    let mut app = App::headless();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(reply) = handle_line(&mut app, line) else {
            continue;
        };
        let mut out = stdout.lock();
        if writeln!(out, "{reply}").and_then(|_| out.flush()).is_err() {
            break;
        }
    }
}

/// One request line to one response line (`None` for notifications).
pub fn handle_line(app: &mut App, line: &str) -> Option<String> {
    let msg: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Some(
                json!({"jsonrpc": "2.0", "id": Value::Null,
                       "error": {"code": -32700, "message": format!("parse error: {e}")}})
                .to_string(),
            );
        }
    };
    let id = msg.get("id").cloned();
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = msg.get("params").cloned().unwrap_or(Value::Null);
    // Notifications carry no id and get no reply.
    let id = id?;
    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "craft-plumilla", "version": env!("CARGO_PKG_VERSION")},
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tool_list()})),
        "tools/call" => {
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match call_tool(app, name, &args) {
                Ok(text) => {
                    Ok(json!({"content": [{"type": "text", "text": text}], "isError": false}))
                }
                Err(e) => Ok(json!({"content": [{"type": "text", "text": e}], "isError": true})),
            }
        }
        other => Err((-32601, format!("method not found: {other}"))),
    };
    Some(match result {
        Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}).to_string(),
        Err((code, message)) => {
            json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
                .to_string()
        }
    })
}

fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": {"type": "object", "properties": props, "required": required},
    })
}

pub fn tool_list() -> Vec<Value> {
    vec![
        tool(
            "open",
            "Open a file as the current document: .cdr, .tdraw, .svg, .svgz, .pdf, .ai, .eps, .ps, .dxf, .psd, .psb, .emf, .wmf, .plt, .txt, .rtf, .docx, or an image.",
            json!({"path": {"type": "string"}}),
            &["path"],
        ),
        tool(
            "new_document",
            "Start a new document with a page of the given size in millimetres (default A4).",
            json!({"width_mm": {"type": "number"}, "height_mm": {"type": "number"}, "title": {"type": "string"}}),
            &[],
        ),
        tool(
            "save",
            "Save the document to a path: .tdraw (native) or .cdr. For .cdr, version picks the file version (7 to 27; default: the version of the .cdr file the document came from, else the one set in the options).",
            json!({"path": {"type": "string"}, "version": {"type": "integer"}}),
            &["path"],
        ),
        tool(
            "export",
            "Export svg, svgz, eps, dxf, emf, wmf, plt, psd or a raster image (png, jpg, jpeg, bmp, tif, tiff, webp, gif) from page_index (default 0). PDF, PDF-based AI, PS, PRN and HTML export the document. ai_version is pdf (default) or 8 (classic AI of the selected page). dpi defaults to 96; quality defaults to 90; webp_lossless defaults to false.",
            json!({"path": {"type": "string"}, "page_index": {"type": "integer", "minimum": 0}, "dpi": {"type": "number", "minimum": 1, "maximum": 2400}, "quality": {"type": "integer", "minimum": 1, "maximum": 100}, "webp_lossless": {"type": "boolean"}, "ai_version": {"type": "string", "enum": ["pdf", "8"]}}),
            &["path"],
        ),
        tool(
            "run_script",
            "Run JavaScript against the document. object_model defaults to native, the existing application API with CreateRectangle(x,y,width,height,radius). vgcore selects the separate macro object model: CreateRectangle(left,top,right,bottom,...), CreateRectangle2(x,y,width,height,...), document units and one-based collections. Coverage is partial; this does not execute COM or the original editor. Returns printed output; changes are one undo step.",
            json!({"source": {"type": "string"}, "object_model":{"type":"string","enum":["native","vgcore"],"default":"native"}}),
            &["source"],
        ),
        tool(
            "document_info",
            "Describe the document: title, pages, layers and every object with its id, type, name, bounds in millimetres, fill and outline.",
            json!({}),
            &[],
        ),
        tool(
            "import",
            "Import CDR, CMX, TDRAW or a raster image transactionally into the active drawing. Optional x_mm/y_mm place its centre.",
            json!({
                "path": {"type":"string"}, "x_mm":{"type":"number"}, "y_mm":{"type":"number"}
            }),
            &["path"],
        ),
        tool(
            "copy_selection",
            "Return SVG with versioned native selection metadata, including referenced resources. Optional ids select objects on the current page for this request only. Does not read or write the operating-system clipboard.",
            json!({"ids":{"type":"array","items":{"type":"integer","minimum":0},"uniqueItems":true,"maxItems":10000}}),
            &[],
        ),
        tool(
            "paste_selection",
            "Paste SVG or versioned native selection bytes into the current drawing, with one undo step. Damaged native data is rejected. Does not read or write the operating-system clipboard.",
            json!({"svg":{"type":"string","maxLength":67108864}}),
            &["svg"],
        ),
        tool("undo", "Undo the last change.", json!({}), &[]),
        tool("redo", "Redo the last undone change.", json!({}), &[]),
    ]
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("missing argument: {key}"))
}

pub fn call_tool(app: &mut App, name: &str, args: &Value) -> Result<String, String> {
    // Headless requests have no GUI frame to install the active document's ICC.
    app.sync_document_colors();
    let result = call_tool_inner(app, name, args);
    // Keep state coherent even when a tool returns an error after changing it.
    app.sync_document_colors();
    result
}

fn call_tool_inner(app: &mut App, name: &str, args: &Value) -> Result<String, String> {
    match name {
        "copy_selection" => {
            if app.editing_text() {
                return Err("copy_selection requires object selection, not text editing".into());
            }
            tracedraw_core::color_convert::validate_document_color_tree(app.doc())?;
            let requested = match args.get("ids") {
                None => None,
                Some(value) => {
                    let values = value.as_array().ok_or("ids must be an array")?;
                    if values.len() > crate::clipboard_wire::MAX_SHAPES {
                        return Err("ids exceeds the selection object limit".into());
                    }
                    let mut seen = std::collections::BTreeSet::new();
                    let mut selected = Vec::with_capacity(values.len());
                    for value in values {
                        let raw = value.as_u64().ok_or("ids must contain unsigned integers")?;
                        if !seen.insert(raw) {
                            return Err("ids contains duplicate object identifiers".into());
                        }
                        let id = tracedraw_core::ShapeId(raw);
                        let (layer, _) = app.doc().shape(id).map_err(|e| e.to_string())?;
                        if !app.page_layers().iter().any(|active| active.id == layer.id) {
                            return Err("a requested object is not on the current page".into());
                        }
                        selected.push(id);
                    }
                    Some(selected)
                }
            };
            let previous = requested.map(|ids| std::mem::replace(&mut app.selection, ids));
            let selected = app.selection.clone();
            let mut published = None;
            let result = app.try_copy_with_publisher(|svg| {
                published = Some(svg.to_owned());
                Ok(())
            });
            if let Some(previous) = previous {
                app.selection = previous;
            }
            if !result? {
                return Err("no selected objects to copy".into());
            }
            let svg = published.ok_or("selection publication produced no bytes")?;
            Ok(json!({
                "format":"craft-corel-selection-v1", "svg":svg,
                "selected_ids":selected.iter().map(|id|id.raw()).collect::<Vec<_>>()
            })
            .to_string())
        }
        "paste_selection" => {
            let svg = str_arg(args, "svg")?;
            if svg.len() > crate::clipboard_wire::MAX_SVG_BYTES {
                return Err("selection SVG exceeds its byte limit".into());
            }
            app.try_paste_svg_text(svg)?;
            Ok(json!({
                "document":describe(app),
                "selected_ids":app.selection.iter().map(|id|id.raw()).collect::<Vec<_>>(),
                "notice":app.status
            })
            .to_string())
        }
        "import" => {
            let path = std::path::PathBuf::from(str_arg(args, "path")?);
            let at = match (args.get("x_mm"), args.get("y_mm")) {
                (None, None) => None,
                (Some(x), Some(y)) => Some(tracedraw_core::Point::new(
                    x.as_f64()
                        .filter(|v| v.is_finite())
                        .ok_or("x_mm must be finite")?,
                    y.as_f64()
                        .filter(|v| v.is_finite())
                        .ok_or("y_mm must be finite")?,
                )),
                _ => return Err("import placement requires both x_mm and y_mm".into()),
            };
            let ids = app.try_import_path(&path, at)?;
            Ok(json!({"imported_ids":ids.iter().map(|id|id.raw()).collect::<Vec<_>>(), "document":describe(app), "notice":app.status}).to_string())
        }
        "open" => {
            let path = std::path::PathBuf::from(str_arg(args, "path")?);
            if !path.exists() {
                return Err(format!("no such file: {}", path.display()));
            }
            app.try_open_path(path)?;
            Ok(describe(app).to_string())
        }
        "new_document" => {
            let w = args
                .get("width_mm")
                .and_then(|v| v.as_f64())
                .unwrap_or(210.0);
            let h = args
                .get("height_mm")
                .and_then(|v| v.as_f64())
                .unwrap_or(297.0);
            if !(w > 0.0 && h > 0.0 && w < 10_000.0 && h < 10_000.0) {
                return Err("page size out of range".into());
            }
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| app.next_untitled_name());
            let doc = App::localized_document(title, tracedraw_core::geometry::Size::new(w, h));
            app.open_document(doc, None);
            Ok(describe(app).to_string())
        }
        "save" => {
            let path = std::path::PathBuf::from(str_arg(args, "path")?);
            let is_cdr = crate::app::is_cdr_path(&path);
            if !is_cdr
                && !path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("tdraw"))
            {
                return Err("save requires .tdraw or .cdr; use export for other formats".into());
            }
            let version = match args.get("version") {
                None | Some(Value::Null) => app.cdr_save_version(),
                Some(v) => v
                    .as_u64()
                    .and_then(|v| u16::try_from(v).ok())
                    .filter(|v| tracedraw_cdr::write::checked_versions().contains(v))
                    .ok_or_else(|| {
                        format!(
                            "version must be {} to {}",
                            tracedraw_cdr::write::CHECKED_SINCE,
                            tracedraw_cdr::write::CHECKED_UNTIL
                        )
                    })?,
            };
            let r = if is_cdr {
                std::fs::write(
                    &path,
                    tracedraw_cdr::write::document_to_cdr_version_with(
                        app.engine.document(),
                        version,
                        &crate::app::cdr_previews(app.engine.document()),
                    ),
                )
                .map_err(|e| e.to_string())
            } else {
                tracedraw_io::save_native(
                    &tracedraw_text::fonts().with_carried_fonts(app.engine.document()),
                    &path,
                )
                .map_err(|e| e.to_string())
            };
            r?;
            app.engine.mark_saved();
            app.file = Some(path.clone());
            if is_cdr {
                app.cdr_version = Some(version);
                app.cdr_unsaved_version = None;
                return Ok(format!("saved {} (version {version})", path.display()));
            }
            Ok(format!("saved {}", path.display()))
        }
        "export" => {
            let path = std::path::PathBuf::from(str_arg(args, "path")?);
            let page_index = match args.get("page_index") {
                None => 0,
                Some(value) => value
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or("page_index must be a non-negative integer")?,
            };
            let dpi = match args.get("dpi") {
                None => 96.0,
                Some(value) => value
                    .as_f64()
                    .filter(|n| n.is_finite() && (1.0..=2400.0).contains(n))
                    .ok_or("dpi must be between 1 and 2400")?,
            };
            let quality = match args.get("quality") {
                None => 90,
                Some(value) => value
                    .as_u64()
                    .filter(|n| (1..=100).contains(n))
                    .ok_or("quality must be an integer from 1 to 100")?
                    as u8,
            };
            let webp_lossless = match args.get("webp_lossless") {
                None => false,
                Some(value) => value.as_bool().ok_or("webp_lossless must be a boolean")?,
            };
            let classic_ai = match args.get("ai_version") {
                None => false,
                Some(Value::String(value)) if value == "pdf" => false,
                Some(Value::String(value)) if value == "8" => true,
                _ => return Err("ai_version must be pdf or 8".into()),
            };
            let options = tracedraw_io::raster::RasterOptions {
                dpi,
                quality,
                webp_lossless,
            };
            export_to(app, &path, page_index, &options, classic_ai)?;
            Ok(format!("exported {}", path.display()))
        }
        "run_script" => {
            let src = str_arg(args, "source")?;
            let out = match args.get("object_model") {
                None => crate::scripting::run(app, src),
                Some(Value::String(model)) if model == "native" => crate::scripting::run(app, src),
                Some(Value::String(model)) if model == "vgcore" => {
                    crate::scripting::run_vgcore(app, src, "Script")
                }
                Some(_) => return Err("object_model must be native or vgcore".into()),
            }?;
            Ok(if out.is_empty() {
                "ok".to_string()
            } else {
                out.join("\n")
            })
        }
        "document_info" => Ok(describe(app).to_string()),
        "undo" => {
            app.undo();
            Ok(describe(app).to_string())
        }
        "redo" => {
            app.redo();
            Ok(describe(app).to_string())
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

fn export_to(
    app: &App,
    path: &std::path::Path,
    page_index: usize,
    options: &tracedraw_io::raster::RasterOptions,
    classic_ai: bool,
) -> Result<(), String> {
    let doc = app.engine.document();
    if page_index >= doc.pages.len() {
        return Err(format!("page_index {page_index} out of range"));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let e = |r: tracedraw_io::Result<()>| r.map_err(|e| e.to_string());
    if tracedraw_io::raster::is_raster_path(path) {
        return e(tracedraw_io::raster::save_page(
            doc, page_index, path, options,
        ));
    }
    match ext.as_str() {
        "svg" => e(tracedraw_io::save_svg(doc, page_index, path)),
        "svgz" => {
            let svg = tracedraw_io::svg::page_to_svg(doc, page_index);
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(svg.as_bytes())
                .and_then(|_| encoder.finish())
                .and_then(|bytes| std::fs::write(path, bytes))
                .map_err(|e| e.to_string())
        }
        "ai" if classic_ai => std::fs::write(path, tracedraw_io::ai::page_to_ai8(doc, page_index))
            .map_err(|e| e.to_string()),
        "pdf" | "ai" => e(tracedraw_io::save_pdf(doc, path)),
        "eps" => std::fs::write(path, tracedraw_io::eps::page_to_eps(doc, page_index))
            .map_err(|e| e.to_string()),
        "ps" | "prn" => {
            std::fs::write(path, tracedraw_io::eps::document_to_ps(doc)).map_err(|e| e.to_string())
        }
        "dxf" => e(tracedraw_io::save_dxf(doc, page_index, path)),
        "emf" => e(tracedraw_io::save_emf(doc, page_index, path)),
        "wmf" => e(tracedraw_io::save_wmf(doc, page_index, path)),
        "plt" => e(tracedraw_io::save_plt(doc, page_index, path)),
        "psd" => e(tracedraw_io::save_psd(doc, page_index, options.dpi, path)),
        "html" => std::fs::write(path, tracedraw_io::html::document_to_html(doc))
            .map_err(|e| e.to_string()),
        other => Err(format!("unsupported export format: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(app: &mut App, id: u64, method: &str, params: Value) -> Value {
        let line =
            json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();
        let reply = handle_line(app, &line).expect("reply");
        serde_json::from_str(&reply).expect("json")
    }

    #[test]
    fn script_object_models_keep_distinct_rectangle_contracts_and_one_undo() {
        fn content(app: &App) -> Value {
            let mut value = serde_json::to_value(app.doc()).unwrap();
            value.as_object_mut().unwrap().remove("ids");
            value
        }
        for model in [None, Some("native"), Some("vgcore")] {
            let mut app = App::headless();
            let before = content(&app);
            let prefix = if model == Some("vgcore") {
                "ActiveDocument.Unit = cdrMillimeter; "
            } else {
                ""
            };
            let mut args = json!({"source":format!("{prefix}const r = ActiveLayer.CreateRectangle(10,30,50,10); print(r.SizeWidth, r.SizeHeight);")});
            if let Some(model) = model {
                args["object_model"] = json!(model);
            }
            let reply = call(
                &mut app,
                1,
                "tools/call",
                json!({"name":"run_script","arguments":args}),
            );
            assert_ne!(reply["result"]["isError"], true, "{reply}");
            let compatible = model == Some("vgcore");
            assert_eq!(
                reply["result"]["content"][0]["text"],
                if compatible { "40 20" } else { "50 10" }
            );
            let objects = &app.doc().pages[0].layers[0].shapes;
            assert_eq!(objects.len(), 1);
            let bounds = objects[0].bounds();
            let expected = if compatible {
                [10.0, 10.0, 50.0, 30.0]
            } else {
                [10.0, 30.0, 60.0, 40.0]
            };
            for (actual, expected) in [bounds.x0, bounds.y0, bounds.x1, bounds.y1]
                .into_iter()
                .zip(expected)
            {
                assert!((actual - expected).abs() < 1e-9, "{model:?}: {bounds:?}");
            }
            assert_eq!(app.engine.history_labels().0.len(), 1);
            assert_eq!(app.engine.undo_label(), Some("Script"));
            let after = content(&app);
            call_tool(&mut app, "undo", &json!({})).unwrap();
            assert_eq!(content(&app), before);
            call_tool(&mut app, "redo", &json!({})).unwrap();
            assert_eq!(content(&app), after);
        }
    }

    #[test]
    fn invalid_script_object_model_never_evaluates_or_discards_redo() {
        for model in [
            Value::Null,
            json!(true),
            json!(42),
            json!([]),
            json!({}),
            json!(""),
            json!("other"),
        ] {
            let mut app = App::headless();
            call_tool(
                &mut app,
                "run_script",
                &json!({"source":"ActiveLayer.CreateRectangle(1,2,3,4);"}),
            )
            .unwrap();
            app.undo();
            let before = app.doc().clone();
            let history = app.engine.history_labels();
            let sequence = app.engine.revision();
            let selection = app.selection.clone();
            let reply = call(
                &mut app,
                1,
                "tools/call",
                json!({"name":"run_script","arguments":{
                    "source":"ActiveLayer.CreateRectangle(5,6,7,8);", "object_model":model
                }}),
            );
            assert_eq!(reply["result"]["isError"], true, "{reply}");
            assert!(
                reply["result"]["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains("object_model")
            );
            assert_eq!(app.doc(), &before);
            assert_eq!(app.engine.history_labels(), history);
            assert_eq!(app.engine.revision(), sequence);
            assert_eq!(app.selection, selection);
        }
    }

    #[test]
    fn macro_document_unit_survives_rpc_calls_native_save_and_reversible_edits() {
        let mut app = App::headless();
        let script = |app: &mut App, source: &str| {
            call_tool(
                app,
                "run_script",
                &json!({"object_model":"vgcore","source":source}),
            )
        };
        script(&mut app, "ActiveDocument.Unit = cdrMillimeter;").unwrap();
        assert_eq!(app.doc().metadata.script_unit, Some(3));
        script(
            &mut app,
            "const r = ActiveLayer.CreateRectangle(10,30,50,10); print(r.SizeWidth, r.SizeHeight);",
        )
        .map(|text| assert_eq!(text, "40 20"))
        .unwrap();
        assert_eq!(app.engine.history_labels().0.len(), 2);
        let geometry = app.doc().pages[0].layers[0].shapes.clone();
        call_tool(&mut app, "undo", &json!({})).unwrap();
        assert!(app.doc().pages[0].layers[0].shapes.is_empty());
        assert_eq!(app.doc().metadata.script_unit, Some(3));
        let before_error = app.doc().clone();
        let history = app.engine.history_labels();
        assert!(script(&mut app, "ActiveDocument.Unit = 17;").is_err());
        assert_eq!(app.doc(), &before_error);
        assert_eq!(app.engine.history_labels(), history);
        call_tool(&mut app, "undo", &json!({})).unwrap();
        assert_eq!(app.doc().metadata.script_unit, None);
        call_tool(&mut app, "redo", &json!({})).unwrap();
        call_tool(&mut app, "redo", &json!({})).unwrap();
        assert_eq!(app.doc().metadata.script_unit, Some(3));
        assert_eq!(app.doc().pages[0].layers[0].shapes, geometry);

        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap()
            .join(".build/scripts-rectangulos-tests")
            .join(format!(
                "rpc-unit-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("unit-mm.tdraw");
        call_tool(&mut app, "save", &json!({"path":path})).unwrap();
        let mut fresh = App::headless();
        call_tool(&mut fresh, "open", &json!({"path":path})).unwrap();
        assert_eq!(fresh.doc().metadata.script_unit, Some(3));
        assert_eq!(
            script(
                &mut fresh,
                "print(ActiveDocument.Unit, ActivePage.Shapes.Item(1).SizeWidth);"
            )
            .unwrap(),
            "3 40"
        );
        assert_eq!(app.doc().pages[0].layers[0].shapes, geometry);
    }

    #[test]
    fn selection_bytes_cross_independent_rpc_apps_without_a_system_clipboard() {
        let mut source = App::headless();
        call_tool(&mut source, "run_script", &json!({"source":
            "ActiveLayer.CreateRectangle(1,2,3,4); ActiveLayer.CreateRectangle(11,12,13,14); ActiveLayer.CreateRectangle(21,22,23,24);"
        })).unwrap();
        let ids: Vec<_> = source.doc().pages[0].layers[0]
            .shapes
            .iter()
            .map(|shape| shape.id)
            .collect();
        source.selection = vec![ids[2]];
        let before_source = source.doc().clone();
        let source_history = source.engine.history_labels();
        let copied = call(
            &mut source,
            101,
            "tools/call",
            json!({
                "name":"copy_selection", "arguments":{"ids":[ids[0].raw(),ids[1].raw()]}
            }),
        );
        assert_eq!(copied["result"]["isError"], false, "{copied}");
        let packet: Value =
            serde_json::from_str(copied["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(packet["format"], "craft-corel-selection-v1");
        let bytes = packet["svg"].as_str().unwrap().to_owned();
        assert!(bytes.contains(crate::clipboard_wire::NAMESPACE));
        assert_eq!(source.doc(), &before_source);
        assert_eq!(source.engine.history_labels(), source_history);
        assert_eq!(
            source.selection,
            vec![ids[2]],
            "temporary request selection must be restored"
        );

        let mut target = App::headless();
        call_tool(
            &mut target,
            "run_script",
            &json!({"source":"ActiveLayer.CreateEllipse(40,40,10,10);"}),
        )
        .unwrap();
        let existing = target.doc().pages[0].layers[0].shapes[0].id;
        target.selection = vec![existing];
        let before = target.doc().clone();
        let history = target.engine.history_labels();
        let pasted = call(
            &mut target,
            102,
            "tools/call",
            json!({
                "name":"paste_selection", "arguments":{"svg":bytes}
            }),
        );
        assert_eq!(pasted["result"]["isError"], false, "{pasted}");
        assert_eq!(target.doc().pages[0].layers[0].shapes.len(), 3);
        assert_eq!(target.selection.len(), 2);
        assert_eq!(target.engine.history_labels().0.len(), history.0.len() + 1);
        let committed = target.doc().clone();
        let pasted_selection = target.selection.clone();
        call_tool(&mut target, "undo", &json!({})).unwrap();
        assert_eq!(target.doc().pages, before.pages);
        assert_eq!(target.doc().fonts, before.fonts);
        assert_eq!(target.doc().inks, before.inks);
        assert_eq!(target.selection, vec![existing]);
        assert_eq!(target.engine.history_labels().0, history.0);
        call_tool(&mut target, "redo", &json!({})).unwrap();
        assert_eq!(target.doc(), &committed);
        assert_eq!(target.selection, pasted_selection);
    }

    #[test]
    fn invalid_selection_tool_requests_preserve_the_document_and_selection() {
        let mut app = App::headless();
        call_tool(
            &mut app,
            "run_script",
            &json!({"source":"ActiveLayer.CreateRectangle(1,2,3,4);"}),
        )
        .unwrap();
        let id = app.doc().pages[0].layers[0].shapes[0].id;
        app.selection = vec![id];
        let before = app.doc().clone();
        let history = app.engine.history_labels();
        let edit_sequence = app.engine.revision();
        for (name, args) in [
            ("copy_selection", json!({"ids":"bad"})),
            ("copy_selection", json!({"ids":[id.raw(),id.raw()]})),
            ("copy_selection", json!({"ids":[-1]})),
            ("copy_selection", json!({"ids":[u64::MAX]})),
            ("copy_selection", json!({"ids":[]})),
            ("paste_selection", json!({})),
            ("paste_selection", json!({"svg":42})),
            (
                "paste_selection",
                json!({"svg":"<svg><metadata><cc:selection xmlns:cc=\"urn:craft-corel:selection\" version=\"99\" encoding=\"base64-json\">bad</cc:selection></metadata></svg>"}),
            ),
        ] {
            let reply = call(
                &mut app,
                103,
                "tools/call",
                json!({"name":name,"arguments":args}),
            );
            assert_eq!(reply["result"]["isError"], true, "{reply}");
            assert_eq!(app.doc(), &before);
            assert_eq!(app.engine.history_labels(), history);
            assert_eq!(app.engine.revision(), edit_sequence);
            assert_eq!(app.selection, vec![id]);
        }
    }

    #[test]
    fn initialize_list_and_call_tools() {
        let mut app = App::headless();
        let init = call(&mut app, 1, "initialize", json!({}));
        assert_eq!(init["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(init["result"]["serverInfo"]["name"], "craft-plumilla");
        // A notification gets no reply.
        assert!(
            handle_line(
                &mut app,
                &json!({"jsonrpc": "2.0", "method": "notifications/initialized"}).to_string()
            )
            .is_none()
        );
        let list = call(&mut app, 2, "tools/list", json!({}));
        let names: Vec<&str> = list["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(names.len(), 11);
        assert!(names.contains(&"run_script") && names.contains(&"export"));
        assert!(names.contains(&"copy_selection") && names.contains(&"paste_selection"));
        // New document, draw through a script, inspect, undo.
        let nd = call(
            &mut app,
            3,
            "tools/call",
            json!({"name": "new_document", "arguments": {"width_mm": 100, "height_mm": 50}}),
        );
        assert_eq!(nd["result"]["isError"], false);
        let run = call(
            &mut app,
            4,
            "tools/call",
            json!({"name": "run_script", "arguments": {"source":
                "var s = ActiveLayer.CreateRectangle(10, 10, 40, 30); s.Fill.ApplyUniformFill(Color.RGB(255, 0, 0)); print(ActivePage.Shapes.Count);"}}),
        );
        assert_eq!(run["result"]["isError"], false, "{run}");
        assert_eq!(run["result"]["content"][0]["text"], "1");
        let info = call(
            &mut app,
            5,
            "tools/call",
            json!({"name": "document_info", "arguments": {}}),
        );
        let text = info["result"]["content"][0]["text"].as_str().unwrap();
        let d: Value = serde_json::from_str(text).unwrap();
        assert_eq!(d["pages"][0]["size_mm"]["width"], 100.0);
        let objs = &d["pages"][0]["layers"][0]["objects"];
        assert_eq!(objs.as_array().unwrap().len(), 1);
        assert_eq!(objs[0]["type"], "rectangle");
        assert_eq!(objs[0]["fill"]["solid"], "R:255 G:0 B:0");
        let undo = call(
            &mut app,
            6,
            "tools/call",
            json!({"name": "undo", "arguments": {}}),
        );
        let d: Value =
            serde_json::from_str(undo["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(
            d["pages"][0]["layers"][0]["objects"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        // Unknown tool and unknown method.
        let bad = call(
            &mut app,
            7,
            "tools/call",
            json!({"name": "nope", "arguments": {}}),
        );
        assert_eq!(bad["result"]["isError"], true);
        let nm = call(&mut app, 8, "resources/list", json!({}));
        assert_eq!(nm["error"]["code"], -32601);
    }

    #[test]
    fn export_and_save_write_files() {
        let mut app = App::headless();
        let dir = std::env::temp_dir().join(format!("tracedraw-mcp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        call(
            &mut app,
            1,
            "tools/call",
            json!({"name": "run_script", "arguments": {"source": "ActiveLayer.CreateEllipse(10, 10, 60, 40);"}}),
        );
        for ext in ["svg", "pdf", "png", "emf", "cdr", "tdraw"] {
            let path = dir.join(format!("out.{ext}"));
            let tool = if ext == "cdr" || ext == "tdraw" {
                "save"
            } else {
                "export"
            };
            let r = call(
                &mut app,
                2,
                "tools/call",
                json!({"name": tool, "arguments": {"path": path.display().to_string()}}),
            );
            assert_eq!(r["result"]["isError"], false, "{ext}: {r}");
            assert!(
                std::fs::metadata(&path)
                    .map(|m| m.len() > 0)
                    .unwrap_or(false),
                "{ext}"
            );
        }
        // The saved .cdr opens again.
        let r = call(
            &mut app,
            3,
            "tools/call",
            json!({"name": "open", "arguments": {"path": dir.join("out.cdr").display().to_string()}}),
        );
        assert_eq!(r["result"]["isError"], false, "{r}");
        let d: Value =
            serde_json::from_str(r["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(d["pages"][0]["layers"][0]["objects"][0]["type"], "ellipse");
        // A version of choice, and one that cannot be written.
        let v21 = dir.join("v21.cdr");
        let r = call(
            &mut app,
            4,
            "tools/call",
            json!({"name": "save", "arguments": {"path": v21.display().to_string(), "version": 21}}),
        );
        assert_eq!(r["result"]["isError"], false, "{r}");
        let bytes = std::fs::read(&v21).unwrap_or_default();
        let (_, report) = tracedraw_cdr::open_bytes(&bytes, "v21").expect("opens");
        assert_eq!(report.version.map(|v| v.0), Some(21));
        assert_eq!(app.cdr_save_version(), 21);
        for bad in [json!(3), json!(6), json!(28), json!("x"), json!(-1)] {
            let r = call(
                &mut app,
                5,
                "tools/call",
                json!({"name": "save", "arguments": {"path": dir.join("bad.cdr").display().to_string(), "version": bad}}),
            );
            assert_eq!(r["result"]["isError"], true, "{bad}: {r}");
        }
        assert!(!dir.join("bad.cdr").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn raster_exports_use_real_codecs_and_ai_svgz_have_real_headers() {
        let mut app = App::headless();
        call_tool(
            &mut app,
            "new_document",
            &json!({"width_mm": 20, "height_mm": 10}),
        )
        .unwrap();
        call_tool(
            &mut app,
            "run_script",
            &json!({"source": "ActiveLayer.CreateEllipse(2, 2, 8, 8);"}),
        )
        .unwrap();
        let dir =
            std::env::temp_dir().join(format!("craft-plumilla-mcp-codecs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (ext, format) in [
            ("png", image::ImageFormat::Png),
            ("jpg", image::ImageFormat::Jpeg),
            ("bmp", image::ImageFormat::Bmp),
            ("tif", image::ImageFormat::Tiff),
            ("webp", image::ImageFormat::WebP),
            ("gif", image::ImageFormat::Gif),
        ] {
            let path = dir.join(format!("out.{ext}"));
            call_tool(
                &mut app,
                "export",
                &json!({"path": path, "dpi": 150, "webp_lossless": true}),
            )
            .unwrap();
            let bytes = std::fs::read(path).unwrap();
            assert_eq!(image::guess_format(&bytes).unwrap(), format);
            let decoded = image::load_from_memory(&bytes).unwrap();
            assert!(decoded.width() > 0 && decoded.height() > 0);
        }
        for (mode, signature) in [("pdf", &b"%PDF-"[..]), ("8", &b"%!PS-Adobe"[..])] {
            let path = dir.join(format!("out-{mode}.ai"));
            call_tool(
                &mut app,
                "export",
                &json!({"path": path, "ai_version": mode}),
            )
            .unwrap();
            assert!(std::fs::read(path).unwrap().starts_with(signature));
        }
        let path = dir.join("out.svgz");
        call_tool(&mut app, "export", &json!({"path": path})).unwrap();
        let data = std::fs::read(path).unwrap();
        assert!(data.starts_with(&[0x1f, 0x8b]));
        let mut text = String::new();
        std::io::Read::read_to_string(
            &mut flate2::read::GzDecoder::new(data.as_slice()),
            &mut text,
        )
        .unwrap();
        assert!(text.contains("<svg"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn import_returns_real_errors_and_commits_one_undo_step() {
        let mut app = App::headless();
        let dir =
            std::env::temp_dir().join(format!("craft-plumilla-mcp-import-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("owned-source.tdraw");
        call_tool(
            &mut app,
            "run_script",
            &json!({"source": "ActiveLayer.CreateRectangle(1, 2, 6, 7);"}),
        )
        .unwrap();
        call_tool(&mut app, "save", &json!({"path": path})).unwrap();
        let source = std::fs::read(&path).unwrap();
        call_tool(
            &mut app,
            "new_document",
            &json!({"width_mm": 100, "height_mm": 70}),
        )
        .unwrap();
        call_tool(
            &mut app,
            "run_script",
            &json!({"source": "ActiveLayer.CreateEllipse(10, 10, 20, 20);"}),
        )
        .unwrap();
        let before = app.doc().clone();
        let history = app.engine.history_labels();
        let revision = app.engine.revision();
        for arguments in [
            json!({"path": dir.join("missing.cdr")}),
            json!({"path": dir.join("unsupported.exe")}),
            json!({"path": path, "x_mm": 1.0}),
            json!({"path": path, "x_mm": "bad", "y_mm": 2.0}),
        ] {
            let reply = call(
                &mut app,
                91,
                "tools/call",
                json!({"name": "import", "arguments": arguments}),
            );
            assert_eq!(reply["result"]["isError"], true, "{reply}");
            assert_eq!(app.doc(), &before);
            assert_eq!(app.engine.history_labels(), history);
            assert_eq!(app.engine.revision(), revision);
        }
        let result: Value = serde_json::from_str(
            &call_tool(
                &mut app,
                "import",
                &json!({"path": path, "x_mm": 40.0, "y_mm": 30.0}),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(result["imported_ids"].as_array().unwrap().len(), 1);
        assert_eq!(app.engine.history_labels().0.len(), history.0.len() + 1);
        assert_eq!(app.doc().pages[0].layers[0].shapes.len(), 2);
        assert_eq!(std::fs::read(&path).unwrap(), source);
        call_tool(&mut app, "undo", &json!({})).unwrap();
        assert_eq!(app.doc().pages, before.pages);
        assert_eq!(app.engine.history_labels().0, history.0);
        call_tool(&mut app, "redo", &json!({})).unwrap();
        assert_eq!(app.doc().pages[0].layers[0].shapes.len(), 2);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn invalid_exports_and_save_formats_preserve_existing_files() {
        let mut app = App::headless();
        let dir =
            std::env::temp_dir().join(format!("craft-plumilla-mcp-refused-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sentinel.png");
        std::fs::write(&path, b"keep existing content").unwrap();
        for (key, value) in [
            ("dpi", json!(0)),
            ("dpi", json!(-1)),
            ("dpi", json!(2401)),
            ("dpi", json!("96")),
            ("quality", json!(0)),
            ("quality", json!(101)),
            ("quality", json!(90.5)),
            ("page_index", json!(-1)),
            ("page_index", json!(0.5)),
            ("page_index", json!(1)),
            ("webp_lossless", json!("true")),
            ("ai_version", json!("unsupported")),
        ] {
            let mut args = json!({"path": path});
            args[key] = value;
            assert!(call_tool(&mut app, "export", &args).is_err(), "{args}");
            assert_eq!(std::fs::read(&path).unwrap(), b"keep existing content");
        }
        assert!(call_tool(&mut app, "save", &json!({"path": path})).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"keep existing content");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_open_returns_an_error_and_preserves_the_current_drawing() {
        let mut app = App::headless();
        call_tool(
            &mut app,
            "run_script",
            &json!({"source": "ActiveLayer.CreateRectangle(1, 1, 5, 5);"}),
        )
        .unwrap();
        let before = describe(&app);
        let path =
            std::env::temp_dir().join(format!("craft-plumilla-mcp-bad-{}.tdraw", std::process::id()));
        std::fs::write(&path, b"{").unwrap();
        let result = call(
            &mut app,
            1,
            "tools/call",
            json!({"name": "open", "arguments": {"path": path}}),
        );
        assert_eq!(result["result"]["isError"], true, "{result}");
        assert_eq!(describe(&app), before);
        std::fs::remove_file(path).unwrap();
    }
}
