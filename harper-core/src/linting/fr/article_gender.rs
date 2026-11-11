use harper_brill::UPOS;

use super::{is_name_after, next_word, precedes_hyphen};
use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{DictWordMetadata, Document, GenderFlags, Token};

/// Articles as (masculine, feminine) pairs.
const ARTICLES: &[(&str, &str)] = &[("le", "la"), ("un", "une")];

/// Flags an article whose gender doesn't match its noun, like "la chat" for "le chat".
#[derive(Default)]
pub struct ArticleGender;

impl Linter for ArticleGender {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let tokens = document.get_tokens();
        let source = document.get_source();

        tokens
            .iter()
            .enumerate()
            .filter(|(_, token)| token.kind.is_word())
            .filter_map(|(i, article)| {
                let noun = next_word(tokens, i)?;
                // In a compound like "méta-analyse", the first part's gender doesn't count.
                if is_name_after(article.get_ch(source), noun.get_ch(source))
                    || precedes_hyphen(tokens, i + 2)
                {
                    return None;
                }

                let written = lowercase(article.get_ch(source));
                let gender = noun_gender(noun)?;

                // "le" and "la" are also object pronouns ("je le visite"), so skip words that
                // can be verbs. "un" and "une" never are.
                let is_definite = written == "le" || written == "la";
                if is_definite && can_be_verb(noun) {
                    return None;
                }
                let correct = correct_article(&written, gender)?;

                // "le"/"la" become "l'" before a vowel or "h"; that's for the elision rule.
                let noun_start = noun.get_ch(source).first()?.to_lowercase().next()?;
                if is_definite && "aeiouhâàäéèêëîïôöûùüæœ".contains(noun_start) {
                    return None;
                }

                Some(Lint {
                    span: article.span,
                    lint_kind: LintKind::Agreement,
                    suggestions: vec![Suggestion::replace_with_match_case_str(
                        correct,
                        article.get_ch(source),
                    )],
                    message: format!(
                        "`{}` is {}, so use `{correct}`.",
                        noun.get_ch(source).iter().collect::<String>(),
                        if gender == GenderFlags::MASCULINE {
                            "masculine"
                        } else {
                            "feminine"
                        },
                    ),
                    priority: 31,
                })
            })
            .collect()
    }

    fn description(&self) -> &str {
        "In French, `le`/`la` and `un`/`une` take the gender of their noun: `le chat`, \
        `la maison`, `un livre`, `une table`."
    }
}

fn lowercase(chars: &[char]) -> String {
    chars.iter().flat_map(|c| c.to_lowercase()).collect()
}

/// The gender of `token` if the tagger reads it as a noun and the dictionary gives it exactly
/// one gender. Nouns that take both, like "élève" or "livre", give `None`.
fn noun_gender(token: &Token) -> Option<GenderFlags> {
    let metadata: &DictWordMetadata = token.kind.as_word()?.as_ref()?;

    let is_noun = metadata.noun.is_some() && metadata.pos_tag == Some(UPOS::NOUN);
    let gender = metadata.gender;
    let is_single_gender = gender == GenderFlags::MASCULINE || gender == GenderFlags::FEMININE;

    (is_noun && is_single_gender).then_some(gender)
}

fn can_be_verb(token: &Token) -> bool {
    token
        .kind
        .as_word()
        .and_then(Option::as_ref)
        .is_some_and(|metadata| metadata.verb.is_some())
}

/// The article to use instead of `written` before a noun of `gender`, if `written` is wrong.
fn correct_article(written: &str, gender: GenderFlags) -> Option<&'static str> {
    ARTICLES
        .iter()
        .find_map(|(masculine, feminine)| match (written, gender) {
            (w, GenderFlags::FEMININE) if w == *masculine => Some(*feminine),
            (w, GenderFlags::MASCULINE) if w == *feminine => Some(*masculine),
            _ => None,
        })
}

