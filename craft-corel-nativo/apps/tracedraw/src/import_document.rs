//! Transactional colour-aware import. Preparation never mutates the live app.

use super::color_import::{prepare_import_color, ImportColorOptions};
use super::App;
use crate::import_place::Placement;
use std::path::Path;
use tracedraw_core::geometry::{Affine, Point, Rect};
use tracedraw_core::{
    Command, Document, DocumentProfile, LayerId, PageId, Shape, ShapeId, ShapeKind,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum ColorImportPlacement {
    InFile,
    Center(Point),
    Cursor(Placement),
}

pub(crate) enum ColorImportRequest {
    File(std::path::PathBuf, ColorImportPlacement),
    Native(
        crate::clipboard::NativeClipboardPayload,
        ColorImportPlacement,
    ),
    /// The sender's instance counter belongs to another process. Its numeric
    /// equality never authorizes references back into the receiving document.
    ExternalNative(
        crate::clipboard::NativeClipboardPayload,
        ColorImportPlacement,
    ),
    /// Decode recognized native bytes only inside the preparation worker.
    ExternalWire(
        String,
        Option<crate::text_editing::InlinePasteTarget>,
        ColorImportPlacement,
    ),
}

#[derive(Clone, Debug)]
pub(crate) struct ImportStamp {
    instance: u64,
    revision: u64,
    page: PageId,
    layer: LayerId,
    selection: Vec<ShapeId>,
    color: crate::settings::ColorPrefs,
    hyphens: Vec<String>,
    symmetry: Option<ShapeId>,
}

pub(crate) struct PreparedColorImport {
    pub(crate) document: Document,
    pub(crate) selected: Vec<ShapeId>,
    pub(crate) notice: Option<String>,
    stamp: ImportStamp,
    pub(crate) elapsed_ms: u128,
    pub(crate) processed_bitmap_pixels: u64,
    label: &'static str,
    symmetry_after: Option<Option<ShapeId>>,
    pub(crate) inline_after: Option<crate::text_editing::InlinePasteAfter>,
}

pub(crate) struct ColorImportJob {
    receiver: std::sync::mpsc::Receiver<Result<PreparedColorImport, String>>,
    cancelled: bool,
    stamp: ImportStamp,
}

fn absolutize_links(doc: &mut Document, source: &Path) -> Result<(), String> {
    if crate::files::WEB {
        return Ok(());
    }
    let source = crate::files::canonical(source);
    let base = source.parent().unwrap_or(Path::new("."));
    tracedraw_core::color_convert::try_map_document_shape_tree(doc, &mut |shape| {
        for (key, value) in &mut shape.data {
            if key == "bitmap.link" && !value.is_empty() && Path::new(value).is_relative() {
                *value = base.join(&*value).display().to_string();
            }
        }
        Ok(())
    })?;
    Ok(())
}

pub(crate) fn validate_resources(doc: &Document) -> Result<(), String> {
    fn physical_ids(
        shape: &Shape,
        seen: &mut std::collections::HashSet<ShapeId>,
    ) -> Result<(), String> {
        if !seen.insert(shape.id) {
            return Err("ambiguous object identifiers in imported page contents".into());
        }
        match &shape.kind {
            ShapeKind::Group { children } => {
                for child in children {
                    physical_ids(child, seen)?;
                }
            }
            ShapeKind::ClipFrame { frame, contents } => {
                physical_ids(frame, seen)?;
                for child in contents {
                    physical_ids(child, seen)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let count = doc.symbols.len();
    let styles: std::collections::HashSet<u32> = doc.object_styles.iter().map(|s| s.id).collect();
    tracedraw_core::color_convert::try_visit_document_shape_tree(doc, &mut |shape| {
        if let ShapeKind::SymbolInstance { index } = &shape.kind {
            if *index >= count {
                return Err("an imported symbol reference is missing".into());
            }
        }
        if shape.style.is_some_and(|id| !styles.contains(&id)) {
            return Err("an imported object style is missing".into());
        }
        Ok(())
    })?;
    // The iterative visitor above rejects deep/oversized trees before any
    // recursive identity checks or clones, including in-memory clipboard data.
    let mut seen = std::collections::HashSet::new();
    for shape in doc.all_layers().flat_map(|layer| &layer.shapes) {
        physical_ids(shape, &mut seen)?;
    }
    let mut edges = vec![Vec::new(); count];
    for (index, symbol) in doc.symbols.iter().enumerate() {
        let mut shape = symbol.shape.clone();
        tracedraw_core::color_convert::try_map_shape_tree(&mut shape, &mut |shape| {
            if let ShapeKind::SymbolInstance { index: to } = shape.kind {
                edges[index].push(to);
            }
            Ok(())
        })?;
    }
    fn visit(
        index: usize,
        edges: &[Vec<usize>],
        states: &mut [u8],
        depth: usize,
    ) -> Result<(), String> {
        if depth > 128 || states[index] == 1 {
            return Err("cyclic or excessively nested symbol resources".into());
        }
        if states[index] == 2 {
            return Ok(());
        }
        states[index] = 1;
        for &to in &edges[index] {
            visit(to, edges, states, depth + 1)?;
        }
        states[index] = 2;
        Ok(())
    }
    let mut states = vec![0; count];
    for index in 0..count {
        visit(index, &edges, &mut states, 0)?;
    }
    Ok(())
}

fn bounds_with_symbols(
    shape: &Shape,
    doc: &Document,
    parent: Affine,
    depth: usize,
) -> Result<Rect, String> {
    if depth > 128 {
        return Err("import bounds exceed their nesting limit".into());
    }
    let transform = parent * shape.transform;
    match &shape.kind {
        ShapeKind::SymbolInstance { index } => bounds_with_symbols(
            &doc.symbols.get(*index).ok_or("missing symbol")?.shape,
            doc,
            transform,
            depth + 1,
        ),
        ShapeKind::Group { children } => {
            let mut bounds: Option<Rect> = None;
            for child in children {
                let b = bounds_with_symbols(child, doc, transform, depth + 1)?;
                bounds = Some(bounds.map_or(b, |a| a.union(b)));
            }
            Ok(bounds.unwrap_or(Rect::ZERO))
        }
        ShapeKind::ClipFrame { frame, .. } => bounds_with_symbols(frame, doc, transform, depth + 1),
        _ => {
            let b = shape.own_visual_bounds();
            let points = [
                Point::new(b.x0, b.y0),
                Point::new(b.x1, b.y0),
                Point::new(b.x1, b.y1),
                Point::new(b.x0, b.y1),
            ];
            let mut out = Rect::from_points(parent * points[0], parent * points[0]);
            for p in &points[1..] {
                out = out.union(Rect::from_points(parent * *p, parent * *p));
            }
            if ![out.x0, out.y0, out.x1, out.y1]
                .iter()
                .all(|v| v.is_finite())
            {
                return Err("import bounds are not finite".into());
            }
            Ok(out)
        }
    }
}

/// Reuse complete equivalent definitions, after colours and dependent symbol
/// references have been resolved in the destination's calibration. Identity
/// fields alone are ignored; geometry, paint, profiles, data and resources are
/// still compared. Existing destination definitions are never rewritten.
fn deduplicate_imported_symbols(
    doc: &mut Document,
    extra: &mut [Shape],
    first: usize,
) -> Result<(), String> {
    use std::collections::HashMap;
    fn normalized(shape: &Shape) -> Result<Shape, String> {
        let mut shape = shape.clone();
        tracedraw_core::color_convert::try_map_shape_tree(&mut shape, &mut |node| {
            node.id = ShapeId(0);
            node.uid = None;
            Ok(())
        })?;
        Ok(shape)
    }
    struct Resolver<'a> {
        doc: &'a Document,
        first: usize,
        resolved: Vec<usize>,
        state: Vec<u8>,
        candidates: HashMap<String, Vec<usize>>,
        forms: HashMap<usize, Shape>,
    }
    impl Resolver<'_> {
        fn resolve(&mut self, index: usize, depth: usize) -> Result<usize, String> {
            if index >= self.resolved.len() || depth > 128 {
                return Err("invalid symbol closure during import".into());
            }
            if index < self.first || self.state[index] == 2 {
                return Ok(self.resolved[index]);
            }
            if self.state[index] == 1 {
                return Err("cyclic imported symbol closure".into());
            }
            self.state[index] = 1;
            let mut shape = self.doc.symbols[index].shape.clone();
            let name = self.doc.symbols[index].name.clone();
            let mut references = std::collections::BTreeSet::new();
            tracedraw_core::color_convert::try_map_shape_tree(&mut shape, &mut |node| {
                if let ShapeKind::SymbolInstance { index } = node.kind {
                    references.insert(index);
                }
                Ok(())
            })?;
            for reference in references {
                self.resolve(reference, depth + 1)?;
            }
            tracedraw_core::color_convert::try_map_shape_tree(&mut shape, &mut |node| {
                node.id = ShapeId(0);
                node.uid = None;
                if let ShapeKind::SymbolInstance { index } = &mut node.kind {
                    *index = self.resolved[*index];
                }
                Ok(())
            })?;
            let mut matching = None;
            if let Some(candidates) = self.candidates.get(&name) {
                for &candidate in candidates {
                    if !self.forms.contains_key(&candidate) {
                        self.forms
                            .insert(candidate, normalized(&self.doc.symbols[candidate].shape)?);
                    }
                    if self.forms.get(&candidate) == Some(&shape) {
                        matching = Some(candidate);
                        break;
                    }
                }
            }
            let canonical = if let Some(candidate) = matching {
                candidate
            } else {
                self.candidates.entry(name).or_default().push(index);
                self.forms.insert(index, shape);
                index
            };
            self.resolved[index] = canonical;
            self.state[index] = 2;
            Ok(canonical)
        }
    }
    let count = doc.symbols.len();
    let mut candidates: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, symbol) in doc.symbols.iter().take(first).enumerate() {
        candidates
            .entry(symbol.name.clone())
            .or_default()
            .push(index);
    }
    let resolved = {
        let mut resolver = Resolver {
            doc,
            first,
            resolved: (0..count).collect(),
            state: vec![0; count],
            candidates,
            forms: HashMap::new(),
        };
        for index in first..count {
            resolver.resolve(index, 0)?;
        }
        resolver.resolved
    };
    let mut compact = vec![0; count];
    let mut next = 0;
    for index in 0..count {
        if resolved[index] == index {
            compact[index] = next;
            next += 1;
        }
    }
    let indices: Vec<_> = resolved.iter().map(|index| compact[*index]).collect();
    let mut remap = |shape: &mut Shape| {
        if let ShapeKind::SymbolInstance { index } = &mut shape.kind {
            *index = *indices
                .get(*index)
                .ok_or("missing imported symbol mapping")?;
        }
        Ok(())
    };
    tracedraw_core::color_convert::try_map_document_shape_tree(doc, &mut remap)?;
    for shape in extra {
        tracedraw_core::color_convert::try_map_shape_tree(shape, &mut remap)?;
    }
    let mut index = 0;
    doc.symbols.retain(|_| {
        let keep = resolved[index] == index;
        index += 1;
        keep
    });
    Ok(())
}

fn merge_import_resources(
    destination: &mut Document,
    mut incoming: Document,
    layer: LayerId,
    page: PageId,
    placement: ColorImportPlacement,
) -> Result<(Vec<ShapeId>, Option<String>), String> {
    use std::collections::HashMap;
    let source_symbols = incoming.symbols.len();
    let symbol_offset = destination.symbols.len();
    let mut ink_map = HashMap::new();
    let mut next_ink = destination.inks.iter().map(|i| i.id).max().unwrap_or(0);
    for ink in &incoming.inks {
        let id = if let Some(known) = destination
            .inks
            .iter()
            .find(|i| i.name == ink.name && i.cmyk == ink.cmyk)
        {
            known.id
        } else {
            next_ink = next_ink
                .checked_add(1)
                .ok_or("no free spot ink identifiers")?;
            destination.inks.push(tracedraw_core::document::Ink {
                id: next_ink,
                ..ink.clone()
            });
            next_ink
        };
        if ink_map.insert(ink.id, id).is_some() {
            return Err("duplicate source ink identifiers".into());
        }
    }
    let mut remap_ink = |color| {
        let mut color = color;
        if let tracedraw_core::Color::Spot { ink, .. } = &mut color {
            *ink = *ink_map.get(ink).ok_or("a source spot ink is missing")?;
        }
        Ok(color)
    };
    tracedraw_core::color_convert::try_map_document_colors(&mut incoming, &mut remap_ink)?;
    super::color_import::try_map_dimension_colors(&mut incoming, &mut remap_ink)?;
    let mut style_map = HashMap::new();
    let mut next_style = destination
        .object_styles
        .iter()
        .map(|s| s.id)
        .max()
        .unwrap_or(0);
    for style in &mut incoming.object_styles {
        let mut resource = Shape::new(
            ShapeId(0),
            ShapeKind::Group {
                children: Vec::new(),
            },
        );
        resource.fill = style.fill.clone();
        resource.stroke = style.stroke.clone();
        let mut foreign_references = false;
        tracedraw_core::color_convert::try_map_shape_tree(&mut resource, &mut |shape| {
            foreign_references |=
                shape.style.is_some() || matches!(shape.kind, ShapeKind::SymbolInstance { .. });
            Ok(())
        })?;
        let id = if let Some(known) = destination.object_styles.iter().find(|known| {
            !foreign_references
                && known.name == style.name
                && known.fill == style.fill
                && known.stroke == style.stroke
                && known.parts == style.parts
                && known.opacity == style.opacity
        }) {
            known.id
        } else {
            next_style = next_style
                .checked_add(1)
                .ok_or("no free object style identifiers")?;
            next_style
        };
        if style_map.insert(style.id, id).is_some() {
            return Err("duplicate source object style identifiers".into());
        }
        style.id = id;
    }
    let state = serde_json::to_value(&*destination.ids_mut()).map_err(|e| e.to_string())?;
    let mut next = state
        .get("next")
        .and_then(serde_json::Value::as_u64)
        .ok_or("invalid destination identifier state")?;
    let mut id_map = HashMap::new();
    tracedraw_core::color_convert::try_map_document_shape_tree(&mut incoming, &mut |shape| {
        let id = if let Some(id) = id_map.get(&shape.id) {
            *id
        } else {
            next = next.checked_add(1).ok_or("no free shape identifiers")?;
            let id = ShapeId(next);
            id_map.insert(shape.id, id);
            id
        };
        shape.id = id;
        shape.uid = None;
        if let ShapeKind::SymbolInstance { index } = &mut shape.kind {
            if *index >= source_symbols {
                return Err("a foreign symbol index is invalid".into());
            }
            *index = symbol_offset
                .checked_add(*index)
                .ok_or("too many symbol resources")?;
        }
        if let Some(style) = &mut shape.style {
            *style = *style_map
                .get(style)
                .ok_or("a source object style is missing")?;
        }
        if let Some(inks) = &mut shape.picture_inks {
            for ink in &mut inks.inks {
                if let tracedraw_core::picture_inks::PictureInk::Spot(id) = ink {
                    *id = *ink_map.get(id).ok_or("a source bitmap ink is missing")?;
                }
            }
        }
        Ok(())
    })?;
    // Linked blend endpoints refer only to imported objects, never a coincidental
    // identifier already present in the destination.
    tracedraw_core::color_convert::try_map_document_shape_tree(&mut incoming, &mut |shape| {
        for effect in shape
            .effects
            .iter_mut()
            .chain(shape.parked_effects.iter_mut().map(|e| &mut e.effect))
        {
            if let tracedraw_core::live::Effect::Blend { end_id, .. } = effect {
                *end_id = end_id.and_then(|id| id_map.get(&id).copied());
            }
        }
        // These keys are the text-flow contract, not arbitrary object data.
        for (key, value) in &mut shape.data {
            if matches!(key.as_str(), "text.next" | "text.prev") {
                if let Some(to) = value
                    .parse::<u64>()
                    .ok()
                    .and_then(|id| id_map.get(&ShapeId(id)))
                {
                    *value = to.raw().to_string();
                } else {
                    value.clear();
                }
            }
        }
        if let Some(mut connection) = crate::connectors::Connection::of(shape) {
            let endpoints = id_map.get(&connection.from).zip(id_map.get(&connection.to));
            shape.data = if let Some((from, to)) = endpoints {
                connection.from = *from;
                connection.to = *to;
                crate::connectors::with_connection(
                    std::mem::take(&mut shape.data),
                    Some(&connection),
                )
            } else {
                crate::connectors::with_connection(std::mem::take(&mut shape.data), None)
            };
        }
        if matches!(shape.kind, ShapeKind::Group { .. }) {
            if let Some((_, stored)) = shape
                .data
                .iter_mut()
                .find(|(key, _)| key == crate::dimensions::DATA_KEY)
            {
                let mut value: serde_json::Value =
                    serde_json::from_str(stored).map_err(|error| error.to_string())?;
                if let Some(anchors) = value.get("anchors") {
                    let mut anchors: Vec<crate::dimensions::Anchor> =
                        serde_json::from_value(anchors.clone())
                            .map_err(|error| error.to_string())?;
                    anchors.retain_mut(|anchor| {
                        if let Some(target) = id_map.get(&anchor.shape) {
                            anchor.shape = *target;
                            true
                        } else {
                            false
                        }
                    });
                    value["anchors"] =
                        serde_json::to_value(anchors).map_err(|error| error.to_string())?;
                    *stored = serde_json::to_string(&value).map_err(|error| error.to_string())?;
                }
            }
        }
        Ok(())
    })?;
    for style in incoming.object_styles {
        if !destination
            .object_styles
            .iter()
            .any(|known| known.id == style.id)
        {
            destination.object_styles.push(style);
        }
    }
    for style in incoming.color_styles {
        if !destination.color_styles.contains(&style) {
            destination.color_styles.push(style);
        }
    }
    for color in incoming.palette {
        if !destination.palette.contains(&color) {
            destination.palette.push(color);
        }
    }
    for font in incoming.fonts {
        if !destination.fonts.contains(&font) {
            destination.fonts.push(font);
        }
    }
    destination.symbols.extend(incoming.symbols);
    let has_masters = !incoming.master.is_empty();
    let mut masters = Vec::new();
    for master in incoming.master {
        next = next
            .checked_add(1)
            .ok_or("no free master resource identifiers")?;
        let mut group = Shape::new(
            ShapeId(next),
            ShapeKind::Group {
                children: master.shapes,
            },
        );
        group.fill = tracedraw_core::Fill::None;
        group.stroke = None;
        group.visible = master.visible;
        group.locked = master.locked;
        group.data = vec![
            ("import.master.name".into(), master.name.clone()),
            (
                "import.master.scope".into(),
                match master.scope {
                    tracedraw_core::document::MasterScope::All => "all",
                    tracedraw_core::document::MasterScope::Odd => "odd",
                    tracedraw_core::document::MasterScope::Even => "even",
                }
                .into(),
            ),
            (
                "import.master.printable".into(),
                master.printable.to_string(),
            ),
            ("import.master.in_front".into(), master.in_front.to_string()),
        ];
        let index = destination.symbols.len();
        destination.symbols.push(tracedraw_core::document::Symbol {
            name: master.name,
            shape: group,
        });
        masters.push((
            index,
            master.scope,
            master.in_front,
            master.visible,
            master.locked,
        ));
    }
    let mut shapes = Vec::new();
    for (page_index, source_page) in incoming.pages.into_iter().enumerate() {
        for front in [false, true] {
            if front {
                shapes.extend(
                    source_page
                        .layers
                        .iter()
                        .flat_map(|l| l.shapes.iter())
                        .cloned(),
                );
            }
            for &(index, scope, in_front, visible, locked) in &masters {
                if in_front == front && scope.applies(page_index) {
                    next = next
                        .checked_add(1)
                        .ok_or("no free master instance identifiers")?;
                    let mut instance =
                        Shape::new(ShapeId(next), ShapeKind::SymbolInstance { index });
                    instance.visible = visible;
                    instance.locked = locked;
                    instance.fill = tracedraw_core::Fill::None;
                    instance.stroke = None;
                    shapes.push(instance);
                }
            }
        }
    }
    // Validate definitions before resolving bounds, including nested instances.
    validate_resources(destination)?;
    deduplicate_imported_symbols(destination, &mut shapes, symbol_offset)?;
    let bounds = shapes
        .iter()
        .map(|s| bounds_with_symbols(s, destination, Affine::IDENTITY, 0))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .reduce(|a, b| a.union(b));
    if let Some(bounds) = bounds {
        let page_bounds = Rect::from_origin_size(
            (0.0, 0.0),
            destination.page(page).map_err(|e| e.to_string())?.size,
        );
        let transform = match placement {
            ColorImportPlacement::InFile => Affine::IDENTITY,
            ColorImportPlacement::Center(at) => Affine::translate(at - bounds.center()),
            ColorImportPlacement::Cursor(placement) => {
                crate::import_place::placement_transform(bounds, placement, page_bounds)
                    .ok_or("invalid import placement")?
            }
        };
        if !transform.as_coeffs().iter().all(|v| v.is_finite()) {
            return Err("import placement is not finite".into());
        }
        for shape in &mut shapes {
            shape.carry_fills(transform, Affine::IDENTITY);
            shape.transform_by(transform);
        }
    }
    let selected = shapes.iter().map(|s| s.id).collect();
    destination
        .layer_mut(layer)
        .map_err(|e| e.to_string())?
        .shapes
        .extend(shapes);
    destination.ids_mut().reserve(next);
    let notice = has_masters.then(|| crate::i18n::tr("status.import_master_symbols"));
    Ok((selected, notice))
}

fn profile_options(
    destination: &Document,
    incoming: &Document,
    color: &crate::settings::ColorPrefs,
) -> Result<ImportColorOptions, String> {
    let default_cmyk = if (destination.cmyk_profile.is_none() || incoming.cmyk_profile.is_none())
        && !color.cmyk_profile_path.is_empty()
    {
        if !crate::files::WEB
            && std::fs::metadata(&color.cmyk_profile_path)
                .map_err(|e| e.to_string())?
                .len()
                > 16 * 1024 * 1024
        {
            return Err("default ICC exceeds its size budget".into());
        }
        let bytes =
            crate::files::read(Path::new(&color.cmyk_profile_path)).map_err(|e| e.to_string())?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("default ICC exceeds its size budget".into());
        }
        let profile = tracedraw_core::icc::Profile::parse(&bytes).map_err(|e| e.to_string())?;
        if profile.color_space() != tracedraw_core::icc::ColorSpace::Cmyk {
            return Err("a CMYK default profile is required".into());
        }
        Some(DocumentProfile::new(&color.cmyk_profile, bytes))
    } else {
        None
    };
    Ok(ImportColorOptions {
        policy: color.import_cmyk,
        default_cmyk,
        bitmap: tracedraw_io::picture_profile::Options {
            intent: tracedraw_core::icc::Intent::from_name(&color.intent).unwrap_or_default(),
            black_point_compensation: color.black_point_compensation,
            preserve_pure_black: color.preserve_pure_black,
        },
        warn_missing: color.warn_import_missing,
        warn_mismatch: color.warn_import_mismatch,
    })
}

pub(crate) fn supports_color_import(path: &Path) -> bool {
    tracedraw_io::raster::is_raster_path(path)
        || matches!(App::file_kind(path).as_str(), "cdr" | "cmx" | "tdraw")
}

fn incoming_file(path: &Path) -> Result<Document, String> {
    if !crate::files::WEB
        && std::fs::metadata(path).map_err(|e| e.to_string())?.len()
            > tracedraw_io::raster::MAX_INPUT_BYTES
    {
        return Err("import input exceeds its file-size budget".into());
    }
    let bytes = crate::files::read(path).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > tracedraw_io::raster::MAX_INPUT_BYTES {
        return Err("import input exceeds its file-size budget".into());
    }
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported drawing");
    let mut doc = if tracedraw_io::raster::is_raster_path(path) {
        tracedraw_io::raster::document_from_bytes(&bytes, title).map_err(|e| e.to_string())?
    } else if App::file_kind(path) == "tdraw" {
        let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
        Document::from_json(text).map_err(|e| e.to_string())?
    } else {
        tracedraw_cdr::open_bytes(&bytes, title)
            .map_err(|e| e.to_string())?
            .0
    };
    if tracedraw_io::raster::is_raster_path(path) {
        for shape in doc.all_layers_mut().flat_map(|l| &mut l.shapes) {
            shape.name = path
                .file_name()
                .and_then(|s| s.to_str())
                .map(str::to_string);
            shape.data.push((
                "bitmap.link".into(),
                crate::files::canonical(path).display().to_string(),
            ));
        }
    }
    // A packaged drawing's fonts become document resources before planning;
    // registering them in the session font cache is deferred until commit.
    if App::file_kind(path) == "tdraw" && !crate::files::WEB {
        if let Some(folder) = path.parent().map(|p| p.join(crate::package::FONTS)) {
            let mut total: usize = doc.fonts.iter().map(|f| f.data.len()).sum();
            for font in tracedraw_text::catalog::font_files(&folder) {
                let size = std::fs::metadata(&font).map_err(|e| e.to_string())?.len();
                if size > 16 * 1024 * 1024 || total as u64 + size > 64 * 1024 * 1024 {
                    return Err("package fonts exceed their size budget".into());
                }
                let data = std::fs::read(&font).map_err(|e| e.to_string())?;
                let faces = tracedraw_text::catalog::describe_file(&data);
                if faces.is_empty() {
                    return Err("a package font is invalid".into());
                }
                let data = std::sync::Arc::new(data);
                for face in faces {
                    if matches!(
                        face.embedding,
                        tracedraw_text::catalog::Embedding::Restricted
                            | tracedraw_text::catalog::Embedding::BitmapOnly
                    ) {
                        continue;
                    }
                    if !doc
                        .fonts
                        .iter()
                        .any(|f| f.family == face.family && f.data.as_slice() == data.as_slice())
                    {
                        doc.fonts.push(tracedraw_core::DocumentFont {
                            family: face.family,
                            data: data.clone(),
                        });
                    }
                }
                total += data.len();
            }
        }
    }
    absolutize_links(&mut doc, path)?;
    Ok(doc)
}

fn build_import(
    destination: Document,
    incoming: Document,
    stamp: ImportStamp,
    placement: ColorImportPlacement,
) -> Result<PreparedColorImport, String> {
    build_import_with_calibration(destination, incoming, stamp, placement, false)
}

fn build_native_import(
    destination: Document,
    incoming: Document,
    stamp: ImportStamp,
    placement: ColorImportPlacement,
) -> Result<PreparedColorImport, String> {
    build_import_with_calibration(destination, incoming, stamp, placement, true)
}

fn build_import_with_calibration(
    mut destination: Document,
    incoming: Document,
    stamp: ImportStamp,
    placement: ColorImportPlacement,
    native_source: bool,
) -> Result<PreparedColorImport, String> {
    let began = web_time::Instant::now();
    validate_resources(&destination)?;
    validate_resources(&incoming)?;
    let mut effective_color = stamp.color.clone();
    if native_source && incoming.cmyk_profile.is_none() && destination.cmyk_profile.is_some() {
        // Both calibrations are explicit: generic source and carried target.
        // An unrelated default ICC file is not needed for this operation.
        effective_color.cmyk_profile_path.clear();
    }
    let mut options = profile_options(&destination, &incoming, &effective_color)?;
    let mut interpreted_pixels = 0u64;
    if native_source && incoming.cmyk_profile.is_none() && options.default_cmyk.is_some() {
        // None in a native payload is the sender's explicit generic model,
        // unlike a profile-missing external file interpreted with local defaults.
        // First materialize only the destination's actual default, including
        // retained previews, then remove the shared source-default fallback.
        let mut assignment = options.clone();
        assignment.policy = 1;
        assignment.warn_missing = false;
        assignment.warn_mismatch = false;
        let empty = Document::new(
            "Clipboard calibration",
            incoming.pages.first().ok_or("clipboard has no page")?.size,
        );
        let assigned = prepare_import_color(&destination, empty, &assignment)?;
        interpreted_pixels = assigned.processed_bitmap_pixels;
        for command in assigned.destination_commands {
            command
                .apply(&mut destination)
                .map_err(|error| error.to_string())?;
        }
        options.default_cmyk = None;
    }
    let plan = prepare_import_color(&destination, incoming, &options)?;
    for command in plan.destination_commands {
        command.apply(&mut destination).map_err(|e| e.to_string())?;
    }
    let (selected, master_notice) = merge_import_resources(
        &mut destination,
        plan.incoming,
        stamp.layer,
        stamp.page,
        placement,
    )?;
    validate_resources(&destination)?;
    if !stamp.hyphens.is_empty() {
        for id in &selected {
            if let Some(shape) = destination.find_shape_mut(*id) {
                if let ShapeKind::Text { spans, para, .. } = &mut shape.kind {
                    let text = spans
                        .iter()
                        .map(|span| span.text.as_str())
                        .collect::<String>()
                        .to_lowercase();
                    if text
                        .split(|c: char| !c.is_alphabetic())
                        .any(|word| stamp.hyphens.iter().any(|known| known == word))
                    {
                        para.hyphenate = true;
                    }
                }
            }
        }
    }
    let notice = match (plan.notice, master_notice) {
        (Some(a), Some(b)) => Some(format!("{a} | {b}")),
        (a, b) => a.or(b),
    };
    Ok(PreparedColorImport {
        document: destination,
        selected,
        notice,
        stamp,
        elapsed_ms: began.elapsed().as_millis(),
        processed_bitmap_pixels: plan
            .processed_bitmap_pixels
            .saturating_add(interpreted_pixels),
        label: "Import",
        symmetry_after: None,
        inline_after: None,
    })
}

fn paste_extras(
    prepared: &mut PreparedColorImport,
    originals: &[Shape],
    same_document: bool,
) -> Result<(), String> {
    let doc = &mut prepared.document;
    if same_document {
        let copied: std::collections::HashMap<ShapeId, ShapeId> = originals
            .iter()
            .zip(&prepared.selected)
            .filter(|(source, _)| {
                doc.find_shape(source.id)
                    .is_some_and(|known| source.uid.is_some() && known.uid == source.uid)
            })
            .map(|(source, copy)| (source.id, *copy))
            .collect();
        let connectors: Vec<(Shape, crate::connectors::Connection)> = doc
            .layers_for_page(prepared.stamp.page)
            .map_err(|e| e.to_string())?
            .iter()
            .flat_map(|layer| &layer.shapes)
            .filter(|shape| !copied.contains_key(&shape.id))
            .filter_map(|shape| {
                crate::connectors::Connection::of(shape)
                    .map(|connection| (shape.clone(), connection))
            })
            .collect();
        let at = |bounds: Rect, fractions: (f64, f64)| {
            Point::new(
                bounds.x0 + bounds.width() * fractions.0,
                bounds.y0 + bounds.height() * fractions.1,
            )
        };
        for (mut shape, connection) in connectors {
            let endpoints = match (copied.get(&connection.from), copied.get(&connection.to)) {
                (Some(from), None) => (*from, connection.to),
                (None, Some(to)) => (connection.from, *to),
                _ => continue,
            };
            let Some(from) = doc.shape_on_page(endpoints.0) else {
                continue;
            };
            let Some(to) = doc.shape_on_page(endpoints.1) else {
                continue;
            };
            let ends = (
                at(from.bounds(), connection.from_anchor),
                at(to.bounds(), connection.to_anchor),
            );
            let joined = crate::connectors::Connection {
                from: endpoints.0,
                to: endpoints.1,
                ends,
                ..connection
            };
            tracedraw_core::color_convert::try_map_shape_tree(&mut shape, &mut |child| {
                let last = serde_json::to_value(doc.ids()).map_err(|e| e.to_string())?["next"]
                    .as_u64()
                    .ok_or("invalid identifier source")?;
                let next = last
                    .checked_add(1)
                    .filter(|v| *v < u64::MAX)
                    .ok_or("no free connector identifiers")?;
                doc.ids_mut().reserve(next);
                child.id = ShapeId(next);
                child.uid = None;
                Ok(())
            })?;
            shape.transform = Affine::IDENTITY;
            shape.kind = ShapeKind::Path {
                path: crate::connectors::route(joined.kind, joined.radius, ends.0, ends.1),
                closed: false,
            };
            shape.data = crate::connectors::with_connection(shape.data, Some(&joined));
            doc.layer_mut(prepared.stamp.layer)
                .map_err(|e| e.to_string())?
                .shapes
                .push(shape);
        }
    }
    if let Some(group_id) = prepared.stamp.symmetry {
        if let Some(group) = doc.find_shape(group_id).cloned() {
            let symmetry: Vec<_> = group
                .effects
                .iter()
                .filter(|effect| matches!(effect, tracedraw_core::live::Effect::Symmetry { .. }))
                .cloned()
                .collect();
            if !symmetry.is_empty() && !prepared.selected.is_empty() {
                let mut members = Vec::new();
                match &group.kind {
                    ShapeKind::Group { children } => {
                        members.extend(children.iter().map(|child| child.id));
                        Command::Ungroup { group: group_id }
                            .apply(doc)
                            .map_err(|e| e.to_string())?;
                    }
                    _ => {
                        members.push(group_id);
                        Command::SetEffects {
                            shape: group_id,
                            effects: group
                                .effects
                                .iter()
                                .filter(|effect| {
                                    !matches!(effect, tracedraw_core::live::Effect::Symmetry { .. })
                                })
                                .cloned()
                                .collect(),
                        }
                        .apply(doc)
                        .map_err(|e| e.to_string())?;
                    }
                }
                for id in &prepared.selected {
                    let effects = doc
                        .shape(*id)
                        .map_err(|e| e.to_string())?
                        .1
                        .effects
                        .iter()
                        .filter(|effect| {
                            !matches!(effect, tracedraw_core::live::Effect::Symmetry { .. })
                        })
                        .cloned()
                        .collect();
                    Command::SetEffects {
                        shape: *id,
                        effects,
                    }
                    .apply(doc)
                    .map_err(|e| e.to_string())?;
                }
                members.extend(prepared.selected.iter().copied());
                Command::Group { shapes: members }
                    .apply(doc)
                    .map_err(|e| e.to_string())?;
                let new_group = doc
                    .all_layers()
                    .flat_map(|layer| &layer.shapes)
                    .filter(|shape| matches!(shape.kind, ShapeKind::Group { .. }))
                    .max_by_key(|shape| shape.id)
                    .ok_or("paste symmetry group was not created")?
                    .id;
                Command::SetEffects {
                    shape: new_group,
                    effects: symmetry,
                }
                .apply(doc)
                .map_err(|e| e.to_string())?;
                prepared.symmetry_after = Some(Some(new_group));
            }
        }
    }
    validate_resources(doc)
}

fn incoming_svg(svg: &str) -> Result<Document, String> {
    let mut ids = tracedraw_core::id::IdSource::default();
    let imported = tracedraw_io::svg_import::parse(svg, &mut ids)?;
    let mut document = Document::new("SVG clipboard", imported.size);
    document.pages[0].layers[0].shapes = imported.shapes;
    document.set_ids(ids);
    validate_resources(&document)?;
    Ok(document)
}

/// Metadata is never human clipboard text. Strip it by namespace/local name
/// before invoking the ordinary SVG character reader for an inline fallback.
fn human_svg(svg: &str) -> Result<String, String> {
    let doc = roxmltree::Document::parse_with_options(
        svg,
        roxmltree::ParsingOptions {
            allow_dtd: false,
            nodes_limit: 100_000,
            ..Default::default()
        },
    )
    .map_err(|error| error.to_string())?;
    let mut ranges: Vec<_> = doc
        .descendants()
        .filter(|node| {
            node.is_element()
                && (node.tag_name().name() == "metadata"
                    || node.tag_name().namespace() == Some(crate::clipboard_wire::NAMESPACE))
        })
        .map(|node| node.range())
        .collect();
    ranges.sort_by_key(|range| range.start);
    let mut clean = String::with_capacity(svg.len());
    let mut copied = 0;
    for range in ranges {
        if range.start < copied {
            continue;
        }
        clean.push_str(&svg[copied..range.start]);
        copied = range.end;
    }
    clean.push_str(&svg[copied..]);
    Ok(clean)
}

fn build_request_on_stack(
    destination: Document,
    stamp: ImportStamp,
    request: ColorImportRequest,
) -> Result<PreparedColorImport, String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::Builder::new()
            .name("clipboard-prepare-sync".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(move || build_request(destination, stamp, request))
            .map_err(|error| error.to_string())?
            .join()
            .map_err(|_| "clipboard preparation failed unexpectedly".to_string())?
    }
    #[cfg(target_arch = "wasm32")]
    {
        build_request(destination, stamp, request)
    }
}

