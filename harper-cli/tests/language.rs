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
