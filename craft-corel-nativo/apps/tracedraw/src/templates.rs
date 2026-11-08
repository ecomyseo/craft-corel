//! Templates for File > New From Template: blank pages of common sizes,
//! a set of designed starting points drawn here from shapes and text, and
//! the user's own `.tdt` files in the settings folder's `templates`
//! folder. See docs/behavior/new-from-template.md.

use std::path::PathBuf;

use tracedraw_core::document::{ParagraphStyle, Shape, ShapeKind, TextAlign, TextSpan};
use tracedraw_core::style::{Fountain, FountainKind, Stroke};
use tracedraw_core::{Affine, Color, Document, Fill, Point, Rect, Size};

/// A template's category, for the category filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    Blank,
    BusinessCards,
    Flyers,
    Posters,
    Stationery,
    Forms,
    Banners,
    SocialMedia,
    Labels,
    Mine,
}

impl Category {
    pub const ALL: [Category; 10] = [
        Category::Blank,
        Category::BusinessCards,
        Category::Flyers,
        Category::Posters,
        Category::Stationery,
        Category::Forms,
        Category::Banners,
        Category::SocialMedia,
        Category::Labels,
        Category::Mine,
    ];

    /// The i18n key of the category's name.
    pub fn key(self) -> &'static str {
        match self {
            Category::Blank => "tpl.cat_blank",
            Category::BusinessCards => "tpl.cat_business_cards",
            Category::Flyers => "tpl.cat_flyers",
            Category::Posters => "tpl.cat_posters",
            Category::Stationery => "tpl.cat_stationery",
            Category::Forms => "tpl.cat_forms",
            Category::Banners => "tpl.cat_banners",
            Category::SocialMedia => "tpl.cat_social",
            Category::Labels => "tpl.cat_labels",
            Category::Mine => "tpl.cat_mine",
        }
    }
}

/// Where a template comes from.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// Built in: the index into `built_in()`.
    BuiltIn(usize),
    /// A `.tdt` file of the user's.
    File(PathBuf),
}

/// One template of the list.
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    /// The name shown (already translated for built-in ones).
    pub name: String,
    pub category: Category,
    pub source: Source,
}

type Build = fn(&mut Doc);

/// The built-in templates: i18n key of the name, category, page size and
/// the drawing.
fn table() -> Vec<(&'static str, Category, Size, Build)> {
    let px = |w: f64, h: f64| Size::new(w * 25.4 / 96.0, h * 25.4 / 96.0);
    vec![
        (
            "welcome.tpl_a4_portrait",
            Category::Blank,
            Size::new(210.0, 297.0),
            blank,
        ),
        (
            "welcome.tpl_a4_landscape",
            Category::Blank,
            Size::new(297.0, 210.0),
            blank,
        ),
        (
            "welcome.tpl_a3",
            Category::Blank,
            Size::new(297.0, 420.0),
            blank,
        ),
        (
            "welcome.tpl_letter",
            Category::Blank,
            Size::new(215.9, 279.4),
            blank,
        ),
        (
            "welcome.tpl_business_card",
            Category::Blank,
            Size::new(90.0, 50.0),
            blank,
        ),
        (
            "welcome.tpl_poster",
            Category::Blank,
            Size::new(420.0, 594.0),
            blank,
        ),
        (
            "welcome.tpl_web_1080p",
            Category::Blank,
            px(1920.0, 1080.0),
            blank,
        ),
        (
            "welcome.tpl_social_square",
            Category::Blank,
            px(1080.0, 1080.0),
            blank,
        ),
        (
            "tpl.card_band",
            Category::BusinessCards,
            Size::new(90.0, 50.0),
            card_band,
        ),
        (
            "tpl.card_dark",
            Category::BusinessCards,
            Size::new(90.0, 50.0),
            card_dark,
        ),
        (
            "tpl.flyer_sale",
            Category::Flyers,
            Size::new(210.0, 297.0),
            flyer_sale,
        ),
        (
            "tpl.flyer_event",
            Category::Flyers,
            Size::new(210.0, 297.0),
            flyer_event,
        ),
        (
            "tpl.poster_dots",
            Category::Posters,
            Size::new(297.0, 420.0),
            poster_dots,
        ),
        (
            "tpl.letterhead",
            Category::Stationery,
            Size::new(210.0, 297.0),
            letterhead,
        ),
        (
            "tpl.certificate",
            Category::Stationery,
            Size::new(297.0, 210.0),
            certificate,
        ),
        (
            "tpl.invoice",
            Category::Forms,
            Size::new(210.0, 297.0),
            invoice,
        ),
        (
            "tpl.banner_birthday",
            Category::Banners,
            Size::new(300.0, 100.0),
            banner_birthday,
        ),
        (
            "tpl.social_quote",
            Category::SocialMedia,
            px(1080.0, 1080.0),
            social_quote,
        ),
        (
            "tpl.label_round",
            Category::Labels,
            Size::new(60.0, 60.0),
            label_round,
        ),
    ]
}

