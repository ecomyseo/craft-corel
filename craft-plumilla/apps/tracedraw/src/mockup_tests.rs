//! The T-shirt mockup of the Help (docs/help/*/effects/mockup.md): a
//! design put on a photo of a shirt with an envelope that follows the
//! chest, uniform transparency with the Multiply merge mode so the cloth's
//! folds and shading show through, and a clip frame of the shirt's shape
//! that keeps it on the cloth. `mockup_steps` checks it works;
//! `mockup_shots` (ignored) draws the Help's pictures of each step into
//! `$HELP_SHOTS`.

use crate::app::App;
use crate::effects_ui::TransparencySettings;
use tracedraw_core::envelope::Mode;
use tracedraw_core::geometry::{BezPath, Point, Rect, Shape as _, Vec2};
use tracedraw_core::live::{Effect, MergeMode};
use tracedraw_core::{Color, Fill, ShapeId, ShapeKind, TextSpan};

/// The photo's size in pixels and resolution.
const PHOTO: (u32, u32) = (900, 1000);
const DPI: f64 = 150.0;

/// The shirt's outline in photo pixels (y down): shoulders, sleeves,
/// body and the neckline.
fn shirt_outline() -> Vec<(f64, f64)> {
    vec![
        (330.0, 110.0),
        (390.0, 150.0),
        (450.0, 165.0),
        (510.0, 150.0),
        (570.0, 110.0),
        (720.0, 170.0),
        (850.0, 390.0),
        (730.0, 450.0),
        (680.0, 360.0),
        (690.0, 900.0),
        (210.0, 900.0),
        (220.0, 360.0),
        (170.0, 450.0),
        (50.0, 390.0),
        (180.0, 170.0),
    ]
}

fn inside(poly: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut c = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            c = !c;
        }
        j = i;
    }
    c
}

/// A photo of a white shirt on a grey backdrop: soft light from the top
/// left, folds running down the body and shade under the arms.
fn shirt_photo() -> image::RgbaImage {
    let poly = shirt_outline();
    image::RgbaImage::from_fn(PHOTO.0, PHOTO.1, |x, y| {
        let (fx, fy) = (x as f64, y as f64);
        let back = 205.0 - fy * 0.04 + (fx * 0.01).sin() * 2.0;
        if !inside(&poly, fx + 0.5, fy + 0.5) {
            let v = back.clamp(0.0, 255.0) as u8;
            return image::Rgba([v, v, v.saturating_add(4), 255]);
        }
        let light = 1.0 - (fx - 250.0).hypot(fy - 150.0) / 2600.0;
        let folds = 0.06 * ((fx * 0.045 + fy * 0.012).sin() * (fy * 0.01).cos())
            + 0.05 * ((fx - 450.0) / 60.0).sin().powi(8)
            + 0.04 * ((fy - 600.0) / 90.0 + fx / 210.0).sin().powi(6);
        let arm = (-((fx - 230.0).powi(2) + (fy - 380.0).powi(2)) / 5000.0).exp()
            + (-((fx - 670.0).powi(2) + (fy - 380.0).powi(2)) / 5000.0).exp();
        let shade = (light - folds - 0.12 * arm).clamp(0.55, 1.0);
        let px = |base: f64| (base * shade).clamp(0.0, 255.0) as u8;
        image::Rgba([px(246.0), px(244.0), px(240.0), 255])
    })
}

/// The ids the mockup is made of.
pub struct Mockup {
    pub photo: ShapeId,
    pub design: ShapeId,
    /// The photo's place on the page.
    pub area: Rect,
}

/// Page millimetres of a photo pixel.
fn to_page(area: Rect, (x, y): (f64, f64)) -> Point {
    Point::new(area.x0 + x * 25.4 / DPI, area.y1 - y * 25.4 / DPI)
}

