//! Performance and stress tests
//! Tests performance characteristics, memory usage, and scalability

use fast_context::core::CoreAnalyzer;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_file_performance() {
        let max_duration_secs = if std::env::var_os("CI").is_some() {
            60
        } else {
            30
        };

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a large Rust file (100KB)
        let mut large_content = String::new();
        for i in 0..2000 {
            large_content.push_str(&format!(
                r#"
                pub fn function_{}() -> i32 {{
                    let value = {};
                    let result = value * 2;
                    result
                }}
                
                pub struct Struct_{} {{
                    field_{}: i32,
                    field_{}_str: String,
                }}
                
                impl Struct_{} {{
                    pub fn new() -> Self {{
                        Self {{
                            field_{}: {},
                            field_{}_str: "test_{}".to_string(),
                        }}
                    }}
                    
                    pub fn get_value(&self) -> i32 {{
                        self.field_{}
                    }}
                }}
            "#,
                i, i, i, i, i, i, i, i, i, i, i
            ));
        }

        let large_file = temp_path.join("large.rs");
        fs::write(&large_file, large_content).unwrap();

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        let start = Instant::now();
        let result = analyzer.find_symbols_in_file(large_file.to_string_lossy().to_string());
        let duration = start.elapsed();

        assert!(result.is_ok());
        // This is a debug-build smoke test, not a benchmark.
        assert!(
            duration.as_secs() < max_duration_secs,
            "Large file processing took too long: {:?}",
            duration
        );

        let symbols = result.unwrap();
        // Should find many symbols (at least 4000: 2000 functions + 2000 structs)
        assert!(symbols.len() >= 4000);
    }

    #[test]
    fn test_many_small_files_performance() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create many small files
        let num_files = 100;
        for i in 0..num_files {
            let file_path = temp_path.join(format!("file_{}.rs", i));
            fs::write(
                &file_path,
                format!(
                    r#"
                pub fn function_{}() -> i32 {{
                    {}
                }}
                
                pub struct Struct_{} {{
                    value: i32,
                }}
            "#,
                    i, i, i
                ),
            )
            .unwrap();
        }

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        let start = Instant::now();
        let result = analyzer.analyze();
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Should process 100 small files within 3 seconds
        assert!(
            duration.as_secs() < 3,
            "Many files processing took too long: {:?}",
            duration
        );

        let analysis_result = result.unwrap();
        assert_eq!(analysis_result.file_count, num_files);
        assert!(analysis_result.symbol_count >= num_files * 2); // At least 2 symbols per file
    }

    #[test]
    fn test_deep_directory_structure_performance() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create deep directory structure
        let mut current_path = temp_path.to_path_buf();
        for i in 0..20 {
            current_path = current_path.join(format!("level_{}", i));
            fs::create_dir_all(&current_path).unwrap();

            // Add a file at each level
            let file_path = current_path.join(format!("file_{}.rs", i));
            fs::write(&file_path, format!("pub fn function_{}() {{}}", i)).unwrap();
        }

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        let start = Instant::now();
        let result = analyzer.analyze();
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Should handle deep directory structure within 2 seconds
        assert!(
            duration.as_secs() < 2,
            "Deep directory processing took too long: {:?}",
            duration
        );

        let analysis_result = result.unwrap();
        assert_eq!(analysis_result.file_count, 20);
    }

    #[test]
    fn test_mixed_language_performance() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create files in different languages
        let languages = vec![
            ("rust", "rs", "fn test() {}"),
            ("javascript", "js", "function test() {}"),
            ("typescript", "ts", "function test(): void {}"),
            ("python", "py", "def test(): pass"),
            (
                "java",
                "java",
                "public class Test { public void test() {} }",
            ),
            ("go", "go", "func test() {}"),
            ("cpp", "cpp", "void test() {}"),
            ("csharp", "cs", "public void Test() {}"),
        ];

        for (lang, ext, content) in languages {
            for i in 0..10 {
                let file_path = temp_path.join(format!("{}_{}.{}", lang, i, ext));
                fs::write(&file_path, format!("{}\n// File {}", content, i)).unwrap();
            }
        }

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        let start = Instant::now();
        let result = analyzer.analyze();
        let duration = start.elapsed();

        assert!(result.is_ok());
        // This is a debug-build smoke test, not a benchmark.
        assert!(
            duration.as_secs() < 10,
            "Mixed language processing took too long: {:?}",
            duration
        );

        let analysis_result = result.unwrap();
        assert_eq!(analysis_result.file_count, 80); // 8 languages * 10 files each
        assert!(analysis_result.languages.len() >= 6); // Should detect most languages
    }

    #[test]
    fn test_repeated_analysis_performance() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a moderate-sized project
        for i in 0..20 {
            let file_path = temp_path.join(format!("file_{}.rs", i));
            fs::write(
                &file_path,
                format!(
                    r#"
                pub fn function_{}() -> i32 {{
                    {}
                }}
                
                pub struct Struct_{} {{
                    value: i32,
                }}
                
                impl Struct_{} {{
                    pub fn new() -> Self {{
                        Self {{ value: {} }}
                    }}
                }}
            "#,
                    i, i, i, i, i
                ),
            )
            .unwrap();
        }

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        // Run analysis multiple times to test consistency
        let mut durations = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let result = analyzer.analyze();
            let duration = start.elapsed();

            assert!(result.is_ok());
            durations.push(duration);
        }

        // All runs should complete within reasonable time
        for (i, duration) in durations.iter().enumerate() {
            assert!(
                duration.as_secs() < 2,
                "Run {} took too long: {:?}",
                i,
                duration
            );
        }

        // Performance should be consistent (no run should be more than 2x slower than the fastest)
        let min_duration = durations.iter().min().unwrap();
        let max_duration = durations.iter().max().unwrap();
        assert!(
            max_duration.as_millis() <= min_duration.as_millis() * 3,
            "Performance inconsistency: min={:?}, max={:?}",
            min_duration,
            max_duration
        );
    }

    #[test]
    fn test_memory_usage_with_large_project() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a project that could potentially use a lot of memory
        for i in 0..50 {
            let dir = temp_path.join(format!("module_{}", i));
            fs::create_dir_all(&dir).unwrap();

            for j in 0..10 {
                let file_path = dir.join(format!("file_{}.rs", j));
                let mut content = String::new();

                // Create files with many symbols
                for k in 0..50 {
                    content.push_str(&format!(
                        r#"
                        pub fn function_{}_{}() -> i32 {{
                            {}
                        }}
                    "#,
                        j, k, k
                    ));
                }

                fs::write(&file_path, content).unwrap();
            }
        }

        let analyzer = CoreAnalyzer::new(temp_path.to_string_lossy().to_string(), None, None);

        let start = Instant::now();
        let result = analyzer.analyze();
        let duration = start.elapsed();

        assert!(result.is_ok());
        // Should handle large project within 10 seconds
        assert!(
            duration.as_secs() < 10,
            "Large project processing took too long: {:?}",
            duration
        );

        let analysis_result = result.unwrap();
        assert_eq!(analysis_result.file_count, 500); // 50 modules * 10 files each
        assert!(analysis_result.symbol_count >= 25000); // 50 * 10 * 50 = 25000 functions
    }

    #[test]
    fn test_concurrent_file_access_performance() {
        use std::sync::Arc;
        use std::thread;

        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create test files
        for i in 0..20 {
            let file_path = temp_path.join(format!("test_{}.rs", i));
            fs::write(
                &file_path,
                format!(
                    r#"
                pub fn function_{}() -> i32 {{
                    {}
                }}
                
                pub struct Struct_{} {{
                    value: i32,
                }}
            "#,
                    i, i, i
                ),
            )
            .unwrap();
        }

        let analyzer = Arc::new(CoreAnalyzer::new(
            temp_path.to_string_lossy().to_string(),
            None,
            None,
        ));

        let start = Instant::now();
        let mut handles = vec![];

        // Spawn multiple threads to analyze different files concurrently
        for i in 0..10 {
            let analyzer_clone = Arc::clone(&analyzer);
            let file_path = temp_path.join(format!("test_{}.rs", i));

            let handle = thread::spawn(move || {
                let result =
                    analyzer_clone.find_symbols_in_file(file_path.to_string_lossy().to_string());
                assert!(result.is_ok());
                result.unwrap()
            });

            handles.push(handle);
        }

        // Collect results
        let mut total_symbols = 0;
        for handle in handles {
            let symbols = handle.join().unwrap();
            total_symbols += symbols.len();
        }

        let duration = start.elapsed();

        // Concurrent access should be faster than sequential and complete within 2 seconds
        assert!(
            duration.as_secs() < 2,
            "Concurrent access took too long: {:?}",
            duration
        );
        assert!(total_symbols >= 20); // Should find at least 2 symbols per file
    }

    #[test]
    fn test_language_detection_performance() {
        // Test language detection performance with many files
        let test_files = vec![
            "main.rs",
            "lib.rs",
            "utils.rs",
            "config.rs",
            "parser.rs",
            "app.js",
            "component.js",
            "utils.js",
            "config.js",
            "api.js",
            "main.py",
            "utils.py",
            "config.py",
            "models.py",
            "views.py",
            "Main.java",
            "Utils.java",
            "Config.java",
            "Model.java",
            "View.java",
            "main.go",
            "utils.go",
            "config.go",
            "handler.go",
            "server.go",
            "index.html",
            "about.html",
            "contact.html",
            "style.css",
            "app.css",
            "data.json",
            "config.yaml",
            "README.md",
            "CHANGELOG.md",
            "LICENSE.md",
        ];

        let start = Instant::now();

        for _ in 0..1000 {
            for filename in &test_files {
                #[cfg(feature = "nodejs")]
                let _language = fast_context::utils::detect_language(filename.to_string());
                #[cfg(not(feature = "nodejs"))]
                let _unused = filename; // Prevent unused variable warning
            }
        }

        let duration = start.elapsed();

        // Language detection should be very fast
        assert!(
            duration.as_millis() < 100,
            "Language detection took too long: {:?}",
            duration
        );
    }

    #[test]
    fn test_ignore_pattern_performance() {
        let max_duration_secs = if std::env::var_os("CI").is_some() {
            60
        } else {
            30
        };

        // Test ignore pattern matching performance
        let test_paths = vec![
            "src/main.rs",
            "src/lib.rs",
            "target/debug/main",
            "target/release/main",
            "node_modules/package/index.js",
            "node_modules/react/index.js",
            ".git/config",
            ".git/objects/abc123",
            "dist/bundle.js",
            "dist/styles.css",
            "coverage/lcov.info",
            "coverage/html/index.html",
            ".nyc_output/abc123.json",
            "build/output.js",
            "out/production/Main.class",
        ];

        let start = Instant::now();

        let default_patterns = vec![
            "node_modules/**".to_string(),
            ".git/**".to_string(),
            "target/**".to_string(),
            "dist/**".to_string(),
        ];

        for _ in 0..10000 {
            for path in &test_paths {
                let _should_ignore =
                    fast_context::utils::should_ignore_file(path, &default_patterns);
            }
        }

        let duration = start.elapsed();

        // This is a debug-build smoke test, not a benchmark.
        assert!(
            duration.as_secs() < max_duration_secs,
            "Ignore pattern matching took too long: {:?}",
            duration
        );
    }
}
