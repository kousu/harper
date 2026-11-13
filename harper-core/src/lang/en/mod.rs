//! English: its dialects, expressions, patterns and rules.

pub(crate) mod dialect;
pub(crate) mod expr;
pub(crate) mod linting;
pub(crate) mod patterns;

use crate::lang::LanguageData;

/// English's data, for [`LanguageFamily::data`](crate::LanguageFamily::data).
pub(crate) struct English;

impl LanguageData for English {
    fn score_adjustment(&self, misspelled: &[char], suggestion: &[char], edit_distance: u8) -> i32 {
        linting::spell_check::english_score_adjustment(misspelled, suggestion, edit_distance)
    }
}
