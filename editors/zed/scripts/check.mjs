// 校验 Zed 扩展：语法能生成且和提交的 parser.c 一致、语料测试通过、仓库里每个 .zm 都能无错误解析、
// 每个查询文件都能对着语法编译，以及各文件里的名字一致。
// 用法：node editors/zed/scripts/check.mjs（需要 C 编译器；tree-sitter CLI 由 npx 按固定版本下载）。
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const TREE_SITTER = 'tree-sitter-cli@0.27.2';
const zed = fileURLToPath(new URL('..', import.meta.url)); // editors/zed/
const repo = join(zed, '..', '..');
const grammar = join(zed, 'grammar');
const languages = join(zed, 'languages', 'zamak');
const errors = [];

function treeSitter(args, cwd = grammar) {
  // npx 在 Windows 上是 npx.cmd，要经过 shell 启动。
  return execFileSync('npx', ['--yes', TREE_SITTER, ...args], {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    shell: process.platform === 'win32',
  });
}

function run(label, args) {
  try {
    return treeSitter(args);
  } catch (error) {
    errors.push(`${label} failed:\n${error.stdout ?? ''}${error.stderr ?? ''}`);
    return '';
  }
}

// 1. 重新生成 parser.c。CI 之后用 git diff 确认提交的生成文件是最新的。
run('tree-sitter generate', ['generate', '--abi', '14']);

// 2. test/corpus 里的语料测试。
run('tree-sitter test', ['test']);

// 3. 仓库里的每个 .zm 文件都要解析得没有 ERROR 和 MISSING。
const skip = new Set(['target', 'node_modules', '.git', 'results', 'grammars', 'generated']);
const sources = [];
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    if (skip.has(name)) continue;
    const path = join(dir, name);
    if (statSync(path).isDirectory()) walk(path);
    else if (name.endsWith('.zm')) sources.push(path);
  }
};
walk(repo);
const scratch = mkdtempSync(join(tmpdir(), 'zamak-zed-'));
const list = join(scratch, 'files.txt');
writeFileSync(list, sources.join('\n'));
if (sources.length < 50) errors.push(`expected the repository's .zm files, found only ${sources.length}`);
const parsed = run('tree-sitter parse', ['parse', '--quiet', '--paths', list]);
for (const line of parsed.split('\n').filter((line) => /ERROR|MISSING/.test(line))) {
  errors.push(`parse error: ${line.trim()}`);
}

// 4. 每个查询都要能编译：节点名、字段名、匿名记号都必须在语法里存在。
const sample = join(scratch, 'sample.zm');
writeFileSync(sample, 'fn main():\n    println("hi")\n');
const queries = readdirSync(languages).filter((name) => name.endsWith('.scm'));
for (const name of ['highlights.scm', 'brackets.scm', 'indents.scm', 'outline.scm']) {
  if (!queries.includes(name)) errors.push(`languages/zamak/${name} is missing`);
}
for (const name of queries) {
  run(`query ${name}`, ['query', '--quiet', join(languages, name), sample]);
}

// 5. 名字一致：extension.toml 的语法名和路径、config.toml 的 grammar 与扩展名、语言服务器的语言名。
const extension = readFileSync(join(zed, 'extension.toml'), 'utf8');
const config = readFileSync(join(languages, 'config.toml'), 'utf8');
const grammarJs = readFileSync(join(grammar, 'grammar.js'), 'utf8');
const name = config.match(/^name = "([^"]+)"/m)?.[1];
if (!/^\[grammars\.zamak\]$/m.test(extension)) errors.push('extension.toml must declare [grammars.zamak]');
if (!/^path = "editors\/zed\/grammar"$/m.test(extension)) errors.push('extension.toml grammar path must be editors/zed/grammar');
if (!/^grammar = "zamak"$/m.test(config)) errors.push('config.toml must use grammar "zamak"');
if (!/^path_suffixes = \["zm"\]$/m.test(config)) errors.push('config.toml must claim .zm files');
if (!extension.includes(`languages = ["${name}"]`)) errors.push(`the language server must serve "${name}"`);
if (!/name: 'zamak'/.test(grammarJs)) errors.push("grammar.js must be named 'zamak'");
// 关键字要和 VS Code 的 TextMate 语法保持同步（那边的检查脚本用同一份列表）。
const keywords = ['catch', 'fail', 'struct', 'enum', 'match', 'elif', 'pass', 'and', 'or', 'not', 'while', 'for', 'pub', 'use', 'extern', 'interface', 'const', 'let', 'mut', 'assert', 'return', 'break', 'continue'];
for (const keyword of keywords) {
  if (!grammarJs.includes(`'${keyword}'`)) errors.push(`grammar.js does not mention the keyword ${keyword}`);
}

if (errors.length) {
  console.error(errors.join('\n'));
  process.exit(1);
}
console.log(
  `zed extension ok: ${sources.length} .zm files parse cleanly (${relative(repo, grammar)}), ${queries.length} queries compile`,
);
