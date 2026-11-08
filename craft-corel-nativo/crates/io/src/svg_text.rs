//! Text in SVG import (#1820).
//!
//! `usvg` is built without its text engine, so it drops every `<text>`
//! element. Before `usvg` reads the file, each `<text>` is read here into
//! editable text (its lines, its spans and their styles, with the CSS
//! cascade of SVG 1.1 section 6 and the white space rules of section
//! 10.15) and replaced in the source by an invisible marker path that
//! keeps the element's transform, opacity, clip, mask and display. `usvg`
//! then places the marker like any other element (nested transforms,
//! `use`, `switch`, clips) and `svg_import` turns each marker back into
//! text objects.
//!
//! Positions follow SVG 1.1 section 10: a text chunk starts at the
//! text's `x`/`y` and at every `tspan` with an `x`; `dx`/`dy` shift the
//! characters that follow. Only the first value of a coordinate list is
//! read; the characters after it are laid out by the text engine.

use std::collections::HashMap;

use roxmltree::{Document, Node, ParsingOptions};
use tracedraw_core::{
    document::{LeadingUnit, ParagraphStyle, Shape, ShapeKind, TextAlign},
    geometry::{Affine, Point},
    id::IdSource,
    Color, Fill, ShapeId, Stroke, TextSpan,
};

const SVG_NS: &str = "http://www.w3.org/2000/svg";
const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// Where the marker path of text number `n` starts: (`MARKER_X`, `n`).
/// Elements copied by `use` lose their ids, so the number travels in
/// the geometry (exact in the single precision `usvg` keeps).
const MARKER_X: f32 = -7_654_321.0;

/// CSS px to points.
const PX_TO_PT: f64 = 0.75;
/// CSS px (SVG user units) to millimetres.
const PX_TO_MM: f64 = 25.4 / 96.0;

/// One `<text>` element, in its own user space (px, Y down).
#[derive(Debug, Clone, Default)]
pub(crate) struct SvgText {
    /// The element's `id`.
    pub name: Option<String>,
    pub chunks: Vec<Chunk>,
    /// The text element's own fill; spans that differ carry their own.
    pub fill: Fill,
    /// Outline colour and width (user units).
    pub stroke: Option<(Color, f64)>,
}

/// A text chunk: characters laid out from one absolute position.
#[derive(Debug, Clone)]
pub(crate) struct Chunk {
    /// Start of the baseline (after `dominant-baseline`), user units.
    pub x: f64,
    pub y: f64,
    pub align: TextAlign,
    /// How far `y` was moved down for `dominant-baseline` (px), taken
    /// off again for a chunk that continues the line.
    pub baseline: f64,
    /// Sizes in points of user space (px * 0.75).
    pub spans: Vec<TextSpan>,
}

impl Chunk {
    fn is_empty(&self) -> bool {
        self.spans.iter().all(|s| s.text.is_empty())
    }
}

/// Read every `<text>` element of `svg` and replace it with a marker
/// path. Gives the source to hand to `usvg` and the texts, indexed by
/// the number [`marker_index`] reads back. Text that cannot be read leaves the
/// source as it is (and `usvg` drops it, as before).
pub(crate) fn prepare(svg: &str) -> (String, Vec<SvgText>) {
    let opt = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let Ok(doc) = Document::parse_with_options(svg, opt) else {
        return (svg.to_string(), Vec::new());
    };
    let css_sources = style_sheets(&doc);
    let css = Css::new(&css_sources);
    let ctx = Ctx {
        css,
        cache: Default::default(),
        viewport: viewport(&doc),
        ids: doc
            .descendants()
            .filter(|n| n.is_element())
            .filter_map(|n| n.attribute("id").map(|id| (id, n)))
            .collect(),
    };
    let mut texts = Vec::new();
    let mut edits: Vec<(std::ops::Range<usize>, String)> = Vec::new();
    for node in doc.descendants().filter(|n| is_svg(*n, "text")) {
        // Text nested in text, or inside a clip, mask, pattern or marker
        // (where a marker path would change the drawing), stays as it is.
        if node.ancestors().skip(1).any(|a| {
            ["text", "clipPath", "mask", "pattern", "marker"]
                .iter()
                .any(|name| is_svg(a, name))
        }) {
            continue;
        }
        let range = node.range();
        if svg.get(range.clone()).is_none_or(|s| !s.starts_with('<'))
            || edits.last().is_some_and(|(r, _)| r.end > range.start)
        {
            continue;
        }
        let hidden = matches!(ctx.own(node, "display").as_deref(), Some("none"))
            || matches!(
                ctx.inherited(node, "visibility").as_deref(),
                Some("hidden" | "collapse")
            );
        let text = if hidden {
            None
        } else {
            Some(read_text(node, &ctx))
        };
        let replacement = match text {
            Some(t) if t.chunks.iter().any(|c| !c.is_empty()) => {
                let marker = marker_path(node, &ctx, texts.len());
                texts.push(t);
                marker
            }
            _ => String::new(),
        };
        edits.push((range, replacement));
    }
    if edits.is_empty() {
        return (svg.to_string(), texts);
    }
    let mut out = String::with_capacity(svg.len());
    let mut at = 0;
    for (range, rep) in &edits {
        out.push_str(&svg[at..range.start]);
        out.push_str(rep);
        at = range.end;
    }
    out.push_str(&svg[at..]);
    (out, texts)
}

