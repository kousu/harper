#[cfg(feature = "en")]
pub mod en;
#[cfg(feature = "fr")]
pub(crate) mod fr;
mod language_data;

pub(crate) use language_data::LanguageData;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumString};

#[cfg(feature = "en")]
use crate::EnglishDialect;
#[cfg(feature = "fr")]
use crate::FrenchDialect;
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
    doc = "A natural language Harper can check, without its regional variant.",
    non_exhaustive
)]
#[non_exhaustive]
pub enum Language {
    #[cfg(feature = "en")]
    English(EnglishDialect),
    /// Experimental; needs the `fr` feature.
    #[cfg(feature = "fr")]
    French(FrenchDialect),
}

impl Language {
    /// The language without its regional variant.
    pub fn family(self) -> LanguageFamily {
        self.into()
    }

    /// Tries to get a language from how a person might write it, as in a command-line option or
    /// a setting: a language name or code (see [`LanguageFamily::try_from_name`]) and optionally
    /// a dialect (see [`EnglishDialect::try_from_name`]). Without a dialect, the language's
    /// default dialect is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use harper_core::Language;
    ///
    /// # #[cfg(feature = "en")] {
    /// use harper_core::EnglishDialect;
    ///
    /// assert_eq!(
    ///     Language::try_from_names("en", Some("uk")),
    ///     Ok(Language::English(EnglishDialect::British))
    /// );
    /// assert_eq!(
    ///     Language::try_from_names("English", None),
    ///     Ok(Language::English(EnglishDialect::American))
    /// );
    /// # }
    /// assert!(Language::try_from_names("xx", None).is_err());
    /// ```
    #[cfg_attr(not(any(feature = "en", feature = "fr")), allow(unused_variables))]
    pub fn try_from_names(
        language: &str,
        dialect: Option<&str>,
    ) -> Result<Self, LanguageNameError> {
        let family = LanguageFamily::try_from_name(language).ok_or_else(|| {
            let name = language.to_lowercase();
            match KNOWN_LANGUAGES
                .iter()
                .find(|(_, names)| names.contains(&name.as_str()))
            {
                Some((known, _)) => LanguageNameError::NotInThisBuild(known),
                None => LanguageNameError::UnknownLanguage(language.to_owned()),
            }
        })?;
        let unknown_dialect = |dialect: &str| LanguageNameError::UnknownDialect {
            family,
            dialect: dialect.to_owned(),
        };

        match family {
            #[cfg(feature = "en")]
            LanguageFamily::English => Ok(Self::English(match dialect {
                Some(d) => EnglishDialect::try_from_name(d).ok_or_else(|| unknown_dialect(d))?,
                None => EnglishDialect::American,
            })),
            #[cfg(feature = "fr")]
            LanguageFamily::French => Ok(Self::French(match dialect {
                Some(d) => FrenchDialect::try_from_name(d).ok_or_else(|| unknown_dialect(d))?,
                None => FrenchDialect::default(),
            })),
        }
    }
}

impl LanguageFamily {
    /// Tries to get a language from its name or ISO 639 code, in any case: "English", "en" or
    /// "eng"; with the `fr` feature, "French", "Français", "fr", "fra" or "fre". Returns `None` if
    /// the name is not recognized.
    #[must_use]
    pub fn try_from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            #[cfg(feature = "en")]
            n if ENGLISH_NAMES.contains(&n) => Some(Self::English),
            #[cfg(feature = "fr")]
            n if FRENCH_NAMES.contains(&n) => Some(Self::French),
            _ => None,
        }
    }
}

impl LanguageFamily {
    /// What Harper ships for this language. This is the only place that lists the languages a
    /// build includes: everything else asks the language for its data.
    pub(crate) fn data(self) -> &'static dyn LanguageData {
        match self {
            #[cfg(feature = "en")]
            Self::English => &en::English,
            #[cfg(feature = "fr")]
            Self::French => &fr::French,
        }
    }
}

/// How English can be named: its name and its ISO 639 codes.
const ENGLISH_NAMES: &[&str] = &["en", "eng", "english"];

/// How French can be named: its English and French names and its ISO 639 codes.
const FRENCH_NAMES: &[&str] = &["fr", "fra", "fre", "french", "français", "francais"];

/// Every language Harper knows, whether or not this build includes it, with the names it can be
/// written as (lowercase). A name found here but not in [`LanguageFamily::try_from_name`] belongs
/// to a language this build leaves out.
const KNOWN_LANGUAGES: &[(&str, &[&str])] = &[("English", ENGLISH_NAMES), ("French", FRENCH_NAMES)];

/// Why [`Language::try_from_names`] didn't recognize a language or dialect.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LanguageNameError {
    #[error("Unknown language: {0}")]
    UnknownLanguage(String),
    /// A language Harper knows, left out of this build by a Cargo feature.
    #[error("This build has no {0} support.")]
    NotInThisBuild(&'static str),
    #[error("Invalid dialect '{dialect}' for {family}")]
    UnknownDialect {
        family: LanguageFamily,
        dialect: String,
    },
}

