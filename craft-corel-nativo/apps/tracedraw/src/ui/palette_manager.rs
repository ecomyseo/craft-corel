//! The Palette Manager docker: a toolbar (filter, Open Palette, the
//! document palette from the drawing or the selection), New Palette, the
//! swatch size slider and rows of the selected palette's bar, then the
//! palettes in three sections (Built-in, My Palettes, Document Palette).
//! Each palette is a row with an eye (shown in a bar or not), a Show bar
//! / Hide bar button and the bar's place, its name, a strip of its first
//! colours, and its colour count, colour model and where it shows.
//! A click selects a palette, a double click on its name renames it, the
//! three-dot button or a right click opens its menu (Open as Palette Bar
//! or Close Palette Bar, Edit, Rename, Reset Name, Duplicate, Lock,
//! Delete, Move Up, Move Down); a row dragged onto another of its
//! section moves there.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::settings::{SWATCH_MAX, SWATCH_MIN, SWATCH_STEPS};
use crate::theme::Tokens;
use crate::ui::bars::Dock;
use crate::ui::palette::Row;
use crate::ui::tips::RichTip;
use egui::{pos2, vec2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use tracedraw_core::{Color, Fill};

/// What the Palette Manager has selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selected {
    /// The palette the default bar shows.
    #[default]
    Main,
    /// A palette of the list.
    Palette(usize),
    /// The document palette.
    Document,
}

/// The Palette Manager's own state.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ManagerState {
    pub selected: Selected,
    /// The filter typed: palettes whose name holds it show.
    pub filter: String,
    /// The row being dragged to a new place, by its palette.
    pub dragging: Option<usize>,
}

/// Height of a palette's row.
const ROW_H: f32 = 54.0;
/// The side of a colour of a row's preview strip.
const CHIP: f32 = 12.0;
/// How many colours a row's preview strip shows at most.
const PREVIEW: usize = 16;

/// The palette `sel` stands for, as an index of the list.
fn index_of(app: &App, sel: Selected) -> Option<usize> {
    match sel {
        Selected::Main => app.visible_palettes.first().copied(),
        Selected::Palette(i) => Some(i).filter(|i| *i < app.palettes.len()),
        Selected::Document => None,
    }
}

/// The bar showing `sel`, if it is shown: the default bar, a palette bar,
/// or (the document palette not in a bar of its own) its row under the
/// drawing.
pub fn bar_of(app: &App, sel: Selected) -> Option<Row> {
    if sel == Selected::Document {
        return Some(
            app.extra_palettes
                .iter()
                .position(|r| r.document)
                .map_or(Row::Document, Row::Extra),
        );
    }
    let i = index_of(app, sel)?;
    if app.visible_palettes.first() == Some(&i) {
        return Some(Row::Main);
    }
    app.extra_palettes
        .iter()
        .position(|r| !r.document && r.index == i)
        .map(Row::Extra)
}

/// The swatch side a slider at `t` (0 at the left end, 1 at the right)
/// gives: [`SWATCH_MIN`] to [`SWATCH_MAX`] in the 2 px steps of
/// Ctrl+wheel, taking a named size ([`SWATCH_STEPS`], the ticks) within
/// 1.5 px of it.
pub fn slider_size(t: f32) -> u8 {
    let t = if t.is_finite() {
        t.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (lo, hi) = (f32::from(SWATCH_MIN), f32::from(SWATCH_MAX));
    let raw = lo + t * (hi - lo);
    let even = ((raw / 2.0).round() * 2.0).clamp(lo, hi);
    let near = SWATCH_STEPS
        .iter()
        .map(|(s, _)| f32::from(*s))
        .find(|s| (s - raw).abs() < 1.5);
    near.unwrap_or(even) as u8
}

/// Where on the slider (0 to 1) the side `size` lies.
pub fn slider_t(size: u8) -> f32 {
    let (lo, hi) = (f32::from(SWATCH_MIN), f32::from(SWATCH_MAX));
    ((f32::from(size) - lo) / (hi - lo)).clamp(0.0, 1.0)
}

/// The id the slider's track keeps its place under (for tests).
pub fn slider_id() -> egui::Id {
    egui::Id::new("palette_manager_size_slider")
}

/// The colour model a palette's colours share: CMYK, RGB, Grey, Spot or
/// Mixed (an i18n key).
pub fn model_key(colors: &[(String, Color)]) -> &'static str {
    let kind = |c: &Color| match c {
        Color::Cmyk { .. } => 0,
        Color::Rgb { .. } => 1,
        Color::Gray { .. } => 2,
        Color::Spot { .. } => 3,
        _ => 4,
    };
    let mut kinds = colors.iter().map(|(_, c)| kind(c));
    let Some(first) = kinds.next() else {
        return "palette.model_none";
    };
    if kinds.any(|k| k != first) {
        return "palette.model_mixed";
    }
    match first {
        0 => "palette.model_cmyk",
        1 => "palette.model_rgb",
        2 => "palette.model_gray",
        3 => "palette.model_spot",
        _ => "palette.model_mixed",
    }
}

/// Where `sel` shows, in words.
/// The short name of a bar's place (Top, Bottom, Left, Right, Floating).
fn place_key(dock: Dock) -> &'static str {
    match dock {
        Dock::Top => "palette.edge_top",
        Dock::Bottom => "palette.edge_bottom",
        Dock::Left => "palette.edge_left",
        Dock::Right => "palette.edge_right",
        Dock::Floating => "palette.float",
    }
}

fn shown_text(app: &App, sel: Selected) -> String {
    let place = |dock: Dock| tr(place_key(dock));
    match bar_of(app, sel) {
        Some(Row::Main) => trf(
            "palette.shown_default",
            &[(
                "place",
                &place(app.settings.bars.dock(crate::ui::bars::Bar::Palette)),
            )],
        ),
        Some(Row::Document) if app.settings.palette.show_document => {
            tr("palette.shown_under_drawing")
        }
        Some(Row::Document) => tr("palette.hidden"),
        Some(Row::Extra(k)) => {
            let dock = app
                .settings
                .palette_bars
                .get(k)
                .map_or(Dock::Bottom, |b| b.dock);
            trf("palette.shown_at", &[("place", &place(dock))])
        }
        None => tr("palette.hidden"),
    }
}

