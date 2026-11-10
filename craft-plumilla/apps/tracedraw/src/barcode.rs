//! Barcode generators and layout for Object > Insert > Barcode: Codabar,
//! Code 39, Code 128 (auto subsets B and C), EAN-13, EAN-8, UPC-A,
//! UPC-E and Interleaved 2 of 5, with 2- and 5-digit add-ons for the EAN
//! and UPC symbols. [`encode_symbol`] gives the bars and spaces;
//! [`build`] lays them out in millimetres with the human-readable text.
//! See docs/behavior/barcode.md.

use tracedraw_core::geometry::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbology {
    Codabar,
    Code39,
    Code128,
    Ean13,
    Ean8,
    UpcA,
    UpcE,
    Interleaved2of5,
}

impl Symbology {
    pub const ALL: [Symbology; 8] = [
        Symbology::Codabar,
        Symbology::Code39,
        Symbology::Code128,
        Symbology::Ean13,
        Symbology::Ean8,
        Symbology::UpcA,
        Symbology::UpcE,
        Symbology::Interleaved2of5,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Symbology::Codabar => "Codabar",
            Symbology::Code39 => "Code 39",
            Symbology::Code128 => "Code 128",
            Symbology::Ean13 => "EAN-13",
            Symbology::Ean8 => "EAN-8",
            Symbology::UpcA => "UPC-A",
            Symbology::UpcE => "UPC-E",
            Symbology::Interleaved2of5 => "Interleaved 2 of 5",
        }
    }

    /// The i18n key of the hint telling what the data may hold.
    pub fn hint_key(self) -> &'static str {
        match self {
            Symbology::Codabar => "barcode.hint_codabar",
            Symbology::Code39 => "barcode.hint_code39",
            Symbology::Code128 => "barcode.hint_code128",
            Symbology::Ean13 => "barcode.hint_ean13",
            Symbology::Ean8 => "barcode.hint_ean8",
            Symbology::UpcA => "barcode.hint_upca",
            Symbology::UpcE => "barcode.hint_upce",
            Symbology::Interleaved2of5 => "barcode.hint_i2of5",
        }
    }

    /// Sample data the wizard starts with for this symbology.
    pub fn sample(self) -> &'static str {
        match self {
            Symbology::Codabar => "1234567890123",
            Symbology::Code39 => "TRACEDRAW-2026",
            Symbology::Code128 => "Craft Plumilla 2026",
            Symbology::Ean13 => "590123412345",
            Symbology::Ean8 => "9638507",
            Symbology::UpcA => "03600029145",
            Symbology::UpcE => "0425261",
            Symbology::Interleaved2of5 => "12345670",
        }
    }

    /// Bars of two widths (narrow and wide), so the wide to narrow ratio
    /// applies.
    pub fn two_widths(self) -> bool {
        matches!(
            self,
            Symbology::Codabar | Symbology::Code39 | Symbology::Interleaved2of5
        )
    }

    /// An optional check character can be added (the advanced settings).
    pub fn optional_check(self) -> bool {
        matches!(self, Symbology::Code39 | Symbology::Interleaved2of5)
    }

    /// EAN and UPC symbols: guard bars, digit groups, add-ons.
    pub fn retail(self) -> bool {
        matches!(
            self,
            Symbology::Ean13 | Symbology::Ean8 | Symbology::UpcA | Symbology::UpcE
        )
    }

    /// Which text options apply to this symbology.
    pub fn supports(self, option: TextOption, has_check: bool) -> bool {
        match option {
            TextOption::CenterSysCheck => {
                matches!(self, Symbology::Ean13 | Symbology::UpcA | Symbology::UpcE)
            }
            TextOption::AddOnText | TextOption::AddOnBottom => self.retail(),
            TextOption::QuietMarks => matches!(self, Symbology::Ean13 | Symbology::Ean8),
            TextOption::TextAbove => true,
            TextOption::FactData => self == Symbology::Code39 && has_check,
            TextOption::Asterisks => self == Symbology::Code39,
            TextOption::StartStop => self == Symbology::Codabar,
        }
    }
}

/// The text options of the wizard's last step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOption {
    CenterSysCheck,
    AddOnText,
    AddOnBottom,
    QuietMarks,
    TextAbove,
    FactData,
    Asterisks,
    StartStop,
}

/// Which part of the symbol a bar or space belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Main,
    /// Guard bars of EAN and UPC symbols, longer than the others.
    Guard,
    /// The 2- or 5-digit add-on after an EAN or UPC symbol.
    AddOn,
}

/// One bar or space: `units` narrow modules, or wide (the ratio times a
/// narrow module) in two-width symbologies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Elem {
    pub bar: bool,
    pub units: u8,
    pub wide: bool,
    pub part: Part,
}

impl Elem {
    fn modules(bar: bool, units: u8, part: Part) -> Self {
        Elem {
            bar,
            units,
            wide: false,
            part,
        }
    }

    fn two(bar: bool, wide: bool) -> Self {
        Elem {
            bar,
            units: 1,
            wide,
            part: Part::Main,
        }
    }

    /// Width in narrow modules.
    pub fn width(&self, ratio: f64) -> f64 {
        if self.wide {
            ratio
        } else {
            self.units as f64
        }
    }
}

/// An encoded symbol: its bars and spaces and the characters a reader
/// sees, split as the human-readable text groups them.
#[derive(Debug, Clone, PartialEq)]
pub struct Encoded {
    pub elems: Vec<Elem>,
    /// The data with the check character, as encoded (start and stop
    /// characters of Codabar included).
    pub data: String,
    /// The check character added, if any.
    pub check: Option<char>,
    /// EAN and UPC: the digits (system digit and check digit included).
    pub digits: String,
    pub addon: String,
}

