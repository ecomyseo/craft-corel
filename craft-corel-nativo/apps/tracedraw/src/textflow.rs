//! Linked paragraph text frames (Text > Paragraph Text Frame > Link / Unlink).
//!
//! A chain is a list of paragraph frames; the text of the whole chain lives
//! in the frames in order and overflow flows from one frame into the next.
//! Links are kept in the object data of each frame (`text.next`,
//! `text.prev`, shape ids), so they survive save and load without a model
//! change. Re-flow runs after typing, linking and resizing a frame.
//!
//! Not the linked text content of `content_link.rs` (texts that hold the
//! same characters); a frame takes part in one or the other.

use crate::app::App;
use crate::i18n::tr;
use std::collections::HashMap;
use tracedraw_core::{
    document::text_outline::TextRequest,
    document::{Shape, ShapeKind, TextSpan},
    geometry::Point,
    Command, ShapeId,
};

const NEXT: &str = "text.next";
const PREV: &str = "text.prev";

fn data_id(data: &[(String, String)], key: &str) -> Option<ShapeId> {
    data.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.parse::<u64>().ok())
        .map(ShapeId)
}

fn with_key(
    mut data: Vec<(String, String)>,
    key: &str,
    value: Option<ShapeId>,
) -> Vec<(String, String)> {
    data.retain(|(k, _)| k != key);
    if let Some(v) = value {
        data.push((key.to_string(), v.raw().to_string()));
    }
    data
}

/// Take the chain links out of a copied object's data: a copy is not in
/// its original's chain (see [`relink_copies`] for chains copied whole).
pub fn without_links(data: &mut Vec<(String, String)>) {
    data.retain(|(k, _)| k != NEXT && k != PREV);
}

/// Copies whose links were taken out, `copies[i]` made of `originals[i]`
/// with the same structure (groups and clip frames copied member by
/// member, in order): a link to a frame copied along points to its copy,
/// so a chain copied whole stays a chain of the copies; a link to any
/// other frame stays out.
pub fn relink_copies(originals: &[Shape], copies: &mut [Shape]) {
    fn map_ids(o: &Shape, c: &Shape, map: &mut HashMap<ShapeId, ShapeId>) {
        map.insert(o.id, c.id);
        match (&o.kind, &c.kind) {
            (ShapeKind::Group { children: a }, ShapeKind::Group { children: b }) => {
                for (x, y) in a.iter().zip(b) {
                    map_ids(x, y, map);
                }
            }
            (
                ShapeKind::ClipFrame {
                    frame: fa,
                    contents: a,
                },
                ShapeKind::ClipFrame {
                    frame: fb,
                    contents: b,
                },
            ) => {
                map_ids(fa, fb, map);
                for (x, y) in a.iter().zip(b) {
                    map_ids(x, y, map);
                }
            }
            _ => {}
        }
    }
    fn relink(o: &Shape, c: &mut Shape, map: &HashMap<ShapeId, ShapeId>) {
        for key in [NEXT, PREV] {
            if let Some(to) = data_id(&o.data, key).and_then(|id| map.get(&id)) {
                c.data = with_key(std::mem::take(&mut c.data), key, Some(*to));
            }
        }
        match (&o.kind, &mut c.kind) {
            (ShapeKind::Group { children: a }, ShapeKind::Group { children: b }) => {
                for (x, y) in a.iter().zip(b) {
                    relink(x, y, map);
                }
            }
            (
                ShapeKind::ClipFrame {
                    frame: fa,
                    contents: a,
                },
                ShapeKind::ClipFrame {
                    frame: fb,
                    contents: b,
                },
            ) => {
                relink(fa, fb, map);
                for (x, y) in a.iter().zip(b) {
                    relink(x, y, map);
                }
            }
            _ => {}
        }
    }
    let mut map = HashMap::new();
    for (o, c) in originals.iter().zip(copies.iter()) {
        map_ids(o, c, &mut map);
    }
    for (o, c) in originals.iter().zip(copies.iter_mut()) {
        relink(o, c, &map);
    }
}

impl App {
    fn frame_data(&self, id: ShapeId) -> Option<Vec<(String, String)>> {
        let s = self.doc().find_shape(id)?;
        matches!(s.kind, ShapeKind::Text { frame: Some(_), .. }).then(|| s.data.clone())
    }