/// The Palette Manager.
pub fn show(app: &mut App, ui: &mut Ui) {
    ui.spacing_mut().item_spacing.y = 4.0;
    toolbar(app, ui);
    // New Palette: an empty palette of one's own, shown in a bar of its
    // own at once, so several palettes stand in bars side by side.
    if ui
        .button(tr("palette.new"))
        .rich_tip(tr("palette.new_tip"))
        .clicked()
    {
        crate::ui::palette_bars::start_new_palette(app);
    }
    ui.add_space(2.0);
    size_controls(app, ui);
    ui.add_space(4.0);
    egui::ScrollArea::vertical()
        .id_salt("palette_manager_list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let filter = app.palette_manager.filter.trim().to_lowercase();
            let fits = |app: &App, i: usize| {
                filter.is_empty() || app.palette_name(i).to_lowercase().contains(&filter)
            };
            let builtin: Vec<usize> = (0..app.palettes.len())
                .filter(|i| app.palettes[*i].builtin && fits(app, *i))
                .collect();
            let mine: Vec<usize> = (0..app.palettes.len())
                .filter(|i| !app.palettes[*i].builtin && fits(app, *i))
                .collect();
            section(ui, &tr("palette.section_builtin"));
            for i in builtin {
                palette_row(app, ui, i);
            }
            section(ui, &tr("palette.section_mine"));
            if mine.is_empty() {
                ui.label(
                    egui::RichText::new(tr("palette.mine_empty"))
                        .small()
                        .color(Tokens::text_dim()),
                );
            }
            for i in mine {
                palette_row(app, ui, i);
            }
            section(ui, &tr("palette.section_document"));
            document_section(app, ui);
        });
    // A row let go off every row stays where it was.
    if app.palette_manager.dragging.is_some() && ui.input(|i| !i.pointer.any_down()) {
        app.palette_manager.dragging = None;
    }
}

/// A section's title: its name and a line.
fn section(ui: &mut Ui, text: &str) {
    ui.add_space(6.0);
    crate::ui::chrome::section(ui, text);
}

/// The toolbar: the filter, then Open Palette and the document palette
/// from the drawing or from the selection.
fn toolbar(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 3.0 * 28.0 - 8.0).max(60.0);
        ui.add(
            egui::TextEdit::singleline(&mut app.palette_manager.filter)
                .hint_text(tr("palette.filter_hint"))
                .desired_width(w),
        )
        .rich_tip(tr("palette.filter_tip"));
        if tool_button(ui, Pic::Folder, true)
            .rich_tip(crate::ui::tips::keyed("menu.window.palette_open"))
            .clicked()
        {
            app.open_palette_file();
        }
        let doc = app.has_document();
        if tool_button(ui, Pic::FromDrawing, doc)
            .rich_tip(crate::ui::tips::keyed("menu.window.palette_from_document"))
            .clicked()
            && doc
        {
            app.palette_from_document();
        }
        let sel = doc && !app.selection.is_empty();
        if tool_button(ui, Pic::FromSelection, sel)
            .rich_tip(crate::ui::tips::keyed("menu.window.palette_from_selection"))
            .clicked()
            && sel
        {
            app.palette_from_selection();
        }
    });
}

/// The pictures of the Palette Manager's buttons.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Pic {
    Folder,
    FromDrawing,
    FromSelection,
    More,
}

