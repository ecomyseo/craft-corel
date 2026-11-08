//! Shared picture decoding. CMYK TIFFs keep their ink planes and source ICC
//! profile alongside a display PNG; profile conversion never mutates the global
//! colour engine. Other currently supported pictures use the RGBA8 decoder.

use crate::{Error, Result};
use image::{DynamicImage, ImageDecoder};
use std::io::Cursor;
use tracedraw_core::document::DocumentProfile;
use tracedraw_core::icc::{ColorSpace, Intent, Profile, Transform};
use tracedraw_core::picture_inks::{PictureInk, PictureInks};
use tracedraw_core::{Color, Shape, ShapeKind};

const MAX_PROFILE_BYTES: usize = 16 * 1024 * 1024;

fn error(value: impl std::fmt::Display) -> Error {
    Error::Format(value.to_string())
}

/// Pixels and their original CMYK planes, if present. `profile` describes those
/// planes, not the already converted sRGB display pixels.
pub struct ReadPicture {
    pub rgba: image::RgbaImage,
    pub planes: Option<Vec<u8>>,
    pub profile: Option<DocumentProfile>,
}

impl ReadPicture {
    pub fn into_png(self) -> Option<(Vec<u8>, Option<Box<PictureInks>>)> {
        let png = crate::raster::encode_rgba(&self.rgba, "png", &Default::default()).ok()?;
        let inks = self.planes.map(|planes| {
            let mut inks = PictureInks::cmyk(&png, planes.into());
            inks.profile = self.profile;
            Box::new(inks)
        });
        Some((png, inks))
    }
}

