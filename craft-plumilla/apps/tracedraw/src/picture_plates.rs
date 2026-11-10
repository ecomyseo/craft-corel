//! Pictures on colour separation plates (#1659).
//!
//! A plate shows the coverage of one ink in grey (black = full ink). A
//! picture is split into its inks once per picture: a picture that keeps
//! its ink channels (`Shape::picture_inks`, a `.cdr` CMYK image) gives
//! them as they are; any other picture (RGB, greyscale, CMYK converted
//! from RGB) has its pixels converted to CMYK with the drawing's colour
//! management (`Color::convert_to`, the conversion vector colours take),
//! each distinct colour once. The pixels' alpha stays on every plate, so
//! transparency, masks and clipping print as they show. The split and
//! the plates made from it are cached by picture, so the plates of a
//! print job, and Print Preview redrawing a plate, do the work once.

use crate::export::Plate;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracedraw_core::color::IccEngine;
use tracedraw_core::document::ShapeKind;
use tracedraw_core::picture_inks::{PictureInk, PictureInks};
use tracedraw_core::{Color, Shape, SharedBytes};

/// A picture split into its inks.
struct Separated {
    width: u32,
    height: u32,
    /// The pixels' alpha.
    alpha: Vec<u8>,
    /// Cyan, magenta, yellow and black of each pixel (255 = full ink).
    cmyk: Vec<[u8; 4]>,
    /// The spot inks the picture holds: the ink's id and its coverage.
    spots: Vec<(u32, Vec<u8>)>,
}

impl Separated {
    fn bytes(&self) -> usize {
        self.alpha.len() * 5 + self.spots.iter().map(|s| s.1.len()).sum::<usize>()
    }
}

/// One cached picture: what it was split from and the plates made of it.
struct Entry {
    png: SharedBytes,
    inks: Option<SharedBytes>,
    engine: Option<Arc<IccEngine>>,
    gray_to_black: bool,
    separated: Arc<Separated>,
    made: Vec<(String, SharedBytes)>,
}

/// The cache, the most recent picture last; kept under this many bytes
/// of split pixels (the last picture always stays).
static CACHE: Mutex<Vec<Entry>> = Mutex::new(Vec::new());
const CACHE_BYTES: usize = 256 << 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Checkpoint {
    BeforeDecode,
    AfterDecode,
    AlphaRow,
    ColourRow,
    ChannelRow,
    PlateRow,
    PngWrite,
    AfterPng,
    BeforePublish,
    Cached,
}
struct Work<'a> {
    cancelled: Option<&'a AtomicBool>,
    #[cfg(test)]
    probe: Option<&'a dyn Fn(Checkpoint)>,
}
impl<'a> Work<'a> {
    fn new(cancelled: Option<&'a AtomicBool>) -> Self {
        Self {
            cancelled,
            #[cfg(test)]
            probe: None,
        }
    }
    fn check(&self, point: Checkpoint) -> Result<(), String> {
        #[cfg(test)]
        if let Some(probe) = self.probe {
            probe(point);
        }
        #[cfg(not(test))]
        let _ = point;
        if self
            .cancelled
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            Err(crate::export::CANCELLED.into())
        } else {
            Ok(())
        }
    }
}

/// The picture `s` shows (a bitmap's pixels) and the ink channels that
/// fit them.
fn picture(s: &Shape) -> Option<(&SharedBytes, Option<&PictureInks>)> {
    let ShapeKind::Bitmap { png, .. } = &s.kind else {
        return None;
    };
    let inks = s.picture_inks.as_deref().filter(|i| i.fits(png.as_slice()));
    Some((png, inks))
}

