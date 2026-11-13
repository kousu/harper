//! This module implements rudimentary, dictionary-based language detection.
//!
//! harper-core never guesses a language itself: whoever calls it says which language the text is
//! in. [`guess_language`] is for front ends that want to offer automatic detection.

use strum::IntoEnumIterator;

use crate::spell::{Dictionary, FstDictionary};
use crate::{Document, LanguageFamily, Token, TokenKind};

/// How much of a text [`guess_language`] reads.
pub const GUESS_SAMPLE_CHARS: usize = 4000;

/// Guess which of the languages built into Harper `text` is written in.
///
/// Each language lexes `text` with its lexer and checks it against its curated dictionary,
/// as [`is_likely_english`] does for English. Among the languages that pass, the one whose
/// dictionary knows the largest share of the words wins. Returns `None` when no language passes:
/// for text that is too short, mostly code, or in a language this build doesn't include.
///
/// It only picks the language. To guess an English dialect as well, see
/// [`EnglishDialect::try_guess_from_document`](crate::EnglishDialect::try_guess_from_document).
///
/// Guess once, on a whole document: short pieces such as a single heading or comment guess badly.
/// Only the first [`GUESS_SAMPLE_CHARS`] characters are read, so guessing stays cheap on long
/// documents.
pub fn guess_language(text: &str) -> Option<LanguageFamily> {
    let source: Vec<char> = text.chars().take(GUESS_SAMPLE_CHARS).collect();

    LanguageFamily::iter()
        .filter_map(|language| {
            let tokens = language.lexer().lex(&source);
            let dictionary = FstDictionary::curated_for(language);
            let has_words = tokens.iter().any(|t| t.kind.is_word());
            (has_words && is_likely_english(&tokens, &source, &dictionary))
                .then(|| (language, known_word_share(&tokens, &source, &dictionary)))
        })
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(language, _)| language)
}

/// The share of the words in `toks` that `dict` knows, from 0 to 1.
fn known_word_share(toks: &[Token], source: &[char], dict: &impl Dictionary) -> f64 {
    let words: Vec<_> = toks.iter().filter(|t| t.kind.is_word()).collect();
    if words.is_empty() {
        return 0.0;
    }
    let known = words
        .iter()
        .filter(|t| dict.contains_word(t.get_ch(source)))
        .count();
    known as f64 / words.len() as f64
}

/// Check if the contents of the document are likely intended to represent
/// English.
pub fn is_doc_likely_english(doc: &Document, dict: &impl Dictionary) -> bool {
    is_likely_english(doc.get_tokens(), doc.get_source(), dict)
}

/// Check if given tokens are likely intended to represent English.
///
/// Nothing here is specific to English except the dictionary: given another language's tokens and
/// dictionary, it checks for that language instead (see [`guess_language`]).
pub fn is_likely_english(toks: &[Token], source: &[char], dict: &impl Dictionary) -> bool {
    let mut total_words = 0;
    let mut valid_words = 0;
    let mut punctuation = 0;
    let mut unlintable = 0;

    for token in toks {
        match token.kind {
            TokenKind::Word(_) => {
                total_words += 1;

                let word_content = token.get_ch(source);
                if dict.contains_word(word_content) {
                    valid_words += 1;
                }
            }
            TokenKind::Punctuation(_) => punctuation += 1,
            TokenKind::Unlintable => unlintable += 1,
            _ => (),
        }
    }

    if total_words <= 7 && total_words - valid_words > 0 {
        return false;
    }

    if unlintable > valid_words {
        return false;
    }

    if (punctuation as f32 * 1.25) > valid_words as f32 {
        return false;
    }

    if (valid_words as f64 / total_words as f64) < 0.7 {
        return false;
    }

    true
}

#[cfg(all(test, feature = "en"))]
mod tests {
    use super::is_doc_likely_english;
    use crate::Document;

    fn assert_not_english(source: &'static str) {
        let dict = crate::lang::en::dictionary::dictionary();
        let doc = Document::new_plain_english(source, &dict);
        let is_likely_english = is_doc_likely_english(&doc, &dict);
        dbg!(source);
        assert!(!is_likely_english);
    }

    fn assert_english(source: &'static str) {
        let dict = crate::lang::en::dictionary::dictionary();
        let doc = Document::new_plain_english(source, &dict);
        let is_likely_english = is_doc_likely_english(&doc, &dict);
        dbg!(source);
        assert!(is_likely_english);
    }

    #[test]
    fn detects_spanish() {
        assert_not_english("Esto es español. Harper no debería marcarlo como inglés.");
    }

    #[test]
    fn detects_french() {
        assert_not_english(
            "C'est du français. Il ne devrait pas être marqué comme anglais par Harper.",
        );
    }

    #[test]
    fn detects_shebang() {
        assert_not_english("#! /bin/bash");
        assert_not_english("#! /usr/bin/fish");
    }

    #[test]
    fn detects_short_english() {
        assert_english("This is English!");
    }

    #[test]
    fn detects_english() {
        assert_english("This is perfectly valid English, evn if it has a cople typos.")
    }

    #[test]
    fn detects_expressive_english() {
        assert_english("Look above! That is real English! So is this: bippity bop!")
    }

    /// Useful for detecting commented-out code.
    #[test]
    fn detects_python_fib() {
        assert_not_english(
            r"
def fibIter(n):
    if n < 2:
        return n
    fibPrev = 1
    fib = 1
    for _ in range(2, n):
        fibPrev, fib = fib, fib + fibPrev
    return fib
        ",
        );
    }

    #[test]
    fn mixed_french_english_park() {
        assert_not_english("Je voudrais promener au the park a huit heures with ma voisine");
    }

    #[test]
    fn mixed_french_english_drunk() {
        assert_not_english("Je ne suis pas drunk, je suis only ivre by you");
    }

    #[test]
    fn mixed_french_english_dress() {
        assert_not_english(
            "Je buy une robe nouveau chaque Tuesday, mais aujourd'hui, je don't have temps",
        );
    }

    #[test]
    fn english_motto() {
        assert_english("I have a simple motto in life");
    }
}

#[cfg(all(test, feature = "en"))]
mod guess_tests {
    use super::guess_language;
    use crate::LanguageFamily;

    #[test]
    fn guesses_english() {
        assert_eq!(
            guess_language("This is a sentence written in plain English, with nothing unusual."),
            Some(LanguageFamily::English)
        );
    }

    #[test]
    fn does_not_guess_code() {
        assert_eq!(guess_language("fn main() { let x = vec![1, 2, 3]; }"), None);
    }

    #[test]
    fn does_not_guess_empty_text() {
        assert_eq!(guess_language(""), None);
    }
}

#[cfg(all(test, not(feature = "en")))]
mod guess_without_english_tests {
    use super::guess_language;

    #[test]
    fn english_is_not_guessed_without_the_feature() {
        assert_eq!(
            guess_language("This is a sentence written in plain English, with nothing unusual."),
            None
        );
    }
}
