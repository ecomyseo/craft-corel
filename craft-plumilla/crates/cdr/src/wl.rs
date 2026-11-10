//! `.cdr` drawings of format versions 1 and 2: the layout before RIFF, whose
//! files start with the letters `WL` (see `docs/cdr-format.md`, "Versions 1
//! and 2 (`WL`)").
//!
//! Everything here was worked out from files a user sent, whose layouts
//! were checked against a later copy of one of them saved in version 5 and
//! against what the drawings plainly show; no code of any other reader was
//! read. All values are little-endian.
//!
//! The file: `WL`, a marker byte, a zero byte, then five u32: the start of
//! the settings (28), the start and end of the object records, the start of
//! the trailer and that minus two. The settings hold the page's box at
//! byte 30 as four i16 (left, top, right, bottom) in thousandths of an inch
//! with the origin at the page's centre. The font table follows the object
//! records: a u16 count, then 84 bytes a font (u16 id, u16 0, the name of
//! the font's `.wfn` file).
//!
//! Object records are 28 bytes for groups and layers; drawn objects carry
//! the flags `0x40, 0x2x` at byte 26, their size at 28 and are followed by
//! 32 more bytes. Their box is at byte 18 (left, top, right, bottom), a
//! point at 32, a 2 x 3 matrix at 36 (six i32 in 16.16 fixed point: a, b,
//! tx, c, d, ty), the object type at 60 (0 rectangle, 1 ellipse, 2 curve,
//! 3 text), the outline at 67 (flag), 68 (width) and 74 (colour), the fill
//! at 94 (type) and 95 (colour), and the shape itself from byte 114.
//! Records run from the front of the stacking order to the back; groups
//! are not kept.
//!
//! Text, from byte 114: spacing of characters, words and lines in percent
//! (i16 at 2, 4 and 6), the font id (u16 at 8), the size in thousandths of
//! an inch (u16 at 11), the alignment (13: 1 left, 2 centre), the number of
//! characters (u16 at 14) and the characters from 26 (Windows-1252, lines
//! split by CR LF). Curves: a u16 point count, two bytes, the points as i16
//! pairs and one node-type byte a point, coded as in later versions.

use crate::parse::ParseReport;
use crate::{Error, Result};
use tracedraw_core::document::{
    Layer, Page, ParagraphStyle, Shape, ShapeKind, TextAlign, TextSpan,
};
use tracedraw_core::geometry::{Affine, Point, Rect, Size};
use tracedraw_core::{Color, Fill, Stroke};

/// Thousandths of an inch to millimetres.
const UNIT_MM: f64 = 25.4 / 1000.0;
/// Most object records read from one file.
const MAX_OBJECTS: usize = 200_000;

/// The file starts with the letters `WL`.
pub fn is_wl(bytes: &[u8]) -> bool {
    bytes.get(0..2) == Some(b"WL".as_slice())
}

fn u16_at(d: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*d.get(at)?, *d.get(at + 1)?]))
}

fn i16_at(d: &[u8], at: usize) -> Option<i16> {
    u16_at(d, at).map(|v| v as i16)
}

fn u32_at(d: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *d.get(at)?,
        *d.get(at + 1)?,
        *d.get(at + 2)?,
        *d.get(at + 3)?,
    ]))
}

fn i32_at(d: &[u8], at: usize) -> Option<i32> {
    u32_at(d, at).map(|v| v as i32)
}

/// One drawn object's record: where it starts and how long its body is.
struct Record {
    at: usize,
    size: usize,
}

