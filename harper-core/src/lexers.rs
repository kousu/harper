//! Lexers split prose in one natural language into tokens: see [`Lexer`].

#[cfg(feature = "en")]
pub use crate::lang::en::lexing::PlainEnglish;
#[cfg(feature = "fr")]
pub use crate::lang::fr::lexing::PlainFrench;
pub use crate::lexing::{Lexer, PlainText};
