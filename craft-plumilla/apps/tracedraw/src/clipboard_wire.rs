//! Pure, bounded transport of a compact native selection alongside its SVG.
//! No operating-system clipboard, font installation, file access or rendering.

use crate::clipboard::NativeClipboardPayload;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Write};
use tracedraw_core::color_convert as trees;
use tracedraw_core::geometry::{Point, Rect};
use tracedraw_core::text_flow::{FrameFlow, TextWrap, WrapRegion};
use tracedraw_core::{Document, DocumentProfile, ShapeKind};

pub(crate) const NAMESPACE: &str = "urn:craft-corel:selection";
pub(crate) const MAX_SVG_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_JSON_BYTES: usize = 32 * 1024 * 1024;
pub(crate) const MAX_BINARY_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const MAX_JSON_DEPTH: usize = 1024;
pub(crate) const MAX_JSON_TOKENS: usize = 1_000_000;
pub(crate) const MAX_SHAPES: usize = 10_000;
pub(crate) const MAX_RESOURCES: usize = 1_024;
pub(crate) const MAX_POLYGON_POINTS: u32 = 100_000;
pub(crate) const MAX_DECODED_PIXELS: u64 = 256 * 1024 * 1024;
const MAX_XML_NODES: u32 = 100_000;
const MAX_XML_TOKENS: usize = 200_000;
// SVG adds layer/root/link/effect wrappers around the model's 128 edges.
// Keep this separate from both model depth and JSON nesting budgets.
const MAX_XML_DEPTH: usize = 512;
const MAX_PROFILE_BYTES: usize = 4 * 1024 * 1024;
const MAX_PIXELS: u64 = 16 * 1024 * 1024;
const MAX_ARRAY: usize = 100_000;
#[cfg(not(target_arch = "wasm32"))]
const NATIVE_STACK_BYTES: usize = 16 * 1024 * 1024;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn work_stack<T: Send>(
    work: impl FnOnce() -> Result<T, String> + Send,
) -> Result<T, String> {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("native-selection-codec".into())
            .stack_size(NATIVE_STACK_BYTES)
            .spawn_scoped(scope, work)
            .map_err(|e| format!("cannot start native selection codec: {e}"))?
            .join()
            .map_err(|_| "native selection codec worker failed".to_string())?
    })
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn work_stack<T>(work: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    work()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u32,
    payload: NativeClipboardPayload,
    supplements: Supplements,
}

/// The model deliberately omits these two *derived* fields from saved drawings.
/// Transport records them too: no skipped state is silently lost on the wire.
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Supplements {
    shapes: Vec<ShapeSupplement>,
    default_flow: Flow,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ShapeSupplement {
    /// Stable ordinal in the mutable document-tree visitor, not an object id.
    ordinal: usize,
    spans: Vec<Vec<bool>>,
    flow: Option<Flow>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Flow {
    wrap: Vec<Region>,
    grid: Option<(f64, f64)>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Region {
    wrap: TextWrap,
    bbox: Rect,
    segments: Vec<(Point, Point)>,
}

impl Flow {
    fn capture(flow: &FrameFlow) -> Self {
        Self {
            wrap: flow
                .wrap
                .iter()
                .map(|r| Region {
                    wrap: r.wrap,
                    bbox: r.bbox,
                    segments: r.segments.clone(),
                })
                .collect(),
            grid: flow.grid,
        }
    }

    fn restore(self) -> FrameFlow {
        FrameFlow::new(
            self.wrap
                .into_iter()
                .map(|r| WrapRegion {
                    wrap: r.wrap,
                    bbox: r.bbox,
                    segments: r.segments,
                })
                .collect(),
            self.grid,
        )
    }
}

fn capture_supplements(doc: &mut Document) -> Result<Supplements, String> {
    let mut supplements = Supplements {
        default_flow: Flow::capture(&doc.defaults.paragraph.flow),
        ..Default::default()
    };
    let mut ordinal = 0;
    trees::try_map_document_shape_tree(doc, &mut |shape| {
        let entry = match &shape.kind {
            ShapeKind::Text { spans, para, .. } => Some(ShapeSupplement {
                ordinal,
                spans: vec![spans.iter().map(|s| s.layout_only).collect()],
                flow: Some(Flow::capture(&para.flow)),
            }),
            ShapeKind::Table(table) => Some(ShapeSupplement {
                ordinal,
                spans: table
                    .cells
                    .iter()
                    .map(|c| c.text.iter().map(|s| s.layout_only).collect())
                    .collect(),
                flow: None,
            }),
            _ => None,
        };
        if let Some(entry) = entry {
            supplements.shapes.push(entry);
        }
        ordinal += 1;
        Ok(())
    })?;
    Ok(supplements)
}

fn restore_supplements(doc: &mut Document, supplements: Supplements) -> Result<(), String> {
    doc.defaults.paragraph.flow = supplements.default_flow.restore();
    let mut entries = supplements.shapes.into_iter().peekable();
    let mut ordinal = 0;
    trees::try_map_document_shape_tree(doc, &mut |shape| {
        let requires = matches!(shape.kind, ShapeKind::Text { .. } | ShapeKind::Table(_));
        if requires {
            let entry = entries
                .next()
                .ok_or("native selection is missing text supplements")?;
            if entry.ordinal != ordinal {
                return Err("native text supplement order is invalid".into());
            }
            let mut sets = entry.spans.into_iter();
            let mut restore = |spans: &mut [tracedraw_core::TextSpan]| -> Result<(), String> {
                let flags = sets
                    .next()
                    .ok_or("native selection is missing text flags")?;
                if flags.len() != spans.len() {
                    return Err("native text flags have the wrong length".into());
                }
                for (span, flag) in spans.iter_mut().zip(flags) {
                    span.layout_only = flag;
                }
                Ok(())
            };
            match &mut shape.kind {
                ShapeKind::Text { spans, para, .. } => {
                    restore(spans)?;
                    para.flow = entry.flow.ok_or("native text flow is missing")?.restore();
                }
                ShapeKind::Table(table) => {
                    if entry.flow.is_some() {
                        return Err("native table has unexpected text flow".into());
                    }
                    for cell in &mut table.cells {
                        restore(&mut cell.text)?;
                    }
                }
                _ => return Err("native text supplement kind is invalid".into()),
            }
            if sets.next().is_some() {
                return Err("native text supplement has extra flags".into());
            }
        } else if entries.peek().is_some_and(|e| e.ordinal <= ordinal) {
            return Err("native text supplement points to another kind of object".into());
        }
        ordinal += 1;
        Ok(())
    })?;
    if entries.next().is_some() {
        return Err("native selection has extra text supplements".into());
    }
    Ok(())
}

/// Conservative recognizer for damaged own data, plus namespace-aware parsing
/// for valid XML using character references or a different namespace prefix.
pub(crate) fn looks_native(text: &str) -> bool {
    if !xml_context(text) {
        return false;
    }
    looks_native_literal(text)
        || (text.len() <= MAX_SVG_BYTES
            && xml(text).is_ok_and(|doc| {
                doc.descendants()
                    .any(|n| n.tag_name().namespace() == Some(NAMESPACE))
            }))
}

fn xml(text: &str) -> Result<roxmltree::Document<'_>, String> {
    if text.len() > MAX_SVG_BYTES {
        return Err("SVG exceeds the native selection byte limit".into());
    }
    // Attribute records are not included in roxmltree's node count. Bound the
    // lexical delimiters too, before the XML parser allocates those records.
    if text
        .bytes()
        .filter(|b| matches!(b, b'<' | b'=' | b'&'))
        .take(MAX_XML_TOKENS + 1)
        .count()
        > MAX_XML_TOKENS
    {
        return Err("SVG exceeds the native selection lexical token limit".into());
    }
    let doc = roxmltree::Document::parse_with_options(
        text,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: MAX_XML_NODES,
            ..Default::default()
        },
    )
    .map_err(|e| format!("invalid selection XML: {e}"))?;
    if doc
        .descendants()
        .any(|n| n.ancestors().take(MAX_XML_DEPTH + 2).count() > MAX_XML_DEPTH + 1)
    {
        return Err("SVG exceeds the native selection nesting limit".into());
    }
    Ok(doc)
}

fn own_block<'a, 'input>(
    doc: &'a roxmltree::Document<'input>,
) -> Result<Option<roxmltree::Node<'a, 'input>>, String> {
    let mut nodes = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().namespace() == Some(NAMESPACE));
    let Some(node) = nodes.next() else {
        return Ok(None);
    };
    if nodes.next().is_some() {
        return Err("native selection contains duplicate blocks".into());
    }
    if node.tag_name().name() != "selection"
        || node.attribute("version") != Some("1")
        || node.attribute("encoding") != Some("base64-json")
        || node.attributes().len() != 2
        || node
            .children()
            .any(|n| n.is_element() || n.is_comment() || n.is_pi())
    {
        return Err("unsupported or damaged native selection block".into());
    }
    let root = doc.root_element();
    if root.tag_name().name() != "svg"
        || !matches!(
            root.tag_name().namespace(),
            None | Some("http://www.w3.org/2000/svg")
        )
        || !node
            .parent_element()
            .is_some_and(|n| n.tag_name().name() == "metadata" && n.parent_element() == Some(root))
    {
        return Err("native selection block is outside SVG metadata".into());
    }
    Ok(Some(node))
}

