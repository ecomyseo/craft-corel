//! Welcome Screen: Get Started (new, open, recent documents), Workspace,
//! What's New, Learn, Templates.

use crate::app::{App, WelcomeTab};
use crate::i18n::{tr, trf};
use crate::theme::Tokens;
use crate::ui::icons;
use crate::ui::tips::RichTip;
use egui::Ui;

/// One release in the What's New history: its version, its release date
/// and the language keys of its short title and its paragraph.
pub(crate) struct Release {
    pub version: &'static str,
    pub date: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

/// How many releases the What's New page lists.
pub(crate) const HISTORY_LEN: usize = 10;

/// The What's New history, newest first: the last [`HISTORY_LEN`]
/// releases. At each release add the new version on top (its title and
/// paragraph in every `lang/*.toml`, written from its `CHANGELOG.md`
/// section) and drop the last row; a test checks that the top row is the
/// version being built.
pub(crate) const RELEASES: &[Release] = &[
    Release {
        version: "0.42.0",
        date: "2026-11-11",
        title: "welcome.news.v0_42_title",
        body: "welcome.news.v0_42_body",
    },
    Release {
        version: "0.41.1",
        date: "2026-11-10",
        title: "welcome.news.v0_41_1_title",
        body: "welcome.news.v0_41_1_body",
    },
    Release {
        version: "0.41.0",
        date: "2026-11-06",
        title: "welcome.news.v0_41_title",
        body: "welcome.news.v0_41_body",
    },
    Release {
        version: "0.40.0",
        date: "2026-11-05",
        title: "welcome.news.v0_40_title",
        body: "welcome.news.v0_40_body",
    },
    Release {
        version: "0.39.0",
        date: "2026-11-04",
        title: "welcome.news.v0_39_title",
        body: "welcome.news.v0_39_body",
    },
    Release {
        version: "0.38.0",
        date: "2026-11-03",
        title: "welcome.news.v0_38_title",
        body: "welcome.news.v0_38_body",
    },
    Release {
        version: "0.37.0",
        date: "2026-11-02",
        title: "welcome.news.v0_37_title",
        body: "welcome.news.v0_37_body",
    },
    Release {
        version: "0.36.0",
        date: "2026-11-02",
        title: "welcome.news.v0_36_title",
        body: "welcome.news.v0_36_body",
    },
    Release {
        version: "0.35.0",
        date: "2026-11-01",
        title: "welcome.news.v0_35_title",
        body: "welcome.news.v0_35_body",
    },
    Release {
        version: "0.34.0",
        date: "2026-11-01",
        title: "welcome.news.v0_34_title",
        body: "welcome.news.v0_34_body",
    },
];

/// Current changes of this edition; historical base versions are not
/// published release tags of the independent project.
pub(crate) fn release_page(_version: &str) -> String {
    "https://github.com/ecomyseo/craft-plumilla/commits/main".into()
}

/// The heading of a release's row: version, date and title.
fn news_heading(r: &Release) -> String {
    format!("{}  \u{b7}  {}  \u{b7}  {}", r.version, r.date, tr(r.title))
}

/// The id of a release's row in the What's New history.
pub(crate) fn news_row_id(version: &str) -> egui::Id {
    egui::Id::new(("welcome_news", version))
}

pub fn welcome_screen(app: &mut App, ui: &mut Ui) {
    ui.add_space(8.0);
    // The tabs go on to a second line on a narrow screen (#1691).
    ui.horizontal_wrapped(|ui| {
        ui.add_space(24.0);
        ui.heading(egui::RichText::new("Craft Plumilla").size(26.0));
        ui.add_space(30.0);
        for (tab, key) in [
            (WelcomeTab::GetStarted, "welcome.get_started"),
            (WelcomeTab::Workspace, "welcome.workspace"),
            (WelcomeTab::News, "welcome.whats_new"),
            (WelcomeTab::Learn, "welcome.learn"),
            (WelcomeTab::Templates, "welcome.templates"),
        ] {
            if ui
                .selectable_label(
                    app.welcome_tab == tab,
                    egui::RichText::new(tr(key)).size(14.0),
                )
                .clicked()
            {
                app.welcome_tab = tab;
            }
        }
    });
    ui.separator();
    // Get Started keeps its buttons in place: only its list of recent
    // documents scrolls, in the window's remaining height (#1383).
    if app.welcome_tab == WelcomeTab::GetStarted {
        ui.add_space(12.0);
        get_started(app, ui);
        return;
    }
    // The page of the other tabs scrolls under the tab row (a small
    // window), with the wheel and a scroll bar (#1288); each tab keeps its
    // own place.
    egui::ScrollArea::vertical()
        .id_salt(("welcome_page", format!("{:?}", app.welcome_tab)))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(12.0);
            match app.welcome_tab {
                WelcomeTab::GetStarted => {}
                WelcomeTab::Workspace => workspace(app, ui),
                WelcomeTab::News => {
                    if let Some(url) = news(ui) {
                        app.open_url(&url);
                    }
                }
                WelcomeTab::Learn => learn(ui),
                WelcomeTab::Templates => templates(app, ui),
            }
            ui.add_space(12.0);
        });
}

