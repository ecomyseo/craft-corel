//! Profile assignment and conversion of retained process-CMYK bitmap samples.
//! This is a pure builder: callers commit the returned shape through commands.
use crate::{Error, Result};
use std::{collections::HashMap, io::Cursor};
use tracedraw_core::icc::{ColorSpace, Intent, Profile, Transform};
use tracedraw_core::{DocumentProfile, Shape, ShapeKind};

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub intent: Intent,
    pub black_point_compensation: bool,
    pub preserve_pure_black: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            intent: Intent::default(),
            black_point_compensation: true,
            preserve_pure_black: false,
        }
    }
}

fn error(message: impl std::fmt::Display) -> Error {
    Error::Format(message.to_string())
}

fn parse(profile: &DocumentProfile) -> Result<Profile> {
    if profile.data.len() > 16 * 1024 * 1024 {
        return Err(error("ICC profile exceeds its size budget"));
    }
    let parsed = Profile::parse(&profile.data).map_err(error)?;
    if parsed.color_space() != ColorSpace::Cmyk {
        return Err(error("a CMYK profile is required"));
    }
    Ok(parsed)
}

/// A concrete destination calibration, rather than a missing ICC file silently
/// interpreted as an unrelated named profile.
#[derive(Clone, Copy, Debug)]
pub enum Destination<'a> {
    Icc(&'a DocumentProfile),
    Builtin,
}

impl<'a> Destination<'a> {
    pub fn profile(self) -> Option<&'a DocumentProfile> {
        match self {
            Self::Icc(profile) => Some(profile),
            Self::Builtin => None,
        }
    }
}

/// Pure process-CMYK conversion shared by vectors and retained bitmap samples.
/// It never consults the installed display engine. The built-in calibration is
/// the explicit press model, not a synthesized ICC carrying another name.
pub struct CmykConverter {
    transform: Option<Transform>,
    source_builtin: bool,
    target_builtin: bool,
    identity: bool,
    preserve_black: bool,
}

impl CmykConverter {
    pub fn new(
        source: Option<&DocumentProfile>,
        target: Destination<'_>,
        options: Options,
    ) -> Result<Self> {
        let from = source.map(parse).transpose()?;
        let to = target.profile().map(parse).transpose()?;
        let identity = match (source, target.profile()) {
            (None, None) => true,
            (Some(left), Some(right)) => left.data == right.data,
            _ => false,
        };
        let builtin = Profile::srgb();
        let transform = if identity {
            None
        } else {
            Some(
                Transform::new(
                    from.as_ref().unwrap_or(&builtin),
                    to.as_ref().unwrap_or(&builtin),
                    options.intent,
                    options.black_point_compensation,
                )
                .ok_or_else(|| error("CMYK profile conversion is unavailable"))?,
            )
        };
        Ok(Self {
            transform,
            source_builtin: from.is_none(),
            target_builtin: to.is_none(),
            identity,
            preserve_black: options.preserve_pure_black,
        })
    }

    pub fn is_identity(&self) -> bool {
        self.identity
    }

    pub fn convert(&self, ink: [f32; 4]) -> Result<[f32; 4]> {
        if !ink.iter().all(|value| value.is_finite()) {
            return Err(error("CMYK values must be finite"));
        }
        if self.identity || (self.preserve_black && ink[..3] == [0.0, 0.0, 0.0]) {
            return Ok(ink);
        }
        let transform = self
            .transform
            .as_ref()
            .ok_or_else(|| error("missing CMYK transform"))?;
        let output = if self.source_builtin {
            transform.transform(&tracedraw_core::press::cmyk_to_srgb(ink))
        } else {
            transform.transform(&ink)
        };
        if !output.iter().all(|value| value.is_finite()) {
            return Err(error("ICC conversion returned non-finite channels"));
        }
        let result = if self.target_builtin {
            let [r, g, b] = output.as_slice() else {
                return Err(error("ICC conversion returned the wrong RGB channel count"));
            };
            tracedraw_core::press::srgb_to_cmyk([*r, *g, *b])
        } else {
            let [c, m, y, k] = output.as_slice() else {
                return Err(error(
                    "ICC conversion returned the wrong CMYK channel count",
                ));
            };
            [*c, *m, *y, *k]
        };
        if !result.iter().all(|value| value.is_finite()) {
            return Err(error("CMYK conversion returned non-finite channels"));
        }
        Ok(result)
    }
}