/// The picture of `s` as made by `make` from its inks (`key` names what
/// `make` makes), from the cache when it was made before.
fn made(
    s: &Shape,
    key: &str,
    make: &dyn Fn(&Separated) -> Option<SharedBytes>,
) -> Option<SharedBytes> {
    made_with_work(s, key, &|sep, _| Ok(make(sep)), &Work::new(None))
        .ok()
        .flatten()
}
fn made_with_work(
    s: &Shape,
    key: &str,
    make: &dyn Fn(&Separated, &Work<'_>) -> Result<Option<SharedBytes>, String>,
    work: &Work<'_>,
) -> Result<Option<SharedBytes>, String> {
    work.check(Checkpoint::Cached)?;
    let Some((png, inks)) = picture(s) else {
        return Ok(None);
    };
    let engine = tracedraw_core::color::engine::icc();
    let gray_to_black = tracedraw_core::color::engine::map_gray_to_black();
    let same = |e: &Entry| {
        SharedBytes::ptr_eq(&e.png, png)
            && match (&e.inks, inks) {
                (Some(a), Some(b)) => SharedBytes::ptr_eq(a, &b.png),
                (None, None) => true,
                _ => false,
            }
            && match (&e.engine, &engine) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && e.gray_to_black == gray_to_black
    };
    let cached = {
        let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
        let at = cache.iter().position(same);
        at.map(|i| {
            // Most recently used last.
            let e = cache.remove(i);
            let found = e
                .made
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, b)| b.clone());
            let sep = e.separated.clone();
            cache.push(e);
            (sep, found)
        })
    };
    let separated = match cached {
        Some((_, Some(done))) => {
            work.check(Checkpoint::Cached)?;
            return Ok(Some(done));
        }
        Some((sep, None)) => sep,
        None => {
            let Some(separated) = separate_with_work(png, inks, work)? else {
                return Ok(None);
            };
            Arc::new(separated)
        }
    };
    let Some(out) = make(&separated, work)? else {
        return Ok(None);
    };
    work.check(Checkpoint::BeforePublish)?;
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    work.check(Checkpoint::BeforePublish)?;
    if let Some(e) = cache.iter_mut().find(|e| same(e)) {
        e.made.push((key.to_string(), out.clone()));
    } else {
        cache.push(Entry {
            png: png.clone(),
            inks: inks.map(|i| i.png.clone()),
            engine,
            gray_to_black,
            separated,
            made: vec![(key.to_string(), out.clone())],
        });
        let mut total: usize = cache.iter().map(|e| e.separated.bytes()).sum();
        while cache.len() > 1 && total > CACHE_BYTES {
            total -= cache.remove(0).separated.bytes();
        }
    }
    Ok(Some(out))
}

/// Split a picture into its inks: from the channels it keeps, else its
/// pixels through the colour management.
fn separate(png: &[u8], inks: Option<&PictureInks>) -> Option<Separated> {
    separate_with_work(png, inks, &Work::new(None))
        .ok()
        .flatten()
}
fn separate_with_work(
    png: &[u8],
    inks: Option<&PictureInks>,
    work: &Work<'_>,
) -> Result<Option<Separated>, String> {
    work.check(Checkpoint::BeforeDecode)?;
    let img = crate::bitmap_fx::decode(png);
    work.check(Checkpoint::AfterDecode)?;
    let Some(img) = img else {
        return Ok(None);
    };
    let (width, height) = img.dimensions();
    let n = width as usize * height as usize;
    let mut alpha = Vec::with_capacity(n);
    for row in img.rows() {
        work.check(Checkpoint::AlphaRow)?;
        alpha.extend(row.map(|pixel| pixel[3]));
    }
    if let Some(inks) = inks {
        if let Some(sep) = from_channels_with_work(inks, width, height, &alpha, work)? {
            return Ok(Some(sep));
        }
    }
    let mut known: HashMap<[u8; 3], [u8; 4]> = HashMap::new();
    let mut cmyk = Vec::with_capacity(n);
    for row in img.rows() {
        work.check(Checkpoint::ColourRow)?;
        for p in row {
            if p[3] == 0 {
                cmyk.push([0; 4]);
                continue;
            }
            let rgb = [p[0], p[1], p[2]];
            let ink = *known.entry(rgb).or_insert_with(|| {
                match Color::rgb8(rgb[0], rgb[1], rgb[2]).convert_to("CMYK") {
                    Color::Cmyk { c, m, y, k } => [byte(c), byte(m), byte(y), byte(k)],
                    _ => [0; 4],
                }
            });
            cmyk.push(ink);
        }
    }
    work.check(Checkpoint::ColourRow)?;
    Ok(Some(Separated {
        width,
        height,
        alpha,
        cmyk,
        spots: Vec::new(),
    }))
}

