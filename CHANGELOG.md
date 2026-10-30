# Changelog

All notable changes to Zamak. This project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
and this file is the source for the text of each GitHub release.

<!-- 中文版在下半部分。Chinese version: scroll down or search for "## 0.1.1（中文）". -->

## Unreleased

- The VS Code extension finds `zam` when you installed it with npm. Before, it looked only for a
  program named `zam` on `PATH`, and on Windows npm installs a `zam.cmd` wrapper that VS Code
  can't start, so there was no completion. It now starts the compiler inside npm's package, also
  looks in npm's global packages, the install scripts' folders, and Cargo's, and starts the
  language server by itself when **Install with npm** finishes.
- `input()` and `input(prompt)` read a line of input with no import, as in Python, so reading is
  as easy as printing. The prompt appears without a newline, and the line comes back without its
  line ending. If the input ends first, the program stops with an error, as an uncaught
  `EOFError` does; use `io.read_line` to read until the input ends.
- Using `?` in a function that can't fail now says so and suggests the fix, such as declaring
  `-> !string`, and names the function the way you wrote it (`int`, `fs.read`) instead of its
  internal name (`std/string:parse_i64`). The message was also missing its Chinese translation.
- Structs can define operators with Python's method names: `a + b` calls `a.__add__(b)`, and
  `__sub__`, `__mul__`, `__truediv__`, `__floordiv__`, `__mod__`, `__lt__`, `__le__`, `__gt__`,
  `__ge__`, `__eq__` (with `!=` as its opposite), and `__neg__` work the same way. Using an
  operator a struct doesn't define names the method to add.
- `end - start` on two `DateTime` values gives a `TimeDelta`, as Python's `timedelta` does, with
  `days`, `seconds`, `microseconds`, and `total_seconds()`. It counts whole microseconds, so nothing
  is rounded. `datetime.timedelta(days=1, hours=2)` makes one, `t + delta` and `t + -delta` move a
  time, and `<`, `==`, and the other comparisons compare the actual moments, even across time zones.
  `Date` values subtract and compare the same way.
- `fn __str__(&self) -> string` decides what `println(x)`, `str(x)`, and `f"{x}"` show for a struct,
  as in Python. A `TimeDelta` now prints as Python's does, such as `2 days, 1:30:15.500000`.
- `std/bigint`: integers of any size, as Python's `int`. `bigint.parse(text)` and `bigint.from_int(n)`
  make one, and `+`, `-`, `*`, `//`, `%`, `-x`, comparisons, `pow`, `abs`, and `to_int` work on it.
  Results are exact, `//` rounds down as in Python, and `println` shows the digits.
- Fixed: checking a long expression took twice as long for each extra term, so `a + b + ...` with
  22 terms took 20 seconds and 40 terms never finished. It's now linear: 300 terms check in a
  fraction of a second. Very deep expressions no longer overflow the stack either.
- Type errors name the types you wrote: `this value is i64, but P is expected` instead of
  `string type mismatch`, and `P` instead of `main:P`.

## 0.2.0

Released 2026-10-24.

0.1 could compute things and print them. 0.2 is about the programs it couldn't write yet: one that
calls a web API, keeps data in a database, reads command-line options, or opens a window. Most of
what's new came from writing four real programs in Zamak and fixing whatever got in the way. They're
in `examples/apps/`: a multithreaded link checker, a to-do list, a notes REST service on SQLite, and
a desktop ledger.

A small program that uses three of the new modules:

```zamak
use std/cli
use std/env
use std/http
use std/json

struct Options:
    repo: string

struct Repo:
    full_name: string
    stargazers_count: i64

fn main() -> !string:
    let o: Options = cli.parse_or_exit(env.args())
    let r = http.get(f"https://api.github.com/repos/{o.repo}", headers={"User-Agent": "zamak"})?
    let repo: Repo = json.decode(r.text()?)?
    println(f"{repo.full_name} has {repo.stargazers_count} stars")
```

### Upgrading from 0.1

Three changes can break existing code.

- `xs.remove(i)` used to remove the element at index `i`. List methods now mean what they mean in
  Python: `remove(x)` removes the first element equal to `x`, and `pop(i)` removes and returns the
  element at `i`. On a `[i64]`, `xs.remove(0)` still compiles but now looks for the value 0, so
  search your code for `.remove(` and change the index calls to `pop`. If the value isn't in the
  list, the program stops with an error, as Python raises `ValueError`.

  ```zamak
  let mut names = ["Ada", "Grace", "Ada"]
  let first = names.pop(0)   # was names.remove(0)
  names.remove("Ada")        # removes the first "Ada" that's left
  ```

- `Json.Number(f64)` is now two variants, `Json.Int(i64)` and `Json.Float(f64)`. `1` and `1.0` keep
  their types through `loads` and `dumps`, and an integer too large for `i64` is an error instead
  of a rounded float. A `match` arm on `Json.Number(n)` becomes two arms.

- A list literal of integers without an annotation is now a list, as in Python. `let xs = [3, 9, 4]`
  used to make a fixed-size array, which couldn't grow, couldn't be passed to a `&[i64]` parameter,
  and didn't work with `sum` or `sorted`. Now `xs.append(5)`, `sum(xs)`, and `largest(xs)` all work.
  Like any list, it moves on assignment, so `let b = xs` leaves `xs` unusable. If you need a
  fixed-size array, write its length in the type: `let grid: [i64; 9] = ...`.

The build cache format changed too, so the first build after upgrading compiles everything again.

### Language

