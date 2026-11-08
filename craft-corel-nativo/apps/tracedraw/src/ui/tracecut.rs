//! The TraceCut window (Launch > TraceCut, Tools > TraceCut): the tools
//! grouped on the left (Cut, Nesting, Measure); on the right the chosen
//! tool's presets and settings, a preview of the selection with what the
//! tool finds, a status line, and Apply, OK and Cancel. Apply works the
//! result out into the preview without touching the drawing; OK applies
//! it in one undo step and closes. See docs/behavior/tracecut.md.
//!
//! The preset row and the preview are adapted from the Sign & Cut window
//! kit of #170 by LaMka21.

use crate::app::{App, Units};
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::tracecut::{Group, PreviewView, Scene, ToolId, TraceCutState, TOLERANCE_RANGE};
use crate::ui::tips::RichTip;
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Pos2, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use tracedraw_core::geometry::{PathEl, Point, Shape as _};

/// Width of the tool list.
const LIST_W: f32 = 210.0;
/// The window's first and smallest size.
const DEFAULT_SIZE: Vec2 = Vec2::new(880.0, 600.0);
const MIN_SIZE: Vec2 = Vec2::new(660.0, 460.0);
/// Width of the labels of the settings rows.
const LABEL_W: f32 = 170.0;
/// Room kept under the preview for the status line and the buttons.
const BELOW_PREVIEW: f32 = 74.0;
/// Colours of the preview.
const OBJECT: Color32 = Color32::from_gray(70);
const MARKED: Color32 = Color32::from_rgb(0xD0, 0x20, 0x20);
/// Moves with the knife up.
const MOVE: Color32 = Color32::from_rgb(0x30, 0x80, 0xD0);
/// The status line under the preview.
const STRIP: Color32 = Color32::from_rgb(0xFF, 0xF4, 0xB0);

/// Which bottom button was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Button {
    Apply,
    Ok,
    Cancel,
}

/// Draw the TraceCut window.
pub fn dialog(app: &mut App, ctx: &Context, st: &mut TraceCutState, close: &mut bool) {
    let title = tr("tracecut.title");
    let mut pressed = None;
    crate::ui::chrome::window(title)
        .resizable(DEFAULT_SIZE, MIN_SIZE)
        .show(ctx, |ui| {
            let body = ui.available_size();
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(LIST_W, body.y),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(LIST_W);
                        tool_list(app, ui, st);
                    },
                );
                ui.separator();
                let right_w = ui.available_width().max(300.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(right_w, body.y),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(right_w);
                        pressed = tool_pane(app, ui, st, right_w);
                    },
                );
            });
            if pressed.is_none() && crate::ui::dialogs::enter_pressed(ui) {
                pressed = Some(Button::Ok);
            }
        });
    match pressed {
        Some(Button::Apply) => {
            st.worked = Some(app.tracecut_work_out(st.tool, &st.settings));
        }
        Some(Button::Ok) => {
            let worked = match st.worked.take() {
                Some(w) if app.tracecut_current(&w, st.tool, &st.settings) => w,
                _ => app.tracecut_work_out(st.tool, &st.settings),
            };
            if !worked.targets.is_empty() || st.tool == ToolId::RegistrationMarks {
                app.tracecut_commit(&worked, &st.settings, Some(ctx.clone()));
            }
            *close = true;
        }
        Some(Button::Cancel) => *close = true,
        None => {}
    }
}

/// The groups and their tools.
fn tool_list(app: &mut App, ui: &mut Ui, st: &mut TraceCutState) {
    ui.add_space(4.0);
    for g in Group::ALL {
        let tools = g.tools();
        if tools.is_empty() {
            continue;
        }
        crate::ui::chrome::section(ui, &tr(g.key()));
        for t in tools {
            let on = st.tool == t;
            if ui
                .add_sized(
                    [LIST_W - 8.0, 24.0],
                    egui::Button::selectable(on, tr(t.key())),
                )
                .rich_tip(crate::ui::tips::keyed(t.key()))
                .clicked()
                && !on
            {
                st.tool = t;
                st.worked = None;
                st.preset_edit = false;
                st.preset_name.clear();
                app.settings.tracecut.tool = t;
            }
        }
    }
}

/// The chosen tool: presets, settings, preview, status and buttons.
fn tool_pane(app: &mut App, ui: &mut Ui, st: &mut TraceCutState, width: f32) -> Option<Button> {
    ui.label(egui::RichText::new(tr(st.tool.key())).size(15.0).strong());
    ui.label(egui::RichText::new(tr(&format!("{}_desc", st.tool.key()))).color(Tokens::text_dim()));
    ui.add_space(4.0);
    preset_row(ui, st);
    ui.add_space(4.0);
    let before = st.settings.clone();
    // A long list of settings scrolls; the preview keeps its room.
    let max_h = (ui.available_height() * 0.5).max(120.0);
    let action = egui::ScrollArea::vertical()
        .id_salt(("tracecut_settings", st.tool.id()))
        .max_height(max_h)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            let a = crate::ui::tracecut_cut::settings(ui, app.units, st);
            crate::ui::tracecut_nest::settings(ui, app.units, st);
            settings(ui, app.units, st);
            a
        })
        .inner;
    if action == Some(crate::ui::tracecut_cut::Action::TestCut) {
        let plot = app.tracecut_test_job(&st.settings.plotter);
        let s = st.settings.plotter.clone();
        app.tracecut_output(&s, &plot, Some(ui.ctx().clone()));
    }
    if st.settings != before {
        // A setting changed: the result shown is not this one's.
        st.worked = None;
    }
    let targets = app.tracecut_targets();
    st.track_targets(&targets);
    // The Measure tools show their results live, worked out again only
    // when the drawing, the selection or a setting changes.
    if st.tool.group() == Group::Measure
        && !targets.is_empty()
        && !st
            .worked
            .as_ref()
            .is_some_and(|w| app.tracecut_current(w, st.tool, &st.settings))
    {
        st.worked = Some(app.tracecut_work_out(st.tool, &st.settings));
    }
    let current = st
        .worked
        .as_ref()
        .filter(|w| app.tracecut_current(w, st.tool, &st.settings));
    let scene = app.tracecut_scene(current);
    let has_targets = !scene.objects.is_empty() || !app.selection.is_empty();
    let input = PreviewInput {
        scene: &scene,
        actual: f64::from(crate::settings::screen_points_per_inch(&app.settings)) / 25.4,
        hint: (!has_targets).then(|| tr("tracecut.empty_hint")),
    };
    let h = (ui.available_height() - BELOW_PREVIEW).max(120.0);
    let view = st.view();
    let report = match current.map(|w| &w.outcome) {
        Some(crate::tracecut::Outcome::Report(r)) if has_targets => Some(r),
        _ => None,
    };
    let view = if let Some(r) = report {
        ui.horizontal_top(|ui| {
            let gap = ui.spacing().item_spacing.x;
            let w = width - 4.0 - REPORT_W - gap;
            let v = preview(ui, Vec2::new(w, h), &input, view);
            report_box(ui, Vec2::new(REPORT_W, h), r);
            v
        })
        .inner
    } else {
        preview(ui, Vec2::new(width - 4.0, h), &input, view)
    };
    let line = match (current, &st.worked) {
        _ if !has_targets => tr("tracecut.select_first"),
        (Some(w), _) => w.outcome.summary(),
        (None, Some(_)) => tr("tracecut.out_of_date"),
        (None, None) => tr("tracecut.press_apply"),
    };
    st.set_view(view);
    status_strip(ui, width - 4.0, &line);
    ui.add_space(8.0);
    buttons(ui, has_targets)
}

