//! Import PDF Pages: which pages of a PDF (or AI file) of several pages
//! File > Import brings in, and where: as new pages after the current
//! one, each at its own size, or on the current page, each page's
//! objects a group. A file of one page comes in as before, without
//! asking.

use crate::app::App;
use crate::i18n::{tr, trf};
use crate::ui::dialogs::{ok_cancel, window};
use crate::ui::tips::RichTip;
use egui::{Color32, Context, RichText};
use std::path::PathBuf;
use std::sync::Arc;
use tracedraw_core::{Command, Fill, Shape, ShapeId, ShapeKind};
use tracedraw_io::pdf_import::ImportedPdf;

const ERROR: Color32 = Color32::from_rgb(0xC4, 0x2B, 0x1C);

/// The dialog's state: the file is read already (its text as the Import
/// PDF dialog said, its password given), so nothing is asked again.
#[derive(Clone)]
pub struct PdfPagesState {
    pub path: PathBuf,
    pub pdf: Arc<ImportedPdf>,
    /// Every page, or those of `range`.
    pub all: bool,
    /// The pages as typed ("1-3, 5", from 1).
    pub range: String,
    /// As new pages after the current one, rather than on it.
    pub new_pages: bool,
    /// OK was pressed with no page chosen.
    pub none_chosen: bool,
}

impl PartialEq for PdfPagesState {
    fn eq(&self, o: &Self) -> bool {
        self.path == o.path
            && Arc::ptr_eq(&self.pdf, &o.pdf)
            && self.all == o.all
            && self.range == o.range
            && self.new_pages == o.new_pages
            && self.none_chosen == o.none_chosen
    }
}

impl std::fmt::Debug for PdfPagesState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PdfPagesState")
            .field("path", &self.path)
            .field("pages", &self.pdf.pages.len())
            .field("all", &self.all)
            .field("range", &self.range)
            .field("new_pages", &self.new_pages)
            .finish_non_exhaustive()
    }
}

impl PdfPagesState {
    /// The chosen pages, from 0, in the file's order, each once.
    pub fn chosen(&self) -> Vec<usize> {
        chosen_pages(self.all, &self.range, self.pdf.pages.len())
    }
}

/// The pages `range` ("1-3, 5", from 1) names among `count`, from 0, in
/// the file's order whatever order they were typed in; every page when
/// `all`.
pub fn chosen_pages(all: bool, range: &str, count: usize) -> Vec<usize> {
    if all {
        return (0..count).collect();
    }
    let mut pages = crate::ui::dialogs::parse_page_list(range, count);
    pages.sort_unstable();
    pages
}

/// Pages (from 0) written as a list from 1 with ranges: [0, 1, 2, 4]
/// gives "1-3, 5".
pub fn page_list_text(pages: &[usize]) -> String {
    let mut sorted = pages.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut parts = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let start = sorted[i];
        let mut end = start;
        while i + 1 < sorted.len() && sorted[i + 1] == end + 1 {
            i += 1;
            end = sorted[i];
        }
        parts.push(if start == end {
            (start + 1).to_string()
        } else {
            format!("{}-{}", start + 1, end + 1)
        });
        i += 1;
    }
    parts.join(", ")
}

impl App {
    /// A PDF of several pages read for File > Import: ask which pages
    /// come in and where.
    pub fn ask_pdf_pages(&mut self, path: &std::path::Path, pdf: ImportedPdf) {
        let n = pdf.pages.len();
        self.dialog = crate::ui::dialogs::Dialog::PdfPages(PdfPagesState {
            path: path.to_path_buf(),
            pdf: Arc::new(pdf),
            all: true,
            range: page_list_text(&(0..n).collect::<Vec<_>>()),
            new_pages: true,
            none_chosen: false,
        });
    }

