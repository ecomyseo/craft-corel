//! Linked text content (Text > Link Text Content, #1568, #1364, #1194).
//!
//! Texts that carry the same [`Shape::content_link`] key always hold the
//! same characters: what is typed, deleted or replaced in any one of them
//! appears in the others, both ways. Only the characters are shared: each
//! text keeps its own fonts, sizes, colours, outline, frame, transform and
//! effects (a blend, extrusion or envelope between or on them included),
//! so the same word can be set in several looks and changed once.
//!
//! The engine keeps the texts equal after every step
//! ([`Document::sync_linked_text`]): the text whose characters the step
//! changed passes them on, and every other text of its link takes them
//! with its own looks, the characters it shared with its old text keeping
//! their own style and new ones taking the style of the character before
//! them. The propagation belongs to the step that made the change, so one
//! Undo takes it all back.
//!
//! This is not the flow of linked paragraph frames (Text > Paragraph Text
//! Frame > Link, `apps/tracedraw/src/textflow.rs`), where one text runs on
//! from one frame into the next: a frame of such a chain cannot have its
//! content linked, and the reverse.

use crate::document::{
    spans_char_count, spans_delete, spans_insert, spans_text, Document, Shape, ShapeKind, TextSpan,
};
use crate::id::ShapeId;
use std::collections::BTreeMap;

/// The characters of a text object (`None` for any other kind).
pub fn text_of(shape: &Shape) -> Option<String> {
    match &shape.kind {
        ShapeKind::Text { spans, .. } => Some(spans_text(spans)),
        _ => None,
    }
}

/// The text's content link, for a text object that has one.
pub fn link_of(shape: &Shape) -> Option<u64> {
    match shape.kind {
        ShapeKind::Text { .. } => shape.content_link,
        _ => None,
    }
}

/// Make `spans` hold the characters `new`, keeping their styles: the
/// characters the old and new text share at the start and at the end keep
/// the spans they were in, the characters in between are replaced, the
/// new ones in the style of the character before them (of the first
/// character at the very start).
pub fn retext_spans(spans: &mut Vec<TextSpan>, new: &str) {
    let old: Vec<char> = spans_text(spans).chars().collect();
    let new: Vec<char> = new.chars().collect();
    if old == new {
        return;
    }
    let prefix = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let room = old.len().min(new.len()) - prefix;
    let suffix = old
        .iter()
        .rev()
        .zip(new.iter().rev())
        .take(room)
        .take_while(|(a, b)| a == b)
        .count();
    spans_delete(spans, prefix, old.len() - suffix);
    let middle: String = new[prefix..new.len() - suffix].iter().collect();
    if middle.is_empty() {
        return;
    }
    if spans.is_empty() {
        spans.push(TextSpan::new("", "", 12.0));
    }
    spans_insert(spans, prefix, &middle);
    debug_assert_eq!(spans_char_count(spans), new.len());
}

/// Call `f` with every text object that has a content link, nested ones
/// too, in drawing order (master layers first, then the pages).
fn each_linked_text<'a>(doc: &'a Document, f: &mut dyn FnMut(u64, &'a Shape)) {
    fn walk<'a>(s: &'a Shape, f: &mut dyn FnMut(u64, &'a Shape)) {
        match &s.kind {
            ShapeKind::Text { .. } => {
                if let Some(link) = s.content_link {
                    f(link, s);
                }
            }
            ShapeKind::Group { children } => children.iter().for_each(|c| walk(c, f)),
            ShapeKind::ClipFrame { frame, contents } => {
                walk(frame, f);
                contents.iter().for_each(|c| walk(c, f));
            }
            _ => {}
        }
    }
    for l in doc.all_layers() {
        for s in &l.shapes {
            walk(s, f);
        }
    }
}

/// Every text object with a content link and its characters.
fn linked_texts(doc: &Document) -> Vec<(u64, ShapeId, String)> {
    let mut out = Vec::new();
    each_linked_text(doc, &mut |link, s| {
        out.push((link, s.id, text_of(s).unwrap_or_default()));
    });
    out
}

impl Document {
    /// The text objects linked to `link`, in drawing order.
    pub fn content_link_members(&self, link: u64) -> Vec<ShapeId> {
        let mut out = Vec::new();
        each_linked_text(self, &mut |l, s| {
            if l == link {
                out.push(s.id);
            }
        });
        out
    }

    /// The other text objects whose content is linked with `id` (empty
    /// when it has no content link, or is alone in its link).
    pub fn content_linked_with(&self, id: ShapeId) -> Vec<ShapeId> {
        let Some(link) = self.find_shape(id).and_then(link_of) else {
            return Vec::new();
        };
        self.content_link_members(link)
            .into_iter()
            .filter(|m| *m != id)
            .collect()
    }

