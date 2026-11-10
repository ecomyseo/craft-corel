//! Editing outcomes driven through egui input, without a native window.

use super::*;
use tracedraw_core::geometry::{PathEl, Shape as _};
use tracedraw_core::live::Effect;
use tracedraw_core::{Color, Command, Fill, Shape, Stroke};

fn acceptance_roots(app: &App) -> Vec<Shape> {
    app.doc()
        .all_layers()
        .flat_map(|layer| layer.shapes.iter().cloned())
        .collect()
}

fn acceptance_tool(ctx: &egui::Context, app: &mut App, tool: Tool) {
    app.snap.off = true;
    app.set_tool(tool);
    let over_canvas = app.view.to_screen(Point::new(80.0, 70.0));
    frame(
        ctx,
        app,
        vec![Event::PointerMoved(over_canvas)],
        Modifiers::NONE,
    );
    frame(ctx, app, Vec::new(), Modifiers::NONE);
}

fn acceptance_drag(ctx: &egui::Context, app: &mut App, from: Point, to: Point) {
    let from = app.view.to_screen(from);
    let to = app.view.to_screen(to);
    drag(ctx, app, from, to, Modifiers::NONE, Vec::new());
}

fn acceptance_click(ctx: &egui::Context, app: &mut App, at: Pos2, mods: Modifiers) {
    frame(ctx, app, vec![Event::PointerMoved(at)], mods);
    frame(
        ctx,
        app,
        vec![button(at, true, PointerButton::Primary, mods)],
        mods,
    );
    frame(
        ctx,
        app,
        vec![button(at, false, PointerButton::Primary, mods)],
        mods,
    );
    frame(ctx, app, Vec::new(), Modifiers::NONE);
}

fn acceptance_path_ends(shape: &Shape) -> (Point, Point) {
    let path = shape.page_path();
    let points: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|element| match element {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => Some(*p),
            PathEl::ClosePath => None,
        })
        .collect();
    (
        *points.first().expect("path start"),
        *points.last().expect("path end"),
    )
}

#[test]
fn sketch_pointer_strokes_join_only_when_enabled_and_undo_as_one_edit() {
    for join in [true, false] {
        let (ctx, mut app, original) = setup();
        let untouched = app.doc().find_shape(original).expect("original").clone();
        app.select(Vec::new());
        app.sketch.join = join;
        app.sketch.join_distance = 2.0;
        app.sketch.smoothing = 0;
        acceptance_tool(&ctx, &mut app, Tool::Sketch);
        let a = Point::new(80.0, 80.0);
        let b = Point::new(110.0, 90.0);
        let c = Point::new(140.0, 70.0);
        acceptance_drag(&ctx, &mut app, a, b);
        assert_eq!(shape_count(&app), 2, "one new sketch stroke");
        let first = *app.selection.first().expect("new stroke selected");
        let first_shape = app.doc().find_shape(first).expect("first stroke");
        assert!(matches!(
            first_shape.kind,
            ShapeKind::Path { closed: false, .. }
        ));
        let (start, end) = acceptance_path_ends(first_shape);
        assert!((start - a).hypot() < 0.05 && (end - b).hypot() < 0.05);
        let before_second = acceptance_roots(&app);
        acceptance_drag(&ctx, &mut app, b, c);
        assert_eq!(shape_count(&app), if join { 2 } else { 3 });
        let selected = *app.selection.first().expect("result selected");
        if join {
            assert_eq!(selected, first, "joining edits the existing curve");
            let (start, end) = acceptance_path_ends(app.doc().find_shape(first).unwrap());
            assert!((start - a).hypot() < 0.05 && (end - c).hypot() < 0.05);
        } else {
            assert_ne!(selected, first);
            assert_eq!(
                app.doc().find_shape(first),
                before_second.iter().find(|s| s.id == first)
            );
            let (start, end) = acceptance_path_ends(app.doc().find_shape(selected).unwrap());
            assert!((start - b).hypot() < 0.05 && (end - c).hypot() < 0.05);
        }
        assert_eq!(app.doc().find_shape(original), Some(&untouched));
        let after_second = acceptance_roots(&app);
        app.undo();
        assert_eq!(
            acceptance_roots(&app),
            before_second,
            "one undo removes the second gesture"
        );
        app.redo();
        assert_eq!(acceptance_roots(&app), after_second);
    }
}

