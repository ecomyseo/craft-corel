//! Object > Clone Effect: a clone keeps following the source object's
//! drop shadow, transparency, extrusion, contour or blend until the
//! clone's own effect is edited.
//!
//! The link lives in the clone's object data (`effect.clone_of` =
//! `<source id>:<shadow|transparency>`), so it survives save and load. After
//! every command that changes a shadow, opacity or transparency, the
//! clones of the changed shapes are refreshed; a command that edits a clone
//! directly breaks its link first.

use crate::app::{App, EffectKind};
use crate::i18n::tr;
use tracedraw_core::{
    document::{Shape, ShapeKind},
    live::Effect,
    Command, ShapeId,
};

const KEY: &str = "effect.clone_of";

fn kind_name(kind: EffectKind) -> &'static str {
    match kind {
        EffectKind::Shadow => "shadow",
        EffectKind::Transparency => "transparency",
        EffectKind::Extrude => "extrude",
        EffectKind::Contour => "contour",
        EffectKind::Blend => "blend",
    }
}

fn copy_kind(kind: EffectKind) -> crate::copy_effect::CopyKind {
    use crate::copy_effect::CopyKind;
    match kind {
        EffectKind::Shadow => CopyKind::Shadow,
        EffectKind::Transparency => CopyKind::Transparency,
        EffectKind::Extrude => CopyKind::Extrude,
        EffectKind::Contour => CopyKind::Contour,
        EffectKind::Blend => CopyKind::Blend,
    }
}

/// The effect of `kind` among `effects` (the live ones: transparency,
/// extrusion, contour, blend).
fn effect_of(effects: &[Effect], kind: EffectKind) -> Option<&Effect> {
    let k = copy_kind(kind);
    effects.iter().find(|e| k.matches(e))
}

/// A blend's own settings, the part a clone follows (the objects it
/// joins are its own).
fn blend_settings(e: &Effect) -> Option<(u32, f64, f64, f64)> {
    match e {
        Effect::Blend { steps, accel_objects, accel_colors, rotation, .. } => {
            Some((*steps, *accel_objects, *accel_colors, *rotation))
        }
        _ => None,
    }
}

/// Parse `effect.clone_of` from object data.
pub fn clone_link(data: &[(String, String)]) -> Option<(ShapeId, EffectKind)> {
    let (_, v) = data.iter().find(|(k, _)| k == KEY)?;
    let (id, kind) = v.split_once(':')?;
    let id = ShapeId(id.parse().ok()?);
    let kind = match kind {
        "shadow" => EffectKind::Shadow,
        "transparency" => EffectKind::Transparency,
        "extrude" => EffectKind::Extrude,
        "contour" => EffectKind::Contour,
        "blend" => EffectKind::Blend,
        _ => return None,
    };
    Some((id, kind))
}

/// The clone's effects with its extrusion, contour or blend following the
/// source's; `None` when there is nothing to change.
fn follow(src: &Shape, clone: &Shape, kind: EffectKind) -> Option<Vec<Effect>> {
    let e = effect_of(&src.effects, kind)?.clone();
    let effects: Vec<Effect> = if kind == EffectKind::Blend {
        let (steps, ao, ac, r) = blend_settings(&e)?;
        clone
            .effects
            .iter()
            .cloned()
            .map(|mut x| {
                if let Effect::Blend { steps: s1, accel_objects: o, accel_colors: k, rotation: q, .. } = &mut x {
                    (*s1, *o, *k, *q) = (steps, ao, ac, r);
                }
                x
            })
            .collect()
    } else {
        crate::copy_effect::with_effect(
            &clone.effects,
            copy_kind(kind),
            crate::copy_effect::carried(&e, src, clone),
        )
    };
    (effects != clone.effects).then_some(effects)
}

fn walk<'a>(s: &'a Shape, out: &mut Vec<&'a Shape>) {
    out.push(s);
    match &s.kind {
        ShapeKind::Group { children } => children.iter().for_each(|c| walk(c, out)),
        ShapeKind::ClipFrame { frame, contents } => {
            walk(frame, out);
            contents.iter().for_each(|c| walk(c, out));
        }
        _ => {}
    }
}

/// Which shapes a command touches for the purpose of effect cloning, and
/// the kinds of effect it may change on them.
fn touched(cmd: &Command) -> Option<(Vec<ShapeId>, Vec<EffectKind>)> {
    match cmd {
        Command::SetShadow { shapes, .. } => Some((shapes.clone(), vec![EffectKind::Shadow])),
        Command::SetOpacity { shapes, .. } => Some((shapes.clone(), vec![EffectKind::Transparency])),
        Command::SetEffects { shape, .. } => Some((
            vec![*shape],
            vec![
                EffectKind::Transparency,
                EffectKind::Extrude,
                EffectKind::Contour,
                EffectKind::Blend,
            ],
        )),
        _ => None,
    }
}