/// Step 1: the photo and the design (a badge and two lines of text)
/// grouped over the chest.
pub fn place_design(app: &mut App) -> Mockup {
    tracedraw_text::install();
    let photo = app
        .place_image_at(shirt_photo(), Some("T-shirt".into()), Some((DPI, DPI)))
        .expect("photo");
    let area = app.doc().find_shape(photo).expect("photo").bounds();
    let c = to_page(area, (450.0, 430.0));
    let badge = app
        .new_shape(ShapeKind::Ellipse {
            rect: Rect::new(c.x - 24.0, c.y - 24.0, c.x + 24.0, c.y + 24.0),
            arc: None,
        })
        .expect("badge");
    app.select(vec![badge]);
    app.apply_fill(Fill::linear(
        Color::rgb8(232, 72, 40),
        Color::rgb8(250, 170, 40),
        90.0,
    ));
    let inner = app
        .new_shape(ShapeKind::Ellipse {
            rect: Rect::new(c.x - 17.0, c.y - 17.0, c.x + 17.0, c.y + 17.0),
            arc: None,
        })
        .expect("ring");
    app.select(vec![inner]);
    app.apply_fill(Fill::Solid(Color::rgb8(30, 60, 120)));
    let text = |app: &mut App, t: &str, size: f64, y: f64| {
        let id = app
            .new_shape(ShapeKind::Text {
                spans: vec![TextSpan::new(t, "Selawik", size)],
                origin: Point::new(c.x - 30.0, y),
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            })
            .expect("text");
        app.select(vec![id]);
        app.apply_fill(Fill::Solid(Color::rgb8(30, 60, 120)));
        id
    };
    let t1 = text(app, "TRACE", 30.0, c.y - 40.0);
    let t2 = text(app, "DRAW", 30.0, c.y - 52.0);
    for t in [t1, t2] {
        if let Some(b) = app.doc().find_shape(t).map(|s| s.bounds()) {
            let dx = c.x - b.center().x;
            app.select(vec![t]);
            app.nudge(Vec2::new(dx, 0.0));
        }
    }
    app.select(vec![badge, inner, t1, t2]);
    app.group_selection();
    let design = app.selection[0];
    Mockup {
        photo,
        design,
        area,
    }
}

/// Step 2: an envelope that rounds the design over the chest.
pub fn wrap_design(app: &mut App, m: &Mockup) {
    app.select(vec![m.design]);
    app.apply_envelope_frame(|b| {
        let mut f = tracedraw_core::envelope::Frame::around(b);
        let (w, h) = (b.width(), b.height());
        for (i, d) in [
            (1, Vec2::new(0.0, -h * 0.07)),
            (5, Vec2::new(0.0, h * 0.03)),
            (3, Vec2::new(w * 0.03, 0.0)),
            (7, Vec2::new(-w * 0.03, 0.0)),
        ] {
            f.move_node(i, d, Mode::SingleArc);
        }
        for s in 0..4 {
            f.reshape_side(s, Mode::SingleArc);
        }
        f
    });
}

/// Step 3: uniform transparency with the Multiply merge mode, so the
/// cloth's shading darkens the print.
pub fn multiply_design(app: &mut App, m: &Mockup) {
    app.select(vec![m.design]);
    app.patch_transparency(
        &TransparencySettings {
            kind: 0,
            amount: 10.0,
            mask: Fill::None,
            merge: MergeMode::Multiply,
            target: 2,
        },
        &[
            crate::mixed::TransparencyField::Kind,
            crate::mixed::TransparencyField::Amount,
            crate::mixed::TransparencyField::Merge,
        ],
    );
}

/// Step 4: the design inside a frame of the shirt's shape, so nothing
/// prints past the cloth.
pub fn clip_to_shirt(app: &mut App, m: &Mockup) -> ShapeId {
    let mut path = BezPath::new();
    for (i, p) in shirt_outline().into_iter().enumerate() {
        let q = to_page(m.area, p);
        if i == 0 {
            path.move_to(q);
        } else {
            path.line_to(q);
        }
    }
    path.close_path();
    let frame = app
        .new_shape(ShapeKind::Path { path, closed: true })
        .expect("frame");
    app.select(vec![frame]);
    app.apply_fill(Fill::None);
    app.run(tracedraw_core::Command::SetStroke {
        shapes: vec![frame],
        stroke: None,
    });
    app.place_inside(vec![m.design], frame);
    frame
}

