//! The colour palettes docked under the drawing window: the default
//! palette, a strip for each further palette shown (#1164), then the
//! document palette.
//!
//! Each row: a grip, the palette menu button, the eyedropper (it adds
//! colours sampled from the drawing; only the document palette can be
//! changed), the scroll arrow, then 20 px swatches in a one-pixel dark
//! frame, and at the right the other scroll arrow and the button that shows
//! every colour at once. A click sets the fill of the selection, a right
//! click its outline, Ctrl+click mixes a tenth of the colour into the fill,
//! a click and hold shows the colour's shades; the wheel scrolls.

use crate::app::App;
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::ui::rulers::Pixels;
use crate::ui::tips::RichTip;
use egui::{pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, Ui};
use tracedraw_core::{Color, Command, Fill};

/// Height of a palette row at the default swatch size: a separator line,
/// the swatches in their frame and a pixel of space on each side.
pub const ROW_H: f32 = 25.0;
/// The side of a swatch at first, and the distance between two (the
/// frame line between them).
const SWATCH: f32 = 20.0;
#[cfg(test)]
const STRIDE: f32 = 21.0;
/// What a row holds besides its swatches across: the line, the frame
/// and a pixel of space on each side.
const ROW_PAD: f32 = ROW_H - SWATCH;

/// The side of a swatch of the default palette: Options, Customization,
/// Color Palette; the palette menu's Swatch Size; or Ctrl+wheel over a
/// palette (20 px at first).
pub fn swatch_side(app: &App) -> f32 {
    app.settings.palette.swatch_size()
}

/// From one swatch to the next: the side and the frame line between.
pub fn stride(app: &App) -> f32 {
    swatch_side(app) + 1.0
}

/// The height of a palette row of one line of swatches.
pub fn row_h(app: &App) -> f32 {
    swatch_side(app) + ROW_PAD
}

/// What a strip follows: the default palette's settings for the default
/// palette (and the document palette row under it), a palette bar's own
/// for a further one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StripPrefs {
    pub dock: crate::ui::bars::Dock,
    pub rows: u8,
    pub swatch: u8,
    pub float_size: [f32; 2],
}

/// The settings `which` follows.
pub fn strip_prefs(app: &App, which: Row) -> StripPrefs {
    let p = &app.settings.palette;
    let main = StripPrefs {
        dock: app.settings.bars.dock(crate::ui::bars::Bar::Palette),
        rows: p.rows,
        swatch: p.swatch_size,
        float_size: p.float_size,
    };
    match which {
        Row::Main => main,
        // A row under the drawing, one line, the default palette's size.
        Row::Document => StripPrefs {
            dock: crate::ui::bars::Dock::Bottom,
            rows: 1,
            ..main
        },
        Row::Extra(k) => match app.settings.palette_bars.get(k) {
            Some(b) => StripPrefs {
                dock: b.dock,
                rows: b.rows,
                swatch: b.swatch_size,
                float_size: b.float_size,
            },
            None => main,
        },
    }
}

/// The side of a swatch of `which`.
pub fn swatch_side_of(app: &App, which: Row) -> f32 {
    let s = strip_prefs(app, which).swatch;
    f32::from(s.clamp(crate::settings::SWATCH_MIN, crate::settings::SWATCH_MAX))
}

/// From one swatch of `which` to the next.
pub fn stride_of(app: &App, which: Row) -> f32 {
    swatch_side_of(app, which) + 1.0
}

/// The most rows (or columns) a floating palette flows into.
pub const MAX_FLOAT_LINES: usize = 24;

/// How a floating palette lays its swatches in the room asked for
/// (`asked`, points, the title left out): a strip along the longer side,
/// as many lines across as whole swatches fit in the shorter one, never
/// shorter than the buttons and two swatches.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flow {
    /// Columns: the strip runs down.
    pub vertical: bool,
    /// Rows (columns when vertical) of swatches.
    pub lines: usize,
    /// The strip's length.
    pub len: f32,
}

/// See [`Flow`]; `side` is the swatch side.
pub fn floating_flow(asked: [f32; 2], side: f32) -> Flow {
    let ok = |v: f32| if v.is_finite() { v.max(1.0) } else { 1.0 };
    let (w, h) = (ok(asked[0]), ok(asked[1]));
    let st = side + 1.0;
    let vertical = h > w;
    let (along, across) = if vertical { (h, w) } else { (w, h) };
    let lines = (((across - side - ROW_PAD) / st).floor() + 1.0).clamp(1.0, MAX_FLOAT_LINES as f32)
        as usize;
    Flow {
        vertical,
        lines,
        len: along.max(min_len(side)),
    }
}

/// The shortest a strip is: its buttons and two swatches.
pub fn min_len(side: f32) -> f32 {
    LEAD + TAIL + 1.0 + 2.0 * (side + 1.0)
}

/// How thick a strip of `lines` rows of swatches `side` wide is.
pub fn thickness_for(side: f32, lines: usize) -> f32 {
    side + ROW_PAD + lines.saturating_sub(1) as f32 * (side + 1.0)
}

/// The size of a floating strip's swatches and buttons (its window less
/// the title and the border): [`floating_flow`] made whole.
pub fn floating_body(app: &App, which: Row) -> (Flow, egui::Vec2) {
    let side = swatch_side_of(app, which);
    let flow = floating_flow(strip_prefs(app, which).float_size, side);
    let thick = thickness_for(side, flow.lines);
    let size = if flow.vertical {
        vec2(thick, flow.len)
    } else {
        vec2(flow.len, thick)
    };
    (flow, size)
}

/// The room a floating strip asks for to show `lines` lines, keeping its
/// length and its direction.
pub fn float_size_for_lines(app: &App, which: Row, lines: usize) -> [f32; 2] {
    let side = swatch_side_of(app, which);
    let (flow, _) = floating_body(app, which);
    let thick = thickness_for(side, lines.max(1));
    if flow.vertical {
        [thick, flow.len.max(thick + 1.0)]
    } else {
        [flow.len.max(thick + 1.0), thick]
    }
}

/// Whether `which` shows the document palette (its row, or a bar of it).
pub fn shows_document(app: &App, which: Row) -> bool {
    match which {
        Row::Document => true,
        Row::Extra(k) => app.extra_palettes.get(k).is_some_and(|r| r.document),
        Row::Main => false,
    }
}

/// The user's own palette `which` shows (a bar of its own, not the
/// document palette), when it takes colours dropped on it.
pub fn user_palette_of(app: &App, which: Row) -> Option<usize> {
    let Row::Extra(k) = which else {
        return None;
    };
    let r = app.extra_palettes.get(k).filter(|r| !r.document)?;
    app.palette_takes_colors(r.index).then_some(r.index)
}

/// How many swatches `which` holds, the No Color well included.
pub fn swatch_count(app: &App, which: Row) -> usize {
    let n = if shows_document(app, which) {
        if app.has_document() {
            app.doc().palette.len()
        } else {
            0
        }
    } else {
        match which {
            Row::Extra(k) => app.extra_palettes.get(k).map_or(0, |r| r.colors.len()),
            _ => app.palette.len(),
        }
    };
    n + usize::from(app.settings.palette.show_no_color)
}
/// From a row's left edge to the frame of the first swatch.
const LEAD: f32 = 77.0;
/// Room for the scroll and expand buttons at the right.
const TAIL: f32 = 51.0;
const FRAME: Color32 = Color32::from_gray(90);
const NO_COLOR_LINE: Color32 = Color32::from_rgb(255, 64, 64);
/// Seconds a swatch is held before its shades pop up.
const HOLD: f64 = 0.5;
/// Seconds the right button is held on a swatch to open its menu when a
/// right click sets the outline.
const RIGHT_HOLD: f64 = 1.0;

/// Which palette a row shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Row {
    Main,
    Document,
    /// The `k`th palette shown after the first ([`App::extra_palettes`]).
    Extra(usize),
}

impl Row {
    /// A small number telling the rows apart in widget ids.
    fn key(self) -> u8 {
        match self {
            Row::Main => 0,
            Row::Document => 1,
            Row::Extra(k) => u8::try_from(k.saturating_add(2)).unwrap_or(u8::MAX),
        }
    }
}

/// A further palette shown in a strip of its own: which palette, its
/// colours (and as an RGB drawing shows them, worked out once when the
/// palettes change), and how far the strip is scrolled.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraRow {
    /// The palette in [`App::palettes`] (nothing for the document
    /// palette).
    pub index: usize,
    /// The bar shows the document palette.
    pub document: bool,
    pub colors: Vec<(String, Color)>,
    pub rgb: Vec<(String, Color)>,
    pub scroll: usize,
}

/// The default palette row.
pub fn palette_row(app: &mut App, ui: &mut Ui) {
    row(app, ui, Row::Main, false);
}

/// The document palette row.
pub fn document_palette_row(app: &mut App, ui: &mut Ui) {
    row(app, ui, Row::Document, false);
}

/// The strip of `which`, a row or (`vertical`) a column.
pub fn strip(app: &mut App, ui: &mut Ui, which: Row, vertical: bool) {
    row(app, ui, which, vertical);
}

/// The default palette as a column (docked on the left or the right).
pub fn palette_column(app: &mut App, ui: &mut Ui) {
    row(app, ui, Row::Main, true);
}

/// A strip's geometry: the row's design (left to right, the swatches
/// along it, 25 px across) laid along x for a row or along y for a
/// column.
#[derive(Clone, Copy)]
struct Axis {
    vertical: bool,
    origin: Pos2,
}

impl Axis {
    /// The rectangle `len` long and `thick` across, starting `along` the
    /// strip and `across` from its edge.
    fn rect(&self, along: f32, across: f32, len: f32, thick: f32) -> Rect {
        if self.vertical {
            Rect::from_min_size(
                pos2(self.origin.x + across, self.origin.y + along),
                vec2(thick, len),
            )
        } else {
            Rect::from_min_size(
                pos2(self.origin.x + along, self.origin.y + across),
                vec2(len, thick),
            )
        }
    }

    fn point(&self, along: f32, across: f32) -> Pos2 {
        self.rect(along, across, 0.0, 0.0).min
    }

    /// A vector `along` the strip and `across` it.
    fn vec(&self, along: f32, across: f32) -> egui::Vec2 {
        if self.vertical {
            vec2(across, along)
        } else {
            vec2(along, across)
        }
    }
}

fn colors_of(app: &App, which: Row) -> Vec<(String, Color)> {
    if which != Row::Document && shows_document(app, which) {
        return colors_of(app, Row::Document);
    }
    match which {
        // An RGB drawing gets the palette's colours as RGB, so what it
        // applies is RGB too; spot inks and registration stay (#483).
        // Worked out when the palette changes, not every frame (#620).
        Row::Main if rgb_drawing(app) => app.palette_rgb.clone(),
        Row::Main => app.palette.clone(),
        Row::Document if !app.has_document() => Vec::new(),
        Row::Document => app
            .doc()
            .palette
            .iter()
            .map(|c| (name_of(app, *c), *c))
            .collect(),
        Row::Extra(k) => match app.extra_palettes.get(k) {
            Some(r) if rgb_drawing(app) => r.rgb.clone(),
            Some(r) => r.colors.clone(),
            None => Vec::new(),
        },
    }
}

/// `palette` as an RGB drawing shows it: the default palette's colours
/// take the RGB default palette's values, defined directly, found by name
/// and colour so a palette edited in the Palette Editor keeps them (#620);
/// other colours their inks as RGB ([`in_rgb`]).
pub fn as_rgb(palette: &[(String, Color)]) -> Vec<(String, Color)> {
    palette
        .iter()
        .map(|(n, c)| match DEFAULTS.get(n.as_str()) {
            Some((cmyk, rgb)) if cmyk == c => (n.clone(), *rgb),
            _ => (n.clone(), in_rgb(*c)),
        })
        .collect()
}