/// The inks a picture keeps, when they decode to planes of its size.
fn from_channels_with_work(
    inks: &PictureInks,
    width: u32,
    height: u32,
    alpha: &[u8],
    work: &Work<'_>,
) -> Result<Option<Separated>, String> {
    work.check(Checkpoint::BeforeDecode)?;
    let planes = image::load_from_memory_with_format(&inks.png, image::ImageFormat::Png)
        .ok()
        .map(|image| image.to_luma8());
    work.check(Checkpoint::AfterDecode)?;
    let Some(planes) = planes else {
        return Ok(None);
    };
    let n = width as usize * height as usize;
    if planes.width() != width
        || planes.height() as usize != height as usize * inks.inks.len()
        || inks.inks.is_empty()
    {
        return Ok(None);
    }
    let raw = planes.as_raw();
    let mut cmyk = Vec::with_capacity(n);
    for _ in 0..height {
        work.check(Checkpoint::ChannelRow)?;
        cmyk.extend(std::iter::repeat_n([0u8; 4], width as usize));
    }
    let mut spots = Vec::new();
    for (i, ink) in inks.inks.iter().enumerate() {
        let Some(plane) = raw.get(i * n..(i + 1) * n) else {
            return Ok(None);
        };
        match *ink {
            PictureInk::Process(k) if k < 4 => {
                for (pixels, values) in cmyk
                    .chunks_mut(width as usize)
                    .zip(plane.chunks(width as usize))
                {
                    work.check(Checkpoint::ChannelRow)?;
                    for (px, v) in pixels.iter_mut().zip(values) {
                        px[k as usize] = *v;
                    }
                }
            }
            PictureInk::Process(_) => {}
            PictureInk::Spot(id) => {
                let mut copied = Vec::with_capacity(n);
                for row in plane.chunks(width as usize) {
                    work.check(Checkpoint::ChannelRow)?;
                    copied.extend_from_slice(row);
                }
                spots.push((id, copied));
            }
        }
    }
    let mut copied_alpha = Vec::with_capacity(n);
    for row in alpha.chunks(width as usize) {
        work.check(Checkpoint::AlphaRow)?;
        copied_alpha.extend_from_slice(row);
    }
    work.check(Checkpoint::ChannelRow)?;
    Ok(Some(Separated {
        width,
        height,
        alpha: copied_alpha,
        cmyk,
        spots,
    }))
}

fn byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// PNG bytes of RGBA pixels.
fn encode(width: u32, height: u32, rgba: Vec<u8>) -> Option<SharedBytes> {
    let img = image::RgbaImage::from_raw(width, height, rgba)?;
    crate::bitmap_fx::encode(&img).map(SharedBytes::from)
}

/// The plate of a picture: the coverage of the plate's ink in grey,
/// with the picture's alpha.
pub fn plate_picture(s: &Shape, plate: Plate) -> Option<SharedBytes> {
    plate_picture_with_work(s, plate, &Work::new(None))
        .ok()
        .flatten()
}
pub(crate) fn plate_picture_with_cancel(
    s: &Shape,
    plate: Plate,
    cancel: &AtomicBool,
) -> Result<Option<SharedBytes>, String> {
    plate_picture_with_work(s, plate, &Work::new(Some(cancel)))
}
fn plate_picture_with_work(
    s: &Shape,
    plate: Plate,
    work: &Work<'_>,
) -> Result<Option<SharedBytes>, String> {
    let key = format!("{plate:?}");
    made_with_work(
        s,
        &key,
        &|sep, work| {
            let none = Vec::new();
            let spot = match plate {
                Plate::Spot(id) => sep
                    .spots
                    .iter()
                    .find(|(i, _)| *i == id)
                    .map(|(_, p)| p)
                    .unwrap_or(&none),
                Plate::Process(_) => &none,
            };
            let mut rgba = Vec::with_capacity(sep.alpha.len() * 4);
            for (i, a) in sep.alpha.iter().enumerate() {
                if i % sep.width as usize == 0 {
                    work.check(Checkpoint::PlateRow)?;
                }
                let v = match plate {
                    Plate::Process(k) => {
                        sep.cmyk.get(i).and_then(|c| c.get(k)).copied().unwrap_or(0)
                    }
                    Plate::Spot(_) => spot.get(i).copied().unwrap_or(0),
                };
                let g = 255 - v;
                rgba.extend_from_slice(&[g, g, g, *a]);
            }
            encode_with_work(sep.width, sep.height, rgba, work)
        },
        work,
    )
}

