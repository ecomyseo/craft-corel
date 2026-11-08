//! The Distort tool's property bar and the handles of the selected
//! distortions. Bar: Push and Pull, Zipper, Twister; the values of the
//! kind shown (the selected object's distortion, else the tool's); Center
//! Distortion, Clear Distortion, Convert to Curves; Apply for selected
//! objects without a distortion.

use crate::app::App;
use crate::distort_tool::{DistortMode, MAX_TURNS};
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::ui::field::NumField;
use crate::ui::propbar::sep;
use crate::ui::tips::RichTip;
use egui::{Color32, Painter, Pos2, Stroke, Ui};

fn dim(ui: &mut Ui, text: String) {
    ui.label(
        egui::RichText::new(text)
            .color(Tokens::text_dim())
            .size(11.0),
    );
}

/// The bar.
pub fn distort_bar(app: &mut App, ui: &mut Ui) {
    // The selected distortion's values are the ones shown and edited.
    if let Some(d) = app.selected_distortion() {
        let mut s = app.distort;
        s.take(&d);
        app.distort = s;
    }
    let shown = app.distort.mode;
    for m in DistortMode::ALL {
        if ui
            .selectable_label(shown == m, tr(m.key()))
            .rich_tip(crate::ui::tips::keyed(m.key()))
            .clicked()
            && m != shown
        {
            app.set_distort_mode(m);
        }
    }
    sep(ui);
    let mut s = app.distort;
    let mut changed = false;
    match shown {
        DistortMode::PushPull => {
            dim(ui, tr("toolbar.amplitude"));
            changed |= ui
                .add(
                    NumField::new(&mut s.amount)
                        .range(-200.0..=200.0)
                        .max_decimals(1),
                )
                .rich_tip(tr("distort.amount_tip"))
                .changed();
        }
        DistortMode::Zipper => {
            dim(ui, tr("toolbar.amplitude"));
            changed |= ui
                .add(
                    NumField::new(&mut s.amplitude)
                        .range(0.0..=100.0)
                        .unit_mm(1.0)
                        .suffix(" mm"),
                )
                .rich_tip(crate::ui::tips::pair(
                    "toolbar.amplitude",
                    "distort.zipper_amplitude_desc",
                ))
                .changed();
            dim(ui, tr("toolbar.frequency"));
            changed |= ui
                .add(
                    NumField::new(&mut s.frequency)
                        .range(0..=tracedraw_core::distort::MAX_FREQUENCY)
                        .width(44.0),
                )
                .rich_tip(tr("distort.frequency_tip"))
                .changed();
            // The teeth's lean from the outline's normal (#1671), shown
            // folded into -90 to 90 (`DistortSettings::lean`).
            dim(ui, tr("distort.tilt"));
            let mut lean = s.lean();
            if ui
                .add(
                    NumField::new(&mut lean)
                        .range(-90.0..=90.0)
                        .max_decimals(0)
                        .suffix("\u{b0}")
                        .width(52.0),
                )
                .rich_tip(crate::ui::tips::keyed("distort.tilt"))
                .changed()
            {
                s.set_lean(lean);
                changed = true;
            }
            for (flag, key, tip) in [
                (&mut s.random, "distort.random", "distort.random_tip"),
                (&mut s.smooth, "distort.smooth", "distort.smooth_tip"),
                (&mut s.local, "distort.local", "distort.local_tip"),
            ] {
                if ui
                    .selectable_label(*flag, tr(key))
                    .rich_tip(tr(tip))
                    .clicked()
                {
                    *flag = !*flag;
                    changed = true;
                }
            }
        }
        DistortMode::Twister => {
            for (cw, key) in [
                (true, "distort.clockwise"),
                (false, "distort.counterclockwise"),
            ] {
                if ui
                    .selectable_label(s.clockwise == cw, tr(key))
                    .rich_tip(crate::ui::tips::keyed(key))
                    .clicked()
                    && s.clockwise != cw
                {
                    s.clockwise = cw;
                    changed = true;
                }
            }
            dim(ui, tr("distort.rotations"));
            changed |= ui
                .add(NumField::new(&mut s.turns).range(0..=MAX_TURNS).width(36.0))
                .rich_tip(crate::ui::tips::keyed("distort.rotations"))
                .changed();
            dim(ui, tr("distort.degrees"));
            changed |= ui
                .add(
                    NumField::new(&mut s.degrees)
                        .range(0.0..=359.0)
                        .max_decimals(0)
                        .suffix("\u{b0}")
                        .width(52.0),
                )
                .rich_tip(crate::ui::tips::keyed("distort.degrees"))
                .changed();
        }
    }
    if changed {
        app.distort = s;
        app.apply_distort_settings();
    }
    sep(ui);
    let distorted = app.selected_distortion().is_some();
    if ui
        .add_enabled(distorted, egui::Button::new(tr("distort.center")))
        .clicked()
    {
        app.center_distortion();
    }
    if ui
        .add_enabled(distorted, egui::Button::new(tr("distort.clear")))
        .clicked()
    {
        app.clear_distortion();
    }
    if ui
        .add_enabled(distorted, egui::Button::new(tr("distort.convert")))
        .clicked()
    {
        app.bake_distortion();
    }
    let undistorted = app.selected_shapes().iter().any(|sh| {
        crate::distort_tool::distortion_of(&sh.effects).is_none()
            && !matches!(
                sh.kind,
                tracedraw_core::ShapeKind::Bitmap { .. } | tracedraw_core::ShapeKind::Group { .. }
            )
    });
    if undistorted
        && ui
            .button(tr("docker.apply"))
            .rich_tip(tr("distort.apply_tip"))
            .clicked()
    {
        app.add_distortion();
    }
    sep(ui);
    dim(ui, tr("distort.hint"));
}

