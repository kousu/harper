use crate::lexing::{FoundToken, Lexer, lex_base_token, lex_hex_number, lex_with};
use crate::{Number, Token, TokenKind};

/// Lex one token of English prose: the shared tokens (see [`lex_base_token`]), plus decimal
/// numbers like "3.14" and plural numbers like "1990s" and "4s".
pub fn lex_english_token(source: &[char]) -> FoundToken {
    lex_english_extras(source).unwrap_or_else(|| lex_base_token(source))
}

/// The tokens only English has: plural decades like "1990s", plural digits like "4s", and
/// numbers with a decimal point like "3.14".
pub(crate) fn lex_english_extras(source: &[char]) -> Option<FoundToken> {
    // Before the shared number lexers, which would match only the digits.
    lex_plural_digit(source)
        .or_else(|| lex_long_decade(source))
        .or_else(|| lex_number(source))
}
pub(super) fn lex_long_decade(source: &[char]) -> Option<FoundToken> {
    // lex 4-digit decades in their plural such as: 1980s 1990s 2000s 2020s
    if source.len() < 5 {
        return None;
    }
    if source[0] != '1' && source[0] != '2' {
        return None;
    }
    if !source[1].is_ascii_digit() {
        return None;
    }
    if !source[2].is_ascii_digit() {
        return None;
    }
    if source[3] != '0' {
        return None;
    }
    if source[4] != 's' {
        return None;
    }

    Some(FoundToken {
        token: TokenKind::Decade,
        next_index: 5,
    })
}
pub(super) fn lex_plural_digit(src: &[char]) -> Option<FoundToken> {
    // Issue #774
    let l = src.len();
    let mut i = 0;

    if src.is_empty() || !src[i].is_alphanumeric() {
        return None;
    }
    i += 1;

    if l > i && src[i] == '\'' {
        i += 1;
    }

    if l > i && src[i] == 's' {
        i += 1;

        if l == i || !src[i].is_alphanumeric() {
            return Some(FoundToken {
                token: TokenKind::Word(None),
                next_index: i,
            });
        }
    }
    None
}
/// Lex a number the way English writes it: digits with an optional decimal point, as in "3.14".
pub(crate) fn lex_number(source: &[char]) -> Option<FoundToken> {
    if source.is_empty() {
        return None;
    }

    // Hexadecimal numbers are the shared lexer's; this would match only their "0".
    if lex_hex_number(source).is_some() {
        return None;
    }

    if !source[0].is_numeric() {
        return None;
    }

    let end = source
        .iter()
        .enumerate()
        .rev()
        .find_map(|(i, v)| v.is_ascii_digit().then_some(i))?;

    let mut s: String = source[0..end + 1].iter().collect();

    // Find the longest possible valid number
    while !s.is_empty() {
        if let Ok(n) = s.parse::<f64>() {
            let precision = s.chars().rev().position(|c| c == '.').unwrap_or_default();

            if !s.ends_with('.') {
                return Some(FoundToken {
                    token: TokenKind::Number(Number {
                        value: n.into(),
                        suffix: None,
                        radix: 10,
                        precision,
                    }),
                    next_index: s.len(),
                });
            }
        }

        s.pop();
    }

    None
}

/// A lexer for English prose (see `lex_english_token`).
#[derive(Clone, Copy)]
pub struct PlainEnglish;

impl Lexer for PlainEnglish {
    fn lex(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_english_token)
    }
}

crate::lexing::impl_parser_for_lexer!(PlainEnglish);

#[cfg(test)]
mod tests {
    use super::lex_number;
    use super::{lex_english_token, lex_long_decade, lex_plural_digit};
    use crate::TokenKind;
    use crate::char_string::char_string;
    use crate::lexing::FoundToken;
    use crate::lexing::lex_word;