impl App {
    /// Every (clone id, source id, kind) link in the document.
    pub fn effect_clones(&self) -> Vec<(ShapeId, ShapeId, EffectKind)> {
        let mut all = Vec::new();
        for l in self.doc().all_layers() {
            for s in &l.shapes {
                walk(s, &mut all);
            }
        }
        all.iter()
            .filter_map(|s| clone_link(&s.data).map(|(src, k)| (s.id, src, k)))
            .collect()
    }

    pub fn is_effect_clone(&self, id: ShapeId) -> bool {
        self.doc()
            .find_shape(id)
            .map(|s| clone_link(&s.data).is_some())
            .unwrap_or(false)
    }

    /// Object > Clone Effect > ... From: copy the effect now and remember the
    /// source so later changes follow.
    pub fn clone_effect_from(&mut self, source: ShapeId, kind: EffectKind) {
        let targets: Vec<ShapeId> = self
            .selection
            .iter()
            .copied()
            .filter(|s| *s != source)
            .collect();
        if targets.is_empty() {
            return;
        }
        self.copy_effect_from(source, copy_kind(kind));
        let mut cmds = Vec::new();
        for id in targets {
            let Some(s) = self.doc().find_shape(id) else {
                continue;
            };
            let mut data = s.data.clone();
            data.retain(|(k, _)| k != KEY);
            data.push((KEY.into(), format!("{}:{}", source.raw(), kind_name(kind))));
            cmds.push(Command::SetObjectData { shape: id, data });
        }
        let _ = self.engine.run_batch("Clone Effect", &cmds);
        self.status = tr("status.cloned_effect");
    }

    /// The commands that make the clones of the extrusions, contours and
    /// blends `cmds` change follow them, for the same undo step (the
    /// dockers change those effects in batches).
    pub fn clone_followers(&self, cmds: &[Command]) -> Vec<Command> {
        let clones = self.effect_clones();
        let mut out = Vec::new();
        for cmd in cmds {
            let Command::SetEffects { shape: src, effects } = cmd else {
                continue;
            };
            let Some(mut s) = self.doc().find_shape(*src).cloned() else {
                continue;
            };
            s.effects = effects.clone();
            for (clone, from, kind) in &clones {
                if from != src || !matches!(kind, EffectKind::Extrude | EffectKind::Contour | EffectKind::Blend) {
                    continue;
                }
                if let Some(next) = self.doc().find_shape(*clone).and_then(|c| follow(&s, c, *kind)) {
                    out.push(Command::SetEffects { shape: *clone, effects: next });
                }
            }
        }
        out
    }

    /// Before a batch of effect changes: the clones edited break their
    /// link, and the clones of the sources edited follow in the batch.
    pub fn prepare_effect_batch(&mut self, cmds: &mut Vec<Command>) {
        for c in cmds.iter() {
            self.break_clone_links_for(c);
        }
        let followers = self.clone_followers(cmds);
        cmds.extend(followers);
    }

    /// Before a user command runs: editing a clone's own effect breaks the
    /// link.
    pub fn break_clone_links_for(&mut self, cmd: &Command) {
        let Some((shapes, kinds)) = touched(cmd) else {
            return;
        };
        let mut cmds = Vec::new();
        for id in shapes {
            let Some(s) = self.doc().find_shape(id) else {
                continue;
            };
            let Some(kind) = clone_link(&s.data).map(|(_, k)| k).filter(|k| kinds.contains(k)) else {
                continue;
            };
            // A change of the clone's effects that leaves the cloned one
            // as it was (another effect edited) keeps the link.
            if let Command::SetEffects { effects, .. } = cmd {
                let same = match kind {
                    EffectKind::Blend => {
                        effect_of(&s.effects, kind).and_then(blend_settings)
                            == effect_of(effects, kind).and_then(blend_settings)
                    }
                    EffectKind::Transparency => true,
                    _ => effect_of(&s.effects, kind) == effect_of(effects, kind),
                };
                if same {
                    continue;
                }
            }
            let mut data = s.data.clone();
            data.retain(|(k, _)| k != KEY);
            cmds.push(Command::SetObjectData { shape: id, data });
        }
        if !cmds.is_empty() {
            let _ = self.engine.run_batch("Clone Effect", &cmds);
        }
    }

