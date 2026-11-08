//! File access for both builds.
//!
//! Natively these are the file system and the system file dialogs, used
//! synchronously as before. In a browser there is no file system: a file
//! the user picks is kept in memory under a synthetic path
//! (`/upload/<name>`), so the path-based open and import code reads it with
//! [`read`] like any file, and [`write`] offers the bytes as a download
//! named after the path's file name.
//!
//! Dialogs come in three shapes:
//! - [`Dialog::pick_file`] runs a closure with the picked path: at once
//!   natively, on a later frame in a browser (see [`run_finished`]).
//! - [`Dialog::pick_into`] stores the picked path under a key that a
//!   dialog's own code collects with [`take_picked`] on the same frame
//!   (natively) or a later one (browser).
//! - [`Dialog::save_file`] runs a closure with the chosen path at once: a
//!   save dialog natively, `/download/<file name>` in a browser.

use crate::app::App;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type Job = Box<dyn FnOnce(&mut App, PathBuf)>;

thread_local! {
    /// Dialogs finished in the background (browser), waiting for a frame.
    static FINISHED: RefCell<Vec<(Job, PathBuf)>> = const { RefCell::new(Vec::new()) };
    /// Picked paths waiting for the dialog that asked for them.
    static PICKED: RefCell<HashMap<String, PathBuf>> = RefCell::new(HashMap::new());
    /// Uploaded files by synthetic path (browser only).
    static STORE: RefCell<HashMap<PathBuf, Vec<u8>>> = RefCell::new(HashMap::new());
}

/// True in the browser build: no file system, downloads instead of saves.
pub const WEB: bool = cfg!(target_arch = "wasm32");

/// The browser's auto-backup folder: the session markers live in the
/// page's local storage and the backups in its IndexedDB database
/// (`web::backups`), and [`read`], [`write_creating`],
/// [`write_creating_with`], [`exists`], [`remove`] and [`list_dir`] reach
/// them under this path as if they were files.
pub const WEB_BACKUP_DIR: &str = "/backup";

/// True for a path in [`WEB_BACKUP_DIR`] in the browser build.
fn in_web_backups(path: &Path) -> bool {
    WEB && path.starts_with(WEB_BACKUP_DIR)
}

/// True in the Android app: files come and go through the system's
/// document picker, named by [`document_path`] (`android.rs`,
/// docs/behavior/android.md).
pub const ANDROID: bool = cfg!(target_os = "android");

/// The folder under which a document the Android picker returned is
/// named: `/content/<its URI, escaped>/<its display name>`. The name
/// gives the title and the type (its extension), the URI the bytes.
pub const DOCUMENT_ROOT: &str = "/content";

/// The path naming the document at the content URI `uri` shown as
/// `name`: `/content/<uri escaped>/<name>`, so that `file_name` and
/// `extension` work as for any file and [`read`] and [`write`] find the
/// URI again with [`document_uri`].
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn document_path(uri: &str, name: &str) -> PathBuf {
    let mut escaped = String::with_capacity(uri.len());
    for c in uri.chars() {
        match c {
            '%' => escaped.push_str("%25"),
            '/' => escaped.push_str("%2F"),
            c => escaped.push(c),
        }
    }
    let name = name.replace(['/', '\\'], "_");
    let name = if name.trim().is_empty() {
        "document".to_string()
    } else {
        name
    };
    PathBuf::from(DOCUMENT_ROOT).join(escaped).join(name)
}

/// The content URI of a path made by [`document_path`]; `None` for any
/// other path.
pub fn document_uri(path: &Path) -> Option<String> {
    let mut parts = path.strip_prefix(DOCUMENT_ROOT).ok()?.components();
    let std::path::Component::Normal(escaped) = parts.next()? else {
        return None;
    };
    let std::path::Component::Normal(_) = parts.next()? else {
        return None;
    };
    if parts.next().is_some() {
        return None;
    }
    let escaped = escaped.to_str()?;
    let mut uri = String::with_capacity(escaped.len());
    let mut chars = escaped.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let code: String = chars.by_ref().take(2).collect();
            match code.as_str() {
                "25" => uri.push('%'),
                "2F" => uri.push('/'),
                other => {
                    uri.push('%');
                    uri.push_str(other);
                }
            }
        } else {
            uri.push(c);
        }
    }
    Some(uri)
}

/// True for a path made by [`document_path`].
pub fn is_document_path(path: &Path) -> bool {
    document_uri(path).is_some()
}

/// A picked document's name with an extension: its own, else one for the
/// MIME type its provider gave (a photo called "IMG_1" of type image/jpeg
/// becomes "IMG_1.jpg"), so the editor can tell the type.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn name_with_extension(name: &str, mime: &str) -> String {
    if Path::new(name).extension().is_some() {
        return name.to_string();
    }
    let ext = match mime.trim().to_ascii_lowercase().as_str() {
        "application/pdf" => "pdf",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/bmp" => "bmp",
        "image/webp" => "webp",
        "image/tiff" => "tiff",
        "image/svg+xml" => "svg",
        "text/plain" => "txt",
        "text/csv" => "csv",
        "application/json" => "json",
        "application/postscript" => "eps",
        _ => return name.to_string(),
    };
    format!("{name}.{ext}")
}

/// `name` as one file name of its own inside a folder: no folder
/// separators or control characters, no leading dots (nothing hidden, no
/// `..`), never empty.
pub fn safe_file_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let clean = clean.trim().trim_start_matches('.').trim();
    if clean.is_empty() {
        "file".into()
    } else {
        clean.to_string()
    }
}

/// The MIME type the system's save picker is told for a file with the
/// extension `ext` (any case); a plain byte stream for the formats it
/// does not know, which keeps the name as typed.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn mime_for_extension(ext: &str) -> &'static str {
    match ext.trim_start_matches('.').to_ascii_lowercase().as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "webp" => "image/webp",
        "tif" | "tiff" => "image/tiff",
        "svg" => "image/svg+xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "json" => "application/json",
        "zip" => "application/zip",
        "eps" | "ps" => "application/postscript",
        _ => "application/octet-stream",
    }
}

/// The folder the Save Drawing dialog starts in when the platform fixes
/// it: the Android app's documents folder; `None` elsewhere (the last
/// folder used, Documents or home).
pub fn save_dir() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        let dir = android_documents_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            log::warn!("cannot create {}: {e}", dir.display());
        }
        return Some(dir);
    }
    #[allow(unreachable_code)]
    None
}

/// Where the Android app saves drawings and exports: its own folder under
/// the shared storage (`Android/data/<package>/files`, which file managers
/// show), set by `android_main`; the settings folder when it is missing.
#[cfg(target_os = "android")]
fn android_documents_dir() -> PathBuf {
    std::env::var_os("TRACEDRAW_DOCUMENTS_DIR")
        .map(PathBuf::from)
        .or_else(crate::settings::config_dir)
        .unwrap_or_else(|| PathBuf::from("/sdcard/Download"))
}

/// The path naming the same file however it was reached (links, `.`
/// and `..`); the absolute path when that fails (the file is gone);
/// `path` as it is in a browser.
pub fn canonical(path: &Path) -> PathBuf {
    if WEB || is_document_path(path) {
        return path.to_path_buf();
    }
    std::fs::canonicalize(path)
        .or_else(|_| std::path::absolute(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

/// Two canonical paths name one file: on Windows without regard to the
/// letter case and to the `\\?\` prefix.
pub fn same_path(a: &Path, b: &Path) -> bool {
    if cfg!(windows) {
        let key = |p: &Path| {
            let s = p.to_string_lossy().to_lowercase();
            s.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(s)
        };
        key(a) == key(b)
    } else {
        a == b
    }
}

/// A canonical path as people type it: on Windows without the `\\?\`
/// prefix (`\\?\C:\a` is `C:\a`, `\\?\UNC\server\share` is
/// `\\server\share`); other paths as they are.
pub fn plain_path(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    match s.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => path.to_path_buf(),
    }
}

/// Read a whole file (or an uploaded one in a browser).
pub fn read(path: impl AsRef<Path>) -> std::io::Result<Vec<u8>> {
    read_limited(path, MAX_DOCUMENT_BYTES)
}

/// Upper bounds for locally opened documents and decoded text. These are
/// resource policies, not claims about another editor's supported file sizes.
pub const MAX_DOCUMENT_BYTES: usize = 512 << 20;
pub const MAX_DOCUMENT_TEXT_BYTES: usize = 256 << 20;

#[cfg(test)]
thread_local! {
    static TEST_INPUT_BUDGET: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

pub(crate) fn effective_input_budget(limit: usize) -> usize {
    #[cfg(test)]
    return TEST_INPUT_BUDGET.with(|value| value.get().map_or(limit, |test| test.min(limit)));
    #[cfg(not(test))]
    limit
}

#[cfg(test)]
pub(crate) fn with_test_input_budget<T>(limit: usize, action: impl FnOnce() -> T) -> T {
    struct Restore(Option<usize>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_INPUT_BUDGET.with(|value| value.set(self.0));
        }
    }
    let _restore = Restore(TEST_INPUT_BUDGET.with(|value| value.replace(Some(limit))));
    action()
}

pub fn read_limited(path: impl AsRef<Path>, limit: usize) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let limit = effective_input_budget(limit);
    let too_large = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("file exceeds the {limit}-byte input budget"),
        )
    };
    let path = path.as_ref();
    #[cfg(target_arch = "wasm32")]
    if in_web_backups(path) {
        let data = crate::web::backups::read(path)
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
        return if data.len() <= limit {
            Ok(data)
        } else {
            Err(too_large())
        };
    }
    if WEB {
        return STORE
            .with(|s| {
                s.borrow().get(path).map(|bytes| {
                    if bytes.len() > limit {
                        Err(too_large())
                    } else {
                        Ok(bytes.clone())
                    }
                })
            })
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} is not available in the browser", path.display()),
                )
            })?;
    }
    #[cfg(target_os = "android")]
    if let Some(uri) = document_uri(path) {
        let data = crate::android::documents::read(&uri)?;
        return if data.len() <= limit {
            Ok(data)
        } else {
            Err(too_large())
        };
    }
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > limit as u64 {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    file.take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        Err(too_large())
    } else {
        Ok(bytes)
    }
}

