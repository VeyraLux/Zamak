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
  <a href="#why-zamak">Why Zamak</a>
  &nbsp;·&nbsp;
  <a href="https://zamak.veyralux.org/">The Zamak Book</a>
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

## Why Zamak

**Reads like Python. Runs like C. Safe without a garbage collector.**

<p align="center">
  <img src="docs/demo.svg" alt="The Zamak mascot turns a Python function into Zamak, then runs it and prints 13.5" width="720"/>
</p>

| As fast as C | 16 KB hello world on Linux | 20 MB where Java uses 365 MB | No lifetime annotations | Overflow checked in release builds |
| :---: | :---: | :---: | :---: | :---: |

### Why this language exists

Zamak grew out of a new way of working: several AI agents editing the same project in parallel,
across worktrees, each one checking, building and testing hundreds or thousands of times a day.
At that pace, how long a build takes, how big the artifact is, and whether a machine can read the
error stop being matters of taste and become matters of throughput.

So Zamak trades differently from most languages. When goals conflict, this order decides:

1. **Build cost** — the whole loop (read → edit → check → build → test → read the error) times
   how often it runs per day, plus disk footprint
2. **Memory safety** — safe by default; unsafe code has to be marked
3. **Runtime performance** — reach the Rust tier and stop; no attempt to beat it
4. **Expressiveness** — Python-level readability, omitting noise but never safety

That is why macros, dependent types, and anything needing whole-program analysis are out of scope:
they slow the build down, and build speed is the first principle here. Generics are not
monomorphized by default — no per-instance copies of the same code.

### What it borrows

Zamak is not trying to be a shadow of another language. It combines the parts each language does
best:

| From | What we wanted | How Zamak gets it |
| --- | --- | --- |
| Rust | Memory safety, no GC | Ownership plus call-frame borrows: values move by default, a borrow never outlives the call, and there are no lifetime annotations |
| Go | Fast builds, one file to ship | Independently compilable units, content-addressed cache, one `zam` command for check, build, test and format |
| C | Small binaries, speed, access to existing code | Emits C and hands it to MSVC, GCC or Clang; no runtime; `extern fn` calls C directly |
| Python | Readability, nothing to unlearn | Indentation blocks, `len`, `range`, `in`, slicing, and Python's division rules |

