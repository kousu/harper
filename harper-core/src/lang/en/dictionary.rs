//! English's curated dictionary: the word list and annotations next to this module, loaded once.

use std::sync::{Arc, LazyLock};

use crate::spell::{FstDictionary, MutableDictionary, TrieDictionary};

pub(crate) const DICTIONARY: &str = include_str!("dictionary.dict");
pub(crate) const ANNOTATIONS: &str = include_str!("annotations.json");

/// English's curated dictionary, as a [`MutableDictionary`].
pub(crate) fn mutable_dictionary() -> Arc<MutableDictionary> {
    static DICT: LazyLock<Arc<MutableDictionary>> = LazyLock::new(|| {
        MutableDictionary::from_rune_files(DICTIONARY, ANNOTATIONS)
            .map(Arc::new)
            .unwrap_or_else(|e| panic!("Failed to load curated dictionary: {}", e))
    });

    (*DICT).clone()
}

/// English's curated dictionary, as an [`FstDictionary`] for fast spell checking.
pub(crate) fn dictionary() -> Arc<FstDictionary> {
    static DICT: LazyLock<Arc<FstDictionary>> =
        LazyLock::new(|| Arc::new((*mutable_dictionary()).clone().into()));

    (*DICT).clone()
}

/// English's curated dictionary, as a [`TrieDictionary`] for prefix and suffix searches.
pub(crate) fn trie_dictionary() -> Arc<TrieDictionary<Arc<FstDictionary>>> {
    static DICT: LazyLock<Arc<TrieDictionary<Arc<FstDictionary>>>> =
        LazyLock::new(|| Arc::new(TrieDictionary::new(dictionary())));

    (*DICT).clone()
}