/// A 26 px square button with a picture.
fn tool_button(ui: &mut Ui, pic: Pic, enabled: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::splat(26.0),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if enabled && resp.hovered() {
        ui.painter().rect(
            rect.shrink(1.0),
            3.0,
            Tokens::tool_hover(),
            Stroke::new(1.0, Tokens::ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    let color = if enabled {
        Tokens::icon()
    } else {
        Tokens::icon_off()
    };
    paint_pic(ui.painter(), rect.shrink(6.0), pic, color);
    resp
}

/// Paint `pic` in `r` (about 14 px), crisp on whole pixels.
fn paint_pic(p: &egui::Painter, r: Rect, pic: Pic, color: Color32) {
    let s = Stroke::new(1.2, color);
    let at = |x: f32, y: f32| pos2((r.left() + x).round() + 0.5, (r.top() + y).round() + 0.5);
    // A palette's three swatches, at the bottom right.
    let badge = |p: &egui::Painter| {
        for (k, c) in [
            Color32::from_rgb(0xE5, 0x39, 0x35),
            Color32::from_rgb(0x43, 0xA0, 0x47),
            Color32::from_rgb(0x1E, 0x88, 0xE5),
        ]
        .into_iter()
        .enumerate()
        {
            let x = r.left() + 6.0 + 3.0 * k as f32;
            p.rect_filled(
                Rect::from_min_size(pos2(x.round(), (r.bottom() - 4.0).round()), vec2(3.0, 4.0)),
                0.0,
                c,
            );
        }
    };
    match pic {
        Pic::Folder => {
            let pts = vec![
                at(0.0, 2.0),
                at(5.0, 2.0),
                at(6.5, 4.0),
                at(13.0, 4.0),
                at(13.0, 12.0),
                at(0.0, 12.0),
            ];
            p.add(egui::Shape::closed_line(pts, s));
            p.line_segment([at(0.0, 6.0), at(13.0, 6.0)], s);
        }
        Pic::FromDrawing => {
            // A page with a corner folded, and the swatches.
            let pts = vec![
                at(1.0, 0.0),
                at(8.0, 0.0),
                at(11.0, 3.0),
                at(11.0, 9.0),
                at(1.0, 9.0),
            ];
            p.add(egui::Shape::closed_line(pts, s));
            badge(p);
        }
        Pic::FromSelection => {
            // A dashed frame round a shape, and the swatches.
            let frame = Rect::from_min_max(at(0.0, 0.0), at(11.0, 9.0));
            for (a, b) in [
                (frame.left_top(), frame.right_top()),
                (frame.right_top(), frame.right_bottom()),
                (frame.right_bottom(), frame.left_bottom()),
                (frame.left_bottom(), frame.left_top()),
            ] {
                p.extend(egui::Shape::dashed_line(&[a, b], s, 2.0, 1.5));
            }
            p.circle_stroke(frame.center(), 2.5, s);
            badge(p);
        }
        Pic::More => {
            for k in -1..=1 {
                p.circle_filled(r.center() + vec2(4.5 * k as f32, 0.0), 1.4, color);
            }
        }
    }
}

/// The swatch size slider and the rows of the selected palette's bar,
/// as the palette menu and Ctrl+wheel set them.
fn size_controls(app: &mut App, ui: &mut Ui) {
    let sel = app.palette_manager.selected;
    let bar = bar_of(app, sel).filter(|b| *b != Row::Document || sel == Selected::Document);
    let name = match sel {
        Selected::Document => app.document_palette_name(),
        _ => index_of(app, sel)
            .map(|i| app.palette_name(i))
            .unwrap_or_default(),
    };
    let frame = egui::Frame::new()
        .fill(Tokens::field())
        .stroke(Stroke::new(1.0, Tokens::border()))
        .corner_radius(3.0)
        .inner_margin(egui::Margin::symmetric(8, 6));
    frame.show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(tr("palette.size")).strong());
            ui.label(
                egui::RichText::new(match bar {
                    Some(_) => name.clone(),
                    None => tr("palette.not_shown_hint"),
                })
                .small()
                .color(Tokens::text_dim()),
            );
        });
        let size = bar.map(|b| crate::ui::palette::strip_prefs(app, b).swatch);
        if let Some(new) = size_slider(
            ui,
            size.unwrap_or(crate::settings::SWATCH_DEFAULT),
            bar.is_some(),
        ) {
            if let Some(b) = bar {
                crate::ui::palette_bars::set_swatch(app, b, new);
            }
        }
        ui.horizontal(|ui| {
            let label = match size {
                Some(s) => match crate::settings::swatch_step_of(s) {
                    Some(i) => format!("{} ({s} px)", tr(SWATCH_STEPS[i].1)),
                    None => trf("palette.size_custom", &[("n", &s.to_string())]),
                },
                None => String::from("-"),
            };
            ui.label(egui::RichText::new(label).color(Tokens::text_dim()));
            // The rows of a bar of its own (the document palette's row
            // under the drawing has one).
            if let Some(b) = bar.filter(|b| *b != Row::Document) {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let prefs = crate::ui::palette::strip_prefs(app, b);
                    let floating = prefs.dock == Dock::Floating;
                    let now = if floating {
                        crate::ui::palette::lines_of(app, b) as u8
                    } else {
                        prefs.rows
                    };
                    let text = |n: u8| {
                        if n == crate::settings::ROWS_FIT {
                            tr("palette.rows_fit")
                        } else {
                            tr(&format!("palette.rows_{n}"))
                        }
                    };
                    crate::ui::keynav::ComboBox::from_id_salt("palette_manager_rows")
                        .selected_text(text(now))
                        .width(84.0)
                        .show_ui(ui, |ui| {
                            for n in (1..=crate::settings::MAX_ROWS)
                                .chain(std::iter::once(crate::settings::ROWS_FIT))
                            {
                                if ui.selectable_label(now == n, text(n)).clicked() {
                                    crate::ui::palette_bars::set_rows(app, b, n);
                                }
                            }
                        })
                        .response
                        .rich_tip(tr("palette.rows_tip"));
                    ui.label(tr("palette.rows_of_selected_bar"));
                });
            }
        });
    });
}

/// The slider: a small swatch at the left, a track with a tick at each
/// named size, a big swatch at the right. Returns the size dragged or
/// clicked to.
fn size_slider(ui: &mut Ui, size: u8, enabled: bool) -> Option<u8> {
    let height = 24.0;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let p = ui.painter();
    let color = if enabled {
        Tokens::icon()
    } else {
        Tokens::icon_off()
    };
    let small = Rect::from_center_size(pos2(row.left() + 6.0, row.center().y), Vec2::splat(7.0));
    let big = Rect::from_center_size(pos2(row.right() - 9.0, row.center().y), Vec2::splat(15.0));
    for r in [small, big] {
        let r = Rect::from_min_size(r.min.round(), r.size());
        p.rect_filled(r, 1.0, Color32::from_rgb(0x4F, 0x9D, 0xE6));
        p.rect_stroke(r, 1.0, Stroke::new(1.0, color), egui::StrokeKind::Inside);
    }
    let track = Rect::from_min_max(
        pos2(small.right() + 10.0, row.top()),
        pos2(big.left() - 10.0, row.bottom()),
    );
    let resp = ui.interact(
        track,
        slider_id(),
        if enabled {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        },
    );
    ui.ctx().data_mut(|d| d.insert_temp(slider_id(), track));
    let y = (track.center().y - 3.0).round() + 0.5;
    let x_of = |t: f32| track.left() + t * track.width();
    let p = ui.painter();
    p.line_segment(
        [pos2(track.left(), y), pos2(track.right(), y)],
        Stroke::new(2.0, Tokens::border()),
    );
    let t = slider_t(size);
    if enabled {
        p.line_segment(
            [pos2(track.left(), y), pos2(x_of(t), y)],
            Stroke::new(2.0, Tokens::ACCENT),
        );
    }
    for (s, _) in SWATCH_STEPS {
        let x = x_of(slider_t(s)).round() + 0.5;
        p.line_segment(
            [pos2(x, y + 4.0), pos2(x, y + 8.0)],
            Stroke::new(1.0, Tokens::text_dim()),
        );
    }
    let knob = pos2(x_of(t), y);
    p.circle(
        knob,
        5.5,
        if enabled {
            Tokens::panel()
        } else {
            Tokens::field()
        },
        Stroke::new(
            1.5,
            if enabled && (resp.hovered() || resp.dragged()) {
                Tokens::ACCENT
            } else {
                color
            },
        ),
    );
    let resp = resp.rich_tip(tr("palette.size_slider_tip"));
    if !enabled {
        return None;
    }
    let pointer = resp.interact_pointer_pos();
    match pointer {
        Some(at) if resp.dragged() || resp.clicked() || resp.drag_started() => {
            let new = slider_size((at.x - track.left()) / track.width().max(1.0));
            (new != size).then_some(new)
        }
        _ => None,
    }
}

