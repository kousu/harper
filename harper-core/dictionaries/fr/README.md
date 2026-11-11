# French dictionary

Loaded by `FstDictionary::curated_for(Language::French)` when the `french` feature is enabled.

## Source

`dictionary.dict` is generated from the
[Grammalecte](https://grammalecte.net/) French lexicon (`lexique-grammalecte-fr-v7.7`),
licensed under the [Mozilla Public License 2.0](https://www.mozilla.org/MPL/2.0/).
The generated file is a derived work and carries the same license.

## Rebuilding

Run `build.sh`. It downloads the lexicon, checks its SHA-256, and runs `lexicon_to_dict.awk`
to write `dictionary.dict`. `build.sh -h` lists its options.

## What is included

- Every inflected form in the lexicon (about 470,000), except the "X" sub-dictionary.
  Both traditional and 1990-reform spellings are accepted.
- Part of speech, noun number, pronoun person and number, and gender (`♂`, `♀`; both for
  words like `élève`).
- For verbs: infinitive and participles, tense and mood (`π` present, `ι` imperfect,
  `σ` simple past, `φ` future, `κ` conditional, `ψ` present subjunctive,
  `ξ` imperfect subjunctive, `ε` imperative), and person (`¹ ² ³` singular, `₁ ₂ ₃` plural).
  A form's tenses and persons are merged, so `mange` is "present indicative, present
  subjunctive or imperative" and "first, second or third person singular".
- Forms in Grammalecte frequency class 6 or above are marked common, which ranks them
  higher among spelling suggestions.
- Curly apostrophes are stored as straight ones (`l'`, `aujourd'hui`); Harper treats them the same.

Flags are defined in `annotations.json`, using the same letters and meanings as the English
`annotations.json`. Affixes are not used: every form is listed in full.

When a form has several parts of speech, its flags are merged. For example, `la` is
marked both masculine (the musical note) and feminine (the article).
