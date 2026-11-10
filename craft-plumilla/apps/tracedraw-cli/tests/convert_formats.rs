//! Format identities, physical raster size and refusal of unknown output formats.
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "craft-plumilla-formats-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("new test-only directory");
        std::fs::write(path.join("input.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="70mm" viewBox="0 0 100 70"><rect width="100" height="70" fill="#ffffff"/><rect x="10" y="10" width="40" height="30" fill="#19a35f"/></svg>"##).expect("owned fixture");
        Self(path)
    }

    fn convert(
        &self,
        input: &Path,
        output: &Path,
        variables: &[(&str, &str)],
    ) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_craft-plumilla-cli"));
        command
            .args(["convert"])
            .arg(input)
            .arg(output)
            .stdin(Stdio::null());
        command
            .env_remove("TRACEDRAW_AI_VERSION")
            .env_remove("TRACEDRAW_PAGE")
            .env_remove("TRACEDRAW_DPI")
            .env_remove("TRACEDRAW_QUALITY")
            .env_remove("TRACEDRAW_WEBP_LOSSLESS");
        for &(key, value) in variables {
            command.env(key, value);
        }
        command.output().expect("run headless CLI")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // This directory was created exclusively by this fixture, under the OS temp directory.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn raster_extensions_contain_the_actual_image_formats_and_can_be_reopened() {
    let fixture = Fixture::new();
    for (extension, expected) in [
        ("png", image::ImageFormat::Png),
        ("jpg", image::ImageFormat::Jpeg),
        ("jpeg", image::ImageFormat::Jpeg),
        ("bmp", image::ImageFormat::Bmp),
        ("tif", image::ImageFormat::Tiff),
        ("tiff", image::ImageFormat::Tiff),
        ("webp", image::ImageFormat::WebP),
        ("gif", image::ImageFormat::Gif),
    ] {
        let output = fixture.0.join(format!("output.{extension}"));
        let result = fixture.convert(&fixture.0.join("input.svg"), &output, &[]);
        assert!(
            result.status.success(),
            "{extension}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let data = std::fs::read(&output).expect("output exists");
        assert_eq!(
            image::guess_format(&data).expect("real image signature"),
            expected,
            "{extension}"
        );
        let decoded = image::load_from_memory(&data)
            .expect("independent image decoder")
            .to_rgb8();
        assert!(
            decoded
                .pixels()
                .any(|pixel| pixel.0[1] > pixel.0[0].saturating_add(30)),
            "{extension}: expected coloured artwork"
        );
        let reopened = fixture.0.join(format!("reopened-{extension}.tdraw"));
        let result = fixture.convert(&output, &reopened, &[]);
        assert!(
            result.status.success(),
            "{extension} reopen: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let document = tracedraw_core::Document::from_json(
            &std::fs::read_to_string(reopened).expect("native JSON"),
        )
        .expect("typed document");
        assert_eq!(
            document.pages[0].layers[0].shapes.len(),
            1,
            "{extension}: bitmap preserved"
        );
    }
}

#[test]
fn png_resolution_survives_the_export_import_cycle() {
    let fixture = Fixture::new();
    let output = fixture.0.join("300dpi.png");
    let result = fixture.convert(
        &fixture.0.join("input.svg"),
        &output,
        &[("TRACEDRAW_DPI", "300")],
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let image = image::open(&output).expect("PNG");
    let reopened = fixture.0.join("300dpi.tdraw");
    let result = fixture.convert(&output, &reopened, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let document = tracedraw_core::Document::from_json(
        &std::fs::read_to_string(reopened).expect("native JSON"),
    )
    .expect("typed document");
    let size = document.pages[0].size;
    assert!((size.width - f64::from(image.width()) * 25.4 / 300.0).abs() < 0.002);
    assert!((size.height - f64::from(image.height()) * 25.4 / 300.0).abs() < 0.002);
}

#[test]
fn ai_is_a_real_pdf_or_classic_ai_file_and_both_reopen() {
    let fixture = Fixture::new();
    for (mode, signature) in [("pdf", &b"%PDF-"[..]), ("8", &b"%!PS-Adobe"[..])] {
        let output = fixture.0.join(format!("output-{mode}.ai"));
        let result = fixture.convert(
            &fixture.0.join("input.svg"),
            &output,
            &[("TRACEDRAW_AI_VERSION", mode)],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            std::fs::read(&output)
                .expect("output")
                .starts_with(signature),
            "{mode}: format signature"
        );
        let reopened = fixture.0.join(format!("reopened-ai-{mode}.tdraw"));
        let result = fixture.convert(&output, &reopened, &[]);
        assert!(
            result.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn unsupported_output_never_overwrites_an_existing_file() {
    let fixture = Fixture::new();
    let output = fixture.0.join("output.unsupported");
    std::fs::write(&output, b"preserve this file").expect("owned sentinel");
    let result = fixture.convert(&fixture.0.join("input.svg"), &output, &[]);
    assert!(!result.status.success());
    assert_eq!(
        std::fs::read(output).expect("sentinel"),
        b"preserve this file"
    );
}

#[test]
fn compressed_svg_is_gzip_with_svg_content_and_reopens() {
    use std::io::Read;
    let fixture = Fixture::new();
    let output = fixture.0.join("output.svgz");
    let result = fixture.convert(&fixture.0.join("input.svg"), &output, &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let data = std::fs::read(&output).expect("SVGZ");
    assert!(data.starts_with(&[0x1f, 0x8b]));
    let mut text = String::new();
    flate2::read::GzDecoder::new(data.as_slice())
        .read_to_string(&mut text)
        .expect("gzip stream");
    assert!(text.contains("<svg"));
    let result = fixture.convert(&output, &fixture.0.join("reopened-svgz.tdraw"), &[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn single_page_formats_honour_the_selected_page_and_reject_out_of_range_pages() {
    let fixture = Fixture::new();
    let mut doc =
        tracedraw_core::Document::new("two pages", tracedraw_core::geometry::Size::new(20.0, 10.0));
    let mut second = doc.pages[0].clone();
    second.id = doc.ids_mut().page();
    second.size = tracedraw_core::geometry::Size::new(60.0, 30.0);
    second.layers = vec![tracedraw_core::Layer::new(doc.ids_mut().layer(), "second")];
    doc.pages.push(second);
    let input = fixture.0.join("pages.tdraw");
    tracedraw_io::save_native(&doc, &input).unwrap();
    for suffix in ["svg", "svgz", "eps"] {
        let output = fixture.0.join(format!("second.{suffix}"));
        let result = fixture.convert(&input, &output, &[("TRACEDRAW_PAGE", "2")]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let reopened = fixture.0.join(format!("second-{suffix}.tdraw"));
        let result = fixture.convert(&output, &reopened, &[]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let actual =
            tracedraw_core::Document::from_json(&std::fs::read_to_string(reopened).unwrap())
                .unwrap();
        assert!(
            (actual.pages[0].size.width - 60.0).abs() < 0.001,
            "{suffix}"
        );
        assert!(
            (actual.pages[0].size.height - 30.0).abs() < 0.001,
            "{suffix}"
        );
        let original = std::fs::read(&output).unwrap();
        let result = fixture.convert(&input, &output, &[("TRACEDRAW_PAGE", "3")]);
        assert!(!result.status.success());
        assert_eq!(std::fs::read(&output).unwrap(), original);
    }
}

#[test]
fn converting_a_cmyk_tiff_to_native_preserves_its_original_inks_profile_and_alpha() {
    let fixture = Fixture::new();
    let profile = tracedraw_core::icc::test_profiles::lut16_cmyk_profile();
    let samples = vec![17, 33, 49, 65, 0, 251, 127, 3, 9, 127];
    let input = fixture.0.join("source-cmyk.tif");
    let bytes =
        tracedraw_io::cmyk_bitmap::cmyka_tiff(2, 1, &samples, 300.0, Some(&profile)).unwrap();
    std::fs::write(&input, &bytes).unwrap();
    let output = fixture.0.join("preserved.tdraw");
    let converted = fixture.convert(&input, &output, &[]);
    assert!(
        converted.status.success(),
        "{}",
        String::from_utf8_lossy(&converted.stderr)
    );
    let doc =
        tracedraw_core::Document::from_json(&std::fs::read_to_string(&output).unwrap()).unwrap();
    let shape = &doc.pages[0].layers[0].shapes[0];
    assert!(shape.cmyk_bitmap);
    let inks = shape.picture_inks.as_ref().unwrap();
    assert_eq!(inks.profile.as_ref().unwrap().data.as_slice(), profile);
    let planes = image::load_from_memory(&inks.png).unwrap().to_luma8();
    assert_eq!(planes.as_raw(), &[17, 251, 33, 127, 49, 3, 65, 9]);
    let tracedraw_core::ShapeKind::Bitmap { png, .. } = &shape.kind else {
        panic!("bitmap");
    };
    let pixels = image::load_from_memory(png).unwrap().to_rgba8();
    assert_eq!(pixels.get_pixel(0, 0).0[3], 0);
    assert_eq!(pixels.get_pixel(1, 0).0[3], 127);
    assert!((doc.pages[0].size.width - 2.0 * 25.4 / 300.0).abs() < 0.000001);
    assert_eq!(doc.cmyk_profile.as_ref().unwrap().data.as_slice(), profile);
    let cdr = fixture.0.join("preserved.cdr");
    assert!(fixture.convert(&output, &cdr, &[]).status.success());
    let reopened = fixture.0.join("reopened.tdraw");
    assert!(fixture.convert(&cdr, &reopened, &[]).status.success());
    let back =
        tracedraw_core::Document::from_json(&std::fs::read_to_string(reopened).unwrap()).unwrap();
    let shape = &back.pages[0].layers[0].shapes[0];
    let inks = shape.picture_inks.as_ref().unwrap();
    assert!(shape.cmyk_bitmap);
    assert_eq!(inks.profile.as_ref().unwrap().data.as_slice(), profile);
    assert_eq!(
        image::load_from_memory(&inks.png).unwrap().to_luma8(),
        planes
    );
    let tracedraw_core::ShapeKind::Bitmap { png, .. } = &shape.kind else {
        panic!("reopened bitmap");
    };
    assert_eq!(image::load_from_memory(png).unwrap().to_rgba8(), pixels);
    assert_eq!(std::fs::read(input).unwrap(), bytes);
}
