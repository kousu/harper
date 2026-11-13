use harper_comments::CommentParser;
use harper_core::lexers::Lexer;
use harper_core::{
    LanguageFamily, Lrc, Masker, Token,
    parsers::{self, Markdown, MarkdownOptions, Mask, Parser, ProseLexer},
};

mod masker;
use harper_core::spell::MutableDictionary;
use itertools::Itertools;
use masker::LiterateHaskellMasker;

/// Parses a Literate Haskell document by masking out the code and considering text as Markdown.
pub struct LiterateHaskellParser {
    inner: Inner,
}

/// What parses the text between the code.
enum Inner {
    Parser(Lrc<dyn Parser>),
    Markdown(MarkdownOptions, ProseLexer),
}

impl LiterateHaskellParser {
    /// A parser that hands the text between the code to `inner`.
    pub fn new(inner: Lrc<dyn Parser>) -> Self {
        Self {
            inner: Inner::Parser(inner),
        }
    }

    /// A parser that reads the text between the code as Markdown, and guesses each document's
    /// language (see [`ProseLexer::Guess`]). Use [`Self::new_markdown_with_language`] when you know it.
    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self {
            inner: Inner::Markdown(markdown_options, ProseLexer::Guess),
        }
    }

    /// Like [`Self::new_markdown`], for text in `language`.
    pub fn new_markdown_with_language(
        markdown_options: MarkdownOptions,
        language: LanguageFamily,
    ) -> Self {
        Self {
            inner: Inner::Markdown(markdown_options, ProseLexer::for_language(language)),
        }
    }

    /// Like [`Self::new_markdown`], with the Markdown handing its prose to `prose`.
    pub fn new_markdown_with_lexer(
        markdown_options: MarkdownOptions,
        prose: Lrc<dyn Lexer>,
    ) -> Self {
        Self {
            inner: Inner::Markdown(markdown_options, ProseLexer::Given(prose)),
        }
    }

    pub fn create_ident_dict(
        &self,
        source: &[char],
        markdown_options: MarkdownOptions,
    ) -> Option<MutableDictionary> {
        let parser = CommentParser::new_from_language_id("haskell", markdown_options).unwrap();
        let mask = LiterateHaskellMasker::code_only().create_mask(source);

        let code = mask
            .iter_allowed(source)
            .flat_map(|(_, src)| src.to_owned())
            .collect_vec();
        parser.create_ident_dict(&code)
    }
}

impl Parser for LiterateHaskellParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let masker = LiterateHaskellMasker::text_only();
        match &self.inner {
            Inner::Parser(inner) => Mask::new(masker, inner.clone()).parse(source),
            Inner::Markdown(options, prose) => {
                let markdown = Markdown::with_lexer(*options, prose.for_masked(&masker, source));
                parsers::parse_masked(&masker, &markdown, source)
            }
        }
    }
}
