use crate::linting::{Lint, LintKind, Linter, Suggestion};
use crate::{
    Document, FrenchOrdinalSuffix, Number, OrdinalSuffix, Span, TokenKind, TokenStringExt,
};

/// Flags ordinal suffixes that don't fit their number, like "2er" for "2e" or "1e" for "1er".
#[derive(Debug, Clone, Copy, Default)]
pub struct CorrectNumberSuffix;

/// Whether `suffix` is plural ("ers", "es", "nds"…).
fn is_plural(suffix: FrenchOrdinalSuffix) -> bool {
    use FrenchOrdinalSuffix::*;

    matches!(suffix, Ers | Res | Es | Emes | Nds | Ndes)
}

/// Whether `suffix` fits `number`: "er"/"re" for 1, "nd"/"nde" for 2, "e"/"ème" for 2 and up.
fn fits(suffix: FrenchOrdinalSuffix, number: f64) -> bool {
    use FrenchOrdinalSuffix::*;

    match suffix {
        Er | Re | Ers | Res => number == 1.0,
        Nd | Nde | Nds | Ndes => number == 2.0,
        E | Es | Eme | Emes => number >= 2.0,
    }
}

/// The suffixes that fit `number`, keeping `suffix`'s number (singular or plural). For 1, both
/// genders, since the rule can't tell which is meant.
fn replacements(suffix: FrenchOrdinalSuffix, number: f64) -> Vec<FrenchOrdinalSuffix> {
    use FrenchOrdinalSuffix::*;

    match (number == 1.0, is_plural(suffix)) {
        (true, false) => vec![Er, Re],
        (true, true) => vec![Ers, Res],
        (false, false) => vec![E],
        (false, true) => vec![Es],
    }
}

impl Linter for CorrectNumberSuffix {
    fn lint(&mut self, document: &Document) -> Vec<Lint> {
        document
            .iter_numbers()
            .filter_map(|token| {
                let TokenKind::Number(Number {
                    value,
                    suffix: Some(OrdinalSuffix::French(suffix)),
                    ..
                }) = token.kind
                else {
                    return None;
                };

                let number = value.0;
                if number < 1.0 || number.fract() != 0.0 || fits(suffix, number) {
                    return None;
                }

                let len = suffix.to_chars().len();
                let span = Span::new(token.span.end - len, token.span.end);
                let suggestions = replacements(suffix, number)
                    .into_iter()
                    .map(|s| Suggestion::ReplaceWith(s.to_chars().to_vec()))
                    .collect();

                Some(Lint {
                    span,
                    lint_kind: LintKind::Miscellaneous,
                    suggestions,
                    message: "This ordinal suffix doesn't fit its number.".to_owned(),
                    priority: 31,
                })
            })
            .collect()
    }

    fn description(&self) -> &str {
        "French ordinals take a suffix that depends on the number: `1er` or `1re` for the first, \
        `2e` (or `2nd`/`2nde`) for the second, and `e` from then on (`3e`, `21e`). This rule \
        flags suffixes that don't fit, like `2er` or `1e`."
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{assert_allows, assert_fixes};
    use super::CorrectNumberSuffix;

    #[test]
    fn allows_correct_suffixes() {
        assert_allows(
            "Le 1er janvier, la 1re fois, les 1ers jours, la 2e place, la 2nde guerre, \
            le 3e jour, le 21e siècle, les 3es places, le 4ème étage.",
            CorrectNumberSuffix,
        );
    }

    #[test]
    fn fixes_premier_on_other_numbers() {
        assert_fixes("le 2er jour", CorrectNumberSuffix, "le 2e jour");
        assert_fixes("la 3re fois", CorrectNumberSuffix, "la 3e fois");
        assert_fixes("les 5ers jours", CorrectNumberSuffix, "les 5es jours");
    }

    #[test]
    fn fixes_e_on_one() {
        assert_fixes("le 1e jour", CorrectNumberSuffix, "le 1er jour");
        assert_fixes("le 1ème jour", CorrectNumberSuffix, "le 1er jour");
        assert_fixes("les 1es jours", CorrectNumberSuffix, "les 1ers jours");
    }

    #[test]
    fn fixes_second_on_other_numbers() {
        assert_fixes("le 3nd jour", CorrectNumberSuffix, "le 3e jour");
        assert_fixes("la 1nde fois", CorrectNumberSuffix, "la 1er fois");
    }

    #[test]
    fn ignores_plain_numbers() {
        assert_allows("Il a 3 chats et 2,5 kilos.", CorrectNumberSuffix);
    }
}