/// The built-in templates, names translated.
pub fn built_in() -> Vec<Template> {
    table()
        .iter()
        .enumerate()
        .map(|(i, (key, cat, _, _))| Template {
            name: crate::i18n::tr(key),
            category: *cat,
            source: Source::BuiltIn(i),
        })
        .collect()
}

/// The page size of built-in template `i`.
pub fn built_in_size(i: usize) -> Option<Size> {
    table().get(i).map(|t| t.2)
}

/// The user's templates: the `.tdt` files of `dir`, by name.
pub fn user_templates(dir: Option<PathBuf>) -> Vec<Template> {
    let mut out = Vec::new();
    if crate::files::WEB {
        return out;
    }
    let Some(dir) = dir else { return out };
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) == Some("tdt") {
            let name = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            out.push(Template {
                name,
                category: Category::Mine,
                source: Source::File(p),
            });
        }
    }
    out.sort_by_key(|t| t.name.to_lowercase());
    out
}

/// The drawing a template makes, titled `title`.
pub fn document(t: &Template, title: &str) -> Option<Document> {
    match &t.source {
        Source::BuiltIn(i) => {
            let tbl = table();
            let (_, _, size, build) = tbl.get(*i)?;
            let mut d = Doc::new(crate::app::App::localized_document(title, *size));
            build(&mut d);
            Some(d.doc)
        }
        Source::File(p) => {
            let text = std::fs::read_to_string(p).ok()?;
            let mut doc = Document::from_json(&text).ok()?;
            doc.title = title.to_string();
            Some(doc)
        }
    }
}

/// Whether `t` matches a search text (in its name or category) and the
/// filters.
pub fn matches(t: &Template, search: &str, mine_only: Option<bool>, cat: Option<Category>) -> bool {
    if let Some(mine) = mine_only {
        if mine != matches!(t.source, Source::File(_)) {
            return false;
        }
    }
    if cat.is_some_and(|c| c != t.category) {
        return false;
    }
    let s = search.trim().to_lowercase();
    s.is_empty()
        || t.name.to_lowercase().contains(&s)
        || crate::i18n::tr(t.category.key())
            .to_lowercase()
            .contains(&s)
}

// ----- Drawing helpers ------------------------------------------------------

const FONT: &str = "Arial";

/// A document being drawn into its first page's first layer; y runs up
/// from the bottom of the page, as everywhere.
pub struct Doc {
    pub doc: Document,
}

