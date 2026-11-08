//! Dragging objects with the right mouse button (Pick tools): a blue
//! outline of the selection follows the pointer while the objects stay
//! put; on release a menu asks what to do there: Move Here, Copy Here,
//! Copy Fill Here, Copy Outline Here, Copy All Properties (onto the object
//! under the pointer), Place Inside (that object becomes the selection's
//! clip frame), or Cancel. Ctrl keeps the drag horizontal or vertical;
//! Esc cancels it.

use crate::app::App;
use egui::{Painter, Response, Ui};
use tracedraw_core::{
    geometry::{Affine, Point, Vec2},
    Command, ShapeId,
};

/// A right-button drag in progress (page space).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RightDrag {
    pub start: Point,
    pub current: Point,
}

impl RightDrag {
    /// How far the selection would go; `constrain` keeps the larger of
    /// the two directions only.
    pub fn offset(&self, constrain: bool) -> Vec2 {
        let d = self.current - self.start;
        if constrain {
            if d.x.abs() >= d.y.abs() {
                Vec2::new(d.x, 0.0)
            } else {
                Vec2::new(0.0, d.y)
            }
        } else {
            d
        }
    }
}

/// The menu shown where a right-button drag ended.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropMenu {
    /// Where the menu opens (screen).
    pub screen: egui::Pos2,
    pub offset: Vec2,
    /// The object under the pointer that is not being dragged.
    pub target: Option<ShapeId>,
}

/// What the drop menu does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropAction {
    Move,
    Copy,
    CopyFill,
    CopyOutline,
    CopyProperties,
    PlaceInside,
    Cancel,
}

impl DropAction {
    pub const ALL: [DropAction; 7] = [
        DropAction::Move,
        DropAction::Copy,
        DropAction::CopyFill,
        DropAction::CopyOutline,
        DropAction::CopyProperties,
        DropAction::PlaceInside,
        DropAction::Cancel,
    ];

    pub fn key(self) -> &'static str {
        match self {
            DropAction::Move => "drop.move_here",
            DropAction::Copy => "drop.copy_here",
            DropAction::CopyFill => "drop.copy_fill_here",
            DropAction::CopyOutline => "drop.copy_outline_here",
            DropAction::CopyProperties => "drop.copy_all_properties",
            DropAction::PlaceInside => "drop.place_inside",
            DropAction::Cancel => "drop.cancel",
        }
    }

    /// Whether the action needs an object under the pointer.
    pub fn needs_target(self) -> bool {
        matches!(
            self,
            DropAction::CopyFill
                | DropAction::CopyOutline
                | DropAction::CopyProperties
                | DropAction::PlaceInside
        )
    }
}

