//! The document tabs above the rulers: the Welcome Screen, one tab per open
//! drawing (an asterisk marks unsaved changes; the close button shows on
//! the hovered tab) and the New tab after the last one, over a blue line
//! that runs the width of the window. The Welcome Screen tab closes to its
//! house alone, which shows the Welcome Screen again (#677).

use crate::app::App;
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::ui::tips::RichTip;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

/// Height of a tab; the blue line adds one pixel under it.
pub const TAB_HEIGHT: f32 = 26.0;
const PAD: f32 = 15.0;
const CLOSE: f32 = 14.0;
const FONT: f32 = 13.0;
/// The line under the tabs.
const LINE: Color32 = Color32::from_rgb(0x00, 0xAD, 0xFE);

/// What a click on the strip asked for.
enum Action {
    Welcome,
    CloseWelcome,
    Switch(usize),
    Close(usize),
    New,
}

pub fn document_tabs(app: &mut App, ui: &mut Ui) {
    let mut action = None;
    let strip = ui.max_rect();
    #[cfg(test)]
    ui.ctx()
        .data_mut(|d| d.insert_temp(egui::Id::new("document_tabs_strip"), strip));
    // Nothing below the line: the strip ends where it does.
    ui.spacing_mut().item_spacing.y = 0.0;
    // The active tab, when it changed since the last frame: the strip
    // slides to show it (a drawing opened or chosen from the Window
    // menu).
    let shown_id = egui::Id::new("document_tabs_active");
    let active_now = (!app.show_welcome).then_some(app.active_doc);
    let changed = ui.ctx().data(|d| d.get_temp::<Option<usize>>(shown_id)) != Some(active_now);
    ui.ctx().data_mut(|d| d.insert_temp(shown_id, active_now));
    // More tabs than the window is wide: the strip slides with its « and
    // » buttons and the mouse wheel (#1908).
    crate::ui::overflow::row_showing(ui, "document_tabs", |ui| {
        ui.spacing_mut().item_spacing.x = 1.0;
        let mut keep = None;
        let welcome = app.show_welcome;
        // Shown again, the Welcome Screen has its whole tab back.
        if welcome && app.settings.welcome_tab_closed {
            app.settings.welcome_tab_closed = false;
        }
        match welcome_tab(ui, welcome, app.settings.welcome_tab_closed) {
            TabClick::Select => action = Some(Action::Welcome),
            TabClick::Close => action = Some(Action::CloseWelcome),
            TabClick::None => {}
        }
        for i in 0..app.docs.len() {
            let title = app.document_tab_title(i);
            let active = !app.show_welcome && i == app.active_doc;
            let (click, rect) = doc_tab(ui, i, &title, active);
            if active && changed {
                keep = Some(rect);
            }
            match click {
                TabClick::Select => action = Some(Action::Switch(i)),
                TabClick::Close => action = Some(Action::Close(i)),
                TabClick::None => {}
            }
        }
        if new_tab(ui) {
            action = Some(Action::New);
        }
        keep
    });
    let line_y = strip.top() + TAB_HEIGHT;
    ui.painter().rect_filled(
        Rect::from_min_max(
            Pos2::new(strip.left(), line_y),
            Pos2::new(strip.right(), line_y + 1.0),
        ),
        0.0,
        LINE,
    );
    ui.allocate_space(Vec2::new(0.0, 1.0));
    match action {
        Some(Action::Welcome) => app.show_welcome = true,
        Some(Action::CloseWelcome) => {
            app.settings.welcome_tab_closed = true;
            app.show_welcome = false;
            // With no drawing open the empty workspace stays, and the
            // Hints docker comes up at its Home topics.
            if !app.has_document() {
                app.open_docker(crate::app::DockerTab::Hints);
                app.hints.go(crate::ui::hints::HintPage::Home);
            }
        }
        Some(Action::Switch(i)) => app.switch_document(i),
        Some(Action::Close(i)) => {
            app.switch_document(i);
            app.close_document();
        }
        Some(Action::New) => app.request_new_document(),
        None => {}
    }
}

fn tab_fill(active: bool, hovered: bool) -> Color32 {
    if active {
        crate::theme::Tokens::tab_active()
    } else if hovered {
        crate::theme::Tokens::tool_hover()
    } else {
        crate::theme::Tokens::tab()
    }
}

