//! Hunspell dictionaries: a `.dic` word list with an `.aff` affix file,
//! as spell checkers of many programs install them and as they can be
//! downloaded for most languages. Written from the public description of
//! the format (the hunspell(5) manual page): the file encoding (`SET`),
//! flag types (`FLAG`), flag aliases (`AF`), prefixes and suffixes
//! (`PFX`, `SFX`) with their conditions, cross products and continuation
//! classes, `NEEDAFFIX`, `FORBIDDENWORD`, `ONLYINCOMPOUND`, `KEEPCASE`,
//! compounding (`COMPOUNDFLAG`, `COMPOUNDBEGIN`, `COMPOUNDMIDDLE`,
//! `COMPOUNDLAST`, `COMPOUNDPERMITFLAG`, `COMPOUNDMIN`, `COMPOUNDWORDMAX`),
//! `IGNORE`, and for
//! suggestions `TRY` and `REP`.

use std::collections::HashMap;

/// A flag, whatever its written form (one character, two characters, a
/// number).
type Flag = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlagKind {
    /// One character per flag (the default; also `FLAG UTF-8`).
    Char,
    /// Two characters per flag (`FLAG long`).
    Long,
    /// Decimal numbers separated by commas (`FLAG num`).
    Num,
}

/// One position of an affix condition: any character, one character, or
/// a set (negated with `^`).
#[derive(Debug, Clone, PartialEq)]
enum Cond {
    Any,
    One(char),
    Set(Vec<char>, bool),
}

impl Cond {
    fn matches(&self, c: char) -> bool {
        match self {
            Cond::Any => true,
            Cond::One(x) => *x == c,
            Cond::Set(set, negated) => set.contains(&c) != *negated,
        }
    }
}

fn parse_condition(s: &str) -> Vec<Cond> {
    if s == "." || s.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '.' => out.push(Cond::Any),
            '[' => {
                let negated = it.peek() == Some(&'^');
                if negated {
                    it.next();
                }
                let mut set = Vec::new();
                for d in it.by_ref() {
                    if d == ']' {
                        break;
                    }
                    set.push(d);
                }
                out.push(Cond::Set(set, negated));
            }
            c => out.push(Cond::One(c)),
        }
    }
    out
}

#[derive(Debug, Clone)]
struct Affix {
    flag: Flag,
    cross: bool,
    strip: Vec<char>,
    add: Vec<char>,
    cond: Vec<Cond>,
    /// Continuation classes: the affixes that may come on top of this one.
    cont: Vec<Flag>,
}

/// A loaded dictionary.
#[derive(Debug, Default)]
pub struct Hunspell {
    flag_kind: Option<FlagKind>,
    aliases: Vec<Vec<Flag>>,
    /// Each word with the flag sets of its entries (a word may be listed
    /// more than once).
    words: HashMap<String, Vec<Vec<Flag>>>,
    prefixes: Vec<Affix>,
    suffixes: Vec<Affix>,
    /// Suffixes by the last character they add (`None`: they add none).
    suffix_index: HashMap<Option<char>, Vec<usize>>,
    prefix_index: HashMap<Option<char>, Vec<usize>>,
    need_affix: Option<Flag>,
    forbidden: Option<Flag>,
    only_in_compound: Option<Flag>,
    keep_case: Option<Flag>,
    compound_flag: Option<Flag>,
    compound_begin: Option<Flag>,
    compound_middle: Option<Flag>,
    compound_last: Option<Flag>,
    compound_permit: Option<Flag>,
    compound_min: usize,
    compound_word_max: usize,
    ignore: Vec<char>,
    try_chars: Vec<char>,
    rep: Vec<(String, String)>,
}

/// The encoding a `SET` line names, as the decoder knows it.
pub fn encoding_for(name: &str) -> &'static encoding_rs::Encoding {
    let n = name.trim().to_ascii_lowercase();
    let label = if let Some(cp) = n.strip_prefix("microsoft-cp") {
        format!("windows-{cp}")
    } else if let Some(rest) = n.strip_prefix("iso8859-") {
        format!("iso-8859-{rest}")
    } else if n.starts_with("tis620") {
        "tis-620".to_string()
    } else {
        n
    };
    encoding_rs::Encoding::for_label(label.as_bytes()).unwrap_or(encoding_rs::UTF_8)
}