- Default and keyword arguments, as in Python: `fn connect(host: &string, port: i64 = 80)`, then
  `connect("a", timeout=1.5)`. A default can be a constant, `Option.None`, or a list or map of
  constants such as `tags: &[string] = []`. Each call gets a fresh default, so calls never share
  one mutable list. Methods take keyword arguments too, and `Point(x=1, y=2)` builds a struct.
- Sets. `{1, 2, 3}` has the type `{i64}` and supports `add`, `remove`, `in`, `union`,
  `intersection`, `difference`, `issubset`, and `sorted(s)`.
- Interfaces. `interface Shape:` lists methods, and any type that has them satisfies it. You don't
  write an `impl`. Use one as a type to keep different types in one list (`[Shape]`), or as a bound
  (`fn total[T: Shape]`), which compiles once per type.
- `bytes` for raw data. `b"GIF\x89"`, `s.encode()`, and `b.decode()` work as in Python, indexing gives
  an integer from 0 to 255, and storing anything outside that range stops the program.
- More string literals: raw strings `r"\d+"`, triple-quoted strings `"""say "hi" now"""`, and the
  escapes `\xNN`, `\0`, `\uXXXX`, and `\UXXXXXXXX`. Interpolation takes any expression, including
  string literals and slices: `f"{d["key"]} {s[:5]}"`.
- Module constants can be lists and maps. `const NAMES = ["Food", "Home"]` works with `for`,
  indexing, methods, and `&` arguments.
- `?` and `catch` work after method calls and chains, as in `line.parse_i64()?` and
  `buf[:n].decode() catch e: "?"`. A multi-line `catch` block can end with a value, `[]` and `{}`
  work as fallbacks, and `catch` blocks can nest.
- A struct, enum, or tuple that holds only numbers, booleans, and other plain values is copied on
  assignment, like a number, so `let b = a` leaves `a` usable.
- Fewer annotations. A list literal of strings, floats, structs, or lists knows its type
  (`let names = ["Ada", "Grace"]`). A type parameter that appears only in the return type is read
  from the annotation, for functions and methods alike: `let q: thread.Queue[i64] = thread.queue()`.
- `match` on the result of a call gives each arm its own copy, so you can change or move it:
  `match jobs.get(): Some(job): done.append(job)`. `match` on a variable still borrows.
- More everyday code compiles: indexing, slicing, and field access on a call result
  (`line.split(",")[0]`, `user().name`), methods on struct literals and on parenthesized values
  (`(a + b).upper()`), `let arg = args[i]` for a string element, and moving a field out of a
  struct you own.
- `kept fn(...)` marks a parameter whose function is stored for later. It only accepts functions
  that don't capture local variables.

### Standard library

New modules, named after their Python counterparts:

| Module | What it does |
| --- | --- |
| `std/http` | Requests in the style of `requests`: `http.get`, `http.post`, `http.post_json`, with `params`, `headers`, `timeout`, and redirects. `http.serve` handles requests on several threads (`workers=n`). `quote`, `unquote`, `parse_query`, and `urljoin` match `urllib.parse`. |
| `std/net` | TCP connections and listeners that close when they go out of scope, plus `start_tls` and `accept_tls`. |
| `std/json` | `loads` and `dumps`, and `json.encode` / `json.decode` for any struct, list, map, `Option`, or enum. A decode error names the path, such as `[0].tags[1]`. |
| `std/sqlite` | SQLite 3.54.1, built into `zam`. `db.query` reads rows straight into structs, and integers keep every digit. |
| `std/cli` | Declare a struct and `cli.parse(args)` fills it from the command line. Field types decide how options are read; `--help` comes for free. |
| `std/thread` | `thread.spawn(work, args...)` and `join()`, in the style of `executor.submit`. `thread.Queue[T]` carries values between threads. Threads share nothing that can change. |
| `std/datetime` | `Date` and `DateTime`, `today()`, `now()`, ISO parsing, `strftime`, and time zone offsets. |
| `std/process` | Run a program and read its output: `run`, `call`, `check_output`. |
| `std/re` | Regular expressions on a linear-time engine, so no pattern can hang your program. No backreferences or lookaround. |
| `std/hashlib`, `std/base64`, `std/bytes` | SHA-256, SHA-1, MD5, and HMAC-SHA256; Base64; `from_hex`, `hex`, and `zeros`. |

HTTPS works out of the box. `zam` carries Mbed TLS 3.6 and checks certificates against the
system's root store (or `SSL_CERT_FILE`). `http.serve` takes `cert_file` and `key_file` to serve
HTTPS.

Smaller additions: string methods `rfind`, `removeprefix`, `removesuffix`, `capitalize`, `title`,
`center`, `zfill`, `isupper`, `islower`, `find(sub, start)`, and a fill character for `ljust`,
`rjust`, and `center`; `append` on lists; `read_bytes`, `write_bytes`, and `append_bytes` in
`std/fs`; `env.program()` for the program's name; `json.shape(sample)` for libraries that read data
by type. Part of the standard library is now written in Zamak itself and compiled with your program.

### Desktop apps

`std/ui` builds windows with each system's own controls: Win32 on Windows, AppKit on macOS, and
GTK 3 on Linux. It has labels, buttons, entries, text areas, check boxes, drop-down lists, option
buttons, sliders, spin boxes, progress bars, tables, tabs, forms, grids, and file and message
dialogs. The program's state is a struct you pass to `app.run`, and handlers receive it as `&mut`.

When the built-in controls aren't enough, `ui.canvas()` lets you draw your own. The `Painter` draws
paths, rounded rectangles, circles, gradients, and text, and `app.on_mouse` and `app.on_key` bring in
mouse and keyboard input.
`std/widgets` is a set of controls drawn this way (switches, sliders, progress rings, line and bar
charts, cards, and badges) in light and dark themes. They're plain Zamak, so you can copy one and
change it. See `examples/ui/widgets.zm`.

