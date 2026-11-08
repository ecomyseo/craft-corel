//! Hindi through a PDF and back (#599): text written by Publish to PDF
//! with its font comes in as the typed words and shapes to the same
//! glyphs as the words themselves. A test binary of its own: the
//! Devanagari test font joins the process's font system here only.

use tracedraw_core::{
    document::text_outline::TextRequest, Document, ParagraphStyle, Point, Shape, ShapeKind, Size,
    TextAlign, TextSpan,
};

const NOTO_DEVA: &[u8] = include_bytes!("../../text/testdata/NotoSansDevanagari-Regular.ttf");
const CORPUS: &str = include_str!("../../text/testdata/devanagari-corpus.txt");
const DEVA: &str = "Noto Sans Devanagari";

fn corpus() -> Vec<&'static str> {
    CORPUS
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

fn fonts() -> &'static tracedraw_text::FontSystem {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        tracedraw_text::add_font_data(NOTO_DEVA.to_vec());
        tracedraw_text::install();
    });
    tracedraw_text::fonts()
}

/// The glyph ids `text` lays out to in the Devanagari font.
fn gids(text: &str) -> Vec<u16> {
    let spans = [TextSpan::new(text, DEVA, 24.0)];
    let para = ParagraphStyle::default();
    fonts()
        .glyph_layout(&TextRequest {
            spans: &spans,
            frame: None,
            align: TextAlign::Left,
            para: &para,
            on_path: None,
        })
        .glyphs
        .iter()
        .map(|g| g.glyph_id)
        .collect()
}

