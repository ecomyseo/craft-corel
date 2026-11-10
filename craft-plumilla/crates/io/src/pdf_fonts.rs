//! Fonts of PDF text: which glyphs of which font a file uses, and the font
//! objects written for them at the end of the file. Embedded fonts are
//! CID fonts (Identity-H, two bytes per glyph) with a ToUnicode map, made
//! from a subset of the font file: TrueType outlines as CIDFontType2
//! (FontFile2), CFF outlines as CIDFontType0 (FontFile3, CIDFontType0C).
//! Fonts left out of the file are simple fonts in the WinAnsi encoding,
//! named for the viewer to find. See docs/behavior/publish-to-pdf.md.

use super::{f, flate, pdf_name, Pdf};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use tracedraw_core::document::text_outline::GlyphFont;

/// How text goes into a PDF file.
#[derive(Debug, Clone, PartialEq)]
pub struct PdfText {
    /// Every text as curves (filled outlines): no fonts in the file.
    pub as_curves: bool,
    /// Put the fonts in the file. Off: fonts are only named, and
    /// characters outside the WinAnsi set are drawn as curves.
    pub embed_fonts: bool,
    /// Put the standard fonts (Helvetica, Times, Courier) in the file too;
    /// off, text in those families names them for the viewer.
    pub embed_base14: bool,
    /// Embed only the glyphs the file uses.
    pub subset: bool,
    /// Subset a font only when it uses fewer than this share of its glyphs
    /// (percent); at or above it the whole font goes in.
    pub subset_below_pct: f64,
}

impl Default for PdfText {
    fn default() -> Self {
        PdfText {
            as_curves: false,
            embed_fonts: true,
            embed_base14: false,
            subset: true,
            subset_below_pct: 80.0,
        }
    }
}

/// What a font's licence allows (its OS/2 embedding bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontLicence {
    /// Embedding allowed, subsets too.
    Embeddable,
    /// Embedding allowed, but only the whole font.
    NoSubsetting,
    /// No embedding (restricted licence, or bitmaps only): text in the
    /// font is drawn as curves.
    Restricted,
}

/// The embedding the font at `index` of `data` allows. A font without an
/// OS/2 table, or one that cannot be read, counts as embeddable (its text
/// becomes curves anyway when it cannot be read).
pub fn font_licence(data: &[u8], index: u32) -> FontLicence {
    let Ok(face) = ttf_parser::RawFace::parse(data, index) else {
        return FontLicence::Embeddable;
    };
    let Some(os2) = face.table(ttf_parser::Tag::from_bytes(b"OS/2")) else {
        return FontLicence::Embeddable;
    };
    match os2.get(8..10) {
        Some(&[hi, lo]) => licence_of(u16::from_be_bytes([hi, lo])),
        _ => FontLicence::Embeddable,
    }
}

/// The licence the OS/2 `fsType` bits give. The usage bits (0 to 3) are
/// exclusive in recent fonts; older ones may set several, and then the
/// least restrictive counts.
pub fn licence_of(fs_type: u16) -> FontLicence {
    let restricted = fs_type & 0x0002 != 0 && fs_type & 0x000C == 0;
    if restricted || fs_type & 0x0200 != 0 {
        FontLicence::Restricted
    } else if fs_type & 0x0100 != 0 {
        FontLicence::NoSubsetting
    } else {
        FontLicence::Embeddable
    }
}

/// The standard font a family names, by style, when it is one of the
/// Helvetica, Times or Courier families.
pub fn base14_name(family: &str, bold: bool, italic: bool) -> Option<&'static str> {
    let key: String = family
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    let names: [&'static str; 4] = match key.as_str() {
        "helvetica" => [
            "Helvetica",
            "Helvetica-Bold",
            "Helvetica-Oblique",
            "Helvetica-BoldOblique",
        ],
        "times" | "timesroman" => [
            "Times-Roman",
            "Times-Bold",
            "Times-Italic",
            "Times-BoldItalic",
        ],
        "courier" => [
            "Courier",
            "Courier-Bold",
            "Courier-Oblique",
            "Courier-BoldOblique",
        ],
        _ => return None,
    };
    Some(names[(bold as usize) + 2 * (italic as usize)])
}

