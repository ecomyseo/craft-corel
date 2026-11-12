//! Right-click context menus: object menu (Pick tool), node menu (Shape
//! tool), page menu (empty area), text editing menu.

use crate::app::{App, DockerTab};
use crate::i18n::tr;
use crate::tools::Tool;
use crate::ui::dialogs::Dialog;
use crate::ui::menus::{body, check, item, item_label, menu_popup_style, radio, sep, sub};
use egui::{Response, Ui};
use tracedraw_core::{document::ShapeKind, Command, ShapeId};

fn ci(ui: &mut Ui, close: &mut bool, key: &str, shortcut: &str, enabled: bool) -> bool {
    let r = item(ui, key, shortcut, enabled);
    if r {
        *close = true;
    }
    r
}

/// [`ci`] with a hover tip for the greyed row.
fn ci_tip(ui: &mut Ui, close: &mut bool, key: &str, enabled: bool, tip: Option<&str>) -> bool {
    let r = crate::ui::menus::item_tip(ui, key, "", enabled, tip);
    if r {
        *close = true;
    }
    r
}

/// The canvas context menu, drawn like the menus of the menu bar, at the
/// position of the right click. Clicking an item or anywhere else closes
/// it; submenus open on hover.
pub fn context_menu(app: &mut App, ui: &mut Ui, response: &Response) {
    let Some((pos, p)) = app.context_menu else {
        return;
    };
    let id = egui::Id::new("canvas_context_menu");
    let ctx = ui.ctx().clone();
    // A new right click (a new position) opens the menu afresh.
    let pos_id = id.with("pos");
    if ctx.data(|d| d.get_temp::<egui::Pos2>(pos_id)) != Some(pos) {
        ctx.data_mut(|d| d.insert_temp(pos_id, pos));
        egui::Popup::open_id(&ctx, id);
    }
    let mut close = false;
    let shown = egui::Popup::new(
        id,
        ctx.clone(),
        egui::PopupAnchor::Position(pos),
        ui.layer_id(),
    )
    .kind(egui::PopupKind::Menu)
    .layout(egui::Layout::top_down_justified(egui::Align::Min))
    .style(menu_popup_style)
    .open_memory(None)
    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
    .show(|ui| {
        ui.set_min_width(200.0);
        body(ui, |ui| {
            let on_guide = app.selection.is_empty()
                && matches!(app.tool, Tool::Pick | Tool::FreeformPick)
                && app.guide_at(p).is_some();
            if let Some(hit) = app.tab_ruler_menu {
                tab_ruler_menu(app, ui, hit, &mut close);
            } else if let Some(locked) = app.context_locked {
                locked_menu(app, ui, locked, &mut close);
            } else if app.editing_text() {
                match app.word_issue_at(p) {
                    Some(w) => spelling_menu(app, ui, &w, &mut close),
                    None => text_edit_menu(app, ui, &mut close),
                }
            } else if app.tool == Tool::Shape {
                node_menu(app, ui, &mut close);
            } else if app.tool == Tool::Blend && !app.selected_blends().is_empty() {
                blend_menu(app, ui, &mut close);
            } else if app.tool == Tool::MeshFill && app.selected_mesh().is_some() {
                mesh_menu(app, ui, p, &mut close);
            } else if on_guide {
                guide_menu(app, ui, &mut close);
            } else if app.selection.is_empty() {
                match empty_area_menu(app, p) {
                    EmptyArea::Page => page_menu(app, ui, p, &mut close),
                    EmptyArea::Desktop => desktop_menu(app, ui, p, &mut close),
                }
            } else {
                object_menu(app, ui, &mut close);
            }
        })
    });
    if close || shown.is_none() {
        egui::Popup::close_id(&ctx, id);
        app.context_menu = None;
        app.context_locked = None;
        app.tab_ruler_menu = None;
        ctx.data_mut(|d| d.remove::<egui::Pos2>(pos_id));
    }
    let _ = response;
}

/// Right-click on the tab ruler (#1852): the four kinds of stop, the one
/// under the pointer checked (a pick changes it, or with no stop there
/// adds one of that kind), and Remove tab on a stop.
fn tab_ruler_menu(app: &mut App, ui: &mut Ui, hit: crate::tab_ruler::RulerHit, close: &mut bool) {
    let current = hit
        .stop
        .and_then(|i| app.tab_ruler().and_then(|r| r.stops.get(i).map(|s| s.kind)));
    for kind in tracedraw_core::TabKind::ALL {
        let key = crate::ui::dialogs::tab_kind_key(kind);
        if radio(ui, key, "", current == Some(kind)) {
            *close = true;
            app.tab_ruler_pick(hit, Some(kind));
        }
    }
    if hit.stop.is_some() {
        sep(ui);
        if ci(ui, close, "dialog.remove_tab", "", true) {
            app.tab_ruler_pick(hit, None);
        }
    }
}

/// Right-click on a guideline: delete, lock or unlock, and the Guidelines
/// docker.
fn guide_menu(app: &mut App, ui: &mut Ui, close: &mut bool) {
    let guides = app.page_guides();
    let selected = app.selected_guides.clone();
    let locked = !selected.is_empty()
        && selected
            .iter()
            .all(|i| guides.get(*i).is_some_and(|g| g.locked));
    if ci(
        ui,
        close,
        "menu.edit.undo",
        "Ctrl+Z",
        app.engine.undo_label().is_some(),
    ) {
        app.undo();
    }
    sep(ui);
    if ci(ui, close, "menu.edit.delete", "Delete", !locked) && !app.delete_selected_guides() {
        app.status = tr("status.guideline_locked");
    }
    sep(ui);
    if locked {
        if ci(ui, close, "guides.unlock", "", true) {
            app.set_guides_locked(&selected, false);
        }
    } else if ci(ui, close, "guides.lock", "", !selected.is_empty()) {
        app.set_guides_locked(&selected, true);
    }
    sep(ui);
    if ci(ui, close, "menu.object.properties", "", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Guidelines;
        if let Some(i) = selected.first().copied() {
            app.open_guide_properties(i);
        }
    }
}

/// Right-click while a text is being edited: the editing commands act on
/// its characters (#1377), and the Properties docker.
fn text_edit_menu(app: &mut App, ui: &mut Ui, close: &mut bool) {
    let picked = app.text_selection_active();
    if ci(
        ui,
        close,
        "menu.edit.undo",
        "Ctrl+Z",
        app.engine.undo_label().is_some(),
    ) {
        app.undo();
    }
    sep(ui);
    if ci(ui, close, "menu.edit.cut", "Ctrl+X", picked) {
        app.text_cut();
    }
    if ci(ui, close, "menu.edit.copy", "Ctrl+C", picked) {
        app.text_copy();
    }
    if ci(ui, close, "menu.edit.paste", "Ctrl+V", true) {
        app.text_paste(None);
    }
    if ci(ui, close, "menu.edit.delete", "Delete", picked) {
        app.text_delete_forward();
    }
    sep(ui);
    if ci(ui, close, "menu.edit.select_all", "Ctrl+A", true) {
        app.text_select_all();
    }
    sep(ui);
    if ci(ui, close, "menu.object.properties", "Alt+Enter", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Properties;
    }
}

/// Right-click with the Blend tool on a blend: a new start or end object
/// (the next click names it), a new path to follow, the path shown
/// (selected) and the blend detached from its path.
fn blend_menu(app: &mut App, ui: &mut Ui, close: &mut bool) {
    if ci(ui, close, "context.blend_new_start", "", true) {
        app.pending_blend_pick = Some(true);
        app.status = tr("status.click_blend_start");
    }
    if ci(ui, close, "context.blend_new_end", "", true) {
        app.pending_blend_pick = Some(false);
        app.status = tr("status.click_blend_end");
    }
    sep(ui);
    if ci(ui, close, "docker.new_path", "", true) {
        app.pending_blend_path = true;
        app.status = tr("status.click_blend_path");
    }
    let on_path = app.selected_blend_path().is_some();
    if ci(ui, close, "context.blend_show_path", "", app.blend_path_object().is_some()) {
        app.show_blend_path();
    }
    if ci(ui, close, "context.blend_detach_path", "", on_path) {
        app.detach_blend_path();
    }
    sep(ui);
    blend_compound_items(app, ui, close);
}