/// Encode `data` (for EAN and UPC, optionally followed by a space and a
/// 2- or 5-digit add-on); `check` adds the optional check character of
/// Code 39 and Interleaved 2 of 5. Errors are i18n keys.
pub fn encode_symbol(sym: Symbology, data: &str, check: bool) -> Result<Encoded, &'static str> {
    let data = data.trim();
    let (main, addon) = if sym.retail() {
        match data.split_once(' ') {
            Some((m, a)) => (m.trim(), a.trim()),
            None => (data, ""),
        }
    } else {
        (data, "")
    };
    let modular = |modules: Vec<u8>, guards: &[(usize, usize)]| -> Vec<Elem> {
        runs(&modules)
            .into_iter()
            .map(|(start, len, bar)| {
                let guard = guards.iter().any(|&(a, b)| start >= a && start < b);
                Elem::modules(bar, len as u8, if guard { Part::Guard } else { Part::Main })
            })
            .collect()
    };
    let mut out = match sym {
        Symbology::Code128 => Encoded {
            elems: modular(code128(main)?, &[]),
            data: main.to_string(),
            check: None,
            digits: String::new(),
            addon: String::new(),
        },
        Symbology::Ean13 | Symbology::UpcA => {
            let input = if sym == Symbology::UpcA {
                if !(11..=12).contains(&main.len()) {
                    return Err("status.barcode_invalid");
                }
                format!("0{main}")
            } else {
                main.to_string()
            };
            let modules = ean13(&input)?;
            let full = full_ean(&input, 12);
            let digits = if sym == Symbology::UpcA {
                full[1..].to_string()
            } else {
                full
            };
            Encoded {
                elems: modular(modules, &[(0, 3), (45, 50), (92, 95)]),
                data: digits.clone(),
                check: digits.chars().last(),
                digits,
                addon: String::new(),
            }
        }
        Symbology::Ean8 => {
            let modules = ean8(main)?;
            let digits = full_ean(main, 7);
            Encoded {
                elems: modular(modules, &[(0, 3), (31, 36), (64, 67)]),
                data: digits.clone(),
                check: digits.chars().last(),
                digits,
                addon: String::new(),
            }
        }
        Symbology::UpcE => {
            let (modules, digits) = upce(main)?;
            Encoded {
                elems: modular(modules, &[(0, 3), (45, 51)]),
                data: digits.clone(),
                check: digits.chars().last(),
                digits,
                addon: String::new(),
            }
        }
        Symbology::Codabar => codabar(main)?,
        Symbology::Code39 => code39(main, check)?,
        Symbology::Interleaved2of5 => interleaved(main, check)?,
    };
    if !addon.is_empty() {
        let modules = ean_addon(addon)?;
        // The gap between the symbol and its add-on.
        out.elems.push(Elem::modules(false, 9, Part::AddOn));
        out.elems.extend(
            runs(&modules)
                .into_iter()
                .map(|(_, len, bar)| Elem::modules(bar, len as u8, Part::AddOn)),
        );
        out.addon = addon.to_string();
    }
    Ok(out)
}

/// Runs of equal modules: (start, length, bar).
fn runs(modules: &[u8]) -> Vec<(usize, usize, bool)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < modules.len() {
        let start = i;
        while i < modules.len() && modules[i] == modules[start] {
            i += 1;
        }
        out.push((start, i - start, modules[start] == 1));
    }
    out
}

/// The EAN digits with the check digit (`n` digits before it).
fn full_ean(text: &str, n: usize) -> String {
    match digits(text) {
        Some(d) if d.len() == n => format!("{text}{}", ean_check(&d)),
        _ => text.to_string(),
    }
}

/// Encode `text` as modules (1 = bar, 0 = space) for the single-width
/// symbologies; an error message key otherwise.
pub fn encode(sym: Symbology, text: &str) -> Result<Vec<u8>, &'static str> {
    match sym {
        Symbology::Code128 => code128(text),
        Symbology::Ean13 => ean13(text),
        Symbology::Ean8 => ean8(text),
        _ => Err("status.barcode_invalid"),
    }
}

// ----- Code 128 ------------------------------------------------------------------

/// Bar widths of the 107 Code 128 symbols (each 6 widths, except stop with 7).
const CODE128_WIDTHS: [&str; 107] = [
    "212222", "222122", "222221", "121223", "121322", "131222", "122213", "122312", "132212",
    "221213", "221312", "231212", "112232", "122132", "122231", "113222", "123122", "123221",
    "223211", "221132", "221231", "213212", "223112", "312131", "311222", "321122", "321221",
    "312212", "322112", "322211", "212123", "212321", "232121", "111323", "131123", "131321",
    "112313", "132113", "132311", "211313", "231113", "231311", "112133", "112331", "132131",
    "113123", "113321", "133121", "313121", "211331", "231131", "213113", "213311", "213131",
    "311123", "311321", "331121", "312113", "312311", "332111", "314111", "221411", "431111",
    "111224", "111422", "121124", "121421", "141122", "141221", "112214", "112412", "122114",
    "122411", "142112", "142211", "241211", "221114", "413111", "241112", "134111", "111242",
    "121142", "121241", "114212", "124112", "124211", "411212", "421112", "421211", "212141",
    "214121", "412121", "111143", "111341", "131141", "114113", "114311", "411113", "411311",
    "113141", "114131", "311141", "411131", "211412", "211214", "211232", "2331112",
];

const START_B: usize = 104;
const START_C: usize = 105;
const CODE_B: usize = 100;
const CODE_C: usize = 99;
const STOP: usize = 106;

