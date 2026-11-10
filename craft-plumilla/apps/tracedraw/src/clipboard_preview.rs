//! Selection SVG preparation, with source-local colour management and explicit
//! structural expansion budgets. No application mutation or OS clipboard.

use crate::clipboard::NativeClipboardPayload;
use crate::settings::ColorPrefs;
use std::io::Write;
use tracedraw_core::{Document, Shape, ShapeKind};

const MAX_EXPANDED_NODES: u64 = 20_000;
const MAX_EXPANDED_DEPTH: usize = 128;
const MAX_TEXT_CHARACTERS: u64 = 65_536;
const MAX_PROFILE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Work {
    pub expanded_nodes: u64,
    pub estimated_svg_bytes: u64,
    pub expanded_depth: usize,
}

#[derive(Default)]
struct Physical {
    nodes: u64,
    bytes: u64,
    depth: usize,
    edges: Vec<(usize, usize, u64)>,
}

fn multiply(a: u64, b: u64) -> Result<u64, String> {
    a.checked_mul(b)
        .ok_or_else(|| "clipboard preview expansion budget overflow".into())
}

fn add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b)
        .ok_or_else(|| "clipboard preview expansion budget overflow".into())
}

fn checked(work: Work) -> Result<Work, String> {
    if work.expanded_nodes > MAX_EXPANDED_NODES
        || work.estimated_svg_bytes > crate::clipboard_wire::MAX_SVG_BYTES as u64
        || work.expanded_depth > MAX_EXPANDED_DEPTH
    {
        return Err("clipboard preview exceeds its expanded work budget".into());
    }
    Ok(work)
}

