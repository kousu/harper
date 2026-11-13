use harper_core::Lrc;
use harper_core::Token;
use harper_core::lexers::Lexer;
use harper_core::parsers::{Markdown, MarkdownOptions, Parser};

use super::without_initiators;

#[derive(Clone)]
pub struct Go {
    inner: Lrc<dyn Parser>,
}

impl Go {
    pub fn new(parser: Lrc<dyn Parser>) -> Self {
        Self { inner: parser }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions, prose: Lrc<dyn Lexer>) -> Self {
        Self::new(Lrc::new(Markdown::with_lexer(markdown_options, prose)))
    }
}

impl Parser for Go {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut actual = without_initiators(source);
        let mut actual_source = actual.get_content(source);

        if matches!(actual_source, ['g', 'o', ':', ..]) {
            let Some(terminator) = source.iter().position(|c| *c == '\n') else {
                return Vec::new();
            };

            actual.start += terminator;

            let Some(new_source) = actual.try_get_content(actual_source) else {
                return Vec::new();
            };

            actual_source = new_source
        }

        let mut new_tokens = self.inner.parse(actual_source);

        new_tokens
            .iter_mut()
            .for_each(|t| t.span.push_by(actual.start));

        new_tokens
    }
}
