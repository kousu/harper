use super::Parser;
use crate::Token;
use crate::lexing::{lex_french_token, lex_with};

/// Like [`super::PlainEnglish`], but splits French elisions ("l'homme" → "l'" + "homme").
#[derive(Clone, Copy)]
pub struct PlainFrench;

impl Parser for PlainFrench {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_french_token)
    }
}
