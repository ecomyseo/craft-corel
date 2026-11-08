//! Converting a drawing's colours between CMYK and RGB (#972): every
//! colour of the objects chosen (fills, gradient stops, mesh nodes,
//! outlines, text, table cells, effects' colours, shadows) or of the
//! whole drawing (with the pages' backgrounds and the symbols) goes to
//! the colour model asked for, through `Color::convert_to`, so the colour
//! engine's profiles (the drawing's colour management) do the conversion.
//! Registration colour never changes; spot colours stay unless asked, and
//! then become process colours of the model. A bitmap takes the colour
//! mode asked for (`Shape::cmyk_bitmap`, #1254); its pixels, which stay
//! RGB for the screen, are taken through the CMYK profile and back by
//! the application when the mode becomes CMYK, so the picture shows as
//! it prints. See docs/behavior/colour-management.md.

use crate::document::ShapeKind;
use crate::{Color, Document, Shape, ShapeId};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// The colour model colours are converted to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorTarget {
    Cmyk,
    Rgb,
}

impl ColorTarget {
    /// The model's name for `Color::convert_to`.
    pub fn model(self) -> &'static str {
        match self {
            ColorTarget::Cmyk => "CMYK",
            ColorTarget::Rgb => "RGB",
        }
    }
}

/// `c` in model `to`: registration colour as it is, a spot colour as it
/// is unless `spots`, a colour already in the model as it is.
pub fn converted(c: Color, to: ColorTarget, spots: bool) -> Color {
    match c {
        Color::Registration => c,
        Color::Spot { .. } if !spots => c,
        _ => c.convert_to(to.model()),
    }
}

/// Change every colour inside the JSON value `v` (a serialised object)
/// by `f`; the number changed.
fn walk(v: &mut serde_json::Value, f: &dyn Fn(Color) -> Color) -> usize {
    match v {
        serde_json::Value::Object(map) => {
            // A colour is serialised as an object tagged with its model;
            // inside a tagged enum (a solid fill) its fields share the
            // object with the enum's tag, which stays.
            if map.contains_key("model") {
                if let Ok(c) =
                    serde_json::from_value::<Color>(serde_json::Value::Object(map.clone()))
                {
                    let to = f(c);
                    if to == c {
                        return 0;
                    }
                    let (Ok(serde_json::Value::Object(old)), Ok(serde_json::Value::Object(new))) =
                        (serde_json::to_value(c), serde_json::to_value(to))
                    else {
                        return 0;
                    };
                    for k in old.keys() {
                        map.remove(k);
                    }
                    map.extend(new);
                    return 1;
                }
            }
            map.values_mut().map(|x| walk(x, f)).sum()
        }
        serde_json::Value::Array(a) => a.iter_mut().map(|x| walk(x, f)).sum(),
        _ => 0,
    }
}

/// Every colour of `x` changed by `f`, through its serialised form, so no
/// colour of any part is missed; `x` stays as it was when it does not
/// read back. The number changed.
fn convert_value<T: Serialize + DeserializeOwned>(x: &mut T, f: &dyn Fn(Color) -> Color) -> usize {
    let Ok(mut v) = serde_json::to_value(&*x) else {
        return 0;
    };
    let n = walk(&mut v, f);
    if n == 0 {
        return 0;
    }
    match serde_json::from_value(v) {
        Ok(new) => {
            *x = new;
            n
        }
        Err(_) => 0,
    }
}

/// Every colour of `s` and of the objects inside it changed by `f`, and a
/// bitmap's colour mode set to `to`; the number changed (a bitmap whose
/// mode changes counts one). Groups' and clip frames' contents are
/// converted one by one, and a bitmap's pixels are left out of the
/// serialised form.
pub fn convert_shape(s: &mut Shape, f: &dyn Fn(Color) -> Color, to: ColorTarget) -> usize {
    let heavy = matches!(
        s.kind,
        ShapeKind::Group { .. } | ShapeKind::ClipFrame { .. } | ShapeKind::Bitmap { .. }
    );
    let mut n = 0;
    if matches!(s.kind, ShapeKind::Bitmap { .. }) {
        let cmyk = to == ColorTarget::Cmyk;
        if s.cmyk_bitmap != cmyk {
            s.cmyk_bitmap = cmyk;
            n += 1;
        }
        // An RGB picture prints its pixels through the colour
        // management, not the inks it was read with (#1659).
        if !cmyk {
            s.picture_inks = None;
        }
    }
    let saved = heavy.then(|| {
        std::mem::replace(
            &mut s.kind,
            ShapeKind::Group {
                children: Vec::new(),
            },
        )
    });
    // The ink channels hold no colour to convert; kept out of the
    // serialised form with the pixels.
    let inks = s.picture_inks.take();
    n += convert_value(s, f);
    s.picture_inks = inks;
    if let Some(mut kind) = saved {
        match &mut kind {
            ShapeKind::Group { children } => {
                n += children
                    .iter_mut()
                    .map(|c| convert_shape(c, f, to))
                    .sum::<usize>();
            }
            ShapeKind::ClipFrame { frame, contents } => {
                n += convert_shape(frame, f, to);
                n += contents
                    .iter_mut()
                    .map(|c| convert_shape(c, f, to))
                    .sum::<usize>();
            }
            _ => {}
        }
        s.kind = kind;
    }
    n
}

