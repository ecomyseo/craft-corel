//! The screenshots of the Help: each dialog, options page and docker
//! drawn headlessly in English on a small demo drawing, rasterised in
//! software from egui's meshes and cropped to the window.
//!
//! `web/help_shots.sh` runs `cargo test -p tracedraw help_shots -- --ignored`
//! with `HELP_SHOTS` set to the output folder (`docs/help/img`). The
//! images are named after the Help topic they illustrate (see
//! docs/behavior/help.md).

use crate::app::{App, DockerTab};
use crate::ui::dialogs::{Dialog, OptionsPage};
use egui::epaint::{Color32, Primitive, TextureId, Vertex};
use std::collections::HashMap;

/// A premultiplied RGBA canvas in 0..1.
struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

struct Texture {
    w: usize,
    h: usize,
    px: Vec<Color32>,
}

#[derive(Default)]
struct Textures(HashMap<TextureId, Texture>);

impl Textures {
    fn apply(&mut self, delta: &egui::TexturesDelta) {
        for (id, ds) in &delta.set {
            for d in ds.iter() {
                let egui::ImageData::Color(img) = &d.image;
                let [w, h] = img.size;
                match d.pos {
                    None => {
                        self.0.insert(
                            *id,
                            Texture {
                                w,
                                h,
                                px: img.pixels.clone(),
                            },
                        );
                    }
                    Some([x0, y0]) => {
                        if let Some(t) = self.0.get_mut(id) {
                            for y in 0..h {
                                for x in 0..w {
                                    if x0 + x < t.w && y0 + y < t.h {
                                        t.px[(y0 + y) * t.w + x0 + x] = img.pixels[y * w + x];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn sample(&self, id: TextureId, u: f32, v: f32) -> [f32; 4] {
        let Some(t) = self.0.get(&id) else {
            return [1.0; 4];
        };
        let fx = (u * t.w as f32 - 0.5).clamp(0.0, (t.w - 1) as f32);
        let fy = (v * t.h as f32 - 0.5).clamp(0.0, (t.h - 1) as f32);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(t.w - 1), (y0 + 1).min(t.h - 1));
        let (ax, ay) = (fx - x0 as f32, fy - y0 as f32);
        let c = |x: usize, y: usize| t.px[y * t.w + x].to_array().map(|b| b as f32 / 255.0);
        let (a, b, cc, d) = (c(x0, y0), c(x1, y0), c(x0, y1), c(x1, y1));
        let mut out = [0.0; 4];
        for i in 0..4 {
            let top = a[i] + (b[i] - a[i]) * ax;
            let bot = cc[i] + (d[i] - cc[i]) * ax;
            out[i] = top + (bot - top) * ay;
        }
        out
    }
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Canvas {
            w,
            h,
            px: vec![[0.0; 4]; w * h],
        }
    }

    fn triangle(&mut self, tex: &Textures, id: TextureId, v: [&Vertex; 3], clip: egui::Rect) {
        let (p0, p1, p2) = (v[0].pos, v[1].pos, v[2].pos);
        let area = (p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y);
        if area.abs() < 1e-6 {
            return;
        }
        let minx = p0.x.min(p1.x).min(p2.x).max(clip.min.x).max(0.0).floor() as i64;
        let maxx =
            p0.x.max(p1.x)
                .max(p2.x)
                .min(clip.max.x)
                .min(self.w as f32)
                .ceil() as i64;
        let miny = p0.y.min(p1.y).min(p2.y).max(clip.min.y).max(0.0).floor() as i64;
        let maxy =
            p0.y.max(p1.y)
                .max(p2.y)
                .min(clip.max.y)
                .min(self.h as f32)
                .ceil() as i64;
        let col = v.map(|x| x.color.to_array().map(|b| b as f32 / 255.0));
        for y in miny..maxy {
            for x in minx..maxx {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                if px < clip.min.x || px > clip.max.x || py < clip.min.y || py > clip.max.y {
                    continue;
                }
                let w0 = ((p1.x - px) * (p2.y - py) - (p2.x - px) * (p1.y - py)) / area;
                let w1 = ((p2.x - px) * (p0.y - py) - (p0.x - px) * (p2.y - py)) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                    continue;
                }
                let u = w0 * v[0].uv.x + w1 * v[1].uv.x + w2 * v[2].uv.x;
                let vv = w0 * v[0].uv.y + w1 * v[1].uv.y + w2 * v[2].uv.y;
                let t = tex.sample(id, u, vv);
                let mut src = [0.0; 4];
                for i in 0..4 {
                    src[i] = (w0 * col[0][i] + w1 * col[1][i] + w2 * col[2][i]) * t[i];
                }
                let d = &mut self.px[y as usize * self.w + x as usize];
                for i in 0..4 {
                    d[i] = src[i] + d[i] * (1.0 - src[3]);
                }
            }
        }
    }

    fn paint(&mut self, tex: &Textures, prims: &[egui::ClippedPrimitive]) {
        for p in prims {
            if let Primitive::Mesh(m) = &p.primitive {
                for tri in m.indices.chunks_exact(3) {
                    let v = [
                        &m.vertices[tri[0] as usize],
                        &m.vertices[tri[1] as usize],
                        &m.vertices[tri[2] as usize],
                    ];
                    self.triangle(tex, m.texture_id, v, p.clip_rect);
                }
            }
        }
    }

    /// The box of the pixels drawn at all (a window and its shadow).
    fn drawn_box(&self) -> Option<(usize, usize, usize, usize)> {
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        for y in 0..self.h {
            for x in 0..self.w {
                if self.px[y * self.w + x][3] > 0.02 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x + 1);
                    y1 = y1.max(y + 1);
                }
            }
        }
        (x1 > x0 && y1 > y0).then_some((x0, y0, x1, y1))
    }

    /// The panel's height without the empty rows at its bottom.
    fn used_height(&self, from: usize) -> usize {
        let row = |y: usize| &self.px[y * self.w..(y + 1) * self.w];
        let last = row(self.h - 1).to_vec();
        let mut y = self.h - 1;
        while y > from
            && row(y)
                .iter()
                .zip(&last)
                .all(|(a, b)| (a[0] - b[0]).abs() < 0.01)
        {
            y -= 1;
        }
        (y + 14).min(self.h)
    }

    fn save(&self, path: &std::path::Path, (x0, y0, x1, y1): (usize, usize, usize, usize)) {
        let mut img = image::RgbaImage::new((x1 - x0) as u32, (y1 - y0) as u32);
        for y in y0..y1 {
            for x in x0..x1 {
                let p = self.px[y * self.w + x];
                let a = p[3];
                let un = |c: f32| {
                    if a > 0.0 {
                        (c / a * 255.0).round().clamp(0.0, 255.0) as u8
                    } else {
                        0
                    }
                };
                img.put_pixel(
                    (x - x0) as u32,
                    (y - y0) as u32,
                    image::Rgba([un(p[0]), un(p[1]), un(p[2]), (a * 255.0).round() as u8]),
                );
            }
        }
        let file = std::fs::File::create(path).unwrap();
        let enc = image::codecs::png::PngEncoder::new_with_quality(
            std::io::BufWriter::new(file),
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::Adaptive,
        );
        image::ImageEncoder::write_image(
            enc,
            img.as_raw(),
            img.width(),
            img.height(),
            image::ExtendedColorType::Rgba8,
        )
        .unwrap();
    }
}

/// A canvas overlay rasterised for a test: what `body` paints on a
/// `w` x `h` point area at `ppp` device pixels per point, read back as
/// 8-bit RGB over black.
pub(crate) struct Raster {
    canvas: Canvas,
}

impl Raster {
    /// The device pixel at column `x`, row `y`.
    pub(crate) fn rgb(&self, x: usize, y: usize) -> [u8; 3] {
        let p = self.canvas.px[y * self.canvas.w + x];
        [p[0], p[1], p[2]].map(|c| (c * 255.0).round().clamp(0.0, 255.0) as u8)
    }

    pub(crate) fn save(&self, path: &std::path::Path) {
        self.canvas.save(path, (0, 0, self.canvas.w, self.canvas.h));
    }
}

pub(crate) fn rasterise(w: usize, h: usize, ppp: f32, body: impl FnOnce(&egui::Painter)) -> Raster {
    let ctx = egui::Context::default();
    ctx.set_pixels_per_point(ppp);
    let mut input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(w as f32, h as f32),
        )),
        ..Default::default()
    };
    input
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .native_pixels_per_point = Some(ppp);
    let mut body = Some(body);
    let mut out = ctx.run_ui(input, |ui| {
        if let Some(b) = body.take() {
            b(ui.painter());
        }
    });
    let mut tex = Textures::default();
    tex.apply(&out.textures_delta);
    out.textures_delta.clear();
    let shapes = std::mem::take(&mut out.shapes);
    let prims = ctx.tessellate(shapes, out.pixels_per_point);
    // Device pixels: the meshes are in points.
    let scale = out.pixels_per_point;
    let prims: Vec<egui::ClippedPrimitive> = prims
        .into_iter()
        .map(|mut p| {
            if let Primitive::Mesh(m) = &mut p.primitive {
                for v in &mut m.vertices {
                    v.pos = egui::pos2(v.pos.x * scale, v.pos.y * scale);
                }
            }
            p.clip_rect = egui::Rect::from_min_max(
                (p.clip_rect.min.to_vec2() * scale).to_pos2(),
                (p.clip_rect.max.to_vec2() * scale).to_pos2(),
            );
            p
        })
        .collect();
    let mut canvas = Canvas::new(
        (w as f32 * scale).round() as usize,
        (h as f32 * scale).round() as usize,
    );
    canvas.paint(&tex, &prims);
    Raster { canvas }
}

/// What a screenshot shows.
enum Shot {
    /// A menu command that opens a dialog.
    Command(&'static str),
    /// A dialog set directly (its state built by the app).
    Dialog(fn(&mut App) -> Dialog),
    Options(OptionsPage),
    Docker(DockerTab),
}

const W: usize = 1400;
const H: usize = 1000;

struct Shooter {
    ctx: egui::Context,
    tex: Textures,
    time: f64,
}

impl Shooter {
    fn new() -> Self {
        let ctx = egui::Context::default();
        crate::theme::install_fonts_bundled(&ctx);
        App::setup_context(&ctx);
        Shooter {
            ctx,
            tex: Textures::default(),
            time: 0.0,
        }
    }