impl App {
    /// Right-button drags with the Pick tools; true when the input was
    /// used here.
    pub fn right_drag_input(&mut self, response: &Response, p: Point) -> bool {
        let constrain = response.ctx.input(|i| i.modifiers.command);
        if response.drag_started_by(egui::PointerButton::Secondary) {
            // A right drag from a selected object, also one covered by
            // another object there, moves the selection as it is (#1684).
            if !self.selection_under(p) {
                let Some(id) = self.hit_test(p) else {
                    return false;
                };
                if !self.selection.contains(&id) {
                    self.select(vec![id]);
                }
            }
            self.right_drag = Some(RightDrag {
                start: p,
                current: p,
            });
            self.drop_menu = None;
            return true;
        }
        let Some(mut drag) = self.right_drag else {
            return false;
        };
        if response.ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.right_drag = None;
            return true;
        }
        if response.dragged_by(egui::PointerButton::Secondary) {
            drag.current = p;
            self.right_drag = Some(drag);
            return true;
        }
        if response.drag_stopped() || !response.ctx.input(|i| i.pointer.secondary_down()) {
            drag.current = p;
            self.right_drag = None;
            let offset = drag.offset(constrain);
            if offset.hypot() * self.view.zoom as f64 >= 2.0 {
                let selection = self.selection.clone();
                self.drop_menu = Some(DropMenu {
                    screen: self.view.to_screen(p),
                    offset,
                    target: self.hit_test_where(p, |id| selection.contains(&id)),
                });
            }
            return true;
        }
        true
    }

    /// The selection's outline where a right-button drag would put it.
    pub fn draw_right_drag(&self, painter: &Painter) {
        let Some(drag) = self.right_drag else {
            return;
        };
        let constrain = painter.ctx().input(|i| i.modifiers.command);
        let t = Affine::translate(drag.offset(constrain));
        crate::canvas::draw_transform_outline(self, painter, t);
    }

    /// Carry out a drop menu choice.
    pub fn drop_action(&mut self, menu: DropMenu, action: DropAction) {
        let sources = self.selected_shapes();
        let Some(first) = sources.first().cloned() else {
            return;
        };
        match action {
            DropAction::Move => self.transform_selection(Affine::translate(menu.offset)),
            DropAction::Copy => self.duplicate_by(menu.offset),
            DropAction::CopyFill | DropAction::CopyOutline | DropAction::CopyProperties => {
                let Some(target) = menu.target else {
                    return;
                };
                let mut cmds = Vec::new();
                if action != DropAction::CopyOutline {
                    cmds.push(Command::SetFill {
                        shapes: vec![target],
                        fill: first.fill.clone(),
                    });
                }
                if action != DropAction::CopyFill {
                    cmds.push(Command::SetStroke {
                        shapes: vec![target],
                        stroke: first.stroke.clone(),
                    });
                }
                if action == DropAction::CopyProperties {
                    cmds.push(Command::SetOpacity {
                        shapes: vec![target],
                        opacity: first.opacity,
                    });
                }
                let label = match action {
                    DropAction::CopyFill => "Copy Fill",
                    DropAction::CopyOutline => "Copy Outline",
                    _ => "Copy Properties",
                };
                if let Err(e) = self.engine.run_batch(label, &cmds) {
                    self.status = e.to_string();
                }
            }
            DropAction::PlaceInside => {
                let Some(target) = menu.target else {
                    return;
                };
                let contents: Vec<ShapeId> = self
                    .selection
                    .iter()
                    .copied()
                    .filter(|id| *id != target)
                    .collect();
                if !contents.is_empty() {
                    self.place_inside(contents, target);
                    self.select(vec![target]);
                }
            }
            DropAction::Cancel => {}
        }
    }

    /// Copies of the selection moved by `offset`, selected.
    pub fn duplicate_by(&mut self, offset: Vec2) {
        let saved = self.duplicate_offset;
        self.duplicate_offset = offset;
        self.duplicate();
        self.duplicate_offset = saved;
    }
}

