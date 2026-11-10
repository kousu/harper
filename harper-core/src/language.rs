use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, VariantArray};

/// A natural language Harper can check.
///
/// Only English has rules, a dictionary and a tagger so far.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    EnumString,
    EnumIter,
    Display,
    VariantArray,
)]
pub enum Language {
    #[default]
    English,
    French,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Language;

    #[test]
    fn defaults_to_english() {
        assert_eq!(Language::default(), Language::English);
    }

    #[test]
    fn round_trips_through_strings() {
        assert_eq!(Language::from_str("French"), Ok(Language::French));
        assert_eq!(Language::French.to_string(), "French");
    }
}
