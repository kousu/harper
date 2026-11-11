use super::{follows_hyphen, is_name_after, next_word};
use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{Document, Span};

/// Words that always elide before a vowel: "le arbre" → "l'arbre".
const ALWAYS_ELIDED: &[&str] = &[
    "le", "la", "de", "je", "me", "te", "se", "ne", "que", "jusque", "lorsque", "puisque",
];

/// Words that elide only before particular words.
const ELIDED_BEFORE: &[(&str, &[&str])] = &[
    // "ce" elides before "être" and "en"; otherwise it becomes "cet" ("cet arbre").
    ("ce", &["est", "était", "étaient", "eut", "eût", "en"]),
    ("si", &["il", "ils"]),
];

/// Words starting with a vowel that still don't trigger elision.
const NO_ELISION_BEFORE: &[&str] = &[
    "onze",
    "onzième",
    "onzièmes",
    "oui",
    "ouistiti",
    "ouistitis",
];

/// Forms like "la une" (a newspaper's front page) that are correct as written.
const KEPT_AS_IS: &[(&str, &str)] = &[("la", "une")];

/// Flags elided words written in full before a vowel, like "le arbre" for "l'arbre".
#[derive(Default)]
pub struct Elision;

impl Linter for Elision {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        let tokens = document.get_tokens();
        let source = document.get_source();

        tokens
            .iter()
            .enumerate()
            .filter(|(i, token)| token.kind.is_word() && !follows_hyphen(tokens, *i))
            .filter_map(|(i, token)| {
                let next = next_word(tokens, i)?;
                // Names and words the dictionary doesn't know are often foreign.
                let is_known = next.kind.as_word().is_some_and(Option::is_some);
                if is_name_after(token.get_ch(source), next.get_ch(source)) || !is_known {
                    return None;
                }

                let word = lowercase(token.get_ch(source));
                let next_word = lowercase(next.get_ch(source));

                should_elide(&word, &next_word).then(|| {
                    // Cover the word and the space after it.
                    let span = Span::new(token.span.start, next.span.start);
                    let written = token.get_ch(source);
                    let elided: Vec<char> = written[..written.len() - 1]
                        .iter()
                        .copied()
                        .chain(['\''])
                        .collect();

                    Lint {
                        span,
                        lint_kind: LintKind::Grammar,
                        suggestions: vec![Suggestion::ReplaceWith(elided.clone())],
                        message: format!(
                            "Before a vowel, write `{}`.",
                            elided.iter().collect::<String>()
                        ),
                        priority: 31,
                    }
                })
            })
            .collect()
    }

    fn description(&self) -> &str {
        "In French, short words like `le`, `de`, `je` and `que` drop their last vowel before a \
        word starting with a vowel: `l'arbre`, `d'eau`, `j'ai`, `qu'il`. Words starting with `h` \
        are left alone, since some do and some don't, and so are names and foreign words."
    }
}

fn lowercase(chars: &[char]) -> String {
    chars.iter().flat_map(|c| c.to_lowercase()).collect()
}

fn should_elide(word: &str, next: &str) -> bool {
    let elides_here = ALWAYS_ELIDED.contains(&word)
        && starts_with_vowel(next)
        && !NO_ELISION_BEFORE.contains(&next)
        && !KEPT_AS_IS.contains(&(word, next));

    let elides_before_next = ELIDED_BEFORE
        .iter()
        .any(|(w, nexts)| *w == word && nexts.contains(&next));

    elides_here || elides_before_next
}

/// Whether `word` starts with a vowel sound that triggers elision.
///
/// "h" is left out because only some words starting with it elide ("l'homme", "le héros").
/// "y" only counts as the pronoun ("j'y vais"), not in words like "yaourt".
fn starts_with_vowel(word: &str) -> bool {
    word == "y"
        || word
            .chars()
            .next()
            .is_some_and(|c| "aeiouâàäéèêëîïôöûùüæœ".contains(c))
}

#[cfg(all(test, feature = "french"))]
mod tests {
    use super::Elision;
    use crate::linting::fr::tests::{assert_allows, assert_fixes};

    #[test]
    fn elides_articles() {
        assert_fixes("Je vois le arbre.", Elision, "Je vois l'arbre.");
        assert_fixes("Elle boit la eau.", Elision, "Elle boit l'eau.");
    }

