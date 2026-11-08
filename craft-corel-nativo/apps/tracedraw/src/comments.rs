//! Review comments (Comments docker): notes pinned to a point, a box or
//! an arrow on a page, with replies, resolved or open. They are part of
//! the drawing (saved in `.tdraw`) but never printed or exported. Every
//! change goes through `Command::SetComments`, so it can be undone.

use crate::app::App;
use egui::{Color32, Painter, Pos2, Stroke as EStroke};
use tracedraw_core::document::{Comment, CommentMark, CommentReply};
use tracedraw_core::geometry::{Point, Rect};
use tracedraw_core::Command;

/// The mark the next press on the page places.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    Pin,
    Rect,
    Arrow,
}

/// The docker's state.
#[derive(Debug, Clone, Default)]
pub struct CommentsState {
    /// Waiting for a press (and drag) on the page to place a mark.
    pub placing: Option<MarkKind>,
    /// Where the drag that places a box or an arrow began.
    pub drag_from: Option<Point>,
    pub drag_to: Option<Point>,
    /// The comment picked in the list or on the page.
    pub selected: Option<u32>,
    /// The reply being typed for the picked comment.
    pub reply: String,
    pub show_resolved: bool,
    /// Marks hidden on the page.
    pub hidden: bool,
    /// The picked comment's text field takes the keyboard next frame.
    pub focus_text: bool,
    /// Text typed into the picked comment, recorded when the field is
    /// left.
    pub drafts: std::collections::HashMap<u32, String>,
}

/// Comment colour: an orange that reads on white and on dark pages.
pub const MARK: Color32 = Color32::from_rgb(0xF2, 0x8C, 0x28);
const PIN_RADIUS: f32 = 9.0;

impl App {
    /// Who new comments and replies are from: the name in the docker, else
    /// the drawing's author, else "Me".
    pub fn comment_author(&self) -> String {
        let name = self.settings.comment_author.trim();
        if !name.is_empty() {
            return name.to_string();
        }
        let author = self.doc().metadata.author.trim();
        if !author.is_empty() {
            return author.to_string();
        }
        crate::i18n::tr("comments.me")
    }

    fn set_comments(&mut self, comments: Vec<Comment>) {
        self.run(Command::SetComments { comments });
    }

    /// Record text still being typed into a comment (before another one
    /// is picked, added or deleted).
    pub fn commit_comment_drafts(&mut self) {
        let drafts: Vec<(u32, String)> = self.comments.drafts.drain().collect();
        for (id, text) in drafts {
            self.edit_comment(id, |c| c.text = text);
        }
    }

    /// Add a comment with `mark` on the current page; it is picked and its
    /// text is typed in the docker. Returns its id.
    pub fn add_comment(&mut self, mark: CommentMark, text: &str) -> u32 {
        self.commit_comment_drafts();
        let mut comments = self.doc().comments.clone();
        let id = comments.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        comments.push(Comment {
            id,
            page: self.page,
            mark,
            author: self.comment_author(),
            time: crate::time_tracker::unix_now(),
            text: text.to_string(),
            replies: Vec::new(),
            resolved: false,
        });
        self.set_comments(comments);
        self.comments.selected = Some(id);
        id
    }

    /// Change one comment through `f` (one undo step).
    pub fn edit_comment(&mut self, id: u32, f: impl FnOnce(&mut Comment)) {
        let mut comments = self.doc().comments.clone();
        let Some(c) = comments.iter_mut().find(|c| c.id == id) else {
            return;
        };
        let before = c.clone();
        f(c);
        if *c != before {
            self.set_comments(comments);
        }
    }

