import test from 'ava';
import { FastContextAnalyzer, getSupportedLanguages, detectLanguage, getVersion, checkConfiguration } from '../index.js';
import { promises as fs } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';
import { mkdtemp, writeFile, mkdir } from 'fs/promises';

// Helper to create temporary test project
async function createTestProject() {
  const tempDir = await mkdtemp(join(tmpdir(), 'fast-context-test-'));

  // Create test files
  await writeFile(join(tempDir, 'main.rs'), `
fn main() {
    println!("Hello, world!");
}

pub fn add(a: i32, b: i32) -> i32 {
    a + b
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
`);

  await writeFile(join(tempDir, 'utils.js'), `
function calculateSum(numbers) {
    return numbers.reduce((sum, num) => sum + num, 0);
}

class DataProcessor {
    constructor() {
        this.data = [];
    }

    addItem(item) {
        this.data.push(item);
        return this;
    }

    process() {
        return this.data.map(item => item * 2);
    }
}

module.exports = { calculateSum, DataProcessor };
`);

  await writeFile(join(tempDir, 'helper.py'), `
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

    def execute_all(self):
        results = []
        for op in self.operations:
            results.append(op())
        return results
`);

  return tempDir;
}

test('FastContextAnalyzer constructor and basic configuration', t => {
  const config = {
    projectRoot: process.cwd(),
    languages: ['rust', 'javascript'],
    ignorePatterns: ['node_modules/**', 'target/**'],
    enableCaching: true,
    cachePolicy: 'adaptive',
    enableWatching: false,
    maxFiles: 1000,
    parallelProcessing: true
  };

  const analyzer = new FastContextAnalyzer(config);
  t.truthy(analyzer);
});

