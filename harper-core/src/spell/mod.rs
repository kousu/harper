//! Contains the relevant code for performing dictionary lookups and spellchecking (i.e. fuzzy
//! dictionary lookups).

use crate::{CharString, CharStringExt, DictWordMetadata, LanguageFamily};

pub use self::dictionary::Dictionary;
pub use self::fst_dictionary::FstDictionary;
pub use self::merged_dictionary::MergedDictionary;
pub use self::mutable_dictionary::MutableDictionary;
pub use self::trie_dictionary::TrieDictionary;
pub use self::word_id::{CanonicalWordId, CaseFoldedWordId};

/// Whether `b` is `a` with an "h" added after a "t".
#[cfg(feature = "en")]
#[deprecated(note = "English only: use `harper_core::lang::en::is_th_h_missing`")]
pub fn is_th_h_missing(a: &[char], b: &[char]) -> bool {
    crate::lang::en::is_th_h_missing(a, b)
}

mod dictionary;
mod fst_dictionary;
mod merged_dictionary;
mod mutable_dictionary;
mod rune;
mod trie_dictionary;
mod word_id;
mod word_map;

#[derive(PartialEq, Debug, Hash, Eq)]
pub struct FuzzyMatchResult<'a> {
    pub word: &'a [char],
    pub edit_distance: u8,
    pub metadata: std::borrow::Cow<'a, DictWordMetadata>,
}

impl PartialOrd for FuzzyMatchResult<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.edit_distance.partial_cmp(&other.edit_distance)
    }
}

/// Scores a possible spelling suggestion based on possible relevance to the user.
///
/// Lower = better. `adjust` adds the language's own preferences; see
/// [`suggest_correct_spelling_with`].
fn score_suggestion(
    misspelled_word: &[char],
    sug: &FuzzyMatchResult,
    adjust: &impl Fn(&[char], &[char], u8) -> i32,
) -> i32 {
    if misspelled_word.is_empty() || sug.word.is_empty() {
        return i32::MAX;
    }

    let mut score = sug.edit_distance as i32 * 10;

    // People are much less likely to mistype the first letter.
    if misspelled_word
        .first()
        .unwrap()
        .eq_ignore_ascii_case(sug.word.first().unwrap())
    {
        score -= 10;
    }

    // If the original word is plural, the correct one probably is too.
    if *misspelled_word.last().unwrap() == 's' && *sug.word.last().unwrap() == 's' {
        score -= 5;
    }

    // Promote suggestions that differ only by an apostrophe
    let check_apostrophe_diff = |longer: &[char], shorter: &[char]| -> bool {
        if let Some(pos) = longer.iter().position(|&c| c == '\'' || c == '’') {
            longer.len() - 1 == shorter.len()
                && longer.starts_with(&shorter[..pos])
                && longer.ends_with(&shorter[pos..])
        } else {
            false
        }
    };

    match (
        misspelled_word.len() as i32 - sug.word.len() as i32,
        sug.metadata.is_apostrophized(),
    ) {
        (1, _)
            if (misspelled_word.contains(&'\'') || misspelled_word.contains(&'\u{2019}'))
                && check_apostrophe_diff(misspelled_word, sug.word) =>
        {
            score -= 8
        }
        (-1, true) if check_apostrophe_diff(sug.word, misspelled_word) => score -= 8,
        _ => {} // not a single-character apostrophe difference
    }

    // Boost common words.
    if sug.metadata.common && sug.metadata.derived_from.is_none() {
        score -= 4;
    }

    // For turning words into contractions.
    if sug.word.iter().filter(|c| **c == '\'').count() == 1 {
        score -= 5;
    }

    if !misspelled_word.contains_vowel() && !sug.word.contains_vowel() {
        score += 10;
    }

    score + adjust(misspelled_word, sug.word, sug.edit_distance)
}

/// Order the suggestions to be shown to the user.
fn order_suggestions<'b>(
    misspelled_word: &[char],
    mut matches: Vec<FuzzyMatchResult<'b>>,
    adjust: &impl Fn(&[char], &[char], u8) -> i32,
) -> Vec<&'b [char]> {
    matches.sort_by_cached_key(|v| score_suggestion(misspelled_word, v, adjust));

    matches.into_iter().map(|v| v.word).collect()
}

/// Get the closest matches in the provided [`Dictionary`] and rank them, preferring the
/// suggestions English writers most likely meant.
/// Implementation is left up to the underlying dictionary.
#[deprecated(note = "use `suggest_correct_spelling_for` with the text's language")]
#[cfg(feature = "en")]
pub fn suggest_correct_spelling<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
) -> Vec<&'a [char]> {
    suggest_correct_spelling_for(
        misspelled_word,
        result_limit,
        max_edit_dist,
        dictionary,
        LanguageFamily::English,
    )
}

/// Get the closest matches in the provided [`Dictionary`] and rank them, preferring the
/// suggestions writers of `language` most likely meant.
pub fn suggest_correct_spelling_for<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
    language: LanguageFamily,
) -> Vec<&'a [char]> {
    let data = language.data();
    suggest_correct_spelling_with(
        misspelled_word,
        result_limit,
        max_edit_dist,
        dictionary,
        |misspelled, suggestion, distance| data.score_adjustment(misspelled, suggestion, distance),
    )
}

/// Like [`suggest_correct_spelling`], with `adjust` adding a language's own preferences to each
/// suggestion's score (lower is better). It is given the misspelled word, the suggestion and their
/// edit distance.
pub fn suggest_correct_spelling_with<'a>(
    misspelled_word: &[char],
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &'a impl Dictionary,
    adjust: impl Fn(&[char], &[char], u8) -> i32,
) -> Vec<&'a [char]> {
    let matches: Vec<FuzzyMatchResult> = dictionary
        .fuzzy_match(misspelled_word, max_edit_dist, result_limit)
        .into_iter()
        .collect();

    order_suggestions(misspelled_word, matches, &adjust)
}

