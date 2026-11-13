use harper_core::LanguageFamily;
use harper_core::Lrc;
use harper_core::lexers::Lexer;
use harper_core::parsers::{self, Parser, ProseLexer};
use harper_core::{Token, TokenKind};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

pub struct PythonParser {
    /// Used to grab the text nodes.
    masker: TreeSitterMasker,
    prose: ProseLexer,
}

impl PythonParser {
    fn node_condition(n: &Node) -> bool {
        if n.kind().contains("comment") {
            return true;
        }
        if n.kind() == "string_content"
            && let Some(expr_stmt) = parent_is_expression_statement(n)
            && (is_module_level_docstring(&expr_stmt)
                || is_fn_or_class_docstrings(&expr_stmt)
                || is_attribute_docstring(&expr_stmt))
        {
            return true;
        }
        false
    }
}

/// The same as [`PythonParser::new`]: it guesses each document's language.
impl Default for PythonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonParser {
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
                tree_sitter_python::LANGUAGE.into(),
                Self::node_condition,
            ),
            prose,
        }
    }
}

impl Parser for PythonParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let mut tokens = parsers::parse_masked(
            &self.masker,
            &self.prose.for_masked(&self.masker, source),
            source,
        );

        let mut prev_kind: Option<&TokenKind> = None;

        for token in &mut tokens {
            if let TokenKind::Space(v) = &mut token.kind {
                if let Some(TokenKind::Newline(_)) = &prev_kind {
                    // Lines in multiline docstrings are indented with spaces to match the current level.
                    // We need to remove such spaces to avoid triggering French spaces rule.
                    *v = 0;
                } else {
                    *v = (*v).clamp(0, 1);
                }
            }

            prev_kind = Some(&token.kind);
        }

        tokens
    }
}

fn parent_is_expression_statement<'a>(node: &Node<'a>) -> Option<Node<'a>> {
    node.parent()
        .filter(|n| n.kind() == "string")
        .and_then(|string_node| string_node.parent())
        .filter(|n| n.kind() == "expression_statement")
}

#[inline]
fn is_module_level_docstring(expr_stmt: &Node) -> bool {
    // (module . (expression_statement (string)))
    expr_stmt.parent().is_some_and(|n| n.kind() == "module")
}

#[inline]
fn is_fn_or_class_docstrings(expr_stmt: &Node) -> bool {
    // (class/func_definition body: (block . (expression_statement (string))))
    expr_stmt
        .parent()
        .filter(|n| n.kind() == "block")
        .and_then(|n| n.parent())
        .is_some_and(|n| n.kind() == "function_definition" || n.kind() == "class_definition")
}

#[inline]
fn is_attribute_docstring(expr_stmt: &Node) -> bool {
    // ((expression_statement (assignment)) . (expression_statement (string)))
    expr_stmt
        .prev_sibling()
        .filter(|s| s.kind() == "expression_statement")
        .and_then(|s| s.child(0))
        .is_some_and(|c| c.kind() == "assignment")
}
