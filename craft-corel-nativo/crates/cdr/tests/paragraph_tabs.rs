//! Tab stops of paragraph text in `.cdr` files (#1852).
//!
//! Written as one artistic text a line (the default), a line is cut at
//! its tabs and each part placed where the stops put it, so a reader with
//! other stops shows the same. Written as paragraph frames (version 16
//! and later style strings), the stops go with the paragraph and come
//! back with their kinds and positions.
//!
//! The line breaking is a stand-in (registered process-wide, so this test
//! binary is its own): one line a paragraph, every character 2 mm wide, a
//! tab going to the paragraph's first stop past it (a right stop ending
//! the text after it there).

use std::io::{Read as _, Write as _};

use tracedraw_core::document::text_outline::{self, Line, TextRequest};
use tracedraw_core::{
    Affine, Color, Document, Fill, Point, Shape, ShapeKind, Size, TabKind, TabStop, TextAlign,
    TextSpan,
};

fn one_line_a_paragraph(req: &TextRequest) -> Vec<Line> {
    let text: String = req.spans.iter().map(|s| s.text.as_str()).collect();
    let chars: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    let mut start = 0;
    while start <= chars.len() {
        let end = chars[start..]
            .iter()
            .position(|c| *c == '\n')
            .map_or(chars.len(), |n| start + n);
        let mut x = 0.0;
        let mut edges = Vec::new();
        for i in start..end {
            edges.push(x);
            if chars[i] == '\t' {
                let rest =
                    chars[i + 1..end].iter().take_while(|c| **c != '\t').count() as f64 * 2.0;
                let stop = req
                    .para
                    .tabs
                    .iter()
                    .find(|t| t.pos > x)
                    .copied()
                    .unwrap_or(TabStop::left(x + 5.0));
                x = match stop.kind {
                    TabKind::Right => (stop.pos - rest).max(x),
                    _ => stop.pos,
                };
            } else {
                x += 2.0;
            }
        }
        edges.push(x);
        lines.push(Line {
            start_char: start,
            end_char: end,
            x: 0.0,
            baseline: -5.0 * (lines.len() + 1) as f64,
            ascent: 4.0,
            descent: 1.0,
            edges,
            starts: Vec::new(),
            ends: Vec::new(),
        });
        start = end + 1;
    }
    lines
}

