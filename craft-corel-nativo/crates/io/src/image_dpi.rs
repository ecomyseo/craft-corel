//! The resolution a bitmap file records, so an imported image comes in at
//! the size it was made for: PNG `pHYs`, JPEG JFIF density or the EXIF
//! resolution tags, TIFF `XResolution`/`YResolution` and the BMP header's
//! pixels per metre. Every reader checks its bounds and gives `None` on
//! anything it does not understand; the caller then uses 96 dpi.

/// Resolution assumed when a file records none.
pub const DEFAULT_DPI: f64 = 96.0;

/// Horizontal and vertical resolution in pixels per inch, when the file
/// states one in absolute units (an aspect ratio alone does not count).
pub fn resolution(bytes: &[u8]) -> Option<(f64, f64)> {
    let r = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        jpeg(bytes)
    } else if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        tiff(bytes)
    } else if bytes.starts_with(b"BM") {
        bmp(bytes)
    } else {
        None
    }?;
    let ok = |v: f64| v.is_finite() && (1.0..=100_000.0).contains(&v);
    (ok(r.0) && ok(r.1)).then_some(r)
}

fn be16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        b.get(at..at.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        b.get(at..at.checked_add(4)?)?.try_into().ok()?,
    ))
}

fn png(b: &[u8]) -> Option<(f64, f64)> {
    let mut at = 8;
    loop {
        let len = be32(b, at)? as usize;
        let kind = b.get(at.checked_add(4)?..at.checked_add(8)?)?;
        let data = at.checked_add(8)?;
        if kind == b"pHYs" {
            if len != 9 {
                return None;
            }
            let x = be32(b, data)? as f64;
            let y = be32(b, data.checked_add(4)?)? as f64;
            // Unit 1: pixels per metre; 0 is an aspect ratio only.
            return (*b.get(data.checked_add(8)?)? == 1).then_some((x * 0.0254, y * 0.0254));
        }
        if kind == b"IDAT" || kind == b"IEND" {
            return None;
        }
        at = data.checked_add(len)?.checked_add(4)?;
    }
}

fn jpeg(b: &[u8]) -> Option<(f64, f64)> {
    let mut at = 2usize;
    let mut exif = None;
    while at.checked_add(4).is_some_and(|end| end <= b.len()) {
        if b.get(at) != Some(&0xFF) {
            return None;
        }
        let marker = *b.get(at.checked_add(1)?)?;
        if marker == 0xFF {
            at += 1;
            continue;
        }
        // Start of scan or end of image: no more headers.
        if marker == 0xDA || marker == 0xD9 {
            break;
        }
        let len = be16(b, at.checked_add(2)?)? as usize;
        let data = b.get(at.checked_add(4)?..at.checked_add(2)?.checked_add(len)?)?;
        if marker == 0xE0 && data.starts_with(b"JFIF\0") {
            let units = *data.get(7)?;
            let x = be16(data, 8)? as f64;
            let y = be16(data, 10)? as f64;
            match units {
                1 => return Some((x, y)),
                2 => return Some((x * 2.54, y * 2.54)),
                _ => {}
            }
        }
        if marker == 0xE1 && data.starts_with(b"Exif\0\0") && exif.is_none() {
            exif = data.get(6..).and_then(tiff);
        }
        at = at.checked_add(2)?.checked_add(len)?;
    }
    exif
}