/// The WinAnsi code of `c`, if the encoding has it.
pub fn win_ansi(c: char) -> Option<u8> {
    let u = c as u32;
    if (0x20..0x7F).contains(&u) || (0xA0..=0xFF).contains(&u) {
        return Some(u as u8);
    }
    const HIGH: [(u32, u8); 27] = [
        (0x20AC, 0x80),
        (0x201A, 0x82),
        (0x0192, 0x83),
        (0x201E, 0x84),
        (0x2026, 0x85),
        (0x2020, 0x86),
        (0x2021, 0x87),
        (0x02C6, 0x88),
        (0x2030, 0x89),
        (0x0160, 0x8A),
        (0x2039, 0x8B),
        (0x0152, 0x8C),
        (0x017D, 0x8E),
        (0x2018, 0x91),
        (0x2019, 0x92),
        (0x201C, 0x93),
        (0x201D, 0x94),
        (0x2022, 0x95),
        (0x2013, 0x96),
        (0x2014, 0x97),
        (0x02DC, 0x98),
        (0x2122, 0x99),
        (0x0161, 0x9A),
        (0x203A, 0x9B),
        (0x0153, 0x9C),
        (0x017E, 0x9E),
        (0x0178, 0x9F),
    ];
    HIGH.iter().find(|(cp, _)| *cp == u).map(|(_, b)| *b)
}

/// How a font of the file is written.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum SlotKind {
    /// Embedded CID font.
    Cid,
    /// Simple font named only: a standard font, or the font's own name.
    Named { base14: Option<&'static str> },
}

/// A font the file uses, with the glyphs used so far.
pub(super) struct FontSlot {
    font: GlyphFont,
    pub(super) kind: SlotKind,
    /// The placeholder object of the font dictionary.
    pub(super) id: usize,
    /// Its name in page resources.
    pub(super) name: String,
    remapper: subsetter::GlyphRemapper,
    /// CID (or code) to the characters it stands for.
    unicode: BTreeMap<u16, String>,
    /// CID (or code) to its width (thousandths of an em).
    widths: BTreeMap<u16, f64>,
    licence: FontLicence,
}

/// One glyph ready to show: the font, the bytes that select it and its
/// width in ems.
pub(super) struct Shown {
    pub(super) slot: usize,
    pub(super) code: Vec<u8>,
    pub(super) width_em: f64,
    /// The characters of the cluster the glyph starts (empty for the
    /// further glyphs of a cluster).
    pub(super) text: String,
    /// The font's character map gives the code exactly `text`.
    pub(super) mapped: bool,
}

fn same_font(a: &GlyphFont, b: &GlyphFont) -> bool {
    std::sync::Arc::ptr_eq(&a.data, &b.data)
        && a.index == b.index
        && a.coords.len() == b.coords.len()
        && a.coords
            .iter()
            .zip(&b.coords)
            .all(|(x, y)| x.0 == y.0 && x.1.to_bits() == y.1.to_bits())
}

/// A face of `font` with its variation applied.
fn face_of(font: &GlyphFont) -> Option<ttf_parser::Face<'_>> {
    let mut face = ttf_parser::Face::parse(&font.data, font.index).ok()?;
    for (tag, v) in &font.coords {
        face.set_variation(ttf_parser::Tag::from_bytes(tag), *v);
    }
    Some(face)
}

/// The glyph's advance, thousandths of an em.
fn advance(font: &GlyphFont, gid: u16) -> f64 {
    face_of(font)
        .map(|face| {
            let upem = face.units_per_em().max(1) as f64;
            face.glyph_hor_advance(ttf_parser::GlyphId(gid))
                .map(|a| a as f64 * 1000.0 / upem)
                .unwrap_or(0.0)
        })
        .unwrap_or(0.0)
}

