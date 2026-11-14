//! French's ordinal suffixes: the "er" in "1er" and the "e" in "2e".

use serde::{Deserialize, Serialize};

/// A French ordinal suffix, as in "1er", "1re", "2e" or "2nd".
///
/// It serializes in lower case (`"er"`), so it doesn't read back as an English suffix.
#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone, Copy, Hash, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FrenchOrdinalSuffix {
    /// "1er": premier.
    Er,
    /// "1re": première.
    Re,
    /// "1ers": premiers.
    Ers,
    /// "1res": premières.
    Res,
    /// "2e": deuxième, troisième, ….
    E,
    /// "2es": deuxièmes, ….
    Es,
    /// "2ème": a common, though not recommended, spelling of "2e".
    Eme,
    /// "2èmes".
    Emes,
    /// "2nd": second.
    Nd,
    /// "2nde": seconde.
    Nde,
    /// "2nds": seconds.
    Nds,
    /// "2ndes": secondes.
    Ndes,
}

impl FrenchOrdinalSuffix {
    /// Every suffix, longest first, so that "ers" is tried before "er".
    const LONGEST_FIRST: [Self; 12] = [
        Self::Ndes,
        Self::Emes,
        Self::Ers,
        Self::Res,
        Self::Eme,
        Self::Nds,
        Self::Nde,
        Self::Er,
        Self::Re,
        Self::Es,
        Self::Nd,
        Self::E,
    ];

    /// The suffix as written.
    pub const fn to_chars(self) -> &'static [char] {
        match self {
            Self::Er => &['e', 'r'],
            Self::Re => &['r', 'e'],
            Self::Ers => &['e', 'r', 's'],
            Self::Res => &['r', 'e', 's'],
            Self::E => &['e'],
            Self::Es => &['e', 's'],
            Self::Eme => &['è', 'm', 'e'],
            Self::Emes => &['è', 'm', 'e', 's'],
            Self::Nd => &['n', 'd'],
            Self::Nde => &['n', 'd', 'e'],
            Self::Nds => &['n', 'd', 's'],
            Self::Ndes => &['n', 'd', 'e', 's'],
        }
    }

    /// The suffix at the start of `source`, if it's a whole suffix (not followed by a letter),
    /// and its length.
    pub fn lex(source: &[char]) -> Option<(Self, usize)> {
        Self::LONGEST_FIRST.into_iter().find_map(|suffix| {
            let chars = suffix.to_chars();
            let matches = source.get(..chars.len()).is_some_and(|start| {
                start
                    .iter()
                    .zip(chars)
                    .all(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
            });
            let ends = source.get(chars.len()).is_none_or(|c| !c.is_alphanumeric());

            (matches && ends).then_some((suffix, chars.len()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FrenchOrdinalSuffix;

    fn lex(text: &str) -> Option<(FrenchOrdinalSuffix, usize)> {
        FrenchOrdinalSuffix::lex(&text.chars().collect::<Vec<_>>())
    }

    #[test]
    fn lexes_the_longest_suffix() {
        assert_eq!(lex("ers "), Some((FrenchOrdinalSuffix::Ers, 3)));
        assert_eq!(lex("er "), Some((FrenchOrdinalSuffix::Er, 2)));
        assert_eq!(lex("ème"), Some((FrenchOrdinalSuffix::Eme, 3)));
        assert_eq!(lex("nde,"), Some((FrenchOrdinalSuffix::Nde, 3)));
        assert_eq!(lex("e"), Some((FrenchOrdinalSuffix::E, 1)));
    }

    #[test]
    fn needs_a_whole_suffix() {
        assert_eq!(lex("erreur"), None);
        assert_eq!(lex("euros"), None);
    }
}