fn code128(text: &str) -> Result<Vec<u8>, &'static str> {
    if text.is_empty() || !text.bytes().all(|b| (32..127).contains(&b)) {
        return Err("status.barcode_invalid");
    }
    let bytes = text.as_bytes();
    // Choose subsets: runs of 4+ digits (or all digits with even length) go to C.
    let mut codes: Vec<usize> = Vec::new();
    let mut i = 0;
    let digits_from =
        |i: usize| -> usize { bytes[i..].iter().take_while(|b| b.is_ascii_digit()).count() };
    let mut in_c = false;
    let all_digits_even = bytes.iter().all(|b| b.is_ascii_digit()) && bytes.len().is_multiple_of(2);
    if all_digits_even || digits_from(0) >= 4 {
        codes.push(START_C);
        in_c = true;
    } else {
        codes.push(START_B);
    }
    while i < bytes.len() {
        let run = digits_from(i);
        if in_c {
            if run >= 2 {
                let v = (bytes[i] - b'0') as usize * 10 + (bytes[i + 1] - b'0') as usize;
                codes.push(v);
                i += 2;
                continue;
            }
            codes.push(CODE_B);
            in_c = false;
        } else if run >= 4 && run % 2 == 0 || run >= 6 {
            codes.push(CODE_C);
            in_c = true;
            continue;
        }
        codes.push((bytes[i] - 32) as usize);
        i += 1;
    }
    // Checksum: start value + sum(position * value) mod 103.
    let mut sum = codes[0];
    for (k, c) in codes.iter().enumerate().skip(1) {
        sum += k * c;
    }
    codes.push(sum % 103);
    codes.push(STOP);
    let mut out = Vec::new();
    for c in codes {
        let mut bar = true;
        for w in CODE128_WIDTHS[c].bytes() {
            let n = (w - b'0') as usize;
            out.extend(std::iter::repeat_n(u8::from(bar), n));
            bar = !bar;
        }
    }
    Ok(out)
}

// ----- EAN -----------------------------------------------------------------------

const EAN_L: [&str; 10] = [
    "0001101", "0011001", "0010011", "0111101", "0100011", "0110001", "0101111", "0111011",
    "0110111", "0001011",
];
const EAN_G: [&str; 10] = [
    "0100111", "0110011", "0011011", "0100001", "0011101", "0111001", "0000101", "0010001",
    "0001001", "0010111",
];
const EAN_R: [&str; 10] = [
    "1110010", "1100110", "1101100", "1000010", "1011100", "1001110", "1010000", "1000100",
    "1001000", "1110100",
];
/// Parity pattern for the left half of EAN-13, indexed by the first digit (L = 0, G = 1).
const EAN13_PARITY: [&str; 10] = [
    "000000", "001011", "001101", "001110", "010011", "011001", "011100", "010101", "010110",
    "011010",
];

fn digits(text: &str) -> Option<Vec<u8>> {
    text.chars()
        .map(|c| c.to_digit(10).map(|d| d as u8))
        .collect()
}

/// EAN check digit for the leading digits (12 for EAN-13, 7 for EAN-8).
pub fn ean_check(d: &[u8]) -> u8 {
    let mut sum = 0u32;
    for (i, v) in d.iter().rev().enumerate() {
        sum += *v as u32 * if i % 2 == 0 { 3 } else { 1 };
    }
    ((10 - sum % 10) % 10) as u8
}

fn push_pattern(out: &mut Vec<u8>, p: &str) {
    out.extend(p.bytes().map(|b| b - b'0'));
}

fn ean13(text: &str) -> Result<Vec<u8>, &'static str> {
    let mut d = digits(text).ok_or("status.barcode_invalid")?;
    match d.len() {
        12 => d.push(ean_check(&d)),
        13 if ean_check(&d[..12]) == d[12] => {}
        _ => return Err("status.barcode_invalid"),
    }
    let mut out = Vec::new();
    push_pattern(&mut out, "101");
    let parity = EAN13_PARITY[d[0] as usize].as_bytes();
    for (k, v) in d[1..7].iter().enumerate() {
        let table = if parity[k] == b'0' { EAN_L } else { EAN_G };
        push_pattern(&mut out, table[*v as usize]);
    }
    push_pattern(&mut out, "01010");
    for v in &d[7..13] {
        push_pattern(&mut out, EAN_R[*v as usize]);
    }
    push_pattern(&mut out, "101");
    Ok(out)
}

fn ean8(text: &str) -> Result<Vec<u8>, &'static str> {
    let mut d = digits(text).ok_or("status.barcode_invalid")?;
    match d.len() {
        7 => d.push(ean_check(&d)),
        8 if ean_check(&d[..7]) == d[7] => {}
        _ => return Err("status.barcode_invalid"),
    }
    let mut out = Vec::new();
    push_pattern(&mut out, "101");
    for v in &d[..4] {
        push_pattern(&mut out, EAN_L[*v as usize]);
    }
    push_pattern(&mut out, "01010");
    for v in &d[4..8] {
        push_pattern(&mut out, EAN_R[*v as usize]);
    }
    push_pattern(&mut out, "101");
    Ok(out)
}

// ----- Two-width symbologies -------------------------------------------------

/// Elements from a wide/narrow pattern ('1' wide), alternating from a bar,
/// then a narrow space between characters when `gap`.
fn push_two(out: &mut Vec<Elem>, pattern: &str, gap: bool) {
    for (i, w) in pattern.bytes().enumerate() {
        out.push(Elem::two(i % 2 == 0, w == b'1'));
    }
    if gap {
        out.push(Elem::two(false, false));
    }
}

const CODABAR: [(char, &str); 20] = [
    ('0', "0000011"),
    ('1', "0000110"),
    ('2', "0001001"),
    ('3', "1100000"),
    ('4', "0010010"),
    ('5', "1000010"),
    ('6', "0100001"),
    ('7', "0100100"),
    ('8', "0110000"),
    ('9', "1001000"),
    ('-', "0001100"),
    ('$', "0011000"),
    (':', "1000101"),
    ('/', "1010001"),
    ('.', "1010100"),
    ('+', "0010101"),
    ('A', "0011010"),
    ('B', "0101001"),
    ('C', "0001011"),
    ('D', "0001110"),
];

