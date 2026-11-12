//! Object > Insert > Identify Barcode: the selected picture is read for a
//! QR code and the text it holds goes to the status line and the clipboard
//! (see docs/behavior/qr-code.md).

use crate::app::App;
use crate::i18n::{tr, trf};
use tracedraw_core::ShapeKind;

/// The text of the first QR code found in the picture `png`, `None` when
/// there is none or it cannot be read.
pub fn read_qr(png: &[u8]) -> Option<String> {
    let img = image::load_from_memory(png).ok()?.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(img);
    prepared
        .detect_grids()
        .into_iter()
        .find_map(|g| g.decode().ok().map(|(_, text)| text))
}

impl App {
    /// Whether Identify Barcode has a picture to read.
    pub fn can_identify_barcode(&self) -> bool {
        self.selected_bitmap().is_some()
    }

    /// Identify Barcode: reads the selected picture; the text read goes to
    /// the status line and the clipboard. Returns it.
    pub fn identify_barcode(&mut self, ctx: Option<&egui::Context>) -> Option<String> {
        let id = self.selected_bitmap()?;
        let png = match &self.doc().find_shape(id)?.kind {
            ShapeKind::Bitmap { png, .. } => png.clone(),
            _ => return None,
        };
        match read_qr(&png) {
            Some(text) => {
                if let Some(ctx) = ctx {
                    ctx.copy_text(text.clone());
                }
                self.status = trf("status.barcode_read", &[("text", &text)]);
                Some(text)
            }
            None => {
                self.status = tr("status.barcode_none");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Rect;

    /// A picture of the QR code of `payload`, as a PNG.
    fn qr_png(payload: &str) -> Vec<u8> {
        let (n, dark) = crate::qr::modules(payload).expect("modules");
        let scale = 6usize;
        let q = 4usize;
        let side = ((n + 2 * q) * scale) as u32;
        let img = image::GrayImage::from_fn(side, side, |x, y| {
            let (mx, my) = (x as usize / scale, y as usize / scale);
            let on = mx >= q && my >= q && mx - q < n && my - q < n && dark[(my - q) * n + (mx - q)];
            image::Luma([if on { 0 } else { 255 }])
        });
        let mut out = Vec::new();
        image::DynamicImage::ImageLuma8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .expect("png");
        out
    }

    fn bitmap(app: &mut App, png: Vec<u8>) {
        let id = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(0.0, 0.0, 40.0, 40.0),
                width_px: 100,
                height_px: 100,
                png: png.into(),
                fx: None,
            })
            .expect("bitmap");
        app.select(vec![id]);
    }

    #[test]
    fn a_qr_picture_is_read_and_a_plain_one_is_not() {
        let mut app = App::headless();
        assert!(!app.can_identify_barcode());
        bitmap(&mut app, qr_png("https://example.com/a?b=1"));
        assert!(app.can_identify_barcode());
        assert_eq!(app.identify_barcode(None).as_deref(), Some("https://example.com/a?b=1"));
        assert!(app.status.contains("https://example.com/a?b=1"), "{}", app.status);
        let blank = {
            let img = image::GrayImage::from_pixel(60, 60, image::Luma([255]));
            let mut out = Vec::new();
            image::DynamicImage::ImageLuma8(img)
                .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
                .expect("png");
            out
        };
        bitmap(&mut app, blank);
        app.status.clear();
        assert_eq!(app.identify_barcode(None), None);
        assert_ne!(app.status, "");
        assert!(!app.status.contains("example.com"));
    }
}
