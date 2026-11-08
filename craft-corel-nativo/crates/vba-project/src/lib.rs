//! Reading VBA macro projects so their macros can run in TraceDraw: the
//! compound files drawing suites keep their macros in (`.gms` and any
//! other file holding a VBA project storage), and the module files the
//! VBA editor exports (`.bas`, `.cls`, `.frm` with its `.frx`). A
//! [`Project`] has the modules' source, the forms with their controls,
//! the references and whether the project is protected. Projects can be
//! written back as module files ([`export_project`]) or as a compound
//! file ([`write_gms`]).
//!
//! The layouts follow the public MS-OVBA and MS-OFORMS specifications.
//! See docs/behavior/vba.md.

pub mod codepage;
pub mod dir;
pub mod forms;
pub mod frm;
pub mod ovba;
pub mod project_stream;

pub use forms::{Control, ControlKind, Font, Form, OleColor};
pub use project_stream::Protection;

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

/// Why a project could not be read.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not a VBA project: {0}")]
    NotAProject(String),
    #[error("damaged VBA project: {0}")]
    Corrupt(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// The kind of a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKind {
    /// A standard (procedural) module: public subs are macros.
    Standard,
    Class,
    /// The code behind a form.
    Form,
    /// The code behind the host document (`ThisMacroStorage`,
    /// `ThisDocument`...).
    Document,
}

/// One module.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub kind: ModuleKind,
    /// The code with its `Attribute` lines; empty when [`has_source`] is
    /// false.
    ///
    /// [`has_source`]: Module::has_source
    pub source: String,
    /// The file kept the code. A project saved with only compiled code
    /// (p-code) has none to read.
    pub has_source: bool,
    pub read_only: bool,
    pub private: bool,
}

impl Module {
    /// The code without its `Attribute` lines.
    pub fn code(&self) -> String {
        self.source
            .lines()
            .filter(|l| !l.trim_start().starts_with("Attribute VB_"))
            .collect::<Vec<_>>()
            .join("\r\n")
    }

    /// The public subs without arguments: what can run as a macro.
    pub fn macros(&self) -> Vec<String> {
        if self.kind != ModuleKind::Standard {
            return Vec::new();
        }
        let mut out = Vec::new();
        for line in self.source.lines() {
            let l = line.trim();
            let lower = l.to_ascii_lowercase();
            let rest = if let Some(r) = lower.strip_prefix("public sub ") {
                &l[l.len() - r.len()..]
            } else if let Some(r) = lower.strip_prefix("sub ") {
                &l[l.len() - r.len()..]
            } else {
                continue;
            };
            let Some((name, args)) = rest.split_once('(') else {
                continue;
            };
            let args = args.split(')').next().unwrap_or("").trim();
            if args.is_empty() {
                out.push(name.trim().to_string());
            }
        }
        out
    }
}

/// A VBA project.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Project {
    pub name: String,
    pub modules: Vec<Module>,
    pub forms: Vec<Form>,
    /// Any protection: a password or locked for viewing. The code of a
    /// protected project is still readable; the protection only asks
    /// editors to hide it.
    pub protected: bool,
    pub protection: Protection,
    /// The names of the libraries and projects it refers to.
    pub references: Vec<String>,
    /// The Windows code page of its text.
    pub code_page: u16,
    pub help_file: String,
    pub description: String,
    /// What could not be read (a module without code, a damaged form...).
    pub warnings: Vec<String>,
}

impl Project {
    /// The module named `name` (any case).
    pub fn module(&self, name: &str) -> Option<&Module> {
        self.modules
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
    }

    /// The form named `name` (any case).
    pub fn form(&self, name: &str) -> Option<&Form> {
        self.forms
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))
    }

    /// Modules saved without their code (compiled only).
    pub fn modules_without_source(&self) -> Vec<&str> {
        self.modules
            .iter()
            .filter(|m| !m.has_source)
            .map(|m| m.name.as_str())
            .collect()
    }

    /// Every macro as (module, sub).
    pub fn macros(&self) -> Vec<(String, String)> {
        self.modules
            .iter()
            .flat_map(|m| m.macros().into_iter().map(move |s| (m.name.clone(), s)))
            .collect()
    }
}

