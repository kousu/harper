use std::process::Command;

fn harper_cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_harper-cli"))
}

/// Lint `text` and return each lint's rule and matched text.
fn lints(args: &[&str], text: &str) -> Vec<(String, String)> {
    let output = harper_cli()
        .args(["--no-color", "lint", "--format", "json"])
        .args(args)
        .arg(text)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("invalid JSON output: {e}\nstdout: {stdout}"));

    parsed[0]["lints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["rule"].as_str().unwrap().to_string(),
                l["matched_text"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// Run `harper-cli lint` with `args` on some text and return its stderr, asserting that it failed.
fn lint_error(args: &[&str]) -> String {
    let output = harper_cli()
        .arg("lint")
        .args(args)
        .arg("text")
        .output()
        .unwrap();

    assert!(!output.status.success());
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn english_is_the_default() {
    let found = lints(&[], "This is an test.");
    assert!(found.iter().any(|(rule, _)| rule == "AnA"), "{found:?}");
}

#[test]
fn english_by_name_or_code() {
    for language in ["en", "eng", "English"] {
        let found = lints(&["--language", language], "This is an test.");
        assert!(found.iter().any(|(rule, _)| rule == "AnA"), "{found:?}");
    }
}

#[test]
fn unknown_language_is_an_error() {
    assert!(lint_error(&["--language", "xx"]).contains("Unknown language: xx"));
}

#[test]
fn dialect_still_applies() {
    let american = lints(&["--dialect", "us"], "The colour is red.");
    let british = lints(&["--dialect", "uk"], "The colour is red.");

    assert!(!american.is_empty(), "{american:?}");
    assert_eq!(british, []);
}

#[test]
fn unknown_dialect_is_an_error() {
    assert!(lint_error(&["--dialect", "xx"]).contains("Invalid dialect 'xx'"));
}

/// Run harper-cli with `args` and return its stdout, asserting that it succeeded.
fn run(args: &[&str]) -> String {
    let output = harper_cli().args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn language_goes_before_or_after_the_subcommand() {
    let before = run(&["--language", "en", "parse", "Hello world."]);
    let after = run(&["parse", "--language", "en", "Hello world."]);
    assert_eq!(before, after);
    assert!(!before.is_empty());
}

#[test]
fn every_subcommand_takes_a_language() {
    for args in [
        &["parse", "Hello."][..],
        &["spans", "Hello."],
        &["annotate", "Hello."],
        &["metadata", "hello"],
        &["mine-words", "Hello."],
        &["nominal-phrases", "Hello."],
        &["config"],
    ] {
        let mut with_language = vec!["--language", "English"];
        with_language.extend_from_slice(args);
        run(&with_language);
    }
}

#[test]
fn unknown_language_is_an_error_for_every_subcommand() {
    let output = harper_cli()
        .args(["--language", "xx", "parse", "Hello."])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown language: xx"));
}

#[cfg(feature = "fr")]
#[test]
fn french_only_spell_checks() {
    // "an" before a consonant is an English `AnA` lint; French gets only spell checking.
    for args in [
        &["--language", "fr"][..],
        &["--language", "French", "--dialect", "ca"],
    ] {
        let found = lints(args, "Un an plus tard.");
        assert!(
            found.iter().all(|(rule, _)| rule == "SpellCheck"),
            "{found:?}"
        );
    }
}

#[cfg(feature = "fr")]
#[test]
fn unknown_french_dialect_is_an_error() {
    assert!(
        lint_error(&["--language", "fr", "--dialect", "us"])
            .contains("Invalid dialect 'us' for French")
    );
}

#[cfg(not(feature = "fr"))]
#[test]
fn french_needs_the_feature() {
    assert!(lint_error(&["--language", "fr"]).contains("This build has no French support."));
}

#[cfg(feature = "fr")]
#[test]
fn french_accepts_correct_text() {
    assert_eq!(
        lints(
            &["--language", "fr"],
            "Le chat mange la souris dans le jardin aujourd'hui."
        ),
        []
    );
}

#[cfg(feature = "fr")]
#[test]
fn french_flags_misspelling() {
    assert_eq!(
        lints(&["--language", "fr"], "Je parle le franssais."),
        [("SpellCheck".to_string(), "franssais".to_string())]
    );
}
