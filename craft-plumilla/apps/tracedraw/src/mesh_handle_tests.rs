//! The Mesh Fill tool's node handles and node types, through the
//! pointer: a node's handles show, a drag on one bends its grid lines
//! (its opposite handle following a smooth or symmetric node), a node
//! drag and a handle drag are one undo step each, and the native file
//! keeps handles and types.

use super::*;
use crate::mesh_tool::{MeshNodeType, MESH_HANDLE_BASE, MESH_NODE_BASE};
use tracedraw_core::{Fill, Mesh, MeshHandle, MeshNodeKind};

fn mesh(app: &App, id: ShapeId) -> Mesh {
    match &app.doc().find_shape(id).expect("shape").fill {
        Fill::Mesh(m) => m.clone(),
        other => panic!("no mesh: {other:?}"),
    }
}

fn at(app: &App, p: Point) -> Pos2 {
    app.view.to_screen(p)
}

/// The 50 x 30 mm rectangle with a 2 x 2 mesh, its centre node selected
/// by a click.
fn meshed() -> (egui::Context, App, ShapeId) {
    let (ctx, mut app, id) = setup();
    app.mesh_rows = 2;
    app.mesh_cols = 2;
    app.set_tool(Tool::MeshFill);
    app.apply_mesh_fill(id);
    for _ in 0..2 {
        frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    }
    let centre = mesh(&app, id).nodes[4].pos;
    let p = at(&app, centre);
    frame(&ctx, &mut app, vec![Event::PointerMoved(p)], Modifiers::NONE);
    frame(&ctx, &mut app, vec![button(p, true, PointerButton::Primary, Modifiers::NONE)], Modifiers::NONE);
    frame(&ctx, &mut app, vec![button(p, false, PointerButton::Primary, Modifiers::NONE)], Modifiers::NONE);
    frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    assert_eq!(app.selected_mesh_node(), Some((id, 4)));
    (ctx, app, id)
}