fn big_button(ui: &mut Ui, icon: icons::Action, title: String, subtitle: String) -> bool {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(240.0, 64.0), egui::Sense::click());
    let bg = if resp.hovered() {
        Tokens::tool_hover()
    } else {
        Tokens::panel()
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, bg);
    painter.rect_stroke(
        rect,
        4.0,
        egui::Stroke::new(1.0, Tokens::border()),
        egui::epaint::StrokeKind::Inside,
    );
    let icon_rect = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 32.0, rect.center().y),
        egui::vec2(28.0, 28.0),
    );
    icons::draw_action(painter, icon_rect, icon, Tokens::ACCENT);
    let text_x = rect.left() + 64.0;
    painter.text(
        egui::pos2(
            text_x,
            rect.center().y - if subtitle.is_empty() { 0.0 } else { 9.0 },
        ),
        egui::Align2::LEFT_CENTER,
        title,
        egui::FontId::proportional(14.0),
        Tokens::text(),
    );
    if !subtitle.is_empty() {
        painter.text(
            egui::pos2(text_x, rect.center().y + 10.0),
            egui::Align2::LEFT_CENTER,
            subtitle,
            egui::FontId::proportional(11.0),
            Tokens::text_dim(),
        );
    }
    resp.clicked()
}

fn get_started(app: &mut App, ui: &mut Ui) {
    // On a phone the recent documents go under the buttons and the page
    // scrolls: side by side they need some 700 points (#1301, #1691).
    if crate::ui::phone::phone() {
        egui::ScrollArea::vertical()
            .id_salt("welcome_get_started")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.set_max_width(ui.available_width() - 12.0);
                        start_buttons(app, ui);
                        // A browser keeps no recent documents.
                        if !crate::files::WEB {
                            ui.add_space(20.0);
                            recent_documents(app, ui, true);
                        }
                    });
                });
            });
        return;
    }
    ui.horizontal_top(|ui| {
        ui.add_space(24.0);
        ui.vertical(|ui| {
            ui.set_width(260.0);
            start_buttons(app, ui);
        });
        // A browser keeps no recent documents.
        if crate::files::WEB {
            return;
        }
        ui.add_space(40.0);
        ui.vertical(|ui| recent_documents(app, ui, false));
    });
}

/// New Document, New From Template, Open and the shortcuts hint.
fn start_buttons(app: &mut App, ui: &mut Ui) {
    if big_button(
        ui,
        icons::Action::New,
        tr("welcome.new_document"),
        crate::shortcuts::shown("Ctrl+N".into()),
    ) {
        app.request_new_document();
    }
    ui.add_space(8.0);
    if big_button(
        ui,
        icons::Action::Welcome,
        tr("welcome.new_from_template"),
        String::new(),
    ) {
        app.open_template_dialog();
    }
    ui.add_space(8.0);
    if big_button(
        ui,
        icons::Action::Open,
        tr("welcome.open_document"),
        "Ctrl+O".into(),
    ) {
        app.open_dialog();
        if app.file.is_some() || app.doc().all_layers().any(|l| !l.shapes.is_empty()) {
            app.show_welcome = false;
        }
    }
    ui.add_space(20.0);
    ui.label(
        egui::RichText::new(tr("welcome.shortcuts_hint"))
            .color(Tokens::text_dim())
            .size(11.0),
    );
}

