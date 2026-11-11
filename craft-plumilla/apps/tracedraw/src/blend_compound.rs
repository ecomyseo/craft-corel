//! Split, Fuse Start, Fuse End and Map Nodes for blends: the Blend tool's
//! right-click menu and the Blend docker (see docs/behavior/blend.md).
//!
//! A split blend is a compound blend: the step chosen becomes an object
//! of the drawing that ends the first blend and starts the second, so
//! each part can be edited on its own. Fusing joins two such parts back
//! into one blend and removes the object between them.

use crate::app::App;
use crate::blend_link::{end_copy, with_blend};
use tracedraw_core::{
    document::Shape,
    geometry::{BezPath, PathEl},
    live::{evaluate, BlendNodes, Effect},
    Command, Point, ShapeId,
};

/// The blend's steps as objects, from the start's side to the end's.
pub fn steps_of(start: &Shape, e: &Effect) -> Vec<Shape> {
    let Effect::Blend { end_id, .. } = e else {
        return Vec::new();
    };
    let mut s = start.clone();
    let mut e = e.clone();
    if let Effect::Blend { options, .. } = &mut e {
        options.steps_below = false;
    }
    s.effects = vec![e];
    let mut steps = evaluate(&s).above;
    // An end kept only in the effect is drawn after the steps.
    if end_id.is_none() {
        steps.pop();
    }
    steps
}

fn flush(sub: &mut Vec<Point>, closed: bool, out: &mut Vec<(Point, Point)>) {
    // A closed subpath whose last node repeats its first has that node once.
    if closed && sub.len() > 1 && (sub[0] - sub[sub.len() - 1]).hypot() < 1e-9 {
        sub.pop();
    }
    let n = sub.len();
    for i in 0..n {
        let next = if i + 1 < n {
            Some(sub[i + 1])
        } else if closed && n > 1 {
            Some(sub[0])
        } else {
            None
        };
        if let Some(next) = next {
            out.push((sub[i], next));
        }
    }
    sub.clear();
}

/// The nodes of `path` (the ends of its segments), each with the next one
/// along its subpath, round a closed one; an open subpath's last node has
/// none and is left out.
pub fn nodes_with_next(path: &BezPath) -> Vec<(Point, Point)> {
    let mut out = Vec::new();
    let mut sub = Vec::new();
    let mut closed = false;
    for el in path.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                flush(&mut sub, closed, &mut out);
                closed = false;
                sub.push(p);
            }
            PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => sub.push(p),
            PathEl::ClosePath => closed = true,
        }
    }
    flush(&mut sub, closed, &mut out);
    out
}

/// The node of `s` nearest the page point `p` and the node after it, in
/// the object's own coordinates (as [`BlendNodes`] keeps them).
fn node_near(s: &Shape, p: Point) -> Option<[Point; 2]> {
    nodes_with_next(&s.local_path())
        .into_iter()
        .min_by(|a, b| {
            let da = (s.transform * a.0 - p).hypot();
            let db = (s.transform * b.0 - p).hypot();
            da.total_cmp(&db)
        })
        .map(|(n, next)| [n, next])
}

/// The first node of `s` and the one after it, in its own coordinates.
fn first_node(s: &Shape) -> Option<[Point; 2]> {
    nodes_with_next(&s.local_path()).first().map(|(a, b)| [*a, *b])
}

impl App {
    /// The blend that `id` starts.
    fn blend_of(&self, id: ShapeId) -> Option<Effect> {
        self.doc()
            .find_shape(id)?
            .effects
            .iter()
            .find(|e| matches!(e, Effect::Blend { .. }))
            .cloned()
    }

    /// The blend that ends at `id`: its start object and the blend.
    fn blend_ending_at(&self, id: ShapeId) -> Option<(ShapeId, Effect)> {
        self.all_blends()
            .into_iter()
            .find(|(_, e)| matches!(e, Effect::Blend { end_id: Some(x), .. } if *x == id))
    }

    /// Split is offered for a selected blend that does not follow a path.
    pub fn can_split_blend(&self) -> bool {
        self.first_selected_blend()
            .is_some_and(|(_, e)| matches!(e, Effect::Blend { path: None, .. }))
    }

