//! tracedraw-cdr: a clean-room reader for `.cdr` files.
//!
//! What we know about the format comes from public reverse-engineering notes
//! and from observing real files, never from any vendor's code.
//!
//! Two containers exist:
//! - **RIFF** (CDR versions 7 through version 13, and the inner stream of newer files):
//!   `RIFF <size> CDR<v>` followed by nested `LIST` chunks.
//! - **ZIP** (version 14 and later): a ZIP archive whose `content/riffData.cdr`
//!   holds the RIFF stream; large payloads live in `content/data/*.dat`
//!   and are referenced from the stream.
//!
//! The reader has three layers:
//! 1. [`container`]: detect the container, hand back the RIFF bytes.
//! 2. [`riff`]: walk the chunk tree without interpreting it.
//! 3. [`parse`]: turn chunks into a [`tracedraw_core::Document`] (best effort).
//!
//! The first two, with the embedded font and compound file readers, live
//! in the `tracedraw-cdr-format` crate, which knows no document model;
//! they are re-exported here under their old paths.
//!
//! Anything we do not understand is skipped with a logged warning, never a
//! failure: opening a file must always yield a document.
#![forbid(unsafe_code)]

mod blend_record;
pub mod check;
pub mod cmx;
pub mod container;
mod kept_blends;
mod kept_effects;
mod kept_units;
pub mod parse;
mod progress;
mod shadow_image;
pub mod spot;
mod text;
pub mod wl;
pub mod write;

use std::path::Path;
use std::sync::Arc;

pub use container::{Container, Version};
pub use parse::{parse_document, parse_document_with, ParseReport};
pub use progress::{OpenPhase, OpenProgress, OpenProgressCallback};
pub use tracedraw_cdr_format::raw::{self, RawTable};
pub use tracedraw_cdr_format::{eot, loda, ole, riff, Error, Result};

/// Files list a radial or square gradient's colours from its edge to its
/// centre (a sun drawn red in the middle stores yellow first); ours run
/// from the centre out. The same mirror turns either order into the
/// other.
pub(crate) fn mirror_centred_stops(f: &mut tracedraw_core::Fountain) {
    use tracedraw_core::FountainKind;
    if matches!(f.kind, FountainKind::Radial | FountainKind::Square) {
        for s in &mut f.stops {
            s.pos = 1.0 - s.pos;
        }
        f.stops.reverse();
    }
}

/// Files measure a fountain fill's angle from its object's own x axis:
/// the fill turns with the object (#1257); ours is an angle on the page.
/// Turn `f` by `sign` times the angle the object's matrix `t` turns its x
/// axis by (1 reading, -1 writing). The matrix's stretch and skew leave
/// the angle as it is, as does a mirror (only where the x axis goes
/// counts). A matrix that does not turn the x axis changes nothing.
pub(crate) fn turn_fountain(
    f: &mut tracedraw_core::Fountain,
    t: tracedraw_core::Affine,
    sign: f64,
) {
    let [a, b, ..] = t.as_coeffs();
    let unturned = a > 0.0 && b.abs() <= 1e-12 * a;
    if !(a.is_finite() && b.is_finite()) || unturned || a.hypot(b) < 1e-12 {
        return;
    }
    let turned = f.angle + sign * b.atan2(a).to_degrees();
    if !turned.is_finite() {
        return;
    }
    // Kept within (-180, 180].
    let mut angle = turned.rem_euclid(360.0);
    if angle > 180.0 {
        angle -= 360.0;
    }
    f.angle = angle;
}

/// Read a `.cdr` file into a document. The report lists what was skipped.
pub fn open(path: impl AsRef<Path>) -> Result<(tracedraw_core::Document, ParseReport)> {
    let mut timing = progress::Timing::new("file");
    let bytes = read_input(path.as_ref(), container::MAX_STREAM)?;
    timing.lap("read-input");
    let title = path
        .as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();
    open_bytes(&bytes, &title)
}

