//! Cross-application clipboard refutation through published bytes only.

use crate::app::{App, color_import_tests as fixture};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use tracedraw_core::geometry::Point;
use tracedraw_core::{Command, Document, ShapeId, ShapeKind};

#[derive(Debug, Clone, PartialEq)]
struct Snapshot {
    document: Document,
    instance: u64,
    revision: u64,
    history: (Vec<&'static str>, Vec<&'static str>),
    step: Option<u64>,
    dirty: bool,
    selection: Vec<ShapeId>,
    page: tracedraw_core::PageId,
    layer: Option<tracedraw_core::LayerId>,
    fonts: Vec<String>,
    preferences: serde_json::Value,
    text_edit: String,
    text_clip: String,
}

fn snapshot(app: &App) -> Snapshot {
    Snapshot {
        document: app.doc().clone(),
        instance: app.engine.instance(),
        revision: app.engine.revision(),
        history: app.engine.history_labels(),
        step: app.engine.last_step_id(),
        dirty: app.engine.is_dirty(),
        selection: app.selection.clone(),
        page: app.page,
        layer: app.active_layer(),
        fonts: app.missing_fonts.clone(),
        preferences: serde_json::to_value(&app.settings.color).unwrap(),
        text_edit: format!("{:?}", app.text_edit),
        text_clip: format!("{:?}", app.text_clip),
    }
}

fn configured(profile: Option<&tracedraw_core::DocumentProfile>) -> App {
    let mut document = fixture::document(profile);
    fixture::add_picture(&mut document, profile);
    fixture::add_vector(&mut document);
    document.pages[0].layers[0].shapes[0].name = Some("Owned wire picture".into());
    document.pages[0].layers[0].shapes[1].name = Some("Owned wire vector".into());
    let page = document.pages[0].id;
    let mut app = App::headless();
    app.engine.replace(document);
    app.page = page;
    app.selection = app.doc().pages[0].layers[0]
        .shapes
        .iter()
        .map(|shape| shape.id)
        .collect();
    app.settings.color.intent = "Saturation".into();
    app.settings.color.black_point_compensation = false;
    app.settings.color.preserve_pure_black = false;
    app.settings.color.cmyk_profile_path.clear();
    app.settings.color.cmyk_profile = "Generic CMYK (open)".into();
    app
}

fn destination(policy: u8) -> App {
    let mut app = configured(Some(&fixture::profile(true)));
    app.selection = vec![app.doc().pages[0].layers[0].shapes[1].id];
    app.settings.color.import_cmyk = policy;
    app.engine
        .run(&Command::SetTitle {
            title: "Edited independent paste target".into(),
        })
        .unwrap();
    app
}

fn publish(app: &mut App) -> String {
    let mut text = None;
    assert!(
        app.try_copy_with_publisher(|published| {
            assert!(text.is_none(), "publish exactly once");
            text = Some(published.to_owned());
            Ok(())
        })
        .unwrap()
    );
    let text = text.expect("the injected publisher must actually receive bytes");
    assert!(crate::clipboard_wire::looks_native(&text));
    text
}

fn json_bytes(text: &str) -> Vec<u8> {
    let document = roxmltree::Document::parse(text).unwrap();
    let node = document
        .descendants()
        .find(|node| {
            node.tag_name().namespace() == Some(crate::clipboard_wire::NAMESPACE)
                && node.tag_name().name() == "selection"
        })
        .unwrap();
    STANDARD.decode(node.text().unwrap().trim()).unwrap()
}

fn replace_json_bytes(text: &str, bytes: &[u8]) -> String {
    let document = roxmltree::Document::parse(text).unwrap();
    let node = document
        .descendants()
        .find(|node| {
            node.tag_name().namespace() == Some(crate::clipboard_wire::NAMESPACE)
                && node.tag_name().name() == "selection"
        })
        .unwrap();
    let range = node.first_child().unwrap().range();
    format!(
        "{}{}{}",
        &text[..range.start],
        STANDARD.encode(bytes),
        &text[range.end..]
    )
}

fn mutate_json(text: &str, mutate: impl FnOnce(&mut serde_json::Value)) -> String {
    let mut value: serde_json::Value = serde_json::from_slice(&json_bytes(text)).unwrap();
    mutate(&mut value);
    replace_json_bytes(text, &serde_json::to_vec(&value).unwrap())
}

fn after_undo(app: &App, expected: &Snapshot) {
    let mut before = expected.document.clone();
    before.set_ids(app.doc().ids().clone());
    assert_eq!(
        app.doc(),
        &before,
        "one undo restores all document resources and paints"
    );
    assert_eq!(app.selection, expected.selection);
    assert_eq!(app.engine.history_labels().0, expected.history.0);
    assert_eq!(app.engine.history_labels().1.last(), Some(&"Paste"));
}

