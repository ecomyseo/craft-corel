//! The members of a `.cdr` package that become parts of the document:
//! the colour profile, the embedded fonts, the document palette and the
//! page preview. Detecting the container and reading its streams is
//! [`tracedraw_cdr_format::container`], re-exported here whole so every
//! path through `tracedraw_cdr::container` stays the same.

pub use tracedraw_cdr_format::container::*;

use std::io::Read;
use tracedraw_core::Color;

/// Embedded members larger than this are skipped (fonts are far smaller;
/// this keeps a damaged size from taking all memory).
const MAX_EMBEDDED: u64 = 64 << 20;

/// The small ZIP colour-context member names the document's intent. Invalid,
/// absent or unknown values remain unset so the application's default applies.
/// This is not a profile conversion and does not infer black-point settings.
pub fn document_rendering_intent(
    bytes: &[u8],
    container: &Container,
) -> Option<tracedraw_core::icc::Intent> {
    let raw = own_member(bytes, container, "color/color.xml", 64 << 10)?;
    let text = std::str::from_utf8(&raw).ok()?;
    let xml = roxmltree::Document::parse(text).ok()?;
    let root = xml.root_element();
    if !root.has_tag_name("Color") {
        return None;
    }
    let mut contexts = root.children().filter(|n| n.has_tag_name("ColorContext"));
    let context = contexts.next()?;
    if contexts.next().is_some() {
        return None;
    }
    let mut intents = context
        .children()
        .filter(|n| n.has_tag_name("RenderingIntent"));
    let intent = intents.next()?;
    if intents.next().is_some() || intent.children().any(|n| n.is_element()) {
        return None;
    }
    let value: String = intent
        .children()
        .filter(|n| n.is_text())
        .filter_map(|n| n.text())
        .collect();
    use tracedraw_core::icc::Intent;
    match value.trim() {
        "Perceptual" => Some(Intent::Perceptual),
        "RelativeColorimetric" => Some(Intent::RelativeColorimetric),
        "Saturation" => Some(Intent::Saturation),
        "AbsoluteColorimetric" => Some(Intent::AbsoluteColorimetric),
        _ => None,
    }
}