fn build_request(
    destination: Document,
    stamp: ImportStamp,
    request: ColorImportRequest,
) -> Result<PreparedColorImport, String> {
    let external = matches!(&request, ColorImportRequest::ExternalNative(_, _));
    match request {
        ColorImportRequest::ExternalWire(svg, inline, placement) => {
            if svg.len() > crate::clipboard_wire::MAX_SVG_BYTES {
                return Err("clipboard SVG exceeds its byte budget".into());
            }
            let decoded = crate::clipboard_wire::decode_svg(&svg);
            if let Some(target) = inline {
                let (source, native) = match decoded {
                    Ok(Some(payload)) => (payload.document, true),
                    // Inline pasting can read independent human SVG characters,
                    // never metadata, even if recognized native data is damaged.
                    Ok(None) | Err(_) => (incoming_svg(&human_svg(&svg)?)?, false),
                };
                let incoming = crate::text_editing::inline_text_document(&source)?;
                let mut prepared = if native {
                    build_native_import(destination, incoming, stamp, ColorImportPlacement::InFile)?
                } else {
                    build_import(destination, incoming, stamp, ColorImportPlacement::InFile)?
                };
                prepared.label = "Paste";
                crate::text_editing::splice_inline_paste(&mut prepared, target)?;
                validate_resources(&prepared.document)?;
                return Ok(prepared);
            }
            if let Some(payload) = decoded? {
                return build_request(
                    destination,
                    stamp,
                    ColorImportRequest::ExternalNative(payload, placement),
                );
            }
            let incoming = incoming_svg(&svg)?;
            let mut prepared = build_import(destination, incoming, stamp, placement)?;
            prepared.label = "Paste";
            Ok(prepared)
        }
        ColorImportRequest::File(path, placement) => {
            build_import(destination, incoming_file(&path)?, stamp, placement)
        }
        ColorImportRequest::Native(payload, placement)
        | ColorImportRequest::ExternalNative(payload, placement) => {
            if payload.version != 1 {
                return Err("unsupported native clipboard payload version".into());
            }
            validate_resources(&payload.document)?;
            let originals = payload
                .document
                .pages
                .iter()
                .flat_map(|page| &page.layers)
                .flat_map(|layer| &layer.shapes)
                .cloned()
                .collect::<Vec<_>>();
            let same_document = !external && payload.source_instance == stamp.instance;
            let mut prepared =
                build_native_import(destination, payload.document, stamp, placement)?;
            prepared.label = "Paste";
            paste_extras(&mut prepared, &originals, same_document)?;
            Ok(prepared)
        }
    }
}

