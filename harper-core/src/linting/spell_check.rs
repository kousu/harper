use std::{iter::once, num::NonZero};

use itertools::Itertools;
use lru::LruCache;
use smallvec::ToSmallVec;

use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::spell::{Dictionary, suggest_correct_spelling_with};
use crate::{CharString, CharStringExt, LSend, Language, TokenStringExt};

/// The parts of spell checking that depend on the language. Each method's default does nothing,
/// for languages that have nothing to add.
pub(crate) trait SpellingConventions: LSend {
    /// Whether to accept `word` without looking it up, as English accepts "haha".
    fn is_ignored(&self, _word: &[char]) -> bool {
        false
    }

    /// Drop the lints in places spelling isn't checked, such as inside proper nouns.
    fn remove_exempt(&self, _document: &Document, _lints: &mut Vec<Lint>) {}

    /// Added to a suggestion's score (lower is better), given the misspelled word, the
    /// suggestion and their edit distance. See [`suggest_correct_spelling_with`].
    fn score_adjustment(
        &self,
        _misspelled: &[char],
        _suggestion: &[char],
        _edit_distance: u8,
    ) -> i32 {
        0
    }
}

pub struct SpellCheck<T, C>
where
    T: Dictionary,
    C: SpellingConventions,
{
    dictionary: T,
    suggestion_cache: LruCache<CharString, Vec<CharString>>,
    /// The language and dialect to accept words from.
    language: Language,
    /// What spell checking `language` needs beyond the dictionary.
    conventions: C,
}

impl<T: Dictionary, C: SpellingConventions> SpellCheck<T, C> {
    pub fn new(dictionary: T, language: Language, conventions: C) -> Self {
        Self {
            dictionary,
            suggestion_cache: LruCache::new(NonZero::new(10000).unwrap()),
            language,
            conventions,
        }
    }

    const MAX_SUGGESTIONS: usize = 3;

    fn suggest_correct_spelling(&mut self, word: &[char]) -> Vec<CharString> {
        if let Some(hit) = self.suggestion_cache.get(word) {
            hit.clone()
        } else {
            let suggestions = self.uncached_suggest_correct_spelling(word);
            self.suggestion_cache.put(word.into(), suggestions.clone());
            suggestions
        }
    }

    fn uncached_suggest_correct_spelling(&self, word: &[char]) -> Vec<CharString> {
        // Back off until we find a match.
        (2..5)
            .filter_map(|dist| {
                let mut iter =
                    suggest_correct_spelling_with(word, 200, dist, &self.dictionary, |m, s, d| {
                        self.conventions.score_adjustment(m, s, d)
                    })
                    .into_iter()
                    .filter(|v| {
                        // Ignore entries outside the configured language and dialect
                        self.dictionary
                            .get_word_metadata(v)
                            .is_some_and(|meta| meta.allows_language(self.language))
                    })
                    .unique_by(|v| v.to_lower())
                    .map(|v| v.to_smallvec());

                // Lazily peek at the first matching element.
                // If None, this entire distance is skipped without allocating a Vec!
                let first = iter.next()?;

                // If we get here, we found at least one match.
                // Now we pull the rest up to our limit and collect.
                let suggestions = once(first)
                    .chain(iter)
                    .take(Self::MAX_SUGGESTIONS)
                    .collect();

                Some(suggestions)
            })
            .next() // Grab the first distance that successfully yielded a Vec
            .unwrap_or_default()
    }
}

impl<T: Dictionary, C: SpellingConventions> Linter for SpellCheck<T, C> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word in document.iter_words() {
            let word_chars = document.get_span_content(&word.span);

            if self.conventions.is_ignored(word_chars) {
                continue;
            }

            // Look the word up in our own dictionary, not the document's: ours holds the user's
            // words too.
            if self
                .dictionary
                .get_word_metadata(word_chars)
                .is_some_and(|metadata| metadata.allows_language(self.language))
                && (self.dictionary.contains_exact_word(word_chars)
                    || self
                        .dictionary
                        .contains_exact_word(word_chars.normalized().as_ref())
                    || self
                        .dictionary
                        .contains_exact_word(word_chars.normalized().to_lower().as_ref()))
            {
                continue;
            };

            let mut possibilities = self.suggest_correct_spelling(word_chars);