/// A palette's row.
fn palette_row(app: &mut App, ui: &mut Ui, i: usize) {
    let sel = if app.visible_palettes.first() == Some(&i) {
        Selected::Main
    } else {
        Selected::Palette(i)
    };
    let selected = index_of(app, app.palette_manager.selected) == Some(i);
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click_and_drag());
    let tip_name = app.palette_name(i);
    let resp = resp.rich_tip_with(|| format!("{tip_name}\n{}", tr("palette.row_tip")));
    let id = egui::Id::new(("palette_manager_row", i));
    ui.ctx().data_mut(|d| d.insert_temp(id, rect));
    let p = ui.painter();
    let dragging_here =
        app.palette_manager.dragging.is_some_and(|d| d != i) && ui.rect_contains_pointer(rect);
    if selected {
        p.rect_filled(rect, 3.0, Tokens::tool_active());
    } else if resp.hovered() {
        p.rect_filled(rect, 3.0, Tokens::tool_hover());
    }
    if dragging_here {
        p.line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(2.0, Tokens::ACCENT),
        );
    }
    let shown = app.visible_palettes.contains(&i);
    // The eye.
    let eye = Rect::from_min_size(rect.min + vec2(2.0, 4.0), vec2(22.0, 20.0));
    let eye_resp = ui
        .interact(eye, id.with("eye"), Sense::click())
        .rich_tip(tr(if shown {
            "palette.hide_tip"
        } else {
            "palette.show_tip"
        }));
    crate::ui::objects_docker::paint_eye(ui, eye, shown);
    if eye_resp.clicked() {
        app.toggle_palette(i);
    }
    let name = app.palette_name(i);
    let (count, model) = {
        let colors = &app.palettes[i].colors;
        (colors.len(), model_key(colors))
    };
    let left = eye.right() + 4.0;
    let more = Rect::from_min_size(
        pos2(rect.right() - 26.0, rect.top() + 2.0),
        Vec2::splat(24.0),
    );
    let controls = bar_controls(app, ui, i, sel, shown, rect, more.left() - 4.0);
    let p = ui.painter();
    let name_galley = p.layout_no_wrap(name.clone(), crate::theme::bold(12.0), Tokens::text());
    let name_rect = Rect::from_min_size(pos2(left, rect.top() + 5.0), name_galley.size());
    p.galley_with_override_text_color(name_rect.min, name_galley, Tokens::text());
    if app.palette_locked(i) {
        let r = Rect::from_center_size(
            pos2(name_rect.right() + 9.0, name_rect.center().y),
            Vec2::splat(12.0),
        );
        crate::ui::objects_docker::paint_padlock(ui.painter(), r, true, Tokens::text_dim());
    }
    // The first colours.
    let strip_top = rect.top() + 22.0;
    let room = (controls - left - 4.0).max(0.0);
    let n = (((room + 1.0) / (CHIP + 1.0)).floor() as usize).min(PREVIEW);
    let p = ui.painter();
    for (k, (_, c)) in app.palettes[i].colors.iter().take(n).enumerate() {
        let r = Rect::from_min_size(
            pos2((left + k as f32 * (CHIP + 1.0)).round(), strip_top.round()),
            Vec2::splat(CHIP),
        );
        p.rect_filled(r, 0.0, crate::canvas::to_color32(*c));
        p.rect_stroke(
            r,
            0.0,
            Stroke::new(1.0, Color32::from_black_alpha(60)),
            egui::StrokeKind::Inside,
        );
    }
    let meta = format!(
        "{} \u{00B7} {} \u{00B7} {}",
        trf("palette.count", &[("n", &count.to_string())]),
        tr(model),
        shown_text(app, sel)
    );
    let meta_clip = Rect::from_min_max(pos2(left, rect.top()), pos2(controls - 2.0, rect.bottom()));
    p.with_clip_rect(meta_clip.intersect(p.clip_rect())).text(
        pos2(left, rect.bottom() - 9.0),
        egui::Align2::LEFT_CENTER,
        meta,
        egui::FontId::proportional(10.5),
        Tokens::text_dim(),
    );
    // The three-dot button.
    let more_resp = ui.interact(more, id.with("more"), Sense::click());
    if more_resp.hovered() {
        ui.painter()
            .rect_filled(more.shrink(2.0), 3.0, Tokens::tool_hover());
    }
    paint_pic(ui.painter(), more.shrink(5.0), Pic::More, Tokens::icon());
    let more_resp = more_resp.rich_tip(tr("palette.actions_tip"));
    egui::Popup::menu(&more_resp)
        .id(id.with("more_menu"))
        .style(crate::ui::menus::menu_popup_style)
        .show(|ui| crate::ui::menus::body(ui, |ui| row_menu(app, ui, i)));
    egui::Popup::context_menu(&resp)
        .id(id.with("context_menu"))
        .style(crate::ui::menus::menu_popup_style)
        .show(|ui| crate::ui::menus::body(ui, |ui| row_menu(app, ui, i)));
    if resp.clicked() || resp.secondary_clicked() {
        app.palette_manager.selected = Selected::Palette(i);
    }
    if resp.double_clicked()
        && name_rect
            .expand(4.0)
            .contains(resp.interact_pointer_pos().unwrap_or(Pos2::ZERO))
    {
        start_rename_index(app, i);
    }
    // Dragging a row onto another of its section moves it there.
    if resp.drag_started() {
        app.palette_manager.dragging = Some(i);
    }
    if resp.drag_stopped() {
        if let Some(from) = app.palette_manager.dragging.take() {
            let to = ui
                .ctx()
                .pointer_latest_pos()
                .and_then(|at| row_under(app, ui.ctx(), at));
            if let Some(to) = to {
                if app.palettes.get(from).map(|p| p.builtin)
                    == app.palettes.get(to).map(|p| p.builtin)
                {
                    app.move_palette_in_list(from, to);
                    app.palette_manager.selected = Selected::Palette(to);
                }
            }
        }
    }
    let _ = rect;
}