/// Map Nodes, Split, Fuse Start and Fuse End of the selected blend (the
/// Blend tool's right-click menu and the Blend docker).
pub fn blend_compound_items(app: &mut App, ui: &mut Ui, close: &mut bool) {
    let any = !app.selected_blends().is_empty();
    if ci(ui, close, "context.blend_map_nodes", "", any) {
        app.pending_map_nodes = Some(None);
        app.status = tr("status.click_start_node");
    }
    if ci(ui, close, "context.blend_split", "", app.can_split_blend()) {
        app.pending_blend_split = true;
        app.status = tr("status.click_blend_step");
    }
    if ci(ui, close, "context.blend_fuse_start", "", app.can_fuse_blend(false)) {
        app.fuse_blend(false);
    }
    if ci(ui, close, "context.blend_fuse_end", "", app.can_fuse_blend(true)) {
        app.fuse_blend(true);
    }
}

/// Right-click with the Mesh Fill tool on an object with a mesh fill: a
/// node on the grid line there, a crossing of two new lines there, the
/// selected node's lines deleted, the colour under the click for the
/// selected node, and the Properties panel.
fn mesh_menu(app: &mut App, ui: &mut Ui, p: tracedraw_core::Point, close: &mut bool) {
    let spot = app.mesh_spot_at(p);
    if ci(
        ui,
        close,
        "context.mesh_add_node",
        "",
        matches!(spot, Some(crate::mesh_tool::MeshSpot::OnLine)),
    ) {
        app.add_mesh_lines_at(p);
    }
    if ci(
        ui,
        close,
        "context.mesh_add_intersection",
        "",
        matches!(spot, Some(crate::mesh_tool::MeshSpot::InCell)),
    ) {
        app.add_mesh_lines_at(p);
    }
    if ci(ui, close, "menu.edit.delete", "Delete", app.selected_mesh_node().is_some()) {
        app.delete_mesh_lines();
    }
    // The node types, as the Shape tool's: To Line and To Curve shape
    // the grid lines through the node, Cusp, Smooth and Symmetrical tie
    // its opposite handles.
    use crate::mesh_tool::MeshNodeType;
    let node = app.selected_mesh_node().is_some();
    sep(ui);
    for (key, t) in [
        ("context.node_to_line", MeshNodeType::Line),
        ("context.node_to_curve", MeshNodeType::Curve),
    ] {
        if ci(ui, close, key, "", node) {
            app.set_mesh_node_type(t);
        }
    }
    sep(ui);
    let kind = app.selected_mesh_node_kind();
    for (key, t, k) in [
        ("context.node_cusp", MeshNodeType::Cusp, tracedraw_core::MeshNodeKind::Cusp),
        ("context.node_smooth", MeshNodeType::Smooth, tracedraw_core::MeshNodeKind::Smooth),
        (
            "context.node_symmetrical",
            MeshNodeType::Symmetric,
            tracedraw_core::MeshNodeKind::Symmetric,
        ),
    ] {
        if crate::ui::menus::check_if(ui, key, "", kind == Some(k), node) {
            *close = true;
            app.set_mesh_node_type(t);
        }
    }
    sep(ui);
    if ci(
        ui,
        close,
        "context.mesh_color_eyedropper",
        "",
        app.selected_mesh_node().is_some(),
    ) {
        app.mesh_node_color_from(p);
    }
    sep(ui);
    if ci(ui, close, "menu.object.properties", "Alt+Enter", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Properties;
    }
}

/// Right-click on a word of the text being edited that the spell checker
/// marks: Paste; the suggestions for a misspelled word, Delete Repeated
/// Word for a word said twice, Stop Ignoring for an ignored one; Ignore
/// All; the Spell Checker and the Thesaurus.
fn spelling_menu(
    app: &mut App,
    ui: &mut Ui,
    w: &crate::spell_assist::WordAt,
    close: &mut bool,
) {
    use crate::spell_assist::Issue;
    if ci(ui, close, "menu.edit.paste", "Ctrl+V", true) {
        app.text_paste(None);
    }
    sep(ui);
    match w.issue {
        Issue::Misspelled => {
            let suggestions = crate::spell::suggest(&w.word);
            if suggestions.is_empty() {
                ci(ui, close, "context.spell_no_suggestions", "", false);
            }
            for s in suggestions {
                if item_label(ui, &s, "", true) {
                    app.replace_word_with(w, &s);
                    *close = true;
                }
            }
        }
        Issue::Repeated => {
            if ci(ui, close, "context.spell_delete_repeated", "", true) {
                app.delete_repeated_word(w);
            }
        }
        Issue::Ignored => {
            if ci(ui, close, "context.spell_unignore", "", true) {
                let word = w.word.clone();
                app.unignore_word(&word);
            }
        }
    }
    if w.issue != Issue::Ignored {
        sep(ui);
        if ci(ui, close, "context.spell_ignore_all", "", true) {
            let word = w.word.clone();
            app.ignore_word(&word);
        }
    }
    sep(ui);
    if ci(ui, close, "menu.text.spell_check", "", true) {
        app.dialog = crate::ui::dialogs::Dialog::WritingTools(
            crate::ui::writing_tools::WritingToolsState::on_tab(0),
        );
    }
    if ci(ui, close, "menu.text.thesaurus", "", true) {
        app.dialog = crate::ui::dialogs::Dialog::WritingTools(
            crate::ui::writing_tools::WritingToolsState::on_tab(2),
        );
    }
}

/// Right-click on a locked object: only unlocking it (it is selected
/// once unlocked) or every locked object of the page.
fn locked_menu(app: &mut App, ui: &mut Ui, id: ShapeId, close: &mut bool) {
    if ci(ui, close, "menu.object.unlock_object", "", true) {
        app.unlock_object(id);
    }
    if ci(ui, close, "menu.object.unlock_all", "", true) {
        app.unlock_all();
    }
}

