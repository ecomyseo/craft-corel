//! Independent SDK-domain ruler preferences; no claims about standard CDR codes.
use crate::{
    app::{self, App, Units},
    scripting,
};
use tracedraw_core::{
    geometry::{Rect, Size},
    Document, Shape, ShapeKind,
};

const LABEL: &str = "Independent document units";

mod ruler_capture;

fn vg(app: &mut App, source: &str) {
    scripting::run_vgcore(app, source, LABEL).unwrap_or_else(|error| panic!("{source}\n{error}"));
}

fn fresh() -> App {
    let mut app = App::headless();
    app.units = Units::Millimeters;
    let doc = Document::new("Independent units", Size::new(100.0, 70.0));
    app.open_document(doc, None);
    vg(&mut app, "ActiveDocument.Unit=cdrMillimeter; ActiveLayer.CreateRectangle2(10,10,40,20,2,4,6,8).Name='Physical sentinel';");
    app.selection = vec![sentinel(&app).id];
    app
}

fn axes(app: &App) -> (Option<u32>, Option<u32>, Option<u32>) {
    let metadata = &app.doc().metadata;
    (
        metadata.script_unit,
        metadata.rulers.horizontal_unit,
        metadata.rulers.vertical_unit,
    )
}

fn sentinel(app: &App) -> &Shape {
    app.doc()
        .all_layers()
        .flat_map(|l| &l.shapes)
        .find(|s| s.name.as_deref() == Some("Physical sentinel"))
        .expect("physical sentinel")
}

fn physical_geometry(app: &App) {
    let ShapeKind::Rect {
        rect,
        radius: _,
        corners: Some(corners),
    } = &sentinel(app).kind
    else {
        panic!("native physical rectangle with independent corners expected");
    };
    assert_eq!(*rect, Rect::new(10.0, 10.0, 50.0, 30.0));
    assert_eq!(corners.radii, [2.0, 4.0, 8.0, 6.0]);
}

fn reversible(app: &mut App, before: &Document, history_len: usize) {
    assert_eq!(app.engine.history_labels().0.len(), history_len + 1);
    let committed = app.doc().clone();
    let selected = app.selection.clone();
    app.undo();
    assert_eq!(
        app.doc(),
        before,
        "undo restores all metadata and exact geometry"
    );
    assert_eq!(app.selection, selected);
    assert_eq!(app.engine.history_labels().0.len(), history_len);
    app.redo();
    assert_eq!(app.doc(), &committed);
    assert_eq!(app.selection, selected);
    physical_geometry(app);
}

#[test]
fn all_seventeen_sdk_identifiers_have_distinct_explicit_app_mappings() {
    let expected = [
        (0, Units::TenthMicrons),
        (1, Units::Inches),
        (2, Units::Feet),
        (3, Units::Millimeters),
        (4, Units::Centimeters),
        (5, Units::Pixels),
        (6, Units::Miles),
        (7, Units::Meters),
        (8, Units::Kilometers),
        (9, Units::Didots),
        (10, Units::Agates),
        (11, Units::Yards),
        (12, Units::Picas),
        (13, Units::Ciceros),
        (14, Units::Points),
        (15, Units::QuarterMillimetersQ),
        (16, Units::QuarterMillimetersH),
    ];
    assert_eq!(Units::ALL.len(), 17);
    for (code, unit) in expected {
        assert_eq!(unit.script_unit(), code);
        assert_eq!(Units::from_script_unit(code), Some(unit));
        assert_eq!(Units::from_id(unit.id()), unit);
        assert_eq!(Units::ALL.iter().filter(|&&u| u == unit).count(), 1);
    }
    assert_eq!(Units::from_script_unit(17), None);
    assert_eq!(Units::from_script_unit(u32::MAX), None);
    assert_eq!(Units::TenthMicrons.mm(), 0.0001);
    assert_eq!(Units::Agates.mm(), 25.4 / 14.0);
    assert_eq!(Units::QuarterMillimetersQ.mm(), 0.25);
    assert_eq!(Units::QuarterMillimetersH.mm(), 0.25);
}

#[test]
fn absent_axes_use_window_fallback_without_materializing_or_using_macro_unit() {
    let mut app = fresh();
    app.units = Units::Feet;
    let before = app.doc().clone();
    assert_eq!(axes(&app), (Some(3), None, None));
    assert_eq!(app.ruler_unit(true), Units::Feet);
    assert_eq!(app.ruler_unit(false), Units::Feet);
    assert_eq!(app.doc(), &before);
    physical_geometry(&app);
}