/// The bitmaps among `shapes` (any depth; the whole drawing when empty)
/// whose colour mode `to` changes: their ids, for the application to
/// take their pixels through the profiles (to CMYK) in the same step.
pub fn bitmaps_changing_mode(doc: &Document, shapes: &[ShapeId], to: ColorTarget) -> Vec<ShapeId> {
    fn walk(s: &Shape, to: ColorTarget, out: &mut Vec<ShapeId>) {
        match &s.kind {
            ShapeKind::Bitmap { .. } => {
                if s.cmyk_bitmap != (to == ColorTarget::Cmyk) {
                    out.push(s.id);
                }
            }
            ShapeKind::Group { children } => children.iter().for_each(|c| walk(c, to, out)),
            ShapeKind::ClipFrame { frame, contents } => {
                walk(frame, to, out);
                contents.iter().for_each(|c| walk(c, to, out));
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    if shapes.is_empty() {
        for l in doc.all_layers() {
            l.shapes.iter().for_each(|s| walk(s, to, &mut out));
        }
        for sym in &doc.symbols {
            walk(&sym.shape, to, &mut out);
        }
    } else {
        for id in shapes {
            if let Some(s) = doc.find_shape(*id) {
                walk(s, to, &mut out);
            }
        }
    }
    out
}

/// Convert the colours of `shapes` (any depth) to `to`, or, with `shapes`
/// empty, of the whole drawing: every object on every layer, the pages'
/// backgrounds and the symbols. The number of colours changed.
pub fn convert_document(
    doc: &mut Document,
    shapes: &[ShapeId],
    to: ColorTarget,
    spots: bool,
) -> usize {
    let f = move |c: Color| converted(c, to, spots);
    let mut n = 0;
    if shapes.is_empty() {
        for l in doc.all_layers_mut() {
            for s in l.shapes.iter_mut() {
                n += convert_shape(s, &f, to);
            }
        }
        for p in doc.pages.iter_mut() {
            if let Some(bg) = &mut p.background {
                n += convert_value(bg, &f);
            }
        }
        for sym in doc.symbols.iter_mut() {
            n += convert_shape(&mut sym.shape, &f, to);
        }
        return n;
    }
    for id in shapes {
        if let Some(s) = doc.find_shape_mut(*id) {
            n += convert_shape(s, &f, to);
        }
    }
    n
}

/// Only the document fields in the colour-preparation transaction. Applying
/// this snapshot does not allocate identifiers, reapply styles, settle default
/// flags, change fonts, or clear spline and transient text-layout attributes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentColorState {
    pub pages: Vec<crate::Page>,
    pub master: Vec<crate::Layer>,
    pub master_guides: Vec<crate::document::Guide>,
    pub object_styles: Vec<crate::ObjectStyle>,
    pub defaults: crate::ObjectDefaults,
    pub color_styles: Vec<crate::ColorStyle>,
    pub symbols: Vec<crate::Symbol>,
    pub metadata: crate::Metadata,
    pub palette: Vec<Color>,
    pub cmyk_profile: Option<crate::DocumentProfile>,
}

impl DocumentColorState {
    pub fn capture(doc: &Document) -> Self {
        Self {
            pages: doc.pages.clone(),
            master: doc.master.clone(),
            master_guides: doc.master_guides.clone(),
            object_styles: doc.object_styles.clone(),
            defaults: doc.defaults.clone(),
            color_styles: doc.color_styles.clone(),
            symbols: doc.symbols.clone(),
            metadata: doc.metadata.clone(),
            palette: doc.palette.clone(),
            cmyk_profile: doc.cmyk_profile.clone(),
        }
    }

    /// The command checks its captured before-state before calling this.
    pub fn apply_to(&self, doc: &mut Document) {
        doc.pages = self.pages.clone();
        doc.master = self.master.clone();
        doc.master_guides = self.master_guides.clone();
        doc.object_styles = self.object_styles.clone();
        doc.defaults = self.defaults.clone();
        doc.color_styles = self.color_styles.clone();
        doc.symbols = self.symbols.clone();
        doc.metadata = self.metadata.clone();
        doc.palette = self.palette.clone();
        doc.cmyk_profile = self.cmyk_profile.clone();
    }
}

type ColorMapper<'a> = dyn FnMut(Color) -> Result<Color, String> + 'a;
type ShapeMapper<'a> = dyn FnMut(&mut Shape) -> Result<(), String> + 'a;
const COLOR_TREE_DEPTH: usize = 128;
const COLOR_TREE_SHAPES: usize = 1_000_000;

fn mapped_colour(colour: &mut Color, f: &mut ColorMapper<'_>) -> Result<usize, String> {
    let changed = f(*colour)?;
    if changed == *colour {
        Ok(0)
    } else {
        *colour = changed;
        Ok(1)
    }
}

fn try_walk_value(value: &mut serde_json::Value, f: &mut ColorMapper<'_>) -> Result<usize, String> {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(model) = map.get("model").and_then(|value| value.as_str()) {
                match serde_json::from_value::<Color>(serde_json::Value::Object(map.clone())) {
                    Ok(colour) => {
                        let changed = f(colour)?;
                        if changed == colour {
                            return Ok(0);
                        }
                        let serde_json::Value::Object(old) =
                            serde_json::to_value(colour).map_err(|error| error.to_string())?
                        else {
                            return Err("invalid colour encoding".into());
                        };
                        let serde_json::Value::Object(new) =
                            serde_json::to_value(changed).map_err(|error| error.to_string())?
                        else {
                            return Err("invalid converted colour encoding".into());
                        };
                        for key in old.keys() {
                            map.remove(key);
                        }
                        map.extend(new);
                        return Ok(1);
                    }
                    Err(error)
                        if matches!(
                            model,
                            "cmyk"
                                | "rgb"
                                | "gray"
                                | "lab"
                                | "hsb"
                                | "hsl"
                                | "spot"
                                | "registration"
                        ) =>
                    {
                        return Err(error.to_string());
                    }
                    Err(_) => {}
                }
            }
            let mut count = 0;
            for value in map.values_mut() {
                count += try_walk_value(value, f)?;
            }
            Ok(count)
        }
        serde_json::Value::Array(values) => {
            let mut count = 0;
            for value in values {
                count += try_walk_value(value, f)?;
            }
            Ok(count)
        }
        _ => Ok(0),
    }
}

/// Fallible mapping of small colour-bearing resources. No update on an error.
/// Shape callers should use the typed mapper, which never serializes pixels.
pub fn try_map_value_colors<T: Serialize + DeserializeOwned>(
    value: &mut T,
    f: &mut ColorMapper<'_>,
) -> Result<usize, String> {
    let mut encoded = serde_json::to_value(&*value).map_err(|error| error.to_string())?;
    let count = try_walk_value(&mut encoded, f)?;
    if count > 0 {
        *value = serde_json::from_value(encoded).map_err(|error| error.to_string())?;
    }
    Ok(count)
}

fn map_fill_local(fill: &mut crate::Fill, f: &mut ColorMapper<'_>) -> Result<usize, String> {
    use crate::{Fill, Pattern};
    let mut count = 0;
    match fill {
        Fill::None | Fill::Pattern(Pattern::Bitmap { .. } | Pattern::Vector { .. }) => {}
        Fill::Solid(colour) => count += mapped_colour(colour, f)?,
        Fill::Fountain(fill) => {
            for stop in &mut fill.stops {
                count += mapped_colour(&mut stop.color, f)?;
            }
        }
        Fill::Mesh(fill) => {
            for node in &mut fill.nodes {
                count += mapped_colour(&mut node.color, f)?;
            }
        }
        Fill::Texture(fill) => {
            count += mapped_colour(&mut fill.color_a, f)?;
            count += mapped_colour(&mut fill.color_b, f)?;
        }
        Fill::Pattern(Pattern::TwoColor { front, back, .. }) => {
            count += mapped_colour(front, f)?;
            count += mapped_colour(back, f)?;
        }
    }
    Ok(count)
}