    /// Next frame in the chain, if any and still present.
    pub fn next_frame(&self, id: ShapeId) -> Option<ShapeId> {
        let next = data_id(&self.frame_data(id)?, NEXT)?;
        self.frame_data(next).map(|_| next)
    }

    /// Previous frame in the chain, if any and still present.
    pub fn prev_frame(&self, id: ShapeId) -> Option<ShapeId> {
        let prev = data_id(&self.frame_data(id)?, PREV)?;
        self.frame_data(prev).map(|_| prev)
    }

    /// First frame of the chain containing `id`.
    pub fn chain_head(&self, id: ShapeId) -> ShapeId {
        let mut cur = id;
        let mut guard = 0;
        while let Some(p) = self.prev_frame(cur) {
            cur = p;
            guard += 1;
            if guard > 1000 {
                break;
            }
        }
        cur
    }

    /// All frames of the chain starting at its head, in flow order.
    pub fn chain_of(&self, id: ShapeId) -> Vec<ShapeId> {
        let mut out = vec![self.chain_head(id)];
        let mut guard = 0;
        while let Some(n) = self.next_frame(*out.last().unwrap_or(&id)) {
            if out.contains(&n) {
                break;
            }
            out.push(n);
            guard += 1;
            if guard > 1000 {
                break;
            }
        }
        out
    }

    pub fn is_linked_frame(&self, id: ShapeId) -> bool {
        self.next_frame(id).is_some() || self.prev_frame(id).is_some()
    }

    /// Link the two selected paragraph frames: the first selected flows into
    /// the second. The second frame's own text is appended to the chain.
    pub fn link_text_frames(&mut self) {
        let [a, b] = self.selection.as_slice() else {
            self.status = tr("status.link_needs_two_frames");
            return;
        };
        let (a, b) = (*a, *b);
        self.link_frames(a, b);
    }

    /// Link frame `a` to flow into `b` (on any page): false, with the
    /// reason on the status bar, when they cannot be linked.
    pub fn link_frames(&mut self, a: ShapeId, b: ShapeId) -> bool {
        let (Some(da), Some(db)) = (self.frame_data(a), self.frame_data(b)) else {
            self.status = tr("status.link_needs_two_frames");
            return false;
        };
        if a == b
            || self.chain_of(a).contains(&b)
            || self.next_frame(a).is_some()
            || self.prev_frame(b).is_some()
        {
            self.status = tr("status.link_already");
            return false;
        }
        // A frame whose text content is linked (Link Text Content) keeps
        // all of its text: it flows into no other frame (#1568).
        let content_linked = |id| {
            self.doc()
                .find_shape(id)
                .and_then(tracedraw_core::linked_text::link_of)
                .is_some()
        };
        if content_linked(a) || content_linked(b) {
            self.status = tr("status.flow_not_content_linked");
            return false;
        }
        let cmds = vec![
            Command::SetObjectData {
                shape: a,
                data: with_key(da, NEXT, Some(b)),
            },
            Command::SetObjectData {
                shape: b,
                data: with_key(db, PREV, Some(a)),
            },
        ];
        let _ = self.engine.run_batch("Link Frames", &cmds);
        self.reflow_chain(a);
        self.status = tr("status.frames_linked");
        true
    }

    /// The frame whose flow tab is under the screen point `at` (Pick
    /// tool): a frame shown with a tab that flows into no other frame.
    pub fn flow_tab_at(&self, at: egui::Pos2) -> Option<ShapeId> {
        self.shown_text_frames(None)
            .into_iter()
            .filter(|f| (f.overflow || f.chosen) && !f.linked)
            .find(|f| {
                let a = self.view.to_screen(f.corners[0]);
                let b = self.view.to_screen(f.corners[1]);
                crate::canvas::flow_tab_rect(a, b, self.handle_size())
                    .expand(2.0)
                    .contains(at)
            })
            .map(|f| f.id)
    }

    /// The frame whose top tab is under the screen point `at` (Pick
    /// tool): a picked frame that no other frame flows into yet.
    pub fn flow_in_tab_at(&self, at: egui::Pos2) -> Option<ShapeId> {
        self.shown_text_frames(None)
            .into_iter()
            .filter(|f| f.chosen && !f.receives)
            .find(|f| {
                let a = self.view.to_screen(f.corners[3]);
                let b = self.view.to_screen(f.corners[2]);
                crate::canvas::flow_in_tab_rect(a, b, self.handle_size())
                    .expand(2.0)
                    .contains(at)
            })
            .map(|f| f.id)
    }

