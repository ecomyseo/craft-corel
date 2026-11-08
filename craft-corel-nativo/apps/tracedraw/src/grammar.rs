//! Grammar check (Text > Proofing > Grammar): rule-based checks over
//! the text of the selected objects, or every text object on the page.
//! Each finding names the rule and the sentence, and carries a mechanical
//! fix when one exists (as a replacement of a character range), which the
//! UI applies through the normal command path.

use crate::app::App;
use tracedraw_core::{document::ShapeKind, Command, ShapeId, TextSpan};

/// The maximum sentence length before `Rule::LongSentence` fires.
pub const LONG_SENTENCE_WORDS: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    DoubledWord,
    DoubleSpace,
    SpaceBeforePunctuation,
    SentenceCapital,
    MissingSpaceAfterPunctuation,
    UnbalancedBrackets,
    UnbalancedQuotes,
    LongSentence,
    RepeatedPunctuation,
    /// "a" before a vowel sound or "an" before a consonant (English).
    ArticleAAn,
    /// A sentence of several words that ends without a full stop, a
    /// question or an exclamation mark.
    MissingSentenceEnd,
}

impl Rule {
    /// i18n key of the rule's description.
    pub fn key(self) -> &'static str {
        match self {
            Rule::DoubledWord => "grammar.rule.doubled_word",
            Rule::DoubleSpace => "grammar.rule.double_space",
            Rule::SpaceBeforePunctuation => "grammar.rule.space_before_punctuation",
            Rule::SentenceCapital => "grammar.rule.sentence_capital",
            Rule::MissingSpaceAfterPunctuation => "grammar.rule.missing_space",
            Rule::UnbalancedBrackets => "grammar.rule.unbalanced_brackets",
            Rule::UnbalancedQuotes => "grammar.rule.unbalanced_quotes",
            Rule::LongSentence => "grammar.rule.long_sentence",
            Rule::RepeatedPunctuation => "grammar.rule.repeated_punctuation",
            Rule::ArticleAAn => "grammar.rule.article_a_an",
            Rule::MissingSentenceEnd => "grammar.rule.missing_sentence_end",
        }
    }
}

/// Replace the characters `start..end` (char offsets) with `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: Rule,
    /// The sentence (trimmed) the issue was found in.
    pub sentence: String,
    pub fix: Option<Fix>,
}

/// Options that depend on the language of the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Options {
    /// French typography puts a space before `; : ! ?`.
    pub space_before_high_punctuation: bool,
    /// English rules (a and an).
    pub english: bool,
}

impl Options {
    pub fn for_language(lang: &str) -> Self {
        Options {
            space_before_high_punctuation: lang.starts_with("fr"),
            english: lang.is_empty() || lang.starts_with("en"),
        }
    }
}

/// Words starting with a vowel letter but a consonant sound, and the
/// other way round, for the a/an rule.
const VOWEL_LETTER_CONSONANT_SOUND: [&str; 8] =
    ["one", "once", "uni", "use", "usu", "euro", "ewe", "ufo"];
const CONSONANT_LETTER_VOWEL_SOUND: [&str; 5] = ["hour", "honest", "honor", "honour", "heir"];

/// The article a word takes: "an" before a vowel sound.
fn wants_an(word: &str) -> bool {
    let w = word.to_lowercase();
    if CONSONANT_LETTER_VOWEL_SOUND
        .iter()
        .any(|p| w.starts_with(p))
    {
        return true;
    }
    if VOWEL_LETTER_CONSONANT_SOUND
        .iter()
        .any(|p| w.starts_with(p))
    {
        return false;
    }
    w.chars().next().is_some_and(|c| "aeiou".contains(c))
}

const TERMINATORS: [char; 3] = ['.', '!', '?'];