    pub fn reply_to_comment(&mut self, id: u32, text: &str) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let reply = CommentReply {
            author: self.comment_author(),
            time: crate::time_tracker::unix_now(),
            text: text.to_string(),
        };
        self.edit_comment(id, |c| c.replies.push(reply));
    }

    pub fn delete_comment(&mut self, id: u32) {
        self.comments.drafts.remove(&id);
        self.commit_comment_drafts();
        let mut comments = self.doc().comments.clone();
        let before = comments.len();
        comments.retain(|c| c.id != id);
        if comments.len() != before {
            self.set_comments(comments);
        }
        if self.comments.selected == Some(id) {
            self.comments.selected = None;
        }
    }

    /// Show a comment: its page becomes current and it is picked.
    pub fn go_to_comment(&mut self, id: u32) {
        self.commit_comment_drafts();
        let Some(page) = self
            .doc()
            .comments
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.page)
        else {
            return;
        };
        if page != self.page {
            if let Some(i) = self.doc().pages.iter().position(|p| p.id == page) {
                self.goto_page(i);
            }
        }
        self.comments.selected = Some(id);
    }

    /// The comment whose mark is under a screen point on the current
    /// page (the number bubble, or inside a box), the last drawn first.
    pub fn comment_at(&self, s: Pos2) -> Option<u32> {
        if self.comments.hidden {
            return None;
        }
        let p = self.view.to_page(s);
        self.doc()
            .comments
            .iter()
            .rev()
            .filter(|c| c.page == self.page && (self.comments.show_resolved || !c.resolved))
            .find(|c| {
                let bubble = self.view.to_screen(c.mark.anchor());
                if (bubble - s).length() <= PIN_RADIUS + 2.0 {
                    return true;
                }
                match c.mark {
                    CommentMark::Rect { rect } => rect.contains(p),
                    _ => false,
                }
            })
            .map(|c| c.id)
    }

    /// The press, drag and release that place a mark while the docker
    /// waits for one; a press on a comment's mark picks it. True when the
    /// pointer was used here (the tools do not see it).
    pub fn comments_pointer(&mut self, response: &egui::Response) -> bool {
        let Some(pos) = response.interact_pointer_pos().or(response.hover_pos()) else {
            return false;
        };
        let p = self.view.to_page(pos);
        if let Some(kind) = self.comments.placing {
            if response.drag_started_by(egui::PointerButton::Primary) {
                self.comments.drag_from = Some(p);
                self.comments.drag_to = Some(p);
            }
            if response.dragged_by(egui::PointerButton::Primary) {
                self.comments.drag_to = Some(p);
            }
            let released = response.drag_stopped() || response.clicked();
            if released {
                let from = self.comments.drag_from.take().unwrap_or(p);
                let to = self.comments.drag_to.take().unwrap_or(p);
                let mark = match kind {
                    MarkKind::Pin => CommentMark::Pin { at: to },
                    MarkKind::Rect if (to - from).hypot() > 1.0 => CommentMark::Rect {
                        rect: Rect::from_points(from, to),
                    },
                    MarkKind::Arrow if (to - from).hypot() > 1.0 => CommentMark::Arrow { from, to },
                    // A click with the box or arrow: a pin there.
                    _ => CommentMark::Pin { at: to },
                };
                self.add_comment(mark, "");
                self.comments.placing = None;
                self.comments.focus_text = true;
            }
            return true;
        }
        if response.clicked_by(egui::PointerButton::Primary)
            && self.docker_shown(crate::app::DockerTab::Comments)
        {
            if let Some(id) = self.comment_at(pos) {
                self.comments.selected = Some(id);
                return true;
            }
        }
        false
    }
}

/// An arrowhead at `tip` coming from `from`.
fn arrow_head(painter: &Painter, from: Pos2, tip: Pos2, stroke: EStroke) {
    let d = (tip - from).normalized();
    if !d.x.is_finite() {
        return;
    }
    let n = egui::vec2(-d.y, d.x);
    let a = tip - d * 10.0 + n * 5.0;
    let b = tip - d * 10.0 - n * 5.0;
    painter.add(egui::Shape::convex_polygon(
        vec![tip, a, b],
        stroke.color,
        EStroke::NONE,
    ));
}

