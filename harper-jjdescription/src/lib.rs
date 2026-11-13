use harper_core::Token;
use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser, ProseLexer};
use harper_core::{LanguageFamily, Lrc};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct JJDescriptionParser {
    /// Used to grab the text nodes, and parse them as markdown.
    masker: TreeSitterMasker,
    markdown_options: MarkdownOptions,
    prose: ProseLexer,
}

impl JJDescriptionParser {
    fn node_condition(n: &Node) -> bool {
        n.kind() == "text"
    }

    /// A parser that guesses each document's language (see [`ProseLexer::Guess`]). Use
    /// [`Self::with_language`] when you know the language.
    pub fn new(markdown_options: MarkdownOptions) -> Self {
        Self::with(markdown_options, ProseLexer::Guess)
    }

    /// A parser for prose in `language`.
    pub fn with_language(markdown_options: MarkdownOptions, language: LanguageFamily) -> Self {
        Self::with(markdown_options, ProseLexer::for_language(language))
    }

    /// A parser whose Markdown hands its prose to `prose`.
    pub fn with_lexer(markdown_options: MarkdownOptions, prose: Lrc<dyn Lexer>) -> Self {
        Self::with(markdown_options, ProseLexer::Given(prose))
    }

    fn with(markdown_options: MarkdownOptions, prose: ProseLexer) -> Self {
        Self {
            masker: TreeSitterMasker::new(
                tree_sitter_jjdescription::language(),
                Self::node_condition,
            ),
            markdown_options,
            prose,
        }
    }
}

impl Parser for JJDescriptionParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        {
            let prose = self.prose.for_masked(&self.masker, source);
            let markdown = Markdown::with_lexer(self.markdown_options, prose);
            parsers::parse_masked(&self.masker, &markdown, source)
        }
    }
}
