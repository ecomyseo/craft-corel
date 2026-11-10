//! Text > Insert Placeholder Text: fills the selected paragraph text
//! frames with filler text, as much as each frame holds, to try a layout
//! before the real copy arrives. The filler is the traditional
//! pseudo-Latin passage printers have used for centuries (public domain).

use crate::app::App;
use tracedraw_core::{geometry::Affine, Command, Shape, ShapeKind, TextSpan};

/// The filler, cycled as often as a frame needs.
const FILLER: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis \
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure \
dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. \
Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim \
id est laborum. Curabitur pretium tincidunt lacus. Nulla gravida orci a odio. Nullam varius, \
turpis et commodo pharetra, est eros bibendum elit, nec luctus magna felis sollicitudin mauris. \
Integer in mauris eu nibh euismod gravida. Duis ac tellus et risus vulputate vehicula. Donec \
lobortis risus a elit. Etiam tempor. Ut ullamcorper, ligula eu tempor congue, eros est euismod \
turpis, id tincidunt sapien risus a quam. Maecenas fermentum consequat mi. Donec fermentum. \
Pellentesque malesuada nulla a mi. Duis sapien sem, aliquet nec, commodo eget, consequat quis, \
neque.";

/// No frame takes more words than this.
const MAX_WORDS: usize = 20_000;
/// A paragraph ends at the first full stop after this many words.
const PARAGRAPH_WORDS: usize = 60;

/// The first `n` words of the filler (cycled), in paragraphs, ending
/// with a full stop.
pub fn placeholder_text(n: usize) -> String {
    let words: Vec<&str> = FILLER.split_whitespace().collect();
    let mut out = String::new();
    let mut in_paragraph = 0;
    for (i, w) in words.iter().cycle().take(n).enumerate() {
        if i > 0 {
            out.push(if in_paragraph == 0 { '\n' } else { ' ' });
        }
        out.push_str(w);
        in_paragraph += 1;
        if in_paragraph >= PARAGRAPH_WORDS && w.ends_with('.') {
            in_paragraph = 0;
        }
    }
    let end = out.trim_end_matches([',', '.', ';', ':']).len();
    out.truncate(end);
    if !out.is_empty() {
        out.push('.');
    }
    out
}

/// Whether `kind` (a paragraph text) lays out without overflowing its
/// frame.
fn fits(shape: &Shape, kind: &ShapeKind) -> bool {
    let probe = Shape {
        kind: kind.clone(),
        ..shape.clone()
    };
    crate::text_editing::layout_of_text_shape(&probe, Affine::IDENTITY)
        .map(|(l, _)| !l.overflow)
        .unwrap_or(false)
}

/// `kind` with its text replaced by `n` words of filler, in the style of
/// its first span (or `default` when it has none).
fn with_words(kind: &ShapeKind, n: usize, default: &TextSpan) -> ShapeKind {
    let mut kind = kind.clone();
    if let ShapeKind::Text { spans, .. } = &mut kind {
        let mut span = spans.first().cloned().unwrap_or_else(|| default.clone());
        span.text = placeholder_text(n);
        *spans = vec![span];
    }
    kind
}

/// The most words of filler `shape` (a paragraph text) holds without
/// overflowing; at least one.
fn words_that_fit(shape: &Shape, default: &TextSpan) -> usize {
    let fits_n = |n: usize| fits(shape, &with_words(&shape.kind, n, default));
    // Double until it overflows, then halve the gap.
    let mut low = 0;
    let mut high = 16;
    while high < MAX_WORDS && fits_n(high) {
        low = high;
        high *= 2;
    }
    let mut high = high.min(MAX_WORDS);
    if low == 0 && !fits_n(1) {
        return 1;
    }
    low = low.max(1);
    if fits_n(high) {
        return high;
    }
    while high - low > 1 {
        let mid = (low + high) / 2;
        if fits_n(mid) {
            low = mid;
        } else {
            high = mid;
        }
    }
    low
}