#[test]
fn setting_one_axis_is_one_reversible_metadata_edit_and_preserves_other_axis() {
    for horizontal in [true, false] {
        let mut app = fresh();
        let before = app.doc().clone();
        let count = app.engine.history_labels().0.len();
        let settings = app.settings.units.clone();
        app.set_ruler_unit(horizontal, Units::Inches);
        assert_eq!(
            axes(&app),
            if horizontal {
                (Some(3), Some(1), None)
            } else {
                (Some(3), None, Some(1))
            }
        );
        assert_eq!(app.units, Units::Millimeters);
        assert_eq!(app.settings.units, settings);
        assert_eq!(app.ruler_unit(horizontal), Units::Inches);
        assert_eq!(app.ruler_unit(!horizontal), Units::Millimeters);
        physical_geometry(&app);
        reversible(&mut app, &before, count);
    }
}

#[test]
fn common_selector_changes_both_axes_once_without_changing_automation_or_geometry() {
    let mut app = fresh();
    vg(
        &mut app,
        "ActiveDocument.Rulers.HUnits=cdrInch; ActiveDocument.Rulers.VUnits=cdrCentimeter;",
    );
    let before = app.doc().clone();
    let count = app.engine.history_labels().0.len();
    app.set_display_units(Units::QuarterMillimetersH);
    assert_eq!(axes(&app), (Some(3), Some(16), Some(16)));
    assert_eq!(app.units, Units::QuarterMillimetersH);
    assert_eq!(app.ruler_unit(true), Units::QuarterMillimetersH);
    assert_eq!(app.ruler_unit(false), Units::QuarterMillimetersH);
    reversible(&mut app, &before, count);
    app.undo();
    assert_eq!(app.units, Units::Millimeters);
    assert_eq!(app.ruler_unit(true), Units::Inches);
    assert_eq!(app.ruler_unit(false), Units::Centimeters);
    app.redo();
    assert_eq!(app.units, Units::QuarterMillimetersH);
}

#[test]
fn common_selector_undo_from_absent_axes_restores_visible_legacy_fallback() {
    let mut app = fresh();
    let before = app.doc().clone();
    assert_eq!(app.ruler_unit(true), Units::Millimeters);
    app.set_display_units(Units::Inches);
    assert_eq!(axes(&app), (Some(3), Some(1), Some(1)));
    app.undo();
    assert_eq!(
        app.doc(),
        &before,
        "do not normalize absent axes into explicit millimeters"
    );
    assert_eq!(app.ruler_unit(true), Units::Millimeters);
    assert_eq!(app.ruler_unit(false), Units::Millimeters);
    assert_eq!(app.units, Units::Millimeters);
    vg(&mut app, "if(ActiveDocument.Rulers.HUnits!==cdrMillimeter || ActiveDocument.Rulers.VUnits!==cdrMillimeter || ActiveDocument.Unit!==cdrMillimeter) throw Error('undo fallback');");
    app.redo();
    assert_eq!(app.ruler_unit(true), Units::Inches);
    assert_eq!(app.ruler_unit(false), Units::Inches);
    assert_eq!(app.units, Units::Inches);
    vg(&mut app, "if(ActiveDocument.Rulers.HUnits!==cdrInch || ActiveDocument.Rulers.VUnits!==cdrInch || ActiveDocument.Unit!==cdrMillimeter) throw Error('redo fallback');");
    physical_geometry(&app);
}

#[test]
fn repeated_axis_and_common_setters_preserve_revision_and_held_redo() {
    let mut app = fresh();
    app.set_display_units(Units::Centimeters);
    app.set_ruler_unit(false, Units::Inches);
    app.undo();
    let before = app.doc().clone();
    let history = app.engine.history_labels();
    let edit_version = app.engine.revision();
    let step = app.engine.last_step_id();
    let selected = app.selection.clone();
    app.set_ruler_unit(true, Units::Centimeters);
    app.set_ruler_unit(false, Units::Centimeters);
    app.set_display_units(Units::Centimeters);
    assert_eq!(app.doc(), &before);
    assert_eq!(app.engine.history_labels(), history);
    assert_eq!(app.engine.revision(), edit_version);
    assert_eq!(app.engine.last_step_id(), step);
    assert_eq!(app.selection, selected);
    app.redo();
    assert_eq!(axes(&app), (Some(3), Some(4), Some(1)));
}

