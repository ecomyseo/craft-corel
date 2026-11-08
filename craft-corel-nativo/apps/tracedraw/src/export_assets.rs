//! The Export docker: a list of assets (whole pages or groups of chosen
//! objects), each with its own name, file format and resolution, exported
//! together to one folder (in a browser, each one downloaded).

use crate::app::App;
use crate::ui::dialogs::{ExportState, EXPORT_FORMATS};
use std::path::{Path, PathBuf};
use tracedraw_core::{PageId, ShapeId};

/// What an asset exports.
#[derive(Debug, Clone, PartialEq)]
pub enum AssetSource {
    Page(PageId),
    /// Objects of a page, cropped to their bounds.
    Objects {
        page: PageId,
        shapes: Vec<ShapeId>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportAsset {
    pub name: String,
    pub source: AssetSource,
    /// Index into `EXPORT_FORMATS`.
    pub format: usize,
    /// Resolution of raster formats (dpi).
    pub dpi: f64,
    /// JPEG and TIFF colour mode, an index into `EXPORT_COLOR_MODES`
    /// (#1763).
    pub color_mode: usize,
    /// Exported by Export; unticked assets stay in the list.
    pub checked: bool,
}

/// The docker's list and where the files go.
#[derive(Debug, Clone, Default)]
pub struct ExportAssets {
    pub assets: Vec<ExportAsset>,
    pub folder: Option<PathBuf>,
    /// Format, resolution and colour mode new assets start with.
    pub format: usize,
    pub dpi: f64,
    pub color_mode: usize,
}

impl ExportAssets {
    pub fn new() -> Self {
        ExportAssets {
            assets: Vec::new(),
            folder: None,
            format: EXPORT_FORMATS
                .iter()
                .position(|(_, e)| *e == "png")
                .unwrap_or(0),
            dpi: 150.0,
            color_mode: 0,
        }
    }
}

/// A file name from an asset name: characters that are not allowed in
/// file names become `_`, and an empty name becomes "asset".
pub fn file_stem(name: &str) -> String {
    let s: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let s = s.trim_matches('.').trim().to_string();
    if s.is_empty() {
        "asset".into()
    } else {
        s
    }
}

/// The paths the checked assets are written to: `folder/name.ext`, with
/// " (2)", " (3)" and so on when two assets would share a file.
pub fn target_paths(assets: &[ExportAsset], folder: &Path) -> Vec<(usize, PathBuf)> {
    let mut taken: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for (i, a) in assets.iter().enumerate().filter(|(_, a)| a.checked) {
        let ext = EXPORT_FORMATS
            .get(a.format)
            .map(|(_, e)| *e)
            .unwrap_or("png");
        let stem = file_stem(&a.name);
        let mut name = format!("{stem}.{ext}");
        let mut n = 2;
        while taken.iter().any(|t| t.eq_ignore_ascii_case(&name)) {
            name = format!("{stem} ({n}).{ext}");
            n += 1;
        }
        taken.push(name.clone());
        out.push((i, folder.join(name)));
    }
    out
}

impl App {
    /// Add every page as an asset (pages already listed are skipped).
    pub fn add_page_assets(&mut self) {
        let pages: Vec<(PageId, String)> = self
            .doc()
            .pages
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let name = if p.name.trim().is_empty() {
                    crate::i18n::trf("doc.page_n", &[("n", &(i + 1).to_string())])
                } else {
                    p.name.clone()
                };
                (p.id, name)
            })
            .collect();
        let (format, dpi, color_mode) = (
            self.export_assets.format,
            self.export_assets.dpi,
            self.export_assets.color_mode,
        );
        for (id, name) in pages {
            if self
                .export_assets
                .assets
                .iter()
                .any(|a| a.source == AssetSource::Page(id))
            {
                continue;
            }
            self.export_assets.assets.push(ExportAsset {
                name,
                source: AssetSource::Page(id),
                format,
                dpi,
                color_mode,
                checked: true,
            });
        }
    }

    /// Add the selection as one asset, named after the object when there
    /// is one.
    pub fn add_selection_asset(&mut self) {
        if self.selection.is_empty() {
            return;
        }
        let shapes = self.selection.clone();
        let name = match shapes.as_slice() {
            [one] => self
                .doc()
                .find_shape(*one)
                .and_then(|s| s.name.clone())
                .unwrap_or_else(|| crate::i18n::tr("export_docker.object")),
            _ => crate::i18n::tr("export_docker.selection"),
        };
        let n = self.export_assets.assets.len() + 1;
        self.export_assets.assets.push(ExportAsset {
            name: format!("{name} {n}"),
            source: AssetSource::Objects {
                page: self.page,
                shapes,
            },
            format: self.export_assets.format,
            dpi: self.export_assets.dpi,
            color_mode: self.export_assets.color_mode,
            checked: true,
        });
    }

