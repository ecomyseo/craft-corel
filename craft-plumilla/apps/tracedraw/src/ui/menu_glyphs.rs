//! Pictures for the rows of the File, Edit, Layout, Object, Effects,
//! Table, Tools and Help menus, drawn on a 16 by 16 grid in the icon
//! colour with a few accents (a red cross, a green plus, a blue mark).

use crate::theme::Tokens;
use egui::{epaint, Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    // Help
    Help,
    Video,
    QuickStart,
    UserGuide,
    WhatsNew,
    Updates,
    MessageSettings,
    Community,
    Info,
    // File
    SaveAs,
    CloseDoc,
    CloseAllDocs,
    Revert,
    PrintPreview,
    Preflight,
    DocProperties,
    Exit,
    Mail,
    Acquire,
    FromTemplate,
    // Edit
    Delete,
    Duplicate,
    Clone,
    SelectAll,
    Find,
    StepRepeat,
    CopyProperties,
    PasteSpecial,
    Repeat,
    // Layout
    PageInsert,
    PageInsertAfter,
    PageInsertBefore,
    PageDelete,
    PageRename,
    PageGoto,
    PageDuplicate,
    PageOrientation,
    PageNumber,
    DocOptions,
    PageBackground,
    PageSize,
    // Object
    AlignLeft,
    AlignRight,
    AlignTop,
    AlignBottom,
    AlignCenterH,
    AlignCenterV,
    CenterPage,
    DistLeft,
    DistCenterH,
    DistSpaceH,
    DistRight,
    DistTop,
    DistCenterV,
    DistSpaceV,
    DistBottom,
    Forward,
    Backward,
    InFront,
    Behind,
    ReverseOrder,
    Hide,
    ShowAll,
    Unlock,
    UnlockAll,
    OutlineToObject,
    JoinCurves,
    Perspective,
    ClearTransform,
    Properties,
    ObjectsDocker,
    ClipInside,
    ClipExtract,
    ClipEdit,
    ClipFinish,
    Barcode,
    QrCode,
    Symmetry,
    Symbol,
    // Effects
    Blend,
    Contour,
    Extrude,
    Lens,
    Bevel,
    BrushStrokes,
    Mosaic,
    Flatten,
    // Table
    TableNew,
    TextToTable,
    TableToText,
    RowAbove,
    RowBelow,
    ColLeft,
    ColRight,
    DelRow,
    DelCol,
    DelTable,
    MergeCells,
    SplitCells,
    DistRows,
    DistCols,
    // Tools
    ColorManagement,
    FontManager,
    TraceCut,
    Ai,
    Clock,
    Script,
    Record,
    Grommet,
    SaveDefaults,
}

const RED: Color32 = Color32::from_rgb(0xD0, 0x2B, 0x2B);
const GREEN: Color32 = Color32::from_rgb(0x1E, 0x9E, 0x3C);
const GOLD: Color32 = Color32::from_rgb(0xE8, 0xA3, 0x17);