/// Codabar: digits and - $ : / . +, up to 30, between start and stop
/// characters A to D (A and B when the data gives none).
fn codabar(text: &str) -> Result<Encoded, &'static str> {
    let upper = text.to_ascii_uppercase();
    let is_ss = |c: char| matches!(c, 'A'..='D');
    let chars: Vec<char> = upper.chars().collect();
    let (start, body, stop) = match (chars.first(), chars.last()) {
        (Some(&a), Some(&b)) if chars.len() >= 2 && is_ss(a) && is_ss(b) => {
            (a, chars[1..chars.len() - 1].to_vec(), b)
        }
        _ => ('A', chars.clone(), 'B'),
    };
    if body.is_empty() || body.len() > 30 || body.iter().any(|c| is_ss(*c)) {
        return Err("status.barcode_invalid");
    }
    let pattern = |c: char| CODABAR.iter().find(|(k, _)| *k == c).map(|(_, p)| *p);
    let mut elems = Vec::new();
    let all: Vec<char> = std::iter::once(start)
        .chain(body.iter().copied())
        .chain(std::iter::once(stop))
        .collect();
    for (i, c) in all.iter().enumerate() {
        let p = pattern(*c).ok_or("status.barcode_invalid")?;
        push_two(&mut elems, p, i + 1 < all.len());
    }
    Ok(Encoded {
        elems,
        data: all.iter().collect(),
        check: None,
        digits: String::new(),
        addon: String::new(),
    })
}

/// Code 39 characters in check-value order, with their patterns.
const CODE39: [(char, &str); 43] = [
    ('0', "000110100"),
    ('1', "100100001"),
    ('2', "001100001"),
    ('3', "101100000"),
    ('4', "000110001"),
    ('5', "100110000"),
    ('6', "001110000"),
    ('7', "000100101"),
    ('8', "100100100"),
    ('9', "001100100"),
    ('A', "100001001"),
    ('B', "001001001"),
    ('C', "101001000"),
    ('D', "000011001"),
    ('E', "100011000"),
    ('F', "001011000"),
    ('G', "000001101"),
    ('H', "100001100"),
    ('I', "001001100"),
    ('J', "000011100"),
    ('K', "100000011"),
    ('L', "001000011"),
    ('M', "101000010"),
    ('N', "000010011"),
    ('O', "100010010"),
    ('P', "001010010"),
    ('Q', "000000111"),
    ('R', "100000110"),
    ('S', "001000110"),
    ('T', "000010110"),
    ('U', "110000001"),
    ('V', "011000001"),
    ('W', "111000000"),
    ('X', "010010001"),
    ('Y', "110010000"),
    ('Z', "011010000"),
    ('-', "010000101"),
    ('.', "110000100"),
    (' ', "011000100"),
    ('$', "010101000"),
    ('/', "010100010"),
    ('+', "010001010"),
    ('%', "000101010"),
];
const CODE39_STAR: &str = "010010100";

/// The Code 39 modulo 43 check character of `text`.
pub fn code39_check(text: &str) -> Option<char> {
    let mut sum = 0usize;
    for c in text.chars() {
        sum += CODE39.iter().position(|(k, _)| *k == c)?;
    }
    Some(CODE39[sum % 43].0)
}

/// Code 39: upper-case letters, digits and - . space $ / + %, between the
/// * start and stop characters.
fn code39(text: &str, check: bool) -> Result<Encoded, &'static str> {
    let upper = text.to_ascii_uppercase();
    if upper.is_empty() || upper.len() > 43 {
        return Err("status.barcode_invalid");
    }
    let check_char = if check {
        Some(code39_check(&upper).ok_or("status.barcode_invalid")?)
    } else {
        None
    };
    let mut elems = Vec::new();
    push_two(&mut elems, CODE39_STAR, true);
    for c in upper.chars().chain(check_char) {
        let p = CODE39
            .iter()
            .find(|(k, _)| *k == c)
            .map(|(_, p)| *p)
            .ok_or("status.barcode_invalid")?;
        push_two(&mut elems, p, true);
    }
    push_two(&mut elems, CODE39_STAR, false);
    Ok(Encoded {
        elems,
        data: upper.chars().chain(check_char).collect(),
        check: check_char,
        digits: String::new(),
        addon: String::new(),
    })
}

const I25: [&str; 10] = [
    "00110", "10001", "01001", "11000", "00101", "10100", "01100", "00011", "10010", "01010",
];

/// Interleaved 2 of 5: digits in pairs (a leading zero makes the count
/// even), the first of each pair in the bars, the second in the spaces.
fn interleaved(text: &str, check: bool) -> Result<Encoded, &'static str> {
    let mut d = digits(text).ok_or("status.barcode_invalid")?;
    if d.is_empty() || d.len() > 40 {
        return Err("status.barcode_invalid");
    }
    let mut check_char = None;
    if check {
        let c = ean_check(&d);
        d.push(c);
        check_char = Some((b'0' + c) as char);
    }
    if d.len() % 2 == 1 {
        d.insert(0, 0);
    }
    let mut elems = Vec::new();
    for _ in 0..2 {
        elems.push(Elem::two(true, false));
        elems.push(Elem::two(false, false));
    }
    for pair in d.chunks(2) {
        let (a, b) = (
            I25[pair[0] as usize].as_bytes(),
            I25[pair[1] as usize].as_bytes(),
        );
        for (wa, wb) in a.iter().zip(b) {
            elems.push(Elem::two(true, *wa == b'1'));
            elems.push(Elem::two(false, *wb == b'1'));
        }
    }
    elems.push(Elem::two(true, true));
    elems.push(Elem::two(false, false));
    elems.push(Elem::two(true, false));
    let data: String = d.iter().map(|v| (b'0' + v) as char).collect();
    Ok(Encoded {
        elems,
        data,
        check: check_char,
        digits: String::new(),
        addon: String::new(),
    })
}

// ----- UPC-E and add-ons -----------------------------------------------------

/// UPC-E parity for number system 0 by check digit (1 = even, G table);
/// number system 1 takes the opposite.
const UPCE_PARITY: [&str; 10] = [
    "111000", "110100", "110010", "110001", "101100", "100110", "100011", "101010", "101001",
    "100101",
];

/// The UPC-A digits (11, without check) a UPC-E number system and six
/// digits stand for.
fn upce_expand(ns: u8, d: &[u8]) -> Vec<u8> {
    let mut a = vec![ns];
    match d[5] {
        0..=2 => a.extend([d[0], d[1], d[5], 0, 0, 0, 0, d[2], d[3], d[4]]),
        3 => a.extend([d[0], d[1], d[2], 0, 0, 0, 0, 0, d[3], d[4]]),
        4 => a.extend([d[0], d[1], d[2], d[3], 0, 0, 0, 0, 0, d[4]]),
        _ => a.extend([d[0], d[1], d[2], d[3], d[4], 0, 0, 0, 0, d[5]]),
    }
    a
}

