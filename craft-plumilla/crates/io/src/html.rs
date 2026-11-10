//! HTML export: one self-contained page with every document page as
//! inline SVG, scaled to the window, with page tabs when there is more
//! than one. No external files, no scripts beyond the tab switch, so the
//! file opens anywhere and can be dropped into a site as is.

use crate::svg::{page_to_svg_with_rollovers, ROLLOVER_CSS};
use std::fmt::Write as _;
use tracedraw_core::Document;

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The whole document as an HTML page.
pub fn document_to_html(doc: &Document) -> String {
    let title = if doc.title.trim().is_empty() {
        "Drawing".to_string()
    } else {
        doc.title.clone()
    };
    let mut out = String::new();
    let _ = writeln!(out, "<!doctype html>");
    let _ = writeln!(out, "<html lang=\"en\">");
    let _ = writeln!(out, "<head>");
    let _ = writeln!(out, "<meta charset=\"utf-8\">");
    let _ = writeln!(
        out,
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
    );
    let _ = writeln!(out, "<title>{}</title>", escape(&title));
    let _ = writeln!(out, "<meta name=\"generator\" content=\"TraceDraw\">");
    let _ = writeln!(
        out,
        "<style>\n\
         :root {{ color-scheme: light dark; }}\n\
         body {{ margin: 0; background: #e9e9e9; font: 14px system-ui, sans-serif; color: #222; }}\n\
         @media (prefers-color-scheme: dark) {{ body {{ background: #2b2b2b; color: #ddd; }} }}\n\
         nav {{ display: flex; gap: 4px; padding: 8px 16px; flex-wrap: wrap; }}\n\
         nav button {{ padding: 4px 10px; border: 1px solid #9995; border-radius: 4px; background: #fff2; cursor: pointer; font: inherit; color: inherit; }}\n\
         nav button[aria-selected=true] {{ background: #3b82f6; color: #fff; border-color: #3b82f6; }}\n\
         main {{ display: flex; flex-direction: column; align-items: center; gap: 24px; padding: 16px; }}\n\
         .page {{ background: #fff; box-shadow: 0 2px 12px #0004; max-width: 100%; }}\n\
         .page svg {{ display: block; width: 100%; height: auto; }}\n\
         .page[hidden] {{ display: none; }}\n\
         {ROLLOVER_CSS}</style>"
    );
    let _ = writeln!(out, "</head>");
    let _ = writeln!(out, "<body>");
    let multi = doc.pages.len() > 1;
    if multi {
        let _ = writeln!(out, "<nav role=\"tablist\" aria-label=\"Pages\">");
        for (i, p) in doc.pages.iter().enumerate() {
            let _ = writeln!(
                out,
                "<button role=\"tab\" aria-selected=\"{}\" data-page=\"{i}\">{}</button>",
                i == 0,
                escape(&p.name)
            );
        }
        let _ = writeln!(out, "</nav>");
    }
    let _ = writeln!(out, "<main>");
    for (i, p) in doc.pages.iter().enumerate() {
        let svg = page_to_svg_with_rollovers(doc, i);
        // Drop the XML prolog if any; inline SVG needs none.
        let svg = svg.trim_start_matches("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        let _ = writeln!(
            out,
            "<section class=\"page\" id=\"page-{i}\" data-name=\"{}\" style=\"width:{}mm\"{}>",
            escape(&p.name),
            fmt(p.size.width),
            if multi && i > 0 { " hidden" } else { "" }
        );
        out.push_str(svg.trim());
        out.push('\n');
        let _ = writeln!(out, "</section>");
    }
    let _ = writeln!(out, "</main>");
    if multi {
        let _ = writeln!(
            out,
            "<script>\n\
             document.querySelectorAll('nav button').forEach(function (b) {{\n\
               b.addEventListener('click', function () {{\n\
                 document.querySelectorAll('nav button').forEach(function (x) {{ x.setAttribute('aria-selected', x === b); }});\n\
                 document.querySelectorAll('.page').forEach(function (p) {{ p.hidden = p.id !== 'page-' + b.dataset.page; }});\n\
               }});\n\
             }});\n\
             </script>"
        );
    }
    let _ = writeln!(out, "</body>");
    let _ = writeln!(out, "</html>");
    out
}