pub(crate) fn object_menu(app: &mut App, ui: &mut Ui, close: &mut bool) {
    let shapes = app.selected_shapes();
    let is_text = shapes
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Text { .. }));
    // A clip frame whose frame is a group ungroups too (#1605).
    let is_group = shapes
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Group { .. }) || s.is_clip_frame_of_group());
    let is_curve = shapes
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Path { .. }));
    let is_bitmap = shapes
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Bitmap { .. }));
    let many = shapes.len() > 1;
    if is_text {
        if ci(ui, close, "menu.text.edit_text", "Ctrl+Shift+T", true) {
            app.open_edit_text_dialog();
        }
        if ci(ui, close, "menu.text.convert", "", true) {
            app.toggle_text_kind();
        }
        if many
            && !app.joinable_text_lines().is_empty()
            && ci(ui, close, "menu.text.join_lines", "", true)
        {
            app.join_text_lines();
        }
        // Linked text content (#1568), offered when it can be used.
        if app.can_link_text_content() && ci(ui, close, "menu.text.link_content", "", true) {
            app.link_text_content();
        }
        if app.selection_has_content_link() && ci(ui, close, "menu.text.unlink_content", "", true) {
            app.unlink_text_content();
        }
        sep(ui);
    }
    if is_bitmap {
        // Edit Bitmap, then Remove Background (greyed until the AI
        // service makes pictures; the tip says why), then tracing,
        // cropping and remixing.
        let ai = app.ai_image_ready() && app.ai_source_bitmap().is_some() && !app.ai_image.busy();
        let tip = app.ai_image_tip();
        if ci(
            ui,
            close,
            "menu.bitmaps.edit_bitmap",
            "",
            app.selected_bitmap().is_some(),
        ) {
            app.edit_bitmap_externally();
        }
        // On this computer, no AI service needed (#231).
        if ci(
            ui,
            close,
            "menu.bitmaps.remove_background_local",
            "",
            app.selected_bitmap().is_some(),
        ) {
            crate::ui::bitmap_dialogs::open_remove_background(app);
        }
        if ci_tip(
            ui,
            close,
            "menu.bitmaps.remove_background",
            ai,
            tip.as_deref(),
        ) {
            app.ai_remove_background();
        }
        if ci(
            ui,
            close,
            "menu.bitmaps.save_bitmap_as",
            "",
            app.selected_bitmap().is_some(),
        ) {
            app.save_bitmap_as();
        }
        // The trace commands of the Image menu (#452), under Trace: the
        // automatic trace, the last trace again or its settings, and the
        // Centerline and Outline presets.
        sub(ui, "menu.bitmaps.trace", |ui| {
            if ci(ui, close, "menu.bitmaps.quick_trace", "", true) {
                app.quick_trace();
            }
            let last = app.last_trace.is_some();
            if ci(ui, close, "menu.bitmaps.repeat_last_trace", "", last) {
                app.repeat_last_trace();
            }
            if ci(ui, close, "menu.bitmaps.last_trace_settings", "", last) {
                app.open_last_trace_settings();
            }
            sep(ui);
            sub(ui, "menu.bitmaps.centerline_trace", |ui| {
                for (key, preset) in crate::ui::menus::CENTERLINE_TRACES {
                    if ci(ui, close, key, "", true) {
                        app.open_trace(preset);
                    }
                }
            });
            sub(ui, "menu.bitmaps.outline_trace", |ui| {
                for (key, preset) in crate::ui::menus::OUTLINE_TRACES {
                    if ci(ui, close, key, "", true) {
                        app.open_trace(preset);
                    }
                }
            });
        });
        if ci(ui, close, "menu.bitmaps.crop_bitmap", "", true) {
            app.set_tool(Tool::Crop);
        }
        if ci_tip(ui, close, "context.ai_remix", ai, tip.as_deref()) {
            app.ai_image.mode = crate::ai_image::Mode::Remix;
            app.ai_image.info = false;
            app.ai_source_from_selection();
            app.open_ai_generate();
        }
        sep(ui);
    }
    // Clones and their masters.
    let has_clone = app.selection_has_clone();
    let has_clones = app.selection_has_clones();
    if has_clone || has_clones {
        if has_clone {
            if ci(ui, close, "context.select_master", "", true) {
                app.select_clone_masters();
            }
            if ci(ui, close, "context.revert_to_master", "", true) {
                app.revert_clones_to_master();
            }
            if ci(ui, close, "context.unlink_clone", "", true) {
                app.unlink_clones();
            }
        }
        if has_clones && ci(ui, close, "context.select_clones", "", true) {
            app.select_clones_of_selection();
        }
        sep(ui);
    }
    // One object (a group most of all): edit it on its own.
    if ci(
        ui,
        close,
        "context.bring_into_focus",
        "",
        shapes.len() == 1 && app.focus_target() != Some(shapes[0].id),
    ) {
        app.bring_into_focus();
    }
    if ci(
        ui,
        close,
        "menu.object.convert_to_curves",
        "Ctrl+Q",
        app.can_convert_to_curves(),
    ) {
        app.convert_to_curves();
    }
    // Break Apart, or the drop shadow's or effect group's version of it.
    let label = app.break_apart_label();
    // Also a warped object (its warp is baked into curves) and artistic
    // text, as the Object menu and Ctrl+K do (#647).
    let can = is_curve
        || is_group
        || label != "menu.object.break_apart"
        || shapes.iter().any(App::breaks_apart);
    if ci(ui, close, label, "Ctrl+K", can) {
        app.break_apart();
    }
    if ci(ui, close, "menu.object.combine", "Ctrl+L", many) {
        app.combine();
    }
    sep(ui);
    if ci(ui, close, "menu.edit.cut", "Ctrl+X", true) {
        app.cut();
    }
    if ci(ui, close, "menu.edit.copy", "Ctrl+C", true) {
        app.copy_with_system();
    }
    if ci(ui, close, "menu.edit.duplicate", "Ctrl+D", true) {
        app.duplicate();
    }
    if ci(ui, close, "menu.edit.delete", "Delete", true) {
        app.delete_selection();
    }
    if ci(ui, close, "menu.object.add_to_shelf", "", true) && app.shelf_add_selection() {
        app.open_docker(DockerTab::Shelf);
    }
    sep(ui);
    sub(ui, "menu.object.order", |ui| {
        if ci(ui, close, "menu.object.to_front_of_page", "Ctrl+Home", true) {
            app.order_page(true);
        }
        if ci(ui, close, "menu.object.to_back_of_page", "Ctrl+End", true) {
            app.order_page(false);
        }
        if ci(ui, close, "menu.object.forward_one", "Ctrl+PgUp", true) {
            app.order(1);
        }
        if ci(ui, close, "menu.object.back_one", "Ctrl+PgDn", true) {
            app.order(2);
        }
        if ci(ui, close, "menu.object.in_front_of", "", true) {
            app.pending_order = Some(true);
        }
        if ci(ui, close, "menu.object.behind", "", true) {
            app.pending_order = Some(false);
        }
        if ci(ui, close, "menu.object.reverse_order", "", many) {
            app.reverse_order();
        }
    });
    if app.can_group() && ci(ui, close, "menu.object.group_group", "Ctrl+G", true) {
        app.group_selection();
    }
    if is_group {
        if ci(ui, close, "menu.object.group_ungroup", "Ctrl+U", true) {
            app.ungroup_selection();
        }
        if ci(ui, close, "menu.object.group_ungroup_all", "", true) {
            app.ungroup_all();
        }
    }
    // An object picked inside a group (Ctrl+click) leaves it.
    let chosen = app.selection.clone();
    if app.can_remove_from_group(&chosen) && ci(ui, close, "context.remove_from_group", "", true) {
        app.remove_from_group(&chosen);
    }
    // Draw in Perspective on: the objects move onto a plane of the grid.
    if let Some(kind) = app.perspective.as_ref().map(|s| s.kind) {
        use crate::perspective_draw::Plane;
        for (pl, key) in [
            (Plane::Left, "context.move_left_plane"),
            (Plane::Right, "context.move_right_plane"),
            (Plane::Top, "context.move_top_plane"),
            (Plane::Ground, "context.move_ground_plane"),
        ] {
            if kind.planes().contains(&pl) && ci(ui, close, key, "", true) {
                app.move_selection_to_plane(pl);
            }
        }
    }
    // Connectors run another way.
    let kinds: Vec<_> = app.selected_connectors().iter().map(|(_, c)| c.kind).collect();
    if !kinds.is_empty() {
        use crate::connectors::ConnectorKind;
        sub(ui, "context.convert_connector", |ui| {
            for (k, key) in [
                (ConnectorKind::Straight, "connector.straight"),
                (ConnectorKind::RightAngle, "connector.right_angle"),
                (ConnectorKind::Rounded, "connector.rounded"),
            ] {
                if ci(ui, close, key, "", kinds.iter().any(|x| *x != k)) {
                    app.convert_connectors(k);
                }
            }
        });
    }
    sep(ui);
    if ci(ui, close, "menu.object.hide_object", "", true) {
        app.set_visible(false);
    }
    let locked = shapes.iter().any(|s| s.locked);
    if ci(
        ui,
        close,
        if locked {
            "menu.object.unlock_object"
        } else {
            "menu.object.lock_object"
        },
        "",
        true,
    ) {
        app.set_locked(!locked);
    }
    sep(ui);
    if ci(ui, close, "menu.object.clip_frame_place_inside", "", true) {
        app.pending_clip_frame = true;
        app.status = tr("status.click_frame");
    }
    sub(ui, "context.frame_type", |ui| {
        let empty_frames = !app.empty_clip_frames().is_empty();
        if ci(ui, close, "context.frame_none", "", empty_frames) {
            app.frames_to_objects();
        }
        let framable = !app.framable().is_empty();
        if ci(ui, close, "context.frame_empty_clip_frame", "", framable) {
            app.objects_to_clip_frames();
        }
        if ci(
            ui,
            close,
            "context.frame_text",
            "",
            app.selection.len() == 1,
        ) {
            app.object_to_text_frame();
        }
    });
    sep(ui);
    if ci(ui, close, "menu.object.symmetry_create", "Alt+S", true) {
        app.create_symmetry();
    }
    if ci(ui, close, "context.create_symbol", "", true) {
        app.create_symbol_from_selection();
    }
    sub(ui, "context.internet_links", |ui| {
        if ci(ui, close, "docker.hyperlink", "", true) {
            app.show_dockers = true;
            app.docker_tab = DockerTab::Links;
        }
        // The selected object's link, in the web browser.
        let link = app
            .selected_shapes()
            .first()
            .and_then(|s| s.link.clone())
            .filter(|l| !l.trim().is_empty());
        if ci(ui, close, "context.launch_browser", "", link.is_some()) {
            if let Some(url) = link {
                app.open_url(&url);
            }
        }
    });
    sub(ui, "docker.object_styles", |ui| {
        if ci(ui, close, "docker.new_style_from_selection", "", true) {
            app.new_style_from_selection();
        }
        let styles = app.doc().object_styles.clone();
        if !styles.is_empty() {
            sep(ui);
        }
        for (i, os) in styles.iter().enumerate() {
            if item_label(ui, &os.name, "", true) {
                app.apply_style(i);
                *close = true;
            }
        }
        let linked = !app.styled_selection().is_empty();
        if linked {
            sep(ui);
        }
        if ci(ui, close, "context.apply_to_style", "", linked) {
            app.update_style_from_selection();
        }
        if ci(ui, close, "context.revert_to_style", "", linked) {
            app.revert_to_styles();
        }
        if ci(ui, close, "docker.break_style_link", "", linked) {
            app.break_style_links();
        }
        if ci(ui, close, "docker.open_object_styles", "", true) {
            app.show_dockers = true;
            app.docker_tab = DockerTab::ObjectStyles;
        }
    });
    sub(ui, "docker.color_styles", |ui| {
        if ci(ui, close, "docker.new_from_selection", "", true) {
            app.show_dockers = true;
            app.docker_tab = DockerTab::ColorStyles;
        }
    });
    sep(ui);
    // Wrap Paragraph Text: on or off, the seven styles and the offset.
    let (wrap, current) = app.wrap_state();
    sub(ui, "context.wrap_paragraph_text", |ui| {
        if check(ui, "context.wrap_text_on", "", wrap) {
            let ids = app.selection.clone();
            app.run(Command::SetWrapText {
                shapes: ids,
                wrap: !wrap,
            });
            *close = true;
        }
        sep(ui);
        for (mode, side) in tracedraw_core::TextWrap::STYLES {
            let here = wrap && current.mode == mode && current.side == side;
            if radio(ui, crate::ui::propbar::wrap_style_key(mode, side), "", here) {
                app.set_wrap_style(mode, side);
                *close = true;
            }
        }
        sep(ui);
        crate::ui::propbar::wrap_offset_field(app, ui, current.offset);
    });
    sep(ui);
    let of = shapes.iter().any(|s| s.overprint_fill);
    let oo = shapes.iter().any(|s| s.overprint_outline);
    if check(ui, "menu.object.overprint_fill", "", of) {
        app.toggle_overprint(true);
        *close = true;
    }
    if check(ui, "menu.object.overprint_outline", "", oo) {
        app.toggle_overprint(false);
        *close = true;
    }
    let hint = shapes.iter().all(|s| app.object_hinted(s.id));
    if check(ui, "menu.object.object_hinting", "", hint) {
        app.toggle_object_hinting();
        *close = true;
    }
    if ci(ui, close, "context.align_pixel_grid", "", true) {
        app.align_to_pixel_grid();
    }
    sep(ui);
    if ci(ui, close, "menu.object.properties", "Alt+Enter", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Properties;
    }
}