/// The default palette's colours by name: their CMYK value and the RGB
/// one they show as in an RGB drawing.
static DEFAULTS: std::sync::LazyLock<std::collections::HashMap<String, (Color, Color)>> =
    std::sync::LazyLock::new(|| {
        crate::palette::default_cmyk()
            .into_iter()
            .zip(crate::palette::default_rgb())
            .map(|((n, cmyk), (_, rgb))| (n, (cmyk, rgb)))
            .collect()
    });

/// Whether the drawing's primary colour mode is RGB.
fn rgb_drawing(app: &App) -> bool {
    app.has_document()
        && app.doc().metadata.primary_color_mode == tracedraw_core::document::PrimaryColorMode::Rgb
}

/// `c` as an RGB colour, a spot ink or registration as it is. A CMYK
/// colour takes the plain RGB of its inks, not the press simulation: each
/// channel is 255 (1 - min(1, ink + K)), so the palette's black is 0 0 0,
/// its red 255 0 0, its tints the matching RGB tints and C20 M80 K20
/// 153 0 204 (#620).
fn in_rgb(c: Color) -> Color {
    match c {
        Color::Spot { .. } | Color::Registration => c,
        Color::Cmyk { c, m, y, k } => {
            let ink = |v: f32| 1.0 - (v.clamp(0.0, 1.0) + k.clamp(0.0, 1.0)).min(1.0);
            let byte = |v: f32| (ink(v) * 255.0).round() as u8;
            Color::rgb8(byte(c), byte(m), byte(y))
        }
        _ => c.convert_to("RGB"),
    }
}

/// The name of `c` in the default palette or the palettes shown; empty
/// when it has none.
pub fn name_of(app: &App, c: Color) -> String {
    app.palette
        .iter()
        .chain(app.palettes.iter().flat_map(|p| p.colors.iter()))
        .find(|(_, pc)| *pc == c)
        .map(|(n, _)| n.clone())
        .unwrap_or_default()
}

/// Tooltip of a swatch: the colour's name, then its values.
pub fn swatch_tip(name: &str, c: Color) -> String {
    let values = color_values(c);
    if name.is_empty() {
        values
    } else {
        format!("{name}\n{values}")
    }
}

/// A swatch's tooltip: the colour's name in bold, then its values one
/// per line (#1082).
pub fn swatch_tip_ui(ui: &mut Ui, name: &str, c: Color) {
    ui.spacing_mut().item_spacing.y = 1.0;
    let size = ui.style().text_styles[&egui::TextStyle::Body].size;
    if !name.is_empty() {
        ui.label(egui::RichText::new(name).font(crate::theme::bold(size)));
    }
    ui.label(color_lines(c));
}

/// A colour's values one per line ("C: 0", "M: 60", ...).
pub fn color_lines(c: Color) -> String {
    match c {
        Color::Cmyk { .. } | Color::Rgb { .. } => color_values(c)
            .split(' ')
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|p| p.join(" "))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => color_values(c),
    }
}

/// Values as the palette tooltips show them.
pub fn color_values(c: Color) -> String {
    let pct = |v: f32| (v * 100.0).round() as i32;
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as i32;
    match c {
        Color::Cmyk { c, m, y, k } => {
            format!("C: {} M: {} Y: {} K: {}", pct(c), pct(m), pct(y), pct(k))
        }
        Color::Rgb { r, g, b } => format!("R: {} G: {} B: {}", byte(r), byte(g), byte(b)),
        Color::Gray { v } => crate::i18n::trf("palette.gray_value", &[("v", &byte(v).to_string())]),
        other => crate::app::color_description(other),
    }
}

fn scroll_of(app: &mut App, which: Row) -> &mut usize {
    match which {
        Row::Main => &mut app.palette_scroll,
        Row::Document => &mut app.doc_palette_scroll,
        Row::Extra(k) if k < app.extra_palettes.len() => &mut app.extra_palettes[k].scroll,
        // A strip gone since: nothing scrolls it.
        Row::Extra(_) => &mut app.palette_scroll_spare,
    }
}

/// How thick the strip of `which` is: one row of swatches, and one more
/// swatch for each further row (#610).
pub fn thickness(app: &App, which: Row) -> f32 {
    thickness_for(swatch_side_of(app, which), lines_of(app, which))
}

/// Rows of swatches in the strip of `which` (the document palette row
/// has one): a floating strip's flow from its size, a docked one's
/// setting, "Fit" as many as show every colour in the length it had
/// last, at most [`crate::settings::MAX_ROWS`].
pub fn lines_of(app: &App, which: Row) -> usize {
    if which == Row::Document {
        return 1;
    }
    let prefs = strip_prefs(app, which);
    let side = swatch_side_of(app, which);
    if prefs.dock == crate::ui::bars::Dock::Floating {
        return floating_flow(prefs.float_size, side).lines;
    }
    crate::settings::rows_of(prefs.rows).unwrap_or_else(|| fit_lines(app, which))
}

/// The rows "Fit" gives `which`: every colour in the length it had
/// last.
pub fn fit_lines(app: &App, which: Row) -> usize {
    let len = app.palette_lens.get(&which.key()).copied().unwrap_or(800.0);
    let cols = visible_slots(len, stride_of(app, which));
    swatch_count(app, which)
        .div_ceil(cols.max(1))
        .clamp(1, usize::from(crate::settings::MAX_ROWS))
}

/// How many swatches `stride` apart fit in a row of this width (the No
/// Color well counts as one).
pub fn visible_slots(width: f32, stride: f32) -> usize {
    ((width - LEAD - TAIL - 1.0) / stride.max(1.0))
        .floor()
        .max(1.0) as usize
}

fn row(app: &mut App, ui: &mut Ui, which: Row, vertical: bool) {
    let lines = lines_of(app, which);
    let thick = thickness(app, which);
    let sw = swatch_side_of(app, which);
    let st = stride_of(app, which);
    let is_doc = shows_document(app, which);
    let size = if vertical {
        vec2(thick, ui.available_height())
    } else {
        vec2(ui.available_width(), thick)
    };
    let (rect, bg) = ui.allocate_exact_size(size, Sense::click());
    if which == Row::Document {
        app.doc_palette_rect = Some(rect);
    }
    // A palette of the user's takes colours dropped on its bar.
    let own = user_palette_of(app, which);
    if let Some(i) = own {
        app.palette_drop_rects.retain(|(k, _)| *k != i);
        app.palette_drop_rects.push((i, rect));
    }
    let hint = if is_doc {
        Some("palette.document_hint")
    } else {
        own.map(|_| "palette.user_empty_hint")
    };
    let painter = ui.painter_at(rect);
    let ax = Axis {
        vertical,
        origin: pos2(rect.left().round(), rect.top().round()),
    };
    let len = if vertical {
        rect.height()
    } else {
        rect.width()
    }
    .round();
    // The length kept for "Fit", which counts the swatches a line holds.
    if app.palette_lens.get(&which.key()) != Some(&len) {
        app.palette_lens.insert(which.key(), len);
        if crate::settings::rows_of(strip_prefs(app, which).rows).is_none() {
            ui.ctx().request_repaint();
        }
    }
    let mut px = Pixels::default();
    px.rect(ax.rect(0.0, 0.0, len, 1.0), crate::theme::Tokens::border());
    // Swatch frames start 2 px from the strip's edge.
    const EDGE: f32 = 2.0;
    // Grip: dots in a zigzag; dragging it moves the palette. Locked bars
    // show no grip, and a floating palette moves by its title.
    let floating = strip_prefs(app, which).dock == crate::ui::bars::Dock::Floating;
    if !app.settings.bars.locked && !floating {
        for i in 0..5 {
            let c = EDGE + 1.0 + 4.0 * i as f32;
            let a = ax.point(3.0, c);
            px.dot(a.x, a.y, crate::theme::Tokens::text_dim());
            let b = ax.point(7.0, c);
            px.dot(b.x, b.y, crate::theme::Tokens::text_dim());
            let m = ax.point(5.0, c + 2.0);
            px.dot(m.x, m.y, crate::theme::Tokens::text_dim());
        }
    }
    px.paint(&painter);
    let grip_rect = ax.rect(0.0, 0.0, 11.0, thick);
    match which {
        // A palette bar of its own moves by itself.
        Row::Extra(k) if !floating => {
            crate::ui::palette_bars::grip_zone(app, ui, k, grip_rect);
        }
        Row::Extra(_) => {}
        // The document palette row moves by itself too (as a bar of its
        // own), never the default palette above it.
        Row::Document if !floating => {
            crate::ui::palette_bars::document_grip_zone(app, ui, grip_rect);
        }
        Row::Main if !floating => {
            crate::ui::bars::grip_zone(
                app,
                ui,
                crate::ui::bars::Bar::Palette,
                grip_rect,
                which.key(),
            );
        }
        _ => {}
    }

    let colors = colors_of(app, which);
    let no_color = app.settings.palette.show_no_color;
    let count = colors.len() + usize::from(no_color);
    // Several rows fill line by line: each line holds the next run of the
    // palette, and the strip scrolls a swatch at a time (#610).
    let cols = visible_slots(len, st);
    let slots = cols * lines;
    let max_scroll = count.saturating_sub(slots);
    let scroll = (*scroll_of(app, which)).min(max_scroll);
    *scroll_of(app, which) = scroll;

    // The palette menu button.
    let menu_rect = ax.rect(13.0, EDGE, 15.0, 22.0);
    let menu_resp = ui.interact(
        menu_rect,
        ui.id().with(("palette_menu", which.key())),
        Sense::click(),
    );
    let c = menu_rect.center();
    painter.add(egui::epaint::PathShape::convex_polygon(
        vec![
            c + ax.vec(-2.0, -4.5),
            c + ax.vec(3.0, 0.5),
            c + ax.vec(-2.0, 5.5),
        ],
        crate::theme::Tokens::text_dim(),
        Stroke::NONE,
    ));
    let menu_resp = menu_resp.rich_tip(tr("palette.menu_tip"));
    egui::Popup::menu(&menu_resp)
        .id(egui::Id::new(("palette_menu_popup", which.key())))
        .style(crate::ui::menus::menu_popup_style)
        .show(|ui| crate::ui::menus::body(ui, |ui| menu(app, ui, which)));
    // A right click on the row outside the swatches opens the same menu.
    egui::Popup::context_menu(&bg)
        .id(egui::Id::new(("palette_context_popup", which.key())))
        .style(crate::ui::menus::menu_popup_style)
        .show(|ui| crate::ui::menus::body(ui, |ui| menu(app, ui, which)));

    // The eyedropper: only the document palette takes new colours.
    let drop_rect = ax.rect(34.0, EDGE + 2.0, 18.0, 18.0);
    let can_sample = is_doc && app.drawing_shown();
    let drop_resp = ui.interact(
        drop_rect,
        ui.id().with(("palette_dropper", which.key())),
        if can_sample {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let active = is_doc && app.pending_palette_sample;
    if active || (can_sample && drop_resp.hovered()) {
        painter.rect(
            drop_rect.expand(1.0),
            0.0,
            if active {
                Tokens::tool_active()
            } else {
                Tokens::tool_hover()
            },
            Stroke::new(1.0, Color32::from_rgb(0x99, 0xD1, 0xFF)),
            egui::StrokeKind::Inside,
        );
    }
    crate::ui::icons::draw(
        &painter,
        drop_rect.shrink(1.0),
        crate::tools::Tool::ColorEyedropper,
        if can_sample {
            Tokens::text()
        } else {
            crate::theme::Tokens::icon_off()
        },
    );
    if can_sample {
        let drop_resp = drop_resp.rich_tip(tr("palette.eyedropper_tip"));
        if drop_resp.clicked() {
            app.pending_palette_sample = !app.pending_palette_sample;
            if app.pending_palette_sample {
                app.status = tr("status.palette_sample");
            }
        }
    }

    // Scroll arrows.
    let left = ax.rect(58.0, EDGE, 16.0, 22.0);
    if chevron(app, ui, left, -1, scroll > 0, which, vertical) {
        *scroll_of(app, which) = scroll.saturating_sub(1);
    }
    let right = ax.rect(len - 49.0, EDGE, 16.0, 22.0);
    if chevron(app, ui, right, 1, scroll < max_scroll, which, vertical) {
        *scroll_of(app, which) = (scroll + 1).min(max_scroll);
    }
    let expand = ax.rect(len - 29.0, EDGE, 18.0, 22.0);
    let expand_resp = ui
        .interact(
            expand,
            ui.id().with(("palette_expand", which.key())),
            Sense::click(),
        )
        .rich_tip(tr("palette.expand_tip"));
    double_chevron(
        &painter,
        expand.center(),
        if count > 0 {
            crate::theme::Tokens::text_dim()
        } else {
            crate::theme::Tokens::icon_off()
        },
        vertical,
    );
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new(("palette_expand_rect", which.key())), expand));
    if expand_resp.clicked() && count > 0 {
        app.palette_expanded = if app.palette_expanded == Some(which) {
            None
        } else {
            Some(which)
        };
    }

    // The swatches.
    let strip = ax.point(LEAD, 0.0);
    ui.ctx().data_mut(|d| {
        d.insert_temp(egui::Id::new(("palette_strip", which.key())), strip);
        d.insert_temp(
            egui::Id::new(("palette_strip_vertical", which.key())),
            vertical,
        );
    });
    let shown = slots.min(count.saturating_sub(scroll));
    if shown == 0 {
        if let Some(hint) = hint.filter(|_| !vertical) {
            painter.text(
                ax.point(LEAD + 8.0, EDGE + 11.0),
                Align2::LEFT_CENTER,
                tr(hint),
                FontId::proportional(11.0),
                Tokens::text_dim(),
            );
        }
        return;
    }
    let mut px = Pixels::default();
    // One frame per line, as long as the swatches it holds.
    for line in 0..shown.div_ceil(cols) {
        let n = cols.min(shown - line * cols);
        px.rect(
            ax.rect(LEAD, EDGE + line as f32 * st, n as f32 * st + 1.0, st + 1.0),
            FRAME,
        );
    }
    px.paint(&painter);
    // Ctrl+wheel over the strip sizes the swatches (every palette row
    // follows); the plain wheel scrolls it.
    if let Some(steps) = crate::ui::bars::ctrl_wheel_steps(
        ui,
        rect,
        ui.id().with(("palette_size_wheel", which.key())),
    ) {
        let now = strip_prefs(app, which).swatch;
        crate::ui::palette_bars::set_swatch(
            app,
            which,
            crate::settings::swatch_resized(now, steps),
        );
    }
    // The plain wheel: a swatch a notch, on the desktop and in the
    // browser alike (#1406).
    let steps = if ui.rect_contains_pointer(rect) {
        crate::ui::field::wheel_steps_with(
            ui,
            ui.id().with(("palette_scroll_wheel", which.key())),
            |m| !(m.ctrl || m.command),
            true,
        )
        .unwrap_or(0)
    } else {
        0
    };
    let by = steps.unsigned_abs() as usize;
    if steps < 0 {
        *scroll_of(app, which) = (scroll + by).min(max_scroll);
    } else if steps > 0 {
        *scroll_of(app, which) = scroll.saturating_sub(by);
    }
    // A finger dragged along a strip longer than its room scrolls it with
    // the finger, a swatch for every swatch's width; one dragged off the
    // strip drags a colour as before (#1997). Known before the swatches
    // are drawn, so the one under the finger drags and fills nothing.
    if ui.input(|i| i.has_touch_screen()) {
        let key = egui::Id::new(("palette_finger", which.key()));
        let before: Option<crate::ui::overflow::Grab> = ui.ctx().data(|d| d.get_temp(key));
        let mut grab = before;
        if max_scroll > 0 {
            let room = ax.rect(LEAD, 0.0, (len - LEAD - TAIL).max(0.0), thick);
            let was = scroll as f32 * st;
            if let Some(pos) =
                crate::ui::overflow::finger_slide(ui, key, &mut grab, room, !vertical, was)
            {
                let to = (pos / st).round().max(0.0) as usize;
                *scroll_of(app, which) = to.min(max_scroll);
                app.palette_drag = None;
            }
        } else {
            grab = None;
        }
        if grab != before {
            ui.ctx().data_mut(|d| match grab {
                Some(g) => {
                    d.insert_temp(key, g);
                }
                None => {
                    d.remove::<crate::ui::overflow::Grab>(key);
                }
            });
        }
    }
    for slot in 0..shown {
        let index = scroll + slot;
        let cell = ax.rect(
            LEAD + 1.0 + (slot % cols) as f32 * st,
            EDGE + 1.0 + (slot / cols) as f32 * st,
            sw,
            sw,
        );
        let color_index = if no_color {
            index.checked_sub(1)
        } else {
            Some(index)
        };
        let entry = color_index.and_then(|i| colors.get(i));
        swatch(app, ui, &painter, cell, entry, which, index, color_index);
    }
    if let Some(hint) = hint.filter(|_| shown < slots && colors.is_empty() && !vertical) {
        painter.text(
            ax.point(LEAD + shown as f32 * st + 9.0, EDGE + 11.0),
            Align2::LEFT_CENTER,
            tr(hint),
            FontId::proportional(11.0),
            Tokens::text_dim(),
        );
    }
}

