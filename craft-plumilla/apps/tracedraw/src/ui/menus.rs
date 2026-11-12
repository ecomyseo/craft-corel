//! The menu bar. Items without an implementation are shown disabled so
//! the layout is complete; wiring them is tracked in docs/features.md.

use crate::app::{App, DockerTab};
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::ui::dialogs::Dialog;
use crate::ui::tips::RichTip;
use egui::Ui;
use tracedraw_core::document::ShapeKind;
use tracedraw_core::Command;

const MENU_WIDTH: f32 = 250.0;
/// Menu rows: 26 px tall, a 30 px grey gutter
/// for icons and check marks, labels from 38 px, shortcuts right-aligned
/// 28 px from the edge, 13 px text.
const ROW_HEIGHT: f32 = 26.0;
const GUTTER: f32 = 30.0;
const LABEL_X: f32 = 38.0;
const SHORTCUT_PAD: f32 = 28.0;
const FONT: f32 = 13.0;
const SEPARATOR_HEIGHT: f32 = 7.0;

/// The style of menu pop-ups: white, a one-pixel grey frame, square
/// corners, rows touching.
pub(crate) fn menu_popup_style(style: &mut egui::Style) {
    egui::containers::menu::menu_style(style);
    style.visuals.window_fill = crate::theme::Tokens::menu();
    style.visuals.window_stroke = egui::Stroke::new(1.0, crate::theme::Tokens::window_border());
    style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    style.visuals.popup_shadow = crate::theme::shadow(egui::epaint::Shadow {
        offset: [2, 2],
        blur: 4,
        spread: 0,
        color: egui::Color32::from_black_alpha(30),
    });
    style.spacing.menu_margin = egui::Margin {
        left: 0,
        right: 0,
        top: 2,
        bottom: 2,
    };
    style.spacing.item_spacing.y = 0.0;
    style.override_font_id = Some(egui::FontId::proportional(FONT));
}

/// The grey gutter down the left of a menu body, under its rows: reserved
/// first, sized once the rows are laid out. A body taller than the window
/// scrolls (wheel, scroll bar, keyboard focus) instead of running past the
/// window's edge; the pop-up then moves to fit inside the window.
pub(crate) fn body<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let limit = screen_body_limit(ui);
    body_within(ui, limit, add)
}

/// The tallest a menu body may be anywhere in the window: the window's
/// height less the pop-up frame around the rows.
fn screen_body_limit(ui: &Ui) -> f32 {
    let frame = egui::Frame::menu(ui.style()).total_margin().sum().y;
    body_limit(ui.ctx().content_rect().height() - frame)
}

/// The tallest the body of a menu-bar menu may be: the room from its
/// title's bottom edge (`title_bottom`, last frame's) down to the window's
/// bottom edge, so the menu stays under its title instead of moving up
/// over the bar. A pixel is kept spare so rounding never makes the menu
/// miss the room below its title.
fn below_body_limit(ui: &Ui, title_bottom: Option<f32>) -> f32 {
    let Some(title_bottom) = title_bottom else {
        return screen_body_limit(ui);
    };
    let frame = egui::Frame::menu(ui.style()).total_margin().sum().y;
    body_limit(ui.ctx().content_rect().bottom() - title_bottom - frame - 1.0)
}

/// At least a few rows, however small the window.
fn body_limit(room: f32) -> f32 {
    room.max(3.0 * ROW_HEIGHT)
}

/// A menu body at most `max_height` tall; taller rows scroll.
fn body_within<R>(ui: &mut Ui, max_height: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let width_id = ui.id().with("menu_width");
    let width = ui.ctx().data(|d| d.get_temp::<f32>(width_id));
    if let Some(w) = width {
        ui.set_min_width(w);
    }
    // The pop-up's room is last frame's size (or a default on its first
    // frame); the rows may need more, up to the limit.
    ui.set_max_height(max_height);
    egui::ScrollArea::vertical()
        .id_salt("menu_scroll")
        .max_height(max_height)
        .show(ui, |ui| {
            if let Some(w) = width {
                ui.set_min_width(w);
            }
            body_rows(ui, width_id, add)
        })
        .inner
}

fn body_rows<R>(ui: &mut Ui, width_id: egui::Id, add: impl FnOnce(&mut Ui) -> R) -> R {
    let slot = ui.painter().add(egui::Shape::Noop);
    let r = {
        let _keys = access::Scope::new();
        add(ui)
    };
    // The row the keys moved to stays in view (#1839).
    crate::ui::keynav::scroll_into_view(ui);
    let rows = ui.min_rect();
    ui.ctx().data_mut(|d| d.insert_temp(width_id, rows.width()));
    let gutter = egui::Rect::from_min_max(
        egui::pos2(rows.left(), rows.top() - 2.0),
        egui::pos2(rows.left() + GUTTER, rows.bottom() + 2.0),
    );
    ui.painter().set(
        slot,
        egui::Shape::rect_filled(gutter, 0.0, crate::theme::Tokens::menu_gutter()),
    );
    r
}

/// One menu row drawn: icon or check mark in the
/// gutter, label, shortcut right-aligned, hover box.
fn menu_row(
    ui: &mut Ui,
    label: &str,
    shortcut: &str,
    enabled: bool,
    mark: Mark,
) -> (egui::Response, egui::layers::ShapeIdx) {
    menu_row_at(ui, label, shortcut, enabled, mark, LABEL_X)
}

/// [`menu_row`] with the label starting `label_x` from the row's left
/// edge (further right than usual when a picture sits after the gutter).
fn menu_row_at(
    ui: &mut Ui,
    label: &str,
    shortcut: &str,
    enabled: bool,
    mark: Mark,
    label_x: f32,
) -> (egui::Response, egui::layers::ShapeIdx) {
    // Fixed width like a native menu; longer translations widen their row.
    let needed = ui.fonts_mut(|f| {
        let l = f
            .layout_no_wrap(
                label.to_string(),
                egui::FontId::proportional(FONT),
                crate::theme::Tokens::text(),
            )
            .size()
            .x;
        let s = if shortcut.is_empty() {
            0.0
        } else {
            f.layout_no_wrap(
                shortcut.to_string(),
                egui::FontId::proportional(FONT),
                crate::theme::Tokens::text(),
            )
            .size()
            .x + 32.0
        };
        label_x + l + s + SHORTCUT_PAD
    });
    // As wide as the widest row (remembered by `body` from the last frame).
    let width = MENU_WIDTH.max(needed).max(ui.min_rect().width());
    let (rect, resp) = ui.allocate_exact_size(
        egui::vec2(width, ROW_HEIGHT),
        if enabled {
            egui::Sense::click()
        } else {
            egui::Sense::hover()
        },
    );
    let mut resp = resp;
    // The access key: underlined; pressed while this is the innermost
    // open menu, it works as a click on the row.
    let access = access::pick(label);
    if let Some((_, c)) = access {
        if enabled && access::pressed(ui, c) {
            resp.flags
                .insert(egui::response::Flags::FAKE_PRIMARY_CLICKED);
        }
    }
    // The keyboard's row (#1839) is lit like the row under the pointer;
    // while the keys move the highlight, the pointer's row is not lit.
    if enabled {
        crate::ui::keynav::note_menu_row(ui.ctx(), resp.id, mark == Mark::Submenu);
    }
    let lit = enabled
        && (resp.has_focus() || resp.hovered() && !crate::ui::keynav::keyboard_mode(ui.ctx()));
    let painter = ui.painter();
    let slot = painter.add(egui::Shape::Noop);
    if lit {
        painter.set(slot, highlight(rect));
    }
    let color = if lit {
        crate::theme::on_highlight()
    } else if enabled {
        crate::theme::Tokens::text()
    } else {
        crate::theme::Tokens::menu_text_off()
    };
    if let Some((at, _)) = access {
        access::underline(
            painter,
            egui::pos2(rect.left() + label_x, rect.center().y),
            label,
            at,
            FONT,
            color,
        );
    }
    let mid = egui::pos2(rect.left() + GUTTER / 2.0, rect.center().y);
    match mark {
        Mark::Check(true) => {
            // A bold check mark.
            let s = egui::Stroke::new(2.0, color);
            painter.line_segment(
                [mid + egui::vec2(-5.0, 0.0), mid + egui::vec2(-1.5, 3.5)],
                s,
            );
            painter.line_segment(
                [mid + egui::vec2(-1.5, 3.5), mid + egui::vec2(5.0, -4.5)],
                s,
            );
        }
        Mark::Radio(true) => {
            painter.circle_filled(mid, 3.5, color);
        }
        _ => {}
    }
    painter.text(
        egui::pos2(rect.left() + label_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(FONT),
        color,
    );
    if !shortcut.is_empty() {
        painter.text(
            egui::pos2(rect.right() - SHORTCUT_PAD, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::proportional(FONT),
            color,
        );
    }
    if mark == Mark::Submenu {
        let c = egui::pos2(rect.right() - 13.0, rect.center().y);
        painter.add(egui::epaint::PathShape::convex_polygon(
            vec![
                c + egui::vec2(-2.0, -4.0),
                c + egui::vec2(2.0, 0.0),
                c + egui::vec2(-2.0, 4.0),
            ],
            if enabled {
                crate::theme::Tokens::text_dim()
            } else {
                crate::theme::Tokens::menu_text_off()
            },
            egui::Stroke::NONE,
        ));
    }
    (resp, slot)
}

/// The box behind a hovered row or an open submenu's row.
fn highlight(rect: egui::Rect) -> egui::Shape {
    egui::Shape::Rect(egui::epaint::RectShape::new(
        rect.shrink2(egui::vec2(2.0, 0.0)),
        0.0,
        crate::theme::Tokens::menu_hover(),
        egui::Stroke::new(1.0, crate::theme::Tokens::menu_hover_line()),
        egui::StrokeKind::Inside,
    ))
}

/// What a row shows in its gutter and at its right end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    None,
    Check(bool),
    Radio(bool),
    Submenu,
}

/// A separator: a light line from the gutter to the right edge.
pub(crate) fn sep(ui: &mut Ui) {
    if invoke::active() {
        #[cfg(test)]
        invoke::separator();
        return;
    }
    let width = ui.min_rect().width().max(MENU_WIDTH);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(width, SEPARATOR_HEIGHT), egui::Sense::hover());
    let y = rect.center().y.round() - 0.5;
    ui.painter().rect_filled(
        egui::Rect::from_min_max(
            egui::pos2(rect.left() + GUTTER + 2.0, y),
            egui::pos2(rect.right(), y + 1.0),
        ),
        0.0,
        crate::theme::Tokens::border(),
    );
}

/// A menu item; `key` is an i18n key. Returns true when clicked. Items
/// with a toolbar counterpart show its icon in the gutter.
pub(crate) fn item(ui: &mut Ui, key: &str, shortcut: &str, enabled: bool) -> bool {
    item_text(ui, &tr(key), shortcut, enabled, menu_icon(key), key)
}

/// A menu item with a hover tip shown while it is greyed (why it cannot
/// run, such as an AI service that makes drawings only).
pub(crate) fn item_tip(
    ui: &mut Ui,
    key: &str,
    shortcut: &str,
    enabled: bool,
    tip: Option<&str>,
) -> bool {
    item_text_tip(ui, &tr(key), shortcut, enabled, menu_icon(key), key, tip)
}

/// A menu item with a ready label (a style name, a file).
pub(crate) fn item_label(ui: &mut Ui, label: &str, shortcut: &str, enabled: bool) -> bool {
    item_text(ui, label, shortcut, enabled, None, label)
}

/// Help > What's New: the Welcome Screen's history of the last releases.
pub(crate) fn open_whats_new(app: &mut App) {
    app.show_welcome = true;
    app.welcome_tab = crate::app::WelcomeTab::News;
}

/// A menu item with a ready label and an optional icon; `id` names it for
/// the test replay.
/// Help > Mark New Features: the menu rows new in this version and the
/// submenus that lead to them, by their keys. Each version replaces the
/// list with its own.
pub const NEW_IN_THIS_VERSION: &[&str] = &[
    // View > Scrollbars hides the drawing window's scrollbars.
    "menu.view",
    "menu.view.scrollbars",
    // View > Page > Page Shadow, apart from Page Border.
    "menu.view.page",
    "menu.view.page_shadow",
    // Bitmaps > Crop to Frame cuts a framed picture to its frame.
    "menu.bitmaps",
    "menu.bitmaps.crop_to_frame",
];

