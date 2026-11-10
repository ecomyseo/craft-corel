//! Exported module files: `.bas` (a standard module), `.cls` (a class,
//! after a `VERSION 1.0 CLASS` header) and `.frm` (a form: a text header
//! of `Begin`/`End` blocks with the form's properties, and for classic
//! forms its controls, then the code). A form made with the forms
//! library keeps its controls in the binary `.frx` file next to it,
//! which is a compound file holding the designer streams.

use crate::forms::{Control, ControlKind, Font, Form, OleColor};
use std::collections::BTreeMap;

/// One `Begin ... End` block of a text form header.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Block {
    /// "VB.CommandButton", "{C62A69F0-...}" for a forms library form.
    pub type_name: String,
    pub name: String,
    /// Properties by lower-case name, strings without their quotes.
    pub props: BTreeMap<String, String>,
    /// Properties stored in the `.frx` file: (file, offset).
    pub frx: BTreeMap<String, (String, u32)>,
    pub font: Option<Font>,
    pub children: Vec<Block>,
}

/// The class id of a form made with the forms library.
pub const FORMS_FORM: &str = "{C62A69F0-16DC-11CE-9E98-00AA00574A4F}";

/// A property value: a quoted string (with `""` for a quote) and what
/// follows it, or the text before a `'` comment.
fn value(raw: &str) -> (String, Option<(String, u32)>) {
    let raw = raw.trim();
    if let Some(rest) = raw.strip_prefix('"') {
        let mut out = String::new();
        let mut chars = rest.char_indices().peekable();
        let mut end = rest.len();
        while let Some((i, c)) = chars.next() {
            if c == '"' {
                if chars.peek().map(|(_, c)| *c) == Some('"') {
                    out.push('"');
                    chars.next();
                    continue;
                }
                end = i + 1;
                break;
            }
            out.push(c);
        }
        let after = rest[end..].trim();
        let frx = after
            .strip_prefix(':')
            .and_then(|o| u32::from_str_radix(o.trim(), 16).ok())
            .map(|o| (out.clone(), o));
        return (out, frx);
    }
    let v = raw.split('\'').next().unwrap_or("").trim();
    (v.to_string(), None)
}

/// A number written as decimal, `&H...&` hex or `True`/`False`.
pub fn number(v: &str) -> Option<i64> {
    let v = v.trim().trim_end_matches('&');
    if let Some(h) = v.strip_prefix("&H").or_else(|| v.strip_prefix("&h")) {
        return i64::from_str_radix(h, 16).ok();
    }
    match v.to_ascii_lowercase().as_str() {
        "true" => Some(-1),
        "false" => Some(0),
        _ => v.parse::<f64>().ok().map(|f| f as i64),
    }
}

fn float(v: &str) -> Option<f32> {
    v.trim()
        .parse::<f32>()
        .ok()
        .or_else(|| number(v).map(|n| n as f32))
}

/// Parse the block starting at `lines[*i]` (a `Begin` line).
fn block(lines: &[&str], i: &mut usize, depth: usize) -> Option<Block> {
    let head = lines.get(*i)?.trim();
    let rest = head.strip_prefix("Begin ")?;
    let mut parts = rest.split_whitespace();
    let mut b = Block {
        type_name: parts.next().unwrap_or("").to_string(),
        name: parts.next().unwrap_or("").to_string(),
        ..Default::default()
    };
    *i += 1;
    while *i < lines.len() {
        let line = lines[*i].trim();
        if line == "End" {
            *i += 1;
            return Some(b);
        }
        if line.starts_with("Begin ") && depth < 16 {
            if let Some(child) = block(lines, i, depth + 1) {
                b.children.push(child);
            }
            continue;
        }
        if let Some(what) = line.strip_prefix("BeginProperty") {
            let what = what.trim().to_ascii_lowercase();
            let mut props = BTreeMap::new();
            let mut nested = 0;
            *i += 1;
            while *i < lines.len() {
                let l = lines[*i].trim();
                *i += 1;
                if l.starts_with("BeginProperty") {
                    nested += 1;
                } else if l.starts_with("EndProperty") {
                    if nested == 0 {
                        break;
                    }
                    nested -= 1;
                } else if let Some((k, v)) = l.split_once('=') {
                    props.insert(k.trim().to_ascii_lowercase(), value(v).0);
                }
            }
            if what.starts_with("font") {
                b.font = Some(font_of(&props));
            }
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim().to_ascii_lowercase();
            let (val, frx) = value(v);
            if let Some(f) = frx {
                b.frx.insert(key.clone(), f);
            }
            b.props.insert(key, val);
        }
        *i += 1;
    }
    Some(b)
}