    /// A click on a frame's flow tab: the next click says where its text
    /// flows on (#371).
    pub fn start_text_link(&mut self, from: ShapeId) {
        self.pending_text_link = Some(from);
        self.text_link_incoming = false;
        self.status = tr("status.flow_text_where");
    }

    /// A click on a frame's top tab: the next click says which frame's
    /// text flows into it (a new frame on empty space).
    pub fn start_text_link_in(&mut self, to: ShapeId) {
        self.pending_text_link = Some(to);
        self.text_link_incoming = true;
        self.status = tr("status.flow_text_from");
    }

    /// Let go of a pending link (Esc, a right click).
    pub fn cancel_text_link(&mut self) {
        self.pending_text_link = None;
        self.text_link_drag = None;
        self.text_link_incoming = false;
        self.text_link_from_tab = false;
    }

    /// The screen point the pending link's line starts from: the middle of
    /// the tab it came from.
    pub fn text_link_origin(&self) -> Option<egui::Pos2> {
        let id = self.pending_text_link?;
        let s = self.doc().find_shape(id)?;
        let c = s.text_frame_corners()?;
        let (a, b) = if self.text_link_incoming {
            (c[3], c[2])
        } else {
            (c[0], c[1])
        };
        let (a, b) = (self.view.to_screen(a), self.view.to_screen(b));
        Some(if self.text_link_incoming {
            crate::canvas::flow_in_tab_rect(a, b, self.handle_size()).center()
        } else {
            crate::canvas::flow_tab_rect(a, b, self.handle_size()).center()
        })
    }

    /// The click after a flow tab, at page point `p` (a drag: the frame
    /// `rect`): on another paragraph frame (on any page) the text flows
    /// into it; elsewhere a new frame takes it, the size of the first one
    /// with its top left corner at `p`, or the dragged rectangle.
    pub fn finish_text_link(&mut self, p: Point, rect: Option<tracedraw_core::geometry::Rect>) {
        let incoming = std::mem::take(&mut self.text_link_incoming);
        self.text_link_from_tab = false;
        let Some(from) = self.pending_text_link.take() else {
            return;
        };
        if rect.is_none() {
            if let Some(other) = self.hit_test(p).filter(|id| self.frame_data(*id).is_some()) {
                if other != from {
                    // From a top tab the frame clicked flows into the
                    // frame of the tab.
                    let (a, b) = if incoming {
                        (other, from)
                    } else {
                        (from, other)
                    };
                    self.link_frames(a, b);
                    self.select(vec![other]);
                    return;
                }
            }
        }
        let Some(src) = self.doc().find_shape(from).cloned() else {
            return;
        };
        let ShapeKind::Text {
            spans,
            frame: Some(size),
            align,
            para,
            ..
        } = &src.kind
        else {
            return;
        };
        let (origin, size) = match rect {
            Some(r) if r.width() > 1.0 && r.height() > 1.0 => (
                Point::new(r.x0, r.y0),
                tracedraw_core::geometry::Size::new(r.width(), r.height()),
            ),
            _ => (Point::new(p.x, p.y - size.height), *size),
        };
        let style = spans.last().cloned().map(|mut s| {
            s.text.clear();
            s
        });
        let Some(span) = style else {
            return;
        };
        let kind = ShapeKind::Text {
            spans: vec![span],
            origin,
            frame: Some(size),
            align: *align,
            para: para.clone(),
            on_path: None,
        };
        let Some(to) = self.new_shape(kind) else {
            return;
        };
        if incoming {
            // The new frame comes first: the text starts in it.
            self.link_frames(to, from);
        } else {
            self.link_frames(from, to);
        }
        self.select(vec![to]);
    }