fn map_optional_fill(
    fill: &mut Option<crate::Fill>,
    f: &mut ColorMapper<'_>,
) -> Result<usize, String> {
    fill.as_mut()
        .map(|fill| map_fill_local(fill, f))
        .transpose()
        .map(|count| count.unwrap_or(0))
}
fn map_stroke(
    stroke: &mut Option<crate::Stroke>,
    f: &mut ColorMapper<'_>,
) -> Result<usize, String> {
    stroke
        .as_mut()
        .map(|stroke| map_stroke_local(stroke, f))
        .transpose()
        .map(|count| count.unwrap_or(0))
}
fn map_stroke_local(stroke: &mut crate::Stroke, f: &mut ColorMapper<'_>) -> Result<usize, String> {
    let mut count = mapped_colour(&mut stroke.color, f)?;
    if let Some(fill) = &mut stroke.fill {
        count += map_fill_local(fill, f)?;
    }
    Ok(count)
}
fn map_cell_borders(
    borders: &mut crate::table::CellBorders,
    f: &mut ColorMapper<'_>,
) -> Result<usize, String> {
    let mut count = 0;
    for side in crate::table::CellSide::ALL {
        if let crate::table::EdgeBorder::Stroke(stroke) = borders.side_mut(side) {
            count += map_stroke_local(stroke, f)?;
        }
    }
    Ok(count)
}
fn map_spans(spans: &mut [crate::TextSpan], f: &mut ColorMapper<'_>) -> Result<usize, String> {
    let mut count = 0;
    for span in spans {
        count += map_optional_fill(&mut span.fill, f)?
            + map_optional_fill(&mut span.background, f)?
            + map_stroke(&mut span.outline, f)?;
    }
    Ok(count)
}
fn map_effect_local(
    effect: &mut crate::live::Effect,
    f: &mut ColorMapper<'_>,
) -> Result<usize, String> {
    use crate::live::Effect;
    let mut count = 0;
    match effect {
        Effect::Contour {
            fill_to,
            outline_to,
            ..
        } => {
            count += mapped_colour(fill_to, f)?;
            if let Some(colour) = outline_to {
                count += mapped_colour(colour, f)?;
            }
        }
        Effect::Extrude {
            shade_from,
            shade_to,
            solid,
            space,
            ..
        } => {
            for colour in [shade_from, shade_to, solid].into_iter().flatten() {
                count += mapped_colour(colour, f)?;
            }
            count += try_map_value_colors(space, f)?;
        }
        Effect::Bevel {
            shadow_color,
            light_color,
            emboss_color,
            ..
        } => {
            count += mapped_colour(shadow_color, f)? + mapped_colour(light_color, f)?;
            if let Some(colour) = emboss_color {
                count += mapped_colour(colour, f)?;
            }
        }
        Effect::BlockShadow { color, .. } => count += mapped_colour(color, f)?,
        Effect::Lens(lens) => count += try_map_value_colors(lens, f)?,
        Effect::Transparency { mask, .. } => count += map_fill_local(mask, f)?,
        Effect::Blend { .. }
        | Effect::Distort(_)
        | Effect::Envelope { .. }
        | Effect::Perspective { .. }
        | Effect::Symmetry { .. } => {}
    }
    Ok(count)
}
fn map_shape_local(shape: &mut Shape, f: &mut ColorMapper<'_>) -> Result<usize, String> {
    let mut count = map_fill_local(&mut shape.fill, f)? + map_stroke(&mut shape.stroke, f)?;
    if let Some(shadow) = &mut shape.shadow {
        count += mapped_colour(&mut shadow.color, f)?;
    }
    for fill in &mut shape.extra_fills {
        count += map_fill_local(fill, f)?;
    }
    for stroke in &mut shape.extra_strokes {
        count += map_stroke_local(stroke, f)?;
    }
    for effect in &mut shape.effects {
        count += map_effect_local(effect, f)?;
    }
    for parked in &mut shape.parked_effects {
        count += map_effect_local(&mut parked.effect, f)?;
    }
    if let Some(qr) = &mut shape.qr {
        count += map_fill_local(&mut qr.dark, f)?
            + map_fill_local(&mut qr.light, f)?
            + map_stroke(&mut qr.outline, f)?;
    }
    match &mut shape.kind {
        ShapeKind::Text { spans, para, .. } => {
            count += map_spans(spans, f)? + map_optional_fill(&mut para.frame_fill, f)?;
        }
        ShapeKind::Table(table) => {
            count += map_fill_local(&mut table.cell_fill, f)? + map_stroke(&mut table.border, f)?;
            for cell in &mut table.cells {
                count += map_optional_fill(&mut cell.fill, f)?
                    + map_spans(&mut cell.text, f)?
                    + map_cell_borders(&mut cell.borders, f)?;
            }
        }
        _ => {}
    }
    Ok(count)
}

