//! The update card: when a newer release is known (found by the check at
//! start or by Help > Updates) a card opens in the middle of the window
//! with the name, an UPDATE badge, the two versions and, in one click,
//! Update now. While the file downloads the card shows the progress; once
//! it is checked its button becomes Restart now (Close and install for
//! the Windows setup and the `.deb`). What's new opens the release notes.
//! Dismiss (or the cross, or Escape) closes it for this session; the check
//! at the next start brings it back until the update is installed or the
//! version is skipped in the Updates box. See docs/behavior/updates.md.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::theme::Tokens;
use crate::ui::dialogs::Dialog;
use crate::updates::{InstallState, InstallTarget, Package, Release, UpdateState};
use egui::{Align2, Color32, Context, RichText};

/// What the card's main button does for a release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// Download the release's file and put it in place.
    Install(Package, InstallTarget),
    /// This copy cannot update itself: the Updates box offers Download.
    Details,
}

/// The release the card is about: the one found, unless it was skipped.
pub fn release(app: &App) -> Option<Release> {
    if crate::files::WEB || crate::updates::store_install() {
        return None;
    }
    match app.updates.get() {
        UpdateState::Available(r) if r.version != app.settings.skipped_update => Some(r),
        _ => None,
    }
}

/// Whether the card shows: a release is out, the card was not dismissed,
/// and no dialog is in the way (the Updates box says the same, and more;
/// the question of the first start comes first).
pub fn visible(app: &App) -> bool {
    !app.update_card_hidden && matches!(app.dialog, Dialog::None) && release(app).is_some()
}

/// What Update now does for `release` on a copy that updates itself with
/// `target`.
pub fn plan(release: &Release, target: Option<InstallTarget>) -> Plan {
    match (release.installer.clone(), target) {
        (Some(package), Some(target)) => Plan::Install(package, target),
        _ => Plan::Details,
    }
}

/// Update now: start the install; where the copy cannot install by
/// itself, show the Updates box, which offers Download.
pub fn update_now(app: &mut App, ctx: &Context, release: &Release, target: Option<InstallTarget>) {
    match plan(release, target) {
        Plan::Install(package, target) => {
            app.installer
                .start(release.version.clone(), package, target, Some(ctx.clone()));
        }
        Plan::Details => app.dialog = Dialog::Updates,
    }
}

/// Restart now, or Close and install: exit the usual way (asking about
/// unsaved drawings), the new version starts when the program has exited.
pub fn restart(app: &mut App) {
    if app.installer.arm() {
        app.request_exit();
    }
}

/// A dismissed card comes back once, when the download it was dismissed
/// during is checked and ready to start.
fn come_back_when_done(app: &mut App, install: &InstallState) {
    if let InstallState::Installed(v) | InstallState::Ready(v) = install {
        if app.update_card_done != *v {
            app.update_card_done = v.clone();
            app.update_card_hidden = false;
        }
    }
}

