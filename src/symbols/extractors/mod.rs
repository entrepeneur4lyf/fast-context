//! Symbol extractors for different programming languages
//!
//! Each language has its own extractor module that implements the SymbolExtractor trait.

use crate::parsers::LanguageId;
use crate::symbols::{Symbol, SymbolExtractor};
use std::collections::HashMap;
use tree_sitter::Tree;

// Language extractor modules
pub mod bash_extractor;
pub mod cpp_extractor;
pub mod csharp_extractor;
pub mod css_extractor;
pub mod dart_extractor;
pub mod go_extractor;
pub mod html_extractor;
pub mod java_extractor;
pub mod javascript_extractor;
pub mod jsdoc_extractor;
pub mod json_extractor;
pub mod lua_extractor;
pub mod markdown_extractor;
pub mod objc_extractor;
pub mod php_extractor;
pub mod python_extractor;
pub mod regex_extractor;
pub mod ruby_extractor;
pub mod rust_extractor;
pub mod scala_extractor;
pub mod swift_extractor;
pub mod typescript_extractor;
pub mod xml_extractor;
pub mod yaml_extractor;
pub mod zig_extractor;

// Re-export extractors
pub use bash_extractor::BashExtractor;
pub use cpp_extractor::CppExtractor;
pub use csharp_extractor::CSharpExtractor;
pub use css_extractor::CssExtractor;
pub use dart_extractor::DartExtractor;
pub use go_extractor::GoExtractor;
pub use html_extractor::HtmlExtractor;
pub use java_extractor::JavaExtractor;
pub use javascript_extractor::JavaScriptExtractor;
pub use json_extractor::JsonExtractor;
pub use lua_extractor::LuaExtractor;
pub use markdown_extractor::MarkdownExtractor;
pub use objc_extractor::ObjectiveCExtractor;
pub use php_extractor::PhpExtractor;
pub use python_extractor::PythonExtractor;
pub use regex_extractor::RegexExtractor;
pub use ruby_extractor::RubyExtractor;
pub use rust_extractor::RustExtractor;
pub use scala_extractor::ScalaExtractor;
pub use swift_extractor::SwiftExtractor;
pub use xml_extractor::XmlExtractor;
pub use yaml_extractor::YamlExtractor;
pub use typescript_extractor::TypeScriptExtractor;
pub use zig_extractor::ZigExtractor;

/// Symbol extractor factory
pub struct SymbolExtractorFactory {
    extractors: HashMap<LanguageId, Box<dyn SymbolExtractor + Send + Sync>>,
}

impl SymbolExtractorFactory {
    pub fn new() -> Self {
        let mut extractors: HashMap<LanguageId, Box<dyn SymbolExtractor + Send + Sync>> =
            HashMap::new();

        // Register language extractors
        extractors.insert(LanguageId::Rust, Box::new(RustExtractor));
        extractors.insert(LanguageId::Python, Box::new(PythonExtractor));
        extractors.insert(LanguageId::JavaScript, Box::new(JavaScriptExtractor));
        extractors.insert(LanguageId::TypeScript, Box::new(TypeScriptExtractor)); // ✅ Dedicated TypeScript extractor
        extractors.insert(LanguageId::Java, Box::new(JavaExtractor));
        extractors.insert(LanguageId::Go, Box::new(GoExtractor));
        extractors.insert(LanguageId::CSharp, Box::new(CSharpExtractor));
        extractors.insert(LanguageId::Cpp, Box::new(CppExtractor));
        extractors.insert(LanguageId::Swift, Box::new(SwiftExtractor));
        extractors.insert(LanguageId::ObjectiveC, Box::new(ObjectiveCExtractor));
        extractors.insert(LanguageId::PHP, Box::new(PhpExtractor));
        extractors.insert(LanguageId::Ruby, Box::new(RubyExtractor));
        extractors.insert(LanguageId::Scala, Box::new(ScalaExtractor));
        extractors.insert(LanguageId::Zig, Box::new(ZigExtractor));
        extractors.insert(LanguageId::Dart, Box::new(DartExtractor));
        extractors.insert(LanguageId::Lua, Box::new(LuaExtractor));
        extractors.insert(LanguageId::Bash, Box::new(BashExtractor));
        extractors.insert(LanguageId::CSS, Box::new(CssExtractor));
        extractors.insert(LanguageId::HTML, Box::new(HtmlExtractor));
        extractors.insert(LanguageId::XML, Box::new(XmlExtractor));
        extractors.insert(LanguageId::JSON, Box::new(JsonExtractor));
        extractors.insert(LanguageId::Regex, Box::new(RegexExtractor));
        extractors.insert(LanguageId::Markdown, Box::new(MarkdownExtractor));
        extractors.insert(LanguageId::YAML, Box::new(YamlExtractor));

        Self { extractors }
    }

    pub fn extract_symbols(
        &self,
        tree: &Tree,
        source: &str,
        file_path: &str,
        language: LanguageId,
    ) -> Vec<Symbol> {
        if let Some(extractor) = self.extractors.get(&language) {
            extractor.extract_symbols(tree, source, file_path)
        } else {
            Vec::new()
        }
    }
}

impl Default for SymbolExtractorFactory {
    fn default() -> Self {
        Self::new()
    }
}
