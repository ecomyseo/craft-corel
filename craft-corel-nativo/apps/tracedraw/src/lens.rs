//! Lens docker state.

use tracedraw_core::{live::Lens, Color};

#[derive(Debug, Clone, PartialEq)]
pub struct LensSettings {
    pub kind: usize,
    pub rate: f64,
    pub color: Color,
    pub color_to: Color,
    pub amount: f64,
    pub frozen: bool,
    pub remove_face: bool,
    /// Bitmap Effects lens: the effects, in order.
    pub effects: Vec<tracedraw_core::BitmapEffect>,
}

impl Default for LensSettings {
    fn default() -> Self {
        LensSettings {
            kind: 0,
            rate: 50.0,
            color: Color::WHITE,
            color_to: Color::BLACK,
            amount: 2.0,
            frozen: false,
            remove_face: false,
            effects: Vec::new(),
        }
    }
}

pub const LENS_KEYS: [&str; 13] = [
    "lens.none",
    "lens.brighten",
    "lens.color_add",
    "lens.color_limit",
    "lens.custom_color_map",
    "lens.fish_eye",
    "lens.heat_map",
    "lens.invert",
    "lens.magnify",
    "lens.tinted_grayscale",
    "lens.transparency",
    "lens.wireframe",
    "lens.bitmap_effects",
];

impl crate::app::App {
    /// Add bitmap effect `id` (default settings) to the selection's
    /// Bitmap Effects lens, making one when there is none.
    pub fn add_lens_effect(&mut self, id: &str) {
        use tracedraw_core::live::Effect;
        let current = self.selected_shapes().first().and_then(|s| {
            s.effects.iter().find_map(|e| match e {
                Effect::Lens(Lens::Effects { effects }) => Some(effects.clone()),
                _ => None,
            })
        });
        let mut effects = current.unwrap_or_default();
        effects.push(crate::fx::new_effect(id));
        self.push_effect(Effect::Lens(Lens::Effects { effects }), true);
    }
}

impl LensSettings {
    pub fn to_lens(&self) -> Option<Lens> {
        Some(match self.kind {
            1 => Lens::Brighten { rate: self.rate },
            2 => Lens::ColorAdd {
                color: self.color,
                rate: self.rate,
            },
            3 => Lens::ColorLimit {
                color: self.color,
                rate: self.rate,
            },
            4 => Lens::CustomColorMap {
                from: self.color_to,
                to: self.color,
                direction: 0,
            },
            5 => Lens::FishEye {
                rate: self.rate * 10.0,
            },
            6 => Lens::HeatMap {
                rotation: self.rate,
            },
            7 => Lens::Invert,
            8 => Lens::Magnify {
                amount: self.amount,
            },
            9 => Lens::TintedGrayscale { color: self.color },
            10 => Lens::Transparency {
                rate: self.rate,
                color: self.color,
            },
            11 => Lens::Wireframe {
                outline: Color::BLACK,
                fill: self.color,
            },
            12 => Lens::Effects {
                effects: self.effects.clone(),
            },
            _ => return None,
        })
    }

    pub fn from_lens(l: &Lens) -> Self {
        let mut s = LensSettings::default();
        match l {
            Lens::Brighten { rate } => {
                s.kind = 1;
                s.rate = *rate;
            }
            Lens::ColorAdd { color, rate } => {
                s.kind = 2;
                s.color = *color;
                s.rate = *rate;
            }
            Lens::ColorLimit { color, rate } => {
                s.kind = 3;
                s.color = *color;
                s.rate = *rate;
            }
            Lens::CustomColorMap { from, to, .. } => {
                s.kind = 4;
                s.color_to = *from;
                s.color = *to;
            }
            Lens::FishEye { rate } => {
                s.kind = 5;
                s.rate = rate / 10.0;
            }
            Lens::HeatMap { rotation } => {
                s.kind = 6;
                s.rate = *rotation;
            }
            Lens::Invert => s.kind = 7,
            Lens::Magnify { amount } => {
                s.kind = 8;
                s.amount = *amount;
            }
            Lens::TintedGrayscale { color } => {
                s.kind = 9;
                s.color = *color;
            }
            Lens::Transparency { rate, color } => {
                s.kind = 10;
                s.rate = *rate;
                s.color = *color;
            }
            Lens::Wireframe { fill, .. } => {
                s.kind = 11;
                s.color = *fill;
            }
            Lens::Effects { effects } => {
                s.kind = 12;
                s.effects = effects.clone();
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::live::Effect;

    #[test]
    fn effects_lens_settings_round_trip_and_effects_are_added_to_the_lens() {
        let mut s = LensSettings {
            kind: 12,
            ..Default::default()
        };
        s.effects.push(crate::fx::new_effect("invert_colors"));
        let lens = s.to_lens().expect("a lens");
        assert_eq!(LensSettings::from_lens(&lens), s);
        let mut app = crate::app::App::headless();
        let layer = app.active_layer().expect("a layer");
        let id = app.engine.new_shape_id();
        let shape = tracedraw_core::Shape::new(
            id,
            tracedraw_core::ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(0.0, 0.0, 20.0, 20.0),
                radius: 0.0,
                corners: None,
            },
        );
        app.run(tracedraw_core::Command::AddShape { layer, shape });
        app.select(vec![id]);
        app.add_lens_effect("gaussian_blur");
        app.add_lens_effect("invert_colors");
        let effects = app
            .doc()
            .find_shape(id)
            .map(|s| s.effects.clone())
            .unwrap_or_default();
        assert_eq!(effects.len(), 1, "one lens");
        match &effects[0] {
            Effect::Lens(Lens::Effects { effects }) => {
                let ids: Vec<&str> = effects.iter().map(|e| e.id.as_str()).collect();
                assert_eq!(ids, ["gaussian_blur", "invert_colors"]);
            }
            e => panic!("{e:?}"),
        }
        // The application's effects over raw pixels: invert.
        let mut px = vec![10, 20, 30, 255];
        crate::fx::lens_effects(&[crate::fx::new_effect("invert_colors")], 1, 1, &mut px);
        assert_eq!(px, vec![245, 235, 225, 255]);
    }
}
