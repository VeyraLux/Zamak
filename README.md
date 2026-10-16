# Zamak

English | [中文](README.zh-CN.md)

Zamak is a statically typed language that compiles to native executables. It's built for a fast loop of editing, checking, and building code, for both people and AI coding agents.

[![CI](https://github.com/VeyraLux/Zamak/actions/workflows/ci.yml/badge.svg)](https://github.com/VeyraLux/Zamak/actions/workflows/ci.yml)

<img width="96" height="96" alt="Zamak" src="ico/Zamak_256x256.png" />

This program counts how often each word appears in its input:

```zamak
use std/io
use std/string

fn main() -> !string:
    let text = io.read_all()?
    let words = string.words(&text)
    let mut counts: {string: i64} = {}

    for word in &words:
        counts[word] = counts.get(word, 0) + 1

    for word in &counts:
        let n = counts[word]
        println("{word}: {n}")
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
- **Ownership without a garbage collector.** Strings, structs, lists, and maps move when you assign them. A borrow (`&x` or `&mut x`) lasts for one call, or until the end of the block for a read-only local such as `let entry = &entries[i]`, so you never write lifetime annotations. Zamak checks integer overflow, division by zero, and out-of-bounds access at run time, and it removes the checks that a conservative range analysis proves redundant.
- **Diagnostics built for tools.** Every error has a stable code such as `ZM301`, a category, a position, and usually a suggested fix. `zam check --json` writes them as one line of JSON. Zamak reports type errors per function, so an agent can fix several of them in one pass.
- **Learnable from one page.** In a benchmark with three models and eight small command-line tasks, each model read only a one-page language reference. All 24 Zamak programs, written in the current indentation-based syntax, passed every test on the first try. The Rust and Go versions also passed all 24 tasks on the first try, so these tasks are too easy to rank the languages. For the method, tasks, and data, see [bench/ai/live](bench/ai/live/README.md).
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
zam test hello       # runs every fn test_...() in the project
zam build hello      # writes build/hello-<hash>(.exe)
zam run app.zm -- a b   # pass arguments to the program
```

`zam` writes build progress to standard error, so standard output contains only your program's output.

## Language tour

This program uses structs, error values, lists, maps, and string interpolation:

```zamak
use std/string

struct Person:
    name: string
    age: i64

// Errors are values: `T!string` returns either a T or an error message.
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

- **Blocks.** Blocks use indentation, like Python. A line that ends with `:` opens a block, and the block's lines are indented 4 spaces deeper. A block with one statement can stay on the same line, such as `if n < 1: return n`. Write `pass` for an empty block. Braces only appear in maps and string interpolation. Indent with spaces; `zam fmt` replaces tabs and normalizes indentation and blank lines.
- **Types.** Zamak has `i64`, `f64`, `bool`, `string` (UTF-8), named structs, fixed arrays `[i64; N]`, lists `[T]`, and maps `{K: V}`. List elements and map values can be `i64`, `f64`, `bool`, `string`, a struct, or another list or map, such as `[[i64]]` or `{string: [string]}`, and map keys can be `i64` or `string`. `grid[i][j]` reads and writes an inner element. Lists have `push`, `pop`, `insert`, `remove`, `sort`, `sort_by`, `reverse`, `contains`, `index_of`, `slice`, and `extend`; maps have `get`, `contains`, `remove`, `keys`, and `values`. A character literal such as `'#'` is an `i64` code point.
- **Numbers.** `f64` follows IEEE 754, so dividing by zero gives `inf` or `NaN`. Zamak never converts between `i64` and `f64` implicitly. Write `f64(n)` or `i64(x)`, where `i64(x)` truncates toward zero. `println` prints the shortest text that reads back as the same `f64`, such as `0.1` or `2.0`, and `fmt.fixed(x, 2)` rounds to two decimal places.
- **Structs.** Declare `struct Point:` with one field per indented line, and build a value with `Point(x: 1, y: 2)`, naming every field. Function names start with a lowercase letter, so `Point(...)` always means a struct.
- **Methods.** Write methods inside the struct or enum block with `&self`, `&mut self`, or `self` as the first parameter, and call them as `p.norm()`. A function without `self`, such as `Point.new(1, 2)`, is called on the type.
- **Struct borrows and struct lists.** A parameter typed `&Point` borrows a struct for one call, and `&mut Point` lets the function change it. In a list of structs, read or write a field with `items[i].qty`, borrow an element with `&items[i]`, and loop with `for item in &items`. Structs move on assignment, so take an element out with `items.pop()` or make a deep copy with `copy(&items[i])`. Structs, enums, lists, and maps compare with `==` and print with `println`, as in `Point(x: 1, y: 2)`.
- **Enums and match.** Declare `enum Shape:` with one variant per indented line, such as `Circle(r: f64)` or `Empty`, and build values with `Shape.Circle(1.0)` or `Shape.Empty`. A `match` lists one arm per line, such as `Circle(r): 3.14 * r * r`, or ends with `_: ...`. Zamak reports an error when an arm is missing. A `match` borrows the value, so arms read the data without moving it. A `match` can also branch on integers, strings, and booleans, as in `1 | 2: "small"`.
- **Control flow.** Use `if`, `elif`, `else`, `while`, `break`, and `continue`. To choose a value, write `let v = if c: a elif d: b else: e`. Combine conditions with `and`, `or`, and `not`, which follow Python's precedence and short-circuit. Loop over a range with `for i in 0..n`, over a list with `for x in list` or `for i, x in list`, over a call result such as `for name in fs.list(dir)?`, and over a map in insertion order with `for k in &map` or `for k, v in &map`. Write `&mut` to change elements in place, as in `for x in &mut list: x += 1`.
- **Errors.** A function that can fail returns `T!E` or `!E`, where `E` is `string`, a struct, or an enum. Use `fail` to return an error, `?` to pass it to the caller, and `catch e:` followed by an indented block to handle it in place, for example with `match e`. When the call's value is used, the block must leave with `return`, `fail`, `break`, or `continue`; when it isn't, as in `fs.remove(&p) catch e:`, the program continues after the block.
- **Generic functions.** Write `fn largest[T](items: &[T]) -> T`. Zamak infers `T` at each call and compiles a separate version per type.
- **Generic types and Option.** Declare `struct Stack[T]` or `enum Tree[T]` and write the type as `Stack[string]`. Methods inside a generic type can use `T`. The built-in `Option[T]` has the variants `Option.Some(value)` and `Option.None`.
- **Function values.** A function type is written `fn(i64) -> bool`. Pass a named function by name, or write an anonymous fn such as `fn(x): x * 2`. For longer code, declare a `fn` inside the function body. Closures can read the enclosing function's variables and change the ones declared with `let mut`. A closure can be called, bound with `let`, and passed as an argument, but never stored or returned, so it never outlives the variables it uses and no garbage collector is needed. Lists and maps can hold functions, such as `[fn(i64) -> i64]`, and a generic function becomes a value when the expected function type is known. Call an element directly with `steps[0](x)`. A function type can fail, as in `fn(&string) -> i64!string`; calls through such a value need `?` or `catch` like any other failing call.
- **Tests.** `assert cond` or `assert cond, "message"` stops the program when `cond` is false and prints the condition, and for `a == b` both values. `zam test` runs every function named `test_...` in its own process, so a crash fails only that test, and prints a summary. Pass a word to run only matching tests: `zam test . parse`.
- **Tuples.** Group values as `(3, "three")` with type `(i64, string)`, read items with `t.0`, and return several values from a function: `let (q, r) = divmod(17, 5)`.
- **Constants.** Declare `const LIMIT = 100` at the top level. Zamak computes the value at compile time from literals, earlier constants, and operators.
- **Modules.** Each file is a module. Import one with `use name`, and export items with `pub fn`, `pub struct`, and `pub const`.
- **Packages.** Find libraries in the [package index](https://github.com/VeyraLux/zamak-index) with `zam search json`, and add one with `zam add mathx`, which writes `mathx = "^1.4.0"`. Version requirements follow Cargo: `^1.2`, `~1.2`, `=1.2.3`, or `>=1.0, <2.0`, and `zam` picks the highest version every package accepts. Remove a dependency with `zam remove mathx`. You can also add a library straight from a Git repository with `zam add mathx https://github.com/you/mathx v1.0`, or from a local directory with `zam add util ../util`. To share your own library, give `zam.toml` a `version`, tag it, and run `zam publish`. `zam.toml` lists dependencies under `[dependencies]`, and `zam.lock` pins each Git dependency to a commit so every build uses the same code, offline too. Import a package's modules as `use mathx` or `use mathx/geo`. Run `zam update` to move to the latest commit of each `rev`.
- **C libraries.** Declare a C function with `extern fn sqrt(x: f64) -> f64` and call it like any other function. Parameters can be `i64`, `f64`, `bool`, or `&string` (passed as a `const char *` that ends with `\0`), and results can be those types or `string`. List C files, include directories, and libraries in a `[native]` section of `zam.toml`, such as `sources = ["c/shim.c"]` and `libraries = ["sqlite3"]`. A package can carry its own `[native]` section, so a Zamak library can wrap a C library for other projects. See `examples/ffi`.
- **Operators.** Integers support the bitwise operators `&`, `|`, `^`, `~`, `<<`, and `>>` with Python's precedence, and literals such as `0xFF`, `0b1010`, and `0o17`.
- **Strings.** Call string functions as methods, such as `name.upper()`, `line.trim().len()`, or `csv.split(",")`, without an import; `len()` counts bytes and `char_len()` counts characters. Join strings with `+`, compare them with `==` or `<`, and update variables with `+=`, `-=`, `*=`, `/=`, and `%=`. Interpolation accepts expressions such as `"{a + b}"` and `"{xs[0]}"`.
- **Printing.** `print(x)` and `println(x)` both print a value and a newline, like Python's `print`. `io.print` prints without a newline.
- **Standard library.** `std/io` reads standard input and prints to standard output or standard error, `std/fs` reads, writes, and appends files, lists and creates directories, and renames and removes entries, `std/path` joins and splits paths, and `std/env` reads arguments and environment variables and sets the exit status. `abs`, `min`, and `max` work on both `i64` and `f64` without an import. `std/string` measures, compares, parses, trims, searches, replaces, converts case, slices, splits, joins, pads, and repeats strings, and converts them to and from characters, code points, and UTF-8 bytes. `std/math` provides square roots, powers, logarithms, trigonometry, rounding, and the constants `math.pi`, `math.e`, and `math.inf`. `std/time` reads the clock, measures durations, sleeps, and formats dates, and `std/random` draws integers and floats, with `random.seed(n)` for repeatable runs.

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

Zamak doesn't support threads or networking yet. C functions take and return only numbers, booleans, and strings; pass pointers and structs through a small C wrapper. There's no language server or read-eval-print loop (REPL) yet. When you use an unsupported construct, `zam check` reports a diagnostic instead of compiling incorrect code.

## Editor and agent support

- **VS Code.** The extension in [editors/vscode](editors/vscode) adds syntax highlighting, bracket and quote matching, automatic indentation after a line that ends with `:`, and folding by indentation. It's declarative and has no runtime dependencies.
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
