//! French's curated dictionary: the word list and annotations next to this module, which
//! `build.sh` generates from the Grammalecte lexicon.

use std::sync::{Arc, LazyLock};

use crate::spell::{FstDictionary, MutableDictionary, TrieDictionary};

pub(crate) const DICTIONARY: &str = include_str!("dictionary.dict");
pub(crate) const ANNOTATIONS: &str = include_str!("annotations.json");

fn load() -> MutableDictionary {
    MutableDictionary::from_rune_files(DICTIONARY, ANNOTATIONS)
        .unwrap_or_else(|e| panic!("Failed to load French dictionary: {e}"))
}

/// French's curated dictionary, as a [`MutableDictionary`].
pub(crate) fn mutable_dictionary() -> Arc<MutableDictionary> {
    static DICT: LazyLock<Arc<MutableDictionary>> = LazyLock::new(|| Arc::new(load()));

    (*DICT).clone()
}

/// French's curated dictionary, as an [`FstDictionary`] for fast spell checking. It's built
/// separately from [`mutable_dictionary`], so the large word list isn't kept twice.
pub(crate) fn dictionary() -> Arc<FstDictionary> {
    static DICT: LazyLock<Arc<FstDictionary>> = LazyLock::new(|| Arc::new(load().into()));

    (*DICT).clone()
}

/// French's curated dictionary, as a [`TrieDictionary`] for prefix and suffix searches.
pub(crate) fn trie_dictionary() -> Arc<TrieDictionary<Arc<FstDictionary>>> {
    static DICT: LazyLock<Arc<TrieDictionary<Arc<FstDictionary>>>> =
        LazyLock::new(|| Arc::new(TrieDictionary::new(dictionary())));

    (*DICT).clone()
}
