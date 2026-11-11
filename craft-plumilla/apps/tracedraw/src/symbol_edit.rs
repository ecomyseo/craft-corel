//! Object > Symbol > Edit Symbol and Finish Editing Symbol: the definition
//! of the selected instance's symbol is edited in place, on its own (focus
//! mode fades the rest of the drawing); finishing makes the edited objects
//! the definition again, so every instance follows, and the instance that
//! was edited shows again. See docs/behavior/symbols.md.

use crate::app::App;
use tracedraw_core::document::{Shape, ShapeKind};
use tracedraw_core::{Affine, Command, ShapeId, Symbol};

/// A symbol being edited: which one, the instance it was opened from and
/// the editable copy of its definition on the page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolEdit {
    pub index: usize,
    pub instance: ShapeId,
    pub copy: ShapeId,
    pub transform: Affine,
}

impl App {
    /// The one selected object is a symbol instance, and no symbol is being
    /// edited.
    pub fn can_edit_symbol(&self) -> bool {
        self.symbol_editing.is_none()
            && matches!(self.selected_shapes().as_slice(),
                [s] if matches!(s.kind, ShapeKind::SymbolInstance { index } if index < self.doc().symbols.len()))
    }

    /// A copy of `s` with new ids for it and for everything inside it.
    pub(crate) fn with_new_ids(&mut self, mut s: Shape) -> Shape {
        match &mut s.kind {
            ShapeKind::Group { children } => {
                let kids = std::mem::take(children);
                *children = kids.into_iter().map(|c| self.with_new_ids(c)).collect();
            }
            ShapeKind::ClipFrame { frame, contents } => {
                let f = std::mem::replace(
                    frame.as_mut(),
                    Shape::new(ShapeId(0), ShapeKind::Group { children: vec![] }),
                );
                **frame = self.with_new_ids(f);
                let kids = std::mem::take(contents);
                *contents = kids.into_iter().map(|c| self.with_new_ids(c)).collect();
            }
            _ => {}
        }
        s.id = self.engine.new_shape_id();
        s
    }

    /// Edit Symbol: the definition put on the page where the instance is,
    /// the instance hidden meanwhile, and the copy in focus.
    pub fn edit_symbol(&mut self) {
        if !self.can_edit_symbol() {
            return;
        }
        let inst = self.selected_shapes()[0].clone();
        let ShapeKind::SymbolInstance { index } = inst.kind else {
            return;
        };
        let Ok((layer, _)) = self.doc().shape(inst.id) else {
            return;
        };
        let layer = layer.id;
        let def = self.doc().symbols[index].shape.clone();
        let mut copy = self.with_new_ids(def);
        copy.absorb(inst.transform);
        let copy_id = copy.id;
        let cmds = vec![
            Command::AddShape { layer, shape: copy },
            Command::SetVisible { shapes: vec![inst.id], visible: false },
        ];
        if let Err(e) = self.engine.run_batch("Edit Symbol", &cmds) {
            self.status = e.to_string();
            return;
        }
        self.symbol_editing = Some(SymbolEdit {
            index,
            instance: inst.id,
            copy: copy_id,
            transform: inst.transform,
        });
        self.focus_on(copy_id);
    }

