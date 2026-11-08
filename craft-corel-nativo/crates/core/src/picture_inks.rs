//! The ink channels of a print picture (#1659).
//!
//! A bitmap's pixels are RGB, for the screen. A picture read as CMYK
//! (a `.cdr` CMYK image) also keeps the ink bytes it was read with, so
//! colour separations print each ink as the file holds it: a picture of
//! cyan ink alone prints on the cyan plate alone, black ink alone on the
//! black plate, which a conversion of the screen pixels back to inks
//! cannot promise. The channels belong to the pixels they came with; once
//! the pixels change (an effect, a mode, a resample) they no longer fit
//! and separations convert the pixels instead.

use crate::SharedBytes;
use serde::{Deserialize, Serialize};

/// The ink of one channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PictureInk {
    /// A process ink: 0 cyan, 1 magenta, 2 yellow, 3 black.
    Process(u8),
    /// A spot ink of the drawing (`Document::inks`) by its id.
    Spot(u32),
}

/// A picture's ink channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PictureInks {
    /// The ink of each channel, in the order of the planes.
    pub inks: Vec<PictureInk>,
    /// The planes, one under the other in the order of `inks`, as one
    /// 8-bit grey PNG as wide as the picture and as high as the picture
    /// times the number of inks; 255 is full ink.
    pub png: SharedBytes,
    /// The [`fingerprint`] of the picture's PNG these channels belong to.
    pub of: u64,
    /// The source profile of these ink planes, kept per picture so drawings
    /// can contain pictures with different CMYK profiles without relabelling.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<crate::document::DocumentProfile>,
}

impl PictureInks {
    /// The CMYK channels of the picture `png` (`planes`: cyan, magenta,
    /// yellow, black, as a grey PNG).
    pub fn cmyk(png: &[u8], planes: SharedBytes) -> PictureInks {
        PictureInks {
            inks: (0..4).map(PictureInk::Process).collect(),
            png: planes,
            of: fingerprint(png),
            profile: None,
        }
    }

    /// Whether the channels belong to the picture `png`.
    pub fn fits(&self, png: &[u8]) -> bool {
        self.of == fingerprint(png)
    }
}

/// A fingerprint of some bytes (64-bit FNV-1a): the same in every run
/// and build, so it can be saved.
pub fn fingerprint(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inks_fit_only_the_pixels_they_came_with() {
        let inks = PictureInks::cmyk(b"picture", SharedBytes::from(vec![1, 2, 3]));
        assert!(inks.fits(b"picture"));
        assert!(!inks.fits(b"picture changed"));
        // The fingerprint is fixed (it is saved in files).
        assert_eq!(fingerprint(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fingerprint(b"a"), 0xaf63_dc4c_8601_ec8c);
        let json = serde_json::to_string(&inks).expect("json");
        let back: PictureInks = serde_json::from_str(&json).expect("read back");
        assert_eq!(back, inks);
    }
}
