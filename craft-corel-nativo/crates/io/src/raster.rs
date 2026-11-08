//! Shared raster import and export for the CLI and desktop. File suffixes
//! select a real codec; unknown formats are errors, never native JSON or PNG.
//!
//! PNG, TIFF and lossless WebP keep straight RGBA samples. JPEG and BMP
//! flatten transparency onto white; GIF has its palette and one-bit alpha.
//! CMYK TIFF import also keeps original 8-bit ink planes and their source ICC
//! profile per picture. Page export is still RGBA8, not a CMYK page compositor.
//! Other higher bit depths and animated/multipage raster data are not retained.

use crate::{Error, Result};
use image::{ImageFormat, RgbaImage};
use std::io::{Cursor, Read};
use std::path::Path;
use tracedraw_core::geometry::{Rect, Size};
use tracedraw_core::{Document, Fill, Shape, ShapeKind};

/// Bound compressed input independently of the decoder's allocation budget.
pub const MAX_INPUT_BYTES: u64 = 256 * 1024 * 1024;
/// No raster operation may allocate its output from unbounded dimensions.
pub const MAX_SIDE: u32 = 32_768;
pub const MAX_PIXELS: u64 = 64_000_000;

#[derive(Debug, Clone, Copy)]
pub struct RasterOptions {
    pub dpi: f64,
    pub quality: u8,
    pub webp_lossless: bool,
}

impl Default for RasterOptions {
    fn default() -> Self {
        Self {
            dpi: crate::image_dpi::DEFAULT_DPI,
            quality: 90,
            webp_lossless: true,
        }
    }
}

fn format(ext: &str) -> Result<ImageFormat> {
    match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "bmp" => Ok(ImageFormat::Bmp),
        "tif" | "tiff" => Ok(ImageFormat::Tiff),
        "webp" => Ok(ImageFormat::WebP),
        "gif" => Ok(ImageFormat::Gif),
        other => Err(Error::Format(format!("unsupported raster format: {other}"))),
    }
}

fn path_extension(path: &Path) -> Result<&str> {
    path.extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::Format("raster file needs a known suffix".into()))
}

pub fn is_raster_path(path: &Path) -> bool {
    path_extension(path).is_ok_and(|ext| format(ext).is_ok())
}

fn image_error(e: impl std::fmt::Display) -> Error {
    Error::Format(e.to_string())
}

pub(crate) fn dimensions(width: u32, height: u32) -> Result<()> {
    if width == 0
        || height == 0
        || width > MAX_SIDE
        || height > MAX_SIDE
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(Error::Format(
            "raster dimensions exceed the pixel budget".into(),
        ));
    }
    Ok(())
}

fn options_valid(options: &RasterOptions) -> Result<()> {
    if !options.dpi.is_finite() || !(1.0..=2400.0).contains(&options.dpi) {
        return Err(Error::Format(
            "raster DPI must be finite and between 1 and 2400".into(),
        ));
    }
    if !(1..=100).contains(&options.quality) {
        return Err(Error::Format(
            "raster quality must be between 1 and 100".into(),
        ));
    }
    Ok(())
}

/// Import one still image into a document at its recorded physical size.
/// Animated formats use the still image selected by the image decoder.
pub fn load_document(path: &Path) -> Result<Document> {
    format(path_extension(path)?)?;
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > MAX_INPUT_BYTES {
        return Err(Error::Format(
            "raster input exceeds the file-size budget".into(),
        ));
    }
    let mut bytes = Vec::new();
    // Take also bounds a file that grows after its metadata was read.
    file.take(MAX_INPUT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(Error::Format(
            "raster input exceeds the file-size budget".into(),
        ));
    }
    document_for_path(&bytes, path)
}

/// A named raster file must contain the codec named by its suffix. The shared
/// byte decoder also supports unnamed clipboard images, but a named file must
/// not silently pass a PNG payload as a TIFF or JPEG.
pub fn document_for_path(bytes: &[u8], path: &Path) -> Result<Document> {
    let expected = format(path_extension(path)?)?;
    let actual = if expected == ImageFormat::Tiff
        && [b"II*\0", b"MM\0*", b"II+\0", b"MM\0+"]
            .iter()
            .any(|magic| bytes.starts_with(*magic))
    {
        ImageFormat::Tiff
    } else {
        image::guess_format(bytes).map_err(image_error)?
    };
    if actual != expected {
        return Err(Error::Format(
            "raster contents do not match the file suffix".into(),
        ));
    }
    let title = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Image");
    document_from_bytes(bytes, title)
}

