//! Autocorrect (Tools > Options > Text > Autocorrect, also Text > Writing
//! Tools > Autocorrect): corrections made while typing, once the word is
//! finished with a space or punctuation. Rules: capitalise the first
//! letter of a sentence, fix two initial capitals ("THis"), capitalise
//! the names of days, typographic quotes by language or as chosen
//! (straight after numbers if asked), the user's replacement table
//! ("(c)" to "©"), and a hyperlink for a typed web address. Options
//! persist in settings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutocorrectPrefs {
    /// Apply the rules while typing (an internal switch; the options
    /// page turns each rule on and off).
    pub enabled: bool,
    pub capitalize_sentences: bool,
    pub fix_two_initial_capitals: bool,
    /// "monday" to "Monday".
    pub capitalize_days: bool,
    /// A typed web address gives the text object that hyperlink.
    pub auto_hyperlinks: bool,
    /// The language whose quotation marks are used (`""`: the
    /// interface language).
    pub quote_language: String,
    pub typographic_quotes: bool,
    /// A quote right after a digit stays straight (6' 2").
    pub straight_after_numbers: bool,
    /// Opening and closing quotes chosen by hand (`None`: the language's).
    pub single_quotes: Option<[char; 2]>,
    pub double_quotes: Option<[char; 2]>,
    /// Apply the replacement table while typing.
    pub replace_while_typing: bool,
    /// (abbreviation, replacement) pairs, matched as whole words.
    pub replacements: Vec<(String, String)>,
}

impl Default for AutocorrectPrefs {
    fn default() -> Self {
        AutocorrectPrefs {
            enabled: true,
            capitalize_sentences: true,
            fix_two_initial_capitals: true,
            capitalize_days: false,
            auto_hyperlinks: false,
            quote_language: String::new(),
            typographic_quotes: true,
            straight_after_numbers: false,
            single_quotes: None,
            double_quotes: None,
            replace_while_typing: true,
            replacements: vec![
                ("(c)".into(), "©".into()),
                ("(r)".into(), "®".into()),
                ("(tm)".into(), "™".into()),
                ("1/2".into(), "½".into()),
                ("1/4".into(), "¼".into()),
                ("3/4".into(), "¾".into()),
            ],
        }
    }
}

impl AutocorrectPrefs {
    /// The quotation style of the chosen language (else the interface's).
    pub fn style(&self) -> QuoteStyle {
        if self.quote_language.is_empty() {
            QuoteStyle::for_language(&crate::i18n::language())
        } else {
            QuoteStyle::for_language(&self.quote_language)
        }
    }

    /// (open, close) double quotes: chosen by hand, else `style`'s.
    pub fn double(&self, style: QuoteStyle) -> (char, char) {
        self.double_quotes
            .map(|[a, b]| (a, b))
            .unwrap_or(style.double())
    }

    /// (open, close) single quotes: chosen by hand, else `style`'s.
    pub fn single(&self, style: QuoteStyle) -> (char, char) {
        self.single_quotes
            .map(|[a, b]| (a, b))
            .unwrap_or(style.single())
    }

    /// Older settings could switch the whole of autocorrect off; now each
    /// rule has its own switch, so that becomes every rule off.
    pub fn upgrade(&mut self) {
        if !self.enabled {
            self.enabled = true;
            self.capitalize_sentences = false;
            self.fix_two_initial_capitals = false;
            self.capitalize_days = false;
            self.auto_hyperlinks = false;
            self.typographic_quotes = false;
            self.replace_while_typing = false;
        }
    }
}

/// The quotation marks the options offer for opening and closing.
pub const QUOTE_CHOICES: [char; 16] = [
    '\'', '‘', '’', '‚', '‹', '›', '"', '“', '”', '„', '«', '»', '「', '」', '『', '』',
];

/// The English names of the days, capitalised by [`capitalize_day`].
const DAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