thread_local! {
    /// Whether Help > Mark New Features is on (set from the app each
    /// frame the menus draw).
    static HIGHLIGHT_NEW: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The Centerline presets, as Image > Trace and a bitmap's right-click
/// menu list them.
pub const CENTERLINE_TRACES: [(&str, crate::trace::Preset); 2] = [
    (
        "menu.bitmaps.trace_technical",
        crate::trace::Preset::Technical,
    ),
    (
        "menu.bitmaps.trace_line_drawing",
        crate::trace::Preset::LineDrawing,
    ),
];

/// The Outline presets.
pub const OUTLINE_TRACES: [(&str, crate::trace::Preset); 7] = [
    ("menu.bitmaps.trace_line_art", crate::trace::Preset::LineArt),
    ("menu.bitmaps.trace_logo", crate::trace::Preset::Logo),
    (
        "menu.bitmaps.trace_detailed_logo",
        crate::trace::Preset::DetailedLogo,
    ),
    ("menu.bitmaps.trace_clipart", crate::trace::Preset::Clipart),
    (
        "menu.bitmaps.trace_low_quality",
        crate::trace::Preset::LowQualityImage,
    ),
    (
        "menu.bitmaps.trace_high_quality",
        crate::trace::Preset::HighQualityImage,
    ),
    (
        "menu.bitmaps.trace_optimized",
        crate::trace::Preset::Optimized,
    ),
];

pub fn set_highlight_new(on: bool) {
    HIGHLIGHT_NEW.with(|h| h.set(on));
}

/// Whether the row `key` is marked as new now.
pub fn marked_new(key: &str) -> bool {
    HIGHLIGHT_NEW.with(|h| h.get()) && NEW_IN_THIS_VERSION.contains(&key)
}

/// The mark of a new row: a small accent disc at the row's right end.
fn paint_new_mark(ui: &Ui, row: egui::Rect) {
    let c = egui::pos2(row.right() - 8.0, row.center().y);
    ui.painter().circle_filled(c, 3.5, Tokens::ACCENT);
}

fn item_text(
    ui: &mut Ui,
    label: &str,
    shortcut: &str,
    enabled: bool,
    icon: Option<crate::ui::icons::Action>,
    id: &str,
) -> bool {
    item_text_tip(ui, label, shortcut, enabled, icon, id, None)
}

/// [`item_text`] with a hover tip for the greyed row.
fn item_text_tip(
    ui: &mut Ui,
    label: &str,
    shortcut: &str,
    enabled: bool,
    icon: Option<crate::ui::icons::Action>,
    id: &str,
    tip: Option<&str>,
) -> bool {
    if invoke::active() {
        return invoke::hit(id, shortcut, enabled);
    }
    let shortcut = invoke::shown_shortcut(id, shortcut);
    let (r, _) = menu_row(ui, label, &shortcut, enabled, Mark::None);
    let r = match tip {
        Some(t) if !enabled => r.rich_tip(t),
        _ => r,
    };
    if marked_new(id) {
        paint_new_mark(ui, r.rect);
    }
    #[cfg(test)]
    if replay::hit(id, enabled) {
        return true;
    }
    let _ = id;
    let ir = egui::Rect::from_center_size(
        egui::pos2(r.rect.left() + GUTTER / 2.0, r.rect.center().y),
        egui::vec2(16.0, 16.0),
    );
    let color = if enabled {
        Tokens::icon()
    } else {
        crate::theme::Tokens::menu_text_off()
    };
    if let Some(a) = icon {
        crate::ui::icons::draw_action(ui.painter(), ir, a, color);
    } else if let Some(p) = menu_pic(id) {
        crate::ui::propbar::draw_pic(ui.painter(), ir.expand(2.0), p, color);
    } else if let Some(g) = crate::ui::menu_glyphs::for_key(id) {
        crate::ui::menu_glyphs::draw(ui.painter(), ir, g, color);
    }
    if r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

/// A menu item with a ready label and a picture (a texture) in the
/// gutter, greyed with the row when it cannot run.
fn item_picture(
    ui: &mut Ui,
    label: &str,
    shortcut: &str,
    enabled: bool,
    picture: Option<egui::TextureId>,
) -> bool {
    if invoke::active() {
        return invoke::hit(label, shortcut, enabled);
    }
    let shortcut = invoke::shown_shortcut(label, shortcut);
    let (r, _) = menu_row(ui, label, &shortcut, enabled, Mark::None);
    #[cfg(test)]
    if replay::hit(label, enabled) {
        return true;
    }
    if let Some(tex) = picture {
        let ir = egui::Rect::from_center_size(
            egui::pos2(r.rect.left() + GUTTER / 2.0, r.rect.center().y),
            egui::vec2(16.0, 16.0),
        );
        let tint = if enabled {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_white_alpha(70)
        };
        ui.painter().image(
            tex,
            ir,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    }
    if enabled && r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

/// Toolbar icon for a menu item, when it has one.
fn menu_icon(key: &str) -> Option<crate::ui::icons::Action> {
    use crate::ui::icons::Action;
    Some(match key {
        "menu.file.new" => Action::New,
        "menu.file.open" => Action::Open,
        "menu.file.save" => Action::Save,
        "menu.file.print" => Action::Print,
        "menu.file.import" => Action::Import,
        "menu.file.export" => Action::Export,
        "menu.file.publish_to_pdf" => Action::Pdf,
        "menu.edit.cut" => Action::Cut,
        "menu.edit.copy" => Action::Copy,
        "menu.edit.paste" => Action::Paste,
        "menu.edit.undo" => Action::Undo,
        "menu.edit.redo" => Action::Redo,
        "menu.tools.options_app" => Action::Options,
        "menu.view.fullscreen_preview" => Action::Fullscreen,
        _ => return None,
    })
}

/// The top menus' title keys, in the menu bar's order.
pub(crate) fn menu_titles() -> Vec<&'static str> {
    all_menus().iter().map(|(k, _)| *k).collect()
}

/// Draw the picture the menu row of `key` shows into `r`; false when the
/// row has none.
pub(crate) fn draw_menu_picture(
    painter: &egui::Painter,
    r: egui::Rect,
    key: &str,
    color: egui::Color32,
) -> bool {
    if let Some(a) = menu_icon(key) {
        crate::ui::icons::draw_action(painter, r, a, color);
        true
    } else if let Some(p) = menu_pic(key) {
        crate::ui::propbar::draw_pic(painter, r.expand(2.0), p, color);
        true
    } else if let Some(g) = crate::ui::menu_glyphs::for_key(key) {
        crate::ui::menu_glyphs::draw(painter, r, g, color);
        true
    } else {
        false
    }
}

/// Whether the menu item `key` has a picture to draw.
pub(crate) fn has_menu_picture(key: &str) -> bool {
    menu_icon(key).is_some()
        || menu_pic(key).is_some()
        || crate::ui::menu_glyphs::for_key(key).is_some()
}

/// Property bar picture for a menu item, when it has one.
fn menu_pic(key: &str) -> Option<crate::ui::propbar::Pic> {
    use crate::ui::propbar::Pic;
    Some(match key {
        "menu.view.zoom_in" => Pic::ZoomIn,
        "menu.view.zoom_out" => Pic::ZoomOut,
        "menu.view.zoom_to_fit" => Pic::ZoomAll,
        "menu.object.group_group" => Pic::Group,
        "menu.object.group_ungroup" => Pic::Ungroup,
        "menu.object.group_ungroup_all" => Pic::UngroupAll,
        "menu.object.combine" => Pic::Combine,
        "menu.object.break_apart"
        | "menu.object.break_shadow_apart"
        | "menu.object.break_contour_apart"
        | "menu.object.break_blend_apart"
        | "menu.object.break_extrude_apart" => Pic::BreakApart,
        "menu.object.weld" => Pic::Weld,
        "menu.object.trim" => Pic::Trim,
        "menu.object.intersect" => Pic::Intersect,
        "menu.object.simplify" => Pic::Simplify,
        "menu.object.front_minus_back" => Pic::FrontMinusBack,
        "menu.object.back_minus_front" => Pic::BackMinusFront,
        "menu.object.boundary" => Pic::Boundary,
        "menu.object.to_front_of_page" => Pic::ToFront,
        "menu.object.to_front_of_layer" => Pic::ToFront,
        "menu.object.to_back_of_page" => Pic::ToBack,
        "menu.object.to_back_of_layer" => Pic::ToBack,
        "menu.object.convert_to_curves" => Pic::ToCurves,
        "menu.object.lock_object" | "menu.object.lock" => Pic::Lock(true),
        "menu.object.group" => Pic::Group,
        "menu.object.shaping" => Pic::Weld,
        "menu.object.transformations" => Pic::Position,
        "menu.object.tf_position" => Pic::Position,
        "menu.object.tf_rotate" => Pic::Rotate,
        "menu.object.tf_scale" => Pic::ScaleMirror,
        "menu.object.tf_size" => Pic::SizeBox,
        "menu.object.tf_skew" => Pic::SkewBox,
        "menu.object.align_distribute_docker" => Pic::AlignDistribute,
        "menu.object.convert_to_bitmap" => Pic::ConvertToBitmap,
        "menu.object.copy_envelope_from" => Pic::EnvelopeCopy,
        "menu.effects.envelope" => Pic::EnvelopeUnconstrained,
        "menu.object.shaping_docker" => Pic::Weld,
        "menu.text.tabs" => Pic::Tabs,
        "menu.text.columns" => Pic::Columns,
        "menu.text.bullets" => Pic::Bullets,
        "menu.text.drop_cap" => Pic::DropCap,
        "menu.text.text_statistics" => Pic::TextStatistics,
        "menu.text.edit_text" => Pic::EditText,
        "menu.text.convert" => Pic::ConvertText,
        "menu.text.fit_text_to_path" => Pic::FitTextToPath,
        "menu.text.straighten_text" => Pic::StraightenText,
        "menu.text.align_to_baseline" => Pic::AlignBaseline,
        "menu.text.change_case" => Pic::ChangeCase,
        "menu.text.encode" => Pic::Encode,
        "menu.bitmaps.convert_to_bitmap" => Pic::ConvertToBitmap,
        "menu.bitmaps.straighten_image" => Pic::StraightenImage,
        "menu.bitmaps.edit_bitmap" => Pic::EditBitmap,
        "menu.bitmaps.crop_bitmap" => Pic::CropBitmap,
        "menu.bitmaps.crop_to_frame" => Pic::CropBitmap,
        "menu.bitmaps.resample" => Pic::Resample,
        "menu.bitmaps.break_link" => Pic::BreakLink,
        "menu.bitmaps.update_from_link" => Pic::UpdateLink,
        "menu.bitmaps.quick_trace" => Pic::QuickTrace,
        "menu.window.new_window" => Pic::NewWindow,
        "menu.window.refresh_window" => Pic::RefreshWindow,
        "menu.window.close_window" => Pic::CloseWindow,
        "menu.window.close_all" => Pic::CloseAll,
        "menu.window.cascade" => Pic::Cascade,
        "menu.window.tile_horizontally" => Pic::TileHorizontally,
        "menu.window.tile_vertically" => Pic::TileVertically,
        "menu.window.combine_windows" => Pic::CombineWindows,
        "menu.window.dock_window" => Pic::DockWindow,
        _ => return None,
    })
}

/// A checkable item (check mark in the gutter when `on`).
pub(crate) fn check(ui: &mut Ui, key: &str, shortcut: &str, on: bool) -> bool {
    mark_item(ui, key, shortcut, Mark::Check(on))
}

/// A checkable item with a ready label (a palette's name); `id` names it
/// for the command list and the test replay.
pub(crate) fn check_label(ui: &mut Ui, label: &str, id: &str, on: bool) -> bool {
    if invoke::active() {
        return invoke::hit(id, "", true);
    }
    let (r, _) = menu_row(ui, label, "", true, Mark::Check(on));
    #[cfg(test)]
    if replay::hit(id, true) {
        return true;
    }
    if r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

/// The width of the picture slot of a [`check_pic`] row, between the
/// gutter and the label.
const PIC_SLOT: f32 = 22.0;

/// A checkable item with a picture after the gutter (check mark, picture,
/// name): `pic` paints the picture in a 16 px square, `None` leaves the
/// slot blank so the names of a list stay aligned.
pub(crate) fn check_pic(
    ui: &mut Ui,
    key: &str,
    on: bool,
    pic: Option<&dyn Fn(&egui::Painter, egui::Rect)>,
) -> bool {
    if invoke::active() {
        return invoke::hit(key, "", true);
    }
    let shortcut = invoke::shown_shortcut(key, "");
    let (r, _) = menu_row_at(
        ui,
        &tr(key),
        &shortcut,
        true,
        Mark::Check(on),
        LABEL_X + PIC_SLOT,
    );
    if marked_new(key) {
        paint_new_mark(ui, r.rect);
    }
    #[cfg(test)]
    if replay::hit(key, true) {
        return true;
    }
    if let Some(pic) = pic {
        let ir = egui::Rect::from_center_size(
            egui::pos2(
                r.rect.left() + LABEL_X + PIC_SLOT / 2.0 - 2.0,
                r.rect.center().y,
            ),
            egui::vec2(16.0, 16.0),
        );
        pic(ui.painter(), ir);
    }
    if r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

/// One of a set of exclusive items (a dot in the gutter when `on`).
pub(crate) fn radio(ui: &mut Ui, key: &str, shortcut: &str, on: bool) -> bool {
    mark_item(ui, key, shortcut, Mark::Radio(on))
}

/// An exclusive item that can be unavailable (greyed, not clickable).
pub(crate) fn radio_if(ui: &mut Ui, key: &str, shortcut: &str, on: bool, enabled: bool) -> bool {
    mark_item_if(ui, key, shortcut, Mark::Radio(on), enabled)
}

/// A checkable item, greyed when `enabled` is false.
pub(crate) fn check_if(ui: &mut Ui, key: &str, shortcut: &str, on: bool, enabled: bool) -> bool {
    mark_item_if(ui, key, shortcut, Mark::Check(on), enabled)
}

fn mark_item(ui: &mut Ui, key: &str, shortcut: &str, mark: Mark) -> bool {
    mark_item_if(ui, key, shortcut, mark, true)
}

fn mark_item_if(ui: &mut Ui, key: &str, shortcut: &str, mark: Mark, enabled: bool) -> bool {
    if invoke::active() {
        return invoke::hit(key, shortcut, enabled);
    }
    let shortcut = invoke::shown_shortcut(key, shortcut);
    let (r, _) = menu_row(ui, &tr(key), &shortcut, enabled, mark);
    if marked_new(key) {
        paint_new_mark(ui, r.rect);
    }
    #[cfg(test)]
    if replay::hit(key, enabled) {
        return true;
    }
    if enabled && r.clicked() {
        ui.close();
        true
    } else {
        false
    }
}

/// A submenu that is not made yet: shown greyed, left out of the
/// commands that take shortcuts.
fn todo_sub(ui: &mut Ui, key: &str) {
    if invoke::active() {
        // Greyed and empty: only its place counts (the menus' outline).
        invoke::enter(key);
        invoke::leave();
        return;
    }
    let (r, _) = menu_row(ui, &tr(key), "", false, Mark::Submenu);
    row_picture(ui, r.rect, key, false);
}

/// The picture of the row of `key` in its gutter, when it has one.
fn row_picture(ui: &Ui, row: egui::Rect, key: &str, enabled: bool) {
    let ir = egui::Rect::from_center_size(
        egui::pos2(row.left() + GUTTER / 2.0, row.center().y),
        egui::vec2(16.0, 16.0),
    );
    let color = if enabled {
        Tokens::icon()
    } else {
        crate::theme::Tokens::menu_text_off()
    };
    draw_menu_picture(ui.painter(), ir, key, color);
}

/// A submenu that is greyed and does not open while `enabled` is false;
/// its items still count for shortcuts and the command list.
pub(crate) fn sub_if<R>(ui: &mut Ui, key: &str, enabled: bool, add: impl FnOnce(&mut Ui) -> R) {
    #[cfg(test)]
    let replaying = replay::active();
    #[cfg(not(test))]
    let replaying = false;
    if enabled || invoke::active() || replaying {
        sub(ui, key, add);
    } else {
        let (r, _) = menu_row(ui, &tr(key), "", false, Mark::Submenu);
        row_picture(ui, r.rect, key, false);
    }
}

/// A submenu row: the submenu opens beside it while it is hovered.
pub(crate) fn sub<R>(ui: &mut Ui, key: &str, add: impl FnOnce(&mut Ui) -> R) {
    let _ = sub_row(ui, key, add);
}

/// [`sub`], returning the row when it was drawn.
fn sub_row<R>(ui: &mut Ui, key: &str, add: impl FnOnce(&mut Ui) -> R) -> Option<egui::Response> {
    if invoke::active() {
        // Walked without the menus on screen: the submenu's items count
        // as if it were open.
        invoke::enter(key);
        let _ = add(ui);
        invoke::leave();
        return None;
    }
    #[cfg(test)]
    if replay::active() {
        // Replaying: submenus are laid out inline so their items count.
        let _ = add(ui);
        return None;
    }
    let (resp, slot) = menu_row(ui, &tr(key), "", true, Mark::Submenu);
    row_picture(ui, resp.rect, key, true);
    if marked_new(key) {
        // Left of the submenu arrow.
        paint_new_mark(ui, resp.rect.translate(egui::vec2(-14.0, 0.0)));
    }
    let open = egui::containers::menu::MenuState::from_ui(ui, |state, _| {
        state.open_item == Some(egui::containers::menu::SubMenu::id_from_widget_id(resp.id))
    });
    if open {
        ui.painter().set(slot, highlight(resp.rect));
    }
    // A row scrolled out of a long menu's view does not open its submenu
    // from a pointer outside that view (over the menu bar, say).
    let view = ui.clip_rect();
    let pointer_in_view = ui
        .ctx()
        .pointer_hover_pos()
        .is_none_or(|p| view.contains(p));
    if !view.intersects(resp.rect) || !(open || pointer_in_view) {
        return Some(resp);
    }
    egui::containers::menu::SubMenu::new().show(ui, &resp, |ui| {
        ui.set_min_width(MENU_WIDTH);
        body(ui, add)
    });
    Some(resp)
}

/// Access keys: one letter of each menu row, underlined, that picks the
/// row from the keyboard while its menu is open; the menu bar's titles
/// open with Alt and their letter.
pub(crate) mod access {
    use egui::Ui;
    use std::cell::RefCell;

    thread_local! {
        /// The letters taken in each menu body being drawn, innermost last.
        static TAKEN: RefCell<Vec<Vec<char>>> = const { RefCell::new(Vec::new()) };
    }

    /// The letters of one menu body, for as long as it is drawn.
    pub struct Scope;

    impl Scope {
        pub fn new() -> Scope {
            TAKEN.with(|t| t.borrow_mut().push(Vec::new()));
            Scope
        }
    }

    impl Drop for Scope {
        fn drop(&mut self) {
            TAKEN.with(|t| {
                t.borrow_mut().pop();
            });
        }
    }

    /// The access key of `label` among `taken`: the first letter or digit
    /// starting a word that is free, else the first free one anywhere.
    /// Returns its char index and the letter in lower case.
    pub fn choose(label: &str, taken: &[char]) -> Option<(usize, char)> {
        let chars: Vec<char> = label.chars().collect();
        // Letters and digits a key press can give.
        let free = |c: char| c.is_ascii_alphanumeric() && !taken.contains(&c.to_ascii_lowercase());
        let lower = |c: char| c.to_lowercase().next().unwrap_or(c);
        let word_start = |i: usize| i == 0 || !chars[i - 1].is_alphanumeric();
        (0..chars.len())
            .find(|&i| word_start(i) && free(chars[i]))
            .or_else(|| (0..chars.len()).find(|&i| free(chars[i])))
            .map(|i| (i, lower(chars[i])))
    }

    /// Keep `letters` out of the rows of the menu body being drawn (the
    /// menu bar's titles, for the menus under More).
    pub fn reserve(letters: &[char]) {
        TAKEN.with(|t| {
            if let Some(taken) = t.borrow_mut().last_mut() {
                taken.extend(letters.iter().filter(|c| **c != ' '));
            }
        });
    }

    /// The access key of a row of the menu body being drawn (none outside
    /// a body).
    pub fn pick(label: &str) -> Option<(usize, char)> {
        TAKEN.with(|t| {
            let mut t = t.borrow_mut();
            let taken = t.last_mut()?;
            let picked = choose(label, taken)?;
            taken.push(picked.1);
            Some(picked)
        })
    }

    /// The key of an ASCII letter or digit.
    pub fn key_of(c: char) -> Option<egui::Key> {
        egui::Key::from_name(&c.to_ascii_uppercase().to_string())
    }

    /// `c` was pressed without Ctrl or Alt and this menu has no submenu
    /// open (the innermost open menu takes the keys).
    pub fn pressed(ui: &Ui, c: char) -> bool {
        let Some(key) = key_of(c) else {
            return false;
        };
        let hit = ui.input(|i| {
            i.events.iter().any(|e| {
                matches!(e, egui::Event::Key { key: k, pressed: true, modifiers, .. }
                    if *k == key && !(modifiers.ctrl || modifiers.alt || modifiers.command))
            })
        });
        hit && egui::containers::menu::MenuState::from_ui(ui, |state, _| state.open_item.is_none())
    }

    /// Underline the char at `at` of `label` drawn left-centred at `pos`.
    pub fn underline(
        painter: &egui::Painter,
        pos: egui::Pos2,
        label: &str,
        at: usize,
        size: f32,
        color: egui::Color32,
    ) {
        let font = egui::FontId::proportional(size);
        let before: String = label.chars().take(at).collect();
        let this: String = label.chars().skip(at).take(1).collect();
        let (x0, w, h) = painter.fonts_mut(|f| {
            let full = f.layout_no_wrap(label.to_string(), font.clone(), color);
            (
                f.layout_no_wrap(before, font.clone(), color).size().x,
                f.layout_no_wrap(this, font.clone(), color).size().x,
                full.size().y,
            )
        });
        let y = (pos.y + h / 2.0 - 2.0).round() + 0.5;
        painter.line_segment(
            [egui::pos2(pos.x + x0, y), egui::pos2(pos.x + x0 + w, y)],
            egui::Stroke::new(1.0, color),
        );
    }

    /// A menu bar title with its access key underlined.
    pub fn title(
        label: &str,
        at: Option<usize>,
        size: f32,
        color: egui::Color32,
    ) -> egui::WidgetText {
        let mut job = egui::text::LayoutJob::default();
        for (i, ch) in label.chars().enumerate() {
            job.append(
                &ch.to_string(),
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::proportional(size),
                    color,
                    underline: if Some(i) == at {
                        egui::Stroke::new(1.0, color)
                    } else {
                        egui::Stroke::NONE
                    },
                    ..Default::default()
                },
            );
        }
        job.into()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn access_keys_prefer_free_word_starts() {
            assert_eq!(choose("Save As...", &[]), Some((0, 's')));
            assert_eq!(choose("Save As...", &['s']), Some((5, 'a')));
            assert_eq!(choose("Save As...", &['s', 'a']), Some((2, 'v')));
            assert_eq!(choose("...", &[]), None);
            // Rows nest: each body keeps its own letters.
            let _outer = Scope::new();
            assert_eq!(pick("File"), Some((0, 'f')));
            {
                let _inner = Scope::new();
                assert_eq!(pick("Find"), Some((0, 'f')));
            }
            assert_eq!(pick("Find"), Some((1, 'i')));
        }
    }
}

/// Menu commands without the menus on screen. A shortcut chosen in
/// Options > Customization > Commands names a menu item by its key; the
/// menus are then walked invisibly and that item reports a click
/// ([`run_command`]). The same walk lists every command with its menus
/// and built-in shortcut ([`commands`]).
pub(crate) mod invoke {
    use std::cell::RefCell;

    /// A menu command: its i18n key, built-in shortcut and the keys of
    /// the menu and submenus it sits in.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Entry {
        pub key: String,
        pub shortcut: String,
        pub path: Vec<String>,
    }

    enum Mode {
        Collect { path: Vec<String>, out: Vec<Entry> },
        Trigger { key: String, fired: bool },
    }

    thread_local! {
        static MODE: RefCell<Option<Mode>> = const { RefCell::new(None) };
        /// The user's shortcuts (`Settings::shortcuts`), for the rows.
        static BINDINGS: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
    }

    pub fn active() -> bool {
        MODE.with(|m| m.borrow().is_some())
    }

    pub fn hit(key: &str, shortcut: &str, enabled: bool) -> bool {
        MODE.with(|m| match m.borrow_mut().as_mut() {
            Some(Mode::Collect { path, out }) => {
                if key.starts_with("menu.") && !out.iter().any(|e| e.key == key) {
                    // The macOS desktop app's own key where macOS keeps
                    // the built-in one (#1409).
                    out.push(Entry {
                        key: key.to_string(),
                        shortcut: crate::shortcuts::mac_shown(shortcut.to_string()),
                        path: path.clone(),
                    });
                }
                false
            }
            Some(Mode::Trigger { key: wanted, fired }) if !*fired && enabled && wanted == key => {
                *fired = true;
                true
            }
            Some(Mode::Trigger { .. }) | None => false,
        })
    }

    #[cfg(test)]
    thread_local! {
        /// Separators (`None`) and submenus entered (their key) while
        /// collecting, in walk order: where they fall among the entries
        /// and in which menus (the menus' outline, for tests; some
        /// submenus list ready labels rather than commands).
        pub static MARKS: RefCell<Vec<(usize, Vec<String>, Option<String>)>> = const { RefCell::new(Vec::new()) };
    }

    #[cfg(test)]
    pub fn separator() {
        MODE.with(|m| {
            if let Some(Mode::Collect { path, out }) = m.borrow().as_ref() {
                MARKS.with(|s| s.borrow_mut().push((out.len(), path.clone(), None)));
            }
        });
    }

    pub fn enter(key: &str) {
        MODE.with(|m| {
            if let Some(Mode::Collect { path, out }) = m.borrow_mut().as_mut() {
                path.push(key.to_string());
                #[cfg(test)]
                MARKS.with(|s| s.borrow_mut().push((out.len(), path.clone(), Some(key.to_string()))));
                #[cfg(not(test))]
                let _ = out;
            }
        });
    }

    pub fn leave() {
        MODE.with(|m| {
            if let Some(Mode::Collect { path, .. }) = m.borrow_mut().as_mut() {
                path.pop();
            }
        });
    }

    pub fn fired() -> bool {
        MODE.with(|m| matches!(&*m.borrow(), Some(Mode::Trigger { fired: true, .. })))
    }

    pub(super) fn start_collect() {
        MODE.with(|m| {
            *m.borrow_mut() = Some(Mode::Collect {
                path: Vec::new(),
                out: Vec::new(),
            })
        });
    }

    pub(super) fn start_trigger(key: &str) {
        MODE.with(|m| {
            *m.borrow_mut() = Some(Mode::Trigger {
                key: key.to_string(),
                fired: false,
            })
        });
    }

    /// Ends a walk however it ends, so the menus never stay invisible.
    pub(super) struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            MODE.with(|m| m.borrow_mut().take());
        }
    }

    /// End the walk: the commands collected, or whether the item fired.
    pub(super) fn finish() -> (Vec<Entry>, bool) {
        MODE.with(|m| match m.borrow_mut().take() {
            Some(Mode::Collect { out, .. }) => (out, false),
            Some(Mode::Trigger { fired, .. }) => (Vec::new(), fired),
            None => (Vec::new(), false),
        })
    }

    pub fn set_bindings(bindings: &[(String, String)]) {
        BINDINGS.with(|b| {
            let mut b = b.borrow_mut();
            if b.as_slice() != bindings {
                *b = bindings.to_vec();
            }
        });
    }

    /// The shortcut a row shows: the user's for `key`, else `default`; in
    /// the browser version the key that replaces one the browser keeps
    /// (`shortcuts::BROWSER_KEYS`, #1409).
    pub fn shown_shortcut(key: &str, default: &str) -> String {
        // The first of its keys (a command may have several, #822).
        let keys = BINDINGS.with(|b| {
            crate::shortcuts::keys_for(&b.borrow(), key, Some(default))
                .into_iter()
                .next()
                .unwrap_or_default()
        });
        crate::shortcuts::shown(keys)
    }
}

/// The twelve menus in their document-menu order. Existing command keys
/// remain unchanged so saved shortcuts keep working when a menu moves.
pub(crate) fn all_menus() -> [(&'static str, fn(&mut App, &mut Ui)); 12] {
    [
        ("menu.file", file_menu),
        ("menu.edit", edit_menu),
        ("menu.view", view_menu),
        ("menu.layout", layout_menu),
        ("menu.object", object_menu),
        ("menu.effects", effects_menu),
        ("menu.bitmaps", bitmaps_menu),
        ("menu.text", text_menu),
        ("menu.table", table_menu),
        ("menu.tools", tools_menu),
        ("menu.window", window_menu),
        ("menu.help", help_menu),
    ]
}

/// The menus the bar shows with no drawing open.
const MENUS_WITHOUT_DRAWING: [&str; 4] = ["menu.file", "menu.tools", "menu.window", "menu.help"];

/// Walk the menus without showing them, in a Ui of their own far off
/// the screen, so nothing is drawn, hit or laid out beside the real
/// widgets. `shown_only`: only the menus the bar shows now.
fn walk_invisibly(app: &mut App, ctx: &egui::Context, shown_only: bool, until_fired: bool) {
    let far = egui::Rect::from_min_size(
        egui::pos2(-100_000.0, -100_000.0),
        egui::vec2(400.0, 50_000.0),
    );
    let mut ui = Ui::new(
        ctx.clone(),
        egui::Id::new("menu_commands_walk"),
        egui::UiBuilder::new()
            .invisible()
            .max_rect(far)
            .layer_id(egui::LayerId::new(
                egui::Order::Background,
                egui::Id::new("menu_commands_walk"),
            )),
    );
    for (title, body) in all_menus() {
        if until_fired && invoke::fired() {
            break;
        }
        if shown_only && !app.has_document() && !MENUS_WITHOUT_DRAWING.contains(&title) {
            continue;
        }
        invoke::enter(title);
        body(app, &mut ui);
        invoke::leave();
    }
}

/// Every command of the menu bar: its key, its built-in shortcut and the
/// menus it sits in, in menu order.
pub fn commands(app: &mut App, ctx: &egui::Context) -> Vec<invoke::Entry> {
    let _guard = invoke::Guard;
    invoke::start_collect();
    walk_invisibly(app, ctx, false, false);
    invoke::finish().0
}

/// Run the menu command `key` as its item would when clicked; false when
/// the menus shown now have no enabled item with that key.
pub fn run_command(app: &mut App, ctx: &egui::Context, key: &str) -> bool {
    let _guard = invoke::Guard;
    invoke::start_trigger(key);
    walk_invisibly(app, ctx, true, true);
    invoke::finish().1
}

/// Test hook: "click" the n-th enabled menu row of a menu body without a
/// pointer, so every menu command can be exercised headlessly.
#[cfg(test)]
pub(crate) mod replay {
    use std::cell::{Cell, RefCell};

    thread_local! {
        static TARGET: Cell<Option<usize>> = const { Cell::new(None) };
        static COUNTER: Cell<usize> = const { Cell::new(0) };
        static HIT: RefCell<Option<String>> = const { RefCell::new(None) };
    }

    /// Items that would block on a native file chooser, start another
    /// program (an editor or the web browser) or close the window.
    const SKIP: [&str; 21] = [
        "menu.bitmaps.save_bitmap_as",
        // The scanner or camera dialog of the system.
        "menu.file.acquire",
        "menu.file.send_to_desktop",
        "menu.file.send_to_documents",
        "menu.file.print",
        "menu.file.send_to_mail",
        "menu.tools.macros_load",
        "menu.help.updates",
        "menu.help.community",
        "menu.help.support",
        "menu.file.open",
        "menu.file.save",
        "menu.file.save_as",
        "menu.file.save_as_template",
        "menu.file.import",
        "menu.file.exit",
        "menu.bitmaps.edit_bitmap",
        "menu.tools.scripts_run",
        "menu.window.palette_open",
        "menu.window.workspace_export",
        "menu.window.workspace_import",
    ];

    pub fn start(target: usize) {
        TARGET.set(Some(target));
        COUNTER.set(0);
        HIT.replace(None);
    }

    /// Stop replaying; returns the key that was hit and how many rows were seen.
    pub fn finish() -> (Option<String>, usize) {
        TARGET.set(None);
        (HIT.take(), COUNTER.get())
    }

    pub fn active() -> bool {
        TARGET.get().is_some()
    }

    pub fn hit(key: &str, enabled: bool) -> bool {
        let Some(t) = TARGET.get() else {
            return false;
        };
        let i = COUNTER.get();
        COUNTER.set(i + 1);
        if i == t && enabled && !SKIP.contains(&key) {
            HIT.replace(Some(key.to_string()));
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_panels_menu_lists_every_panel_once() {
        let mut listed: Vec<DockerTab> = DockerTab::MENU_GROUPS
            .iter()
            .flat_map(|g| g.iter().copied())
            .chain(DockerTab::EFFECTS_MENU)
            .collect();
        assert_eq!(listed.len(), DockerTab::ALL.len());
        listed.dedup();
        for t in DockerTab::ALL {
            assert_eq!(listed.iter().filter(|x| **x == t).count(), 1, "{t:?}");
        }
    }

    use super::*;
    use egui::Context;
    use tracedraw_core::geometry::{Point, Rect};
    use tracedraw_core::ShapeKind;

    /// A document with one of the object kinds the menus act on.
    fn mixed_app() -> App {
        let mut app = App::headless();
        app.settings.autocorrect.enabled = false;
        let r = app
            .new_shape(ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.new_shape(ShapeKind::Ellipse {
            rect: Rect::new(70.0, 10.0, 120.0, 40.0),
            arc: None,
        });
        app.start_text(Point::new(20.0, 80.0), None);
        app.text_insert("Menu test");
        app.finish_text();
        app.select(vec![r]);
        app.duplicate();
        app.convert_to_bitmap(50.0, true);
        app.create_table(Rect::new(10.0, 120.0, 80.0, 160.0));
        app.select_all();
        app
    }

    fn menus() -> Vec<(&'static str, fn(&mut App, &mut Ui))> {
        all_menus().to_vec()
    }

    /// Menu order and real command paths include an independent Table menu.
    /// Existing keys and shortcuts still resolve after relocating commands.
    #[test]
    fn the_menu_bar_lists_twelve_menus_in_order() {
        let keys: Vec<&str> = all_menus().iter().map(|(k, _)| *k).collect();
        assert_eq!(
            keys,
            [
                "menu.file",
                "menu.edit",
                "menu.view",
                "menu.layout",
                "menu.object",
                "menu.effects",
                "menu.bitmaps",
                "menu.text",
                "menu.table",
                "menu.tools",
                "menu.window",
                "menu.help",
            ]
        );
        crate::i18n::with_thread_language("en", || {
            let titles: Vec<String> = keys.iter().map(|k| tr(k)).collect();
            assert_eq!(
                titles,
                [
                    "File", "Edit", "View", "Layout", "Object", "Effects", "Bitmaps", "Text",
                    "Table", "Tools", "Window", "Help",
                ]
            );
        });
        crate::i18n::with_thread_language("es", || {
            let titles: Vec<String> = keys.iter().map(|k| tr(k)).collect();
            assert_eq!(
                titles,
                [
                    "Archivo",
                    "Edición",
                    "Ver",
                    "Diseño",
                    "Objeto",
                    "Efectos",
                    "Mapas de bits",
                    "Texto",
                    "Tabla",
                    "Herramientas",
                    "Ventana",
                    "Ayuda",
                ]
            );
        });
        for key in MENUS_WITHOUT_DRAWING {
            assert!(keys.contains(&key), "{key} is not a menu");
        }
        let ctx = crate::theme::ui_context();
        let mut app = mixed_app();
        let mut entries = Vec::new();
        run_frame(&ctx, |ui| entries = commands(&mut app, ui.ctx()));
        let find = |key: &str| entries.iter().find(|e| e.key == key).cloned();
        assert!(
            entries
                .iter()
                .any(|e| e.path.first() == Some(&"menu.table".to_string())),
            "Table must expose its own commands"
        );
        let table = find("menu.table.create_new_table").expect("Create New Table");
        assert_eq!(table.path, ["menu.table"]);
        assert_eq!(
            find("menu.table.merge_cells")
                .expect("Merge Cells")
                .shortcut,
            "Ctrl+M"
        );
        let trace = find("menu.bitmaps.quick_trace").expect("Automatic trace");
        assert_eq!(trace.path, ["menu.bitmaps"]);
        assert_eq!(
            find("menu.bitmaps.trace_logo").expect("Logo preset").path,
            ["menu.bitmaps", "menu.bitmaps.outline_trace"]
        );
        assert_eq!(
            find("menu.layout.insert_page").expect("Insert Page").path,
            ["menu.layout"]
        );
        assert!(find("menu.text.grammar").is_some(), "the grammar checker");
        assert!(find("menu.text.grammatik").is_none(), "the old key is gone");
        // Command traversal follows the same order as the visible menu bar.
        let first = |menu: &str| {
            entries
                .iter()
                .position(|e| e.path.first().map(String::as_str) == Some(menu))
                .unwrap_or_else(|| panic!("{menu} has no command"))
        };
        let order: Vec<usize> = keys.iter().map(|k| first(k)).collect();
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
    }

    fn run_frame(ctx: &Context, f: impl FnOnce(&mut Ui)) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            ..Default::default()
        };
        let mut f = Some(f);
        let mut out = ctx.run_ui(input, |ui| {
            if let Some(f) = f.take() {
                f(ui);
            }
        });
        out.textures_delta.clear();
    }

    /// A frame of the whole window with `events` as its input.
    fn root_frame(ctx: &Context, app: &mut App, events: Vec<egui::Event>) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            events,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| crate::ui::root(app, ui));
        out.textures_delta.clear();
    }

    fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    /// The menus list their commands with the menus they sit in and their
    /// built-in shortcuts, and run one by its key with nothing shown.
    /// Page > Page Spacing: the presets and Custom... (the dialog in the
    /// drawing's units), next to Page Layout.
    #[test]
    fn the_page_menu_sets_the_page_spacing() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        let mut entries = Vec::new();
        run_frame(&ctx, |ui| entries = commands(&mut app, ui.ctx()));
        let pos = |key: &str| entries.iter().position(|e| e.key == key);
        let custom = pos("menu.layout.page_spacing_custom").expect("Custom...");
        assert_eq!(
            entries[custom].path,
            ["menu.layout", "menu.layout.page_spacing"]
        );
        let layout = pos("menu.layout.page_layout").expect("Page Layout");
        assert_eq!(pos("menu.layout.page_spacing_0"), Some(layout + 1));
        for (mm, key) in crate::multipage::SPACING_PRESETS {
            let mut ran = false;
            run_frame(&ctx, |ui| ran = run_command(&mut app, ui.ctx(), key));
            assert!(ran, "{key}");
            assert_eq!(app.page_spacing(), mm, "{key}");
        }
        run_frame(&ctx, |ui| {
            run_command(&mut app, ui.ctx(), "menu.layout.page_spacing_10");
        });
        app.units = crate::app::Units::Inches;
        let mut ran = false;
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.layout.page_spacing_custom")
        });
        assert!(ran);
        match app.dialog {
            Dialog::PageSpacing { value } => assert!((value - 10.0 / 25.4).abs() < 1e-9),
            ref d => panic!("{d:?}"),
        }
    }

    /// Page > Facing Pages and its Start On submenu (#1972): run by key,
    /// each one undo step; choosing a side turns facing pages on.
    #[test]
    fn the_page_menu_turns_facing_pages_on_and_picks_the_side() {
        use tracedraw_core::spreads::PageSide;
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        let mut entries = Vec::new();
        run_frame(&ctx, |ui| entries = commands(&mut app, ui.ctx()));
        let pos = |key: &str| entries.iter().position(|e| e.key == key);
        let left = pos("menu.layout.facing_start_left").expect("Left Side");
        assert_eq!(
            entries[left].path,
            ["menu.layout", "menu.layout.facing_start"]
        );
        let mut ran = false;
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.layout.facing_pages")
        });
        assert!(ran);
        assert!(app.doc().metadata.facing_pages);
        assert_eq!(app.doc().metadata.facing_start, PageSide::Right);
        run_frame(&ctx, |ui| {
            run_command(&mut app, ui.ctx(), "menu.layout.facing_pages");
        });
        assert!(!app.doc().metadata.facing_pages);
        run_frame(&ctx, |ui| {
            run_command(&mut app, ui.ctx(), "menu.layout.facing_start_left");
        });
        assert!(app.doc().metadata.facing_pages);
        assert_eq!(app.doc().metadata.facing_start, PageSide::Left);
        app.undo();
        assert!(!app.doc().metadata.facing_pages);
        assert_eq!(app.doc().metadata.facing_start, PageSide::Right);
    }

    #[test]
    fn menu_commands_are_listed_and_run_by_key() {
        let ctx = crate::theme::ui_context();
        let mut app = mixed_app();
        let mut entries = Vec::new();
        run_frame(&ctx, |ui| entries = commands(&mut app, ui.ctx()));
        assert!(!invoke::active());
        let find = |key: &str| entries.iter().find(|e| e.key == key).cloned();
        let export = find("menu.file.export").expect("export listed");
        assert_eq!(export.shortcut, "Ctrl+E");
        assert_eq!(export.path, ["menu.file"]);
        let all = find("menu.edit.select_all_objects").expect("submenu items listed");
        assert_eq!(all.path, ["menu.edit", "menu.edit.select_all"]);
        assert_eq!(all.shortcut, "Ctrl+A");
        assert!(find("menu.window.new_window").is_some());
        assert!(find("menu.window.cascade").is_some());
        // Every row Mark New Features marks is a command, or a menu or
        // submenu on the way to one.
        // (Window > Dockers lists the panels, which are not commands.)
        for key in NEW_IN_THIS_VERSION {
            let docker =
                *key == "menu.window.dockers" || DockerTab::ALL.iter().any(|t| t.key() == *key);
            assert!(
                docker
                    || find(key).is_some()
                    || entries.iter().any(|e| e.path.iter().any(|p| p == key)),
                "{key} is not in the menus"
            );
        }
        assert!(!marked_new("menu.view.scrollbars"));
        set_highlight_new(true);
        assert!(marked_new("menu.view.scrollbars"));
        assert!(!marked_new("menu.file.export"));
        set_highlight_new(false);
        assert!(
            find("menu.edit.select_all").is_none(),
            "submenus are not commands"
        );
        assert!(entries.len() > 200, "{} commands", entries.len());
        for (i, e) in entries.iter().enumerate() {
            assert!(e.key.starts_with("menu."), "{e:?}");
            assert!(!entries[..i].iter().any(|o| o.key == e.key), "{e:?} twice");
        }
        // Run one: nothing selected, then Select All.
        app.selection.clear();
        let mut ran = false;
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.edit.select_all_objects")
        });
        assert!(ran && !app.selection.is_empty());
        // Unknown keys and commands not available now run nothing.
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "no.such.key")
        });
        assert!(!ran);
        app.selection.clear();
        let shapes = app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum::<usize>();
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.edit.duplicate")
        });
        assert!(!ran, "Duplicate needs a selection");
        assert_eq!(
            app.doc().pages[0]
                .layers
                .iter()
                .map(|l| l.shapes.len())
                .sum::<usize>(),
            shapes
        );
        assert!(!invoke::active());
    }

    /// The Effects menu: Flatten Effects, the bitmap groups, then the
    /// effect panels, checked while open; Object > Create holds Arrowhead
    /// and Pattern Fill, which asks for the pattern's type.
    #[test]
    fn effects_menu_checks_the_open_effect_panels() {
        let ctx = crate::theme::ui_context();
        let mut app = mixed_app();
        let mut entries = Vec::new();
        run_frame(&ctx, |ui| entries = commands(&mut app, ui.ctx()));
        let find = |key: &str| entries.iter().find(|e| e.key == key).cloned();
        assert_eq!(
            find("menu.effects.flatten").expect("flatten").path,
            ["menu.effects"]
        );
        assert!(find("menu.effects.picker").is_none_or(|e| e.path == ["menu.window"]));
        assert_eq!(
            find("menu.effects.contour").expect("contour").shortcut,
            "Ctrl+F9"
        );
        let create: Vec<String> = entries
            .iter()
            .filter(|e| e.path == ["menu.object", "menu.object.create"])
            .map(|e| e.key.clone())
            .collect();
        assert_eq!(
            create,
            ["menu.object.create_arrowhead", "menu.object.create_pattern"]
        );
        assert!(find("menu.table.create_new_table").is_some());
        // Table's commands belong to its independent top-level menu.
        assert_eq!(
            find("menu.table.create_new_table").expect("table").path,
            ["menu.table"]
        );
        // The panels: checked while their tab is in the shown column.
        app.show_dockers = false;
        assert!(!effect_docker_open(&app, DockerTab::Blend));
        let mut ran = false;
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.effects.blend")
        });
        assert!(ran && app.show_dockers && app.docker_tab == DockerTab::Blend);
        assert!(effect_docker_open(&app, DockerTab::Blend));
        // Pattern Fill opens its dialog.
        let id = app.doc().pages[0]
            .layers
            .iter()
            .flat_map(|l| &l.shapes)
            .map(|s| s.id)
            .next()
            .expect("a shape");
        app.select(vec![id]);
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.object.create_pattern")
        });
        assert!(ran && matches!(app.dialog, Dialog::CreatePattern { vector: true }));
    }

    /// File > Close All closes every drawing, asking about unsaved ones.
    #[test]
    fn file_close_all_closes_every_drawing() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.new_document();
        app.new_document();
        assert_eq!(app.docs.len(), 3);
        let mut ran = false;
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.file.close_all")
        });
        assert!(ran && !app.has_document());
    }

    /// Window > New Window opens a second window of the active drawing.
    #[test]
    fn window_new_window_opens_a_second_window() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.show_welcome = false;
        let mut ran = false;
        run_frame(&ctx, |ui| {
            ran = run_command(&mut app, ui.ctx(), "menu.window.new_window")
        });
        assert!(ran);
        assert_eq!(app.docs.len(), 2);
        assert_eq!(app.active_doc, 1);
        assert!(app.document_tab_title(0).ends_with(":1"));
        assert!(app.document_tab_title(1).ends_with(":2"));
    }

    /// A shortcut of the user's runs its menu command, takes over a
    /// built-in shortcut and shows in the menu row.
    #[test]
    fn shortcuts_of_the_users_run_menu_commands() {
        use egui::{Key, Modifiers};
        let ctx = crate::theme::ui_context();
        let mut app = mixed_app();
        app.show_welcome = false;
        let count = |app: &App| {
            app.doc().pages[0]
                .layers
                .iter()
                .map(|l| l.shapes.len())
                .sum::<usize>()
        };
        app.settings
            .shortcuts
            .push(("menu.edit.select_all_objects".into(), "Alt+Shift+K".into()));
        app.selection.clear();
        root_frame(
            &ctx,
            &mut app,
            vec![key(Key::K, Modifiers::ALT | Modifiers::SHIFT)],
        );
        root_frame(&ctx, &mut app, vec![]);
        assert!(!app.selection.is_empty(), "{}", app.status);
        assert_eq!(
            invoke::shown_shortcut("menu.edit.select_all_objects", "Ctrl+A"),
            "Alt+Shift+K"
        );
        assert_eq!(invoke::shown_shortcut("menu.edit.cut", "Ctrl+X"), "Ctrl+X");
        // Ctrl+D (built in for Duplicate) given to Select All: no copy.
        app.settings.shortcuts[0].1 = "Ctrl+D".into();
        app.selection.clear();
        let before = count(&app);
        root_frame(&ctx, &mut app, vec![key(Key::D, Modifiers::COMMAND)]);
        assert_eq!(count(&app), before);
        assert!(!app.selection.is_empty());
        // The built-in Ctrl+A still selects everything.
        app.selection.clear();
        root_frame(&ctx, &mut app, vec![key(Key::A, Modifiers::COMMAND)]);
        assert!(!app.selection.is_empty());
        // A command not available now says so.
        app.settings
            .shortcuts
            .push(("menu.edit.duplicate".into(), "Alt+Shift+J".into()));
        app.selection.clear();
        root_frame(
            &ctx,
            &mut app,
            vec![key(Key::J, Modifiers::ALT | Modifiers::SHIFT)],
        );
        assert_eq!(count(&app), before);
        assert!(
            app.status.contains(&tr("menu.edit.duplicate")),
            "{}",
            app.status
        );
    }

    /// #1409: in the browser version the menus show the key that runs a
    /// command whose own key the browser keeps, and that key runs it; the
    /// desktop shows and runs the command's own key only.
    #[test]
    fn browser_kept_shortcuts_have_alternatives_in_the_menus() {
        use egui::{Key, Modifiers};
        let ctx = crate::theme::ui_context();
        let mut app = mixed_app();
        app.show_welcome = false;
        let first = app.doc().pages[0]
            .layers
            .iter()
            .find_map(|l| l.shapes.first().map(|s| s.id))
            .expect("a shape");
        let shapes = move |app: &App| {
            app.doc().pages[0]
                .layers
                .iter()
                .find(|l| l.shapes.iter().any(|s| s.id == first))
                .map(|l| l.shapes.iter().map(|s| s.id).collect::<Vec<_>>())
                .unwrap_or_default()
        };
        let alt_shift = Modifiers::ALT | Modifiers::SHIFT;
        let back = |app: &mut App| app.selection = vec![first];

        // Desktop: Ctrl+PgUp shown and run; Alt+PgUp does nothing.
        crate::shortcuts::set_web_keys_for_test(false);
        root_frame(&ctx, &mut app, vec![]);
        assert_eq!(
            invoke::shown_shortcut("menu.object.forward_one", "Ctrl+PgUp"),
            "Ctrl+PgUp"
        );
        back(&mut app);
        let before = shapes(&app);
        root_frame(&ctx, &mut app, vec![key(Key::PageUp, Modifiers::ALT)]);
        assert_eq!(shapes(&app), before);

        // Browser: the rows show the alternatives (context menus too, as
        // they draw their rows the same way), and they run the commands.
        crate::shortcuts::set_web_keys_for_test(true);
        for (id, own, web) in [
            ("menu.object.forward_one", "Ctrl+PgUp", "Alt+PgUp"),
            ("menu.object.back_one", "Ctrl+PgDn", "Alt+PgDn"),
            ("menu.file.new", "Ctrl+N", "Alt+Shift+N"),
            ("menu.window.refresh_window", "Ctrl+W", "Ctrl+F5"),
            ("menu.window.close_window", "Ctrl+F4", "Alt+Shift+W"),
            ("menu.text.text_docker", "Ctrl+T", "Alt+Shift+X"),
            ("menu.text.edit_text", "Ctrl+Shift+T", "Alt+Shift+T"),
            ("menu.object.convert_to_curves", "Ctrl+Q", "Alt+Shift+C"),
            ("menu.file.exit", "Alt+F4", "Alt+F4"),
            ("menu.file.save", "Ctrl+S", "Ctrl+S"),
        ] {
            assert_eq!(invoke::shown_shortcut(id, own), web, "{id}");
        }
        let mut entries = Vec::new();
        run_frame(&ctx, |ui| entries = commands(&mut app, ui.ctx()));
        assert!(entries
            .iter()
            .any(|e| e.key == "menu.object.forward_one" && e.shortcut == "Ctrl+PgUp"));
        // Alt+PgUp brings the bottom object one step forward.
        back(&mut app);
        root_frame(&ctx, &mut app, vec![key(Key::PageUp, Modifiers::ALT)]);
        root_frame(&ctx, &mut app, vec![]);
        assert_ne!(shapes(&app), before);
        assert_eq!(shapes(&app)[1], first);
        // Alt+PgDn takes it back.
        root_frame(&ctx, &mut app, vec![key(Key::PageDown, Modifiers::ALT)]);
        root_frame(&ctx, &mut app, vec![]);
        assert_eq!(shapes(&app), before);
        // Alt+Shift+1 (the physical 1, `!` on the keyboard) stores the
        // selection as group 1, which 1 selects again.
        back(&mut app);
        root_frame(
            &ctx,
            &mut app,
            vec![egui::Event::Key {
                key: Key::Exclamationmark,
                physical_key: Some(Key::Num1),
                pressed: true,
                repeat: false,
                modifiers: alt_shift,
            }],
        );
        app.selection.clear();
        root_frame(&ctx, &mut app, vec![key(Key::Num1, Modifiers::NONE)]);
        assert_eq!(app.selection, vec![first]);
        // Alt+Shift+N asks for a new drawing.
        let docs = app.docs.len();
        root_frame(&ctx, &mut app, vec![key(Key::N, alt_shift)]);
        for _ in 0..3 {
            root_frame(&ctx, &mut app, vec![]);
        }
        assert!(
            app.docs.len() > docs || !matches!(app.dialog, Dialog::None),
            "{}",
            app.status
        );
        crate::shortcuts::set_web_keys_for_test(false);
    }

    /// #1409: each browser alternative is a key no menu command, tool or
    /// panel uses, and no key the editor checks by itself.
    #[test]
    fn browser_alternatives_are_free() {
        let ctx = crate::theme::ui_context();
        let mut app = mixed_app();
        app.show_welcome = false;
        let mut entries = Vec::new();
        run_frame(&ctx, |ui| entries = commands(&mut app, ui.ctx()));
        let mut used: Vec<String> = entries.into_iter().map(|e| e.shortcut).collect();
        used.extend(
            crate::tools::Tool::ALL
                .iter()
                .filter_map(|t| t.shortcut())
                .map(str::to_string),
        );
        used.extend(DockerTab::ALL.iter().map(|t| t.shortcut().to_string()));
        // The keys the canvas checks itself: `pressed(Key::X, mods)`.
        let src = include_str!("../interaction.rs");
        for part in src.split("pressed(Key::").skip(1) {
            let Some((k, rest)) = part.split_once(',') else {
                continue;
            };
            let Some((m, _)) = rest.split_once(')') else {
                continue;
            };
            let mut text = String::new();
            if m.contains("cmd") || m.contains("COMMAND") {
                text.push_str("Ctrl+");
            }
            if m.contains("ALT") {
                text.push_str("Alt+");
            }
            if m.contains("SHIFT") {
                text.push_str("Shift+");
            }
            let k = k.trim();
            let k = k.strip_prefix("Num").unwrap_or(k);
            text.push_str(k);
            used.push(text);
        }
        assert!(used
            .iter()
            .any(|u| crate::shortcuts::same(u, "Alt+Shift+R")));
        for (taken, alt) in crate::shortcuts::BROWSER_KEYS {
            let Some(alt) = alt else { continue };
            assert!(
                !used.iter().any(|u| crate::shortcuts::same(u, alt)),
                "{alt} (for {taken}) is used"
            );
        }
    }

    /// A frame of the menu bar alone in a window `height` tall.
    fn bar_frame(ctx: &Context, app: &mut App, height: f32, events: Vec<egui::Event>) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, height),
            )),
            events,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| menu_bar(app, ui));
        out.textures_delta.clear();
    }

    /// The rects of the pop-ups on screen.
    fn popup_rects(ctx: &Context) -> Vec<egui::Rect> {
        ctx.memory(|m| {
            m.areas()
                .visible_layer_ids()
                .into_iter()
                .filter(|l| l.order == egui::Order::Foreground)
                .filter_map(|l| m.area_rect(l.id))
                .collect()
        })
    }

    /// The Object menu opened with Alt and its access key in a window
    /// `height` tall: the rects of the pop-ups shown.
    fn open_object_menu(height: f32) -> Vec<egui::Rect> {
        let ctx = crate::theme::ui_context();
        let mut app = mixed_app();
        let mut taken = Vec::new();
        let mut letter = ' ';
        for key in ["menu.file", "menu.edit", "menu.view", "menu.object"] {
            let c = access::choose(&tr(key), &taken).map(|a| a.1).unwrap_or(' ');
            taken.push(c);
            letter = c;
        }
        let k = access::key_of(letter).expect("object menu access key");
        bar_frame(&ctx, &mut app, height, Vec::new());
        bar_frame(&ctx, &mut app, height, vec![key(k, egui::Modifiers::ALT)]);
        for _ in 0..4 {
            bar_frame(&ctx, &mut app, height, Vec::new());
        }
        popup_rects(&ctx)
    }

    /// A long menu in a short window stays inside the window, under its
    /// title, and scrolls instead of running past the bottom edge (#1008).
    #[test]
    fn a_long_menu_fits_a_short_window() {
        let tall = open_object_menu(1200.0);
        assert_eq!(tall.len(), 1, "{tall:?}");
        let full = tall[0];
        let height = 360.0;
        assert!(full.height() > height, "the Object menu is {full:?}");
        let short = open_object_menu(height);
        assert_eq!(short.len(), 1, "{short:?}");
        let menu = short[0];
        assert!(
            menu.bottom() <= height + 0.5,
            "{menu:?} runs past the window"
        );
        assert!(
            (menu.top() - full.top()).abs() < 0.5,
            "{menu:?} left its title"
        );
        assert!(menu.height() > 3.0 * ROW_HEIGHT, "{menu:?}");
    }

    /// One frame of a menu pop-up with `rows` rows at the top left of a
    /// window `height` tall: the pop-up's rect and the last row's rect.
    fn long_menu_frame(
        ctx: &Context,
        height: f32,
        rows: usize,
        events: Vec<egui::Event>,
    ) -> (egui::Rect, egui::Rect) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, height),
            )),
            events,
            ..Default::default()
        };
        let mut last = egui::Rect::NOTHING;
        let mut area = egui::Rect::NOTHING;
        let mut out = ctx.run_ui(input, |ui| {
            let r = egui::Area::new(egui::Id::new("long menu"))
                .fixed_pos(egui::pos2(10.0, 10.0))
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    ui.set_style(ui.ctx().global_style());
                    menu_popup_style(ui.style_mut());
                    egui::Frame::menu(ui.style()).show(ui, |ui| {
                        body(ui, |ui| {
                            for i in 0..rows {
                                item_label(ui, &format!("Row {i}"), "", true);
                            }
                            last = ui.label("last").rect;
                        })
                    })
                });
            area = r.response.rect;
        });
        out.textures_delta.clear();
        (area, last)
    }

    /// A menu that fits the window shows all its rows, however tall it
    /// is, without a scroll bar's limit.
    #[test]
    fn a_menu_that_fits_shows_every_row() {
        let ctx = crate::theme::ui_context();
        let mut shown = (egui::Rect::NOTHING, egui::Rect::NOTHING);
        for _ in 0..4 {
            shown = long_menu_frame(&ctx, 1000.0, 30, Vec::new());
        }
        let (area, last) = shown;
        assert!(area.height() > 30.0 * ROW_HEIGHT, "{area:?}");
        assert!(last.bottom() <= area.bottom(), "{last:?} in {area:?}");
    }

    /// The wheel scrolls a menu body taller than the window, so its last
    /// rows come into view.
    #[test]
    fn the_wheel_scrolls_a_long_menu_body() {
        let ctx = crate::theme::ui_context();
        let height = 300.0;
        let frame = |events: Vec<egui::Event>| long_menu_frame(&ctx, height, 40, events);
        for _ in 0..3 {
            frame(Vec::new());
        }
        let (area, before) = frame(vec![egui::Event::PointerMoved(egui::pos2(60.0, 100.0))]);
        assert!(area.bottom() <= height + 0.5, "{area:?}");
        assert!(
            before.top() > height,
            "the last row starts in view: {before:?}"
        );
        let wheel = egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, -2000.0),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::NONE,
        };
        frame(vec![wheel]);
        let mut after = before;
        for _ in 0..30 {
            after = frame(Vec::new()).1;
        }
        assert!(
            after.bottom() <= height,
            "the last row stays hidden: {after:?}"
        );
    }

    /// Every enabled row of every menu is clicked once against a mixed
    /// document; the application survives each command.
    #[test]
    fn every_menu_item_runs_without_panicking() {
        let ctx = crate::theme::ui_context();
        let mut hits = 0usize;
        for (name, body) in menus() {
            let mut app = mixed_app();
            let mut target = 0usize;
            loop {
                replay::start(target);
                run_frame(&ctx, |ui| body(&mut app, ui));
                let (hit, rows) = replay::finish();
                if let Some(key) = &hit {
                    hits += 1;
                    // Draw the whole window once so a dialog or docker the
                    // command opened is laid out too.
                    run_frame(&ctx, |ui| crate::ui::root(&mut app, ui));
                    assert!(!app.doc().pages.is_empty(), "{name}: {key} left no page");
                    app.dialog = crate::ui::dialogs::Dialog::None;
                    app.finish_text();
                    if app.doc().pages[0]
                        .layers
                        .iter()
                        .all(|l| l.shapes.is_empty())
                        || app.show_welcome
                    {
                        app = mixed_app();
                    }
                    app.show_welcome = false;
                    app.select_all();
                }
                target += 1;
                if target >= rows {
                    break;
                }
            }
        }
        assert!(hits > 150, "only {hits} menu items were exercised");
    }

    /// What can be seen of the app: the whole window drawn (its texts
    /// and where its shapes are), the menu `body` drawn (its check
    /// marks), the drawing, the settings and the dialog.
    fn visible_state(ctx: &Context, app: &mut App, body: fn(&mut App, &mut Ui)) -> [u64; 7] {
        use std::hash::{Hash, Hasher};
        fn shapes(out: &egui::FullOutput, h: &mut impl Hasher) {
            fn walk(shape: &egui::Shape, h: &mut impl Hasher) {
                match shape {
                    egui::Shape::Text(t) => t.galley.text().hash(h),
                    egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, h)),
                    _ => {}
                }
                let r = shape.visual_bounding_rect();
                if r.is_finite() {
                    for v in [r.min.x, r.min.y, r.max.x, r.max.y] {
                        ((v * 2.0).round() as i64).hash(h);
                    }
                }
            }
            out.shapes.len().hash(h);
            for s in &out.shapes {
                walk(&s.shape, h);
            }
        }
        let new = std::collections::hash_map::DefaultHasher::new;
        let mut parts = [0u64; 7];
        let input = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            ..Default::default()
        };
        // Two frames: the first lays out what the command changed.
        for i in 0..2 {
            let mut out = ctx.run_ui(input(), |ui| crate::ui::root(app, ui));
            if i == 1 {
                let mut h = new();
                shapes(&out, &mut h);
                parts[0] = h.finish();
            }
            out.textures_delta.clear();
        }
        let mut out = ctx.run_ui(input(), |ui| body(app, ui));
        let mut h = new();
        shapes(&out, &mut h);
        parts[1] = h.finish();
        out.textures_delta.clear();
        let one = |s: String| {
            let mut h = new();
            s.hash(&mut h);
            h.finish()
        };
        parts[2] = one(format!("{:?}", app.doc()));
        parts[3] = one(serde_json::to_string(&app.settings).unwrap_or_default());
        parts[4] = one(format!("{:?}", app.dialog));
        // What the window shows of the app that is drawn into the page's
        // picture (a texture, the same shapes either way) or kept in it.
        parts[5] = one(format!(
            "{:?}",
            (
                (app.show_rulers, app.show_grid, app.show_status_bar, app.show_dockers, app.docker_tab),
                (app.show_guides, app.show_page_border, app.show_page_shadow, app.show_bleed),
                (app.show_printable_area, app.show_pixel_grid, app.show_baseline_grid),
                (app.show_non_printing, app.show_standard_toolbar, app.show_property_bar),
                (app.show_toolbox, app.show_text_toolbar, app.show_zoom_toolbar),
                (app.show_transform_toolbar, app.show_left_dockers, app.show_welcome),
                (app.view, app.snap, app.window_layout, app.multipage, app.page_sorter),
                (app.mask_outside_page, app.fullscreen_preview, app.rollover_live, app.tool),
                (&app.selection, app.docs.len(), app.active_doc, app.page, &app.status),
                (app.symmetry_edit, app.clipboard.is_some(), app.show_menu_bar, app.acquire_source),
            )
        ));
        // Pages and files handed to the system (help pages, the browser).
        parts[6] = crate::export::OPENED_IN_TESTS.with(|o| o.borrow().len() as u64);
        parts
    }

    /// Commands that are enabled in the test drawing and rightly change
    /// nothing there, with why: the state they set is the one it has, or
    /// the drawing has nothing for them. Kept short and explained: a row
    /// that never does anything does not belong here.
    const NOTHING_TO_CHANGE: &[(&str, &str)] = &[
        ("menu.file.acquire_wia", "WIA is already the chosen source"),
        ("menu.edit.paste", "the test clipboard is empty"),
        ("menu.edit.select_all_objects", "everything is selected already"),
        ("menu.view.enhanced", "Enhanced is already the view"),
        ("menu.layout.page_number_even", "a drawing of one page has no even page"),
        ("menu.layout.page_spacing_30", "30 mm is already the spacing"),
        ("menu.object.clear_transformations", "nothing in the drawing is transformed"),
        ("menu.object.unlock_all", "nothing in the drawing is locked"),
        ("menu.text.straighten_text", "the text has no shifted or turned characters"),
        ("menu.text.align_to_baseline", "the text has no shifted characters"),
        ("menu.window.refresh_window", "it draws again what is drawn"),
        ("menu.window.workspace_default", "Default is already the workspace"),
        ("bars.size_small", "Small is already the button size"),
        ("bars.reset", "the bars are where they start"),
        ("palette.default_cmyk", "the only palette shown; Window > Toolbars > Color Palette hides it"),
    ];

    /// Every enabled menu command changes something that can be seen:
    /// the window, the menu's own marks, the drawing, the settings or a
    /// dialog. A command that changes nothing at all is a row that only
    /// looks like it works.
    #[test]
    fn every_menu_item_has_a_visible_effect() {
        let ctx = crate::theme::ui_context();
        let mut silent = Vec::new();
        let mut measured = 0usize;
        for (name, body) in menus() {
            let mut app = mixed_app();
            app.select_all();
            // Parts that change between two drawings with nothing done
            // (an animation) are left out of this menu's comparison.
            let a = visible_state(&ctx, &mut app, body);
            let b = visible_state(&ctx, &mut app, body);
            let steady: Vec<usize> = (0..7).filter(|i| a[*i] == b[*i]).collect();
            if steady.len() < 7 {
                println!("{name}: unsteady parts {:?}", (0..7).filter(|i| a[*i] != b[*i]).collect::<Vec<_>>());
            }
            assert!(steady.contains(&2) && steady.contains(&3), "{name}: drawing or settings unsteady");
            let same = |x: &[u64; 7], y: &[u64; 7]| steady.iter().all(|i| x[*i] == y[*i]);
            let mut target = 0usize;
            loop {
                let mut app = mixed_app();
                app.select_all();
                let before = visible_state(&ctx, &mut app, body);
                replay::start(target);
                run_frame(&ctx, |ui| body(&mut app, ui));
                let (hit, rows) = replay::finish();
                if let Some(key) = &hit {
                    measured += 1;
                    if same(&visible_state(&ctx, &mut app, body), &before)
                        && !NOTHING_TO_CHANGE.iter().any(|(k, _)| k == key)
                    {
                        silent.push(format!("{name}: {key}"));
                    }
                }
                target += 1;
                if target >= rows {
                    break;
                }
            }
        }
        println!("{measured} commands measured, {} silent", silent.len());
        for s in &silent {
            println!("SILENT {s}");
        }
        assert!(measured > 150, "only {measured} menu items were measured");
        assert!(silent.is_empty(), "{} commands without a visible effect", silent.len());
    }
}