/// The preset list, with a button that shows the row to save, delete or
/// reset them.
fn preset_row(ui: &mut Ui, st: &mut TraceCutState) {
    let tool = st.tool;
    let (names, current) = {
        let p = st.presets.tool(tool);
        (
            p.items.keys().cloned().collect::<Vec<String>>(),
            p.current.clone(),
        )
    };
    let mut picked: Option<String> = None;
    ui.horizontal(|ui| {
        ui.add_sized(
            [LABEL_W, 22.0],
            egui::Label::new(tr("tracecut.preset")).truncate(),
        );
        let shown = current
            .clone()
            .unwrap_or_else(|| tr("tracecut.preset_none"));
        crate::ui::keynav::ComboBox::from_id_salt(("tracecut_presets", tool.id()))
            .selected_text(shown)
            .width(200.0)
            .show_ui(ui, |ui| {
                for n in &names {
                    if ui
                        .selectable_label(current.as_ref() == Some(n), n)
                        .clicked()
                    {
                        picked = Some(n.clone());
                    }
                }
            })
            .response
            .rich_tip(crate::ui::tips::keyed("tracecut.preset"));
        if ui
            .add(egui::Button::selectable(st.preset_edit, "\u{2699}"))
            .rich_tip(crate::ui::tips::keyed("tracecut.preset_manage"))
            .clicked()
        {
            st.preset_edit = !st.preset_edit;
            if st.preset_name.is_empty() {
                st.preset_name = current.clone().unwrap_or_default();
            }
        }
    });
    if let Some(n) = picked {
        st.presets.apply_preset(tool, &n, &mut st.settings);
        st.preset_name = n;
        st.worked = None;
    }
    if !st.preset_edit {
        return;
    }
    ui.horizontal(|ui| {
        ui.add_space(LABEL_W + ui.spacing().item_spacing.x);
        ui.add_sized(
            [160.0, 22.0],
            egui::TextEdit::singleline(&mut st.preset_name).hint_text(tr("tracecut.preset_name")),
        );
        let named = !st.preset_name.trim().is_empty();
        if ui
            .add_enabled(named, egui::Button::new(tr("tracecut.preset_save")))
            .clicked()
        {
            let name = st.preset_name.clone();
            st.presets.save_preset(tool, &name, &st.settings);
        }
        if ui
            .add_enabled(named, egui::Button::new(tr("tracecut.preset_delete")))
            .clicked()
        {
            let name = st.preset_name.clone();
            st.presets.delete_preset(tool, &name);
        }
        if ui.button(tr("tracecut.preset_reset")).clicked() {
            st.settings.reset(tool);
            st.worked = None;
        }
    });
}

/// A settings row: the label on the left, `add` after it.
pub(crate) fn row<R>(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.horizontal(|ui| {
        if label.is_empty() {
            ui.add_space(LABEL_W);
        } else {
            ui.add_sized([LABEL_W, 22.0], egui::Label::new(label).truncate());
        }
        add(ui)
    })
    .inner
}

/// A length field in the document's units holding millimetres.
pub(crate) fn length(ui: &mut Ui, u: Units, mm: &mut f64, range: (f64, f64), tip: &str) {
    let mut v = u.world_from_mm(*mm);
    let r = ui
        .add_sized(
            [110.0, 22.0],
            crate::ui::field::NumField::new(&mut v)
                .range(u.world_from_mm(range.0)..=u.world_from_mm(range.1))
                .max_decimals(3)
                .suffix(format!(" {}", u.short()))
                .unit_mm(u.mm()),
        )
        .rich_tip(crate::ui::tips::keyed(tip));
    if r.changed() && v.is_finite() {
        *mm = u.world_to_mm(v).clamp(range.0, range.1);
    }
}

