//! Rules for French text. They expect a [`Document`](crate::Document) built for
//! [`LanguageFamily::French`](crate::LanguageFamily::French), so that elisions like "l'" are words of their own.

mod article_gender;
mod correct_number_suffix;
mod elision;

pub use article_gender::ArticleGender;
pub use correct_number_suffix::CorrectNumberSuffix;
pub use elision::Elision;

use crate::Token;

/// The word token after `index`, if only a single space separates them.
fn next_word(tokens: &[Token], index: usize) -> Option<&Token> {
    match tokens.get(index + 1..index + 3)? {
        [space, word] if space.kind.is_space() && word.kind.is_word() => Some(word),
        _ => None,
    }
}

/// Whether `word` is a name, acronym, Roman numeral or code ("Ivan", "GPL", "IV", "M3") written
/// after a lowercase `before`. Such words often don't follow the usual rules for articles.
fn is_name_after(before: &[char], word: &[char]) -> bool {
    let is_all_upper = before.iter().all(|c| !c.is_lowercase());
    let starts_upper = word.first().is_some_and(|c| c.is_uppercase());
    let has_digit = word.iter().any(|c| c.is_ascii_digit());

    has_digit || (starts_upper && !is_all_upper)
}

/// Whether the token after `index` is a hyphen, as in "méta-analyse".
fn precedes_hyphen(tokens: &[Token], index: usize) -> bool {
    tokens.get(index + 1).is_some_and(|t| t.kind.is_hyphen())
}

/// Whether the token before `index` is a hyphen, as in "ai-je" or "fais-le".
fn follows_hyphen(tokens: &[Token], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|i| tokens.get(i))
        .is_some_and(|t| t.kind.is_hyphen())
}

#[cfg(all(test, feature = "fr"))]
pub(crate) mod tests {
    use crate::linting::Linter;
    use crate::{Document, LanguageFamily};

    fn french_document(text: &str) -> Document {
        Document::new_with_language(text, LanguageFamily::French)
    }

    /// Apply the first suggestion of every lint to `text`.
    pub fn fix(text: &str, mut linter: impl Linter) -> String {
        let document = french_document(text);
        let mut lints = linter.lint(&document);
        lints.sort_by_key(|lint| std::cmp::Reverse(lint.span.start));

        let mut source = document.get_source().to_vec();
        for lint in lints {
            if let Some(suggestion) = lint.suggestions.first() {
                suggestion.apply(lint.span, &mut source);
            }
        }

        source.into_iter().collect()
    }

    #[track_caller]
    pub fn assert_fixes(text: &str, linter: impl Linter, expected: &str) {
        assert_eq!(fix(text, linter), expected);
    }

    #[track_caller]
    pub fn assert_allows(text: &str, mut linter: impl Linter) {
        let lints = linter.lint(&french_document(text));
        assert!(lints.is_empty(), "unexpected lints in {text:?}: {lints:#?}");
    }
}
