mod offset_cursor;
mod typst_translator;

use harper_core::lexers::Lexer;
use typst_translator::TypstTranslator;

use harper_core::{
    Lrc, Token,
    parsers::{Parser, PlainEnglish},
};
use itertools::Itertools;
use typst_syntax::{
    Source, SyntaxNode,
    ast::{AstNode, Expr, Markup},
};

/// A parser for Typst files. It hands the prose it finds to an inner parser, which is
/// [`PlainEnglish`] unless it was built with [`Typst::with_lexer`].
#[derive(Clone)]
pub struct Typst {
    prose: Lrc<dyn Lexer>,
}

impl Typst {
    /// A Typst parser that hands its prose to `prose`.
    pub fn with_lexer(prose: Lrc<dyn Lexer>) -> Self {
        Self { prose }
    }
}

/// A Typst parser whose prose is English.
impl Default for Typst {
    fn default() -> Self {
        Self::with_lexer(Lrc::new(PlainEnglish))
    }
}

impl Parser for Typst {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let source_str: String = source.iter().collect();

        // Transform the source into an AST through the `typst_syntax` crate
        let typst_document = Source::detached(source_str);
        let typst_tree = Markup::from_untyped(typst_document.root())
            .expect("Unable to create typst document from parsed tree!");

        // Recurse through AST to create tokens
        let parse_helper = TypstTranslator::new(&typst_document, self.prose.as_ref());
        let mut buf = Vec::new();
        let exprs = typst_tree.exprs().collect_vec();
        let exprs = convert_parbreaks(&mut buf, &exprs);
        parse_helper.parse_exprs(&exprs)
    }
}

/// Converts newlines after certain elements to paragraph breaks
/// This is accomplished here instead of in the translating module because at this point there is
/// still semantic information associated with the elements.
///
/// Newlines are separate expressions in the parse tree (as the Space variant)
fn convert_parbreaks<'a>(buf: &'a mut Vec<SyntaxNode>, exprs: &'a [Expr]) -> Vec<Expr<'a>> {
    // Owned collection of nodes forcibly casted to paragraph breaks
    *buf = exprs
        .iter()
        .map(|e| {
            let mut node = SyntaxNode::placeholder(typst_syntax::SyntaxKind::Parbreak);
            node.synthesize(e.span());
            node
        })
        .collect_vec();

    let should_parbreak = |e1, e2, e3| {
        matches!(e2, Expr::Space(_))
            && (matches!(e1, Expr::Heading(_) | Expr::ListItem(_))
                || matches!(e3, Expr::Heading(_) | Expr::ListItem(_)))
    };

    let mut res: Vec<Expr> = Vec::new();
    let mut last_element: Option<Expr> = None;
    for ((i, expr), (_, next_expr)) in exprs.iter().enumerate().tuple_windows() {
        let mut current_expr = *expr;
        if let Some(last_element) = last_element
            && should_parbreak(last_element, *expr, *next_expr)
        {
            let pbreak = typst_syntax::ast::Parbreak::from_untyped(&buf[i])
                .expect("Unable to convert expression to Parbreak");
            current_expr = Expr::Parbreak(pbreak);
        }
        res.push(current_expr);
        last_element = Some(*expr)
    }
    // Push last element because it will be excluded by tuple_windows() above
    if let Some(last) = exprs.iter().last() {
        res.push(*last);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use harper_core::parsers::StrParser;

    /// A lexer that makes every letter a word of its own.
    struct LetterLexer;

    impl Lexer for LetterLexer {
        fn lex(&self, source: &[char]) -> Vec<Token> {
            source
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let kind = if c.is_alphabetic() {
                        harper_core::TokenKind::blank_word()
                    } else {
                        harper_core::TokenKind::Space(1)
                    };
                    Token::new(harper_core::Span::new(i, i + 1), kind)
                })
                .collect()
        }
    }

    #[test]
    fn uses_given_lexer() {
        let source: Vec<char> = "= Hi\n\nok #\"no\"".chars().collect();
        let words: Vec<String> = Typst::with_lexer(Lrc::new(LetterLexer))
            .parse(&source)
            .iter()
            .filter(|t| t.kind.is_word())
            .map(|t| t.span.get_content_string(&source))
            .collect();
        assert_eq!(words, ["H", "i", "o", "k", "n", "o"]);
    }

    #[test]
    fn issue_1898() {
        Typst::default().parse_str("#for ");
        Typst::default().parse_str("#(.$#$$$. ");
        Typst::default().parse_str("=#{m\"\".'m\"\"#p#");
    }

    #[test]
    fn issue_2126_show_rule_preserves_token_order() {
        let tokens = Typst::default().parse_str("#show \"a\": \"a\"");
        let spans = tokens
            .iter()
            .map(|token| (token.span.start, token.span.end))
            .collect_vec();

        assert_eq!(spans, vec![(7, 8), (12, 13)]);
    }
}