/// The chosen tool's settings.
fn settings(ui: &mut Ui, units: Units, st: &mut TraceCutState) {
    match st.tool {
        ToolId::FindDuplicates => {
            let s = &mut st.settings.duplicates;
            row(ui, &tr("tracecut.tolerance"), |ui| {
                length(
                    ui,
                    units,
                    &mut s.tolerance,
                    TOLERANCE_RANGE,
                    "tracecut.tolerance",
                )
            });
            row(ui, &tr("tracecut.on_ok"), |ui| {
                ui.radio_value(&mut s.delete, true, tr("tracecut.delete_duplicates"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.delete_duplicates"));
                ui.radio_value(&mut s.delete, false, tr("tracecut.select_duplicates"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.select_duplicates"));
            });
        }
        ToolId::FindCrossings => {
            let s = &mut st.settings.crossings;
            row(ui, &tr("tracecut.tolerance"), |ui| {
                length(
                    ui,
                    units,
                    &mut s.tolerance,
                    TOLERANCE_RANGE,
                    "tracecut.tolerance",
                )
            });
            row(ui, "", |ui| {
                ui.checkbox(&mut s.between, tr("tracecut.crossings_between"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.crossings_between"));
            });
        }
        // The other Cut tools: ui/tracecut_cut.rs.
        _ => {}
    }
}

/// One notch of the wheel zooms by this much, as on the drawing.
const WHEEL_NOTCH: f64 = 1.15;
/// The zoom buttons' and keys' step.
const ZOOM_STEP: f64 = 1.25;
/// The corner buttons over the preview, points square.
const CORNER_BUTTON: f32 = 24.0;
/// Room kept round the result when it is fitted.
const FIT_MARGIN: f32 = 16.0;
/// Width of the measurements beside the preview.
const REPORT_W: f32 = 230.0;

/// What a corner button over the preview does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoomButton {
    In,
    Out,
    Fit,
    Selection,
    Actual,
}

impl ZoomButton {
    const ALL: [ZoomButton; 5] = [
        ZoomButton::In,
        ZoomButton::Out,
        ZoomButton::Fit,
        ZoomButton::Selection,
        ZoomButton::Actual,
    ];

    fn key(self) -> &'static str {
        match self {
            ZoomButton::In => "tracecut.zoom_in",
            ZoomButton::Out => "tracecut.zoom_out",
            ZoomButton::Fit => "tracecut.zoom_fit",
            ZoomButton::Selection => "tracecut.zoom_selection",
            ZoomButton::Actual => "tracecut.zoom_actual",
        }
    }

    #[cfg(test)]
    fn mark(self) -> &'static str {
        match self {
            ZoomButton::In => "tracecut_zoom_in",
            ZoomButton::Out => "tracecut_zoom_out",
            ZoomButton::Fit => "tracecut_zoom_fit",
            ZoomButton::Selection => "tracecut_zoom_selection",
            ZoomButton::Actual => "tracecut_zoom_actual",
        }
    }

    fn draw(self, p: &egui::Painter, r: egui::Rect, c: Color32) {
        use crate::ui::before_after::{draw_fit, draw_zoom};
        match self {
            ZoomButton::In => draw_zoom(p, r, c, true),
            ZoomButton::Out => draw_zoom(p, r, c, false),
            ZoomButton::Fit => draw_fit(p, r, c),
            ZoomButton::Selection => {
                // The fit frame round a selected square.
                draw_fit(p, r, c);
                let k = r.width() / 16.0;
                p.rect_filled(
                    egui::Rect::from_center_size(r.center(), Vec2::splat(4.0 * k)),
                    0.0,
                    c,
                );
            }
            ZoomButton::Actual => {
                p.text(
                    r.center(),
                    Align2::CENTER_CENTER,
                    "1:1",
                    FontId::proportional(11.0),
                    c,
                );
            }
        }
    }
}

/// What the preview draws and how it may be zoomed.
struct PreviewInput<'a> {
    scene: &'a Scene,
    /// Interface points per millimetre at real size (1:1).
    actual: f64,
    /// The hint shown in place of an empty box.
    hint: Option<String>,
}

/// The whole result: the objects, what was found and the moves.
fn full_extent(scene: &Scene) -> Option<tracedraw_core::geometry::Rect> {
    let mut e = scene.extent;
    let mut add = |q: Point| {
        if q.x.is_finite() && q.y.is_finite() {
            let r = tracedraw_core::geometry::Rect::from_points(q, q);
            e = Some(e.map_or(r, |e| e.union(r)));
        }
    };
    for q in &scene.points {
        add(*q);
    }
    for (a, b) in &scene.moves {
        add(*a);
        add(*b);
    }
    e
}

/// The zoom that fits `r` in `area`.
fn fit_zoom(r: tracedraw_core::geometry::Rect, area: egui::Rect) -> f64 {
    let (w, h) = (r.width().max(1e-6), r.height().max(1e-6));
    let z = (f64::from(area.width()) / w).min(f64::from(area.height()) / h);
    if z.is_finite() && z > 0.0 {
        z
    } else {
        1.0
    }
}

/// The preview: the scene in a white box, y up, fitted until it is
/// zoomed (the wheel at the pointer, pinch, the corner buttons, +, -
/// and 0 with the pointer over it) or panned (a drag with the left or
/// middle button); a double click fits it again. The outlines are drawn
/// again at every zoom, and what lies outside the box is skipped.
/// Returns the view after this frame's zooming and panning.
fn preview(
    ui: &mut Ui,
    size: Vec2,
    input: &PreviewInput,
    view: Option<PreviewView>,
) -> Option<PreviewView> {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    #[cfg(test)]
    crate::ui::before_after::mark(ui, "tracecut_preview", rect);
    let p = ui.painter_at(rect);
    p.rect(
        rect,
        CornerRadius::ZERO,
        Color32::WHITE,
        Stroke::new(1.0, Tokens::border()),
        StrokeKind::Inside,
    );
    if let Some(hint) = &input.hint {
        let galley = ui.painter().layout(
            hint.clone(),
            FontId::proportional(15.0),
            Color32::from_gray(110),
            (rect.width() - 40.0).max(40.0),
        );
        let at = rect.center() - galley.size() / 2.0;
        #[cfg(test)]
        crate::ui::before_after::mark(
            ui,
            "tracecut_hint",
            egui::Rect::from_min_size(at, galley.size()),
        );
        p.galley(at, galley, Color32::from_gray(110));
        return view;
    }
    let scene = input.scene;
    let Some(extent) = full_extent(scene) else {
        return view;
    };
    let area = rect.shrink(FIT_MARGIN);
    let fit = fit_zoom(extent, area);
    let fitted = PreviewView {
        zoom: fit,
        centre: extent.center(),
    };
    let lo = fit * 0.1;
    let hi = (fit * 4.0).max(2000.0);
    let mut view = view;
    let mut v = view.unwrap_or(fitted);
    let middle = rect.center();
    let to_page = |v: &PreviewView, s: Pos2| {
        Point::new(
            v.centre.x + f64::from(s.x - middle.x) / v.zoom,
            v.centre.y - f64::from(s.y - middle.y) / v.zoom,
        )
    };
    // Zoom by `f` keeping the page point under `at` where it is.
    let zoom_at = |v: &mut PreviewView, at: Pos2, f: f64| {
        let q = to_page(v, at);
        v.zoom = (v.zoom * f).clamp(lo, hi);
        v.centre = Point::new(
            q.x - f64::from(at.x - middle.x) / v.zoom,
            q.y + f64::from(at.y - middle.y) / v.zoom,
        );
    };
    let mut changed = false;

    // The corner buttons, over the box (drawn after it, so they take
    // the pointer before it).
    let mut corner = None;
    let mut corners = Vec::with_capacity(ZoomButton::ALL.len());
    let mut x = rect.right() - 4.0;
    for b in ZoomButton::ALL.iter().rev() {
        let r = egui::Rect::from_min_size(
            Pos2::new(x - CORNER_BUTTON, rect.top() + 4.0),
            Vec2::splat(CORNER_BUTTON),
        );
        x -= CORNER_BUTTON + 2.0;
        #[cfg(test)]
        crate::ui::before_after::mark(ui, b.mark(), r);
        let id = ui.id().with(("tracecut_zoom", b.key()));
        let br = ui
            .interact(r, id, Sense::click())
            .rich_tip(crate::ui::tips::keyed(b.key()));
        if br.clicked() {
            corner = Some(*b);
        }
        corners.push((*b, r, br.hovered()));
    }
    match corner {
        Some(ZoomButton::In) => {
            zoom_at(&mut v, middle, ZOOM_STEP);
            changed = true;
        }
        Some(ZoomButton::Out) => {
            zoom_at(&mut v, middle, 1.0 / ZOOM_STEP);
            changed = true;
        }
        Some(ZoomButton::Fit) => view = None,
        Some(ZoomButton::Selection) => {
            let sel = scene
                .objects
                .iter()
                .map(|p| p.bounding_box())
                .filter(|b| b.x0.is_finite() && b.y0.is_finite() && b.x1.is_finite())
                .reduce(|a, b| a.union(b))
                .unwrap_or(extent);
            v = PreviewView {
                zoom: fit_zoom(sel, area).clamp(lo, hi),
                centre: sel.center(),
            };
            changed = true;
        }
        Some(ZoomButton::Actual) => {
            v.zoom = input.actual.clamp(lo, hi);
            changed = true;
        }
        None => {}
    }

    let over = resp.hovered() || resp.dragged();
    if corner.is_none() && resp.double_clicked() {
        view = None;
        changed = false;
        v = fitted;
    } else if resp.dragged_by(egui::PointerButton::Primary)
        || resp.dragged_by(egui::PointerButton::Middle)
    {
        let d = resp.drag_delta();
        if d != Vec2::ZERO {
            v.centre = Point::new(
                v.centre.x - f64::from(d.x) / v.zoom,
                v.centre.y + f64::from(d.y) / v.zoom,
            );
            changed = true;
        }
    }
    if resp.hovered() {
        let web = crate::ui::field::web_wheel();
        let (notches, zoom_delta, touch, pointer) = ui.input(|i| {
            let notches: f32 = i
                .raw
                .events
                .iter()
                .map(|e| match e {
                    egui::Event::MouseWheel { unit, delta, .. } => {
                        delta.y / crate::ui::field::wheel_notch(*unit, web)
                    }
                    _ => 0.0,
                })
                .sum();
            // A pinch: two fingers on a touch screen, or a touchpad's
            // pinch (egui's zoom delta also holds Ctrl+wheel, spread over
            // frames, which the notches above already count).
            let touchpad: f32 = i
                .raw
                .events
                .iter()
                .map(|e| match e {
                    egui::Event::Zoom(f) => *f,
                    _ => 1.0,
                })
                .product();
            let fingers = i.multi_touch().map_or(1.0, |t| t.zoom_delta);
            (
                notches,
                touchpad * fingers,
                i.multi_touch().map(|t| t.translation_delta),
                i.pointer.hover_pos(),
            )
        });
        let at = pointer.unwrap_or(middle);
        if notches != 0.0 {
            // A notch (Ctrl or not) zooms as on the drawing.
            zoom_at(&mut v, at, WHEEL_NOTCH.powf(f64::from(notches)));
            changed = true;
        } else if zoom_delta != 1.0 {
            // A pinch.
            zoom_at(&mut v, at, f64::from(zoom_delta));
            changed = true;
        }
        if let Some(t) = touch.filter(|t| *t != Vec2::ZERO) {
            v.centre = Point::new(
                v.centre.x - f64::from(t.x) / v.zoom,
                v.centre.y + f64::from(t.y) / v.zoom,
            );
            changed = true;
        }
        if notches != 0.0 || zoom_delta != 1.0 {
            crate::ui::field::take_wheel(ui);
        }
        if !ui.ctx().text_edit_focused() {
            let key = ui.input_mut(|i| {
                let mut got = None;
                i.events.retain(|e| match e {
                    egui::Event::Key {
                        key,
                        pressed,
                        modifiers,
                        ..
                    } if !(modifiers.ctrl || modifiers.alt || modifiers.command) => {
                        let k = match key {
                            egui::Key::Plus | egui::Key::Equals => Some(ZoomButton::In),
                            egui::Key::Minus => Some(ZoomButton::Out),
                            egui::Key::Num0 => Some(ZoomButton::Fit),
                            // Space held for a pan presses no button.
                            egui::Key::Space => None,
                            _ => return true,
                        };
                        if *pressed && k.is_some() {
                            got = k;
                        }
                        false
                    }
                    _ => true,
                });
                got
            });
            match key {
                Some(ZoomButton::In) => {
                    zoom_at(&mut v, at, ZOOM_STEP);
                    changed = true;
                }
                Some(ZoomButton::Out) => {
                    zoom_at(&mut v, at, 1.0 / ZOOM_STEP);
                    changed = true;
                }
                Some(_) => {
                    view = None;
                    changed = false;
                    v = fitted;
                }
                None => {}
            }
        }
    }
    if over {
        ui.ctx().set_cursor_icon(if resp.dragged() {
            egui::CursorIcon::Grabbing
        } else {
            egui::CursorIcon::Grab
        });
    }
    if changed {
        view = Some(v);
    }
    let zoomed = view.is_some();
    let v = view.unwrap_or(fitted);

    // Draw, at this zoom.
    let (s, c) = (v.zoom, v.centre);
    let to = |q: Point| {
        Pos2::new(
            (f64::from(middle.x) + (q.x - c.x) * s) as f32,
            (f64::from(middle.y) - (q.y - c.y) * s) as f32,
        )
    };
    // The page area in view, a little wider for the strokes.
    let seen = {
        let a = to_page(&v, rect.min - Vec2::splat(8.0));
        let b = to_page(&v, rect.max + Vec2::splat(8.0));
        tracedraw_core::geometry::Rect::from_points(a, b)
    };
    let visible = |b: tracedraw_core::geometry::Rect| {
        b.x1 >= seen.x0 && b.x0 <= seen.x1 && b.y1 >= seen.y0 && b.y0 <= seen.y1
    };
    let tol = (0.5 / s).max(1e-6);
    let draw = |path: &tracedraw_core::geometry::BezPath, stroke: Stroke| {
        // Fitted, everything is in view; zoomed, what is not is skipped.
        if zoomed && !visible(path.bounding_box()) {
            return;
        }
        let mut pts: Vec<Pos2> = Vec::new();
        let mut start: Option<Pos2> = None;
        let flush = |pts: &mut Vec<Pos2>| {
            if pts.len() > 1 {
                p.add(egui::Shape::line(std::mem::take(pts), stroke));
            } else {
                pts.clear();
            }
        };
        tracedraw_core::geometry::flatten(path, tol, &mut |el| match el {
            PathEl::MoveTo(q) => {
                flush(&mut pts);
                let a = to(q);
                start = Some(a);
                pts.push(a);
            }
            PathEl::LineTo(q) => pts.push(to(q)),
            PathEl::ClosePath => {
                if let Some(a) = start {
                    pts.push(a);
                }
            }
            _ => {}
        });
        flush(&mut pts);
    };
    for path in &scene.objects {
        draw(path, Stroke::new(1.0, OBJECT));
    }
    for path in &scene.marked {
        draw(path, Stroke::new(2.5, MARKED));
    }
    let shown = rect.expand(8.0);
    for q in &scene.points {
        let a = to(*q);
        if shown.contains(a) {
            p.circle_stroke(a, 6.0, Stroke::new(2.0, MARKED));
        }
    }
    for (a, b) in &scene.moves {
        let (a, b) = (to(*a), to(*b));
        if shown.intersects(egui::Rect::from_two_pos(a, b)) {
            p.line_segment([a, b], Stroke::new(1.0, MOVE));
        }
    }
    // The buttons on top of the drawing.
    for (b, r, hovered) in corners {
        // The box stays white in every theme, and so do its buttons.
        let (fill, edge) = if hovered {
            (
                Color32::from_rgb(0xDD, 0xEB, 0xF7),
                Color32::from_rgb(0x60, 0x90, 0xC8),
            )
        } else {
            (Color32::from_white_alpha(230), Color32::from_gray(200))
        };
        p.rect(r, 3.0, fill, Stroke::new(1.0, edge), StrokeKind::Inside);
        b.draw(
            &p,
            egui::Rect::from_center_size(r.center(), Vec2::splat(16.0)),
            Color32::from_gray(50),
        );
    }
    view
}

/// A calculation's result, to read beside the preview.
fn report_box(ui: &mut Ui, size: Vec2, r: &crate::tracecut_nest::Report) {
    ui.allocate_ui_with_layout(size, egui::Layout::top_down(egui::Align::Min), |ui| {
        egui::Frame::new()
            .fill(Tokens::field())
            .stroke(Stroke::new(1.0, Tokens::border()))
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                ui.set_width(size.x - 14.0);
                ui.set_min_height(size.y - 14.0);
                #[cfg(test)]
                crate::ui::before_after::mark(ui, "tracecut_report", ui.max_rect());
                for (i, line) in r.lines.iter().enumerate() {
                    let text = egui::RichText::new(line);
                    ui.add(
                        egui::Label::new(if i == 0 {
                            text.strong().size(14.0)
                        } else {
                            text
                        })
                        .wrap(),
                    );
                }
            });
    });
}

