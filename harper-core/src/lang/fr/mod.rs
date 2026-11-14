//! French: its dialects.

pub(crate) mod dialect;

use std::sync::Arc;

use harper_brill::UPOS;

use crate::lang::LanguageData;
use crate::lexers::{Lexer, PlainText};
use crate::linting::{FlatConfig, LintGroup, StructuredConfig};
use crate::spell::{FstDictionary, MutableDictionary, TrieDictionary};
use crate::{Language, Lrc};

/// French's data, for [`LanguageFamily::data`](crate::LanguageFamily::data). French is
/// experimental: it has no dictionary, lexer or tagger of its own yet.
pub(crate) struct French;

impl LanguageData for French {
    /// French has no dictionary yet, so it gets an empty one.
    fn dictionary(&self) -> Arc<FstDictionary> {
        Arc::new(FstDictionary::new(Vec::new()))
    }

    fn mutable_dictionary(&self) -> Arc<MutableDictionary> {
        Arc::new(MutableDictionary::new())
    }

    fn trie_dictionary(&self) -> Arc<TrieDictionary<Arc<FstDictionary>>> {
        Arc::new(TrieDictionary::new(self.dictionary()))
    }

    /// French uses [`PlainText`] until it has its own lexer.
    fn lexer(&self) -> Lrc<dyn Lexer> {
        Lrc::new(PlainText)
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