fn tiff(b: &[u8]) -> Option<(f64, f64)> {
    let le = b.starts_with(b"II");
    let u16_at = |at: usize| -> Option<u16> {
        let s: [u8; 2] = b.get(at..at.checked_add(2)?)?.try_into().ok()?;
        Some(if le {
            u16::from_le_bytes(s)
        } else {
            u16::from_be_bytes(s)
        })
    };
    let u32_at = |at: usize| -> Option<u32> {
        let s: [u8; 4] = b.get(at..at.checked_add(4)?)?.try_into().ok()?;
        Some(if le {
            u32::from_le_bytes(s)
        } else {
            u32::from_be_bytes(s)
        })
    };
    let ifd = u32_at(4)? as usize;
    let count = u16_at(ifd)? as usize;
    let (mut x, mut y, mut unit) = (None, None, 2u16);
    for i in 0..count.min(512) {
        let e = ifd.checked_add(2)?.checked_add(i.checked_mul(12)?)?;
        let tag = u16_at(e)?;
        let kind = u16_at(e.checked_add(2)?)?;
        let value = |e: usize| -> Option<f64> {
            match kind {
                // RATIONAL: numerator and denominator at the offset.
                5 => {
                    let off = u32_at(e.checked_add(8)?)? as usize;
                    let d = u32_at(off.checked_add(4)?)?;
                    (d != 0).then(|| u32_at(off).map(|n| n as f64 / d as f64))?
                }
                3 => u16_at(e.checked_add(8)?).map(f64::from),
                4 => u32_at(e.checked_add(8)?).map(f64::from),
                _ => None,
            }
        };
        match tag {
            282 => x = value(e),
            283 => y = value(e),
            296 => unit = u16_at(e.checked_add(8)?)?,
            _ => {}
        }
    }
    let (x, y) = (x?, y.or(x)?);
    match unit {
        2 => Some((x, y)),
        3 => Some((x * 2.54, y * 2.54)),
        _ => None,
    }
}