/// The status line under the preview.
fn status_strip(ui: &mut Ui, width: f32, text: &str) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::hover());
    ui.painter().rect_filled(r, 0.0, STRIP);
    ui.painter().text(
        Pos2::new(r.left() + 6.0, r.center().y),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(12.0),
        Color32::from_gray(30),
    );
}

/// Apply, OK and Cancel at the bottom right.
fn buttons(ui: &mut Ui, enabled: bool) -> Option<Button> {
    let labels = [tr("tracecut.apply"), tr("dialog.ok"), tr("dialog.cancel")];
    let mut pressed = None;
    ui.horizontal(|ui| {
        let w = 100.0;
        let room = ui.available_width() - 3.0 * w - 2.0 * ui.spacing().item_spacing.x;
        ui.add_space(room.max(0.0));
        for (i, l) in labels.iter().enumerate() {
            let on = enabled || i == 2;
            let tip = match i {
                0 => crate::ui::tips::keyed("tracecut.apply"),
                1 => crate::ui::tips::pair("dialog.ok", "tracecut.ok_desc"),
                _ => crate::ui::tips::pair("dialog.cancel", "tracecut.cancel_desc"),
            };
            // Greyed out, Apply and OK say why.
            let why = match i {
                0 => crate::ui::tips::pair("tracecut.apply", "tracecut.needs_selection"),
                _ => crate::ui::tips::pair("dialog.ok", "tracecut.needs_selection"),
            };
            let r = ui
                .add_enabled(on, egui::Button::new(l).min_size(Vec2::new(w, 27.0)))
                .rich_tip(tip);
            let r = if on { r } else { r.rich_disabled_tip(why) };
            if r.clicked() {
                pressed = Some(match i {
                    0 => Button::Apply,
                    1 => Button::Ok,
                    _ => Button::Cancel,
                });
            }
        }
    });
    pressed
}