Zamak brings these together in one language. The numbers come from benchmarks you can run yourself; see [How Zamak compares](#how-zamak-compares) and [Features](#features).

- **Python's readability at C's speed.** Zamak reads like Python: `for i in range(n)`, `xs[-1]`, `f"..."`, and Python's division rules. It compiles to native code that beats C on binary-trees, fannkuch-redux, primes, and word frequency, and trails C by about 1% on spectral-norm, while running compute-bound programs 43 to 204 times faster than Python.
- **Memory safety you don't have to fight.** Ownership rules out use-after-free and double frees at compile time, with no garbage collector. Read-only borrows happen automatically, a borrow lasts at most one block, and you never write lifetime annotations. When the checker does reject code, the error names the problem and suggests the fix.
- **Safer integer math than C, Rust, or Go.** Integer overflow, division by zero, and out-of-bounds indexes stop the program with a clear error in every build, including release builds. The compiler removes the checks it can prove unnecessary, so safe code still runs at C speed, and `@wrapping` turns checks off in one function when you want wrapping, such as in a hash.
- **Small programs with a small footprint.** On Linux, a Zamak hello world is 16 KB: about the size of C's, and 1/17 of the same program in Rust even after `strip` plus LTO. On Windows the binary is bigger either way (`cl` links the C runtime statically): about 143 KB with the default MSVC toolchain, against 106 KB for a maximally stripped Rust build, so there the two are in the same range. On binary-trees, Zamak is the fastest of the seven languages tested and uses 20 MB, where Java and Node.js use more than 360 MB.
- **Built for AI coding agents.** The whole language fits in a one-page reference that agents read before they write code. Every diagnostic has a stable code and a suggested fix, also as JSON. `zam new --template ai` sets up a project with a skill file for your agent, and `zam mcp` lets agents check, run, test, and look up docs through MCP.
- **Fast feedback.** `zam check` finishes in less than 0.1 seconds on a 5,000-line project, and a one-line edit rebuilds in under a second, because `zam` caches every module by content.

### Compared with popular languages

Each popular language asks you to give something up, and Zamak is designed so that you don't have to choose:

- **Python** is easy to read and write, but it's slow, it needs an interpreter on every machine, and it finds type errors only when the code runs.
- **C** is fast and small, but one wrong index or integer overflow silently corrupts memory.
- **Rust** is fast and memory safe, but lifetimes and the borrow checker take a long time to learn, and its release builds don't check integer overflow.
- **Go** is simple and safe, but it ships a garbage collector, and a hello world is 2.3 MB.
- **Java** and **Node.js** need a large runtime, and they use hundreds of megabytes on allocation-heavy work.

### When to choose something else

Zamak is young. It doesn't support threads or networking yet, and its package ecosystem is just starting. For production services, large existing codebases, or work that depends on mature libraries, use an established language today, and try Zamak for command-line tools, data processing, learning, and code that AI agents write.

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

- **Familiar syntax.** If you have written Python, you can read Zamak without adjusting. `len(xs)`, `for i in range(n)`, `x in xs`, `a if c else b`, `xs[-1]`, `xs[1:3]`, `sum(xs)`, `sorted(xs)`, `int(s)`, `str(n)`, `2 ** 10`, `f"..."`, and `#` comments mean what they mean in Python. Division follows Python too: `7 / 2` is `3.5`, `7 // 2` is `3`, and `-7 % 2` is `1`. Strings count characters, so `len("你好")` is 2.
- **Ownership without a garbage collector.** Strings, structs, lists, and maps move when you assign them. Passing a variable to a read-only parameter borrows it automatically, as in `show(name)`; a function that changes its argument says so at the call site with `&mut x`. A borrow lasts for one call, or until the end of the block for a read-only local such as `let entry = &entries[i]`, so you never write lifetime annotations. Zamak checks integer overflow, division by zero, and out-of-bounds access at run time, and it removes the checks that a conservative range analysis proves redundant. For a hot function where wrapping is intended, such as a hash, write `@wrapping` above `fn` to turn off overflow checks in that function only.
- **Fast feedback.** `zam check` checks types and ownership without calling a C compiler. `zam build` caches each module by content, so a one-line edit rebuilds only the module you changed. On the project's 10-module, 5,250-line benchmark (Windows, MSVC, one machine), a one-line edit rebuilds in about 0.75 seconds, and `zam check` finishes in less than 0.1 seconds. For the method and raw numbers, see [docs/ENGINEERING.md](docs/ENGINEERING.md) and [docs/PERFORMANCE.md](docs/PERFORMANCE.md) (in Chinese).
- **Diagnostics built for tools.** Every error has a stable code such as `ZM301`, a category, a position, and usually a suggested fix. If you use a Python spelling that Zamak writes differently, the message shows the Zamak spelling. `zam check --json` writes the diagnostics as one line of JSON. Zamak reports type errors per function, so an agent can fix several of them in one pass.
- **Learnable from one page.** In a benchmark with three models and eight small command-line tasks, each model read only a one-page language reference. The models solved all 24 Zamak tasks, 21 of them on the first try. The three other first attempts were one missing import, one missing `?`, and one `{{` written as an interpolated brace. The Rust and Go versions passed 23 and 24 tasks on the first try, so these tasks are too easy to rank the languages. For the method, tasks, and data, see [bench/ai/live](bench/ai/live/README.md).
- **Native, standalone output.** Zamak generates C and compiles it with Microsoft C++ (MSVC), GCC, or Clang. The executable runs without Zamak or Rust installed. On the integer benchmarks in [docs/PERFORMANCE.md](docs/PERFORMANCE.md), its runtime performance is in the same range as Rust and Go.

## How Zamak compares

Zamak brings together what C, Rust, Go, and Python each do best. These charts run the same programs in each language on one machine: four from [The Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) and two everyday tasks. Shorter bars are better.

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

Zamak runs in the same range as C on every compute-bound program here, while it checks overflow and bounds by default. It's the fastest of the seven languages on binary-trees and on word frequency; on spectral-norm it is within 1% of C and on primes within 2% of Rust; only n-body puts it third (Rust first, Go second), 5% ahead of C. Its one opt-out is `@wrapping` on the spectral-norm hot function, which turns off overflow checks for that function only. For the method, the full results, and how to run the benchmark yourself, see [bench/compare](bench/compare/README.md).

## Use Zamak with your AI agent

Zamak treats AI coding agents as first-class readers. To start a project that your agent can work on right away, run:

```sh
zam new hello --template ai
```

The project includes:

- **`.claude/skills/zamak/SKILL.md`**, a skill that Claude Code loads whenever it reads or writes Zamak.
- **`AGENTS.md`**, the same guide for Codex, Cursor, GitHub Copilot, and other agents that read `AGENTS.md`.
- **`.mcp.json`**, which starts `zam mcp` for agents that support the Model Context Protocol (MCP), such as Claude Code and Cursor.

Both files contain the check-fix loop and the one-page language reference, so your agent writes Zamak correctly from its first line. They come from the reference built into `zam`, so a new project always matches the version you have installed.

While it works, your agent reads diagnostics from `zam check --json`. Every diagnostic has a stable code, such as `ZM301`, a position, and usually a suggested fix.

Through `zam mcp`, an agent can call Zamak directly: `check`, `build`, `run` (with arguments and standard input), `test`, `format`, `explain` for a diagnostic code, and `docs` for the language reference and the standard library. See [Work with AI coding agents](https://zamak.veyralux.org/ai-agents.html).

## Quickstart

Install the prebuilt compiler, then make a project. You need a C compiler for `zam build` and `zam run`; see [Install Zamak](#install-zamak).

```sh
npm install -g @veyralux/zamak                # Or: bun add -g @veyralux/zamak. Or try it first with: npx @veyralux/zamak --version
zam new hello                       # Creates hello/zam.toml and hello/src/main.zm.
zam run hello                       # Prints "hello from zamak".
```

To update later, run `zam upgrade`. Without Node.js, use the install script (Linux and macOS: `curl -fsSL https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.sh | sh`; Windows: `irm https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.ps1 | iex`) or build from source below.

## Install Zamak

To build Zamak from source instead of installing it with npm, you need Rust stable and a C compiler. In the repository root, run:

```sh
cargo install --path zam --locked
```

Zamak finds a C compiler on each platform as follows:

| Platform | C compiler |
| --- | --- |
| Windows x64 | Visual Studio Build Tools with the **Desktop development with C++** workload, located through `vswhere`. If it isn't installed, Zamak uses `gcc` (for example, MinGW-w64) or `clang` from `PATH`. |
| Linux | `cc`, `clang`, or `gcc` from `PATH`. On Debian and Ubuntu, install `build-essential`. |
| macOS | Xcode Command Line Tools. To install them, run `xcode-select --install`. |

To choose a GCC-style compiler yourself, set the `CC` environment variable. `zam check` doesn't need a C compiler. Continuous integration (CI) runs the full test suite on all three platforms.

## Create your first project

```sh
zam new hello        # creates hello/zam.toml and hello/src/main.zm
zam run hello        # prints: hello from zamak
zam check hello      # type and borrow check only
zam fmt hello        # normalizes indentation
zam test hello       # runs every fn test_...() in the project
zam build hello      # writes build/hello-<hash>(.exe)
zam run app.zm -- a b   # pass arguments to the program
zam doc hello        # writes an HTML API page from the pub declarations
```

`zam` writes build progress to standard error, so standard output contains only your program's output.

## Language tour

This program uses structs, error values, lists, maps, and string interpolation:

```zamak
use std/string

struct Person:
    name: string
    age: i64

# Errors are values: `T!string` returns either a T or an error message.
fn parse_age(text: &string) -> i64!string:
    let age = string.parse_i64(text)?

    if age < 0:
        fail "age must not be negative"

    age

fn main() -> !string:
    let mut ages: {string: i64} = {}
    let mut names: [string] = []

    for i in 0..3:
        names.push("user{i}")

    for name in &names:
        ages[name] = string.byte_len(name) * 10

    let raw = "42"
    let age = parse_age(&raw) catch e:
        println("bad input: {e}")
        return
    let p = Person(name: "Ada", age: age)
    println("{p.name} is {p.age}")
```

- **Blocks.** Indentation defines blocks, as in Python. A line ending with `:` opens one, and the body is indented 4 spaces deeper; a single statement may stay on the same line (`if n < 1: return n`), and an empty block is `pass`. Braces appear only in maps and interpolation. Run `zam fmt` to replace tabs and normalize indentation.
- **Types.** `i64`, `f64`, `bool`, `string` (UTF-8), named structs, fixed arrays `[i64; N]`, lists `[T]`, maps `{K: V}`, and tuples. Lists and maps nest to any depth (`[[i64]]`, `{string: [string]}`), and `xs[-1]` and `xs[a:b]` index and slice as in Python.
- **Numbers.** Mixing `i64` and `f64` converts the integer to `f64`, as in Python. `/` gives an `f64`, `//` floors, `%` takes the sign of the divisor, and division by zero stops the program. Convert with `f64(n)` or `i64(x)`.
- **Structs and methods.** Declare `struct Point:` with one field per indented line and build one with `Point(x: 1, y: 2)`, naming every field. Methods go in the struct block with `&self`, `&mut self`, or `self` first, and are called as `p.norm()`. A function without `self` is called on the type, as in `Point.new(1, 2)`.
- **Ownership.** Numbers and booleans copy; strings, structs, lists, maps, and tuples move. Use `copy(x)` for a deep copy. A parameter typed `&Point` or `&[T]` borrows for one call, and `&mut` allows writes.
- **Enums and match.** Declare `enum Shape:` with one variant per indented line, such as `Circle(r: f64)` or `Empty`, and build values with `Shape.Circle(1.0)`. A `match` lists one arm per line and must cover every variant or end with `_:`. The built-in `Option[T]` has `Option.Some(v)` and `Option.None`.
- **Control flow.** `if`, `elif`, `else`, `while`, `break`, `continue`, and `and`/`or`/`not` with Python's precedence. Choose a value with `let v = if c: a elif d: b else: e`. Loop with `for i in 0..n`, `for x in xs`, `for i, x in &xs`, `for k, v in &map`, or `for x in &mut xs: x += 1`.
- **Errors.** A function that can fail returns `T!E` or `!E`, where `E` is a string, struct, or enum. `fail` returns an error, `?` passes it to the caller, and `catch e:` handles it in place. `main` may carry an error type; an uncaught error goes to stderr and exits with code 1.
- **Generics.** Write `fn largest[T](items: &[T]) -> T` or `struct Stack[T]`. The call site infers the type argument, and Zamak compiles a separate version per type.
- **Function values.** A function type is `fn(i64) -> bool`. Pass a named function by name or write `fn(x): x * 2`. Closures may read and change the enclosing function's variables, and may be called, bound, and passed, but never stored or returned, so no garbage collector is needed.
- **Strings.** Call string functions as methods without importing: `name.upper()`, `csv.split(",")`, `line.trim().len()`. Character indexing and slicing work as in Python (`len("你好")` is 2); `byte_len`, `byte_at`, and `byte_slice` work in UTF-8 bytes. Interpolation accepts expressions such as `"{a + b}"`.
- **Tests.** `assert cond, "message"` stops the program when `cond` is false. `zam test` runs every function named `test_...` in its own process.
- **Modules and packages.** Each file is a module; import one with `use name`, and export with `pub fn` or `pub struct`. Find libraries with `zam search json` and add one with `zam add mathx`, which writes a Cargo-style version requirement. `zam publish` shares your own library.
- **Constants.** Declare `const LIMIT = 100` at the top level; the value is computed at compile time.
- **C libraries.** Declare a C function with `extern fn sqrt(x: f64) -> f64` and call it directly. List C files, include directories, and libraries under `[native]` in `zam.toml`. See `examples/ffi`.
- **Printing and operators.** `print(x)` and `println(x)` print a value and a newline. Arithmetic uses `+ - * / // % **`, `x in xs` tests membership, and integers support `& | ^ ~ << >>` plus `0xFF`-style literals.
- **Standard library.** `std/io`, `std/fs`, `std/path`, `std/env`, `std/string`, `std/math`, `std/time`, `std/random`, and `std/fmt`. Python's `len`, `str`, `int`, `float`, `abs`, `min`, `max`, `sum`, and `sorted` need no import.

[The Zamak Book](https://zamak.veyralux.org/) covers every feature with runnable examples, in English and Chinese. For a larger program, see [examples/md2html](examples/md2html), a Markdown-to-HTML converter with four modules and about 270 lines. It handles headings, lists, quotes, fenced code, inline formatting, links, and HTML escaping. To run it:

```sh
zam run examples/md2html -- examples/md2html/sample.md
```

## Diagnostics

When a check fails, `zam` shows the error code, the location, the source line, and a suggested fix:

```text
error[ZM301]: use after move: name
 --> moved.zm:8:5
  |
8 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

On a system set to Chinese, `zam` shows the same diagnostic in Chinese. Set `ZAMAK_LANG` to `en` or `zh` to choose the language yourself.

`zam check --json moved.zm` reports the same error as one line of JSON, always in English:

```json
{"command":"check","ok":false,"diagnostics":[{"file":"moved.zm","line":8,"column":5,"function":"main:main","code":"ZM301","category":"ownership","message":"use after move: name","suggestion":"borrow the value or move it only after the last use"}]}
```

For the list of error codes and the recommended loop for AI agents (check, make the smallest fix, format, check, build, and run), see [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md) (in Chinese).

## Current limitations

Zamak doesn't support threads or networking yet. C functions take and return only numbers, booleans, and strings; pass pointers and structs through a small C wrapper. There's no read-eval-print loop (REPL) yet. When you use an unsupported construct, `zam check` reports a diagnostic instead of compiling incorrect code.

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