/// One swatch; `entry` is `None` for the No Color well.
#[allow(clippy::too_many_arguments)]
fn swatch(
    app: &mut App,
    ui: &mut Ui,
    painter: &egui::Painter,
    cell: Rect,
    entry: Option<&(String, Color)>,
    which: Row,
    index: usize,
    color_index: Option<usize>,
) {
    let id = ui.id().with(("swatch", which.key(), index));
    // Nothing to colour on the Welcome Screen or with no drawing open:
    // the swatches keep their colours and their tooltips but take no
    // clicks or drags (#482, #1082).
    let sense = if app.drawing_shown() {
        Sense::click_and_drag()
    } else {
        Sense::hover()
    };
    let resp = ui.interact(cell, id, sense);
    match entry {
        None => {
            painter.rect_filled(cell, 0.0, Color32::WHITE);
            painter.line_segment(
                [
                    cell.left_bottom() + vec2(2.5, -2.5),
                    cell.right_top() + vec2(-2.5, 2.5),
                ],
                Stroke::new(1.0, NO_COLOR_LINE),
            );
        }
        Some((_, c)) => {
            painter.rect_filled(cell, 0.0, crate::canvas::to_color32(*c));
        }
    }
    let current = shows_document(app, which)
        && color_index.is_some()
        && app.doc_palette_current == color_index;
    if resp.hovered() || resp.is_pointer_button_down_on() || current {
        // A white frame around the swatch under the pointer.
        let mut px = Pixels::default();
        let o = cell.expand(1.0);
        px.rect(
            Rect::from_min_size(o.min, vec2(o.width(), 1.0)),
            Color32::WHITE,
        );
        px.rect(
            Rect::from_min_size(pos2(o.left(), o.bottom() - 1.0), vec2(o.width(), 1.0)),
            Color32::WHITE,
        );
        px.rect(
            Rect::from_min_size(o.min, vec2(1.0, o.height())),
            Color32::WHITE,
        );
        px.rect(
            Rect::from_min_size(pos2(o.right() - 1.0, o.top()), vec2(1.0, o.height())),
            Color32::WHITE,
        );
        px.paint(painter);
    }
    let resp = match entry {
        None => resp.rich_tip(tr("palette.no_color_tip")),
        Some((name, c)) => resp.on_hover_ui(|ui| swatch_tip_ui(ui, name, *c)),
    };

    // Dragging the colour: inside an object it fills it, on its outline
    // it colours the outline, on the document palette it joins it. A
    // finger sliding the strip along drags no colour, and the swatch it
    // lifts from takes no click (#1997).
    let sliding = || crate::ui::overflow::finger_sliding(ui.ctx());
    if resp.drag_started_by(egui::PointerButton::Primary)
        && app.palette_shades.is_none()
        && !sliding()
    {
        app.palette_drag = Some(entry.map(|(_, c)| *c));
    }
    if app.palette_drag.is_some() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.palette_drag = None;
    }
    if resp.dragged() {
        if let (Some(color), Some(pos)) = (app.palette_drag, ui.ctx().pointer_latest_pos()) {
            drag_chip(app, ui.ctx(), color, pos);
        }
    }
    // A press that slipped a few pixels but was let go on the swatch it
    // began on is a click, not a drag: egui counts any move past its
    // click distance as a drag, and the colour went nowhere (#1336).
    // The same for the right button, which sets the outline.
    let on_cell = ui
        .ctx()
        .pointer_latest_pos()
        .is_some_and(|pos| cell.contains(pos));
    let slipped_click = resp.drag_stopped_by(egui::PointerButton::Primary) && on_cell && !sliding();
    let slipped_right = resp.drag_stopped_by(egui::PointerButton::Secondary) && on_cell;
    if resp.drag_stopped() {
        let dragged = app.palette_drag.take();
        if !slipped_click && !slipped_right {
            if let (Some(color), Some(pos)) = (dragged, ui.ctx().pointer_latest_pos()) {
                app.drop_color(color, pos);
            }
        }
    }

    // A finger has no right button: held on the swatch and let go, it
    // sets the outline; held for a second, the shades (#1611, #1707).
    let touch_outline = touch_hold(app, ui, &resp, cell, entry, id);

    // Click and hold: the colour's shades (a finger's are above).
    if let Some((_, c)) = entry {
        if resp.is_pointer_button_down_on()
            && ui.input(|i| i.pointer.primary_down() && !i.any_touches())
            && app.palette_shades.is_none()
        {
            let held = ui.input(|i| {
                i.pointer
                    .press_start_time()
                    .map(|t| i.time - t)
                    .unwrap_or(0.0)
            });
            if held >= HOLD && !resp.dragged() {
                app.palette_shades = Some((*c, cell.center_top()));
                ui.ctx()
                    .data_mut(|d| d.insert_temp(shades_fired_id(), true));
            } else {
                ui.ctx().request_repaint();
            }
        }
    }
    let fired = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(shades_fired_id()))
        .unwrap_or(false);
    if fired && (resp.clicked() || resp.drag_stopped()) {
        // The release that ends a hold does not apply the colour.
        ui.ctx().data_mut(|d| d.remove::<bool>(shades_fired_id()));
        return;
    }
    // Only that release: once the button is up (the hold ended over the
    // shades, or held past a click's time), the mark goes, or it took the
    // next click on any swatch and the colour was not applied (#1336).
    if fired && ui.input(|i| !i.pointer.any_down() && !i.pointer.any_released()) {
        ui.ctx().data_mut(|d| d.remove::<bool>(shades_fired_id()));
    }

    if (resp.clicked() || slipped_click) && !touch_outline {
        if shows_document(app, which) {
            app.doc_palette_current = color_index;
        }
        match entry {
            None => app.apply_fill(Fill::None),
            Some((_, c)) => {
                if ui.input(|i| i.modifiers.ctrl) {
                    app.mix_into_fill(*c);
                } else {
                    app.palette_fill(*c);
                }
            }
        }
    }
    let menu_id = egui::Id::new(("palette_swatch_menu", which.key(), index));
    if app.settings.palette.right_click_outline {
        // A right click sets the outline; held for a second it opens the
        // palette menu instead (the release then does nothing).
        if resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.secondary_down()) {
            let held = ui.input(|i| {
                i.pointer
                    .press_start_time()
                    .map(|t| i.time - t)
                    .unwrap_or(0.0)
            });
            let open = egui::Popup::is_id_open(ui.ctx(), menu_id);
            if held >= RIGHT_HOLD && !open {
                ui.ctx().data_mut(|d| d.insert_temp(menu_id, true));
            } else if !open {
                ui.ctx().request_repaint();
            }
        }
        let hold_open = ui
            .ctx()
            .data_mut(|d| d.remove_temp::<bool>(menu_id))
            .unwrap_or(false);
        // Not a right click that makes a copy during a drag on the canvas,
        // nor egui's long press of a finger (the hold above sets it).
        if (resp.secondary_clicked() || slipped_right)
            && !resp.long_touched()
            && !hold_open
            && !app.transforming()
        {
            app.apply_outline_color(entry.map(|(_, c)| *c));
        }
        egui::Popup::menu(&resp)
            .id(menu_id)
            .open_memory(hold_open.then_some(egui::SetOpenCommand::Bool(true)))
            .at_pointer_fixed()
            .style(crate::ui::menus::menu_popup_style)
            .show(|ui| crate::ui::menus::body(ui, |ui| menu(app, ui, which)));
    } else {
        egui::Popup::context_menu(&resp)
            .id(menu_id)
            .style(crate::ui::menus::menu_popup_style)
            .show(|ui| crate::ui::menus::body(ui, |ui| menu(app, ui, which)));
    }
}

