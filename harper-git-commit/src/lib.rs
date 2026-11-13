use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser, ProseLexer};
use harper_core::{LanguageFamily, Lrc};
use harper_core::{Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct GitCommitParser {
    masker: TreeSitterMasker,
    markdown_options: MarkdownOptions,
    prose: ProseLexer,
}

impl GitCommitParser {
    fn node_condition(n: &Node) -> bool {
        matches!(n.kind(), "subject" | "message_line" | "breaking_change")
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
                tree_sitter_gitcommit::LANGUAGE.into(),
                Self::node_condition,
            ),
            markdown_options,
            prose,
        }
    }
}

impl Default for GitCommitParser {
    fn default() -> Self {
        Self::new(MarkdownOptions::default())
    }
}

impl Parser for GitCommitParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = {
            let prose = self.prose.for_masked(&self.masker, source);
            let markdown = Markdown::with_lexer(self.markdown_options, prose);
            parsers::parse_masked(&self.masker, &markdown, source)
        };

        for token in &mut tokens {
            if let TokenKind::Space(v) = &mut token.kind {
                *v = (*v).clamp(0, 1);
            }
        }

        tokens
    }
}