pub(crate) fn node_menu(app: &mut App, ui: &mut Ui, close: &mut bool) {
    use tracedraw_core::nodes::NodeType;
    // A B-spline's chosen control points: fixed (the curve passes
    // through them) or floating (#1758).
    if let Some(all_fixed) = app.chosen_spline_points() {
        if check(ui, "context.spline_fixed", "C", all_fixed) {
            app.set_selected_spline_fixed(true);
            *close = true;
        }
        if check(ui, "context.spline_floating", "S", !all_fixed) {
            app.set_selected_spline_fixed(false);
            *close = true;
        }
        sep(ui);
    }
    let has_nodes = !app.node_selection.is_empty();
    if ci(ui, close, "context.node_add", "", has_nodes) {
        app.add_node_midpoints();
    }
    // The chosen nodes go, the curve stays; with no node chosen the
    // selected object goes, as with the Delete key (#1982).
    if ci(
        ui,
        close,
        "context.node_delete",
        "Delete",
        has_nodes || !app.selection.is_empty(),
    ) {
        app.delete_nodes_or_selection();
    }
    sep(ui);
    if ci(
        ui,
        close,
        "context.node_join",
        "",
        app.node_selection.len() == 2,
    ) {
        app.join_selected_nodes();
    }
    if ci(ui, close, "context.node_break", "", has_nodes) {
        app.break_selected_nodes();
    }
    sep(ui);
    if ci(ui, close, "context.node_to_line", "", has_nodes) {
        app.selected_segments_to_line();
    }
    if ci(ui, close, "context.node_to_curve", "", has_nodes) {
        app.selected_segments_to_curve();
    }
    sep(ui);
    if ci(ui, close, "context.node_cusp", "C", has_nodes) {
        app.set_selected_node_type(NodeType::Cusp);
    }
    if ci(ui, close, "context.node_smooth", "S", has_nodes) {
        app.set_selected_node_type(NodeType::Smooth);
    }
    if ci(ui, close, "context.node_symmetrical", "Y", has_nodes) {
        app.set_selected_node_type(NodeType::Symmetrical);
    }
    sep(ui);
    if ci(ui, close, "context.node_reverse", "", true) {
        app.reverse_selected_curves();
    }
    if ci(ui, close, "context.node_reverse_subpaths", "", has_nodes) {
        app.reverse_selected_subpaths();
    }
    if ci(ui, close, "context.node_extract", "", has_nodes) {
        app.extract_subpath();
    }
    sep(ui);
    if ci(ui, close, "context.node_close", "", true) {
        app.close_selected_curves();
    }
    sub(ui, "context.close_path", |ui| {
        for (mode, key) in crate::close_path::CloseMode::ALL {
            if ci(ui, close, key, "", true) {
                app.close_selected_paths(mode);
            }
        }
    });
    if check(ui, "context.elastic_mode", "", app.elastic_mode) {
        app.elastic_mode = !app.elastic_mode;
        *close = true;
    }
    sep(ui);
    if ci(ui, close, "menu.object.properties", "Alt+Enter", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Properties;
    }
}

/// Where a right-click with nothing selected falls: on the page or on
/// the desktop around it (the reference has a menu for each).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmptyArea {
    Page,
    Desktop,
}