/// Menu bar titles: size and spacing (about 25 px between titles, the
/// first one 11 px from the edge).
const BAR_FONT: f32 = 13.0;

/// Where the menu bar's titles were drawn, and how many menus are under
/// its More title (`(Vec<Rect>, usize)` in the context's data).
pub(crate) const MENU_BAR_LAYOUT: &str = "menu_bar_layout";

/// The menu under More whose submenu opens once More is shown (Alt and
/// its letter).
const MORE_PENDING: &str = "menu_bar_more_pending";

/// The access keys of the menus under More, in their order (for tests).
pub(crate) const MENU_MORE_KEYS: &str = "menu_bar_more_keys";

/// Room kept at the end of a menu bar too short for every title, for the
/// "More" title (»).
const MORE_W: f32 = 28.0;

/// A menu title's width on the bar before it was drawn once.
fn title_width(ui: &Ui, label: &str) -> f32 {
    let text = ui.fonts_mut(|f| {
        f.layout_no_wrap(
            label.to_string(),
            egui::FontId::proportional(BAR_FONT),
            Tokens::text(),
        )
        .size()
        .x
    });
    text + 2.0 * ui.spacing().button_padding.x
}

fn menu_bar_style(style: &mut egui::Style) {
    egui::containers::menu::menu_style(style);
    style.override_font_id = Some(egui::FontId::proportional(BAR_FONT));
    // An open or hovered title: light blue with a blue frame, square.
    for w in [
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        w.weak_bg_fill = crate::theme::Tokens::menu_hover();
        w.bg_fill = crate::theme::Tokens::menu_hover();
        w.bg_stroke = egui::Stroke::new(1.0, crate::theme::Tokens::menu_hover_line());
        w.corner_radius = egui::CornerRadius::ZERO;
        w.expansion = 0.0;
        w.fg_stroke.color = crate::theme::on_highlight();
    }
    style.visuals.widgets.inactive.fg_stroke.color = crate::theme::Tokens::text();
    style.spacing.button_padding = egui::vec2(10.0, 3.0);
    style.spacing.item_spacing.x = 3.0;
}