/// Pure snapshot preparation shared by synchronous hooks and the copy worker.
pub(crate) fn native_payload_from_document(
    source: &Document,
    page: PageId,
    shapes: &[Shape],
    instance: u64,
    color: &crate::settings::ColorPrefs,
) -> Result<crate::clipboard::NativeClipboardPayload, String> {
    let mut payload = crate::clipboard::NativeClipboardPayload::capture(source, shapes, instance)?;
    // The application knows the active viewport; a generic document capture
    // cannot infer it from the first page or from the selected geometry.
    payload.document.pages[0].size = source.page(page).map_err(|error| error.to_string())?.size;
    if payload.document.cmyk_profile.is_none() {
        // Untagged content was displayed using this source application's
        // actual working default, which can differ from the paste target.
        let options = profile_options(source, &payload.document, color)?;
        payload.document.cmyk_profile = options.default_cmyk;
        payload.document.metadata.cmyk_profile = payload
            .document
            .cmyk_profile
            .as_ref()
            .map(|profile| profile.name.clone())
            .unwrap_or_else(|| "Generic CMYK (open)".into());
    }
    Ok(payload)
}

impl App {
    pub(crate) fn native_payload_for_selection(
        &self,
        shapes: &[Shape],
    ) -> Result<crate::clipboard::NativeClipboardPayload, String> {
        native_payload_from_document(
            self.doc(),
            self.page,
            shapes,
            self.engine.instance(),
            &self.settings.color,
        )
    }

