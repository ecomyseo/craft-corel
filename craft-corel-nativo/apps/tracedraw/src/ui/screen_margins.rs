//! Screen margins (#1917): on Android the window covers the whole screen,
//! under the camera cutout and up to the rounded corners of the glass, so
//! parts of the menu bar, the toolbox, the palette and the status bar were
//! hidden on a phone. The interface is laid out inside a safe area instead:
//! the room the system reports for its bars, the cutout and the rounded
//! corners (when "Follow the system" is on), plus margins of the user's own
//! on each side, in density-independent pixels (dp), since many phones do
//! not report their corners. The area outside is filled with the theme's
//! frame colour, and the inner corners can be rounded to follow the glass.
//! The margins are egui's safe area insets, so every panel, docker, dialog
//! and menu keeps to `Context::content_rect`. See docs/behavior/android.md.

use std::cell::Cell;

use crate::settings::Settings;

/// The user's screen margins (Options > Display, Android).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ScreenMargins {
    /// Keep clear of what the system reports: its bars, the camera
    /// cutout and the rounded corners.
    pub follow_system: bool,
    /// Margins of the user's own, dp, added to the system's.
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    /// Changing one side changes all four.
    pub linked: bool,
    /// The radius of the interface's inner corners, dp (0: square).
    pub corner_radius: f32,
}

impl Default for ScreenMargins {
    fn default() -> Self {
        ScreenMargins {
            follow_system: true,
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
            linked: false,
            corner_radius: 0.0,
        }
    }
}

/// The largest margin or corner radius taken, dp.
pub const MAX_DP: f32 = 200.0;

impl ScreenMargins {
    /// The four margins, clamped to `0..=MAX_DP` (a hand-edited settings
    /// file may hold anything).
    fn sides(&self) -> [f32; 4] {
        [self.top, self.bottom, self.left, self.right].map(clamp_dp)
    }
}

fn clamp_dp(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, MAX_DP)
    } else {
        0.0
    }
}

/// What the system reports, in physical pixels: the room taken by its
/// bars and the camera cutout on each side, and the radius of each
/// rounded corner of the screen (top left, top right, bottom right,
/// bottom left; 0 when square or not reported).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SystemInsets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    pub corners: [f32; 4],
}

/// The activity's answer, "top bottom left right tl tr br bl" in pixels
/// (`TraceDrawActivity.getScreenInsets`); `None` when it has none yet.
pub fn parse_system(answer: &str) -> Option<SystemInsets> {
    let v: Vec<f32> = answer
        .split_whitespace()
        .map(|w| w.parse::<f32>().ok().filter(|v| v.is_finite() && *v >= 0.0))
        .collect::<Option<Vec<f32>>>()?;
    if v.len() < 4 {
        return None;
    }
    let corner = |i: usize| v.get(4 + i).copied().unwrap_or(0.0);
    Some(SystemInsets {
        top: v[0],
        bottom: v[1],
        left: v[2],
        right: v[3],
        corners: [corner(0), corner(1), corner(2), corner(3)],
    })
}

/// How far a rounded corner of radius `r` reaches into the two sides
/// that meet there: a rectangle whose corner is that far in on both
/// sides touches the arc, so nothing in it is cut by the glass.
pub fn corner_reach(r: f32) -> f32 {
    r * (1.0 - std::f32::consts::FRAC_1_SQRT_2)
}

/// The safe area insets, in interface points: the system's (pixels over
/// `pixels_per_point`) when followed, the larger of its bars or cutout
/// and its corners' reach on each side, plus the user's margins (dp over
/// the interface `zoom`, a dp being one system point).
pub fn insets(
    margins: &ScreenMargins,
    system: Option<SystemInsets>,
    pixels_per_point: f32,
    zoom: f32,
) -> egui::epaint::MarginF32 {
    let ppp = if pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    };
    let zoom = if zoom > 0.0 { zoom } else { 1.0 };
    let mut sys = [0.0f32; 4];
    if let (true, Some(s)) = (margins.follow_system, system) {
        let [tl, tr, br, bl] = s.corners.map(corner_reach);
        sys = [
            s.top.max(tl).max(tr),
            s.bottom.max(bl).max(br),
            s.left.max(tl).max(bl),
            s.right.max(tr).max(br),
        ]
        .map(|px| px / ppp);
    }
    let [t, b, l, r] = margins.sides();
    egui::epaint::MarginF32 {
        top: sys[0] + t / zoom,
        bottom: sys[1] + b / zoom,
        left: sys[2] + l / zoom,
        right: sys[3] + r / zoom,
    }
}