struct LimitedWriter(Vec<u8>);
impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_JSON_BYTES.saturating_sub(self.0.len()) {
            return Err(std::io::Error::other(
                "native selection exceeds the JSON byte limit",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut out = LimitedWriter(Vec::new());
    serde_json::to_writer(&mut out, value).map_err(|e| e.to_string())?;
    preflight(&out.0)?;
    Ok(out.0)
}

/// No application state changes until the caller has published the returned
/// text successfully. The visible SVG is retained byte for byte.
pub(crate) fn encode_svg(payload: &NativeClipboardPayload, svg: &str) -> Result<String, String> {
    borrowed_guard(payload)?;
    if svg.len() > MAX_SVG_BYTES {
        return Err("SVG exceeds the native selection byte limit".into());
    }
    work_stack(|| encode_svg_inner(payload, svg))
}

fn encode_svg_inner(payload: &NativeClipboardPayload, svg: &str) -> Result<String, String> {
    let parsed = xml(svg)?;
    if own_block(&parsed)?.is_some() {
        return Err("SVG already contains a native selection".into());
    }
    let root = parsed.root_element();
    if root.tag_name().name() != "svg"
        || !matches!(
            root.tag_name().namespace(),
            None | Some("http://www.w3.org/2000/svg")
        )
    {
        return Err("selection preview is not SVG".into());
    }
    validate_native_inner(payload)?;
    let mut copy = payload.clone();
    let supplements = capture_supplements(&mut copy.document)?;
    let bytes = json_bytes(&Wire {
        version: 1,
        payload: copy,
        supplements,
    })?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let block = format!("<metadata><cc:selection xmlns:cc=\"{NAMESPACE}\" version=\"1\" encoding=\"base64-json\">{encoded}</cc:selection></metadata>");
    let range = root.range();
    let root_text = svg.get(range.clone()).ok_or("SVG root range is invalid")?;
    let name = root_text
        .get(1..)
        .and_then(|s| {
            s.split(|c: char| c.is_whitespace() || c == '/' || c == '>')
                .next()
        })
        .ok_or("invalid SVG root name")?;
    let extra = if root_text.ends_with("/>") {
        name.len() + 3
    } else {
        0
    };
    if svg
        .len()
        .checked_add(block.len())
        .and_then(|n| n.checked_add(extra))
        .is_none_or(|n| n > MAX_SVG_BYTES)
    {
        return Err("SVG with native selection exceeds its byte limit".into());
    }
    let mut output = String::with_capacity(svg.len() + block.len() + extra);
    if root_text.ends_with("/>") {
        let end = range.end.checked_sub(2).ok_or("invalid empty SVG range")?;
        // Preserve the original namespace prefix on the closing root tag.
        output.push_str(svg.get(..end).ok_or("invalid SVG root end")?);
        output.push('>');
        output.push_str(&block);
        output.push_str("</");
        output.push_str(name);
        output.push('>');
        output.push_str(svg.get(range.end..).ok_or("invalid SVG trailer")?);
    } else {
        let close = root_text.rfind("</").ok_or("SVG closing tag is missing")? + range.start;
        output.push_str(svg.get(..close).ok_or("invalid SVG closing range")?);
        output.push_str(&block);
        output.push_str(svg.get(close..).ok_or("invalid SVG suffix")?);
    }
    if output.len() > MAX_SVG_BYTES {
        return Err("SVG with native selection exceeds its byte limit".into());
    }
    Ok(output)
}

/// Foreign data is None; damaged or unsupported own data is always an error.
/// No lossy SVG fallback is possible after recognizing an own block.
pub(crate) fn decode_svg(svg: &str) -> Result<Option<NativeClipboardPayload>, String> {
    if svg.len() > MAX_SVG_BYTES {
        return Err("SVG exceeds the native selection byte limit".into());
    }
    work_stack(|| decode_svg_inner(svg))
}

fn decode_svg_inner(svg: &str) -> Result<Option<NativeClipboardPayload>, String> {
    let parsed = match xml(svg) {
        Ok(doc) => doc,
        Err(error) if looks_native_literal(svg) || xml_context(svg) => return Err(error),
        Err(_) => return Ok(None),
    };
    let Some(block) = own_block(&parsed)? else {
        return Ok(None);
    };
    let text = block.text().ok_or("native selection block is empty")?;
    let max_encoded = MAX_JSON_BYTES
        .div_ceil(3)
        .checked_mul(4)
        .ok_or("invalid native byte limit")?;
    if text.len() > max_encoded {
        return Err("native selection exceeds its encoded byte limit".into());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|e| format!("invalid native selection base64: {e}"))?;
    preflight(&bytes)?;
    let mut wire: Wire = deserialize(&bytes)?;
    if wire.version != 1 {
        return Err("unsupported native selection JSON version".into());
    }
    // The model intentionally ignores unknown fields in ordinary files. A
    // versioned clipboard must not silently discard any such extension. Compare
    // semantic JSON against the same schema serialized back, before moving or
    // regenerating anything. Whitespace and object-key order remain immaterial.
    let input: serde_json::Value = deserialize(&bytes)?;
    let canonical: serde_json::Value = deserialize(&json_bytes(&wire)?)?;
    if input != canonical {
        return Err("native selection contains unsupported or noncanonical model fields".into());
    }
    drop(input);
    drop(canonical);
    restore_supplements(&mut wire.payload.document, wire.supplements)?;
    validate(&wire.payload)?;
    crate::clipboard_preview::budget(&wire.payload.document)?;
    crate::app::import_document::validate_resources(&wire.payload.document)?;
    // Re-capture is a dependency-closure check: unrelated pages/resources and
    // missing font/style/ink/symbol dependencies cannot hitchhike in metadata.
    let page = wire
        .payload
        .document
        .pages
        .first()
        .ok_or("native selection has no page")?;
    let shapes = &page
        .layers
        .first()
        .ok_or("native selection has no layer")?
        .shapes;
    let compact = NativeClipboardPayload::capture(
        &wire.payload.document,
        shapes,
        wire.payload.source_instance,
    )?;
    if compact.document.symbols != wire.payload.document.symbols
        || compact.document.object_styles != wire.payload.document.object_styles
        || compact.document.inks != wire.payload.document.inks
        || compact.document.fonts != wire.payload.document.fonts
    {
        return Err("native selection is not a compact dependency closure".into());
    }
    Ok(Some(wire.payload))
}

fn looks_native_literal(text: &str) -> bool {
    if !xml_context(text) {
        return false;
    }
    if text.contains(NAMESPACE) {
        return true;
    }
    if !text.contains('&') {
        return false;
    }
    // Recognize an entity-escaped namespace even when the enclosing XML is
    // truncated. This does not parse/decode arbitrary document contents and
    // allocates nothing. The namespace contains its initial 'u' only once.
    let target = NAMESPACE.as_bytes();
    let mut matched = 0;
    let mut offset = 0;
    while let Some(&byte) = text.as_bytes().get(offset) {
        let mut current = byte;
        if byte == b'&' && text.as_bytes().get(offset + 1) == Some(&b'#') {
            let mut at = offset + 2;
            let radix = if text.as_bytes().get(at) == Some(&b'x') {
                at += 1;
                16
            } else {
                10
            };
            let start = at;
            let mut code = Some(0u8);
            while let Some(digit) = text
                .as_bytes()
                .get(at)
                .and_then(|b| char::from(*b).to_digit(radix))
            {
                code = code
                    .and_then(|n| n.checked_mul(radix as u8))
                    .and_then(|n| n.checked_add(digit as u8));
                at += 1;
            }
            if at > start && text.as_bytes().get(at) == Some(&b';') {
                if let Some(code) = code {
                    current = code;
                    offset = at;
                }
            }
        }
        if target.get(matched) == Some(&current) {
            matched += 1;
            if matched == target.len() {
                return true;
            }
        } else {
            matched = usize::from(target.first() == Some(&current));
        }
        offset += 1;
    }
    false
}

fn xml_context(text: &str) -> bool {
    text.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
        .starts_with('<')
}

/// The publisher calls this *before* making a visible SVG preview. It does not
/// mutate the selection and rejects size/depth/resources/cycles first. Preview
/// expansion itself needs the caller's separate work budget for symbol DAGs.
pub(crate) fn validate_native(payload: &NativeClipboardPayload) -> Result<(), String> {
    borrowed_guard(payload)?;
    work_stack(|| validate_native_inner(payload))
}

fn borrowed_guard(payload: &NativeClipboardPayload) -> Result<(), String> {
    let doc = &payload.document;
    if payload.version != 1
        || doc.pages.len() != 1
        || doc.pages.first().is_none_or(|p| {
            p.layers.len() != 1 || p.layers.first().is_none_or(|l| l.shapes.len() > MAX_SHAPES)
        })
        || !doc.master.is_empty()
        || doc.symbols.len() > MAX_RESOURCES
        || doc.fonts.len() > MAX_RESOURCES
        || doc.inks.len() > MAX_RESOURCES
        || doc.object_styles.len() > MAX_RESOURCES
    {
        return Err("native selection exceeds its preliminary structural limits".into());
    }
    let mut count = 0;
    trees::try_visit_document_shape_tree(doc, &mut |shape| {
        count += 1;
        if count > MAX_SHAPES {
            return Err("native selection exceeds its preliminary object limit".into());
        }
        if matches!(&shape.kind, ShapeKind::Group { children } if children.len() > MAX_SHAPES)
            || matches!(&shape.kind, ShapeKind::ClipFrame { contents, .. } if contents.len() > MAX_SHAPES)
        {
            return Err("native selection exceeds its preliminary child limit".into());
        }
        Ok(())
    })?;
    Ok(())
}

fn validate_native_inner(payload: &NativeClipboardPayload) -> Result<(), String> {
    validate(payload)?;
    // Bound serialized state (including binary fields) before cloning its trees.
    // The strict read also rejects nulls emitted from non-finite model numbers.
    let preliminary = json_bytes(payload)?;
    let _: NativeClipboardPayload = deserialize(&preliminary)?;
    crate::clipboard_preview::budget(&payload.document)?;
    crate::app::import_document::validate_resources(&payload.document)
}

fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    preflight(bytes)?;
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    // The iterative preflight enforces the model's existing 1024-bracket cap,
    // byte/token/resource limits and duplicate-key rejection before this call.
    decoder.disable_recursion_limit();
    let value =
        T::deserialize(&mut decoder).map_err(|e| format!("invalid native selection JSON: {e}"))?;
    decoder.end().map_err(|e| e.to_string())?;
    Ok(value)
}

fn profile(profile: &DocumentProfile) -> Result<(), String> {
    if profile.data.len() > MAX_PROFILE_BYTES {
        return Err("native ICC profile exceeds its byte limit".into());
    }
    let parsed = tracedraw_core::icc::Profile::parse(&profile.data)
        .map_err(|e| format!("invalid native ICC profile: {e}"))?;
    if parsed.color_space() != tracedraw_core::icc::ColorSpace::Cmyk {
        return Err("native CMYK profile has another colour space".into());
    }
    Ok(())
}

fn fill_assets(
    fill: &tracedraw_core::Fill,
    add: &mut dyn FnMut(usize) -> Result<(), String>,
    decoded_pixels: &mut u64,
) -> Result<(), String> {
    if let tracedraw_core::Fill::Pattern(tracedraw_core::Pattern::Vector { shapes, .. }) = fill {
        if shapes.len() > MAX_SHAPES {
            return Err("native vector pattern exceeds its object count limit".into());
        }
    }
    if let tracedraw_core::Fill::Pattern(tracedraw_core::Pattern::Bitmap {
        png: bytes,
        width_px,
        height_px,
        ..
    }) = fill
    {
        add(bytes.len())?;
        png(bytes, *width_px, *height_px, decoded_pixels)?;
    }
    Ok(())
}

fn stroke_assets(
    stroke: Option<&tracedraw_core::Stroke>,
    add: &mut dyn FnMut(usize) -> Result<(), String>,
    decoded_pixels: &mut u64,
) -> Result<(), String> {
    if let Some(fill) = stroke.and_then(|s| s.fill.as_deref()) {
        fill_assets(fill, add, decoded_pixels)?;
    }
    Ok(())
}

fn span_assets(
    spans: &[tracedraw_core::TextSpan],
    add: &mut dyn FnMut(usize) -> Result<(), String>,
    decoded_pixels: &mut u64,
) -> Result<(), String> {
    for span in spans {
        for fill in [&span.fill, &span.background].into_iter().flatten() {
            fill_assets(fill, add, decoded_pixels)?;
        }
        stroke_assets(span.outline.as_ref(), add, decoded_pixels)?;
    }
    Ok(())
}

fn flow_budget(flow: &FrameFlow) -> Result<usize, String> {
    if flow.wrap.len() > MAX_RESOURCES
        || flow
            .wrap
            .iter()
            .try_fold(0usize, |n, r| n.checked_add(r.segments.len()))
            .is_none_or(|n| n > MAX_ARRAY)
    {
        return Err("native text flow exceeds its resource limit".into());
    }
    if flow
        .grid
        .is_some_and(|(a, b)| !a.is_finite() || !b.is_finite())
        || flow.wrap.iter().any(|r| {
            !r.wrap.offset.is_finite()
                || ![r.bbox.x0, r.bbox.y0, r.bbox.x1, r.bbox.y1]
                    .iter()
                    .all(|v| v.is_finite())
                || r.segments
                    .iter()
                    .any(|(a, b)| ![a.x, a.y, b.x, b.y].iter().all(|v| v.is_finite()))
        })
    {
        return Err("native text flow contains non-finite values".into());
    }
    Ok(flow.wrap.len() + flow.wrap.iter().map(|r| r.segments.len()).sum::<usize>())
}

fn png_with_budget(
    bytes: &[u8],
    width: u32,
    height: u32,
    max_pixels: u64,
    decoded_pixels: &mut u64,
) -> Result<(), String> {
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > max_pixels {
        return Err("native bitmap exceeds its pixel limit".into());
    }
    if bytes.len() > MAX_BINARY_BYTES {
        return Err("native bitmap exceeds its byte limit".into());
    }
    // Reserve the aggregate *before* starting a decoder. Count every preview,
    // original, raster effect and stacked ink plane, even if highly compressible.
    // No hash or fingerprint is treated as authentication or byte identity.
    *decoded_pixels = decoded_pixels
        .checked_add(u64::from(width) * u64::from(height))
        .ok_or("native bitmap aggregate pixel count overflow")?;
    if *decoded_pixels > MAX_DECODED_PIXELS {
        return Err("native bitmaps exceed their aggregate decoded pixel budget".into());
    }
    let mut reader = image::ImageReader::with_format(Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(width);
    limits.max_image_height = Some(height);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| format!("invalid native bitmap PNG: {e}"))?;
    if image.width() != width || image.height() != height {
        return Err("native bitmap dimensions do not match its PNG".into());
    }
    Ok(())
}

fn png(bytes: &[u8], width: u32, height: u32, decoded_pixels: &mut u64) -> Result<(), String> {
    png_with_budget(bytes, width, height, MAX_PIXELS, decoded_pixels)
}

fn validate(payload: &NativeClipboardPayload) -> Result<(), String> {
    if payload.version != 1 {
        return Err("unsupported native selection payload version".into());
    }
    let doc = &payload.document;
    if doc.pages.len() != 1
        || doc.pages.first().is_none_or(|p| p.layers.len() != 1)
        || !doc.master.is_empty()
        || !doc.comments.is_empty()
        || !doc.selection_groups.is_empty()
        || doc.fonts.len() > MAX_RESOURCES
        || doc.symbols.len() > MAX_RESOURCES
        || doc.object_styles.len() > MAX_RESOURCES
        || doc.inks.len() > MAX_RESOURCES
    {
        return Err("native selection exceeds its compact resource limits".into());
    }
    let page = doc
        .pages
        .first()
        .ok_or("native selection page is missing")?;
    let layer = page
        .layers
        .first()
        .ok_or("native selection layer is missing")?;
    let baseline = Document::new("Clipboard", page.size);
    let base_page = baseline
        .pages
        .first()
        .ok_or("native baseline page is missing")?;
    let base_layer = base_page
        .layers
        .first()
        .ok_or("native baseline layer is missing")?;
    if doc.title != baseline.title
        || !doc.palette.is_empty()
        || !doc.color_styles.is_empty()
        || !doc.master_guides.is_empty()
        || doc.time_tracker != baseline.time_tracker
        || doc.saved_view.is_some()
        || doc.defaults != baseline.defaults
        || !page.guides.is_empty()
        || page.background.is_some()
        || page.id != base_page.id
        || page.name != base_page.name
        || layer.id != base_layer.id
        || layer.name != base_layer.name
        || layer.visible != base_layer.visible
        || layer.printable != base_layer.printable
        || layer.locked != base_layer.locked
        || layer.scope != base_layer.scope
        || layer.in_front != base_layer.in_front
    {
        return Err("native selection contains unrelated document or page resources".into());
    }
    // Only the actual CMYK working-profile label is recorded after capture by
    // the source app. All other document settings remain the compact baseline.
    // Serialize into the bounded writer before cloning any attacker-controlled
    // strings, then compare a small typed metadata value with that baseline.
    let mut metadata: tracedraw_core::document::Metadata =
        deserialize(&json_bytes(&doc.metadata)?)?;
    metadata.cmyk_profile.clear();
    if metadata != baseline.metadata {
        return Err("native selection contains unrelated document settings".into());
    }
    if doc
        .pages
        .first()
        .and_then(|p| p.layers.first())
        .is_some_and(|l| l.shapes.len() > MAX_SHAPES)
    {
        return Err("native selection exceeds its root object count limit".into());
    }
    let mut count = 0;
    let mut decoded_pixels = 0;
    let mut binary = 0usize;
    let mut add = |bytes: usize| -> Result<(), String> {
        binary = binary
            .checked_add(bytes)
            .ok_or("native binary byte count overflow")?;
        if binary > MAX_BINARY_BYTES {
            return Err("native binary resources exceed their total byte limit".into());
        }
        Ok(())
    };
    if let Some(p) = &doc.cmyk_profile {
        add(p.data.len())?;
        profile(p)?;
    }
    for font in &doc.fonts {
        add(font.data.len())?;
    }
    let mut flow_total = flow_budget(&doc.defaults.paragraph.flow)?;
    for page in &doc.pages {
        if let Some(fill) = &page.background {
            fill_assets(fill, &mut add, &mut decoded_pixels)?;
        }
    }
    for style in &doc.object_styles {
        fill_assets(&style.fill, &mut add, &mut decoded_pixels)?;
        stroke_assets(style.stroke.as_ref(), &mut add, &mut decoded_pixels)?;
    }
    for properties in [
        &doc.defaults.graphic,
        &doc.defaults.artistic_text,
        &doc.defaults.paragraph_text,
    ] {
        fill_assets(&properties.fill, &mut add, &mut decoded_pixels)?;
        stroke_assets(properties.stroke.as_ref(), &mut add, &mut decoded_pixels)?;
    }
    if let Some(fill) = &doc.defaults.paragraph.frame_fill {
        fill_assets(fill, &mut add, &mut decoded_pixels)?;
    }
    trees::try_visit_document_shape_tree(doc, &mut |shape| {
        count += 1;
        if count > MAX_SHAPES {
            return Err("native selection exceeds its object count limit".into());
        }
        fill_assets(&shape.fill, &mut add, &mut decoded_pixels)?;
        for fill in &shape.extra_fills {
            fill_assets(fill, &mut add, &mut decoded_pixels)?;
        }
        stroke_assets(shape.stroke.as_ref(), &mut add, &mut decoded_pixels)?;
        for stroke in &shape.extra_strokes {
            stroke_assets(Some(stroke), &mut add, &mut decoded_pixels)?;
        }
        if let Some(qr) = &shape.qr {
            fill_assets(&qr.dark, &mut add, &mut decoded_pixels)?;
            fill_assets(&qr.light, &mut add, &mut decoded_pixels)?;
            stroke_assets(qr.outline.as_ref(), &mut add, &mut decoded_pixels)?;
        }
        for effect in shape
            .effects
            .iter()
            .chain(shape.parked_effects.iter().map(|e| &e.effect))
        {
            if let tracedraw_core::live::Effect::Transparency { mask, .. } = effect {
                fill_assets(mask, &mut add, &mut decoded_pixels)?;
            }
        }
        match &shape.kind {
            ShapeKind::Polygon {
                points,
                complex: None,
                ..
            } if *points > MAX_POLYGON_POINTS => {
                return Err("native polygon exceeds its vertex generation budget".into());
            }
            ShapeKind::Group { children } if children.len() > MAX_SHAPES => {
                return Err("native group exceeds its object count limit".into());
            }
            ShapeKind::ClipFrame { contents, .. } if contents.len() > MAX_SHAPES => {
                return Err("native clip frame exceeds its object count limit".into());
            }
            ShapeKind::Text { spans, para, .. } => {
                if spans.len() > MAX_ARRAY {
                    return Err("native text exceeds its span count limit".into());
                }
                span_assets(spans, &mut add, &mut decoded_pixels)?;
                flow_total = flow_total
                    .checked_add(flow_budget(&para.flow)?)
                    .ok_or("native text flow resource count overflow")?;
                if flow_total > MAX_ARRAY {
                    return Err("native text flows exceed their total resource limit".into());
                }
                if let Some(fill) = &para.frame_fill {
                    fill_assets(fill, &mut add, &mut decoded_pixels)?;
                }
            }
            ShapeKind::Table(table) => {
                if table.cells.len() > MAX_ARRAY {
                    return Err("native table exceeds its cell count limit".into());
                }
                let rows = table.row_heights.len();
                let cols = table.col_widths.len();
                if rows == 0 || cols == 0 || rows.checked_mul(cols).is_none_or(|n| n > MAX_ARRAY) {
                    return Err("native table exceeds its grid generation budget".into());
                }
                for cell in &table.cells {
                    if cell.row_span == 0
                        || cell.col_span == 0
                        || cell
                            .row
                            .checked_add(cell.row_span)
                            .is_none_or(|end| u64::from(end) > rows as u64)
                        || cell
                            .col
                            .checked_add(cell.col_span)
                            .is_none_or(|end| u64::from(end) > cols as u64)
                    {
                        return Err("native table cell span is outside its grid".into());
                    }
                }
                fill_assets(&table.cell_fill, &mut add, &mut decoded_pixels)?;
                stroke_assets(table.border.as_ref(), &mut add, &mut decoded_pixels)?;
                for cell in &table.cells {
                    if let Some(fill) = &cell.fill {
                        fill_assets(fill, &mut add, &mut decoded_pixels)?;
                    }
                    span_assets(&cell.text, &mut add, &mut decoded_pixels)?;
                    for side in tracedraw_core::table::CellSide::ALL {
                        if let tracedraw_core::table::EdgeBorder::Stroke(stroke) =
                            cell.borders.side(side)
                        {
                            stroke_assets(Some(stroke), &mut add, &mut decoded_pixels)?;
                        }
                    }
                }
            }
            _ => {}
        }
        if let ShapeKind::Bitmap {
            width_px,
            height_px,
            png: bytes,
            fx,
            ..
        } = &shape.kind
        {
            add(bytes.len())?;
            png(bytes, *width_px, *height_px, &mut decoded_pixels)?;
            if let Some(fx) = fx {
                add(fx.png.len())?;
                png(&fx.png, fx.width_px, fx.height_px, &mut decoded_pixels)?;
            }
            if let Some(inks) = &shape.picture_inks {
                if inks.inks.is_empty() || inks.inks.len() > 8 || !inks.fits(bytes) {
                    return Err("native bitmap ink planes do not match their preview".into());
                }
                let rows = height_px
                    .checked_mul(
                        u32::try_from(inks.inks.len()).map_err(|_| "native ink count overflow")?,
                    )
                    .ok_or("native ink plane height overflow")?;
                add(inks.png.len())?;
                png_with_budget(
                    &inks.png,
                    *width_px,
                    rows,
                    MAX_PIXELS * inks.inks.len() as u64,
                    &mut decoded_pixels,
                )?;
                if let Some(p) = &inks.profile {
                    add(p.data.len())?;
                    profile(p)?;
                }
                for ink in &inks.inks {
                    match ink {
                        tracedraw_core::picture_inks::PictureInk::Process(id) if *id > 3 => {
                            return Err("native bitmap process ink is invalid".into())
                        }
                        tracedraw_core::picture_inks::PictureInk::Spot(id)
                            if !doc.inks.iter().any(|i| i.id == *id) =>
                        {
                            return Err("native bitmap spot ink is missing".into())
                        }
                        _ => {}
                    }
                }
            }
        } else if shape.picture_inks.is_some() {
            return Err("native ink planes belong to a non-bitmap object".into());
        }
        if let Some(fx) = &shape.raster_fx {
            add(fx.png.len())?;
            if !fx.png.is_empty() {
                png(&fx.png, fx.width_px, fx.height_px, &mut decoded_pixels)?;
            }
        }
        Ok(())
    })?;
    let unique = |values: Vec<u32>| {
        values
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == values.len()
    };
    if !unique(doc.inks.iter().map(|s| s.id).collect())
        || !unique(doc.object_styles.iter().map(|s| s.id).collect())
    {
        return Err("native selection contains ambiguous resource identifiers".into());
    }
    Ok(())
}

/// A small JSON grammar preflight: no Value tree, strings or binary resources
/// are allocated. Grammar/depth/tokens/array/resource/blob budgets are checked
/// before serde sees a single model field. Its stack is an explicit Vec.
fn preflight(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > MAX_JSON_BYTES {
        return Err("native selection exceeds its JSON byte limit".into());
    }
    struct Scan<'a> {
        bytes: &'a [u8],
        at: usize,
        tokens: usize,
        binary: usize,
        shapes: usize,
        shape_depth: usize,
    }
    struct Frame {
        kind: u8,
        state: u8,
        count: usize,
        limit: usize,
        key: String,
        keys: std::collections::BTreeSet<String>,
        profile: bool,
        model_shape: bool,
        start: usize,
    }
    impl Frame {
        fn new(kind: u8, limit: usize) -> Self {
            Self {
                kind,
                state: 0,
                count: 0,
                limit,
                key: String::new(),
                keys: Default::default(),
                profile: false,
                model_shape: false,
                start: 0,
            }
        }
    }
    impl Scan<'_> {
        fn ws(&mut self) {
            while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) {
                self.at += 1;
            }
        }
        fn string(&mut self) -> Result<&[u8], String> {
            let start = self.at;
            if self.bytes.get(self.at) != Some(&b'"') {
                return Err("native JSON expects a string".into());
            }
            self.at += 1;
            loop {
                match self.bytes.get(self.at).copied() {
                    Some(b'"') => {
                        self.at += 1;
                        return self
                            .bytes
                            .get(start..self.at)
                            .ok_or("invalid native JSON string range".into());
                    }
                    Some(b'\\') => {
                        self.at += 2;
                    }
                    Some(0..=31) | None => {
                        return Err("native JSON string is truncated or contains controls".into())
                    }
                    Some(_) => self.at += 1,
                }
            }
        }
    }
    let mut scan = Scan {
        bytes,
        at: 0,
        tokens: 0,
        binary: 0,
        shapes: 0,
        shape_depth: 0,
    };
    let mut stack = vec![Frame::new(0, 1)];
    let mut shape_ranges = Vec::new();
    loop {
        scan.ws();
        let frame = stack.last_mut().ok_or("invalid native JSON parser state")?;
        let key = match (frame.kind, frame.state) {
            (0, 0) => {
                frame.state = 1;
                String::new()
            }
            (0, _) => {
                return if scan.at == bytes.len() {
                    // Object-key order cannot bypass the model-depth guard:
                    // a parent may put its own `kind` after all its children.
                    shape_ranges.sort_unstable_by_key(|&(start, _)| start);
                    let mut ends = Vec::new();
                    for (start, end) in shape_ranges {
                        while ends.last().is_some_and(|&e| e <= start) {
                            ends.pop();
                        }
                        if ends.len() >= 129 {
                            return Err("native JSON exceeds the model object nesting limit".into());
                        }
                        ends.push(end);
                    }
                    Ok(())
                } else {
                    Err("native JSON has trailing data".into())
                };
            }
            (1, 0 | 4) => {
                if scan.bytes.get(scan.at) == Some(&b'}') && frame.state == 0 {
                    scan.at += 1;
                    if let Some(closed) = stack.pop() {
                        if closed.model_shape {
                            scan.shape_depth -= 1;
                            shape_ranges.push((closed.start, scan.at));
                        }
                    }
                    continue;
                }
                let raw = scan.string()?;
                if raw.len() > 258 {
                    return Err("native JSON key exceeds its byte limit".into());
                }
                let key: String = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
                if !frame.keys.insert(key.clone()) {
                    return Err("native JSON has a duplicate object key".into());
                }
                if key == "kind" {
                    scan.shapes += 1;
                    if scan.shapes > MAX_SHAPES {
                        return Err("native JSON exceeds its object count limit".into());
                    }
                }
                frame.key = key;
                frame.state = 1;
                continue;
            }
            (1, 1) => {
                if scan.bytes.get(scan.at) != Some(&b':') {
                    return Err("native JSON expects a colon".into());
                }
                scan.at += 1;
                frame.state = 2;
                continue;
            }
            (1, 2) => {
                frame.state = 3;
                let key = std::mem::take(&mut frame.key);
                if frame.profile && key == "data" {
                    "@profile_data".into()
                } else {
                    key
                }
            }
            (1, 3) => {
                match scan.bytes.get(scan.at) {
                    Some(b'}') => {
                        scan.at += 1;
                        if let Some(closed) = stack.pop() {
                            if closed.model_shape {
                                scan.shape_depth -= 1;
                                shape_ranges.push((closed.start, scan.at));
                            }
                        }
                    }
                    Some(b',') => {
                        scan.at += 1;
                        frame.state = 4;
                    }
                    _ => return Err("native JSON object is truncated".into()),
                }
                continue;
            }
            (2, 0 | 2) => {
                if scan.bytes.get(scan.at) == Some(&b']') && frame.state == 0 {
                    scan.at += 1;
                    stack.pop();
                    continue;
                }
                frame.count += 1;
                if frame.count > frame.limit {
                    return Err("native JSON array exceeds its resource limit".into());
                }
                frame.state = 1;
                String::new()
            }
            (2, 1) => {
                match scan.bytes.get(scan.at) {
                    Some(b']') => {
                        scan.at += 1;
                        stack.pop();
                    }
                    Some(b',') => {
                        scan.at += 1;
                        frame.state = 2;
                    }
                    _ => return Err("native JSON array is truncated".into()),
                }
                continue;
            }
            _ => return Err("invalid native JSON parser frame".into()),
        };
        scan.ws();
        scan.tokens += 1;
        if stack.len() > MAX_JSON_DEPTH || scan.tokens > MAX_JSON_TOKENS {
            return Err("native JSON exceeds its depth or token limit".into());
        }
        match scan.bytes.get(scan.at).copied() {
            Some(b'{') => {
                scan.at += 1;
                if key == "kind" {
                    if let Some(parent) = stack.last_mut() {
                        if !parent.model_shape {
                            parent.model_shape = true;
                            scan.shape_depth += 1;
                            // The model counts edges from a depth-zero root.
                            if scan.shape_depth > 129 {
                                return Err(
                                    "native JSON exceeds the model object nesting limit".into()
                                );
                            }
                        }
                    }
                }
                let mut frame = Frame::new(1, MAX_ARRAY);
                frame.start = scan.at - 1;
                frame.profile = matches!(key.as_str(), "cmyk_profile" | "profile");
                stack.push(frame);
            }
            Some(b'[') => {
                scan.at += 1;
                let limit =
                    if matches!(key.as_str(), "symbols" | "object_styles" | "inks" | "fonts") {
                        MAX_RESOURCES
                    } else {
                        MAX_ARRAY
                    };
                stack.push(Frame::new(2, limit));
            }
            Some(b'"') => {
                let raw = scan.string()?;
                if matches!(key.as_str(), "png" | "data" | "@profile_data") {
                    // Canonical base64 has no escapes. For escaped or malformed
                    // text the same calculation stays a conservative bound.
                    let padding = raw
                        .iter()
                        .rev()
                        .skip(1)
                        .take_while(|&&b| b == b'=')
                        .take(2)
                        .count();
                    let estimate = raw
                        .len()
                        .saturating_sub(2)
                        .div_ceil(4)
                        .saturating_mul(3)
                        .saturating_sub(padding);
                    if key == "@profile_data" && estimate > MAX_PROFILE_BYTES {
                        return Err("native JSON ICC profile exceeds its byte limit".into());
                    }
                    scan.binary = scan
                        .binary
                        .checked_add(estimate)
                        .ok_or("native binary byte count overflow")?;
                    if scan.binary > MAX_BINARY_BYTES {
                        return Err("native JSON binary resources exceed their byte limit".into());
                    }
                }
            }
            Some(_) => {
                let start = scan.at;
                while scan
                    .bytes
                    .get(scan.at)
                    .is_some_and(|b| !b.is_ascii_whitespace() && !matches!(b, b',' | b'}' | b']'))
                {
                    scan.at += 1;
                }
                let raw = scan
                    .bytes
                    .get(start..scan.at)
                    .ok_or("invalid native JSON token range")?;
                if !matches!(raw, b"null" | b"true" | b"false") {
                    let text = std::str::from_utf8(raw).map_err(|e| e.to_string())?;
                    if text.is_empty() || !text.parse::<f64>().is_ok_and(f64::is_finite) {
                        return Err("native JSON contains a non-finite or invalid number".into());
                    }
                }
            }
            None => return Err("native JSON is truncated".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Size;
    use tracedraw_core::{document::TextSpan, Shape, ShapeId};

    fn payload() -> NativeClipboardPayload {
        let mut doc = Document::new("Source", Size::new(40.0, 30.0));
        let shape = Shape::new(
            ShapeId(1),
            ShapeKind::Rect {
                rect: Rect::new(1.0, 2.0, 8.0, 9.0),
                radius: 0.0,
                corners: None,
            },
        );
        doc.pages[0].layers[0].shapes.push(shape.clone());
        NativeClipboardPayload::capture(&doc, &[shape], 123).unwrap()
    }

    fn unvalidated_svg(mut payload: NativeClipboardPayload) -> String {
        let supplements = capture_supplements(&mut payload.document).unwrap();
        let bytes = serde_json::to_vec(&Wire {
            version: 1,
            payload,
            supplements,
        })
        .unwrap();
        let text = base64::engine::general_purpose::STANDARD.encode(bytes);
        format!("<svg><metadata><cc:selection xmlns:cc=\"{NAMESPACE}\" version=\"1\" encoding=\"base64-json\">{text}</cc:selection></metadata></svg>")
    }

    fn measured_json_depth(bytes: &[u8]) -> usize {
        let (mut depth, mut maximum, mut quoted, mut escaped) = (0usize, 0usize, false, false);
        for &byte in bytes {
            if quoted {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    quoted = false;
                }
            } else {
                match byte {
                    b'"' => quoted = true,
                    b'{' | b'[' => {
                        depth += 1;
                        maximum = maximum.max(depth);
                    }
                    b'}' | b']' => depth = depth.saturating_sub(1),
                    _ => {}
                }
            }
        }
        maximum
    }

    #[test]
    fn aggregate_pixels_are_reserved_before_png_decoder_runs() {
        let image = image::RgbaImage::from_raw(1, 1, vec![1, 3, 5, 0]).unwrap();
        let mut data = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut data, image::ImageFormat::Png)
            .unwrap();
        let mut decoded = MAX_DECODED_PIXELS - 1;
        png(data.get_ref(), 1, 1, &mut decoded).unwrap();
        assert_eq!(decoded, MAX_DECODED_PIXELS);
        // This would fail as malformed PNG if the decoder were started.
        assert!(png(b"not a PNG", 1, 1, &mut decoded)
            .unwrap_err()
            .contains("aggregate decoded pixel budget"));
        let mut decoded = u64::MAX;
        assert!(png(b"not a PNG", 1, 1, &mut decoded)
            .unwrap_err()
            .contains("overflow"));

        // The aggregate does not divide the per-image budget by the channels.
        // Empty input fails immediately; no large image is created or decoded.
        let mut decoded = 0;
        assert!(png(b"", 16_384, 1_024, &mut decoded)
            .unwrap_err()
            .contains("invalid native bitmap PNG"));
        assert!(
            png_with_budget(b"", 16_384, 8_192, MAX_PIXELS * 8, &mut decoded)
                .unwrap_err()
                .contains("invalid native bitmap PNG")
        );
        assert_eq!(decoded, MAX_PIXELS * 9);
    }

    #[test]
    fn scalar_generation_limits_reject_tiny_wire_before_geometry() {
        let mut p = payload();
        p.document.pages[0].layers[0].shapes[0].kind = ShapeKind::Polygon {
            rect: Rect::new(1.0, 2.0, 8.0, 9.0),
            points: 5,
            sharpness: 0.0,
            complex: None,
            twist: 0.0,
        };
        assert!(validate_native(&p).is_ok());
        if let ShapeKind::Polygon { points, .. } = &mut p.document.pages[0].layers[0].shapes[0].kind
        {
            *points = u32::MAX;
        }
        let svg = unvalidated_svg(p);
        assert!(svg.len() < 16_384);
        assert!(decode_svg(&svg).unwrap_err().contains("polygon"));

        let mut p = payload();
        let mut table =
            tracedraw_core::document::Table::new(Rect::new(1.0, 2.0, 8.0, 9.0), 1, 1, None);
        p.document.pages[0].layers[0].shapes[0].kind = ShapeKind::Table(table.clone());
        assert!(validate_native(&p).is_ok());
        table.row_heights = vec![1.0; 1_000];
        table.col_widths = vec![1.0; 1_000];
        p.document.pages[0].layers[0].shapes[0].kind = ShapeKind::Table(table);
        let svg = unvalidated_svg(p);
        assert!(svg.len() < 32_768);
        assert!(decode_svg(&svg).unwrap_err().contains("table"));

        let mut p = payload();
        let mut table =
            tracedraw_core::document::Table::new(Rect::new(1.0, 2.0, 8.0, 9.0), 1, 1, None);
        table.cells[0].row = u32::MAX;
        table.cells[0].row_span = 2;
        p.document.pages[0].layers[0].shapes[0].kind = ShapeKind::Table(table);
        assert!(decode_svg(&unvalidated_svg(p))
            .unwrap_err()
            .contains("span"));
    }

    #[test]
    fn explosive_symbol_dag_is_rejected_on_receive_without_rendering() {
        let mut p = payload();
        let mut leaf = p.document.pages[0].layers[0].shapes[0].clone();
        leaf.id = ShapeId(2);
        p.document.symbols.push(tracedraw_core::Symbol {
            name: "Leaf".into(),
            shape: leaf,
        });
        for index in 0..26 {
            let previous = p.document.symbols.len() - 1;
            let id = 100 + index * 3;
            p.document.symbols.push(tracedraw_core::Symbol {
                name: format!("Double{index}"),
                shape: Shape::new(
                    ShapeId(id),
                    ShapeKind::Group {
                        children: vec![
                            Shape::new(
                                ShapeId(id + 1),
                                ShapeKind::SymbolInstance { index: previous },
                            ),
                            Shape::new(
                                ShapeId(id + 2),
                                ShapeKind::SymbolInstance { index: previous },
                            ),
                        ],
                    },
                ),
            });
        }
        let last = p.document.symbols.len() - 1;
        p.document.pages[0].layers[0].shapes[0].kind = ShapeKind::SymbolInstance { index: last };
        let svg = unvalidated_svg(p);
        assert!(svg.len() < 128 * 1024);
        assert!(decode_svg(&svg)
            .unwrap_err()
            .contains("expanded work budget"));
    }

    #[test]
    fn native_svg_roundtrip_is_compact_and_versioned() {
        let payload = payload();
        for svg in [
            "<svg xmlns='http://www.w3.org/2000/svg'><rect/></svg>",
            "<s:svg xmlns:s='http://www.w3.org/2000/svg'/>",
        ] {
            let wire = encode_svg(&payload, svg).unwrap();
            assert!(looks_native(&wire));
            let decoded = decode_svg(&wire).unwrap().unwrap();
            assert_eq!(decoded.document, payload.document);
            assert_eq!(decoded.source_instance, 123);
            assert!(encode_svg(&payload, &wire).is_err());
        }
        assert!(decode_svg("<svg><rect/></svg>").unwrap().is_none());
        assert!(decode_svg("ordinary text").unwrap().is_none());
        assert!(!looks_native("urn:craft-corel:selection"));
        assert!(decode_svg("urn:craft-corel:selection").unwrap().is_none());
        assert!(looks_native(
            "\u{feff} \n<svg xmlns:x='urn:craft-corel:selection'"
        ));
    }

    #[test]
    fn own_unknown_duplicate_truncated_and_invalid_blocks_never_fallback() {
        let wire = encode_svg(&payload(), "<svg/>").unwrap();
        assert!(decode_svg(&wire.replace("version=\"1\"", "version=\"2\"")).is_err());
        assert!(decode_svg(&wire.replace(
            "</metadata>",
            "</metadata><metadata><x:selection xmlns:x='urn:craft-corel:selection'/></metadata>"
        ))
        .is_err());
        assert!(decode_svg(&wire[..wire.len() - 7]).is_err());
        assert!(decode_svg("<svg><metadata><x:selection xmlns:x='urn:craft-corel:selection' version='1' encoding='base64-json'>!</x:selection></metadata></svg>").is_err());
        assert!(looks_native("<svg xmlns:x='urn:craft-corel:selection'"));
        let escaped = "<svg xmlns:x='&#117;rn:craft-corel:selecti&#x6f;n'";
        assert!(looks_native(escaped));
        assert!(decode_svg(escaped).is_err());
        assert!(looks_native(
            "<svg xmlns:x='&#00000000000000000117;rn:craft-corel:selection'"
        ));
    }

    #[test]
    fn text_skipped_state_has_typed_supplements() {
        let mut p = payload();
        let mut span = TextSpan::new("Visible", "Family", 12.0);
        span.layout_only = true;
        let mut para = tracedraw_core::document::ParagraphStyle::default();
        para.flow = FrameFlow::new(
            vec![WrapRegion {
                wrap: TextWrap::default(),
                bbox: Rect::new(1.0, 2.0, 3.0, 4.0),
                segments: vec![(Point::new(1.0, 2.0), Point::new(3.0, 4.0))],
            }],
            Some((2.0, 4.0)),
        );
        p.document.pages[0].layers[0].shapes[0].kind = ShapeKind::Text {
            spans: vec![span],
            origin: Point::ZERO,
            frame: Some(Size::new(8.0, 9.0)),
            align: Default::default(),
            para,
            on_path: None,
        };
        let raw = serde_json::to_vec(&p).unwrap();
        let ordinary: NativeClipboardPayload = serde_json::from_slice(&raw).unwrap();
        assert_ne!(ordinary.document, p.document);
        let wire = encode_svg(&p, "<svg/>").unwrap();
        assert_eq!(decode_svg(&wire).unwrap().unwrap().document, p.document);
    }

    #[test]
    fn preflight_rejects_limits_before_model_deserialization() {
        let deep = format!(
            "{}0{}",
            "[".repeat(MAX_JSON_DEPTH + 2),
            "]".repeat(MAX_JSON_DEPTH + 2)
        );
        assert!(preflight(deep.as_bytes()).is_err());
        let shapes = format!(
            "{}{{\"kind\":{{\"type\":\"rect\"}}}}{}",
            "{\"kind\":{\"type\":\"group\",\"children\":[".repeat(130),
            "]}}".repeat(130)
        );
        assert!(preflight(shapes.as_bytes()).is_err());
        let reordered = format!(
            "{}{{\"kind\":{{}}}}{}",
            "{\"x\":".repeat(130),
            ",\"kind\":{}}".repeat(130)
        );
        assert!(preflight(reordered.as_bytes()).is_err());
        assert!(preflight(br#"{"version":1,"version":2}"#).is_err());
        assert!(preflight(br#"{"x":1e999}"#).is_err());
        assert!(preflight(br#"{"x":"unfinished}"#).is_err());
        let many = format!(
            "{{\"fonts\":[{}]}}",
            std::iter::repeat_n("null", MAX_RESOURCES + 1)
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(preflight(many.as_bytes()).is_err());
        for invalid in ["[1,]", "{\"x\":1,}", "{\"x\" 1}", "[", "}", "01", "[null]x"] {
            // Numeric JSON grammar is also checked by serde before its model
            // visitor, so this assertion includes that final bounded stage.
            assert!(
                deserialize::<serde_json::Value>(invalid.as_bytes()).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn invalid_icc_and_foreign_resources_are_rejected() {
        let mut p = payload();
        p.document.cmyk_profile = Some(DocumentProfile::new("Damaged", vec![0; 128]));
        assert!(encode_svg(&p, "<svg/>").is_err());
        p.document.cmyk_profile = None;
        p.document
            .palette
            .push(tracedraw_core::Color::rgb8(1, 3, 5));
        assert!(validate_native(&p).is_err());
        p.document.palette.clear();
        p.document.time_tracker.active = Some(0);
        assert!(validate_native(&p).is_err());
        p.document.time_tracker.active = None;
        p.document.pages[0].layers[0].shapes[0].kind = ShapeKind::SymbolInstance { index: 6 };
        assert!(encode_svg(&p, "<svg/>").is_err());
    }

    #[test]
    fn profile_font_preview_ink_planes_and_fingerprint_are_exact_bytes() {
        let mut p = payload();
        let encode = |image: image::DynamicImage| {
            let mut out = Cursor::new(Vec::new());
            image.write_to(&mut out, image::ImageFormat::Png).unwrap();
            tracedraw_core::SharedBytes::from(out.into_inner())
        };
        let pixels = image::RgbaImage::from_raw(
            2,
            2,
            vec![1, 3, 5, 0, 7, 9, 11, 1, 13, 15, 17, 127, 19, 21, 23, 255],
        )
        .unwrap();
        let preview = encode(image::DynamicImage::ImageRgba8(pixels));
        let planes = encode(image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(2, 8, (1..=16).collect()).unwrap(),
        ));
        let profile = DocumentProfile::new(
            "Own CMYK A",
            tracedraw_core::icc::test_profiles::intent_cmyk_profile(false),
        );
        p.document.cmyk_profile = Some(profile.clone());
        p.document.metadata.cmyk_profile = "Explicit real working profile".into();
        let shape = &mut p.document.pages[0].layers[0].shapes[0];
        let mut inks = tracedraw_core::picture_inks::PictureInks::cmyk(&preview, planes);
        inks.profile = Some(profile);
        shape.picture_inks = Some(Box::new(inks));
        shape.cmyk_bitmap = true;
        shape.kind = ShapeKind::Bitmap {
            rect: Rect::new(1.0, 2.0, 3.0, 4.0),
            width_px: 2,
            height_px: 2,
            png: preview,
            fx: None,
        };
        let mut text = Shape::new(
            ShapeId(2),
            ShapeKind::Text {
                spans: vec![TextSpan::new("Árbol, niño y piñón Ñ", "TestSymbol", 12.0)],
                origin: Point::ZERO,
                frame: None,
                align: Default::default(),
                para: Default::default(),
                on_path: None,
            },
        );
        text.link = Some("https://example.invalid/selection".into());
        p.document.pages[0].layers[0].shapes.push(text);
        p.document.fonts.push(tracedraw_core::DocumentFont::new(
            "TestSymbol",
            include_bytes!("../../../crates/text/testdata/TestSymbol.ttf").to_vec(),
        ));
        let wire = encode_svg(&p, "<svg/>").unwrap();
        let decoded = decode_svg(&wire).unwrap().unwrap();
        assert_eq!(decoded.document, p.document);
    }

    #[test]
    fn nested_groups_preserve_existing_model_depth() {
        for groups in [20, 64, 128] {
            eprintln!("wire_depth groups={groups} stage=construct");
            let mut p = payload();
            let mut shape = p.document.pages[0].layers[0].shapes.remove(0);
            for depth in 0..groups {
                shape = Shape::new(
                    ShapeId(depth + 2),
                    ShapeKind::Group {
                        children: vec![shape],
                    },
                );
            }
            p.document.pages[0].layers[0].shapes.push(shape);
            eprintln!("wire_depth groups={groups} stage=encode");
            let wire = encode_svg(&p, "<svg/>").unwrap();
            let parsed = xml(&wire).unwrap();
            let encoded = own_block(&parsed).unwrap().unwrap().text().unwrap();
            let json = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap();
            eprintln!(
                "wire_depth groups={groups} stage=decode svg_bytes={} json_depth={}",
                wire.len(),
                measured_json_depth(&json)
            );
            let decoded = decode_svg(&wire).unwrap().unwrap();
            eprintln!("wire_depth groups={groups} stage=outside_clone");
            let outside = decoded.clone();
            eprintln!("wire_depth groups={groups} stage=outside_equality");
            assert_eq!(
                decoded.document.pages[0].layers[0].shapes,
                p.document.pages[0].layers[0].shapes
            );
            eprintln!("wire_depth groups={groups} stage=outside_drop");
            drop(outside);
            drop(decoded);
            drop(p);
            eprintln!("wire_depth groups={groups} stage=complete");
        }
        let mut p = payload();
        let mut shape = p.document.pages[0].layers[0].shapes.remove(0);
        for depth in 0..129 {
            shape = Shape::new(
                ShapeId(depth + 2),
                ShapeKind::Group {
                    children: vec![shape],
                },
            );
        }
        p.document.pages[0].layers[0].shapes.push(shape);
        assert!(encode_svg(&p, "<svg/>").is_err());
    }

    #[test]
    fn real_svg_preview_roundtrips_and_recopies_128_model_groups() {
        let mut p = payload();
        let mut shape = p.document.pages[0].layers[0].shapes.remove(0);
        for depth in 0..128 {
            shape = Shape::new(
                ShapeId(depth + 2),
                ShapeKind::Group {
                    children: vec![shape],
                },
            );
        }
        p.document.pages[0].layers[0].shapes.push(shape);
        let preferences = crate::settings::ColorPrefs::default();
        let preview = crate::clipboard_preview::render(&p, &preferences).unwrap();
        let parsed = xml(&preview).unwrap();
        let xml_depth = parsed
            .descendants()
            .map(|node| node.ancestors().count())
            .max()
            .unwrap();
        assert!(
            xml_depth > 128,
            "the real SVG must include wrappers beyond model depth"
        );
        assert!(xml_depth <= MAX_XML_DEPTH + 1);
        let copied = encode_svg(&p, &preview).unwrap();
        let pasted = decode_svg(&copied).unwrap().unwrap();
        assert_eq!(pasted.document, p.document);
        let next_preview = crate::clipboard_preview::render(&pasted, &preferences).unwrap();
        let recopied = encode_svg(&pasted, &next_preview).unwrap();
        assert_eq!(decode_svg(&recopied).unwrap().unwrap().document, p.document);
        eprintln!(
            "wire_real_svg groups=128 xml_ancestors={xml_depth} copy_bytes={} recopy_bytes={}",
            copied.len(),
            recopied.len()
        );

        let excessive = format!(
            "<svg>{}{}</svg>",
            "<g>".repeat(MAX_XML_DEPTH + 1),
            "</g>".repeat(MAX_XML_DEPTH + 1)
        );
        assert!(
            xml(&excessive).is_err(),
            "the independent XML budget remains enforced"
        );
    }

    #[test]
    fn finite_coordinates_roundtrip_bit_for_bit() {
        let mut p = payload();
        let values = [
            0.84551240822557006,
            f64::MIN_POSITIVE,
            1.2345678901234567e-100,
            1.2345678901234567e100,
            -0.0,
        ];
        for value in values {
            p.document.pages[0].layers[0].shapes[0].transform =
                tracedraw_core::geometry::Affine::translate((value, -value));
            let wire = encode_svg(&p, "<svg/>").unwrap();
            let back = decode_svg(&wire).unwrap().unwrap();
            let a = p.document.pages[0].layers[0].shapes[0]
                .transform
                .as_coeffs();
            let b = back.document.pages[0].layers[0].shapes[0]
                .transform
                .as_coeffs();
            assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits));
        }
    }

    #[test]
    fn unknown_model_fields_cannot_be_silently_discarded() {
        let wire = encode_svg(&payload(), "<svg/>").unwrap();
        let parsed = xml(&wire).unwrap();
        let block = own_block(&parsed).unwrap().unwrap();
        let encoded = block.text().unwrap();
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        let mut value: serde_json::Value = deserialize(&bytes).unwrap();
        value["payload"]["document"]["future_colour_resource"] = true.into();
        let changed =
            base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&value).unwrap());
        assert!(decode_svg(&wire.replace(encoded, &changed)).is_err());
    }

    #[test]
    fn wire_cost_512_cmyka_is_measured_without_rendering() {
        let mut p = payload();
        let mut rgba = Vec::with_capacity(512 * 512 * 4);
        let mut planes = Vec::with_capacity(512 * 512 * 4);
        let mut seed = 0x53a9_27d1_u32;
        for index in 0..512 * 512 {
            for _ in 0..3 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                rgba.push((seed as u8) | 1);
            }
            rgba.push([0, 1, 127, 255][index % 4]);
            for _ in 0..4 {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                planes.push(seed as u8);
            }
        }
        let encode = |image: image::DynamicImage| {
            let mut out = Cursor::new(Vec::new());
            image.write_to(&mut out, image::ImageFormat::Png).unwrap();
            tracedraw_core::SharedBytes::from(out.into_inner())
        };
        let preview = encode(image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(512, 512, rgba).unwrap(),
        ));
        let planes = encode(image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(512, 2048, planes).unwrap(),
        ));
        let profile = DocumentProfile::new(
            "Own benchmark A",
            tracedraw_core::icc::test_profiles::intent_cmyk_profile(false),
        );
        p.document.cmyk_profile = Some(profile.clone());
        let shape = &mut p.document.pages[0].layers[0].shapes[0];
        let mut inks = tracedraw_core::picture_inks::PictureInks::cmyk(&preview, planes);
        inks.profile = Some(profile);
        shape.picture_inks = Some(Box::new(inks));
        shape.cmyk_bitmap = true;
        shape.kind = ShapeKind::Bitmap {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            width_px: 512,
            height_px: 512,
            png: preview,
            fx: None,
        };
        for pass in 0..3 {
            let start = std::time::Instant::now();
            let wire = encode_svg(&p, "<svg/>").unwrap();
            let encoded = start.elapsed();
            let start = std::time::Instant::now();
            let back = decode_svg(&wire).unwrap().unwrap();
            let decoded = start.elapsed();
            assert_eq!(back.document, p.document);
            println!(
                "wire_512_cmyka pass={pass} svg_bytes={} encode_ms={:.3} decode_ms={:.3}",
                wire.len(),
                encoded.as_secs_f64() * 1000.0,
                decoded.as_secs_f64() * 1000.0
            );
        }
    }
}