/// Subset `font` keeping the glyphs of `remapper` (their new ids are the
/// CIDs), as a font program and whether it holds CFF outlines.
fn subset_font(font: &GlyphFont, remapper: &subsetter::GlyphRemapper) -> Option<(Vec<u8>, bool)> {
    let out = if font.coords.is_empty() {
        subsetter::subset(&font.data, font.index, remapper).ok()?
    } else {
        let coords: Vec<(subsetter::Tag, f32)> = font
            .coords
            .iter()
            .map(|(t, v)| (subsetter::Tag::new(t), *v))
            .collect();
        subsetter::subset_with_variations(&font.data, font.index, &coords, remapper).ok()?
    };
    if out.get(0..4) == Some(b"OTTO") {
        // The bare CFF table is the font program.
        let raw = ttf_parser::RawFace::parse(&out, 0).ok()?;
        let cff = raw.table(ttf_parser::Tag::from_bytes(b"CFF "))?;
        Some((cff.to_vec(), true))
    } else {
        Some((out, false))
    }
}

impl Pdf {
    /// The slot of `font` written as `kind`, made on first use; `None`
    /// when the font cannot go into the file that way.
    fn font_slot(&mut self, font: &GlyphFont, kind: SlotKind) -> Option<usize> {
        if let Some(i) = self
            .fonts
            .iter()
            .position(|s| s.kind == kind && same_font(&s.font, font))
        {
            return Some(i);
        }
        let key = (
            std::sync::Arc::as_ptr(&font.data) as usize,
            font.index,
            kind.clone(),
        );
        if self.bad_fonts.contains(&key) {
            return None;
        }
        let licence = font_licence(&font.data, font.index);
        let face = face_of(font);
        let ok = match (&kind, &face) {
            (_, None) => false,
            (SlotKind::Cid, Some(_)) => {
                licence != FontLicence::Restricted
                    && subset_font(font, &subsetter::GlyphRemapper::new()).is_some()
            }
            (SlotKind::Named { .. }, Some(_)) => true,
        };
        if !ok {
            self.bad_fonts.push(key);
            return None;
        }
        let id = self.add_str(String::new());
        let name = format!("F{}", self.fonts.len());
        self.fonts.push(FontSlot {
            font: font.clone(),
            kind,
            id,
            name,
            remapper: subsetter::GlyphRemapper::new(),
            unicode: BTreeMap::new(),
            widths: BTreeMap::new(),
            licence,
        });
        Some(self.fonts.len() - 1)
    }

    /// The licence of `font`.
    pub(super) fn licence(&self, font: &GlyphFont) -> FontLicence {
        self.fonts
            .iter()
            .find(|s| same_font(&s.font, font))
            .map(|s| s.licence)
            .unwrap_or_else(|| font_licence(&font.data, font.index))
    }

    /// Glyph `gid` of `font`, standing for `text`, in an embedded font.
    pub(super) fn show_embedded(
        &mut self,
        font: &GlyphFont,
        gid: u16,
        text: &str,
    ) -> Option<Shown> {
        let slot = self.font_slot(font, SlotKind::Cid)?;
        let s = &mut self.fonts[slot];
        let cid = s.remapper.remap(gid);
        let mapped = text.is_empty()
            || s.unicode
                .entry(cid)
                .or_insert_with(|| text.to_string())
                .as_str()
                == text;
        let w = *s.widths.entry(cid).or_insert_with(|| advance(font, gid));
        Some(Shown {
            slot,
            code: cid.to_be_bytes().to_vec(),
            width_em: w / 1000.0,
            text: text.to_string(),
            mapped,
        })
    }

