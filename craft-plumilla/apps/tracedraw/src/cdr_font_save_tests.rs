use super::*;
use tracedraw_core::{Document, DocumentFont};

const FONT: &[u8] = include_bytes!("../../../crates/text/testdata/TestSymbol.ttf");

fn fixture() -> Document {
    let mut doc = Document::new("Carried font save", Size::new(30.0, 30.0));
    doc.fonts
        .push(DocumentFont::new("TestSymbol", FONT.to_vec()));
    let text = Shape::new(
        doc.ids_mut().shape(),
        ShapeKind::Text {
            spans: vec![TextSpan::new("AB", "TestSymbol", 12.0)],
            origin: Point::new(5.0, 15.0),
            frame: None,
            align: tracedraw_core::TextAlign::Left,
            para: Default::default(),
            on_path: None,
        },
    );
    doc.pages[0].layers[0].shapes.push(text);
    doc
}

#[test]
fn mcp_save_keeps_a_carried_font_after_independent_cdr_reopen() {
    let mut app = App::headless();
    app.open_document(fixture(), None);
    let faces = cdr_font_faces(app.doc());
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0].family, "TestSymbol");
    assert_eq!(faces[0].data.as_ref().unwrap().as_slice(), FONT);
    assert_eq!(
        tracedraw_cdr::write::embedded_size(&faces),
        FONT.len() as u64
    );
    let folder = std::env::temp_dir().join(format!(
        "craft-cdr-carried-font-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    std::fs::create_dir(&folder).unwrap();
    let path = folder.join("carried.cdr");
    crate::mcp::call_tool(
        &mut app,
        "save",
        &serde_json::json!({
            "path": path.to_string_lossy(), "version": 21,
        }),
    )
    .expect("save through the real automation handler");
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&folder).unwrap();
    let (reopened, report) = tracedraw_cdr::open_bytes(&bytes, "independent font reopen").unwrap();
    assert_eq!(app.doc().fonts[0].data.as_slice(), FONT);
    assert_eq!(
        reopened.fonts.len(),
        1,
        "saved CDR must carry its font: {:?}",
        report.warnings
    );
    assert_eq!(reopened.fonts[0].data.as_slice(), FONT);
    let independent = tracedraw_text::FontSystem::empty();
    assert!(!independent.has_family("TestSymbol"));
    independent.add_document_fonts(&reopened.fonts);
    assert!(independent.is_document_family("TestSymbol"));
    assert_eq!(
        independent
            .face_file("TestSymbol", false, false)
            .unwrap()
            .as_slice(),
        FONT
    );
    assert!(independent.substitute("TestSymbol").is_none());
}

#[test]
fn explicit_no_embedding_and_restricted_fonts_stay_excluded() {
    let doc = fixture();
    tracedraw_text::fonts().add_document_fonts(&doc.fonts);
    let mut previews = cdr_previews(&doc);
    previews.embed_fonts = false;
    let bytes = tracedraw_cdr::write::document_to_cdr_version_with(&doc, 21, &previews);
    let (reopened, _) = tracedraw_cdr::open_bytes(&bytes, "explicit no embedding").unwrap();
    assert!(
        reopened.fonts.is_empty(),
        "an explicit unchecked option stays authoritative"
    );
    let mut no_carried = doc.clone();
    no_carried.fonts.clear();
    assert!(
        !cdr_previews(&no_carried).embed_fonts,
        "automatic save does not start embedding installed fonts"
    );

    let mut restricted = FONT.to_vec();
    let count = u16::from_be_bytes(restricted[4..6].try_into().unwrap()) as usize;
    let record = restricted[12..12 + count * 16]
        .chunks_exact(16)
        .find(|record| &record[..4] == b"OS/2")
        .unwrap();
    let offset = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
    restricted[offset + 8..offset + 10].copy_from_slice(&2u16.to_be_bytes());
    let fonts = tracedraw_text::FontSystem::empty();
    fonts.add_fonts(vec![restricted.clone()]);
    previews.fonts = cdr_font_faces_from(&doc, &fonts);
    assert_eq!(
        previews.fonts[0].data.as_ref().unwrap().as_slice(),
        restricted
    );
    assert_eq!(tracedraw_cdr::write::embedded_size(&previews.fonts), 0);
    assert_eq!(
        tracedraw_cdr::write::left_out(&previews.fonts),
        vec![("TestSymbol".into(), tracedraw_cdr::write::LeftOut::Licence),]
    );
    previews.embed_fonts = true;
    let bytes = tracedraw_cdr::write::document_to_cdr_version_with(&doc, 21, &previews);
    let (reopened, _) = tracedraw_cdr::open_bytes(&bytes, "restricted font").unwrap();
    assert!(
        reopened.fonts.is_empty(),
        "automatic embedding cannot bypass the licence gate"
    );
}