    /// After a user command ran: push the new effect to the clones of the
    /// changed shapes. Runs on the engine directly so it never recurses.
    pub fn sync_effect_clones_for(&mut self, cmd: &Command) {
        let Some((changed, kinds)) = touched(cmd) else {
            return;
        };
        let clones = self.effect_clones();
        for (clone, src, kind) in clones {
            if !kinds.contains(&kind) || !changed.contains(&src) {
                continue;
            }
            let Some(s) = self.doc().find_shape(src).cloned() else {
                continue;
            };
            let cmds = match kind {
                EffectKind::Extrude | EffectKind::Contour | EffectKind::Blend => {
                    let Some(c) = self.doc().find_shape(clone).cloned() else {
                        continue;
                    };
                    match follow(&s, &c, kind) {
                        Some(effects) => vec![Command::SetEffects { shape: clone, effects }],
                        None => continue,
                    }
                }
                EffectKind::Shadow => vec![Command::SetShadow {
                    shapes: vec![clone],
                    shadow: s.shadow,
                }],
                EffectKind::Transparency => {
                    let Some(c) = self.doc().find_shape(clone).cloned() else {
                        continue;
                    };
                    let mut effects: Vec<Effect> = c
                        .effects
                        .iter()
                        .filter(|e| !matches!(e, Effect::Transparency { .. }))
                        .cloned()
                        .collect();
                    if let Some(t) = s
                        .effects
                        .iter()
                        .find(|e| matches!(e, Effect::Transparency { .. }))
                    {
                        effects.push(t.clone());
                    }
                    vec![
                        Command::SetOpacity {
                            shapes: vec![clone],
                            opacity: s.opacity,
                        },
                        Command::SetEffects {
                            shape: clone,
                            effects,
                        },
                    ]
                }
            };
            let _ = self.engine.run_batch(cmd.label(), &cmds);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Rect;

    fn rect(app: &mut App, x: f64) -> ShapeId {
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(x, 10.0, x + 20.0, 30.0),
            radius: 0.0,
            corners: None,
        })
        .unwrap()
    }

    fn contour_steps(app: &App, id: ShapeId) -> Option<u32> {
        app.doc().find_shape(id)?.effects.iter().find_map(|e| match e {
            Effect::Contour { steps, .. } => Some(*steps),
            _ => None,
        })
    }

    /// Clone Effect > Contour From and Extrude From: the clone takes the
    /// effect and follows it when the source's is changed in its docker,
    /// in the same undo step, until the clone's own is changed.
    #[test]
    fn a_contour_and_an_extrusion_clone_follow_their_source() {
        let mut app = App::headless();
        let src = rect(&mut app, 10.0);
        let clone = rect(&mut app, 50.0);
        app.select(vec![src]);
        app.contour.steps = 3;
        app.add_contour();
        assert_eq!(contour_steps(&app, src), Some(3));
        app.select(vec![clone]);
        app.clone_effect_from(src, EffectKind::Contour);
        assert_eq!(contour_steps(&app, clone), Some(3));
        assert!(app.is_effect_clone(clone));
        // The source's contour changes in the docker: the clone follows.
        app.select(vec![src]);
        app.contour.steps = 6;
        app.apply_contour_settings();
        assert_eq!(contour_steps(&app, src), Some(6));
        assert_eq!(contour_steps(&app, clone), Some(6), "the clone follows");
        app.undo();
        assert_eq!(contour_steps(&app, clone), Some(3), "one undo step for both");
        app.redo();
        // The clone's own contour changed: the link breaks.
        app.select(vec![clone]);
        app.contour.steps = 2;
        app.apply_contour_settings();
        assert!(!app.is_effect_clone(clone));
        app.select(vec![src]);
        app.contour.steps = 9;
        app.apply_contour_settings();
        assert_eq!(contour_steps(&app, clone), Some(2));
        // Extrusion: the clone takes it and is linked.
        let ex = rect(&mut app, 90.0);
        let ex_clone = rect(&mut app, 130.0);
        app.select(vec![ex]);
        app.add_extrusion();
        assert!(app.doc().find_shape(ex).expect("ex").effects.iter().any(|e| matches!(e, Effect::Extrude { .. })));
        app.select(vec![ex_clone]);
        app.clone_effect_from(ex, EffectKind::Extrude);
        assert!(app.doc().find_shape(ex_clone).expect("c").effects.iter().any(|e| matches!(e, Effect::Extrude { .. })));
        assert_eq!(clone_link(&app.doc().find_shape(ex_clone).expect("c").data), Some((ex, EffectKind::Extrude)));
    }

    #[test]
    fn clone_follows_source_until_edited() {
        let mut app = App::headless();
        let src = rect(&mut app, 10.0);
        let clone = rect(&mut app, 50.0);
        app.run(Command::SetOpacity {
            shapes: vec![src],
            opacity: 0.5,
        });
        app.select(vec![clone]);
        app.clone_effect_from(src, EffectKind::Transparency);
        assert!(app.is_effect_clone(clone));
        assert_eq!(app.doc().find_shape(clone).unwrap().opacity, 0.5);
        // The source changes, the clone follows.
        app.run(Command::SetOpacity {
            shapes: vec![src],
            opacity: 0.2,
        });
        assert_eq!(app.doc().find_shape(clone).unwrap().opacity, 0.2);
        // Editing the clone breaks the link.
        app.run(Command::SetOpacity {
            shapes: vec![clone],
            opacity: 0.9,
        });
        assert!(!app.is_effect_clone(clone));
        app.run(Command::SetOpacity {
            shapes: vec![src],
            opacity: 0.1,
        });
        assert_eq!(app.doc().find_shape(clone).unwrap().opacity, 0.9);
    }
}