fn directory(name: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../evidencias/craft-corel-nativo/portapapeles/refutacion")
        .join(format!(
            "{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn published_bytes_preserve_inks_alpha_and_profiles_between_independent_apps_for_all_policies() {
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    let mut source = configured(Some(&a));
    let source_before = snapshot(&source);
    let text = publish(&mut source);
    assert_eq!(snapshot(&source), source_before);
    let native = crate::clipboard_wire::decode_svg(&text).unwrap().unwrap();
    assert_eq!(native.document.cmyk_profile.as_ref().unwrap().data, a.data);
    assert_eq!(fixture::bitmaps(&native.document).len(), 1);
    fixture::assert_picture(
        fixture::bitmaps(&native.document)[0],
        &fixture::CMYKA,
        Some(&a),
    );
    for policy in [0, 1, 2] {
        let mut target = destination(policy);
        assert_ne!(source.engine.instance(), target.engine.instance());
        assert!(target.clipboard.is_none(), "no shared internal Clipboard");
        let before = snapshot(&target);
        let old_id = fixture::bitmaps(target.doc())[0].id;
        target.try_paste_svg_text(&text).unwrap();
        assert_eq!(
            target.engine.history_labels().0.len(),
            before.history.0.len() + 1
        );
        assert_eq!(target.engine.history_labels().0.last(), Some(&"Paste"));
        let target_profile = if policy == 2 { &a } else { &b };
        assert_eq!(
            target.doc().cmyk_profile.as_ref().unwrap().data,
            target_profile.data
        );
        let pictures = fixture::bitmaps(target.doc());
        assert_eq!(
            pictures.len(),
            2,
            "native picture must survive, not an RGB-only SVG approximation"
        );
        for picture in pictures {
            let expected = if picture.id == old_id && policy == 2 {
                fixture::expected_samples(&fixture::CMYKA, Some(&b), Some(&a), fixture::options())
            } else if picture.id != old_id && policy == 0 {
                fixture::expected_samples(&fixture::CMYKA, Some(&a), Some(&b), fixture::options())
            } else {
                fixture::CMYKA.to_vec()
            };
            fixture::assert_picture(picture, &expected, Some(target_profile));
        }
        let after = target.doc().clone();
        let selected = target.selection.clone();
        assert_eq!(selected.len(), 2);
        target.undo();
        after_undo(&target, &before);
        target.redo();
        assert_eq!(target.doc(), &after);
        assert_eq!(target.selection, selected);
        assert_eq!(snapshot(&source), source_before);
    }
}

#[test]
fn source_without_document_tag_publishes_its_real_working_profile_before_cross_app_paste() {
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    let mut source = configured(None);
    let path = directory("source-default").join("working-a.icc");
    std::fs::write(&path, a.data.as_slice()).unwrap();
    source.settings.color.cmyk_profile_path = path.display().to_string();
    source.settings.color.cmyk_profile = a.name.clone();
    let before = snapshot(&source);
    let text = publish(&mut source);
    assert_eq!(snapshot(&source), before);
    let payload = crate::clipboard_wire::decode_svg(&text).unwrap().unwrap();
    assert_eq!(payload.document.cmyk_profile.as_ref().unwrap().data, a.data);
    assert_eq!(payload.document.metadata.cmyk_profile, a.name);
    let mut target = destination(0);
    target.try_paste_svg_text(&text).unwrap();
    let incoming = fixture::bitmaps(target.doc())
        .into_iter()
        .find(|shape| target.selection.contains(&shape.id))
        .unwrap();
    fixture::assert_picture(
        incoming,
        &fixture::expected_samples(&fixture::CMYKA, Some(&a), Some(&b), fixture::options()),
        Some(&b),
    );
}

#[test]
fn explicit_source_builtin_is_not_reinterpreted_as_a_different_target_working_default() {
    let b = fixture::profile(true);
    let path = directory("target-default").join("working-b.icc");
    std::fs::write(&path, b.data.as_slice()).unwrap();
    let mut source = configured(None);
    let source_before = snapshot(&source);
    let text = publish(&mut source);
    assert!(
        crate::clipboard_wire::decode_svg(&text)
            .unwrap()
            .unwrap()
            .document
            .cmyk_profile
            .is_none()
    );
    for policy in [0, 1, 2] {
        let mut target = configured(None);
        target.settings.color.import_cmyk = policy;
        target.settings.color.cmyk_profile_path = path.display().to_string();
        target.settings.color.cmyk_profile = b.name.clone();
        target.selection = vec![target.doc().pages[0].layers[0].shapes[1].id];
        target
            .engine
            .run(&Command::SetTitle {
                title: "Edited target using working B".into(),
            })
            .unwrap();
        let before = snapshot(&target);
        let old_id = fixture::bitmaps(target.doc())[0].id;
        target.try_paste_svg_text(&text).unwrap();
        if policy == 2 {
            assert!(target.doc().cmyk_profile.is_none());
        } else {
            assert_eq!(target.doc().cmyk_profile.as_ref().unwrap().data, b.data);
        }
        let pictures = fixture::bitmaps(target.doc());
        assert_eq!(pictures.len(), 2);
        for picture in pictures {
            let expected = if picture.id == old_id && policy == 2 {
                fixture::expected_samples(&fixture::CMYKA, Some(&b), None, fixture::options())
            } else if picture.id != old_id && policy == 0 {
                fixture::expected_samples(&fixture::CMYKA, None, Some(&b), fixture::options())
            } else {
                fixture::CMYKA.to_vec()
            };
            fixture::assert_picture(
                picture,
                &expected,
                if policy == 2 { None } else { Some(&b) },
            );
        }
        let after = target.doc().clone();
        target.undo();
        after_undo(&target, &before);
        target.redo();
        assert_eq!(target.doc(), &after);
        assert_eq!(snapshot(&source), source_before);
    }
}

#[test]
fn failed_publication_cannot_cut_or_replace_a_previous_clipboard() {
    let mut source = configured(Some(&fixture::profile(false)));
    let _previous_bytes = publish(&mut source);
    source
        .engine
        .run(&Command::SetTitle {
            title: "Held on redo before publication failure".into(),
        })
        .unwrap();
    source.undo();
    let before = snapshot(&source);
    let clipboard = format!("{:?}", source.clipboard);
    for cutting in [false, true] {
        let mut captured = None;
        let rejected = |text: &str| {
            captured = Some(text.to_owned());
            Err("Injected publisher rejected bytes".into())
        };
        let result = if cutting {
            source.try_cut_with_publisher(rejected)
        } else {
            source.try_copy_with_publisher(rejected)
        };
        assert!(result.is_err());
        assert!(
            captured.is_some(),
            "exercise failure after the publisher received the candidate"
        );
        assert_eq!(snapshot(&source), before);
        assert_eq!(format!("{:?}", source.clipboard), clipboard);
    }
    let mut malformed = source.doc().clone();
    malformed.pages[0].layers[0].shapes[0]
        .picture_inks
        .as_mut()
        .unwrap()
        .png = vec![1, 2, 3, 4].into();
    source.engine.replace(malformed);
    let before = snapshot(&source);
    let mut called = false;
    assert!(
        source
            .try_cut_with_publisher(|_| {
                called = true;
                Ok(())
            })
            .is_err()
    );
    assert!(!called, "reject invalid native data before any publication");
    assert_eq!(snapshot(&source), before);
    assert_eq!(format!("{:?}", source.clipboard), clipboard);
}

fn text_source(text: &str) -> App {
    use tracedraw_core::{Color, Fill, Shape, document::TextSpan};
    let mut source = configured(Some(&fixture::profile(false)));
    let id = source.engine.new_shape_id();
    let mut shape = Shape::new(
        id,
        ShapeKind::Text {
            spans: vec![TextSpan::new(text, "Selawik", 18.0)],
            origin: Point::new(2.0, 20.0),
            frame: None,
            align: Default::default(),
            para: Default::default(),
            on_path: None,
        },
    );
    shape.fill = Fill::Solid(Color::rgb8(23, 143, 67));
    source
        .engine
        .run(&Command::AddShape {
            layer: source.active_layer().unwrap(),
            shape,
        })
        .unwrap();
    source.selection = vec![id];
    source
}

#[test]
fn text_receives_characters_without_metadata_even_when_native_json_is_invalid() {
    let mut source = text_source("Hello ñ");
    let text = publish(&mut source);
    let damaged = replace_json_bytes(&text, b"{invalid native JSON");
    let mut target = destination(0);
    target.start_text(Point::new(5.0, 40.0), None);
    target.text_insert("x");
    target.try_paste_svg_text(&text).unwrap();
    assert_eq!(
        target.edit_text(),
        "xHello ñ",
        "valid native text must preserve its exact characters"
    );
    let mut target = destination(0);
    target.start_text(Point::new(5.0, 40.0), None);
    target.text_insert("x");
    let before = snapshot(&target);
    assert!(
        target.try_paste_svg_text(&damaged).is_err(),
        "invalid JSON with an outlined-only SVG has no recoverable character channel and must report rejection"
    );
    assert_eq!(snapshot(&target), before);
    assert_eq!(target.edit_text(), "x");
}

#[test]
fn copying_text_edits_publishes_plain_characters_and_failed_cut_retains_them() {
    let mut source = destination(0);
    source.start_text(Point::new(5.0, 40.0), None);
    source.text_insert("Texto ágil ñ");
    source.text_edit.as_mut().unwrap().anchor = 0;
    let mut captured = None;
    assert!(
        source
            .try_copy_with_publisher(|text| {
                captured = Some(text.to_owned());
                Ok(())
            })
            .unwrap()
    );
    assert_eq!(captured.as_deref(), Some("Texto ágil ñ"));
    let before = snapshot(&source);
    let text_before = source.edit_text();
    assert!(
        source
            .try_cut_with_publisher(|_| Err("Injected text publication rejection".into()))
            .is_err()
    );
    assert_eq!(snapshot(&source), before);
    assert_eq!(source.edit_text(), text_before);
}

#[test]
fn foreign_svg_still_pastes_as_vector_and_undoes_in_one_step() {
    let text = r##"<svg xmlns="http://www.w3.org/2000/svg" width="20mm" height="10mm" viewBox="0 0 20 10"><rect x="2" y="1" width="8" height="6" fill="#12ab34"/></svg>"##;
    assert!(!crate::clipboard_wire::looks_native(text));
    assert!(crate::clipboard_wire::decode_svg(text).unwrap().is_none());
    let mut target = destination(1);
    let before = snapshot(&target);
    target.try_paste_svg_text(text).unwrap();
    assert_eq!(target.selection.len(), 1);
    let shape = target.doc().find_shape(target.selection[0]).unwrap();
    assert_eq!(
        shape.fill,
        tracedraw_core::Fill::Solid(tracedraw_core::Color::rgb8(18, 171, 52))
    );
    assert_eq!(
        target.engine.history_labels().0.len(),
        before.history.0.len() + 1
    );
    target.undo();
    after_undo(&target, &before);
}

#[test]
fn namespace_identity_does_not_depend_on_the_emitters_prefix() {
    let mut source = configured(Some(&fixture::profile(false)));
    let text = publish(&mut source);
    let renamed = text
        .replace("xmlns:cc=", "xmlns:alternative=")
        .replace("cc:selection", "alternative:selection");
    assert_ne!(renamed, text);
    let mut target = destination(1);
    target.try_paste_svg_text(&renamed).unwrap();
    assert_eq!(fixture::bitmaps(target.doc()).len(), 2);
    let incoming = fixture::bitmaps(target.doc())
        .into_iter()
        .find(|shape| target.selection.contains(&shape.id))
        .unwrap();
    fixture::assert_picture(incoming, &fixture::CMYKA, Some(&fixture::profile(true)));
}

#[test]
fn malformed_owned_packets_never_fall_back_to_the_valid_svg_picture() {
    let mut source = configured(Some(&fixture::profile(false)));
    let text = publish(&mut source);
    let document = roxmltree::Document::parse(&text).unwrap();
    let native = document
        .descendants()
        .find(|node| {
            node.tag_name().namespace() == Some(crate::clipboard_wire::NAMESPACE)
                && node.tag_name().name() == "selection"
        })
        .unwrap();
    let block = &text[native.parent().unwrap().range()];
    let closing = text.rfind("</svg>").unwrap();
    let duplicated = format!("{}{}{}", &text[..closing], block, &text[closing..]);
    let original_json = json_bytes(&text);
    let duplicate_json = String::from_utf8(original_json.clone()).unwrap().replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert_ne!(duplicate_json.as_bytes(), original_json);
    let cases = vec![
        (
            "unknown XML version",
            text.replace("version=\"1\"", "version=\"999\""),
        ),
        (
            "unknown JSON version",
            mutate_json(&text, |value| value["version"] = serde_json::json!(999)),
        ),
        (
            "unknown payload version",
            mutate_json(&text, |value| {
                value["payload"]["version"] = serde_json::json!(999)
            }),
        ),
        ("duplicate native block", duplicated),
        (
            "duplicate JSON field",
            replace_json_bytes(&text, duplicate_json.as_bytes()),
        ),
        (
            "truncated JSON",
            replace_json_bytes(&text, &original_json[..original_json.len() - 3]),
        ),
        ("truncated XML", text[..closing - 2].to_owned()),
        (
            "invalid document ICC",
            mutate_json(&text, |value| {
                value["payload"]["document"]["cmyk_profile"]["data"] =
                    serde_json::json!(STANDARD.encode([1, 2, 3, 4]))
            }),
        ),
        (
            "invalid own picture ICC",
            mutate_json(&text, |value| {
                value["payload"]["document"]["pages"][0]["layers"][0]["shapes"][0]["picture_inks"]
                    ["profile"]["data"] = serde_json::json!(STANDARD.encode([1, 2, 3, 4]))
            }),
        ),
        (
            "invalid ink PNG",
            mutate_json(&text, |value| {
                value["payload"]["document"]["pages"][0]["layers"][0]["shapes"][0]["picture_inks"]
                    ["png"] = serde_json::json!(STANDARD.encode([1, 2, 3, 4]))
            }),
        ),
        (
            "wrong picture fingerprint",
            mutate_json(&text, |value| {
                let of = &mut value["payload"]["document"]["pages"][0]["layers"][0]["shapes"][0]["picture_inks"]
                    ["of"];
                *of = serde_json::json!(of.as_u64().unwrap() ^ 1);
            }),
        ),
        (
            "unreferenced palette resource",
            mutate_json(&text, |value| {
                value["payload"]["document"]["palette"] =
                    serde_json::to_value(vec![tracedraw_core::Color::rgb8(199, 77, 2)]).unwrap()
            }),
        ),
    ];
    for (name, damaged) in cases {
        assert!(
            crate::clipboard_wire::looks_native(&damaged),
            "own packet recognized: {name}"
        );
        assert!(
            crate::clipboard_wire::decode_svg(&damaged).is_err(),
            "must reject {name}"
        );
        let mut target = destination(0);
        target
            .engine
            .run(&Command::SetTitle {
                title: "Keep this redo step".into(),
            })
            .unwrap();
        target.undo();
        let before = snapshot(&target);
        let clipboard = format!("{:?}", target.clipboard);
        assert!(
            target.try_paste_svg_text(&damaged).is_err(),
            "no SVG fallback: {name}"
        );
        assert_eq!(snapshot(&target), before, "atomic rejection: {name}");
        assert_eq!(
            format!("{:?}", target.clipboard),
            clipboard,
            "clipboard unchanged: {name}"
        );
    }
}

fn prefixed_root(text: &str) -> String {
    text.replacen("<svg ", "<s:svg xmlns:s=\"http://www.w3.org/2000/svg\" ", 1)
        .replacen("</svg>", "</s:svg>", 1)
}

#[test]
fn public_paste_routes_prefixed_owned_svg_as_native_and_rejects_corruption_atomically() {
    let mut source = configured(Some(&fixture::profile(false)));
    let published = publish(&mut source);
    let prefixed = prefixed_root(&published);
    assert_ne!(prefixed, published);
    let mut target = destination(1);
    let before = snapshot(&target);
    target.paste_from(crate::clipboard::ClipboardContent {
        text: Some(prefixed.clone()),
        ..Default::default()
    });
    let incoming = fixture::bitmaps(target.doc())
        .into_iter()
        .find(|shape| target.selection.contains(&shape.id))
        .unwrap();
    fixture::assert_picture(incoming, &fixture::CMYKA, Some(&fixture::profile(true)));
    assert_eq!(
        target.engine.history_labels().0.len(),
        before.history.0.len() + 1
    );
    let closing = prefixed.rfind("</s:svg>").unwrap();
    for damaged in [
        replace_json_bytes(&prefixed, b"{invalid"),
        prefixed[..closing - 2].to_owned(),
    ] {
        let mut target = destination(0);
        let before = snapshot(&target);
        target.paste_from(crate::clipboard::ClipboardContent {
            text: Some(damaged),
            ..Default::default()
        });
        assert_eq!(
            snapshot(&target),
            before,
            "public route must never insert transport text or use the SVG fallback after a native error"
        );
    }
}

#[test]
fn ordinary_text_merely_mentioning_the_namespace_stays_literal_in_both_paste_contexts() {
    let plain = "Referencia urn:craft-corel:selection, texto normal ñ";
    let mut target = destination(0);
    target.paste_from(crate::clipboard::ClipboardContent {
        text: Some(plain.into()),
        ..Default::default()
    });
    let text: Vec<_> = target.doc().pages[0].layers[0]
        .shapes
        .iter()
        .filter_map(|shape| {
            if let ShapeKind::Text { spans, .. } = &shape.kind {
                Some(
                    spans
                        .iter()
                        .map(|span| span.text.as_str())
                        .collect::<String>(),
                )
            } else {
                None
            }
        })
        .collect();
    assert_eq!(text, vec![plain]);
    let mut target = destination(0);
    target.start_text(Point::new(5.0, 40.0), None);
    target.text_insert("x");
    target.text_paste_text(Some(plain.into()));
    assert_eq!(target.edit_text(), format!("x{plain}"));
    let mut target = destination(0);
    target.start_text(Point::new(5.0, 40.0), None);
    target.text_insert("x");
    target.paste_from(crate::clipboard::ClipboardContent {
        text: Some(plain.into()),
        ..Default::default()
    });
    assert_eq!(target.edit_text(), format!("x{plain}"));
}

fn rich_source() -> App {
    use crate::connectors::{Connection, ConnectorKind};
    use crate::dimensions::{Anchor, Dimension, DimensionStyle, Measure};
    use tracedraw_core::{
        Color, Fill, Shape,
        document::{DocumentFont, Ink, ObjectStyle, Symbol, TextSpan},
        style::{Pattern, Stroke},
    };
    let mut app = configured(Some(&fixture::profile(false)));
    let mut doc = app.doc().clone();
    doc.pages[0].layers[0].shapes[0].uid = Some(tracedraw_core::document::Uid(
        0x11111111111111111111111111111111,
    ));
    let inside = doc.pages[0].layers[0].shapes[1].id;
    let outside = doc.ids_mut().shape();
    let mut unselected = doc.pages[0].layers[0].shapes[1].clone();
    unselected.id = outside;
    unselected.name = Some("Unselected external endpoint".into());
    doc.pages[0].layers[0].shapes.push(unselected);
    let ink = Ink {
        id: 7,
        name: "Collision ink".into(),
        cmyk: [0.17, 0.53, 0.91, 0.23],
    };
    let mut style_probe = doc.pages[0].layers[0].shapes[1].clone();
    style_probe.fill = Fill::Solid(ink.color(0.625));
    let mut style = ObjectStyle::from_shape("Collision style", &style_probe);
    style.id = 7;
    doc.object_styles.push(style);
    let mut unused = ObjectStyle::from_shape("Unused style", &style_probe);
    unused.id = 8;
    doc.object_styles.push(unused);
    doc.inks.push(ink);
    doc.inks.push(Ink {
        id: 8,
        name: "Unused source ink".into(),
        cmyk: [0.1, 0.2, 0.3, 0.4],
    });
    let mut inner = fixture::picture(doc.cmyk_profile.as_ref());
    inner.id = doc.ids_mut().shape();
    inner.style = Some(7);
    inner.name = Some("Hidden native picture".into());
    doc.symbols.push(Symbol {
        name: "Inner stroke dependency".into(),
        shape: inner,
    });
    let inner_reference = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 0 },
    );
    let outer = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Group {
            children: vec![inner_reference],
        },
    );
    doc.symbols.push(Symbol {
        name: "Outer stroke dependency".into(),
        shape: outer,
    });
    let unused_shape = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Rect {
            rect: tracedraw_core::Rect::new(1.0, 2.0, 3.0, 4.0),
            radius: 0.0,
            corners: None,
        },
    );
    doc.symbols.push(Symbol {
        name: "Unused source symbol".into(),
        shape: unused_shape,
    });
    let reference = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 1 },
    );
    let mut span = TextSpan::new("Stroke-carried text", "TestSymbol", 13.25);
    span.layout_only = true;
    span.background = Some(Fill::Solid(Color::cmyk_pct(11.0, 23.0, 47.0, 31.0)));
    let mut para = tracedraw_core::document::ParagraphStyle::default();
    para.flow = tracedraw_core::text_flow::FrameFlow::new(
        Vec::new(),
        Some((0.123456789012345, 4.23456789012345)),
    );
    let mut tile_text = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Text {
            spans: vec![span],
            origin: Point::new(0.000000000123456789, 2.123456789012345),
            frame: Some(tracedraw_core::Size::new(25.0, 12.0)),
            align: Default::default(),
            para,
            on_path: None,
        },
    );
    tile_text.name = Some("Text inside outline pattern".into());
    let mut stroke = Stroke::new(Color::cmyk_pct(17.0, 33.0, 49.0, 65.0), 0.3456789012345);
    stroke.fill = Some(Box::new(Fill::Pattern(Pattern::Vector {
        shapes: vec![reference, tile_text],
        tile: tracedraw_core::Size::new(28.123456789, 16.23456789),
    })));
    doc.pages[0].layers[0].shapes[1].stroke = Some(stroke);
    doc.pages[0].layers[0].shapes[1].transform = tracedraw_core::geometry::Affine::new([
        1.000000000000003,
        0.000000000123,
        0.000000000456,
        1.000000000000007,
        0.123456789012345,
        -0.234567890123456,
    ]);
    doc.fonts.push(DocumentFont::new(
        "TestSymbol",
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").to_vec(),
    ));
    let mut selected = app.selection.clone();
    for (name, from, to) in [
        (
            "Internal native connector",
            doc.pages[0].layers[0].shapes[0].id,
            inside,
        ),
        ("External native connector", inside, outside),
    ] {
        let connection = Connection {
            from,
            to,
            from_anchor: (0.5, 0.5),
            to_anchor: (0.5, 0.5),
            kind: ConnectorKind::RightAngle,
            radius: 2.25,
            ends: (Point::new(7.0, 9.0), Point::new(25.0, 20.0)),
        };
        let mut shape = Shape::new(
            doc.ids_mut().shape(),
            ShapeKind::Path {
                path: crate::connectors::route(
                    connection.kind,
                    connection.radius,
                    connection.ends.0,
                    connection.ends.1,
                ),
                closed: false,
            },
        );
        shape.name = Some(name.into());
        shape.data = crate::connectors::with_connection(shape.data, Some(&connection));
        selected.push(shape.id);
        doc.pages[0].layers[0].shapes.push(shape);
    }
    let mut dimension = Dimension::new(
        Measure::Linear {
            a: Point::new(10.0, 15.0),
            b: Point::new(30.0, 15.0),
            at: Point::new(10.0, 24.0),
        },
        DimensionStyle::default(),
    );
    dimension.anchors = vec![
        Anchor {
            point: 0,
            shape: inside,
            node: 0,
            segment: None,
        },
        Anchor {
            point: 1,
            shape: outside,
            node: 0,
            segment: None,
        },
    ];
    let children = dimension.shapes(
        [
            doc.ids_mut().shape(),
            doc.ids_mut().shape(),
            doc.ids_mut().shape(),
        ],
        "TestSymbol",
        app.units,
    );
    let mut group = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Group {
            children: children.to_vec(),
        },
    );
    group.name = Some("Native editable dimension".into());
    let mut stored = serde_json::to_value(&dimension).unwrap();
    stored["owned_extension"] = serde_json::json!({"retain":"measurement extension"});
    group.data.push((
        crate::dimensions::DATA_KEY.into(),
        serde_json::to_string(&stored).unwrap(),
    ));
    selected.push(group.id);
    doc.pages[0].layers[0].shapes.push(group);
    app.engine.replace(doc);
    app.selection = selected;
    app
}

