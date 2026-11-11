//! Bitmap effects the reference's Effects menu has that the first groups
//! did not: each group module lists them in the place the reference's
//! menu configuration gives them (see docs/behavior/bitmap-effects.md).

use super::util::*;
use super::*;
use image::{Rgba, RgbaImage};
use std::f32::consts::{PI, TAU};

fn smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn px(q: &Rgba<u8>) -> [f32; 3] {
    [q[0] as f32, q[1] as f32, q[2] as f32]
}

fn out(c: [f32; 3], a: u8) -> Rgba<u8> {
    Rgba([clamp8(c[0]), clamp8(c[1]), clamp8(c[2]), a])
}

fn lum(c: [f32; 3]) -> f32 {
    0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]
}

/// A soft spot: 1 at the centre, 0 from `reach` pixels out.
fn spot(x: f32, y: f32, cx: f32, cy: f32, reach: f32) -> f32 {
    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / reach.max(1.0);
    (1.0 - d * d).max(0.0).powi(2)
}

fn centre(img: &RgbaImage, p: &P) -> (f32, f32) {
    (
        p.f32("center_x") / 100.0 * img.width() as f32,
        p.f32("center_y") / 100.0 * img.height() as f32,
    )
}

fn larger_side(img: &RgbaImage) -> f32 {
    img.width().max(img.height()).max(1) as f32
}

/// The picture lit by a height field: flat 1, its slopes brighter or
/// darker by `strength` (0..1).
fn relief(img: &RgbaImage, h: &[f32], strength: f32, dark: impl Fn(u32, u32) -> f32) -> RgbaImage {
    let (w, hh) = img.dimensions();
    let s = shade(h, w, hh, 135.0, 6.0);
    map_px(img, |x, y, q| {
        let i = (y * w + x) as usize;
        let f = (1.0 + (s[i] - 0.5) * 2.0 * strength) * (1.0 - dark(x, y) * strength);
        let c = px(q);
        out([c[0] * f, c[1] * f, c[2] * f], q[3])
    })
}

/// 3D Effects > Zigzag: rings of waves out from a centre, like a stone
/// dropped in a pond, fading towards the edges.
pub fn zigzag(img: &RgbaImage, p: &P) -> RgbaImage {
    let (cx, cy) = centre(img, p);
    let amp = p.f32("amplitude");
    let period = p.f32("period").max(2.0);
    let fade = larger_side(img);
    remap(img, false, |x, y| {
        let (dx, dy) = (x - cx, y - cy);
        let r = (dx * dx + dy * dy).sqrt();
        if r < 1e-3 {
            return (x, y);
        }
        let d = amp * (TAU * r / period).sin() * (1.0 - r / fade).max(0.0);
        (x + dx / r * d, y + dy / r * d)
    })
}

/// Adjust > Shadows/Midtones/Highlights: each tonal range made lighter
/// or darker on its own.
pub fn shadows_midtones_highlights(img: &RgbaImage, p: &P) -> RgbaImage {
    let (sh, mt, hl) = (
        p.f32("shadows") / 100.0,
        p.f32("midtones") / 100.0,
        p.f32("highlights") / 100.0,
    );
    map_px(img, |_, _, q| {
        let c = px(q);
        let l = lum(c) / 255.0;
        let d = 0.4
            * 255.0
            * (sh * (1.0 - smooth(0.0, 0.5, l))
                + mt * (1.0 - (2.0 * l - 1.0).abs())
                + hl * smooth(0.5, 1.0, l));
        out([c[0] + d, c[1] + d, c[2] + d], q[3])
    })
}