fn rgb(hex: u32) -> Color {
    Color::rgb8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

impl Doc {
    fn new(doc: Document) -> Self {
        Doc { doc }
    }

    fn size(&self) -> Size {
        self.doc
            .pages
            .first()
            .map(|p| p.size)
            .unwrap_or(Size::new(210.0, 297.0))
    }

    fn push(&mut self, kind: ShapeKind, fill: Fill, stroke: Option<Stroke>) {
        let id = self.doc.ids_mut().shape();
        let mut s = Shape::new(id, kind);
        s.fill = fill;
        s.stroke = stroke;
        if let Some(l) = self
            .doc
            .pages
            .first_mut()
            .and_then(|p| p.layers.first_mut())
        {
            l.shapes.push(s);
        }
    }

    fn rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: u32) {
        self.push(
            ShapeKind::Rect {
                rect: Rect::new(x0, y0, x1, y1),
                radius: 0.0,
                corners: None,
            },
            Fill::Solid(rgb(color)),
            None,
        );
    }

    fn gradient_rect(&mut self, r: Rect, from: u32, to: u32, angle: f64) {
        self.push(
            ShapeKind::Rect {
                rect: r,
                radius: 0.0,
                corners: None,
            },
            Fill::Fountain(Fountain::two(
                FountainKind::Linear,
                rgb(from),
                rgb(to),
                angle,
            )),
            None,
        );
    }

    fn frame(&mut self, r: Rect, color: u32, width: f64, radius: f64) {
        self.push(
            ShapeKind::Rect {
                rect: r,
                radius,
                corners: None,
            },
            Fill::None,
            Some(Stroke::new(rgb(color), width)),
        );
    }

    fn circle(&mut self, cx: f64, cy: f64, r: f64, color: u32) {
        self.push(
            ShapeKind::Ellipse {
                rect: Rect::new(cx - r, cy - r, cx + r, cy + r),
                arc: None,
            },
            Fill::Solid(rgb(color)),
            None,
        );
    }

    fn ring(&mut self, cx: f64, cy: f64, r: f64, color: u32, width: f64) {
        self.push(
            ShapeKind::Ellipse {
                rect: Rect::new(cx - r, cy - r, cx + r, cy + r),
                arc: None,
            },
            Fill::None,
            Some(Stroke::new(rgb(color), width)),
        );
    }

    fn line(&mut self, a: Point, b: Point, color: u32, width: f64) {
        let mut path = tracedraw_core::BezPath::new();
        path.move_to(a);
        path.line_to(b);
        self.push(
            ShapeKind::Path {
                path,
                closed: false,
            },
            Fill::None,
            Some(Stroke::new(rgb(color), width)),
        );
    }

    /// Paragraph text in a frame whose top left corner is (`x`, `top`).
    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        x: f64,
        top: f64,
        w: f64,
        h: f64,
        text: &str,
        size_pt: f64,
        bold: bool,
        align: TextAlign,
        color: u32,
    ) {
        let span = TextSpan {
            bold,
            ..TextSpan::new(text, FONT, size_pt)
        };
        self.push(
            ShapeKind::Text {
                spans: vec![span],
                // A frame's origin is its bottom left corner.
                origin: Point::new(x, top - h),
                frame: Some(Size::new(w, h)),
                align,
                para: ParagraphStyle::default(),
                on_path: None,
            },
            Fill::Solid(rgb(color)),
            None,
        );
    }

    /// Spaced text (tracking in percent of the em).
    #[allow(clippy::too_many_arguments)]
    fn spaced(
        &mut self,
        x: f64,
        top: f64,
        w: f64,
        text: &str,
        size_pt: f64,
        tracking: f64,
        align: TextAlign,
        color: u32,
    ) {
        let span = TextSpan {
            bold: true,
            tracking_pct: tracking,
            ..TextSpan::new(text, FONT, size_pt)
        };
        let h = size_pt * 0.3528 * 1.6;
        self.push(
            ShapeKind::Text {
                spans: vec![span],
                // A frame's origin is its bottom left corner.
                origin: Point::new(x, top - h),
                frame: Some(Size::new(w, h)),
                align,
                para: ParagraphStyle::default(),
                on_path: None,
            },
            Fill::Solid(rgb(color)),
            None,
        );
    }

    fn star(&mut self, cx: f64, cy: f64, r: f64, points: u32, color: u32) {
        self.push(
            ShapeKind::Polygon {
                rect: Rect::new(cx - r, cy - r, cx + r, cy + r),
                points,
                sharpness: 0.5,
                complex: None,
                twist: 0.0,
            },
            Fill::Solid(rgb(color)),
            None,
        );
    }

    /// Turn the last shape by `deg` degrees about (`cx`, `cy`).
    fn turn_last(&mut self, deg: f64, cx: f64, cy: f64) {
        if let Some(s) = self
            .doc
            .pages
            .first_mut()
            .and_then(|p| p.layers.first_mut())
            .and_then(|l| l.shapes.last_mut())
        {
            let c = tracedraw_core::Vec2::new(cx, cy);
            s.transform =
                Affine::translate(c) * Affine::rotate(deg.to_radians()) * Affine::translate(-c);
        }
    }
}

const LOREM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.";

// ----- The designs ----------------------------------------------------------

