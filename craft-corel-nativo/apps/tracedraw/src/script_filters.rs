//! Script filters: file formats read or written by JavaScript. A filter
//! script defines `TraceDrawFilter` (see `docs/behavior/script-filters.md`);
//! Scripts panel > Install as Filter keeps it in the settings, and File >
//! Import and File > Export offer its formats.

use crate::app::App;
use crate::i18n::trf;
use crate::scripting::{self, ScriptFilter};
use std::path::Path;

/// File text for a filter: UTF-8 without a byte order mark; bytes that
/// are not UTF-8 become replacement characters.
pub fn file_text(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    String::from_utf8_lossy(bytes).into_owned()
}

impl App {
    /// The installed filter that imports files with this extension.
    pub fn import_filter_for(&self, ext: &str) -> Option<ScriptFilter> {
        self.settings
            .script_filters
            .iter()
            .find(|f| f.import && f.handles(ext))
            .cloned()
    }

    /// Install the script as a filter, replacing one of the same name.
    pub fn install_script_filter(&mut self, src: &str) {
        match scripting::filter_info(src) {
            Ok(f) => {
                let name = f.name.clone();
                let filters = &mut self.settings.script_filters;
                match filters.iter_mut().find(|g| g.name == f.name) {
                    Some(g) => *g = f,
                    None => filters.push(f),
                }
                self.settings.save();
                self.status = trf("status.filter_installed", &[("name", &name)]);
            }
            Err(e) => {
                self.status = trf("status.filter_failed", &[("name", ""), ("e", &e)]);
            }
        }
    }

    pub fn remove_script_filter(&mut self, name: &str) {
        self.settings.script_filters.retain(|f| f.name != name);
        self.settings.save();
    }

    /// File > Import of a file a filter reads: its objects go to the
    /// active layer, selected, as one undo step.
    pub fn import_with_script(&mut self, path: &Path, filter: &ScriptFilter) {
        let text = match crate::files::read(path) {
            Ok(b) => file_text(&b),
            Err(e) => {
                self.status = trf("status.import_failed", &[("e", &e.to_string())]);
                return;
            }
        };
        match scripting::run_filter_import(self, filter, &text) {
            Ok(lines) => self.scripts_output.extend(lines),
            Err(e) => {
                self.status = trf("status.filter_failed", &[("name", &filter.name), ("e", &e)])
            }
        }
    }

