//! Private Craft Corel units extension, deliberately separate from standard MCFG.
//! Numeric identifiers here belong to the SDK domain, not CDR record codes.
use serde_json::{json, Value};
use std::io::{Cursor, Read, Write};
use tracedraw_cdr::{container, open_bytes, riff, write::document_to_cdr_version};
use tracedraw_core::{
    document::Corners,
    geometry::{PathEl, Shape as _},
    Color, Document, Fill, Point, Rect, Shape, ShapeKind, Size,
};

const MEMBER: &str = "craft-corel/document-units.json";
type Triple = (Option<u32>, Option<u32>, Option<u32>);

fn scene(units: Triple) -> Document {
    let mut doc = Document::new("Private units scene", Size::new(100.0, 70.0));
    let mut ids = doc.ids().clone();
    let mut shape = Shape::new(
        ids.shape(),
        ShapeKind::Rect {
            rect: Rect::new(10.0, 10.0, 50.0, 30.0),
            radius: 0.0,
            corners: Some(Corners {
                radii: [2.0, 4.0, 8.0, 6.0],
                ..Default::default()
            }),
        },
    );
    shape.fill = Fill::Solid(Color::rgb8(20, 150, 80));
    shape.stroke = None;
    doc.pages[0].layers[0].shapes.push(shape);
    doc.set_ids(ids);
    doc.metadata.script_unit = units.0;
    doc.metadata.rulers.horizontal_unit = units.1;
    doc.metadata.rulers.vertical_unit = units.2;
    doc
}

fn triple(doc: &Document) -> Triple {
    (
        doc.metadata.script_unit,
        doc.metadata.rulers.horizontal_unit,
        doc.metadata.rulers.vertical_unit,
    )
}

fn near(a: f64, b: f64) {
    assert!(a.is_finite() && (a - b).abs() <= 0.002, "{a} != {b}");
}

fn geometry(doc: &Document) {
    assert_eq!(doc.pages.len(), 1);
    near(doc.pages[0].size.width, 100.0);
    near(doc.pages[0].size.height, 70.0);
    let shapes: Vec<_> = doc.all_layers().flat_map(|l| &l.shapes).collect();
    assert_eq!(shapes.len(), 1);
    let shape = shapes[0];
    assert_eq!(shape.fill, Fill::Solid(Color::rgb8(20, 150, 80)));
    assert!(shape.stroke.is_none());
    let path = shape.page_path();
    let bounds = path.bounding_box();
    for (a, b) in [bounds.x0, bounds.y0, bounds.x1, bounds.y1]
        .into_iter()
        .zip([10.0, 10.0, 50.0, 30.0])
    {
        near(a, b);
    }
    let endpoints: Vec<Point> = path
        .elements()
        .iter()
        .filter_map(|e| match e {
            PathEl::MoveTo(p)
            | PathEl::LineTo(p)
            | PathEl::QuadTo(_, p)
            | PathEl::CurveTo(_, _, p) => Some(*p),
            PathEl::ClosePath => None,
        })
        .collect();
    // Four different physical radii, checked on drawn tangent endpoints even
    // when newer CDR writers intentionally represent rectangles as curves.
    for (x, y) in [
        (12.0, 30.0),
        (10.0, 28.0),
        (46.0, 30.0),
        (50.0, 26.0),
        (44.0, 10.0),
        (50.0, 16.0),
        (18.0, 10.0),
        (10.0, 18.0),
    ] {
        assert!(
            endpoints
                .iter()
                .any(|p| (p.x - x).abs() <= 0.002 && (p.y - y).abs() <= 0.002),
            "missing tangent ({x},{y}) in {endpoints:?}"
        );
    }
}

