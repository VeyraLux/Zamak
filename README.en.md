# Zamak

English | [中文](README.md)

A statically typed language that compiles to native executables, designed around a fast edit → check → build loop for people and AI agents.

[![CI](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml/badge.svg)](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml)

<img width="160" height="160" alt="Zamak" src="ico/Zamak_256x256.png" />

```zamak
use std/io
use std/string

fn main() -> !string {
    let text = io.read_all()?
    let words = string.words(&text)
    let mut counts: {string: i64} = {}
    for word in &words {
        counts[word] = counts.get(word, 0) + 1
    }
    for word in &counts {
        let n = counts[word]
        println("{word}: {n}")
    }
}
```

```text
$ echo "to be or not to be" | zam run count.zm
to: 2
be: 2
or: 1
not: 1
```

> **Status: prototype (0.1.0).** The language is usable for learning, experiments, and small command-line tools. Syntax and cache formats may still change. Not for production use.

## Why Zamak

- **Fast feedback.** `zam check` type-checks and borrow-checks without invoking a C compiler. Builds are cached by content per module, so a one-line edit rebuilds only the affected module. On the project's 10-module, 5,250-line benchmark (Windows, MSVC, one machine), a one-line edit rebuilds in about 0.75 s and `check` takes under 0.1 s. Methodology and raw numbers are in [docs/ENGINEERING.md](docs/ENGINEERING.md) and [docs/PERFORMANCE.md](docs/PERFORMANCE.md) (Chinese).
- **Ownership without a garbage collector.** Strings, structs, lists, and maps move on assignment. Borrows (`&x`, `&mut x`) live only for the duration of a call, which keeps the rules small: there are no lifetime annotations. Integer overflow, division by zero, and out-of-bounds access are checked at run time; a conservative range analysis removes checks it can prove redundant.
- **Diagnostics built for tools.** Every error has a stable code (`ZM301`), a category, a position, and usually a suggested fix. `zam check --json` emits them as one line of JSON, and type errors are reported per function so an agent can fix several in one pass.
- **Native, standalone output.** Zamak generates C and compiles it with MSVC, GCC, or Clang. The resulting executable needs neither Zamak nor Rust installed. Runtime performance on the integer benchmarks in `docs/PERFORMANCE.md` is in the same range as Rust and Go; treat that as a sanity check, not a headline.

## Install

Build from source with Rust stable and a C compiler:

```sh
cargo install --path zam --locked
```

| Platform | C compiler |
| --- | --- |
| Windows x64 | Visual Studio Build Tools ("Desktop development with C++"), found through `vswhere`. Without it, Zamak falls back to `gcc` (for example MinGW-w64) and then `clang` on `PATH`. |
| Linux | `cc`, `clang`, or `gcc` on `PATH` (for example `build-essential`). |
| macOS | Xcode Command Line Tools (`xcode-select --install`). |

Set `CC` to choose a GCC-style compiler explicitly. `zam check` needs no C compiler. All three platforms run the full test suite in CI.

## First project

```sh
zam new hello        # creates hello/zam.toml and hello/src/main.zm
zam run hello        # prints: hello from zamak
zam check hello      # type and borrow check only
zam fmt hello        # normalizes indentation
zam build hello      # writes build/hello-<hash>(.exe)
zam run app.zm -- a b   # pass arguments to the program
```

Build progress goes to stderr; stdout carries only the program's output.

## The language in one screen

```zamak
use std/string

struct Person {
    name: string
    age: i64
}

// Errors are values: `T!string` returns either a T or an error message.
fn parse_age(text: &string) -> i64!string {
    let age = string.parse_i64(text)?
    if age < 0 {
        fail "age must not be negative"
    }
    age
}

fn main() -> !string {
    let mut ages: {string: i64} = {}
    let mut names: [string] = []
    for i in 0..3 {
        names.push("user{i}")
    }
    for name in &names {
        ages[name] = string.byte_len(name) * 10
    }
    let raw = "42"
    let age = parse_age(&raw) catch e {
        println("bad input: {e}")
        return
    }
    let p = Person { name: "Ada" age: age }
    println("{p.name} is {p.age}")
}
```

- Types: `i64`, `bool`, `string` (UTF-8), named structs, fixed arrays `[i64; N]`, lists `[T]`, and maps `{K: V}`. List elements and map values can be `i64`, `bool`, or `string`; map keys are `i64` or `string`.
- Control flow: `if`/`else if`/`else`, `while`, `for i in 0..n`, `for x in &list`, `for k in &map` (insertion order), `break`, `continue`.
- Errors: `T!string` and `!string` signatures, `fail`, `?` to propagate, `catch e { ... }` to handle in place.
- Modules: one file per module, `use name` and `pub fn`/`pub struct`.
- Standard library: `std/io` (stdin, `println`), `std/fs` (read and write whole files), `std/env` (arguments), and `std/string` (length, compare, parse, trim, find, slice, split, words, lines, join).

The [tutorial](TUTORIAL.md) (Chinese) walks through every feature with runnable examples.

## Diagnostics

```text
error[ZM301]: use after move: name
 --> moved.zm:8:5
  |
8 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

The same error from `zam check --json moved.zm`:

```json
{"command":"check","ok":false,"diagnostics":[{"file":"moved.zm","line":8,"column":5,"function":"main:main","code":"ZM301","category":"ownership","message":"use after move: name","suggestion":"borrow the value or move it only after the last use"}]}
```

The code list and the recommended agent loop (check → minimal fix → fmt → check → build → run) are in [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md) (Chinese). `zam new <dir> --template ai` adds an `AI.md` that describes the loop to a coding agent.

## Not yet supported

Floating point, generics, enums and `match`, closures, threads, FFI, lists or maps of structs, nested containers, networking, directory listing, a package manager, a language server, and a REPL. Unsupported constructs fail at `zam check` with a diagnostic rather than being silently miscompiled.

## Editor support

`editors/vscode/` contains a declarative VS Code extension with syntax highlighting and bracket configuration.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) (Chinese). Changes must pass `cargo test --locked`, `cargo fmt --check`, and `cargo clippy --all-targets --locked -- -D warnings` in `zam/`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Unless you state otherwise, contributions are licensed the same way.
