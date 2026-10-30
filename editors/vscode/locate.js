// 找到真正的 zam 可执行文件。npm 装的 `zam` 是包装：Windows 上是 zam.cmd，其他系统是指向 zam.js
// 的链接。VS Code 的扩展进程启动不了 .cmd（Node 会报 EINVAL），zam.js 又要求 PATH 上有 node，
// 所以这里顺着包装找到 npm 为本平台装的原生 zam，直接启动它。
'use strict';
const fs = require('fs');
const os = require('os');
const path = require('path');

const PACKAGES = {
  'win32-x64': 'zamak-win32-x64',
  'linux-x64': 'zamak-linux-x64',
  'darwin-arm64': 'zamak-darwin-arm64',
  'linux-arm64': 'zamak-linux-arm64',
  'darwin-x64': 'zamak-darwin-x64',
};

function isFile(file) {
  try {
    return fs.statSync(file).isFile();
  } catch {
    return false;
  }
}

// 以 `#!` 开头的是脚本（npm 的 zam.js），不是原生程序。
function isScript(file) {
  try {
    const fd = fs.openSync(file, 'r');
    const head = Buffer.alloc(2);
    fs.readSync(fd, head, 0, 2, 0);
    fs.closeSync(fd);
    return head.toString() === '#!';
  } catch {
    return false;
  }
}

// `@veyralux/zamak` 包所在目录 → 它为本平台装的原生 zam（和 bin/zam.js 的找法一样）。
function fromPackage(packageDir, system) {
  const name = PACKAGES[`${system.platform}-${system.arch}`];
  if (!name || !isFile(path.join(packageDir, 'package.json'))) return undefined;
  const binary = system.platform === 'win32' ? 'zam.exe' : 'zam';
  try {
    return require.resolve(`${name}/bin/${binary}`, { paths: [packageDir] });
  } catch {
    return undefined;
  }
}

// PATH 里的一个目录：有原生 zam 就用它；只有 npm 的包装就顺着找到原生的那个。
function fromDirectory(dir, system) {
  if (system.platform === 'win32') {
    const native = path.join(dir, 'zam.exe');
    if (isFile(native)) return native;
    if (isFile(path.join(dir, 'zam.cmd'))) {
      return fromPackage(path.join(dir, 'node_modules', '@veyralux', 'zamak'), system);
    }
    return undefined;
  }
  const file = path.join(dir, 'zam');
  if (!isFile(file)) return undefined;
  if (!isScript(file)) return file;
  // bin/zam → …/node_modules/@veyralux/zamak/bin/zam.js
  let target;
  try {
    target = fs.realpathSync(file);
  } catch {
    return undefined;
  }
  return fromPackage(path.dirname(path.dirname(target)), system);
}

// 安装脚本、Cargo、npm 的常见位置。VS Code 从程序坞或开始菜单打开时，PATH 可能没有这些目录。
function usualDirectories(env, system) {
  const home = system.home;
  if (system.platform === 'win32') {
    return [
      env.LOCALAPPDATA && path.join(env.LOCALAPPDATA, 'zamak', 'bin'),
      env.APPDATA && path.join(env.APPDATA, 'npm'),
      path.join(home, '.cargo', 'bin'),
    ].filter(Boolean);
  }
  return [
    path.join(home, '.zamak', 'bin'),
    path.join(home, '.cargo', 'bin'),
    path.join(home, '.npm-global', 'bin'),
    path.join(home, '.local', 'bin'),
    '/opt/homebrew/bin',
    '/usr/local/bin',
  ];
}

// 按顺序找：设置里的 zamak.path → PATH（和终端里敲 zam 的结果一致）→ `npm root -g` 下的包 → 常见位置。
// 找到时返回 { command, from }，command 是可以直接启动的原生程序的完整路径。
function locate(options = {}) {
  const env = options.env || process.env;
  const system = {
    platform: options.platform || process.platform,
    arch: options.arch || process.arch,
    home: options.home || os.homedir(),
  };
  // 只写了名字（例如旧版本的默认值 "zam"）不算指定了位置，照样自己找。
  const configured = /[\\/]/.test(options.configured || '') ? options.configured : undefined;
  if (configured) {
    // 设置里填了 npm 的包装（zam.cmd 或 zam.js 的链接）也能用：顺着找到原生程序。
    const wrapped =
      /\.(cmd|bat)$/i.test(configured) || (system.platform !== 'win32' && isScript(configured));
    if (!wrapped) return { command: configured, from: 'zamak.path' };
    const native = fromDirectory(path.dirname(configured), system);
    return native ? { command: native, from: 'zamak.path' } : undefined;
  }
  const searchPath = env.PATH || env.Path || '';
  for (const dir of searchPath.split(path.delimiter).filter(Boolean)) {
    const found = fromDirectory(dir, system);
    if (found) return { command: found, from: 'PATH' };
  }
  if (options.npmRoot) {
    const found = fromPackage(path.join(options.npmRoot, '@veyralux', 'zamak'), system);
    if (found) return { command: found, from: 'npm' };
  }
  for (const dir of usualDirectories(env, system)) {
    const found = fromDirectory(dir, system);
    if (found) return { command: found, from: dir };
  }
  return undefined;
}

module.exports = { locate, PACKAGES };
