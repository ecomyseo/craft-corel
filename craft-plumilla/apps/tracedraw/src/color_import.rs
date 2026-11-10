//! Pure CMYK import preparation. No application state, identifier allocation,
//! font installation, history or process-wide display engine is touched here.
use std::collections::HashMap;
use tracedraw_core::color_convert::{self, DocumentColorState};
use tracedraw_core::{Color, Command, Document, DocumentProfile, ShapeKind};
use tracedraw_io::picture_profile::{self, CmykConverter, Destination};

const BUILTIN_NAME: &str = "Generic CMYK (open)";

#[derive(Clone, Debug)]
pub struct ImportColorOptions {
    pub policy: u8,
    /// Actual default-profile bytes resolved before entering this pure builder.
    /// None means the explicit generic press calibration, not a named ICC.
    pub default_cmyk: Option<DocumentProfile>,
    pub bitmap: picture_profile::Options,
    pub warn_missing: bool,
    pub warn_mismatch: bool,
}

pub struct ImportColorPlan {
    pub incoming: Document,
    pub destination_commands: Vec<Command>,
    pub notice: Option<String>,
    pub processed_bitmap_pixels: u64,
}

fn target(profile: Option<&DocumentProfile>) -> Destination<'_> {
    profile
        .map(Destination::Icc)
        .unwrap_or(Destination::Builtin)
}

fn validate(profile: Option<&DocumentProfile>) -> Result<(), String> {
    if let Some(profile) = profile {
        if profile.data.len() > 16 * 1024 * 1024 {
            return Err("ICC profile exceeds its size budget".into());
        }
        let parsed = tracedraw_core::icc::Profile::parse(&profile.data)
            .map_err(|error| error.to_string())?;
        if parsed.color_space() != tracedraw_core::icc::ColorSpace::Cmyk {
            return Err("a CMYK working profile is required".into());
        }
    }
    Ok(())
}

fn same(left: Option<&DocumentProfile>, right: Option<&DocumentProfile>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left.data == right.data,
        _ => false,
    }
}

fn set_calibration(doc: &mut Document, profile: Option<&DocumentProfile>) {
    doc.cmyk_profile = profile.cloned();
    doc.metadata.cmyk_profile = profile
        .map(|profile| profile.name.as_str())
        .unwrap_or(BUILTIN_NAME)
        .to_string();
}

/// A dimension keeps its regenerating style in app-owned object data. Read its
/// typed style, but replace only the two colour fields in the original object:
/// measurements, anchors, extension fields and unrelated data remain intact.
/// This adapter is also usable when remapping spot-ink resource identifiers.
pub(crate) fn try_map_dimension_colors(
    doc: &mut Document,
    f: &mut dyn FnMut(Color) -> Result<Color, String>,
) -> Result<(), String> {
    color_convert::try_map_document_shape_tree(doc, &mut |shape| {
        if !matches!(shape.kind, ShapeKind::Group { .. }) {
            return Ok(());
        }
        let Some((_, encoded)) = shape
            .data
            .iter_mut()
            .find(|(key, _)| key == crate::dimensions::DATA_KEY)
        else {
            return Ok(());
        };
        let mut value: serde_json::Value =
            serde_json::from_str(encoded).map_err(|error| error.to_string())?;
        let style: crate::dimensions::DimensionStyle = value
            .get("style")
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        let line = f(style.line_color)?;
        let text = f(style.text_color)?;
        if line == style.line_color && text == style.text_color {
            return Ok(());
        }
        let fields = value
            .as_object_mut()
            .ok_or_else(|| "invalid measurement object data".to_string())?;
        let style = fields
            .entry("style")
            .or_insert_with(|| serde_json::Value::Object(Default::default()))
            .as_object_mut()
            .ok_or_else(|| "invalid measurement style object".to_string())?;
        for (key, colour) in [("line_color", line), ("text_color", text)] {
            style.insert(
                key.into(),
                serde_json::to_value(colour).map_err(|error| error.to_string())?,
            );
        }
        *encoded = serde_json::to_string(&value).map_err(|error| error.to_string())?;
        Ok(())
    })?;
    Ok(())
}

fn recalibrate(
    doc: &mut Document,
    source: Option<&DocumentProfile>,
    destination: Option<&DocumentProfile>,
    convert: bool,
    options: picture_profile::Options,
) -> Result<u64, String> {
    validate(destination)?;
    if convert {
        let converter = CmykConverter::new(source, target(destination), options)
            .map_err(|error| error.to_string())?;
        let mut cache = HashMap::<[u32; 4], [f32; 4]>::new();
        let mut map = |colour| {
            let Color::Cmyk { c, m, y, k } = colour else {
                return Ok(colour);
            };
            let input = [c, m, y, k];
            let key = input.map(f32::to_bits);
            let output = if let Some(output) = cache.get(&key) {
                *output
            } else {
                let output = converter
                    .convert(input)
                    .map_err(|error| error.to_string())?;
                if cache.len() < 65_536 {
                    cache.insert(key, output);
                }
                output
            };
            let [c, m, y, k] = output;
            Ok(Color::Cmyk { c, m, y, k })
        };
        color_convert::try_map_document_colors(doc, &mut map)?;
        try_map_dimension_colors(doc, &mut map)?;
    }
    let mut pixels = 0u64;
    color_convert::try_map_document_shape_tree(doc, &mut |shape| {
        if shape.cmyk_bitmap {
            if let ShapeKind::Bitmap {
                width_px,
                height_px,
                ..
            } = &shape.kind
            {
                pixels = pixels.saturating_add(u64::from(*width_px) * u64::from(*height_px));
            }
        }
        if let Some(updated) =
            picture_profile::reprofile_to(shape, source, target(destination), convert, options)
                .map_err(|error| error.to_string())?
        {
            shape.kind = updated.kind;
            shape.picture_inks = updated.picture_inks;
        }
        Ok(())
    })?;
    set_calibration(doc, destination);
    Ok(pixels)
}

