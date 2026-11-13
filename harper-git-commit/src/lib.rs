use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser, ProseLexer};
use harper_core::{LanguageFamily, Lrc, Mask, Masker, Span, Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct GitCommitParser {
    masker: GitCommitMasker,
    markdown_options: MarkdownOptions,
    prose: ProseLexer,
}

struct GitCommitMasker {
    content: TreeSitterMasker,
    prefixes: TreeSitterMasker,
}

impl Masker for GitCommitMasker {
    fn create_mask(&self, source: &[char]) -> Mask {
        let content = self.content.create_mask(source);
        let prefixes = self.prefixes.create_mask(source);
        let Some((prefix, _)) = prefixes.iter_allowed(source).next() else {
            return content;
        };

        // Keep the original Git classification, including subjects starting with '#'.
        let mut mask = Mask::new_blank();
        for (span, _) in content.iter_allowed(source) {
            if !span.overlaps_with(prefix) {
                mask.push_allowed(span);
                continue;
            }

            if span.start < prefix.start {
                mask.push_allowed(Span::new(span.start, prefix.start));
            }

            let mut start = prefix.end;
            while start < span.end && matches!(source[start], ' ' | '\t') {
                start += 1;
            }
            if start < span.end {
                mask.push_allowed(Span::new(start, span.end));
            }
        }

        mask
    }
}

impl GitCommitParser {
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
            masker: GitCommitMasker {
                content: TreeSitterMasker::new(
                    tree_sitter_gitcommit::LANGUAGE.into(),
                    |node: &Node| {
                        matches!(node.kind(), "subject" | "message_line" | "breaking_change")
                    },
                ),
                prefixes: TreeSitterMasker::new(tree_sitter_gitcommit::LANGUAGE.into(), |node| {
                    node.kind() == "prefix"
                        && node
                            .parent()
                            .is_some_and(|parent| parent.kind() == "subject")
                }),
            },
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