/// Adjust > Tint/Temperature: warmer or cooler, greener or more magenta.
pub fn tint_temperature(img: &RgbaImage, p: &P) -> RgbaImage {
    let t = p.f32("temperature") / 100.0;
    let g = p.f32("tint") / 100.0;
    map_px(img, |_, _, q| {
        let c = px(q);
        // Higher temperature cools (more blue), as in Image Adjustments.
        out(
            [
                c[0] * (1.0 - 0.25 * t) * (1.0 - 0.1 * g),
                c[1] * (1.0 + 0.2 * g),
                c[2] * (1.0 + 0.25 * t) * (1.0 - 0.1 * g),
            ],
            q[3],
        )
    })
}

/// Adjust > Grayscale: grey made from the colours with a weight for each
/// of the six hue families (reds, yellows, greens, cyans, blues and
/// magentas), so two colours of one brightness can come out different.
pub fn grayscale(img: &RgbaImage, p: &P) -> RgbaImage {
    let w = [
        p.f32("red") / 100.0,
        p.f32("green") / 100.0,
        p.f32("blue") / 100.0,
    ];
    let (yellow, cyan, magenta) = (
        p.f32("yellow") / 100.0,
        p.f32("cyan") / 100.0,
        p.f32("magenta") / 100.0,
    );
    map_px(img, |_, _, q| {
        let c = px(q);
        let mut i = [0usize, 1, 2];
        i.sort_by(|a, b| c[*b].total_cmp(&c[*a]));
        let (hi, mid, lo) = (i[0], i[1], i[2]);
        let secondary = match (hi.min(mid), hi.max(mid)) {
            (0, 1) => yellow,
            (1, 2) => cyan,
            _ => magenta,
        };
        let v = c[lo] + (c[hi] - c[mid]) * w[hi] + (c[mid] - c[lo]) * secondary;
        out([v, v, v], q[3])
    })
}

/// Adjust > Vibrance: more saturation where there is little, little more
/// where there is a lot, so skin and strong colours do not clip.
pub fn vibrance(img: &RgbaImage, p: &P) -> RgbaImage {
    let a = p.f32("amount") / 100.0;
    map_px(img, |_, _, q| {
        let c = px(q);
        let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
        let s = (mx - mn) / 255.0;
        let l = lum(c);
        let k = 1.0 + a * (1.0 - s);
        out(
            [l + (c[0] - l) * k, l + (c[1] - l) * k, l + (c[2] - l) * k],
            q[3],
        )
    })
}

/// Art Strokes > Dabble: the picture dabbed on with a brush, each dab
/// pulling the colour from a little way off in its own direction.
pub fn dabble(img: &RgbaImage, p: &P) -> RgbaImage {
    let size = p.f32("brush_size").max(1.0);
    let k = p.f32("coarseness") / 100.0 * size;
    let moved = remap(img, false, |x, y| {
        let a = value_noise(x, y, size * 2.0, 17) * TAU * 2.0;
        (x + a.cos() * k, y + a.sin() * k)
    });
    box_blur(&moved, (size / 3.0).round().max(1.0) as u32)
}

/// Zero-padded box blur of the alpha channel, `passes` times.
fn alpha_blur(img: &RgbaImage, r: usize, passes: usize) -> Vec<f32> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut a: Vec<f32> = img.pixels().map(|q| q[3] as f32).collect();
    let n = (2 * r + 1) as f32;
    for _ in 0..passes {
        let mut t = vec![0.0; a.len()];
        for y in 0..h {
            for x in 0..w {
                let (lo, hi) = (x.saturating_sub(r), (x + r).min(w - 1));
                t[y * w + x] = (lo..=hi).map(|k| a[y * w + k]).sum::<f32>() / n;
            }
        }
        for y in 0..h {
            for x in 0..w {
                let (lo, hi) = (y.saturating_sub(r), (y + r).min(h - 1));
                a[y * w + x] = (lo..=hi).map(|k| t[k * w + x]).sum::<f32>() / n;
            }
        }
    }
    a
}