fn empty_area_menu(app: &App, p: tracedraw_core::Point) -> EmptyArea {
    if app.page_rect().contains(p) {
        EmptyArea::Page
    } else {
        EmptyArea::Desktop
    }
}

/// [`check`] that closes the menu when picked.
fn cc(ui: &mut Ui, close: &mut bool, key: &str, on: bool) -> bool {
    let r = check(ui, key, "", on);
    if r {
        *close = true;
    }
    r
}

/// Right-click on the page with nothing selected, as the reference's
/// Page menu: Full-Screen Preview, View, Create Object, Import, Paste and
/// Page Setup. (Its Insert Object row places OLE objects, which this
/// program does not have.)
fn page_menu(app: &mut App, ui: &mut Ui, p: tracedraw_core::Point, close: &mut bool) {
    if ci(ui, close, "menu.view.fullscreen_preview", "F9", true) {
        app.fullscreen_preview = true;
    }
    view_sub(app, ui, close);
    sep(ui);
    create_object_sub(app, ui, close);
    // A file into the drawing, placed with the import cursor (#1931).
    if ci(ui, close, "menu.file.import", "Ctrl+I", true) {
        app.import();
    }
    // The external clipboard may contain data even without a local copy.
    // Read it only on activation, just like Edit > Paste.
    if ci(ui, close, "menu.edit.paste", "Ctrl+V", true) {
        app.paste_any_at(Some(p));
    }
    sep(ui);
    if ci(ui, close, "context.page_setup", "", true) {
        let s = app.page_size();
        app.dialog = Dialog::PageSize {
            width: s.width,
            height: s.height,
            all_pages: true,
        };
    }
}

/// Right-click on the desktop around the page with nothing selected, as
/// the reference's Document Desktop menu: the page menu's rows, Paste in
/// View, AI Generate and Document Properties.
fn desktop_menu(app: &mut App, ui: &mut Ui, p: tracedraw_core::Point, close: &mut bool) {
    if ci(ui, close, "menu.view.fullscreen_preview", "F9", true) {
        app.fullscreen_preview = true;
    }
    view_sub(app, ui, close);
    sep(ui);
    create_object_sub(app, ui, close);
    // An empty symmetry whose mirror line passes through the click.
    if ci(
        ui,
        close,
        "menu.object.symmetry_create",
        "Alt+S",
        app.symmetry_edit.is_none(),
    ) {
        app.create_new_symmetry_at(p);
    }
    if ci(ui, close, "menu.file.import", "Ctrl+I", true) {
        app.import();
    }
    if ci(ui, close, "menu.edit.paste", "Ctrl+V", true) {
        app.paste_any_at(Some(p));
    }
    if ci(
        ui,
        close,
        "menu.edit.paste_in_view",
        "Ctrl+Shift+V",
        app.clipboard.is_some(),
    ) {
        app.paste_in_view();
    }
    sep(ui);
    sub(ui, "menu.object.ai_generate", |ui| {
        for (key, mode) in [
            ("menu.object.ai_create_image", crate::ai_image::Mode::Create),
            ("menu.object.ai_remix_image", crate::ai_image::Mode::Remix),
            (
                "menu.object.ai_create_vector",
                crate::ai_image::Mode::Vector,
            ),
            ("menu.object.ai_draw_words", crate::ai_image::Mode::Draw),
        ] {
            if ci(ui, close, key, "", true) {
                app.ai_image.mode = mode;
                app.ai_image.info = false;
                app.open_ai_generate();
            }
        }
    });
    sep(ui);
    if ci(ui, close, "menu.file.document_properties", "", true) {
        app.dialog = Dialog::DocumentProperties(Box::default());
    }
}

/// The page and desktop menus' View submenu, as the reference's: the
/// Toolbox, Color Palettes, the Property Bar and Status Bar, Rulers,
/// Grid, Pixel Grid, Guidelines, Printable Area, the rollovers' Live
/// Preview and Text Frames.
/// Right-click on the empty workspace with no drawing open, as the
/// reference's CorelDRAW Desktop menu: New, New From Template, Open; View;
/// New Window (greyed without a drawing); About.
pub(crate) fn workspace_menu(app: &mut App, ui: &mut Ui, close: &mut bool) {
    if ci(ui, close, "menu.file.new", "Ctrl+N", true) {
        app.request_new_document();
    }
    if ci(ui, close, "menu.file.new_from_template", "", true) {
        app.open_template_dialog();
    }
    if ci(ui, close, "menu.file.open", "Ctrl+O", true) {
        app.open_dialog();
    }
    sep(ui);
    view_sub(app, ui, close);
    sep(ui);
    let doc = app.has_document();
    if ci(ui, close, "menu.window.new_window", "", doc) {
        app.new_window();
    }
    sep(ui);
    if ci(ui, close, "menu.help.about", "", true) {
        app.dialog = Dialog::About;
    }
}

fn view_sub(app: &mut App, ui: &mut Ui, close: &mut bool) {
    sub(ui, "menu.view", |ui| {
        if cc(ui, close, "menu.window.toolbar_toolbox", app.show_toolbox) {
            app.show_toolbox = !app.show_toolbox;
        }
        sub(ui, "menu.window.color_palettes", |ui| {
            crate::ui::menus::color_palettes_body(app, ui)
        });
        if cc(ui, close, "menu.window.toolbar_property_bar", app.show_property_bar) {
            app.show_property_bar = !app.show_property_bar;
        }
        if cc(ui, close, "menu.window.toolbar_status_bar", app.show_status_bar) {
            app.show_status_bar = !app.show_status_bar;
        }
        if cc(ui, close, "menu.view.rulers", app.show_rulers) {
            app.show_rulers = !app.show_rulers;
        }
        if cc(ui, close, "menu.view.grid", app.show_grid) {
            app.show_grid = !app.show_grid;
        }
        if cc(ui, close, "menu.view.pixel_grid", app.show_pixel_grid) {
            app.show_pixel_grid = !app.show_pixel_grid;
        }
        if cc(ui, close, "menu.view.guidelines", app.show_guides) {
            app.show_guides = !app.show_guides;
        }
        if cc(ui, close, "menu.view.printable_area", app.show_printable_area) {
            app.show_printable_area = !app.show_printable_area;
        }
        if cc(ui, close, "menu.object.rollover_live_preview", app.rollover_live) {
            app.toggle_rollover_live_preview();
        }
        let frames = app.settings.show_text_frames;
        if cc(ui, close, "context.text_frames", frames) {
            app.settings.show_text_frames = !frames;
            app.settings.save();
        }
    });
}

/// The tools of the page and desktop menus' Create Object submenu, in
/// the reference's order (`None`: a separator).
const CREATE_OBJECT_TOOLS: [Option<Tool>; 14] = [
    None,
    Some(Tool::Text),
    Some(Tool::Table),
    // Artistic Media: a submenu of its modes.
    Some(Tool::BrushStrokes),
    Some(Tool::Bezier),
    None,
    Some(Tool::Ellipse),
    Some(Tool::Freehand),
    Some(Tool::GraphPaper),
    Some(Tool::CommonShapes),
    Some(Tool::Polygon),
    Some(Tool::Star),
    Some(Tool::Rectangle),
    Some(Tool::Spiral),
];