pub fn menu_bar(app: &mut App, ui: &mut Ui) {
    set_highlight_new(app.highlight_whats_new);
    // Alt and a letter the user gave a command keeps running that command
    // instead of opening a menu; while Options waits for a shortcut's keys
    // no menu opens either.
    let capturing = app.shortcut_editor.capture.is_some();
    let user_alt_keys: Vec<egui::Key> = app
        .settings
        .shortcuts
        .iter()
        .filter_map(|(_, v)| crate::shortcuts::parse(v))
        .filter(|(m, _)| m.alt && !(m.ctrl || m.command || m.shift))
        .map(|(_, k)| k)
        .collect();
    let blocked =
        move |c: char| capturing || access::key_of(c).is_some_and(|k| user_alt_keys.contains(&k));
    egui::MenuBar::new()
        .style(menu_bar_style)
        .config(egui::containers::menu::MenuConfig::new().style(menu_popup_style))
        .ui(ui, |ui| {
            // The title the keyboard is on (Alt pressed alone, #1839).
            let bar_focus = crate::ui::keynav::bar_focus(ui.ctx());
            // A menu opened from the keyboard lights its title as a click
            // does.
            let open_title = crate::ui::keynav::open_title(ui.ctx());
            // Each title's access key, for Alt and the letter.
            let mut keys: Vec<char> = Vec::new();
            let mut menu = |ui: &mut Ui, key: &str, f: &mut dyn FnMut(&mut Ui)| -> egui::Response {
                let label = tr(key);
                let access = access::choose(&label, &keys);
                let lit = bar_focus == Some(keys.len()) || open_title == Some(keys.len());
                keys.push(access.map(|a| a.1).unwrap_or(' '));
                let title_bottom_id = ui.id().with(key).with("title_bottom");
                let title_bottom = ui.ctx().data(|d| d.get_temp::<f32>(title_bottom_id));
                // Lit like a title under the pointer.
                let inactive = ui.style().visuals.widgets.inactive;
                if lit {
                    ui.style_mut().visuals.widgets.inactive = ui.style().visuals.widgets.hovered;
                }
                let r = ui
                    .menu_button(
                        access::title(
                            &label,
                            access.map(|a| a.0),
                            BAR_FONT,
                            crate::theme::Tokens::text(),
                        ),
                        |ui| {
                            ui.set_min_width(MENU_WIDTH);
                            let limit = below_body_limit(ui, title_bottom);
                            body_within(ui, limit, |ui| f(ui))
                        },
                    )
                    .response;
                ui.style_mut().visuals.widgets.inactive = inactive;
                ui.ctx()
                    .data_mut(|d| d.insert_temp(title_bottom_id, r.rect.bottom()));
                if marked_new(key) {
                    // A dot over the title's top right corner.
                    ui.painter().circle_filled(
                        egui::pos2(r.rect.right() - 3.0, r.rect.top() + 4.0),
                        3.0,
                        Tokens::ACCENT,
                    );
                }
                r
            };
            // The menus in their order; with no drawing open only File,
            // Tools, Window and Help remain.
            let has_document = app.has_document();
            let mut titles = Vec::new();
            // The titles that do not fit the bar (a phone, a tablet in
            // the Touch workspace, a narrow window) go under the "More"
            // title at its end (#1301, #1691).
            let menus: Vec<(&str, fn(&mut App, &mut Ui))> = all_menus()
                .into_iter()
                .filter(|(key, _)| has_document || MENUS_WITHOUT_DRAWING.contains(key))
                .collect();
            let gap = ui.spacing().item_spacing.x;
            let width_id = |ui: &Ui, key: &str| ui.id().with(key).with("title_width");
            let widths: Vec<f32> = menus
                .iter()
                .map(|(key, _)| {
                    let id = width_id(ui, key);
                    ui.ctx()
                        .data(|d| d.get_temp::<f32>(id))
                        .unwrap_or_else(|| title_width(ui, &tr(key)))
                })
                .collect();
            let all = widths.iter().map(|w| w + gap).sum::<f32>();
            let room = if all > ui.available_width() {
                // The last visible title also leaves a gap before More.
                // Reserve it along with the button instead of letting
                // that final spacing push More past a narrow window.
                ui.available_width() - MORE_W - gap
            } else {
                f32::INFINITY
            };
            let mut used = 0.0;
            let mut more: Vec<(&str, fn(&mut App, &mut Ui))> = Vec::new();
            for ((key, body), width) in menus.into_iter().zip(widths) {
                if !more.is_empty() || used + width > room {
                    more.push((key, body));
                    continue;
                }
                let r = menu(ui, key, &mut |ui| body(app, ui));
                let id = width_id(ui, key);
                ui.ctx().data_mut(|d| d.insert_temp(id, r.rect.width()));
                used += r.rect.width() + gap;
                titles.push(r);
            }
            // The menus under More keep the access keys they have on a
            // wide bar: Alt and the letter open More and that menu in it.
            let bar_keys = keys.clone();
            let mut more_keys: Vec<char> = Vec::new();
            for (key, _) in &more {
                let mut taken = bar_keys.clone();
                taken.extend(&more_keys);
                more_keys.push(access::choose(&tr(key), &taken).map_or(' ', |a| a.1));
            }
            let mut more_title = None;
            if !more.is_empty() {
                let pending_id = egui::Id::new(MORE_PENDING);
                let r = ui
                    .menu_button(egui::RichText::new("\u{bb}").size(BAR_FONT), |ui| {
                        ui.set_min_width(MENU_WIDTH);
                        body(ui, |ui| {
                            access::reserve(&bar_keys);
                            let pending = ui.ctx().data(|d| d.get_temp::<usize>(pending_id));
                            for (i, (key, f)) in more.iter().enumerate() {
                                let row = sub_row(ui, key, |ui| f(app, ui));
                                if let Some(row) = row.filter(|_| pending == Some(i)) {
                                    open_row_submenu(ui, row.id);
                                    ui.ctx().data_mut(|d| d.remove::<usize>(pending_id));
                                }
                            }
                        })
                    })
                    .response;
                keys.push(' ');
                more_title = Some(r.clone());
                titles.push(r);
            }
            // Where the titles went and how many are under More, for the
            // layout checks.
            let shown: Vec<egui::Rect> = titles.iter().map(|r| r.rect).collect();
            ui.ctx().data_mut(|d| {
                d.insert_temp(egui::Id::new(MENU_BAR_LAYOUT), (shown, more.len()));
                d.insert_temp(egui::Id::new(MENU_MORE_KEYS), more_keys.clone());
            });
            switch_on_hover(ui.ctx(), &titles);
            open_with_alt(ui.ctx(), &titles, &keys, &blocked);
            if let Some(more_title) = &more_title {
                open_more_with_alt(ui.ctx(), &titles, more_title, &more_keys, &blocked);
            }
            // The titles for the keys of the next frame: Left and Right
            // go from menu to menu (#1839).
            crate::ui::keynav::note_titles(
                ui.ctx(),
                crate::ui::keynav::Titles {
                    popups: titles
                        .iter()
                        .map(egui::Popup::default_response_id)
                        .collect(),
                    keys: keys.clone(),
                },
            );
        });
}

/// Alt+letter used by commands (symmetry, Alt+Q, Alt+Z): they keep it
/// over opening a menu.
const ALT_TAKEN: [char; 5] = ['e', 's', 'x', 'q', 'z'];

/// Alt and a title's access key open that menu (closing another one),
/// unless `blocked` keeps that letter for something else.
fn open_with_alt(
    ctx: &egui::Context,
    titles: &[egui::Response],
    keys: &[char],
    blocked: &dyn Fn(char) -> bool,
) {
    let popup_id = egui::Popup::default_response_id;
    let hit = keys
        .iter()
        .zip(titles)
        .find(|(c, _)| alt_pressed(ctx, **c, blocked));
    if let Some((_, r)) = hit {
        for t in titles {
            egui::Popup::close_id(ctx, popup_id(t));
        }
        egui::Popup::open_id(ctx, popup_id(r));
        // Its first row lit, for the arrows and Enter (#1839).
        crate::ui::keynav::opened_by_keys(ctx, popup_id(r));
    }
}

/// Alt and the access key of a menu under More (») open More with that
/// menu's submenu open, its first row lit, as the title would on a
/// wide bar.
fn open_more_with_alt(
    ctx: &egui::Context,
    titles: &[egui::Response],
    more_title: &egui::Response,
    more_keys: &[char],
    blocked: &dyn Fn(char) -> bool,
) {
    let Some(at) = more_keys.iter().position(|c| alt_pressed(ctx, *c, blocked)) else {
        return;
    };
    let popup_id = egui::Popup::default_response_id;
    for t in titles {
        egui::Popup::close_id(ctx, popup_id(t));
    }
    egui::Popup::open_id(ctx, popup_id(more_title));
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(MORE_PENDING), at));
    ctx.request_repaint();
}

/// Alt and the access key `c` were pressed (without Ctrl or Shift) and
/// nothing else keeps that letter.
fn alt_pressed(ctx: &egui::Context, c: char, blocked: &dyn Fn(char) -> bool) -> bool {
    !ALT_TAKEN.contains(&c)
        && !blocked(c)
        && access::key_of(c).is_some_and(|key| {
            ctx.input(|i| {
                i.events.iter().any(|e| {
                    matches!(e, egui::Event::Key { key: k, pressed: true, modifiers, .. }
                        if *k == key && modifiers.alt && !modifiers.ctrl && !modifiers.shift)
                })
            })
        })
}

/// Open the submenu of the row `row` of the menu being drawn (next
/// frame), its first row lit.
fn open_row_submenu(ui: &Ui, row: egui::Id) {
    let menu = egui::containers::menu::find_menu_root(ui).id;
    crate::ui::keynav::open_submenu_by_keys(ui.ctx(), menu, row);
}

/// With one menu open, pointing at another title of the bar opens that
/// menu instead, as desktop menu bars do.
fn switch_on_hover(ctx: &egui::Context, titles: &[egui::Response]) {
    let popup_id = egui::Popup::default_response_id;
    let Some(open) = titles
        .iter()
        .position(|r| egui::Popup::is_id_open(ctx, popup_id(r)))
    else {
        return;
    };
    if let Some(hovered) = titles.iter().position(|r| r.hovered()) {
        if hovered != open {
            egui::Popup::close_id(ctx, popup_id(&titles[open]));
            egui::Popup::open_id(ctx, popup_id(&titles[hovered]));
        }
    }
}