    /// Bring the pages `chosen` (from 0) of `pdf`, read from `path`, into
    /// the drawing as one undo step, the file's spot inks joining the
    /// drawing's. With `new_pages` each becomes a page after the current
    /// one, at its own size and in order, and the first of them is shown;
    /// otherwise their objects go on the active layer, each page's a group
    /// when there are several. What came is selected.
    pub fn import_pdf_pages(
        &mut self,
        path: &std::path::Path,
        pdf: &ImportedPdf,
        chosen: &[usize],
        new_pages: bool,
    ) {
        let pages: Vec<(usize, &tracedraw_io::pdf_import::ImportedPage)> = chosen
            .iter()
            .filter_map(|&i| pdf.pages.get(i).map(|p| (i, p)))
            .collect();
        if pages.is_empty() {
            return;
        }
        let mut cmds = Vec::new();
        // The file's spot inks join the drawing's: one of the same name is
        // the same ink, the others are added.
        let (inks, map) = crate::app::merge_inks(&self.doc().inks, &pdf.inks);
        if inks != self.doc().inks {
            cmds.push(Command::SetInks { inks });
        }
        let recolor = |mut s: Shape| {
            if !map.is_empty() {
                crate::color_policy::recolor_shape(&mut s, &|c| crate::app::renumber_ink(c, &map));
            }
            s
        };
        if new_pages {
            let at = self.page_index() + 1;
            let base = self.doc().pages.len();
            for (k, (_, p)) in pages.iter().enumerate() {
                cmds.push(Command::InsertPage {
                    at: at + k,
                    name: Some(trf("doc.page_n", &[("n", &(base + k + 1).to_string())])),
                    size: p.size,
                    layer: Some(trf("docker.layer_n", &[("n", "1")])),
                    shapes: p.shapes.iter().cloned().map(recolor).collect(),
                });
            }
            if let Err(e) = self.engine.run_batch("Import", &cmds) {
                self.status = e.to_string();
                return;
            }
            let ids: Vec<ShapeId> = self
                .doc()
                .pages
                .iter()
                .skip(at)
                .take(pages.len())
                .flat_map(|p| &p.layers)
                .flat_map(|l| &l.shapes)
                .map(|s| s.id)
                .collect();
            self.hyphenate_imported(&ids);
            // The objects stay where the file has them on their pages: a
            // place given with the import cursor is not for them.
            self.import_place_waiting = None;
            self.goto_page(at);
            let first: Vec<ShapeId> = self
                .doc()
                .pages
                .get(at)
                .map(|p| p.layers.iter().flat_map(|l| &l.shapes).map(|s| s.id))
                .into_iter()
                .flatten()
                .collect();
            self.select(first);
            self.status = trf(
                "status.imported_pdf_pages",
                &[("pages", &pages.len().to_string())],
            );
        } else {
            let Some(layer) = self.active_layer() else {
                return;
            };
            let several = pages.len() > 1;
            let mut ids = Vec::new();
            for (i, p) in &pages {
                let mut children = Vec::with_capacity(p.shapes.len());
                for s in p.shapes.iter().cloned().map(recolor) {
                    let id = self.engine.new_shape_id();
                    children.push(crate::app::reid_pub(s, id, &mut self.engine));
                }
                if several && children.len() > 1 {
                    let id = self.engine.new_shape_id();
                    let mut g = Shape::new(id, ShapeKind::Group { children });
                    g.fill = Fill::None;
                    g.stroke = None;
                    g.name = Some(trf("doc.page_n", &[("n", &(i + 1).to_string())]));
                    ids.push(id);
                    cmds.push(Command::AddShape { layer, shape: g });
                } else {
                    for shape in children {
                        ids.push(shape.id);
                        cmds.push(Command::AddShape { layer, shape });
                    }
                }
            }
            if let Err(e) = self.engine.run_batch("Import", &cmds) {
                self.status = e.to_string();
                return;
            }
            self.hyphenate_imported(&ids);
            self.select(ids);
        }
        if !pdf.warnings.is_empty() {
            log::warn!("pdf import: {}", pdf.warnings.join("; "));
        }
        // The chosen pages' pictures that came in smaller (#1835).
        let shrunk: Vec<_> = pdf
            .shrunk_pictures
            .iter()
            .filter(|p| chosen.contains(&p.page))
            .copied()
            .collect();
        if !shrunk.is_empty() && matches!(self.dialog, crate::ui::dialogs::Dialog::None) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            self.dialog = crate::ui::dialogs::Dialog::OpenNotes(crate::ui::dialogs::OpenNotes {
                name,
                shrunk: crate::ui::dialogs::shrunk_sizes(&shrunk),
                ..Default::default()
            });
        }
    }
}

