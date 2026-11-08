//! Moving the selection to another page by dragging it onto that page's
//! tab (#1346). While the Pick tool moves the selection, the page tab
//! under the pointer lights up; resting there for
//! [`PAGE_TAB_HOVER_SECS`] sends the selection to that page, which
//! becomes the current one, and the move goes on there (the objects
//! stay where they were on the page while the pointer is over the tabs).
//! Let go on a tab, the selection goes to that page at once. Either way
//! the objects keep their place on the page; with Ctrl held (or a right
//! click during the move) copies go and the originals stay. The whole
//! drag is one undo step. See `docs/behavior/pages.md`, Moving objects to another page by its tab.

use crate::app::App;
use crate::app::Drag;
use egui::Pos2;
use tracedraw_core::{Command, LayerId, ShapeId};

/// How long a moved selection rests on a page tab before that page
/// opens, in seconds.
pub const PAGE_TAB_HOVER_SECS: f64 = 0.6;

impl App {
    /// The page (by index) whose tab is at `at` (screen), as last drawn.
    pub fn page_tab_at(&self, at: Pos2) -> Option<usize> {
        self.page_tab_rects
            .iter()
            .find(|(_, r)| r.contains(at))
            .map(|(i, _)| *i)
    }

    /// The page tab under the pointer.
    pub fn page_tab_under_pointer(&self) -> Option<usize> {
        self.pointer_screen.and_then(|p| self.page_tab_at(p))
    }

    /// The layer of page `idx` that objects sent there go to: its
    /// topmost visible, unlocked layer, else its topmost unlocked one
    /// (as [`App::active_layer`] chooses for the current page).
    fn page_drop_layer(&self, idx: usize) -> Option<LayerId> {
        let page = self.doc().pages.get(idx)?;
        if page.id == self.page {
            return self.active_layer();
        }
        page.layers
            .iter()
            .rev()
            .find(|l| !l.locked && l.visible)
            .or_else(|| page.layers.iter().rev().find(|l| !l.locked))
            .map(|l| l.id)
    }

    /// Send the selection to page `idx`, at the same place on the page:
    /// moved, or copied when `copy` (the originals stay). That page
    /// becomes the current one with the objects (or their copies)
    /// selected. False when nothing went (no selection, the page has no
    /// unlocked layer, it is the current page).
    pub fn send_selection_to_page(&mut self, idx: usize, copy: bool) -> bool {
        if self.selection.is_empty() || idx == self.page_index() {
            return false;
        }
        let Some(layer) = self.page_drop_layer(idx) else {
            self.status = crate::i18n::tr("shelf.no_layer");
            return false;
        };
        if !copy {
            let ids = self.selection.clone();
            crate::ui::objects_docker::move_to_layer(self, &ids, layer);
            return true;
        }
        let mut cmds = Vec::new();
        let mut copies: Vec<ShapeId> = Vec::new();
        for s in self.selected_shapes() {
            let id = self.engine.new_shape_id();
            let shape = crate::app::reid_pub(s, id, &mut self.engine);
            copies.push(shape.id);
            cmds.push(Command::AddShape { layer, shape });
        }
        if cmds.is_empty() {
            return false;
        }
        if let Err(e) = self.engine.run_batch("Copy to Page", &cmds) {
            self.status = e.to_string();
            return false;
        }
        self.goto_page(idx);
        self.picked_layer = Some(layer);
        self.select(copies);
        true
    }

    /// Each frame, after the page tabs are drawn: while the selection is
    /// moved, the tab under the pointer and how long it has been there;
    /// resting [`PAGE_TAB_HOVER_SECS`] on another page's tab sends the
    /// selection there and the move goes on on that page. `time` is the
    /// frame's time (seconds), `ctrl` whether Ctrl (Cmd) is held.
    pub fn track_page_tab_drag(&mut self, time: f64, ctrl: bool) {
        if !matches!(self.drag, Drag::Move { .. }) {
            self.page_tab_hover = None;
            self.end_page_drag_group();
            return;
        }
        self.page_drag_ctrl = ctrl;
        let over = self.page_tab_under_pointer();
        match over {
            Some(i) if i != self.page_index() => match self.page_tab_hover {
                Some((j, since)) if j == i => {
                    if time - since >= PAGE_TAB_HOVER_SECS {
                        self.page_tab_hover = None;
                        if self.page_drag_group.is_none() {
                            self.page_drag_group = Some(self.engine.begin_group());
                        }
                        let copy = ctrl || std::mem::take(&mut self.drag_copy);
                        self.send_selection_to_page(i, copy);
                    }
                }
                _ => self.page_tab_hover = Some((i, time)),
            },
            _ => self.page_tab_hover = None,
        }
        // On the tabs after the page changed: the objects stay where they
        // are on the page, and the move goes on from the pointer.
        if over.is_some() && self.page_drag_group.is_some() {
            if let Some(at) = self.pointer_screen {
                let p = self.view.to_page(at);
                self.start_move(p);
            }
        }
    }

    /// A move let go on a page tab: the selection goes to that page, at
    /// its place there (the move's way to the tab is not applied).
    pub fn drop_on_page_tab(&mut self, idx: usize) {
        let copy = std::mem::take(&mut self.drag_copy) || self.page_drag_ctrl;
        self.drop_frame = None;
        self.send_selection_to_page(idx, copy);
        self.end_page_drag_group();
    }

    /// The move that changed page is over: its steps become one.
    pub fn end_page_drag_group(&mut self) {
        self.page_tab_hover = None;
        if let Some(mark) = self.page_drag_group.take() {
            self.engine.end_group(mark, "Move to Page");
        }
    }
}