/// The package without TraceDraw's own members: what another reader
/// sees.
fn without_own_members(bytes: &[u8]) -> Vec<u8> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let stored =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for i in 0..z.len() {
        let mut f = z.by_index(i).expect("member");
        let name = f.name().expect("name").into_owned();
        if name.starts_with("tracedraw/") {
            continue;
        }
        let mut data = Vec::new();
        f.read_to_end(&mut data).expect("read");
        out.start_file(name, stored).expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

/// A menu frame: "Mango<tab>89" over "Sorbet<tab>890", a right stop at
/// 60 mm and a left one at 70 mm.
fn menu() -> Document {
    let mut doc = Document::new("menu", Size::new(200.0, 100.0));
    let mut ids = doc.ids().clone();
    let mut t = Shape::new(
        ids.shape(),
        ShapeKind::Text {
            spans: vec![TextSpan::new("Mango\t89\nSorbet\t890", "Arial", 12.0)],
            origin: Point::ZERO,
            frame: Some(Size::new(80.0, 40.0)),
            align: TextAlign::Left,
            para: tracedraw_core::ParagraphStyle {
                tabs: vec![TabStop::new(60.0, TabKind::Right), TabStop::left(70.0)],
                ..Default::default()
            },
            on_path: None,
        },
    );
    t.transform = Affine::translate((20.0, 80.0));
    t.fill = Fill::Solid(Color::BLACK);
    t.stroke = None;
    doc.pages[0].layers[0].shapes = vec![t];
    doc.set_ids(ids);
    doc
}

/// Every text of `shapes` (in groups too): its characters and where its
/// origin is on the page.
fn texts(shapes: &[Shape], at: Affine, out: &mut Vec<(String, Point, Shape)>) {
    for s in shapes {
        let t = at * s.transform;
        match &s.kind {
            ShapeKind::Text { spans, origin, .. } => out.push((
                spans.iter().map(|sp| sp.text.as_str()).collect(),
                t * *origin,
                s.clone(),
            )),
            ShapeKind::Group { children } => texts(children, t, out),
            _ => {}
        }
    }
}

/// Written as artistic lines, every version: the parts after the tabs
/// stand where the stops put them, 60 mm less the price's width from the
/// frame's left edge (as any reader shows them, whatever its own stops).
#[test]
fn lines_are_cut_at_their_tabs_and_keep_the_stops_places() {
    text_outline::set_lines(one_line_a_paragraph);
    let doc = menu();
    for v in [7u16, 9, 12, 13, 14, 16, 21] {
        let bytes = tracedraw_cdr::write::document_to_cdr_version(&doc, v);
        let drawn = if v >= 14 {
            without_own_members(&bytes)
        } else {
            bytes.clone()
        };
        let (shown, _) = tracedraw_cdr::open_bytes(&drawn, "drawn").expect("reads");
        let mut found = Vec::new();
        texts(
            &shown.pages[0].layers[0].shapes,
            Affine::IDENTITY,
            &mut found,
        );
        let at = |t: &str| {
            found
                .iter()
                .find(|(s, _, _)| s == t)
                .map(|(_, p, _)| *p)
                .unwrap_or_else(|| panic!("{v}: {t} not written: {found:?}"))
        };
        assert!(found.iter().all(|(s, _, _)| !s.contains('\t')), "{v}");
        let (mango, price) = (at("Mango"), at("89"));
        let (sorbet, big) = (at("Sorbet"), at("890"));
        // Right stop at 60: "89" (4 mm) from 56, "890" (6 mm) from 54.
        assert!(
            (price.x - mango.x - 56.0).abs() < 0.01,
            "{v}: {mango:?} {price:?}"
        );
        assert!(
            (big.x - sorbet.x - 54.0).abs() < 0.01,
            "{v}: {sorbet:?} {big:?}"
        );
        assert!((price.y - mango.y).abs() < 0.01, "{v}");
    }
}

/// Written as paragraph frames (version 16 on, style strings), the stops
/// come back with their kinds and positions; centre and decimal stops,
/// whose codes no sample has shown yet, come back as left stops.
#[test]
fn paragraph_frames_keep_their_tab_stops() {
    text_outline::set_lines(one_line_a_paragraph);
    let mut doc = menu();
    let previews = tracedraw_cdr::write::Previews {
        paragraph_frames: true,
        ..Default::default()
    };
    for v in [16u16, 19, 21] {
        let bytes = tracedraw_cdr::write::document_to_cdr_version_with(&doc, v, &previews);
        let (shown, _) =
            tracedraw_cdr::open_bytes(&without_own_members(&bytes), "drawn").expect("reads");
        let mut found = Vec::new();
        texts(
            &shown.pages[0].layers[0].shapes,
            Affine::IDENTITY,
            &mut found,
        );
        let frame = found
            .iter()
            .find(|(_, _, s)| matches!(s.kind, ShapeKind::Text { frame: Some(_), .. }))
            .unwrap_or_else(|| panic!("{v}: no paragraph text: {found:?}"));
        let ShapeKind::Text { para, .. } = &frame.2.kind else {
            unreachable!()
        };
        assert_eq!(
            para.tabs,
            vec![TabStop::new(60.0, TabKind::Right), TabStop::left(70.0)],
            "{v}"
        );
    }
    // A centre stop goes out as a centre one (code 2, #1852; it went out
    // as a left one before the code was known).
    if let ShapeKind::Text { para, .. } = &mut doc.pages[0].layers[0].shapes[0].kind {
        para.tabs[1].kind = TabKind::Center;
    }
    let bytes = tracedraw_cdr::write::document_to_cdr_version_with(&doc, 21, &previews);
    let (shown, _) =
        tracedraw_cdr::open_bytes(&without_own_members(&bytes), "drawn").expect("reads");
    let mut found = Vec::new();
    texts(
        &shown.pages[0].layers[0].shapes,
        Affine::IDENTITY,
        &mut found,
    );
    let tabs = found
        .iter()
        .find_map(|(_, _, s)| match &s.kind {
            ShapeKind::Text {
                frame: Some(_),
                para,
                ..
            } => Some(para.tabs.clone()),
            _ => None,
        })
        .expect("paragraph text");
    assert_eq!(tabs[1], TabStop::new(70.0, TabKind::Center));
}