/// The current page's comment marks: boxes and arrows in orange, each
/// with its number in a bubble; the picked one stronger. Resolved ones
/// only when the docker shows them, faded.
pub fn draw_comments(app: &App, painter: &Painter) {
    if app.comments.hidden && app.comments.placing.is_none() {
        return;
    }
    let view = &app.view;
    for (n, c) in app.doc().comments.iter().enumerate() {
        if c.page != app.page || (c.resolved && !app.comments.show_resolved) {
            continue;
        }
        let picked = app.comments.selected == Some(c.id);
        let color = if c.resolved {
            MARK.gamma_multiply(0.45)
        } else {
            MARK
        };
        let line = EStroke::new(if picked { 2.5 } else { 1.5 }, color);
        match c.mark {
            CommentMark::Pin { .. } => {}
            CommentMark::Rect { rect } => {
                painter.rect_stroke(
                    view.rect_to_screen(rect),
                    2.0,
                    line,
                    egui::StrokeKind::Middle,
                );
            }
            CommentMark::Arrow { from, to } => {
                let (a, b) = (view.to_screen(from), view.to_screen(to));
                painter.line_segment([a, b], line);
                arrow_head(painter, a, b, line);
            }
        }
        let at = view.to_screen(c.mark.anchor());
        painter.circle_filled(at, PIN_RADIUS + if picked { 2.0 } else { 0.0 }, color);
        painter.circle_stroke(at, PIN_RADIUS, EStroke::new(1.0, Color32::WHITE));
        painter.text(
            at,
            egui::Align2::CENTER_CENTER,
            (n + 1).to_string(),
            egui::FontId::proportional(11.0),
            Color32::WHITE,
        );
    }
    // The box or arrow being dragged out.
    if let (Some(kind), Some(a), Some(b)) = (
        app.comments.placing,
        app.comments.drag_from,
        app.comments.drag_to,
    ) {
        let line = EStroke::new(1.5, MARK);
        match kind {
            MarkKind::Rect => {
                painter.rect_stroke(
                    view.rect_to_screen(Rect::from_points(a, b)),
                    2.0,
                    line,
                    egui::StrokeKind::Middle,
                );
            }
            MarkKind::Arrow => {
                let (sa, sb) = (view.to_screen(a), view.to_screen(b));
                painter.line_segment([sa, sb], line);
                arrow_head(painter, sa, sb, line);
            }
            MarkKind::Pin => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_are_added_answered_resolved_and_deleted_as_undoable_steps() {
        let mut app = App::headless();
        app.settings.comment_author = "Rita".into();
        let id = app.add_comment(
            CommentMark::Rect {
                rect: Rect::new(10.0, 10.0, 60.0, 40.0),
            },
            "Logo too small",
        );
        assert_eq!(app.comments.selected, Some(id));
        let c = app.doc().comments[0].clone();
        assert_eq!(
            (c.author.as_str(), c.text.as_str()),
            ("Rita", "Logo too small")
        );
        assert!(c.time > 1_600_000_000);
        app.reply_to_comment(id, "  Bigger now ");
        app.reply_to_comment(id, "   ");
        assert_eq!(app.doc().comments[0].replies.len(), 1);
        assert_eq!(app.doc().comments[0].replies[0].text, "Bigger now");
        app.edit_comment(id, |c| c.resolved = true);
        assert!(app.doc().comments[0].resolved);
        app.undo();
        assert!(!app.doc().comments[0].resolved);
        // A resolved comment is hidden on the page unless asked for.
        let inside = app.view.to_screen(Point::new(30.0, 20.0));
        assert_eq!(app.comment_at(inside), Some(id));
        app.edit_comment(id, |c| c.resolved = true);
        assert_eq!(app.comment_at(inside), None);
        app.comments.show_resolved = true;
        assert_eq!(app.comment_at(inside), Some(id));
        app.delete_comment(id);
        assert!(app.doc().comments.is_empty());
        assert_eq!(app.comments.selected, None);
        // Text being typed is kept when another comment is added.
        let a = app.add_comment(
            CommentMark::Pin {
                at: Point::new(1.0, 1.0),
            },
            "",
        );
        app.comments.drafts.insert(a, "Typed".into());
        app.add_comment(
            CommentMark::Pin {
                at: Point::new(2.0, 2.0),
            },
            "",
        );
        assert_eq!(app.doc().comments[0].text, "Typed");
        assert!(app.comments.drafts.is_empty());
        // Without a name, the drawing's author, then "Me".
        app.settings.comment_author.clear();
        assert_eq!(app.comment_author(), crate::i18n::tr("comments.me"));
    }

    #[test]
    fn a_comment_on_another_page_takes_the_view_there() {
        let mut app = App::headless();
        let first = app.page;
        app.add_page();
        let id = app.add_comment(
            CommentMark::Pin {
                at: Point::new(5.0, 5.0),
            },
            "Here",
        );
        let second = app.page;
        assert_ne!(first, second);
        app.goto_page(0);
        app.comments.selected = None;
        app.go_to_comment(id);
        assert_eq!((app.page, app.comments.selected), (second, Some(id)));
    }
}
