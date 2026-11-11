use std::{fs::File, path::Path};

use rs_conllu::{Sentence, TokenID, parse_file};

use crate::UPOS;

/// Produce an iterator over the sentences in a `.conllu` file.
/// Will panic on error, so this should not be used outside of training.
pub fn iter_sentences_in_conllu(path: impl AsRef<Path>) -> impl Iterator<Item = Sentence> {
    let file = File::open(path).unwrap();
    let doc = parse_file(file);

    doc.map(|v| v.unwrap())
}

/// The words of `sent` as they appear in the text, with their tags.
///
/// Treebanks list a contraction like French "du" both as written and as its parts
/// ("de" + "le"). Harper sees the written word, so the parts are dropped and the written
/// word takes the tag of its first part. Empty nodes, which never appear in the text,
/// are dropped too.
pub fn surface_words(sent: &Sentence) -> Vec<(String, Option<UPOS>)> {
    let tokens = &sent.tokens;
    let mut words = Vec::new();
    let mut i = 0;

    while let Some(token) = tokens.get(i) {
        match token.id {
            TokenID::Range(start, end) => {
                let first_tag = tokens.get(i + 1).and_then(|t| t.upos);
                words.push((token.form.clone(), first_tag.and_then(UPOS::from_conllu)));
                i += end - start + 2;
            }
            TokenID::Empty(..) => i += 1,
            TokenID::Single(_) => {
                words.push((token.form.clone(), token.upos.and_then(UPOS::from_conllu)));
                i += 1;
            }
        }
    }

    words
}

#[cfg(test)]
mod tests {
    use rs_conllu::parse_sentence;

    use super::surface_words;
    use crate::UPOS;

    fn words_of(conllu: &str) -> Vec<(String, Option<UPOS>)> {
        let sent = parse_sentence(conllu).unwrap();
        surface_words(&sent)
    }

    #[test]
    fn keeps_plain_words() {
        let words = words_of(
            "1\tIl\til\tPRON\t_\t_\t2\tnsubj\t_\t_\n2\tparle\tparler\tVERB\t_\t_\t0\troot\t_\t_\n\n",
        );
        assert_eq!(
            words,
            [
                ("Il".to_string(), Some(UPOS::PRON)),
                ("parle".to_string(), Some(UPOS::VERB))
            ]
        );
    }

    #[test]
    fn collapses_contractions() {
        let words = words_of(
            "1\tparle\tparler\tVERB\t_\t_\t0\troot\t_\t_\n2-3\tdu\t_\t_\t_\t_\t_\t_\t_\t_\n2\tde\tde\tADP\t_\t_\t4\tcase\t_\t_\n3\tle\tle\tDET\t_\t_\t4\tdet\t_\t_\n4\tvillage\tvillage\tNOUN\t_\t_\t1\tobl\t_\t_\n\n",
        );
        assert_eq!(
            words,
            [
                ("parle".to_string(), Some(UPOS::VERB)),
                ("du".to_string(), Some(UPOS::ADP)),
                ("village".to_string(), Some(UPOS::NOUN))
            ]
        );
    }

    #[test]
    fn drops_empty_nodes() {
        let words = words_of(
            "1\tIl\til\tPRON\t_\t_\t0\troot\t_\t_\n1.1\tfait\tfaire\tVERB\t_\t_\t_\t_\t0:root\t_\n\n",
        );
        assert_eq!(words, [("Il".to_string(), Some(UPOS::PRON))]);
    }
}
