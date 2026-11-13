//! English spelling conventions for [`SpellCheck`](crate::linting::spell_check::SpellCheck).

use itertools::Itertools;

use super::informal_laughter::is_informal_laughter;
use super::proper_noun_capitalization_linters::proper_nouns_expr;
use crate::expr::{Filter, LongestMatchOf, SequenceExpr};
use crate::linting::Lint;
use crate::linting::spell_check::SpellingConventions;
use crate::{Document, remove_lints_overlapping_expr};

/// What spell checking English needs beyond the dictionary.
pub(crate) struct EnglishSpelling {
    proper_nouns: LongestMatchOf,
}

impl EnglishSpelling {
    pub(crate) fn new() -> Self {
        Self {
            proper_nouns: proper_nouns_expr(),
        }
    }
}

impl SpellingConventions for EnglishSpelling {
    fn is_ignored(&self, word: &[char]) -> bool {
        is_informal_laughter(word)
    }

    fn remove_exempt(&self, document: &Document, lints: &mut Vec<Lint>) {
        remove_lints_overlapping_expr(&parenthetical_plural_s_expr(), document, lints);
        // Names keep their official spelling in every dialect, such as `Pearl Harbor` in British
        // English. Their capitalization is handled by `ProperNounCapitalizationLinter`.
        remove_lints_overlapping_expr(&self.proper_nouns, document, lints);
    }

    fn score_adjustment(&self, misspelled: &[char], suggestion: &[char], edit_distance: u8) -> i32 {
        english_score_adjustment(misspelled, suggestion, edit_distance)
    }
}

/// How much English spelling habits make `suggestion` a likely fix for `misspelled` (lower is
/// better): dialect spellings like "color"/"colour", "ie"/"ei" and "er"/"re", and a missing "h"
/// after "t".
pub fn english_score_adjustment(
    misspelled: &[char],
    suggestion: &[char],
    edit_distance: u8,
) -> i32 {
    let mut score = 0;

    if is_th_h_missing(misspelled, suggestion) {
        score -= 6;
    }

    // Detect dialect-specific variations
    if edit_distance == 1
        && (is_cksz_misspelling(misspelled, suggestion)
            || is_ou_misspelling(misspelled, suggestion)
            || is_ll_misspelling(misspelled, suggestion)
            || is_ay_ey_misspelling(misspelled, suggestion)
            || is_th_h_missing(misspelled, suggestion))
    {
        score -= 6;
    }

    if edit_distance <= 2 {
        if is_ei_ie_misspelling(misspelled, suggestion) {
            score -= 11;
        }
        if is_er_misspelling(misspelled, suggestion) {
            score -= 15;
        }
    }

    score
}

fn parenthetical_plural_s_expr() -> Filter {
    Filter::new(vec![
        Box::new(
            SequenceExpr::default()
                .then_any_word()
                .then_kind_where(|kind| kind.is_open_round())
                .t_aco("s")
                .then_kind_where(|kind| kind.is_close_round()),
        ),
        Box::new(
            SequenceExpr::default()
                .then_kind_where(|kind| kind.is_open_round())
                .t_aco("s")
                .then_kind_where(|kind| kind.is_close_round()),
        ),
        Box::new(SequenceExpr::aco("s")),
    ])
}

