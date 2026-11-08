//! The user forms of a running VBA macro: their state, shared between the
//! macro's thread (which reads and sets control properties) and the UI
//! (which draws the forms and records what the user does). See
//! docs/behavior/vba.md, "Forms on screen".

use serde_json::{json, Value};
use tracedraw_vba_project::{Control, ControlKind, Form};

/// A font as forms use it.
#[derive(Debug, Clone, PartialEq)]
pub struct FormFont {
    pub name: String,
    /// Points.
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for FormFont {
    fn default() -> Self {
        FormFont {
            name: "Tahoma".into(),
            size: 8.0,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

impl From<&tracedraw_vba_project::Font> for FormFont {
    fn from(f: &tracedraw_vba_project::Font) -> Self {
        FormFont {
            name: f.name.clone(),
            size: if f.size > 0.0 { f.size } else { 8.0 },
            bold: f.bold,
            italic: f.italic,
            underline: f.underline,
        }
    }
}

/// One control of a shown form.
#[derive(Debug, Clone, PartialEq)]
pub struct LiveControl {
    pub name: String,
    pub kind: ControlKind,
    /// The container it sits in (index in [`LiveForm::controls`]).
    pub parent: Option<usize>,
    /// Points, from the container's top left.
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub caption: String,
    /// A text box's or combo box's text.
    pub text: String,
    /// Check boxes, option and toggle buttons.
    pub checked: bool,
    /// Spin buttons, scroll bars, multi-pages and tab strips.
    pub number: i32,
    pub min: i32,
    pub max: i32,
    pub small_change: i32,
    pub large_change: i32,
    /// List and combo boxes: rows of columns.
    pub list: Vec<Vec<String>>,
    pub list_index: i32,
    /// Rows chosen in a list box that allows several.
    pub selected: Vec<bool>,
    /// 0 one row, 1 several, 2 extended (list boxes).
    pub multi_select: i32,
    pub column_count: i32,
    /// Combo boxes: 2 is a drop-down list (no typing).
    pub style: i32,
    pub enabled: bool,
    pub visible: bool,
    pub locked: bool,
    pub tag: String,
    pub tip: String,
    pub group_name: String,
    pub max_length: u32,
    pub multiline: bool,
    pub password_char: Option<char>,
    pub default: bool,
    pub cancel: bool,
    pub fore: Option<(u8, u8, u8)>,
    pub back: Option<(u8, u8, u8)>,
    pub font: FormFont,
    pub picture: Option<Vec<u8>>,
    /// Tab strips: the tabs' captions.
    pub items: Vec<String>,
    pub tab_index: i32,
}

impl LiveControl {
    fn from_model(c: &Control, parent: Option<usize>, form_font: &FormFont) -> LiveControl {
        let number = c.position.unwrap_or(0);
        let checked = matches!(
            c.kind,
            ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton
        ) && matches!(c.value.trim(), "1" | "-1" | "True" | "true");
        let text = match c.kind {
            ControlKind::TextBox | ControlKind::ComboBox => c.value.clone(),
            _ => String::new(),
        };
        let number = match c.kind {
            ControlKind::MultiPage | ControlKind::TabStrip => {
                c.value.trim().parse().unwrap_or(number)
            }
            _ => number,
        };
        LiveControl {
            name: c.name.clone(),
            kind: c.kind,
            parent,
            left: c.left,
            top: c.top,
            width: c.width,
            height: c.height,
            caption: c.caption.clone(),
            text,
            checked,
            number,
            min: c.min.unwrap_or(0),
            max: c.max.unwrap_or(match c.kind {
                ControlKind::ScrollBar => 32767,
                _ => 100,
            }),
            small_change: c.small_change.unwrap_or(1),
            large_change: c.large_change.unwrap_or(1),
            list: Vec::new(),
            list_index: -1,
            selected: Vec::new(),
            multi_select: 0,
            column_count: c.column_count.max(1) as i32,
            style: 0,
            enabled: c.enabled,
            visible: c.visible,
            locked: c.locked,
            tag: c.tag.clone(),
            tip: c.tip.clone(),
            group_name: c.group_name.clone(),
            max_length: c.max_length,
            multiline: c.multiline,
            password_char: c.password_char,
            default: c.default,
            cancel: c.cancel,
            fore: c.fore_color.map(|o| o.rgb()),
            back: c.back_color.map(|o| o.rgb()),
            font: c
                .font
                .as_ref()
                .map(FormFont::from)
                .unwrap_or_else(|| form_font.clone()),
            picture: c.picture.clone(),
            items: c.items.clone(),
            tab_index: c.tab_index as i32,
        }
    }

    /// A list or combo box's text of row `i`, column `c`.
    pub fn cell(&self, row: usize, col: usize) -> String {
        self.list
            .get(row)
            .and_then(|r| r.get(col))
            .cloned()
            .unwrap_or_default()
    }

    /// The value VBA reads as `Value`.
    pub fn value(&self) -> Value {
        match self.kind {
            ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton => {
                json!(self.checked)
            }
            ControlKind::TextBox | ControlKind::ComboBox => json!(self.text),
            ControlKind::ListBox => {
                if self.list_index >= 0 {
                    json!(self.cell(self.list_index as usize, 0))
                } else {
                    Value::Null
                }
            }
            ControlKind::SpinButton
            | ControlKind::ScrollBar
            | ControlKind::MultiPage
            | ControlKind::TabStrip => json!(self.number),
            _ => json!(self.caption),
        }
    }
}

/// A form a macro made (shown or not).
#[derive(Debug, Clone, PartialEq)]
pub struct LiveForm {
    pub id: u32,
    /// The form's module.
    pub module: String,
    pub caption: String,
    /// Inside size in points.
    pub width: f32,
    pub height: f32,
    pub back: Option<(u8, u8, u8)>,
    pub fore: Option<(u8, u8, u8)>,
    pub font: FormFont,
    pub picture: Option<Vec<u8>>,
    pub enabled: bool,
    pub visible: bool,
    pub modal: bool,
    pub controls: Vec<LiveControl>,
    /// The control with the keyboard focus (UI side).
    pub focus: Option<String>,
    pub tag: String,
}

impl LiveForm {
    /// A form from its designer model (or an empty one when the project
    /// has no designer for it).
    pub fn from_model(id: u32, module: &str, model: Option<&Form>) -> LiveForm {
        let font = model
            .and_then(|f| f.font.as_ref())
            .map(FormFont::from)
            .unwrap_or_default();
        let mut form = LiveForm {
            id,
            module: module.to_string(),
            caption: model
                .map(|f| f.caption.clone())
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| module.to_string()),
            width: model.map(|f| f.width).filter(|w| *w > 0.0).unwrap_or(240.0),
            height: model
                .map(|f| f.height)
                .filter(|h| *h > 0.0)
                .unwrap_or(180.0),
            back: model.and_then(|f| f.back_color).map(|o| o.rgb()),
            fore: model.and_then(|f| f.fore_color).map(|o| o.rgb()),
            font: font.clone(),
            picture: model.and_then(|f| f.picture.clone()),
            enabled: model.is_none_or(|f| f.enabled),
            visible: false,
            modal: true,
            controls: Vec::new(),
            focus: None,
            tag: String::new(),
        };
        if let Some(m) = model {
            fn add(form: &mut LiveForm, cs: &[Control], parent: Option<usize>, font: &FormFont) {
                for c in cs {
                    let i = form.controls.len();
                    form.controls.push(LiveControl::from_model(c, parent, font));
                    add(form, &c.children, Some(i), font);
                }
            }
            add(&mut form, &m.controls, None, &font);
        }
        form
    }

    pub fn control(&self, name: &str) -> Option<&LiveControl> {
        self.controls
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn control_mut(&mut self, name: &str) -> Option<&mut LiveControl> {
        self.controls
            .iter_mut()
            .find(|c| c.name.eq_ignore_ascii_case(name))
    }

    /// The pages of multi-page `i`, in order.
    pub fn pages_of(&self, i: usize) -> Vec<usize> {
        self.controls
            .iter()
            .enumerate()
            .filter(|(_, c)| c.parent == Some(i) && c.kind == ControlKind::Page)
            .map(|(j, _)| j)
            .collect()
    }

    /// Whether control `i` shows: itself, its containers, and the page it
    /// sits on is the multi-page's current one.
    pub fn shows(&self, i: usize) -> bool {
        let c = &self.controls[i];
        if !c.visible {
            return false;
        }
        let Some(p) = c.parent else {
            return true;
        };
        let parent = &self.controls[p];
        if c.kind == ControlKind::Page && parent.kind == ControlKind::MultiPage {
            let pages = self.pages_of(p);
            let current = pages.get(parent.number.max(0) as usize).copied();
            if current != Some(i) {
                return false;
            }
        }
        self.shows(p)
    }

    /// Tick option button `name`: the others of its group (same group
    /// name, or same container when it has none) go off. Returns the ones
    /// turned off.
    pub fn tick_option(&mut self, name: &str) -> Vec<String> {
        let Some(i) = self
            .controls
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(name))
        else {
            return Vec::new();
        };
        let (group, parent) = (self.controls[i].group_name.clone(), self.controls[i].parent);
        self.controls[i].checked = true;
        let mut off = Vec::new();
        for (j, c) in self.controls.iter_mut().enumerate() {
            if j == i || c.kind != ControlKind::OptionButton || !c.checked {
                continue;
            }
            let same = if group.is_empty() {
                c.group_name.is_empty() && c.parent == parent
            } else {
                c.group_name.eq_ignore_ascii_case(&group)
            };
            if same {
                c.checked = false;
                off.push(c.name.clone());
            }
        }
        off
    }
}

/// Every form of a run.
#[derive(Debug, Default)]
pub struct Forms {
    pub list: Vec<LiveForm>,
    next_id: u32,
}

fn rgb_of(v: &Value) -> Option<(u8, u8, u8)> {
    let n = v.as_f64()? as i64;
    // OLE colours: &H00BBGGRR.
    Some((
        (n & 0xFF) as u8,
        ((n >> 8) & 0xFF) as u8,
        ((n >> 16) & 0xFF) as u8,
    ))
}

fn ole(c: Option<(u8, u8, u8)>, default: (u8, u8, u8)) -> Value {
    let (r, g, b) = c.unwrap_or(default);
    json!(r as i64 | (g as i64) << 8 | (b as i64) << 16)
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", f as i64)
            } else {
                format!("{f}")
            }
        }
        other => other.to_string(),
    }
}

