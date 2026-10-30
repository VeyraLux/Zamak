# Changelog

## Unreleased

- Finds `zam` when you installed it with npm. On Windows, npm's `zam` is a `zam.cmd` wrapper that VS Code can't start, so the extension now starts the compiler inside npm's package directly. It also looks in npm's global packages, the install scripts' folders, and Cargo's, which helps when VS Code was opened before the install and its `PATH` is out of date.
- After **Install with npm**, the extension notices when the install finishes and starts the language server. You don't need to select **I installed it**.
- `zamak.path` is empty by default. A bare name such as the old default `zam` no longer stops the search. Hover over the status bar item to see which `zam` the extension runs.

## 0.2.0

- Highlighting for `bytes` literals (`b"GIF\x89"`), `\0` and `\xNN` escapes, and the `interface` keyword and the `bytes` type.
- Released together with Zamak 0.2.0. Everything the language server reports comes from `zam lsp`, so the new language features show up in diagnostics, completion, and hover without an extension update.

## 0.1.1

- One-click install: when `zam` isn't on `PATH`, the extension offers **Install with npm** (runs `npm install -g @veyralux/zamak` in a terminal) and **Zamak: Check for the zam Compiler Again** to pick it up afterwards.
- The status bar shows the `zam` version in use, and turns into a warning with an install entry point when `zam` is missing.
- Parameter hints while typing a call, and **Rename** / **Find All References** for names in the current file.
- Completion after `x.` uses the type the checker inferred instead of guessing from the name.

## 0.1.0

- Syntax highlighting for `.zm` files, with comment toggling, bracket pairing, and automatic indentation after a line that ends with `:`.
- Diagnostics as you type, including unsaved changes, from `zam lsp`.
- Completion, hover, go to definition, and Format Document.
- The `zamak.path` setting and the **Zamak: Restart Language Server** command.