/// Create Object: the drawing tool to draw with next, the active one
/// marked, Artistic Media's modes, and New Symbol from the selection.
fn create_object_sub(app: &mut App, ui: &mut Ui, close: &mut bool) {
    sub(ui, "context.create_object", |ui| {
        for (i, tool) in CREATE_OBJECT_TOOLS.iter().enumerate() {
            let Some(tool) = *tool else {
                if i > 0 {
                    sep(ui);
                }
                continue;
            };
            if tool == Tool::BrushStrokes {
                sub(ui, "tool.brush_strokes", |ui| {
                    for mode in crate::media::MediaMode::ALL {
                        let on = app.tool == tool && app.media_mode == mode;
                        if radio(ui, mode.key(), "", on) {
                            *close = true;
                            app.set_tool(tool);
                            app.media_mode = mode;
                        }
                    }
                });
                continue;
            }
            if radio(ui, &format!("tool.{}", tool.id()), "", app.tool == tool) {
                *close = true;
                app.set_tool(tool);
            }
        }
        if ci(ui, close, "menu.object.symbol_new", "", !app.selection.is_empty()) {
            app.create_symbol_from_selection();
        }
    });
}

#[cfg(all(test, not(any(target_arch = "wasm32", target_os = "android"))))]
mod page_paste_tests {
    use super::*;
    use tracedraw_core::Point;

    #[test]
    fn page_context_paste_accepts_external_data_without_a_local_copy() {
        let mut app = App::headless();
        assert!(app.clipboard.is_none() && app.text_clip.is_none());
        let ctx = crate::theme::ui_context();
        let at = Point::new(150.0, 120.0);
        let mut close = false;
        let row = super::page_desktop_menu_tests::row_of(page_menu, "menu.edit.paste");
        crate::ui::menus::replay::start(row);
        let external = crate::clipboard::ClipboardContent {
            text: Some(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10mm\" height=\"20mm\" viewBox=\"0 0 10 20\"><rect width=\"10\" height=\"20\" fill=\"#15964f\"/></svg>".into(),
            ),
            ..Default::default()
        };
        let mut output = crate::clipboard::with_test_external_clipboard(external, || {
            ctx.run_ui(egui::RawInput::default(), |ui| {
                page_menu(&mut app, ui, at, &mut close);
            })
        });
        output.textures_delta.clear();
        let (hit, _) = crate::ui::menus::replay::finish();
        assert_eq!(hit.as_deref(), Some("menu.edit.paste"));
        assert!(close, "the enabled Paste command must close the menu");
        assert!(
            crate::clipboard::read_system().text.is_none(),
            "test input must be scoped"
        );
        let bounds = app.selection_bounds().expect("external SVG must be pasted");
        assert!((bounds.width() - 10.0).abs() < 1e-6);
        assert!((bounds.height() - 20.0).abs() < 1e-6);
        assert!((bounds.center().x - at.x).abs() < 1e-6);
        assert!((bounds.center().y - at.y).abs() < 1e-6);
        assert_eq!(
            app.doc()
                .all_layers()
                .map(|l| l.shapes.len())
                .sum::<usize>(),
            1
        );
        app.undo();
        assert_eq!(
            app.doc()
                .all_layers()
                .map(|l| l.shapes.len())
                .sum::<usize>(),
            0
        );
        app.redo();
        assert_eq!(app.selection_bounds(), Some(bounds));
    }
}

#[cfg(test)]
mod page_desktop_menu_tests {
    use super::*;
    use tracedraw_core::Point;

    type Menu = fn(&mut App, &mut Ui, tracedraw_core::Point, &mut bool);

    /// Click the `row`-th row of `menu` (submenus laid out inline) on
    /// `app`: the key clicked, and the number of rows.
    pub(super) fn click(app: &mut App, menu: Menu, row: usize) -> (Option<String>, usize) {
        crate::i18n::set_language("en");
        let ctx = crate::theme::ui_context();
        let mut close = false;
        crate::ui::menus::replay::start(row);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            menu(app, ui, Point::new(150.0, 120.0), &mut close)
        });
        out.textures_delta.clear();
        let r = crate::ui::menus::replay::finish();
        // The Color Palettes rows are Window > Color Palettes' own: they
        // close the popup through egui, as in the menu bar.
        if let Some(k) = r.0.as_deref() {
            let palettes = k.starts_with("palette") || k.starts_with("menu.window.palette");
            assert!(close || palettes, "a picked row closes the menu: {k}");
        }
        r
    }

    /// The keys of `menu`'s enabled rows, in order, each from a fresh
    /// drawing (a click changes it).
    fn keys(menu: Menu) -> Vec<String> {
        let mut out = Vec::new();
        let mut row = 0;
        loop {
            let mut app = App::headless();
            let (hit, rows) = click(&mut app, menu, row);
            out.extend(hit);
            row += 1;
            if row >= rows {
                return out;
            }
        }
    }

    /// The row of `key` in `menu`.
    pub(super) fn row_of(menu: Menu, key: &str) -> usize {
        row_in(menu, key, App::headless)
    }

    /// The row of `key` in `menu` on drawings made by `make`.
    fn row_in(menu: Menu, key: &str, make: impl Fn() -> App) -> usize {
        let mut row = 0;
        loop {
            let mut app = make();
            let (hit, rows) = click(&mut app, menu, row);
            if hit.as_deref() == Some(key) {
                return row;
            }
            row += 1;
            assert!(row < rows, "{key} not in the menu");
        }
    }

    fn in_order(keys: &[String], want: &[&str]) {
        let mut at = 0;
        for w in want {
            match keys[at..].iter().position(|k| k == w) {
                Some(i) => at += i + 1,
                None => panic!("{w} missing or out of order in {keys:?}"),
            }
        }
    }

    /// The page menu has the reference's rows in its order: Full-Screen
    /// Preview, View (toolbars, palettes, rulers, grids, guidelines,
    /// printable area, rollover preview, text frames), Create Object,
    /// Paste and Page Setup; the desktop menu adds Paste in View, AI
    /// Generate and Document Properties.
    #[test]
    fn the_page_and_desktop_menus_follow_the_reference() {
        let page = keys(page_menu);
        let view = [
            "menu.view.fullscreen_preview",
            "menu.window.toolbar_toolbox",
            "menu.window.toolbar_property_bar",
            "menu.window.toolbar_status_bar",
            "menu.view.rulers",
            "menu.view.grid",
            "menu.view.pixel_grid",
            "menu.view.guidelines",
            "menu.view.printable_area",
            "menu.object.rollover_live_preview",
            "context.text_frames",
            "tool.text",
            "tool.table",
            "media.preset",
            "tool.bezier",
            "tool.ellipse",
            "tool.freehand",
            "tool.graph_paper",
            "tool.polygon",
            "tool.star",
            "tool.rectangle",
            "tool.spiral",
        ];
        in_order(&page, &view);
        in_order(&page, &["tool.spiral", "menu.edit.paste", "context.page_setup"]);
        assert!(!page.iter().any(|k| k == "menu.edit.paste_in_view"));
        assert!(!page.iter().any(|k| k == "menu.file.document_properties"));
        let desktop = keys(desktop_menu);
        in_order(&desktop, &view);
        in_order(
            &desktop,
            &[
                "tool.spiral",
                "menu.object.symmetry_create",
                "menu.edit.paste",
                "menu.object.ai_create_image",
                "menu.object.ai_draw_words",
                "menu.file.document_properties",
            ],
        );
        assert!(!desktop.iter().any(|k| k == "context.page_setup"));
    }

    /// The rows act: a view toggle flips its setting, a tool row picks
    /// the tool, an Artistic Media mode picks the tool in that mode, Page
    /// Setup and Document Properties open their dialogs.
    #[test]
    fn the_page_and_desktop_rows_act() {
        let mut app = App::headless();
        let rulers = app.show_rulers;
        click(&mut app, page_menu, row_of(page_menu, "menu.view.rulers"));
        assert_eq!(app.show_rulers, !rulers);
        let grid = app.show_grid;
        click(&mut app, desktop_menu, row_of(desktop_menu, "menu.view.grid"));
        assert_eq!(app.show_grid, !grid);
        let live = app.rollover_live;
        click(&mut app, page_menu, row_of(page_menu, "menu.object.rollover_live_preview"));
        assert_eq!(app.rollover_live, !live);
        click(&mut app, page_menu, row_of(page_menu, "tool.rectangle"));
        assert_eq!(app.tool, Tool::Rectangle);
        click(&mut app, page_menu, row_of(page_menu, "media.sprayer"));
        assert_eq!(app.tool, Tool::BrushStrokes);
        assert_eq!(app.media_mode, crate::media::MediaMode::Sprayer);
        click(&mut app, page_menu, row_of(page_menu, "context.page_setup"));
        assert!(matches!(app.dialog, Dialog::PageSize { .. }));
        app.dialog = Dialog::None;
        click(&mut app, desktop_menu, row_of(desktop_menu, "menu.file.document_properties"));
        assert!(matches!(app.dialog, Dialog::DocumentProperties(_)));
        app.dialog = Dialog::None;
        click(&mut app, page_menu, row_of(page_menu, "menu.view.fullscreen_preview"));
        assert!(app.fullscreen_preview);
    }

    fn empty_app() -> App {
        let mut app = App::headless();
        app.close_all_documents();
        assert!(!app.has_document());
        app
    }

    fn workspace(app: &mut App, ui: &mut Ui, _p: Point, close: &mut bool) {
        workspace_menu(app, ui, close)
    }

    /// With no drawing open, the workspace menu has the reference's rows
    /// (New, New From Template, Open; View; New Window, greyed; About),
    /// and they act.
    #[test]
    fn the_empty_workspace_menu_follows_the_reference() {
        let mut keys = Vec::new();
        let mut row = 0;
        loop {
            let mut app = empty_app();
            let (hit, rows) = click(&mut app, workspace, row);
            keys.extend(hit);
            row += 1;
            if row >= rows {
                break;
            }
        }
        in_order(
            &keys,
            &[
                "menu.file.new",
                "menu.file.new_from_template",
                "menu.window.toolbar_toolbox",
                "menu.view.rulers",
                "menu.help.about",
            ],
        );
        assert!(!keys.iter().any(|k| k == "menu.window.new_window"), "{keys:?}");
        let mut app = empty_app();
        click(&mut app, workspace, row_in(workspace, "menu.help.about", empty_app));
        assert!(matches!(app.dialog, Dialog::About));
        let mut app = empty_app();
        click(&mut app, workspace, row_in(workspace, "menu.file.new", empty_app));
        assert!(app.has_document() || !matches!(app.dialog, Dialog::None));
    }

    /// With a copy on the clipboard the desktop menu's Paste in View
    /// pastes it, one more object on the page.
    #[test]
    fn the_desktop_menu_pastes_in_view() {
        let copied = || {
            let mut app = App::headless();
            let id = app
                .new_shape(ShapeKind::Rect {
                    rect: tracedraw_core::Rect::new(10.0, 10.0, 30.0, 20.0),
                    radius: 0.0,
                    corners: None,
                })
                .expect("shape");
            app.select(vec![id]);
            app.copy();
            app.select(Vec::new());
            app
        };
        let mut app = copied();
        let count = |app: &App| app.doc().all_layers().map(|l| l.shapes.len()).sum::<usize>();
        assert_eq!(count(&app), 1);
        let row = row_in(desktop_menu, "menu.edit.paste_in_view", copied);
        click(&mut app, desktop_menu, row);
        assert_eq!(count(&app), 2);
        assert_eq!(app.selection.len(), 1);
    }

    /// A right-click with nothing selected opens the page menu on the
    /// page and the desktop menu off it.
    #[test]
    fn the_empty_area_menu_depends_on_where_the_click_falls() {
        let app = App::headless();
        let page = app.page_rect();
        assert_eq!(empty_area_menu(&app, page.center()), EmptyArea::Page);
        assert_eq!(
            empty_area_menu(&app, Point::new(page.x1 + 20.0, page.center().y)),
            EmptyArea::Desktop
        );
        assert_eq!(
            empty_area_menu(&app, Point::new(page.center().x, page.y0 - 5.0)),
            EmptyArea::Desktop
        );
    }
}

