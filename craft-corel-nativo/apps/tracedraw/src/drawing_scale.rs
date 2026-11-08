//! Drawing scale (Document Setup > Rulers > Drawing scale): a distance
//! on the page stands for a distance in the world, so a plan drawn at
//! 1:100 shows real sizes on the rulers, in the coordinates, the sizes and
//! the dimension labels. The scale lives in the drawing
//! (`RulerSettings::scale_page` and `scale_world`, in mm); `Units`
//! converts through it with `world_from_mm` and `world_to_mm`.

use crate::app::Units;
use tracedraw_core::document::RulerSettings;

/// A typical scale: `page` units of `page_unit` on the page for `world`
/// units of `world_unit` in the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset {
    pub label: &'static str,
    pub page: f64,
    pub page_unit: Units,
    pub world: f64,
    pub world_unit: Units,
}

impl Preset {
    /// World millimetres per page millimetre.
    pub fn ratio(&self) -> f64 {
        self.world_unit.to_mm(self.world) / self.page_unit.to_mm(self.page)
    }
}

const fn ratio(label: &'static str, page: f64, world: f64) -> Preset {
    Preset {
        label,
        page,
        page_unit: Units::Millimeters,
        world,
        world_unit: Units::Millimeters,
    }
}

const fn imperial(label: &'static str, page_in: f64, world_ft: f64) -> Preset {
    Preset {
        label,
        page: page_in,
        page_unit: Units::Inches,
        world: world_ft,
        world_unit: Units::Feet,
    }
}

/// The Typical scales list.
pub const PRESETS: [Preset; 28] = [
    ratio("1:1", 1.0, 1.0),
    ratio("1:2", 1.0, 2.0),
    ratio("1:5", 1.0, 5.0),
    ratio("1:10", 1.0, 10.0),
    ratio("1:20", 1.0, 20.0),
    ratio("1:25", 1.0, 25.0),
    ratio("1:50", 1.0, 50.0),
    ratio("1:75", 1.0, 75.0),
    ratio("1:100", 1.0, 100.0),
    ratio("1:200", 1.0, 200.0),
    ratio("1:250", 1.0, 250.0),
    ratio("1:500", 1.0, 500.0),
    ratio("1:1000", 1.0, 1000.0),
    ratio("1:2000", 1.0, 2000.0),
    ratio("1:2500", 1.0, 2500.0),
    ratio("1:5000", 1.0, 5000.0),
    ratio("1:10000", 1.0, 10000.0),
    ratio("2:1", 2.0, 1.0),
    ratio("5:1", 5.0, 1.0),
    ratio("10:1", 10.0, 1.0),
    imperial("1/16 in = 1 ft", 0.0625, 1.0),
    imperial("1/8 in = 1 ft", 0.125, 1.0),
    imperial("1/4 in = 1 ft", 0.25, 1.0),
    imperial("1/2 in = 1 ft", 0.5, 1.0),
    imperial("3/4 in = 1 ft", 0.75, 1.0),
    imperial("1 in = 1 ft", 1.0, 1.0),
    imperial("1 in = 10 ft", 1.0, 10.0),
    imperial("1 in = 100 ft", 1.0, 100.0),
];

/// The preset the settings match (the same ratio, and inch and foot
/// presets only when the distances were given in those units).
pub fn matching_preset(r: &RulerSettings) -> Option<&'static Preset> {
    let ratio = r.world_per_page();
    let page_unit = Units::from_id(unit_id(&r.scale_page_units));
    let world_unit = Units::from_id(unit_id(&r.scale_world_units));
    PRESETS.iter().find(|p| {
        let imperial_units = p.page_unit == Units::Inches && p.world_unit == Units::Feet;
        let units_match =
            !imperial_units || (page_unit == Units::Inches && world_unit == Units::Feet);
        units_match && ((p.ratio() - ratio).abs() <= ratio * 1e-9)
    })
}

/// A stored unit id, "mm" when empty.
fn unit_id(id: &str) -> &str {
    if id.is_empty() {
        "mm"
    } else {
        id
    }
}

/// The page and world units the settings were given in.
pub fn units_of(r: &RulerSettings) -> (Units, Units) {
    (
        Units::from_id(unit_id(&r.scale_page_units)),
        Units::from_id(unit_id(&r.scale_world_units)),
    )
}

/// Set the scale to `page` of `page_unit` for `world` of `world_unit`;
/// values that are not positive and finite leave it as it was.
pub fn set(r: &mut RulerSettings, page: f64, page_unit: Units, world: f64, world_unit: Units) {
    if !(page.is_finite() && page > 0.0 && world.is_finite() && world > 0.0) {
        return;
    }
    r.scale_page = page_unit.to_mm(page);
    r.scale_world = world_unit.to_mm(world);
    r.scale_page_units = page_unit.id().to_string();
    r.scale_world_units = world_unit.id().to_string();
}