/// Sentence ranges (char offsets) of a text: split after `. ! ?` followed
/// by whitespace or the end, and at line breaks.
pub fn sentences(chars: &[char]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut end_here = false;
        if c == '\n' {
            end_here = true;
        } else if c == '.' && (is_abbreviation(chars, i) || chars.get(i + 1) == Some(&'.')) {
            // "e.g." or an ellipsis: the sentence goes on.
            let mut j = i;
            while j < chars.len() && chars[j] == '.' {
                j += 1;
            }
            i = j;
            continue;
        } else if TERMINATORS.contains(&c) {
            // Consume a run of terminators and closing quotes or brackets.
            let mut j = i + 1;
            while j < chars.len()
                && (TERMINATORS.contains(&chars[j])
                    || matches!(chars[j], '"' | '\'' | ')' | ']' | '”' | '’' | '»'))
            {
                j += 1;
            }
            if j >= chars.len() || chars[j].is_whitespace() {
                i = j - 1;
                end_here = true;
            }
        }
        if end_here {
            push_range(chars, &mut out, start, i + 1);
            start = i + 1;
        }
        i += 1;
    }
    push_range(chars, &mut out, start, chars.len());
    out
}

/// A period after a single letter ("e.g.", "J. Smith") or after another
/// abbreviation point is not the end of a sentence.
fn is_abbreviation(chars: &[char], dot: usize) -> bool {
    let mut k = dot;
    while k > 0 && chars[k - 1].is_alphanumeric() {
        k -= 1;
    }
    let run = dot - k;
    (run == 1 && chars[k].is_alphabetic()) || (run > 0 && k > 0 && chars[k - 1] == '.')
}

fn push_range(chars: &[char], out: &mut Vec<(usize, usize)>, start: usize, end: usize) {
    let mut s = start;
    let mut e = end.min(chars.len());
    while s < e && chars[s].is_whitespace() {
        s += 1;
    }
    while e > s && chars[e - 1].is_whitespace() {
        e -= 1;
    }
    if s < e {
        out.push((s, e));
    }
}

fn sentence_at(chars: &[char], ranges: &[(usize, usize)], pos: usize) -> String {
    let r = ranges
        .iter()
        .find(|(s, e)| pos >= *s && pos < *e)
        .or_else(|| ranges.iter().find(|(s, _)| *s > pos))
        .or_else(|| ranges.last());
    match r {
        Some((s, e)) => chars[*s..*e].iter().collect(),
        None => chars.iter().collect(),
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\'' || c == '’' || c == '-'
}

/// Words as (start, end) char ranges.
fn words(chars: &[char]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if is_word_char(chars[i]) {
            let s = i;
            while i < chars.len() && is_word_char(chars[i]) {
                i += 1;
            }
            out.push((s, i));
        } else {
            i += 1;
        }
    }
    out
}

