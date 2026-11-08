//! Independent geometry and transaction oracles for two distinct script models.
//! SDK27.2 supports valid signatures/radii; invalid-input guards, normalization
//! and the existing lower-left dimensional anchor are explicitly owned policy.
use crate::{app::App, scripting};
use tracedraw_core::{
    Document, LayerId, PageId, Shape, ShapeId, ShapeKind,
    geometry::{Affine, PathEl, Point, Rect, Shape as _},
};

const LABEL: &str = "Rectangle contract";

#[derive(Clone, Debug, PartialEq)]
struct Snapshot {
    document: Document,
    history: (Vec<&'static str>, Vec<&'static str>),
    selection: Vec<ShapeId>,
    page: PageId,
    layer: Option<LayerId>,
    edit_version: u64,
    step: Option<u64>,
    dirty: bool,
}

fn snapshot(app: &App) -> Snapshot {
    Snapshot {
        document: app.doc().clone(),
        history: app.engine.history_labels(),
        selection: app.selection.clone(),
        page: app.page,
        layer: app.active_layer(),
        edit_version: app.engine.revision(),
        step: app.engine.last_step_id(),
        dirty: app.engine.is_dirty(),
    }
}

fn vg(app: &mut App, source: &str) -> Vec<String> {
    scripting::run_vgcore(app, source, LABEL).unwrap_or_else(|error| panic!("{source}\n{error}"))
}

fn app() -> App {
    let mut app = App::headless();
    scripting::run(
        &mut app,
        "ActiveLayer.CreateRectangle(-30,-20,10,10).Name='Existing sentinel';",
    )
    .unwrap();
    app.selection = vec![named(&app, "Existing sentinel").id];
    app
}

fn named<'a>(app: &'a App, name: &str) -> &'a Shape {
    app.doc()
        .all_layers()
        .flat_map(|layer| &layer.shapes)
        .find(|shape| shape.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing shape {name}"))
}

fn root_count(app: &App) -> usize {
    app.doc().all_layers().map(|layer| layer.shapes.len()).sum()
}

fn rectangle(shape: &Shape) -> (Rect, [f64; 4]) {
    match &shape.kind {
        ShapeKind::Rect {
            rect,
            radius,
            corners,
        } => (
            *rect,
            corners
                .as_ref()
                .map(|corners| corners.radii)
                .unwrap_or([*radius; 4]),
        ),
        _ => panic!("expected rectangle, got {:?}", shape.kind),
    }
}

fn near(actual: f64, expected: f64) {
    assert!(
        actual.is_finite() && (actual - expected).abs() <= 1e-8_f64.max(expected.abs() * 1e-10),
        "{actual} != {expected}",
    );
}

fn geometry(shape: &Shape, expected: Rect, radii: [f64; 4]) {
    let (actual, actual_radii) = rectangle(shape);
    for (value, expected) in [actual.x0, actual.y0, actual.x1, actual.y1]
        .into_iter()
        .zip([expected.x0, expected.y0, expected.x1, expected.y1])
    {
        near(value, expected);
    }
    for (value, expected) in actual_radii.into_iter().zip(radii) {
        near(value, expected);
    }
    assert_eq!(shape.transform, Affine::IDENTITY);
}

fn undo_to(app: &mut App, before: &Snapshot) {
    app.undo();
    let mut expected = before.document.clone();
    expected.set_ids(app.doc().ids().clone());
    assert_eq!(app.doc(), &expected);
    assert_eq!(app.selection, before.selection);
    assert_eq!(app.page, before.page);
    assert_eq!(app.engine.history_labels().0, before.history.0);
}

fn reversible(app: &mut App, before: &Snapshot) {
    assert_eq!(
        app.engine.history_labels().0.len(),
        before.history.0.len() + 1
    );
    assert_eq!(app.engine.undo_label(), Some(LABEL));
    let committed = app.doc().clone();
    let selected = app.selection.clone();
    undo_to(app, before);
    app.redo();
    assert_eq!(app.doc(), &committed);
    assert_eq!(app.selection, selected);
}

