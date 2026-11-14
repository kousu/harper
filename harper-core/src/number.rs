use std::fmt::Display;

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[cfg(feature = "en")]
use crate::EnglishOrdinalSuffix;
#[cfg(feature = "fr")]
use crate::FrenchOrdinalSuffix;

/// Represents a written number.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd)]
pub struct Number {
    /// The actual value of the number
    pub value: OrderedFloat<f64>,
    /// Whether it contains a suffix (like the 1__st__ element).
    pub suffix: Option<OrdinalSuffix>,
    /// What base it is in (hex v.s. decimal, for example).
    pub radix: u32,
    /// The level of precision the number is formatted with.
    pub precision: usize,
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.radix == 16 {
            write!(f, "0x{:X}", self.value.0 as u64)?;
        } else {
            write!(f, "{:.*}", self.precision, self.value.0)?;
        }

        if let Some(suffix) = self.suffix {
            for c in suffix.to_chars() {
                write!(f, "{c}")?;
            }
        }

        Ok(())
    }
}

/// A number's ordinal suffix, as its language writes it, such as English's "st" in "1st".
///
/// It serializes as the language's suffix alone, like `"St"`.
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Hash, Eq)]
#[serde(untagged)]
#[non_exhaustive]
pub enum OrdinalSuffix {
    #[cfg(feature = "en")]
    English(EnglishOrdinalSuffix),
    #[cfg(feature = "fr")]
    French(FrenchOrdinalSuffix),
}

impl OrdinalSuffix {
    /// The suffix as written.
    pub const fn to_chars(self) -> &'static [char] {
        match self {
            #[cfg(feature = "en")]
            Self::English(suffix) => suffix.to_chars(),
            #[cfg(feature = "fr")]
            Self::French(suffix) => suffix.to_chars(),
        }
    }
}

/// The old English variants, for code written before suffixes had a language.
#[cfg(feature = "en")]
#[allow(non_upper_case_globals)]
impl OrdinalSuffix {
    #[deprecated(note = "use `OrdinalSuffix::English(EnglishOrdinalSuffix::Th)`")]
    pub const Th: Self = Self::English(EnglishOrdinalSuffix::Th);
    #[deprecated(note = "use `OrdinalSuffix::English(EnglishOrdinalSuffix::St)`")]
    pub const St: Self = Self::English(EnglishOrdinalSuffix::St);
    #[deprecated(note = "use `OrdinalSuffix::English(EnglishOrdinalSuffix::Nd)`")]
    pub const Nd: Self = Self::English(EnglishOrdinalSuffix::Nd);
    #[deprecated(note = "use `OrdinalSuffix::English(EnglishOrdinalSuffix::Rd)`")]
    pub const Rd: Self = Self::English(EnglishOrdinalSuffix::Rd);

    #[deprecated(note = "use `EnglishOrdinalSuffix::correct_suffix_for`")]
    pub fn correct_suffix_for(number: impl Into<f64>) -> Option<Self> {
        EnglishOrdinalSuffix::correct_suffix_for(number).map(Self::English)
    }

    #[deprecated(note = "use `EnglishOrdinalSuffix::from_chars`")]
    pub fn from_chars(chars: &[char]) -> Option<Self> {
        EnglishOrdinalSuffix::from_chars(chars).map(Self::English)
    }
}

#[cfg(feature = "en")]
impl From<EnglishOrdinalSuffix> for OrdinalSuffix {
    fn from(suffix: EnglishOrdinalSuffix) -> Self {
        Self::English(suffix)
    }
}

#[cfg(feature = "fr")]
impl From<FrenchOrdinalSuffix> for OrdinalSuffix {
    fn from(suffix: FrenchOrdinalSuffix) -> Self {
        Self::French(suffix)
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    #[cfg(feature = "en")]
    use crate::EnglishOrdinalSuffix;

    use super::Number;

    #[test]
    fn hex_fifteen() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.0),
                suffix: None,
                radix: 16,
                precision: 0
            }
            .to_string(),
            "0xF"
        )
    }

    #[test]
    fn decimal_fifteen() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.0),
                suffix: None,
                radix: 10,
                precision: 0
            }
            .to_string(),
            "15"
        )
    }

    #[cfg(feature = "en")]
    #[test]
    fn decimal_fifteen_suffix() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.0),
                suffix: Some(EnglishOrdinalSuffix::Th.into()),
                radix: 10,
                precision: 0
            }
            .to_string(),
            "15th"
        )
    }

    #[cfg(feature = "en")]
    /// Suffixes serialize as they did before they had a language.
    #[test]
    fn suffix_serializes_as_before() {
        let suffix: crate::OrdinalSuffix = EnglishOrdinalSuffix::Th.into();
        assert_eq!(serde_json::to_string(&suffix).unwrap(), "\"Th\"");
        assert_eq!(
            serde_json::from_str::<crate::OrdinalSuffix>("\"Th\"").unwrap(),
            suffix
        );
    }

    #[test]
    fn decimal_fifteen_and_a_half() {
        assert_eq!(
            Number {
                value: OrderedFloat(15.5),
                suffix: None,
                radix: 10,
                precision: 2
            }
            .to_string(),
            "15.50"
        )
    }
}
