//! Devanagari half forms and conjuncts (#599): every way the text engine
//! cuts text into shaping runs (by script, by the face that has the
//! letters, by span, by line) keeps each syllable whole, so a word draws
//! with the glyphs the font makes for it shaped in one piece.

use super::*;
use tracedraw_core::color::Color;

/// Noto Sans Devanagari (SIL Open Font License, see
/// `testdata/NotoSansDevanagari-OFL.txt`): full Devanagari GSUB and GPOS.
const NOTO_DEVA: &[u8] = include_bytes!("../testdata/NotoSansDevanagari-Regular.ttf");
/// A Latin-only face, for the fallback cases.
const QUICKSAND: &[u8] = include_bytes!("../testdata/Quicksand-VF.ttf");
const DEVA: &str = "Noto Sans Devanagari";

/// Words with half forms, conjuncts, reph, rakar, nukta and vowel signs
/// on conjuncts (the i sign before a cluster, u and uu signs under one),
/// one per line.
const CORPUS_TEXT: &str = include_str!("../testdata/devanagari-corpus.txt");

fn corpus() -> Vec<&'static str> {
    CORPUS_TEXT
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

fn system() -> FontSystem {
    let mut db = fontdb::Database::new();
    db.load_font_data(NOTO_DEVA.to_vec());
    db.load_font_data(QUICKSAND.to_vec());
    FontSystem::from_db(db)
}

/// The glyphs the font makes for `text` shaped as one run.
fn whole(text: &str) -> Vec<u16> {
    let face = rustybuzz::Face::from_slice(NOTO_DEVA, 0).expect("font");
    let mut b = rustybuzz::UnicodeBuffer::new();
    b.push_str(text);
    b.guess_segment_properties();
    let out = rustybuzz::shape(&face, &[], b);
    out.glyph_infos()
        .iter()
        .map(|i| i.glyph_id as u16)
        .collect()
}

fn deva_gids(f: &FontSystem, spans: &[TextSpan], frame: Option<Size>) -> Vec<u16> {
    let para = ParagraphStyle::default();
    f.glyph_layout(&TextRequest {
        spans,
        frame,
        align: TextAlign::Left,
        para: &para,
        on_path: None,
    })
    .glyphs
    .iter()
    .filter(|g| g.font.data.len() == NOTO_DEVA.len())
    .map(|g| g.glyph_id)
    .collect()
}

fn colored(text: &str, family: &str, rgb: u8) -> TextSpan {
    let mut s = TextSpan::new(text, family, 24.0);
    s.fill = Some(tracedraw_core::Fill::Solid(Color::rgb8(rgb, 0, 0)));
    s
}

/// The corpus words that come out with other glyphs than the whole word.
fn failures(mut laid: impl FnMut(&str) -> Vec<u16>) -> Vec<String> {
    corpus()
        .into_iter()
        .filter(|w| laid(w) != whole(w))
        .map(|w| w.to_string())
        .collect()
}

#[test]
fn the_font_makes_conjuncts_without_dotted_circles_or_halants() {
    let face = ttf_parser::Face::parse(NOTO_DEVA, 0).expect("font");
    let circle = face.glyph_index('\u{25CC}').map(|g| g.0);
    let halant = face.glyph_index('\u{094D}').map(|g| g.0);
    for w in corpus() {
        let g = whole(w);
        assert!(!g.contains(&circle.unwrap_or(u16::MAX)), "{w}");
        assert!(!g.contains(&halant.unwrap_or(u16::MAX)), "{w}");
    }
}

#[test]
fn a_word_in_a_devanagari_font_is_shaped_whole() {
    let f = system();
    let bad = failures(|w| deva_gids(&f, &[TextSpan::new(w, DEVA, 24.0)], None));
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn a_word_after_latin_text_is_shaped_whole() {
    let f = system();
    let bad = failures(|w| deva_gids(&f, &[TextSpan::new(format!("PDF {w}"), DEVA, 24.0)], None));
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn a_word_drawn_by_the_fallback_face_is_shaped_whole() {
    let f = system();
    let bad = failures(|w| {
        deva_gids(
            &f,
            &[TextSpan::new(format!("Ab {w}"), "Quicksand", 24.0)],
            None,
        )
    });
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn a_word_split_between_spans_is_shaped_whole() {
    let f = system();
    let mut bad = Vec::new();
    for w in corpus() {
        let cuts: Vec<usize> = w.char_indices().map(|(i, _)| i).skip(1).collect();
        for cut in cuts {
            let spans = [colored(&w[..cut], DEVA, 10), colored(&w[cut..], DEVA, 200)];
            if deva_gids(&f, &spans, None) != whole(w) {
                bad.push(format!("{}|{}", &w[..cut], &w[cut..]));
            }
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}

#[test]
fn a_word_broken_across_lines_breaks_between_syllables() {
    let f = system();
    let para = ParagraphStyle::default();
    let mut bad = Vec::new();
    let mut broken = 0;
    for w in corpus() {
        let chars: Vec<char> = w.chars().collect();
        let spans = [TextSpan::new(w, DEVA, 24.0)];
        let req = |frame| TextRequest {
            spans: &spans,
            frame,
            align: TextAlign::Left,
            para: &para,
            on_path: None,
        };
        let wide = f.layout(&req(None)).bounds.width();
        // Narrower than the word: it is broken inside, never inside a
        // syllable.
        for frac in [0.3, 0.5, 0.7] {
            let l = f.layout(&req(Some(Size::new(wide * frac, 500.0))));
            for line in l.lines.iter().skip(1) {
                broken += 1;
                let i = line.start_char;
                if i > 0 && i < chars.len() && continues_syllable(chars[i - 1], chars[i]) {
                    let (a, b): (String, String) =
                        (chars[..i].iter().collect(), chars[i..].iter().collect());
                    bad.push(format!("{a}|{b} at {frac}"));
                }
            }
        }
    }
    assert!(broken > 50, "{broken}");
    assert!(bad.is_empty(), "{bad:?}");
}

/// Only a consonant of the same script continues a syllable after a
/// linker: a Latin letter or another script's letter starts its own, so
/// "क्a" does not send the Devanagari syllable to a Latin face. Tamil's
/// pulli and Sinhala's al-lakuna are no linkers.
#[test]
fn a_letter_after_a_virama_joins_only_in_its_own_script() {
    assert!(continues_syllable('\u{094D}', 'क'));
    assert!(continues_syllable('\u{09CD}', 'ক'));
    assert!(!continues_syllable('\u{094D}', 'a'));
    assert!(!continues_syllable('\u{094D}', 'ক'));
    assert!(!continues_syllable('\u{0BCD}', 'க'));
    assert!(!continues_syllable('\u{0DCA}', 'ක'));
    assert!(continues_syllable('\u{200D}', 'य'));
    assert!(continues_syllable('क', '\u{093F}'));
    assert!(!continues_syllable('a', 'b'));
    let f = system();
    let spans = [TextSpan::new("क्a", DEVA, 24.0)];
    let para = ParagraphStyle::default();
    let laid = f.glyph_layout(&TextRequest {
        spans: &spans,
        frame: None,
        align: TextAlign::Left,
        para: &para,
        on_path: None,
    });
    let first = laid.glyphs.first().expect("glyphs");
    assert_eq!(
        first.font.data.len(),
        NOTO_DEVA.len(),
        "क् stays in its face"
    );
}
