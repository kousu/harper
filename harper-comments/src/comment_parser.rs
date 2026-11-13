use harper_core::lexers::Lexer;
use std::path::Path;

use crate::comment_parsers;
use comment_parsers::{Go, JavaDoc, JsDoc, Lua, Solidity, Unit};
use harper_core::parsers::{self, MarkdownOptions, Parser, ProseLexer};
use harper_core::spell::MutableDictionary;
use harper_core::{LanguageFamily, Lrc, Token};
use tree_sitter::Node;

use crate::masker::CommentMasker;

pub struct CommentParser {
    masker: CommentMasker,
    language_id: &'static str,
    markdown_options: MarkdownOptions,
    prose: ProseLexer,
}

impl CommentParser {
    pub fn create_ident_dict(&self, source: &[char]) -> Option<MutableDictionary> {
        self.masker.create_ident_dict(source)
    }

    /// A parser for the comments in `language_id` source code. It guesses the natural language
    /// of each document's comments (see [`ProseLexer::Guess`]); use [`Self::new_from_language_id_with_language`]
    /// when you know it.
    pub fn new_from_language_id(
        language_id: &str,
        markdown_options: MarkdownOptions,
    ) -> Option<Self> {
        Self::new_from_language_id_with(language_id, markdown_options, ProseLexer::Guess)
    }

    /// Like [`Self::new_from_language_id`], for comments in `language`.
    pub fn new_from_language_id_with_language(
        language_id: &str,
        markdown_options: MarkdownOptions,
        language: LanguageFamily,
    ) -> Option<Self> {
        Self::new_from_language_id_with(
            language_id,
            markdown_options,
            ProseLexer::for_language(language),
        )
    }

    /// Like [`Self::new_from_language_id`], handing the prose in comments to `prose`.
    pub fn new_from_language_id_with_lexer(
        language_id: &str,
        markdown_options: MarkdownOptions,
        prose: Lrc<dyn Lexer>,
    ) -> Option<Self> {
        Self::new_from_language_id_with(language_id, markdown_options, ProseLexer::Given(prose))
    }

