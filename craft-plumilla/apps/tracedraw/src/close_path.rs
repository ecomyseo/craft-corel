//! Curve commands the reference keeps in its shape tool's menus: Close Path
//! in four ways (the closest ends or each subpath's start and end, with a
//! straight or a curved line), Reverse Subpaths for the subpaths holding
//! the chosen nodes only, and Swap Arrowheads. See docs/behavior/curves.md.

use crate::app::App;
use tracedraw_core::document::ShapeKind;
use tracedraw_core::geometry::{BezPath, PathEl, Point, Vec2};
use tracedraw_core::{nodes, Command, ShapeId};

/// How Close Path joins the open ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseMode {
    /// The open subpaths chained by their closest ends, straight lines.
    ClosestStraight,
    /// The same with curves that carry on the ends' directions.
    ClosestCurve,
    /// Each open subpath's end back to its start, a straight line.
    StartEndStraight,
    /// The same with a curve.
    StartEndCurve,
}

impl CloseMode {
    pub const ALL: [(CloseMode, &'static str); 4] = [
        (CloseMode::ClosestStraight, "context.close_closest_straight"),
        (CloseMode::ClosestCurve, "context.close_closest_curve"),
        (CloseMode::StartEndStraight, "context.close_start_end_straight"),
        (CloseMode::StartEndCurve, "context.close_start_end_curve"),
    ];

    fn curved(self) -> bool {
        matches!(self, CloseMode::ClosestCurve | CloseMode::StartEndCurve)
    }
}

/// The subpaths of `path`, each starting with its `MoveTo`.
fn subpaths(path: &BezPath) -> Vec<Vec<PathEl>> {
    let mut out: Vec<Vec<PathEl>> = Vec::new();
    for el in nodes::normalize(path).elements() {
        if matches!(el, PathEl::MoveTo(_)) || out.is_empty() {
            out.push(Vec::new());
        }
        out.last_mut().expect("a subpath").push(*el);
    }
    out
}

fn is_closed(sp: &[PathEl]) -> bool {
    matches!(sp.last(), Some(PathEl::ClosePath))
}

fn start(sp: &[PathEl]) -> Point {
    match sp.first() {
        Some(PathEl::MoveTo(p)) => *p,
        _ => Point::ZERO,
    }
}

fn end(sp: &[PathEl]) -> Point {
    match sp.last() {
        Some(PathEl::MoveTo(p) | PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p)) => *p,
        _ => start(sp),
    }
}

fn unit(v: Vec2) -> Vec2 {
    let l = v.hypot();
    if l > 1e-9 {
        v / l
    } else {
        Vec2::ZERO
    }
}

/// The direction the subpath leaves its start in.
fn start_dir(sp: &[PathEl]) -> Vec2 {
    let s = start(sp);
    match sp.get(1) {
        Some(PathEl::LineTo(p)) => unit(*p - s),
        Some(PathEl::QuadTo(c, _)) | Some(PathEl::CurveTo(c, _, _)) => unit(*c - s),
        _ => Vec2::ZERO,
    }
}

/// The direction the subpath arrives at its end in.
fn end_dir(sp: &[PathEl]) -> Vec2 {
    let n = sp.len();
    let before = |i: usize| -> Point {
        match sp.get(i) {
            Some(PathEl::MoveTo(p) | PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p)) => *p,
            _ => start(sp),
        }
    };
    match sp.last() {
        Some(PathEl::LineTo(p)) if n >= 2 => unit(*p - before(n - 2)),
        Some(PathEl::QuadTo(c, p)) | Some(PathEl::CurveTo(_, c, p)) => unit(*p - *c),
        _ => Vec2::ZERO,
    }
}

fn reversed(sp: &[PathEl]) -> Vec<PathEl> {
    nodes::reverse(&BezPath::from_vec(sp.to_vec())).elements().to_vec()
}

/// The segment from `a` (arriving in `da`) to `b` (leaving in `db`).
fn joint(a: Point, da: Vec2, b: Point, db: Vec2, curved: bool) -> PathEl {
    let k = (b - a).hypot() / 3.0;
    if curved && k > 1e-9 && (da != Vec2::ZERO || db != Vec2::ZERO) {
        PathEl::CurveTo(a + da * k, b - db * k, b)
    } else {
        PathEl::LineTo(b)
    }
}