    /// Finish Editing Symbol: the edited objects become the definition
    /// (every instance follows), the copy goes and the instance shows and
    /// is selected again; one undo step.
    pub fn finish_symbol_edit(&mut self) {
        let Some(e) = self.symbol_editing.take() else {
            return;
        };
        let mut cmds = Vec::new();
        if let Some(edited) = self.doc().find_shape(e.copy).cloned() {
            let mut def = edited;
            def.transform_by(e.transform.inverse());
            def.id = ShapeId(0);
            let name = self
                .doc()
                .symbols
                .get(e.index)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            cmds.push(Command::SetSymbol {
                index: e.index,
                symbol: Symbol { name, shape: def },
            });
            cmds.push(Command::DeleteShapes { shapes: vec![e.copy] });
        }
        if self.doc().find_shape(e.instance).is_some() {
            cmds.push(Command::SetVisible { shapes: vec![e.instance], visible: true });
        }
        self.exit_focus();
        if !cmds.is_empty() {
            if let Err(err) = self.engine.run_batch("Finish Editing Symbol", &cmds) {
                self.status = err.to_string();
                return;
            }
        }
        if self.doc().find_shape(e.instance).is_some() {
            self.select(vec![e.instance]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::menus::run_command;
    use tracedraw_core::Rect;

    /// Edit Symbol puts the definition where the instance is and in focus;
    /// a change made there reaches every instance once finished, in one
    /// undo step, and the instance edited shows again.
    #[test]
    fn a_symbol_is_edited_in_place_and_every_instance_follows() {
        crate::i18n::set_language("en");
        let ctx = crate::theme::ui_context();
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        let mut app = App::headless();
        let r = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 20.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.select(vec![r]);
        app.create_symbol_from_selection();
        let first = app.selection[0];
        app.insert_symbol_instance(0);
        let second = app.selection[0];
        assert!(!run_command(&mut app, &ctx, "menu.object.symbol_finish"), "nothing being edited");
        // The instance edited stands somewhere else than the definition.
        app.run(Command::TransformShapes {
            shapes: vec![first],
            transform: Affine::translate((40.0, 30.0)),
        });
        app.select(vec![first]);
        assert!(run_command(&mut app, &ctx, "menu.object.symbol_edit"));
        let e = app.symbol_editing.expect("editing");
        assert_eq!(app.focus_target(), Some(e.copy));
        assert!(!app.doc().find_shape(first).expect("instance").visible, "hidden while edited");
        let copy = app.doc().find_shape(e.copy).expect("copy").bounds();
        let inst = app.doc().find_shape(first).expect("instance").bounds();
        assert!((copy.x0 - inst.x0).abs() < 1e-6 && (copy.y0 - inst.y0).abs() < 1e-6, "{copy:?} {inst:?}");
        assert!(!run_command(&mut app, &ctx, "menu.object.symbol_edit"), "one edit at a time");
        // Make the definition twice as wide.
        let before = app.doc().symbols[0].shape.bounds();
        app.select(vec![e.copy]);
        // Twice as wide, about its own left edge, so it stays in place.
        let at = app.doc().find_shape(e.copy).expect("copy").bounds();
        app.run(Command::TransformShapes {
            shapes: vec![e.copy],
            transform: Affine::translate((at.x0, at.y0))
                * Affine::scale_non_uniform(2.0, 1.0)
                * Affine::translate((-at.x0, -at.y0)),
        });
        assert!(run_command(&mut app, &ctx, "menu.object.symbol_finish"));
        assert!(app.symbol_editing.is_none() && app.focus_target().is_none());
        assert!(app.doc().find_shape(e.copy).is_none(), "the copy goes");
        assert!(app.doc().find_shape(first).expect("instance").visible);
        assert_eq!(app.selection, vec![first]);
        let after = app.doc().symbols[0].shape.bounds();
        assert!((after.width() - 2.0 * before.width()).abs() < 1e-6, "{before:?} {after:?}");
        // Taken back through the instance's place: the definition stays
        // where it was, not where the instance stands.
        assert!((after.x0 - before.x0).abs() < 1e-6 && (after.y0 - before.y0).abs() < 1e-6, "{before:?} {after:?}");
        // Both instances still show the symbol, now the new definition.
        for id in [first, second] {
            let s = app.doc().find_shape(id).expect("instance");
            assert!(matches!(s.kind, ShapeKind::SymbolInstance { index: 0 }));
            assert_eq!(s.expand(&app.doc().symbols)[0].bounds().width(), after.width());
        }
        // One undo step back to the old definition.
        app.undo();
        assert!((app.doc().symbols[0].shape.bounds().width() - before.width()).abs() < 1e-6);
    }
}