/// A finger on a swatch (a touch screen, where a press comes with a
/// touch): let go after [`HOLD`] it sets the outline, as a right click
/// does, when Options say a right click sets it; held [`RIGHT_HOLD`] it
/// shows the shades, and the release then does nothing. A tap stays a
/// click (the fill), a slide a drag. The press is followed by hand, as
/// egui hands a finger held past its click time to its own long press
/// (the right click, and the menu when a right click opens it).
/// True on the release that set the outline: that release fills nothing.
fn touch_hold(
    app: &mut App,
    ui: &Ui,
    resp: &egui::Response,
    cell: Rect,
    entry: Option<&(String, Color)>,
    id: egui::Id,
) -> bool {
    let key = id.with("touch_hold");
    let (now, touched, down, start, origin, latest) = ui.input(|i| {
        (
            i.time,
            i.any_touches(),
            i.pointer.primary_down(),
            i.pointer.press_start_time(),
            i.pointer.press_origin(),
            i.pointer.latest_pos(),
        )
    });
    let mut since = ui.ctx().data(|d| d.get_temp::<f64>(key));
    if since.is_none()
        && touched
        && down
        && resp.is_pointer_button_down_on()
        && origin.is_some_and(|o| cell.contains(o))
    {
        since = start;
        if let Some(t) = since {
            ui.ctx().data_mut(|d| d.insert_temp(key, t));
        }
    }
    let Some(since) = since else {
        return false;
    };
    let slid = match (origin, latest) {
        (Some(o), Some(p)) => o.distance(p) > ui.ctx().options(|o| o.input_options.max_click_dist),
        _ => false,
    };
    if slid || resp.dragged() || app.palette_drag.is_some() {
        ui.ctx().data_mut(|d| d.remove::<f64>(key));
        return false;
    }
    let held = now - since;
    if down {
        if held >= RIGHT_HOLD {
            ui.ctx().data_mut(|d| d.remove::<f64>(key));
            if let Some((_, c)) = entry {
                if app.palette_shades.is_none() {
                    app.palette_shades = Some((*c, cell.center_top()));
                }
            }
        } else {
            ui.ctx().request_repaint();
        }
        return false;
    }
    ui.ctx().data_mut(|d| d.remove::<f64>(key));
    if held < HOLD || !app.settings.palette.right_click_outline || app.transforming() {
        return false;
    }
    app.apply_outline_color(entry.map(|(_, c)| *c));
    true
}

/// The colour under the pointer while it is dragged: a filled chip over
/// a fill, a ring over an outline, and a pointer saying whether letting
/// go changes anything.
fn drag_chip(app: &App, ctx: &egui::Context, color: Option<Color>, pos: Pos2) {
    use crate::color_drop::ColorTarget;
    let target = app.color_target(pos);
    ctx.set_cursor_icon(if target == ColorTarget::Nothing {
        egui::CursorIcon::NoDrop
    } else {
        egui::CursorIcon::Copy
    });
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        egui::Id::new("palette_drag_chip"),
    ));
    let r = Rect::from_min_size(pos + vec2(12.0, 12.0), vec2(16.0, 16.0));
    let fill = color.map(crate::canvas::to_color32);
    painter.rect_filled(r.expand(1.0), 0.0, Color32::WHITE);
    match (target, fill) {
        (ColorTarget::Outline(_), Some(c)) => {
            painter.rect_stroke(
                r.shrink(2.0),
                0.0,
                Stroke::new(4.0, c),
                egui::StrokeKind::Inside,
            );
        }
        (_, Some(c)) => {
            painter.rect_filled(r, 0.0, c);
        }
        (_, None) => {
            painter.line_segment(
                [r.left_bottom(), r.right_top()],
                Stroke::new(1.5, NO_COLOR_LINE),
            );
        }
    }
    painter.rect_stroke(
        r,
        0.0,
        Stroke::new(1.0, Color32::from_gray(0x5A)),
        egui::StrokeKind::Outside,
    );
}

fn shades_fired_id() -> egui::Id {
    egui::Id::new("palette_shades_fired")
}

/// A scroll chevron; returns true when clicked while enabled. Holding it
/// keeps scrolling.
#[allow(clippy::too_many_arguments)]
fn chevron(
    app: &App,
    ui: &mut Ui,
    r: Rect,
    dir: i32,
    enabled: bool,
    which: Row,
    vertical: bool,
) -> bool {
    let id = ui.id().with(("palette_chevron", which.key(), dir));
    let key = if dir < 0 {
        "palette.scroll_back"
    } else {
        "palette.scroll_forward"
    };
    let resp = ui
        .interact(r, id, Sense::click())
        .rich_tip(crate::ui::tips::keyed(key));
    let color = if enabled {
        crate::theme::Tokens::text_dim()
    } else {
        crate::theme::Tokens::icon_off()
    };
    let c = r.center();
    let d = dir as f32;
    let at = |along: f32, across: f32| {
        if vertical {
            pos2(c.x + across, c.y + along)
        } else {
            pos2(c.x + along, c.y + across)
        }
    };
    let pts = [at(-2.5 * d, -5.0), at(2.5 * d, 0.0), at(-2.5 * d, 5.0)];
    ui.painter().add(egui::epaint::PathShape::line(
        pts.to_vec(),
        Stroke::new(2.0, color),
    ));
    let _ = app;
    if !enabled {
        return false;
    }
    if resp.clicked() {
        return true;
    }
    // Repeat while held.
    if resp.is_pointer_button_down_on() {
        let held = ui.input(|i| {
            i.pointer
                .press_start_time()
                .map(|t| i.time - t)
                .unwrap_or(0.0)
        });
        ui.ctx().request_repaint();
        if held > 0.4 {
            let step = ui.input(|i| {
                (i.time * 12.0).floor() != ((i.time - i.unstable_dt as f64) * 12.0).floor()
            });
            return step;
        }
    }
    false
}

fn double_chevron(painter: &egui::Painter, c: Pos2, color: Color32, vertical: bool) {
    let at = |along: f32, across: f32| {
        if vertical {
            pos2(c.x + across, c.y + along)
        } else {
            pos2(c.x + along, c.y + across)
        }
    };
    for d in [-2.5f32, 2.5] {
        let pts = vec![at(d - 2.0, -3.5), at(d + 1.5, 0.0), at(d - 2.0, 3.5)];
        painter.add(egui::epaint::PathShape::line(pts, Stroke::new(1.3, color)));
    }
}

/// The palette menu (the arrow button, or a right click on the row).
pub fn menu(app: &mut App, ui: &mut Ui, which: Row) {
    use crate::ui::menus::{check, item, sep};
    ui.set_min_width(220.0);
    let document = shows_document(app, which);
    if which == Row::Main || !document {
        if which == Row::Main && item(ui, "menu.window.palette_open", "", true) {
            app.open_palette_file();
        }
        if item(ui, "menu.window.palette_editor", "", true) {
            app.dialog = crate::ui::dialogs::Dialog::PaletteEditor(Default::default());
        }
        if item(ui, "menu.window.palette_manager", "", true) {
            app.show_dockers = true;
            app.docker_tab = crate::app::DockerTab::Palettes;
        }
        sep(ui);
    }
    if document {
        document_items(app, ui);
        sep(ui);
    }
    if which == Row::Main
        && check(
            ui,
            "palette.show_document",
            "",
            app.settings.palette.show_document,
        )
    {
        app.settings.palette.show_document = !app.settings.palette.show_document;
        app.settings.save();
    }
    if which != Row::Document {
        crate::ui::palette_bars::strip_items(app, ui, which);
    }
    crate::ui::palette_bars::open_bar_items(app, ui);
    if item(ui, "palette.new", "", true) {
        crate::ui::palette_bars::start_new_palette(app);
    }
    if let Some(i) = user_palette_of(app, which) {
        let sel = app.has_document() && !app.selection.is_empty();
        if item(ui, "palette.add_selection_to_palette", "", sel) {
            app.add_selection_colors_to_palette(i);
        }
        bitmap_colors_items(app, ui, Some(i));
    }
    // Sort Colors: a palette of the user's, or the document palette.
    let user = user_palette_of(app, which);
    let document = shows_document(app, which) && app.has_document();
    if user.is_some() || document {
        crate::ui::menus::sub(ui, "palette.sort", |ui| {
            for (by, key) in crate::palette_sort::SortBy::ALL {
                if item(ui, key, "", true) {
                    match user {
                        Some(i) => app.sort_user_palette(i, by),
                        None => app.sort_document_palette(by),
                    }
                }
            }
        });
    }
    if let Row::Extra(k) = which {
        sep(ui);
        if item(ui, "palette.rename", "", true) {
            crate::ui::palette_bars::start_rename(app, which);
        }
        if item(ui, "palette.close", "", k < app.settings.palette_bars.len()) {
            crate::ui::palette_bars::close_bar(app, k);
        }
    } else if which == Row::Main && item(ui, "palette.rename", "", true) {
        crate::ui::palette_bars::start_rename(app, which);
    }
}

/// How many colours Add Colors from Bitmap offers to take.
pub const BITMAP_COLOR_COUNTS: [u32; 4] = [8, 16, 32, 64];

/// Add Colors from Bitmap (#1840): a submenu of how many colours of the
/// selected picture join the user's palette `palette`, or the document
/// palette when None; greyed without a picture selected.
fn bitmap_colors_items(app: &mut App, ui: &mut Ui, palette: Option<usize>) {
    use crate::ui::menus::{item_label, sub_if};
    let picture = app.has_document() && app.selected_bitmap().is_some();
    sub_if(ui, "palette.add_from_bitmap", picture, |ui| {
        for n in BITMAP_COLOR_COUNTS {
            let label = crate::i18n::trf("palette.count", &[("n", &n.to_string())]);
            if item_label(ui, &label, "", picture) {
                app.add_bitmap_colors_to_palette(palette, n);
            }
        }
    });
}