fn collision_destination() -> App {
    use tracedraw_core::{
        Color, Fill, Shape,
        document::{Ink, ObjectStyle, Symbol},
    };
    let mut app = destination(1);
    let mut doc = app.doc().clone();
    doc.pages[0].layers[0].shapes[0].uid = Some(tracedraw_core::document::Uid(
        0x11111111111111111111111111111111,
    ));
    let mut trap = doc.pages[0].layers[0].shapes[1].clone();
    trap.id = ShapeId(5);
    trap.name = Some("Destination numeric endpoint trap".into());
    doc.pages[0].layers[0].shapes.push(trap);
    let mut remote = doc.pages[0].layers[0].shapes[1].clone();
    remote.id = ShapeId(300);
    remote.name = Some("Destination unrelated endpoint".into());
    doc.pages[0].layers[0].shapes.push(remote);
    let connection = crate::connectors::Connection {
        from: ShapeId(3),
        to: ShapeId(300),
        from_anchor: (0.5, 0.5),
        to_anchor: (0.5, 0.5),
        kind: Default::default(),
        radius: 1.0,
        ends: (Point::new(7.0, 9.0), Point::new(30.0, 20.0)),
    };
    let mut connector = Shape::new(
        ShapeId(301),
        ShapeKind::Path {
            path: crate::connectors::route(
                connection.kind,
                connection.radius,
                connection.ends.0,
                connection.ends.1,
            ),
            closed: false,
        },
    );
    connector.name = Some("Existing destination connector must not be copied".into());
    connector.data = crate::connectors::with_connection(connector.data, Some(&connection));
    doc.pages[0].layers[0].shapes.push(connector);
    let mut definition = doc.pages[0].layers[0].shapes[1].clone();
    definition.id = ShapeId(302);
    definition.fill = Fill::Solid(Color::rgb8(201, 17, 53));
    doc.symbols.push(Symbol {
        name: "Existing destination symbol".into(),
        shape: definition.clone(),
    });
    let mut style = ObjectStyle::from_shape("Collision style", &definition);
    style.id = 7;
    doc.object_styles.push(style);
    doc.inks.push(Ink {
        id: 7,
        name: "Collision ink".into(),
        cmyk: [0.8, 0.1, 0.4, 0.01],
    });
    doc.ids_mut().reserve(303);
    app.engine.replace(doc);
    app.selection = vec![ShapeId(4)];
    app.engine
        .run(&Command::SetTitle {
            title: "Edited destination with namespace collisions".into(),
        })
        .unwrap();
    app
}