fn fnv(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn mcfg(bytes: &[u8]) -> Vec<u8> {
    let container = container::detect(bytes).unwrap();
    let main = container::riff_stream(bytes, &container).unwrap();
    let externals = container::external_streams(bytes, &container);
    let tree = riff::parse_with_externals(&main, externals, container.version.0).unwrap();
    let mut found = Vec::new();
    tree.root.walk(&mut |chunk, _| {
        if chunk.id == *b"mcfg" {
            found.push(tree.data(&main, chunk).to_vec());
        }
    });
    assert_eq!(
        found.len(),
        1,
        "fixture must have one opaque standard configuration"
    );
    found.pop().unwrap()
}

fn entries(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    (0..archive.len())
        .map(|i| {
            let mut member = archive.by_index(i).unwrap();
            let name = member.name().unwrap().into_owned();
            let mut data = Vec::new();
            member.read_to_end(&mut data).unwrap();
            (name, data)
        })
        .collect()
}

// A tiny independent stored-ZIP fixture writer permits duplicate central names
// that the production ZIP writer correctly refuses to produce.
fn stored_zip(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    fn u16v(out: &mut Vec<u8>, value: u16) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    fn u32v(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    fn crc(data: &[u8]) -> u32 {
        let mut c = u32::MAX;
        for &b in data {
            c ^= u32::from(b);
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    (c >> 1) ^ 0xedb88320
                } else {
                    c >> 1
                };
            }
        }
        !c
    }
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let offset = out.len() as u32;
        let crc = crc(data);
        let len = data.len() as u32;
        let n = name.len() as u16;
        u32v(&mut out, 0x04034b50);
        u16v(&mut out, 20);
        u16v(&mut out, 0);
        u16v(&mut out, 0);
        u16v(&mut out, 0);
        u16v(&mut out, 0);
        u32v(&mut out, crc);
        u32v(&mut out, len);
        u32v(&mut out, len);
        u16v(&mut out, n);
        u16v(&mut out, 0);
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        u32v(&mut central, 0x02014b50);
        u16v(&mut central, 20);
        u16v(&mut central, 20);
        u16v(&mut central, 0);
        u16v(&mut central, 0);
        u16v(&mut central, 0);
        u16v(&mut central, 0);
        u32v(&mut central, crc);
        u32v(&mut central, len);
        u32v(&mut central, len);
        u16v(&mut central, n);
        u16v(&mut central, 0);
        u16v(&mut central, 0);
        u16v(&mut central, 0);
        u16v(&mut central, 0);
        u32v(&mut central, 0);
        u32v(&mut central, offset);
        central.extend_from_slice(name.as_bytes());
    }
    let offset = out.len() as u32;
    let length = central.len() as u32;
    out.extend_from_slice(&central);
    u32v(&mut out, 0x06054b50);
    u16v(&mut out, 0);
    u16v(&mut out, 0);
    u16v(&mut out, entries.len() as u16);
    u16v(&mut out, entries.len() as u16);
    u32v(&mut out, length);
    u32v(&mut out, offset);
    u16v(&mut out, 0);
    out
}

fn chunk(id: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut out = id.to_vec();
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
    if data.len() % 2 == 1 {
        out.push(0);
    }
    out
}

fn top_chunks(bytes: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
    assert_eq!(&bytes[..4], b"RIFF");
    let mut pos = 12;
    let mut result = Vec::new();
    let end = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize + 8;
    assert_eq!(end, bytes.len());
    while pos < end {
        let id = bytes[pos..pos + 4].try_into().unwrap();
        let len = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        result.push((id, bytes[pos + 8..pos + 8 + len].to_vec()));
        pos += 8 + len + (len & 1);
    }
    assert_eq!(pos, end);
    result
}

fn riff_with_chunks(original: &[u8], chunks: &[([u8; 4], Vec<u8>)]) -> Vec<u8> {
    let mut body = original[8..12].to_vec();
    for (id, data) in chunks {
        body.extend(chunk(id, data));
    }
    let mut out = b"RIFF".to_vec();
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend(body);
    out
}

fn payload(bytes: &[u8]) -> Vec<u8> {
    if bytes.starts_with(b"RIFF") {
        let mut records: Vec<_> = top_chunks(bytes)
            .into_iter()
            .filter(|(id, _)| id == b"ccun")
            .collect();
        assert_eq!(records.len(), 1);
        records.pop().unwrap().1
    } else {
        let mut records: Vec<_> = entries(bytes)
            .into_iter()
            .filter(|(name, _)| name == MEMBER)
            .collect();
        assert_eq!(records.len(), 1);
        records.pop().unwrap().1
    }
}