    fn frame(&mut self, app: &mut App, mut body: impl FnMut(&mut App, &egui::Context)) -> Canvas {
        self.time += 0.5;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(W as f32, H as f32),
            )),
            time: Some(self.time),
            ..Default::default()
        };
        let mut out = self.ctx.run_ui(input, |ui| {
            let ctx = ui.ctx().clone();
            crate::ui::apply_theme(&ctx, &crate::settings::effective_theme(&app.settings));
            crate::ui::apply_shadows(&ctx, app.settings.ui_shadows);
            body(app, &ctx);
        });
        self.tex.apply(&out.textures_delta);
        let shapes = std::mem::take(&mut out.shapes);
        let prims = self.ctx.tessellate(shapes, out.pixels_per_point);
        let mut c = Canvas::new(W, H);
        c.paint(&self.tex, &prims);
        for id in &out.textures_delta.free {
            self.tex.0.remove(id);
        }
        out.textures_delta.clear();
        c
    }
}

/// A drawing to show: a filled rectangle, an ellipse, a line of text
/// and a small generated bitmap, the rectangle selected.
fn demo_app() -> App {
    use tracedraw_core::geometry::{Point, Rect};
    use tracedraw_core::{Color, Fill, ShapeKind};
    let mut app = App::headless();
    crate::i18n::set_language("en");
    let rect = app.new_shape(ShapeKind::Rect {
        rect: Rect::new(30.0, 40.0, 110.0, 100.0),
        radius: 0.0,
        corners: None,
    });
    let _ = app.new_shape(ShapeKind::Ellipse {
        rect: Rect::new(120.0, 50.0, 180.0, 110.0),
        arc: None,
    });
    let _ = app.new_shape(ShapeKind::Text {
        spans: vec![tracedraw_core::TextSpan::new(
            "Sample text",
            "Selawik",
            24.0,
        )],
        origin: Point::new(30.0, 150.0),
        frame: None,
        align: Default::default(),
        para: Default::default(),
        on_path: None,
    });
    if let Some(id) = rect {
        app.select(vec![id]);
        app.apply_fill(Fill::Solid(Color::rgb8(70, 130, 220)));
    }
    app
}