#[test]
fn sketch_short_input_and_a_distant_stroke_do_not_join_or_damage_the_previous_curve() {
    let (ctx, mut app, _) = setup();
    app.select(Vec::new());
    app.sketch.join = true;
    app.sketch.join_distance = 1.0;
    acceptance_tool(&ctx, &mut app, Tool::Sketch);
    let before = acceptance_roots(&app);
    app.finish_sketch(vec![Point::new(80.0, 80.0)]);
    assert_eq!(acceptance_roots(&app), before);
    acceptance_drag(
        &ctx,
        &mut app,
        Point::new(80.0, 80.0),
        Point::new(110.0, 90.0),
    );
    let first = *app.selection.first().expect("first sketch");
    let first_shape = app.doc().find_shape(first).unwrap().clone();
    acceptance_drag(
        &ctx,
        &mut app,
        Point::new(130.0, 80.0),
        Point::new(150.0, 100.0),
    );
    assert_eq!(shape_count(&app), 3);
    assert_ne!(app.selection[0], first);
    assert_eq!(app.doc().find_shape(first), Some(&first_shape));
}

#[test]
fn graph_paper_pointer_drag_creates_twelve_editable_cells_at_independent_expected_positions() {
    let (ctx, mut app, original) = setup();
    let untouched = app.doc().find_shape(original).unwrap().clone();
    let before = acceptance_roots(&app);
    app.graph_rows = 3;
    app.graph_cols = 4;
    acceptance_tool(&ctx, &mut app, Tool::GraphPaper);
    let a = Point::new(80.0, 70.0);
    let b = Point::new(140.0, 100.0);
    acceptance_drag(&ctx, &mut app, a, b);
    assert_eq!(shape_count(&app), 2);
    assert_eq!(app.selection.len(), 1);
    let group = app.doc().find_shape(app.selection[0]).unwrap();
    let ShapeKind::Group { children } = &group.kind else {
        panic!("Graph Paper must create an editable group of cells");
    };
    assert_eq!(children.len(), 12);
    let mut occupied = std::collections::HashSet::new();
    let mut ids = std::collections::HashSet::from([original, group.id]);
    for cell in children {
        assert!(matches!(cell.kind, ShapeKind::Rect { .. }));
        assert!(ids.insert(cell.id), "cell IDs must not collide");
        let bounds = (group.transform * cell.page_path()).bounding_box();
        assert!(close(bounds.width(), 15.0) && close(bounds.height(), 10.0));
        let column = ((bounds.x0 - 80.0) / 15.0).round() as i32;
        let row = ((bounds.y0 - 70.0) / 10.0).round() as i32;
        assert!((0..4).contains(&column) && (0..3).contains(&row));
        assert!(close(bounds.x0, 80.0 + f64::from(column) * 15.0));
        assert!(close(bounds.y0, 70.0 + f64::from(row) * 10.0));
        assert!(
            occupied.insert((row, column)),
            "no overlapping duplicate cells"
        );
    }
    assert_eq!(occupied.len(), 12);
    assert_eq!(ids.len(), 14);
    assert_eq!(app.doc().find_shape(original), Some(&untouched));
    let after = acceptance_roots(&app);
    app.undo();
    assert_eq!(
        acceptance_roots(&app),
        before,
        "the complete grid is one undo step"
    );
    app.redo();
    assert_eq!(acceptance_roots(&app), after);
}

#[test]
fn outline_color_tool_reaches_the_panel_and_its_outline_button_changes_only_stroke_paint() {
    let (ctx, mut app, original) = setup();
    app.run(Command::SetStroke {
        shapes: vec![original],
        stroke: Some(Stroke {
            color: Color::BLACK,
            width: 0.75,
            ..Stroke::default()
        }),
    });
    let before = app.doc().find_shape(original).unwrap().clone();
    acceptance_tool(&ctx, &mut app, Tool::OutlineColor);
    assert_eq!(app.tool, Tool::Pick);
    assert!(app.show_dockers);
    assert_eq!(app.docker_tab, crate::app::DockerTab::Color);
    for _ in 0..3 {
        frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    }
    let blue = Color::rgb8(20, 80, 210);
    app.color_docker.lock = false;
    app.mixer_color = blue;
    let texts = texts_drawn(&ctx, &mut app, Vec::new());
    let at =
        text_at(&texts, &crate::i18n::tr("docker.outline")).expect("the Outline action is visible");
    acceptance_click(&ctx, &mut app, at, Modifiers::NONE);
    let after = app.doc().find_shape(original).unwrap().clone();
    assert_eq!(
        after.fill, before.fill,
        "outline colour must not recolour the fill"
    );
    assert_eq!(after.kind, before.kind);
    assert_eq!(after.transform, before.transform);
    let old_stroke = before.stroke.as_ref().expect("initial stroke");
    let new_stroke = after.stroke.as_ref().expect("new stroke");
    assert_eq!(new_stroke.color, blue);
    assert_eq!(new_stroke.width, old_stroke.width);
    assert_eq!(app.selection, vec![original]);
    app.undo();
    assert_eq!(app.doc().find_shape(original), Some(&before));
    app.redo();
    assert_eq!(app.doc().find_shape(original), Some(&after));
}

