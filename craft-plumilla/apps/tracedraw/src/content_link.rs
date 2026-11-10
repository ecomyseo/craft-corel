//! Linked text content (Text > Link Text Content, the object menu and
//! the Blend panel, #1568, #1364, #1194): two texts or more hold the same
//! characters, each with its own looks. The engine keeps them equal
//! (`tracedraw_core::linked_text`); this module builds the commands, keeps
//! the frames of a text flow chain out of it and shows which texts are
//! linked.
//!
//! Not to be mixed up with the flow of linked paragraph frames
//! (`textflow.rs`, Text > Paragraph Text Frame > Link), where one text
//! runs on from frame to frame.

use crate::app::App;
use crate::i18n::{tr, trf};
use tracedraw_core::{linked_text::link_of, Command, ShapeId, ShapeKind};

impl App {
    /// The selected objects that are text, in the order they were
    /// selected (from the selection's shared copy: no search of the
    /// drawing for each, as menus ask every frame).
    fn selected_text_ids(&self) -> Vec<ShapeId> {
        self.selected_rc()
            .iter()
            .filter(|s| matches!(s.kind, ShapeKind::Text { .. }))
            .map(|s| s.id)
            .collect()
    }

    /// The selected texts and the text being edited that have a content
    /// link, with their link.
    fn linked_selection(&self) -> Vec<(ShapeId, u64)> {
        let mut out: Vec<(ShapeId, u64)> = self
            .selected_rc()
            .iter()
            .filter_map(|s| link_of(s).map(|l| (s.id, l)))
            .collect();
        if let Some(te) = &self.text_edit {
            if !out.iter().any(|(id, _)| *id == te.shape) {
                if let Some(l) = self.doc().find_shape(te.shape).and_then(link_of) {
                    out.push((te.shape, l));
                }
            }
        }
        out
    }

    /// Text > Link Text Content can run: two texts or more selected that
    /// are not all in one link already.
    pub fn can_link_text_content(&self) -> bool {
        let ids = self.selected_text_ids();
        if ids.len() < 2 {
            return false;
        }
        let links: Vec<Option<u64>> = ids
            .iter()
            .map(|id| self.doc().find_shape(*id).and_then(link_of))
            .collect();
        !(links[0].is_some() && links.iter().all(|l| *l == links[0]))
    }

    /// The selected texts and the text being edited that have a content
    /// link (the canvas marks them).
    pub fn linked_selection_ids(&self) -> Vec<ShapeId> {
        self.linked_selection()
            .into_iter()
            .map(|(id, _)| id)
            .collect()
    }

    /// A selected text (or the text being edited) has its content linked.
    pub fn selection_has_content_link(&self) -> bool {
        !self.linked_selection().is_empty()
    }

    /// Link the text content of the selected texts: the others take the
    /// characters of the text selected first. Frames of a text flow chain
    /// are refused (their text runs on from frame to frame).
    pub fn link_text_content(&mut self) {
        let ids = self.selected_text_ids();
        if ids.len() < 2 {
            self.status = tr("status.content_link_needs_two");
            return;
        }
        if ids.iter().any(|id| self.is_linked_frame(*id)) {
            self.status = tr("status.content_link_not_flow");
            return;
        }
        self.finish_text();
        match self.engine.run(&Command::LinkTextContent {
            shapes: ids.clone(),
        }) {
            Ok(()) => {
                let n = ids
                    .first()
                    .map(|id| self.doc().content_linked_with(*id).len() + 1)
                    .unwrap_or(0);
                self.status = trf("status.content_linked", &[("n", &n.to_string())]);
            }
            Err(e) => self.status = e.to_string(),
        }
    }

    /// Take the selected texts out of their content links; each keeps the
    /// text it shows.
    pub fn unlink_text_content(&mut self) {
        let ids: Vec<ShapeId> = self
            .linked_selection()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        if ids.is_empty() {
            return;
        }
        match self.engine.run(&Command::UnlinkTextContent { shapes: ids }) {
            Ok(()) => self.status = tr("status.content_unlinked"),
            Err(e) => self.status = e.to_string(),
        }
    }