fn bmp(b: &[u8]) -> Option<(f64, f64)> {
    let header = u32::from_le_bytes(b.get(14..18)?.try_into().ok()?);
    if header < 40 {
        return None;
    }
    let x = i32::from_le_bytes(b.get(38..42)?.try_into().ok()?);
    let y = i32::from_le_bytes(b.get(42..46)?.try_into().ok()?);
    (x > 0 && y > 0).then_some((x as f64 * 0.0254, y as f64 * 0.0254))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(format: image::ImageFormat) -> Vec<u8> {
        let img = image::RgbaImage::from_pixel(4, 2, image::Rgba([10, 20, 30, 255]));
        let mut out = Vec::new();
        let dynamic = if format == image::ImageFormat::Jpeg {
            image::DynamicImage::ImageRgb8(image::DynamicImage::ImageRgba8(img).to_rgb8())
        } else {
            image::DynamicImage::ImageRgba8(img)
        };
        dynamic
            .write_to(&mut std::io::Cursor::new(&mut out), format)
            .expect("encode");
        out
    }

    /// A PNG with a `pHYs` chunk put in after `IHDR`.
    fn png_with_phys(ppm: u32, unit: u8) -> Vec<u8> {
        let png = encode(image::ImageFormat::Png);
        let ihdr_end = 8 + 8 + 13 + 4;
        let mut data = Vec::new();
        data.extend_from_slice(&ppm.to_be_bytes());
        data.extend_from_slice(&ppm.to_be_bytes());
        data.push(unit);
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());
        chunk.extend_from_slice(b"pHYs");
        chunk.extend_from_slice(&data);
        chunk.extend_from_slice(&[0, 0, 0, 0]); // CRC is not checked here.
        let mut out = png[..ihdr_end].to_vec();
        out.extend(chunk);
        out.extend_from_slice(&png[ihdr_end..]);
        out
    }

    #[test]
    fn png_phys_in_pixels_per_metre() {
        // 11811 px/m is 300 dpi.
        let (x, y) = resolution(&png_with_phys(11811, 1)).expect("dpi");
        assert!(
            (x - 300.0).abs() < 0.1 && (y - 300.0).abs() < 0.1,
            "{x} {y}"
        );
        // Aspect ratio only, or no chunk: nothing.
        assert_eq!(resolution(&png_with_phys(11811, 0)), None);
        assert_eq!(resolution(&encode(image::ImageFormat::Png)), None);
    }

    /// A JPEG whose JFIF header says `units` and 150 x 72.
    fn jpeg_with_jfif(units: u8) -> Vec<u8> {
        let mut j = encode(image::ImageFormat::Jpeg);
        // Find the APP0 JFIF segment and rewrite its density.
        let at = j
            .windows(5)
            .position(|w| w == b"JFIF\0")
            .expect("the encoder writes JFIF");
        j[at + 7] = units;
        j[at + 8..at + 10].copy_from_slice(&150u16.to_be_bytes());
        j[at + 10..at + 12].copy_from_slice(&72u16.to_be_bytes());
        j
    }

    #[test]
    fn jpeg_jfif_density() {
        assert_eq!(resolution(&jpeg_with_jfif(1)), Some((150.0, 72.0)));
        let (x, _) = resolution(&jpeg_with_jfif(2)).expect("dpcm");
        assert!((x - 381.0).abs() < 1e-9);
        assert_eq!(resolution(&jpeg_with_jfif(0)), None);
    }

    /// A little-endian TIFF directory with the resolution tags.
    fn tiff_header(x: u32, unit: u16) -> Vec<u8> {
        let mut b = b"II*\0".to_vec();
        b.extend_from_slice(&8u32.to_le_bytes());
        let entries: u16 = 3;
        b.extend_from_slice(&entries.to_le_bytes());
        let rational_at = 8 + 2 + 12 * entries as u32 + 4;
        let entry = |b: &mut Vec<u8>, tag: u16, kind: u16, value: u32| {
            b.extend_from_slice(&tag.to_le_bytes());
            b.extend_from_slice(&kind.to_le_bytes());
            b.extend_from_slice(&1u32.to_le_bytes());
            b.extend_from_slice(&value.to_le_bytes());
        };
        entry(&mut b, 282, 5, rational_at);
        entry(&mut b, 283, 5, rational_at);
        entry(&mut b, 296, 3, unit as u32);
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&x.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes());
        b
    }

    #[test]
    fn tiff_and_exif_resolution() {
        assert_eq!(resolution(&tiff_header(200, 2)), Some((200.0, 200.0)));
        let (x, _) = resolution(&tiff_header(100, 3)).expect("per cm");
        assert!((x - 254.0).abs() < 1e-9);
        assert_eq!(resolution(&tiff_header(200, 1)), None);
        // The same directory inside a JPEG's EXIF segment.
        let mut exif = b"Exif\0\0".to_vec();
        exif.extend(tiff_header(240, 2));
        let mut j = vec![0xFF, 0xD8, 0xFF, 0xE1];
        j.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
        j.extend(exif);
        j.extend_from_slice(&[0xFF, 0xD9]);
        assert_eq!(resolution(&j), Some((240.0, 240.0)));
    }

    #[test]
    fn malformed_tiff_offsets_never_wrap_on_32_bit_targets() {
        for offset in [u32::MAX, u32::MAX - 1, u32::MAX - 3] {
            let mut bad_ifd = tiff_header(200, 2);
            bad_ifd[4..8].copy_from_slice(&offset.to_le_bytes());
            assert_eq!(resolution(&bad_ifd), None);
            let mut bad_rational = tiff_header(200, 2);
            // First IFD entry starts at 10; its value offset is at 18.
            bad_rational[18..22].copy_from_slice(&offset.to_le_bytes());
            assert_eq!(resolution(&bad_rational), None);
        }
    }

    #[test]
    fn bmp_pixels_per_metre() {
        let mut b = encode(image::ImageFormat::Bmp);
        b[38..42].copy_from_slice(&3780i32.to_le_bytes());
        b[42..46].copy_from_slice(&3780i32.to_le_bytes());
        let (x, _) = resolution(&b).expect("dpi");
        assert!((x - 96.0).abs() < 0.1, "{x}");
    }

    #[test]
    fn broken_or_unknown_input_gives_nothing() {
        for b in [
            &b""[..],
            b"\x89PNG\r\n\x1a\n\xff\xff\xff\xffpHYs",
            &[0xFF, 0xD8, 0xFF, 0xE0, 0x00],
            b"II*\0\xff\xff\xff\xff",
            b"MM\0*\0\0\0\x08\x00\x01",
            b"BM",
            b"GIF89a",
        ] {
            assert_eq!(resolution(b), None);
        }
        let mut huge = png_with_phys(u32::MAX, 1);
        assert_eq!(resolution(&huge), None);
        huge.truncate(40);
        assert_eq!(resolution(&huge), None);
    }
}