fn encode_with_work(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    work: &Work<'_>,
) -> Result<Option<SharedBytes>, String> {
    let Some(image) = image::RgbaImage::from_raw(width, height, rgba) else {
        return Ok(None);
    };
    Ok(encode_image_with_work(&image, work)?.map(SharedBytes::from))
}
pub(crate) fn encode_rgba_with_cancel(
    image: &image::RgbaImage,
    cancel: &AtomicBool,
) -> Result<Option<Vec<u8>>, String> {
    encode_image_with_work(image, &Work::new(Some(cancel)))
}
fn encode_image_with_work(
    image: &image::RgbaImage,
    work: &Work<'_>,
) -> Result<Option<Vec<u8>>, String> {
    if work.cancelled.is_none() {
        return Ok(crate::bitmap_fx::encode(image));
    }
    use std::io::{Cursor, Seek, SeekFrom, Write};
    struct Checked<'a, 'b> {
        bytes: Cursor<Vec<u8>>,
        work: &'a Work<'b>,
    }
    impl Write for Checked<'_, '_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.work
                .check(Checkpoint::PngWrite)
                .map_err(std::io::Error::other)?;
            let count = self.bytes.write(bytes)?;
            self.work
                .check(Checkpoint::PngWrite)
                .map_err(std::io::Error::other)?;
            Ok(count)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.work
                .check(Checkpoint::PngWrite)
                .map_err(std::io::Error::other)?;
            self.bytes.flush()
        }
    }
    impl Seek for Checked<'_, '_> {
        fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
            self.work
                .check(Checkpoint::PngWrite)
                .map_err(std::io::Error::other)?;
            self.bytes.seek(from)
        }
    }
    work.check(Checkpoint::PngWrite)?;
    let mut writer = Checked {
        bytes: Cursor::new(Vec::new()),
        work,
    };
    let result = image.write_to(&mut writer, image::ImageFormat::Png);
    work.check(Checkpoint::AfterPng)?;
    Ok(result.ok().map(|_| writer.bytes.into_inner()))
}

/// A picture as printed with only some inks: the process inks of
/// `process` and the spot inks of `spots` (each with its colour as
/// RGB); the others are paper.
pub fn inks_picture(
    s: &Shape,
    process: [bool; 4],
    spots: &[(u32, [u8; 3])],
) -> Option<SharedBytes> {
    let key = format!("{process:?}{spots:?}");
    made(s, &key, &|sep| {
        let mut shown: HashMap<[u8; 4], [u8; 3]> = HashMap::new();
        let on: Vec<(&[u8], [u8; 3])> = sep
            .spots
            .iter()
            .filter_map(|(id, plane)| {
                spots
                    .iter()
                    .find(|(i, _)| i == id)
                    .map(|(_, rgb)| (plane.as_slice(), *rgb))
            })
            .collect();
        let mut rgba = Vec::with_capacity(sep.alpha.len() * 4);
        for (i, a) in sep.alpha.iter().enumerate() {
            let mut ink = sep.cmyk.get(i).copied().unwrap_or([0; 4]);
            for (v, keep) in ink.iter_mut().zip(process) {
                if !keep {
                    *v = 0;
                }
            }
            let rgb = *shown.entry(ink).or_insert_with(|| {
                let f = |v: u8| v as f32 / 255.0;
                Color::Cmyk {
                    c: f(ink[0]),
                    m: f(ink[1]),
                    y: f(ink[2]),
                    k: f(ink[3]),
                }
                .to_rgb8()
            });
            // Spot inks print over the process inks, as duotone inks do.
            let mut c = rgb.map(|v| v as f32 / 255.0);
            for (plane, spot) in &on {
                let cover = plane.get(i).copied().unwrap_or(0) as f32 / 255.0;
                for (v, s) in c.iter_mut().zip(spot) {
                    *v *= 1.0 - cover * (1.0 - *s as f32 / 255.0);
                }
            }
            rgba.extend_from_slice(&[byte(c[0]), byte(c[1]), byte(c[2]), *a]);
        }
        encode(sep.width, sep.height, rgba)
    })
}