/// The Welcome Screen tab: a house and the title, with a close button on
/// hover; `closed`, the house alone, which shows the Welcome Screen
/// again.
fn welcome_tab(ui: &mut Ui, active: bool, closed: bool) -> TabClick {
    let icon = 16.0;
    if closed {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(TAB_HEIGHT), Sense::hover());
        let resp = ui.interact(rect, egui::Id::new(WELCOME_TAB), Sense::click());
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, tab_fill(active, resp.hovered()));
        house(
            painter,
            rect.center() - Vec2::splat(icon / 2.0),
            crate::theme::Tokens::icon(),
        );
        return if resp
            .rich_tip(crate::ui::tips::keyed("welcome.title"))
            .clicked()
        {
            TabClick::Select
        } else {
            TabClick::None
        };
    }
    let label = tr("welcome.title");
    let galley = ui.painter().layout_no_wrap(
        label,
        egui::FontId::proportional(FONT),
        crate::theme::Tokens::text(),
    );
    let width = 6.0 + icon + 15.0 + galley.size().x + 27.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, TAB_HEIGHT), Sense::hover());
    let resp = ui.interact(rect, egui::Id::new(WELCOME_TAB), Sense::click());
    let close_rect = Rect::from_center_size(
        Pos2::new(rect.right() - 6.0 - CLOSE / 2.0, rect.center().y),
        Vec2::splat(CLOSE),
    );
    let close_resp = ui
        .interact(close_rect, egui::Id::new(WELCOME_CLOSE), Sense::click())
        .rich_tip(crate::ui::tips::keyed("tabs.close_welcome"));
    let close_hovered = close_resp.hovered();
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, tab_fill(active, resp.hovered()));
    house(
        painter,
        Pos2::new(rect.left() + 6.0, rect.top() + 5.0),
        crate::theme::Tokens::icon(),
    );
    painter.galley(
        Pos2::new(
            rect.left() + 6.0 + icon + 15.0,
            (rect.center().y - galley.size().y / 2.0).round(),
        ),
        galley,
        crate::theme::Tokens::text(),
    );
    if resp.hovered() || close_hovered {
        close_mark(painter, close_rect, close_hovered);
    }
    let resp = resp.rich_tip(crate::ui::tips::keyed("welcome.title"));
    if close_resp.clicked() || resp.middle_clicked() {
        TabClick::Close
    } else if resp.clicked() {
        TabClick::Select
    } else {
        TabClick::None
    }
}

/// The ids of the Welcome Screen tab and of its close button.
pub(crate) const WELCOME_TAB: &str = "welcome_tab";
pub(crate) const WELCOME_CLOSE: &str = "welcome_tab_close";

/// A tab's close button: an X, on a square while the pointer is over it.
fn close_mark(painter: &egui::Painter, close_rect: Rect, hovered: bool) {
    if hovered {
        painter.rect_filled(close_rect, 2.0, Tokens::border());
    }
    let s = Stroke::new(1.2, Tokens::text_dim());
    let r = close_rect.shrink(3.5);
    painter.line_segment([r.left_top(), r.right_bottom()], s);
    painter.line_segment([r.right_top(), r.left_bottom()], s);
}

/// A filled house, 16 pixels square, top-left at `o`: a roof with eaves
/// and a body with a door.
fn house(painter: &egui::Painter, o: Pos2, color: Color32) {
    let roof = vec![
        Pos2::new(o.x + 8.0, o.y),
        Pos2::new(o.x + 16.0, o.y + 8.5),
        Pos2::new(o.x, o.y + 8.5),
    ];
    painter.add(egui::Shape::convex_polygon(roof, color, Stroke::NONE));
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(o.x + 2.0, o.y + 8.0),
            Pos2::new(o.x + 14.0, o.y + 16.0),
        ),
        0.0,
        color,
    );
    painter.rect_filled(
        Rect::from_min_max(
            Pos2::new(o.x + 6.0, o.y + 12.0),
            Pos2::new(o.x + 10.0, o.y + 16.0),
        ),
        0.0,
        crate::theme::Tokens::tab(),
    );
}

/// The New tab: a grey plus on a tab-coloured square.
fn new_tab(ui: &mut Ui) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(TAB_HEIGHT), Sense::click());
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, tab_fill(false, resp.hovered()));
    let c = rect.center();
    let s = Stroke::new(2.0, crate::theme::Tokens::text_dim());
    painter.line_segment([c - Vec2::new(7.0, 0.0), c + Vec2::new(7.0, 0.0)], s);
    painter.line_segment([c - Vec2::new(0.0, 7.0), c + Vec2::new(0.0, 7.0)], s);
    resp.rich_tip(tr("menu.file.new")).clicked()
}

enum TabClick {
    None,
    Select,
    Close,
}

/// The id of the tab of the drawing `i`.
pub(crate) fn doc_tab_id(i: usize) -> egui::Id {
    egui::Id::new(("doc_tab", i))
}

