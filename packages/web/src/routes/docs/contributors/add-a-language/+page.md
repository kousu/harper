---
title: Add a Language
description: Teach Harper a new natural language, from a Cargo feature and a dictionary to a lexer, spell checking and rules.
---

# Add a Language

Harper checks English by default, but it isn't tied to English.
Each language is a separate part of `harper-core`, behind its own Cargo feature, and plugs into the same places.
This page lists those places in the order that's easiest to work through.

Each step leaves Harper building and its tests passing, so it's a good place to commit.
Open a [draft pull request](https://elijahpotter.dev/articles/never_wait) early, and say which language you're adding: someone else may already be working on it.

In the examples below, the new language is Esperanto, with the code `eo`.
Use your language's [ISO 639-1 code](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes).

## How Harper Sees a Language

- [`Language`](https://docs.rs/harper-core/latest/harper_core/enum.Language.html) names a language and its dialect, such as `Language::English(EnglishDialect::British)`.
  [`LanguageFamily`](https://docs.rs/harper-core/latest/harper_core/enum.LanguageFamily.html) is the same without the dialect, for code that only cares about the language.
- Whoever calls `harper-core` usually says which language the text is in: the `--language` option of `harper-cli`, or the `language` setting of `harper-ls`.
  When they don't, `guess_language` picks one by checking the text against each built-in language's dictionary.
  A new language takes part in guessing as soon as it has a lexer and a dictionary.
- Each language is behind a Cargo feature named after its code.
  `harper-core` builds with any set of languages, including none, and the language-independent parts (file-format parsers, the shared lexer, dictionaries, Weir, the lint machinery) don't depend on any of them.

## 1. Add a Cargo Feature

In `harper-core/Cargo.toml`, add a feature for the language, off by default:

```toml title="harper-core/Cargo.toml"
[features]
# Experimental Esperanto support.
eo = []
```

Gate everything you add for the language with `#[cfg(feature = "eo")]`.
Add a matching feature to `harper-cli` and `harper-ls` that turns on `harper-core/eo`, so the front ends can offer the language.

## 2. Add the `Language` Variant

In `harper-core/src/lang.rs`:

- Add a dialect enum, even if the language has only one dialect, and a `Language::Esperanto(EsperantoDialect)` variant behind the feature.
- Teach `LanguageFamily::try_from_name` the language's names and codes (its English name, its own name, and its ISO 639 codes), and give the dialect a `try_from_name`.
  `Language::try_from_names` then works for it, and so do `harper-cli --language` and the `harper-ls` `language` setting.
- Add the language's names to `KNOWN_LANGUAGES`, the table of every language Harper knows, outside the feature gate.
  A build without the feature then says it has no support for the language, instead of not recognizing its name.

The compiler then points at every `match` on `Language` or `LanguageFamily` that needs an arm for the new language.
Those are the places the rest of this page goes through.

## 3. Add a Dictionary

Put the language's data in its own folder, `harper-core/src/lang/eo/`.
The language's code goes in the same folder, as the module `lang::eo`: declare it in `harper-core/src/lang.rs` with `#[cfg(feature = "eo")] mod eo;`.
The dictionary uses the same `dictionary.dict` and `annotations.json` format as English (see [the dictionary page](./dictionary)).
If you generate it from another source, such as Hunspell files, commit the script and say in a `README.md` where the source comes from and how to regenerate the dictionary.
Check the source's license, and state it in the `README.md`.

Then:

- Load the dictionary and return it from the language's arm in `FstDictionary::curated_for`.
- If some words belong to only some dialects, give the language its own dialect flags and a `WordDialects` variant, so they can't be confused with another language's.

## 4. Add a Lexer

A [lexer](https://docs.rs/harper-core/latest/harper_core/lexers/trait.Lexer.html) splits prose in one language into words, numbers, punctuation and whitespace.
It recognizes words, not structure: file-format parsers such as `Markdown` recognize the headings, lists and code, and hand the prose in between to the lexer, so every file format works for every language.
Built with `with_language`, they use the given language's lexer; built with `new()` or `Default`, they guess each document's language with `guess_language` and use that language's.

Start with [`PlainText`](https://docs.rs/harper-core/latest/harper_core/lexers/struct.PlainText.html), which uses only the tokens every Latin-script language shares, and return it from the language's arm in `LanguageFamily::lexer`.
If the language splits words differently, write its own lexer in `harper-core/src/lang/eo/lexing.rs`: a function that lexes one token, trying the language's own tokens first and then falling back to `lex_base_token`, and a `PlainEsperanto` type that implements `Lexer` with it, as `PlainEnglish` does.

## 5. Spell Check and the Lint Group

`LintGroup::new_curated_for_language` builds the rules for a language.
Give it an arm for the new language that adds a `SpellCheck` with the language's dictionary.

`SpellCheck` takes a `SpellingConventions` value for the parts of spell checking that differ between languages: words to ignore, suggestions to drop, and how to rank suggestions.
An implementation that does nothing is enough to start with.

At this point Harper can spell check the language.
Try it with `harper-cli lint --language eo <file>`.

## 6. Part-of-Speech Tags (Optional)

Many grammar rules need to know which words are nouns, verbs and so on.
Harper tags words with a [Brill tagger](./brill), and marks noun phrases with a chunker.
Both are trained per language on a [Universal Dependencies](https://universaldependencies.org/) treebank, with `harper-cli train-brill-tagger` and `harper-cli train-brill-chunker`.

Put the training script and the trained models in `harper-brill/src/lang/eo/`, behind an `eo` feature in `harper-brill`, and return them from the language's arm in `tag_and_chunk` in `harper-core/src/document.rs`.
Record the treebank's version and license.
Until then, the language's words have no tags, and rules that need tags won't fire.

## 7. Grammar Rules

Put the language's rules in `harper-core/src/lang/eo/linting/` and add them to its lint group.
[Author a Rule](./author-a-rule) explains how to write one; the steps are the same as for English.
Don't use the English-only helpers in `lang/en/expr/` and `lang/en/patterns/`.

## 8. Test Every Combination

A language must not break builds that leave it out, and other languages must not break it.
Run `harper-core`'s tests with:

- no language: `cargo test -p harper-core --no-default-features`;
- only the new language: `cargo test -p harper-core --no-default-features --features eo`;
- the default (English): `cargo test -p harper-core`;
- English and the new language: `cargo test -p harper-core --features eo`.

Put the language's tests in modules gated by its feature, and tests that need English in modules gated by `en` too.
