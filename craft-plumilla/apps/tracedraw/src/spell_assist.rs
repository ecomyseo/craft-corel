//! Right-click on a word of the text being edited: a word the spell
//! checker does not know (its suggestions, Ignore All), the second of two
//! same words in a row (Delete Repeated Word, Ignore All) and a word
//! ignored before (Stop Ignoring). The menus also give Paste, the Spell
//! Checker and the Thesaurus (see `docs/behavior/text.md`, Spelling on
//! right-click).

use crate::app::App;
use tracedraw_core::geometry::Point;

/// What is wrong with a word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// The dictionaries do not know it.
    Misspelled,
    /// It repeats the word before it.
    Repeated,
    /// It is not known and was ignored this session.
    Ignored,
}

/// A word of the text being edited, with its character range and issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordAt {
    pub start: usize,
    pub end: usize,
    pub word: String,
    pub issue: Issue,
    /// For a repeated word: where the space before it starts, so deleting
    /// it leaves one word and one space.
    pub delete_from: usize,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\''
}

/// The word around character `idx` of `text`, as (start, end).
fn word_bounds(text: &[char], idx: usize) -> Option<(usize, usize)> {
    let idx = idx.min(text.len());
    let mut a = idx;
    while a > 0 && text.get(a - 1).is_some_and(|c| is_word_char(*c)) {
        a -= 1;
    }
    let mut b = idx;
    while text.get(b).is_some_and(|c| is_word_char(*c)) {
        b += 1;
    }
    (a < b).then_some((a, b))
}

/// The issue of the word around `idx` in `text`, if any; `known` decides
/// spelling and `ignored` the words ignored this session (lowercase).
pub fn issue_at(
    text: &str,
    idx: usize,
    known: impl Fn(&str) -> bool,
    ignored: &std::collections::HashSet<String>,
) -> Option<WordAt> {
    let chars: Vec<char> = text.chars().collect();
    let (start, end) = word_bounds(&chars, idx)?;
    let word: String = chars[start..end].iter().collect();
    let lower = word.to_lowercase();
    // The word before, across spaces only (not across a line or a stop).
    let mut s = start;
    while s > 0 && chars[s - 1] == ' ' {
        s -= 1;
    }
    if s < start && s > 0 {
        if let Some((pa, pb)) = word_bounds(&chars, s) {
            let prev: String = chars[pa..pb].iter().collect();
            if pb == s && prev.to_lowercase() == lower && !ignored.contains(&lower) {
                return Some(WordAt {
                    start,
                    end,
                    word,
                    issue: Issue::Repeated,
                    delete_from: s,
                });
            }
        }
    }
    if ignored.contains(&lower) {
        return Some(WordAt {
            start,
            end,
            word,
            issue: Issue::Ignored,
            delete_from: start,
        });
    }
    (!known(&word)).then_some(WordAt {
        start,
        end,
        word,
        issue: Issue::Misspelled,
        delete_from: start,
    })
}

impl App {
    /// The word under page point `p` of the text being edited, when the
    /// spelling menus have something to say about it.
    pub fn word_issue_at(&self, p: Point) -> Option<WordAt> {
        let idx = self.text_hit_char(p)?;
        issue_at(
            &self.edit_text(),
            idx,
            crate::spell::is_known,
            &self.spell_ignored,
        )
    }

    /// A suggestion in the word's place (one undo step, as typing).
    pub fn replace_word_with(&mut self, w: &WordAt, replacement: &str) {
        if let Some(te) = self.text_edit.as_mut() {
            te.anchor = w.start;
            te.caret = w.end;
        }
        self.text_insert(replacement);
    }

    /// Delete Repeated Word: the second word and the space before it go.
    pub fn delete_repeated_word(&mut self, w: &WordAt) {
        if let Some(te) = self.text_edit.as_mut() {
            te.anchor = w.delete_from;
            te.caret = w.end;
        }
        self.text_delete_forward();
    }

    /// Ignore All: the word is not marked again this session.
    pub fn ignore_word(&mut self, word: &str) {
        self.spell_ignored.insert(word.to_lowercase());
    }

    /// Stop Ignoring: the word is checked again.
    pub fn unignore_word(&mut self, word: &str) {
        self.spell_ignored.remove(&word.to_lowercase());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn known(w: &str) -> bool {
        ["the", "cat", "sat"].contains(&w.to_lowercase().as_str())
    }

    #[test]
    fn a_word_is_misspelled_repeated_ignored_or_fine() {
        let none = HashSet::new();
        let text = "the the cat szat";
        // "the" a second time, right after the first.
        let w = issue_at(text, 5, known, &none).expect("repeated");
        assert_eq!((w.issue, w.start, w.end, w.delete_from), (Issue::Repeated, 4, 7, 3));
        // The first "the" is fine.
        assert_eq!(issue_at(text, 1, known, &none), None);
        let w = issue_at(text, 14, known, &none).expect("misspelled");
        assert_eq!((w.issue, w.word.as_str()), (Issue::Misspelled, "szat"));
        let ignored: HashSet<String> = ["szat".to_string()].into();
        let w = issue_at(text, 14, known, &ignored).expect("ignored");
        assert_eq!(w.issue, Issue::Ignored);
        // Not on a word.
        assert_eq!(issue_at("cat  sat", 4, known, &none), None);
        // Repeated across a line break is not a repetition.
        assert_eq!(issue_at("the\nthe", 5, known, &none), None);
    }
}
