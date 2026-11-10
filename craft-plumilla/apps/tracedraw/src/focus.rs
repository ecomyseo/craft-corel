//! Focus mode: one group or object of a complex drawing edited on its
//! own. Everything else fades (Options > Craft Plumilla > Edit > Visibility of
//! inactive objects), clicks pick only inside the object in focus, and a
//! bar at the top of the drawing window shows the way in: the drawing,
//! then each group, each a step back out. See docs/behavior/focus-mode.md.

use crate::app::App;
use tracedraw_core::document::{Shape, ShapeKind};
use tracedraw_core::ShapeId;

impl App {
    /// The object in focus, when focus mode is on and it still exists.
    pub fn focus_target(&self) -> Option<ShapeId> {
        let id = *self.focus.last()?;
        self.doc().find_shape(id).map(|_| id)
    }

    /// The way in: the groups around the object in focus, then itself.
    pub fn focus_path(&self) -> Vec<ShapeId> {
        match self.focus_target() {
            Some(id) => {
                let mut path = self.doc().ancestors(id);
                path.push(id);
                path
            }
            None => Vec::new(),
        }
    }

    /// Bring into Focus: the one selected object (a group, a clip frame or
    /// any object) is edited on its own.
    pub fn bring_into_focus(&mut self) {
        let [id] = self.selection.as_slice() else {
            return;
        };
        let id = *id;
        self.focus = vec![id];
        // Its parts are what is picked now.
        self.select(Vec::new());
        self.raster.borrow_mut().invalidate();
    }

    /// Bring `id` into focus.
    pub fn focus_on(&mut self, id: ShapeId) {
        if self.doc().find_shape(id).is_some() {
            self.focus = vec![id];
            self.select(Vec::new());
            self.raster.borrow_mut().invalidate();
        }
    }

    /// One level out: the group holding the object in focus, or out of
    /// focus mode from the outermost one.
    pub fn focus_up(&mut self) {
        let Some(id) = self.focus_target() else {
            self.exit_focus();
            return;
        };
        let was = id;
        match self.doc().ancestors(id).last().copied() {
            Some(parent) => {
                self.focus = vec![parent];
                self.select(vec![was]);
            }
            None => {
                self.exit_focus();
                self.select(vec![was]);
            }
        }
        self.raster.borrow_mut().invalidate();
    }

    /// Leave focus mode.
    pub fn exit_focus(&mut self) {
        if !self.focus.is_empty() {
            self.focus.clear();
            self.raster.borrow_mut().invalidate();
        }
    }

    /// The object in focus in page space, for picking inside it.
    pub fn focus_shape_on_page(&self) -> Option<Shape> {
        self.doc().shape_on_page(self.focus_target()?)
    }

    /// The visibility of the faded objects, 0 to 1.
    pub fn focus_visibility(&self) -> f32 {
        (self.settings.focus_visibility as f32 / 100.0).clamp(0.0, 1.0)
    }

    /// An object can be brought into focus by a double click: a group or
    /// a clip frame.
    pub fn focus_on_double_click(&self, id: ShapeId) -> bool {
        self.settings.focus_double_click
            && matches!(
                self.doc().find_shape(id).map(|s| &s.kind),
                Some(ShapeKind::Group { .. } | ShapeKind::ClipFrame { .. })
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{Point, Rect};

    fn rect_at(app: &mut App, x: f64) -> ShapeId {
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(x, 10.0, x + 10.0, 20.0),
            radius: 0.0,
            corners: None,
        })
        .expect("rect")
    }

    #[test]
    fn focus_goes_in_picks_inside_and_comes_back_out() {
        let mut app = App::headless();
        let a = rect_at(&mut app, 10.0);
        let b = rect_at(&mut app, 30.0);
        let outside = rect_at(&mut app, 60.0);
        app.select(vec![a, b]);
        app.group_selection();
        let group = app.selection[0];
        app.bring_into_focus();
        assert_eq!(app.focus_target(), Some(group));
        assert_eq!(app.focus_path(), vec![group]);
        // A click picks a part of the group, and nothing outside it.
        assert_eq!(app.hit_test(Point::new(15.0, 15.0)), Some(a));
        assert_eq!(app.hit_test(Point::new(35.0, 15.0)), Some(b));
        assert_eq!(app.hit_test(Point::new(65.0, 15.0)), None);
        let _ = outside;
        // One level out leaves focus mode with the group selected.
        app.focus_up();
        assert_eq!(app.focus_target(), None);
        assert_eq!(app.selection, vec![group]);
        assert_eq!(app.hit_test(Point::new(15.0, 15.0)), Some(group));
        // Double clicks bring groups into focus when the option is on.
        assert!(app.focus_on_double_click(group));
        app.settings.focus_double_click = false;
        assert!(!app.focus_on_double_click(group));
        // A deleted object ends focus.
        app.focus_on(group);
        app.select(vec![group]);
        app.exit_focus();
        assert!(app.focus.is_empty());
    }

    #[test]
    fn the_rest_of_the_drawing_fades() {
        let mut app = App::headless();
        let a = rect_at(&mut app, 10.0);
        let b = rect_at(&mut app, 30.0);
        for id in [a, b] {
            app.select(vec![id]);
            app.apply_fill(tracedraw_core::Fill::Solid(tracedraw_core::Color::rgb8(
                0, 0, 0,
            )));
        }
        let doc = app.doc().clone();
        let page = doc.pages[0].id;
        let opts = |focus| tracedraw_render::RenderOptions {
            width: 60,
            height: 40,
            view: tracedraw_render::ViewTransform {
                zoom: 1.0,
                origin_x: 0.0,
                origin_y: 40.0,
            },
            focus,
            ..Default::default()
        };
        let img =
            tracedraw_render::render_page(&doc, page, &opts(Some((vec![a], 0.2)))).expect("render");
        // The object in focus is drawn as it is; the other one faded.
        assert_eq!(img.pixel(15, 25).expect("px").alpha(), 255);
        let faded = img.pixel(35, 25).expect("px").alpha();
        assert!((40..=60).contains(&faded), "{faded}");
    }
}