/// The marker path for text number `n`: invisible, with the element's
/// transform and the properties `usvg` applies to it as a whole.
fn marker_path(node: Node, ctx: &Ctx, n: usize) -> String {
    let mut s = format!("<path d=\"M{MARKER_X} {n}h1v1h-1z\" fill=\"none\" stroke=\"none\"");
    for name in [
        "transform",
        "clip-path",
        "mask",
        "systemLanguage",
        "requiredFeatures",
        "requiredExtensions",
    ] {
        if let Some(v) = node.attribute(name) {
            s.push_str(&format!(" {name}=\"{}\"", escape(v)));
        }
    }
    if let Some(v) = ctx.own(node, "opacity") {
        s.push_str(&format!(" opacity=\"{}\"", escape(&v)));
    }
    s.push_str("/>");
    s
}

/// The text number a path from [`prepare`]'s source stands for.
pub(crate) fn marker_index(p: &usvg::Path) -> Option<usize> {
    if p.is_visible() || p.fill().is_some() || p.stroke().is_some() {
        return None;
    }
    let first = p.data().points().first()?;
    (first.x == MARKER_X && first.y >= 0.0 && first.y.fract() == 0.0).then_some(first.y as usize)
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn is_svg(n: Node, name: &str) -> bool {
    n.is_element() && n.tag_name().name() == name && n.tag_name().namespace() == Some(SVG_NS)
}

/// The text of every `<style>` element that holds CSS.
fn style_sheets(doc: &Document) -> Vec<String> {
    doc.descendants()
        .filter(|n| is_svg(*n, "style"))
        .filter(|n| {
            n.attribute("type")
                .is_none_or(|t| t.is_empty() || t.eq_ignore_ascii_case("text/css"))
        })
        .map(|n| {
            n.descendants()
                .filter(|c| c.is_text())
                .filter_map(|c| c.text())
                .collect::<String>()
        })
        .collect()
}

/// Width and height of the outermost viewport, for percentages.
fn viewport(doc: &Document) -> (f64, f64) {
    let root = doc.root_element();
    if let Some(vb) = root.attribute("viewBox") {
        let v: Vec<f64> = svgtypes::NumberListParser::from(vb)
            .filter_map(|n| n.ok())
            .collect();
        if v.len() == 4 && v[2] > 0.0 && v[3] > 0.0 {
            return (v[2], v[3]);
        }
    }
    let len = |name: &str| {
        root.attribute(name)
            .and_then(|v| v.parse::<svgtypes::Length>().ok())
            .filter(|l| l.unit != svgtypes::LengthUnit::Percent && l.number > 0.0)
            .map(|l| l.number * unit_px(l.unit, 16.0))
    };
    (
        len("width").unwrap_or(100.0),
        len("height").unwrap_or(100.0),
    )
}

/// Size of one `unit` in px, for a font size of `font_px`.
fn unit_px(unit: svgtypes::LengthUnit, font_px: f64) -> f64 {
    use svgtypes::LengthUnit as U;
    match unit {
        U::None | U::Px | U::Percent => 1.0,
        U::Em => font_px,
        U::Ex => font_px / 2.0,
        U::In => 96.0,
        U::Cm => 96.0 / 2.54,
        U::Mm => 96.0 / 25.4,
        U::Pt => 4.0 / 3.0,
        U::Pc => 16.0,
    }
}

/// The style sheets, with each rule's specificity and place.
struct Css<'a> {
    rules: Vec<([u8; 3], usize, simplecss::Rule<'a>)>,
}

impl<'a> Css<'a> {
    fn new(sources: &'a [String]) -> Self {
        let mut sheet = simplecss::StyleSheet::new();
        for s in sources {
            sheet.parse_more(s);
        }
        let rules = sheet
            .rules
            .into_iter()
            .enumerate()
            .map(|(i, r)| (r.selector.specificity(), i, r))
            .collect::<Vec<_>>();
        let mut rules = rules;
        rules.sort_by_key(|(spec, order, _)| (*spec, *order));
        Css { rules }
    }
}

/// A roxmltree element as the CSS selector matcher sees it.
#[derive(Clone, Copy)]
struct El<'a, 'input>(Node<'a, 'input>);

impl simplecss::Element for El<'_, '_> {
    fn parent_element(&self) -> Option<Self> {
        self.0.parent_element().map(El)
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.0.prev_sibling_element().map(El)
    }

    fn has_local_name(&self, name: &str) -> bool {
        self.0.tag_name().name() == name
    }

    fn attribute_matches(&self, local_name: &str, operator: simplecss::AttributeOperator) -> bool {
        self.0
            .attribute(local_name)
            .is_some_and(|v| operator.matches(v))
    }

    fn pseudo_class_matches(&self, class: simplecss::PseudoClass) -> bool {
        match class {
            simplecss::PseudoClass::FirstChild => self.0.prev_sibling_element().is_none(),
            _ => false,
        }
    }
}

struct Ctx<'a, 'input> {
    css: Css<'a>,
    /// [`Ctx::props`] of the elements looked at so far.
    cache: std::cell::RefCell<HashMap<roxmltree::NodeId, std::rc::Rc<HashMap<String, String>>>>,
    viewport: (f64, f64),
    ids: HashMap<&'input str, Node<'input, 'input>>,
}

/// Set `name: value` in `props`, the `font` shorthand as its parts.
fn declare(props: &mut HashMap<String, String>, name: &str, value: &str) {
    if name != "font" {
        props.insert(name.to_string(), value.trim().to_string());
        return;
    }
    if let Ok(f) = svgtypes::FontShorthand::from_str(value) {
        for (k, v) in [
            ("font-family", f.font_family),
            ("font-size", f.font_size),
            ("font-weight", f.font_weight.unwrap_or("normal")),
            ("font-style", f.font_style.unwrap_or("normal")),
        ] {
            props.insert(k.to_string(), v.trim().to_string());
        }
    }
}

impl Ctx<'_, '_> {
    /// The value an element sets itself for `prop`; `inherit` counts as
    /// not set.
    fn own(&self, n: Node, prop: &str) -> Option<String> {
        let props = self.props(n);
        let v = props.get(prop)?;
        (v != "inherit").then(|| v.clone())
    }

    /// Every property an element sets itself: its presentation
    /// attributes, overridden by the style sheet rules that match it
    /// (less specific first, then in order), overridden by its `style`
    /// attribute. Worked out once per element.
    fn props(&self, n: Node) -> std::rc::Rc<HashMap<String, String>> {
        if let Some(p) = self.cache.borrow().get(&n.id()) {
            return p.clone();
        }
        let mut props = HashMap::new();
        if let Some(v) = n.attribute("font") {
            declare(&mut props, "font", v);
        }
        for a in n.attributes().filter(|a| a.namespace().is_none()) {
            if a.name() != "font" && a.name() != "style" {
                declare(&mut props, a.name(), a.value());
            }
        }
        // The rules are kept sorted by specificity, then order.
        for (_, _, rule) in &self.css.rules {
            if rule.selector.matches(&El(n)) {
                for d in &rule.declarations {
                    declare(&mut props, d.name, d.value);
                }
            }
        }
        if let Some(style) = n.attribute("style") {
            for d in simplecss::DeclarationTokenizer::from(style) {
                declare(&mut props, d.name, d.value);
            }
        }
        let props = std::rc::Rc::new(props);
        self.cache.borrow_mut().insert(n.id(), props.clone());
        props
    }

    /// The value of an inherited property: the nearest element that sets it.
    fn inherited(&self, n: Node, prop: &str) -> Option<String> {
        n.ancestors()
            .filter(|a| a.is_element())
            .find_map(|a| self.own(a, prop))
    }

    /// Computed font size in px.
    fn font_size(&self, n: Node) -> f64 {
        let chain: Vec<Node> = n.ancestors().filter(|a| a.is_element()).collect();
        let mut size = 16.0;
        for a in chain.iter().rev() {
            let Some(v) = self.own(*a, "font-size") else {
                continue;
            };
            size = match v.as_str() {
                "xx-small" => 9.0,
                "x-small" => 10.0,
                "small" => 13.0,
                "medium" => 16.0,
                "large" => 18.0,
                "x-large" => 24.0,
                "xx-large" => 32.0,
                "larger" => size * 1.2,
                "smaller" => size / 1.2,
                _ => match v.parse::<svgtypes::Length>() {
                    Ok(l) if l.unit == svgtypes::LengthUnit::Percent => size * l.number / 100.0,
                    Ok(l) => l.number * unit_px(l.unit, size),
                    Err(_) => size,
                },
            };
        }
        if size.is_finite() && size > 0.0 {
            size
        } else {
            16.0
        }
    }

    /// A length value in px; percentages of `percent_of`.
    fn length(&self, v: &str, font_px: f64, percent_of: f64) -> Option<f64> {
        let l = v.parse::<svgtypes::Length>().ok()?;
        let px = if l.unit == svgtypes::LengthUnit::Percent {
            l.number * percent_of / 100.0
        } else {
            l.number * unit_px(l.unit, font_px)
        };
        px.is_finite().then_some(px)
    }

    /// The first value of a coordinate list attribute, in px.
    fn first_coord(&self, n: Node, name: &str, font_px: f64, percent_of: f64) -> Option<f64> {
        let v = n.attribute(name)?;
        let l = svgtypes::LengthListParser::from(v).next()?.ok()?;
        let px = if l.unit == svgtypes::LengthUnit::Percent {
            l.number * percent_of / 100.0
        } else {
            l.number * unit_px(l.unit, font_px)
        };
        px.is_finite().then_some(px)
    }

    /// The colour of a paint property (`fill`, `stroke`); `None` for
    /// `none`. Gradients and patterns give their first stop colour, or
    /// their fallback.
    fn paint(&self, n: Node, prop: &str, default: Option<Color>) -> Option<Color> {
        let Some(v) = self.inherited(n, prop) else {
            return default;
        };
        let to_color = |c: svgtypes::Color| Color::rgb8(c.red, c.green, c.blue);
        match svgtypes::Paint::from_str(&v) {
            Ok(svgtypes::Paint::None) => None,
            Ok(svgtypes::Paint::Color(c)) => Some(to_color(c)),
            Ok(svgtypes::Paint::CurrentColor) => Some(
                self.inherited(n, "color")
                    .and_then(|c| c.parse::<svgtypes::Color>().ok())
                    .map(to_color)
                    .unwrap_or(Color::BLACK),
            ),
            Ok(svgtypes::Paint::FuncIRI(id, fallback)) => self
                .first_stop(id)
                .or(match fallback {
                    Some(svgtypes::PaintFallback::Color(c)) => Some(to_color(c)),
                    Some(svgtypes::PaintFallback::None) => None,
                    _ => Some(Color::BLACK),
                })
                .or(Some(Color::BLACK)),
            Ok(_) => Some(Color::BLACK),
            Err(_) => default,
        }
    }

    /// The colour of a gradient's first stop, following one `href`.
    fn first_stop(&self, id: &str) -> Option<Color> {
        let mut node = *self.ids.get(id)?;
        for _ in 0..4 {
            let stop = node.children().find(|c| is_svg(*c, "stop"));
            if let Some(stop) = stop {
                let c = self
                    .own(stop, "stop-color")
                    .and_then(|c| c.parse::<svgtypes::Color>().ok())?;
                return Some(Color::rgb8(c.red, c.green, c.blue));
            }
            let href = node
                .attribute(("http://www.w3.org/1999/xlink", "href"))
                .or_else(|| node.attribute("href"))?;
            node = *self.ids.get(href.strip_prefix('#')?)?;
        }
        None
    }
}

/// How far the baseline lies below the point `dominant-baseline` or
/// `alignment-baseline` names, in ems. The font's own metrics are not
/// known here; these are the usual proportions of a Latin font.
fn baseline_offset(v: &str) -> f64 {
    match v {
        "middle" | "mathematical" => 0.25,
        "central" => 0.35,
        "hanging" => 0.8,
        "text-top" | "text-before-edge" | "before-edge" => 0.9,
        "ideographic" | "text-bottom" | "text-after-edge" | "after-edge" => -0.2,
        _ => 0.0,
    }
}

/// Reads one text element.
struct Reader<'c, 'a, 'input> {
    ctx: &'c Ctx<'a, 'input>,
    chunks: Vec<Chunk>,
    /// Shift of the characters that follow, from `dx`/`dy` since the
    /// chunk started (px).
    dx: f64,
    dy: f64,
    /// The last character added was a space, or none was added to this
    /// chunk yet (white space collapsing).
    after_space: bool,
}

