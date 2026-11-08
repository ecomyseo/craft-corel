//! Object styles: named sets of fill, outline and transparency that
//! objects stay linked to (Object Styles panel, the context menu's Object
//! Styles). The links and their updates are the engine's
//! (`Command::SetObjectStyles`, `Command::LinkObjectStyle`).

use crate::app::App;
use crate::i18n::{tr, trf};
use std::collections::HashMap;
use tracedraw_core::{document::ShapeKind, Command, Document, ObjectStyle, Shape};

/// `styles` with a free id for each style that has none, as the engine
/// gives them.
pub fn with_ids(mut styles: Vec<ObjectStyle>) -> Vec<ObjectStyle> {
    let mut next = styles.iter().map(|s| s.id).max().unwrap_or(0);
    for st in styles.iter_mut().filter(|s| s.id == 0) {
        next += 1;
        st.id = next;
    }
    styles
}

/// How many objects each style has linked, groups and clip frames
/// included.
pub fn linked_counts(doc: &Document) -> HashMap<u32, usize> {
    fn walk(s: &Shape, out: &mut HashMap<u32, usize>) {
        if let Some(id) = s.style {
            *out.entry(id).or_default() += 1;
        }
        match &s.kind {
            ShapeKind::Group { children } => children.iter().for_each(|c| walk(c, out)),
            ShapeKind::ClipFrame { frame, contents } => {
                walk(frame, out);
                contents.iter().for_each(|c| walk(c, out));
            }
            _ => {}
        }
    }
    let mut out = HashMap::new();
    for s in doc.all_layers().flat_map(|l| &l.shapes) {
        walk(s, &mut out);
    }
    out
}

/// Unlink `shape` (and what it holds) from styles that are not the same
/// in `here` as they were in `from`, the drawing it was copied from; the
/// properties stay.
pub fn keep_links_to(shape: &mut Shape, from: &[ObjectStyle], here: &[ObjectStyle]) {
    if let Some(id) = shape.style {
        let same = match (
            from.iter().find(|s| s.id == id),
            here.iter().find(|s| s.id == id),
        ) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        };
        if !same {
            shape.style = None;
        }
    }
    match &mut shape.kind {
        ShapeKind::Group { children } => children
            .iter_mut()
            .for_each(|c| keep_links_to(c, from, here)),
        ShapeKind::ClipFrame { frame, contents } => {
            keep_links_to(frame, from, here);
            contents
                .iter_mut()
                .for_each(|c| keep_links_to(c, from, here));
        }
        _ => {}
    }
}

impl App {
    /// A new style from the first selected object, linked to the whole
    /// selection, as one undo step.
    pub fn new_style_from_selection(&mut self) {
        let Some(first) = self.selected_shapes().first().cloned() else {
            return;
        };
        let mut styles = with_ids(self.doc().object_styles.clone());
        let mut style = ObjectStyle::from_shape(
            trf("docker.style_n", &[("n", &(styles.len() + 1).to_string())]),
            &first,
        );
        style.id = styles.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        let id = style.id;
        styles.push(style);
        let cmds = vec![
            Command::SetObjectStyles { styles },
            Command::LinkObjectStyle {
                shapes: self.selection.clone(),
                style: Some(id),
            },
        ];
        if let Err(e) = self.engine.run_batch("New Style", &cmds) {
            self.status = e.to_string();
        } else {
            self.status = tr("status.style_created");
        }
    }

    /// Link the selection to the style at `index` of the list and give it
    /// the style's parts.
    pub fn apply_style(&mut self, index: usize) {
        if self.selection.is_empty() {
            return;
        }
        let styles = self.doc().object_styles.clone();
        let Some(style) = styles.get(index) else {
            return;
        };
        let mut cmds = Vec::new();
        let id = if style.id == 0 {
            // A style from an older file: number the styles first.
            let numbered = with_ids(styles.clone());
            let id = numbered[index].id;
            cmds.push(Command::SetObjectStyles { styles: numbered });
            id
        } else {
            style.id
        };
        cmds.push(Command::LinkObjectStyle {
            shapes: self.selection.clone(),
            style: Some(id),
        });
        if let Err(e) = self.engine.run_batch("Apply Style", &cmds) {
            self.status = e.to_string();
        }
    }