impl LanguageFamily {
    /// The lexer for prose in this language, for file-format parsers to hand their prose to (see
    /// [`Markdown::with_language`](crate::parsers::Markdown::with_language)).
    pub fn lexer(self) -> Lrc<dyn Lexer> {
        self.data().lexer()
    }
}

#[cfg(all(test, feature = "en"))]
mod tests {
    use std::str::FromStr;

    use super::{Language, LanguageFamily, LanguageNameError};
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
    fn family_from_name_or_code() {
        for name in ["en", "EN", "eng", "English", "english"] {
            assert_eq!(
                LanguageFamily::try_from_name(name),
                Some(LanguageFamily::English),
                "{name}"
            );
        }
        assert_eq!(LanguageFamily::try_from_name("xx"), None);
        assert_eq!(LanguageFamily::try_from_name(""), None);
    }

    #[test]
    fn language_from_names() {
        assert_eq!(
            Language::try_from_names("en", Some("au")),
            Ok(Language::English(EnglishDialect::Australian))
        );
        assert_eq!(
            Language::try_from_names("ENGLISH", Some("en-IN")),
            Ok(Language::English(EnglishDialect::Indian))
        );
        assert_eq!(
            Language::try_from_names("en", None),
            Ok(Language::English(EnglishDialect::American))
        );
    }

    #[cfg(not(feature = "fr"))]
    #[test]
    fn french_needs_the_feature() {
        let error = Language::try_from_names("fr", None).unwrap_err();
        assert_eq!(error, LanguageNameError::NotInThisBuild("French"));
        assert_eq!(error.to_string(), "This build has no French support.");
    }

    #[test]
    fn unknown_names_are_errors() {
        assert_eq!(
            Language::try_from_names("xx", Some("us")),
            Err(LanguageNameError::UnknownLanguage("xx".to_owned()))
        );

        let error = Language::try_from_names("en", Some("xx")).unwrap_err();
        assert_eq!(
            error,
            LanguageNameError::UnknownDialect {
                family: LanguageFamily::English,
                dialect: "xx".to_owned()
            }
        );
        assert_eq!(error.to_string(), "Invalid dialect 'xx' for English");
    }

    #[test]
    fn dialect_from_name() {
        for (name, dialect) in [
            ("American", EnglishDialect::American),
            ("us", EnglishDialect::American),
            ("en_US", EnglishDialect::American),
            ("British", EnglishDialect::British),
            ("UK", EnglishDialect::British),
            ("gb", EnglishDialect::British),
            ("Australian", EnglishDialect::Australian),
            ("aus", EnglishDialect::Australian),
            ("Canadian", EnglishDialect::Canadian),
            ("en-ca", EnglishDialect::Canadian),
            ("Indian", EnglishDialect::Indian),
            ("bharat", EnglishDialect::Indian),
        ] {
            assert_eq!(EnglishDialect::try_from_name(name), Some(dialect), "{name}");
        }
        assert_eq!(EnglishDialect::try_from_name("xx"), None);
    }

    #[test]
    fn every_dialect_parses_from_its_own_name() {
        use strum::IntoEnumIterator;

        for dialect in EnglishDialect::iter() {
            assert_eq!(
                EnglishDialect::try_from_name(&dialect.to_string()),
                Some(dialect)
            );
        }
    }

    #[test]
    fn language_round_trips_through_json() {
        let language = Language::English(EnglishDialect::Canadian);
        let json = serde_json::to_string(&language).unwrap();

        assert_eq!(json, r#"{"English":"Canadian"}"#);
        assert_eq!(serde_json::from_str::<Language>(&json).unwrap(), language);
    }
}

#[cfg(all(test, not(feature = "en")))]
mod without_english_tests {
    use super::{Language, LanguageNameError};

    #[test]
    fn english_needs_the_feature() {
        let error = Language::try_from_names("en", None).unwrap_err();
        assert_eq!(error, LanguageNameError::NotInThisBuild("English"));
        assert_eq!(error.to_string(), "This build has no English support.");
    }
}

#[cfg(all(test, feature = "fr"))]
mod french_tests {
    use super::{FrenchDialect, Language, LanguageFamily, LanguageNameError};

    #[test]
    fn french_words_have_french_dialects() {
        use crate::DictWordMetadata;

        let quebecois = DictWordMetadata {
            dialects: Some(FrenchDialect::Canada.into()),
            ..Default::default()
        };
        assert!(quebecois.allows_language(Language::French(FrenchDialect::Canada)));
        assert!(!quebecois.allows_language(Language::French(FrenchDialect::France)));
    }

