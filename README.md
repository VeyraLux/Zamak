<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/logo-dark.svg"/>
    <img src="docs/logo-light.svg" alt="Zamak" width="520"/>
  </picture>
</p>

<p align="center">
  <strong>English</strong>
  &nbsp;·&nbsp;
  <a href="./README.zh-CN.md">简体中文</a>
  &nbsp;·&nbsp;
  <a href="#quickstart">Quickstart</a>
  &nbsp;&middot;&nbsp;
  <a href="#why-zamak">Why Zamak</a>
  &nbsp;·&nbsp;
  <a href="https://zamak.veyralux.org/">The Book</a>
  &nbsp;·&nbsp;
  <a href="#language-tour">Language tour</a>
  &nbsp;·&nbsp;
  <a href="./editors/vscode">VS Code</a>
  &nbsp;·&nbsp;
  <a href="https://github.com/VeyraLux/zamak-index">Packages</a>
</p>

<p align="center">
  <a href="https://github.com/VeyraLux/Zamak/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/VeyraLux/Zamak/ci.yml?style=flat-square&label=ci&labelColor=161b22&logo=githubactions&logoColor=white" alt="CI"/></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-8b949e.svg?style=flat-square&labelColor=161b22" alt="license: MIT or Apache-2.0"/></a>
  <img src="https://img.shields.io/badge/version-0.1.1-f97362.svg?style=flat-square&labelColor=161b22" alt="version 0.1.1"/>
  <img src="https://img.shields.io/badge/platforms-Windows%20%C2%B7%20Linux%20%C2%B7%20macOS-6366f1.svg?style=flat-square&labelColor=161b22" alt="platforms: Windows, Linux, macOS"/>
  <a href="https://github.com/VeyraLux/Zamak/stargazers"><img src="https://img.shields.io/github/stars/VeyraLux/Zamak.svg?style=flat-square&color=06b6d4&labelColor=161b22&logo=github&logoColor=white" alt="GitHub stars"/></a>
</p>

<br/>

<p align="center"><strong>Open source · Native code · Ownership without garbage collection</strong></p>
<h3 align="center">Written with clarity. Compiled with precision.</h3>
<p align="center">Zamak is a statically typed language for people and AI agents alike. Its syntax stays light and familiar. Its compiler produces standalone native executables. Ownership manages memory without a garbage collector, and every error comes with a stable code and a fix you can act on.</p>
<p align="center">Created by <strong>rayL-K</strong> · <strong>VeyraLux</strong></p>

<br/>

## Quickstart

Install the compiler, then create and run a project:

```sh
npm install -g @veyralux/zamak    # Or: npx @veyralux/zamak --version to try it first
zam new hello                     # Creates hello/zam.toml and hello/src/main.zm
zam run hello                     # Prints "hello from zamak"
```

`zam` needs a C compiler for `build` and `run` (`zam check` does not). On Windows that is Visual Studio Build Tools with the **Desktop development with C++** workload, found through `vswhere`, falling back to `gcc` or `clang` on `PATH`; on Linux install `build-essential`; on macOS run `xcode-select --install`. Set `CC` to choose a compiler yourself.

Other ways to install, and the full command set:

