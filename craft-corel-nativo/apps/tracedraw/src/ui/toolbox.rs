//! The vertical toolbox: one button
//! per flyout showing the tool last used from it, a small arrow at the
//! bottom-right of the buttons that have a flyout, tooltips with the tool
//! name, shortcut and what the tool does, and vertical flyouts listing
//! each tool's icon, name and shortcut with separators between the kinds
//! of tools. Once a flyout is open, hovering another flyout button opens
//! that one instead. The button at the end shows or hides flyouts.

use crate::app::App;
use crate::i18n::tr;
use crate::theme::{self, Tokens};
use crate::tools::{Tool, GROUPS};
use crate::ui::icons;
use crate::ui::tips::RichTip;
use egui::{Pos2, Rect, Sense, Stroke, Ui, Vec2};

/// Tools after which a flyout draws a separator.
const SEPARATE_AFTER: [Tool; 6] = [
    Tool::FreeformPick,
    Tool::SegmentDelete,
    Tool::CommonShapes,
    Tool::SegmentDimension,
    Tool::RoundedConnector,
    Tool::AreaFill,
];

/// Toolbox groups followed by a separator line: the Shape, Zoom, Polygon,
/// Text, Connector, Transparency and Fill groups. The Variable Outline
/// button sits in the drawing block, under the brush tools, with no line
/// between them.
const GROUP_BREAKS: [usize; 7] = [1, 3, 9, 10, 12, 14, 16];

/// Whether group `gi` is shown (the Outline flyout is hidden by default).
pub fn group_visible(app: &App, gi: usize) -> bool {
    let Some(group) = GROUPS.get(gi) else {
        return false;
    };
    if group.hidden {
        return app.settings.show_outline_flyout;
    }
    !app.settings
        .toolbox_hidden
        .iter()
        .any(|id| id == group.tools[0].id())
}

/// The tooltip of a tool: its name and shortcut in bold, then what it does.
pub fn tool_tooltip(ui: &mut Ui, tool: Tool) {
    let head = if tool.shortcut_label().is_empty() {
        crate::i18n::trf("toolbox.tool_title", &[("t", &tool.name())])
    } else {
        format!(
            "{} ({})",
            crate::i18n::trf("toolbox.tool_title", &[("t", &tool.name())]),
            tool.shortcut_label()
        )
    };
    ui.set_max_width(260.0);
    ui.label(egui::RichText::new(head).font(theme::bold(12.0)));
    ui.label(egui::RichText::new(tr(&format!("tooldesc.{}", tool.id()))).size(12.0));
}

/// Buttons per row a floating toolbox starts with.
pub const FLOATING_COLUMNS: usize = 2;

/// The width of a floating toolbox's grid of buttons at first: two
/// buttons wide.
pub fn floating_width(app: &App) -> f32 {
    FLOATING_COLUMNS as f32 * button_width(app) + (FLOATING_COLUMNS - 1) as f32 * FLOW_GAP
}

/// The space between the buttons of a floating toolbox, across and down.
pub const FLOW_GAP: f32 = 1.0;
/// The cell a separator takes between two groups in a row: the line and
/// two pixels of space on each side.
pub const SEP_CELL: f32 = 5.0;
/// The line between two rows of a floating toolbox where a group ends.
pub const LINE_ROW: f32 = 3.0;

/// How the buttons of a floating toolbox flow in the room asked for: the
/// buttons in each row, and the room the rows take.
#[derive(Debug, Clone, PartialEq)]
pub struct Flow {
    /// Buttons in each row, top to bottom.
    pub rows: Vec<usize>,
    /// The widest row and the rows' height together: buttons, the gaps
    /// between them, the separator cells in the rows and the lines
    /// between the rows.
    pub size: Vec2,
}

impl Flow {
    /// The index of the first button of each row.
    pub fn row_starts(&self) -> Vec<usize> {
        let mut starts = Vec::with_capacity(self.rows.len());
        let mut at = 0;
        for n in &self.rows {
            starts.push(at);
            at += n;
        }
        starts
    }
}

/// The buttons in each row of a toolbox `width` wide, left to right then
/// down: a button goes on the row while it fits, with [`FLOW_GAP`]
/// before it and a separator cell ([`SEP_CELL`] and a gap) when a group
/// ends before it; else it starts the next row. `breaks[i]` says a group
/// ends after button `i`. A row always holds at least one button.
pub fn flow_rows(width: f32, button_w: f32, breaks: &[bool]) -> Vec<usize> {
    let mut rows = Vec::new();
    let mut in_row = 0usize;
    let mut row_w = 0.0f32;
    for i in 0..breaks.len() {
        let sep_before = i > 0 && breaks.get(i - 1).copied().unwrap_or(false);
        let need = if sep_before {
            FLOW_GAP + SEP_CELL + FLOW_GAP + button_w
        } else {
            FLOW_GAP + button_w
        };
        if in_row > 0 && row_w + need > width + 0.5 {
            rows.push(in_row);
            in_row = 0;
        }
        if in_row == 0 {
            row_w = button_w;
        } else {
            row_w += need;
        }
        in_row += 1;
    }
    if in_row > 0 {
        rows.push(in_row);
    }
    rows
}

/// The room `rows` take with buttons `button` large: the widest row
/// across, and down the rows with a gap between them and a line
/// ([`LINE_ROW`]) where a group ends at a row's end.
pub fn flow_size(rows: &[usize], button: Vec2, breaks: &[bool]) -> Vec2 {
    let mut width = 0.0f32;
    let mut height = 0.0f32;
    let mut at = 0usize;
    for (r, n) in rows.iter().enumerate() {
        if r > 0 {
            height += FLOW_GAP;
            if at > 0 && breaks.get(at - 1).copied().unwrap_or(false) {
                height += LINE_ROW + FLOW_GAP;
            }
        }
        let seps = (at..at + n.saturating_sub(1))
            .filter(|i| breaks.get(*i).copied().unwrap_or(false))
            .count();
        let w = *n as f32 * button.x
            + n.saturating_sub(1) as f32 * FLOW_GAP
            + seps as f32 * (SEP_CELL + FLOW_GAP);
        width = width.max(w);
        height += button.y;
        at += n;
    }
    Vec2::new(width, height)
}