/// Run every rule over `text`.
pub fn check_text(text: &str, opts: Options) -> Vec<Finding> {
    let chars: Vec<char> = text.chars().collect();
    let ranges = sentences(&chars);
    let mut out = Vec::new();
    let sentence = |pos: usize| sentence_at(&chars, &ranges, pos);

    // Doubled words: "the the", separated by whitespace only.
    let ws = words(&chars);
    for pair in ws.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let gap = &chars[a.1..b.0];
        if gap.is_empty() || !gap.iter().all(|c| *c == ' ' || *c == '\t') {
            continue;
        }
        let wa: String = chars[a.0..a.1].iter().collect::<String>().to_lowercase();
        let wb: String = chars[b.0..b.1].iter().collect::<String>().to_lowercase();
        if wa == wb && wa.chars().any(|c| c.is_alphabetic()) {
            out.push(Finding {
                rule: Rule::DoubledWord,
                sentence: sentence(a.0),
                fix: Some(Fix {
                    start: a.1,
                    end: b.1,
                    text: String::new(),
                }),
            });
        }
    }

    // Double spaces.
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ' ' {
            let s = i;
            while i < chars.len() && chars[i] == ' ' {
                i += 1;
            }
            if i - s >= 2 && s > 0 && i < chars.len() {
                out.push(Finding {
                    rule: Rule::DoubleSpace,
                    sentence: sentence(s),
                    fix: Some(Fix {
                        start: s,
                        end: i,
                        text: " ".into(),
                    }),
                });
            }
        } else {
            i += 1;
        }
    }

    // Space before punctuation.
    for (i, c) in chars.iter().enumerate() {
        let is_low = matches!(c, ',' | '.');
        let is_high = matches!(c, ';' | ':' | '!' | '?');
        if !(is_low || is_high) || (is_high && opts.space_before_high_punctuation) {
            continue;
        }
        if i == 0 || !(chars[i - 1] == ' ' || chars[i - 1] == '\t') {
            continue;
        }
        // "..." written as " . . ." or a lone "." after a space is left alone.
        if *c == '.' && chars.get(i + 1).map(|n| *n == '.').unwrap_or(false) {
            continue;
        }
        let mut s = i;
        while s > 0 && (chars[s - 1] == ' ' || chars[s - 1] == '\t') {
            s -= 1;
        }
        if s == 0 || !is_word_char(chars[s - 1]) {
            continue;
        }
        out.push(Finding {
            rule: Rule::SpaceBeforePunctuation,
            sentence: sentence(i),
            fix: Some(Fix {
                start: s,
                end: i,
                text: String::new(),
            }),
        });
    }

    // Sentence capital and long sentences.
    for (s, e) in &ranges {
        let mut first = *s;
        while first < *e && !chars[first].is_alphanumeric() {
            first += 1;
        }
        if first < *e && chars[first].is_lowercase() {
            let upper: String = chars[first].to_uppercase().collect();
            out.push(Finding {
                rule: Rule::SentenceCapital,
                sentence: chars[*s..*e].iter().collect(),
                fix: Some(Fix {
                    start: first,
                    end: first + 1,
                    text: upper,
                }),
            });
        }
        let n = words(&chars[*s..*e]).len();
        if n > LONG_SENTENCE_WORDS {
            out.push(Finding {
                rule: Rule::LongSentence,
                sentence: chars[*s..*e].iter().collect(),
                fix: None,
            });
        }
    }

    // Missing space after a period or comma between two words.
    let mut i = 1;
    while i + 1 < chars.len() {
        let c = chars[i];
        if (c == '.' || c == ',') && chars[i - 1].is_alphabetic() && chars[i + 1].is_alphabetic() {
            // The token around it: skip abbreviations (e.g.), URLs and addresses.
            let mut ts = i;
            while ts > 0 && !chars[ts - 1].is_whitespace() {
                ts -= 1;
            }
            let mut te = i;
            while te < chars.len() && !chars[te].is_whitespace() {
                te += 1;
            }
            let token: String = chars[ts..te].iter().collect();
            let looks_like_abbrev = c == '.' && is_abbreviation(&chars, i);
            let looks_like_address =
                token.contains("://") || token.contains('@') || token.contains("www.");
            if !looks_like_abbrev && !looks_like_address {
                out.push(Finding {
                    rule: Rule::MissingSpaceAfterPunctuation,
                    sentence: sentence(i),
                    fix: Some(Fix {
                        start: i + 1,
                        end: i + 1,
                        text: " ".into(),
                    }),
                });
            }
        }
        i += 1;
    }

    // Unbalanced brackets (stack over the whole text).
    let mut stack: Vec<char> = Vec::new();
    let mut unbalanced_at: Option<usize> = None;
    for (i, c) in chars.iter().enumerate() {
        match c {
            '(' | '[' | '{' => stack.push(*c),
            ')' | ']' | '}' => {
                let want = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if stack.pop() != Some(want) {
                    unbalanced_at = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    if unbalanced_at.is_none() && !stack.is_empty() {
        unbalanced_at = chars.iter().rposition(|c| matches!(c, '(' | '[' | '{'));
    }
    if let Some(pos) = unbalanced_at {
        out.push(Finding {
            rule: Rule::UnbalancedBrackets,
            sentence: sentence(pos),
            fix: None,
        });
    }

    // Unbalanced quotes: an odd number of straight double quotes, or
    // mismatched curly pairs.
    let count = |ch: char| chars.iter().filter(|c| **c == ch).count();
    let straight = count('"');
    let (low, hi_open, hi_close) = (count('„'), count('“'), count('”'));
    // German „...“ closes with the character English opens with.
    let curly_ok = if low > 0 {
        low == hi_open + hi_close
    } else {
        hi_open == hi_close
    };
    if straight % 2 == 1 || !curly_ok || count('«') != count('»') {
        let pos = chars
            .iter()
            .rposition(|c| matches!(c, '"' | '“' | '”' | '„' | '«' | '»'))
            .unwrap_or(0);
        out.push(Finding {
            rule: Rule::UnbalancedQuotes,
            sentence: sentence(pos),
            fix: None,
        });
    }

    // Repeated punctuation: "!!", "??", ",,", ".." (three dots are an ellipsis).
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c, '!' | '?' | ',' | ';' | ':' | '.') {
            let s = i;
            while i < chars.len() && chars[i] == c {
                i += 1;
            }
            let n = i - s;
            let ellipsis = c == '.' && n == 3;
            if n >= 2 && !ellipsis {
                let text = if c == '.' && n > 3 {
                    "...".to_string()
                } else {
                    c.to_string()
                };
                out.push(Finding {
                    rule: Rule::RepeatedPunctuation,
                    sentence: sentence(s),
                    fix: Some(Fix {
                        start: s,
                        end: i,
                        text,
                    }),
                });
            }
        } else {
            i += 1;
        }
    }

    // A and an, before the next word.
    if opts.english {
        for pair in ws.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let art: String = chars[a.0..a.1].iter().collect();
            let lower = art.to_lowercase();
            if lower != "a" && lower != "an" {
                continue;
            }
            let gap = &chars[a.1..b.0];
            if gap.is_empty() || !gap.iter().all(|c| *c == ' ') {
                continue;
            }
            let next: String = chars[b.0..b.1].iter().collect();
            if !next.chars().next().is_some_and(char::is_alphabetic)
                || next.chars().all(|c| c.is_uppercase())
            {
                continue;
            }
            let an = wants_an(&next);
            if an == (lower == "an") {
                continue;
            }
            let mut text = if an {
                "an".to_string()
            } else {
                "a".to_string()
            };
            if art.starts_with(|c: char| c.is_uppercase()) {
                text = text[..1].to_uppercase() + &text[1..];
            }
            out.push(Finding {
                rule: Rule::ArticleAAn,
                sentence: sentence(a.0),
                fix: Some(Fix {
                    start: a.0,
                    end: a.1,
                    text,
                }),
            });
        }
    }

    // A paragraph's last sentence of three words or more with no end mark.
    let mut start = 0;
    for (i, c) in chars
        .iter()
        .enumerate()
        .chain(std::iter::once((chars.len(), &'\n')))
    {
        if *c != '\n' {
            continue;
        }
        let para: String = chars[start..i].iter().collect();
        let trimmed = para.trim_end();
        let words = trimmed.split_whitespace().count();
        let last = trimmed.chars().last();
        if words >= 3
            && last.is_some_and(|l| l.is_alphanumeric())
            && trimmed.chars().next().is_some_and(char::is_uppercase)
        {
            let end = start + trimmed.chars().count();
            out.push(Finding {
                rule: Rule::MissingSentenceEnd,
                sentence: sentence(end.saturating_sub(1)),
                fix: Some(Fix {
                    start: end,
                    end,
                    text: ".".into(),
                }),
            });
        }
        start = i + 1;
    }

    out.sort_by_key(|f| f.fix.as_ref().map(|x| x.start).unwrap_or(usize::MAX));
    out
}