fn named<'a>(app: &'a App, name: &str) -> &'a tracedraw_core::Shape {
    app.doc()
        .pages
        .iter()
        .flat_map(|page| &page.layers)
        .flat_map(|layer| &layer.shapes)
        .find(|shape| shape.name.as_deref() == Some(name))
        .unwrap()
}

fn assert_rich_import(target: &App, before: &Snapshot, expected_roots: usize) {
    use tracedraw_core::{Fill, style::Pattern};
    assert_eq!(
        target.doc().pages[0].layers[0].shapes.len(),
        before.document.pages[0].layers[0].shapes.len() + expected_roots,
        "foreign bytes must never trigger same-document auxiliary connector copies"
    );
    assert_eq!(target.doc().symbols.len(), 3);
    assert_eq!(target.doc().symbols[0], before.document.symbols[0]);
    assert!(
        !target
            .doc()
            .symbols
            .iter()
            .any(|symbol| symbol.name == "Unused source symbol")
    );
    assert_eq!(target.doc().fonts.len(), 1);
    assert_eq!(target.doc().fonts[0].family, "TestSymbol");
    assert_eq!(
        target.doc().fonts[0].data.as_slice(),
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").as_slice()
    );
    assert_eq!(
        target.doc().inks.iter().find(|ink| ink.id == 7).unwrap(),
        &before.document.inks[0]
    );
    let ink = target
        .doc()
        .inks
        .iter()
        .find(|ink| ink.id != 7 && ink.cmyk == [0.17, 0.53, 0.91, 0.23])
        .unwrap();
    assert!(
        !target
            .doc()
            .inks
            .iter()
            .any(|ink| ink.name == "Unused source ink")
    );
    let vector = target
        .selection
        .iter()
        .filter_map(|id| target.doc().find_shape(*id))
        .find(|shape| shape.name.as_deref() == Some("Owned wire vector"))
        .unwrap();
    let Fill::Pattern(Pattern::Vector { shapes, .. }) =
        vector.stroke.as_ref().unwrap().fill.as_deref().unwrap()
    else {
        panic!("native outline pattern");
    };
    let ShapeKind::SymbolInstance { index: outer } = shapes[0].kind else {
        panic!("native symbol in outline pattern");
    };
    assert_eq!(target.doc().symbols[outer].name, "Outer stroke dependency");
    let ShapeKind::Group { children } = &target.doc().symbols[outer].shape.kind else {
        panic!("outer symbol group");
    };
    let ShapeKind::SymbolInstance { index: inner } = children[0].kind else {
        panic!("nested symbol");
    };
    assert_eq!(target.doc().symbols[inner].name, "Inner stroke dependency");
    let picture = &target.doc().symbols[inner].shape;
    fixture::assert_picture(picture, &fixture::CMYKA, Some(&fixture::profile(true)));
    let style = target
        .doc()
        .object_styles
        .iter()
        .find(|style| Some(style.id) == picture.style)
        .unwrap();
    assert_ne!(style.id, 7);
    let Fill::Solid(tracedraw_core::Color::Spot { ink: style_ink, .. }) = style.fill else {
        panic!("native spot in transitive object style");
    };
    assert_eq!(style_ink, ink.id);
    let ShapeKind::Text {
        spans,
        para,
        origin,
        ..
    } = &shapes[1].kind
    else {
        panic!("text in outline pattern");
    };
    assert!(
        spans[0].layout_only,
        "preserve omitted TextSpan field through bytes"
    );
    assert_eq!(spans[0].font_family, "TestSymbol");
    assert_eq!(para.flow.grid, Some((0.123456789012345, 4.23456789012345)));
    assert_eq!(origin.x.to_bits(), 0.000000000123456789f64.to_bits());
    let internal =
        crate::connectors::Connection::of(named(target, "Internal native connector")).unwrap();
    assert!(target.selection.contains(&internal.from) && target.selection.contains(&internal.to));
    assert!(
        crate::connectors::Connection::of(named(target, "External native connector")).is_none(),
        "never bind to destination id5"
    );
    let dimension =
        crate::dimensions::Dimension::of(named(target, "Native editable dimension")).unwrap();
    assert_eq!(dimension.anchors.len(), 1);
    assert!(target.selection.contains(&dimension.anchors[0].shape));
    let stored = &named(target, "Native editable dimension")
        .data
        .iter()
        .find(|(key, _)| key == crate::dimensions::DATA_KEY)
        .unwrap()
        .1;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(stored).unwrap()["owned_extension"]["retain"],
        "measurement extension"
    );
}

#[test]
fn outline_pattern_dependencies_cross_wire_and_numeric_instance_collision_never_means_same_document()
 {
    let mut source = rich_source();
    let before_source = snapshot(&source);
    let published = publish(&mut source);
    let mut target = collision_destination();
    let before = snapshot(&target);
    let text = mutate_json(&published, |value| {
        value["payload"]["source_instance"] = serde_json::json!(target.engine.instance())
    });
    let decoded = crate::clipboard_wire::decode_svg(&text).unwrap().unwrap();
    assert_eq!(decoded.source_instance, target.engine.instance());
    assert_eq!(
        decoded.document.symbols.len(),
        2,
        "only dependencies reachable through Stroke.fill"
    );
    assert_eq!(decoded.document.object_styles.len(), 1);
    assert_eq!(decoded.document.inks.len(), 1);
    let imported_count = source.selection.len();
    target.try_paste_svg_text(&text).unwrap();
    assert_eq!(
        target.engine.history_labels().0.len(),
        before.history.0.len() + 1
    );
    assert_rich_import(&target, &before, imported_count);
    let after = target.doc().clone();
    let selected = target.selection.clone();
    target.undo();
    after_undo(&target, &before);
    target.redo();
    assert_eq!(target.doc(), &after);
    assert_eq!(target.selection, selected);
    assert_eq!(snapshot(&source), before_source);
}