    pub(crate) fn uses_builtin_color_import(&self, path: &Path) -> bool {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        supports_color_import(path) && self.import_filter_for(&extension).is_none()
    }

    fn import_stamp(&self) -> Result<ImportStamp, String> {
        Ok(ImportStamp {
            instance: self.engine.instance(),
            revision: self.engine.revision(),
            page: self.page,
            layer: self
                .active_layer()
                .ok_or("no writable import layer is selected")?,
            selection: self.selection.clone(),
            color: self.settings.color.clone(),
            hyphens: if self.settings.hyphens_when_pasting {
                self.settings
                    .custom_hyphens
                    .iter()
                    .map(|(_, word, _)| word.to_lowercase())
                    .collect()
            } else {
                Vec::new()
            },
            symmetry: self.symmetry_edit,
        })
    }

    pub(crate) fn prepare_color_import(
        &self,
        incoming: Document,
        at: Option<Point>,
    ) -> Result<PreparedColorImport, String> {
        let stamp = self.import_stamp()?;
        build_import(
            self.doc().clone(),
            incoming,
            stamp,
            at.map_or(ColorImportPlacement::InFile, ColorImportPlacement::Center),
        )
    }

    pub(crate) fn commit_color_import(
        &mut self,
        prepared: PreparedColorImport,
    ) -> Result<Vec<ShapeId>, String> {
        let began = web_time::Instant::now();
        let now = self.import_stamp()?;
        let old = &prepared.stamp;
        if prepared
            .inline_after
            .as_ref()
            .is_some_and(|after| !after.target.matches(self))
        {
            return Err("the text paste caret changed during preparation".into());
        }
        if now.instance != old.instance
            || now.revision != old.revision
            || now.page != old.page
            || now.layer != old.layer
            || now.selection != old.selection
            || now.color != old.color
            || now.hyphens != old.hyphens
            || now.symmetry != old.symmetry
        {
            return Err("the import destination changed during preparation".into());
        }
        if prepared.document.title != self.doc().title
            || prepared
                .document
                .pages
                .iter()
                .map(|p| p.id)
                .collect::<Vec<_>>()
                != self.doc().pages.iter().map(|p| p.id).collect::<Vec<_>>()
        {
            return Err("prepared import changed destination pages or title".into());
        }
        prepared
            .document
            .page(old.page)
            .map_err(|e| e.to_string())?;
        if !prepared.document.all_layers().any(|l| l.id == old.layer) {
            return Err("prepared import lost its destination layer".into());
        }
        validate_resources(&prepared.document)?;
        self.engine
            .run_batch(
                prepared.label,
                &[Command::ApplyImportDocument {
                    document: Box::new(prepared.document),
                }],
            )
            .map_err(|e| e.to_string())?;
        let ids = prepared.selected;
        if let Some(step) = self.engine.last_step_id() {
            self.import_selections.insert(
                (self.engine.instance(), step),
                (old.selection.clone(), ids.clone()),
            );
        }
        self.add_carried_fonts(&self.doc().fonts.clone());
        self.select(ids.clone());
        if let Some(symmetry) = prepared.symmetry_after {
            self.symmetry_edit = symmetry;
        }
        self.apply_color_settings();
        if let Some(after) = prepared.inline_after {
            self.finish_inline_paste(after);
        }
        self.status = prepared.notice.unwrap_or_else(|| {
            crate::i18n::tr(if prepared.label == "Paste" {
                "status.paste_ready"
            } else {
                "status.import_ready"
            })
        });
        log::info!(
            "colour-aware import prepared in {} ms ({} bitmap pixels); commit took {} ms",
            prepared.elapsed_ms,
            prepared.processed_bitmap_pixels,
            began.elapsed().as_millis()
        );
        Ok(ids)
    }

