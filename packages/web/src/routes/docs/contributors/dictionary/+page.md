---
title: Updating the Curated Dictionary
description: Add words and annotations to the Harper curated English dictionary and validate dictionary changes.
---

# Updating the Curated Dictionary

The curated dictionary is the English dictionary Harper uses as reference internally when analyzing or modifying English text.
It is common, especially with technical language, to come across words that are not in this dictionary.
If this happens to you, please open a PR to get them in.

PR [#343](https://github.com/Automattic/harper/pull/343) is a practical example of the ideas described here.

## One Dictionary per Language

Harper can check more than one language, and each language has its own dictionary in its own folder, `harper-core/src/lang/<code>/`.
English, for example, is in `harper-core/src/lang/en/`.
A language that isn't built into Harper (see its Cargo feature, such as `en`) has no dictionary in the binary at all.

This page describes the English dictionary.
Other languages use the same `dictionary.dict` and `annotations.json` format unless their folder has a `README.md` saying otherwise, for example when the dictionary is generated from another source by a script.
In that case, change the source and rerun the script instead of editing the output.

### Dialects

A word that belongs to only some dialects of its language says so with the `dialects` metadata field.
Each language has its own dialect names: in the English dictionary they are `AMERICAN`, `BRITISH`, `AUSTRALIAN`, `CANADIAN` and `INDIAN`, and can be combined, as in `"dialects": "AMERICAN | CANADIAN"`.
A word with no `dialects` field is accepted in every dialect of its language.

## The English Dictionary

There are two files you need to worry about.
[`harper-core/src/lang/en/dictionary.dict`](https://github.com/Automattic/harper/blob/master/harper-core/src/lang/en/dictionary.dict) and [`harper-core/src/lang/en/annotations.json`](https://github.com/Automattic/harper/blob/master/harper-core/src/lang/en/annotations.json) (formerly `affixes.json`).
The first is a list of words, tagged with modifiers defined in the second.

For example, all words, such as "move", tagged with `L`, will be expanded to two dictionary entries, "move" and "movement".
In `annotations.json`, this expansion rule looks like this:

```js title=annotations.json
{
	"L": {
		// A description of the rule.
		"#": "'-ment' suffix",
        // Denotes that the area of interest is at the _end_ of the base word.
		"kind": "suffix",
        // Declare that it is OK to use the result of the expansion with other expansions.
		"cross_product": true,
        // The actual replacement rules that result in an expansion.
		"replacements": [
			{
                // If present, remove this text from the area of interest before expansion.
				"remove": "",
				"add": "ment",
                // A simplified regex-like pattern that describes what the area of interest must look like in order for this particular replacement to be applied.
				"condition": "."
			}
		],
        // The metadata that should be applied to the expanded word.
		"target": {},
        // The metadata that should be applied to the base word.
		"base_metadata": {}
	}
}
```

Those familiar with `hunspell` might notice some similarities with their dictionary format.
The main differences are the [metadata fields.](https://docs.rs/harper-core/latest/harper_core/lexeme_metadata/struct.LexemeMetadata.html)

There is a separate section, `properties` that is specifically for special rules that add only metadata to the words they're applied to.

## Adding Nouns

You don't need to know any of the nitty-gritty details to add nouns to the dictionary.
Use the tool we have in the repo:

```bash
just addnoun <YOUR NOUN HERE>
```

If this command doesn't look familiar, [read our setup documentation for contributors](./environment).
