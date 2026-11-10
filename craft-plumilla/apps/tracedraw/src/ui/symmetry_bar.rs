//! The symmetry bar: while a symmetry is edited (Object > Symmetry > Edit
//! Symmetry), a bar at the top left of the drawing window with Finish,
//! the number of mirror lines, the centre, the angle of rotation, Show
//! full preview and Show symmetry lines. It stays while the symmetry is
//! edited, whatever is selected inside it.

use crate::app::App;
use crate::i18n::tr;
use crate::theme::Tokens;
use crate::ui::field::NumField;
use crate::ui::tips::RichTip;
use egui::{Color32, Context, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};

const GREEN: Color32 = Color32::from_rgb(0x1E, 0x9E, 0x3C);
const ROW: f32 = 20.0;

/// The mirror lines icon: three crossing dotted lines.
fn lines_icon(ui: &mut Ui) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
    let p = ui.painter();
    let c = r.center();
    for k in 0..3 {
        let a = (90.0 + 60.0 * k as f32).to_radians();
        let d = Vec2::new(a.cos(), -a.sin()) * 7.0;
        for i in -3..=3 {
            let q = c + d * (i as f32 / 3.0);
            p.circle_filled(q, 0.9, Tokens::text());
        }
    }
}

/// The angle of rotation icon: a turning arrow.
fn angle_icon(ui: &mut Ui) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        "\u{21BB}",
        egui::FontId::proportional(15.0),
        Tokens::text(),
    );
}

/// A two-state icon button; `draw` paints the icon in the rectangle.
fn toggle(
    ui: &mut Ui,
    on: &mut bool,
    tip: &str,
    draw: impl Fn(&egui::Painter, Rect, Color32),
) -> bool {
    let (r, resp) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
    let p = ui.painter();
    if *on {
        p.rect(
            r.shrink(1.0),
            2.0,
            Tokens::selected_row(),
            Stroke::new(1.0, Tokens::selected_row_line()),
            egui::StrokeKind::Inside,
        );
    } else if resp.hovered() {
        p.rect_filled(r.shrink(1.0), 2.0, Tokens::tool_hover());
    }
    draw(p, r.shrink(5.0), Tokens::text());
    let resp = resp.rich_tip(tip);
    if resp.clicked() {
        *on = !*on;
        return true;
    }
    false
}

pub fn show(app: &mut App, ctx: &Context) {
    // The symmetry being edited, whatever is selected inside it.
    let Some((id, center, angle, lines)) = app.edited_symmetry() else {
        return;
    };
    let at = app.canvas_rect.min + egui::vec2(14.0, 14.0);
    egui::Area::new(egui::Id::new("symmetry_bar"))
        .fixed_pos(at)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Tokens::panel())
                .stroke(Stroke::new(1.0, Tokens::border()))
                .inner_margin(egui::Margin::symmetric(10, 6))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let finish = ui
                            .add(
                                egui::Button::new(
                                    RichText::new(format!("      {}", tr("symmetry.finish")))
                                        .color(Tokens::text()),
                                )
                                .frame(false),
                            )
                            .rich_tip(tr("menu.object.symmetry_finish"));
                        // The tick in green, over the label's start.
                        let r = finish.rect;
                        ui.painter().text(
                            Pos2::new(r.left() + 2.0, r.center().y),
                            egui::Align2::LEFT_CENTER,
                            "\u{2714}",
                            egui::FontId::proportional(16.0),
                            GREEN,
                        );
                        if finish.clicked() {
                            app.finish_symmetry_edit();
                            return;
                        }
                        ui.separator();
                        let (mut n, mut a) = (lines as u32, angle);
                        let mut c = center;
                        let mut changed = false;
                        lines_icon(ui);
                        changed |= ui
                            .add_sized([52.0, ROW], NumField::new(&mut n).range(1..=12))
                            .rich_tip(tr("dialog.mirror_lines"))
                            .changed();
                        ui.separator();
                        let u = app.units;
                        let d = app.settings.precision.min(6) as usize;
                        let (mut x, mut y) = (u.world_from_mm(c.x), u.world_from_mm(c.y));
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            for (label, v) in [("X:", &mut x), ("Y:", &mut y)] {
                                let r = ui.add_sized(
                                    [120.0, ROW],
                                    NumField::new(v)
                                        .prefix(label)
                                        .max_decimals(d)
                                        .suffix(format!(" {}", u.short()))
                                        .unit_mm(u.mm()),
                                );
                                if r.changed() {
                                    changed = true;
                                }
                            }
                        });
                        if changed {
                            c = tracedraw_core::geometry::Point::new(
                                u.world_to_mm(x),
                                u.world_to_mm(y),
                            );
                        }
                        ui.separator();
                        angle_icon(ui);
                        changed |= ui
                            .add_sized(
                                [72.0, ROW],
                                NumField::new(&mut a)
                                    .range(0.0..=360.0)
                                    .max_decimals(1)
                                    .suffix("°"),
                            )
                            .rich_tip(tr("dialog.mirror_angle"))
                            .changed();
                        if changed {
                            app.set_symmetry_of(id, c, a, n.clamp(1, 12) as u8);
                        }
                        ui.separator();
                        let mut full = app.settings.symmetry_full_preview;
                        if toggle(ui, &mut full, &tr("symmetry.full_preview"), |p, r, col| {
                            // Three overlapping circles: the copies drawn.
                            let s = r.width() / 4.0;
                            for q in [
                                r.center() + Vec2::new(-s, s * 0.6),
                                r.center() + Vec2::new(s, s * 0.6),
                                r.center() + Vec2::new(0.0, -s * 0.8),
                            ] {
                                p.circle_stroke(q, s * 1.1, Stroke::new(1.2, col));
                            }
                        }) {
                            app.settings.symmetry_full_preview = full;
                            app.raster.borrow_mut().invalidate();
                        }
                        let mut shown = app.settings.symmetry_show_lines;
                        if toggle(ui, &mut shown, &tr("symmetry.show_lines"), |p, r, col| {
                            // Crossed dotted lines.
                            for (a, b) in [
                                (r.left_top(), r.right_bottom()),
                                (r.right_top(), r.left_bottom()),
                            ] {
                                for i in 0..=5 {
                                    let q = a + (b - a) * (i as f32 / 5.0);
                                    p.circle_filled(q, 1.0, col);
                                }
                            }
                        }) {
                            app.settings.symmetry_show_lines = shown;
                            if !shown {
                                app.symmetry_turning = false;
                            }
                        }
                    });
                });
        });
}
