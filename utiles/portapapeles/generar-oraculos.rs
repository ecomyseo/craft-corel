//! Independent expectations: ICC primitives only, no clipboard/import/render code.
use std::{error::Error, path::PathBuf};
use tracedraw_core::icc::{Intent, Profile, Transform};

fn converted(
    samples: &[u8],
    source: &Profile,
    target: &Profile,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let transform = Transform::new(source, target, Intent::Saturation, false)
        .ok_or("unsupported ink transform")?;
    let mut result = samples.to_vec();
    for pixel in result.chunks_exact_mut(5) {
        let input: [f32; 4] = std::array::from_fn(|i| f32::from(pixel[i]) / 255.0);
        for (byte, value) in pixel[..4].iter_mut().zip(transform.transform(&input)) {
            *byte = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    Ok(result)
}

fn display(samples: &[u8], source: &Profile) -> Result<Vec<u8>, Box<dyn Error>> {
    let transform = Transform::new(
        source,
        &Profile::srgb(),
        Intent::RelativeColorimetric,
        false,
    )
    .ok_or("unsupported display transform")?;
    let mut result = Vec::new();
    for pixel in samples.chunks_exact(5) {
        let input: [f32; 4] = std::array::from_fn(|i| f32::from(pixel[i]) / 255.0);
        result.extend(
            transform
                .transform(&input)
                .into_iter()
                .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8),
        );
        result.push(pixel[4]);
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let folder = PathBuf::from(std::env::args().nth(1).ok_or("missing fixture folder")?);
    let a = Profile::parse(&std::fs::read(folder.join("a.icc"))?)?;
    let b = Profile::parse(&std::fs::read(folder.join("b.icc"))?)?;
    let original = std::fs::read(folder.join("original.cmyka"))?;
    let ab = converted(&original, &a, &b)?;
    let ba = converted(&original, &b, &a)?;
    let vector = [0.17f32, 0.33, 0.49, 0.65];
    let vector_ab = Transform::new(&a, &b, Intent::Saturation, false)
        .ok_or("unsupported vector A to B transform")?
        .transform(&vector);
    let vector_ba = Transform::new(&b, &a, Intent::Saturation, false)
        .ok_or("unsupported vector B to A transform")?
        .transform(&vector);
    let json = format!(
        "{{\"a_to_b_cmyka\":{ab:?},\"b_to_a_cmyka\":{ba:?},\"original_a_rgba\":{:?},\"original_b_rgba\":{:?},\"a_to_b_rgba\":{:?},\"b_to_a_rgba\":{:?},\"vector_a_to_b\":{vector_ab:?},\"vector_b_to_a\":{vector_ba:?}}}\n",
        display(&original, &a)?,
        display(&original, &b)?,
        display(&ab, &b)?,
        display(&ba, &a)?
    );
    std::fs::write(folder.join("display-oracles.json"), json)?;
    Ok(())
}
