//! Drop shadow geometry and settings: flat and perspective shadows, how
//! the feathered edge is shaped, the on-canvas handles and the built-in
//! presets. The shadow itself is [`crate::document::Shadow`]; the
//! renderer draws it with the formulas here (see
//! `docs/behavior/drop-shadow.md`).

use crate::document::Shadow;
use crate::geometry::{Affine, Point, Rect, Vec2};
use crate::live::MergeMode;
use serde::{Deserialize, Serialize};

/// The edge of the object a perspective shadow stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShadowEdge {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}

impl ShadowEdge {
    pub const ALL: [ShadowEdge; 4] = [
        ShadowEdge::Bottom,
        ShadowEdge::Top,
        ShadowEdge::Left,
        ShadowEdge::Right,
    ];
}

/// A perspective shadow: the object laid flat from one of its edges.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Perspective {
    pub edge: ShadowEdge,
    /// Direction the shadow falls in from the edge, degrees counter-
    /// clockwise from +x.
    pub angle: f64,
    /// Length of the shadow over the object's extent across the edge
    /// (its height for the top and bottom edges, its width for the
    /// others): 1 is as long as the object.
    pub stretch: f64,
    /// How much the shadow fades towards its far end, 0 (not at all) to 1
    /// (gone at the end).
    pub fade: f64,
}

impl Perspective {
    /// The perspective shadow a drag from `edge` starts with.
    pub fn new(edge: ShadowEdge) -> Self {
        let angle = match edge {
            ShadowEdge::Bottom => 20.0,
            ShadowEdge::Top => -20.0,
            ShadowEdge::Left => 160.0,
            ShadowEdge::Right => 20.0,
        };
        Perspective {
            edge,
            angle,
            stretch: 0.6,
            fade: 0.5,
        }
    }
}

/// Which side of the object's edge the feathering spreads to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeatherDirection {
    /// Across the edge, half inside and half outside (a soft blur).
    #[default]
    Middle,
    /// Only inside the edge: the shadow keeps its outline and fades in.
    Inside,
    /// Only outside the edge: the shadow is solid up to the outline and
    /// fades beyond it.
    Outside,
    /// The colours averaged over a square the size of the feathering (one
    /// even box pass): a straight, even ramp across the edge.
    Average,
}

impl FeatherDirection {
    pub const ALL: [FeatherDirection; 4] = [
        FeatherDirection::Inside,
        FeatherDirection::Middle,
        FeatherDirection::Outside,
        FeatherDirection::Average,
    ];
}

/// The shape of the feathered ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeatherEdges {
    /// Opacity follows the ramp evenly.
    #[default]
    Linear,
    /// Falls off quickly: the ramp value squared.
    Squared,
    /// Stays dense and falls off late: `1 - (1 - t)^2`.
    InvertedSquared,
    /// No ramp: the whole feathered area is solid.
    Flat,
}

impl FeatherEdges {
    pub const ALL: [FeatherEdges; 4] = [
        FeatherEdges::Linear,
        FeatherEdges::Squared,
        FeatherEdges::InvertedSquared,
        FeatherEdges::Flat,
    ];
}

/// The anchor, inward unit normal and extent of an edge of `b`.
fn edge_frame(b: Rect, edge: ShadowEdge) -> (Point, Vec2, f64) {
    let c = b.center();
    match edge {
        ShadowEdge::Bottom => (Point::new(c.x, b.y0), Vec2::new(0.0, 1.0), b.height()),
        ShadowEdge::Top => (Point::new(c.x, b.y1), Vec2::new(0.0, -1.0), b.height()),
        ShadowEdge::Left => (Point::new(b.x0, c.y), Vec2::new(1.0, 0.0), b.width()),
        ShadowEdge::Right => (Point::new(b.x1, c.y), Vec2::new(-1.0, 0.0), b.width()),
    }
}

impl Shadow {
    /// Where the shadow starts (its start handle) for an object with page
    /// bounds `b`: the centre for a flat shadow, the middle of the edge
    /// for a perspective one.
    pub fn anchor(&self, b: Rect) -> Point {
        match &self.perspective {
            None => b.center(),
            Some(p) => edge_frame(b, p.edge).0,
        }
    }

    /// Where the shadow goes to (its end handle): the anchor moved by the
    /// offset (flat), or the point the far side of the object falls on
    /// (perspective).
    pub fn end_point(&self, b: Rect) -> Point {
        match &self.perspective {
            None => b.center() + self.offset,
            Some(p) => {
                let (a, _, len) = edge_frame(b, p.edge);
                let r = p.angle.to_radians();
                let d = finite_or(p.stretch, 0.0) * len.max(0.0);
                a + Vec2::new(r.cos(), r.sin()) * d
            }
        }
    }

