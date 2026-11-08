//! Object > Copy Effect: the selected objects take an effect of the
//! object clicked next. Drop shadow and transparency are copied as they
//! are; live effects (perspective, distortion, extrusion, contour, lens,
//! bevel, block shadow) replace the targets' own of the same kind in its
//! place, or join their effects. What an effect pins to the source's own
//! space (a perspective's corners, a distortion's centre, a vanishing
//! point locked to the object) is carried over by where it sits in the
//! source's box, so it sits the same way in each target's. A blend's
//! steps, accelerations and rotation go to the selection's blends.
//! One undo step.

use crate::app::App;
use tracedraw_core::{
    geometry::{Affine, Point, Rect, Shape as _, Vec2},
    live::Effect,
    Command, Shape, ShapeId,
};

/// The effects Copy Effect offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    Shadow,
    Transparency,
    Perspective,
    Distortion,
    Extrude,
    Contour,
    Blend,
    Lens,
    Bevel,
    BlockShadow,
}

impl CopyKind {
    /// The menu's entries after Drop Shadow, Transparency and Envelope.
    pub const LIVE: [CopyKind; 8] = [
        CopyKind::Perspective,
        CopyKind::Distortion,
        CopyKind::Extrude,
        CopyKind::Contour,
        CopyKind::Blend,
        CopyKind::Lens,
        CopyKind::Bevel,
        CopyKind::BlockShadow,
    ];

    pub fn key(self) -> &'static str {
        match self {
            CopyKind::Shadow => "menu.object.copy_shadow_from",
            CopyKind::Transparency => "menu.object.copy_transparency_from",
            CopyKind::Perspective => "menu.object.copy_perspective_from",
            CopyKind::Distortion => "menu.object.copy_distortion_from",
            CopyKind::Extrude => "menu.object.copy_extrude_from",
            CopyKind::Contour => "menu.object.copy_contour_from",
            CopyKind::Blend => "menu.object.copy_blend_from",
            CopyKind::Lens => "menu.object.copy_lens_from",
            CopyKind::Bevel => "menu.object.copy_bevel_from",
            CopyKind::BlockShadow => "menu.object.copy_block_shadow_from",
        }
    }

    /// Is `e` an effect of this kind?
    pub fn matches(self, e: &Effect) -> bool {
        match self {
            CopyKind::Perspective => matches!(e, Effect::Perspective { .. }),
            CopyKind::Distortion => matches!(e, Effect::Distort(_)),
            CopyKind::Extrude => matches!(e, Effect::Extrude { .. }),
            CopyKind::Contour => matches!(e, Effect::Contour { .. }),
            CopyKind::Blend => matches!(e, Effect::Blend { .. }),
            CopyKind::Lens => matches!(e, Effect::Lens(_)),
            CopyKind::Bevel => matches!(e, Effect::Bevel { .. }),
            CopyKind::BlockShadow => matches!(e, Effect::BlockShadow { .. }),
            CopyKind::Transparency => matches!(e, Effect::Transparency { .. }),
            CopyKind::Shadow => false,
        }
    }
}

/// The map taking box `from` onto box `to` (each side stretched on its
/// own; a box with no width or height keeps that scale).
pub fn box_map(from: Rect, to: Rect) -> Affine {
    let sx = if from.width().abs() > 1e-9 {
        to.width() / from.width()
    } else {
        1.0
    };
    let sy = if from.height().abs() > 1e-9 {
        to.height() / from.height()
    } else {
        1.0
    };
    Affine::translate(Vec2::new(to.x0, to.y0))
        * Affine::scale_non_uniform(sx, sy)
        * Affine::translate(Vec2::new(-from.x0, -from.y0))
}

fn local_box(s: &Shape) -> Rect {
    s.local_path().bounding_box()
}

/// `e` (an effect of `src`) as it sits on `dst`.
pub fn carried(e: &Effect, src: &Shape, dst: &Shape) -> Effect {
    let m = box_map(local_box(src), local_box(dst));
    let ok = |p: Point| p.x.is_finite() && p.y.is_finite();
    match e {
        Effect::Perspective { corners } => {
            let moved = corners.map(|c| m * c);
            if moved.iter().all(|p| ok(*p)) {
                Effect::Perspective { corners: moved }
            } else {
                e.clone()
            }
        }
        Effect::Distort(d) => match d.center() {
            Some(c) if ok(m * c) => Effect::Distort(d.with_center(Some(m * c))),
            _ => e.clone(),
        },
        Effect::Extrude {
            vanishing: Some(v),
            on_object: true,
            ..
        } if ok(m * *v) => {
            let mut x = e.clone();
            if let Effect::Extrude { vanishing, .. } = &mut x {
                *vanishing = Some(m * *v);
            }
            x
        }
        _ => e.clone(),
    }
}

/// `effects` with the one `kind` matches replaced in place by `e` (added
/// at the end when there is none).
fn with_effect(effects: &[Effect], kind: CopyKind, e: Effect) -> Vec<Effect> {
    let mut out = Vec::with_capacity(effects.len() + 1);
    let mut placed = false;
    for x in effects {
        if kind.matches(x) {
            if !placed {
                out.push(e.clone());
                placed = true;
            }
        } else {
            out.push(x.clone());
        }
    }
    if !placed {
        out.push(e);
    }
    out
}