    /// Bring every link's texts to the same characters after a change
    /// from `before`: in each link whose texts differ, the text the change
    /// altered (the first in drawing order when several were) gives its
    /// characters to the others; with none altered (a text just joined
    /// the link, pasted or undone into it) a text that was there before
    /// gives them, else the first. Cheap when nothing is linked.
    pub fn sync_linked_text(&mut self, before: &Document) {
        let now = linked_texts(self);
        if now.is_empty() {
            return;
        }
        let mut links: BTreeMap<u64, Vec<(ShapeId, String)>> = BTreeMap::new();
        for (link, id, text) in now {
            links.entry(link).or_default().push((id, text));
        }
        for members in links.values() {
            let Some((_, first)) = members.first() else {
                continue;
            };
            if members.iter().all(|(_, t)| t == first) {
                continue;
            }
            let was: Vec<Option<String>> = members
                .iter()
                .map(|(id, _)| before.find_shape(*id).and_then(text_of))
                .collect();
            let changed = members
                .iter()
                .zip(&was)
                .find(|((_, t), w)| w.as_ref().is_some_and(|w| w != t));
            let kept = members.iter().zip(&was).find(|(_, w)| w.is_some());
            let source = changed
                .or(kept)
                .map(|((_, t), _)| t.clone())
                .unwrap_or_else(|| first.clone());
            for (id, text) in members {
                if *text == source {
                    continue;
                }
                if let Some(Shape {
                    kind: ShapeKind::Text { spans, .. },
                    ..
                }) = self.find_shape_mut(*id)
                {
                    retext_spans(spans, &source);
                }
            }
        }
    }

    /// A key no link of the drawing uses yet, made from `seed` (an
    /// object's identity, so that texts pasted from another drawing do not
    /// join a link of this one by chance).
    pub fn fresh_content_link(&self, seed: u64) -> u64 {
        let mut used = std::collections::BTreeSet::new();
        each_linked_text(self, &mut |l, _| {
            used.insert(l);
        });
        let mut key = seed.max(1);
        while used.contains(&key) {
            key = key.wrapping_add(1).max(1);
        }
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Layer, ParagraphStyle, TextAlign};
    use crate::geometry::Point;

    fn span(text: &str, font: &str) -> TextSpan {
        TextSpan::new(text, font, 24.0)
    }

    fn text(id: u64, spans: Vec<TextSpan>, link: Option<u64>) -> Shape {
        let mut s = Shape::new(
            ShapeId(id),
            ShapeKind::Text {
                spans,
                origin: Point::ZERO,
                frame: None,
                align: TextAlign::Left,
                para: ParagraphStyle::default(),
                on_path: None,
            },
        );
        s.content_link = link;
        s
    }

    fn doc_with(shapes: Vec<Shape>) -> Document {
        let mut doc = Document::default();
        let layer: &mut Layer = &mut doc.pages[0].layers[0];
        layer.shapes = shapes;
        doc
    }

    fn spans_of(doc: &Document, id: u64) -> Vec<TextSpan> {
        match &doc.find_shape(ShapeId(id)).map(|s| s.kind.clone()) {
            Some(ShapeKind::Text { spans, .. }) => spans.clone(),
            _ => Vec::new(),
        }
    }

    #[test]
    fn retext_keeps_the_styles_of_the_shared_characters() {
        let mut spans = vec![span("Big", "Serif"), span(" Sale", "Sans")];
        retext_spans(&mut spans, "Big Summer Sale");
        assert_eq!(spans_text(&spans), "Big Summer Sale");
        // "Big" stays in Serif, " Sale" in Sans; the new " Summer" takes
        // the style of the character before it.
        assert_eq!(spans[0].font_family, "Serif");
        assert!(spans[0].text.starts_with("Big"));
        assert_eq!(spans.last().map(|s| s.font_family.as_str()), Some("Sans"));
        assert!(spans.last().is_some_and(|s| s.text.ends_with(" Sale")));
    }