/// "monday" (and "monday's", "monday,") to "Monday".
pub fn capitalize_day(word: &str) -> String {
    let letters: String = word.chars().take_while(|c| c.is_alphabetic()).collect();
    let rest = &word[letters.len()..];
    let ends_word = rest.is_empty() || rest.starts_with(['\'', '’', '.', ',', ';', ':', '!', '?']);
    if ends_word && DAYS.contains(&letters.to_lowercase().as_str()) {
        let mut cs = letters.chars();
        if let Some(f) = cs.next() {
            if f.is_lowercase() {
                return f.to_uppercase().chain(cs).collect::<String>() + rest;
            }
        }
    }
    word.to_string()
}

/// The web address a typed word is (`http://`, `https://` or `www.`
/// with a dot after it), trailing punctuation and brackets left out;
/// `www.` addresses get `http://`.
pub fn web_address(word: &str) -> Option<String> {
    let w = word
        .trim_start_matches(['(', '[', '<', '"', '\'', '“', '‘'])
        .trim_end_matches([
            '.', ',', ';', ':', '!', '?', ')', ']', '>', '"', '\'', '”', '’',
        ]);
    let lower = w.to_ascii_lowercase();
    let host = lower
        .strip_prefix("http://")
        .or_else(|| lower.strip_prefix("https://"))
        .or_else(|| lower.strip_prefix("www."))?;
    let name = host.split(['/', '?', '#']).next().unwrap_or("");
    let parts: Vec<&str> = name.split('.').collect();
    let named = name.contains('.') || lower.starts_with("www.");
    if !named || parts.iter().any(|p| p.is_empty()) || w.contains(char::is_whitespace) {
        return None;
    }
    Some(if lower.starts_with("www.") {
        format!("http://{w}")
    } else {
        w.to_string()
    })
}

/// The web address of the word just finished by the last typed
/// character, if any.
pub fn typed_web_address(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let last = *chars.last()?;
    if !last.is_whitespace() {
        return None;
    }
    let end = chars.len() - 1;
    let mut start = end;
    while start > 0 && !chars[start - 1].is_whitespace() {
        start -= 1;
    }
    web_address(&chars[start..end].iter().collect::<String>())
}

/// Quotation mark style; chosen from the UI language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteStyle {
    /// “ ” and ‘ ’ (English, Portuguese, Spanish and most others).
    Curly,
    /// „ “ and ‚ ‘ (German).
    German,
    /// « » and ‹ › (French, Russian).
    Guillemets,
}

impl QuoteStyle {
    pub fn for_language(lang: &str) -> Self {
        let base = lang.split('-').next().unwrap_or(lang).to_lowercase();
        match base.as_str() {
            "de" | "cs" => QuoteStyle::German,
            "fr" | "ru" => QuoteStyle::Guillemets,
            _ => QuoteStyle::Curly,
        }
    }

    /// (open, close) for double quotes.
    pub fn double(self) -> (char, char) {
        match self {
            QuoteStyle::Curly => ('“', '”'),
            QuoteStyle::German => ('„', '“'),
            QuoteStyle::Guillemets => ('«', '»'),
        }
    }

    /// (open, close) for single quotes; the closing one doubles as the apostrophe.
    pub fn single(self) -> (char, char) {
        match self {
            QuoteStyle::Curly => ('‘', '’'),
            QuoteStyle::German => ('‚', '‘'),
            QuoteStyle::Guillemets => ('‹', '›'),
        }
    }
}

/// Characters that end a word and trigger the corrections while typing.
fn is_trigger(c: char) -> bool {
    c.is_whitespace() || matches!(c, '.' | ',' | ';' | ':' | '!' | '?')
}

/// A quote at `pos` opens when nothing, whitespace or an opening bracket
/// or dash precedes it.
fn opens_quote(chars: &[char], pos: usize) -> bool {
    match pos.checked_sub(1).and_then(|i| chars.get(i)) {
        None => true,
        Some(p) => {
            p.is_whitespace() || matches!(p, '(' | '[' | '{' | '-' | '\u{2013}' | '\u{2014}' | '/')
        }
    }
}

/// Straight quotes to typographic ones over the whole text.
#[cfg(test)]
pub fn typographic_quotes(text: &str, style: QuoteStyle) -> String {
    quotes_with(text, style.double(), style.single(), false)
}