/// The streams of a compound file, by path ("/VBA/dir").
fn all_streams(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, Error> {
    let mut cf = cfb::CompoundFile::open(Cursor::new(bytes))
        .map_err(|e| Error::NotAProject(format!("not a compound file ({e})")))?;
    let paths: Vec<String> = cf
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_string_lossy().replace('\\', "/"))
        .collect();
    let mut out = BTreeMap::new();
    let mut total = 0usize;
    for p in paths {
        let mut s = cf
            .open_stream(&p)
            .map_err(|e| Error::Corrupt(format!("{p}: {e}")))?;
        let mut buf = Vec::new();
        s.read_to_end(&mut buf)
            .map_err(|e| Error::Corrupt(format!("{p}: {e}")))?;
        total += buf.len();
        if total > 512 * 1024 * 1024 {
            return Err(Error::Corrupt("streams too large".into()));
        }
        out.insert(p, buf);
    }
    Ok(out)
}

/// Read a VBA project from a compound file (`.gms` or any file with a
/// `VBA` storage holding a `dir` stream).
pub fn read_gms(bytes: &[u8]) -> Result<Project, Error> {
    let streams = all_streams(compound_part(bytes))?;
    project_from_streams(&streams)
}

/// The compound file inside a macro project file. Files saved by drawing
/// suites start with a short header ("GMS", a version byte and a few
/// little-endian fields, 18 bytes in the files seen) before the compound
/// file; plain compound files start with its signature.
pub fn compound_part(bytes: &[u8]) -> &[u8] {
    const MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    if bytes.starts_with(&MAGIC) || !bytes.starts_with(b"GMS") {
        return bytes;
    }
    bytes
        .windows(MAGIC.len())
        .take(512)
        .position(|w| w == MAGIC)
        .map_or(bytes, |at| &bytes[at..])
}

fn project_from_streams(streams: &BTreeMap<String, Vec<u8>>) -> Result<Project, Error> {
    // The project's root holds the VBA storage.
    let dir_path = streams
        .keys()
        .filter(|p| p.to_ascii_lowercase().ends_with("/vba/dir"))
        .min_by_key(|p| p.len())
        .ok_or_else(|| Error::NotAProject("no VBA storage with a dir stream".into()))?
        .clone();
    let vba = &dir_path[..dir_path.len() - "dir".len()];
    let root = &vba[..vba.len() - "VBA/".len()];
    let dir = dir::parse(&ovba::decompress(&streams[&dir_path])?)?;
    let cp = dir.code_page;
    let info = streams
        .get(&format!("{root}PROJECT"))
        .map(|b| project_stream::parse(&codepage::decode(b, cp)))
        .unwrap_or_default();
    let mut project = Project {
        name: if dir.name.is_empty() {
            info.name.clone()
        } else {
            dir.name.clone()
        },
        protected: info.protection.any(),
        protection: info.protection,
        references: dir.references.iter().map(|r| r.name.clone()).collect(),
        code_page: cp,
        help_file: if dir.help_file.is_empty() {
            info.help_file.clone()
        } else {
            dir.help_file.clone()
        },
        description: dir.doc_string.clone(),
        ..Default::default()
    };
    let listed = |list: &[String], name: &str| list.iter().any(|n| n.eq_ignore_ascii_case(name));
    for m in &dir.modules {
        let kind = if listed(&info.forms, &m.name) {
            ModuleKind::Form
        } else if listed(&info.documents, &m.name) {
            ModuleKind::Document
        } else if listed(&info.classes, &m.name) {
            ModuleKind::Class
        } else if listed(&info.standard, &m.name) || m.procedural {
            ModuleKind::Standard
        } else {
            ModuleKind::Class
        };
        let stream = streams
            .iter()
            .find(|(p, _)| {
                p.len() == vba.len() + m.stream_name.len()
                    && p.starts_with(vba)
                    && p[vba.len()..].eq_ignore_ascii_case(&m.stream_name)
            })
            .map(|(_, b)| b);
        let (source, has_source) = match stream {
            None => {
                project.warnings.push(format!(
                    "{}: its stream {} is missing",
                    m.name, m.stream_name
                ));
                (String::new(), false)
            }
            Some(bytes) => {
                let at = m.text_offset as usize;
                match bytes.get(at..) {
                    Some(rest) if rest.len() > 1 => match ovba::decompress(rest) {
                        Ok(text) => (codepage::decode(&text, cp), true),
                        Err(e) => {
                            project.warnings.push(format!("{}: {e}", m.name));
                            (String::new(), false)
                        }
                    },
                    _ => {
                        project.warnings.push(format!(
                            "{}: saved as compiled code only, without its source",
                            m.name
                        ));
                        (String::new(), false)
                    }
                }
            }
        };
        if kind == ModuleKind::Form {
            let prefix = format!("{root}{}/", m.stream_name);
            let designer: forms::Streams = streams
                .iter()
                .filter(|(p, _)| {
                    p.len() > prefix.len() && p[..prefix.len()].eq_ignore_ascii_case(&prefix)
                })
                .map(|(p, b)| (p[prefix.len()..].to_string(), b.clone()))
                .collect();
            if designer.is_empty() {
                project.warnings.push(format!(
                    "{}: the form's designer storage is missing",
                    m.name
                ));
            } else {
                let form = forms::read_form(&m.name, designer);
                project
                    .warnings
                    .extend(form.warnings.iter().map(|w| format!("{}: {w}", m.name)));
                project.forms.push(form);
            }
        }
        project.modules.push(Module {
            name: m.name.clone(),
            kind,
            source,
            has_source,
            read_only: m.read_only,
            private: m.private,
        });
    }
    if project.modules.is_empty() {
        project.warnings.push("the project has no modules".into());
    }
    Ok(project)
}

