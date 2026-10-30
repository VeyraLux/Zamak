// Checks that the extension finds the native zam behind npm's wrappers, the install
// scripts, and an explicit zamak.path. Runs in CI: node editors/vscode/scripts/check-locate.mjs
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { locate } = require('../locate.js');

const root = fs.mkdtempSync(path.join(os.tmpdir(), 'zamak-locate-'));
const home = path.join(root, 'home');
fs.mkdirSync(home);

function write(file, text = '') {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, text);
  return file;
}

// The platform package that npm installs next to @veyralux/zamak.
function npmPackage(modules, platformPackage, binary) {
  write(path.join(modules, '@veyralux', 'zamak', 'package.json'), '{"name": "@veyralux/zamak"}');
  write(path.join(modules, '@veyralux', 'zamak', 'bin', 'zam.js'), '#!/usr/bin/env node\n');
  write(path.join(modules, platformPackage, 'package.json'), `{"name": "${platformPackage}"}`);
  return write(path.join(modules, platformPackage, 'bin', binary), 'native');
}

const windows = { platform: 'win32', arch: 'x64', home };

// 1. npm on Windows: the prefix has zam.cmd and node_modules, and the native zam.exe sits in
//    the platform package. The extension must start zam.exe, never zam.cmd.
{
  const prefix = path.join(root, 'npm-windows');
  write(path.join(prefix, 'zam.cmd'), '@node "%~dp0\\node_modules\\@veyralux\\zamak\\bin\\zam.js" %*');
  const native = npmPackage(path.join(prefix, 'node_modules'), 'zamak-win32-x64', 'zam.exe');
  const found = locate({ ...windows, env: { PATH: prefix } });
  assert.deepEqual(found, { command: native, from: 'PATH' });
}

// 2. A native zam.exe on PATH, such as the one install.ps1 or cargo puts there.
{
  const dir = path.join(root, 'native-windows');
  const native = write(path.join(dir, 'zam.exe'), 'native');
  assert.deepEqual(locate({ ...windows, env: { Path: dir } }), { command: native, from: 'PATH' });
}

// 3. Installed with npm after VS Code started: PATH doesn't have it yet, but `npm root -g` does.
{
  const modules = path.join(root, 'npm-root', 'node_modules');
  const native = npmPackage(modules, 'zamak-win32-x64', 'zam.exe');
  assert.deepEqual(locate({ ...windows, env: { PATH: '' }, npmRoot: modules }), {
    command: native,
    from: 'npm',
  });
}

// 4. install.ps1's default folder, which isn't on PATH until a new terminal opens.
{
  const local = path.join(root, 'localappdata');
  const native = write(path.join(local, 'zamak', 'bin', 'zam.exe'), 'native');
  const found = locate({ ...windows, env: { PATH: '', LOCALAPPDATA: local } });
  assert.deepEqual(found, { command: native, from: path.join(local, 'zamak', 'bin') });
}

// 5. zamak.path set to npm's wrapper still starts the native program.
{
  const prefix = path.join(root, 'npm-windows');
  const found = locate({ ...windows, env: { PATH: '' }, configured: path.join(prefix, 'zam.cmd') });
  assert.equal(found.command, path.join(prefix, 'node_modules', 'zamak-win32-x64', 'bin', 'zam.exe'));
}

// 6. zamak.path set to a native program is used as it is.
assert.deepEqual(locate({ ...windows, env: { PATH: '' }, configured: 'D:\\tools\\zam.exe' }), {
  command: 'D:\\tools\\zam.exe',
  from: 'zamak.path',
});

// 7. A bare name, such as the old default "zam", isn't a location: the extension still searches.
{
  const prefix = path.join(root, 'npm-windows');
  const found = locate({ ...windows, env: { PATH: prefix }, configured: 'zam' });
  assert.equal(found.from, 'PATH');
}

// 8. Not installed anywhere.
assert.equal(locate({ ...windows, env: { PATH: path.join(root, 'empty') } }), undefined);

// 9. npm on Linux and macOS: bin/zam is a link to the package's zam.js.
{
  const prefix = path.join(root, 'npm-unix');
  const native = npmPackage(path.join(prefix, 'lib', 'node_modules'), 'zamak-linux-x64', 'zam');
  const link = path.join(prefix, 'bin', 'zam');
  fs.mkdirSync(path.dirname(link), { recursive: true });
  let linked = true;
  try {
    fs.symlinkSync(path.join(prefix, 'lib', 'node_modules', '@veyralux', 'zamak', 'bin', 'zam.js'), link);
  } catch {
    // Windows without Developer Mode can't create links; CI runs this case on Linux.
    linked = false;
  }
  if (linked) {
    const found = locate({ platform: 'linux', arch: 'x64', home, env: { PATH: path.join(prefix, 'bin') } });
    assert.deepEqual(found, { command: fs.realpathSync(native), from: 'PATH' });
  }
  console.log(linked ? 'unix npm link: checked' : 'unix npm link: skipped (no symlinks here)');
}

fs.rmSync(root, { recursive: true, force: true });
console.log('locate: all cases pass');