/// Construct the image document without filesystem access (also used by the
/// browser's uploaded files). A tagged CMYK image initializes the new drawing's
/// working profile; placing an image into an existing drawing is a separate path.
pub fn document_from_bytes(bytes: &[u8], title: &str) -> Result<Document> {
    let picture = crate::picture::read(bytes)?;
    let (width, height) = picture.rgba.dimensions();
    let (dx, dy) = crate::image_dpi::resolution(bytes)
        .unwrap_or((crate::image_dpi::DEFAULT_DPI, crate::image_dpi::DEFAULT_DPI));
    let size = Size::new(f64::from(width) * 25.4 / dx, f64::from(height) * 25.4 / dy);
    let mut doc = Document::new(title, size);
    doc.cmyk_profile = picture.profile.clone();
    let (png, inks) = picture
        .into_png()
        .ok_or_else(|| Error::Format("image could not be stored as PNG".into()))?;
    let id = doc.ids_mut().shape();
    let mut shape = Shape::new(
        id,
        ShapeKind::Bitmap {
            rect: Rect::new(0.0, 0.0, size.width, size.height),
            width_px: width,
            height_px: height,
            png: png.into(),
            fx: None,
        },
    );
    shape.fill = Fill::None;
    shape.stroke = None;
    shape.cmyk_bitmap = inks.is_some();
    shape.picture_inks = inks;
    let layer = doc
        .pages
        .first_mut()
        .and_then(|p| p.layers.first_mut())
        .ok_or_else(|| Error::Format("new image document has no layer".into()))?;
    layer.shapes.push(shape);
    Ok(doc)
}

/// Render the selected page and write actual bytes for the path's codec.
pub fn save_page(
    doc: &Document,
    page_index: usize,
    path: &Path,
    options: &RasterOptions,
) -> Result<()> {
    let ext = path_extension(path)?;
    let codec = format(ext)?;
    options_valid(options)?;
    let page = doc
        .pages
        .get(page_index)
        .ok_or_else(|| Error::Format("raster page index is out of range".into()))?;
    let zoom = options.dpi / 25.4;
    let width = tracedraw_render::pixel_extent(page.size.width * zoom)
        .ok_or_else(|| Error::Format("raster page width is invalid or too large".into()))?;
    let height = tracedraw_render::pixel_extent(page.size.height * zoom)
        .ok_or_else(|| Error::Format("raster page height is invalid or too large".into()))?;
    dimensions(width, height)?;
    if codec == ImageFormat::WebP && (width > 16_383 || height > 16_383) {
        return Err(Error::Format("WebP dimensions exceed 16383 pixels".into()));
    }
    let pm = tracedraw_render::render_page(
        doc,
        page.id,
        &tracedraw_render::RenderOptions {
            width,
            height,
            view: tracedraw_render::ViewTransform {
                zoom,
                origin_x: 0.0,
                origin_y: f64::from(height),
            },
            ..Default::default()
        },
    )
    .ok_or_else(|| Error::Format("raster page could not be rendered".into()))?;
    let mut rgba = RgbaImage::new(width, height);
    for (out, p) in rgba.pixels_mut().zip(pm.pixels()) {
        let p = p.demultiply();
        *out = image::Rgba([p.red(), p.green(), p.blue(), p.alpha()]);
    }
    let bytes = encode_rgba(&rgba, ext, options)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Encode straight RGBA, without quantizing colours in any lossless codec.
/// Quality affects JPEG and lossy WebP; it never rounds lossless WebP samples.
pub fn encode_rgba(img: &RgbaImage, ext: &str, options: &RasterOptions) -> Result<Vec<u8>> {
    let codec = format(ext)?;
    options_valid(options)?;
    let (width, height) = img.dimensions();
    dimensions(width, height)?;
    if codec == ImageFormat::WebP && (width > 16_383 || height > 16_383) {
        return Err(Error::Format("WebP dimensions exceed 16383 pixels".into()));
    }
    let mut out = Cursor::new(Vec::new());
    match codec {
        ImageFormat::Jpeg => {
            let rgb = on_white(img);
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, options.quality)
                .encode_image(&rgb)
                .map_err(image_error)?;
        }
        ImageFormat::Bmp => on_white(img)
            .write_to(&mut out, codec)
            .map_err(image_error)?,
        ImageFormat::Tiff => {
            let mut encoder = tiff::encoder::TiffEncoder::new(&mut out).map_err(image_error)?;
            let mut page = encoder
                .new_image::<tiff::encoder::colortype::RGBA8>(width, height)
                .map_err(image_error)?;
            page.resolution(
                tiff::tags::ResolutionUnit::Inch,
                tiff::encoder::Rational {
                    n: (options.dpi * 1000.0).round() as u32,
                    d: 1000,
                },
            );
            page.write_data(img.as_raw()).map_err(image_error)?;
        }
        ImageFormat::WebP if !options.webp_lossless => {
            if let Some(bytes) = webp_lossy(img, options.quality) {
                return Ok(bytes);
            }
            // If lossy encoding cannot represent the image, preserve its
            // samples in a true lossless file instead of rounding them.
            img.write_to(&mut out, codec).map_err(image_error)?;
        }
        _ => img.write_to(&mut out, codec).map_err(image_error)?,
    }
    Ok(record_dpi(out.into_inner(), ext, options.dpi))
}