fn spawn_known_child(mode: &str, input: Option<&str>) -> std::path::PathBuf {
    use std::process::{Command as ProcessCommand, Stdio};
    let output = directory(mode);
    let profile = output.join("config");
    std::fs::create_dir_all(&profile).unwrap();
    let input_path = output.join("published.svg");
    if let Some(text) = input {
        std::fs::write(&input_path, text).unwrap();
    }
    let exe = std::env::current_exe().unwrap();
    let test = "clipboard_wire_tests::child_executes_only_under_explicit_wire_test_environment";
    let stdout = std::fs::File::create(output.join("stdout.log")).unwrap();
    let stderr = std::fs::File::create(output.join("stderr.log")).unwrap();
    let mut command = ProcessCommand::new(&exe);
    command
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env("CRAFT_COREL_WIRE_CHILD_MODE", mode)
        .env("CRAFT_COREL_WIRE_CHILD_OUTPUT", &output)
        .env("CRAFT_COREL_WIRE_CHILD_INPUT", &input_path)
        .env("TRACEDRAW_CONFIG_DIR", &profile)
        .env("CRAFT_COREL_CONFIG_DIR", &profile)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000 | 0x00004000);
    }
    std::fs::write(
        output.join("command.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "executable":exe,"source":"std::env::current_exe of this known Rust test harness",
            "arguments":["--exact",test,"--nocapture","--test-threads=1"],"mode":mode,
            "windows_creation_flags":"CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS",
            "profile":profile,
        }))
        .unwrap(),
    )
    .unwrap();
    let mut child = command.spawn().unwrap();
    let began = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if began.elapsed() > std::time::Duration::from_secs(45) {
            child.kill().unwrap();
            let _ = child.wait();
            panic!(
                "known child timed out: {mode}; evidence {}",
                output.display()
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert!(
        status.success(),
        "known child failed: {mode}, status {status}; evidence {}",
        output.display()
    );
    assert!(
        output.join("result.json").is_file(),
        "the exact child filter must execute, not just match zero tests"
    );
    output
}

fn chain_app(depth: usize) -> (App, Option<Document>) {
    let mut doc = fixture::document(Some(&fixture::profile(false)));
    fixture::add_vector(&mut doc);
    let mut tree = doc.pages[0].layers[0].shapes.pop().unwrap();
    for _ in 0..depth {
        tree = tracedraw_core::Shape::new(
            doc.ids_mut().shape(),
            ShapeKind::Group {
                children: vec![tree],
            },
        );
    }
    let root = tree.id;
    doc.pages[0].layers[0].shapes.push(tree);
    let mut app = App::headless();
    app.page = doc.pages[0].id;
    let original = if depth > 128 {
        Some(app.engine.test_swap_document(doc))
    } else {
        app.engine.replace(doc);
        None
    };
    app.selection = vec![root];
    app.settings.color.intent = "Saturation".into();
    app.settings.color.black_point_compensation = false;
    app.settings.color.preserve_pure_black = false;
    app.settings.color.cmyk_profile_path.clear();
    (app, original)
}

fn chain_ids(doc: &Document) -> Vec<ShapeId> {
    let mut output = Vec::new();
    let mut pending: Vec<_> = doc.pages[0].layers[0].shapes.iter().collect();
    while let Some(shape) = pending.pop() {
        output.push(shape.id);
        if let ShapeKind::Group { children } = &shape.kind {
            pending.extend(children);
        }
    }
    output
}

fn dismantle_chain(app: &mut App, original: Document) {
    let mut doc = app.engine.test_swap_document(original);
    let mut pending = std::mem::take(&mut doc.pages[0].layers[0].shapes);
    while let Some(mut shape) = pending.pop() {
        if let ShapeKind::Group { children } = std::mem::replace(
            &mut shape.kind,
            ShapeKind::Group {
                children: Vec::new(),
            },
        ) {
            pending.extend(children);
        }
    }
}

#[test]
fn child_executes_only_under_explicit_wire_test_environment() {
    let Ok(mode) = std::env::var("CRAFT_COREL_WIRE_CHILD_MODE") else {
        return;
    };
    let output =
        std::path::PathBuf::from(std::env::var_os("CRAFT_COREL_WIRE_CHILD_OUTPUT").unwrap());
    if mode.starts_with("hostile-wire-") {
        let input =
            std::fs::read_to_string(std::env::var_os("CRAFT_COREL_WIRE_CHILD_INPUT").unwrap())
                .unwrap();
        let error = crate::clipboard_wire::decode_svg(&input)
            .expect_err("hostile input must fail before any geometric expansion");
        let mut target = destination(0);
        let before = snapshot(&target);
        let clipboard = format!("{:?}", target.clipboard);
        assert!(target.try_paste_svg_text(&input).is_err(), "{mode}");
        assert_eq!(snapshot(&target), before, "{mode}");
        assert_eq!(format!("{:?}", target.clipboard), clipboard, "{mode}");
        std::fs::write(
            output.join("result.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "passed":true,"mode":mode,"decode_error":error,
                "input_bytes":input.len(),"destination_complete_snapshot_unchanged":true,
                "encoded_by_production_encoder":false,"geometry_was_never_expanded":true,
            }))
            .unwrap(),
        )
        .unwrap();
        return;
    }
    if mode == "independent-process" {
        let input =
            std::fs::read_to_string(std::env::var_os("CRAFT_COREL_WIRE_CHILD_INPUT").unwrap())
                .unwrap();
        let mut target = collision_destination();
        let before = snapshot(&target);
        let text = mutate_json(&input, |value| {
            value["payload"]["source_instance"] = serde_json::json!(target.engine.instance())
        });
        let value: serde_json::Value = serde_json::from_slice(&json_bytes(&text)).unwrap();
        let roots = value["payload"]["document"]["pages"][0]["layers"][0]["shapes"]
            .as_array()
            .unwrap()
            .len();
        target.try_paste_svg_text(&text).unwrap();
        assert_rich_import(&target, &before, roots);
        let after = target.doc().clone();
        let selected = target.selection.clone();
        target.undo();
        after_undo(&target, &before);
        target.redo();
        assert_eq!(target.doc(), &after);
        assert_eq!(target.selection, selected);
        std::fs::write(
            output.join("received.tdraw"),
            serde_json::to_vec_pretty(target.doc()).unwrap(),
        )
        .unwrap();
        std::fs::write(output.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "passed":true,"mode":mode,"input_bytes":input.len(),"source_instance_collision_forced":true,
            "rich_resources_verified":true,"outline_pattern_transitive_dependencies_verified":true,
            "supplements_verified_before_serializing_output":true,"one_undo_and_redo_verified":true,
        })).unwrap()).unwrap();
        return;
    }
    let depth = match mode.as_str() {
        "depth20" => 20,
        "depth64" => 64,
        "hostile140" => 140,
        _ => panic!("unknown controlled child mode"),
    };
    let (mut source, original) = chain_app(depth);
    if depth > 128 {
        // No recursive snapshot, serialization or clone of the hostile tree.
        let ids = chain_ids(source.doc());
        let selection = source.selection.clone();
        let counter = source.doc().ids().peek_next();
        let history = source.engine.history_labels();
        let revision = source.engine.revision();
        let mut called = false;
        let error = source
            .try_cut_with_publisher(|_| {
                called = true;
                Ok(())
            })
            .err();
        let after_ids = chain_ids(source.doc());
        let after_selection = source.selection.clone();
        let after_counter = source.doc().ids().peek_next();
        let after_history = source.engine.history_labels();
        let after_revision = source.engine.revision();
        dismantle_chain(
            &mut source,
            original.expect("hostile injection saves the exact original document"),
        );
        assert!(!called);
        assert!(
            error
                .as_deref()
                .is_some_and(|error| error.contains("traversal budget")),
            "{error:?}"
        );
        assert_eq!(after_ids, ids);
        assert_eq!(after_selection, selection);
        assert_eq!(after_counter, counter);
        assert_eq!(after_history, history);
        assert_eq!(after_revision, revision);
    } else {
        let before_ids = chain_ids(source.doc());
        let text = publish(&mut source);
        assert_eq!(chain_ids(source.doc()), before_ids);
        let decoded = crate::clipboard_wire::decode_svg(&text).unwrap().unwrap();
        assert_eq!(chain_ids(&decoded.document), before_ids);
        let mut target = destination(1);
        let before = snapshot(&target);
        target.try_paste_svg_text(&text).unwrap();
        assert_eq!(target.selection.len(), 1);
        target.undo();
        after_undo(&target, &before);
        target.redo();
        assert_eq!(target.selection.len(), 1);
    }
    std::fs::write(output.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"passed":true,"mode":mode,"depth":depth,"native_process_stack_used":true,"test_did_not_enlarge_RUST_MIN_STACK":true})).unwrap()).unwrap();
}

#[test]
fn actual_published_file_is_consumed_by_a_separate_known_harness_process() {
    let mut source = rich_source();
    let before = snapshot(&source);
    let text = publish(&mut source);
    let out = spawn_known_child("independent-process", Some(&text));
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("result.json")).unwrap()).unwrap();
    assert_eq!(report["passed"], true);
    assert_eq!(report["rich_resources_verified"], true);
    assert_eq!(snapshot(&source), before);
}

#[test]
fn supported_group_depths_and_hostile_source_preclone_guard_run_in_controlled_processes() {
    for mode in ["depth20", "depth64", "hostile140"] {
        spawn_known_child(mode, None);
    }
}

#[test]
fn inline_native_text_carries_real_font_spot_and_pattern_resources_in_one_undo_step() {
    use tracedraw_core::{
        Color, Fill, Shape,
        document::{DocumentFont, Ink, Symbol, TextSpan},
        style::Pattern,
    };
    let mut source = text_source("temporary");
    let text_id = source.selection[0];
    let mut doc = source.doc().clone();
    let ink = Ink {
        id: 7,
        name: "Collision ink".into(),
        cmyk: [0.17, 0.53, 0.91, 0.23],
    };
    let mut first = TextSpan::new("A", "TestSymbol", 19.25);
    first.fill = Some(Fill::Solid(ink.color(0.625)));
    let mut definition = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Rect {
            rect: tracedraw_core::Rect::new(0.0, 0.0, 2.0, 2.0),
            radius: 0.0,
            corners: None,
        },
    );
    definition.fill = Fill::Solid(Color::cmyk_pct(17.0, 33.0, 49.0, 65.0));
    doc.symbols.push(Symbol {
        name: "Inline pattern dependency".into(),
        shape: definition,
    });
    let reference = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::SymbolInstance { index: 0 },
    );
    let mut second = TextSpan::new("B", "TestSymbol", 19.25);
    second.fill = Some(Fill::Pattern(Pattern::Vector {
        shapes: vec![reference],
        tile: tracedraw_core::Size::new(3.0, 3.0),
    }));
    let ShapeKind::Text { spans, .. } = &mut doc.find_shape_mut(text_id).unwrap().kind else {
        unreachable!();
    };
    *spans = vec![first, second];
    doc.inks.push(ink);
    doc.fonts.push(DocumentFont::new(
        "TestSymbol",
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").to_vec(),
    ));
    source.engine.replace(doc);
    source.selection = vec![text_id];
    let text = publish(&mut source);
    let mut target = collision_destination();
    target.settings.color.import_cmyk = 0;
    target.start_text(Point::new(1.0, 50.0), None);
    // Establish the exact pre-paste text separately from the editor's
    // sentence-capitalization setting. Only the incoming AB is under test.
    target.text_insert("Prefix ");
    assert_eq!(target.edit_text(), "Prefix ");
    let before = snapshot(&target);
    let edited = target.text_edit.as_ref().unwrap().shape;
    target.try_paste_svg_text(&text).unwrap();
    assert_eq!(target.edit_text(), "Prefix AB");
    assert_eq!(
        target.engine.history_labels().0.len(),
        before.history.0.len() + 1
    );
    assert_eq!(target.doc().fonts.len(), 1);
    assert_eq!(target.doc().fonts[0].family, "TestSymbol");
    assert_eq!(
        target.doc().fonts[0].data.as_slice(),
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").as_slice()
    );
    let ShapeKind::Text { spans, .. } = &target.doc().find_shape(edited).unwrap().kind else {
        unreachable!();
    };
    let first = spans.iter().find(|span| span.text == "A").unwrap();
    assert_eq!(first.font_family, "TestSymbol");
    let Some(Fill::Solid(Color::Spot { ink, .. })) = first.fill else {
        panic!("native inline spot fill");
    };
    assert_ne!(ink, 7);
    assert_eq!(
        target
            .doc()
            .inks
            .iter()
            .find(|candidate| candidate.id == ink)
            .unwrap()
            .cmyk,
        [0.17, 0.53, 0.91, 0.23]
    );
    let second = spans.iter().find(|span| span.text == "B").unwrap();
    let Some(Fill::Pattern(Pattern::Vector { shapes, .. })) = &second.fill else {
        panic!("native inline pattern");
    };
    let ShapeKind::SymbolInstance { index } = shapes[0].kind else {
        unreachable!();
    };
    assert_eq!(
        target.doc().symbols[index].name,
        "Inline pattern dependency"
    );
    assert_eq!(
        target.doc().pages[0].layers[0].shapes.len(),
        before.document.pages[0].layers[0].shapes.len(),
        "temporary imported geometry must not leak into the text edit"
    );
    let after = target.doc().clone();
    target.undo();
    after_undo(&target, &before);
    target.redo();
    assert_eq!(target.doc(), &after);
}