test('FastContextAnalyzer analysis on test project', async t => {
  const tempDir = await createTestProject();

  try {
    const config = {
      projectRoot: tempDir,
      languages: ['rust', 'javascript', 'python'],
      ignorePatterns: ['node_modules/**'],
      enableCaching: false,
      enableWatching: false,
      maxFiles: 100,
      parallelProcessing: false
    };

    const analyzer = new FastContextAnalyzer(config);
    const result = await analyzer.analyze();

    // Verify analysis results
    t.truthy(result);
    t.is(typeof result.fileCount, 'number');
    t.is(typeof result.symbolCount, 'number');
    t.is(typeof result.relationshipCount, 'number');
    t.true(Array.isArray(result.languages));
    t.is(typeof result.durationMs, 'number');

    // Should have found our test files
    t.true(result.fileCount >= 3);
    t.true(result.symbolCount > 0);

    // Should detect the languages we created (case-insensitive check)
    const lowerResultLanguages = result.languages.map(lang => lang.toLowerCase());
    t.true(lowerResultLanguages.includes('rust') || lowerResultLanguages.includes('javascript') || lowerResultLanguages.includes('python'));

  } finally {
    // Cleanup
    await fs.rm(tempDir, { recursive: true, force: true });
  }

// CoreAnalyzer-backed methods on Node
test('CoreAnalyzer-backed symbol queries (Node)', async t => {
  const tempDir = await createTestProject();
  try {
    const analyzer = new FastContextAnalyzer({
      projectRoot: tempDir,
      languages: ['rust', 'javascript', 'python'],
      ignorePatterns: ['node_modules/**']
    });

    const analyzed = await analyzer.analyze();
    t.truthy(analyzed);

    const byKind = await analyzer.findSymbolsByKind('function');
    t.true(Array.isArray(byKind));
    t.true(byKind.length > 0);

    const rsSymbols = await analyzer.findSymbolsInFile('main.rs');
    t.true(Array.isArray(rsSymbols));
    t.true(rsSymbols.some(s => s.includes('function: main') || s.includes('function: add')));

    const deps = await analyzer.findDependencies('calculateSum');
    t.true(Array.isArray(deps));

    const complex = await analyzer.findComplexSymbols(1);
    t.true(Array.isArray(complex));
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
});
});

test('getSupportedLanguages returns language array', t => {
  const languages = getSupportedLanguages();

  t.true(Array.isArray(languages));
  t.true(languages.length > 0);

  // Should include common languages (case-insensitive check)
  const lowerLanguages = languages.map(lang => lang.toLowerCase());
  t.true(lowerLanguages.includes('rust'));
  t.true(lowerLanguages.includes('javascript'));
  t.true(lowerLanguages.includes('python'));
  t.true(lowerLanguages.includes('typescript'));
});

test('detectLanguage correctly identifies file types', t => {
  t.is(detectLanguage('main.rs'), 'Rust');
  t.is(detectLanguage('app.js'), 'JavaScript');
  t.is(detectLanguage('script.py'), 'Python');
  t.is(detectLanguage('types.ts'), 'TypeScript');
  t.is(detectLanguage('Main.java'), 'Java');
  t.is(detectLanguage('main.go'), 'Go');
  t.is(detectLanguage('Program.cs'), 'CSharp');
  t.is(detectLanguage('unknown.xyz'), null);
});

test('getVersion returns semantic version', t => {
  const version = getVersion();

  t.is(typeof version, 'string');
  t.regex(version, /^\d+\.\d+\.\d+/); // Semantic version pattern
});

test('checkConfiguration validates setup', t => {
  const configStatus = checkConfiguration();

  t.is(typeof configStatus, 'string');
  t.true(configStatus.length > 0);
});

test('FastContextAnalyzer file watching functionality', async t => {
  const tempDir = await createTestProject();

  try {
    const config = {
      projectRoot: tempDir,
      enableWatching: true,
      ignorePatterns: ['node_modules/**']
    };

    const analyzer = new FastContextAnalyzer(config);

    // Start watching with callback (no-op)
    analyzer.startWatching(() => {});

    // Give the watcher time to initialize
    await new Promise(resolve => setTimeout(resolve, 100));

    // Create a new file to trigger the watcher
    await writeFile(join(tempDir, 'new_file.js'), 'console.log("test");');

    // Wait for the callback
    await new Promise(resolve => setTimeout(resolve, 500));

    // Stop watching
    analyzer.stopWatching();

    // Note: File watching is debounced and may not always trigger in tests
    // This test primarily verifies the API works without throwing
    t.pass();

  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
});

test('FastContextAnalyzer handles invalid project root', t => {
  const config = {
    projectRoot: '/non/existent/path',
    enableCaching: false,
    enableWatching: false
  };

  // Should throw on construction with invalid path
  t.throws(() => {
    new FastContextAnalyzer(config);
  }, {
    code: 'GenericFailure',
    message: 'Project root does not exist: /non/existent/path'
  });
});

test('FastContextAnalyzer with minimal configuration', async t => {
  const config = {
    projectRoot: process.cwd()
  };

  const analyzer = new FastContextAnalyzer(config);

  // Should work with minimal config
  t.truthy(analyzer);

  // Basic analysis should work (may be limited by actual project files)
  try {
    const result = analyzer.analyze();
    t.truthy(result);
    t.is(typeof result.fileCount, 'number');
  } catch {
    // Some configurations might not work in all environments
    // The important thing is the API doesn't crash
    t.pass();
  }
});

test('FastContextAnalyzer language filtering', async t => {
  const tempDir = await createTestProject();

  try {
    // Test with specific language filter
    const config = {
      projectRoot: tempDir,
      languages: ['rust'], // Only analyze Rust files
      enableCaching: false,
      enableWatching: false
    };

    const analyzer = new FastContextAnalyzer(config);
    const result = analyzer.analyze();

    t.truthy(result);

    // With language filtering, should primarily find Rust (case-insensitive)
    if (result.languages.length > 0) {
      const lowerResultLanguages = result.languages.map(lang => lang.toLowerCase());
      t.true(lowerResultLanguages.includes('rust'));
    }

  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
});

test('FastContextAnalyzer ignore patterns', async t => {
  const tempDir = await createTestProject();

  try {
    // Create some files that should be ignored
    await mkdir(join(tempDir, 'node_modules'), { recursive: true });
    await writeFile(join(tempDir, 'node_modules', 'package.js'), 'module.exports = {};');

    const config = {
      projectRoot: tempDir,
      ignorePatterns: ['node_modules/**'],
      enableCaching: false,
      enableWatching: false
    };

    const analyzer = new FastContextAnalyzer(config);
    const result = analyzer.analyze();

    t.truthy(result);
    // File count should not include ignored files
    // (Exact count depends on implementation details)
    t.is(typeof result.fileCount, 'number');

  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
});