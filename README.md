# Zamak

English | [中文](README.zh-CN.md)

Zamak is a statically typed language that compiles to native executables. It's built for a fast loop of editing, checking, and building code, for both people and AI coding agents.

[![CI](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml/badge.svg)](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml)

<img width="96" height="96" alt="Zamak" src="ico/Zamak_256x256.png" />

This program counts how often each word appears in its input:

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

Run it with some input:

```text
$ echo "to be or not to be" | zam run count.zm
to: 2
be: 2
or: 1
not: 1
```

> [!NOTE]
> Zamak is a prototype (version 0.1.0). Use it to learn, experiment, and write small command-line tools. Syntax and cache formats might change, so don't use it in production yet.

## Why Zamak

- **Fast feedback.** `zam check` checks types and ownership without calling a C compiler. `zam build` caches each module by content, so a one-line edit rebuilds only the module you changed. On the project's 10-module, 5,250-line benchmark (Windows, MSVC, one machine), a one-line edit rebuilds in about 0.75 seconds, and `zam check` finishes in less than 0.1 seconds. For the method and raw numbers, see [docs/ENGINEERING.md](docs/ENGINEERING.md) and [docs/PERFORMANCE.md](docs/PERFORMANCE.md) (in Chinese).
- **Ownership without a garbage collector.** Strings, structs, lists, and maps move when you assign them. A borrow (`&x` or `&mut x`) lasts only for one call, so you never write lifetime annotations. Zamak checks integer overflow, division by zero, and out-of-bounds access at run time, and it removes the checks that a conservative range analysis proves redundant.
- **Diagnostics built for tools.** Every error has a stable code such as `ZM301`, a category, a position, and usually a suggested fix. `zam check --json` writes them as one line of JSON. Zamak reports type errors per function, so an agent can fix several of them in one pass.
- **Learnable from one page.** In a benchmark with three models and eight small command-line tasks, each model read only a one-page language reference. Of the 24 Zamak programs, 23 passed every test on the first try, and the last one passed after one fix. The Rust and Go versions passed all 24 tasks on the first try. For the method, tasks, and data, see [bench/ai/live](bench/ai/live/README.md).
- **Native, standalone output.** Zamak generates C and compiles it with Microsoft C++ (MSVC), GCC, or Clang. The executable runs without Zamak or Rust installed. On the integer benchmarks in [docs/PERFORMANCE.md](docs/PERFORMANCE.md), its runtime performance is in the same range as Rust and Go.

## Install Zamak

To build Zamak from source, you need Rust stable and a C compiler. In the repository root, run:

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
zam build hello      # writes build/hello-<hash>(.exe)
zam run app.zm -- a b   # pass arguments to the program
```

`zam` writes build progress to standard error, so standard output contains only your program's output.

## Language tour

This program uses structs, error values, lists, maps, and string interpolation:

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

- **Types.** Zamak has `i64`, `f64`, `bool`, `string` (UTF-8), named structs, fixed arrays `[i64; N]`, lists `[T]`, and maps `{K: V}`. List elements and map values can be `i64`, `f64`, `bool`, or `string`, and map keys can be `i64` or `string`. A character literal such as `'#'` is an `i64` code point.
- **Numbers.** `f64` follows IEEE 754, so dividing by zero gives `inf` or `NaN`. Zamak never converts between `i64` and `f64` implicitly. Write `f64(n)` or `i64(x)`, where `i64(x)` truncates toward zero. `println` prints the shortest text that reads back as the same `f64`, such as `0.1` or `2.0`, and `fmt.fixed(x, 2)` rounds to two decimal places.
- **Control flow.** Use `if`, `else if`, `else`, `while`, `break`, and `continue`. To choose a value, write `let v = if c { a } else { b }`. Loop over a range with `for i in 0..n`, over a list with `for x in &list`, and over map keys in insertion order with `for k in &map`.
- **Errors.** A function that can fail returns `T!string` or `!string`. Use `fail` to return an error, `?` to pass it to the caller, and `catch e { ... }` to handle it in place.
- **Modules.** Each file is a module. Import one with `use name`, and export items with `pub fn` and `pub struct`.
- **Strings.** Join strings with `+`, compare them with `==` or `<`, and update variables with `+=`, `-=`, `*=`, `/=`, and `%=`. Interpolation accepts expressions such as `"{a + b}"` and `"{xs[0]}"`.
- **Standard library.** `std/io` reads standard input and prints to standard output or standard error, `std/fs` reads and writes whole files, and `std/env` reads arguments. `std/string` measures, compares, parses, trims, searches, replaces, converts case, slices, splits, and joins strings. `std/math` provides square roots, powers, logarithms, trigonometry, rounding, and the constants `math.pi`, `math.e`, and `math.inf`.

The [tutorial](TUTORIAL.md) (in Chinese) covers every feature with runnable examples. For a larger program, see [examples/md2html](examples/md2html), a Markdown-to-HTML converter with four modules and about 270 lines. It handles headings, lists, quotes, fenced code, inline formatting, links, and HTML escaping. To run it:

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

`zam check --json moved.zm` reports the same error as one line of JSON:

```json
{"command":"check","ok":false,"diagnostics":[{"file":"moved.zm","line":8,"column":5,"function":"main:main","code":"ZM301","category":"ownership","message":"use after move: name","suggestion":"borrow the value or move it only after the last use"}]}
```

For the list of error codes and the recommended loop for AI agents (check, make the smallest fix, format, check, build, and run), see [docs/AI_WORKFLOW.md](docs/AI_WORKFLOW.md) (in Chinese). To give a coding agent instructions for this loop, create your project with `zam new <dir> --template ai`, which adds an `AI.md` file.

## Current limitations

Zamak doesn't support generics, enums and `match`, closures, threads, a foreign function interface (FFI), lists or maps of structs, nested containers, networking, or directory listing yet. There's no package manager, language server, or read-eval-print loop (REPL) yet. When you use an unsupported construct, `zam check` reports a diagnostic instead of compiling incorrect code.

## Editor and agent support

- **VS Code.** The extension in [editors/vscode](editors/vscode) adds syntax highlighting and bracket and quote matching. It's declarative and has no runtime dependencies.
- **AI coding agents.** [.agents/skills/zamak/SKILL.md](.agents/skills/zamak/SKILL.md) describes the commands, syntax, and common diagnostics for agents that read `.agents/skills`.

## Contribute

For the development guide, see [CONTRIBUTING.md](CONTRIBUTING.md) (in Chinese). Before you submit a change, make sure these commands pass in the `zam` folder:

```sh
cargo test --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
```

## License

You can use Zamak under either the [MIT license](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE). Unless you state otherwise, you license your contributions under the same terms.
