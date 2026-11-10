//! `.ggr` gradient files, read into fountain fill stops.
//!
//! The format is text: a `GIMP Gradient` header line (the string the
//! format requires), an optional `Name:` line,
//! the number of segments, then one line per segment: left, middle and
//! right positions (0 to 1), the left colour (r g b a), the right colour
//! (r g b a), the blending function (0 linear, 1 curved, 2 sine, 3 sphere
//! increasing, 4 sphere decreasing, 5 step), the colouring (0 RGB, 1 HSV
//! counter-clockwise, 2 HSV clockwise) and, in newer files, the kind of
//! each end colour (0 fixed, 1 and 2 foreground, 3 and 4 background).
//!
//! Fountain fills blend their stops in a straight line, so a segment that
//! is not a plain linear RGB blend with its middle halfway is sampled into
//! several stops. Alpha is not kept: fountain stops are opaque.

use tracedraw_core::color::{hsb_to_rgb, rgb_to_hsb};
use tracedraw_core::{Color, Stop};

/// Stops each non-linear segment is sampled into.
const SAMPLES: usize = 16;

/// A gradient read from a `.ggr` file.
#[derive(Debug, Clone, PartialEq)]
pub struct GgrGradient {
    pub name: String,
    pub stops: Vec<Stop>,
}

#[derive(Debug, Clone, Copy)]
struct Segment {
    left: f64,
    middle: f64,
    right: f64,
    from: [f64; 3],
    to: [f64; 3],
    blend: u32,
    coloring: u32,
}

/// Read a `.ggr` file. Lines that do not parse are skipped; a file with no
/// usable segment is an error.
pub fn read(text: &str) -> Result<GgrGradient, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    if lines.next() != Some("GIMP Gradient") {
        return Err("not a .ggr gradient file".into());
    }
    let mut name = String::new();
    let mut segments = Vec::new();
    let mut count: Option<usize> = None;
    for line in lines {
        if let Some(n) = line.strip_prefix("Name:") {
            name = n.trim().to_string();
            continue;
        }
        if count.is_none() {
            count = line.parse().ok();
            continue;
        }
        if let Some(s) = segment(line) {
            segments.push(s);
        }
    }
    if segments.is_empty() {
        return Err("the gradient has no segments".into());
    }
    segments.sort_by(|a, b| a.left.total_cmp(&b.left));
    Ok(GgrGradient {
        name,
        stops: stops_of(&segments),
    })
}

fn segment(line: &str) -> Option<Segment> {
    let v: Vec<f64> = line
        .split_whitespace()
        .map(|w| w.parse::<f64>())
        .collect::<Result<_, _>>()
        .ok()?;
    if v.len() < 13 || v.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let ends = v.get(13..15).unwrap_or(&[0.0, 0.0]);
    // Foreground and background ends take black and white, the usual
    // defaults.
    let end = |rgb: [f64; 3], kind: f64| match kind as u32 {
        1 | 2 => [0.0, 0.0, 0.0],
        3 | 4 => [1.0, 1.0, 1.0],
        _ => rgb,
    };
    let clamp = |x: f64| x.clamp(0.0, 1.0);
    let left = clamp(v[0]);
    let right = clamp(v[2]).max(left);
    Some(Segment {
        left,
        middle: v[1].clamp(left, right),
        right,
        from: end([clamp(v[3]), clamp(v[4]), clamp(v[5])], ends[0]),
        to: end(
            [clamp(v[7]), clamp(v[8]), clamp(v[9])],
            ends.get(1).copied().unwrap_or(0.0),
        ),
        blend: v[11] as u32,
        coloring: v[12] as u32,
    })
}

/// How far between the segment's colours the point `pos` (0 to 1 within
/// the segment) is, with the middle at `mid` (0 to 1 within the segment).
fn factor(blend: u32, pos: f64, mid: f64) -> f64 {
    let mid = mid.clamp(1e-6, 1.0 - 1e-6);
    let linear = if pos <= mid {
        0.5 * pos / mid
    } else {
        0.5 + 0.5 * (pos - mid) / (1.0 - mid)
    };
    match blend {
        1 => pos.max(0.0).powf(0.5f64.ln() / mid.ln()),
        2 => ((-std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * linear).sin() + 1.0) / 2.0,
        3 => (1.0 - (linear - 1.0).powi(2)).max(0.0).sqrt(),
        4 => 1.0 - (1.0 - linear * linear).max(0.0).sqrt(),
        5 => {
            if pos < mid {
                0.0
            } else {
                1.0
            }
        }
        _ => linear,
    }
}