            // If the misspelled word is capitalized, capitalize the results too.
            if let Some(mis_f) = word_chars.first()
                && mis_f.is_uppercase()
            {
                for sug_f in possibilities.iter_mut().filter_map(|w| {
                    // Skip words that have uppercase chars in any position except the first.
                    // (For words with specific capitalization, like 'macOS')
                    w.iter()
                        .skip(1)
                        .all(|c| !c.is_uppercase())
                        .then_some(w.first_mut())
                        .flatten()
                }) {
                    if let Some(upper_c) = sug_f.to_uppercase().next() {
                        *sug_f = upper_c;
                    }
                }
            }

            // `OrthographicConsistency` leaves two-letter words to us, since they're often typos.
            // Offer the all-caps form last, so that `uk` can still become `UK`.
            if word_chars.len() == 2
                && let Some(canonical) = self.dictionary.get_correct_capitalization_of(word_chars)
                && canonical.iter().all(|c| c.is_uppercase())
            {
                possibilities.push(canonical.to_smallvec());
            }

            let suggestions: Vec<_> = possibilities
                .iter()
                .map(|sug| Suggestion::ReplaceWith(sug.to_vec()))
                .collect();

            // If there's only one suggestion, save the user a step in the GUI
            let message = if suggestions.len() == 1 {
                format!(
                    "Did you mean `{}`?",
                    possibilities.first().unwrap().iter().collect::<String>()
                )
            } else {
                format!(
                    "Did you mean to spell `{}` this way?",
                    document.get_span_content_str(&word.span)
                )
            };

            lints.push(Lint {
                span: word.span,
                lint_kind: LintKind::Spelling,
                suggestions,
                message,
                priority: 63,
            })
        }

        self.conventions.remove_exempt(document, &mut lints);

        lints
    }

    fn description(&self) -> &'static str {
        "Looks and provides corrections for misspelled words."
    }
}

#[cfg(all(test, feature = "en"))]
mod tests {
    use super::{SpellCheck, SpellingConventions};
    use crate::linting::{Lint, Linter};

    use crate::{Document, EnglishDialect, Language};

    /// Ignores "zzz", drops every lint on "qqq", and prefers suggestions starting with "w".
    struct TestSpelling;

    impl SpellingConventions for TestSpelling {
        fn is_ignored(&self, word: &[char]) -> bool {
            word == ['z', 'z', 'z']
        }

        fn remove_exempt(&self, document: &Document, lints: &mut Vec<Lint>) {
            lints.retain(|lint| document.get_span_content_str(&lint.span) != "qqq");
        }

        fn score_adjustment(&self, _misspelled: &[char], suggestion: &[char], _: u8) -> i32 {
            if suggestion.first() == Some(&'w') {
                -100
            } else {
                0
            }
        }
    }

    /// The words `SpellCheck` with `conventions` flags in `text`.
    fn flagged(text: &str, conventions: impl SpellingConventions) -> Vec<String> {
        let document = Document::new_plain_english_curated(text);
        let mut linter = SpellCheck::new(
            crate::lang::en::dictionary::dictionary(),
            Language::English(EnglishDialect::American),
            conventions,
        );

        linter
            .lint(&document)
            .iter()
            .map(|lint| document.get_span_content_str(&lint.span))
            .collect()
    }

    struct NoConventions;
    impl SpellingConventions for NoConventions {}

    #[test]
    fn flags_unknown_words_without_conventions() {
        assert_eq!(flagged("zzz and qqq", NoConventions), ["zzz", "qqq"]);
    }

    #[test]
    fn conventions_can_ignore_words() {
        assert!(!flagged("zzz", TestSpelling).contains(&"zzz".to_owned()));
    }

    #[test]
    fn conventions_can_remove_lints() {
        assert!(!flagged("qqq", TestSpelling).contains(&"qqq".to_owned()));
    }

    #[test]
    fn conventions_adjust_suggestion_order() {
        let document = Document::new_plain_english_curated("thw");
        let mut linter = SpellCheck::new(
            crate::lang::en::dictionary::dictionary(),
            Language::English(EnglishDialect::American),
            TestSpelling,
        );
        let lints = linter.lint(&document);
        let first = &lints[0].suggestions[0];

        assert!(
            matches!(first, crate::linting::Suggestion::ReplaceWith(w) if w.first() == Some(&'w')),
            "{first:?}"
        );
    }
}
