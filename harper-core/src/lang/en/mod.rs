//! English: its dialects, word lists, expressions, patterns and rules. Its data files sit next
//! to this module.

pub(crate) mod dialect;
pub(crate) mod dictionary;
pub(crate) mod expr;
pub(crate) mod indefinite_article;
pub(crate) mod irregular_nouns;
pub(crate) mod irregular_verbs;
pub(crate) mod linting;
pub(crate) mod patterns;
pub(crate) mod regular_nouns;
pub(crate) mod thesaurus_helper;
pub(crate) mod title_case;

use crate::lang::LanguageData;

/// English's data, for [`LanguageFamily::data`](crate::LanguageFamily::data).
pub(crate) struct English;

impl LanguageData for English {
    fn score_adjustment(&self, misspelled: &[char], suggestion: &[char], edit_distance: u8) -> i32 {
        linting::spell_check::english_score_adjustment(misspelled, suggestion, edit_distance)
    }
}