fn read_text(text: Node, ctx: &Ctx) -> SvgText {
    let mut r = Reader {
        ctx,
        chunks: Vec::new(),
        dx: 0.0,
        dy: 0.0,
        after_space: true,
    };
    r.element(text, true);
    let preserve = xml_space_preserve(text);
    if !preserve {
        for c in &mut r.chunks {
            trim_end(&mut c.spans);
        }
    }
    let fill = match ctx.paint(text, "fill", Some(Color::BLACK)) {
        Some(c) => Fill::Solid(c),
        None => Fill::None,
    };
    for c in &mut r.chunks {
        c.spans.retain(|s| !s.text.is_empty());
        for s in &mut c.spans {
            if s.fill.as_ref() == Some(&fill) {
                s.fill = None;
            }
        }
    }
    let stroke = ctx.paint(text, "stroke", None).map(|c| {
        let fs = ctx.font_size(text);
        let w = ctx
            .inherited(text, "stroke-width")
            .and_then(|v| ctx.length(&v, fs, ctx.viewport.0))
            .unwrap_or(1.0);
        (c, w)
    });
    SvgText {
        name: text.attribute("id").map(str::to_string),
        chunks: r.chunks,
        fill,
        stroke,
    }
}

fn xml_space_preserve(n: Node) -> bool {
    n.ancestors()
        .find_map(|a| a.attribute((XML_NS, "space")))
        .is_some_and(|v| v == "preserve")
}