    /// Glyph `gid` of `font` in a font named only (a standard font when
    /// `base14` is given): only a single WinAnsi character drawn by the
    /// font's own glyph for it qualifies.
    pub(super) fn show_named(
        &mut self,
        font: &GlyphFont,
        gid: u16,
        text: &str,
        base14: Option<&'static str>,
    ) -> Option<Shown> {
        let mut chars = text.chars();
        let (Some(c), None) = (chars.next(), chars.next()) else {
            return None;
        };
        let code = win_ansi(c)?;
        let face = face_of(font)?;
        if face.glyph_index(c).map(|g| g.0) != Some(gid) {
            return None;
        }
        let slot = self.font_slot(font, SlotKind::Named { base14 })?;
        let s = &mut self.fonts[slot];
        let w = *s
            .widths
            .entry(code as u16)
            .or_insert_with(|| advance(font, gid));
        let mapped = s
            .unicode
            .entry(code as u16)
            .or_insert_with(|| text.to_string())
            .as_str()
            == text;
        Some(Shown {
            slot,
            code: vec![code],
            width_em: w / 1000.0,
            text: text.to_string(),
            mapped,
        })
    }

    /// Write every font's objects into its placeholder: the subsets are
    /// made now, once every page has named its glyphs.
    pub(super) fn finish_fonts(&mut self, text: &PdfText) {
        let slots = std::mem::take(&mut self.fonts);
        for mut s in slots {
            let body = match s.kind.clone() {
                SlotKind::Cid => self.cid_font(&mut s, text),
                SlotKind::Named { base14 } => self.named_font(&s, base14),
            };
            if let Some(obj) = self.objects.get_mut(s.id - 1) {
                *obj = body.into_bytes();
            }
            let name = ps_name(&s.font, s.kind == SlotKind::Cid);
            let list = match s.kind {
                SlotKind::Cid => &mut self.report.embedded,
                SlotKind::Named { base14: Some(b) } => {
                    if !self.report.named.iter().any(|n| n == b) {
                        self.report.named.push(b.to_string());
                    }
                    continue;
                }
                SlotKind::Named { base14: None } => &mut self.report.named,
            };
            if !list.contains(&name) {
                list.push(name);
            }
        }
    }

    fn named_font(&mut self, s: &FontSlot, base14: Option<&'static str>) -> String {
        let (first, last) = (32u16, 255u16);
        let mut widths = String::new();
        for code in first..=last {
            let w = s.widths.get(&code).copied().unwrap_or(0.0);
            let _ = write!(widths, "{} ", f(w));
        }
        let widths = widths.trim_end();
        if let Some(name) = base14 {
            return format!(
                "<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding /FirstChar {first} /LastChar {last} /Widths [{widths}] >>"
            );
        }
        let name = ps_name(&s.font, false);
        let descriptor = self.descriptor(&s.font, &name, 32, None);
        format!(
            "<< /Type /Font /Subtype /TrueType /BaseFont {} /Encoding /WinAnsiEncoding /FirstChar {first} /LastChar {last} /Widths [{widths}] /FontDescriptor {descriptor} 0 R >>",
            pdf_name(&name)
        )
    }