/// The page drawn at `zoom` pixels per mm, cut to the photo.
fn render(app: &App, area: Rect, zoom: f64) -> image::RgbaImage {
    let doc = app.doc();
    let page = doc.pages[0].id;
    let (w, h) = (
        (area.width() * zoom).round() as u32,
        (area.height() * zoom).round() as u32,
    );
    let opts = tracedraw_render::RenderOptions {
        width: w,
        height: h,
        view: tracedraw_render::ViewTransform {
            zoom,
            origin_x: -area.x0 * zoom,
            origin_y: area.y1 * zoom,
        },
        ..Default::default()
    };
    let px = tracedraw_render::render_page(doc, page, &opts).expect("render");
    let mut img = image::RgbaImage::new(w, h);
    for (i, p) in px.pixels().iter().enumerate() {
        let c = p.demultiply();
        img.as_mut()[i * 4..i * 4 + 4].copy_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    img
}

/// The darkest of the design's blue where the cloth is shaded and where
/// it is lit: Multiply lets the shading through.
fn blue_at(img: &image::RgbaImage, at: (u32, u32)) -> u8 {
    img.get_pixel(at.0, at.1)[2]
}

/// Each step does what the Help says: the envelope changes the design's
/// outline, Multiply lets the cloth's shading darken it, and the clip
/// frame holds it.
#[test]
fn mockup_steps() {
    let mut app = App::headless();
    let m = place_design(&mut app);
    let flat = app.doc().find_shape(m.design).expect("design").clone();
    wrap_design(&mut app, &m);
    let wrapped = app.doc().find_shape(m.design).expect("design").clone();
    assert!(wrapped
        .effects
        .iter()
        .any(|e| matches!(e, Effect::Envelope { .. })));
    assert_ne!(
        tracedraw_core::live::evaluate(&flat)
            .main
            .page_path()
            .bounding_box(),
        tracedraw_core::live::evaluate(&wrapped)
            .main
            .page_path()
            .bounding_box()
    );
    multiply_design(&mut app, &m);
    let s = app.doc().find_shape(m.design).expect("design").clone();
    let t = crate::mixed::ShapeTransparency::of(&s);
    assert_eq!(
        (t.kind, t.merge),
        (1, MergeMode::Multiply),
        "uniform, Multiply"
    );
    assert!((t.amount - 10.0).abs() < 0.5, "{}", t.amount);
    assert_eq!(s.opacity, 1.0);
    let frame = clip_to_shirt(&mut app, &m);
    let clip = app.doc().find_shape(frame).expect("frame").clone();
    let ShapeKind::ClipFrame { contents, .. } = &clip.kind else {
        panic!("a clip frame: {:?}", clip.kind);
    };
    assert_eq!(contents.len(), 1);
    // Drawn: the print over the badge's ring takes the cloth's shading.
    let img = render(&app, m.area, 4.0);
    let at = |x: f64, y: f64| {
        let p = to_page(m.area, (x, y));
        (
            ((p.x - m.area.x0) * 4.0) as u32,
            ((m.area.y1 - p.y) * 4.0) as u32,
        )
    };
    let lit = blue_at(&img, at(450.0, 430.0));
    assert!(lit < 200, "the print is there: {lit}");
    let photo = render_photo_only(&m);
    let cloth = |p: (u32, u32)| photo.get_pixel(p.0, p.1)[2] as f64;
    // Multiply: print = design colour x cloth; where the cloth is darker
    // the print is darker.
    let (a, b) = (at(445.0, 430.0), at(455.0, 432.0));
    let (pa, pb) = (blue_at(&img, a) as f64, blue_at(&img, b) as f64);
    if (cloth(a) - cloth(b)).abs() > 6.0 {
        assert_eq!(cloth(a) < cloth(b), pa <= pb, "{pa} {pb}");
    }
}

/// The photo alone, drawn as the mockup is (for comparing shading).
fn render_photo_only(m: &Mockup) -> image::RgbaImage {
    let mut app = App::headless();
    let photo = app
        .place_image_at(shirt_photo(), Some("T-shirt".into()), Some((DPI, DPI)))
        .expect("photo");
    let area = app.doc().find_shape(photo).expect("photo").bounds();
    assert_eq!(area, m.area);
    render(&app, area, 4.0)
}

/// The Help's pictures, one per step, into `$HELP_SHOTS`.
#[test]
#[ignore]
fn mockup_shots() {
    let Some(dir) = std::env::var_os("HELP_SHOTS") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut app = App::headless();
    let m = place_design(&mut app);
    let save = |app: &App, name: &str| {
        let img = render(app, m.area, 3.0);
        img.save(dir.join(format!("effects-mockup-{name}.png")))
            .unwrap();
    };
    save(&app, "1-design");
    wrap_design(&mut app, &m);
    save(&app, "2-envelope");
    multiply_design(&mut app, &m);
    save(&app, "3-multiply");
    // A print larger than the chest, to show what the frame cuts.
    app.select(vec![m.design]);
    let c = app
        .doc()
        .find_shape(m.design)
        .expect("design")
        .bounds()
        .center();
    app.transform_selection(
        tracedraw_core::geometry::Affine::translate(c.to_vec2())
            * tracedraw_core::geometry::Affine::scale(1.7)
            * tracedraw_core::geometry::Affine::translate(-c.to_vec2()),
    );
    save(&app, "4-large");
    clip_to_shirt(&mut app, &m);
    save(&app, "5-final");
}
