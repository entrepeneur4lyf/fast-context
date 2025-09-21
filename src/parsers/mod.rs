//! # Tree-sitter Parser Integration
//!
//! Multi-language parser factory supporting 20+ programming languages via Tree-sitter.
//! Provides unified AST parsing interface for consistent symbol extraction across languages.

use serde::{Deserialize, Serialize};
use lru::LruCache;
use std::num::NonZeroUsize;
use tree_sitter::{Language, Parser, Tree};


/// Supported programming languages for analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageId {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    Go,
    CSharp,
    Cpp,
    Swift,
    ObjectiveC,
    PHP,
    Ruby,
    Scala,
    Zig,
    Dart,
    Lua,
    Bash,
    CSS,
    HTML,
    XML,
    JSON,
    YAML,
    Markdown,
    JSDoc,
    Regex,
}

impl std::fmt::Display for LanguageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::Python => write!(f, "Python"),
            Self::JavaScript => write!(f, "JavaScript"),
            Self::TypeScript => write!(f, "TypeScript"),
            Self::Java => write!(f, "Java"),
            Self::Go => write!(f, "Go"),
            Self::CSharp => write!(f, "CSharp"),
            Self::Cpp => write!(f, "Cpp"),
            Self::Swift => write!(f, "Swift"),
            Self::ObjectiveC => write!(f, "ObjectiveC"),
            Self::PHP => write!(f, "PHP"),
            Self::Ruby => write!(f, "Ruby"),
            Self::Scala => write!(f, "Scala"),
            Self::Zig => write!(f, "Zig"),
            Self::Dart => write!(f, "Dart"),
            Self::Lua => write!(f, "Lua"),
            Self::Bash => write!(f, "Bash"),
            Self::CSS => write!(f, "CSS"),
            Self::HTML => write!(f, "HTML"),
            Self::XML => write!(f, "XML"),
            Self::JSON => write!(f, "JSON"),
            Self::YAML => write!(f, "YAML"),
            Self::Markdown => write!(f, "Markdown"),
            Self::JSDoc => write!(f, "JSDoc"),
            Self::Regex => write!(f, "Regex"),
        }
    }
}

impl LanguageId {
    /// Convert language string to LanguageId
    pub fn from_string(lang: &str) -> Option<Self> {
        match lang.to_lowercase().as_str() {
            "rust" => Some(Self::Rust),
            "python" => Some(Self::Python),
            "javascript" => Some(Self::JavaScript),
            "typescript" => Some(Self::TypeScript),
            "java" => Some(Self::Java),
            "go" => Some(Self::Go),
            "csharp" => Some(Self::CSharp),
            "cpp" => Some(Self::Cpp),
            "swift" => Some(Self::Swift),
            "objectivec" => Some(Self::ObjectiveC),
            "php" => Some(Self::PHP),
            "ruby" => Some(Self::Ruby),
            "scala" => Some(Self::Scala),
            "zig" => Some(Self::Zig),
            "dart" => Some(Self::Dart),
            "lua" => Some(Self::Lua),
            "bash" => Some(Self::Bash),
            "css" => Some(Self::CSS),
            "html" => Some(Self::HTML),
            "xml" => Some(Self::XML),
            "json" => Some(Self::JSON),
            "yaml" => Some(Self::YAML),
            "markdown" => Some(Self::Markdown),
            "jsdoc" => Some(Self::JSDoc),
            "regex" => Some(Self::Regex),
            _ => None,
        }
    }

    /// Convert LanguageId to lowercase string for consistency
    pub fn to_lowercase_string(&self) -> String {
        match self {
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Java => "java",
            Self::Go => "go",
            Self::CSharp => "csharp",
            Self::Cpp => "cpp",
            Self::Swift => "swift",
            Self::ObjectiveC => "objectivec",
            Self::PHP => "php",
            Self::Ruby => "ruby",
            Self::Scala => "scala",
            Self::Zig => "zig",
            Self::Dart => "dart",
            Self::Lua => "lua",
            Self::Bash => "bash",
            Self::CSS => "css",
            Self::HTML => "html",
            Self::XML => "xml",
            Self::JSON => "json",
            Self::YAML => "yaml",
            Self::Markdown => "markdown",
            Self::JSDoc => "jsdoc",
            Self::Regex => "regex",
        }.to_string()
    }

    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "py" | "pyw" => Some(Self::Python),
            "js" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "java" => Some(Self::Java),
            "go" => Some(Self::Go),
            "cs" => Some(Self::CSharp),
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hxx" | "h++" => Some(Self::Cpp),
            "swift" => Some(Self::Swift),
            "m" | "mm" => Some(Self::ObjectiveC),
            "php" => Some(Self::PHP),
            "rb" => Some(Self::Ruby),
            "scala" | "sc" => Some(Self::Scala),
            "zig" => Some(Self::Zig),
            "dart" => Some(Self::Dart),
            "lua" => Some(Self::Lua),
            "sh" | "bash" => Some(Self::Bash),
            "css" => Some(Self::CSS),
            "html" | "htm" => Some(Self::HTML),
            "xml" => Some(Self::XML),
            "json" => Some(Self::JSON),
            "yaml" | "yml" => Some(Self::YAML),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }

    /// Get the tree-sitter language for this language ID
    pub fn tree_sitter_language(&self) -> Option<Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Self::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            Self::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            Self::Java => Some(tree_sitter_java::LANGUAGE.into()),
            Self::Go => Some(tree_sitter_go::LANGUAGE.into()),
            Self::CSharp => Some(tree_sitter_c_sharp::LANGUAGE.into()),
            Self::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
            Self::Swift => Some(tree_sitter_swift::LANGUAGE.into()),
            Self::ObjectiveC => Some(tree_sitter_objc::LANGUAGE.into()),
            Self::PHP => Some(tree_sitter_php::LANGUAGE_PHP.into()),
            Self::Ruby => Some(tree_sitter_ruby::LANGUAGE.into()),
            Self::Scala => Some(tree_sitter_scala::LANGUAGE.into()),
            Self::Zig => Some(tree_sitter_zig::LANGUAGE.into()),
            Self::Dart => Some(tree_sitter_dart::language()),
            Self::Lua => Some(tree_sitter_lua::LANGUAGE.into()),
            Self::Bash => Some(tree_sitter_bash::LANGUAGE.into()),
            Self::CSS => Some(tree_sitter_css::LANGUAGE.into()),
            Self::HTML => Some(tree_sitter_html::LANGUAGE.into()),
            Self::XML => Some(tree_sitter_xml::LANGUAGE_XML.into()),
            Self::JSON => Some(tree_sitter_json::LANGUAGE.into()),
            Self::YAML => Some(tree_sitter_yaml::LANGUAGE.into()),
            Self::Markdown => Some(tree_sitter_md::LANGUAGE.into()),
            Self::JSDoc => Some(tree_sitter_jsdoc::LANGUAGE.into()),
            Self::Regex => Some(tree_sitter_regex::LANGUAGE.into()),
        }
    }
}