/// Read a whole file as UTF-8 text.
pub fn read_to_string(path: impl AsRef<Path>) -> std::io::Result<String> {
    read_to_string_limited(path, MAX_DOCUMENT_TEXT_BYTES)
}

pub fn read_to_string_limited(path: impl AsRef<Path>, limit: usize) -> std::io::Result<String> {
    let bytes = read_limited(path, limit)?;
    String::from_utf8(bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

/// Write a whole file; in a browser, download it under the path's name
/// (or keep it in memory, under [`MEMORY_ROOT`]).
pub fn write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    let path = path.as_ref();
    if in_memory(path) {
        store(path, bytes.as_ref().to_vec());
        return Ok(());
    }
    if WEB {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "download".into());
        return download(&name, bytes.as_ref());
    }
    #[cfg(target_os = "android")]
    if let Some(uri) = document_uri(path) {
        return crate::android::documents::write(&uri, bytes.as_ref());
    }
    std::fs::write(path, bytes)
}

/// Write a file as `fill` makes it, without holding the whole file in
/// memory; in a browser, the bytes are gathered and downloaded.
pub fn write_with(
    path: impl AsRef<Path>,
    fill: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    // A download, or a document the Android picker returned: gathered,
    // then handed over whole.
    if WEB || is_document_path(path) {
        let mut bytes = Vec::new();
        fill(&mut bytes)?;
        return write(path, bytes);
    }
    let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
    fill(&mut w)?;
    std::io::Write::flush(&mut w)
}

/// Write a file as `fill` makes it, creating its folder first
/// (auto-backups). It is written under a `.part` name and renamed when
/// complete, so a write cut short never leaves half a file under the
/// real name. In a browser only the auto-backup folder
/// ([`WEB_BACKUP_DIR`]) is written.
pub fn write_creating_with(
    path: impl AsRef<Path>,
    fill: impl FnOnce(&mut dyn std::io::Write) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if in_web_backups(path) {
        let mut bytes = Vec::new();
        fill(&mut bytes)?;
        return write_creating(path, bytes);
    }
    if WEB {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    let part = PathBuf::from(part);
    let written = std::fs::File::create(&part).and_then(|f| {
        let mut w = std::io::BufWriter::new(f);
        fill(&mut w)?;
        std::io::Write::flush(&mut w)
    });
    match written.and_then(|()| std::fs::rename(&part, path)) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            Err(e)
        }
    }
}

/// Copy a file, creating the target folder (backups). Nothing happens in
/// a browser, which has no files to copy.
pub fn copy(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    if WEB {
        return Ok(());
    }
    if let Some(dir) = to.as_ref().parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::copy(from, to).map(|_| ())
}

/// True when a file exists on disk (never in a browser). A document the
/// Android picker returned counts as existing: the system holds it and
/// asked about replacing it already.
pub fn exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    #[cfg(target_arch = "wasm32")]
    if in_web_backups(path) {
        return crate::web::backups::exists(path);
    }
    if WEB {
        return false;
    }
    (ANDROID && is_document_path(path)) || path.is_file()
}

/// Write a file, creating its folder first (auto-backups). In a browser
/// only the auto-backup folder ([`WEB_BACKUP_DIR`]) is written.
pub fn write_creating(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> std::io::Result<()> {
    #[cfg(target_arch = "wasm32")]
    if in_web_backups(path.as_ref()) {
        return crate::web::backups::write(path.as_ref(), bytes.as_ref());
    }
    if WEB {
        return Ok(());
    }
    if let Some(dir) = path.as_ref().parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, bytes)
}

/// Write `bytes` to a device such as a serial port (`/dev/ttyUSB0`,
/// `COM3`) without creating or truncating anything: the device must
/// exist. A bare Windows port name (`COM10`) is opened through the
/// device namespace (`\\.\COM10`), which every port number needs past
/// 9. Not in a browser or on Android, which have no such devices.
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn write_device(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;
    let path = path.trim();
    let upper = path.to_ascii_uppercase();
    let device = if cfg!(windows)
        && upper.starts_with("COM")
        && upper.len() > 3
        && upper[3..].bytes().all(|b| b.is_ascii_digit())
    {
        format!("\\\\.\\{path}")
    } else {
        path.to_string()
    };
    let mut f = std::fs::OpenOptions::new().write(true).open(device)?;
    f.write_all(bytes)?;
    f.flush()
}

/// Delete a file (in a browser, only in the auto-backup folder and the
/// folders kept in memory).
pub fn remove(path: impl AsRef<Path>) -> std::io::Result<()> {
    #[cfg(target_arch = "wasm32")]
    if in_web_backups(path.as_ref()) {
        crate::web::backups::remove(path.as_ref());
        return Ok(());
    }
    if in_memory(path.as_ref()) {
        forget(path.as_ref());
    }
    if WEB {
        return Ok(());
    }
    std::fs::remove_file(path)
}

/// The kind of system the app runs on, where what a dialog can do
/// depends on it (#1399).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Desktop,
    Web,
    Android,
}

impl Platform {
    /// The platform of this build.
    pub const fn current() -> Platform {
        if WEB {
            Platform::Web
        } else if ANDROID {
            Platform::Android
        } else {
            Platform::Desktop
        }
    }
}

/// The places where the desktop asks for a folder (#1399).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderUse {
    /// The Export docker's folder: the assets are written into it.
    ExportAssets,
    /// File > Package: the drawing, its pictures and fonts.
    Package,
    /// Font Manager > Add Folder: its fonts are read.
    FontFolder,
    /// Print Merge's pictures folder: the pictures cells name are read.
    MergePictures,
    /// Picture Mosaic's library: its pictures are read.
    MosaicLibrary,
    /// Options > Text > Spelling, More dictionaries: Hunspell files read.
    DictionaryFolder,
    /// Options > Save, the backup made before saving.
    BackupFolder,
    /// Options > Save, the auto-backups.
    AutoBackupFolder,
}

impl FolderUse {
    #[cfg(test)]
    pub const ALL: [FolderUse; 8] = [
        FolderUse::ExportAssets,
        FolderUse::Package,
        FolderUse::FontFolder,
        FolderUse::MergePictures,
        FolderUse::MosaicLibrary,
        FolderUse::DictionaryFolder,
        FolderUse::BackupFolder,
        FolderUse::AutoBackupFolder,
    ];

    /// What the folder button does in this build.
    pub fn action(self) -> FolderAction {
        folder_action(self, Platform::current())
    }

    /// The tip that says why the button does what it does in this build
    /// (an i18n key), `None` on the desktop.
    pub fn tip(self) -> Option<&'static str> {
        folder_tip(self, Platform::current())
    }
}

/// What a folder button does (#1399).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderAction {
    /// The system's folder dialog ([`pick_folder`]), on the desktop.
    PickFolder(FolderToken),
    /// The files are written into one zip file, downloaded in a browser
    /// and saved where the system's save picker says on Android.
    SaveZip,
    /// The files a folder would hold are chosen with the open picker,
    /// several at once.
    OpenFiles,
    /// No folder can be chosen: the button is greyed, with this tip (an
    /// i18n key) saying why.
    Unavailable(&'static str),
}

/// Leave to show the system's folder dialog: only [`folder_action`] makes
/// one, and only for the desktop, so no dialog asks for a folder in a
/// browser or on Android, where none can be picked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderToken(());

