//! TraceDraw: files dragged over a window on Wayland (#1142).
//!
//! winit 0.30 does not bind the Wayland data device, so a native Wayland
//! window never heard of files dragged over it from a file manager: no
//! `HoveredFile`, no `DroppedFile`, and the compositor showed the drag as
//! refused. The Wayland backend (`platform_impl/linux/wayland/state.rs`)
//! now takes the drag's events and asks this module what to send:
//!
//! - a drag that enters a window offering `text/uri-list` is accepted
//!   (copy) and its list is read at once, so that the files are known
//!   while they hover (the program's hint names what the drop will do);
//! - every enter and motion sends the pointer's place, as the X11 and
//!   macOS copies do, because the system drag sends the window no pointer
//!   moves: where the files are let go decides between importing them and
//!   opening them (TraceDraw's `drop_files.rs`);
//! - the drop sends `DroppedFile` for each file once the list is read (it
//!   may come after the drop), and the offer is then finished;
//! - a drag that leaves without a drop cancels the hovered files.
//!
//! This file depends on nothing in the crate, and TraceDraw's tests
//! include and run it (`winit_file_drop::tests`).

use std::path::PathBuf;

/// The type of a list of files on the Wayland data device (RFC 2483).
pub const URI_LIST: &str = "text/uri-list";

/// The local files of a `text/uri-list`: one URI a line (CR LF, or LF
/// alone), `#` comment lines and blank lines skipped. Only `file:` URIs
/// of this computer count (`file:///p`, `file://localhost/p`, `file:/p`);
/// their `%XX` escapes are decoded, so a name with spaces or non-ASCII
/// letters comes back as it is on disk.
pub fn file_paths(list: &[u8]) -> Vec<PathBuf> {
    list.split(|&b| b == b'\n')
        .map(trim)
        .filter(|line| !line.is_empty() && line[0] != b'#')
        .filter_map(local_path)
        .collect()
}

fn trim(line: &[u8]) -> &[u8] {
    let mut line = line;
    while let [rest @ .., last] = line {
        if matches!(last, b'\r' | b'\0' | b' ' | b'\t') {
            line = rest;
        } else {
            break;
        }
    }
    while let [first, rest @ ..] = line {
        if matches!(first, b' ' | b'\t') {
            line = rest;
        } else {
            break;
        }
    }
    line
}

/// The path of one `file:` URI of this computer.
fn local_path(uri: &[u8]) -> Option<PathBuf> {
    let scheme = uri.get(..5)?;
    if !scheme.eq_ignore_ascii_case(b"file:") {
        return None;
    }
    let rest = &uri[5..];
    let path = if let Some(after) = rest.strip_prefix(b"//") {
        // An authority: empty or `localhost` is this computer.
        let slash = after.iter().position(|&b| b == b'/')?;
        let host = &after[..slash];
        if !host.is_empty() && !host.eq_ignore_ascii_case(b"localhost") {
            return None;
        }
        &after[slash..]
    } else if rest.first() == Some(&b'/') {
        rest
    } else {
        return None;
    };
    // A query or fragment is no part of the file's name.
    let end = path
        .iter()
        .position(|&b| b == b'?' || b == b'#')
        .unwrap_or(path.len());
    let bytes = percent_decode(&path[..end])?;
    Some(path_from_bytes(bytes))
}

fn percent_decode(text: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text[i] == b'%' {
            let hi = hex(*text.get(i + 1)?)?;
            let lo = hex(*text.get(i + 2)?)?;
            out.push(hi << 4 | lo);
            i += 3;
        } else {
            out.push(text[i]);
            i += 1;
        }
    }
    // A NUL cannot be in a path.
    (!out.is_empty() && !out.contains(&0)).then_some(out)
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(unix)]
fn path_from_bytes(bytes: Vec<u8>) -> PathBuf {
    use std::os::unix::ffi::OsStringExt;
    PathBuf::from(std::ffi::OsString::from_vec(bytes))
}