/// The picture of a menu row, by its key.
pub fn for_key(key: &str) -> Option<Glyph> {
    use Glyph::*;
    Some(match key {
        "menu.help.product_help" | "menu.help.help_topics" => Help,
        "menu.help.video_tutorials" => Video,
        "menu.help.quick_start_guide" => QuickStart,
        "menu.help.user_guide" => UserGuide,
        "menu.help.whats_new" => WhatsNew,
        "menu.help.updates" => Updates,
        "menu.help.message_settings" => MessageSettings,
        "menu.help.community" => Community,
        "menu.help.support" | "menu.help.about" => Info,
        "menu.file.save_as" | "menu.file.save_as_template" => SaveAs,
        "menu.file.close" => CloseDoc,
        "menu.file.close_all" => CloseAllDocs,
        "menu.file.revert" => Revert,
        "menu.file.print_preview" => PrintPreview,
        "menu.file.preflight" => Preflight,
        "menu.file.document_properties" => DocProperties,
        "menu.file.exit" => Exit,
        "menu.file.send_to_mail" => Mail,
        "menu.file.new_from_template" => FromTemplate,
        "menu.file.acquire_image" => Acquire,
        "menu.edit.delete" => Delete,
        "menu.edit.duplicate" => Duplicate,
        "menu.edit.clone" => Clone,
        "menu.edit.select_all_objects" => SelectAll,
        "menu.edit.find_and_replace" => Find,
        "menu.edit.step_and_repeat" => StepRepeat,
        "menu.edit.copy_properties_from" => CopyProperties,
        "menu.edit.paste_special" | "menu.edit.paste_in_view" => PasteSpecial,
        "menu.edit.repeat" => Repeat,
        "menu.layout.insert_page" => PageInsert,
        "menu.layout.delete_page" | "page_tab.delete" => PageDelete,
        "page_tab.insert_after" => PageInsertAfter,
        "page_tab.insert_before" => PageInsertBefore,
        "menu.layout.rename_page" => PageRename,
        "menu.layout.go_to_page" => PageGoto,
        "menu.layout.duplicate_page" => PageDuplicate,
        "menu.layout.switch_orientation" => PageOrientation,
        "menu.layout.page_number_settings" | "menu.object.insert_page_number" => PageNumber,
        "menu.layout.document_options" | "menu.layout.page_layout" => DocOptions,
        "menu.layout.page_background" => PageBackground,
        "menu.layout.page_size" => PageSize,
        "menu.object.align_left" | "menu.object.align_distribute" => AlignLeft,
        "menu.object.align_right" => AlignRight,
        "menu.object.align_top" => AlignTop,
        "menu.object.align_bottom" => AlignBottom,
        "menu.object.align_centers_h" => AlignCenterH,
        "menu.object.align_centers_v" => AlignCenterV,
        "menu.object.center_to_page" => CenterPage,
        "menu.object.distribute_left" => DistLeft,
        "menu.object.distribute_centers_h" => DistCenterH,
        "menu.object.distribute_spacing_h" => DistSpaceH,
        "menu.object.distribute_right" => DistRight,
        "menu.object.distribute_top" => DistTop,
        "menu.object.distribute_centers_v" => DistCenterV,
        "menu.object.distribute_spacing_v" => DistSpaceV,
        "menu.object.distribute_bottom" => DistBottom,
        "menu.object.forward_one" | "menu.object.order" => Forward,
        "menu.object.back_one" => Backward,
        "menu.object.in_front_of" => InFront,
        "menu.object.behind" => Behind,
        "menu.object.reverse_order" => ReverseOrder,
        "menu.object.hide_object" | "menu.object.hide" => Hide,
        "menu.object.show_all" => ShowAll,
        "menu.object.unlock_object" => Unlock,
        "menu.object.unlock_all" => UnlockAll,
        "menu.object.convert_outline_to_object" => OutlineToObject,
        "menu.object.join_curves" => JoinCurves,
        "menu.object.add_perspective" => Perspective,
        "menu.object.clear_transformations" => ClearTransform,
        "menu.object.properties" => Properties,
        "menu.object.objects" => ObjectsDocker,
        "menu.object.clip_frame_place_inside" | "menu.object.clip_frame" => ClipInside,
        "menu.object.clip_frame_extract" => ClipExtract,
        "menu.object.clip_frame_edit" => ClipEdit,
        "menu.object.clip_frame_finish" => ClipFinish,
        "menu.object.insert_barcode" | "menu.object.insert" => Barcode,
        "menu.object.insert_qr" => QrCode,
        "menu.object.symmetry_create" | "menu.object.symmetry_edit" | "menu.object.symmetry" => {
            Symmetry
        }
        "menu.object.symbol_new"
        | "menu.object.symbol_manager"
        | "menu.object.create_symbol"
        | "menu.object.symbol" => Symbol,
        "menu.effects.blend" => Blend,
        "menu.effects.contour" => Contour,
        "menu.effects.extrude" => Extrude,
        "menu.effects.lens" => Lens,
        "menu.effects.bevel" => Bevel,
        "menu.effects.brush_strokes" => BrushStrokes,
        "menu.effects.picture_mosaic" | "menu.effects.vector_mosaic" => Mosaic,
        "menu.effects.flatten" => Flatten,
        "menu.table.create_new_table" | "menu.object.create_table" => TableNew,
        "menu.table.convert_text_to_table" => TextToTable,
        "menu.table.convert_table_to_text" => TableToText,
        "menu.table.insert_row_above" => RowAbove,
        "menu.table.insert_row_below" => RowBelow,
        "menu.table.insert_column_left" => ColLeft,
        "menu.table.insert_column_right" => ColRight,
        "menu.table.delete_row" => DelRow,
        "menu.table.delete_column" => DelCol,
        "menu.table.delete_table" => DelTable,
        "menu.table.merge_cells" => MergeCells,
        "menu.table.unmerge_cells"
        | "menu.table.split_into_rows"
        | "menu.table.split_into_columns" => SplitCells,
        "menu.table.distribute_rows" => DistRows,
        "menu.table.distribute_columns" => DistCols,
        "menu.tools.color_management" => ColorManagement,
        "menu.tools.font_manager" => FontManager,
        "menu.tools.tracecut" => TraceCut,
        "menu.tools.ai_drawing" | "menu.object.ai_draw_words" => Ai,
        "menu.tools.ai_generate"
        | "menu.tools.options_ai"
        | "menu.bitmaps.remove_background"
        | "context.ai_remix"
        | "menu.object.ai_generate"
        | "menu.object.ai_create_image"
        | "menu.object.ai_remix_image"
        | "menu.object.ai_create_vector" => Ai,
        "menu.tools.time_tracker" => Clock,
        "menu.tools.scripts_run" | "menu.tools.scripts_docker" => Script,
        "menu.tools.scripts_record" => Record,
        "menu.tools.border_and_grommet" => Grommet,
        "menu.tools.save_settings_as_default" => SaveDefaults,
        _ => return None,
    })
}

fn q(r: Rect, x: f32, y: f32) -> Pos2 {
    r.min + Vec2::new(x, y) * (r.width() / 16.0)
}