#[test]
fn identical_names_preserve_distinct_native_xywh_and_vg_ltrb_contracts() {
    let mut native = app();
    scripting::run(
        &mut native,
        "ActiveLayer.CreateRectangle(10,40,30,20,3).Name='Native';",
    )
    .unwrap();
    geometry(
        named(&native, "Native"),
        Rect::new(10.0, 40.0, 40.0, 60.0),
        [3.0; 4],
    );
    let mut macros = app();
    let before = snapshot(&macros);
    vg(
        &mut macros,
        "ActiveDocument.Unit=cdrMillimeter; ActiveLayer.CreateRectangle(10,40,30,20,0).Name='VG';",
    );
    geometry(
        named(&macros, "VG"),
        Rect::new(10.0, 20.0, 30.0, 40.0),
        [0.0; 4],
    );
    reversible(&mut macros, &before);
}

#[test]
fn vg_sides_and_dimensions_match_under_each_explicit_document_unit() {
    for (unit, factor) in [
        ("cdrMillimeter", 1.0),
        ("cdrCentimeter", 10.0),
        ("cdrInch", 25.4),
        ("cdrPoint", 25.4 / 72.0),
    ] {
        let mut app = app();
        let before = snapshot(&app);
        vg(
            &mut app,
            &format!(
                "ActiveDocument.Unit={unit};
             ActiveLayer.CreateRectangle(1,6,4,2).Name='Sides';
             ActiveLayer.CreateRectangle2(1,2,3,4).Name='Dimensions';"
            ),
        );
        let rect = Rect::new(factor, 2.0 * factor, 4.0 * factor, 6.0 * factor);
        geometry(named(&app, "Sides"), rect, [0.0; 4]);
        // The dimensional lower-left anchor is retained owned policy, not a
        // resolution of the contradictory public SDK prose.
        geometry(named(&app, "Dimensions"), rect, [0.0; 4]);
        reversible(&mut app, &before);
    }
}

#[test]
fn asymmetric_percentage_corners_have_independent_physical_tangent_points() {
    let mut app = app();
    let before = snapshot(&app);
    vg(
        &mut app,
        "ActiveDocument.Unit=cdrMillimeter;
        ActiveLayer.CreateRectangle(10,40,50,20,10,20,30,40).Name='Asymmetric';",
    );
    let shape = named(&app, "Asymmetric");
    geometry(
        shape,
        Rect::new(10.0, 20.0, 50.0, 40.0),
        [1.0, 2.0, 4.0, 3.0],
    );
    let path = shape.local_path();
    let bounds = path.bounding_box();
    for (actual, expected) in [bounds.x0, bounds.y0, bounds.x1, bounds.y1]
        .into_iter()
        .zip([10.0, 20.0, 50.0, 40.0])
    {
        near(actual, expected);
    }
    let endpoints: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|part| match part {
            PathEl::MoveTo(p)
            | PathEl::LineTo(p)
            | PathEl::QuadTo(_, p)
            | PathEl::CurveTo(_, _, p) => Some(*p),
            PathEl::ClosePath => None,
        })
        .collect();
    // Eight tangencies computed directly from four unequal radii, without
    // using Corners.layout or another rectangle constructor as the oracle.
    for expected in [
        Point::new(11.0, 40.0),
        Point::new(10.0, 39.0),
        Point::new(48.0, 40.0),
        Point::new(50.0, 38.0),
        Point::new(14.0, 20.0),
        Point::new(10.0, 24.0),
        Point::new(47.0, 20.0),
        Point::new(50.0, 23.0),
    ] {
        assert!(
            endpoints
                .iter()
                .any(|p| (p.x - expected.x).abs() < 1e-8 && (p.y - expected.y).abs() < 1e-8),
            "missing tangent {expected:?}; {endpoints:?}"
        );
    }
    reversible(&mut app, &before);
}