/// Assign keeps source ink bytes and changes their calibration. Convert changes
/// ink bytes through the source and destination ICCs. Each picture's own ICC has
/// priority over the inherited document ICC. Untagged input uses the local press
/// model; conversion requires an available CMYK destination ICC.
pub fn reprofile(
    shape: &Shape,
    inherited: Option<&DocumentProfile>,
    target: Option<&DocumentProfile>,
    convert: bool,
    options: Options,
) -> Result<Option<Shape>> {
    reprofile_impl(shape, inherited, target, convert, options, false)
}

/// Explicit profile conversion/assignment used by import preparation. Unlike
/// the historical API, Destination::Builtin authorizes a real conversion back
/// to generic press inks. Matching calibrations validate planes but retain the
/// exact original shape and byte buffers.
pub fn reprofile_to(
    shape: &Shape,
    inherited: Option<&DocumentProfile>,
    target: Destination<'_>,
    convert: bool,
    options: Options,
) -> Result<Option<Shape>> {
    reprofile_impl(shape, inherited, target.profile(), convert, options, true)
}

fn reprofile_impl(
    shape: &Shape,
    inherited: Option<&DocumentProfile>,
    target: Option<&DocumentProfile>,
    convert: bool,
    options: Options,
    explicit_builtin: bool,
) -> Result<Option<Shape>> {
    let ShapeKind::Bitmap {
        width_px,
        height_px,
        ..
    } = &shape.kind
    else {
        return Ok(None);
    };
    if !shape.cmyk_bitmap {
        return Ok(None);
    }
    let destination = target.map(parse).transpose()?;
    if convert && destination.is_none() && !explicit_builtin {
        if inherited.is_none()
            && shape
                .picture_inks
                .as_ref()
                .is_some_and(|inks| inks.profile.is_none())
        {
            return Ok(None);
        }
        return Err(error(
            "CMYK conversion requires an available destination ICC profile",
        ));
    }
    // Reuse the bounded, checked retained-plane reader, including straight alpha
    // and exact samples beneath transparent pixels. No RGB-to-ink reconstruction.
    let mut sample_shape = shape.clone();
    if let Some(inks) = sample_shape.picture_inks.as_mut() {
        inks.profile = None;
    }
    let bytes = crate::picture::original_cmyk_tiff(&sample_shape, 96.0)?;
    let mut decoder = tiff::decoder::Decoder::new(Cursor::new(bytes)).map_err(error)?;
    let tiff::decoder::DecodingResult::U8(mut samples) = decoder.read_image().map_err(error)?
    else {
        return Err(error("retained CMYK data is not eight-bit"));
    };
    let count = (*width_px as usize)
        .checked_mul(*height_px as usize)
        .ok_or_else(|| error("invalid bitmap dimensions"))?;
    let channels = samples
        .len()
        .checked_div(count)
        .filter(|channels| matches!(channels, 4 | 5))
        .ok_or_else(|| error("invalid retained sample layout"))?;
    if samples.len() != count * channels {
        return Err(error("incomplete retained samples"));
    }
    if convert {
        let source = shape
            .picture_inks
            .as_ref()
            .and_then(|inks| inks.profile.as_ref())
            .or(inherited);
        let destination = target.map(Destination::Icc).unwrap_or(Destination::Builtin);
        let converter = CmykConverter::new(source, destination, options)?;
        let carried = shape
            .picture_inks
            .as_ref()
            .and_then(|inks| inks.profile.as_ref());
        let same_carried = match (carried, target) {
            (None, None) => true,
            (Some(left), Some(right)) => left.data == right.data,
            _ => false,
        };
        if explicit_builtin && converter.is_identity() && same_carried {
            return Ok(None);
        }
        if !converter.is_identity() {
            let mut cache = HashMap::<[u8; 4], [u8; 4]>::new();
            for pixel in samples.chunks_exact_mut(channels) {
                let ink = [pixel[0], pixel[1], pixel[2], pixel[3]];
                if options.preserve_pure_black && ink[..3] == [0, 0, 0] {
                    continue;
                }
                let converted = if let Some(value) = cache.get(&ink) {
                    *value
                } else {
                    let input = ink.map(|v| f32::from(v) / 255.0);
                    let value = converter
                        .convert(input)?
                        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
                    if cache.len() < 65_536 {
                        cache.insert(ink, value);
                    }
                    value
                };
                if let Some(output) = pixel.get_mut(..4) {
                    output.copy_from_slice(&converted);
                }
            }
        }
    }
    let icc = target.map(|p| p.data.as_slice());
    let bytes = if channels == 5 {
        crate::cmyk_bitmap::cmyka_tiff(*width_px, *height_px, &samples, 96.0, icc)
    } else {
        crate::cmyk_bitmap::cmyk_tiff(*width_px, *height_px, &samples, 96.0, icc)
    }
    .ok_or_else(|| error("could not encode recalibrated CMYK samples"))?;
    let mut picture = crate::picture::read(&bytes)?;
    picture.profile = target.cloned();
    let (png, inks) = picture
        .into_png()
        .ok_or_else(|| error("could not encode recalibrated bitmap"))?;
    let mut result = shape.clone();
    if let ShapeKind::Bitmap { png: pixels, .. } = &mut result.kind {
        *pixels = png.into();
    }
    result.picture_inks = inks;
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::icc::test_profiles as p;

    fn profiles() -> (DocumentProfile, DocumentProfile) {
        (
            DocumentProfile::new("Original", p::intent_cmyk_profile(false)),
            DocumentProfile::new("Other", p::intent_cmyk_profile(true)),
        )
    }

    fn samples() -> Vec<u8> {
        vec![
            17, 33, 49, 65, 0, 251, 127, 3, 9, 127, 0, 0, 0, 85, 255, 11, 29, 57, 113, 255,
        ]
    }

    fn shape(profile: Option<&DocumentProfile>) -> Shape {
        let bytes = crate::cmyk_bitmap::cmyka_tiff(
            2,
            2,
            &samples(),
            300.0,
            profile.map(|p| p.data.as_slice()),
        )
        .unwrap();
        let doc = crate::raster::document_from_bytes(&bytes, "profile fixture").unwrap();
        let mut shape = doc.pages[0].layers[0].shapes[0].clone();
        shape.name = Some("Original picture".into());
        shape.data.push(("bitmap.link".into(), "source.tif".into()));
        shape.transform = tracedraw_core::geometry::Affine::translate((7.0, 9.0));
        shape
    }

    fn parts(shape: &Shape) -> (Vec<u8>, Option<Vec<u8>>, image::RgbaImage) {
        let bytes = crate::picture::original_cmyk_tiff(shape, 300.0).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(Cursor::new(bytes)).unwrap();
        let profile = decoder.get_tag_u8_vec(tiff::tags::Tag::IccProfile).ok();
        let tiff::decoder::DecodingResult::U8(samples) = decoder.read_image().unwrap() else {
            panic!("samples");
        };
        let ShapeKind::Bitmap { png, .. } = &shape.kind else {
            panic!("bitmap");
        };
        (samples, profile, crate::picture::read(png).unwrap().rgba)
    }

    #[test]
    fn explicit_builtin_destination_converts_real_inks_and_keeps_alpha() {
        let (a, _) = profiles();
        let original = shape(Some(&a));
        let options = Options::default();
        let transform = Transform::new(
            &parse(&a).unwrap(),
            &Profile::srgb(),
            options.intent,
            options.black_point_compensation,
        )
        .unwrap();
        let mut expected = samples();
        for pixel in expected.chunks_exact_mut(5) {
            let rgb = transform.transform(
                &pixel[..4]
                    .iter()
                    .map(|v| f32::from(*v) / 255.0)
                    .collect::<Vec<_>>(),
            );
            let ink = tracedraw_core::press::srgb_to_cmyk([rgb[0], rgb[1], rgb[2]]);
            for (out, value) in pixel[..4].iter_mut().zip(ink) {
                *out = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
        let updated = reprofile_to(&original, None, Destination::Builtin, true, options)
            .unwrap()
            .unwrap();
        let (actual, profile, _) = parts(&updated);
        assert_eq!(actual, expected);
        assert_ne!(actual, samples());
        assert!(profile.is_none(), "generic is not an invented ICC");
        assert_eq!(
            actual.chunks_exact(5).map(|p| p[4]).collect::<Vec<_>>(),
            vec![0, 127, 255, 255]
        );
        assert_eq!(
            original,
            shape(Some(&a)),
            "pure preparation cannot edit the source"
        );
        assert!(
            reprofile(&original, None, None, true, options).is_err(),
            "legacy absence remains explicit"
        );
    }

    #[test]
    fn explicit_builtin_source_uses_press_forward_before_icc() {
        let (a, _) = profiles();
        let options = Options::default();
        let transform = Transform::new(
            &Profile::srgb(),
            &parse(&a).unwrap(),
            options.intent,
            options.black_point_compensation,
        )
        .unwrap();
        let mut expected = samples();
        for pixel in expected.chunks_exact_mut(5) {
            let ink = [pixel[0], pixel[1], pixel[2], pixel[3]].map(|v| f32::from(v) / 255.0);
            let converted = transform.transform(&tracedraw_core::press::cmyk_to_srgb(ink));
            for (out, value) in pixel[..4].iter_mut().zip(converted) {
                *out = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
        let updated = reprofile_to(&shape(None), None, Destination::Icc(&a), true, options)
            .unwrap()
            .unwrap();
        assert_eq!(parts(&updated).0, expected);
        assert_eq!(parts(&updated).1.as_deref(), Some(a.data.as_slice()));
    }

    #[test]
    fn explicit_identity_retains_exact_shape_but_materializes_inherited_calibration() {
        let (a, _) = profiles();
        let mut same = a.clone();
        same.name = "Different label".into();
        let original = shape(Some(&a));
        assert!(reprofile_to(
            &original,
            None,
            Destination::Icc(&same),
            true,
            Options::default()
        )
        .unwrap()
        .is_none());
        let untagged = shape(None);
        let updated = reprofile_to(
            &untagged,
            Some(&a),
            Destination::Icc(&a),
            true,
            Options::default(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            parts(&updated).0,
            samples(),
            "an inherited identity does not round samples"
        );
        assert_eq!(parts(&updated).1.as_deref(), Some(a.data.as_slice()));
        assert_ne!(
            parts(&updated).2,
            parts(&untagged).2,
            "tagging refreshes the previously generic preview"
        );
        let mut stale = original.clone();
        stale.picture_inks.as_mut().unwrap().of ^= 1;
        assert!(reprofile_to(
            &stale,
            None,
            Destination::Icc(&same),
            true,
            Options::default()
        )
        .is_err());
        let converter = CmykConverter::new(None, Destination::Builtin, Options::default()).unwrap();
        assert!(converter.convert([f32::NAN, 0.0, 0.0, 0.0]).is_err());
    }

    #[test]
    fn assignment_keeps_all_inks_alpha_and_geometry_but_changes_calibration() {
        let (a, b) = profiles();
        let original = shape(Some(&a));
        let before = original.clone();
        let mut assigned = reprofile(&original, None, Some(&b), false, Options::default())
            .unwrap()
            .unwrap();
        let (samples_b, profile_b, display_b) = parts(&assigned);
        let (_, _, display_a) = parts(&original);
        assert_eq!(samples_b, samples());
        assert_eq!(profile_b.as_deref(), Some(b.data.as_slice()));
        assert_ne!(display_a, display_b);
        assert_eq!(
            display_a.pixels().map(|p| p.0[3]).collect::<Vec<_>>(),
            display_b.pixels().map(|p| p.0[3]).collect::<Vec<_>>()
        );
        assert_eq!(original, before, "builder cannot mutate the original");
        assigned.kind = original.kind.clone();
        assigned.picture_inks = original.picture_inks.clone();
        assert_eq!(
            assigned, original,
            "geometry, id, links, effects and attributes stay"
        );
        let untagged = reprofile(&original, None, None, false, Options::default())
            .unwrap()
            .unwrap();
        let (inks, icc, display) = parts(&untagged);
        assert_eq!(inks, samples());
        assert!(icc.is_none());
        assert_eq!(display, parts(&shape(None)).2);
    }

    #[test]
    fn conversion_uses_each_picture_profile_intent_bpc_and_pure_black_option() {
        let (a, b) = profiles();
        let original = shape(Some(&b));
        let from = parse(&b).unwrap();
        let to = parse(&a).unwrap();
        let mut intent_outputs = std::collections::HashSet::new();
        let mut bpc_outputs = Vec::new();
        for intent in [
            Intent::RelativeColorimetric,
            Intent::AbsoluteColorimetric,
            Intent::Perceptual,
            Intent::Saturation,
        ] {
            for bpc in [false, true] {
                for black in [false, true] {
                    let options = Options {
                        intent,
                        black_point_compensation: bpc,
                        preserve_pure_black: black,
                    };
                    // The inherited profile matches the target; using it instead
                    // of the picture's own profile would incorrectly do nothing.
                    let updated = reprofile(&original, Some(&a), Some(&a), true, options)
                        .unwrap()
                        .unwrap();
                    let transform = Transform::new(&from, &to, intent, bpc).unwrap();
                    let mut expected = samples();
                    for pixel in expected.chunks_exact_mut(5) {
                        if black && pixel[..3] == [0, 0, 0] {
                            continue;
                        }
                        let converted = transform.transform(
                            &pixel[..4]
                                .iter()
                                .map(|v| f32::from(*v) / 255.0)
                                .collect::<Vec<_>>(),
                        );
                        for (out, value) in pixel[..4].iter_mut().zip(converted) {
                            *out = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
                        }
                    }
                    let (actual, icc, display) = parts(&updated);
                    assert_eq!(actual, expected);
                    if !bpc && !black {
                        intent_outputs.insert(actual.clone());
                    }
                    if intent == Intent::RelativeColorimetric && !black {
                        bpc_outputs.push(actual.clone());
                    }
                    assert_ne!(actual, samples(), "calibrations differ");
                    assert_eq!(icc.as_deref(), Some(a.data.as_slice()));
                    assert_eq!(
                        actual.chunks_exact(5).map(|p| p[4]).collect::<Vec<_>>(),
                        vec![0, 127, 255, 255]
                    );
                    let encoded =
                        crate::cmyk_bitmap::cmyka_tiff(2, 2, &expected, 300.0, Some(&a.data))
                            .unwrap();
                    assert_eq!(display, crate::picture::read(&encoded).unwrap().rgba);
                }
            }
        }
        assert_eq!(
            intent_outputs.len(),
            4,
            "all four intents must be observably exercised"
        );
        assert_ne!(
            bpc_outputs[0], bpc_outputs[1],
            "the fixture must exercise nonidentity black point compensation"
        );
        let same = reprofile(&original, None, Some(&b), true, Options::default())
            .unwrap()
            .unwrap();
        assert_eq!(
            parts(&same).0,
            samples(),
            "identical profiles keep exact original samples"
        );
    }

    #[test]
    fn inherited_or_local_default_source_is_used_only_without_a_picture_profile() {
        let (a, b) = profiles();
        let original = shape(None);
        assert!(
            reprofile(&original, None, None, true, Options::default())
                .unwrap()
                .is_none(),
            "the same local calibration is a no-op"
        );
        let inherited = reprofile(&original, Some(&a), Some(&b), true, Options::default())
            .unwrap()
            .unwrap();
        let local = reprofile(&original, None, Some(&b), true, Options::default())
            .unwrap()
            .unwrap();
        assert_ne!(parts(&inherited).0, parts(&local).0);
        assert_eq!(
            parts(&local)
                .0
                .chunks_exact(5)
                .map(|p| p[4])
                .collect::<Vec<_>>(),
            vec![0, 127, 255, 255]
        );
    }

    #[test]
    fn invalid_profiles_or_stale_planes_fail_without_changing_the_shape() {
        let (a, _) = profiles();
        let mut original = shape(Some(&a));
        let invalid = DocumentProfile::new("Invalid", vec![0; 16]);
        let rgb = DocumentProfile::new(
            "RGB",
            p::matrix_trc_profile(p::para_srgb(), p::para_srgb(), p::para_srgb()),
        );
        for target in [Some(&invalid), Some(&rgb), None] {
            let before = original.clone();
            assert!(reprofile(&original, None, target, true, Options::default()).is_err());
            assert_eq!(before, original);
        }
        let mut repair = original.clone();
        repair.picture_inks.as_mut().unwrap().profile = Some(invalid.clone());
        let assigned = reprofile(&repair, None, Some(&a), false, Options::default())
            .unwrap()
            .unwrap();
        assert_eq!(
            parts(&assigned).0,
            samples(),
            "assignment needs raw values, not the invalid old ICC"
        );
        assert!(reprofile(&repair, None, Some(&a), true, Options::default()).is_err());
        original.picture_inks.as_mut().unwrap().of ^= 1;
        assert!(reprofile(&original, None, Some(&a), false, Options::default()).is_err());
        original.cmyk_bitmap = false;
        assert!(
            reprofile(&original, None, Some(&a), false, Options::default())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn opaque_four_channel_cmyk_stays_opaque_without_inventing_an_alpha_plane() {
        let (a, b) = profiles();
        let values = [17, 33, 49, 65, 0, 0, 0, 85, 251, 127, 3, 9];
        let tiff = crate::cmyk_bitmap::cmyk_tiff(3, 1, &values, 300.0, Some(&a.data)).unwrap();
        let doc = crate::raster::document_from_bytes(&tiff, "opaque").unwrap();
        let updated = reprofile(
            &doc.pages[0].layers[0].shapes[0],
            None,
            Some(&b),
            false,
            Options::default(),
        )
        .unwrap()
        .unwrap();
        let (actual, icc, display) = parts(&updated);
        assert_eq!(actual, values);
        assert_eq!(icc.as_deref(), Some(b.data.as_slice()));
        assert!(display.pixels().all(|p| p.0[3] == 255));
    }
}
