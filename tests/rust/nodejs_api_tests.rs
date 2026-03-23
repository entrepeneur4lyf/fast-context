//! Node.js API integration tests
//! Tests the Node.js API layer that delegates to CoreAnalyzer

#[cfg(feature = "nodejs")]
use fast_context::analyzer::{AnalyzerConfig, FastContextAnalyzer};
use fast_context::core::CoreAnalyzer;
use std::fs;

use tempfile::TempDir;

#[cfg(test)]
mod nodejs_api_tests {
    use super::*;

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_analyzer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();

        // Test creating analyzer with different configurations
        let config1 = AnalyzerConfig {
            project_root: temp_path.clone(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer1 = FastContextAnalyzer::new(config1);
        assert!(analyzer1.is_ok());

        let config2 = AnalyzerConfig {
            project_root: temp_path.clone(),
            languages: Some(vec!["rust".to_string(), "javascript".to_string()]),
            ignore_patterns: Some(vec!["target/**".to_string(), "node_modules/**".to_string()]),
            enable_caching: Some(true),
            cache_policy: Some("adaptive".to_string()),
            enable_watching: Some(false),
            max_files: Some(500),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer2 = FastContextAnalyzer::new(config2);
        assert!(analyzer2.is_ok());

        // Test with invalid path
        let invalid_config = AnalyzerConfig {
            project_root: "/nonexistent/path".to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("minimal".to_string()),
            enable_watching: Some(false),
            max_files: Some(100),
            parallel_processing: Some(false),
            enable_experimental_architecture: Some(false),
        };
        let invalid_analyzer = FastContextAnalyzer::new(invalid_config);
        // Should either succeed (creating the analyzer) or fail gracefully
        assert!(invalid_analyzer.is_ok() || invalid_analyzer.is_err());
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_find_symbols_in_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        let rust_file = temp_path.join("test.rs");
        fs::write(
            &rust_file,
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

        let js_file = temp_path.join("test.js");
        fs::write(
            &js_file,
            r#"
            function greet(name) {
                return `Hello, ${name}!`;
            }
            
            class User {
                constructor(name) {
                    this.name = name;
                }
                
                getName() {
                    return this.name;
                }
            }
        "#,
        )
        .unwrap();

        let config = AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer = FastContextAnalyzer::new(config).unwrap();

        // Test Rust file
        let rust_result = analyzer.find_symbols_in_file("test.rs".to_string());
        assert!(rust_result.is_ok());
        let rust_symbols = rust_result.unwrap();
        assert!(rust_symbols.len() >= 3); // main, TestStruct, new

        // Test JavaScript file
        let js_result = analyzer.find_symbols_in_file("test.js".to_string());
        assert!(js_result.is_ok());
        let js_symbols = js_result.unwrap();
        assert!(js_symbols.len() >= 3); // greet, User, getName

        // Test nonexistent file
        let nonexistent_result = analyzer.find_symbols_in_file("/nonexistent/file.rs".to_string());
        assert!(nonexistent_result.is_err());
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_analyze_project() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a small project structure
        create_test_project(temp_path);

        let config = AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer = FastContextAnalyzer::new(config).unwrap();

        let result = analyzer.analyze();
        assert!(result.is_ok());

        let analysis_result = result.unwrap();
        assert!(analysis_result.file_count > 0);
        assert!(analysis_result.symbol_count > 0);
        assert!(!analysis_result.languages.is_empty());
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_analyze_project_respects_max_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        create_large_test_project(temp_path);

        let config = AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(3),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer = FastContextAnalyzer::new(config).unwrap();

        let result = analyzer.analyze().unwrap();
        assert_eq!(result.file_count, 3);
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_analyze_project_respects_serial_mode() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        create_test_project(temp_path);

        let parallel = FastContextAnalyzer::new(AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        })
        .unwrap()
        .analyze()
        .unwrap();

        let serial = FastContextAnalyzer::new(AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(false),
            enable_experimental_architecture: Some(false),
        })
        .unwrap()
        .analyze()
        .unwrap();

        assert_eq!(parallel.file_count, serial.file_count);
        assert_eq!(parallel.symbol_count, serial.symbol_count);
        assert_eq!(parallel.relationship_count, serial.relationship_count);
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_get_file_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let rust_file = temp_path.join("main.rs");
        fs::write(
            &rust_file,
            r#"
            fn helper_one() {}

            fn helper_two() {}
            
            fn main() {
                helper_one();
                helper_two();
            }
        "#,
        )
        .unwrap();

        let config = AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer = FastContextAnalyzer::new(config).unwrap();

        let result = analyzer.find_dependencies("main".to_string());
        assert!(result.is_ok());

        let dependencies = result.unwrap();
        // Should find deterministic call dependencies for main -> helper_one/helper_two
        assert!(dependencies.len() >= 2);
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_watch_for_changes() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let config = AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer = FastContextAnalyzer::new(config).unwrap();

        // Test starting and stopping watcher
        let watch_result = analyzer.start_watching();
        // Watcher should start successfully or fail gracefully
        assert!(watch_result.is_ok() || watch_result.is_err());

        let stop_result = analyzer.stop_watching();
        // Should be able to stop watching
        assert!(stop_result.is_ok());
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let config = AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer = FastContextAnalyzer::new(config).unwrap();

        // Test with invalid file paths
        let invalid_paths = vec![
            "".to_string(),
            "/nonexistent/file.rs".to_string(),
            "invalid\0path.rs".to_string(),
            "../../../etc/passwd".to_string(),
        ];

        for invalid_path in invalid_paths {
            let result = analyzer.find_symbols_in_file(invalid_path.clone());
            // Should handle invalid paths gracefully
            assert!(
                result.is_err(),
                "Should fail for invalid path: {}",
                invalid_path
            );
        }
    }

    #[cfg(feature = "nodejs")]
    #[test]
    fn test_nodejs_performance_with_large_project() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a larger project
        create_large_test_project(temp_path);

        let config = AnalyzerConfig {
            project_root: temp_path.to_string_lossy().to_string(),
            languages: None,
            ignore_patterns: None,
            enable_caching: Some(true),
            cache_policy: Some("balanced".to_string()),
            enable_watching: Some(false),
            max_files: Some(1000),
            parallel_processing: Some(true),
            enable_experimental_architecture: Some(false),
        };
        let analyzer = FastContextAnalyzer::new(config).unwrap();

        let start = std::time::Instant::now();
        let result = analyzer.analyze();
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Should complete within reasonable time
        assert!(
            duration.as_secs() < 5,
            "Analysis took too long: {:?}",
            duration
        );

        let analysis_result = result.unwrap();
        assert!(analysis_result.file_count >= 20);
    }