    /// Unlink the selected frames from their neighbours; each keeps the text
    /// it currently shows.
    pub fn unlink_text_frames(&mut self) {
        let mut cmds = Vec::new();
        for id in self.selection.clone() {
            let Some(d) = self.frame_data(id) else {
                continue;
            };
            let next = data_id(&d, NEXT);
            let prev = data_id(&d, PREV);
            if next.is_none() && prev.is_none() {
                continue;
            }
            cmds.push(Command::SetObjectData {
                shape: id,
                data: with_key(with_key(d, NEXT, None), PREV, None),
            });
            if let Some(n) = next {
                if let Some(dn) = self.frame_data(n) {
                    cmds.push(Command::SetObjectData {
                        shape: n,
                        data: with_key(dn, PREV, None),
                    });
                }
            }
            if let Some(p) = prev {
                if let Some(dp) = self.frame_data(p) {
                    cmds.push(Command::SetObjectData {
                        shape: p,
                        data: with_key(dp, NEXT, None),
                    });
                }
            }
        }
        if !cmds.is_empty() {
            let _ = self.engine.run_batch("Unlink Frames", &cmds);
        }
    }

    /// Distribute the chain's text over its frames: each frame keeps what
    /// fits, the rest flows on; the last frame takes whatever is left.
    pub fn reflow_chain(&mut self, any: ShapeId) {
        let chain = self.chain_of(any);
        if chain.len() < 2 {
            return;
        }
        let Some(all) = self.chain_text(&chain) else {
            return;
        };
        let cmds = self.flow_commands(&chain, all);
        if !cmds.is_empty() {
            let _ = self.engine.run_batch("Text Flow", &cmds);
        }
    }

    /// The whole text of the frames `chain`, in that order; `None` when
    /// one of them is missing.
    fn chain_text(&self, chain: &[ShapeId]) -> Option<Vec<TextSpan>> {
        let mut all: Vec<TextSpan> = Vec::new();
        for id in chain {
            let s = self.doc().find_shape(*id)?;
            if let ShapeKind::Text { spans, .. } = &s.kind {
                all.extend(spans.iter().cloned());
            }
        }
        Some(all)
    }

    /// The commands that lay the text `all` over the frames `chain` in
    /// order: each frame keeps what fits, the rest flows on and the last
    /// frame takes whatever is left. Only frames whose text changes get a
    /// command.
    fn flow_commands(&self, chain: &[ShapeId], all: Vec<TextSpan>) -> Vec<Command> {
        let kinds: Vec<ShapeKind> = chain
            .iter()
            .filter_map(|id| self.doc().find_shape(*id).map(|s| s.kind.clone()))
            .collect();
        if kinds.len() != chain.len() {
            return Vec::new();
        }
        let fonts = tracedraw_text::fonts();
        let mut cmds = Vec::new();
        let mut rest = all;
        let n = chain.len();
        for (i, (id, kind)) in chain.iter().zip(kinds).enumerate() {
            let ShapeKind::Text {
                origin,
                frame,
                align,
                para,
                on_path,
                spans: old,
            } = kind
            else {
                continue;
            };
            let mine = if i + 1 == n {
                std::mem::take(&mut rest)
            } else {
                let layout = fonts.layout(&TextRequest {
                    spans: &rest,
                    frame,
                    align,
                    para: &para,
                    on_path: on_path.as_ref(),
                });
                let (head, tail) = tracedraw_core::split_spans_at(&rest, layout.fitted_chars);
                rest = tail;
                head
            };
            if mine == old {
                continue;
            }
            cmds.push(Command::SetShapeKind {
                shape: *id,
                kind: ShapeKind::Text {
                    spans: mine,
                    origin,
                    frame,
                    align,
                    para,
                    on_path,
                },
            });
        }
        cmds
    }