/// Blur > Feather: the edges of the picture and of its transparent parts
/// fade out over the width.
pub fn feather(img: &RgbaImage, p: &P) -> RgbaImage {
    let r = (p.f32("width") / 2.0).round().max(1.0) as usize;
    let a = alpha_blur(img, r, 2);
    let w = img.width();
    map_px(img, |x, y, q| {
        let f = (a[(y * w + x) as usize] / 255.0 * 2.0 - 1.0).clamp(0.0, 1.0);
        Rgba([q[0], q[1], q[2], clamp8(q[3] as f32 * f)])
    })
}

/// Camera > Lens Flare: the glow, rays and rings a bright light makes in
/// a camera lens, screened over the picture.
pub fn lens_flare(img: &RgbaImage, p: &P) -> RgbaImage {
    let (cx, cy) = centre(img, p);
    let m = larger_side(img);
    let b = p.f32("brightness") / 100.0;
    let (mx, my) = (img.width() as f32 / 2.0, img.height() as f32 / 2.0);
    let ghosts = [(0.6, 0.04), (1.1, 0.07), (1.5, 0.03)];
    map_px(img, |x, y, q| {
        let (x, y) = (x as f32, y as f32);
        let (dx, dy) = (x - cx, y - cy);
        let d = (dx * dx + dy * dy).sqrt() / m;
        let glow = (-(d * 7.0).powi(2)).exp();
        let rays = (0.5f32 + 0.5 * (dy.atan2(dx) * 12.0).cos()).powi(8) * (-d * 6.0).exp() * 0.5;
        let mut rings = 0.0;
        for (t, size) in ghosts {
            let (gx, gy) = (cx + (mx - cx) * 2.0 * t, cy + (my - cy) * 2.0 * t);
            let g = ((x - gx).powi(2) + (y - gy).powi(2)).sqrt() / m;
            rings += (-((g - size) / 0.008).powi(2)).exp() * 0.35 + (g < size) as u8 as f32 * 0.06;
        }
        let add = (b * (glow + rays + rings)).min(1.0) * 255.0;
        let c = px(q);
        let tint = [1.0, 0.92, 0.75];
        let s = |v: f32, k: usize| 255.0 - (255.0 - v) * (255.0 - add * tint[k]) / 255.0;
        out([s(c[0], 0), s(c[1], 1), s(c[2], 2)], q[3])
    })
}

/// Camera > Lighting Effects: a spotlight of a colour; the rest of the
/// picture keeps a little light.
pub fn lighting(img: &RgbaImage, p: &P) -> RgbaImage {
    let (cx, cy) = centre(img, p);
    let reach = p.f32("range") / 100.0 * larger_side(img);
    let k = p.f32("light_intensity") / 100.0;
    let lc = p.rgb("light_color").map(|v| v as f32 / 255.0);
    map_px(img, |x, y, q| {
        let f = 0.35 + 1.3 * k * spot(x as f32, y as f32, cx, cy, reach);
        let c = px(q);
        out([c[0] * f * lc[0], c[1] * f * lc[1], c[2] * f * lc[2]], q[3])
    })
}

/// Camera > Spot Filter: a coloured filter that is strongest at a point
/// and fades out around it.
pub fn spot_filter(img: &RgbaImage, p: &P) -> RgbaImage {
    let (cx, cy) = centre(img, p);
    let reach = p.f32("range") / 100.0 * larger_side(img);
    let density = p.f32("density") / 100.0;
    let fc = p.rgb("filter_color").map(|v| v as f32 / 255.0);
    map_px(img, |x, y, q| {
        let t = density * spot(x as f32, y as f32, cx, cy, reach);
        let c = px(q);
        let l = lum(c);
        let f = |k: usize| c[k] + (l * fc[k] * 1.4 - c[k]) * t;
        out([f(0), f(1), f(2)], q[3])
    })
}

