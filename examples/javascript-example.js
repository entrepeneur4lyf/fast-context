const {
  FastContextAnalyzer,
  getSupportedLanguages,
  getVersion,
} = require('../index.js')

function main() {
  const analyzer = new FastContextAnalyzer({
    projectRoot: process.cwd(),
    ignorePatterns: ['node_modules/**', '.git/**', 'target/**'],
    enableWatching: false,
    parallelProcessing: true,
  })

  const result = analyzer.analyze()

  console.log(`fast-context ${getVersion()}`)
  console.log(`supported languages: ${getSupportedLanguages().length}`)
  console.log(
    JSON.stringify(
      {
        fileCount: result.fileCount,
        symbolCount: result.symbolCount,
        relationshipCount: result.relationshipCount,
        durationMs: result.durationMs,
        languages: result.languages,
        skippedFileCount: result.skippedFileCount,
        skippedFiles: result.skippedFiles.slice(0, 3),
      },
      null,
      2
    )
  )
}

main()
