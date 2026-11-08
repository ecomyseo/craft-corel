//! Forms (UserForms) of a VBA project: the designer storage each form has
//! next to the `VBA` storage, laid out as in the public MS-OFORMS
//! specification. The `f` stream holds the form's own properties and the
//! list of its controls ("sites"), the `o` stream the properties of each
//! control one after the other, and a frame or a multi-page keeps its
//! controls in a storage of its own (`i<ID>`). Every read is bounds
//! checked; a part that cannot be read leaves a warning and the rest of
//! the form.

use crate::codepage::utf16;
use crate::Error;
use std::collections::BTreeMap;

/// A colour as forms store it (OLE_COLOR).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OleColor {
    Rgb(u8, u8, u8),
    /// A system colour by index (button face, window text...).
    System(u16),
    /// A palette entry.
    Palette(u16),
}

impl OleColor {
    pub fn from_u32(v: u32) -> OleColor {
        match v >> 24 {
            0x80 => OleColor::System((v & 0xFFFF) as u16),
            0x01 => OleColor::Palette((v & 0xFFFF) as u16),
            _ => OleColor::Rgb(v as u8, (v >> 8) as u8, (v >> 16) as u8),
        }
    }

    /// The colour as an RGB triple, system colours as the classic
    /// Windows defaults.
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            OleColor::Rgb(r, g, b) => (r, g, b),
            OleColor::Palette(_) => (0, 0, 0),
            OleColor::System(i) => match i {
                // Window text, button text, menu text, caption text.
                0x08 | 0x12 | 0x07 | 0x09 => (0, 0, 0),
                // Window background.
                0x05 => (255, 255, 255),
                // Highlight, highlight text.
                0x0D => (0, 120, 215),
                0x0E => (255, 255, 255),
                // Gray text, button shadow.
                0x11 | 0x10 => (109, 109, 109),
                // Button face and the rest.
                _ => (240, 240, 240),
            },
        }
    }
}

/// A font as forms store it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Font {
    pub name: String,
    /// Size in points.
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikeout: bool,
    pub charset: u8,
}

/// The kinds of controls a form holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlKind {
    Label,
    TextBox,
    CommandButton,
    CheckBox,
    OptionButton,
    ToggleButton,
    ComboBox,
    ListBox,
    Frame,
    MultiPage,
    Page,
    Image,
    SpinButton,
    ScrollBar,
    TabStrip,
    /// A control this reader does not know, by its class index.
    Other(u16),
}

impl ControlKind {
    /// The kind for a class cache index (MS-OFORMS 2.4.1).
    pub fn from_cache_index(i: u16) -> ControlKind {
        match i {
            7 => ControlKind::Page,
            12 => ControlKind::Image,
            14 => ControlKind::Frame,
            16 => ControlKind::SpinButton,
            17 => ControlKind::CommandButton,
            18 => ControlKind::TabStrip,
            21 => ControlKind::Label,
            23 => ControlKind::TextBox,
            24 => ControlKind::ListBox,
            25 => ControlKind::ComboBox,
            26 => ControlKind::CheckBox,
            27 => ControlKind::OptionButton,
            28 => ControlKind::ToggleButton,
            47 => ControlKind::ScrollBar,
            57 => ControlKind::MultiPage,
            other => ControlKind::Other(other),
        }
    }

    /// The kind for a control class id (text, braces optional).
    pub fn from_clsid(clsid: &str) -> Option<ControlKind> {
        let c = clsid
            .trim_matches(|c| c == '{' || c == '}')
            .to_ascii_uppercase();
        Some(match c.as_str() {
            "978C9E23-D4B0-11CE-BF2D-00AA003F40D0" => ControlKind::Label,
            "8BD21D10-EC42-11CE-9E0D-00AA006002F3" => ControlKind::TextBox,
            "D7053240-CE69-11CD-A777-00DD01143C57" => ControlKind::CommandButton,
            "8BD21D40-EC42-11CE-9E0D-00AA006002F3" => ControlKind::CheckBox,
            "8BD21D50-EC42-11CE-9E0D-00AA006002F3" => ControlKind::OptionButton,
            "8BD21D60-EC42-11CE-9E0D-00AA006002F3" => ControlKind::ToggleButton,
            "8BD21D30-EC42-11CE-9E0D-00AA006002F3" => ControlKind::ComboBox,
            "8BD21D20-EC42-11CE-9E0D-00AA006002F3" => ControlKind::ListBox,
            "6E182020-F460-11CE-9BCD-00AA00608E01" => ControlKind::Frame,
            "46E31370-3F7A-11CE-BED6-00AA00611080" => ControlKind::MultiPage,
            "5CEF5610-713D-11CE-80C9-00AA00611080" => ControlKind::Page,
            "4C599241-6926-101B-9992-00000B65C6F9" => ControlKind::Image,
            "79176FB0-B7F2-11CE-97EF-00AA006D2776" => ControlKind::SpinButton,
            "DFD181E0-5E2F-11CE-A449-00AA004A803D" => ControlKind::ScrollBar,
            "EAE50EB0-4A62-11CE-BED6-00AA00611080" => ControlKind::TabStrip,
            _ => return None,
        })
    }

    /// The kind for a type name in a text form ("VB.CommandButton",
    /// "MSForms.TextBox"...).
    pub fn from_type_name(name: &str) -> Option<ControlKind> {
        let n = name.rsplit('.').next().unwrap_or(name).to_ascii_lowercase();
        Some(match n.as_str() {
            "label" => ControlKind::Label,
            "textbox" => ControlKind::TextBox,
            "commandbutton" => ControlKind::CommandButton,
            "checkbox" => ControlKind::CheckBox,
            "optionbutton" => ControlKind::OptionButton,
            "togglebutton" => ControlKind::ToggleButton,
            "combobox" => ControlKind::ComboBox,
            "listbox" => ControlKind::ListBox,
            "frame" => ControlKind::Frame,
            "multipage" => ControlKind::MultiPage,
            "page" => ControlKind::Page,
            "image" | "picturebox" => ControlKind::Image,
            "spinbutton" => ControlKind::SpinButton,
            "scrollbar" | "hscrollbar" | "vscrollbar" => ControlKind::ScrollBar,
            "tabstrip" => ControlKind::TabStrip,
            _ => return None,
        })
    }

    /// A container whose controls are its children.
    pub fn is_container(self) -> bool {
        matches!(
            self,
            ControlKind::Frame | ControlKind::MultiPage | ControlKind::Page
        )
    }
}