/// The document palette's own items.
fn document_items(app: &mut App, ui: &mut Ui) {
    use crate::ui::menus::{check, item, sep, sub};
    let doc = app.has_document();
    if check(
        ui,
        "palette.auto_update",
        "",
        app.settings.palette.auto_update_document,
    ) {
        app.settings.palette.auto_update_document = !app.settings.palette.auto_update_document;
        app.settings.save();
    }
    sep(ui);
    if item(
        ui,
        "palette.add_from_selection",
        "",
        doc && !app.selection.is_empty(),
    ) {
        app.palette_from_selection();
    }
    if item(ui, "palette.add_from_document", "", doc) {
        app.add_document_colors_to_palette();
    }
    bitmap_colors_items(app, ui, None);
    let current = if doc {
        app.doc_palette_current
            .filter(|i| *i < app.doc().palette.len())
    } else {
        None
    };
    if item(ui, "palette.delete_color", "", doc && current.is_some()) {
        if let Some(i) = current {
            let mut colors = app.doc().palette.clone();
            colors.remove(i);
            app.run(Command::SetDocumentPalette { colors });
            app.doc_palette_current = None;
        }
    }
    sep(ui);
    sub(ui, "palette.palette", |ui| {
        if item(ui, "palette.reset", "", doc) {
            app.reset_document_palette();
        }
        if item(ui, "palette.rename", "", true) {
            crate::ui::palette_bars::start_rename(app, Row::Document);
        }
    });
}

/// Pop-ups drawn over the window: a colour's shades and the expanded
/// palette. Called once per frame after the rows.
pub fn popups(app: &mut App, ctx: &egui::Context) {
    let sw = swatch_side(app);
    let st = stride(app);
    if let Some((color, anchor)) = app.palette_shades {
        let id = egui::Id::new("palette_shades");
        let grid = shades(color);
        let side = 5.0 * st + 1.0;
        let screen = ctx.content_rect();
        // Above the swatch, kept on the screen (a palette docked at the
        // top or on a side has no room above some swatches).
        let mut pos = pos2(anchor.x - side / 2.0, anchor.y - side - 6.0);
        if pos.y < screen.top() {
            pos.y = anchor.y + sw + 6.0;
        }
        pos.x = pos.x.clamp(
            screen.left(),
            (screen.right() - side - 4.0).max(screen.left()),
        );
        let mut picked = None;
        let area = egui::Area::new(id)
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let (r, _) = ui.allocate_exact_size(vec2(side, side), Sense::hover());
                let painter = ui.painter();
                painter.rect_filled(r, 0.0, FRAME);
                for (i, c) in grid.iter().enumerate() {
                    let (col, row) = ((i % 5) as f32, (i / 5) as f32);
                    let cell = Rect::from_min_size(
                        r.min + vec2(1.0 + col * st, 1.0 + row * st),
                        vec2(sw, sw),
                    );
                    painter.rect_filled(cell, 0.0, crate::canvas::to_color32(*c));
                    let resp = ui
                        .interact(cell, id.with(i), Sense::click())
                        .rich_tip(color_values(*c));
                    if resp.hovered() {
                        painter.rect_stroke(
                            cell.expand(1.0),
                            0.0,
                            Stroke::new(1.0, Color32::WHITE),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if resp.clicked() {
                        picked = Some((*c, false));
                    }
                    if resp.secondary_clicked() {
                        picked = Some((*c, true));
                    }
                }
            });
        if let Some((c, outline)) = picked {
            app.palette_shades = None;
            if outline {
                app.apply_outline_color(Some(c));
            } else {
                app.palette_fill(c);
            }
        } else if clicked_outside(ctx, area.response.rect) {
            app.palette_shades = None;
        }
    }
    if let Some(which) = app.palette_expanded {
        let colors = colors_of(app, which);
        let id = egui::Id::new("palette_expanded");
        let screen = ctx.content_rect();
        // Above the row, from its first swatch.
        let strip = ctx
            .data(|d| d.get_temp::<Pos2>(egui::Id::new(("palette_strip", which.key()))))
            .unwrap_or(pos2(
                screen.left() + LEAD,
                screen.bottom() - 3.0 * row_h(app),
            ));
        // Clear of every row of the strip.
        let thick = thickness(app, which);
        let vertical = ctx
            .data(|d| d.get_temp::<bool>(egui::Id::new(("palette_strip_vertical", which.key()))))
            .unwrap_or(false);
        let (cols, size, pos) = if vertical {
            // Beside a column: towards the middle of the window.
            let cols = 12usize;
            let rows = colors.len().div_ceil(cols).max(1);
            let size = vec2(cols as f32 * st + 1.0, rows as f32 * st + 1.0);
            let x = if strip.x > screen.center().x {
                strip.x - size.x - 24.0
            } else {
                strip.x + thick + 8.0
            };
            let y = strip
                .y
                .min(screen.bottom() - size.y - 24.0)
                .max(screen.top());
            (cols, size, pos2(x, y))
        } else {
            let cols = (((screen.right() - strip.x - 24.0) / st).floor() as usize).clamp(1, 33);
            let rows = colors.len().div_ceil(cols).max(1);
            let size = vec2(cols as f32 * st + 1.0, rows as f32 * st + 1.0);
            // Above the row, or below it when it is docked at the top.
            let above = strip.y - size.y - 16.0;
            let y = if above < screen.top() {
                strip.y + thick + 8.0
            } else {
                above
            };
            (cols, size, pos2(strip.x - 7.0, y))
        };
        let mut picked = None;
        let area = egui::Area::new(id)
            .order(egui::Order::Foreground)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    let (r, _) = ui.allocate_exact_size(size, Sense::hover());
                    let painter = ui.painter();
                    painter.rect_filled(r, 0.0, FRAME);
                    for (i, (name, c)) in colors.iter().enumerate() {
                        let (col, row) = ((i % cols) as f32, (i / cols) as f32);
                        let cell = Rect::from_min_size(
                            r.min + vec2(1.0 + col * st, 1.0 + row * st),
                            vec2(sw, sw),
                        );
                        painter.rect_filled(cell, 0.0, crate::canvas::to_color32(*c));
                        let resp = ui
                            .interact(cell, id.with(i), Sense::click())
                            .rich_tip(swatch_tip(name, *c));
                        if resp.hovered() {
                            painter.rect_stroke(
                                cell.expand(1.0),
                                0.0,
                                Stroke::new(1.0, Color32::WHITE),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if resp.clicked() {
                            picked = Some((*c, false));
                        }
                        if resp.secondary_clicked() {
                            picked = Some((*c, true));
                        }
                    }
                });
            });
        if let Some((c, outline)) = picked {
            app.palette_expanded = None;
            if outline {
                app.apply_outline_color(Some(c));
            } else {
                app.palette_fill(c);
            }
        } else if clicked_outside(ctx, area.response.rect) && !on_expand_button(ctx, which) {
            app.palette_expanded = None;
        }
    }
    // The release that ends a click and hold is over.
    if ctx.input(|i| i.pointer.any_released()) {
        ctx.data_mut(|d| d.remove::<bool>(shades_fired_id()));
    }
    crate::ui::palette_bars::rename_window(app, ctx);
    crate::ui::palette_bars::new_palette_window(app, ctx);
}

/// The expand button toggles the pop-up itself.
fn on_expand_button(ctx: &egui::Context, which: Row) -> bool {
    let r = ctx.data(|d| d.get_temp::<Rect>(egui::Id::new(("palette_expand_rect", which.key()))));
    let p = ctx.input(|i| i.pointer.interact_pos());
    matches!((r, p), (Some(r), Some(p)) if r.contains(p))
}

fn clicked_outside(ctx: &egui::Context, r: Rect) -> bool {
    ctx.input(|i| {
        i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !r.contains(p))
    })
}

/// A 5 x 5 grid around a colour: lighter tints upwards, more black
/// downwards, less colourful to the left, more to the right; the colour
/// itself in the middle.
pub fn shades(c: Color) -> Vec<Color> {
    let mut out = Vec::with_capacity(25);
    for row in -2i32..=2 {
        for col in -2i32..=2 {
            out.push(shade(c, row, col));
        }
    }
    out
}

