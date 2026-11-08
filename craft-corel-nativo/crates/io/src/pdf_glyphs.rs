//! Glyph outlines from the fonts a PDF embeds, for importing its text as
//! curves (#386): TrueType programs (`FontFile2`), OpenType ones
//! (`FontFile3` of subtype `OpenType`) and bare CFF ones (`FontFile3` of
//! subtype `Type1C` or `CIDFontType0C`). A character code becomes a glyph
//! as the PDF reference describes for each font type: through the
//! `CIDToGIDMap` (or the identity) for two-byte TrueType fonts, through the
//! CFF charset for CID-keyed CFF fonts, through the font's own `cmap`
//! (symbolic (3,0) at 0xF000 + code, Mac Roman (1,0), else Unicode by the
//! PDF encoding) for simple TrueType fonts, and through the CFF built-in
//! encoding (else the glyph name of the character) for simple CFF fonts.

use std::collections::HashMap;
use tracedraw_core::geometry::{Affine, BezPath};

/// An embedded font program and how codes reach its glyphs.
pub(crate) struct GlyphSource {
    data: Vec<u8>,
    /// A bare CFF program (else an sfnt: TrueType or OpenType).
    cff: bool,
    /// Two-byte codes (a Type0 font): the code is a CID.
    two_byte: bool,
    /// Type0 TrueType: CID to glyph index (`None`: the identity).
    cid_to_gid: Option<Vec<u16>>,
    /// CID-keyed CFF: CID to glyph index, from its charset.
    cff_cids: HashMap<u16, u16>,
    /// Simple fonts: the character each code stands for.
    encoding: Option<[Option<char>; 256]>,
}

#[derive(Default)]
struct Builder {
    path: BezPath,
}

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to((f64::from(x), f64::from(y)));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to((f64::from(x), f64::from(y)));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.path
            .quad_to((f64::from(x1), f64::from(y1)), (f64::from(x), f64::from(y)));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.path.curve_to(
            (f64::from(x1), f64::from(y1)),
            (f64::from(x2), f64::from(y2)),
            (f64::from(x), f64::from(y)),
        );
    }
    fn close(&mut self) {
        self.path.close_path();
    }
}

impl GlyphSource {
    /// A source for `data`, or `None` when no outlines can be read from it.
    pub(crate) fn new(
        data: Vec<u8>,
        cff: bool,
        two_byte: bool,
        cid_to_gid: Option<Vec<u16>>,
        encoding: Option<[Option<char>; 256]>,
    ) -> Option<Self> {
        let mut cff_cids = HashMap::new();
        if cff {
            let t = ttf_parser::cff::Table::parse(&data)?;
            if two_byte {
                for g in 0..t.number_of_glyphs() {
                    if let Some(cid) = t.glyph_cid(ttf_parser::GlyphId(g)) {
                        cff_cids.insert(cid, g);
                    }
                }
            }
        } else {
            ttf_parser::Face::parse(&data, 0).ok()?;
        }
        Some(GlyphSource {
            data,
            cff,
            two_byte,
            cid_to_gid,
            cff_cids,
            encoding,
        })
    }

    /// The outline of the glyph for `code`, one unit to the em (the font
    /// size scales it), its origin at the glyph's origin; `None` for a
    /// glyph with no outline or none to be found.
    pub(crate) fn outline(&self, code: u32) -> Option<BezPath> {
        let mut b = Builder::default();
        if self.cff {
            let t = ttf_parser::cff::Table::parse(&self.data)?;
            let gid = if self.two_byte {
                let cid = u16::try_from(code).ok()?;
                ttf_parser::GlyphId(self.cff_cids.get(&cid).copied().unwrap_or(cid))
            } else {
                let by_code = u8::try_from(code).ok().and_then(|c| t.glyph_index(c));
                match by_code {
                    Some(g) if g.0 != 0 => g,
                    _ => {
                        let c = self.char_of(code)?;
                        t.glyph_index_by_name(glyph_name(c)?)?
                    }
                }
            };
            t.outline(gid, &mut b).ok()?;
            let m = t.matrix();
            let k = Affine::new([
                f64::from(m.sx),
                f64::from(m.ky),
                f64::from(m.kx),
                f64::from(m.sy),
                f64::from(m.tx),
                f64::from(m.ty),
            ]);
            return Some(k * b.path);
        }
        let face = ttf_parser::Face::parse(&self.data, 0).ok()?;
        let gid = if self.two_byte {
            let cid = usize::try_from(code).ok()?;
            let g = match &self.cid_to_gid {
                Some(map) => *map.get(cid)?,
                None => u16::try_from(cid).ok()?,
            };
            ttf_parser::GlyphId(g)
        } else {
            self.simple_glyph(&face, code)?
        };
        face.outline_glyph(gid, &mut b)?;
        let upem = f64::from(face.units_per_em().max(1));
        Some(Affine::scale(1.0 / upem) * b.path)
    }

    fn char_of(&self, code: u32) -> Option<char> {
        self.encoding
            .as_ref()
            .and_then(|t| t.get(code as usize).copied().flatten())
    }

    /// A simple TrueType font's glyph for `code`.
    fn simple_glyph(&self, face: &ttf_parser::Face, code: u32) -> Option<ttf_parser::GlyphId> {
        use ttf_parser::PlatformId;
        let cmap = face.tables().cmap?;
        for st in cmap.subtables {
            let g = match (st.platform_id, st.encoding_id) {
                (PlatformId::Windows, 0) => st
                    .glyph_index(0xF000 + code)
                    .or_else(|| st.glyph_index(code)),
                (PlatformId::Macintosh, 0) => st.glyph_index(code),
                _ => None,
            };
            if let Some(g) = g.filter(|g| g.0 != 0) {
                return Some(g);
            }
        }
        let c = self.char_of(code)?;
        face.glyph_index(c).filter(|g| g.0 != 0)
    }
}

/// The standard glyph name of a character, for fonts found by name: the
/// ASCII letters and digits by their own names, a few common signs, else
/// `uniXXXX`-style names are not tried.
fn glyph_name(c: char) -> Option<&'static str> {
    const NAMES: [(char, &str); 30] = [
        (' ', "space"),
        ('.', "period"),
        (',', "comma"),
        (':', "colon"),
        (';', "semicolon"),
        ('-', "hyphen"),
        ('!', "exclam"),
        ('?', "question"),
        ('(', "parenleft"),
        (')', "parenright"),
        ('\'', "quotesingle"),
        ('"', "quotedbl"),
        ('/', "slash"),
        ('&', "ampersand"),
        ('@', "at"),
        ('%', "percent"),
        ('+', "plus"),
        ('=', "equal"),
        ('0', "zero"),
        ('1', "one"),
        ('2', "two"),
        ('3', "three"),
        ('4', "four"),
        ('5', "five"),
        ('6', "six"),
        ('7', "seven"),
        ('8', "eight"),
        ('9', "nine"),
        ('_', "underscore"),
        ('#', "numbersign"),
    ];
    if let Some((_, n)) = NAMES.iter().find(|(k, _)| *k == c) {
        return Some(n);
    }
    const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let i = LETTERS.find(c)?;
    Some(&LETTERS[i..i + 1])
}