fn trim_end(spans: &mut [TextSpan]) {
    for s in spans.iter_mut().rev() {
        let t = s.text.trim_end_matches(' ').len();
        s.text.truncate(t);
        if !s.text.is_empty() {
            break;
        }
    }
}

impl Reader<'_, '_, '_> {
    fn element(&mut self, el: Node, is_text: bool) {
        let ctx = self.ctx;
        if matches!(ctx.own(el, "display").as_deref(), Some("none")) {
            return;
        }
        let fs = ctx.font_size(el);
        let (vw, vh) = ctx.viewport;
        let x = ctx.first_coord(el, "x", fs, vw);
        let y = ctx.first_coord(el, "y", fs, vh);
        let dx = ctx.first_coord(el, "dx", fs, vw).unwrap_or(0.0);
        let dy = ctx.first_coord(el, "dy", fs, vh).unwrap_or(0.0);
        if is_text || x.is_some() {
            // A new chunk: `y` continues the current line when not given.
            let prev_y = self.chunks.last().map_or(0.0, |c| c.y - c.baseline);
            let start_y = y.unwrap_or(prev_y + self.dy) + dy;
            let align = match ctx.inherited(el, "text-anchor").as_deref() {
                Some("middle") => TextAlign::Center,
                Some("end") => TextAlign::Right,
                _ => TextAlign::Left,
            };
            let base = ctx
                .inherited(el, "dominant-baseline")
                .map_or(0.0, |v| baseline_offset(&v))
                + ctx
                    .own(el, "alignment-baseline")
                    .map_or(0.0, |v| baseline_offset(&v));
            if !xml_space_preserve(el) {
                if let Some(c) = self.chunks.last_mut() {
                    trim_end(&mut c.spans);
                }
            }
            self.chunks.push(Chunk {
                x: x.unwrap_or(0.0) + dx,
                y: start_y + base * fs,
                align,
                baseline: base * fs,
                spans: Vec::new(),
            });
            self.dx = 0.0;
            self.dy = 0.0;
            self.after_space = true;
        } else {
            if let Some(y) = y {
                let c = self.chunks.last().map_or(0.0, |c| c.y - c.baseline);
                self.dy = y - c;
            }
            self.dx += dx;
            self.dy += dy;
        }
        let span = self.style(el, fs);
        let preserve = xml_space_preserve(el);
        for child in el.children() {
            if child.is_text() {
                if let Some(t) = child.text() {
                    self.push(t, &span, preserve);
                }
            } else if ["tspan", "a", "textPath"]
                .iter()
                .any(|name| is_svg(child, name))
            {
                self.element(child, false);
            }
        }
    }

    /// The span style of an element's own characters.
    fn style(&self, el: Node, fs: f64) -> TextSpan {
        let ctx = self.ctx;
        let family = ctx
            .inherited(el, "font-family")
            .and_then(|v| svgtypes::parse_font_families(&v).ok())
            .and_then(|f| f.into_iter().next())
            .map(|f| match f {
                svgtypes::FontFamily::Named(n) => n,
                // Generic families: fonts every system has, or a
                // substitute drawn to the same widths.
                svgtypes::FontFamily::Serif => "Times New Roman".into(),
                svgtypes::FontFamily::SansSerif => "Arial".into(),
                svgtypes::FontFamily::Monospace => "Courier New".into(),
                svgtypes::FontFamily::Cursive => "Comic Sans MS".into(),
                svgtypes::FontFamily::Fantasy => "Impact".into(),
            })
            .unwrap_or_else(|| "Arial".into());
        let mut s = TextSpan::new("", family, fs * PX_TO_PT);
        s.bold = match ctx.inherited(el, "font-weight").as_deref() {
            Some("bold" | "bolder") => true,
            Some(w) => w.parse::<f64>().is_ok_and(|w| w >= 600.0),
            None => false,
        };
        s.italic = matches!(
            ctx.inherited(el, "font-style").as_deref(),
            Some("italic" | "oblique")
        );
        let deco = ctx.inherited(el, "text-decoration").unwrap_or_default();
        s.underline = deco.contains("underline");
        s.strikethrough = deco.contains("line-through");
        if let Some(ls) = ctx
            .inherited(el, "letter-spacing")
            .and_then(|v| ctx.length(&v, fs, fs))
        {
            s.tracking_pct = ls / fs * 100.0;
        }
        let shift = match ctx.own(el, "baseline-shift").as_deref() {
            Some("super") => fs / 3.0,
            Some("sub") => -fs / 5.0,
            Some(v) => ctx.length(v, fs, fs).unwrap_or(0.0),
            None => 0.0,
        };
        s.baseline_shift_pt = (shift - self.dy) * PX_TO_PT;
        s.offset_x_pct = self.dx / fs * 100.0;
        if let Some(r) = el
            .attribute("rotate")
            .and_then(|v| svgtypes::NumberListParser::from(v).next())
            .and_then(|r| r.ok())
        {
            s.angle_deg = -r;
        }
        s.fill = Some(match ctx.paint(el, "fill", Some(Color::BLACK)) {
            Some(c) => Fill::Solid(c),
            None => Fill::None,
        });
        s
    }

    /// Add the characters of a text node in `style`, collapsing white
    /// space unless `xml:space="preserve"` (SVG 1.1 section 10.15).
    fn push(&mut self, t: &str, style: &TextSpan, preserve: bool) {
        let Some(chunk) = self.chunks.last_mut() else {
            return;
        };
        let mut add = String::new();
        for ch in t.chars() {
            let ch = match ch {
                '\n' | '\r' if !preserve => continue,
                '\n' | '\r' | '\t' => ' ',
                c => c,
            };
            if ch == ' ' && !preserve && self.after_space {
                continue;
            }
            self.after_space = ch == ' ';
            add.push(ch);
        }
        if add.is_empty() {
            return;
        }
        let same = chunk.spans.last().is_some_and(|last| {
            let mut a = last.clone();
            a.text.clear();
            a == *style
        });
        match chunk.spans.last_mut() {
            Some(last) if same => last.text.push_str(&add),
            _ => {
                let mut s = style.clone();
                s.text = add;
                chunk.spans.push(s);
            }
        }
    }
}