### Calling C

- `extern type Db: "sqlite3", close = sqlite3_close` declares a C pointer that Zamak closes exactly
  once. C functions return it as `Option[Db]`, where NULL is `None`.
- `[native] headers` compiles calls against the library's own header, so `int` and `size_t`
  parameters work without a wrapper.
- `[native]` also compiles C++ and, on macOS, Objective-C, and `frameworks = ["Foundation"]` links
  macOS frameworks. Native sources compile in parallel.

### Builds and tools

- Rebuilds are faster. `zam` remembers which C compiler it found, so a build where nothing changed
  takes about 4 ms on Windows, down from about 100 ms.
- Builds leave one CPU core free and run the C compiler at below-normal priority, so a cold build no
  longer makes the desktop stutter.
- Prebuilt compilers for two more platforms: macOS on Intel and Linux on arm64, from npm and the
  install script.
- `THIRD-PARTY-NOTICES.md` lists the licenses of the bundled Mbed TLS, libui-ng, and SQLite, and
  ships in every package. If you distribute a program that uses HTTPS, TLS, or `std/ui`, include the
  matching notice with it.
- The Zamak Book has a new appendix, "How Zamak works". It follows a program from source to
  executable and shows the C it turns into, how memory is freed, and how the build cache decides
  what to rebuild.

### Fixes

- On Windows, builds running at the same time could report `corrupt cache entry`. Under load, MSVC's
  multithreaded code generation sometimes produced different instructions for the same C file, so
  `zam` now compiles each file on one code generation thread.
- On Linux, a window's only child (such as a canvas) filled just its natural width instead of the
  window. Changing a table's rows from code also triggered `on_change`. Both now behave as they do
  on Windows and macOS.
- `datetime.now()` and other local-time functions could return the wrong offset when several threads
  called them at once.
- A `CC` wrapper script that adds flags, such as `exec gcc -fsanitize=thread "$@"`, shared cached
  objects with plain `gcc`. An empty `CC` now means the default compiler, as in `make`.
- `obj.method(x)?` crashed the compiler when the method's type parameter had the same name as
  another type's, such as `T`.
- `catch` inside an argument that's passed by reference, as in
  `int(env.var("PORT") catch e: "8080")`, was rejected.
- Clearer errors: assigning a list to a string variable reports a type mismatch instead of a borrow
  error, and `let x = f()` where `f` returns nothing says so.

## 0.1.1

Released 2026-10-19.

A packaging fix, published the same day as 0.1.0. Nothing in the language changed, so programs
and caches from 0.1.0 carry over unchanged.

- **`zam upgrade` now prints the correct package name.** The 0.1.0 binaries told you to run
  `npm install -g zamak`, which is not the published package. It now prints
  `npm install -g @veyralux/zamak`.
- **The VS Code extension is told the real package name too**, so its one-click install and its
  "install with npm" hint point at a package that exists.
- **The benchmark gained two size profiles.** `bench/compare` used to force `CC=gcc`, so its
  reported binary size was the gcc number; on Windows the shipped compiler uses MSVC, which links
  the C runtime statically and produces a much larger file. `run.py` can now measure both
  (`zamak-msvc`) and a maximally stripped Rust build (`rust-min`) for comparison.
- **The README's size claims are stated per platform** instead of quoting a single number.

## 0.1.0

Released 2026-10-19.

The first release: a statically typed language with Python-shaped syntax that compiles to a
standalone native executable, together with the `zam` toolchain, a VS Code extension, and an
MCP server for AI coding agents.

### Language

- **Indentation-based blocks**, like Python. A line ending in `:` opens a block; `pass` writes an
  empty one. Combine conditions with `and`, `or`, and `not` following Python's precedence.
- **Static types** with inference: `i64`, `f64`, `bool`, `string` (UTF-8), structs, enums,
  fixed arrays `[i64; N]`, lists `[T]`, maps `{K: V}`, tuples, and `fn(...) -> T`.
- **Ownership and borrow checking without a garbage collector.** Values move on assignment,
  `&` and `&mut` borrow for one call, and closures may not outlive the variables they use.
- **Errors are values.** A function returns `T!E`; `fail` returns an error, `?` propagates it, and
  `catch e:` handles it in place.
- **Generics** for functions (`fn largest[T](items: &[T]) -> T`) and for structs and enums
  (`struct Stack[T]`). Each call site gets a version specialized to its types. Built-in `Option[T]`.
- **Structs and enums with methods**, `match` with payload bindings and literal arms, and struct
  literals that name every field.
- **Closures and function values**: `fn(i64) -> bool`, anonymous `fn(x): x * 2`, functions declared
  inside a function, and lists or maps of functions.
- **UTF-8 strings** with interpolation: `"{name}"`, `"{a + b}"`, `"{xs[0]}"`, `"{{"` for a literal
  brace, and Python string methods on literals such as `"a,b".split(",")`.
- **Format specifiers in interpolation** (new in this release): `f"{ratio:.3}"`, `f"{n:>5}"`,
  `f"{name:*^7}"`, `f"{count:03}"`, and `f"{x:08.2}"`, following
  `[[fill]align][width][.precision]`. A precision is only accepted for `f64`, so an `i64` is an
  error instead of a silent rounding.
- **Recursive functions inside a function** (new in this release): a `fn` declared in a function
  body may call itself. Mutual recursion, where the two functions call each other, is still not
  supported; declare those at the top level.