    /// The commands that keep the text of a chain when the objects
    /// `shapes` are deleted (#1986): a frame of a chain among them, or
    /// inside a group or clip frame among them, leaves its text to the
    /// chain. The frames left are linked to one another in their order,
    /// skipping the deleted ones, and the chain's whole text flows anew
    /// over them: the frame after a deleted one shows what it showed, and
    /// the last frame takes the rest, with the overflow mark when it does
    /// not fit. A chain of two left with one frame leaves an ordinary
    /// paragraph frame holding all the text. The commands are meant to
    /// run after the deletion, in its step, so one Undo brings the frame
    /// and the flow back together. Empty when no frame of a chain goes.
    pub fn flow_around_deleted(&self, shapes: &[ShapeId]) -> Vec<Command> {
        fn frames_in(s: &Shape, out: &mut Vec<ShapeId>) {
            match &s.kind {
                ShapeKind::Text { frame: Some(_), .. } => out.push(s.id),
                ShapeKind::Group { children } => {
                    for c in children {
                        frames_in(c, out);
                    }
                }
                ShapeKind::ClipFrame { frame, contents } => {
                    frames_in(frame, out);
                    for c in contents {
                        frames_in(c, out);
                    }
                }
                _ => {}
            }
        }
        let mut gone = Vec::new();
        for id in shapes {
            if let Some(s) = self.doc().find_shape(*id) {
                frames_in(s, &mut gone);
            }
        }
        gone.retain(|id| self.is_linked_frame(*id));
        let mut heads: Vec<ShapeId> = Vec::new();
        for id in &gone {
            let head = self.chain_head(*id);
            if !heads.contains(&head) {
                heads.push(head);
            }
        }
        let mut cmds = Vec::new();
        for head in heads {
            let chain = self.chain_of(head);
            let Some(all) = self.chain_text(&chain) else {
                continue;
            };
            let left: Vec<ShapeId> = chain
                .iter()
                .copied()
                .filter(|id| !gone.contains(id))
                .collect();
            if left.is_empty() {
                continue;
            }
            // The frames left are linked in order, the first with nothing
            // before it and the last with nothing after it.
            for (i, id) in left.iter().enumerate() {
                let Some(data) = self.frame_data(*id) else {
                    continue;
                };
                let prev = i.checked_sub(1).and_then(|j| left.get(j).copied());
                let next = left.get(i + 1).copied();
                if data_id(&data, PREV) == prev && data_id(&data, NEXT) == next {
                    continue;
                }
                cmds.push(Command::SetObjectData {
                    shape: *id,
                    data: with_key(with_key(data, PREV, prev), NEXT, next),
                });
            }
            cmds.extend(self.flow_commands(&left, all));
        }
        cmds
    }