fn blank(_: &mut Doc) {}

fn card_band(d: &mut Doc) {
    d.rect(0.0, 0.0, 90.0, 50.0, 0xffffff);
    d.rect(0.0, 0.0, 90.0, 9.0, 0x1f8a8a);
    d.rect(0.0, 9.0, 90.0, 10.0, 0xf2b134);
    d.circle(14.0, 34.0, 7.0, 0x1f8a8a);
    d.circle(14.0, 34.0, 3.5, 0xf2b134);
    d.text(
        26.0,
        41.0,
        60.0,
        8.0,
        "Lorem Ipsum",
        13.0,
        true,
        TextAlign::Left,
        0x1d2b36,
    );
    d.text(
        26.0,
        33.5,
        60.0,
        6.0,
        "Dolor sit amet",
        8.0,
        false,
        TextAlign::Left,
        0x1f8a8a,
    );
    d.text(
        4.0,
        7.2,
        82.0,
        5.0,
        "+00 000 000 000   lorem@ipsum.com",
        7.0,
        false,
        TextAlign::Center,
        0xffffff,
    );
}

fn card_dark(d: &mut Doc) {
    d.rect(0.0, 0.0, 90.0, 50.0, 0x1d2b36);
    d.rect(0.0, 0.0, 30.0, 50.0, 0xe8505b);
    d.ring(15.0, 25.0, 9.0, 0xffffff, 1.2);
    d.star(15.0, 25.0, 5.5, 5, 0xffffff);
    d.text(
        36.0,
        36.0,
        50.0,
        8.0,
        "LOREM IPSUM",
        12.0,
        true,
        TextAlign::Left,
        0xffffff,
    );
    d.text(
        36.0,
        28.0,
        50.0,
        6.0,
        "Consectetur",
        8.0,
        false,
        TextAlign::Left,
        0xe8505b,
    );
    d.line(
        Point::new(36.0, 20.0),
        Point::new(84.0, 20.0),
        0x5c6b77,
        0.3,
    );
    d.text(
        36.0,
        17.0,
        50.0,
        12.0,
        "+00 000 000 000\nlorem@ipsum.com",
        7.0,
        false,
        TextAlign::Left,
        0xd0d6db,
    );
}

fn flyer_sale(d: &mut Doc) {
    d.rect(0.0, 0.0, 210.0, 297.0, 0xffffff);
    d.rect(0.0, 110.0, 210.0, 297.0, 0xf7d21c);
    d.circle(158.0, 120.0, 34.0, 0x1d2b36);
    d.text(
        124.0,
        134.0,
        68.0,
        14.0,
        "50%",
        40.0,
        true,
        TextAlign::Center,
        0xf7d21c,
    );
    d.text(
        124.0,
        115.0,
        68.0,
        8.0,
        "OFF",
        18.0,
        true,
        TextAlign::Center,
        0xffffff,
    );
    d.text(
        20.0,
        270.0,
        170.0,
        20.0,
        "LOREM",
        72.0,
        true,
        TextAlign::Left,
        0x1d2b36,
    );
    d.text(
        20.0,
        240.0,
        170.0,
        20.0,
        "IPSUM",
        72.0,
        true,
        TextAlign::Left,
        0x1d2b36,
    );
    d.text(
        20.0,
        205.0,
        110.0,
        40.0,
        LOREM,
        12.0,
        false,
        TextAlign::Left,
        0x1d2b36,
    );
    d.text(
        20.0,
        80.0,
        170.0,
        40.0,
        LOREM,
        11.0,
        false,
        TextAlign::Left,
        0x5c6b77,
    );
    d.rect(20.0, 20.0, 190.0, 34.0, 0x1d2b36);
    d.text(
        20.0,
        30.5,
        170.0,
        8.0,
        "WWW.LOREMIPSUM.COM",
        14.0,
        true,
        TextAlign::Center,
        0xffffff,
    );
}

