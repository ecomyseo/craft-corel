//! CMYK bitmap files for print work (#1763): a baseline JPEG of four
//! components and a TIFF separated (CMYK) image, both written from ink
//! coverages (0 = no ink, 255 = full ink), with the resolution and, when
//! given, the ICC profile the inks are in.
//!
//! Written from public specifications only:
//! - JPEG: ITU-T T.81 | ISO/IEC 10918-1 (baseline sequential DCT,
//!   Huffman coding; the example quantization and Huffman tables of its
//!   Annex K), with the Adobe `APP14` marker of Adobe Technical Note 5116
//!   (transform 0: the components are stored as they are, no colour
//!   transform). The components are stored inverted (255 = no ink), the
//!   convention readers of Adobe CMYK JPEG files expect.
//! - The resolution of a JPEG in an Exif `APP1` segment (CIPA DC-008,
//!   Exif: a TIFF header and IFD0 with XResolution, YResolution and
//!   ResolutionUnit).
//! - TIFF: the TIFF 6.0 specification, section 16 (CMYK images:
//!   PhotometricInterpretation 5, InkSet 1), uncompressed, chunky.
//! - ICC profiles in JPEG (`APP2` "ICC_PROFILE" segments) and TIFF (tag
//!   34675): ICC.1, annex B.

/// The usual quality scale (1 to 100) as a factor on the example
/// quantization table: 1 at 50, smaller above, larger below; 100 keeps
/// every step at 1.
fn quality_factor(quality: u8) -> Option<f64> {
    let q = f64::from(quality.clamp(1, 100));
    (quality < 100).then(|| 2f64.powf((50.0 - q) / 20.0))
}

/// T.81 Annex K, table K.1, in natural (row by row) order.
const BASE_QUANT: [u16; 64] = [
    16, 11, 10, 16, 24, 40, 51, 61, //
    12, 12, 14, 19, 26, 58, 60, 55, //
    14, 13, 16, 24, 40, 57, 69, 56, //
    14, 17, 22, 29, 51, 87, 80, 62, //
    18, 22, 37, 56, 68, 109, 103, 77, //
    24, 35, 55, 64, 81, 104, 113, 92, //
    49, 64, 78, 87, 103, 121, 120, 101, //
    72, 92, 95, 98, 112, 100, 103, 99,
];

/// T.81 Annex K, table K.3: DC code lengths (number of codes of each
/// length 1 to 16) and the categories they code.
const DC_BITS: [u8; 16] = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0];
const DC_VALS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

/// T.81 Annex K, table K.5: AC code lengths and run/size symbols.
const AC_BITS: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d];
const AC_VALS: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
    0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xa1, 0x08, 0x23, 0x42, 0xb1, 0xc1, 0x15, 0x52, 0xd1, 0xf0,
    0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0a, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2a, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
    0x4a, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6a, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8a, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9a, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xba, 0xc2, 0xc3, 0xc4, 0xc5,
    0xc6, 0xc7, 0xc8, 0xc9, 0xca, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7, 0xd8, 0xd9, 0xda, 0xe1, 0xe2,
    0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xea, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8,
    0xf9, 0xfa,
];

/// The zig-zag sequence of T.81 figure A.6: the natural index of each
/// coefficient in the order they are coded, anti-diagonal by
/// anti-diagonal, alternating direction.
fn zigzag() -> [usize; 64] {
    let mut out = [0usize; 64];
    let mut k = 0;
    for s in 0..15usize {
        let lo = s.saturating_sub(7);
        let hi = s.min(7);
        let rows: Vec<usize> = if s % 2 == 0 {
            (lo..=hi).rev().collect()
        } else {
            (lo..=hi).collect()
        };
        for row in rows {
            if let Some(slot) = out.get_mut(k) {
                *slot = row * 8 + (s - row);
            }
            k += 1;
        }
    }
    out
}

/// A Huffman code table for encoding: code and length by symbol (T.81
/// Annex C: codes of each length in order, counting up).
struct Codes {
    code: [u16; 256],
    len: [u8; 256],
}

fn codes(bits: &[u8; 16], vals: &[u8]) -> Codes {
    let mut c = Codes {
        code: [0; 256],
        len: [0; 256],
    };
    let mut code: u16 = 0;
    let mut at = 0usize;
    for (i, n) in bits.iter().enumerate() {
        for _ in 0..*n {
            if let Some(v) = vals.get(at) {
                c.code[*v as usize] = code;
                c.len[*v as usize] = i as u8 + 1;
            }
            code = code.wrapping_add(1);
            at += 1;
        }
        code <<= 1;
    }
    c
}