/// The `SET` of an affix file read as bytes (ASCII-compatible lines).
fn declared_encoding(aff: &[u8]) -> &'static encoding_rs::Encoding {
    for line in aff.split(|b| *b == b'\n') {
        let line = String::from_utf8_lossy(line);
        let mut parts = line.split_whitespace();
        if parts.next() == Some("SET") {
            if let Some(name) = parts.next() {
                return encoding_for(name);
            }
        }
    }
    encoding_rs::UTF_8
}

fn decode(bytes: &[u8], enc: &'static encoding_rs::Encoding) -> String {
    // A byte order mark, if any, decides.
    let (text, _, _) = enc.decode(bytes);
    text.into_owned()
}

impl Hunspell {
    /// Read a dictionary from its affix and word files.
    pub fn from_bytes(aff: &[u8], dic: &[u8]) -> Hunspell {
        let enc = declared_encoding(aff);
        let mut h = Hunspell {
            compound_min: 3,
            ..Default::default()
        };
        h.read_aff(&decode(aff, enc));
        h.read_dic(&decode(dic, enc));
        h
    }

    /// A plain word list (a `.dic` file with no affix file).
    pub fn from_word_list(dic: &[u8]) -> Hunspell {
        let mut h = Hunspell {
            compound_min: 3,
            ..Default::default()
        };
        h.read_dic(&decode(dic, encoding_rs::UTF_8));
        h
    }