- [Install from the script or build from source](https://zamak.veyralux.org/installation.html)
- [Every `zam` command](https://zamak.veyralux.org/tooling.html)
- [Create your first project](https://zamak.veyralux.org/first-program.html)

## Why Zamak

**Reads like Python. Runs like C. Safe without a garbage collector.**

<p align="center">
  <img src="docs/demo.svg" alt="The Zamak mascot turns a Python function into Zamak, then runs it and prints 13.5" width="720"/>
</p>

| As fast as C | 16 KB hello world on Linux | 20 MB where Java uses 365 MB | No lifetime annotations | Overflow checked in release builds |
| :---: | :---: | :---: | :---: | :---: |

Zamak is built for a way of working where several AI agents edit one project in parallel and build thousands of times a day. When goals conflict it prioritizes build cost, then memory safety, then runtime performance, then expressiveness, which is why there are no macros, no dependent types, and no lifetime annotations.

- [Why this language exists, and what it borrows from Rust, Go, C, and Python](https://zamak.veyralux.org/why-zamak.html)
- [How it compares with popular languages, and when to choose something else](https://zamak.veyralux.org/why-zamak.html#compared-with-popular-languages)

## Example

This program counts how often each word appears in its input:

```zamak
use std/io

fn main() -> !string:
    let text = io.read_all()?
    let mut counts: {string: i64} = {}

    for word in text.words():
        counts[word] = counts.get(word, 0) + 1

    for word, n in counts:
        print(f"{word}: {n}")
```

```text
$ echo "to be or not to be" | zam run count.zm
to: 2
be: 2
or: 1
not: 1
```

> [!NOTE]
> Zamak 0.1.1 is an early release. Use it to learn, experiment, and write command-line tools. Syntax and cache formats might still change before 1.0, so wait before you use it in production.

## Features

- **Familiar syntax.** `len(xs)`, `for i in range(n)`, `xs[-1]`, `xs[1:3]`, `f"..."`, and Python's division rules. `7 / 2` is `3.5`, `7 // 2` is `3`, and `len("你好")` is 2.
- **Ownership without a garbage collector.** Values move on assignment; a read-only parameter borrows automatically; `&mut x` marks a call that changes its argument. A borrow lasts one call, so you never write lifetime annotations.
- **Checked where it matters.** Integer overflow, division by zero, and out-of-bounds access stop the program in every build, and the compiler removes the checks it can prove redundant.
- **Fast feedback.** `zam check` needs no C compiler and finishes in under 0.1 s on a 5,000-line project; a one-line edit rebuilds in about 0.75 s.
- **Native, standalone output.** Zamak generates C and compiles it with MSVC, GCC, or Clang. The executable needs neither Zamak nor Rust.

See [Grammar at a glance](https://zamak.veyralux.org/grammar.html) for the syntax and [B. Performance](https://zamak.veyralux.org/appendix-performance.html) for build and run numbers.

## How Zamak compares

The same six programs, run in seven languages on one machine:

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/compare-speed-dark.svg"/>
    <img src="docs/compare-speed-light.svg" alt="Run time of n-body, spectral-norm, fannkuch-redux, binary-trees, primes, and word frequency in Zamak, C, Rust, Go, Java, Node.js, and Python" width="920"/>
  </picture>
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/compare-footprint-dark.svg"/>
    <img src="docs/compare-footprint-light.svg" alt="Executable size, startup time, and peak memory of Zamak, C, Rust, Go, Java, Node.js, and Python" width="920"/>
  </picture>
</p>

| | Zamak | C | Rust | Go | Python |
| --- | :---: | :---: | :---: | :---: | :---: |
| Memory safe by default | ✓ | ✗ | ✓ | ✓ | ✓ |
| No garbage collector | ✓ | ✓ | ✓ | ✗ | ✗ |
| Integer overflow checked in release builds | ✓ | ✗ | ✗ | ✗ | not needed |
| Array bounds checked | ✓ | ✗ | ✓ | ✓ | ✓ |
| Standalone native executable | ✓ | ✓ | ✓ | ✓ | ✗ |
| No lifetime annotations | ✓ | ✓ | ✗ | ✓ | ✓ |

Zamak stays in C's range on every compute-bound program while checking overflow and bounds by default. It is the fastest of the seven on binary-trees and word frequency, within 1% of C on spectral-norm, and third on n-body. For the method, full results, and the scripts, see [bench/compare](bench/compare/README.md).

## Use Zamak with your AI agent

```sh
zam new myapp --template ai
```

That creates `.claude/skills/zamak/SKILL.md`, `AGENTS.md`, and `.mcp.json` alongside the project, each carrying a one-page language reference and the check-fix loop. The text is built into `zam`, so it always matches your version. Your agent reads diagnostics from `zam check --json`, and through `zam mcp` it can call `check`, `build`, `run`, `test`, `format`, `explain`, and `docs` directly.

- [Work with AI coding agents](https://zamak.veyralux.org/ai-agents.html)
- [The AI benchmark behind these claims](bench/ai/live/README.md)

## Language tour

One program, using structs, error values, lists, borrows, and interpolation:

```zamak
use std/string

struct Person:
    name: string
    age: i64

fn parse_age(text: &string) -> i64!string:
    let age = string.parse_i64(text)?
    if age < 0:
        fail "age must not be negative"
    age

fn main() -> !string:
    let mut names: [string] = []
    for i in 0..3:
        names.push("user{i}")

    let age = parse_age("42") catch e:
        println("bad input: {e}")
        return

    let p = Person(name: "Ada", age: age)
    for name in &names:
        println("{name} is {string.byte_len(name)} bytes")
    println("{p.name} is {p.age}")
```

Blocks are indentation, `T!E` is an error value, `catch` handles it in place, `&names` borrows for the loop, and `{...}` interpolates.

- [Grammar at a glance](https://zamak.veyralux.org/grammar.html) has every construct, one section per topic with a runnable example.
- [The Book](https://zamak.veyralux.org/) goes deeper per chapter.
- [examples/md2html](examples/md2html) is a four-module Markdown-to-HTML converter, about 270 lines: `zam run examples/md2html -- examples/md2html/sample.md`.

## Diagnostics

Every error carries a stable code, the location, and a suggested fix:

```text
error[ZM301]: use after move: name
 --> moved.zm:8:5
  |
8 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

`zam check --json` reports the same thing as one line of JSON, always in English, which is what an agent reads. On a system set to Chinese the text diagnostic is Chinese; set `ZAMAK_LANG` to `en` or `zh` to choose.

See [diagnostic codes](https://zamak.veyralux.org/appendix-diagnostics.html) and the [agent workflow](docs/AI_WORKFLOW.md).

## Current limitations

No threads or networking yet, and no REPL. C functions take and return numbers, booleans, and strings; pass pointers and structs through a small C wrapper. `zam check` reports a diagnostic for anything unsupported instead of compiling it.

## Editor and agent support

- **VS Code.** The extension in [editors/vscode](editors/vscode) adds syntax highlighting and runs `zam lsp`, the Zamak language server, for diagnostics as you type, completion, hover, go to definition, and formatting. Other editors that support the Language Server Protocol can run `zam lsp` too.
- **AI coding agents.** `zam mcp` is an MCP server with tools to check, build, run, test, and format code, explain diagnostics, and look up docs. `zam new --template ai` adds a skill file, `AGENTS.md`, and `.mcp.json` to a new project, and [.agents/skills/zamak/SKILL.md](.agents/skills/zamak/SKILL.md) guides agents that work on the compiler itself.

## Contribute

For the development guide, see [CONTRIBUTING.md](CONTRIBUTING.md) (in Chinese). Before you submit a change, make sure these commands pass in the `zam` folder:

```sh
cargo test --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
```

## License

You can use Zamak under either the [MIT license](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE). Unless you state otherwise, you license your contributions under the same terms.