/// The flow of a floating toolbox asked to be `asked` large: the buttons
/// flow in the width asked for; when the rows then stand taller than the
/// height asked for (by more than half a button, so a grid kept in cells
/// holds its columns when its buttons change size), the rows get wider
/// (a button at a time) until they fit it, or until they are one row. A
/// tall narrow window so gives two columns (or one), a wider one more
/// columns and fewer rows, a wide short one a row or two. The flow is
/// never narrower than one button.
pub fn flow(asked: Vec2, button: Vec2, breaks: &[bool]) -> Flow {
    let button = Vec2::new(button.x.max(1.0), button.y.max(1.0));
    let mut width = if asked.x.is_finite() {
        asked.x.max(button.x)
    } else {
        button.x
    };
    let fit_height = if asked.y.is_finite() {
        asked.y + button.y / 2.0
    } else {
        f32::INFINITY
    };
    let mut rows = flow_rows(width, button.x, breaks);
    let mut size = flow_size(&rows, button, breaks);
    for _ in 0..breaks.len() {
        if rows.len() <= 1 || size.y <= fit_height + 0.5 {
            break;
        }
        width += button.x + FLOW_GAP;
        rows = flow_rows(width, button.x, breaks);
        size = flow_size(&rows, button, breaks);
    }
    Flow { rows, size }
}

/// Where a group ends in the floating toolbox's row of buttons: one entry
/// per button shown, the "+" button last (no separator after it).
pub fn floating_breaks(app: &App) -> Vec<bool> {
    let mut breaks: Vec<bool> = (0..GROUPS.len())
        .filter(|gi| group_visible(app, *gi))
        .map(|gi| GROUP_BREAKS.contains(&gi))
        .collect();
    // The "+" at the end.
    breaks.push(false);
    breaks
}

/// A cell of the floating toolbox's grid: a button and the gap after it.
/// The room the grid asks for is kept in cells (`bars.toolbox_size`), so
/// a toolbox three buttons wide stays three wide when its buttons
/// change size.
pub fn floating_cell(app: &App) -> Vec2 {
    Vec2::new(
        button_width(app) + FLOW_GAP,
        Tokens::tool_button() + FLOW_GAP,
    )
}

/// The room a floating toolbox's grid asks for (points): the size it
/// was last resized to (kept in the preferences, in cells), else two
/// buttons wide and as tall as it needs.
pub fn floating_asked(app: &App) -> Vec2 {
    let cell = floating_cell(app);
    match app.settings.bars.toolbox_size {
        Some([w, h]) if w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0 => {
            Vec2::new(w * cell.x, h * cell.y)
        }
        _ => Vec2::new(floating_width(app), f32::INFINITY),
    }
}

/// A height (points) no grid reaches: asked for while a side edge of
/// the floating toolbox is dragged, so the rows take the height they
/// need. Finite, so the preferences can hold it should the drag be cut
/// short.
pub const FREE_HEIGHT: f32 = 1.0e6;

/// `room` (points) as the preferences keep it: in cells of the grid.
pub fn floating_cells(app: &App, room: Vec2) -> [f32; 2] {
    let cell = floating_cell(app);
    [
        (room.x / cell.x).max(f32::EPSILON),
        (room.y / cell.y).max(f32::EPSILON),
    ]
}

/// The flow of the floating toolbox as it is now set up.
pub fn floating_flow(app: &App) -> Flow {
    let button = Vec2::new(button_width(app), Tokens::tool_button());
    flow(floating_asked(app), button, &floating_breaks(app))
}

/// The room a caption takes on each side of its text.
const CAPTION_PAD: f32 = 6.0;

/// The width of a toolbox button: its side, plus the widest tool name
/// (as measured when the toolbox was last drawn) when captions show
/// (the bars' menu > Customize > Toolbox Toolbar); a caption alone
/// takes the width of the name.
pub fn button_width(app: &App) -> f32 {
    let captions = app.settings.toolbox_captions;
    let side = Tokens::tool_button();
    if !captions.caption() {
        return side;
    }
    let text = app.toolbox_caption_w + 2.0 * CAPTION_PAD;
    if captions.image() {
        side + text
    } else {
        text.max(side)
    }
}

/// The width of a docked toolbox column (its buttons and the margins).
pub fn column_width(app: &App) -> f32 {
    button_width(app) + 6.0
}

/// The widest tool name among the flyouts shown (any tool of each, so
/// the buttons keep their width whichever tool they show), in the
/// caption font.
fn widest_caption(app: &App, ui: &Ui) -> f32 {
    let font = caption_font();
    ui.ctx().fonts_mut(|f| {
        GROUPS
            .iter()
            .enumerate()
            .filter(|(gi, _)| group_visible(app, *gi))
            .flat_map(|(_, g)| g.tools.iter())
            .map(|t| {
                f.layout_no_wrap(t.name(), font.clone(), egui::Color32::BLACK)
                    .size()
                    .x
            })
            .fold(0.0, f32::max)
    })
}

fn caption_font() -> egui::FontId {
    egui::FontId::proportional(12.0)
}