/// Text of a module file: UTF-8 when it is, else the system code page the
/// file was exported in, guessed from its bytes (Cyrillic, Central
/// European, Greek and the others); `cp` when the guess is the Western
/// default and `cp` names another code page.
fn file_text(bytes: &[u8], cp: u16) -> String {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let guess = detector.guess(None, false);
    let enc = if guess == encoding_rs::WINDOWS_1252 && cp != 1252 {
        codepage::encoding(cp)
    } else {
        guess
    };
    enc.decode_without_bom_handling(bytes).0.into_owned()
}

/// Add one module file (`.bas`, `.cls`, `.frm`) to `project`. `frx` is
/// the `.frx` file of a form, when there is one.
pub fn add_module_file(project: &mut Project, file_name: &str, bytes: &[u8], frx: Option<&[u8]>) {
    let text = file_text(bytes, project.code_page.max(1));
    let m = frm::parse_module(file_name, &text);
    if m.kind == ModuleKind::Form {
        let mut form = match &m.form {
            Some(b) => frm::form_from_block(&m.name, b),
            None => Form {
                name: m.name.clone(),
                caption: m.name.clone(),
                enabled: true,
                ..Default::default()
            },
        };
        let blob = m
            .form
            .as_ref()
            .and_then(|b| b.frx.get("oleobjectblob").cloned());
        if let (Some((_, offset)), Some(frx)) = (blob.as_ref(), frx) {
            match frx.get(*offset as usize..).map(all_streams) {
                Some(Ok(streams)) => {
                    let designer: forms::Streams = streams
                        .into_iter()
                        .map(|(p, b)| (p.trim_start_matches('/').to_string(), b))
                        .collect();
                    let caption = form.caption.clone();
                    let (w, h) = (form.width, form.height);
                    let mut read = forms::read_form(&m.name, designer);
                    if read.caption == read.name && !caption.is_empty() {
                        read.caption = caption;
                    }
                    if read.width == 0.0 {
                        read.width = w;
                        read.height = h;
                    }
                    form = read;
                }
                Some(Err(e)) => form.warnings.push(format!("the .frx file: {e}")),
                None => form
                    .warnings
                    .push("the .frx file is shorter than its offset".into()),
            }
        } else if blob.is_some() {
            form.warnings
                .push("its controls are in the .frx file, which was not given".into());
        }
        project
            .warnings
            .extend(form.warnings.iter().map(|w| format!("{}: {w}", m.name)));
        project
            .forms
            .retain(|f| !f.name.eq_ignore_ascii_case(&m.name));
        project.forms.push(form);
    }
    project
        .modules
        .retain(|x| !x.name.eq_ignore_ascii_case(&m.name));
    project.modules.push(Module {
        name: m.name,
        kind: m.kind,
        source: m.source,
        has_source: true,
        read_only: false,
        private: false,
    });
}

