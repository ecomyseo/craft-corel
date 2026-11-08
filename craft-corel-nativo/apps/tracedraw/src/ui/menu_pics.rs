//! Pictures for menu rows of the Text, Bitmaps and Window menus, drawn on
//! a 16 by 16 grid like the property bar pictures.

use crate::theme::Tokens;
use crate::ui::propbar::Pic;
use egui::{epaint, Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

fn q(r: Rect, x: f32, y: f32) -> Pos2 {
    r.min + Vec2::new(x, y) * (r.width() / 16.0)
}

/// Accent colours for the parts that carry meaning (a close cross, a new
/// window badge, a link); disabled rows draw everything in `color`.
const RED: Color32 = Color32::from_rgb(0xD0, 0x2B, 0x2B);
const GREEN: Color32 = Color32::from_rgb(0x1E, 0x9E, 0x3C);
const BLUE: Color32 = Color32::from_rgb(0x1F, 0x6F, 0xC5);

pub fn draw(painter: &Painter, r: Rect, pic: Pic, color: Color32) {
    let k = r.width() / 16.0;
    let s = Stroke::new(1.1 * k, color);
    let thin = Stroke::new(0.85 * k, color);
    let bar = Stroke::new(1.3 * k, color);
    let enabled = color == Tokens::icon();
    let accent = |c: Color32| if enabled { c } else { color };
    let line = |a: (f32, f32), b: (f32, f32), st: Stroke| {
        painter.line_segment([q(r, a.0, a.1), q(r, b.0, b.1)], st);
    };
    let outline = |pts: &[(f32, f32)], st: Stroke| {
        painter.add(epaint::PathShape::closed_line(
            pts.iter().map(|(x, y)| q(r, *x, *y)).collect(),
            st,
        ));
    };
    let polyline = |pts: &[(f32, f32)], st: Stroke| {
        painter.add(epaint::PathShape::line(
            pts.iter().map(|(x, y)| q(r, *x, *y)).collect(),
            st,
        ));
    };
    let filled = |pts: &[(f32, f32)], fill: Color32| {
        painter.add(epaint::PathShape::convex_polygon(
            pts.iter().map(|(x, y)| q(r, *x, *y)).collect(),
            fill,
            Stroke::NONE,
        ));
    };
    let rect = |x0: f32, y0: f32, x1: f32, y1: f32, st: Stroke| {
        outline(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)], st);
    };
    let text = |x: f32, y: f32, t: &str, size: f32| {
        painter.text(
            q(r, x, y),
            Align2::CENTER_CENTER,
            t,
            FontId::proportional(size * k),
            color,
        );
    };
    // A window: frame and title bar.
    let window = |x0: f32, y0: f32, x1: f32, y1: f32| {
        painter.rect_filled(
            Rect::from_min_max(q(r, x0, y0), q(r, x1, y1)),
            0.0,
            Tokens::field(),
        );
        rect(x0, y0, x1, y1, thin);
        painter.rect_filled(
            Rect::from_min_max(q(r, x0, y0), q(r, x1, y0 + 2.0)),
            0.0,
            accent(BLUE),
        );
    };
    let cross = |cx: f32, cy: f32, h: f32, c: Color32| {
        let st = Stroke::new(1.5 * k, c);
        line((cx - h, cy - h), (cx + h, cy + h), st);
        line((cx - h, cy + h), (cx + h, cy - h), st);
    };
    match pic {
        Pic::Tabs => {
            // Tab stops on a ruler above indented lines.
            line((1.0, 3.0), (15.0, 3.0), thin);
            for x in [4.0, 9.0] {
                polyline(&[(x, 1.0), (x, 5.0), (x + 2.0, 5.0)], thin);
            }
            for (x0, y) in [(1.0, 8.0), (4.0, 11.0), (9.0, 14.0)] {
                line((x0, y), (15.0, y), bar);
            }
        }
        Pic::Columns => {
            for x0 in [1.0, 9.0] {
                for i in 0..5 {
                    let y = 2.0 + i as f32 * 3.0;
                    line((x0, y), (x0 + 6.0, y), bar);
                }
            }
        }
        Pic::TextStatistics => {
            text(5.0, 4.5, "ab", 8.5);
            for (i, h) in [4.0, 7.0, 5.0, 9.0].into_iter().enumerate() {
                let x = 2.0 + i as f32 * 3.5;
                painter.rect_filled(
                    Rect::from_min_max(q(r, x, 15.0 - h), q(r, x + 2.0, 15.0)),
                    0.0,
                    color,
                );
            }
        }
        Pic::ConvertText => {
            // A letter beside a frame of lines.
            text(4.0, 6.0, "A", 10.0);
            rect(8.5, 1.5, 15.0, 14.5, thin);
            for y in [5.0, 8.0, 11.0] {
                line((10.0, y), (13.5, y), thin);
            }
            polyline(&[(2.0, 11.5), (2.0, 14.0), (6.5, 14.0)], thin);
        }
        Pic::FitTextToPath => {
            // Letters riding a curve with a node.
            let pts: Vec<(f32, f32)> = (0..=12)
                .map(|i| {
                    let t = i as f32 / 12.0;
                    (
                        1.5 + t * 13.0,
                        13.0 - (t * std::f32::consts::PI).sin() * 8.0,
                    )
                })
                .collect();
            polyline(&pts, s);
            rect(0.5, 12.0, 2.5, 14.0, thin);
            rect(13.5, 12.0, 15.5, 14.0, thin);
            text(8.0, 2.5, "T", 6.0);
        }
        Pic::StraightenText => {
            text(5.0, 4.5, "a", 8.0);
            text(11.0, 6.0, "a", 8.0);
            line((1.0, 13.5), (15.0, 13.5), s);
            polyline(&[(8.0, 9.0), (8.0, 12.0)], thin);
        }
        Pic::AlignBaseline => {
            text(4.0, 6.5, "A", 9.0);
            text(10.5, 8.0, "a", 9.0);
            line((1.0, 12.5), (15.0, 12.5), thin);
            line((1.0, 14.5), (15.0, 14.5), thin);
        }
        Pic::ChangeCase => {
            text(5.0, 9.0, "a", 8.0);
            text(11.0, 7.5, "A", 11.0);
            polyline(&[(3.0, 4.0), (6.0, 1.5), (9.0, 4.0)], thin);
        }
        Pic::Encode => {
            text(4.0, 11.0, "A", 9.0);
            text(10.5, 4.5, "</>", 6.0);
        }
        Pic::ConvertToBitmap => {
            // A shape becoming a grid of pixels.
            outline(&[(1.0, 8.0), (4.5, 2.0), (8.0, 8.0)], s);
            for i in 0..3 {
                for j in 0..3 {
                    let (x, y) = (8.5 + i as f32 * 2.3, 8.5 + j as f32 * 2.3);
                    let on = (i + j) % 2 == 0;
                    let rr = Rect::from_min_max(q(r, x, y), q(r, x + 2.0, y + 2.0));
                    if on {
                        painter.rect_filled(rr, 0.0, color);
                    } else {
                        painter.rect_stroke(rr, 0.0, thin, egui::StrokeKind::Inside);
                    }
                }
            }
        }
        Pic::StraightenImage => {
            // A tilted picture and a level.
            outline(&[(3.0, 2.0), (14.0, 4.0), (12.5, 12.0), (1.5, 10.0)], thin);
            line((1.0, 14.5), (15.0, 14.5), s);
            filled(&[(5.0, 9.0), (8.0, 5.5), (11.0, 9.5)], color);
        }
        Pic::EditBitmap => {
            rect(1.5, 3.5, 11.5, 14.5, thin);
            filled(&[(3.0, 13.0), (6.0, 8.5), (9.0, 13.0)], color);
            // A pencil.
            line((9.0, 9.0), (15.0, 3.0), Stroke::new(2.0 * k, color));
            line((8.5, 9.5), (9.0, 9.0), thin);
        }
        Pic::CropBitmap => {
            polyline(&[(4.0, 1.0), (4.0, 12.0), (15.0, 12.0)], s);
            polyline(&[(1.0, 4.0), (12.0, 4.0), (12.0, 15.0)], s);
        }
        Pic::Resample => {
            rect(1.5, 6.5, 9.5, 14.5, thin);
            rect(1.5, 1.5, 14.5, 14.5, Stroke::new(0.7 * k, color));
            line((7.0, 9.0), (13.0, 3.0), thin);
            filled(&[(13.5, 2.5), (10.0, 3.2), (12.8, 6.0)], color);
        }
        Pic::BreakLink | Pic::UpdateLink => {
            // Two chain links, apart when broken.
            let gap = if pic == Pic::BreakLink { 1.5 } else { 0.0 };
            let link = |x0: f32, y0: f32| {
                painter.rect_stroke(
                    Rect::from_min_max(q(r, x0, y0), q(r, x0 + 7.0, y0 + 4.0)),
                    2.0 * k,
                    Stroke::new(1.3 * k, accent(BLUE)),
                    egui::StrokeKind::Middle,
                );
            };
            link(1.0, 9.0 + gap);
            link(5.0 + gap * 2.0, 6.0 - gap);
            if pic == Pic::BreakLink {
                cross(13.0, 13.0, 1.8, accent(RED));
            } else {
                polyline(&[(11.0, 1.5), (14.0, 1.5), (14.0, 4.5)], thin);
                line((14.0, 1.5), (10.5, 5.0), thin);
            }
        }
        Pic::QuickTrace => {
            rect(1.5, 1.5, 10.5, 10.5, Stroke::new(0.7 * k, color));
            for i in 0..3 {
                for j in 0..3 {
                    if (i + j) % 2 == 1 {
                        let (x, y) = (1.5 + i as f32 * 3.0, 1.5 + j as f32 * 3.0);
                        painter.rect_filled(
                            Rect::from_min_max(q(r, x, y), q(r, x + 3.0, y + 3.0)),
                            0.0,
                            color,
                        );
                    }
                }
            }
            let pts: Vec<(f32, f32)> = (0..=10)
                .map(|i| {
                    let t = i as f32 / 10.0;
                    (6.0 + t * 9.0, 14.5 - (t * 3.0).sin() * 5.0)
                })
                .collect();
            polyline(&pts, Stroke::new(1.3 * k, accent(BLUE)));
        }
        Pic::NewWindow => {
            window(1.5, 3.5, 12.5, 14.5);
            painter.rect_filled(
                Rect::from_min_max(q(r, 9.5, 0.5), q(r, 15.5, 6.5)),
                0.0,
                accent(GREEN),
            );
            let w = Stroke::new(1.2 * k, Color32::WHITE);
            line((10.5, 3.5), (14.5, 3.5), w);
            line((12.5, 1.5), (12.5, 5.5), w);
        }
        Pic::RefreshWindow => {
            window(1.5, 3.5, 12.5, 14.5);
            let c = q(r, 11.5, 4.5);
            let pts: Vec<Pos2> = (0..=10)
                .map(|i| {
                    let a = 0.6 + i as f32 * 0.5;
                    c + Vec2::new(a.cos(), a.sin()) * 3.3 * k
                })
                .collect();
            painter.add(epaint::PathShape::line(
                pts,
                Stroke::new(1.4 * k, accent(RED)),
            ));
            filled(&[(14.5, 6.5), (15.5, 3.5), (12.5, 4.5)], accent(RED));
        }
        Pic::CloseWindow => {
            window(1.5, 3.5, 12.5, 14.5);
            cross(12.5, 4.0, 2.5, accent(RED));
        }
        Pic::CloseAll => {
            window(3.5, 1.5, 14.5, 10.5);
            window(1.5, 5.5, 11.5, 14.5);
            cross(12.5, 12.0, 2.5, accent(RED));
        }
        Pic::Cascade => {
            window(1.0, 1.0, 10.0, 8.0);
            window(3.5, 4.5, 12.5, 11.5);
            window(6.0, 8.0, 15.0, 15.0);
        }
        Pic::TileHorizontally => {
            window(1.5, 1.0, 14.5, 7.5);
            window(1.5, 8.5, 14.5, 15.0);
        }
        Pic::TileVertically => {
            window(1.0, 1.5, 7.5, 14.5);
            window(8.5, 1.5, 15.0, 14.5);
        }
        Pic::CombineWindows => {
            window(1.0, 1.0, 9.0, 9.0);
            window(7.0, 7.0, 15.0, 15.0);
            line((9.5, 4.0), (12.0, 4.0), thin);
            line((4.0, 9.5), (4.0, 12.0), thin);
        }
        Pic::DockWindow => {
            rect(1.0, 1.0, 15.0, 15.0, thin);
            window(3.0, 5.0, 13.0, 13.0);
            line((8.0, 1.0), (8.0, 4.0), thin);
        }
        _ => {}
    }
}