fn mix(s: &Segment, f: f64) -> Color {
    let [r, g, b] = match s.coloring {
        1 | 2 => {
            let (h0, s0, v0) = rgb_to_hsb(s.from[0] as f32, s.from[1] as f32, s.from[2] as f32);
            let (h1, s1, v1) = rgb_to_hsb(s.to[0] as f32, s.to[1] as f32, s.to[2] as f32);
            // Counter-clockwise goes up the hue circle, clockwise down.
            let mut dh = h1 - h0;
            if s.coloring == 1 && dh < 0.0 {
                dh += 360.0;
            } else if s.coloring == 2 && dh > 0.0 {
                dh -= 360.0;
            }
            let f32f = f as f32;
            let c = hsb_to_rgb(h0 + dh * f32f, s0 + (s1 - s0) * f32f, v0 + (v1 - v0) * f32f);
            [c[0] as f64, c[1] as f64, c[2] as f64]
        }
        _ => [0, 1, 2].map(|i| s.from[i] + (s.to[i] - s.from[i]) * f),
    };
    Color::rgb8(
        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn stops_of(segments: &[Segment]) -> Vec<Stop> {
    let mut out: Vec<Stop> = Vec::new();
    let mut push = |pos: f64, color: Color| {
        // Two segments meeting at one place with one colour need one stop.
        if out
            .last()
            .is_some_and(|l| (l.pos - pos).abs() < 1e-9 && l.color == color)
        {
            return;
        }
        out.push(Stop::new(pos, color));
    };
    for s in segments {
        let width = s.right - s.left;
        let mid = if width > 0.0 {
            (s.middle - s.left) / width
        } else {
            0.5
        };
        let plain = s.blend == 0 && s.coloring == 0 && (mid - 0.5).abs() < 1e-9;
        let n = if plain || width <= 0.0 { 1 } else { SAMPLES };
        for i in 0..=n {
            let t = i as f64 / n as f64;
            push(s.left + width * t, mix(s, factor(s.blend, t, mid)));
        }
    }
    if out.len() == 1 {
        let only = out[0];
        out.push(Stop::new(1.0, only.color));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str = "GIMP Gradient\nName: Red to blue\n2\n\
0.0 0.25 0.5 1 0 0 1 1 1 1 1 0 0\n\
0.5 0.75 1.0 1 1 1 1 0 0 1 1 0 0\n";

    #[test]
    fn linear_segments_give_their_ends_as_stops() {
        let g = read(TWO).expect("gradient");
        assert_eq!(g.name, "Red to blue");
        let stops: Vec<(f64, [u8; 3])> =
            g.stops.iter().map(|s| (s.pos, s.color.to_rgb8())).collect();
        assert_eq!(
            stops,
            vec![
                (0.0, [255, 0, 0]),
                (0.5, [255, 255, 255]),
                (1.0, [0, 0, 255])
            ]
        );
    }

    #[test]
    fn other_blends_and_hsv_are_sampled() {
        // One sine segment from red to green through HSV counter-clockwise
        // (red, yellow, green), the newer form with end colour kinds.
        let text = "GIMP Gradient\n1\n0 0.5 1 1 0 0 1 0 1 0 1 2 1 0 0\n";
        let g = read(text).expect("gradient");
        assert_eq!(g.stops.len(), SAMPLES + 1);
        assert_eq!(g.stops[0].color.to_rgb8(), [255, 0, 0]);
        assert_eq!(g.stops[SAMPLES].color.to_rgb8(), [0, 255, 0]);
        let mid = g.stops[SAMPLES / 2].color.to_rgb8();
        assert!(mid[0] > 200 && mid[1] > 200 && mid[2] < 30, "{mid:?}");
        // Positions run from 0 to 1 in order.
        assert!(g.stops.windows(2).all(|w| w[0].pos <= w[1].pos));
    }

    #[test]
    fn foreground_and_background_ends_and_bad_files() {
        let text = "GIMP Gradient\n1\n0 0.5 1 1 0 0 1 1 0 0 1 0 0 1 3\n";
        let g = read(text).expect("gradient");
        assert_eq!(g.stops[0].color.to_rgb8(), [0, 0, 0]);
        assert_eq!(g.stops[1].color.to_rgb8(), [255, 255, 255]);
        assert!(read("hello").is_err());
        assert!(read("GIMP Gradient\n1\nnot numbers\n").is_err());
        // Cut anywhere: an error or a gradient, never a panic.
        for cut in 0..TWO.len() {
            let _ = read(&TWO[..cut]);
        }
    }
}