/// Whether this build lays the interface out inside screen margins: the
/// Android app.
pub const OFFERED: bool = crate::files::ANDROID;

thread_local! {
    /// The system's insets last read, when, and for which window size.
    static SYSTEM: Cell<(Option<SystemInsets>, f64, [f32; 2])> =
        const { Cell::new((None, f64::NEG_INFINITY, [0.0, 0.0])) };
}

/// The system's insets, read again when the window changes size (a
/// rotation) and once a second meanwhile.
fn system_insets(raw: &egui::RawInput) -> Option<SystemInsets> {
    let size = raw
        .screen_rect
        .map(|r| [r.width(), r.height()])
        .unwrap_or([0.0, 0.0]);
    let now = raw.time.unwrap_or(0.0);
    let (last, at, at_size) = SYSTEM.with(Cell::get);
    if at_size == size && (0.0..1.0).contains(&(now - at)) {
        return last;
    }
    let read = read_system();
    SYSTEM.with(|s| s.set((read.or(last), now, size)));
    read.or(last)
}

#[cfg(target_os = "android")]
fn read_system() -> Option<SystemInsets> {
    crate::android::screen_insets::read().and_then(|a| parse_system(&a))
}

#[cfg(not(target_os = "android"))]
fn read_system() -> Option<SystemInsets> {
    None
}

/// Give this frame's input the safe area the settings ask for (Android;
/// other builds keep the whole window).
pub fn apply(ctx: &egui::Context, settings: &Settings, raw: &mut egui::RawInput) {
    if !OFFERED {
        return;
    }
    let system = system_insets(raw);
    set(ctx, &settings.screen_margins, system, raw);
}

/// Put the insets for `margins` and `system` in `raw` (always, so that
/// margins set back to nothing take effect: egui keeps the last ones).
pub fn set(
    ctx: &egui::Context,
    margins: &ScreenMargins,
    system: Option<SystemInsets>,
    raw: &mut egui::RawInput,
) {
    let zoom = ctx.zoom_factor();
    let native = raw.viewport().native_pixels_per_point.unwrap_or(1.0);
    let m = insets(margins, system, native * zoom, zoom);
    raw.safe_area_insets = Some(egui::SafeAreaInsets(m));
}

/// The interface's inner corner radius, interface points.
fn corner_points(ctx: &egui::Context, margins: &ScreenMargins) -> f32 {
    clamp_dp(margins.corner_radius) / ctx.zoom_factor().max(0.01)
}

/// The layer the frame around the safe area is painted on: above the
/// panels, so that the rounded inner corners cover them.
pub fn layer() -> egui::LayerId {
    egui::LayerId::new(egui::Order::Foreground, egui::Id::new("screen_margins"))
}

/// Fill the room outside the safe area with the theme's frame colour and
/// round its inner corners. Nothing when there is no margin.
pub fn paint(ctx: &egui::Context, settings: &Settings) {
    if !OFFERED {
        return;
    }
    paint_frame(ctx, &settings.screen_margins);
}

/// [`paint`] for any build (tests).
pub fn paint_frame(ctx: &egui::Context, margins: &ScreenMargins) {
    let outer = ctx.viewport_rect();
    let inner = ctx.content_rect();
    let radius = corner_points(ctx, margins)
        .min(inner.width() / 2.0)
        .min(inner.height() / 2.0)
        .max(0.0);
    if inner == outer && radius == 0.0 {
        return;
    }
    let color = crate::theme::Tokens::title_bar();
    // The whole window: a layer's painter keeps to the safe area.
    let mut painter = ctx.layer_painter(layer());
    painter.set_clip_rect(outer);
    for strip in [
        egui::Rect::from_min_max(outer.min, egui::pos2(outer.max.x, inner.min.y)),
        egui::Rect::from_min_max(egui::pos2(outer.min.x, inner.max.y), outer.max),
        egui::Rect::from_min_max(
            egui::pos2(outer.min.x, inner.min.y),
            egui::pos2(inner.min.x, inner.max.y),
        ),
        egui::Rect::from_min_max(
            egui::pos2(inner.max.x, inner.min.y),
            egui::pos2(outer.max.x, inner.max.y),
        ),
    ] {
        if strip.is_positive() {
            painter.rect_filled(strip, 0.0, color);
        }
    }
    if radius >= 0.5 {
        painter.add(corner_mesh(inner, radius, color));
    }
}

