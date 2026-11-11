//! The palette menu's Sort Colors: a palette of the user's or the
//! document palette put in order by name, hue, saturation, brightness,
//! RGB or HSB value, or turned round (see docs/behavior/palettes.md).

use crate::app::App;
use tracedraw_core::{Color, Command};

/// What Sort Colors orders by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Name,
    Hue,
    Saturation,
    Brightness,
    Rgb,
    Hsb,
    Reverse,
}

impl SortBy {
    pub const ALL: [(SortBy, &'static str); 7] = [
        (SortBy::Hue, "palette.sort_hue"),
        (SortBy::Saturation, "palette.sort_saturation"),
        (SortBy::Brightness, "palette.sort_brightness"),
        (SortBy::Hsb, "palette.sort_hsb"),
        (SortBy::Rgb, "palette.sort_rgb"),
        (SortBy::Name, "palette.sort_name"),
        (SortBy::Reverse, "palette.sort_reverse"),
    ];
}

/// Hue (0..360), saturation and brightness (0..1) of a colour.
fn hsb(c: Color) -> (f64, f64, f64) {
    let [r, g, b] = c.to_rgb8().map(|v| v as f64 / 255.0);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let d = mx - mn;
    let h = if d <= 1e-12 {
        0.0
    } else if mx == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if mx == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    let s = if mx <= 1e-12 { 0.0 } else { d / mx };
    (h, s, mx)
}

/// `colors` in the order `by` gives (a stable sort: equal colours keep
/// their order).
pub fn sorted(colors: &[(String, Color)], by: SortBy) -> Vec<(String, Color)> {
    let mut v = colors.to_vec();
    match by {
        SortBy::Reverse => v.reverse(),
        SortBy::Name => v.sort_by_key(|a| a.0.to_lowercase()),
        SortBy::Rgb => v.sort_by_key(|a| a.1.to_rgb8()),
        _ => {
            let key = |c: Color| {
                let (h, s, b) = hsb(c);
                match by {
                    SortBy::Hue => (h, s, b),
                    SortBy::Saturation => (s, h, b),
                    SortBy::Brightness => (b, h, s),
                    _ => (h, s, b),
                }
            };
            v.sort_by(|a, b| {
                let (x, y) = (key(a.1), key(b.1));
                x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1)).then(x.2.total_cmp(&y.2))
            });
        }
    }
    v
}

impl App {
    /// Sort the user's palette `i`.
    pub fn sort_user_palette(&mut self, i: usize, by: SortBy) {
        if self.palette_locked(i) {
            return;
        }
        let Some(p) = self.palettes.get_mut(i) else {
            return;
        };
        p.colors = sorted(&p.colors, by);
        self.store_user_palettes();
        self.rebuild_palette();
    }

    /// Sort the document palette (one undo step).
    pub fn sort_document_palette(&mut self, by: SortBy) {
        let named: Vec<(String, Color)> = self.doc().palette.iter().map(|c| (String::new(), *c)).collect();
        let colors: Vec<Color> = sorted(&named, by).into_iter().map(|(_, c)| c).collect();
        if colors != self.doc().palette {
            self.run(Command::SetDocumentPalette { colors });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(c: [u8; 3], n: &str) -> (String, Color) {
        (n.to_string(), Color::rgb8(c[0], c[1], c[2]))
    }

    /// Each order measured on a few colours.
    #[test]
    fn the_palette_sorts_by_each_order() {
        let cols = vec![
            named([0, 0, 255], "blue"),
            named([255, 0, 0], "Red"),
            named([0, 255, 0], "green"),
            named([128, 128, 128], "grey"),
            named([40, 40, 40], "dark"),
        ];
        let names = |v: &[(String, Color)]| v.iter().map(|c| c.0.clone()).collect::<Vec<_>>();
        assert_eq!(names(&sorted(&cols, SortBy::Name)), ["blue", "dark", "green", "grey", "Red"]);
        assert_eq!(names(&sorted(&cols, SortBy::Reverse)), ["dark", "grey", "green", "Red", "blue"]);
        // Hue: the greys (no hue) first, then red 0, green 120, blue 240.
        assert_eq!(names(&sorted(&cols, SortBy::Hue))[2..], ["Red", "green", "blue"]);
        // Brightness: dark, grey, then the pure colours.
        assert_eq!(names(&sorted(&cols, SortBy::Brightness))[..2], ["dark", "grey"]);
        // A dark blue before a bright red by brightness, after it by hue.
        let two = vec![named([255, 0, 0], "red"), named([0, 0, 100], "navy")];
        assert_eq!(names(&sorted(&two, SortBy::Brightness)), ["navy", "red"]);
        assert_eq!(names(&sorted(&two, SortBy::Hue)), ["red", "navy"]);
        // Saturation: the greys before the pure colours.
        let s = names(&sorted(&cols, SortBy::Saturation));
        assert!(s[..2].contains(&"grey".to_string()) && s[..2].contains(&"dark".to_string()), "{s:?}");
        // RGB: by red, then green, then blue.
        assert_eq!(names(&sorted(&cols, SortBy::Rgb)), ["blue", "green", "dark", "grey", "Red"]);
    }

    /// New Palette from Selection: a new palette of the user's holding the
    /// selection's colours; nothing with nothing selected.
    #[test]
    fn a_new_palette_takes_the_selection_colours() {
        let mut app = App::headless();
        let id = app
            .new_shape(tracedraw_core::document::ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.select(vec![id]);
        app.apply_fill(tracedraw_core::Fill::Solid(Color::rgb8(200, 30, 40)));
        let n = app.palettes.len();
        let i = app.new_palette_from_selection().expect("a palette");
        assert_eq!(app.palettes.len(), n + 1);
        assert!(app.palettes[i].colors.iter().any(|(_, c)| c.to_rgb8() == [200, 30, 40]), "{:?}", app.palettes[i].colors);
        app.select(Vec::new());
        assert!(app.new_palette_from_selection().is_none());
    }

    /// The document palette is sorted in one undo step.
    #[test]
    fn the_document_palette_sorts_and_undoes() {
        let mut app = App::headless();
        let before = vec![Color::rgb8(250, 250, 250), Color::rgb8(10, 10, 10)];
        app.run(Command::SetDocumentPalette { colors: before.clone() });
        app.sort_document_palette(SortBy::Brightness);
        assert_eq!(app.doc().palette, vec![Color::rgb8(10, 10, 10), Color::rgb8(250, 250, 250)]);
        app.undo();
        assert_eq!(app.doc().palette, before);
    }
}