- **Methods on a generic type may declare their own type parameters** (new in this release), such as
  `fn map[U](&self, f: fn(T) -> U) -> U`. `U` is inferred from the arguments at each call.
- **Tests and assertions**: `assert cond, "message"` and `test_...` functions that `zam test` runs
  one process each.
- **Modules and packages**: every file is a module, `pub` exports it, `zam.toml` declares
  dependencies, and `zam.lock` pins Git dependencies to a commit for reproducible offline builds.
- **C libraries**: declare `extern fn sqrt(x: f64) -> f64` and list C sources, includes, and
  libraries in a `[native]` section of `zam.toml`.
- **Standard library**: `std/string`, `std/io`, `std/fmt`, `std/fs`, `std/path`,
  `std/env`, `std/math`, `std/time`, and `std/random`, plus Python's `len`, `str`, `int`, `float`,
  `abs`, `min`, `max`, `sum`, and `sorted` with no import.

### The `zam` command

- `check`, `build`, `run`, `test`, and `fmt` for the edit loop; `new` to create a project;
  `add`, `remove`, `update`, `search`, and `publish` for packages; `doc`, `lsp`, `mcp`, `upgrade`,
  and `demo`.
- **Every module is cached by content**, so building the same code twice compiles it once, and
  changing one module rebuilds only that module and its dependents. `zam check` on a 5,000-line
  project takes under 0.1 s.
- **`zam doc` is new in this release.** It turns `pub` declarations and the `#` comments above them
  into a single self-contained HTML API page (no JavaScript, light and dark), listing fields,
  enum variants, and methods under their type. `zam doc . site` picks the output folder.
- **Bilingual messages.** `--help` lists every subcommand with a description in both languages, and
  the choice follows `ZAMAK_LANG`. Diagnostics, suggestions, and the runtime panics of compiled
  programs (index out of bounds, division by zero, missing key, and so on) are all translated.
- **Steadier diagnostic codes.** Errors no longer carry a redundant `main:` prefix in single-file
  mode, real counterexamples never fall through to the catch-all ZM999, and the codes and their
  fixes are documented in the Book's appendix and served to AI agents over MCP.

### Editor support

- Syntax highlighting, comment toggling, bracket pairing, and automatic indentation after `:`.
- Diagnostics as you type, including unsaved changes, completion, hover, go to definition, and
  Format Document, all from `zam lsp`.
- **Signature help** while typing a call, showing the parameter list and highlighting the argument
  you are on, for local functions, imported functions, struct methods, and the standard library.
- **Rename** and **Find All References** for names in the current file. Text with the same name
  inside comments and strings is left alone.
- **Completion after `x.`** uses the type the checker inferred rather than guessing from the name,
  falling back to the old behavior while the file does not yet check.
- **One-click install.** When `zam` is not on `PATH`, the extension offers **Install with npm** and
  a command to look for the compiler again. The status bar shows the `zam` version in use.

### AI coding agents

- `zam mcp` speaks the Model Context Protocol on stdio and offers **eight tools**: `check`, `build`,
  `run`, `test`, `format`, `explain`, `docs`, and `hover` (new in this release), which returns the
  type or signature at a file, line, and column.
- **MCP resources** (new in this release): `zamak://reference` serves a one-page language reference
  and `zamak://diagnostics` the table of error codes, so an agent can read the language without
  shelling out.
- `.agents/skills/zamak/SKILL.md` and `zam new --template ai` cover the same ground for agents that
  read the repository.
- Measured on the offline benchmark in `bench/ai` and on live models: all 24 Zamak programs across
  three models were solved, 19 of them on the first try. The most common first-try failure is a
  missing `use std/string`, which the diagnostic names directly.

### Performance

Zamak compiles through C11 into a standalone native executable, so there is no runtime to ship and
no garbage collector to pause. On the comparison suite it runs in the same range as C on the
compute-bound programs while checking integer overflow and array bounds by default, and it produces
small executables with a small memory footprint. Builds stay fast because every module is cached by
content. The charts, the current numbers, and the method are in [README.md](README.md),
[docs/PERFORMANCE.md](docs/PERFORMANCE.md), and [bench/compare](bench/compare/README.md).

### Notes

- Threads, networking, a REPL, and a debugger are not implemented yet. `zam check` reports an
  unsupported construct rather than compiling it into incorrect code.
- This release removes an unused 1.7 MB legacy logo from `ico/`.

## 0.1.0-rc.1

The first public preview. It shipped the language as described above, the package index, the C
interop layer, and the `zam lsp` server. Everything under "new in this release" above landed after
this preview.

---

## 未发布（中文）

- VS Code 扩展能找到用 npm 装的 `zam` 了。以前它只在 `PATH` 里找名为 `zam` 的程序，而 npm 在 Windows
  上装的是 VS Code 启动不了的 `zam.cmd` 包装，所以没有补全。现在它直接启动 npm 包里的编译器，还会去 npm
  的全局包、安装脚本和 Cargo 用的文件夹里找；点了“用 npm 安装”之后，装好会自动启动语言服务器。
- 新增内置的 `input()` 和 `input(prompt)`，和 Python 一样不用导入就能读一行输入，读和写一样方便。提示不换行，
  读到的这一行去掉了行尾的换行。输入先结束时程序报错停下，就像没人接住的 `EOFError`；要一直读到输入结束，
  用 `io.read_line`。
- 在不会失败的函数里用 `?` 时，报错会说清原因并给出改法（例如声明 `-> !string`），函数名也按你写的样子显示
  （`int`、`fs.read`），不再显示内部名字（`std/string:parse_i64`）。这条报错以前还缺中文翻译。