#[test]
fn two_point_line_pointer_clicks_create_exact_endpoints_and_ctrl_constrains_the_second_point() {
    for constrain in [false, true] {
        let (ctx, mut app, original) = setup();
        let before = acceptance_roots(&app);
        app.select(Vec::new());
        app.settings.constrain_angle = 45.0;
        acceptance_tool(&ctx, &mut app, Tool::TwoPointLine);
        let a = Point::new(80.0, 80.0);
        let raw_b = Point::new(120.0, 84.0);
        let at = app.view.to_screen(a);
        acceptance_click(&ctx, &mut app, at, Modifiers::NONE);
        assert_eq!(shape_count(&app), 1, "the first point is only a preview");
        let at = app.view.to_screen(raw_b);
        acceptance_click(
            &ctx,
            &mut app,
            at,
            if constrain {
                Modifiers::CTRL
            } else {
                Modifiers::NONE
            },
        );
        assert_eq!(shape_count(&app), 2);
        let line = app.doc().find_shape(app.selection[0]).expect("new line");
        assert_ne!(line.id, original);
        assert!(matches!(line.kind, ShapeKind::Path { closed: false, .. }));
        let (start, end) = acceptance_path_ends(line);
        assert!((start - a).hypot() < 0.05);
        if constrain {
            assert!(close(end.y, a.y));
            assert!(end.x > a.x + 35.0);
        } else {
            assert!((end - raw_b).hypot() < 0.05);
        }
        let after = acceptance_roots(&app);
        app.undo();
        assert_eq!(acceptance_roots(&app), before);
        app.redo();
        assert_eq!(acceptance_roots(&app), after);
    }
}

#[test]
fn block_shadow_pointer_drag_keeps_the_source_and_commits_one_reversible_effect() {
    let (ctx, mut app, original) = setup();
    let before = app.doc().find_shape(original).unwrap().clone();
    let colour = Color::rgb8(12, 34, 56);
    app.block_shadow_color = colour;
    app.block_shadow_gap = 0.75;
    acceptance_tool(&ctx, &mut app, Tool::BlockShadow);
    acceptance_drag(
        &ctx,
        &mut app,
        Point::new(35.0, 25.0),
        Point::new(45.0, 32.0),
    );
    let after = app.doc().find_shape(original).unwrap().clone();
    assert_eq!(after.kind, before.kind);
    assert_eq!(after.fill, before.fill);
    assert_eq!(after.transform, before.transform);
    let shadows: Vec<_> = after
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::BlockShadow { offset, color, gap } => Some((*offset, *color, *gap)),
            _ => None,
        })
        .collect();
    assert_eq!(shadows.len(), 1);
    assert!(close(shadows[0].0.x, 10.0) && close(shadows[0].0.y, 7.0));
    assert_eq!(shadows[0].1, colour);
    assert_eq!(shadows[0].2, 0.75);
    assert_eq!(app.selection, vec![original]);
    app.undo();
    assert_eq!(app.doc().find_shape(original), Some(&before));
    app.redo();
    assert_eq!(app.doc().find_shape(original), Some(&after));
}