/// `path` with its open subpaths closed as `mode` says; closed subpaths
/// stay as they are.
pub fn close_path(path: &BezPath, mode: CloseMode) -> BezPath {
    let sps = subpaths(path);
    let (closed, open): (Vec<_>, Vec<_>) = sps.into_iter().partition(|sp| is_closed(sp) || sp.len() < 2);
    if open.is_empty() {
        return path.clone();
    }
    let curved = mode.curved();
    let mut out: Vec<PathEl> = closed.into_iter().flatten().collect();
    let close = |sp: &mut Vec<PathEl>| {
        if curved {
            let j = joint(end(sp), end_dir(sp), start(sp), start_dir(sp), true);
            if !matches!(j, PathEl::LineTo(_)) {
                sp.push(j);
            }
        }
        sp.push(PathEl::ClosePath);
    };
    match mode {
        CloseMode::StartEndStraight | CloseMode::StartEndCurve => {
            for mut sp in open {
                close(&mut sp);
                out.extend(sp);
            }
        }
        CloseMode::ClosestStraight | CloseMode::ClosestCurve => {
            let mut rest = open;
            let mut chain = rest.remove(0);
            while !rest.is_empty() {
                let e = end(&chain);
                let (i, flip) = (0..rest.len())
                    .flat_map(|i| [(i, false), (i, true)])
                    .min_by(|x, y| {
                        let d = |(i, flip): (usize, bool)| {
                            let p = if flip { end(&rest[i]) } else { start(&rest[i]) };
                            (p - e).hypot()
                        };
                        d(*x).total_cmp(&d(*y))
                    })
                    .expect("a subpath left");
                let next = rest.remove(i);
                let next = if flip { reversed(&next) } else { next };
                chain.push(joint(e, end_dir(&chain), start(&next), start_dir(&next), curved));
                chain.extend(next.into_iter().skip(1));
            }
            close(&mut chain);
            out.extend(chain);
        }
    }
    BezPath::from_vec(out)
}

/// `path` with the subpaths that hold any of the element indices
/// `chosen` reversed, the others as they are.
pub fn reverse_subpaths(path: &BezPath, chosen: &[usize]) -> BezPath {
    let mut out = Vec::new();
    let mut at = 0;
    for sp in subpaths(path) {
        let range = at..at + sp.len();
        at += sp.len();
        if chosen.iter().any(|i| range.contains(i)) {
            out.extend(reversed(&sp));
        } else {
            out.extend(sp);
        }
    }
    BezPath::from_vec(out)
}

impl App {
    /// Close Path on the selected curves (or the curves of the chosen
    /// nodes): one undo step.
    pub fn close_selected_paths(&mut self, mode: CloseMode) {
        let ids: Vec<ShapeId> = if self.node_selection.is_empty() {
            self.selection.clone()
        } else {
            let mut v: Vec<ShapeId> = self.node_selection.iter().map(|(id, _)| *id).collect();
            v.dedup();
            v
        };
        let mut cmds = Vec::new();
        for id in ids {
            let Some(s) = self.doc().find_shape(id) else {
                continue;
            };
            let ShapeKind::Path { path, .. } = &s.kind else {
                continue;
            };
            let np = close_path(path, mode);
            if np.elements() != path.elements() {
                cmds.push(Command::SetShapeKind {
                    shape: id,
                    kind: ShapeKind::Path { path: np, closed: true },
                });
            }
        }
        if !cmds.is_empty() {
            let _ = self.engine.run_batch("Close Path", &cmds);
        }
    }

    /// Reverse Subpaths: only the subpaths holding chosen nodes turn round.
    pub fn reverse_selected_subpaths(&mut self) {
        let mut by_shape: Vec<(ShapeId, Vec<usize>)> = Vec::new();
        for (id, i) in self.node_selection.iter().copied() {
            match by_shape.iter_mut().find(|(s, _)| *s == id) {
                Some((_, v)) => v.push(i),
                None => by_shape.push((id, vec![i])),
            }
        }
        let mut cmds = Vec::new();
        for (id, chosen) in by_shape {
            let Some(s) = self.doc().find_shape(id) else {
                continue;
            };
            let ShapeKind::Path { path, closed } = &s.kind else {
                continue;
            };
            cmds.push(Command::SetShapeKind {
                shape: id,
                kind: ShapeKind::Path { path: reverse_subpaths(path, &chosen), closed: *closed },
            });
        }
        if !cmds.is_empty() {
            let _ = self.engine.run_batch("Reverse Subpaths", &cmds);
            self.node_selection.clear();
        }
    }