fn file_menu(app: &mut App, ui: &mut Ui) {
    // Commands that work on a drawing are off on the Welcome Screen alone.
    let doc = app.has_document();
    if item(ui, "menu.file.new", "Ctrl+N", true) {
        app.request_new_document();
    }
    if item(ui, "menu.file.new_from_template", "", true) {
        app.open_template_dialog();
    }
    if item(ui, "menu.file.open", "Ctrl+O", true) {
        app.open_dialog();
    }
    let recent = app.settings.recent_files.clone();
    sub(ui, "menu.file.open_recent", |ui| {
        if recent.is_empty() {
            ui.add_enabled(
                false,
                egui::Button::new(tr("menu.file.no_recent")).frame(false),
            );
        }
        for p in recent.iter().take(10) {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if menu_row(ui, &name, "", true, Mark::None)
                .0
                .rich_tip(p.display().to_string())
                .clicked()
            {
                ui.close();
                app.open_path(p.clone());
            }
        }
        if !recent.is_empty() {
            sep(ui);
            if item(ui, "menu.file.clear_recent", "", true) {
                app.settings.recent_files.clear();
                crate::thumbnails::clear();
                app.recent_thumbs.clear();
                app.settings.save();
            }
        }
    });
    sub(ui, "menu.file.github", |ui| {
        if item(ui, "menu.file.github_open", "", true) {
            app.github.status.clear();
            app.dialog = Dialog::GithubOpen(app.github_form(false));
        }
        if item(ui, "menu.file.github_save", "", doc) {
            app.github.status.clear();
            app.dialog = Dialog::GithubSave(app.github_form(true));
        }
        if item(ui, "menu.file.github_history", "", app.remote.is_some()) {
            app.github.status.clear();
            app.github.history = None;
            app.dialog = Dialog::GithubHistory;
        }
    });
    sep(ui);
    if item(ui, "menu.file.close", "", doc) {
        app.close_document();
    }
    if item(ui, "menu.file.close_all", "", doc) {
        app.close_all_documents();
    }
    sep(ui);
    if item(ui, "menu.file.save", "Ctrl+S", doc) {
        app.save(false);
    }
    if item(ui, "menu.file.save_as", "Ctrl+Shift+S", doc) {
        app.save(true);
    }
    if item(ui, "menu.file.save_as_template", "", doc) {
        app.save_as_template();
    }
    if item(
        ui,
        "menu.file.revert",
        "",
        app.file.is_some() && app.engine.is_dirty(),
    ) {
        if let Some(p) = app.file.clone() {
            // Read again (not the open drawing shown, see `open_path`).
            app.open_path_with_password(p, "");
        }
    }
    sep(ui);
    sub(ui, "menu.file.acquire_image", |ui| {
        use crate::acquire::Source;
        // TWAIN: offered once acquiring through it is done.
        if check_if(
            ui,
            "menu.file.acquire_twain",
            "",
            app.acquire_source == Source::Twain,
            crate::acquire::twain_available(),
        ) {
            app.set_acquire_source(Source::Twain);
        }
        if check_if(
            ui,
            "menu.file.acquire_wia",
            "",
            app.acquire_source == Source::Wia,
            crate::acquire::wia_available(),
        ) {
            app.set_acquire_source(Source::Wia);
        }
        sep(ui);
        if item(ui, "menu.file.acquire", "", app.can_acquire()) {
            app.acquire_image();
        }
    });
    if item(ui, "menu.file.import", "Ctrl+I", doc) {
        app.import();
    }
    if item(ui, "menu.file.export", "Ctrl+E", doc) {
        app.export();
    }
    sub(ui, "menu.file.export_for", |ui| {
        if item(ui, "menu.file.export_for_office", "", doc) {
            app.dialog = Dialog::Export(crate::ui::dialogs::ExportState::office());
        }
        if item(ui, "menu.file.export_for_web", "", doc) {
            app.dialog = Dialog::Export(crate::ui::dialogs::ExportState::web());
        }
    });
    sub(ui, "menu.file.send_to", |ui| {
        use crate::export::SendTarget;
        for (key, t) in [
            ("menu.file.send_to_desktop", SendTarget::Desktop),
            ("menu.file.send_to_documents", SendTarget::Documents),
            ("menu.file.send_to_mail", SendTarget::Mail),
        ] {
            // Greyed with the reason where the platform has no such place
            // (Android has no desktop folder, #1398).
            let why = t.why_not().map(tr);
            if item_tip(ui, key, "", doc && why.is_none(), why.as_deref()) {
                app.send_to(t);
            }
        }
    });
    if item(ui, "menu.file.publish_to_pdf", "", doc) {
        app.publish_pdf();
    }
    // Folders are a desktop thing (#852).
    if item(ui, "menu.file.package", "", doc && !crate::files::WEB) {
        app.package_dialog();
    }
    sep(ui);
    sub(ui, "menu.file.print_merge", |ui| {
        if item(ui, "menu.file.print_merge_create", "", doc) {
            app.dialog = Dialog::PrintMerge(Default::default());
        }
        let loaded = app.merge_state.is_some();
        if item(ui, "menu.file.print_merge_edit", "", loaded) {
            if let Some(st) = app.merge_state.clone() {
                app.dialog = Dialog::PrintMerge(st);
            }
        }
        if item(ui, "menu.file.print_merge_perform", "", loaded) {
            if let Some(st) = app.merge_state.clone() {
                crate::print_merge::perform(app, &st);
            }
        }
        if item(ui, "menu.file.print_merge_new_document", "", loaded) {
            app.create_merged_document(false);
        }
    });
    sep(ui);
    if item(ui, "menu.file.print", "Ctrl+P", doc) {
        app.print();
    }
    if item(ui, "menu.file.print_preview", "", doc) {
        app.print_preview(None);
    }
    if item(ui, "menu.file.preflight", "", doc) {
        app.dialog = Dialog::Preflight(Default::default());
    }
    sep(ui);
    if item(ui, "menu.file.document_properties", "", doc) {
        app.dialog = Dialog::DocumentProperties(Box::default());
    }
    sep(ui);
    if item(ui, "menu.file.exit", "Alt+F4", true) {
        app.request_exit();
    }
}

fn edit_menu(app: &mut App, ui: &mut Ui) {
    let has = !app.selection.is_empty();
    let undo = app
        .engine
        .undo_label()
        .map(|l| format!("{} {l}", tr("menu.edit.undo")))
        .unwrap_or_else(|| tr("menu.edit.undo"));
    let redo = app
        .engine
        .redo_label()
        .map(|l| format!("{} {l}", tr("menu.edit.redo")))
        .unwrap_or_else(|| tr("menu.edit.redo"));
    if item_text(
        ui,
        &undo,
        "Ctrl+Z",
        app.engine.undo_label().is_some(),
        Some(crate::ui::icons::Action::Undo),
        "menu.edit.undo",
    ) {
        app.undo();
    }
    if item_text(
        ui,
        &redo,
        "Ctrl+Shift+Z",
        app.engine.redo_label().is_some(),
        Some(crate::ui::icons::Action::Redo),
        "menu.edit.redo",
    ) {
        app.redo();
    }
    if item(
        ui,
        "menu.edit.repeat",
        "Ctrl+R",
        app.last_repeatable().is_some(),
    ) {
        app.repeat_last();
    }
    sep(ui);
    // Focus mode, as the reference's Edit menu has it: one object (a
    // group most of all) edited on its own, and the way back out.
    let one = app.selection.as_slice();
    let focusable = one.len() == 1 && app.focus_target() != Some(one[0]);
    if item(ui, "menu.edit.bring_into_focus", "", focusable) {
        app.bring_into_focus();
    }
    if item(ui, "menu.edit.exit_focus_mode", "", app.focus_target().is_some()) {
        app.exit_focus();
    }
    sep(ui);
    // Guidelines selected (no objects) are cut and copied too.
    let guides_picked = !app.selected_guides.is_empty();
    // While a text is being edited they act on its selected characters
    // (#1377).
    let can_copy = if app.editing_text() {
        app.text_selection_active()
    } else {
        has || guides_picked
    };
    if item(ui, "menu.edit.cut", "Ctrl+X", can_copy) {
        app.cut();
    }
    if item(ui, "menu.edit.copy", "Ctrl+C", can_copy) {
        app.copy_with_system();
    }
    if item(ui, "menu.edit.copy_properties_from", "Ctrl+Shift+A", has) {
        app.open_copy_properties();
    }
    if item(ui, "menu.edit.paste", "Ctrl+V", true) {
        app.paste_any();
    }
    if item(
        ui,
        "menu.edit.paste_in_view",
        "Ctrl+Shift+V",
        app.clipboard.is_some(),
    ) {
        app.paste_in_view();
    }
    if item(ui, "menu.edit.paste_special", "", true) {
        app.dialog = Dialog::PasteSpecial(Default::default());
    }
    if app.editing_text() {
        // The selected characters, as the Delete key does.
        if item(
            ui,
            "menu.edit.delete",
            "Delete",
            app.text_selection_active(),
        ) {
            app.text_delete_forward();
        }
    } else if item(ui, "menu.edit.delete", "Delete", has || app.nodes_chosen()) {
        // With the Shape tool and nodes chosen: the nodes, not the
        // object they belong to (#1982).
        app.delete_nodes_or_selection();
    }
    sep(ui);
    // Guidelines selected (no objects) duplicate too.
    if item(ui, "menu.edit.duplicate", "Ctrl+D", has || guides_picked) {
        app.duplicate();
    }
    if item(ui, "menu.edit.clone", "", app.can_clone()) {
        app.clone_selection();
    }
    sep(ui);
    sub(ui, "menu.edit.select_all", |ui| {
        if item(ui, "menu.edit.select_all_objects", "Ctrl+A", true) {
            app.select_all();
        }
        if item(ui, "menu.edit.select_all_text", "", true) {
            app.select_all_of(|s| matches!(s.kind, ShapeKind::Text { .. }));
        }
        if item(ui, "menu.edit.select_all_guidelines", "", true) {
            app.select_all_guides();
        }
        if item(ui, "menu.edit.select_all_nodes", "", has) {
            app.select_all_nodes();
        }
    });
    sep(ui);
    if item(ui, "menu.edit.find_and_replace", "Ctrl+F", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::FindReplace;
    }
    if item(ui, "menu.edit.step_and_repeat", "Ctrl+Shift+D", has) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::StepAndRepeat;
    }
}

fn view_menu(app: &mut App, ui: &mut Ui) {
    for (key, mode) in [
        ("menu.view.wireframe", crate::app::ViewMode::Wireframe),
        ("menu.view.normal", crate::app::ViewMode::Normal),
        ("menu.view.enhanced", crate::app::ViewMode::Enhanced),
        ("menu.view.pixels", crate::app::ViewMode::Pixels),
    ] {
        if radio(ui, key, "", app.view_mode == mode) {
            app.view_mode = mode;
            app.wireframe = mode == crate::app::ViewMode::Wireframe;
        }
    }
    // Right after the view modes, where the report asks for it (#615).
    if check(ui, "menu.view.mask_outside_page", "", app.mask_outside_page) {
        app.mask_outside_page = !app.mask_outside_page;
    }
    sep(ui);
    if item(ui, "menu.view.fullscreen_preview", "F9", true) {
        app.fullscreen_preview = true;
    }
    if item(
        ui,
        "menu.view.preview_selected_only",
        "",
        !app.selection.is_empty(),
    ) {
        app.preview_selected_only = true;
        app.fullscreen_preview = true;
    }
    if check(ui, "menu.view.page_sorter", "", app.page_sorter) {
        app.page_sorter = !app.page_sorter;
        if app.page_sorter {
            app.multipage = false;
        }
    }
    if check(ui, "menu.view.multipage", "", app.multipage) {
        app.set_multipage(!app.multipage);
    }
    sep(ui);
    if item(ui, "menu.view.zoom_in", "Ctrl++", true) {
        app.zoom_step(true);
    }
    if item(ui, "menu.view.zoom_out", "Ctrl+-", true) {
        app.zoom_step(false);
    }
    if item(ui, "menu.view.zoom_to_fit", "F4", true) {
        app.zoom_to_fit();
    }
    sep(ui);
    if check(ui, "menu.view.proof_colors", "", app.proof_colors) {
        app.proof_colors = !app.proof_colors;
    }
    if check(
        ui,
        "menu.view.simulate_overprints",
        "",
        app.simulate_overprints,
    ) {
        app.simulate_overprints = !app.simulate_overprints;
        app.raster.borrow_mut().invalidate();
    }
    if check(
        ui,
        "menu.view.rasterize_complex_effects",
        "",
        app.rasterize_complex_effects,
    ) {
        app.rasterize_complex_effects = !app.rasterize_complex_effects;
        app.raster.borrow_mut().invalidate();
    }
    sep(ui);
    sub(ui, "menu.view.page", |ui| {
        if check(ui, "menu.view.page_border", "", app.show_page_border) {
            app.show_page_border = !app.show_page_border;
        }
        if check(ui, "menu.view.page_shadow", "", app.show_page_shadow) {
            app.show_page_shadow = !app.show_page_shadow;
        }
        if check(ui, "menu.view.bleed", "", app.show_bleed) {
            app.show_bleed = !app.show_bleed;
        }
        if check(ui, "menu.view.printable_area", "", app.show_printable_area) {
            app.show_printable_area = !app.show_printable_area;
        }
    });
    sub(ui, "menu.view.grid", |ui| {
        if check(ui, "menu.view.document_grid", "", app.show_grid) {
            app.show_grid = !app.show_grid;
        }
        if check(ui, "menu.view.pixel_grid", "", app.show_pixel_grid) {
            app.show_pixel_grid = !app.show_pixel_grid;
        }
        if check(ui, "menu.view.baseline_grid", "", app.show_baseline_grid) {
            app.show_baseline_grid = !app.show_baseline_grid;
        }
    });
    if check(ui, "menu.view.rulers", "Alt+Shift+R", app.show_rulers) {
        app.show_rulers = !app.show_rulers;
    }
    if check(ui, "menu.view.scrollbars", "", app.settings.show_scrollbars) {
        app.settings.show_scrollbars = !app.settings.show_scrollbars;
        app.settings.save();
    }
    if check(ui, "menu.view.guidelines", "", app.show_guides) {
        app.show_guides = !app.show_guides;
    }
    if check(
        ui,
        "menu.view.alignment_guides",
        "Alt+Shift+A",
        app.snap.alignment_guides,
    ) {
        app.snap.alignment_guides = !app.snap.alignment_guides;
    }
    if check(
        ui,
        "menu.view.dynamic_guides",
        "Alt+Shift+D",
        app.snap.dynamic_guides,
    ) {
        app.snap.dynamic_guides = !app.snap.dynamic_guides;
    }
    sep(ui);
    sub(ui, "menu.view.snap_to", |ui| {
        if check(ui, "menu.view.snap_pixels", "", app.snap.pixels) {
            app.snap.pixels = !app.snap.pixels;
        }
        if check(ui, "menu.view.snap_document_grid", "Ctrl+Y", app.snap.grid) {
            app.snap.grid = !app.snap.grid;
        }
        if check(
            ui,
            "menu.view.snap_baseline_grid",
            "",
            app.snap.baseline_grid,
        ) {
            app.snap.baseline_grid = !app.snap.baseline_grid;
        }
        if check(ui, "menu.view.snap_guidelines", "", app.snap.guides) {
            app.snap.guides = !app.snap.guides;
        }
        if check(ui, "menu.view.snap_objects", "Alt+Z", app.snap.objects) {
            app.snap.objects = !app.snap.objects;
        }
        if check(ui, "menu.view.snap_page", "", app.snap.page) {
            app.snap.page = !app.snap.page;
        }
    });
    if check(ui, "menu.view.snap_off", "Alt+Q", app.snap.off) {
        app.snap.off = !app.snap.off;
    }
}

pub(crate) fn layout_menu(app: &mut App, ui: &mut Ui) {
    if item(ui, "menu.layout.insert_page", "", true) {
        app.dialog = Dialog::InsertPage {
            count: 1,
            after: true,
        };
    }
    if item(ui, "menu.layout.duplicate_page", "", true) {
        app.dialog = Dialog::duplicate_page();
    }
    if item(ui, "menu.layout.rename_page", "", true) {
        let name = app
            .doc()
            .page(app.page)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        app.dialog = Dialog::RenamePage { name };
    }
    if item(ui, "menu.layout.delete_page", "", app.doc().pages.len() > 1) {
        app.ask_delete_page();
    }
    if item(ui, "menu.layout.go_to_page", "", true) {
        app.dialog = Dialog::GoToPage {
            page: app.page_index() + 1,
        };
    }
    sep(ui);
    sub(ui, "menu.layout.insert_page_number", |ui| {
        for (key, where_) in [
            (
                "menu.layout.page_number_active",
                crate::ops2::PageNumberWhere::Active,
            ),
            (
                "menu.layout.page_number_all",
                crate::ops2::PageNumberWhere::All,
            ),
            (
                "menu.layout.page_number_odd",
                crate::ops2::PageNumberWhere::Odd,
            ),
            (
                "menu.layout.page_number_even",
                crate::ops2::PageNumberWhere::Even,
            ),
        ] {
            if key == "menu.layout.page_number_all" {
                sep(ui);
            }
            if item(ui, key, "", true) {
                app.insert_page_number(where_);
            }
        }
    });
    if item(ui, "menu.layout.page_number_settings", "", true) {
        app.dialog = Dialog::PageNumberSettings;
    }
    sep(ui);
    if item(ui, "menu.layout.switch_orientation", "", true) {
        let s = app.page_size();
        let page = app.page;
        app.run(Command::ResizePage {
            page,
            size: tracedraw_core::geometry::Size::new(s.height, s.width),
        });
        app.fit_pending = true;
    }
    sep(ui);
    if item(ui, "menu.layout.document_options", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::DocumentGeneral;
    }
    if item(ui, "menu.layout.page_size", "", true) {
        let s = app.page_size();
        app.dialog = Dialog::PageSize {
            width: s.width,
            height: s.height,
            all_pages: true,
        };
    }
    if item(ui, "menu.layout.page_layout", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::Layout;
    }
    // The space between the pages of the Multipage View.
    sub(ui, "menu.layout.page_spacing", |ui| {
        let now = app.page_spacing();
        for (mm, key) in crate::multipage::SPACING_PRESETS {
            if radio(ui, key, "", (now - mm).abs() < 1e-9) {
                app.set_page_spacing(mm);
            }
        }
        sep(ui);
        if item(ui, "menu.layout.page_spacing_custom", "", true) {
            app.dialog = Dialog::PageSpacing {
                value: app.units.from_mm(now),
            };
        }
    });
    // Facing pages: the pages in spreads, the first page on the right
    // or on the left (#1972, #1660).
    {
        use tracedraw_core::spreads::PageSide;
        let (on, start) = {
            let m = &app.doc().metadata;
            (m.facing_pages, m.facing_start)
        };
        if check(ui, "menu.layout.facing_pages", "", on) {
            app.set_facing_pages(!on, start);
        }
        sub(ui, "menu.layout.facing_start", |ui| {
            for (side, key) in [
                (PageSide::Right, "menu.layout.facing_start_right"),
                (PageSide::Left, "menu.layout.facing_start_left"),
            ] {
                if radio(ui, key, "", on && start == side) {
                    app.set_facing_pages(true, side);
                }
            }
        });
    }
    if item(
        ui,
        "menu.layout.fit_page_selection",
        "",
        !app.selection.is_empty(),
    ) {
        app.fit_page_to_selection();
    }
    if item(
        ui,
        "menu.layout.fit_page_drawing",
        "",
        app.drawing_bounds().is_some(),
    ) {
        app.fit_page_to_drawing();
    }
    if item(ui, "menu.layout.page_background", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::Background;
    }
}

