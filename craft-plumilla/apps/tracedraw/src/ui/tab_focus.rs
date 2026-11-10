//! Tab and Shift+Tab move the keyboard focus inside the window that has
//! it (#1258), for every window and area: the floating panels, the Font
//! Manager, the palette windows, the effect picker, macro forms, the
//! dialogs and the rest. egui hands the focus on in the order the widgets
//! were drawn over the whole screen, so from a window's last control (or
//! with nothing focused) it went to the next window drawn, or round to
//! the menu bar, drawn first, whose File menu then took the keys.
//!
//! Here, at the end of a frame with a Tab, the place the focus should go
//! is worked out from the widgets of the focused widget's own layer:
//! the next (Shift+Tab: the previous) focusable control drawn there, and
//! past the last (Shift+Tab: before the first) its first (last) input
//! box, or its first (last) control when it has none. With nothing
//! focused, Tab goes to the first input box of the window on top (Shift+
//! Tab to its last). The main window counts as one window without its
//! menu bar: Tab never lands there, and with nothing focused and no
//! window open it stays with the drawing (Tab steps through the
//! objects). The next frame puts the focus there before anything is
//! drawn, as egui's own move (a Shift+Tab, a Tab past the last widget)
//! lands only then. A control that keeps Tab for itself (a code editor
//! typing a tab) keeps the focus; with a menu or a list open nothing
//! changes. See `docs/behavior/shortcuts.md`, Modal dialogs.

use egui::{Context, Id, LayerId, Order, Rect};

/// Where a Tab of the last frame sends the focus.
#[derive(Clone, Copy, Debug)]
struct Pending {
    /// The focused widget when the Tab came.
    from: Option<Id>,
    /// The widget to focus; `None`: no widget (the drawing keeps Tab).
    to: Option<Id>,
    /// The layer the focus moves in.
    layer: LayerId,
}

fn pending_id() -> Id {
    Id::new("tab_focus_pending")
}

fn start_id() -> Id {
    Id::new("tab_focus_start")
}

fn menu_bar_id() -> Id {
    Id::new("tab_focus_menu_bar")
}

/// Takes over a move egui still hands to the next widget drawn.
fn sink_id() -> Id {
    Id::new("tab_focus_sink")
}

/// The menu bar's place this frame: its controls are left out of the
/// main window's Tab order.
pub fn set_menu_bar(ctx: &Context, rect: Rect) {
    ctx.data_mut(|d| d.insert_temp(menu_bar_id(), rect));
}

/// Before anything of the frame is drawn: the focus goes where the last
/// frame's Tab sends it, unless the focused control kept the Tab (the
/// focus is still where it was); then the focus the frame starts with is
/// kept for [`end`].
pub fn begin(ctx: &Context) {
    let pending = ctx.data_mut(|d| {
        let p = d.get_temp::<Pending>(pending_id());
        d.remove::<Pending>(pending_id());
        p
    });
    if let Some(p) = pending {
        let now = ctx.memory(|m| m.focused());
        // From nothing focused the Tab always moves; from a control
        // only when it let the Tab go (egui moved the focus on).
        let moved = p.from.is_none() || now != p.from;
        if moved && !egui::Popup::is_any_open(ctx) {
            ctx.memory_mut(|m| {
                m.interested_in_focus(sink_id(), p.layer);
                match p.to {
                    Some(id) => m.request_focus(id),
                    None => {
                        if let Some(f) = m.focused() {
                            m.surrender_focus(f);
                        }
                    }
                }
            });
        }
    }
    let focused = ctx.memory(|m| m.focused());
    ctx.data_mut(|d| d.insert_temp(start_id(), focused));
}

/// Tab (false) or Shift+Tab (true) pressed this frame, from the raw
/// events (a key handler may have taken the Tab already).
fn tab_pressed(ctx: &Context) -> Option<bool> {
    ctx.input(|i| {
        i.raw.events.iter().rev().find_map(|e| match e {
            egui::Event::Key {
                key: egui::Key::Tab,
                pressed: true,
                modifiers,
                ..
            } if !modifiers.any() => Some(false),
            egui::Event::Key {
                key: egui::Key::Tab,
                pressed: true,
                modifiers,
                ..
            } if modifiers.shift_only() => Some(true),
            _ => None,
        })
    })
}

