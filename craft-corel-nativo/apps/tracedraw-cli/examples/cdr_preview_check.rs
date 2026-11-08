//! Compare our rendering of a `.cdr` file with the preview the file
//! carries (`previews/page1.png` in ZIP files, the `DISP` bitmap in RIFF
//! files). Writes `<out>.png` with the preview on the left and our render
//! of the same area on the right, and prints the mean colour difference.
//!
//! usage: cargo run --release --example cdr_preview_check -- <file.cdr> <out.png>
//!
//! RIFF previews frame every object of the first page (on the page or
//! off it); ZIP previews frame the page.

use tracedraw_core::geometry::Rect;
use tracedraw_render::{render_page, RenderOptions, ViewTransform};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (Some(input), Some(output)) = (args.get(1), args.get(2)) else {
        eprintln!("usage: cdr_preview_check <file.cdr> <out.png>");
        std::process::exit(2);
    };
    let bytes = match std::fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{input}: {e}");
            std::process::exit(1);
        }
    };
    let Some((preview, frames_page)) = preview(&bytes) else {
        eprintln!("{input}: no preview");
        std::process::exit(1);
    };
    let (doc, _) = match tracedraw_cdr::open_bytes(&bytes, "check") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{input}: {e}");
            std::process::exit(1);
        }
    };
    tracedraw_text::install();
    if !doc.fonts.is_empty() {
        tracedraw_text::fonts().add_document_fonts(&doc.fonts);
    }
    // CMYK colours through the profile the drawing carries, as on screen.
    if let Some(p) = &doc.cmyk_profile {
        if let Ok(profile) = tracedraw_core::icc::Profile::parse(&p.data) {
            if profile.color_space() == tracedraw_core::icc::ColorSpace::Cmyk {
                tracedraw_core::color::engine::install(tracedraw_core::color::IccEngine::new(
                    None,
                    Some(profile),
                    tracedraw_core::icc::Intent::default(),
                    false,
                ));
            }
        }
    }
    let Some(page) = doc.pages.first() else {
        std::process::exit(1);
    };
    let page_rect = Rect::new(0.0, 0.0, page.size.width, page.size.height);
    let area = if frames_page {
        page_rect
    } else {
        let bounds = doc
            .content_bounds(page.id)
            .ok()
            .flatten()
            .unwrap_or(page_rect);
        visible_area(&doc, page.id, bounds).unwrap_or(bounds)
    };
    let (pw, ph) = (preview.width(), preview.height());
    // Fit the area into the preview, centred, as previews are drawn.
    let zoom = (pw as f64 / area.width().max(1e-6)).min(ph as f64 / area.height().max(1e-6));
    let ox = (pw as f64 - area.width() * zoom) / 2.0 - area.x0 * zoom;
    let oy = (ph as f64 + area.height() * zoom) / 2.0 + area.y0 * zoom;
    let Some(ours) = render_page(
        &doc,
        page.id,
        &RenderOptions {
            width: pw,
            height: ph,
            view: ViewTransform {
                zoom,
                origin_x: ox,
                origin_y: oy,
            },
            preview: None,
            wireframe: false,
            simulate_overprints: false,
            complex_effects: true,
            anti_alias: true,
            draft: false,
            focus: None,
        },
    ) else {
        std::process::exit(1);
    };
    let mut side = image::RgbImage::from_pixel(pw * 2 + 4, ph, image::Rgb([255, 0, 255]));
    let mut diff = 0.0f64;
    for y in 0..ph {
        for x in 0..pw {
            let p = preview.get_pixel(x, y).0;
            side.put_pixel(x, y, image::Rgb([p[0], p[1], p[2]]));
            let o = ours.pixel(x, y).map(|c| c.demultiply());
            let q = match o {
                Some(c) => {
                    let a = c.alpha() as f64 / 255.0;
                    let mix = |v: u8| (v as f64 * a + 255.0 * (1.0 - a)).round() as u8;
                    [mix(c.red()), mix(c.green()), mix(c.blue())]
                }
                None => [255, 255, 255],
            };
            side.put_pixel(pw + 4 + x, y, image::Rgb(q));
            diff += (0..3)
                .map(|i| (p[i] as f64 - q[i] as f64).abs())
                .sum::<f64>()
                / 3.0;
        }
    }
    println!("{:.2}", diff / (pw as f64 * ph as f64));
    if let Err(e) = side.save(output) {
        eprintln!("{output}: {e}");
    }
}

/// The part of `bounds` where something is drawn: objects without fill
/// or outline count in the content's bounds but not in a preview.
fn visible_area(
    doc: &tracedraw_core::Document,
    page: tracedraw_core::PageId,
    bounds: Rect,
) -> Option<Rect> {
    const SIDE: f64 = 1024.0;
    let zoom = SIDE / bounds.width().max(bounds.height()).max(1e-6);
    let w = (bounds.width() * zoom).ceil().max(1.0) as u32;
    let h = (bounds.height() * zoom).ceil().max(1.0) as u32;
    let pm = render_page(
        doc,
        page,
        &RenderOptions {
            width: w,
            height: h,
            view: ViewTransform {
                zoom,
                origin_x: -bounds.x0 * zoom,
                origin_y: bounds.y1 * zoom,
            },
            preview: None,
            wireframe: false,
            simulate_overprints: false,
            complex_effects: false,
            anti_alias: false,
            draft: true,
            focus: None,
        },
    )?;
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for y in 0..h {
        for x in 0..w {
            if pm.pixel(x, y).is_some_and(|p| p.alpha() > 16) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    Some(Rect::new(
        bounds.x0 + f64::from(x0) / zoom,
        bounds.y1 - f64::from(y1) / zoom,
        bounds.x0 + f64::from(x1) / zoom,
        bounds.y1 - f64::from(y0) / zoom,
    ))
}

/// The preview image and whether it frames the page (ZIP) rather than
/// the objects (RIFF).
fn preview(bytes: &[u8]) -> Option<(image::RgbImage, bool)> {
    if bytes.starts_with(b"PK") {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
        let mut f = zip.by_name("previews/page1.png").ok()?;
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut data).ok()?;
        return Some((image::load_from_memory(&data).ok()?.to_rgb8(), true));
    }
    // RIFF: the DISP chunk among the first ones holds a u32 then a DIB
    // (BITMAPINFOHEADER, palette, pixels); give it a file header.
    let at = bytes.get(12..512)?.windows(4).position(|w| w == b"DISP")? + 12;
    let size = u32::from_le_bytes(bytes.get(at + 4..at + 8)?.try_into().ok()?) as usize;
    let dib = bytes.get(at + 12..at + 8 + size)?;
    let bpp = u16::from_le_bytes(dib.get(14..16)?.try_into().ok()?);
    let used = u32::from_le_bytes(dib.get(32..36)?.try_into().ok()?) as usize;
    let colours = if used > 0 {
        used
    } else if bpp <= 8 {
        1 << bpp
    } else {
        0
    };
    let offset = 14 + 40 + colours * 4;
    let mut bmp = Vec::with_capacity(14 + dib.len());
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&((14 + dib.len()) as u32).to_le_bytes());
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&(offset as u32).to_le_bytes());
    bmp.extend_from_slice(dib);
    Some((image::load_from_memory(&bmp).ok()?.to_rgb8(), false))
}