/// Width of a palette row's Show bar / Hide bar button and place list.
const BAR_CONTROLS_W: f32 = 92.0;

/// A palette row's Show bar / Hide bar button and, while its bar shows,
/// the list of where the bar is (Top, Bottom, Left, Right, Floating),
/// right-aligned up to `right`. Returns their left edge.
fn bar_controls(
    app: &mut App,
    ui: &mut Ui,
    i: usize,
    sel: Selected,
    shown: bool,
    rect: Rect,
    right: f32,
) -> f32 {
    let left = right - BAR_CONTROLS_W;
    let id = egui::Id::new(("palette_manager_bar", i));
    // The last palette shown keeps its bar.
    let can = !shown || app.visible_palettes.len() > 1;
    let button = Rect::from_min_size(pos2(left, rect.top() + 4.0), vec2(BAR_CONTROLS_W, 20.0));
    let label = tr(if shown {
        "palette.hide_bar"
    } else {
        "palette.show_bar"
    });
    // In a child of its own: the row's layout stays as it is.
    let mut at = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(button)
            .id_salt(id.with("btn")),
    );
    let resp = at
        .add_sized(
            button.size(),
            egui::Button::new(egui::RichText::new(label).size(11.0))
                .truncate()
                .selected(shown),
        )
        .rich_tip(tr(if shown {
            "palette.hide_bar_tip"
        } else {
            "palette.show_bar_tip"
        }));
    if resp.clicked() && can {
        app.toggle_palette(i);
        return left;
    }
    if let Some(bar) = bar_of(app, sel).filter(|_| shown) {
        let now = crate::ui::palette::strip_prefs(app, bar).dock;
        let name = |dock: Dock| tr(place_key(dock));
        let place = Rect::from_min_size(pos2(left, rect.top() + 28.0), vec2(BAR_CONTROLS_W, 20.0));
        let locked = app.settings.bars.locked;
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(place).id_salt(id));
        child.add_enabled_ui(!locked, |ui| {
            crate::ui::keynav::ComboBox::from_id_salt(id.with("place"))
                .selected_text(egui::RichText::new(name(now)).size(11.0))
                .width(BAR_CONTROLS_W - 4.0)
                .show_ui(ui, |ui| {
                    for (dock, _) in crate::ui::palette_bars::PLACES {
                        if ui.selectable_label(now == dock, name(dock)).clicked() {
                            crate::ui::palette_bars::set_dock(app, bar, dock, None);
                        }
                    }
                })
                .response
                .rich_tip(tr("palette.bar_place_tip"));
        });
    }
    left
}

/// The palette whose row lies under `at`.
fn row_under(app: &App, ctx: &egui::Context, at: Pos2) -> Option<usize> {
    (0..app.palettes.len()).find(|i| {
        ctx.data(|d| d.get_temp::<Rect>(egui::Id::new(("palette_manager_row", *i))))
            .is_some_and(|r| r.contains(at))
    })
}

fn start_rename_index(app: &mut App, i: usize) {
    if let Some(p) = app.palettes.get(i) {
        app.palette_rename = Some((crate::settings::palette_id(p), app.palette_name(i)));
    }
}

/// A palette's menu in the Palette Manager.
fn row_menu(app: &mut App, ui: &mut Ui, i: usize) {
    use crate::ui::menus::{item, sep};
    ui.set_min_width(200.0);
    let Some(p) = app.palettes.get(i) else {
        return;
    };
    let id = crate::settings::palette_id(p);
    let builtin = p.builtin;
    let shown = app.visible_palettes.contains(&i);
    let renamed = app.settings.palette_names.contains_key(&id);
    if !shown {
        if item(ui, "palette.open_as_bar", "", true) {
            app.open_palette_bar(&id);
        }
    } else if item(ui, "palette.close_bar", "", app.visible_palettes.len() > 1) {
        app.toggle_palette(i);
    }
    sep(ui);
    if item(ui, "docker.edit", "", true) {
        app.dialog =
            crate::ui::dialogs::Dialog::PaletteEditor(crate::ui::dialogs::PaletteEditorState {
                palette: i,
                ..Default::default()
            });
    }
    if item(ui, "palette.rename", "", true) {
        start_rename_index(app, i);
    }
    if item(ui, "palette.reset_name", "", renamed) {
        app.rename_palette(&id, "");
    }
    if item(ui, "palette.duplicate", "", true) {
        if let Some(k) = app.duplicate_palette(i) {
            app.palette_manager.selected = Selected::Palette(k);
        }
    }
    if !builtin {
        let locked = app.palette_locked(i);
        if item(
            ui,
            if locked {
                "docker.palette_unlock"
            } else {
                "docker.palette_lock"
            },
            "",
            true,
        ) {
            app.toggle_palette_lock(i);
        }
        if item(ui, "palette.delete", "", true) {
            app.delete_palette(i);
            app.palette_manager.selected = Selected::Main;
        }
    }
    sep(ui);
    let same = |k: usize| app.palettes.get(k).is_some_and(|q| q.builtin == builtin);
    let up = i.checked_sub(1).filter(|k| same(*k));
    let down = Some(i + 1).filter(|k| same(*k));
    if item(ui, "palette.move_up", "", up.is_some()) {
        if let Some(k) = up {
            app.move_palette_in_list(i, k);
            app.palette_manager.selected = Selected::Palette(k);
        }
    }
    if item(ui, "palette.move_down", "", down.is_some()) {
        if let Some(k) = down {
            app.move_palette_in_list(i, k);
            app.palette_manager.selected = Selected::Palette(k);
        }
    }
}