/// Custom > Band Pass: only the details between two sizes kept (the
/// difference of two blurs), on mid grey; Percentage mixes it with the
/// picture.
pub fn band_pass(img: &RgbaImage, p: &P) -> RgbaImage {
    let inner = p.f32("inner_radius");
    let outer = p.f32("outer_radius").max(inner + 0.5);
    let (a, b) = (gaussian(img, inner), gaussian(img, outer));
    let t = p.f32("percentage") / 100.0;
    map_px(img, |x, y, q| {
        let (pa, pb) = (a.get_pixel(x, y), b.get_pixel(x, y));
        let c = px(q);
        let f = |k: usize| c[k] + ((pa[k] as f32 - pb[k] as f32) * 2.0 + 128.0 - c[k]) * t;
        out([f(0), f(1), f(2)], q[3])
    })
}

/// The kernel of User Defined, row by row.
pub const KERNEL: [&str; 9] = ["k11", "k12", "k13", "k21", "k22", "k23", "k31", "k32", "k33"];

/// Custom > User Defined: a 3 by 3 convolution of the user's, divided
/// and offset.
pub fn user_defined(img: &RgbaImage, p: &P) -> RgbaImage {
    let k: Vec<f32> = KERNEL.iter().map(|n| p.f32(n)).collect();
    let div = p.f32("divisor").max(1.0);
    let off = p.f32("offset");
    let (w, h) = (img.width() as i64, img.height() as i64);
    map_px(img, |x, y, q| {
        let mut s = [0f32; 3];
        for (i, kv) in k.iter().enumerate() {
            let (dx, dy) = (i as i64 % 3 - 1, i as i64 / 3 - 1);
            let sx = (x as i64 + dx).clamp(0, w - 1) as u32;
            let sy = (y as i64 + dy).clamp(0, h - 1) as u32;
            let c = px(img.get_pixel(sx, sy));
            for ch in 0..3 {
                s[ch] += c[ch] * kv;
            }
        }
        out([s[0] / div + off, s[1] / div + off, s[2] / div + off], q[3])
    })
}

/// Distort > Shear: each row moved sideways, the top one way and the
/// bottom the other, wrapping round.
pub fn shear(img: &RgbaImage, p: &P) -> RgbaImage {
    let (w, h) = (img.width() as f32, img.height() as f32);
    let a = p.f32("amount") / 100.0 * w;
    remap(img, false, |x, y| {
        let t = if h > 1.0 { y / (h - 1.0) - 0.5 } else { 0.0 };
        ((x - a * t).rem_euclid(w), y)
    })
}

/// Noise > 3D Stereo Noise: a random-dot stereogram whose depth is the
/// picture's brightness (looked at cross-eyed, the picture stands out).
pub fn three_d_stereo_noise(img: &RgbaImage, p: &P) -> RgbaImage {
    let (w, h) = img.dimensions();
    let pattern = p.f32("spacing").max(4.0);
    let depth = p.f32("depth");
    let mut res = RgbaImage::new(w, h);
    for y in 0..h {
        let mut row = vec![0u8; w as usize];
        for x in 0..w as usize {
            let z = luma(img.get_pixel(x as u32, y)) / 255.0;
            let s = (pattern - depth * z).round().max(2.0) as usize;
            row[x] = if x >= s {
                row[x - s]
            } else if hash01(x as u32, y, 29) > 0.5 {
                255
            } else {
                0
            };
            let a = img.get_pixel(x as u32, y)[3];
            res.put_pixel(x as u32, y, Rgba([row[x], row[x], row[x], a]));
        }
    }
    res
}

/// Texture > Brick Wall: the picture painted on bricks, the mortar
/// joints sunk and darker.
pub fn brick_wall(img: &RgbaImage, p: &P) -> RgbaImage {
    let (bw, bh) = (p.f32("width").max(4.0), p.f32("height").max(3.0));
    let joint = (bh / 8.0).max(1.0);
    let (w, hh) = img.dimensions();
    let mortar = |x: u32, y: u32| {
        let row = (y as f32 / bh).floor();
        let off = if row as i64 % 2 == 1 { bw / 2.0 } else { 0.0 };
        let bx = (x as f32 + off).rem_euclid(bw);
        let by = (y as f32).rem_euclid(bh);
        bx < joint || by < joint
    };
    let h: Vec<f32> = (0..hh)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| if mortar(x, y) { 0.0 } else { 0.8 + 0.2 * value_noise(x as f32, y as f32, 3.0, 5) })
        .collect();
    let k = p.f32("intensity") / 100.0;
    relief(img, &h, k, |x, y| if mortar(x, y) { 0.5 } else { 0.0 })
}