#[cfg(not(unix))]
fn path_from_bytes(bytes: Vec<u8>) -> PathBuf {
    PathBuf::from(String::from_utf8_lossy(&bytes).into_owned())
}

/// What the window is told.
#[derive(Debug, Clone, PartialEq)]
pub enum Out<W> {
    /// The pointer is at `x`, `y` in the window (`None`: over its frame,
    /// outside the area the program draws in).
    Moved(W, Option<(f64, f64)>),
    /// A file hovers over the window.
    Hovered(W, PathBuf),
    /// A file was let go over the window.
    Dropped(W, PathBuf),
    /// The hovered files left without a drop.
    Cancelled(W),
}

/// What to do with the offer after a drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterDrop {
    /// The files went out: finish and destroy the offer.
    Finish,
    /// Read the list (again) for this ticket; the files go out when it
    /// is read.
    Read(u64),
    /// The list is still being read; the files go out when it is.
    Wait,
    /// Nothing of ours was dropped: destroy the offer.
    Ignore,
}

#[derive(Debug)]
struct Drag<W> {
    window: W,
    /// The list's reading under way, or the one that gave `paths`.
    ticket: u64,
    reading: bool,
    paths: Vec<PathBuf>,
    hovered: bool,
    dropped: bool,
}

/// One drag of files over the windows of one seat.
#[derive(Debug)]
pub struct FileDrag<W> {
    drag: Option<Drag<W>>,
    next_ticket: u64,
}

impl<W> Default for FileDrag<W> {
    fn default() -> Self {
        Self {
            drag: None,
            next_ticket: 1,
        }
    }
}

