mod email_address;
use crate::lang::en::lexing as english;
mod hostname;
mod url;

use hostname::lex_hostname_token;
use ordered_float::OrderedFloat;
use url::lex_url;

pub use english::lex_english_token;

use self::email_address::lex_email_address;
use crate::char_ext::CharExt;
use crate::punctuation::{Punctuation, Quote};
use crate::{Number, Span, Token, TokenKind};

#[derive(Debug, Eq, PartialEq)]
pub struct FoundToken {
    /// The index of the character __after__ the lexed token
    pub next_index: usize,
    /// Token lexed
    pub token: TokenKind,
}

/// Lex `source` with the provided `lex_fn`.
///
/// `lex_fn` should be a function that takes a subslice of the source, and returns the first found
/// token.
pub fn lex_with(source: &[char], lex_fn: fn(&[char]) -> FoundToken) -> Vec<Token> {
    let mut cursor = 0;
    let mut tokens = Vec::new();

    loop {
        if cursor >= source.len() {
            return tokens;
        }

        let FoundToken { token, next_index } = lex_fn(&source[cursor..]);

        tokens.push(Token {
            span: Span::new(cursor, cursor + next_index),
            kind: token,
        });
        cursor += next_index;
    }
}

/// Lex one token of Latin-script prose: the kinds of token every language shares. Each language's
/// lexer tries its own kinds first, then falls back to this one.
pub fn lex_base_token(source: &[char]) -> FoundToken {
    lex_regexish(source).unwrap_or_else(|| lex_shared_token(source))
}

/// The shared tokens other than regex-like brackets.
fn lex_shared_token(source: &[char]) -> FoundToken {
    [
        lex_punctuation,
        lex_tabs,
        lex_spaces,
        lex_newlines,
        lex_hex_number, // Before lex_number, which would match the initial 0
        lex_number,
        lex_url,
        lex_email_address,
        lex_hostname_token,
        lex_word,
    ]
    .into_iter()
    .find_map(|lexer| lexer(source))
    .unwrap_or_else(lex_catch)
}

/// Lex one token of a Weir rule. It lexes "1990s" and "4s" as English does, so rules match
/// English text.
pub fn lex_weir_token(source: &[char]) -> FoundToken {
    english::lex_english_extras(source).unwrap_or_else(|| lex_shared_token(source))
}

pub(crate) fn lex_word(source: &[char]) -> Option<FoundToken> {
    let is_apostrophe = |c: char| lex_punctuation(&[c]).is_some_and(|t| t.token.is_apostrophe());

    let mut end = source
        .iter()
        .position(|c| !c.is_latin_letter() && !c.is_ascii_digit() && !is_apostrophe(*c))
        .unwrap_or(source.len());

    while end >= 1 && is_apostrophe(source[end - 1]) {
        end -= 1;
    }

    if end == 0 {
        None
    } else {
        Some(FoundToken {
            next_index: end,
            token: TokenKind::Word(None),
        })
    }
}

pub(crate) fn lex_number(source: &[char]) -> Option<FoundToken> {
    if source.is_empty() {
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

// Often in comments we mention partial- or pseudo- regexes. Here's an example from Ghidra:
// ([a-z0-9]+ only) - We previously flagged just the z0 in the middle of it.
fn lex_regexish(src: &[char]) -> Option<FoundToken> {
    let l = src.len();
    let mut i = 0;

    if i >= l || src[i] != '[' {
        return None;
    }
    i += 1;

    loop {
        if i >= l || !src[i].is_alphanumeric() {
            return None;
        }
        i += 1;
        if i < l && src[i] == '-' {
            i += 1;
            if i >= l || !src[i].is_alphanumeric() {
                return None;
            }
            i += 1;
        }

        if i >= l || src[i] != ']' {
            continue;
        }
        break;
    }

    Some(FoundToken {
        token: TokenKind::Regexish,
        next_index: i + 1,
    })
}

fn lex_hex_number(source: &[char]) -> Option<FoundToken> {
    // < 3 to avoid accepting 0x alone
    if source.len() < 3 || source[0] != '0' || source[1] != 'x' || !source[2].is_ascii_hexdigit() {
        return None;
    }

    let mut i = 2;
    let len = source.len();

    while i < len {
        let next = source[i];

        if !next.is_ascii_hexdigit() {
            if !next.is_alphanumeric() {
                break;
            } else {
                return None;
            }
        }

        i += 1;
    }

    let s: String = source[2..i].iter().collect();

    // Should always succeed unless the logic above is broken
    if let Ok(n) = u64::from_str_radix(&s, 16) {
        return Some(FoundToken {
            token: TokenKind::Number(Number {
                value: OrderedFloat(n as f64),
                suffix: None,
                radix: 16,
                precision: 0,
            }),
            next_index: s.len() + 2,
        });
    }

    None
}

fn lex_newlines(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == '\n').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Newline(count),
            next_index: count,
        })
    } else {
        None
    }
}