fn replace_payload(bytes: &[u8], payloads: &[Vec<u8>]) -> Vec<u8> {
    if bytes.starts_with(b"RIFF") {
        let mut parts: Vec<_> = top_chunks(bytes)
            .into_iter()
            .filter(|(id, _)| id != b"ccun")
            .collect();
        let before = parts
            .iter()
            .position(|(id, _)| id == b"sumi")
            .unwrap_or(parts.len());
        for (offset, payload) in payloads.iter().cloned().enumerate() {
            parts.insert(before + offset, (*b"ccun", payload));
        }
        riff_with_chunks(bytes, &parts)
    } else {
        let mut parts: Vec<_> = entries(bytes)
            .into_iter()
            .filter(|(name, _)| name != MEMBER)
            .collect();
        parts.extend(payloads.iter().cloned().map(|p| (MEMBER.to_owned(), p)));
        stored_zip(&parts)
    }
}

fn check_bad(bytes: &[u8], reason: &str) {
    let (doc, report) = open_bytes(bytes, "Rejected units").unwrap_or_else(|e| {
        panic!("base scene must open despite bad private units ({reason}): {e}")
    });
    assert_eq!(
        triple(&doc),
        (None, None, None),
        "atomic rejection: {reason}"
    );
    geometry(&doc);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.to_ascii_lowercase().contains("unit")),
        "missing units diagnostic for {reason}: {:?}",
        report.warnings
    );
}

// The public reader supplies opaque inflated bytes. This fixture serializer
// independently regenerates CPng streams and size indices; it never invokes
// the production writer's private compression/record-building helpers.
struct ConfigurationFixture<'a> {
    tree: &'a riff::Tree,
    main: &'a [u8],
    replacement: &'a [Vec<u8>],
    matched: usize,
}

impl ConfigurationFixture<'_> {
    fn emit(out: &mut Vec<u8>, id: &[u8; 4], data: &[u8], indexed: bool, pool: &mut Vec<u32>) {
        let size = u32::try_from(data.len()).unwrap();
        let field = if indexed {
            let index = u32::try_from(pool.len()).unwrap();
            pool.push(size);
            index
        } else {
            size
        };
        out.extend_from_slice(id);
        out.extend_from_slice(&field.to_le_bytes());
        out.extend_from_slice(data);
        if data.len() % 2 == 1 {
            out.push(0);
        }
    }

    fn cpng(data: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        let mut block = b"CPng\x01\x00\x04\x00".to_vec();
        block.extend(encoder.finish().unwrap());
        block
    }

    fn nodes(&mut self, nodes: &[riff::Chunk], indexed: bool, pool: &mut Vec<u32>) -> Vec<u8> {
        let mut out = Vec::new();
        for node in nodes {
            // Tree's virtual inflated-stream wrapper has no on-disk header.
            if node.id == *b"cmpr" && node.list_type == Some(*b"cmpr") {
                out.extend(self.nodes(&node.children, indexed, pool));
                continue;
            }
            if node.id == *b"mcfg" && node.list_type.is_none() {
                self.matched += 1;
                for data in self.replacement {
                    Self::emit(&mut out, &node.id, data, indexed, pool);
                }
                continue;
            }
            let data = if let Some(kind) = node.list_type {
                let mut data = kind.to_vec();
                if kind == *b"cmpr" {
                    let mut inner_pool = Vec::new();
                    let inner = self.nodes(&node.children, true, &mut inner_pool);
                    let sizes: Vec<u8> = inner_pool
                        .iter()
                        .flat_map(|size| size.to_le_bytes())
                        .collect();
                    let chunks_block = Self::cpng(&inner);
                    let sizes_block = Self::cpng(&sizes);
                    for length in [
                        chunks_block.len(),
                        inner.len(),
                        sizes_block.len(),
                        sizes.len(),
                    ] {
                        data.extend_from_slice(&u32::try_from(length).unwrap().to_le_bytes());
                    }
                    data.extend(chunks_block);
                    data.extend(sizes_block);
                } else if node.children.is_empty() {
                    // In particular, stlt is an opaque binary list record.
                    data.extend_from_slice(self.tree.data(self.main, node));
                } else {
                    data.extend(self.nodes(&node.children, indexed, pool));
                }
                data
            } else {
                self.tree.data(self.main, node).to_vec()
            };
            Self::emit(&mut out, &node.id, &data, indexed, pool);
        }
        out
    }
}