    /// File > Export through a filter: the page, every page or the
    /// selection, as the dialog says.
    pub fn export_with_script(
        &mut self,
        path: &Path,
        filter: &ScriptFilter,
        st: &crate::ui::dialogs::ExportState,
    ) -> Result<String, String> {
        let (doc, _) = crate::export::export_document(self, st);
        let text = scripting::run_filter_export(doc, filter)?;
        crate::files::write(path, text).map_err(|e| e.to_string())?;
        Ok(trf(
            "status.exported",
            &[("path", &path.display().to_string())],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::document::ShapeKind;

    /// A format of one rectangle per line: "x y w h".
    const BOXES: &str = r#"
const TraceDrawFilter = {
  name: 'Boxes',
  extensions: ['.BOX', 'bx'],
  Import(text) {
    text.split('\n').filter(l => l.trim()).forEach(l => {
      const [x, y, w, h] = l.trim().split(/\s+/).map(Number);
      ActiveLayer.CreateRectangle(x, y, w, h);
    });
  },
  Export() {
    return ActivePage.Shapes.All.Shapes.map(s =>
      [s.PositionX, s.PositionY, s.SizeWidth, s.SizeHeight].map(v => v.toFixed(1)).join(' ')
    ).join('\n');
  },
};
"#;

    fn temp_file(name: &str, text: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("td-filter-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let p = dir.join(name);
        std::fs::write(&p, text).expect("write");
        p
    }

    fn shapes(app: &App) -> usize {
        app.doc().pages[0]
            .layers
            .iter()
            .map(|l| l.shapes.len())
            .sum()
    }

    #[test]
    fn a_filter_says_what_it_reads_and_writes() {
        let f = scripting::filter_info(BOXES).expect("filter");
        assert_eq!(f.name, "Boxes");
        assert_eq!(f.extensions, vec!["box", "bx"]);
        assert!(f.import && f.export);
        assert_eq!(f.label(), "Boxes (*.box, *.bx)");
        assert!(f.handles("BX"));
        // Scripts that are no filter.
        for bad in [
            "print('hi')",
            "const TraceDrawFilter = {extensions: ['a'], Import() {}};",
            "const TraceDrawFilter = {name: 'A', Import() {}};",
            "const TraceDrawFilter = {name: 'A', extensions: ['a']};",
            "const TraceDrawFilter = {name: 'A', extensions: ['a'], Import() {}}; while (true) {}",
            "syntax error here (",
        ] {
            assert!(scripting::filter_info(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn importing_runs_the_filter_as_one_step() {
        let mut app = App::headless();
        app.install_script_filter(BOXES);
        assert_eq!(app.settings.script_filters.len(), 1);
        // Installing again replaces it.
        app.install_script_filter(BOXES);
        assert_eq!(app.settings.script_filters.len(), 1);
        let before = shapes(&app);
        let p = temp_file("two.box", "\u{feff}10 20 30 40\n50 60 5 5\n");
        app.import_path(&p);
        assert_eq!(shapes(&app), before + 2);
        assert_eq!(app.selection.len(), 2);
        let b = app.selection_bounds().expect("bounds");
        assert!(
            (b.x0 - 10.0).abs() < 1e-6 && (b.x1 - 55.0).abs() < 1e-6,
            "{b:?}"
        );
        assert!(app
            .selected_shapes()
            .iter()
            .all(|s| matches!(s.kind, ShapeKind::Rect { .. })));
        app.undo();
        assert_eq!(shapes(&app), before);
        // A failing filter keeps nothing.
        let mut broken = app.settings.script_filters[0].clone();
        broken.source = broken.source.replace(
            "CreateRectangle(x, y, w, h);",
            "CreateRectangle(x, y, w, h); throw new Error('stop');",
        );
        app.settings.script_filters[0] = broken;
        app.import_path(&p);
        assert_eq!(shapes(&app), before);
        assert!(app.status.contains("stop"), "{}", app.status);
    }

    #[test]
    fn exporting_writes_what_the_filter_returns() {
        let mut app = App::headless();
        app.install_script_filter(BOXES);
        let f = app.settings.script_filters[0].clone();
        let p = temp_file("in.box", "10 20 30 40\n");
        app.import_path(&p);
        let n = shapes(&app);
        let out = p.with_file_name("out.box");
        let st = crate::ui::dialogs::ExportState::default();
        app.export_with_script(&out, &f, &st).expect("export");
        let text = std::fs::read_to_string(&out).expect("read");
        assert!(text.lines().any(|l| l == "10.0 20.0 30.0 40.0"), "{text}");
        // What the filter changes while exporting is dropped.
        let mut changing = f.clone();
        changing.source = changing.source.replace(
            "Export() {",
            "Export() { ActiveLayer.CreateEllipse(0, 0, 5, 5);",
        );
        app.export_with_script(&out, &changing, &st)
            .expect("export");
        assert_eq!(shapes(&app), n);
        // Export must give text.
        let mut silent = f;
        silent.source = silent.source.replace("return ActivePage", "ActivePage");
        assert!(app.export_with_script(&out, &silent, &st).is_err());
    }

    #[test]
    fn path_data_is_in_page_millimetres() {
        let src = r#"
const TraceDrawFilter = { name: 'Paths', extensions: ['pd'],
  Export() { return ActivePage.Shapes.All.Shapes.map(s => s.PathData).join('\n'); } };
"#;
        let f = scripting::filter_info(src).expect("filter");
        let mut app = App::headless();
        app.install_script_filter(BOXES);
        let p = temp_file("one.box", "10 20 30 40\n");
        app.import_path(&p);
        let out = p.with_file_name("one.pd");
        app.export_with_script(&out, &f, &crate::ui::dialogs::ExportState::default())
            .expect("export");
        let text = std::fs::read_to_string(&out).expect("read");
        assert!(text.starts_with('M'), "{text}");
        for v in ["10", "20", "40", "60"] {
            assert!(text.contains(v), "{v} in {text}");
        }
    }
}