#[test]
fn svg_fallback_uses_source_calibration_without_disturbing_an_outer_colour_scope() {
    use tracedraw_core::Color;
    use tracedraw_core::color::engine;
    use tracedraw_core::icc::{Profile, Transform};
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    for (untagged, builtin_engine) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut source = configured(if untagged { None } else { Some(&a) });
        source.settings.color.builtin_engine = builtin_engine;
        if untagged {
            let path = directory("fallback-default").join("a.icc");
            std::fs::write(&path, a.data.as_slice()).unwrap();
            source.settings.color.cmyk_profile_path = path.display().to_string();
            source.settings.color.cmyk_profile = a.name.clone();
        } else {
            let mut doc = source.doc().clone();
            let id = doc.pages[0].layers[0].shapes[0].id;
            let mut own_b = fixture::picture(Some(&b));
            own_b.id = id;
            doc.pages[0].layers[0].shapes[0] = own_b;
            source.engine.replace(doc);
        }
        let before = snapshot(&source);
        let ambient = engine::icc();
        engine::with_rgb_converter(std::rc::Rc::new(|_| [1.0, 0.0, 1.0]), || {
            let text = publish(&mut source);
            assert_eq!(
                Color::cmyk_pct(17.0, 33.0, 49.0, 65.0).to_rgb8(),
                [255, 0, 255],
                "outer scope restored after publication"
            );
            let xml = roxmltree::Document::parse(&text).unwrap();
            let vector = xml
                .descendants()
                .find(|node| node.attribute("id") == Some("4"))
                .unwrap();
            let Some(Color::Rgb { r, g, b: blue }) =
                Color::from_hex(vector.attribute("fill").unwrap())
            else {
                panic!("plain vector SVG fallback");
            };
            let fill = [r, g, blue].map(|v| (v * 255.0).round() as u8);
            let expected: Vec<f32> = Transform::new(
                &Profile::parse(&a.data).unwrap(),
                &Profile::srgb(),
                fixture::options().intent,
                false,
            )
            .unwrap()
            .transform(&[0.17, 0.33, 0.49, 0.65]);
            let expected: [u8; 3] = expected
                .into_iter()
                .map(|v| (v * 255.0).round() as u8)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let builtin = Color::cmyk_pct(17.0, 33.0, 49.0, 65.0)
                .to_rgb_f32_builtin()
                .map(|v| (v * 255.0).round() as u8);
            assert_ne!(
                expected, builtin,
                "the source ICC oracle must distinguish a generic fallback"
            );
            assert_ne!(
                expected,
                [255, 0, 255],
                "the source ICC oracle must distinguish the ambient sentinel"
            );
            assert_eq!(
                fill,
                if builtin_engine { builtin } else { expected },
                "SVG vector must use the source's chosen engine, never the ambient sentinel"
            );
            let parsed = tracedraw_io::svg_import::parse(
                &text,
                &mut tracedraw_core::id::IdSource::default(),
            )
            .unwrap();
            let mut pending: Vec<_> = parsed.shapes.iter().collect();
            let mut pictures = Vec::new();
            while let Some(shape) = pending.pop() {
                match &shape.kind {
                    ShapeKind::Group { children } => pending.extend(children),
                    ShapeKind::ClipFrame { frame, contents } => {
                        pending.push(frame);
                        pending.extend(contents);
                    }
                    ShapeKind::Bitmap { .. } => pictures.push(shape),
                    _ => {}
                }
            }
            assert_eq!(pictures.len(), 1);
            let picture = pictures[0];
            let ShapeKind::Bitmap { png, .. } = &picture.kind else {
                unreachable!();
            };
            let rgba = tracedraw_io::picture::read(png).unwrap().rgba;
            let expected_picture = fixture::picture(if untagged && builtin_engine {
                None
            } else {
                Some(if untagged { &a } else { &b })
            });
            assert_eq!(
                rgba,
                fixture::raw_picture(&expected_picture).2,
                "SVG bitmap must respect its own ICC or the source default"
            );
        });
        assert_eq!(snapshot(&source), before);
        match (ambient, engine::icc()) {
            (Some(before), Some(after)) => assert!(std::sync::Arc::ptr_eq(&before, &after)),
            (None, None) => {}
            _ => panic!("ambient ICC engine was changed"),
        }
    }
}

#[test]
fn oversized_native_resource_and_object_counts_are_terminal_and_atomic() {
    let mut source = configured(Some(&fixture::profile(false)));
    let text = publish(&mut source);
    let resource_excess = mutate_json(&text, |value| {
        value["payload"]["document"]["inks"]=serde_json::Value::Array((0..crate::clipboard_wire::MAX_RESOURCES+1).map(|index|serde_json::json!({"id":index+1,"name":format!("Owned extra ink {index}"),"cmyk":[0.1,0.2,0.3,0.4]})).collect());
    });
    let object_excess = mutate_json(&text, |value| {
        let document = &mut value["payload"]["document"];
        let prototype = document["pages"][0]["layers"][0]["shapes"][1].clone();
        let shapes = document["pages"][0]["layers"][0]["shapes"]
            .as_array_mut()
            .unwrap();
        for index in 0..crate::clipboard_wire::MAX_SHAPES + 1 {
            let mut shape = prototype.clone();
            shape["id"] = serde_json::json!(100 + index);
            shape.as_object_mut().unwrap().remove("uid");
            shapes.push(shape);
        }
        document["ids"]["next"] = serde_json::json!(crate::clipboard_wire::MAX_SHAPES + 102);
    });
    for (case, oversized) in [
        ("resource count", resource_excess),
        ("object count", object_excess),
    ] {
        let error = crate::clipboard_wire::decode_svg(&oversized).err().unwrap();
        assert!(
            error.contains("limit"),
            "budget-specific rejection: {case}: {error}"
        );
        let mut target = destination(0);
        let before = snapshot(&target);
        assert!(target.try_paste_svg_text(&oversized).is_err());
        assert_eq!(snapshot(&target), before);
    }
}

fn packet_with_kind(text: &str, kind: ShapeKind) -> String {
    let table_flags = match &kind {
        ShapeKind::Table(table) => Some(
            table
                .cells
                .iter()
                .map(|cell| vec![false; cell.text.len()])
                .collect::<Vec<_>>(),
        ),
        _ => None,
    };
    mutate_json(text, |value| {
        value["payload"]["document"]["pages"][0]["layers"][0]["shapes"][1]["kind"] =
            serde_json::to_value(kind).unwrap();
        if let Some(spans) = table_flags {
            value["supplements"]["shapes"] =
                serde_json::json!([{"ordinal":1,"spans":spans,"flow":null}]);
        }
    })
}

fn packet_with_symbol_dag(text: &str, levels: usize) -> String {
    use tracedraw_core::Shape;
    use tracedraw_core::document::Symbol;
    let mut symbols = Vec::new();
    for index in 0..levels {
        let kind = if index == 0 {
            ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(0.0, 0.0, 1.0, 1.0),
                radius: 0.0,
                corners: None,
            }
        } else {
            ShapeKind::Group {
                children: (0..2)
                    .map(|child| {
                        Shape::new(
                            ShapeId(100 + index as u64 * 3 + child),
                            ShapeKind::SymbolInstance { index: index - 1 },
                        )
                    })
                    .collect(),
            }
        };
        symbols.push(Symbol {
            name: format!("Inbound DAG {index}"),
            shape: Shape::new(ShapeId(99 + index as u64 * 3), kind),
        });
    }
    mutate_json(text, |value| {
        let doc = &mut value["payload"]["document"];
        doc["symbols"] = serde_json::to_value(symbols).unwrap();
        doc["pages"][0]["layers"][0]["shapes"][1]["kind"] =
            serde_json::to_value(ShapeKind::SymbolInstance { index: levels - 1 }).unwrap();
        doc["ids"]["next"] = serde_json::json!(100 + levels * 3);
    })
}

fn wire_table() -> tracedraw_core::document::Table {
    use tracedraw_core::document::{Table, TableCell};
    Table {
        rect: tracedraw_core::Rect::new(0.0, 0.0, 10.0, 10.0),
        col_widths: vec![5.0, 5.0],
        row_heights: vec![5.0, 5.0],
        cells: (0..4)
            .map(|index| TableCell {
                row: index / 2,
                col: index % 2,
                row_span: 1,
                col_span: 1,
                text: Vec::new(),
                fill: None,
                align: Default::default(),
                borders: Default::default(),
            })
            .collect(),
        border: None,
        cell_fill: tracedraw_core::Fill::None,
        padding: 0.5,
        margins: None,
        spacing: None,
    }
}

#[test]
fn incoming_dag_and_tiny_scalar_expansion_attacks_are_rejected_before_paste_in_known_children() {
    let mut source = configured(Some(&fixture::profile(false)));
    let text = publish(&mut source);
    let polygon = |points| ShapeKind::Polygon {
        rect: tracedraw_core::Rect::new(0.0, 0.0, 10.0, 10.0),
        points,
        sharpness: 0.0,
        complex: None,
        twist: 0.0,
    };
    // These are receiver tests. They deliberately bypass encode_svg, whose
    // sender-side guard must not conceal an unprotected receiving path.
    for positive in [
        packet_with_kind(&text, polygon(17)),
        packet_with_symbol_dag(&text, 3),
        packet_with_kind(&text, ShapeKind::Table(wire_table())),
    ] {
        crate::clipboard_wire::decode_svg(&positive)
            .unwrap()
            .unwrap();
        let mut target = destination(1);
        let before = snapshot(&target);
        target.try_paste_svg_text(&positive).unwrap();
        target.undo();
        after_undo(&target, &before);
    }
    let mut overflowing_span = wire_table();
    overflowing_span.cells[0].row = u32::MAX;
    overflowing_span.cells[0].row_span = u32::MAX;
    let mut too_many_grid_slots = wire_table();
    too_many_grid_slots.row_heights = vec![1.0; 1024];
    too_many_grid_slots.col_widths = vec![1.0; 1024];
    for (mode, hostile) in [
        (
            "hostile-wire-polygon",
            packet_with_kind(&text, polygon(u32::MAX)),
        ),
        ("hostile-wire-dag", packet_with_symbol_dag(&text, 40)),
        (
            "hostile-wire-table-span",
            packet_with_kind(&text, ShapeKind::Table(overflowing_span)),
        ),
        (
            "hostile-wire-table-product",
            packet_with_kind(&text, ShapeKind::Table(too_many_grid_slots)),
        ),
    ] {
        spawn_known_child(mode, Some(&hostile));
    }
}