pub fn read(bytes: &[u8]) -> Result<ReadPicture> {
    if bytes.len() as u64 > crate::raster::MAX_INPUT_BYTES {
        return Err(error("picture exceeds the input-size budget"));
    }
    if [b"II*\0", b"MM\0*", b"II+\0", b"MM\0+"]
        .iter()
        .any(|magic| bytes.starts_with(*magic))
    {
        if let Some(picture) = read_cmyk_tiff(bytes)? {
            return Ok(picture);
        }
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(crate::raster::MAX_SIDE);
    limits.max_image_height = Some(crate::raster::MAX_SIDE);
    limits.max_alloc = Some(crate::raster::MAX_PIXELS * 4);
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(error)?;
    reader.limits(limits);
    let decoder = reader.into_decoder().map_err(error)?;
    let (width, height) = decoder.dimensions();
    crate::raster::dimensions(width, height)?;
    let rgba = DynamicImage::from_decoder(decoder)
        .map_err(error)?
        .into_rgba8();
    Ok(ReadPicture {
        rgba,
        planes: None,
        profile: None,
    })
}

fn read_cmyk_tiff(bytes: &[u8]) -> Result<Option<ReadPicture>> {
    use tiff::decoder::{Decoder, DecodingResult, Limits};
    use tiff::tags::Tag;
    let mut limits = Limits::default();
    limits.decoding_buffer_size = (crate::raster::MAX_PIXELS * 5) as usize;
    limits.ifd_value_size = MAX_PROFILE_BYTES;
    let mut decoder = Decoder::new(Cursor::new(bytes))
        .map_err(error)?
        .with_limits(limits);
    let step = match decoder.colortype().map_err(error)? {
        tiff::ColorType::CMYK(8) => 4,
        tiff::ColorType::CMYKA(8) => 5,
        tiff::ColorType::CMYK(_) | tiff::ColorType::CMYKA(_) => {
            return Err(error("CMYK TIFF currently requires 8 bits per channel"))
        }
        _ => return Ok(None),
    };
    let (width, height) = decoder.dimensions().map_err(error)?;
    crate::raster::dimensions(width, height)?;
    let planar = decoder
        .find_tag_unsigned::<u16>(Tag::PlanarConfiguration)
        .map_err(error)?
        .unwrap_or(1);
    if !matches!(planar, 1 | 2) {
        return Err(error("invalid CMYK TIFF planar configuration"));
    }
    if step == 5
        && decoder
            .find_tag_unsigned_vec::<u16>(Tag::ExtraSamples)
            .map_err(error)?
            .as_deref()
            != Some(&[2])
    {
        return Err(error("CMYKA TIFF currently requires unassociated alpha"));
    }
    let profile_bytes = decoder
        .find_tag(Tag::IccProfile)
        .map_err(error)?
        .map(|value| value.into_u8_vec().map_err(error))
        .transpose()?;
    let mut display = None;
    let profile = if let Some(data) = profile_bytes {
        if data.len() > MAX_PROFILE_BYTES {
            return Err(error("embedded ICC profile exceeds its size budget"));
        }
        let parsed = Profile::parse(&data).map_err(error)?;
        if parsed.color_space() != ColorSpace::Cmyk {
            return Err(error("embedded ICC profile does not describe CMYK samples"));
        }
        display = Some(
            Transform::new(&parsed, &Profile::srgb(), Intent::default(), false).ok_or_else(
                || error("embedded CMYK ICC profile has no supported display transform"),
            )?,
        );
        Some(DocumentProfile::new(
            parsed.description().unwrap_or("Embedded CMYK"),
            data,
        ))
    } else {
        None
    };
    let count = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| error("invalid CMYK dimensions"))?;
    let length = count
        .checked_mul(step)
        .ok_or_else(|| error("invalid CMYK buffer size"))?;
    // Unlike read_image(), this API reads every plane of a planar TIFF.
    let mut decoded = DecodingResult::U8(Vec::new());
    let layout = decoder.read_image_to_buffer(&mut decoded).map_err(error)?;
    let DecodingResult::U8(data) = decoded else {
        return Err(error("unexpected CMYK TIFF channel storage"));
    };
    if layout.complete_len != length || data.len() != length {
        return Err(error("CMYK channel buffer has the wrong length"));
    }
    let mut planes = vec![0; count * 4];
    let mut rgba = Vec::with_capacity(count * 4);
    let mut cache = std::collections::HashMap::new();
    for index in 0..count {
        let sample = |channel: usize| {
            let offset = if planar == 2 {
                channel * count + index
            } else {
                index * step + channel
            };
            data.get(offset)
                .copied()
                .ok_or_else(|| error("incomplete CMYK sample"))
        };
        let ink = [sample(0)?, sample(1)?, sample(2)?, sample(3)?];
        for (channel, value) in ink.iter().enumerate() {
            planes[channel * count + index] = *value;
        }
        let rgb = if let Some(rgb) = cache.get(&ink) {
            *rgb
        } else {
            let channels = ink.map(|value| f32::from(value) / 255.0);
            let rgb = if let Some(transform) = &display {
                transform
                    .cmyk_to_rgb(channels)
                    .ok_or_else(|| error("CMYK display transform failed"))?
                    .map(byte)
            } else {
                let [c, m, y, k] = channels;
                Color::Cmyk { c, m, y, k }.to_rgb_f32_builtin().map(byte)
            };
            // Bound memoization even for a photograph with unique sample values.
            if cache.len() < 4096 {
                cache.insert(ink, rgb);
            }
            rgb
        };
        rgba.extend_from_slice(&[
            rgb[0],
            rgb[1],
            rgb[2],
            if step == 5 { sample(4)? } else { 255 },
        ]);
    }
    let rgba = image::RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| error("invalid display pixels"))?;
    let stacked_height = height
        .checked_mul(4)
        .ok_or_else(|| error("invalid CMYK planes height"))?;
    let gray = image::GrayImage::from_raw(width, stacked_height, planes)
        .ok_or_else(|| error("invalid CMYK planes"))?;
    let mut png = Cursor::new(Vec::new());
    gray.write_to(&mut png, image::ImageFormat::Png)
        .map_err(error)?;
    Ok(Some(ReadPicture {
        rgba,
        planes: Some(png.into_inner()),
        profile,
    }))
}

fn byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn cmyk_profile(data: &[u8]) -> Result<Profile> {
    if data.len() > MAX_PROFILE_BYTES {
        return Err(error("ICC profile exceeds its size budget"));
    }
    let profile = Profile::parse(data).map_err(error)?;
    if profile.color_space() != ColorSpace::Cmyk {
        return Err(error("ICC profile does not describe CMYK samples"));
    }
    Ok(profile)
}