/// The toolbox, docked at `dock`: a row along the top or the bottom, a
/// column on a side, or floating as a grid two buttons wide.
pub fn toolbox(app: &mut App, ui: &mut Ui, dock: crate::ui::bars::Dock) {
    use crate::ui::bars::Dock;
    // Nothing to draw on the Welcome Screen: the tools show pale.
    if !app.drawing_shown() {
        ui.disable();
    }
    // A right click on the toolbox's empty part opens its menu, as on the
    // other bars.
    crate::ui::bars::background(app, ui, crate::ui::bars::Bar::Toolbox);
    if dock != Dock::Floating {
        size_by_wheel(app, ui, ui.max_rect());
    }
    match dock {
        Dock::Top | Dock::Bottom => {
            crate::ui::overflow::row(ui, "toolbox_row", |ui| tools(app, ui, dock, None));
        }
        Dock::Floating => floating_grid(app, ui),
        Dock::Left | Dock::Right => {
            // Buttons shorter, down to a floor, so the column fits the
            // window without scrolling (#494).
            let room = ui.available_rect_before_wrap().height();
            let side = fitted_side(Tokens::tool_button(), visible_groups(app), room);
            crate::ui::overflow::column(ui, "toolbox_column", |ui| {
                tools(app, ui, dock, Some(side))
            });
        }
    }
}

/// Ctrl+wheel over the toolbox (in `rect`) steps its buttons' size: up
/// larger, down smaller, from the size it has (its own, or the default
/// button size it follows), as the bars' menu > Customize > Toolbox
/// Toolbar > Button Size does. The wheel is taken, so the toolbox does
/// not slide with it and the drawing does not zoom.
pub fn size_by_wheel(app: &mut App, ui: &Ui, rect: Rect) {
    if let Some(steps) =
        crate::ui::bars::ctrl_wheel_steps(ui, rect, egui::Id::new("toolbox_size_wheel"))
    {
        let now = app
            .settings
            .toolbox_button_size
            .unwrap_or(app.settings.button_size);
        app.settings.toolbox_button_size = Some(now.stepped(steps));
    }
}

/// The height a column toolbox's buttons get so `n` of them (with the
/// grip) fit `room`: their side, less when that does not fit, never under
/// [`MIN_FITTED_SIDE`] (the overflow arrows take over from there).
pub fn fitted_side(side: f32, n: usize, room: f32) -> f32 {
    const GRIP: f32 = 9.0;
    if n == 0 || !room.is_finite() {
        return side;
    }
    let n = n as f32;
    let fit = ((room - GRIP - (n - 1.0)) / n).floor();
    fit.clamp(MIN_FITTED_SIDE.min(side), side)
}

/// A flyout row's icon box and height at the toolbox's button size:
/// 18 and 24 px up to Medium, growing with the larger sizes (24 and 30
/// px at Large, 30 and 36 px at Extra Large) so the list matches the
/// buttons that open it.
pub fn flyout_metrics() -> (f32, f32) {
    flyout_metrics_for(Tokens::tool_icon())
}

/// [`flyout_metrics`] for toolbox icons of side `tool_icon`.
pub fn flyout_metrics_for(tool_icon: f32) -> (f32, f32) {
    let icon = (tool_icon * 0.75).round().max(18.0);
    (icon, (icon + 6.0).max(24.0))
}

/// The shortest a column toolbox's buttons get to fit the window.
pub const MIN_FITTED_SIDE: f32 = 22.0;

/// How many buttons the toolbox shows.
fn visible_groups(app: &App) -> usize {
    (0..GROUPS.len())
        .filter(|gi| group_visible(app, *gi))
        .count()
}

fn tools(app: &mut App, ui: &mut Ui, dock: crate::ui::bars::Dock, fitted: Option<f32>) {
    use crate::ui::bars::Bar;
    let horizontal = dock.horizontal();
    ui.spacing_mut().item_spacing = if horizontal {
        Vec2::new(1.0, 0.0)
    } else {
        Vec2::new(0.0, 1.0)
    };
    crate::ui::bars::grip(app, ui, Bar::Toolbox, horizontal);
    // Captions: the icon, the tool's name, or both on each button; the
    // names measured now size the buttons from the next frame.
    let captions = app.settings.toolbox_captions;
    if captions.caption() {
        app.toolbox_caption_w = widest_caption(app, ui);
    }
    let side = fitted.unwrap_or_else(Tokens::tool_button);
    for (gi, _) in GROUPS.iter().enumerate() {
        if !group_visible(app, gi) {
            continue;
        }
        tool_button(app, ui, gi, side, captions, dock);
        if GROUP_BREAKS.contains(&gi) {
            ui.add_space(2.0);
            if horizontal {
                let (r, _) = ui.allocate_exact_size(Vec2::new(1.0, side), Sense::hover());
                ui.painter().vline(
                    r.center().x,
                    r.y_range().shrink(4.0),
                    Stroke::new(1.0, Tokens::border()),
                );
            } else {
                let (r, _) =
                    ui.allocate_exact_size(Vec2::new(button_width(app), 1.0), Sense::hover());
                ui.painter().hline(
                    r.x_range().shrink(4.0),
                    r.center().y,
                    Stroke::new(1.0, Tokens::border()),
                );
            }
            ui.add_space(2.0);
        }
    }
    quick_customize(app, ui, horizontal);
}

