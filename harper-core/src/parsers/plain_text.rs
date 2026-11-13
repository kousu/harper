use super::Parser;
use crate::Token;
use crate::lexing::{lex_base_token, lex_with};

/// A parser that reads all of its input as prose, with only the tokens every Latin-script
/// language shares (see `lexing::lex_base_token`). For text in a language Harper has no parser for.
#[derive(Clone, Copy)]
pub struct PlainText;

impl Parser for PlainText {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_base_token)
    }
}
