use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use dirs::{config_dir, data_local_dir};
use globset::{Glob, GlobSet};
use harper_core::{EnglishDialect, Language, linting::FlatConfig, parsers::MarkdownOptions};
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl DiagnosticSeverity {
    /// Converts `self` to the equivalent LSP type.
    pub fn to_lsp(self) -> tower_lsp_server::lsp_types::DiagnosticSeverity {
        match self {
            DiagnosticSeverity::Error => tower_lsp_server::lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticSeverity::Warning => tower_lsp_server::lsp_types::DiagnosticSeverity::WARNING,
            DiagnosticSeverity::Information => {
                tower_lsp_server::lsp_types::DiagnosticSeverity::INFORMATION
            }
            DiagnosticSeverity::Hint => tower_lsp_server::lsp_types::DiagnosticSeverity::HINT,
        }
    }
}

/// Configuration for how code actions are displayed.
/// Originally motivated by [#89](https://github.com/automattic/harper/issues/89).
#[derive(Debug, Clone, Default)]
pub struct CodeActionConfig {
    /// Instructs `harper-ls` to place unstable code actions last.
    /// In this case, "unstable" refers to their existence and action.
    ///
    /// For example, we always want to allow users to add "misspelled" elements
    /// to dictionary, regardless of the spelling suggestions.
    pub force_stable: bool,
}

impl CodeActionConfig {
    pub fn from_lsp_config(value: Value) -> Result<Self> {
        let mut base = CodeActionConfig::default();

        let Value::Object(value) = value else {
            bail!("The code action configuration must be an object.");
        };

        if let Some(force_stable_val) = value.get("ForceStable") {
            let Value::Bool(force_stable) = force_stable_val else {
                bail!("ForceStable must be a boolean value.");
            };
            base.force_stable = *force_stable;
        };

        Ok(base)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub user_dict_path: PathBuf,
    pub file_dict_path: PathBuf,
    pub workspace_dict_path: PathBuf,
    pub ignored_lints_path: PathBuf,
    pub stats_path: PathBuf,
    pub lint_config: FlatConfig,
    pub diagnostic_severity: DiagnosticSeverity,
    pub code_action_config: CodeActionConfig,
    pub isolate_english: bool,
    pub markdown_options: MarkdownOptions,
    /// The language of the documents. English, in the dialect from the `dialect` setting.
    pub language: Language,
    /// Maximum length (in bytes) a file can have before it's skipped.
    /// Above this limit, the file will not be linted.
    pub max_file_length: usize,
    pub exclude_patterns: GlobSet,
    /// Delay in milliseconds after typing stops before diagnostics are published.
    /// Set to 0 to publish diagnostics immediately.
    pub diagnostic_delay_ms: u64,
}

impl Config {
    /// The English dialect that words in the user's dictionaries are tagged with.
    pub fn english_dialect(&self) -> EnglishDialect {
        match self.language {
            Language::English(dialect) => dialect,
        }
    }