#[test]
fn mesh_node_handles_bend_the_grid_and_follow_the_node_type() {
    let (ctx, mut app, id) = meshed();
    // The centre node has a handle each way, a third of the way along.
    let hs = app.mesh_node_handles();
    assert_eq!(hs.len(), 4);
    let right = hs
        .iter()
        .find(|(k, ..)| *k == MESH_HANDLE_BASE + 4 * 4 + MeshHandle::Right as usize)
        .expect("right handle")
        .1;
    let m0 = mesh(&app, id);
    let (n, r) = (m0.nodes[4].pos, m0.nodes[5].pos);
    assert!((right - n.lerp(r, 1.0 / 3.0)).hypot() < 1e-6, "{right:?}");

    // Smooth, then the right handle dragged 6 mm up: the left handle
    // turns with it, its length kept; one undo step.
    assert!(app.set_mesh_node_type(MeshNodeType::Smooth));
    assert_eq!(mesh(&app, id).nodes[4].kind, MeshNodeKind::Smooth);
    let left_len = (mesh(&app, id).handle_at(4, MeshHandle::Left).expect("left") - n).hypot();
    let depth = app.engine.history_labels().0.len();
    let to = right + tracedraw_core::geometry::Vec2::new(0.0, -6.0);
    let (sa, sb) = (at(&app, right), at(&app, to));
    drag(&ctx, &mut app, sa, sb, Modifiers::NONE, Vec::new());
    assert_eq!(app.engine.history_labels().0.len(), depth + 1, "{:?}", app.engine.history_labels().0);
    let m = mesh(&app, id);
    let got = m.handle_at(4, MeshHandle::Right).expect("right");
    assert!((got - to).hypot() < 0.5, "{got:?} vs {to:?}");
    let left = m.handle_at(4, MeshHandle::Left).expect("left");
    let (vr, vl) = (got - n, left - n);
    assert!((vr.x * vl.y - vr.y * vl.x).abs() < 1e-6 * vr.hypot() * vl.hypot().max(1.0), "in line");
    assert!(vr.dot(vl) < 0.0, "opposite");
    assert!((vl.hypot() - left_len).abs() < 1e-6, "length kept");
    // The grid line now bends: its middle is off the straight line.
    assert!(m.is_curved());
    assert_eq!(app.selected_mesh_node(), Some((id, 4)));
    app.undo();
    assert_eq!(mesh(&app, id).handle_at(4, MeshHandle::Right), Some(right));
    app.redo();
    assert_eq!(mesh(&app, id), m);

    // Symmetrical: both handles as long.
    assert!(app.set_mesh_node_type(MeshNodeType::Symmetric));
    let m = mesh(&app, id);
    let (a, b) = (
        m.handle_at(4, MeshHandle::Right).expect("r") - n,
        m.handle_at(4, MeshHandle::Left).expect("l") - n,
    );
    assert!((a.hypot() - b.hypot()).abs() < 1e-6);
    assert!((a + b).hypot() < 1e-6);

    // The right neighbour's handle towards the node bent too.
    let mut bent = mesh(&app, id);
    let nb = bent.nodes[5].pos;
    bent.nodes[5].set_handle(MeshHandle::Left, nb + tracedraw_core::geometry::Vec2::new(-5.0, 4.0));
    app.run(tracedraw_core::Command::SetFill {
        shapes: vec![id],
        fill: Fill::Mesh(bent),
    });
    // To Line: the grid lines through the node straight again, at both
    // ends.
    assert!(app.set_mesh_node_type(MeshNodeType::Line));
    let m = mesh(&app, id);
    for h in MeshHandle::ALL {
        let nb = m.neighbour(4, h).expect("inner node");
        let p = m.handle_at(4, h).expect("handle");
        assert!((p - n.lerp(m.nodes[nb].pos, 1.0 / 3.0)).hypot() < 1e-6, "{h:?}");
        let q = m.handle_at(nb, h.opposite()).expect("back");
        assert!((q - m.nodes[nb].pos.lerp(n, 1.0 / 3.0)).hypot() < 1e-6, "{h:?} back");
    }
    // To Curve: handles of their own, the shape unchanged.
    assert!(app.set_mesh_node_type(MeshNodeType::Curve));
    let c = mesh(&app, id);
    assert!(c.nodes[4].handles.is_some());
    for h in MeshHandle::ALL {
        assert!((c.handle_at(4, h).expect("h") - m.handle_at(4, h).expect("h")).hypot() < 1e-6);
    }

    // A node drag is one undo step too.
    let depth = app.engine.history_labels().0.len();
    let from = n;
    let to = n + tracedraw_core::geometry::Vec2::new(4.0, 3.0);
    let (sa, sb) = (at(&app, from), at(&app, to));
    drag(&ctx, &mut app, sa, sb, Modifiers::NONE, Vec::new());
    assert_eq!(app.engine.history_labels().0.len(), depth + 1);
    assert!((mesh(&app, id).nodes[4].pos - to).hypot() < 0.5);

    // Saved and opened again: handles and types stay.
    assert!(app.set_mesh_node_type(MeshNodeType::Smooth));
    let saved = mesh(&app, id);
    let path = std::env::temp_dir().join(format!("mesh-handles-{}.tdraw", std::process::id()));
    tracedraw_io::save_native(app.doc(), &path).expect("save");
    let back = tracedraw_io::load_native(&path).expect("load");
    let _ = std::fs::remove_file(&path);
    let (_, s) = back.shape(id).expect("shape");
    assert_eq!(s.fill, Fill::Mesh(saved));
    let _ = MESH_NODE_BASE;
}

/// With no drawing open and the Welcome Screen closed, a right-click on
/// the empty workspace opens its menu (`ui::context::workspace_menu`).
#[test]
fn a_right_click_on_the_empty_workspace_opens_its_menu() {
    let ctx = crate::theme::ui_context();
    let mut app = App::headless();
    app.close_all_documents();
    // The Welcome Screen's tab closed: the empty workspace shows.
    app.settings.welcome_tab_closed = true;
    app.show_welcome = false;
    assert!(!app.has_document());
    for _ in 0..3 {
        frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    }
    assert!(!egui::Popup::is_any_open(&ctx));
    let p = Pos2::new(800.0, 600.0);
    frame(&ctx, &mut app, vec![Event::PointerMoved(p)], Modifiers::NONE);
    frame(&ctx, &mut app, vec![button(p, true, PointerButton::Secondary, Modifiers::NONE)], Modifiers::NONE);
    frame(&ctx, &mut app, vec![button(p, false, PointerButton::Secondary, Modifiers::NONE)], Modifiers::NONE);
    frame(&ctx, &mut app, Vec::new(), Modifiers::NONE);
    assert!(egui::Popup::is_any_open(&ctx), "no menu opened");
}