/// What the button of `place` does on `platform`. The desktop picks a
/// folder; elsewhere a folder that is only written into becomes one zip
/// file, a folder that is read becomes several picked files, and the
/// backup folders, which the app writes on its own later, cannot be
/// chosen.
pub fn folder_action(place: FolderUse, platform: Platform) -> FolderAction {
    use FolderAction::*;
    use FolderUse::*;
    match (platform, place) {
        (Platform::Desktop, _) => PickFolder(FolderToken(())),
        // File > Package is greyed in a browser (#1416).
        (Platform::Web, Package) => Unavailable("package.desktop_only"),
        (_, ExportAssets | Package) => SaveZip,
        (_, FontFolder | MergePictures | MosaicLibrary | DictionaryFolder) => OpenFiles,
        (Platform::Web, BackupFolder | AutoBackupFolder) => Unavailable("folder.backup_web"),
        (Platform::Android, BackupFolder | AutoBackupFolder) => {
            Unavailable("folder.backup_android")
        }
    }
}

/// The tip of the button of `place` on `platform` (an i18n key): why it
/// writes a zip, opens files or is greyed; `None` where it picks a folder.
pub fn folder_tip(place: FolderUse, platform: Platform) -> Option<&'static str> {
    let web = platform == Platform::Web;
    match folder_action(place, platform) {
        FolderAction::PickFolder(_) => None,
        FolderAction::SaveZip if web => Some("folder.zip_web"),
        FolderAction::SaveZip => Some("folder.zip_android"),
        FolderAction::OpenFiles if web => Some("folder.files_web"),
        // The pictures of Print Merge: the system's folder picker (#1926).
        FolderAction::OpenFiles if place == FolderUse::MergePictures => Some("folder.tree_android"),
        FolderAction::OpenFiles => Some("folder.files_android"),
        FolderAction::Unavailable(key) => Some(key),
    }
}

/// Choose a folder with the system's dialog (`None` when cancelled). The
/// token comes from [`folder_action`], which gives one on the desktop only.
pub fn pick_folder(token: FolderToken) -> Option<PathBuf> {
    let _ = token;
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        rfd::FileDialog::new().pick_folder()
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    {
        None
    }
}

/// Where the browser keeps the files of [`gather`] and [`zip_written`]
/// in memory, as if they were in folders.
const MEMORY_ROOT: &str = "/memory";

/// True for a path the browser keeps in memory ([`MEMORY_ROOT`]).
fn in_memory(path: &Path) -> bool {
    WEB && path.starts_with(MEMORY_ROOT)
}

/// The folder for the app's own work called `name`: under
/// [`MEMORY_ROOT`] in a browser, in the temporary folder elsewhere.
fn work_dir(name: &str) -> PathBuf {
    if WEB {
        return PathBuf::from(MEMORY_ROOT).join(name);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::temp_dir()
            .join(format!("tracedraw-work-{}", std::process::id()))
            .join(name)
    }
    #[cfg(target_arch = "wasm32")]
    {
        PathBuf::from(MEMORY_ROOT).join(name)
    }
}

/// Empty `dir` (made when missing; in a browser, its files forgotten).
fn clear_dir(dir: &Path) -> std::io::Result<()> {
    if WEB {
        STORE.with(|s| s.borrow_mut().retain(|p, _| !p.starts_with(dir)));
        return Ok(());
    }
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir)
}

/// Picked files put together in a folder of their own called `key`, for
/// code that reads a folder (Print Merge's pictures, the Picture Mosaic
/// library) where no folder can be picked (#1399): copies in the
/// temporary folder (Android), kept in memory in a browser. The files
/// the folder held before are gone. The folder.
pub fn gather(key: &str, paths: &[PathBuf]) -> std::io::Result<PathBuf> {
    let dir = work_dir(&format!("picked-{}", safe_file_name(key)));
    clear_dir(&dir)?;
    for path in paths {
        let name = path
            .file_name()
            .map(|n| safe_file_name(&n.to_string_lossy()))
            .unwrap_or_else(|| "file".into());
        let bytes = read(path)?;
        write(dir.join(name), bytes)?;
    }
    Ok(dir)
}

/// True for a folder [`gather`] made.
pub fn is_gathered(dir: &Path) -> bool {
    dir.parent() == Some(work_dir("").as_path())
        && dir
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("picked-"))
}

/// The files `fill` writes (with [`write`]) into the folder it is given,
/// as one zip file with each file under `top/`, in name order (#1399):
/// where no folder can be picked, the files go where the user says as
/// one file. The folder is gone afterwards.
pub fn zip_written(top: &str, fill: impl FnOnce(&Path)) -> Result<Vec<u8>, String> {
    static WORK: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = WORK.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let work = work_dir(&format!("zip-{n}"));
    let top = safe_file_name(top);
    let dir = work.join(&top);
    clear_dir(&dir).map_err(|e| e.to_string())?;
    fill(&dir);
    let mut files = list_dir(&dir);
    files.sort();
    let entries: Result<Vec<(String, Vec<u8>)>, String> = files
        .iter()
        .map(|p| {
            let name = p.file_name().unwrap_or_default().to_string_lossy();
            read(p)
                .map(|b| (format!("{top}/{name}"), b))
                .map_err(|e| e.to_string())
        })
        .collect();
    let _ = clear_dir(&work);
    if !WEB {
        let _ = std::fs::remove_dir_all(&work);
    }
    zip_entries(&entries?)
}

/// Named files as one zip file (deflated).
pub fn zip_entries(entries: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    use std::io::Write as _;
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in entries {
        zip.start_file(name.as_str(), options)
            .map_err(|e| e.to_string())?;
        zip.write_all(data).map_err(|e| e.to_string())?;
    }
    zip.finish()
        .map(|c| c.into_inner())
        .map_err(|e| e.to_string())
}

/// The files directly inside a folder (not its subfolders), in no
/// particular order; empty when it cannot be read, and in a browser
/// outside the auto-backup folder and the folders kept in memory.
pub fn list_dir(path: impl AsRef<Path>) -> Vec<PathBuf> {
    #[cfg(target_arch = "wasm32")]
    if in_web_backups(path.as_ref()) {
        return crate::web::backups::list(path.as_ref());
    }
    if in_memory(path.as_ref()) {
        let dir = path.as_ref();
        return STORE.with(|s| {
            s.borrow()
                .keys()
                .filter(|p| p.parent() == Some(dir))
                .cloned()
                .collect()
        });
    }
    if WEB {
        return Vec::new();
    }
    match std::fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// One entry of a folder listing (the Save Drawing dialog).
#[derive(Debug, Clone, PartialEq)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    /// Bytes (files).
    pub size: u64,
    /// Seconds since 1970, when known.
    pub modified: Option<i64>,
}

/// The folders and files directly inside `dir`: folders first, then by
/// name ignoring case; hidden ones (a leading dot) left out. Empty in a
/// browser or when the folder cannot be read.
pub fn list_entries(dir: impl AsRef<Path>) -> Vec<DirEntry> {
    if WEB {
        return Vec::new();
    }
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<DirEntry> = read
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                return None;
            }
            let path = e.path();
            // Follow links: a link to a folder lists as a folder.
            let meta = std::fs::metadata(&path).ok();
            let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());
            let size = meta.as_ref().map_or(0, |m| m.len());
            let modified = meta
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64);
            Some(DirEntry {
                name,
                path,
                is_dir,
                size,
                modified,
            })
        })
        .collect();
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    out
}

/// True for a folder on disk (never in a browser).
pub fn is_dir(path: impl AsRef<Path>) -> bool {
    !WEB && path.as_ref().is_dir()
}

/// Make a folder (nothing in a browser).
pub fn create_dir(path: impl AsRef<Path>) -> std::io::Result<()> {
    if WEB {
        return Ok(());
    }
    std::fs::create_dir(path)
}

/// Make a folder and the folders it is in (nothing in a browser).
pub fn create_dir_all(path: impl AsRef<Path>) -> std::io::Result<()> {
    if WEB {
        return Ok(());
    }
    std::fs::create_dir_all(path)
}

/// Rename a file or folder; one already called `to` is never replaced
/// (nothing in a browser).
pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    if WEB {
        return Ok(());
    }
    if to.as_ref().exists() {
        return Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
    }
    std::fs::rename(from, to)
}

/// A folder worth a shortcut in the Save Drawing dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Home,
    Desktop,
    Documents,
    Downloads,
    Pictures,
    /// A drive or the file system's root.
    Root,
    /// A mounted drive, memory card, network share or other volume
    /// (#1391): named after its folder.
    Drive,
}

/// The user's home folder.
pub fn home_dir() -> Option<PathBuf> {
    if WEB {
        return None;
    }
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .filter(|h| !h.is_empty())
        .map(PathBuf::from)
}

