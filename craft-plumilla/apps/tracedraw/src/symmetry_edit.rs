//! Editing a symmetry (Object > Symmetry > Edit Symmetry): the object
//! being edited keeps its bar and mirror lines whatever is selected; a
//! click or a marquee picks the original objects inside it (the copies
//! are not picked); the mirror lines are dragged with the Pick tool to
//! move the centre, and a click on a line shows round handles at the
//! ends of the lines that turn them. With Show full preview off the
//! copies show as outlines only. See docs/behavior/symmetry.md.

use crate::app::{App, Drag};
use tracedraw_core::geometry::{Point, Vec2};
use tracedraw_core::live::Effect;
use tracedraw_core::{Command, Shape, ShapeId};

/// How far from a mirror line, in screen pixels, a press takes it.
const LINE_REACH: f32 = 5.0;
/// How far from the centre, in screen pixels, the turning handles sit.
pub const HANDLE_RADIUS: f32 = 90.0;
/// The undo label of the changes made while editing.
const LABEL: &str = "Edit Symmetry";

impl App {
    /// The symmetry being edited: the object, its centre, the first
    /// line's angle and the number of lines.
    pub fn edited_symmetry(&self) -> Option<(ShapeId, Point, f64, u8)> {
        let id = self.symmetry_edit?;
        let s = self.doc().find_shape(id)?;
        s.effects.iter().find_map(|e| match e {
            Effect::Symmetry {
                center,
                angle,
                lines,
            } => Some((id, *center, *angle, *lines)),
            _ => None,
        })
    }

    /// Give the symmetry of `id` a new centre, angle and number of lines.
    /// Changes made one after another while editing are one undo step.
    pub fn set_symmetry_of(&mut self, id: ShapeId, center: Point, angle: f64, lines: u8) {
        let Some(s) = self.doc().find_shape(id) else {
            return;
        };
        let effects: Vec<Effect> = s
            .effects
            .iter()
            .map(|e| match e {
                Effect::Symmetry { .. } => Effect::Symmetry {
                    center,
                    angle: angle.rem_euclid(360.0),
                    lines: lines.clamp(1, 12),
                },
                other => other.clone(),
            })
            .collect();
        let cmd = Command::SetEffects { shape: id, effects };
        let r = if self.engine.undo_label() == Some(LABEL) && self.symmetry_step == Some(id) {
            self.engine.amend(&cmd)
        } else {
            self.engine.run_with_label(&cmd, LABEL)
        };
        match r {
            Ok(()) => self.symmetry_step = Some(id),
            Err(e) => self.status = e.to_string(),
        }
    }

    /// The direction of mirror line `k` of `lines` for a first line at
    /// `angle` degrees.
    fn line_direction(angle: f64, k: usize, lines: u8) -> Vec2 {
        let a = (angle + 180.0 * k as f64 / lines.max(1) as f64).to_radians();
        Vec2::new(a.cos(), a.sin())
    }

    /// The turning handles while they show: at both ends of each mirror
    /// line, [`HANDLE_RADIUS`] pixels from the centre (page space).
    pub fn symmetry_handles(&self) -> Vec<Point> {
        let Some((_, center, angle, lines)) = self.edited_symmetry() else {
            return Vec::new();
        };
        if !self.symmetry_turning {
            return Vec::new();
        }
        let r = HANDLE_RADIUS as f64 / self.view.zoom.max(1e-6) as f64;
        (0..lines as usize)
            .flat_map(|k| {
                let d = Self::line_direction(angle, k, lines);
                [center + d * r, center - d * r]
            })
            .collect()
    }

    /// Whether a press at `p` (page space) takes the mirror lines: on a
    /// turning handle (to turn them), else on the centre or on a line off
    /// the objects (to move them). `None` when nothing of the symmetry is
    /// there.
    fn symmetry_part_at(&self, p: Point) -> Option<bool> {
        let (_, center, angle, lines) = self.edited_symmetry()?;
        if !self.settings.symmetry_show_lines {
            return None;
        }
        let tol = LINE_REACH as f64 / self.view.zoom.max(1e-6) as f64;
        if self
            .symmetry_handles()
            .iter()
            .any(|h| (*h - p).hypot() <= tol * 1.6)
        {
            return Some(true);
        }
        if (p - center).hypot() <= tol * 1.6 {
            return Some(false);
        }
        let on_line = (0..lines as usize).any(|k| {
            let d = Self::line_direction(angle, k, lines);
            let v = p - center;
            (v.x * d.y - v.y * d.x).abs() <= tol
        });
        // The lines cross the whole window: where one runs over an
        // object, the object is picked and dragged, not the line.
        (on_line && self.hit_test(p).is_none()).then_some(false)
    }