/// Read a version 1 or 2 drawing.
pub fn open_bytes(bytes: &[u8], title: &str) -> Result<(tracedraw_core::Document, ParseReport)> {
    if !is_wl(bytes) {
        return Err(Error::NotCdr);
    }
    let d = bytes;
    let start = u32_at(d, 8).ok_or(Error::Truncated(8))? as usize;
    let end = u32_at(d, 12).ok_or(Error::Truncated(12))? as usize;
    if start < 28 || end > d.len() || start > end {
        return Err(Error::NotCdr);
    }
    let mut report = ParseReport::default();
    report.warn(format!(
        "version 1 or 2 drawing (`WL`, marker '{}'): read from layouts seen in a few files",
        d.get(2)
            .map(|c| (*c as char).to_string())
            .unwrap_or_default()
    ));
    // The page: four i16 at byte 30, in thousandths of an inch around the
    // page's centre; letter portrait when they do not make sense.
    let page_box = (|| {
        let (x0, y1, x1, y0) = (
            i16_at(d, 30)?,
            i16_at(d, 32)?,
            i16_at(d, 34)?,
            i16_at(d, 36)?,
        );
        let (w, h) = (x1 as f64 - x0 as f64, y1 as f64 - y0 as f64);
        (w > 100.0 && h > 100.0).then_some((w * UNIT_MM, h * UNIT_MM))
    })()
    .unwrap_or((215.9, 279.4));
    let size = Size::new(page_box.0, page_box.1);
    let fonts = font_table(d, end);

    let mut doc = tracedraw_core::Document::new(title, size);
    doc.pages.clear();
    let page_id = doc.ids_mut().page();
    let layer_id = doc.ids_mut().layer();
    let mut layer = Layer::new(layer_id, "Layer 1");
    let centre = Affine::translate((size.width / 2.0, size.height / 2.0));

    let mut at = start;
    let mut count = 0usize;
    while at + 30 <= end && count < MAX_OBJECTS {
        count += 1;
        let Some(flags) = u16_at(d, at + 26) else {
            break;
        };
        if flags & 0xff != 0x40 {
            // A group or layer: 28 bytes, nothing drawn.
            at += 28;
            continue;
        }
        let Some(body) = u16_at(d, at + 28).map(|s| s as usize) else {
            break;
        };
        if body + 32 < 118 {
            report.warn(format!(
                "object record at {at} shorter than its fields; stopped"
            ));
            break;
        }
        let rec = Record { at, size: body };
        at += body + 32;
        match read_object(d, &rec, &fonts, centre, &mut doc, &mut report) {
            Some(shape) => {
                layer.shapes.push(shape);
                report.shapes += 1;
            }
            None => report.skipped_objects += 1,
        }
    }
    if at != end {
        report.warn(format!("object records end at {at}, the header says {end}"));
    }
    // Records run from the front of the stacking order to the back.
    layer.shapes.reverse();
    doc.pages.push(Page {
        id: page_id,
        name: "Page 1".into(),
        size,
        layers: vec![layer],
        guides: Vec::new(),
        background: None,
    });
    report.pages = 1;
    report.layers = 1;
    report.version = None;
    doc.title = title.to_string();
    Ok((doc, report))
}

/// Fonts by id: the family of each `.wfn` file the drawing names.
fn font_table(d: &[u8], at: usize) -> Vec<(u16, String)> {
    let Some(n) = u16_at(d, at) else {
        return Vec::new();
    };
    (0..n.min(256) as usize)
        .filter_map(|k| {
            let e = at + 2 + 84 * k;
            let id = u16_at(d, e)?;
            let raw = d.get(e + 4..e + 18)?;
            let name: String = raw
                .iter()
                .take_while(|c| **c != 0)
                .map(|c| *c as char)
                .collect();
            Some((id, family_of(&name)))
        })
        .collect()
}

/// A current family for one of the fonts shipped with versions 1 and 2, by
/// the name of its file; others keep the file's name.
pub(crate) fn family_of(wfn: &str) -> String {
    let stem = wfn
        .trim()
        .to_ascii_lowercase()
        .trim_end_matches(".wfn")
        .to_string();
    let known = [
        ("swz", "Arial"),
        ("switzerland", "Arial"),
        ("switzerlandnarrow", "Arial Narrow"),
        ("switzerlandblack", "Arial Black"),
        ("swzn", "Arial Narrow"),
        ("swzblack", "Arial Black"),
        ("avalon", "URW Gothic"),
        ("toronto", "Times New Roman"),
        ("brooklyn", "Bookman Old Style"),
        ("cuprtino", "Garamond"),
        ("bodnoff", "Bodoni MT"),
        ("france", "Optima"),
        ("banff", "Brush Script MT"),
        ("bangkok", "Cooper Black"),
        ("monospac", "Courier New"),
    ];
    known
        .iter()
        .find(|(k, _)| *k == stem)
        .map(|(_, f)| f.to_string())
        .unwrap_or(stem)
}

/// A colour slot: a model byte, then four bytes. Model 1: cyan, magenta,
/// yellow and black in percent. Model 0: a u16 index and a u16 tint, index
/// 0 black at the tint (in percent), index 1 white.
fn colour(d: &[u8], model_at: usize, report: &mut ParseReport) -> Option<Color> {
    let model = *d.get(model_at)?;
    let v = d.get(model_at + 1..model_at + 5)?;
    let pct = |b: u8| (b as f32).min(100.0);
    match model {
        1 => Some(Color::cmyk_pct(pct(v[0]), pct(v[1]), pct(v[2]), pct(v[3]))),
        0 => {
            let index = u16::from_le_bytes([v[0], v[1]]);
            let tint = u16::from_le_bytes([v[2], v[3]]).min(100) as f32;
            match index {
                0 => Some(Color::cmyk_pct(0.0, 0.0, 0.0, tint)),
                1 => Some(Color::cmyk_pct(0.0, 0.0, 0.0, 0.0)),
                other => {
                    report.warn(format!(
                        "palette colour {other} of a version 1 or 2 drawing read as black"
                    ));
                    Some(Color::cmyk_pct(0.0, 0.0, 0.0, tint))
                }
            }
        }
        other => {
            report.warn(format!(
                "colour model {other} of a version 1 or 2 drawing not read"
            ));
            None
        }
    }
}

