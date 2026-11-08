//! Toolbox icons drawn with the painter, no bitmaps: the "bead on a
//! wire" style of docs/behavior/icons.md on a 24 unit grid. Ink draws
//! the object, the accent (brand blue) the action, the node motif marks
//! the live point.

use crate::theme::Tokens;
use crate::tools::Tool;
use crate::ui::icon_paint::{polar, ribbon, rrect, spiral, star, Pen, LINE, THIN};
use egui::{epaint, Color32, Painter, Pos2, Rect, Stroke, Vec2};

fn p(r: Rect, x: f32, y: f32) -> Pos2 {
    r.min + Vec2::new(x, y) * (r.width() / 16.0)
}

static COLORED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Appearance > Coloured tool icons.
pub fn set_colored(on: bool) {
    COLORED.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// The colour of a tool's group, for coloured icons.
pub fn group_color(tool: Tool) -> Color32 {
    let group = crate::tools::GROUPS
        .iter()
        .position(|g| g.tools.contains(&tool))
        .unwrap_or(0);
    // One hue per toolbox group, dark enough for light panels: Pick,
    // Shape, Crop, Zoom, Curve, the brushes and Variable Outline (one
    // hue, as one block), Rectangle, Ellipse and Polygon (one hue), Text,
    // Dimension, Connector, Effect, Transparency, Eyedropper, Fill,
    // Outline.
    const HUES: [u32; 18] = [
        0x2F6FD0, 0x8A4FD0, 0xD24A43, 0x2A9D9D, 0x2E9E4F, 0xE07B1A, 0xE07B1A, 0x1E88E5, 0x1E88E5,
        0x1E88E5, 0x5A5A5A, 0x7A8288, 0x16A085, 0xC0398B, 0x5D6D9E, 0xB9770E, 0xE67E22, 0x34495E,
    ];
    const _: () = assert!(HUES.len() == crate::tools::GROUPS.len());
    let v = HUES.get(group).copied().unwrap_or(0x5A5A5A);
    let c = Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8);
    if crate::theme::theme().is_dark() {
        // Lighter on dark panels.
        let mix = |a: u8| (a as f32 + (255.0 - a as f32) * 0.35) as u8;
        Color32::from_rgb(mix(c.r()), mix(c.g()), mix(c.b()))
    } else {
        c
    }
}

/// The colour a tool's icon is drawn in: its group's with coloured
/// icons on (not in the high contrast theme), else `plain`.
pub fn tool_color(tool: Tool, plain: Color32) -> Color32 {
    if COLORED.load(std::sync::atomic::Ordering::Relaxed)
        && crate::theme::theme() != crate::theme::Theme::HighContrast
    {
        group_color(tool)
    } else {
        plain
    }
}

/// The accent of the icons: the brand blue, lighter on dark panels.
pub fn accent() -> Color32 {
    if crate::theme::theme().is_dark() {
        Color32::from_rgb(0x5C, 0xAE, 0xFF)
    } else {
        Color32::from_rgb(0x2F, 0x6F, 0xD0)
    }
}

/// Draw `tool`'s icon in `r` with `color` as its ink, on the panel.
pub fn draw(painter: &Painter, r: Rect, tool: Tool, color: Color32) {
    draw_on(painter, r, tool, color, Tokens::panel());
}