    /// Assets whose page or objects are gone are dropped; objects that
    /// went are left out of their asset.
    pub fn prune_export_assets(&mut self) {
        let doc = self.engine.document();
        let pages: Vec<PageId> = doc.pages.iter().map(|p| p.id).collect();
        let mut assets = std::mem::take(&mut self.export_assets.assets);
        assets.retain_mut(|a| match &mut a.source {
            AssetSource::Page(p) => pages.contains(p),
            AssetSource::Objects { page, shapes } => {
                shapes.retain(|id| doc.find_shape(*id).is_some());
                pages.contains(page) && !shapes.is_empty()
            }
        });
        self.export_assets.assets = assets;
    }

    /// Export every checked asset to `folder`; the number written and the
    /// errors met (asset name and message).
    pub fn export_assets_to(&mut self, folder: &Path) -> (usize, Vec<(String, String)>) {
        self.prune_export_assets();
        let targets = target_paths(&self.export_assets.assets, folder);
        let (page, selection) = (self.page, self.selection.clone());
        let mut written = 0;
        let mut errors = Vec::new();
        for (i, path) in targets {
            let Some(asset) = self.export_assets.assets.get(i).cloned() else {
                continue;
            };
            let mut st = ExportState {
                format: asset.format,
                dpi: asset.dpi.max(crate::ui::dialogs::MIN_EXPORT_DPI),
                color_mode: asset.color_mode,
                ..ExportState::default()
            };
            match &asset.source {
                AssetSource::Page(p) => {
                    self.page = *p;
                    self.selection.clear();
                }
                AssetSource::Objects { page, shapes } => {
                    self.page = *page;
                    self.selection = shapes.clone();
                    st.selection_only = true;
                }
            }
            match crate::export::export(self, &path, &st) {
                Ok(_) => written += 1,
                Err(e) => errors.push((asset.name.clone(), e)),
            }
        }
        self.page = page;
        self.selection = selection;
        (written, errors)
    }

    /// The docker's Export button: into the chosen folder (asked for when
    /// none is set yet) on the desktop; in a browser and on Android, where
    /// no folder can be picked, as one zip file, downloaded or saved where
    /// the system's picker says (#1399).
    pub fn export_assets(&mut self) {
        let folder = match crate::files::FolderUse::ExportAssets.action() {
            crate::files::FolderAction::PickFolder(token) => self
                .export_assets
                .folder
                .clone()
                .or_else(|| crate::files::pick_folder(token)),
            crate::files::FolderAction::SaveZip => {
                self.export_assets_through_picker();
                return;
            }
            crate::files::FolderAction::OpenFiles | crate::files::FolderAction::Unavailable(_) => {
                None
            }
        };
        let Some(folder) = folder else {
            return;
        };
        self.export_assets.folder = Some(folder.clone());
        let (written, errors) = self.export_assets_to(&folder);
        self.status = export_status(written, &errors);
    }

    /// The name of the zip file of the assets: the drawing's.
    fn export_assets_zip_name(&self) -> String {
        let stem = self
            .file
            .as_ref()
            .and_then(|f| f.file_stem())
            .map(|s| s.to_string_lossy().to_string())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "assets".into());
        crate::files::safe_file_name(&stem)
    }

    /// Every checked asset in one zip file, under a folder called `top`:
    /// the zip's bytes, the number written and the errors met.
    pub fn export_assets_zip(
        &mut self,
        top: &str,
    ) -> Result<(Vec<u8>, usize, Vec<(String, String)>), String> {
        let mut result = (0, Vec::new());
        let zip = crate::files::zip_written(top, |dir| result = self.export_assets_to(dir))?;
        Ok((zip, result.0, result.1))
    }

    /// The assets as one zip file: the save picker (Android) or the
    /// download (browser) names it after the drawing, and it is made and
    /// written once the place is known.
    fn export_assets_through_picker(&mut self) {
        if !self.export_assets.assets.iter().any(|a| a.checked) {
            return;
        }
        let name = self.export_assets_zip_name();
        crate::files::Dialog::new()
            .add_filter("ZIP", &["zip"])
            .set_file_name(format!("{name}.zip"))
            .save_file(self, move |app, path| {
                app.status = match app.export_assets_zip(&name).and_then(|(zip, n, errors)| {
                    crate::files::write(&path, zip)
                        .map(|()| (n, errors))
                        .map_err(|e| e.to_string())
                }) {
                    Ok((n, errors)) => export_status(n, &errors),
                    Err(e) => e,
                };
            });
    }
}

