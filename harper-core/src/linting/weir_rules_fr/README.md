# French Weir rules

Weir rules in this folder run only on French documents, through
`LintGroup::new_curated_for_language(.., Language::French)`.

They work like the English rules in `../weir_rules`: a top-level `RuleName.weir` file is one
public rule, and a subfolder of `.weir` files is exposed as a single rule named after it.