/// Convenience function over [`suggest_correct_spelling`] that does conversions
/// for you.
#[deprecated(note = "use `suggest_correct_spelling_str_for` with the text's language")]
#[cfg(feature = "en")]
pub fn suggest_correct_spelling_str(
    misspelled_word: impl Into<String>,
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &impl Dictionary,
) -> Vec<String> {
    suggest_correct_spelling_str_for(
        misspelled_word,
        result_limit,
        max_edit_dist,
        dictionary,
        LanguageFamily::English,
    )
}

/// Convenience function over [`suggest_correct_spelling_for`] that does conversions
/// for you.
pub fn suggest_correct_spelling_str_for(
    misspelled_word: impl Into<String>,
    result_limit: usize,
    max_edit_dist: u8,
    dictionary: &impl Dictionary,
    language: LanguageFamily,
) -> Vec<String> {
    let chars: CharString = misspelled_word.into().chars().collect();
    suggest_correct_spelling_for(&chars, result_limit, max_edit_dist, dictionary, language)
        .into_iter()
        .map(|a| a.to_string())
        .collect()
}

#[cfg(all(test, feature = "en"))]
mod tests {
    use itertools::Itertools;

    use crate::CharStringExt;

    use super::suggest_correct_spelling_str_for;
    use crate::LanguageFamily;

    const RESULT_LIMIT: usize = 200;
    const MAX_EDIT_DIST: u8 = 2;

    #[test]
    fn normalizes_weve() {
        let word = ['w', 'e', '’', 'v', 'e'];
        let norm = word.normalized();

        assert_eq!(norm.clone(), vec!['w', 'e', '\'', 'v', 'e'])
    }

    #[test]
    fn punctation_no_duplicates() {
        let results = suggest_correct_spelling_str_for(
            "punctation",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &crate::lang::en::dictionary::dictionary(),
            LanguageFamily::English,
        );

        assert!(results.iter().all_unique())
    }

    #[test]
    fn youre_contraction() {
        assert_suggests_correction("youre", "you're");
    }

    #[test]
    fn thats_contraction() {
        assert_suggests_correction("thats", "that's");
    }

    #[test]
    fn weve_contraction() {
        assert_suggests_correction("weve", "we've");
    }

    #[test]
    fn this_correction() {
        assert_suggests_correction("ths", "this");
    }

    #[test]
    fn issue_624_no_duplicates() {
        let results = suggest_correct_spelling_str_for(
            "Semantical",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &crate::lang::en::dictionary::dictionary(),
            LanguageFamily::English,
        );

        assert!(results.iter().all_unique())
    }

    #[test]
    fn issue_182() {
        assert_suggests_correction("Im", "I'm");
    }

    #[test]
    fn fst_spellcheck_hvllo() {
        let results = suggest_correct_spelling_str_for(
            "hvllo",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &crate::lang::en::dictionary::dictionary(),
            LanguageFamily::English,
        );

        assert!(results.iter().take(3).contains(&"hello".to_string()));
    }

    /// Assert that the default suggestion settings result in a specific word
    /// being in the top three results for a given misspelling.
    #[track_caller]
    fn assert_suggests_correction(misspelled_word: &str, correct: &str) {
        let results = suggest_correct_spelling_str_for(
            misspelled_word,
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &crate::lang::en::dictionary::dictionary(),
            LanguageFamily::English,
        );

        dbg!(&results);

        assert!(results.iter().take(3).contains(&correct.to_string()));
    }

    #[test]
    fn spellcheck_hvllo() {
        assert_suggests_correction("hvllo", "hello");
    }

    #[test]
    fn spellcheck_aout() {
        assert_suggests_correction("aout", "about");
    }

    #[test]
    fn spellchecking_is_deterministic() {
        let results1 = suggest_correct_spelling_str_for(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &crate::lang::en::dictionary::dictionary(),
            LanguageFamily::English,
        );
        let results2 = suggest_correct_spelling_str_for(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &crate::lang::en::dictionary::dictionary(),
            LanguageFamily::English,
        );
        let results3 = suggest_correct_spelling_str_for(
            "hello",
            RESULT_LIMIT,
            MAX_EDIT_DIST,
            &crate::lang::en::dictionary::dictionary(),
            LanguageFamily::English,
        );

        assert_eq!(results1, results2);
        assert_eq!(results1, results3);
    }

    #[test]
    fn adviced_correction() {
        assert_suggests_correction("adviced", "advised");
    }

    #[test]
    fn aknowledged_correction() {
        assert_suggests_correction("aknowledged", "acknowledged");
    }

    #[test]
    fn alcaholic_correction() {
        assert_suggests_correction("alcaholic", "alcoholic");
    }

    #[test]
    fn slaves_correction() {
        assert_suggests_correction("Slaves", "Slavs");
    }

    #[test]
    fn conciousness_correction() {
        assert_suggests_correction("conciousness", "consciousness");
    }

    #[test]
    fn v_apostrophe_s_suggests_vs() {
        assert_suggests_correction("v's", "vs");
    }

    #[test]
    fn v_apostrophe_typographical_s_suggests_vs() {
        assert_suggests_correction("v’s", "vs");
    }

    #[test]
    fn missing_apostrophe_childrens_suggests_childrens() {
        assert_suggests_correction("childrens", "children's");
    }
}