    /// How many entries the word file has.
    pub fn len(&self) -> usize {
        self.words.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    fn kind(&self) -> FlagKind {
        self.flag_kind.unwrap_or(FlagKind::Char)
    }

    /// The flags written in `s`, or an alias number's flags.
    fn flags(&self, s: &str) -> Vec<Flag> {
        if !self.aliases.is_empty() {
            if let Ok(n) = s.trim().parse::<usize>() {
                return self
                    .aliases
                    .get(n.wrapping_sub(1))
                    .cloned()
                    .unwrap_or_default();
            }
        }
        self.parse_flags(s)
    }

    fn parse_flags(&self, s: &str) -> Vec<Flag> {
        match self.kind() {
            FlagKind::Char => s.chars().map(|c| c as Flag).collect(),
            FlagKind::Long => {
                let cs: Vec<char> = s.chars().collect();
                cs.chunks(2)
                    .map(|p| ((p[0] as Flag) << 16) | p.get(1).map_or(0, |c| *c as Flag))
                    .collect()
            }
            FlagKind::Num => s
                .split(',')
                .filter_map(|n| n.trim().parse::<Flag>().ok())
                .collect(),
        }
    }

    fn one_flag(&self, s: Option<&str>) -> Option<Flag> {
        s.and_then(|s| self.parse_flags(s).first().copied())
    }

    fn read_aff(&mut self, text: &str) {
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim_start_matches('\u{feff}');
            i += 1;
            let mut p = line.split_whitespace();
            let Some(key) = p.next() else { continue };
            match key {
                "FLAG" => {
                    self.flag_kind = match p.next() {
                        Some("long") => Some(FlagKind::Long),
                        Some("num") => Some(FlagKind::Num),
                        _ => Some(FlagKind::Char),
                    }
                }
                "AF" => {
                    // The count line, then one line of flags per alias.
                    if let Some(f) = p.next() {
                        if f.parse::<usize>().is_err() {
                            let flags = self.parse_flags(f);
                            self.aliases.push(flags);
                        }
                    }
                }
                "NEEDAFFIX" | "PSEUDOROOT" => self.need_affix = self.one_flag(p.next()),
                "FORBIDDENWORD" => self.forbidden = self.one_flag(p.next()),
                "ONLYINCOMPOUND" => self.only_in_compound = self.one_flag(p.next()),
                "KEEPCASE" => self.keep_case = self.one_flag(p.next()),
                "COMPOUNDFLAG" => self.compound_flag = self.one_flag(p.next()),
                "COMPOUNDBEGIN" => self.compound_begin = self.one_flag(p.next()),
                "COMPOUNDMIDDLE" => self.compound_middle = self.one_flag(p.next()),
                "COMPOUNDLAST" | "COMPOUNDEND" => self.compound_last = self.one_flag(p.next()),
                "COMPOUNDPERMITFLAG" => self.compound_permit = self.one_flag(p.next()),
                "COMPOUNDMIN" => {
                    if let Some(n) = p.next().and_then(|n| n.parse::<usize>().ok()) {
                        self.compound_min = n.max(1);
                    }
                }
                "COMPOUNDWORDMAX" => {
                    if let Some(n) = p.next().and_then(|n| n.parse::<usize>().ok()) {
                        self.compound_word_max = n;
                    }
                }
                "IGNORE" => self.ignore = p.next().unwrap_or("").chars().collect(),
                "TRY" => self.try_chars = p.next().unwrap_or("").chars().collect(),
                "REP" => {
                    if let (Some(a), Some(b)) = (p.next(), p.next()) {
                        if a.parse::<usize>().is_err() {
                            self.rep.push((a.replace('_', " "), b.replace('_', " ")));
                        }
                    }
                }
                "PFX" | "SFX" => {
                    let (Some(flag), Some(cross), Some(count)) = (p.next(), p.next(), p.next())
                    else {
                        continue;
                    };
                    let Ok(count) = count.parse::<usize>() else {
                        continue;
                    };
                    let flag = self.parse_flags(flag).first().copied().unwrap_or(0);
                    let cross = cross == "Y";
                    for _ in 0..count {
                        let Some(rule) = lines.get(i) else { break };
                        i += 1;
                        let mut r = rule.split_whitespace();
                        if r.next() != Some(key) {
                            continue;
                        }
                        let _flag = r.next();
                        let strip = r.next().unwrap_or("0");
                        let add = r.next().unwrap_or("0");
                        let cond = r.next().unwrap_or(".");
                        let (add, cont) = match add.split_once('/') {
                            Some((a, c)) => (a, self.flags(c)),
                            None => (add, Vec::new()),
                        };
                        let chars = |s: &str| -> Vec<char> {
                            if s == "0" {
                                Vec::new()
                            } else {
                                s.chars().collect()
                            }
                        };
                        let mut add = chars(add);
                        add.retain(|c| !self.ignore.contains(c));
                        let affix = Affix {
                            flag,
                            cross,
                            strip: chars(strip),
                            add,
                            cond: parse_condition(cond),
                            cont,
                        };
                        if key == "PFX" {
                            let k = affix.add.first().copied();
                            self.prefix_index
                                .entry(k)
                                .or_default()
                                .push(self.prefixes.len());
                            self.prefixes.push(affix);
                        } else {
                            let k = affix.add.last().copied();
                            self.suffix_index
                                .entry(k)
                                .or_default()
                                .push(self.suffixes.len());
                            self.suffixes.push(affix);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn read_dic(&mut self, text: &str) {
        for (n, line) in text.lines().enumerate() {
            let line = line.trim_start_matches('\u{feff}');
            // The first line is the entry count; tab-led lines are comments.
            if n == 0 && line.trim().chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if line.starts_with('\t') || line.starts_with('#') {
                continue;
            }
            // The word ends at an unescaped slash (its flags) or at white
            // space (morphological fields).
            let mut word = String::new();
            let mut flags = String::new();
            let mut chars = line.chars().peekable();
            let mut in_flags = false;
            while let Some(c) = chars.next() {
                if c == '\\' && chars.peek() == Some(&'/') && !in_flags {
                    word.push('/');
                    chars.next();
                } else if c == '/' && !in_flags && !word.is_empty() {
                    in_flags = true;
                } else if c.is_whitespace() {
                    break;
                } else if in_flags {
                    flags.push(c);
                } else {
                    word.push(c);
                }
            }
            word.retain(|c| !self.ignore.contains(&c));
            if word.is_empty() {
                continue;
            }
            let flags = if flags.is_empty() {
                Vec::new()
            } else {
                self.flags(&flags)
            };
            self.words.entry(word).or_default().push(flags);
        }
    }

    fn entries(&self, w: &str) -> &[Vec<Flag>] {
        self.words.get(w).map_or(&[], Vec::as_slice)
    }

    fn has(set: &[Flag], f: Option<Flag>) -> bool {
        f.is_some_and(|f| set.contains(&f))
    }

    /// Whether `word` is spelled right.
    pub fn check(&self, word: &str) -> bool {
        let word: String = word
            .chars()
            .map(|c| if c == '’' { '\'' } else { c })
            .filter(|c| !self.ignore.contains(c))
            .collect();
        if word.is_empty() {
            return true;
        }
        self.check_cased(&word)
    }

    fn check_cased(&self, w: &str) -> bool {
        if self.check_form(w, false) {
            return true;
        }
        let mut cs = w.chars();
        let first = cs.next().unwrap_or(' ');
        let rest: String = cs.collect();
        let has_lower = w.chars().any(char::is_lowercase);
        let lower = w.to_lowercase();
        if first.is_uppercase() && !rest.chars().any(char::is_uppercase) {
            // "Casa": also as "casa" (not for words kept in their case).
            return self.check_form(&lower, true);
        }
        if !has_lower && w.chars().any(char::is_uppercase) {
            // "CASA": as "Casa" or "casa".
            let cap: String = first
                .to_uppercase()
                .chain(rest.to_lowercase().chars())
                .collect();
            return self.check_form(&cap, true) || self.check_form(&lower, true);
        }
        false
    }

    /// `w` as a root, an affixed root or a compound. `recased`: `w` is not
    /// the case typed, so words kept in their case do not count.
    fn check_form(&self, w: &str, recased: bool) -> bool {
        let entries = self.entries(w);
        if entries.iter().any(|f| Self::has(f, self.forbidden)) {
            return false;
        }
        if entries.iter().any(|f| {
            !Self::has(f, self.need_affix)
                && !Self::has(f, self.only_in_compound)
                && !(recased && Self::has(f, self.keep_case))
        }) {
            return true;
        }
        let chars: Vec<char> = w.chars().collect();
        if self.affixed(&chars) {
            return true;
        }
        self.compounds() && self.compound(&chars, 0)
    }

    /// The root `w` stands on when `sfx` is taken off its end.
    fn without_suffix(w: &[char], sfx: &Affix) -> Option<Vec<char>> {
        if w.len() < sfx.add.len() || !w.ends_with(&sfx.add) {
            return None;
        }
        let mut root: Vec<char> = w[..w.len() - sfx.add.len()].to_vec();
        root.extend(&sfx.strip);
        if root.is_empty() || root.len() < sfx.cond.len() {
            return None;
        }
        let tail = &root[root.len() - sfx.cond.len()..];
        tail.iter()
            .zip(&sfx.cond)
            .all(|(c, k)| k.matches(*c))
            .then_some(root)
    }

    fn without_prefix(w: &[char], pfx: &Affix) -> Option<Vec<char>> {
        if w.len() < pfx.add.len() || !w.starts_with(&pfx.add) {
            return None;
        }
        let mut root = pfx.strip.clone();
        root.extend(&w[pfx.add.len()..]);
        if root.is_empty() || root.len() < pfx.cond.len() {
            return None;
        }
        root.iter()
            .zip(&pfx.cond)
            .all(|(c, k)| k.matches(*c))
            .then_some(root)
    }

    fn suffixes_for<'a>(&'a self, w: &[char]) -> impl Iterator<Item = &'a Affix> + 'a {
        let last = w.last().copied();
        self.suffix_index
            .get(&last)
            .into_iter()
            .chain(self.suffix_index.get(&None))
            .flatten()
            .map(|i| &self.suffixes[*i])
    }

    fn prefixes_for<'a>(&'a self, w: &[char]) -> impl Iterator<Item = &'a Affix> + 'a {
        let first = w.first().copied();
        self.prefix_index
            .get(&first)
            .into_iter()
            .chain(self.prefix_index.get(&None))
            .flatten()
            .map(|i| &self.prefixes[*i])
    }

    /// A root entry with every flag in `need`, not forbidden.
    fn root_with(&self, root: &[char], need: &[Flag]) -> bool {
        let key: String = root.iter().collect();
        self.entries(&key)
            .iter()
            .any(|f| !Self::has(f, self.forbidden) && need.iter().all(|n| f.contains(n)))
    }

    /// An affix whose continuation classes hold `NEEDAFFIX` cannot end
    /// a word by itself.
    fn needs_more(&self, a: &Affix) -> bool {
        Self::has(&a.cont, self.need_affix)
    }

    fn affixed(&self, w: &[char]) -> bool {
        for sfx in self.suffixes_for(w) {
            let Some(root) = Self::without_suffix(w, sfx) else {
                continue;
            };
            if !self.needs_more(sfx) && self.root_with(&root, &[sfx.flag]) {
                return true;
            }
            // Two suffixes: the inner one allows the outer one.
            for inner in self.suffixes_for(&root) {
                if !inner.cont.contains(&sfx.flag) {
                    continue;
                }
                if let Some(r2) = Self::without_suffix(&root, inner) {
                    if self.root_with(&r2, &[inner.flag]) {
                        return true;
                    }
                }
            }
        }
        for pfx in self.prefixes_for(w) {
            let Some(root) = Self::without_prefix(w, pfx) else {
                continue;
            };
            if !self.needs_more(pfx) && self.root_with(&root, &[pfx.flag]) {
                return true;
            }
            if !pfx.cross {
                continue;
            }
            // A prefix and a suffix together.
            for sfx in self.suffixes_for(&root) {
                if !sfx.cross {
                    continue;
                }
                if let Some(r2) = Self::without_suffix(&root, sfx) {
                    if self.root_with(&r2, &[pfx.flag, sfx.flag])
                        || (sfx.cont.contains(&pfx.flag) && self.root_with(&r2, &[sfx.flag]))
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn compounds(&self) -> bool {
        self.compound_flag.is_some() || self.compound_begin.is_some()
    }

    /// A compound part at its place: a root, or a root with an affix,
    /// whose flags (with those the affix gives) hold the compound flag or
    /// the flag of that place. Suffixes may end the last part and
    /// prefixes start the first; elsewhere only affixes that carry
    /// `COMPOUNDPERMITFLAG` or add nothing.
    fn part(&self, part: &[char], place: usize, last: bool) -> bool {
        let flag_ok = |f: &[Flag], extra: &[Flag]| {
            if Self::has(f, self.forbidden) {
                return false;
            }
            let has = |x: Option<Flag>| Self::has(f, x) || Self::has(extra, x);
            has(self.compound_flag)
                || match (place, last) {
                    (0, _) => has(self.compound_begin),
                    (_, true) => has(self.compound_last),
                    _ => has(self.compound_middle),
                }
        };
        let permitted = |a: &Affix| a.add.is_empty() || Self::has(&a.cont, self.compound_permit);
        let forms = |p: &[char]| -> Vec<String> {
            let s: String = p.iter().collect();
            let mut v = vec![s.clone()];
            if place > 0 {
                // Nouns written with a capital alone are small inside.
                let mut cs = s.chars();
                if let Some(f) = cs.next() {
                    v.push(f.to_uppercase().chain(cs).collect());
                }
            }
            v
        };
        for form in forms(part) {
            if self.entries(&form).iter().any(|f| flag_ok(f, &[])) {
                return true;
            }
            let cs: Vec<char> = form.chars().collect();
            for sfx in self.suffixes_for(&cs) {
                if !(last || permitted(sfx)) {
                    continue;
                }
                if let Some(root) = Self::without_suffix(&cs, sfx) {
                    let key: String = root.iter().collect();
                    if self
                        .entries(&key)
                        .iter()
                        .any(|f| f.contains(&sfx.flag) && flag_ok(f, &sfx.cont))
                    {
                        return true;
                    }
                }
            }
            for pfx in self.prefixes_for(&cs) {
                if !(place == 0 || permitted(pfx)) {
                    continue;
                }
                if let Some(root) = Self::without_prefix(&cs, pfx) {
                    let key: String = root.iter().collect();
                    if self
                        .entries(&key)
                        .iter()
                        .any(|f| f.contains(&pfx.flag) && flag_ok(f, &pfx.cont))
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn compound(&self, w: &[char], place: usize) -> bool {
        let min = self.compound_min;
        if w.len() < min * 2 && place == 0 {
            return false;
        }
        if self.compound_word_max > 0 && place + 2 > self.compound_word_max {
            return false;
        }
        for i in min..=w.len().saturating_sub(min) {
            if !self.part(&w[..i], place, false) {
                continue;
            }
            let rest = &w[i..];
            if self.part(rest, place + 1, true) || self.compound(rest, place + 1) {
                return true;
            }
        }
        false
    }

    /// Spellings near `word` that the dictionary knows, best first: the
    /// replacements of the affix file, then single edits with its `TRY`
    /// letters, then two words.
    pub fn suggest(&self, word: &str, max: usize) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let push = |s: String, out: &mut Vec<String>| {
            if s != word
                && !s.starts_with('-')
                && !s.ends_with('-')
                && !out.contains(&s)
                && out.len() < max
            {
                let known = if s.contains(' ') {
                    s.split(' ').all(|p| !p.is_empty() && self.check(p))
                } else {
                    self.check(&s)
                };
                if known {
                    out.push(s);
                }
            }
        };
        for (from, to) in &self.rep {
            let mut start = 0;
            while let Some(at) = word[start..].find(from.as_str()) {
                let at = start + at;
                let s = format!("{}{}{}", &word[..at], to, &word[at + from.len()..]);
                push(s, &mut out);
                start = at + from.len().max(1);
            }
        }
        let chars: Vec<char> = word.chars().collect();
        let mut letters = self.try_chars.clone();
        if letters.is_empty() {
            letters = "esianrtolcdugmphbyfvkwzxjq".chars().collect();
        }
        for c in &chars {
            if !letters.contains(c) && c.is_alphabetic() {
                letters.push(*c);
            }
        }
        for i in 0..chars.len() {
            if i + 1 < chars.len() {
                let mut t = chars.clone();
                t.swap(i, i + 1);
                push(t.iter().collect(), &mut out);
            }
            let mut d = chars.clone();
            d.remove(i);
            push(d.iter().collect(), &mut out);
            for a in &letters {
                let mut t = chars.clone();
                if t[i] != *a {
                    t[i] = *a;
                    push(t.iter().collect(), &mut out);
                }
            }
        }
        for i in 0..=chars.len() {
            for a in &letters {
                let mut t = chars.clone();
                t.insert(i, *a);
                push(t.iter().collect(), &mut out);
            }
        }
        // Two words, each of two letters or more.
        for i in 2..chars.len().saturating_sub(1) {
            let a: String = chars[..i].iter().collect();
            let b: String = chars[i..].iter().collect();
            push(format!("{a} {b}"), &mut out);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small affix file in the format's own terms: plurals and a past
    /// tense by suffix (one with a condition and a strip), a prefix that
    /// combines with them, a second suffix on top of the first, a word
    /// that needs an affix, a forbidden form and compounding.
    const AFF: &str = "SET UTF-8
TRY esiarntolcdugmphbyfvkwz
REP 1
REP f ph
NEEDAFFIX N
FORBIDDENWORD X
COMPOUNDFLAG C
COMPOUNDMIN 3
KEEPCASE K

SFX S Y 3
SFX S 0 s [^sxy]
SFX S y ies [^aeiou]y
SFX S 0 es [sx]

SFX D Y 2
SFX D 0 ed [^e]
SFX D 0 d e

PFX U Y 1
PFX U 0 un .

SFX L Y 1
SFX L 0 ly/M .

SFX M Y 1
SFX M 0 ness .
";

    const DIC: &str = "10
cat/S
fly/S
box/S
walk/DSU
bake/D
lock/DUS
kind/L
stem/N
gooses/X
haus/C
tür/C
iPhone/K
";

    fn dict() -> Hunspell {
        Hunspell::from_bytes(AFF.as_bytes(), DIC.as_bytes())
    }

    #[test]
    fn roots_and_affixes() {
        let h = dict();
        for w in [
            "cat",
            "cats",
            "flies",
            "boxes",
            "walked",
            "baked",
            "unlocked",
            "unlocks",
            "kindly",
            "kindlyness",
        ] {
            assert!(h.check(w), "{w}");
        }
        for w in ["flys", "boxs", "bakeed", "unbaked", "catly", "kindness"] {
            assert!(!h.check(w), "{w}");
        }
    }

    #[test]
    fn flags_that_restrict() {
        let h = dict();
        // Needs an affix: the root alone is not a word.
        assert!(!h.check("stem"));
        // A forbidden form, and the compound and case rules.
        assert!(!h.check("gooses"));
        assert!(h.check("haustür"));
        assert!(h.check("haustürhaus"));
        assert!(!h.check("hau"));
        assert!(h.check("iPhone"));
        assert!(!h.check("Iphone"));
    }

    #[test]
    fn capitals_and_apostrophes() {
        let h = dict();
        assert!(h.check("Cats"));
        assert!(h.check("CATS"));
        assert!(!h.check("cAts"));
    }

    #[test]
    fn suggestions() {
        let h = dict();
        let s = h.suggest("cta", 8);
        assert!(s.contains(&"cat".to_string()), "{s:?}");
        let s = h.suggest("flyes", 8);
        assert!(s.contains(&"flies".to_string()), "{s:?}");
        let s = h.suggest("catbox", 8);
        assert!(s.contains(&"cat box".to_string()), "{s:?}");
    }

    #[test]
    fn long_and_numeric_flags_and_aliases() {
        let aff = "FLAG long\nSFX Aa Y 1\nSFX Aa 0 s .\n";
        let h = Hunspell::from_bytes(aff.as_bytes(), b"1\ndog/Aa\n");
        assert!(h.check("dogs"));
        let aff = "FLAG num\nSFX 101 Y 1\nSFX 101 0 s .\n";
        let h = Hunspell::from_bytes(aff.as_bytes(), b"1\ndog/7,101\n");
        assert!(h.check("dogs"));
        let aff = "AF 1\nAF S\nSFX S Y 1\nSFX S 0 s .\n";
        let h = Hunspell::from_bytes(aff.as_bytes(), b"1\ndog/1\n");
        assert!(h.check("dogs"));
    }

    #[test]
    fn eight_bit_files() {
        // "maçã" in ISO 8859-1, with a suffix.
        let aff = b"SET ISO8859-1\nSFX S Y 1\nSFX S 0 s .\n";
        let dic = b"1\nma\xe7\xe3/S\n";
        let h = Hunspell::from_bytes(aff, dic);
        assert!(h.check("maçãs"));
        assert!(!h.check("maça"));
    }

    /// Real dictionaries, where `HUNSPELL_TEST_DIR` holds them (not in
    /// the repository): `en_US`, `pt_BR` and `de_DE_frami` `.aff`/`.dic`.
    #[test]
    #[ignore]
    fn real_dictionaries() {
        let Some(dir) = std::env::var_os("HUNSPELL_TEST_DIR") else {
            return;
        };
        let dir = std::path::PathBuf::from(dir);
        let load = |name: &str| {
            let t = std::time::Instant::now();
            let aff = std::fs::read(dir.join(format!("{name}.aff"))).expect("aff");
            let dic = std::fs::read(dir.join(format!("{name}.dic"))).expect("dic");
            let h = Hunspell::from_bytes(&aff, &dic);
            eprintln!("{name}: {} entries in {:?}", h.len(), t.elapsed());
            h
        };
        let cases: [(&str, &[&str], &[&str]); 3] = [
            (
                "en_US",
                &[
                    "the",
                    "houses",
                    "walked",
                    "running",
                    "happiness",
                    "unhappily",
                    "children's",
                    "Paris",
                    "NASA",
                    "doesn't",
                    "doesn’t",
                    "reorganization",
                    "quickly",
                ],
                &["hte", "recieve", "paris", "teh", "walkked"],
            ),
            (
                "pt_BR",
                &[
                    "casa",
                    "casas",
                    "ação",
                    "ações",
                    "coração",
                    "corações",
                    "falávamos",
                    "desenhávamos",
                    "inconstitucionalmente",
                    "Brasil",
                    "computadores",
                ],
                &["kasa", "acão", "coracao", "computadorez"],
            ),
            (
                "de_DE_frami",
                &[
                    "Haus",
                    "Häuser",
                    "Kindergarten",
                    "Gartenhaus",
                    "gehen",
                    "ging",
                    "Straße",
                    "schönen",
                ],
                &["Hauss", "gehn", "Strase"],
            ),
        ];
        for (name, good, bad) in cases {
            let h = load(name);
            let t = std::time::Instant::now();
            for w in good {
                assert!(h.check(w), "{name}: {w} should be known");
            }
            for w in bad {
                assert!(!h.check(w), "{name}: {w} should be unknown");
            }
            eprintln!("{name}: checks in {:?}", t.elapsed());
            let t = std::time::Instant::now();
            let s = h.suggest(bad[0], 8);
            eprintln!("{name}: {} -> {s:?} in {:?}", bad[0], t.elapsed());
            assert!(!s.is_empty(), "{name}: no suggestion for {}", bad[0]);
        }
    }

    #[test]
    fn malformed_files_do_not_panic() {
        for aff in [
            "SFX",
            "SFX S Y x",
            "SFX S Y 3\nSFX S",
            "PFX P N 1\nPFX P 0 [ab",
            "AF\nFLAG long\n",
        ] {
            let h = Hunspell::from_bytes(aff.as_bytes(), b"x\n/\n\\/a/\nab/S\n");
            let _ = h.check("abs");
            let _ = h.suggest("abz", 4);
        }
    }
}