pub fn draw(painter: &Painter, r: Rect, g: Glyph, color: Color32) {
    use Glyph::*;
    let k = r.width() / 16.0;
    let s = Stroke::new(1.1 * k, color);
    let thin = Stroke::new(0.85 * k, color);
    let bold = Stroke::new(1.6 * k, color);
    let enabled = color == Tokens::icon();
    let accent = |c: Color32| if enabled { c } else { color };
    let paper = Tokens::field();
    let line = |a: (f32, f32), b: (f32, f32), st: Stroke| {
        painter.line_segment([q(r, a.0, a.1), q(r, b.0, b.1)], st);
    };
    let poly = |pts: &[(f32, f32)], st: Stroke| {
        painter.add(epaint::PathShape::line(
            pts.iter().map(|(x, y)| q(r, *x, *y)).collect(),
            st,
        ));
    };
    let closed = |pts: &[(f32, f32)], fill: Color32, st: Stroke| {
        painter.add(epaint::PathShape::convex_polygon(
            pts.iter().map(|(x, y)| q(r, *x, *y)).collect(),
            fill,
            st,
        ));
    };
    let boxr = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(q(r, x0, y0), q(r, x1, y1));
    let rect = |x0: f32, y0: f32, x1: f32, y1: f32, fill: Color32, st: Stroke| {
        painter.rect(
            boxr(x0, y0, x1, y1),
            0.0,
            fill,
            st,
            egui::StrokeKind::Middle,
        );
    };
    let fillr = |x0: f32, y0: f32, x1: f32, y1: f32, fill: Color32| {
        painter.rect_filled(boxr(x0, y0, x1, y1), 0.0, fill);
    };
    let circle = |x: f32, y: f32, rad: f32, fill: Color32, st: Stroke| {
        painter.circle(q(r, x, y), rad * k, fill, st);
    };
    let text = |x: f32, y: f32, t: &str, size: f32, c: Color32| {
        painter.text(
            q(r, x, y),
            Align2::CENTER_CENTER,
            t,
            FontId::proportional(size * k),
            c,
        );
    };
    // A page with a folded corner.
    let page = |x0: f32, y0: f32, x1: f32, y1: f32| {
        closed(
            &[(x0, y0), (x1 - 3.0, y0), (x1, y0 + 3.0), (x1, y1), (x0, y1)],
            paper,
            s,
        );
        poly(
            &[(x1 - 3.0, y0), (x1 - 3.0, y0 + 3.0), (x1, y0 + 3.0)],
            thin,
        );
    };
    let cross = |cx: f32, cy: f32, h: f32| {
        let st = Stroke::new(1.6 * k, accent(RED));
        line((cx - h, cy - h), (cx + h, cy + h), st);
        line((cx - h, cy + h), (cx + h, cy - h), st);
    };
    // The accent of the pages: the icons' blue (#1682).
    let blue = accent(crate::ui::icons::accent());
    let plus = |cx: f32, cy: f32, h: f32| {
        let st = Stroke::new(1.6 * k, blue);
        line((cx - h, cy), (cx + h, cy), st);
        line((cx, cy - h), (cx, cy + h), st);
    };
    let arrow = |a: (f32, f32), b: (f32, f32), st: Stroke| {
        line(a, b, st);
        let d = Vec2::new(b.0 - a.0, b.1 - a.1).normalized() * 2.6;
        let n = Vec2::new(-d.y, d.x) * 0.8;
        closed(
            &[
                b,
                (b.0 - d.x + n.x, b.1 - d.y + n.y),
                (b.0 - d.x - n.x, b.1 - d.y - n.y),
            ],
            st.color,
            Stroke::NONE,
        );
    };
    let gear = |cx: f32, cy: f32, rad: f32| {
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::TAU / 8.0;
            let d = (a.cos(), a.sin());
            line(
                (cx + d.0 * rad * 0.6, cy + d.1 * rad * 0.6),
                (cx + d.0 * rad, cy + d.1 * rad),
                Stroke::new(1.6 * k, color),
            );
        }
        circle(cx, cy, rad * 0.65, paper, s);
        circle(cx, cy, rad * 0.25, color, Stroke::NONE);
    };
    let book = |inner: &str| {
        closed(
            &[(2.5, 2.0), (12.5, 2.0), (12.5, 14.0), (2.5, 14.0)],
            paper,
            s,
        );
        line((4.5, 2.0), (4.5, 14.0), thin);
        text(8.5, 8.0, inner, 7.0, color);
    };
    let eye = |cx: f32, cy: f32, open: bool| {
        let pts: Vec<(f32, f32)> = (0..=12)
            .map(|i| {
                let t = i as f32 / 12.0 * std::f32::consts::PI;
                (cx - 6.0 * t.cos(), cy - 3.5 * t.sin())
            })
            .chain((0..=12).map(|i| {
                let t = i as f32 / 12.0 * std::f32::consts::PI;
                (cx + 6.0 * t.cos(), cy + 3.5 * t.sin())
            }))
            .collect();
        poly(&pts, s);
        circle(cx, cy, 1.8, color, Stroke::NONE);
        if !open {
            line(
                (cx - 6.0, cy + 5.0),
                (cx + 6.0, cy - 5.0),
                Stroke::new(1.4 * k, accent(RED)),
            );
        }
    };
    let lock = |open: bool| {
        fillr(4.0, 8.0, 12.0, 14.5, color);
        let lift = if open { 2.5 } else { 0.0 };
        poly(
            &[
                (5.5, 8.0 - lift * 0.0),
                (5.5, 5.0 - lift),
                (6.5, 3.0 - lift),
                (9.5, 3.0 - lift),
                (10.5, 5.0 - lift),
                (10.5, if open { 4.0 - lift } else { 8.0 }),
            ],
            bold,
        );
    };
    // Two blocks for align and distribute pictures.
    let blk = |x0: f32, y0: f32, x1: f32, y1: f32, c: Color32| {
        fillr(x0, y0, x1, y1, c);
    };

    let grid = |x0: f32, y0: f32, x1: f32, y1: f32, rows: usize, cols: usize| {
        rect(x0, y0, x1, y1, paper, s);
        for i in 1..rows {
            let y = y0 + (y1 - y0) * i as f32 / rows as f32;
            line((x0, y), (x1, y), thin);
        }
        for i in 1..cols {
            let x = x0 + (x1 - x0) * i as f32 / cols as f32;
            line((x, y0), (x, y1), thin);
        }
    };
    match g {
        Help => {
            rect(2.0, 2.0, 14.0, 14.0, paper, s);
            text(8.0, 8.3, "?", 11.0, blue);
        }
        Video => {
            rect(1.5, 3.0, 14.5, 13.0, paper, s);
            for x in [3.0, 13.0] {
                for y in [4.5, 7.0, 9.5, 11.5] {
                    fillr(x - 0.6, y - 0.5, x + 0.6, y + 0.5, color);
                }
            }
            closed(&[(6.5, 5.5), (10.5, 8.0), (6.5, 10.5)], blue, Stroke::NONE);
        }
        QuickStart => {
            book("");
            arrow(
                (6.0, 11.0),
                (11.0, 5.0),
                Stroke::new(1.4 * k, accent(GREEN)),
            );
        }
        UserGuide => book("i"),
        WhatsNew => {
            let c = (8.0, 8.0);
            let pts: Vec<(f32, f32)> = (0..16)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 16.0;
                    let rad = if i % 2 == 0 { 7.0 } else { 4.0 };
                    (c.0 + a.cos() * rad, c.1 + a.sin() * rad)
                })
                .collect();
            painter.add(epaint::PathShape::closed_line(
                pts.iter().map(|(x, y)| q(r, *x, *y)).collect(),
                s,
            ));
            circle(8.0, 8.0, 2.0, accent(GOLD), Stroke::NONE);
        }
        Updates => {
            arrow((8.0, 1.5), (8.0, 11.0), bold);
            line((2.0, 14.0), (14.0, 14.0), bold);
        }
        MessageSettings => {
            arrow((6.0, 1.5), (6.0, 8.5), s);
            line((1.5, 11.0), (10.0, 11.0), s);
            gear(12.0, 12.0, 3.6);
        }
        Community => {
            closed(
                &[
                    (2.0, 3.0),
                    (14.0, 3.0),
                    (14.0, 11.0),
                    (7.0, 11.0),
                    (4.0, 14.0),
                    (4.5, 11.0),
                    (2.0, 11.0),
                ],
                paper,
                s,
            );
            for x in [5.0, 8.0, 11.0] {
                circle(x, 7.0, 0.9, color, Stroke::NONE);
            }
        }
        Info => {
            circle(8.0, 8.0, 6.5, color, Stroke::NONE);
            text(8.0, 8.5, "i", 10.0, Tokens::panel());
        }
        SaveAs => {
            rect(1.5, 1.5, 11.5, 11.5, paper, s);
            fillr(3.5, 1.5, 9.5, 5.0, color);
            line((8.0, 15.0), (15.0, 8.0), Stroke::new(2.0 * k, accent(GOLD)));
        }
        CloseDoc => {
            page(2.0, 1.5, 11.5, 14.5);
            cross(12.0, 12.0, 2.5);
        }
        CloseAllDocs => {
            page(4.5, 0.5, 13.5, 11.0);
            page(1.5, 3.5, 10.5, 14.5);
            cross(12.5, 12.5, 2.3);
        }
        Revert => {
            page(4.0, 1.5, 13.5, 14.5);
            poly(
                &[(9.0, 6.0), (4.0, 6.0), (2.0, 8.5), (4.0, 11.0), (8.0, 11.0)],
                s,
            );
            closed(&[(8.5, 4.0), (10.5, 6.0), (8.5, 8.0)], color, Stroke::NONE);
        }
        PrintPreview => {
            page(1.5, 1.5, 10.5, 13.0);
            circle(10.0, 9.5, 3.0, paper, s);
            line((12.2, 11.7), (15.0, 14.5), bold);
        }
        Preflight => {
            page(2.0, 1.5, 12.0, 14.5);
            poly(
                &[(4.5, 8.5), (7.0, 11.0), (12.5, 4.5)],
                Stroke::new(1.8 * k, accent(GREEN)),
            );
        }
        DocProperties => {
            page(2.0, 1.5, 12.0, 14.5);
            circle(11.5, 11.5, 3.5, blue, Stroke::NONE);
            text(11.5, 11.7, "i", 6.0, Color32::WHITE);
        }
        Exit => {
            rect(2.0, 2.0, 9.0, 14.0, paper, s);
            arrow((7.0, 8.0), (15.0, 8.0), Stroke::new(1.4 * k, accent(RED)));
        }
        Mail => {
            rect(1.5, 3.5, 14.5, 12.5, paper, s);
            poly(&[(1.5, 3.5), (8.0, 9.0), (14.5, 3.5)], s);
        }
        Acquire => {
            rect(1.5, 7.0, 14.5, 12.5, paper, s);
            line((3.0, 9.5), (13.0, 9.5), Stroke::new(1.4 * k, blue));
            page(4.0, 1.5, 12.0, 7.0);
        }
        FromTemplate => {
            page(2.0, 1.5, 12.0, 14.5);
            for (y, w) in [(5.0, 6.0), (7.5, 4.0), (10.0, 6.0)] {
                fillr(4.0, y - 0.6, 4.0 + w, y + 0.6, blue);
            }
            plus(12.5, 12.5, 2.5);
        }
        Delete => cross(8.0, 8.0, 5.5),
        Duplicate => {
            rect(1.5, 1.5, 9.5, 9.5, paper, s);
            rect(6.5, 6.5, 14.5, 14.5, paper, s);
        }
        Clone => {
            rect(1.5, 1.5, 9.5, 9.5, paper, s);
            rect(6.5, 6.5, 14.5, 14.5, paper, Stroke::new(1.1 * k, blue));
            line((5.5, 5.5), (10.5, 10.5), thin);
        }
        SelectAll => {
            let d = Stroke::new(1.0 * k, color);
            for (a, b) in [
                ((1.5, 1.5), (14.5, 1.5)),
                ((14.5, 1.5), (14.5, 14.5)),
                ((14.5, 14.5), (1.5, 14.5)),
                ((1.5, 14.5), (1.5, 1.5)),
            ] {
                painter.extend(egui::Shape::dashed_line(
                    &[q(r, a.0, a.1), q(r, b.0, b.1)],
                    d,
                    2.0 * k,
                    1.5 * k,
                ));
            }
            fillr(4.5, 4.5, 8.5, 8.5, blue);
            circle(10.5, 10.5, 2.3, color, Stroke::NONE);
        }
        Find => {
            circle(6.5, 6.5, 4.5, paper, bold);
            line((9.8, 9.8), (14.5, 14.5), Stroke::new(2.4 * k, color));
        }
        StepRepeat => {
            for i in 0..3 {
                let o = i as f32 * 4.0;
                rect(1.5 + o, 1.5 + o, 6.5 + o, 6.5 + o, paper, s);
            }
        }
        CopyProperties => {
            rect(1.5, 1.5, 8.0, 8.0, blue, s);
            rect(8.0, 8.0, 14.5, 14.5, paper, s);
            arrow((5.0, 10.5), (9.5, 10.5), thin);
        }
        PasteSpecial => {
            rect(2.5, 3.0, 12.5, 14.5, paper, s);
            fillr(5.0, 1.5, 10.0, 4.0, color);
            text(7.5, 9.5, "*", 9.0, blue);
        }
        Repeat => {
            let pts: Vec<(f32, f32)> = (0..=14)
                .map(|i| {
                    let a = -0.6 + i as f32 * 0.38;
                    (8.0 + a.cos() * 5.0, 8.0 + a.sin() * 5.0)
                })
                .collect();
            poly(&pts, bold);
            closed(
                &[(13.5, 2.5), (14.5, 7.5), (10.0, 5.5)],
                color,
                Stroke::NONE,
            );
        }
        PageInsert => {
            page(1.5, 1.5, 11.0, 14.5);
            plus(12.0, 11.5, 3.0);
        }
        PageDelete => {
            page(1.5, 1.5, 11.0, 14.5);
            cross(12.0, 11.5, 2.6);
        }
        PageRename => {
            page(1.5, 1.5, 11.0, 14.5);
            line((13.0, 4.0), (13.0, 14.0), s);
            line((11.5, 4.0), (14.5, 4.0), s);
            line((11.5, 14.0), (14.5, 14.0), s);
        }
        PageGoto => {
            page(1.5, 1.5, 11.0, 14.5);
            arrow((7.0, 8.0), (15.0, 8.0), Stroke::new(1.4 * k, blue));
        }
        PageDuplicate => {
            page(4.5, 0.5, 14.0, 11.5);
            page(1.5, 3.5, 11.0, 15.0);
            plus(12.0, 12.0, 2.8);
        }
        PageInsertAfter | PageInsertBefore => {
            // Two pages side by side: the current one with lines of
            // text, the new one empty with a green plus, on the right
            // for After and on the left for Before.
            let after = g == PageInsertAfter;
            let (old, new) = if after { (0.5, 8.5) } else { (8.5, 0.5) };
            page(old, 4.5, old + 7.0, 15.5);
            for y in [9.0, 11.0, 13.0] {
                line((old + 1.5, y), (old + 5.5, y), thin);
            }
            page(new, 4.5, new + 7.0, 15.5);
            plus(new + 3.5, 10.5, 2.4);
            // A small arrow over the pair toward the new page.
            let (a, b) = if after { (4.0, 12.0) } else { (12.0, 4.0) };
            arrow((a, 2.2), (b, 2.2), Stroke::new(1.0 * k, blue));
        }
        PageOrientation => {
            rect(1.5, 4.0, 14.5, 12.0, paper, s);
            rect(5.0, 1.5, 11.0, 14.5, Color32::TRANSPARENT, thin);
        }
        PageNumber => {
            page(2.0, 1.5, 12.0, 14.5);
            text(7.0, 9.5, "#", 8.0, blue);
        }
        DocOptions => {
            page(1.5, 1.5, 11.0, 14.5);
            gear(11.5, 11.5, 4.0);
        }
        PageBackground => {
            // The page on a backdrop in the accent: a tinted square with
            // the accent frame, the page over it.
            rect(
                1.5,
                1.5,
                14.5,
                14.5,
                accent(blue.gamma_multiply(0.22)),
                Stroke::new(1.0 * k, blue),
            );
            page(4.0, 3.5, 12.0, 13.0);
        }
        PageSize => {
            page(3.0, 3.5, 12.5, 15.0);
            arrow((2.0, 1.5), (14.0, 1.5), thin);
            arrow((14.0, 1.5), (2.0, 1.5), thin);
        }
        AlignLeft | AlignRight | AlignCenterH => {
            let x = match g {
                AlignLeft => 2.0,
                AlignRight => 14.0,
                _ => 8.0,
            };
            line((x, 1.0), (x, 15.0), s);
            let place = |w: f32| match g {
                AlignLeft => (2.0, 2.0 + w),
                AlignRight => (14.0 - w, 14.0),
                _ => (8.0 - w / 2.0, 8.0 + w / 2.0),
            };
            let (a, b) = place(10.0);
            blk(a, 3.0, b, 6.5, blue);
            let (a, b) = place(6.0);
            blk(a, 9.5, b, 13.0, color);
        }
        AlignTop | AlignBottom | AlignCenterV => {
            let y = match g {
                AlignTop => 2.0,
                AlignBottom => 14.0,
                _ => 8.0,
            };
            line((1.0, y), (15.0, y), s);
            let place = |h: f32| match g {
                AlignTop => (2.0, 2.0 + h),
                AlignBottom => (14.0 - h, 14.0),
                _ => (8.0 - h / 2.0, 8.0 + h / 2.0),
            };
            let (a, b) = place(10.0);
            blk(3.0, a, 6.5, b, blue);
            let (a, b) = place(6.0);
            blk(9.5, a, 13.0, b, color);
        }
        CenterPage => {
            page(2.0, 1.5, 14.0, 14.5);
            blk(5.5, 5.5, 10.5, 10.5, blue);
        }
        DistLeft | DistCenterH | DistSpaceH | DistRight => {
            for (i, (y0, y1)) in [(3.0, 8.0), (6.0, 13.0), (4.0, 10.0)].iter().enumerate() {
                let x = 1.5 + i as f32 * 5.0;
                blk(x, *y0, x + 3.0, *y1, if i == 1 { blue } else { color });
            }
            let mark = |x: f32| line((x, 0.5), (x, 15.5), thin);
            match g {
                DistLeft => [1.5, 6.5, 11.5].into_iter().for_each(mark),
                DistRight => [4.5, 9.5, 14.5].into_iter().for_each(mark),
                DistCenterH => [3.0, 8.0, 13.0].into_iter().for_each(mark),
                _ => {
                    arrow((4.5, 15.0), (6.5, 15.0), thin);
                    arrow((9.5, 15.0), (11.5, 15.0), thin);
                }
            }
        }
        DistTop | DistCenterV | DistSpaceV | DistBottom => {
            for (i, (x0, x1)) in [(3.0, 8.0), (6.0, 13.0), (4.0, 10.0)].iter().enumerate() {
                let y = 1.5 + i as f32 * 5.0;
                blk(*x0, y, *x1, y + 3.0, if i == 1 { blue } else { color });
            }
            let mark = |y: f32| line((0.5, y), (15.5, y), thin);
            match g {
                DistTop => [1.5, 6.5, 11.5].into_iter().for_each(mark),
                DistBottom => [4.5, 9.5, 14.5].into_iter().for_each(mark),
                DistCenterV => [3.0, 8.0, 13.0].into_iter().for_each(mark),
                _ => {
                    arrow((15.0, 4.5), (15.0, 6.5), thin);
                    arrow((15.0, 9.5), (15.0, 11.5), thin);
                }
            }
        }
        Forward | Backward | InFront | Behind => {
            let front_blue = matches!(g, Forward | InFront);
            let (back, front) = if front_blue {
                (color, blue)
            } else {
                (blue, color)
            };
            rect(1.5, 1.5, 9.5, 9.5, back, s);
            rect(6.5, 6.5, 14.5, 14.5, front, s);
            if matches!(g, InFront | Behind) {
                arrow((12.0, 1.0), (12.0, 4.5), thin);
            }
        }
        ReverseOrder => {
            rect(1.5, 1.5, 8.5, 8.5, blue, s);
            rect(7.5, 7.5, 14.5, 14.5, color, s);
            arrow((12.0, 5.5), (12.0, 1.5), thin);
            arrow((4.0, 10.5), (4.0, 14.5), thin);
        }
        Hide => eye(8.0, 8.0, false),
        ShowAll => eye(8.0, 8.0, true),
        Unlock => lock(true),
        UnlockAll => {
            lock(true);
            text(12.5, 4.0, "*", 7.0, blue);
        }
        OutlineToObject => {
            circle(
                8.0,
                8.0,
                5.5,
                Color32::TRANSPARENT,
                Stroke::new(2.6 * k, color),
            );
            arrow((1.0, 14.5), (5.0, 10.5), Stroke::new(1.0 * k, blue));
        }
        JoinCurves => {
            poly(&[(1.5, 12.0), (5.0, 5.0), (8.0, 8.0)], s);
            poly(&[(8.0, 8.0), (11.0, 11.0), (14.5, 4.0)], s);
            circle(8.0, 8.0, 1.8, blue, Stroke::NONE);
        }
        Perspective => {
            closed(
                &[(4.0, 3.0), (12.0, 3.0), (15.0, 13.0), (1.0, 13.0)],
                paper,
                s,
            );
            line((8.0, 3.0), (8.0, 13.0), thin);
            line((2.5, 8.0), (13.5, 8.0), thin);
        }
        ClearTransform => {
            closed(
                &[(4.0, 2.0), (14.0, 4.0), (12.0, 14.0), (2.0, 12.0)],
                paper,
                s,
            );
            cross(12.0, 12.0, 2.5);
        }
        Properties => {
            rect(1.5, 1.5, 14.5, 14.5, paper, s);
            for (y, c) in [(5.0, blue), (8.0, color), (11.0, color)] {
                fillr(3.5, y - 0.7, 5.0, y + 0.7, c);
                line((6.5, y), (12.5, y), thin);
            }
        }
        ObjectsDocker => {
            for (i, c) in [blue, color, color].iter().enumerate() {
                let y = 2.0 + i as f32 * 4.5;
                closed(
                    &[(8.0, y), (15.0, y + 2.5), (8.0, y + 5.0), (1.0, y + 2.5)],
                    if i == 0 { *c } else { paper },
                    s,
                );
            }
        }
        ClipInside | ClipExtract => {
            rect(1.5, 1.5, 14.5, 14.5, paper, s);
            circle(8.0, 8.0, 3.5, blue, Stroke::NONE);
            if g == ClipInside {
                arrow((15.5, 0.5), (10.5, 5.5), thin);
            } else {
                arrow((10.5, 5.5), (15.5, 0.5), thin);
            }
        }
        ClipEdit => {
            rect(1.5, 1.5, 14.5, 14.5, paper, s);
            circle(8.0, 8.0, 3.5, blue, Stroke::NONE);
            line((9.0, 14.5), (15.0, 8.5), Stroke::new(2.0 * k, accent(GOLD)));
        }
        ClipFinish => {
            rect(1.5, 1.5, 14.5, 14.5, paper, s);
            poly(
                &[(4.0, 8.5), (7.0, 11.5), (12.5, 4.5)],
                Stroke::new(1.8 * k, accent(GREEN)),
            );
        }
        Barcode => {
            for (x, w) in [
                (2.0, 1.0),
                (4.0, 0.6),
                (5.5, 1.4),
                (8.0, 0.6),
                (9.5, 1.0),
                (11.5, 0.6),
                (13.0, 1.2),
            ] {
                fillr(x, 2.5, x + w, 13.5, color);
            }
        }
        QrCode => {
            for (x, y) in [(1.5, 1.5), (10.0, 1.5), (1.5, 10.0)] {
                rect(x, y, x + 4.5, y + 4.5, Color32::TRANSPARENT, s);
                fillr(x + 1.5, y + 1.5, x + 3.0, y + 3.0, color);
            }
            for (x, y) in [
                (10.0, 10.0),
                (12.5, 12.5),
                (8.0, 8.0),
                (12.5, 9.0),
                (9.0, 13.0),
            ] {
                fillr(x, y, x + 1.6, y + 1.6, color);
            }
        }
        Symmetry => {
            line((8.0, 0.5), (8.0, 15.5), Stroke::new(1.0 * k, accent(RED)));
            closed(&[(6.5, 3.0), (6.5, 13.0), (1.5, 9.0)], color, Stroke::NONE);
            closed(&[(9.5, 3.0), (14.5, 9.0), (9.5, 13.0)], paper, s);
        }
        Symbol => {
            circle(8.0, 8.0, 6.5, paper, s);
            closed(&[(8.0, 3.5), (12.0, 11.0), (4.0, 11.0)], blue, Stroke::NONE);
        }
        Blend => {
            for i in 0..5 {
                let t = i as f32 / 4.0;
                let x = 2.5 + t * 11.0;
                let c = if enabled {
                    Color32::from_rgb(
                        (0x1F as f32 + (0xD0 - 0x1F) as f32 * t) as u8,
                        (0x6F as f32 - 0x44 as f32 * t) as u8,
                        (0xC5 as f32 - 0x9A as f32 * t) as u8,
                    )
                } else {
                    color
                };
                circle(x, 8.0, 2.4 - t * 0.6 + 0.6, c, Stroke::NONE);
            }
        }
        Contour => {
            for (i, rad) in [6.5, 4.5, 2.5].iter().enumerate() {
                let c = if i == 2 { blue } else { Color32::TRANSPARENT };
                rect(8.0 - rad, 8.0 - rad, 8.0 + rad, 8.0 + rad, c, s);
            }
        }
        Extrude => {
            closed(&[(1.5, 5.5), (9.5, 5.5), (9.5, 14.5), (1.5, 14.5)], blue, s);
            closed(&[(1.5, 5.5), (6.0, 1.5), (14.0, 1.5), (9.5, 5.5)], paper, s);
            closed(
                &[(9.5, 5.5), (14.0, 1.5), (14.0, 10.5), (9.5, 14.5)],
                color,
                s,
            );
        }
        Lens => {
            rect(1.5, 1.5, 10.5, 10.5, blue, Stroke::NONE);
            circle(
                9.5,
                9.5,
                5.0,
                accent(Color32::from_rgba_unmultiplied(255, 220, 0, 150)),
                s,
            );
        }
        Bevel => {
            rect(1.5, 1.5, 14.5, 14.5, color, Stroke::NONE);
            closed(
                &[(1.5, 1.5), (14.5, 1.5), (11.0, 5.0), (5.0, 5.0)],
                paper,
                Stroke::NONE,
            );
            closed(
                &[(1.5, 1.5), (5.0, 5.0), (5.0, 11.0), (1.5, 14.5)],
                paper,
                Stroke::NONE,
            );
            rect(5.0, 5.0, 11.0, 11.0, blue, Stroke::NONE);
        }
        BrushStrokes => {
            let pts: Vec<(f32, f32)> = (0..=12)
                .map(|i| {
                    let t = i as f32 / 12.0;
                    (1.5 + t * 13.0, 9.0 - (t * 6.0).sin() * 4.0)
                })
                .collect();
            poly(&pts, Stroke::new(2.8 * k, blue));
        }
        Mosaic => {
            for y in 0..4 {
                for x in 0..4 {
                    let c = if (x + y) % 3 == 0 {
                        blue
                    } else if (x * y) % 2 == 1 {
                        color
                    } else {
                        paper
                    };
                    rect(
                        1.5 + x as f32 * 3.3,
                        1.5 + y as f32 * 3.3,
                        4.3 + x as f32 * 3.3,
                        4.3 + y as f32 * 3.3,
                        c,
                        thin,
                    );
                }
            }
        }
        Flatten => {
            rect(1.5, 1.5, 9.5, 9.5, blue, s);
            rect(5.5, 5.5, 14.5, 14.5, paper, s);
            arrow((8.0, 1.0), (8.0, 6.0), thin);
        }
        TableNew => {
            grid(1.5, 2.5, 14.5, 13.5, 3, 3);
        }
        TextToTable => {
            for y in [3.0, 6.0] {
                line((1.5, y), (8.0, y), s);
            }
            grid(7.0, 8.0, 14.5, 14.5, 2, 2);
            arrow((3.0, 9.0), (6.0, 12.0), thin);
        }
        TableToText => {
            grid(1.5, 1.5, 9.0, 8.0, 2, 2);
            for y in [11.0, 14.0] {
                line((8.0, y), (14.5, y), s);
            }
            arrow((10.0, 4.0), (13.0, 7.0), thin);
        }
        RowAbove | RowBelow => {
            // The new row (blue) above or below the table.
            let (grid_y, y) = if g == RowAbove {
                (6.0, 3.0)
            } else {
                (1.5, 13.0)
            };
            grid(1.5, grid_y, 14.5, grid_y + 8.5, 2, 3);
            fillr(1.5, y - 1.5, 10.0, y + 1.5, blue);
            plus(13.0, y, 2.0);
        }
        ColLeft | ColRight => {
            let (grid_x, x) = if g == ColLeft {
                (6.0, 3.0)
            } else {
                (1.5, 13.0)
            };
            grid(grid_x, 1.5, grid_x + 8.5, 14.5, 3, 2);
            fillr(x - 1.5, 1.5, x + 1.5, 10.0, blue);
            plus(x, 13.0, 2.0);
        }
        DelRow => {
            grid(1.5, 1.5, 14.5, 10.0, 2, 3);
            cross(8.0, 13.0, 2.2);
        }
        DelCol => {
            grid(1.5, 1.5, 10.0, 14.5, 3, 2);
            cross(13.0, 8.0, 2.2);
        }
        DelTable => {
            grid(1.5, 1.5, 12.0, 12.0, 3, 3);
            cross(12.0, 12.0, 2.5);
        }
        MergeCells => {
            rect(1.5, 3.0, 14.5, 13.0, paper, s);
            arrow((3.0, 8.0), (7.0, 8.0), thin);
            arrow((13.0, 8.0), (9.0, 8.0), thin);
        }
        SplitCells => {
            grid(1.5, 3.0, 14.5, 13.0, 1, 2);
            arrow((7.0, 8.0), (3.0, 8.0), thin);
            arrow((9.0, 8.0), (13.0, 8.0), thin);
        }
        DistRows => {
            grid(1.5, 1.5, 14.5, 14.5, 3, 1);
            arrow((12.0, 3.0), (12.0, 13.0), thin);
        }
        DistCols => {
            grid(1.5, 1.5, 14.5, 14.5, 1, 3);
            arrow((3.0, 12.0), (13.0, 12.0), thin);
        }
        ColorManagement => {
            for (x, y, c) in [
                (6.0, 6.0, Color32::from_rgb(0xE0, 0x30, 0x30)),
                (10.0, 6.0, Color32::from_rgb(0x30, 0xA0, 0x40)),
                (8.0, 10.0, Color32::from_rgb(0x30, 0x60, 0xE0)),
            ] {
                circle(x, y, 4.0, accent(c), Stroke::NONE);
            }
        }
        FontManager => {
            text(6.0, 8.0, "A", 12.0, color);
            text(12.0, 10.0, "a", 9.0, blue);
        }
        TraceCut => {
            // A shape with its cut line dashed round it and the knife
            // cutting along it: the cutting plotter tools.
            let cut = Stroke::new(1.3 * k, blue);
            let n = 16;
            for i in (0..n).step_by(2) {
                let a0 = i as f32 / n as f32 * std::f32::consts::TAU;
                let a1 = (i + 1) as f32 / n as f32 * std::f32::consts::TAU;
                line(
                    (7.0 + 5.5 * a0.cos(), 9.0 + 5.5 * a0.sin()),
                    (7.0 + 5.5 * a1.cos(), 9.0 + 5.5 * a1.sin()),
                    cut,
                );
            }
            // The knife: a handle and its blade, the tip on the cut line.
            line((15.0, 1.0), (11.8, 4.2), bold);
            closed(&[(11.8, 4.2), (9.9, 4.6), (11.0, 5.9)], color, Stroke::NONE);
        }
        Ai => {
            closed(
                &[
                    (6.0, 1.0),
                    (7.5, 6.5),
                    (13.0, 8.0),
                    (7.5, 9.5),
                    (6.0, 15.0),
                    (4.5, 9.5),
                    (-1.0, 8.0),
                    (4.5, 6.5),
                ],
                accent(GOLD),
                Stroke::NONE,
            );
            text(12.5, 4.0, "+", 7.0, blue);
        }
        Clock => {
            circle(8.0, 8.0, 6.5, paper, s);
            line((8.0, 8.0), (8.0, 3.5), s);
            line((8.0, 8.0), (11.0, 9.5), s);
        }
        Script => {
            page(2.0, 1.5, 13.0, 14.5);
            text(7.5, 9.0, "{}", 7.0, blue);
        }
        Record => {
            circle(8.0, 8.0, 5.0, accent(RED), Stroke::NONE);
        }
        Grommet => {
            rect(1.5, 1.5, 14.5, 14.5, paper, s);
            for (x, y) in [
                (3.5, 3.5),
                (12.5, 3.5),
                (3.5, 12.5),
                (12.5, 12.5),
                (8.0, 3.5),
                (8.0, 12.5),
            ] {
                circle(x, y, 1.2, Color32::TRANSPARENT, thin);
            }
        }
        SaveDefaults => {
            rect(1.5, 1.5, 11.5, 11.5, paper, s);
            fillr(3.5, 1.5, 9.5, 5.0, color);
            gear(12.0, 12.0, 3.6);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracecut_has_its_own_picture() {
        assert_eq!(for_key("menu.tools.tracecut"), Some(Glyph::TraceCut));
        let ctx = crate::theme::ui_context();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let r = Rect::from_min_size(Pos2::ZERO, Vec2::splat(16.0));
            assert!(crate::ui::menus::draw_menu_picture(
                ui.painter(),
                r,
                "menu.tools.tracecut",
                Color32::GRAY
            ));
        });
        out.textures_delta.clear();
    }
}
