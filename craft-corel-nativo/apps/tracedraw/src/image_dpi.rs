//! Resolution metadata is shared with the command-line raster importer.
pub use tracedraw_io::image_dpi::{resolution, DEFAULT_DPI};

#[cfg(test)]
mod tests {
    fn png_with_phys(ppm: u32, _unit: u8) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(4, 2, image::Rgba([10, 20, 30, 255]));
        tracedraw_io::raster::encode_rgba(
            &img,
            "png",
            &tracedraw_io::raster::RasterOptions {
                dpi: f64::from(ppm) * 0.0254,
                ..Default::default()
            },
        )
        .expect("encode")
    }

    fn encode(format: image::ImageFormat) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(4, 2, image::Rgba([10, 20, 30, 255]));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, format).expect("encode");
        out.into_inner()
    }

    #[test]
    fn import_places_a_bitmap_at_its_resolution() {
        let dir = std::env::temp_dir().join(format!("tracedraw-dpi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let at_300 = dir.join("at300.png");
        std::fs::write(&at_300, png_with_phys(11811, 1)).expect("write");
        let plain = dir.join("plain.png");
        std::fs::write(&plain, encode(image::ImageFormat::Png)).expect("write");
        let mut app = crate::app::App::headless();
        let width = |app: &mut crate::app::App, path: &std::path::Path| {
            app.import_bitmap(path);
            let s = app.selected_shapes();
            match &s[0].kind {
                tracedraw_core::ShapeKind::Bitmap { rect, .. } => rect.width(),
                other => panic!("not a bitmap: {other:?}"),
            }
        };
        // 4 pixels at 300 dpi, and at 96 dpi when the file says nothing.
        let w = width(&mut app, &at_300);
        assert!((w - 4.0 * 25.4 / 300.0).abs() < 1e-3, "{w}");
        let w = width(&mut app, &plain);
        assert!((w - 4.0 * 25.4 / 96.0).abs() < 1e-9, "{w}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