/// Texture > Bubbles: soap bubbles over the picture, each a bright ring
/// with a highlight.
pub fn bubbles(img: &RgbaImage, p: &P) -> RgbaImage {
    let size = p.f32("size").max(2.0);
    let density = p.f32("density") / 100.0;
    let cell = size * 1.5;
    map_px(img, |x, y, q| {
        let (fx, fy) = (x as f32, y as f32);
        let (ci, cj) = ((fx / cell).floor() as i64, (fy / cell).floor() as i64);
        let mut add = 0.0f32;
        for j in cj - 1..=cj + 1 {
            for i in ci - 1..=ci + 1 {
                let (ui, uj) = (i as u32, j as u32);
                if hash01(ui, uj, 41) >= density {
                    continue;
                }
                let bx = (i as f32 + hash01(ui, uj, 42)) * cell;
                let by = (j as f32 + hash01(ui, uj, 43)) * cell;
                let r = size * (0.5 + 0.5 * hash01(ui, uj, 44)) / 2.0;
                let d = ((fx - bx).powi(2) + (fy - by).powi(2)).sqrt();
                if d <= r + 1.5 {
                    let ring = (1.0 - ((d - r).abs() / 1.5)).max(0.0) * 0.7;
                    let hx = ((fx - (bx - r * 0.4)).powi(2) + (fy - (by - r * 0.4)).powi(2)).sqrt();
                    let shine = (1.0 - hx / (r * 0.25).max(1.0)).max(0.0) * 0.8;
                    add += ring + shine + if d < r { 0.08 } else { 0.0 };
                }
            }
        }
        let a = add.min(1.0) * 255.0;
        let c = px(q);
        let s = |v: f32| 255.0 - (255.0 - v) * (255.0 - a) / 255.0;
        out([s(c[0]), s(c[1]), s(c[2])], q[3])
    })
}

fn weave(x: f32, y: f32, size: f32) -> f32 {
    let (u, v) = (x / size, y / size);
    let over = (u.floor() as i64 + v.floor() as i64) % 2 == 0;
    let t = if over { (u.fract() * PI).sin() } else { (v.fract() * PI).sin() };
    0.6 * t + 0.4 * value_noise(x, y, 2.0, 11)
}

/// Texture > Canvas: the picture on a woven canvas.
pub fn canvas(img: &RgbaImage, p: &P) -> RgbaImage {
    let size = p.f32("size").max(2.0);
    let (w, hh) = img.dimensions();
    let h: Vec<f32> = (0..hh)
        .flat_map(|y| (0..w).map(move |x| weave(x as f32, y as f32, size)))
        .collect();
    relief(img, &h, p.f32("intensity") / 100.0, |_, _| 0.0)
}

/// Texture > Plaster Wall: the picture on rough plaster.
pub fn plaster_wall(img: &RgbaImage, p: &P) -> RgbaImage {
    let size = p.f32("size").max(2.0);
    let (w, hh) = img.dimensions();
    let h: Vec<f32> = (0..hh)
        .flat_map(|y| (0..w).map(move |x| fractal(x as f32, y as f32, size, 3)))
        .collect();
    relief(img, &h, p.f32("intensity") / 100.0, |_, _| 0.0)
}

