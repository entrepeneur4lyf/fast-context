import test from 'ava';
import { getSupportedLanguages, detectLanguage, getVersion, checkConfiguration } from '../index.js';

// Unit tests for utility functions that don't require heavy setup

test('getSupportedLanguages returns expected languages', t => {
  const languages = getSupportedLanguages();
  
  t.true(Array.isArray(languages));
  t.true(languages.length > 10); // Should support many languages
  
  // Core languages should be supported (case-insensitive check)
  const coreLanguages = ['rust', 'javascript', 'typescript', 'python', 'java', 'go'];
  const lowerLanguages = languages.map(lang => lang.toLowerCase());
  for (const lang of coreLanguages) {
    t.true(lowerLanguages.includes(lang), `Should include ${lang}`);
  }
});

test('detectLanguage handles common file extensions', t => {
  const testCases = [
    // Rust
    ['main.rs', 'Rust'],
    ['lib.rs', 'Rust'],
    ['mod.rs', 'Rust'],
    
    // JavaScript/TypeScript
    ['app.js', 'JavaScript'],
    ['script.mjs', 'JavaScript'],
    ['types.ts', 'TypeScript'],
    
    // Python
    ['script.py', 'Python'],
    ['module.pyw', 'Python'],
    
    // Java
    ['Main.java', 'Java'],
    ['Application.java', 'Java'],
    
    // Go
    ['main.go', 'Go'],
    ['handler.go', 'Go'],
    
    // C#
    ['Program.cs', 'CSharp'],
    ['Service.cs', 'CSharp'],
    
    // Unknown (detectLanguage returns null for unsupported extensions)
    ['unknown.xyz', null],
    ['', null],
  ];

  for (const [filename, expected] of testCases) {
    const actual = detectLanguage(filename);
    if (expected === null) {
      t.is(actual, expected, `detectLanguage('${filename}') should return null`);
    } else {
      t.is(actual, expected, `detectLanguage('${filename}') should return '${expected}'`);
    }
  }
});

test('detectLanguage handles edge cases', t => {
  // Case sensitivity
  t.is(detectLanguage('MAIN.RS'), 'Rust');
  t.is(detectLanguage('APP.JS'), 'JavaScript');
  
  // Multiple extensions
  t.is(detectLanguage('bundle.min.js'), 'JavaScript');
  t.is(detectLanguage('types.d.ts'), 'TypeScript');
  
  // Special files
  t.is(detectLanguage('Dockerfile'), 'Dockerfile');
  t.is(detectLanguage('Makefile'), 'Makefile');
  
  // Empty or invalid
  t.is(detectLanguage(''), null);
  t.is(detectLanguage('.'), null);
  t.is(detectLanguage('..'), null);
});

test('getVersion returns valid semantic version', t => {
  const version = getVersion();
  
  t.is(typeof version, 'string');
  t.true(version.length > 0);
  
  // Should match semantic versioning pattern
  const semverPattern = /^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?(?:\+[a-zA-Z0-9.-]+)?$/;
  t.regex(version, semverPattern, 'Version should follow semantic versioning');
  
  // Should start with a number
  t.regex(version, /^\d/, 'Version should start with a number');
});

test('checkConfiguration returns status message', t => {
  const status = checkConfiguration();
  
  t.is(typeof status, 'string');
  t.true(status.length > 0);
  
  // Should contain some indication of configuration status
  // (Exact message depends on implementation)
  t.true(status.length > 10, 'Status message should be meaningful');
});

test('Module exports contain expected functions', async t => {
  const module = await import('../index.js');
  
  // Core utility functions
  t.is(typeof module.getSupportedLanguages, 'function');
  t.is(typeof module.detectLanguage, 'function');
  t.is(typeof module.getVersion, 'function');
  t.is(typeof module.checkConfiguration, 'function');
  
  // Main class
  t.is(typeof module.FastContextAnalyzer, 'function');
});

test('FastContextAnalyzer class exists and is constructible', async t => {
  const { FastContextAnalyzer } = await import('../index.js');
  
  t.is(typeof FastContextAnalyzer, 'function');
  
  // Should be constructible with minimal config
  const config = { projectRoot: process.cwd() };
  const analyzer = new FastContextAnalyzer(config);
  
  t.truthy(analyzer);
  t.true(analyzer instanceof FastContextAnalyzer);
});

test('FastContextAnalyzer methods exist', async t => {
  const { FastContextAnalyzer } = await import('../index.js');
  
  const config = { projectRoot: process.cwd() };
  const analyzer = new FastContextAnalyzer(config);
  
  // Core methods should exist
  t.is(typeof analyzer.analyze, 'function');
  t.is(typeof analyzer.startWatching, 'function');
  t.is(typeof analyzer.stopWatching, 'function');
});

test('getSupportedLanguages is consistent', t => {
  // Should return the same result on multiple calls
  const languages1 = getSupportedLanguages();
  const languages2 = getSupportedLanguages();
  
  t.deepEqual(languages1, languages2);
  
  // Languages should be in consistent order (implementation-defined)
  t.true(languages1.length > 0);
  t.true(Array.isArray(languages1));
});

test('detectLanguage performance', t => {
  // Should handle many calls efficiently
  const filenames = [
    'main.rs', 'app.js', 'script.py', 'Main.java', 'main.go',
    'style.css', 'index.html', 'config.json', 'unknown.xyz'
  ];
  
  const start = Date.now();
  
  for (let i = 0; i < 1000; i++) {
    for (const filename of filenames) {
      detectLanguage(filename);
    }
  }
  
  const duration = Date.now() - start;
  
  // Should complete 9000 calls in reasonable time (< 1 second)
  t.true(duration < 1000, `Language detection took ${duration}ms for 9000 calls`);
});