    /// Re-flow every chain that touches the given shapes (after a resize).
    pub fn reflow_chains_of(&mut self, ids: &[ShapeId]) {
        let mut done: Vec<ShapeId> = Vec::new();
        for id in ids {
            if !self.is_linked_frame(*id) {
                continue;
            }
            let head = self.chain_head(*id);
            if done.contains(&head) {
                continue;
            }
            done.push(head);
            self.reflow_chain(head);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Point, Size};
    use tracedraw_core::TextAlign;

    fn frame(app: &mut App, text: &str, w: f64, h: f64) -> ShapeId {
        let kind = ShapeKind::Text {
            spans: vec![TextSpan::new(text, "DejaVu Sans", 12.0)],
            origin: Point::new(10.0, 10.0),
            frame: Some(Size::new(w, h)),
            align: TextAlign::Left,
            para: Default::default(),
            on_path: None,
        };
        app.new_shape(kind).unwrap()
    }

    #[test]
    fn linking_flows_overflow_into_the_second_frame() {
        tracedraw_text::install();
        let mut app = App::headless();
        let long = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";
        let a = frame(&mut app, long, 30.0, 8.0);
        let b = frame(&mut app, "", 60.0, 60.0);
        app.select(vec![a, b]);
        app.link_text_frames();
        assert_eq!(app.next_frame(a), Some(b));
        let text_of = |app: &App, id: ShapeId| -> String {
            match &app.doc().find_shape(id).unwrap().kind {
                ShapeKind::Text { spans, .. } => spans.iter().map(|s| s.text.as_str()).collect(),
                _ => String::new(),
            }
        };
        let ta = text_of(&app, a);
        let tb = text_of(&app, b);
        assert!(!ta.is_empty() && !tb.is_empty(), "a={ta:?} b={tb:?}");
        assert_eq!(format!("{ta}{tb}"), long);
        // Unlinking keeps the split text in place.
        app.select(vec![a]);
        app.unlink_text_frames();
        assert_eq!(app.next_frame(a), None);
        assert_eq!(text_of(&app, a), ta);
    }

    /// The flow tab (#371): a click on it, then a click on another page's
    /// empty space makes a frame there that the text flows into; a click
    /// on a frame links to that frame.
    #[test]
    fn the_flow_tab_links_to_a_new_frame_on_another_page() {
        tracedraw_text::install();
        let mut app = App::headless();
        app.canvas_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
        let page = app.page_rect();
        app.view.fit(page, app.canvas_rect);
        let long = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";
        let a = frame(&mut app, long, 30.0, 8.0);
        app.select(vec![a]);
        // The tab hangs under the bottom edge's middle (10 to 40 at y 10).
        let edge = app.view.to_screen(Point::new(25.0, 10.0));
        let tab = crate::canvas::flow_tab_rect(
            app.view.to_screen(Point::new(10.0, 10.0)),
            app.view.to_screen(Point::new(40.0, 10.0)),
            app.handle_size(),
        );
        assert!(tab.top() > edge.y);
        assert_eq!(app.flow_tab_at(tab.center()), Some(a));
        app.start_text_link(a);
        assert_eq!(app.pending_text_link, Some(a));
        // On to page 2, a click on its empty space.
        let first_page = app.page;
        app.add_page();
        assert_ne!(app.page, first_page);
        app.finish_text_link(Point::new(50.0, 150.0), None);
        assert!(app.pending_text_link.is_none());
        let b = app.next_frame(a).expect("a new frame");
        let s = app.doc().find_shape(b).cloned().expect("frame");
        let corners = s.text_frame_corners().expect("a frame");
        // Its top left at the click, the first frame's size.
        assert!((corners[3].x - 50.0).abs() < 1e-9 && (corners[3].y - 150.0).abs() < 1e-9);
        let text = |id: ShapeId| match &app.doc().find_shape(id).unwrap().kind {
            ShapeKind::Text { spans, .. } => {
                spans.iter().map(|s| s.text.as_str()).collect::<String>()
            }
            _ => String::new(),
        };
        assert!(!text(b).is_empty());
        assert_eq!(format!("{}{}", text(a), text(b)), long);
        // A frame that flows on has no tab to click.
        app.page = first_page;
        app.select(vec![a]);
        assert_eq!(app.flow_tab_at(tab.center()), None);
    }

    fn text_of(app: &App, id: ShapeId) -> String {
        match app.doc().find_shape(id).map(|s| &s.kind) {
            Some(ShapeKind::Text { spans, .. }) => spans.iter().map(|s| s.text.as_str()).collect(),
            _ => String::new(),
        }
    }

    const LONG: &str = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";

    /// A chain of three frames, the first two a line high, the third
    /// large enough for the rest; with the text each shows.
    fn chain_of_three(app: &mut App) -> ([ShapeId; 3], [String; 3]) {
        let a = frame(app, LONG, 30.0, 8.0);
        let b = frame(app, "", 30.0, 8.0);
        let c = frame(app, "", 60.0, 60.0);
        assert!(app.link_frames(a, b));
        assert!(app.link_frames(b, c));
        assert_eq!(app.chain_of(a), vec![a, b, c]);
        let texts = [text_of(app, a), text_of(app, b), text_of(app, c)];
        assert!(
            texts.iter().all(|t| !t.is_empty()),
            "every frame shows some of the text: {texts:?}"
        );
        assert_eq!(texts.concat(), LONG);
        ([a, b, c], texts)
    }

    /// Deleting the middle frame of a chain (#1986): the frame goes, its
    /// text does not. The frames left are a chain of two, the text flows
    /// on into the last one (which shows what the deleted one had), and
    /// one Undo brings the frame and the flow back.
    #[test]
    fn deleting_the_middle_frame_of_a_chain_keeps_its_text() {
        tracedraw_text::install();
        let mut app = App::headless();
        let ([a, b, c], [ta, tb, tc]) = chain_of_three(&mut app);
        app.select(vec![b]);
        app.delete_selection();
        assert!(app.doc().find_shape(b).is_none(), "the frame is deleted");
        assert_eq!(app.chain_of(a), vec![a, c], "a chain of two is left");
        assert_eq!(app.next_frame(a), Some(c));
        assert_eq!(app.prev_frame(c), Some(a));
        assert_eq!(text_of(&app, a), ta, "the first frame shows the same");
        assert_eq!(
            text_of(&app, c),
            format!("{tb}{tc}"),
            "the last frame takes what the deleted one had"
        );
        assert_eq!(format!("{}{}", text_of(&app, a), text_of(&app, c)), LONG);
        assert!(app.selection.is_empty());
        // One Undo: the three frames and their flow as before.
        assert_eq!(app.engine.undo_label(), Some("Delete"));
        app.undo();
        assert_eq!(app.chain_of(a), vec![a, b, c]);
        assert_eq!(text_of(&app, a), ta);
        assert_eq!(text_of(&app, b), tb);
        assert_eq!(text_of(&app, c), tc);
        // Redo deletes it again, with the flow.
        app.redo();
        assert_eq!(app.chain_of(a), vec![a, c]);
        assert_eq!(text_of(&app, c), format!("{tb}{tc}"));
    }

    /// Deleting the first frame: the text starts in the second, which
    /// shows the start of it, and flows on as before.
    #[test]
    fn deleting_the_first_frame_of_a_chain_keeps_its_text() {
        tracedraw_text::install();
        let mut app = App::headless();
        let ([a, b, c], [ta, tb, tc]) = chain_of_three(&mut app);
        app.select(vec![a]);
        app.delete_selection();
        assert!(app.doc().find_shape(a).is_none());
        assert_eq!(app.chain_of(c), vec![b, c]);
        assert_eq!(app.prev_frame(b), None, "the second frame is now the head");
        assert_eq!(app.next_frame(b), Some(c));
        // The second frame is the first one's size: it shows the same
        // start of the text.
        assert_eq!(text_of(&app, b), ta);
        assert_eq!(format!("{}{}", text_of(&app, b), text_of(&app, c)), LONG);
        app.undo();
        assert_eq!(app.chain_of(a), vec![a, b, c]);
        assert_eq!(
            [text_of(&app, a), text_of(&app, b), text_of(&app, c)],
            [ta, tb, tc]
        );
    }

    /// Deleting the last frame: the frame before it takes the rest of
    /// the text and shows the overflow mark, since it does not fit.
    #[test]
    fn deleting_the_last_frame_of_a_chain_keeps_its_text() {
        tracedraw_text::install();
        let mut app = App::headless();
        let ([a, b, c], [ta, tb, tc]) = chain_of_three(&mut app);
        app.select(vec![c]);
        app.delete_selection();
        assert!(app.doc().find_shape(c).is_none());
        assert_eq!(app.chain_of(a), vec![a, b]);
        assert_eq!(app.next_frame(b), None, "the second frame is now the end");
        assert_eq!(text_of(&app, a), ta);
        assert_eq!(text_of(&app, b), format!("{tb}{tc}"));
        let shape = app.doc().find_shape(b).cloned().expect("frame");
        assert!(app.text_overflows(&shape), "the rest does not fit");
        app.select(vec![b]);
        let shown = app
            .shown_text_frames(None)
            .into_iter()
            .find(|f| f.id == b)
            .expect("the frame is shown");
        assert!(shown.overflow && !shown.linked && shown.receives);
        app.undo();
        assert_eq!(app.chain_of(a), vec![a, b, c]);
        assert_eq!(
            [text_of(&app, a), text_of(&app, b), text_of(&app, c)],
            [ta, tb, tc]
        );
        let shape = app.doc().find_shape(b).cloned().expect("frame");
        assert!(!app.text_overflows(&shape), "the text flows on again");
    }

    /// Deleting a frame of a chain of two leaves one ordinary paragraph
    /// frame with all the text: the small one overflows, the large one
    /// holds it.
    #[test]
    fn deleting_a_frame_of_a_chain_of_two_leaves_one_frame_with_all_the_text() {
        tracedraw_text::install();
        let mut app = App::headless();
        let a = frame(&mut app, LONG, 30.0, 8.0);
        let b = frame(&mut app, "", 60.0, 60.0);
        assert!(app.link_frames(a, b));
        let (ta, tb) = (text_of(&app, a), text_of(&app, b));
        assert!(!ta.is_empty() && !tb.is_empty());
        // The large frame deleted: the small one holds all the text and
        // overflows, linked to nothing.
        app.select(vec![b]);
        app.delete_selection();
        assert!(app.doc().find_shape(b).is_none());
        assert!(!app.is_linked_frame(a));
        assert_eq!(app.chain_of(a), vec![a]);
        assert_eq!(text_of(&app, a), LONG);
        let shape = app.doc().find_shape(a).cloned().expect("frame");
        assert!(app.text_overflows(&shape));
        assert_eq!(
            app.doc().find_shape(a).map(|s| s.data.clone()),
            Some(Vec::new()),
            "no link is left in the frame's data"
        );
        app.undo();
        assert_eq!(app.chain_of(a), vec![a, b]);
        assert_eq!((text_of(&app, a), text_of(&app, b)), (ta, tb));
        // The small frame deleted: the large one holds all the text.
        app.select(vec![a]);
        app.delete_selection();
        assert!(app.doc().find_shape(a).is_none());
        assert!(!app.is_linked_frame(b));
        assert_eq!(text_of(&app, b), LONG);
        let shape = app.doc().find_shape(b).cloned().expect("frame");
        assert!(!app.text_overflows(&shape));
    }

    /// Cut takes a frame of a chain out the same way: the text stays
    /// with the chain (the clipboard has the frame as it showed).
    #[test]
    fn cutting_a_frame_of_a_chain_keeps_its_text() {
        tracedraw_text::install();
        let mut app = App::headless();
        let ([a, b, c], [ta, tb, tc]) = chain_of_three(&mut app);
        app.select(vec![b]);
        app.cut();
        assert!(app.doc().find_shape(b).is_none());
        assert_eq!(app.chain_of(a), vec![a, c]);
        assert_eq!(text_of(&app, a), ta);
        assert_eq!(text_of(&app, c), format!("{tb}{tc}"));
        app.undo();
        assert_eq!(app.chain_of(a), vec![a, b, c]);
        assert_eq!(text_of(&app, b), tb);
    }

    /// A frame of a chain deleted inside a group goes the same way: the
    /// chain keeps its text.
    #[test]
    fn deleting_a_group_holding_a_frame_of_a_chain_keeps_its_text() {
        tracedraw_text::install();
        let mut app = App::headless();
        let ([a, b, c], [ta, tb, tc]) = chain_of_three(&mut app);
        let other = frame(&mut app, "aside", 20.0, 20.0);
        app.select(vec![b, other]);
        app.group_selection();
        let group = app.selection[0];
        assert_ne!(group, b);
        assert_eq!(
            app.chain_of(a),
            vec![a, b, c],
            "a grouped frame stays in its chain"
        );
        app.select(vec![group]);
        app.delete_selection();
        assert!(app.doc().find_shape(b).is_none());
        assert_eq!(app.chain_of(a), vec![a, c]);
        assert_eq!(text_of(&app, a), ta);
        assert_eq!(text_of(&app, c), format!("{tb}{tc}"));
        app.undo();
        assert_eq!(app.chain_of(a), vec![a, b, c]);
        assert_eq!(text_of(&app, b), tb);
        assert_eq!(text_of(&app, c), tc);
    }

    /// A copy of one frame of a chain is a frame of its own: editing it
    /// does not flow into the original's next frame. A chain copied whole
    /// is a chain of the copies.
    #[test]
    fn copies_leave_the_chain_unless_copied_whole() {
        tracedraw_text::install();
        let mut app = App::headless();
        let long = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty";
        let a = frame(&mut app, long, 30.0, 8.0);
        let b = frame(&mut app, "", 60.0, 60.0);
        app.select(vec![a, b]);
        app.link_text_frames();
        let text_of = |app: &App, id: ShapeId| -> String {
            match &app.doc().find_shape(id).unwrap().kind {
                ShapeKind::Text { spans, .. } => spans.iter().map(|s| s.text.as_str()).collect(),
                _ => String::new(),
            }
        };
        let tb = text_of(&app, b);
        // One frame copied: the copy is linked to nothing, and typing in
        // it leaves the original's chain alone.
        app.select(vec![a]);
        app.duplicate();
        let a2 = app.selection[0];
        assert_ne!(a2, a);
        assert!(!app.is_linked_frame(a2));
        app.begin_text_edit(a2);
        app.text_insert("more words here and there ");
        assert_eq!(
            text_of(&app, b),
            tb,
            "the original's next frame is untouched"
        );
        assert_eq!(app.next_frame(a), Some(b));
        // The copy pasted: the same.
        app.select(vec![b]);
        app.copy();
        app.paste();
        let b2 = app.selection[0];
        assert!(!app.is_linked_frame(b2));
        // The whole chain copied: the copies are a chain of their own.
        app.select(vec![a, b]);
        app.duplicate();
        let (a3, b3) = (app.selection[0], app.selection[1]);
        assert_eq!(app.next_frame(a3), Some(b3));
        assert_eq!(app.chain_of(b3), vec![a3, b3]);
        assert_eq!(app.next_frame(a), Some(b), "the original chain stays");
        // Copies of a copied chain are made with every other copy (reid):
        // nothing links back to the originals.
        app.select(vec![a3]);
        app.duplicate();
        assert!(!app.is_linked_frame(app.selection[0]));
    }
}
