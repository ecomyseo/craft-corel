//! Office Open XML spreadsheets (`.xlsx`): the first worksheet's cells as
//! text, for Print Merge data (#377, #378). Read from the public
//! ECMA-376 structure: `xl/workbook.xml` lists the sheets, its
//! relationships part names each sheet's file, `xl/sharedStrings.xml` holds
//! the shared strings, and a sheet's `sheetData` holds rows of cells whose
//! `r` attribute gives the column (`B3`), `t` the type (`s` a shared string
//! index, `inlineStr`, `str`, `b`, or a number) and `v` (or `is/t`) the
//! value.

use std::io::Read;

/// The most bytes one part may have when unpacked.
const MAX_PART: u64 = 64 * 1024 * 1024;

/// The first sheet as rows of cell texts, empty rows and trailing empty
/// cells left out; ragged rows padded to the widest one.
pub fn read_rows(bytes: &[u8]) -> Result<Vec<Vec<String>>, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("not an XLSX file: {e}"))?;
    let part = |zip: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>, name: &str| {
        let mut f = zip.by_name(name).map_err(|_| format!("{name} missing"))?;
        if f.size() > MAX_PART {
            return Err(format!("{name} too large"));
        }
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(|e| e.to_string())?;
        Ok::<String, String>(s)
    };
    let sheet = first_sheet_path(
        &part(&mut zip, "xl/workbook.xml")?,
        &part(&mut zip, "xl/_rels/workbook.xml.rels").unwrap_or_default(),
    )
    .unwrap_or_else(|| "xl/worksheets/sheet1.xml".to_string());
    let shared = match part(&mut zip, "xl/sharedStrings.xml") {
        Ok(xml) => shared_strings(&xml)?,
        Err(_) => Vec::new(),
    };
    sheet_rows(&part(&mut zip, &sheet)?, &shared)
}

/// The first sheet's part name, from the workbook and its relationships.
fn first_sheet_path(workbook: &str, rels: &str) -> Option<String> {
    let wb = roxmltree::Document::parse(workbook).ok()?;
    let sheet = wb.descendants().find(|n| n.has_tag_name("sheet"))?;
    let rid = sheet
        .attributes()
        .find(|a| a.name() == "id")
        .map(|a| a.value().to_string())?;
    let rels = roxmltree::Document::parse(rels).ok()?;
    let target = rels
        .descendants()
        .filter(|n| n.has_tag_name("Relationship"))
        .find(|n| n.attribute("Id") == Some(rid.as_str()))?
        .attribute("Target")?;
    // Targets are relative to `xl/`, or absolute from the package root.
    Some(match target.strip_prefix('/') {
        Some(abs) => abs.to_string(),
        None => format!("xl/{target}"),
    })
}

/// The text of a string item: its `t`, or its runs' `t`s together
/// (phonetic runs left out).
fn item_text(n: roxmltree::Node) -> String {
    let mut s = String::new();
    for t in n.descendants().filter(|d| d.has_tag_name("t")) {
        if t.ancestors().any(|a| a.has_tag_name("rPh")) {
            continue;
        }
        s.push_str(t.text().unwrap_or(""));
    }
    s
}

fn shared_strings(xml: &str) -> Result<Vec<String>, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("sharedStrings.xml: {e}"))?;
    Ok(doc
        .root_element()
        .children()
        .filter(|n| n.has_tag_name("si"))
        .map(item_text)
        .collect())
}

/// The column of a cell reference (`A1` 0, `AB7` 27).
pub fn column_of(r: &str) -> Option<usize> {
    let letters: String = r.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    if letters.is_empty() || letters.len() > 3 {
        return None;
    }
    let mut n = 0usize;
    for c in letters.chars() {
        n = n * 26 + (c.to_ascii_uppercase() as usize - 'A' as usize + 1);
    }
    Some(n - 1)
}