pub fn show(app: &mut App, ctx: &Context) {
    let install = app.installer.get();
    come_back_when_done(app, &install);
    if !visible(app) {
        return;
    }
    let Some(release) = release(app) else {
        return;
    };
    let version = env!("CARGO_PKG_VERSION");
    let working = matches!(install, InstallState::Downloading { .. });
    let (status, button): (String, Option<String>) = match &install {
        InstallState::Idle => (tr("updates.card_text"), Some(tr("updates.update_now"))),
        InstallState::Failed(e) => (
            trf("updates.install_failed", &[("error", e)]),
            Some(tr("updates.update_now")),
        ),
        InstallState::Downloading { done, total } => {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
            let mb = |b: u64| format!("{:.1}", b as f64 / 1_048_576.0);
            (
                trf(
                    "updates.downloading",
                    &[
                        ("new", &release.version),
                        ("done", &mb(*done)),
                        ("total", &mb(*total)),
                    ],
                ),
                None,
            )
        }
        InstallState::Installed(v) => (
            trf("updates.installed", &[("new", v)]),
            Some(tr("updates.restart")),
        ),
        InstallState::Ready(v) => (
            trf("updates.ready", &[("new", v)]),
            Some(tr("updates.restart_install")),
        ),
    };
    // A dim layer and a blocker under the card: like the dialogs, the card
    // takes every click meant for the window behind it.
    ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("update_card_dim"),
    ))
    .rect_filled(ctx.content_rect(), 0.0, Color32::from_black_alpha(110));
    crate::ui::chrome::modal_blocker(ctx);

    let icon = crate::ui::brand::icon(ctx);
    let mut pressed = false;
    let mut notes = false;
    let mut dismiss = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    egui::Area::new(egui::Id::new("update_card"))
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, -30.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Tokens::panel())
                .stroke(egui::Stroke::new(1.0, Tokens::border()))
                .corner_radius(6.0)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 6],
                    blur: 24,
                    spread: 0,
                    color: Color32::from_black_alpha(90),
                })
                .inner_margin(egui::Margin::same(20))
                .show(ui, |ui| {
                    ui.set_width(420.0);
                    ui.horizontal(|ui| {
                        let (r, _) =
                            ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::hover());
                        crate::ui::brand::paint_icon(ui.painter(), icon.as_ref(), r);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Craft Corel").size(20.0).strong());
                                ui.label(
                                    RichText::new(tr("updates.badge"))
                                        .size(11.0)
                                        .strong()
                                        .color(Color32::WHITE)
                                        .background_color(Tokens::ACCENT),
                                );
                            });
                            ui.label(
                                RichText::new(trf(
                                    "updates.versions",
                                    &[("version", version), ("new", &release.version)],
                                ))
                                .monospace()
                                .size(13.0)
                                .color(Tokens::text_dim()),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui
                                .button(RichText::new("\u{00D7}").size(16.0))
                                .on_hover_text(tr("updates.dismiss"))
                                .clicked()
                            {
                                dismiss = true;
                            }
                        });
                    });
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        if working {
                            ui.add(egui::Spinner::new().size(16.0));
                        }
                        ui.label(RichText::new(status).size(14.0).color(Tokens::text()));
                    });
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui
                            .link(RichText::new(tr("welcome.whats_new")).size(14.0))
                            .clicked()
                        {
                            notes = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(tr("updates.dismiss")).clicked() {
                                dismiss = true;
                            }
                            if let Some(label) = &button {
                                let primary = egui::Button::new(
                                    RichText::new(label).color(Color32::WHITE).strong(),
                                )
                                .fill(Tokens::ACCENT);
                                pressed = ui.add(primary).clicked();
                            }
                        });
                    });
                });
        });
    if notes {
        app.open_url(&release.page);
    }
    if dismiss {
        app.update_card_hidden = true;
    } else if pressed {
        match install {
            InstallState::Idle | InstallState::Failed(_) => {
                update_now(app, ctx, &release, crate::updates::install_target())
            }
            InstallState::Installed(_) | InstallState::Ready(_) => restart(app),
            InstallState::Downloading { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(with_package: bool) -> Release {
        Release {
            version: "9.0.0".into(),
            page: "https://example.org/r".into(),
            download: None,
            installer: with_package.then(|| Package {
                url: "https://example.org/a".into(),
                sha256: "0".repeat(64),
                size: 1,
            }),
        }
    }

    fn found(app: &mut App, r: Release) {
        *app.updates.state.lock().expect("lock") = UpdateState::Available(r);
    }

    fn frame(ctx: &Context, app: &mut App) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| show(app, &ui.ctx().clone()));
        out.textures_delta.clear();
    }

    /// The card shows for a newer release and stays out of the way of
    /// dialogs, a skipped version and a dismissed card.
    #[test]
    fn the_card_shows_only_for_a_release_to_update_to() {
        let mut app = App::headless();
        assert!(!visible(&app), "nothing found");
        for state in [
            UpdateState::Checking,
            UpdateState::Latest,
            UpdateState::Failed("x".into()),
        ] {
            *app.updates.state.lock().expect("lock") = state;
            assert!(!visible(&app));
        }
        found(&mut app, release(true));
        assert!(visible(&app));
        app.dialog = Dialog::Updates;
        assert!(!visible(&app), "the Updates box says it");
        app.dialog = Dialog::None;
        app.settings.skipped_update = "9.0.0".into();
        assert!(!visible(&app), "skipped");
        app.settings.skipped_update = "8.0.0".into();
        assert!(visible(&app));
        app.update_card_hidden = true;
        assert!(!visible(&app), "dismissed for this session");
    }

    /// Update now installs where the copy and the release allow it; else
    /// the Updates box offers Download.
    #[test]
    fn update_now_installs_when_it_can() {
        let target = InstallTarget::Tarball("/opt/tracedraw/tracedraw".into());
        let r = release(true);
        let package = r.installer.clone().expect("package");
        assert_eq!(
            plan(&r, Some(target.clone())),
            Plan::Install(package, target.clone())
        );
        assert_eq!(plan(&r, None), Plan::Details, "this copy cannot");
        assert_eq!(
            plan(&release(false), Some(target)),
            Plan::Details,
            "the release has no checked file"
        );
        // Where nothing can be installed the Updates box opens.
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        update_now(&mut app, &ctx, &r, None);
        assert!(matches!(app.dialog, Dialog::Updates));
    }

    /// This edition refuses an update restart even with prepared state.
    #[test]
    fn restart_needs_an_update_in_place() {
        let mut app = App::headless();
        restart(&mut app);
        assert!(!app.installer.is_armed() && !app.quit_now);
        app.installer.staged_for_test(crate::updates::Launch {
            program: "tracedraw".into(),
            args: Vec::new(),
        });
        restart(&mut app);
        assert!(
            !app.installer.is_armed(),
            "edition policy rejects prepared updates"
        );
        assert!(!app.quit_now, "the running application stays open");
    }

    /// A card dismissed during the download comes back once, when the
    /// file is ready, and a second dismissal then stays.
    #[test]
    fn a_dismissed_card_comes_back_when_the_download_is_ready() {
        let mut app = App::headless();
        app.update_card_hidden = true;
        come_back_when_done(&mut app, &InstallState::Downloading { done: 1, total: 2 });
        assert!(app.update_card_hidden, "still downloading");
        come_back_when_done(&mut app, &InstallState::Ready("9.0.0".into()));
        assert!(!app.update_card_hidden, "ready: back");
        app.update_card_hidden = true;
        come_back_when_done(&mut app, &InstallState::Ready("9.0.0".into()));
        assert!(app.update_card_hidden, "dismissed again: it stays");
    }

    /// The card draws in every state of an install, with and without the
    /// first-start question open, and Escape dismisses it.
    #[test]
    fn the_card_draws_in_every_state() {
        let ctx = crate::theme::ui_context();
        let mut app = App::headless();
        found(&mut app, release(true));
        let installs = [
            InstallState::Idle,
            InstallState::Downloading {
                done: 1 << 20,
                total: 3 << 20,
            },
            InstallState::Installed("9.0.0".into()),
            InstallState::Ready("9.0.0".into()),
            InstallState::Failed("digest".into()),
        ];
        for install in &installs {
            *app.installer.state.lock().expect("lock") = install.clone();
            app.update_card_hidden = false;
            app.update_card_done = "9.0.0".into();
            for _ in 0..2 {
                frame(&ctx, &mut app);
            }
            assert!(visible(&app), "{install:?}");
        }
        let mut escape = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            ..Default::default()
        };
        escape.events.push(egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        let mut out = ctx.run_ui(escape, |ui| show(&mut app, &ui.ctx().clone()));
        out.textures_delta.clear();
        assert!(app.update_card_hidden, "Escape dismissed it");
    }
}