    fn cid_font(&mut self, s: &mut FontSlot, text: &PdfText) -> String {
        let total = face_of(&s.font).map(|f| f.number_of_glyphs()).unwrap_or(0);
        let used = s.remapper.num_gids() as f64;
        let whole = !text.subset
            || s.licence == FontLicence::NoSubsetting
            || (total > 0 && used * 100.0 / total as f64 >= text.subset_below_pct);
        if whole {
            for g in 0..total {
                s.remapper.remap(g);
            }
        }
        let Some((program, cff)) = subset_font(&s.font, &s.remapper) else {
            // Checked when the slot was made; an empty font keeps the file
            // readable should it fail now.
            return "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into();
        };
        let base = ps_name(&s.font, true);
        let name = if whole {
            base
        } else {
            format!("{}+{base}", subset_tag(&s.font, &s.remapper))
        };
        let data = flate(&program);
        let file = if cff {
            self.stream(
                &format!(
                    "/Subtype /CIDFontType0C /Filter /FlateDecode /Length1 {}",
                    program.len()
                ),
                &data,
            )
        } else {
            self.stream(
                &format!("/Filter /FlateDecode /Length1 {}", program.len()),
                &data,
            )
        };
        let descriptor = self.descriptor(&s.font, &name, 4, Some((file, cff)));
        let mut w = String::new();
        for (cid, width) in &s.widths {
            let _ = write!(w, "{cid} [{}] ", f(*width));
        }
        let descendant = self.add_str(format!(
            "<< /Type /Font /Subtype {} /BaseFont {} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {descriptor} 0 R /DW 0 /W [{}]{} >>",
            if cff { "/CIDFontType0" } else { "/CIDFontType2" },
            pdf_name(&name),
            w.trim_end(),
            if cff { "" } else { " /CIDToGIDMap /Identity" }
        ));
        let cmap = to_unicode(&s.unicode);
        let cmap_id = self.stream("/Filter /FlateDecode", &flate(cmap.as_bytes()));
        format!(
            "<< /Type /Font /Subtype /Type0 /BaseFont {} /Encoding /Identity-H /DescendantFonts [{descendant} 0 R] /ToUnicode {cmap_id} 0 R >>",
            pdf_name(&name)
        )
    }

    /// The font descriptor of `font`; `file` is its font program (CFF or
    /// TrueType) when embedded.
    fn descriptor(
        &mut self,
        font: &GlyphFont,
        name: &str,
        flags: u32,
        file: Option<(usize, bool)>,
    ) -> usize {
        let (bbox, italic, ascent, descent, cap, flags) = match face_of(font) {
            Some(face) => {
                let k = 1000.0 / face.units_per_em().max(1) as f64;
                let b = face.global_bounding_box();
                let italic = face.italic_angle() as f64;
                let mut flags = flags;
                if face.is_monospaced() {
                    flags |= 1;
                }
                if italic.abs() > 1e-6 || face.is_italic() {
                    flags |= 64;
                }
                (
                    [b.x_min, b.y_min, b.x_max, b.y_max].map(|v| v as f64 * k),
                    italic,
                    face.ascender() as f64 * k,
                    face.descender() as f64 * k,
                    face.capital_height()
                        .map(|c| c as f64 * k)
                        .unwrap_or(face.ascender() as f64 * k),
                    flags,
                )
            }
            None => (
                [0.0, -200.0, 1000.0, 900.0],
                0.0,
                800.0,
                -200.0,
                700.0,
                flags,
            ),
        };
        let file = match file {
            Some((id, true)) => format!(" /FontFile3 {id} 0 R"),
            Some((id, false)) => format!(" /FontFile2 {id} 0 R"),
            None => String::new(),
        };
        self.add_str(format!(
            "<< /Type /FontDescriptor /FontName {} /Flags {flags} /FontBBox [{} {} {} {}] /ItalicAngle {} /Ascent {} /Descent {} /CapHeight {} /StemV 80{file} >>",
            pdf_name(name),
            f(bbox[0]),
            f(bbox[1]),
            f(bbox[2]),
            f(bbox[3]),
            f(italic),
            f(ascent),
            f(descent),
            f(cap)
        ))
    }
}

/// The font's PostScript name (name ID 6), else its family without
/// spaces; letters a PDF name may hold only.
fn ps_name(font: &GlyphFont, instance: bool) -> String {
    let face = ttf_parser::Face::parse(&font.data, font.index).ok();
    let pick = |id: u16| -> Option<String> {
        let face = face.as_ref()?;
        face.names()
            .into_iter()
            .filter(|n| n.name_id == id)
            .find_map(|n| n.to_string())
    };
    let raw = pick(ttf_parser::name_id::POST_SCRIPT_NAME)
        .or_else(|| pick(ttf_parser::name_id::FAMILY))
        .unwrap_or_else(|| "Font".into());
    let mut out: String = raw
        .chars()
        .filter(|c| c.is_ascii_graphic() && !"()<>[]{}/%#".contains(*c))
        .collect();
    if out.is_empty() {
        out = "Font".into();
    }
    // Variable fonts: the instance as part of an embedded font's name.
    for (tag, v) in font.coords.iter().filter(|_| instance) {
        let t: String = tag
            .iter()
            .map(|b| *b as char)
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        let _ = write!(out, "-{t}{}", v.round() as i64);
    }
    out
}

