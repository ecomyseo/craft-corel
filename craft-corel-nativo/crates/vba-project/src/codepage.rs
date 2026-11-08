//! Text in a VBA project is stored in the project's Windows code page
//! (module names, the PROJECT stream, the source), or as UTF-16.

use encoding_rs::Encoding;

/// The encoding of Windows code page `cp` (Western European when it is
/// unknown).
pub fn encoding(cp: u16) -> &'static Encoding {
    use encoding_rs as e;
    match cp {
        437 | 850 | 1252 | 20127 | 28591 => e::WINDOWS_1252,
        866 => e::IBM866,
        874 => e::WINDOWS_874,
        932 => e::SHIFT_JIS,
        936 => e::GBK,
        949 => e::EUC_KR,
        950 => e::BIG5,
        1200 => e::UTF_16LE,
        1201 => e::UTF_16BE,
        1250 => e::WINDOWS_1250,
        1251 => e::WINDOWS_1251,
        1253 => e::WINDOWS_1253,
        1254 => e::WINDOWS_1254,
        1255 => e::WINDOWS_1255,
        1256 => e::WINDOWS_1256,
        1257 => e::WINDOWS_1257,
        1258 => e::WINDOWS_1258,
        10000 => e::MACINTOSH,
        20866 => e::KOI8_R,
        21866 => e::KOI8_U,
        28592 => e::ISO_8859_2,
        28595 => e::ISO_8859_5,
        28597 => e::ISO_8859_7,
        28605 => e::ISO_8859_15,
        54936 => e::GB18030,
        65001 => e::UTF_8,
        _ => e::WINDOWS_1252,
    }
}

/// Bytes in code page `cp` as text.
pub fn decode(bytes: &[u8], cp: u16) -> String {
    encoding(cp)
        .decode_without_bom_handling(bytes)
        .0
        .into_owned()
}

/// Text as bytes in code page `cp` (characters it lacks become `?`).
pub fn encode(text: &str, cp: u16) -> Vec<u8> {
    let enc = encoding(cp);
    if enc == encoding_rs::UTF_16LE {
        return text.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    }
    let (bytes, _, had_errors) = enc.encode(text);
    if !had_errors {
        return bytes.into_owned();
    }
    // Encode character by character so a missing one becomes '?' instead
    // of an HTML reference.
    let mut out = Vec::new();
    let mut buf = [0u8; 4];
    for c in text.chars() {
        let (b, _, bad) = enc.encode(c.encode_utf8(&mut buf));
        if bad {
            out.push(b'?');
        } else {
            out.extend_from_slice(&b);
        }
    }
    out
}

/// UTF-16 little-endian bytes as text.
pub fn utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&units)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn western_cyrillic_and_japanese_pages() {
        assert_eq!(
            decode(&[0x43, 0x6F, 0x72, 0xE7, 0xE3, 0x6F], 1252),
            "Corção"
        );
        assert_eq!(decode(&[0xCF, 0xF0, 0xE8], 1251), "При");
        assert_eq!(decode(&[0x93, 0xFA], 932), "日");
        assert_eq!(encode("Corção", 1252), [0x43, 0x6F, 0x72, 0xE7, 0xE3, 0x6F]);
        assert_eq!(encode("日", 1252), b"?");
        assert_eq!(utf16(&[0x41, 0x00, 0xE5, 0x65]), "A日");
    }
}
