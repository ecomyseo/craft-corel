//! System clipboard and external bitmap editing.
//!
//! Copy puts the selection on the system clipboard as SVG text next to the
//! internal clipboard, so other applications can paste it. Paste Special
//! offers the system clipboard's content as text, as a bitmap or as vector
//! objects (SVG). Image > Edit Bitmap hands a PNG to the system's image
//! editor and Update from Link reads it back.

use crate::app::App;
use crate::i18n::{tr, trf};
use tracedraw_core::{
    document::{ShapeKind, TextSpan},
    geometry::Point,
    Command, ShapeId,
};

/// Versioned in-memory clipboard: only selected geometry and its dependencies.
/// The legacy Clipboard fields remain readable; absence of this payload never
/// manufactures an ICC profile for content whose calibration was not recorded.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct NativeClipboardPayload {
    pub version: u32,
    pub document: tracedraw_core::Document,
    pub source_instance: u64,
}

impl NativeClipboardPayload {
    pub(crate) fn capture(
        source: &tracedraw_core::Document,
        shapes: &[tracedraw_core::Shape],
        source_instance: u64,
    ) -> Result<Self, String> {
        use std::collections::{BTreeSet, VecDeque};
        use tracedraw_core::{Document, Shape};
        tracedraw_core::color_convert::validate_shape_color_trees(shapes)?;
        tracedraw_core::color_convert::validate_document_color_tree(source)?;
        #[derive(Default)]
        struct Dependencies {
            symbols: BTreeSet<usize>,
            styles: BTreeSet<u32>,
            inks: BTreeSet<u32>,
            fonts: BTreeSet<String>,
            pending: VecDeque<(bool, usize)>,
        }
        fn inspect(doc: &Document, deps: &mut Dependencies) -> Result<(), String> {
            tracedraw_core::color_convert::try_visit_document_shape_tree(doc, &mut |shape| {
                if let ShapeKind::SymbolInstance { index } = shape.kind {
                    if deps.symbols.insert(index) {
                        deps.pending.push_back((true, index));
                    }
                }
                if let Some(style) = shape.style {
                    if deps.styles.insert(style) {
                        deps.pending.push_back((false, style as usize));
                    }
                }
                if let Some(inks) = &shape.picture_inks {
                    for ink in &inks.inks {
                        if let tracedraw_core::picture_inks::PictureInk::Spot(id) = ink {
                            deps.inks.insert(*id);
                        }
                    }
                }
                let mut add = |spans: &[TextSpan]| {
                    deps.fonts
                        .extend(spans.iter().map(|span| span.font_family.clone()));
                };
                match &shape.kind {
                    ShapeKind::Text { spans, .. } => add(spans),
                    ShapeKind::Table(table) => table.cells.iter().for_each(|cell| add(&cell.text)),
                    _ => {}
                }
                if let Some(dimension) = crate::dimensions::Dimension::of(shape) {
                    if !dimension.style.font.is_empty() {
                        deps.fonts.insert(dimension.style.font);
                    }
                }
                Ok(())
            })?;
            let mut colors = doc.clone();
            let mut ink_dependencies = |color| {
                if let tracedraw_core::Color::Spot { ink, .. } = color {
                    deps.inks.insert(ink);
                }
                Ok(color)
            };
            tracedraw_core::color_convert::try_map_document_colors(
                &mut colors,
                &mut ink_dependencies,
            )?;
            crate::color_import::try_map_dimension_colors(&mut colors, &mut ink_dependencies)?;
            Ok(())
        }
        let size = source
            .pages
            .first()
            .ok_or("clipboard source has no page")?
            .size;
        let mut document = Document::new("Clipboard", size);
        document.pages[0].layers[0].shapes = shapes.to_vec();
        document.cmyk_profile = source.cmyk_profile.clone();
        document.metadata.cmyk_profile = source
            .cmyk_profile
            .as_ref()
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| "Generic CMYK (open)".into());
        let mut deps = Dependencies::default();
        inspect(&document, &mut deps)?;
        let mut selected_symbols = std::collections::BTreeMap::new();
        let mut selected_styles = std::collections::BTreeMap::new();
        while let Some((symbol, index)) = deps.pending.pop_front() {
            let mut resource = Document::new("Clipboard dependency", size);
            if symbol {
                let value = source
                    .symbols
                    .get(index)
                    .ok_or("clipboard symbol dependency is missing")?
                    .clone();
                resource.pages[0].layers[0].shapes.push(value.shape.clone());
                selected_symbols.insert(index, value);
            } else {
                let id = u32::try_from(index)
                    .map_err(|_| "clipboard style identifier exceeds its range")?;
                let mut matches = source.object_styles.iter().filter(|style| style.id == id);
                let value = matches
                    .next()
                    .ok_or("clipboard object style dependency is missing")?
                    .clone();
                if matches.next().is_some() {
                    return Err("clipboard object style identifier is ambiguous".into());
                }
                let mut shape = Shape::new(
                    ShapeId(0),
                    ShapeKind::Group {
                        children: Vec::new(),
                    },
                );
                shape.fill = value.fill.clone();
                shape.stroke = value.stroke.clone();
                resource.pages[0].layers[0].shapes.push(shape);
                selected_styles.insert(id, value);
            }
            inspect(&resource, &mut deps)?;
        }
        let symbol_map: std::collections::BTreeMap<usize, usize> = selected_symbols
            .keys()
            .enumerate()
            .map(|(new, old)| (*old, new))
            .collect();
        document.symbols = selected_symbols.into_values().collect();
        document.object_styles = selected_styles.into_values().collect();
        for id in deps.inks {
            let mut matches = source.inks.iter().filter(|ink| ink.id == id);
            let ink = matches
                .next()
                .ok_or("clipboard spot ink dependency is missing")?
                .clone();
            if matches.next().is_some() {
                return Err("clipboard spot ink identifier is ambiguous".into());
            }
            document.inks.push(ink);
        }
        document.fonts = source
            .fonts
            .iter()
            .filter(|font| {
                if deps.fonts.is_empty() {
                    return false;
                }
                // DocumentFont.family is a display label, not the font's
                // identity. An isolated database has no installed fonts or
                // fallback families that could retain unrelated font bytes.
                let probe = tracedraw_text::FontSystem::empty();
                probe.add_document_fonts(std::slice::from_ref(*font));
                deps.fonts.iter().any(|family| probe.has_family(family))
            })
            .cloned()
            .collect();
        let mut maximum = 0;
        tracedraw_core::color_convert::try_map_document_shape_tree(&mut document, &mut |shape| {
            maximum = maximum.max(shape.id.raw());
            if let ShapeKind::SymbolInstance { index } = &mut shape.kind {
                *index = *symbol_map
                    .get(index)
                    .ok_or("clipboard symbol closure is incomplete")?;
            }
            Ok(())
        })?;
        document.ids_mut().reserve(maximum);
        crate::app::import_document::validate_resources(&document)?;
        Ok(Self {
            version: 1,
            document,
            source_instance,
        })
    }
}

impl crate::app::Clipboard {
    pub(crate) fn from_native(payload: NativeClipboardPayload) -> Self {
        Self {
            shapes: payload.document.pages[0].layers[0].shapes.clone(),
            styles: payload.document.object_styles.clone(),
            symbols: payload.document.symbols.clone(),
            native: Some(payload),
            system: Default::default(),
            guides: Vec::new(),
        }
    }
}

/// What the system clipboard currently offers.
#[derive(Debug, Clone, Default)]
pub struct ClipboardContent {
    pub text: Option<String>,
    pub image: Option<image::RgbaImage>,
    /// Files copied in a file manager.
    pub files: Vec<std::path::PathBuf>,
}

impl ClipboardContent {
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.image.is_none() && self.files.is_empty()
    }

    /// What a [`Mark`] keeps of this content.
    pub fn mark(&self, seq: Option<u32>) -> Mark {
        Mark {
            seq,
            text: hash_of(&self.text),
            files: hash_of(&self.files),
            image: self.image.as_ref().map(image_sample),
        }
    }

    pub fn is_svg(&self) -> bool {
        self.text.as_deref().map(looks_svg_text).unwrap_or(false)
    }
}

/// Cheap envelope classification only. Full XML/native parsing happens on the
/// import worker, including prefixed roots and damaged recognized transports.
pub(crate) fn looks_svg_text(text: &str) -> bool {
    let original = text.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    if !original.starts_with('<') {
        return false;
    }
    let mut root = original;
    loop {
        let ending = if root.starts_with("<?") {
            Some("?>")
        } else if root.starts_with("<!--") {
            Some("-->")
        } else {
            None
        };
        let Some(ending) = ending else {
            break;
        };
        let Some(end) = root.find(ending) else {
            break;
        };
        root = root[end + ending.len()..].trim_start();
    }
    let name = root
        .strip_prefix('<')
        .unwrap_or("")
        .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .next()
        .unwrap_or("");
    name.rsplit(':').next() == Some("svg")
        || original.contains("<svg")
        || crate::clipboard_wire::looks_native(original)
}

/// What the system clipboard held when Craft Plumilla copied, so that a paste
/// can tell whether something was copied elsewhere since (#183). Where
/// the system counts clipboard changes (Windows) the count decides and
/// nothing is read; elsewhere the text and the files decide, and a
/// picture is decoded and sampled only when there is neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mark {
    /// The system's change count (Windows), `None` where there is none.
    pub seq: Option<u32>,
    text: u64,
    files: u64,
    image: Option<u64>,
}