fn flyer_event(d: &mut Doc) {
    d.rect(0.0, 0.0, 210.0, 297.0, 0x20214f);
    for (i, c) in [0xe8505b, 0xf2b134, 0x2fb5a8, 0x8a6fdf].iter().enumerate() {
        d.circle(105.0, 190.0, 80.0 - i as f64 * 18.0, *c);
    }
    d.circle(105.0, 190.0, 8.0, 0x20214f);
    d.spaced(
        10.0,
        92.0,
        190.0,
        "LOREM FEST",
        44.0,
        12.0,
        TextAlign::Center,
        0xffffff,
    );
    d.text(
        10.0,
        68.0,
        190.0,
        10.0,
        "00.00.0000  ·  00:00",
        18.0,
        false,
        TextAlign::Center,
        0xf2b134,
    );
    d.text(
        30.0,
        52.0,
        150.0,
        30.0,
        LOREM,
        11.0,
        false,
        TextAlign::Center,
        0xd0d6db,
    );
}

fn poster_dots(d: &mut Doc) {
    d.rect(0.0, 0.0, 297.0, 420.0, 0xf4efe6);
    let colours = [0xe8505b, 0xf2b134, 0x2fb5a8, 0x20214f];
    for row in 0..5 {
        for col in 0..4 {
            let c = colours[(row + col) % colours.len()];
            let (cx, cy) = (60.0 + col as f64 * 59.0, 380.0 - row as f64 * 50.0);
            d.circle(cx, cy, 20.0 - ((row * 3 + col) % 4) as f64 * 3.0, c);
        }
    }
    d.rect(0.0, 0.0, 297.0, 120.0, 0x20214f);
    d.text(
        24.0,
        105.0,
        250.0,
        30.0,
        "LOREM IPSUM",
        96.0,
        true,
        TextAlign::Left,
        0xf4efe6,
    );
    d.text(
        24.0,
        70.0,
        180.0,
        50.0,
        LOREM,
        12.0,
        false,
        TextAlign::Left,
        0xd0d6db,
    );
}

fn letterhead(d: &mut Doc) {
    let s = d.size();
    d.gradient_rect(
        Rect::new(0.0, s.height - 30.0, s.width, s.height),
        0x2fb5a8,
        0x6fd3c7,
        0.0,
    );
    d.circle(25.0, s.height - 15.0, 8.0, 0xffffff);
    d.circle(25.0, s.height - 15.0, 4.0, 0x2fb5a8);
    d.text(
        38.0,
        s.height - 9.0,
        150.0,
        10.0,
        "Lorem Ipsum",
        20.0,
        true,
        TextAlign::Left,
        0xffffff,
    );
    d.text(
        20.0,
        s.height - 45.0,
        170.0,
        30.0,
        "Dolor Sit\nAmet Street 00\n0000 Consectetur",
        10.0,
        false,
        TextAlign::Left,
        0x1d2b36,
    );
    d.text(
        20.0,
        s.height - 80.0,
        170.0,
        150.0,
        &format!("{LOREM}\n\n{LOREM}\n\n{LOREM}"),
        11.0,
        false,
        TextAlign::Left,
        0x1d2b36,
    );
    d.line(
        Point::new(20.0, 22.0),
        Point::new(190.0, 22.0),
        0x2fb5a8,
        0.5,
    );
    d.text(
        20.0,
        18.0,
        170.0,
        8.0,
        "+00 000 000 000   ·   lorem@ipsum.com   ·   www.loremipsum.com",
        8.0,
        false,
        TextAlign::Center,
        0x5c6b77,
    );
}

fn certificate(d: &mut Doc) {
    d.rect(0.0, 0.0, 297.0, 210.0, 0xfffdf6);
    d.frame(Rect::new(10.0, 10.0, 287.0, 200.0), 0xb08d3c, 2.0, 0.0);
    d.frame(Rect::new(15.0, 15.0, 282.0, 195.0), 0xb08d3c, 0.5, 0.0);
    d.spaced(
        20.0,
        175.0,
        257.0,
        "CERTIFICATE",
        40.0,
        20.0,
        TextAlign::Center,
        0x1d2b36,
    );
    d.text(
        20.0,
        145.0,
        257.0,
        10.0,
        "Lorem ipsum dolor sit amet",
        14.0,
        false,
        TextAlign::Center,
        0x5c6b77,
    );
    d.text(
        20.0,
        125.0,
        257.0,
        20.0,
        "Consectetur Adipiscing",
        32.0,
        true,
        TextAlign::Center,
        0xb08d3c,
    );
    d.text(
        48.0,
        95.0,
        200.0,
        30.0,
        LOREM,
        11.0,
        false,
        TextAlign::Center,
        0x5c6b77,
    );
    d.star(148.5, 45.0, 16.0, 12, 0xb08d3c);
    d.circle(148.5, 45.0, 10.0, 0xf2d27a);
    for x in [40.0, 197.0] {
        d.line(
            Point::new(x, 40.0),
            Point::new(x + 60.0, 40.0),
            0x1d2b36,
            0.4,
        );
    }
}