/// The status line after exporting assets: how many were written, or the
/// first error.
fn export_status(written: usize, errors: &[(String, String)]) -> String {
    match errors.first() {
        None => crate::i18n::trf("export_docker.done", &[("n", &written.to_string())]),
        Some((name, e)) => format!("{name}: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_safe_and_never_collide() {
        assert_eq!(file_stem(" Cover: front/back? "), "Cover_ front_back_");
        assert_eq!(file_stem("..."), "asset");
        let a = |name: &str, format: usize, checked: bool| ExportAsset {
            name: name.into(),
            source: AssetSource::Page(PageId(1)),
            format,
            dpi: 72.0,
            color_mode: 0,
            checked,
        };
        let png = EXPORT_FORMATS
            .iter()
            .position(|(_, e)| *e == "png")
            .unwrap();
        let pdf = EXPORT_FORMATS
            .iter()
            .position(|(_, e)| *e == "pdf")
            .unwrap();
        let list = [
            a("Logo", png, true),
            a("logo", png, true),
            a("Logo", pdf, true),
            a("Skipped", png, false),
        ];
        let paths = target_paths(&list, Path::new("out"));
        let names: Vec<String> = paths
            .iter()
            .map(|(_, p)| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, ["Logo.png", "logo (2).png", "Logo.pdf"]);
        assert_eq!(paths.iter().map(|(i, _)| *i).collect::<Vec<_>>(), [0, 1, 2]);
    }

    /// In a browser and on Android (#1399) the checked assets go out as
    /// one zip file named after the drawing, each under its folder.
    #[test]
    fn assets_export_as_one_zip_where_no_folder_can_be_picked() {
        let mut app = App::headless();
        app.add_page();
        app.export_assets = ExportAssets::new();
        app.add_page_assets();
        let svg = EXPORT_FORMATS
            .iter()
            .position(|(_, e)| *e == "svg")
            .unwrap();
        for a in &mut app.export_assets.assets {
            a.format = svg;
        }
        app.export_assets.assets[1].checked = false;
        assert_eq!(app.export_assets_zip_name(), "assets");
        app.file = Some(PathBuf::from("/drawings/Flyer.tdraw"));
        assert_eq!(app.export_assets_zip_name(), "Flyer");
        let (zip, written, errors) = app.export_assets_zip("Flyer").unwrap();
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(written, 1);
        let archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).unwrap();
        let names: Vec<String> = archive
            .file_names()
            .map(|n| n.unwrap().to_string())
            .collect();
        assert_eq!(names, ["Flyer/Page 1.svg"]);
        assert_eq!(
            export_status(written, &errors),
            crate::i18n::trf("export_docker.done", &[("n", "1")])
        );
    }

    #[test]
    fn pages_and_a_selection_export_together_to_one_folder() {
        let mut app = App::headless();
        app.add_page();
        let layer = app.active_layer().expect("a layer");
        let id = app.engine.new_shape_id();
        let mut shape = tracedraw_core::Shape::new(
            id,
            tracedraw_core::ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(10.0, 10.0, 40.0, 30.0),
                radius: 0.0,
                corners: None,
            },
        );
        shape.name = Some("Badge".into());
        app.run(tracedraw_core::Command::AddShape { layer, shape });
        app.select(vec![id]);
        app.export_assets = ExportAssets::new();
        app.add_page_assets();
        app.add_page_assets();
        assert_eq!(app.export_assets.assets.len(), 2, "pages are listed once");
        app.add_selection_asset();
        assert_eq!(app.export_assets.assets[2].name, "Badge 3");
        let svg = EXPORT_FORMATS
            .iter()
            .position(|(_, e)| *e == "svg")
            .unwrap();
        app.export_assets.assets[0].format = svg;
        app.export_assets.assets[1].checked = false;
        let dir = std::env::temp_dir().join(format!("td-assets-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (page, sel) = (app.page, app.selection.clone());
        let (written, errors) = app.export_assets_to(&dir);
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(written, 2);
        assert!(dir.join("Page 1.svg").is_file());
        let png = image::open(dir.join("Badge 3.png")).unwrap();
        // The selection asset is cropped to the rectangle: 30 x 20 mm.
        let ratio = png.width() as f64 / png.height() as f64;
        assert!((ratio - 1.5).abs() < 0.05, "{ratio}");
        assert!(!dir.join("Page 2.png").exists());
        // A CMYK JPEG asset is written in CMYK; new assets start with
        // the colour mode of the docker's row (#1763).
        app.export_assets.format = EXPORT_FORMATS
            .iter()
            .position(|(_, e)| *e == "jpg")
            .unwrap();
        app.export_assets.color_mode = 1;
        app.export_assets.assets.clear();
        app.add_selection_asset();
        assert_eq!(app.export_assets.assets[0].color_mode, 1);
        let (written, errors) = app.export_assets_to(&dir);
        assert!(errors.is_empty() && written == 1, "{errors:?}");
        let jpg = std::fs::read(dir.join("Badge 1.jpg")).unwrap();
        assert!(jpg.windows(5).any(|w| w == b"Adobe"), "CMYK JPEG");
        app.export_assets.assets[0].color_mode = 0;
        app.export_assets_to(&dir);
        let jpg = std::fs::read(dir.join("Badge 1.jpg")).unwrap();
        assert!(!jpg.windows(5).any(|w| w == b"Adobe"), "RGB JPEG");
        app.select(vec![id]);
        app.add_page_assets();
        // The current page and selection are as they were.
        assert_eq!((app.page, app.selection.clone()), (page, sel));
        // A deleted object drops its asset.
        app.run(tracedraw_core::Command::DeleteShapes { shapes: vec![id] });
        app.prune_export_assets();
        assert_eq!(app.export_assets.assets.len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