fn rebuild_compressed_mcfg(bytes: &[u8], replacement: &[Vec<u8>]) -> Vec<u8> {
    let container = container::detect(bytes).unwrap();
    assert_eq!(
        container.version.0, 14,
        "do not replace the legacy test with an easier version"
    );
    let container::ContainerKind::Zip {
        ref riff_member, ..
    } = container.kind
    else {
        panic!("version14 ZIP expected")
    };
    let main = container::riff_stream(bytes, &container).unwrap();
    let externals = container::external_streams(bytes, &container);
    assert!(
        externals.is_empty(),
        "legacy fixture uses indexed CPng, not redirects"
    );
    let tree = riff::parse_with_externals(&main, externals, 14).unwrap();
    let mut fixture = ConfigurationFixture {
        tree: &tree,
        main: &main,
        replacement,
        matched: 0,
    };
    let rewritten = fixture.nodes(&tree.root.children, false, &mut Vec::new());
    assert_eq!(
        fixture.matched, 1,
        "mutate exactly one complete original opaque configuration"
    );
    let mut body = main[8..12].to_vec();
    body.extend(rewritten);
    let mut riff = b"RIFF".to_vec();
    riff.extend_from_slice(&u32::try_from(body.len()).unwrap().to_le_bytes());
    riff.extend(body);
    assert!(
        top_chunks(&riff)
            .iter()
            .any(|(id, data)| id == b"LIST" && data.starts_with(b"cmpr")),
        "legacy compression must remain present"
    );
    let mut parts = entries(bytes);
    parts
        .iter_mut()
        .find(|(name, _)| name == riff_member)
        .unwrap()
        .1 = riff;
    stored_zip(&parts)
}

fn zip_with_compressed_mcfg(bytes: &[u8], replacement: &[Vec<u8>]) -> Vec<u8> {
    // Calibrate this actual version14 recompressor before every negative.
    let original = mcfg(bytes);
    let positive = rebuild_compressed_mcfg(bytes, &[original.clone()]);
    assert_eq!(mcfg(&positive), original);
    let (doc, _) = open_bytes(&positive, "Recompressed unchanged positive control").unwrap();
    assert_eq!(triple(&doc), (Some(3), Some(1), Some(4)));
    geometry(&doc);
    rebuild_compressed_mcfg(bytes, replacement)
}

#[test]
fn all_writer_versions_preserve_three_independent_private_preferences_without_scaling() {
    for version in 7..=27 {
        let source = scene((Some(3), Some(1), Some(4)));
        let before = source.clone();
        let bytes = document_to_cdr_version(&source, version);
        assert_eq!(source, before, "writing never mutates source");
        let (doc, _) = open_bytes(&bytes, "Owned round trip").unwrap();
        assert_eq!(
            triple(&doc),
            (Some(3), Some(1), Some(4)),
            "version {version}"
        );
        geometry(&doc);
    }
}

#[test]
fn none_zero_all_sdk_codes_and_unknown_u32_are_preserved_exactly_by_owned_extension() {
    let mut cases = vec![
        (None, None, None),
        (Some(0), None, Some(16)),
        (None, Some(0), None),
        (Some(17), Some(u32::MAX), Some(17)),
        (Some(u32::MAX), Some(16), None),
    ];
    cases.extend((0..=16).map(|u| (Some(u), Some((u + 1) % 17), Some((u + 7) % 17))));
    for version in [7, 13, 14, 21, 27] {
        for &expected in &cases {
            let bytes = document_to_cdr_version(&scene(expected), version);
            let (doc, _) = open_bytes(&bytes, "Owned exact options").unwrap();
            assert_eq!(triple(&doc), expected, "version {version}");
            geometry(&doc);
        }
    }
}

#[test]
fn private_json_schema_and_binding_are_independently_checked_on_riff_and_zip() {
    for version in [7, 13, 14, 21, 27] {
        let bytes = document_to_cdr_version(&scene((Some(0), Some(1), Some(4))), version);
        let data = payload(&bytes);
        assert!(data.len() <= 1024);
        let configuration = mcfg(&bytes);
        assert!(configuration.len() <= 65536);
        let value: Value = serde_json::from_slice(&data).unwrap();
        assert_eq!(
            value,
            json!({"owner":"craft-corel.document-units","format":1,"mcfg":{"bytes":configuration.len(),"fnv1a64":fnv(&configuration)},"script_unit":0,"horizontal_unit":1,"vertical_unit":4})
        );
    }
}