/// The four pieces between the corners of `rect` and arcs of `radius`
/// inside them, as triangle fans from each corner.
pub fn corner_mesh(rect: egui::Rect, radius: f32, color: egui::Color32) -> egui::Mesh {
    use std::f32::consts::{FRAC_PI_2, PI};
    const STEPS: usize = 12;
    let mut mesh = egui::Mesh::default();
    // Corner, arc centre, and the angle the arc starts at.
    let corners = [
        (rect.left_top(), egui::vec2(radius, radius), PI),
        (rect.right_top(), egui::vec2(-radius, radius), 1.5 * PI),
        (rect.right_bottom(), egui::vec2(-radius, -radius), 0.0),
        (rect.left_bottom(), egui::vec2(radius, -radius), FRAC_PI_2),
    ];
    for (corner, to_centre, start) in corners {
        let centre = corner + to_centre;
        let base = mesh.vertices.len() as u32;
        mesh.colored_vertex(corner, color);
        for i in 0..=STEPS {
            let a = start + FRAC_PI_2 * i as f32 / STEPS as f32;
            mesh.colored_vertex(centre + radius * egui::vec2(a.cos(), a.sin()), color);
        }
        for i in 0..STEPS as u32 {
            mesh.add_triangle(base, base + 1 + i, base + 2 + i);
        }
    }
    mesh
}

/// Options > Display, Screen margins: follow the system, the four
/// margins (linked or not), the corner radius and Reset; every change
/// shows at once.
pub fn options_section(ui: &mut egui::Ui, m: &mut ScreenMargins) {
    use crate::i18n::tr;
    use crate::ui::tips::RichTip as _;
    crate::ui::chrome::section(ui, &tr("options.screen_margins"));
    ui.label(
        egui::RichText::new(tr("options.screen_margins_help"))
            .color(crate::theme::Tokens::text_dim())
            .size(12.0),
    );
    ui.checkbox(&mut m.follow_system, tr("options.screen_follow_system"))
        .rich_tip(crate::ui::tips::pair(
            "options.screen_follow_system",
            "options.screen_follow_system_tip",
        ));
    let before = m.sides();
    crate::ui::options::grid("screen_margins").show(ui, |ui| {
        for (key, value) in [
            ("options.screen_top", &mut m.top),
            ("options.screen_bottom", &mut m.bottom),
            ("options.screen_left", &mut m.left),
            ("options.screen_right", &mut m.right),
        ] {
            crate::ui::options::row_label(ui, &tr(key));
            ui.horizontal(|ui| {
                ui.add_sized(
                    [70.0, 22.0],
                    crate::ui::field::NumField::new(value)
                        .range(0.0..=MAX_DP)
                        .speed(0.5)
                        .max_decimals(0),
                )
                .rich_tip(crate::ui::tips::pair(key, "options.screen_margin_tip"));
                ui.label("dp");
            });
            ui.end_row();
        }
        crate::ui::options::row_label(ui, &tr("options.screen_corner_radius"));
        ui.horizontal(|ui| {
            ui.add_sized(
                [70.0, 22.0],
                crate::ui::field::NumField::new(&mut m.corner_radius)
                    .range(0.0..=MAX_DP)
                    .speed(0.5)
                    .max_decimals(0),
            )
            .rich_tip(crate::ui::tips::pair(
                "options.screen_corner_radius",
                "options.screen_corner_radius_tip",
            ));
            ui.label("dp");
        });
        ui.end_row();
    });
    if m.linked {
        link_sides(m, before);
    }
    ui.horizontal(|ui| {
        ui.checkbox(&mut m.linked, tr("options.screen_margins_link"))
            .rich_tip(crate::ui::tips::pair(
                "options.screen_margins_link",
                "options.screen_margins_link_tip",
            ));
        if ui
            .button(tr("options.reset"))
            .rich_tip(crate::ui::tips::pair(
                "options.reset",
                "options.screen_margins_reset_tip",
            ))
            .clicked()
        {
            *m = ScreenMargins {
                linked: m.linked,
                ..ScreenMargins::default()
            };
        }
    });
}

