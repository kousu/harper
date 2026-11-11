use super::{FoundToken, lex_english_token, lex_punctuation};
use crate::TokenKind;
use crate::char_ext::CharExt;

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
/// Identical to [`lex_english_token`], except that an elided word and its apostrophe
/// ("l'", "qu'", "jusqu'") becomes its own word.
pub fn lex_french_token(source: &[char]) -> FoundToken {
    lex_elision(source).unwrap_or_else(|| lex_english_token(source))
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
        && next.is_english_lingual()
        && ELIDED_WORDS.contains(&prefix.as_str())
        && !starts_with_kept_whole(source)
}

fn starts_with_kept_whole(source: &[char]) -> bool {
    KEPT_WHOLE.iter().any(|whole| {
        let len = whole.chars().count();
        let is_whole_word = source.get(len).is_none_or(|c| !c.is_english_lingual());

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