fn shade(c: Color, row: i32, col: i32) -> Color {
    match c {
        Color::Cmyk { c, m, y, k } => {
            let mut v = [c, m, y];
            // Chroma: spread the inks away from (or towards) their mean.
            let mean = (v[0] + v[1] + v[2]) / 3.0;
            let s = 1.0 + 0.25 * col as f32;
            for x in &mut v {
                *x = mean + (*x - mean) * s;
            }
            let mut k = k;
            if row < 0 {
                // Tints: less of every ink.
                let f = 1.0 + 0.3 * row as f32;
                for x in &mut v {
                    *x *= f;
                }
                k *= f;
            } else {
                k += 0.2 * row as f32;
            }
            Color::Cmyk {
                c: round_pct(v[0]),
                m: round_pct(v[1]),
                y: round_pct(v[2]),
                k: round_pct(k),
            }
        }
        other => {
            let (h, s, b) = other.to_hsb();
            let s = (s * (1.0 + 0.25 * col as f64)).clamp(0.0, 1.0);
            let b = if row < 0 {
                b + (1.0 - b) * 0.35 * (-row) as f64
            } else {
                b * (1.0 - 0.25 * row as f64)
            };
            let tint = if row < 0 {
                s * (1.0 + 0.3 * row as f64)
            } else {
                s
            };
            Color::from_hsb(h, tint.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
        }
    }
}

fn round_pct(v: f32) -> f32 {
    (v.clamp(0.0, 1.0) * 100.0).round() / 100.0
}

/// Mix `share` of colour `b` into colour `a`, in `a`'s model when both are
/// CMYK or both RGB, else by way of RGB.
pub fn mix(a: Color, b: Color, share: f32) -> Color {
    let l = |x: f32, y: f32| x + (y - x) * share;
    match (a, b) {
        (
            Color::Cmyk { c, m, y, k },
            Color::Cmyk {
                c: c2,
                m: m2,
                y: y2,
                k: k2,
            },
        ) => Color::Cmyk {
            c: l(c, c2),
            m: l(m, m2),
            y: l(y, y2),
            k: l(k, k2),
        },
        _ => {
            let [r, g, bb] = a.to_rgb_f32();
            let [r2, g2, b2] = b.to_rgb_f32();
            Color::Rgb {
                r: l(r, r2),
                g: l(g, g2),
                b: l(bb, b2),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A drawing with a picture of four colours (red, green, blue and
    /// white quarters), selected when `select`.
    fn app_with_picture(select: bool) -> App {
        let mut app = App::headless();
        app.show_welcome = false;
        let img = image::RgbaImage::from_fn(40, 40, |x, y| match (x < 20, y < 20) {
            (true, true) => image::Rgba([220, 20, 30, 255]),
            (false, true) => image::Rgba([20, 160, 40, 255]),
            (true, false) => image::Rgba([30, 40, 200, 255]),
            (false, false) => image::Rgba([255, 255, 255, 255]),
        });
        let id = app.place_image(img, None).expect("picture");
        if select {
            app.select(vec![id]);
        } else {
            app.select(Vec::new());
        }
        app
    }

    /// Click the row labelled `label` of the palette menu of `which`, if
    /// it is there and enabled. Says whether it was.
    fn click_label(app: &mut App, make: impl Fn() -> App, which: Row, label: &str) -> bool {
        use crate::ui::menus::replay;
        let ctx = crate::theme::ui_context();
        for target in 0..300 {
            let mut probe = make();
            replay::start(target);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| menu(&mut probe, ui, which));
            out.textures_delta.clear();
            let (hit, seen) = replay::finish();
            if hit.as_deref() == Some(label) {
                replay::start(target);
                let mut out = ctx.run_ui(egui::RawInput::default(), |ui| menu(app, ui, which));
                out.textures_delta.clear();
                replay::finish();
                return true;
            }
            if target >= seen {
                break;
            }
        }
        false
    }

    /// Add Colors from Bitmap (#1840): the palette menu takes the colours
    /// that stand for the selected picture into the document palette
    /// (one undo step) or into a palette of the user's; without a
    /// picture selected the rows are greyed.
    #[test]
    fn colours_come_from_the_selected_picture() {
        let sixteen = crate::i18n::trf("palette.count", &[("n", "16")]);
        let want = [
            Color::rgb8(220, 20, 30),
            Color::rgb8(20, 160, 40),
            Color::rgb8(30, 40, 200),
            Color::rgb8(255, 255, 255),
        ];
        // The document palette.
        let mut app = app_with_picture(true);
        let before = app.doc().palette.clone();
        assert!(click_label(
            &mut app,
            || app_with_picture(true),
            Row::Document,
            &sixteen
        ));
        let after = app.doc().palette.clone();
        assert!(after.starts_with(&before), "the colours there stay first");
        for c in want {
            assert!(after.contains(&c), "{c:?} in {after:?}");
        }
        assert_eq!(after.len(), before.len() + 4, "{after:?}");
        app.undo();
        assert_eq!(app.doc().palette, before, "one undo step");
        // Nothing selected: greyed.
        let mut app = app_with_picture(false);
        assert!(!click_label(
            &mut app,
            || app_with_picture(false),
            Row::Document,
            &sixteen
        ));
        // A palette of the user's, from its own bar's menu.
        let make = || {
            let mut app = app_with_picture(true);
            app.new_palette("Mine").expect("palette");
            app
        };
        let mut app = make();
        let i = app.palettes.len() - 1;
        let k = app
            .extra_palettes
            .iter()
            .position(|r| !r.document && r.index == i)
            .expect("its bar");
        assert!(click_label(&mut app, make, Row::Extra(k), &sixteen));
        let got: Vec<Color> = app.palettes[i].colors.iter().map(|(_, c)| *c).collect();
        assert_eq!(got.len(), 4, "{got:?}");
        for c in want {
            assert!(got.contains(&c), "{c:?} in {got:?}");
        }
    }

    #[test]
    fn tooltips_name_the_colour_and_its_values() {
        let c = Color::cmyk_pct(60.0, 0.0, 40.0, 20.0);
        assert_eq!(
            swatch_tip("Light Green", c),
            "Light Green\nC: 60 M: 0 Y: 40 K: 20"
        );
        assert_eq!(swatch_tip("", Color::rgb8(255, 0, 10)), "R: 255 G: 0 B: 10");
        assert_eq!(
            color_lines(Color::cmyk_pct(0.0, 60.0, 100.0, 0.0)),
            "C: 0\nM: 60\nY: 100\nK: 0"
        );
        assert_eq!(color_lines(Color::rgb8(255, 0, 10)), "R: 255\nG: 0\nB: 10");
    }

    #[test]
    fn the_default_palette_has_greys_process_colours_and_tints() {
        let app = App::headless();
        assert_eq!(app.palette.len(), 94);
        let names: Vec<&str> = app.palette.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(&names[..3], &["Black", "90% Black", "80% Black"]);
        assert_eq!(names[10], "White");
        assert_eq!(
            &names[11..17],
            &["Blue", "Cyan", "Green", "Yellow", "Red", "Magenta"]
        );
        let (_, peach) = app
            .palette
            .iter()
            .find(|(n, _)| n == "Peach")
            .expect("peach tint");
        assert_eq!(*peach, Color::cmyk_pct(0.0, 40.0, 60.0, 0.0));
        let dark = app
            .palette
            .iter()
            .find(|(n, _)| n == "Dark Green")
            .map(|(_, c)| *c);
        assert_eq!(dark, Some(Color::cmyk_pct(20.0, 0.0, 0.0, 80.0)));
    }

    #[test]
    fn shades_keep_the_colour_in_the_middle() {
        let c = Color::cmyk_pct(0.0, 60.0, 100.0, 0.0);
        let g = shades(c);
        assert_eq!(g.len(), 25);
        assert_eq!(g[12], c);
        // Lighter above, darker below.
        assert!(g[2].luminance() > c.luminance());
        assert!(g[22].luminance() < c.luminance());
    }

    #[test]
    fn mixing_moves_a_tenth_of_the_way() {
        let a = Color::cmyk_pct(0.0, 0.0, 0.0, 0.0);
        let b = Color::cmyk_pct(100.0, 0.0, 0.0, 50.0);
        assert_eq!(mix(a, b, 0.1), Color::cmyk_pct(10.0, 0.0, 0.0, 5.0));
    }

    /// The eyedropper of the document palette picking a colour it has
    /// already frames that swatch, as the last colour used; a new colour
    /// joins and takes the frame, and applying a palette colour moves it
    /// there (#1125).
    #[test]
    fn a_colour_picked_again_frames_its_swatch() {
        let mut app = App::headless();
        let red = Color::rgb8(255, 0, 0);
        let blue = Color::rgb8(0, 0, 255);
        app.add_to_document_palette(red);
        app.add_to_document_palette(blue);
        let n = app.doc().palette.len();
        let at = |app: &App, c: Color| app.doc().palette.iter().position(|x| *x == c);
        assert_eq!(app.doc_palette_current, at(&app, blue));
        app.add_to_document_palette(red);
        assert_eq!(app.doc().palette.len(), n, "no second red swatch");
        assert_eq!(app.doc_palette_current, at(&app, red));
        app.remember_color(blue);
        assert_eq!(app.doc_palette_current, at(&app, blue));
    }

    #[test]
    fn applying_a_colour_adds_it_to_the_document_palette_in_the_same_step() {
        use tracedraw_core::{geometry::Rect as R, ShapeKind};
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: R::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        let steps = app.engine.history_labels().0.len();
        let green = Color::cmyk_pct(100.0, 0.0, 100.0, 0.0);
        app.apply_fill(Fill::Solid(green));
        assert_eq!(app.doc().palette, vec![green]);
        assert_eq!(app.engine.history_labels().0.len(), steps + 1);
        // Applying it again does not add it twice; the outline adds black.
        app.apply_fill(Fill::Solid(green));
        let black = Color::cmyk_pct(0.0, 0.0, 0.0, 100.0);
        app.apply_outline_color(Some(black));
        assert_eq!(app.doc().palette, vec![green, black]);
        app.undo();
        assert_eq!(app.doc().palette, vec![green]);
        // Off in the options: nothing is added.
        app.settings.palette.auto_update_document = false;
        app.apply_fill(Fill::Solid(Color::cmyk_pct(0.0, 100.0, 0.0, 0.0)));
        assert_eq!(app.doc().palette, vec![green]);
    }

    /// On the Welcome Screen the palette takes no clicks: a click on a
    /// swatch fills the selection of the drawing behind only while the
    /// drawing shows (#482).
    #[test]
    fn the_palette_rests_on_the_welcome_screen() {
        use tracedraw_core::{geometry::Rect as R, ShapeKind};
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: R::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        let ctx = crate::theme::ui_context();
        // A second apart, so no two clicks make a double click.
        let clock = std::cell::Cell::new(0.0);
        let click = |app: &mut App, x: f32| {
            clock.set(clock.get() + 1.0);
            // Over the swatch first, then pressed and let go.
            for (i, pressed) in [(0, false), (1, true), (2, false), (3, false)] {
                let mut events = vec![egui::Event::PointerMoved(egui::pos2(x, 10.0))];
                if i == 1 || i == 2 {
                    events.push(egui::Event::PointerButton {
                        pos: egui::pos2(x, 10.0),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 100.0),
                    )),
                    events,
                    time: Some({
                        clock.set(clock.get() + 0.05);
                        clock.get()
                    }),
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ui| palette_row(app, ui));
                out.textures_delta.clear();
            }
        };
        let fill = |app: &App| app.doc().find_shape(id).expect("shape").fill.clone();
        // A swatch that fills while the drawing shows...
        let before = fill(&app);
        let x = (0..30)
            .map(|k| 30.0 + k as f32 * 21.0)
            .find(|x| {
                click(&mut app, *x);
                let changed = fill(&app) != before;
                if changed {
                    app.undo();
                }
                changed
            })
            .expect("a swatch");
        // ...does nothing on the Welcome Screen.
        app.show_welcome = true;
        click(&mut app, x);
        assert_eq!(fill(&app), before);
        app.show_welcome = false;
        click(&mut app, x);
        assert_ne!(fill(&app), before);
        // The swatches keep their full colours on the Welcome Screen
        // (#1082): the row paints the same colours as with the drawing.
        let fills = |app: &mut App| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 100.0),
                )),
                ..Default::default()
            };
            let out = ctx.run_ui(input, |ui| palette_row(app, ui));
            out.shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Rect(r) => Some(r.fill),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let shown = fills(&mut app);
        app.show_welcome = true;
        let welcome = fills(&mut app);
        assert!(shown.len() > 10, "{shown:?}");
        assert_eq!(welcome, shown);
    }

    /// Every colour at once, for a palette of three rows along the top:
    /// the window opens below all three rows, not over them (#610).
    #[test]
    fn every_colour_opens_clear_of_all_the_rows() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.settings.palette.rows = 3;
        let run = |app: &mut App| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 700.0),
                )),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                palette_row(app, ui);
                popups(app, ui.ctx());
            });
            out.textures_delta.clear();
        };
        run(&mut app);
        app.palette_expanded = Some(Row::Main);
        run(&mut app);
        run(&mut app);
        let area = ctx
            .memory(|m| m.area_rect(egui::Id::new("palette_expanded")))
            .expect("every colour shown");
        assert!(
            area.top() >= thickness(&app, Row::Main),
            "{area:?} over the rows"
        );
    }

    /// With two rows the default palette is twice as thick and fills line
    /// by line: Black sits beside the No Color well with 90% Black next to
    /// it, and the second line goes on where the first ends; scrolling
    /// moves one swatch (#610).
    #[test]
    fn the_default_palette_can_show_several_rows() {
        use tracedraw_core::{geometry::Rect as R, ShapeKind};
        let mut app = App::headless();
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: R::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let height = std::cell::Cell::new(0.0);
        let click = |app: &mut App, p: egui::Pos2| {
            clock.set(clock.get() + 1.0);
            for (i, pressed) in [(0, false), (1, true), (2, false), (3, false)] {
                let mut events = vec![egui::Event::PointerMoved(p)];
                if i == 1 || i == 2 {
                    events.push(egui::Event::PointerButton {
                        pos: p,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 100.0),
                    )),
                    events,
                    time: Some({
                        clock.set(clock.get() + 0.05);
                        clock.get()
                    }),
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ui| {
                    let top = ui.cursor().top();
                    palette_row(app, ui);
                    height.set(ui.cursor().top() - top);
                });
                out.textures_delta.clear();
            }
        };
        let fill = |app: &App| app.doc().find_shape(id).expect("shape").fill.clone();
        let before = fill(&app);
        // The first column, one swatch down.
        let x = LEAD + 1.0 + SWATCH / 2.0;
        let second_line = egui::pos2(x, 3.0 + STRIDE + SWATCH / 2.0);
        click(&mut app, second_line);
        assert_eq!(fill(&app), before, "one row: nothing there");
        assert_eq!(thickness(&app, Row::Main), ROW_H);
        app.settings.palette.rows = 2;
        assert_eq!(thickness(&app, Row::Main), ROW_H + STRIDE);
        assert_eq!(thickness(&app, Row::Document), ROW_H);
        click(&mut app, second_line);
        let solid = |app: &App| match fill(app) {
            Fill::Solid(c) => c,
            other => panic!("{other:?}"),
        };
        // The second line starts with the swatch after the first line's
        // last (the No Color well takes the first slot).
        let cols = visible_slots(800.0, STRIDE);
        assert_eq!(solid(&app), app.palette[cols - 1].1);
        assert!(height.get() >= ROW_H + STRIDE, "{}", height.get());
        click(&mut app, egui::pos2(x + STRIDE, 3.0 + SWATCH / 2.0));
        assert_eq!(solid(&app), Color::cmyk_pct(0.0, 0.0, 0.0, 100.0));
        click(&mut app, egui::pos2(x + 2.0 * STRIDE, 3.0 + SWATCH / 2.0));
        assert_eq!(solid(&app), Color::cmyk_pct(0.0, 0.0, 0.0, 90.0));
        // The scroll arrow moves one swatch: Black comes under the pointer
        // of the first swatch.
        click(&mut app, egui::pos2(800.0 - 49.0 + 8.0, 12.0));
        assert_eq!(app.palette_scroll, 1);
        click(&mut app, egui::pos2(x, 3.0 + SWATCH / 2.0));
        assert_eq!(solid(&app), Color::cmyk_pct(0.0, 0.0, 0.0, 100.0));
        // Up to eight rows; more is not kept.
        app.settings.palette.rows = 5;
        assert_eq!(thickness(&app, Row::Main), ROW_H + 4.0 * STRIDE);
        app.settings.palette.rows = 9;
        assert_eq!(thickness(&app, Row::Main), ROW_H + 7.0 * STRIDE);
    }

    /// The palette's own menu has Rows > 1 Row, 2 Rows, 3 Rows: picking
    /// one changes the strip's rows and keeps it in the preferences
    /// (#610).
    #[test]
    fn the_palette_menu_picks_the_rows() {
        let mut app = App::headless();
        let ctx = crate::theme::ui_context();
        App::setup_context(&ctx);
        let clock = std::cell::Cell::new(0.0);
        // One frame with `events`; the place of each text drawn.
        let frame = |app: &mut App, events: Vec<egui::Event>| -> Vec<(String, egui::Rect)> {
            clock.set(clock.get() + 0.1);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                events,
                time: Some(clock.get()),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                ui.add_space(400.0);
                palette_row(app, ui)
            });
            out.textures_delta.clear();
            fn walk(s: &egui::Shape, texts: &mut Vec<(String, egui::Rect)>) {
                match s {
                    egui::Shape::Text(t) => texts.push((
                        t.galley.text().to_string(),
                        t.galley.rect.translate(t.pos.to_vec2()),
                    )),
                    egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, texts)),
                    _ => {}
                }
            }
            let mut texts = Vec::new();
            out.shapes.iter().for_each(|c| walk(&c.shape, &mut texts));
            texts
        };
        let press = |p: egui::Pos2| {
            vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton {
                    pos: p,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        };
        let release = |p: egui::Pos2| {
            vec![egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }]
        };
        let texts = frame(&mut app, Vec::new());
        assert!(texts.iter().all(|(t, _)| t != &tr("palette.rows")));
        // The menu button at the start of the strip.
        let button = egui::pos2(13.0 + 7.5, 400.0 + 2.0 + 11.0);
        frame(&mut app, press(button));
        frame(&mut app, release(button));
        let mut texts = frame(&mut app, Vec::new());
        let at = |texts: &[(String, egui::Rect)], key: &str| {
            texts
                .iter()
                .find(|(t, _)| t == &tr(key))
                .map(|(_, r)| r.center())
        };
        let rows = at(&texts, "palette.rows").expect("Rows in the palette menu");
        // Hovering Rows opens its submenu.
        for _ in 0..4 {
            texts = frame(&mut app, vec![egui::Event::PointerMoved(rows)]);
        }
        let two = at(&texts, "palette.rows_2").expect("2 Rows in the submenu");
        assert!(at(&texts, "palette.rows_1").is_some());
        assert!(at(&texts, "palette.rows_3").is_some());
        assert_eq!(crate::settings::rows_of(app.settings.palette.rows), Some(1));
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(egui::pos2(two.x, rows.y))],
        );
        frame(&mut app, vec![egui::Event::PointerMoved(two)]);
        frame(&mut app, press(two));
        frame(&mut app, release(two));
        frame(&mut app, Vec::new());
        assert_eq!(crate::settings::rows_of(app.settings.palette.rows), Some(2));
        assert_eq!(thickness(&app, Row::Main), ROW_H + STRIDE);
    }

    /// An RGB drawing's palette holds the palette's colours as RGB (named
    /// as before); a CMYK drawing's keeps them CMYK (#483).
    #[test]
    fn an_rgb_drawing_gets_an_rgb_palette() {
        let mut app = App::headless();
        app.palette = vec![
            ("Cyan".into(), Color::cmyk_pct(100.0, 0.0, 0.0, 0.0)),
            ("Mine".into(), Color::rgb8(10, 20, 30)),
        ];
        app.palette_rgb = as_rgb(&app.palette);
        let cmyk = colors_of(&app, Row::Main);
        assert_eq!(cmyk, app.palette);
        let mut metadata = app.doc().metadata.clone();
        metadata.primary_color_mode = tracedraw_core::document::PrimaryColorMode::Rgb;
        app.run(tracedraw_core::Command::SetMetadata { metadata });
        let rgb = colors_of(&app, Row::Main);
        assert_eq!(rgb.len(), 2);
        assert!(rgb.iter().all(|(_, c)| c.model_name() == "RGB"), "{rgb:?}");
        assert_eq!(rgb[0].0, "Cyan");
        assert_eq!(rgb[0].1, Color::rgb8(0, 255, 255));
        assert_eq!(rgb[1].1, Color::rgb8(10, 20, 30));
    }

    /// The default palette in an RGB drawing holds clean RGB values: black
    /// 0 0 0, white 255 255 255, the primaries at full, the tints as RGB
    /// tints (#620).
    #[test]
    fn the_default_palette_of_an_rgb_drawing_is_clean_rgb() {
        let mut app = App::headless();
        let mut metadata = app.doc().metadata.clone();
        metadata.primary_color_mode = tracedraw_core::document::PrimaryColorMode::Rgb;
        app.run(tracedraw_core::Command::SetMetadata { metadata });
        let rgb = colors_of(&app, Row::Main);
        let of = |name: &str| {
            rgb.iter()
                .find(|(n, _)| n == name)
                .map(|(_, c)| *c)
                .expect(name)
        };
        assert_eq!(rgb.len(), 94);
        assert_eq!(of("Black"), Color::rgb8(0, 0, 0));
        assert_eq!(of("White"), Color::rgb8(255, 255, 255));
        assert_eq!(of("50% Black"), Color::rgb8(128, 128, 128));
        assert_eq!(of("Red"), Color::rgb8(255, 0, 0));
        assert_eq!(of("Green"), Color::rgb8(0, 255, 0));
        assert_eq!(of("Blue"), Color::rgb8(0, 0, 255));
        assert_eq!(of("Yellow"), Color::rgb8(255, 255, 0));
        assert_eq!(of("Magenta"), Color::rgb8(255, 0, 255));
        assert_eq!(of("Orange"), Color::rgb8(255, 102, 0));
        assert!(rgb.iter().all(|(_, c)| c.model_name() == "RGB"));
        // The colours with black in them are clean RGB
        // values too (#620).
        assert_eq!(of("Purple"), Color::rgb8(153, 0, 204));
        assert_eq!(of("Dusty Blue"), Color::rgb8(102, 102, 204));
        assert_eq!(of("Sky Blue"), Color::rgb8(0, 204, 255));
        assert_eq!(of("Dark Green"), Color::rgb8(0, 51, 51));
        assert_eq!(of("Powder Blue"), Color::rgb8(204, 204, 255));
        assert_eq!(of("Dark Brown"), Color::rgb8(102, 51, 51));
        assert_eq!(of("90% Black"), Color::rgb8(26, 26, 26));
        // Every channel is a multiple of 17 or a 10% grey.
        for (n, c) in &rgb {
            let [r, g, b] = c.to_rgb8();
            let grey = r == g && g == b;
            assert!(
                grey || [r, g, b].iter().all(|v| v % 17 == 0),
                "{n}: {r} {g} {b}"
            );
        }
    }

    /// A colour added to the default palette in the Palette Editor moves
    /// the others along, and they keep their RGB values; the Palette
    /// Manager's built-in RGB default palette is the same table (#620).
    #[test]
    fn edited_default_palettes_keep_their_rgb_values() {
        let mut palette = crate::palette::default_cmyk();
        palette.insert(0, ("Mine".into(), Color::cmyk_pct(10.0, 0.0, 0.0, 0.0)));
        palette.retain(|(n, _)| n != "Black");
        let rgb = as_rgb(&palette);
        let of = |name: &str| rgb.iter().find(|(n, _)| n == name).map(|(_, c)| *c);
        assert_eq!(of("Hot Pink"), Some(Color::rgb8(255, 51, 153)));
        assert_eq!(of("Dark Green"), Some(Color::rgb8(0, 51, 51)));
        assert_eq!(of("Mine"), Some(Color::rgb8(230, 255, 255)));
        // A default name with another colour is converted from its inks.
        let changed = vec![(
            "Hot Pink".to_string(),
            Color::cmyk_pct(0.0, 100.0, 0.0, 0.0),
        )];
        assert_eq!(as_rgb(&changed)[0].1, Color::rgb8(255, 0, 255));
        let builtin = crate::palette::builtin_palettes()
            .into_iter()
            .find(|p| p.key == "palette.default_rgb")
            .expect("the RGB default palette");
        assert_eq!(builtin.colors, crate::palette::default_rgb());
    }

    /// A CMYK colour outside the default palette takes 255 (1 - min(1,
    /// ink + K)) per channel in an RGB drawing (#620).
    #[test]
    fn inks_with_black_become_rgb_by_adding_the_black() {
        assert_eq!(
            in_rgb(Color::cmyk_pct(60.0, 40.0, 0.0, 40.0)),
            Color::rgb8(0, 51, 153)
        );
        assert_eq!(
            in_rgb(Color::cmyk_pct(0.0, 20.0, 20.0, 60.0)),
            Color::rgb8(102, 51, 51)
        );
        assert_eq!(
            in_rgb(Color::cmyk_pct(0.0, 0.0, 0.0, 100.0)),
            Color::rgb8(0, 0, 0)
        );
    }

    /// Ctrl+wheel over a palette row sizes its swatches (up larger, down
    /// smaller, 2 px a notch, 8 to 48 px) and scrolls nothing; the
    /// plain wheel scrolls it and sizes nothing.
    #[test]
    fn ctrl_wheel_sizes_the_swatches_and_the_plain_wheel_scrolls() {
        let mut app = App::headless();
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let frame = |app: &mut App, events: Vec<egui::Event>| {
            clock.set(clock.get() + 0.1);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 100.0),
                )),
                events,
                time: Some(clock.get()),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| palette_row(app, ui));
            out.textures_delta.clear();
        };
        let over = egui::pos2(LEAD + 60.0, 12.0);
        let wheel = |dy: f32, modifiers: egui::Modifiers| {
            vec![
                egui::Event::PointerMoved(over),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::vec2(0.0, dy),
                    phase: egui::TouchPhase::Move,
                    modifiers,
                },
            ]
        };
        frame(&mut app, Vec::new());
        frame(&mut app, vec![egui::Event::PointerMoved(over)]);
        assert_eq!(swatch_side(&app), 20.0);
        frame(&mut app, wheel(-1.0, egui::Modifiers::CTRL));
        assert_eq!(app.settings.palette.swatch_size, 18);
        assert_eq!(app.palette_scroll, 0, "Ctrl+wheel scrolls nothing");
        frame(&mut app, Vec::new());
        assert_eq!(thickness(&app, Row::Main), 23.0);
        frame(&mut app, wheel(1.0, egui::Modifiers::CTRL));
        frame(&mut app, wheel(1.0, egui::Modifiers::CTRL));
        assert_eq!(app.settings.palette.swatch_size, 22);
        for _ in 0..40 {
            frame(&mut app, wheel(1.0, egui::Modifiers::CTRL));
        }
        assert_eq!(
            app.settings.palette.swatch_size,
            crate::settings::SWATCH_MAX
        );
        frame(&mut app, Vec::new());
        assert_eq!(thickness(&app, Row::Main), 48.0 + 5.0);
        assert_eq!(app.palette_scroll, 0);
        // The plain wheel scrolls, a swatch a notch, and sizes nothing.
        frame(&mut app, wheel(-1.0, egui::Modifiers::NONE));
        assert_eq!(app.palette_scroll, 1);
        frame(&mut app, wheel(-1.0, egui::Modifiers::NONE));
        assert_eq!(app.palette_scroll, 2);
        frame(&mut app, wheel(1.0, egui::Modifiers::NONE));
        assert_eq!(app.palette_scroll, 1);
        assert_eq!(
            app.settings.palette.swatch_size,
            crate::settings::SWATCH_MAX
        );
    }

    /// In the browser a wheel notch comes as three lines (Firefox) or as
    /// 100 pixels (Chromium): the plain wheel scrolls a palette row one
    /// swatch a notch and Ctrl+wheel sizes its swatches one size a notch,
    /// as on the desktop (#1406). Parts of a notch from a touchpad add
    /// up to one step; each half of a browser notch scrolled a whole one.
    #[test]
    fn a_browser_wheel_notch_steps_a_palette_row_once() {
        let mut app = App::headless();
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let frame = |app: &mut App, events: Vec<egui::Event>| {
            clock.set(clock.get() + 0.1);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 100.0),
                )),
                events,
                time: Some(clock.get()),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| palette_row(app, ui));
            out.textures_delta.clear();
        };
        let over = egui::pos2(LEAD + 60.0, 12.0);
        let turn = |unit, dy: f32, modifiers| {
            vec![
                egui::Event::PointerMoved(over),
                egui::Event::MouseWheel {
                    unit,
                    delta: egui::vec2(0.0, dy),
                    phase: egui::TouchPhase::Move,
                    modifiers,
                },
            ]
        };
        use egui::MouseWheelUnit::{Line, Point};
        let none = egui::Modifiers::NONE;
        frame(&mut app, Vec::new());
        frame(&mut app, vec![egui::Event::PointerMoved(over)]);
        crate::ui::field::set_web_wheel(true);
        let mut scroll = 0i64;
        for (unit, dy, step) in [
            (Line, -3.0, 1),
            (Point, -100.0, 1),
            (Line, 3.0, -1),
            (Point, -50.0, 0),
            (Point, -50.0, 1),
            (Point, -40.0, 0),
            (Point, 40.0, 0),
        ] {
            frame(&mut app, turn(unit, dy, none));
            scroll += step;
            assert_eq!(app.palette_scroll as i64, scroll, "{unit:?} {dy}");
        }
        assert_eq!(swatch_side(&app), 20.0);
        frame(&mut app, turn(Line, 3.0, egui::Modifiers::CTRL));
        assert_eq!(app.settings.palette.swatch_size, 22);
        frame(&mut app, turn(Point, -100.0, egui::Modifiers::CTRL));
        assert_eq!(app.settings.palette.swatch_size, 20);
        assert_eq!(
            app.palette_scroll as i64, scroll,
            "Ctrl+wheel scrolls nothing"
        );
        crate::ui::field::set_web_wheel(false);
    }

    #[test]
    fn rows_fit_whole_swatches() {
        assert_eq!(visible_slots(LEAD + TAIL + 1.0 + 21.0 * 10.0, STRIDE), 10);
        assert_eq!(visible_slots(10.0, STRIDE), 1);
    }

    /// What a finger on a swatch does, with the events a touch screen
    /// sends (a touch and the pointer it moves): a tap fills, a hold of
    /// half a second let go sets the outline, a hold of a second shows
    /// the shades and the release then changes nothing (#1611, #1707).
    /// The mouse keeps its own: a held left button shows the shades.
    #[test]
    fn a_finger_held_on_a_swatch_sets_the_outline() {
        use tracedraw_core::{geometry::Rect as R, ShapeKind};
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let frame = |app: &mut App, events: Vec<egui::Event>, dt: f64| {
            clock.set(clock.get() + dt);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 100.0),
                )),
                events,
                time: Some(clock.get()),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| palette_row(app, ui));
            out.textures_delta.clear();
        };
        let at = egui::pos2(LEAD + 6.5 * STRIDE, 12.0);
        let touch = |phase: egui::TouchPhase| egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(7),
            phase,
            pos: at,
            force: None,
        };
        let press = |pressed: bool| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        // A finger down for `secs`, then lifted; frames every 50 ms.
        let finger = |app: &mut App, secs: f64| {
            frame(
                app,
                vec![
                    touch(egui::TouchPhase::Start),
                    egui::Event::PointerMoved(at),
                    press(true),
                ],
                0.05,
            );
            let mut held = 0.0;
            while held + 0.05 < secs {
                frame(app, Vec::new(), 0.05);
                held += 0.05;
            }
            frame(
                app,
                vec![
                    touch(egui::TouchPhase::End),
                    press(false),
                    egui::Event::PointerGone,
                ],
                0.05,
            );
            frame(app, Vec::new(), 0.05);
        };
        let mut app = App::headless();
        app.show_welcome = false;
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: R::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        let look = |app: &App| {
            let s = app.doc().find_shape(id).expect("rect");
            (s.fill.clone(), s.stroke.as_ref().map(|k| k.color))
        };
        frame(&mut app, Vec::new(), 0.05);
        let (fill0, outline0) = look(&app);
        // A tap fills.
        finger(&mut app, 0.1);
        let (fill, outline) = look(&app);
        assert_ne!(fill, fill0, "a tap fills");
        assert_eq!(outline, outline0, "a tap leaves the outline");
        let Fill::Solid(color) = fill.clone() else {
            panic!("a solid fill: {fill:?}");
        };
        app.undo();
        assert_eq!(look(&app), (fill0.clone(), outline0));
        // Held, let go: the outline takes the colour, the fill stays.
        finger(&mut app, 0.6);
        assert_eq!(
            look(&app),
            (fill0.clone(), Some(color)),
            "a hold sets the outline"
        );
        assert!(app.palette_shades.is_none());
        app.undo();
        assert_eq!(look(&app), (fill0.clone(), outline0));
        // Held for a second: the shades, and the release changes nothing.
        finger(&mut app, 1.2);
        assert_eq!(
            look(&app),
            (fill0.clone(), outline0),
            "the long hold shows the shades"
        );
        assert!(app.palette_shades.is_some());
        app.palette_shades = None;
        frame(&mut app, Vec::new(), 0.05);
        // The mouse: the left button held shows the shades at once.
        frame(
            &mut app,
            vec![egui::Event::PointerMoved(at), press(true)],
            0.05,
        );
        for _ in 0..12 {
            frame(&mut app, Vec::new(), 0.05);
        }
        assert!(
            app.palette_shades.is_some(),
            "a held button shows the shades"
        );
        frame(&mut app, vec![press(false)], 0.05);
        assert_eq!(look(&app), (fill0, outline0));
    }

    /// A finger dragged along a palette row longer than its room scrolls
    /// the row with the finger, a swatch for every swatch's width, and
    /// fills nothing when it lifts; a finger dragged off the row drags
    /// its colour as before; the mouse dragged along the row scrolls
    /// nothing (#1997).
    #[test]
    fn a_finger_dragged_along_the_palette_scrolls_it() {
        use tracedraw_core::{geometry::Rect as R, ShapeKind};
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let frame = |app: &mut App, events: Vec<egui::Event>| {
            clock.set(clock.get() + 0.02);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(300.0, 40.0),
                )),
                events,
                time: Some(clock.get()),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| palette_row(app, ui));
            out.textures_delta.clear();
        };

        let touch = |phase, pos| egui::Event::Touch {
            device_id: egui::TouchDeviceId(1),
            id: egui::TouchId(2),
            phase,
            pos,
            force: None,
        };
        let press = |pos, pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        // A finger (or, without `finger`, the mouse) down at `from`, moved
        // to `to` in `steps`; `lift` lets go there, else it stays down.
        let drag = |app: &mut App,
                    from: egui::Pos2,
                    to: egui::Pos2,
                    steps: usize,
                    finger: bool,
                    lift: bool| {
            let mut events = vec![egui::Event::PointerMoved(from), press(from, true)];
            if finger {
                events.insert(0, touch(egui::TouchPhase::Start, from));
            }
            frame(app, events);
            for k in 1..=steps {
                let p = from + (to - from) * (k as f32 / steps as f32);
                let mut events = vec![egui::Event::PointerMoved(p)];
                if finger {
                    events.insert(0, touch(egui::TouchPhase::Move, p));
                }
                frame(app, events);
            }
            if lift {
                let mut events = vec![press(to, false), egui::Event::PointerGone];
                if finger {
                    events.insert(0, touch(egui::TouchPhase::End, to));
                }
                frame(app, events);
                frame(app, Vec::new());
            }
        };

        let mut app = App::headless();
        app.show_welcome = false;
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: R::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            })
            .expect("shape");
        app.select(vec![id]);
        let fill = |app: &App| app.doc().find_shape(id).expect("rect").fill.clone();
        for _ in 0..3 {
            frame(&mut app, Vec::new());
        }
        assert_eq!(app.palette_scroll, 0);
        assert!(
            swatch_count(&app, Row::Main) > visible_slots(300.0, STRIDE),
            "the row is longer than its room"
        );
        let fill0 = fill(&app);
        // The fifth swatch, dragged 80 px along the row: the first 10
        // decide the slide, the 70 after it scroll three swatches.
        let at = egui::pos2(LEAD + 1.0 + 4.0 * STRIDE + 10.0, 13.0);
        drag(&mut app, at, at - egui::vec2(80.0, 0.0), 8, true, true);
        assert_eq!(app.palette_scroll, 3);
        assert_eq!(fill(&app), fill0, "the slide filled nothing");
        assert_eq!(app.palette_drag, None);
        // Off the row: the colour comes along.
        drag(&mut app, at, at - egui::vec2(0.0, 30.0), 3, true, false);
        assert!(
            app.palette_drag.is_some(),
            "a finger dragged off the row drags the colour"
        );
        assert_eq!(app.palette_scroll, 3);
        frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(
            &mut app,
            vec![
                touch(egui::TouchPhase::End, at - egui::vec2(0.0, 30.0)),
                press(at - egui::vec2(0.0, 30.0), false),
                egui::Event::PointerGone,
            ],
        );
        frame(&mut app, Vec::new());
        // The mouse along the row scrolls nothing (it drags the colour).
        drag(&mut app, at, at - egui::vec2(80.0, 0.0), 8, false, false);
        assert_eq!(app.palette_scroll, 3, "the mouse scrolls nothing");
        frame(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        frame(
            &mut app,
            vec![
                press(at - egui::vec2(80.0, 0.0), false),
                egui::Event::PointerGone,
            ],
        );
        assert_eq!(app.palette_scroll, 3);
        assert_eq!(fill(&app), fill0);
    }
}