impl App {
    /// Fill each selected paragraph text frame with as much filler text as
    /// it holds, in the style of its first character, as one undo step.
    /// Artistic text and other objects are left alone.
    pub fn insert_placeholder_text(&mut self) {
        if self.text_edit.is_some() {
            self.finish_text();
        }
        let default = TextSpan::new("", self.text_font.clone(), self.text_size_pt);
        let mut cmds = Vec::new();
        for s in self.selected_shapes() {
            if !matches!(s.kind, ShapeKind::Text { frame: Some(_), .. }) {
                continue;
            }
            let n = words_that_fit(&s, &default);
            cmds.push(Command::SetShapeKind {
                shape: s.id,
                kind: with_words(&s.kind, n, &default),
            });
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Insert Placeholder Text", &cmds) {
            self.status = format!("Insert Placeholder Text: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Point, Size};
    use tracedraw_core::{ShapeId, TextAlign};

    #[test]
    fn filler_ends_with_a_full_stop_in_paragraphs() {
        assert_eq!(placeholder_text(0), "");
        assert_eq!(placeholder_text(2), "Lorem ipsum.");
        // "amet," loses its comma for the full stop.
        assert_eq!(placeholder_text(5), "Lorem ipsum dolor sit amet.");
        let long = placeholder_text(400);
        assert_eq!(long.split_whitespace().count(), 400);
        assert!(long.ends_with('.') && long.contains('\n'));
        assert!(long.lines().all(|p| !p.trim().is_empty()));
    }

    fn frame(app: &mut App, text: &str, w: f64, h: f64) -> ShapeId {
        let kind = ShapeKind::Text {
            spans: vec![TextSpan::new(text, "DejaVu Sans", 12.0)],
            origin: Point::new(10.0, 10.0),
            frame: Some(Size::new(w, h)),
            align: TextAlign::Left,
            para: Default::default(),
            on_path: None,
        };
        app.new_shape(kind).expect("frame")
    }

    fn text_of(app: &App, id: ShapeId) -> String {
        match app.doc().find_shape(id).map(|s| &s.kind) {
            Some(ShapeKind::Text { spans, .. }) => spans.iter().map(|s| s.text.as_str()).collect(),
            _ => String::new(),
        }
    }

    #[test]
    fn frames_fill_up_to_their_size() {
        tracedraw_text::install();
        let mut app = App::headless();
        let small = frame(&mut app, "", 40.0, 20.0);
        let large = frame(&mut app, "keep my style", 120.0, 90.0);
        let artistic = app
            .new_shape(ShapeKind::Text {
                spans: vec![TextSpan::new("headline", "DejaVu Sans", 24.0)],
                origin: Point::new(10.0, 150.0),
                frame: None,
                align: TextAlign::Left,
                para: Default::default(),
                on_path: None,
            })
            .expect("artistic");
        app.select(vec![small, large, artistic]);
        app.insert_placeholder_text();
        let (s, l) = (text_of(&app, small), text_of(&app, large));
        assert!(s.starts_with("Lorem ipsum") && s.ends_with('.'), "{s}");
        assert!(l.split_whitespace().count() > s.split_whitespace().count());
        assert_eq!(text_of(&app, artistic), "headline");
        // As full as it gets: the text fits and a few more words would not
        // (when fonts lay text out at all).
        let shape = app.doc().find_shape(large).cloned().expect("large");
        assert!(fits(&shape, &shape.kind));
        let default = TextSpan::new("", "DejaVu Sans", 12.0);
        let n = l.split_whitespace().count();
        let laid_out = crate::text_editing::layout_of_text_shape(&shape, Affine::IDENTITY)
            .is_some_and(|(layout, _)| !layout.lines.is_empty());
        if laid_out {
            assert!(!fits(&shape, &with_words(&shape.kind, n + 8, &default)));
        }
        // One undo step brings everything back.
        app.undo();
        assert_eq!(text_of(&app, small), "");
        assert_eq!(text_of(&app, large), "keep my style");
    }
}
