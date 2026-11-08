//! The compression VBA projects use for their `dir` stream and module
//! source (the "CompressedContainer" of the public MS-OVBA specification,
//! section 2.4.1): a signature byte, then chunks of at most 4096 bytes of
//! output, each either stored as is or as literal bytes and copy tokens.

use crate::Error;

/// Bytes of output one chunk holds at most.
const CHUNK: usize = 4096;
/// The most output a container may expand to, so a hostile file cannot
/// make us allocate without bound.
const MAX_OUTPUT: usize = 256 * 1024 * 1024;

/// The bit count of a copy token's offset for a token `difference` bytes
/// into its chunk's output: the smallest count whose power of two reaches
/// the difference, at least 4.
fn bit_count(difference: usize) -> u32 {
    let mut bits = 0;
    while (1usize << bits) < difference {
        bits += 1;
    }
    bits.max(4)
}

/// Expand a compressed container.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, Error> {
    let corrupt = |what: &str| Error::Corrupt(format!("compressed data: {what}"));
    if data.first() != Some(&1) {
        return Err(corrupt("no signature byte"));
    }
    let mut out: Vec<u8> = Vec::with_capacity(data.len() * 2);
    let mut pos = 1;
    while pos < data.len() {
        if data.len() - pos < 2 {
            return Err(corrupt("a chunk header is cut short"));
        }
        let header = u16::from_le_bytes([data[pos], data[pos + 1]]);
        if (header >> 12) & 0b111 != 0b011 {
            return Err(corrupt("a chunk header has a wrong signature"));
        }
        let size = (header & 0x0FFF) as usize + 3;
        let end = (pos + size).min(data.len());
        let start_out = out.len();
        let mut p = pos + 2;
        if header & 0x8000 == 0 {
            // Stored as is.
            out.extend_from_slice(&data[p..end]);
        } else {
            while p < end {
                let flags = data[p];
                p += 1;
                for bit in 0..8 {
                    if p >= end {
                        break;
                    }
                    if flags >> bit & 1 == 0 {
                        out.push(data[p]);
                        p += 1;
                        continue;
                    }
                    if p + 2 > end {
                        return Err(corrupt("a copy token is cut short"));
                    }
                    let token = u16::from_le_bytes([data[p], data[p + 1]]) as usize;
                    p += 2;
                    let difference = out.len() - start_out;
                    let bits = bit_count(difference);
                    let length_mask = 0xFFFFusize >> bits;
                    let length = (token & length_mask) + 3;
                    let offset = (token >> (16 - bits)) + 1;
                    if difference == 0 || offset > difference {
                        return Err(corrupt("a copy token points before its chunk"));
                    }
                    if out.len() + length > MAX_OUTPUT {
                        return Err(corrupt("it expands too far"));
                    }
                    let from = out.len() - offset;
                    // Byte by byte: the copy may overlap what it writes.
                    for i in 0..length {
                        let b = out[from + i];
                        out.push(b);
                    }
                }
            }
        }
        if out.len() - start_out > CHUNK + 2048 || out.len() > MAX_OUTPUT {
            return Err(corrupt("a chunk expands too far"));
        }
        pos = end;
    }
    Ok(out)
}

/// The longest earlier run in `chunk` matching the bytes at `pos`, as
/// (offset back, length), when it is at least 3 bytes long. The nearest
/// run wins among runs of the same length.
fn longest_match(chunk: &[u8], pos: usize) -> Option<(usize, usize)> {
    let bits = bit_count(pos);
    let max_length = (0xFFFFusize >> bits) + 3;
    let mut best: Option<(usize, usize)> = None;
    for candidate in (0..pos).rev() {
        let mut length = 0;
        while pos + length < chunk.len()
            && length < max_length
            && chunk[candidate + length] == chunk[pos + length]
        {
            length += 1;
        }
        if length >= 3 && best.is_none_or(|(_, l)| length > l) {
            best = Some((pos - candidate, length));
        }
    }
    best
}