/// The scale as a short ratio for labels: "1:100", "2:1", "1:48"; `None`
/// at 1:1.
pub fn short_label(r: &RulerSettings) -> Option<String> {
    if !r.is_scaled() {
        return None;
    }
    let ratio = r.world_per_page();
    let number = |v: f64| {
        let s = format!("{v:.3}");
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
    };
    Some(if ratio >= 1.0 {
        format!("1:{}", number(ratio))
    } else {
        format!("{}:1", number(1.0 / ratio))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_the_ratio_their_label_says() {
        let find = |l: &str| PRESETS.iter().find(|p| p.label == l).copied();
        assert_eq!(find("1:100").map(|p| p.ratio()), Some(100.0));
        assert_eq!(find("2:1").map(|p| p.ratio()), Some(0.5));
        // A quarter inch for a foot is 1:48.
        let quarter = find("1/4 in = 1 ft").map(|p| p.ratio()).unwrap_or(0.0);
        assert!((quarter - 48.0).abs() < 1e-9, "{quarter}");
        // Every label is different.
        for (i, a) in PRESETS.iter().enumerate() {
            assert!(PRESETS[i + 1..].iter().all(|b| b.label != a.label));
        }
    }

    #[test]
    fn the_settings_find_their_preset_and_label() {
        let mut r = RulerSettings::default();
        assert_eq!(matching_preset(&r).map(|p| p.label), Some("1:1"));
        assert_eq!(short_label(&r), None);
        set(&mut r, 1.0, Units::Centimeters, 1.0, Units::Meters);
        assert_eq!(r.world_per_page(), 100.0);
        assert_eq!(matching_preset(&r).map(|p| p.label), Some("1:100"));
        assert_eq!(short_label(&r).as_deref(), Some("1:100"));
        assert_eq!(units_of(&r), (Units::Centimeters, Units::Meters));
        // 1:48 in metric units is not the inch-and-foot preset.
        set(&mut r, 1.0, Units::Millimeters, 48.0, Units::Millimeters);
        assert_eq!(matching_preset(&r), None);
        set(&mut r, 0.25, Units::Inches, 1.0, Units::Feet);
        assert_eq!(matching_preset(&r).map(|p| p.label), Some("1/4 in = 1 ft"));
        assert_eq!(short_label(&r).as_deref(), Some("1:48"));
        // Enlarging scales read the other way round.
        set(&mut r, 5.0, Units::Millimeters, 1.0, Units::Millimeters);
        assert_eq!(short_label(&r).as_deref(), Some("5:1"));
        // Nonsense leaves the scale alone.
        set(&mut r, 0.0, Units::Millimeters, 1.0, Units::Millimeters);
        set(
            &mut r,
            1.0,
            Units::Millimeters,
            f64::NAN,
            Units::Millimeters,
        );
        assert_eq!(short_label(&r).as_deref(), Some("5:1"));
    }

    #[test]
    fn zoom_100_follows_the_ruler_calibration() {
        let mut app = crate::app::App::headless();
        app.canvas_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(800.0, 600.0));
        // 96 pixels per inch at the system's scaling are 96 interface
        // points at 100 %, where a point is such a pixel.
        let base = crate::ui::BASE_ZOOM;
        app.set_zoom_percent(100.0);
        assert!((app.view.zoom - 96.0 / base / 25.4).abs() < 1e-4);
        app.settings.screen_ppi = 120.0;
        assert!((app.zoom_percent() - 80.0).abs() < 1e-3);
        app.set_zoom_percent(100.0);
        assert!((app.view.zoom - 120.0 / base / 25.4).abs() < 1e-4);
        // A larger interface leaves real sizes real: at 125 % a point is
        // 1.25 times as large, so there are fewer of them per millimetre.
        app.settings.ui_size = 1.25;
        app.set_zoom_percent(100.0);
        assert!((app.view.zoom - 120.0 / (1.25 * base) / 25.4).abs() < 1e-4);
        // Out of range values are kept within 48 to 400.
        app.settings.screen_ppi = 5000.0;
        assert_eq!(crate::settings::screen_ppi(&app.settings), 400.0);
        app.settings.screen_ppi = f32::NAN;
        assert_eq!(crate::settings::screen_ppi(&app.settings), 96.0);
    }

    #[test]
    fn world_distances_go_through_the_scale() {
        crate::app::set_drawing_scale(100.0);
        // 10 page mm at 1:100 are a metre.
        assert!((Units::Meters.world_from_mm(10.0) - 1.0).abs() < 1e-12);
        assert!((Units::Meters.world_to_mm(2.5) - 25.0).abs() < 1e-12);
        // Physical sizes do not change.
        assert!((Units::Millimeters.from_mm(10.0) - 10.0).abs() < 1e-12);
        crate::app::set_drawing_scale(-3.0);
        assert_eq!(crate::app::drawing_scale(), 1.0);
        assert!((Units::Millimeters.world_from_mm(10.0) - 10.0).abs() < 1e-12);
    }
}
