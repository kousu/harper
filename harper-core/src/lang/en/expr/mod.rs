//! Expressions that match English words.

mod duration_expr;
mod mergeable_words;
mod pronoun_be;
mod reflexive_pronoun;
mod spelled_number_expr;
mod time_unit_expr;

pub use duration_expr::DurationExpr;
pub use mergeable_words::MergeableWords;
pub use pronoun_be::PronounBe;
pub use reflexive_pronoun::ReflexivePronoun;
pub use spelled_number_expr::SpelledNumberExpr;
pub use time_unit_expr::TimeUnitExpr;

use crate::LanguageFamily;
use crate::expr::{Expr, FixedPhrase, SequenceExpr, SimilarToPhrase};

/// Matches `phrase`, split into words as English splits them: [`FixedPhrase::from_phrase_for`]
/// in English.
pub fn fixed_phrase(phrase: &str) -> FixedPhrase {
    FixedPhrase::from_phrase_for(phrase, LanguageFamily::English)
}

/// Matches phrases within `max_edit_dist` of `phrase`, split into words as English splits them:
/// [`SimilarToPhrase::from_phrase_for`] in English.
pub fn similar_to_phrase(phrase: &str, max_edit_dist: u8) -> SimilarToPhrase {
    SimilarToPhrase::from_phrase_for(phrase, max_edit_dist, LanguageFamily::English)
}