fn doc_tab(ui: &mut Ui, i: usize, title: &str, active: bool) -> (TabClick, Rect) {
    let galley = ui.painter().layout_no_wrap(
        title.to_string(),
        egui::FontId::proportional(FONT),
        crate::theme::Tokens::text(),
    );
    let width = (PAD + galley.size().x + 6.0 + CLOSE + 8.0).max(116.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, TAB_HEIGHT), Sense::hover());
    let resp = ui.interact(rect, doc_tab_id(i), Sense::click());
    let close_rect = Rect::from_center_size(
        Pos2::new(rect.right() - 8.0 - CLOSE / 2.0, rect.center().y),
        Vec2::splat(CLOSE),
    );
    let close_id = ui.id().with(("doc_tab_close", i));
    let close_resp = ui
        .interact(close_rect, close_id, Sense::click())
        .rich_tip(crate::ui::tips::keyed("tabs.close_drawing"));
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, tab_fill(active, resp.hovered()));
    painter.galley(
        Pos2::new(
            rect.left() + PAD,
            (rect.center().y - galley.size().y / 2.0).round(),
        ),
        galley,
        crate::theme::Tokens::text(),
    );
    if resp.hovered() || close_resp.hovered() {
        close_mark(painter, close_rect, close_resp.hovered());
    }
    let resp = resp.rich_tip(format!("{title}\n{}", tr("tabs.drawing_desc")));
    let click = if close_resp.clicked() || resp.middle_clicked() {
        TabClick::Close
    } else if resp.clicked() {
        TabClick::Select
    } else {
        TabClick::None
    };
    (click, rect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, PointerButton};

    fn frame(ctx: &egui::Context, app: &mut App, events: Vec<Event>) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 800.0))),
            events,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| crate::ui::root(app, ui));
        out.textures_delta.clear();
    }

    fn click(ctx: &egui::Context, app: &mut App, at: Pos2) {
        frame(ctx, app, vec![Event::PointerMoved(at)]);
        for pressed in [true, false] {
            frame(
                ctx,
                app,
                vec![Event::PointerButton {
                    pos: at,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
        frame(ctx, app, Vec::new());
    }

    fn tab(ctx: &egui::Context, i: usize) -> Rect {
        ctx.read_response(doc_tab_id(i))
            .map(|r| r.rect)
            .expect("the tab is drawn")
    }

    /// Twelve drawings in a 700 point window: the tabs past the window's
    /// edge are reached with the strip's » button and the mouse wheel and
    /// can be chosen; choosing a drawing elsewhere (the Window menu)
    /// slides its tab into view (#1908).
    #[test]
    fn tabs_past_the_window_edge_can_be_reached() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        app.show_welcome = false;
        while app.docs.len() < 12 {
            app.new_document();
        }
        app.switch_document(0);
        for _ in 0..3 {
            frame(&ctx, &mut app, Vec::new());
        }
        // The room between the strip's « and » buttons.
        let strip = ctx
            .data(|d| d.get_temp::<Rect>(egui::Id::new("document_tabs_strip")))
            .expect("the strip is drawn");
        let b = crate::ui::overflow::BUTTON;
        let window = Rect::from_min_max(
            Pos2::new(strip.left() + b - 0.5, strip.top()),
            Pos2::new(strip.right() - b + 0.5, strip.top() + TAB_HEIGHT),
        );
        let last = app.docs.len() - 1;
        assert!(tab(&ctx, last).left() > window.right(), "past the edge");
        // » until the last tab is wholly in the window.
        let first = tab(&ctx, 0);
        let forward = Pos2::new(strip.right() - b / 2.0, first.center().y);
        for _ in 0..20 {
            if window.contains_rect(tab(&ctx, last)) {
                break;
            }
            click(&ctx, &mut app, forward);
        }
        let r = tab(&ctx, last);
        assert!(window.contains_rect(r), "{r:?} reached with »");
        click(&ctx, &mut app, r.center());
        assert_eq!(app.active_doc, last, "and chosen");
        // The wheel over the strip slides it back to the first tab.
        for _ in 0..40 {
            frame(
                &ctx,
                &mut app,
                vec![
                    Event::PointerMoved(Pos2::new(window.center().x, first.center().y)),
                    Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: Vec2::new(0.0, 200.0),
                        modifiers: egui::Modifiers::NONE,
                        phase: egui::TouchPhase::Move,
                    },
                ],
            );
        }
        let r = ctx
            .read_response(egui::Id::new(WELCOME_TAB))
            .map(|r| r.rect)
            .expect("the Welcome Screen tab");
        assert!(window.contains_rect(r), "{r:?} back with the wheel");
        assert!(tab(&ctx, 0).left() < window.right());
        assert!(!window.contains_rect(tab(&ctx, last)));
        // Another drawing chosen without the strip: its tab shows.
        app.switch_document(last - 1);
        for _ in 0..3 {
            frame(&ctx, &mut app, Vec::new());
        }
        let r = tab(&ctx, last - 1);
        assert!(window.contains_rect(r), "{r:?} the active tab shows");
    }
}