/// The floating toolbox's grid: its buttons in the rows [`flow`] gives
/// for the room asked (see [`floating_asked`]), a separator between two
/// groups in a row and a line between two rows where a group ends, the
/// "+" button last; the window hugs the grid, and what the grid took
/// is kept as the room asked, so the layout holds from frame to frame.
fn floating_grid(app: &mut App, ui: &mut Ui) {
    use crate::ui::bars::Dock;
    let captions = app.settings.toolbox_captions;
    if captions.caption() {
        app.toolbox_caption_w = widest_caption(app, ui);
    }
    let side = Tokens::tool_button();
    let bw = button_width(app);
    let breaks = floating_breaks(app);
    let flow = floating_flow(app);
    let starts = flow.row_starts();
    let visible: Vec<usize> = (0..GROUPS.len())
        .filter(|gi| group_visible(app, *gi))
        .collect();
    let top_left = ui.cursor().min;
    ui.spacing_mut().item_spacing = Vec2::new(FLOW_GAP, FLOW_GAP);
    ui.vertical(|ui| {
        for (r, start) in starts.iter().enumerate() {
            let n = flow.rows.get(r).copied().unwrap_or(0);
            if r > 0 && *start > 0 && breaks.get(start - 1).copied().unwrap_or(false) {
                // A line across the grid where a group ends at a row's end.
                let (lr, _) =
                    ui.allocate_exact_size(Vec2::new(flow.size.x, LINE_ROW), Sense::hover());
                ui.painter().hline(
                    lr.x_range().shrink(4.0),
                    lr.center().y,
                    Stroke::new(1.0, Tokens::border()),
                );
            }
            ui.horizontal(|ui| {
                for i in *start..start + n {
                    match visible.get(i) {
                        Some(gi) => tool_button(app, ui, *gi, side, captions, Dock::Floating),
                        // The "+" after the last group, a cell of its own.
                        None => {
                            ui.allocate_ui_with_layout(
                                Vec2::new(bw, side),
                                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                |ui| quick_customize_button(app, ui),
                            );
                        }
                    }
                    let last_in_row = i + 1 >= start + n;
                    if !last_in_row && breaks.get(i).copied().unwrap_or(false) {
                        // A separator between two groups in the row.
                        let (sr, _) =
                            ui.allocate_exact_size(Vec2::new(SEP_CELL, side), Sense::hover());
                        ui.painter().vline(
                            sr.center().x,
                            sr.y_range().shrink(4.0),
                            Stroke::new(1.0, Tokens::border()),
                        );
                    }
                }
            });
        }
    });
    let grid = Rect::from_min_size(top_left, flow.size);
    ui.ctx()
        .data_mut(|d| d.insert_temp(floating_grid_id(), grid));
    // The grid's own size is what the window asks for from now on, so a
    // size dragged past what the buttons need settles on the grid.
    if !crate::ui::bars::toolbox_resizing(ui.ctx()) {
        let hug = floating_cells(app, flow.size);
        if app.settings.bars.toolbox_size != Some(hug) {
            app.settings.bars.toolbox_size = Some(hug);
        }
    }
}

/// Where the floating toolbox's grid of buttons was drawn this frame.
pub fn floating_grid_id() -> egui::Id {
    egui::Id::new("floating_toolbox_grid")
}

/// One toolbox button, for group `gi`: `side` high and as wide as
/// [`button_width`], showing the tool last used from the group (its icon,
/// its name or both by `captions`), with the flyout arrow and the
/// flyout itself when open.
fn tool_button(
    app: &mut App,
    ui: &mut Ui,
    gi: usize,
    side: f32,
    captions: crate::settings::Captions,
    dock: crate::ui::bars::Dock,
) {
    let Some(group) = GROUPS.get(gi) else {
        return;
    };
    let size = Vec2::new(button_width(app), side);
    // The button shows the tool last used in this group, even while a
    // tool of another group is active.
    let shown = if group.tools.contains(&app.tool) {
        app.tool
    } else {
        app.toolbox_last
            .get(gi)
            .copied()
            .filter(|t| group.tools.contains(t))
            .unwrap_or(group.tools[0])
    };
    let active = group.tools.contains(&app.tool);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let has_flyout = group.tools.len() > 1;
    let arrow_zone = Rect::from_min_max(rect.right_bottom() - Vec2::splat(10.0), rect.max);
    // The active tool's button is white with a grey frame; hovering
    // tints a button blue.
    let bg = if active {
        crate::theme::Tokens::field()
    } else if resp.hovered() {
        Tokens::tool_hover()
    } else {
        Tokens::panel()
    };
    ui.painter().rect_filled(rect, 0.0, bg);
    if active {
        ui.painter().rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, Tokens::control_border()),
            egui::StrokeKind::Inside,
        );
    }
    // The icon's box grows with the button size (16 to 40 px), shrunk
    // with a button squeezed to fit a short column.
    let icon_side = crate::theme::icon_in(side, Tokens::tool_button(), Tokens::tool_icon());
    if captions.image() {
        let icon_rect = Rect::from_center_size(
            egui::pos2(rect.left() + side / 2.0, rect.center().y),
            Vec2::splat(icon_side),
        );
        icons::draw(
            ui.painter(),
            icon_rect,
            shown,
            icons::tool_color(shown, Tokens::icon()),
        );
    }
    if captions.caption() {
        let left = if captions.image() {
            rect.left() + side + CAPTION_PAD
        } else {
            rect.left() + CAPTION_PAD
        };
        ui.painter().text(
            egui::pos2(left, rect.center().y),
            egui::Align2::LEFT_CENTER,
            shown.name(),
            caption_font(),
            Tokens::text(),
        );
    }
    if has_flyout {
        // The flyout arrow: a small triangle at the bottom-right.
        let p = rect.right_bottom() - Vec2::new(2.0, 2.0);
        ui.painter().add(egui::epaint::PathShape::convex_polygon(
            vec![p, p - Vec2::new(5.0, 0.0), p - Vec2::new(0.0, 5.0)],
            Tokens::icon(),
            Stroke::NONE,
        ));
    }
    // No tooltip while a flyout is open: it would cover the list.
    let resp = if app.flyout_open.is_none() {
        resp.on_hover_ui(|ui| tool_tooltip(ui, shown))
    } else {
        resp
    };
    let on_arrow = resp
        .interact_pointer_pos()
        .is_some_and(|p| arrow_zone.contains(p));

    // Opening the flyout: the arrow, a drag, or a press
    // held for 0.4 s; with a flyout already open, hovering opens this
    // one instead. Not by a finger sliding the toolbox along its length
    // (#1997): the button under it is only passing by.
    if has_flyout {
        let held = resp.is_pointer_button_down_on()
            && ui.input(|i| {
                let started = i.pointer.press_start_time().unwrap_or(i.time);
                i.time - started >= 0.4
            });
        if resp.is_pointer_button_down_on() && !held {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }
        let switch = resp.hovered() && app.flyout_open.is_some_and(|o| o != gi);
        let pressed = (resp.drag_started() || resp.long_touched() || held)
            && !crate::ui::overflow::finger_sliding(ui.ctx());
        if (resp.clicked() && on_arrow) || pressed || switch {
            app.flyout_open = Some(gi);
        }
    }
    // A right click on a tool button opens the toolbox's menu
    // (customize, button size, the bars), as on the other bars.
    crate::ui::bars::bar_menu(app, &resp, crate::ui::bars::Bar::Toolbox);
    // A plain click picks the shown tool (not when it opened the flyout).
    if resp.clicked() && !(has_flyout && on_arrow) && app.flyout_open != Some(gi) {
        app.choose_tool(shown);
    }
    // Double-clicking a tool button: Pick selects every object, Shape
    // every node, Zoom fits the drawing, Rectangle adds a frame around
    // the page (with Shift, around the selection).
    if resp.double_clicked() {
        match shown {
            Tool::Pick => app.select_all(),
            Tool::Zoom => app.zoom_to_fit(),
            Tool::Rectangle if ui.input(|i| i.modifiers.shift) => app.add_selection_frame(),
            Tool::Rectangle => app.add_page_frame(),
            // The Shape tool selects every node of the selected curves.
            Tool::Shape => app.select_all_nodes(),
            _ => {}
        }
    }
    if app.flyout_open == Some(gi) {
        flyout(app, ui, gi, rect, dock);
    }
}