fn demo_bitmap(app: &mut App) {
    let img = image::RgbaImage::from_fn(96, 64, |x, y| {
        image::Rgba([(x * 2 + 40) as u8, (y * 3 + 60) as u8, 180, 255])
    });
    if let Some(id) = app.place_image(img, Some("Demo".into())) {
        app.select(vec![id]);
    }
}

/// A product shot for Remove Background: an orange ball with a soft
/// shadowed rim and a highlight, on a light backdrop that darkens toward
/// the bottom.
fn demo_photo(app: &mut App) {
    let img = image::RgbaImage::from_fn(240, 180, |x, y| {
        let (dx, dy) = (x as f32 - 120.0, y as f32 - 86.0);
        let r = (dx * dx + dy * dy).sqrt();
        let back = 238.0 - y as f32 * 0.25;
        let ball = |shade: f32| [255.0 * shade, 140.0 * shade, 30.0 * shade];
        let shade = (1.0 - r / 120.0).clamp(0.55, 1.0);
        let hi = (1.0 - ((dx + 22.0).powi(2) + (dy + 24.0).powi(2)).sqrt() / 26.0).max(0.0);
        let c = ball(shade);
        let c = [
            c[0] + (255.0 - c[0]) * hi,
            c[1] + (255.0 - c[1]) * hi,
            c[2] + (255.0 - c[2]) * hi,
        ];
        let a = (60.0 - r).clamp(0.0, 1.0);
        let px = |i: usize| (c[i] * a + back * (1.0 - a)).round() as u8;
        image::Rgba([px(0), px(1), px(2), 255])
    });
    if let Some(id) = app.place_image(img, Some("Photo".into())) {
        app.select(vec![id]);
    }
}

