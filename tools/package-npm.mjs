// 把 GitHub Release 的三个平台压缩包做成 npm 包：每个平台一个装着 zam 的子包，加上主包 zamak
// （启动器 + optionalDependencies，npm 只装与当前平台匹配的那个子包）。
//   node tools/package-npm.mjs <版本，如 0.1.0> <放压缩包的目录> <输出目录>
// 输出目录里每个子目录都是一个可以直接 `npm publish` 的包。
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, chmodSync, readdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const [version, assets, out] = process.argv.slice(2);
if (!version || !assets || !out) {
  console.error('usage: node tools/package-npm.mjs <version> <assets> <out>');
  process.exit(2);
}
const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const PLATFORMS = [
  { name: 'zamak-win32-x64', os: 'win32', cpu: 'x64', archive: 'zamak-windows-x64.zip', binary: 'zam.exe' },
  { name: 'zamak-linux-x64', os: 'linux', cpu: 'x64', archive: 'zamak-linux-x64.tar.gz', binary: 'zam' },
  { name: 'zamak-darwin-arm64', os: 'darwin', cpu: 'arm64', archive: 'zamak-macos-arm64.tar.gz', binary: 'zam' },
];
const common = {
  version,
  homepage: 'https://zamak.veyralux.org',
  repository: { type: 'git', url: 'git+https://github.com/VeyraLux/Zamak.git' },
  license: 'MIT OR Apache-2.0',
  author: 'VeyraLux (founded by rayL-K)',
};

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });
for (const platform of PLATFORMS) {
  const dir = join(out, platform.name);
  mkdirSync(join(dir, 'bin'), { recursive: true });
  // 解开压缩包，只取出 bin/zam。
  const tmp = mkdtempSync(join(tmpdir(), 'zamak-npm-'));
  const archive = join(assets, platform.archive);
  // 先把压缩包复制进临时目录再用相对路径解开：Windows 上的 GNU tar 会把 `C:` 当成远程主机名。
  cpSync(archive, join(tmp, platform.archive));
  if (platform.archive.endsWith('.zip')) {
    execFileSync('unzip', ['-q', platform.archive], { cwd: tmp });
  } else {
    execFileSync('tar', ['-xzf', platform.archive], { cwd: tmp });
  }
  const folder = readdirSync(tmp).find((entry) => entry.startsWith('zamak-') && !entry.includes('.'));
  cpSync(join(tmp, folder, 'bin', platform.binary), join(dir, 'bin', platform.binary));
  rmSync(tmp, { recursive: true, force: true });
  if (platform.binary === 'zam') chmodSync(join(dir, 'bin', 'zam'), 0o755);
  for (const license of ['LICENSE-MIT', 'LICENSE-APACHE']) cpSync(join(root, license), join(dir, license));
  writeFileSync(join(dir, 'package.json'), JSON.stringify({
    name: platform.name,
    description: `The zam compiler of Zamak for ${platform.os} ${platform.cpu}. Install the zamak package instead.`,
    ...common,
    os: [platform.os],
    cpu: [platform.cpu],
    files: ['bin', 'LICENSE-MIT', 'LICENSE-APACHE'],
    preferUnplugged: true,
  }, null, 2) + '\n');
  writeFileSync(join(dir, 'README.md'), `# ${platform.name}\n\nThe \`zam\` compiler for ${platform.os} ${platform.cpu}. Install [zamak](https://www.npmjs.com/package/@veyralux/zamak) instead; it picks this package for you.\n`);
}

// 主包：启动器、README，子包按同一个版本精确依赖。
const main = join(out, 'zamak');
cpSync(join(root, 'npm', 'zamak'), main, { recursive: true });
for (const license of ['LICENSE-MIT', 'LICENSE-APACHE']) cpSync(join(root, license), join(main, license));
const manifest = JSON.parse(readFileSync(join(main, 'package.json'), 'utf8'));
manifest.version = version;
manifest.files = ['bin', 'LICENSE-MIT', 'LICENSE-APACHE'];
manifest.optionalDependencies = Object.fromEntries(PLATFORMS.map((p) => [p.name, version]));
writeFileSync(join(main, 'package.json'), JSON.stringify(manifest, null, 2) + '\n');
chmodSync(join(main, 'bin', 'zam.js'), 0o755);
console.log(`npm packages for ${version} in ${out}: ${[...PLATFORMS.map((p) => p.name), 'zamak'].join(', ')}`);
