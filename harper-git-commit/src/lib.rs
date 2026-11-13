use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Markdown, MarkdownOptions, Parser, PlainEnglish};
use harper_core::{Lrc, Mask, Masker, Span, Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct GitCommitParser {
    inner: parsers::Mask<GitCommitMasker, Markdown>,
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
    /// A parser whose prose is English.
    pub fn new(markdown_options: MarkdownOptions) -> Self {
        Self::with_lexer(markdown_options, Lrc::new(PlainEnglish))
    }

    /// A parser whose Markdown hands its prose to `prose`.
    pub fn with_lexer(markdown_options: MarkdownOptions, prose: Lrc<dyn Lexer>) -> Self {
        Self {
            inner: parsers::Mask::new(
                GitCommitMasker {
                    content: TreeSitterMasker::new(
                        tree_sitter_gitcommit::LANGUAGE.into(),
                        |node: &Node| {
                            matches!(node.kind(), "subject" | "message_line" | "breaking_change")
                        },
                    ),
                    prefixes: TreeSitterMasker::new(
                        tree_sitter_gitcommit::LANGUAGE.into(),
                        |node| {
                            node.kind() == "prefix"
                                && node
                                    .parent()
                                    .is_some_and(|parent| parent.kind() == "subject")
                        },
                    ),
                },
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