/// Straight quotes to `double` and `single` (open, close) over the whole
/// text; with `after_numbers` a quote right after a digit stays.
fn quotes_with(
    text: &str,
    double: (char, char),
    single: (char, char),
    after_numbers: bool,
) -> String {
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let (open, close) = match c {
                '"' => double,
                '\'' => single,
                o => return *o,
            };
            if after_numbers && i > 0 && chars[i - 1].is_ascii_digit() {
                *c
            } else if opens_quote(&chars, i) {
                open
            } else {
                close
            }
        })
        .collect()
}

/// "THis" to "This": a word of three or more letters whose first two are
/// capitals and the rest lowercase.
pub fn fix_two_initial_capitals_word(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let letters: Vec<char> = chars
        .iter()
        .copied()
        .filter(|c| c.is_alphabetic())
        .collect();
    if letters.len() < 3
        || !letters[0].is_uppercase()
        || !letters[1].is_uppercase()
        || !letters[2..].iter().all(|c| c.is_lowercase())
    {
        return word.to_string();
    }
    let mut seen = 0;
    chars
        .iter()
        .map(|c| {
            if c.is_alphabetic() {
                seen += 1;
                if seen == 2 {
                    return c.to_lowercase().collect::<String>();
                }
            }
            c.to_string()
        })
        .collect()
}

fn fix_two_initial_capitals(text: &str) -> String {
    map_words(text, fix_two_initial_capitals_word)
}

/// Apply `f` to every run of non-whitespace characters.
fn map_words(text: &str, mut f: impl FnMut(&str) -> String) -> String {
    let mut out = String::new();
    let mut word = String::new();
    for c in text.chars() {
        if c.is_whitespace() {
            if !word.is_empty() {
                out.push_str(&f(&word));
                word.clear();
            }
            out.push(c);
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        out.push_str(&f(&word));
    }
    out
}

/// Capitalise the first letter of every sentence.
pub fn capitalize_sentences(text: &str) -> String {
    let mut out = String::new();
    let mut at_start = true;
    let mut after_terminator = false;
    for c in text.chars() {
        if at_start && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            at_start = false;
            after_terminator = false;
            continue;
        }
        if c == '\n' {
            at_start = true;
            after_terminator = false;
        } else if matches!(c, '.' | '!' | '?') {
            after_terminator = true;
        } else if c.is_whitespace() {
            if after_terminator {
                at_start = true;
            }
        } else if c.is_alphanumeric() {
            at_start = false;
            after_terminator = false;
        } else if !matches!(
            c,
            '"' | '\'' | '“' | '„' | '«' | '‘' | '‚' | '‹' | '(' | '['
        ) {
            // Closing quotes and other punctuation keep the pending state;
            // opening quotes and brackets may precede the first letter.
            after_terminator = after_terminator && matches!(c, ')' | ']' | '”' | '’' | '»' | '“');
        }
        out.push(c);
    }
    out
}

/// Replace whole words found in the table.
pub fn apply_replacements(text: &str, table: &[(String, String)]) -> String {
    map_words(text, |w| replace_word(w, table))
}

fn replace_word(word: &str, table: &[(String, String)]) -> String {
    // Exact match first, then a match that leaves trailing punctuation alone.
    if let Some((_, to)) = table.iter().find(|(from, _)| from == word) {
        return to.clone();
    }
    let trimmed = word.trim_end_matches(['.', ',', ';', ':', '!', '?']);
    if trimmed.len() != word.len() {
        if let Some((_, to)) = table.iter().find(|(from, _)| from == trimmed) {
            return format!("{to}{}", &word[trimmed.len()..]);
        }
    }
    word.to_string()
}

/// All enabled rules over a whole text (Apply to selection).
pub fn apply_all(text: &str, prefs: &AutocorrectPrefs, style: QuoteStyle) -> String {
    let mut t = text.to_string();
    if prefs.replace_while_typing && !prefs.replacements.is_empty() {
        t = apply_replacements(&t, &prefs.replacements);
    }
    if prefs.fix_two_initial_capitals {
        t = fix_two_initial_capitals(&t);
    }
    if prefs.capitalize_days {
        t = map_words(&t, capitalize_day);
    }
    if prefs.capitalize_sentences {
        t = capitalize_sentences(&t);
    }
    if prefs.typographic_quotes {
        t = quotes_with(
            &t,
            prefs.double(style),
            prefs.single(style),
            prefs.straight_after_numbers,
        );
    }
    t
}