/// Six capital letters naming a subset (from its font and glyphs).
fn subset_tag(font: &GlyphFont, remapper: &subsetter::GlyphRemapper) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    let mut feed = |b: u8| {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    };
    for b in (font.data.len() as u64).to_be_bytes() {
        feed(b);
    }
    for b in font.index.to_be_bytes() {
        feed(b);
    }
    for g in remapper.remapped_gids() {
        for b in g.to_be_bytes() {
            feed(b);
        }
    }
    (0..6)
        .map(|i| (b'A' + ((h >> (i * 5)) % 26) as u8) as char)
        .collect()
}

/// A ToUnicode CMap mapping each two-byte CID to its characters.
fn to_unicode(map: &BTreeMap<u16, String>) -> String {
    let mut out = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(&u16, &String)> = map.iter().filter(|(_, t)| !t.is_empty()).collect();
    for chunk in entries.chunks(100) {
        let _ = writeln!(out, "{} beginbfchar", chunk.len());
        for (cid, text) in chunk {
            let mut hex = String::new();
            for unit in text.encode_utf16() {
                let _ = write!(hex, "{unit:04X}");
            }
            let _ = writeln!(out, "<{cid:04X}> <{hex}>");
        }
        out.push_str("endbfchar\n");
    }
    out.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn licence_bits_decide_the_embedding() {
        assert_eq!(licence_of(0), FontLicence::Embeddable);
        assert_eq!(licence_of(0x0002), FontLicence::Restricted);
        assert_eq!(licence_of(0x0004), FontLicence::Embeddable);
        assert_eq!(licence_of(0x0008), FontLicence::Embeddable);
        // Old fonts setting several bits: the least restrictive counts.
        assert_eq!(licence_of(0x0006), FontLicence::Embeddable);
        assert_eq!(licence_of(0x0104), FontLicence::NoSubsetting);
        assert_eq!(licence_of(0x0200), FontLicence::Restricted);
        // Data that is no font is not refused.
        assert_eq!(font_licence(b"nonsense", 0), FontLicence::Embeddable);
    }

    #[test]
    fn standard_fonts_and_win_ansi_codes() {
        assert_eq!(
            base14_name("Helvetica", true, false),
            Some("Helvetica-Bold")
        );
        assert_eq!(
            base14_name("Times Roman", false, true),
            Some("Times-Italic")
        );
        assert_eq!(
            base14_name("Courier", true, true),
            Some("Courier-BoldOblique")
        );
        assert_eq!(base14_name("Arial", false, false), None);
        assert_eq!(base14_name("DejaVu Sans", false, false), None);
        assert_eq!(win_ansi('A'), Some(0x41));
        assert_eq!(win_ansi('\u{e9}'), Some(0xE9));
        assert_eq!(win_ansi('\u{20AC}'), Some(0x80));
        assert_eq!(win_ansi('\u{3B1}'), None);
        assert_eq!(win_ansi('\n'), None);
    }

    #[test]
    fn to_unicode_maps_cids_to_utf16() {
        let mut m = BTreeMap::new();
        m.insert(1u16, "A".to_string());
        m.insert(2u16, "fi".to_string());
        m.insert(3u16, String::new());
        m.insert(4u16, "\u{1F600}".to_string());
        let c = to_unicode(&m);
        assert!(c.contains("3 beginbfchar"));
        assert!(c.contains("<0001> <0041>"));
        assert!(c.contains("<0002> <00660069>"));
        assert!(c.contains("<0004> <D83DDE00>"));
        assert!(!c.contains("<0003>"));
    }
}