/// Preserve each bitmap's own calibration and samples while materializing an
/// interpreted working default. Used for policy 2's incoming file and for an
/// untagged destination that inherits an actual installed default. Invalid
/// retained data is detected before committing any destination changes.
fn keep_incoming(
    doc: &mut Document,
    effective: Option<&DocumentProfile>,
    options: picture_profile::Options,
) -> Result<u64, String> {
    validate(effective)?;
    let mut pixels = 0u64;
    color_convert::try_map_document_shape_tree(doc, &mut |shape| {
        if !shape.cmyk_bitmap {
            return Ok(());
        }
        if let ShapeKind::Bitmap {
            width_px,
            height_px,
            ..
        } = &shape.kind
        {
            pixels = pixels.saturating_add(u64::from(*width_px) * u64::from(*height_px));
        }
        let own = shape
            .picture_inks
            .as_ref()
            .and_then(|inks| inks.profile.as_ref())
            .cloned();
        let profile = own.as_ref().or(effective);
        if let Some(updated) =
            picture_profile::reprofile_to(shape, effective, target(profile), true, options)
                .map_err(|error| error.to_string())?
        {
            // An untagged bitmap uses the selected file-default calibration.
            // When it changes only interpretation, the ink samples stay exact.
            shape.kind = updated.kind;
            shape.picture_inks = updated.picture_inks;
        }
        Ok(())
    })?;
    set_calibration(doc, effective);
    Ok(pixels)
}

/// Prepare all resources before a caller remaps identifiers or installs fonts.
/// Policy 0 converts incoming content; policy 1 assigns destination calibration;
/// policy 2 changes the existing document and preserves incoming colour values.
pub fn prepare_import_color(
    destination: &Document,
    mut incoming: Document,
    options: &ImportColorOptions,
) -> Result<ImportColorPlan, String> {
    if options.policy > 2 {
        return Err("unknown CMYK import policy".into());
    }
    color_convert::validate_document_color_tree(destination)?;
    color_convert::validate_document_color_tree(&incoming)?;
    let destination_effective = destination
        .cmyk_profile
        .as_ref()
        .or(options.default_cmyk.as_ref());
    let incoming_effective = incoming
        .cmyk_profile
        .clone()
        .or_else(|| options.default_cmyk.clone());
    let mut missing = incoming.cmyk_profile.is_none();
    let mut mismatch = !same(incoming_effective.as_ref(), destination_effective);
    // Compare all embedded bitmap profiles by bytes, not by display names.
    color_convert::try_visit_document_shape_tree(&incoming, &mut |shape| {
        if shape.cmyk_bitmap {
            let own = shape
                .picture_inks
                .as_ref()
                .and_then(|inks| inks.profile.as_ref());
            missing |= own.is_none() && incoming.cmyk_profile.is_none();
            if let Some(profile) = own {
                mismatch |= !same(Some(profile), destination_effective);
            }
        }
        Ok(())
    })?;
    let mut warnings = Vec::new();
    if options.warn_missing && missing {
        warnings.push(crate::i18n::tr("color_mgmt.warn_missing"));
    }
    if options.warn_mismatch && mismatch {
        warnings.push(crate::i18n::tr("color_mgmt.warn_mismatch_data"));
    }
    let before = DocumentColorState::capture(destination);
    let mut prepared_destination = destination.clone();
    let processed_bitmap_pixels = if options.policy == 2 {
        let changed = recalibrate(
            &mut prepared_destination,
            destination_effective,
            incoming_effective.as_ref(),
            true,
            options.bitmap,
        )?;
        changed.saturating_add(keep_incoming(
            &mut incoming,
            incoming_effective.as_ref(),
            options.bitmap,
        )?)
    } else {
        // Materialize a real default used by an untagged destination, so future
        // display does not reinterpret imported numbers through another default.
        // Its untagged pictures need their interpreted preview refreshed too;
        // each picture that already carries an ICC retains that own calibration.
        let destination_pixels =
            if destination.cmyk_profile.is_none() && destination_effective.is_some() {
                keep_incoming(
                    &mut prepared_destination,
                    destination_effective,
                    options.bitmap,
                )?
            } else {
                0
            };
        destination_pixels.saturating_add(recalibrate(
            &mut incoming,
            incoming_effective.as_ref(),
            destination_effective,
            options.policy == 0,
            options.bitmap,
        )?)
    };
    let after = DocumentColorState::capture(&prepared_destination);
    let destination_commands = if before == after {
        Vec::new()
    } else {
        vec![Command::SetDocumentColorState {
            before: Box::new(before),
            after: Box::new(after),
        }]
    };
    Ok(ImportColorPlan {
        incoming,
        destination_commands,
        notice: (!warnings.is_empty()).then(|| warnings.join(" | ")),
        processed_bitmap_pixels,
    })
}
