use criterion::{Criterion, criterion_group, criterion_main};
use harper_core::linting::{LintGroup, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Document, EnglishDialect};
use std::hint::black_box;

static ESSAY: &str = include_str!("./essay.md");

fn parse_essay(c: &mut Criterion) {
    c.bench_function("parse_essay", |b| {
        b.iter(|| {
            Document::new_with_parser_and_language(
                black_box(ESSAY),
                &harper_core::parsers::Markdown::with_language(
                    harper_core::parsers::MarkdownOptions::default(),
                    harper_core::LanguageFamily::English,
                ),
                harper_core::LanguageFamily::English,
            )
        });
    });
}

fn lint_essay(c: &mut Criterion) {
    let dictionary = FstDictionary::curated_for(harper_core::LanguageFamily::English);
    let mut lint_set = LintGroup::new_curated_for_language(
        dictionary,
        harper_core::Language::English(EnglishDialect::American),
    );
    let document = Document::new_with_parser_and_language(
        black_box(ESSAY),
        &harper_core::parsers::Markdown::with_language(
            harper_core::parsers::MarkdownOptions::default(),
            harper_core::LanguageFamily::English,
        ),
        harper_core::LanguageFamily::English,
    );

    c.bench_function("lint_essay", |b| {
        b.iter(|| lint_set.lint(&document));
    });
}

fn lint_essay_uncached(c: &mut Criterion) {
    c.bench_function("lint_essay_uncached", |b| {
        b.iter(|| {
            let dictionary = FstDictionary::curated_for(harper_core::LanguageFamily::English);
            let mut lint_set = LintGroup::new_curated_for_language(
                dictionary.clone(),
                harper_core::Language::English(EnglishDialect::American),
            );
            let document = Document::new_with_parser_and_language(
                black_box(ESSAY),
                &harper_core::parsers::Markdown::with_language(
                    harper_core::parsers::MarkdownOptions::default(),
                    harper_core::LanguageFamily::English,
                ),
                harper_core::LanguageFamily::English,
            );
            lint_set.lint(&document)
        })
    });
}

pub fn criterion_benchmark(c: &mut Criterion) {
    parse_essay(c);
    lint_essay(c);
    lint_essay_uncached(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