/// A number as a cell shows it plainly: whole numbers without a decimal
/// point, others without trailing zeros.
fn number_text(v: &str) -> String {
    match v.parse::<f64>() {
        Ok(x) if x.is_finite() && x.fract() == 0.0 && x.abs() < 1e15 => format!("{}", x as i64),
        Ok(x) if x.is_finite() => format!("{x}"),
        _ => v.to_string(),
    }
}

fn sheet_rows(xml: &str, shared: &[String]) -> Result<Vec<Vec<String>>, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("worksheet: {e}"))?;
    let Some(data) = doc.descendants().find(|n| n.has_tag_name("sheetData")) else {
        return Ok(Vec::new());
    };
    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in data.children().filter(|n| n.has_tag_name("row")) {
        let mut cells: Vec<String> = Vec::new();
        let mut next = 0usize;
        for c in row.children().filter(|n| n.has_tag_name("c")) {
            let col = c.attribute("r").and_then(column_of).unwrap_or(next);
            next = col + 1;
            if col > 16_383 {
                continue;
            }
            let v = c
                .children()
                .find(|n| n.has_tag_name("v"))
                .and_then(|n| n.text())
                .unwrap_or("");
            let text = match c.attribute("t") {
                Some("s") => v
                    .trim()
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| shared.get(i).cloned())
                    .unwrap_or_default(),
                Some("inlineStr") => c
                    .children()
                    .find(|n| n.has_tag_name("is"))
                    .map(item_text)
                    .unwrap_or_default(),
                Some("b") => (if v.trim() == "1" { "TRUE" } else { "FALSE" }).to_string(),
                Some("str") | Some("e") => v.to_string(),
                _ => number_text(v.trim()),
            };
            if cells.len() <= col {
                cells.resize(col + 1, String::new());
            }
            cells[col] = text;
        }
        while cells.last().is_some_and(|s| s.is_empty()) {
            cells.pop();
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    for r in &mut rows {
        r.resize(width, String::new());
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A workbook made here, as a spreadsheet program writes one.
    fn book(sheet: &str, shared: &str) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let o: zip::write::SimpleFileOptions = Default::default();
            for (name, body) in [
                (
                    "xl/workbook.xml",
                    r#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Cards" sheetId="1" r:id="rId7"/></sheets></workbook>"#,
                ),
                (
                    "xl/_rels/workbook.xml.rels",
                    r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId7" Type="worksheet" Target="worksheets/cards.xml"/></Relationships>"#,
                ),
                ("xl/worksheets/cards.xml", sheet),
                ("xl/sharedStrings.xml", shared),
            ] {
                z.start_file(name, o).unwrap();
                z.write_all(body.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn the_first_sheet_reads_as_rows_of_text() {
        let shared = r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><si><t>Name</t></si><si><t>Photo</t></si><si><r><t>Ana </t></r><r><t>Silva</t></r></si><si><t>ana.jpg</t></si></sst>"#;
        let sheet = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>
            <row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c><c r="C1" t="inlineStr"><is><t>ID</t></is></c></row>
            <row r="2"><c r="A2" t="s"><v>2</v></c><c r="B2" t="s"><v>3</v></c><c r="C2"><v>1042</v></c></row>
            <row r="4"><c r="A4" t="str"><v>Bo</v></c><c r="C4"><v>2.50</v></c><c r="D4"/></row>
        </sheetData></worksheet>"#;
        let rows = read_rows(&book(sheet, shared)).unwrap();
        assert_eq!(
            rows,
            vec![
                vec!["Name".to_string(), "Photo".into(), "ID".into()],
                vec!["Ana Silva".into(), "ana.jpg".into(), "1042".into()],
                vec!["Bo".into(), "".into(), "2.5".into()],
            ]
        );
    }

    #[test]
    fn column_letters_and_bad_input() {
        assert_eq!(column_of("A1"), Some(0));
        assert_eq!(column_of("Z9"), Some(25));
        assert_eq!(column_of("AB7"), Some(27));
        assert_eq!(column_of("12"), None);
        assert!(read_rows(b"not a zip").is_err());
    }
}
