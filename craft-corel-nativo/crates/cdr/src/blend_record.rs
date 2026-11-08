//! The settings of a live blend in its record (`LIST lnkg` of object
//! type 7, argument 3000) from version 23 on (#1192).
//!
//! Learned only by comparing files saved by the program that writes the
//! format, each with one blend between the same two objects and one
//! setting changed (the reporter's 134 version 27 files of #1192 and
//! #1576), and by measuring the steps on the pictures those files carry
//! (see `CLEANROOM.md` and `docs/cdr-format.md`). After the two centres
//! (bytes 28 to 44) the argument holds:
//!
//! - 44: u32 flags: 0x02 colours around the hue circle, 0x10 with it the
//!   hue falling from the lower object (the program's clockwise; without
//!   it the hue rises), 0x80 the rotation turns each step about its own
//!   centre (clear: loop), 0x100 the colour acceleration is even, 0x200
//!   the object acceleration is even, 0x400 set in the default record
//!   (0x780) and in one file whose accelerations are both even, clear in
//!   others of that kind (meaning not known), 0x800 the acceleration
//!   applies to the steps' sizes;
//! - 48: i32 rotation in millionths of a degree;
//! - 52: u32 0;
//! - 56: f64 colour acceleration factor, 64: f64 object acceleration
//!   factor: `1.3 ^ (s / 4)` for a whole `s` from -100 to 100, 1
//!   when even. A factor above 1 packs the steps (or their colours)
//!   toward the upper object, below 1 toward the lower one: step `t` of
//!   the way from the lower object lies `ln(1 + (f - 1) t) / ln f` of
//!   the way.
//!
//! Older versions keep zeros or other bytes there (versions 9 and 13:
//! zeros and leftover memory), so the settings are read and written from
//! version 23 only, where the record has the version 25 and 27 layout.

use tracedraw_core::live::{ColorBlend, BLEND_ACCEL_FACTOR};

/// The first version whose blend record holds the settings.
pub(crate) const SETTINGS_SINCE: u16 = 23;
/// Where the settings sit in the 88-byte argument.
const FLAGS_AT: usize = 44;
const ANGLE_AT: usize = 48;
const COLOR_ACCEL_AT: usize = 56;
const OBJECT_ACCEL_AT: usize = 64;
/// Bytes of the argument up to the end of the object acceleration.
const SETTINGS_END: usize = OBJECT_ACCEL_AT + 8;

const HUE: u32 = 0x02;
const HUE_FALLS: u32 = 0x10;
const OWN_CENTRE: u32 = 0x80;
const COLOR_EVEN: u32 = 0x100;
const OBJECT_EVEN: u32 = 0x200;
const BOTH_EVEN: u32 = 0x400;
const SIZING: u32 = 0x800;
/// The flags of a blend with every setting at its default, as every
/// default blend of the files holds them.
pub(crate) const DEFAULT_FLAGS: u32 = OWN_CENTRE | COLOR_EVEN | OBJECT_EVEN | BOTH_EVEN;
/// Millionths of a degree.
const ANGLE_UNIT: f64 = 1e6;
/// The largest rotation kept: ten turns either way.
const MAX_ROTATION: f64 = 3600.0;

/// A blend's settings as TraceDraw keeps them (`Effect::Blend`), for a
/// blend whose start is the lower of its two objects, the object the
/// record counts its steps from (and the one a reader gives the blend).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Settings {
    pub accel_objects: f64,
    pub accel_colors: f64,
    /// Degrees.
    pub rotation: f64,
    pub loop_rotation: bool,
    pub accel_sizing: bool,
    pub color_blend: ColorBlend,
}

impl Settings {
    /// The same blend seen from its other end: the start the upper
    /// object. The accelerations change sign and the hue turns the other
    /// way. The rotation keeps its value: the record turns the steps from
    /// the lower object, TraceDraw from the start, so a blend whose start
    /// is the upper object turns its steps from the other end there.
    pub(crate) fn reversed(self) -> Settings {
        Settings {
            accel_objects: -self.accel_objects,
            accel_colors: -self.accel_colors,
            color_blend: match self.color_blend {
                ColorBlend::Direct => ColorBlend::Direct,
                ColorBlend::Clockwise => ColorBlend::CounterClockwise,
                ColorBlend::CounterClockwise => ColorBlend::Clockwise,
            },
            ..self
        }
    }
}

/// The acceleration (-1 to 1, from the lower object) of a factor `f`;
/// 0 for a factor that is no number or not above 0 (TraceDraw wrote 0
/// there before #1192).
fn accel_of(f: f64) -> f64 {
    if !(f.is_finite() && f > 0.0) {
        return 0.0;
    }
    // A factor above 1 packs the steps toward the upper object, the
    // end: a negative acceleration.
    let a = -f.ln() / BLEND_ACCEL_FACTOR.ln();
    // The slider's whole percents come back as they were.
    let a = (a * 1e9).round() / 1e9;
    a.clamp(-1.0, 1.0)
}