/// Whether TraceCut is listed in Launch and in the Tools menu.
pub fn shown(app: &App) -> bool {
    app.settings.tracecut.show
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::trf;
    use crate::tracecut::Outcome;
    use crate::ui::dialogs::Dialog;
    use tracedraw_core::ShapeKind;

    fn frame(ctx: &Context, app: &mut App, enter: bool) {
        let mut input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(1400.0, 900.0),
            )),
            ..Default::default()
        };
        if enter {
            input.events.push(egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
        let mut out = ctx.run_ui(input, |ui| crate::ui::dialogs::show(app, &ui.ctx().clone()));
        out.textures_delta.clear();
    }

    fn two_squares(app: &mut App) -> (tracedraw_core::ShapeId, tracedraw_core::ShapeId) {
        let kind = ShapeKind::Rect {
            rect: tracedraw_core::geometry::Rect::new(10.0, 10.0, 40.0, 40.0),
            radius: 0.0,
            corners: None,
        };
        let a = app.new_shape(kind.clone()).expect("a");
        let b = app.new_shape(kind).expect("b");
        app.select(vec![a, b]);
        (a, b)
    }

    #[test]
    fn every_tool_draws_with_and_without_a_selection() {
        let ctx = crate::theme::ui_context();
        for selected in [false, true] {
            for t in ToolId::ALL {
                let mut app = App::headless();
                two_squares(&mut app);
                if !selected {
                    app.select(Vec::new());
                }
                let mut st = TraceCutState::new(t);
                st.preset_edit = true;
                app.dialog = Dialog::TraceCut(Box::new(st));
                frame(&ctx, &mut app, false);
                frame(&ctx, &mut app, false);
                assert!(
                    matches!(&app.dialog, Dialog::TraceCut(s) if s.tool == t),
                    "{t:?} stays open"
                );
            }
        }
    }

    #[test]
    fn enter_is_ok_and_ok_is_one_undo_step() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        let (a, b) = two_squares(&mut app);
        app.settings.tracecut.tool = ToolId::FindDuplicates;
        app.open_tracecut();
        frame(&ctx, &mut app, false);
        let before = app.engine.history_labels().0.len();
        frame(&ctx, &mut app, true);
        frame(&ctx, &mut app, false);
        assert!(matches!(app.dialog, Dialog::None));
        assert!(app.doc().find_shape(a).is_some());
        assert!(app.doc().find_shape(b).is_none());
        assert_eq!(app.engine.history_labels().0.len(), before + 1);
    }

    #[test]
    fn a_window_without_a_selection_changes_nothing_on_ok() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        two_squares(&mut app);
        app.select(Vec::new());
        app.open_tracecut();
        let rev = app.engine.revision();
        frame(&ctx, &mut app, false);
        frame(&ctx, &mut app, true);
        assert_eq!(app.engine.revision(), rev);
    }

    #[test]
    fn the_window_keeps_its_size_from_frame_to_frame() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        two_squares(&mut app);
        let mut st = TraceCutState::new(ToolId::FindDuplicates);
        st.worked = Some(app.tracecut_work_out(st.tool, &st.settings));
        assert!(matches!(
            st.worked.as_ref().map(|w| &w.outcome),
            Some(Outcome::Duplicates(d)) if d.len() == 1
        ));
        app.dialog = Dialog::TraceCut(Box::new(st));
        let mut sizes = Vec::new();
        for _ in 0..6 {
            frame(&ctx, &mut app, false);
            let id = egui::Id::new(("chrome_dialog", tr("tracecut.title")));
            if let Some(r) = ctx.memory(|m| m.area_rect(id)) {
                sizes.push(r.size());
            }
        }
        assert!(sizes.len() >= 2, "the window was drawn");
        let last = sizes[sizes.len() - 1];
        let before = sizes[sizes.len() - 2];
        assert!((last - before).length() < 0.5, "{sizes:?}");
    }

    /// Run `f` inside a frame, where the menus can be walked.
    fn in_frame(ctx: &Context, app: &mut App, f: impl FnOnce(&mut App, &Context)) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(1400.0, 900.0),
            )),
            ..Default::default()
        };
        let mut f = Some(f);
        let mut out = ctx.run_ui(input, |ui| {
            if let Some(f) = f.take() {
                f(app, &ui.ctx().clone());
            }
        });
        out.textures_delta.clear();
    }

    #[test]
    fn tools_tracecut_opens_the_window_and_options_can_hide_it() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        let listed = |app: &mut App| {
            let mut found = false;
            in_frame(&ctx, app, |app, c| {
                found = crate::ui::menus::commands(app, c)
                    .iter()
                    .any(|e| e.key == "menu.tools.tracecut");
            });
            found
        };
        assert!(listed(&mut app));
        let mut ran = false;
        in_frame(&ctx, &mut app, |app, c| {
            ran = crate::ui::menus::run_command(app, c, "menu.tools.tracecut");
        });
        assert!(ran);
        assert!(matches!(app.dialog, Dialog::TraceCut(_)));
        app.dialog = Dialog::None;
        app.settings.tracecut.show = false;
        assert!(!listed(&mut app));
    }

    /// A frame with `events`, the keys `mods` held.
    fn run(ctx: &Context, app: &mut App, events: Vec<egui::Event>, mods: egui::Modifiers) {
        let mut all = vec![egui::Event::ModifiersChanged(mods)];
        all.extend(events);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(1400.0, 900.0),
            )),
            events: all,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| crate::ui::dialogs::show(app, &ui.ctx().clone()));
        out.textures_delta.clear();
    }

    fn press(at: Pos2, pressed: bool, button: egui::PointerButton) -> egui::Event {
        egui::Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }

    fn wheel(notches: f32, mods: egui::Modifiers) -> egui::Event {
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: Vec2::new(0.0, notches),
            phase: egui::TouchPhase::Move,
            modifiers: mods,
        }
    }

    fn mark(ctx: &Context, name: &str) -> Option<egui::Rect> {
        crate::ui::before_after::rects(ctx).0.get(name).copied()
    }

    fn state(app: &App) -> &TraceCutState {
        match &app.dialog {
            Dialog::TraceCut(s) => s,
            _ => panic!("the TraceCut window is open"),
        }
    }

    /// The page point under `s` in the preview `rect` with view `v`.
    fn page_at(v: PreviewView, rect: egui::Rect, s: Pos2) -> Point {
        Point::new(
            v.centre.x + f64::from(s.x - rect.center().x) / v.zoom,
            v.centre.y - f64::from(s.y - rect.center().y) / v.zoom,
        )
    }

    /// Two squares apart, selected, and the window open on `tool`.
    fn opened(tool: ToolId) -> (Context, App) {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        let mut ids = Vec::new();
        for (x, y) in [(10.0, 10.0), (80.0, 40.0)] {
            let kind = ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(x, y, x + 30.0, y + 30.0),
                radius: 0.0,
                corners: None,
            };
            ids.push(app.new_shape(kind).expect("square"));
        }
        app.select(ids);
        app.dialog = Dialog::TraceCut(Box::new(TraceCutState::new(tool)));
        for _ in 0..3 {
            run(&ctx, &mut app, Vec::new(), egui::Modifiers::NONE);
        }
        (ctx, app)
    }

    /// Frames past the double-click time.
    fn settle(ctx: &Context, app: &mut App) {
        for _ in 0..40 {
            run(ctx, app, Vec::new(), egui::Modifiers::NONE);
        }
    }

    #[test]
    fn the_wheel_zooms_the_preview_at_the_pointer() {
        let (ctx, mut app) = opened(ToolId::FindDuplicates);
        let rect = mark(&ctx, "tracecut_preview").expect("the preview is drawn");
        assert_eq!(state(&app).view(), None, "fitted at first");
        let at = rect.center() + Vec2::new(90.0, 40.0);
        let none = egui::Modifiers::NONE;
        run(&ctx, &mut app, vec![egui::Event::PointerMoved(at)], none);
        run(&ctx, &mut app, vec![wheel(1.0, none)], none);
        let first = state(&app).view().expect("zoomed in");
        // Another notch: 1.15 times, the point under the pointer stays.
        let under = page_at(first, rect, at);
        run(&ctx, &mut app, vec![wheel(1.0, none)], none);
        let second = state(&app).view().expect("zoomed in");
        assert!(
            (second.zoom / first.zoom - 1.15).abs() < 1e-6,
            "{first:?} {second:?}"
        );
        let now = page_at(second, rect, at);
        assert!((now - under).hypot() < 1e-6, "{under:?} {now:?}");
        // Ctrl+wheel zooms the same, and back out.
        let ctrl = egui::Modifiers::CTRL;
        run(&ctx, &mut app, vec![wheel(-1.0, ctrl)], ctrl);
        let third = state(&app).view().expect("zoomed");
        assert!((second.zoom / third.zoom - 1.15).abs() < 1e-6, "{third:?}");
        assert!((page_at(third, rect, at) - under).hypot() < 1e-6);
        // The keys while the pointer is over it: + zooms in, 0 fits.
        let key = |k: egui::Key| egui::Event::Key {
            key: k,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: none,
        };
        run(&ctx, &mut app, vec![key(egui::Key::Plus)], none);
        let fourth = state(&app).view().expect("zoomed");
        assert!(
            (fourth.zoom / third.zoom - ZOOM_STEP).abs() < 1e-6,
            "{third:?} {fourth:?}"
        );
        run(&ctx, &mut app, vec![key(egui::Key::Num0)], none);
        assert_eq!(state(&app).view(), None, "0 fits");
    }

    #[test]
    fn a_drag_pans_the_preview_and_a_double_click_fits_it() {
        let (ctx, mut app) = opened(ToolId::FindDuplicates);
        let rect = mark(&ctx, "tracecut_preview").expect("the preview is drawn");
        let none = egui::Modifiers::NONE;
        let start = rect.center() + Vec2::new(-60.0, 30.0);
        run(&ctx, &mut app, vec![egui::Event::PointerMoved(start)], none);
        let primary = egui::PointerButton::Primary;
        run(&ctx, &mut app, vec![press(start, true, primary)], none);
        let mut at = start;
        for _ in 0..4 {
            at += Vec2::new(10.0, 5.0);
            run(&ctx, &mut app, vec![egui::Event::PointerMoved(at)], none);
        }
        run(&ctx, &mut app, vec![press(at, false, primary)], none);
        let v = state(&app).view().expect("panned");
        // The page point that was under the pointer went with it.
        let fitted_centre = {
            let s = app.tracecut_scene(None);
            full_extent(&s).expect("extent").center()
        };
        let moved = page_at(v, rect, rect.center()) - fitted_centre;
        assert!((moved.x + 40.0 / v.zoom).abs() < 1e-6, "{moved:?} {v:?}");
        assert!((moved.y - 20.0 / v.zoom).abs() < 1e-6, "{moved:?} {v:?}");
        settle(&ctx, &mut app);
        // A double click fits again.
        for _ in 0..2 {
            run(&ctx, &mut app, vec![press(at, true, primary)], none);
            run(&ctx, &mut app, vec![press(at, false, primary)], none);
        }
        assert_eq!(state(&app).view(), None, "the double click fits");
        // The middle button pans too.
        settle(&ctx, &mut app);
        let middle = egui::PointerButton::Middle;
        run(&ctx, &mut app, vec![press(at, true, middle)], none);
        for _ in 0..3 {
            at += Vec2::new(-8.0, 0.0);
            run(&ctx, &mut app, vec![egui::Event::PointerMoved(at)], none);
        }
        run(&ctx, &mut app, vec![press(at, false, middle)], none);
        assert!(state(&app).view().is_some(), "the middle button pans");
    }

    #[test]
    fn the_corner_buttons_zoom_and_fit_and_each_tool_keeps_its_zoom() {
        let (ctx, mut app) = opened(ToolId::FindDuplicates);
        let none = egui::Modifiers::NONE;
        let click = |ctx: &Context, app: &mut App, name: &str| {
            let r = mark(ctx, name).expect(name);
            settle(ctx, app);
            run(ctx, app, vec![egui::Event::PointerMoved(r.center())], none);
            let p = egui::PointerButton::Primary;
            run(ctx, app, vec![press(r.center(), true, p)], none);
            run(ctx, app, vec![press(r.center(), false, p)], none);
        };
        click(&ctx, &mut app, ZoomButton::In.mark());
        let zoomed = state(&app).view().expect("zoom in");
        click(&ctx, &mut app, ZoomButton::Out.mark());
        let out = state(&app).view().expect("zoom out");
        assert!((zoomed.zoom / out.zoom - ZOOM_STEP).abs() < 1e-6);
        click(&ctx, &mut app, ZoomButton::Actual.mark());
        let actual = state(&app).view().expect("1:1");
        let per_mm = f64::from(crate::settings::screen_points_per_inch(&app.settings)) / 25.4;
        assert!((actual.zoom - per_mm).abs() < 1e-6, "{actual:?} {per_mm}");
        click(&ctx, &mut app, ZoomButton::Selection.mark());
        assert!(state(&app).view().is_some(), "zoom to selection");
        click(&ctx, &mut app, ZoomButton::In.mark());
        let kept = state(&app).view();
        // Another tool fits; coming back finds the zoom kept.
        if let Dialog::TraceCut(s) = &mut app.dialog {
            s.tool = ToolId::FindCrossings;
        }
        run(&ctx, &mut app, Vec::new(), none);
        assert_eq!(state(&app).view(), None);
        if let Dialog::TraceCut(s) = &mut app.dialog {
            s.tool = ToolId::FindDuplicates;
        }
        run(&ctx, &mut app, Vec::new(), none);
        assert_eq!(state(&app).view(), kept);
        click(&ctx, &mut app, ZoomButton::Fit.mark());
        assert_eq!(state(&app).view(), None, "the fit button");
        // A new selection fits again.
        click(&ctx, &mut app, ZoomButton::In.mark());
        assert!(state(&app).view().is_some());
        let first = app.selection.first().copied();
        app.select(first.into_iter().collect());
        run(&ctx, &mut app, Vec::new(), none);
        assert_eq!(state(&app).view(), None, "another selection fits");
    }

    #[test]
    fn without_a_selection_the_preview_says_so_and_ok_is_greyed_out() {
        let (ctx, mut app) = opened(ToolId::FindDuplicates);
        assert!(mark(&ctx, "tracecut_hint").is_none());
        crate::ui::tips::start_recording();
        run(&ctx, &mut app, Vec::new(), egui::Modifiers::NONE);
        let why = crate::ui::tips::pair("dialog.ok", "tracecut.needs_selection");
        assert!(!crate::ui::tips::recorded().contains(&why));
        app.select(Vec::new());
        ctx.data_mut(|d| {
            d.remove::<crate::ui::before_after::Rects>(egui::Id::new("preview_rects"))
        });
        crate::ui::tips::start_recording();
        run(&ctx, &mut app, Vec::new(), egui::Modifiers::NONE);
        let hint = mark(&ctx, "tracecut_hint").expect("the hint is shown");
        let rect = mark(&ctx, "tracecut_preview").expect("the preview");
        assert!(rect.contains_rect(hint));
        assert!((hint.center() - rect.center()).length() < 1.0, "centred");
        assert!(
            crate::ui::tips::recorded().contains(&why),
            "OK says why it is grey"
        );
        assert_eq!(
            tr("tracecut.empty_hint"),
            "Select the objects to cut, nest or measure"
        );
    }

    #[test]
    fn the_measure_tools_show_their_results_without_apply() {
        for tool in [ToolId::Measure, ToolId::CuttingTime, ToolId::Cost] {
            let (ctx, app) = opened(tool);
            let st = state(&app);
            let report = match st.worked.as_ref().map(|w| &w.outcome) {
                Some(Outcome::Report(r)) => r.clone(),
                other => panic!("{tool:?}: {other:?}"),
            };
            assert!(app.tracecut_current(st.worked.as_ref().expect("w"), tool, &st.settings));
            assert!(mark(&ctx, "tracecut_report").is_some(), "{tool:?} shows it");
            if tool == ToolId::Measure {
                // Two 30 mm squares: 240 mm round, 1800 mm².
                let all = report.lines.join("\n");
                let u = app.units;
                let len = trf(
                    "tracecut.measure_length",
                    &[("v", &crate::tracecut_cut::fmt_len(u, 240.0))],
                );
                let k = u.world_from_mm(1.0);
                let area = format!("{:.2} {}²", 1800.0 * k * k, u.short());
                assert_eq!(report.lines.first(), Some(&len), "{all}");
                assert!(all.contains(&area), "{area} in {all}");
            }
        }
    }
}