    /// Move the end handle to `p`: the offset of a flat shadow, the angle
    /// and stretch of a perspective one.
    pub fn set_end_point(&mut self, b: Rect, p: Point) {
        if !(p.x.is_finite() && p.y.is_finite()) {
            return;
        }
        match self.perspective.as_mut() {
            None => self.offset = p - b.center(),
            Some(per) => {
                let (a, _, len) = edge_frame(b, per.edge);
                let d = p - a;
                if d.hypot() > 1e-9 {
                    per.angle = d.atan2().to_degrees();
                }
                if len > 1e-9 {
                    per.stretch = (d.hypot() / len).clamp(0.0, 10.0);
                }
            }
        }
    }

    /// The page-space transform that takes the object's silhouette (page
    /// bounds `b`) to the shadow's: a move by the offset, or for a
    /// perspective shadow the shear that keeps the edge in place and puts
    /// the opposite side on the end point. With anchor `A`, inward normal
    /// `n`, extent `L` and `D = end - A`, a point `p` goes to
    /// `p + s (D - L n)` where `s = (p - A) . n / L`.
    pub fn silhouette_transform(&self, b: Rect) -> Affine {
        let Some(per) = &self.perspective else {
            return Affine::translate(finite_vec(self.offset));
        };
        let (a, n, len) = edge_frame(b, per.edge);
        if len <= 1e-9 || !len.is_finite() {
            return Affine::IDENTITY;
        }
        let d = self.end_point(b) - a;
        let v = d - n * len;
        let an = a.x * n.x + a.y * n.y;
        Affine::new([
            1.0 + v.x * n.x / len,
            v.y * n.x / len,
            v.x * n.y / len,
            1.0 + v.y * n.y / len,
            -an / len * v.x,
            -an / len * v.y,
        ])
    }

    /// For a perspective shadow that fades: from where (full opacity) to
    /// where (opacity times `1 - fade`) the fade runs, page space.
    pub fn fade_axis(&self, b: Rect) -> Option<(Point, Point, f64)> {
        let per = self.perspective.as_ref()?;
        let fade = finite_or(per.fade, 0.0).clamp(0.0, 1.0);
        if fade <= 0.0 {
            return None;
        }
        Some((self.anchor(b), self.end_point(b), fade))
    }