impl Mark {
    /// Whether the clipboard, now at change count `seq` with `text` and
    /// `files`, still holds what was marked; `image` decodes and samples
    /// its picture, called only when the rest cannot tell.
    pub fn still_holds(
        &self,
        seq: Option<u32>,
        text: &Option<String>,
        files: &[std::path::PathBuf],
        image: impl FnOnce() -> Option<u64>,
    ) -> bool {
        if let (Some(then), Some(now)) = (self.seq, seq) {
            return then == now;
        }
        if hash_of(text) != self.text || hash_of(&files) != self.files {
            return false;
        }
        // Equal text or files: the same copy (a picture copied elsewhere
        // replaces them).
        if text.is_some() || !files.is_empty() {
            return true;
        }
        image() == self.image
    }
}

fn hash_of<T: std::hash::Hash + ?Sized>(v: &T) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

/// A picture's size and about 4096 of its bytes spread over it: enough to
/// tell two copied pictures apart without hashing 48 MB.
fn image_sample(img: &image::RgbaImage) -> u64 {
    use std::hash::{Hash, Hasher};
    let raw = img.as_raw();
    let step = (raw.len() / 4096).max(1);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (img.width(), img.height()).hash(&mut h);
    for b in raw.iter().step_by(step) {
        b.hash(&mut h);
    }
    h.finish()
}

/// The system's clipboard change count: Windows counts every change, so
/// a paste can tell nothing changed without reading the clipboard.
#[cfg(all(windows, not(test)))]
pub fn change_count() -> Option<u32> {
    #[link(name = "user32", kind = "raw-dylib")]
    unsafe extern "system" {
        fn GetClipboardSequenceNumber() -> u32;
    }
    // SAFETY: no arguments, it only returns a number (0 without access to
    // the window station).
    let n = unsafe { GetClipboardSequenceNumber() };
    (n != 0).then_some(n)
}

// Unit tests inject counters and clipboard data through paste_with; they
// never query the user's clipboard sequence or open its native adapter.
#[cfg(any(not(windows), test))]
pub fn change_count() -> Option<u32> {
    None
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn read_system_text_and_files() -> ClipboardContent {
    read_text_and_files()
}

#[cfg(target_arch = "wasm32")]
fn read_system_text_and_files() -> ClipboardContent {
    ClipboardContent::default()
}

/// The system clipboard's text and files on Android, through the
/// activity's clipboard service: documents other applications copied
/// (content URIs) are named like the ones the document picker returns,
/// so pasting imports them; a lone picture is the clipboard's picture
/// ([`read_system_image`]), placed as a bitmap (#1403).
#[cfg(target_os = "android")]
fn read_system_text_and_files() -> ClipboardContent {
    let (files, _) = split_documents(crate::android::clipboard::documents());
    ClipboardContent {
        text: crate::android::clipboard::text(),
        image: None,
        files,
    }
}

/// A document on the clipboard (Android): its content URI, MIME type and
/// shown name.
#[cfg(any(target_os = "android", test))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipDocument {
    pub uri: String,
    pub mime: String,
    pub name: String,
}

/// The documents `getClipboardDocuments` lists (Android): one per line,
/// "uri TAB mime TAB name".
#[cfg(any(target_os = "android", test))]
pub fn parse_clip_documents(lines: &str) -> Vec<ClipDocument> {
    lines
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let uri = parts.next()?.trim();
            if uri.is_empty() {
                return None;
            }
            Some(ClipDocument {
                uri: uri.to_string(),
                mime: parts.next().unwrap_or("").trim().to_string(),
                name: parts.next().unwrap_or("").trim().to_string(),
            })
        })
        .collect()
}

/// The clipboard's documents as files to import, or the URI of its
/// picture when it holds one picture alone (copied from a gallery, a
/// browser or a chat: placed as a bitmap like a picture copied on the
/// desktop). Several documents are all imported.
#[cfg(any(target_os = "android", test))]
fn split_documents(docs: Vec<ClipDocument>) -> (Vec<std::path::PathBuf>, Option<String>) {
    if let [doc] = docs.as_slice() {
        if doc.mime.starts_with("image/") {
            return (Vec::new(), Some(doc.uri.clone()));
        }
    }
    let files = docs
        .iter()
        .map(|d| crate::files::document_path(&d.uri, &d.name))
        .collect();
    (files, None)
}

/// The system clipboard's text, read where the system lets an
/// application read it (not in a browser, which hands it only to paste
/// events).
pub fn system_text() -> Option<String> {
    read_system_text_and_files().text
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn read_system_image() -> Option<image::RgbaImage> {
    read_image()
}

#[cfg(target_arch = "wasm32")]
fn read_system_image() -> Option<image::RgbaImage> {
    None
}

/// The picture another application copied (Android): one image document
/// alone on the clipboard, decoded.
#[cfg(target_os = "android")]
fn read_system_image() -> Option<image::RgbaImage> {
    let (_, uri) = split_documents(crate::android::clipboard::documents());
    let bytes = crate::android::documents::read(&uri?).ok()?;
    match image::load_from_memory(&bytes) {
        Ok(img) => Some(img.into_rgba8()),
        Err(e) => {
            log::warn!("the copied picture cannot be read: {e}");
            None
        }
    }
}

/// The system clipboard as seen by the browser build: empty, because
/// browsers hand clipboard contents only to paste events and, in the
/// background, to a clipboard read ([`Handed`]).
#[cfg(target_arch = "wasm32")]
pub fn read_system() -> ClipboardContent {
    ClipboardContent::default()
}

/// The system clipboard on Android: its text, documents and picture
/// (`android.rs`).
#[cfg(target_os = "android")]
pub fn read_system() -> ClipboardContent {
    let mut content = read_system_text_and_files();
    content.image = read_system_image();
    content
}

/// Read text, files and image from the system clipboard; failures yield
/// `None` fields (headless sessions have no clipboard).
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn read_system() -> ClipboardContent {
    let mut content = read_text_and_files();
    content.image = read_image();
    content
}

thread_local! {
    /// The one connection to the system clipboard, kept for the session
    /// (#1488): a new one for each read cost a new connection to the X
    /// server each time.
    #[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), not(test)))]
    static SYSTEM_CLIPBOARD: std::cell::RefCell<Option<arboard::Clipboard>> =
        const { std::cell::RefCell::new(None) };
}

/// Run `f` with the system clipboard, opened on first use; `None` when it
/// cannot be opened (headless sessions have no clipboard).
#[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), not(test)))]
fn with_clipboard<T>(f: impl FnOnce(&mut arboard::Clipboard) -> T) -> Option<T> {
    SYSTEM_CLIPBOARD.with(|cell| {
        let mut cb = cell.try_borrow_mut().ok()?;
        if cb.is_none() {
            *cb = arboard::Clipboard::new().ok();
        }
        cb.as_mut().map(f)
    })
}

/// Whether the clipboard offers text and a list of files, where
/// Craft Plumilla reads its list of types before the clipboard library reads
/// it (an X11 desktop: no `WAYLAND_DISPLAY`); `None` elsewhere, where the
/// library is asked for both.
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn x11_offers_text_and_files() -> Option<(bool, bool)> {
    #[cfg(all(unix, not(target_os = "macos"), not(test)))]
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return crate::x11_clipboard::offer().map(|o| (o.has_text(), o.has_files()));
    }
    None
}

/// Whether a paste shortcut may read the clipboard's text: asked by the
/// windowing layer before each one (`eframe::set_paste_text_gate`). On an
/// X11 desktop only when the clipboard offers text: a program that holds
/// a picture alone may answer a text request with the picture sent in
/// parts, which the text read stops after the first part, and then wait
/// for the rest and answer nobody, so the picture never pasted and the
/// window froze for the reads that waited (#1488).
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn paste_may_read_text() -> bool {
    x11_offers_text_and_files().is_none_or(|(text, _)| text)
}

/// The clipboard's text and files, not its picture (decoding one costs
/// far more). On an X11 desktop each is asked for only when the
/// clipboard offers it (#1488, [`paste_may_read_text`]).
#[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), not(test)))]
fn read_text_and_files() -> ClipboardContent {
    let (text_offered, files_offered) = x11_offers_text_and_files().unwrap_or((true, true));
    if !text_offered && !files_offered {
        return ClipboardContent::default();
    }
    with_clipboard(|cb| {
        let text = text_offered
            .then(|| cb.get_text().ok())
            .flatten()
            .filter(|t| !t.is_empty());
        let files = if files_offered {
            cb.get()
                .file_list()
                .unwrap_or_default()
                .into_iter()
                .map(without_line_end)
                .collect()
        } else {
            Vec::new()
        };
        ClipboardContent {
            text,
            image: None,
            files,
        }
    })
    .unwrap_or_default()
}

#[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), test))]
fn read_text_and_files() -> ClipboardContent {
    TEST_INPUT_CLIPBOARD.with(|slot| {
        let mut content = slot.borrow().clone().unwrap_or_default();
        content.image = None;
        content
    })
}

#[cfg(test)]
thread_local! {
    static TEST_INPUT_CLIPBOARD: std::cell::RefCell<Option<ClipboardContent>> = const { std::cell::RefCell::new(None) };
}

/// Exercise real paste handlers with isolated input, never the OS clipboard.
#[cfg(test)]
pub(crate) fn with_test_external_clipboard<R>(
    content: ClipboardContent,
    f: impl FnOnce() -> R,
) -> R {
    struct Restore(Option<ClipboardContent>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_INPUT_CLIPBOARD.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }
    let previous = TEST_INPUT_CLIPBOARD.with(|slot| slot.replace(Some(content)));
    let _restore = Restore(previous);
    f()
}