/// The factor of acceleration `a` (from the lower object).
fn factor_of(a: f64) -> f64 {
    let a = if a.is_finite() {
        a.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    BLEND_ACCEL_FACTOR.powf(-a)
}

fn u32_at(d: &[u8], at: usize) -> Option<u32> {
    let b = d.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

fn f64_at(d: &[u8], at: usize) -> Option<f64> {
    let b = d.get(at..at.checked_add(8)?)?;
    let mut a = [0u8; 8];
    a.copy_from_slice(b);
    Some(f64::from_le_bytes(a))
}

/// The settings of the blend argument `arg` (its bytes) in a version `v`
/// file, for a blend whose start is the lower object. `None` before
/// version 23 or when the argument is too short.
pub(crate) fn decode(arg: &[u8], v: u16) -> Option<Settings> {
    if v < SETTINGS_SINCE || arg.len() < SETTINGS_END {
        return None;
    }
    let flags = u32_at(arg, FLAGS_AT)?;
    let angle = u32_at(arg, ANGLE_AT)? as i32 as f64 / ANGLE_UNIT;
    let color = f64_at(arg, COLOR_ACCEL_AT)?;
    let object = f64_at(arg, OBJECT_ACCEL_AT)?;
    let color_blend = if flags & HUE == 0 {
        ColorBlend::Direct
    } else if flags & HUE_FALLS != 0 {
        // The hue falls from the lower object, the start: TraceDraw's
        // Counterclockwise (hue decreasing).
        ColorBlend::CounterClockwise
    } else {
        ColorBlend::Clockwise
    };
    Some(Settings {
        accel_objects: accel_of(object),
        accel_colors: accel_of(color),
        rotation: angle.clamp(-MAX_ROTATION, MAX_ROTATION),
        loop_rotation: flags & OWN_CENTRE == 0,
        accel_sizing: flags & SIZING != 0,
        color_blend,
    })
}

/// The settings' bytes 44 to 72 of the blend argument (flags, angle, a
/// zero word, the colour and the object factor) for `s`, a blend whose
/// start is the lower object.
pub(crate) fn encode(s: &Settings) -> Vec<u8> {
    let even = |a: f64| !a.is_finite() || a == 0.0;
    let mut flags = 0;
    match s.color_blend {
        ColorBlend::Direct => {}
        ColorBlend::Clockwise => flags |= HUE,
        ColorBlend::CounterClockwise => flags |= HUE | HUE_FALLS,
    }
    if !s.loop_rotation {
        flags |= OWN_CENTRE;
    }
    if even(s.accel_colors) {
        flags |= COLOR_EVEN;
    }
    if even(s.accel_objects) {
        flags |= OBJECT_EVEN;
    }
    if even(s.accel_colors) && even(s.accel_objects) && s.rotation == 0.0 {
        flags |= BOTH_EVEN;
    }
    if s.accel_sizing {
        flags |= SIZING;
    }
    let rotation = if s.rotation.is_finite() {
        s.rotation.clamp(-MAX_ROTATION, MAX_ROTATION)
    } else {
        0.0
    };
    let mut out = Vec::with_capacity(SETTINGS_END - FLAGS_AT);
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&((rotation * ANGLE_UNIT).round() as i32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&factor_of(s.accel_colors).to_le_bytes());
    out.extend_from_slice(&factor_of(s.accel_objects).to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An 88-byte blend argument of 50 steps with `settings` (bytes 44 to
    /// 72), as the version 27 files hold it.
    fn arg(settings: &[u8]) -> Vec<u8> {
        let mut a = Vec::new();
        a.extend_from_slice(&50u16.to_le_bytes());
        a.extend_from_slice(&0u16.to_le_bytes());
        a.extend_from_slice(&100_000u32.to_le_bytes());
        a.extend_from_slice(&[0; 20]);
        for c in [-641_479i32, 173_097, 717_846, 160_370] {
            a.extend_from_slice(&c.to_le_bytes());
        }
        a.extend_from_slice(settings);
        a.extend_from_slice(&[0; 16]);
        a
    }

    fn settings(flags: u32, angle: i32, color: f64, object: f64) -> Vec<u8> {
        let mut s = Vec::new();
        s.extend_from_slice(&flags.to_le_bytes());
        s.extend_from_slice(&angle.to_le_bytes());
        s.extend_from_slice(&0u32.to_le_bytes());
        s.extend_from_slice(&color.to_le_bytes());
        s.extend_from_slice(&object.to_le_bytes());
        s
    }

    /// The records of the reporter's files, field by field (#1192).
    #[test]
    fn the_settings_of_the_files_are_read() {
        // Default: every flag of an untouched blend, factors 1.
        let d = decode(&arg(&settings(0x780, 0, 1.0, 1.0)), 27).expect("read");
        assert_eq!(d, Settings::default());
        // Object acceleration 37 (factor 1.3 ^ (37 / 4)), colour -2: the
        // steps packed toward the upper object (the end), their colours
        // a little toward the lower one.
        let d = decode(
            &arg(&settings(
                0x80,
                0,
                1.3f64.powf(-0.5),
                1.3f64.powf(37.0 / 4.0),
            )),
            27,
        )
        .expect("read");
        assert!((d.accel_objects + 0.37).abs() < 1e-9, "{d:?}");
        assert!((d.accel_colors - 0.02).abs() < 1e-9, "{d:?}");
        // The extremes, exactly.
        let d = decode(
            &arg(&settings(
                0x80,
                0,
                705.641_001_486_682_3,
                1.0 / 705.641_001_486_682_3,
            )),
            27,
        )
        .expect("read");
        assert_eq!((d.accel_colors, d.accel_objects), (-1.0, 1.0));
        // Rotation 45 degrees about each step's centre; -260 in a loop.
        let d = decode(&arg(&settings(0x380, 45_000_000, 1.0, 1.0)), 27).expect("read");
        assert_eq!((d.rotation, d.loop_rotation), (45.0, false));
        let d = decode(&arg(&settings(0x300, -260_000_000, 1.0, 1.0)), 27).expect("read");
        assert_eq!((d.rotation, d.loop_rotation), (-260.0, true));
        // Colour direction: the hue falling from the lower object (0x12,
        // the program's clockwise: red, magenta, blue, green) and rising
        // (0x02: red, yellow, green).
        let d = decode(&arg(&settings(0x792, 0, 1.0, 1.0)), 27).expect("read");
        assert_eq!(d.color_blend, ColorBlend::CounterClockwise);
        let d = decode(&arg(&settings(0x382, 0, 1.0, 1.0)), 27).expect("read");
        assert_eq!(d.color_blend, ColorBlend::Clockwise);
        // Apply to sizing.
        let d = decode(&arg(&settings(0x980, 0, 1.0, 2.5)), 27).expect("read");
        assert!(d.accel_sizing);
    }

    /// Records TraceDraw wrote before (factors 0), older versions and
    /// short or broken arguments read as the defaults or not at all.
    #[test]
    fn records_without_settings_read_as_even() {
        let d = decode(&arg(&settings(0x780, 0, 0.0, 0.0)), 25).expect("read");
        assert_eq!(d, Settings::default());
        assert_eq!(decode(&arg(&settings(0x780, 0, 2.0, 2.0)), 22), None);
        assert_eq!(decode(&arg(&[]), 27), None);
        let d = decode(
            &arg(&settings(0x780, i32::MAX, f64::NAN, f64::INFINITY)),
            27,
        )
        .expect("read");
        assert_eq!((d.accel_colors, d.accel_objects), (0.0, 0.0));
        assert_eq!(d.rotation, i32::MAX as f64 / ANGLE_UNIT);
        let d = decode(&arg(&settings(0x780, 0, 1e300, -3.0)), 27).expect("read");
        assert_eq!((d.accel_colors, d.accel_objects), (-1.0, 0.0));
    }

    /// What is written reads back the same, and a default blend gets
    /// the default record's bytes.
    #[test]
    fn the_settings_go_back_as_they_came() {
        assert_eq!(encode(&Settings::default()), settings(0x780, 0, 1.0, 1.0));
        let s = Settings {
            accel_objects: 0.53,
            accel_colors: -0.15,
            rotation: -90.5,
            loop_rotation: true,
            accel_sizing: true,
            color_blend: ColorBlend::CounterClockwise,
        };
        let bytes = encode(&s);
        assert_eq!(bytes.len(), 28);
        let back = decode(&arg(&bytes), 27).expect("read");
        assert!((back.accel_objects - 0.53).abs() < 1e-9);
        assert!((back.accel_colors + 0.15).abs() < 1e-9);
        assert_eq!(back.rotation, -90.5);
        assert!(back.loop_rotation && back.accel_sizing);
        assert_eq!(back.color_blend, ColorBlend::CounterClockwise);
        // The file's own words for these settings.
        assert_eq!(u32_at(&bytes, 0), Some(0x812));
        let f = f64_at(&bytes, 20).expect("factor");
        assert!((f - 1.3f64.powf(-53.0 / 4.0)).abs() < 1e-9 * f);
        // Counterclockwise is the record's 0x12; seen from the upper
        // object the same blend turns the other way and packs the other
        // way.
        let ccw = Settings {
            color_blend: ColorBlend::CounterClockwise,
            ..Settings::default()
        };
        assert_eq!(u32_at(&encode(&ccw), 0), Some(0x792));
        let r = s.reversed();
        assert_eq!(r.color_blend, ColorBlend::Clockwise);
        assert!((r.accel_objects + 0.53).abs() < 1e-12);
        assert_eq!(r.reversed(), s);
    }
}
