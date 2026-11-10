//! Hyphenation: break points inside a word (character indices after which
//! a hyphen may be inserted). Uses the pattern file the app installs (a
//! hyphenation dictionary of Liang patterns, `hyph_*.dic`, see
//! [`PatternSet::parse`]); otherwise a compact Liang-style pattern set for
//! English and Portuguese plus a vowel/consonant heuristic for everything
//! else. Minimum 2 letters before and 3 after a break by default.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// Patterns read from a hyphenation dictionary: one or two levels (the
/// second after `NEXTLEVEL`; a word breaks where either level allows it),
/// and the fewest letters before and after a break.
#[derive(Debug, Clone, Default)]
pub struct PatternSet {
    levels: Vec<Patterns>,
    longest: usize,
    pub left_min: usize,
    pub right_min: usize,
}

impl PatternSet {
    /// Read a hyphenation dictionary already decoded: the first line names
    /// its character set, `LEFTHYPHENMIN` and `RIGHTHYPHENMIN` lines set
    /// the minimums, `NEXTLEVEL` starts a second level, `%` and `#` start
    /// comments, and every other line is a Liang pattern (digits between
    /// letters, `.` for the word's ends). Patterns with a replacement
    /// (`/`) are left out. `None` when no pattern is found.
    pub fn parse(text: &str) -> Option<PatternSet> {
        let mut set = PatternSet {
            levels: vec![Patterns::new()],
            longest: 0,
            left_min: MIN_BEFORE,
            right_min: MIN_AFTER,
        };
        for (n, line) in text.lines().enumerate() {
            let line = line.trim_start_matches('\u{feff}').trim();
            if n == 0 || line.is_empty() || line.starts_with('%') || line.starts_with('#') {
                continue;
            }
            let mut words = line.split_whitespace();
            let first = words.next().unwrap_or("");
            let number = || words.clone().next().and_then(|v| v.parse::<usize>().ok());
            match first {
                "LEFTHYPHENMIN" => {
                    if let Some(v) = number() {
                        set.left_min = v.max(1);
                    }
                    continue;
                }
                "RIGHTHYPHENMIN" => {
                    if let Some(v) = number() {
                        set.right_min = v.max(1);
                    }
                    continue;
                }
                "NEXTLEVEL" => {
                    set.levels.push(Patterns::new());
                    continue;
                }
                _ => {}
            }
            if first.chars().all(|c| c.is_ascii_uppercase() || c == '_') || first.contains('/') {
                continue;
            }
            let (letters, digits) = parse_one(first);
            if letters.is_empty() {
                continue;
            }
            set.longest = set.longest.max(letters.chars().count());
            if let Some(level) = set.levels.last_mut() {
                level.insert(letters.to_lowercase(), digits);
            }
        }
        set.levels.retain(|l| !l.is_empty());
        (!set.levels.is_empty()).then_some(set)
    }