#[test]
fn public_macro_rulers_persist_across_calls_and_leave_macro_lengths_in_millimeters() {
    let mut app = fresh();
    let before = app.doc().clone();
    let count = app.engine.history_labels().0.len();
    vg(&mut app, "var r=ActiveDocument.Rulers; r.HUnits=cdrInch; r.VUnits=cdrCentimeter; if(r.HUnits!==cdrInch || r.VUnits!==cdrCentimeter || ActiveDocument.Unit!==cdrMillimeter) throw Error('independence');");
    assert_eq!(axes(&app), (Some(3), Some(1), Some(4)));
    reversible(&mut app, &before, count);
    vg(&mut app, "if(ActiveDocument.Rulers.HUnits!==cdrInch || ActiveDocument.Rulers.VUnits!==cdrCentimeter || ActiveDocument.Unit!==cdrMillimeter) throw Error('persistence'); ActiveLayer.CreateRectangle2(0,0,10,10).Name='Ten millimeters';");
    let shape = app
        .doc()
        .all_layers()
        .flat_map(|l| &l.shapes)
        .find(|s| s.name.as_deref() == Some("Ten millimeters"))
        .unwrap();
    let ShapeKind::Rect { rect, .. } = &shape.kind else {
        panic!("rectangle expected")
    };
    assert_eq!(*rect, Rect::new(0.0, 0.0, 10.0, 10.0));
    physical_geometry(&app);
}

#[test]
fn changing_macro_unit_does_not_change_explicit_or_absent_display_axes() {
    for explicit in [false, true] {
        let mut app = fresh();
        app.units = Units::Feet;
        if explicit {
            vg(
                &mut app,
                "ActiveDocument.Rulers.HUnits=cdrInch; ActiveDocument.Rulers.VUnits=cdrCentimeter;",
            );
        }
        let before = app.doc().clone();
        let count = app.engine.history_labels().0.len();
        vg(&mut app, "ActiveDocument.Unit=cdrUnitQ;");
        assert_eq!(
            axes(&app),
            if explicit {
                (Some(15), Some(1), Some(4))
            } else {
                (Some(15), None, None)
            }
        );
        assert_eq!(
            app.ruler_unit(true),
            if explicit { Units::Inches } else { Units::Feet }
        );
        assert_eq!(
            app.ruler_unit(false),
            if explicit {
                Units::Centimeters
            } else {
                Units::Feet
            }
        );
        reversible(&mut app, &before, count);
    }
}

#[test]
fn invalid_public_axis_setters_leave_document_history_selection_and_redo_unchanged() {
    for axis in ["HUnits", "VUnits"] {
        for value in [
            "null",
            "undefined",
            "-1",
            "17",
            "4294967295",
            "1.5",
            "NaN",
            "Infinity",
            "'3'",
            "{}",
        ] {
            let mut app = fresh();
            app.set_display_units(Units::Centimeters);
            app.set_ruler_unit(false, Units::Inches);
            app.undo();
            let before = app.doc().clone();
            let history = app.engine.history_labels();
            let edit_version = app.engine.revision();
            let step = app.engine.last_step_id();
            let selected = app.selection.clone();
            let source = format!("ActiveDocument.Rulers.{axis}={value};");
            assert!(
                scripting::run_vgcore(&mut app, &source, LABEL).is_err(),
                "{source}"
            );
            assert_eq!(app.doc(), &before, "{source}");
            assert_eq!(app.engine.history_labels(), history, "{source}");
            assert_eq!(app.engine.revision(), edit_version, "{source}");
            assert_eq!(app.engine.last_step_id(), step, "{source}");
            assert_eq!(app.selection, selected, "{source}");
            app.redo();
            assert_eq!(axes(&app), (Some(3), Some(4), Some(1)));
        }
    }
}