    /// The step of the selected blend under the page point `p`: of the
    /// steps whose box holds it, the one with the nearest centre.
    pub fn blend_step_at(&self, p: Point) -> Option<usize> {
        let (a, e) = self.first_selected_blend()?;
        let start = self.doc().find_shape(a)?;
        steps_of(start, &e)
            .iter()
            .enumerate()
            .filter(|(_, s)| s.bounds().contains(p))
            .min_by(|x, y| {
                let dx = (x.1.bounds().center() - p).hypot();
                let dy = (y.1.bounds().center() - p).hypot();
                dx.total_cmp(&dy)
            })
            .map(|(i, _)| i)
    }

    /// Split: step `k` of the selected blend (0 = the one next to its
    /// start) becomes an object of the drawing that ends the first part
    /// and starts the second. The steps on either side stay where they
    /// were and the turn stays with the second part; a part left with no
    /// steps has no
    /// blend. One undo step; the new object comes out selected.
    pub fn split_blend(&mut self, k: usize) -> Option<ShapeId> {
        let (a, e) = self.first_selected_blend()?;
        let Effect::Blend {
            path: None,
            rotation,
            options,
            ..
        } = &e
        else {
            return None;
        };
        let (rotation, below) = (*rotation, options.steps_below);
        // The new object is already turned part of the way: the first
        // part only morphs into it, and the second keeps the whole turn,
        // so that fusing the two gives the turn back.
        let start = self.doc().find_shape(a)?.clone();
        let steps = steps_of(&start, &e);
        let n = steps.len();
        if k >= n {
            return None;
        }
        let (layer, _) = self.doc().locate(a).ok()?;
        let mid_id = self.engine.new_shape_id();
        let mut mid = steps[k].clone();
        mid.id = mid_id;
        mid.effects.clear();
        let mid_nodes = first_node(&mid);
        // From the start to the new object.
        let mut first = e.clone();
        if let Effect::Blend {
            end,
            end_id,
            steps,
            rotation: r,
            options,
            ..
        } = &mut first
        {
            **end = end_copy(&mid);
            *end_id = Some(mid_id);
            *steps = k as u32;
            *r = 0.0;
            options.map_nodes = options
                .map_nodes
                .zip(mid_nodes)
                .map(|(m, n)| BlendNodes { start: m.start, end: n });
        }
        // From the new object to the old end.
        let mut second = e.clone();
        if let Effect::Blend {
            steps,
            rotation: r,
            options,
            ..
        } = &mut second
        {
            *steps = (n - k - 1) as u32;
            *r = rotation;
            options.map_nodes = options
                .map_nodes
                .zip(mid_nodes)
                .map(|(m, n)| BlendNodes { start: n, end: m.end });
        }
        if n - k > 1 {
            mid.effects.push(second);
        }
        let cmds = vec![
            Command::AddShape { layer, shape: mid },
            // Between the two objects, where the steps are drawn.
            Command::OrderRelative {
                shapes: vec![mid_id],
                reference: a,
                in_front: !below,
            },
            Command::SetEffects {
                shape: a,
                effects: with_blend(&start.effects, (k > 0).then_some(first)),
            },
        ];
        if let Err(err) = self.engine.run_batch("Split Blend", &cmds) {
            self.status = err.to_string();
            return None;
        }
        self.refresh_blends();
        self.select(vec![mid_id]);
        Some(mid_id)
    }

    /// The parts a Fuse joins: (first start, its blend, the object between,
    /// its blend). Fuse End takes a selected blend whose end object starts
    /// another blend; Fuse Start one whose start object ends another.
    fn fuse_parts(&self, at_end: bool) -> Option<(ShapeId, Effect, ShapeId, Effect)> {
        self.selected_blends().into_iter().find_map(|(id, e)| {
            let (a, ea, mid, em) = if at_end {
                let Effect::Blend {
                    end_id: Some(mid), ..
                } = &e
                else {
                    return None;
                };
                let em = self.blend_of(*mid)?;
                (id, e.clone(), *mid, em)
            } else {
                let (a, ea) = self.blend_ending_at(id)?;
                (a, ea, id, e.clone())
            };
            let free = |x: &Effect| matches!(x, Effect::Blend { path: None, .. });
            let loops = matches!(&em, Effect::Blend { end_id: Some(x), .. } if *x == a);
            (free(&ea) && free(&em) && a != mid && !loops).then_some((a, ea, mid, em))
        })
    }