/// Convert interleaved CMYK/CMYKA to a requested output profile before embedding
/// that profile. Alpha stays unassociated and byte-for-byte unchanged.
pub fn convert_cmyk_samples(
    samples: &mut [u8],
    channels: usize,
    source: &[u8],
    target: &[u8],
) -> Result<()> {
    if !matches!(channels, 4 | 5) || !samples.len().is_multiple_of(channels) {
        return Err(error("invalid interleaved CMYK sample layout"));
    }
    let src = cmyk_profile(source)?;
    let dst = cmyk_profile(target)?;
    if source == target {
        return Ok(());
    }
    let transform = Transform::new(&src, &dst, Intent::default(), false)
        .ok_or_else(|| error("the requested CMYK profile conversion is unavailable"))?;
    for pixel in samples.chunks_exact_mut(channels) {
        let input = [pixel[0], pixel[1], pixel[2], pixel[3]].map(|v| f32::from(v) / 255.0);
        let converted = transform.transform(&input);
        if converted.len() != 4 {
            return Err(error("CMYK conversion returned an invalid sample"));
        }
        for (out, value) in pixel.iter_mut().take(4).zip(converted) {
            *out = byte(value);
        }
    }
    Ok(())
}

/// Write a preserved process-CMYK bitmap without converting its ink samples to
/// RGB and back. An edited PNG or spot channels require another explicit path.
pub fn original_cmyk_tiff(shape: &Shape, dpi: f64) -> Result<Vec<u8>> {
    let ShapeKind::Bitmap {
        png,
        width_px,
        height_px,
        ..
    } = &shape.kind
    else {
        return Err(error("not a bitmap"));
    };
    let inks = shape
        .picture_inks
        .as_deref()
        .filter(|inks| inks.fits(png))
        .ok_or_else(|| {
            error("original picture inks are unavailable or no longer fit the pixels")
        })?;
    if inks.inks != (0..4).map(PictureInk::Process).collect::<Vec<_>>() {
        return Err(error("picture does not have four process-CMYK planes"));
    }
    crate::raster::dimensions(*width_px, *height_px)?;
    let pixels = read(png)?.rgba;
    let mut reader =
        image::ImageReader::with_format(Cursor::new(inks.png.as_slice()), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(*width_px);
    limits.max_image_height = height_px.checked_mul(4);
    limits.max_alloc = Some(crate::raster::MAX_PIXELS * 4);
    reader.limits(limits);
    let planes = reader.decode().map_err(error)?.into_luma8();
    if pixels.dimensions() != (*width_px, *height_px)
        || planes.dimensions() != (*width_px, height_px.saturating_mul(4))
    {
        return Err(error("picture plane dimensions do not match the bitmap"));
    }
    let count = (*width_px as usize) * (*height_px as usize);
    let plane = planes.as_raw();
    let alpha = pixels.pixels().any(|p| p.0[3] != 255);
    let mut samples = Vec::with_capacity(count * if alpha { 5 } else { 4 });
    for (i, pixel) in pixels.pixels().enumerate() {
        samples.extend_from_slice(&[
            plane[i],
            plane[count + i],
            plane[2 * count + i],
            plane[3 * count + i],
        ]);
        if alpha {
            samples.push(pixel.0[3]);
        }
    }
    let icc = inks.profile.as_ref().map(|profile| profile.data.as_slice());
    if let Some(profile) = icc {
        cmyk_profile(profile)?;
    }
    let encoded = if alpha {
        crate::cmyk_bitmap::cmyka_tiff(*width_px, *height_px, &samples, dpi, icc)
    } else {
        crate::cmyk_bitmap::cmyk_tiff(*width_px, *height_px, &samples, dpi, icc)
    };
    encoded.ok_or_else(|| error("CMYK TIFF encoding failed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::icc::test_profiles as profiles;

    fn samples() -> Vec<u8> {
        vec![
            17, 33, 49, 65, 0, 251, 127, 3, 9, 127, 0, 255, 0, 0, 255, 11, 29, 57, 113, 255,
        ]
    }

    fn alternate_profile() -> Vec<u8> {
        let mut lut = profiles::lut16_cmyk_lab_tag();
        // mft2: 52-byte header, four 2-entry input tables, then 16 Lab cells.
        // A darker synthetic calibration with the same ink samples.
        for cell in lut[68..164].chunks_exact_mut(6) {
            let lightness = u16::from_be_bytes([cell[0], cell[1]]) / 2;
            cell[..2].copy_from_slice(&lightness.to_be_bytes());
        }
        profiles::Builder::new(b"prtr", b"CMYK", b"Lab ", 2)
            .tag(b"desc", profiles::desc("Darker synthetic CMYK"))
            .tag(b"wtpt", profiles::xyz_tag([0.9642, 1.0, 0.8249]))
            .tag(b"A2B0", lut)
            .tag(b"B2A0", profiles::mba_lab_cmyk_tag())
            .build()
    }

    fn file(icc: Option<&[u8]>) -> Vec<u8> {
        crate::cmyk_bitmap::cmyka_tiff(2, 2, &samples(), 300.0, icc).unwrap()
    }

    fn tiff_parts(bytes: &[u8]) -> (Vec<u8>, Option<Vec<u8>>) {
        let mut dec = tiff::decoder::Decoder::new(Cursor::new(bytes)).unwrap();
        let icc = dec
            .find_tag(tiff::tags::Tag::IccProfile)
            .unwrap()
            .map(|v| v.into_u8_vec().unwrap());
        let tiff::decoder::DecodingResult::U8(samples) = dec.read_image().unwrap() else {
            panic!("8-bit samples");
        };
        (samples, icc)
    }

    #[test]
    fn cmyka_tiff_keeps_samples_alpha_and_each_picture_profile_through_native_json() {
        let profile = profiles::lut16_cmyk_profile();
        let mut document =
            crate::raster::document_from_bytes(&file(Some(&profile)), "inks").unwrap();
        let alternate = alternate_profile();
        let second = crate::raster::document_from_bytes(&file(Some(&alternate)), "second").unwrap();
        let mut shape = second.pages[0].layers[0].shapes[0].clone();
        shape.id = document.ids_mut().shape();
        document.pages[0].layers[0].shapes.push(shape);
        let restored = tracedraw_core::Document::from_json(&document.to_json().unwrap()).unwrap();
        assert_eq!(
            restored.cmyk_profile.as_ref().map(|p| p.data.as_slice()),
            Some(profile.as_slice()),
            "opening initializes the working profile; appending a second image keeps it"
        );
        for (shape, profile) in restored.pages[0].layers[0]
            .shapes
            .iter()
            .zip([&profile, &alternate])
        {
            let saved = original_cmyk_tiff(shape, 300.0).unwrap();
            let (actual, embedded) = tiff_parts(&saved);
            assert_eq!(
                actual,
                samples(),
                "all four original inks and alpha, including transparent pixels"
            );
            assert_eq!(embedded.as_deref(), Some(profile.as_slice()));
        }
        let left = &restored.pages[0].layers[0].shapes[0];
        let right = &restored.pages[0].layers[0].shapes[1];
        let (ShapeKind::Bitmap { png: a, .. }, ShapeKind::Bitmap { png: b, .. }) =
            (&left.kind, &right.kind)
        else {
            panic!("bitmaps");
        };
        assert_ne!(
            read(a).unwrap().rgba,
            read(b).unwrap().rgba,
            "source profile must affect the preview"
        );
        assert_eq!(
            left.picture_inks.as_ref().unwrap().png,
            right.picture_inks.as_ref().unwrap().png,
            "changing calibration must not change the source inks"
        );
    }

    #[test]
    fn explicit_profile_conversion_changes_inks_and_keeps_unassociated_alpha() {
        let source = profiles::lut16_cmyk_profile();
        let target = alternate_profile();
        let mut actual = samples();
        convert_cmyk_samples(&mut actual, 5, &source, &source).unwrap();
        assert_eq!(
            actual,
            samples(),
            "same profile preserves exact ink samples"
        );
        convert_cmyk_samples(&mut actual, 5, &source, &target).unwrap();
        assert_ne!(actual, samples());
        assert_eq!(
            actual.chunks_exact(5).map(|p| p[4]).collect::<Vec<_>>(),
            vec![0, 127, 255, 255]
        );
        assert!(convert_cmyk_samples(&mut actual, 3, &source, &target).is_err());
        assert!(convert_cmyk_samples(&mut actual[..3], 4, &source, &target).is_err());
        assert!(convert_cmyk_samples(&mut actual, 5, &source, b"invalid").is_err());
    }

    #[test]
    fn invalid_or_mismatched_embedded_profiles_are_reported() {
        assert!(read(&file(Some(b"invalid profile"))).is_err());
        let rgb = profiles::matrix_trc_profile(
            profiles::curv_gamma(1.0),
            profiles::curv_gamma(1.0),
            profiles::curv_gamma(1.0),
        );
        assert!(read(&file(Some(&rgb))).is_err());
    }

    fn patch_tag(bytes: &mut [u8], tag: u16, value: u32) {
        assert_eq!(&bytes[..2], b"II");
        let offset = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let count = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()) as usize;
        for entry in bytes[offset + 2..offset + 2 + 12 * count].chunks_exact_mut(12) {
            if u16::from_le_bytes([entry[0], entry[1]]) == tag {
                entry[8..12].copy_from_slice(&value.to_le_bytes());
                return;
            }
        }
        panic!("fixture tag {tag} must exist");
    }

    fn replace_long_array(bytes: &mut Vec<u8>, tag: u16, values: &[u32]) {
        let pointer = bytes.len() as u32;
        bytes.extend(values.iter().flat_map(|value| value.to_le_bytes()));
        let offset = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let count = u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()) as usize;
        for entry in bytes[offset + 2..offset + 2 + 12 * count].chunks_exact_mut(12) {
            if u16::from_le_bytes([entry[0], entry[1]]) == tag {
                entry[4..8].copy_from_slice(&(values.len() as u32).to_le_bytes());
                entry[8..12].copy_from_slice(&pointer.to_le_bytes());
                return;
            }
        }
        panic!("fixture tag {tag} must exist");
    }

    #[test]
    fn separate_plane_cmyka_tiff_reads_all_five_planes() {
        let profile = profiles::lut16_cmyk_profile();
        let original = samples();
        let mut bytes = file(Some(&profile));
        let start = bytes.len() as u32;
        for channel in 0..5 {
            bytes.extend(original.chunks_exact(5).map(|pixel| pixel[channel]));
        }
        replace_long_array(
            &mut bytes,
            273,
            &(0..5).map(|n| start + n * 4).collect::<Vec<_>>(),
        );
        replace_long_array(&mut bytes, 279, &[4; 5]);
        patch_tag(&mut bytes, 284, 2);
        let document = crate::raster::document_from_bytes(&bytes, "planar").unwrap();
        let saved = original_cmyk_tiff(&document.pages[0].layers[0].shapes[0], 300.0).unwrap();
        let (actual, embedded) = tiff_parts(&saved);
        assert_eq!(actual, original);
        assert_eq!(embedded, Some(profile));
    }

    #[test]
    fn bigtiff_cmyk_uses_the_same_profile_and_ink_preservation_path() {
        let profile = profiles::lut16_cmyk_profile();
        let samples = vec![17, 33, 49, 65, 251, 127, 3, 9];
        let mut bytes = Cursor::new(Vec::new());
        {
            let mut encoder = tiff::encoder::TiffEncoder::new_big(&mut bytes).unwrap();
            let mut image = encoder
                .new_image::<tiff::encoder::colortype::CMYK8>(2, 1)
                .unwrap();
            image
                .encoder()
                .write_tag(tiff::tags::Tag::IccProfile, profile.as_slice())
                .unwrap();
            image.write_data(&samples).unwrap();
        }
        let doc = crate::raster::document_from_bytes(&bytes.into_inner(), "big").unwrap();
        let saved = original_cmyk_tiff(&doc.pages[0].layers[0].shapes[0], 300.0).unwrap();
        let (actual, embedded) = tiff_parts(&saved);
        assert_eq!(actual, samples);
        assert_eq!(embedded, Some(profile));
    }

    #[test]
    fn malformed_or_oversized_tiff_is_rejected_before_pixel_decoding() {
        let valid = file(None);
        for length in [0, 1, 4, 7, valid.len() - 1] {
            assert!(read(&valid[..length]).is_err());
        }
        let mut bomb = valid.clone();
        patch_tag(&mut bomb, 256, u32::MAX);
        assert!(read(&bomb)
            .err()
            .unwrap()
            .to_string()
            .contains("pixel budget"));
        let mut associated = valid;
        patch_tag(&mut associated, 338, 1);
        assert!(read(&associated).is_err());
    }

    #[test]
    fn edited_pixels_cannot_be_exported_as_untouched_original_inks() {
        let mut doc = crate::raster::document_from_bytes(&file(None), "original").unwrap();
        let shape = &mut doc.pages[0].layers[0].shapes[0];
        if let ShapeKind::Bitmap { png, .. } = &mut shape.kind {
            *png = vec![1, 2, 3].into();
        }
        assert!(original_cmyk_tiff(shape, 300.0).is_err());
    }
}