    /// Swap Arrowheads: each selected outline's start and end arrowheads
    /// change places (one undo step).
    pub fn swap_arrowheads(&mut self) {
        let cmds: Vec<Command> = self
            .paint_targets()
            .iter()
            .filter_map(|s| {
                let mut st = s.stroke.clone()?;
                if st.start_arrow == st.end_arrow {
                    return None;
                }
                std::mem::swap(&mut st.start_arrow, &mut st.end_arrow);
                Some(Command::SetStroke { shapes: vec![s.id], stroke: Some(st) })
            })
            .collect();
        if !cmds.is_empty() {
            let _ = self.engine.run_batch("Swap Arrowheads", &cmds);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_open() -> BezPath {
        let mut p = BezPath::new();
        p.move_to((0.0, 0.0));
        p.line_to((10.0, 0.0));
        p.move_to((30.0, 0.0));
        p.line_to((12.0, 0.0));
        p
    }

    fn closes(p: &BezPath) -> usize {
        p.elements().iter().filter(|e| matches!(e, PathEl::ClosePath)).count()
    }

    /// Start and end: each open subpath closed on itself; the curved way
    /// adds a curve that carries on the ends' directions.
    #[test]
    fn start_and_end_close_each_subpath() {
        let p = two_open();
        let s = close_path(&p, CloseMode::StartEndStraight);
        assert_eq!(subpaths(&s).len(), 2);
        assert_eq!(closes(&s), 2);
        let mut arc = BezPath::new();
        arc.move_to((0.0, 0.0));
        arc.curve_to((0.0, 10.0), (10.0, 10.0), (10.0, 0.0));
        let c = close_path(&arc, CloseMode::StartEndCurve);
        assert_eq!(closes(&c), 1);
        assert_eq!(c.elements().iter().filter(|e| matches!(e, PathEl::CurveTo(..))).count(), 2);
        // A closed path stays as it is.
        assert_eq!(close_path(&s, CloseMode::StartEndCurve).elements(), s.elements());
    }

    /// Closest ends: the open subpaths become one closed subpath, the
    /// second joined by its nearer end (12, 0), turned round.
    #[test]
    fn closest_ends_chain_the_subpaths() {
        let c = close_path(&two_open(), CloseMode::ClosestStraight);
        let sps = subpaths(&c);
        assert_eq!(sps.len(), 1);
        assert_eq!(closes(&c), 1);
        let pts: Vec<Point> = c
            .elements()
            .iter()
            .filter_map(|e| match e {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
                _ => None,
            })
            .collect();
        assert_eq!(pts, vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0), Point::new(12.0, 0.0), Point::new(30.0, 0.0)]);
        let curved = close_path(&two_open(), CloseMode::ClosestCurve);
        assert_eq!(subpaths(&curved).len(), 1);
    }

    /// On the drawing: Close Path on a selected curve (one undo step) and
    /// Swap Arrowheads on its outline.
    #[test]
    fn the_app_closes_paths_and_swaps_arrowheads() {
        let mut app = App::headless();
        let id = app.new_shape(ShapeKind::Path { path: two_open(), closed: false }).expect("curve");
        app.select(vec![id]);
        app.close_selected_paths(CloseMode::ClosestStraight);
        let path_of = |app: &App| match &app.doc().find_shape(id).expect("curve").kind {
            ShapeKind::Path { path, .. } => path.clone(),
            _ => panic!("not a curve"),
        };
        assert_eq!(subpaths(&path_of(&app)).len(), 1);
        assert_eq!(closes(&path_of(&app)), 1);
        app.undo();
        assert_eq!(closes(&path_of(&app)), 0);
        // Arrowheads change places.
        let arrow = tracedraw_core::Arrowhead::ALL[1].clone();
        let mut st = app.doc().find_shape(id).expect("curve").stroke.clone().expect("outline");
        st.start_arrow = arrow.clone();
        st.end_arrow = tracedraw_core::Arrowhead::None;
        app.run(Command::SetStroke { shapes: vec![id], stroke: Some(st) });
        app.swap_arrowheads();
        let st = app.doc().find_shape(id).expect("curve").stroke.clone().expect("outline");
        assert_eq!(st.start_arrow, tracedraw_core::Arrowhead::None);
        assert_eq!(st.end_arrow, arrow);
    }