/// The document palette: its row (eye for its row under the drawing,
/// name, count, where it shows, menu), its colours, and Add Selection
/// Colors, Add Document Colors and Clear.
fn document_section(app: &mut App, ui: &mut Ui) {
    let selected = app.palette_manager.selected == Selected::Document;
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::click());
    let resp = resp.rich_tip(tr("palette.document_row_tip"));
    let id = egui::Id::new("palette_manager_document");
    if selected {
        ui.painter().rect_filled(rect, 3.0, Tokens::tool_active());
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 3.0, Tokens::tool_hover());
    }
    let shown = app.settings.palette.show_document;
    let eye = Rect::from_min_size(rect.min + vec2(2.0, 4.0), vec2(22.0, 20.0));
    let eye_resp = ui
        .interact(eye, id.with("eye"), Sense::click())
        .rich_tip(tr("palette.document_eye_tip"));
    crate::ui::objects_docker::paint_eye(ui, eye, shown);
    if eye_resp.clicked() {
        app.settings.palette.show_document = !shown;
    }
    let colors: Vec<Color> = if app.has_document() {
        app.doc().palette.clone()
    } else {
        Vec::new()
    };
    let name = app.document_palette_name();
    let left = eye.right() + 4.0;
    let p = ui.painter();
    let galley = p.layout_no_wrap(name, crate::theme::bold(12.0), Tokens::text());
    let name_rect = Rect::from_min_size(pos2(left, rect.top() + 5.0), galley.size());
    p.galley_with_override_text_color(name_rect.min, galley, Tokens::text());
    p.text(
        pos2(left, rect.bottom() - 8.0),
        egui::Align2::LEFT_CENTER,
        format!(
            "{} \u{00B7} {}",
            trf("palette.count", &[("n", &colors.len().to_string())]),
            shown_text(app, Selected::Document)
        ),
        egui::FontId::proportional(10.5),
        Tokens::text_dim(),
    );
    let more = Rect::from_min_size(
        pos2(rect.right() - 26.0, rect.top() + 2.0),
        Vec2::splat(24.0),
    );
    let more_resp = ui.interact(more, id.with("more"), Sense::click());
    if more_resp.hovered() {
        ui.painter()
            .rect_filled(more.shrink(2.0), 3.0, Tokens::tool_hover());
    }
    paint_pic(ui.painter(), more.shrink(5.0), Pic::More, Tokens::icon());
    let more_resp = more_resp.rich_tip(tr("palette.actions_tip"));
    egui::Popup::menu(&more_resp)
        .id(id.with("more_menu"))
        .style(crate::ui::menus::menu_popup_style)
        .show(|ui| crate::ui::menus::body(ui, |ui| document_menu(app, ui)));
    egui::Popup::context_menu(&resp)
        .id(id.with("context_menu"))
        .style(crate::ui::menus::menu_popup_style)
        .show(|ui| crate::ui::menus::body(ui, |ui| document_menu(app, ui)));
    if resp.clicked() || resp.secondary_clicked() {
        app.palette_manager.selected = Selected::Document;
    }
    if resp.double_clicked() {
        crate::ui::palette_bars::start_rename(app, Row::Document);
    }
    // Its colours: a click fills, a right click colours the outline.
    if colors.is_empty() {
        ui.label(
            egui::RichText::new(tr("palette.document_hint"))
                .small()
                .color(Tokens::text_dim()),
        );
    } else {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(1.0, 1.0);
            for c in colors {
                let (r, resp) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::click());
                let resp = resp.rich_tip_with(|| {
                    format!(
                        "{}\n{}",
                        crate::app::color_description(c),
                        tr("docker.palette_chip_desc")
                    )
                });
                ui.painter()
                    .rect_filled(r, 0.0, crate::canvas::to_color32(c));
                ui.painter().rect_stroke(
                    r,
                    0.0,
                    Stroke::new(1.0, Color32::from_black_alpha(60)),
                    egui::StrokeKind::Inside,
                );
                if resp.clicked() {
                    app.apply_fill(Fill::Solid(c));
                }
                if resp.secondary_clicked() {
                    app.apply_outline_color(Some(c));
                }
            }
        });
    }
    let doc = app.has_document();
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(
                doc && !app.selection.is_empty(),
                egui::Button::new(tr("palette.add_selection_colors")),
            )
            .rich_tip(tr("palette.add_selection_colors_tip"))
            .clicked()
        {
            app.palette_from_selection();
        }
        if ui
            .add_enabled(doc, egui::Button::new(tr("palette.add_from_document")))
            .rich_tip(tr("palette.add_from_document_tip"))
            .clicked()
        {
            app.add_document_colors_to_palette();
        }
        let any = doc && !app.doc().palette.is_empty();
        if ui
            .add_enabled(any, egui::Button::new(tr("palette.clear")))
            .rich_tip(tr("palette.clear_tip"))
            .clicked()
        {
            app.clear_document_palette();
        }
    });
}

