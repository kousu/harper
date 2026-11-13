use harper_core::Lrc;
use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser, PlainEnglish};
use harper_core::{Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct GitCommitParser {
    inner: parsers::Mask<TreeSitterMasker, Markdown>,
}

impl GitCommitParser {
    fn node_condition(n: &Node) -> bool {
        matches!(n.kind(), "subject" | "message_line" | "breaking_change")
    }

    /// A parser whose prose is English.
    pub fn new(markdown_options: MarkdownOptions) -> Self {
        Self::with_lexer(markdown_options, Lrc::new(PlainEnglish))
    }

    /// A parser whose Markdown hands its prose to `prose`.
    pub fn with_lexer(markdown_options: MarkdownOptions, prose: Lrc<dyn Lexer>) -> Self {
        Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(tree_sitter_gitcommit::LANGUAGE.into(), Self::node_condition),
                Markdown::with_lexer(markdown_options, prose),
            ),
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
        let mut tokens = self.inner.parse(source);

        for token in &mut tokens {
            if let TokenKind::Space(v) = &mut token.kind {
                *v = (*v).clamp(0, 1);
            }
        }

        tokens
    }
}