/// The shortcuts that exist on this computer: home, desktop, documents,
/// downloads and pictures (the names the system gives them, read from
/// the user's folder settings on Linux), then the drives (Windows) or
/// the root, then the mounted drives, memory cards and network shares
/// ([`mounted_drives`]). Empty in a browser.
pub fn places() -> Vec<(Place, PathBuf)> {
    let mut out = Vec::new();
    let Some(home) = home_dir() else {
        return out;
    };
    out.push((Place::Home, home.clone()));
    let xdg = xdg_user_dirs(&home);
    for (place, key, default) in [
        (Place::Desktop, "XDG_DESKTOP_DIR", "Desktop"),
        (Place::Documents, "XDG_DOCUMENTS_DIR", "Documents"),
        (Place::Downloads, "XDG_DOWNLOAD_DIR", "Downloads"),
        (Place::Pictures, "XDG_PICTURES_DIR", "Pictures"),
    ] {
        let dir = xdg
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, p)| p.clone())
            .unwrap_or_else(|| home.join(default));
        if dir.is_dir() && dir != home {
            out.push((place, dir));
        }
    }
    if cfg!(windows) {
        for letter in b'C'..=b'Z' {
            let root = PathBuf::from(format!("{}:\\", letter as char));
            if root.is_dir() {
                out.push((Place::Root, root));
            }
        }
    } else {
        out.push((Place::Root, PathBuf::from("/")));
    }
    for drive in mounted_drives() {
        if !out.iter().any(|(_, p)| *p == drive) {
            out.push((Place::Drive, drive));
        }
    }
    out
}

/// Where this computer puts mounted drives (#1391).
#[derive(Debug, Clone, Default)]
pub struct MountRoots {
    /// The system's table of mounts (`/proc/self/mounts` on Linux).
    pub mounts: Option<String>,
    /// Folders whose subfolders are mounted drives: `/media/$USER`,
    /// `/run/media/$USER`, `/media` and `/mnt` on Linux, `/Volumes` on
    /// macOS.
    pub parents: Vec<PathBuf>,
    /// The folder of the desktop's network shares and phones
    /// (`$XDG_RUNTIME_DIR/gvfs`).
    pub gvfs: Option<PathBuf>,
}

impl MountRoots {
    /// This computer's.
    pub fn system() -> Self {
        let mut roots = MountRoots::default();
        if WEB || ANDROID || cfg!(windows) {
            return roots;
        }
        if cfg!(target_os = "macos") {
            roots.parents.push(PathBuf::from("/Volumes"));
            return roots;
        }
        roots.mounts = std::fs::read_to_string("/proc/self/mounts").ok();
        let user = std::env::var("USER")
            .ok()
            .filter(|u| !u.is_empty())
            .or_else(|| {
                home_dir().and_then(|h| h.file_name().map(|n| n.to_string_lossy().to_string()))
            });
        if let Some(user) = user {
            roots.parents.push(Path::new("/media").join(&user));
            roots.parents.push(Path::new("/run/media").join(&user));
        }
        roots.parents.push(PathBuf::from("/media"));
        roots.parents.push(PathBuf::from("/mnt"));
        roots.gvfs = std::env::var_os("XDG_RUNTIME_DIR")
            .filter(|d| !d.is_empty())
            .map(|d| PathBuf::from(d).join("gvfs"));
        roots
    }
}

/// The mounted drives, memory cards and network shares of this computer,
/// for the Save Drawing dialog (#1391). Empty on Windows (its drive
/// letters are listed as roots), on Android and in a browser.
pub fn mounted_drives() -> Vec<PathBuf> {
    drives_in(&MountRoots::system())
}

/// The drives `roots` holds: the folders mounted under one of its
/// parents (from the mount table, and the parents' subfolders, so a
/// drive shows when the table cannot be read), then the network shares
/// under `gvfs`. Each once, in that order, sorted by name within each;
/// the parents themselves, the boot volume (a link to `/` in
/// `/Volumes`), hidden folders and folders that are not there are left
/// out.
pub fn drives_in(roots: &MountRoots) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mounted = roots
        .mounts
        .as_deref()
        .map(parse_mount_points)
        .unwrap_or_default();
    // Folders that are only parents of drives (/media holding $USER).
    let is_parent = |p: &Path| roots.parents.iter().any(|q| q == p);
    for parent in &roots.parents {
        let mut here: Vec<PathBuf> = mounted
            .iter()
            .filter(|m| m.parent() == Some(parent.as_path()))
            .cloned()
            .collect();
        for e in list_entries(parent) {
            if e.is_dir {
                here.push(e.path);
            }
        }
        here.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
        for p in here {
            let hidden = p
                .file_name()
                .is_none_or(|n| n.to_string_lossy().starts_with('.'));
            let boot = std::fs::read_link(&p).is_ok_and(|t| t == Path::new("/"));
            if hidden || boot || is_parent(&p) || !p.is_dir() || found.contains(&p) {
                continue;
            }
            found.push(p);
        }
    }
    if let Some(gvfs) = &roots.gvfs {
        for e in list_entries(gvfs) {
            if e.is_dir && !found.contains(&e.path) {
                found.push(e.path);
            }
        }
    }
    found
}

/// The mount points of a mount table (`/proc/self/mounts`: device,
/// mount point, type, ... per line; a space in a path is written `\040`).
pub fn parse_mount_points(text: &str) -> Vec<PathBuf> {
    text.lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(|p| PathBuf::from(unescape_mount(p)))
        .collect()
}

/// `\040` (space), `\011` (tab), `\012` (new line) and `\134`
/// (backslash) of a mount table.
fn unescape_mount(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\'
            && i + 3 < bytes.len()
            && bytes[i + 1..i + 4]
                .iter()
                .all(|b| (b'0'..=b'7').contains(b))
        {
            let n = bytes[i + 1..i + 4]
                .iter()
                .fold(0u32, |n, b| n * 8 + u32::from(b - b'0'));
            out.push(n as u8);
            i += 4;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A drive's name in the Save Drawing dialog: its folder's name; for a
/// network share of the desktop (`smb-share:server=nas,share=photos`)
/// "photos (nas)", or the server alone.
pub fn drive_name(path: &Path) -> String {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    let Some((_, fields)) = name.split_once(':') else {
        return name;
    };
    let field = |key: &str| {
        fields
            .split(',')
            .find_map(|f| f.strip_prefix(key)?.strip_prefix('='))
            .map(str::to_string)
    };
    match (field("share"), field("server").or_else(|| field("host"))) {
        (Some(share), Some(server)) => format!("{share} ({server})"),
        (None, Some(server)) => server,
        _ => name,
    }
}

/// The folders bookmarked in the desktop's file manager
/// (`~/.config/gtk-3.0/bookmarks`, else `~/.gtk-bookmarks`), with their
/// names, that exist. Empty on Windows, macOS, Android and in a browser.
pub fn desktop_bookmarks() -> Vec<(String, PathBuf)> {
    if WEB || ANDROID || cfg!(windows) || cfg!(target_os = "macos") {
        return Vec::new();
    }
    let Some(home) = home_dir() else {
        return Vec::new();
    };
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    [
        config.join("gtk-3.0/bookmarks"),
        home.join(".gtk-bookmarks"),
    ]
    .iter()
    .find_map(|f| std::fs::read_to_string(f).ok())
    .map(|text| parse_bookmarks(&text))
    .unwrap_or_default()
    .into_iter()
    .filter(|(_, p)| p.is_dir())
    .collect()
}

/// The `file://` lines of a bookmarks file (`file:///home/me/My%20Work
/// Work`: the address, then an optional name), with the name given or
/// the folder's own; other addresses (`sftp://`, `smb://`) are left out,
/// as they are no folders here.
pub fn parse_bookmarks(text: &str) -> Vec<(String, PathBuf)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (uri, label) = match line.split_once(' ') {
                Some((u, l)) => (u, l.trim()),
                None => (line, ""),
            };
            let path = PathBuf::from(percent_decode(uri.strip_prefix("file://")?));
            if !path.is_absolute() {
                return None;
            }
            let name = if label.is_empty() {
                path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string())
            } else {
                label.to_string()
            };
            Some((name, path))
        })
        .collect()
}

/// `%20` and the other `%XX` escapes of an address, as bytes (UTF-8).
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The folders used lately, newest first, for the Save Drawing dialog
/// (#1391): the last folder a dialog used, then the folders of the
/// recent drawings; each once, at most `max`, only those that exist.
pub fn recent_dirs(last_dir: Option<&Path>, recent_files: &[PathBuf], max: usize) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let dirs = last_dir.map(Path::to_path_buf).into_iter().chain(
        recent_files
            .iter()
            .filter_map(|f| f.parent().map(Path::to_path_buf)),
    );
    for d in dirs {
        if out.len() >= max {
            break;
        }
        if !d.as_os_str().is_empty() && !out.contains(&d) && is_dir(&d) {
            out.push(d);
        }
    }
    out
}

/// The `XDG_*_DIR` folders of `~/.config/user-dirs.dirs` (Linux), with
/// `$HOME` expanded.
fn xdg_user_dirs(home: &Path) -> Vec<(String, PathBuf)> {
    let Ok(text) = std::fs::read_to_string(home.join(".config/user-dirs.dirs")) else {
        return Vec::new();
    };
    parse_user_dirs(&text, home)
}

