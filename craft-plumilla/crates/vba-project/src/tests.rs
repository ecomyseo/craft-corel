//! Whole projects: compound files built stream by stream as a VBA host
//! lays them out, read back; module files; writing and reading again.

use super::*;
use std::collections::BTreeMap;

const MODULE1: &str = "Attribute VB_Name = \"Module1\"\r\n\
Option Explicit\r\n\
\r\n\
Public Sub MakeGrid()\r\n\
    Dim i As Integer\r\n\
    For i = 1 To 5\r\n\
        ActiveLayer.CreateRectangle i, i, i + 1, i + 1\r\n\
    Next i\r\n\
End Sub\r\n\
\r\n\
Sub ShowForm()\r\n\
    frmGrid.Show\r\n\
End Sub\r\n\
\r\n\
Private Sub Helper(x As Long)\r\n\
End Sub\r\n";

const FORM_CODE: &str = "Attribute VB_Name = \"frmGrid\"\r\n\
Attribute VB_Base = \"0{6D9F49DC-1111-2222-3333-444444444444}{55555555-6666-7777-8888-999999999999}\"\r\n\
Attribute VB_GlobalNameSpace = False\r\n\
Attribute VB_Creatable = False\r\n\
Attribute VB_PredeclaredId = True\r\n\
Attribute VB_Exposed = False\r\n\
Private Sub cmdOK_Click()\r\n\
    Unload Me\r\n\
End Sub\r\n";

fn grid_form() -> Form {
    let mut label = Control::new(ControlKind::Label, "lblCount");
    label.caption = "Rows:".into();
    label.left = 6.0;
    label.top = 6.0;
    label.width = 40.0;
    label.height = 12.0;
    let mut text = Control::new(ControlKind::TextBox, "txtCount");
    text.value = "5".into();
    text.left = 50.0;
    text.top = 6.0;
    text.width = 40.0;
    text.height = 18.0;
    let mut ok = Control::new(ControlKind::CommandButton, "cmdOK");
    ok.caption = "OK".into();
    ok.left = 50.0;
    ok.top = 40.0;
    ok.width = 50.0;
    ok.height = 22.0;
    Form {
        name: "frmGrid".into(),
        caption: "Grid".into(),
        width: 160.0,
        height: 90.0,
        enabled: true,
        controls: vec![label, text, ok],
        ..Default::default()
    }
}

/// A compound file as a VBA host saves it: compiled code (here a
/// stand-in) before each module's compressed source, a project name and
/// references in the dir stream, the PROJECT stream with the protection
/// values, and the form's designer storage.
fn host_project(cp: u16, password: bool) -> Vec<u8> {
    let pcode = vec![0xAB; 300];
    let mut w = dir::DirWriter::new("GridTools", cp);
    w.reference(
        "stdole",
        "*\\G{00020430-0000-0000-C000-000000000046}#2.0#0#stdole2.tlb#OLE Automation",
    );
    w.reference(
        "Office",
        "*\\G{2DF8D04C-5BFA-101B-BDE5-00AA0044DE52}#2.8#0#MSO.DLL#Office",
    );
    w.modules(4);
    w.module("Module1", "Module1", pcode.len() as u32, true);
    w.module("frmGrid", "frmGrid", pcode.len() as u32, false);
    w.module(
        "ThisMacroStorage",
        "ThisMacroStorage",
        pcode.len() as u32,
        false,
    );
    // Compiled only: the source offset is the stream's end.
    w.module("Compiled", "Compiled", pcode.len() as u32, true);
    let mut streams: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    streams.insert("VBA/dir".into(), ovba::compress(&w.finish()));
    let with_pcode = |source: &str| {
        let mut b = pcode.clone();
        b.extend(ovba::compress(&codepage::encode(source, cp)));
        b
    };
    streams.insert("VBA/Module1".into(), with_pcode(MODULE1));
    streams.insert("VBA/frmGrid".into(), with_pcode(FORM_CODE));
    streams.insert(
        "VBA/ThisMacroStorage".into(),
        with_pcode("Attribute VB_Name = \"ThisMacroStorage\"\r\nPrivate Sub GlobalMacroStorage_Start()\r\nEnd Sub\r\n"),
    );
    streams.insert("VBA/Compiled".into(), pcode.clone());
    streams.insert(
        "VBA/_VBA_PROJECT".into(),
        vec![0xCC, 0x61, 0xB5, 0x00, 0x00, 0x01, 0x00],
    );
    let info = project_stream::ProjectInfo {
        id: "{11111111-2222-3333-4444-555555555555}".into(),
        name: "GridTools".into(),
        standard: vec!["Module1".into(), "Compiled".into()],
        forms: vec!["frmGrid".into()],
        documents: vec!["ThisMacroStorage".into()],
        protection: Protection {
            password,
            locked_for_viewing: password,
            user_protected: false,
        },
        ..Default::default()
    };
    streams.insert(
        "PROJECT".into(),
        codepage::encode(&project_stream::write(&info), cp),
    );
    for (p, b) in forms::write_form(&grid_form()) {
        streams.insert(format!("frmGrid/{p}"), b);
    }
    compound(&streams).unwrap()
}