/// UPC-E: six digits, or a number system (0 or 1) and six digits, with or
/// without the check digit. Returns the modules and the eight digits.
fn upce(text: &str) -> Result<(Vec<u8>, String), &'static str> {
    let d = digits(text).ok_or("status.barcode_invalid")?;
    let (ns, six, given) = match d.len() {
        6 => (0, &d[..], None),
        7 => (d[0], &d[1..7], None),
        8 => (d[0], &d[1..7], Some(d[7])),
        _ => return Err("status.barcode_invalid"),
    };
    if ns > 1 {
        return Err("status.barcode_invalid");
    }
    let check = ean_check(&upce_expand(ns, six));
    if given.is_some_and(|g| g != check) {
        return Err("status.barcode_invalid");
    }
    let parity = UPCE_PARITY[check as usize].as_bytes();
    let mut out = Vec::new();
    push_pattern(&mut out, "101");
    for (k, v) in six.iter().enumerate() {
        let even = (parity[k] == b'1') != (ns == 1);
        push_pattern(
            &mut out,
            if even {
                EAN_G[*v as usize]
            } else {
                EAN_L[*v as usize]
            },
        );
    }
    push_pattern(&mut out, "010101");
    let digits = std::iter::once(ns)
        .chain(six.iter().copied())
        .chain(std::iter::once(check))
        .map(|v| (b'0' + v) as char)
        .collect();
    Ok((out, digits))
}

const ADDON5_PARITY: [&str; 10] = [
    "11000", "10100", "10010", "10001", "01100", "00110", "00011", "01010", "01001", "00101",
];

/// A 2- or 5-digit add-on (1 = G table in the parity patterns).
fn ean_addon(text: &str) -> Result<Vec<u8>, &'static str> {
    let d = digits(text).ok_or("status.barcode_invalid")?;
    let parity: Vec<u8> = match d.len() {
        2 => {
            let v = (d[0] * 10 + d[1]) % 4;
            vec![v >> 1, v & 1]
        }
        5 => {
            let c = (3 * (d[0] + d[2] + d[4]) as u32 + 9 * (d[1] + d[3]) as u32) % 10;
            ADDON5_PARITY[c as usize]
                .bytes()
                .map(|b| b - b'0')
                .collect()
        }
        _ => return Err("status.barcode_invalid"),
    };
    let mut out = Vec::new();
    push_pattern(&mut out, "1011");
    for (k, v) in d.iter().enumerate() {
        if k > 0 {
            push_pattern(&mut out, "01");
        }
        let table = if parity[k] == 1 { EAN_G } else { EAN_L };
        push_pattern(&mut out, table[*v as usize]);
    }
    Ok(out)
}

// ----- Layout ----------------------------------------------------------------

/// Text alignment under (or over) the bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

/// Everything the wizard asks, in millimetres and points.
#[derive(Debug, Clone, PartialEq)]
pub struct BarcodeSpec {
    pub symbology: Symbology,
    pub data: String,
    /// The printer's resolution: the narrow module is a whole number of
    /// its dots.
    pub dpi: u32,
    /// Ink spread taken off every bar, in printer dots.
    pub reduction_px: f64,
    /// Size of the symbol against its nominal 0.33 mm narrow module.
    pub magnification: f64,
    pub bar_height_mm: f64,
    /// Wide to narrow ratio of two-width symbologies.
    pub ratio: f64,
    /// Advanced: the optional check character.
    pub check_digit: bool,
    /// Advanced: a white background reaching this many narrow modules past
    /// the bars on each side (0: none).
    pub quiet_zone: u32,
    pub font: String,
    pub size_pt: f64,
    pub bold: bool,
    pub italic: bool,
    pub alignment: TextAlignment,
    pub human_readable: bool,
    pub center_sys_check: bool,
    pub show_addon_text: bool,
    pub addon_text_bottom: bool,
    pub quiet_marks: bool,
    pub text_above: bool,
    pub fact_data: bool,
    pub asterisks: bool,
    pub start_stop: bool,
}

impl Default for BarcodeSpec {
    fn default() -> Self {
        BarcodeSpec {
            symbology: Symbology::Code128,
            data: Symbology::Code128.sample().into(),
            dpi: 300,
            reduction_px: 1.0,
            magnification: 100.0,
            bar_height_mm: 25.4,
            ratio: 2.0,
            check_digit: false,
            quiet_zone: 0,
            font: "Times New Roman".into(),
            size_pt: 12.0,
            bold: false,
            italic: false,
            alignment: TextAlignment::Center,
            human_readable: true,
            center_sys_check: true,
            show_addon_text: true,
            addon_text_bottom: false,
            quiet_marks: false,
            text_above: false,
            fact_data: false,
            asterisks: true,
            start_stop: false,
        }
    }
}

/// The nominal narrow module at 100 %.
pub const NOMINAL_MODULE_MM: f64 = 0.33;

impl BarcodeSpec {
    /// The narrow module: the nominal one magnified, rounded to whole
    /// printer dots (at least one).
    pub fn module_mm(&self) -> f64 {
        let dpi = self.dpi.max(1) as f64;
        let wanted = NOMINAL_MODULE_MM * self.magnification.max(1.0) / 100.0;
        let dots = (wanted * dpi / 25.4).round().max(1.0);
        dots * 25.4 / dpi
    }

    /// An option of the last step, when it applies to the symbology.
    pub fn option(&self, o: TextOption) -> bool {
        let on = match o {
            TextOption::CenterSysCheck => self.center_sys_check,
            TextOption::AddOnText => self.show_addon_text,
            TextOption::AddOnBottom => self.addon_text_bottom,
            TextOption::QuietMarks => self.quiet_marks,
            TextOption::TextAbove => self.text_above,
            TextOption::FactData => self.fact_data,
            TextOption::Asterisks => self.asterisks,
            TextOption::StartStop => self.start_stop,
        };
        on && self.symbology.supports(o, self.check_digit)
    }
}