    /// The selected objects that are linked to a style.
    pub fn styled_selection(&self) -> Vec<tracedraw_core::ShapeId> {
        self.selected_shapes()
            .iter()
            .filter(|s| s.style.is_some())
            .map(|s| s.id)
            .collect()
    }

    /// Break the selection's links to styles; its properties stay.
    pub fn break_style_links(&mut self) {
        let shapes = self.styled_selection();
        if !shapes.is_empty() {
            self.run(Command::LinkObjectStyle {
                shapes,
                style: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{geometry::Rect, Color, Fill};

    #[test]
    fn a_style_from_the_selection_links_it_and_follows_edits() {
        let mut app = App::headless();
        let rect = |app: &mut App, x: f64| {
            app.new_shape(ShapeKind::Rect {
                rect: Rect::new(x, 0.0, x + 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape")
        };
        let a = rect(&mut app, 0.0);
        let b = rect(&mut app, 20.0);
        let red = Fill::Solid(Color::rgb8(255, 0, 0));
        app.select(vec![a]);
        app.apply_fill(red.clone());
        app.new_style_from_selection();
        let styles = app.doc().object_styles.clone();
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].fill, red);
        assert_eq!(linked_counts(app.doc()).get(&styles[0].id), Some(&1));
        // Applied to the other one, which takes the red fill.
        app.select(vec![b]);
        app.apply_style(0);
        assert_eq!(
            app.doc().shape(b).map(|(_, s)| s.fill.clone()).ok(),
            Some(red)
        );
        assert_eq!(linked_counts(app.doc()).get(&styles[0].id), Some(&2));
        // A style edit reaches both.
        let mut edited = app.doc().object_styles.clone();
        edited[0].fill = Fill::Solid(Color::rgb8(0, 0, 255));
        app.run(Command::SetObjectStyles { styles: edited });
        for id in [a, b] {
            assert_eq!(
                app.doc().shape(id).map(|(_, s)| s.fill.clone()).ok(),
                Some(Fill::Solid(Color::rgb8(0, 0, 255)))
            );
        }
        // Breaking the link of one leaves one linked.
        app.select(vec![a]);
        assert_eq!(app.styled_selection(), vec![a]);
        app.break_style_links();
        assert!(app.styled_selection().is_empty());
        assert_eq!(linked_counts(app.doc()).values().sum::<usize>(), 1);
        // Styles of older files get numbered when applied.
        let mut doc = app.doc().clone();
        let plain = ObjectStyle::from_shape("Plain", doc.shape(a).map(|(_, s)| s).expect("a"));
        doc.object_styles[0].id = 0;
        doc.object_styles.push(plain);
        app.open_document(doc, None);
        app.select(vec![a]);
        app.apply_style(1);
        let ids: Vec<u32> = app.doc().object_styles.iter().map(|s| s.id).collect();
        assert_eq!(ids, vec![1, 2]);
        assert_eq!(app.doc().shape(a).map(|(_, s)| s.style).ok(), Some(Some(2)));
        // Copied into a drawing without that style, the link goes; within
        // the drawing it stays.
        app.copy();
        app.paste();
        let pasted = app.selection[0];
        assert_eq!(
            app.doc().shape(pasted).map(|(_, s)| s.style).ok(),
            Some(Some(2))
        );
        app.new_document();
        app.paste();
        let pasted = app.selection[0];
        assert_eq!(
            app.doc().shape(pasted).map(|(_, s)| s.style).ok(),
            Some(None)
        );
    }
}