fn check_input_size(bytes: u64, limit: u64) -> Result<()> {
    if bytes > limit {
        return Err(Error::UnsupportedContainer(format!(
            "input exceeds the {limit}-byte budget"
        )));
    }
    Ok(())
}

fn read_input(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    check_input_size(file.metadata()?.len(), limit)?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1)).read_to_end(&mut bytes)?;
    check_input_size(bytes.len() as u64, limit)?;
    Ok(bytes)
}

/// Same as [`open`], from memory. CMX exchange files (RIFF form `CMX1`)
/// are read as well, by [`cmx::open_bytes`].
pub fn open_bytes(bytes: &[u8], title: &str) -> Result<(tracedraw_core::Document, ParseReport)> {
    open_bytes_keeping(bytes, title).map(|(doc, report, _)| (doc, report))
}

/// [`open_bytes`] that also hands back the records the reader left
/// uninterpreted (the chunks and `loda` arguments it does not read),
/// keyed by object, as bytes shared with the file's streams. Empty for
/// CMX and `WL` files. The writer does not use it yet; it is what a
/// later save will put back untouched.
pub fn open_bytes_keeping(
    bytes: &[u8],
    title: &str,
) -> Result<(tracedraw_core::Document, ParseReport, RawTable)> {
    open_bytes_keeping_with_progress(bytes, title, None)
}