- 结构体可以用 Python 的方法名定义运算符：`a + b` 调用 `a.__add__(b)`，`__sub__`、`__mul__`、`__truediv__`、
  `__floordiv__`、`__mod__`、`__lt__`、`__le__`、`__gt__`、`__ge__`、`__eq__`（`!=` 是它的反面）和 `__neg__` 同理。
  对没定义的运算符，报错会说出要加哪个方法。
- 两个 `DateTime` 相减（`end - start`）得到 `TimeDelta`，和 Python 的 `timedelta` 一样，有 `days`、`seconds`、
  `microseconds` 和 `total_seconds()`。按整数微秒计算，不会四舍五入。`datetime.timedelta(days=1, hours=2)` 创建时长，
  `t + delta`、`t + -delta` 移动时间，`<`、`==` 等比较的是实际时刻，时区不同也没关系。`Date` 也能这样相减和比较。
- 结构体定义 `fn __str__(&self) -> string` 后，`println(x)`、`str(x)`、`f"{x}"` 显示它返回的文字，同 Python。
  `TimeDelta` 因此显示成 Python 的样子，例如 `2 days, 1:30:15.500000`。
- 新增 `std/bigint`：任意大小的整数，同 Python 的 `int`。用 `bigint.parse(text)`、`bigint.from_int(n)` 创建，
  支持 `+`、`-`、`*`、`//`、`%`、`-x`、比较、`pow`、`abs`、`to_int`。结果都是精确的，`//` 同 Python 向下取整，
  `println` 显示十进制数字。
- 修复：表达式每多一项，检查时间就翻一倍，`a + b + ...` 22 项要 20 秒，40 项一直卡住。现在是线性的：
  300 项不到一秒就检查完。嵌套很深的表达式也不会再栈溢出。
- 类型报错写出你写的类型：显示“这个值是 i64，这里需要 P”，不再是“字符串类型不匹配”；类型名显示 `P`，不再是 `main:P`。

## 0.2.0（中文）

2026-10-24 发布。

0.1 能算东西、能打印。0.2 补的是以前写不出来的程序：调网络接口、把数据存进数据库、读命令行选项、开一个窗口。
这一版的新东西大多来自用 Zamak 写四个真实程序时碰到的问题，程序都在 `examples/apps/` 里：多线程的链接检查器、
命令行待办清单、基于 SQLite 的笔记 REST 服务，以及一个桌面记账本。

下面这个小程序用到了三个新模块：

```zamak
use std/cli
use std/env
use std/http
use std/json

struct Options:
    repo: string

struct Repo:
    full_name: string
    stargazers_count: i64

fn main() -> !string:
    let o: Options = cli.parse_or_exit(env.args())
    let r = http.get(f"https://api.github.com/repos/{o.repo}", headers={"User-Agent": "zamak"})?
    let repo: Repo = json.decode(r.text()?)?
    println(f"{repo.full_name} has {repo.stargazers_count} stars")
```

### 从 0.1 升级

有三处改动可能让旧代码出问题。

- 以前 `xs.remove(i)` 按下标删除。现在列表方法与 Python 同义：`remove(x)` 删掉第一个等于 `x` 的元素，
  `pop(i)` 删掉并返回下标 `i` 处的元素。在 `[i64]` 上，`xs.remove(0)` 仍然能编译，但意思变成了“删掉值 0”，
  所以请搜一下代码里的 `.remove(`，把按下标删除的地方改成 `pop`。要删的值不在列表里时，程序会报错停下，
  和 Python 抛出 `ValueError` 一样。

  ```zamak
  let mut names = ["Ada", "Grace", "Ada"]
  let first = names.pop(0)   # 以前写 names.remove(0)
  names.remove("Ada")        # 删掉剩下的第一个 "Ada"
  ```

- `Json.Number(f64)` 拆成了 `Json.Int(i64)` 和 `Json.Float(f64)` 两个变体。`1` 和 `1.0` 经过 `loads`、`dumps`
  之后类型不变；超出 `i64` 范围的整数会报错，不会悄悄变成一个近似的小数。原来匹配 `Json.Number(n)` 的分支要拆成两个。

- 不写标注的整数列表字面量现在就是列表，和 Python 一样。以前 `let xs = [3, 9, 4]` 得到的是定长数组：不能增长，
  不能传给 `&[i64]` 参数，也不能用 `sum`、`sorted`。现在 `xs.append(5)`、`sum(xs)`、`largest(xs)` 都能用。
  和所有列表一样，赋值时会移动，`let b = xs` 之后 `xs` 就不能再用了。需要定长数组时，把长度写进类型：
  `let grid: [i64; 9] = ...`。

构建缓存的格式也变了，升级后第一次构建会全部重新编译。

### 语言

- 默认参数和关键字实参，写法同 Python：`fn connect(host: &string, port: i64 = 80)`，调用时写
  `connect("a", timeout=1.5)`。默认值可以是常量、`Option.None`，或由常量组成的列表和字典，例如
  `tags: &[string] = []`。每次调用拿到的都是新的默认值，不会出现几次调用共用同一个列表的问题。
  方法也能用关键字实参，结构体可以写 `Point(x=1, y=2)`。
- 集合。`{1, 2, 3}` 的类型是 `{i64}`，支持 `add`、`remove`、`in`、`union`、`intersection`、`difference`、
  `issubset` 和 `sorted(s)`。
- 接口。`interface Shape:` 列出一组方法，哪个类型有这些方法，就算实现了这个接口，不用写 `impl`。
  接口可以当类型用，把不同类型的值放进同一个列表（`[Shape]`）；也可以当约束用（`fn total[T: Shape]`），
  这时每种类型各编译一份。