fn font_of(p: &BTreeMap<String, String>) -> Font {
    let flag = |k: &str| p.get(k).and_then(|v| number(v)).unwrap_or(0) != 0;
    Font {
        name: p.get("name").cloned().unwrap_or_default(),
        size: p.get("size").and_then(|v| float(v)).unwrap_or(8.0),
        bold: p.get("weight").and_then(|v| number(v)).unwrap_or(400) >= 600,
        italic: flag("italic"),
        underline: flag("underline"),
        strikeout: flag("strikethrough"),
        charset: p.get("charset").and_then(|v| number(v)).unwrap_or(0) as u8,
    }
}

/// The first `Begin` block of a text form header (the form itself).
pub fn parse_header(text: &str) -> Option<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let mut i = lines
        .iter()
        .position(|l| l.trim_start().starts_with("Begin "))?;
    block(&lines, &mut i, 0)
}

/// What a module file holds.
#[derive(Debug, Clone, PartialEq)]
pub struct TextModule {
    pub name: String,
    pub kind: crate::ModuleKind,
    /// The code, with its `Attribute` lines.
    pub source: String,
    /// A form's header block.
    pub form: Option<Block>,
}

/// The value of `Attribute VB_Name = "..."` in `source`.
pub fn vb_name(source: &str) -> Option<String> {
    source.lines().find_map(|l| {
        let l = l.trim();
        let rest = l.strip_prefix("Attribute VB_Name")?;
        let v = rest.trim_start().strip_prefix('=')?;
        Some(value(v).0)
    })
}

/// Read a `.bas`, `.cls` or `.frm` file's text. `file_name` gives the
/// kind by its extension and the name when the text has none.
pub fn parse_module(file_name: &str, text: &str) -> TextModule {
    let ext = file_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let stem = file_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(file_name)
        .split('.')
        .next()
        .unwrap_or(file_name)
        .to_string();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    let mut form = None;
    while i < lines.len() {
        let l = lines[i].trim();
        if l.starts_with("VERSION ") || l.starts_with("Object = ") || l.is_empty() {
            i += 1;
        } else if l == "BEGIN" {
            // A class header: up to END.
            while i < lines.len() && lines[i].trim() != "END" {
                i += 1;
            }
            i += 1;
        } else if l.starts_with("Begin ") {
            form = block(&lines, &mut i, 0);
        } else {
            break;
        }
    }
    let source = lines[i.min(lines.len())..].join("\r\n");
    let source = if text.ends_with('\n') && !source.is_empty() {
        source + "\r\n"
    } else {
        source
    };
    let kind = match ext.as_str() {
        "cls" => crate::ModuleKind::Class,
        "frm" | "dsr" => crate::ModuleKind::Form,
        _ => crate::ModuleKind::Standard,
    };
    let name = vb_name(&source)
        .or_else(|| {
            form.as_ref()
                .map(|b: &Block| b.name.clone())
                .filter(|n| !n.is_empty())
        })
        .unwrap_or(stem);
    TextModule {
        name,
        kind,
        source,
        form,
    }
}

/// Points for a length in twips.
fn twips(v: Option<&String>) -> f32 {
    v.and_then(|v| float(v)).unwrap_or(0.0) / 20.0
}

fn color(v: Option<&String>) -> Option<OleColor> {
    v.and_then(|v| number(v))
        .map(|n| OleColor::from_u32(n as u32))
}