/// The drop menu, drawn like the context menu where the drag ended.
pub fn drop_menu_ui(app: &mut App, ui: &mut Ui) {
    let Some(menu) = app.drop_menu else {
        return;
    };
    let id = egui::Id::new("canvas_drop_menu");
    let ctx = ui.ctx().clone();
    let pos_id = id.with("pos");
    if ctx.data(|d| d.get_temp::<egui::Pos2>(pos_id)) != Some(menu.screen) {
        ctx.data_mut(|d| d.insert_temp(pos_id, menu.screen));
        egui::Popup::open_id(&ctx, id);
    }
    let mut chosen: Option<DropAction> = None;
    let shown = egui::Popup::new(
        id,
        ctx.clone(),
        egui::PopupAnchor::Position(menu.screen),
        ui.layer_id(),
    )
    .kind(egui::PopupKind::Menu)
    .layout(egui::Layout::top_down_justified(egui::Align::Min))
    .style(crate::ui::menus::menu_popup_style)
    .open_memory(None)
    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
    .show(|ui| {
        ui.set_min_width(190.0);
        crate::ui::menus::body(ui, |ui| {
            for action in DropAction::ALL {
                if matches!(action, DropAction::CopyFill | DropAction::Cancel) {
                    crate::ui::menus::sep(ui);
                }
                let enabled = !action.needs_target() || menu.target.is_some();
                if crate::ui::menus::item(ui, action.key(), "", enabled) {
                    chosen = Some(action);
                }
            }
        })
    });
    if let Some(action) = chosen {
        app.drop_action(menu, action);
    }
    if chosen.is_some() || shown.is_none() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        egui::Popup::close_id(&ctx, id);
        app.drop_menu = None;
        ctx.data_mut(|d| d.remove::<egui::Pos2>(pos_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{Color, Fill, Rect, ShapeKind, Stroke};

    fn two_boxes(app: &mut App) -> (ShapeId, ShapeId) {
        let a = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("a");
        let b = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(50.0, 0.0, 70.0, 20.0),
                radius: 0.0,
                corners: None,
            })
            .expect("b");
        app.run(Command::SetFill {
            shapes: vec![a],
            fill: Fill::Solid(Color::rgb8(255, 0, 0)),
        });
        app.run(Command::SetStroke {
            shapes: vec![a],
            stroke: Some(Stroke::new(Color::rgb8(0, 0, 255), 1.5)),
        });
        (a, b)
    }

    fn bounds(app: &App, id: ShapeId) -> tracedraw_core::Rect {
        app.doc()
            .shape(id)
            .map(|(_, s)| s.bounds())
            .unwrap_or_default()
    }

    #[test]
    fn a_ctrl_drag_keeps_to_one_direction() {
        let d = RightDrag {
            start: Point::new(0.0, 0.0),
            current: Point::new(10.0, 3.0),
        };
        assert_eq!(d.offset(false), Vec2::new(10.0, 3.0));
        assert_eq!(d.offset(true), Vec2::new(10.0, 0.0));
    }

    #[test]
    fn move_here_and_copy_here() {
        let mut app = App::headless();
        let (a, _) = two_boxes(&mut app);
        app.select(vec![a]);
        let menu = DropMenu {
            screen: egui::Pos2::ZERO,
            offset: Vec2::new(20.0, 5.0),
            target: None,
        };
        app.drop_action(menu, DropAction::Copy);
        // A copy, selected, where the drop was; the original stays.
        assert_eq!(app.selection.len(), 1);
        let copy = app.selection[0];
        assert_ne!(copy, a);
        assert!((bounds(&app, copy).x0 - 20.0).abs() < 1e-6);
        assert!((bounds(&app, a).x0).abs() < 1e-6);
        app.select(vec![a]);
        app.drop_action(menu, DropAction::Move);
        assert!((bounds(&app, a).x0 - 20.0).abs() < 1e-6);
        // Cancel changes nothing.
        let before = app.doc().clone();
        app.drop_action(menu, DropAction::Cancel);
        assert_eq!(*app.doc(), before);
    }

    #[test]
    fn fill_outline_and_properties_go_onto_the_target() {
        let mut app = App::headless();
        let (a, b) = two_boxes(&mut app);
        app.select(vec![a]);
        let menu = DropMenu {
            screen: egui::Pos2::ZERO,
            offset: Vec2::new(55.0, 5.0),
            target: Some(b),
        };
        app.drop_action(menu, DropAction::CopyFill);
        let shape = |app: &App, id| app.doc().shape(id).map(|(_, s)| s.clone()).ok();
        let (sa, sb) = (shape(&app, a).expect("a"), shape(&app, b).expect("b"));
        assert_eq!(sb.fill, sa.fill);
        assert_ne!(sb.stroke, sa.stroke);
        app.drop_action(menu, DropAction::CopyOutline);
        let sb = shape(&app, b).expect("b");
        assert_eq!(sb.stroke, sa.stroke);
        // The source did not move.
        assert!((bounds(&app, a).x0).abs() < 1e-6);
        // Without a target these do nothing.
        let none = DropMenu {
            target: None,
            ..menu
        };
        let before = app.doc().clone();
        app.drop_action(none, DropAction::CopyProperties);
        assert_eq!(*app.doc(), before);
    }

    #[test]
    fn place_inside_makes_the_target_a_clip_frame() {
        let mut app = App::headless();
        let (a, b) = two_boxes(&mut app);
        app.select(vec![a]);
        let menu = DropMenu {
            screen: egui::Pos2::ZERO,
            offset: Vec2::new(55.0, 5.0),
            target: Some(b),
        };
        app.drop_action(menu, DropAction::PlaceInside);
        let frame = app.doc().shape(b).map(|(_, s)| s.clone()).expect("frame");
        assert!(matches!(frame.kind, ShapeKind::ClipFrame { .. }));
        assert_eq!(app.selection, vec![b]);
    }
}
