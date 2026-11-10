//! The settings of TraceCut's Nesting and Measure tools in the TraceCut
//! window. See docs/behavior/tracecut.md.

use crate::app::Units;
use crate::i18n::tr;
use crate::tracecut::{ToolId, TraceCutState};
use crate::ui::tips::RichTip;
use crate::ui::tracecut::{length, row};
use egui::Ui;
use tracedraw_tracecut::nest::shapes::Rotations;
use tracedraw_tracecut::nest::{Corner, RollDir, SheetSpec};

/// A number field with its tip.
fn number(ui: &mut Ui, v: &mut f64, range: (f64, f64), decimals: usize, tip: &str) {
    let mut x = *v;
    let r = ui
        .add_sized(
            [110.0, 22.0],
            crate::ui::field::NumField::new(&mut x)
                .range(range.0..=range.1)
                .max_decimals(decimals),
        )
        .rich_tip(crate::ui::tips::keyed(tip));
    if r.changed() && x.is_finite() {
        *v = x.clamp(range.0, range.1);
    }
}

/// A whole-number field with its tip.
fn whole(ui: &mut Ui, v: &mut u32, range: (u32, u32), tip: &str) {
    let mut x = f64::from(*v);
    number(ui, &mut x, (f64::from(range.0), f64::from(range.1)), 0, tip);
    *v = x.round().clamp(f64::from(range.0), f64::from(range.1)) as u32;
}

/// A check box with its tip.
fn check(ui: &mut Ui, v: &mut bool, key: &str) {
    ui.checkbox(v, tr(key))
        .rich_tip(crate::ui::tips::keyed(key));
}

/// A list of `options`, each named by its i18n key.
fn choice<T: PartialEq + Copy>(ui: &mut Ui, id: &str, v: &mut T, options: &[(T, &str)], tip: &str) {
    let shown = options
        .iter()
        .find(|(o, _)| o == v)
        .map(|(_, k)| tr(k))
        .unwrap_or_default();
    crate::ui::keynav::ComboBox::from_id_salt(("tracecut", id))
        .selected_text(shown)
        .width(200.0)
        .show_ui(ui, |ui| {
            for (o, k) in options {
                ui.selectable_value(v, *o, tr(k));
            }
        })
        .response
        .rich_tip(crate::ui::tips::keyed(tip));
}

/// The material: sheets or a roll, its size, margin, spacing and corner.
fn sheet(ui: &mut Ui, units: Units, s: &mut SheetSpec) {
    row(ui, &tr("tracecut.material"), |ui| {
        ui.radio_value(&mut s.roll, false, tr("tracecut.sheets"))
            .rich_tip(crate::ui::tips::keyed("tracecut.sheets"));
        ui.radio_value(&mut s.roll, true, tr("tracecut.roll"))
            .rich_tip(crate::ui::tips::keyed("tracecut.roll"));
    });
    if s.roll {
        row(ui, &tr("tracecut.roll_width"), |ui| {
            length(
                ui,
                units,
                &mut s.width,
                (1.0, 100_000.0),
                "tracecut.roll_width",
            )
        });
        row(ui, &tr("tracecut.roll_dir"), |ui| {
            choice(
                ui,
                "roll_dir",
                &mut s.roll_dir,
                &[
                    (RollDir::Right, "tracecut.roll_right"),
                    (RollDir::Up, "tracecut.roll_up"),
                ],
                "tracecut.roll_dir",
            )
        });
    } else {
        row(ui, &tr("tracecut.sheet_width"), |ui| {
            length(
                ui,
                units,
                &mut s.width,
                (1.0, 100_000.0),
                "tracecut.sheet_width",
            )
        });
        row(ui, &tr("tracecut.sheet_height"), |ui| {
            length(
                ui,
                units,
                &mut s.height,
                (1.0, 100_000.0),
                "tracecut.sheet_height",
            )
        });
    }
    row(ui, &tr("tracecut.sheet_margin"), |ui| {
        length(
            ui,
            units,
            &mut s.margin,
            (0.0, 1000.0),
            "tracecut.sheet_margin",
        )
    });
    row(ui, &tr("tracecut.spacing"), |ui| {
        length(ui, units, &mut s.spacing, (0.0, 1000.0), "tracecut.spacing")
    });
    row(ui, &tr("tracecut.corner"), |ui| {
        choice(
            ui,
            "corner",
            &mut s.corner,
            &[
                (Corner::BottomLeft, "tracecut.origin_bottom_left"),
                (Corner::BottomRight, "tracecut.origin_bottom_right"),
                (Corner::TopLeft, "tracecut.origin_top_left"),
                (Corner::TopRight, "tracecut.origin_top_right"),
            ],
            "tracecut.corner",
        )
    });
}