/// The clipboard's picture, decoded. On an X11 desktop the clipboard's
/// types are read and its best picture type is asked for, with time for
/// a busy program that sends a large picture in parts (#1488, see
/// `x11_clipboard.rs`); elsewhere (Windows, macOS, a Wayland desktop),
/// and when that finds nothing, the clipboard library reads its PNG.
#[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), not(test)))]
fn read_image() -> Option<image::RgbaImage> {
    #[cfg(all(unix, not(target_os = "macos")))]
    let x11_first = std::env::var_os("WAYLAND_DISPLAY").is_none();
    #[cfg(all(unix, not(target_os = "macos")))]
    if x11_first {
        if let Some(img) = crate::x11_clipboard::read_image() {
            return Some(img);
        }
    }
    let from_library = with_clipboard(|cb| cb.get_image().ok())
        .flatten()
        .and_then(|img| {
            image::RgbaImage::from_raw(img.width as u32, img.height as u32, img.bytes.into_owned())
        });
    // A Wayland desktop whose clipboard the library cannot read: the
    // X11 one the desktop keeps beside it.
    #[cfg(all(unix, not(target_os = "macos")))]
    if from_library.is_none() && !x11_first {
        return crate::x11_clipboard::read_image();
    }
    from_library
}

#[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), test))]
fn read_image() -> Option<image::RgbaImage> {
    TEST_INPUT_CLIPBOARD.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|content| content.image.clone())
    })
}

/// A path from a copied file list without the carriage return its line
/// may keep: the list's lines end in CR LF and the clipboard library
/// splits it at LF only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn without_line_end(path: std::path::PathBuf) -> std::path::PathBuf {
    match path.to_str() {
        Some(s) if s.ends_with(['\r', '\n']) => s.trim_end_matches(['\r', '\n']).into(),
        _ => path,
    }
}

/// Put text on the system clipboard (best effort); whether it went there.
#[cfg(all(not(any(target_arch = "wasm32", target_os = "android")), not(test)))]
pub fn write_text(text: &str) -> bool {
    with_clipboard(|cb| cb.set_text(text.to_string()).is_ok()).unwrap_or(false)
}

#[cfg(test)]
thread_local! {
    /// Existing command tests publish into an isolated per-thread sink. Tests
    /// of rejection/partial writes inject their publisher explicitly instead.
    static TEST_PUBLISHED_TEXT: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub fn write_text(text: &str) -> bool {
    TEST_PUBLISHED_TEXT.with(|slot| *slot.borrow_mut() = Some(text.to_string()));
    true
}

/// Put text on the system clipboard through the activity's clipboard
/// service (Android); whether it went there.
#[cfg(all(target_os = "android", not(test)))]
pub fn write_text(text: &str) -> bool {
    crate::android::clipboard::set_text(text)
}

/// Put text on the system clipboard through egui (browser build).
#[cfg(all(target_arch = "wasm32", not(test)))]
pub fn write_text(text: &str) -> bool {
    crate::web::copy_text(text);
    true
}

/// A file a browser handed over for a paste: a picture or a file copied
/// in another application (#1403).
#[derive(Debug, Clone, Default)]
pub struct PastedFile {
    pub name: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

/// What a browser handed over for one paste, read in the background: the
/// files of a paste event (Ctrl+V), or what the clipboard API read for
/// Edit > Paste (#1403).
#[derive(Debug, Clone, Default)]
pub struct Handed {
    pub text: Option<String>,
    pub files: Vec<PastedFile>,
    /// From Edit > Paste (a clipboard read), not from a paste event: its
    /// text may be the copy made here, which then pastes as objects.
    pub from_menu: bool,
    /// The browser did not let the clipboard be read (Edit > Paste).
    pub refused: bool,
    /// Where the pasted objects go (the page's context menu), page space.
    pub at: Option<Point>,
}

thread_local! {
    /// Pastes handed over since the last frame.
    static HANDED: std::cell::RefCell<Vec<Handed>> = const { std::cell::RefCell::new(Vec::new()) };
    /// Pastes being read in the background.
    static AWAITED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// A paste is being read in the background: the paste shortcut of this
/// frame must not paste the copy here as well (`App::paste_with_event_text`).
#[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
pub fn expect_handed() {
    AWAITED.with(|a| a.set(a.get() + 1));
}

/// A paste read in the background is ready for the next frame. Only the
/// files of a paste event were awaited ([`expect_handed`]): Edit > Paste
/// never holds the paste shortcut back, as the browser may wait for the
/// user to allow the clipboard to be read.
#[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
pub fn hand_over(handed: Handed) {
    if !handed.from_menu {
        AWAITED.with(|a| a.set(a.get().saturating_sub(1)));
    }
    HANDED.with(|h| h.borrow_mut().push(handed));
}

/// Whether a paste is still being read in the background.
pub fn handed_awaited() -> bool {
    AWAITED.with(|a| a.get() > 0)
}

/// The image file among `files` when it is the only file and decodes: a
/// picture copied in another application, placed as a bitmap.
fn lone_picture(files: &[PastedFile]) -> Option<image::RgbaImage> {
    let [file] = files else {
        return None;
    };
    if !file.mime.starts_with("image/") {
        return None;
    }
    match image::load_from_memory(&file.bytes) {
        Ok(img) => Some(img.into_rgba8()),
        Err(e) => {
            log::warn!("the pasted picture cannot be read: {e}");
            None
        }
    }
}

struct PreparedClipboardCopy {
    clipboard: crate::app::Clipboard,
    transport: String,
}

#[derive(Clone)]
struct CopyStamp {
    instance: u64,
    edit_version: u64,
    page: tracedraw_core::PageId,
    selection: Vec<ShapeId>,
    color: crate::settings::ColorPrefs,
    text: Option<(ShapeId, usize, usize)>,
}

impl CopyStamp {
    fn capture(app: &App) -> Self {
        Self {
            instance: app.engine.instance(),
            edit_version: app.engine.revision(),
            page: app.page,
            selection: app.selection.clone(),
            color: app.settings.color.clone(),
            text: app
                .text_edit
                .as_ref()
                .map(|edit| (edit.shape, edit.caret, edit.anchor)),
        }
    }

    fn matches(&self, app: &App) -> bool {
        self.instance == app.engine.instance()
            && self.edit_version == app.engine.revision()
            && self.page == app.page
            && self.selection == app.selection
            && self.color == app.settings.color
            && self.text
                == app
                    .text_edit
                    .as_ref()
                    .map(|edit| (edit.shape, edit.caret, edit.anchor))
    }
}

pub(crate) struct ClipboardCopyJob {
    receiver: std::sync::mpsc::Receiver<Result<PreparedClipboardCopy, String>>,
    stamp: CopyStamp,
    cut: bool,
    sequence: Option<u32>,
}

fn prepare_native_copy(
    document: &tracedraw_core::Document,
    page: tracedraw_core::PageId,
    shapes: &[tracedraw_core::Shape],
    instance: u64,
    color: &crate::settings::ColorPrefs,
) -> Result<PreparedClipboardCopy, String> {
    let native = crate::app::import_document::native_payload_from_document(
        document, page, shapes, instance, color,
    )?;
    let svg = crate::clipboard_preview::render(&native, color)?;
    let transport = crate::clipboard_wire::encode_svg(&native, &svg)?;
    let mut clipboard = crate::app::Clipboard::from_native(native);
    clipboard.system = ClipboardContent {
        text: Some(transport.clone()),
        ..Default::default()
    }
    .mark(None);
    Ok(PreparedClipboardCopy {
        clipboard,
        transport,
    })
}

fn prepare_native_copy_on_stack(
    document: &tracedraw_core::Document,
    page: tracedraw_core::PageId,
    shapes: &[tracedraw_core::Shape],
    instance: u64,
    color: &crate::settings::ColorPrefs,
) -> Result<PreparedClipboardCopy, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("clipboard-copy-sync".into())
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    prepare_native_copy(document, page, shapes, instance, color)
                })
                .map_err(|error| error.to_string())?
                .join()
                .map_err(|_| "clipboard copy preparation failed unexpectedly".to_string())?
        })
    }
    #[cfg(target_arch = "wasm32")]
    {
        prepare_native_copy(document, page, shapes, instance, color)
    }
}

impl App {
    #[cfg(test)]
    pub(crate) fn start_clipboard_copy(&mut self, cut: bool) -> Result<bool, String> {
        self.start_clipboard_copy_with_sequence(cut, None)
    }

    pub(crate) fn start_system_clipboard_copy(&mut self, cut: bool) -> Result<bool, String> {
        let sequence = change_count();
        #[cfg(all(windows, not(test)))]
        if sequence.is_none() {
            return Err("clipboard sequence is unavailable".into());
        }
        self.start_clipboard_copy_with_sequence(cut, sequence)
    }