/// Read module files given as (file name, bytes): `.bas`, `.cls`, `.frm`
/// with their `.frx` files, and whole projects (`.gms` and other compound
/// files), merged into one project named `name`.
pub fn read_named(name: &str, files: &[(String, Vec<u8>)]) -> Result<Project, Error> {
    let mut project = Project {
        name: name.to_string(),
        code_page: 1252,
        ..Default::default()
    };
    let ext = |n: &str| n.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    for (file, bytes) in files {
        match ext(file).as_str() {
            "bas" | "cls" | "frm" => {
                let stem = &file[..file.len() - 4];
                let frx = files
                    .iter()
                    .find(|(n, _)| {
                        n.len() == stem.len() + 4 && n[..stem.len()] == *stem && ext(n) == "frx"
                    })
                    .map(|(_, b)| b.as_slice());
                add_module_file(&mut project, file, bytes, frx);
            }
            "frx" => {}
            _ => {
                let other = read_gms(bytes)?;
                if project.modules.is_empty() && files.len() == 1 {
                    return Ok(other);
                }
                project.protected |= other.protected;
                project.references.extend(other.references);
                project.warnings.extend(other.warnings);
                project.forms.extend(other.forms);
                project.modules.extend(other.modules);
            }
        }
    }
    if project.modules.is_empty() {
        return Err(Error::NotAProject("no modules in the files given".into()));
    }
    Ok(project)
}

/// Read files or folders: a compound file, or module files (a folder
/// stands for the module files in it). The project takes the name of the
/// first folder or file.
pub fn read_files<P: AsRef<Path>>(paths: &[P]) -> Result<Project, Error> {
    let mut files = Vec::new();
    let mut name = String::new();
    for p in paths {
        let p = p.as_ref();
        if name.is_empty() {
            name = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Project".into());
        }
        if p.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(p)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|e| {
                    let x = e
                        .extension()
                        .map(|x| x.to_string_lossy().to_ascii_lowercase())
                        .unwrap_or_default();
                    matches!(x.as_str(), "bas" | "cls" | "frm" | "frx")
                })
                .collect();
            entries.sort();
            for e in entries {
                files.push((e.to_string_lossy().to_string(), std::fs::read(&e)?));
            }
        } else {
            files.push((p.to_string_lossy().to_string(), std::fs::read(p)?));
        }
    }
    read_named(&name, &files)
}

// ----- Writing --------------------------------------------------------------

/// A file to write: its name and bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportedFile {
    pub name: String,
    pub bytes: Vec<u8>,
}

fn crlf(text: &str) -> String {
    let mut out = text.replace("\r\n", "\n").replace('\n', "\r\n");
    if !out.ends_with("\r\n") && !out.is_empty() {
        out.push_str("\r\n");
    }
    out
}

/// The module's source, with an `Attribute VB_Name` line when it has
/// none (and the class attributes for classes).
fn with_attributes(m: &Module) -> String {
    let mut head = String::new();
    if frm::vb_name(&m.source).is_none() {
        head += &format!("Attribute VB_Name = \"{}\"\r\n", m.name);
        if matches!(m.kind, ModuleKind::Class | ModuleKind::Document) {
            head += "Attribute VB_GlobalNameSpace = False\r\nAttribute VB_Creatable = False\r\nAttribute VB_PredeclaredId = False\r\nAttribute VB_Exposed = False\r\n";
        }
    }
    crlf(&(head + &m.source))
}

/// A compound file holding `streams` (paths relative to the root).
fn compound(streams: &BTreeMap<String, Vec<u8>>) -> Result<Vec<u8>, Error> {
    let mut cf = cfb::CompoundFile::create(Cursor::new(Vec::new()))?;
    for (path, data) in streams {
        let full = format!("/{}", path.trim_start_matches('/'));
        if let Some(parent) = Path::new(&full).parent() {
            if parent != Path::new("/") {
                cf.create_storage_all(parent)?;
            }
        }
        let mut s = cf.create_stream(&full)?;
        s.write_all(data)?;
    }
    cf.flush()?;
    Ok(cf.into_inner().into_inner())
}