- `bytes` 类型，用来存原始数据。`b"GIF\x89"`、`s.encode()`、`b.decode()` 与 Python 相同，下标读出 0 到 255 的整数，
  存入这个范围以外的数会让程序停下。
- 更多字符串写法：原始字符串 `r"\d+"`、三引号字符串 `"""say "hi" now"""`，以及转义 `\xNN`、`\0`、`\uXXXX`、
  `\UXXXXXXXX`。插值里可以写任何表达式，包括字符串字面量和切片：`f"{d["key"]} {s[:5]}"`。
- 模块常量可以是列表和字典。`const NAMES = ["Food", "Home"]` 能用在 `for`、下标、方法调用和 `&` 参数里。
- 方法调用和方法链后面也能接 `?` 和 `catch`，例如 `line.parse_i64()?`、`buf[:n].decode() catch e: "?"`。
  多行的 `catch` 块可以用一个值结尾，`[]` 和 `{}` 可以当兜底值，`catch` 块也可以嵌套。
- 只含数字、布尔值等简单值的结构体、枚举和元组，赋值时像数字一样复制，`let b = a` 之后 `a` 照样能用。
- 少写类型标注。字符串、小数、结构体或列表组成的列表字面量能自己推出类型（`let names = ["Ada", "Grace"]`）。
  只出现在返回类型里的类型参数，会从变量的标注里读出来，函数和方法都一样：`let q: thread.Queue[i64] = thread.queue()`。
- 对函数调用的结果做 `match` 时，每个分支拿到的是自己的一份，可以修改或移走：
  `match jobs.get(): Some(job): done.append(job)`。对变量做 `match` 仍然是借用。
- 更多常见写法能编译了：对调用结果取下标、切片、读字段（`line.split(",")[0]`、`user().name`），
  在结构体字面量和括号表达式上调方法（`(a + b).upper()`），用 `let arg = args[i]` 取出字符串元素，
  以及从自己拥有的结构体里移走一个字段。
- `kept fn(...)` 标记一个会被存起来以后再调用的函数参数，它只接受不捕获局部变量的函数。

### 标准库

新模块，名字与 Python 里对应的模块一致：

| 模块 | 用途 |
| --- | --- |
| `std/http` | 用法照着 `requests`：`http.get`、`http.post`、`http.post_json`，支持 `params`、`headers`、`timeout` 和重定向。`http.serve` 用多个线程处理请求（`workers=n`）。`quote`、`unquote`、`parse_query`、`urljoin` 与 `urllib.parse` 相同。 |
| `std/net` | TCP 连接和监听，离开作用域时自动关闭；还有 `start_tls` 和 `accept_tls`。 |
| `std/json` | `loads` 和 `dumps`；`json.encode` / `json.decode` 能转换任意结构体、列表、字典、`Option` 和枚举。解析出错时会指出位置，例如 `[0].tags[1]`。 |
| `std/sqlite` | 内置 SQLite 3.54.1。`db.query` 直接把行读成结构体，整数一位不丢。 |
| `std/cli` | 声明一个结构体，`cli.parse(args)` 就把命令行填进去。字段的类型决定选项怎么读，`--help` 自动就有。 |
| `std/thread` | `thread.spawn(work, args...)` 和 `join()`，用法照着 `executor.submit`。线程之间用 `thread.Queue[T]` 传值，不共享任何可变的东西。 |
| `std/datetime` | `Date` 和 `DateTime`、`today()`、`now()`、ISO 格式解析、`strftime`，以及时区偏移。 |
| `std/process` | 运行别的程序并读取输出：`run`、`call`、`check_output`。 |
| `std/re` | 正则表达式，用的是线性时间的引擎，什么样的表达式都不会让程序卡死。不支持反向引用和环视。 |
| `std/hashlib`、`std/base64`、`std/bytes` | SHA-256、SHA-1、MD5、HMAC-SHA256；Base64；`from_hex`、`hex`、`zeros`。 |

HTTPS 装好就能用。`zam` 自带 Mbed TLS 3.6，按系统的根证书（或 `SSL_CERT_FILE`）校验证书。
`http.serve` 传入 `cert_file` 和 `key_file` 就能提供 HTTPS 服务。

其他小的补充：字符串方法 `rfind`、`removeprefix`、`removesuffix`、`capitalize`、`title`、`center`、`zfill`、
`isupper`、`islower`、`find(sub, start)`，`ljust`、`rjust`、`center` 可以指定填充字符；列表有了 `append`；
`std/fs` 新增 `read_bytes`、`write_bytes`、`append_bytes`；`env.program()` 返回程序名；
`json.shape(sample)` 给按类型读数据的库用。标准库有一部分改用 Zamak 自己写，和你的程序一起编译。

### 桌面程序

`std/ui` 用各系统自己的控件做窗口：Windows 上是 Win32，macOS 上是 AppKit，Linux 上是 GTK 3。
有标签、按钮、输入框、多行文本框、复选框、下拉列表、单选按钮、滑块、数字框、进度条、表格、标签页、表单、网格，
以及文件和消息对话框。程序的状态是一个结构体，交给 `app.run`，事件处理函数以 `&mut` 拿到它。

自带的控件不够用时，可以用 `ui.canvas()` 自己画。`Painter` 能画路径、圆角矩形、圆、渐变和文字，
`app.on_mouse` 和 `app.on_key` 接收鼠标和键盘输入。`std/widgets` 是一套这样画出来的控件（开关、滑块、环形进度、
折线图和柱状图、卡片、徽标），有浅色和深色两套主题。它们就是普通的 Zamak 代码，可以复制一份改成你想要的样子。
参见 `examples/ui/widgets.zm`。

