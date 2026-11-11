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

#[test]
fn french_accepts_correct_text() {
    assert_eq!(
        lints(
            &["--language", "fr"],
            "Le chat mange la souris dans l'école aujourd'hui."
        ),
        []
    );
}

#[test]
fn french_flags_misspelling() {
    assert_eq!(
        lints(&["--language", "fr"], "J'aime le franssais."),
        [("SpellCheck".to_string(), "franssais".to_string())]
    );
}

#[test]
fn french_skips_english_rules() {
    // "an" before a consonant is an English `AnA` lint, but not French.
    let found = lints(&["--language", "fr"], "Un an plus tard.");
    assert!(found.iter().all(|(rule, _)| rule != "AnA"), "{found:?}");
}

#[test]
fn english_is_the_default() {
    let found = lints(&[], "This is an test.");
    assert!(found.iter().any(|(rule, _)| rule == "AnA"), "{found:?}");
}

#[test]
fn unknown_language_is_an_error() {
    let output = harper_cli()
        .args(["lint", "--language", "xx", "text"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown language: xx"));
}