    pub(crate) fn try_import_path(
        &mut self,
        path: &Path,
        at: Option<Point>,
    ) -> Result<Vec<ShapeId>, String> {
        if !supports_color_import(path) {
            return Err("this import API supports CDR, CMX, TDRAW and raster images".into());
        }
        if !self.uses_builtin_color_import(path) {
            return Err("an installed script filter overrides this extension; transactional import cannot validate its result".into());
        }
        if at.is_some_and(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err("import placement must be finite".into());
        }
        let stamp = self.import_stamp()?;
        let incoming = incoming_file(path)?;
        let at = at.or_else(|| {
            tracedraw_io::raster::is_raster_path(path).then(|| self.page_rect().center())
        });
        let prepared = build_import(
            self.doc().clone(),
            incoming,
            stamp,
            at.map_or(ColorImportPlacement::InFile, ColorImportPlacement::Center),
        )?;
        self.commit_color_import(prepared)
    }

    pub(crate) fn dispatch_color_import(
        &mut self,
        path: &Path,
        placement: ColorImportPlacement,
    ) -> Result<(), String> {
        if self.color_import_context.is_none() || crate::files::WEB {
            let stamp = self.import_stamp()?;
            let incoming = incoming_file(path)?;
            let placement = if matches!(placement, ColorImportPlacement::InFile)
                && tracedraw_io::raster::is_raster_path(path)
            {
                ColorImportPlacement::Center(self.page_rect().center())
            } else {
                placement
            };
            let prepared = build_import(self.doc().clone(), incoming, stamp, placement)?;
            self.commit_color_import(prepared)?;
            Ok(())
        } else {
            self.start_color_import(path, placement)
        }
    }

