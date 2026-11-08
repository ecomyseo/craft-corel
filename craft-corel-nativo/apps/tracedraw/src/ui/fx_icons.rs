//! Pictures for the bitmap effects in the Effects menu: each effect with
//! its default settings applied to a small landscape (sky, sun, hills and
//! a house), so the row shows what the effect does.

use image::{Rgba, RgbaImage};

/// Side of the sample picture, pixels (drawn at 16 interface points).
const SIDE: u32 = 24;

/// The sample: a sky fading to the horizon, a sun, a green hill and a
/// house with a red roof.
#[cfg(test)]
pub fn sample() -> RgbaImage {
    sample_sized(SIDE)
}

/// The sample at `side` pixels.
pub fn sample_sized(side: u32) -> RgbaImage {
    let side = side.max(4);
    let s = side as f32;
    RgbaImage::from_fn(side, side, |x, y| {
        let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
        let hill = s * 0.62 + ((fx / s) * 3.2).sin() * s * 0.06;
        let sun = ((fx - s * 0.72).powi(2) + (fy - s * 0.26).powi(2)).sqrt();
        let in_house = (s * 0.18..s * 0.42).contains(&fx) && (s * 0.56..s * 0.80).contains(&fy);
        let roof_mid = s * 0.30;
        let in_roof =
            fy >= s * 0.40 && fy < s * 0.56 && (fx - roof_mid).abs() < (fy - s * 0.40) * 0.9 + 1.0;
        let in_door = (s * 0.27..s * 0.33).contains(&fx) && (s * 0.68..s * 0.80).contains(&fy);
        let c: [u8; 3] = if in_door {
            [70, 45, 30]
        } else if in_roof {
            [200, 40, 40]
        } else if in_house {
            [240, 230, 205]
        } else if sun < s * 0.12 {
            [255, 205, 40]
        } else if fy > hill {
            let t = ((fy - hill) / (s - hill)).clamp(0.0, 1.0);
            [
                (70.0 - 30.0 * t) as u8,
                (170.0 - 60.0 * t) as u8,
                (60.0 - 20.0 * t) as u8,
            ]
        } else {
            let t = (fy / hill).clamp(0.0, 1.0);
            [
                (60.0 + 140.0 * t) as u8,
                (120.0 + 100.0 * t) as u8,
                (220.0 + 30.0 * t) as u8,
            ]
        };
        Rgba([c[0], c[1], c[2], 255])
    })
}

/// The sample with effect `id` applied with its default settings, cut back
/// to the sample's size when the effect grew it.
#[cfg(test)]
pub fn picture(id: &str) -> RgbaImage {
    picture_sized(id, SIDE)
}

/// The picture at `side` pixels.
pub fn picture_sized(id: &str, side: u32) -> RgbaImage {
    let src = sample_sized(side);
    let side = src.width();
    let out = crate::fx::apply_effect(&src, &crate::fx::new_effect(id));
    if out.width() == side && out.height() == side {
        return out;
    }
    // Effects that reach past the edges grow the picture around it.
    let dx = out.width().saturating_sub(side) / 2;
    let dy = out.height().saturating_sub(side) / 2;
    RgbaImage::from_fn(side, side, |x, y| {
        out.get_pixel_checked(x + dx, y + dy)
            .copied()
            .unwrap_or(Rgba([0, 0, 0, 0]))
    })
}

/// The texture of effect `id`'s picture, made the first time it is asked
/// for and kept for the session.
pub fn texture(ctx: &egui::Context, id: &str) -> Option<egui::TextureId> {
    texture_sized(ctx, id, SIDE, true)
}

/// The texture of effect `id`'s picture at `side` pixels; made now only
/// when `make` (a caller drawing many can spread the work over frames).
pub fn texture_sized(
    ctx: &egui::Context,
    id: &str,
    side: u32,
    make: bool,
) -> Option<egui::TextureId> {
    let key = egui::Id::new(("fx_icon", id, side));
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(key)) {
        return Some(t.id());
    }
    if !make {
        return None;
    }
    let img = picture_sized(id, side);
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    );
    let handle = ctx.load_texture(
        format!("fx_icon_{id}_{side}"),
        color,
        egui::TextureOptions::LINEAR,
    );
    let tex = handle.id();
    ctx.data_mut(|d| d.insert_temp(key, handle));
    Some(tex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_effect_has_a_picture_of_the_sample_size() {
        let plain = sample();
        let mut changed = 0;
        let mut total = 0;
        for (_, group) in crate::fx::groups() {
            for fx in group.iter() {
                let p = picture(fx.id);
                assert_eq!((p.width(), p.height()), (SIDE, SIDE), "{}", fx.id);
                total += 1;
                if p.as_raw() != plain.as_raw() {
                    changed += 1;
                }
            }
        }
        // Nearly every effect changes the sample with its defaults.
        assert!(changed * 10 >= total * 8, "{changed} of {total}");
    }
}