fn on_white(img: &RgbaImage) -> image::RgbImage {
    image::RgbImage::from_fn(img.width(), img.height(), |x, y| {
        let p = img.get_pixel(x, y).0;
        let a = u32::from(p[3]);
        let c = |v: u8| ((u32::from(v) * a + 255 * (255 - a) + 127) / 255) as u8;
        image::Rgb([c(p[0]), c(p[1]), c(p[2])])
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn webp_lossy(img: &RgbaImage, quality: u8) -> Option<Vec<u8>> {
    let (width, height) = img.dimensions();
    webp::Encoder::from_rgba(img.as_raw(), width, height)
        .encode_simple(false, f32::from(quality))
        .ok()
        .map(|bytes| bytes.to_vec())
}

#[cfg(target_arch = "wasm32")]
fn webp_lossy(img: &RgbaImage, quality: u8) -> Option<Vec<u8>> {
    webp_lossy_pure(img, quality)
}

#[cfg(any(target_arch = "wasm32", test))]
fn webp_lossy_pure(img: &RgbaImage, quality: u8) -> Option<Vec<u8>> {
    use vaam_webp::{ColorType, EncoderParams, WebPEncoder};
    let (width, height) = img.dimensions();
    if width == 0 || height == 0 || width > 16_383 || height > 16_383 {
        return None;
    }
    let mut out = Vec::new();
    let mut encoder = WebPEncoder::new(&mut out);
    let mut params = EncoderParams::default();
    params.use_lossy = true;
    params.lossy_quality = quality.clamp(1, 100);
    encoder.set_params(params);
    let done = if img.pixels().all(|p| p.0[3] == 255) {
        let rgb = img
            .pixels()
            .flat_map(|p| [p.0[0], p.0[1], p.0[2]])
            .collect::<Vec<_>>();
        encoder.encode(&rgb, width, height, ColorType::Rgb8)
    } else {
        encoder.encode(img.as_raw(), width, height, ColorType::Rgba8)
    };
    done.ok().map(|()| out)
}

/// Stamp supported density metadata; unknown formats are left untouched.
pub fn record_dpi(mut bytes: Vec<u8>, ext: &str, dpi: f64) -> Vec<u8> {
    if !(dpi.is_finite() && dpi >= 1.0) {
        return bytes;
    }
    let per_metre = (dpi / 0.0254).round().min(f64::from(i32::MAX)) as u32;
    match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
        "png" => {
            // Signature (8) and the IHDR chunk (4 + 4 + 13 + 4).
            if bytes.len() < 33
                || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
                || bytes.get(12..16) != Some(b"IHDR")
            {
                return bytes;
            }
            let mut chunk = Vec::with_capacity(21);
            chunk.extend_from_slice(&9u32.to_be_bytes());
            chunk.extend_from_slice(b"pHYs");
            chunk.extend_from_slice(&per_metre.to_be_bytes());
            chunk.extend_from_slice(&per_metre.to_be_bytes());
            chunk.push(1);
            let crc = crc32(&chunk[4..]);
            chunk.extend_from_slice(&crc.to_be_bytes());
            // Replace an existing density instead of creating two pHYs
            // chunks when the desktop stamps its chosen resolution.
            let mut at = 33usize;
            let mut previous = None;
            while let Some(header) = bytes.get(at..at.saturating_add(8)) {
                let Some(len) = header.get(..4).and_then(|s| <[u8; 4]>::try_from(s).ok()) else {
                    break;
                };
                let Some(end) = at
                    .checked_add(12)
                    .and_then(|n| n.checked_add(u32::from_be_bytes(len) as usize))
                else {
                    break;
                };
                if end > bytes.len() {
                    break;
                }
                if header.get(4..8) == Some(b"pHYs") {
                    previous = Some(at..end);
                    break;
                }
                if header.get(4..8) == Some(b"IDAT") {
                    break;
                }
                at = end;
            }
            if let Some(range) = previous {
                bytes.splice(range, chunk);
            } else {
                bytes.splice(33..33, chunk);
            }
            bytes
        }
        "jpg" | "jpeg" => {
            // SOI, then the JFIF APP0 segment: units at 13, densities at
            // 14 and 16.
            if bytes.len() < 18
                || bytes.get(2..4) != Some(&[0xFF, 0xE0])
                || bytes.get(6..11) != Some(b"JFIF\0")
            {
                return bytes;
            }
            let d = (dpi.round().min(f64::from(u16::MAX)) as u16).to_be_bytes();
            bytes[13] = 1;
            bytes[14..16].copy_from_slice(&d);
            bytes[16..18].copy_from_slice(&d);
            bytes
        }
        "bmp" => {
            // File header (14), then the info header with the pixels per
            // metre at 24 and 28.
            if bytes.len() < 46 || !bytes.starts_with(b"BM") {
                return bytes;
            }
            let m = per_metre.to_le_bytes();
            bytes[38..42].copy_from_slice(&m);
            bytes[42..46].copy_from_slice(&m);
            bytes
        }
        "tif" | "tiff" => {
            stamp_tiff_dpi(&mut bytes, dpi);
            bytes
        }
        _ => bytes,
    }
}

/// Update the standard density tags already written by the TIFF encoder.
/// Other layouts remain untouched rather than guessing new IFD offsets.
fn stamp_tiff_dpi(bytes: &mut [u8], dpi: f64) {
    let le = bytes.starts_with(b"II*\0");
    if !le && !bytes.starts_with(b"MM\0*") {
        return;
    }
    let short = |b: &[u8], at: usize| -> Option<u16> {
        let v: [u8; 2] = b.get(at..at.checked_add(2)?)?.try_into().ok()?;
        Some(if le {
            u16::from_le_bytes(v)
        } else {
            u16::from_be_bytes(v)
        })
    };
    let long = |b: &[u8], at: usize| -> Option<u32> {
        let v: [u8; 4] = b.get(at..at.checked_add(4)?)?.try_into().ok()?;
        Some(if le {
            u32::from_le_bytes(v)
        } else {
            u32::from_be_bytes(v)
        })
    };
    let Some(ifd) = long(bytes, 4).map(|v| v as usize) else {
        return;
    };
    let Some(count) = short(bytes, ifd) else {
        return;
    };
    for i in 0..usize::from(count).min(512) {
        let Some(at) = ifd.checked_add(2).and_then(|at| at.checked_add(i * 12)) else {
            return;
        };
        let Some(entry) = bytes.get(at..at.saturating_add(12)) else {
            return;
        };
        let Some(tag) = short(entry, 0) else { return };
        if tag == 296 {
            let v = if le {
                2u16.to_le_bytes()
            } else {
                2u16.to_be_bytes()
            };
            if let Some(unit) = bytes.get_mut(at + 8..at + 10) {
                unit.copy_from_slice(&v);
            }
        } else if (tag == 282 || tag == 283)
            && short(entry, 2) == Some(5)
            && long(entry, 4) == Some(1)
        {
            let Some(off) = long(entry, 8).map(|v| v as usize) else {
                return;
            };
            let Some(value) = bytes.get_mut(off..off.saturating_add(8)) else {
                return;
            };
            let n = (dpi * 1000.0).round().min(f64::from(u32::MAX)) as u32;
            let (n, d) = if le {
                (n.to_le_bytes(), 1000u32.to_le_bytes())
            } else {
                (n.to_be_bytes(), 1000u32.to_be_bytes())
            };
            if let Some(dst) = value.get_mut(..4) {
                dst.copy_from_slice(&n);
            }
            if let Some(dst) = value.get_mut(4..) {
                dst.copy_from_slice(&d);
            }
        }
    }
}

/// The CRC-32 a PNG chunk ends with.
fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c ^= u32::from(b);
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
    }
    !c
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp(name: &str) -> std::path::PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "craft-raster-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("directory");
        dir.join(name)
    }

    fn samples() -> RgbaImage {
        RgbaImage::from_fn(7, 5, |x, y| {
            image::Rgba([
                (x * 29 + 1) as u8,
                (y * 37 + 3) as u8,
                (x * 11 + y * 13 + 5) as u8,
                if x == 0 {
                    0
                } else if x == 1 {
                    127
                } else {
                    255
                },
            ])
        })
    }

    #[test]
    fn lossless_webp_preserves_odd_rgb_and_alpha_at_every_quality() {
        let original = samples();
        for quality in [40, 90, 100] {
            let bytes = encode_rgba(
                &original,
                "webp",
                &RasterOptions {
                    quality,
                    ..Default::default()
                },
            )
            .expect("encode");
            let decoded = image::load_from_memory(&bytes)
                .expect("decode")
                .into_rgba8();
            assert_eq!(decoded, original, "lossless at quality {quality}");
        }
    }

    #[test]
    fn raster_codecs_write_decodable_files_and_keep_physical_size() {
        let original = samples();
        for ext in ["PNG", "jpg", "jpeg", "bmp", "tif", "tiff", "webp", "gif"] {
            let options = RasterOptions {
                dpi: 300.0,
                ..Default::default()
            };
            let bytes = encode_rgba(&original, ext, &options).expect(ext);
            let decoded = image::load_from_memory(&bytes)
                .expect("codec output")
                .into_rgba8();
            assert_eq!(decoded.dimensions(), original.dimensions());
            if matches!(ext, "PNG" | "tif" | "tiff" | "webp") {
                assert_eq!(decoded, original, "{ext}");
            }
            let path = temp(&format!("image.{ext}"));
            std::fs::write(&path, &bytes).expect("input");
            let doc = load_document(&path).expect("import");
            let dpi = if matches!(ext, "webp" | "gif") {
                96.0
            } else {
                300.0
            };
            assert!(
                (doc.pages[0].size.width - 7.0 * 25.4 / dpi).abs() < 0.001,
                "{ext}"
            );
            assert!(
                (doc.pages[0].size.height - 5.0 * 25.4 / dpi).abs() < 0.001,
                "{ext}"
            );
            std::fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
        }
    }

    #[test]
    fn save_page_rejects_unknown_formats_and_invalid_render_sizes_before_writing() {
        let doc = Document::new("page", Size::new(25.4, 12.7));
        let options = RasterOptions::default();
        let path = temp("output.unknown");
        assert!(save_page(&doc, 0, &path, &options).is_err());
        assert!(!path.exists());
        let png = path.with_extension("png");
        assert!(save_page(&doc, 9, &png, &options).is_err());
        assert!(save_page(
            &doc,
            0,
            &png,
            &RasterOptions {
                dpi: f64::NAN,
                ..options
            }
        )
        .is_err());
        assert!(save_page(
            &doc,
            0,
            &png,
            &RasterOptions {
                quality: 0,
                ..options
            }
        )
        .is_err());
        let huge = Document::new("large", Size::new(1e20, 1e20));
        assert!(save_page(&huge, 0, &png, &options).is_err());
        assert!(!png.exists());
        std::fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn save_page_uses_the_selected_page_and_keeps_transparency() {
        let doc = Document::new("page", Size::new(25.4, 12.7));
        let path = temp("output.png");
        save_page(&doc, 0, &path, &RasterOptions::default()).expect("render");
        let image = image::open(&path).expect("image").into_rgba8();
        assert_eq!(image.dimensions(), (96, 48));
        assert!(image.pixels().all(|p| p.0[3] == 0));
        let doc = load_document(&path).expect("import");
        // PNG stores integral pixels/metre, so 96 DPI becomes 96.012.
        assert!((doc.pages[0].size.width - 25.4).abs() < 0.005);
        std::fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn imported_raster_round_trips_keep_integral_pixel_extents() {
        let original = RgbaImage::from_pixel(378, 265, image::Rgba([31, 63, 127, 255]));
        let options = RasterOptions::default();
        for ext in ["jpeg", "tiff", "webp", "gif"] {
            let path = temp(&format!("input.{ext}"));
            let input = encode_rgba(&original, ext, &options).expect("encode input");
            std::fs::write(&path, input).expect("write input");
            let doc = load_document(&path).expect("import original");
            let output = path.with_file_name(format!("output.{ext}"));
            save_page(&doc, 0, &output, &options).expect("export imported image");
            assert_eq!(
                image::image_dimensions(&output).expect("dimensions"),
                (378, 265),
                "{ext}"
            );
            let back = load_document(&output).expect("reimport exported image");
            let png = path.with_file_name("after-reimport.png");
            save_page(&back, 0, &png, &options).expect("export again");
            assert_eq!(
                image::image_dimensions(&png).expect("dimensions"),
                (378, 265),
                "{ext}"
            );
            std::fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
        }
    }

    #[test]
    fn raster_export_still_covers_real_fractional_pixels() {
        let options = RasterOptions::default();
        let zoom = options.dpi / 25.4;
        let doc = Document::new("fractional", Size::new(378.001 / zoom, 265.0 / zoom));
        let path = temp("fractional.png");
        save_page(&doc, 0, &path, &options).expect("render fraction");
        assert_eq!(
            image::image_dimensions(&path).expect("dimensions"),
            (379, 265)
        );
        std::fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn malformed_or_mislabeled_input_is_rejected() {
        let path = temp("broken.png");
        std::fs::write(&path, b"not an image").expect("input");
        assert!(load_document(&path).is_err());
        let mislabeled = path.with_extension("jpg");
        let png = encode_rgba(&samples(), "png", &RasterOptions::default()).expect("png");
        std::fs::write(&mislabeled, png).expect("input");
        assert!(load_document(&mislabeled).is_err());
        assert!(!is_raster_path(Path::new("a.unknown")));
        assert!(is_raster_path(Path::new("a.JPEG")));
        assert!(dimensions(32768, 32768).is_err());
        std::fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn restamping_density_replaces_png_metadata_and_updates_tiff() {
        for ext in ["png", "tiff"] {
            let bytes = encode_rgba(&samples(), ext, &RasterOptions::default()).expect("encode");
            let bytes = record_dpi(bytes, ext, 300.0);
            let bytes = record_dpi(bytes, ext, 150.0);
            let (x, y) = crate::image_dpi::resolution(&bytes).expect("DPI");
            // Integral PNG pixels/metre quantize DPI by at most 0.0127.
            assert!(
                (x - 150.0).abs() < 0.02 && (y - 150.0).abs() < 0.02,
                "{ext}: {x} {y}"
            );
            assert!(image::load_from_memory(&bytes).is_ok(), "valid {ext}");
            if ext == "png" {
                assert_eq!(bytes.windows(4).filter(|s| *s == b"pHYs").count(), 1);
            }
        }
    }

    #[test]
    fn declared_pixel_bomb_is_rejected_before_decoding_image_data() {
        let mut bytes = encode_rgba(&samples(), "png", &RasterOptions::default()).expect("encode");
        // A tiny compressed file claims a billion output pixels. The
        // IHDR CRC is valid so the rejection must not rely on bad CRC.
        bytes[16..20].copy_from_slice(&32_768u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&32_768u32.to_be_bytes());
        let crc = crc32(&bytes[12..29]);
        bytes[29..33].copy_from_slice(&crc.to_be_bytes());
        let path = temp("bomb.png");
        std::fs::write(&path, bytes).expect("input");
        let err = load_document(&path).expect_err("pixel budget");
        assert!(err.to_string().contains("pixel budget"), "{err}");
        std::fs::remove_dir_all(path.parent().expect("parent")).expect("cleanup");
    }

    #[test]
    fn web_lossy_encoder_remains_decodable_and_keeps_alpha() {
        let mut original = RgbaImage::from_fn(128, 64, |x, y| {
            image::Rgba([x as u8, y as u8, (x ^ y) as u8, 255])
        });
        for x in 0..16 {
            for y in 0..64 {
                original.get_pixel_mut(x, y).0[3] = 0;
            }
        }
        let bytes = webp_lossy_pure(&original, 60).expect("web lossy");
        assert!(bytes.windows(4).any(|s| s == b"VP8 "));
        let decoded = image::load_from_memory(&bytes)
            .expect("decode")
            .into_rgba8();
        assert_eq!(decoded.dimensions(), original.dimensions());
        assert_eq!(decoded.get_pixel(4, 4).0[3], 0);
        assert_eq!(decoded.get_pixel(40, 4).0[3], 255);
        assert!(webp_lossy_pure(&RgbaImage::new(0, 0), 60).is_none());
    }
}