/// The CMYK colour profile a version 14+ ZIP file carries (a member
/// under `color/profiles/cmyk/`, seen in version 17 to 21 files next to
/// `color/color.xml`, which names it), when it reads as an ICC profile.
/// `None` for RIFF files and files without one.
/// The picture of the first page a ZIP container keeps (`previews/` from
/// version 17, `metadata/thumbnails/` before), when there is one.
pub fn page_preview(bytes: &[u8], container: &Container) -> Option<crate::spot::Preview> {
    if container.kind == ContainerKind::Riff {
        return None;
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    for name in ["previews/page1.png", "metadata/thumbnails/page1.bmp"] {
        let Ok(f) = archive.by_name(name) else {
            continue;
        };
        if f.size() > MAX_PROFILE {
            continue;
        }
        let mut data = Vec::new();
        if std::io::Read::read_to_end(&mut f.take(MAX_PROFILE), &mut data).is_err() {
            continue;
        }
        if let Some(p) = crate::spot::Preview::decode(&data) {
            return Some(p);
        }
    }
    None
}

pub fn cmyk_profile(
    bytes: &[u8],
    container: &Container,
) -> Option<tracedraw_core::DocumentProfile> {
    document_profile(bytes, container, "cmyk", tracedraw_core::icc::ColorSpace::Cmyk, "Cmyk")
}

pub fn rgb_profile(bytes: &[u8], container: &Container) -> Option<tracedraw_core::DocumentProfile> {
    document_profile(bytes, container, "rgb", tracedraw_core::icc::ColorSpace::Rgb, "Rgb")
}

pub fn gray_profile(bytes: &[u8], container: &Container) -> Option<tracedraw_core::DocumentProfile> {
    document_profile(bytes, container, "grayscale", tracedraw_core::icc::ColorSpace::Gray, "Grayscale")
}

fn color_xml(bytes: &[u8], container: &Container) -> Option<String> {
    String::from_utf8(own_member(bytes, container, "color/color.xml", 64 << 10)?).ok()
}

pub(crate) fn declared_profile_name(bytes: &[u8], container: &Container, id: &str) -> Option<String> {
    let text = color_xml(bytes, container)?;
    let xml = roxmltree::Document::parse(&text).ok()?;
    let root = xml.root_element();
    if !root.has_tag_name("Color") { return None; }
    let context = root.children().find(|node| node.has_tag_name("ColorContext"))?;
    let name = context.children()
        .find(|node| node.has_tag_name("ColorProfile") && node.attribute("id").is_some_and(|value| value.eq_ignore_ascii_case(id)))?
        .text()?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

pub fn document_color_context(bytes: &[u8], container: &Container) -> Option<tracedraw_core::document::DocumentColorContext> {
    let text = color_xml(bytes, container)?;
    let xml = roxmltree::Document::parse(&text).ok()?;
    let root = xml.root_element();
    if !root.has_tag_name("Color") { return None; }
    let context = root.children().find(|node| node.has_tag_name("ColorContext"))?;
    let flag = |name| root.children().find(|node| node.has_tag_name(name)).and_then(|node| node.text()).and_then(|value| match value.trim() {
        "true" | "1" => Some(true), "false" | "0" => Some(false), _ => None,
    });
    Some(tracedraw_core::document::DocumentColorContext {
        primary_model: context.children().find(|node| node.has_tag_name("ColorModel")).and_then(|node| node.text()).unwrap_or("").trim().to_owned(),
        has_rgb_objects: flag("HasRgbObjects"),
        has_cmyk_objects: flag("HasCmykObjects"),
        has_gray_objects: flag("HasGrayscaleObjects"),
    })
}

fn document_profile(
    bytes: &[u8], container: &Container, folder: &str,
    space: tracedraw_core::icc::ColorSpace, id: &str,
) -> Option<tracedraw_core::DocumentProfile> {
    if container.kind == ContainerKind::Riff {
        return None;
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).ok()?;
    let mut names: Vec<String> = archive
        .file_names()
        .filter_map(|n| n.ok())
        .filter(|n| {
            let lower = n.to_ascii_lowercase();
            lower.starts_with(&format!("color/profiles/{folder}/"))
                && (lower.ends_with(".icc") || lower.ends_with(".icm"))
        })
        .map(String::from)
        .collect();
    names.sort();
    for name in names {
        let Ok(f) = archive.by_name(&name) else {
            continue;
        };
        if f.size() > MAX_PROFILE {
            continue;
        }
        let mut data = Vec::new();
        if f.take(MAX_PROFILE).read_to_end(&mut data).is_err() {
            continue;
        }
        let Ok(profile) = tracedraw_core::icc::Profile::parse(&data) else {
            continue;
        };
        if profile.color_space() != space {
            continue;
        }
        let file = name.rsplit('/').next().unwrap_or(&name);
        let label = declared_profile_name(bytes, container, id)
            .or_else(|| profile.description().map(String::from))
            .unwrap_or_else(|| file.to_string());
        return Some(tracedraw_core::DocumentProfile::new(label, data));
    }
    None
}

/// The largest colour profile read from a file.
const MAX_PROFILE: u64 = 16 << 20;

/// The fonts a ZIP file embeds: each `embed/embedding<n>` member holds a
/// header of its own and then an EOT structure (see [`crate::eot`]).
/// Returns the fonts, without repeats, and a note for each member that
/// could not be read.
pub fn embedded_fonts(
    bytes: &[u8],
    container: &Container,
) -> (Vec<tracedraw_core::DocumentFont>, Vec<String>) {
    let mut fonts: Vec<tracedraw_core::DocumentFont> = Vec::new();
    let mut notes = Vec::new();
    if container.kind == ContainerKind::Riff {
        return (fonts, notes);
    }
    let cursor = std::io::Cursor::new(bytes);
    let Ok(mut archive) = zip::ZipArchive::new(cursor) else {
        return (fonts, notes);
    };
    // Next to each font, newer files keep pictures of it and a sample
    // ("embedding0.png", "embedding0-Extended.png",
    // "embedding0.sample.curve"): not fonts, left alone.
    const NOT_FONTS: [&str; 7] = [".png", ".curve", ".jpg", ".jpeg", ".bmp", ".xml", ".json"];
    let mut names: Vec<String> = archive
        .file_names()
        .filter_map(|n| n.ok())
        .filter(|n| {
            let lower = n.to_ascii_lowercase();
            lower.starts_with("embed/")
                && !n.ends_with('/')
                && !NOT_FONTS.iter().any(|e| lower.ends_with(e))
        })
        .map(String::from)
        .collect();
    names.sort();
    for name in names {
        let Ok(f) = archive.by_name(&name) else {
            continue;
        };
        if f.size() > MAX_EMBEDDED {
            notes.push(format!("embedded member {name} is too large, skipped"));
            continue;
        }
        let mut member = Vec::new();
        if f.take(MAX_EMBEDDED).read_to_end(&mut member).is_err() {
            notes.push(format!("embedded member {name} could not be unpacked"));
            continue;
        }
        let Some(at) = crate::eot::find(&member) else {
            notes.push(format!(
                "embedded member {name} holds no font that can be read"
            ));
            continue;
        };
        match member.get(at..).map(crate::eot::read) {
            Some(Ok(font)) => {
                if !fonts.iter().any(|f| f.data.as_slice() == font.data) {
                    fonts.push(tracedraw_core::DocumentFont::new(font.family, font.data));
                }
            }
            Some(Err(e)) => notes.push(format!("embedded font in {name}: {e}")),
            None => {}
        }
    }
    (fonts, notes)
}

/// The document palette of an version 14+ ZIP file (`color/docPalette.xml`):
/// each `<color cs=".." name=".." tints=".."/>` with its name. CMYK, RGB
/// and grey colours are read; other colour spaces are skipped.
pub fn document_palette(bytes: &[u8], container: &Container) -> Vec<(String, Color)> {
    if container.kind == ContainerKind::Riff {
        return Vec::new();
    }
    let cursor = std::io::Cursor::new(bytes);
    let Ok(mut archive) = zip::ZipArchive::new(cursor) else {
        return Vec::new();
    };
    let mut xml = String::new();
    for name in ["color/docPalette.xml", "color/DocumentPalette.xml"] {
        if let Ok(f) = archive.by_name(name) {
            let mut raw = Vec::new();
            if f.take(MAX_PROFILE).read_to_end(&mut raw).is_ok() {
                xml = String::from_utf8_lossy(&raw).into_owned();
                break;
            }
        }
    }
    parse_palette_xml(&xml)
}

/// The colours of a palette XML document, in order.
pub fn parse_palette_xml(xml: &str) -> Vec<(String, Color)> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find("<color ") {
        rest = &rest[at + 7..];
        let end = rest.find('>').unwrap_or(rest.len());
        let tag = &rest[..end];
        rest = &rest[end..];
        let cs = attribute(tag, "cs")
            .unwrap_or_default()
            .to_ascii_uppercase();
        let name = attribute(tag, "name").unwrap_or_default();
        let tints: Vec<f32> = attribute(tag, "tints")
            .unwrap_or_default()
            .split(',')
            .filter_map(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 1.0))
            .collect();
        let color = match (cs.as_str(), tints.as_slice()) {
            ("CMYK", [c, m, y, k]) => Color::Cmyk {
                c: *c,
                m: *m,
                y: *y,
                k: *k,
            },
            ("RGB", [r, g, b]) => Color::Rgb {
                r: *r,
                g: *g,
                b: *b,
            },
            ("GRAY" | "GRAYSCALE", [v]) => Color::Gray { v: *v },
            // The other models, each component a fraction of 255 (the
            // names give the values: "C60 M169 Y255", "H119 S80 B190",
            // "H295 L165 S69", "L76 a-14 b66", "Y100 I69 Q89"; seen in a
            // version 17 file). CMY is the complement of RGB.
            ("CMY", [c, m, y]) => Color::Rgb {
                r: 1.0 - c,
                g: 1.0 - m,
                b: 1.0 - y,
            },
            ("HSB" | "HSV", [h, s, b]) => Color::Hsb {
                h: h * 360.0,
                s: *s,
                b: *b,
            },
            ("HLS", [h, l, s]) => Color::Hsl {
                h: h * 360.0,
                s: *s,
                l: *l,
            },
            ("LAB", [l, a, b]) => Color::Lab {
                l: l * 100.0,
                a: a * 255.0 - 128.0,
                b: b * 255.0 - 128.0,
            },
            ("YIQ", [y, i, q]) => Color::Yiq {
                y: *y,
                i: *i,
                q: *q,
            },
            ("REGISTRATION", _) => Color::Registration,
            _ => {
                log::warn!("document palette colour {name:?} in {cs:?} skipped");
                continue;
            }
        };
        out.push((name, color));
    }
    out
}