impl App {
    /// Copy the effect `kind` of `source` to the other selected objects.
    pub fn copy_effect_from(&mut self, source: ShapeId, kind: CopyKind) {
        let Some(src) = self.doc().find_shape(source).cloned() else {
            return;
        };
        let targets: Vec<Shape> = self
            .selected_shapes()
            .into_iter()
            .filter(|s| s.id != source)
            .collect();
        if targets.is_empty() {
            return;
        }
        let ids: Vec<ShapeId> = targets.iter().map(|s| s.id).collect();
        let mut cmds = Vec::new();
        match kind {
            CopyKind::Shadow => cmds.push(Command::SetShadow {
                shapes: ids,
                shadow: src.shadow,
            }),
            CopyKind::Transparency => {
                cmds.push(Command::SetOpacity {
                    shapes: ids,
                    opacity: src.opacity,
                });
                let t = src.effects.iter().find(|e| kind.matches(e)).cloned();
                for s in &targets {
                    let mut effects: Vec<Effect> = s
                        .effects
                        .iter()
                        .filter(|e| !kind.matches(e))
                        .cloned()
                        .collect();
                    effects.extend(t.clone());
                    cmds.push(Command::SetEffects {
                        shape: s.id,
                        effects,
                    });
                }
            }
            CopyKind::Blend => {
                let Some(Effect::Blend {
                    steps,
                    accel_objects,
                    accel_colors,
                    rotation,
                    ..
                }) = src.effects.iter().find(|e| kind.matches(e)).cloned()
                else {
                    self.status = crate::i18n::tr("status.no_effect_to_copy");
                    return;
                };
                self.change_blends("Copy Effect", |e| {
                    let mut e = e.clone();
                    if let Effect::Blend {
                        steps: s,
                        accel_objects: o,
                        accel_colors: c,
                        rotation: r,
                        ..
                    } = &mut e
                    {
                        *s = steps;
                        *o = accel_objects;
                        *c = accel_colors;
                        *r = rotation;
                    }
                    Some(e)
                });
                return;
            }
            _ => {
                let Some(e) = src.effects.iter().find(|e| kind.matches(e)).cloned() else {
                    self.status = crate::i18n::tr("status.no_effect_to_copy");
                    return;
                };
                for s in &targets {
                    cmds.push(Command::SetEffects {
                        shape: s.id,
                        effects: with_effect(&s.effects, kind, carried(&e, &src, s)),
                    });
                }
            }
        }
        if let Err(e) = self.engine.run_batch("Copy Effect", &cmds) {
            self.status = e.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{distort::Distort, ShapeKind};

    fn rect(app: &mut App, r: Rect) -> ShapeId {
        app.new_shape(ShapeKind::Rect {
            rect: r,
            radius: 0.0,
            corners: None,
        })
        .expect("rect")
    }

    fn effects(app: &App, id: ShapeId) -> Vec<Effect> {
        app.doc().find_shape(id).expect("object").effects.clone()
    }

    #[test]
    fn live_effects_copy_and_sit_the_same_way() {
        let mut app = App::headless();
        let src = rect(&mut app, Rect::new(0.0, 0.0, 10.0, 10.0));
        let dst = rect(&mut app, Rect::new(100.0, 0.0, 140.0, 20.0));
        // A distortion centred at the source's top right corner, and a
        // contour.
        app.run(Command::SetEffects {
            shape: src,
            effects: vec![
                Effect::Distort(Distort::Twister {
                    angle_deg: 90.0,
                    center: Some(Point::new(10.0, 10.0)),
                }),
                Effect::Contour {
                    steps: 2,
                    offset: 1.0,
                    outside: true,
                    to_center: false,
                    fill_to: tracedraw_core::Color::BLACK,
                    outline_to: None,
                    color_blend: 0,
                    corners: None,
                    accel_objects: 0.0,
                    accel_colors: 0.0,
                },
            ],
        });
        app.select(vec![dst]);
        let before = app.engine.history_labels().0.len();
        app.copy_effect_from(src, CopyKind::Distortion);
        app.copy_effect_from(src, CopyKind::Contour);
        assert_eq!(app.engine.history_labels().0.len(), before + 2);
        let e = effects(&app, dst);
        assert_eq!(e.len(), 2);
        // The centre at the target's top right corner.
        assert_eq!(
            e[0],
            Effect::Distort(Distort::Twister {
                angle_deg: 90.0,
                center: Some(Point::new(140.0, 20.0)),
            })
        );
        assert!(matches!(e[1], Effect::Contour { steps: 2, .. }));
        // Copied again: replaced in its place, not added.
        app.copy_effect_from(src, CopyKind::Distortion);
        assert_eq!(effects(&app, dst).len(), 2);
        // A perspective's corners follow the box.
        let corners = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(8.0, 10.0),
            Point::new(2.0, 10.0),
        ];
        app.run(Command::SetEffects {
            shape: src,
            effects: vec![Effect::Perspective { corners }],
        });
        app.copy_effect_from(src, CopyKind::Perspective);
        let Some(Effect::Perspective { corners: c }) = effects(&app, dst)
            .into_iter()
            .find(|e| matches!(e, Effect::Perspective { .. }))
        else {
            panic!("a perspective");
        };
        assert_eq!(c[2], Point::new(132.0, 20.0));
        // Nothing of that kind on the source: nothing changes.
        let n = app.engine.history_labels().0.len();
        app.copy_effect_from(src, CopyKind::Lens);
        assert_eq!(app.engine.history_labels().0.len(), n);
    }

    #[test]
    fn every_kind_has_a_menu_entry() {
        for k in CopyKind::LIVE {
            assert!(k.key().starts_with("menu.object.copy_"));
        }
        let m = box_map(
            Rect::new(0.0, 0.0, 0.0, 10.0),
            Rect::new(5.0, 5.0, 5.0, 25.0),
        );
        assert_eq!(m * Point::new(0.0, 5.0), Point::new(5.0, 15.0));
    }
}