### 调用 C

- `extern type Db: "sqlite3", close = sqlite3_close` 声明一个 C 指针类型，Zamak 保证正好关闭一次。
  C 函数以 `Option[Db]` 返回它，NULL 就是 `None`。
- `[native] headers` 按 C 库自己的头文件生成调用，`int`、`size_t` 参数不用再写 C 包装。
- `[native]` 还能编译 C++，在 macOS 上还能编译 Objective-C；`frameworks = ["Foundation"]` 链接 macOS 框架。
  原生源码并行编译。

### 构建与工具

- 重新构建更快了。`zam` 会记住找到的 C 编译器，在 Windows 上，什么都没改的构建从大约 100 毫秒降到大约 4 毫秒。
- 构建时留出一个 CPU 核，并以较低优先级运行 C 编译器，冷构建不会再让电脑卡顿。
- 新增两个平台的预编译版：Intel 芯片的 macOS 和 arm64 的 Linux，npm 和安装脚本都能装。
- 新增 `THIRD-PARTY-NOTICES.md`，列出内置的 Mbed TLS、libui-ng、SQLite 的许可证，每个安装包都带着它。
  如果你分发的程序用到了 HTTPS、TLS 或 `std/ui`，请把对应的声明一起附上。
- Zamak 语言手册新增附录“Zamak 是怎么工作的”：跟着一个程序从源码走到可执行文件，看它变成什么样的 C、
  内存在哪里释放，以及构建缓存怎么决定要重新编译什么。

### 修复

- Windows 上同时运行几个构建时，可能报 `corrupt cache entry`。原因是 MSVC 的多线程代码生成在机器繁忙时，
  同一个 C 文件偶尔会编出不同的指令。现在 `zam` 让每个文件只用一个代码生成线程。
- Linux 上，窗口里唯一的子控件（例如画布）只占自身的宽度，没有铺满窗口；用代码修改表格的行也会触发 `on_change`。
  现在两处都和 Windows、macOS 一致。
- 多个线程同时调用 `datetime.now()` 等本地时间函数时，时区偏移可能出错。
- 加了参数的 `CC` 包装脚本（例如 `exec gcc -fsanitize=thread "$@"`）会和普通的 `gcc` 共用缓存的目标文件。
  现在 `CC` 为空表示使用默认编译器，和 `make` 一样。
- 方法的类型参数和其他类型的类型参数同名（例如都叫 `T`）时，`obj.method(x)?` 会让编译器崩溃。
- 按引用传递的参数里写 `catch`，例如 `int(env.var("PORT") catch e: "8080")`，以前会被拒绝。
- 报错更清楚：把列表赋给字符串变量时，报的是类型不同，而不是借用错误；`f` 什么都不返回时写 `let x = f()`，
  会直接说明这一点。

## 0.1.1（中文）

发布日期：2026-10-19。

一个打包层面的修复，与 0.1.0 同一天发布。语言本身没有任何变化，0.1.0 的程序和缓存都能照常沿用。

- **`zam upgrade` 现在打印正确的包名。** 0.1.0 的二进制会提示你运行 `npm install -g zamak`，
  而那不是实际发布的包。现在输出 `npm install -g @veyralux/zamak`。
- **VS Code 扩展里的包名也一并改正**，一键安装和「用 npm 安装」的提示现在指向真实存在的包。
- **基准新增两种体积口径。** `bench/compare` 以前强制 `CC=gcc`，报出来的是 gcc 口径的体积；
  而 Windows 上发布版用的是 MSVC，静态链接 C 运行库，产物大得多。`run.py` 现在两种都能测
  （`zamak-msvc`），也能测压到极限的 Rust（`rust-min`）作对照。
- **README 的体积说法改成按平台分别说明**，不再只给一个数字。

## 0.1.0（中文）

发布日期：2026-10-19。

第一个正式版本：一门语法接近 Python 的静态类型语言，编译成独立的原生可执行文件；同时包含
`zam` 工具链、VS Code 扩展，以及给 AI 编程助手用的 MCP 服务器。

### 语言

- **靠缩进的代码块**，与 Python 相同。以 `:` 结尾的行开启一个块，空块写 `pass`；条件用
  `and`、`or`、`not` 组合，优先级与 Python 一致。
- **静态类型**加类型推断：`i64`、`f64`、`bool`、`string`（UTF-8）、结构体、枚举、固定数组
  `[i64; N]`、列表 `[T]`、字典 `{K: V}`、元组，以及 `fn(...) -> T`。
- **所有权与借用检查，不需要垃圾回收。** 赋值即移动；`&` 与 `&mut` 只在一次调用内借用；
  闭包不会比它用到的变量活得更久。
- **错误是值。** 函数返回 `T!E`；`fail` 返回错误，`?` 往上传递，`catch e:` 就地处理。
- **泛型**：函数写 `fn largest[T](items: &[T]) -> T`，结构体与枚举写 `struct Stack[T]`；
  每个调用点生成一份按类型特化的版本。内置 `Option[T]`。
- **结构体与枚举带方法**，`match` 支持带载荷的分支与字面量分支，结构体字面量要求写全字段名。
- **闭包与函数值**：类型 `fn(i64) -> bool`，匿名函数 `fn(x): x * 2`，也能在函数体里声明 `fn`；
  列表与字典可以存放函数。
- **UTF-8 字符串**与插值：`"{name}"`、`"{a + b}"`、`"{xs[0]}"`，字面大括号写 `"{{"`；
  字面量上可以直接用 Python 的字符串方法，例如 `"a,b".split(",")`。