#[cfg(test)]
mod blend_and_mesh_menu_tests {
    use super::*;
    use crate::mesh_tool::MeshSpot;
    use tracedraw_core::live::Effect;
    use tracedraw_core::{Color, Fill, Point, Rect};

    fn square(app: &mut App, x: f64, c: Color) -> ShapeId {
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(x, 0.0, x + 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("square");
        app.run(Command::SetFill {
            shapes: vec![id],
            fill: Fill::Solid(c),
        });
        id
    }

    /// Click the `row`-th row of a context menu body drawn by `menu`.
    fn click(app: &mut App, row: usize, menu: impl Fn(&mut App, &mut Ui, &mut bool)) -> Option<String> {
        crate::i18n::set_language("en");
        let ctx = crate::theme::ui_context();
        let mut close = false;
        crate::ui::menus::replay::start(row);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| menu(app, ui, &mut close));
        out.textures_delta.clear();
        crate::ui::menus::replay::finish().0
    }

    fn blend_of(app: &App, id: ShapeId) -> Option<Effect> {
        app.doc()
            .find_shape(id)?
            .effects
            .iter()
            .find(|e| matches!(e, Effect::Blend { .. }))
            .cloned()
    }

    /// The Blend tool's menu on a blend: New End and New Start wait for a
    /// click and then move the blend's end or start to the object
    /// clicked; a new path is followed, shown and detached.
    #[test]
    fn the_blend_menu_changes_its_ends_and_its_path() {
        let mut app = App::headless();
        let end = square(&mut app, 100.0, Color::rgb8(0, 160, 80));
        let start = square(&mut app, 0.0, Color::rgb8(250, 220, 0));
        let other = square(&mut app, 200.0, Color::rgb8(200, 0, 0));
        let another = square(&mut app, 300.0, Color::rgb8(0, 0, 200));
        let path = app
            .new_shape(ShapeKind::Path {
                path: tracedraw_core::effects::polygon(&[
                    Point::new(0.0, 50.0),
                    Point::new(150.0, 80.0),
                    Point::new(300.0, 50.0),
                ]),
                closed: false,
            })
            .expect("path");
        app.select(vec![start, end]);
        app.blend_selection(3, 0.0, 0.0, 0.0);
        app.refresh_blends();
        app.tool = Tool::Blend;
        app.select(vec![start]);
        // New Start, New End, New Path, Show Path (off), Detach (off).
        assert_eq!(click(&mut app, 1, blend_menu).as_deref(), Some("context.blend_new_end"));
        assert_eq!(app.pending_blend_pick, Some(false));
        app.pending_blend_pick = None;
        app.set_blend_end(other);
        match blend_of(&app, start) {
            Some(Effect::Blend { end_id, .. }) => assert_eq!(end_id, Some(other)),
            e => panic!("{e:?}"),
        }
        assert_eq!(click(&mut app, 0, blend_menu).as_deref(), Some("context.blend_new_start"));
        assert_eq!(app.pending_blend_pick, Some(true));
        app.pending_blend_pick = None;
        app.set_blend_start(another);
        assert!(blend_of(&app, start).is_none(), "the old start no longer blends");
        match blend_of(&app, another) {
            Some(Effect::Blend { end_id, .. }) => assert_eq!(end_id, Some(other)),
            e => panic!("{e:?}"),
        }
        assert_eq!(app.selection, vec![another]);
        assert_eq!(click(&mut app, 3, blend_menu), None, "Show Path waits for a path");
        app.set_blend_path(path);
        app.select(vec![another]);
        assert!(app.selected_blend_path().is_some());
        assert_eq!(click(&mut app, 3, blend_menu).as_deref(), Some("context.blend_show_path"));
        assert_eq!(app.selection, vec![path]);
        app.select(vec![another]);
        assert_eq!(click(&mut app, 4, blend_menu).as_deref(), Some("context.blend_detach_path"));
        assert!(app.selected_blend_path().is_none());
        // Map Nodes and Split wait for clicks; Fuse is off with one part.
        assert_eq!(click(&mut app, 5, blend_menu).as_deref(), Some("context.blend_map_nodes"));
        assert_eq!(app.pending_map_nodes, Some(None));
        app.pending_map_nodes = None;
        assert_eq!(click(&mut app, 6, blend_menu).as_deref(), Some("context.blend_split"));
        assert!(app.pending_blend_split);
        app.pending_blend_split = false;
        assert_eq!(click(&mut app, 7, blend_menu), None, "nothing to fuse");
        assert_eq!(click(&mut app, 8, blend_menu), None, "nothing to fuse");
        let mid = app.split_blend(1).expect("split");
        app.select(vec![mid]);
        assert_eq!(click(&mut app, 8, blend_menu).as_deref(), Some("context.blend_fuse_end"));
        assert!(app.doc().find_shape(mid).is_none());
        // Fuse and Split each undo in one step.
        app.undo();
        assert!(app.doc().find_shape(mid).is_some());
        app.undo();
        assert!(app.doc().find_shape(mid).is_none());
        // Each change was one step: undo brings the path back.
        app.undo();
        app.select(vec![another]);
        assert!(app.selected_blend_path().is_some());
    }