fn fill_roots<'a>(fill: &'a crate::Fill, out: &mut Vec<&'a Shape>) {
    if let crate::Fill::Pattern(crate::Pattern::Vector { shapes, .. }) = fill {
        out.extend(shapes);
    }
}
fn stroke_roots<'a>(stroke: &'a crate::Stroke, out: &mut Vec<&'a Shape>) {
    if let Some(fill) = &stroke.fill {
        fill_roots(fill, out);
    }
}
fn span_roots<'a>(spans: &'a [crate::TextSpan], out: &mut Vec<&'a Shape>) {
    for span in spans {
        for fill in [&span.fill, &span.background].into_iter().flatten() {
            fill_roots(fill, out);
        }
        if let Some(outline) = &span.outline {
            stroke_roots(outline, out);
        }
    }
}
fn effect_roots<'a>(effect: &'a crate::live::Effect, out: &mut Vec<&'a Shape>) {
    match effect {
        crate::live::Effect::Blend { end, .. } => out.push(end),
        crate::live::Effect::Transparency { mask, .. } => fill_roots(mask, out),
        _ => {}
    }
}
fn child_roots(shape: &Shape) -> Vec<&Shape> {
    let mut out = Vec::new();
    fill_roots(&shape.fill, &mut out);
    for stroke in shape.stroke.iter().chain(&shape.extra_strokes) {
        stroke_roots(stroke, &mut out);
    }
    for fill in &shape.extra_fills {
        fill_roots(fill, &mut out);
    }
    if let Some(qr) = &shape.qr {
        fill_roots(&qr.dark, &mut out);
        fill_roots(&qr.light, &mut out);
        if let Some(outline) = &qr.outline {
            stroke_roots(outline, &mut out);
        }
    }
    for effect in &shape.effects {
        effect_roots(effect, &mut out);
    }
    for parked in &shape.parked_effects {
        effect_roots(&parked.effect, &mut out);
    }
    match &shape.kind {
        ShapeKind::Group { children } => out.extend(children),
        ShapeKind::ClipFrame { frame, contents } => {
            out.push(frame);
            out.extend(contents);
        }
        ShapeKind::Text { spans, para, .. } => {
            span_roots(spans, &mut out);
            if let Some(fill) = &para.frame_fill {
                fill_roots(fill, &mut out);
            }
        }
        ShapeKind::Table(table) => {
            fill_roots(&table.cell_fill, &mut out);
            if let Some(border) = &table.border {
                stroke_roots(border, &mut out);
            }
            for cell in &table.cells {
                if let Some(fill) = &cell.fill {
                    fill_roots(fill, &mut out);
                }
                span_roots(&cell.text, &mut out);
                for side in crate::table::CellSide::ALL {
                    if let crate::table::EdgeBorder::Stroke(stroke) = cell.borders.side(side) {
                        stroke_roots(stroke, &mut out);
                    }
                }
            }
        }
        _ => {}
    }
    out
}
fn resource_roots(doc: &Document) -> Vec<&Shape> {
    let mut roots = doc
        .all_layers()
        .flat_map(|layer| &layer.shapes)
        .collect::<Vec<_>>();
    roots.extend(doc.symbols.iter().map(|symbol| &symbol.shape));
    for page in &doc.pages {
        if let Some(fill) = &page.background {
            fill_roots(fill, &mut roots);
        }
    }
    for style in &doc.object_styles {
        fill_roots(&style.fill, &mut roots);
        if let Some(stroke) = &style.stroke {
            stroke_roots(stroke, &mut roots);
        }
    }
    for defaults in [
        &doc.defaults.graphic,
        &doc.defaults.artistic_text,
        &doc.defaults.paragraph_text,
    ] {
        fill_roots(&defaults.fill, &mut roots);
        if let Some(stroke) = &defaults.stroke {
            stroke_roots(stroke, &mut roots);
        }
    }
    if let Some(fill) = &doc.defaults.paragraph.frame_fill {
        fill_roots(fill, &mut roots);
    }
    roots
}
fn validate_roots(roots: Vec<&Shape>) -> Result<(), String> {
    let mut pending = roots
        .into_iter()
        .map(|shape| (shape, 0))
        .collect::<Vec<_>>();
    let mut count = 0usize;
    while let Some((shape, depth)) = pending.pop() {
        count += 1;
        if depth > COLOR_TREE_DEPTH || count > COLOR_TREE_SHAPES {
            return Err("colour tree exceeds its traversal budget".into());
        }
        pending.extend(
            child_roots(shape)
                .into_iter()
                .map(|child| (child, depth + 1)),
        );
    }
    Ok(())
}

pub fn validate_document_color_tree(doc: &Document) -> Result<(), String> {
    validate_roots(resource_roots(doc))
}

/// Validate borrowed clipboard/selection roots before cloning or recursively
/// inspecting them. Includes nested patterns and live-effect endpoints.
pub fn validate_shape_color_trees(shapes: &[Shape]) -> Result<(), String> {
    validate_roots(shapes.iter().collect())
}

/// Inspect one borrowed shape tree with its physical depth. Symbol references
/// are visited as instances; definitions are not expanded by this traversal.
/// Callers can bound an expanded symbol DAG before cloning or rendering it.
pub fn try_visit_shape_color_tree(
    root: &Shape,
    f: &mut dyn FnMut(&Shape, usize) -> Result<(), String>,
) -> Result<usize, String> {
    let mut pending = vec![(root, 0usize)];
    let mut count = 0usize;
    while let Some((shape, depth)) = pending.pop() {
        count += 1;
        if depth > COLOR_TREE_DEPTH || count > COLOR_TREE_SHAPES {
            return Err("colour tree exceeds its traversal budget".into());
        }
        f(shape, depth)?;
        pending.extend(
            child_roots(shape)
                .into_iter()
                .map(|child| (child, depth + 1)),
        );
    }
    Ok(count)
}