impl<W: Copy + PartialEq> FileDrag<W> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a drag of files is over a window or was dropped and is
    /// still being read.
    pub fn active(&self) -> bool {
        self.drag.is_some()
    }

    fn ticket(&mut self) -> u64 {
        let t = self.next_ticket;
        self.next_ticket += 1;
        t
    }

    /// A drag enters `window` at `at`; `files` tells whether it offers a
    /// list of files. Returns the ticket of the list's reading to start.
    pub fn enter(
        &mut self,
        window: W,
        at: Option<(f64, f64)>,
        files: bool,
        out: &mut Vec<Out<W>>,
    ) -> Option<u64> {
        self.leave(out);
        // A drop still being read is given up for the new drag.
        self.drag = None;
        out.push(Out::Moved(window, at));
        if !files {
            return None;
        }
        let ticket = self.ticket();
        self.drag = Some(Drag {
            window,
            ticket,
            reading: true,
            paths: Vec::new(),
            hovered: false,
            dropped: false,
        });
        Some(ticket)
    }

    /// The drag moves to `at` over `window`.
    pub fn motion(&mut self, window: W, at: Option<(f64, f64)>, out: &mut Vec<Out<W>>) {
        match &self.drag {
            Some(d) if d.dropped => {}
            _ => out.push(Out::Moved(window, at)),
        }
    }

    /// The list of reading `ticket` has been read. Returns `true` when the
    /// files went out after a drop: the offer is then finished.
    pub fn data(&mut self, ticket: u64, list: &[u8], out: &mut Vec<Out<W>>) -> bool {
        let Some(d) = self
            .drag
            .as_mut()
            .filter(|d| d.ticket == ticket && d.reading)
        else {
            return false;
        };
        d.reading = false;
        d.paths = file_paths(list);
        if d.dropped {
            let d = self.drag.take().expect("drag checked above");
            for p in d.paths {
                out.push(Out::Dropped(d.window, p));
            }
            return true;
        }
        if !d.hovered && !d.paths.is_empty() {
            d.hovered = true;
            for p in &d.paths {
                out.push(Out::Hovered(d.window, p.clone()));
            }
        }
        false
    }

    /// Reading `ticket` failed. Returns `true` when the offer of a drop is
    /// to be given up (destroyed without files).
    pub fn read_failed(&mut self, ticket: u64, out: &mut Vec<Out<W>>) -> bool {
        let Some(d) = self
            .drag
            .as_mut()
            .filter(|d| d.ticket == ticket && d.reading)
        else {
            return false;
        };
        d.reading = false;
        if d.dropped {
            let d = self.drag.take().expect("drag checked above");
            if d.hovered {
                out.push(Out::Cancelled(d.window));
            }
            return true;
        }
        false
    }

    /// The files are let go.
    pub fn drop(&mut self, out: &mut Vec<Out<W>>) -> AfterDrop {
        let Some(d) = self.drag.as_mut() else {
            return AfterDrop::Ignore;
        };
        d.dropped = true;
        if d.reading {
            return AfterDrop::Wait;
        }
        if d.paths.is_empty() {
            // Read at the enter without a file (or not at all): the source
            // may only answer now.
            let ticket = self.ticket();
            let d = self.drag.as_mut().expect("drag checked above");
            d.ticket = ticket;
            d.reading = true;
            return AfterDrop::Read(ticket);
        }
        let d = self.drag.take().expect("drag checked above");
        for p in d.paths {
            out.push(Out::Dropped(d.window, p));
        }
        AfterDrop::Finish
    }

    /// The drag leaves the window. After a drop (the compositor sends a
    /// leave then too) the files still go out once read.
    pub fn leave(&mut self, out: &mut Vec<Out<W>>) {
        if self.drag.as_ref().is_some_and(|d| d.dropped) {
            return;
        }
        if let Some(d) = self.drag.take() {
            if d.hovered {
                out.push(Out::Cancelled(d.window));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_managers_list_gives_its_local_files() {
        let list = b"file:///home/ana/Envelope-test.cdr\r\nfile:///home/ana/screen%20shot.png\r\n";
        assert_eq!(
            file_paths(list),
            vec![
                PathBuf::from("/home/ana/Envelope-test.cdr"),
                PathBuf::from("/home/ana/screen shot.png")
            ]
        );
    }

    #[test]
    fn escapes_comments_hosts_and_other_schemes() {
        let list = b"# a comment\n\nfile://localhost/tmp/a.svg\nFILE:/tmp/b.tdraw\n\
            file://other-host/tmp/c.cdr\nhttps://example.org/d.png\nfile:///tmp/%D8%AC.cdr\0";
        assert_eq!(
            file_paths(list),
            vec![
                PathBuf::from("/tmp/a.svg"),
                PathBuf::from("/tmp/b.tdraw"),
                PathBuf::from("/tmp/\u{62c}.cdr")
            ]
        );
        // Broken escapes and NULs give no path.
        assert!(file_paths(b"file:///tmp/a%2").is_empty());
        assert!(file_paths(b"file:///tmp/a%00b").is_empty());
        assert!(file_paths(b"file://").is_empty());
        assert!(file_paths(b"").is_empty());
    }

    #[test]
    fn hover_then_drop_sends_the_place_and_the_files() {
        let mut drag = FileDrag::new();
        let mut out = Vec::new();
        let ticket = drag
            .enter(7, Some((10.0, 20.0)), true, &mut out)
            .expect("read the list");
        assert_eq!(out, vec![Out::Moved(7, Some((10.0, 20.0)))]);
        out.clear();
        assert!(!drag.data(ticket, b"file:///tmp/a.cdr\r\n", &mut out));
        assert_eq!(out, vec![Out::Hovered(7, PathBuf::from("/tmp/a.cdr"))]);
        out.clear();
        drag.motion(7, Some((300.0, 40.0)), &mut out);
        assert_eq!(drag.drop(&mut out), AfterDrop::Finish);
        assert_eq!(
            out,
            vec![
                Out::Moved(7, Some((300.0, 40.0))),
                Out::Dropped(7, PathBuf::from("/tmp/a.cdr"))
            ]
        );
        out.clear();
        // The compositor's leave after the drop cancels nothing.
        drag.leave(&mut out);
        assert!(out.is_empty());
        assert!(!drag.active());
    }

    #[test]
    fn a_drop_before_the_list_is_read_waits_for_it() {
        let mut drag = FileDrag::new();
        let mut out = Vec::new();
        let ticket = drag
            .enter(1, Some((5.0, 5.0)), true, &mut out)
            .expect("read the list");
        out.clear();
        assert_eq!(drag.drop(&mut out), AfterDrop::Wait);
        drag.leave(&mut out);
        assert!(out.is_empty());
        assert!(drag.data(ticket, b"file:///tmp/x.svg\nfile:///tmp/y.png\n", &mut out));
        assert_eq!(
            out,
            vec![
                Out::Dropped(1, PathBuf::from("/tmp/x.svg")),
                Out::Dropped(1, PathBuf::from("/tmp/y.png"))
            ]
        );
        assert!(!drag.active());
    }

    #[test]
    fn leaving_without_a_drop_cancels_the_hovered_files() {
        let mut drag = FileDrag::new();
        let mut out = Vec::new();
        let ticket = drag
            .enter(1, Some((1.0, 1.0)), true, &mut out)
            .expect("read the list");
        drag.data(ticket, b"file:///tmp/a.cdr", &mut out);
        out.clear();
        drag.leave(&mut out);
        assert_eq!(out, vec![Out::Cancelled(1)]);
        // A late answer to the old reading is ignored.
        out.clear();
        assert!(!drag.data(ticket, b"file:///tmp/a.cdr", &mut out));
        assert!(out.is_empty());
    }

    #[test]
    fn a_drag_without_files_only_moves_the_pointer() {
        let mut drag = FileDrag::new();
        let mut out = Vec::new();
        assert_eq!(drag.enter(3, None, false, &mut out), None);
        assert_eq!(out, vec![Out::Moved(3, None)]);
        out.clear();
        assert_eq!(drag.drop(&mut out), AfterDrop::Ignore);
        drag.leave(&mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn an_empty_list_on_hover_is_read_again_at_the_drop() {
        let mut drag = FileDrag::new();
        let mut out = Vec::new();
        let first = drag
            .enter(2, Some((0.0, 0.0)), true, &mut out)
            .expect("read the list");
        drag.data(first, b"", &mut out);
        out.clear();
        let AfterDrop::Read(second) = drag.drop(&mut out) else {
            panic!("the list is read again")
        };
        assert_ne!(first, second);
        assert!(drag.data(second, b"file:///tmp/late.cdr", &mut out));
        assert_eq!(out, vec![Out::Dropped(2, PathBuf::from("/tmp/late.cdr"))]);
    }

    #[test]
    fn a_failed_read_after_the_drop_gives_the_offer_up() {
        let mut drag = FileDrag::new();
        let mut out = Vec::new();
        let ticket = drag
            .enter(2, Some((0.0, 0.0)), true, &mut out)
            .expect("read the list");
        assert_eq!(drag.drop(&mut out), AfterDrop::Wait);
        out.clear();
        assert!(drag.read_failed(ticket, &mut out));
        assert!(out.is_empty());
        assert!(!drag.active());
    }

    #[test]
    fn a_new_drag_cancels_the_files_of_the_last() {
        let mut drag = FileDrag::new();
        let mut out = Vec::new();
        let t = drag
            .enter(1, Some((0.0, 0.0)), true, &mut out)
            .expect("read the list");
        drag.data(t, b"file:///tmp/a.cdr", &mut out);
        out.clear();
        drag.enter(2, Some((4.0, 4.0)), true, &mut out);
        assert_eq!(
            out,
            vec![Out::Cancelled(1), Out::Moved(2, Some((4.0, 4.0)))]
        );
    }
}
