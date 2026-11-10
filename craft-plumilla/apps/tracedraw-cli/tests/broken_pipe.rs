//! The CLI's output piped into a reader that stops early (`| head`) ends
//! the program quietly instead of a panic.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[test]
fn a_reader_that_stops_early_does_not_make_it_panic() {
    let sample = concat!(env!("CARGO_MANIFEST_DIR"), "/../../sample.tdraw");
    let mut child = Command::new(env!("CARGO_BIN_EXE_craft-plumilla-cli"))
        .args(["info", sample])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("runs");
    let stdout = child.stdout.take().expect("stdout");
    let mut first = String::new();
    BufReader::new(stdout)
        .read_line(&mut first)
        .expect("one line");
    // The reader is gone now.
    let out = child.wait_with_output().expect("ends");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("panicked"), "{err}");
    assert!(out.status.success(), "{:?}: {err}", out.status);
    assert!(!first.is_empty());
}
