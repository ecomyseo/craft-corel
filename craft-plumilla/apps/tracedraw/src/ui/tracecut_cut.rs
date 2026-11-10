//! The settings of TraceCut's Cut tools in the TraceCut window: plotter
//! output (with the test cut), weeding, bridges, cut order, CutContour
//! and registration marks. See docs/behavior/tracecut.md.

use crate::app::Units;
use crate::i18n::tr;
use crate::tracecut::{ToolId, TraceCutState};
use crate::tracecut_send::{Destination, CAN_SEND};
use crate::ui::tips::RichTip;
use crate::ui::tracecut::{length, row};
use egui::Ui;
use tracedraw_tracecut::cut::hpgl::{Machine, Origin, MAX_PASSES};
use tracedraw_tracecut::cut::marks::MarkStyle;
use tracedraw_tracecut::cut::order::{Direction, StartAt};
use tracedraw_tracecut::cut::weed::{WeedMode, MAX_DIVIDERS};

/// What a Cut tool's settings asked for besides a change of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Cut the test square now.
    TestCut,
}

/// A plain number field.
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

/// A whole-number field.
fn count(ui: &mut Ui, v: &mut u32, range: (u32, u32), tip: &str) {
    let mut x = f64::from(*v);
    number(ui, &mut x, (f64::from(range.0), f64::from(range.1)), 0, tip);
    *v = x.round().clamp(f64::from(range.0), f64::from(range.1)) as u32;
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

/// A check box with its tip.
fn check(ui: &mut Ui, v: &mut bool, key: &str) {
    ui.checkbox(v, tr(key))
        .rich_tip(crate::ui::tips::keyed(key));
}

/// The settings of a Cut tool other than the clean-up ones; `None` for
/// those.
pub fn settings(ui: &mut Ui, units: Units, st: &mut TraceCutState) -> Option<Action> {
    let mut action = None;
    match st.tool {
        ToolId::PlotterOutput => action = plotter(ui, units, st),
        ToolId::Weeding => {
            let s = &mut st.settings.weeding;
            row(ui, &tr("tracecut.weed_mode"), |ui| {
                choice(
                    ui,
                    "weed_mode",
                    &mut s.mode,
                    &[
                        (WeedMode::Lines, "tracecut.weed_lines"),
                        (WeedMode::Boxes, "tracecut.weed_boxes"),
                    ],
                    "tracecut.weed_mode",
                )
            });
            match s.mode {
                WeedMode::Lines => {
                    row(ui, &tr("tracecut.weed_gap"), |ui| {
                        length(ui, units, &mut s.gap, (0.0, 100.0), "tracecut.weed_gap")
                    });
                    row(ui, &tr("tracecut.weed_shorten"), |ui| {
                        length(
                            ui,
                            units,
                            &mut s.shorten,
                            (0.0, 100.0),
                            "tracecut.weed_shorten",
                        )
                    });
                    row(ui, &tr("tracecut.weed_dividers"), |ui| {
                        count(
                            ui,
                            &mut s.dividers,
                            (0, MAX_DIVIDERS),
                            "tracecut.weed_dividers",
                        )
                    });
                }
                WeedMode::Boxes => {
                    row(ui, &tr("tracecut.weed_box_x"), |ui| {
                        length(
                            ui,
                            units,
                            &mut s.box_x,
                            (0.0, 1000.0),
                            "tracecut.weed_box_x",
                        )
                    });
                    row(ui, &tr("tracecut.weed_box_y"), |ui| {
                        length(
                            ui,
                            units,
                            &mut s.box_y,
                            (0.0, 1000.0),
                            "tracecut.weed_box_y",
                        )
                    });
                }
            }
            row(ui, "", |ui| check(ui, &mut s.frame, "tracecut.weed_frame"));
            if s.frame {
                row(ui, &tr("tracecut.weed_margin"), |ui| {
                    length(
                        ui,
                        units,
                        &mut s.margin,
                        (0.0, 1000.0),
                        "tracecut.weed_margin",
                    )
                });
            }
        }
        ToolId::Bridges => {
            let s = &mut st.settings.bridges;
            row(ui, &tr("tracecut.bridge_length"), |ui| {
                length(
                    ui,
                    units,
                    &mut s.length,
                    (0.1, 100.0),
                    "tracecut.bridge_length",
                )
            });
            row(ui, &tr("tracecut.bridge_count"), |ui| {
                ui.radio_value(&mut s.per_outline, true, tr("tracecut.bridge_per_outline"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.bridge_per_outline"));
                ui.radio_value(&mut s.per_outline, false, tr("tracecut.bridge_every"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.bridge_every"));
            });
            if s.per_outline {
                row(ui, "", |ui| {
                    count(ui, &mut s.count, (1, 100), "tracecut.bridge_per_outline")
                });
            } else {
                row(ui, "", |ui| {
                    length(
                        ui,
                        units,
                        &mut s.every,
                        (1.0, 10_000.0),
                        "tracecut.bridge_every",
                    )
                });
            }
            row(ui, &tr("tracecut.bridge_min"), |ui| {
                length(
                    ui,
                    units,
                    &mut s.min_outline,
                    (0.0, 100_000.0),
                    "tracecut.bridge_min",
                )
            });
        }
        ToolId::CutOrder => {
            let s = &mut st.settings.order;
            row(ui, &tr("tracecut.order_start"), |ui| {
                choice(
                    ui,
                    "order_start",
                    &mut s.start,
                    &[
                        (StartAt::Nearest, "tracecut.start_nearest"),
                        (StartAt::BottomLeft, "tracecut.start_bottom_left"),
                        (StartAt::Keep, "tracecut.start_keep"),
                    ],
                    "tracecut.order_start",
                )
            });
            row(ui, &tr("tracecut.order_direction"), |ui| {
                choice(
                    ui,
                    "order_direction",
                    &mut s.direction,
                    &[
                        (Direction::Keep, "tracecut.direction_keep"),
                        (Direction::Clockwise, "tracecut.direction_cw"),
                        (Direction::CounterClockwise, "tracecut.direction_ccw"),
                    ],
                    "tracecut.order_direction",
                )
            });
            row(ui, "", |ui| {
                check(ui, &mut s.inner_first, "tracecut.inner_first")
            });
        }
        ToolId::CutContour => {
            let s = &mut st.settings.contour;
            row(ui, &tr("tracecut.contour_width"), |ui| {
                length(
                    ui,
                    units,
                    &mut s.width,
                    (0.0, 10.0),
                    "tracecut.contour_width",
                )
            });
            row(ui, "", |ui| {
                check(ui, &mut s.remove_fill, "tracecut.contour_remove_fill")
            });
        }
        ToolId::RegistrationMarks => {
            let s = &mut st.settings.marks;
            row(ui, &tr("tracecut.marks_place"), |ui| {
                ui.radio_value(&mut s.page, false, tr("tracecut.marks_around"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.marks_around"));
                ui.radio_value(&mut s.page, true, tr("tracecut.marks_page"))
                    .rich_tip(crate::ui::tips::keyed("tracecut.marks_page"));
            });
            let r = &mut s.spec;
            row(ui, &tr("tracecut.marks_style"), |ui| {
                choice(
                    ui,
                    "marks_style",
                    &mut r.style,
                    &[
                        (MarkStyle::Square, "tracecut.mark_square"),
                        (MarkStyle::Circle, "tracecut.mark_circle"),
                        (MarkStyle::Corner, "tracecut.mark_corner"),
                    ],
                    "tracecut.marks_style",
                )
            });
            if r.style == MarkStyle::Corner {
                row(ui, &tr("tracecut.mark_arm"), |ui| {
                    length(
                        ui,
                        units,
                        &mut r.corner_length,
                        (1.0, 200.0),
                        "tracecut.mark_arm",
                    )
                });
                row(ui, &tr("tracecut.mark_arm_width"), |ui| {
                    length(
                        ui,
                        units,
                        &mut r.corner_width,
                        (0.1, 50.0),
                        "tracecut.mark_arm_width",
                    )
                });
            } else {
                row(ui, &tr("tracecut.mark_size"), |ui| {
                    length(ui, units, &mut r.size, (0.5, 100.0), "tracecut.mark_size")
                });
            }
            row(ui, &tr("tracecut.mark_offset"), |ui| {
                length(
                    ui,
                    units,
                    &mut r.offset,
                    (0.0, 200.0),
                    "tracecut.mark_offset",
                )
            });
            row(ui, &tr("tracecut.mark_spacing"), |ui| {
                length(
                    ui,
                    units,
                    &mut r.spacing,
                    (0.0, 10_000.0),
                    "tracecut.mark_spacing",
                )
            });
        }
        _ => {}
    }
    action
}

/// Plotter Output: the machine and its knife, the order, where the job
/// goes, and the test cut.
fn plotter(ui: &mut Ui, units: Units, st: &mut TraceCutState) -> Option<Action> {
    let s = &mut st.settings.plotter;
    let mut action = None;
    // The machine sets the commands; a profile with commands of its own
    // shows as Custom.
    let mut machine = Machine::of(&s.profile);
    row(ui, &tr("tracecut.machine"), |ui| {
        let shown = match machine {
            Some(m) => tr(machine_key(m)),
            None => tr("tracecut.machine_custom"),
        };
        crate::ui::keynav::ComboBox::from_id_salt(("tracecut", "machine"))
            .selected_text(shown)
            .width(200.0)
            .show_ui(ui, |ui| {
                for m in Machine::ALL {
                    ui.selectable_value(&mut machine, Some(m), tr(machine_key(m)));
                }
            })
            .response
            .rich_tip(crate::ui::tips::keyed("tracecut.machine"));
    });
    if let Some(m) = machine {
        if Machine::of(&s.profile) != Some(m) {
            s.profile = m.apply(&s.profile);
        }
    }
    let p = &mut s.profile;
    row(ui, &tr("tracecut.knife_offset"), |ui| {
        length(
            ui,
            units,
            &mut p.knife_offset,
            (0.0, 2.0),
            "tracecut.knife_offset",
        )
    });
    row(ui, &tr("tracecut.overcut"), |ui| {
        length(ui, units, &mut p.overcut, (0.0, 10.0), "tracecut.overcut")
    });
    row(ui, &tr("tracecut.passes"), |ui| {
        count(ui, &mut p.passes, (1, MAX_PASSES), "tracecut.passes")
    });
    row(ui, &tr("tracecut.speed"), |ui| {
        number(ui, &mut p.velocity, (0.0, 2000.0), 1, "tracecut.speed")
    });
    row(ui, &tr("tracecut.force"), |ui| {
        number(ui, &mut p.force, (0.0, 2000.0), 0, "tracecut.force")
    });
    row(ui, &tr("tracecut.origin"), |ui| {
        choice(
            ui,
            "origin",
            &mut p.origin,
            &[
                (Origin::BottomLeft, "tracecut.origin_bottom_left"),
                (Origin::BottomRight, "tracecut.origin_bottom_right"),
                (Origin::TopLeft, "tracecut.origin_top_left"),
                (Origin::TopRight, "tracecut.origin_top_right"),
            ],
            "tracecut.origin",
        )
    });
    row(ui, "", |ui| {
        check(ui, &mut p.mirror, "tracecut.mirror");
        check(ui, &mut p.rotate, "tracecut.rotate");
    });
    row(ui, "", |ui| {
        check(ui, &mut p.weed_frame, "tracecut.weed_frame")
    });
    if p.weed_frame {
        row(ui, &tr("tracecut.weed_margin"), |ui| {
            length(
                ui,
                units,
                &mut p.weed_margin,
                (0.0, 1000.0),
                "tracecut.weed_margin",
            )
        });
    }
    row(ui, "", |ui| {
        check(ui, &mut s.optimize, "tracecut.optimize");
        if s.optimize {
            check(ui, &mut s.inner_first, "tracecut.inner_first");
        }
    });
    if CAN_SEND {
        row(ui, &tr("tracecut.destination"), |ui| {
            for d in Destination::ALL {
                ui.radio_value(&mut s.destination, d, tr(d.key()))
                    .rich_tip(crate::ui::tips::keyed(d.key()));
            }
        });
        match s.destination {
            Destination::Port => {
                row(ui, &tr("tracecut.port"), |ui| {
                    ui.add_sized([200.0, 22.0], egui::TextEdit::singleline(&mut s.port))
                        .rich_tip(crate::ui::tips::keyed("tracecut.port"));
                });
            }
            Destination::Network => {
                row(ui, &tr("tracecut.host"), |ui| {
                    ui.add_sized(
                        [200.0, 22.0],
                        egui::TextEdit::singleline(&mut s.host).hint_text("192.168.1.20:9100"),
                    )
                    .rich_tip(crate::ui::tips::keyed("tracecut.host"));
                });
            }
            Destination::File => {}
        }
    }
    row(ui, &tr("tracecut.test_size"), |ui| {
        length(
            ui,
            units,
            &mut s.test_size,
            (5.0, 200.0),
            "tracecut.test_size",
        );
        if ui
            .button(tr("tracecut.test_cut"))
            .rich_tip(crate::ui::tips::keyed("tracecut.test_cut"))
            .clicked()
        {
            action = Some(Action::TestCut);
        }
    });
    action
}

/// The i18n key of a built-in machine's name.
fn machine_key(m: Machine) -> &'static str {
    match m {
        Machine::Hpgl => "tracecut.machine_hpgl",
        Machine::HpglPenSelect => "tracecut.machine_hpgl_pen",
        Machine::HpglLines => "tracecut.machine_hpgl_lines",
    }
}