/// The screenshots: file name (the topic, `/` and `#` as `-`), what to
/// show and whether a bitmap is selected first.
fn shots() -> Vec<(String, Shot, bool)> {
    use OptionsPage as P;
    let mut v: Vec<(String, Shot, bool)> = Vec::new();
    let name = |t: &str| t.replace(['/', '#'], "-");
    let cmd = [
        ("menu.file.new", false),
        ("menu.file.new_from_template", false),
        ("menu.file.save_as", false),
        ("menu.file.document_properties", false),
        ("menu.file.export", false),
        ("menu.file.print", false),
        ("menu.file.print_merge_create", false),
        ("menu.file.preflight", false),
        ("menu.file.publish_to_pdf", false),
        ("menu.file.github_open", false),
        ("menu.file.github_save", false),
        ("menu.edit.paste_special", false),
        ("menu.layout.insert_page", false),
        ("menu.layout.rename_page", false),
        ("menu.layout.go_to_page", false),
        ("menu.layout.page_size", false),
        ("menu.layout.page_number_settings", false),
        ("menu.table.create_new_table", false),
        ("menu.object.create_pattern", false),
        ("menu.object.insert_barcode", false),
        ("menu.bitmaps.convert_to_bitmap", false),
        ("menu.bitmaps.straighten_image", true),
        ("menu.bitmaps.resample", true),
        ("menu.bitmaps.inflate_manual", true),
        ("menu.bitmaps.mode_bw", true),
        ("menu.bitmaps.mode_duotone", true),
        ("menu.bitmaps.mode_paletted", true),
        ("menu.bitmaps.trace_logo", true),
        ("menu.bitmaps.remove_background_local", true),
        ("menu.text.tabs", false),
        ("menu.text.columns", false),
        ("menu.text.bullets", false),
        ("menu.text.drop_cap", false),
        ("menu.text.text_statistics", false),
        ("menu.text.spell_check", false),
        ("menu.text.custom_hyphens", false),
        ("menu.tools.border_and_grommet", false),
        ("menu.tools.color_management", false),
        ("menu.tools.font_manager", false),
        ("menu.tools.tracecut", false),
        ("menu.tools.time_tracker", false),
        ("menu.window.palette_editor", false),
        ("menu.help.about", false),
    ];
    for (k, bmp) in cmd {
        v.push((k.to_string(), Shot::Command(k), bmp));
    }
    v.push((
        name("objects/objects-and-layers#rename"),
        Shot::Dialog(|app| match app.selection.first().copied() {
            Some(shape) => Dialog::RenameObject {
                shape,
                name: "Rectangle".into(),
            },
            None => Dialog::None,
        }),
        false,
    ));
    v.push((
        name("color/fills#edit-fill"),
        Shot::Dialog(|app| {
            app.apply_fill(tracedraw_core::Fill::linear(
                tracedraw_core::Color::rgb8(220, 30, 90),
                tracedraw_core::Color::rgb8(40, 90, 200),
                0.0,
            ));
            app.open_edit_fill();
            std::mem::replace(&mut app.dialog, Dialog::None)
        }),
        false,
    ));
    v.push((
        name("lines-shapes/outlines#outline-pen"),
        Shot::Dialog(|app| {
            app.open_outline_pen();
            std::mem::replace(&mut app.dialog, Dialog::None)
        }),
        false,
    ));
    v.push((
        name("lines-shapes/shape-tool#align-nodes"),
        Shot::Dialog(|_| Dialog::NodeAlign {
            horizontal: true,
            vertical: true,
        }),
        false,
    ));
    v.push((
        name("get-started/github#history"),
        Shot::Dialog(|_| Dialog::GithubHistory),
        false,
    ));
    v.push((
        name("text/writing-tools#change-case"),
        Shot::Dialog(|_| Dialog::ChangeCase(crate::ops2::CaseMode::Sentence)),
        false,
    ));
    v.push((
        name("text/writing-tools#encode"),
        Shot::Dialog(|_| Dialog::Encode(Default::default())),
        false,
    ));
    v.push((
        name("objects/shelf#remove"),
        Shot::Dialog(|_| Dialog::ConfirmShelfRemove {
            uid: 1,
            name: "Logo".into(),
            dont_ask: false,
        }),
        false,
    ));
    v.push((
        name("file-formats/publish-pdf#password"),
        Shot::Dialog(|_| {
            Dialog::PdfPassword(crate::ui::pdf_password::PdfPasswordState {
                path: "brochure.pdf".into(),
                import: false,
                password: String::new(),
                wrong: false,
                focused_once: true,
            })
        }),
        false,
    ));
    v.push((
        name("templates-styles/object-styles#document-defaults"),
        Shot::Dialog(|_| {
            Dialog::ChangeDefaults(crate::ui::defaults_dialog::State {
                change: crate::defaults::DefaultChange::Fill(tracedraw_core::Fill::Solid(
                    tracedraw_core::Color::rgb8(70, 130, 220),
                )),
                kinds: [true, false, false],
                dont_ask: false,
            })
        }),
        false,
    ));
    v.push((
        name("effects/bitmap-effects"),
        Shot::Dialog(|app| {
            crate::ui::effect_dialog::choose_effect(app, "gaussian_blur");
            std::mem::replace(&mut app.dialog, Dialog::None)
        }),
        true,
    ));
    v.push((
        name("customize/reset-settings"),
        Shot::Dialog(|_| Dialog::ConfirmResetSettings),
        false,
    ));
    v.push((
        name("get-started/open-and-save#closing"),
        Shot::Dialog(|_| Dialog::ConfirmClose),
        false,
    ));
    v.push((
        name("get-started/browser-version"),
        Shot::Dialog(|_| Dialog::WebNotice),
        false,
    ));
    // Text > Proofing > Autocorrect: the Autocorrect tab of
    // Options > Text.
    v.push((
        name("text/writing-tools#autocorrect"),
        Shot::Dialog(|app| {
            app.options_page = crate::ui::dialogs::OptionsPage::Text;
            app.options_text_tab = crate::ui::options::AUTOCORRECT_TAB;
            Dialog::Options
        }),
        false,
    ));
    v.push((
        name("welcome/about-and-updates#updates"),
        Shot::Dialog(|_| Dialog::Updates),
        false,
    ));
    for s in [
        crate::ui::options::OptionsSet::App,
        crate::ui::options::OptionsSet::Customization,
        crate::ui::options::OptionsSet::Tools,
        crate::ui::options::OptionsSet::Global,
        crate::ui::options::OptionsSet::Workspaces,
        crate::ui::options::OptionsSet::Ai,
        crate::ui::options::OptionsSet::Document,
    ] {
        for p in s.pages() {
            v.push((
                name(crate::help::options_topic(*p)),
                Shot::Options(*p),
                false,
            ));
        }
    }
    for t in DockerTab::ALL {
        let _ = P::General;
        v.push((name(crate::help::docker_topic(t)), Shot::Docker(t), false));
    }
    v
}