- **插值里的格式说明符**（本版新增）：`f"{ratio:.3}"`、`f"{n:>5}"`、`f"{name:*^7}"`、
  `f"{count:03}"`、`f"{x:08.2}"`，写法是 `[[填充]对齐][宽度][.小数位]`。小数位只对 `f64`
  有意义，给 `i64` 写精度会直接报错，而不是悄悄四舍五入。
- **函数里定义的 fn 支持递归**（本版新增）：函数体里声明的 `fn` 可以调用自己。互相调用
  （前向引用）仍然不支持，这类函数写在顶层。
- **泛型类型的方法可以带自己的类型参数**（本版新增），例如 `fn map[U](&self, f: fn(T) -> U) -> U`；
  `U` 在每次调用处按实参推断。
- **测试与断言**：`assert cond, "message"`；`test_...` 函数由 `zam test` 每个各起一个进程运行。
- **模块与包**：每个文件是一个模块，`pub` 对外导出；`zam.toml` 声明依赖，`zam.lock` 把 Git
  依赖锁定到具体提交，构建可复现、可离线。
- **调用 C 库**：写 `extern fn sqrt(x: f64) -> f64`，在 `zam.toml` 的 `[native]` 里列出 C 源码、
  include 目录与库。
- **标准库**：`std/string`、`std/io`、`std/fmt`、`std/fs`、`std/path`、`std/env`、
  `std/math`、`std/time`、`std/random`；Python 的 `len`、`str`、`int`、`float`、`abs`、`min`、
  `max`、`sum`、`sorted` 无需导入。

### `zam` 命令

- 编辑循环用 `check`、`build`、`run`、`test`、`fmt`；`new` 创建项目；`add`、`remove`、`update`、
  `search`、`publish` 管理包；此外还有 `doc`、`lsp`、`mcp`、`upgrade`、`demo`。
- **每个模块按内容缓存**：同样的代码构建两次只编译一遍；改动一个模块只重建它和依赖它的模块。
  5000 行的项目 `zam check` 不到 0.1 秒。
- **`zam doc` 是本版新增。** 它把 `pub` 声明和它们上方的 `#` 注释变成一张自包含的 HTML API
  页面（不需要 JavaScript，跟随系统深浅色），字段、枚举变体与方法列在各自类型下面。
  `zam doc . site` 可以指定输出目录。
- **中英双语提示。** `--help` 用两种语言列出每个子命令的说明，语言跟随 `ZAMAK_LANG`；报错、
  修改建议，以及编译后程序在运行时的 panic（下标越界、除以零、键不存在等）都有中文。
- **错误码更稳定。** 单文件模式下报错不再带多余的 `main:` 前缀；真实的错误例子不会再落到兜底的
  ZM999；错误码与对应的改法写在 Book 的附录里，也通过 MCP 提供给 AI 助手。

### 编辑器支持

- 语法高亮、注释切换、括号配对，以及在以 `:` 结尾的行之后自动缩进。
- 由 `zam lsp` 提供边打边检查的诊断（包含未保存的改动）、补全、悬停、跳转定义与格式化文档。
- **参数提示**：输入调用时显示参数列表并高亮当前这个实参，覆盖本文件函数、跨模块函数、结构体方法
  与标准库。
- **重命名**与**查找引用**：作用于当前文件里的名字；注释和字符串里同名的文字不会被动到。
- **`x.` 之后的补全**改问检查器要真实类型，而不是按名字猜；文件还没通过检查时退回原来的行为。
- **一键安装**：`zam` 不在 `PATH` 上时，扩展提供「用 npm 安装」和「重新查找编译器」；状态栏显示
  正在使用的 `zam` 版本。

### AI 编程助手

- `zam mcp` 在 stdio 上讲 Model Context Protocol，提供**八个工具**：`check`、`build`、`run`、
  `test`、`format`、`explain`、`docs`，以及本版新增的 `hover`（按文件、行、列返回那里的类型或签名）。
- **MCP 资源**（本版新增）：`zamak://reference` 提供一页纸的语言参考，`zamak://diagnostics` 提供
  错误码表，助手不用起子进程就能读到语言说明。
- `.agents/skills/zamak/SKILL.md` 与 `zam new --template ai` 为直接读仓库的助手准备了同样的内容。
- 离线基准（`bench/ai`）与真实模型的实测结果：三个模型一共 24 个 Zamak 程序全部解出，其中 19 个
  一次通过。最常见的首次失败是漏写 `use std/string`，而报错会直接指出要加的那一行。

### 性能

Zamak 经 C11 编译成独立的原生可执行文件：不需要随程序分发运行时，也没有垃圾回收带来的停顿。
在对比套件里，计算密集的程序与 C 处于同一区间，而整数溢出与数组下标默认有检查；产出的可执行
文件小，内存占用也小。每个模块按内容缓存，所以构建一直很快。测速图、当前数字与测量方法见
[README.zh-CN.md](README.zh-CN.md)、[docs/PERFORMANCE.md](docs/PERFORMANCE.md) 与
[bench/compare](bench/compare/README.md)。

### 说明

- 线程、网络、REPL 与调试器尚未实现。遇到不支持的写法时，`zam check` 会报错，而不是把它编译成
  行为不正确的代码。
- 本版删掉了 `ico/` 里一个没人引用、1.7 MB 的旧版图标。

## 0.1.0-rc.1（中文）

第一个公开预览版：上面描述的语言本身、包索引、C 互操作层，以及 `zam lsp` 服务器。上面标注
「本版新增」的内容都是在这个预览版之后加入的。