    // test various kinds of number
    #[test]
    fn lexes_0() {
        let source: Vec<_> = "0".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_0_point_0() {
        let source: Vec<_> = "0.0".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_00() {
        let source: Vec<_> = "00".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[ignore = "Negative numbers are not yet supported"]
    fn lexes_negative_1() {
        let source: Vec<_> = "-1".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[ignore = "Positive numbers with a leading + are not supported"]
    fn lexes_positive_1() {
        let source: Vec<_> = "+1".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_pi() {
        let source: Vec<_> = "3.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_speed_of_light() {
        let source: Vec<_> = "3.00e8".chars().collect();
        assert!(matches!(
            lex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn doesnt_lex_cjk_numeral() {
        let source: Vec<_> = "二".chars().collect();
        assert!(lex_number(&source).is_none());
    }

    #[test]
    fn doesnt_lex_thai_digit() {
        let source: Vec<_> = "๑".chars().collect();
        assert!(lex_number(&source).is_none());
    }

    #[test]
    fn lexes_cjk_as_unlintable() {
        let source: Vec<_> = "世".chars().collect();
        assert!(lex_word(&source).is_none());
    }

    #[test]
    fn lexes_0s() {
        let source: Vec<_> = "0s".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_1_apostrophe_s() {
        let source: Vec<_> = "1's".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_0s_and_1s() {
        let source: Vec<_> = "0s and 1s".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_1s_and_0s_apostrophes() {
        let source: Vec<_> = "1's and 0's".chars().collect();
        assert!(matches!(
            // lex_token(&source),
            lex_plural_digit(&source),
            Some(FoundToken {
                token: TokenKind::Word(_),
                ..
            })
        ));
    }

    #[test]
    fn doesnt_lex_0s_joined_letter() {
        let source: Vec<_> = "0ss".chars().collect();
        assert!(lex_plural_digit(&source).is_none());
    }

    #[test]
    fn doesnt_lex_1s_apostrophe_joined_number() {
        let source: Vec<_> = "1's1".chars().collect();
        assert!(lex_plural_digit(&source).is_none());
    }

    #[test]
    fn lexes_20c_decade() {
        let source: Vec<_> = "1980s".chars().collect();
        assert!(matches!(
            lex_long_decade(&source),
            Some(FoundToken {
                token: TokenKind::Decade,
                ..
            })
        ));
    }

    #[test]
    fn lexes_21c_decade() {
        let source: Vec<_> = "2020s".chars().collect();
        assert!(matches!(
            lex_long_decade(&source),
            Some(FoundToken {
                token: TokenKind::Decade,
                ..
            })
        ));
    }

    #[test]
    fn lexes_ancient_decade() {
        let source: Vec<_> = "1010s".chars().collect();
        assert!(matches!(
            lex_long_decade(&source),
            Some(FoundToken {
                token: TokenKind::Decade,
                ..
            })
        ));
    }

    #[test]
    fn lexes_word_before_decade() {
        let source: Vec<_> = "late 1980s".chars().collect();
        assert!(matches!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Word(_),
                ..
            }
        ));
    }

    #[test]
    fn lexes_word_after_decade() {
        let source: Vec<_> = "1980s and".chars().collect();
        assert!(matches!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Decade,
                ..
            }
        ));
    }

    #[test]
    fn doesnt_lex_far_future_decade() {
        let source: Vec<_> = "3190s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_too_ancient_decade() {
        let source: Vec<_> = "100s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_0_prefixed_decade() {
        let source: Vec<_> = "0100s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_uppercase_decade() {
        let source: Vec<_> = "2000S".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_overlong_decade() {
        let source: Vec<_> = "20000s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_apostrophe_long_decade() {
        let source: Vec<_> = "2020's".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_bad_apostrophe_short_decade() {
        let source: Vec<_> = "80's".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn doesnt_lex_good_apostrophe_short_decade() {
        let source: Vec<_> = "'90s".chars().collect();
        assert!(lex_long_decade(&source).is_none());
    }

    #[test]
    fn accepts_sentence_with_decade() {
        let sentence: Vec<_> = "To the early 1990s there were a lot of Movies where the bad guys were former Russian intelligence agents.".chars().collect();
        let expected_tokens = [
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Decade,
        ];

        let mut next_index = 0;

        for expected_token in expected_tokens.iter() {
            if next_index >= sentence.len() {
                break; // Exit if we've processed the entire source
            }

            let token = lex_english_token(&sentence[next_index..]);
            assert_eq!(token.token, *expected_token);
            next_index += token.next_index;
        }
    }

    #[test]
    fn rejects_sentence_with_number() {
        let sentence: Vec<_> = "To the early 1990s there were a lot of Movies where the bad guys were former Russian intelligence agents.".chars().collect();
        let expected_tokens = [
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Word(None),
            TokenKind::Space(1),
            TokenKind::Number(Default::default()),
        ];

        let mut next_index = 0;

        for (i, expected_token) in expected_tokens.iter().enumerate() {
            if next_index >= sentence.len() {
                break; // Exit if we've processed the entire source
            }

            let token = lex_english_token(&sentence[next_index..]);

            if i < 6 {
                assert_eq!(token.token, *expected_token);
            } else {
                assert_ne!(token.token, *expected_token);
            }

            next_index += token.next_index;
        }
    }

    #[test]
    fn issue_1010() {
        let source = char_string!("3.");

        let tok = lex_number(&source).unwrap();
        assert_eq!(tok.next_index, 1);
    }

    #[test]
    fn lexes_full_number() {
        let source = char_string!("3.0");

        let tok = lex_number(&source).unwrap();
        assert_eq!(tok.next_index, 3);
    }
}
