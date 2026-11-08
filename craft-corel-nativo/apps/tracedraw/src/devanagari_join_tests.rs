//! Hindi lines a PDF brings, joined into one paragraph (#599).

use tracedraw_core::{ParagraphStyle, Point, ShapeKind, TextAlign, TextSpan};

/// Hindi lines a PDF brings one object a line, joined into a paragraph:
/// typing at the start of the first line flows the words on to the next
/// lines, and no line starts inside a syllable (a vowel sign or the rest
/// of a conjunct on its own) (#599).
#[test]
fn joined_hindi_lines_flow_and_break_between_syllables() {
    tracedraw_text::install();
    let mut app = crate::app::App::headless();
    let lines = [
        "राष्ट्रीय विद्यालय में स्थिति स्पष्ट",
        "शुद्धि ज्ञान कृष्ण हृदय श्रीमती",
        "द्वारा उज्ज्वल संयुक्ताक्षर अर्धाक्षर",
    ];
    let mut ids = Vec::new();
    for (i, text) in lines.iter().enumerate() {
        let id = app
            .new_shape(ShapeKind::Text {
                spans: vec![TextSpan::new(*text, "DejaVu Sans", 10.0)],
                origin: Point::new(20.0, 200.0 - 6.0 * i as f64),
                frame: None,
                align: TextAlign::Left,
                para: ParagraphStyle::default(),
                on_path: None,
            })
            .expect("text");
        ids.push(id);
    }
    app.select(ids);
    app.join_text_lines();
    let id = app.selection[0];
    let s = app.doc().find_shape(id).expect("paragraph").clone();
    let (before, _) =
        crate::text_editing::layout_of_text_shape(&s, kurbo::Affine::IDENTITY).expect("layout");
    assert_eq!(before.lines.len(), 3, "{:?}", before.baselines);
    let mut kind = s.kind.clone();
    if let ShapeKind::Text { spans, .. } = &mut kind {
        spans[0].text.insert_str(0, "क्षेत्र त्रिभुवन स्त्री इन्द्रिय ");
    }
    app.run(tracedraw_core::Command::SetShapeKind { shape: id, kind });
    let s = app.doc().find_shape(id).expect("paragraph").clone();
    let (after, _) =
        crate::text_editing::layout_of_text_shape(&s, kurbo::Affine::IDENTITY).expect("layout");
    assert!(after.lines.len() > 3, "{:?}", after.baselines);
    let ShapeKind::Text { spans, .. } = &s.kind else {
        panic!("text");
    };
    let chars: Vec<char> = tracedraw_core::document::spans_text(spans)
        .chars()
        .collect();
    for l in &after.lines[1..] {
        assert_eq!(
            chars.get(l.start_char.wrapping_sub(1)),
            Some(&' '),
            "line at {}",
            l.start_char
        );
    }
}
