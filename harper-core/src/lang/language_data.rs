//! What each language provides to the shared code.

/// What shared code needs from one language.
///
/// Each language implements it on a marker type in its own module (`lang::en::English`), and
/// [`LanguageFamily::data`](crate::LanguageFamily::data) picks one. Shared code calls these methods
/// instead of matching on the language, so adding a language doesn't touch it.
///
/// Anything that needs the language's dialect, like building its rules, stays a `match` on
/// [`Language`](crate::Language), since each language's dialects are a different type.
pub(crate) trait LanguageData: Sync {
    /// How much more likely `suggestion` is as a fix for `misspelled` to writers of this language,
    /// beyond edit distance (lower is better).
    fn score_adjustment(&self, misspelled: &[char], suggestion: &[char], edit_distance: u8) -> i32;
}