/// The recent documents heading, list and Clear button; the list scrolls
/// on its own except on a `phone`, whose whole page scrolls.
fn recent_documents(app: &mut App, ui: &mut Ui, phone: bool) {
    ui.strong(tr("welcome.recent_documents"));
    let recent = app.settings.recent_files.clone();
    if recent.is_empty() {
        ui.label(egui::RichText::new(tr("welcome.no_recent")).color(Tokens::text_dim()));
    }
    ui.add_space(4.0);
    // The list has a scroll area of its own, as tall as the window
    // leaves under the heading, less the Clear button and a margin
    // (#1383); the heading, the button and the left column stay put. On
    // a phone the whole page scrolls instead.
    let clear = if recent.is_empty() {
        0.0
    } else {
        ui.spacing().interact_size.y + ui.spacing().item_spacing.y
    };
    let mut actions = Vec::new();
    if phone {
        for p in &recent {
            actions.push((recent_row(app, ui, p), p));
        }
    } else {
        let height = (ui.available_height() - clear - 12.0).max(RECENT_ROW.y);
        egui::ScrollArea::vertical()
            .id_salt("welcome_recent_list")
            .max_height(height)
            .auto_shrink([true, true])
            .show(ui, |ui| {
                for p in &recent {
                    actions.push((recent_row(app, ui, p), p));
                }
            });
    }
    for (action, p) in actions {
        match action {
            RecentAction::None => {}
            RecentAction::Open => {
                app.open_path(p.clone());
                app.show_welcome = false;
            }
            RecentAction::Remove => app.remove_recent(p),
        }
    }
    if !app.settings.recent_files.is_empty()
        && ui.small_button(tr("menu.file.clear_recent")).clicked()
    {
        app.settings.recent_files.clear();
        app.settings.save();
        crate::thumbnails::clear();
        app.recent_thumbs.clear();
    }
}

/// What a click on a recent document's row asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecentAction {
    None,
    Open,
    /// Take it off the list (the x on the row, or the row's right-click
    /// menu), #1152.
    Remove,
}

/// The id of the recent documents row of `path`.
pub(crate) fn recent_row_id(path: &std::path::Path) -> egui::Id {
    egui::Id::new(("welcome_recent_row", path))
}

/// The id of the x that takes `path` off the recent documents list.
pub(crate) fn remove_recent_id(path: &std::path::Path) -> egui::Id {
    egui::Id::new(("welcome_recent_remove", path))
}

/// One recent drawing: its thumbnail (a blank page when it has none), its
/// name and folder; a missing file is greyed out. While the pointer is on
/// the row an x at its right takes it off the list, as does Remove from
/// List in its right-click menu (#1152).
/// The size of a recent document's row.
const RECENT_ROW: egui::Vec2 = egui::vec2(380.0, 56.0);