/// A picture as CMYK ink coverages, for a CMYK file of it (Save Bitmap
/// As, Edit Bitmap, #1834): cyan, magenta, yellow and black of each
/// pixel, row by row, 255 = full ink, with its size. The inks it keeps
/// give them as they are, other pictures their pixels through the colour
/// management, as on the separation plates. A spot ink it keeps prints
/// as its process mix over the process inks (`spots`: each ink's id
/// and its C, M, Y and K from 0 to 1); a see-through pixel holds as much
/// less ink as it lets the paper show. With `alpha` each pixel keeps
/// its full inks and a fifth byte, its opacity (unassociated alpha,
/// #1925), for a file that keeps transparency.
pub fn picture_cmyk(
    s: &Shape,
    spots: &[(u32, [f32; 4])],
    alpha: bool,
) -> Option<(u32, u32, Vec<u8>)> {
    let (png, inks) = picture(s)?;
    let sep = separate(png, inks)?;
    let mut out = Vec::with_capacity(sep.alpha.len() * if alpha { 5 } else { 4 });
    for (i, a) in sep.alpha.iter().enumerate() {
        let ink = sep.cmyk.get(i).copied().unwrap_or([0; 4]);
        let mut v = ink.map(|x| x as f32 / 255.0);
        for (id, plane) in &sep.spots {
            let Some((_, mix)) = spots.iter().find(|(s, _)| s == id) else {
                continue;
            };
            let cover = plane.get(i).copied().unwrap_or(0) as f32 / 255.0;
            for (x, m) in v.iter_mut().zip(mix) {
                *x = 1.0 - (1.0 - *x) * (1.0 - cover * m.clamp(0.0, 1.0));
            }
        }
        if alpha {
            out.extend(v.map(byte));
            out.push(*a);
            continue;
        }
        let a = *a as f32 / 255.0;
        out.extend(v.map(|x| byte(x * a)));
    }
    Some((sep.width, sep.height, out))
}