fn object_menu(app: &mut App, ui: &mut Ui) {
    let has = !app.selection.is_empty();
    let many = app.selection.len() > 1;
    sub(ui, "menu.object.create", |ui| {
        if item(
            ui,
            "menu.object.create_arrowhead",
            "",
            app.selection.len() == 1,
        ) {
            app.create_arrowhead_from_selection();
        }
        // Pattern Fill asks whether the pattern is vector or bitmap.
        if item(
            ui,
            "menu.object.create_pattern",
            "",
            !app.selection.is_empty(),
        ) {
            app.dialog = Dialog::CreatePattern { vector: true };
        }
    });
    sub(ui, "menu.object.insert", |ui| {
        if item(ui, "menu.object.insert_qr", "", true) {
            app.insert_qr_code();
            // Its settings are edited in Properties.
            app.show_dockers = true;
            app.docker_tab = DockerTab::Properties;
        }
        if item(ui, "menu.object.insert_barcode", "", true) {
            app.dialog = Dialog::Barcode(Default::default());
        }
        let can_read = app.can_identify_barcode();
        if item(ui, "menu.object.identify_barcode", "", can_read) {
            let ctx = ui.ctx().clone();
            app.identify_barcode(Some(&ctx));
        }
        if item(ui, "menu.object.insert_page_number", "", true) {
            app.insert_page_number(crate::ops2::PageNumberWhere::Active);
        }
    });
    // AI Generate: the docker in each mode, and Remove Background.
    sub(ui, "menu.object.ai_generate", |ui| {
        let doc = app.has_document();
        for (key, mode) in [
            ("menu.object.ai_create_image", crate::ai_image::Mode::Create),
            ("menu.object.ai_remix_image", crate::ai_image::Mode::Remix),
            (
                "menu.object.ai_create_vector",
                crate::ai_image::Mode::Vector,
            ),
            ("menu.object.ai_draw_words", crate::ai_image::Mode::Draw),
        ] {
            if item(ui, key, "", doc) {
                app.ai_image.mode = mode;
                app.ai_image.info = false;
                app.open_ai_generate();
            }
        }
        let cutout =
            app.ai_image_ready() && app.ai_source_bitmap().is_some() && !app.ai_image.busy();
        let tip = app.ai_image_tip();
        if item_tip(
            ui,
            "menu.bitmaps.remove_background",
            "",
            cutout,
            tip.as_deref(),
        ) {
            app.ai_remove_background();
        }
    });
    sep(ui);
    sub(ui, "menu.object.clip_frame", |ui| {
        let is_clip = app
            .selected_shapes()
            .iter()
            .any(|s| matches!(s.kind, ShapeKind::ClipFrame { .. }));
        if item(ui, "menu.object.clip_frame_place_inside", "", has) {
            app.pending_clip_frame = true;
            app.status = tr("status.click_frame");
        }
        if item(ui, "menu.object.clip_frame_extract", "", is_clip) {
            let clips: Vec<_> = app
                .selected_shapes()
                .iter()
                .filter(|s| matches!(s.kind, ShapeKind::ClipFrame { .. }))
                .map(|s| s.id)
                .collect();
            for c in clips {
                app.run(Command::ExtractContents { clip: c });
            }
        }
        sep(ui);
        // Walked for the list of commands (Options > Customization >
        // Commands) both rows count, so each can take a shortcut (#1148).
        let editing = app.clip_frame_edit.is_some();
        if (!editing || invoke::active())
            && item(ui, "menu.object.clip_frame_edit", "", is_clip && !editing)
        {
            app.edit_clip_frame();
        }
        if (editing || invoke::active()) && item(ui, "menu.object.clip_frame_finish", "", editing) {
            app.finish_clip_frame_edit();
        }
        sep(ui);
        let framable = !app.framable().is_empty();
        if item(ui, "menu.object.clip_frame_create_empty", "", framable) {
            app.objects_to_clip_frames();
        }
        let locked = app
            .selected_shapes()
            .iter()
            .filter(|s| matches!(s.kind, ShapeKind::ClipFrame { .. }))
            .all(|s| app.clip_frame_locked(s.id));
        if check_if(ui, "menu.object.clip_frame_lock", "", is_clip && locked, is_clip) {
            app.toggle_clip_frame_lock();
        }
        sep(ui);
        // Fit the contents of the selected frames to them.
        let clips: Vec<_> = app
            .selected_shapes()
            .iter()
            .filter(|s| matches!(s.kind, ShapeKind::ClipFrame { .. }))
            .map(|s| s.id)
            .collect();
        use crate::ui::clip_bar::FitContents;
        for (key, mode) in [
            ("menu.object.clip_fit_center", FitContents::Center),
            ("menu.object.clip_fit_proportional", FitContents::FitProportionally),
            ("menu.object.clip_fill_proportional", FitContents::FillProportionally),
            ("menu.object.clip_stretch", FitContents::Stretch),
        ] {
            if item(ui, key, "", !clips.is_empty()) {
                for c in &clips {
                    app.fit_clip_contents(*c, mode);
                }
            }
        }
        sep(ui);
        if item(ui, "menu.object.clip_frame_copy_from", "", has) {
            app.pending_copy_powerclip = true;
            app.status = tr("status.click_source_object");
        }
        sep(ui);
        let masks: Vec<_> = app
            .selected_shapes()
            .iter()
            .filter(|s| matches!(s.kind, ShapeKind::ClipFrame { .. }))
            .map(|s| s.opacity_mask)
            .collect();
        let masked = !masks.is_empty() && masks.iter().all(|m| m.is_some());
        if check_if(ui, "menu.object.clip_frame_opacity_mask", "", masked, is_clip) {
            app.set_opacity_masks(!masked, false);
        }
        let inverted = masked && masks.iter().all(|m| m.is_some_and(|m| m.invert));
        if check_if(ui, "menu.object.clip_frame_invert_mask", "", inverted, masked) {
            app.set_opacity_masks(true, !inverted);
        }
    });
    sub(ui, "menu.object.symmetry", |ui| {
        let has_sym = app.selected_symmetry().is_some();
        let editing = app.symmetry_edit.is_some();
        if item(
            ui,
            "menu.object.symmetry_create",
            "Alt+S",
            !has_sym && !editing,
        ) {
            // With nothing selected, an empty symmetry to draw into.
            if has {
                app.create_symmetry();
            } else {
                let center = app.view_center();
                app.create_new_symmetry_at(center);
            }
        }
        if item(
            ui,
            "menu.object.symmetry_edit",
            "Alt+E",
            has_sym && !editing,
        ) {
            app.edit_symmetry();
        }
        if item(ui, "menu.object.symmetry_finish", "Alt+Shift+E", editing) {
            app.finish_symmetry_edit();
        }
        if item(ui, "menu.object.symmetry_remove", "Alt+Shift+S", has_sym) {
            app.remove_symmetry();
        }
        if item(ui, "menu.object.symmetry_break", "Alt+X", has_sym) {
            app.break_symmetry_link();
        }
    });
    sub(ui, "menu.object.symbol", |ui| {
        let has_symbols = !app.doc().symbols.is_empty();
        let has_instance = app
            .selected_shapes()
            .iter()
            .any(|s| matches!(s.kind, ShapeKind::SymbolInstance { .. }));
        if item(ui, "menu.object.symbol_new", "", has) {
            app.create_symbol_from_selection();
        }
        if item(ui, "menu.object.symbol_edit", "", app.can_edit_symbol()) {
            app.edit_symbol();
        }
        let editing = app.symbol_editing.is_some();
        if item(ui, "menu.object.symbol_finish", "", editing) {
            app.finish_symbol_edit();
        }
        if item(ui, "menu.object.symbol_revert", "", has_instance) {
            app.revert_symbol_instances();
        }
        if item(ui, "menu.object.symbol_insert", "", has_symbols) {
            let last = app.doc().symbols.len().saturating_sub(1);
            app.insert_symbol_instance(last);
        }
        sep(ui);
        if item(ui, "menu.object.symbol_manager", "Ctrl+F3", true) {
            app.show_dockers = true;
            app.docker_tab = DockerTab::Symbols;
        }
    });
    sub(ui, "menu.object.rollover", |ui| {
        let selected = app.selected_rollover().is_some();
        let editing = app.edited_rollover().is_some();
        if item(ui, "menu.object.rollover_create", "", app.can_create_rollover()) {
            app.create_rollover();
        }
        if item(ui, "menu.object.rollover_extract", "", selected && !editing) {
            app.extract_rollover();
        }
        sep(ui);
        if item(ui, "menu.object.rollover_edit", "", selected && !editing) {
            app.edit_rollover();
        }
        if item(ui, "menu.object.rollover_finish", "", editing) {
            app.finish_rollover();
        }
        let shown = app.edited_rollover_state();
        for (state, key) in crate::rollover::STATE_KEYS.iter().enumerate() {
            if radio_if(ui, key, "", shown == Some(state), editing) {
                app.set_edited_rollover_state(state);
            }
        }
        sep(ui);
        if check(ui, "menu.object.rollover_live_preview", "", app.rollover_live) {
            app.toggle_rollover_live_preview();
        }
    });
    sep(ui);
    sub(ui, "menu.object.transformations", |ui| {
        use crate::app::TransformTab;
        for (key, shortcut, tab) in [
            ("menu.object.tf_position", "Alt+F7", TransformTab::Position),
            ("menu.object.tf_rotate", "Alt+F8", TransformTab::Rotate),
            ("menu.object.tf_scale", "Alt+F9", TransformTab::Scale),
            ("menu.object.tf_size", "Alt+F10", TransformTab::Size),
            ("menu.object.tf_skew", "", TransformTab::Skew),
        ] {
            if item(ui, key, shortcut, true) {
                crate::ui::transform_docker::open(app, tab, false);
            }
        }
    });
    if item(ui, "menu.object.clear_transformations", "", has) {
        app.clear_transformations();
    }
    sep(ui);
    sub(ui, "menu.object.copy_effect", |ui| {
        use crate::copy_effect::CopyKind;
        // The reference's order; Envelope is copied its own way.
        for kind in [
            Some(CopyKind::Perspective),
            None,
            Some(CopyKind::Blend),
            Some(CopyKind::Extrude),
            Some(CopyKind::Contour),
            Some(CopyKind::Lens),
            Some(CopyKind::Shadow),
            Some(CopyKind::Distortion),
            Some(CopyKind::Transparency),
            Some(CopyKind::Bevel),
            Some(CopyKind::BlockShadow),
        ] {
            match kind {
                None => {
                    if item(ui, "menu.object.copy_envelope_from", "", has) {
                        app.start_copy_envelope();
                    }
                }
                Some(kind) => {
                    if item(ui, kind.key(), "", has) {
                        app.pending_copy_effect = Some(kind);
                        app.status = tr("status.click_source_object");
                    }
                }
            }
        }
    });
    sub(ui, "menu.object.clone_effect", |ui| {
        // Blend, Extrude and Contour first, as in the reference.
        for (key, kind) in [
            ("menu.object.clone_blend_from", crate::app::EffectKind::Blend),
            ("menu.object.clone_extrude_from", crate::app::EffectKind::Extrude),
            ("menu.object.clone_contour_from", crate::app::EffectKind::Contour),
        ] {
            if item(ui, key, "", has) {
                app.pending_clone_effect = Some(kind);
                app.status = tr("status.click_source_object");
            }
        }
        if item(ui, "menu.object.clone_shadow_from", "", has) {
            app.pending_clone_effect = Some(crate::app::EffectKind::Shadow);
            app.status = tr("status.click_source_object");
        }
        if item(ui, "menu.object.clone_transparency_from", "", has) {
            app.pending_clone_effect = Some(crate::app::EffectKind::Transparency);
            app.status = tr("status.click_source_object");
        }
    });
    let with_effects = app.selected_shapes().iter().any(|s| !s.effects.is_empty());
    if item(ui, "menu.object.clear_effect", "", with_effects) {
        app.clear_effects();
    }
    sep(ui);
    sub(ui, "menu.object.align_distribute", |ui| {
        use crate::ops::{Align, Distribute};
        for (key, shortcut, a) in [
            ("menu.object.align_left", "L", Align::Left),
            ("menu.object.align_right", "R", Align::Right),
            ("menu.object.align_top", "T", Align::Top),
            ("menu.object.align_bottom", "B", Align::Bottom),
            ("menu.object.align_centers_h", "E", Align::CenterH),
            ("menu.object.align_centers_v", "C", Align::CenterV),
            ("menu.object.center_to_page", "P", Align::CenterPage),
        ] {
            if a == Align::CenterPage {
                sep(ui);
            }
            if item(ui, key, shortcut, has) {
                app.align(a);
            }
        }
        for (key, a) in [
            ("menu.object.center_page_h", Align::CenterV),
            ("menu.object.center_page_v", Align::CenterH),
        ] {
            if item(ui, key, "", has) {
                app.align_with(a, crate::ops2::AlignTo::PageCenter);
            }
        }
        sep(ui);
        if item(ui, "menu.object.align_pixel_grid", "", has) {
            app.align_to_pixel_grid();
        }
        sep(ui);
        let three = app.selection.len() >= 3;
        for (key, d) in [
            ("menu.object.distribute_left", Distribute::Left),
            ("menu.object.distribute_centers_h", Distribute::CentersH),
            ("menu.object.distribute_spacing_h", Distribute::SpacingH),
            ("menu.object.distribute_right", Distribute::Right),
            ("menu.object.distribute_top", Distribute::Top),
            ("menu.object.distribute_centers_v", Distribute::CentersV),
            ("menu.object.distribute_spacing_v", Distribute::SpacingV),
            ("menu.object.distribute_bottom", Distribute::Bottom),
        ] {
            if item(ui, key, "", three) {
                app.distribute(d);
            }
        }
        sep(ui);
        if item(ui, "menu.object.align_distribute_docker", "", true) {
            app.show_dockers = true;
            app.docker_tab = DockerTab::AlignDistribute;
        }
    });
    if item(ui, "menu.object.fit_objects_to_path", "", true) {
        app.open_docker(crate::app::DockerTab::FitToPath);
    }
    sub(ui, "menu.object.order", |ui| {
        if item(ui, "menu.object.to_front_of_page", "Ctrl+Home", has) {
            app.order_page(true);
        }
        if item(ui, "menu.object.to_back_of_page", "Ctrl+End", has) {
            app.order_page(false);
        }
        sep(ui);
        if item(ui, "menu.object.to_front_of_layer", "Shift+PgUp", has) {
            app.order(0);
        }
        if item(ui, "menu.object.to_back_of_layer", "Shift+PgDn", has) {
            app.order(3);
        }
        if item(ui, "menu.object.forward_one", "Ctrl+PgUp", has) {
            app.order(1);
        }
        if item(ui, "menu.object.back_one", "Ctrl+PgDn", has) {
            app.order(2);
        }
        sep(ui);
        if item(ui, "menu.object.in_front_of", "", has) {
            app.pending_order = Some(true);
            app.status = tr("status.click_reference_object");
        }
        if item(ui, "menu.object.behind", "", has) {
            app.pending_order = Some(false);
            app.status = tr("status.click_reference_object");
        }
        sep(ui);
        if item(ui, "menu.object.reverse_order", "", many) {
            app.reverse_order();
        }
    });
    sub(ui, "menu.object.group", |ui| {
        if item(ui, "menu.object.group_group", "Ctrl+G", app.can_group()) {
            app.group_selection();
        }
        let groups = app
            .selected_shapes()
            .iter()
            .any(|s| matches!(s.kind, ShapeKind::Group { .. }));
        if item(ui, "menu.object.group_ungroup", "Ctrl+U", groups) {
            app.ungroup_selection();
        }
        if item(ui, "menu.object.group_ungroup_all", "", groups) {
            app.ungroup_all();
        }
    });
    sep(ui);
    sub(ui, "menu.object.hide", |ui| {
        if item(ui, "menu.object.hide_object", "", has) {
            app.set_visible(false);
        }
        let hidden = app.selected_shapes().iter().any(|s| !s.visible);
        if item(ui, "menu.object.show_selected", "", hidden) {
            app.set_visible(true);
        }
        // Greyed with nothing hidden on the page.
        let any_hidden = app
            .doc()
            .page(app.page)
            .is_ok_and(|p| p.layers.iter().flat_map(|l| &l.shapes).any(|s| !s.visible));
        if item(ui, "menu.object.show_all", "", any_hidden) {
            app.show_all();
        }
    });
    sub(ui, "menu.object.lock", |ui| {
        if item(ui, "menu.object.lock_object", "", has) {
            app.set_locked(true);
        }
        if item(ui, "menu.object.unlock_object", "", has) {
            app.set_locked(false);
        }
        if item(ui, "menu.object.unlock_all", "", true) {
            app.unlock_all();
        }
    });
    sep(ui);
    sub(ui, "menu.object.shaping", |ui| {
        use crate::ops::Shaping;
        for (key, op, need_two) in [
            ("menu.object.weld", Shaping::Weld, true),
            ("menu.object.trim", Shaping::Trim, true),
            ("menu.object.intersect", Shaping::Intersect, true),
            ("menu.object.simplify", Shaping::Simplify, false),
            (
                "menu.object.front_minus_back",
                Shaping::FrontMinusBack,
                true,
            ),
            (
                "menu.object.back_minus_front",
                Shaping::BackMinusFront,
                true,
            ),
            ("menu.object.boundary", Shaping::Boundary, false),
        ] {
            let on = if need_two {
                op.ready(app.selection.len())
            } else {
                has
            };
            if item(ui, key, "", on) {
                app.shaping(op);
            }
        }
        sep(ui);
        if item(ui, "menu.object.shaping_docker", "", true) {
            app.show_dockers = true;
            app.docker_tab = DockerTab::Shaping;
        }
    });
    if item(ui, "menu.object.combine", "Ctrl+L", many) {
        app.combine();
    }
    // Break Apart, or the drop shadow's or effect group's version of it.
    let label = app.break_apart_label();
    if item(ui, label, "Ctrl+K", has) {
        app.break_apart();
    }
    sub(ui, "menu.object.perspective", |ui| {
        if item(ui, "menu.object.add_perspective", "", has) {
            app.add_perspective();
        }
        sep(ui);
        if check(
            ui,
            "menu.view.draw_in_perspective",
            "",
            app.perspective.is_some(),
        ) {
            app.toggle_perspective_drawing();
        }
    });
    sep(ui);
    if item(
        ui,
        "menu.object.convert_to_curves",
        "Ctrl+Q",
        app.can_convert_to_curves(),
    ) {
        app.convert_to_curves();
    }
    if item(ui, "menu.object.convert_to_bitmap", "", has) {
        app.dialog = Dialog::ConvertToBitmap(Default::default());
    }
    if item(
        ui,
        "menu.object.convert_outline_to_object",
        "Ctrl+Shift+Q",
        has,
    ) {
        app.convert_outline_to_object();
    }
    if item(ui, "menu.object.join_curves", "", true) {
        app.show_dockers = true;
        app.docker_tab = crate::app::DockerTab::JoinCurves;
    }
    sep(ui);
    let has_fill = app
        .selected_shapes()
        .iter()
        .any(|s| !matches!(s.fill, tracedraw_core::Fill::None));
    if check(
        ui,
        "menu.object.overprint_fill",
        "",
        app.selected_shapes().iter().any(|s| s.overprint_fill),
    ) {
        app.toggle_overprint(true);
    }
    if check(
        ui,
        "menu.object.overprint_outline",
        "",
        app.selected_shapes().iter().any(|s| s.overprint_outline),
    ) {
        app.toggle_overprint(false);
    }
    let _ = has_fill;
    let has_bitmap = app
        .selected_shapes()
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Bitmap { .. }));
    if check(
        ui,
        "menu.object.overprint_bitmap",
        "",
        has_bitmap && app.selected_shapes().iter().any(|s| s.overprint_fill),
    ) && has_bitmap
    {
        app.toggle_overprint(true);
    }
    let hinted = has && app.selection.iter().all(|id| app.object_hinted(*id));
    if check(ui, "menu.object.object_hinting", "", hinted) && has {
        app.toggle_object_hinting();
    }
    sep(ui);
    if check(
        ui,
        "menu.object.properties",
        "Alt+Enter",
        app.docker_shown(DockerTab::Properties),
    ) {
        app.open_docker(DockerTab::Properties);
    }
    if check(
        ui,
        "menu.object.objects",
        "",
        app.docker_shown(DockerTab::Objects),
    ) {
        app.open_docker(DockerTab::Objects);
    }
}

fn effects_menu(app: &mut App, ui: &mut Ui) {
    let has = !app.selection.is_empty();
    let has_bitmap = app
        .selected_shapes()
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Bitmap { .. }));
    let with_effects = app.selected_shapes().iter().any(|s| !s.effects.is_empty());
    if item(ui, "menu.effects.flatten", "", with_effects) {
        app.flatten_effects();
    }
    sep(ui);
    // Bitmap effect groups.
    for (key, group) in crate::fx::groups() {
        sub(ui, key, |ui| {
            for fx in group.iter() {
                if crate::fx::separated_before(fx.id) {
                    sep(ui);
                }
                let shortcut = match fx.id {
                    "brightness_contrast_intensity" => "Ctrl+B",
                    "color_balance" => "Ctrl+Shift+B",
                    "hue_saturation_lightness" => "Ctrl+Shift+U",
                    _ => "",
                };
                // Effects with settings open a dialog, marked "...".
                let dots = if fx.params.is_empty() && fx.id != "tone_curve" {
                    ""
                } else {
                    "..."
                };
                let label = format!("{}{dots}", tr(&format!("fx.{}", fx.id)));
                // The row's picture is the effect itself, on a small
                // landscape (made once, when the menu first shows it).
                let picture = if invoke::active() {
                    None
                } else {
                    crate::ui::fx_icons::texture(ui.ctx(), fx.id)
                };
                // Bitmaps and vector objects alike (a vector object is
                // drawn as the picture its effects make of it).
                if item_picture(ui, &label, shortcut, has_bitmap || has, picture) {
                    crate::ui::effect_dialog::choose_effect(app, fx.id);
                }
            }
        });
    }
    sep(ui);
    // The effect panels: each opens its docker, checked while its tab is
    // in the docker column.
    for (key, shortcut, tab) in [
        ("menu.effects.brush_strokes", "", DockerTab::BrushStrokes),
        ("menu.effects.bevel", "", DockerTab::Bevel),
        ("menu.effects.blend", "", DockerTab::Blend),
        ("menu.effects.contour", "Ctrl+F9", DockerTab::Contour),
        ("menu.effects.envelope", "Ctrl+F7", DockerTab::Envelope),
        ("menu.effects.extrude", "", DockerTab::Extrude),
        ("menu.effects.lens", "Alt+F3", DockerTab::Lens),
        ("menu.effects.picture_mosaic", "", DockerTab::PictureMosaic),
        ("menu.effects.vector_mosaic", "", DockerTab::VectorMosaic),
    ] {
        if check(ui, key, shortcut, effect_docker_open(app, tab)) {
            app.open_docker(tab);
        }
    }
}

/// An effect panel's tab is in an open docker column, or floating.
pub(crate) fn effect_docker_open(app: &App, tab: DockerTab) -> bool {
    match app.docker_place(tab) {
        Some(crate::app::DockerPlace::Right) => app.show_dockers,
        Some(crate::app::DockerPlace::Left) => app.show_left_dockers,
        Some(p @ (crate::app::DockerPlace::RightInner | crate::app::DockerPlace::LeftInner)) => {
            app.docker_column(p).2
        }
        Some(crate::app::DockerPlace::Floating) => true,
        // Brought to the front and joining the default column.
        None => app.show_dockers && app.docker_tab == tab,
    }
}