#[test]
fn unknown_stored_axis_is_preserved_but_public_host_rejects_reading_it() {
    for unknown in [17, u32::MAX] {
        let mut app = fresh();
        let mut doc = app.doc().clone();
        doc.metadata.rulers.horizontal_unit = Some(unknown);
        doc.metadata.rulers.vertical_unit = Some(4);
        app.open_document(doc, None);
        app.units = Units::Feet;
        let before = app.doc().clone();
        let history = app.engine.history_labels();
        assert_eq!(
            app.ruler_unit(true),
            Units::Feet,
            "safe UI fallback does not rewrite metadata"
        );
        assert_eq!(app.ruler_unit(false), Units::Centimeters);
        assert!(
            scripting::run_vgcore(&mut app, "var u=ActiveDocument.Rulers.HUnits;", LABEL).is_err()
        );
        assert_eq!(app.doc(), &before);
        assert_eq!(app.engine.history_labels(), history);
        let json = app.doc().to_json().unwrap();
        let restored = Document::from_json(&json).unwrap();
        assert_eq!(restored.metadata.rulers.horizontal_unit, Some(unknown));
        assert_eq!(restored.metadata.rulers.vertical_unit, Some(4));
        physical_geometry(&app);
    }
}

#[test]
fn native_document_json_preserves_null_zero_unknown_and_independent_axes() {
    for (script, horizontal, vertical) in [
        (None, None, None),
        (Some(0), Some(16), Some(0)),
        (Some(3), Some(1), Some(4)),
        (Some(u32::MAX), Some(17), Some(u32::MAX)),
    ] {
        let mut app = fresh();
        let mut doc = app.doc().clone();
        doc.metadata.script_unit = script;
        doc.metadata.rulers.horizontal_unit = horizontal;
        doc.metadata.rulers.vertical_unit = vertical;
        let restored = Document::from_json(&doc.to_json().unwrap()).unwrap();
        assert_eq!(restored, doc);
        app.open_document(restored, None);
        assert_eq!(axes(&app), (script, horizontal, vertical));
        physical_geometry(&app);
    }
}

#[test]
fn two_document_tabs_keep_distinct_axis_preferences_and_window_fallbacks() {
    let mut app = fresh();
    app.units = Units::Feet;
    app.set_ruler_unit(true, Units::Inches);
    let a = app.active_doc;
    let first = app.doc().clone();
    let mut second = Document::new("Second units", Size::new(100.0, 70.0));
    second.metadata.script_unit = Some(15);
    second.metadata.rulers.vertical_unit = Some(16);
    app.open_document(second, None);
    app.units = Units::Centimeters;
    let b = app.active_doc;
    assert_eq!(axes(&app), (Some(15), None, Some(16)));
    assert_eq!(app.ruler_unit(true), Units::Centimeters);
    assert_eq!(app.ruler_unit(false), Units::QuarterMillimetersH);
    app.switch_document(a);
    assert_eq!(app.doc(), &first);
    assert_eq!(app.units, Units::Feet);
    assert_eq!(app.ruler_unit(true), Units::Inches);
    assert_eq!(app.ruler_unit(false), Units::Feet);
    vg(&mut app, "if(ActiveDocument.Rulers.HUnits!==cdrInch || ActiveDocument.Rulers.VUnits!==cdrFoot || ActiveDocument.Unit!==cdrMillimeter) throw Error('first tab');");
    physical_geometry(&app);
    app.switch_document(b);
    assert_eq!(axes(&app), (Some(15), None, Some(16)));
    assert_eq!(app.units, Units::Centimeters);
    vg(&mut app, "if(ActiveDocument.Rulers.HUnits!==cdrCentimeter || ActiveDocument.Rulers.VUnits!==cdrUnitH || ActiveDocument.Unit!==cdrUnitQ) throw Error('second tab');");
}

#[test]
fn legacy_fallback_and_common_selector_history_are_isolated_between_documents() {
    let mut app = fresh();
    let first = app.active_doc;
    app.set_display_units(Units::Inches);
    app.open_document(
        Document::new("Second legacy fallback", Size::new(100.0, 70.0)),
        None,
    );
    let second = app.active_doc;
    app.units = Units::Feet;
    app.set_display_units(Units::Centimeters);
    app.undo();
    assert_eq!(axes(&app), (None, None, None));
    assert_eq!(app.units, Units::Feet);
    assert_eq!(app.ruler_unit(true), Units::Feet);
    assert_eq!(app.ruler_unit(false), Units::Feet);
    app.switch_document(first);
    assert_eq!(app.units, Units::Inches);
    app.undo();
    assert_eq!(axes(&app), (Some(3), None, None));
    assert_eq!(app.units, Units::Millimeters);
    assert_eq!(app.ruler_unit(true), Units::Millimeters);
    assert_eq!(app.ruler_unit(false), Units::Millimeters);
    vg(&mut app,"if(ActiveDocument.Rulers.HUnits!==cdrMillimeter || ActiveDocument.Rulers.VUnits!==cdrMillimeter) throw Error('first fallback');");
    app.redo();
    assert_eq!(app.units, Units::Inches);
    app.switch_document(second);
    assert_eq!(app.units, Units::Feet);
    assert_eq!(app.ruler_unit(true), Units::Feet);
    vg(&mut app,"if(ActiveDocument.Rulers.HUnits!==cdrFoot || ActiveDocument.Rulers.VUnits!==cdrFoot) throw Error('second fallback');");
    app.redo();
    assert_eq!(app.units, Units::Centimeters);
    assert_eq!(axes(&app), (None, Some(4), Some(4)));
}