    pub(crate) fn try_paste_native(
        &mut self,
        payload: &crate::clipboard::NativeClipboardPayload,
    ) -> Result<Vec<ShapeId>, String> {
        validate_resources(&payload.document)?;
        validate_resources(self.doc())?;
        let request = ColorImportRequest::Native(payload.clone(), ColorImportPlacement::InFile);
        let prepared = build_request_on_stack(self.doc().clone(), self.import_stamp()?, request)?;
        self.commit_color_import(prepared)
    }

    pub(crate) fn dispatch_native_paste(
        &mut self,
        payload: crate::clipboard::NativeClipboardPayload,
        at: Option<Point>,
    ) -> Result<(), String> {
        let placement = at.map_or(ColorImportPlacement::InFile, ColorImportPlacement::Center);
        let request = ColorImportRequest::Native(payload, placement);
        if self.color_import_context.is_none() || crate::files::WEB {
            validate_resources(self.doc())?;
            let prepared =
                build_request_on_stack(self.doc().clone(), self.import_stamp()?, request)?;
            self.commit_color_import(prepared)?;
            Ok(())
        } else {
            self.start_import_request(request)
        }
    }

    pub(crate) fn dispatch_wire_paste(&mut self, svg: String) -> Result<(), String> {
        self.dispatch_wire_paste_at(svg, None)
    }