#[test]
fn absolute_radii_and_rectangle_rect_preserve_all_four_independent_corners() {
    let mut app = app();
    let before = snapshot(&app);
    vg(&mut app, "ActiveDocument.Unit=cdrMillimeter;
        ActiveLayer.CreateRectangle2(10,20,40,20,1,2,3,4).Name='Dimensions';
        ActiveLayer.CreateRectangleRect({x:10,y:20,Width:40,Height:20},1,2,3,4).Name='Rect object';");
    for name in ["Dimensions", "Rect object"] {
        geometry(
            named(&app, name),
            Rect::new(10.0, 20.0, 50.0, 40.0),
            [1.0, 2.0, 4.0, 3.0],
        );
    }
    reversible(&mut app, &before);
}

#[test]
fn each_optional_corner_defaults_independently_to_zero() {
    for constructor in ["CreateRectangle", "CreateRectangle2", "CreateRectangleRect"] {
        for argument in 0..4 {
            let mut values = ["undefined"; 4];
            values[argument] = if constructor == "CreateRectangle" {
                "50"
            } else {
                "2.5"
            };
            let mut app = app();
            let args = match constructor {
                "CreateRectangle" => "10,40,50,20",
                "CreateRectangle2" => "10,20,40,20",
                _ => "{x:10,y:20,Width:40,Height:20}",
            };
            vg(
                &mut app,
                &format!(
                    "ActiveDocument.Unit=cdrMillimeter;
                ActiveLayer.{constructor}({args},{}).Name='Optional';",
                    values.join(",")
                ),
            );
            let mut expected = [0.0; 4];
            expected[[0, 1, 3, 2][argument]] = if constructor == "CreateRectangle" {
                5.0
            } else {
                2.5
            };
            geometry(
                named(&app, "Optional"),
                Rect::new(10.0, 20.0, 50.0, 40.0),
                expected,
            );
        }
    }
}

#[test]
fn max_radius_is_half_the_shorter_side_not_the_largest_assigned_radius() {
    let mut app = app();
    let output = vg(
        &mut app,
        "ActiveDocument.Unit=cdrMillimeter;
        const r=ActiveLayer.CreateRectangle2(10,20,40,20,1,2,3,4); r.Name='Maximum';
        print(r.Rectangle.MaxRadius); ActiveDocument.Unit=cdrInch;
        print(r.Rectangle.MaxRadius); ActiveDocument.Unit=cdrMillimeter;
        r.Rectangle.SetRoundness(100); print(r.Rectangle.MaxRadius,r.Rectangle.CornerUpperLeft);",
    );
    near(output[0].parse().unwrap(), 10.0);
    near(output[1].parse().unwrap(), 10.0 / 25.4);
    assert_eq!(output[2], "10 100");
    geometry(
        named(&app, "Maximum"),
        Rect::new(10.0, 20.0, 50.0, 40.0),
        [10.0; 4],
    );
}

#[test]
fn physical_radii_follow_units_while_roundness_uses_the_documented_half_side() {
    let mut app = app();
    let output=vg(&mut app,"ActiveDocument.Unit=cdrMillimeter;
        const r=ActiveLayer.CreateRectangle2(0,0,40,20); r.Name='Setters';
        r.Rectangle.SetRadius(3.25); print(r.Rectangle.RadiusUpperLeft);
        r.Rectangle.SetRoundness(50); print(r.Rectangle.RadiusUpperLeft,r.Rectangle.CornerLowerRight);
        ActiveDocument.Unit=cdrInch; r.Rectangle.SetRadius(0.125);
        print(r.Rectangle.RadiusUpperLeft);");
    near(output[0].parse().unwrap(), 3.25);
    assert_eq!(output[1], "5 50");
    near(output[2].parse().unwrap(), 0.125);
    geometry(
        named(&app, "Setters"),
        Rect::new(0.0, 0.0, 40.0, 20.0),
        [3.175; 4],
    );
}

#[test]
fn owned_zero_and_inverted_extent_policy_remains_explicit_and_finite() {
    let mut app = app();
    let output = vg(
        &mut app,
        "ActiveDocument.Unit=cdrMillimeter;
        ActiveLayer.CreateRectangle2(50,20,-40,20).Name='Inverted';
        const r=ActiveLayer.CreateRectangle2(10,20,0,20); r.Name='Zero';
        r.Rectangle.SetRoundness(50);
        print(r.Rectangle.MaxRadius,r.Rectangle.CornerUpperLeft,r.Rectangle.RadiusUpperLeft);",
    );
    geometry(
        named(&app, "Inverted"),
        Rect::new(10.0, 20.0, 50.0, 40.0),
        [0.0; 4],
    );
    geometry(
        named(&app, "Zero"),
        Rect::new(10.0, 20.0, 10.0, 40.0),
        [0.0; 4],
    );
    assert_eq!(output[0], "0 0 0");
}

#[test]
fn owned_finite_absolute_radii_above_maximum_keep_the_previous_core_policy() {
    let mut app = app();
    vg(
        &mut app,
        "ActiveDocument.Unit=cdrMillimeter;
        ActiveLayer.CreateRectangle2(0,0,40,20,100,0,0,0).Name='Large asymmetric';
        const r=ActiveLayer.CreateRectangle2(50,0,40,20);r.Name='Large uniform';
        r.Rectangle.SetRadius(100);",
    );
    geometry(
        named(&app, "Large asymmetric"),
        Rect::new(0.0, 0.0, 40.0, 20.0),
        [100.0, 0.0, 0.0, 0.0],
    );
    geometry(
        named(&app, "Large uniform"),
        Rect::new(50.0, 0.0, 90.0, 20.0),
        [100.0; 4],
    );
    // This is a regression guarantee of the owned model, not a claim about
    // the original application's undocumented response outside MaxRadius.
    for name in ["Large asymmetric", "Large uniform"] {
        let path = named(&app, name).local_path();
        let bounds = path.bounding_box();
        assert!(
            [bounds.x0, bounds.y0, bounds.x1, bounds.y1]
                .into_iter()
                .all(f64::is_finite)
        );
        near(bounds.width(), 40.0);
        near(bounds.height(), 20.0);
    }
}

#[test]
fn invalid_vg_constructors_never_add_a_partial_shape_consume_ids_or_erase_redo() {
    let invalid = [
        "ActiveLayer.CreateRectangle();",
        "ActiveLayer.CreateRectangle(NaN,40,50,20);",
        "ActiveLayer.CreateRectangle(10,Infinity,50,20);",
        "ActiveLayer.CreateRectangle(10,40,50,20,-1,0,0,0);",
        "ActiveLayer.CreateRectangle(10,40,50,20,0,101,0,0);",
        "ActiveLayer.CreateRectangle(10,40,50,20,10,20,NaN,40);",
        "ActiveLayer.CreateRectangle(10,40,50,20,10,20,30,Infinity);",
        "ActiveLayer.CreateRectangle2(0,0);",
        "ActiveLayer.CreateRectangle2(10,20,40,20,NaN);",
        "ActiveLayer.CreateRectangle2(10,20,40,20,1,-2,3,4);",
        "ActiveLayer.CreateRectangle2(10,20,40,20,1,2,Infinity,4);",
        "ActiveLayer.CreateRectangle2(10,20,40,20,1,2,3,NaN);",
        "ActiveLayer.CreateRectangle2(1e308,0,1e308,20);",
        "ActiveDocument.Unit=cdrInch;ActiveLayer.CreateRectangle2(1e308,0,0,1);",
        "ActiveLayer.CreateRectangleRect({x:0,y:0,Width:40,Height:20},1,2,3,NaN);",
        "ActiveLayer.CreateRectangleRect({x:0,y:0,Width:NaN,Height:20});",
    ];
    for source in invalid {
        let (unit, source) = match source.strip_prefix("ActiveDocument.Unit=cdrInch;") {
            Some(constructor) => ("cdrInch", constructor),
            None => ("cdrMillimeter", source),
        };
        let mut app = app();
        // Unit is now persistent and a legitimate preceding User command.
        // Calibrate it before the rejection snapshot and held redo.
        vg(&mut app, &format!("ActiveDocument.Unit={unit};"));
        scripting::run(
            &mut app,
            "ActiveLayer.CreateRectangle(0,0,2,3).Name='Held redo';",
        )
        .unwrap();
        let held = app.doc().clone();
        app.undo();
        let before = snapshot(&app);
        let result = scripting::run_vgcore(
            &mut app,
            &format!("ActiveDocument.Unit={unit};{source}"),
            LABEL,
        );
        assert!(
            result.is_err(),
            "invalid constructor unexpectedly succeeded: {source}"
        );
        assert_eq!(snapshot(&app), before, "{source}");
        app.redo();
        assert_eq!(app.doc(), &held, "original redo was lost: {source}");
    }
}

#[test]
fn invalid_constructor_keeps_only_valid_earlier_user_commands_in_one_transaction() {
    let mut app = app();
    let before = snapshot(&app);
    let error = scripting::run_vgcore(
        &mut app,
        "ActiveDocument.Unit=cdrMillimeter;
        ActiveLayer.CreateRectangle2(10,20,40,20,1,2,3,4).Name='Kept before error';
        ActiveLayer.CreateRectangle2(60,20,40,20,1,2,3,NaN);",
        LABEL,
    )
    .unwrap_err();
    assert!(!error.is_empty());
    assert_eq!(root_count(&app), root_count_from(&before.document) + 1);
    assert_eq!(
        app.doc().ids().peek_next(),
        before.document.ids().peek_next() + 1
    );
    geometry(
        named(&app, "Kept before error"),
        Rect::new(10.0, 20.0, 50.0, 40.0),
        [1.0, 2.0, 4.0, 3.0],
    );
    reversible(&mut app, &before);
}

fn root_count_from(doc: &Document) -> usize {
    doc.all_layers().map(|layer| layer.shapes.len()).sum()
}

#[test]
fn invalid_corner_setters_and_typed_host_bypasses_preserve_the_existing_rectangle() {
    for operation in [
        "r.Rectangle.SetRoundness(101);",
        "r.Rectangle.SetRadius(NaN);",
        "r.Rectangle.RadiusLowerLeft=-1;",
        "r.Rectangle.RadiusUpperRight=Infinity;",
        "r.Rectangle.CornerUpperLeft=NaN;",
        "r.Rectangle.CornerLowerRight=-1;",
        "const k=__call('vg_kind',{id:r.id});k.radius=-1;__call('vg_set_kind',{id:r.id,kind:k});",
        "__call('vg_add_shape',{layer:ActiveLayer.id,kind:{type:'rect',rect:{x0:0,y0:0,x1:10,y1:10},radius:-1}});",
    ] {
        let mut app = app();
        vg(&mut app, "ActiveDocument.Unit=cdrMillimeter;");
        let before = snapshot(&app);
        let source = format!(
            "ActiveDocument.Unit=cdrMillimeter;const r=ActivePage.Shapes.Item(1);{operation}"
        );
        let result = scripting::run_vgcore(&mut app, &source, LABEL);
        assert!(result.is_err(), "{operation}");
        assert_eq!(snapshot(&app), before, "{operation}");
    }
}

#[test]
fn untrusted_native_script_retains_its_existing_global_rollback_contract() {
    let mut app = app();
    let before = snapshot(&app);
    let result = scripting::run_untrusted(
        &mut app,
        "ActiveLayer.CreateRectangle(10,20,30,40);throw new Error('Owned rollback control');",
        "Untrusted rectangle",
    );
    assert!(result.is_err());
    assert_eq!(snapshot(&app), before);
}

#[test]
fn macro_unit_persists_between_calls_including_zero_without_changing_native_mm() {
    let mut app = app();
    vg(&mut app, "ActiveDocument.Unit=cdrMillimeter;");
    assert_eq!(app.doc().metadata.script_unit, Some(3));
    vg(
        &mut app,
        "ActiveLayer.CreateRectangle2(1,2,3,4).Name='Persistent mm';",
    );
    geometry(
        named(&app, "Persistent mm"),
        Rect::new(1.0, 2.0, 4.0, 6.0),
        [0.0; 4],
    );
    vg(&mut app, "ActiveDocument.Unit=0;");
    assert_eq!(app.doc().metadata.script_unit, Some(0));
    let output = vg(
        &mut app,
        "print(ActiveDocument.Unit);
        ActiveLayer.CreateRectangle2(10000,20000,30000,40000).Name='Persistent zero';",
    );
    assert_eq!(output[0], "0");
    geometry(
        named(&app, "Persistent zero"),
        Rect::new(1.0, 2.0, 4.0, 6.0),
        [0.0; 4],
    );
    scripting::run(
        &mut app,
        "ActiveLayer.CreateRectangle(1,2,3,4,1).Name='Native still mm';",
    )
    .unwrap();
    geometry(
        named(&app, "Native still mm"),
        Rect::new(1.0, 2.0, 4.0, 6.0),
        [1.0; 4],
    );
    assert_eq!(app.doc().metadata.script_unit, Some(0));
}

#[test]
fn macro_unit_is_document_local_and_absence_keeps_only_the_historical_owned_default() {
    let mut a = App::headless();
    let mut b = App::headless();
    vg(&mut a, "ActiveDocument.Unit=cdrMillimeter;");
    assert_eq!(vg(&mut a, "print(ActiveDocument.Unit);"), vec!["3"]);
    assert_eq!(vg(&mut b, "print(ActiveDocument.Unit);"), vec!["1"]);
    assert_eq!(b.doc().metadata.script_unit, None);
    vg(&mut b, "ActiveDocument.Unit=cdrCentimeter;");
    vg(
        &mut a,
        "ActiveLayer.CreateRectangle2(1,2,3,4).Name='Document A';",
    );
    vg(
        &mut b,
        "ActiveLayer.CreateRectangle2(1,2,3,4).Name='Document B';",
    );
    geometry(
        named(&a, "Document A"),
        Rect::new(1.0, 2.0, 4.0, 6.0),
        [0.0; 4],
    );
    geometry(
        named(&b, "Document B"),
        Rect::new(10.0, 20.0, 40.0, 60.0),
        [0.0; 4],
    );
    assert_eq!(a.doc().metadata.script_unit, Some(3));
    assert_eq!(b.doc().metadata.script_unit, Some(4));
}

#[test]
fn unit_change_has_one_undo_and_same_value_does_not_destroy_a_held_redo() {
    let mut app = app();
    let before = snapshot(&app);
    vg(&mut app, "ActiveDocument.Unit=cdrMillimeter;");
    assert_eq!(app.doc().metadata.script_unit, Some(3));
    reversible(&mut app, &before);
    vg(&mut app, "ActiveDocument.Unit=cdrCentimeter;");
    let held = app.doc().clone();
    app.undo();
    assert_eq!(app.doc().metadata.script_unit, Some(3));
    let before = snapshot(&app);
    vg(&mut app, "ActiveDocument.Unit=cdrMillimeter;");
    assert_eq!(
        snapshot(&app),
        before,
        "setting the same stored unit must be a genuine no-op"
    );
    app.redo();
    assert_eq!(app.doc(), &held);
}

#[test]
fn save_restore_settings_restores_absence_and_nested_stored_values_exactly() {
    let mut app = App::headless();
    let original = app.doc().clone();
    let output = vg(
        &mut app,
        "const d=ActiveDocument;d.SaveSettings();
        d.Unit=cdrMillimeter;d.RestoreSettings();print(d.Unit);",
    );
    assert_eq!(output, vec!["1"]);
    assert_eq!(app.doc().metadata.script_unit, None);
    assert_eq!(app.doc(), &original);
    let json: serde_json::Value = serde_json::from_str(&app.doc().to_json().unwrap()).unwrap();
    assert!(json["metadata"].get("script_unit").is_none());
    assert_eq!(vg(&mut app, "print(ActiveDocument.Unit);"), vec!["1"]);
    vg(&mut app, "ActiveDocument.Unit=cdrMillimeter;");
    let output = vg(
        &mut app,
        "const d=ActiveDocument;d.SaveSettings();d.Unit=cdrInch;
        d.SaveSettings();d.Unit=cdrCentimeter;d.RestoreSettings();print(d.Unit);
        d.RestoreSettings();print(d.Unit);",
    );
    assert_eq!(output, vec!["1", "3"]);
    assert_eq!(app.doc().metadata.script_unit, Some(3));
}

#[test]
fn saved_tdraw_reopens_with_its_macro_unit_and_unchanged_millimetre_geometry() {
    let mut source = app();
    vg(
        &mut source,
        "ActiveDocument.Unit=cdrCentimeter;
        ActiveLayer.CreateRectangle2(1,2,3,4,0.1,0.2,0.3,0.4).Name='Saved';",
    );
    let before = snapshot(&source);
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../evidencias/craft-corel-nativo/scripts-rectangulos/pruebas/generated")
        .join(format!(
            "unit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("owned-script-unit.tdraw");
    let bytes = source.doc().to_json().unwrap().into_bytes();
    std::fs::write(&path, &bytes).unwrap();
    let loaded = Document::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(&loaded, &before.document);
    let mut target = App::headless();
    target.page = loaded.pages[0].id;
    target.engine.replace(loaded);
    assert_eq!(vg(&mut target, "print(ActiveDocument.Unit);"), vec!["4"]);
    geometry(
        named(&target, "Saved"),
        Rect::new(10.0, 20.0, 40.0, 60.0),
        [1.0, 2.0, 4.0, 3.0],
    );
    vg(
        &mut target,
        "ActiveLayer.CreateRectangle2(2,3,1,2).Name='After reopen';",
    );
    geometry(
        named(&target, "After reopen"),
        Rect::new(20.0, 30.0, 30.0, 50.0),
        [0.0; 4],
    );
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert_eq!(snapshot(&source), before);
}

#[test]
fn invalid_unit_setters_preserve_document_ids_selection_history_and_redo() {
    for value in [
        "17",
        "-1",
        "1.5",
        "NaN",
        "Infinity",
        "null",
        "undefined",
        "{}",
    ] {
        let mut app = app();
        vg(&mut app, "ActiveDocument.Unit=0;");
        scripting::run(
            &mut app,
            "ActiveLayer.CreateRectangle(0,0,1,1).Name='Held unit redo';",
        )
        .unwrap();
        let held = app.doc().clone();
        app.undo();
        let before = snapshot(&app);
        let result = scripting::run_vgcore(
            &mut app,
            &format!(
                "ActiveDocument.Unit={value};ActiveLayer.CreateRectangle2(1,2,3,4).Name='Must not run';"
            ),
            LABEL,
        );
        assert!(result.is_err(), "{value}");
        assert_eq!(snapshot(&app), before, "{value}");
        app.redo();
        assert_eq!(app.doc(), &held, "{value}");
    }
}

#[test]
fn invalid_persisted_macro_unit_rejects_vg_before_source_but_leaves_native_available() {
    let mut app = app();
    let mut doc = app.doc().clone();
    doc.metadata.script_unit = Some(17);
    app.engine.replace(doc);
    let before = snapshot(&app);
    let result = scripting::run_vgcore(
        &mut app,
        "ActiveLayer.CreateRectangle2(1,2,3,4).Name='Must not evaluate';",
        LABEL,
    );
    assert!(result.is_err());
    assert_eq!(snapshot(&app), before);
    scripting::run(
        &mut app,
        "ActiveLayer.CreateRectangle(1,2,3,4).Name='Native available';",
    )
    .unwrap();
    geometry(
        named(&app, "Native available"),
        Rect::new(1.0, 2.0, 4.0, 6.0),
        [0.0; 4],
    );
    assert_eq!(app.doc().metadata.script_unit, Some(17));
}

#[test]
fn native_selection_keeps_mm_geometry_without_replacing_the_target_macro_unit() {
    let mut source = app();
    vg(
        &mut source,
        "ActiveDocument.Unit=cdrCentimeter;
        ActiveLayer.CreateRectangle2(1,2,3,4,0.1,0.2,0.3,0.4).Name='Transferred geometry';",
    );
    source.selection = vec![named(&source, "Transferred geometry").id];
    let source_before = snapshot(&source);
    let mut wire = None;
    assert!(
        source
            .try_copy_with_publisher(|text| {
                wire = Some(text.to_owned());
                Ok(())
            })
            .unwrap()
    );
    let wire = wire.unwrap();
    let packet = crate::clipboard_wire::decode_svg(&wire).unwrap().unwrap();
    assert_eq!(
        packet.document.metadata.script_unit, None,
        "A compact selection carries physical geometry, not the source scripting unit setting"
    );
    let mut target = app();
    vg(&mut target, "ActiveDocument.Unit=cdrMillimeter;");
    let before = snapshot(&target);
    target.try_paste_svg_text(&wire).unwrap();
    assert_eq!(target.doc().metadata.script_unit, Some(3));
    geometry(
        named(&target, "Transferred geometry"),
        Rect::new(10.0, 20.0, 40.0, 60.0),
        [1.0, 2.0, 4.0, 3.0],
    );
    let committed = target.doc().clone();
    target.undo();
    let mut expected = before.document.clone();
    expected.set_ids(target.doc().ids().clone());
    assert_eq!(target.doc(), &expected);
    assert_eq!(target.selection, before.selection);
    target.redo();
    assert_eq!(target.doc(), &committed);
    assert_eq!(snapshot(&source), source_before);
}