/// The texts of the imported file's text objects, in order.
fn imported_texts(bytes: &[u8]) -> Vec<String> {
    let mut ids = tracedraw_core::id::IdSource::default();
    let imp = tracedraw_io::pdf_import::parse(bytes, &mut ids).expect("reads");
    imp.pages
        .iter()
        .flat_map(|p| p.shapes.iter())
        .filter_map(|s| match &s.kind {
            ShapeKind::Text { spans, .. } => {
                Some(spans.iter().map(|s| s.text.as_str()).collect::<String>())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn hindi_exported_with_its_font_imports_as_the_typed_words() {
    let _ = fonts();
    let words = corpus();
    let mut doc = Document::new("t", Size::new(200.0, 20.0 + 12.0 * words.len() as f64));
    let layer = doc.pages[0].layers[0].id;
    for (i, w) in words.iter().enumerate() {
        let id = doc.ids_mut().shape();
        let mut s = Shape::new(
            id,
            ShapeKind::Text {
                spans: vec![TextSpan::new(*w, DEVA, 24.0)],
                origin: Point::new(10.0, 10.0 + 12.0 * i as f64),
                frame: None,
                align: TextAlign::Left,
                para: ParagraphStyle::default(),
                on_path: None,
            },
        );
        s.fill = tracedraw_core::Fill::Solid(tracedraw_core::Color::BLACK);
        if let Ok(l) = doc.layer_mut(layer) {
            l.shapes.push(s);
        }
    }
    let (bytes, report) =
        tracedraw_io::pdf::document_to_pdf_report(&doc, &tracedraw_io::pdf::PdfOptions::default());
    assert!(report.restricted.is_empty());
    let texts = imported_texts(&bytes);
    let bad: Vec<String> = words
        .iter()
        .filter(|w| !texts.iter().any(|t| t == *w))
        .map(|w| w.to_string())
        .collect();
    assert!(bad.is_empty(), "missing {bad:?} in {texts:?}");
    for t in &texts {
        assert_eq!(gids(t), gids(t.trim()), "{t}");
    }
}

/// Latin text, one glyph for each character, goes without marked
/// content: only clusters that need it get an ActualText.
#[test]
fn latin_text_exported_has_no_actual_text() {
    let _ = fonts();
    let mut doc = Document::new("t", Size::new(100.0, 40.0));
    let layer = doc.pages[0].layers[0].id;
    let id = doc.ids_mut().shape();
    let mut s = Shape::new(
        id,
        ShapeKind::Text {
            spans: vec![TextSpan::new("Hello World", "DejaVu Sans", 24.0)],
            origin: Point::new(10.0, 20.0),
            frame: None,
            align: TextAlign::Left,
            para: ParagraphStyle::default(),
            on_path: None,
        },
    );
    s.fill = tracedraw_core::Fill::Solid(tracedraw_core::Color::BLACK);
    if let Ok(l) = doc.layer_mut(layer) {
        l.shapes.push(s);
    }
    let bytes = tracedraw_io::pdf::document_to_pdf(&doc);
    let pdf = lopdf::Document::load_mem(&bytes).expect("parses");
    let mut ops = Vec::new();
    for (_, page) in pdf.get_pages() {
        let content = pdf.get_page_content(page);
        let content = lopdf::content::Content::decode(&content).expect("decodes");
        ops.extend(content.operations.into_iter().map(|o| o.operator));
    }
    assert!(ops.iter().any(|o| o == "TJ"), "{ops:?}");
    assert!(!ops.iter().any(|o| o == "BDC"), "{ops:?}");
    assert_eq!(imported_texts(&bytes), ["Hello World"]);
}

/// The glyphs `text` lays out to in the Devanagari font, in drawn order,
/// each with the characters the shaper gives it (the first glyph of a
/// cluster holds the cluster's characters, the others none).
fn glyphs(text: &str) -> Vec<(u16, String)> {
    let spans = [TextSpan::new(text, DEVA, 24.0)];
    let para = ParagraphStyle::default();
    fonts()
        .glyph_layout(&TextRequest {
            spans: &spans,
            frame: None,
            align: TextAlign::Left,
            para: &para,
            on_path: None,
        })
        .glyphs
        .into_iter()
        .map(|g| (g.glyph_id, g.text))
        .collect()
}

/// A PDF of `words`, one a line, in the Devanagari font with two-byte
/// glyph codes, whose ToUnicode map is scrambled the way an office word
/// processor's PDFs of Hindi were seen to be (#1806): the glyphs of each
/// cluster, in drawn order, are mapped to its characters in typed order,
/// every glyph after the first to one character and the first to the
/// rest, and a glyph keeps the text of the first cluster it appears in.
fn scrambled_pdf(words: &[&str]) -> Vec<u8> {
    use lopdf::{dictionary, Object, Stream};
    let face = ttf_parser::Face::parse(NOTO_DEVA, 0).expect("font");
    let mut tounicode: std::collections::BTreeMap<u16, String> = Default::default();
    let mut content = String::from("BT /F1 12 Tf\n");
    let mut used = std::collections::BTreeSet::new();
    for (i, w) in words.iter().enumerate() {
        let gl = glyphs(w);
        // Clusters: a glyph with characters and the glyphs after it
        // without.
        let mut clusters: Vec<(String, Vec<u16>)> = Vec::new();
        for (gid, text) in &gl {
            match clusters.last_mut() {
                Some((_, ids)) if text.is_empty() => ids.push(*gid),
                _ => clusters.push((text.clone(), vec![*gid])),
            }
        }
        for (text, ids) in &clusters {
            let chars: Vec<char> = text.chars().collect();
            let (m, k) = (chars.len(), ids.len());
            for (j, gid) in ids.iter().enumerate() {
                let part: String = if m < k {
                    chars.get(j).map(|c| c.to_string()).unwrap_or_default()
                } else if j == 0 {
                    chars[..m - k + 1].iter().collect()
                } else {
                    chars[m - k + j].to_string()
                };
                tounicode.entry(*gid).or_insert(part);
            }
        }
        let hex: String = gl.iter().map(|(g, _)| format!("{g:04X}")).collect();
        used.extend(gl.iter().map(|(g, _)| *g));
        let y = 800 - 20 * i as i32;
        content.push_str(&format!("1 0 0 1 20 {y} Tm <{hex}> Tj\n"));
    }
    content.push_str("ET\n");
    let utf16 = |s: &str| -> String { s.encode_utf16().map(|u| format!("{u:04X}")).collect() };
    let entries: String = tounicode
        .iter()
        .filter(|(_, t)| !t.is_empty())
        .map(|(g, t)| format!("<{g:04X}> <{}>\n", utf16(t)))
        .collect();
    let cmap = format!(
        "/CIDInit /ProcSet findresource begin 12 dict begin begincmap\n\
         /CMapName /Adobe-Identity-UCS def /CMapType 2 def\n\
         1 begincodespacerange <0000> <FFFF> endcodespacerange\n\
         {} beginbfchar\n{entries}endbfchar\n\
         endcmap CMapName currentdict /CMap defineresource pop end end",
        entries.lines().count()
    );
    let upem = f64::from(face.units_per_em());
    let widths: Vec<Object> = used
        .iter()
        .flat_map(|g| {
            let w = f64::from(face.glyph_hor_advance(ttf_parser::GlyphId(*g)).unwrap_or(0));
            [
                Object::Integer(i64::from(*g)),
                Object::Array(vec![Object::Real((w * 1000.0 / upem) as f32)]),
            ]
        })
        .collect();
    let mut doc = lopdf::Document::with_version("1.7");
    let file = doc.add_object(Stream::new(dictionary! {}, NOTO_DEVA.to_vec()));
    let descriptor = doc.add_object(dictionary! {
        "Type" => "FontDescriptor", "FontName" => "Mangal", "Flags" => 32,
        "FontBBox" => vec![0.into(), (-300).into(), 1000.into(), 1000.into()],
        "ItalicAngle" => 0, "Ascent" => 1000, "Descent" => -300, "CapHeight" => 700,
        "StemV" => 80, "FontFile2" => file,
    });
    let cid = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" => "Mangal",
        "CIDSystemInfo" => dictionary! { "Registry" => Object::string_literal("Adobe"),
            "Ordering" => Object::string_literal("Identity"), "Supplement" => 0 },
        "FontDescriptor" => descriptor, "CIDToGIDMap" => "Identity", "W" => widths,
    });
    let to_unicode = doc.add_object(Stream::new(dictionary! {}, cmap.into_bytes()));
    let font = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type0", "BaseFont" => "Mangal",
        "Encoding" => "Identity-H", "DescendantFonts" => vec![cid.into()],
        "ToUnicode" => to_unicode,
    });
    let contents = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
    let pages_id = doc.new_object_id();
    let page = doc.add_object(dictionary! {
        "Type" => "Page", "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        "Contents" => contents,
        "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(
            dictionary! { "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1 },
        ),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    let mut out = Vec::new();
    doc.save_to(&mut out).expect("writes");
    out
}

/// Words of the reporter's file of #1806 (an office word processor's PDF
/// in another font, not committed) and of the lists sent with it, written
/// with a ToUnicode map scrambled as that file's is, come back as typed.
/// Read with the map as it is, 21 of these words came in wrong, such as
/// ऋषि as ऋषष, वर्ष as वषड, डॉक्टर as र्ॉक्टर and सहिष्णुता as सहहष्णुता.
#[test]
fn hindi_with_a_scrambled_tounicode_map_imports_as_the_typed_words() {
    let _ = fonts();
    let words = [
        "उर्मिला",
        "वार्ड",
        "ग्राम",
        "श्री",
        "श्रीमती",
        "संख्या",
        "अधीक्षक",
        "हस्ताक्षर",
        "राजस्व",
        "वार्षिक",
        "अर्धवार्षिक",
        "प्रमाणपत्र",
        "ऋतु",
        "ऋषि",
        "निर्गत",
        "वर्ष",
        "सहिष्णुता",
        "सम्प्रभुता",
        "समस्थानिक",
        "व्यवस्थित",
        "प्रोडक्शन",
        "डॉक्टर",
        "कॉल",
        "पश्चात",
        "पश्चिम",
        "प्रश्न",
        "उत्तर",
        "उत्तराखंड",
        "सुसज्जित",
        "सौजन्य",
        "सौभाग्य",
        "पुष्प",
        "दैत्य",
        "परमेश्वर",
        "प्रहरी",
        "प्राण",
        "प्रकार",
        "विश्वनाथ",
        "प्रस्तुति",
        "पार्श्व",
        "पार्थ",
        "शिल्प",
        "पाश्चात्य",
        "स्थानांतरण",
        "श्वसन",
        "श्वास",
        "श्रुति",
        "तृषा",
        "त्रिशा",
        "नृत्य",
        "वाद्ययंत्र",
        "व्याघ्र",
        "व्याकुल",
    ];
    let texts = imported_texts(&scrambled_pdf(&words));
    let bad: Vec<String> = words
        .iter()
        .zip(texts.iter().chain(std::iter::repeat(&String::new())))
        .filter(|(w, t)| *w != t)
        .map(|(w, t)| format!("{w} -> {t}"))
        .collect();
    assert!(bad.is_empty(), "{bad:#?}");
}

/// Hard words from the lists sent with #1806, written by TraceDraw with
/// their font and read back.
#[test]
fn hard_hindi_and_sanskrit_words_come_back_from_a_pdf() {
    let _ = fonts();
    let words = [
        "ऋष्यशृङ्गः",
        "मृच्छकटिकम्",
        "प्रज्ञाचक्षुः",
        "अविद्यास्मितारागद्वेषाभिनिवेशाः",
        "किंकर्तव्यविमूढ़",
        "प्रत्युत्पन्नमति",
        "उज्ज्वल",
        "बुद्धि",
        "विद्वज्जन",
        "राष्ट्राध्यक्ष",
        "श्रेष्ठ",
        "त्र्यंबकेश्वर",
        "क्षत्रिय",
        "उद्भ्रांत",
        "उच्छ्वास",
        "उच्छृंखल",
        "वत्स्य",
        "स्त्री",
        "चिह्न",
        "ब्रह्मर्षि",
        "पूर्वोत्तर",
        "पुष्पेन्द्र",
        "प्रतिनियुक्ति",
        "स्वास्थ्य",
        "दुःसाहस",
        "निःशुल्क",
        "हृदय",
        "सम्प्रदाय",
        "आर्द्र",
        "कार्त्स्न्य",
    ];
    let mut doc = Document::new("t", Size::new(200.0, 20.0 + 12.0 * words.len() as f64));
    let layer = doc.pages[0].layers[0].id;
    for (i, w) in words.iter().enumerate() {
        let id = doc.ids_mut().shape();
        let mut s = Shape::new(
            id,
            ShapeKind::Text {
                spans: vec![TextSpan::new(*w, DEVA, 24.0)],
                origin: Point::new(10.0, 10.0 + 12.0 * i as f64),
                frame: None,
                align: TextAlign::Left,
                para: ParagraphStyle::default(),
                on_path: None,
            },
        );
        s.fill = tracedraw_core::Fill::Solid(tracedraw_core::Color::BLACK);
        if let Ok(l) = doc.layer_mut(layer) {
            l.shapes.push(s);
        }
    }
    let bytes = tracedraw_io::pdf::document_to_pdf(&doc);
    let texts = imported_texts(&bytes);
    let bad: Vec<&&str> = words
        .iter()
        .filter(|w| !texts.iter().any(|t| t == **w))
        .collect();
    assert!(bad.is_empty(), "missing {bad:?} in {texts:?}");
}