/// One control on a form.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    pub kind: ControlKind,
    pub name: String,
    pub id: i32,
    /// Position and size in points, from the container's top left.
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub caption: String,
    /// The text or value (a text box's text, "1" for a ticked box...).
    pub value: String,
    pub tag: String,
    pub tip: String,
    pub control_source: String,
    pub row_source: String,
    pub group_name: String,
    pub font: Option<Font>,
    pub fore_color: Option<OleColor>,
    pub back_color: Option<OleColor>,
    pub border_color: Option<OleColor>,
    pub enabled: bool,
    pub visible: bool,
    pub locked: bool,
    pub tab_stop: bool,
    pub tab_index: i16,
    pub default: bool,
    pub cancel: bool,
    /// List and combo boxes: 0 plain, 1 with option marks.
    pub list_style: u8,
    pub max_length: u32,
    pub multiline: bool,
    pub word_wrap: bool,
    pub password_char: Option<char>,
    pub accelerator: Option<char>,
    pub column_count: i16,
    /// Spin buttons and scroll bars.
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub position: Option<i32>,
    pub small_change: Option<i32>,
    pub large_change: Option<i32>,
    /// -1 automatic, 0 vertical, 1 horizontal.
    pub orientation: Option<i32>,
    /// The picture's bytes as stored (usually a BMP, GIF, JPEG or icon).
    pub picture: Option<Vec<u8>>,
    /// Tab captions (tab strips).
    pub items: Vec<String>,
    /// Frames, multi-pages and pages: the controls inside.
    pub children: Vec<Control>,
}

impl Control {
    pub fn new(kind: ControlKind, name: &str) -> Control {
        Control {
            kind,
            name: name.to_string(),
            id: 0,
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
            caption: String::new(),
            value: String::new(),
            tag: String::new(),
            tip: String::new(),
            control_source: String::new(),
            row_source: String::new(),
            group_name: String::new(),
            font: None,
            fore_color: None,
            back_color: None,
            border_color: None,
            enabled: true,
            visible: true,
            locked: false,
            tab_stop: true,
            tab_index: -1,
            default: false,
            cancel: false,
            list_style: 0,
            max_length: 0,
            multiline: false,
            word_wrap: false,
            password_char: None,
            accelerator: None,
            column_count: 1,
            min: None,
            max: None,
            position: None,
            small_change: None,
            large_change: None,
            orientation: None,
            picture: None,
            items: Vec::new(),
            children: Vec::new(),
        }
    }

    /// This control and every control inside it, depth first.
    pub fn walk(&self) -> Vec<&Control> {
        let mut out = vec![self];
        for c in &self.children {
            out.extend(c.walk());
        }
        out
    }
}

/// A form: its own properties and its controls.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Form {
    /// The form module's name.
    pub name: String,
    pub caption: String,
    /// Inside size in points.
    pub width: f32,
    pub height: f32,
    pub back_color: Option<OleColor>,
    pub fore_color: Option<OleColor>,
    pub font: Option<Font>,
    pub picture: Option<Vec<u8>>,
    pub enabled: bool,
    pub controls: Vec<Control>,
    /// The designer storage's streams as stored (paths relative to it,
    /// "/" between parts), so the form can be written back unchanged.
    pub raw: Vec<(String, Vec<u8>)>,
    /// What could not be read.
    pub warnings: Vec<String>,
}

impl Form {
    /// Every control on the form, inside containers too.
    pub fn all_controls(&self) -> Vec<&Control> {
        self.controls.iter().flat_map(|c| c.walk()).collect()
    }

    /// The control named `name` (any case), wherever it is.
    pub fn control(&self, name: &str) -> Option<&Control> {
        self.all_controls()
            .into_iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }
}

/// Points for a length in HIMETRIC (hundredths of a millimetre).
pub fn himetric_to_points(v: i32) -> f32 {
    v as f32 * 72.0 / 2540.0
}

/// HIMETRIC for a length in points.
pub fn points_to_himetric(v: f32) -> i32 {
    (v * 2540.0 / 72.0).round() as i32
}

// ----- Reading ------------------------------------------------------------

fn corrupt(what: impl Into<String>) -> Error {
    Error::Corrupt(format!("form: {}", what.into()))
}

/// A cursor over one structure, with the alignment rules of MS-OFORMS:
/// a field starts on a multiple of its own size, counted from `base`.
struct R<'a> {
    d: &'a [u8],
    pos: usize,
    base: usize,
}