#[cfg(all(test, feature = "french"))]
mod tests {
    use super::ArticleGender;
    use crate::linting::fr::tests::{assert_allows, assert_fixes};

    #[test]
    fn fixes_feminine_article_on_masculine_noun() {
        assert_fixes("La chat dort.", ArticleGender, "Le chat dort.");
    }

    #[test]
    fn fixes_masculine_article_on_feminine_noun() {
        assert_fixes("Je vois le maison.", ArticleGender, "Je vois la maison.");
    }

    #[test]
    fn fixes_indefinite_articles() {
        assert_fixes(
            "Il a une livre de cuisine et un table.",
            ArticleGender,
            "Il a une livre de cuisine et une table.",
        );
        assert_fixes(
            "Elle lit une journal.",
            ArticleGender,
            "Elle lit un journal.",
        );
    }

    #[test]
    fn keeps_capitalization() {
        assert_fixes("LE MAISON", ArticleGender, "LA MAISON");
        assert_fixes("Une chien aboie.", ArticleGender, "Un chien aboie.");
    }

    #[test]
    fn fixes_several_in_one_sentence() {
        assert_fixes(
            "La chat et le maison sont dans une jardin.",
            ArticleGender,
            "Le chat et la maison sont dans un jardin.",
        );
    }

    #[test]
    fn allows_correct_articles() {
        assert_allows(
            "Le chat et la souris jouent dans un jardin avec une balle.",
            ArticleGender,
        );
    }

    #[test]
    fn allows_nouns_with_both_genders() {
        assert_allows(
            "Le livre et la livre, un élève et une élève.",
            ArticleGender,
        );
    }

    #[test]
    fn allows_object_pronouns() {
        // "le" and "la" are pronouns here, and "porte" and "garde" are verbs.
        assert_allows("Je le porte et elle la garde.", ArticleGender);
        assert_allows("Le géologue le visite en 1934.", ArticleGender);
        assert_allows("Aggregor le capture.", ArticleGender);
    }

    #[test]
    fn checks_un_une_before_noun_verb_homographs() {
        assert_fixes("Il fait un visite.", ArticleGender, "Il fait une visite.");
    }

    #[test]
    fn allows_plural_articles() {
        assert_allows("Les chats et des maisons.", ArticleGender);
    }

    #[test]
    fn leaves_vowels_to_elision() {
        assert_allows("La arbre et le eau.", ArticleGender);
    }

    #[test]
    fn checks_un_une_before_vowels() {
        assert_fixes("Une arbre pousse.", ArticleGender, "Un arbre pousse.");
        assert_allows("Une idée et un avion.", ArticleGender);
    }

    #[test]
    fn allows_unknown_words() {
        assert_allows("Le blorptastique.", ArticleGender);
    }

    #[test]
    fn allows_article_before_adjective() {
        // The article agrees with "maison", not with what directly follows it.
        assert_allows("La grande maison et le petit chat.", ArticleGender);
    }

    #[test]
    fn allows_article_and_noun_split_by_punctuation() {
        assert_allows("Il voit le, maison ou pas.", ArticleGender);
    }

    #[test]
    fn allows_names_acronyms_and_codes() {
        assert_allows("Il respecte la GPL et conduit la M3.", ArticleGender);
        assert_allows("Il visite le Louvre et la Bretagne.", ArticleGender);
    }

    #[test]
    fn allows_hyphenated_compounds() {
        assert_allows("Une méta-analyse indique un risque.", ArticleGender);
        assert_allows("Il y a une auto-école et un porte-monnaie.", ArticleGender);
    }

    #[test]
    fn checks_noun_before_hyphenated_pronoun() {
        // The hyphen must come right after the noun to count.
        assert_fixes("Une chien, mi-loup.", ArticleGender, "Un chien, mi-loup.");
    }

    #[test]
    fn allows_numeral_un() {
        assert_allows("Il est une heure et un quart.", ArticleGender);
    }
}
