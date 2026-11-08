//! Image > Resample: a new pixel size for the selected bitmaps, set as
//! pixels, a percentage or a resolution, with the aspect ratio kept or
//! not, and the way pixels are worked out (smooth, sharp, hard pixels for
//! pixel art, or smooth with the detail sharpened again after enlarging).

use crate::app::App;
use image::RgbaImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Method {
    /// Lanczos: smooth, no stair steps (photos, most pictures).
    #[default]
    Smooth,
    /// Catmull-Rom: a little crisper.
    Sharp,
    /// Nearest pixel: hard pixels kept (pixel art, icons, screenshots).
    Pixels,
    /// Lanczos, then an unsharp mask scaled with the enlargement, to win
    /// back the edges an enlargement softens.
    Detail,
}

impl Method {
    pub const ALL: [Method; 4] = [
        Method::Smooth,
        Method::Sharp,
        Method::Pixels,
        Method::Detail,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Method::Smooth => "resample.smooth",
            Method::Sharp => "resample.sharp",
            Method::Pixels => "resample.pixels",
            Method::Detail => "resample.detail",
        }
    }
}

/// The picture at `w` x `h` pixels.
pub fn resample(img: &RgbaImage, w: u32, h: u32, method: Method) -> RgbaImage {
    use image::imageops::{resize, FilterType};
    let (w, h) = (w.clamp(1, 20_000), h.clamp(1, 20_000));
    match method {
        Method::Smooth => resize(img, w, h, FilterType::Lanczos3),
        Method::Sharp => resize(img, w, h, FilterType::CatmullRom),
        Method::Pixels => resize(img, w, h, FilterType::Nearest),
        Method::Detail => {
            let out = resize(img, w, h, FilterType::Lanczos3);
            let grow =
                (w as f32 / img.width().max(1) as f32).max(h as f32 / img.height().max(1) as f32);
            if grow <= 1.05 {
                return out;
            }
            // The blur an enlargement adds grows with it.
            let sigma = (0.6 * grow).clamp(0.6, 4.0);
            let amount = (0.35 + 0.25 * grow.ln()).clamp(0.35, 1.2);
            crate::fx::noise::unsharp(&out, sigma, amount, 2.0, true)
        }
    }
}

/// The dialog's fields.
#[derive(Debug, Clone, PartialEq)]
pub struct ResampleState {
    /// The first selected bitmap's pixel size and size on the page (mm).
    pub orig: (u32, u32),
    pub size_mm: (f64, f64),
    pub w: u32,
    pub h: u32,
    pub keep_aspect: bool,
    pub method: Method,
}

impl ResampleState {
    pub fn new(orig: (u32, u32), size_mm: (f64, f64)) -> Self {
        ResampleState {
            orig: (orig.0.max(1), orig.1.max(1)),
            size_mm,
            w: orig.0.max(1),
            h: orig.1.max(1),
            keep_aspect: true,
            method: Method::Smooth,
        }
    }

    /// Set the width; with the aspect kept, the height follows.
    pub fn set_w(&mut self, w: u32) {
        self.w = w.clamp(1, 20_000);
        if self.keep_aspect {
            self.h = ((self.w as f64 * self.orig.1 as f64 / self.orig.0 as f64).round() as u32)
                .clamp(1, 20_000);
        }
    }

    pub fn set_h(&mut self, h: u32) {
        self.h = h.clamp(1, 20_000);
        if self.keep_aspect {
            self.w = ((self.h as f64 * self.orig.0 as f64 / self.orig.1 as f64).round() as u32)
                .clamp(1, 20_000);
        }
    }

    /// Percent of the original width and height.
    pub fn percent(&self) -> (f64, f64) {
        (
            self.w as f64 / self.orig.0 as f64 * 100.0,
            self.h as f64 / self.orig.1 as f64 * 100.0,
        )
    }

    pub fn set_percent(&mut self, px: f64, py: f64) {
        self.w = ((self.orig.0 as f64 * px / 100.0).round() as u32).clamp(1, 20_000);
        self.h = ((self.orig.1 as f64 * if self.keep_aspect { px } else { py } / 100.0).round()
            as u32)
            .clamp(1, 20_000);
    }