    pub(crate) fn start_clipboard_copy_with_sequence(
        &mut self,
        cut: bool,
        sequence: Option<u32>,
    ) -> Result<bool, String> {
        if self.clipboard_copy_job.is_some() {
            return Err("another clipboard preparation is still running".into());
        }
        tracedraw_core::color_convert::validate_document_color_tree(self.doc())?;
        let began = web_time::Instant::now();
        let shapes = self.selected_shapes();
        if shapes.is_empty() {
            return Ok(false);
        }
        let document = self.doc().clone();
        let stamp = CopyStamp::capture(self);
        let worker_stamp = stamp.clone();
        let context = self.color_import_context.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::Builder::new()
            .name("clipboard-copy".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || {
                let prepared = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    prepare_native_copy(
                        &document,
                        worker_stamp.page,
                        &shapes,
                        worker_stamp.instance,
                        &worker_stamp.color,
                    )
                }))
                .unwrap_or_else(|_| Err("clipboard preparation failed unexpectedly".into()));
                let _ = sender.send(prepared);
                if let Some(context) = context {
                    context.request_repaint();
                }
            })
            .map_err(|error| error.to_string())?;
        #[cfg(target_arch = "wasm32")]
        {
            // Browser execution still lacks a worker transport and publication
            // acknowledgement; this synchronous fallback makes no frame claim.
            let _ = sender.send(prepare_native_copy(
                &document,
                worker_stamp.page,
                &shapes,
                worker_stamp.instance,
                &worker_stamp.color,
            ));
            if let Some(context) = context {
                context.request_repaint();
            }
        }
        self.clipboard_copy_job = Some(ClipboardCopyJob {
            receiver,
            stamp,
            cut,
            sequence,
        });
        self.status = tr(if cut {
            "status.cut_preparing"
        } else {
            "status.copy_preparing"
        });
        log::info!(
            "clipboard copy snapshot took {} ms",
            began.elapsed().as_millis()
        );
        Ok(true)
    }

    pub(crate) fn cancel_clipboard_copy(&mut self) {
        // Dropping the receiver discards any late result on its worker stack.
        self.clipboard_copy_job = None;
    }

    #[cfg(test)]
    pub(crate) fn poll_clipboard_copy_with_publisher(
        &mut self,
        publish: impl FnOnce(&str) -> Result<(), String>,
    ) -> Result<bool, String> {
        self.poll_clipboard_copy_with_sequence_and_publisher(None, publish)
    }

    pub(crate) fn poll_clipboard_copy_with_sequence_and_publisher(
        &mut self,
        sequence: Option<u32>,
        publish: impl FnOnce(&str) -> Result<(), String>,
    ) -> Result<bool, String> {
        let Some(job) = self.clipboard_copy_job.take() else {
            return Ok(false);
        };
        match job.receiver.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                self.clipboard_copy_job = Some(job);
                Ok(false)
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Err("clipboard preparation stopped without a result".into())
            }
            Ok(result) => {
                if !job.stamp.matches(self) {
                    return Err("the clipboard source changed during preparation".into());
                }
                if job.sequence != sequence {
                    return Err("the system clipboard changed during preparation".into());
                }
                let prepared = result?;
                publish(&prepared.transport)?;
                self.clipboard = Some(prepared.clipboard);
                self.text_clip = None;
                self.status = trf(
                    "status.objects_copied",
                    &[("n", &job.stamp.selection.len().to_string())],
                );
                if job.cut {
                    self.delete_copied_selection();
                }
                Ok(true)
            }
        }
    }

    pub(crate) fn poll_clipboard_copy(&mut self) {
        let Some(job) = self.clipboard_copy_job.as_ref() else {
            return;
        };
        // Headless/MCP callers consume jobs through the injected publisher.
        // This adapter belongs only to the graphical application.
        if self.color_import_context.is_none() {
            return;
        }
        let tracks_sequence = job.sequence.is_some();
        let sequence = tracks_sequence.then(change_count).flatten();
        match self.poll_clipboard_copy_with_sequence_and_publisher(sequence, |text| {
            write_text(text)
                .then_some(())
                .ok_or_else(|| "clipboard publication failed".to_string())
        }) {
            Ok(true) => {
                if let Some(clipboard) = &mut self.clipboard {
                    clipboard.system.seq = tracks_sequence.then(change_count).flatten();
                }
            }
            Ok(false) => {}
            Err(error) => self.status = trf("status.clipboard_failed", &[("e", &error)]),
        }
    }
    /// The selection as a standalone SVG document (page-sized viewport so
    /// positions survive a round trip).
    pub fn selection_svg(&self) -> Option<String> {
        tracedraw_core::color_convert::validate_document_color_tree(self.doc()).ok()?;
        let shapes = self.selected_shapes();
        if shapes.is_empty() {
            return None;
        }
        let native = self.native_payload_for_selection(&shapes).ok()?;
        self.clipboard_svg(&native).ok()
    }

    fn clipboard_svg(&self, native: &NativeClipboardPayload) -> Result<String, String> {
        crate::clipboard_preview::render(native, &self.settings.color)
    }

    /// Prepare the complete byte transport before publishing, then commit the
    /// matching in-memory copy. The injected publisher never reads the OS.
    pub(crate) fn try_copy_with_publisher(
        &mut self,
        publish: impl FnOnce(&str) -> Result<(), String>,
    ) -> Result<bool, String> {
        tracedraw_core::color_convert::validate_document_color_tree(self.doc())?;
        if self.editing_text() {
            return self.try_text_copy_with_publisher(publish);
        }
        let shapes = self.selected_shapes();
        if shapes.is_empty() {
            return Ok(false);
        }
        let prepared = prepare_native_copy_on_stack(
            self.doc(),
            self.page,
            &shapes,
            self.engine.instance(),
            &self.settings.color,
        )?;
        publish(&prepared.transport)?;
        self.clipboard = Some(prepared.clipboard);
        self.text_clip = None;
        self.status = trf(
            "status.objects_copied",
            &[("n", &self.selection.len().to_string())],
        );
        Ok(true)
    }

    /// A rejected/partial publication cannot delete the selected source data.
    pub(crate) fn try_cut_with_publisher(
        &mut self,
        publish: impl FnOnce(&str) -> Result<(), String>,
    ) -> Result<bool, String> {
        if self.editing_text() {
            return self.try_text_cut_with_publisher(publish);
        }
        if !self.try_copy_with_publisher(publish)? {
            return Ok(false);
        }
        self.delete_copied_selection();
        Ok(true)
    }

    fn delete_copied_selection(&mut self) {
        let selected = self.selection.clone();
        let revision = self.engine.revision();
        self.delete_selection();
        if self.engine.revision() != revision {
            if let Some(step) = self.engine.last_step_id() {
                // Keep the selection on the deletion's existing step, including
                // its linked-text amendments, without creating another step.
                self.import_selections.insert(
                    (self.engine.instance(), step),
                    (selected, self.selection.clone()),
                );
            }
        }
    }

    /// Copy to the internal clipboard and, as SVG, to the system clipboard.
    pub fn copy_with_system(&mut self) {
        // Guidelines selected and no objects: they are copied (#376).
        if !self.editing_text() && self.selection.is_empty() && self.copy_selected_guides() {
            return;
        }
        if !self.editing_text() && self.color_import_context.is_some() && !crate::files::WEB {
            if let Err(error) = self.start_system_clipboard_copy(false) {
                self.status = trf("status.clipboard_failed", &[("e", &error)]);
            }
            return;
        }
        match self.try_copy_with_publisher(|text| {
            write_text(text)
                .then_some(())
                .ok_or_else(|| "clipboard publication failed".to_string())
        }) {
            Ok(true) => {
                if let Some(clipboard) = &mut self.clipboard {
                    clipboard.system.seq = change_count();
                }
            }
            Ok(false) => {}
            Err(error) => {
                self.status = trf("status.import_failed", &[("e", &error)]);
            }
        }
    }

    /// Remember what the system clipboard holds right after a copy here
    /// (`written`: the text the copy put there), so that a paste can tell
    /// whether something was copied elsewhere since. What was written is
    /// what the clipboard holds (writing text replaces everything there),
    /// so it is not read back; a copy that wrote nothing (nodes, guides)
    /// reads it, its picture only where the system keeps no change count
    /// and there is no text or file (#183).
    pub fn mark_system_clipboard(&mut self, written: Option<&str>) {
        let seq = change_count();
        let mark = match written {
            Some(text) => ClipboardContent {
                text: Some(text.to_string()),
                ..Default::default()
            }
            .mark(seq),
            None if seq.is_some() => Mark {
                seq,
                ..Default::default()
            },
            None => {
                let mut now = read_system_text_and_files();
                if now.text.is_none() && now.files.is_empty() {
                    now.image = read_system_image();
                }
                now.mark(seq)
            }
        };
        if let Some(cb) = self.clipboard.as_mut() {
            cb.system = mark;
        }
    }

    /// Paste the system clipboard's text as artistic text at the page
    /// centre. Characters copied while editing a text, while the
    /// clipboard still holds them, keep their formatting and their
    /// text's fill and outline (#1378).
    pub fn paste_as_text(&mut self, text: &str) {
        let c = self.page_rect().center();
        let clip = self
            .text_clip
            .as_ref()
            .filter(|clip| clip.plain == crate::text_editing::unify_line_ends(text))
            .cloned();
        let (spans, fill, stroke) = match clip {
            Some(clip) => (
                clip.spans
                    .into_iter()
                    .filter(|sp| !sp.text.is_empty())
                    .collect(),
                clip.fill,
                clip.stroke,
            ),
            None => (
                vec![TextSpan {
                    bold: self.text_bold,
                    italic: self.text_italic,
                    ..TextSpan::new(text, self.text_font.clone(), self.text_size_pt)
                }],
                tracedraw_core::Fill::Solid(tracedraw_core::Color::BLACK),
                None,
            ),
        };
        let kind = ShapeKind::Text {
            spans,
            origin: Point::new(c.x, c.y),
            frame: None,
            align: tracedraw_core::TextAlign::Left,
            para: Default::default(),
            on_path: None,
        };
        if let Some(id) = self.new_shape(kind) {
            self.run(Command::SetFill {
                shapes: vec![id],
                fill,
            });
            self.run(Command::SetStroke {
                shapes: vec![id],
                stroke,
            });
            self.select(vec![id]);
        }
    }

    /// Paste SVG text from the system clipboard as objects.
    pub fn paste_as_svg(&mut self, svg: &str) {
        if let Err(error) = self.try_paste_svg_text(svg) {
            self.status = trf("status.import_failed", &[("e", &error)]);
        }
    }

    /// Consume actual incoming bytes, not a shared in-memory clipboard. A
    /// recognized native error is terminal and must never fall back to SVG.
    pub(crate) fn try_paste_svg_text(&mut self, svg: &str) -> Result<(), String> {
        if svg.len() > crate::clipboard_wire::MAX_SVG_BYTES {
            return Err("clipboard transport exceeds its byte budget".into());
        }
        if self.editing_text() {
            let target = crate::text_editing::InlinePasteTarget::capture(self)
                .ok_or_else(|| "clipboard text target is unavailable".to_string())?;
            return self.dispatch_inline_wire_paste(svg.to_string(), target);
        }
        self.dispatch_wire_paste(svg.to_string())
    }

    /// Paste the system clipboard's image as a bitmap object.
    pub fn paste_as_bitmap(&mut self, img: image::RgbaImage) {
        if let Some(id) = self.place_image(img, Some(tr("kind.bitmap_short"))) {
            self.select(vec![id]);
        }
    }

    /// Edit > Paste: what was copied last, here or in another
    /// application.
    pub fn paste_any(&mut self) {
        self.paste_any_at(None);
    }

    /// Edit > Paste with the pasted objects centred on `at` (the page's
    /// context menu), when given. In a browser the clipboard is read in
    /// the background (its API asks the user the first time) and the
    /// paste happens on a later frame ([`App::take_handed_pastes`]);
    /// where it cannot be read, the copy here is pasted (#1403).
    pub fn paste_any_at(&mut self, at: Option<Point>) {
        #[cfg(target_arch = "wasm32")]
        if crate::web::read_clipboard(Handed {
            from_menu: true,
            at,
            ..Default::default()
        }) {
            return;
        }
        self.paste_with_event_text_at(None, at);
    }

    /// Centre what a paste selected on `at`, when given and the paste
    /// selected something new.
    fn move_pasted_to(&mut self, before: &[ShapeId], at: Option<Point>) {
        let Some(at) = at else {
            return;
        };
        if self.selection.is_empty() || self.selection == before {
            return;
        }
        if let Some(b) = self.selection_bounds() {
            let shapes = self.selection.clone();
            self.run(Command::TransformShapes {
                shapes,
                transform: tracedraw_core::Affine::translate(at - b.center()),
            });
        }
    }

    /// The pastes a browser handed over since the last frame (#1403):
    /// pictures and files of a paste event, and what Edit > Paste read
    /// from the clipboard. Called once per frame.
    pub fn take_handed_pastes(&mut self) {
        let handed: Vec<Handed> = HANDED.with(|h| std::mem::take(&mut *h.borrow_mut()));
        for h in handed {
            self.paste_handed(h);
        }
    }

    /// Paste what a browser handed over: a picture alone as a bitmap,
    /// other files imported, text as for a paste here.
    pub fn paste_handed(&mut self, h: Handed) {
        let before_revision = self.engine.revision();
        if self.editing_text() {
            // Into the text at the caret; pictures do not go into text.
            if h.from_menu {
                self.text_paste_text(h.text.filter(|t| !t.is_empty()));
            }
            return;
        }
        if h.refused {
            self.status = tr("status.clipboard_refused");
        }
        let image = lone_picture(&h.files);
        let mut stored = Vec::new();
        if image.is_none() {
            for (i, f) in h.files.iter().enumerate() {
                let name = if f.name.trim().is_empty() {
                    format!("pasted-{i}")
                } else {
                    f.name.replace(['/', '\\'], "_")
                };
                let path = std::path::PathBuf::from("/upload/pasted").join(name);
                crate::files::store(&path, f.bytes.clone());
                stored.push(path);
            }
        }
        let text = h.text.filter(|t| !t.is_empty());
        if h.from_menu {
            // The clipboard may still hold the copy made here.
            self.paste_with_at(
                None,
                || ClipboardContent {
                    text,
                    image: None,
                    files: stored.clone(),
                },
                || image,
                h.at,
            );
        } else {
            self.paste_from_unmarked_at(
                ClipboardContent {
                    text,
                    image,
                    files: stored.clone(),
                },
                h.at,
            );
        }
        for path in &stored {
            crate::files::forget(path);
        }
        if h.refused && self.engine.revision() != before_revision {
            self.status = tr("status.clipboard_refused");
        }
    }

    /// Ctrl+V, with the text of the paste event when there was one (a
    /// browser offers the clipboard's text only that way).
    pub fn paste_with_event_text(&mut self, text: Option<String>) {
        self.paste_with_event_text_at(text, None);
    }

    fn paste_with_event_text_at(&mut self, text: Option<String>, at: Option<Point>) {
        // While a text is being edited: into the text at the caret
        // (#1377).
        if self.editing_text() {
            self.text_paste(text);
            return;
        }
        // Pictures or files of the paste event are being read (browser):
        // they are pasted when they arrive, not the copy here (#1403).
        if handed_awaited() {
            return;
        }
        if crate::files::WEB {
            let content = ClipboardContent {
                text: text.filter(|t| !t.is_empty()),
                ..Default::default()
            };
            self.paste_with_at(None, || content, || None, at);
        } else {
            self.paste_with_at(
                change_count(),
                read_system_text_and_files,
                read_system_image,
                at,
            );
        }
    }

    /// Paste with the clipboard at change count `seq`: `read` gives its
    /// text and files, `read_image` decodes its picture, called only when
    /// the change check needs it or it is what gets pasted (no files, no
    /// SVG), and at most once (#183).
    #[cfg(test)]
    fn paste_with(
        &mut self,
        seq: Option<u32>,
        read: impl FnOnce() -> ClipboardContent,
        read_image: impl FnOnce() -> Option<image::RgbaImage>,
    ) {
        self.paste_with_at(seq, read, read_image, None);
    }

    fn paste_with_at(
        &mut self,
        seq: Option<u32>,
        read: impl FnOnce() -> ClipboardContent,
        read_image: impl FnOnce() -> Option<image::RgbaImage>,
        at: Option<Point>,
    ) {
        if self.editing_text() {
            // Character editing always consumes the incoming text channel,
            // independently of the object clipboard's sequence or image.
            self.text_paste_text(read().text.filter(|text| !text.is_empty()));
            return;
        }
        // Nothing copied anywhere since the copy here: nothing to read.
        if self
            .clipboard
            .as_ref()
            .is_some_and(|cb| seq.is_some() && cb.system.seq == seq)
        {
            self.paste_at(at);
            return;
        }
        let mut content = read();
        let mut read_image = Some(read_image);
        let mut take_image = || read_image.take().and_then(|f| f());
        if let Some(cb) = &self.clipboard {
            let held = cb
                .system
                .still_holds(seq, &content.text, &content.files, || {
                    content.image = take_image();
                    content.image.as_ref().map(image_sample)
                });
            if held {
                self.paste_at(at);
                return;
            }
        }
        if content.image.is_none() && content.files.is_empty() && !content.is_svg() {
            content.image = take_image();
        }
        self.paste_from_unmarked_at(content, at);
    }

    /// [`App::paste_with`] on `content` (the tests' clipboard).
    #[cfg(test)]
    pub fn paste_from(&mut self, content: ClipboardContent) {
        let ClipboardContent { text, image, files } = content;
        self.paste_with(
            None,
            || ClipboardContent {
                text,
                image: None,
                files,
            },
            || image,
        );
    }

    /// What the system clipboard offers, or the copy here when it offers
    /// nothing.
    fn paste_from_unmarked_at(&mut self, content: ClipboardContent, at: Option<Point>) {
        if content.is_empty() && self.clipboard.is_some() {
            self.paste_at(at);
            return;
        }
        // Characters copied while editing a text, where the clipboard
        // cannot be read (a browser's menu Paste): a text of them.
        if content.is_empty() {
            let before = self.selection.clone();
            if let Some(plain) = self.text_clip.as_ref().map(|c| c.plain.clone()) {
                self.paste_as_text(&plain);
                self.move_pasted_to(&before, at);
            }
            return;
        }
        if !content.files.is_empty() {
            for path in &content.files {
                self.import_path_placed(path, at);
            }
            return;
        }
        if content.is_svg() {
            if let Some(t) = content.text {
                if let Err(error) = self.dispatch_wire_paste_at(t, at) {
                    self.status = trf("status.import_failed", &[("e", &error)]);
                }
            }
        } else if let Some(img) = content.image {
            let before = self.selection.clone();
            self.paste_as_bitmap(img);
            self.move_pasted_to(&before, at);
        } else if let Some(t) = content.text {
            let before = self.selection.clone();
            self.paste_as_text(&t);
            self.move_pasted_to(&before, at);
        }
    }

    // ----- external bitmap editing -------------------------------------------------

    fn bitmap_link(&self, id: ShapeId) -> Option<String> {
        self.doc()
            .find_shape(id)?
            .data
            .iter()
            .find(|(k, _)| k == "bitmap.link")
            .map(|(_, v)| v.clone())
    }

    pub fn selected_bitmap(&self) -> Option<ShapeId> {
        let id = *self.selection.first()?;
        let s = self.doc().find_shape(id)?;
        matches!(s.kind, ShapeKind::Bitmap { .. }).then_some(id)
    }

    pub fn selected_bitmap_is_linked(&self) -> bool {
        self.selected_bitmap()
            .and_then(|id| self.bitmap_link(id))
            .is_some()
    }

    /// The selected bitmap's PNG, its effects included.
    #[cfg(test)]
    pub fn selected_bitmap_png(&self) -> Option<tracedraw_core::SharedBytes> {
        let id = self.selected_bitmap()?;
        match &self.doc().find_shape(id)?.kind {
            ShapeKind::Bitmap { png, .. } => Some(png.clone()),
            _ => None,
        }
    }

    /// Save Bitmap As (the bitmap's context menu): the picture as PNG,
    /// JPEG or TIFF (#1834), the kind of the file it came from offered
    /// first. A CMYK picture saved as JPEG or TIFF keeps its inks; see
    /// [`crate::bitmap_file`].
    pub fn save_bitmap_as(&mut self) {
        let Some(id) = self.selected_bitmap() else {
            return;
        };
        let Some(own) = self.doc().find_shape(id).map(crate::bitmap_file::own_kind) else {
            return;
        };
        let name = format!("{}.{}", self.document_name(), own.ext());
        let mut dialog = crate::files::Dialog::new();
        let others = crate::bitmap_file::BitmapFile::ALL
            .into_iter()
            .filter(|k| *k != own);
        for kind in std::iter::once(own).chain(others) {
            dialog = dialog.add_filter(kind.filter_name(), kind.exts());
        }
        dialog
            .set_file_name(name)
            .save_file(self, move |app, path| {
                app.status = match app.save_bitmap_to(id, &path, own) {
                    Ok(()) => crate::i18n::trf(
                        "status.exported",
                        &[("path", &path.display().to_string())],
                    ),
                    Err(e) => crate::i18n::trf("status.export_failed", &[("e", &e)]),
                };
            });
    }

    /// Write picture `id` to `path` as the kind its extension names, else
    /// as `fallback`.
    pub fn save_bitmap_to(
        &self,
        id: ShapeId,
        path: &std::path::Path,
        fallback: crate::bitmap_file::BitmapFile,
    ) -> Result<(), String> {
        let s = self.doc().find_shape(id).ok_or("no picture")?;
        let kind = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(crate::bitmap_file::BitmapFile::from_name)
            .unwrap_or(fallback);
        let icc = crate::bitmap_file::is_cmyk(s)
            .then(|| {
                s.picture_inks.as_deref().filter(|inks| {
                matches!(&s.kind, tracedraw_core::ShapeKind::Bitmap { png, .. } if inks.fits(png))
            }).and_then(|inks| inks.profile.as_ref()).map(|profile| profile.data.to_vec())
                .or_else(|| self.bitmap_cmyk_profile())
            })
            .flatten();
        let bytes = crate::bitmap_file::write(self.doc(), s, kind, icc.as_deref())?;
        crate::files::write(path, &bytes).map_err(|e| e.to_string())
    }

    /// The CMYK profile a CMYK picture's file carries: the one its inks
    /// are converted with, the drawing's own while it is used, else the
    /// one picked under Colour Management; none with the built-in
    /// conversions.
    fn bitmap_cmyk_profile(&self) -> Option<Vec<u8>> {
        let c = &self.settings.color;
        if c.builtin_engine {
            return None;
        }
        if c.open_cmyk == 0 && crate::app::carried_cmyk_profile(self.doc()).is_some() {
            return self.doc().cmyk_profile.as_ref().map(|p| p.data.to_vec());
        }
        self.settings_cmyk_profile().map(|p| p.data.to_vec())
    }

    /// Edit Bitmap's file of picture `id` in `dir`: a CMYK TIFF for a
    /// CMYK picture, so the other program gets its inks (#1834), else a
    /// PNG. The path written.
    pub fn write_bitmap_for_editing(
        &self,
        id: ShapeId,
        dir: &std::path::Path,
    ) -> Result<std::path::PathBuf, String> {
        use crate::bitmap_file::BitmapFile;
        let s = self.doc().find_shape(id).ok_or("no picture")?;
        let kind = if crate::bitmap_file::is_cmyk(s) {
            BitmapFile::Tiff
        } else {
            BitmapFile::Png
        };
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("bitmap-{}.{}", id.raw(), kind.ext()));
        self.save_bitmap_to(id, &path, kind)?;
        Ok(path)
    }

    /// Image > Edit Bitmap: write the picture to a file next to the
    /// system's temporary files (a CMYK TIFF for a CMYK picture, else a
    /// PNG), link the object to it and open the system editor (on
    /// Android, an editing app through "open with").
    pub fn edit_bitmap_externally(&mut self) {
        if crate::files::WEB {
            // A browser cannot hand a file to another program.
            self.status = tr("status.not_in_browser");
            return;
        }
        let Some(id) = self.selected_bitmap() else {
            return;
        };
        let Some(s) = self.doc().find_shape(id).cloned() else {
            return;
        };
        let dir = std::env::temp_dir().join("tracedraw-edit");
        let path = match self.write_bitmap_for_editing(id, &dir) {
            Ok(p) => p,
            Err(e) => {
                self.status = e;
                return;
            }
        };
        let mut data = s.data.clone();
        data.retain(|(k, _)| k != "bitmap.link");
        data.push(("bitmap.link".into(), path.display().to_string()));
        self.run(Command::SetObjectData { shape: id, data });
        // Android hands the picture to an editing app the user picks, with
        // the right to write it back (#1398); the file's watch reads it
        // again when it does.
        #[cfg(target_os = "android")]
        {
            let mime = if path.extension().is_some_and(|e| e == "tif") {
                "image/tiff"
            } else {
                "image/png"
            };
            self.status = if crate::android::outside::edit(&path, mime) {
                tr("status.bitmap_sent_to_app")
            } else {
                tr("status.no_app_edits_pictures")
            };
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        match crate::export::open_in_editor(&self.settings.bitmap_editor, &path) {
            Ok(()) => self.status = tr("status.bitmap_sent_to_editor"),
            Err(e) => self.status = e,
        }
    }

    /// Image > Update from Link: re-read the linked file into the object.
    pub fn update_bitmap_from_link(&mut self) {
        let Some(id) = self.selected_bitmap() else {
            return;
        };
        self.reload_linked_bitmap(id);
    }

    /// Re-read the file bitmap `id` is linked to into the object (Update
    /// from Link, or the file changed on disk). False when it is not
    /// linked or the file cannot be read (the status bar says so).
    pub fn reload_linked_bitmap(&mut self, id: ShapeId) -> bool {
        let Some(path) = self.bitmap_link(id) else {
            return false;
        };
        // A package's links are relative to its drawing (#852).
        let path = crate::package::resolve_link(&path, self.link_base().as_deref())
            .display()
            .to_string();
        // A CMYK TIFF keeps its inks (#1834).
        let read = crate::files::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|b| crate::bitmap_file::read(&b));
        let picture = match read {
            Ok(p) => p,
            Err(e) => {
                self.status = trf("status.could_not_read", &[("p", &path), ("e", &e)]);
                return false;
            }
        };
        let Some(s) = self.doc().find_shape(id).cloned() else {
            return false;
        };
        let ShapeKind::Bitmap { rect, .. } = s.kind else {
            return false;
        };
        let (w, h) = picture.rgba.dimensions();
        let Some((png, inks)) = picture.into_png() else {
            return false;
        };
        let mut cmds = vec![Command::SetShapeKind {
            shape: id,
            kind: ShapeKind::Bitmap {
                rect,
                width_px: w,
                height_px: h,
                png: png.into(),
                fx: None,
            },
        }];
        // A CMYK file makes it a CMYK picture with those inks; other
        // pixels leave the mode as it was and drop inks that no longer
        // fit.
        if inks.is_some() || s.picture_inks.is_some() {
            cmds.push(Command::SetPictureInks {
                shape: id,
                cmyk: s.cmyk_bitmap || inks.is_some(),
                inks,
            });
        }
        if cmds.len() == 1 {
            if let Some(cmd) = cmds.pop() {
                self.run(cmd);
            }
        } else if let Err(e) = self.engine.run_batch("Update from Link", &cmds) {
            self.status = e.to_string();
            return false;
        }
        self.status = tr("status.bitmap_updated");
        true
    }

    /// Linked bitmaps (Edit Bitmap, an imported file) follow their file:
    /// once a second the files' modification times are read, and a
    /// bitmap whose file changed since it was last seen is read again, so
    /// a save in the outside editor shows here at once. Desktop only.
    /// `now` is the frame's time; `ctx` keeps a frame coming while the
    /// watch is on.
    pub fn watch_linked_bitmaps(&mut self, ctx: Option<&egui::Context>, now: web_time::Instant) {
        if crate::files::WEB {
            return;
        }
        let gap = std::time::Duration::from_secs(1);
        if let Some(last) = self.linked_checked {
            let since = now.duration_since(last);
            if since < gap {
                if let Some(ctx) = ctx {
                    if !self.linked_mtimes.is_empty() {
                        ctx.request_repaint_after(gap - since);
                    }
                }
                return;
            }
        }
        self.linked_checked = Some(now);
        let linked: Vec<(ShapeId, String)> = self
            .doc()
            .all_layers()
            .flat_map(|l| l.shapes.iter())
            .filter_map(|s| {
                s.data
                    .iter()
                    .find(|(k, _)| k == "bitmap.link")
                    .map(|(_, v)| (s.id, v.clone()))
            })
            .collect();
        self.linked_mtimes
            .retain(|id, _| linked.iter().any(|(l, _)| l == id));
        let base = self.link_base();
        for (id, path) in linked {
            let path = crate::package::resolve_link(&path, base.as_deref());
            let Ok(modified) = std::fs::metadata(&path).and_then(|m| m.modified()) else {
                continue;
            };
            match self.linked_mtimes.insert(id, modified) {
                Some(seen) if seen != modified => {
                    self.reload_linked_bitmap(id);
                }
                _ => {}
            }
        }
        if let Some(ctx) = ctx {
            if !self.linked_mtimes.is_empty() {
                ctx.request_repaint_after(gap);
            }
        }
    }

    /// Image > Break Link: forget the file; the pixels stay embedded.
    pub fn break_bitmap_link(&mut self) {
        let Some(id) = self.selected_bitmap() else {
            return;
        };
        let Some(s) = self.doc().find_shape(id).cloned() else {
            return;
        };
        let mut data = s.data;
        data.retain(|(k, _)| k != "bitmap.link");
        self.run(Command::SetObjectData { shape: id, data });
    }
}

