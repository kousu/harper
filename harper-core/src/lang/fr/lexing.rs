use super::ordinal::FrenchOrdinalSuffix;
use crate::char_ext::CharExt;
use crate::lexing::{FoundToken, Lexer, lex_base_token, lex_hex_number, lex_punctuation, lex_with};
use crate::{Number, Token, TokenKind};

/// Elided words that French splits off the following word, as in "l'homme" → "l'" + "homme".
///
/// Taken from the Universal Dependencies French treebank, which treats these as separate words.
const ELIDED_WORDS: &[&str] = &[
    "l", "d", "s", "qu", "n", "c", "jusqu", "j", "m", "lorsqu", "puisqu", "t", "quoiqu",
];

/// Fixed expressions that start like an elision but stay a single word.
const KEPT_WHOLE: &[&str] = &["l'on"];

/// Lex one token of French text.
///
/// The shared tokens (see [`lex_base_token`]), except that:
/// - an elided word and its apostrophe ("l'", "qu'", "jusqu'") becomes its own word, as in the
///   Universal Dependencies French treebank;
/// - numbers are written the French way: "3,14", and ordinals like "1er" and "2e".
pub fn lex_french_token(source: &[char]) -> FoundToken {
    lex_elision(source)
        .or_else(|| lex_french_number(source))
        .unwrap_or_else(|| lex_base_token(source))
}