/// The next widget in a layer's Tab order (`back`: the previous one),
/// going round to the first input box (the last) past the end. `inputs`
/// says which of `list` are input boxes. `from` is not in `list`: the
/// first (last) input box.
fn step(list: &[Id], inputs: &[bool], from: Option<Id>, back: bool) -> Option<Id> {
    let first_input = || {
        list.iter()
            .zip(inputs)
            .find(|(_, b)| **b)
            .map(|(id, _)| *id)
            .or(list.first().copied())
    };
    let last_input = || {
        list.iter()
            .zip(inputs)
            .rfind(|(_, b)| **b)
            .map(|(id, _)| *id)
            .or(list.last().copied())
    };
    let Some(i) = from.and_then(|f| list.iter().position(|id| *id == f)) else {
        return if back { last_input() } else { first_input() };
    };
    if back {
        if i == 0 {
            last_input()
        } else {
            list.get(i - 1).copied()
        }
    } else if i + 1 < list.len() {
        list.get(i + 1).copied()
    } else {
        first_input()
    }
}

/// After everything of the frame is drawn: with a Tab pressed, the place
/// the next frame puts the focus.
pub fn end(ctx: &Context) {
    let Some(back) = tab_pressed(ctx) else {
        return;
    };
    if egui::Popup::is_any_open(ctx) {
        return;
    }
    let start: Option<Id> = ctx.data(|d| d.get_temp(start_id())).flatten();
    let menu_bar: Option<Rect> = ctx.data(|d| d.get_temp(menu_bar_id()));
    let in_menu_bar = |layer: LayerId, r: Rect| {
        layer == LayerId::background() && menu_bar.is_some_and(|m| m.contains_rect(r))
    };
    let visible: Vec<LayerId> = ctx.memory(|m| {
        let areas = m.areas();
        m.layer_ids()
            .filter(|l| areas.is_visible(l))
            .collect::<Vec<_>>()
    });
    let found = ctx.viewport(|v| {
        let widgets = &v.this_pass.widgets;
        let focusable = |layer: LayerId| -> Vec<Id> {
            widgets
                .get_layer(layer)
                .filter(|w| w.enabled && w.sense.is_focusable())
                .filter(|w| !in_menu_bar(layer, w.rect))
                .map(|w| w.id)
                .collect()
        };
        match start {
            Some(s) => {
                let w = widgets.get(s).or_else(|| v.prev_pass.widgets.get(s))?;
                // The menu bar keeps its own keys.
                if in_menu_bar(w.layer_id, w.rect) {
                    return None;
                }
                Some((w.layer_id, focusable(w.layer_id)))
            }
            None => {
                // The window on top: a window or area over the main one.
                let top = [Order::Foreground, Order::Middle]
                    .into_iter()
                    .find_map(|o| {
                        visible
                            .iter()
                            .rev()
                            .filter(|l| l.order == o)
                            .map(|l| (*l, focusable(*l)))
                            .find(|(_, ids)| !ids.is_empty())
                    });
                Some(top.unwrap_or((LayerId::background(), Vec::new())))
            }
        }
    });
    let Some((layer, list)) = found else {
        return;
    };
    let to = if start.is_none() && layer == LayerId::background() {
        // No window: the drawing keeps Tab.
        None
    } else {
        let inputs: Vec<bool> = list
            .iter()
            .map(|id| egui::TextEdit::load_state(ctx, *id).is_some())
            .collect();
        match step(&list, &inputs, start, back) {
            Some(id) => Some(id),
            None => return,
        }
    };
    ctx.data_mut(|d| {
        d.insert_temp(
            pending_id(),
            Pending {
                from: start,
                to,
                layer,
            },
        )
    });
    ctx.request_repaint();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_goes_round_to_the_first_input_box() {
        let ids: Vec<Id> = (0..4).map(|i| Id::new(("w", i))).collect();
        // A button, two boxes, a button.
        let inputs = [false, true, true, false];
        assert_eq!(step(&ids, &inputs, Some(ids[0]), false), Some(ids[1]));
        assert_eq!(step(&ids, &inputs, Some(ids[3]), false), Some(ids[1]));
        assert_eq!(step(&ids, &inputs, Some(ids[1]), true), Some(ids[0]));
        assert_eq!(step(&ids, &inputs, Some(ids[0]), true), Some(ids[2]));
        assert_eq!(step(&ids, &inputs, None, false), Some(ids[1]));
        assert_eq!(step(&ids, &inputs, None, true), Some(ids[2]));
        // No box: the first and last controls.
        let none = [false; 4];
        assert_eq!(step(&ids, &none, Some(ids[3]), false), Some(ids[0]));
        assert_eq!(step(&ids, &none, Some(ids[0]), true), Some(ids[3]));
        assert_eq!(step(&[], &[], None, false), None);
    }
}
