//! English's ordinal suffixes: the "st" in "1st".

use is_macro::Is;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// An English ordinal suffix: "th", "st", "nd" or "rd".
#[derive(
    Debug, Serialize, Deserialize, Default, PartialEq, PartialOrd, Clone, Copy, Is, Hash, Eq,
)]
pub enum EnglishOrdinalSuffix {
    #[default]
    Th,
    St,
    Nd,
    Rd,
}

impl EnglishOrdinalSuffix {
    pub fn correct_suffix_for(number: impl Into<f64>) -> Option<Self> {
        let number = number.into();

        if number < 0.0 || number - number.floor() > f64::EPSILON || number > u64::MAX as f64 {
            return None;
        }

        let integer = number as u64;

        if let 11..=13 = integer % 100 {
            return Some(Self::Th);
        };

        Some(match integer % 10 {
            0 | 4..=9 => Self::Th,
            1 => Self::St,
            2 => Self::Nd,
            3 => Self::Rd,
            _ => unreachable!(),
        })
    }

    pub const fn to_chars(self) -> &'static [char] {
        match self {
            Self::Th => &['t', 'h'],
            Self::St => &['s', 't'],
            Self::Nd => &['n', 'd'],
            Self::Rd => &['r', 'd'],
        }
    }

    /// Check the characters in a buffer to see if it matches a number suffix.
    pub fn from_chars(chars: &[char]) -> Option<Self> {
        let lower_chars: [char; 2] = chars.iter().map(char::to_ascii_lowercase).collect_array()?;

        match lower_chars {
            ['t', 'h'] => Some(Self::Th),
            ['s', 't'] => Some(Self::St),
            ['n', 'd'] => Some(Self::Nd),
            ['r', 'd'] => Some(Self::Rd),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::EnglishOrdinalSuffix;

    #[test]
    fn issue_1051() {
        let word = "story".chars().collect_vec();
        assert_eq!(None, EnglishOrdinalSuffix::from_chars(&word));
    }
}