/// Put `picture`'s result in place of every picture in `s` and the
/// objects it holds (groups, clip frames); a picture it gives nothing
/// for stays.
pub fn swap_pictures(s: &mut Shape, picture: &dyn Fn(&Shape) -> Option<SharedBytes>) {
    if matches!(s.kind, ShapeKind::Bitmap { .. }) {
        if let Some(new) = picture(s) {
            if let ShapeKind::Bitmap { png, fx, .. } = &mut s.kind {
                *png = new;
                *fx = None;
            }
            s.cmyk_bitmap = false;
            s.picture_inks = None;
        }
        return;
    }
    match &mut s.kind {
        ShapeKind::Group { children } => {
            children.iter_mut().for_each(|c| swap_pictures(c, picture))
        }
        ShapeKind::ClipFrame { frame, contents } => {
            swap_pictures(frame, picture);
            contents.iter_mut().for_each(|c| swap_pictures(c, picture));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use tracedraw_core::{Rect, ShapeId};

    fn fixture(native: bool) -> Shape {
        let image = image::RgbaImage::from_fn(17, 9, |x, y| {
            image::Rgba([
                (x * 11) as u8,
                (y * 23) as u8,
                ((x + y) * 7) as u8,
                [0, 37, 128, 255][((x + y) % 4) as usize],
            ])
        });
        let png: SharedBytes = crate::bitmap_fx::encode(&image).unwrap().into();
        let mut shape = Shape::new(
            ShapeId(99001),
            ShapeKind::Bitmap {
                rect: Rect::new(0.0, 0.0, 17.0, 9.0),
                width_px: 17,
                height_px: 9,
                png: png.clone(),
                fx: None,
            },
        );
        if native {
            let image = image::GrayImage::from_fn(17, 9 * 5, |x, y| {
                image::Luma([((x * 3 + (y % 9) * 7 + (y / 9) * 31) % 256) as u8])
            });
            let mut bytes = std::io::Cursor::new(Vec::new());
            image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
            shape.picture_inks = Some(Box::new(PictureInks {
                inks: vec![
                    PictureInk::Process(0),
                    PictureInk::Process(1),
                    PictureInk::Process(2),
                    PictureInk::Process(3),
                    PictureInk::Spot(7),
                ],
                png: bytes.into_inner().into(),
                of: tracedraw_core::picture_inks::fingerprint(&png),
                profile: None,
            }));
        }
        shape
    }

    #[test]
    fn cancelled_preparation_never_publishes_a_partial_separation_or_png() {
        for (native, phase, at) in [
            (false, Checkpoint::BeforeDecode, 1),
            (false, Checkpoint::AfterDecode, 1),
            (false, Checkpoint::AlphaRow, 2),
            (false, Checkpoint::ColourRow, 2),
            (true, Checkpoint::ChannelRow, 3),
            (true, Checkpoint::PlateRow, 2),
            (false, Checkpoint::PngWrite, 4),
            (false, Checkpoint::AfterPng, 1),
            (false, Checkpoint::BeforePublish, 1),
        ] {
            let shape = fixture(native);
            let (png, _) = picture(&shape).unwrap();
            let cancelled = AtomicBool::new(false);
            let visits = Cell::new(0);
            let rechecks_after_cancel = Cell::new(0);
            let probe = |point| {
                if point == phase {
                    if cancelled.load(Ordering::Relaxed) {
                        rechecks_after_cancel.set(rechecks_after_cancel.get() + 1);
                    }
                    visits.set(visits.get() + 1);
                    if visits.get() == at {
                        cancelled.store(true, Ordering::Relaxed);
                    }
                }
            };
            let work = Work {
                cancelled: Some(&cancelled),
                probe: Some(&probe),
            };
            let result = plate_picture_with_work(&shape, Plate::Process(0), &work);
            assert!(
                matches!(result, Err(ref error) if error == crate::export::CANCELLED),
                "{phase:?}"
            );
            // png::Writer::drop attempts IEND after a write error. That
            // cleanup write must encounter the already-raised flag too.
            let cleanup_rechecks = usize::from(phase == Checkpoint::PngWrite);
            assert_eq!(visits.get(), at + cleanup_rechecks, "{phase:?}");
            assert_eq!(rechecks_after_cancel.get(), cleanup_rechecks, "{phase:?}");
            assert!(cancelled.load(Ordering::Relaxed), "{phase:?}");
            assert!(
                !CACHE
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .iter()
                    .any(|entry| SharedBytes::ptr_eq(&entry.png, png)),
                "{phase:?}"
            );
        }
    }

    #[test]
    fn uncancelled_plates_match_legacy_png_bytes_and_independent_ink_alpha_pixels() {
        tracedraw_core::color::engine::with_icc_scope(None, false, || {
            for native in [false, true] {
                for plate in [
                    Plate::Process(0),
                    Plate::Process(1),
                    Plate::Process(2),
                    Plate::Process(3),
                    Plate::Spot(7),
                ] {
                    let old = fixture(native);
                    let new = fixture(native); // equal content, distinct retained allocation
                    let expected = plate_picture(&old, plate).unwrap();
                    let actual = plate_picture_with_cancel(&new, plate, &AtomicBool::new(false))
                        .unwrap()
                        .unwrap();
                    assert_eq!(actual.as_slice(), expected.as_slice());
                    let decoded = crate::bitmap_fx::decode(&actual).unwrap();
                    let original = crate::bitmap_fx::decode(picture(&new).unwrap().0).unwrap();
                    assert_eq!(decoded.dimensions(), (17, 9));
                    for (x, y, pixel) in decoded.enumerate_pixels() {
                        let raw = original.get_pixel(x, y);
                        let ink = if native {
                            let channel = match plate {
                                Plate::Process(k) => k,
                                Plate::Spot(_) => 4,
                            };
                            ((x * 3 + y * 7 + channel as u32 * 31) % 256) as u8
                        } else if raw[3] == 0 {
                            0
                        } else {
                            match (
                                plate,
                                Color::rgb8(raw[0], raw[1], raw[2]).convert_to("CMYK"),
                            ) {
                                (Plate::Process(k), Color::Cmyk { c, m, y, k: black }) => {
                                    byte([c, m, y, black][k])
                                }
                                _ => 0,
                            }
                        };
                        assert_eq!(
                            pixel.0,
                            [255 - ink, 255 - ink, 255 - ink, raw[3]],
                            "{native:?}/{plate:?}/{x}/{y}"
                        );
                    }
                }
            }
        });
    }

    #[test]
    fn cached_complete_plate_and_invalid_input_observe_cancel_without_losing_legacy_behaviour() {
        let mut shape = fixture(false);
        let valid = plate_picture(&shape, Plate::Process(0)).unwrap();
        let flag = AtomicBool::new(true);
        assert!(
            matches!(plate_picture_with_cancel(&shape, Plate::Process(0), &flag), Err(ref e) if e == crate::export::CANCELLED)
        );
        flag.store(false, Ordering::Relaxed);
        assert_eq!(
            plate_picture_with_cancel(&shape, Plate::Process(0), &flag)
                .unwrap()
                .unwrap(),
            valid
        );
        if let ShapeKind::Bitmap { png, .. } = &mut shape.kind {
            *png = vec![1, 2, 3].into();
        }
        assert!(plate_picture(&shape, Plate::Process(0)).is_none());
        flag.store(true, Ordering::Relaxed);
        assert!(
            matches!(plate_picture_with_cancel(&shape, Plate::Process(0), &flag), Err(ref e) if e == crate::export::CANCELLED)
        );
    }
}
