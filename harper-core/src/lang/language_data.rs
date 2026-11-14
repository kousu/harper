//! What each language provides to the shared code.

use std::sync::Arc;

use harper_brill::UPOS;

use crate::linting::{FlatConfig, StructuredConfig};
use crate::spell::{FstDictionary, MutableDictionary, TrieDictionary};

/// What shared code needs from one language.
///
/// Each language implements it on a marker type in its own module (`lang::en::English`), and
/// [`LanguageFamily::data`](crate::LanguageFamily::data) picks one. Shared code calls these methods
/// instead of matching on the language, so adding a language doesn't touch it.
///
/// Anything that needs the language's dialect, like building its rules, stays a `match` on
/// [`Language`](crate::Language), since each language's dialects are a different type.
pub(crate) trait LanguageData: Sync {
    /// The curated dictionary.
    fn dictionary(&self) -> Arc<FstDictionary>;

    /// The curated dictionary, as a [`MutableDictionary`].
    fn mutable_dictionary(&self) -> Arc<MutableDictionary>;

    /// The curated dictionary, as a [`TrieDictionary`].
    fn trie_dictionary(&self) -> Arc<TrieDictionary<Arc<FstDictionary>>>;

    /// Tag each word of a sentence with its part of speech, and mark which words are in a noun
    /// phrase.
    fn tag_and_chunk(&self, sentence: &[String]) -> (Vec<Option<UPOS>>, Vec<bool>);

    /// The structured config of the curated rules, including labels.
    fn default_config(&self) -> StructuredConfig;

    /// The flat config of the curated rules.
    fn default_flat_config(&self) -> FlatConfig;

    /// How much more likely `suggestion` is as a fix for `misspelled` to writers of this language,
    /// beyond edit distance (lower is better).
    fn score_adjustment(&self, misspelled: &[char], suggestion: &[char], edit_distance: u8) -> i32;
}