fn truth(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        Value::String(s) => {
            let l = s.trim().to_ascii_lowercase();
            l == "true" || l.parse::<f64>().is_ok_and(|n| n != 0.0)
        }
        _ => false,
    }
}

fn int_of(v: &Value) -> i32 {
    match v {
        Value::Number(n) => n.as_f64().unwrap_or(0.0).round() as i32,
        Value::Bool(b) => -(*b as i32),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(0.0).round() as i32,
        _ => 0,
    }
}

/// Rows from what a macro assigns to `List`: a list of values or of rows.
fn rows_of(v: &Value) -> Vec<Vec<String>> {
    match v {
        Value::Array(rows) => rows
            .iter()
            .map(|r| match r {
                Value::Array(cols) => cols.iter().map(text_of).collect(),
                other => vec![text_of(other)],
            })
            .collect(),
        Value::Null => Vec::new(),
        other => vec![vec![text_of(other)]],
    }
}

impl Forms {
    /// A new form for module `module` from its designer model.
    pub fn create(&mut self, module: &str, model: Option<&Form>) -> &LiveForm {
        self.next_id += 1;
        let form = LiveForm::from_model(self.next_id, module, model);
        self.list.push(form);
        self.list.last().expect("just added")
    }

    pub fn get(&self, id: u32) -> Option<&LiveForm> {
        self.list.iter().find(|f| f.id == id)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut LiveForm> {
        self.list.iter_mut().find(|f| f.id == id)
    }

    pub fn remove(&mut self, id: u32) {
        self.list.retain(|f| f.id != id);
    }

    /// Any form on screen.
    pub fn any_visible(&self) -> bool {
        self.list.iter().any(|f| f.visible)
    }

    /// A modal form on screen (it blocks the rest of the app).
    pub fn modal_visible(&self) -> bool {
        self.list.iter().any(|f| f.visible && f.modal)
    }

    /// Property `prop` of control `ctl` of form `id` ("" for the form).
    pub fn get_prop(&self, id: u32, ctl: &str, prop: &str) -> Result<Value, String> {
        let form = self.get(id).ok_or("the form was unloaded")?;
        let p = prop.to_ascii_lowercase();
        if ctl.is_empty() {
            return Ok(match p.as_str() {
                "caption" => json!(form.caption),
                "width" => json!(form.width + 6.0),
                "height" => json!(form.height + 24.0),
                "insidewidth" => json!(form.width),
                "insideheight" => json!(form.height),
                "visible" => json!(form.visible),
                "enabled" => json!(form.enabled),
                "tag" => json!(form.tag),
                "name" => json!(form.module),
                "backcolor" => ole(form.back, (240, 240, 240)),
                "forecolor" => ole(form.fore, (0, 0, 0)),
                "left" | "top" | "startupposition" | "scrolltop" | "scrollleft" => json!(0),
                "activecontrol" => json!(form.focus.clone().unwrap_or_default()),
                "font.name" => json!(form.font.name),
                "font.size" => json!(form.font.size),
                "font.bold" => json!(form.font.bold),
                "font.italic" => json!(form.font.italic),
                _ => return Err(format!("the form has no property {prop}")),
            });
        }
        let c = form
            .control(ctl)
            .ok_or_else(|| format!("no control named {ctl}"))?;
        Ok(match p.as_str() {
            "name" => json!(c.name),
            "caption" => json!(c.caption),
            "value" => c.value(),
            "text" => match c.kind {
                ControlKind::ListBox => json!(if c.list_index >= 0 {
                    c.cell(c.list_index as usize, 0)
                } else {
                    String::new()
                }),
                ControlKind::TextBox | ControlKind::ComboBox => json!(c.text),
                _ => json!(c.caption),
            },
            "seltext" => json!(c.text),
            "sellength" => json!(c.text.chars().count()),
            "selstart" => json!(0),
            "enabled" => json!(c.enabled),
            "visible" => json!(c.visible),
            "locked" => json!(c.locked),
            "tag" => json!(c.tag),
            "controltiptext" => json!(c.tip),
            "groupname" => json!(c.group_name),
            "left" => json!(c.left),
            "top" => json!(c.top),
            "width" => json!(c.width),
            "height" => json!(c.height),
            "listindex" => json!(c.list_index),
            "listcount" => json!(c.list.len()),
            "columncount" => json!(c.column_count),
            "multiselect" => json!(c.multi_select),
            "style" => json!(c.style),
            "min" => json!(c.min),
            "max" => json!(c.max),
            "smallchange" => json!(c.small_change),
            "largechange" => json!(c.large_change),
            "maxlength" => json!(c.max_length),
            "multiline" => json!(c.multiline),
            "passwordchar" => json!(c.password_char.map(String::from).unwrap_or_default()),
            "default" => json!(c.default),
            "cancel" => json!(c.cancel),
            "tabindex" => json!(c.tab_index),
            "tabstop" => json!(true),
            "backcolor" => ole(c.back, (255, 255, 255)),
            "forecolor" => ole(c.fore, (0, 0, 0)),
            "font.name" => json!(c.font.name),
            "font.size" => json!(c.font.size),
            "font.bold" => json!(c.font.bold),
            "font.italic" => json!(c.font.italic),
            "font.underline" => json!(c.font.underline),
            "list" => json!(c.list),
            "selected" => json!(c.selected),
            "kind" => json!(kind_name(c.kind)),
            "specialeffect" | "borderstyle" | "textalign" | "autosize" | "wordwrap" => json!(0),
            _ => return Err(format!("{} has no property {prop}", c.name)),
        })
    }

    /// Set property `prop`. Returns what changed so the caller fires the
    /// events: (`Change`, `Click`) and the option buttons turned off.
    pub fn set_prop(
        &mut self,
        id: u32,
        ctl: &str,
        prop: &str,
        v: &Value,
    ) -> Result<SetOutcome, String> {
        let form = self.get_mut(id).ok_or("the form was unloaded")?;
        let p = prop.to_ascii_lowercase();
        let mut out = SetOutcome::default();
        if ctl.is_empty() {
            match p.as_str() {
                "caption" => form.caption = text_of(v),
                "width" => form.width = (v.as_f64().unwrap_or(0.0) as f32 - 6.0).max(20.0),
                "height" => form.height = (v.as_f64().unwrap_or(0.0) as f32 - 24.0).max(20.0),
                "insidewidth" => form.width = v.as_f64().unwrap_or(0.0) as f32,
                "insideheight" => form.height = v.as_f64().unwrap_or(0.0) as f32,
                "enabled" => form.enabled = truth(v),
                "tag" => form.tag = text_of(v),
                "backcolor" => form.back = rgb_of(v),
                "forecolor" => form.fore = rgb_of(v),
                "font.name" => form.font.name = text_of(v),
                "font.size" => form.font.size = v.as_f64().unwrap_or(8.0) as f32,
                "font.bold" => form.font.bold = truth(v),
                "font.italic" => form.font.italic = truth(v),
                // Kept by the form itself; nothing to draw.
                "left" | "top" | "startupposition" | "scrolltop" | "scrollleft" | "visible" => {}
                _ => return Err(format!("the form has no property {prop}")),
            }
            return Ok(out);
        }
        let kind = form
            .control(ctl)
            .ok_or_else(|| format!("no control named {ctl}"))?
            .kind;
        if kind == ControlKind::OptionButton && matches!(p.as_str(), "value") && truth(v) {
            let was = form.control(ctl).is_some_and(|c| c.checked);
            out.off = form.tick_option(ctl);
            out.changed = !was;
            out.click = !was;
            return Ok(out);
        }
        let c = form.control_mut(ctl).expect("checked above");
        match p.as_str() {
            "caption" => c.caption = text_of(v),
            "value" | "text" => match c.kind {
                ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton => {
                    let t = truth(v);
                    out.changed = t != c.checked;
                    out.click = out.changed;
                    c.checked = t;
                }
                ControlKind::TextBox => {
                    let t = text_of(v);
                    out.changed = t != c.text;
                    c.text = t;
                }
                ControlKind::ComboBox | ControlKind::ListBox => {
                    let t = text_of(v);
                    let row = c.list.iter().position(|r| r.first() == Some(&t));
                    let idx = row.map(|r| r as i32).unwrap_or(-1);
                    out.changed =
                        idx != c.list_index || (c.kind == ControlKind::ComboBox && t != c.text);
                    out.click = idx != c.list_index && idx >= 0;
                    c.list_index = idx;
                    if c.kind == ControlKind::ComboBox {
                        c.text = t;
                    }
                }
                ControlKind::SpinButton
                | ControlKind::ScrollBar
                | ControlKind::MultiPage
                | ControlKind::TabStrip => {
                    let n = int_of(v);
                    let n = if matches!(c.kind, ControlKind::SpinButton | ControlKind::ScrollBar) {
                        n.clamp(c.min.min(c.max), c.max.max(c.min))
                    } else {
                        n
                    };
                    out.changed = n != c.number;
                    c.number = n;
                }
                _ => c.caption = text_of(v),
            },
            "listindex" => {
                let n = int_of(v);
                if n < -1 || n >= c.list.len() as i32 {
                    return Err("Invalid property value (ListIndex)".into());
                }
                out.changed = n != c.list_index;
                out.click = out.changed;
                c.list_index = n;
                if c.kind == ControlKind::ComboBox {
                    c.text = if n >= 0 {
                        c.cell(n as usize, 0)
                    } else {
                        String::new()
                    };
                }
            }
            "enabled" => c.enabled = truth(v),
            "visible" => c.visible = truth(v),
            "locked" => c.locked = truth(v),
            "tag" => c.tag = text_of(v),
            "controltiptext" => c.tip = text_of(v),
            "groupname" => c.group_name = text_of(v),
            "left" => c.left = v.as_f64().unwrap_or(0.0) as f32,
            "top" => c.top = v.as_f64().unwrap_or(0.0) as f32,
            "width" => c.width = v.as_f64().unwrap_or(0.0) as f32,
            "height" => c.height = v.as_f64().unwrap_or(0.0) as f32,
            "columncount" => c.column_count = int_of(v).max(1),
            "multiselect" => c.multi_select = int_of(v),
            "style" => c.style = int_of(v),
            "min" => c.min = int_of(v),
            "max" => c.max = int_of(v),
            "smallchange" => c.small_change = int_of(v),
            "largechange" => c.large_change = int_of(v),
            "maxlength" => c.max_length = int_of(v).max(0) as u32,
            "multiline" => c.multiline = truth(v),
            "passwordchar" => c.password_char = text_of(v).chars().next(),
            "default" => c.default = truth(v),
            "cancel" => c.cancel = truth(v),
            "tabindex" => c.tab_index = int_of(v),
            "backcolor" => c.back = rgb_of(v),
            "forecolor" => c.fore = rgb_of(v),
            "font.name" => c.font.name = text_of(v),
            "font.size" => c.font.size = v.as_f64().unwrap_or(8.0) as f32,
            "font.bold" => c.font.bold = truth(v),
            "font.italic" => c.font.italic = truth(v),
            "font.underline" => c.font.underline = truth(v),
            "list" => {
                c.list = rows_of(v);
                c.selected = vec![false; c.list.len()];
                c.list_index = -1;
            }
            "seltext" => c.text = text_of(v),
            // Accepted and not drawn.
            "specialeffect" | "borderstyle" | "textalign" | "autosize" | "wordwrap" | "tabstop"
            | "selstart" | "sellength" | "boundcolumn" | "textcolumn" | "columnwidths"
            | "columnheads" | "matchentry" | "showdropbuttonwhen" | "picture" | "mousepointer" => {}
            _ => return Err(format!("{} has no property {prop}", c.name)),
        }
        Ok(out)
    }

    /// Run method `method` of control `ctl` (AddItem, RemoveItem, Clear,
    /// SetFocus, List(row, column) and Selected(row)).
    pub fn call(
        &mut self,
        id: u32,
        ctl: &str,
        method: &str,
        args: &[Value],
    ) -> Result<Value, String> {
        let form = self.get_mut(id).ok_or("the form was unloaded")?;
        let m = method.to_ascii_lowercase();
        if m == "setfocus" {
            form.focus = Some(ctl.to_string());
            return Ok(Value::Null);
        }
        if m == "pages" {
            let i = form
                .controls
                .iter()
                .position(|c| c.name.eq_ignore_ascii_case(ctl))
                .ok_or_else(|| format!("no control named {ctl}"))?;
            let names: Vec<String> = form
                .pages_of(i)
                .into_iter()
                .map(|j| form.controls[j].name.clone())
                .collect();
            return Ok(json!(names));
        }
        let c = form
            .control_mut(ctl)
            .ok_or_else(|| format!("no control named {ctl}"))?;
        let row = |c: &LiveControl, v: Option<&Value>| -> Result<usize, String> {
            let i = v.map(int_of).unwrap_or(-1);
            if i < 0 || i as usize >= c.list.len() {
                return Err("Invalid argument (row out of range)".into());
            }
            Ok(i as usize)
        };
        match m.as_str() {
            "additem" => {
                let text = args.first().map(text_of).unwrap_or_default();
                let at = args
                    .get(1)
                    .filter(|v| !v.is_null())
                    .map(int_of)
                    .unwrap_or(c.list.len() as i32);
                if at < 0 || at as usize > c.list.len() {
                    return Err("Invalid argument (AddItem index)".into());
                }
                c.list.insert(at as usize, vec![text]);
                c.selected.insert(at as usize, false);
                if c.list_index >= at && c.list_index >= 0 && (at as usize) < c.list.len() - 1 {
                    c.list_index += 1;
                }
            }
            "removeitem" => {
                let i = row(c, args.first())?;
                c.list.remove(i);
                if i < c.selected.len() {
                    c.selected.remove(i);
                }
                if c.list_index == i as i32 {
                    c.list_index = -1;
                } else if c.list_index > i as i32 {
                    c.list_index -= 1;
                }
            }
            "clear" => {
                c.list.clear();
                c.selected.clear();
                c.list_index = -1;
                if c.kind == ControlKind::ComboBox {
                    c.text.clear();
                }
            }
            "getlist" => {
                let i = row(c, args.first())?;
                let col = args.get(1).map(int_of).unwrap_or(0).max(0) as usize;
                return Ok(json!(c.cell(i, col)));
            }
            "setlist" => {
                let i = row(c, args.first())?;
                let col = args.get(1).map(int_of).unwrap_or(0).max(0) as usize;
                let v = args.get(2).map(text_of).unwrap_or_default();
                let r = &mut c.list[i];
                while r.len() <= col {
                    r.push(String::new());
                }
                r[col] = v;
                if c.kind == ControlKind::ComboBox && c.list_index == i as i32 && col == 0 {
                    c.text = c.cell(i, 0);
                }
            }
            "getselected" => {
                let i = row(c, args.first())?;
                return Ok(json!(
                    c.selected.get(i).copied().unwrap_or(false)
                        || (c.multi_select == 0 && c.list_index == i as i32)
                ));
            }
            "setselected" => {
                let i = row(c, args.first())?;
                let on = args.get(1).map(truth).unwrap_or(false);
                if c.selected.len() < c.list.len() {
                    c.selected.resize(c.list.len(), false);
                }
                c.selected[i] = on;
                if c.multi_select == 0 {
                    c.list_index = if on { i as i32 } else { -1 };
                }
            }
            "repaint" | "move" | "zorder" => {}
            _ => return Err(format!("{} has no method {method}", c.name)),
        }
        Ok(Value::Null)
    }
}

/// What setting a property changed.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SetOutcome {
    pub changed: bool,
    pub click: bool,
    /// Option buttons of the same group turned off.
    pub off: Vec<String>,
}