fn lex_tabs(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == '\t').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Space(count * 2),
            next_index: count,
        })
    } else {
        None
    }
}

fn lex_spaces(source: &[char]) -> Option<FoundToken> {
    let count = source.iter().take_while(|c| **c == ' ').count();

    if count > 0 {
        Some(FoundToken {
            token: TokenKind::Space(count),
            next_index: count,
        })
    } else {
        None
    }
}

fn lex_punctuation(source: &[char]) -> Option<FoundToken> {
    if let Some(found) = lex_quote(source) {
        return Some(found);
    }

    let c = source.first()?;
    let punct = Punctuation::from_char(*c)?;

    Some(FoundToken {
        next_index: 1,
        token: TokenKind::Punctuation(punct),
    })
}

/// Specifically deals only with double quotes: ASCII and smart/typographic
fn lex_quote(source: &[char]) -> Option<FoundToken> {
    let c = *source.first()?;

    if c == '\"' || c == '“' || c == '”' {
        Some(FoundToken {
            next_index: 1,
            token: TokenKind::Punctuation(Punctuation::Quote(Quote { twin_loc: None })),
        })
    } else {
        None
    }
}

/// Covers cases not covered by the other lints.
fn lex_catch() -> FoundToken {
    FoundToken {
        next_index: 1,
        token: TokenKind::Unlintable,
    }
}

#[cfg(test)]
mod tests {
    use super::lex_english_token;
    use super::lex_hex_number;
    use super::{FoundToken, TokenKind};
    use crate::Punctuation;

    #[test]
    fn lexes_youtube_as_hostname() {
        let source: Vec<_> = "youtube.com".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Hostname,
                next_index: source.len()
            }
        );
    }

    #[test]
    fn doesnt_lex_regex_mini_range() {
        let source: Vec<_> = "[]".chars().collect();
        assert!(!matches!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 2
            }
        ))
    }

    #[test]
    fn lexes_regex_one_letter() {
        let source: Vec<_> = "[a]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 3
            }
        );
    }

    #[test]
    fn lexes_regex_two_letters() {
        let source: Vec<_> = "[az]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 4
            }
        );
    }

    #[test]
    fn lexes_regex_digits() {
        let source: Vec<_> = "[123]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 5
            }
        );
    }

    #[test]
    fn lexes_regex_two_alphanumeric() {
        let source: Vec<_> = "[a0b1c2]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 8
            }
        );
    }

    #[test]
    fn lexes_regex_one_range() {
        let source: Vec<_> = "[a-z]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 5
            }
        );
    }

    #[test]
    fn lexes_regex_letter_plus_range() {
        let source: Vec<_> = "[ax-z]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 6
            }
        );
    }

    #[test]
    fn lexes_regex_range_plus_letter() {
        let source: Vec<_> = "[a-cz]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 6
            }
        );
    }

    #[test]
    fn lexes_regex_two_ranges() {
        let source: Vec<_> = "[a-cx-z]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                next_index: 8
            }
        );
    }

    #[test]
    fn doesnt_lex_regex_broken_two_ranges() {
        // You can't end a range and start a range with a single letter
        let source: Vec<_> = "[a-x-z]".chars().collect();
        assert_eq!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Punctuation(Punctuation::OpenSquare),
                next_index: 1
            }
        );
    }

    #[test]
    fn doesnt_lex_regex_hyphen_at_start() {
        let source: Vec<_> = "[a-]".chars().collect();
        assert!(!matches!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                ..
            }
        ));
    }

    #[test]
    fn doesnt_lex_regex_hyphen_at_end() {
        let source: Vec<_> = "[-z]".chars().collect();
        assert!(!matches!(
            lex_english_token(&source),
            FoundToken {
                token: TokenKind::Regexish,
                ..
            }
        ));
    }

    #[test]
    fn lexes_good_hex_numeric() {
        let source: Vec<_> = "0x0".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_lowercase() {
        let source: Vec<_> = "0xa".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_uppercase() {
        let source: Vec<_> = "0xF".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_mixed_case() {
        let source: Vec<_> = "0xaF".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_lowercase_long() {
        let source: Vec<_> = "0x0123456789abcdef".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn lexes_good_hex_uppercase_long() {
        let source: Vec<_> = "0x0123456789ABCDEF".chars().collect();
        assert!(matches!(
            lex_hex_number(&source),
            Some(FoundToken {
                token: TokenKind::Number(_),
                ..
            })
        ));
    }

    #[test]
    fn does_not_lex_prefix_only() {
        let source: Vec<_> = "0x".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }

    #[test]
    fn does_not_lex_bad_alphabetic() {
        let source: Vec<_> = "0xg".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }

    #[test]
    fn does_not_lex_bad_after_good() {
        let source: Vec<_> = "0x123g".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }

    #[test]
    fn does_not_lex_uppercase_prefix() {
        let source: Vec<_> = "0Xf00d".chars().collect();
        assert!(lex_hex_number(&source).is_none());
    }
}
