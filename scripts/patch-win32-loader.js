const fs = require('fs');
const path = require('path');

const bindingPath = path.join(__dirname, '..', 'index.js');
let source = fs.readFileSync(bindingPath, 'utf8');

const prologueNeedle = `const { platform, arch } = process

let nativeBinding = null`

const prologuePatch = `const { platform, arch } = process
const supportedPlatforms = new Set([
  'darwin-x64',
  'darwin-arm64',
  'freebsd-x64',
  'linux-x64-gnu',
  'linux-x64-musl',
  'linux-arm64-gnu',
  'linux-arm64-musl',
  'linux-arm-gnueabihf',
  'win32-x64-msvc',
  'win32-arm64-msvc',
])

let nativeBinding = null`

const supportedPlatformNeedle = `function isMusl() {`

const supportedPlatformPatch = `function supportedPlatformKey() {
  switch (platform) {
    case 'win32':
      if (arch === 'x64') return 'win32-x64-msvc'
      if (arch === 'arm64') return 'win32-arm64-msvc'
      return \`\${platform}-\${arch}\`
    case 'darwin':
      return \`\${platform}-\${arch}\`
    case 'freebsd':
      return \`\${platform}-\${arch}\`
    case 'linux':
      if (arch === 'x64') return isMusl() ? 'linux-x64-musl' : 'linux-x64-gnu'
      if (arch === 'arm64') return isMusl() ? 'linux-arm64-musl' : 'linux-arm64-gnu'
      if (arch === 'arm') return isMusl() ? 'linux-arm-musleabihf' : 'linux-arm-gnueabihf'
      return \`\${platform}-\${arch}\`
    default:
      return \`\${platform}-\${arch}\`
  }
}

function isMusl() {`

const original = `      case 'x64':
        localFileExisted = existsSync(
          join(__dirname, 'fast-context.win32-x64-msvc.node')
        )
        try {
          if (localFileExisted) {
            nativeBinding = require('./fast-context.win32-x64-msvc.node')
          } else {
            nativeBinding = require('fast-context-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break`;

const patched = `      case 'x64':
        const localMsvcFile = join(__dirname, 'fast-context.win32-x64-msvc.node')
        const localGnuFile = join(__dirname, 'fast-context.win32-x64-gnu.node')
        localFileExisted = existsSync(localMsvcFile) || existsSync(localGnuFile)
        try {
          if (existsSync(localMsvcFile)) {
            nativeBinding = require('./fast-context.win32-x64-msvc.node')
          } else if (existsSync(localGnuFile)) {
            nativeBinding = require('./fast-context.win32-x64-gnu.node')
          } else {
            nativeBinding = require('fast-context-win32-x64-msvc')
          }
        } catch (e) {
          loadError = e
        }
        break`;

const unsupportedNeedle = `if (!nativeBinding) {
  if (loadError) {
    throw loadError
  }
  throw new Error(\`Failed to load native binding\`)
}`

const unsupportedPatch = `if (!nativeBinding) {
  const platformKey = supportedPlatformKey()
  if (!supportedPlatforms.has(platformKey)) {
    throw new Error(
      \`Unsupported platform: \${platformKey}. Supported Node.js release targets: \${Array.from(
        supportedPlatforms
      ).join(', ')}\`
    )
  }
  if (loadError) {
    throw loadError
  }
  throw new Error(\`Failed to load native binding\`)
}`

if (!source.includes(prologuePatch)) {
  if (!source.includes(prologueNeedle)) {
    throw new Error('Could not find loader prologue to patch');
  }
  source = source.replace(prologueNeedle, prologuePatch);
}

if (!source.includes(patched)) {
  if (!source.includes(original)) {
    throw new Error('Could not find win32 x64 loader block to patch');
  }
  source = source.replace(original, patched);
}

if (!source.includes(supportedPlatformPatch)) {
  if (!source.includes(supportedPlatformNeedle)) {
    throw new Error('Could not find platform helper insertion point');
  }
  source = source.replace(supportedPlatformNeedle, supportedPlatformPatch);
}

if (!source.includes(unsupportedPatch)) {
  if (!source.includes(unsupportedNeedle)) {
    throw new Error('Could not find unsupported platform block to patch');
  }
  source = source.replace(unsupportedNeedle, unsupportedPatch);
}

fs.writeFileSync(bindingPath, source);