    /// English's fixups, such as joining "R&D" into one word, don't apply to French text.
    #[test]
    fn english_fixups_skip_french_text() {
        use crate::Document;

        let french = Document::new_with_language("R&D", LanguageFamily::French);
        assert_eq!(french.get_tokens().len(), 3);

        #[cfg(feature = "en")]
        {
            let english = Document::new_with_language("R&D", LanguageFamily::English);
            assert_eq!(english.get_tokens().len(), 1);
        }
    }

    /// User dictionaries tag their words with the language being checked, so a French user's
    /// words must count as French.
    #[test]
    fn a_language_tags_words_as_its_own() {
        use crate::WordDialects;

        let canada = Language::French(FrenchDialect::Canada);
        let dialects = WordDialects::from(canada);
        assert!(dialects.allows(canada));
        assert!(!dialects.allows(Language::French(FrenchDialect::France)));
        #[cfg(feature = "en")]
        assert!(!dialects.allows(Language::English(crate::EnglishDialect::American)));
    }

    #[cfg(feature = "en")]
    #[test]
    fn french_and_english_dialects_dont_mix() {
        use crate::{DictWordMetadata, EnglishDialect, WordDialects};

        let quebecois = DictWordMetadata {
            dialects: Some(FrenchDialect::Canada.into()),
            ..Default::default()
        };
        assert!(!quebecois.allows_language(Language::English(EnglishDialect::Canadian)));

        let british = WordDialects::from(EnglishDialect::British);
        assert!(!british.allows(Language::French(FrenchDialect::France)));
        assert_eq!(british.union(FrenchDialect::Belgium.into()), british);
    }

    #[test]
    fn french_dialects_read_from_annotation_strings() {
        use crate::{FrenchDialectFlags, WordDialects};

        let dialects: WordDialects = serde_json::from_str(r#""CANADA | BELGIUM""#).unwrap();
        assert_eq!(
            dialects,
            WordDialects::French(FrenchDialectFlags::CANADA | FrenchDialectFlags::BELGIUM)
        );
        assert_eq!(
            serde_json::to_string(&dialects).unwrap(),
            r#""CANADA | BELGIUM""#
        );
    }

    #[test]
    fn french_from_name_or_code() {
        for name in ["fr", "FR", "fra", "fre", "French", "français", "Francais"] {
            assert_eq!(
                LanguageFamily::try_from_name(name),
                Some(LanguageFamily::French),
                "{name}"
            );
        }
    }

    #[test]
    fn french_dialect_from_name() {
        for (name, dialect) in [
            ("France", FrenchDialect::France),
            ("fr_FR", FrenchDialect::France),
            ("ca", FrenchDialect::Canada),
            ("Québec", FrenchDialect::Canada),
            ("fr-CA", FrenchDialect::Canada),
            ("Belgique", FrenchDialect::Belgium),
            ("be", FrenchDialect::Belgium),
            ("Suisse", FrenchDialect::Switzerland),
            ("CH", FrenchDialect::Switzerland),
        ] {
            assert_eq!(FrenchDialect::try_from_name(name), Some(dialect), "{name}");
        }
        assert_eq!(FrenchDialect::try_from_name("uk"), None);
    }

    #[test]
    fn french_from_names() {
        assert_eq!(
            Language::try_from_names("fr", Some("ca")),
            Ok(Language::French(FrenchDialect::Canada))
        );
        assert_eq!(
            Language::try_from_names("French", None),
            Ok(Language::French(FrenchDialect::France))
        );
    }

    #[test]
    fn english_dialect_is_not_french() {
        assert_eq!(
            Language::try_from_names("fr", Some("British")),
            Err(LanguageNameError::UnknownDialect {
                family: LanguageFamily::French,
                dialect: "British".to_owned()
            })
        );
    }

    #[test]
    fn french_family() {
        assert_eq!(
            Language::French(FrenchDialect::Canada).family(),
            LanguageFamily::French
        );
    }

    #[test]
    fn french_defaults_to_france() {
        assert_eq!(FrenchDialect::default(), FrenchDialect::France);
    }

    #[test]
    fn french_round_trips_through_json() {
        let language = Language::French(FrenchDialect::Belgium);
        let json = serde_json::to_string(&language).unwrap();

        assert_eq!(json, r#"{"French":"Belgium"}"#);
        assert_eq!(serde_json::from_str::<Language>(&json).unwrap(), language);
    }
}

#[cfg(all(test, feature = "en"))]
mod document_tests {
    use super::LanguageFamily;
    use crate::Document;
    use crate::parsers::Markdown;

    #[test]
    fn english_document_matches_default() {
        let text = "# Title\n\nI can't see the *tree*.";
        let english = Document::new_with_parser_and_language(
            text,
            &Markdown::default(),
            LanguageFamily::English,
        );
        let default = Document::new_with_parser(text, &Markdown::default());

        assert_eq!(english.get_tokens(), default.get_tokens());
    }
}
