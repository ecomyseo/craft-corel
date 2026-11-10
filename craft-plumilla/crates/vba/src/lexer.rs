//! Tokens of VBA source: identifiers (keywords included, matched by the
//! parser without regard to case), numbers, strings, date literals,
//! operators and statement ends. Comments, line continuations and type
//! suffixes (`%`, `&`, `!`, `#`, `$`, `@`, `^`) are dropped here.

use crate::CompileError;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Num(f64),
    Str(String),
    /// A date literal as a serial (days since 1899-12-30).
    Date(f64),
    Op(&'static str),
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: u32,
    /// First token of a physical line (labels must start a line).
    pub line_start: bool,
    /// Whitespace came right before it.
    pub space_before: bool,
}

const OPS2: [&str; 4] = ["<=", ">=", "<>", ":="];
const OPS1: [&str; 18] = [
    "+", "-", "*", "/", "\\", "^", "&", "=", "<", ">", "(", ")", ",", ".", "!", ":", ";", "#",
];

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub fn lex(module: &str, lines: &[String]) -> Result<Vec<Token>, CompileError> {
    let mut out: Vec<Token> = Vec::new();
    let mut li = 0usize;
    let mut stmt_start = true;
    // The line before ended with a continuation: its first token is glued.
    let mut glued = false;
    while li < lines.len() {
        let cs: Vec<char> = lines[li].chars().collect();
        let line_no = li as u32 + 1;
        let mut i = 0usize;
        let mut first = !glued;
        let mut continued = false;
        let mut glue_first = glued;
        let err = |msg: String| CompileError {
            module: module.to_string(),
            line: line_no,
            message: msg,
        };
        while i < cs.len() {
            let c = cs[i];
            let space = i > 0 && cs[i - 1].is_whitespace() && !glue_first;
            if c.is_whitespace() {
                i += 1;
                continue;
            }
            // Line continuation: " _" at the end of the line.
            if c == '_'
                && (i == 0 || cs[i - 1].is_whitespace())
                && cs[i + 1..].iter().all(|c| c.is_whitespace())
            {
                continued = true;
                break;
            }
            let push = |out: &mut Vec<Token>, tok: Tok, first: &mut bool| {
                out.push(Token {
                    tok,
                    line: line_no,
                    line_start: *first,
                    space_before: space,
                });
                *first = false;
            };
            if !c.is_whitespace() {
                glue_first = false;
            }
            if c == '\'' {
                // A comment; one ending in " _" goes on to the next line.
                let rest: String = cs[i..].iter().collect();
                if rest.trim_end().ends_with(" _") {
                    li += 1;
                    while li < lines.len() && lines[li].trim_end().ends_with(" _") {
                        li += 1;
                    }
                }
                break;
            }
            if c == '"' {
                let mut s = String::new();
                i += 1;
                loop {
                    if i >= cs.len() {
                        return Err(err("Unterminated string".into()));
                    }
                    if cs[i] == '"' {
                        if i + 1 < cs.len() && cs[i + 1] == '"' {
                            s.push('"');
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    s.push(cs[i]);
                    i += 1;
                }
                push(&mut out, Tok::Str(s), &mut first);
                stmt_start = false;
                continue;
            }
            if c == '[' {
                // [Bracketed identifier]
                let end = cs[i..].iter().position(|&c| c == ']').map(|p| p + i);
                let Some(end) = end else {
                    return Err(err("Missing ]".into()));
                };
                let name: String = cs[i + 1..end].iter().collect();
                push(&mut out, Tok::Ident(name), &mut first);
                i = end + 1;
                stmt_start = false;
                continue;
            }
            if is_ident_start(c) {
                let st = i;
                while i < cs.len() && is_ident_char(cs[i]) {
                    i += 1;
                }
                let name: String = cs[st..i].iter().collect();
                // A type suffix right after the name (not a bang member access).
                if i < cs.len() && "%&!#$@^".contains(cs[i]) {
                    let next = cs.get(i + 1).copied();
                    let bang_member = cs[i] == '!' && next.is_some_and(is_ident_start);
                    let hex = cs[i] == '&'
                        && next.is_some_and(|n| n == 'H' || n == 'h' || n == 'O' || n == 'o');
                    if !bang_member && !hex {
                        i += 1;
                    }
                }
                if stmt_start && name.eq_ignore_ascii_case("rem") {
                    break;
                }
                push(&mut out, Tok::Ident(name), &mut first);
                stmt_start = false;
                continue;
            }
            if c.is_ascii_digit() || (c == '.' && cs.get(i + 1).is_some_and(|d| d.is_ascii_digit()))
            {
                let st = i;
                while i < cs.len() && cs[i].is_ascii_digit() {
                    i += 1;
                }
                if i < cs.len() && cs[i] == '.' {
                    i += 1;
                    while i < cs.len() && cs[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                if i < cs.len() && matches!(cs[i], 'e' | 'E' | 'd' | 'D') {
                    let mut j = i + 1;
                    if j < cs.len() && (cs[j] == '+' || cs[j] == '-') {
                        j += 1;
                    }
                    if j < cs.len() && cs[j].is_ascii_digit() {
                        while j < cs.len() && cs[j].is_ascii_digit() {
                            j += 1;
                        }
                        i = j;
                    }
                }
                let text: String = cs[st..i]
                    .iter()
                    .map(|&c| if c == 'd' || c == 'D' { 'e' } else { c })
                    .collect();
                if i < cs.len() && "%&!#@^".contains(cs[i]) {
                    i += 1;
                }
                let v: f64 = text
                    .parse()
                    .map_err(|_| err(format!("Bad number {text}")))?;
                push(&mut out, Tok::Num(v), &mut first);
                stmt_start = false;
                continue;
            }
            if c == '&'
                && cs
                    .get(i + 1)
                    .is_some_and(|n| matches!(n, 'H' | 'h' | 'O' | 'o'))
                && cs.get(i + 2).is_some_and(|d| d.is_ascii_hexdigit())
            {
                let hex = matches!(cs[i + 1], 'H' | 'h');
                i += 2;
                let st = i;
                while i < cs.len() && cs[i].is_ascii_hexdigit() {
                    i += 1;
                }
                let digits: String = cs[st..i].iter().collect();
                let long = i < cs.len() && cs[i] == '&';
                if i < cs.len() && "%&^".contains(cs[i]) {
                    i += 1;
                }
                let raw = u64::from_str_radix(&digits, if hex { 16 } else { 8 })
                    .map_err(|_| err(format!("Bad number &{digits}")))?;
                // Integer literals wrap like the 16-bit and 32-bit types.
                let v = if !long && raw <= 0xFFFF {
                    raw as u16 as i16 as f64
                } else if raw <= 0xFFFF_FFFF {
                    raw as u32 as i32 as f64
                } else {
                    raw as f64
                };
                push(&mut out, Tok::Num(v), &mut first);
                stmt_start = false;
                continue;
            }
            if c == '#' {
                // A date literal #...# on this line.
                if let Some(end) = cs[i + 1..]
                    .iter()
                    .position(|&c| c == '#')
                    .map(|p| p + i + 1)
                {
                    let inner: String = cs[i + 1..end].iter().collect();
                    if let Some(serial) = parse_date_literal(&inner) {
                        push(&mut out, Tok::Date(serial), &mut first);
                        i = end + 1;
                        stmt_start = false;
                        continue;
                    }
                }
            }
            let two: String = cs[i..(i + 2).min(cs.len())].iter().collect();
            if let Some(op) = OPS2.iter().find(|o| **o == two) {
                push(&mut out, Tok::Op(op), &mut first);
                i += 2;
                stmt_start = false;
                continue;
            }
            let one = c.to_string();
            if let Some(op) = OPS1.iter().find(|o| **o == one) {
                push(&mut out, Tok::Op(op), &mut first);
                i += 1;
                stmt_start = *op == ":";
                continue;
            }
            return Err(err(format!("Unexpected character '{c}'")));
        }
        li += 1;
        glued = continued;
        if !continued {
            out.push(Token {
                tok: Tok::Newline,
                line: line_no,
                line_start: false,
                space_before: false,
            });
            stmt_start = true;
        }
    }
    out.push(Token {
        tok: Tok::Eof,
        line: lines.len() as u32 + 1,
        line_start: false,
        space_before: false,
    });
    Ok(out)
}

const MONTHS: [&str; 12] = [
    "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
];

/// Days from 1899-12-30 to a civil date.
pub fn serial_from_ymd(y: i64, m: i64, d: i64) -> f64 {
    // Days from the civil algorithm (proleptic Gregorian).
    let (y, m) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * m + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468; // days since 1970-01-01
    (days + 25569) as f64
}

/// `#1/31/2020#`, `#2020-01-31#`, `#Jan 31, 2020#`, with an optional
/// time (`13:45`, `1:45:10 PM`), or a time alone.
pub fn parse_date_literal(s: &str) -> Option<f64> {
    let t = s.trim();
    if t.is_empty() || !t.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    if !t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || " /-:,.".contains(c))
    {
        return None;
    }
    let lower = t.to_ascii_lowercase();
    let (pm, am) = (lower.ends_with("pm"), lower.ends_with("am"));
    let core = if pm || am {
        lower[..lower.len() - 2].trim()
    } else {
        lower.as_str()
    };
    let mut date_part = core;
    let mut time_part = "";
    if let Some(pos) = core.find(':') {
        // The time is the last space-separated group with a colon.
        let start = core[..pos].rfind(' ').map(|p| p + 1).unwrap_or(0);
        date_part = core[..start].trim();
        time_part = core[start..].trim();
    }
    let mut serial = 0.0;
    if !date_part.is_empty() {
        let parts: Vec<&str> = date_part
            .split(['/', '-', ' ', ','])
            .filter(|p| !p.is_empty())
            .collect();
        if parts.len() != 3 {
            return None;
        }
        let month_name = |p: &str| {
            MONTHS
                .iter()
                .position(|m| p.starts_with(m))
                .map(|i| i as i64 + 1)
        };
        let (y, m, d) = if let Some(m) = month_name(parts[0]) {
            (parts[2].parse().ok()?, m, parts[1].parse().ok()?)
        } else if let Some(m) = month_name(parts[1]) {
            (parts[2].parse().ok()?, m, parts[0].parse().ok()?)
        } else if parts[0].len() == 4 {
            (
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            )
        } else {
            (
                parts[2].parse().ok()?,
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
            )
        };
        let y: i64 = if y < 30 {
            2000 + y
        } else if y < 100 {
            1900 + y
        } else {
            y
        };
        if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
            return None;
        }
        serial = serial_from_ymd(y, m, d);
    }
    if !time_part.is_empty() {
        let tp: Vec<&str> = time_part.split(':').collect();
        let mut h: f64 = tp.first()?.trim().parse().ok()?;
        let mi: f64 = tp.get(1).map_or(Some(0.0), |v| v.trim().parse().ok())?;
        let se: f64 = tp.get(2).map_or(Some(0.0), |v| v.trim().parse().ok())?;
        if pm && h < 12.0 {
            h += 12.0;
        }
        if am && h == 12.0 {
            h = 0.0;
        }
        serial += (h * 3600.0 + mi * 60.0 + se) / 86400.0;
    } else if date_part.is_empty() {
        return None;
    }
    Some(serial)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str) -> Vec<Tok> {
        let lines: Vec<String> = s.lines().map(str::to_string).collect();
        lex("M", &lines)
            .unwrap()
            .into_iter()
            .map(|t| t.tok)
            .collect()
    }

    #[test]
    fn suffixes_hex_continuations_and_comments() {
        let t = toks("x% = &HFF + &HFFFF + &H10000 ' note\nb$ = a!c & _\n \"q\"\"r\"");
        assert_eq!(
            t,
            vec![
                Tok::Ident("x".into()),
                Tok::Op("="),
                Tok::Num(255.0),
                Tok::Op("+"),
                Tok::Num(-1.0),
                Tok::Op("+"),
                Tok::Num(65536.0),
                Tok::Newline,
                Tok::Ident("b".into()),
                Tok::Op("="),
                Tok::Ident("a".into()),
                Tok::Op("!"),
                Tok::Ident("c".into()),
                Tok::Op("&"),
                Tok::Str("q\"r".into()),
                Tok::Newline,
                Tok::Eof,
            ]
        );
    }

    #[test]
    fn dates_and_rem() {
        assert_eq!(parse_date_literal("1/31/2020"), Some(43861.0));
        assert_eq!(parse_date_literal("2020-01-31 12:00"), Some(43861.5));
        assert_eq!(parse_date_literal("6:00 PM"), Some(0.75));
        assert_eq!(parse_date_literal("1"), None);
        let t = toks("Rem a comment\nd = #1/2/2000#");
        assert_eq!(t[0], Tok::Newline);
        assert_eq!(t[3], Tok::Date(36527.0));
    }
}