    pub fn from_lsp_config(workspace_root: &Path, value: Value) -> Result<Self> {
        let mut base = Config::default();

        let workspace_root = workspace_root.canonicalize()?;
        let workspace_root = workspace_root.as_path();

        let Value::Object(value) = value else {
            bail!("Settings must be an object.");
        };

        let Some(Value::Object(value)) = value.get("harper-ls") else {
            bail!("Settings must contain a \"harper-ls\" key.");
        };

        if let Some(v) = value.get("userDictPath") {
            if !v.is_string() {
                bail!("userDict path must be a string.");
            }

            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.user_dict_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        }

        if let Some(v) = value.get("fileDictPath") {
            if !v.is_string() {
                bail!("fileDict path must be a string.");
            }

            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.file_dict_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        }

        if let Some(v) = value.get("workspaceDictPath") {
            if !v.is_string() {
                bail!("workspaceDict path must be a string.");
            }
            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.workspace_dict_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        } else {
            // Resolve the default path in the project root
            base.workspace_dict_path = base
                .workspace_dict_path
                .try_resolve_in(workspace_root)?
                .to_path_buf();
        }

        if let Some(v) = value.get("ignoredLintsPath") {
            if !v.is_string() {
                bail!("ignoredLintsPath path must be a string.");
            }

            let path = v.as_str().unwrap();
            if !path.is_empty() {
                base.ignored_lints_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            }
        }

        if let Some(v) = value.get("statsPath") {
            if let Value::String(path) = v {
                base.stats_path = path.try_resolve_in(workspace_root)?.to_path_buf();
            } else {
                bail!("statsPath must be a string.");
            }
        }

        if let Some(v) = value.get("linters") {
            base.lint_config = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("diagnosticSeverity") {
            base.diagnostic_severity = serde_json::from_value(v.clone())?;
        }

        if let Some(dialect) = string_setting(value, "dialect")? {
            let family = base.language.family().to_string();
            base.language = Language::try_from_names(&family, Some(dialect))?;
        }

        if let Some(v) = value.get("codeActions") {
            base.code_action_config = CodeActionConfig::from_lsp_config(v.clone())?;
        }

        if let Some(v) = value.get("isolateEnglish") {
            if let Value::Bool(v) = v {
                base.isolate_english = *v;
            } else {
                bail!("isolateEnglish path must be a boolean.");
            }
        }

        if let Some(v) = value.get("maxFileLength") {
            base.max_file_length = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("markdown")
            && let Some(v) = v.get("IgnoreLinkTitle")
        {
            base.markdown_options.ignore_link_title = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("excludePatterns") {
            let Some(a) = v.as_array() else {
                bail!("excludePatterns must be an array.");
            };

            let patterns: Vec<Value> = a.to_vec();
            if !patterns.is_empty() {
                let mut builder = GlobSet::builder();

                for pattern in patterns {
                    builder.add(Glob::new(pattern.as_str().unwrap())?);
                }

                base.exclude_patterns = builder.build()?;
            }
        }

        if let Some(v) = value.get("diagnosticDelayMs") {
            base.diagnostic_delay_ms = serde_json::from_value(v.clone())?;
        }

        Ok(base)
    }
}

/// The setting `key`, which must be a string if it is present.
fn string_setting<'a>(settings: &'a Map<String, Value>, key: &str) -> Result<Option<&'a str>> {
    match settings.get(key) {
        None => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => bail!("{key} must be a string."),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_dict_path: config_dir().unwrap().join("harper-ls/dictionary.txt"),
            file_dict_path: data_local_dir()
                .unwrap()
                .join("harper-ls/file_dictionaries/"),
            workspace_dict_path: ".harper-dictionary.txt".into(),
            ignored_lints_path: data_local_dir().unwrap().join("harper-ls/ignored_lints/"),
            stats_path: data_local_dir().unwrap().join("harper-ls/stats.txt"),
            lint_config: FlatConfig::default(),
            diagnostic_severity: DiagnosticSeverity::Hint,
            code_action_config: CodeActionConfig::default(),
            isolate_english: false,
            markdown_options: MarkdownOptions::default(),
            language: Language::English(EnglishDialect::American),
            max_file_length: 120_000,
            exclude_patterns: GlobSet::empty(),
            diagnostic_delay_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use harper_core::{EnglishDialect, Language};
    use serde_json::json;

    use super::Config;

    fn parse(settings: serde_json::Value) -> anyhow::Result<Config> {
        Config::from_lsp_config(std::path::Path::new("."), json!({ "harper-ls": settings }))
    }

    #[test]
    fn defaults_to_american_english() {
        assert_eq!(
            parse(json!({})).unwrap().language,
            Language::English(EnglishDialect::American)
        );
    }

    #[test]
    fn dialect_alone_still_works() {
        assert_eq!(
            parse(json!({ "dialect": "British" })).unwrap().language,
            Language::English(EnglishDialect::British)
        );
    }

    #[test]
    fn parses_dialect() {
        let config = parse(json!({ "dialect": "Canadian" })).unwrap();

        assert_eq!(config.language, Language::English(EnglishDialect::Canadian));
        assert_eq!(config.english_dialect(), EnglishDialect::Canadian);
    }

    #[test]
    fn accepts_codes_like_the_cli() {
        assert_eq!(
            parse(json!({ "dialect": "uk" })).unwrap().language,
            Language::English(EnglishDialect::British)
        );
        assert_eq!(
            parse(json!({ "dialect": "en-AU" })).unwrap().language,
            Language::English(EnglishDialect::Australian)
        );
    }

    #[test]
    fn rejects_unknown_dialect() {
        assert!(parse(json!({ "dialect": "Klingon" })).is_err());
    }

    #[test]
    fn rejects_non_string_dialect() {
        assert!(parse(json!({ "dialect": true })).is_err());
    }

    #[test]
    fn parses_diagnostic_delay() {
        let config = Config::from_lsp_config(
            std::path::Path::new("."),
            json!({
                "harper-ls": {
                    "diagnosticDelayMs": 750
                }
            }),
        )
        .unwrap();

        assert_eq!(config.diagnostic_delay_ms, 750);
    }

    #[test]
    fn defaults_diagnostic_delay_to_zero() {
        let config = Config::default();

        assert_eq!(config.diagnostic_delay_ms, 0);
    }

    #[test]
    fn parses_stats_path_without_touching_file_dict_path() {
        let config = Config::from_lsp_config(
            std::path::Path::new("."),
            json!({
                "harper-ls": {
                    "statsPath": ".harper-stats.txt"
                }
            }),
        )
        .unwrap();

        assert!(config.stats_path.ends_with(".harper-stats.txt"));
        assert!(
            config
                .file_dict_path
                .ends_with("harper-ls/file_dictionaries/"),
            "file dictionary path should keep its default value"
        );
    }
}