fn invoice(d: &mut Doc) {
    d.rect(0.0, 0.0, 210.0, 297.0, 0xffffff);
    d.rect(0.0, 262.0, 210.0, 297.0, 0x20214f);
    d.text(
        20.0,
        287.0,
        100.0,
        15.0,
        "INVOICE",
        28.0,
        true,
        TextAlign::Left,
        0xffffff,
    );
    d.text(
        110.0,
        286.0,
        80.0,
        20.0,
        "Lorem Ipsum\nDolor Sit Amet 00",
        9.0,
        false,
        TextAlign::Right,
        0xd0d6db,
    );
    d.text(
        20.0,
        250.0,
        90.0,
        25.0,
        "Lorem ipsum:\nDolor sit amet\nConsectetur 00",
        10.0,
        false,
        TextAlign::Left,
        0x1d2b36,
    );
    d.text(
        120.0,
        250.0,
        70.0,
        25.0,
        "No. 0000\n00.00.0000",
        10.0,
        false,
        TextAlign::Right,
        0x1d2b36,
    );
    // The table: a header row and ten lines.
    let (x0, x1, top) = (20.0, 190.0, 215.0);
    d.rect(x0, top - 9.0, x1, top, 0xe8505b);
    let cols = [
        (x0, "QTY"),
        (40.0, "DESCRIPTION"),
        (130.0, "PRICE"),
        (160.0, "TOTAL"),
    ];
    for (x, h) in cols {
        d.text(
            x + 2.0,
            top - 2.0,
            30.0,
            6.0,
            h,
            9.0,
            true,
            TextAlign::Left,
            0xffffff,
        );
    }
    for i in 0..10 {
        let y = top - 9.0 - (i + 1) as f64 * 9.0;
        if i % 2 == 1 {
            d.rect(x0, y, x1, y + 9.0, 0xf3f4f6);
        }
    }
    let bottom = top - 9.0 - 90.0;
    for x in [40.0, 130.0, 160.0] {
        d.line(Point::new(x, bottom), Point::new(x, top), 0xd0d6db, 0.25);
    }
    d.frame(Rect::new(x0, bottom, x1, top), 0x1d2b36, 0.35, 0.0);
    d.rect(130.0, bottom - 12.0, x1, bottom, 0x20214f);
    d.text(
        132.0,
        bottom - 3.0,
        56.0,
        6.0,
        "TOTAL",
        11.0,
        true,
        TextAlign::Left,
        0xffffff,
    );
    d.text(
        20.0,
        40.0,
        170.0,
        20.0,
        LOREM,
        8.0,
        false,
        TextAlign::Left,
        0x5c6b77,
    );
}

fn banner_birthday(d: &mut Doc) {
    d.rect(0.0, 0.0, 300.0, 100.0, 0xffffff);
    let colours = [0x20214f, 0x8a6fdf, 0xe8505b, 0xf2b134];
    for (k, cx) in [-10.0, 60.0, 240.0, 310.0].into_iter().enumerate() {
        for i in 0..colours.len() {
            let c = colours[(i + k) % colours.len()];
            d.circle(cx, 0.0, 40.0 - i as f64 * 9.0, c);
        }
    }
    d.rect(60.0, 42.0, 240.0, 70.0, 0xf2b134);
    d.spaced(
        60.0,
        64.0,
        180.0,
        "HAPPY BIRTHDAY",
        40.0,
        4.0,
        TextAlign::Center,
        0x20214f,
    );
}