/// The text objects for `text`, placed with `to_page` (the marker's
/// absolute transform followed by the page mapping).
pub(crate) fn to_shape(text: &SvgText, to_page: Affine, ids: &mut IdSource) -> Option<Shape> {
    let mut chunks: Vec<&Chunk> = text.chunks.iter().collect();
    while chunks.first().is_some_and(|c| c.is_empty()) {
        chunks.remove(0);
    }
    while chunks.last().is_some_and(|c| c.is_empty()) {
        chunks.pop();
    }
    let first = *chunks.first()?;
    let mut shapes = Vec::new();
    if let Some((spans, leading)) = as_lines(&chunks) {
        shapes.push(text_shape(text, first, spans, Some(leading), to_page, ids));
    } else {
        for c in chunks.iter().filter(|c| !c.is_empty()) {
            shapes.push(text_shape(text, c, c.spans.clone(), None, to_page, ids));
        }
    }
    let mut shape = if shapes.len() == 1 {
        shapes.pop()?
    } else {
        Shape::new(
            ShapeId(ids.shape().0),
            ShapeKind::Group { children: shapes },
        )
    };
    shape.name = text.name.clone();
    Some(shape)
}

/// Chunks that are the lines of one paragraph (the same start and
/// alignment, each line the same distance below the one before, or a
/// whole number of times it where empty lines were left out) as the
/// spans of one text with line breaks, and the line distance (px).
fn as_lines(chunks: &[&Chunk]) -> Option<(Vec<TextSpan>, f64)> {
    let first = chunks.first()?;
    if chunks.len() < 2 {
        return None;
    }
    let size = first.spans.first().map_or(16.0, |s| s.size_pt / PX_TO_PT);
    let tol = size * 0.01 + 1e-6;
    let steps: Vec<f64> = chunks.windows(2).map(|w| w[1].y - w[0].y).collect();
    let step = steps.iter().copied().fold(f64::INFINITY, f64::min);
    if step.is_nan() || step <= tol {
        return None;
    }
    let mut spans: Vec<TextSpan> = first.spans.clone();
    for (c, d) in chunks[1..].iter().zip(&steps) {
        let lines = (d / step).round();
        if c.align != first.align
            || (c.x - first.x).abs() > tol
            || (d - lines * step).abs() > tol * lines
            || lines > 100.0
        {
            return None;
        }
        // The first chunk has characters, so there is a last span.
        if let Some(last) = spans.last_mut() {
            last.text.push_str(&"\n".repeat(lines as usize));
        }
        spans.extend(c.spans.iter().cloned());
    }
    Some((spans, step))
}