fn recent_row(app: &mut App, ui: &mut Ui, path: &std::path::Path) -> RecentAction {
    const ROW: egui::Vec2 = RECENT_ROW;
    const THUMB: egui::Vec2 = egui::vec2(64.0, 48.0);
    const X: f32 = 20.0;
    // Asked of the disk on a worker thread: a drawing on a share that does
    // not answer never holds the window (#1388).
    let exists = crate::file_presence::exists(path, Some(ui.ctx()));
    let thumb = if exists {
        app.recent_thumbnail(ui.ctx(), path)
    } else {
        None
    };
    // No wider than the room (a phone's page).
    let row = egui::vec2(ROW.x.min(ui.available_width().max(THUMB.x + 40.0)), ROW.y);
    let (rect, _) = ui.allocate_exact_size(row, egui::Sense::hover());
    let resp = ui.interact(rect, recent_row_id(path), egui::Sense::click());
    let on_row = ui.rect_contains_pointer(rect);
    let painter = ui.painter();
    if exists && on_row {
        painter.rect_filled(rect, 2.0, Tokens::tool_hover());
    }
    let frame = egui::Rect::from_min_size(rect.min + egui::vec2(4.0, 4.0), THUMB);
    painter.rect_filled(frame, 0.0, egui::Color32::WHITE);
    if let Some(t) = &thumb {
        let size = t.size_vec2();
        let scale = (THUMB.x / size.x.max(1.0)).min(THUMB.y / size.y.max(1.0));
        let r = egui::Rect::from_center_size(frame.center(), size * scale);
        painter.image(
            t.id(),
            r,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
    painter.rect_stroke(
        frame,
        0.0,
        egui::Stroke::new(1.0, Tokens::border()),
        egui::StrokeKind::Inside,
    );
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    // A document the Android picker returned has a URI, not a folder.
    let folder = path
        .parent()
        .filter(|_| !crate::files::is_document_path(path))
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    let text_x = frame.right() + 10.0;
    let color = if exists {
        Tokens::text()
    } else {
        Tokens::text_dim()
    };
    // The texts stop short of the x.
    let clip =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.right() - X - 8.0, rect.bottom()));
    let texts = painter.with_clip_rect(clip);
    texts.text(
        egui::pos2(text_x, frame.top() + 6.0),
        egui::Align2::LEFT_TOP,
        name,
        egui::FontId::proportional(13.0),
        color,
    );
    texts.text(
        egui::pos2(text_x, frame.top() + 26.0),
        egui::Align2::LEFT_TOP,
        folder,
        egui::FontId::proportional(11.0),
        Tokens::text_dim(),
    );
    let mut action = RecentAction::None;
    // The x shows only while the pointer is on the row; added after the
    // row, it takes the click before the row does.
    if on_row {
        let xr = egui::Rect::from_center_size(
            egui::pos2(rect.right() - X / 2.0 - 4.0, rect.center().y),
            egui::vec2(X, X),
        );
        let x = ui
            .interact(xr, remove_recent_id(path), egui::Sense::click())
            .rich_tip(crate::ui::tips::keyed("welcome.remove_recent"));
        let p = ui.painter();
        if x.hovered() {
            p.rect_filled(xr, 3.0, Tokens::border());
        }
        let c = xr.center();
        let s = egui::Stroke::new(
            1.5,
            if x.hovered() {
                Tokens::text()
            } else {
                Tokens::text_dim()
            },
        );
        p.line_segment([c + egui::vec2(-4.0, -4.0), c + egui::vec2(4.0, 4.0)], s);
        p.line_segment([c + egui::vec2(-4.0, 4.0), c + egui::vec2(4.0, -4.0)], s);
        if x.clicked() {
            action = RecentAction::Remove;
        }
    }
    resp.context_menu(|ui| {
        if ui
            .button(tr("welcome.remove_recent"))
            .rich_tip(crate::ui::tips::keyed("welcome.remove_recent"))
            .clicked()
        {
            action = RecentAction::Remove;
            ui.close();
        }
    });
    let resp = resp.rich_tip(path.display().to_string());
    if action == RecentAction::None && exists && resp.clicked() {
        action = RecentAction::Open;
    }
    action
}

