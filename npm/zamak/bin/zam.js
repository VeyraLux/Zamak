#!/usr/bin/env node
// The `zam` command installed by npm, npx, and bunx. It runs the native compiler from the
// package that npm picked for this platform, and handles `zam upgrade` itself, so the
// compiler isn't running while npm replaces it.
'use strict';
const { spawnSync } = require('child_process');
const path = require('path');

const PACKAGES = {
  'win32-x64': 'zamak-win32-x64',
  'linux-x64': 'zamak-linux-x64',
  'darwin-arm64': 'zamak-darwin-arm64',
  'linux-arm64': 'zamak-linux-arm64',
  'darwin-x64': 'zamak-darwin-x64',
};

function fail(message) {
  process.stderr.write(`zam: ${message}\n`);
  process.exit(1);
}

function binary() {
  const platform = `${process.platform}-${process.arch}`;
  const name = PACKAGES[platform];
  if (!name) {
    fail(`no prebuilt compiler for ${platform}. Build from source: cargo install --git https://github.com/VeyraLux/Zamak zam --locked`);
  }
  try {
    return require.resolve(`${name}/bin/${process.platform === 'win32' ? 'zam.exe' : 'zam'}`);
  } catch {
    fail(`the ${name} package is missing. Reinstall with: npm install -g @veyralux/zamak`);
  }
}

// `zam upgrade`: update with the package manager that installed Zamak.
function upgrade() {
  const here = __dirname.split(path.sep);
  let command;
  if (here.includes('_npx')) {
    process.stdout.write('zam: npx already runs the version you ask for. Use npx @veyralux/zamak@latest for the newest release.\n');
    return 0;
  } else if (here.includes('.bun')) {
    command = ['bun', ['add', '-g', '@veyralux/zamak@latest']];
  } else {
    command = ['npm', ['install', '-g', '@veyralux/zamak@latest']];
  }
  const [program, args] = command;
  process.stdout.write(`zam: ${program} ${args.join(' ')}\n`);
  if (process.env.ZAMAK_UPGRADE_DRY_RUN) {
    return 0;
  }
  const result = spawnSync(program, args, { stdio: 'inherit', shell: process.platform === 'win32' });
  if (result.error) {
    fail(`couldn't run ${program}: ${result.error.message}`);
  }
  return result.status === null ? 1 : result.status;
}

const args = process.argv.slice(2);
if (args.length === 1 && args[0] === 'upgrade') {
  process.exit(upgrade());
}
const result = spawnSync(binary(), args, { stdio: 'inherit' });
if (result.error) {
  fail(result.error.message);
}
if (result.signal) {
  process.kill(process.pid, result.signal);
}
process.exit(result.status === null ? 1 : result.status);