/// The paste key test of the windowing layer's gate (vendor/README.md,
/// #1488).
#[cfg(all(test, not(any(target_arch = "wasm32", target_os = "android"))))]
#[path = "../../../vendor/eframe/src/native/paste_gate.rs"]
mod eframe_paste_gate;

#[cfg(test)]
mod tests {
    use super::*;

    fn shape_count(app: &App) -> usize {
        app.doc().all_layers().map(|l| l.shapes.len()).sum()
    }

    /// A drawing with one rectangle, copied here; the system clipboard
    /// then held `system`.
    fn copied(system: &ClipboardContent) -> App {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(10.0, 10.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.select(vec![id]);
        app.copy();
        if let Some(cb) = app.clipboard.as_mut() {
            cb.system = system.mark(None);
        }
        app
    }

    fn svg_text() -> ClipboardContent {
        ClipboardContent {
            text: Some("<svg xmlns=\"http://www.w3.org/2000/svg\"/>".into()),
            ..Default::default()
        }
    }

    #[test]
    fn paste_sequence_changes_choose_external_fixture_without_system_clipboard() {
        let mut app = copied(&svg_text());
        app.clipboard.as_mut().unwrap().system.seq = Some(101);
        app.paste_with(
            Some(101),
            || panic!("unchanged clipboard must not be read"),
            || panic!("unchanged image must not be read"),
        );
        assert_eq!(shape_count(&app), 2);
        assert!(is_rect(&app));
        app.paste_with(
            Some(102),
            || ClipboardContent {
                text: Some("External fixture".into()),
                ..Default::default()
            },
            || None,
        );
        let selected = app.selected_shapes();
        let ShapeKind::Text { spans, .. } = &selected[0].kind else {
            panic!("external sequence must paste the external text")
        };
        assert_eq!(
            spans
                .iter()
                .map(|span| span.text.as_str())
                .collect::<String>(),
            "External fixture"
        );
        assert_eq!(shape_count(&app), 3);
        assert_eq!(
            change_count(),
            None,
            "unit adapters never query the user's sequence"
        );
    }

    fn picture() -> ClipboardContent {
        ClipboardContent {
            image: Some(image::RgbaImage::from_pixel(
                3,
                2,
                image::Rgba([1, 2, 3, 255]),
            )),
            ..Default::default()
        }
    }

    fn is_bitmap(app: &App) -> bool {
        app.selected_shapes()
            .first()
            .is_some_and(|s| matches!(s.kind, ShapeKind::Bitmap { .. }))
    }

    #[test]
    fn the_copy_here_is_pasted_while_the_system_clipboard_still_holds_it() {
        let mut app = copied(&svg_text());
        app.paste_from(svg_text());
        assert_eq!(shape_count(&app), 2);
        assert!(matches!(
            app.selected_shapes().first().map(|s| &s.kind),
            Some(ShapeKind::Rect { .. })
        ));
        // No system clipboard (a session without one): the copy here.
        app.paste_from(ClipboardContent::default());
        assert_eq!(shape_count(&app), 3);
    }

    #[test]
    fn a_picture_copied_elsewhere_afterwards_wins_over_the_copy_here() {
        let mut app = copied(&svg_text());
        app.paste_from(picture());
        assert_eq!(shape_count(&app), 2);
        assert!(is_bitmap(&app), "the picture, not the rectangle");
        // Text copied elsewhere becomes text.
        app.paste_from(ClipboardContent {
            text: Some("Hello".into()),
            ..Default::default()
        });
        assert!(matches!(
            app.selected_shapes().first().map(|s| &s.kind),
            Some(ShapeKind::Text { .. })
        ));
    }

    fn png_file(name: &str) -> PastedFile {
        let mut bytes = Vec::new();
        image::RgbaImage::from_pixel(5, 4, image::Rgba([200, 30, 30, 255]))
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("png");
        PastedFile {
            name: name.into(),
            mime: "image/png".into(),
            bytes,
        }
    }

    fn is_rect(app: &App) -> bool {
        matches!(
            app.selected_shapes().first().map(|s| &s.kind),
            Some(ShapeKind::Rect { .. })
        )
    }

    /// #1403: a picture copied in another application comes with the
    /// browser's paste event as a file, read in the background; the
    /// paste shortcut's frame pastes nothing meanwhile (not the copy
    /// here), and the picture is placed as a bitmap when it arrives.
    #[test]
    fn a_picture_pasted_in_a_browser_is_placed_as_a_bitmap() {
        let mut app = copied(&svg_text());
        expect_handed();
        app.paste_with_event_text(None);
        assert_eq!(shape_count(&app), 1, "the copy here waits");
        hand_over(Handed {
            files: vec![png_file("image.png")],
            ..Default::default()
        });
        assert!(!handed_awaited());
        app.take_handed_pastes();
        assert_eq!(shape_count(&app), 2);
        assert!(is_bitmap(&app), "the picture, not the rectangle");
        // Taken once.
        app.take_handed_pastes();
        assert_eq!(shape_count(&app), 2);
        // A file that is no picture it can read pastes nothing.
        hand_over(Handed {
            files: vec![PastedFile {
                name: "broken.png".into(),
                mime: "image/png".into(),
                bytes: vec![1, 2, 3],
            }],
            ..Default::default()
        });
        app.take_handed_pastes();
        assert_eq!(shape_count(&app), 2);
        // The next paste is no longer held back.
        app.paste_with_event_text(None);
        assert_eq!(shape_count(&app), 3);
    }

    /// #1403: Edit > Paste in a browser reads the clipboard with its API:
    /// the copy made here while the clipboard still holds it, text and
    /// pictures copied elsewhere otherwise, and the copy here (with a
    /// word in the status bar) when the browser refuses.
    #[test]
    fn edit_paste_in_a_browser_takes_what_the_clipboard_api_read() {
        let mut app = copied(&svg_text());
        let menu = |text: Option<&str>, files: Vec<PastedFile>| Handed {
            text: text.map(String::from),
            files,
            from_menu: true,
            ..Default::default()
        };
        app.paste_handed(menu(svg_text().text.as_deref(), Vec::new()));
        assert_eq!(shape_count(&app), 2);
        assert!(is_rect(&app), "the copy here, as objects");
        app.paste_handed(menu(Some("Hello"), Vec::new()));
        assert!(matches!(
            app.selected_shapes().first().map(|s| &s.kind),
            Some(ShapeKind::Text { .. })
        ));
        app.paste_handed(menu(None, vec![png_file("clipboard.png")]));
        assert!(is_bitmap(&app));
        let n = shape_count(&app);
        app.paste_handed(Handed {
            refused: true,
            from_menu: true,
            ..Default::default()
        });
        assert_eq!(shape_count(&app), n + 1);
        assert!(is_rect(&app), "refused: the copy here");
        assert_eq!(app.status, tr("status.clipboard_refused"));
    }

    /// Edit > Paste waits for the browser (which may ask the user first)
    /// without holding back the paste shortcut; a paste event's files do.
    #[test]
    fn only_a_paste_events_files_hold_back_the_shortcut() {
        expect_handed();
        hand_over(Handed {
            from_menu: true,
            ..Default::default()
        });
        assert!(handed_awaited(), "the paste event's files still come");
        hand_over(Handed::default());
        assert!(!handed_awaited());
        HANDED.with(|h| h.borrow_mut().clear());
    }

    /// The page's context menu pastes where it was opened, and moves
    /// nothing when nothing was pasted (#1403: the browser pastes on a
    /// later frame, with the place kept for then).
    #[test]
    fn a_paste_from_the_context_menu_lands_where_it_was_opened() {
        let mut app = copied(&svg_text());
        let at = Point::new(150.0, 120.0);
        app.paste_handed(Handed {
            files: vec![png_file("image.png")],
            at: Some(at),
            ..Default::default()
        });
        let b = app.selection_bounds().expect("pasted");
        assert!((b.center().x - at.x).abs() < 1e-6 && (b.center().y - at.y).abs() < 1e-6);
        // Nothing pasted (nothing copied anywhere): the selected
        // objects stay where they are.
        let mut app = copied(&svg_text());
        app.clipboard = None;
        let before = app.selection_bounds().expect("selected");
        app.paste_handed(Handed {
            at: Some(Point::new(0.0, 0.0)),
            ..Default::default()
        });
        assert_eq!(app.selection_bounds(), Some(before));
    }

    /// While a text is being edited, a picture pasted from elsewhere
    /// does not go into it, and Edit > Paste's text does.
    #[test]
    fn a_picture_pasted_while_editing_text_is_left_out() {
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Text {
                spans: vec![TextSpan::new("ab", "Sans", 24.0)],
                origin: Point::new(20.0, 20.0),
                frame: None,
                align: tracedraw_core::TextAlign::Left,
                para: Default::default(),
                on_path: None,
            })
            .expect("text");
        app.select(vec![id]);
        app.begin_text_edit(id);
        assert!(app.editing_text());
        app.paste_handed(Handed {
            files: vec![png_file("image.png")],
            ..Default::default()
        });
        assert_eq!(shape_count(&app), 1);
        app.paste_handed(Handed {
            text: Some("cd".into()),
            from_menu: true,
            ..Default::default()
        });
        let text: String = match &app.doc().find_shape(id).expect("text").kind {
            ShapeKind::Text { spans, .. } => spans.iter().map(|s| s.text.as_str()).collect(),
            _ => String::new(),
        };
        assert!(text.contains("cd"), "{text}");
    }

