const fs = require('fs');
const path = require('path');

const bindingPath = path.join(__dirname, '..', 'index.js');
let source = fs.readFileSync(bindingPath, 'utf8');

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

if (source.includes(patched)) {
  process.exit(0);
}

if (!source.includes(original)) {
  throw new Error('Could not find win32 x64 loader block to patch');
}

source = source.replace(original, patched);
fs.writeFileSync(bindingPath, source);