/// Draw every screenshot into `$HELP_SHOTS` (run by `web/help_shots.sh`).
#[test]
#[ignore]
fn help_shots() {
    let Some(dir) = std::env::var_os("HELP_SHOTS") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut made = Vec::new();
    let mut missed = Vec::new();
    let only: Vec<String> = std::env::var("HELP_SHOTS_ONLY")
        .map(|v| v.split(',').map(str::to_string).collect())
        .unwrap_or_default();
    for (file, shot, bitmap) in shots() {
        if !only.is_empty() && !only.iter().any(|o| file.starts_with(o.as_str())) {
            continue;
        }
        eprintln!("shot {file}");
        let mut sh = Shooter::new();
        let mut app = demo_app();
        if matches!(shot, Shot::Command("menu.bitmaps.remove_background_local")) {
            demo_photo(&mut app);
        } else if bitmap {
            demo_bitmap(&mut app);
        }
        let mut file = file;
        match &shot {
            Shot::Command(k) => {
                let mut ran = false;
                sh.frame(&mut app, |app, ctx| {
                    ran = crate::ui::menus::run_command(app, ctx, k)
                });
                let topic = crate::help::dialog_topic(&app.dialog, app.options_page);
                if !ran || matches!(app.dialog, Dialog::None) {
                    missed.push(format!("{k}: no dialog"));
                    continue;
                }
                file = topic.replace(['/', '#'], "-");
            }
            Shot::Dialog(f) => app.dialog = f(&mut app),
            Shot::Options(p) => {
                app.options_page = *p;
                app.dialog = Dialog::Options;
            }
            Shot::Docker(t) => {
                app.show_dockers = true;
                if !app.open_dockers.contains(t) {
                    app.open_dockers.push(*t);
                }
                app.docker_tab = *t;
            }
        }
        let docker = matches!(shot, Shot::Docker(_));
        let mut canvas = None;
        for _ in 0..8 {
            canvas = Some(sh.frame(&mut app, |app, ctx| {
                if docker {
                    let rect =
                        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(340.0, 940.0));
                    let id = egui::Id::new("help_shot_docker");
                    let mut ui = egui::Ui::new(
                        ctx.clone(),
                        id,
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layer_id(egui::LayerId::new(egui::Order::Middle, id)),
                    );
                    ui.painter()
                        .rect_filled(rect, 0.0, crate::theme::Tokens::field());
                    crate::ui::dockers::dockers(app, &mut ui, crate::app::DockerPlace::Right);
                } else {
                    crate::ui::dialogs::show(app, ctx);
                }
            }));
            if !docker && matches!(app.dialog, Dialog::None) {
                break;
            }
        }
        let Some(c) = canvas else { continue };
        let area = if docker {
            Some((0, 0, 340, c.used_height(120)))
        } else {
            c.drawn_box()
        };
        let Some(area) = area else {
            missed.push(format!("{file}: nothing drawn"));
            continue;
        };
        let path = dir.join(format!("{file}.png"));
        c.save(&path, area);
        made.push(file);
    }
    let list = made.join("\n") + "\n";
    std::fs::write(dir.join("shots.txt"), list).unwrap();
    eprintln!("{} screenshots; not made: {missed:#?}", made.len());
}