    /// The texts whose content is linked with a selected text or the text
    /// being edited, each once, the selection itself left out: the canvas
    /// marks them.
    pub fn content_linked_others(&self) -> Vec<ShapeId> {
        let mine = self.linked_selection();
        let mut links: Vec<u64> = mine.iter().map(|(_, l)| *l).collect();
        links.sort_unstable();
        links.dedup();
        let mut out: Vec<ShapeId> = Vec::new();
        for l in links {
            for other in self.doc().content_link_members(l) {
                let selected = self.selection.contains(&other)
                    || self.text_edit.as_ref().is_some_and(|t| t.shape == other);
                if !selected && !out.contains(&other) {
                    out.push(other);
                }
            }
        }
        out
    }

    /// The Blend panel's "Link text content of the ends": the blend's two
    /// ends when both are text, and whether their content is linked.
    pub fn blend_text_ends(&self) -> Option<([ShapeId; 2], bool)> {
        let (start, e) = self.selected_blends().into_iter().next()?;
        let tracedraw_core::live::Effect::Blend {
            end_id: Some(end), ..
        } = e
        else {
            return None;
        };
        let doc = self.doc();
        let (a, b) = (doc.find_shape(start)?, doc.find_shape(end)?);
        if !matches!(a.kind, ShapeKind::Text { .. }) || !matches!(b.kind, ShapeKind::Text { .. }) {
            return None;
        }
        let linked = link_of(a).is_some() && link_of(a) == link_of(b);
        Some(([start, end], linked))
    }

