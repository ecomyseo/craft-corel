//! The object context menu's Frame Type: an object becomes an empty clip
//! frame (the shape as it is, ready to hold contents), an empty clip frame
//! becomes its object again (No Frame), and an object becomes an empty
//! paragraph text frame of its size in its place.

use crate::app::App;
use tracedraw_core::{
    document::{ParagraphStyle, Shape, ShapeKind, TextSpan},
    geometry::{Point, Size},
    Command, ShapeId,
};

impl App {
    /// The selected objects on a layer that can become clip frames (not
    /// clip frames already).
    pub fn framable(&self) -> Vec<ShapeId> {
        let doc = self.doc();
        self.selection
            .iter()
            .copied()
            .filter(|id| {
                doc.shape(*id)
                    .is_ok_and(|(_, s)| !matches!(s.kind, ShapeKind::ClipFrame { .. }))
            })
            .collect()
    }

    /// The selected clip frames that hold nothing.
    pub fn empty_clip_frames(&self) -> Vec<ShapeId> {
        let doc = self.doc();
        self.selection
            .iter()
            .copied()
            .filter(|id| {
                doc.shape(*id).is_ok_and(|(_, s)| {
                    matches!(&s.kind, ShapeKind::ClipFrame { contents, .. } if contents.is_empty())
                })
            })
            .collect()
    }

    /// Frame Type > Create Empty Clip Frame: each object becomes a clip
    /// frame holding nothing yet, one undo step.
    pub fn objects_to_clip_frames(&mut self) {
        let cmds: Vec<Command> = self
            .framable()
            .into_iter()
            .map(|frame| Command::PlaceInside {
                contents: Vec::new(),
                frame,
            })
            .collect();
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Create Empty Clip Frame", &cmds) {
            self.status = e.to_string();
        }
    }

    /// Frame Type > No Frame: each empty clip frame becomes the object it
    /// was made from again, one undo step.
    pub fn frames_to_objects(&mut self) {
        let cmds: Vec<Command> = self
            .empty_clip_frames()
            .into_iter()
            .map(|clip| Command::ExtractContents { clip })
            .collect();
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("No Frame", &cmds) {
            self.status = e.to_string();
        }
    }

    /// Frame Type > Create Empty Text Frame: the selected object gives way
    /// to an empty paragraph text frame of its size in its place, in the
    /// text settings new text takes; one undo step. The frame stays
    /// selected, ready for the Text tool.
    pub fn object_to_text_frame(&mut self) {
        let [id] = self.selection[..] else {
            return;
        };
        let Some(bounds) = self.doc().shape_on_page(id).map(|s| s.bounds()) else {
            return;
        };
        if bounds.width() < 1e-6 || bounds.height() < 1e-6 {
            return;
        }
        let Some(layer) = self.active_layer() else {
            return;
        };
        let span = TextSpan {
            bold: self.text_bold,
            italic: self.text_italic,
            ..TextSpan::new("", self.text_font.clone(), self.text_size_pt)
        };
        let para = ParagraphStyle {
            hyphenate: self.text_hyphenation,
            ..ParagraphStyle::default()
        };
        let frame_id = self.engine.new_shape_id();
        let mut shape = Shape::new(
            frame_id,
            ShapeKind::Text {
                spans: vec![span],
                origin: Point::new(bounds.x0, bounds.y0),
                frame: Some(Size::new(bounds.width(), bounds.height())),
                align: self.text_align,
                para,
                on_path: None,
            },
        );
        self.doc().defaults.apply_to(&mut shape);
        let cmds = vec![
            Command::DeleteShapes { shapes: vec![id] },
            Command::AddShape { layer, shape },
        ];
        if let Err(e) = self.engine.run_batch("Create Empty Text Frame", &cmds) {
            self.status = e.to_string();
            return;
        }
        self.select(vec![frame_id]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Affine;

    fn turned_rect(app: &mut App) -> ShapeId {
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(10.0, 10.0, 50.0, 30.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.run(Command::TransformShapes {
            shapes: vec![id],
            transform: Affine::rotate_about(0.5, Point::new(30.0, 20.0)),
        });
        app.select(vec![id]);
        id
    }

    /// An object becomes an empty clip frame where it was (its outline in
    /// place, turned or not) and No Frame gives it back.
    #[test]
    fn empty_clip_frames_come_and_go_in_place() {
        let mut app = App::headless();
        let id = turned_rect(&mut app);
        let before = app.doc().shape_on_page(id).expect("rect").bounds();
        assert_eq!(app.framable(), vec![id]);
        app.objects_to_clip_frames();
        let s = app.doc().shape_on_page(id).expect("clip");
        assert!(matches!(&s.kind, ShapeKind::ClipFrame { contents, .. } if contents.is_empty()));
        let b = s.bounds();
        for (x, y) in [
            (b.x0, before.x0),
            (b.y0, before.y0),
            (b.x1, before.x1),
            (b.y1, before.y1),
        ] {
            assert!((x - y).abs() < 1e-6, "{b:?} != {before:?}");
        }
        assert!(app.framable().is_empty());
        assert_eq!(app.empty_clip_frames(), vec![id]);
        app.frames_to_objects();
        let s = app.doc().shape_on_page(id).expect("rect again");
        assert!(matches!(s.kind, ShapeKind::Rect { .. }));
        assert!(app.empty_clip_frames().is_empty());
    }

    #[test]
    fn an_object_gives_way_to_an_empty_text_frame_of_its_size() {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(10.0, 10.0, 50.0, 30.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        app.object_to_text_frame();
        assert!(app.doc().find_shape(id).is_none());
        let frame = app.selection[0];
        let s = app.doc().find_shape(frame).cloned().expect("frame");
        assert!(matches!(
            s.kind,
            ShapeKind::Text { frame: Some(f), .. } if (f.width - 40.0).abs() < 1e-6 && (f.height - 20.0).abs() < 1e-6
        ));
        // The empty frame stays until the user removes it.
        assert!(app.text_edit.is_none());
        app.undo();
        assert!(app.doc().find_shape(id).is_some());
    }
}
