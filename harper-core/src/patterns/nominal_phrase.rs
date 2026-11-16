use crate::Token;

use super::Pattern;

/// A pattern that uses primitive syntax-tree heuristics to locate nominal phrases.
/// Given that it does not take context into account, it is not recommended for new code.
/// Please prefer [`DictWordMetadata::np_member`](crate::DictWordMetadata::np_member).
#[derive(Default)]
pub struct NominalPhrase;

impl Pattern for NominalPhrase {
    fn matches(&self, tokens: &[Token], _source: &[char]) -> Option<usize> {
        let mut cursor = 0;

        loop {
            let tok = tokens.get(cursor)?;

            if (tok.kind.is_adjective()
                || tok.kind.is_determiner()
                || tok.kind.is_verb_progressive_form())
                && let Some(next) = tokens.get(cursor + 1)
                && next.kind.is_whitespace()
            {
                cursor += 2;
                continue;
            }

            if tok.kind.is_nominal() {
                return Some(cursor + 1);
            }

            return None;
        }
    }
}

#[cfg(all(test, feature = "en"))]
mod tests {
    use super::super::DocPattern;
    use super::NominalPhrase;
    use crate::{Document, Span, Token, patterns::Pattern};

    trait SpanVecExt {
        fn to_strings(&self, doc: &Document) -> Vec<String>;
    }

    impl SpanVecExt for Vec<Span<Token>> {
        fn to_strings(&self, doc: &Document) -> Vec<String> {
            self.iter()
                .map(|sp| {
                    doc.get_tokens()[sp.start..sp.end]
                        .iter()
                        .map(|tok| doc.get_span_content_str(&tok.span))
                        .collect::<String>()
                })
                .collect()
        }
    }

    #[test]
    fn simple_apple() {
        let doc = Document::new_with_parser_and_language(
            "A red apple",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(matches.to_strings(&doc), vec!["A red apple"])
    }

    #[test]
    fn complex_apple() {
        let doc = Document::new_with_parser_and_language(
            "A red apple with a long stem",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(matches.to_strings(&doc), vec!["A red apple", "a long stem"])
    }

    #[test]
    fn list_fruit() {
        let doc = Document::new_with_parser_and_language(
            "An apple, a banana and a pear",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

        assert_eq!(
            matches.to_strings(&doc),
            vec!["An apple", "a banana", "a pear"]
        )
    }

    #[test]
    fn simplest_banana() {
        let doc = Document::new_with_parser_and_language(
            "a banana",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        assert!(
            NominalPhrase
                .matches(doc.get_tokens(), doc.get_source())
                .is_some()
        );
    }

    #[test]
    fn food() {
        let doc = Document::new_with_parser_and_language(
            "My favorite foods are pizza, sushi, tacos and burgers.",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        let matches = NominalPhrase.find_all_matches_in_doc(&doc);

        dbg!(&matches);
        dbg!(matches.to_strings(&doc));

        for span in &matches {
            let gc = span
                .to_char_span(doc.get_tokens())
                .get_content(doc.get_source());
            dbg!(gc);
        }

        assert_eq!(
            matches.to_strings(&doc),
            vec!["My favorite foods", "pizza", "sushi", "tacos", "burgers"]
        )
    }

    #[test]
    fn simplest_way() {
        let doc = Document::new_with_parser_and_language(
            "a way",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        assert!(
            NominalPhrase
                .matches(doc.get_tokens(), doc.get_source())
                .is_some()
        );
    }

    #[test]
    fn present_participle_way() {
        let doc = Document::new_with_parser_and_language(
            "a winning way",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        assert!(
            NominalPhrase
                .matches(doc.get_tokens(), doc.get_source())
                .is_some()
        );
    }

    #[test]
    fn perfect_participle_way() {
        let doc = Document::new_with_parser_and_language(
            "a failed way",
            &crate::parsers::Markdown::with_language(
                crate::parsers::MarkdownOptions::default(),
                crate::LanguageFamily::English,
            ),
            crate::LanguageFamily::English,
        );
        assert!(
            NominalPhrase
                .matches(doc.get_tokens(), doc.get_source())
                .is_some()
        );
    }
}
