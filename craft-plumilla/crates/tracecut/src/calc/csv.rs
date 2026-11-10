//! CSV text for the price lists: reading what spreadsheets
//! write (comma, semicolon or tab between fields, quoted fields, decimal
//! comma) and writing plain comma-separated text with a decimal point.

/// One list read from CSV: the rows that made sense and how many did not.
#[derive(Debug, Clone, PartialEq)]
pub struct Imported<T> {
    pub items: Vec<T>,
    /// Rows left out (too few fields, a value that is not a number). A
    /// header row is not counted.
    pub skipped: usize,
}

/// The field separator of `text`: the one of tab, semicolon and comma
/// found most often outside quotes on the first line that has any text
/// (on a tie: tab, then semicolon, then comma).
pub fn detect_separator(text: &str) -> char {
    let first = text
        .lines()
        .map(|l| l.trim_start_matches('\u{feff}'))
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    let mut counts = [0usize; 3];
    let mut quoted = false;
    for c in first.chars() {
        match c {
            '"' => quoted = !quoted,
            '\t' if !quoted => counts[0] += 1,
            ';' if !quoted => counts[1] += 1,
            ',' if !quoted => counts[2] += 1,
            _ => {}
        }
    }
    let seps = ['\t', ';', ','];
    let mut best = 2;
    for i in [1, 0] {
        if counts[i] > 0 && counts[i] >= counts[best] {
            best = i;
        }
    }
    seps[best]
}

/// Split CSV text into rows of fields. Quoted fields may hold the
/// separator, line breaks and doubled quotes. Empty lines are dropped,
/// fields are trimmed.
pub fn parse(text: &str) -> Vec<Vec<String>> {
    let sep = detect_separator(text);
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = text.chars().peekable();
    let end_row = |row: &mut Vec<String>, rows: &mut Vec<Vec<String>>| {
        let r = std::mem::take(row);
        if r.iter().any(|f| !f.is_empty()) {
            rows.push(r);
        }
    };
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '\r' => {}
            '\n' => {
                row.push(std::mem::take(&mut field).trim().to_string());
                end_row(&mut row, &mut rows);
            }
            c if c == sep => row.push(std::mem::take(&mut field).trim().to_string()),
            c => field.push(c),
        }
    }
    row.push(field.trim().to_string());
    end_row(&mut row, &mut rows);
    rows
}

/// A number as spreadsheets write it: `1234.5`, `1234,5`, `1 234,50`,
/// `1.234,50` or `1,234.50`. `None` for anything else, infinities and
/// not-a-number included.
pub fn parse_number(s: &str) -> Option<f64> {
    let mut t: String = s
        .trim()
        .chars()
        .filter(|c| !matches!(c, ' ' | '\u{a0}' | '\u{202f}' | '\''))
        .collect();
    let dot = t.rfind('.');
    let comma = t.rfind(',');
    t = match (dot, comma) {
        (Some(d), Some(c)) if c > d => t.replace('.', "").replace(',', "."),
        (Some(_), Some(_)) => t.replace(',', ""),
        (None, Some(_)) if t.matches(',').count() == 1 => t.replace(',', "."),
        (None, Some(_)) => t.replace(',', ""),
        _ => t,
    };
    if t.is_empty() || !t.chars().all(|c| c.is_ascii_digit() || "+-.eE".contains(c)) {
        return None;
    }
    t.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// A field, quoted when it holds a comma, a quote or a line break.
pub fn quote(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r', ';', '\t']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Rows as comma-separated text, one line per row ending in CR LF.
pub fn write(rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    for r in rows {
        let line: Vec<String> = r.iter().map(|f| quote(f)).collect();
        out.push_str(&line.join(","));
        out.push_str("\r\n");
    }
    out
}

/// A number for CSV: the shortest text that reads back the same.
pub fn number(v: f64) -> String {
    if v.is_finite() {
        format!("{v}")
    } else {
        "0".into()
    }
}

/// Read the rows of a list: `read` turns a row into an item or `None`.
/// A first row that does not read is taken as the header and not
/// counted as skipped.
pub(crate) fn read_rows<T>(text: &str, read: impl Fn(&[String]) -> Option<T>) -> Imported<T> {
    let mut items = Vec::new();
    let mut skipped = 0;
    for (i, row) in parse(text).iter().enumerate() {
        if row.first().is_some_and(|f| f.starts_with('#')) {
            continue;
        }
        match read(row) {
            Some(item) => items.push(item),
            None if i == 0 => {}
            None => skipped += 1,
        }
    }
    Imported { items, skipped }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separators_are_found() {
        assert_eq!(detect_separator("a,b,c\n"), ',');
        assert_eq!(detect_separator("a;b;\"c,d\"\n"), ';');
        assert_eq!(detect_separator("a\tb\n"), '\t');
        // Decimal commas do not outvote the semicolons.
        assert_eq!(detect_separator("Vinyl, white;roll;8,50"), ';');
        assert_eq!(detect_separator(""), ',');
    }

    #[test]
    fn quoted_fields_keep_separators_quotes_and_line_breaks() {
        let rows = parse("\u{feff}name,price\r\n\"A, \"\"big\"\"\nsign\", 12.5 \r\n\r\n,\n");
        assert_eq!(
            rows,
            vec![
                vec!["name".to_string(), "price".to_string()],
                vec!["A, \"big\"\nsign".to_string(), "12.5".to_string()],
            ]
        );
    }

    #[test]
    fn numbers_with_decimal_commas_and_thousands() {
        assert_eq!(parse_number("12.5"), Some(12.5));
        assert_eq!(parse_number("12,5"), Some(12.5));
        assert_eq!(parse_number("1 234,50"), Some(1234.5));
        assert_eq!(parse_number("1.234,50"), Some(1234.5));
        assert_eq!(parse_number("1,234.50"), Some(1234.5));
        assert_eq!(parse_number("1,234,567"), Some(1_234_567.0));
        assert_eq!(parse_number("-3"), Some(-3.0));
        assert_eq!(parse_number("inf"), None);
        assert_eq!(parse_number("NaN"), None);
        assert_eq!(parse_number("1e999"), None);
        assert_eq!(parse_number("12 EUR"), None);
        assert_eq!(parse_number(""), None);
    }

    #[test]
    fn written_text_reads_back() {
        let rows = vec![
            vec!["Name, with comma".to_string(), number(0.55)],
            vec!["Say \"hi\"".to_string(), number(f64::NAN)],
        ];
        let text = write(&rows);
        assert!(text.ends_with("\r\n"));
        assert_eq!(
            parse(&text),
            vec![
                vec!["Name, with comma".to_string(), "0.55".to_string()],
                vec!["Say \"hi\"".to_string(), "0".to_string()],
            ]
        );
    }

    #[test]
    fn a_header_is_not_a_skipped_row() {
        let read = |r: &[String]| r.get(1).and_then(|f| parse_number(f));
        let got = read_rows("name,value\nA,1\nB,x\n# note\nC,3\n", read);
        assert_eq!(got.items, vec![1.0, 3.0]);
        assert_eq!(got.skipped, 1);
        let got = read_rows("", read);
        assert!(got.items.is_empty() && got.skipped == 0);
    }

    #[test]
    fn an_unclosed_quote_does_not_panic() {
        let rows = parse("\"open,1\n2,3");
        assert_eq!(rows.len(), 1);
    }
}