fn finish_copy_job(app: &mut App, reject: bool) -> Result<String, String> {
    let began = std::time::Instant::now();
    let mut published = None;
    while app.clipboard_copy_job.is_some() {
        if began.elapsed() > std::time::Duration::from_secs(30) {
            panic!("known clipboard preparation did not settle within 30 seconds");
        }
        app.poll_clipboard_copy_with_publisher(|text| {
            published = Some(text.to_owned());
            if reject {
                Err("Injected asynchronous publication rejection".into())
            } else {
                Ok(())
            }
        })?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    published.ok_or_else(|| "copy job ended without publication".into())
}

#[test]
fn asynchronous_copy_and_cut_publish_real_bytes_and_delete_only_after_success() {
    for cut in [false, true] {
        let mut source = configured(Some(&fixture::profile(false)));
        let before = snapshot(&source);
        assert!(source.start_clipboard_copy(cut).unwrap());
        assert_eq!(
            snapshot(&source),
            before,
            "enqueue must not edit the document"
        );
        let text = finish_copy_job(&mut source, false).unwrap();
        let mut target = destination(0);
        target.try_paste_svg_text(&text).unwrap();
        assert_eq!(target.selection.len(), before.selection.len());
        if cut {
            assert!(source.doc().pages[0].layers[0].shapes.is_empty());
            assert_eq!(
                source.engine.history_labels().0.len(),
                before.history.0.len() + 1
            );
            let after = source.doc().clone();
            source.undo();
            assert_eq!(source.doc(), &before.document);
            assert_eq!(source.selection, before.selection);
            source.redo();
            assert_eq!(source.doc(), &after);
        } else {
            assert_eq!(snapshot(&source), before);
        }
    }
}

#[test]
fn asynchronous_cut_rejects_stale_sources_cancellation_and_publication_failure_atomically() {
    for reason in ["selection", "edit", "colour", "caret", "cancel", "provider"] {
        let mut app = configured(Some(&fixture::profile(false)));
        publish(&mut app);
        if reason == "caret" {
            app.start_text(Point::new(1.0, 50.0), None);
            app.text_insert("Caret");
            app.selection = vec![ShapeId(3), ShapeId(4)];
        }
        assert!(app.start_clipboard_copy(true).unwrap());
        match reason {
            "selection" => app.selection.reverse(),
            "edit" => {
                app.engine
                    .run(&Command::SetTitle {
                        title: "Changed while preparing".into(),
                    })
                    .unwrap();
            }
            "colour" => app.settings.color.import_cmyk = 2,
            "caret" => app.text_edit.as_mut().unwrap().caret = 0,
            "cancel" => app.cancel_clipboard_copy(),
            "provider" => {}
            _ => unreachable!(),
        }
        let before = snapshot(&app);
        let clipboard = format!("{:?}", app.clipboard);
        let result = finish_copy_job(&mut app, reason == "provider");
        assert!(result.is_err(), "{reason}");
        if reason != "provider" && reason != "cancel" {
            assert!(result.unwrap_err().contains("source changed"), "{reason}");
        }
        assert_eq!(snapshot(&app), before, "{reason}");
        assert_eq!(format!("{:?}", app.clipboard), clipboard, "{reason}");
    }
}

#[test]
fn asynchronous_cut_checks_an_injected_external_sequence_before_publishing_or_deleting() {
    use std::time::{Duration, Instant};
    for current in [Some(101u32), Some(102u32), None] {
        let mut source = configured(Some(&fixture::profile(false)));
        publish(&mut source);
        let before = snapshot(&source);
        let previous_clipboard = format!("{:?}", source.clipboard);
        assert!(
            source
                .start_clipboard_copy_with_sequence(true, Some(101))
                .unwrap()
        );
        let began = Instant::now();
        let mut publications = Vec::new();
        let mut error = None;
        while source.clipboard_copy_job.is_some() && began.elapsed() < Duration::from_secs(30) {
            match source.poll_clipboard_copy_with_sequence_and_publisher(current, |text| {
                publications.push(text.to_owned());
                Ok(())
            }) {
                Ok(_) => {}
                Err(message) => {
                    error = Some(message);
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(
            source.clipboard_copy_job.is_none(),
            "sequence {current:?} did not settle"
        );
        if current == Some(101) {
            assert!(error.is_none(), "{error:?}");
            assert_eq!(publications.len(), 1);
            assert!(source.doc().pages[0].layers[0].shapes.is_empty());
            let mut target = destination(0);
            target.try_paste_svg_text(&publications[0]).unwrap();
            assert_eq!(target.selection.len(), before.selection.len());
            source.undo();
            assert_eq!(source.doc(), &before.document);
            assert_eq!(source.selection, before.selection);
        } else {
            assert!(
                error.is_some(),
                "changed or unreadable sequence must reject"
            );
            assert!(
                publications.is_empty(),
                "provider must never receive stale bytes"
            );
            assert_eq!(snapshot(&source), before);
            assert_eq!(format!("{:?}", source.clipboard), previous_clipboard);
        }
    }
}

#[test]
fn embedded_font_dependencies_use_real_names_instead_of_display_labels_or_fallbacks() {
    use tracedraw_core::document::DocumentFont;
    let used = DocumentFont::new(
        "Display label differs from the real family",
        include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").to_vec(),
    );
    // Its display label deliberately lies in the opposite direction: neither
    // the label nor a fallback family may make this unrelated file a dependency.
    let unused = DocumentFont::new(
        "TestSymbol",
        include_bytes!("../../../crates/text/testdata/Quicksand-VF.ttf").to_vec(),
    );
    let empty = tracedraw_text::FontSystem::empty();
    assert!(empty.families().is_empty());
    assert!(!empty.has_family("TestSymbol"));
    let unused_probe = tracedraw_text::FontSystem::empty();
    assert!(
        !unused_probe
            .add_document_fonts(std::slice::from_ref(&unused))
            .is_empty()
    );
    assert!(unused_probe.has_family("Quicksand"));
    assert!(!unused_probe.has_family("TestSymbol"));
    let used_probe = tracedraw_text::FontSystem::empty();
    assert!(
        !used_probe
            .add_document_fonts(std::slice::from_ref(&used))
            .is_empty()
    );
    assert!(used_probe.has_family("TestSymbol"));
    let mut source = text_source("AB");
    let mut doc = source.doc().clone();
    let ShapeKind::Text { spans, .. } = &mut doc.find_shape_mut(source.selection[0]).unwrap().kind
    else {
        unreachable!()
    };
    for span in spans {
        span.font_family = "TestSymbol".into();
    }
    doc.fonts = vec![used.clone(), unused];
    source.engine.replace(doc);
    let before = snapshot(&source);
    let text = publish(&mut source);
    let decoded = crate::clipboard_wire::decode_svg(&text).unwrap().unwrap();
    assert_eq!(decoded.document.fonts, vec![used.clone()]);
    let mut target = destination(0);
    let destination_before = snapshot(&target);
    target.try_paste_svg_text(&text).unwrap();
    assert_eq!(target.doc().fonts, vec![used]);
    let after = target.doc().clone();
    let selection = target.selection.clone();
    target.undo();
    after_undo(&target, &destination_before);
    target.redo();
    assert_eq!(target.doc(), &after);
    assert_eq!(target.selection, selection);
    assert_eq!(snapshot(&source), before);
}

#[test]
fn active_second_page_sets_native_and_fallback_viewport_for_copy_jobs() {
    let mut source = configured(Some(&fixture::profile(false)));
    let mut doc = source.doc().clone();
    let mut second = doc.pages[0].clone();
    second.id = doc.ids_mut().page();
    second.layers[0].id = doc.ids_mut().layer();
    second.size = tracedraw_core::Size::new(210.0, 297.0);
    let mut selected = second.layers[0].shapes.remove(1);
    selected.kind = ShapeKind::Rect {
        rect: tracedraw_core::Rect::new(150.0, 220.0, 180.0, 260.0),
        radius: 0.0,
        corners: None,
    };
    second.layers[0].shapes = vec![selected];
    let page = second.id;
    doc.pages[0].layers[0].shapes.clear();
    doc.pages.push(second);
    source.engine.replace(doc);
    source.page = page;
    source.selection = vec![ShapeId(4)];
    let before = snapshot(&source);
    for asynchronous in [false, true] {
        let text = if asynchronous {
            source.start_clipboard_copy(false).unwrap();
            finish_copy_job(&mut source, false).unwrap()
        } else {
            publish(&mut source)
        };
        let decoded = crate::clipboard_wire::decode_svg(&text).unwrap().unwrap();
        assert_eq!(
            decoded.document.pages[0].size,
            tracedraw_core::Size::new(210.0, 297.0)
        );
        assert_eq!(
            decoded.document.pages[0].layers[0].shapes[0].kind,
            source.doc().find_shape(ShapeId(4)).unwrap().kind,
            "the complete rectangle outside the first viewport is preserved"
        );
        let svg = roxmltree::Document::parse(&text).unwrap();
        let root = svg.root_element();
        assert_eq!(root.attribute("width"), Some("210mm"));
        assert_eq!(root.attribute("height"), Some("297mm"));
        let view: Vec<f64> = root
            .attribute("viewBox")
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(view, vec![0.0, 0.0, 210.0, 297.0]);
        let geometry = svg
            .descendants()
            .find(|node| node.attribute("id") == Some("4"))
            .unwrap();
        let path =
            tracedraw_core::geometry::BezPath::from_svg(geometry.attribute("d").unwrap()).unwrap();
        use tracedraw_core::geometry::Shape as _;
        assert_eq!(
            path.bounding_box(),
            tracedraw_core::Rect::new(150.0, 37.0, 180.0, 77.0)
        );
        assert_eq!(snapshot(&source), before);
    }
}

#[test]
#[ignore = "records real 512-square copy/paste jobs and a 5000-object unselected document"]
fn large_wire_jobs_record_virtual_frames_and_full_inks_preview_and_source_preservation() {
    use std::time::{Duration, Instant};
    let output = directory("async-512");
    let a = fixture::profile(false);
    let b = fixture::profile(true);
    let generation = Instant::now();
    let pixels: Vec<u8> = (0..512 * 512usize)
        .flat_map(|index| {
            [
                index as u8,
                (index >> 8) as u8,
                (((index * 37) >> 4) + (index >> 16) * 61) as u8,
                ((index * 19) >> 7) as u8,
                [0, 1, 127, 255][index % 4],
            ]
        })
        .collect();
    let bytes =
        tracedraw_io::cmyk_bitmap::cmyka_tiff(512, 512, &pixels, 300.0, Some(a.data.as_slice()))
            .unwrap();
    let mut source = configured(Some(&a));
    let mut doc = source.doc().clone();
    let mut bitmap = tracedraw_io::raster::document_from_bytes(&bytes, "Owned async 512")
        .unwrap()
        .pages[0]
        .layers[0]
        .shapes
        .remove(0);
    bitmap.id = ShapeId(3);
    bitmap.name = Some("Owned large wire picture".into());
    doc.pages[0].layers[0].shapes[0] = bitmap;
    for _ in 0..5000 {
        let mut shape = doc.pages[0].layers[0].shapes[1].clone();
        shape.id = doc.ids_mut().shape();
        shape.uid = None;
        doc.pages[0].layers[0].shapes.push(shape);
    }
    source.engine.replace(doc);
    source.selection = vec![ShapeId(3), ShapeId(4)];
    let fixture_ms = generation.elapsed().as_secs_f64() * 1000.0;
    let snapshot_began = Instant::now();
    let before = snapshot(&source);
    let independent_snapshot_ms = snapshot_began.elapsed().as_secs_f64() * 1000.0;
    let ctx = egui::Context::default();
    source.color_import_context = Some(ctx.clone());
    let start = Instant::now();
    source.start_clipboard_copy(false).unwrap();
    let copy_enqueue_ms = start.elapsed().as_secs_f64() * 1000.0;
    let waiting = Instant::now();
    let mut copy_frames = Vec::new();
    let mut text = None;
    let mut publication_ms = None;
    while source.clipboard_copy_job.is_some() && waiting.elapsed() < Duration::from_secs(180) {
        let frame = Instant::now();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("Independent clipboard copy worker measurement");
            source
                .poll_clipboard_copy_with_publisher(|bytes| {
                    text = Some(bytes.to_owned());
                    Ok(())
                })
                .unwrap();
        });
        output.textures_delta.clear();
        let elapsed = frame.elapsed().as_secs_f64() * 1000.0;
        if source.clipboard_copy_job.is_some() {
            copy_frames.push(elapsed);
        } else {
            publication_ms = Some(elapsed);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let copy_wall_ms = waiting.elapsed().as_secs_f64() * 1000.0;
    assert!(source.clipboard_copy_job.is_none());
    assert!(copy_frames.len() >= 2);
    assert_eq!(snapshot(&source), before);
    let text = text.unwrap();
    std::fs::write(output.join("published.svg"), &text).unwrap();
    let decoded = crate::clipboard_wire::decode_svg(&text).unwrap().unwrap();
    assert_eq!(decoded.document.pages[0].layers[0].shapes.len(), 2);
    fixture::assert_picture(fixture::bitmaps(&decoded.document)[0], &pixels, Some(&a));
    let mut target = destination(0);
    let target_before = snapshot(&target);
    let old_bitmap = fixture::bitmaps(target.doc())[0].id;
    target.color_import_context = Some(ctx.clone());
    let start = Instant::now();
    target.dispatch_wire_paste(text.clone()).unwrap();
    let paste_enqueue_ms = start.elapsed().as_secs_f64() * 1000.0;
    let waiting = Instant::now();
    let mut paste_frames = Vec::new();
    let mut commit_ms = None;
    while target.color_import_job.is_some() && waiting.elapsed() < Duration::from_secs(180) {
        let frame = Instant::now();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("Independent clipboard paste worker measurement");
            target.poll_color_import();
        });
        output.textures_delta.clear();
        let elapsed = frame.elapsed().as_secs_f64() * 1000.0;
        if target.color_import_job.is_some() {
            paste_frames.push(elapsed);
            assert_eq!(target.engine.history_labels(), target_before.history);
        } else {
            commit_ms = Some(elapsed);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let paste_wall_ms = waiting.elapsed().as_secs_f64() * 1000.0;
    assert!(target.color_import_job.is_none());
    assert!(paste_frames.len() >= 2);
    assert_eq!(
        target.engine.history_labels().0.len(),
        target_before.history.0.len() + 1
    );
    // Retain the complete oracle outside measured frames, with one pair of
    // parsed profiles instead of rebuilding them for every pixel.
    let options = fixture::options();
    assert!(!options.preserve_pure_black);
    let oracle_a = tracedraw_core::icc::Profile::parse(&a.data).unwrap();
    let oracle_b = tracedraw_core::icc::Profile::parse(&b.data).unwrap();
    let oracle = tracedraw_core::icc::Transform::new(
        &oracle_a,
        &oracle_b,
        options.intent,
        options.black_point_compensation,
    )
    .unwrap();
    let mut expected = pixels.clone();
    for pixel in expected.chunks_exact_mut(5) {
        let input: [f32; 4] = std::array::from_fn(|index| f32::from(pixel[index]) / 255.0);
        for (byte, value) in pixel[..4].iter_mut().zip(oracle.transform(&input)) {
            *byte = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    for index in [0usize, 1, 127, 65536, 512 * 512 - 1] {
        assert_eq!(
            &expected[index * 5..index * 5 + 5],
            fixture::expected_samples(
                &pixels[index * 5..index * 5 + 5],
                Some(&a),
                Some(&b),
                fixture::options()
            ),
        );
    }
    let pictures = fixture::bitmaps(target.doc());
    assert_eq!(pictures.len(), 2);
    let incoming = pictures
        .into_iter()
        .find(|picture| picture.id != old_bitmap)
        .unwrap();
    fixture::assert_picture(incoming, &expected, Some(&b));
    let after = target.doc().clone();
    let selection = target.selection.clone();
    target.undo();
    after_undo(&target, &target_before);
    target.redo();
    assert_eq!(target.doc(), &after);
    assert_eq!(target.selection, selection);
    assert_eq!(snapshot(&source), before);
    copy_frames.sort_by(f64::total_cmp);
    paste_frames.sort_by(f64::total_cmp);
    let result = serde_json::json!({
        "selected_roots":2,"unselected_roots":5000,"pixels":512*512,
        "fixture_generation_ms":fixture_ms,"independent_snapshot_ms":independent_snapshot_ms,
        "copy_enqueue_including_production_document_clone_ms":copy_enqueue_ms,
        "copy_worker_and_poll_wall_ms":copy_wall_ms,"copy_publication_virtual_frame_ms":publication_ms,
        "copy_wait_frames":copy_frames.len(),"copy_frame_median_ms":copy_frames[copy_frames.len()/2],
        "copy_frame_p95_ms":copy_frames[copy_frames.len()*95/100],"copy_frame_max_ms":copy_frames.last(),
        "paste_enqueue_including_target_snapshot_ms":paste_enqueue_ms,
        "paste_decode_convert_and_poll_wall_ms":paste_wall_ms,"paste_commit_virtual_frame_ms":commit_ms,
        "paste_wait_frames":paste_frames.len(),"paste_frame_median_ms":paste_frames[paste_frames.len()/2],
        "paste_frame_p95_ms":paste_frames[paste_frames.len()*95/100],"paste_frame_max_ms":paste_frames.last(),
        "transport_bytes":text.len(),"all_1310720_ink_alpha_bytes_and_1048576_preview_bytes_checked":true,
        "source_intact":true,"one_undo_and_redo":true,
        "scope":"Minimal virtual egui label and actual worker polling; no desktop or full editor smoothness claim"
    });
    std::fs::write(
        output.join("timings.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("LUMEN_CLIPBOARD_WORKER_512 {result}");
}

#[test]
fn asynchronous_paste_here_commits_the_requested_center_in_the_same_single_undo() {
    use std::time::{Duration, Instant};
    let mut source = configured(Some(&fixture::profile(false)));
    let source_before = snapshot(&source);
    let text = publish(&mut source);
    let ctx = egui::Context::default();
    let mut target = destination(0);
    target.color_import_context = Some(ctx.clone());
    let before = snapshot(&target);
    let at = Point::new(85.125, 55.875);
    target.paste_handed(crate::clipboard::Handed {
        text: Some(text),
        files: Vec::new(),
        from_menu: true,
        refused: false,
        at: Some(at),
    });
    assert!(
        target.color_import_job.is_some(),
        "exercise the actual asynchronous receiving path"
    );
    assert_eq!(
        snapshot(&target),
        before,
        "no premature move of the old selection"
    );
    let waiting = Instant::now();
    while target.color_import_job.is_some() && waiting.elapsed() < Duration::from_secs(30) {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui.label("Paste here transaction");
            target.poll_color_import();
        });
        output.textures_delta.clear();
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(target.color_import_job.is_none());
    let center = target.selection_bounds().unwrap().center();
    assert!(
        (center.x - at.x).abs() < 1e-9 && (center.y - at.y).abs() < 1e-9,
        "{center:?} vs {at:?}",
    );
    assert_eq!(
        target.engine.history_labels().0.len(),
        before.history.0.len() + 1
    );
    let after = target.doc().clone();
    let selected = target.selection.clone();
    target.undo();
    after_undo(&target, &before);
    target.redo();
    assert_eq!(target.doc(), &after);
    assert_eq!(target.selection, selected);
    assert_eq!(snapshot(&source), source_before);
}