    pub(crate) fn dispatch_wire_paste_at(
        &mut self,
        svg: String,
        at: Option<Point>,
    ) -> Result<(), String> {
        self.dispatch_wire_request(
            svg,
            None,
            at.map_or(ColorImportPlacement::InFile, ColorImportPlacement::Center),
        )
    }

    pub(crate) fn dispatch_inline_wire_paste(
        &mut self,
        svg: String,
        target: crate::text_editing::InlinePasteTarget,
    ) -> Result<(), String> {
        self.dispatch_wire_request(svg, Some(target), ColorImportPlacement::InFile)
    }

    fn dispatch_wire_request(
        &mut self,
        svg: String,
        inline: Option<crate::text_editing::InlinePasteTarget>,
        placement: ColorImportPlacement,
    ) -> Result<(), String> {
        if svg.len() > crate::clipboard_wire::MAX_SVG_BYTES {
            return Err("clipboard SVG exceeds its byte budget".into());
        }
        if self.color_import_context.is_none() || crate::files::WEB {
            validate_resources(self.doc())?;
            let prepared = build_request_on_stack(
                self.doc().clone(),
                self.import_stamp()?,
                ColorImportRequest::ExternalWire(svg, inline, placement),
            )?;
            self.commit_color_import(prepared)?;
            Ok(())
        } else {
            self.start_import_request(ColorImportRequest::ExternalWire(svg, inline, placement))
        }
    }