/// Entropy-coded bits, 0xFF bytes followed by a stuffed zero (T.81 F.1.2.3).
struct Bits {
    out: Vec<u8>,
    acc: u32,
    n: u32,
}

impl Bits {
    fn put(&mut self, value: u32, len: u32) {
        if len == 0 {
            return;
        }
        self.acc = (self.acc << len) | (value & ((1 << len) - 1));
        self.n += len;
        while self.n >= 8 {
            self.n -= 8;
            let b = (self.acc >> self.n) as u8;
            self.out.push(b);
            if b == 0xFF {
                self.out.push(0);
            }
        }
        self.acc &= (1 << self.n) - 1;
    }

    /// Pad the last byte with one bits.
    fn finish(mut self) -> Vec<u8> {
        if self.n > 0 {
            let pad = 8 - self.n;
            self.put((1 << pad) - 1, pad);
        }
        self.out
    }
}

/// The number of bits of the magnitude of `v` (its category, T.81 F.1.2.1).
fn category(v: i32) -> u32 {
    32 - v.unsigned_abs().leading_zeros()
}

/// The additional bits of `v` in its category: `v` when positive, else
/// `v - 1` (its low bits).
fn extra(v: i32) -> u32 {
    if v < 0 {
        (v - 1) as u32
    } else {
        v as u32
    }
}

/// The forward DCT of T.81 A.3.3 on an 8 x 8 block of level-shifted
/// samples, `cos[u][x]` being `C(u) / 2 * cos((2x + 1) u pi / 16)`.
fn fdct(block: &[f32; 64], cos: &[[f32; 8]; 8]) -> [f32; 64] {
    let mut rows = [0f32; 64];
    for y in 0..8 {
        for u in 0..8 {
            let mut s = 0.0;
            for x in 0..8 {
                s += cos[u][x] * block[y * 8 + x];
            }
            rows[y * 8 + u] = s;
        }
    }
    let mut out = [0f32; 64];
    for u in 0..8 {
        for v in 0..8 {
            let mut s = 0.0;
            for y in 0..8 {
                s += cos[v][y] * rows[y * 8 + u];
            }
            out[v * 8 + u] = s;
        }
    }
    out
}