/// Correction after the last typed character. Only the word just finished
/// (or the quote just typed) is touched, so earlier text the user edited
/// on purpose stays as it is. Returns the new text when anything changed.
pub fn on_typed(text: &str, prefs: &AutocorrectPrefs, style: QuoteStyle) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let last = *chars.last()?;
    let n = chars.len();
    if last == '"' || last == '\'' {
        if !prefs.typographic_quotes {
            return None;
        }
        if prefs.straight_after_numbers && n > 1 && chars[n - 2].is_ascii_digit() {
            return None;
        }
        let opening = opens_quote(&chars, n - 1);
        let (o, c) = if last == '"' {
            prefs.double(style)
        } else {
            prefs.single(style)
        };
        let mut out: String = chars[..n - 1].iter().collect();
        out.push(if opening { o } else { c });
        return Some(out);
    }
    if !is_trigger(last) {
        return None;
    }
    // The word before the trigger.
    let mut end = n - 1;
    while end > 0 && is_trigger(chars[end - 1]) {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && !chars[start - 1].is_whitespace() {
        start -= 1;
    }
    if start == end {
        return None;
    }
    let word: String = chars[start..end].iter().collect();
    let mut fixed = word.clone();
    if prefs.replace_while_typing && !prefs.replacements.is_empty() {
        fixed = replace_word(&fixed, &prefs.replacements);
    }
    if prefs.fix_two_initial_capitals {
        fixed = fix_two_initial_capitals_word(&fixed);
    }
    if prefs.capitalize_days {
        fixed = capitalize_day(&fixed);
    }
    if prefs.capitalize_sentences {
        let before: String = chars[..start].iter().collect();
        let trailing_newline = chars[..start]
            .iter()
            .rev()
            .take_while(|c| c.is_whitespace())
            .any(|c| *c == '\n');
        let trimmed = before.trim_end();
        let at_sentence_start = trailing_newline
            || match trimmed.chars().last() {
                None => true,
                Some(p) => {
                    matches!(p, '.' | '!' | '?') && !(p == '.' && is_abbreviation_before(trimmed))
                }
            };
        if at_sentence_start {
            let mut cs = fixed.chars();
            if let Some(f) = cs.next() {
                if f.is_lowercase() {
                    let rest: String = cs.collect();
                    fixed = f.to_uppercase().chain(rest.chars()).collect();
                }
            }
        }
    }
    if fixed == word {
        return None;
    }
    let mut out: String = chars[..start].iter().collect();
    out.push_str(&fixed);
    out.extend(chars[end..].iter());
    Some(out)
}

/// Apply every enabled rule to the selected text objects (span by span,
/// so styling is kept). Returns the number of objects changed.
pub fn apply_to_selection(app: &mut crate::app::App) -> usize {
    use tracedraw_core::{document::ShapeKind, Command, TextSpan};
    let prefs = app.settings.autocorrect.clone();
    let style = prefs.style();
    let mut changed = 0;
    for s in app.text_shapes() {
        let ShapeKind::Text {
            spans,
            origin,
            frame,
            align,
            para,
            on_path,
        } = s.kind
        else {
            continue;
        };
        let new_spans: Vec<TextSpan> = spans
            .iter()
            .map(|sp| TextSpan {
                text: apply_all(&sp.text, &prefs, style),
                ..sp.clone()
            })
            .collect();
        if new_spans == spans {
            continue;
        }
        app.run(Command::SetShapeKind {
            shape: s.id,
            kind: ShapeKind::Text {
                spans: new_spans,
                origin,
                frame,
                align,
                para,
                on_path,
            },
        });
        changed += 1;
    }
    changed
}