/// Texture > Screen Door: the picture seen through a fine mesh.
pub fn screen_door(img: &RgbaImage, p: &P) -> RgbaImage {
    let s = p.u("spacing").max(2);
    let o = p.f32("opacity") / 100.0;
    map_px(img, |x, y, q| {
        if x % s == 0 || y % s == 0 {
            let c = px(q);
            out([c[0] * (1.0 - o), c[1] * (1.0 - o), c[2] * (1.0 - o)], q[3])
        } else {
            *q
        }
    })
}

/// Texture > Underpainting: the picture painted roughly on canvas, its
/// small detail smoothed away.
pub fn underpainting(img: &RgbaImage, p: &P) -> RgbaImage {
    let r = (p.f32("brush_size") / 2.0).round().max(1.0) as u32;
    let soft = box_blur(&rank(img, r, 0.5), r);
    let t = p.f32("texture") / 100.0;
    map_px(&soft, |x, y, q| {
        let f = 1.0 + (weave(x as f32, y as f32, 4.0) - 0.5) * t * 0.5;
        let c = px(q);
        out([c[0] * f, c[1] * f, c[2] * f], q[3])
    })
}

/// Transform > Threshold: black or white, at the level chosen.
pub fn threshold(img: &RgbaImage, p: &P) -> RgbaImage {
    let level = p.f32("level");
    map_px(img, |_, _, q| {
        let v = if luma(q) >= level { 255 } else { 0 };
        Rgba([v, v, v, q[3]])
    })
}

/// Blur > Tune Blur: the kinds of blur side by side as one effect.
pub fn tune_blur(img: &RgbaImage, p: &P) -> RgbaImage {
    let r = p.f32("radius").max(0.5);
    match p.i("method") {
        0 => gaussian(img, r),
        1 => box_blur(img, r.round().max(1.0) as u32),
        _ => mix(img, &gaussian(img, r), 0.5),
    }
}

/// Noise > Tune Noise: noise added evenly or in a bell curve, or taken
/// away with a median.
pub fn tune_noise(img: &RgbaImage, p: &P) -> RgbaImage {
    let a = p.f32("amount") / 100.0 * 128.0;
    match p.i("method") {
        2 => box_blur(img, 1),
        m => map_px(img, |x, y, q| {
            let n = if m == 0 {
                hash01(x, y, 53) - 0.5
            } else {
                (hash01(x, y, 53) + hash01(x, y, 54) + hash01(x, y, 55)) / 3.0 - 0.5
            } * 2.0
                * a;
            let c = px(q);
            out([c[0] + n, c[1] + n, c[2] + n], q[3])
        }),
    }
}