fn bitmaps_menu(app: &mut App, ui: &mut Ui) {
    let has = !app.selection.is_empty();
    let has_bitmap = app
        .selected_shapes()
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Bitmap { .. }));
    if item(ui, "menu.bitmaps.convert_to_bitmap", "", has) {
        app.dialog = Dialog::ConvertToBitmap(Default::default());
    }
    sep(ui);
    if item(ui, "menu.bitmaps.straighten_image", "", has_bitmap) {
        app.dialog = Dialog::StraightenImage(Default::default());
    }
    // The lab: temperature, tint, saturation, tones and the black and
    // white points in one dialog (the Image Adjustments effect).
    if item(ui, "menu.bitmaps.image_adjustment_lab", "", has_bitmap) {
        crate::ui::effect_dialog::choose_effect(app, "image_adjustments");
    }
    if item(ui, "menu.bitmaps.remove_jpeg_artifacts", "", has_bitmap) {
        crate::ui::effect_dialog::choose_effect(app, "jpeg_artifact_removal");
    }
    sep(ui);
    if item(
        ui,
        "menu.bitmaps.edit_bitmap",
        "",
        app.selected_bitmap().is_some(),
    ) {
        app.edit_bitmap_externally();
    }
    // AI: greyed until the AI service makes pictures (Tools > Options >
    // AI > Service); the tip says why. Next to it, the one made on this
    // computer, with no AI service.
    let cutout = app.ai_image_ready() && app.ai_source_bitmap().is_some() && !app.ai_image.busy();
    let tip = app.ai_image_tip();
    if item_tip(
        ui,
        "menu.bitmaps.remove_background",
        "",
        cutout,
        tip.as_deref(),
    ) {
        app.ai_remove_background();
    }
    if item(ui, "menu.bitmaps.remove_background_local", "", has_bitmap) {
        crate::ui::bitmap_dialogs::open_remove_background(app);
    }
    if item(ui, "menu.bitmaps.crop_bitmap", "", has_bitmap) {
        app.set_tool(crate::tools::Tool::Crop);
    }
    let framed = !app.framed_pictures().is_empty();
    if item(ui, "menu.bitmaps.crop_to_frame", "", framed) {
        app.crop_pictures_to_frames();
    }
    if item(ui, "menu.bitmaps.resample", "", has_bitmap) {
        app.open_resample();
    }
    let styled = app.ai_image_ready() && !app.selection.is_empty() && !app.ai_image.busy();
    sub(ui, "menu.bitmaps.ai_art_styles", |ui| {
        for (i, (key, _)) in crate::ai_image::ART_STYLES.iter().enumerate() {
            if item_tip(ui, key, "", styled, tip.as_deref()) {
                app.ai_art_style(i);
            }
        }
    });
    // Black and white, duotone and paletted open their dialogs; the
    // others convert at once.
    sub(ui, "menu.bitmaps.mode", |ui| {
        if item(ui, "menu.bitmaps.mode_bw", "", has_bitmap) {
            app.dialog = Dialog::BitmapBw(Default::default());
        }
        if item(ui, "menu.bitmaps.mode_grayscale", "", has_bitmap) {
            app.set_bitmap_mode(crate::bitmap_fx::ColorMode::Grayscale);
        }
        if item(ui, "menu.bitmaps.mode_duotone", "", has_bitmap) {
            app.dialog = Dialog::BitmapDuotone(Default::default());
        }
        if item(ui, "menu.bitmaps.mode_paletted", "", has_bitmap) {
            app.dialog = Dialog::BitmapPaletted(Default::default());
        }
        // RGB and CMYK are the bitmaps' colour mode: the mode of every
        // selected bitmap is ticked (#1254).
        let modes: Vec<bool> = app
            .selected_rc()
            .iter()
            .filter(|s| matches!(s.kind, ShapeKind::Bitmap { .. }))
            .map(|s| s.cmyk_bitmap)
            .collect();
        let all_rgb = has_bitmap && modes.iter().all(|c| !c);
        let all_cmyk = has_bitmap && modes.iter().all(|c| *c);
        if mark_item_if(
            ui,
            "menu.bitmaps.mode_rgb",
            "",
            Mark::Check(all_rgb),
            has_bitmap,
        ) {
            app.set_bitmap_mode(crate::bitmap_fx::ColorMode::Rgb);
        }
        if item(ui, "menu.bitmaps.mode_lab", "", has_bitmap) {
            app.apply_to_bitmaps("Lab Color", |img, _| crate::bitmap_modes::lab_mode(img));
        }
        if mark_item_if(
            ui,
            "menu.bitmaps.mode_cmyk",
            "",
            Mark::Check(all_cmyk),
            has_bitmap,
        ) {
            app.set_bitmap_mode(crate::bitmap_fx::ColorMode::Cmyk);
        }
    });
    sub(ui, "menu.bitmaps.inflate", |ui| {
        // The same switch as Document Setup > General: effects that
        // spread past the edges grow the bitmap first.
        let auto = app.doc().metadata.auto_inflate_bitmaps();
        if check(ui, "menu.bitmaps.inflate_auto", "", auto) {
            app.set_auto_inflate_bitmaps(!auto);
        }
        if item(ui, "menu.bitmaps.inflate_manual", "", has_bitmap) {
            app.dialog = Dialog::InflateBitmap(crate::ui::dialogs::InflateState::for_app(app));
        }
    });
    sep(ui);
    let linked = app.selected_bitmap_is_linked();
    if item(ui, "menu.bitmaps.break_link", "", linked) {
        app.break_bitmap_link();
    }
    if item(ui, "menu.bitmaps.update_from_link", "", linked) {
        app.update_bitmap_from_link();
    }
    sep(ui);
    // The trace commands in the menu itself, as the reference has them:
    // the automatic trace, the last trace again or its settings, and the
    // Centerline and Outline presets.
    if item(ui, "menu.bitmaps.quick_trace", "", has_bitmap) {
        app.quick_trace();
    }
    let last = app.last_trace.is_some();
    if item(ui, "menu.bitmaps.repeat_last_trace", "", has_bitmap && last) {
        app.repeat_last_trace();
    }
    if item(
        ui,
        "menu.bitmaps.last_trace_settings",
        "",
        has_bitmap && last,
    ) {
        app.open_last_trace_settings();
    }
    sub_if(ui, "menu.bitmaps.centerline_trace", has_bitmap, |ui| {
        for (key, preset) in CENTERLINE_TRACES {
            if item(ui, key, "", has_bitmap) {
                app.open_trace(preset);
            }
        }
    });
    sub_if(ui, "menu.bitmaps.outline_trace", has_bitmap, |ui| {
        for (key, preset) in OUTLINE_TRACES {
            if item(ui, key, "", has_bitmap) {
                app.open_trace(preset);
            }
        }
    });
    sep(ui);
    todo_sub(ui, "menu.bitmaps.plugins");
    sep(ui);
    if item(ui, "menu.bitmaps.bitmap_mask", "", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::BitmapMask;
    }
}

fn text_menu(app: &mut App, ui: &mut Ui) {
    let is_text = app
        .selected_shapes()
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Text { .. }));
    let is_para = app
        .selected_shapes()
        .iter()
        .any(|s| matches!(s.kind, ShapeKind::Text { frame: Some(_), .. }));
    if item(ui, "menu.text.tabs", "", true) {
        app.dialog = Dialog::TextTabs;
    }
    if item(ui, "menu.text.columns", "", true) {
        app.dialog = Dialog::TextColumns;
    }
    if item(ui, "menu.text.bullets", "", true) {
        app.dialog = Dialog::TextBullets;
    }
    if item(ui, "menu.text.drop_cap", "", true) {
        app.dialog = Dialog::TextDropCap;
    }
    if item(ui, "menu.text.text_statistics", "", true) {
        app.dialog = Dialog::TextStatistics;
    }
    if item(ui, "menu.text.font_sampler", "", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::FontSampler;
    }
    sep(ui);
    // Vertical text, offered with a CJK interface language or for text
    // in those scripts (#257).
    // (Listed in Options > Customization > Commands all the same, so it
    // can take a shortcut, #1148.)
    let offered = app.vertical_text_offered();
    if offered || invoke::active() {
        let on = app.vertical_text_on();
        if mark_item_if(ui, "menu.text.vertical_text", "", Mark::Check(on), offered) {
            app.set_vertical_text(!on);
        }
    }
    if item(ui, "menu.text.edit_text", "Ctrl+Shift+T", is_text) {
        app.open_edit_text_dialog();
    }
    sub(ui, "menu.text.insert_formatting_code", |ui| {
        for (key, code) in [
            ("menu.text.code_em_space", "\u{2003}"),
            ("menu.text.code_en_space", "\u{2002}"),
            ("menu.text.code_quarter_space", "\u{2005}"),
            ("menu.text.code_nonbreaking_space", "\u{00A0}"),
            ("menu.text.code_tab", "\t"),
            ("menu.text.code_column_break", "\u{000C}"),
            ("menu.text.code_line_break", "\u{2028}"),
            ("", ""),
            ("menu.text.code_em_dash", "\u{2014}"),
            ("menu.text.code_en_dash", "\u{2013}"),
            ("menu.text.code_nonbreaking_hyphen", "\u{2011}"),
            ("menu.text.code_optional_hyphen", "\u{00AD}"),
        ] {
            if key.is_empty() {
                sep(ui);
            } else if item(ui, key, "", app.text_edit.is_some()) {
                app.insert_text(code);
            }
        }
        sep(ui);
        if item(ui, "menu.text.custom_hyphens", "", true) {
            app.dialog = Dialog::CustomHyphens(Default::default());
        }
    });
    sep(ui);
    if item(ui, "menu.text.convert", "", is_text) {
        app.toggle_text_kind();
    }
    let joinable = !app.joinable_text_lines().is_empty();
    if item(ui, "menu.text.join_lines", "", joinable) {
        app.join_text_lines();
    }
    if check(ui, "menu.text.show_non_printing", "", app.show_non_printing) {
        app.show_non_printing = !app.show_non_printing;
    }
    sep(ui);
    sub(ui, "menu.text.paragraph_text_frame", |ui| {
        if item(ui, "menu.text.frame_fit_text", "", is_para) {
            app.fit_text_to_frame();
        }
        if item(ui, "menu.text.insert_placeholder_text", "", is_para) {
            app.insert_placeholder_text();
        }
        sep(ui);
        if item(ui, "menu.text.frame_create_in_selected", "", app.selection.len() == 1) {
            app.object_to_text_frame();
        }
        sep(ui);
        if item(ui, "menu.text.frame_link", "", app.selection.len() == 2) {
            app.link_text_frames();
        }
        let linked = app.selection.iter().any(|id| app.is_linked_frame(*id));
        if item(ui, "menu.text.frame_unlink", "", linked) {
            app.unlink_text_frames();
        }
    });
    // Linked text content (#1568): texts that hold the same characters,
    // each in its own look; not the frame flow above.
    if item(
        ui,
        "menu.text.link_content",
        "",
        app.can_link_text_content(),
    ) {
        app.link_text_content();
    }
    if item(
        ui,
        "menu.text.unlink_content",
        "",
        app.selection_has_content_link(),
    ) {
        app.unlink_text_content();
    }
    sep(ui);
    // With a text and a path selected they go together; otherwise the
    // next click picks the path.
    if item(ui, "menu.text.fit_text_to_path", "", app.has_document()) {
        app.begin_fit_text_to_path();
    }
    // Shifted and rotated characters go back (the selected characters
    // while editing, the chosen character nodes with the Shape tool, else
    // all); Straighten Text also takes text off its path.
    if item(ui, "menu.text.straighten_text", "", is_text) {
        app.straighten_chars();
    }
    if item(ui, "menu.text.align_to_baseline", "Alt+F12", is_text) {
        app.align_chars_to_baseline();
    }
    let on_grid = app
        .text_shapes()
        .iter()
        .any(|s| matches!(&s.kind, ShapeKind::Text { para, .. } if para.baseline_grid_mm > 0.0));
    if mark_item_if(
        ui,
        "menu.text.align_to_baseline_grid",
        "",
        Mark::Check(on_grid),
        is_para,
    ) {
        app.toggle_baseline_grid();
    }
    sep(ui);
    if check(ui, "menu.text.use_hyphenation", "", app.uses_hyphenation()) {
        app.toggle_use_hyphenation();
    }
    sub(ui, "menu.text.writing_tools", |ui| {
        if item(ui, "menu.text.spell_check", "", true) {
            app.dialog =
                Dialog::WritingTools(crate::ui::writing_tools::WritingToolsState::on_tab(0));
        }
        if item(ui, "menu.text.grammar", "", true) {
            app.dialog =
                Dialog::WritingTools(crate::ui::writing_tools::WritingToolsState::on_tab(1));
        }
        if item(ui, "menu.text.thesaurus", "", true) {
            app.dialog =
                Dialog::WritingTools(crate::ui::writing_tools::WritingToolsState::on_tab(2));
        }
        if item(ui, "menu.text.autocorrect", "", true) {
            app.dialog = Dialog::Options;
            app.options_page = crate::ui::dialogs::OptionsPage::Text;
            app.options_text_tab = crate::ui::options::AUTOCORRECT_TAB;
        }
    });
    sep(ui);
    if item(ui, "menu.text.change_case", "Shift+F3", is_text) {
        app.dialog = Dialog::ChangeCase(crate::ops2::CaseMode::Sentence);
    }
    if item(ui, "menu.text.make_web_compatible", "", is_text) {
        app.make_text_web_compatible();
    }
    sep(ui);
    if item(ui, "menu.text.encode", "", is_text) {
        app.dialog = Dialog::Encode(Default::default());
    }
    sep(ui);
    if item(ui, "menu.text.text_docker", "Ctrl+T", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Text;
    }
    if item(ui, "menu.text.glyphs", "Ctrl+F11", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Glyphs;
    }
}

fn table_menu(app: &mut App, ui: &mut Ui) {
    let in_table = app.selected_table().is_some();
    if item(ui, "menu.table.create_new_table", "", true) {
        app.dialog = Dialog::CreateTable {
            rows: app.table_rows,
            cols: app.table_cols,
            width: app.table_size.0,
            height: app.table_size.1,
        };
    }
    if item(
        ui,
        "menu.table.convert_text_to_table",
        "",
        app.selected_shapes()
            .iter()
            .any(|s| matches!(s.kind, ShapeKind::Text { .. })),
    ) {
        if let Some(text) = app.selected_text_for_table() {
            app.dialog = Dialog::TextToTable(crate::table::TextToTable::detect(&text));
        }
    }
    if item(ui, "menu.table.convert_table_to_text", "", in_table) {
        app.convert_table_to_text();
    }
    sep(ui);
    sub(ui, "menu.table.insert", |ui| {
        for (key, op) in [
            (
                "menu.table.insert_row_above",
                crate::table::TableOp::RowAbove,
            ),
            (
                "menu.table.insert_row_below",
                crate::table::TableOp::RowBelow,
            ),
            (
                "menu.table.insert_column_left",
                crate::table::TableOp::ColLeft,
            ),
            (
                "menu.table.insert_column_right",
                crate::table::TableOp::ColRight,
            ),
        ] {
            if item(ui, key, "", in_table) {
                app.table_op(op);
            }
        }
        sep(ui);
        if item(ui, "menu.table.insert_rows", "", in_table) {
            app.open_table_count(crate::ui::table_bar::TableCountKind::InsertRows);
        }
        if item(ui, "menu.table.insert_columns", "", in_table) {
            app.open_table_count(crate::ui::table_bar::TableCountKind::InsertCols);
        }
    });
    sub(ui, "menu.table.select", |ui| {
        for (key, op) in [
            ("menu.table.select_cell", crate::table::TableOp::SelectCell),
            ("menu.table.select_row", crate::table::TableOp::SelectRow),
            ("menu.table.select_column", crate::table::TableOp::SelectCol),
            (
                "menu.table.select_table",
                crate::table::TableOp::SelectTable,
            ),
        ] {
            if item(ui, key, "", in_table) {
                app.table_op(op);
            }
        }
    });
    sub(ui, "menu.table.delete", |ui| {
        for (key, op) in [
            ("menu.table.delete_row", crate::table::TableOp::DeleteRow),
            ("menu.table.delete_column", crate::table::TableOp::DeleteCol),
            (
                "menu.table.delete_table",
                crate::table::TableOp::DeleteTable,
            ),
        ] {
            if item(ui, key, "", in_table) {
                app.table_op(op);
            }
        }
    });
    sub(ui, "menu.table.distribute", |ui| {
        for (key, op) in [
            (
                "menu.table.distribute_rows",
                crate::table::TableOp::DistributeRows,
            ),
            (
                "menu.table.distribute_columns",
                crate::table::TableOp::DistributeCols,
            ),
        ] {
            if item(ui, key, "", in_table) {
                app.table_op(op);
            }
        }
    });
    sep(ui);
    if item(ui, "menu.table.merge_cells", "Ctrl+M", in_table) {
        app.table_op(crate::table::TableOp::Merge);
    }
    if item(ui, "menu.table.unmerge_cells", "", in_table) {
        app.table_op(crate::table::TableOp::Unmerge);
    }
    if item(ui, "menu.table.split_into_rows", "", in_table) {
        app.open_table_count(crate::ui::table_bar::TableCountKind::SplitRows);
    }
    if item(ui, "menu.table.split_into_columns", "", in_table) {
        app.open_table_count(crate::ui::table_bar::TableCountKind::SplitCols);
    }
}

/// Tools > Macros: the Macro Manager, Run Macro, the editor, loading a
/// project, and every loaded macro by project (so each can get a
/// shortcut in Options > Customization).
fn macros_items(app: &mut App, ui: &mut Ui) {
    app.macros.ensure_loaded();
    let running = app.macros.run.is_some();
    if item(ui, "menu.tools.macro_manager", "", true) {
        app.show_macro_manager();
    }
    if item(ui, "menu.tools.macros_run", "Alt+Shift+F8", true) {
        app.open_run_macro();
    }
    if item(ui, "menu.tools.macro_editor", "Alt+F11", true) {
        app.open_macro_editor();
    }
    if item(ui, "menu.tools.macros_load", "", true) {
        app.load_macro_project();
    }
    if item(ui, "menu.tools.macros_stop", "", running) {
        app.stop_macro();
    }
    let all: Vec<(String, Vec<(String, String)>)> = app
        .macros
        .projects
        .iter()
        .map(|p| (p.name().to_string(), p.macros()))
        .collect();
    if !all.is_empty() {
        sep(ui);
    }
    let mut run = None;
    for (i, (project, macros)) in all.iter().enumerate() {
        let key = crate::macros::project_menu_key(project);
        sub(ui, &key, |ui| {
            for (m, n) in macros {
                let id = crate::macros::command_key(project, m, n);
                if item_text(ui, &format!("{m}.{n}"), "", !running, None, &id) {
                    run = Some((i, m.clone(), n.clone()));
                }
            }
        });
    }
    if let Some((i, m, n)) = run {
        app.request_macro(i, &m, &n);
    }
}

fn tools_menu(app: &mut App, ui: &mut Ui) {
    sub(ui, "menu.tools.options", |ui| options_items(app, ui));
    if item(ui, "menu.tools.save_settings_as_default", "", true) {
        app.save_defaults();
    }
    sep(ui);
    if item(ui, "menu.tools.color_management", "", true) {
        app.dialog = Dialog::ColorManagement(Default::default());
    }
    if item(ui, "menu.tools.convert_colors", "", true) {
        app.convert_colors_dialog();
    }
    sep(ui);
    sub(ui, "menu.tools.scripts", |ui| {
        // As the reference's Scripts submenu: editor and run | recording |
        // temporary script | the panel.
        if item(ui, "menu.tools.scripts_editor", "", true) {
            app.open_macro_editor();
        }
        if item(ui, "menu.tools.scripts_run", "", true) {
            app.run_script_file();
        }
        sep(ui);
        let recording = app.recording.is_some();
        if item(ui, "menu.tools.scripts_record", "", !recording) {
            app.start_recording(false);
        }
        if item(ui, "menu.tools.scripts_stop_recording", "", recording) {
            app.stop_recording();
        }
        if item(ui, "menu.tools.scripts_cancel_recording", "", recording) {
            app.cancel_recording();
        }
        sep(ui);
        if item(ui, "menu.tools.scripts_record_temporary", "", !recording) {
            app.start_recording(true);
        }
        if item(ui, "menu.tools.scripts_run_temporary", "", app.temporary_script.is_some()) {
            app.run_temporary_script();
        }
        sep(ui);
        if item(ui, "menu.tools.scripts_docker", "", true) {
            app.show_dockers = true;
            app.docker_tab = DockerTab::Scripts;
        }
    });
    sub(ui, "menu.tools.macros", |ui| macros_items(app, ui));
    if item(ui, "menu.tools.ai_generate", "", true) {
        app.open_ai_generate();
    }
    if item(ui, "menu.tools.font_manager", "", true) {
        app.dialog = Dialog::FontManager(Default::default());
    }
    if crate::ui::tracecut::shown(app) && item(ui, "menu.tools.tracecut", "", true) {
        app.open_tracecut();
    }
    if item(ui, "menu.tools.border_and_grommet", "", true) {
        app.dialog = Dialog::BorderGrommet(Default::default());
    }
    sep(ui);
    let doc = app.has_document() && !app.show_welcome;
    if item(ui, "menu.tools.time_tracker", "", doc) {
        app.dialog = Dialog::TimeTracker(Default::default());
    }
}
/// Every bar with a check at each one shown (Window > Toolbars and the
/// bars' menu): the menu bar (always there), the standard toolbar, the
/// property bar, the toolbox, the colour palette, the AI prompt bar, the
/// status bar, the Text, Zoom and Transform toolbars and the time
/// tracker's bar.
pub(crate) fn toolbars_list(app: &mut App, ui: &mut Ui) {
    if check(ui, "menu.window.toolbar_menu_bar", "", app.show_menu_bar) {
        app.show_menu_bar = !app.show_menu_bar;
    }
    if check(
        ui,
        "menu.window.toolbar_standard",
        "",
        app.show_standard_toolbar,
    ) {
        app.show_standard_toolbar = !app.show_standard_toolbar;
    }
    if check(
        ui,
        "menu.window.toolbar_property_bar",
        "",
        app.show_property_bar,
    ) {
        app.show_property_bar = !app.show_property_bar;
    }
    if check(ui, "menu.window.toolbar_toolbox", "", app.show_toolbox) {
        app.show_toolbox = !app.show_toolbox;
    }
    let palette = app.settings.palette.show_palette;
    if check(ui, "bars.palette", "", palette) {
        app.settings.palette.show_palette = !palette;
    }
    // Shown while an AI service is set up (AI Drawing).
    let ai_bar = app.settings.ai.show_bar;
    if check(ui, "bars.ai_prompt", "", ai_bar) {
        app.settings.ai.show_bar = !ai_bar;
    }
    if check(
        ui,
        "menu.window.toolbar_status_bar",
        "",
        app.show_status_bar,
    ) {
        app.show_status_bar = !app.show_status_bar;
    }
    if check(ui, "menu.window.toolbar_text", "", app.show_text_toolbar) {
        app.show_text_toolbar = !app.show_text_toolbar;
    }
    if check(ui, "menu.window.toolbar_zoom", "", app.show_zoom_toolbar) {
        app.show_zoom_toolbar = !app.show_zoom_toolbar;
    }
    if check(
        ui,
        "menu.window.toolbar_transform",
        "",
        app.show_transform_toolbar,
    ) {
        app.show_transform_toolbar = !app.show_transform_toolbar;
    }
    let tracker = app.settings.time_tracker.show_bar;
    if check(ui, "menu.window.toolbar_time_tracker", "", tracker) {
        app.settings.time_tracker.show_bar = !tracker;
    }
}

fn window_menu(app: &mut App, ui: &mut Ui) {
    let doc = app.has_document() && !app.show_welcome;
    if item(ui, "menu.window.new_window", "", doc) {
        app.new_window();
    }
    if item(ui, "menu.window.refresh_window", "Ctrl+W", true) {
        app.raster.borrow_mut().invalidate();
    }
    if item(ui, "menu.window.close_window", "Ctrl+F4", true) {
        app.close_document();
    }
    if item(ui, "menu.window.close_all", "", true) {
        app.close_all_documents();
    }
    sep(ui);
    {
        use crate::ui::windows::{set_layout, WindowLayout};
        let has = app.has_document();
        let arranged = app.window_layout != WindowLayout::Tabs;
        for (key, layout) in [
            ("menu.window.cascade", WindowLayout::Cascade),
            (
                "menu.window.tile_horizontally",
                WindowLayout::TileHorizontal,
            ),
            ("menu.window.tile_vertically", WindowLayout::TileVertical),
        ] {
            if item(ui, key, "", has) {
                app.show_welcome = false;
                set_layout(app, layout);
            }
        }
        for key in ["menu.window.combine_windows", "menu.window.dock_window"] {
            if item(ui, key, "", arranged) {
                set_layout(app, WindowLayout::Tabs);
            }
        }
    }
    sep(ui);
    sub(ui, "menu.window.workspace", |ui| {
        for (key, ws) in [
            (
                "menu.window.workspace_default",
                crate::app::Workspace::Default,
            ),
            ("menu.window.workspace_lite", crate::app::Workspace::Lite),
            (
                "menu.window.workspace_classic",
                crate::app::Workspace::Classic,
            ),
            (
                "menu.window.workspace_illustration",
                crate::app::Workspace::Illustration,
            ),
            (
                "menu.window.workspace_page_layout",
                crate::app::Workspace::PageLayout,
            ),
            ("menu.window.workspace_touch", crate::app::Workspace::Touch),
        ] {
            if check(ui, key, "", app.workspace == ws) {
                app.set_workspace(ws);
            }
        }
        // The workspaces the user saved, then making and deleting them.
        let saved = crate::workspace_file::saved_dir()
            .map(|d| crate::workspace_file::saved_in(&d))
            .unwrap_or_default();
        for (name, path) in &saved {
            if item_label(ui, name, "", true) {
                app.import_workspace_from(path, crate::workspace_file::WorkspaceParts::ALL, true);
            }
        }
        sep(ui);
        if item(ui, "menu.window.workspace_new", "", true) {
            app.workspace_new = Some(String::new());
        }
        sub_if(ui, "menu.window.workspace_delete", !saved.is_empty(), |ui| {
            for (name, path) in &saved {
                if item_label(ui, name, "", true) {
                    let _ = std::fs::remove_file(path);
                    app.status = crate::i18n::trf("status.workspace_deleted", &[("name", name)]);
                }
            }
        });
        // Every part, whatever the options chose.
        if item(ui, "menu.window.workspace_import", "", true) {
            app.import_workspace_dialog(crate::workspace_file::WorkspaceParts::ALL, true);
        }
        if item(ui, "menu.window.workspace_export", "", true) {
            app.export_workspace_dialog(crate::workspace_file::WorkspaceParts::ALL);
        }
    });
    sep(ui);
    sub(ui, "menu.window.dockers", |ui| {
        let row = |ui: &mut Ui, app: &mut App, tab: DockerTab| {
            let shown = app.docker_shown(tab);
            if check(ui, tab.key(), tab.shortcut(), shown) {
                if shown {
                    app.close_docker(tab);
                } else {
                    app.open_docker(tab);
                }
            }
        };
        for (i, group) in DockerTab::MENU_GROUPS.iter().enumerate() {
            if i > 0 {
                sep(ui);
            }
            for tab in group.iter() {
                row(ui, app, *tab);
            }
            // The effect panels follow the text group, as a submenu.
            if i == 5 {
                sep(ui);
                sub(ui, "menu.window.dockers_effects", |ui| {
                    for tab in DockerTab::EFFECTS_MENU {
                        row(ui, app, tab);
                    }
                });
            }
        }
    });
    if check(ui, "menu.effects.picker", "", app.effect_picker.open) {
        app.effect_picker.open = !app.effect_picker.open;
    }
    sub(ui, "menu.window.toolbars", |ui| {
        // First, as in the reference: the size of every bar's buttons.
        sub(ui, "bars.default_button_size", |ui| {
            for (key, size) in crate::ui::bars::size_menu(false) {
                let Some(size) = size else { continue };
                if radio(ui, key, "", app.settings.button_size == size) {
                    app.settings.button_size = size;
                }
            }
        });
        sep(ui);
        toolbars_list(app, ui);
        sep(ui);
        let locked = app.settings.bars.locked;
        if check(ui, "bars.lock", "", locked) {
            app.settings.bars.locked = !locked;
        }
        if item(ui, "bars.reset", "", true) {
            crate::ui::bars::reset(app);
        }
    });
    sub(ui, "menu.window.color_palettes", |ui| color_palettes_body(app, ui));
    sep(ui);
    if radio(ui, "menu.window.welcome_screen", "", app.show_welcome) {
        app.show_welcome = true;
    }
    // One entry per open drawing; a dot marks the one in front.
    for i in 0..app.docs.len() {
        let title = app.document_tab_title(i);
        let on = !app.show_welcome && i == app.active_doc;
        if menu_row(ui, &title, "", true, Mark::Radio(on)).0.clicked() {
            ui.close();
            app.switch_document(i);
        }
    }
}

