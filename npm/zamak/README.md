# Zamak

[Zamak](https://zamak.veyralux.org) is a programming language that reads like Python and runs like C. This package installs `zam`, the Zamak compiler, for Windows x64, Linux x64, and macOS on Apple silicon.

```sh
npm install -g zamak     # or: bun add -g zamak
zam new hello
zam run hello
```

To try it without installing, run `npx zamak --version` or `bunx zamak --version`.

To update, run `zam upgrade`. It runs `npm install -g zamak@latest` (or the Bun equivalent).

On Windows, `zam build` and `zam run` need the MSVC C++ build tools; on Linux and macOS, they need `cc` (GCC or Clang). `zam check` works without a C compiler.

- Documentation: <https://zamak.veyralux.org> ([简体中文](https://zamak.veyralux.org/zh-CN/))
- Source: <https://github.com/VeyraLux/Zamak>

Created by rayL-K · VeyraLux. Licensed under MIT or Apache-2.0.