    /// #1403 (Android): the clipboard's documents as the activity lists
    /// them; one picture alone is the clipboard's picture, anything else
    /// is imported as files.
    #[test]
    fn android_clipboard_documents_become_a_picture_or_files() {
        let docs = parse_clip_documents(
            "content://media/external/images/1\timage/jpeg\tIMG_1.jpg\n\ncontent://x/doc\t\tplan.svg",
        );
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].mime, "image/jpeg");
        assert_eq!(docs[1].name, "plan.svg");
        let (files, picture) = split_documents(docs[..1].to_vec());
        assert!(files.is_empty());
        assert_eq!(
            picture.as_deref(),
            Some("content://media/external/images/1")
        );
        let (files, picture) = split_documents(docs.clone());
        assert!(picture.is_none());
        assert_eq!(files.len(), 2);
        assert_eq!(
            crate::files::document_uri(&files[1]).as_deref(),
            Some("content://x/doc")
        );
        assert_eq!(
            files[1].file_name().and_then(|n| n.to_str()),
            Some("plan.svg")
        );
        assert!(parse_clip_documents("").is_empty());
    }

    #[test]
    fn files_copied_in_a_file_manager_are_imported() {
        let dir = std::env::temp_dir().join(format!("tracedraw-paste-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("dot.png");
        image::RgbaImage::from_pixel(4, 2, image::Rgba([0, 120, 215, 255]))
            .save(&path)
            .expect("png");
        let mut app = copied(&svg_text());
        app.paste_from(ClipboardContent {
            files: vec![path],
            ..Default::default()
        });
        assert_eq!(shape_count(&app), 2);
        assert!(is_bitmap(&app));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn copied_file_paths_lose_the_line_end() {
        use std::path::PathBuf;
        assert_eq!(
            without_line_end(PathBuf::from("/home/a/b c.psd\r")),
            PathBuf::from("/home/a/b c.psd")
        );
        assert_eq!(
            without_line_end(PathBuf::from("/home/a.png")),
            PathBuf::from("/home/a.png")
        );
    }

    #[test]
    fn the_mark_follows_the_content() {
        let empty = ClipboardContent::default();
        assert!(empty.is_empty());
        assert_eq!(empty.mark(None), ClipboardContent::default().mark(None));
        assert_ne!(empty.mark(None), svg_text().mark(None));
        assert_ne!(svg_text().mark(None), picture().mark(None));
        assert!(!picture().is_empty());
        // Pictures of one size told apart by their pixels.
        let mut other = picture();
        if let Some(img) = other.image.as_mut() {
            img.put_pixel(2, 1, image::Rgba([9, 9, 9, 255]));
        }
        assert_ne!(picture().mark(None), other.mark(None));
    }

    /// The change check reads as little as it can (#183): the system's
    /// change count decides alone; else equal text or files say the same
    /// copy without looking at the picture, and the picture is sampled
    /// only on a clipboard with neither.
    #[test]
    fn the_change_check_decodes_a_picture_only_when_it_must() {
        let no_image = || -> Option<u64> { panic!("the picture was decoded") };
        let counted = svg_text().mark(Some(7));
        assert!(counted.still_holds(Some(7), &None, &[], no_image));
        assert!(!counted.still_holds(Some(8), &svg_text().text, &[], no_image));
        let text = svg_text().mark(None);
        assert!(text.still_holds(None, &svg_text().text, &[], no_image));
        assert!(!text.still_holds(None, &Some("other".into()), &[], no_image));
        assert!(!text.still_holds(None, &None, &[], no_image));
        let pic = picture().mark(None);
        let sample = picture().image.as_ref().map(image_sample);
        let mut decoded = 0;
        assert!(pic.still_holds(None, &None, &[], || {
            decoded += 1;
            sample
        }));
        assert!(!pic.still_holds(None, &None, &[], || {
            decoded += 1;
            None
        }));
        assert_eq!(decoded, 2);
    }

    /// Pasting the copy here never decodes the picture of a clipboard that
    /// still holds the copy's text; a picture copied elsewhere is decoded
    /// once and pasted, files are imported without decoding one (#183).
    #[test]
    fn a_paste_decodes_the_picture_only_when_it_needs_it() {
        let mut app = copied(&svg_text());
        let no_image = || -> Option<image::RgbaImage> { panic!("the picture was decoded") };
        app.paste_with(None, svg_text, no_image);
        assert_eq!(shape_count(&app), 2);
        // The system's change count unchanged: nothing is read at all.
        if let Some(cb) = app.clipboard.as_mut() {
            cb.system.seq = Some(3);
        }
        app.paste_with(Some(3), || panic!("the clipboard was read"), no_image);
        assert_eq!(shape_count(&app), 3);
        // A picture copied elsewhere since: decoded once, pasted.
        let mut decoded = 0;
        app.paste_with(Some(4), ClipboardContent::default, || {
            decoded += 1;
            picture().image
        });
        assert_eq!(decoded, 1);
        assert!(is_bitmap(&app));
        // Text written by the copy here is what a paste reads back.
        let mut app = copied(&ClipboardContent::default());
        app.mark_system_clipboard(svg_text().text.as_deref());
        app.paste_with(None, svg_text, no_image);
        assert!(matches!(
            app.selected_shapes().first().map(|s| &s.kind),
            Some(ShapeKind::Rect { .. })
        ));
    }
}

/// What copy and paste cost with a 4000 x 3000 picture on the system
/// clipboard: run in a session with a clipboard (on Linux an X server,
/// such as `Xvfb :99` with `DISPLAY=:99`) with
/// `cargo test --release -p tracedraw -- --ignored clipboard_costs --nocapture`.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod clipboard_bench {
    use super::*;

    #[derive(Default)]
    struct MemoryProvider {
        content: ClipboardContent,
        sequence: u32,
    }

    impl MemoryProvider {
        fn publish(&mut self, text: &str) -> Result<(), String> {
            self.sequence += 1;
            self.content = ClipboardContent {
                text: Some(text.to_string()),
                ..Default::default()
            };
            Ok(())
        }

        fn paste_into(&self, app: &mut App) {
            app.paste_with(
                Some(self.sequence),
                || ClipboardContent {
                    text: self.content.text.clone(),
                    image: None,
                    files: self.content.files.clone(),
                },
                || self.content.image.clone(),
            );
        }

        fn mark(&self, app: &mut App) {
            if let Some(clipboard) = &mut app.clipboard {
                clipboard.system = Mark {
                    seq: Some(self.sequence),
                    ..Default::default()
                };
            }
        }
    }

    fn picture_elsewhere(cb: &mut MemoryProvider, shade: u8) {
        let (w, h) = (4000usize, 3000usize);
        let bytes: Vec<u8> = (0..w * h * 4).map(|i| (i % 251) as u8 ^ shade).collect();
        cb.sequence += 1;
        cb.content = ClipboardContent {
            image: Some(
                image::RgbaImage::from_raw(w as u32, h as u32, bytes).expect("fixture dimensions"),
            ),
            ..Default::default()
        };
    }

    fn time(label: &str, f: impl FnOnce()) {
        let t = std::time::Instant::now();
        f();
        eprintln!("{label}: {:.1} ms", t.elapsed().as_secs_f64() * 1000.0);
    }

    #[test]
    #[ignore = "isolated 4000x3000 memory-provider workload; never reads or writes the OS clipboard"]
    fn clipboard_costs() {
        let mut cb = MemoryProvider::default();
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(10.0, 10.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.select(vec![id]);
        picture_elsewhere(&mut cb, 0);
        time("Memory-provider copy over a foreign picture", || {
            app.try_copy_with_publisher(|text| cb.publish(text))
                .expect("memory publication");
            cb.mark(&mut app);
        });
        time("Memory-provider paste of that copy", || {
            cb.paste_into(&mut app)
        });
        picture_elsewhere(&mut cb, 1);
        app.select(vec![id]);
        app.copy();
        time("a copy of nodes or guides (mark, picture kept)", || {
            cb.mark(&mut app);
        });
        time("Memory-provider paste, picture still held", || {
            cb.paste_into(&mut app);
        });
        time("Memory-provider RGBA copy alone, not OS decoding", || {
            let _ = std::hint::black_box(cb.content.image.clone());
        });
        time("Memory-provider change counter", || {
            std::hint::black_box(cb.sequence);
        });
        picture_elsewhere(&mut cb, 2);
        let before = app
            .doc()
            .all_layers()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        time("Memory-provider paste of a new foreign picture", || {
            cb.paste_into(&mut app);
        });
        let after = app
            .doc()
            .all_layers()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        assert_eq!(after, before + 1);
    }
}

#[cfg(test)]
mod linked_watch_tests {
    use crate::app::App;
    use tracedraw_core::geometry::Rect;
    use tracedraw_core::{Command, ShapeKind};

    fn png(w: u32, h: u32) -> Vec<u8> {
        tiny_skia::Pixmap::new(w, h)
            .and_then(|p| p.encode_png().ok())
            .unwrap_or_default()
    }

    /// A linked bitmap follows its file: a change on disk is read in on
    /// the next check (once a second), as Update from Link does; a file
    /// seen for the first time is only remembered (#284).
    #[test]
    fn a_linked_bitmap_is_read_again_when_its_file_changes() {
        if crate::files::WEB {
            return;
        }
        let dir = std::env::temp_dir().join(format!("tracedraw-link-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("picture.png");
        std::fs::write(&path, png(2, 2)).expect("png");
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(10.0, 10.0, 30.0, 30.0),
                width_px: 2,
                height_px: 2,
                png: png(2, 2).into(),
                fx: None,
            })
            .expect("bitmap");
        app.run(Command::SetObjectData {
            shape: id,
            data: vec![("bitmap.link".into(), path.display().to_string())],
        });
        let width = |app: &App| match &app.doc().find_shape(id).expect("bitmap").kind {
            ShapeKind::Bitmap { width_px, .. } => *width_px,
            _ => 0,
        };
        let t0 = web_time::Instant::now();
        app.watch_linked_bitmaps(None, t0);
        assert_eq!(width(&app), 2, "first sight: remembered, not read");
        // The file changes (its time stamp moved on by two seconds).
        std::fs::write(&path, png(3, 3)).expect("png");
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(2);
        std::fs::File::options()
            .write(true)
            .open(&path)
            .and_then(|f| f.set_modified(later))
            .expect("mtime");
        // Within the second: not looked at yet.
        app.watch_linked_bitmaps(None, t0 + std::time::Duration::from_millis(500));
        assert_eq!(width(&app), 2);
        app.watch_linked_bitmaps(None, t0 + std::time::Duration::from_millis(1100));
        assert_eq!(width(&app), 3, "read again");
        // Unchanged since: left alone (no new undo step).
        let depth = app.engine.history_labels().0.len();
        app.watch_linked_bitmaps(None, t0 + std::time::Duration::from_millis(2200));
        assert_eq!(app.engine.history_labels().0.len(), depth);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
