#[cfg(feature = "en")]
pub mod en;
mod language_data;

pub(crate) use language_data::LanguageData;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumString};

#[cfg(feature = "en")]
use crate::EnglishDialect;
use crate::Lrc;
use crate::lexers::Lexer;

/// A natural language Harper can check, together with its regional variant.
///
/// Code that only cares about the language, not the region, can use [`LanguageFamily`],
/// which has the same variants without the dialect: `language.family()`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumDiscriminants)]
#[strum_discriminants(
    name(LanguageFamily),
    derive(Display, EnumString, EnumIter, Hash, Serialize, Deserialize),
    doc = "A natural language Harper can check, without its regional variant."
)]
pub enum Language {
    #[cfg(feature = "en")]
    English(EnglishDialect),
}

impl Language {
    /// The language without its regional variant.
    pub fn family(self) -> LanguageFamily {
        self.into()
    }
}

impl LanguageFamily {
    /// What Harper ships for this language. This is the only place that lists the languages a
    /// build includes: everything else asks the language for its data.
    pub(crate) fn data(self) -> &'static dyn LanguageData {
        match self {
            #[cfg(feature = "en")]
            Self::English => &en::English,
        }
    }
}

impl LanguageFamily {
    /// The lexer for prose in this language, for file-format parsers to hand their prose to (see
    /// [`Markdown::with_lexer`](crate::parsers::Markdown::with_lexer)).
    pub fn lexer(self) -> Lrc<dyn Lexer> {
        self.data().lexer()
    }
}

#[cfg(all(test, feature = "en"))]
mod tests {
    use std::str::FromStr;

    use super::{Language, LanguageFamily};
    use crate::EnglishDialect;

    #[test]
    fn family_drops_the_dialect() {
        assert_eq!(
            Language::English(EnglishDialect::British).family(),
            LanguageFamily::English
        );
        assert_eq!(
            Language::English(EnglishDialect::American).family(),
            Language::English(EnglishDialect::Indian).family()
        );
    }

    #[test]
    fn family_round_trips_through_strings() {
        assert_eq!(
            LanguageFamily::from_str("English"),
            Ok(LanguageFamily::English)
        );
        assert_eq!(LanguageFamily::English.to_string(), "English");
    }

    #[test]
    fn language_round_trips_through_json() {
        let language = Language::English(EnglishDialect::Canadian);
        let json = serde_json::to_string(&language).unwrap();

        assert_eq!(json, r#"{"English":"Canadian"}"#);
        assert_eq!(serde_json::from_str::<Language>(&json).unwrap(), language);
    }
}
