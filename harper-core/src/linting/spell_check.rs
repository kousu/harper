use std::num::NonZero;

use lru::LruCache;
use smallvec::ToSmallVec;

use super::Suggestion;
use super::{Lint, LintKind, Linter};
use crate::document::Document;
use crate::edit_distance::edit_distance;
use crate::spell::{Dictionary, spelling_candidates_with};
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

    /// How many spelling candidates [`Self::order_by_context`] chooses from.
    const CANDIDATES: usize = 3;

    /// Reorder `candidates`, in spelling order with their scores, to fit the word at token
    /// `index` of `document`, such as by the words around it. By default, keeps the spelling
    /// order.
    fn order_by_context(
        &self,
        candidates: Vec<(CharString, i32)>,
        _document: &Document,
        _index: usize,
    ) -> Vec<CharString> {
        candidates
            .into_iter()
            .map(|(candidate, _)| candidate)
            .collect()
    }
}

pub struct SpellCheck<T, C>
where
    T: Dictionary,
    C: SpellingConventions,
{
    dictionary: T,
    suggestion_cache: LruCache<CharString, Vec<(CharString, i32)>>,
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

    fn suggest_correct_spelling(&mut self, word: &[char]) -> Vec<(CharString, i32)> {
        if let Some(hit) = self.suggestion_cache.get(word) {
            hit.clone()
        } else {
            let suggestions = spelling_candidates_with(
                word,
                &self.dictionary,
                C::CANDIDATES,
                |meta| meta.allows_language(self.language),
                |m, s, d| self.conventions.score_adjustment(m, s, d),
            );
            self.suggestion_cache.put(word.into(), suggestions.clone());
            suggestions
        }
    }
}

impl<T: Dictionary, C: SpellingConventions> Linter for SpellCheck<T, C> {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let mut lints = Vec::new();

        for word_index in document.iter_word_indices() {
            let word = &document.get_tokens()[word_index];
            let word_chars = document.get_span_content(&word.span);

            if self.conventions.is_ignored(word_chars) {
                continue;
            }

            if let Some(Some(metadata)) = word.kind.as_word()
                && metadata.allows_language(self.language)
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

            let candidates = self.suggest_correct_spelling(word_chars);
            let mut possibilities: Vec<CharString> = self
                .conventions
                .order_by_context(candidates, document, word_index)
                .into_iter()
                .take(Self::MAX_SUGGESTIONS)
                .collect();

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
                priority: if possibilities.first().is_some_and(|candidate| {
                    word_chars.len() < 255
                        && candidate.len() < 255
                        && edit_distance(word_chars, candidate) == 1
                }) {
                    32
                } else {
                    63
                },
            })
        }

        self.conventions.remove_exempt(document, &mut lints);

        lints
    }

    fn description(&self) -> &'static str {
        "Looks and provides corrections for misspelled words."
    }
}

#[cfg(test)]
mod tests {
    use super::{SpellCheck, SpellingConventions};
    use crate::linting::{Lint, Linter};
    use crate::spell::FstDictionary;
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
            FstDictionary::curated(),
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
            FstDictionary::curated(),
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
