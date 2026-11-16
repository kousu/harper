use harper_core::linting::{LintGroup, LintKind, Linter};
use harper_core::spell::FstDictionary;
use harper_core::{Document, EnglishDialect};
use harper_git_commit::GitCommitParser;

/// Creates a unit test checking that the linting of a git commit document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.txt, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>]() {
                let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".txt")
                    )
                );

                let dict = FstDictionary::curated_for(harper_core::LanguageFamily::English);
                let document = Document::new_with_parser_and_language(source, &GitCommitParser::default(), harper_core::LanguageFamily::English);

                let mut linter = LintGroup::new_curated_for_language(dict, harper_core::Language::English(EnglishDialect::American));
                let lints = linter.lint(&document);

                dbg!(&lints);
                assert_eq!(lints.len(), $correct_expected);

                for token in document.tokens() {
                    assert!(token.span.try_get_content(document.get_source()).is_some());
                }
            }
        }
    };
}

create_test!(simple_commit.txt, 2);
create_test!(complex_verbose_commit.txt, 2);
create_test!(conventional_commit.txt, 3);
create_test!(markdown_body_commit.txt, 3);

#[test]
fn conventional_commit_prefixes_are_not_parsed() {
    for prefix in [
        "feat(harper): ",
        "fix(git-commit): ",
        "feat(nvf)!: ",
        "chore: ",
        "feat(caf\u{e9}): ",
    ] {
        let source = format!("{prefix}add missing files\n");
        let document = Document::new_with_parser_and_language(
            &source,
            &GitCommitParser::default(),
            harper_core::LanguageFamily::English,
        );
        let description_start = prefix.chars().count();

        assert!(
            document
                .tokens()
                .all(|token| token.span.start >= description_start),
            "conventional commit prefix was parsed: {prefix:?}"
        );
        assert!(
            document
                .tokens()
                .any(|token| token.span.get_content_string(document.get_source()) == "missing")
        );
    }
}

#[test]
fn conventional_commit_descriptions_are_still_spellchecked() {
    let dict = FstDictionary::curated_for(harper_core::LanguageFamily::English);
    let mut linter = LintGroup::new_curated_for_language(
        dict.clone(),
        harper_core::Language::English(EnglishDialect::American),
    );

    for source in [
        "feat(harper): add mising files\n",
        "fix(git-commit): add mising files\n",
        "feat(nvf)!: add mising files\n",
        "chore: add mising files\n",
        "feat(caf\u{e9}): add mising files\n",
    ] {
        let document = Document::new_with_parser_and_language(
            source,
            &GitCommitParser::default(),
            harper_core::LanguageFamily::English,
        );
        let misspellings: Vec<_> = linter
            .lint(&document)
            .into_iter()
            .filter(|lint| lint.lint_kind == LintKind::Spelling)
            .map(|lint| lint.span.get_content_string(document.get_source()))
            .collect();

        assert_eq!(misspellings, ["mising"], "source: {source:?}");
    }
}

#[test]
fn ordinary_subjects_and_commit_bodies_are_still_spellchecked() {
    let dict = FstDictionary::curated_for(harper_core::LanguageFamily::English);
    let mut linter = LintGroup::new_curated_for_language(
        dict.clone(),
        harper_core::Language::English(EnglishDialect::American),
    );

    for source in [
        "add mising files\n",
        "feat(harper): add files\n\nmising files\n",
        "# ignored comment\nfeat(harper): add mising files\n",
    ] {
        let document = Document::new_with_parser_and_language(
            source,
            &GitCommitParser::default(),
            harper_core::LanguageFamily::English,
        );
        let misspellings: Vec<_> = linter
            .lint(&document)
            .into_iter()
            .filter(|lint| lint.lint_kind == LintKind::Spelling)
            .map(|lint| lint.span.get_content_string(document.get_source()))
            .collect();

        assert_eq!(misspellings, ["mising"], "source: {source:?}");
    }
}

#[test]
fn hash_prefixed_descriptions_are_not_treated_as_comments() {
    let dict = FstDictionary::curated_for(harper_core::LanguageFamily::English);
    let mut linter = LintGroup::new_curated_for_language(
        dict.clone(),
        harper_core::Language::English(EnglishDialect::American),
    );

    for source in [
        "feat(harper): #123 add mising files\n",
        "# ignored comment\nfeat(harper): #123 add mising files\n",
        "feat(caf\u{e9}): #123 add mising files\n",
        "feat(harper): #123 add files\n\nmising files\n",
    ] {
        let document = Document::new_with_parser_and_language(
            source,
            &GitCommitParser::default(),
            harper_core::LanguageFamily::English,
        );
        let misspellings: Vec<_> = linter
            .lint(&document)
            .into_iter()
            .filter(|lint| lint.lint_kind == LintKind::Spelling)
            .map(|lint| lint.span.get_content_string(document.get_source()))
            .collect();

        assert_eq!(misspellings, ["mising"], "source: {source:?}");
    }
}