/// Apply a fix to a text.
pub fn apply_fix(text: &str, fix: &Fix) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = fix.start.min(chars.len());
    let end = fix.end.clamp(start, chars.len());
    let mut out: String = chars[..start].iter().collect();
    out.push_str(&fix.text);
    out.extend(chars[end..].iter());
    out
}

/// Text of a text shape, spans joined.
fn shape_text(spans: &[TextSpan]) -> String {
    spans.iter().map(|s| s.text.as_str()).collect()
}

/// Apply `fix` to the shape's text through the command path. The span
/// containing the range is edited in place; a range crossing span
/// boundaries merges the text into the first span's style.
pub fn apply_fix_to_shape(app: &mut App, id: ShapeId, fix: &Fix) {
    let Some(s) = app.doc().find_shape(id).cloned() else {
        return;
    };
    let ShapeKind::Text {
        spans,
        origin,
        frame,
        align,
        para,
        on_path,
    } = s.kind
    else {
        return;
    };
    let new_spans = apply_fix_to_spans(spans, fix);
    app.run(Command::SetShapeKind {
        shape: id,
        kind: ShapeKind::Text {
            spans: new_spans,
            origin,
            frame,
            align,
            para,
            on_path,
        },
    });
}

fn apply_fix_to_spans(spans: Vec<TextSpan>, fix: &Fix) -> Vec<TextSpan> {
    let mut offset = 0;
    for (i, sp) in spans.iter().enumerate() {
        let len = sp.text.chars().count();
        if fix.start >= offset && fix.end <= offset + len {
            let local = Fix {
                start: fix.start - offset,
                end: fix.end - offset,
                text: fix.text.clone(),
            };
            let mut out = spans.clone();
            out[i].text = apply_fix(&sp.text, &local);
            return out;
        }
        offset += len;
    }
    let joined = shape_text(&spans);
    let Some(first) = spans.into_iter().next() else {
        return Vec::new();
    };
    vec![TextSpan {
        text: apply_fix(&joined, fix),
        ..first
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(text: &str) -> Vec<Rule> {
        check_text(text, Options::default())
            .into_iter()
            .map(|f| f.rule)
            .collect()
    }

    fn fixed(text: &str, rule: Rule) -> String {
        let f = check_text(text, Options::default())
            .into_iter()
            .find(|f| f.rule == rule)
            .expect("finding");
        apply_fix(text, f.fix.as_ref().expect("fix"))
    }

    #[test]
    fn clean_text_has_no_findings() {
        assert!(rules("The cat sat on the mat. It was happy!").is_empty());
        assert!(rules("Visit www.example.com or e.g. mail a@b.c today.").is_empty());
        assert!(rules("Wait... really?").is_empty());
    }

    #[test]
    fn doubled_word() {
        assert_eq!(rules("See the the cat."), vec![Rule::DoubledWord]);
        assert_eq!(fixed("See the The cat.", Rule::DoubledWord), "See the cat.");
        assert!(rules("It was 10 10 wide.").is_empty());
    }

    #[test]
    fn spaces() {
        assert_eq!(rules("A  b."), vec![Rule::DoubleSpace]);
        assert_eq!(fixed("A   b.", Rule::DoubleSpace), "A b.");
        assert_eq!(rules("Hello , world."), vec![Rule::SpaceBeforePunctuation]);
        assert_eq!(
            fixed("Hello , world.", Rule::SpaceBeforePunctuation),
            "Hello, world."
        );
        let fr = Options::for_language("fr");
        assert!(check_text("Bonjour !", fr).is_empty());
        assert_eq!(rules("Bonjour !"), vec![Rule::SpaceBeforePunctuation]);
    }

    #[test]
    fn capitals_and_missing_space() {
        assert_eq!(rules("this is it."), vec![Rule::SentenceCapital]);
        assert_eq!(
            fixed("Ok. this is it.", Rule::SentenceCapital),
            "Ok. This is it."
        );
        assert_eq!(
            fixed("Ok.\nthis is it.", Rule::SentenceCapital),
            "Ok.\nThis is it."
        );
        assert_eq!(
            rules("Hello,world."),
            vec![Rule::MissingSpaceAfterPunctuation]
        );
        assert_eq!(
            fixed("One.Two.", Rule::MissingSpaceAfterPunctuation),
            "One. Two."
        );
        assert!(rules("Pi is 3.14, or 3,14 here.").is_empty());
    }

    #[test]
    fn brackets_quotes_and_repeats() {
        assert_eq!(rules("A (b c."), vec![Rule::UnbalancedBrackets]);
        assert_eq!(rules("A b) c."), vec![Rule::UnbalancedBrackets]);
        assert!(rules("A (b [c] d) e.").is_empty());
        assert_eq!(rules("He said \"hi."), vec![Rule::UnbalancedQuotes]);
        assert!(rules("He said \"hi\" and “bye”.").is_empty());
        assert_eq!(
            rules("Wow!! Really??"),
            vec![Rule::RepeatedPunctuation, Rule::RepeatedPunctuation]
        );
        assert_eq!(fixed("Wow!! Yes.", Rule::RepeatedPunctuation), "Wow! Yes.");
        assert_eq!(
            fixed("Wait.... Yes.", Rule::RepeatedPunctuation),
            "Wait... Yes."
        );
    }

    #[test]
    fn long_sentence() {
        let long = (0..45)
            .map(|i| format!("W{i}"))
            .collect::<Vec<_>>()
            .join(" ")
            + ".";
        let r = rules(&format!("Fine. {long}"));
        assert_eq!(r, vec![Rule::LongSentence]);
        let f = &check_text(&format!("Fine. {long}"), Options::default())[0];
        assert!(f.fix.is_none());
        assert!(f.sentence.starts_with("W0 W1"));
        assert!(rules("Yes, e.g. this one. And U.S. too.").is_empty());
    }

    #[test]
    fn fixes_stay_inside_their_span() {
        let spans = vec![
            TextSpan::new("See the ", "A", 12.0),
            TextSpan::new("the cat.", "B", 12.0),
        ];
        let text = shape_text(&spans);
        let f = check_text(&text, Options::default())
            .into_iter()
            .find(|f| f.rule == Rule::DoubledWord)
            .unwrap();
        // Crosses the span boundary: merged into one span.
        let out = apply_fix_to_spans(spans.clone(), f.fix.as_ref().unwrap());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text, "See the cat.");
        assert_eq!(out[0].font_family, "A");
        // Inside the second span: style preserved.
        let fix = Fix {
            start: 12,
            end: 12,
            text: "big ".into(),
        };
        let out = apply_fix_to_spans(spans, &fix);
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].text, "the big cat.");
    }

    #[test]
    fn malformed_input_never_panics() {
        for s in ["", " ", "...", "(((", "\"", "\n\n\n", ".a", "a."] {
            let _ = check_text(s, Options::default());
            let _ = apply_fix(
                s,
                &Fix {
                    start: 99,
                    end: 5,
                    text: "x".into(),
                },
            );
        }
    }

    #[test]
    fn a_and_an_follow_the_next_sound_and_sentences_need_an_end() {
        let en = Options::for_language("en");
        let f = check_text("I ate a apple and an banana in an hour.", en);
        let fixes: Vec<_> = f
            .iter()
            .filter(|x| x.rule == Rule::ArticleAAn)
            .filter_map(|x| x.fix.clone())
            .collect();
        assert_eq!(fixes.len(), 2, "{f:?}");
        assert_eq!(fixes[0].text, "an");
        assert_eq!(fixes[1].text, "a");
        // "a one" and "an hour" are right; other languages are not checked.
        assert!(!check_text("It is a one off, an hour long.", en)
            .iter()
            .any(|x| x.rule == Rule::ArticleAAn));
        assert!(!check_text("Vi a apple.", Options::for_language("pt"))
            .iter()
            .any(|x| x.rule == Rule::ArticleAAn));
        let f = check_text("This sentence has no end", en);
        let end = f
            .iter()
            .find(|x| x.rule == Rule::MissingSentenceEnd)
            .expect("missing end");
        assert_eq!(
            apply_fix("This sentence has no end", end.fix.as_ref().expect("fix")),
            "This sentence has no end."
        );
        assert!(!check_text("Two words", en)
            .iter()
            .any(|x| x.rule == Rule::MissingSentenceEnd));
    }
}
