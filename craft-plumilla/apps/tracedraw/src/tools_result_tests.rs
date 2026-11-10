//! Every tool, driven through egui input, leaves a result that can be
//! checked: a change to the drawing that one undo takes back, one redo
//! brings again and the native file keeps; for the tools that do not
//! change the drawing, the view, a panel or a dialog they act on.

use super::*;
use tracedraw_core::Shape;

/// What a tool is expected to act on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// The drawing changes (new objects or changed ones).
    Drawing,
    /// The view moves or zooms; the drawing stays.
    View,
    /// A panel or dialog opens; the drawing stays.
    Panel,
    /// The selection changes; the drawing stays.
    Selection,
}

fn outcome(tool: Tool) -> Outcome {
    match tool {
        Tool::Zoom | Tool::Pan => Outcome::View,
        Tool::OutlinePen | Tool::OutlineColor => Outcome::Panel,
        Tool::Pick | Tool::FreeformPick => Outcome::Drawing,
        _ => Outcome::Drawing,
    }
}

fn roots(app: &App) -> Vec<Shape> {
    app.doc()
        .all_layers()
        .flat_map(|l| l.shapes.iter().cloned())
        .collect()
}

/// A drawing with a filled rectangle at (10, 10)-(60, 40) and a second,
/// unfilled-outline square at (80, 10)-(110, 40), nothing selected.
fn scene() -> (egui::Context, App, ShapeId, ShapeId) {
    let (ctx, mut app, a) = setup();
    let b = app
        .new_shape(ShapeKind::Rect {
            rect: Rect::new(80.0, 10.0, 110.0, 40.0),
            radius: 0.0,
            corners: None,
        })
        .expect("second");
    app.select(vec![b]);
    app.apply_fill(tracedraw_core::Fill::Solid(tracedraw_core::Color::rgb8(
        30, 90, 200,
    )));
    app.snap.off = true;
    app.select(Vec::new());
    for _ in 0..2 {
        frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    }
    (ctx, app, a, b)
}

fn at(app: &App, x: f64, y: f64) -> Pos2 {
    app.view.to_screen(Point::new(x, y))
}

fn click(ctx: &egui::Context, app: &mut App, p: Pos2) {
    frame(ctx, app, vec![Event::PointerMoved(p)], Modifiers::NONE);
    frame(ctx, app, vec![button(p, true, PointerButton::Primary, Modifiers::NONE)], Modifiers::NONE);
    frame(ctx, app, vec![button(p, false, PointerButton::Primary, Modifiers::NONE)], Modifiers::NONE);
    frame(ctx, app, Vec::new(), Modifiers::NONE);
}

fn double_click(ctx: &egui::Context, app: &mut App, p: Pos2) {
    frame(ctx, app, vec![Event::PointerMoved(p)], Modifiers::NONE);
    for _ in 0..2 {
        frame(ctx, app, vec![button(p, true, PointerButton::Primary, Modifiers::NONE)], Modifiers::NONE);
        frame(ctx, app, vec![button(p, false, PointerButton::Primary, Modifiers::NONE)], Modifiers::NONE);
    }
    frame(ctx, app, Vec::new(), Modifiers::NONE);
}

fn enter(ctx: &egui::Context, app: &mut App) {
    frame(ctx, app, vec![key(egui::Key::Enter, Modifiers::NONE)], Modifiers::NONE);
    frame(ctx, app, Vec::new(), Modifiers::NONE);
}

/// The gestures tried, in order, until one gives the tool's outcome: a
/// drag on empty page, a drag across the first object, a click on it, a
/// click on empty page, clicks along a line ended with a double-click
/// and Enter, a drag from the first object to the second.
fn gestures(ctx: &egui::Context, app: &mut App, tool: Tool, a: ShapeId, step: usize) -> bool {
    let empty_a = at(app, 120.0, 120.0);
    let empty_b = at(app, 170.0, 160.0);
    let on_a = at(app, 35.0, 25.0);
    let across_a = (at(app, 5.0, 25.0), at(app, 65.0, 25.0));
    let on_b = at(app, 95.0, 25.0);
    match step {
        0 => drag(ctx, app, empty_a, empty_b, Modifiers::NONE, Vec::new()),
        1 => {
            app.select(vec![a]);
            app.set_tool(tool);
            drag(ctx, app, across_a.0, across_a.1, Modifiers::NONE, Vec::new());
        }
        2 => {
            app.select(vec![a]);
            app.set_tool(tool);
            click(ctx, app, on_a);
        }
        3 => click(ctx, app, empty_a),
        4 => {
            click(ctx, app, empty_a);
            click(ctx, app, at(app, 150.0, 130.0));
            click(ctx, app, at(app, 160.0, 160.0));
            double_click(ctx, app, at(app, 130.0, 170.0));
            enter(ctx, app);
        }
        5 => {
            app.select(vec![a]);
            app.set_tool(tool);
            drag(ctx, app, on_a, on_b, Modifiers::NONE, Vec::new());
        }
        6 => {
            // Text: a click, then typing.
            click(ctx, app, empty_a);
            frame(ctx, app, vec![Event::Text("abc".into())], Modifiers::NONE);
            frame(ctx, app, vec![key(egui::Key::Escape, Modifiers::NONE)], Modifiers::NONE);
            frame(ctx, app, Vec::new(), Modifiers::NONE);
        }
        7 => {
            // Three points: a drag for the first two, a click for the third.
            drag(ctx, app, empty_a, empty_b, Modifiers::NONE, Vec::new());
            click(ctx, app, at(app, 140.0, 175.0));
        }
        8 => {
            // Crop: the area over the first object, then Enter.
            app.select(vec![a]);
            app.set_tool(tool);
            drag(ctx, app, at(app, 20.0, 15.0), at(app, 45.0, 35.0), Modifiers::NONE, Vec::new());
            enter(ctx, app);
            double_click(ctx, app, at(app, 30.0, 25.0));
        }
        9 => {
            // Eyedroppers: sample the first object, then apply to the second.
            click(ctx, app, on_a);
            click(ctx, app, on_b);
        }
        10 => {
            // A node or a corner of the selection dragged away.
            app.select(vec![a]);
            app.set_tool(tool);
            drag(ctx, app, at(app, 60.0, 40.0), at(app, 85.0, 70.0), Modifiers::NONE, Vec::new());
        }
        11 => {
            // Along the first object's left side.
            app.select(vec![a]);
            app.set_tool(tool);
            drag(ctx, app, at(app, 10.0, 15.0), at(app, 12.0, 35.0), Modifiers::NONE, Vec::new());
        }
        _ => return false,
    }
    true
}

