# Zamak for VS Code

Language support for [Zamak](https://zamak.veyralux.org), the language that reads like Python and runs like C.

- **Syntax highlighting** for `.zm` files, with comment toggling, bracket pairing, and automatic indentation after a line that ends with `:`.
- **Diagnostics as you type.** The extension checks the whole project, including changes you haven't saved, and underlines each error with its code and a suggested fix.
- **Completion** for keywords, built-in functions, your declarations and local variables, functions after a module name such as `string.`, and methods and fields after a value. After `x.`, the methods and fields come from the type the checker inferred, so `let p = make()` completes `p.` with what `make` returns.
- **Parameter hints** while you type a call: after `foo(`, the parameter list appears and the argument you are writing is highlighted.
- **Rename** and **Find All References** for local variables and functions in the current file.
- **Hover** shows the signature of a function or the fields of a struct.
- **Go to definition** for functions, structs, enums, constants, and local variables, across modules.
- **Format Document** runs the same formatter as `zam fmt`.

Created by rayL-K · VeyraLux

## Requirements

Install Zamak, for example with `npm install -g @veyralux/zamak`. The extension looks for `zam` on your `PATH`, in npm's global packages, and in the folders that the install scripts and Cargo use, then starts `zam lsp` in the background. Without `zam`, highlighting still works.

If the extension can't find `zam`, it offers an **Install with npm** button that runs `npm install -g @veyralux/zamak` in a terminal, and starts the language server by itself when the install finishes. The status bar shows the `zam` version in use; hover over it to see which file it runs.

## Settings

| Setting | Default | Description |
| --- | --- | --- |
| `zamak.path` | (empty) | The full path of the `zam` program. Leave it empty to let the extension find it. If it can't, run `where zam` (Windows) or `which zam` (Linux and macOS) in a terminal and use the path it prints. |

Diagnostics follow the VS Code display language: Chinese when VS Code is in Chinese, English otherwise. Set the `ZAMAK_LANG` environment variable to `zh` or `en` to choose yourself.

If you change your Zamak installation, run **Zamak: Restart Language Server** from the Command Palette. **Zamak: Install the zam Compiler (npm)** opens a terminal running `npm install -g @veyralux/zamak`; **Zamak: Check for the zam Compiler Again** looks for `zam` once more and starts the server if it is now there.

## 中文说明

Zamak 语言的 VS Code 扩展：语法高亮，边写边报错（没保存的内容也检查），补全（`x.` 后面按检查器推断出的真实类型列出方法与字段），输入调用时显示参数提示，本文件内的重命名与查找引用，悬停查看签名，跳到定义，以及与 `zam fmt` 相同的格式化。需要先安装 Zamak，例如 `npm install -g @veyralux/zamak`。扩展会在 `PATH`、npm 的全局包，以及安装脚本和 Cargo 用的文件夹里自己找 `zam`。还是找不到时，在终端里运行 `where zam`（Windows）或 `which zam`（Linux、macOS），把显示的路径填进设置 `zamak.path`。没装 `zam` 时扩展会给出「用 npm 安装」按钮，在终端里执行安装，装好后自动启动语言服务器；状态栏显示当前用的 `zam` 版本，鼠标悬停能看到用的是哪个文件。VS Code 界面是中文时，报错也是中文。

## Development

Install the dependency once, then link the folder into your extensions directory:

```powershell
cd editors/vscode
npm install
# Windows
New-Item -ItemType Junction -Path "$env:USERPROFILE\.vscode\extensions\zamak-0.2.1" -Target "$PWD"
# macOS and Linux
ln -s "$PWD" ~/.vscode/extensions/zamak-0.2.1
```

To build a `.vsix` package, run:

```powershell
npx @vscode/vsce package
code --install-extension zamak-0.2.1.vsix
```

`node scripts/check-grammar.mjs` checks that the three JSON files parse, that every TextMate regular expression compiles, that the language ID, file extension, and scope name agree, and that every command registered in `extension.js` is declared in `package.json` (and the other way round). When you change keywords or operators, update `syntaxes/zamak.tmLanguage.json`, `language-configuration.json`, and the keyword list in that script.

The language server lives in the compiler (`zam/src/lsp.rs`), so new compiler releases improve the extension without a new extension release.

### Publish to the Marketplace

1. Create a publisher at <https://marketplace.visualstudio.com/manage> and replace the placeholder `"publisher": "zamak"` in `package.json` with its ID.
2. Create a personal access token in Azure DevOps with the **Marketplace → Manage** scope.
3. Update `CHANGELOG.md` and the `version` in `package.json`.
4. Run `npx @vscode/vsce publish`.

`icon.png` is `docs/icon-512.png` (from `tools/make-logo.py`) scaled to 128×128.