#[test]
fn a_host_project_gives_its_modules_forms_and_references() {
    let p = read_gms(&host_project(1252, false)).unwrap();
    assert_eq!(p.name, "GridTools");
    assert_eq!(p.references, ["stdole", "Office"]);
    assert!(!p.protected);
    assert_eq!(p.modules.len(), 4);
    let m = p.module("module1").unwrap();
    assert_eq!(m.kind, ModuleKind::Standard);
    assert_eq!(m.source, MODULE1);
    assert_eq!(m.macros(), ["MakeGrid", "ShowForm"]);
    assert!(!m.code().contains("Attribute"));
    assert_eq!(p.module("frmGrid").unwrap().kind, ModuleKind::Form);
    assert_eq!(
        p.module("ThisMacroStorage").unwrap().kind,
        ModuleKind::Document
    );
    let compiled = p.module("Compiled").unwrap();
    assert!(!compiled.has_source);
    assert_eq!(p.modules_without_source(), ["Compiled"]);
    assert!(
        p.warnings.iter().any(|w| w.contains("compiled code only")),
        "{:?}",
        p.warnings
    );
    assert_eq!(
        p.macros(),
        [
            ("Module1".to_string(), "MakeGrid".to_string()),
            ("Module1".to_string(), "ShowForm".to_string())
        ]
    );
    let form = p.form("frmGrid").unwrap();
    assert_eq!(form.caption, "Grid");
    assert_eq!(form.controls.len(), 3);
    assert_eq!(form.control("cmdOK").unwrap().caption, "OK");
    assert_eq!(form.control("txtCount").unwrap().value, "5");
}

#[test]
fn protection_and_code_pages() {
    let p = read_gms(&host_project(1252, true)).unwrap();
    assert!(p.protected);
    assert!(p.protection.password && p.protection.locked_for_viewing);
    // The code is still there.
    assert_eq!(p.module("Module1").unwrap().source, MODULE1);
    // A Japanese project: names and code in Shift-JIS.
    let mut w = dir::DirWriter::new("図形", 932);
    w.modules(1);
    w.module("標準", "標準", 0, true);
    let mut streams: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    streams.insert("VBA/dir".into(), ovba::compress(&w.finish()));
    let src =
        "Attribute VB_Name = \"標準\"\r\nSub 描く()\r\n    MsgBox \"こんにちは\"\r\nEnd Sub\r\n";
    streams.insert(
        "VBA/標準".into(),
        ovba::compress(&codepage::encode(src, 932)),
    );
    let p = read_gms(&compound(&streams).unwrap()).unwrap();
    assert_eq!(p.name, "図形");
    assert_eq!(p.code_page, 932);
    assert_eq!(p.modules[0].name, "標準");
    assert_eq!(p.modules[0].source, src);
    assert_eq!(p.modules[0].macros(), ["描く"]);
}

#[test]
fn files_that_are_not_projects_or_are_damaged() {
    assert!(matches!(
        read_gms(b"not a compound file"),
        Err(Error::NotAProject(_))
    ));
    let mut no_vba: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    no_vba.insert("Something".into(), vec![1, 2, 3]);
    assert!(matches!(
        read_gms(&compound(&no_vba).unwrap()),
        Err(Error::NotAProject(_))
    ));
    let mut bad_dir: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    bad_dir.insert("VBA/dir".into(), vec![0x01, 0x05, 0xB0, 0x00]);
    assert!(read_gms(&compound(&bad_dir).unwrap()).is_err());
    // Cut short anywhere: an error or a project, never a panic.
    let bytes = host_project(1252, false);
    for n in (0..bytes.len()).step_by(97) {
        let _ = read_gms(&bytes[..n]);
    }
    // Flipped bytes inside the streams.
    let mut b = bytes.clone();
    for i in (600..b.len()).step_by(53) {
        b[i] ^= 0x5A;
        let _ = read_gms(&b);
    }
}