    /// Turn the content link of the selected blend's text ends on or off.
    pub fn set_blend_text_link(&mut self, on: bool) {
        let Some((ends, linked)) = self.blend_text_ends() else {
            return;
        };
        if on == linked {
            return;
        }
        let cmd = if on {
            if ends.iter().any(|id| self.is_linked_frame(*id)) {
                self.status = tr("status.content_link_not_flow");
                return;
            }
            Command::LinkTextContent {
                shapes: ends.to_vec(),
            }
        } else {
            Command::UnlinkTextContent {
                shapes: ends.to_vec(),
            }
        };
        if let Err(e) = self.engine.run(&cmd) {
            self.status = e.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::document::{spans_text, ParagraphStyle, TextAlign, TextSpan};
    use tracedraw_core::geometry::{Point, Size};

    fn text(app: &mut App, t: &str, font: &str, frame: Option<Size>) -> ShapeId {
        app.new_shape(ShapeKind::Text {
            spans: vec![TextSpan::new(t, font, 24.0)],
            origin: Point::new(10.0, 10.0),
            frame,
            align: TextAlign::Left,
            para: ParagraphStyle::default(),
            on_path: None,
        })
        .expect("text")
    }

    fn chars(app: &App, id: ShapeId) -> String {
        match &app.doc().find_shape(id).expect("text").kind {
            ShapeKind::Text { spans, .. } => spans_text(spans),
            _ => String::new(),
        }
    }

    fn font(app: &App, id: ShapeId) -> String {
        match &app.doc().find_shape(id).expect("text").kind {
            ShapeKind::Text { spans, .. } => spans[0].font_family.clone(),
            _ => String::new(),
        }
    }

    #[test]
    fn typing_in_either_linked_text_changes_both_and_undo_takes_both_back() {
        crate::i18n::with_thread_language("en", || {
            let mut app = App::headless();
            let a = text(&mut app, "SALE", "Serif", None);
            let b = text(&mut app, "x", "Sans", None);
            app.select(vec![a, b]);
            assert!(app.can_link_text_content());
            app.link_text_content();
            assert_eq!(chars(&app, b), "SALE");
            assert_eq!(font(&app, b), "Sans");
            assert!(!app.can_link_text_content());
            assert!(app.selection_has_content_link());
            // The status bar names the link of a selected linked text.
            app.select(vec![a]);
            assert!(
                crate::ui::status::object_info(&app).contains("text content linked with 1 more")
            );
            // Typing in the second text, in place, as the Text tool does.
            app.select(vec![b]);
            app.begin_text_edit(b);
            assert!(crate::ui::status::object_info(&app).contains("changes 1 more"));
            app.text_insert("!");
            assert_eq!(chars(&app, a), "SALE!");
            app.text_insert("!");
            assert_eq!(chars(&app, a), "SALE!!");
            app.finish_text();
            // The other text marks the one linked to it.
            app.select(vec![a]);
            assert_eq!(app.content_linked_others(), vec![b]);
            // One step for the typing: one Undo takes both texts back.
            app.undo();
            assert_eq!(chars(&app, a), "SALE");
            assert_eq!(chars(&app, b), "SALE");
            // Unlinked, they go their own way.
            app.select(vec![a, b]);
            app.unlink_text_content();
            assert!(!app.selection_has_content_link());
            app.begin_text_edit(a);
            app.text_insert("X");
            app.finish_text();
            assert_eq!(chars(&app, b), "SALE");
        });
    }

    #[test]
    fn frames_of_a_text_flow_chain_are_not_content_linked_and_the_reverse() {
        crate::i18n::set_language("en");
        let mut app = App::headless();
        let frame = Some(Size::new(40.0, 20.0));
        let a = text(&mut app, "one", "Serif", frame);
        let b = text(&mut app, "two", "Serif", frame);
        let c = text(&mut app, "three", "Sans", frame);
        assert!(app.link_frames(a, b));
        app.select(vec![a, c]);
        app.link_text_content();
        assert_eq!(app.status, tr("status.content_link_not_flow"));
        assert_eq!(app.doc().find_shape(c).and_then(link_of), None);
        // A frame whose content is linked flows into no other frame.
        let d = text(&mut app, "four", "Sans", frame);
        app.select(vec![c, d]);
        app.link_text_content();
        assert!(app.doc().find_shape(c).and_then(link_of).is_some());
        assert!(!app.link_frames(c, d));
        assert_eq!(app.status, tr("status.flow_not_content_linked"));
        assert!(!app.is_linked_frame(c));
    }

    #[test]
    fn a_duplicate_stays_linked() {
        crate::i18n::set_language("en");
        let mut app = App::headless();
        let a = text(&mut app, "SALE", "Serif", None);
        let b = text(&mut app, "SALE", "Sans", None);
        app.select(vec![a, b]);
        app.link_text_content();
        app.select(vec![b]);
        app.duplicate();
        let copy = app.selection[0];
        assert_ne!(copy, b);
        assert_eq!(app.doc().content_linked_with(a).len(), 2);
        app.select(vec![copy]);
        app.begin_text_edit(copy);
        app.text_insert("S");
        app.finish_text();
        assert_eq!(chars(&app, a), "SALES");
        assert_eq!(chars(&app, b), "SALES");
    }

    #[test]
    fn the_blend_panel_links_the_text_ends() {
        crate::i18n::set_language("en");
        let mut app = App::headless();
        let a = text(&mut app, "3D", "Serif", None);
        let b = text(&mut app, "x", "Sans", None);
        app.select(vec![a, b]);
        app.blend_selection(5, 0.0, 0.0, 0.0);
        app.select(vec![a]);
        assert_eq!(app.blend_text_ends(), Some(([a, b], false)));
        app.set_blend_text_link(true);
        assert_eq!(chars(&app, b), "3D");
        assert_eq!(app.blend_text_ends(), Some(([a, b], true)));
        app.set_blend_text_link(false);
        assert_eq!(app.blend_text_ends(), Some(([a, b], false)));
    }
}
