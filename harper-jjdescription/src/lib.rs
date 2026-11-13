use harper_core::Lrc;
use harper_core::Token;
use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser, PlainEnglish};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct JJDescriptionParser {
    /// Used to grab the text nodes, and parse them as markdown.
    inner: parsers::Mask<TreeSitterMasker, Markdown>,
}

impl JJDescriptionParser {
    fn node_condition(n: &Node) -> bool {
        n.kind() == "text"
    }

    /// A parser whose prose is English.
    pub fn new(markdown_options: MarkdownOptions) -> Self {
        Self::with_lexer(markdown_options, Lrc::new(PlainEnglish))
    }

    /// A parser whose Markdown hands its prose to `prose`.
    pub fn with_lexer(markdown_options: MarkdownOptions, prose: Lrc<dyn Lexer>) -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_jjdescription::language(), Self::node_condition),
                Markdown::with_lexer(markdown_options, prose),
            ),
        }
    }
}

impl Parser for JJDescriptionParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}