fn read_object(
    d: &[u8],
    rec: &Record,
    fonts: &[(u16, String)],
    centre: Affine,
    doc: &mut tracedraw_core::Document,
    report: &mut ParseReport,
) -> Option<Shape> {
    let r = rec.at;
    let (ox, oy) = (i16_at(d, r + 32)? as f64, i16_at(d, r + 34)? as f64);
    let m: Vec<f64> = (0..6)
        .map(|k| i32_at(d, r + 36 + 4 * k).map(|v| v as f64 / 65536.0))
        .collect::<Option<Vec<_>>>()?;
    let (a, b, tx, c, dd, ty) = (m[0], m[1], m[2], m[3], m[4], m[5]);
    // x' = a x + b y + tx + point x, y' = c x + d y + ty + point y, in
    // thousandths of an inch; the shape's own coordinates become mm.
    let transform = centre * Affine::new([a, c, b, dd, (tx + ox) * UNIT_MM, (ty + oy) * UNIT_MM]);
    let kind_code = *d.get(r + 60)?;
    let g = r + 114;
    let kind = match kind_code {
        0 | 1 => {
            let (w, h) = (
                i16_at(d, g)? as f64 * UNIT_MM,
                i16_at(d, g + 2)? as f64 * UNIT_MM,
            );
            let rect = Rect::new(w.min(0.0), h.min(0.0), w.max(0.0), h.max(0.0));
            if kind_code == 0 {
                let radius = i16_at(d, g + 6).map(|v| v as f64 * UNIT_MM).unwrap_or(0.0);
                let radius = if radius > 0.0 && radius <= rect.width().min(rect.height()) / 2.0 {
                    radius
                } else {
                    0.0
                };
                ShapeKind::Rect {
                    rect,
                    radius,
                    corners: None,
                }
            } else {
                let (s, e) = (i16_at(d, g + 4)?, i16_at(d, g + 6)?);
                if s != e {
                    report.warn("arcs and pies of version 1 or 2 drawings drawn as whole ellipses");
                }
                ShapeKind::Ellipse { rect, arc: None }
            }
        }
        2 => {
            let n = u16_at(d, g)? as usize;
            let pts_at = g + 4;
            let types = d.get(pts_at + 4 * n..pts_at + 5 * n)?;
            let pts: Vec<Point> = (0..n)
                .map(|k| {
                    Some(Point::new(
                        i16_at(d, pts_at + 4 * k)? as f64 * UNIT_MM,
                        i16_at(d, pts_at + 4 * k + 2)? as f64 * UNIT_MM,
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            let (path, closed) = crate::parse::build_path(&pts, types);
            if path.elements().is_empty() {
                return None;
            }
            ShapeKind::Path { path, closed }
        }
        3 => {
            let font_id = u16_at(d, g + 8)?;
            let size_in = u16_at(d, g + 11)? as f64 / 1000.0;
            let n = u16_at(d, g + 14)? as usize;
            let raw = d.get(g + 26..(g + 26 + n).min(r + rec.size + 32))?;
            let raw: Vec<u8> = raw.iter().copied().take_while(|c| *c != 0).collect();
            let text = crate::text::decode_cp1252(&raw)
                .replace("\r\n", "\n")
                .replace('\r', "\n");
            if text.trim().is_empty() {
                return None;
            }
            let family = fonts
                .iter()
                .find(|(id, _)| *id == font_id)
                .map(|(_, f)| f.clone())
                .unwrap_or_else(|| "Arial".into());
            let size_pt = if size_in > 0.0 { size_in * 72.0 } else { 24.0 };
            // Spacing in percent: characters at +2, words at +4, lines at
            // +6; the alignment code at +13.
            let pct = |at: usize, lo: f64, hi: f64, default: f64| {
                i16_at(d, at)
                    .map(|v| v as f64)
                    .filter(|v| (lo..=hi).contains(v))
                    .unwrap_or(default)
            };
            let para = ParagraphStyle {
                char_spacing_pct: pct(g + 2, -100.0, 2000.0, 0.0),
                word_spacing_pct: pct(g + 4, 0.0, 2000.0, 100.0),
                leading_pct: pct(g + 6, 0.0, 2000.0, 100.0),
                ..Default::default()
            };
            let align = match d.get(g + 13).copied() {
                Some(2) => TextAlign::Center,
                Some(3) => TextAlign::Right,
                _ => TextAlign::Left,
            };
            ShapeKind::Text {
                spans: vec![TextSpan::new(text, family, size_pt)],
                origin: Point::ZERO,
                frame: None,
                align,
                para,
                on_path: None,
            }
        }
        other => {
            report.warn(format!(
                "object type {other} of a version 1 or 2 drawing not read"
            ));
            return None;
        }
    };
    let mut shape = Shape::new(doc.ids_mut().shape(), kind);
    shape.transform = transform;
    shape.fill = match d.get(r + 94)? {
        0 => Fill::None,
        _ => colour(d, r + 95, report)
            .map(Fill::Solid)
            .unwrap_or(Fill::None),
    };
    shape.stroke = if *d.get(r + 67)? != 0 {
        let width = u16_at(d, r + 68)? as f64 * UNIT_MM;
        colour(d, r + 74, report).map(|c| {
            if width > 0.0 {
                Stroke::new(c, width)
            } else {
                Stroke::hairline(c)
            }
        })
    } else {
        None
    };
    Some(shape)
}

#[cfg(test)]
mod tests {
    use super::*;

    const START: usize = 64;

    fn put16(v: &mut [u8], at: usize, x: i16) {
        v[at..at + 2].copy_from_slice(&x.to_le_bytes());
    }

    fn put32(v: &mut [u8], at: usize, x: i32) {
        v[at..at + 4].copy_from_slice(&x.to_le_bytes());
    }

    /// A drawn object's record with an identity matrix moved to `(x, y)`
    /// thousandths of an inch, a CMYK fill and the shape bytes `geom`.
    fn object(kind: u8, body: usize, x: i16, y: i16, fill: [u8; 4], geom: &[u8]) -> Vec<u8> {
        let mut r = vec![0u8; body + 32];
        r[26] = 0x40;
        r[27] = 0x20;
        put16(&mut r, 28, body as i16);
        put16(&mut r, 32, x);
        put16(&mut r, 34, y);
        put32(&mut r, 36, 65536);
        put32(&mut r, 52, 65536);
        r[60] = kind;
        r[94] = 1;
        r[95] = 1;
        r[96..100].copy_from_slice(&fill);
        r[114..114 + geom.len()].copy_from_slice(geom);
        r
    }

    fn group() -> Vec<u8> {
        let mut r = vec![0u8; 28];
        r[0] = 0x18;
        r[27] = 0x01;
        r
    }

    /// A whole file: header, a letter-size page, the records and a font
    /// table naming `swz.wfn` as font 3.
    fn file(records: &[Vec<u8>]) -> Vec<u8> {
        let mut v = vec![0u8; START];
        v[0..2].copy_from_slice(b"WL");
        v[2] = b'e';
        for r in records {
            v.extend_from_slice(r);
        }
        let end = v.len();
        let mut fonts = vec![0u8; 2 + 84];
        fonts[0] = 1;
        fonts[2] = 3;
        fonts[6..13].copy_from_slice(b"swz.wfn");
        v.extend_from_slice(&fonts);
        let trailer = v.len();
        put32(&mut v, 4, 28);
        put32(&mut v, 8, START as i32);
        put32(&mut v, 12, end as i32);
        put32(&mut v, 16, trailer as i32);
        put32(&mut v, 20, trailer as i32 - 2);
        put16(&mut v, 30, -4250);
        put16(&mut v, 32, 5500);
        put16(&mut v, 34, 4250);
        put16(&mut v, 36, -5500);
        v
    }

    fn rect_geom(w: i16, h: i16) -> Vec<u8> {
        let mut g = vec![0u8; 8];
        put16(&mut g, 0, w);
        put16(&mut g, 2, h);
        g
    }

    #[test]
    fn reads_page_shapes_and_stacking_order() {
        let mut outlined = object(1, 96, -1000, 0, [0, 100, 100, 0], &rect_geom(2000, 1000));
        outlined[67] = 1;
        put16(&mut outlined, 68, 100);
        outlined[74] = 1;
        outlined[78] = 100;
        let bytes = file(&[
            group(),
            outlined,
            object(0, 96, 0, 0, [100, 0, 0, 0], &rect_geom(1000, 500)),
        ]);
        let (doc, report) = open_bytes(&bytes, "t").expect("opens");
        let page = &doc.pages[0];
        assert!((page.size.width - 215.9).abs() < 1e-6);
        assert!((page.size.height - 279.4).abs() < 1e-6);
        let shapes = &page.layers[0].shapes;
        assert_eq!(shapes.len(), 2);
        assert_eq!(report.shapes, 2);
        // The first record is the front-most object, so it is drawn last.
        assert!(matches!(shapes[0].kind, ShapeKind::Rect { .. }));
        assert!(matches!(shapes[1].kind, ShapeKind::Ellipse { .. }));
        assert_eq!(
            shapes[0].fill,
            Fill::Solid(Color::cmyk_pct(100.0, 0.0, 0.0, 0.0))
        );
        assert_eq!(
            shapes[1].fill,
            Fill::Solid(Color::cmyk_pct(0.0, 100.0, 100.0, 0.0))
        );
        let stroke = shapes[1].stroke.as_ref().expect("outline");
        assert!((stroke.width - 2.54).abs() < 1e-9);
        assert!(shapes[0].stroke.is_none());
        // The ellipse's centre sits 1 inch left of the page's centre.
        let c = shapes[1].transform * Point::new(0.0, 0.0);
        assert!((c.x - (215.9 / 2.0 - 25.4)).abs() < 1e-6);
        assert!((c.y - 279.4 / 2.0).abs() < 1e-6);
        assert!(!report.warnings.iter().any(|w| w.contains("end at")));
    }

    #[test]
    fn reads_text_with_font_size_and_alignment() {
        let chars = b"Hi\r\nthere";
        let mut g = vec![0u8; 26 + chars.len()];
        put16(&mut g, 4, 100);
        put16(&mut g, 6, 150);
        put16(&mut g, 8, 3);
        g[10] = 1;
        put16(&mut g, 11, 500);
        g[13] = 2;
        put16(&mut g, 14, chars.len() as i16);
        g[26..].copy_from_slice(chars);
        let bytes = file(&[object(3, 160, 0, 0, [0, 0, 0, 100], &g)]);
        let (doc, _) = open_bytes(&bytes, "t").expect("opens");
        let shape = &doc.pages[0].layers[0].shapes[0];
        let ShapeKind::Text {
            spans, align, para, ..
        } = &shape.kind
        else {
            panic!("not text: {:?}", shape.kind);
        };
        assert_eq!(spans[0].text, "Hi\nthere");
        assert_eq!(spans[0].font_family, "Arial");
        assert!((spans[0].size_pt - 36.0).abs() < 1e-9);
        assert_eq!(*align, TextAlign::Center);
        assert!((para.leading_pct - 150.0).abs() < 1e-9);
    }

    #[test]
    fn reads_curves() {
        // A closed triangle: move, line, line with the closing bit.
        let mut g = vec![0u8; 4 + 5 * 3];
        put16(&mut g, 0, 3);
        for (k, (x, y)) in [(0i16, 0i16), (1000, 0), (0, 1000)].iter().enumerate() {
            put16(&mut g, 4 + 4 * k, *x);
            put16(&mut g, 6 + 4 * k, *y);
        }
        g[16] = 0x00;
        g[17] = 0x40;
        g[18] = 0x48;
        let bytes = file(&[object(2, 112, 0, 0, [0, 0, 0, 100], &g)]);
        let (doc, _) = open_bytes(&bytes, "t").expect("opens");
        let ShapeKind::Path { closed, .. } = &doc.pages[0].layers[0].shapes[0].kind else {
            panic!("not a path");
        };
        assert!(*closed);
    }

    #[test]
    fn truncated_and_damaged_files_never_panic() {
        let bytes = file(&[
            object(0, 96, 0, 0, [100, 0, 0, 0], &rect_geom(1000, 500)),
            object(1, 96, 0, 0, [0, 0, 0, 100], &rect_geom(500, 500)),
        ]);
        for len in 0..bytes.len() {
            let _ = open_bytes(&bytes[..len], "t");
        }
        for at in 0..bytes.len() {
            for v in [0u8, 0x40, 0xff] {
                let mut b = bytes.clone();
                b[at] = v;
                let _ = open_bytes(&b, "t");
            }
        }
        assert!(open_bytes(b"RIFF", "t").is_err());
    }

    #[test]
    fn maps_shipped_font_names() {
        assert_eq!(family_of("TORONTO.WFN"), "Times New Roman");
        assert_eq!(family_of("odd.wfn"), "odd");
    }
}