/// Correction > Tune Sharpness: unsharp masking with a Gaussian or an
/// average, or a softer adaptive one.
pub fn tune_sharpness(img: &RgbaImage, p: &P) -> RgbaImage {
    let k = p.f32("amount") / 100.0;
    let soft = match p.i("method") {
        1 => box_blur(img, 1),
        _ => gaussian(img, 1.0),
    };
    let k = if p.i("method") == 2 { k * 0.5 } else { k };
    map_px(img, |x, y, q| {
        let b = soft.get_pixel(x, y);
        let c = px(q);
        let f = |i: usize| c[i] + (c[i] - b[i] as f32) * k;
        out([f(0), f(1), f(2)], q[3])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn run(id: &str, img: &RgbaImage, set: &[(&str, f64)]) -> RgbaImage {
        let spec = super::super::spec(id).unwrap_or_else(|| panic!("{id} is in the catalogue"));
        let values: BTreeMap<String, f64> = set.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        (spec.apply)(img, &P { spec, values: &values })
    }

    fn flat(w: u32, h: u32, c: [u8; 3]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([c[0], c[1], c[2], 255]))
    }

    fn ramp() -> RgbaImage {
        RgbaImage::from_fn(64, 48, |x, y| {
            Rgba([(x * 4) as u8, (y * 5) as u8, ((x + y) * 2) as u8, 255])
        })
    }

    fn variance(img: &RgbaImage) -> f32 {
        let v: Vec<f32> = img.pixels().map(luma).collect();
        let m = v.iter().sum::<f32>() / v.len() as f32;
        v.iter().map(|x| (x - m).powi(2)).sum::<f32>() / v.len() as f32
    }

    fn sat(q: &Rgba<u8>) -> f32 {
        let c = px(q);
        c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
    }

    /// Each new effect does what its name says, measured on its result;
    /// at its "nothing" setting it leaves the picture as it was.
    #[test]
    fn the_new_effects_do_what_they_say() {
        let r = ramp();
        // Zigzag: still at the centre, moved elsewhere, nothing at 0.
        let z = run("zigzag", &r, &[]);
        assert_ne!(z, r);
        assert_eq!(run("zigzag", &r, &[("amplitude", 0.0)]), r);
        // Shadows lifted: dark pixels brighter, light ones barely.
        let s = run("shadows_midtones_highlights", &r, &[("shadows", 100.0)]);
        assert!(luma(s.get_pixel(1, 1)) > luma(r.get_pixel(1, 1)) + 20.0);
        assert!((luma(s.get_pixel(63, 47)) - luma(r.get_pixel(63, 47))).abs() < 3.0);
        // Cooler: more blue than red on grey.
        let g = flat(8, 8, [128, 128, 128]);
        let t = run("tint_temperature", &g, &[("temperature", 100.0)]);
        assert!(t.get_pixel(0, 0)[2] > t.get_pixel(0, 0)[0] + 40);
        // Grayscale: grey, and the red weight changes a red.
        let red = flat(4, 4, [220, 20, 20]);
        let lo = run("grayscale", &red, &[("red", 0.0)]).get_pixel(0, 0).0;
        let hi = run("grayscale", &red, &[("red", 100.0)]).get_pixel(0, 0).0;
        assert!(lo[0] == lo[1] && lo[1] == lo[2] && hi[0] > lo[0] + 100, "{lo:?} {hi:?}");
        // Vibrance: a dull colour gains more than a strong one.
        let two = RgbaImage::from_fn(2, 1, |x, _| if x == 0 { Rgba([140, 120, 110, 255]) } else { Rgba([240, 30, 20, 255]) });
        let v = run("vibrance", &two, &[("amount", 100.0)]);
        let gain = |x| sat(v.get_pixel(x, 0)) / sat(two.get_pixel(x, 0));
        assert!(gain(0) > gain(1) + 0.3, "{} {}", gain(0), gain(1));
        // Dabble changes the picture.
        assert_ne!(run("dabble", &r, &[]), r);
        // Feather: clear at the corner, solid in the middle.
        let f = run("feather", &flat(60, 60, [10, 200, 10]), &[("width", 20.0)]);
        assert_eq!(f.get_pixel(0, 0)[3], 0);
        assert_eq!(f.get_pixel(30, 30)[3], 255);
        // Lens flare and lighting: brighter at their centre than far away.
        let dark = flat(80, 60, [30, 30, 30]);
        let lf = run("lens_flare", &dark, &[("center_x", 25.0), ("center_y", 25.0)]);
        assert!(luma(lf.get_pixel(20, 15)) > luma(lf.get_pixel(79, 59)) + 100.0);
        let li = run("lighting", &flat(80, 60, [120, 120, 120]), &[("center_x", 25.0), ("center_y", 25.0)]);
        assert!(luma(li.get_pixel(20, 15)) > luma(li.get_pixel(79, 59)) + 60.0);
        // Spot filter: tinted at the spot, untouched far from it.
        let sf = run("spot_filter", &g, &[("range", 30.0), ("density", 100.0)]);
        assert_ne!(sf.get_pixel(4, 4), g.get_pixel(4, 4));
        let wide = flat(100, 100, [128, 128, 128]);
        assert_eq!(run("spot_filter", &wide, &[("range", 20.0)]).get_pixel(0, 0), wide.get_pixel(0, 0));
        // Band pass: a flat picture becomes mid grey; at 0 % nothing.
        assert_eq!(run("band_pass", &flat(20, 20, [200, 90, 30]), &[]).get_pixel(10, 10).0, [128, 128, 128, 255]);
        assert_eq!(run("band_pass", &r, &[("percentage", 0.0)]), r);
        // User defined: the identity kernel leaves it; a box kernel blurs.
        assert_eq!(run("user_defined", &r, &[]), r);
        let checker = RgbaImage::from_fn(10, 10, |x, y| if (x + y) % 2 == 0 { Rgba([0, 0, 0, 255]) } else { Rgba([255, 255, 255, 255]) });
        let mut box9: Vec<(&str, f64)> = KERNEL.iter().map(|k| (*k, 1.0)).collect();
        box9.push(("divisor", 9.0));
        assert!(variance(&run("user_defined", &checker, &box9)) < variance(&checker) / 10.0);
        // Shear: nothing at 0; top and bottom rows moved apart.
        assert_eq!(run("shear", &r, &[("amount", 0.0)]), r);
        let sh = run("shear", &r, &[("amount", 50.0)]);
        assert_ne!(sh.get_pixel(10, 0), r.get_pixel(10, 0));
        // 3D stereo noise: black and white dots repeating every pattern
        // width where the depth is 0.
        let st = run("three_d_stereo_noise", &flat(120, 4, [0, 0, 0]), &[("spacing", 30.0)]);
        assert!(st.pixels().all(|q| (q[0] == 0 || q[0] == 255) && q[0] == q[1] && q[1] == q[2]));
        assert!((30..120).all(|x| st.get_pixel(x, 2) == st.get_pixel(x - 30, 2)));
        // Brick wall: mortar darker than the middle of a brick.
        let bw = run("brick_wall", &flat(80, 40, [180, 180, 180]), &[("width", 40.0), ("height", 20.0)]);
        assert!(luma(bw.get_pixel(20, 0)) + 30.0 < luma(bw.get_pixel(20, 10)));
        // Bubbles add light; none at density 0.
        let lit = run("bubbles", &dark, &[("density", 100.0)]);
        assert!(lit.pixels().map(luma).fold(0.0, f32::max) > 150.0);
        assert_eq!(run("bubbles", &dark, &[("density", 0.0)]), dark);
        // Canvas and plaster give a flat picture a texture; none at 0.
        let mid = flat(40, 40, [128, 128, 128]);
        for id in ["canvas", "plaster_wall"] {
            assert!(variance(&run(id, &mid, &[])) > 2.0, "{id}");
            assert_eq!(run(id, &mid, &[("intensity", 0.0)]), mid, "{id}");
        }
        // Screen door: the mesh lines darker, the holes unchanged.
        let sd = run("screen_door", &mid, &[("spacing", 4.0), ("opacity", 50.0)]);
        assert_eq!(sd.get_pixel(0, 1)[0], 64);
        assert_eq!(sd.get_pixel(1, 1)[0], 128);
        // Underpainting smooths detail away.
        assert!(variance(&run("underpainting", &checker, &[("texture", 0.0)])) < variance(&checker) / 2.0);
        // Threshold: only black and white.
        assert!(run("threshold", &r, &[]).pixels().all(|q| q[0] == 0 || q[0] == 255));
        // Tune blur smooths, tune sharpness sharpens, tune noise adds noise
        // to a flat picture and its median takes it away.
        assert!(variance(&run("tune_blur", &checker, &[("radius", 2.0)])) < variance(&checker) / 4.0);
        let soft = run("tune_blur", &r, &[("radius", 3.0)]);
        assert!(variance(&run("tune_sharpness", &soft, &[("amount", 200.0)])) > variance(&soft));
        assert!(variance(&run("tune_noise", &mid, &[("amount", 50.0)])) > 50.0);
        assert_eq!(run("tune_noise", &mid, &[("method", 2.0)]), mid);
    }
}