/// The settings of the Nesting and Measure tools.
pub fn settings(ui: &mut Ui, units: Units, st: &mut TraceCutState) {
    match st.tool {
        ToolId::ShapeNesting => {
            let s = &mut st.settings.shape_nest;
            sheet(ui, units, &mut s.sheet);
            row(ui, &tr("tracecut.rotations"), |ui| {
                choice(
                    ui,
                    "rotations",
                    &mut s.rotations,
                    &[
                        (Rotations::None, "tracecut.rot_none"),
                        (Rotations::Half, "tracecut.rot_half"),
                        (Rotations::Quarter, "tracecut.rot_quarter"),
                        (Rotations::Eighth, "tracecut.rot_eighth"),
                    ],
                    "tracecut.rotations",
                )
            });
            row(ui, &tr("tracecut.accuracy"), |ui| {
                length(
                    ui,
                    units,
                    &mut s.tolerance,
                    (0.01, 50.0),
                    "tracecut.accuracy",
                )
            });
            row(ui, &tr("tracecut.search_time"), |ui| {
                number(ui, &mut s.seconds, (0.0, 60.0), 1, "tracecut.search_time")
            });
            row(ui, "", |ui| check(ui, &mut s.holes, "tracecut.nest_holes"));
            row(ui, "", |ui| {
                check(ui, &mut s.draw_sheets, "tracecut.draw_sheets")
            });
        }
        ToolId::RectNesting => {
            let s = &mut st.settings.rect_nest;
            sheet(ui, units, &mut s.sheet);
            row(ui, "", |ui| check(ui, &mut s.rotate, "tracecut.nest_turn"));
            row(ui, "", |ui| {
                check(ui, &mut s.draw_sheets, "tracecut.draw_sheets")
            });
        }
        ToolId::CuttingTime => time(ui, st),
        ToolId::Cost => {
            let s = &mut st.settings.cost;
            row(ui, &tr("tracecut.cost_material"), |ui| {
                ui.radio_value(&mut s.by_roll, true, tr("tracecut.cost_by_roll"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.cost_by_roll"));
                ui.radio_value(&mut s.by_roll, false, tr("tracecut.cost_by_area"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.cost_by_area"));
            });
            row(ui, &tr("tracecut.cost_material_price"), |ui| {
                number(
                    ui,
                    &mut s.material_price,
                    (0.0, 1e9),
                    2,
                    "tracecut.cost_material_price",
                )
            });
            if s.by_roll {
                row(ui, &tr("tracecut.roll_width"), |ui| {
                    length(
                        ui,
                        units,
                        &mut s.roll_width,
                        (1.0, 100_000.0),
                        "tracecut.roll_width",
                    )
                });
            }
            row(ui, &tr("tracecut.cost_cutting"), |ui| {
                number(ui, &mut s.cut_price, (0.0, 1e9), 2, "tracecut.cost_cutting")
            });
            row(ui, &tr("tracecut.cost_weeding"), |ui| {
                number(
                    ui,
                    &mut s.weed_price,
                    (0.0, 1e9),
                    2,
                    "tracecut.cost_weeding",
                )
            });
            row(ui, &tr("tracecut.cost_pieces"), |ui| {
                number(
                    ui,
                    &mut s.piece_price,
                    (0.0, 1e9),
                    2,
                    "tracecut.cost_pieces",
                )
            });
            row(ui, &tr("tracecut.cost_hours"), |ui| {
                number(ui, &mut s.hour_price, (0.0, 1e9), 2, "tracecut.cost_hours")
            });
            row(ui, &tr("tracecut.cost_reserve"), |ui| {
                length(
                    ui,
                    units,
                    &mut s.reserve,
                    (0.0, 1000.0),
                    "tracecut.cost_reserve",
                )
            });
            row(ui, &tr("tracecut.cost_copies"), |ui| {
                whole(ui, &mut s.copies, (1, 100_000), "tracecut.cost_copies")
            });
            row(ui, &tr("tracecut.cost_currency"), |ui| {
                ui.add_sized([110.0, 22.0], egui::TextEdit::singleline(&mut s.currency))
                    .rich_tip(crate::ui::tips::keyed("tracecut.cost_currency"));
            });
        }
        _ => {}
    }
}

/// Cutting Time: the machine's speeds.
fn time(ui: &mut Ui, st: &mut TraceCutState) {
    let s = &mut st.settings.time;
    row(ui, &tr("tracecut.cut_speed"), |ui| {
        number(
            ui,
            &mut s.cut_speed,
            (0.1, 100_000.0),
            1,
            "tracecut.cut_speed",
        )
    });
    row(ui, &tr("tracecut.travel_speed"), |ui| {
        number(
            ui,
            &mut s.travel_speed,
            (0.1, 100_000.0),
            1,
            "tracecut.travel_speed",
        )
    });
    row(ui, &tr("tracecut.lift_time"), |ui| {
        number(ui, &mut s.lift_time, (0.0, 60.0), 2, "tracecut.lift_time")
    });
    row(ui, &tr("tracecut.accel_factor"), |ui| {
        number(
            ui,
            &mut s.accel_factor,
            (1.0, 10.0),
            2,
            "tracecut.accel_factor",
        )
    });
    row(ui, &tr("tracecut.passes"), |ui| {
        whole(ui, &mut s.passes, (1, 100), "tracecut.passes")
    });
    row(ui, "", |ui| {
        check(ui, &mut s.nearest, "tracecut.time_nearest")
    });
}
