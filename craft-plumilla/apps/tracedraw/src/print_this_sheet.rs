//! Print Preview's Print This Sheet: only the sheet the preview shows goes
//! to the printer or the file (see docs/behavior/print.md).

use crate::app::App;
use crate::export::print_sheets;
use crate::ui::dialogs::PrintState;
use tracedraw_core::Document;

/// The sheets Print sends: all of them, or with Print This Sheet
/// (`only_sheet`) the one the preview shows (the last one when the layout
/// has since made fewer).
pub fn sheets_to_print(app: &App, st: &PrintState) -> Document {
    let mut doc = print_sheets(app, st);
    if let Some(k) = st.only_sheet {
        if !doc.pages.is_empty() {
            let page = doc.pages.remove(k.min(doc.pages.len() - 1));
            doc.pages = vec![page];
        }
    }
    doc
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{geometry::Rect, ShapeKind};

    /// Three pages, a square on each at its own place.
    fn three_pages() -> App {
        let mut app = App::headless();
        for k in 0..3 {
            if k > 0 {
                app.insert_page(true);
            }
            app.new_shape(ShapeKind::Rect {
                rect: Rect::new(10.0 * k as f64, 0.0, 10.0 * k as f64 + 5.0, 5.0),
                radius: 0.0,
                corners: None,
            })
            .expect("square");
        }
        app
    }

    #[test]
    fn print_this_sheet_sends_only_the_sheet_shown() {
        let app = three_pages();
        let all = sheets_to_print(&app, &PrintState::default());
        assert_eq!(all.pages.len(), 3);
        let st = PrintState {
            only_sheet: Some(1),
            ..PrintState::default()
        };
        let one = sheets_to_print(&app, &st);
        assert_eq!(one.pages.len(), 1);
        assert_eq!(one.pages[0], all.pages[1]);
        // A sheet past the end (the layout changed): the last one.
        let st = PrintState {
            only_sheet: Some(9),
            ..PrintState::default()
        };
        let last = sheets_to_print(&app, &st);
        assert_eq!(last.pages.len(), 1);
        assert_eq!(last.pages[0], all.pages[2]);
    }

    /// The pieces are connected: Print builds its sheets here, and the
    /// preview's button names the sheet shown for that one print.
    #[test]
    fn print_goes_through_the_sheet_chosen() {
        let export = include_str!("export.rs");
        let body = export
            .split("pub fn print(app: &mut App, st: &PrintState)")
            .nth(1)
            .expect("export::print");
        let head = body.lines().take(3).collect::<Vec<_>>().join("\n");
        assert!(head.contains("print_this_sheet::sheets_to_print(app, st)"), "{head}");
        let preview = include_str!("ui/print_preview.rs");
        assert!(preview.contains("pv.st.only_sheet = print_one.then_some(pv.sheet);"));
        assert!(preview.contains("pv.st.only_sheet = None;"));
    }
}