    pub(crate) fn start_color_import(
        &mut self,
        path: &Path,
        placement: ColorImportPlacement,
    ) -> Result<(), String> {
        let placement = if matches!(placement, ColorImportPlacement::InFile)
            && tracedraw_io::raster::is_raster_path(path)
        {
            ColorImportPlacement::Center(self.page_rect().center())
        } else {
            placement
        };
        self.start_import_request(ColorImportRequest::File(path.to_path_buf(), placement))
    }

    fn start_import_request(&mut self, request: ColorImportRequest) -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        let pasting = matches!(
            &request,
            ColorImportRequest::Native(_, _)
                | ColorImportRequest::ExternalNative(_, _)
                | ColorImportRequest::ExternalWire(_, _, _)
        );
        if let Some(job) = &self.color_import_job {
            if job.cancelled
                || self.color_import_queue.len() >= 32
                || self.engine.instance() != job.stamp.instance
                || self.page != job.stamp.page
                || self.active_layer() != Some(job.stamp.layer)
            {
                return Err("an import is still being prepared for another destination".into());
            }
            let text_bytes = |request: &ColorImportRequest| match request {
                ColorImportRequest::ExternalWire(svg, _, _) => svg.len(),
                _ => 0,
            };
            let total = self
                .color_import_queue
                .iter()
                .map(text_bytes)
                .sum::<usize>()
                .saturating_add(text_bytes(&request));
            if total > 2 * crate::clipboard_wire::MAX_SVG_BYTES {
                return Err("clipboard import queue exceeds its byte budget".into());
            }
            self.color_import_queue.push_back(request);
            return Ok(());
        }
        if crate::files::WEB {
            let prepared = build_request(self.doc().clone(), self.import_stamp()?, request)?;
            self.commit_color_import(prepared)?;
            return Ok(());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let snapshot_began = web_time::Instant::now();
            let stamp = self.import_stamp()?;
            validate_resources(self.doc())?;
            let destination = self.doc().clone();
            log::info!(
                "colour-aware import snapshot took {} ms",
                snapshot_began.elapsed().as_millis()
            );
            let ctx = self.color_import_context.clone();
            let (sender, receiver) = std::sync::mpsc::channel();
            let job_stamp = stamp.clone();
            std::thread::Builder::new()
                .name("colour-import".into())
                .stack_size(16 * 1024 * 1024)
                .spawn(move || {
                    let began = web_time::Instant::now();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        build_request(destination, stamp, request).map(|mut prepared| {
                            prepared.elapsed_ms = began.elapsed().as_millis();
                            prepared
                        })
                    }))
                    .unwrap_or_else(|_| Err("import preparation failed unexpectedly".into()));
                    let _ = sender.send(result);
                    if let Some(ctx) = ctx {
                        ctx.request_repaint();
                    }
                })
                .map_err(|e| e.to_string())?;
            self.color_import_job = Some(ColorImportJob {
                receiver,
                cancelled: false,
                stamp: job_stamp,
            });
            self.status = crate::i18n::tr(if pasting {
                "status.paste_preparing"
            } else {
                "status.import_preparing"
            });
        }
        Ok(())
    }

    pub(crate) fn cancel_color_import(&mut self) {
        self.cancel_clipboard_copy();
        self.color_import_queue.clear();
        if let Some(job) = &mut self.color_import_job {
            job.cancelled = true;
        }
    }

    pub(crate) fn poll_color_import(&mut self) {
        self.poll_clipboard_copy();
        let Some(job) = self.color_import_job.take() else {
            return;
        };
        match job.receiver.try_recv() {
            Ok(result) => {
                if !job.cancelled {
                    if let Err(e) = result.and_then(|prepared| self.commit_color_import(prepared)) {
                        self.color_import_queue.clear();
                        self.status = crate::i18n::trf("status.import_failed", &[("e", &e)]);
                    } else if let Some(request) = self.color_import_queue.pop_front() {
                        if let Err(e) = self.start_import_request(request) {
                            self.status = crate::i18n::trf("status.import_failed", &[("e", &e)]);
                            self.color_import_queue.clear();
                        }
                    }
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => self.color_import_job = Some(job),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.color_import_queue.clear();
                if !job.cancelled {
                    self.status =
                        crate::i18n::trf("status.import_failed", &[("e", "import worker stopped")]);
                }
            }
        }
    }
}