    /// The Mesh Fill tool's menu: Add Intersection inside a cell, Add Node
    /// on a grid line, each enabled only there; Delete takes the selected
    /// node's lines away.
    #[test]
    fn the_mesh_menu_adds_and_deletes_nodes_where_clicked() {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 40.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        app.tool = Tool::MeshFill;
        app.mesh_rows = 2;
        app.mesh_cols = 2;
        app.apply_mesh_fill(id);
        let grid = |app: &App| match &app.doc().find_shape(id).expect("s").fill {
            Fill::Mesh(m) => (m.rows, m.cols),
            _ => (0, 0),
        };
        let cell = Point::new(7.0, 13.0);
        let line = Point::new(20.0, 13.0);
        let node = Point::new(20.0, 20.0);
        assert_eq!(app.mesh_spot_at(cell), Some(MeshSpot::InCell));
        assert_eq!(app.mesh_spot_at(line), Some(MeshSpot::OnLine));
        assert_eq!(app.mesh_spot_at(node), Some(MeshSpot::OnNode));
        assert_eq!(app.mesh_spot_at(Point::new(90.0, 90.0)), None);
        // In a cell: Add Node off, Add Intersection on.
        assert_eq!(click(&mut app, 0, |a, ui, c| mesh_menu(a, ui, cell, c)), None);
        assert_eq!(
            click(&mut app, 1, |a, ui, c| mesh_menu(a, ui, cell, c)).as_deref(),
            Some("context.mesh_add_intersection")
        );
        assert_eq!(grid(&app), (3, 3));
        assert!(app.selected_mesh_node().is_some(), "the new node is selected");
        // Delete: the new node's two lines go.
        assert_eq!(
            click(&mut app, 2, |a, ui, c| mesh_menu(a, ui, cell, c)).as_deref(),
            Some("menu.edit.delete")
        );
        assert_eq!(grid(&app), (2, 2));
        // On a grid line: Add Node adds the one line across it.
        assert_eq!(
            click(&mut app, 0, |a, ui, c| mesh_menu(a, ui, line, c)).as_deref(),
            Some("context.mesh_add_node")
        );
        let (r, c) = grid(&app);
        assert_eq!(r + c, 5, "one line more");
    }
}

#[cfg(test)]
mod spelling_menu_tests {
    use super::*;
    use crate::spell_assist::{issue_at, Issue};
    use tracedraw_core::document::{ParagraphStyle, TextAlign, TextSpan};
    use tracedraw_core::Point;

    fn editing(text: &str) -> (App, ShapeId) {
        tracedraw_text::install();
        crate::i18n::set_language("en");
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Text {
                spans: vec![TextSpan::new(text, "Arial", 24.0)],
                origin: Point::new(10.0, 10.0),
                frame: None,
                align: TextAlign::Left,
                para: ParagraphStyle::default(),
                on_path: None,
            })
            .expect("a text");
        app.begin_text_edit(id);
        (app, id)
    }

    /// Click the `row`-th enabled row of the spelling menu of the word at
    /// character `idx`; the key of the row clicked.
    fn click(app: &mut App, idx: usize, row: usize) -> Option<String> {
        let w = issue_at(&app.edit_text(), idx, crate::spell::is_known, &app.spell_ignored)
            .expect("a marked word");
        let ctx = crate::theme::ui_context();
        let mut close = false;
        crate::ui::menus::replay::start(row);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            spelling_menu(app, ui, &w, &mut close);
        });
        out.textures_delta.clear();
        crate::ui::menus::replay::finish().0
    }

    /// A word said twice: Delete Repeated Word leaves it once, with one
    /// space, in one undo step.
    #[test]
    fn a_repeated_word_is_deleted_from_the_menu() {
        let (mut app, _) = editing("the the cat");
        let w = issue_at(&app.edit_text(), 5, |_| true, &app.spell_ignored).expect("w");
        assert_eq!(w.issue, Issue::Repeated);
        // Paste, then Delete Repeated Word.
        assert_eq!(click(&mut app, 5, 1).as_deref(), Some("context.spell_delete_repeated"));
        assert_eq!(app.edit_text(), "the cat");
    }

    /// A word the dictionaries do not know: its suggestions replace it;
    /// Ignore All stops marking it and Stop Ignoring marks it again.
    #[test]
    fn a_misspelled_word_takes_a_suggestion_or_is_ignored() {
        let word = "zqxvjk";
        let (mut app, _) = editing(&format!("say {word} now"));
        assert!(!crate::spell::is_known(word));
        // Paste, the suggestions (or the greyed No suggestions row), then
        // Ignore All.
        let n = crate::spell::suggest(word).len().max(1);
        assert_eq!(click(&mut app, 6, 1 + n).as_deref(), Some("context.spell_ignore_all"));
        assert!(app.spell_ignored.contains(word));
        let w = issue_at(&app.edit_text(), 6, crate::spell::is_known, &app.spell_ignored)
            .expect("still marked as ignored");
        assert_eq!(w.issue, Issue::Ignored);
        // Paste, then Stop Ignoring.
        assert_eq!(click(&mut app, 6, 1).as_deref(), Some("context.spell_unignore"));
        assert!(!app.spell_ignored.contains(word));
        let misspelled = "beautifull";
        let suggestions = crate::spell::suggest(misspelled);
        if let Some(first) = suggestions.first() {
            let (mut app, _) = editing(&format!("a {misspelled} day"));
            assert_eq!(click(&mut app, 4, 1).as_deref(), Some(first.as_str()));
            assert_eq!(app.edit_text(), format!("a {first} day"));
        }
    }

    /// The word under the pointer of the text being edited is the one the
    /// menu is about.
    #[test]
    fn the_word_under_the_pointer_is_found() {
        let (app, id) = editing("the the cat");
        let s = app.doc().find_shape(id).cloned().expect("text");
        let nodes = crate::text_nodes::char_nodes(&s);
        // A little above the baseline of the fifth character (the second
        // "the").
        let (_, at) = nodes.iter().find(|(c, _)| *c == 5).copied().expect("node");
        let probe = [Point::new(at.x + 0.5, at.y + 2.0), Point::new(at.x + 0.5, at.y - 2.0)];
        let found = probe.iter().find_map(|p| app.word_issue_at(*p));
        let w = found.expect("the repeated word under the pointer");
        assert_eq!((w.issue, w.start, w.end), (Issue::Repeated, 4, 7));
    }
}