/// The compressed form of one chunk of at most 4096 bytes.
fn compress_chunk(chunk: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(chunk.len() + chunk.len() / 8 + 1);
    let mut pos = 0;
    while pos < chunk.len() {
        let flag_at = out.len();
        out.push(0);
        let mut flags = 0u8;
        for bit in 0..8 {
            if pos >= chunk.len() {
                break;
            }
            match longest_match(chunk, pos) {
                Some((offset, length)) => {
                    let bits = bit_count(pos);
                    let token = ((offset - 1) << (16 - bits)) | (length - 3);
                    out.extend_from_slice(&(token as u16).to_le_bytes());
                    flags |= 1 << bit;
                    pos += length;
                }
                None => {
                    out.push(chunk[pos]);
                    pos += 1;
                }
            }
        }
        out[flag_at] = flags;
    }
    out
}

/// Compress `data` into a container that [`decompress`] (and VBA itself)
/// reads back. A chunk that does not get smaller is stored as is.
pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut out = vec![1u8];
    for chunk in data.chunks(CHUNK) {
        let packed = compress_chunk(chunk);
        if packed.len() > CHUNK {
            let header = 0x3000u16 | (chunk.len() + 2 - 3) as u16;
            out.extend_from_slice(&header.to_le_bytes());
            out.extend_from_slice(chunk);
        } else {
            let header = 0xB000u16 | (packed.len() + 2 - 3) as u16;
            out.extend_from_slice(&header.to_le_bytes());
            out.extend_from_slice(&packed);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The "no compression" example of the specification (3.2.1): only
    /// literal bytes, in tokens of eight.
    #[test]
    fn the_specification_example_without_copies() {
        let packed = [
            0x01, 0x19, 0xB0, 0x00, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x00, 0x69,
            0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x00, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76,
            0x2E,
        ];
        let text = b"abcdefghijklmnopqrstuv.";
        assert_eq!(decompress(&packed).unwrap(), text);
        assert_eq!(compress(text), packed);
    }

    /// A long run: one literal, then one copy token one byte back (bit
    /// count 4, so 12 bits of length: 0x045 + 3 = 72 more bytes).
    #[test]
    fn a_run_is_one_literal_and_one_copy() {
        let packed = [0x01, 0x03, 0xB0, 0x02, 0x61, 0x45, 0x00];
        let run = vec![b'a'; 73];
        assert_eq!(decompress(&packed).unwrap(), run);
        assert_eq!(compress(&run), packed);
    }

    #[test]
    fn text_and_noise_go_round_trip_over_several_chunks() {
        let mut text = Vec::new();
        for i in 0..900 {
            text.extend_from_slice(
                format!(
                    "Sub Macro{i}()\r\n    ActiveLayer.CreateRectangle 0, 0, {i}, 1\r\nEnd Sub\r\n"
                )
                .as_bytes(),
            );
        }
        let packed = compress(&text);
        assert!(
            packed.len() < text.len() / 3,
            "{} of {}",
            packed.len(),
            text.len()
        );
        assert_eq!(decompress(&packed).unwrap(), text);
        // Bytes that do not compress are stored as is.
        let mut x: u32 = 0x1234_5678;
        let noise: Vec<u8> = (0..10_000)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x >> 24) as u8
            })
            .collect();
        let packed = compress(&noise);
        assert_eq!(packed[2] & 0x80, 0, "the first chunk is stored as is");
        assert_eq!(decompress(&packed).unwrap(), noise);
        assert_eq!(decompress(&compress(&[])).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn broken_containers_are_errors_not_panics() {
        for bad in [
            &[][..],
            &[0x02, 0x00][..],
            &[0x01, 0x19][..],
            &[0x01, 0x19, 0x40, 0x00][..],
            &[0x01, 0x03, 0xB0, 0x01, 0x45, 0x00][..],
            &[0x01, 0x03, 0xB0, 0x02, 0x61, 0x45][..],
        ] {
            assert!(decompress(bad).is_err(), "{bad:?}");
        }
        // Every prefix of a valid container either reads or fails cleanly.
        let packed = compress(b"Attribute VB_Name = \"Module1\"\r\nSub A()\r\nEnd Sub\r\n");
        for n in 0..packed.len() {
            let _ = decompress(&packed[..n]);
        }
    }
}
