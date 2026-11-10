//! Colour management of drawings: the ICC profiles found on this computer
//! (for the Colour Management dialog's lists), assigning or converting a
//! drawing's profiles, and the open and import policies. See
//! docs/behavior/colour-management.md.

use crate::app::App;
use tracedraw_core::icc::{ColorSpace, Intent, Profile, Transform};
use tracedraw_core::{Color, Command, Fill, Shape, ShapeKind};

#[cfg(test)]
#[path = "color_open_tests.rs"]
mod opening_tests;

#[cfg(test)]
#[path = "color_screen_tests.rs"]
mod screen_tests;

/// The colour spaces the dialog lists profiles for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Rgb,
    Cmyk,
    Gray,
}

/// One confirmation of the Document tab, including its intent and mode.
pub struct DocumentColorChange<'a> {
    pub rgb: &'a FoundOrName,
    pub cmyk: &'a FoundOrName,
    pub gray: &'a FoundOrName,
    pub convert: bool,
    pub change_profiles: bool,
    pub primary: tracedraw_core::document::PrimaryColorMode,
    pub intent: &'a str,
}

/// An ICC profile file found on this computer.
#[derive(Debug, Clone, PartialEq)]
pub struct FoundProfile {
    pub name: String,
    pub path: String,
    pub space: Space,
}

/// The folders where systems keep colour profiles.
fn profile_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<std::path::PathBuf> = vec![
        "/usr/share/color/icc".into(),
        "/usr/local/share/color/icc".into(),
        "/Library/ColorSync/Profiles".into(),
        "/System/Library/ColorSync/Profiles".into(),
        r"C:\Windows\System32\spool\drivers\color".into(),
    ];
    if let Some(home) = crate::files::home_dir() {
        dirs.push(home.join(".local/share/color/icc"));
        dirs.push(home.join(".local/share/icc"));
        dirs.push(home.join(".color/icc"));
        dirs.push(home.join("Library/ColorSync/Profiles"));
    }
    dirs
}

/// Every readable RGB, CMYK and grayscale profile in the system folders
/// (and their subfolders, two levels down), sorted by name, each name
/// once.
pub fn system_profiles() -> Vec<FoundProfile> {
    let mut out: Vec<FoundProfile> = Vec::new();
    fn walk(dir: &std::path::Path, depth: usize, out: &mut Vec<FoundProfile>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                if depth < 2 {
                    walk(&p, depth + 1, out);
                }
                continue;
            }
            let ext = p
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext != "icc" && ext != "icm" {
                continue;
            }
            let Ok(bytes) = std::fs::read(&p) else {
                continue;
            };
            let Ok(profile) = Profile::parse(&bytes) else {
                continue;
            };
            let space = match profile.color_space() {
                ColorSpace::Rgb => Space::Rgb,
                ColorSpace::Cmyk => Space::Cmyk,
                ColorSpace::Gray => Space::Gray,
                _ => continue,
            };
            let name = profile
                .description()
                .map(str::to_string)
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| {
                    p.file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                });
            out.push(FoundProfile {
                name,
                path: p.display().to_string(),
                space,
            });
        }
    }
    if crate::files::WEB {
        return out;
    }
    for d in profile_dirs() {
        walk(&d, 0, &mut out);
    }
    out.sort_by_key(|a| a.name.to_lowercase());
    out.dedup_by(|a, b| a.name == b.name && a.space == b.space);
    out
}

/// A named colour preset: the default profiles of a region.
pub struct Preset {
    pub key: &'static str,
    pub rgb: &'static str,
    pub cmyk: &'static str,
    pub gray: &'static str,
}

/// The built-in presets. Their profiles are used when a file of that name
/// is found on this computer; otherwise the name is kept and the built-in
/// formulas convert.
pub const PRESETS: [Preset; 4] = [
    Preset {
        key: "color_mgmt.preset_north_america",
        rgb: "sRGB IEC61966-2.1",
        cmyk: "U.S. Web Coated (SWOP) v2",
        gray: "Dot Gain 20%",
    },
    Preset {
        key: "color_mgmt.preset_europe",
        rgb: "sRGB IEC61966-2.1",
        cmyk: "Coated FOGRA39 (ISO 12647-2:2004)",
        gray: "Dot Gain 15%",
    },
    Preset {
        key: "color_mgmt.preset_japan",
        rgb: "sRGB IEC61966-2.1",
        cmyk: "Japan Color 2001 Coated",
        gray: "Dot Gain 15%",
    },
    Preset {
        key: "color_mgmt.preset_web",
        rgb: "sRGB IEC61966-2.1",
        cmyk: "Generic CMYK (open)",
        gray: "Gray Gamma 2.2",
    },
];

