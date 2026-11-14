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

use std::sync::{Arc, OnceLock};

use crate::EnglishDialect;
use crate::lang::LanguageData;
use crate::linting::{FlatConfig, StructuredConfig};
use crate::spell::{FstDictionary, MutableDictionary, TrieDictionary};

/// English's data, for [`LanguageFamily::data`](crate::LanguageFamily::data).
pub(crate) struct English;

impl LanguageData for English {
    fn dictionary(&self) -> Arc<FstDictionary> {
        dictionary::dictionary()
    }

    fn mutable_dictionary(&self) -> Arc<MutableDictionary> {
        dictionary::mutable_dictionary()
    }

    fn trie_dictionary(&self) -> Arc<TrieDictionary<Arc<FstDictionary>>> {
        dictionary::trie_dictionary()
    }

    fn default_config(&self) -> StructuredConfig {
        linting::default_config()
    }

    fn default_flat_config(&self) -> FlatConfig {
        static CURATED: OnceLock<FlatConfig> = OnceLock::new();

        CURATED
            .get_or_init(|| {
                // The dictionary and dialect do not matter, we're just after the config.
                linting::new_curated(MutableDictionary::new().into(), EnglishDialect::American)
                    .config
            })
            .clone()
    }

    fn score_adjustment(&self, misspelled: &[char], suggestion: &[char], edit_distance: u8) -> i32 {
        linting::spell_check::english_score_adjustment(misspelled, suggestion, edit_distance)
    }
}
