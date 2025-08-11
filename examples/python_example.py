#!/usr/bin/env python3
"""
Example usage of fast-context Python bindings.

This demonstrates how to use the fast-context library from Python
to analyze codebases with high performance.
"""

import os
import sys
import tempfile
from pathlib import Path

# Add the package to path for development
sys.path.insert(0, str(Path(__file__).parent.parent / "python"))

try:
    from fast_context import (
        FastContextAnalyzer,
        AnalyzerConfig,
        get_supported_languages,
        detect_language,
        get_version,
    )
except ImportError as e:
    print(f"❌ Failed to import fast_context: {e}")
    print("💡 Make sure to build the Python extension first:")
    print("   maturin develop --features python")
    sys.exit(1)


def create_test_project():
    """Create a temporary test project with sample files."""
    temp_dir = tempfile.mkdtemp(prefix="fast_context_test_")
    
    # Create sample Rust file
    rust_file = Path(temp_dir) / "main.rs"
    rust_file.write_text("""
fn main() {
    println!("Hello, world!");
}

pub fn calculate(a: i32, b: i32) -> i32 {
    if a > b {
        a + b
    } else {
        a - b
    }
}

pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn add(&mut self, n: i32) -> &mut Self {
        self.value += n;
        self
    }
}
""")
    
    # Create sample Python file
    python_file = Path(temp_dir) / "example.py"
    python_file.write_text("""
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

class MathHelper:
    def __init__(self):
        self.operations = []
    
    def add_operation(self, op):
        self.operations.append(op)
        return self
    
    def complex_calculation(self, data):
        result = 0
        for item in data:
            if item > 0:
                for i in range(item):
                    if i % 2 == 0:
                        result += i
                    else:
                        result -= i
            elif item < 0:
                result *= abs(item)
        return result
""")
    
    # Create sample JavaScript file
    js_file = Path(temp_dir) / "app.js"
    js_file.write_text("""
function greet(name) {
    return `Hello, ${name}!`;
}

class Person {
    constructor(name, age) {
        this.name = name;
        this.age = age;
    }
    
    speak() {
        return greet(this.name);
    }
    
    isAdult() {
        return this.age >= 18;
    }
}

const person = new Person("Alice", 25);
console.log(person.speak());
""")
    
    return temp_dir


def main():
    """Main example function."""
    print("🐍 Fast-Context Python Bindings Example")
    print("=" * 50)
    
    # Show version and supported languages
    print(f"📦 Version: {get_version()}")
    print(f"🔧 Supported languages: {', '.join(get_supported_languages())}")
    print()
    
    # Test language detection
    print("🔍 Language Detection:")
    test_files = ["main.rs", "app.py", "script.js", "style.css", "README.md"]
    for file in test_files:
        lang = detect_language(file)
        print(f"  {file} -> {lang or 'Unknown'}")
    print()
    
    # Create test project
    print("📁 Creating test project...")
    project_dir = create_test_project()
    print(f"   Created at: {project_dir}")
    
    try:
        # Configure analyzer
        config = AnalyzerConfig(
            project_root=project_dir,
            languages=["rust", "python", "javascript"],
            ignore_patterns=["*.tmp", "*.log"],
            enable_caching=True,
            enable_watching=False,
            max_files=1000,
            parallel_processing=True,
        )
        
        # Create analyzer
        print("🔧 Initializing analyzer...")
        analyzer = FastContextAnalyzer(config)
        
        # Analyze the project
        print("📊 Analyzing codebase...")
        result = analyzer.analyze()
        
        print(f"✅ Analysis complete!")
        print(f"   Files analyzed: {result.file_count}")
        print(f"   Symbols found: {result.symbol_count}")
        print(f"   Relationships: {result.relationship_count}")
        print(f"   Languages: {', '.join(result.languages)}")
        print(f"   Duration: {result.duration_ms}ms")
        if result.memory_usage_mb:
            print(f"   Memory usage: {result.memory_usage_mb:.2f}MB")
        print()
        
        # Find functions
        print("🔍 Finding functions...")
        functions = analyzer.find_symbols_by_kind("function")
        print(f"   Found {len(functions)} functions: {', '.join(functions[:5])}")
        if len(functions) > 5:
            print(f"   ... and {len(functions) - 5} more")
        print()
        
        # Find classes
        print("🔍 Finding classes...")
        classes = analyzer.find_symbols_by_kind("class")
        print(f"   Found {len(classes)} classes: {', '.join(classes)}")
        print()
        
        # Find symbols in specific file
        print("🔍 Finding symbols in main.rs...")
        file_symbols = analyzer.find_symbols_in_file("main.rs")
        print(f"   Found {len(file_symbols)} symbols: {', '.join(file_symbols)}")
        print()
        
        # Find dependencies
        print("🔍 Finding dependencies of 'Calculator'...")
        deps = analyzer.find_dependencies("Calculator")
        print(f"   Found {len(deps)} dependencies: {', '.join(deps)}")
        print()
        
        # Find complex symbols
        print("🔍 Finding complex symbols (complexity > 5)...")
        complex_symbols = analyzer.find_complex_symbols(5)
        print(f"   Found {len(complex_symbols)} complex symbols: {', '.join(complex_symbols)}")
        print()
        
        # Test file watching
        print("👀 Testing file watching...")
        try:
            analyzer.start_watching()
            print("   ✅ File watching started")
            analyzer.stop_watching()
            print("   ✅ File watching stopped")
        except Exception as e:
            print(f"   ⚠️  File watching test failed: {e}")
        
        print("🎉 All tests completed successfully!")
        
    except Exception as e:
        print(f"❌ Error during analysis: {e}")
        return 1
    
    finally:
        # Cleanup
        import shutil
        shutil.rmtree(project_dir, ignore_errors=True)
        print(f"🧹 Cleaned up test project")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