    /// Insert Spread Before / After: with facing pages, two pages before the
    /// current spread or after it, in one undo step; without, one page.
    #[test]
    fn spreads_are_inserted_two_pages_at_a_time() {
        let mut app = App::headless();
        for _ in 0..3 {
            app.insert_page(true);
        }
        assert_eq!(app.doc().pages.len(), 4);
        let mut md = app.doc().metadata.clone();
        md.facing_pages = true;
        app.run(Command::SetMetadata { metadata: md });
        // Pages 1 | 2 3 | 4: the current page 3 is in the spread 2-3.
        app.goto_page(2);
        let ids: Vec<_> = app.doc().pages.iter().map(|p| p.id).collect();
        app.insert_spread(true);
        let now: Vec<_> = app.doc().pages.iter().map(|p| p.id).collect();
        assert_eq!(now.len(), 6);
        assert_eq!(now[..3], ids[..3], "the spread 2-3 stays in front");
        assert_eq!(now[5], ids[3], "page 4 after the two new ones");
        assert_eq!(app.page, now[3], "the first new page is current");
        app.undo();
        assert_eq!(app.doc().pages.len(), 4, "one undo step");
        app.goto_page(2);
        app.insert_spread(false);
        let now: Vec<_> = app.doc().pages.iter().map(|p| p.id).collect();
        assert_eq!(now[0], ids[0]);
        assert_eq!(now[3..5], ids[1..3], "the spread 2-3 moves after the new pages");
    }

    /// Internet Links > Launch Browser, in the object's right-click menu,
    /// opens the selected object's link; greyed when it has none.
    #[test]
    fn the_link_opens_in_the_browser() {
        crate::i18n::set_language("en");
        let ctx = crate::theme::ui_context();
        let make = |link: Option<&str>| {
            let mut app = App::headless();
            let id = app.new_shape(ShapeKind::Path { path: two_open(), closed: false }).expect("curve");
            let mut s = app.doc().find_shape(id).expect("s").clone();
            s.link = link.map(str::to_string);
            app.run(Command::DeleteShapes { shapes: vec![id] });
            let layer = app.active_layer().expect("layer");
            app.run(Command::AddShape { layer, shape: s });
            app.select(vec![id]);
            app
        };
        let mut found = None;
        let mut row = 0;
        loop {
            let mut app = make(Some("https://example.org/a"));
            let mut close = false;
            let before = crate::export::OPENED_IN_TESTS.with(|o| o.borrow().len());
            crate::ui::menus::replay::start(row);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                crate::ui::context::object_menu(&mut app, ui, &mut close)
            });
            out.textures_delta.clear();
            let (hit, rows) = crate::ui::menus::replay::finish();
            if hit.as_deref() == Some("context.launch_browser") {
                let opened = crate::export::OPENED_IN_TESTS.with(|o| o.borrow()[before..].to_vec());
                assert_eq!(opened, ["https://example.org/a"]);
                found = Some(row);
                break;
            }
            row += 1;
            assert!(row < rows, "Launch Browser not offered");
        }
        // Without a link the row is there but greyed: nothing opens.
        let mut app = make(None);
        let mut close = false;
        crate::ui::menus::replay::start(found.expect("row"));
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            crate::ui::context::object_menu(&mut app, ui, &mut close)
        });
        out.textures_delta.clear();
        assert_ne!(crate::ui::menus::replay::finish().0.as_deref(), Some("context.launch_browser"));
    }

    /// Only the subpath holding a chosen node turns round.
    #[test]
    fn reverse_subpaths_turns_only_the_chosen() {
        let p = two_open();
        let r = reverse_subpaths(&p, &[3]);
        let sps = subpaths(&r);
        assert_eq!(start(&sps[0]), Point::new(0.0, 0.0), "the first stays");
        assert_eq!(start(&sps[1]), Point::new(12.0, 0.0), "the second turned round");
        assert_eq!(reverse_subpaths(&p, &[]).elements(), nodes::normalize(&p).elements());
    }
}