/// Draw `tool`'s icon in `r` with `color` as its ink, on a surface of
/// colour `bg` (the opaque bodies take it). A dimmed icon (the theme's
/// disabled icon colour or a translucent ink) stays monochrome; the high
/// contrast theme keeps its one icon colour.
pub fn draw_on(painter: &Painter, r: Rect, tool: Tool, color: Color32, bg: Color32) {
    let plain = color == Tokens::icon_off()
        || color.a() < 255
        || crate::theme::theme() == crate::theme::Theme::HighContrast;
    let acc = if plain { color } else { accent() };
    let pen = Pen::new(painter, r, color, acc, bg);
    let ink = color;
    let a = |f: f32| Pen::alpha(ink, f);
    let b = |f: f32| Pen::alpha(acc, f);
    let pt = |x: f32, y: f32| Pos2::new(x, y);
    match tool {
        // Pick group.
        Tool::Pick => pen.pointer(5.4, 3.4, 1.0, true),
        Tool::FreeformPick => {
            // A lasso closing on the live node, with a small pointer.
            pen.ghost("M5 12c-2-6 4-9 9-8s7 5 5 9-6 4-9 3", ink);
            pen.line("M5 12c0 3 2 5 4 7", ink);
            pen.node(9.0, 19.0, true);
            pen.pointer(13.5, 12.5, 0.58, false);
        }
        Tool::FreeTransform => {
            // A tilted box, the rotation arc and the node on the pivot.
            pen.rotated(-14.0, 11.0, 13.0)
                .body(&rrect(5.0, 7.0, 12.0, 12.0, 1.5));
            pen.line("M14.5 4.5a6 6 0 0 1 6 4.5", acc);
            pen.arrow_head(20.8, 10.2, 0.3, 1.0, 3.2, acc);
            pen.node(3.73, 8.63, true);
        }

        // Shape edit group.
        Tool::Shape => {
            // A node with its two handles on a curve, the pointer on it.
            pen.line("M3 8C6 16 9 16 12 12", ink);
            pen.line("M12 12c3-4 6-5 9-9", ink);
            pen.thin("M15.5 7.3L8.5 16.7", acc);
            pen.handle(15.5, 7.3, 1.8);
            pen.handle(8.5, 16.7, 1.8);
            pen.node(12.0, 12.0, true);
            pen.pointer(12.8, 12.9, 0.58, false);
        }
        Tool::Smooth => {
            // Jagged ink becomes a smooth wave from the node on.
            pen.line("M3 16l3-7 3 7 3-7", ink);
            pen.line("M12 9c2.5 0 3 7 5.5 7s3-7 4.5-7", acc);
            pen.node(12.0, 9.0, true);
        }
        Tool::Smear => {
            // A blob pulled into a tail, the node on the pulled tip.
            pen.body("M8 5a6.5 6.5 0 0 0 0 13c3 0 4-2.5 7-3.5l5.5-2.5-5.5-2.5C12 8.5 11 5 8 5z");
            pen.node(20.5, 12.0, true);
        }
        Tool::Twirl => {
            // A shape with a spiral winding inside, the node at its end.
            pen.body(&rrect(3.0, 3.0, 18.0, 18.0, 4.0));
            pen.stroke_pts(
                &spiral(12.0, 12.0, 1.75, 0.5, 6.2, 50, -40.0),
                false,
                LINE,
                acc,
            );
            let (x, y) = polar(12.0, 12.0, 6.2, -40.0 + 1.75 * 360.0);
            pen.node(x, y, true);
        }
        Tool::AttractRepel => {
            // Four arrows converging on a shape.
            pen.circle(12.0, 12.0, 3.2, pen.wash(), LINE, ink);
            for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let (fx, fy) = (12.0 + 7.2 * dx, 12.0 + 7.2 * dy);
                let (tx, ty) = (12.0 + 4.6 * dx, 12.0 + 4.6 * dy);
                pen.stroke_pts(
                    &[pt(fx, fy), pt(tx + 1.2 * dx, ty + 1.2 * dy)],
                    false,
                    THIN,
                    acc,
                );
                pen.arrow_head(tx, ty, -dx, -dy, 2.6, acc);
            }
        }
        Tool::Smudge => {
            // A round brush dragging paint into a tapered smear.
            pen.stroke("M20 4l-7.5 7.5", 3.0, ink);
            pen.fill(
                "M12.5 11.5c-1.5-.5-3.5.5-4 2.5s0 3.5 2 3 3.2-2.5 3.5-4z",
                ink,
                true,
            );
            let smear = ribbon(
                [pt(9.0, 15.0), pt(7.0, 17.0), pt(5.5, 18.0), pt(3.0, 20.5)],
                |t| 0.3 + 1.7 * (1.0 - t),
                16,
            );
            pen.fill_pts(&smear, acc, true);
        }
        Tool::Roughen => {
            // A body in ink, its top edge sawn into a zigzag.
            pen.body("M4 9v10a1.5 1.5 0 0 0 1.5 1.5h13A1.5 1.5 0 0 0 20 19V9");
            pen.line("M4 9l2-4 2 4 2-4 2 4 2-4 2 4 2-4 2 4", acc);
        }

        Tool::VariableOutline => {
            // An ink ribbon swelling in the middle, the width handle
            // pulled off the curve.
            let band = ribbon(
                [pt(3.0, 18.0), pt(7.0, 6.0), pt(17.0, 6.0), pt(21.0, 18.0)],
                |t| 0.5 + 2.6 * (std::f32::consts::PI * t).sin(),
                28,
            );
            pen.fill_pts(&band, pen.wash(), false);
            pen.stroke_pts(&band, true, LINE, ink);
            pen.thin("M12 9V3.5", acc);
            pen.handle(12.0, 3.5, 1.8);
            pen.node(12.0, 9.0, true);
        }

        // Crop group.
        Tool::Crop => {
            // Two interlocking corner brackets framing the kept square,
            // a solid accent dot on the corner being dragged.
            pen.line("M7 3V17H21", ink);
            pen.line("M3 7H17V21", ink);
            pen.rect(15.0, 15.0, 4.0, 4.0, 0.8, acc, 0.0, Color32::TRANSPARENT);
        }
        Tool::Knife => {
            // A knife on the diagonal: the solid handle at the top right,
            // a wide blade with a straight back and a curved edge to the
            // point at the bottom left, the cut it leaves in the accent.
            let t = pen.rotated(-40.0, 12.0, 12.0);
            t.fill(&rrect(14.0, 8.5, 8.0, 5.0, 1.6), ink, true);
            t.body("M2 10H14V16.5C9.5 16.5 5 14.5 2 10Z");
            pen.line("M3 21.5h6", acc);
        }
        Tool::SegmentDelete => {
            // A curve with the doomed segment a ghost, crossed out.
            pen.line("M3 17C7 5 10 5 12 11", ink);
            pen.ghost("M12 11c2 6 5 6 9-6", acc);
            pen.line("M14.5 13.5l4 4M18.5 13.5l-4 4", acc);
            pen.node(3.0, 17.0, false);
            pen.node(12.0, 11.0, false);
            pen.node(21.0, 5.0, false);
        }
        Tool::Eraser => {
            // A block eraser on its end at 45 degrees: the sleeve solid,
            // the rubber a body, the rubbed-out mark in the accent.
            let t = pen.rotated(45.0, 12.0, 12.0);
            t.body(&rrect(8.5, 3.0, 7.0, 16.0, 1.5));
            t.fill(
                "M8.5 4.5a1.5 1.5 0 0 1 1.5-1.5h4a1.5 1.5 0 0 1 1.5 1.5V9h-7z",
                ink,
                true,
            );
            pen.line("M3 21h6", acc);
        }

        // Zoom group.
        Tool::Zoom => {
            // A lens showing a magnified curve and node.
            pen.circle(10.0, 10.0, 7.5, pen.wash(), LINE, ink);
            pen.stroke("M15.5 15.5L21 21", 3.0, ink);
            pen.line("M4.5 12.5c3 0 4.5-6 9-5.5", acc);
            pen.node(13.5, 7.0, true);
        }
        Tool::Pan => {
            // A page shifted from its ghost, the drag arrow.
            pen.ghost(&rrect(3.0, 3.0, 13.0, 13.0, 1.5), a(0.4));
            pen.solid(&rrect(8.0, 8.0, 13.0, 13.0, 1.5));
            pen.line("M6 6l5.5 5.5", acc);
            pen.arrow_head(13.5, 13.5, 1.0, 1.0, 3.0, acc);
            pen.node(6.0, 6.0, true);
        }

        // Curve group.
        Tool::Freehand => {
            // A pencil drawing a scribble; the live end is a node.
            pen.body("M9.2 14.2l9.4-9.4a1.6 1.6 0 0 1 2.3 0l.3.3a1.6 1.6 0 0 1 0 2.3l-9.4 9.4z");
            pen.fill("M9.2 14.2l2.6 2.6-5.3 1.4 1.3-5.3z", ink, false);
            pen.stroke("M9.2 14.2l2.6 2.6-5.3 1.4 1.3-5.3z", 1.2, ink);
            pen.thin("M17.2 6.2l1.4 1.4", ink);
            pen.line("M6.5 18.2c1.5 2.5 3 3 5 1.5s3-2 5-.5 3 1.5 4.5.5", acc);
            pen.node(21.0, 19.7, true);
        }
        Tool::TwoPointLine => {
            pen.line("M4 19L20 5", ink);
            pen.node(4.0, 19.0, false);
            pen.node(20.0, 5.0, true);
        }
        Tool::Bezier => {
            // An S curve with vertical handles, the live end node.
            pen.line("M4 19C4 7 20 17 20 5", ink);
            pen.thin("M4 19v-6M20 5v6", acc);
            pen.handle(4.0, 13.0, 1.8);
            pen.handle(20.0, 11.0, 1.8);
            pen.node(4.0, 19.0, false);
            pen.node(20.0, 5.0, true);
        }
        Tool::Pen => {
            // A nib drawing a segment to a node.
            pen.body("M9 15l2.5-8.5L19 3l2 2-3.5 7.5z");
            pen.thin("M9 15l5-5", ink);
            pen.circle(15.0, 9.0, 1.3, ink, 0.0, ink);
            pen.line("M9 15c-2 1-4 3-5 5", acc);
            pen.node(4.0, 20.0, true);
        }
        Tool::BSpline => {
            // A ghost control polygon with round handles, the node on
            // the curve.
            pen.ghost("M3 19L7 4 15 20 21 5", a(0.4));
            pen.line("M5 11.5Q7 4 11 12Q15 20 18 12.5", ink);
            for (x, y) in [(3.0, 19.0), (7.0, 4.0), (15.0, 20.0), (21.0, 5.0)] {
                pen.handle(x, y, 1.8);
            }
            pen.node(11.0, 12.0, true);
        }
        Tool::Polyline => {
            pen.line("M3 19L7 6 13 15 20 4", ink);
            pen.node(3.0, 19.0, false);
            pen.node(7.0, 6.0, false);
            pen.node(13.0, 15.0, false);
            pen.node(20.0, 4.0, true);
        }
        Tool::ThreePointCurve => {
            // An arc with hollow ends and the live apex.
            pen.line("M3 18A9.5 9.5 0 0 1 21 18", ink);
            pen.node(3.0, 18.0, false);
            pen.node(21.0, 18.0, false);
            pen.node(12.0, 11.5, true);
        }

        // Drawing group.
        Tool::BrushStrokes => {
            // A calligraphic S swash: the accent ribbon on a thin ink
            // path, thick in the middle, tapered ends, the node at the
            // live end.
            pen.thin("M4.5 20C3 8 21 16 19.5 4", ink);
            let swash = ribbon(
                [pt(4.5, 20.0), pt(3.0, 8.0), pt(21.0, 16.0), pt(19.5, 4.0)],
                |t| 0.4 + 3.1 * (std::f32::consts::PI * t).sin().powf(0.9),
                36,
            );
            pen.fill_pts(&swash, acc, true);
            pen.node(19.5, 4.0, true);
        }
        Tool::PainterlyBrush => {
            // A round brush head and three dabs.
            pen.stroke("M20 4l-7.5 7.5", 3.0, ink);
            pen.fill(
                "M12.5 11.5c-1.5-.5-3.5.5-4 2.5s0 3.5 2 3 3.2-2.5 3.5-4z",
                ink,
                true,
            );
            pen.circle(6.0, 18.0, 2.6, acc, 0.0, acc);
            pen.circle(11.0, 20.0, 1.8, b(0.7), 0.0, acc);
            pen.circle(3.5, 13.0, 1.3, b(0.7), 0.0, acc);
        }
        Tool::ShapeRecognition => {
            // A faint sketched square recognised as a clean shape, the
            // node on its corner.
            pen.thin(
                "M4.5 5.5c3-1.2 7 1 10.5-.8 1.2 3-1 7 .8 10.5-3 1.2-7-1-10.5.8-1.2-3 1-7-.8-10.5z",
                a(0.5),
            );
            pen.rect(8.5, 8.5, 12.0, 12.0, 1.5, b(0.14), LINE, acc);
            pen.node(20.5, 20.5, true);
        }
        Tool::Sketch => {
            // Several sketchy strokes that read as one line, the live
            // end a node.
            pen.line("M3 17.5C8 9 14 9 21 6", a(0.4));
            pen.line("M3 15C8 7 14 7.5 21 4", a(0.4));
            pen.line("M3 16.3C8 8 14 8.2 21 5", ink);
            pen.node(21.0, 5.0, true);
        }

        // Rectangle group.
        Tool::Rectangle => {
            // A true square with right-angled corners, on integer units.
            pen.body(&rrect(4.0, 4.0, 16.0, 16.0, 0.0));
            pen.node(20.0, 4.0, true);
        }
        Tool::ThreePointRectangle => {
            // A tilted box: two hollow base points, the live third.
            let ang = (-24.0f32).to_radians();
            let d = Vec2::new(ang.cos(), ang.sin());
            let n = Vec2::new(-d.y, d.x);
            let p0 = pt(3.5, 15.5);
            let p1 = p0 + d * 14.0;
            let p2 = p1 + n * 8.0;
            let p3 = p0 + n * 8.0;
            let quad = [p0, p1, p2, p3];
            pen.fill_pts(&quad, pen.wash(), false);
            pen.stroke_pts(&quad, true, LINE, ink);
            pen.node(p0.x, p0.y, false);
            pen.node(p1.x, p1.y, false);
            pen.node(p2.x, p2.y, true);
        }

        // Ellipse group.
        Tool::Ellipse => {
            // The same 16 unit box as the square.
            pen.circle(12.0, 12.0, 8.0, pen.wash(), LINE, ink);
            let (x, y) = polar(12.0, 12.0, 8.0, -45.0);
            pen.node(x, y, true);
        }
        Tool::ThreePointEllipse => {
            // A tilted ellipse, the axis ends hollow, the width point live.
            let (rx, ry) = (9.0f32, 5.5f32);
            let oval: Vec<Pos2> = (0..48)
                .map(|i| {
                    let t = i as f32 / 48.0 * std::f32::consts::TAU;
                    pt(12.0 + rx * t.cos(), 12.0 + ry * t.sin())
                })
                .collect();
            let turned = pen.rotated(-30.0, 12.0, 12.0);
            turned.fill_pts(&oval, pen.wash(), false);
            turned.stroke_pts(&oval, true, LINE, ink);
            let (x, y) = polar(12.0, 12.0, 9.0, 150.0);
            pen.node(x, y, false);
            let (x, y) = polar(12.0, 12.0, 9.0, -30.0);
            pen.node(x, y, false);
            let (x, y) = polar(12.0, 12.0, 5.5, 60.0);
            pen.node(x, y, true);
        }

        // Polygon group.
        Tool::Polygon => {
            // A regular hexagon whose vertical sides sit on units 4 and
            // 20, the square's sides.
            let r = 8.0 / 30f32.to_radians().cos();
            let hex: Vec<Pos2> = [-90.0, -30.0, 30.0, 90.0, 150.0, 210.0]
                .iter()
                .map(|&a| {
                    let (x, y) = polar(12.0, 12.0, r, a);
                    pt(x, y)
                })
                .collect();
            pen.fill_pts(&hex, pen.wash(), false);
            pen.stroke_pts(&hex, true, LINE, ink);
            pen.node(hex[0].x, hex[0].y, true);
        }
        Tool::Star => {
            let s = star(12.0, 13.0, 9.5, 4.3, 5, -90.0);
            pen.fill_pts(&s, pen.wash(), false);
            pen.stroke_pts(&s, true, LINE, ink);
            pen.node(s[0].x, s[0].y, true);
        }
        Tool::Spiral => {
            let s = spiral(12.0, 12.5, 2.25, 0.7, 8.8, 80, -90.0);
            pen.stroke_pts(&s, false, LINE, ink);
            if let Some(end) = s.last() {
                pen.node(end.x, end.y, true);
            }
        }
        Tool::CommonShapes => {
            // An arrow shape with the node on its tip.
            pen.body("M3 8.5L12.5 8.5 12.5 3.5 21 12 12.5 20.5 12.5 15.5 3 15.5z");
            pen.node(21.0, 12.0, true);
        }
        Tool::ActionLines => {
            // A shape rushing right, speed lines trailing left.
            pen.circle(16.0, 12.0, 4.8, pen.wash(), LINE, ink);
            pen.line("M3 7.5h6.5M2 12h6M3 16.5h6.5", acc);
        }
        Tool::GraphPaper => {
            // A grid of cells, the node on its corner.
            pen.body(&rrect(3.0, 3.0, 18.0, 18.0, 2.0));
            pen.thin("M9 3v18M15 3v18M3 9h18M3 15h18", ink);
            pen.node(21.0, 21.0, true);
        }

        // Text group.
        Tool::Text => {
            // A bold T inside the dashed text frame.
            pen.dashed(&rrect(2.5, 2.5, 19.0, 19.0, 3.0), LINE, acc, 2.7, 2.2);
            pen.stroke("M7.6 7.5h8.8M12 7.5v10", 3.0, ink);
        }
        Tool::Table => {
            // A grid with an accent header row.
            pen.body(&rrect(3.0, 4.0, 18.0, 16.0, 2.0));
            pen.fill("M3 8.5V6a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v2.5z", acc, false);
            pen.thin("M3 8.5h18M3 14.25h18M9 8.5V20M15 8.5V20", ink);
        }

        // Dimension group.
        Tool::ParallelDimension => {
            // The measured edge in ink, the offset dimension line with
            // ticks in the accent.
            pen.line("M3 12L12 3", ink);
            pen.node(3.0, 12.0, false);
            pen.node(12.0, 3.0, false);
            pen.thin("M5.5 14.5l2.5 2.5M14.5 5.5l2.5 2.5", a(0.45));
            pen.line("M9 20L20 9", acc);
            pen.line("M7 18l4 4M18 7l4 4", acc);
        }
        Tool::HorizontalVerticalDimension => {
            // A box measured across the top and down the right side.
            pen.body(&rrect(4.0, 9.5, 10.5, 10.5, 1.5));
            pen.node(4.0, 9.5, false);
            pen.node(14.5, 20.0, false);
            pen.line("M4 5h10.5M4 3v4M14.5 3v4", acc);
            pen.line("M19 9.5V20M17 9.5h4M17 20h4", acc);
        }
        Tool::AngularDimension => {
            // Two rays and the arrowed arc.
            pen.line("M4 20h17M4 20L14.5 4.5", ink);
            pen.node(4.0, 20.0, false);
            pen.line("M15 20A11 11 0 0 0 10.2 10.9", acc);
            pen.arrow_head(15.6, 20.0, 1.0, 0.0, 2.8, acc);
            pen.arrow_head(9.85, 10.4, -0.55, -0.83, 2.8, acc);
        }
        Tool::SegmentDimension => {
            // An open path with hollow nodes, one segment measured.
            pen.line("M3 19L8 11h8l5 8", ink);
            pen.node(3.0, 19.0, false);
            pen.node(8.0, 11.0, false);
            pen.node(16.0, 11.0, false);
            pen.node(21.0, 19.0, false);
            pen.line("M8 5.5h8M8 3.5v4M16 3.5v4", acc);
        }
        Tool::Callout => {
            // A text box with a tail ending in a node.
            pen.body(&rrect(3.0, 3.0, 18.0, 11.5, 2.5));
            pen.thin("M7 7h10M7 10.5h6", ink);
            pen.line("M10 14.5l-4.5 5.5", ink);
            pen.node(5.5, 20.0, true);
        }

        // Connector group: two boxes joined by a link with round anchors.
        Tool::Connector | Tool::RightAngleConnector | Tool::RoundedConnector => {
            pen.body(&rrect(3.0, 3.0, 7.0, 7.0, 1.5));
            pen.body(&rrect(14.0, 14.0, 7.0, 7.0, 1.5));
            let link = match tool {
                Tool::Connector => "M10 6.5C15 6.5 9 17.5 14 17.5",
                Tool::RightAngleConnector => "M10 6.5h3v11h1",
                _ => "M10 6.5h.5a2.5 2.5 0 0 1 2.5 2.5v6a2.5 2.5 0 0 0 2.5 2.5h.5",
            };
            pen.line(link, acc);
            pen.handle(10.0, 6.5, 1.8);
            pen.handle(14.0, 17.5, 1.8);
        }
        Tool::AnchorEditing => {
            // A box with its anchors on the edges, the right one live
            // under the pointer.
            pen.body(&rrect(4.0, 4.0, 14.0, 14.0, 1.5));
            pen.node(11.0, 4.0, false);
            pen.node(4.0, 11.0, false);
            pen.node(11.0, 18.0, false);
            pen.node(18.0, 11.0, true);
            pen.pointer(17.4, 11.6, 0.58, false);
        }

        // Effects group.
        Tool::DropShadow => {
            // A square box with its solid shadow offset behind it, the
            // shadow in the accent.
            pen.fill(
                &rrect(8.0, 8.0, 12.0, 12.0, 0.0),
                Pen::alpha(acc, 0.6),
                true,
            );
            pen.solid(&rrect(4.0, 4.0, 12.0, 12.0, 0.0));
        }
        Tool::Contour => {
            // A triangle with two offset outlines fading outward.
            let base = [pt(12.0, 6.0), pt(19.0, 18.0), pt(5.0, 18.0)];
            let (cx, cy) = (12.0, 14.0);
            for (sc, op) in [(1.55, 0.35), (1.28, 0.6)] {
                let pts: Vec<Pos2> = base
                    .iter()
                    .map(|q| pt(cx + (q.x - cx) * sc, cy + (q.y - cy) * sc))
                    .collect();
                pen.stroke_pts(&pts, true, LINE, b(op));
            }
            pen.fill_pts(&base, pen.wash(), false);
            pen.stroke_pts(&base, true, LINE, ink);
        }
        Tool::Blend => {
            // A square morphs into an accent circle along a diagonal.
            pen.body(&rrect(3.0, 14.0, 7.0, 7.0, 0.5));
            pen.rect(7.8, 9.5, 6.5, 6.5, 1.6, Color32::TRANSPARENT, LINE, b(0.45));
            pen.rect(12.5, 5.0, 6.0, 6.0, 2.4, Color32::TRANSPARENT, LINE, b(0.7));
            pen.circle(18.5, 5.5, 2.8, acc, 0.0, acc);
        }
        Tool::Distort => {
            // A square with bowed sides, the corner node pulled.
            pen.body("M4 4c5 3 11 3 16 0c-3 5-3 11 0 16c-5-3-11-3-16 0c3-5 3-11 0-16z");
            pen.dashed("M20 4l-5 5", THIN, acc, 1.5, 1.5);
            pen.node(20.0, 4.0, true);
        }
        Tool::Envelope => {
            // A wavy box with four live corner nodes.
            pen.body("M4 5c3 2 5 2 8 0s5-2 8 0v14c-3-2-5-2-8 0s-5 2-8 0z");
            for (x, y) in [(4.0, 5.0), (20.0, 5.0), (4.0, 19.0), (20.0, 19.0)] {
                pen.node(x, y, true);
            }
        }
        Tool::Extrude => {
            // The front face opaque in ink, the depth faces in the accent.
            pen.fill("M3 9l6-6h12v12l-6 6z", b(0.3), false);
            pen.line("M3 9l6-6h12v12l-6 6z", acc);
            pen.line("M15 9l6-6", acc);
            pen.solid(&rrect(3.0, 9.0, 12.0, 12.0, 1.0));
        }
        Tool::BlockShadow => {
            // A circle with a solid accent block cast behind it.
            pen.stroke("M9.5 9.5L15 15", 12.0, acc);
            pen.circle(9.5, 9.5, 6.0, bg, LINE, ink);
        }

        Tool::Transparency => {
            // A circle filled with a fade from ink at the left to nothing
            // at the right, the node to handle gradient bar across it.
            pen.fade_circle(12.0, 12.0, 8.0, ink, Color32::TRANSPARENT);
            pen.circle(12.0, 12.0, 8.0, Color32::TRANSPARENT, LINE, ink);
            pen.thin("M6 18L18 6", acc);
            pen.node(6.0, 18.0, true);
            pen.handle(18.0, 6.0, 1.75);
        }

        // Eyedropper group: the diagonal pipette with the drop drawn
        // into its tip.
        Tool::ColorEyedropper | Tool::AttributesEyedropper => {
            if tool == Tool::ColorEyedropper {
                // The tip on a swatch.
                pen.rect(2.0, 15.5, 6.5, 6.5, 1.5, acc, 0.0, acc);
            }
            let t = pen.rotated(45.0, 12.0, 12.0);
            t.body(&rrect(9.5, 2.5, 5.0, 5.5, 2.2));
            t.line("M10 8.5h4", ink);
            t.solid("M10.6 9v6.5L12 19.5l1.4-4V9z");
            t.fill("M11.4 14v1.4l.6 2.2.6-2.2V14z", acc, false);
            if tool == Tool::AttributesEyedropper {
                // A property card.
                pen.rect(13.5, 13.5, 8.0, 8.0, 1.6, acc, 0.0, acc);
                pen.stroke("M15.5 16h4.2M15.5 18h2.8M15.5 20h3.6", 1.2, bg);
            }
        }

        // Fill group.
        Tool::InteractiveFill => {
            // A paint pail, tilted a little, the paint in it in the
            // accent and a drop leaving the lower lip.
            let t = pen.rotated(-15.0, 12.0, 14.0);
            t.fill("M6.6 13.5h10.8l-.8 6.5H7.4z", acc, false);
            t.body("M5 8h14l-1.5 12h-11z");
            t.thin("M8 8a4 4 0 0 1 8 0", ink);
            pen.circle(3.5, 14.0, 1.6, acc, 0.0, acc);
        }
        Tool::AreaFill => {
            // Two rings, the enclosed lens filled.
            pen.fill("M12 6.4A7 7 0 0 0 12 17.6A7 7 0 0 0 12 6.4z", acc, true);
            pen.circle(8.5, 12.0, 7.0, Color32::TRANSPARENT, LINE, ink);
            pen.circle(15.5, 12.0, 7.0, Color32::TRANSPARENT, LINE, ink);
        }
        Tool::MeshFill => {
            // A curved mesh with four small handles and the live node.
            pen.body(&rrect(3.0, 3.0, 18.0, 18.0, 2.5));
            pen.thin("M3 11c6 4 12-4 18 0M11 3c4 6-4 12 0 18", ink);
            pen.thin("M7 8.5l9 7M16 8.5l-9 7", acc);
            for (x, y) in [(7.0, 8.5), (16.0, 8.5), (7.0, 15.5), (16.0, 15.5)] {
                pen.handle(x, y, 1.5);
            }
            pen.node(11.5, 11.5, true);
        }

        // Outline group.
        Tool::OutlinePen => {
            // A ring with a thick accent outline segment ending in a node.
            pen.circle(12.0, 12.0, 8.0, pen.wash(), LINE, ink);
            let (x0, y0) = polar(12.0, 12.0, 8.0, -20.0);
            let (x1, y1) = polar(12.0, 12.0, 8.0, -150.0);
            pen.stroke(
                &format!("M{x0:.2} {y0:.2}A8 8 0 0 0 {x1:.2} {y1:.2}"),
                4.0,
                acc,
            );
            pen.node(x0, y0, true);
        }
        Tool::OutlineColor => {
            // A ring with a thick ink outline and the colour swatch.
            pen.circle(10.0, 10.0, 7.5, pen.wash(), 3.0, ink);
            pen.rect(13.5, 13.5, 8.0, 8.0, 1.6, acc, 0.0, acc);
        }
    }
}
/// Standard toolbar actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    New,
    Open,
    Save,
    Print,
    Cut,
    Copy,
    Paste,
    /// Delete the selection: a waste bin.
    Delete,
    /// Show or hide the dockers: a window with a panel at the right.
    Dockers,
    Undo,
    Redo,
    Import,
    Export,
    Pdf,
    /// Snapping off: the snap icon crossed out in red when on.
    SnapOff,
    Options,
    Welcome,
    Fullscreen,
    /// View toggles: a ruler, the grid and guidelines, each with an eye.
    ShowRulers,
    ShowGrid,
    ShowGuidelines,
    /// Launch other tools: a window.
    Launch,
    /// A cloud with an arrow down (open from the Git repository) or up
    /// (save to it as a commit).
    GitPull,
    GitPush,
}