/// Parse result containing the AST and metadata
#[derive(Debug)]
pub struct ParseResult {
    pub tree: Tree,
    pub language: LanguageId,
    pub source: String,
}

/// Multi-language parser factory with LRU caching and size limits
pub struct ParserFactory {
    parsers: LruCache<LanguageId, Parser>,
    max_parsers: usize,
}

impl ParserFactory {
    /// Create a new parser factory with default size limits
    pub fn new() -> Self {
        Self::with_capacity(10) // Reasonable default for most projects
    }

    /// Create a new parser factory with custom capacity
    pub fn with_capacity(max_parsers: usize) -> Self {
        // Validate cache size is reasonable (between 1 and 1000)
        let validated_size = crate::validation::validate_range(max_parsers, 1, 1000, "max_parsers")
            .unwrap_or_else(|e| {
                eprintln!("Warning: Invalid parser cache size {}: {}", max_parsers, e);
                10 // Default to reasonable value
            });
        
        Self {
            parsers: LruCache::new(NonZeroUsize::new(validated_size).unwrap_or_else(|| NonZeroUsize::new(1).unwrap())),
            max_parsers: validated_size,
        }
    }

    /// Get or create a parser for the specified language
    pub fn get_parser(&mut self, language: LanguageId) -> Option<&mut Parser> {
        if self.parsers.get(&language).is_none() {
            if let Some(ts_language) = language.tree_sitter_language() {
                let mut parser = Parser::new();
                if parser.set_language(&ts_language).is_ok() {
                    self.parsers.put(language, parser);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        self.parsers.get_mut(&language)
    }

    /// Parse source code and return AST
    pub fn parse(&mut self, source: &str, language: LanguageId) -> Option<ParseResult> {
        let parser = self.get_parser(language)?;
        let tree = parser.parse(source, None)?;

        Some(ParseResult {
            tree,
            language,
            source: source.to_string(),
        })
    }

    /// Get current cache size
    pub fn cache_size(&self) -> usize {
        self.parsers.len()
    }

    /// Get maximum cache capacity
    pub fn cache_capacity(&self) -> usize {
        self.max_parsers
    }

    /// Clear all cached parsers
    pub fn clear_cache(&mut self) {
        self.parsers.clear();
    }

    /// Resize the cache and evict excess parsers if needed
    pub fn resize_cache(&mut self, new_capacity: usize) {
        self.max_parsers = new_capacity;
        self.parsers.resize(NonZeroUsize::new(new_capacity).unwrap_or_else(|| NonZeroUsize::new(1).unwrap()));
    }

    /// Parse file by detecting language from extension
    pub fn parse_file(&mut self, content: &str, file_path: &str) -> Option<ParseResult> {
        let extension = std::path::Path::new(file_path).extension()?.to_str()?;

        let language = LanguageId::from_extension(extension)?;
        self.parse(content, language)
    }
}

impl Default for ParserFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(LanguageId::from_extension("rs"), Some(LanguageId::Rust));
        assert_eq!(LanguageId::from_extension("py"), Some(LanguageId::Python));
        assert_eq!(
            LanguageId::from_extension("js"),
            Some(LanguageId::JavaScript)
        );
        assert_eq!(LanguageId::from_extension("unknown"), None);
    }

    #[test]
    fn test_parser_creation() {
        let mut factory = ParserFactory::new();

        // Test Rust parser
        let result = factory.parse("fn main() {}", LanguageId::Rust);
        assert!(result.is_some());

        // Test Python parser
        let result = factory.parse("def main(): pass", LanguageId::Python);
        assert!(result.is_some());
    }
}