/// Every colour of a fill, changed by `f`.
pub fn map_fill(fill: &Fill, f: &dyn Fn(Color) -> Color) -> Fill {
    let mut fill = fill.clone();
    match &mut fill {
        Fill::Solid(c) => *c = f(*c),
        Fill::Fountain(g) => g.stops.iter_mut().for_each(|s| s.color = f(s.color)),
        Fill::Mesh(m) => m.nodes.iter_mut().for_each(|n| n.color = f(n.color)),
        _ => {}
    }
    fill
}

/// The commands that change every colour of `shapes` (and of the shapes
/// inside groups and clip frames) by `f`, for the shapes that change.
pub fn recolor_commands(shapes: &[Shape], f: &dyn Fn(Color) -> Color, out: &mut Vec<Command>) {
    for s in shapes {
        let fill = map_fill(&s.fill, f);
        if fill != s.fill {
            out.push(Command::SetFill {
                shapes: vec![s.id],
                fill,
            });
        }
        if let Some(st) = &s.stroke {
            let c = f(st.color);
            if c != st.color {
                let mut st = st.clone();
                st.color = c;
                out.push(Command::SetStroke {
                    shapes: vec![s.id],
                    stroke: Some(st),
                });
            }
        }
        match &s.kind {
            ShapeKind::Group { children } => recolor_commands(children, f, out),
            ShapeKind::ClipFrame { frame, contents } => {
                recolor_commands(std::slice::from_ref(frame.as_ref()), f, out);
                recolor_commands(contents, f, out);
            }
            _ => {}
        }
    }
}

/// Change the colours of `shape` (and its children) in place by `f`.
pub fn recolor_shape(shape: &mut Shape, f: &dyn Fn(Color) -> Color) {
    shape.fill = map_fill(&shape.fill, f);
    if let Some(st) = &mut shape.stroke {
        st.color = f(st.color);
    }
    match &mut shape.kind {
        ShapeKind::Group { children } => children.iter_mut().for_each(|c| recolor_shape(c, f)),
        ShapeKind::ClipFrame { frame, contents } => {
            recolor_shape(frame, f);
            contents.iter_mut().for_each(|c| recolor_shape(c, f));
        }
        _ => {}
    }
}

/// A conversion of CMYK colours from profile `from` to profile `to`
/// keeping their appearance; with `preserve_black`, colours made of black
/// ink only stay so. `None` when the profiles give no transform.
pub fn cmyk_converter(
    from: &Profile,
    to: &Profile,
    intent: Intent,
    preserve_black: bool,
) -> Option<impl Fn(Color) -> Color> {
    cmyk_converter_options(
        Some(from),
        to,
        tracedraw_io::picture_profile::Options {
            intent,
            black_point_compensation: true,
            preserve_pure_black: preserve_black,
        },
    )
}

fn cmyk_converter_options(
    from: Option<&Profile>,
    to: &Profile,
    options: tracedraw_io::picture_profile::Options,
) -> Option<impl Fn(Color) -> Color> {
    if to.color_space() != ColorSpace::Cmyk
        || from.is_some_and(|p| p.color_space() != ColorSpace::Cmyk)
    {
        return None;
    }
    let builtin = Profile::srgb();
    let source_cmyk = from.is_some();
    let t = Transform::new(
        from.unwrap_or(&builtin),
        to,
        options.intent,
        options.black_point_compensation,
    )?;
    Some(move |c: Color| match c {
        Color::Cmyk { c, m, y, k } => {
            if options.preserve_pure_black && c == 0.0 && m == 0.0 && y == 0.0 {
                return Color::Cmyk { c, m, y, k };
            }
            let v = if source_cmyk {
                t.transform(&[c, m, y, k])
            } else {
                t.transform(&Color::Cmyk { c, m, y, k }.to_rgb_f32_builtin())
            };
            if v.len() < 4 {
                return Color::Cmyk { c, m, y, k };
            }
            let q = |x: f32| (x.clamp(0.0, 1.0) * 1000.0).round() / 1000.0;
            Color::Cmyk {
                c: q(v[0]),
                m: q(v[1]),
                y: q(v[2]),
                k: q(v[3]),
            }
        }
        other => other,
    })
}