#[derive(Debug, Clone, PartialEq)]
struct PaintedText {
    text: String,
    x: f32,
    y: f32,
    angle: f32,
    width: f32,
}

fn painted_text(shape: &egui::Shape, texts: &mut Vec<PaintedText>) {
    match shape {
        egui::Shape::Text(t) => texts.push(PaintedText {
            text: t.galley.text().to_owned(),
            x: t.pos.x,
            y: t.pos.y,
            angle: t.angle,
            width: t.galley.size().x,
        }),
        egui::Shape::Vec(shapes) => {
            for shape in shapes {
                painted_text(shape, texts);
            }
        }
        _ => {}
    }
}

fn ruler_paint(app: &mut App) -> Vec<PaintedText> {
    let ctx = crate::theme::ui_context();
    let full = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
    app.view.zoom = 10.0;
    app.view.origin = egui::pos2(100.0, 500.0);
    app::set_drawing_scale(1.0);
    let mut texts = Vec::new();
    let mut capture = ruler_capture::Capture::default();
    for frame in 0..2 {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(full),
                ..Default::default()
            },
            |ui| crate::ui::rulers::rulers(app, ui, full),
        );
        capture.frame(&ctx, &out, frame == 1);
        texts.clear();
        for shape in &out.shapes {
            painted_text(&shape.shape, &mut texts);
        }
        out.textures_delta.clear();
    }
    assert!(!texts.is_empty(), "real egui painting must emit text");
    texts
}

fn painted_one_at(texts: &[PaintedText], horizontal: bool, position: f32) {
    assert!(
        texts.iter().any(|t| {
            let same_axis = if horizontal {
                t.angle.abs() < 0.001
            } else {
                (t.angle + std::f32::consts::FRAC_PI_2).abs() < 0.001
            };
            let center = if horizontal {
                t.x + t.width / 2.0
            } else {
                t.y - t.width / 2.0
            };
            t.text.parse::<f64>() == Ok(1.0) && same_axis && (center - position).abs() <= 1.0
        }),
        "missing painted 1 at axis={horizontal}, pixel={position}: {texts:?}"
    );
}

#[test]
fn real_egui_ruler_paint_uses_each_axis_and_ignores_automation_unit() {
    let mut app = fresh();
    vg(
        &mut app,
        "ActiveDocument.Rulers.HUnits=cdrInch; ActiveDocument.Rulers.VUnits=cdrCentimeter;",
    );
    let inch_cm = ruler_paint(&mut app);
    // Independent screen positions: origin=(100,500), 10px/mm. One inch
    // is 25.4mm, one centimeter is 10mm. No production tick helper is used.
    painted_one_at(&inch_cm, true, 354.5);
    painted_one_at(&inch_cm, false, 400.5);
    vg(&mut app, "ActiveDocument.Unit=cdrUnitH;");
    assert_eq!(ruler_paint(&mut app), inch_cm);
    vg(
        &mut app,
        "ActiveDocument.Rulers.HUnits=cdrCentimeter; ActiveDocument.Rulers.VUnits=cdrInch;",
    );
    let cm_inch = ruler_paint(&mut app);
    painted_one_at(&cm_inch, true, 200.5);
    painted_one_at(&cm_inch, false, 246.5);
    assert_ne!(
        cm_inch, inch_cm,
        "swapping the axes must change actual painted marks"
    );
    app.undo();
    assert_eq!(ruler_paint(&mut app), inch_cm);
    app.redo();
    assert_eq!(ruler_paint(&mut app), cm_inch);
    physical_geometry(&app);
}
