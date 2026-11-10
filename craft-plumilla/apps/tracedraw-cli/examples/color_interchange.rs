//! Original synthetic CMYK/ICC fixtures for repeatable file interchange checks.
use std::path::PathBuf;
use tracedraw_core::icc::test_profiles;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("an output directory is required")?,
    );
    std::fs::create_dir_all(&out)?;
    let profile = test_profiles::lut16_cmyk_profile();
    let samples: Vec<u8> = (0..16u8)
        .flat_map(|index| {
            [
                index * 16,
                33,
                49,
                65,
                [0, 127, 255][usize::from(index % 3)],
            ]
        })
        .collect();
    let tiff = tracedraw_io::cmyk_bitmap::cmyka_tiff(4, 4, &samples, 96.0, Some(&profile))
        .ok_or("fixture encoding failed")?;
    std::fs::write(out.join("source.tif"), &tiff)?;
    std::fs::write(out.join("source.icc"), &profile)?;
    let doc = tracedraw_io::raster::document_for_path(&tiff, &out.join("source.tif"))?;
    tracedraw_io::save_native(&doc, out.join("source.tdraw"))?;
    let mut per_image_only = doc.clone();
    per_image_only.cmyk_profile = None;
    tracedraw_io::save_native(&per_image_only, out.join("per-image-only.tdraw"))?;
    let mut matching = doc.clone();
    matching.cmyk_profile = matching.pages[0].layers[0].shapes[0]
        .picture_inks
        .as_ref()
        .and_then(|inks| inks.profile.clone());
    tracedraw_io::save_native(&matching, out.join("matching-profile.tdraw"))?;
    let unprofiled = tracedraw_io::cmyk_bitmap::cmyka_tiff(4, 4, &samples, 96.0, None)
        .ok_or("unprofiled fixture encoding failed")?;
    std::fs::write(out.join("unprofiled.tif"), &unprofiled)?;
    tracedraw_io::save_native(
        &tracedraw_io::raster::document_for_path(&unprofiled, &out.join("unprofiled.tif"))?,
        out.join("unprofiled.tdraw"),
    )?;
    std::fs::write(
        out.join("preserved.tif"),
        tracedraw_io::picture::original_cmyk_tiff(&doc.pages[0].layers[0].shapes[0], 96.0)?,
    )?;
    if std::env::args_os().any(|arg| arg == "--bench") {
        for side in [256_u32, 1024] {
            for mode in ["palette", "varied", "unique"] {
                let values: Vec<u8> = (0..side * side)
                    .flat_map(|index| {
                        let c = if mode == "palette" {
                            (index % 8) as u8 * 32
                        } else {
                            index as u8
                        };
                        let m = if mode == "palette" {
                            33
                        } else {
                            (index >> 8) as u8
                        };
                        if mode == "unique" {
                            let [c, m, y, k] = index
                                .wrapping_mul(1_664_525)
                                .wrapping_add(1_013_904_223)
                                .to_le_bytes();
                            [c, m, y, k, 255]
                        } else {
                            [c, m, 49, 65, 255]
                        }
                    })
                    .collect();
                let bytes = tracedraw_io::cmyk_bitmap::cmyka_tiff(
                    side,
                    side,
                    &values,
                    96.0,
                    Some(&profile),
                )
                .ok_or("benchmark encoding failed")?;
                let doc = tracedraw_io::raster::document_from_bytes(&bytes, "benchmark")?;
                let name = format!("benchmark-{side}-{mode}");
                let target = out.join(format!("{name}.cdr"));
                if !target.exists() {
                    std::fs::write(
                        target,
                        tracedraw_cdr::write::document_to_cdr_version(&doc, 27),
                    )?;
                }
                if let tracedraw_core::ShapeKind::Bitmap { png, .. } =
                    &doc.pages[0].layers[0].shapes[0].kind
                {
                    std::fs::write(out.join(format!("{name}-expected.png")), png.as_ref())?;
                }
            }
        }
    }
    println!("{}", out.display());
    Ok(())
}
