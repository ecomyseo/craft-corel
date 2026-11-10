//! Versioned private unit preferences, separate from standard CDR fields.
//!
//! These settings are for this reader/writer only. MCFG is opaque: a bounded
//! fingerprint detects accidentally stale preferences after another editor
//! changed the configuration. It is not authentication or a cryptographic hash.

use std::fmt;
use std::io::Read;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use tracedraw_core::Document;

use crate::parse::ParseReport;
use crate::riff::Tree;

pub(crate) const MEMBER: &str = "craft-corel/document-units.json";
pub(crate) const CHUNK: &[u8; 4] = b"ccun";
pub(crate) const MAX_MEMBER: usize = 1024;
const MAX_CONFIGURATION: usize = 65_536;
const OWNER: &str = "craft-corel.document-units";
const FORMAT: u32 = 1;

/// A required JSON field whose value can explicitly be null. An ordinary
/// Option also accepts an absent field, which would lose the distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(transparent)]
struct StoredUnit(Option<u32>);

impl<'de> Deserialize<'de> for StoredUnit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UnitVisitor;
        impl Visitor<'_> for UnitVisitor {
            type Value = StoredUnit;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an explicit null or unsigned 32-bit integer")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StoredUnit(None))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                u32::try_from(value)
                    .map(|value| StoredUnit(Some(value)))
                    .map_err(|_| E::invalid_value(de::Unexpected::Unsigned(value), &self))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                u32::try_from(value)
                    .map(|value| StoredUnit(Some(value)))
                    .map_err(|_| E::invalid_value(de::Unexpected::Signed(value), &self))
            }
        }
        deserializer.deserialize_any(UnitVisitor)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Binding {
    bytes: u32,
    fnv1a64: String,
}