    /// How many patterns the levels hold.
    pub fn len(&self) -> usize {
        self.levels.iter().map(HashMap::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The break points of `word` (small letters).
    fn points(&self, lower: &[char]) -> Vec<usize> {
        let mut out = Vec::new();
        for level in &self.levels {
            out.extend(liang(
                lower,
                level,
                self.longest,
                self.left_min,
                self.right_min,
            ));
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// The installed pattern file, if any.
fn installed() -> &'static RwLock<Option<Arc<PatternSet>>> {
    static SET: OnceLock<RwLock<Option<Arc<PatternSet>>>> = OnceLock::new();
    SET.get_or_init(|| RwLock::new(None))
}

/// Hyphenate with `set` from now on (`None`: the built-in patterns).
pub fn set_patterns(set: Option<PatternSet>) {
    *installed().write().unwrap_or_else(|e| e.into_inner()) = set.map(Arc::new);
}

/// Words with hyphens the user placed (custom optional hyphens): the word
/// in lower case and its break points, used before the patterns.
fn custom() -> &'static RwLock<HashMap<String, Vec<usize>>> {
    static CUSTOM: OnceLock<RwLock<HashMap<String, Vec<usize>>>> = OnceLock::new();
    CUSTOM.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Replace the custom optional hyphens: each word (any case) with the
/// character counts after which it may break.
pub fn set_custom(defs: Vec<(String, Vec<usize>)>) {
    let mut map = custom().write().unwrap_or_else(|e| e.into_inner());
    map.clear();
    for (w, p) in defs {
        map.insert(w.to_lowercase(), p);
    }
}

const MIN_BEFORE: usize = 2;
const MIN_AFTER: usize = 3;

/// A few common Liang patterns; digits mark odd (allowed) and even
/// (forbidden) break positions. Enough for ordinary words; the heuristic
/// handles the rest.
const PATTERNS_EN: &str = ".ab3a .ad4der .an3ti .ar5s .as3c .be5g .co3e .de3o .dis1 .ex1 .im1p .in1 .mis1 .non1 .out1 .over1 .pre1 .re1 .un1 .un3der 1ba 1be 1bi 1bo 1bu 1ca 1ce 1ci 1co 1cu 1da 1de 1di 1do 1du 1fa 1fe 1fi 1fo 1fu 1ga 1ge 1gi 1go 1gu 1ha 1he 1hi 1ho 1hu 1ja 1je 1jo 1ju 1ka 1ke 1ki 1ko 1la 1le 1li 1lo 1lu 1ma 1me 1mi 1mo 1mu 1na 1ne 1ni 1no 1nu 1pa 1pe 1pi 1po 1pu 1ra 1re 1ri 1ro 1ru 1sa 1se 1si 1so 1su 1ta 1te 1ti 1to 1tu 1va 1ve 1vi 1vo 1vu 1wa 1we 1wi 1wo 1za 1ze 1zo 2ch 2ck 2ll 2ss 2tt 2st 2sh 2th 2ph 2ng c2h c2k g2h p2h s2h t2h w2h q2u 4ed. 4er. 2es. 4ing. 2ly. 1ment 2ness 1tion 1sion 4ful. 2ble. 2ple. 2tle. 2gle. 2cle. 2dle. 2kle. 2zle. 4ous. 1ly 2ed2 3ment. 2ly. 1er 2ers. 1ing 2ngs. 1al 2ally";

const PATTERNS_PT: &str = "1ba 1be 1bi 1bo 1bu 1ca 1ce 1ci 1co 1cu 1da 1de 1di 1do 1du 1fa 1fe 1fi 1fo 1fu 1ga 1ge 1gi 1go 1gu 1ja 1je 1ji 1jo 1ju 1la 1le 1li 1lo 1lu 1ma 1me 1mi 1mo 1mu 1na 1ne 1ni 1no 1nu 1pa 1pe 1pi 1po 1pu 1ra 1re 1ri 1ro 1ru 1sa 1se 1si 1so 1su 1ta 1te 1ti 1to 1tu 1va 1ve 1vi 1vo 1vu 1xa 1xe 1xi 1xo 1xu 1za 1ze 1zi 1zo 1zu 1ça 1ço 1çu 2ch 2lh 2nh 2qu 2gu 2rr 2ss 1br 1bl 1cr 1cl 1dr 1fr 1fl 1gr 1gl 1pr 1pl 1tr 1vr 2bs 2ns 2rs 2ls 2ps 1ção 1mente";

type Patterns = HashMap<String, Vec<u8>>;

/// One pattern as its letters and the level of each gap (one more than
/// the letters).
fn parse_one(pat: &str) -> (String, Vec<u8>) {
    let mut letters = String::new();
    let mut digits: Vec<u8> = Vec::new();
    let mut pending = 0u8;
    for c in pat.chars() {
        if let Some(d) = c.to_digit(10) {
            pending = d as u8;
        } else {
            digits.push(pending);
            pending = 0;
            letters.push(c);
        }
    }
    digits.push(pending);
    (letters, digits)
}

fn parse(src: &str) -> Patterns {
    src.split_whitespace().map(parse_one).collect()
}

/// Liang's algorithm: the odd gaps of `lower` (small letters) under
/// `pats`, at least `left` letters from the start and `right` from the end.
fn liang(lower: &[char], pats: &Patterns, longest: usize, left: usize, right: usize) -> Vec<usize> {
    let dotted: Vec<char> = std::iter::once('.')
        .chain(lower.iter().copied())
        .chain(std::iter::once('.'))
        .collect();
    let n = dotted.len();
    let mut levels = vec![0u8; n + 1];
    for start in 0..n {
        for end in start + 1..=n.min(start + longest) {
            let key: String = dotted[start..end].iter().collect();
            if let Some(d) = pats.get(&key) {
                for (k, v) in d.iter().enumerate() {
                    let idx = start + k;
                    if idx < levels.len() && *v > levels[idx] {
                        levels[idx] = *v;
                    }
                }
            }
        }
    }
    // levels[j] is the gap before dotted[j]; a break after `i` letters of
    // the word is the gap before lower[i], that is before dotted[i + 1].
    (1..lower.len())
        .filter(|&i| levels.get(i + 1).is_some_and(|l| l % 2 == 1))
        .filter(|&i| i >= left && lower.len() - i >= right)
        .collect()
}

fn patterns(lang: &str) -> &'static Patterns {
    static EN: OnceLock<Patterns> = OnceLock::new();
    static PT: OnceLock<Patterns> = OnceLock::new();
    if lang.starts_with("pt") || lang.starts_with("es") {
        PT.get_or_init(|| parse(PATTERNS_PT))
    } else {
        EN.get_or_init(|| parse(PATTERNS_EN))
    }
}

/// Liang's algorithm over the pattern set, then the heuristic for words
/// the patterns leave unbroken.
/// Words with a capital letter (names, acronyms) are not broken.
/// Punctuation before or after the word ("parola," at a line's end, a
/// quotation) is left aside and the letters between broken; an apostrophe
/// inside the word ("dell'anno", "l’università") is kept for the patterns
/// that know it, and without a pattern file only the part after the last
/// one breaks (#255).
pub fn break_points_lang(word: &str, lang: &str) -> Vec<usize> {
    let all: Vec<char> = word.chars().collect();
    // The user's own hyphens win, capitals or not.
    let mine = |w: &str| {
        custom()
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&w.to_lowercase())
            .cloned()
    };
    if let Some(p) = mine(word) {
        return p
            .iter()
            .copied()
            .filter(|&i| i > 0 && i < all.len())
            .collect();
    }
    let Some(start) = all.iter().position(|c| c.is_alphabetic()) else {
        return Vec::new();
    };
    let end = all
        .iter()
        .rposition(|c| c.is_alphabetic())
        .map_or(start, |i| i + 1);
    let chars = &all[start..end];
    let shift = |p: Vec<usize>| -> Vec<usize> { p.into_iter().map(|i| i + start).collect() };
    if start > 0 || end < all.len() {
        let core: String = chars.iter().collect();
        if let Some(p) = mine(&core) {
            return shift(
                p.into_iter()
                    .filter(|&i| i > 0 && i < chars.len())
                    .collect(),
            );
        }
    }
    if !chars.iter().all(|c| c.is_alphabetic() || is_apostrophe(*c)) {
        return Vec::new();
    }
    let lower: Vec<char> = chars
        .iter()
        .flat_map(|c| c.to_lowercase())
        .map(|c| if is_apostrophe(c) { '\'' } else { c })
        .collect();
    // A pattern file: words with a capital first letter break too (nouns
    // in some languages); other capitals (acronyms, names in capitals,
    // inner capitals) do not.
    let set = installed()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if set.is_none() {
        // The built-in patterns know no apostrophes: the part after the
        // last one.
        if let Some(a) = chars.iter().rposition(|c| is_apostrophe(*c)) {
            let after = a + 1;
            let p = break_points_with(&chars[after..], &lower[after..], lang, None);
            return shift(p.into_iter().map(|i| i + after).collect());
        }
    }
    shift(break_points_with(chars, &lower, lang, set.as_deref()))
}

/// A straight or a typographic apostrophe.
fn is_apostrophe(c: char) -> bool {
    c == '\'' || c == '\u{2019}'
}

/// The break points of a word of letters (`chars`, and `lower` in small
/// letters) with the pattern file `set`, else the built-in patterns.
fn break_points_with(
    chars: &[char],
    lower: &[char],
    lang: &str,
    set: Option<&PatternSet>,
) -> Vec<usize> {
    if let Some(set) = set {
        if chars.iter().skip(1).any(|c| c.is_uppercase()) || lower.len() != chars.len() {
            return Vec::new();
        }
        return set.points(lower);
    }
    if chars.len() < MIN_BEFORE + MIN_AFTER || chars.iter().any(|c| c.is_uppercase()) {
        return Vec::new();
    }
    let pats = patterns(lang);
    let mut out = liang(lower, pats, 8, MIN_BEFORE, MIN_AFTER);
    if out.is_empty() {
        out = heuristic(lower);
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Default language: English patterns.
pub fn break_points(word: &str) -> Vec<usize> {
    break_points_lang(word, "en")
}

fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'y'
            | 'á'
            | 'é'
            | 'í'
            | 'ó'
            | 'ú'
            | 'â'
            | 'ê'
            | 'ô'
            | 'ã'
            | 'õ'
            | 'à'
    )
}

/// Break between a vowel and a following consonant+vowel group (V-CV).
fn heuristic(w: &[char]) -> Vec<usize> {
    let mut out = Vec::new();
    for i in 1..w.len().saturating_sub(1) {
        if is_vowel(w[i - 1])
            && !is_vowel(w[i])
            && i + 1 < w.len()
            && is_vowel(w[i + 1])
            && i >= MIN_BEFORE
            && w.len() - i >= MIN_AFTER
        {
            out.push(i);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn respects_minimums() {
        assert!(break_points("abc").is_empty());
        for p in break_points("information") {
            assert!(p >= 2 && 11 - p >= 3);
        }
    }

    #[test]
    fn finds_a_break_in_common_words() {
        assert!(!break_points("hyphenation").is_empty());
        assert!(!break_points_lang("computador", "pt").is_empty());
    }

    #[test]
    fn breaks_fall_on_the_pattern_positions() {
        // "-tion" breaks before the t, never one letter later.
        let p = break_points("instructions");
        assert!(p.contains(&7), "{p:?}");
        assert!(!p.contains(&8), "{p:?}");
        assert!(break_points("information").contains(&7));
        // Digraphs stay together.
        assert!(!break_points("telephone").contains(&5));
        assert!(!break_points("achievement").contains(&2));
    }

    #[test]
    fn names_and_acronyms_are_not_broken() {
        assert!(break_points("TraceDraw").is_empty());
        assert!(break_points("Portugal").is_empty());
        assert!(break_points("UNESCO").is_empty());
    }

    /// A hyphenation dictionary: the minimums, comments, a second level,
    /// replacement patterns left out; installed, it replaces the built-in
    /// patterns, and a capital first letter no longer stops a break.
    #[test]
    fn pattern_files() {
        let text = "UTF-8\nLEFTHYPHENMIN 1\nRIGHTHYPHENMIN 1\n% a comment\n1ba\n1na\nc1k/k=k,1,2\nNEXTLEVEL\n1nas\n";
        let set = PatternSet::parse(text).expect("patterns");
        assert_eq!((set.left_min, set.right_min), (1, 1));
        assert_eq!(set.len(), 3);
        assert_eq!(
            set.points(&"banana".chars().collect::<Vec<_>>()),
            vec![2, 4]
        );
        assert_eq!(
            set.points(&"bananas".chars().collect::<Vec<_>>()),
            vec![2, 4]
        );
        assert!(PatternSet::parse("UTF-8\n% nothing\n").is_none());
        // As installed (not installed here: other tests run meanwhile).
        let with = |w: &str| {
            let chars: Vec<char> = w.chars().collect();
            let lower: Vec<char> = w.to_lowercase().chars().collect();
            break_points_with(&chars, &lower, "en", Some(&set))
        };
        assert_eq!(with("Banana"), vec![2, 4]);
        assert!(with("BANANA").is_empty());
        assert!(with("baNana").is_empty());
        assert!(
            break_points("Banana").is_empty(),
            "built-in: names stay whole"
        );
    }

    /// A word with punctuation round it breaks between its letters, the
    /// points counted in the whole word; an apostrophe inside it is kept
    /// (the built-in patterns break after it only); other marks inside a
    /// word still keep it whole (#255).
    #[test]
    fn punctuation_round_a_word_and_apostrophes_inside_it() {
        let bare = break_points("information");
        assert!(!bare.is_empty());
        assert_eq!(break_points("information,"), bare);
        assert_eq!(break_points("information."), bare);
        let shifted: Vec<usize> = bare.iter().map(|i| i + 1).collect();
        assert_eq!(break_points("\u{201c}information\u{201d}"), shifted);
        let after: Vec<usize> = break_points("information").iter().map(|i| i + 2).collect();
        assert_eq!(break_points("l'information"), after);
        assert_eq!(break_points("l\u{2019}information"), after);
        assert!(break_points("e-mail").is_empty());
        assert!(break_points("...").is_empty());
        // A pattern set that knows the apostrophe breaks before it.
        let set = PatternSet::parse("UTF-8\n1l'\n").expect("set");
        let w: Vec<char> = "dell'anno".chars().collect();
        assert_eq!(break_points_with(&w, &w, "it", Some(&set)), vec![3]);
    }

    #[test]
    fn skips_non_letters() {
        assert!(break_points("12345678").is_empty());
    }
}