    /// The resolution the new width gives at the size on the page.
    pub fn dpi(&self) -> f64 {
        self.w as f64 / (self.size_mm.0 / 25.4).max(1e-6)
    }

    pub fn set_dpi(&mut self, dpi: f64) {
        let dpi = dpi.clamp(1.0, 4800.0);
        self.w = ((self.size_mm.0 / 25.4 * dpi).round() as u32).clamp(1, 20_000);
        self.h = ((self.size_mm.1 / 25.4 * dpi).round() as u32).clamp(1, 20_000);
        if self.keep_aspect {
            self.set_w(self.w);
        }
    }

    /// Memory of the new picture, bytes (4 per pixel).
    pub fn bytes(&self) -> u64 {
        self.w as u64 * self.h as u64 * 4
    }
}

impl App {
    /// Image > Resample: the dialog, set from the first selected bitmap.
    pub fn open_resample(&mut self) {
        let Some((_, rect, w, h, _)) = self.bitmap_shapes().into_iter().next() else {
            return;
        };
        self.dialog = crate::ui::dialogs::Dialog::Resample(ResampleState::new(
            (w, h),
            (rect.width(), rect.height()),
        ));
    }

    /// Resample every selected bitmap by the dialog's scale (the first
    /// one gets exactly the size typed), keeping its place and size on
    /// the page. One undo step per bitmap.
    pub fn resample_bitmaps(&mut self, st: &ResampleState) {
        let sx = st.w as f64 / st.orig.0 as f64;
        let sy = st.h as f64 / st.orig.1 as f64;
        for (id, rect, w, h, png) in self.bitmap_shapes() {
            let Some(img) = crate::bitmap_fx::decode(&png) else {
                continue;
            };
            let nw = ((w.max(1) as f64 * sx).round() as u32).max(1);
            let nh = ((h.max(1) as f64 * sy).round() as u32).max(1);
            if (nw, nh) == (img.width(), img.height()) && st.method != Method::Detail {
                continue;
            }
            let out = resample(&img, nw, nh, st.method);
            self.replace_bitmap(id, rect, &out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn sizes_follow_pixels_percent_and_resolution() {
        let mut st = ResampleState::new((400, 200), (101.6, 50.8));
        assert!((st.dpi() - 100.0).abs() < 1e-9);
        st.set_w(800);
        assert_eq!((st.w, st.h), (800, 400));
        st.set_percent(50.0, 999.0);
        assert_eq!((st.w, st.h), (200, 100));
        st.set_dpi(300.0);
        assert_eq!((st.w, st.h), (1200, 600));
        st.keep_aspect = false;
        st.set_h(50);
        assert_eq!((st.w, st.h), (1200, 50));
        st.set_percent(10.0, 20.0);
        assert_eq!((st.w, st.h), (40, 40));
        assert_eq!(st.bytes(), 40 * 40 * 4);
        st.set_w(0);
        assert_eq!(st.w, 1);
    }

    #[test]
    fn methods_keep_pixels_hard_or_smooth_and_detail_sharpens() {
        // A two-pixel checker enlarged four times.
        let img = RgbaImage::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        });
        let hard = resample(&img, 8, 8, Method::Pixels);
        assert!(hard.pixels().all(|p| p.0[0] == 0 || p.0[0] == 255));
        let smooth = resample(&img, 8, 8, Method::Smooth);
        assert!(smooth.pixels().any(|p| p.0[0] > 10 && p.0[0] < 245));
        // A soft edge enlarged: Detail makes it steeper than Smooth.
        let ramp = RgbaImage::from_fn(16, 4, |x, _| {
            let v = if x < 8 { 40 } else { 215 };
            Rgba([v, v, v, 255])
        });
        let s = resample(&ramp, 64, 16, Method::Smooth);
        let d = resample(&ramp, 64, 16, Method::Detail);
        let spread = |im: &RgbaImage| {
            let row: Vec<i32> = (0..im.width())
                .map(|x| im.get_pixel(x, 8).0[0] as i32)
                .collect();
            row.iter().max().copied().unwrap_or(0) - row.iter().min().copied().unwrap_or(0)
        };
        assert!(spread(&d) >= spread(&s), "{} {}", spread(&d), spread(&s));
        assert_eq!((d.width(), d.height()), (64, 16));
    }
}
