//! Dragging a colour from a palette: dropped inside an object it becomes
//! the object's fill, dropped on the object's outline the outline's
//! colour, dropped on the document palette it joins it. The object need
//! not be selected and the selection stays as it was. No Color takes the
//! fill or the outline away. See docs/behavior/color-palettes.md.

use crate::app::App;
use egui::Pos2;
use tracedraw_core::{Color, Fill, ShapeId};

/// How near an object's outline a drop counts as on it, screen points.
const OUTLINE_PX: f64 = 4.0;

/// What a colour dropped at a point goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTarget {
    Fill(ShapeId),
    Outline(ShapeId),
    /// The end handle of a drop shadow (Drop Shadow tool): its colour.
    Shadow(ShapeId),
    DocumentPalette,
    /// The bar of the user's palette (by its index): it joins it.
    Palette(usize),
    Nothing,
}

impl App {
    /// The user's palette whose bar lies at `at` (screen), shown now and
    /// taking colours.
    pub fn palette_bar_at(&self, at: Pos2) -> Option<usize> {
        self.palette_drop_rects
            .iter()
            .rev()
            .find(|(_, r)| r.contains(at))
            .map(|(i, _)| *i)
            .filter(|i| {
                self.palette_takes_colors(*i)
                    && self.visible_palettes.iter().skip(1).any(|k| k == i)
            })
    }

    /// What a colour dropped at `at` (screen) would change.
    pub fn color_target(&self, at: Pos2) -> ColorTarget {
        if self.doc_palette_rect.is_some_and(|r| r.contains(at)) && self.has_document() {
            return ColorTarget::DocumentPalette;
        }
        if let Some(i) = self.palette_bar_at(at) {
            return ColorTarget::Palette(i);
        }
        if !self.canvas_rect.contains(at) || !self.has_document() || self.show_welcome {
            return ColorTarget::Nothing;
        }
        let p = self.view.to_page(at);
        if self.shadow_handle_at(p) == Some(crate::shadow_tool::ShadowHandle::End) {
            if let Some((id, ..)) = self.selected_shadow() {
                return ColorTarget::Shadow(id);
            }
        }
        // Inside a closed object counts even when it has no fill yet.
        let Some(id) = self.hit_test_inside(p) else {
            return ColorTarget::Nothing;
        };
        let Some(shape) = self.doc().find_shape(id) else {
            return ColorTarget::Nothing;
        };
        let tol = OUTLINE_PX / f64::from(self.view.zoom.max(1e-6));
        if crate::canvas::distance_to_path(&shape.page_path(), p) <= tol {
            ColorTarget::Outline(id)
        } else {
            ColorTarget::Fill(id)
        }
    }

    /// Drop `color` (`None`: No Color) at `at` (screen); false when
    /// nothing took it.
    pub fn drop_color(&mut self, color: Option<Color>, at: Pos2) -> bool {
        if let Some(c) = color {
            if self.drop_on_fill_line(c, self.view.to_page(at)) {
                return true;
            }
        }
        match self.color_target(at) {
            ColorTarget::Palette(i) => match color {
                Some(c) => self.add_colors_to_palette(i, &[c]),
                None => false,
            },
            ColorTarget::DocumentPalette => match color {
                Some(c) => {
                    self.add_to_document_palette(c);
                    true
                }
                None => false,
            },
            ColorTarget::Fill(id) => {
                let saved = std::mem::replace(&mut self.selection, vec![id]);
                self.apply_fill(color.map(Fill::Solid).unwrap_or(Fill::None));
                self.selection = saved;
                true
            }
            ColorTarget::Outline(id) => {
                let saved = std::mem::replace(&mut self.selection, vec![id]);
                self.apply_outline_color(color);
                self.selection = saved;
                true
            }
            ColorTarget::Shadow(id) => match color {
                Some(c) => {
                    self.set_shadow_color(id, c);
                    true
                }
                None => false,
            },
            ColorTarget::Nothing => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Point, Rect};
    use tracedraw_core::ShapeKind;

    fn app_with_rect() -> (App, ShapeId) {
        let mut app = App::headless();
        app.canvas_rect = egui::Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0));
        let page = app.page_rect();
        app.view.fit(page, app.canvas_rect);
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.select(vec![id]);
        app.apply_fill(Fill::Solid(Color::rgb8(0, 0, 255)));
        app.select(Vec::new());
        (app, id)
    }

    #[test]
    fn colours_dropped_inside_fill_and_on_the_outline_colour_the_outline() {
        let (mut app, id) = app_with_rect();
        let red = Color::rgb8(255, 0, 0);
        let inside = app.view.to_screen(Point::new(35.0, 25.0));
        assert_eq!(app.color_target(inside), ColorTarget::Fill(id));
        assert!(app.drop_color(Some(red), inside));
        let s = app.doc().find_shape(id).cloned().expect("shape");
        assert_eq!(s.fill, Fill::Solid(red));
        // The selection stays as it was (nothing).
        assert!(app.selection.is_empty());
        // On the edge: the outline.
        let green = Color::rgb8(0, 160, 0);
        let edge = app.view.to_screen(Point::new(35.0, 10.0));
        assert_eq!(app.color_target(edge), ColorTarget::Outline(id));
        assert!(app.drop_color(Some(green), edge));
        let s = app.doc().find_shape(id).cloned().expect("shape");
        assert_eq!(s.stroke.map(|st| st.color), Some(green));
        assert_eq!(s.fill, Fill::Solid(red));
        // No Color inside takes the fill away; one undo brings it back.
        assert!(app.drop_color(None, inside));
        assert_eq!(
            app.doc().find_shape(id).map(|s| s.fill.clone()),
            Some(Fill::None)
        );
        app.undo();
        assert_eq!(
            app.doc().find_shape(id).map(|s| s.fill.clone()),
            Some(Fill::Solid(red))
        );
        // Empty page and outside the drawing: nothing.
        let empty = app.view.to_screen(Point::new(150.0, 200.0));
        assert_eq!(app.color_target(empty), ColorTarget::Nothing);
        assert!(!app.drop_color(Some(red), empty));
        assert!(!app.drop_color(Some(red), Pos2::new(5000.0, 5000.0)));
    }

    #[test]
    fn an_object_without_a_fill_takes_one_inside() {
        let (mut app, _filled) = app_with_rect();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(100.0, 100.0, 150.0, 140.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        let s = app.doc().find_shape(id).cloned().expect("shape");
        assert_eq!(s.fill, Fill::None);
        let inside = app.view.to_screen(Point::new(125.0, 120.0));
        assert_eq!(app.color_target(inside), ColorTarget::Fill(id));
        let red = Color::rgb8(255, 0, 0);
        assert!(app.drop_color(Some(red), inside));
        assert_eq!(
            app.doc().find_shape(id).map(|s| s.fill.clone()),
            Some(Fill::Solid(red))
        );
    }

    #[test]
    fn colours_dropped_on_the_document_palette_join_it() {
        let (mut app, _id) = app_with_rect();
        let row = egui::Rect::from_min_size(Pos2::new(0.0, 700.0), egui::vec2(800.0, 24.0));
        app.doc_palette_rect = Some(row);
        let at = row.center();
        assert_eq!(app.color_target(at), ColorTarget::DocumentPalette);
        let teal = Color::rgb8(0, 128, 128);
        assert!(app.drop_color(Some(teal), at));
        assert!(app.doc().palette.contains(&teal));
        // No Color has nothing to add.
        assert!(!app.drop_color(None, at));
    }
}