/// Open with optional byte/pixel progress. The final 100 is emitted only when
/// the complete document and retained records are ready to return.
pub fn open_bytes_keeping_with_progress(
    bytes: &[u8],
    title: &str,
    callback: Option<OpenProgressCallback>,
) -> Result<(tracedraw_core::Document, ParseReport, RawTable)> {
    let progress = progress::Progress::new(callback);
    let mut timing = progress::Timing::new("open");
    check_input_size(bytes.len() as u64, container::MAX_STREAM)?;
    if cmx::is_cmx(bytes) {
        let result = cmx::open_bytes(bytes, title).map(|(d, r)| (d, r, RawTable::default()))?;
        progress.report(OpenPhase::Complete, 100, 1, 1);
        return Ok(result);
    }
    if wl::is_wl(bytes) {
        let result = wl::open_bytes(bytes, title).map(|(d, r)| (d, r, RawTable::default()))?;
        progress.report(OpenPhase::Complete, 100, 1, 1);
        return Ok(result);
    }
    let container = container::detect(bytes)?;
    timing.lap("detect");
    let riff = Arc::new(container::riff_stream(bytes, &container)?);
    timing.lap("riff-decompress");
    let externals = container::external_streams_with_progress(bytes, &container, |done, total| {
        progress.streams(done, total);
    });
    timing.lap("external-decompress");
    let root = riff::parse_with_externals(&riff, externals, container.version.0)?;
    timing.lap("riff-tree");
    let cmyk_profile = container::cmyk_profile(bytes, &container);
    // Blends TraceDraw saved as groups open live again (#986).
    let kept = container::own_member(
        bytes,
        &container,
        kept_blends::MEMBER,
        kept_blends::MAX_MEMBER,
    )
    .map(|b| kept_blends::decode(&b))
    .unwrap_or_default();
    // Objects TraceDraw saved as what their effects and shadows draw open
    // live again too (#1031).
    let kept_effects = container::own_member(
        bytes,
        &container,
        kept_effects::MEMBER,
        kept_effects::MAX_MEMBER,
    )
    .map(|b| kept_effects::decode(&b))
    .unwrap_or_default();
    timing.lap("profiles-and-sidecars");
    let mut doc_report = parse::parse_document_kept_progress(
        &root,
        &riff,
        container.version,
        cmyk_profile.as_ref().map(|p| p.data.as_slice()),
        &kept,
        &kept_effects,
        &progress,
    );
    timing.lap("document-and-bitmaps");
    kept_units::restore(
        &mut doc_report.0,
        &mut doc_report.1,
        &root,
        &riff,
        (container.kind != container::ContainerKind::Riff).then_some(bytes),
    );
    doc_report.0.title = title.to_string();
    if let Some(intent) = container::document_rendering_intent(bytes, &container) {
        doc_report.0.metadata.rendering_intent = intent.name().to_string();
    }
    doc_report.0.palette = container::document_palette(bytes, &container)
        .into_iter()
        .map(|(_, c)| c)
        .collect();
    // Spot library colours take what the file's page preview shows; a
    // version 3 file's thumbnail (`imhd`, a `.bmp` of the drawing) serves
    // the same way.
    if !doc_report.0.inks.is_empty() {
        let thumbnail = || {
            let mut found = None;
            root.root.walk(&mut |c, _| {
                if c.is(b"imhd") && found.is_none() {
                    found = Some(c.clone());
                }
            });
            let c = found?;
            let mut p = spot::Preview::decode(root.data(&riff, &c))?;
            p.covers_drawing = true;
            Some(p)
        };
        let preview = if container.version.0 < 4 {
            thumbnail()
        } else {
            container::page_preview(bytes, &container)
        };
        if let Some(preview) = preview {
            let n = spot::colours_from_preview(&mut doc_report.0, &preview);
            if n > 0 {
                doc_report.1.warn(format!(
                    "{n} spot colour(s) take their look from the file's preview"
                ));
            }
        }
    }
    let (fonts, notes) = container::embedded_fonts(bytes, &container);
    doc_report.0.fonts = fonts;
    doc_report.0.cmyk_profile = cmyk_profile;
    doc_report.0.rgb_profile = container::rgb_profile(bytes, &container);
    doc_report.0.gray_profile = container::gray_profile(bytes, &container);
    doc_report.0.color_context = container::document_color_context(bytes, &container);
    if let Some(context) = &doc_report.0.color_context {
        if context.primary_model.eq_ignore_ascii_case("Rgb") {
            doc_report.0.metadata.primary_color_mode = tracedraw_core::document::PrimaryColorMode::Rgb;
        } else if context.primary_model.eq_ignore_ascii_case("Cmyk") {
            doc_report.0.metadata.primary_color_mode = tracedraw_core::document::PrimaryColorMode::Cmyk;
        }
    }
    if let Some(name) = doc_report.0.rgb_profile.as_ref().map(|profile| profile.name.clone())
        .or_else(|| container::declared_profile_name(bytes, &container, "Rgb")) {
        doc_report.0.metadata.rgb_profile = name;
    }
    if let Some(name) = doc_report.0.gray_profile.as_ref().map(|profile| profile.name.clone())
        .or_else(|| container::declared_profile_name(bytes, &container, "Grayscale")) {
        doc_report.0.metadata.gray_profile = name;
    }
    if let Some(name) = doc_report.0.cmyk_profile.as_ref().map(|profile| profile.name.clone())
        .or_else(|| container::declared_profile_name(bytes, &container, "Cmyk")) {
        doc_report.0.metadata.cmyk_profile = name;
    }
    inherit_picture_profile(&mut doc_report.0);
    for n in notes {
        doc_report.1.warn(n);
    }
    // What the reader left, by object: the records a later save can put
    // back. The version the report settled on reads the argument tables.
    let version = doc_report.1.version.unwrap_or(container.version).0;
    timing.lap("resources-and-profiles");
    progress.report(OpenPhase::Document, 98, 1, 2);
    let kept = raw::collect(&root, riff, version, &parse::known());
    doc_report.1.unknown = kept.summary();
    timing.lap("raw-records");
    progress.report(OpenPhase::Complete, 100, 2, 2);
    Ok((doc_report.0, doc_report.1, kept))
}