#[test]
fn action_lines_pointer_drag_commits_requested_segments_with_independent_parallel_positions() {
    let (ctx, mut app, original) = setup();
    let untouched = app.doc().find_shape(original).unwrap().clone();
    app.action_lines_count = 7;
    app.action_lines_radial = false;
    acceptance_tool(&ctx, &mut app, Tool::ActionLines);
    acceptance_drag(
        &ctx,
        &mut app,
        Point::new(80.0, 70.0),
        Point::new(150.0, 105.0),
    );
    assert_eq!(shape_count(&app), 2);
    let shape = app.doc().find_shape(app.selection[0]).unwrap();
    assert_ne!(shape.id, original);
    assert_eq!(shape.fill, Fill::None);
    assert!(shape.stroke.is_some());
    let path = shape.page_path();
    assert_eq!(path.elements().len(), 14);
    for (index, pair) in path.elements().chunks_exact(2).enumerate() {
        let (PathEl::MoveTo(a), PathEl::LineTo(b)) = (&pair[0], &pair[1]) else {
            panic!("one independent straight segment per action line");
        };
        let y = 70.0 + (index as f64 + 0.5) * 5.0;
        assert!(close(a.y, y) && close(b.y, y));
        assert!(a.x >= 80.0 - 0.05 && a.x < b.x);
        assert!(close(b.x, 150.0));
    }
    assert_eq!(app.doc().find_shape(original), Some(&untouched));
}

#[test]
fn spiral_pointer_drag_creates_three_editable_turns_in_both_radius_modes() {
    let mut geometries = Vec::new();
    for logarithmic in [false, true] {
        let (ctx, mut app, original) = setup();
        let before = acceptance_roots(&app);
        app.spiral_revolutions = 3;
        app.spiral_logarithmic = logarithmic;
        acceptance_tool(&ctx, &mut app, Tool::Spiral);
        let rect = tracedraw_core::Rect::new(80.0, 70.0, 140.0, 110.0);
        acceptance_drag(
            &ctx,
            &mut app,
            Point::new(rect.x0, rect.y0),
            Point::new(rect.x1, rect.y1),
        );
        assert_eq!(shape_count(&app), 2);
        let shape = app.doc().find_shape(app.selection[0]).unwrap();
        assert_ne!(shape.id, original);
        assert!(matches!(shape.kind, ShapeKind::Path { closed: false, .. }));
        let path = shape.page_path();
        let bounds = path.bounding_box();
        assert!(rect.inflate(0.1, 0.1).contains_rect(bounds));
        let centre = rect.center();
        let mut angles = Vec::new();
        let mut radii = Vec::new();
        for element in path.elements() {
            let point = match element {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => *p,
                PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => *p,
                PathEl::ClosePath => panic!("a spiral must remain open"),
            };
            assert!(point.x.is_finite() && point.y.is_finite());
            let x = (point.x - centre.x) / (rect.width() / 2.0);
            let y = (point.y - centre.y) / (rect.height() / 2.0);
            radii.push(x.hypot(y));
            if x.hypot(y) > 1e-6 {
                angles.push(y.atan2(x));
            }
        }
        let turn = std::f64::consts::TAU;
        // Exponential growth can concentrate the earlier turns near the
        // centre. Check the radial law and angular sweep, not an arbitrary
        // fraction of the drag rectangle occupied by the tight path bounds.
        assert!(radii.windows(2).all(|pair| pair[1] + 1e-5 >= pair[0]));
        assert!(radii.last().unwrap() - radii.first().unwrap() > 0.5);
        let (initial, halfway) = if logarithmic {
            ((-4.0_f64).exp(), (-2.0_f64).exp())
        } else {
            (0.0, 0.5)
        };
        assert!((radii[0] - initial).abs() < 1e-3);
        assert!((radii[radii.len() / 2] - halfway).abs() < 0.01);
        assert!((radii.last().unwrap() - 1.0).abs() < 1e-3);
        let rotation: f64 = angles
            .windows(2)
            .map(|pair| {
                let mut delta = pair[1] - pair[0];
                while delta > std::f64::consts::PI {
                    delta -= turn;
                }
                while delta < -std::f64::consts::PI {
                    delta += turn;
                }
                delta
            })
            .sum();
        assert!(
            rotation > turn * 2.75 && rotation < turn * 3.25,
            "{rotation}"
        );
        let (_, end) = acceptance_path_ends(shape);
        assert!((end - Point::new(rect.x1, centre.y)).hypot() < 0.05);
        geometries.push(path);
        let after = acceptance_roots(&app);
        app.undo();
        assert_eq!(acceptance_roots(&app), before);
        app.redo();
        assert_eq!(acceptance_roots(&app), after);
    }
    assert_ne!(
        geometries[0], geometries[1],
        "radius mode changes real geometry"
    );
}