fn workspace(app: &mut App, ui: &mut Ui) {
    ui.horizontal_top(|ui| {
        ui.add_space(24.0);
        ui.vertical(|ui| {
            ui.strong(tr("welcome.choose_workspace"));
            for (ws, key, desc) in [
                (
                    crate::app::Workspace::Default,
                    "menu.window.workspace_default",
                    "welcome.ws_default_desc",
                ),
                (
                    crate::app::Workspace::Lite,
                    "menu.window.workspace_lite",
                    "welcome.ws_lite_desc",
                ),
                (
                    crate::app::Workspace::Classic,
                    "menu.window.workspace_classic",
                    "welcome.ws_classic_desc",
                ),
                (
                    crate::app::Workspace::Illustration,
                    "menu.window.workspace_illustration",
                    "welcome.ws_illustration_desc",
                ),
                (
                    crate::app::Workspace::PageLayout,
                    "menu.window.workspace_page_layout",
                    "welcome.ws_page_layout_desc",
                ),
                (
                    crate::app::Workspace::Touch,
                    "menu.window.workspace_touch",
                    "welcome.ws_touch_desc",
                ),
            ] {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(
                            app.workspace == ws,
                            egui::RichText::new(tr(key)).size(13.0),
                        )
                        .clicked()
                    {
                        app.set_workspace(ws);
                        app.save_settings();
                    }
                    ui.label(
                        egui::RichText::new(tr(desc))
                            .color(Tokens::text_dim())
                            .size(11.0),
                    );
                });
            }
            ui.add_space(12.0);
            ui.strong(tr("options.language"));
            let current = crate::i18n::language();
            ui.horizontal_wrapped(|ui| {
                for (code, name) in crate::i18n::LANGUAGES {
                    if ui.selectable_label(current == code, name).clicked() {
                        crate::i18n::set_language(code);
                        app.settings.language = code.to_string();
                        app.settings.save();
                    }
                }
            });
        });
    });
}

/// The What's New history: a row per release, newest first and open, the
/// others closed; a row opens to its paragraph and a link to its release
/// page. Gives the address of a link clicked.
fn news(ui: &mut Ui) -> Option<String> {
    let mut open = None;
    ui.horizontal_top(|ui| {
        ui.add_space(24.0);
        ui.vertical(|ui| {
            ui.set_max_width(640.0);
            ui.label(tr("welcome.news.source_history"));
            ui.add_space(8.0);
            for (i, r) in RELEASES.iter().take(HISTORY_LEN).enumerate() {
                ui.add_space(4.0);
                let heading = egui::RichText::new(news_heading(r)).strong().size(14.0);
                egui::CollapsingHeader::new(heading)
                    .id_salt(news_row_id(r.version))
                    .default_open(i == 0)
                    .show(ui, |ui| {
                        ui.label(tr(r.body));
                        ui.add_space(2.0);
                        let url = release_page(r.version);
                        if ui
                            .link(tr("welcome.news.more"))
                            .on_hover_text(&url)
                            .clicked()
                        {
                            open = Some(url);
                        }
                    });
            }
        });
    });
    open
}

fn learn(ui: &mut Ui) {
    ui.horizontal_top(|ui| {
        ui.add_space(24.0);
        ui.vertical(|ui| {
            ui.set_max_width(640.0);
            for (t, b) in [
                ("welcome.learn_1_title", "welcome.learn_1_body"),
                ("welcome.learn_2_title", "welcome.learn_2_body"),
                ("welcome.learn_3_title", "welcome.learn_3_body"),
                ("welcome.learn_4_title", "welcome.learn_4_body"),
                ("welcome.learn_5_title", "welcome.learn_5_body"),
            ] {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(tr(t)).strong().size(14.0));
                ui.label(tr(b));
            }
        });
    });
}

