//! Integration tests for cross-module functionality
//! Tests the interaction between different components

use fast_context::core::CoreAnalyzerOptions;
#[cfg(feature = "nodejs")]
use fast_context::detect_language;
use fast_context::CoreAnalyzer;
#[cfg(feature = "nodejs")]
use fast_context::LanguageId;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_end_to_end_analysis_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a realistic project structure
        create_test_project_structure(temp_path);

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        // Test full project analysis
        let result = analyzer.analyze();
        assert!(result.is_ok());

        let analysis_result = result.unwrap();
        assert!(analysis_result.file_count > 0);
        assert!(analysis_result.symbol_count > 0);
        assert!(!analysis_result.languages.is_empty());
    }

    #[test]
    fn test_multi_language_project_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create files in different languages
        create_multi_language_project(temp_path);

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.analyze();

        assert!(result.is_ok());
        let analysis_result = result.unwrap();
        let languages: Vec<String> = analysis_result
            .languages
            .iter()
            .map(|lang| lang.to_lowercase())
            .collect();

        // Should detect multiple languages
        assert!(languages.len() >= 3);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"javascript".to_string()));
        assert!(languages.contains(&"python".to_string()));
    }

    #[test]
    fn test_ignore_patterns_integration() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create project with files that should be ignored
        create_project_with_ignored_files(temp_path);

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.analyze();

        assert!(result.is_ok());
        let analysis_result = result.unwrap();

        // Should not include ignored files in the count
        // We created 3 source files and 3 ignored files, so should only count 3
        assert_eq!(analysis_result.file_count, 3);
    }

    #[test]
    fn test_max_files_limit_is_enforced() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        create_large_test_project(temp_path);

        let analyzer = CoreAnalyzer::with_options(
            temp_path.to_string_lossy().to_string(),
            None,
            None,
            CoreAnalyzerOptions {
                max_files: Some(7),
                parallel_processing: true,
            },
        );

        let result = analyzer.analyze().unwrap();
        assert_eq!(result.file_count, 7);
    }

    #[test]
    fn test_parallel_processing_toggle_preserves_results() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        create_multi_language_project(temp_path);

        let parallel = CoreAnalyzer::with_options(
            temp_path.to_string_lossy().to_string(),
            None,
            None,
            CoreAnalyzerOptions {
                max_files: None,
                parallel_processing: true,
            },
        )
        .analyze()
        .unwrap();

        let serial = CoreAnalyzer::with_options(
            temp_path.to_string_lossy().to_string(),
            None,
            None,
            CoreAnalyzerOptions {
                max_files: None,
                parallel_processing: false,
            },
        )
        .analyze()
        .unwrap();

        assert_eq!(parallel.file_count, serial.file_count);
        assert_eq!(parallel.symbol_count, serial.symbol_count);
        assert_eq!(parallel.relationships.len(), serial.relationships.len());
    }

    #[test]
    fn test_analysis_reports_skipped_supported_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("main.rs"), "fn main() {}").unwrap();
        fs::write(temp_path.join("oversized.rs"), "a".repeat(11 * 1024 * 1024)).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.analyze().unwrap();

        assert_eq!(result.file_count, 1);
        assert_eq!(result.skipped_files.len(), 1);
        assert_eq!(result.skipped_files[0].stage, "read");
        assert!(result.skipped_files[0].file_path.ends_with("oversized.rs"));
        assert!(result.skipped_files[0].reason.contains("File too large"));
    }

    #[test]
    fn test_symbol_extraction_across_languages() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Test symbol extraction for each supported language
        let test_files = create_symbol_test_files(temp_path);

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        for (file_path, expected_symbols) in test_files {
            let result = analyzer.find_symbols_in_file(file_path.to_string_lossy().to_string());
            assert!(result.is_ok());

            let symbols = result.unwrap();
            assert!(
                symbols.len() >= expected_symbols,
                "Expected at least {} symbols, got {}",
                expected_symbols,
                symbols.len()
            );
        }
    }

    #[test]
    #[cfg(feature = "nodejs")]
    fn test_language_detection_integration() {
        let test_cases = vec![
            ("main.rs", LanguageId::Rust),
            ("app.js", LanguageId::JavaScript),
            ("component.tsx", LanguageId::TypeScript),
            ("script.py", LanguageId::Python),
            ("Main.java", LanguageId::Java),
            ("main.go", LanguageId::Go),
            ("index.php", LanguageId::PHP),
            ("app.rb", LanguageId::Ruby),
            ("ViewController.swift", LanguageId::Swift),
            ("widget.dart", LanguageId::Dart),
            ("main.zig", LanguageId::Zig),
            ("script.lua", LanguageId::Lua),
            ("setup.sh", LanguageId::Bash),
            ("style.css", LanguageId::CSS),
            ("index.html", LanguageId::HTML),
            ("config.xml", LanguageId::XML),
            ("data.json", LanguageId::JSON),
            ("config.yaml", LanguageId::YAML),
            ("README.md", LanguageId::Markdown),
            ("pattern.regex", LanguageId::Regex),
            ("AppDelegate.m", LanguageId::ObjectiveC),
        ];

        for (filename, expected_lang) in test_cases {
            let detected = detect_language(filename.to_string());
            if let Some(detected_str) = detected {
                let detected_lang = LanguageId::from_string(detected_str.as_str());

                assert_eq!(
                    detected_lang,
                    Some(expected_lang),
                    "Failed for file: {}, expected: {:?}, got: {:?}",
                    filename,
                    expected_lang,
                    detected_lang
                );
            } else {
                panic!("No language detected for file: {}", filename);
            }
        }
    }

    #[test]
    fn test_performance_with_large_project() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a larger project structure
        create_large_test_project(temp_path);

        let start = std::time::Instant::now();
        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.analyze();
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Analysis should complete within reasonable time (5 seconds for large project)
        assert!(
            duration.as_secs() < 5,
            "Analysis took too long: {:?}",
            duration
        );

        let analysis_result = result.unwrap();
        assert!(analysis_result.file_count >= 50); // Should have processed many files
    }

    #[test]
    fn test_error_recovery_in_mixed_project() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create project with both valid and invalid files
        create_mixed_validity_project(temp_path);

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.analyze();

        // Should succeed despite some invalid files
        assert!(result.is_ok());

        let analysis_result = result.unwrap();
        // Should have processed the valid files
        assert!(analysis_result.file_count > 0);
        assert!(analysis_result.symbol_count > 0);
    }

    // Helper functions to create test project structures

    fn create_test_project_structure(base_path: &std::path::Path) {
        // Create src directory
        let src_dir = base_path.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Create main.rs
        fs::write(
            src_dir.join("main.rs"),
            r#"
            fn main() {
                println!("Hello, world!");
            }
            
            struct TestStruct {
                field: i32,
            }
            
            impl TestStruct {
                fn new() -> Self {
                    Self { field: 0 }
                }
            }
        "#,
        )
        .unwrap();

        // Create lib.rs
        fs::write(
            src_dir.join("lib.rs"),
            r#"
            pub mod utils;
            
            pub fn public_function() -> i32 {
                42
            }
            
            const CONSTANT: &str = "test";
        "#,
        )
        .unwrap();

        // Create utils.rs
        fs::write(
            src_dir.join("utils.rs"),
            r#"
            pub fn helper_function() -> String {
                "helper".to_string()
            }
            
            pub enum TestEnum {
                Variant1,
                Variant2(i32),
            }
        "#,
        )
        .unwrap();
    }

    fn create_multi_language_project(base_path: &std::path::Path) {
        // Rust file
        fs::write(base_path.join("main.rs"), "fn main() {}").unwrap();

        // JavaScript file
        fs::write(
            base_path.join("app.js"),
            r#"
            function greet(name) {
                return `Hello, ${name}!`;
            }
            
            class App {
                constructor() {
                    this.name = "App";
                }
            }
        "#,
        )
        .unwrap();

        // Python file
        fs::write(
            base_path.join("script.py"),
            r#"
            def hello_world():
                return "Hello, World!"
            
            class MyClass:
                def __init__(self):
                    self.value = 42
        "#,
        )
        .unwrap();

        // TypeScript file
        fs::write(
            base_path.join("component.ts"),
            r#"
            interface User {
                name: string;
                age: number;
            }
            
            function createUser(name: string, age: number): User {
                return { name, age };
            }
        "#,
        )
        .unwrap();
    }

    fn create_project_with_ignored_files(base_path: &std::path::Path) {
        // Source files (should be included)
        fs::write(base_path.join("main.rs"), "fn main() {}").unwrap();
        fs::write(base_path.join("lib.rs"), "pub fn test() {}").unwrap();
        fs::write(base_path.join("utils.rs"), "pub fn helper() {}").unwrap();

        // Create ignored directories and files
        let node_modules = base_path.join("node_modules");
        fs::create_dir_all(&node_modules).unwrap();
        fs::write(node_modules.join("package.js"), "module.exports = {};").unwrap();

        let target_dir = base_path.join("target");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("build.rs"), "fn main() {}").unwrap();

        let git_dir = base_path.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "[core]").unwrap();
    }

    fn create_symbol_test_files(base_path: &std::path::Path) -> Vec<(std::path::PathBuf, usize)> {
        let mut test_files = Vec::new();

        // Rust file with multiple symbols
        let rust_file = base_path.join("test.rs");
        fs::write(
            &rust_file,
            r#"
            fn function1() {}
            fn function2() {}
            struct Struct1 {}
            enum Enum1 { A, B }
            const CONST1: i32 = 42;
        "#,
        )
        .unwrap();
        test_files.push((rust_file, 4));

        // JavaScript file with multiple symbols
        let js_file = base_path.join("test.js");
        fs::write(
            &js_file,
            r#"
            function func1() {}
            function func2() {}
            class Class1 {}
            const const1 = 42;
            var var1 = "test";
        "#,
        )
        .unwrap();
        test_files.push((js_file, 4));

        // Python file with multiple symbols
        let py_file = base_path.join("test.py");
        fs::write(
            &py_file,
            r#"
            def function1():
                pass
            
            def function2():
                pass
            
            class Class1:
                pass
            
            CONSTANT1 = 42
            variable1 = "test"
        "#,
        )
        .unwrap();
        test_files.push((py_file, 4));

        test_files
    }

    fn create_large_test_project(base_path: &std::path::Path) {
        // Create multiple directories with files
        for i in 0..10 {
            let dir = base_path.join(format!("module_{}", i));
            fs::create_dir_all(&dir).unwrap();

            for j in 0..5 {
                let file_path = dir.join(format!("file_{}.rs", j));
                fs::write(
                    &file_path,
                    format!(
                        r#"
                    pub fn function_{}() -> i32 {{
                        {}
                    }}
                    
                    pub struct Struct_{} {{
                        field: i32,
                    }}
                    
                    impl Struct_{} {{
                        pub fn new() -> Self {{
                            Self {{ field: {} }}
                        }}
                    }}
                "#,
                        j,
                        j * 10,
                        j,
                        j,
                        j
                    ),
                )
                .unwrap();
            }
        }
    }

    fn create_mixed_validity_project(base_path: &std::path::Path) {
        // Valid files
        fs::write(base_path.join("valid1.rs"), "fn test() {}").unwrap();
        fs::write(base_path.join("valid2.js"), "function test() {}").unwrap();

        // Invalid/malformed files
        fs::write(base_path.join("invalid1.json"), r#"{"broken": json"#).unwrap();
        fs::write(base_path.join("invalid2.rs"), "fn broken( {}").unwrap();

        // Binary file
        let binary_data: Vec<u8> = (0..255).collect();
        fs::write(base_path.join("binary.bin"), binary_data).unwrap();

        // Empty file
        fs::write(base_path.join("empty.rs"), "").unwrap();
    }
}
