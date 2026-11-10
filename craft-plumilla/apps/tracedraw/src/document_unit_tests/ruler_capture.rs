//! Test-only CPU inspection of actual egui meshes and its actual font atlas.
//! This is an artifact for visual review, not a GPU screenshot or pixel oracle.
use egui::{
    epaint::{ImageData, Primitive},
    ColorImage, TextureId,
};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(1);

#[derive(Default)]
pub struct Capture {
    atlas: Option<ColorImage>,
}

impl Capture {
    pub fn frame(&mut self, ctx: &egui::Context, output: &egui::FullOutput, save: bool) {
        let Some(directory) = std::env::var_os("CRAFT_PLUMILLA_RULER_CAPTURE_DIR") else {
            return;
        };
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap()
            .canonicalize()
            .unwrap();
        let directory = PathBuf::from(directory)
            .canonicalize()
            .expect("capture directory must already exist");
        assert!(
            directory.starts_with(&workspace),
            "capture directory must remain inside the workspace"
        );
        for (id, patches) in &output.textures_delta.set {
            assert_eq!(*id, TextureId::Managed(0), "unexpected ruler texture");
            for delta in patches {
                let ImageData::Color(image) = &delta.image;
                if let Some([x, y]) = delta.pos {
                    let atlas = self
                        .atlas
                        .as_mut()
                        .expect("partial texture requires an existing atlas");
                    assert!(
                        x + image.size[0] <= atlas.size[0] && y + image.size[1] <= atlas.size[1]
                    );
                    for row in 0..image.size[1] {
                        let at = (y + row) * atlas.size[0] + x;
                        atlas.pixels[at..at + image.size[0]].copy_from_slice(
                            &image.pixels[row * image.size[0]..(row + 1) * image.size[0]],
                        );
                    }
                } else {
                    self.atlas = Some(image.as_ref().clone());
                }
            }
        }
        if !save {
            return;
        }
        assert!(
            (output.pixels_per_point - 1.0).abs() < 1e-6,
            "capture requires one pixel per point"
        );
        let atlas = self
            .atlas
            .as_ref()
            .expect("real font texture must be present");
        let mut image = image::RgbaImage::from_pixel(800, 600, image::Rgba([255, 255, 255, 255]));
        for clipped in ctx.tessellate(output.shapes.clone(), output.pixels_per_point) {
            let Primitive::Mesh(mesh) = clipped.primitive else {
                panic!("unsupported callback in ruler capture");
            };
            assert_eq!(
                mesh.texture_id,
                TextureId::Managed(0),
                "unknown mesh texture"
            );
            assert_eq!(mesh.indices.len() % 3, 0);
            for indices in mesh.indices.chunks_exact(3) {
                let v = [
                    mesh.vertices[indices[0] as usize],
                    mesh.vertices[indices[1] as usize],
                    mesh.vertices[indices[2] as usize],
                ];
                let [a, b, c] = v.map(|v| v.pos);
                let denominator = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
                if denominator.abs() < 1e-8 {
                    continue;
                }
                let left =
                    a.x.min(b.x)
                        .min(c.x)
                        .max(clipped.clip_rect.min.x)
                        .floor()
                        .max(0.0) as u32;
                let right =
                    a.x.max(b.x)
                        .max(c.x)
                        .min(clipped.clip_rect.max.x)
                        .ceil()
                        .clamp(0.0, 800.0) as u32;
                let top =
                    a.y.min(b.y)
                        .min(c.y)
                        .max(clipped.clip_rect.min.y)
                        .floor()
                        .max(0.0) as u32;
                let bottom =
                    a.y.max(b.y)
                        .max(c.y)
                        .min(clipped.clip_rect.max.y)
                        .ceil()
                        .clamp(0.0, 600.0) as u32;
                for y in top..bottom {
                    for x in left..right {
                        let p = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
                        if !clipped.clip_rect.contains(p) {
                            continue;
                        }
                        let w0 =
                            ((b.y - c.y) * (p.x - c.x) + (c.x - b.x) * (p.y - c.y)) / denominator;
                        let w1 =
                            ((c.y - a.y) * (p.x - c.x) + (a.x - c.x) * (p.y - c.y)) / denominator;
                        let weights = [w0, w1, 1.0 - w0 - w1];
                        if weights.iter().any(|w| *w < -1e-6) {
                            continue;
                        }
                        let uv = v
                            .iter()
                            .zip(weights)
                            .fold(egui::Vec2::ZERO, |sum, (v, w)| sum + v.uv.to_vec2() * w);
                        let sample = sample(atlas, uv);
                        let mut tint = [0.0; 4];
                        for (vertex, w) in v.iter().zip(weights) {
                            for (channel, c) in tint.iter_mut().zip(vertex.color.to_array()) {
                                *channel += w * c as f32 / 255.0;
                            }
                        }
                        let alpha = (sample[3] * tint[3]).clamp(0.0, 1.0);
                        let target = image.get_pixel_mut(x, y);
                        for channel in 0..3 {
                            target[channel] = ((sample[channel] * tint[channel]
                                + target[channel] as f32 / 255.0 * (1.0 - alpha))
                                * 255.0)
                                .round()
                                .clamp(0.0, 255.0)
                                as u8;
                        }
                        target[3] = 255;
                    }
                }
            }
        }
        let number = NEXT.fetch_add(1, Ordering::Relaxed);
        image
            .save(directory.join(format!("rulers-{number:03}.png")))
            .expect("save actual egui ruler capture");
    }
}

fn sample(atlas: &ColorImage, uv: egui::Vec2) -> [f32; 4] {
    let x = (uv.x * atlas.size[0] as f32 - 0.5).clamp(0.0, (atlas.size[0] - 1) as f32);
    let y = (uv.y * atlas.size[1] as f32 - 0.5).clamp(0.0, (atlas.size[1] - 1) as f32);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(atlas.size[0] - 1);
    let y1 = (y0 + 1).min(atlas.size[1] - 1);
    let dx = x - x0 as f32;
    let dy = y - y0 as f32;
    let mut out = [0.0; 4];
    for ((x, y), w) in [
        ((x0, y0), (1.0 - dx) * (1.0 - dy)),
        ((x1, y0), dx * (1.0 - dy)),
        ((x0, y1), (1.0 - dx) * dy),
        ((x1, y1), dx * dy),
    ] {
        for (target, value) in out
            .iter_mut()
            .zip(atlas.pixels[y * atlas.size[0] + x].to_array())
        {
            *target += w * value as f32 / 255.0;
        }
    }
    out
}