/// A line of human-readable text: its characters and where its baseline
/// starts, centres or ends.
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub text: String,
    pub at: Point,
    pub alignment: TextAlignment,
}

/// The laid-out symbol, in millimetres with y up and the bars' lower left
/// corner at the origin.
#[derive(Debug, Clone, PartialEq)]
pub struct Built {
    pub bars: Vec<Rect>,
    pub labels: Vec<Label>,
    /// The white background with the quiet zone, when asked for.
    pub background: Option<Rect>,
    /// Width of the symbol, bars and add-on.
    pub width: f64,
}

/// Lay out `spec`: bars, labels and background.
pub fn build(spec: &BarcodeSpec) -> Result<(Built, Encoded), &'static str> {
    let enc = encode_symbol(
        spec.symbology,
        &spec.data,
        spec.check_digit && spec.symbology.optional_check(),
    )?;
    let m = spec.module_mm();
    let ratio = if spec.symbology.two_widths() {
        spec.ratio.clamp(2.0, 3.0)
    } else {
        1.0
    };
    let reduction = (spec.reduction_px.max(0.0) * 25.4 / spec.dpi.max(1) as f64).min(m * 0.9);
    let h = spec.bar_height_mm.max(1.0);
    let text_h = if spec.human_readable {
        spec.size_pt.max(1.0) * 25.4 / 72.0
    } else {
        0.0
    };
    let above = spec.option(TextOption::TextAbove);
    let retail = spec.symbology.retail();
    // Retail symbols with their digits below: the guards go down into the
    // text, the other bars stop above it.
    let lift = if retail && spec.human_readable && !above {
        text_h * 0.6
    } else {
        0.0
    };
    let addon_text = spec.human_readable && spec.option(TextOption::AddOnText);
    let addon_bottom = spec.option(TextOption::AddOnBottom);
    let mut bars = Vec::new();
    let mut x = 0.0;
    let mut main_end = 0.0;
    let mut addon_start = None;
    for e in &enc.elems {
        let w = e.width(ratio) * m;
        if e.part == Part::AddOn && addon_start.is_none() {
            addon_start = Some(x);
        }
        if e.part != Part::AddOn {
            main_end = x + w;
        }
        if e.bar {
            let (y0, y1) = match e.part {
                Part::Guard => (0.0, h),
                Part::Main => (lift, h),
                Part::AddOn => {
                    let top = if addon_text && !addon_bottom {
                        h - text_h * 1.1
                    } else {
                        h
                    };
                    let bottom = if addon_text && addon_bottom {
                        text_h * 1.1
                    } else {
                        lift
                    };
                    (bottom, top)
                }
            };
            bars.push(Rect::new(x, y0, x + (w - reduction).max(w * 0.1), y1));
        }
        x += w;
    }
    let width = x;
    let mut labels = Vec::new();
    if spec.human_readable {
        let base_y = if above {
            h + text_h * 0.25
        } else {
            -text_h * 0.85 + lift
        };
        let line = |text: String, at_x: f64, alignment| Label {
            text,
            at: Point::new(at_x, base_y),
            alignment,
        };
        let groups = |d: &str, from: usize, to: usize| d.get(from..to).unwrap_or("").to_string();
        let digits = enc.digits.clone();
        let centre = spec.option(TextOption::CenterSysCheck);
        match spec.symbology {
            Symbology::Ean13 if !above && centre => {
                // The first digit before the bars, two groups of six under
                // the halves.
                labels.push(line(groups(&digits, 0, 1), -m, TextAlignment::Right));
                labels.push(line(groups(&digits, 1, 7), 24.0 * m, TextAlignment::Center));
                labels.push(line(
                    groups(&digits, 7, 13),
                    71.0 * m,
                    TextAlignment::Center,
                ));
            }
            Symbology::UpcA if !above && centre => {
                labels.push(line(groups(&digits, 0, 1), -m, TextAlignment::Right));
                labels.push(line(groups(&digits, 1, 6), 26.5 * m, TextAlignment::Center));
                labels.push(line(
                    groups(&digits, 6, 11),
                    68.5 * m,
                    TextAlignment::Center,
                ));
                labels.push(line(
                    groups(&digits, 11, 12),
                    main_end + m,
                    TextAlignment::Left,
                ));
            }
            Symbology::UpcE if !above && centre => {
                labels.push(line(groups(&digits, 0, 1), -m, TextAlignment::Right));
                labels.push(line(groups(&digits, 1, 7), 24.0 * m, TextAlignment::Center));
                labels.push(line(
                    groups(&digits, 7, 8),
                    main_end + m,
                    TextAlignment::Left,
                ));
            }
            Symbology::Ean8 if !above => {
                labels.push(line(groups(&digits, 0, 4), 17.0 * m, TextAlignment::Center));
                labels.push(line(groups(&digits, 4, 8), 50.0 * m, TextAlignment::Center));
            }
            _ => {
                let mut text = if retail {
                    digits.clone()
                } else {
                    enc.data.clone()
                };
                match spec.symbology {
                    Symbology::Code39 => {
                        if !spec.option(TextOption::FactData) {
                            if let Some(c) = enc.check {
                                if text.ends_with(c) {
                                    text.pop();
                                }
                            }
                        }
                        if spec.option(TextOption::Asterisks) {
                            text = format!("*{text}*");
                        }
                    }
                    Symbology::Codabar if !spec.option(TextOption::StartStop) => {
                        let n = text.chars().count();
                        text = text.chars().skip(1).take(n.saturating_sub(2)).collect();
                    }
                    _ => {}
                }
                let at_x = match spec.alignment {
                    TextAlignment::Left => 0.0,
                    TextAlignment::Center => main_end / 2.0,
                    TextAlignment::Right => main_end,
                };
                labels.push(line(text, at_x, spec.alignment));
            }
        }
        if spec.option(TextOption::QuietMarks) && !above {
            labels.push(line(">".into(), main_end + 2.0 * m, TextAlignment::Left));
            if spec.symbology == Symbology::Ean8 {
                labels.push(line("<".into(), -2.0 * m, TextAlignment::Right));
            }
        }
        if let (Some(start), true) = (addon_start, addon_text && !enc.addon.is_empty()) {
            let y = if addon_bottom {
                -text_h * 0.85 + lift
            } else {
                h - text_h * 0.85
            };
            labels.push(Label {
                text: enc.addon.clone(),
                at: Point::new((start + 9.0 * m + width) / 2.0, y),
                alignment: TextAlignment::Center,
            });
        }
    }
    let background = (spec.quiet_zone > 0).then(|| {
        let q = spec.quiet_zone as f64 * m;
        let low = if spec.human_readable && !above {
            -text_h * 1.1
        } else {
            0.0
        };
        let high = if spec.human_readable && above {
            h + text_h * 1.3
        } else {
            h
        };
        Rect::new(-q, low - m, width + q, high + m)
    });
    Ok((
        Built {
            bars,
            labels,
            background,
            width,
        },
        enc,
    ))
}