    fn new_from_language_id_with(
        language_id: &str,
        markdown_options: MarkdownOptions,
        prose: ProseLexer,
    ) -> Option<Self> {
        let (language_id, language) = match language_id {
            "c" => ("c", tree_sitter_c::LANGUAGE),
            "clojure" => ("clojure", tree_sitter_clojure::LANGUAGE),
            "cmake" => ("cmake", tree_sitter_cmake::LANGUAGE),
            "cpp" => ("cpp", tree_sitter_cpp::LANGUAGE),
            "csharp" => ("csharp", tree_sitter_c_sharp::LANGUAGE),
            "dart" => ("dart", harper_tree_sitter_dart::LANGUAGE),
            "gleam" => ("gleam", tree_sitter_gleam::LANGUAGE),
            "elixir" => ("elixir", tree_sitter_elixir::LANGUAGE),
            "go" => ("go", tree_sitter_go::LANGUAGE),
            "groovy" => ("groovy", tree_sitter_groovy::LANGUAGE),
            "haskell" => ("haskell", tree_sitter_haskell::LANGUAGE),
            "daml" => ("daml", tree_sitter_haskell::LANGUAGE),
            "java" => ("java", tree_sitter_java::LANGUAGE),
            "javascript" => ("javascript", tree_sitter_javascript::LANGUAGE),
            "javascriptreact" => ("javascriptreact", tree_sitter_typescript::LANGUAGE_TSX),
            "kotlin" => ("kotlin", tree_sitter_kotlin_ng::LANGUAGE),
            "lua" => ("lua", tree_sitter_lua::LANGUAGE),
            "nix" => ("nix", tree_sitter_nix::LANGUAGE),
            "php" => ("php", tree_sitter_php::LANGUAGE_PHP),
            "powershell" => ("powershell", tree_sitter_powershell::LANGUAGE),
            "r" => ("r", tree_sitter_r::LANGUAGE),
            "ruby" => ("ruby", tree_sitter_ruby::LANGUAGE),
            "rust" => ("rust", tree_sitter_rust::LANGUAGE),
            "scala" => ("scala", tree_sitter_scala::LANGUAGE),
            "shellscript" => ("shellscript", tree_sitter_bash::LANGUAGE),
            "solidity" => ("solidity", tree_sitter_solidity::LANGUAGE),
            "swift" => ("swift", tree_sitter_swift::LANGUAGE),
            "toml" => ("toml", tree_sitter_toml_ng::LANGUAGE),
            "typescript" => ("typescript", tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
            "typescriptreact" => ("typescriptreact", tree_sitter_typescript::LANGUAGE_TSX),
            "zig" => ("zig", tree_sitter_zig::LANGUAGE),
            _ => return None,
        };

        Some(Self {
            masker: CommentMasker::new(language.into(), Self::node_condition),
            language_id,
            markdown_options,
            prose,
        })
    }

    /// The parser for the comments of this programming language, handing their prose to `prose`.
    fn comment_parser(&self, prose: Lrc<dyn Lexer>) -> Box<dyn Parser> {
        let markdown_options = self.markdown_options;
        match self.language_id {
            "go" => Box::new(Go::new_markdown(markdown_options, prose)),
            "java" => Box::new(JavaDoc::with_lexer(prose)),
            "javascript" | "javascriptreact" | "typescript" | "typescriptreact" => {
                Box::new(JsDoc::new_markdown(markdown_options, prose))
            }
            "lua" => Box::new(Lua::new_markdown(markdown_options, prose)),
            "solidity" => Box::new(Solidity::new_markdown(markdown_options, prose)),
            _ => Box::new(Unit::new_markdown(markdown_options, prose)),
        }
    }

    /// Infer the programming language from a provided filename.
    pub fn new_from_filename(filename: &Path, markdown_options: MarkdownOptions) -> Option<Self> {
        Self::new_from_language_id(Self::filename_to_filetype(filename)?, markdown_options)
    }

    /// Like [`Self::new_from_filename`], for comments in `language`.
    pub fn new_from_filename_with_language(
        filename: &Path,
        markdown_options: MarkdownOptions,
        language: LanguageFamily,
    ) -> Option<Self> {
        Self::new_from_language_id_with_language(
            Self::filename_to_filetype(filename)?,
            markdown_options,
            language,
        )
    }

    /// Convert a provided path to a corresponding Language Server Protocol file
    /// type.
    ///
    /// Note to contributors: try to keep this in sync with
    /// [`Self::new_from_language_id`]
    fn filename_to_filetype(path: &Path) -> Option<&'static str> {
        Some(match path.extension()?.to_str()? {
            "c" => "c",
            "bb" | "cljc" | "cljd" | "clj" | "cljs" => "clojure",
            "cmake" => "cmake",
            "cpp" | "h" => "cpp",
            "cs" => "csharp",
            "dart" => "dart",
            "gleam" => "gleam",
            "ex" | "exs" => "elixir",
            "go" => "go",
            "groovy" | "gradle" => "groovy",
            "hs" => "haskell",
            "daml" => "daml",
            "java" => "java",
            "js" => "javascript",
            "jsx" => "javascriptreact",
            "kt" | "kts" => "kotlin",
            "lua" => "lua",
            "nix" => "nix",
            "php" => "php",
            "ps1" | "psd1" | "psm1" => "powershell",
            "r" | "R" => "r",
            "rb" => "ruby",
            "rs" => "rust",
            "sbt" | "sc" | "scala" | "mill" => "scala",
            "bash" | "sh" => "shellscript",
            "sol" => "solidity",
            "swift" => "swift",
            "toml" => "toml",
            "ts" => "typescript",
            "tsx" => "typescriptreact",
            "zig" => "zig",
            _ => return None,
        })
    }

    fn node_condition(n: &Node) -> bool {
        n.kind().contains("comment")
    }
}

impl Parser for CommentParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        let prose = self.prose.for_masked(&self.masker, source);
        parsers::parse_masked(&self.masker, &self.comment_parser(prose), source)
    }
}

#[cfg(test)]
mod tests {
    use super::CommentParser;
    use harper_core::parsers::{MarkdownOptions, StrParser};

    #[test]
    fn hang() {
        use std::sync::mpsc::channel;
        use std::thread;
        use std::time::Duration;

        let (tx, rx) = channel::<()>();

        let handle = thread::spawn(move || {
            let opts = MarkdownOptions::default();
            let parser = CommentParser::new_from_language_id("java", opts).unwrap();
            let _res = parser.parse_str("//{@j");
            tx.send(()).expect("send failed");
        });

        rx.recv_timeout(Duration::from_secs(10)).expect("timed out");
        handle.join().expect("failed to join");
    }
}