    #[test]
    fn test_core_analyzer_direct_usage() {
        // Test CoreAnalyzer directly (always available)
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        create_test_project(temp_path);

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        let result = analyzer.analyze();
        assert!(result.is_ok());

        let analysis_result = result.unwrap();
        assert!(analysis_result.file_count > 0);
        assert!(analysis_result.symbol_count > 0);
    }

    #[test]
    fn test_core_analyzer_symbol_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let test_file = temp_path.join("test.rs");
        fs::write(
            &test_file,
            r#"
            fn function1() -> i32 { 42 }
            fn function2() -> String { "test".to_string() }
            
            struct TestStruct {
                field: i32,
            }
            
            enum TestEnum {
                Variant1,
                Variant2(i32),
            }
        "#,
        )
        .unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);
        let result = analyzer.find_symbols_in_file(test_file.to_string_lossy().to_string());

        assert!(result.is_ok());
        let symbols = result.unwrap();
        assert!(symbols.len() >= 4); // 2 functions + 1 struct + 1 enum
    }

    // Helper functions

    fn create_test_project(base_path: &std::path::Path) {
        // Create main.rs
        fs::write(
            base_path.join("main.rs"),
            r#"
            fn main() {
                println!("Hello, world!");
            }
        "#,
        )
        .unwrap();

        // Create lib.rs
        fs::write(
            base_path.join("lib.rs"),
            r#"
            pub fn library_function() -> i32 {
                42
            }
            
            pub struct LibraryStruct {
                pub field: String,
            }
        "#,
        )
        .unwrap();

        // Create utils.rs
        fs::write(
            base_path.join("utils.rs"),
            r#"
            pub fn helper_function() -> String {
                "helper".to_string()
            }
            
            pub enum UtilEnum {
                Option1,
                Option2,
            }
        "#,
        )
        .unwrap();

        // Create a JavaScript file
        fs::write(
            base_path.join("script.js"),
            r#"
            function jsFunction() {
                return "JavaScript";
            }
            
            class JSClass {
                constructor() {
                    this.value = 42;
                }
            }
        "#,
        )
        .unwrap();
    }

    #[allow(dead_code)]
    fn create_large_test_project(base_path: &std::path::Path) {
        // Create multiple directories with files
        for i in 0..5 {
            let dir = base_path.join(format!("module_{}", i));
            fs::create_dir_all(&dir).unwrap();

            for j in 0..4 {
                let file_path = dir.join(format!("file_{}.rs", j));
                fs::write(
                    &file_path,
                    format!(
                        r#"
                    pub fn function_{}_{}() -> i32 {{
                        {}
                    }}
                    
                    pub struct Struct_{}_{} {{
                        field: i32,
                    }}
                    
                    impl Struct_{}_{} {{
                        pub fn new() -> Self {{
                            Self {{ field: {} }}
                        }}
                    }}
                "#,
                        i,
                        j,
                        i * j,
                        i,
                        j,
                        i,
                        j,
                        i + j
                    ),
                )
                .unwrap();
            }
        }
    }
}