fn text_shape(
    text: &SvgText,
    chunk: &Chunk,
    mut spans: Vec<TextSpan>,
    leading_px: Option<f64>,
    to_page: Affine,
    ids: &mut IdSource,
) -> Shape {
    // Text space: mm, Y up, the origin on the chunk's baseline start.
    let mut t = to_page
        * Affine::translate((chunk.x, chunk.y))
        * Affine::scale_non_uniform(1.0 / PX_TO_MM, -1.0 / PX_TO_MM);
    // A uniform scale moves into the font size, so the text reads at
    // the size it is drawn; a skew or a mirror stays in the transform.
    let [a, b, c, d, e, f] = t.as_coeffs();
    let det = a * d - b * c;
    let k = det.abs().sqrt();
    let mut factor = 1.0;
    if det > 0.0 && k.is_finite() && (a - d).abs() <= k * 1e-6 && (b + c).abs() <= k * 1e-6 {
        t = Affine::new([a / k, b / k, c / k, d / k, e, f]);
        factor = k;
    }
    for s in &mut spans {
        s.size_pt *= factor;
        s.baseline_shift_pt *= factor;
    }
    let mut para = ParagraphStyle::default();
    if let Some(l) = leading_px {
        para.leading_unit = LeadingUnit::Points;
        para.leading_pct = l * PX_TO_PT * factor;
    }
    let mut shape = Shape::new(
        ShapeId(ids.shape().0),
        ShapeKind::Text {
            spans,
            origin: Point::ZERO,
            frame: None,
            align: chunk.align,
            para,
            on_path: None,
        },
    );
    shape.transform = t;
    shape.fill = text.fill.clone();
    shape.stroke = text
        .stroke
        .map(|(color, w)| Stroke::new(color, (w * PX_TO_MM * factor).max(Stroke::HAIRLINE)));
    shape
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svg_import::parse;

    const MM: f64 = PX_TO_MM;

    fn import(svg: &str) -> Vec<Shape> {
        parse(svg, &mut IdSource::default()).expect("parse").shapes
    }

    /// Every text object, depth first.
    fn texts(shapes: &[Shape]) -> Vec<&Shape> {
        let mut out = Vec::new();
        for s in shapes {
            match &s.kind {
                ShapeKind::Text { .. } => out.push(s),
                ShapeKind::Group { children } => out.extend(texts(children)),
                ShapeKind::ClipFrame { contents, .. } => out.extend(texts(contents)),
                _ => {}
            }
        }
        out
    }

    fn spans(s: &Shape) -> &[TextSpan] {
        match &s.kind {
            ShapeKind::Text { spans, .. } => spans,
            _ => panic!("not text"),
        }
    }

    fn content(s: &Shape) -> String {
        spans(s).iter().map(|s| s.text.as_str()).collect()
    }

    fn align(s: &Shape) -> TextAlign {
        match &s.kind {
            ShapeKind::Text { align, .. } => *align,
            _ => panic!("not text"),
        }
    }

    fn origin(s: &Shape) -> Point {
        s.transform * Point::ZERO
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn the_reported_file_keeps_its_text() {
        // The sample attached to #1820, line ends and all.
        let svg = "<svg viewBox=\"0 0 300 100\" xmlns=\"http://www.w3.org/2000/svg\">\r\n  <!-- Background Rectangle -->\r\n  <rect x=\"20\" y=\"20\" width=\"260\" height=\"60\" rx=\"10\" ry=\"10\" fill=\"#FFDE4D\" />\r\n\r\n  <!-- Text Object Layered on Top -->\r\n  <text x=\"150\" y=\"50\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"sans-serif\" font-size=\"20\" font-weight=\"bold\" fill=\"#1A1A1A\">\r\n    Hello SVG\r\n  </text>\r\n</svg>\r\n";
        let shapes = import(svg);
        assert_eq!(shapes.len(), 2, "{shapes:#?}");
        assert!(matches!(shapes[0].kind, ShapeKind::Path { .. }));
        let t = &shapes[1];
        assert_eq!(content(t), "Hello SVG");
        let s = &spans(t)[0];
        assert_eq!(s.font_family, "Arial");
        assert!(close(s.size_pt, 15.0), "{}", s.size_pt);
        assert!(s.bold && !s.italic);
        assert_eq!(s.fill, None);
        assert_eq!(t.fill, Fill::Solid(Color::rgb8(0x1a, 0x1a, 0x1a)));
        assert_eq!(t.stroke, None);
        assert_eq!(align(t), TextAlign::Center);
        // Centred on x = 150; the central baseline sits 0.35 em below y.
        let o = origin(t);
        assert!(close(o.x, 150.0 * MM), "{o:?}");
        assert!(close(o.y, (100.0 - 57.0) * MM), "{o:?}");
        // No rotation, mirror or scale left in the transform.
        let [a, b, c, d, _, _] = t.transform.as_coeffs();
        assert!(close(a, 1.0) && close(b, 0.0) && close(c, 0.0) && close(d, 1.0));
    }

    #[test]
    fn tspans_give_spans_with_their_own_style() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <text x="10" y="40" font-family="Georgia, serif" font-size="12pt">Plain <tspan font-weight="700" fill="red">bold red</tspan> <tspan style="font-style:italic;text-decoration:underline">and italic</tspan></text>
        </svg>"##;
        let shapes = import(svg);
        let t = texts(&shapes);
        assert_eq!(t.len(), 1);
        let sp = spans(t[0]);
        let parts: Vec<&str> = sp.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(parts, ["Plain ", "bold red", " ", "and italic"]);
        assert!(sp
            .iter()
            .all(|s| s.font_family == "Georgia" && close(s.size_pt, 12.0)));
        assert!(sp[1].bold && !sp[0].bold);
        assert_eq!(sp[1].fill, Some(Fill::Solid(Color::rgb8(255, 0, 0))));
        assert_eq!(sp[0].fill, None);
        assert!(sp[3].italic && sp[3].underline);
        assert_eq!(align(t[0]), TextAlign::Left);
    }

    #[test]
    fn text_in_transformed_groups_lands_where_it_is_drawn() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <g transform="translate(20 10)"><g transform="scale(2)">
                <text x="5" y="20" font-size="10">Big</text>
            </g></g>
            <text x="50" y="50" font-size="10" transform="rotate(-90 50 50)">Up</text>
        </svg>"##;
        let shapes = import(svg);
        let t = texts(&shapes);
        assert_eq!(t.len(), 2);
        // The scale moves into the font size: 10 px * 2 = 15 pt.
        assert!(close(spans(t[0])[0].size_pt, 15.0));
        let o = origin(t[0]);
        assert!(
            close(o.x, 30.0 * MM) && close(o.y, (100.0 - 50.0) * MM),
            "{o:?}"
        );
        // Rotated a quarter turn counter-clockwise on the page.
        let [a, b, c, d, _, _] = t[1].transform.as_coeffs();
        assert!(close(a, 0.0) && close(b, 1.0) && close(c, -1.0) && close(d, 0.0));
        assert!(close(spans(t[1])[0].size_pt, 7.5));
    }

    #[test]
    fn style_sheets_style_attributes_and_attributes_cascade() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <style type="text/css"><![CDATA[
                text { font-family: 'Courier New'; fill: #00f }
                .title { font-size: 24px; font-weight: bold }
                #main { fill: #0f0 }
                .small { font: italic 8px "Times New Roman", serif }
            ]]></style>
            <g text-anchor="end">
                <text id="main" class="title" x="190" y="30" fill="#f00" font-size="5">A</text>
                <text class="title" x="190" y="60" style="fill:#123456;font-size:2em">B</text>
                <text class="small" x="10" y="90">C</text>
            </g>
        </svg>"##;
        let shapes = import(svg);
        let t = texts(&shapes);
        assert_eq!(t.len(), 3);
        // The rule beats the attribute; the id rule beats the type rule.
        assert_eq!(t[0].fill, Fill::Solid(Color::rgb8(0, 255, 0)));
        assert_eq!(t[0].name.as_deref(), Some("main"));
        let a = &spans(t[0])[0];
        assert_eq!(a.font_family, "Courier New");
        assert!(close(a.size_pt, 18.0) && a.bold);
        assert_eq!(align(t[0]), TextAlign::Right);
        // The style attribute beats every rule; em is of the parent's 16 px.
        assert_eq!(t[1].fill, Fill::Solid(Color::rgb8(0x12, 0x34, 0x56)));
        assert!(close(spans(t[1])[0].size_pt, 24.0));
        // The font shorthand.
        let c = &spans(t[2])[0];
        assert_eq!(c.font_family, "Times New Roman");
        assert!(c.italic && !c.bold && close(c.size_pt, 6.0));
        assert_eq!(t[2].fill, Fill::Solid(Color::rgb8(0, 0, 255)));
    }

    #[test]
    fn exported_lines_become_one_text_with_line_breaks() {
        // The way editors write a paragraph: namespaced attributes, kept
        // white space, a tspan per line with its own x and y, an empty
        // line left out.
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg"
                xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
                xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
                width="200mm" height="100mm" viewBox="0 0 200 100">
            <g inkscape:label="Layer 1" inkscape:groupmode="layer">
            <text xml:space="preserve" style="font-size:10px;line-height:1.25;font-family:'DejaVu Sans';fill:#000000;stroke:none" x="20" y="30" id="text1"><tspan sodipodi:role="line" id="tspan1" x="20" y="30">First  line</tspan><tspan sodipodi:role="line" x="20" y="42.5" id="tspan2" style="font-weight:bold">Second</tspan><tspan sodipodi:role="line" x="20" y="67.5" id="tspan3">Fourth</tspan></text>
            </g>
        </svg>"##;
        let shapes = import(svg);
        let t = texts(&shapes);
        assert_eq!(t.len(), 1, "{shapes:#?}");
        assert_eq!(content(t[0]), "First  line\nSecond\n\nFourth");
        assert!(spans(t[0]).iter().any(|s| s.bold && s.text == "Second\n\n"));
        // 10 user units are 10 mm here: 28.35 pt; lines 12.5 mm apart.
        let s = &spans(t[0])[0];
        assert!(
            (s.size_pt - 10.0 / 25.4 * 72.0).abs() < 1e-6,
            "{}",
            s.size_pt
        );
        match &t[0].kind {
            ShapeKind::Text { para, .. } => {
                assert_eq!(para.leading_unit, LeadingUnit::Points);
                assert!((para.leading_pct - 12.5 / 25.4 * 72.0).abs() < 1e-6);
            }
            _ => unreachable!(),
        }
        let o = origin(t[0]);
        assert!(close(o.x, 20.0) && close(o.y, 70.0), "{o:?}");
        assert_eq!(t[0].stroke, None);
    }

    #[test]
    fn lines_that_do_not_line_up_stay_separate_texts() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <text id="t"><tspan x="10" y="20">Left</tspan><tspan x="100" y="20">Right</tspan></text>
        </svg>"##;
        let shapes = import(svg);
        assert_eq!(shapes.len(), 1);
        assert!(matches!(shapes[0].kind, ShapeKind::Group { .. }));
        assert_eq!(shapes[0].name.as_deref(), Some("t"));
        let t = texts(&shapes);
        assert_eq!(
            t.iter().map(|s| content(s)).collect::<Vec<_>>(),
            ["Left", "Right"]
        );
        assert!(close(origin(t[1]).x, 100.0 * MM));
    }

    #[test]
    fn dx_and_dy_shift_the_characters_that_follow() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <text x="10" y="50" font-size="20">H<tspan dy="-10">2</tspan><tspan dy="10" dx="4">O</tspan><tspan x="10" dy="1.5em">next</tspan></text>
        </svg>"##;
        let shapes = import(svg);
        let t = texts(&shapes);
        assert_eq!(content(t[0]), "H2O\nnext");
        let sp = spans(t[0]);
        // 10 px up = 7.5 pt; back down, then 4 px right (20 % of 20 px).
        assert!(close(sp[1].baseline_shift_pt, 7.5), "{sp:#?}");
        assert!(close(sp[2].baseline_shift_pt, 0.0) && close(sp[2].offset_x_pct, 20.0));
        match &t[0].kind {
            ShapeKind::Text { para, .. } => assert!(close(para.leading_pct, 22.5)),
            _ => unreachable!(),
        }
    }

    #[test]
    fn white_space_collapses_unless_preserved() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <text x="0" y="10">
                a	 b
                <tspan>  c </tspan>
            </text>
            <text x="0" y="50" xml:space="preserve"> a  b </text>
        </svg>"##;
        let shapes = import(svg);
        let t = texts(&shapes);
        assert_eq!(content(t[0]), "a b c");
        assert_eq!(content(t[1]), " a  b ");
    }

    #[test]
    fn hidden_empty_and_clip_text_is_left_out_and_order_is_kept() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
            <defs><clipPath id="c"><text x="0" y="50" font-size="40">clip</text></clipPath></defs>
            <rect width="10" height="10" fill="red"/>
            <text x="0" y="20">middle</text>
            <text x="0" y="30" display="none">hidden</text>
            <text x="0" y="40" style="visibility:hidden">hidden too</text>
            <text x="0" y="45">   </text>
            <rect y="20" width="10" height="10" fill="blue"/>
            <g clip-path="url(#c)"><rect width="200" height="100"/></g>
        </svg>"##;
        let shapes = import(svg);
        let kinds: Vec<&str> = shapes
            .iter()
            .map(|s| match s.kind {
                ShapeKind::Text { .. } => "text",
                ShapeKind::Path { .. } => "path",
                _ => "other",
            })
            .collect();
        // The clip whose only child is text reads nothing, as before.
        assert_eq!(kinds, ["path", "text", "path"], "{shapes:#?}");
        assert_eq!(content(&shapes[1]), "middle");
    }

    #[test]
    fn text_reused_through_use_appears_at_each_place() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="200" height="100">
            <defs><g id="label"><text x="0" y="10" fill="url(#g)">Tag</text></g>
            <linearGradient id="g"><stop offset="0" stop-color="#336699"/></linearGradient></defs>
            <use xlink:href="#label" x="10" y="10"/>
            <use href="#label" x="100" y="50"/>
        </svg>"##;
        let shapes = import(svg);
        let t = texts(&shapes);
        assert_eq!(t.len(), 2, "{shapes:#?}");
        assert!(close(origin(t[0]).x, 10.0 * MM) && close(origin(t[1]).x, 100.0 * MM));
        assert!(close(origin(t[1]).y, (100.0 - 60.0) * MM));
        assert_eq!(t[0].fill, Fill::Solid(Color::rgb8(0x33, 0x66, 0x99)));
    }

    #[test]
    fn outlined_text_keeps_its_outline() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="96">
            <text x="0" y="50" fill="none" stroke="#ff0000" stroke-width="2" font-size="30">O</text>
        </svg>"##;
        let t = import(svg);
        assert_eq!(t[0].fill, Fill::None);
        let s = t[0].stroke.as_ref().expect("outline");
        assert_eq!(s.color, Color::rgb8(255, 0, 0));
        assert!(close(s.width, 2.0 * MM));
    }

    #[test]
    fn broken_text_markup_still_opens() {
        for svg in [
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text x="1e400" y="nan" font-size="-4" dx="x">a</text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text style="font:;fill:url(#none)">a<tspan x="">b</tspan></text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text><text>nested</text></text></svg>"#,
            r#"<svg xmlns="http://www.w3.org/2000/svg"><text>a & b</text></svg>"#,
        ] {
            let _ = parse(svg, &mut IdSource::default());
        }
    }
}