impl Binding {
    fn of(data: &[u8]) -> Result<Self, String> {
        if data.is_empty() || data.len() > MAX_CONFIGURATION {
            return Err("MCFG exceeds the private unit binding budget or is empty".into());
        }
        let mut fingerprint = 0xcbf29ce484222325u64;
        for byte in data {
            fingerprint ^= u64::from(*byte);
            fingerprint = fingerprint.wrapping_mul(0x100000001b3);
        }
        Ok(Self {
            bytes: data.len() as u32,
            fnv1a64: format!("{fingerprint:016x}"),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    owner: String,
    format: u32,
    mcfg: Binding,
    script_unit: StoredUnit,
    horizontal_unit: StoredUnit,
    vertical_unit: StoredUnit,
}

pub(crate) fn encode(doc: &Document, configuration: &[u8]) -> Result<Vec<u8>, String> {
    let member = Member {
        owner: OWNER.into(),
        format: FORMAT,
        mcfg: Binding::of(configuration)?,
        script_unit: StoredUnit(doc.metadata.script_unit),
        horizontal_unit: StoredUnit(doc.metadata.rulers.horizontal_unit),
        vertical_unit: StoredUnit(doc.metadata.rulers.vertical_unit),
    };
    let data = serde_json::to_vec(&member).map_err(|error| error.to_string())?;
    if data.len() > MAX_MEMBER {
        return Err("private unit preferences exceed their byte budget".into());
    }
    Ok(data)
}

fn decode(data: &[u8], binding: &Binding) -> Result<Member, String> {
    if data.len() > MAX_MEMBER {
        return Err("private unit preferences exceed their byte budget".into());
    }
    let member: Member = serde_json::from_slice(data).map_err(|error| error.to_string())?;
    if member.owner != OWNER {
        return Err("private unit preferences have a different owner".into());
    }
    if member.format != FORMAT {
        return Err("private unit preferences use an unsupported layout".into());
    }
    if member.mcfg != *binding {
        return Err("private unit preferences no longer match MCFG".into());
    }
    Ok(member)
}

fn configuration(tree: &Tree, main: &[u8]) -> Result<Binding, String> {
    let mut count = 0usize;
    let mut binding = None;
    tree.root.walk(&mut |chunk, _| {
        if chunk.list_type.is_none() && chunk.id == *b"mcfg" {
            count = count.saturating_add(1);
            if count == 1 {
                binding = Some(Binding::of(tree.data(main, chunk)));
            }
        }
    });
    if count != 1 {
        return Err("private unit preferences require exactly one MCFG".into());
    }
    binding.ok_or("private unit preferences have no MCFG")?
}

fn riff_member(tree: &Tree, main: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut count = 0usize;
    let mut member = None;
    tree.root.walk(&mut |chunk, _| {
        if chunk.list_type.is_none() && chunk.id == *CHUNK {
            count = count.saturating_add(1);
            if count == 1 {
                let data = tree.data(main, chunk);
                member = Some(if data.len() <= MAX_MEMBER {
                    Ok(data.to_vec())
                } else {
                    Err("private RIFF unit preferences exceed their byte budget".to_string())
                });
            }
        }
    });
    if count > 1 {
        return Err("duplicate private RIFF unit preference chunks".into());
    }
    member.transpose()
}

fn member_alias(name: &str) -> bool {
    let normalized = name.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/").eq_ignore_ascii_case(MEMBER)
}

fn zip_member(bytes: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|error| error.to_string())?;
    let mut count = 0usize;
    let mut canonical = false;
    for name in archive.file_names().filter_map(Result::ok) {
        if member_alias(&name) {
            count = count.saturating_add(1);
            canonical = name == MEMBER;
        }
    }
    if count == 0 {
        return Ok(None);
    }
    // ZipArchive's name index deduplicates names. Its separate raw-entry
    // flag is needed as well as the case/separator-alias check above.
    if archive.has_duplicate_names() || count != 1 || !canonical {
        return Err("duplicate or noncanonical private ZIP unit preference members".into());
    }
    archive
        .validate_header_consistency()
        .map_err(|error| error.to_string())?;
    let options = zip::ZipReadOptions::new().enforce_header_consistency(true);
    let file = archive
        .by_name_with_options(MEMBER, options)
        .map_err(|error| error.to_string())?;
    if file.size() > MAX_MEMBER as u64 {
        return Err("private ZIP unit preferences declare too many bytes".into());
    }
    let mut data = Vec::new();
    file.take(MAX_MEMBER as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|error| error.to_string())?;
    if data.len() > MAX_MEMBER {
        return Err("private ZIP unit preferences inflate past their byte budget".into());
    }
    Ok(Some(data))
}

/// Apply all three own settings together only after the container, schema
/// and configuration binding pass. Standard unit fields are never inferred.
pub(crate) fn restore(
    doc: &mut Document,
    report: &mut ParseReport,
    tree: &Tree,
    main: &[u8],
    zip: Option<&[u8]>,
) {
    let result: Result<(), String> = (|| {
        let riff = riff_member(tree, main)?;
        let data = if let Some(bytes) = zip {
            let zipped = zip_member(bytes)?;
            if riff.is_some() {
                return Err(
                    "private RIFF unit preferences are not allowed in a ZIP container".into(),
                );
            }
            zipped
        } else {
            riff
        };
        let Some(data) = data else {
            return Ok(());
        };
        let binding = configuration(tree, main)?;
        let member = decode(&data, &binding)?;
        doc.metadata.script_unit = member.script_unit.0;
        doc.metadata.rulers.horizontal_unit = member.horizontal_unit.0;
        doc.metadata.rulers.vertical_unit = member.vertical_unit.0;
        Ok(())
    })();
    if let Err(error) = result {
        report.warn(format!("Craft Plumilla unit preferences ignored: {error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_nullable_fields_do_not_accept_missing_or_duplicate_preferences() {
        let doc = Document::new(
            "Unit schema",
            tracedraw_core::geometry::Size::new(40.0, 20.0),
        );
        let data = encode(&doc, b"hello").unwrap();
        let binding = Binding::of(b"hello").unwrap();
        let decoded = decode(&data, &binding).unwrap();
        assert_eq!(decoded.script_unit.0, None);
        assert_eq!(binding.fnv1a64, "a430d84680aabd0b");
        let text = String::from_utf8(data).unwrap();
        assert!(decode(
            text.replace("\"script_unit\":null,", "").as_bytes(),
            &binding
        )
        .is_err());
        assert!(decode(
            text.replace(
                "\"script_unit\":null",
                "\"script_unit\":null,\"script_unit\":3"
            )
            .as_bytes(),
            &binding
        )
        .is_err());
        for invalid in ["-1", "1.5", "true", "\"3\"", "4294967296"] {
            assert!(decode(
                text.replace(
                    "\"script_unit\":null",
                    &format!("\"script_unit\":{invalid}")
                )
                .as_bytes(),
                &binding
            )
            .is_err());
        }
    }

    #[test]
    fn unknown_representable_values_are_kept_without_unit_interpretation() {
        let mut doc = Document::new(
            "Unknown own units",
            tracedraw_core::geometry::Size::new(40.0, 20.0),
        );
        doc.metadata.script_unit = Some(u32::MAX);
        doc.metadata.rulers.horizontal_unit = Some(17);
        doc.metadata.rulers.vertical_unit = Some(0);
        let data = encode(&doc, b"hello").unwrap();
        let decoded = decode(&data, &Binding::of(b"hello").unwrap()).unwrap();
        assert_eq!(decoded.script_unit.0, Some(u32::MAX));
        assert_eq!(decoded.horizontal_unit.0, Some(17));
        assert_eq!(decoded.vertical_unit.0, Some(0));
        assert!(decode(&data, &Binding::of(b"Hello").unwrap()).is_err());
    }
}