/// A profile from its file (`None` for an empty or unreadable path).
pub fn load(path: &str) -> Option<Profile> {
    if path.is_empty() {
        return None;
    }
    let bytes = crate::files::read(std::path::Path::new(path)).ok()?;
    Profile::parse(&bytes).ok()
}

fn named_profile(
    name: &str,
    path: &str,
) -> Result<Option<tracedraw_core::DocumentProfile>, String> {
    if path.is_empty() {
        return Ok(None);
    }
    let bytes = crate::files::read(std::path::Path::new(path)).map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("ICC profile exceeds its size budget".into());
    }
    let parsed = Profile::parse(&bytes).map_err(|e| e.to_string())?;
    if parsed.color_space() != ColorSpace::Cmyk {
        return Err("a CMYK destination profile is required".into());
    }
    Ok(Some(tracedraw_core::DocumentProfile::new(name, bytes)))
}

fn picture_options(color: &crate::settings::ColorPrefs) -> tracedraw_io::picture_profile::Options {
    tracedraw_io::picture_profile::Options {
        intent: Intent::from_name(&color.intent).unwrap_or_default(),
        black_point_compensation: color.black_point_compensation,
        preserve_pure_black: color.preserve_pure_black,
    }
}

fn reprofile_tree(
    shape: &mut Shape,
    inherited: Option<&tracedraw_core::DocumentProfile>,
    target: Option<&tracedraw_core::DocumentProfile>,
    convert: bool,
    options: tracedraw_io::picture_profile::Options,
) -> Result<(), String> {
    if let Some(updated) =
        tracedraw_io::picture_profile::reprofile(shape, inherited, target, convert, options)
            .map_err(|e| e.to_string())?
    {
        shape.kind = updated.kind;
        shape.picture_inks = updated.picture_inks;
    }
    match &mut shape.kind {
        ShapeKind::Group { children } => {
            for child in children {
                reprofile_tree(child, inherited, target, convert, options)?;
            }
        }
        ShapeKind::ClipFrame { frame, contents } => {
            reprofile_tree(frame, inherited, target, convert, options)?;
            for child in contents {
                reprofile_tree(child, inherited, target, convert, options)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Build every update before committing anything, including definitions outside
/// the page tree. Symbol instance indices stay stable.
fn profile_commands(
    doc: &tracedraw_core::Document,
    inherited: Option<&tracedraw_core::DocumentProfile>,
    target: Option<&tracedraw_core::DocumentProfile>,
    convert: bool,
    options: tracedraw_io::picture_profile::Options,
) -> Result<Vec<Command>, String> {
    let source = if convert {
        inherited
            .map(|p| Profile::parse(&p.data).map_err(|e| e.to_string()))
            .transpose()?
    } else {
        None
    };
    let destination = target
        .map(|p| Profile::parse(&p.data).map_err(|e| e.to_string()))
        .transpose()?;
    let same_working_profile = match (inherited, target) {
        (None, None) => true,
        (Some(a), Some(b)) => a.data == b.data,
        _ => false,
    };
    let converter = if convert && !same_working_profile {
        let to = destination
            .as_ref()
            .ok_or("CMYK conversion requires an available destination ICC profile")?;
        Some(
            cmyk_converter_options(source.as_ref(), to, options)
                .ok_or("CMYK conversion is unavailable")?,
        )
    } else {
        None
    };
    let change = |shape: &Shape| -> Result<Shape, String> {
        let mut updated = shape.clone();
        if let Some(f) = &converter {
            recolor_shape(&mut updated, f);
        }
        reprofile_tree(&mut updated, inherited, target, convert, options)?;
        Ok(updated)
    };
    let mut commands = Vec::new();
    for before in doc.all_layers().flat_map(|layer| &layer.shapes) {
        let after = change(before)?;
        if after.fill != before.fill {
            commands.push(Command::SetFill {
                shapes: vec![before.id],
                fill: after.fill,
            });
        }
        if after.stroke != before.stroke {
            commands.push(Command::SetStroke {
                shapes: vec![before.id],
                stroke: after.stroke,
            });
        }
        if after.kind != before.kind {
            commands.push(Command::SetShapeKind {
                shape: before.id,
                kind: after.kind,
            });
        }
        if after.picture_inks != before.picture_inks {
            commands.push(Command::SetPictureInks {
                shape: before.id,
                cmyk: after.cmyk_bitmap,
                inks: after.picture_inks,
            });
        }
    }
    for (index, symbol) in doc.symbols.iter().enumerate() {
        let shape = change(&symbol.shape)?;
        if shape != symbol.shape {
            commands.push(Command::SetSymbol {
                index,
                symbol: tracedraw_core::document::Symbol {
                    name: symbol.name.clone(),
                    shape,
                },
            });
        }
    }
    Ok(commands)
}

impl App {
    /// Assign or convert the active drawing's profiles. With `convert`,
    /// CMYK colours are converted from the drawing's CMYK profile to the
    /// new one so they look the same; otherwise the values stay and only
    /// the profiles change (one undo step).
    pub fn set_document_profiles(
        &mut self,
        rgb: &FoundOrName,
        cmyk: &FoundOrName,
        gray: &FoundOrName,
        convert: bool,
    ) {
        let intent = self.settings.color.intent.clone();
        self.apply_document_color(DocumentColorChange {
            rgb,
            cmyk,
            gray,
            convert,
            change_profiles: true,
            primary: self.doc().metadata.primary_color_mode,
            intent: &intent,
        });
    }

    pub fn apply_document_color(&mut self, change: DocumentColorChange<'_>) -> bool {
        let result = (|| -> Result<Vec<Command>, String> {
            let target = if change.change_profiles {
                if change.cmyk.path().is_none() {
                    self.doc()
                        .cmyk_profile
                        .as_ref()
                        .filter(|p| p.name == change.cmyk.name())
                        .cloned()
                } else {
                    named_profile(change.cmyk.name(), change.cmyk.path().unwrap_or(""))?
                }
            } else {
                self.doc().cmyk_profile.clone()
            };
            let mut commands = Vec::new();
            if change.change_profiles && (change.convert || target != self.doc().cmyk_profile) {
                let mut inherited = self.doc().cmyk_profile.clone();
                if change.convert && inherited.is_none() {
                    inherited = named_profile(
                        &self.settings.color.cmyk_profile,
                        &self.settings.color.cmyk_profile_path,
                    )?;
                }
                let mut options = picture_options(&self.settings.color);
                options.intent = Intent::from_name(change.intent).unwrap_or_default();
                commands = profile_commands(
                    self.doc(),
                    inherited.as_ref(),
                    target.as_ref(),
                    change.convert,
                    options,
                )?;
            }
            let mut metadata = self.doc().metadata.clone();
            if change.change_profiles {
                metadata.rgb_profile = change.rgb.name().to_string();
                metadata.cmyk_profile = change.cmyk.name().to_string();
                metadata.gray_profile = change.gray.name().to_string();
            }
            metadata.primary_color_mode = change.primary;
            metadata.rendering_intent = change.intent.to_string();
            if metadata != self.doc().metadata {
                commands.push(Command::SetMetadata { metadata });
            }
            if target != self.doc().cmyk_profile {
                commands.push(Command::SetCmykProfile { profile: target });
            }
            Ok(commands)
        })();
        let commands = match result {
            Ok(commands) => commands,
            Err(error) => {
                self.status = crate::i18n::trf("color_mgmt.apply_failed", &[("e", &error)]);
                return false;
            }
        };
        if !commands.is_empty() {
            if let Err(error) = self.engine.run_batch("Color Management", &commands) {
                self.status =
                    crate::i18n::trf("color_mgmt.apply_failed", &[("e", &error.to_string())]);
                return false;
            }
        }
        self.apply_color_settings();
        true
    }

    /// The open policy, right after a drawing opened: warnings for a
    /// missing or different CMYK profile, and Assign or Convert to the
    /// default profile.
    pub fn apply_open_policy(&mut self) -> Result<Option<String>, String> {
        let prepared = prepare_open_policy(self.doc(), &self.settings.color)?;
        if let Some(document) = prepared.document {
            self.engine.replace(document);
            self.apply_color_settings();
        }
        Ok(prepared.notice)
    }
}

/// A profile chosen in the dialog: one found on this computer, or a name
/// only (the built-in model, or a preset's profile not installed).
#[derive(Debug, Clone, PartialEq)]
pub enum FoundOrName {
    Found(FoundProfile),
    Name(String),
}

impl FoundOrName {
    pub fn name(&self) -> &str {
        match self {
            FoundOrName::Found(p) => &p.name,
            FoundOrName::Name(n) => n,
        }
    }

    pub fn path(&self) -> Option<&str> {
        match self {
            FoundOrName::Found(p) => Some(&p.path),
            FoundOrName::Name(_) => None,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tracedraw_core::icc::test_profiles as tp;

    pub(crate) fn bitmap_fixture() -> (tracedraw_core::Document, Vec<u8>, Vec<u8>, Vec<u8>) {
        let a = tp::lut16_cmyk_profile();
        let b = tp::intent_cmyk_profile(true);
        let samples = vec![
            17, 33, 49, 65, 0, 251, 127, 3, 9, 127, 0, 0, 0, 85, 255, 11, 29, 57, 113, 255,
        ];
        let bytes = tracedraw_io::cmyk_bitmap::cmyka_tiff(2, 2, &samples, 300.0, Some(&a)).unwrap();
        let mut doc =
            tracedraw_io::raster::document_from_bytes(&bytes, "profile operations").unwrap();
        let root = doc.pages[0].layers[0].shapes[0].clone();
        let mut child = root.clone();
        child.id = doc.ids_mut().shape();
        let group = Shape::new(
            doc.ids_mut().shape(),
            ShapeKind::Group {
                children: vec![child.clone()],
            },
        );
        doc.pages[0].layers[0].shapes.push(group);
        child.id = doc.ids_mut().shape();
        let frame = Shape::new(
            doc.ids_mut().shape(),
            ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(0.0, 0.0, 1.0, 1.0),
                radius: 0.0,
                corners: None,
            },
        );
        let clip = Shape::new(
            doc.ids_mut().shape(),
            ShapeKind::ClipFrame {
                frame: Box::new(frame),
                contents: vec![child.clone()],
            },
        );
        doc.pages[0].layers[0].shapes.push(clip);
        child.id = doc.ids_mut().shape();
        doc.symbols.push(tracedraw_core::document::Symbol {
            name: "Picture definition".into(),
            shape: child,
        });
        let mut master_bitmap = root.clone();
        master_bitmap.id = doc.ids_mut().shape();
        let mut master_layer =
            tracedraw_core::document::Layer::new(doc.ids_mut().layer(), "Master pictures");
        master_layer.shapes.push(master_bitmap);
        doc.master.push(master_layer);
        (doc, a, b, samples)
    }

    pub(crate) fn collect_bitmaps(doc: &tracedraw_core::Document) -> Vec<&Shape> {
        let mut pending: Vec<_> = doc
            .all_layers()
            .flat_map(|l| &l.shapes)
            .chain(doc.symbols.iter().map(|s| &s.shape))
            .collect();
        let mut bitmaps = Vec::new();
        while let Some(shape) = pending.pop() {
            match &shape.kind {
                ShapeKind::Bitmap { .. } => bitmaps.push(shape),
                ShapeKind::Group { children } => pending.extend(children),
                ShapeKind::ClipFrame { frame, contents } => {
                    pending.push(frame);
                    pending.extend(contents);
                }
                _ => {}
            }
        }
        bitmaps
    }

    pub(crate) fn decoded_samples(shape: &Shape) -> Vec<u8> {
        let tiff = tracedraw_io::picture::original_cmyk_tiff(shape, 300.0).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(tiff)).unwrap();
        let tiff::decoder::DecodingResult::U8(samples) = decoder.read_image().unwrap() else {
            panic!("samples");
        };
        samples
    }

    pub(crate) fn target_file(bytes: &[u8]) -> (std::path::PathBuf, FoundOrName) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../.build/profile-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("target.icc");
        std::fs::write(&path, bytes).unwrap();
        (
            root,
            FoundOrName::Found(FoundProfile {
                name: "Chosen CMYK".into(),
                path: path.display().to_string(),
                space: Space::Cmyk,
            }),
        )
    }

    #[test]
    fn assigning_and_converting_picture_profiles_is_one_undo_step_including_symbols() {
        for convert in [false, true] {
            let (doc, a, b, original_samples) = bitmap_fixture();
            let (root, target) = target_file(&b);
            let mut app = App::headless();
            app.engine.replace(doc);
            app.settings.color.cmyk_profile_path.clear();
            app.settings.color.preserve_pure_black = true;
            app.settings.color.intent = "Saturation".into();
            app.settings.color.black_point_compensation = false;
            let before = app.doc().clone();
            app.set_document_profiles(
                &FoundOrName::Name("sRGB".into()),
                &target,
                &FoundOrName::Name("Gray".into()),
                convert,
            );
            assert_eq!(app.engine.history_labels().0.len(), 1, "{}", app.status);
            assert_eq!(app.doc().cmyk_profile.as_ref().unwrap().data.as_slice(), b);
            let mut expected = original_samples.clone();
            if convert {
                let transform = Transform::new(
                    &Profile::parse(&a).unwrap(),
                    &Profile::parse(&b).unwrap(),
                    Intent::Saturation,
                    false,
                )
                .unwrap();
                for pixel in expected.chunks_exact_mut(5) {
                    if pixel[..3] == [0, 0, 0] {
                        continue;
                    }
                    let output = transform.transform(
                        &pixel[..4]
                            .iter()
                            .map(|v| f32::from(*v) / 255.0)
                            .collect::<Vec<_>>(),
                    );
                    for (byte, v) in pixel[..4].iter_mut().zip(output) {
                        *byte = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
                assert_ne!(expected, original_samples);
            }
            let bitmaps = collect_bitmaps(app.doc());
            assert_eq!(bitmaps.len(), 5);
            for bitmap in bitmaps {
                assert_eq!(decoded_samples(bitmap), expected);
                assert_eq!(
                    bitmap
                        .picture_inks
                        .as_ref()
                        .unwrap()
                        .profile
                        .as_ref()
                        .unwrap()
                        .data
                        .as_slice(),
                    b
                );
            }
            let saved = root.join("changed.tdraw");
            tracedraw_io::save_native(app.doc(), &saved).unwrap();
            tracedraw_io::save_native(&before, root.join("before.tdraw")).unwrap();
            std::fs::write(root.join("operation.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "convert":convert,"intent":"Saturation","black_point_compensation":false,
                "preserve_pure_black":true,"expected_samples":expected,"bitmap_count":5,
                "scope":"Document operation through the compiled headless application; original synthetic data"
            })).unwrap()).unwrap();
            let restored = tracedraw_io::load_native(&saved).unwrap();
            assert_eq!(restored, app.doc().clone());
            let changed = app.doc().clone();
            app.undo();
            assert_eq!(app.doc(), &before);
            app.redo();
            assert_eq!(app.doc(), &changed);
            assert_eq!(std::fs::read(root.join("target.icc")).unwrap(), b);
        }
    }

    #[test]
    fn an_invalid_profile_or_late_bad_picture_keeps_the_whole_document_and_history() {
        let (mut doc, _, b, _) = bitmap_fixture();
        let (root, target) = target_file(&b);
        let ShapeKind::Group { children } = &mut doc.pages[0].layers[0].shapes[1].kind else {
            panic!("group");
        };
        children[0].picture_inks.as_mut().unwrap().png = vec![1, 2, 3].into();
        let mut app = App::headless();
        app.engine.replace(doc);
        let before = app.doc().clone();
        app.set_document_profiles(
            &FoundOrName::Name("sRGB".into()),
            &target,
            &FoundOrName::Name("Gray".into()),
            false,
        );
        assert_eq!(app.doc(), &before);
        assert!(app.engine.history_labels().0.is_empty());
        assert!(!app.status.is_empty());
        let (valid, _, _, _) = bitmap_fixture();
        app.engine.replace(valid);
        let before = app.doc().clone();
        std::fs::write(root.join("target.icc"), b"invalid").unwrap();
        app.set_document_profiles(
            &FoundOrName::Name("sRGB".into()),
            &target,
            &FoundOrName::Name("Gray".into()),
            true,
        );
        assert_eq!(app.doc(), &before);
        assert!(app.engine.history_labels().0.is_empty());
    }

    #[test]
    fn converting_to_identical_icc_keeps_vector_values_and_original_picture_samples() {
        let (mut doc, a, _, samples) = bitmap_fixture();
        let mut vector = Shape::new(
            doc.ids_mut().shape(),
            ShapeKind::Rect {
                rect: tracedraw_core::Rect::new(0.0, 0.0, 1.0, 1.0),
                radius: 0.0,
                corners: None,
            },
        );
        let fill = Fill::Solid(Color::Cmyk {
            c: 0.123456,
            m: 0.234567,
            y: 0.345678,
            k: 0.456789,
        });
        vector.fill = fill.clone();
        let id = vector.id;
        doc.pages[0].layers[0].shapes.push(vector);
        let (_, target) = target_file(&a);
        let mut app = App::headless();
        app.engine.replace(doc);
        app.set_document_profiles(
            &FoundOrName::Name("sRGB".into()),
            &target,
            &FoundOrName::Name("Gray".into()),
            true,
        );
        assert_eq!(app.engine.history_labels().0.len(), 1, "{}", app.status);
        assert_eq!(app.doc().find_shape(id).unwrap().fill, fill);
        for bitmap in collect_bitmaps(app.doc()) {
            assert_eq!(decoded_samples(bitmap), samples);
        }
    }

    #[test]
    fn fills_and_outlines_of_nested_shapes_are_recoloured() {
        let mut ids = tracedraw_core::id::IdSource::default();
        let mut inner = Shape::new(
            ids.shape(),
            ShapeKind::Rect {
                rect: tracedraw_core::geometry::Rect::new(0.0, 0.0, 1.0, 1.0),
                radius: 0.0,
                corners: None,
            },
        );
        inner.fill = Fill::Solid(Color::cmyk_pct(0.0, 100.0, 0.0, 0.0));
        let group = Shape::new(
            ids.shape(),
            ShapeKind::Group {
                children: vec![inner.clone()],
            },
        );
        let to_red = |c: Color| match c {
            Color::Cmyk { .. } => Color::rgb8(255, 0, 0),
            o => o,
        };
        let mut cmds = Vec::new();
        recolor_commands(std::slice::from_ref(&group), &to_red, &mut cmds);
        assert_eq!(cmds.len(), 1);
        assert!(
            matches!(&cmds[0], Command::SetFill { shapes, fill: Fill::Solid(_) } if shapes == &vec![inner.id])
        );
        let mut g = group.clone();
        recolor_shape(&mut g, &to_red);
        let ShapeKind::Group { children } = &g.kind else {
            panic!("group");
        };
        assert_eq!(children[0].fill, Fill::Solid(Color::rgb8(255, 0, 0)));
    }

    #[test]
    fn cmyk_converts_between_profiles_and_pure_black_can_stay() {
        let p = Profile::parse(&tp::mba_cmyk_profile()).expect("profile");
        let f = cmyk_converter(&p, &p, Intent::RelativeColorimetric, true).expect("transform");
        // Black ink only stays as it is.
        let k = Color::Cmyk {
            c: 0.0,
            m: 0.0,
            y: 0.0,
            k: 1.0,
        };
        assert_eq!(f(k), k);
        // Other colours go through the profiles and stay CMYK.
        assert!(matches!(
            f(Color::cmyk_pct(20.0, 40.0, 60.0, 10.0)),
            Color::Cmyk { .. }
        ));
        // Non-CMYK colours are left alone.
        assert_eq!(f(Color::rgb8(1, 2, 3)), Color::rgb8(1, 2, 3));
    }

    #[test]
    fn presets_name_their_three_profiles() {
        for p in PRESETS {
            assert!(!p.rgb.is_empty() && !p.cmyk.is_empty() && !p.gray.is_empty());
        }
        assert_eq!(FoundOrName::Name("x".into()).path(), None);
    }
}

/// A transactional opening-policy result, safe to prepare on a worker.
pub(crate) struct PreparedOpenPolicy {
    pub(crate) document: Option<tracedraw_core::Document>,
    pub(crate) notice: Option<String>,
}

pub(crate) fn prepare_open_policy(
    doc: &tracedraw_core::Document,
    color: &crate::settings::ColorPrefs,
) -> Result<PreparedOpenPolicy, String> {
    if color.open_cmyk > 2 {
        return Err("unknown CMYK opening policy".into());
    }
    let carried = doc.cmyk_profile.clone();
    // Keeping a tagged file does not require a usable default profile unless
    // the user requested a comparison warning.
    if color.open_cmyk == 0 && carried.is_some() && !color.warn_open_mismatch {
        return Ok(PreparedOpenPolicy {
            document: None,
            notice: None,
        });
    }
    let target = named_profile(&color.cmyk_profile, &color.cmyk_profile_path)?;
    let notice = match (&carried, &target) {
        (None, _) if color.warn_open_missing => Some(crate::i18n::tr("color_mgmt.warn_missing")),
        (Some(a), Some(b)) if color.warn_open_mismatch && a.data != b.data => {
            Some(if a.name == b.name {
                crate::i18n::tr("color_mgmt.warn_mismatch_data")
            } else {
                crate::i18n::trf(
                    "color_mgmt.warn_mismatch",
                    &[("doc", &a.name), ("default", &b.name)],
                )
            })
        }
        (Some(a), None) if color.warn_open_mismatch && a.name != color.cmyk_profile => {
            Some(crate::i18n::trf(
                "color_mgmt.warn_mismatch",
                &[("doc", &a.name), ("default", &color.cmyk_profile)],
            ))
        }
        _ => None,
    };
    if color.open_cmyk == 0 && carried.is_some() {
        return Ok(PreparedOpenPolicy {
            document: None,
            notice,
        });
    }
    // Untagged input is interpreted in the chosen default space, not converted
    // from an unrelated open document or from an invented embedded profile.
    let convert = color.open_cmyk == 2 && carried.is_some();
    let commands = profile_commands(
        doc,
        carried.as_ref(),
        target.as_ref(),
        convert,
        picture_options(&color),
    )?;
    let mut opened = doc.clone();
    for command in commands {
        command.apply(&mut opened).map_err(|e| e.to_string())?;
    }
    opened.metadata.cmyk_profile = target
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| color.cmyk_profile.clone());
    opened.cmyk_profile = target;
    Ok(PreparedOpenPolicy {
        document: Some(opened),
        notice,
    })
}

/// Screen calibration is constructed off the UI thread, then installed once.
pub(crate) struct PreparedScreenColors {
    pub(crate) icc: Option<tracedraw_core::color::IccEngine>,
    pub(crate) profile_key: Option<usize>,
    pub(crate) document_intent_key: String,
    pub(crate) map_gray_to_black: bool,
}

pub(crate) fn prepare_screen_colors(
    doc: &tracedraw_core::Document,
    color: &crate::settings::ColorPrefs,
) -> PreparedScreenColors {
    let load = |path: &str| -> Option<Profile> {
        if path.is_empty() {
            return None;
        }
        crate::files::read(std::path::Path::new(path))
            .ok()
            .and_then(|bytes| Profile::parse(&bytes).ok())
    };
    let rgb = load(&color.rgb_profile_path);
    let carried = doc.cmyk_profile.clone();
    let cmyk = crate::app::carried_cmyk_profile(doc).or_else(|| load(&color.cmyk_profile_path));
    let icc = if color.builtin_engine || (rgb.is_none() && cmyk.is_none()) {
        None
    } else {
        Some(tracedraw_core::color::IccEngine::new(
            rgb,
            cmyk,
            Intent::from_name(&doc.metadata.rendering_intent)
                .or_else(|| Intent::from_name(&color.intent))
                .unwrap_or_default(),
            color.black_point_compensation,
        ))
    };
    PreparedScreenColors {
        icc,
        profile_key: carried.map(|p| std::sync::Arc::as_ptr(&p.data) as usize),
        document_intent_key: doc.metadata.rendering_intent.clone(),
        map_gray_to_black: color.map_gray_to_black,
    }
}

impl App {
    pub(crate) fn install_prepared_screen_colors(&mut self, colors: PreparedScreenColors) {
        use tracedraw_core::color::engine;
        self.color_profile_key = colors.profile_key;
        self.color_document_intent_key = colors.document_intent_key;
        engine::set_map_gray_to_black(colors.map_gray_to_black);
        match colors.icc {
            Some(icc) => engine::install(icc),
            None => engine::clear(),
        }
        self.raster.borrow_mut().invalidate();
    }
}