/// Lex a number the way French writes it: digits with an optional decimal comma ("3,14"), or a
/// whole number with an ordinal suffix ("1er", "2e", "2nde").
fn lex_french_number(source: &[char]) -> Option<FoundToken> {
    // Hexadecimal numbers are the shared lexer's; this would match only their "0".
    if lex_hex_number(source).is_some() {
        return None;
    }

    let whole = source.iter().take_while(|c| c.is_ascii_digit()).count();
    if whole == 0 {
        return None;
    }

    let fraction = match source.get(whole) {
        Some(',') => source[whole + 1..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .count(),
        _ => 0,
    };
    let len = if fraction > 0 {
        whole + 1 + fraction
    } else {
        whole
    };

    let digits: String = source[..len]
        .iter()
        .map(|&c| if c == ',' { '.' } else { c })
        .collect();
    let value: f64 = digits.parse().ok()?;

    let suffix = (fraction == 0)
        .then(|| FrenchOrdinalSuffix::lex(&source[len..]))
        .flatten();

    Some(FoundToken {
        token: TokenKind::Number(Number {
            value: value.into(),
            suffix: suffix.map(|(suffix, _)| suffix.into()),
            radix: 10,
            precision: fraction,
        }),
        next_index: len + suffix.map_or(0, |(_, suffix_len)| suffix_len),
    })
}

fn lex_elision(source: &[char]) -> Option<FoundToken> {
    let len = ELIDED_WORDS
        .iter()
        .map(|word| word.chars().count())
        .find(|&len| is_elision_of_length(source, len))?;

    Some(FoundToken {
        next_index: len + 1,
        token: TokenKind::Word(None),
    })
}

/// Whether `source` starts with an elided word of `len` letters, an apostrophe and then a letter.
fn is_elision_of_length(source: &[char], len: usize) -> bool {
    let Some(&[.., apostrophe, next]) = source.get(..len + 2) else {
        return false;
    };

    let prefix: String = source[..len]
        .iter()
        .flat_map(|c| c.to_lowercase())
        .collect();
    let is_apostrophe = lex_punctuation(&[apostrophe]).is_some_and(|t| t.token.is_apostrophe());

    is_apostrophe
        && next.is_latin_letter()
        && ELIDED_WORDS.contains(&prefix.as_str())
        && !starts_with_kept_whole(source)
}

fn starts_with_kept_whole(source: &[char]) -> bool {
    KEPT_WHOLE.iter().any(|whole| {
        let len = whole.chars().count();
        let is_whole_word = source.get(len).is_none_or(|c| !c.is_latin_letter());

        is_whole_word
            && source.get(..len).is_some_and(|start| {
                start
                    .iter()
                    .map(|c| c.normalized().to_lowercase().to_string())
                    .collect::<String>()
                    == *whole
            })
    })
}

/// A lexer for French prose. It is like [`PlainEnglish`](crate::lexers::PlainEnglish), except
/// that elided words ("l'", "qu'", "jusqu'") are words of their own and numbers are written the
/// French way (see `lex_french_token`).
///
/// To check French in other file formats, build their parsers with
/// [`LanguageFamily::French`](crate::LanguageFamily::French), as in
/// [`Markdown::with_language`](crate::parsers::Markdown::with_language). An elision split by
/// markup, as in Markdown's `l’*école*`, isn't found, because those parsers hand the lexer each
/// piece of text separately.
#[derive(Clone, Copy)]
pub struct PlainFrench;

impl Lexer for PlainFrench {
    fn lex(&self, source: &[char]) -> Vec<Token> {
        lex_with(source, lex_french_token)
    }
}

crate::lexing::impl_parser_for_lexer!(PlainFrench);

#[cfg(test)]
mod tests {
    use super::lex_french_token;
    use crate::lexing::lex_with;

    fn words(text: &str) -> Vec<String> {
        let source: Vec<char> = text.chars().collect();
        lex_with(&source, lex_french_token)
            .into_iter()
            .filter(|t| t.kind.is_word())
            .map(|t| t.span.get_content_string(&source))
            .collect()
    }

    fn numbers(text: &str) -> Vec<crate::Number> {
        let source: Vec<char> = text.chars().collect();
        lex_with(&source, lex_french_token)
            .into_iter()
            .filter_map(|t| t.kind.as_number().copied())
            .collect()
    }

    #[test]
    fn lexes_decimal_commas() {
        let numbers = numbers("3,14 et 1,5");
        assert_eq!(numbers.len(), 2);
        assert_eq!(numbers[0].value.0, 3.14);
        assert_eq!(numbers[0].precision, 2);
        assert_eq!(numbers[1].value.0, 1.5);
    }

    #[test]
    fn lexes_ordinals_as_one_number() {
        use crate::{FrenchOrdinalSuffix as S, OrdinalSuffix};

        let suffixes: Vec<_> = numbers("le 1er, la 1re, la 2e, le 3ème, la 2nde")
            .into_iter()
            .map(|n| n.suffix)
            .collect();
        assert_eq!(
            suffixes,
            [S::Er, S::Re, S::E, S::Eme, S::Nde].map(|s| Some(OrdinalSuffix::French(s)))
        );
        assert!(words("le 1er janvier").iter().all(|w| w != "er"));
    }

    #[test]
    fn keeps_numbers_and_words_apart() {
        assert_eq!(numbers("10 euros")[0].suffix, None);
        assert_eq!(words("10 euros"), ["euros"]);
        assert_eq!(numbers("0x1F")[0].radix, 16);
    }

    #[test]
    fn splits_common_elisions() {
        assert_eq!(words("l'homme"), ["l'", "homme"]);
        assert_eq!(words("d'accord"), ["d'", "accord"]);
        assert_eq!(words("qu'il"), ["qu'", "il"]);
        assert_eq!(words("j'ai"), ["j'", "ai"]);
        assert_eq!(words("n'est"), ["n'", "est"]);
        assert_eq!(words("c'est"), ["c'", "est"]);
        assert_eq!(words("s'il"), ["s'", "il"]);
        assert_eq!(words("m'appelle"), ["m'", "appelle"]);
        assert_eq!(words("t'aime"), ["t'", "aime"]);
    }

    #[test]
    fn splits_long_elisions() {
        assert_eq!(words("jusqu'à"), ["jusqu'", "à"]);
        assert_eq!(words("lorsqu'elle"), ["lorsqu'", "elle"]);
        assert_eq!(words("puisqu'on"), ["puisqu'", "on"]);
        assert_eq!(words("quoiqu'il"), ["quoiqu'", "il"]);
    }

    #[test]
    fn splits_with_curly_apostrophe() {
        assert_eq!(words("l’homme"), ["l’", "homme"]);
    }

    #[test]
    fn splits_capitalized() {
        assert_eq!(words("L'homme"), ["L'", "homme"]);
        assert_eq!(words("Qu'est-ce"), ["Qu'", "est", "ce"]);
    }

    #[test]
    fn splits_before_accented_letter() {
        assert_eq!(words("l'école"), ["l'", "école"]);
    }

    #[test]
    fn keeps_fixed_words_whole() {
        assert_eq!(words("aujourd'hui"), ["aujourd'hui"]);
        assert_eq!(words("quelqu'un"), ["quelqu'un"]);
        assert_eq!(words("presqu'île"), ["presqu'île"]);
        assert_eq!(words("O'Neal"), ["O'Neal"]);
    }

    #[test]
    fn keeps_l_on_whole() {
        assert_eq!(words("si l'on veut"), ["si", "l'on", "veut"]);
        assert_eq!(words("l'oncle"), ["l'", "oncle"]);
    }

    #[test]
    fn leaves_lone_apostrophe_alone() {
        assert_eq!(words("l' homme"), ["l", "homme"]);
    }

    #[test]
    fn lexes_a_sentence() {
        assert_eq!(
            words("Il parle de l'homme du village aujourd'hui."),
            [
                "Il",
                "parle",
                "de",
                "l'",
                "homme",
                "du",
                "village",
                "aujourd'hui"
            ]
        );
    }

    #[test]
    fn keeps_word_inside_longer_word() {
        assert_eq!(words("bel'homme"), ["bel'homme"]);
    }
}