    /// Whether Fuse End (`at_end`) or Fuse Start has two parts to join.
    pub fn can_fuse_blend(&self, at_end: bool) -> bool {
        self.fuse_parts(at_end).is_some()
    }

    /// Fuse End / Fuse Start: the two parts of a split blend become one
    /// again and the object between them goes; the blend keeps as many
    /// steps as the parts and that object made, and both parts' turn. One
    /// undo step; the start object comes out selected.
    pub fn fuse_blend(&mut self, at_end: bool) -> bool {
        let Some((a, ea, mid, em)) = self.fuse_parts(at_end) else {
            return false;
        };
        let (
            Effect::Blend {
                steps: na,
                rotation: ra,
                options: oa,
                ..
            },
            Effect::Blend {
                end,
                end_id,
                steps: nm,
                rotation: rm,
                options: om,
                ..
            },
        ) = (&ea, &em)
        else {
            return false;
        };
        let mut joined = ea.clone();
        if let Effect::Blend {
            end: je,
            end_id: jid,
            steps,
            rotation,
            options,
            ..
        } = &mut joined
        {
            *je = end.clone();
            *jid = *end_id;
            *steps = na + nm + 1;
            *rotation = ra + rm;
            options.map_nodes = match (oa.map_nodes, om.map_nodes) {
                (Some(x), Some(y)) => Some(BlendNodes {
                    start: x.start,
                    end: y.end,
                }),
                _ => None,
            };
        }
        let Some(start) = self.doc().find_shape(a).cloned() else {
            return false;
        };
        let cmds = vec![
            Command::SetEffects {
                shape: a,
                effects: with_blend(&start.effects, Some(joined)),
            },
            Command::DeleteShapes { shapes: vec![mid] },
        ];
        let label = if at_end { "Fuse End" } else { "Fuse Start" };
        if let Err(err) = self.engine.run_batch(label, &cmds) {
            self.status = err.to_string();
            return false;
        }
        self.refresh_blends();
        self.select(vec![a]);
        true
    }

    /// Map Nodes: the steps pair the start object's node nearest the page
    /// point `on_start` with the end object's node nearest `on_end`, each
    /// outline running on to its next node. One undo step.
    pub fn map_blend_nodes(&mut self, on_start: Point, on_end: Point) -> bool {
        let Some((a, e)) = self.first_selected_blend() else {
            return false;
        };
        let Effect::Blend { end, end_id, .. } = &e else {
            return false;
        };
        let Some(start) = self.doc().find_shape(a).cloned() else {
            return false;
        };
        let end_shape = match end_id {
            Some(id) => self.doc().find_shape(*id).cloned(),
            None => Some((**end).clone()),
        };
        let Some(end_shape) = end_shape else {
            return false;
        };
        let (Some(s), Some(t)) = (node_near(&start, on_start), node_near(&end_shape, on_end)) else {
            return false;
        };
        let mut next = e.clone();
        if let Effect::Blend { options, .. } = &mut next {
            options.map_nodes = Some(BlendNodes { start: s, end: t });
        }
        if next == e {
            return false;
        }
        let cmds = vec![Command::SetEffects {
            shape: a,
            effects: with_blend(&start.effects, Some(next)),
        }];
        if let Err(err) = self.engine.run_batch("Map Nodes", &cmds) {
            self.status = err.to_string();
            return false;
        }
        true
    }