/// Read-only inspection of every shape-bearing document resource.
pub fn try_visit_document_shape_tree(
    doc: &Document,
    f: &mut dyn FnMut(&Shape) -> Result<(), String>,
) -> Result<usize, String> {
    let mut pending = resource_roots(doc)
        .into_iter()
        .map(|shape| (shape, 0))
        .collect::<Vec<_>>();
    let mut count = 0usize;
    while let Some((shape, depth)) = pending.pop() {
        count += 1;
        if depth > COLOR_TREE_DEPTH || count > COLOR_TREE_SHAPES {
            return Err("colour tree exceeds its traversal budget".into());
        }
        f(shape)?;
        pending.extend(
            child_roots(shape)
                .into_iter()
                .map(|child| (child, depth + 1)),
        );
    }
    Ok(count)
}
fn visit_fill(
    fill: &mut crate::Fill,
    f: &mut ShapeMapper<'_>,
    depth: usize,
) -> Result<usize, String> {
    let mut count = 0;
    if let crate::Fill::Pattern(crate::Pattern::Vector { shapes, .. }) = fill {
        for shape in shapes {
            count += visit_shape(shape, f, depth)?;
        }
    }
    Ok(count)
}
fn visit_spans(
    spans: &mut [crate::TextSpan],
    f: &mut ShapeMapper<'_>,
    depth: usize,
) -> Result<usize, String> {
    let mut count = 0;
    for span in spans {
        for fill in [&mut span.fill, &mut span.background].into_iter().flatten() {
            count += visit_fill(fill, f, depth)?;
        }
        if let Some(stroke) = &mut span.outline {
            count += visit_stroke(stroke, f, depth)?;
        }
    }
    Ok(count)
}
fn visit_stroke(
    stroke: &mut crate::Stroke,
    f: &mut ShapeMapper<'_>,
    depth: usize,
) -> Result<usize, String> {
    if let Some(fill) = &mut stroke.fill {
        visit_fill(fill, f, depth)
    } else {
        Ok(0)
    }
}
fn visit_effect(
    effect: &mut crate::live::Effect,
    f: &mut ShapeMapper<'_>,
    depth: usize,
) -> Result<usize, String> {
    match effect {
        crate::live::Effect::Blend { end, .. } => visit_shape(end, f, depth),
        crate::live::Effect::Transparency { mask, .. } => visit_fill(mask, f, depth),
        _ => Ok(0),
    }
}
fn visit_shape(shape: &mut Shape, f: &mut ShapeMapper<'_>, depth: usize) -> Result<usize, String> {
    if depth > COLOR_TREE_DEPTH {
        return Err("colour tree exceeds its traversal depth".into());
    }
    f(shape)?;
    let mut count = 1;
    count += visit_fill(&mut shape.fill, f, depth + 1)?;
    for stroke in shape.stroke.iter_mut().chain(&mut shape.extra_strokes) {
        count += visit_stroke(stroke, f, depth + 1)?;
    }
    for fill in &mut shape.extra_fills {
        count += visit_fill(fill, f, depth + 1)?;
    }
    if let Some(qr) = &mut shape.qr {
        count += visit_fill(&mut qr.dark, f, depth + 1)? + visit_fill(&mut qr.light, f, depth + 1)?;
        if let Some(stroke) = &mut qr.outline {
            count += visit_stroke(stroke, f, depth + 1)?;
        }
    }
    for effect in &mut shape.effects {
        count += visit_effect(effect, f, depth + 1)?;
    }
    for parked in &mut shape.parked_effects {
        count += visit_effect(&mut parked.effect, f, depth + 1)?;
    }
    match &mut shape.kind {
        ShapeKind::Group { children } => {
            for child in children {
                count += visit_shape(child, f, depth + 1)?;
            }
        }
        ShapeKind::ClipFrame { frame, contents } => {
            count += visit_shape(frame, f, depth + 1)?;
            for child in contents {
                count += visit_shape(child, f, depth + 1)?;
            }
        }
        ShapeKind::Text { spans, para, .. } => {
            count += visit_spans(spans, f, depth + 1)?;
            if let Some(fill) = &mut para.frame_fill {
                count += visit_fill(fill, f, depth + 1)?;
            }
        }
        ShapeKind::Table(table) => {
            count += visit_fill(&mut table.cell_fill, f, depth + 1)?;
            if let Some(stroke) = &mut table.border {
                count += visit_stroke(stroke, f, depth + 1)?;
            }
            for cell in &mut table.cells {
                if let Some(fill) = &mut cell.fill {
                    count += visit_fill(fill, f, depth + 1)?;
                }
                count += visit_spans(&mut cell.text, f, depth + 1)?;
                for side in crate::table::CellSide::ALL {
                    if let crate::table::EdgeBorder::Stroke(stroke) = cell.borders.side_mut(side) {
                        count += visit_stroke(stroke, f, depth + 1)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(count)
}
fn visit_document(doc: &mut Document, f: &mut ShapeMapper<'_>) -> Result<usize, String> {
    let mut count = 0;
    for layer in doc.all_layers_mut() {
        for shape in &mut layer.shapes {
            count += visit_shape(shape, f, 0)?;
        }
    }
    for symbol in &mut doc.symbols {
        count += visit_shape(&mut symbol.shape, f, 0)?;
    }
    for page in &mut doc.pages {
        if let Some(fill) = &mut page.background {
            count += visit_fill(fill, f, 0)?;
        }
    }
    for style in &mut doc.object_styles {
        count += visit_fill(&mut style.fill, f, 0)?;
        if let Some(stroke) = &mut style.stroke {
            count += visit_stroke(stroke, f, 0)?;
        }
    }
    for defaults in [
        &mut doc.defaults.graphic,
        &mut doc.defaults.artistic_text,
        &mut doc.defaults.paragraph_text,
    ] {
        count += visit_fill(&mut defaults.fill, f, 0)?;
        if let Some(stroke) = &mut defaults.stroke {
            count += visit_stroke(stroke, f, 0)?;
        }
    }
    if let Some(fill) = &mut doc.defaults.paragraph.frame_fill {
        count += visit_fill(fill, f, 0)?;
    }
    Ok(count)
}

/// Visit every typed shape container, including vector-pattern fills and live
/// blend endpoints. Failure leaves the original root untouched; pixels, retained
/// planes, geometry and font buffers are never serialized by this traversal.
pub fn try_map_shape_tree(shape: &mut Shape, f: &mut ShapeMapper<'_>) -> Result<usize, String> {
    validate_roots(vec![shape])?;
    let mut prepared = shape.clone();
    let count = visit_shape(&mut prepared, f, 0)?;
    *shape = prepared;
    Ok(count)
}
pub fn try_map_document_shape_tree(
    doc: &mut Document,
    f: &mut ShapeMapper<'_>,
) -> Result<usize, String> {
    validate_roots(resource_roots(doc))?;
    let mut prepared = doc.clone();
    let count = visit_document(&mut prepared, f)?;
    *doc = prepared;
    Ok(count)
}
pub fn try_map_shape_colors(shape: &mut Shape, f: &mut ColorMapper<'_>) -> Result<usize, String> {
    let mut count = 0;
    try_map_shape_tree(shape, &mut |shape| {
        count += map_shape_local(shape, f)?;
        Ok(())
    })?;
    Ok(count)
}

/// All process-colour locations without changing the colour mode of a bitmap.
/// This maps typed shape colours in place on a temporary document, preserving
/// serde-skipped text state and avoiding pixel/ink/font serialization entirely.
pub fn try_map_document_colors(
    doc: &mut Document,
    f: &mut ColorMapper<'_>,
) -> Result<usize, String> {
    validate_roots(resource_roots(doc))?;
    let mut copy = doc.clone();
    let mut count = 0;
    visit_document(&mut copy, &mut |shape| {
        count += map_shape_local(shape, f)?;
        Ok(())
    })?;
    for page in &mut copy.pages {
        if let Some(fill) = &mut page.background {
            count += map_fill_local(fill, f)?;
        }
        for guide in &mut page.guides {
            if let Some(colour) = &mut guide.color {
                count += mapped_colour(colour, f)?;
            }
        }
    }
    for guide in &mut copy.master_guides {
        if let Some(colour) = &mut guide.color {
            count += mapped_colour(colour, f)?;
        }
    }
    for style in &mut copy.object_styles {
        count += map_fill_local(&mut style.fill, f)? + map_stroke(&mut style.stroke, f)?;
    }
    for defaults in [
        &mut copy.defaults.graphic,
        &mut copy.defaults.artistic_text,
        &mut copy.defaults.paragraph_text,
    ] {
        count += map_fill_local(&mut defaults.fill, f)? + map_stroke(&mut defaults.stroke, f)?;
    }
    count += map_optional_fill(&mut copy.defaults.paragraph.frame_fill, f)?;
    count += try_map_value_colors(&mut copy.metadata, f)?
        + try_map_value_colors(&mut copy.color_styles, f)?;
    for colour in &mut copy.palette {
        count += mapped_colour(colour, f)?;
    }
    *doc = copy;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Rect, Size};
    use crate::style::{Fill, Fountain, Stroke};

    fn rect(id: u64) -> Shape {
        Shape::new(
            ShapeId(id),
            ShapeKind::Rect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                radius: 0.0,
                corners: None,
            },
        )
    }

    fn is_model(c: Color, m: &str) -> bool {
        c.model_name() == m
    }

    fn patterned_outline(id: u64) -> Stroke {
        let mut child = rect(id);
        child.fill = Fill::Solid(Color::Cmyk {
            c: 0.2,
            m: 0.4,
            y: 0.6,
            k: 0.1,
        });
        let mut stroke = Stroke::new(Color::BLACK, 0.2);
        stroke.fill = Some(Box::new(Fill::Pattern(crate::Pattern::Vector {
            shapes: vec![child],
            tile: Size::new(10.0, 10.0),
        })));
        stroke
    }

    fn outlined_pattern_document() -> Document {
        let mut doc = Document::new("Outline pattern resources", Size::new(100.0, 100.0));
        let mut graphic = rect(1);
        graphic.stroke = Some(patterned_outline(100));
        graphic.extra_strokes.push(patterned_outline(101));
        graphic.qr = Some(Box::new(crate::document::QrCode {
            outline: Some(patterned_outline(102)),
            ..Default::default()
        }));
        let mut span = crate::TextSpan::new("Retained text", "Sans", 12.0);
        span.layout_only = true;
        span.outline = Some(patterned_outline(103));
        let text = Shape::new(
            ShapeId(2),
            ShapeKind::Text {
                spans: vec![span],
                origin: crate::geometry::Point::new(1.0, 2.0),
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            },
        );
        let mut table = crate::document::Table::new(
            Rect::new(0.0, 0.0, 20.0, 20.0),
            1,
            1,
            Some(patterned_outline(104)),
        );
        let mut edge = patterned_outline(105);
        // A nested text in a cell edge used to be serialized by the generic
        // colour mapper, losing its deliberately skipped layout state.
        if let Some(fill) = edge.fill.as_deref_mut() {
            if let Fill::Pattern(crate::Pattern::Vector { shapes, .. }) = fill {
                let mut nested = crate::TextSpan::new("Edge text", "Sans", 9.0);
                nested.layout_only = true;
                shapes[0].kind = ShapeKind::Text {
                    spans: vec![nested],
                    origin: Default::default(),
                    frame: None,
                    align: Default::default(),
                    para: Default::default(),
                    on_path: None,
                };
            }
        }
        table.cells[0].borders.top = crate::table::EdgeBorder::Stroke(Box::new(edge));
        let mut cell_span = crate::TextSpan::new("Cell text", "Sans", 10.0);
        cell_span.outline = Some(patterned_outline(106));
        table.cells[0].text.push(cell_span);
        doc.pages[0].layers[0].shapes = vec![
            graphic,
            text,
            Shape::new(ShapeId(3), ShapeKind::Table(table)),
        ];
        let mut styled = rect(4);
        styled.stroke = Some(patterned_outline(107));
        doc.object_styles
            .push(crate::document::ObjectStyle::from_shape(
                "Outline style",
                &styled,
            ));
        doc.defaults.graphic.stroke = Some(patterned_outline(108));
        doc.defaults.artistic_text.stroke = Some(patterned_outline(109));
        doc.defaults.paragraph_text.stroke = Some(patterned_outline(110));
        doc
    }

    #[test]
    fn outline_patterns_are_visible_to_borrowed_and_mutable_resource_visitors() {
        let mut doc = outlined_pattern_document();
        let mut physical_depths = std::collections::BTreeMap::new();
        let physical_count =
            try_visit_shape_color_tree(&doc.pages[0].layers[0].shapes[0], &mut |shape, depth| {
                physical_depths.insert(shape.id, depth);
                Ok(())
            })
            .unwrap();
        assert_eq!(physical_count, 4);
        assert_eq!(physical_depths[&ShapeId(1)], 0);
        for id in 100..=102 {
            assert_eq!(physical_depths[&ShapeId(id)], 1);
        }
        let mut borrowed = std::collections::BTreeSet::new();
        try_visit_document_shape_tree(&doc, &mut |shape| {
            assert!(borrowed.insert(shape.id));
            Ok(())
        })
        .unwrap();
        for id in 100..=110 {
            assert!(
                borrowed.contains(&ShapeId(id)),
                "missing outline container {id}"
            );
        }
        let mut mutable = std::collections::BTreeSet::new();
        try_map_document_shape_tree(&mut doc, &mut |shape| {
            assert!(mutable.insert(shape.id));
            shape
                .data
                .push(("visited".into(), shape.id.raw().to_string()));
            Ok(())
        })
        .unwrap();
        assert_eq!(borrowed, mutable);
        try_visit_document_shape_tree(&doc, &mut |shape| {
            assert!(shape.data.iter().any(|(key, _)| key == "visited"));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn outline_pattern_colours_convert_without_losing_nested_text_state() {
        let mut doc = outlined_pattern_document();
        try_map_document_colors(&mut doc, &mut |color| {
            Ok(match color {
                Color::Cmyk { .. } => Color::Registration,
                other => other,
            })
        })
        .unwrap();
        let mut found = 0;
        try_visit_document_shape_tree(&doc, &mut |shape| {
            if (100..=110).contains(&shape.id.raw()) {
                assert_eq!(shape.fill, Fill::Solid(Color::Registration));
                found += 1;
            }
            if shape.id == ShapeId(105) {
                let ShapeKind::Text { spans, .. } = &shape.kind else {
                    panic!("edge text");
                };
                assert!(spans[0].layout_only);
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(found, 11);
    }

    #[test]
    fn gradient_and_solid_outline_fills_convert_as_document_colours() {
        let ink = Color::Cmyk {
            c: 0.2,
            m: 0.4,
            y: 0.6,
            k: 0.1,
        };
        let mut gradient = Stroke::new(Color::BLACK, 0.2);
        gradient.fill = Some(Box::new(Fill::Fountain(Fountain::two(
            crate::style::FountainKind::Linear,
            ink,
            ink,
            0.0,
        ))));
        let mut solid = Stroke::new(Color::BLACK, 0.2);
        solid.fill = Some(Box::new(Fill::Solid(ink)));
        let mut shape = rect(1);
        shape.stroke = Some(gradient);
        let style = crate::document::ObjectStyle::from_shape("Gradient outline", &shape);
        shape.extra_strokes.push(solid.clone());
        shape.qr = Some(Box::new(crate::document::QrCode {
            outline: Some(solid.clone()),
            ..Default::default()
        }));
        let mut doc = Document::new("Outline fill colours", Size::new(100.0, 100.0));
        // Keep the fixture focused on its seven outline-fill colours rather
        // than also converting the two default CMYK text fills.
        doc.defaults.artistic_text.fill = Fill::None;
        doc.defaults.paragraph_text.fill = Fill::None;
        doc.pages[0].layers[0].shapes.push(shape);
        doc.object_styles.push(style);
        doc.defaults.graphic.stroke = Some(solid);
        let count = try_map_document_colors(&mut doc, &mut |color| {
            Ok(match color {
                Color::Cmyk { .. } => Color::Registration,
                other => other,
            })
        })
        .unwrap();
        assert_eq!(count, 7, "two gradients and three solid outline fills");
        for stroke in [
            doc.pages[0].layers[0].shapes[0].stroke.as_ref().unwrap(),
            doc.object_styles[0].stroke.as_ref().unwrap(),
        ] {
            let Some(Fill::Fountain(ramp)) = stroke.fill.as_deref() else {
                panic!("gradient");
            };
            assert!(
                ramp.stops
                    .iter()
                    .all(|stop| stop.color == Color::Registration)
            );
        }
        assert_eq!(
            doc.defaults
                .graphic
                .stroke
                .as_ref()
                .unwrap()
                .fill
                .as_deref(),
            Some(&Fill::Solid(Color::Registration))
        );
    }

    #[test]
    fn excessive_nesting_inside_outline_patterns_is_rejected_before_clone() {
        let mut shape = rect(1);
        for id in 2..=131 {
            let mut outer = rect(id);
            let mut stroke = Stroke::new(Color::BLACK, 0.2);
            stroke.fill = Some(Box::new(Fill::Pattern(crate::Pattern::Vector {
                shapes: vec![shape],
                tile: Size::new(10.0, 10.0),
            })));
            outer.stroke = Some(stroke);
            shape = outer;
        }
        assert!(validate_shape_color_trees(std::slice::from_ref(&shape)).is_err());
        // Dismantle the hostile fixture iteratively rather than exercising
        // recursive destructors on a deliberately over-budget tree.
        while let Some(fill) = shape.stroke.as_mut().and_then(|stroke| stroke.fill.take()) {
            let Fill::Pattern(crate::Pattern::Vector { mut shapes, .. }) = *fill else {
                break;
            };
            let Some(child) = shapes.pop() else {
                break;
            };
            shape = child;
        }
    }

    #[test]
    fn borrowed_colour_tree_budget_rejects_depth_before_clone() {
        let mut shape = rect(1);
        for id in 2..=130 {
            shape = Shape::new(
                ShapeId(id),
                ShapeKind::Group {
                    children: vec![shape],
                },
            );
        }
        assert!(validate_shape_color_trees(std::slice::from_ref(&shape)).is_err());
        let shallow = rect(131);
        assert!(validate_shape_color_trees(std::slice::from_ref(&shallow)).is_ok());
    }

    #[test]
    fn pure_colour_mapper_preserves_unsaved_text_and_bitmap_payloads() {
        let mut span = crate::TextSpan::new("Live text", "Sans", 12.0);
        span.layout_only = true;
        span.fill = Some(Fill::Solid(Color::rgb8(7, 8, 9)));
        span.background = Some(Fill::Solid(Color::rgb8(9, 8, 7)));
        let text = Shape::new(
            ShapeId(1),
            ShapeKind::Text {
                spans: vec![span],
                origin: crate::geometry::Point::new(1.0, 2.0),
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            },
        );
        let mut bitmap = Shape::new(
            ShapeId(2),
            ShapeKind::Bitmap {
                rect: Rect::new(1.0, 2.0, 3.0, 4.0),
                width_px: 1,
                height_px: 1,
                png: vec![1, 2, 3, 4].into(),
                fx: None,
            },
        );
        bitmap.cmyk_bitmap = true;
        bitmap.picture_inks = Some(Box::new(crate::picture_inks::PictureInks::cmyk(
            b"preserved",
            vec![17, 33, 49, 65].into(),
        )));
        let mut doc = Document::new("pure", Size::new(100.0, 100.0));
        doc.pages[0].layers[0].shapes = vec![text, bitmap.clone()];
        try_map_document_colors(&mut doc, &mut |_| Ok(Color::Registration)).unwrap();
        let ShapeKind::Text { spans, .. } = &doc.pages[0].layers[0].shapes[0].kind else {
            panic!("text");
        };
        assert!(
            spans[0].layout_only,
            "serde-skipped layout state must survive"
        );
        assert_eq!(spans[0].fill, Some(Fill::Solid(Color::Registration)));
        assert_eq!(spans[0].background, Some(Fill::Solid(Color::Registration)));
        let mapped_bitmap = &doc.pages[0].layers[0].shapes[1];
        assert_eq!(mapped_bitmap.kind, bitmap.kind);
        assert_eq!(mapped_bitmap.picture_inks, bitmap.picture_inks);
        assert!(mapped_bitmap.cmyk_bitmap);
    }

    #[test]
    fn pure_colour_mapper_late_symbol_failure_is_atomic() {
        let mut doc = Document::new("atomic", Size::new(100.0, 100.0));
        let mut first = rect(1);
        first.fill = Fill::Solid(Color::rgb8(10, 20, 30));
        doc.pages[0].layers[0].shapes.push(first);
        let sentinel = Color::rgb8(213, 17, 29);
        let mut last = rect(2);
        last.fill = Fill::Solid(sentinel);
        doc.symbols.push(crate::document::Symbol {
            name: "unused".into(),
            shape: last,
        });
        let before = doc.clone();
        let mut visited = 0;
        let result = try_map_document_colors(&mut doc, &mut |c| {
            visited += 1;
            if c == sentinel {
                Err("late symbol failure".into())
            } else {
                Ok(Color::Registration)
            }
        });
        assert!(result.is_err());
        assert!(visited > 1);
        assert_eq!(doc, before, "all earlier temporary edits must be discarded");
        assert_eq!(doc.ids(), before.ids());
    }

    #[test]
    fn guarded_colour_command_preserves_external_fonts_ids_and_rejects_stale() {
        let mut doc = Document::new("guard", Size::new(100.0, 100.0));
        doc.pages[0].layers[0].shapes.push(rect(1));
        let before = DocumentColorState::capture(&doc);
        let mut changed = doc.clone();
        changed.pages[0].layers[0].shapes[0].fill = Fill::Solid(Color::Registration);
        let command = crate::Command::SetDocumentColorState {
            before: Box::new(before),
            after: Box::new(DocumentColorState::capture(&changed)),
        };
        doc.fonts.push(crate::document::DocumentFont::new(
            "External",
            vec![1, 2, 3],
        ));
        doc.ids_mut().next();
        let ids = doc.ids().clone();
        let fonts = doc.fonts.clone();
        command.apply(&mut doc).unwrap();
        assert_eq!(doc.ids(), &ids);
        assert_eq!(doc.fonts, fonts);
        let committed = doc.clone();
        assert!(matches!(
            command.apply(&mut doc),
            Err(crate::Error::StaleColorState)
        ));
        assert_eq!(doc, committed);
    }

    #[test]
    fn every_colour_goes_to_the_model_asked_spots_and_registration_stay() {
        let mut doc = Document::new("t", Size::new(100.0, 100.0));
        let mut a = rect(1);
        a.fill = Fill::Fountain(Fountain::two(
            crate::style::FountainKind::Linear,
            Color::rgb8(255, 0, 0),
            Color::Hsb {
                h: 120.0,
                s: 1.0,
                b: 1.0,
            },
            0.0,
        ));
        a.stroke = Some(Stroke::new(Color::Registration, 0.2));
        let mut b = rect(2);
        let spot = Color::Spot {
            ink: 1,
            tint: 0.5,
            c: 0.0,
            m: 1.0,
            y: 0.0,
            k: 0.0,
        };
        b.fill = Fill::Solid(spot);
        b.stroke = Some(Stroke::new(Color::rgb8(0, 0, 255), 0.2));
        let mut g = Shape::new(ShapeId(3), ShapeKind::Group { children: vec![b] });
        g.fill = Fill::None;
        g.stroke = None;
        doc.pages[0].layers[0].shapes = vec![a, g];
        doc.pages[0].background = Some(Fill::Solid(Color::rgb8(250, 250, 240)));
        let n = convert_document(&mut doc, &[], ColorTarget::Cmyk, false);
        assert_eq!(n, 4, "two stops, the blue outline, the background");
        let shapes = &doc.pages[0].layers[0].shapes;
        let Fill::Fountain(f) = &shapes[0].fill else {
            panic!("fountain");
        };
        assert!(f.stops.iter().all(|s| is_model(s.color, "CMYK")));
        assert_eq!(
            shapes[0].stroke.as_ref().map(|s| s.color),
            Some(Color::Registration)
        );
        let ShapeKind::Group { children } = &shapes[1].kind else {
            panic!("group");
        };
        assert_eq!(children[0].fill, Fill::Solid(spot), "spots stay");
        assert!(
            children[0]
                .stroke
                .as_ref()
                .is_some_and(|s| is_model(s.color, "CMYK"))
        );
        assert!(matches!(doc.pages[0].background, Some(Fill::Solid(c)) if is_model(c, "CMYK")));
        // Again: nothing left to change.
        assert_eq!(convert_document(&mut doc, &[], ColorTarget::Cmyk, false), 0);
        // Spots when asked, and back to RGB for the selected group only.
        assert_eq!(
            convert_document(&mut doc, &[ShapeId(3)], ColorTarget::Rgb, true),
            2
        );
        let shapes = &doc.pages[0].layers[0].shapes;
        let ShapeKind::Group { children } = &shapes[1].kind else {
            panic!("group");
        };
        assert!(matches!(children[0].fill, Fill::Solid(c) if is_model(c, "RGB")));
        let Fill::Fountain(f) = &shapes[0].fill else {
            panic!("fountain");
        };
        assert!(
            f.stops.iter().all(|s| is_model(s.color, "CMYK")),
            "not selected"
        );
    }

    #[test]
    fn colours_of_effects_and_text_convert_too() {
        let mut s = Shape::new(
            ShapeId(1),
            ShapeKind::Text {
                spans: vec![crate::TextSpan::new("x", "Sans", 12.0)],
                origin: crate::geometry::Point::new(0.0, 0.0),
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            },
        );
        s.fill = Fill::Solid(Color::rgb8(10, 20, 30));
        s.stroke = None;
        s.shadow = Some(crate::document::Shadow {
            color: Color::rgb8(0, 0, 0),
            ..Default::default()
        });
        let before = serde_json::to_string(&s).unwrap_or_default();
        let n = convert_shape(
            &mut s,
            &|c| converted(c, ColorTarget::Cmyk, false),
            ColorTarget::Cmyk,
        );
        assert!(n >= 2, "the fill and the shadow's colour: {n}");
        let after = serde_json::to_string(&s).unwrap_or_default();
        assert_ne!(before, after);
        assert!(!after.contains("\"model\":\"rgb\""), "{after}");
    }

    /// A bitmap takes the colour mode asked for and counts as one change;
    /// a mode it has already changes nothing (#1254). Inside a group too,
    /// and the drawing's bitmaps whose mode changes are listed for their
    /// pixels.
    #[test]
    fn bitmaps_take_the_colour_mode() {
        let mut doc = Document::new("t", Size::new(100.0, 100.0));
        let mut bm = Shape::new(
            ShapeId(1),
            ShapeKind::Bitmap {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                width_px: 1,
                height_px: 1,
                png: Vec::new().into(),
                fx: None,
            },
        );
        bm.fill = Fill::None;
        bm.stroke = None;
        let mut g = Shape::new(ShapeId(2), ShapeKind::Group { children: vec![bm] });
        g.fill = Fill::None;
        g.stroke = None;
        doc.pages[0].layers[0].shapes = vec![g];
        assert_eq!(
            bitmaps_changing_mode(&doc, &[], ColorTarget::Cmyk),
            vec![ShapeId(1)]
        );
        assert_eq!(
            bitmaps_changing_mode(&doc, &[ShapeId(2)], ColorTarget::Rgb),
            Vec::new()
        );
        assert_eq!(
            convert_document(&mut doc, &[ShapeId(2)], ColorTarget::Cmyk, false),
            1
        );
        let inside = |doc: &Document| match &doc.pages[0].layers[0].shapes[0].kind {
            ShapeKind::Group { children } => children[0].cmyk_bitmap,
            _ => panic!("group"),
        };
        assert!(inside(&doc));
        assert_eq!(convert_document(&mut doc, &[], ColorTarget::Cmyk, false), 0);
        assert_eq!(
            bitmaps_changing_mode(&doc, &[], ColorTarget::Rgb),
            vec![ShapeId(1)]
        );
        let set_inks = |doc: &mut Document| {
            if let ShapeKind::Group { children } = &mut doc.pages[0].layers[0].shapes[0].kind {
                children[0].picture_inks = Some(Box::new(crate::picture_inks::PictureInks::cmyk(
                    b"png",
                    vec![0u8; 4].into(),
                )));
            }
        };
        let has_inks = |doc: &Document| match &doc.pages[0].layers[0].shapes[0].kind {
            ShapeKind::Group { children } => children[0].picture_inks.is_some(),
            _ => panic!("group"),
        };
        set_inks(&mut doc);
        assert_eq!(convert_document(&mut doc, &[], ColorTarget::Rgb, false), 1);
        assert!(!inside(&doc));
        // An RGB picture no longer prints from its CMYK channels (#1659).
        assert!(!has_inks(&doc));
        // The mode survives the saved form.
        let json = serde_json::to_string(&doc.pages[0].layers[0].shapes[0]).unwrap_or_default();
        assert!(!json.contains("cmyk_bitmap"), "off is the default: {json}");
    }
}
