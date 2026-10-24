# Zamak

[Zamak](https://zamak.veyralux.org) is a programming language that reads like Python and runs like C. This package installs `zam`, the Zamak compiler, for Windows x64, Linux (x64 and arm64), and macOS (Apple silicon and Intel).

```sh
npm install -g @veyralux/zamak     # or: bun add -g @veyralux/zamak
zam new hello
zam run hello
```

To try it without installing, run `npx @veyralux/zamak --version` or `bunx @veyralux/zamak --version`.

To update, run `zam upgrade`. It runs `npm install -g @veyralux/zamak@latest` (or the Bun equivalent).

On Windows, `zam build` and `zam run` need the MSVC C++ build tools; on Linux and macOS, they need `cc` (GCC or Clang). `zam check` works without a C compiler.

- Documentation: <https://zamak.veyralux.org> ([简体中文](https://zamak.veyralux.org/zh-CN/))
- Source: <https://github.com/VeyraLux/Zamak>

Created by rayL-K · VeyraLux. Licensed under MIT or Apache-2.0.