/// A control from a classic form's text block.
fn control(b: &Block) -> Control {
    let kind = ControlKind::from_type_name(&b.type_name).unwrap_or(ControlKind::Other(0));
    let mut c = Control::new(kind, &b.name);
    let p = &b.props;
    let flag = |k: &str, d: bool| {
        p.get(k)
            .and_then(|v| number(v))
            .map(|n| n != 0)
            .unwrap_or(d)
    };
    c.left = twips(p.get("left"));
    c.top = twips(p.get("top"));
    c.width = twips(p.get("width"));
    c.height = twips(p.get("height"));
    c.caption = p.get("caption").cloned().unwrap_or_default();
    c.value = p
        .get("text")
        .or_else(|| p.get("value"))
        .cloned()
        .unwrap_or_default();
    c.tag = p.get("tag").cloned().unwrap_or_default();
    c.tip = p
        .get("tooltiptext")
        .or_else(|| p.get("controltiptext"))
        .cloned()
        .unwrap_or_default();
    c.enabled = flag("enabled", true);
    c.visible = flag("visible", true);
    c.locked = flag("locked", false);
    c.tab_stop = flag("tabstop", true);
    c.multiline = flag("multiline", false);
    c.default = flag("default", false);
    c.cancel = flag("cancel", false);
    c.tab_index = p.get("tabindex").and_then(|v| number(v)).unwrap_or(-1) as i16;
    c.max_length = p.get("maxlength").and_then(|v| number(v)).unwrap_or(0) as u32;
    c.password_char = p.get("passwordchar").and_then(|v| v.chars().next());
    c.group_name = p.get("groupname").cloned().unwrap_or_default();
    c.fore_color = color(p.get("forecolor"));
    c.back_color = color(p.get("backcolor"));
    let int = |k: &str| p.get(k).and_then(|v| number(v)).map(|n| n as i32);
    c.min = int("min");
    c.max = int("max");
    c.small_change = int("smallchange");
    c.large_change = int("largechange");
    c.font = b.font.clone();
    if b.type_name.eq_ignore_ascii_case("VB.HScrollBar") {
        c.orientation = Some(1);
    } else if b.type_name.eq_ignore_ascii_case("VB.VScrollBar") {
        c.orientation = Some(0);
    }
    c.children = b.children.iter().map(control).collect();
    c
}