#[test]
fn stripping_private_extension_keeps_standard_geometry_and_does_not_infer_macro_or_axis_units() {
    for version in [7, 13, 14, 21, 27] {
        let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), version);
        let stripped = replace_payload(&bytes, &[]);
        let (doc, _) = open_bytes(&stripped, "Standard only").unwrap();
        assert_eq!(
            triple(&doc),
            (None, None, None),
            "no standard unit mapping is demonstrated in this phase"
        );
        geometry(&doc);
        assert_eq!(
            mcfg(&bytes),
            mcfg(&stripped),
            "private data must not edit standard configuration"
        );
    }
}

#[test]
fn malformed_private_schemas_reject_all_three_values_without_losing_base_scene() {
    for version in [13, 14, 27] {
        let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), version);
        let good: Value = serde_json::from_slice(&payload(&bytes)).unwrap();
        let mut bad = Vec::new();
        for key in [
            "owner",
            "format",
            "mcfg",
            "script_unit",
            "horizontal_unit",
            "vertical_unit",
        ] {
            let mut value = good.clone();
            value.as_object_mut().unwrap().remove(key);
            bad.push((
                format!("missing {key}"),
                serde_json::to_vec(&value).unwrap(),
            ));
        }
        for (key, value) in [
            ("owner", json!("other.owner")),
            ("format", json!(2)),
            ("format", json!(1.0)),
            ("extra", json!(true)),
            ("script_unit", json!(-1)),
            ("horizontal_unit", json!(4294967296_u64)),
            ("vertical_unit", json!(1.5)),
            ("script_unit", json!("3")),
            ("horizontal_unit", json!(false)),
            ("vertical_unit", json!([])),
            ("mcfg", json!({"bytes":0,"fnv1a64":"0000000000000000"})),
        ] {
            let mut candidate = good.clone();
            candidate[key] = value;
            bad.push((
                format!("bad {key}"),
                serde_json::to_vec(&candidate).unwrap(),
            ));
        }
        let original = String::from_utf8(payload(&bytes)).unwrap();
        bad.push((
            "duplicate property".into(),
            format!("{{\"script_unit\":16,{}", &original[1..]).into_bytes(),
        ));
        bad.push((
            "truncated JSON".into(),
            original.as_bytes()[..original.len() - 1].to_vec(),
        ));
        bad.push(("oversized JSON".into(), vec![b' '; 1025]));
        bad.push(("invalid UTF-8".into(), vec![0xff, 0xfe]));
        for (reason, value) in bad {
            check_bad(
                &replace_payload(&bytes, &[value]),
                &format!("v{version}: {reason}"),
            );
        }
    }
}

#[test]
fn duplicate_private_records_are_rejected_even_if_byte_identical() {
    for version in [13, 14, 27] {
        let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), version);
        let data = payload(&bytes);
        check_bad(
            &replace_payload(&bytes, &[data.clone(), data]),
            "duplicate private records",
        );
    }
}

#[test]
fn noncanonical_zip_member_aliases_cannot_override_owned_preferences() {
    let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), 27);
    for alias in [
        "Craft-Corel/document-units.json",
        "craft-corel/DOCUMENT-UNITS.JSON",
        "craft-corel\\document-units.json",
        "./craft-corel/document-units.json",
        "craft-corel//document-units.json",
    ] {
        let mut parts = entries(&bytes);
        parts.push((alias.to_owned(), payload(&bytes)));
        check_bad(&stored_zip(&parts), alias);
    }
}

#[test]
fn mixed_zip_and_riff_channels_cannot_override_owned_preferences() {
    let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), 14);
    let container = container::detect(&bytes).unwrap();
    let container::ContainerKind::Zip { riff_member, .. } = container.kind else {
        panic!("zip expected")
    };
    let mut parts = entries(&bytes);
    let main = &mut parts
        .iter_mut()
        .find(|(name, _)| name == &riff_member)
        .unwrap()
        .1;
    let mut chunks = top_chunks(main);
    let before = chunks
        .iter()
        .position(|(id, _)| id == b"sumi")
        .unwrap_or(chunks.len());
    chunks.insert(before, (*b"ccun", payload(&bytes)));
    *main = riff_with_chunks(main, &chunks);
    check_bad(&stored_zip(&parts), "mixed ZIP and RIFF channels");
}

