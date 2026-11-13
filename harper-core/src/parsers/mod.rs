//! Adds support for parsing various programming and markup languages through a unified trait: [`Parser`].

mod collapse_identifiers;
mod isolate_english;
mod markdown;
mod mask;
mod oops_all_headings;
mod org_mode;

#[cfg(feature = "en")]
pub use crate::lexers::PlainEnglish;
pub use crate::lexers::PlainText;
use blanket::blanket;
pub use collapse_identifiers::CollapseIdentifiers;
pub use isolate_english::IsolateEnglish;
pub use markdown::{Markdown, MarkdownOptions};
pub use mask::Mask;
pub use oops_all_headings::OopsAllHeadings;
pub use org_mode::OrgMode;

/// The lexer that file-format parsers hand their prose to unless they are built with one:
/// English's, or [`PlainText`] in a build without English.
pub(crate) fn default_prose() -> crate::Lrc<dyn crate::lexers::Lexer> {
    #[cfg(feature = "en")]
    return crate::Lrc::new(PlainEnglish);
    #[cfg(not(feature = "en"))]
    return crate::Lrc::new(PlainText);
}

use crate::{LSend, Token, TokenStringExt};

#[cfg_attr(feature = "concurrent", blanket(derive(Ref, Box, Arc)))]
#[cfg_attr(not(feature = "concurrent"), blanket(derive(Ref, Box, Rc)))]
pub trait Parser: LSend {
    fn parse(&self, source: &[char]) -> Vec<Token>;
}

pub trait StrParser {
    fn parse_str(&self, source: impl AsRef<str>) -> Vec<Token>;
}

impl<T> StrParser for T
where
    T: Parser,
{
    fn parse_str(&self, source: impl AsRef<str>) -> Vec<Token> {
        let source: Vec<_> = source.as_ref().chars().collect();
        self.parse(&source)
    }
}

#[cfg(all(test, feature = "en"))]
mod tests {
    use super::{Markdown, OrgMode, Parser, PlainEnglish};
    use crate::Punctuation;
    use crate::TokenKind::{self, *};
    use crate::lexers::Lexer;

    fn assert_tokens_eq(test_str: impl AsRef<str>, expected: &[TokenKind], parser: &impl Parser) {
        let chars: Vec<_> = test_str.as_ref().chars().collect();
        let tokens = parser.parse(&chars);
        let kinds: Vec<_> = tokens.into_iter().map(|v| v.kind).collect();

        assert_eq!(&kinds, expected)
    }

    fn assert_tokens_eq_plain(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        assert_tokens_eq(test_str, expected, &PlainEnglish);
    }

    fn assert_tokens_eq_md(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        assert_tokens_eq(test_str, expected, &Markdown::default())
    }

    fn assert_tokens_eq_org(test_str: impl AsRef<str>, expected: &[TokenKind]) {
        assert_tokens_eq(test_str, expected, &OrgMode::default())
    }

    #[test]
    fn single_letter() {
        assert_tokens_eq_plain("a", &[TokenKind::blank_word()])
    }

    #[test]
    fn sentence() {
        assert_tokens_eq_plain(
            "hello world, my friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                Space(1),
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        )
    }

    #[test]
    fn sentence_md() {
        assert_tokens_eq_md(
            "__hello__ world, [my]() friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                Space(1),
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        );
    }

    #[test]
    fn inserts_newlines() {
        assert_tokens_eq_md(
            "__hello__ world,\n\n[my]() friend",
            &[
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
                Punctuation(Punctuation::Comma),
                ParagraphBreak,
                TokenKind::blank_word(),
                Space(1),
                TokenKind::blank_word(),
            ],
        );
    }

    /// Make sure that the English parser correctly identifies non-English
    /// characters as part of the same word.
    #[test]
    fn parses_non_english() {
        assert_tokens_eq_plain("Løvetann", &[TokenKind::blank_word()]);
        assert_tokens_eq_plain("Naïve", &[TokenKind::blank_word()]);
    }

    /// A lexer that makes every letter a word of its own, so tests can tell it apart from
    /// [`PlainEnglish`].
    struct LetterLexer;

    impl Lexer for LetterLexer {
        fn lex(&self, source: &[char]) -> Vec<crate::Token> {
            source
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let kind = if c.is_alphabetic() {
                        TokenKind::blank_word()
                    } else {
                        Space(1)
                    };
                    crate::Token::new(crate::Span::new(i, i + 1), kind)
                })
                .collect()
        }
    }

    /// The text of each word `parser` finds in `source`.
    fn words(source: &str, parser: &impl Parser) -> Vec<String> {
        let chars: Vec<char> = source.chars().collect();
        parser
            .parse(&chars)
            .iter()
            .filter(|t| t.kind.is_word())
            .map(|t| t.span.get_content_string(&chars))
            .collect()
    }

    #[test]
    fn markdown_uses_given_lexer() {
        let parser = Markdown::with_lexer(Default::default(), crate::Lrc::new(LetterLexer));
        assert_eq!(words("# Hi\n\n*ok* `code`", &parser), ["H", "i", "o", "k"]);
    }

    #[test]
    fn org_mode_uses_given_lexer() {
        let parser = OrgMode::with_lexer(crate::Lrc::new(LetterLexer));
        assert_eq!(
            words("* Hi\nok\n#+BEGIN_SRC\ncode\n#+END_SRC", &parser),
            ["H", "i", "o", "k"]
        );
    }

    #[test]
    fn markdown_defaults_to_english() {
        assert_eq!(
            words("# Hi\n\n*ok* `code`", &Markdown::default()),
            words(
                "# Hi\n\n*ok* `code`",
                &Markdown::with_lexer(Default::default(), crate::Lrc::new(PlainEnglish))
            )
        );
        assert_eq!(words("Hi there", &Markdown::default()), ["Hi", "there"]);
    }

    #[test]
    fn org_mode_defaults_to_english() {
        assert_eq!(words("* Hi there", &OrgMode::default()), ["Hi", "there"]);
    }

    #[test]
    fn org_mode_basic() {
        assert_tokens_eq_org(
            "hello world",
            &[TokenKind::blank_word(), Space(1), TokenKind::blank_word()],
        );
    }
}