/// A form from a text header block: its caption, size and, for classic
/// forms, its controls (a forms library form has its controls in the
/// `.frx` file).
pub fn form_from_block(name: &str, b: &Block) -> Form {
    let p = &b.props;
    let (w, h) = if p.contains_key("clientwidth") {
        (twips(p.get("clientwidth")), twips(p.get("clientheight")))
    } else {
        (twips(p.get("width")), twips(p.get("height")))
    };
    Form {
        name: name.to_string(),
        caption: p
            .get("caption")
            .cloned()
            .unwrap_or_else(|| name.to_string()),
        width: w,
        height: h,
        back_color: color(p.get("backcolor")),
        fore_color: color(p.get("forecolor")),
        font: b.font.clone(),
        enabled: true,
        controls: b.children.iter().map(control).collect(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORMS_FRM: &str = "VERSION 5.00\r\n\
Begin {C62A69F0-16DC-11CE-9E98-00AA00574A4F} frmGrid \r\n\
   Caption         =   \"Grid \"\"helper\"\"\"\r\n\
   ClientHeight    =   3015\r\n\
   ClientLeft      =   45\r\n\
   ClientTop       =   390\r\n\
   ClientWidth     =   4560\r\n\
   OleObjectBlob   =   \"frmGrid.frx\":0000\r\n\
   StartUpPosition =   1  'CenterOwner\r\n\
End\r\n\
Attribute VB_Name = \"frmGrid\"\r\n\
Attribute VB_GlobalNameSpace = False\r\n\
Private Sub cmdOK_Click()\r\n\
    Unload Me\r\n\
End Sub\r\n";

    const CLASSIC_FRM: &str = "VERSION 5.00\r\n\
Begin VB.Form Form1 \r\n\
   Caption         =   \"Classic\"\r\n\
   ClientHeight    =   2400\r\n\
   ClientWidth     =   3600\r\n\
   Begin VB.Frame Frame1 \r\n\
      Caption         =   \"Box\"\r\n\
      Height          =   1200\r\n\
      Left            =   120\r\n\
      Top             =   120\r\n\
      Width           =   2000\r\n\
      Begin VB.CheckBox Check1 \r\n\
         Caption         =   \"On\"\r\n\
         Value           =   1  'Checked\r\n\
         Height          =   300\r\n\
         Left            =   100\r\n\
         Top             =   200\r\n\
         Width           =   900\r\n\
      End\r\n\
   End\r\n\
   Begin VB.TextBox Text1 \r\n\
      BeginProperty Font \r\n\
         Name            =   \"Arial\"\r\n\
         Size            =   9.75\r\n\
         Weight          =   700\r\n\
         Italic          =   -1  'True\r\n\
      EndProperty\r\n\
      ForeColor       =   &H000000FF&\r\n\
      Enabled         =   0   'False\r\n\
      Height          =   285\r\n\
      Left            =   120\r\n\
      Text            =   \"hello\"\r\n\
      Top             =   1500\r\n\
      Width           =   1500\r\n\
   End\r\n\
End\r\n\
Attribute VB_Name = \"Form1\"\r\n\
Sub X()\r\nEnd Sub\r\n";

    #[test]
    fn a_forms_library_form_points_to_its_frx() {
        let m = parse_module("C:\\macros\\frmGrid.frm", FORMS_FRM);
        assert_eq!(m.name, "frmGrid");
        assert_eq!(m.kind, crate::ModuleKind::Form);
        assert!(m.source.starts_with("Attribute VB_Name = \"frmGrid\""));
        assert!(m.source.contains("Unload Me"));
        let b = m.form.unwrap();
        assert_eq!(b.type_name, FORMS_FORM);
        assert_eq!(b.props["caption"], "Grid \"helper\"");
        assert_eq!(b.props["startupposition"], "1");
        assert_eq!(b.frx["oleobjectblob"], ("frmGrid.frx".to_string(), 0));
        let f = form_from_block("frmGrid", &b);
        assert_eq!(f.width, 228.0);
        assert_eq!(f.height, 150.75);
    }

    #[test]
    fn a_classic_form_has_its_controls_in_the_text() {
        let m = parse_module("Form1.frm", CLASSIC_FRM);
        let f = form_from_block(&m.name, m.form.as_ref().unwrap());
        assert_eq!(f.caption, "Classic");
        assert_eq!(f.controls.len(), 2);
        let frame = &f.controls[0];
        assert_eq!(frame.kind, ControlKind::Frame);
        assert_eq!((frame.left, frame.width), (6.0, 100.0));
        assert_eq!(frame.children[0].kind, ControlKind::CheckBox);
        assert_eq!(frame.children[0].value, "1");
        let text = &f.controls[1];
        assert_eq!(text.kind, ControlKind::TextBox);
        assert_eq!(text.value, "hello");
        assert!(!text.enabled);
        assert_eq!(text.fore_color, Some(OleColor::Rgb(255, 0, 0)));
        let font = text.font.as_ref().unwrap();
        assert_eq!(font.name, "Arial");
        assert!(font.bold && font.italic && font.size == 9.75);
        assert!(m.source.contains("Sub X()"));
    }

    #[test]
    fn classes_and_standard_modules() {
        let cls = "VERSION 1.0 CLASS\r\nBEGIN\r\n  MultiUse = -1  'True\r\nEND\r\nAttribute VB_Name = \"clsShape\"\r\nPublic W As Double\r\n";
        let m = parse_module("clsShape.cls", cls);
        assert_eq!(
            (m.name.as_str(), m.kind),
            ("clsShape", crate::ModuleKind::Class)
        );
        assert!(m.source.starts_with("Attribute VB_Name"));
        let bas = parse_module("Tools.bas", "Sub Go()\nEnd Sub\n");
        assert_eq!(
            (bas.name.as_str(), bas.kind),
            ("Tools", crate::ModuleKind::Standard)
        );
        assert_eq!(bas.source, "Sub Go()\r\nEnd Sub\r\n");
        // Unclosed blocks and odd lines do not panic.
        let _ = parse_module(
            "x.frm",
            "Begin VB.Form F\r\n Begin VB.Label L\r\n Caption = \"unterminated\r\n",
        );
        assert_eq!(number("&H8000000F&"), Some(0x8000000F));
        assert_eq!(number("True"), Some(-1));
    }
}