/// With the sides linked, the side that changed sets all four.
pub fn link_sides(m: &mut ScreenMargins, before: [f32; 4]) {
    let now = [m.top, m.bottom, m.left, m.right];
    if let Some(v) = now
        .iter()
        .zip(before.iter())
        .find(|(a, b)| a != b)
        .map(|(a, _)| *a)
    {
        m.top = v;
        m.bottom = v;
        m.left = v;
        m.right = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn margins(top: f32, bottom: f32, left: f32, right: f32) -> ScreenMargins {
        ScreenMargins {
            top,
            bottom,
            left,
            right,
            ..ScreenMargins::default()
        }
    }

    /// The activity's answer is read; a short or broken one is not.
    #[test]
    fn the_system_answer_is_parsed() {
        let s = parse_system("120 66 0 0 90 90 80 80").expect("parsed");
        assert_eq!((s.top, s.bottom, s.left, s.right), (120.0, 66.0, 0.0, 0.0));
        assert_eq!(s.corners, [90.0, 90.0, 80.0, 80.0]);
        // Before Android 12 no corners are reported.
        assert_eq!(parse_system("10 0 0 0").map(|s| s.corners), Some([0.0; 4]));
        assert_eq!(parse_system(""), None);
        assert_eq!(parse_system("1 2 3"), None);
        assert_eq!(parse_system("1 2 x 4"), None);
        assert_eq!(parse_system("1 -2 3 4"), None);
    }

    /// The system's pixels become interface points, a rounded corner
    /// keeps both sides it joins clear of its arc, and the user's dp
    /// margins come on top at the interface zoom.
    #[test]
    fn insets_add_the_users_margins_to_the_systems() {
        let system = SystemInsets {
            top: 100.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
            corners: [0.0, 0.0, 100.0, 100.0],
        };
        // 2.5 pixels per point, interface at 100 %.
        let m = insets(&margins(0.0, 0.0, 0.0, 0.0), Some(system), 2.5, 1.0);
        assert!((m.top - 40.0).abs() < 1e-3, "{m:?}");
        let reach = corner_reach(100.0) / 2.5;
        assert!((m.bottom - reach).abs() < 1e-3, "{m:?}");
        assert!((m.left - reach).abs() < 1e-3 && (m.right - reach).abs() < 1e-3);
        // 10 dp more at the top, with the interface at 125 %.
        let m = insets(&margins(10.0, 0.0, 0.0, 0.0), Some(system), 2.5, 1.25);
        assert!((m.top - (40.0 + 8.0)).abs() < 1e-3, "{m:?}");
        // Not following the system: the user's margins only.
        let own = ScreenMargins {
            follow_system: false,
            ..margins(4.0, 6.0, 8.0, 12.0)
        };
        let m = insets(&own, Some(system), 2.5, 2.0);
        assert_eq!((m.top, m.bottom, m.left, m.right), (2.0, 3.0, 4.0, 6.0));
        // Nonsense from a settings file is held in range.
        let wild = margins(f32::NAN, -5.0, 1e9, 0.0);
        let m = insets(&wild, None, 1.0, 1.0);
        assert_eq!((m.top, m.bottom, m.left, m.right), (0.0, 0.0, MAX_DP, 0.0));
    }

    /// A rectangle inset by a corner's reach touches the arc and does not
    /// cross it.
    #[test]
    fn a_corners_reach_keeps_the_rectangle_inside_the_arc() {
        let r = 80.0;
        let d = corner_reach(r);
        // The inner corner (d, d) seen from the arc's centre (r, r).
        let dist = ((r - d).powi(2) * 2.0).sqrt();
        assert!((dist - r).abs() < 1e-3);
    }

    /// With the sides linked, the one that changed sets the others.
    #[test]
    fn linked_sides_follow_the_one_changed() {
        let mut m = margins(5.0, 5.0, 5.0, 5.0);
        let before = m.sides();
        m.left = 12.0;
        link_sides(&mut m, before);
        assert_eq!([m.top, m.bottom, m.left, m.right], [12.0; 4]);
    }

    /// The four corner pieces lie inside the rectangle, in its corners.
    #[test]
    fn the_corner_mesh_stays_in_the_corners() {
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(300.0, 600.0));
        let mesh = corner_mesh(rect, 24.0, egui::Color32::RED);
        assert_eq!(mesh.indices.len(), 4 * 12 * 3);
        for v in &mesh.vertices {
            assert!(rect.expand(1e-3).contains(v.pos), "{:?}", v.pos);
            let near_x =
                (v.pos.x - rect.left()).abs() <= 24.001 || (v.pos.x - rect.right()).abs() <= 24.001;
            let near_y =
                (v.pos.y - rect.top()).abs() <= 24.001 || (v.pos.y - rect.bottom()).abs() <= 24.001;
            assert!(near_x && near_y, "{:?}", v.pos);
        }
    }

    /// One frame of the whole interface on a 400 x 800 point phone screen
    /// with `margins` given as the safe area (no system insets).
    fn phone_frame(
        ctx: &egui::Context,
        app: &mut crate::app::App,
        margins: Option<&ScreenMargins>,
        frame: bool,
    ) -> egui::FullOutput {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400.0, 800.0),
            )),
            ..Default::default()
        };
        if let Some(m) = margins {
            set(ctx, m, None, &mut input);
        }
        let mut out = ctx.run_ui(input, |ui| {
            crate::ui::root(app, ui);
            if let (true, Some(m)) = (frame, margins) {
                paint_frame(ui.ctx(), m);
            }
        });
        out.textures_delta.clear();
        out
    }

    /// The parts of what was painted that show (inside their clip).
    fn painted(out: &egui::FullOutput) -> Vec<egui::Rect> {
        out.shapes
            .iter()
            .map(|c| c.shape.visual_bounding_rect().intersect(c.clip_rect))
            .filter(|r| r.is_positive())
            .collect()
    }

    /// The menu bar, the bars, the toolbox, the palette, the dockers and
    /// the status bar are all laid out inside the margins (#1917): nothing
    /// is painted in them, so the notch and the rounded corners cover
    /// nothing. Without margins the interface reaches the screen's edges.
    #[test]
    fn the_interface_keeps_inside_the_screen_margins() {
        let ctx = crate::theme::ui_context();
        let mut app = crate::app::App::headless();
        app.show_welcome = false;
        let full = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 800.0));
        // Without margins: the menu bar starts at the top left corner and
        // the status bar ends at the bottom.
        let mut out = phone_frame(&ctx, &mut app, None, false);
        for _ in 0..3 {
            out = phone_frame(&ctx, &mut app, None, false);
        }
        let all = painted(&out)
            .into_iter()
            .fold(egui::Rect::NOTHING, |a, r| a.union(r));
        assert!(all.min.x <= 0.5 && all.min.y <= 0.5, "{all:?}");
        assert!(all.max.y >= 799.5, "{all:?}");

        let m = ScreenMargins {
            follow_system: true,
            top: 30.0,
            bottom: 20.0,
            left: 12.0,
            right: 16.0,
            linked: false,
            corner_radius: 0.0,
        };
        for _ in 0..4 {
            out = phone_frame(&ctx, &mut app, Some(&m), false);
        }
        let inside = egui::Rect::from_min_max(egui::pos2(12.0, 30.0), egui::pos2(384.0, 780.0));
        assert_eq!(ctx.content_rect(), inside);
        assert_eq!(ctx.viewport_rect(), full);
        let shown = painted(&out);
        assert!(shown.len() > 50, "the interface was drawn");
        for r in &shown {
            assert!(
                inside.expand(0.5).contains_rect(*r),
                "{r:?} is under a margin"
            );
        }
        // The interface still fills the safe area.
        let all = shown.iter().fold(egui::Rect::NOTHING, |a, r| a.union(*r));
        assert!((all.min.y - 30.0).abs() <= 1.0 && (all.max.y - 780.0).abs() <= 1.0);

        // The frame fills the margins and rounds the inner corners.
        let rounded = ScreenMargins {
            corner_radius: 18.0,
            ..m
        };
        let out = phone_frame(&ctx, &mut app, Some(&rounded), true);
        let shown = painted(&out);
        for strip in [
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(400.0, 30.0)),
            egui::Rect::from_min_max(egui::pos2(0.0, 780.0), egui::pos2(400.0, 800.0)),
        ] {
            assert!(
                shown.iter().any(|r| r.contains_rect(strip)),
                "{strip:?} is not filled"
            );
        }
        let has_corners = out.shapes.iter().any(|c| match &c.shape {
            egui::Shape::Mesh(mesh) => mesh.vertices.iter().any(|v| v.pos == inside.left_top()),
            _ => false,
        });
        assert!(has_corners, "the inner corners are not rounded");

        // Margins taken back: the whole screen again (egui keeps the last
        // safe area unless it is given anew).
        let none = ScreenMargins {
            follow_system: false,
            ..ScreenMargins::default()
        };
        phone_frame(&ctx, &mut app, Some(&none), true);
        assert_eq!(ctx.content_rect(), full);
    }

    /// Settings written before the margins existed read with the system
    /// followed and no margins of the user's own.
    #[test]
    fn old_settings_follow_the_system() {
        let s: Settings = serde_json::from_str("{}").expect("settings");
        assert_eq!(s.screen_margins, ScreenMargins::default());
        assert!(s.screen_margins.follow_system);
        let back: ScreenMargins =
            serde_json::from_str(&serde_json::to_string(&margins(1.0, 2.0, 3.0, 4.0)).unwrap())
                .unwrap();
        assert_eq!(back, margins(1.0, 2.0, 3.0, 4.0));
    }
}