#[test]
fn opaque_standard_configuration_change_invalidates_private_preferences() {
    // Version14 stores its configuration in indexed CPng inside the ZIP.
    // Alter an opaque spare byte, not an interpreted unit or geometry field.
    let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), 14);
    let configuration = mcfg(&bytes);
    let mut changed = configuration.clone();
    *changed.last_mut().unwrap() ^= 1;
    let mutated = zip_with_compressed_mcfg(&bytes, &[changed.clone()]);
    assert_eq!(
        mcfg(&mutated),
        changed,
        "the whole changed configuration must actually reach the reader"
    );
    assert_ne!(mcfg(&mutated), configuration);
    check_bad(
        &mutated,
        "stale private units after opaque MCFG modification",
    );
}

#[test]
fn corrupted_zip_member_crc_rejects_private_units_but_not_standard_geometry() {
    let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), 27);
    let data = payload(&bytes);
    let mut mutated = stored_zip(&entries(&bytes));
    let positions: Vec<_> = mutated
        .windows(data.len())
        .enumerate()
        .filter_map(|(p, w)| (w == data).then_some(p))
        .collect();
    assert_eq!(positions.len(), 1);
    // Whitespace remains valid JSON; only the ZIP integrity check should fail.
    let p = positions[0];
    assert_eq!(mutated[p], b'{');
    let space = data.iter().position(|b| *b == b' ').unwrap_or(data.len());
    if space < data.len() {
        mutated[p + space] = b'\t';
    } else {
        // Changing an ASCII digit preserves a well-formed, otherwise valid
        // schema; without CRC checking it would restore the wrong preference.
        let needle = b"\"script_unit\":3";
        let at = data
            .windows(needle.len())
            .position(|w| w == needle)
            .expect("compact schema fixture");
        mutated[p + at + needle.len() - 1] = b'4';
    }
    check_bad(&mutated, "private member CRC");
}

#[test]
fn duplicate_opaque_standard_configurations_invalidate_private_preferences() {
    let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), 14);
    let configuration = mcfg(&bytes);
    let duplicated = zip_with_compressed_mcfg(&bytes, &[configuration.clone(), configuration]);
    check_bad(&duplicated, "duplicate opaque standard configurations");
}

#[test]
fn oversized_opaque_configuration_is_rejected_even_with_matching_private_binding() {
    let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), 14);
    let mut configuration = mcfg(&bytes);
    configuration.resize(65537, 0);
    let oversized = zip_with_compressed_mcfg(&bytes, &[configuration.clone()]);
    let mut value: Value = serde_json::from_slice(&payload(&bytes)).unwrap();
    value["mcfg"] = json!({"bytes":65537,"fnv1a64":fnv(&configuration)});
    check_bad(
        &replace_payload(&oversized, &[serde_json::to_vec(&value).unwrap()]),
        "oversized opaque configuration with valid binding",
    );
}

#[test]
fn valid_json_at_1024_bytes_is_accepted_but_same_schema_at_1025_is_rejected() {
    for version in [13, 14, 27] {
        let bytes = document_to_cdr_version(&scene((Some(3), Some(1), Some(4))), version);
        let original = payload(&bytes);
        assert!(original.len() < 1024);
        let mut boundary = original.clone();
        boundary.resize(1024, b' ');
        let mut excess = original;
        excess.resize(1025, b' ');
        let expected: Value = serde_json::from_slice(&boundary).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&excess).unwrap(),
            expected,
            "both controls are valid, identical JSON values"
        );
        let accepted = replace_payload(&bytes, &[boundary]);
        let (doc, report) = open_bytes(&accepted, "Private JSON exact budget").unwrap();
        assert_eq!(
            triple(&doc),
            (Some(3), Some(1), Some(4)),
            "version {version}"
        );
        assert!(!report
            .warnings
            .iter()
            .any(|w| w.contains("Craft Corel unit preferences ignored")));
        geometry(&doc);
        check_bad(
            &replace_payload(&bytes, &[excess]),
            "valid identical private JSON exceeds byte budget by one",
        );
    }
}
