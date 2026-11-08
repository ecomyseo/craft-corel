//! Small independent fixtures at file/decompression byte-budget boundaries.
//! No large hostile files, OS UI, production paths or original application.
use super::App;
use crate::files;
use std::{
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(1);

struct Fixture {
    path: PathBuf,
    original: Vec<u8>,
}

impl Fixture {
    fn new(suffix: &str, bytes: &[u8]) -> Self {
        let folder = std::env::temp_dir()
            .join("craft-corel-file-budget-fixtures")
            .join(std::process::id().to_string());
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join(format!(
            "case-{}.{}",
            NEXT.fetch_add(1, Ordering::Relaxed),
            suffix
        ));
        std::fs::write(&path, bytes).unwrap();
        Self {
            path,
            original: bytes.to_vec(),
        }
    }
    fn intact(&self) {
        assert_eq!(
            std::fs::read(&self.path).unwrap(),
            self.original,
            "source file must remain byte-for-byte intact"
        );
    }
}

fn svg_exact(size: usize) -> String {
    let mut text=r#"<svg xmlns="http://www.w3.org/2000/svg" width="20mm" height="10mm" viewBox="0 0 20 10"><rect x="1" y="2" width="3" height="4" fill="rgb(20,150,80)"/></svg>"#.to_owned();
    assert!(text.len() <= size);
    text.extend(std::iter::repeat_n(' ', size - text.len()));
    assert_eq!(text.len(), size);
    // The positive and negative controls differ only by trailing whitespace.
    // Both are complete, valid SVG documents before the budget is enforced.
    tracedraw_io::svg_import::parse(&text, &mut tracedraw_core::id::IdSource::default()).unwrap();
    text
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

#[test]
fn bounded_binary_read_accepts_exact_limit_rejects_plus_one_and_preserves_source() {
    for limit in [0, 1, 31, 256] {
        let exact = Fixture::new("bin", &vec![0xa5; limit]);
        assert_eq!(
            files::read_limited(&exact.path, limit).unwrap(),
            exact.original
        );
        exact.intact();
        let excess = Fixture::new("bin", &vec![0xa5; limit + 1]);
        assert!(files::read_limited(&excess.path, limit).is_err());
        excess.intact();
        assert_eq!(
            files::read_limited(&excess.path, limit + 1).unwrap(),
            excess.original,
            "positive control proves the same file is readable"
        );
    }
}

#[test]
fn text_budget_counts_utf8_bytes_and_never_truncates_a_character() {
    let text = "España: ñ, € y é";
    assert!(text.len() > text.chars().count());
    let fixture = Fixture::new("txt", text.as_bytes());
    assert_eq!(
        files::read_to_string_limited(&fixture.path, text.len()).unwrap(),
        text
    );
    assert!(files::read_to_string_limited(&fixture.path, text.len() - 1).is_err());
    assert!(files::read_to_string_limited(&fixture.path, text.chars().count()).is_err());
    fixture.intact();
}

#[test]
fn invalid_utf8_is_rejected_as_text_but_accepted_as_bounded_binary() {
    let fixture = Fixture::new("txt", &[b'a', 0xc3, 0x28]);
    assert_eq!(
        files::read_limited(&fixture.path, 3).unwrap(),
        fixture.original
    );
    assert!(files::read_to_string_limited(&fixture.path, 3).is_err());
    fixture.intact();
    let empty = Fixture::new("txt", b"");
    assert_eq!(files::read_to_string_limited(&empty.path, 0).unwrap(), "");
}

#[test]
fn missing_file_is_an_error_and_is_never_created_by_bounded_read() {
    let fixture = Fixture::new("bin", b"unchanged");
    let missing = fixture.path.with_extension("missing");
    assert!(!missing.exists());
    assert!(files::read_limited(&missing, 256).is_err());
    assert!(files::read_to_string_limited(&missing, 256).is_err());
    assert!(!missing.exists());
    fixture.intact();
}

#[test]
fn plain_svg_byte_limit_has_valid_exact_and_plus_one_controls() {
    let exact = svg_exact(256);
    let excess = svg_exact(257);
    let accepted = Fixture::new("svg", exact.as_bytes());
    let rejected = Fixture::new("svg", excess.as_bytes());
    assert_eq!(
        App::read_svg_text_limited(&accepted.path, 256).unwrap(),
        exact
    );
    assert!(App::read_svg_text_limited(&rejected.path, 256).is_err());
    assert_eq!(
        App::read_svg_text_limited(&rejected.path, 257).unwrap(),
        excess
    );
    accepted.intact();
    rejected.intact();
}

#[test]
fn compressed_svg_budget_counts_expanded_utf8_not_short_gzip_input() {
    let exact = svg_exact(256);
    let excess = svg_exact(257);
    let exact_gzip = gzip(exact.as_bytes());
    let excess_gzip = gzip(excess.as_bytes());
    assert!(
        exact_gzip.len() < 256 && excess_gzip.len() < 256,
        "compressed controls are deliberately small"
    );
    let accepted = Fixture::new("svgz", &exact_gzip);
    let rejected = Fixture::new("svgz", &excess_gzip);
    assert_eq!(
        App::read_svg_text_limited(&accepted.path, 256).unwrap(),
        exact
    );
    assert!(App::read_svg_text_limited(&rejected.path, 256).is_err());
    assert_eq!(
        App::read_svg_text_limited(&rejected.path, 257).unwrap(),
        excess
    );
    accepted.intact();
    rejected.intact();
}

#[test]
fn compressed_svg_checks_crc_truncation_and_utf8_even_at_exact_output_limit() {
    let text = svg_exact(256);
    let bytes = gzip(text.as_bytes());
    let good = Fixture::new("svgz", &bytes);
    assert_eq!(App::read_svg_text_limited(&good.path, 256).unwrap(), text);
    let mut crc = bytes.clone();
    let at = crc.len() - 8;
    crc[at] ^= 1;
    let crc = Fixture::new("svgz", &crc);
    assert!(App::read_svg_text_limited(&crc.path, 256).is_err());
    let truncated = Fixture::new("svgz", &bytes[..bytes.len() - 4]);
    assert!(App::read_svg_text_limited(&truncated.path, 256).is_err());
    let invalid = Fixture::new("svgz", &gzip(&[0xc3, 0x28]));
    assert!(App::read_svg_text_limited(&invalid.path, 2).is_err());
    good.intact();
    crc.intact();
    truncated.intact();
    invalid.intact();
}

#[derive(Debug, PartialEq)]
struct AppState {
    doc: tracedraw_core::Document,
    history: (Vec<&'static str>, Vec<&'static str>),
    selected: Vec<tracedraw_core::ShapeId>,
    page: tracedraw_core::PageId,
    layer: Option<tracedraw_core::LayerId>,
    edit_version: u64,
    step: Option<u64>,
    engine: u64,
    dirty: bool,
    file: Option<PathBuf>,
    opened_from: Option<PathBuf>,
    cdr_version: Option<u16>,
    tdraw_version: Option<u32>,
    tabs: usize,
    active: usize,
    units: super::Units,
    fallback: Option<super::Units>,
}

fn state(app: &App) -> AppState {
    AppState {
        doc: app.doc().clone(),
        history: app.engine.history_labels(),
        selected: app.selection.clone(),
        page: app.page,
        layer: app.active_layer(),
        edit_version: app.engine.revision(),
        step: app.engine.last_step_id(),
        engine: app.engine.instance(),
        dirty: app.engine.is_dirty(),
        file: app.file.clone(),
        opened_from: app.opened_from.clone(),
        cdr_version: app.cdr_version,
        tdraw_version: app.tdraw_version,
        tabs: app.docs.len(),
        active: app.active_doc,
        units: app.units,
        fallback: app.ruler_fallback_units,
    }
}

fn app_with_redo() -> (App, tracedraw_core::Document) {
    use tracedraw_core::{
        geometry::{Rect, Size},
        Document, ShapeKind,
    };
    let mut app = App::headless();
    app.open_document(Document::new("Budget target", Size::new(100.0, 70.0)), None);
    let first = app
        .new_shape(ShapeKind::Rect {
            rect: Rect::new(20.0, 20.0, 30.0, 30.0),
            radius: 2.0,
            corners: None,
        })
        .unwrap();
    app.new_shape(ShapeKind::Rect {
        rect: Rect::new(40.0, 20.0, 50.0, 30.0),
        radius: 3.0,
        corners: None,
    })
    .unwrap();
    let redone = app.doc().clone();
    app.undo();
    app.selection = vec![first];
    assert!(!app.engine.history_labels().1.is_empty());
    (app, redone)
}

fn roots(app: &App) -> usize {
    app.doc().all_layers().map(|l| l.shapes.len()).sum()
}

#[test]
fn real_plain_svg_import_rejects_budget_plus_one_without_losing_active_selection_or_redo() {
    let exact = Fixture::new("svg", svg_exact(256).as_bytes());
    let excess = Fixture::new("svg", svg_exact(257).as_bytes());
    let (mut app, redone) = app_with_redo();
    let before = state(&app);
    files::with_test_input_budget(256, || app.import_svg(&excess.path));
    assert!(app.status.contains("budget"), "{}", app.status);
    assert_eq!(
        state(&app),
        before,
        "failed import must not allocate IDs or mutate any editing state"
    );
    app.redo();
    assert_eq!(app.doc(), &redone, "held redo remains usable");
    let (mut positive, _) = app_with_redo();
    let count = roots(&positive);
    files::with_test_input_budget(256, || positive.import_svg(&exact.path));
    assert_eq!(
        roots(&positive),
        count + 1,
        "exact-boundary control reaches actual import and commit"
    );
    let (mut same_file, _) = app_with_redo();
    let count = roots(&same_file);
    files::with_test_input_budget(257, || same_file.import_svg(&excess.path));
    assert_eq!(
        roots(&same_file),
        count + 1,
        "same rejected source succeeds at its exact budget"
    );
    exact.intact();
    excess.intact();
}

#[test]
fn real_svgz_import_rejects_expansion_after_small_compressed_input_without_mutation() {
    let mut text =
        r#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1"/></svg>"#.to_owned();
    assert!(text.len() < 128);
    text.extend(std::iter::repeat_n(' ', 512 - text.len()));
    let bytes = gzip(text.as_bytes());
    assert!(
        bytes.len() < 128,
        "input must pass the128-byte budget so this isolates expansion"
    );
    let fixture = Fixture::new("svgz", &bytes);
    let (mut app, redone) = app_with_redo();
    let before = state(&app);
    files::with_test_input_budget(128, || app.import_svg(&fixture.path));
    assert!(
        app.status.contains("expanded text budget"),
        "{}",
        app.status
    );
    assert_eq!(state(&app), before);
    app.redo();
    assert_eq!(app.doc(), &redone);
    let (mut positive, _) = app_with_redo();
    let count = roots(&positive);
    files::with_test_input_budget(512, || positive.import_svg(&fixture.path));
    assert_eq!(
        roots(&positive),
        count + 1,
        "same valid gzip imports when its expanded size is allowed"
    );
    fixture.intact();
}

#[test]
fn real_tdraw_open_rejects_before_switching_document_or_discarding_redo() {
    let (mut app, redone) = app_with_redo();
    let source = app.doc().clone();
    let json = source.to_json().unwrap();
    assert!(json.len() > 128);
    assert_eq!(tracedraw_core::Document::from_json(&json).unwrap(), source);
    let fixture = Fixture::new("tdraw", json.as_bytes());
    let before = state(&app);
    let error =
        files::with_test_input_budget(128, || app.try_open_path(fixture.path.clone())).unwrap_err();
    assert!(error.contains("budget"), "{error}");
    assert_eq!(state(&app), before);
    app.redo();
    assert_eq!(app.doc(), &redone);
    let (mut positive, _) = app_with_redo();
    let tabs = positive.docs.len();
    files::with_test_input_budget(json.len(), || positive.try_open_path(fixture.path.clone()))
        .unwrap();
    assert_eq!(
        positive.docs.len(),
        tabs + 1,
        "same valid document opens at its exact byte budget"
    );
    assert_eq!(positive.doc(), &source);
    fixture.intact();
}

#[test]
fn thread_local_budget_restores_after_nested_calls_and_panic_without_global_environment_changes() {
    let fixture = Fixture::new("bin", b"1234");
    assert_eq!(files::effective_input_budget(99), 99);
    files::with_test_input_budget(4, || {
        assert!(files::read(&fixture.path).is_ok());
        files::with_test_input_budget(3, || assert!(files::read(&fixture.path).is_err()));
        assert!(files::read(&fixture.path).is_ok());
        assert_eq!(files::effective_input_budget(99), 4);
    });
    assert_eq!(files::effective_input_budget(99), 99);
    let caught = std::panic::catch_unwind(|| {
        files::with_test_input_budget(3, || panic!("owned fixture exercises RAII restoration"))
    });
    assert!(caught.is_err());
    assert_eq!(files::effective_input_budget(99), 99);
    assert_eq!(files::read(&fixture.path).unwrap(), b"1234");
    fixture.intact();
}