#[test]
fn a_project_written_as_a_compound_file_reads_back() {
    let p = read_gms(&host_project(1251, true)).unwrap();
    let again = read_gms(&write_gms(&p).unwrap()).unwrap();
    assert_eq!(again.name, p.name);
    assert_eq!(again.protection, p.protection);
    assert_eq!(again.code_page, 1251);
    for m in &p.modules {
        let back = again.module(&m.name).unwrap();
        assert_eq!(back.kind, m.kind, "{}", m.name);
        if m.has_source {
            assert_eq!(back.source, m.source, "{}", m.name);
        }
    }
    let form = again.form("frmGrid").unwrap();
    assert_eq!(form.controls.len(), 3);
    assert_eq!(form.caption, "Grid");
}

#[test]
fn module_files_export_and_read_back() {
    let p = read_gms(&host_project(1252, false)).unwrap();
    let files = export_project(&p).unwrap();
    let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"Module1.bas"));
    assert!(names.contains(&"frmGrid.frm") && names.contains(&"frmGrid.frx"));
    assert!(names.contains(&"ThisMacroStorage.cls"));
    let frm = files.iter().find(|f| f.name == "frmGrid.frm").unwrap();
    let text = String::from_utf8_lossy(&frm.bytes);
    assert!(text.starts_with("VERSION 5.00\r\nBegin {C62A69F0"));
    assert!(text.contains("OleObjectBlob   =   \"frmGrid.frx\":0000"));
    let pairs: Vec<(String, Vec<u8>)> = files
        .iter()
        .map(|f| (f.name.clone(), f.bytes.clone()))
        .collect();
    let back = read_named("Exported", &pairs).unwrap();
    assert_eq!(back.module("Module1").unwrap().source, MODULE1);
    assert_eq!(
        back.module("ThisMacroStorage").unwrap().kind,
        ModuleKind::Class
    );
    let form = back.form("frmGrid").unwrap();
    assert_eq!(form.caption, "Grid");
    assert_eq!(form.controls.len(), 3, "{:?}", back.warnings);
    assert!(back.module("frmGrid").unwrap().source.contains("Unload Me"));
    // A form without its .frx keeps its header and warns.
    let lone = read_named("Lone", &[("frmGrid.frm".to_string(), frm.bytes.clone())]).unwrap();
    assert_eq!(lone.form("frmGrid").unwrap().caption, "Grid");
    assert!(lone.warnings.iter().any(|w| w.contains(".frx")));
    // A project file among the files: its modules are added.
    let mixed = read_named(
        "Mixed",
        &[
            (
                "Tools.bas".to_string(),
                b"Sub Hello()\r\nEnd Sub\r\n".to_vec(),
            ),
            ("GridTools.gms".to_string(), host_project(1252, false)),
        ],
    )
    .unwrap();
    assert!(mixed.module("Tools").is_some() && mixed.module("Module1").is_some());
    assert!(read_named("Empty", &[]).is_err());
}

#[test]
fn folders_of_module_files() {
    let dir = std::env::temp_dir().join(format!("tracedraw-vba-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let p = read_gms(&host_project(1252, false)).unwrap();
    for f in export_project(&p).unwrap() {
        std::fs::write(dir.join(&f.name), &f.bytes).unwrap();
    }
    std::fs::write(dir.join("notes.txt"), b"not a module").unwrap();
    let back = read_files(&[&dir]).unwrap();
    assert_eq!(back.modules.len(), 4);
    assert_eq!(back.form("frmGrid").unwrap().controls.len(), 3);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn exported_files_in_a_cyrillic_code_page_are_read_as_such() {
    let text = "Attribute VB_Name = \"m\"\r\n' Версия пресета\r\nPublic Sub Go()\r\nEnd Sub\r\n";
    let (bytes, _, _) = encoding_rs::WINDOWS_1251.encode(text);
    let p = crate::read_named("p", &[("m.bas".to_string(), bytes.into_owned())]).expect("project");
    assert!(
        p.modules[0].source.contains("Версия пресета"),
        "{}",
        p.modules[0].source
    );
}

#[test]
fn a_project_file_with_the_short_header_reads_like_the_compound_file() {
    let plain = host_project(1252, false);
    let mut wrapped = b"GMS\x01\x0a\x00\x00\x00\x02\x00\x00\x00\x00\x00\x01\x00\x00\x00".to_vec();
    wrapped.extend_from_slice(&plain);
    let a = crate::read_gms(&plain).expect("plain");
    let b = crate::read_gms(&wrapped).expect("with the header");
    assert_eq!(a.modules.len(), b.modules.len());
    assert_eq!(a.forms.len(), b.forms.len());
    assert!(crate::read_gms(b"GMS\x01 not a project").is_err());
}