/// Modules -> bar rectangles as (x_offset_in_modules, width_in_modules).
pub fn bars(modules: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < modules.len() {
        if modules[i] == 1 {
            let start = i;
            while i < modules.len() && modules[i] == 1 {
                i += 1;
            }
            out.push((start, i - start));
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ean13_known_vector() {
        // 590123412345 -> check digit 7.
        assert_eq!(ean_check(&[5, 9, 0, 1, 2, 3, 4, 1, 2, 3, 4, 5]), 7);
        let m = ean13("5901234123457").unwrap();
        assert_eq!(m.len(), 95);
        assert!(ean13("5901234123450").is_err());
    }

    #[test]
    fn ean8_length() {
        let m = ean8("9638507").unwrap();
        assert_eq!(m.len(), 67);
    }

    #[test]
    fn code128_checksum_and_shape() {
        // "ABC" in subset B: start 104, 33, 34, 35; check = (104 + 33 + 68 + 105) % 103 = 4.
        let m = code128("ABC").unwrap();
        // 4 data/start symbols + check + stop = 6 * 11 + 2 = 68 modules.
        assert_eq!(m.len(), 68);
        assert_eq!(m[0], 1);
        assert_eq!(*m.last().unwrap(), 1);
        // Digits use subset C and come out shorter.
        let d = code128("12345678").unwrap();
        assert!(d.len() < code128("1234567A").unwrap().len());
        assert!(code128("").is_err());
    }

    fn widths(e: &Encoded, ratio: f64) -> f64 {
        e.elems.iter().map(|x| x.width(ratio)).sum()
    }

    #[test]
    fn code39_has_three_wide_of_nine_and_its_check() {
        assert_eq!(code39_check("CODE39"), Some('W'));
        let e = encode_symbol(Symbology::Code39, "code39", true).unwrap();
        assert_eq!(e.data, "CODE39W");
        // Start, 7 characters, stop: 9 elements each and 8 gaps.
        assert_eq!(e.elems.len(), 9 * 9 + 8);
        for c in e.elems.chunks(10) {
            assert_eq!(c[..9].iter().filter(|x| x.wide).count(), 3);
        }
        assert!(encode_symbol(Symbology::Code39, "a~b", false).is_err());
        // The ratio widens the symbol.
        assert!(widths(&e, 3.0) > widths(&e, 2.0));
    }

    #[test]
    fn codabar_wraps_the_data_in_start_and_stop() {
        let e = encode_symbol(Symbology::Codabar, "40156", false).unwrap();
        assert_eq!(e.data, "A40156B");
        assert_eq!(e.elems.len(), 7 * 7 + 6);
        let e = encode_symbol(Symbology::Codabar, "c12-3d", false).unwrap();
        assert_eq!(e.data, "C12-3D");
        assert!(encode_symbol(Symbology::Codabar, "12X", false).is_err());
        assert!(encode_symbol(Symbology::Codabar, &"1".repeat(31), false).is_err());
    }

    #[test]
    fn interleaved_pairs_digits_and_pads_odd_counts() {
        let e = encode_symbol(Symbology::Interleaved2of5, "123", false).unwrap();
        assert_eq!(e.data, "0123");
        // Start 4, two pairs of 10, stop 3.
        assert_eq!(e.elems.len(), 4 + 20 + 3);
        let c = encode_symbol(Symbology::Interleaved2of5, "1234567", true).unwrap();
        assert_eq!(c.check, Some('0'));
        assert_eq!(c.data, "12345670");
    }

    #[test]
    fn upc_symbols_and_add_ons() {
        let a = encode_symbol(Symbology::UpcA, "03600029145", false).unwrap();
        assert_eq!(a.digits, "036000291452");
        let total: usize = a.elems.iter().map(|x| x.units as usize).sum();
        assert_eq!(total, 95);
        let e = encode_symbol(Symbology::UpcE, "0425261", false).unwrap();
        assert_eq!(e.digits.len(), 8);
        let total: usize = e.elems.iter().map(|x| x.units as usize).sum();
        assert_eq!(total, 51);
        assert!(encode_symbol(Symbology::UpcE, "2425261", false).is_err());
        // A 5-digit add-on: 9 modules apart, 47 modules long.
        let w = encode_symbol(Symbology::Ean13, "590123412345 52495", false).unwrap();
        assert_eq!(w.addon, "52495");
        let addon: usize = w
            .elems
            .iter()
            .filter(|x| x.part == Part::AddOn)
            .map(|x| x.units as usize)
            .sum();
        assert_eq!(addon, 9 + 47);
        let two = encode_symbol(Symbology::Ean13, "590123412345 12", false).unwrap();
        let addon: usize = two
            .elems
            .iter()
            .filter(|x| x.part == Part::AddOn)
            .map(|x| x.units as usize)
            .sum();
        assert_eq!(addon, 9 + 20);
        assert!(encode_symbol(Symbology::Ean13, "590123412345 123", false).is_err());
    }

    fn spec(sym: Symbology, data: &str) -> BarcodeSpec {
        BarcodeSpec {
            symbology: sym,
            data: data.into(),
            ..Default::default()
        }
    }

    #[test]
    fn printer_settings_size_the_bars() {
        let s = spec(Symbology::Code128, "ABC");
        // 0.33 mm at 300 dpi is 4 dots: 0.3387 mm.
        assert!((s.module_mm() - 4.0 * 25.4 / 300.0).abs() < 1e-9);
        let (b, _) = build(&s).unwrap();
        let (big, _) = build(&BarcodeSpec {
            magnification: 200.0,
            ..s.clone()
        })
        .unwrap();
        assert!(big.width > b.width * 1.9);
        // Bar width reduction takes dots off every bar.
        let (thin, _) = build(&BarcodeSpec {
            reduction_px: 2.0,
            ..s.clone()
        })
        .unwrap();
        assert!(thin.bars[0].width() < b.bars[0].width());
        assert_eq!(thin.width, b.width);
        let (tall, _) = build(&BarcodeSpec {
            bar_height_mm: 40.0,
            ..s.clone()
        })
        .unwrap();
        assert!((tall.bars[0].height() - 40.0).abs() < 1e-9);
        // The ratio only matters to two-width symbologies.
        let c = spec(Symbology::Code39, "AB");
        let (r2, _) = build(&c).unwrap();
        let (r3, _) = build(&BarcodeSpec { ratio: 3.0, ..c }).unwrap();
        assert!(r3.width > r2.width);
        let (same, _) = build(&BarcodeSpec { ratio: 3.0, ..s }).unwrap();
        assert_eq!(same.width, b.width);
    }

    #[test]
    fn text_options_change_the_labels() {
        let c = spec(Symbology::Code39, "AB");
        let (b, _) = build(&c).unwrap();
        assert_eq!(b.labels[0].text, "*AB*");
        let (b, _) = build(&BarcodeSpec {
            asterisks: false,
            ..c.clone()
        })
        .unwrap();
        assert_eq!(b.labels[0].text, "AB");
        // FACT data shows the check character, only with one.
        let checked = BarcodeSpec {
            check_digit: true,
            asterisks: false,
            ..c.clone()
        };
        assert_eq!(build(&checked).unwrap().0.labels[0].text, "AB");
        let fact = BarcodeSpec {
            fact_data: true,
            ..checked
        };
        assert_eq!(
            build(&fact).unwrap().0.labels[0].text,
            "AB".to_string() + &code39_check("AB").unwrap().to_string()
        );
        // Above or below the bars.
        let (below, _) = build(&c).unwrap();
        let (above, _) = build(&BarcodeSpec {
            text_above: true,
            ..c.clone()
        })
        .unwrap();
        assert!(below.labels[0].at.y < 0.0 && above.labels[0].at.y > c.bar_height_mm);
        // Not human readable: no labels.
        let (none, _) = build(&BarcodeSpec {
            human_readable: false,
            ..c.clone()
        })
        .unwrap();
        assert!(none.labels.is_empty());
        // Alignment.
        let (left, _) = build(&BarcodeSpec {
            alignment: TextAlignment::Left,
            ..c.clone()
        })
        .unwrap();
        assert_eq!(left.labels[0].at.x, 0.0);
        // Codabar start and stop characters.
        let cb = spec(Symbology::Codabar, "123");
        assert_eq!(build(&cb).unwrap().0.labels[0].text, "123");
        let ss = BarcodeSpec {
            start_stop: true,
            ..cb
        };
        assert_eq!(build(&ss).unwrap().0.labels[0].text, "A123B");
    }

    #[test]
    fn retail_symbols_group_their_digits() {
        let e = spec(Symbology::Ean13, "590123412345");
        let (b, _) = build(&e).unwrap();
        let texts: Vec<&str> = b.labels.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, ["5", "901234", "123457"]);
        // Guards go lower than the other bars.
        let low = b.bars.iter().map(|r| r.y0).fold(f64::MAX, f64::min);
        let high = b.bars.iter().map(|r| r.y0).fold(f64::MIN, f64::max);
        assert!(low < high);
        // One line when not centred.
        let (one, _) = build(&BarcodeSpec {
            center_sys_check: false,
            ..e.clone()
        })
        .unwrap();
        assert_eq!(one.labels.len(), 1);
        assert_eq!(one.labels[0].text, "5901234123457");
        // Quiet zone marks and the add-on's text.
        let (q, _) = build(&BarcodeSpec {
            quiet_marks: true,
            ..e.clone()
        })
        .unwrap();
        assert!(q.labels.iter().any(|l| l.text == ">"));
        let with = BarcodeSpec {
            data: "590123412345 12".into(),
            ..e.clone()
        };
        let (a, _) = build(&with).unwrap();
        assert!(a.labels.iter().any(|l| l.text == "12"));
        let (hidden, _) = build(&BarcodeSpec {
            show_addon_text: false,
            ..with.clone()
        })
        .unwrap();
        assert!(!hidden.labels.iter().any(|l| l.text == "12"));
        let (bottom, _) = build(&BarcodeSpec {
            addon_text_bottom: true,
            ..with
        })
        .unwrap();
        let y = |b: &Built| b.labels.iter().find(|l| l.text == "12").unwrap().at.y;
        assert!(y(&bottom) < y(&a));
        let upc = spec(Symbology::UpcA, "03600029145");
        let texts: Vec<String> = build(&upc)
            .unwrap()
            .0
            .labels
            .into_iter()
            .map(|l| l.text)
            .collect();
        assert_eq!(texts, ["0", "36000", "29145", "2"]);
        // A quiet zone puts a white background behind.
        let (bg, _) = build(&BarcodeSpec {
            quiet_zone: 10,
            ..upc
        })
        .unwrap();
        let r = bg.background.unwrap();
        assert!(r.x0 < 0.0 && r.x1 > bg.width);
    }
}