fn segment(out: &mut Vec<u8>, marker: u8, body: &[u8]) {
    out.extend_from_slice(&[0xFF, marker]);
    out.extend_from_slice(&((body.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(body);
}

/// The ICC profile in `APP2` segments: "ICC_PROFILE", a zero, the
/// segment's number from 1 and the count, then up to 65519 bytes each.
fn icc_segments(out: &mut Vec<u8>, icc: &[u8]) {
    const CHUNK: usize = 65_519;
    let chunks: Vec<&[u8]> = icc.chunks(CHUNK).collect();
    if icc.is_empty() || chunks.len() > 255 {
        return;
    }
    for (i, c) in chunks.iter().enumerate() {
        let mut body = b"ICC_PROFILE\0".to_vec();
        body.push(i as u8 + 1);
        body.push(chunks.len() as u8);
        body.extend_from_slice(c);
        segment(out, 0xE2, &body);
    }
}

/// `width` x `height` pixels of ink coverages (`ink`: cyan, magenta,
/// yellow and black of each pixel, row by row, 255 = full ink) as a
/// baseline JPEG of four components at `quality` (1 to 100), recording
/// `dpi` and carrying `icc`. `None` when the size does not fit a JPEG or
/// `ink` is not that many pixels.
pub fn cmyk_jpeg(
    width: u32,
    height: u32,
    ink: &[u8],
    quality: u8,
    dpi: f64,
    icc: Option<&[u8]>,
) -> Option<Vec<u8>> {
    let (w, h) = (width as usize, height as usize);
    if width == 0 || height == 0 || width > 65_535 || height > 65_535 || ink.len() != w * h * 4 {
        return None;
    }
    let zz = zigzag();
    let factor = quality_factor(quality);
    let quant: [u16; 64] = std::array::from_fn(|i| match factor {
        Some(f) => (f64::from(BASE_QUANT[i]) * f).round().clamp(1.0, 255.0) as u16,
        None => 1,
    });
    let dc = codes(&DC_BITS, &DC_VALS);
    let ac = codes(&AC_BITS, &AC_VALS);

    let mut out = vec![0xFF, 0xD8];
    // Adobe APP14: version 100, no flags, transform 0.
    let mut adobe = b"Adobe".to_vec();
    adobe.extend_from_slice(&100u16.to_be_bytes());
    adobe.extend_from_slice(&[0, 0, 0, 0, 0]);
    segment(&mut out, 0xEE, &adobe);
    if let Some(res) = resolution_ifd(dpi) {
        let mut exif = b"Exif\0\0".to_vec();
        exif.extend_from_slice(&res);
        segment(&mut out, 0xE1, &exif);
    }
    if let Some(icc) = icc {
        icc_segments(&mut out, icc);
    }
    // One 8-bit table for every component, in zig-zag order.
    let mut dqt = vec![0u8];
    dqt.extend(zz.iter().map(|&n| quant[n] as u8));
    segment(&mut out, 0xDB, &dqt);
    // Baseline frame: four components, no subsampling, table 0.
    let mut sof = vec![8];
    sof.extend_from_slice(&(height as u16).to_be_bytes());
    sof.extend_from_slice(&(width as u16).to_be_bytes());
    sof.push(4);
    for id in 1..=4u8 {
        sof.extend_from_slice(&[id, 0x11, 0]);
    }
    segment(&mut out, 0xC0, &sof);
    let mut dht = vec![0x00];
    dht.extend_from_slice(&DC_BITS);
    dht.extend_from_slice(&DC_VALS);
    dht.push(0x10);
    dht.extend_from_slice(&AC_BITS);
    dht.extend_from_slice(&AC_VALS);
    segment(&mut out, 0xC4, &dht);
    let mut sos = vec![4];
    for id in 1..=4u8 {
        sos.extend_from_slice(&[id, 0x00]);
    }
    sos.extend_from_slice(&[0, 63, 0]);
    segment(&mut out, 0xDA, &sos);

    let cos: [[f32; 8]; 8] = std::array::from_fn(|u| {
        std::array::from_fn(|x| {
            let c = if u == 0 {
                std::f32::consts::FRAC_1_SQRT_2
            } else {
                1.0
            };
            c / 2.0 * ((2.0 * x as f32 + 1.0) * u as f32 * std::f32::consts::PI / 16.0).cos()
        })
    });
    let mut bits = Bits {
        out: Vec::with_capacity(w * h),
        acc: 0,
        n: 0,
    };
    let mut pred = [0i32; 4];
    for by in (0..h).step_by(8) {
        for bx in (0..w).step_by(8) {
            for (comp, last) in pred.iter_mut().enumerate() {
                // The block, edge pixels repeated past the image.
                let block: [f32; 64] = std::array::from_fn(|i| {
                    let x = (bx + i % 8).min(w - 1);
                    let y = (by + i / 8).min(h - 1);
                    let v = ink.get((y * w + x) * 4 + comp).copied().unwrap_or(0);
                    // Stored inverted: 255 is no ink.
                    f32::from(255 - v) - 128.0
                });
                let f = fdct(&block, &cos);
                let q: [i32; 64] = std::array::from_fn(|i| {
                    (f[i] / f32::from(quant[i])).round().clamp(-1023.0, 1023.0) as i32
                });
                let diff = q[0] - *last;
                *last = q[0];
                let cat = category(diff);
                bits.put(
                    u32::from(dc.code[cat as usize]),
                    u32::from(dc.len[cat as usize]),
                );
                bits.put(extra(diff), cat);
                let mut run = 0u32;
                for &n in zz.iter().skip(1) {
                    let z = q[n];
                    if z == 0 {
                        run += 1;
                        continue;
                    }
                    while run > 15 {
                        bits.put(u32::from(ac.code[0xF0]), u32::from(ac.len[0xF0]));
                        run -= 16;
                    }
                    let cat = category(z);
                    let sym = ((run << 4) | cat) as usize;
                    bits.put(u32::from(ac.code[sym]), u32::from(ac.len[sym]));
                    bits.put(extra(z), cat);
                    run = 0;
                }
                if run > 0 {
                    bits.put(u32::from(ac.code[0]), u32::from(ac.len[0]));
                }
            }
        }
    }
    out.extend_from_slice(&bits.finish());
    out.extend_from_slice(&[0xFF, 0xD9]);
    Some(out)
}

/// A TIFF field: tag, type (1 BYTE, 3 SHORT, 4 LONG, 5 RATIONAL, 7
/// UNDEFINED), count and its value's bytes (little-endian).
struct Field {
    tag: u16,
    kind: u16,
    count: u32,
    value: Vec<u8>,
}

fn short(tag: u16, values: &[u16]) -> Field {
    Field {
        tag,
        kind: 3,
        count: values.len() as u32,
        value: values.iter().flat_map(|v| v.to_le_bytes()).collect(),
    }
}

fn long(tag: u16, values: &[u32]) -> Field {
    Field {
        tag,
        kind: 4,
        count: values.len() as u32,
        value: values.iter().flat_map(|v| v.to_le_bytes()).collect(),
    }
}

fn rational(tag: u16, num: u32, den: u32) -> Field {
    let mut value = num.to_le_bytes().to_vec();
    value.extend_from_slice(&den.to_le_bytes());
    Field {
        tag,
        kind: 5,
        count: 1,
        value,
    }
}

/// An IFD at file offset `at`: the count, the fields (sorted by tag),
/// a zero next-IFD offset, then the values longer than four bytes, each
/// starting on an even offset.
fn ifd(fields: &mut [Field], at: u32) -> Vec<u8> {
    fields.sort_by_key(|f| f.tag);
    let head = 2 + 12 * fields.len() + 4;
    let mut out = Vec::with_capacity(head);
    let mut extra: Vec<u8> = Vec::new();
    out.extend_from_slice(&(fields.len() as u16).to_le_bytes());
    for f in fields.iter() {
        out.extend_from_slice(&f.tag.to_le_bytes());
        out.extend_from_slice(&f.kind.to_le_bytes());
        out.extend_from_slice(&f.count.to_le_bytes());
        if f.value.len() <= 4 {
            let mut v = f.value.clone();
            v.resize(4, 0);
            out.extend_from_slice(&v);
        } else {
            let offset = at as usize + head + extra.len();
            out.extend_from_slice(&(offset as u32).to_le_bytes());
            extra.extend_from_slice(&f.value);
            if extra.len() % 2 == 1 {
                extra.push(0);
            }
        }
    }
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&extra);
    out
}

/// `dpi` as a TIFF rational in thousandths.
fn dpi_rational(dpi: f64) -> Option<(u32, u32)> {
    (dpi.is_finite() && dpi > 0.0).then(|| ((dpi * 1000.0).round().clamp(1.0, 4.0e9) as u32, 1000))
}

/// A little-endian TIFF header and an IFD0 that holds the resolution
/// alone, in dots per inch: an Exif block's body.
fn resolution_ifd(dpi: f64) -> Option<Vec<u8>> {
    let (num, den) = dpi_rational(dpi)?;
    let mut out = b"II*\0".to_vec();
    out.extend_from_slice(&8u32.to_le_bytes());
    let mut fields = vec![
        rational(282, num, den),
        rational(283, num, den),
        short(296, &[2]),
    ];
    out.extend_from_slice(&ifd(&mut fields, 8));
    Some(out)
}

/// `width` x `height` pixels of ink coverages (as for [`cmyk_jpeg`]) as
/// an uncompressed CMYK TIFF (PhotometricInterpretation 5, InkSet 1),
/// recording `dpi` and carrying `icc`.
pub fn cmyk_tiff(
    width: u32,
    height: u32,
    ink: &[u8],
    dpi: f64,
    icc: Option<&[u8]>,
) -> Option<Vec<u8>> {
    tiff_of(width, height, ink, 4, dpi, icc)
}

/// As [`cmyk_tiff`] with a fifth byte per pixel, its opacity (0 = clear,
/// 255 = opaque), written as an unassociated alpha channel
/// (ExtraSamples 2) so the file keeps the picture's transparency: the
/// inks are each pixel's full inks, not reduced by its opacity.
pub fn cmyka_tiff(
    width: u32,
    height: u32,
    ink_alpha: &[u8],
    dpi: f64,
    icc: Option<&[u8]>,
) -> Option<Vec<u8>> {
    tiff_of(width, height, ink_alpha, 5, dpi, icc)
}

/// A CMYK TIFF of `samples` bytes per pixel: 4 (inks) or 5 (inks and
/// alpha).
fn tiff_of(
    width: u32,
    height: u32,
    ink: &[u8],
    samples: usize,
    dpi: f64,
    icc: Option<&[u8]>,
) -> Option<Vec<u8>> {
    let (w, h) = (width as usize, height as usize);
    if width == 0 || height == 0 || ink.len() != w.checked_mul(h)?.checked_mul(samples)? {
        return None;
    }
    // Strips of about 64 KB, a whole number of rows each.
    let row = w * samples;
    let rows_per_strip = (65_536 / row).clamp(1, h);
    let strips: Vec<&[u8]> = ink.chunks(rows_per_strip * row).collect();
    let (num, den) = dpi_rational(dpi).unwrap_or((72_000, 1000));
    let fields = |offsets: &[u32]| {
        let mut f = vec![
            long(256, &[width]),
            long(257, &[height]),
            short(258, &vec![8; samples]),
            short(259, &[1]),
            short(262, &[5]),
            long(273, offsets),
            short(277, &[samples as u16]),
            long(278, &[rows_per_strip as u32]),
            long(
                279,
                &strips.iter().map(|s| s.len() as u32).collect::<Vec<_>>(),
            ),
            rational(282, num, den),
            rational(283, num, den),
            short(284, &[1]),
            short(296, &[2]),
            short(332, &[1]),
        ];
        if samples == 5 {
            f.push(short(338, &[2]));
        }
        if let Some(icc) = icc.filter(|i| !i.is_empty()) {
            f.push(Field {
                tag: 34675,
                kind: 7,
                count: icc.len() as u32,
                value: icc.to_vec(),
            });
        }
        f
    };
    // The IFD right after the header, the pixels after it.
    let placeholder = vec![0u32; strips.len()];
    let ifd_len = ifd(&mut fields(&placeholder), 8).len();
    let mut at = u32::try_from(8 + ifd_len).ok()?;
    let mut offsets = Vec::with_capacity(strips.len());
    for s in &strips {
        offsets.push(at);
        at = at.checked_add(s.len() as u32)?;
    }
    let mut out = b"II*\0".to_vec();
    out.extend_from_slice(&8u32.to_le_bytes());
    out.extend_from_slice(&ifd(&mut fields(&offsets), 8));
    for s in strips {
        out.extend_from_slice(s);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A TIFF field's values as numbers (SHORT and LONG), read the way
    /// TIFF 6.0 lays them out.
    fn tiff_field(b: &[u8], tag: u16) -> Option<Vec<u32>> {
        let le16 = |at: usize| Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?));
        let le32 = |at: usize| Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?));
        let at = le32(4)? as usize;
        for i in 0..le16(at)? as usize {
            let e = at + 2 + 12 * i;
            if le16(e)? != tag {
                continue;
            }
            let (kind, count) = (le16(e + 2)?, le32(e + 4)? as usize);
            let size = match kind {
                3 => 2,
                4 => 4,
                _ => return Some(vec![le32(e + 8)?]),
            };
            let base = if size * count <= 4 {
                e + 8
            } else {
                le32(e + 8)? as usize
            };
            return (0..count)
                .map(|k| {
                    if size == 2 {
                        le16(base + 2 * k).map(u32::from)
                    } else {
                        le32(base + 4 * k)
                    }
                })
                .collect();
        }
        None
    }

    fn pixels(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
        (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .flat_map(|(x, y)| f(x, y))
            .collect()
    }

    #[test]
    fn the_annex_k_tables_code_every_symbol_once() {
        assert_eq!(AC_BITS.iter().map(|&b| b as usize).sum::<usize>(), 162);
        assert_eq!(DC_BITS.iter().map(|&b| b as usize).sum::<usize>(), 12);
        let mut want: Vec<u8> = vec![0x00, 0xF0];
        for run in 0..16u8 {
            for size in 1..=10u8 {
                want.push(run << 4 | size);
            }
        }
        let mut have = AC_VALS.to_vec();
        want.sort_unstable();
        have.sort_unstable();
        assert_eq!(have, want);
        let z = zigzag();
        assert_eq!(&z[..10], &[0, 1, 8, 16, 9, 2, 3, 10, 17, 24]);
        assert_eq!(&z[60..], &[47, 55, 62, 63]);
        let mut sorted = z.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..64).collect::<Vec<_>>());
    }

    /// The JPEG decodes (with the decoder TraceDraw opens files with) to
    /// the inks written: four components, stored inverted under an Adobe
    /// marker with no transform, so cyan alone reads back as cyan.
    #[test]
    fn a_cmyk_jpeg_reads_back_with_its_inks() {
        let (w, h) = (37, 21);
        let ink = pixels(w, h, |x, _| {
            if x < 18 {
                [255, 0, 0, 0]
            } else {
                [0, 128, 0, 0]
            }
        });
        let icc = vec![7u8; 70_000];
        let jpg = cmyk_jpeg(w, h, &ink, 95, 300.0, Some(&icc)).expect("jpeg");
        assert!(jpg.windows(5).any(|s| s == b"Adobe"));
        let at = jpg
            .windows(2)
            .position(|s| s == [0xFF, 0xC0])
            .expect("SOF0");
        assert_eq!(jpg[at + 9], 4, "four components");
        assert_eq!(
            jpg.windows(12).filter(|s| s == b"ICC_PROFILE\0").count(),
            2,
            "a profile over 64 KB in two segments"
        );
        let img = image::load_from_memory_with_format(&jpg, image::ImageFormat::Jpeg)
            .expect("decodes")
            .to_rgb8();
        assert_eq!(img.dimensions(), (w, h));
        // The decoder shows CMYK as (255 - ink) times (255 - black): cyan
        // alone is no red, full green and blue.
        let p = img.get_pixel(4, 10).0;
        assert!(p[0] < 6 && p[1] > 249 && p[2] > 249, "{p:?}");
        let p = img.get_pixel(32, 10).0;
        assert!(
            p[0] > 249 && (p[1] as i32 - 127).abs() < 6 && p[2] > 249,
            "{p:?}"
        );
        // Quality 100 and 1 both decode; 1 is smaller.
        let busy = pixels(w, h, |x, y| {
            [(x * 7) as u8, (y * 11) as u8, (x * y) as u8, 3]
        });
        let best = cmyk_jpeg(w, h, &busy, 100, 72.0, None).expect("100");
        let worst = cmyk_jpeg(w, h, &busy, 1, 72.0, None).expect("1");
        assert!(worst.len() < best.len());
        for b in [&best, &worst] {
            image::load_from_memory(b).expect("decodes");
        }
        assert!(cmyk_jpeg(0, 4, &[], 90, 72.0, None).is_none());
        assert!(cmyk_jpeg(2, 2, &[0; 15], 90, 72.0, None).is_none());
    }

    #[test]
    fn a_cmyk_tiff_holds_the_inks_as_they_are() {
        let (w, h) = (300, 90);
        let ink = pixels(w, h, |x, y| [x as u8, y as u8, 0, 255]);
        let icc = vec![1u8, 2, 3];
        let tif = cmyk_tiff(w, h, &ink, 150.0, Some(&icc)).expect("tiff");
        assert_eq!(tiff_field(&tif, 262), Some(vec![5]), "separated");
        assert_eq!(tiff_field(&tif, 332), Some(vec![1]), "CMYK inks");
        assert_eq!(tiff_field(&tif, 277), Some(vec![4]));
        assert_eq!(tiff_field(&tif, 258), Some(vec![8, 8, 8, 8]));
        let offsets = tiff_field(&tif, 273).expect("strips");
        let counts = tiff_field(&tif, 279).expect("counts");
        assert!(offsets.len() > 1);
        let mut back = Vec::new();
        for (o, c) in offsets.iter().zip(&counts) {
            back.extend_from_slice(&tif[*o as usize..(*o + *c) as usize]);
        }
        assert_eq!(back, ink);
        assert_eq!(
            tiff_field(&tif, 34675),
            Some(vec![u32::from_le_bytes([1, 2, 3, 0])]),
            "a short profile inline"
        );
        assert!(cmyk_tiff(2, 2, &[0; 3], 72.0, None).is_none());
    }

    /// #1925: a CMYK TIFF with transparency has five samples a pixel,
    /// the fifth an unassociated alpha (ExtraSamples 2).
    #[test]
    fn a_cmyka_tiff_keeps_the_alpha_beside_the_inks() {
        let (w, h) = (3, 2);
        let px: Vec<u8> = (0..w * h)
            .flat_map(|i| [200, 10, i as u8, 0, if i % 2 == 0 { 255 } else { 0 }])
            .collect();
        let tif = cmyka_tiff(w, h, &px, 72.0, None).expect("tiff");
        assert_eq!(tiff_field(&tif, 262), Some(vec![5]));
        assert_eq!(tiff_field(&tif, 277), Some(vec![5]));
        assert_eq!(tiff_field(&tif, 258), Some(vec![8, 8, 8, 8, 8]));
        assert_eq!(tiff_field(&tif, 338), Some(vec![2]), "unassociated alpha");
        let o = tiff_field(&tif, 273).expect("strips")[0] as usize;
        assert_eq!(&tif[o..o + px.len()], &px[..]);
        assert_eq!(
            tiff_field(&cmyk_tiff(1, 1, &[0; 4], 72.0, None).expect("tiff"), 338),
            None
        );
        assert!(cmyka_tiff(2, 2, &[0; 16], 72.0, None).is_none());
    }
}