    /// The final coverage (0 to 1) of a pixel whose blurred coverage is
    /// `b`, after the feather direction and edge shape.
    pub fn feather_value(&self, b: f64) -> f64 {
        let b = finite_or(b, 0.0).clamp(0.0, 1.0);
        let t = match self.feather_direction {
            FeatherDirection::Middle | FeatherDirection::Average => b,
            FeatherDirection::Outside => (2.0 * b).min(1.0),
            FeatherDirection::Inside => (2.0 * b - 1.0).max(0.0),
        };
        match self.feather_edges {
            FeatherEdges::Linear => t,
            FeatherEdges::Squared => t * t,
            FeatherEdges::InvertedSquared => 1.0 - (1.0 - t) * (1.0 - t),
            FeatherEdges::Flat => {
                if t > 0.01 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    /// The bounds the shadow covers for an object with page path bounds
    /// `b`: the moved or sheared box grown by the feathering.
    pub fn bounds(&self, b: Rect) -> Rect {
        let t = self.silhouette_transform(b);
        let corners = [
            Point::new(b.x0, b.y0),
            Point::new(b.x1, b.y0),
            Point::new(b.x0, b.y1),
            Point::new(b.x1, b.y1),
        ]
        .map(|p| t * p);
        let mut r = Rect::from_points(corners[0], corners[1]);
        for c in &corners[2..] {
            r = r.union_pt(*c);
        }
        let pad = finite_or(self.blur, 0.0).max(0.0) * 3.0;
        r.inflate(pad, pad)
    }

    /// The shadow a drag starting at `p` on an object with bounds `b`
    /// begins with: a perspective shadow when `p` is in the outer fifth
    /// of the object next to an edge (the top and bottom edges first), a
    /// flat one otherwise. `base` gives the colour, opacity, feathering
    /// and merge mode.
    pub fn for_drag_start(base: &Shadow, b: Rect, p: Point) -> Shadow {
        let mut sh = *base;
        let edge = edge_near(b, p);
        sh.perspective = edge.map(|e| {
            let mut per = base
                .perspective
                .filter(|q| q.edge == e)
                .unwrap_or_else(|| Perspective::new(e));
            per.edge = e;
            per
        });
        sh
    }

    /// The built-in presets: a stable id (the interface translates it) and
    /// the shadow.
    pub fn presets() -> Vec<(&'static str, Shadow)> {
        let base = Shadow::default();
        let flat = |dx: f64, dy: f64, opacity: f64, blur: f64| Shadow {
            offset: Vec2::new(dx, dy),
            opacity,
            blur,
            ..base
        };
        let persp = |edge: ShadowEdge, angle: f64, stretch: f64, fade: f64| Shadow {
            opacity: 0.6,
            blur: 1.0,
            perspective: Some(Perspective {
                edge,
                angle,
                stretch,
                fade,
            }),
            ..base
        };
        vec![
            ("soft", flat(2.0, -2.0, 0.5, 1.5)),
            ("tight", flat(0.7, -0.7, 0.7, 0.4)),
            ("lifted", flat(4.0, -5.0, 0.35, 4.0)),
            (
                "glow",
                Shadow {
                    offset: Vec2::ZERO,
                    opacity: 0.8,
                    blur: 3.0,
                    feather_direction: FeatherDirection::Outside,
                    merge: MergeMode::Normal,
                    ..base
                },
            ),
            ("floor_right", persp(ShadowEdge::Bottom, 25.0, 0.6, 0.6)),
            ("floor_left", persp(ShadowEdge::Bottom, 155.0, 0.6, 0.6)),
            ("wall_behind", persp(ShadowEdge::Top, 70.0, 0.4, 0.4)),
        ]
    }
}

/// The edge `p` lies next to, if any (within a fifth of the size).
pub fn edge_near(b: Rect, p: Point) -> Option<ShadowEdge> {
    let (w, h) = (b.width(), b.height());
    if !(w > 1e-9 && h > 1e-9) {
        return None;
    }
    let fy = (p.y - b.y0) / h;
    let fx = (p.x - b.x0) / w;
    if fy <= 0.2 {
        Some(ShadowEdge::Bottom)
    } else if fy >= 0.8 {
        Some(ShadowEdge::Top)
    } else if fx <= 0.2 {
        Some(ShadowEdge::Left)
    } else if fx >= 0.8 {
        Some(ShadowEdge::Right)
    } else {
        None
    }
}

fn finite_or(v: f64, d: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        d
    }
}

fn finite_vec(v: Vec2) -> Vec2 {
    Vec2::new(finite_or(v.x, 0.0), finite_or(v.y, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect::new(10.0, 0.0, 30.0, 40.0)
    }

    #[test]
    fn a_bottom_perspective_shadow_keeps_the_base_and_lays_the_top_on_the_end_point() {
        let b = rect();
        let mut sh = Shadow::for_drag_start(&Shadow::default(), b, Point::new(20.0, 2.0));
        assert_eq!(sh.perspective.map(|p| p.edge), Some(ShadowEdge::Bottom));
        sh.set_end_point(b, Point::new(60.0, 10.0));
        let t = sh.silhouette_transform(b);
        // The base stays put.
        for x in [10.0, 20.0, 30.0] {
            let q = t * Point::new(x, 0.0);
            assert!((q - Point::new(x, 0.0)).hypot() < 1e-9, "{q:?}");
        }
        // The top centre lands on the end point, the top corners beside it.
        let top = t * Point::new(20.0, 40.0);
        assert!((top - Point::new(60.0, 10.0)).hypot() < 1e-9, "{top:?}");
        let tl = t * Point::new(10.0, 40.0);
        assert!((tl - Point::new(50.0, 10.0)).hypot() < 1e-9, "{tl:?}");
        // Halfway up goes halfway.
        let mid = t * Point::new(20.0, 20.0);
        assert!((mid - Point::new(40.0, 5.0)).hypot() < 1e-9, "{mid:?}");
        // Angle and stretch follow the end point.
        let per = sh.perspective.expect("perspective");
        assert!((per.angle - (10.0f64).atan2(40.0).to_degrees()).abs() < 1e-9);
        assert!((per.stretch - (40.0f64.hypot(10.0) / 40.0)).abs() < 1e-9);
        assert!((sh.end_point(b) - Point::new(60.0, 10.0)).hypot() < 1e-9);
    }

    #[test]
    fn a_top_perspective_shadow_hangs_from_the_top_edge() {
        let b = rect();
        let mut sh = Shadow::for_drag_start(&Shadow::default(), b, Point::new(20.0, 39.0));
        assert_eq!(sh.perspective.map(|p| p.edge), Some(ShadowEdge::Top));
        sh.set_end_point(b, Point::new(35.0, 60.0));
        let t = sh.silhouette_transform(b);
        let q = t * Point::new(30.0, 40.0);
        assert!((q - Point::new(30.0, 40.0)).hypot() < 1e-9);
        let bottom = t * Point::new(20.0, 0.0);
        assert!(
            (bottom - Point::new(35.0, 60.0)).hypot() < 1e-9,
            "{bottom:?}"
        );
        let (a, e, fade) = sh.fade_axis(b).expect("fades");
        assert_eq!((a, e), (Point::new(20.0, 40.0), Point::new(35.0, 60.0)));
        assert!(fade > 0.0);
    }

    #[test]
    fn a_drag_from_the_middle_gives_a_flat_shadow_moved_by_the_offset() {
        let b = rect();
        let mut sh = Shadow::for_drag_start(&Shadow::default(), b, Point::new(20.0, 20.0));
        assert!(sh.perspective.is_none());
        sh.set_end_point(b, Point::new(23.0, 17.0));
        assert_eq!(sh.offset, Vec2::new(3.0, -3.0));
        assert_eq!(
            sh.silhouette_transform(b) * Point::new(0.0, 0.0),
            Point::new(3.0, -3.0)
        );
        assert!(sh.fade_axis(b).is_none());
        assert_eq!(sh.anchor(b), b.center());
    }

    #[test]
    fn feather_directions_and_edges_shape_the_ramp() {
        let mut sh = Shadow::default();
        assert_eq!(sh.feather_value(0.5), 0.5);
        sh.feather_direction = FeatherDirection::Outside;
        assert_eq!(sh.feather_value(0.5), 1.0);
        assert_eq!(sh.feather_value(0.25), 0.5);
        sh.feather_direction = FeatherDirection::Inside;
        assert_eq!(sh.feather_value(0.5), 0.0);
        assert_eq!(sh.feather_value(0.75), 0.5);
        sh.feather_direction = FeatherDirection::Middle;
        sh.feather_edges = FeatherEdges::Squared;
        assert_eq!(sh.feather_value(0.5), 0.25);
        sh.feather_edges = FeatherEdges::InvertedSquared;
        assert_eq!(sh.feather_value(0.5), 0.75);
        sh.feather_edges = FeatherEdges::Flat;
        assert_eq!(sh.feather_value(0.2), 1.0);
        assert_eq!(sh.feather_value(0.0), 0.0);
        assert_eq!(sh.feather_value(f64::NAN), 0.0);
    }

    #[test]
    fn old_shadows_load_flat_and_normal_and_new_ones_round_trip() {
        let old = r#"{"offset":{"x":2.0,"y":-2.0},"opacity":0.5,"blur":1.5,"color":{"model":"gray","v":0.0}}"#;
        let sh: Shadow = serde_json::from_str(old).expect("old shadow");
        assert_eq!(sh.merge, MergeMode::Normal);
        assert!(sh.perspective.is_none());
        assert_eq!(sh.feather_direction, FeatherDirection::Middle);
        assert_eq!(sh.feather_edges, FeatherEdges::Linear);
        // New shadows multiply.
        assert_eq!(Shadow::default().merge, MergeMode::Multiply);
        for (_, p) in Shadow::presets() {
            let json = serde_json::to_string(&p).expect("json");
            let back: Shadow = serde_json::from_str(&json).expect("back");
            assert_eq!(back, p);
        }
    }

    #[test]
    fn degenerate_boxes_and_values_do_not_break_the_geometry() {
        let flat = Rect::new(0.0, 5.0, 10.0, 5.0);
        let sh = Shadow::for_drag_start(&Shadow::default(), flat, Point::new(5.0, 5.0));
        assert!(sh.perspective.is_none());
        let mut per = Shadow {
            perspective: Some(Perspective::new(ShadowEdge::Bottom)),
            ..Shadow::default()
        };
        let t = per.silhouette_transform(flat);
        assert!(t.as_coeffs().iter().all(|c| c.is_finite()));
        per.set_end_point(rect(), Point::new(f64::NAN, 0.0));
        assert_eq!(per.perspective.map(|p| p.stretch), Some(0.6));
        let b = per.bounds(rect());
        assert!(b.width().is_finite() && b.height().is_finite());
    }
}
