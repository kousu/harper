//! The regional dialects of French.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

/// A regional variant of French.
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
    Display,
    EnumString,
    EnumIter,
)]
pub enum FrenchDialect {
    #[default]
    France,
    Canada,
    Belgium,
    Switzerland,
}

impl FrenchDialect {
    /// Tries to get a dialect from how a person might write it, in any case: its country in
    /// English or French ("Canada", "Québec"), its country code ("CA") or its locale ("fr-CA",
    /// "fr_CA"). Returns `None` if the name is not recognized.
    #[must_use]
    pub fn try_from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "fr" | "france" | "fr-fr" | "fr_fr" => Some(Self::France),
            "ca" | "canada" | "quebec" | "québec" | "fr-ca" | "fr_ca" => Some(Self::Canada),
            "be" | "belgium" | "belgique" | "fr-be" | "fr_be" => Some(Self::Belgium),
            "ch" | "switzerland" | "suisse" | "fr-ch" | "fr_ch" => Some(Self::Switzerland),
            _ => None,
        }
    }
}

bitflags::bitflags! {
    /// A set of French dialects, as bit flags.
    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Hash, Default)]
    #[serde(transparent)]
    pub struct FrenchDialectFlags: u8 {
        const FRANCE = 1 << 0;
        const CANADA = 1 << 1;
        const BELGIUM = 1 << 2;
        const SWITZERLAND = 1 << 3;
    }
}

impl FrenchDialectFlags {
    /// The set with only `dialect` in it.
    #[must_use]
    pub fn from_dialect(dialect: FrenchDialect) -> Self {
        use FrenchDialect::*;

        match dialect {
            France => Self::FRANCE,
            Canada => Self::CANADA,
            Belgium => Self::BELGIUM,
            Switzerland => Self::SWITZERLAND,
        }
    }

    /// Whether `dialect` is in the set, or the set is empty (every dialect).
    #[must_use]
    pub fn allows(self, dialect: FrenchDialect) -> bool {
        self.is_empty() || self.intersects(Self::from_dialect(dialect))
    }
}
