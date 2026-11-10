//! `convert` reports on stderr what the .cdr reader skipped or guessed,
//! as it does for the other formats, and still writes the file.

use std::process::Command;

#[test]
fn converting_a_cdr_file_reports_the_reader_warnings() {
    let dir = std::env::temp_dir().join(format!("craft-plumilla-cli-warn-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    // A page size past what the reader takes: it warns and assumes A4.
    let doc =
        tracedraw_core::Document::new("w", tracedraw_core::geometry::Size::new(150_000.0, 1_000.0));
    let input = dir.join("wide.cdr");
    std::fs::write(&input, tracedraw_cdr::write::document_to_cdr(&doc)).expect("written");
    let output = dir.join("wide.pdf");
    let out = Command::new(env!("CARGO_BIN_EXE_craft-plumilla-cli"))
        .arg("convert")
        .arg(&input)
        .arg(&output)
        .output()
        .expect("runs");
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    let written = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(out.status.success(), "{:?}: {err}", out.status);
    assert!(written > 0);
    assert!(
        err.contains("warning: ") && err.contains("assuming A4"),
        "{err}"
    );
}