    #[test]
    fn elides_de() {
        assert_fixes("Un verre de eau.", Elision, "Un verre d'eau.");
    }

    #[test]
    fn elides_pronouns() {
        assert_fixes("Je ai faim.", Elision, "J'ai faim.");
        assert_fixes("Il se appelle Paul.", Elision, "Il s'appelle Paul.");
        assert_fixes("Tu me as vu.", Elision, "Tu m'as vu.");
        assert_fixes("Je te aime.", Elision, "Je t'aime.");
    }

    #[test]
    fn elides_ne() {
        assert_fixes("Ce ne est pas grave.", Elision, "Ce n'est pas grave.");
    }

    #[test]
    fn elides_que_and_compounds() {
        assert_fixes("Je pense que il vient.", Elision, "Je pense qu'il vient.");
        assert_fixes(
            "Attends jusque à demain.",
            Elision,
            "Attends jusqu'à demain.",
        );
        assert_fixes("Lorsque elle arrive.", Elision, "Lorsqu'elle arrive.");
        assert_fixes("Puisque on part.", Elision, "Puisqu'on part.");
    }

    #[test]
    fn elides_ce_before_etre() {
        assert_fixes("Ce est vrai.", Elision, "C'est vrai.");
        assert_fixes("Ce était beau.", Elision, "C'était beau.");
    }

    #[test]
    fn keeps_ce_as_determiner() {
        // This needs "cet", which is a different mistake.
        assert_allows("Ce arbre est grand.", Elision);
    }

    #[test]
    fn elides_si_before_il() {
        assert_fixes("Si il pleut, reste.", Elision, "S'il pleut, reste.");
        assert_fixes("Si ils viennent.", Elision, "S'ils viennent.");
        assert_allows("Si elle vient.", Elision);
    }

    #[test]
    fn elides_before_accented_vowel() {
        assert_fixes("Je aime le été.", Elision, "J'aime l'été.");
        assert_fixes("Il parle de économie.", Elision, "Il parle d'économie.");
    }

    #[test]
    fn elides_before_pronoun_y() {
        assert_fixes("Je y vais.", Elision, "J'y vais.");
        assert_fixes("Il ne y a rien.", Elision, "Il n'y a rien.");
    }

    #[test]
    fn keeps_capital_letter() {
        assert_fixes("Le avion part.", Elision, "L'avion part.");
        assert_fixes("JE AI FAIM.", Elision, "J'AI FAIM.");
    }

    #[test]
    fn allows_correct_elision() {
        assert_allows(
            "J'ai vu l'arbre et l'eau d'ici, jusqu'à ce qu'il parte.",
            Elision,
        );
    }

    #[test]
    fn allows_consonants() {
        assert_allows("Le chat de la voisine que je connais.", Elision);
    }

    #[test]
    fn allows_words_starting_with_h() {
        assert_allows("Le héros mange le haricot.", Elision);
        assert_allows("Le homme.", Elision);
    }

    #[test]
    fn allows_words_starting_with_y() {
        assert_allows("Je mange le yaourt.", Elision);
    }

    #[test]
    fn allows_onze_and_oui() {
        assert_allows("Le onze novembre, le oui a gagné.", Elision);
    }

    #[test]
    fn allows_la_une() {
        assert_allows("Il fait la une des journaux.", Elision);
    }

    #[test]
    fn allows_acronyms_and_roman_numerals() {
        assert_allows("Sous la IV et la V République.", Elision);
        assert_allows("Le format de AVI.", Elision);
    }

    #[test]
    fn allows_names_and_foreign_words() {
        assert_allows("Les travaux de Emmanuel Régis.", Elision);
        assert_allows("Le magazine le Eton Candle.", Elision);
        assert_allows("Un plat de ovocné knedliky.", Elision);
    }

    #[test]
    fn allows_after_hyphen() {
        assert_allows("Ai-je eu raison ?", Elision);
        assert_allows("Fais-le entrer.", Elision);
    }

    #[test]
    fn allows_across_punctuation() {
        assert_allows("Je pense que, à mon avis, il vient.", Elision);
    }

    #[test]
    fn flags_each_mistake() {
        assert_fixes(
            "Je ai bu le eau de une source.",
            Elision,
            "J'ai bu l'eau d'une source.",
        );
    }
}
