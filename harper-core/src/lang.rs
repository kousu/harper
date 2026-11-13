pub mod en;
mod language_data;

pub(crate) use language_data::LanguageData;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumString};

#[cfg(feature = "en")]
use crate::EnglishDialect;

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

    /// Tries to get a language from how a person might write it, as in a command-line option or
    /// a setting: a language name or code (see [`LanguageFamily::try_from_name`]) and optionally
    /// a dialect (see [`EnglishDialect::try_from_name`]). Without a dialect, the language's
    /// default dialect is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use harper_core::{EnglishDialect, Language};
    ///
    /// assert_eq!(
    ///     Language::try_from_names("en", Some("uk")),
    ///     Ok(Language::English(EnglishDialect::British))
    /// );
    /// assert_eq!(
    ///     Language::try_from_names("English", None),
    ///     Ok(Language::English(EnglishDialect::American))
    /// );
    /// assert!(Language::try_from_names("xx", None).is_err());
    /// ```
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
        }
    }
}

impl LanguageFamily {
    /// Tries to get a language from its name or ISO 639 code, in any case: "English", "en" or
    /// "eng". Returns `None` if the name is not recognized.
    #[must_use]
    pub fn try_from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            #[cfg(feature = "en")]
            n if ENGLISH_NAMES.contains(&n) => Some(Self::English),
            _ => None,
        }
    }
}

impl LanguageFamily {
    /// What Harper ships for this language. This is the only place that lists the languages a
    /// build includes: everything else asks the language for its data.
    pub(crate) fn data(self) -> &'static dyn LanguageData {
        match self {
            Self::English => &en::English,
        }
    }
}

/// How English can be named: its name and its ISO 639 codes.
const ENGLISH_NAMES: &[&str] = &["en", "eng", "english"];

/// Every language Harper knows, whether or not this build includes it, with the names it can be
/// written as (lowercase). A name found here but not in [`LanguageFamily::try_from_name`] belongs
/// to a language this build leaves out.
const KNOWN_LANGUAGES: &[(&str, &[&str])] = &[("English", ENGLISH_NAMES)];

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

#[cfg(not(feature = "en"))]
compile_error!("harper-core needs at least one language: enable the `en` feature.");

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