    #[test]
    fn retext_to_nothing_keeps_a_span_for_the_style() {
        let mut spans = vec![span("Hello", "Serif")];
        retext_spans(&mut spans, "");
        assert_eq!(spans_text(&spans), "");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].font_family, "Serif");
        retext_spans(&mut spans, "Hi");
        assert_eq!(spans_text(&spans), "Hi");
        assert_eq!(spans[0].font_family, "Serif");
    }

    #[test]
    fn retext_handles_repeated_letters_and_wide_characters() {
        let mut spans = vec![span("aaa", "A")];
        retext_spans(&mut spans, "aa");
        assert_eq!(spans_text(&spans), "aa");
        let mut spans = vec![span("नमस्ते", "A")];
        retext_spans(&mut spans, "नमस्ते दुनिया");
        assert_eq!(spans_text(&spans), "नमस्ते दुनिया");
    }

    #[test]
    fn a_change_to_any_linked_text_reaches_the_others_with_their_own_looks() {
        let before = doc_with(vec![
            text(1, vec![span("SALE", "Serif")], Some(7)),
            text(2, vec![span("SALE", "Sans")], Some(7)),
            text(3, vec![span("SALE", "Mono")], None),
        ]);
        // The second text is edited: it reaches the first, not the third.
        let mut doc = before.clone();
        if let Some(Shape {
            kind: ShapeKind::Text { spans, .. },
            ..
        }) = doc.find_shape_mut(ShapeId(2))
        {
            spans[0].text = "SUMMER SALE".into();
        }
        doc.sync_linked_text(&before);
        assert_eq!(spans_text(&spans_of(&doc, 1)), "SUMMER SALE");
        assert_eq!(spans_of(&doc, 1)[0].font_family, "Serif");
        assert_eq!(spans_text(&spans_of(&doc, 3)), "SALE");
        // And back from the first.
        let before = doc.clone();
        if let Some(Shape {
            kind: ShapeKind::Text { spans, .. },
            ..
        }) = doc.find_shape_mut(ShapeId(1))
        {
            spans[0].text = "WINTER SALE".into();
        }
        doc.sync_linked_text(&before);
        assert_eq!(spans_text(&spans_of(&doc, 2)), "WINTER SALE");
        assert_eq!(spans_of(&doc, 2)[0].font_family, "Sans");
    }

    #[test]
    fn linked_texts_in_groups_follow_too() {
        let mut group = Shape::new(
            ShapeId(10),
            ShapeKind::Group {
                children: vec![text(2, vec![span("A", "Sans")], Some(3))],
            },
        );
        group.content_link = Some(3); // A group never takes part.
        let before = doc_with(vec![text(1, vec![span("A", "Serif")], Some(3)), group]);
        let mut doc = before.clone();
        if let Some(Shape {
            kind: ShapeKind::Text { spans, .. },
            ..
        }) = doc.find_shape_mut(ShapeId(1))
        {
            spans[0].text = "B".into();
        }
        doc.sync_linked_text(&before);
        assert_eq!(spans_text(&spans_of(&doc, 2)), "B");
        assert_eq!(doc.content_link_members(3), vec![ShapeId(1), ShapeId(2)]);
        assert_eq!(doc.content_linked_with(ShapeId(2)), vec![ShapeId(1)]);
    }

    #[test]
    fn a_text_joining_a_link_takes_the_links_characters() {
        let before = doc_with(vec![text(1, vec![span("ONE", "Serif")], Some(5))]);
        let mut doc = before.clone();
        doc.pages[0].layers[0]
            .shapes
            .push(text(2, vec![span("OLD", "Sans")], Some(5)));
        doc.sync_linked_text(&before);
        assert_eq!(spans_text(&spans_of(&doc, 2)), "ONE");
        assert_eq!(spans_text(&spans_of(&doc, 1)), "ONE");
    }

    #[test]
    fn fresh_keys_avoid_links_in_use() {
        let doc = doc_with(vec![text(1, vec![span("A", "S")], Some(9))]);
        assert_eq!(doc.fresh_content_link(9), 10);
        assert_eq!(doc.fresh_content_link(0), 1);
        assert_eq!(doc.fresh_content_link(4), 4);
    }

    fn engine_with_two_linked() -> crate::Engine {
        let mut eng = crate::Engine::new(doc_with(vec![
            text(1, vec![span("SALE", "Serif")], None),
            text(2, vec![span("OLD", "Sans")], None),
        ]));
        eng.run(&crate::Command::LinkTextContent {
            shapes: vec![ShapeId(1), ShapeId(2)],
        })
        .unwrap();
        eng
    }

    fn set_text(eng: &crate::Engine, id: u64, t: &str) -> crate::Command {
        let mut kind = eng.document().find_shape(ShapeId(id)).unwrap().kind.clone();
        if let ShapeKind::Text { spans, .. } = &mut kind {
            spans[0].text = t.into();
        }
        crate::Command::SetShapeKind {
            shape: ShapeId(id),
            kind,
        }
    }

    #[test]
    fn linking_gives_the_first_texts_characters_and_one_key() {
        let eng = engine_with_two_linked();
        let doc = eng.document();
        assert_eq!(spans_text(&spans_of(doc, 2)), "SALE");
        assert_eq!(spans_of(doc, 2)[0].font_family, "Sans");
        let a = doc.find_shape(ShapeId(1)).unwrap().content_link;
        assert!(a.is_some());
        assert_eq!(a, doc.find_shape(ShapeId(2)).unwrap().content_link);
        assert_eq!(eng.history_labels().0.last(), Some(&"Link Text Content"));
    }

    #[test]
    fn typing_in_either_text_updates_the_other_and_undo_is_one_step() {
        let mut eng = engine_with_two_linked();
        let steps = eng.history_labels().0.len();
        let cmd = set_text(&eng, 2, "SALES");
        eng.run(&cmd).unwrap();
        assert_eq!(spans_text(&spans_of(eng.document(), 1)), "SALES");
        // Typing on joins the step (as the text editor does).
        let cmd = set_text(&eng, 2, "SALES!");
        eng.amend(&cmd).unwrap();
        assert_eq!(spans_text(&spans_of(eng.document(), 1)), "SALES!");
        assert_eq!(eng.history_labels().0.len(), steps + 1);
        eng.undo().unwrap();
        assert_eq!(spans_text(&spans_of(eng.document(), 1)), "SALE");
        assert_eq!(spans_text(&spans_of(eng.document(), 2)), "SALE");
        // The other way round, in a batch.
        let cmd = set_text(&eng, 1, "NEW");
        eng.run_batch("Edit Text", &[cmd]).unwrap();
        assert_eq!(spans_text(&spans_of(eng.document(), 2)), "NEW");
        assert_eq!(spans_of(eng.document(), 2)[0].font_family, "Sans");
    }

    #[test]
    fn unlinked_texts_go_their_own_way() {
        let mut eng = engine_with_two_linked();
        eng.run(&crate::Command::UnlinkTextContent {
            shapes: vec![ShapeId(1)],
        })
        .unwrap();
        // The text left alone leaves its link too.
        for id in [1, 2] {
            let s = eng.document().find_shape(ShapeId(id)).unwrap();
            assert_eq!(s.content_link, None);
        }
        let cmd = set_text(&eng, 1, "ONLY ONE");
        eng.run(&cmd).unwrap();
        assert_eq!(spans_text(&spans_of(eng.document(), 2)), "SALE");
    }

    #[test]
    fn linking_refuses_objects_that_are_not_text() {
        let mut doc = doc_with(vec![text(1, vec![span("A", "S")], None)]);
        doc.pages[0].layers[0].shapes.push(Shape::new(
            ShapeId(2),
            ShapeKind::Group { children: vec![] },
        ));
        let mut eng = crate::Engine::new(doc);
        let err = eng.run(&crate::Command::LinkTextContent {
            shapes: vec![ShapeId(1), ShapeId(2)],
        });
        assert!(matches!(err, Err(crate::Error::NotText(_))));
        assert_eq!(
            eng.document().find_shape(ShapeId(1)).unwrap().content_link,
            None
        );
    }

    #[test]
    fn linking_a_text_to_a_linked_one_joins_the_whole_link() {
        let mut eng = engine_with_two_linked();
        let cmd = crate::Command::AddShape {
            layer: eng.document().pages[0].layers[0].id,
            shape: text(3, vec![span("THIRD", "Mono")], None),
        };
        eng.run(&cmd).unwrap();
        eng.run(&crate::Command::LinkTextContent {
            shapes: vec![ShapeId(2), ShapeId(3)],
        })
        .unwrap();
        let doc = eng.document();
        assert_eq!(spans_text(&spans_of(doc, 3)), "SALE");
        let key = doc.find_shape(ShapeId(1)).unwrap().content_link;
        assert_eq!(doc.content_link_members(key.unwrap()).len(), 3);
    }

    #[test]
    fn the_link_is_saved_and_read_back() {
        let eng = engine_with_two_linked();
        let json = eng.document().to_json().unwrap();
        let back = Document::from_json(&json).unwrap();
        let key = back.find_shape(ShapeId(1)).unwrap().content_link;
        assert!(key.is_some());
        assert_eq!(key, back.find_shape(ShapeId(2)).unwrap().content_link);
        // Unlinked objects write nothing for it.
        let plain = doc_with(vec![text(1, vec![span("A", "S")], None)]);
        assert!(!plain.to_json().unwrap().contains("content_link"));
    }
}