fn fmt(v: f64) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::{
        document::{Page, ShapeKind},
        geometry::{Rect, Size},
        Color, Fill, Shape,
    };

    #[test]
    fn two_pages_give_tabs_and_inline_svg() {
        let mut doc = Document::new("Site & Co", Size::new(100.0, 50.0));
        let mut ids = doc.ids().clone();
        let mut r = Shape::new(
            ids.shape(),
            ShapeKind::Rect {
                rect: Rect::new(10.0, 10.0, 40.0, 30.0),
                radius: 0.0,
                corners: None,
            },
        );
        r.fill = Fill::Solid(Color::rgb8(0, 128, 0));
        doc.pages[0].layers[0].shapes.push(r);
        let pid = ids.page();
        let lid = ids.layer();
        doc.pages.push(Page {
            id: pid,
            name: "Back".into(),
            size: Size::new(100.0, 50.0),
            layers: vec![tracedraw_core::document::Layer::new(lid, "Layer 1")],
            guides: Vec::new(),
            background: None,
        });
        doc.set_ids(ids);
        let html = document_to_html(&doc);
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("<title>Site &amp; Co</title>"));
        assert_eq!(html.matches("<svg").count(), 2);
        assert_eq!(html.matches("role=\"tab\"").count(), 2);
        assert!(html.contains("id=\"page-1\"") && html.contains(" hidden>"));
        assert!(
            html.contains("#008000")
                || html.contains("rgb(0, 128, 0)")
                || html.contains("fill=\"#008000\"")
        );
        assert!(!html.contains("<?xml"));
    }

    /// A rollover goes to the page with its three states, Normal shown
    /// and the style sheet that swaps them under the pointer; its link
    /// opens in its target. A plain SVG keeps only the state shown.
    #[test]
    fn a_rollover_brings_its_three_states_to_the_page() {
        let mut doc = Document::new("r", Size::new(100.0, 50.0));
        let layer = doc.pages[0].layers[0].id;
        let mut ids = Vec::new();
        for (x, c) in [(10.0, Color::rgb8(200, 0, 0)), (50.0, Color::rgb8(0, 0, 200))] {
            let id = doc.ids_mut().shape();
            let mut s = Shape::new(
                id,
                ShapeKind::Rect {
                    rect: Rect::new(x, 10.0, x + 30.0, 30.0),
                    radius: 0.0,
                    corners: None,
                },
            );
            s.fill = Fill::Solid(c);
            doc.layer_mut(layer).expect("layer").shapes.push(s);
            ids.push(id);
        }
        tracedraw_core::Command::CreateRollover { shapes: ids }
            .apply(&mut doc)
            .expect("rollover");
        let r = doc.pages[0].layers[0].shapes[0].id;
        tracedraw_core::Command::SetRolloverState { shape: r, state: 1 }
            .apply(&mut doc)
            .expect("over");
        let first_over = match &doc.find_shape(r).expect("r").kind {
            ShapeKind::Group { children } => children[0].id,
            _ => panic!("a group"),
        };
        tracedraw_core::Command::SetFill {
            shapes: vec![first_over],
            fill: Fill::Solid(Color::rgb8(0, 160, 0)),
        }
        .apply(&mut doc)
        .expect("fill");
        tracedraw_core::Command::SetRolloverState { shape: r, state: 0 }
            .apply(&mut doc)
            .expect("normal");
        tracedraw_core::Command::SetLink {
            shapes: vec![r],
            link: Some("https://example.org/".into()),
        }
        .apply(&mut doc)
        .expect("link");
        tracedraw_core::Command::SetRolloverTarget {
            shape: r,
            target: "_blank".into(),
        }
        .apply(&mut doc)
        .expect("target");
        let html = document_to_html(&doc);
        assert!(html.contains("class=\"ro\""));
        for k in 0..3 {
            assert!(html.contains(&format!("class=\"ro-{k}\"")), "state {k}");
        }
        assert!(html.contains(".ro:hover > .ro-1"));
        assert!(html.contains("target=\"_blank\""));
        // The green of Over is inside the Over state, not in Normal.
        let over = html.split("class=\"ro-1\"").nth(1).expect("over");
        let over = over.split("class=\"ro-2\"").next().expect("over part");
        let normal = html.split("class=\"ro-0\"").nth(1).expect("normal");
        let normal = normal.split("class=\"ro-1\"").next().expect("normal part");
        let green = |s: &str| s.contains("#00a000") || s.contains("rgb(0, 160, 0)");
        assert!(green(over) && !green(normal), "{over}");
        // A plain SVG: the state shown only.
        let svg = crate::svg::page_to_svg(&doc, 0);
        assert!(!svg.contains("class=\"ro\""));
        assert!(!green(&svg));
    }

    #[test]
    fn one_page_has_no_tabs() {
        let doc = Document::new("", Size::new(10.0, 10.0));
        let html = document_to_html(&doc);
        assert!(!html.contains("<nav"));
        assert!(html.contains("<title>Drawing</title>"));
    }
}