/// Read `KEY="$HOME/Folder"` lines.
pub(crate) fn parse_user_dirs(text: &str, home: &Path) -> Vec<(String, PathBuf)> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            let value = value.trim().trim_matches('"');
            let path = match value.strip_prefix("$HOME") {
                Some(rest) => home.join(rest.trim_start_matches('/')),
                None => PathBuf::from(value),
            };
            Some((key.trim().to_string(), path))
        })
        .collect()
}

/// Decode an image file (any format the image crate was built with).
pub fn open_image(path: impl AsRef<Path>) -> Result<image::DynamicImage, String> {
    let bytes = read(path).map_err(|e| e.to_string())?;
    image::load_from_memory(&bytes).map_err(|e| e.to_string())
}

/// Keep bytes under a synthetic path so [`read`] finds them (browser).
pub fn store(path: &Path, bytes: Vec<u8>) {
    STORE.with(|s| {
        s.borrow_mut().insert(path.to_path_buf(), bytes);
    });
}

/// Keep an uploaded file past the dialog that picked it (a profile that
/// is read again later): returns the path to read it from. Natively the
/// file stays where it is.
pub fn keep(path: &Path) -> PathBuf {
    if !WEB {
        return path.to_path_buf();
    }
    let kept = PathBuf::from("/kept").join(path.file_name().unwrap_or_default());
    if let Ok(bytes) = read(path) {
        store(&kept, bytes);
    }
    kept
}

/// Forget an uploaded file once nothing needs it any more.
pub fn forget(path: &Path) {
    STORE.with(|s| {
        s.borrow_mut().remove(path);
    });
}

/// Run the dialogs that finished since the last frame (browser, Android).
/// Called once per frame before the interface is drawn.
pub fn run_finished(app: &mut App) {
    #[cfg(target_os = "android")]
    android_pickers::run_answers(app);
    let jobs: Vec<(Job, PathBuf)> = FINISHED.with(|f| std::mem::take(&mut *f.borrow_mut()));
    for (job, path) in jobs {
        job(app, path.clone());
        forget(&path);
    }
}

/// The paths whose extension is one of `exts` (lower case; any case in
/// the name), all of them when `exts` is empty: a folder's files kept to
/// the types a dialog asks for (#1926).
#[cfg(any(target_os = "android", test))]
pub fn with_extensions(paths: Vec<PathBuf>, exts: &[String]) -> Vec<PathBuf> {
    if exts.is_empty() {
        return paths;
    }
    paths
        .into_iter()
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| exts.iter().any(|x| x.eq_ignore_ascii_case(e)))
        })
        .collect()
}

/// The Android document picker (Storage Access Framework): a request
/// shows the system's picker and remembers what to do with the answer,
/// which the activity hands back on a later frame (`android.rs`).
#[cfg(target_os = "android")]
mod android_pickers {
    use super::*;
    use crate::android::documents;

    /// What a request waits to do with the picked documents.
    enum Waiting {
        /// Run with the first document.
        One(Job),
        /// Run with every document.
        Many(Box<dyn FnOnce(&mut App, Vec<PathBuf>)>),
        /// Keep the first document for [`take_picked`] under this key.
        Into(String),
    }

    thread_local! {
        static WAITING: RefCell<HashMap<i32, Waiting>> = RefCell::new(HashMap::new());
        /// Documents other apps opened with Craft Corel, waiting for no
        /// dialog or question to be up (#646).
        static OPENED: RefCell<Vec<PathBuf>> = const { RefCell::new(Vec::new()) };
    }

    fn wait(request: i32, waiting: Waiting) {
        WAITING.with(|w| {
            w.borrow_mut().insert(request, waiting);
        });
    }

    fn forget(request: i32) {
        WAITING.with(|w| {
            w.borrow_mut().remove(&request);
        });
    }

    /// Show the picker for one document (or several) and run `then`.
    fn pick(multiple: bool, then: Waiting) {
        let request = documents::next_request();
        wait(request, then);
        if !documents::open(request, multiple) {
            forget(request);
        }
    }

    pub fn pick_one(then: Job) {
        pick(false, Waiting::One(then));
    }

    pub fn pick_many(then: Box<dyn FnOnce(&mut App, Vec<PathBuf>)>) {
        pick(true, Waiting::Many(then));
    }

    pub fn pick_into(key: String) {
        pick(false, Waiting::Into(key));
    }

    /// Show the system's folder picker and run `then` with the documents
    /// in the folder chosen (#1926).
    pub fn pick_tree(then: Box<dyn FnOnce(&mut App, Vec<PathBuf>)>) {
        let request = documents::next_request();
        wait(request, Waiting::Many(then));
        if !documents::open_tree(request) {
            forget(request);
        }
    }

    /// Show the picker for a place to save `name` and run `then` with the
    /// document made there.
    pub fn save(name: &str, then: Job) {
        let ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let request = documents::next_request();
        wait(request, Waiting::One(then));
        if !documents::create(request, mime_for_extension(ext), name) {
            forget(request);
        }
    }

    /// The paths of an answer's documents.
    fn paths(answer: &documents::Answer) -> Vec<PathBuf> {
        answer
            .picked
            .iter()
            .map(|p| document_path(&p.uri, &name_with_extension(&p.name, &p.mime)))
            .collect()
    }

    /// Run what the answers that arrived since the last frame were
    /// waited for; a cancelled picker runs nothing.
    pub fn run_answers(app: &mut App) {
        for answer in documents::take_answers() {
            // Documents another app opened with Craft Corel: each in a tab
            // of its own, as a file manager's do on the desktop (#646).
            if answer.request == documents::OPENED_ELSEWHERE {
                let paths = paths(&answer);
                OPENED.with(|o| o.borrow_mut().extend(paths));
                continue;
            }
            let Some(waiting) = WAITING.with(|w| w.borrow_mut().remove(&answer.request)) else {
                continue;
            };
            let paths = paths(&answer);
            match waiting {
                Waiting::One(job) => {
                    if let Some(path) = paths.into_iter().next() {
                        job(app, path);
                    }
                }
                Waiting::Many(job) => {
                    if !paths.is_empty() {
                        job(app, paths);
                    }
                }
                Waiting::Into(key) => {
                    if let Some(path) = paths.into_iter().next() {
                        PICKED.with(|p| {
                            p.borrow_mut().insert(key, path);
                        });
                    }
                }
            }
        }
        // Like the files later starts hand over on the desktop, they wait
        // while a dialog or a question is up or the drawings are closing
        // (instance::open_handed_over).
        if !app.closing_all && matches!(app.dialog, crate::ui::dialogs::Dialog::None) {
            for path in OPENED.with(|o| std::mem::take(&mut *o.borrow_mut())) {
                app.open_path(path);
            }
        }
    }
}

/// Files dropped on the window this frame: `then` runs with each one's
/// path, at once natively, on a later frame in a browser (where the bytes
/// are read first and kept under `/upload/<name>` like a picked file).
pub fn take_dropped(
    ctx: &egui::Context,
    app: &mut App,
    then: impl FnOnce(&mut App, PathBuf) + Clone + 'static,
) {
    let dropped = ctx.input(|i| i.raw.dropped_files.clone());
    for file in dropped {
        #[cfg(not(target_arch = "wasm32"))]
        then.clone()(app, file.path().to_path_buf());
        #[cfg(target_arch = "wasm32")]
        {
            let _ = &app;
            let then = then.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match file.bytes_async().await {
                    Ok(bytes) => {
                        let name = file.path().file_name().unwrap_or_default();
                        let path = PathBuf::from("/upload").join(name);
                        store(&path, bytes);
                        let job: Job = Box::new(then);
                        FINISHED.with(|f| f.borrow_mut().push((job, path)));
                        crate::web::request_repaint();
                    }
                    Err(e) => log::warn!("{}: {e}", file.path().display()),
                }
            });
        }
    }
}

/// The path picked for `key` by [`Dialog::pick_into`], once.
pub fn take_picked(key: &str) -> Option<PathBuf> {
    PICKED.with(|p| p.borrow_mut().remove(key))
}

/// Whether the file picker of a browser with this user agent, platform
/// and number of touch points filters by the system's file types rather
/// than by extension: on an iPhone, iPad or iPod and on Android, where
/// files whose extension the system has no type for (`.cdr`, `.cmx`,
/// `.tdraw` ...) can be greyed out once the page names the extensions it
/// takes (#1311). iPadOS browsers call themselves a Mac ("MacIntel",
/// "Macintosh") with a touch screen; Macs have no touch points. Desktop
/// browsers match the extensions themselves and keep the filters.
#[cfg(any(target_arch = "wasm32", test))]
pub fn picker_filters_by_type(user_agent: &str, platform: &str, touch_points: u32) -> bool {
    let named = ["iPhone", "iPad", "iPod"]
        .iter()
        .any(|d| user_agent.contains(d) || platform.starts_with(d));
    let mac = platform == "MacIntel" || user_agent.contains("Macintosh");
    named || (mac && touch_points > 1) || user_agent.contains("Android")
}