/// The flyout of group `gi`, opened beside its button: to the right of a
/// toolbox on the left or floating, to the left of one on the right,
/// below one along the top and above one along the bottom.
fn flyout(app: &mut App, ui: &mut Ui, gi: usize, button: Rect, dock: crate::ui::bars::Dock) {
    use crate::ui::bars::Dock;
    let Some(group) = GROUPS.get(gi) else {
        return;
    };
    // The floating grid: beside the button while the grid is taller
    // than wide, under it while it is wider (a row or two).
    let grid: Option<Rect> = if dock == Dock::Floating {
        ui.ctx().data(|d| d.get_temp(floating_grid_id()))
    } else {
        None
    };
    let wide = grid.is_some_and(|g| g.width() > g.height());
    let (pos, pivot) = match dock {
        Dock::Right => (
            button.left_top() - Vec2::new(2.0, 0.0),
            egui::Align2::RIGHT_TOP,
        ),
        Dock::Top => (
            button.left_bottom() + Vec2::new(0.0, 2.0),
            egui::Align2::LEFT_TOP,
        ),
        Dock::Floating if wide => (
            button.left_bottom() + Vec2::new(0.0, 2.0),
            egui::Align2::LEFT_TOP,
        ),
        Dock::Bottom => (
            button.left_top() - Vec2::new(0.0, 2.0),
            egui::Align2::LEFT_BOTTOM,
        ),
        Dock::Left | Dock::Floating => (
            button.right_top() + Vec2::new(2.0, 0.0),
            egui::Align2::LEFT_TOP,
        ),
    };
    let shown = app.toolbox_last.get(gi).copied().unwrap_or(group.tools[0]);
    let (fly_icon, fly_row) = flyout_metrics();
    let mut close = false;
    let area = egui::Area::new(egui::Id::new(("flyout", gi)))
        .pivot(pivot)
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .fade_in(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .inner_margin(egui::Margin::symmetric(2, 3))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                    let row_w = group
                        .tools
                        .iter()
                        .map(|t| {
                            let name = ui.painter().layout_no_wrap(
                                t.name(),
                                egui::FontId::proportional(12.0),
                                Tokens::text(),
                            );
                            name.size().x
                        })
                        .fold(0.0_f32, f32::max)
                        + 98.0
                        + fly_icon;
                    for t in group.tools {
                        let (r, rr) =
                            ui.allocate_exact_size(Vec2::new(row_w, fly_row), Sense::click());
                        let current =
                            *t == app.tool || (*t == shown && !group.tools.contains(&app.tool));
                        if rr.hovered() {
                            ui.painter().rect_filled(r, 0.0, Tokens::tool_hover());
                        }
                        // A check mark beside the tool the button shows.
                        if current {
                            let c = Pos2::new(r.min.x + 9.0, r.center().y);
                            let s = Stroke::new(1.4, Tokens::icon());
                            ui.painter().line_segment(
                                [c + Vec2::new(-3.5, 0.0), c + Vec2::new(-1.0, 3.0)],
                                s,
                            );
                            ui.painter().line_segment(
                                [c + Vec2::new(-1.0, 3.0), c + Vec2::new(4.0, -3.5)],
                                s,
                            );
                        }
                        let icon = Rect::from_center_size(
                            Pos2::new(r.min.x + 21.0 + fly_icon / 2.0, r.center().y),
                            Vec2::splat(fly_icon),
                        );
                        icons::draw(
                            ui.painter(),
                            icon,
                            *t,
                            icons::tool_color(*t, Tokens::icon()),
                        );
                        ui.painter().text(
                            Pos2::new(r.min.x + 28.0 + fly_icon, r.center().y),
                            egui::Align2::LEFT_CENTER,
                            t.name(),
                            egui::FontId::proportional(12.0),
                            Tokens::text(),
                        );
                        if !t.shortcut_label().is_empty() {
                            ui.painter().text(
                                Pos2::new(r.max.x - 8.0, r.center().y),
                                egui::Align2::RIGHT_CENTER,
                                t.shortcut_label(),
                                egui::FontId::proportional(12.0),
                                Tokens::text(),
                            );
                        }
                        if rr.on_hover_ui(|ui| tool_tooltip(ui, *t)).clicked() {
                            app.choose_tool(*t);
                            close = true;
                        }
                        if SEPARATE_AFTER.contains(t) {
                            let (sr, _) =
                                ui.allocate_exact_size(Vec2::new(row_w, 7.0), Sense::hover());
                            ui.painter().hline(
                                sr.x_range().shrink(6.0),
                                sr.center().y,
                                Stroke::new(1.0, Tokens::border()),
                            );
                        }
                    }
                });
        });
    // Close when the pointer wanders away from the flyout and its button
    // (no click needed, so the next press reaches the canvas untouched),
    // or with a click elsewhere.
    let zone = area.response.rect.union(button).expand(24.0);
    let (away, clicked_elsewhere) = ui.input(|i| {
        let pos = i.pointer.latest_pos();
        let away = pos.map(|p| !zone.contains(p)).unwrap_or(false);
        let click = i.pointer.any_pressed() && pos.is_some_and(|p| !zone.contains(p));
        (away, click)
    });
    // Moving onto another toolbox button keeps a flyout open (it switches).
    let over_toolbox = ui.input(|i| i.pointer.latest_pos()).is_some_and(|p| {
        if dock == Dock::Floating {
            // The grid: any button of the floating toolbox.
            grid.is_some_and(|g| g.expand(4.0).contains(p))
        } else if dock.horizontal() {
            p.y <= button.max.y + 2.0 && p.y >= button.min.y - 4.0
        } else {
            p.x <= button.max.x + 2.0 && p.x >= button.min.x - 4.0
        }
    });
    if close || clicked_elsewhere || (away && !over_toolbox) {
        app.flyout_open = None;
    }
}