/// "210 x 297 mm, Portrait" in the document's units.
fn size_text(app: &App, size: tracedraw_core::geometry::Size) -> String {
    let u = app.units;
    let r = |v: f64| (u.from_mm(v) * 100.0).round() / 100.0;
    let orientation = if size.width > size.height {
        tr("dialog.landscape")
    } else {
        tr("dialog.portrait")
    };
    format!(
        "{} x {} {}, {orientation}",
        r(size.width),
        r(size.height),
        u.short()
    )
}

pub fn dialog(app: &mut App, ctx: &Context, st: &mut PdfPagesState, close: &mut bool) {
    window(ctx, tr("pdf_pages.title")).show(ctx, |ui| {
        ui.set_max_width(420.0);
        let name = st
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let count = st.pdf.pages.len();
        ui.label(trf(
            "pdf_pages.prompt",
            &[("name", &name), ("n", &count.to_string())],
        ));
        ui.add_space(6.0);
        ui.label(tr("pdf_pages.pages"));
        ui.radio_value(&mut st.all, true, tr("pdf_pages.all"));
        ui.horizontal(|ui| {
            ui.radio_value(&mut st.all, false, tr("pdf_pages.range"));
            let field = ui.add(
                egui::TextEdit::singleline(&mut st.range)
                    .desired_width(160.0)
                    .id(egui::Id::new("pdf_pages_range")),
            );
            if field.changed() || field.gained_focus() {
                st.all = false;
            }
            field.rich_tip(tr("pdf_pages.range_tip"));
        });
        // Each page with its size; ticking one changes the list above.
        let chosen = st.chosen();
        egui::ScrollArea::vertical()
            .max_height(180.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for (i, p) in st.pdf.pages.iter().enumerate() {
                    let mut on = chosen.contains(&i);
                    let label = format!(
                        "{}: {}",
                        trf("doc.page_n", &[("n", &(i + 1).to_string())]),
                        size_text(app, p.size)
                    );
                    if ui.checkbox(&mut on, label).changed() {
                        let mut picked: Vec<usize> =
                            chosen.iter().copied().filter(|&k| k != i).collect();
                        if on {
                            picked.push(i);
                        }
                        st.range = page_list_text(&picked);
                        st.all = false;
                    }
                }
            });
        let chosen = st.chosen();
        ui.add_space(6.0);
        ui.label(tr("pdf_pages.where"));
        ui.radio_value(&mut st.new_pages, true, tr("pdf_pages.new_pages"))
            .rich_tip(tr("pdf_pages.new_pages_tip"));
        ui.radio_value(&mut st.new_pages, false, tr("pdf_pages.current_page"))
            .rich_tip(tr("pdf_pages.current_page_tip"));
        if st.none_chosen && chosen.is_empty() {
            ui.label(RichText::new(tr("pdf_pages.none_chosen")).color(ERROR));
        }
        if ok_cancel(ui, close) {
            if chosen.is_empty() {
                st.none_chosen = true;
                *close = false;
                return;
            }
            let path = st.path.clone();
            let pdf = st.pdf.clone();
            app.import_pdf_pages(&path, &pdf, &chosen, st.new_pages);
            if !st.new_pages
                && app.import_place_waiting.is_none()
                && app.settings.formats.import_place == crate::prefs::ImportPlace::ViewCenter
            {
                // As File > Import puts what a file holds without asking.
                let at = app.view_center();
                app.move_import_to(at);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::dialogs::Dialog;
    use egui::{vec2, Pos2, Rect};
    use tracedraw_core::geometry::Size;

    #[test]
    fn page_lists_read_and_write_ranges() {
        assert_eq!(chosen_pages(true, "2", 3), vec![0, 1, 2]);
        assert_eq!(chosen_pages(false, "1-3, 5", 6), vec![0, 1, 2, 4]);
        // Typed out of order or twice: the file's order, each once.
        assert_eq!(chosen_pages(false, "5, 2-3, 2", 6), vec![1, 2, 4]);
        // Pages past the end and words are left out.
        assert_eq!(chosen_pages(false, "4-9, x", 5), vec![3, 4]);
        assert!(chosen_pages(false, "", 5).is_empty());
        assert_eq!(page_list_text(&[4, 0, 1, 2]), "1-3, 5");
        assert_eq!(page_list_text(&[1]), "2");
        assert_eq!(page_list_text(&[]), "");
        assert_eq!(
            chosen_pages(false, &page_list_text(&[0, 2, 3]), 4),
            [0, 2, 3]
        );
    }

    /// A PDF of three pages of different sizes, each with as many
    /// rectangles as its number.
    fn three_page_pdf(dir: &std::path::Path) -> std::path::PathBuf {
        let sizes = [
            Size::new(210.0, 297.0),
            Size::new(297.0, 210.0),
            Size::new(100.0, 50.0),
        ];
        let mut src = tracedraw_core::Document::new("three", sizes[0]);
        let mut ids = src.ids().clone();
        for (i, size) in sizes.iter().enumerate() {
            if i > 0 {
                let mut p = src.pages[0].clone();
                p.id = ids.page();
                p.size = *size;
                p.layers[0].id = ids.layer();
                p.layers[0].shapes.clear();
                src.pages.push(p);
            }
            for k in 0..=i {
                let x = 10.0 + 15.0 * k as f64;
                let mut s = Shape::new(
                    ids.shape(),
                    ShapeKind::Rect {
                        rect: tracedraw_core::geometry::Rect::new(x, 10.0, x + 10.0, 20.0),
                        radius: 0.0,
                        corners: None,
                    },
                );
                s.fill = Fill::Solid(tracedraw_core::Color::rgb8(200, 0, 0));
                s.stroke = None;
                src.pages[i].layers[0].shapes.push(s);
            }
        }
        src.set_ids(ids);
        let path = dir.join("three.pdf");
        std::fs::write(&path, tracedraw_io::pdf::document_to_pdf(&src)).expect("written");
        path
    }

    fn frame(app: &mut App, ctx: &egui::Context, events: Vec<egui::Event>) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
            events,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| crate::ui::dialogs::show(app, ui.ctx()));
        out.textures_delta.clear();
    }

    fn enter() -> egui::Event {
        egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }
    }

    fn undo_steps(app: &App) -> usize {
        app.engine.history_labels().0.len()
    }

    fn near(a: Size, b: Size) -> bool {
        (a.width - b.width).abs() < 0.1 && (a.height - b.height).abs() < 0.1
    }

    /// Importing a PDF of several pages asks which and where (after the
    /// question how to read its text); OK brings the chosen pages in
    /// after the current one with their sizes as one undo step, or on
    /// the current page as one group each.
    #[test]
    fn importing_a_pdf_of_several_pages_asks_which_and_where() {
        let dir = std::env::temp_dir().join(format!("tracedraw-pdfpages-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = three_page_pdf(&dir);
        let mut app = App::headless();
        app.settings.formats.pdf_ask_text = true;
        let first = app.doc().pages[0].id;
        let first_size = app.doc().pages[0].size;
        app.run(app.localized_page(Size::new(50.0, 50.0)));
        let last = app.doc().pages[1].id;
        app.goto_page(0);
        let ctx = crate::theme::ui_context();

        // The text question comes first; its OK reads the file and asks
        // for the pages.
        app.import_pdf(&path);
        assert!(matches!(app.dialog, Dialog::PdfText(_)), "{:?}", app.dialog);
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![enter()]);
        assert_eq!(app.doc().pages.len(), 2, "nothing came yet");
        let Dialog::PdfPages(st) = &mut app.dialog else {
            panic!("no pages asked: {:?}", app.dialog);
        };
        assert!(st.all && st.new_pages);
        assert_eq!(st.range, "1-3");

        // Nothing chosen: OK says so and keeps asking.
        st.all = false;
        st.range = "9".into();
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![enter()]);
        assert_eq!(app.doc().pages.len(), 2);
        let Dialog::PdfPages(st) = &mut app.dialog else {
            panic!("closed with nothing chosen: {:?}", app.dialog);
        };
        assert!(st.none_chosen);

        // Pages 3 and 1, typed out of order: as new pages after the
        // current one, in the file's order.
        st.range = "3, 1".into();
        let pdf = st.pdf.clone();
        let undo = undo_steps(&app);
        frame(&mut app, &ctx, vec![]);
        frame(&mut app, &ctx, vec![enter()]);
        assert!(matches!(app.dialog, Dialog::None), "{:?}", app.dialog);
        let d = app.doc();
        assert_eq!(d.pages.len(), 4);
        assert_eq!(d.pages[0].id, first);
        assert_eq!(d.pages[0].size, first_size);
        assert_eq!(d.pages[3].id, last);
        assert!(
            near(d.pages[1].size, Size::new(210.0, 297.0)),
            "{:?}",
            d.pages[1].size
        );
        assert!(
            near(d.pages[2].size, Size::new(100.0, 50.0)),
            "{:?}",
            d.pages[2].size
        );
        let count = |i: usize| {
            d.pages[i]
                .layers
                .iter()
                .map(|l| l.shapes.len())
                .sum::<usize>()
        };
        assert_eq!((count(1), count(2)), (1, 3));
        // The first new page is shown with its objects selected.
        assert_eq!(app.page, app.doc().pages[1].id);
        assert_eq!(app.selection.len(), 1);
        assert_eq!(undo_steps(&app), undo + 1, "one undo step");
        app.undo();
        assert_eq!(app.doc().pages.len(), 2);

        // On the current page: one group a page, stacked, one undo step.
        app.goto_page(0);
        let layer = app.active_layer().expect("layer");
        let undo = undo_steps(&app);
        app.import_pdf_pages(&path, &pdf, &[1, 2], false);
        assert_eq!(app.doc().pages.len(), 2);
        let shapes = &app.doc().layer(layer).expect("layer").shapes;
        assert_eq!(shapes.len(), 2);
        assert!(shapes
            .iter()
            .all(|s| matches!(&s.kind, ShapeKind::Group { children } if children.len() >= 2)));
        assert_eq!(app.selection.len(), 2);
        assert_eq!(undo_steps(&app), undo + 1, "one undo step");
        // One page on the current page comes as its objects, ungrouped.
        app.undo();
        app.import_pdf_pages(&path, &pdf, &[2], false);
        let shapes = &app.doc().layer(layer).expect("layer").shapes;
        assert_eq!(shapes.len(), 3);
        // The PDF reader gives paths, not rectangles; what matters is
        // that no group was made around them.
        assert!(shapes
            .iter()
            .all(|s| !matches!(s.kind, ShapeKind::Group { .. })));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A PDF of one page comes in on the current page without asking.
    #[test]
    fn a_pdf_of_one_page_comes_in_without_asking() {
        let dir = std::env::temp_dir().join(format!("tracedraw-pdfpage1-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let mut src = tracedraw_core::Document::new("one", Size::new(100.0, 50.0));
        let mut ids = src.ids().clone();
        let s = Shape::new(
            ids.shape(),
            ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(10.0, 10.0, 20.0, 20.0),
                radius: 0.0,
                corners: None,
            },
        );
        src.pages[0].layers[0].shapes.push(s);
        src.set_ids(ids);
        let path = dir.join("one.pdf");
        std::fs::write(&path, tracedraw_io::pdf::document_to_pdf(&src)).expect("written");
        let mut app = App::headless();
        app.settings.formats.pdf_ask_text = false;
        app.import_pdf(&path);
        assert!(matches!(app.dialog, Dialog::None), "{:?}", app.dialog);
        assert_eq!(app.doc().pages.len(), 1);
        assert_eq!(app.selection.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