/// Returns whether the two words are the same, expect that one is written
/// with 'ou' and the other with 'o'.
///
/// E.g. "color" and "colour"
pub(crate) fn is_ou_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len().abs_diff(b.len()) != 1 {
        return false;
    }

    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    loop {
        match (
            a_iter.next().map(char::to_ascii_lowercase),
            b_iter.next().map(char::to_ascii_lowercase),
        ) {
            (Some('o'), Some('o')) => {
                let mut a_next = a_iter.next().map(char::to_ascii_lowercase);
                let mut b_next = b_iter.next().map(char::to_ascii_lowercase);
                if a_next != b_next {
                    if a_next == Some('u') {
                        a_next = a_iter.next().map(char::to_ascii_lowercase);
                    } else if b_next == Some('u') {
                        b_next = b_iter.next().map(char::to_ascii_lowercase);
                    }

                    if a_next != b_next {
                        return false;
                    }
                }
            }
            (Some(a_char), Some(b_char)) => {
                if !a_char.eq_ignore_ascii_case(&b_char) {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

/// Returns whether the two words are the same, expect for a single confusion of:
///
/// - `s` and `z`. E.g."realize" and "realise"
/// - `s` and `c`. E.g. "defense" and "defence"
/// - `k` and `c`. E.g. "skepticism" and "scepticism"
pub(crate) fn is_cksz_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    if a.is_empty() {
        return true;
    }

    // the first character must be the same
    if !a[0].eq_ignore_ascii_case(&b[0]) {
        return false;
    }

    let mut found = false;
    for (a_char, b_char) in a.iter().copied().zip(b.iter().copied()) {
        let a_char = a_char.to_ascii_lowercase();
        let b_char = b_char.to_ascii_lowercase();

        if a_char != b_char {
            if (a_char == 's' && b_char == 'z')
                || (a_char == 'z' && b_char == 's')
                || (a_char == 's' && b_char == 'c')
                || (a_char == 'c' && b_char == 's')
                || (a_char == 'k' && b_char == 'c')
                || (a_char == 'c' && b_char == 'k')
            {
                if found {
                    return false;
                }
                found = true;
            } else {
                return false;
            }
        }
    }

    found
}

/// Returns whether the two words are the same, expect that one is written
/// with '-er' and the other with '-re'.
///
/// E.g. "meter" and "metre"
pub(crate) fn is_er_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() || a.len() <= 4 {
        return false;
    }

    let len = a.len();
    let a_suffix = [&a[len - 2], &a[len - 1]].map(char::to_ascii_lowercase);
    let b_suffix = [&b[len - 2], &b[len - 1]].map(char::to_ascii_lowercase);

    if a_suffix == ['r', 'e'] && b_suffix == ['e', 'r']
        || a_suffix == ['e', 'r'] && b_suffix == ['r', 'e']
    {
        return a[0..len - 2]
            .iter()
            .copied()
            .zip(b[0..len - 2].iter().copied())
            .all(|(a_char, b_char)| a_char.eq_ignore_ascii_case(&b_char));
    }

    false
}

/// Returns whether the two words are the same, expect that one is written
/// with 'll' and the other with 'l'.
///
/// E.g. "traveller" and "traveler"
pub(crate) fn is_ll_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len().abs_diff(b.len()) != 1 {
        return false;
    }

    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    loop {
        match (
            a_iter.next().map(char::to_ascii_lowercase),
            b_iter.next().map(char::to_ascii_lowercase),
        ) {
            (Some('l'), Some('l')) => {
                let mut a_next = a_iter.next().map(char::to_ascii_lowercase);
                let mut b_next = b_iter.next().map(char::to_ascii_lowercase);
                if a_next != b_next {
                    if a_next == Some('l') {
                        a_next = a_iter.next().map(char::to_ascii_lowercase);
                    } else if b_next == Some('l') {
                        b_next = b_iter.next().map(char::to_ascii_lowercase);
                    }

                    if a_next != b_next {
                        return false;
                    }
                }
            }
            (Some(a_char), Some(b_char)) => {
                if !a_char.eq_ignore_ascii_case(&b_char) {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

/// Whether `b` is `a` with an "h" added after a "t", as in "teh" for "the": a common English
/// misspelling.
pub fn is_th_h_missing(a: &[char], b: &[char]) -> bool {
    a.iter().any(|c| c.eq_ignore_ascii_case(&'t'))
        && b.iter()
            .tuple_windows()
            .any(|(a, b)| a.eq_ignore_ascii_case(&'t') && b.eq_ignore_ascii_case(&'h'))
}

/// Returns whether the two words are the same, except that one is written
/// with 'ay' and the other with 'ey'.
///
/// E.g. "gray" and "grey"
pub(crate) fn is_ay_ey_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut found_ay_ey = false;
    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    while let (Some(&a_char), Some(&b_char)) = (a_iter.next(), b_iter.next()) {
        if a_char.eq_ignore_ascii_case(&b_char) {
            continue;
        }

        // Check for 'a'/'e' difference
        if (a_char.eq_ignore_ascii_case(&'a') && b_char.eq_ignore_ascii_case(&'e'))
            || (a_char.eq_ignore_ascii_case(&'e') && b_char.eq_ignore_ascii_case(&'a'))
        {
            // Check if next character is 'y' for both
            if let (Some(&a_next), Some(&b_next)) = (a_iter.next(), b_iter.next())
                && a_next.eq_ignore_ascii_case(&'y')
                && b_next.eq_ignore_ascii_case(&'y')
            {
                if found_ay_ey {
                    return false; // More than one ay/ey difference
                }
                found_ay_ey = true;
                continue;
            }
        }
        return false; // Non-ay/ey difference found
    }

    if !found_ay_ey {
        return false;
    }
    found_ay_ey
}

/// Returns whether the two words are the same, except that one is written
/// with 'ei' and the other with 'ie'.
///
/// E.g. "recieved" instead of "received", "cheif" instead of "chief"
pub(crate) fn is_ei_ie_misspelling(a: &[char], b: &[char]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut found_ei_ie = false;
    let mut a_iter = a.iter();
    let mut b_iter = b.iter();

    while let (Some(&a_char), Some(&b_char)) = (a_iter.next(), b_iter.next()) {
        if a_char.eq_ignore_ascii_case(&b_char) {
            continue;
        }

        // Check for 'e' vs 'i' in first position
        if a_char.eq_ignore_ascii_case(&'e') && b_char.eq_ignore_ascii_case(&'i') {
            if let (Some(&a_next), Some(&b_next)) = (a_iter.next(), b_iter.next()) {
                // Next chars must be 'i' and 'e' respectively
                if a_next.eq_ignore_ascii_case(&'i') && b_next.eq_ignore_ascii_case(&'e') {
                    if found_ei_ie {
                        return false; // More than one ei/ie difference
                    }
                    found_ei_ie = true;
                    continue;
                }
            }
        }
        // Check for 'i' vs 'e' in first position
        else if a_char.eq_ignore_ascii_case(&'i')
            && b_char.eq_ignore_ascii_case(&'e')
            && let (Some(&a_next), Some(&b_next)) = (a_iter.next(), b_iter.next())
        {
            // Next chars must be 'e' and 'i' respectively
            if a_next.eq_ignore_ascii_case(&'e') && b_next.eq_ignore_ascii_case(&'i') {
                if found_ei_ie {
                    return false; // More than one ei/ie difference
                }
                found_ei_ie = true;
                continue;
            }
        }
        return false;
    }
    found_ei_ie
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::EnglishSpelling;
    use crate::Language;
    use crate::linting::Linter;
    use crate::linting::pooled_linter::for_tests::create_test_pool;
    use crate::linting::spell_check::SpellCheck;
    use crate::linting::tests::{assert_good_and_bad_suggestions, assert_no_lints};
    use crate::spell::{Dictionary, FstDictionary, MergedDictionary, MutableDictionary};
    use crate::{DictWordMetadata, Document};
    use crate::{
        EnglishDialect,
        linting::tests::{assert_lint_count, assert_suggestion_result},
    };
    use std::sync::Arc;

    create_test_pool!(
        SpellCheck,
        SpellCheck<Arc<FstDictionary>, EnglishSpelling>,
        SpellCheck::new(
            crate::lang::en::dictionary::dictionary(),
            Language::English(EnglishDialect::American),
            EnglishSpelling::new()
        )
    );

    // Capitalization tests

    #[test]
    fn america_capitalized() {
        assert_suggestion_result(
            "The word america should be capitalized.",
            test_linter(),
            "The word America should be capitalized.",
        );
    }

    // EnglishDialect tests

    #[test]
    fn harper_automattic_capitalized() {
        assert_lint_count("So should harper and automattic.", test_linter(), 2);
    }

    #[test]
    fn american_color_in_british_dialect() {
        assert_lint_count(
            "Do you like the color?",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn lowercase_uk_suggests_all_caps() {
        assert_good_and_bad_suggestions(
            "We flew to the uk.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            &["We flew to the UK."],
            &[],
        );
    }

    #[test]
    fn title_case_nz_suggests_all_caps() {
        assert_good_and_bad_suggestions(
            "We flew to Nz.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            &["We flew to NZ."],
            &[],
        );
    }

    #[test]
    fn british_ise_words_in_british_dialect() {
        assert_lint_count(
            "We anonymise it with the customisable tokeniser, then we synthesised the results.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn british_ise_words_in_american_dialect() {
        assert_lint_count(
            "We anonymise it with the customisable tokeniser, then we synthesised the results.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::American),
                EnglishSpelling::new(),
            ),
            4,
        );
    }

    #[test]
    fn american_spelling_in_name_in_british_dialect() {
        assert_lint_count(
            "The attack on Pearl Harbor came before Labor Day.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn british_spelling_in_name_in_american_dialect() {
        assert_lint_count(
            "The Ministry of Defence met the Labour Party.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::American),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn american_spelling_outside_name_in_british_dialect() {
        assert_lint_count(
            "The ship left the harbor near Pearl Harbor.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn canadian_words_in_australian_dialect() {
        assert_lint_count(
            "Does your mom like yogourt?",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            2,
        );
    }

    #[test]
    fn australian_words_in_canadian_dialect() {
        assert_lint_count(
            "We mine bauxite to make aluminium.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Canadian),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn mum_and_mummy_not_just_commonwealth() {
        assert_lint_count(
            "Mum's the word about that Egyptian mummy.",
            test_linter(),
            0,
        );
    }

    #[test]
    fn australian_verandah() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn australian_verandah_in_american_dialect() {
        assert_lint_count("Our house has a verandah.", test_linter(), 1);
    }

    #[test]
    fn australian_verandah_in_british_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn australian_verandah_in_canadian_dialect() {
        assert_lint_count(
            "Our house has a verandah.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Canadian),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn mixing_australian_and_canadian_dialects() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn mixing_canadian_and_australian_dialects() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Canadian),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn australian_and_canadian_spellings_that_are_not_american() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            test_linter(),
            2,
        );
    }

    #[test]
    fn australian_and_canadian_spellings_that_are_not_british() {
        assert_lint_count(
            "In summer we sit on the verandah and eat yogourt.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            2,
        );
    }

    #[test]
    fn australian_labour_vs_labor() {
        assert_lint_count(
            "In Australia we write 'labour' but the political party is the 'Labor Party'.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            0,
        )
    }

    #[test]
    fn australian_words_flagged_for_american_english() {
        assert_lint_count(
            "There's an esky full of beers in the back of the ute.",
            test_linter(),
            2,
        );
    }

    #[test]
    fn american_words_not_flagged_for_australian_english() {
        assert_lint_count(
            "In general, utes have unibody construction while pickups have frames.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn abandonware_correction() {
        assert_suggestion_result(
            "abanonedware",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            "abandonware",
        );
    }

    // Unit tests for specific spellcheck corrections

    #[test]
    fn corrects_abandonedware_1131_1166() {
        // assert_suggestion_result(
        assert_suggestion_result(
            "Abandonedware is abandoned. Do not bother submitting issues about the empty page bug. Author moved to greener pastures",
            test_linter(),
            "Abandonware is abandoned. Do not bother submitting issues about the empty page bug. Author moved to greener pastures",
        );
    }

    #[test]
    fn afterwards_not_us() {
        assert_lint_count("afterwards", test_linter(), 1);
    }

    #[test]
    fn afterward_is_us() {
        assert_lint_count("afterward", test_linter(), 0);
    }

    #[test]
    fn afterward_not_au() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn afterwards_is_au() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Australian),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn afterward_not_ca() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Canadian),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn afterwards_is_ca() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Canadian),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn afterward_not_uk() {
        assert_lint_count(
            "afterward",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            1,
        );
    }

    #[test]
    fn afterwards_is_uk() {
        assert_lint_count(
            "afterwards",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            0,
        );
    }

    #[test]
    fn corrects_hes() {
        assert_suggestion_result(
            "hes",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "he's",
        );
    }

    #[test]
    fn corrects_shes() {
        assert_suggestion_result(
            "shes",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "she's",
        );
    }

    #[test]
    fn issue_1876() {
        let user_dialect = EnglishDialect::American;

        // Create a user dictionary with a word normally of another dialect in it.
        let mut user_dict = MutableDictionary::new();
        user_dict.append_word_str(
            "Calibre",
            DictWordMetadata {
                dialects: Some(user_dialect.into()),
                ..Default::default()
            },
        );

        // Create a merged dictionary, using curated first.
        let mut merged_dict = MergedDictionary::new();
        merged_dict.add_dictionary(crate::lang::en::dictionary::dictionary());
        merged_dict.add_dictionary(std::sync::Arc::from(user_dict));
        assert!(merged_dict.contains_word_str("Calibre"));

        // No dialect issues should be found if the word from another dialect is in our user dictionary.
        assert_eq!(
            SpellCheck::new(
                merged_dict.clone(),
                Language::English(user_dialect),
                EnglishSpelling::new()
            )
            .lint(&Document::new_with_language(
                "I like to use the software Calibre.",
                crate::LanguageFamily::English
            ))
            .len(),
            0,
            "Calibre is not part of the user's dialect!"
        );

        assert_eq!(
            SpellCheck::new(
                merged_dict.clone(),
                Language::English(user_dialect),
                EnglishSpelling::new()
            )
            .lint(&Document::new_with_language(
                "I like to use the spelling colour.",
                crate::LanguageFamily::English
            ))
            .len(),
            1
        );
    }

    #[test]
    fn matt_is_allowed() {
        for dialect in EnglishDialect::iter() {
            dbg!(dialect);
            assert_no_lints(
                "Matt is a great name.",
                SpellCheck::new(
                    crate::lang::en::dictionary::dictionary(),
                    Language::English(dialect),
                    EnglishSpelling::new(),
                ),
            );
        }
    }

    #[test]
    fn issue_2026() {
        assert_suggestion_result(
            "'Tere' is supposed to be 'There'",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "'There' is supposed to be 'There'",
        );

        assert_suggestion_result(
            "'fll' is supposed to be 'fill'",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "'fill' is supposed to be 'fill'",
        );
    }
    #[test]
    fn issue_2261() {
        assert_suggestion_result(
            "Generaly",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "Generally",
        );
    }

    #[test]
    fn flag_prepone_in_non_indian_english() {
        assert_lint_count("We had to prepone the meeting", test_linter(), 1);
    }

    #[test]
    fn dont_flag_prepone_in_indian_english() {
        assert_no_lints(
            "We had to prepone the meeting",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::Indian),
                EnglishSpelling::new(),
            ),
        );
    }

    #[test]
    fn dont_flag_certain_entries_with_multiple_case_variants_in_dict() {
        assert_no_lints("PR", test_linter());
        assert_no_lints("MB", test_linter());
        assert_no_lints("OS", test_linter()); // Issue #2585
    }

    #[test]
    fn allows_multiple_parenthetical_plural_s_markers() {
        assert_no_lints("Review the person(s) and document(s).", test_linter());
    }

    #[test]
    fn allows_uppercase_parenthetical_plural_s() {
        assert_no_lints("CONTACT THE PERSON(S).", test_linter());
    }

    #[test]
    fn still_flags_misspelled_word_before_parenthetical_plural_s() {
        assert_lint_count("Please ask each persson(s) to sign.", test_linter(), 1);
    }

    #[test]
    fn still_flags_parenthetical_s_without_preceding_word() {
        assert_lint_count("Please mark (s) on the form.", test_linter(), 1);
    }

    #[test]
    fn still_flags_spaced_parenthetical_s() {
        assert_lint_count("Please ask each person (s) to sign.", test_linter(), 1);
    }

    #[test]
    fn still_flags_longer_parenthetical_marker() {
        assert_lint_count("Please ask each person(ss) to sign.", test_linter(), 1);
    }

    // Tests that were previously in `spell/mod.rs`

    // Tests for dialect-specific misspelling patterns

    // is_ou_misspelling
    #[test]
    fn suggest_color_for_colour_lowercase() {
        assert_suggestion_result("colour", test_linter(), "color");
    }

    #[test]
    fn suggest_colour_for_color_lowercase() {
        assert_suggestion_result(
            "color",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "colour",
        );
    }

    // titlecase
    #[test]
    fn suggest_color_for_colour_titlecase() {
        assert_suggestion_result("Colour", test_linter(), "Color");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_colour_for_color_titlecase() {
        assert_suggestion_result(
            "Color",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "Colour",
        );
    }

    // all-caps
    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_color_for_colour_all_caps() {
        assert_suggestion_result("COLOUR", test_linter(), "COLOR");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_colour_for_color_all_caps() {
        assert_suggestion_result(
            "COLOR",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "COLOUR",
        );
    }

    // is_cksz_misspelling

    // s/z as in realise/realize
    #[test]
    #[ignore = "both spellings are acceptable in UK, AU, and IN despite popular opinion"]
    fn suggest_realise_for_realize() {
        assert_suggestion_result(
            "realize",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "realise",
        );
    }

    #[test]
    fn suggest_realize_for_realise() {
        assert_suggestion_result("realise", test_linter(), "realize");
    }

    #[test]
    #[ignore = "both spellings are acceptable in UK, AU, and IN despite popular opinion"]
    fn suggest_realise_for_realize_titlecase() {
        assert_suggestion_result(
            "Realize",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "Realise",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realize_for_realise_titlecase() {
        assert_suggestion_result("Realise", test_linter(), "Realize");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realise_for_realize_all_caps() {
        assert_suggestion_result(
            "REALIZE",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "REALISE",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_realize_for_realise_all_caps() {
        assert_suggestion_result("REALISE", test_linter(), "REALIZE");
    }

    // s/c as in defense/defence
    #[test]
    fn suggest_defence_for_defense() {
        assert_suggestion_result(
            "defense",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "defence",
        );
    }

    #[test]
    fn suggest_defense_for_defence() {
        assert_suggestion_result("defence", test_linter(), "defense");
    }

    #[test]
    fn suggest_defense_for_defence_titlecase() {
        assert_suggestion_result(
            "Defense",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "Defence",
        );
    }

    #[test]
    fn suggest_defence_for_defense_titlecase() {
        assert_suggestion_result("Defence", test_linter(), "Defense");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_defense_for_defence_all_caps() {
        assert_suggestion_result(
            "DEFENSE",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "DEFENCE",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_defence_for_defense_all_caps() {
        assert_suggestion_result("DEFENCE", test_linter(), "DEFENSE");
    }

    // k/c as in skeptic/sceptic
    #[test]
    fn suggest_sceptic_for_skeptic() {
        assert_suggestion_result(
            "skeptic",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "sceptic",
        );
    }

    #[test]
    fn suggest_skeptic_for_sceptic() {
        assert_suggestion_result("sceptic", test_linter(), "skeptic");
    }

    #[test]
    fn suggest_sceptic_for_skeptic_titlecase() {
        assert_suggestion_result(
            "Skeptic",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "Sceptic",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_skeptic_for_sceptic_titlecase() {
        assert_suggestion_result("Sceptic", test_linter(), "Skeptic");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_skeptic_for_sceptic_all_caps() {
        assert_suggestion_result(
            "SKEPTIC",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "SCEPTIC",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_sceptic_for_skeptic_all_caps() {
        assert_suggestion_result("SCEPTIC", test_linter(), "SKEPTIC");
    }

    // is_er_misspelling
    // as in meter/metre
    #[test]
    fn suggest_centimeter_for_centimetre() {
        assert_suggestion_result("centimetre", test_linter(), "centimeter");
    }

    #[test]
    fn suggest_centimetre_for_centimeter() {
        assert_suggestion_result(
            "centimeter",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "centimetre",
        );
    }

    #[test]
    fn suggest_centimeter_for_centimetre_titlecase() {
        assert_suggestion_result("Centimetre", test_linter(), "Centimeter");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimetre_for_centimeter_titlecase() {
        assert_suggestion_result(
            "Centimeter",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "Centimetre",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimeter_for_centimetre_all_caps() {
        assert_suggestion_result("CENTIMETRE", test_linter(), "CENTIMETER");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_centimetre_for_centimeter_all_caps() {
        assert_suggestion_result(
            "CENTIMETER",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "CENTIMETRE",
        );
    }

    // is_ll_misspelling
    // as in traveller/traveler
    #[test]
    fn suggest_traveler_for_traveller() {
        assert_suggestion_result("traveller", test_linter(), "traveler");
    }

    #[test]
    fn suggest_traveller_for_traveler() {
        assert_suggestion_result(
            "traveler",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "traveller",
        );
    }

    #[test]
    fn suggest_traveler_for_traveller_titlecase() {
        assert_suggestion_result("Traveller", test_linter(), "Traveler");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveller_for_traveler_titlecase() {
        assert_suggestion_result(
            "Traveler",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "Traveller",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveler_for_traveller_all_caps() {
        assert_suggestion_result("TRAVELLER", test_linter(), "TRAVELER");
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_traveller_for_traveler_all_caps() {
        assert_suggestion_result(
            "TRAVELER",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "TRAVELLER",
        );
    }

    // is_ay_ey_misspelling
    // as in gray/grey

    #[test]
    fn suggest_grey_for_gray_in_non_american() {
        assert_suggestion_result(
            "I've got a gray cat.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "I've got a grey cat.",
        );
    }

    #[test]
    fn suggest_gray_for_grey_in_american() {
        assert_suggestion_result(
            "It's a greyscale photo.",
            test_linter(),
            "It's a grayscale photo.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_grey_for_gray_in_non_american_titlecase() {
        assert_suggestion_result(
            "I've Got a Gray Cat.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "I've Got a Grey Cat.",
        );
    }

    #[test]
    fn suggest_gray_for_grey_in_american_titlecase() {
        assert_suggestion_result(
            "It's a Greyscale Photo.",
            test_linter(),
            "It's a Grayscale Photo.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_grey_for_gray_in_non_american_all_caps() {
        assert_suggestion_result(
            "GRAY",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "GREY",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn suggest_gray_for_grey_in_american_all_caps() {
        assert_suggestion_result("GREY", test_linter(), "GRAY");
    }

    // Tests for non-dialectal misspelling patterns

    // is_ei_ie_misspelling
    #[test]
    fn fix_cheif_and_recieved() {
        assert_suggestion_result(
            "The cheif recieved a letter.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "The chief received a letter.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn fix_cheif_and_recieved_titlecase() {
        assert_suggestion_result(
            "The Cheif Recieved a Letter.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "The Chief Received a Letter.",
        );
    }

    #[test]
    #[ignore = "known failure due to bug"]
    fn fix_cheif_and_recieved_all_caps() {
        assert_suggestion_result(
            "THE CHEIF RECIEVED A LETTER.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "THE CHEIF RECEIVED A LETTER.",
        );
    }

    #[test]
    fn fix_vs_apostrophe() {
        assert_suggestion_result(
            "v's",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "vs",
        );
    }

    #[test]
    fn fix_vs_typographical_apostrophe() {
        assert_suggestion_result(
            "v’s",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "vs",
        );
    }

    #[test]
    fn fix_childrens_missing_apostrophe() {
        assert_suggestion_result(
            "childrens",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
            "children's",
        );
    }

    #[test]
    fn allows_informal_laughter() {
        for source in ["hahah", "hahaha", "hahahah", "Hahahah", "HAHAHA"] {
            assert_no_lints(source, test_linter());
        }
    }

    #[test]
    fn allows_surveilled_and_surveilling() {
        assert_no_lints(
            "The suspect was closely surveilled by the agents.",
            test_linter(),
        );
        assert_no_lints("They were surveilling the premises.", test_linter());
        assert_no_lints(
            "The suspect was closely surveilled by the agents.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
        );
        assert_no_lints(
            "They were surveilling the premises.",
            SpellCheck::new(
                crate::lang::en::dictionary::dictionary(),
                Language::English(EnglishDialect::British),
                EnglishSpelling::new(),
            ),
        );
    }
}
