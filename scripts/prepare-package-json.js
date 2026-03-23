const fs = require('fs')
const path = require('path')

const mode = process.argv[2] || 'local'
const packageJsonPath = path.join(__dirname, '..', 'package.json')
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))

packageJson.optionalDependencies = {
  'fast-context-darwin-arm64': packageJson.version,
  'fast-context-darwin-x64': packageJson.version,
  'fast-context-linux-arm64-gnu': packageJson.version,
  'fast-context-linux-arm64-musl': packageJson.version,
  'fast-context-linux-arm-gnueabihf': packageJson.version,
  'fast-context-linux-x64-gnu': packageJson.version,
  'fast-context-linux-x64-musl': packageJson.version,
  'fast-context-win32-arm64-msvc': packageJson.version,
  'fast-context-win32-x64-msvc': packageJson.version,
}

packageJson.files =
  mode === 'publish'
    ? ['index.js', 'index.d.ts', 'README.md']
    : ['index.js', 'index.d.ts', 'README.md', '*.node']

delete packageJson.dependencies

fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`)