pub fn kind_name(k: ControlKind) -> &'static str {
    match k {
        ControlKind::Label => "Label",
        ControlKind::TextBox => "TextBox",
        ControlKind::CommandButton => "CommandButton",
        ControlKind::CheckBox => "CheckBox",
        ControlKind::OptionButton => "OptionButton",
        ControlKind::ToggleButton => "ToggleButton",
        ControlKind::ComboBox => "ComboBox",
        ControlKind::ListBox => "ListBox",
        ControlKind::Frame => "Frame",
        ControlKind::MultiPage => "MultiPage",
        ControlKind::Page => "Page",
        ControlKind::Image => "Image",
        ControlKind::SpinButton => "SpinButton",
        ControlKind::ScrollBar => "ScrollBar",
        ControlKind::TabStrip => "TabStrip",
        ControlKind::Other(_) => "Control",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> Forms {
        let mut model = Form {
            name: "F".into(),
            caption: "Options".into(),
            width: 200.0,
            height: 120.0,
            enabled: true,
            ..Default::default()
        };
        let mut a = Control::new(ControlKind::OptionButton, "optA");
        a.value = "1".into();
        let b = Control::new(ControlKind::OptionButton, "optB");
        let list = Control::new(ControlKind::ListBox, "lst");
        let mut text = Control::new(ControlKind::TextBox, "tb");
        text.value = "hi".into();
        model.controls = vec![a, b, list, text];
        let mut forms = Forms::default();
        forms.create("F", Some(&model));
        forms
    }

    #[test]
    fn values_lists_and_option_groups() {
        let mut f = form();
        assert_eq!(f.get_prop(1, "", "Caption").unwrap(), json!("Options"));
        assert_eq!(f.get_prop(1, "tb", "Text").unwrap(), json!("hi"));
        assert_eq!(f.get_prop(1, "optA", "Value").unwrap(), json!(true));
        let o = f.set_prop(1, "optB", "Value", &json!(true)).unwrap();
        assert_eq!(o.off, vec!["optA".to_string()]);
        assert!(o.click);
        assert_eq!(f.get_prop(1, "optA", "Value").unwrap(), json!(false));
        f.call(1, "lst", "AddItem", &[json!("one")]).unwrap();
        f.call(1, "lst", "AddItem", &[json!("zero"), json!(0)])
            .unwrap();
        assert_eq!(f.get_prop(1, "lst", "ListCount").unwrap(), json!(2));
        assert_eq!(
            f.call(1, "lst", "GetList", &[json!(1)]).unwrap(),
            json!("one")
        );
        let o = f.set_prop(1, "lst", "ListIndex", &json!(1)).unwrap();
        assert!(o.changed && o.click);
        assert_eq!(f.get_prop(1, "lst", "Value").unwrap(), json!("one"));
        assert!(f.set_prop(1, "lst", "ListIndex", &json!(5)).is_err());
        f.call(1, "lst", "RemoveItem", &[json!(1)]).unwrap();
        assert_eq!(f.get_prop(1, "lst", "ListIndex").unwrap(), json!(-1));
        f.set_prop(1, "lst", "List", &json!([["a", "1"], ["b", "2"]]))
            .unwrap();
        assert_eq!(
            f.call(1, "lst", "GetList", &[json!(1), json!(1)]).unwrap(),
            json!("2")
        );
        let o = f.set_prop(1, "tb", "Text", &json!("hi")).unwrap();
        assert!(!o.changed);
        assert!(f.get_prop(1, "nothing", "Value").is_err());
    }
}