/// The "+" at the end of the toolbox: tick the flyouts to show.
fn quick_customize(app: &mut App, ui: &mut Ui, horizontal: bool) {
    ui.add_space(4.0);
    let layout = if horizontal {
        egui::Layout::left_to_right(egui::Align::Center)
    } else {
        egui::Layout::top_down(egui::Align::Center)
    };
    ui.with_layout(layout, |ui| quick_customize_button(app, ui));
}

/// The "+" button itself and its menu.
fn quick_customize_button(app: &mut App, ui: &mut Ui) {
    ui.visuals_mut().widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
    ui.menu_button(
        egui::RichText::new("+")
            .size(flyout_metrics().0)
            .color(Tokens::text_dim()),
        |ui| {
            for (gi, group) in GROUPS.iter().enumerate() {
                let mut on = group_visible(app, gi);
                let label = group
                    .tools
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(", ");
                if ui.checkbox(&mut on, label).changed() {
                    set_group_visible(app, gi, on);
                }
            }
            ui.separator();
            if ui.button(tr("toolbox.reset")).clicked() {
                app.settings.toolbox_hidden.clear();
                app.settings.show_outline_flyout = false;
                ui.close();
            }
        },
    )
    .response
    .rich_tip(tr("toolbox.quick_customize"));
}

fn set_group_visible(app: &mut App, gi: usize, on: bool) {
    let Some(group) = GROUPS.get(gi) else {
        return;
    };
    if group.hidden {
        app.settings.show_outline_flyout = on;
        return;
    }
    let id = group.tools[0].id().to_string();
    app.settings.toolbox_hidden.retain(|h| *h != id);
    if !on {
        app.settings.toolbox_hidden.push(id);
    }
}