    /// A Pick tool press while a symmetry is edited: a drag on a line or
    /// the centre moves the lines, one on a turning handle turns them.
    /// True when the press was taken.
    pub fn symmetry_press(&mut self, p: Point) -> bool {
        let Some(turn) = self.symmetry_part_at(p) else {
            return false;
        };
        let Some((id, center, angle, _)) = self.edited_symmetry() else {
            return false;
        };
        self.drag = Drag::SymmetryLines {
            id,
            turn,
            from: p,
            start_center: center,
            start_angle: angle,
        };
        true
    }

    /// A Pick tool click while a symmetry is edited: on a line, the
    /// turning handles show (or hide). True when the click was taken.
    pub fn symmetry_click(&mut self, p: Point) -> bool {
        match self.symmetry_part_at(p) {
            Some(false) => {
                self.symmetry_turning = !self.symmetry_turning;
                true
            }
            Some(true) => true,
            None => false,
        }
    }

    /// The mirror lines dragged to `p`: moved by the pointer's way, or
    /// turned about the centre by the angle the pointer swept.
    pub fn symmetry_drag_to(&mut self, p: Point, snap_angle: bool) {
        let Drag::SymmetryLines {
            id,
            turn,
            from,
            start_center,
            start_angle,
        } = self.drag.clone()
        else {
            return;
        };
        let Some((_, _, _, lines)) = self.edited_symmetry() else {
            return;
        };
        if turn {
            let a0 = (from - start_center).atan2();
            let a1 = (p - start_center).atan2();
            let mut a = start_angle + (a1 - a0).to_degrees();
            if snap_angle {
                let step = self.settings.constrain_angle.max(1.0);
                a = (a / step).round() * step;
            }
            self.set_symmetry_of(id, start_center, a, lines);
        } else {
            let to = self.snap_point_excluding(start_center + (p - from), &[]);
            self.set_symmetry_of(id, to, start_angle, lines);
        }
    }

    /// `s` without its symmetry when it is the object whose symmetry is
    /// edited: while editing, its copies are not picked, and its box is
    /// its own, as in the reference program.
    pub fn without_edited_symmetry(&self, s: &Shape) -> Option<Shape> {
        if self.symmetry_edit != Some(s.id)
            || !s
                .effects
                .iter()
                .any(|e| matches!(e, Effect::Symmetry { .. }))
        {
            return None;
        }
        let mut own = s.clone();
        own.effects
            .retain(|e| !matches!(e, Effect::Symmetry { .. }));
        Some(own)
    }

    /// The members of the symmetry being edited, in page space, when it
    /// is a group: what a click or a marquee picks while editing.
    pub fn symmetry_members_on_page(&self) -> Option<(ShapeId, Vec<Shape>)> {
        let id = self.symmetry_edit?;
        let g = self.doc().shape_on_page(id)?;
        match &g.kind {
            tracedraw_core::ShapeKind::Group { children } => Some((
                id,
                children
                    .iter()
                    .filter(|c| c.visible && !c.locked)
                    .map(|c| {
                        let mut c = c.clone();
                        c.transform_by(g.transform);
                        c
                    })
                    .collect(),
            )),
            _ => None,
        }
    }

    /// The document as drawn while a symmetry is edited with Show full
    /// preview off: the object without its symmetry (its copies are then
    /// outlined by the canvas). `None` otherwise.
    pub fn symmetry_without_copies(&self) -> Option<std::rc::Rc<tracedraw_core::Document>> {
        if self.settings.symmetry_full_preview {
            return None;
        }
        let id = self.symmetry_edit?;
        let rev = self.engine.revision();
        // Made once per change of the drawing, not every frame.
        if let Some((r, i, doc)) = self.symmetry_doc.borrow().as_ref() {
            if *r == rev && *i == id {
                return Some(doc.clone());
            }
        }
        let mut doc = self.doc().clone();
        let s = doc.find_shape_mut(id)?;
        let before = s.effects.len();
        s.effects.retain(|e| !matches!(e, Effect::Symmetry { .. }));
        if s.effects.len() == before {
            return None;
        }
        let doc = std::rc::Rc::new(doc);
        *self.symmetry_doc.borrow_mut() = Some((rev, id, doc.clone()));
        Some(doc)
    }

    /// The copies of the symmetry being edited, in page space, when Show
    /// full preview is off (they are then drawn as outlines).
    pub fn symmetry_copies_outlined(&self) -> Vec<Shape> {
        if self.settings.symmetry_full_preview {
            return Vec::new();
        }
        let Some(id) = self.symmetry_edit else {
            return Vec::new();
        };
        let Some(s) = self.doc().shape_on_page(id) else {
            return Vec::new();
        };
        let mut only = s.clone();
        only.effects
            .retain(|e| matches!(e, Effect::Symmetry { .. }));
        tracedraw_core::live::evaluate(&only).above
    }
}