struct Count(u64);
impl Write for Count {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("preview byte count overflow"))?;
        if self.0 > crate::clipboard_wire::MAX_JSON_BYTES as u64 {
            return Err(std::io::Error::other(
                "preview source exceeds its byte budget",
            ));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn physical(root: &Shape) -> Result<Physical, String> {
    let mut out = Physical::default();
    let mut text = 0;
    let mut path: Vec<(usize, u64)> = Vec::new();
    let mut maximum_weight = 1;
    tracedraw_core::color_convert::try_visit_shape_color_tree(root, &mut |shape, depth| {
        out.depth = out.depth.max(depth);
        while path.last().is_some_and(|&(d, _)| d >= depth) {
            path.pop();
        }
        let parent_weight = path.last().map_or(1, |&(_, weight)| weight);
        let mut local_text = 0;
        let mut factor = 1u64;
        factor = add(
            factor,
            (shape.extra_fills.len() + shape.extra_strokes.len()) as u64,
        )?;
        match &shape.kind {
            ShapeKind::Text { spans, .. } => {
                local_text = spans.iter().map(|s| s.text.chars().count() as u64).sum();
                // text_parts repeats the laid-out text for its character looks.
                factor = multiply(factor, (spans.len() as u64).max(1))?;
            }
            ShapeKind::Table(table) => {
                factor = multiply(factor, add(multiply(table.cells.len() as u64, 6)?, 1)?)?;
                local_text = table
                    .cells
                    .iter()
                    .flat_map(|c| &c.text)
                    .map(|s| s.text.chars().count() as u64)
                    .sum();
            }
            ShapeKind::ClipFrame { .. } => factor = multiply(factor, 3)?,
            ShapeKind::Path { closed: false, .. }
                if !matches!(shape.fill, tracedraw_core::Fill::None) =>
            {
                factor = multiply(factor, 2)?
            }
            _ => {}
        }
        for effect in &shape.effects {
            let copies = match effect {
                tracedraw_core::live::Effect::Contour {
                    steps, to_center, ..
                } => {
                    if *to_center {
                        999
                    } else {
                        u64::from((*steps).clamp(1, 999))
                    }
                }
                tracedraw_core::live::Effect::Blend { steps, options, .. } => {
                    if options.spacing.is_some() {
                        1000
                    } else {
                        u64::from((*steps).clamp(1, 999)) + 1
                    }
                }
                tracedraw_core::live::Effect::Symmetry { lines, .. } => {
                    u64::from((*lines).clamp(1, 12))
                }
                tracedraw_core::live::Effect::Bevel { level, .. } => {
                    2 * u64::from((*level).clamp(1, 8))
                }
                tracedraw_core::live::Effect::Extrude { .. } => 64,
                _ => 0,
            };
            factor = add(factor, copies)?;
        }
        let weight = multiply(parent_weight, factor)?;
        out.nodes = add(out.nodes, weight)?;
        text = add(text, multiply(local_text, weight)?)?;
        maximum_weight = maximum_weight.max(weight);
        path.push((depth, weight));
        if let ShapeKind::SymbolInstance { index } = &shape.kind {
            out.edges.push((*index, depth, weight));
        }
        if out.nodes > MAX_EXPANDED_NODES || text > MAX_TEXT_CHARACTERS {
            return Err("clipboard preview exceeds its physical expansion budget".into());
        }
        Ok(())
    })?;
    let mut count = Count(0);
    serde_json::to_writer(&mut count, root).map_err(|e| e.to_string())?;
    out.bytes = add(
        multiply(multiply(count.0, 8)?, maximum_weight)?,
        add(multiply(out.nodes, 4096)?, multiply(text, 4096)?)?,
    )?;
    Ok(out)
}

fn combine(base: &Physical, known: &[Option<Work>]) -> Result<Work, String> {
    let mut value = Work {
        expanded_nodes: base.nodes,
        estimated_svg_bytes: base.bytes,
        expanded_depth: base.depth,
    };
    for &(target, depth, weight) in &base.edges {
        let child = known
            .get(target)
            .and_then(|v| *v)
            .ok_or("clipboard preview has an unresolved symbol dependency")?;
        value.expanded_nodes = add(
            value.expanded_nodes,
            multiply(child.expanded_nodes, weight)?,
        )?;
        value.estimated_svg_bytes = add(
            value.estimated_svg_bytes,
            multiply(child.estimated_svg_bytes, weight)?,
        )?;
        value.expanded_depth = value.expanded_depth.max(
            depth
                .checked_add(1)
                .and_then(|d| d.checked_add(child.expanded_depth))
                .ok_or("clipboard preview depth overflow")?,
        );
    }
    checked(value)
}

/// Exact reference-DAG multiplicity and virtual height; conservative estimated
/// geometry/SVG work for effects, text, table cells and repeated appearance.
/// This is a pre-render guard, not a proof of arbitrary font geometry size.
pub(crate) fn budget(doc: &Document) -> Result<Work, String> {
    if doc.symbols.len() > crate::clipboard_wire::MAX_RESOURCES {
        return Err("clipboard preview has too many symbol definitions".into());
    }
    let definitions = doc
        .symbols
        .iter()
        .map(|s| physical(&s.shape))
        .collect::<Result<Vec<_>, _>>()?;
    let mut states = vec![0u8; definitions.len()];
    let mut known = vec![None; definitions.len()];
    for start in 0..definitions.len() {
        if states.get(start) == Some(&2) {
            continue;
        }
        let mut pending = vec![(start, 0usize)];
        while let Some((node, edge)) = pending.last_mut() {
            let current = *node;
            let descriptor = definitions
                .get(current)
                .ok_or("clipboard preview symbol is missing")?;
            if let Some(state) = states.get_mut(current) {
                *state = 1;
            }
            if let Some(&(target, _, _)) = descriptor.edges.get(*edge) {
                *edge += 1;
                match states.get(target) {
                    Some(0) => pending.push((target, 0)),
                    Some(1) => return Err("clipboard preview contains a symbol cycle".into()),
                    Some(2) => {}
                    _ => return Err("clipboard preview symbol reference is missing".into()),
                }
            } else {
                let work = combine(descriptor, &known)?;
                *known
                    .get_mut(current)
                    .ok_or("clipboard preview result slot is missing")? = Some(work);
                *states
                    .get_mut(current)
                    .ok_or("clipboard preview state slot is missing")? = 2;
                pending.pop();
            }
        }
    }
    let mut total = Work::default();
    for layer in doc.all_layers() {
        for root in &layer.shapes {
            let root = combine(&physical(root)?, &known)?;
            total.expanded_nodes = add(total.expanded_nodes, root.expanded_nodes)?;
            total.estimated_svg_bytes = add(total.estimated_svg_bytes, root.estimated_svg_bytes)?;
            total.expanded_depth = total.expanded_depth.max(root.expanded_depth);
            checked(total)?;
        }
    }
    checked(total)
}

fn read_rgb_profile(path: &str) -> Result<Vec<u8>, String> {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        use std::io::Read;
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        file.take((MAX_PROFILE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("clipboard display profile exceeds its byte budget".into());
        }
        Ok(bytes)
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android"))]
    {
        let bytes = crate::files::read(path).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("clipboard display profile exceeds its byte budget".into());
        }
        Ok(bytes)
    }
}

/// May read the explicitly configured RGB profile, but changes no application,
/// document, global colour engine or operating-system clipboard state.
pub(crate) fn render(
    native: &NativeClipboardPayload,
    color: &ColorPrefs,
) -> Result<String, String> {
    crate::clipboard_wire::work_stack(|| render_inner(native, color))
}

fn render_inner(native: &NativeClipboardPayload, color: &ColorPrefs) -> Result<String, String> {
    use std::rc::Rc;
    use tracedraw_core::color::{engine, IccEngine};
    use tracedraw_core::icc::{ColorSpace, Intent, Profile};
    use tracedraw_io::picture_profile::{self, Destination};

    crate::clipboard_wire::validate_native(native)?;
    let mut preview = native.document.clone();
    let inherited = native.document.cmyk_profile.clone();
    let intent = Intent::from_name(&color.intent).unwrap_or_default();
    let options = picture_profile::Options {
        intent,
        black_point_compensation: color.black_point_compensation,
        preserve_pure_black: color.preserve_pure_black,
    };
    tracedraw_core::color_convert::try_map_document_shape_tree(&mut preview, &mut |shape| {
        let own = shape
            .picture_inks
            .as_ref()
            .and_then(|inks| inks.profile.as_ref())
            .cloned();
        // The screen draws stored RGB PNG bytes, not the retained ink planes.
        // Builtin vector colours do not reinterpret an untagged legacy bitmap
        // through the document's inherited ICC merely for this fallback.
        if color.builtin_engine && own.is_none() {
            return Ok(());
        }
        let target = own
            .as_ref()
            .or(inherited.as_ref())
            .map(Destination::Icc)
            .unwrap_or(Destination::Builtin);
        if let Some(updated) =
            picture_profile::reprofile_to(shape, inherited.as_ref(), target, true, options)
                .map_err(|e| e.to_string())?
        {
            shape.kind = updated.kind;
            shape.picture_inks = updated.picture_inks;
        }
        Ok(())
    })?;
    let parse = |bytes: &[u8], space| -> Result<Profile, String> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err("clipboard display profile exceeds its byte budget".into());
        }
        let profile = Profile::parse(bytes).map_err(|e| e.to_string())?;
        if profile.color_space() != space {
            return Err("clipboard display profile has the wrong colour space".into());
        }
        Ok(profile)
    };
    let cmyk = inherited
        .as_ref()
        .map(|p| parse(&p.data, ColorSpace::Cmyk))
        .transpose()?;
    let rgb = if color.rgb_profile_path.is_empty() || color.builtin_engine {
        None
    } else {
        Some(parse(
            &read_rgb_profile(&color.rgb_profile_path)?,
            ColorSpace::Rgb,
        )?)
    };
    let has_cmyk = cmyk.is_some() && !color.builtin_engine;
    let has_rgb = rgb.is_some();
    let pure = if color.builtin_engine || (!has_cmyk && !has_rgb) {
        None
    } else {
        Some(IccEngine::new(
            rgb,
            cmyk,
            intent,
            color.black_point_compensation,
        ))
    };
    let failed = Rc::new(std::cell::Cell::new(false));
    let conversion_failed = Rc::clone(&failed);
    let converter: engine::RgbConverter = Rc::new(move |colour| {
        let colour = colour.process();
        let managed = pure.as_ref().and_then(|e| e.to_rgb_f32(colour));
        if managed.is_none()
            && (matches!(colour, tracedraw_core::Color::Cmyk { .. }) && has_cmyk
                || matches!(colour, tracedraw_core::Color::Rgb { .. }) && has_rgb)
        {
            conversion_failed.set(true);
        }
        let rgb = managed.unwrap_or_else(|| colour.to_rgb_f32_builtin());
        if !rgb.iter().all(|v| v.is_finite()) {
            conversion_failed.set(true);
        }
        rgb
    });
    let svg = engine::with_rgb_converter(converter, || {
        tracedraw_io::svg::page_to_svg_limited(&preview, 0, crate::clipboard_wire::MAX_SVG_BYTES)
    })?;
    if failed.get() {
        return Err("clipboard preview colour conversion failed".into());
    }
    if svg.len() > crate::clipboard_wire::MAX_SVG_BYTES {
        return Err("clipboard SVG exceeds its byte budget".into());
    }
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::{Rect, Size};
    use tracedraw_core::{ShapeId, Symbol};

    fn rect() -> Shape {
        Shape::new(
            ShapeId(1),
            ShapeKind::Rect {
                rect: Rect::new(1.0, 2.0, 8.0, 9.0),
                radius: 0.0,
                corners: None,
            },
        )
    }
    fn reference(index: usize) -> Shape {
        Shape::new(
            ShapeId(100 + index as u64),
            ShapeKind::SymbolInstance { index },
        )
    }

    #[test]
    fn virtual_depth_counts_definition_contents_not_just_graph_edges() {
        let mut doc = Document::new("Clipboard", Size::new(40.0, 30.0));
        let mut nested = rect();
        for n in 0..64 {
            nested = Shape::new(
                ShapeId(n + 2),
                ShapeKind::Group {
                    children: vec![nested],
                },
            );
        }
        doc.symbols.push(Symbol {
            name: "Deep".into(),
            shape: nested,
        });
        doc.symbols.push(Symbol {
            name: "Two".into(),
            shape: reference(0),
        });
        doc.pages[0].layers[0].shapes.push(reference(1));
        assert_eq!(budget(&doc).unwrap().expanded_depth, 66);
        for n in 0..64 {
            doc.symbols.push(Symbol {
                name: format!("Link{n}"),
                shape: reference(doc.symbols.len() - 1),
            });
        }
        let last = doc.symbols.len() - 1;
        doc.pages[0].layers[0].shapes = vec![reference(last)];
        assert!(budget(&doc).is_err());
    }

    #[test]
    fn repeated_reference_multiplicity_and_cycles_are_bounded_without_expanding() {
        let mut doc = Document::new("Clipboard", Size::new(40.0, 30.0));
        doc.symbols.push(Symbol {
            name: "Leaf".into(),
            shape: rect(),
        });
        doc.symbols.push(Symbol {
            name: "Pair".into(),
            shape: Shape::new(
                ShapeId(5),
                ShapeKind::Group {
                    children: vec![reference(0), reference(0)],
                },
            ),
        });
        doc.pages[0].layers[0].shapes.push(reference(1));
        assert_eq!(budget(&doc).unwrap().expanded_nodes, 6);
        for n in 0..16 {
            let index = doc.symbols.len() - 1;
            doc.symbols.push(Symbol {
                name: format!("Pair{n}"),
                shape: Shape::new(
                    ShapeId(10 + n),
                    ShapeKind::Group {
                        children: vec![reference(index), reference(index)],
                    },
                ),
            });
        }
        assert!(budget(&doc).is_err());
        doc.symbols[0].shape = reference(1);
        assert!(budget(&doc).is_err());
    }
}