/// Where and how large the Distort handles are drawn, screen points
/// (#1574). Each handle stands on a corner of the device pixel grid and
/// is a whole number of device pixels across, so its ring and edge fill
/// whole pixels: it shows crisp and upright wherever the view puts it,
/// not smeared over two pixels and lopsided.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HandleMarks {
    pub center: Pos2,
    pub end: Pos2,
    pub slider: Option<Pos2>,
    /// Half the side of a square handle, the radius of a round one, to
    /// the outside of its ring: half the handle size.
    pub half: f32,
    /// The ring's width, inside `half`.
    pub ring: f32,
    /// The dark edge's width, outside `half`.
    pub edge: f32,
}

/// The marks of a handle line from `center` to `end` (screen points)
/// for handles `size` points across (`App::handle_size`) at `ppp` device
/// pixels per point.
pub(crate) fn handle_marks(
    center: Pos2,
    end: Pos2,
    slider: Option<Pos2>,
    size: f32,
    ppp: f32,
) -> HandleMarks {
    let ppp = if ppp.is_finite() && ppp > 0.0 {
        ppp
    } else {
        1.0
    };
    let size = if size.is_finite() { size } else { 10.0 };
    let snap = |p: Pos2| Pos2::new((p.x * ppp).round() / ppp, (p.y * ppp).round() / ppp);
    HandleMarks {
        center: snap(center),
        end: snap(end),
        slider: slider.map(snap),
        half: (size * ppp / 2.0).round().max(2.0) / ppp,
        ring: (2.0 * ppp).round().max(2.0) / ppp,
        edge: ppp.round().max(1.0) / ppp,
    }
}