/// The Color Palettes submenu: Window > Color Palettes, and View >
/// Color Palettes of the page and desktop context menus.
pub(crate) fn color_palettes_body(app: &mut App, ui: &mut Ui) {
    let entries: Vec<(usize, String, String, bool)> = app
        .palettes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                i,
                p.name_key().to_string(),
                app.palette_name(i),
                app.visible_palettes.contains(&i),
            )
        })
        .collect();
    // New palettes and opening one, then the editors, then the palettes
    // shown, as the reference's menu configuration has them.
    if item(
        ui,
        "menu.window.palette_from_selection",
        "",
        !app.selection.is_empty(),
    ) {
        app.palette_from_selection();
    }
    if item(ui, "menu.window.palette_add_from_document", "", app.has_document()) {
        app.add_document_colors_to_palette();
    }
    sep(ui);
    if item(ui, "menu.window.palette_new_from_selection", "", !app.selection.is_empty()) {
        app.new_palette_from_selection();
    }
    if item(ui, "menu.window.palette_from_document", "", true) {
        app.palette_from_document();
    }
    if item(ui, "palette.new", "", true) {
        crate::ui::palette_bars::start_new_palette(app);
    }
    if item(ui, "menu.window.palette_open", "", true) {
        app.open_palette_file();
    }
    let open = app.settings.palette.show_palette
        || app.settings.palette.show_document
        || !app.settings.palette_bars.is_empty();
    if item(ui, "menu.window.palette_close_all", "", open) {
        app.close_all_palettes();
    }
    sep(ui);
    if item(ui, "menu.window.palette_editor", "", true) {
        app.dialog = Dialog::PaletteEditor(Default::default());
    }
    if item(ui, "menu.window.palette_manager", "", true) {
        app.show_dockers = true;
        app.docker_tab = DockerTab::Palettes;
    }
    sep(ui);
    if check(
        ui,
        "palette.show_document",
        "",
        app.settings.palette.show_document,
    ) {
        app.settings.palette.show_document = !app.settings.palette.show_document;
        app.settings.save();
    }
    for (i, key, name, on) in entries {
        if check_label(ui, &name, &key, on) {
            app.toggle_palette(i);
        }
    }
    let doc_open = app.extra_palettes.iter().any(|r| r.document);
    let doc_name = app.document_palette_name();
    if check_label(ui, &doc_name, "palette.document_bar", doc_open) {
        match app.extra_palettes.iter().position(|r| r.document) {
            Some(k) => crate::ui::palette_bars::close_bar(app, k),
            None => app.open_palette_bar(crate::settings::DOCUMENT_PALETTE),
        }
    }
}

fn help_menu(app: &mut App, ui: &mut Ui) {
    if item(ui, "menu.help.help_topics", "", true) {
        app.open_help(crate::help::HOME);
    }
    // The page of the active docker, else of the active tool (F1).
    if item(ui, "menu.help.product_help", "F1", true) {
        let topic = crate::help::context_topic(app);
        app.open_help(topic);
    }
    sep(ui);
    if item(ui, "menu.help.welcome_screen", "", true) {
        app.show_welcome = true;
    }
    if item(ui, "menu.help.video_tutorials", "", true) {
        app.show_welcome = true;
        app.welcome_tab = crate::app::WelcomeTab::Learn;
    }
    let hints_open = app.docker_shown(DockerTab::Hints);
    if check(ui, "menu.help.hints", "", hints_open) {
        if hints_open {
            app.close_docker(DockerTab::Hints);
        } else {
            app.open_docker(DockerTab::Hints);
        }
    }
    if item(ui, "menu.help.quick_start_guide", "", true) {
        app.show_welcome = true;
        app.welcome_tab = crate::app::WelcomeTab::Learn;
    }
    if item(ui, "menu.help.user_guide", "", true) {
        app.show_welcome = true;
        app.welcome_tab = crate::app::WelcomeTab::Learn;
    }
    sep(ui);
    if item(ui, "menu.help.whats_new", "", true) {
        open_whats_new(app);
    }
    if check(
        ui,
        "menu.help.highlight_whats_new",
        "",
        app.highlight_whats_new,
    ) {
        app.highlight_whats_new = !app.highlight_whats_new;
    }
    sep(ui);
    if item(ui, "menu.help.updates", "", true) {
        app.check_for_updates(Some(ui.ctx().clone()));
    }
    if item(ui, "menu.help.message_settings", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::General;
    }
    if item(ui, "menu.help.community", "", true) {
        app.open_url("https://github.com/ecomyseo/craft-plumilla/issues");
    }
    if item(ui, "menu.help.support", "", true) {
        app.open_url("https://github.com/ecomyseo/craft-plumilla/issues");
    }
    sep(ui);
    if item(ui, "menu.help.about", "", true) {
        app.dialog = Dialog::About;
    }
}

/// Tools > Options and the standard bar's Options button: the option
/// pages to open.
pub(crate) fn options_items(app: &mut App, ui: &mut Ui) {
    if item(ui, "menu.tools.options_app", "Ctrl+J", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::General;
    }
    if item(ui, "menu.tools.options_customization", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::Appearance;
    }
    if item(ui, "menu.tools.options_tools", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::Pick;
    }
    if item(ui, "menu.tools.options_global", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::GlobalGeneral;
    }
    if item(ui, "menu.tools.options_workspaces", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::Workspace;
    }
    if item(ui, "menu.tools.options_ai", "", true) {
        app.dialog = Dialog::Options;
        app.options_page = crate::ui::dialogs::OptionsPage::AiGeneral;
    }
}

#[cfg(test)]
mod outline_tests {
    use super::*;

    /// Tools > Scripts: start, stop and cancel a recording, and a temporary
    /// script that is kept and run again.
    #[test]
    fn the_scripts_submenu_records_cancels_and_replays() {
        use tracedraw_core::Rect;
        crate::i18n::set_language("en");
        let ctx = crate::theme::ui_context();
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        let mut app = App::headless();
        let count = |app: &App| app.doc().pages[0].layers.iter().map(|l| l.shapes.len()).sum::<usize>();
        let draw = |app: &mut App| {
            app.new_shape(ShapeKind::Rect { rect: Rect::new(0.0, 0.0, 10.0, 10.0), radius: 0.0, corners: None })
                .expect("rect");
        };
        assert!(!run_command(&mut app, &ctx, "menu.tools.scripts_stop_recording"));
        assert!(!run_command(&mut app, &ctx, "menu.tools.scripts_run_temporary"));
        // Cancel keeps nothing.
        app.script_source.clear();
        assert!(run_command(&mut app, &ctx, "menu.tools.scripts_record"));
        assert!(app.recording.is_some());
        draw(&mut app);
        assert!(run_command(&mut app, &ctx, "menu.tools.scripts_cancel_recording"));
        assert!(app.recording.is_none() && app.temporary_script.is_none());
        assert!(app.script_source.is_empty(), "nothing goes to the Scripts panel either");
        // A temporary recording is kept and runs again.
        assert!(run_command(&mut app, &ctx, "menu.tools.scripts_record_temporary"));
        draw(&mut app);
        assert!(run_command(&mut app, &ctx, "menu.tools.scripts_stop_recording"));
        assert!(app.temporary_script.is_some());
        let before = count(&app);
        assert!(run_command(&mut app, &ctx, "menu.tools.scripts_run_temporary"));
        assert_eq!(count(&app), before + 1, "{}", app.status);
        // A normal recording goes to the Scripts panel.
        assert!(run_command(&mut app, &ctx, "menu.tools.scripts_record"));
        draw(&mut app);
        assert!(run_command(&mut app, &ctx, "menu.tools.scripts_stop_recording"));
        assert!(app.script_source.contains("Recorded macro"));
    }

    /// Window > Toolbars > Default Button Size sets every bar's buttons.
    #[test]
    fn the_toolbars_submenu_sets_the_button_size() {
        crate::i18n::set_language("en");
        let ctx = crate::theme::ui_context();
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        let mut app = App::headless();
        let key = crate::settings::ButtonSize::Large.key();
        assert!(run_command(&mut app, &ctx, key));
        assert_eq!(app.settings.button_size, crate::settings::ButtonSize::Large);
    }

    /// The rows taken in from the reference's submenus do what they say:
    /// PowerClip's empty frame and fitting, centring on the page each way,
    /// Show Objects, a text frame of the selected object, the document's
    /// colours and Close All Palettes.
    #[test]
    fn the_rows_taken_from_the_reference_work() {
        use tracedraw_core::{Command, Rect};
        crate::i18n::set_language("en");
        let ctx = crate::theme::ui_context();
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        let mut app = App::headless();
        let rect = |app: &mut App, r: Rect| {
            app.new_shape(ShapeKind::Rect { rect: r, radius: 0.0, corners: None }).expect("rect")
        };
        let bounds = |app: &App, id| app.doc().find_shape(id).expect("shape").bounds();
        // PowerClip: contents centred in their frame.
        let inside = rect(&mut app, Rect::new(0.0, 0.0, 10.0, 10.0));
        let frame = rect(&mut app, Rect::new(100.0, 100.0, 200.0, 160.0));
        app.run(Command::PlaceInside { contents: vec![inside], frame });
        app.select(vec![frame]);
        assert!(run_command(&mut app, &ctx, "menu.object.clip_fit_center"));
        let b = bounds(&app, inside);
        assert!((b.center().x - 150.0).abs() < 1e-6 && (b.center().y - 130.0).abs() < 1e-6, "{b:?}");
        assert!(run_command(&mut app, &ctx, "menu.object.clip_stretch"));
        let b = bounds(&app, inside);
        assert!((b.width() - 100.0).abs() < 1e-6 && (b.height() - 60.0).abs() < 1e-6, "{b:?}");
        // Copy PowerClip From: another object takes a copy of the contents,
        // centred the way they sit in the frame they come from.
        let target = rect(&mut app, Rect::new(400.0, 400.0, 460.0, 440.0));
        app.select(vec![target]);
        assert!(run_command(&mut app, &ctx, "menu.object.clip_frame_copy_from"));
        assert!(app.pending_copy_powerclip);
        app.pending_copy_powerclip = false;
        app.copy_powerclip_from(frame);
        let t = app.doc().find_shape(target).expect("target").clone();
        let ShapeKind::ClipFrame { contents, .. } = &t.kind else { panic!("not a clip frame: {:?}", t.kind) };
        assert_eq!(contents.len(), 1);
        let src_off = bounds(&app, inside).center() - bounds(&app, frame).center();
        let copy_off = contents[0].bounds().center() - t.bounds().center();
        assert!((src_off - copy_off).hypot() < 1e-6, "{src_off:?} {copy_off:?}");
        assert_ne!(contents[0].id, inside, "a copy, with its own id");
        // An object becomes an empty clip frame.
        let plain = rect(&mut app, Rect::new(300.0, 0.0, 340.0, 40.0));
        app.select(vec![plain]);
        assert!(run_command(&mut app, &ctx, "menu.object.clip_frame_create_empty"));
        assert!(matches!(&app.doc().find_shape(plain).expect("frame").kind, ShapeKind::ClipFrame { contents, .. } if contents.is_empty()));
        // Centre to the page horizontally: x only.
        let s = rect(&mut app, Rect::new(5.0, 7.0, 25.0, 17.0));
        app.select(vec![s]);
        let page = app.page_rect();
        assert!(run_command(&mut app, &ctx, "menu.object.center_page_h"));
        let b = bounds(&app, s);
        assert!((b.center().x - page.center().x).abs() < 1e-6 && (b.y0 - 7.0).abs() < 1e-6, "{b:?}");
        assert!(run_command(&mut app, &ctx, "menu.object.center_page_v"));
        let b = bounds(&app, s);
        assert!((b.center().y - page.center().y).abs() < 1e-6, "{b:?}");
        assert!(run_command(&mut app, &ctx, "menu.object.align_pixel_grid"));
        // Show Objects: a hidden object selected comes back.
        app.select(vec![s]);
        app.set_visible(false);
        app.select(vec![s]);
        assert!(!app.doc().find_shape(s).expect("s").visible);
        assert!(run_command(&mut app, &ctx, "menu.object.show_selected"));
        assert!(app.doc().find_shape(s).expect("s").visible);
        // A paragraph text frame of the selected object.
        app.select(vec![s]);
        assert!(run_command(&mut app, &ctx, "menu.text.frame_create_in_selected"));
        // Close All Palettes puts every palette away.
        assert!(run_command(&mut app, &ctx, "menu.window.palette_close_all"));
        assert!(!app.settings.palette.show_palette && !app.settings.palette.show_document);
        assert!(app.settings.palette_bars.is_empty());
        app.settings.palette.show_palette = true;
        assert!(run_command(&mut app, &ctx, "menu.window.palette_add_from_document"));
    }

    /// The top level of every menu in order: items, submenus (`>`) and
    /// separators (`|`), printed one line per menu for the row-by-row
    /// contrast with the reference's menu configuration
    /// (`utiles/claude-goal/contraste_menus.py`). Leading, trailing and
    /// doubled separators would be drawing errors and are refused.
    #[test]
    fn menu_outline() {
        crate::i18n::set_language("en");
        let mut app = App::headless();
        let ctx = crate::theme::ui_context();
        invoke::MARKS.with(|s| s.borrow_mut().clear());
        let mut entries = Vec::new();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| entries = commands(&mut app, ui.ctx()));
        out.textures_delta.clear();
        let marks = invoke::MARKS.with(|s| s.take());
        let mut menus: Vec<(String, Vec<String>)> = Vec::new();
        let mut push = |top: &str, token: String| match menus.iter_mut().find(|(t, _)| t == top) {
            Some((_, v)) => v.push(token),
            None => menus.push((top.to_string(), vec![token])),
        };
        let mut opened: Vec<(String, String)> = Vec::new();
        let mut open = |top: &str, sub: &str, push: &mut dyn FnMut(&str, String)| {
            let k = (top.to_string(), sub.to_string());
            if !opened.contains(&k) {
                opened.push(k);
                push(top, format!(">{sub}"));
            }
        };
        for i in 0..=entries.len() {
            for (_, path, what) in marks.iter().filter(|(at, _, _)| *at == i) {
                match (what, path.as_slice()) {
                    (None, [top]) => push(top, "|".into()),
                    (Some(_), [top, sub]) => open(top, sub, &mut push),
                    _ => {}
                }
            }
            let Some(e) = entries.get(i) else { break };
            match e.path.as_slice() {
                [top] => push(top, e.key.clone()),
                [top, sub, ..] => open(top, sub, &mut push),
                [] => {}
            }
        }
        // Every submenu too, one line each ("OUTLINE2 menu > submenu: ...").
        let mut subs: Vec<(Vec<String>, Vec<String>)> = Vec::new();
        let mut put = |at: &[String], token: String| match subs.iter_mut().find(|(p, _)| p == at) {
            Some((_, v)) => {
                if !(token.starts_with('>') && v.contains(&token)) {
                    v.push(token)
                }
            }
            None => subs.push((at.to_vec(), vec![token])),
        };
        for i in 0..=entries.len() {
            for (_, path, what) in marks.iter().filter(|(at, _, _)| *at == i) {
                match what {
                    None if path.len() >= 2 => put(path, "|".into()),
                    Some(s) if path.len() >= 3 => put(&path[..path.len() - 1], format!(">{s}")),
                    _ => {}
                }
            }
            let Some(e) = entries.get(i) else { break };
            if e.path.len() >= 2 {
                put(&e.path, e.key.clone());
            }
        }
        // The bitmap effect groups list ready labels: their effects (the
        // walk saw only their separators).
        let groups = crate::fx::groups();
        subs.retain(|(p, _)| !(p.len() == 2 && p[0] == "menu.effects" && groups.iter().any(|(k, _)| p[1] == *k)));
        let mut put = |at: &[String], token: String| match subs.iter_mut().find(|(p, _)| p == at) {
            Some((_, v)) => v.push(token),
            None => subs.push((at.to_vec(), vec![token])),
        };
        for (key, group) in groups {
            let at = ["menu.effects".to_string(), key.to_string()];
            for fx in group.iter() {
                if crate::fx::separated_before(fx.id) {
                    put(&at, "|".into());
                }
                put(&at, format!("fx.{}", fx.id));
            }
        }
        for (path, tokens) in &subs {
            println!("OUTLINE2 {}: {}", path.join(" > "), tokens.join(" "));
        }
        // Submenus the reference's menu configuration settles.
        let sub = |path: &str| {
            subs.iter()
                .find(|(p, _)| p.join(" > ") == path)
                .map(|(_, v)| v.join(" "))
                .unwrap_or_else(|| panic!("no submenu {path}"))
        };
        for (path, want) in [
            ("menu.file > menu.file.export_for", "menu.file.export_for_office menu.file.export_for_web"),
            ("menu.object > menu.object.order", "menu.object.to_front_of_page menu.object.to_back_of_page | menu.object.to_front_of_layer menu.object.to_back_of_layer menu.object.forward_one menu.object.back_one | menu.object.in_front_of menu.object.behind | menu.object.reverse_order"),
            ("menu.object > menu.object.clip_frame", "menu.object.clip_frame_place_inside menu.object.clip_frame_extract | menu.object.clip_frame_edit menu.object.clip_frame_finish |"),
            ("menu.object > menu.object.perspective", "menu.object.add_perspective | menu.view.draw_in_perspective"),
            ("menu.object > menu.object.copy_effect", "menu.object.copy_perspective_from menu.object.copy_envelope_from menu.object.copy_blend_from menu.object.copy_extrude_from menu.object.copy_contour_from menu.object.copy_lens_from menu.object.copy_shadow_from menu.object.copy_distortion_from"),
            ("menu.effects > menu.effects.adjust", "fx.auto_adjust | fx.image_adjustments | fx.contrast_enhancement fx.target_balance fx.tone_curve | fx.shadows_midtones_highlights fx.brightness_contrast_intensity fx.color_balance fx.gamma | fx.tint_temperature fx.hue_saturation_lightness fx.grayscale fx.vibrance"),
            ("menu.text > menu.text.paragraph_text_frame", "menu.text.frame_fit_text menu.text.insert_placeholder_text | menu.text.frame_create_in_selected | menu.text.frame_link menu.text.frame_unlink"),
        ] {
            let got = sub(path);
            assert!(got.starts_with(want), "{path}: {got}");
        }
        assert!(sub("menu.effects > menu.effects.contour_group").ends_with("fx.local_equalization"));
        assert_eq!(menus.len(), 12, "{:?}", menus.iter().map(|m| &m.0).collect::<Vec<_>>());
        for (top, tokens) in &menus {
            println!("OUTLINE {top}: {}", tokens.join(" "));
            let mut v: Vec<&String> = tokens.iter().collect();
            // A conditional item between two separators may be absent here.
            v.dedup();
            assert_ne!(v.first().map(|s| s.as_str()), Some("|"), "{top} starts with a separator");
        }
        // What the reference's menu configuration settles (DrawUI 27, see
        // `utiles/claude-goal/mapa-menus-referencia.json`).
        let menu = |k: &str| menus.iter().find(|(t, _)| t == k).map(|(_, v)| v.join(" ")).unwrap_or_default();
        let edit = menu("menu.edit");
        let cut_to_delete = &edit[edit.find("menu.edit.cut").expect("cut")..edit.find("menu.edit.delete").expect("delete")];
        assert!(!cut_to_delete.contains('|'), "Edit: Cut to Delete is one group: {edit}");
        assert!(edit.starts_with("menu.edit.undo menu.edit.redo menu.edit.repeat | menu.edit.bring_into_focus menu.edit.exit_focus_mode | menu.edit.cut"), "Edit: focus mode after Repeat: {edit}");
        let object = menu("menu.object");
        let at = |k: &str| object.find(k).unwrap_or_else(|| panic!("{k} in {object}"));
        assert!(at(">menu.object.create") < at(">menu.object.insert") && at(">menu.object.insert") < at(">menu.object.ai_generate"), "Object: Create, Insert, AI Generate: {object}");
        let bitmaps = menu("menu.bitmaps");
        assert!(bitmaps.starts_with("menu.bitmaps.convert_to_bitmap | menu.bitmaps.straighten_image menu.bitmaps.image_adjustment_lab menu.bitmaps.remove_jpeg_artifacts |"), "Bitmaps: {bitmaps}");
        assert!(bitmaps.contains("menu.bitmaps.quick_trace menu.bitmaps.repeat_last_trace menu.bitmaps.last_trace_settings >menu.bitmaps.centerline_trace >menu.bitmaps.outline_trace |"), "Bitmaps: the trace commands at the top level: {bitmaps}");
    }
}
