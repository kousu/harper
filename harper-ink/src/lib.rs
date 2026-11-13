use harper_core::LanguageFamily;
use harper_core::Lrc;
use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Parser, ProseLexer};
use harper_core::{Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct InkParser {
    masker: TreeSitterMasker,
    prose: ProseLexer,
}

impl InkParser {
    fn node_condition(n: &Node) -> bool {
        matches!(n.kind(), "contentText" | "blockComment" | "lineComment")
    }
}

/// The same as [`InkParser::new`]: it guesses each document's language.
impl Default for InkParser {
    fn default() -> Self {
        Self::new()
    }
}

impl InkParser {
    /// A parser that guesses each document's language (see [`ProseLexer::Guess`]). Use
    /// [`Self::with_language`] when you know the language.
    pub fn new() -> Self {
        Self::with(ProseLexer::Guess)
    }

    /// A parser for prose in `language`.
    pub fn with_language(language: LanguageFamily) -> Self {
        Self::with(ProseLexer::for_language(language))
    }

    /// A parser that hands its prose to `prose`.
    pub fn with_lexer(prose: Lrc<dyn Lexer>) -> Self {
        Self::with(ProseLexer::Given(prose))
    }

    fn with(prose: ProseLexer) -> Self {
        Self {
            masker: TreeSitterMasker::new(
                tree_sitter_ink_lbz::LANGUAGE.into(),
                Self::node_condition,
            ),
            prose,
        }
    }
}

impl Parser for InkParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = parsers::parse_masked(
            &self.masker,
            &self.prose.for_masked(&self.masker, source),
            source,
        );

        for token in &mut tokens {
            if let TokenKind::Space(v) = &mut token.kind {
                *v = (*v).clamp(0, 1);
            }
        }

        tokens
    }
}