/// The value of `key="..."` in a tag, with the five XML entities decoded.
fn attribute(tag: &str, key: &str) -> Option<String> {
    let pat = format!("{key}=\"");
    let mut search = tag;
    loop {
        let at = search.find(&pat)?;
        // A whole attribute name, not the end of a longer one.
        let ok = at == 0 || search[..at].ends_with(char::is_whitespace);
        let after = &search[at + pat.len()..];
        if ok {
            let end = after.find('"')?;
            let v = &after[..end];
            return Some(
                v.replace("&quot;", "\"")
                    .replace("&apos;", "'")
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&"),
            );
        }
        search = after;
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn document_palette_xml_reads_names_and_tints() {
        let xml = r#"<?xml version="1.0"?><palette guid="x" name="Document Palette"><colors><page><color cs="CMYK" name="Green" tints="1,0,1,0"/><color cs="CMYK" name="Mint Green" tints="0.4,0,0.4,0"/><color cs="RGB" name="R &amp; B" tints="1,0,0.5"/><color cs="SPOT" name="Ink" tints="1"/><color cs="CMYK" name="Short" tints="1,0"/><color cs="CMYK" name="Black" tints="0,0,0,1"/></page></colors></palette>"#;
        let p = parse_palette_xml(xml);
        let names: Vec<&str> = p.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["Green", "Mint Green", "R & B", "Black"]);
        assert_eq!(
            p[1].1,
            Color::Cmyk {
                c: 0.4,
                m: 0.0,
                y: 0.4,
                k: 0.0
            }
        );
        assert_eq!(
            p[2].1,
            Color::Rgb {
                r: 1.0,
                g: 0.0,
                b: 0.5
            }
        );
    }

    #[test]
    fn document_palette_reads_the_other_colour_models() {
        let xml = r#"<colors><color cs="CMY" name="C60 M169 Y255" tints="0.235294,0.662745,1"/><color cs="HSB" name="H119 S80 B190" tints="0.330556,0.313725,0.745098"/><color cs="HLS" name="H295 L165 S69" tints="0.819444,0.647059,0.270588"/><color cs="LAB" name="L76 a-14 b66" tints="0.764706,0.447059,0.760784"/><color cs="YIQ" name="Y100 I69 Q89" tints="0.392157,0.270588,0.34902"/><color cs="Gray" name="Grey" tints="0.639216"/><color cs="Registration" name="Reg" tints="1,1,1,1"/><color cs="PANTONE 448 C" name="Spot" tints="1"/></colors>"#;
        let p = parse_palette_xml(xml);
        assert_eq!(p.len(), 7, "{p:?}");
        let close = |a: f32, b: f32| (a - b).abs() < 0.01;
        match p[0].1 {
            Color::Rgb { r, g, b } => assert!(close(r, 0.765) && close(g, 0.337) && close(b, 0.0)),
            other => panic!("{other:?}"),
        }
        match p[1].1 {
            Color::Hsb { h, s, b } => {
                assert!(close(h, 119.0) && close(s, 0.3137) && close(b, 0.745))
            }
            other => panic!("{other:?}"),
        }
        match p[2].1 {
            Color::Hsl { h, s, l } => {
                assert!(close(h, 295.0) && close(s, 0.2706) && close(l, 0.647))
            }
            other => panic!("{other:?}"),
        }
        match p[3].1 {
            Color::Lab { l, a, b } => assert!(close(l, 76.47) && close(a, -14.0) && close(b, 66.0)),
            other => panic!("{other:?}"),
        }
        assert!(matches!(p[4].1, Color::Yiq { .. }));
        assert!(matches!(p[5].1, Color::Gray { .. }));
        assert_eq!(p[6].1, Color::Registration);
    }

    #[test]
    fn broken_palette_xml_yields_what_it_can() {
        assert!(parse_palette_xml("").is_empty());
        assert!(parse_palette_xml("<color cs=\"CMYK\" tints=\"a,b,c,d\"").is_empty());
        let p = parse_palette_xml("<color tints=\"9,-1,0.5,0\" cs=\"cmyk\">");
        assert_eq!(
            p[0].1,
            Color::Cmyk {
                c: 1.0,
                m: 0.0,
                y: 0.5,
                k: 0.0
            }
        );
    }

    /// The bundled interface font (SIL Open Font License).
    const FONT: &[u8] = include_bytes!("../../../apps/tracedraw/assets/fonts/Selawik-Regular.ttf");

    /// A ZIP file around a written RIFF stream, with extra members.
    fn zip_file(members: &[(&str, Vec<u8>)]) -> Vec<u8> {
        use std::io::Write;
        // A bare RIFF stream, as versions before 14 write it.
        let riff = crate::write::document_to_cdr_version(&tracedraw_core::Document::default(), 12);
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut w = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            w.start_file("content/riffData.cdr", opts).expect("member");
            w.write_all(&riff).expect("write");
            for (name, data) in members {
                w.start_file(*name, opts).expect("member");
                w.write_all(data).expect("write");
            }
            w.finish().expect("zip");
        }
        buf.into_inner()
    }

    /// An `embed/` member: a header of its own, then the EOT structure.
    fn embedding(font: &[u8], family: &str) -> Vec<u8> {
        let mut m = vec![1, 0, 0, 0, 3, 8, 0, 0];
        m.extend("Version 1.0".encode_utf16().flat_map(|u| u.to_le_bytes()));
        m.extend_from_slice(&[0x5f; 300]);
        m.extend(crate::eot::wrap(font, family, 0x1000_0000));
        m
    }

    #[test]
    fn document_rendering_intent_comes_from_color_xml() {
        for (token, canonical) in [
            ("Perceptual", "Perceptual"),
            ("RelativeColorimetric", "Relative colorimetric"),
            ("Saturation", "Saturation"),
            ("AbsoluteColorimetric", "Absolute colorimetric"),
        ] {
            for content in [token.to_string(), format!("<![CDATA[{token}]]>")] {
                let xml = format!(
                    "<Color><ColorContext><ColorModel>Cmyk</ColorModel><RenderingIntent> {content} </RenderingIntent></ColorContext></Color>"
                );
                let bytes = zip_file(&[("color/color.xml", xml.into_bytes())]);
                let (doc, _) = crate::open_bytes(&bytes, "synthetic intent").expect("opens");
                assert_eq!(doc.metadata.rendering_intent, canonical, "{token}");
                assert!(doc.cmyk_profile.is_none(), "intent invents no profile");
            }
        }
    }

    #[test]
    fn document_rendering_intent_ignores_missing_unknown_or_invalid_xml() {
        for xml in [
            "",
            "<Color/>",
            "<Color><ColorContext><RenderingIntent>Unknown</RenderingIntent></ColorContext></Color>",
            "<Color><ColorContext><RenderingIntent>RelativeColorimetric",
            "<Color><RenderingIntent>Saturation</RenderingIntent></Color>",
            "<!-- <Color><ColorContext><RenderingIntent>Saturation</RenderingIntent></ColorContext></Color> -->",
            "<Color><ColorContext><RenderingIntent><Nested>Saturation</Nested></RenderingIntent></ColorContext></Color>",
            "<Color><ColorContext><RenderingIntent>Saturation</RenderingIntent></ColorContext></Color><extra/>",
            "<Color><ColorContext><RenderingIntent><!--Saturation--></RenderingIntent></ColorContext></Color>",
            "<Color><ColorContext><RenderingIntent>Saturation</RenderingIntent><RenderingIntent>Perceptual</RenderingIntent></ColorContext></Color>",
            "<Color><ColorContext><RenderingIntent>Saturation</RenderingIntent></ColorContext><ColorContext><RenderingIntent>Perceptual</RenderingIntent></ColorContext></Color>",
        ] {
            let bytes = zip_file(&[("color/color.xml", xml.as_bytes().to_vec())]);
            let (doc, _) = crate::open_bytes(&bytes, "missing intent").expect("opens");
            assert!(doc.metadata.rendering_intent.is_empty(), "{xml}");
        }
        let (doc, _) = crate::open_bytes(&zip_file(&[]), "no member").expect("opens");
        assert!(doc.metadata.rendering_intent.is_empty());
        let oversized = format!(
            "<Color><ColorContext><RenderingIntent>Saturation</RenderingIntent></ColorContext><!--{}--></Color>",
            "x".repeat(64 << 10)
        );
        let bytes = zip_file(&[("color/color.xml", oversized.into_bytes())]);
        let (doc, _) = crate::open_bytes(&bytes, "oversized XML").expect("opens");
        assert!(doc.metadata.rendering_intent.is_empty());
        let riff = crate::write::document_to_cdr_version(&tracedraw_core::Document::default(), 12);
        let (doc, _) = crate::open_bytes(&riff, "bare RIFF").expect("opens");
        assert!(doc.metadata.rendering_intent.is_empty());
    }

    #[test]
    fn the_cmyk_profile_comes_with_the_document() {
        use tracedraw_core::icc::test_profiles as tp;
        let cmyk = tp::lut16_cmyk_profile();
        let rgb = tp::matrix_trc_profile(tp::para_srgb(), tp::para_srgb(), tp::para_srgb());
        // An RGB profile in the CMYK folder and a damaged one are passed
        // over; the CMYK profile is kept, named by its description.
        let bytes = zip_file(&[
            ("color/color.xml", b"<Color/>".to_vec()),
            (
                "color/profiles/cmyk/a damaged.icc",
                b"not a profile".to_vec(),
            ),
            ("color/profiles/cmyk/b rgb.icc", rgb),
            ("color/profiles/cmyk/c press.icc", cmyk.clone()),
        ]);
        let (doc, _) = crate::open_bytes(&bytes, "colour").expect("opens");
        let p = doc.cmyk_profile.expect("profile");
        assert_eq!(p.name, "Synthetic CMYK");
        assert_eq!(p.data.as_slice(), cmyk.as_slice());
        // None in a file without one, nor in a RIFF file.
        let (doc, _) = crate::open_bytes(&zip_file(&[]), "plain").expect("opens");
        assert!(doc.cmyk_profile.is_none());
        // A bare RIFF stream, as versions before 14 write it.
        let riff = crate::write::document_to_cdr_version(&tracedraw_core::Document::default(), 12);
        assert!(crate::open_bytes(&riff, "riff")
            .expect("opens")
            .0
            .cmyk_profile
            .is_none());
    }

    #[test]
    fn all_document_profiles_and_context_survive_a_zip_roundtrip() {
        use tracedraw_core::icc::test_profiles as tp;
        let rgb = tp::matrix_trc_profile(tp::para_srgb(), tp::para_srgb(), tp::para_srgb());
        let gray = tp::Builder::new(b"mntr", b"GRAY", b"XYZ ", 2)
            .tag(b"kTRC", tp::curv_gamma(1.8)).build();
        let cmyk = tp::lut16_cmyk_profile();
        let xml = b"<Color><ColorContext><ColorModel>Rgb</ColorModel><ColorProfile id=\"Rgb\">RGB &amp; Paper</ColorProfile><ColorProfile id=\"Cmyk\">Press</ColorProfile><ColorProfile id=\"Grayscale\">Gray Paper</ColorProfile><RenderingIntent>Saturation</RenderingIntent></ColorContext><HasRgbObjects>true</HasRgbObjects><HasCmykObjects>false</HasCmykObjects><HasGrayscaleObjects>true</HasGrayscaleObjects></Color>";
        let bytes = zip_file(&[
            ("color/color.xml", xml.to_vec()),
            ("color/profiles/rgb/source.icm", rgb.clone()),
            ("color/profiles/cmyk/source.icc", cmyk.clone()),
            ("color/profiles/grayscale/source.icc", gray.clone()),
        ]);
        let (doc, _) = crate::open_bytes(&bytes, "profiles").expect("source");
        assert_eq!(doc.metadata.primary_color_mode, tracedraw_core::document::PrimaryColorMode::Rgb);
        assert_eq!(doc.rgb_profile.as_ref().expect("RGB").name, "RGB & Paper");
        assert_eq!(doc.rgb_profile.as_ref().expect("RGB").data.as_slice(), rgb.as_slice());
        assert_eq!(doc.gray_profile.as_ref().expect("gray").data.as_slice(), gray.as_slice());
        assert_eq!(doc.cmyk_profile.as_ref().expect("CMYK").data.as_slice(), cmyk.as_slice());
        let out = crate::write::document_to_cdr_version(&doc, 27);
        let (back, _) = crate::open_bytes(&out, "reopened").expect("roundtrip");
        assert_eq!(back.rgb_profile, doc.rgb_profile);
        assert_eq!(back.gray_profile, doc.gray_profile);
        assert_eq!(back.cmyk_profile, doc.cmyk_profile);
        assert_eq!(back.color_context, doc.color_context);
        assert_eq!(back.metadata.rendering_intent, doc.metadata.rendering_intent);
        assert_eq!(back.metadata.primary_color_mode, tracedraw_core::document::PrimaryColorMode::Rgb);
        let mut edited = doc.clone();
        edited.metadata.primary_color_mode = tracedraw_core::document::PrimaryColorMode::Cmyk;
        let changed = crate::write::document_to_cdr_version(&edited, 27);
        let (changed, _) = crate::open_bytes(&changed, "changed primary mode").expect("changed drawing");
        assert_eq!(changed.metadata.primary_color_mode, tracedraw_core::document::PrimaryColorMode::Cmyk);
        assert_eq!(changed.color_context.as_ref().expect("context").primary_model, "Cmyk");
    }

    #[test]
    fn a_new_rgb_drawing_writes_its_primary_mode_without_source_context() {
        let mut source = tracedraw_core::Document::new("RGB", tracedraw_core::geometry::Size::new(30.0, 20.0));
        source.metadata.primary_color_mode = tracedraw_core::document::PrimaryColorMode::Rgb;
        let bytes = crate::write::document_to_cdr_version(&source, 27);
        let (back, _) = crate::open_bytes(&bytes, "RGB").expect("RGB drawing");
        assert_eq!(back.metadata.primary_color_mode, source.metadata.primary_color_mode);
    }

    #[test]
    fn embedded_source_profiles_reject_damaged_and_wrong_colour_spaces() {
        use tracedraw_core::icc::test_profiles as tp;
        let rgb = tp::matrix_trc_profile(tp::para_srgb(), tp::para_srgb(), tp::para_srgb());
        let bytes = zip_file(&[
            ("color/profiles/rgb/broken.icc", b"invalid".to_vec()),
            ("color/profiles/rgb/cmyk.icc", tp::lut16_cmyk_profile()),
            ("color/profiles/grayscale/rgb.icc", rgb),
        ]);
        let (doc, _) = crate::open_bytes(&bytes, "bad profiles").expect("drawing still opens");
        assert!(doc.rgb_profile.is_none());
        assert!(doc.gray_profile.is_none());
    }

    #[test]
    fn embedded_fonts_come_with_the_document() {
        let bytes = zip_file(&[
            ("embed/embedding0", embedding(FONT, "Selawik")),
            // The same font again, and a member that holds none.
            ("embed/embedding1", embedding(FONT, "Selawik")),
            ("embed/embedding2", b"nothing to see".to_vec()),
            // Pictures and a sample of a font, kept next to it.
            ("embed/embedding0.png", b"\x89PNG".to_vec()),
            ("embed/embedding0-Extended.png", b"\x89PNG".to_vec()),
            ("embed/embedding0.sample.curve", b"curve".to_vec()),
        ]);
        let (doc, report) = crate::open_bytes(&bytes, "fonts").expect("opens");
        assert_eq!(doc.fonts.len(), 1);
        assert_eq!(doc.fonts[0].family, "Selawik");
        assert_eq!(doc.fonts[0].data.as_slice(), FONT);
        assert!(
            report.warnings.iter().any(|w| w.contains("embedding2")),
            "{:?}",
            report.warnings
        );
        assert!(
            !report
                .warnings
                .iter()
                .any(|w| w.contains(".png") || w.contains(".curve")),
            "{:?}",
            report.warnings
        );
        // Compressed fonts are named in the report and left out.
        let mut c = embedding(FONT, "Selawik");
        let at = crate::eot::find(&c).expect("eot");
        c[at + 12] |= 4;
        let (doc, report) =
            crate::open_bytes(&zip_file(&[("embed/embedding0", c)]), "c").expect("opens");
        assert!(doc.fonts.is_empty());
        assert!(report.warnings.iter().any(|w| w.contains("compressed")));
        // A RIFF file has no embedded members; the document still opens.
        // A bare RIFF stream, as versions before 14 write it.
        let riff = crate::write::document_to_cdr_version(&tracedraw_core::Document::default(), 12);
        let (doc, _) = crate::open_bytes(&riff, "riff").expect("opens");
        assert!(doc.fonts.is_empty());
    }

    #[test]
    fn cut_or_damaged_font_members_never_stop_the_file_opening() {
        let member = embedding(FONT, "Selawik");
        for cut in [0, 1, 10, 100, 400, member.len() / 2, member.len() - 1] {
            let bytes = zip_file(&[("embed/embedding0", member[..cut].to_vec())]);
            let (doc, _) = crate::open_bytes(&bytes, "cut").expect("opens");
            assert!(doc.fonts.is_empty(), "cut at {cut}");
        }
        let mut flipped = member.clone();
        for i in (0..flipped.len()).step_by(97) {
            flipped[i] ^= 0xa5;
        }
        let bytes = zip_file(&[("embed/embedding0", flipped)]);
        assert!(crate::open_bytes(&bytes, "flip").is_ok());
    }
}