fn templates(app: &mut App, ui: &mut Ui) {
    ui.horizontal_top(|ui| {
        ui.add_space(24.0);
        ui.vertical(|ui| {
            ui.strong(tr("welcome.templates"));
            let list = crate::templates::built_in();
            if let Some(i) = crate::ui::template_dialog::strip(ui, &list) {
                if let Some(t) = list.get(i) {
                    let name = app.next_untitled_name();
                    if let Some(doc) = crate::templates::document(t, &name) {
                        app.open_document(doc, None);
                    }
                }
            }
            ui.add_space(6.0);
            if ui.button(tr("tpl.more")).clicked() {
                app.open_template_dialog();
            }
            ui.add_space(12.0);
            ui.strong(tr("welcome.my_templates"));
            if ui.button(tr("welcome.open_template")).clicked() {
                crate::files::Dialog::new()
                    .add_filter("Craft Plumilla template", &["tdt", "tdraw"])
                    .pick_file(app, |app, p| app.open_as_template(p));
            }
            let dir = crate::settings::config_dir().map(|d| d.join("templates"));
            if let Some(dir) = dir {
                if let Ok(rd) = std::fs::read_dir(&dir) {
                    for e in rd.flatten() {
                        let p = e.path();
                        if p.extension().and_then(|x| x.to_str()) == Some("tdt") {
                            let name = p
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            if ui.button(name).clicked() {
                                app.open_as_template(p.clone());
                            }
                        }
                    }
                }
                ui.label(
                    egui::RichText::new(trf(
                        "welcome.templates_folder",
                        &[("d", &dir.display().to_string())],
                    ))
                    .color(Tokens::text_dim())
                    .size(11.0),
                );
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One frame of the Welcome Screen in a window 1200 x 600; the top of
    /// the New Document button's title as drawn.
    fn run(ctx: &egui::Context, app: &mut App, events: Vec<egui::Event>) -> Option<f32> {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 600.0),
            )),
            events,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| welcome_screen(app, ui));
        });
        out.textures_delta.clear();
        let title = tr("welcome.new_document");
        out.shapes.iter().find_map(|c| match &c.shape {
            egui::Shape::Text(t) if t.galley.text() == title => Some(t.pos.y),
            _ => None,
        })
    }

    /// #1288: recent documents below the window are reached with the
    /// wheel.
    #[test]
    fn the_recent_documents_scroll_into_view() {
        if crate::files::WEB {
            return;
        }
        let mut app = App::headless();
        app.show_welcome = true;
        app.welcome_tab = WelcomeTab::GetStarted;
        let paths: Vec<std::path::PathBuf> = (0..30)
            .map(|i| std::path::PathBuf::from(format!("/no/such/folder/drawing-{i}.tdraw")))
            .collect();
        app.settings.recent_files = paths.clone();
        let ctx = crate::theme::ui_context();
        run(&ctx, &mut app, Vec::new());
        run(&ctx, &mut app, Vec::new());
        let last = paths.last().expect("a path");
        let bottom = |ctx: &egui::Context| {
            ctx.read_response(recent_row_id(last))
                .expect("the last row is laid out")
                .rect
                .bottom()
        };
        let before = bottom(&ctx);
        assert!(before > 600.0, "{before}");
        run(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(egui::pos2(600.0, 300.0))],
        );
        for _ in 0..40 {
            run(
                &ctx,
                &mut app,
                vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::vec2(0.0, -10.0),
                    phase: egui::TouchPhase::Move,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
        for _ in 0..30 {
            run(&ctx, &mut app, Vec::new());
        }
        let after = bottom(&ctx);
        assert!(after <= 600.0, "the last row ends at {after}");
    }

    /// #1383: only the list of recent documents scrolls; the New and Open
    /// buttons and the list's heading stay where they are, and the wheel
    /// over the buttons scrolls nothing.
    #[test]
    fn only_the_recent_documents_scroll() {
        if crate::files::WEB {
            return;
        }
        let mut app = App::headless();
        app.show_welcome = true;
        app.welcome_tab = WelcomeTab::GetStarted;
        let paths: Vec<std::path::PathBuf> = (0..30)
            .map(|i| std::path::PathBuf::from(format!("/no/such/folder/drawing-{i}.tdraw")))
            .collect();
        app.settings.recent_files = paths.clone();
        let ctx = crate::theme::ui_context();
        run(&ctx, &mut app, Vec::new());
        let new_at = run(&ctx, &mut app, Vec::new()).expect("the New Document button");
        let row = |ctx: &egui::Context, p: &std::path::Path| {
            ctx.read_response(recent_row_id(p))
                .expect("the row is laid out")
                .rect
        };
        let first = row(&ctx, &paths[0]);
        let wheel = |ctx: &egui::Context, app: &mut App, at: egui::Pos2| {
            run(ctx, app, vec![egui::Event::PointerMoved(at)]);
            let mut new_at = None;
            for _ in 0..40 {
                new_at = run(
                    ctx,
                    app,
                    vec![egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Line,
                        delta: egui::vec2(0.0, -10.0),
                        phase: egui::TouchPhase::Move,
                        modifiers: egui::Modifiers::NONE,
                    }],
                );
            }
            for _ in 0..30 {
                new_at = run(ctx, app, Vec::new());
            }
            new_at
        };
        // The wheel over the buttons moves neither them nor the list.
        let at = wheel(&ctx, &mut app, egui::pos2(120.0, 150.0));
        assert_eq!(at, Some(new_at));
        assert_eq!(row(&ctx, &paths[0]), first);
        // The wheel over the list scrolls it to its last row, inside the
        // window, and the buttons stay.
        let at = wheel(&ctx, &mut app, first.center());
        assert_eq!(at, Some(new_at));
        let last = row(&ctx, paths.last().expect("a path"));
        assert!(
            last.bottom() <= 600.0,
            "the last row ends at {}",
            last.bottom()
        );
        assert!(row(&ctx, &paths[0]).top() < first.top());
        // The list starts under its heading and never above it: its clip
        // keeps the rows off the heading.
        assert!(first.top() > new_at - 20.0);
    }

    /// The version numbers of `v` (`0.39.0`, `0.39.0-rc.1`) as numbers.
    fn numbers(v: &str) -> Vec<u32> {
        let core = v.split(['-', '+']).next().unwrap_or(v);
        core.split('.')
            .map(|n| n.parse().expect("a number"))
            .collect()
    }

    /// A release PR cannot forget What's New: its newest row is the
    /// version being built (major.minor.patch).
    #[test]
    fn the_newest_release_is_this_version() {
        let newest = RELEASES.first().expect("a release");
        assert_eq!(
            numbers(newest.version),
            numbers(env!("CARGO_PKG_VERSION")),
            "add the What's New paragraph of {} on top of RELEASES (ui/welcome.rs) \
             and drop the oldest; see MAINTAINING.md",
            env!("CARGO_PKG_VERSION")
        );
    }

    /// At most ten releases, newest first, each with its own keys, all of
    /// them in the English file (`every_language_covers_english_keys`
    /// checks the other languages).
    #[test]
    fn the_history_holds_ten_releases_with_their_texts() {
        assert!(!RELEASES.is_empty());
        assert!(
            RELEASES.len() <= HISTORY_LEN,
            "{} releases; drop the oldest",
            RELEASES.len()
        );
        for pair in RELEASES.windows(2) {
            assert!(
                numbers(pair[0].version) > numbers(pair[1].version),
                "{} before {}",
                pair[0].version,
                pair[1].version
            );
            assert!(pair[0].date >= pair[1].date, "{}", pair[0].version);
        }
        let en = crate::i18n::parse(include_str!("../../lang/en.toml"));
        for r in RELEASES {
            let n = numbers(r.version);
            assert_eq!(n.len(), 3, "{}", r.version);
            assert_eq!(r.date.len(), 10, "{}", r.date);
            let stem = format!("welcome.news.v{}_{}", n[0], n[1]);
            assert!(r.title.starts_with(&stem), "{} for {}", r.title, r.version);
            assert!(r.body.starts_with(&stem), "{} for {}", r.body, r.version);
            for key in [r.title, r.body] {
                let text = en
                    .get(key)
                    .unwrap_or_else(|| panic!("{key} not in en.toml"));
                assert!(!text.is_empty(), "{key}");
                assert!(!text.contains('\u{2014}'), "{key} has an em dash");
            }
        }
        assert!(en.contains_key("welcome.news.more"));
    }

    /// Inherited history links to actual changes of this edition,
    /// without inventing a release tag for each historical version.
    #[test]
    fn release_history_links_to_current_project_changes() {
        for r in RELEASES {
            let url = release_page(r.version);
            assert_eq!(url, "https://github.com/ecomyseo/craft-plumilla/commits/main");
            assert!(!url.contains("/releases/tag/"), "{}", r.version);
        }
        assert_eq!(
            release_page("0.39.0"),
            "https://github.com/ecomyseo/craft-plumilla/commits/main"
        );
    }

    /// One frame of the What's New page alone in a window 1200 x 2000:
    /// the link clicked, if any, and every text drawn with its rectangle.
    fn news_frame(
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> (Option<String>, Vec<(String, egui::Rect)>) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 2000.0),
            )),
            events,
            ..Default::default()
        };
        let mut url = None;
        let mut out = ctx.run_ui(input, |ui| {
            egui::CentralPanel::default().show(ui, |ui| url = news(ui));
        });
        out.textures_delta.clear();
        let texts = out
            .shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) => Some((
                    t.galley.text().to_string(),
                    t.galley.rect.translate(t.pos.to_vec2()),
                )),
                _ => None,
            })
            .collect();
        (url, texts)
    }

    /// Moves to `at`, presses and lets go there; the link clicked, if any.
    fn news_click(ctx: &egui::Context, at: egui::Pos2) -> Option<String> {
        let button = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        let mut url = news_frame(ctx, vec![egui::Event::PointerMoved(at)]).0;
        url = url.or(news_frame(ctx, vec![button(true)]).0);
        url = url.or(news_frame(ctx, vec![button(false)]).0);
        // Let the row's opening animation end.
        for _ in 0..30 {
            url = url.or(news_frame(ctx, Vec::new()).0);
        }
        url
    }

    fn find(texts: &[(String, egui::Rect)], text: &str) -> Option<egui::Rect> {
        texts.iter().find(|(t, _)| t == text).map(|(_, r)| *r)
    }

    /// Help > What's New shows the history: every release's row, the
    /// newest open with its paragraph and the others closed.
    #[test]
    fn help_whats_new_opens_the_history() {
        let mut app = App::headless();
        app.show_welcome = true;
        app.welcome_tab = WelcomeTab::GetStarted;
        let ctx = crate::theme::ui_context();
        crate::ui::menus::open_whats_new(&mut app);
        assert_eq!(app.welcome_tab, WelcomeTab::News);
        let input = || egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 2000.0),
            )),
            ..Default::default()
        };
        let mut texts = Vec::new();
        for _ in 0..3 {
            let mut out = ctx.run_ui(input(), |ui| {
                egui::CentralPanel::default().show(ui, |ui| welcome_screen(&mut app, ui));
            });
            out.textures_delta.clear();
            texts = out
                .shapes
                .iter()
                .filter_map(|c| match &c.shape {
                    egui::Shape::Text(t) => Some(t.galley.text().to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>();
        }
        for r in RELEASES {
            assert!(texts.contains(&news_heading(r)), "{}", r.version);
        }
        assert!(texts.contains(&tr(RELEASES[0].body)));
        for r in &RELEASES[1..] {
            assert!(!texts.contains(&tr(r.body)), "{} is open", r.version);
        }
    }

    /// A click on a closed row opens it to its paragraph and its link,
    /// which opens that release's page; the newest row stays open.
    #[test]
    fn a_click_opens_a_closed_release() {
        if crate::files::WEB {
            return;
        }
        let ctx = crate::theme::ui_context();
        news_frame(&ctx, Vec::new());
        let (_, texts) = news_frame(&ctx, Vec::new());
        let r = &RELEASES[2];
        assert!(find(&texts, &tr(r.body)).is_none(), "{} is open", r.version);
        assert!(find(&texts, &tr(RELEASES[0].body)).is_some());
        let heading = find(&texts, &news_heading(r)).expect("the row's heading");
        assert_eq!(news_click(&ctx, heading.center()), None);
        let (_, texts) = news_frame(&ctx, Vec::new());
        let body = find(&texts, &tr(r.body)).expect("the row opened");
        assert!(body.top() > heading.bottom());
        assert!(find(&texts, &tr(RELEASES[0].body)).is_some());
        assert!(find(&texts, &tr(RELEASES[1].body)).is_none());
        // Its link is the one under its paragraph, and opens its page.
        let link = texts
            .iter()
            .filter(|(t, rect)| *t == tr("welcome.news.more") && rect.top() > body.bottom())
            .map(|(_, rect)| *rect)
            .min_by(|a, b| a.top().total_cmp(&b.top()))
            .expect("the row's link");
        assert_eq!(
            news_click(&ctx, link.center()),
            Some(release_page(r.version))
        );
    }
}