/// Every tool icon on a toolbox-like panel, light and dark, at 1x and
/// 2x pixels per point, into `$ICON_SHEET` (a check of the icon style
/// by eye; see docs/behavior/icons.md).
#[test]
#[ignore]
fn icon_sheet() {
    let Some(dir) = std::env::var_os("ICON_SHEET") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    use crate::tools::Tool;
    let cols = 8;
    let rows = Tool::ALL.len().div_ceil(cols);
    let cell = 32.0;
    let icon = 22.0;
    for theme in [crate::theme::Theme::Light, crate::theme::Theme::Dark] {
        crate::theme::set_theme(theme);
        for ppp in [1.0f32, 2.0] {
            let ctx = egui::Context::default();
            ctx.set_pixels_per_point(ppp);
            let w = (cols as f32 * cell) as usize;
            let h = (rows as f32 * cell) as usize;
            let mut tex = Textures::default();
            let mut canvas = None;
            for _ in 0..2 {
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(w as f32, h as f32),
                    )),
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ui| {
                    let painter = ui.painter();
                    let all =
                        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w as f32, h as f32));
                    painter.rect_filled(all, 0.0, crate::theme::Tokens::panel());
                    for (i, t) in Tool::ALL.iter().enumerate() {
                        let (c, r) = ((i % cols) as f32, (i / cols) as f32);
                        let center = egui::pos2((c + 0.5) * cell, (r + 0.5) * cell);
                        let rect = egui::Rect::from_center_size(center, egui::Vec2::splat(icon));
                        crate::ui::icons::draw(painter, rect, *t, crate::theme::Tokens::icon());
                    }
                });
                tex.apply(&out.textures_delta);
                let shapes = std::mem::take(&mut out.shapes);
                let mut prims = ctx.tessellate(shapes, out.pixels_per_point);
                for p in &mut prims {
                    p.clip_rect = egui::Rect::from_min_max(
                        (p.clip_rect.min.to_vec2() * ppp).to_pos2(),
                        (p.clip_rect.max.to_vec2() * ppp).to_pos2(),
                    );
                    if let Primitive::Mesh(m) = &mut p.primitive {
                        for v in &mut m.vertices {
                            v.pos = (v.pos.to_vec2() * ppp).to_pos2();
                        }
                    }
                }
                let (pw, ph) = ((w as f32 * ppp) as usize, (h as f32 * ppp) as usize);
                let mut c = Canvas::new(pw, ph);
                c.paint(&tex, &prims);
                out.textures_delta.clear();
                canvas = Some((c, pw, ph));
            }
            let (c, pw, ph) = canvas.unwrap();
            let name = format!("icons_{}_{}x.png", theme.id(), ppp as u32);
            c.save(&dir.join(name), (0, 0, pw, ph));
        }
    }
    crate::theme::set_theme(crate::theme::Theme::Light);
}

