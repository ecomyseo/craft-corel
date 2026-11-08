//! The Craft Corel look shared by the About box and the splash screen: a
//! deep blue gradient, faint light blue curves with selected nodes (a path
//! being edited), the application icon and the name in two colours.

use egui::epaint::CubicBezierShape;
use egui::{Color32, Context, CornerRadius, Painter, Pos2, Rect, Stroke, TextureHandle, Vec2};
use std::sync::Arc;

pub const TOP_LEFT: Color32 = Color32::from_rgb(0x24, 0x44, 0x75);
pub const TOP_RIGHT: Color32 = Color32::from_rgb(0x1A, 0x31, 0x58);
pub const BOTTOM: Color32 = Color32::from_rgb(0x0E, 0x19, 0x2D);
pub const CURVE: Color32 = Color32::from_rgb(0x5C, 0xAE, 0xFF);
pub const NAME_ACCENT: Color32 = Color32::from_rgb(0x7F, 0xC3, 0xFF);
pub const TAGLINE: Color32 = Color32::from_rgb(0xC9, 0xD7, 0xEC);
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub const FOOTNOTE: Color32 = Color32::from_rgb(0x8F, 0xA3, 0xC0);

/// The 128 px application icon, decoded once per session.
pub fn icon(ctx: &Context) -> Option<TextureHandle> {
    let id = egui::Id::new("brand_icon_texture");
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return Some(t);
    }
    let color = icon_image()?;
    let tex = ctx.load_texture("brand_icon", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    Some(tex)
}

/// The embedded 128 px icon as an egui image.
pub(crate) fn icon_image() -> Option<egui::ColorImage> {
    let bytes = include_bytes!("../../../../assets/craft-corel/icon-128.png");
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [w, h],
        img.as_raw(),
    ))
}

/// Paint the icon texture into `rect`.
pub fn paint_icon(painter: &Painter, icon: Option<&TextureHandle>, rect: Rect) {
    if let Some(tex) = icon {
        painter.image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}

/// The point at `t` (0 to 1) on the cubic Bezier `p`.
pub(crate) fn cubic_point(p: &[Pos2; 4], t: f32) -> Pos2 {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    Pos2::new(
        a * p[0].x + b * p[1].x + c * p[2].x + d * p[3].x,
        a * p[0].y + b * p[1].y + c * p[2].y + d * p[3].y,
    )
}

/// The tangent direction at `t` on the cubic Bezier `p` (not normalised).
pub(crate) fn cubic_tangent(p: &[Pos2; 4], t: f32) -> Vec2 {
    let u = 1.0 - t;
    (p[1] - p[0]) * (3.0 * u * u) + (p[2] - p[1]) * (6.0 * u * t) + (p[3] - p[2]) * (3.0 * t * t)
}

/// Fill `rect` with the diagonal gradient.
pub fn gradient(painter: &Painter, rect: Rect) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), TOP_LEFT);
    mesh.colored_vertex(rect.right_top(), TOP_RIGHT);
    mesh.colored_vertex(rect.right_bottom(), BOTTOM);
    mesh.colored_vertex(rect.left_bottom(), BOTTOM);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
}

/// A selected node on `curve` at `t`: the square node, its two handles
/// along the tangent and their round ends, all `scale` times the About
/// box's size.
fn node(painter: &Painter, curve: &[Pos2; 4], t: f32, scale: f32) {
    let at = cubic_point(curve, t);
    let dir = cubic_tangent(curve, t).normalized() * 30.0 * scale;
    let handle = Stroke::new(1.0, Color32::from_white_alpha(110));
    for end in [at - dir, at + dir] {
        painter.line_segment([at, end], handle);
        painter.circle(end, 3.0 * scale, CURVE, Stroke::new(1.0, Color32::WHITE));
    }
    let square = Rect::from_center_size(at, Vec2::splat(8.0 * scale));
    painter.rect(
        square,
        CornerRadius::ZERO,
        TOP_RIGHT,
        Stroke::new(1.3, Color32::WHITE),
        egui::StrokeKind::Inside,
    );
}

/// The main curve across the right part of `r`.
pub(crate) fn main_curve(r: Rect) -> [Pos2; 4] {
    let at = |x: f32, y: f32| Pos2::new(r.left() + x * r.width(), r.top() + y * r.height());
    [
        at(0.60, 1.08),
        at(0.73, 0.02),
        at(0.86, 1.12),
        at(1.03, 0.18),
    ]
}

/// The faint curves on the right of `r`: two echoes and the main curve
/// with a selected node at each of `nodes` (curve parameters 0 to 1).
/// `scale` sizes the strokes and nodes.
pub fn curves(painter: &Painter, r: Rect, scale: f32, nodes: &[f32]) {
    let at = |x: f32, y: f32| Pos2::new(r.left() + x * r.width(), r.top() + y * r.height());
    let echoes = [
        (
            [
                at(0.68, 1.08),
                at(0.79, 0.30),
                at(0.92, 1.15),
                at(1.03, 0.48),
            ],
            60,
        ),
        (
            [
                at(0.54, -0.08),
                at(0.70, 0.62),
                at(0.83, -0.25),
                at(1.03, -0.02),
            ],
            40,
        ),
    ];
    for (points, alpha) in echoes {
        painter.add(CubicBezierShape::from_points_stroke(
            points,
            false,
            Color32::TRANSPARENT,
            Stroke::new(1.2 * scale, CURVE.gamma_multiply_u8(alpha)),
        ));
    }
    let main = main_curve(r);
    painter.add(CubicBezierShape::from_points_stroke(
        main,
        false,
        Color32::TRANSPARENT,
        Stroke::new(2.0 * scale, CURVE.gamma_multiply_u8(190)),
    ));
    for &t in nodes {
        node(painter, &main, t, scale);
    }
}

/// "Craft Corel" laid out in the bold face at `size`: "Trace" in white,
/// "Draw" in the light blue accent.
pub fn name_galley(painter: &Painter, size: f32) -> Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::default();
    let font = crate::theme::bold(size);
    for (part, color) in [("Craft ", Color32::WHITE), ("Corel", NAME_ACCENT)] {
        job.append(
            part,
            0.0,
            egui::TextFormat {
                font_id: font.clone(),
                color,
                ..Default::default()
            },
        );
    }
    painter.layout_job(job)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_icon_decodes() {
        let img = icon_image().expect("icon decodes");
        assert_eq!(img.size, [128, 128]);
    }

    #[test]
    fn cubic_helpers_follow_the_curve() {
        let p = [
            Pos2::new(0.0, 0.0),
            Pos2::new(0.0, 10.0),
            Pos2::new(10.0, 10.0),
            Pos2::new(10.0, 0.0),
        ];
        assert_eq!(cubic_point(&p, 0.0), p[0]);
        assert_eq!(cubic_point(&p, 1.0), p[3]);
        let mid = cubic_point(&p, 0.5);
        assert!((mid.x - 5.0).abs() < 1e-4 && (mid.y - 7.5).abs() < 1e-4);
        // At the middle of this symmetric arch the tangent is horizontal.
        let t = cubic_tangent(&p, 0.5);
        assert!(t.y.abs() < 1e-4 && t.x > 0.0);
    }

    #[test]
    fn the_main_curve_spans_the_right_part() {
        let r = Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 100.0));
        let c = main_curve(r);
        assert!(c[0].x > 250.0 && c[3].x > 500.0);
    }
}