impl<'a> R<'a> {
    fn new(d: &'a [u8], pos: usize) -> R<'a> {
        R { d, pos, base: pos }
    }

    fn align(&mut self, n: usize) {
        let off = self.pos - self.base;
        if !off.is_multiple_of(n) {
            self.pos += n - off % n;
        }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let s = self
            .d
            .get(self.pos..self.pos + n)
            .ok_or_else(|| corrupt(format!("{n} bytes wanted at {}", self.pos)))?;
        self.pos += n;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, Error> {
        self.align(2);
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, Error> {
        self.align(4);
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn i32(&mut self) -> Result<i32, Error> {
        Ok(self.u32()? as i32)
    }

    /// Raw little-endian fields without alignment (headers).
    fn raw_u16(&mut self) -> Result<u16, Error> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn raw_u32(&mut self) -> Result<u32, Error> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// A string whose length and compression flag were given in the
    /// data block, padded to 4 bytes.
    fn string(&mut self, count_and_flag: u32) -> Result<String, Error> {
        let n = (count_and_flag & 0x7FFF_FFFF) as usize;
        if n == 0 {
            return Ok(String::new());
        }
        self.align(4);
        let b = self.take(n)?;
        let s = if count_and_flag & 0x8000_0000 != 0 {
            // One byte per character.
            b.iter().map(|&c| c as char).collect()
        } else {
            utf16(b)
        };
        self.align(4);
        Ok(s)
    }

    /// An fmSize or fmPosition: two 4-byte values.
    fn pair(&mut self) -> Result<(i32, i32), Error> {
        Ok((self.i32()?, self.i32()?))
    }

    fn guid(&mut self) -> Result<String, Error> {
        let b = self.take(16)?;
        Ok(format!(
            "{{{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}}}",
            u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            u16::from_le_bytes([b[4], b[5]]),
            u16::from_le_bytes([b[6], b[7]]),
            b[8],
            b[9],
            b[10],
            b[11],
            b[12],
            b[13],
            b[14],
            b[15]
        ))
    }
}

const STD_FONT: &str = "{0BE35203-8F91-11CE-9DE3-00AA004BB851}";
const TEXT_PROPS: &str = "{AFC20920-DA4E-11CE-B943-00AA006887B4}";
const STD_PICTURE: &str = "{0BE35204-8F91-11CE-9DE3-00AA004BB851}";

/// A GuidAndPicture: the picture's bytes.
fn picture(r: &mut R) -> Result<Vec<u8>, Error> {
    let guid = r.guid()?;
    if guid != STD_PICTURE {
        return Err(corrupt(format!("a picture of unknown kind {guid}")));
    }
    let preamble = r.raw_u32()?;
    if preamble != 0x0000_746C {
        return Err(corrupt("a picture without its preamble"));
    }
    let size = r.raw_u32()? as usize;
    Ok(r.take(size)?.to_vec())
}

/// A TextProps structure (MS-OFORMS 2.3.1).
fn text_props(r: &mut R) -> Result<Font, Error> {
    let _minor = r.u8()?;
    let _major = r.u8()?;
    let cb = r.raw_u16()? as usize;
    let start = r.pos;
    let mut b = R::new(r.d, r.pos);
    let mask = b.raw_u32()?;
    b.base = b.pos;
    let mut font = Font {
        size: 8.0,
        ..Default::default()
    };
    let name_len = if mask & 1 != 0 { b.u32()? } else { 0 };
    if mask & 2 != 0 {
        let e = b.u32()?;
        font.bold = e & 1 != 0;
        font.italic = e & 2 != 0;
        font.underline = e & 4 != 0;
        font.strikeout = e & 8 != 0;
    }
    if mask & 4 != 0 {
        font.size = b.u32()? as f32 / 20.0;
    }
    if mask & 0x10 != 0 {
        font.charset = b.u8()?;
    }
    if mask & 0x20 != 0 {
        b.u8()?;
    }
    if mask & 0x40 != 0 {
        b.u8()?;
    }
    if mask & 0x80 != 0 {
        let w = b.u16()?;
        font.bold |= w >= 600;
    }
    b.align(4);
    b.base = b.pos;
    font.name = b.string(name_len)?;
    r.pos = start + cb;
    Ok(font)
}

/// A GuidAndFont: a standard font or text properties.
fn guid_and_font(r: &mut R) -> Result<Font, Error> {
    let guid = r.guid()?;
    match guid.as_str() {
        STD_FONT => {
            let _version = r.u8()?;
            let charset = r.raw_u16()?;
            let flags = r.u8()?;
            let weight = r.raw_u16()?;
            let height = r.raw_u32()?;
            let len = r.u8()? as usize;
            let face = r.take(len)?;
            Ok(Font {
                name: face.iter().map(|&c| c as char).collect(),
                size: height as f32 / 10_000.0,
                bold: weight >= 600,
                italic: flags & 2 != 0,
                underline: flags & 4 != 0,
                strikeout: flags & 8 != 0,
                charset: charset as u8,
            })
        }
        TEXT_PROPS => text_props(r),
        other => Err(corrupt(format!("a font of unknown kind {other}"))),
    }
}

/// The form's own properties from its FormControl (MS-OFORMS 2.2.10).
#[derive(Debug, Default)]
struct FormControl {
    caption: String,
    width: f32,
    height: f32,
    back_color: Option<OleColor>,
    fore_color: Option<OleColor>,
    font: Option<Font>,
    picture: Option<Vec<u8>>,
    enabled: bool,
    class_table: Vec<Option<ControlKind>>,
    sites: Vec<Site>,
}

/// A control's place on its form (an OleSiteConcreteControl).
#[derive(Debug, Clone)]
struct Site {
    name: String,
    tag: String,
    id: i32,
    flags: u32,
    stream_size: u32,
    tab_index: i16,
    class: u16,
    group_id: u16,
    left: i32,
    top: i32,
    tip: String,
    control_source: String,
    row_source: String,
}

fn form_control(f: &[u8]) -> Result<FormControl, Error> {
    let mut r = R::new(f, 0);
    let _minor = r.u8()?;
    let _major = r.u8()?;
    let cb = r.raw_u16()? as usize;
    let block_start = r.pos;
    let mask = r.raw_u32()?;
    r.base = r.pos;
    let mut fc = FormControl {
        enabled: true,
        ..Default::default()
    };
    if mask & (1 << 1) != 0 {
        fc.back_color = Some(OleColor::from_u32(r.u32()?));
    }
    if mask & (1 << 2) != 0 {
        fc.fore_color = Some(OleColor::from_u32(r.u32()?));
    }
    if mask & (1 << 3) != 0 {
        r.u32()?;
    }
    let mut boolean = 0x0000_0004u32;
    if mask & (1 << 6) != 0 {
        boolean = r.u32()?;
    }
    fc.enabled = boolean & 4 != 0;
    if mask & (1 << 7) != 0 {
        r.u8()?;
    }
    if mask & (1 << 8) != 0 {
        r.u8()?;
    }
    if mask & (1 << 9) != 0 {
        r.u8()?;
    }
    if mask & (1 << 13) != 0 {
        r.u32()?;
    }
    let mut mouse_icon = 0u16;
    if mask & (1 << 15) != 0 {
        mouse_icon = r.u16()?;
    }
    if mask & (1 << 16) != 0 {
        r.u8()?;
    }
    if mask & (1 << 17) != 0 {
        r.u8()?;
    }
    if mask & (1 << 18) != 0 {
        r.u32()?;
    }
    let caption_len = if mask & (1 << 19) != 0 { r.u32()? } else { 0 };
    if mask & (1 << 20) != 0 {
        r.u16()?;
    }
    let mut picture_marker = 0u16;
    if mask & (1 << 21) != 0 {
        picture_marker = r.u16()?;
    }
    if mask & (1 << 22) != 0 {
        r.u32()?;
    }
    if mask & (1 << 23) != 0 {
        r.u8()?;
    }
    if mask & (1 << 25) != 0 {
        r.u8()?;
    }
    if mask & (1 << 26) != 0 {
        r.u32()?;
    }
    if mask & (1 << 27) != 0 {
        r.u32()?;
    }
    r.align(4);
    r.base = r.pos;
    if mask & (1 << 10) != 0 {
        let (w, h) = r.pair()?;
        fc.width = himetric_to_points(w);
        fc.height = himetric_to_points(h);
    }
    if mask & (1 << 11) != 0 {
        r.pair()?;
    }
    if mask & (1 << 12) != 0 {
        r.pair()?;
    }
    fc.caption = r.string(caption_len)?;
    // The stream data starts where the size says the blocks end.
    if block_start + cb <= f.len() {
        r.pos = block_start + cb;
    }
    if mouse_icon == 0xFFFF {
        picture(&mut r)?;
    }
    if mask & (1 << 20) != 0 {
        fc.font = Some(guid_and_font(&mut r)?);
    }
    if picture_marker == 0xFFFF {
        fc.picture = Some(picture(&mut r)?);
    }
    // FormSiteData: the class table, unless the form says it is left out.
    r.base = r.pos;
    if boolean & 0x8000 == 0 {
        let count = r.raw_u16()?;
        for _ in 0..count {
            fc.class_table.push(class_info(&mut r)?);
        }
    }
    let count_sites = r.raw_u32()? as usize;
    let count_bytes = r.raw_u32()? as usize;
    let sites_start = r.pos;
    let mut n = 0;
    while n < count_sites {
        let _depth = r.u8()?;
        let type_or_count = r.u8()?;
        if type_or_count & 0x80 != 0 {
            let _type = r.u8()?;
            n += (type_or_count & 0x7F) as usize;
        } else {
            n += 1;
        }
    }
    // The sites start on a 4-byte boundary after the depths and types.
    let used = r.pos - sites_start;
    r.pos = sites_start + used.div_ceil(4) * 4;
    for _ in 0..count_sites {
        fc.sites.push(site(&mut r)?);
    }
    if r.pos > sites_start + count_bytes + 4 {
        return Err(corrupt("the sites run past their size"));
    }
    Ok(fc)
}

/// One class table entry: the class id it names, as a control kind.
fn class_info(r: &mut R) -> Result<Option<ControlKind>, Error> {
    let _version = r.raw_u16()?;
    let cb = r.raw_u16()? as usize;
    let start = r.pos;
    let end = start + cb;
    let mut b = R::new(r.d, r.pos);
    let mask = b.raw_u32()?;
    b.base = b.pos;
    // Data block fields, in order, by mask bit and size.
    for (bit, size) in [
        (4, 4),  // ClassTableFlags and VarFlags
        (5, 4),  // CountOfMethods
        (6, 4),  // DispidBind
        (7, 2),  // GetBindIndex
        (8, 2),  // PutBindIndex
        (9, 2),  // BindType
        (10, 2), // GetValueIndex
        (11, 2), // PutValueIndex
        (12, 2), // ValueType
        (13, 4), // DispidRowset
        (14, 2), // SetRowset
    ] {
        if mask & (1 << bit) != 0 {
            if size == 4 {
                b.u32()?;
            } else {
                b.u16()?;
            }
        }
    }
    b.align(4);
    let kind = if mask & 1 != 0 {
        ControlKind::from_clsid(&b.guid()?)
    } else {
        None
    };
    r.pos = end.min(r.d.len());
    Ok(kind)
}

fn site(r: &mut R) -> Result<Site, Error> {
    let _version = r.raw_u16()?;
    let cb = r.raw_u16()? as usize;
    let start = r.pos;
    let mut b = R::new(r.d, r.pos);
    let mask = b.raw_u32()?;
    b.base = b.pos;
    let mut s = Site {
        name: String::new(),
        tag: String::new(),
        id: 0,
        flags: 0x0000_0033,
        stream_size: 0,
        tab_index: -1,
        class: 0x7FFF,
        group_id: 0,
        left: 0,
        top: 0,
        tip: String::new(),
        control_source: String::new(),
        row_source: String::new(),
    };
    let name_len = if mask & 1 != 0 { b.u32()? } else { 0 };
    let tag_len = if mask & 2 != 0 { b.u32()? } else { 0 };
    if mask & 4 != 0 {
        s.id = b.i32()?;
    }
    if mask & 8 != 0 {
        b.u32()?;
    }
    if mask & 0x10 != 0 {
        s.flags = b.u32()?;
    }
    if mask & 0x20 != 0 {
        s.stream_size = b.u32()?;
    }
    if mask & 0x40 != 0 {
        s.tab_index = b.u16()? as i16;
    }
    if mask & 0x80 != 0 {
        s.class = b.u16()?;
    }
    if mask & 0x200 != 0 {
        s.group_id = b.u16()?;
    }
    let tip_len = if mask & 0x800 != 0 { b.u32()? } else { 0 };
    let lic_len = if mask & 0x1000 != 0 { b.u32()? } else { 0 };
    let source_len = if mask & 0x2000 != 0 { b.u32()? } else { 0 };
    let row_len = if mask & 0x4000 != 0 { b.u32()? } else { 0 };
    b.align(4);
    b.base = b.pos;
    s.name = b.string(name_len)?;
    s.tag = b.string(tag_len)?;
    if mask & 0x100 != 0 {
        // SitePosition: left, then top (checked against saved projects).
        let (left, top) = b.pair()?;
        s.left = left;
        s.top = top;
    }
    s.tip = b.string(tip_len)?;
    b.string(lic_len)?;
    s.control_source = b.string(source_len)?;
    s.row_source = b.string(row_len)?;
    r.pos = start + cb;
    if r.pos > r.d.len() {
        return Err(corrupt("a site runs past the stream"));
    }
    Ok(s)
}

/// The fields a control's data block holds: (mask bit, size in bytes,
/// what it is). Strings give their length here and their text in the
/// extra data block.
#[derive(Clone, Copy, PartialEq)]
enum F {
    Skip,
    ForeColor,
    BackColor,
    BorderColor,
    Various,
    Caption,
    Value,
    GroupName,
    MaxLength,
    PasswordChar,
    ColumnCount,
    ListStyle,
    Accelerator,
    MouseIcon,
    Picture,
    Min,
    Max,
    Position,
    SmallChange,
    LargeChange,
    Orientation,
    DisplayStyle,
}

/// How a control's structure is laid out: the mask's size, the data
/// block's fields as (mask bit, size in bytes, field), the mask bit of
/// its size, whether the caption comes before the size in the extra data
/// block (otherwise value, caption and group name come after it), the
/// pictures in its stream data, and whether TextProps follow.
struct Layout<'a> {
    mask_bytes: usize,
    fields: &'a [(u32, usize, F)],
    size_bit: u32,
    caption_first: bool,
    streams: &'a [F],
    text_props: bool,
}

/// Read a control's structure laid out as `layout`.
fn control_data(data: &[u8], c: &mut Control, layout: Layout) -> Result<(), Error> {
    let Layout {
        mask_bytes,
        fields,
        size_bit,
        caption_first,
        streams,
        text_props: has_text_props,
    } = layout;
    let mut r = R::new(data, 0);
    let _minor = r.u8()?;
    let _major = r.u8()?;
    let cb = r.raw_u16()? as usize;
    let start = r.pos;
    let mask: u64 = if mask_bytes == 8 {
        r.raw_u32()? as u64 | (r.raw_u32()? as u64) << 32
    } else {
        r.raw_u32()? as u64
    };
    r.base = r.pos;
    let mut caption_len = 0;
    let mut value_len = 0;
    let mut group_len = 0;
    let mut mouse_icon = 0u16;
    let mut picture_marker = 0u16;
    let mut various: Option<u32> = None;
    for &(bit, size, field) in fields {
        if mask & (1u64 << bit) == 0 {
            continue;
        }
        let v: u32 = match size {
            1 => r.u8()? as u32,
            2 => r.u16()? as u32,
            _ => r.u32()?,
        };
        match field {
            F::ForeColor => c.fore_color = Some(OleColor::from_u32(v)),
            F::BackColor => c.back_color = Some(OleColor::from_u32(v)),
            F::BorderColor => c.border_color = Some(OleColor::from_u32(v)),
            F::Various => various = Some(v),
            F::Caption => caption_len = v,
            F::Value => value_len = v,
            F::GroupName => group_len = v,
            F::MaxLength => c.max_length = v,
            F::PasswordChar => c.password_char = char::from_u32(v).filter(|&ch| ch != '\0'),
            F::ColumnCount => c.column_count = v as i16,
            F::ListStyle => c.list_style = v as u8,
            F::Accelerator => c.accelerator = char::from_u32(v).filter(|&ch| ch != '\0'),
            F::MouseIcon => mouse_icon = v as u16,
            F::Picture => picture_marker = v as u16,
            F::Min => c.min = Some(v as i32),
            F::Max => c.max = Some(v as i32),
            F::Position => c.position = Some(v as i32),
            F::SmallChange => c.small_change = Some(v as i32),
            F::LargeChange => c.large_change = Some(v as i32),
            F::Orientation => c.orientation = Some(v as i32),
            F::DisplayStyle => {
                c.kind = match v {
                    1 => ControlKind::TextBox,
                    2 => ControlKind::ListBox,
                    3 | 7 => ControlKind::ComboBox,
                    4 => ControlKind::CheckBox,
                    5 => ControlKind::OptionButton,
                    6 => ControlKind::ToggleButton,
                    _ => c.kind,
                }
            }
            F::Skip => {}
        }
    }
    if let Some(v) = various {
        c.enabled = v & 2 != 0;
        c.locked = v & 4 != 0;
        c.word_wrap = v & (1 << 23) != 0;
        c.multiline = v & (1 << 28) != 0;
    }
    r.align(4);
    r.base = r.pos;
    if caption_first {
        c.caption = r.string(caption_len)?;
    }
    if mask & (1u64 << size_bit) != 0 {
        let (w, h) = r.pair()?;
        c.width = himetric_to_points(w);
        c.height = himetric_to_points(h);
    }
    if !caption_first {
        c.value = r.string(value_len)?;
        c.caption = r.string(caption_len)?;
        c.group_name = r.string(group_len)?;
    }
    if start + cb <= data.len() {
        r.pos = start + cb;
    }
    for s in streams {
        let wanted = match s {
            F::MouseIcon => mouse_icon == 0xFFFF,
            F::Picture => picture_marker == 0xFFFF,
            _ => false,
        };
        if wanted {
            let p = picture(&mut r)?;
            if *s == F::Picture {
                c.picture = Some(p);
            }
        }
    }
    if has_text_props && r.pos + 4 <= data.len() {
        c.font = Some(text_props(&mut r)?);
    }
    Ok(())
}

/// Fill `c` from its bytes in the `o` stream.
fn read_control(data: &[u8], c: &mut Control) -> Result<(), Error> {
    use F::*;
    match c.kind {
        ControlKind::Label => control_data(
            data,
            c,
            Layout {
                mask_bytes: 4,
                fields: &[
                    (0, 4, ForeColor),
                    (1, 4, BackColor),
                    (2, 4, Various),
                    (3, 4, Caption),
                    (4, 4, Skip),
                    (6, 1, Skip),
                    (7, 4, BorderColor),
                    (8, 2, Skip),
                    (9, 2, Skip),
                    (10, 2, Picture),
                    (11, 2, Accelerator),
                    (12, 2, MouseIcon),
                ],
                size_bit: 5,
                caption_first: true,
                streams: &[Picture, MouseIcon],
                text_props: true,
            },
        ),
        ControlKind::CommandButton => control_data(
            data,
            c,
            Layout {
                mask_bytes: 4,
                fields: &[
                    (0, 4, ForeColor),
                    (1, 4, BackColor),
                    (2, 4, Various),
                    (3, 4, Caption),
                    (4, 4, Skip),
                    (6, 1, Skip),
                    (7, 2, Picture),
                    (8, 2, Accelerator),
                    (10, 2, MouseIcon),
                ],
                size_bit: 5,
                caption_first: true,
                streams: &[Picture, MouseIcon],
                text_props: true,
            },
        ),
        ControlKind::TextBox
        | ControlKind::CheckBox
        | ControlKind::OptionButton
        | ControlKind::ToggleButton
        | ControlKind::ComboBox
        | ControlKind::ListBox => control_data(
            data,
            c,
            Layout {
                mask_bytes: 8,
                fields: &[
                    (0, 4, Various),
                    (1, 4, BackColor),
                    (2, 4, ForeColor),
                    (3, 4, MaxLength),
                    (4, 1, Skip),
                    (5, 1, Skip),
                    (6, 1, DisplayStyle),
                    (7, 1, Skip),
                    (9, 2, PasswordChar),
                    (10, 4, Skip),
                    (11, 2, Skip),
                    (12, 2, Skip),
                    (13, 2, ColumnCount),
                    (14, 2, Skip),
                    (15, 2, Skip),
                    (16, 1, Skip),
                    (17, 1, ListStyle),
                    (18, 1, Skip),
                    (20, 1, Skip),
                    (21, 1, Skip),
                    (22, 4, Value),
                    (23, 4, Caption),
                    (24, 4, Skip),
                    (25, 4, BorderColor),
                    (26, 4, Skip),
                    (27, 2, MouseIcon),
                    (28, 2, Picture),
                    (29, 2, Accelerator),
                    (32, 4, GroupName),
                ],
                size_bit: 8,
                caption_first: false,
                streams: &[MouseIcon, Picture],
                text_props: true,
            },
        ),
        ControlKind::Image => control_data(
            data,
            c,
            Layout {
                mask_bytes: 4,
                fields: &[
                    (3, 4, BorderColor),
                    (4, 4, BackColor),
                    (5, 1, Skip),
                    (6, 1, Skip),
                    (7, 1, Skip),
                    (8, 1, Skip),
                    (10, 2, Picture),
                    (11, 1, Skip),
                    (14, 2, MouseIcon),
                    (13, 4, Various),
                ],
                size_bit: 9,
                caption_first: true,
                streams: &[Picture, MouseIcon],
                text_props: false,
            },
        ),
        ControlKind::SpinButton => control_data(
            data,
            c,
            Layout {
                mask_bytes: 4,
                fields: &[
                    (0, 4, ForeColor),
                    (1, 4, BackColor),
                    (2, 4, Various),
                    (5, 4, Min),
                    (6, 4, Max),
                    (7, 4, Position),
                    (8, 4, Skip),
                    (9, 4, Skip),
                    (10, 4, SmallChange),
                    (11, 4, Orientation),
                    (12, 4, Skip),
                    (13, 2, MouseIcon),
                    (14, 1, Skip),
                ],
                size_bit: 3,
                caption_first: true,
                streams: &[MouseIcon],
                text_props: false,
            },
        ),
        ControlKind::ScrollBar => control_data(
            data,
            c,
            Layout {
                mask_bytes: 4,
                fields: &[
                    (0, 4, ForeColor),
                    (1, 4, BackColor),
                    (2, 4, Various),
                    (4, 1, Skip),
                    (5, 4, Min),
                    (6, 4, Max),
                    (7, 4, Position),
                    (9, 4, Skip),
                    (10, 4, Skip),
                    (11, 4, SmallChange),
                    (12, 4, LargeChange),
                    (13, 4, Orientation),
                    (14, 2, Skip),
                    (15, 4, Skip),
                    (16, 2, MouseIcon),
                ],
                size_bit: 3,
                caption_first: true,
                streams: &[MouseIcon],
                text_props: false,
            },
        ),
        ControlKind::TabStrip => tab_strip(data, c),
        _ => Ok(()),
    }
}

/// A tab strip: its size and tab captions (MS-OFORMS 2.2.9).
fn tab_strip(data: &[u8], c: &mut Control) -> Result<(), Error> {
    let mut r = R::new(data, 0);
    let _minor = r.u8()?;
    let _major = r.u8()?;
    let _cb = r.raw_u16()?;
    let mask = r.raw_u32()?;
    r.base = r.pos;
    let mut items_len = 0;
    let mut tabs = 0usize;
    for (bit, size) in [
        (0u32, 4usize),
        (1, 4),
        (2, 4),
        (5, 4),
        (6, 1),
        (8, 4),
        (9, 4),
        (11, 4),
        (12, 4),
        (15, 4),
        (17, 4),
        (18, 4),
        (20, 4),
        (21, 4),
        (22, 4),
        (23, 4),
        (24, 2),
    ] {
        if mask & (1 << bit) == 0 {
            continue;
        }
        let v = match size {
            1 => r.u8()? as u32,
            2 => r.u16()? as u32,
            _ => r.u32()?,
        };
        match bit {
            1 => c.back_color = Some(OleColor::from_u32(v)),
            2 => c.fore_color = Some(OleColor::from_u32(v)),
            5 => items_len = v,
            22 => tabs = v as usize,
            _ => {}
        }
    }
    r.align(4);
    r.base = r.pos;
    if mask & (1 << 4) != 0 {
        let (w, h) = r.pair()?;
        c.width = himetric_to_points(w);
        c.height = himetric_to_points(h);
    }
    if items_len > 0 {
        let end = r.pos + items_len as usize;
        while r.pos + 4 <= end.min(data.len()) && c.items.len() < tabs.max(1) * 4 {
            let n = r.raw_u32()?;
            c.items.push(r.string(n)?);
        }
        if tabs > 0 {
            c.items.truncate(tabs);
        }
    }
    Ok(())
}

/// The designer storage's streams, by path relative to it.
pub type Streams = BTreeMap<String, Vec<u8>>;

/// The child storage of the control with id `id` under `prefix`.
fn child_prefix(streams: &Streams, prefix: &str, id: i32) -> Option<String> {
    let candidates = [format!("{prefix}i{id:02}/"), format!("{prefix}i{id}/")];
    candidates
        .into_iter()
        .find(|p| streams.contains_key(&format!("{p}f")))
}

/// The controls of the container whose streams start with `prefix`
/// (`""` for the form itself), with its own properties.
fn container(
    streams: &Streams,
    prefix: &str,
    depth: usize,
    warnings: &mut Vec<String>,
) -> Result<(FormControl, Vec<Control>), Error> {
    if depth > 8 {
        return Err(corrupt("containers nested too deep"));
    }
    let f = streams
        .get(&format!("{prefix}f"))
        .ok_or_else(|| corrupt(format!("no {prefix}f stream")))?;
    let fc = form_control(f)?;
    let empty = Vec::new();
    let o = streams.get(&format!("{prefix}o")).unwrap_or(&empty);
    let mut at = 0usize;
    let mut controls = Vec::new();
    for s in &fc.sites {
        let kind = if s.class & 0x8000 != 0 {
            fc.class_table
                .get((s.class & 0x7FFF) as usize)
                .copied()
                .flatten()
                .unwrap_or(ControlKind::Other(s.class))
        } else {
            ControlKind::from_cache_index(s.class)
        };
        let mut c = Control::new(kind, &s.name);
        c.id = s.id;
        c.left = himetric_to_points(s.left);
        c.top = himetric_to_points(s.top);
        c.tag = s.tag.clone();
        c.tip = s.tip.clone();
        c.control_source = s.control_source.clone();
        c.row_source = s.row_source.clone();
        c.tab_index = s.tab_index;
        c.tab_stop = s.flags & 1 != 0;
        c.visible = s.flags & 2 != 0;
        c.default = s.flags & 4 != 0;
        c.cancel = s.flags & 8 != 0;
        let size = s.stream_size as usize;
        if size > 0 {
            match o.get(at..at + size) {
                Some(bytes) => {
                    if let Err(e) = read_control(bytes, &mut c) {
                        warnings.push(format!("{}: {e}", s.name));
                    }
                }
                None => warnings.push(format!("{}: its properties run past the o stream", s.name)),
            }
            at += size;
        }
        if kind.is_container() {
            match child_prefix(streams, prefix, s.id) {
                Some(p) => match container(streams, &p, depth + 1, warnings) {
                    Ok((inner, children)) => {
                        if c.caption.is_empty() {
                            c.caption = inner.caption;
                        }
                        if c.width == 0.0 {
                            c.width = inner.width;
                            c.height = inner.height;
                        }
                        if c.back_color.is_none() {
                            c.back_color = inner.back_color;
                        }
                        if c.font.is_none() {
                            c.font = inner.font;
                        }
                        c.children = children;
                    }
                    Err(e) => warnings.push(format!("{}: {e}", s.name)),
                },
                None => warnings.push(format!("{}: its storage is missing", s.name)),
            }
        }
        controls.push(c);
    }
    Ok((fc, controls))
}

/// Read a form from its designer storage's streams. `name` is the form
/// module's name. A form whose `f` stream cannot be read comes back with
/// a warning and no controls.
pub fn read_form(name: &str, streams: Streams) -> Form {
    let mut form = Form {
        name: name.to_string(),
        caption: name.to_string(),
        enabled: true,
        ..Default::default()
    };
    // The text header gives the caption and the size too.
    if let Some(header) = streams.get("\u{3}VBFrame") {
        let text = String::from_utf8_lossy(header);
        if let Some(h) = crate::frm::parse_header(&text) {
            if let Some(c) = h.props.get("caption") {
                form.caption = c.clone();
            }
            let twips = |k: &str| h.props.get(k).and_then(|v| v.parse::<f32>().ok());
            if let (Some(w), Some(hh)) = (twips("clientwidth"), twips("clientheight")) {
                form.width = w / 20.0;
                form.height = hh / 20.0;
            }
        }
    }
    let mut warnings = Vec::new();
    match container(&streams, "", 0, &mut warnings) {
        Ok((fc, controls)) => {
            if !fc.caption.is_empty() {
                form.caption = fc.caption;
            }
            if fc.width > 0.0 {
                form.width = fc.width;
                form.height = fc.height;
            }
            form.back_color = fc.back_color;
            form.fore_color = fc.fore_color;
            form.font = fc.font;
            form.picture = fc.picture;
            form.enabled = fc.enabled;
            form.controls = controls;
        }
        Err(e) => warnings.push(e.to_string()),
    }
    form.warnings = warnings;
    form.raw = streams.into_iter().collect();
    form
}

// ----- Writing (forms built from scratch) ---------------------------------

/// Streams for a designer storage holding `form` with its top-level
/// controls of the kinds this writer knows (labels, buttons, text boxes,
/// check boxes, option buttons, toggle buttons, list and combo boxes):
/// what a form exported by this crate or made in a test holds.
pub fn write_form(form: &Form) -> Streams {
    let mut streams = Streams::new();
    let mut o = Vec::new();
    let mut sites = Vec::new();
    for (i, c) in form.controls.iter().enumerate() {
        let data = write_control(c);
        let id = if c.id != 0 { c.id } else { i as i32 + 1 };
        sites.push((c, data.len() as u32, id));
        o.extend_from_slice(&data);
    }
    streams.insert("f".into(), write_form_control(form, &sites));
    streams.insert("o".into(), o);
    streams.insert(
        "\u{3}VBFrame".into(),
        format!(
            "VERSION 5.00\r\nBegin {{C62A69F0-16DC-11CE-9E98-00AA00574A4F}} {}\r\n   Caption         =   \"{}\"\r\n   ClientHeight    =   {}\r\n   ClientWidth     =   {}\r\nEnd\r\n",
            form.name,
            form.caption,
            (form.height * 20.0).round(),
            (form.width * 20.0).round()
        )
        .into_bytes(),
    );
    streams
}

struct W {
    out: Vec<u8>,
    base: usize,
}

impl W {
    fn new() -> W {
        W {
            out: Vec::new(),
            base: 0,
        }
    }
    fn align(&mut self, n: usize) {
        while !(self.out.len() - self.base).is_multiple_of(n) {
            self.out.push(0);
        }
    }
    fn u8(&mut self, v: u8) {
        self.out.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.align(2);
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.align(4);
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    /// A string stored one byte per character when it can be.
    fn string(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.align(4);
        if s.chars().all(|c| (c as u32) < 256) {
            self.out.extend(s.chars().map(|c| c as u8));
        } else {
            self.out
                .extend(s.encode_utf16().flat_map(|u| u.to_le_bytes()));
        }
        self.align(4);
    }
}

fn string_len(s: &str) -> u32 {
    if s.chars().all(|c| (c as u32) < 256) {
        s.chars().count() as u32 | 0x8000_0000
    } else {
        (s.encode_utf16().count() * 2) as u32
    }
}

fn text_props_bytes(font: &Font) -> Vec<u8> {
    let mut w = W::new();
    let mask = 1u32 | 2 | 4 | 0x10;
    w.out.extend_from_slice(&mask.to_le_bytes());
    w.base = w.out.len();
    w.u32(string_len(&font.name));
    let effects = font.bold as u32
        | (font.italic as u32) << 1
        | (font.underline as u32) << 2
        | (font.strikeout as u32) << 3;
    w.u32(effects);
    w.u32((font.size * 20.0).round() as u32);
    w.u8(font.charset);
    w.align(4);
    w.base = w.out.len();
    w.string(&font.name);
    let mut out = vec![0u8, 2];
    out.extend_from_slice(&(w.out.len() as u16).to_le_bytes());
    out.extend_from_slice(&w.out);
    out
}

fn write_control(c: &Control) -> Vec<u8> {
    // Containers keep their properties in their own storage.
    if c.kind.is_container() {
        return Vec::new();
    }
    let (w_him, h_him) = (points_to_himetric(c.width), points_to_himetric(c.height));
    let mut w = W::new();
    let various = 0x2C80_0019u32 | (c.enabled as u32) << 1 | (c.multiline as u32) << 28;
    let (major, body) = match c.kind {
        ControlKind::Label | ControlKind::CommandButton => {
            let mask: u32 = 1 << 2 | 1 << 3 | 1 << 5;
            w.out.extend_from_slice(&mask.to_le_bytes());
            w.base = w.out.len();
            w.u32(various);
            w.u32(string_len(&c.caption));
            w.align(4);
            w.base = w.out.len();
            w.string(&c.caption);
            w.u32(w_him as u32);
            w.u32(h_him as u32);
            (2u8, w.out)
        }
        _ => {
            let display = match c.kind {
                ControlKind::ListBox => 2,
                ControlKind::ComboBox => 3,
                ControlKind::CheckBox => 4,
                ControlKind::OptionButton => 5,
                ControlKind::ToggleButton => 6,
                _ => 1,
            };
            let mut mask: u64 = 1 | 1 << 6 | 1 << 8 | 1 << 22 | 1 << 23;
            if !c.group_name.is_empty() {
                mask |= 1 << 32;
            }
            w.out.extend_from_slice(&(mask as u32).to_le_bytes());
            w.out
                .extend_from_slice(&((mask >> 32) as u32).to_le_bytes());
            w.base = w.out.len();
            w.u32(various);
            w.u8(display);
            w.u32(string_len(&c.value));
            w.u32(string_len(&c.caption));
            if !c.group_name.is_empty() {
                w.u32(string_len(&c.group_name));
            }
            w.align(4);
            w.base = w.out.len();
            w.u32(w_him as u32);
            w.u32(h_him as u32);
            w.string(&c.value);
            w.string(&c.caption);
            w.string(&c.group_name);
            (2u8, w.out)
        }
    };
    let mut out = vec![0u8, major];
    out.extend_from_slice(&(body.len() as u16).to_le_bytes());
    out.extend_from_slice(&body);
    if let Some(font) = &c.font {
        out.extend_from_slice(&text_props_bytes(font));
    }
    out
}

fn cache_index(kind: ControlKind) -> u16 {
    match kind {
        ControlKind::Label => 21,
        ControlKind::CommandButton => 17,
        ControlKind::TextBox => 23,
        ControlKind::ListBox => 24,
        ControlKind::ComboBox => 25,
        ControlKind::CheckBox => 26,
        ControlKind::OptionButton => 27,
        ControlKind::ToggleButton => 28,
        ControlKind::Image => 12,
        ControlKind::Frame => 14,
        ControlKind::SpinButton => 16,
        ControlKind::TabStrip => 18,
        ControlKind::ScrollBar => 47,
        ControlKind::MultiPage => 57,
        ControlKind::Page => 7,
        ControlKind::Other(i) => i,
    }
}

fn write_form_control(form: &Form, sites: &[(&Control, u32, i32)]) -> Vec<u8> {
    let mut w = W::new();
    let mask: u32 = 1 << 6 | 1 << 10 | 1 << 19;
    w.out.extend_from_slice(&mask.to_le_bytes());
    w.base = w.out.len();
    // Enabled, and no class table saved.
    w.u32(0x0000_8004);
    w.u32(string_len(&form.caption));
    w.align(4);
    w.base = w.out.len();
    w.u32(points_to_himetric(form.width) as u32);
    w.u32(points_to_himetric(form.height) as u32);
    w.string(&form.caption);
    let mut out = vec![0u8, 4];
    out.extend_from_slice(&(w.out.len() as u16).to_le_bytes());
    out.extend_from_slice(&w.out);
    // Sites: depths and types, then each site.
    let mut depths = Vec::new();
    for _ in sites {
        depths.extend_from_slice(&[0u8, 1]);
    }
    while depths.len() % 4 != 0 {
        depths.push(0);
    }
    let mut site_bytes = Vec::new();
    for (c, size, id) in sites {
        let mut s = W::new();
        let mask: u32 = 1 | 4 | 0x10 | 0x20 | 0x40 | 0x80 | 0x100;
        s.out.extend_from_slice(&mask.to_le_bytes());
        s.base = s.out.len();
        s.u32(string_len(&c.name));
        s.u32(*id as u32);
        let flags = 0x10u32 | c.tab_stop as u32 | (c.visible as u32) << 1;
        s.u32(flags);
        s.u32(*size);
        s.u16(c.tab_index as u16);
        s.u16(cache_index(c.kind));
        s.align(4);
        s.base = s.out.len();
        s.string(&c.name);
        s.u32(points_to_himetric(c.left) as u32);
        s.u32(points_to_himetric(c.top) as u32);
        site_bytes.extend_from_slice(&0u16.to_le_bytes());
        site_bytes.extend_from_slice(&(s.out.len() as u16).to_le_bytes());
        site_bytes.extend_from_slice(&s.out);
    }
    out.extend_from_slice(&(sites.len() as u32).to_le_bytes());
    out.extend_from_slice(&((depths.len() + site_bytes.len()) as u32).to_le_bytes());
    out.extend_from_slice(&depths);
    out.extend_from_slice(&site_bytes);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Form {
        let mut label = Control::new(ControlKind::Label, "lblWidth");
        label.caption = "Width (mm):".into();
        label.left = 12.0;
        label.top = 6.0;
        label.width = 60.0;
        label.height = 12.0;
        label.tab_index = 0;
        let mut text = Control::new(ControlKind::TextBox, "txtWidth");
        text.value = "25.4".into();
        text.left = 78.0;
        text.top = 6.0;
        text.width = 48.0;
        text.height = 18.0;
        text.tab_index = 1;
        text.font = Some(Font {
            name: "Tahoma".into(),
            size: 8.25,
            bold: true,
            ..Default::default()
        });
        let mut check = Control::new(ControlKind::CheckBox, "chkAll");
        check.caption = "Every page".into();
        check.value = "1".into();
        check.enabled = false;
        check.tab_index = 2;
        let mut opt = Control::new(ControlKind::OptionButton, "optLeft");
        opt.caption = "Left".into();
        opt.group_name = "Side".into();
        opt.tab_index = 3;
        let mut ok = Control::new(ControlKind::CommandButton, "cmdOK");
        ok.caption = "OK".into();
        ok.left = 90.0;
        ok.top = 60.0;
        ok.width = 54.0;
        ok.height = 24.0;
        ok.tab_index = 4;
        let mut name = Control::new(ControlKind::TextBox, "txtNome");
        name.value = "Ação 日本".into();
        name.multiline = true;
        name.tab_index = 5;
        Form {
            name: "frmResize".into(),
            caption: "Resize selection".into(),
            width: 220.0,
            height: 110.0,
            enabled: true,
            controls: vec![label, text, check, opt, ok, name],
            ..Default::default()
        }
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.05
    }

    #[test]
    fn a_written_form_reads_back_with_its_controls() {
        let form = sample();
        let back = read_form("frmResize", write_form(&form));
        assert!(back.warnings.is_empty(), "{:?}", back.warnings);
        assert_eq!(back.caption, "Resize selection");
        assert!(
            close(back.width, 220.0) && close(back.height, 110.0),
            "{} {}",
            back.width,
            back.height
        );
        assert_eq!(back.controls.len(), 6);
        let label = back.control("LBLWIDTH").unwrap();
        assert_eq!(label.kind, ControlKind::Label);
        assert_eq!(label.caption, "Width (mm):");
        assert!(close(label.left, 12.0) && close(label.top, 6.0));
        assert!(close(label.width, 60.0) && close(label.height, 12.0));
        let text = back.control("txtWidth").unwrap();
        assert_eq!(text.kind, ControlKind::TextBox);
        assert_eq!(text.value, "25.4");
        assert_eq!(text.tab_index, 1);
        let font = text.font.as_ref().unwrap();
        assert_eq!(font.name, "Tahoma");
        assert!(font.bold && close(font.size, 8.25));
        let check = back.control("chkAll").unwrap();
        assert_eq!(check.kind, ControlKind::CheckBox);
        assert_eq!(
            (check.caption.as_str(), check.value.as_str()),
            ("Every page", "1")
        );
        assert!(!check.enabled);
        let opt = back.control("optLeft").unwrap();
        assert_eq!(opt.kind, ControlKind::OptionButton);
        assert_eq!(opt.group_name, "Side");
        assert_eq!(back.control("cmdOK").unwrap().caption, "OK");
        let name = back.control("txtNome").unwrap();
        assert_eq!(name.value, "Ação 日本");
        assert!(name.multiline);
        // The streams are kept for writing the form back.
        assert!(back.raw.iter().any(|(p, _)| p == "f"));
    }

    #[test]
    fn broken_streams_leave_warnings_not_panics() {
        let streams = write_form(&sample());
        let f = streams["f"].clone();
        let o = streams["o"].clone();
        for n in 0..f.len() {
            let mut s = streams.clone();
            s.insert("f".into(), f[..n].to_vec());
            let form = read_form("x", s);
            assert!(!form.warnings.is_empty() || form.controls.len() <= 6);
        }
        for n in 0..o.len() {
            let mut s = streams.clone();
            s.insert("o".into(), o[..n].to_vec());
            let _ = read_form("x", s);
        }
        // Garbage of every length.
        for n in 0..64 {
            let mut s = Streams::new();
            s.insert("f".into(), (0..n).map(|i| (i * 37 + 11) as u8).collect());
            s.insert("o".into(), vec![0xFF; n]);
            let _ = read_form("x", s);
        }
        let none = read_form("x", Streams::new());
        assert!(!none.warnings.is_empty());
        assert_eq!(none.caption, "x");
    }

    #[test]
    fn colours_kinds_and_units() {
        assert_eq!(OleColor::from_u32(0x0000_00FF), OleColor::Rgb(255, 0, 0));
        assert_eq!(OleColor::from_u32(0x8000_000F), OleColor::System(0x0F));
        assert_eq!(OleColor::System(5).rgb(), (255, 255, 255));
        assert_eq!(ControlKind::from_cache_index(57), ControlKind::MultiPage);
        assert_eq!(
            ControlKind::from_clsid("{d7053240-ce69-11cd-a777-00dd01143c57}"),
            Some(ControlKind::CommandButton)
        );
        assert_eq!(
            ControlKind::from_type_name("VB.CommandButton"),
            Some(ControlKind::CommandButton)
        );
        assert_eq!(
            ControlKind::from_type_name("MSForms.TextBox"),
            Some(ControlKind::TextBox)
        );
        assert!(close(himetric_to_points(2540), 72.0));
        assert_eq!(points_to_himetric(72.0), 2540);
    }

    #[test]
    fn a_frame_keeps_its_controls_in_its_own_storage() {
        // The frame's own f and o streams under i07/.
        let mut inner = sample();
        inner.caption = "Options".into();
        inner.width = 100.0;
        inner.height = 50.0;
        inner.controls.truncate(2);
        let inner_streams = write_form(&inner);
        let mut frame = Control::new(ControlKind::Frame, "fraOptions");
        frame.id = 7;
        frame.left = 6.0;
        frame.top = 30.0;
        let mut outer = sample();
        outer.controls = vec![frame];
        let mut streams = write_form(&outer);
        // A frame has no properties in the o stream.
        streams.insert("o".into(), Vec::new());
        for (k, v) in inner_streams {
            if k != "\u{3}VBFrame" {
                streams.insert(format!("i07/{k}"), v);
            }
        }
        let form = read_form("frmOuter", streams);
        let frame = &form.controls[0];
        assert_eq!(frame.kind, ControlKind::Frame);
        assert_eq!(frame.id, 7);
        assert_eq!(frame.caption, "Options");
        assert!(close(frame.width, 100.0));
        assert_eq!(frame.children.len(), 2);
        assert_eq!(form.control("txtWidth").unwrap().value, "25.4");
    }
}