#[test]
fn polygon_pointer_drag_creates_the_requested_vertex_count_and_one_undo_step() {
    for points in [5, 7] {
        let (ctx, mut app, original) = setup();
        let before = acceptance_roots(&app);
        app.polygon_points = points;
        acceptance_tool(&ctx, &mut app, Tool::Polygon);
        acceptance_drag(
            &ctx,
            &mut app,
            Point::new(80.0, 70.0),
            Point::new(140.0, 130.0),
        );
        assert_eq!(shape_count(&app), 2);
        let shape = app.doc().find_shape(app.selection[0]).unwrap();
        assert_ne!(shape.id, original);
        assert!(matches!(shape.kind, ShapeKind::Polygon { points: count, .. } if count == points));
        let path = shape.page_path();
        assert!(path
            .elements()
            .iter()
            .any(|element| matches!(element, PathEl::ClosePath)));
        let mut vertices = Vec::<Point>::new();
        for element in path.elements() {
            if let PathEl::MoveTo(point) | PathEl::LineTo(point) = element {
                if vertices
                    .iter()
                    .all(|other| (*other - *point).hypot() > 1e-6)
                {
                    vertices.push(*point);
                }
            }
        }
        assert_eq!(vertices.len(), points as usize);
        let bounds = path.bounding_box();
        assert!(close(bounds.x0, 80.0) && close(bounds.y0, 70.0));
        assert!(close(bounds.width(), 60.0) && close(bounds.height(), 60.0));
        assert_ne!(path.winding(Point::new(110.0, 100.0)), 0);
        let after = acceptance_roots(&app);
        app.undo();
        assert_eq!(acceptance_roots(&app), before);
        app.redo();
        assert_eq!(acceptance_roots(&app), after);
    }
}

#[test]
fn common_shape_pointer_drag_commits_a_closed_right_arrow_with_expected_inside_and_outside_points()
{
    let (ctx, mut app, original) = setup();
    let before = acceptance_roots(&app);
    app.common_shape = crate::common_shapes::CommonShape::RightArrow;
    acceptance_tool(&ctx, &mut app, Tool::CommonShapes);
    acceptance_drag(
        &ctx,
        &mut app,
        Point::new(80.0, 70.0),
        Point::new(140.0, 100.0),
    );
    assert_eq!(shape_count(&app), 2);
    let shape = app.doc().find_shape(app.selection[0]).unwrap();
    assert_ne!(shape.id, original);
    assert!(matches!(shape.kind, ShapeKind::Path { closed: true, .. }));
    let path = shape.page_path();
    let bounds = path.bounding_box();
    assert!(close(bounds.x0, 80.0) && close(bounds.y0, 70.0));
    assert!(close(bounds.width(), 60.0) && close(bounds.height(), 30.0));
    for point in [
        Point::new(81.0, 85.0),
        Point::new(110.0, 85.0),
        Point::new(139.0, 85.0),
    ] {
        assert_ne!(path.winding(point), 0, "arrow contains {point:?}");
    }
    for point in [Point::new(81.0, 71.0), Point::new(139.0, 71.0)] {
        assert_eq!(path.winding(point), 0, "arrow excludes {point:?}");
    }
    let after = acceptance_roots(&app);
    app.undo();
    assert_eq!(acceptance_roots(&app), before);
    app.redo();
    assert_eq!(acceptance_roots(&app), after);
}

#[test]
fn pan_pointer_drag_moves_only_the_view_by_the_actual_screen_displacement() {
    let (ctx, mut app, original) = setup();
    acceptance_tool(&ctx, &mut app, Tool::Pan);
    let document = app.doc().clone();
    let selected = app.selection.clone();
    let before = app.view;
    let from = before.to_screen(Point::new(80.0, 70.0));
    let delta = egui::vec2(60.0, -30.0);
    drag(
        &ctx,
        &mut app,
        from,
        from + delta,
        Modifiers::NONE,
        Vec::new(),
    );
    assert!((app.view.origin - before.origin - delta).length() < 0.05);
    assert_eq!(app.view.zoom, before.zoom);
    assert_eq!(app.doc(), &document);
    assert_eq!(app.selection, selected);
    assert_eq!(app.selection, vec![original]);
    let page_point = Point::new(35.0, 25.0);
    assert!(
        (app.view.to_screen(page_point) - before.to_screen(page_point) - delta).length() < 0.05
    );
}