/// A file dialog description, the same in both builds.
#[derive(Debug, Clone, Default)]
pub struct Dialog {
    filters: Vec<(String, Vec<String>)>,
    file_name: Option<String>,
}

impl Dialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push((
            name.into(),
            extensions.iter().map(|e| e.to_string()).collect(),
        ));
        self
    }

    pub fn set_file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = Some(name.into());
        self
    }

    /// Pick a file and run `then` with its path: at once natively, on a
    /// later frame in a browser.
    pub fn pick_file(self, app: &mut App, then: impl FnOnce(&mut App, PathBuf) + 'static) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        if let Some(path) = self.native_pick() {
            then(app, path);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = app;
            self.web_pick(Box::new(then));
        }
        // The system's document picker; `then` runs when it answers.
        #[cfg(target_os = "android")]
        {
            let _ = app;
            android_pickers::pick_one(Box::new(then));
        }
    }

    /// Pick one or more files and run `then` with their paths: at once
    /// natively, on a later frame in a browser and on Android.
    /// Choose a folder where only files can be picked otherwise, and run
    /// `then` with the files in it the filters take (#1926). On Android
    /// the system's folder picker (`ACTION_OPEN_DOCUMENT_TREE`) asks for
    /// the folder and its files come as picked documents, on a later
    /// frame; in a browser, which has no folder picker, the files are
    /// picked several at once ([`Self::pick_files`]).
    pub fn pick_folder_files(
        self,
        app: &mut App,
        then: impl FnOnce(&mut App, Vec<PathBuf>) + 'static,
    ) {
        #[cfg(target_os = "android")]
        {
            let _ = app;
            let exts = self.extensions();
            android_pickers::pick_tree(Box::new(move |app: &mut App, paths: Vec<PathBuf>| {
                let paths = with_extensions(paths, &exts);
                if !paths.is_empty() {
                    then(app, paths);
                }
            }));
        }
        #[cfg(not(target_os = "android"))]
        self.pick_files(app, then);
    }

    /// Every extension the filters name, lower case.
    #[cfg(any(target_os = "android", test))]
    fn extensions(&self) -> Vec<String> {
        self.filters
            .iter()
            .flat_map(|(_, exts)| exts.iter().map(|e| e.to_ascii_lowercase()))
            .collect()
    }

    pub fn pick_files(self, app: &mut App, then: impl FnOnce(&mut App, Vec<PathBuf>) + 'static) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            let mut d = rfd::FileDialog::new();
            for (name, exts) in &self.filters {
                let exts: Vec<&str> = exts.iter().map(|e| e.as_str()).collect();
                d = d.add_filter(name, &exts);
            }
            if let Some(paths) = d.pick_files() {
                then(app, paths);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = app;
            self.web_pick_many(Box::new(then));
        }
        #[cfg(target_os = "android")]
        {
            let _ = app;
            android_pickers::pick_many(Box::new(then));
        }
    }

    /// Pick a file and keep its path for [`take_picked`]`(key)`.
    pub fn pick_into(self, key: &str) {
        let key = key.to_string();
        #[cfg(target_os = "android")]
        android_pickers::pick_into(key);
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        if let Some(path) = self.native_pick() {
            PICKED.with(|p| {
                p.borrow_mut().insert(key, path);
            });
        }
        // The upload outlives this job: the dialog reads it on a later frame.
        #[cfg(target_arch = "wasm32")]
        self.web_pick(Box::new(move |_app, path| {
            let kept = keep(&path);
            PICKED.with(|p| {
                p.borrow_mut().insert(key, kept);
            });
        }));
    }

    /// Choose where to save and run `then` with that path; `then` writes
    /// with [`write`]. A save dialog natively, at once; in a browser there
    /// is no dialog: the path is `/download/<file name>` and the write
    /// becomes a download; on Android the system's picker asks where,
    /// and `then` runs on a later frame with the document it made.
    pub fn save_file(self, app: &mut App, then: impl FnOnce(&mut App, PathBuf) + 'static) {
        #[cfg(target_os = "android")]
        {
            let _ = app;
            let name = self.suggested_name();
            android_pickers::save(&name, Box::new(then));
        }
        #[cfg(not(target_os = "android"))]
        if let Some(path) = self.save_path() {
            then(app, path);
        }
    }

    /// Save `bytes` where the user says ([`Dialog::save_file`]); a
    /// failed write is told on the status line.
    pub fn save_bytes(self, app: &mut App, bytes: impl Into<Vec<u8>>) {
        let bytes = bytes.into();
        self.save_file(app, move |app, path| {
            if let Err(e) = write(&path, bytes) {
                app.status = e.to_string();
            }
        });
    }

    /// The name offered for a new file: the one set, with the first
    /// filter's extension when it has none.
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    fn suggested_name(&self) -> String {
        let name = self
            .file_name
            .clone()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| "drawing".into());
        if Path::new(&name).extension().is_some() {
            return name;
        }
        match self.filters.first().and_then(|(_, exts)| exts.first()) {
            Some(ext) => format!("{name}.{ext}"),
            None => name,
        }
    }

    /// The path to save to, or `None` when the user cancels: a save
    /// dialog natively, `/download/<file name>` in a browser (write it with
    /// [`write`] to download it). On Android, where the picker answers on
    /// a later frame (see [`Dialog::save_file`]), the few callers that
    /// need a path at once get one in the app's own documents folder.
    pub fn save_path(self) -> Option<PathBuf> {
        if WEB {
            let name = self.file_name.unwrap_or_else(|| "download".into());
            return Some(PathBuf::from("/download").join(name));
        }
        #[cfg(target_os = "android")]
        {
            let name = self.suggested_name();
            let dir = android_documents_dir();
            if let Err(e) = std::fs::create_dir_all(&dir) {
                log::warn!("cannot create {}: {e}", dir.display());
            }
            log::info!("saved without a picker to {}", dir.display());
            return Some(dir.join(name));
        }
        #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
        {
            let mut d = rfd::FileDialog::new();
            for (name, exts) in &self.filters {
                let exts: Vec<&str> = exts.iter().map(|e| e.as_str()).collect();
                d = d.add_filter(name, &exts);
            }
            if let Some(n) = &self.file_name {
                d = d.set_file_name(n);
            }
            return d.save_file();
        }
        #[allow(unreachable_code)]
        None
    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    fn native_pick(&self) -> Option<PathBuf> {
        let mut d = rfd::FileDialog::new();
        for (name, exts) in &self.filters {
            let exts: Vec<&str> = exts.iter().map(|e| e.as_str()).collect();
            d = d.add_filter(name, &exts);
        }
        d.pick_file()
    }

    /// The browser's picker for several files: each is kept under
    /// `/upload/<name>` and `then` runs with them all on a later frame.
    #[cfg(target_arch = "wasm32")]
    fn web_pick_many(self, then: Box<dyn FnOnce(&mut App, Vec<PathBuf>)>) {
        let d = self.web_dialog();
        wasm_bindgen_futures::spawn_local(async move {
            let Some(handles) = d.pick_files().await else {
                return;
            };
            let mut paths = Vec::new();
            for handle in handles {
                let path = PathBuf::from("/upload").join(handle.file_name());
                store(&path, handle.read().await);
                paths.push(path);
            }
            if paths.is_empty() {
                return;
            }
            // One job for them all, under the first path; the others are
            // forgotten once it ran.
            let first = paths[0].clone();
            let job: Job = Box::new(move |app: &mut App, _| {
                let rest = paths.clone();
                then(app, paths);
                for p in rest.iter().skip(1) {
                    forget(p);
                }
            });
            FINISHED.with(|f| f.borrow_mut().push((job, first)));
            crate::web::request_repaint();
        });
    }

    /// The browser's picker with the filters, where they help.
    #[cfg(target_arch = "wasm32")]
    fn web_dialog(&self) -> rfd::AsyncFileDialog {
        let mut d = rfd::AsyncFileDialog::new();
        // On an iPhone, iPad or Android device the picker can grey out
        // every file whose extension the system does not know, `.cdr`
        // among them, as soon as the page names the extensions it takes:
        // there it offers every file instead (#1311).
        if !crate::web::picker_filters_by_type() {
            for (name, exts) in &self.filters {
                let exts: Vec<&str> = exts.iter().map(|e| e.as_str()).collect();
                d = d.add_filter(name, &exts);
            }
        }
        d
    }

    #[cfg(target_arch = "wasm32")]
    fn web_pick(self, then: Job) {
        let d = self.web_dialog();
        wasm_bindgen_futures::spawn_local(async move {
            let Some(handle) = d.pick_file().await else {
                return;
            };
            let name = handle.file_name();
            let bytes = handle.read().await;
            let path = PathBuf::from("/upload").join(name);
            store(&path, bytes);
            FINISHED.with(|f| f.borrow_mut().push((then, path)));
            crate::web::request_repaint();
        });
    }
}