/// Retained process inks were decoded through the drawing's embedded CMYK
/// profile. Keep that association when the picture moves to another drawing or
/// is saved as a TIFF, rather than silently relabelling its original samples.
fn inherit_picture_profile(doc: &mut tracedraw_core::Document) {
    let Some(profile) = doc
        .cmyk_profile
        .as_ref()
        .filter(|p| {
            tracedraw_core::icc::Profile::parse(&p.data)
                .is_ok_and(|profile| profile.color_space() == tracedraw_core::icc::ColorSpace::Cmyk)
        })
        .cloned()
    else {
        return;
    };
    fn assign(shapes: &mut [tracedraw_core::Shape], profile: &tracedraw_core::DocumentProfile) {
        let mut pending: Vec<_> = shapes.iter_mut().collect();
        while let Some(shape) = pending.pop() {
            if let tracedraw_core::ShapeKind::Bitmap { png, .. } = &shape.kind {
                if let Some(inks) = shape
                    .picture_inks
                    .as_deref_mut()
                    .filter(|inks| inks.fits(png))
                {
                    if inks.profile.is_none() {
                        inks.profile = Some(profile.clone());
                    }
                }
            }
            match &mut shape.kind {
                tracedraw_core::ShapeKind::Group { children } => {
                    pending.extend(children.iter_mut())
                }
                tracedraw_core::ShapeKind::ClipFrame { frame, contents } => {
                    pending.push(frame);
                    pending.extend(contents.iter_mut());
                }
                _ => {}
            }
        }
    }
    for layer in doc
        .master
        .iter_mut()
        .chain(doc.pages.iter_mut().flat_map(|page| page.layers.iter_mut()))
    {
        assign(&mut layer.shapes, &profile);
    }
    for symbol in &mut doc.symbols {
        assign(std::slice::from_mut(&mut symbol.shape), &profile);
    }
}

#[cfg(test)]
mod input_budget_tests {
    use super::*;

    #[test]
    fn measured_open_progress_is_monotonic_and_completes_only_with_document() {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let received = events.clone();
        let callback: OpenProgressCallback =
            Arc::new(move |event| received.lock().unwrap().push(event));
        let bytes = b"RIFF\x04\x00\x00\x00CDRD";
        let (doc, _, _) =
            open_bytes_keeping_with_progress(bytes, "Progress fixture", Some(callback)).unwrap();
        assert_eq!(doc.title, "Progress fixture");
        let events = events.lock().unwrap();
        assert_eq!(events.first().unwrap().percent, 0);
        assert_eq!(events.last().unwrap().phase, OpenPhase::Complete);
        assert_eq!(events.last().unwrap().percent, 100);
        assert!(events[..events.len() - 1]
            .iter()
            .all(|event| event.percent < 100));
        assert!(events
            .windows(2)
            .all(|pair| pair[0].percent <= pair[1].percent));
        assert!(events.iter().any(|event| event.phase == OpenPhase::Bitmaps));
        assert!(events
            .iter()
            .any(|event| event.phase == OpenPhase::Document));
    }

    #[test]
    fn failed_open_progress_never_reports_completion() {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let received = events.clone();
        let callback: OpenProgressCallback =
            Arc::new(move |event| received.lock().unwrap().push(event));
        assert!(
            open_bytes_keeping_with_progress(b"invalid", "Invalid fixture", Some(callback))
                .is_err()
        );
        assert!(events
            .lock()
            .unwrap()
            .iter()
            .all(|event| event.percent < 100 && event.phase != OpenPhase::Complete));
    }

    #[test]
    fn bounded_cdr_file_read_accepts_exact_budget_and_rejects_one_more() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "craft-plumilla-cdr-input-{}-{unique}.bin",
            std::process::id()
        ));
        struct Remove(std::path::PathBuf);
        impl Drop for Remove {
            fn drop(&mut self) {
                let _ = std::fs::remove_file(&self.0);
            }
        }
        let _cleanup = Remove(path.clone());
        let content = b"RIFF-small-input";
        std::fs::write(&path, content).unwrap();
        assert_eq!(read_input(&path, content.len() as u64).unwrap(), content);
        let error = read_input(&path, content.len() as u64 - 1).unwrap_err();
        assert!(error.to_string().contains("budget"));
        assert!(check_input_size(container::MAX_STREAM, container::MAX_STREAM).is_ok());
        assert!(check_input_size(container::MAX_STREAM + 1, container::MAX_STREAM).is_err());
        assert!(check_input_size(0, 0).is_ok());
        assert!(check_input_size(1, 0).is_err());
    }
}
