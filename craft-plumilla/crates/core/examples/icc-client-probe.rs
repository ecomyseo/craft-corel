//! Read-only comparison probe for a supplied ICC profile and CMYK samples.
use tracedraw_core::icc::{Intent, Profile, Transform};

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    assert!([3, 4].contains(&args.len()), "CMYK profile, sample JSON, optional RGB profile required");
    let profile = Profile::parse(&std::fs::read(&args[1]).expect("profile file"))
        .expect("valid profile");
    let input: Vec<[u8; 4]> = serde_json::from_slice(&std::fs::read(&args[2]).expect("samples file"))
        .expect("sample array");
    let mut result = Vec::new();
    for bpc in [false, true] {
        let transform = Transform::new(&profile, &Profile::srgb(), Intent::RelativeColorimetric, bpc)
            .expect("CMYK to sRGB transform");
        let samples: Vec<_> = input.iter().map(|sample| {
            let rgb = transform.cmyk_to_rgb(sample.map(|v| f32::from(v) / 255.0))
                .expect("four channel conversion");
            let bytes = rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
            serde_json::json!({"cmyk": sample, "rgb": bytes, "rgb_float": rgb})
        }).collect();
        result.push(serde_json::json!({"bpc": bpc, "samples": samples}));
    }
    if args.len() == 4 {
        let rgb = Profile::parse(&std::fs::read(&args[3]).expect("RGB profile file"))
            .expect("valid RGB profile");
        for bpc in [false, true] {
            let transform = Transform::new(&rgb, &profile, Intent::RelativeColorimetric, bpc)
                .expect("RGB to CMYK transform");
            let samples: Vec<_> = input.iter().map(|sample| {
                let input = [sample[0], sample[1], sample[2]];
                let inks = transform.rgb_to_cmyk(input.map(|v| f32::from(v) / 255.0))
                    .expect("three channel conversion");
                let bytes = inks.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
                serde_json::json!({"rgb": input, "cmyk": bytes, "cmyk_float": inks})
            }).collect();
            result.push(serde_json::json!({"direction": "RGBtoCMYK", "bpc": bpc, "samples": samples}));
        }
    }
    println!("{}", serde_json::to_string(&result).expect("JSON"));
}