/// Offer bytes as a browser download (no-op natively).
#[cfg(target_arch = "wasm32")]
fn download(name: &str, bytes: &[u8]) -> std::io::Result<()> {
    crate::web::download(name, bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

#[cfg(not(target_arch = "wasm32"))]
fn download(_name: &str, _bytes: &[u8]) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// iPhones and iPads (also iPadOS, which calls itself a Mac with a
    /// touch screen) and Android browsers get a picker without extension
    /// filters; Macs, Windows and Linux keep them (#1311).
    #[test]
    fn mobile_browsers_pick_without_extension_filters() {
        let ipad_os = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";
        assert!(picker_filters_by_type(ipad_os, "MacIntel", 5));
        assert!(!picker_filters_by_type(ipad_os, "MacIntel", 0), "a Mac");
        let iphone = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15";
        assert!(picker_filters_by_type(iphone, "iPhone", 5));
        let old_ipad = "Mozilla/5.0 (iPad; CPU OS 12_0 like Mac OS X) AppleWebKit/605.1.15";
        assert!(picker_filters_by_type(old_ipad, "iPad", 5));
        let android = "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 Chrome/130.0 Mobile Safari/537.36";
        assert!(picker_filters_by_type(android, "Linux armv8l", 5));
        let firefox_android =
            "Mozilla/5.0 (Android 14; Mobile; rv:131.0) Gecko/131.0 Firefox/131.0";
        assert!(picker_filters_by_type(firefox_android, "Linux aarch64", 5));
        let windows = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/130.0 Safari/537.36";
        assert!(
            !picker_filters_by_type(windows, "Win32", 10),
            "a touch laptop"
        );
        let linux = "Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0";
        assert!(!picker_filters_by_type(linux, "Linux x86_64", 0));
    }

    #[test]
    fn picked_paths_are_taken_once() {
        PICKED.with(|p| {
            p.borrow_mut().insert("k".into(), PathBuf::from("/x/a.csv"));
        });
        assert_eq!(take_picked("k"), Some(PathBuf::from("/x/a.csv")));
        assert_eq!(take_picked("k"), None);
    }

    #[test]
    fn finished_jobs_run_once_and_free_their_upload() {
        let mut app = App::headless();
        let path = PathBuf::from("/upload/t.txt");
        store(&path, b"hello".to_vec());
        FINISHED.with(|f| {
            f.borrow_mut().push((
                Box::new(|app: &mut App, p: PathBuf| {
                    app.status = format!("got {}", p.display());
                }),
                path.clone(),
            ))
        });
        run_finished(&mut app);
        assert_eq!(app.status, "got /upload/t.txt");
        run_finished(&mut app);
        assert_eq!(app.status, "got /upload/t.txt");
        assert!(STORE.with(|s| s.borrow().get(&path).is_none()));
    }

    /// A document the Android picker returned is named by its URI and its
    /// display name: the name gives the title and the type, the URI comes
    /// back whole from the path, however many slashes and percent signs
    /// it holds (#893).
    #[test]
    fn a_picked_document_keeps_its_uri_and_shows_its_name() {
        let uri = "content://com.android.externalstorage.documents/document/primary%3ADrawings%2Flogo.cdr";
        let path = document_path(uri, "logo.cdr");
        assert_eq!(path.file_name().unwrap(), "logo.cdr");
        assert_eq!(path.extension().unwrap(), "cdr");
        assert!(path.starts_with(DOCUMENT_ROOT));
        assert!(is_document_path(&path));
        assert_eq!(document_uri(&path).as_deref(), Some(uri));
        let slash_path = PathBuf::from("/content/content:%2F%2Fx%2Fdoc/logo.cdr");
        let native_path = PathBuf::from(DOCUMENT_ROOT)
            .join("content:%2F%2Fx%2Fdoc")
            .join("logo.cdr");
        for path in [&slash_path, &native_path] {
            assert_eq!(document_uri(path).as_deref(), Some("content://x/doc"));
            assert_eq!(document_uri(&path.join("unexpected-child")), None);
        }
        assert_eq!(canonical(&path), path);
        // A name with a slash cannot become a folder; an empty one is named.
        assert_eq!(
            document_path(uri, "a/b.svg").file_name().unwrap(),
            "a_b.svg"
        );
        assert_eq!(document_path(uri, "  ").file_name().unwrap(), "document");
        // Ordinary paths are not documents.
        assert_eq!(document_uri(Path::new("/home/a/content/x")), None);
        assert_eq!(document_uri(Path::new("/content")), None);
        assert!(!is_document_path(Path::new("/tmp/logo.cdr")));
    }

    /// A picked document without an extension gets one from its MIME
    /// type, and the save picker is told the type of the name it saves.
    #[test]
    fn names_and_mime_types_follow_each_other() {
        assert_eq!(name_with_extension("IMG_1", "image/jpeg"), "IMG_1.jpg");
        assert_eq!(
            name_with_extension("report", "application/pdf"),
            "report.pdf"
        );
        assert_eq!(name_with_extension("logo.cdr", ""), "logo.cdr");
        assert_eq!(
            name_with_extension("notes", "application/x-unknown"),
            "notes"
        );
        assert_eq!(mime_for_extension("pdf"), "application/pdf");
        assert_eq!(mime_for_extension(".SVG"), "image/svg+xml");
        assert_eq!(mime_for_extension("tdraw"), "application/octet-stream");
        assert_eq!(mime_for_extension("cdr"), "application/octet-stream");
        assert_eq!(
            Dialog::new()
                .add_filter("PDF", &["pdf"])
                .set_file_name("Graphic1")
                .suggested_name(),
            "Graphic1.pdf"
        );
        assert_eq!(
            Dialog::new()
                .add_filter("PDF", &["pdf"])
                .set_file_name("Graphic1.pdf")
                .suggested_name(),
            "Graphic1.pdf"
        );
        assert_eq!(Dialog::new().suggested_name(), "drawing");
    }

    /// Every folder button has one behaviour per platform (#1399): the
    /// desktop picks a folder; a browser and Android never do (no folder
    /// can be picked there), and get a zip for folders written into, the
    /// open picker for folders read, or a greyed button with its reason.
    #[test]
    fn folder_buttons_pick_a_folder_on_the_desktop_only() {
        use FolderAction::*;
        use FolderUse::*;
        for place in FolderUse::ALL {
            assert!(
                matches!(folder_action(place, Platform::Desktop), PickFolder(_)),
                "{place:?}"
            );
            assert_eq!(folder_tip(place, Platform::Desktop), None, "{place:?}");
            for platform in [Platform::Web, Platform::Android] {
                let action = folder_action(place, platform);
                assert!(!matches!(action, PickFolder(_)), "{place:?} {platform:?}");
                // Never silent: each says why it does what it does.
                let tip = folder_tip(place, platform).expect("a tip");
                assert!(crate::i18n::has_english(tip), "{tip}");
                if let Unavailable(key) = action {
                    assert_eq!(tip, key);
                }
            }
        }
        let table = |platform| {
            FolderUse::ALL
                .iter()
                .map(|&p| folder_action(p, platform))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            table(Platform::Web),
            [
                SaveZip,
                Unavailable("package.desktop_only"),
                OpenFiles,
                OpenFiles,
                OpenFiles,
                OpenFiles,
                Unavailable("folder.backup_web"),
                Unavailable("folder.backup_web"),
            ]
        );
        assert_eq!(
            table(Platform::Android),
            [
                SaveZip,
                SaveZip,
                OpenFiles,
                OpenFiles,
                OpenFiles,
                OpenFiles,
                Unavailable("folder.backup_android"),
                Unavailable("folder.backup_android"),
            ]
        );
        assert_eq!(
            folder_tip(ExportAssets, Platform::Web),
            Some("folder.zip_web")
        );
        assert_eq!(
            folder_tip(MosaicLibrary, Platform::Android),
            Some("folder.files_android")
        );
        assert_eq!(Platform::current(), Platform::Desktop);
        assert!(matches!(FontFolder.action(), PickFolder(_)));
        // Print Merge's pictures: the folder picker on Android (#1926).
        assert_eq!(
            folder_tip(MergePictures, Platform::Android),
            Some("folder.tree_android")
        );
        assert_eq!(
            folder_tip(MergePictures, Platform::Web),
            Some("folder.files_web")
        );
    }

    /// #1926: a folder's documents are kept to the types the dialog asks
    /// for (any case), all of them with no filter; the dialog's
    /// extensions are its filters', lower case. The Print Merge docker
    /// asks for a folder where only files could be picked, and on Android
    /// the folder picker answers like the open picker.
    #[test]
    fn a_folders_files_keep_to_the_dialogs_types() {
        let paths: Vec<PathBuf> = ["a.png", "b.JPG", "notes.txt", "c.tiff", "noext"]
            .iter()
            .map(PathBuf::from)
            .collect();
        let d = Dialog::new()
            .add_filter("Pictures", &["png", "JPG", "tiff"])
            .add_filter("More", &["webp"]);
        let exts = d.extensions();
        assert_eq!(exts, ["png", "jpg", "tiff", "webp"]);
        assert_eq!(
            with_extensions(paths.clone(), &exts),
            [
                PathBuf::from("a.png"),
                PathBuf::from("b.JPG"),
                PathBuf::from("c.tiff")
            ]
        );
        assert_eq!(with_extensions(paths.clone(), &[]), paths);
        let docker = include_str!("ui/print_merge_docker.rs");
        assert!(docker.contains(".pick_folder_files(app, |app, paths| {"));
        let files = include_str!("files.rs");
        assert!(files.contains("android_pickers::pick_tree("));
        assert!(files.contains("if !documents::open_tree(request) {"));
    }

    /// The app's sources outside this file, as (path, text).
    fn app_sources() -> Vec<(String, String)> {
        fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
            entries.sort_by_key(|e| e.path());
            for e in entries {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push((
                        p.display().to_string(),
                        std::fs::read_to_string(&p).unwrap(),
                    ));
                }
            }
        }
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut out = Vec::new();
        walk(&src, &mut out);
        out.retain(|(p, _)| !p.ends_with("files.rs"));
        out
    }

    /// No dialog shows the folder dialog on its own: each one asks
    /// `FolderUse::action` for the running platform, whose leave to pick
    /// a folder (the token [`pick_folder`] takes) exists on the desktop
    /// only. The six dialogs of #1399 and Options all go through it.
    #[test]
    fn no_dialog_picks_a_folder_without_asking_the_platform() {
        let sources = app_sources();
        for (path, text) in &sources {
            assert!(!text.contains("pick_folder()"), "{path}");
            assert!(!text.contains("folder_action("), "{path}");
            assert!(!text.contains("FolderToken"), "{path}");
            if text.contains("files::pick_folder(") {
                assert!(text.contains(".action()"), "{path}");
            }
        }
        for name in [
            "export_docker.rs",
            "package_dialog.rs",
            "font_manager.rs",
            "print_merge_docker.rs",
            "options.rs",
            "picture_mosaic_docker.rs",
            "export_assets.rs",
        ] {
            let (_, text) = sources
                .iter()
                .find(|(p, _)| p.ends_with(&format!("/{name}")))
                .expect(name);
            assert!(text.contains("FolderUse::"), "{name}");
        }
    }

    /// Presets, masks, scripts, macro exports and palettes are saved
    /// through `Dialog::save_file`, which asks the system's save picker
    /// on Android (#1402): `save_path`, which saves there without asking,
    /// is left to Export as PDF's Browse, which Android does not show.
    #[test]
    fn saves_ask_where_on_android() {
        for (path, text) in app_sources() {
            if text.contains(".save_path()") {
                assert!(path.ends_with("publish_pdf.rs"), "{path}");
            }
        }
    }

    /// Picked files gathered into a folder of their own read like any
    /// folder (Print Merge's pictures, the Picture Mosaic library), and a
    /// new pick replaces the files of the last (#1399).
    #[test]
    fn picked_files_gather_into_a_folder() {
        let src = std::env::temp_dir().join(format!("tracedraw-gather-src-{}", std::process::id()));
        std::fs::create_dir_all(&src).unwrap();
        let (a, b) = (src.join("a.png"), src.join("b c.jpg"));
        std::fs::write(&a, [1u8, 2]).unwrap();
        std::fs::write(&b, [3u8]).unwrap();
        let dir = gather("test pictures", &[a.clone(), b.clone()]).unwrap();
        assert!(is_gathered(&dir), "{}", dir.display());
        assert!(!is_gathered(&src));
        let mut names: Vec<String> = list_dir(&dir)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        names.sort();
        assert_eq!(names, ["a.png", "b c.jpg"]);
        assert_eq!(read(dir.join("a.png")).unwrap(), [1, 2]);
        let again = gather("test pictures", std::slice::from_ref(&b)).unwrap();
        assert_eq!(again, dir);
        assert_eq!(list_dir(&dir).len(), 1);
        assert!(gather("test pictures", &[src.join("missing.png")]).is_err());
        let _ = std::fs::remove_dir_all(&src);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The files written into the folder `zip_written` gives come back as
    /// one zip, each under the top folder, in name order; the folder is
    /// gone afterwards (#1399).
    #[test]
    fn written_files_come_back_as_one_zip() {
        let mut seen = PathBuf::new();
        let zip = zip_written("Flyer", |dir| {
            seen = dir.to_path_buf();
            write(dir.join("b.svg"), "<svg/>").unwrap();
            write(dir.join("a.png"), [9u8; 4]).unwrap();
        })
        .unwrap();
        assert!(!seen.exists(), "{}", seen.display());
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
        let names: Vec<String> = archive
            .file_names()
            .map(|n| n.unwrap().to_string())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(sorted, ["Flyer/a.png", "Flyer/b.svg"]);
        let mut svg = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("Flyer/b.svg").unwrap(), &mut svg)
            .unwrap();
        assert_eq!(svg, "<svg/>");
    }

    #[test]
    fn native_read_and_write_use_the_file_system() {
        let dir = std::env::temp_dir().join(format!("tracedraw-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("a.bin");
        write(&p, [1u8, 2, 3]).unwrap();
        assert_eq!(read(&p).unwrap(), vec![1, 2, 3]);
        assert!(read(dir.join("missing")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The Save Drawing dialog lists the mounted drives (#1391): those
    /// under /media/$USER, /run/media/$USER and /mnt (from the mount table,
    /// a space written `\040`, and from the folders there), then the
    /// desktop's network shares, each once, without hidden folders or the
    /// parents themselves.
    #[test]
    fn mounted_drives_memory_cards_and_shares_are_listed() {
        let root = std::env::temp_dir().join(format!("tracedraw-drives-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let media = root.join("media");
        let user_media = media.join("ana");
        let run_media = root.join("run/media/ana");
        let mnt = root.join("mnt");
        let gvfs = root.join("gvfs");
        for d in [
            user_media.join("USB STICK"),
            user_media.join(".hidden"),
            run_media.join("CARD"),
            mnt.join("backup"),
            gvfs.join("smb-share:server=nas,share=photos"),
        ] {
            std::fs::create_dir_all(&d).expect("folder");
        }
        let table = format!(
            "/dev/sda1 / ext4 rw 0 0\n/dev/sdb1 {}/USB\\040STICK vfat rw 0 0\nproc /proc proc rw 0 0\n",
            user_media.display()
        );
        assert_eq!(
            parse_mount_points(&table)[1],
            user_media.join("USB STICK"),
            "a space in a mount point"
        );
        let roots = MountRoots {
            mounts: Some(table),
            parents: vec![
                user_media.clone(),
                run_media.clone(),
                media.clone(),
                mnt.clone(),
            ],
            gvfs: Some(gvfs.clone()),
        };
        assert_eq!(
            drives_in(&roots),
            vec![
                user_media.join("USB STICK"),
                run_media.join("CARD"),
                mnt.join("backup"),
                gvfs.join("smb-share:server=nas,share=photos"),
            ]
        );
        assert_eq!(drive_name(&user_media.join("USB STICK")), "USB STICK");
        assert_eq!(
            drive_name(&gvfs.join("smb-share:server=nas,share=photos")),
            "photos (nas)"
        );
        assert_eq!(drive_name(&gvfs.join("mtp:host=Phone_123")), "Phone_123");
        // Nothing mounted, nothing there: no drives.
        assert!(drives_in(&MountRoots::default()).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The desktop's bookmarks: `file://` addresses with `%20` escapes
    /// and an optional name; other addresses are no folders here (#1391).
    #[test]
    fn desktop_bookmarks_are_read() {
        assert_eq!(
            parse_bookmarks(
                "file:///home/ana/My%20Work Work\nfile:///srv/print\nsftp://host/x Remote\n\n"
            ),
            vec![
                ("Work".to_string(), PathBuf::from("/home/ana/My Work")),
                ("print".to_string(), PathBuf::from("/srv/print")),
            ]
        );
    }

    /// Recent folders: the last folder used, then the folders of the
    /// recent drawings, each once, only those that exist, at most `max`.
    #[test]
    fn recent_folders_come_from_the_last_folder_and_recent_drawings() {
        let root = std::env::temp_dir().join(format!("tracedraw-recent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (a, b, c) = (root.join("a"), root.join("b"), root.join("c"));
        for d in [&a, &b, &c] {
            std::fs::create_dir_all(d).expect("folder");
        }
        let recent = vec![
            b.join("one.tdraw"),
            a.join("two.cdr"),
            root.join("gone/three.tdraw"),
            c.join("four.tdraw"),
        ];
        assert_eq!(
            recent_dirs(Some(&a), &recent, 6),
            vec![a.clone(), b.clone(), c.clone()]
        );
        assert_eq!(recent_dirs(None, &recent, 2), vec![b, a]);
        let _ = std::fs::remove_dir_all(&root);
    }
}