    /// A click while Split or Map Nodes waits for one (see
    /// `interaction2`): Split takes the step under `p`; Map Nodes takes a
    /// node of the start object, then one of the end object.
    pub(crate) fn blend_compound_click(&mut self, p: Point) {
        if self.pending_blend_split {
            self.pending_blend_split = false;
            match self.blend_step_at(p) {
                Some(k) => {
                    self.split_blend(k);
                    self.status.clear();
                }
                None => self.status = crate::i18n::tr("status.cancelled"),
            }
            return;
        }
        match self.pending_map_nodes.take() {
            Some(None) => {
                self.pending_map_nodes = Some(Some(p));
                self.status = crate::i18n::tr("status.click_end_node");
            }
            Some(Some(first)) => {
                self.map_blend_nodes(first, p);
                self.status.clear();
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{geometry::Rect, Color, Fill, ShapeKind};

    fn square(app: &mut App, x: f64, c: Color) -> ShapeId {
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(x, 0.0, x + 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("square");
        app.run(Command::SetFill {
            shapes: vec![id],
            fill: Fill::Solid(c),
        });
        id
    }

    fn blended(steps: u32) -> (App, ShapeId, ShapeId) {
        let mut app = App::headless();
        let a = square(&mut app, 0.0, Color::rgb8(250, 0, 0));
        let b = square(&mut app, 100.0, Color::rgb8(0, 0, 250));
        app.select(vec![a, b]);
        app.blend_selection(steps, 0.0, 0.0, 30.0);
        app.refresh_blends();
        (app, a, b)
    }

    /// Every step of the blends that start at `a` and follow on, and each
    /// object between them, in order: centres and fill colours.
    fn chain(app: &App, a: ShapeId) -> Vec<(Point, [u8; 3])> {
        let mut out = Vec::new();
        let mut at = a;
        while let Some(e) = app.blend_of(at) {
            let s = app.doc().find_shape(at).expect("start").clone();
            for st in steps_of(&s, &e) {
                out.push((st.bounds().center(), st.fill.preview_color().map_or([0, 0, 0], |c| c.to_rgb8())));
            }
            let Effect::Blend { end_id: Some(next), .. } = e else {
                break;
            };
            if app.blend_of(next).is_none() {
                break;
            }
            let m = app.doc().find_shape(next).expect("middle");
            out.push((m.bounds().center(), m.fill.preview_color().map_or([0, 0, 0], |c| c.to_rgb8())));
            at = next;
        }
        out
    }

    fn same(x: &[(Point, [u8; 3])], y: &[(Point, [u8; 3])]) {
        assert_eq!(x.len(), y.len(), "{x:?}\n{y:?}");
        for (p, q) in x.iter().zip(y) {
            assert!((p.0 - q.0).hypot() < 0.01, "{x:?}\n{y:?}");
            for c in 0..3 {
                assert!((p.1[c] as i32 - q.1[c] as i32).abs() <= 1, "{x:?}\n{y:?}");
            }
        }
    }

    /// Split makes the chosen step an object that ends the first part and
    /// starts the second; every step stays where it was; one undo step.
    #[test]
    fn split_makes_a_compound_blend_with_the_steps_in_place() {
        let (mut app, a, b) = blended(5);
        let before = chain(&app, a);
        assert_eq!(before.len(), 5);
        assert!(app.can_split_blend());
        // A click on the third step picks it.
        assert_eq!(app.blend_step_at(before[2].0), Some(2));
        assert_eq!(app.blend_step_at(Point::new(500.0, 500.0)), None);
        let mid = app.split_blend(2).expect("split");
        app.refresh_blends();
        assert_eq!(app.selection, vec![mid]);
        let first = app.blend_of(a).expect("first part");
        let second = app.blend_of(mid).expect("second part");
        assert!(matches!(first, Effect::Blend { end_id: Some(x), steps: 2, .. } if x == mid));
        assert!(matches!(second, Effect::Blend { end_id: Some(x), steps: 2, .. } if x == b));
        // The turn stays with the second part.
        if let (Effect::Blend { rotation: r1, .. }, Effect::Blend { rotation: r2, .. }) = (&first, &second) {
            assert!(r1.abs() < 1e-9 && (r2 - 30.0).abs() < 1e-9, "{r1} {r2}");
        }
        same(&chain(&app, a), &before);
        // The new object lies between the two in the stacking order.
        assert!(app.stacked_below(a, mid) && app.stacked_below(mid, b));
        app.undo();
        app.refresh_blends();
        assert!(app.doc().find_shape(mid).is_none());
        assert!(matches!(app.blend_of(a), Some(Effect::Blend { steps: 5, end_id: Some(x), .. }) if x == b));
    }

    /// A split at the first step leaves the start with no blend.
    #[test]
    fn a_split_at_the_first_step_leaves_no_first_part() {
        let (mut app, a, b) = blended(3);
        let mid = app.split_blend(0).expect("split");
        assert!(app.blend_of(a).is_none());
        assert!(matches!(app.blend_of(mid), Some(Effect::Blend { steps: 2, end_id: Some(x), .. }) if x == b));
    }

    /// Fuse End and Fuse Start join the two parts again, the object
    /// between them gone and the steps back as they were.
    #[test]
    fn fusing_joins_the_parts_again() {
        for at_end in [true, false] {
            let (mut app, a, b) = blended(5);
            let before = chain(&app, a);
            assert!(!app.can_fuse_blend(at_end));
            let mid = app.split_blend(2).expect("split");
            app.refresh_blends();
            assert!(app.can_fuse_blend(at_end), "{at_end}");
            // Fuse End from the first part's start, Fuse Start from the
            // object between.
            app.select(vec![if at_end { a } else { mid }]);
            assert!(app.fuse_blend(at_end));
            app.refresh_blends();
            assert!(app.doc().find_shape(mid).is_none());
            assert_eq!(app.selection, vec![a]);
            assert!(matches!(app.blend_of(a), Some(Effect::Blend { steps: 5, end_id: Some(x), rotation, .. }) if x == b && (rotation - 30.0).abs() < 1e-9));
            same(&chain(&app, a), &before);
            app.undo();
            app.refresh_blends();
            assert!(app.doc().find_shape(mid).is_some());
        }
    }

    /// Map Nodes pairs the nodes clicked: the start's top-left corner with
    /// the end's bottom-right one; the steps then twist.
    #[test]
    fn map_nodes_pairs_the_nodes_clicked() {
        let (mut app, a, b) = blended(3);
        app.select(vec![a]);
        let plain: Vec<_> = chain(&app, a);
        assert!(app.map_blend_nodes(Point::new(0.2, 0.3), Point::new(109.0, 9.6)));
        app.refresh_blends();
        let Some(Effect::Blend { options, .. }) = app.blend_of(a) else {
            panic!("blend");
        };
        let m = options.map_nodes.expect("nodes");
        let sa = app.doc().find_shape(a).expect("a").clone();
        let sb = app.doc().find_shape(b).expect("b").clone();
        assert!((sa.transform * m.start[0] - Point::new(0.0, 0.0)).hypot() < 1e-9, "{m:?}");
        assert!((sb.transform * m.end[0] - Point::new(110.0, 10.0)).hypot() < 1e-9, "{m:?}");
        // The next node of each runs along its outline: a corner next to it.
        assert!(((sa.transform * m.start[1]) - (sa.transform * m.start[0])).hypot() > 9.0);
        // The steps are no longer the plain ones (their outlines turn).
        let paths = |app: &App| {
            let s = app.doc().find_shape(a).expect("a").clone();
            let e = app.blend_of(a).expect("blend");
            steps_of(&s, &e).iter().map(|x| x.page_path()).collect::<Vec<_>>()
        };
        let mapped = paths(&app);
        app.undo();
        app.refresh_blends();
        assert_ne!(paths(&app), mapped);
        assert_eq!(chain(&app, a).len(), plain.len());
    }

    /// The clicks: Split waits for a step, Map Nodes for two nodes.
    #[test]
    fn the_clicks_split_and_map() {
        let (mut app, a, _) = blended(3);
        let steps = chain(&app, a);
        app.pending_blend_split = true;
        app.blend_compound_click(steps[1].0);
        assert!(!app.pending_blend_split);
        assert!(app.blend_of(a).is_some_and(|e| matches!(e, Effect::Blend { steps: 1, .. })));
        let (mut app, a, _) = blended(3);
        app.select(vec![a]);
        app.pending_map_nodes = Some(None);
        app.blend_compound_click(Point::new(0.0, 10.0));
        assert_eq!(app.pending_map_nodes, Some(Some(Point::new(0.0, 10.0))));
        app.blend_compound_click(Point::new(100.0, 0.0));
        assert_eq!(app.pending_map_nodes, None);
        assert!(matches!(app.blend_of(a), Some(Effect::Blend { options, .. }) if options.map_nodes.is_some()));
    }

    #[test]
    fn the_nodes_run_on_round_a_closed_outline() {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((10.0, 0.0));
        p.line_to((10.0, 10.0));
        p.line_to((0.0, 0.0));
        p.close_path();
        let n = nodes_with_next(&p);
        assert_eq!(n.len(), 3, "{n:?}");
        assert_eq!(n[2], (Point::new(10.0, 10.0), Point::new(0.0, 0.0)));
        let mut open = BezPath::new();
        open.move_to((0.0, 0.0));
        open.line_to((5.0, 0.0));
        assert_eq!(nodes_with_next(&open), vec![(Point::new(0.0, 0.0), Point::new(5.0, 0.0))]);
    }
}
