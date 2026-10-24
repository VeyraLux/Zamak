# Working on Zamak

This file is for anyone who changes Zamak itself, people and AI coding agents alike. If you're writing programs *in* Zamak, see the [Zamak Book](https://zamak.veyralux.org/) instead.

If a file named `AGENTS.local.md` exists next to this one, read it too. It holds a maintainer's local notes and isn't part of the repository.

## What's where

| Path | What it is |
| --- | --- |
| `zam/src/compiler.rs` | Lexer, parser, type and ownership checker, and the rewrites and derives that run before checking. |
| `zam/src/native.rs` | The C back end, the C runtime, and the C compiler and linker drivers. |
| `zam/src/project.rs` | Projects, modules, the content-addressed build cache, and the bundled C libraries. |
| `zam/src/main.rs`, `lsp.rs`, `mcp.rs`, `zh.rs` | The `zam` command, the language server, the MCP server, and Chinese error messages. |
| `zam/std/*.zm` | Standard library modules written in Zamak. |
| `zam/vendor/` | Bundled C sources: Mbed TLS, libui-ng, and SQLite. |
| `zam/tests/` | Integration tests. Each one compiles and runs real Zamak programs. |
| `examples/` | Runnable examples, including the apps in `examples/apps/`. |
| `book/` | The Zamak Book, in English and Chinese. Every code block is compiled and its output checked. |
| `bench/` | The language comparison (`bench/compare`) and the AI benchmark (`bench/ai`). |
| `.agents/skills/zamak/SKILL.md` | Detailed notes on how the compiler works, in Chinese. Read the parts about the area you change. |

## Build and test

Work in `zam/`:

```sh
cargo build
cargo test --no-fail-fast
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

- The tests build many programs and use every CPU core. To keep your machine usable, run them at low priority, limit the threads (`cargo test -- --test-threads=4`), or set `ZAMAK_JOBS` to limit how many C files `zam` compiles at once.
- To run only what you touched, pass the test file: `cargo test --test lists`.
- Linux and macOS run in CI. On Windows, `tools/wsl-verify.sh` runs the full suite with GCC and Clang in WSL, plus AddressSanitizer on chosen examples.

## Rules

- **Branches.** Changes to the compiler, standard library, tests, or examples go to `dev`. `main` moves forward only after CI passes on `dev`. Changes to docs alone may go straight to `main`.
- **Commits.** English, in the form `<type>: <summary>`, where the type is `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `build`, or `chore`. The body says what changed and why.
- **Every change comes with a test.** A new feature needs a runnable example, a test that runs it and checks its output, and tests for what it rejects. A bug fix needs a test that fails without the fix.
- **Generated C must compile with MSVC, GCC, and Clang.** Keep platform differences in `native.rs` and `project.rs`. In tests, don't hard-code `.exe`, `\` in paths, or CRLF; read the output path from the `OUTPUT` line on stderr.
- **Cache keys.** When the IR or the generated C changes shape, bump `ZAM-IR-N` in `compiler.rs` or `native-c-N` in `project.rs`, as `SKILL.md` describes.
- **No new dependencies** without a strong reason. Zamak bundles what it needs.

## Design principles

When a choice isn't obvious, these decide it:

1. **Read like Python.** Use Python's names and meanings for anything Python has: `append`, `remove(x)`, `pop(i)`, `ljust(width, fill)`, `json.dumps`. If Zamak spells something the same way, it must mean the same thing.
2. **No silent surprises.** A value is either kept exactly or rejected with a clear error. Never round, truncate, or guess. An error says what's wrong and how to fix it.
3. **Batteries included.** Prefer building a capability into `zam` (as with TLS, SQLite, and windows) over asking users to install something.
4. **Same behavior on every platform.** When a C library behaves differently on Windows, macOS, and Linux, Zamak hides the difference.

## Done means everything is in sync

These are the steps people most often forget. A change isn't done until each one that applies is finished:

| If you changed… | Also update |
| --- | --- |
| Syntax or a language rule | `book/en` **and** `book/zh-CN` (the chapter, plus `grammar.md` and `appendix-python.md` if they mention it), `bench/ai/live/zamak-reference.md`, `SKILL.md`, `CHANGELOG.md` (English and Chinese halves), and `editors/vscode/syntaxes/zamak.tmLanguage.json` for new keywords. |
| A standard library module | `book/*/src/standard-library.md`, `zamak-reference.md`, `CHANGELOG.md`, and an example in `examples/`. |
| An error message | Add its Chinese in `zam/src/zh.rs` (a test fails otherwise). A new error code also goes in `book/*/src/appendix-diagnostics.md`. |
| Something that used to be rejected and now works | Run `pwsh -File bench/ai/run.ps1 -ZamPath zam/target/debug/zam.exe` and `python bench/ai/live/run.py --verify-solutions --zam zam/target/debug/zam.exe`. Replace any counterexample in `bench/ai/cases.json` that now compiles. Update "not supported yet" lists in the book and README. |
| Speed, memory, or binary size | Run `python bench/compare/run.py` with a release build, and update the numbers in `README.md`, `README.zh-CN.md`, `book/*/src/appendix-performance.md`, and `docs/PERFORMANCE.md`. |
| What Zamak can or can't do | The feature and limitation lists in **both** READMEs. |
| A library in `zam/vendor/` (added or upgraded) | `THIRD-PARTY-NOTICES.md` (version and license text), the library's `README.md` in `zam/vendor/`, and the `native-c-N` version in `project.rs`. |
| `zam` commands or the MCP server | `book/*/src/tooling.md`, the README, and `zam/src/mcp.rs` if an MCP tool should expose it. |

Two files are compiled into `zam`, so when they're out of date, every user gets stale information: `bench/ai/live/zamak-reference.md` (the MCP `docs` tool, and the guide that `zam new --template ai` writes) and `book/en/src/appendix-diagnostics.md` (the MCP `explain` tool).

## Don't

- Don't publish anything: no tags, releases, package uploads, or site deploys. Maintainers do these.
- Don't commit build output, `results/`, caches, or local notes.
- Don't put real names, keys, or tokens in any file.