/// The New button's green badge with a white plus.
const NEW_GREEN: Color32 = Color32::from_rgb(0x1E, 0x9E, 0x3C);

pub fn draw_action(painter: &Painter, r: Rect, a: Action, color: Color32) {
    let k = r.width() / 16.0;
    let s = Stroke::new(1.25 * k, color);
    let thin = Stroke::new(0.9 * k, color);
    let enabled = color == Tokens::icon();
    // A page with a folded top-right corner.
    let page = |painter: &Painter, x0: f32, y0: f32, x1: f32, y1: f32| {
        painter.add(epaint::PathShape::closed_line(
            vec![
                p(r, x0, y0),
                p(r, x1 - 3.0, y0),
                p(r, x1, y0 + 3.0),
                p(r, x1, y1),
                p(r, x0, y1),
            ],
            s,
        ));
        painter.add(epaint::PathShape::line(
            vec![
                p(r, x1 - 3.0, y0),
                p(r, x1 - 3.0, y0 + 3.0),
                p(r, x1, y0 + 3.0),
            ],
            thin,
        ));
    };
    // A small eye at the top right (view toggles).
    let eye = |painter: &Painter| {
        let c = p(r, 12.0, 3.0);
        painter.add(epaint::PathShape::closed_line(
            vec![
                c + Vec2::new(-3.2, 0.0) * k,
                c + Vec2::new(-1.2, -1.5) * k,
                c + Vec2::new(1.2, -1.5) * k,
                c + Vec2::new(3.2, 0.0) * k,
                c + Vec2::new(1.2, 1.5) * k,
                c + Vec2::new(-1.2, 1.5) * k,
            ],
            thin,
        ));
        painter.circle_filled(c, 0.9 * k, color);
    };
    match a {
        Action::New => {
            page(painter, 2.5, 2.5, 11.5, 15.0);
            let badge = Rect::from_min_max(p(r, 9.5, 0.5), p(r, 15.5, 6.5));
            painter.rect_filled(badge, 0.0, if enabled { NEW_GREEN } else { color });
            let c = badge.center();
            let w = Stroke::new(1.2 * k, Color32::WHITE);
            painter.line_segment(
                [c - Vec2::new(2.0 * k, 0.0), c + Vec2::new(2.0 * k, 0.0)],
                w,
            );
            painter.line_segment(
                [c - Vec2::new(0.0, 2.0 * k), c + Vec2::new(0.0, 2.0 * k)],
                w,
            );
        }
        Action::Open => {
            // Back of the folder, then its open front flap, filled dark.
            painter.add(epaint::PathShape::closed_line(
                vec![
                    p(r, 1.5, 3.0),
                    p(r, 6.0, 3.0),
                    p(r, 7.5, 4.5),
                    p(r, 13.0, 4.5),
                    p(r, 13.0, 7.0),
                ],
                s,
            ));
            painter.add(epaint::PathShape::convex_polygon(
                vec![
                    p(r, 3.5, 7.0),
                    p(r, 15.0, 7.0),
                    p(r, 12.5, 14.0),
                    p(r, 1.5, 14.0),
                ],
                Color32::TRANSPARENT,
                s,
            ));
            painter.line_segment([p(r, 1.5, 3.0), p(r, 1.5, 14.0)], s);
        }
        Action::Save => {
            // A solid floppy disk with a white label and shutter.
            painter.add(epaint::PathShape::convex_polygon(
                vec![
                    p(r, 1.5, 1.5),
                    p(r, 12.5, 1.5),
                    p(r, 14.5, 3.5),
                    p(r, 14.5, 14.5),
                    p(r, 1.5, 14.5),
                ],
                color,
                Stroke::NONE,
            ));
            painter.rect_filled(
                Rect::from_min_max(p(r, 4.0, 1.5), p(r, 11.0, 5.5)),
                0.0,
                Color32::WHITE,
            );
            painter.rect_filled(
                Rect::from_min_max(p(r, 8.5, 2.2), p(r, 10.0, 4.8)),
                0.0,
                color,
            );
            painter.rect_filled(
                Rect::from_min_max(p(r, 4.0, 9.0), p(r, 12.0, 14.5)),
                0.0,
                Color32::WHITE,
            );
        }
        Action::Print => {
            // Paper rising out of a tray.
            painter.add(epaint::PathShape::line(
                vec![
                    p(r, 4.5, 9.0),
                    p(r, 4.5, 1.5),
                    p(r, 11.5, 1.5),
                    p(r, 11.5, 9.0),
                ],
                s,
            ));
            painter.add(epaint::PathShape::closed_line(
                vec![
                    p(r, 1.5, 9.0),
                    p(r, 14.5, 9.0),
                    p(r, 14.5, 14.5),
                    p(r, 1.5, 14.5),
                ],
                s,
            ));
            painter.line_segment([p(r, 1.5, 12.0), p(r, 14.5, 12.0)], thin);
        }
        Action::Cut | Action::Copy | Action::Paste => {
            // Pages with text lines: Cut leaves a dashed page, Copy shows
            // two pages, Paste a dark clipboard behind a page.
            let lines = |painter: &Painter, x0: f32, y0: f32, x1: f32| {
                for i in 0..3 {
                    let y = y0 + i as f32 * 2.2;
                    painter.line_segment([p(r, x0, y), p(r, x1, y)], thin);
                }
            };
            match a {
                Action::Cut => {
                    page(painter, 1.5, 1.5, 9.5, 11.5);
                    lines(painter, 3.5, 4.5, 7.5);
                    for (x, y) in [
                        (10.5, 11.0),
                        (12.5, 11.0),
                        (14.5, 11.0),
                        (14.5, 13.0),
                        (14.5, 15.0),
                        (12.5, 15.0),
                        (10.5, 15.0),
                        (10.5, 13.0),
                    ] {
                        painter.rect_filled(
                            Rect::from_center_size(p(r, x, y), Vec2::splat(0.9 * k)),
                            0.0,
                            color,
                        );
                    }
                }
                Action::Copy => {
                    page(painter, 1.5, 1.5, 9.5, 11.0);
                    lines(painter, 3.5, 4.5, 7.5);
                    painter.rect_filled(
                        Rect::from_min_max(p(r, 6.0, 5.5), p(r, 14.5, 15.0)),
                        0.0,
                        Color32::WHITE,
                    );
                    page(painter, 6.0, 5.5, 14.5, 15.0);
                    lines(painter, 8.0, 9.0, 12.5);
                }
                _ => {
                    painter.rect_filled(
                        Rect::from_min_max(p(r, 1.5, 2.0), p(r, 9.5, 14.0)),
                        k,
                        color,
                    );
                    painter.rect_filled(
                        Rect::from_min_max(p(r, 7.0, 6.0), p(r, 14.5, 15.0)),
                        0.0,
                        Color32::WHITE,
                    );
                    page(painter, 7.0, 6.0, 14.5, 15.0);
                    lines(painter, 9.0, 9.0, 12.5);
                }
            }
        }
        Action::Undo | Action::Redo => {
            // A dashed circular arrow.
            let flip = a == Action::Redo;
            let fx = |x: f32| if flip { 16.0 - x } else { x };
            let c = (8.0, 8.5);
            let rad = 5.5;
            let n = 16;
            for i in 0..n {
                if i % 3 == 2 {
                    continue;
                }
                let t0 = std::f32::consts::PI * (1.05 - 1.5 * i as f32 / n as f32);
                let t1 = std::f32::consts::PI * (1.05 - 1.5 * (i + 1) as f32 / n as f32);
                painter.line_segment(
                    [
                        p(r, fx(c.0 + rad * t0.cos()), c.1 - rad * t0.sin()),
                        p(r, fx(c.0 + rad * t1.cos()), c.1 - rad * t1.sin()),
                    ],
                    s,
                );
            }
            painter.add(epaint::PathShape::convex_polygon(
                vec![p(r, fx(0.5), 6.0), p(r, fx(5.0), 5.0), p(r, fx(3.5), 9.5)],
                color,
                Stroke::NONE,
            ));
        }
        Action::Import | Action::Export => {
            // A box with an arrow through its top.
            painter.add(epaint::PathShape::line(
                vec![
                    p(r, 5.0, 2.0),
                    p(r, 2.0, 2.0),
                    p(r, 2.0, 15.0),
                    p(r, 14.0, 15.0),
                    p(r, 14.0, 2.0),
                    p(r, 11.0, 2.0),
                ],
                s,
            ));
            let up = a == Action::Export;
            let (y0, y1) = if up { (12.0, 2.5) } else { (1.0, 11.5) };
            let w = Stroke::new(1.8 * k, color);
            painter.line_segment([p(r, 8.0, y0), p(r, 8.0, y1)], w);
            let d = if up { 1.0 } else { -1.0 };
            painter.add(epaint::PathShape::convex_polygon(
                vec![
                    p(r, 8.0, y1 - d * 0.5),
                    p(r, 5.0, y1 + d * 3.5),
                    p(r, 11.0, y1 + d * 3.5),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Action::Pdf => {
            painter.text(
                p(r, 8.0, 5.0),
                egui::Align2::CENTER_CENTER,
                "PDF",
                egui::FontId::proportional(r.width() * 0.42),
                color,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(r, 4.0, 10.0), p(r, 8.5, 14.5)),
                0.0,
                thin,
                epaint::StrokeKind::Middle,
            );
            painter.rect_filled(
                Rect::from_min_max(p(r, 5.0, 12.0), p(r, 7.5, 14.5)),
                0.0,
                color,
            );
        }
        Action::Fullscreen => {
            // A dark screen inside corner brackets.
            painter.rect_filled(
                Rect::from_min_max(p(r, 3.5, 3.5), p(r, 12.5, 12.5)),
                0.0,
                color,
            );
            for (x, y, dx, dy) in [
                (1.0, 1.0, 1.0, 1.0),
                (15.0, 1.0, -1.0, 1.0),
                (1.0, 15.0, 1.0, -1.0),
                (15.0, 15.0, -1.0, -1.0),
            ] {
                painter.add(epaint::PathShape::line(
                    vec![p(r, x + 3.0 * dx, y), p(r, x, y), p(r, x, y + 3.0 * dy)],
                    thin,
                ));
            }
        }
        Action::SnapOff => {
            // A box with an arrow pointing into its corner.
            painter.add(epaint::PathShape::closed_line(
                vec![
                    p(r, 2.0, 3.0),
                    p(r, 12.0, 3.0),
                    p(r, 12.0, 14.0),
                    p(r, 2.0, 14.0),
                ],
                s,
            ));
            painter.add(epaint::PathShape::convex_polygon(
                vec![p(r, 4.0, 9.0), p(r, 9.0, 5.5), p(r, 9.0, 12.5)],
                Color32::TRANSPARENT,
                s,
            ));
            let red = Stroke::new(1.4 * k, Color32::from_rgb(0xE8, 0x11, 0x23));
            painter.line_segment([p(r, 10.5, 0.5), p(r, 15.5, 5.5)], red);
            painter.line_segment([p(r, 15.5, 0.5), p(r, 10.5, 5.5)], red);
        }
        Action::Options => {
            // A filled gear.
            let c = p(r, 8.0, 8.0);
            for i in 0..8 {
                let ang = i as f32 * std::f32::consts::TAU / 8.0;
                painter.line_segment(
                    [
                        c + Vec2::new(ang.cos(), ang.sin()) * 4.0 * k,
                        c + Vec2::new(ang.cos(), ang.sin()) * 7.0 * k,
                    ],
                    Stroke::new(2.6 * k, color),
                );
            }
            painter.circle_filled(c, 5.0 * k, color);
            painter.circle_filled(c, 2.0 * k, Tokens::panel());
        }
        Action::Welcome => {
            for y in [4.0, 8.0, 12.0] {
                painter.line_segment([p(r, 2.5, y), p(r, 13.5, y)], Stroke::new(1.8 * k, color));
            }
        }
        Action::ShowRulers => {
            // An L-shaped ruler with ticks.
            painter.add(epaint::PathShape::closed_line(
                vec![
                    p(r, 1.5, 1.5),
                    p(r, 5.0, 1.5),
                    p(r, 5.0, 11.0),
                    p(r, 14.5, 11.0),
                    p(r, 14.5, 14.5),
                    p(r, 1.5, 14.5),
                ],
                s,
            ));
            for i in 0..4 {
                let y = 3.5 + i as f32 * 2.0;
                painter.line_segment([p(r, 1.5, y), p(r, 3.2, y)], thin);
                let x = 7.0 + i as f32 * 2.0;
                painter.line_segment([p(r, x, 14.5), p(r, x, 12.8)], thin);
            }
            eye(painter);
        }
        Action::ShowGrid => {
            for i in 0..5 {
                let v = 1.5 + i as f32 * 3.2;
                painter.line_segment([p(r, 1.5, v.max(6.0)), p(r, 14.5, v.max(6.0))], thin);
                painter.line_segment([p(r, v, 6.0), p(r, v, 14.5)], thin);
            }
            painter.line_segment([p(r, 1.5, 14.5), p(r, 14.5, 14.5)], thin);
            eye(painter);
        }
        Action::ShowGuidelines => {
            // A dashed cross.
            for i in 0..6 {
                let t = 1.5 + i as f32 * 2.3;
                painter.line_segment([p(r, t, 10.0), p(r, t + 1.2, 10.0)], thin);
                painter.line_segment([p(r, 6.5, t + 2.0), p(r, 6.5, t + 3.0)], thin);
            }
            eye(painter);
        }
        Action::GitPull | Action::GitPush => {
            // A cloud: a flat base and three bumps.
            painter.add(epaint::PathShape::line(
                vec![
                    p(r, 4.5, 11.5),
                    p(r, 2.6, 11.0),
                    p(r, 1.5, 9.4),
                    p(r, 2.0, 7.6),
                    p(r, 3.6, 6.8),
                    p(r, 4.4, 4.6),
                    p(r, 6.6, 3.2),
                    p(r, 9.2, 3.4),
                    p(r, 11.0, 5.2),
                    p(r, 13.2, 5.6),
                    p(r, 14.5, 7.6),
                    p(r, 14.2, 10.0),
                    p(r, 12.6, 11.4),
                    p(r, 11.5, 11.5),
                ],
                s,
            ));
            let (tip, tail) = if a == Action::GitPull {
                (14.6, 8.0)
            } else {
                (8.0, 14.6)
            };
            painter.line_segment([p(r, 8.0, tail), p(r, 8.0, tip)], s);
            let dir = if a == Action::GitPull { -1.0 } else { 1.0 };
            painter.add(epaint::PathShape::line(
                vec![
                    p(r, 5.8, tip + 2.2 * dir),
                    p(r, 8.0, tip),
                    p(r, 10.2, tip + 2.2 * dir),
                ],
                s,
            ));
        }
        Action::Delete => {
            // A waste bin: the lid with its handle, the tapered body and
            // three slats.
            painter.line_segment([p(r, 2.0, 4.0), p(r, 14.0, 4.0)], s);
            painter.add(epaint::PathShape::line(
                vec![
                    p(r, 6.0, 4.0),
                    p(r, 6.5, 2.0),
                    p(r, 9.5, 2.0),
                    p(r, 10.0, 4.0),
                ],
                thin,
            ));
            painter.add(epaint::PathShape::closed_line(
                vec![
                    p(r, 3.5, 4.0),
                    p(r, 12.5, 4.0),
                    p(r, 11.5, 14.5),
                    p(r, 4.5, 14.5),
                ],
                s,
            ));
            for (x0, x1) in [(6.0, 6.4), (8.0, 8.0), (10.0, 9.6)] {
                painter.line_segment([p(r, x0, 6.5), p(r, x1, 12.5)], thin);
            }
        }
        Action::Dockers => {
            painter.rect_stroke(
                Rect::from_min_max(p(r, 1.5, 2.0), p(r, 14.5, 14.5)),
                0.0,
                s,
                epaint::StrokeKind::Middle,
            );
            painter.rect_filled(
                Rect::from_min_max(p(r, 9.5, 2.0), p(r, 14.5, 14.5)),
                0.0,
                color,
            );
            for y in [5.0, 8.0, 11.0] {
                painter.line_segment([p(r, 3.5, y), p(r, 7.5, y)], thin);
            }
        }
        Action::Launch => {
            painter.rect_stroke(
                Rect::from_min_max(p(r, 1.5, 2.0), p(r, 14.5, 14.5)),
                0.0,
                s,
                epaint::StrokeKind::Middle,
            );
            painter.line_segment([p(r, 1.5, 4.5), p(r, 14.5, 4.5)], s);
            for (y, x1) in [(7.5, 10.0), (9.5, 8.0), (11.5, 9.0)] {
                painter.line_segment([p(r, 4.0, y), p(r, x1, y)], thin);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coloured_icons_follow_the_toolbox_groups_and_can_be_off() {
        let plain = Color32::from_rgb(1, 2, 3);
        crate::theme::set_theme(crate::theme::Theme::Light);
        set_colored(false);
        assert_eq!(tool_color(Tool::Crop, plain), plain);
        set_colored(true);
        assert_eq!(
            tool_color(Tool::Knife, plain),
            tool_color(Tool::Crop, plain)
        );
        assert_ne!(tool_color(Tool::Pick, plain), tool_color(Tool::Crop, plain));
        assert_eq!(crate::tools::GROUPS.len(), 18);
        // Each group keeps its colour: the Variable Outline button, under
        // the brush tools, shares theirs; the groups after it keep the
        // hues they had.
        assert_eq!(
            tool_color(Tool::VariableOutline, plain),
            tool_color(Tool::BrushStrokes, plain)
        );
        assert_ne!(
            tool_color(Tool::VariableOutline, plain),
            tool_color(Tool::Rectangle, plain)
        );
        assert_eq!(
            tool_color(Tool::Rectangle, plain),
            tool_color(Tool::Polygon, plain)
        );
        assert_ne!(
            tool_color(Tool::Text, plain),
            tool_color(Tool::Polygon, plain)
        );
        assert_ne!(
            tool_color(Tool::InteractiveFill, plain),
            tool_color(Tool::OutlinePen, plain)
        );
        set_colored(false);
    }

    /// The shapes `draw` adds for `tool` in a 22 px box.
    fn shapes_of(tool: Tool, color: Color32) -> Vec<egui::Shape> {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(40.0))),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| {
            let r = Rect::from_min_size(Pos2::new(4.0, 4.0), Vec2::splat(22.0));
            draw(ui.painter(), r, tool, color);
        });
        out.textures_delta.clear();
        out.shapes.into_iter().map(|s| s.shape).collect()
    }

    /// The colours a shape is drawn in.
    fn colors(s: &egui::Shape, out: &mut Vec<Color32>) {
        match s {
            egui::Shape::Path(p) => {
                out.push(p.fill);
                if let epaint::ColorMode::Solid(c) = p.stroke.color {
                    out.push(c);
                }
            }
            egui::Shape::Circle(c) => {
                out.push(c.fill);
                out.push(c.stroke.color);
            }
            egui::Shape::Rect(r) => {
                out.push(r.fill);
                out.push(r.stroke.color);
            }
            egui::Shape::LineSegment { stroke, .. } => out.push(stroke.color),
            egui::Shape::Mesh(m) => out.extend(m.vertices.iter().map(|v| v.color)),
            egui::Shape::Vec(v) => v.iter().for_each(|s| colors(s, out)),
            _ => {}
        }
    }

    #[test]
    fn every_tool_icon_draws_in_its_box() {
        crate::theme::set_theme(crate::theme::Theme::Light);
        for tool in Tool::ALL {
            let shapes = shapes_of(tool, Tokens::icon());
            assert!(shapes.len() > 1, "{tool:?} draws nothing");
            let bounds = shapes
                .iter()
                .map(|s| s.visual_bounding_rect())
                .filter(|b| b.is_positive())
                .reduce(|a, b| a.union(b))
                .expect("something visible");
            // Inside the box, a stroke's width of slack.
            let box_ = Rect::from_min_size(Pos2::new(4.0, 4.0), Vec2::splat(22.0)).expand(2.5);
            assert!(box_.contains_rect(bounds), "{tool:?} spills: {bounds:?}");
            assert!(
                bounds.width() > 8.0 && bounds.height() > 8.0,
                "{tool:?} too small: {bounds:?}"
            );
        }
    }

    #[test]
    fn the_accent_marks_the_live_point_and_dimmed_icons_stay_plain() {
        crate::theme::set_theme(crate::theme::Theme::Light);
        let mut cs = Vec::new();
        shapes_of(Tool::Rectangle, Tokens::icon())
            .iter()
            .for_each(|s| colors(s, &mut cs));
        assert!(cs.contains(&accent()), "no accent in the Rectangle icon");
        assert!(cs.contains(&Tokens::icon()), "no ink in the Rectangle icon");
        let mut cs = Vec::new();
        shapes_of(Tool::Rectangle, Tokens::icon_off())
            .iter()
            .for_each(|s| colors(s, &mut cs));
        assert!(!cs.contains(&accent()), "a dimmed icon carries the accent");
        // The accent follows the theme: lighter on dark panels.
        let light = accent();
        crate::theme::set_theme(crate::theme::Theme::Dark);
        assert_ne!(accent(), light);
        assert!(accent().r() > light.r());
        crate::theme::set_theme(crate::theme::Theme::Light);
    }
}
