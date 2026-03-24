import { FastContextAnalyzer, getSupportedLanguages, getVersion } from '../index.js'
import type { AnalysisResultJs, AnalyzerConfig } from '../index.js'

function summarize(result: AnalysisResultJs) {
  return {
    fileCount: result.fileCount,
    symbolCount: result.symbolCount,
    relationshipCount: result.relationshipCount,
    durationMs: result.durationMs,
    languages: result.languages,
    skippedFileCount: result.skippedFileCount,
    skippedFiles: result.skippedFiles.slice(0, 3),
  }
}

function main() {
  const config: AnalyzerConfig = {
    projectRoot: process.cwd(),
    ignorePatterns: ['node_modules/**', '.git/**', 'target/**'],
    enableWatching: false,
    parallelProcessing: true,
  }

  const analyzer = new FastContextAnalyzer(config)
  const result = analyzer.analyze()

  console.log(`fast-context ${getVersion()}`)
  console.log(`supported languages: ${getSupportedLanguages().length}`)
  console.log(JSON.stringify(summarize(result), null, 2))
}

main()
