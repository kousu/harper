---
title: Harper's Architecture
description: Explore the Harper architecture and find the components that parse text, check grammar, and support integrations.
---

# Harper's Architecture

This document seeks to solve one simple problem:

> "Roughly, it takes 2x more time to write a patch if you are unfamiliar with the project, but it takes 10x more time to figure out **where** you should change the code." - [Alex Kladov](https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html)

This document is meant to serve as a kind of table of contents for the Harper project.
Hopefully, we can reduce that 10x down to something a little more reasonable.

## What does Harper do?

Harper tries to do one thing well: find grammatical and spelling errors in text.
If possible, provide suggestions to correct those errors.
An error and its possible corrections together form what we call a lint.

In this vein, Harper serves the role of a [Linter](<https://en.wikipedia.org/wiki/Lint_(software)>) for natural language.
English is the default, and the language with by far the most rules.
Other languages can be added alongside it (see [Add a Language](./add-a-language)).

## `harper-core`

`harper-core` is where all the magic happens.
It contains the code needed to tokenize, parse, analyze and lint text.

At a high level, there are just a couple types you need to worry about.

- [Language](https://docs.rs/harper-core/latest/harper_core/enum.Language.html): A natural language and its dialect, such as `Language::English(EnglishDialect::British)`. [`LanguageFamily`](https://docs.rs/harper-core/latest/harper_core/enum.LanguageFamily.html) is the same without the dialect. Callers usually say which language the text is in; when they don't, Harper guesses it with [`guess_language`](https://docs.rs/harper-core/latest/harper_core/language_detection/fn.guess_language.html).
- [Document](https://docs.rs/harper-core/latest/harper_core/struct.Document.html): A representation of a document in one language, built with [`Document::new_with_language`](https://docs.rs/harper-core/latest/harper_core/struct.Document.html#method.new_with_language) or one of its sibling constructors, which guess the language when it isn't given. Implements [`TokenStringExt`](https://docs.rs/harper-core/latest/harper_core/trait.TokenStringExt.html) to make it easier to query.
- [Lexer](https://docs.rs/harper-core/latest/harper_core/lexers/trait.Lexer.html): A trait for splitting prose in one natural language into [Tokens](https://docs.rs/harper-core/latest/harper_core/struct.Token.html): words, numbers, punctuation and whitespace. It recognizes words, not structure. Examples are `PlainEnglish`, and `PlainText` for the tokens every Latin-script language shares. [`LanguageFamily::lexer`](https://docs.rs/harper-core/latest/harper_core/enum.LanguageFamily.html#method.lexer) returns the one for a language. Every lexer is also a parser that reads all of its input as prose.
- [Parser](https://docs.rs/harper-core/latest/harper_core/parsers/trait.Parser.html): A trait that describes an object that consumes text and emits tokens. _File-format parsers_ recognize the structure of a format, such as Markdown, HTML or the comments of a program, and hand the prose in between to a lexer: the one for the language given with `with_language`, any lexer given with `with_lexer`, or, when built with `new()`, the one for the language guessed from each document (see [`ProseLexer`](https://docs.rs/harper-core/latest/harper_core/parsers/enum.ProseLexer.html)). The [Markdown parser](https://docs.rs/harper-core/latest/harper_core/parsers/struct.Markdown.html) is a great example. The name "parser" comes from these.
- [Linter](https://docs.rs/harper-core/latest/harper_core/linting/trait.Linter.html): A trait that, provided a document, will produce zero or more [Lints](https://docs.rs/harper-core/latest/harper_core/linting/struct.Lint.html#). This is usually done using direct queries on the document or by implementing a [`PatternLinter`](https://docs.rs/harper-core/latest/harper_core/linting/trait.PatternLinter.html).

Each language is a separate part of `harper-core`, behind its own Cargo feature (`en` for English, on by default), and `harper-core` builds with any set of languages, including none.
A language's code and data live in a folder named after its code, `harper-core/src/lang/<code>/`:

- its dictionary and other word lists;
- `lexing.rs`: its lexer, if it needs its own;
- `linting/`: its rules, including Weir rules and, for English, `default_config.json`;
- for English, `expr/` and `patterns/`: helpers that only match English words.

One thing lives elsewhere: its part-of-speech tagger and chunker models, if it has them, in `harper-brill/src/lang/<code>/`.

Everything else in `harper-core` (tokens, file-format parsers, dictionaries, expressions and patterns, Weir, the lint machinery) is shared by every language.

If you want to add a rule to Harper, read [Author a Rule](./author-a-rule).
To add a language, read [Add a Language](./add-a-language).

## `harper-ls`

`harper-ls` is a language server that wraps around `harper-core`.
In essence, it enables text editors and IDEs to access the capabilities of Harper over a network or via standard input/output.

If you aren't familiar with what a language server does, I would suggest reading [this](https://tamerlan.dev/an-introduction-to-the-language-server-protocol/) or the [official language server protocol documentation](https://microsoft.github.io/language-server-protocol/).

When Harper is used through Neovim, Visual Studio Code, Helix, Emacs or Sublime Text, `harper-ls` is the interface.

You can read more about it [here](../integrations/language-server).

## `harper.js`

`harper.js` is a JavaScript/TypeScript module that enables developers to use Harper on any platform that supports JavaScript and WebAssembly.
Most of the JavaScript code in `harper.js` exists to load and manage the underlying WebAssembly module (otherwise known as `harper-wasm`).

[There are more details about it in the documentation.](../harperjs/introduction)