/// The handles of each selected distortion: the centre (a diamond), a
/// line with an arrow to the end handle (a square for the amount or
/// amplitude, a circle for the Twister's angle) and the Zipper's
/// frequency slider on the line, a smaller circle (#1122). As the Shape
/// tool's nodes: white inside a ring of the main colour of Options >
/// Nodes and Handles, as large as its Handle size, with a dark edge
/// outside, so each shows on a white page (the edge), on a dark drawing
/// (the fill) and on the node colour itself; the line in the main colour
/// on a white halo (#1574).
pub fn draw_handles(app: &App, painter: &Painter) {
    if app.tool != crate::tools::Tool::Distort {
        return;
    }
    let [r, g, b] = app.settings.nodes.main_rgb;
    let main = Color32::from_rgb(r, g, b);
    let dark = Tokens::HANDLE;
    let size = app.handle_size();
    let ppp = painter.pixels_per_point();
    for h in app.distortion_handles() {
        let m = handle_marks(
            app.view.to_screen(h.center),
            app.view.to_screen(h.end),
            h.slider.map(|p| app.view.to_screen(p)),
            size,
            ppp,
        );
        let (c, e) = (m.center, m.end);
        if (e - c).length() > m.half + m.edge {
            // The line, its white halo first, then the arrow head with
            // its tip on the end handle's edge.
            painter.line_segment([c, e], Stroke::new(m.ring + 2.0 * m.edge, Color32::WHITE));
            painter.line_segment([c, e], Stroke::new(m.ring, main));
            let dir = (e - c).normalized();
            let side = egui::vec2(-dir.y, dir.x);
            let tip = e - dir * (m.half + m.edge);
            let base = tip - dir * (3.0 * m.ring + m.edge);
            painter.add(egui::Shape::convex_polygon(
                vec![tip, base + side * 2.0 * m.ring, base - side * 2.0 * m.ring],
                main,
                Stroke::new(m.edge, Color32::WHITE),
            ));
        }
        match h.mode {
            DistortMode::Twister => round_mark(painter, e, m.half, &m, main, dark),
            DistortMode::PushPull | DistortMode::Zipper => {
                for (half, color) in [
                    (m.half + m.edge, dark),
                    (m.half, main),
                    (m.half - m.ring, Color32::WHITE),
                ] {
                    if half > 0.0 {
                        painter.rect_filled(
                            egui::Rect::from_center_size(e, egui::vec2(2.0 * half, 2.0 * half)),
                            0.0,
                            color,
                        );
                    }
                }
            }
        }
        if let Some(sl) = m.slider {
            round_mark(
                painter,
                sl,
                (m.half - m.edge).max(m.ring + m.edge),
                &m,
                main,
                dark,
            );
        }
        // The centre: a diamond reaching a pixel past a square handle.
        let d = m.half + m.edge;
        let s2 = std::f32::consts::SQRT_2;
        for (half, color) in [
            (d + m.edge * s2, dark),
            (d, main),
            (d - m.ring * s2, Color32::WHITE),
        ] {
            if half > 0.0 {
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        c + egui::vec2(0.0, -half),
                        c + egui::vec2(half, 0.0),
                        c + egui::vec2(0.0, half),
                        c + egui::vec2(-half, 0.0),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
        }
    }
}

/// A round handle of radius `r` at `p`: dark edge, ring, white fill.
/// Each disc is a polygon with a corner on every axis, so it mirrors
/// across the handle line both ways (a small circle of the painter's is
/// only turned round, a shade lopsided).
fn round_mark(painter: &Painter, p: Pos2, r: f32, m: &HandleMarks, main: Color32, dark: Color32) {
    let disc = |r: f32, color: Color32| {
        let pts = (0..32)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / 32.0;
                p + egui::vec2(a.cos(), a.sin()) * r
            })
            .collect();
        egui::Shape::convex_polygon(pts, color, Stroke::NONE)
    };
    painter.add(disc(r + m.edge, dark));
    painter.add(disc(r, main));
    if r > m.ring {
        painter.add(disc(r - m.ring, Color32::WHITE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{distort::Distort, geometry::Rect, ShapeKind};

    #[test]
    fn the_bar_draws_for_every_kind_and_shows_the_selection() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.tool = crate::tools::Tool::Distort;
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 20.0, 20.0),
                radius: 0.0,
                corners: None,
            })
            .expect("square");
        let frame = |app: &mut App| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 300.0),
                )),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                ui.horizontal(|ui| distort_bar(app, ui));
                draw_handles(app, ui.painter());
            });
            out.textures_delta.clear();
        };
        for m in DistortMode::ALL {
            app.distort.mode = m;
            frame(&mut app);
        }
        // A selected twister shows as the bar's values.
        app.select(vec![id]);
        app.distort.mode = DistortMode::Twister;
        app.add_distortion();
        app.distort.mode = DistortMode::PushPull;
        frame(&mut app);
        assert_eq!(app.distort.mode, DistortMode::Twister);
        assert!(matches!(
            app.selected_distortion(),
            Some(Distort::Twister { .. })
        ));
        // The handles draw for every kind.
        for m in DistortMode::ALL {
            app.set_distort_mode(m);
            frame(&mut app);
            assert_eq!(app.distortion_handles().len(), 1);
        }
    }

    use crate::help_shots::{rasterise, Raster};
    use crate::settings::HandleSize;

    const W: usize = 200;
    const H: usize = 160;

    /// A 40 x 10 mm rectangle with a distortion of kind `mode` about its
    /// middle, the Distort tool on, the view at a fractional offset so
    /// the handles fall between pixels (#1574).
    fn distorted(mode: DistortMode) -> App {
        let mut app = App::headless();
        app.tool = crate::tools::Tool::Distort;
        app.view.zoom = 3.0;
        app.view.origin = egui::pos2(30.37, 110.61);
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 40.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rectangle");
        app.select(vec![id]);
        app.distort.mode = mode;
        app.distort.amount = 60.0;
        // The Zipper's slider halfway along, clear of the centre.
        app.distort.amplitude = 4.0;
        app.distort.frequency = 50;
        app.add_distortion();
        assert_eq!(app.distortion_handles().len(), 1);
        app
    }

    fn render(app: &App, bg: Color32) -> Raster {
        rasterise(W, H, 1.0, |p| {
            p.rect_filled(p.clip_rect(), 0.0, bg);
            draw_handles(app, p);
        })
    }

    /// The handles' screen positions: centre, end, slider.
    fn spots(app: &App) -> Vec<egui::Pos2> {
        let h = &app.distortion_handles()[0];
        [Some(h.center), Some(h.end), h.slider]
            .into_iter()
            .flatten()
            .map(|p| app.view.to_screen(p))
            .collect()
    }

    fn luminance(c: [u8; 3]) -> f64 {
        let lin = |v: u8| {
            let s = v as f64 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
    }

    fn contrast(a: [u8; 3], b: [u8; 3]) -> f64 {
        let (la, lb) = (luminance(a), luminance(b));
        (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
    }

    /// Between pixels the handles are not drawn half in one pixel and
    /// half in the next, which shows them smeared and lopsided: each
    /// stands on the pixel grid, so the picture is a mirror image about
    /// the handle line (a straight line here) and every handle is upright
    /// on it (#1574).
    #[test]
    fn the_handles_stand_upright_on_the_pixel_grid() {
        for mode in DistortMode::ALL {
            let app = distorted(mode);
            let s = spots(&app);
            let (c, e) = (s[0], s[1]);
            let img = render(&app, Color32::WHITE);
            let horizontal = (c.y - e.y).abs() < 1e-3;
            assert!(
                horizontal || (c.x - e.x).abs() < 1e-3,
                "{mode:?}: a straight handle line, {c:?} to {e:?}"
            );
            // The mirror axis: the pixel edge nearest the line.
            let k = if horizontal { c.y } else { c.x }.round() as usize;
            let (across, along) = if horizontal { (H, W) } else { (W, H) };
            let px = |a: usize, b: usize| {
                if horizontal {
                    img.rgb(b, a)
                } else {
                    img.rgb(a, b)
                }
            };
            for d in 0..20.min(k).min(across - k) {
                for b in 0..along {
                    let (p, q) = (px(k - 1 - d, b), px(k + d, b));
                    let off = p.iter().zip(&q).map(|(x, y)| x.abs_diff(*y)).max();
                    assert!(
                        off.unwrap_or(0) <= 4,
                        "{mode:?}: not a mirror image {d} px off the line at {b}: {p:?} vs {q:?}"
                    );
                }
            }
        }
    }

    /// The end handle is as large as Options > Nodes and Handles > Handle
    /// size makes the selection handles (#1285), with a pixel of halo
    /// each side.
    #[test]
    fn the_handles_take_the_handle_size() {
        for size in HandleSize::ALL {
            let mut app = distorted(DistortMode::PushPull);
            app.settings.nodes.handles = size;
            let e = spots(&app)[1];
            let grey = Color32::from_gray(128);
            let img = render(&app, grey);
            let x = e.x.round() as usize;
            let drawn = (0..H).filter(|y| img.rgb(x, *y) != [128; 3]).count();
            assert_eq!(
                drawn,
                size.px() as usize + 2,
                "{size:?}: the end handle's height"
            );
        }
    }

    /// The handles take the main colour of Options > Nodes and Handles
    /// for their ring, filled white inside, with a dark edge outside.
    #[test]
    fn the_handles_take_the_node_colour() {
        let mut app = distorted(DistortMode::PushPull);
        app.settings.nodes.main_rgb = [0, 160, 0];
        for size in HandleSize::ALL {
            app.settings.nodes.handles = size;
            let e = spots(&app)[1];
            let img = render(&app, Color32::WHITE);
            let (x, y) = (e.x.round() as usize, e.y.round() as usize);
            let half = size.px() as usize / 2;
            assert_eq!(img.rgb(x, y - half), [0, 160, 0], "{size:?}: the ring");
            assert_eq!(img.rgb(x, y + half - 1), [0, 160, 0], "{size:?}: the ring");
            assert_eq!(img.rgb(x, y - 1), [255; 3], "{size:?}: the fill");
            let edge = Tokens::HANDLE;
            let dark = [edge.r(), edge.g(), edge.b()];
            assert_eq!(img.rgb(x, y - half - 1), dark, "{size:?}: the edge");
            assert_eq!(img.rgb(x, y + half), dark, "{size:?}: the edge");
        }
    }

    /// Every handle shows clearly on a white page, on a dark drawing and
    /// on the node colour itself (#1574).
    #[test]
    fn the_handles_show_on_light_and_dark() {
        for mode in DistortMode::ALL {
            let app = distorted(mode);
            let [r, g, b] = app.settings.nodes.main_rgb;
            for (bg, least) in [
                (Color32::WHITE, 7.0),
                (Color32::from_gray(0x20), 7.0),
                (Color32::from_rgb(r, g, b), 3.0),
            ] {
                let img = render(&app, bg);
                let back = [bg.r(), bg.g(), bg.b()];
                for p in spots(&app) {
                    let (cx, cy) = (p.x.round() as i64, p.y.round() as i64);
                    let mut best: f64 = 1.0;
                    for y in cy - 8..cy + 8 {
                        for x in cx - 8..cx + 8 {
                            if x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H {
                                best = best.max(contrast(img.rgb(x as usize, y as usize), back));
                            }
                        }
                    }
                    assert!(
                        best >= least,
                        "{mode:?} on {back:?}: the handle at {p:?} has contrast {best:.2}"
                    );
                }
            }
        }
    }

    /// The marks stand on device pixel corners and are whole device
    /// pixels across at any interface size, so they never tilt or smear
    /// (#1574).
    #[test]
    fn the_marks_are_rounded_to_device_pixels() {
        let whole = |v: f32, ppp: f32| ((v * ppp) - (v * ppp).round()).abs() < 1e-4;
        for ppp in [1.0, 1.25, 1.5, 2.0, 3.0] {
            for size in HandleSize::ALL {
                let m = handle_marks(
                    egui::pos2(70.37, 110.61),
                    egui::pos2(130.9, 40.12),
                    Some(egui::pos2(100.5, 75.49)),
                    size.px(),
                    ppp,
                );
                let slider = m.slider.expect("slider");
                for p in [m.center, m.end, slider] {
                    assert!(whole(p.x, ppp) && whole(p.y, ppp), "{p:?} at {ppp}");
                }
                assert!((m.center.x - 70.37).abs() <= 0.5 / ppp + 1e-4);
                assert!((m.end.y - 40.12).abs() <= 0.5 / ppp + 1e-4);
                for v in [m.half, m.ring, m.edge] {
                    assert!(whole(v, ppp) && v > 0.0, "{v} at {ppp}");
                }
                assert!((2.0 * m.half - size.px()).abs() <= 1.0 / ppp + 1e-4);
                assert!(m.ring < m.half);
            }
        }
        // Nonsense in, something drawable out.
        let m = handle_marks(egui::Pos2::ZERO, egui::Pos2::ZERO, None, f32::NAN, 0.0);
        assert!(m.half.is_finite() && m.half > 0.0);
    }

    /// A press is taken out to the edge of a large handle.
    #[test]
    fn the_handles_are_taken_to_their_edge() {
        let mut app = distorted(DistortMode::PushPull);
        app.settings.nodes.handles = HandleSize::Small;
        assert_eq!(app.distort_pick_px(), 6.0);
        app.settings.nodes.handles = HandleSize::Large;
        assert_eq!(app.distort_pick_px(), 7.0);
    }

    /// Before and after pictures for the pull request: `DISTORT_SHOTS`
    /// names the folder.
    #[test]
    #[ignore]
    fn distort_handle_shots() {
        let Ok(dir) = std::env::var("DISTORT_SHOTS") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&dir);
        let tag = std::env::var("DISTORT_SHOTS_TAG").unwrap_or_else(|_| "after".into());
        for mode in DistortMode::ALL {
            let app = distorted(mode);
            for (name, bg) in [
                ("light", Color32::WHITE),
                ("dark", Color32::from_gray(0x20)),
            ] {
                render(&app, bg).save(&dir.join(format!("{tag}-{mode:?}-{name}.png")));
            }
        }
    }
}