#[test]
#[ignore]
fn scratch_196_window() {
    let dir = std::path::PathBuf::from(
        "/tmp/claude-0/-home-claude/ecd2ad2c-4fd8-5343-ab00-e2a436bfbe5e/scratchpad/v196",
    );
    std::fs::create_dir_all(&dir).unwrap();
    for lang in ["en", "de", "ru", "pt-BR"] {
        let mut sh = Shooter::new();
        let mut app = demo_app();
        crate::i18n::set_language(lang);
        app.show_welcome = false;
        app.show_dockers = true;
        app.docker_tab = DockerTab::Properties;
        app.settings.properties_closed.clear();
        let sel = app.selection.clone();
        for (tag, s) in [("none", Vec::new()), ("rect", sel)] {
            app.select(s);
            let mut c = None;
            for _ in 0..4 {
                sh.time += 0.5;
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(W as f32, H as f32),
                    )),
                    time: Some(sh.time),
                    ..Default::default()
                };
                let mut out = sh.ctx.run_ui(input, |ui| crate::ui::root(&mut app, ui));
                sh.tex.apply(&out.textures_delta);
                let shapes = std::mem::take(&mut out.shapes);
                let prims = sh.ctx.tessellate(shapes, out.pixels_per_point);
                let mut cv = Canvas::new(W, H);
                cv.paint(&sh.tex, &prims);
                out.textures_delta.clear();
                c = Some(cv);
            }
            c.unwrap()
                .save(&dir.join(format!("{lang}_{tag}.png")), (0, 0, W, H));
        }
    }
    crate::i18n::set_language("en");
}