/// The files of one module: `.bas`, `.cls`, or `.frm` with its `.frx`.
pub fn export_module(project: &Project, m: &Module) -> Result<Vec<ExportedFile>, Error> {
    let cp = project.code_page.max(1);
    let source = with_attributes(m);
    Ok(match m.kind {
        ModuleKind::Standard => vec![ExportedFile {
            name: format!("{}.bas", m.name),
            bytes: codepage::encode(&source, cp),
        }],
        ModuleKind::Class | ModuleKind::Document => {
            let text =
                format!("VERSION 1.0 CLASS\r\nBEGIN\r\n  MultiUse = -1  'True\r\nEND\r\n{source}");
            vec![ExportedFile {
                name: format!("{}.cls", m.name),
                bytes: codepage::encode(&text, cp),
            }]
        }
        ModuleKind::Form => {
            let form = project.form(&m.name);
            let caption = form
                .map(|f| f.caption.clone())
                .unwrap_or_else(|| m.name.clone());
            let (w, h) = form.map(|f| (f.width, f.height)).unwrap_or((240.0, 180.0));
            let quoted = caption.replace('"', "\"\"");
            let text = format!(
                "VERSION 5.00\r\nBegin {} {} \r\n   Caption         =   \"{quoted}\"\r\n   ClientHeight    =   {}\r\n   ClientLeft      =   45\r\n   ClientTop       =   390\r\n   ClientWidth     =   {}\r\n   OleObjectBlob   =   \"{}.frx\":0000\r\n   StartUpPosition =   1  'CenterOwner\r\nEnd\r\n{source}",
                frm::FORMS_FORM,
                m.name,
                (h * 20.0).round(),
                (w * 20.0).round(),
                m.name,
            );
            let streams: BTreeMap<String, Vec<u8>> = match form {
                Some(f) if !f.raw.is_empty() => f.raw.iter().cloned().collect(),
                Some(f) => forms::write_form(f),
                None => forms::write_form(&Form {
                    name: m.name.clone(),
                    caption: caption.clone(),
                    width: w,
                    height: h,
                    enabled: true,
                    ..Default::default()
                }),
            };
            vec![
                ExportedFile {
                    name: format!("{}.frm", m.name),
                    bytes: codepage::encode(&text, cp),
                },
                ExportedFile {
                    name: format!("{}.frx", m.name),
                    bytes: compound(&streams)?,
                },
            ]
        }
    })
}

/// The files of every module of `project`.
pub fn export_project(project: &Project) -> Result<Vec<ExportedFile>, Error> {
    let mut out = Vec::new();
    for m in &project.modules {
        out.extend(export_module(project, m)?);
    }
    Ok(out)
}

/// The `PROJECTwm` stream: each module's name in the code page and in
/// UTF-16.
fn project_wm(project: &Project) -> Vec<u8> {
    let mut out = Vec::new();
    for m in &project.modules {
        out.extend(codepage::encode(&m.name, project.code_page.max(1)));
        out.push(0);
        out.extend(m.name.encode_utf16().flat_map(|u| u.to_le_bytes()));
        out.extend([0, 0]);
    }
    out.extend([0, 0]);
    out
}

/// Write `project` as a compound file with a VBA project storage, the
/// layout of `.gms` files: the source of every module (no compiled code,
/// so the host compiles it when it loads it), the PROJECT stream and the
/// forms' designer storages.
pub fn write_gms(project: &Project) -> Result<Vec<u8>, Error> {
    let cp = project.code_page.max(1);
    let mut streams: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut w = dir::DirWriter::new(&project.name, cp);
    w.reference("stdole", "*\\G{00020430-0000-0000-C000-000000000046}#2.0#0#C:\\Windows\\System32\\stdole2.tlb#OLE Automation");
    w.modules(project.modules.len() as u16);
    for m in &project.modules {
        w.module(&m.name, &m.name, 0, m.kind == ModuleKind::Standard);
        let source = with_attributes(m);
        streams.insert(
            format!("VBA/{}", m.name),
            ovba::compress(&codepage::encode(&source, cp)),
        );
    }
    streams.insert("VBA/dir".into(), ovba::compress(&w.finish()));
    // Header with version 0xFFFF: no compiled code to trust.
    streams.insert(
        "VBA/_VBA_PROJECT".into(),
        vec![0xCC, 0x61, 0xFF, 0xFF, 0x00, 0x00, 0x00],
    );
    let kind_names = |k: ModuleKind| -> Vec<String> {
        project
            .modules
            .iter()
            .filter(|m| m.kind == k)
            .map(|m| m.name.clone())
            .collect()
    };
    let info = project_stream::ProjectInfo {
        id: "{00000000-0000-0000-0000-000000000000}".into(),
        name: project.name.clone(),
        standard: kind_names(ModuleKind::Standard),
        classes: kind_names(ModuleKind::Class),
        forms: kind_names(ModuleKind::Form),
        documents: kind_names(ModuleKind::Document),
        protection: project.protection,
        ..Default::default()
    };
    streams.insert(
        "PROJECT".into(),
        codepage::encode(&project_stream::write(&info), cp),
    );
    streams.insert("PROJECTwm".into(), project_wm(project));
    for f in &project.forms {
        let designer: BTreeMap<String, Vec<u8>> = if f.raw.is_empty() {
            forms::write_form(f)
        } else {
            f.raw.iter().cloned().collect()
        };
        for (p, b) in designer {
            streams.insert(format!("{}/{p}", f.name), b);
        }
    }
    compound(&streams)
}

#[cfg(test)]
mod tests;
