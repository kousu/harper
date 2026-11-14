//! French: its dialects, dictionary, lexer and rules. Its data files sit next to this module.

pub(crate) mod dialect;
pub(crate) mod dictionary;
pub(crate) mod lexing;
pub(crate) mod ordinal;

use std::sync::Arc;

use harper_brill::UPOS;

use crate::lang::LanguageData;
use crate::lexers::{Lexer, PlainFrench};
use crate::linting::{FlatConfig, LintGroup, StructuredConfig};
use crate::spell::{FstDictionary, MutableDictionary, TrieDictionary};
use crate::{Language, Lrc};

/// French's data, for [`LanguageFamily::data`](crate::LanguageFamily::data). French is
/// experimental: it has no tagger of its own yet.
pub(crate) struct French;

impl LanguageData for French {
    fn dictionary(&self) -> Arc<FstDictionary> {
        dictionary::dictionary()
    }

    fn mutable_dictionary(&self) -> Arc<MutableDictionary> {
        dictionary::mutable_dictionary()
    }

    fn trie_dictionary(&self) -> Arc<TrieDictionary<Arc<FstDictionary>>> {
        dictionary::trie_dictionary()
    }

    fn lexer(&self) -> Lrc<dyn Lexer> {
        Lrc::new(PlainFrench)
    }

    /// French has no tagger or chunker yet.
    fn tag_and_chunk(&self, sentence: &[String]) -> (Vec<Option<UPOS>>, Vec<bool>) {
        (vec![None; sentence.len()], vec![false; sentence.len()])
    }

    /// French has no labeled default config yet: all of its rules are on.
    fn default_config(&self) -> StructuredConfig {
        StructuredConfig {
            settings: Vec::new(),
        }
    }

    fn default_flat_config(&self) -> FlatConfig {
        // The dictionary and dialect don't matter: we're only after the config.
        LintGroup::new_curated_for_language(
            MutableDictionary::new().into(),
            Language::French(Default::default()),
        )
        .config
    }

    /// French has no spelling preferences yet.
    fn score_adjustment(&self, _: &[char], _: &[char], _: u8) -> i32 {
        0
    }
}