/// The bars' menu > Customize > Toolbox Toolbar > Reset to Default: every
/// flyout shown (the Outline flyout hidden, as at first), each button
/// showing its first tool, the default button size and icons only.
pub fn reset_toolbox(app: &mut App) {
    app.settings.toolbox_hidden.clear();
    app.settings.show_outline_flyout = false;
    app.settings.toolbox_button_size = None;
    app.settings.toolbox_captions = crate::settings::Captions::ImageOnly;
    app.toolbox_last = GROUPS.iter().map(|g| g.tools[0]).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The buttons follow the default button size, or the toolbox's
    /// own; captions widen them by the widest tool name.
    #[test]
    fn buttons_follow_the_sizes_and_the_captions() {
        let _sizes = crate::theme::button_sizes_lock();
        use crate::settings::{ButtonSize, Captions};
        let mut app = App::headless();
        crate::theme::set_button_sizes(ButtonSize::Medium, None);
        assert_eq!(button_width(&app), 32.0);
        crate::theme::set_button_sizes(ButtonSize::Large, None);
        assert_eq!(button_width(&app), 42.0);
        crate::theme::set_button_sizes(ButtonSize::Large, Some(ButtonSize::Small));
        assert_eq!(button_width(&app), 26.0);
        assert_eq!(crate::theme::Tokens::bar_button(), 42.0);
        assert_eq!(crate::theme::Tokens::prop_button(), 38.0);
        app.toolbox_caption_w = 50.0;
        app.settings.toolbox_captions = Captions::CaptionRight;
        assert_eq!(button_width(&app), 26.0 + 50.0 + 2.0 * CAPTION_PAD);
        app.settings.toolbox_captions = Captions::CaptionOnly;
        assert_eq!(button_width(&app), 50.0 + 2.0 * CAPTION_PAD);
        assert_eq!(floating_width(&app), 2.0 * (50.0 + 2.0 * CAPTION_PAD) + 1.0);
        // Reset to Default: icons only, the default size, every flyout.
        app.settings.toolbox_button_size = Some(ButtonSize::Small);
        set_group_visible(&mut app, 2, false);
        reset_toolbox(&mut app);
        assert_eq!(app.settings.toolbox_captions, Captions::ImageOnly);
        assert_eq!(app.settings.toolbox_button_size, None);
        assert!(group_visible(&app, 2));
        crate::theme::set_button_sizes(ButtonSize::Medium, None);
    }

    /// The icons grow with the buttons at every size step, in the
    /// toolbox, on the standard and property bars and in the flyouts;
    /// a column squeezed to fit keeps the icon in proportion. (From the
    /// size table, not the sizes in force: other tests running at the
    /// same time read those.)
    #[test]
    fn icons_grow_with_every_button_size() {
        use crate::settings::ButtonSize;
        let mut last: Option<[f32; 7]> = None;
        for size in ButtonSize::ALL {
            let m = size.metrics();
            let (fly_icon, fly_row) = flyout_metrics_for(m.icon);
            let now = [
                m.button,
                crate::theme::icon_in(m.button, m.button, m.icon),
                m.prop_button,
                m.prop_icon,
                fly_icon,
                fly_row,
                m.bar_icon,
            ];
            assert_eq!(now[1], m.icon);
            if let Some(last) = last {
                assert!(now[6] > last[6], "{size:?} bar icon: {now:?} vs {last:?}");
                for i in 0..4 {
                    assert!(now[i] > last[i], "{size:?} {i}: {now:?} vs {last:?}");
                }
                assert!(now[4] >= last[4] && now[5] >= last[5]);
            }
            last = Some(now);
        }
        // The flyouts grow at the larger sizes and keep 18 px icons in
        // 24 px rows up to Medium.
        assert_eq!(flyout_metrics_for(24.0), (18.0, 24.0));
        assert_eq!(flyout_metrics_for(40.0), (30.0, 36.0));
        // Squeezed: 32 px icons in 42 px buttons draw 16 px in 21 px.
        assert_eq!(crate::theme::icon_in(21.0, 42.0, 32.0), 16.0);
        assert_eq!(crate::theme::icon_in(50.0, 42.0, 32.0), 32.0);
    }

    #[test]
    fn flyout_groups_can_be_hidden_and_reset() {
        let mut app = App::headless();
        assert!(group_visible(&app, 0));
        let outline = GROUPS.iter().position(|g| g.hidden).expect("outline group");
        assert!(!group_visible(&app, outline));
        set_group_visible(&mut app, 2, false);
        assert!(!group_visible(&app, 2));
        set_group_visible(&mut app, outline, true);
        assert!(group_visible(&app, outline));
        set_group_visible(&mut app, 2, true);
        assert!(group_visible(&app, 2));
    }

    /// The group that holds `tool`.
    fn group_of(tool: Tool) -> usize {
        GROUPS
            .iter()
            .position(|g| g.tools.contains(&tool))
            .expect("in a group")
    }

    /// The Variable Outline tool has a button of its own, right under the
    /// brush tools' flyout, inside the drawing block: no separator line
    /// between the two, and the Rectangle flyout follows it.
    #[test]
    fn the_variable_outline_button_sits_under_the_brush_tools() {
        let brushes = group_of(Tool::BrushStrokes);
        let vo = group_of(Tool::VariableOutline);
        assert_eq!(vo, brushes + 1);
        assert_eq!(GROUPS[vo].tools, &[Tool::VariableOutline]);
        assert!(!GROUPS[vo].hidden);
        assert_eq!(group_of(Tool::Rectangle), vo + 1);
        assert!(!GROUP_BREAKS.contains(&brushes));
        assert!(!GROUP_BREAKS.contains(&vo));
    }

    /// The separator lines follow the Shape, Zoom, Polygon, Text,
    /// Connector, Transparency and Fill groups, whatever their indices.
    #[test]
    fn separator_lines_follow_the_same_groups() {
        let after = [
            Tool::Shape,
            Tool::Zoom,
            Tool::Polygon,
            Tool::Text,
            Tool::Connector,
            Tool::Transparency,
            Tool::InteractiveFill,
        ]
        .map(group_of);
        assert_eq!(GROUP_BREAKS, after);
    }

    #[test]
    fn separators_split_the_toolbox_subgroups() {
        // Each separator tool sits inside a flyout and is not its last tool.
        for t in SEPARATE_AFTER {
            let g = GROUPS
                .iter()
                .find(|g| g.tools.contains(&t))
                .expect("in a group");
            assert_ne!(g.tools.last(), Some(&t), "{t:?}");
        }
    }

    /// The floating toolbox's buttons flow in the width asked for: two
    /// to a row in a window two buttons wide, more in a wider one, with
    /// a separator cell between two groups in a row and a line between
    /// two rows where a group ends; the rows widen (a button at a time)
    /// until they fit the height asked for.
    #[test]
    fn a_floating_toolbox_flows_to_the_room_asked_for() {
        // Six buttons; the groups end after the second and the fifth.
        let breaks = [false, true, false, false, true, false];
        let b = Vec2::splat(32.0);
        // Two buttons wide (65 px): two to a row until a separator cell
        // would not fit beside a button.
        let two = flow(Vec2::new(65.0, f32::INFINITY), b, &breaks);
        assert_eq!(two.rows, vec![2, 2, 1, 1]);
        assert_eq!(two.row_starts(), vec![0, 2, 4, 5]);
        // The widest row, and the rows with a gap between them and a
        // line (and its gaps) after the second and the fifth button.
        assert_eq!(two.size, Vec2::new(65.0, 32.0 + 37.0 + 33.0 + 37.0));
        // Wider: five in the first row (a separator cell among them),
        // the sixth on its own, under a line.
        let wide = flow(Vec2::new(200.0, f32::INFINITY), b, &breaks);
        assert_eq!(wide.rows, vec![5, 1]);
        assert_eq!(
            wide.size,
            Vec2::new(5.0 * 32.0 + 4.0 + 6.0, 32.0 + 1.0 + 4.0 + 32.0)
        );
        // Wide enough for one row: no lines at all.
        let row = flow(Vec2::new(250.0, f32::INFINITY), b, &breaks);
        assert_eq!(row.rows, vec![6]);
        assert_eq!(row.size, Vec2::new(6.0 * 32.0 + 5.0 + 12.0, 32.0));
        // A short window: the rows widen until they fit its height (with
        // half a button of slack).
        assert_eq!(
            flow(Vec2::new(65.0, 125.0), b, &breaks).rows,
            vec![2, 2, 1, 1]
        );
        let short = flow(Vec2::new(65.0, 100.0), b, &breaks);
        assert_eq!(short.rows, vec![2, 3, 1]);
        assert_eq!(
            short.size,
            Vec2::new(3.0 * 32.0 + 2.0, 32.0 + 5.0 + 32.0 + 5.0 + 32.0)
        );
        assert_eq!(flow(Vec2::new(65.0, 60.0), b, &breaks).rows, vec![3, 3]);
        // Shorter than any two rows: one row.
        assert_eq!(flow(Vec2::new(65.0, 10.0), b, &breaks).rows, vec![6]);
        // Narrower than a button: one to a row.
        let one = flow(Vec2::new(10.0, f32::INFINITY), b, &breaks);
        assert_eq!(one.rows, vec![1; 6]);
        assert_eq!(one.size, Vec2::new(32.0, 6.0 * 32.0 + 5.0 + 8.0));
        // Nothing to lay out takes no room.
        assert_eq!(flow(Vec2::new(65.0, 100.0), b, &[]).size, Vec2::ZERO);
    }

    /// The toolbox floats two buttons wide at first; the size it was
    /// resized to (kept in the preferences) gives it more to a row, and
    /// an unusable size is as none.
    #[test]
    fn the_floating_toolbox_starts_two_buttons_wide_and_keeps_its_size() {
        let mut app = App::headless();
        // The button sizes are shared by the process (a frame sets them
        // from the settings): this test draws no frame, so it sets them.
        crate::theme::set_button_sizes(app.settings.button_size, app.settings.toolbox_button_size);
        let shown = visible_groups(&app) + 1;
        assert_eq!(floating_breaks(&app).len(), shown);
        assert_eq!(floating_asked(&app).x, floating_width(&app));
        // The room is kept in cells: a button and its gap each.
        let cell = floating_cell(&app);
        assert_eq!(
            cell,
            Vec2::new(button_width(&app) + 1.0, Tokens::tool_button() + 1.0)
        );
        assert_eq!(floating_cells(&app, cell * 3.0), [3.0, 3.0]);
        app.settings.bars.toolbox_size = Some([3.0, 2.0]);
        assert_eq!(floating_asked(&app), cell * Vec2::new(3.0, 2.0));
        app.settings.bars.toolbox_size = None;
        let first = floating_flow(&app);
        assert!(first.rows.iter().all(|n| *n <= 2), "{:?}", first.rows);
        assert_eq!(first.rows.iter().sum::<usize>(), shown);
        app.settings.bars.toolbox_size = Some([10_000.0, 10_000.0]);
        assert_eq!(floating_flow(&app).rows, vec![shown]);
        app.settings.bars.toolbox_size = Some([f32::NAN, 5.0]);
        assert_eq!(floating_flow(&app), first);
        app.settings.bars.toolbox_size = Some([0.0, 5.0]);
        assert_eq!(floating_flow(&app), first);
        // The window: the grid, its margins and the title, never
        // narrower than the title's least width.
        app.settings.bars.toolbox_size = None;
        let window = crate::ui::bars::toolbox_window_size(&app);
        assert!(window.x >= crate::ui::bars::FLOATING_MIN_W);
        assert_eq!(
            window.y,
            first.size.y + crate::ui::bars::TOOLBOX_MARGIN.y + crate::ui::bars::TITLE_H
        );
    }

    /// A column toolbox fits the window: its buttons get shorter as the
    /// room shrinks, down to a floor, and keep their size when they fit
    /// (#494).
    #[test]
    fn a_column_toolbox_fits_the_window() {
        // 20 buttons of 32 px with the grip need 9 + 640 + 19 = 668.
        assert_eq!(fitted_side(32.0, 20, 900.0), 32.0);
        assert_eq!(fitted_side(32.0, 20, 668.0), 32.0);
        assert_eq!(fitted_side(32.0, 20, 600.0), 28.0);
        assert_eq!(fitted_side(32.0, 20, 300.0), MIN_FITTED_SIDE);
        // A small toolbox never grows, and nothing to fit changes nothing.
        assert_eq!(fitted_side(20.0, 20, 300.0), 20.0);
        assert_eq!(fitted_side(32.0, 0, 10.0), 32.0);
    }

    /// The toolbox on a side, taller than the window: a finger dragged up
    /// its column slides the column with the finger, so a tap at one spot
    /// picks another tool after the slide than before; the slide opens no
    /// flyout and picks nothing itself (#1997).
    #[test]
    fn a_finger_dragged_along_the_toolbox_column_slides_it() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.show_welcome = false;
        let clock = std::cell::Cell::new(0.0);
        let frame = |app: &mut App, events: Vec<egui::Event>| {
            clock.set(clock.get() + 0.02);
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(40.0, 200.0))),
                events,
                time: Some(clock.get()),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| toolbox(app, ui, crate::ui::bars::Dock::Left));
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

        for _ in 0..3 {
            frame(&mut app, Vec::new());
        }
        assert!(
            visible_groups(&app) as f32 * MIN_FITTED_SIDE > 200.0,
            "the column is taller than the window"
        );
        let spot = egui::pos2(20.0, 60.0);
        drag(&mut app, spot, spot, 1, true, true);
        let before = app.tool;
        // Up by 110 px: the first 10 decide the slide, 100 move the column.
        drag(
            &mut app,
            egui::pos2(20.0, 150.0),
            egui::pos2(20.0, 40.0),
            11,
            true,
            true,
        );
        assert_eq!(app.tool, before, "the slide picked nothing");
        assert_eq!(app.flyout_open, None, "the slide opened no flyout");
        drag(&mut app, spot, spot, 1, true, true);
        assert_ne!(app.tool, before, "another tool is under the spot now");
    }
}
