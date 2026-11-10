//! The clip frame bar: with one clip frame selected (Pick tool), a bar
//! at the top left of the drawing window with Edit, Select Contents,
//! Extract Contents, the contents lock and Fit Contents (centre, fit,
//! fill, stretch); while the contents are edited, Finish. See
//! docs/behavior/clip-frame.md, The clip frame bar.

use crate::app::App;
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::ui::tips::RichTip;
use egui::{Color32, Context, RichText};
use tracedraw_core::ShapeKind;

const GREEN: Color32 = Color32::from_rgb(0x1E, 0x9E, 0x3C);

/// How the contents are fitted to the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitContents {
    /// The contents' centre on the frame's centre; no scaling.
    Center,
    /// Scaled alike in both directions to fit inside the frame, centred.
    FitProportionally,
    /// Scaled alike to cover the frame, centred.
    FillProportionally,
    /// Scaled in each direction to the frame's box.
    Stretch,
}

impl FitContents {
    pub const ALL: [FitContents; 4] = [
        FitContents::Center,
        FitContents::FitProportionally,
        FitContents::FillProportionally,
        FitContents::Stretch,
    ];

    pub fn key(self) -> &'static str {
        match self {
            FitContents::Center => "clip.fit_center",
            FitContents::FitProportionally => "clip.fit_proportional",
            FitContents::FillProportionally => "clip.fill_proportional",
            FitContents::Stretch => "clip.stretch",
        }
    }
}

/// The one clip frame selected, when the bar applies.
fn selected_clip(app: &App) -> Option<tracedraw_core::ShapeId> {
    let shapes = app.selected_shapes();
    match shapes.as_slice() {
        [s] if matches!(s.kind, ShapeKind::ClipFrame { .. }) => Some(s.id),
        _ => None,
    }
}

pub fn show(app: &mut App, ctx: &Context) {
    let editing = app.clip_frame_edit.is_some();
    let clip = selected_clip(app);
    if !editing && (clip.is_none() || app.tool != crate::tools::Tool::Pick) {
        return;
    }
    let at = app.canvas_rect.min + egui::vec2(14.0, 14.0);
    egui::Area::new(egui::Id::new("clip_frame_bar"))
        .fixed_pos(at)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Tokens::panel())
                .stroke(egui::Stroke::new(1.0, Tokens::border()))
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if editing {
                            finish_button(app, ui);
                            return;
                        }
                        let Some(clip) = clip else {
                            return;
                        };
                        let button = |ui: &mut egui::Ui, key: &str| {
                            ui.add(
                                egui::Button::new(RichText::new(tr(key)).color(Tokens::text()))
                                    .frame(false),
                            )
                            .rich_tip(tr(&format!("{key}_tip")))
                            .clicked()
                        };
                        if button(ui, "clip.edit") {
                            app.edit_clip_frame();
                            return;
                        }
                        ui.separator();
                        if button(ui, "clip.select_contents") {
                            app.select_clip_contents(clip);
                        }
                        if button(ui, "clip.extract") {
                            app.run(tracedraw_core::Command::ExtractContents { clip });
                            app.select(vec![clip]);
                        }
                        let locked = app.clip_frame_locked(clip);
                        if ui
                            .add(egui::Button::selectable(
                                locked,
                                RichText::new(tr("clip.lock")).color(Tokens::text()),
                            ))
                            .rich_tip(tr("clip.lock_tip"))
                            .clicked()
                        {
                            app.toggle_clip_frame_lock();
                        }
                        ui.separator();
                        let resp = ui.add(
                            egui::Button::new(
                                RichText::new(format!("{} \u{23F7}", tr("clip.fit_contents")))
                                    .color(Tokens::text()),
                            )
                            .frame(false),
                        );
                        egui::Popup::menu(&resp)
                            .id(egui::Id::new("clip_fit_menu"))
                            .style(crate::ui::menus::menu_popup_style)
                            .show(|ui| {
                                crate::ui::menus::body(ui, |ui| {
                                    for mode in FitContents::ALL {
                                        if crate::ui::menus::item(ui, mode.key(), "", true) {
                                            app.fit_clip_contents(clip, mode);
                                        }
                                    }
                                })
                            });
                    });
                });
        });
}

/// The Finish button with its green tick, as the symmetry bar has it.
fn finish_button(app: &mut App, ui: &mut egui::Ui) {
    let finish = ui
        .add(
            egui::Button::new(
                RichText::new(format!("      {}", tr("clip.finish"))).color(Tokens::text()),
            )
            .frame(false),
        )
        .rich_tip(tr("menu.object.clip_frame_finish"));
    let r = finish.rect;
    ui.painter().text(
        egui::pos2(r.left() + 2.0, r.center().y),
        egui::Align2::LEFT_CENTER,
        "\u{2714}",
        egui::FontId::proportional(16.0),
        GREEN,
    );
    if finish.clicked() {
        app.finish_clip_frame_edit();
    }
}
