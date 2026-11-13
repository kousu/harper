use crate::language_detection::guess_language;
use crate::lexers::{Lexer, PlainText};
use crate::{LanguageFamily, Lrc, Token};

/// The lexer for the prose in a document: the natural-language text that a file-format parser,
/// such as [`Markdown`](super::Markdown), finds between its markup. Usually that's a language's
/// [`lexer`](LanguageFamily::lexer), such as `PlainEnglish`.
///
/// It is a [`Lexer`] itself, so it can lex a plain-text document directly.
#[derive(Clone)]
pub enum ProseLexer {
    /// Guess the language from the whole document each time it's lexed (see
    /// [`guess_language`]), and use that language's lexer.
    Guess,
    /// Always use this lexer.
    Given(Lrc<dyn Lexer>),
}

impl ProseLexer {
    /// The lexer of `language`.
    pub fn for_language(language: LanguageFamily) -> Self {
        Self::Given(language.lexer())
    }

    /// The lexer to use for a document whose prose is `text`.
    ///
    /// For [`ProseLexer::Guess`], that's the lexer of the language [`guess_language`] picks.
    /// When it can't tell, the first language built into Harper is used (English, when it's
    /// built in), and with no language built in, [`PlainText`].
    pub fn for_text(&self, text: &[char]) -> Lrc<dyn Lexer> {
        match self {
            Self::Given(lexer) => lexer.clone(),
            Self::Guess => {
                let text: String = text.iter().collect();
                guessed_or_first_language(&text)
                    .map(LanguageFamily::lexer)
                    .unwrap_or_else(|| Lrc::new(PlainText))
            }
        }
    }

    /// The lexer to use for `source`, whose prose is what `masker` lets through. Only extracts
    /// that text when it has to guess.
    pub fn for_masked(&self, masker: &impl crate::Masker, source: &[char]) -> Lrc<dyn Lexer> {
        match self {
            Self::Given(lexer) => lexer.clone(),
            Self::Guess => self.for_text(&super::masked_text(masker, source)),
        }
    }
}

/// Lexes `source` as a whole plain-text document: [`ProseLexer::Guess`] guesses its language
/// from all of it.
impl Lexer for ProseLexer {
    fn lex(&self, source: &[char]) -> Vec<Token> {
        self.for_text(source).lex(source)
    }
}

crate::lexing::impl_parser_for_lexer!(ProseLexer);

fn guessed_or_first_language(text: &str) -> Option<LanguageFamily> {
    use strum::IntoEnumIterator;

    guess_language(text).or_else(|| LanguageFamily::iter().next())
}

impl From<LanguageFamily> for ProseLexer {
    fn from(language: LanguageFamily) -> Self {
        Self::for_language(language)
    }
}

impl From<Lrc<dyn Lexer>> for ProseLexer {
    fn from(lexer: Lrc<dyn Lexer>) -> Self {
        Self::Given(lexer)
    }
}