fn social_quote(d: &mut Doc) {
    let s = d.size();
    d.gradient_rect(
        Rect::new(0.0, 0.0, s.width, s.height),
        0x8a6fdf,
        0xe8505b,
        45.0,
    );
    d.text(
        30.0,
        s.height - 40.0,
        s.width - 60.0,
        60.0,
        "\u{201c}",
        160.0,
        true,
        TextAlign::Left,
        0xffffff,
    );
    d.text(
        30.0,
        s.height - 110.0,
        s.width - 60.0,
        100.0,
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        80.0,
        true,
        TextAlign::Left,
        0xffffff,
    );
    d.text(
        30.0,
        60.0,
        s.width - 60.0,
        20.0,
        "LOREM IPSUM",
        24.0,
        false,
        TextAlign::Left,
        0xffe7a8,
    );
}

fn label_round(d: &mut Doc) {
    d.circle(30.0, 30.0, 30.0, 0x2fb5a8);
    d.ring(30.0, 30.0, 26.0, 0xffffff, 0.6);
    d.star(30.0, 44.0, 4.0, 5, 0xffffff);
    d.turn_last(0.0, 30.0, 44.0);
    d.text(
        5.0,
        37.0,
        50.0,
        12.0,
        "LOREM",
        18.0,
        true,
        TextAlign::Center,
        0xffffff,
    );
    d.text(
        5.0,
        25.0,
        50.0,
        8.0,
        "IPSUM DOLOR",
        8.0,
        false,
        TextAlign::Center,
        0xffffff,
    );
    d.text(
        5.0,
        16.0,
        50.0,
        8.0,
        "0000",
        8.0,
        false,
        TextAlign::Center,
        0xffe7a8,
    );
}

/// A thumbnail of the template's first page, `px` on its longer side:
/// width, height and premultiplied RGBA.
pub fn thumbnail(doc: &Document, px: u32) -> Option<(u32, u32, Vec<u8>)> {
    let page = doc.pages.first()?;
    let longer = page.size.width.max(page.size.height);
    if longer <= 0.0 {
        return None;
    }
    let dpi = (px as f64 / longer * 25.4).max(1.0);
    let pm = tracedraw_render::render_page_image(doc, page.id, dpi)?;
    Some((pm.width(), pm.height(), pm.data().to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_built_in_template_makes_a_drawing_of_its_size() {
        let all = built_in();
        assert!(all.len() >= 15);
        for (i, t) in all.iter().enumerate() {
            let doc = document(t, "T").expect("drawing");
            let size = built_in_size(i).expect("size");
            assert_eq!(doc.pages[0].size, size, "{}", t.name);
            if t.category != Category::Blank {
                assert!(doc.pages[0].layers[0].shapes.len() >= 4, "{}", t.name);
            }
            let (thumb_w, thumb_h, rgba) = thumbnail(&doc, 64).expect("thumbnail");
            assert!(thumb_w.max(thumb_h) <= 66 && rgba.len() == (thumb_w * thumb_h * 4) as usize);
        }
    }

    #[test]
    fn search_and_filters() {
        let all = built_in();
        let cards: Vec<_> = all
            .iter()
            .filter(|t| matches(t, "", None, Some(Category::BusinessCards)))
            .collect();
        assert_eq!(cards.len(), 2);
        assert!(all.iter().all(|t| !matches(t, "", Some(true), None)));
        assert!(all.iter().any(|t| matches(t, "INVOICE", None, None)));
        assert!(!all.iter().any(|t| matches(t, "no such thing", None, None)));
        let mine = Template {
            name: "Mine".into(),
            category: Category::Mine,
            source: Source::File(PathBuf::from("/nowhere/mine.tdt")),
        };
        assert!(matches(&mine, "mi", Some(true), None));
        // A file that cannot be read gives no drawing.
        assert!(document(&mine, "T").is_none());
    }
}

#[cfg(test)]
mod sheet {
    /// Writes every template to `$TPL_SHEET/<n>.png` (run by hand).
    #[test]
    #[ignore]
    fn write_sheet() {
        let Ok(dir) = std::env::var("TPL_SHEET") else {
            return;
        };
        for (i, t) in super::built_in().iter().enumerate() {
            let doc = super::document(t, "T").expect("doc");
            let pm = tracedraw_render::render_page_image(&doc, doc.pages[0].id, 60.0).expect("pm");
            pm.save_png(format!("{dir}/{i:02}.png")).expect("png");
        }
    }
}