/// The document palette's menu in the Palette Manager.
fn document_menu(app: &mut App, ui: &mut Ui) {
    use crate::ui::menus::{item, sep};
    ui.set_min_width(200.0);
    let bar = app.extra_palettes.iter().position(|r| r.document);
    match bar {
        None => {
            if item(ui, "palette.open_as_bar", "", true) {
                app.open_palette_bar(crate::settings::DOCUMENT_PALETTE);
            }
        }
        Some(k) => {
            if item(ui, "palette.close_bar", "", true) {
                crate::ui::palette_bars::close_bar(app, k);
            }
        }
    }
    sep(ui);
    if item(ui, "palette.rename", "", true) {
        crate::ui::palette_bars::start_rename(app, Row::Document);
    }
    let renamed = app
        .settings
        .palette_names
        .contains_key(crate::settings::DOCUMENT_PALETTE);
    if item(ui, "palette.reset_name", "", renamed) {
        app.rename_palette(crate::settings::DOCUMENT_PALETTE, "");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slider_steps_as_ctrl_wheel_and_takes_the_named_sizes() {
        assert_eq!(slider_size(0.0), SWATCH_MIN);
        assert_eq!(slider_size(1.0), SWATCH_MAX);
        assert_eq!(slider_size(-3.0), SWATCH_MIN);
        assert_eq!(slider_size(f32::NAN), SWATCH_MIN);
        for (s, _) in SWATCH_STEPS {
            assert_eq!(slider_size(slider_t(s)), s);
            // Near a tick it takes the tick.
            assert_eq!(slider_size(slider_t(s) + 0.02), s);
        }
        // Between ticks, the even sizes of Ctrl+wheel.
        let mid = slider_size(slider_t(34));
        assert_eq!(mid, 34);
        assert!((SWATCH_MIN..=SWATCH_MAX).all(|v| slider_size(slider_t(v)).is_multiple_of(2)));
    }

    /// One frame of the Palette Manager with `events`.
    fn run(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
        clock: &std::cell::Cell<f64>,
    ) {
        clock.set(clock.get() + 0.05);
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(360.0, 900.0))),
            events,
            time: Some(clock.get()),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| show(app, ui));
        out.textures_delta.clear();
    }

    fn click(ctx: &egui::Context, app: &mut App, at: Pos2, clock: &std::cell::Cell<f64>) {
        let b = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        run(ctx, app, vec![egui::Event::PointerMoved(at)], clock);
        run(ctx, app, vec![b(true)], clock);
        run(ctx, app, vec![b(false)], clock);
        clock.set(clock.get() + 1.0);
        run(ctx, app, Vec::new(), clock);
    }

    /// The slider sizes the swatches of the selected palette's bar live:
    /// the default bar at first, a palette bar once its row is picked.
    #[test]
    fn the_slider_sizes_the_selected_palette_s_bar() {
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let mut app = App::headless();
        run(&ctx, &mut app, Vec::new(), &clock);
        let track = ctx
            .data(|d| d.get_temp::<Rect>(slider_id()))
            .expect("the slider");
        let x = |s: u8| track.left() + slider_t(s) * track.width();
        click(&ctx, &mut app, pos2(x(28), track.center().y), &clock);
        assert_eq!(app.settings.palette.swatch_size, 28);
        // A drag to the right end: the largest.
        let start = pos2(x(28), track.center().y);
        let b = |at, pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        run(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start)],
            &clock,
        );
        run(&ctx, &mut app, vec![b(start, true)], &clock);
        for k in 1..=10 {
            let p = start + vec2((track.right() + 20.0 - start.x) * k as f32 / 10.0, 0.0);
            run(&ctx, &mut app, vec![egui::Event::PointerMoved(p)], &clock);
        }
        run(
            &ctx,
            &mut app,
            vec![b(pos2(track.right() + 20.0, start.y), false)],
            &clock,
        );
        assert_eq!(app.settings.palette.swatch_size, SWATCH_MAX);
        // A palette bar selected: the slider sizes it, not the default.
        let gray = app
            .palettes
            .iter()
            .position(|p| p.key == "palette.grayscale")
            .expect("grayscale");
        let id = crate::settings::palette_id(&app.palettes[gray]);
        app.open_palette_bar(&id);
        app.palette_manager.selected = Selected::Palette(gray);
        run(&ctx, &mut app, Vec::new(), &clock);
        click(&ctx, &mut app, pos2(x(14), track.center().y), &clock);
        assert_eq!(app.settings.palette_bars[0].swatch_size, 14);
        assert_eq!(app.settings.palette.swatch_size, SWATCH_MAX);
    }

    /// The list's eye opens a palette in a bar and closes it; its actions
    /// duplicate, rename, move and delete palettes.
    /// Runs the New Palette window over `frames` frames of `events`.
    fn new_palette_frame(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
        clock: &std::cell::Cell<f64>,
    ) {
        clock.set(clock.get() + 0.05);
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1000.0, 800.0))),
            events,
            time: Some(clock.get()),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| {
            crate::ui::palette_bars::new_palette_window(app, ui.ctx())
        });
        out.textures_delta.clear();
    }

    /// New Palette asks for a name ("Palette 1" at first), makes an
    /// empty palette of the user's under My Palettes, shows it at once
    /// in a bar of its own docked at the bottom, and keeps it in the
    /// preferences; four of them stand in four bars.
    #[test]
    fn new_palette_makes_a_named_empty_palette_shown_in_its_own_bar() {
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let mut app = App::headless();
        let before = app.palettes.len();
        crate::ui::palette_bars::start_new_palette(&mut app);
        assert_eq!(app.palette_new.as_deref(), Some("Palette 1"));
        new_palette_frame(&ctx, &mut app, Vec::new(), &clock);
        new_palette_frame(&ctx, &mut app, Vec::new(), &clock);
        let all = egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        };
        new_palette_frame(&ctx, &mut app, vec![all], &clock);
        new_palette_frame(
            &ctx,
            &mut app,
            vec![egui::Event::Text("Brand".into())],
            &clock,
        );
        let enter = egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        new_palette_frame(&ctx, &mut app, vec![enter], &clock);
        new_palette_frame(&ctx, &mut app, Vec::new(), &clock);
        assert!(app.palette_new.is_none());
        assert_eq!(app.palettes.len(), before + 1);
        let i = before;
        let p = &app.palettes[i];
        assert_eq!(p.name, "Brand");
        assert!(p.colors.is_empty() && !p.builtin && p.path.is_none());
        assert!(app.visible_palettes.contains(&i));
        let k = app
            .extra_palettes
            .iter()
            .position(|r| r.index == i)
            .expect("a bar of its own");
        assert_eq!(app.settings.palette_bars[k].dock, Dock::Bottom);
        assert_eq!(app.palette_manager.selected, Selected::Palette(i));
        assert_eq!(app.settings.user_palettes.len(), 1);
        assert_eq!(app.settings.user_palettes[0].name, "Brand");
        // The next names count on; a name taken gets a number.
        assert_eq!(app.next_palette_name(), "Palette 1");
        let a = app.new_palette("").expect("Palette 1");
        assert_eq!(app.palettes[a].name, "Palette 1");
        assert_eq!(app.next_palette_name(), "Palette 2");
        let b = app.new_palette("Brand").expect("another Brand");
        assert_eq!(app.palettes[b].name, "Brand 2");
        let c = app
            .new_palette(&app.next_palette_name())
            .expect("Palette 2");
        assert_eq!(app.palettes[c].name, "Palette 2");
        // Four palettes of the user's, each in a bar at the bottom.
        let bottom = app
            .settings
            .palette_bars
            .iter()
            .filter(|b| b.dock == Dock::Bottom)
            .count();
        assert_eq!(bottom, 4);
        assert_eq!(app.visible_palettes.len(), 5);
        // Colours added, saved and read back: the palettes and their bars.
        assert!(app.add_colors_to_palette(i, &[Color::rgb8(10, 20, 30)]));
        app.sync_settings();
        let json = serde_json::to_string(&app.settings).expect("saved");
        let back: crate::settings::Settings = serde_json::from_str(&json).expect("read");
        let mut again = App::headless();
        again.apply_settings(back);
        let j = again
            .palettes
            .iter()
            .position(|p| p.name == "Brand")
            .expect("Brand back");
        assert_eq!(again.palettes[j].colors.len(), 1);
        assert_eq!(again.palettes[j].colors[0].1, Color::rgb8(10, 20, 30));
        assert!(again.visible_palettes.contains(&j));
        assert_eq!(again.extra_palettes.len(), 4);
        // Built-in palettes take no colours.
        assert!(!again.add_colors_to_palette(0, &[Color::rgb8(1, 2, 3)]));
    }

    /// Each palette's row has a Show bar / Hide bar button: it opens the
    /// palette in a bar of its own and closes it again; while shown, the
    /// row's list moves the bar.
    #[test]
    fn show_bar_and_hide_bar_toggle_the_palette_s_bar() {
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let mut app = App::headless();
        run(&ctx, &mut app, Vec::new(), &clock);
        let gray = app
            .palettes
            .iter()
            .position(|p| p.key == "palette.grayscale")
            .expect("grayscale");
        let row = ctx
            .data(|d| d.get_temp::<Rect>(egui::Id::new(("palette_manager_row", gray))))
            .expect("the grayscale row");
        let button = pos2(row.right() - 30.0 - BAR_CONTROLS_W / 2.0, row.top() + 14.0);
        click(&ctx, &mut app, button, &clock);
        assert!(app.visible_palettes.contains(&gray), "Show bar shows it");
        assert_eq!(app.extra_palettes.len(), 1);
        assert_eq!(bar_of(&app, Selected::Palette(gray)), Some(Row::Extra(0)));
        click(&ctx, &mut app, button, &clock);
        assert!(!app.visible_palettes.contains(&gray), "Hide bar hides it");
        assert!(app.extra_palettes.is_empty());
        // The default bar's palette, the only one shown, keeps its bar.
        let main = ctx
            .data(|d| d.get_temp::<Rect>(egui::Id::new(("palette_manager_row", 0))))
            .expect("the default palette's row");
        let main_button = pos2(
            main.right() - 30.0 - BAR_CONTROLS_W / 2.0,
            main.top() + 14.0,
        );
        click(&ctx, &mut app, main_button, &clock);
        assert_eq!(app.visible_palettes, vec![0]);
    }

    #[test]
    fn the_list_shows_hides_and_manages_palettes() {
        let ctx = crate::theme::ui_context();
        let clock = std::cell::Cell::new(0.0);
        let mut app = App::headless();
        run(&ctx, &mut app, Vec::new(), &clock);
        let gray = app
            .palettes
            .iter()
            .position(|p| p.key == "palette.grayscale")
            .expect("grayscale");
        let row = ctx
            .data(|d| d.get_temp::<Rect>(egui::Id::new(("palette_manager_row", gray))))
            .expect("the grayscale row");
        let eye = row.min + vec2(13.0, 14.0);
        click(&ctx, &mut app, eye, &clock);
        assert!(app.visible_palettes.contains(&gray));
        assert_eq!(app.extra_palettes.len(), 1);
        click(&ctx, &mut app, eye, &clock);
        assert!(!app.visible_palettes.contains(&gray));
        // A click on the row selects it.
        click(&ctx, &mut app, row.center(), &clock);
        assert_eq!(app.palette_manager.selected, Selected::Palette(gray));
        // Duplicate: a palette of the user's, named after it.
        let copy = app.duplicate_palette(gray).expect("a copy");
        assert!(!app.palettes[copy].builtin);
        assert_eq!(app.palettes[copy].colors, app.palettes[gray].colors);
        assert!(app.palette_name(copy).contains(&app.palette_name(gray)));
        let second = app.duplicate_palette(gray).expect("another copy");
        assert_ne!(app.palette_name(copy), app.palette_name(second));
        // Shown in a bar, renamed, moved up: the bar keeps it.
        let id = crate::settings::palette_id(&app.palettes[second]);
        app.open_palette_bar(&id);
        app.move_palette_in_list(second, copy);
        assert_eq!(app.extra_palettes[0].index, copy);
        assert_eq!(app.visible_palettes, vec![0, copy]);
        // Deleted: its bar closes and the list is one shorter.
        let n = app.palettes.len();
        app.delete_palette(copy);
        assert_eq!(app.palettes.len(), n - 1);
        assert!(app.extra_palettes.is_empty());
        // Built-in palettes are never deleted.
        app.delete_palette(gray);
        assert_eq!(app.palettes.len(), n - 1);
        // The filter keeps the rows whose name holds it.
        app.palette_manager.filter = "gray".into();
        ctx.data_mut(|d| d.clear());
        run(&ctx, &mut app, Vec::new(), &clock);
        let shows = |i: usize| {
            ctx.data(|d| d.get_temp::<Rect>(egui::Id::new(("palette_manager_row", i))))
                .is_some()
        };
        assert!(shows(gray));
        assert!(!shows(0));
    }

    #[test]
    fn the_colour_model_is_named() {
        let cmyk = vec![("a".to_string(), Color::cmyk_pct(0.0, 0.0, 0.0, 100.0))];
        assert_eq!(model_key(&cmyk), "palette.model_cmyk");
        let rgb = vec![("a".to_string(), Color::rgb8(1, 2, 3))];
        assert_eq!(model_key(&rgb), "palette.model_rgb");
        let both = [cmyk, rgb].concat();
        assert_eq!(model_key(&both), "palette.model_mixed");
        assert_eq!(model_key(&[]), "palette.model_none");
    }
}