/// What the tool did, after `step`'s gesture.
fn acted(app: &App, before: &[Shape], zoom: f32, origin: (f32, f32), panel: bool) -> bool {
    roots(app) != before
        || app.view.zoom != zoom
        || (app.view.origin.x, app.view.origin.y) != origin
        || (!panel && (app.show_dockers || !matches!(app.dialog, crate::ui::dialogs::Dialog::None)))
}

#[test]
fn every_tool_leaves_a_result_that_undoes_redoes_and_saves() {
    let mut report = Vec::new();
    let mut failures = Vec::new();
    let dir = std::env::temp_dir().join(format!("tool-results-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("dir");
    let mut seen = std::collections::HashSet::new();
    for tool in Tool::ALL {
        if !seen.insert(format!("{tool:?}")) {
            continue;
        }
        let want = outcome(tool);
        let mut done = None;
        for step in 0..12 {
            let (ctx, mut app, a, _) = scene();
            app.show_dockers = false;
            app.dialog = crate::ui::dialogs::Dialog::None;
            app.set_tool(tool);
            frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
            let before = roots(&app);
            let steps_before = app.engine.history_labels().0.len();
            let (zoom, origin) = (app.view.zoom, (app.view.origin.x, app.view.origin.y));
            let panel = app.show_dockers;
            if !gestures(&ctx, &mut app, tool, a, step) {
                break;
            }
            app.finish_text();
            if !acted(&app, &before, zoom, origin, panel) {
                continue;
            }
            let after = roots(&app);
            let changed = after != before;
            match want {
                Outcome::Drawing if changed => {
                    // The gesture's steps undone back to before, redone to
                    // after: one step for a single drag or click.
                    let labels: Vec<&str> = app.engine.history_labels().0.into_iter().skip(steps_before).collect();
                    let steps = labels.len();
                    for _ in 0..steps {
                        app.undo();
                    }
                    let undone = steps > 0 && roots(&app) == before;
                    for _ in 0..steps {
                        app.redo();
                    }
                    let redone = roots(&app) == after;
                    let single = step == 4 || steps == 1;
                    // The native file keeps the result.
                    let path = dir.join(format!("{tool:?}.tdraw"));
                    let saved = tracedraw_io::save_native(app.doc(), &path).is_ok();
                    let back = tracedraw_io::load_native(&path).ok();
                    let kept = back.is_some_and(|d| {
                        let r: Vec<Shape> = d.all_layers().flat_map(|l| l.shapes.iter().cloned()).collect();
                        r.len() == after.len()
                            && r.iter().zip(&after).all(|(x, y)| x.kind == y.kind && x.fill == y.fill && x.effects == y.effects)
                    });
                    done = Some((
                        step,
                        undone && redone && saved && kept && single,
                        format!("{steps} step(s) {labels:?}, undo {undone} redo {redone} save {saved} reopen {kept}"),
                    ));
                }
                Outcome::View if !changed && app.view.zoom != zoom || (app.view.origin.x, app.view.origin.y) != origin => {
                    done = Some((step, roots(&app) == before, "view".into()));
                }
                Outcome::Panel if !changed => {
                    done = Some((step, true, "panel".into()));
                }
                Outcome::Selection if !changed => {
                    done = Some((step, true, "selection".into()));
                }
                _ => continue,
            }
            break;
        }
        match done {
            Some((step, true, how)) => report.push(format!("{tool:?}: gesture {step}, {how}")),
            Some((step, false, how)) => failures.push(format!("{tool:?}: gesture {step}, {how}")),
            None => failures.push(format!("{tool:?}: no gesture gave a result")),
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    println!("{}", report.join("\n"));
    assert!(failures.is_empty(), "{} tools without a checked result:\n{}", failures.len(), failures.join("\n"));
}
