# Zamak for Zed

Language support for [Zamak](https://zamak.veyralux.org) in the [Zed](https://zed.dev) editor.

- **Syntax highlighting** for `.zm` files from a tree-sitter grammar, including string interpolation such as `"{name:>8}"`, escapes, raw and bytes strings, and error types such as `i64!string`.
- **Brackets and quotes in pairs.** Typing `(`, `[`, `{`, `"`, or `'` inserts the closing one, except `"` and `'` inside a string or a comment. Matching brackets are highlighted.
- **Indentation.** After a line that ends with `:`, the next line is indented 4 spaces. `elif` and `else` move back to line up with their `if`.
- **Outline and folding.** The outline panel and breadcrumbs list functions, structs, enums, interfaces, constants, fields, and variants. Every block folds.
- **Language server.** The extension starts `zam lsp` for diagnostics as you type, completion, parameter hints, hover, go to definition, rename, find references, and **Format Document** (the same formatter as `zam fmt`).

Created by rayL-K · VeyraLux

## Requirements

Install Zamak, for example with `npm install -g @veyralux/zamak`. The extension looks for `zam` on the `PATH` of your login shell, the same `PATH` a terminal has. Without `zam`, highlighting and the editing features above still work.

If Zed can't find `zam`, run `which zam` (Linux and macOS) or `where zam` (Windows) in a terminal, and add the path it prints to your Zed settings (**zed: open settings**):

```json
{
  "lsp": {
    "zam": {
      "binary": {
        "path": "/home/you/.zamak/bin/zam",
        "arguments": ["lsp"]
      }
    }
  }
}
```

Keep `"arguments": ["lsp"]`: when `path` is set, Zed starts that program with exactly these arguments.

Diagnostics follow your system language. Set the `ZAMAK_LANG` environment variable to `zh` or `en` in your shell profile to choose yourself.

## 中文说明

Zamak 语言的 Zed 扩展：基于 tree-sitter 语法的高亮（包括字符串插值 `"{name:>8}"`、转义、原始字符串与 bytes 字面量、`i64!string` 这样的错误类型）；输入 `(`、`[`、`{`、`"`、`'` 时自动补上另一半（字符串和注释里的引号除外），高亮配对的括号；以 `:` 结尾的行之后自动缩进 4 格，`elif`、`else` 自动回到对应 `if` 的位置；大纲面板列出函数、结构体、枚举、接口、常量、字段与变体，每个块都能折叠；并启动 `zam lsp`，提供边写边报错、补全、参数提示、悬停、跳到定义、重命名、查找引用，以及与 `zam fmt` 相同的格式化。

需要先安装 Zamak，例如 `npm install -g @veyralux/zamak`。扩展在登录 shell 的 `PATH`（与终端里相同）上找 `zam`。找不到时，在终端里运行 `which zam`（Linux、macOS）或 `where zam`（Windows），把路径填进 Zed 设置里的 `lsp.zam.binary.path`，同时写上 `"arguments": ["lsp"]`（见上面的例子）。报错语言跟随系统语言；想自己指定，在 shell 配置里设 `ZAMAK_LANG=zh` 或 `ZAMAK_LANG=en`。

## Development

You need Rust from [rustup](https://rustup.rs) and a C compiler. In Zed, run **zed: install dev extension** and choose the `editors/zed` folder. Zed builds `src/zamak.rs` to WebAssembly, fetches the grammar named in `extension.toml`, and compiles it.

`extension.toml` takes the grammar from this repository at the commit in `rev`, so Zed sees a grammar change only after it's committed. To try an uncommitted change, commit it on a local branch, then point `extension.toml` at your clone for the moment:

```toml
[grammars.zamak]
repository = "file:///home/you/Zamak"
rev = "<the output of git rev-parse HEAD>"
path = "editors/zed/grammar"
```

Then select **Rebuild** next to Zamak on the Extensions page (**zed: extensions**). Don't commit the `file://` URL.

| Path | What it is |
| --- | --- |
| `extension.toml` | The extension's name, version, grammar, and language server. |
| `src/zamak.rs` | Finds `zam` and starts `zam lsp`. |
| `languages/zamak/config.toml` | File extension, comments, bracket and quote pairs, and indentation rules. |
| `languages/zamak/*.scm` | Tree-sitter queries: highlighting, brackets, indentation, outline, and text objects. |
| `grammar/grammar.js` | The grammar. `grammar/src/scanner.c` turns line ends and indentation into tokens. |
| `grammar/src/parser.c` | Generated from `grammar.js`. Don't edit it; regenerate it. |
| `grammar/test/corpus/` | Parse tests: each one is Zamak source and the tree it must produce. |

After you change `grammar.js` or `scanner.c`, regenerate the parser and run the checks:

```sh
cd editors/zed/grammar
npx --yes tree-sitter-cli@0.27.2 generate --abi 14
npx --yes tree-sitter-cli@0.27.2 test
cd ../../..
node editors/zed/scripts/check.mjs
```

`check.mjs` regenerates the parser, runs the corpus tests, parses every `.zm` file in the repository and fails on any syntax error, compiles each query against the grammar, and checks that the names in `extension.toml`, `config.toml`, and `grammar.js` agree. CI also runs `git diff --exit-code` afterwards, so commit the regenerated `grammar/src/` together with `grammar.js`.

When you add a keyword or new syntax to the language, update `grammar/grammar.js` and `languages/zamak/highlights.scm` along with the VS Code grammar, and add a case to `grammar/test/corpus/`. Zed lets later patterns in `highlights.scm` override earlier ones, so general rules go first.

The language server lives in the compiler (`zam/src/lsp.rs`), so new compiler releases improve the extension without a new extension release.