/// The text (already trimmed) ends with an abbreviation point such as
/// "e.g." or "U.S.", a list number "3." or an ellipsis, none of which
/// starts a new sentence.
fn is_abbreviation_before(trimmed: &str) -> bool {
    let Some(stripped) = trimmed.strip_suffix('.') else {
        return false;
    };
    let run = stripped
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric())
        .count();
    run == 1 || stripped.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalises_sentences() {
        assert_eq!(
            capitalize_sentences("hello. how are you? fine!"),
            "Hello. How are you? Fine!"
        );
        assert_eq!(capitalize_sentences("one\ntwo"), "One\nTwo");
        assert_eq!(
            capitalize_sentences("he said. \"yes.\""),
            "He said. \"Yes.\""
        );
        assert_eq!(capitalize_sentences("3.14 is pi"), "3.14 is pi");
    }

    #[test]
    fn two_initial_capitals() {
        assert_eq!(fix_two_initial_capitals_word("THis"), "This");
        assert_eq!(fix_two_initial_capitals_word("THIS"), "THIS");
        assert_eq!(fix_two_initial_capitals_word("McDonald"), "McDonald");
        assert_eq!(fix_two_initial_capitals_word("AB"), "AB");
        assert_eq!(fix_two_initial_capitals_word("THis,"), "This,");
        assert_eq!(fix_two_initial_capitals("THe BIg dog"), "The Big dog");
    }

    #[test]
    fn quotes_follow_the_language() {
        assert_eq!(
            typographic_quotes("\"hi\" it's", QuoteStyle::Curly),
            "“hi” it’s"
        );
        assert_eq!(typographic_quotes("\"hi\"", QuoteStyle::German), "„hi“");
        assert_eq!(typographic_quotes("\"hi\"", QuoteStyle::Guillemets), "«hi»");
        assert_eq!(typographic_quotes("('a')", QuoteStyle::Curly), "(‘a’)");
        assert_eq!(QuoteStyle::for_language("pt-BR"), QuoteStyle::Curly);
        assert_eq!(QuoteStyle::for_language("de"), QuoteStyle::German);
        assert_eq!(QuoteStyle::for_language("fr"), QuoteStyle::Guillemets);
        assert_eq!(QuoteStyle::for_language("es"), QuoteStyle::Curly);
    }

    #[test]
    fn replacement_table() {
        let p = AutocorrectPrefs::default();
        assert_eq!(
            apply_replacements("Acme (c) 2026, (tm).", &p.replacements),
            "Acme © 2026, ™."
        );
        assert_eq!(apply_replacements("copy(c)", &p.replacements), "copy(c)");
    }

    #[test]
    fn apply_all_runs_every_rule() {
        let p = AutocorrectPrefs::default();
        assert_eq!(
            apply_all("THe \"big\" dog. it ran (c)", &p, QuoteStyle::Curly),
            "The “big” dog. It ran ©"
        );
        let off = AutocorrectPrefs {
            capitalize_sentences: false,
            fix_two_initial_capitals: false,
            typographic_quotes: false,
            replacements: Vec::new(),
            ..p
        };
        assert_eq!(
            apply_all("THe \"big\" dog. it", &off, QuoteStyle::Curly),
            "THe \"big\" dog. it"
        );
    }

    #[test]
    fn typing_corrects_the_finished_word_only() {
        let p = AutocorrectPrefs::default();
        let s = QuoteStyle::Curly;
        // Nothing until the word is finished.
        assert_eq!(on_typed("hello", &p, s), None);
        assert_eq!(on_typed("hello ", &p, s).as_deref(), Some("Hello "));
        assert_eq!(
            on_typed("Hello. world.", &p, s).as_deref(),
            Some("Hello. World.")
        );
        assert_eq!(on_typed("Hello world.", &p, s), None);
        assert_eq!(on_typed("Say THis ", &p, s).as_deref(), Some("Say This "));
        assert_eq!(on_typed("Acme (c) ", &p, s).as_deref(), Some("Acme © "));
        assert_eq!(on_typed("Acme (c), ", &p, s).as_deref(), Some("Acme ©, "));
        assert_eq!(on_typed("e.g. this ", &p, s), None);
        assert_eq!(
            on_typed("Line\nnext ", &p, s).as_deref(),
            Some("Line\nNext ")
        );
        // Quotes are converted as they are typed.
        assert_eq!(on_typed("He said \"", &p, s).as_deref(), Some("He said “"));
        assert_eq!(
            on_typed("He said “hi\"", &p, s).as_deref(),
            Some("He said “hi”")
        );
        assert_eq!(on_typed("it'", &p, s).as_deref(), Some("it’"));
        assert_eq!(on_typed("\"", &p, QuoteStyle::German).as_deref(), Some("„"));
        // Earlier text the user lower-cased on purpose is left alone.
        assert_eq!(on_typed("hello world. this is ", &p, s), None);
        let off = AutocorrectPrefs {
            typographic_quotes: false,
            ..AutocorrectPrefs::default()
        };
        assert_eq!(on_typed("x \"", &off, s), None);
    }

    #[test]
    fn prefs_round_trip_and_defaults() {
        let p = AutocorrectPrefs::default();
        let j = serde_json::to_string(&p).unwrap();
        let back: AutocorrectPrefs = serde_json::from_str(&j).unwrap();
        assert_eq!(back, p);
        let partial: AutocorrectPrefs = serde_json::from_str("{\"enabled\":false}").unwrap();
        assert!(!partial.enabled && partial.capitalize_sentences);
    }

    #[test]
    fn days_quotes_and_links_follow_the_options() {
        let mut p = AutocorrectPrefs::default();
        let st = QuoteStyle::Curly;
        // Days: off by default, then capitalised, possessives too.
        assert_eq!(on_typed("see you monday ", &p, st), None);
        p.capitalize_days = true;
        assert_eq!(
            on_typed("see you monday ", &p, st).as_deref(),
            Some("see you Monday ")
        );
        assert_eq!(capitalize_day("friday's"), "Friday's");
        assert_eq!(capitalize_day("mondays"), "mondays");
        assert_eq!(capitalize_day("sundae"), "sundae");
        // Straight quotes after numbers.
        assert_eq!(on_typed("6'", &p, st).as_deref(), Some("6’"));
        p.straight_after_numbers = true;
        assert_eq!(on_typed("6'", &p, st), None);
        assert_eq!(on_typed("a '", &p, st).as_deref(), Some("a ‘"));
        assert_eq!(apply_all("he is 6' 2\" tall", &p, st), "He is 6' 2\" tall");
        // Quotes chosen by hand win over the language's.
        p.double_quotes = Some(['»', '«']);
        assert_eq!(on_typed("x \"", &p, st).as_deref(), Some("x »"));
        // Another language's marks.
        p.double_quotes = None;
        p.quote_language = "de".into();
        assert_eq!(p.double(p.style()), ('„', '“'));
        // The replacement table only while it is on.
        p.replace_while_typing = false;
        assert_eq!(on_typed("(c) ", &p, st), None);
        p.replace_while_typing = true;
        assert_eq!(on_typed("(c) ", &p, st).as_deref(), Some("© "));
        // Web addresses.
        assert_eq!(
            web_address("www.example.com,").as_deref(),
            Some("http://www.example.com")
        );
        assert_eq!(
            web_address("(https://tracedraw.org/help)").as_deref(),
            Some("https://tracedraw.org/help")
        );
        assert_eq!(web_address("http://localhost"), None);
        assert_eq!(web_address("example.com"), None);
        assert_eq!(web_address("www."), None);
        assert_eq!(
            typed_web_address("go to www.a.org ").as_deref(),
            Some("http://www.a.org")
        );
        assert_eq!(typed_web_address("go to www.a.org"), None);
    }

    #[test]
    fn older_settings_with_autocorrect_off_turn_every_rule_off() {
        let mut p = AutocorrectPrefs {
            enabled: false,
            ..Default::default()
        };
        p.upgrade();
        assert!(p.enabled);
        assert!(!p.capitalize_sentences && !p.fix_two_initial_capitals);
        assert!(!p.typographic_quotes && !p.replace_while_typing);
        assert_eq!(on_typed("hello. (c) ", &p, QuoteStyle::Curly), None);
        let mut on = AutocorrectPrefs::default();
        on.upgrade();
        assert_eq!(on, AutocorrectPrefs::default());
    }

    #[test]
    fn malformed_input_never_panics() {
        let p = AutocorrectPrefs::default();
        for s in ["", " ", "\"", "'", ".", "\n", "  . ", "(c)"] {
            let _ = on_typed(s, &p, QuoteStyle::Curly);
            let _ = apply_all(s, &p, QuoteStyle::German);
        }
    }
}
