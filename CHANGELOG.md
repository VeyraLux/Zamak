# Changelog

All notable changes to Zamak. This project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
and this file is the source for the text of each GitHub release.

<!-- 中文版在下半部分。Chinese version: scroll down or search for "## 0.1.1（中文）". -->

## Unreleased

- **`bytes` for raw data.** `b"GIF\x89"`, `s.encode()`, and `b.decode()` work as in Python, and
  indexing gives an integer from 0 to 255. `bytes` works like a list (`len`, slices, `push`,
  `for b in data`), and writing a value outside 0 to 255 stops the program. `std/bytes` adds
  `from_hex`, `hex`, and `zeros`; `std/fs` adds `read_bytes`, `write_bytes`, and `append_bytes`.
- **String escapes `\xNN` and `\0`**, as in Python.
- **`?` and `catch` work on method calls** such as `text.decode()?` and `line.parse_i64()?`.
- **Clearer error** when you assign a list to a string variable: it now says the types differ
  instead of "cannot move borrowed value".
- **C handles.** `extern type Db: "sqlite3", close = sqlite3_close` declares a C pointer type that
  Zamak closes exactly once when it goes out of scope. C functions return it as `Option[Db]` (NULL
  is `None`) or through an output parameter `&mut Option[Db]`, and take it as `&Db`.
- **`[native] headers`** lets Zamak call a C library through its own header, so `int` and
  `size_t` parameters work without a C wrapper. `extern fn` also takes `&bytes` and `&mut bytes`.
- **`?` and `catch` after a method chain**, such as `buf[:n].decode() catch e: "?"`.
- **Default and keyword arguments**, as in Python: `fn connect(host: &string, port: i64 = 80)`
  and `connect("a", timeout=1.5)`. Defaults are constants or `Option.None`, so calls never share
  a mutable default. Methods take keyword arguments, and `Point(x=1, y=2)` builds a struct.
- **Sets.** `{1, 2, 3}` has the type `{i64}`, with `add`, `remove`, `in`, `union`,
  `intersection`, `difference`, `issubset`, and `sorted(s)`.
- **New standard library modules**, named as in Python: `std/process` (run programs and read their
  output), `std/json` (`loads`, `dumps`), `std/re` (regular expressions on a linear-time engine,
  so no pattern can hang a program; no backreferences or lookaround), `std/hashlib` (SHA-256,
  SHA-1, MD5, HMAC-SHA256), and `std/base64`.
- **Raw and triple-quoted strings**: `r"\d+"`, `"""say "hi" now"""`, and `r"""{"a": 1}"""`, plus the
  `\uXXXX` and `\UXXXXXXXX` escapes.
- Parts of the standard library are now written in Zamak and compiled with your program.
- **Interfaces.** `interface Shape:` lists methods, and any type with those methods satisfies it,
  with no `impl` to write. Use it as a type to keep values of different types in one list
  (`[Shape]`, calls dispatch to each value), or as a bound on a type parameter
  (`fn total[T: Shape]`, compiled once per type).
- **Networking.** `std/http` sends requests in the shape of Python's `requests` (`http.get`,
  `http.post`, `r.status`, `r.text()`, `r.json()`) and serves them with `http.serve`. `std/net` has
  TCP connections and listeners that close when they go out of scope.
- **HTTPS.** `https://` URLs use TLS and check the server's certificate against the system's roots
  (or `SSL_CERT_FILE`). `http.serve` takes `cert_file` and `key_file` to serve HTTPS, and `std/net` has
  `conn.start_tls(host)` and `listener.accept_tls(cert_file, key_file)`. TLS comes from Mbed TLS 3.6
  (Apache-2.0), which `zam` carries and compiles into the cache the first time a program needs it.
- `[native]` C sources compile in parallel, and a header's `__FILE__` no longer puts the build
  machine's paths into the program.
- **Threads.** `std/thread` runs a function on a new thread with `thread.spawn(work, args...)`, in
  the shape of Python's `executor.submit`. Arguments move to the new thread, and `join()` returns the
  result (or the function's error). Threads share nothing that can change; `thread.Queue[T]` carries
  values between them, like Python's `queue.Queue`. `println` writes whole lines from any thread, and
  TLS works from several threads at once.
- A generic function whose type parameter appears only in its return type infers it from the
  annotation: `let q: thread.Queue[i64] = thread.queue()`.
- A list literal of strings, floats, booleans, structs, or lists no longer needs an annotation:
  `let names = ["Ada", "Grace"]` is a `[string]`.
- `[native]` compiles C++ (`.cpp`, `.cc`, `.cxx`) and, on macOS, Objective-C (`.m`) files as well as C,
  and links the C++ runtime when it needs to. `frameworks = ["Foundation"]` links macOS frameworks.
- A struct, enum, or tuple that holds only numbers, booleans, function values, and other such types
  is copied on assignment, like a number, so the old variable stays usable: `let b = a` with
  `a: Point`.
- **Windows and controls.** `std/ui` builds desktop programs with each system's native controls
  (Win32, AppKit, GTK 3): windows, labels, buttons, entries, text areas, check boxes, drop-down lists,
  option buttons, sliders, spin boxes, progress bars, columns, rows, forms, grids, groups, tabs, and
  file and message dialogs. The program's state is a struct passed to `app.run`, and handlers such as
  `app.on_click(button, clicked)` receive it as `&mut`. It is built on libui-ng (MIT), which `zam`
  carries and compiles into the cache the first time a program needs it.
- `kept fn(...)` marks a parameter whose function is stored for later, so it only takes functions
  that don't use local variables.
- A generic method of a struct that isn't generic, such as `fn add[C: Control](&self, child: &C)`,
  now infers its type parameters.
- Reading a string element or field gives a copy: `let arg = args[i]` and `let why = c.error` work,
  and the list or struct keeps its value.
- String methods `rfind`, `removeprefix`, `removesuffix`, `capitalize`, `title`, `center`, `zfill`,
  `isupper`, and `islower`, and `s.find(sub, start)`.
- Interpolation takes any expression, including string literals and slices: `f"{d["key"]} {s[:5]}"`.
- `std/http` follows redirects by default (`follow=false` turns it off) and has `http.urljoin`.
- `examples/apps/linkcheck` checks the links of a web page with several threads.
- **Dates and times.** `std/datetime` follows Python's `datetime`: `Date` and `DateTime`, `today()`,
  `now()`, parsing and `isoformat()`, `strftime`, `add_days`, `days_since`, and time zone offsets.
- More everyday shapes compile: indexing, slicing, and reading a field of a call result
  (`line.split(",")[0]`, `user().name`), `?` after an element's method (`args[1].parse_i64()?`),
  methods on struct literals, nested `catch e:`, `fail x` in several `catch` blocks, and a borrowed
  plain value where a value is needed.
- `examples/apps/todo` is a command-line task list that stores its tasks as JSON.
- **JSON for any type.** `json.encode(value)` and `json.decode(text)` convert structs, lists, maps,
  `Option`, and enums to and from JSON with no code to write; errors give the path, such as
  `[0].tags[1]: the JSON value is not a string`. `Json` also has `as_list`, `lookup`, and `is_str`.
- Moving a list, map, or struct field out of a struct you own uses up the struct: `let tasks = store.tasks`.
- `[]` and `{}` work as a `catch` fallback value.
- **`std/sqlite`**: SQLite 3.54.1 is built into `zam`. `db.execute(sql, params)`, `db.execute_named(sql, &value)`,
  and `let rows: [T] = db.query(sql, params)?` store structs and read rows back as structs. Integers keep every digit,
  `BOOLEAN` columns read back as `bool`, and anything that doesn't fit is an error.
- **Breaking:** `Json.Number(f64)` is now two variants, `Json.Int(i64)` and `Json.Float(f64)`, as in Python.
  `1` and `1.0` keep their types through `loads` and `dumps`, and an integer that doesn't fit in `i64` is an error.
- Lists have `append`, the same as `push`.
- **Breaking:** list methods now mean what they mean in Python. `xs.remove(x)` removes the first element equal to `x`,
  and `xs.pop(i)` removes and returns the element at `i`; `xs.remove(i)` used to do what `pop(i)` does now.
- Module constants can be lists and maps: `const NAMES = ["Food", "Home"]` works with `for`, `[i]`, methods, and `&` arguments.
- A multi-line `catch` block can end with a value: the lines before it run first, and the value is the result when the call fails.
- **`http.serve` handles requests on several threads at once** (`workers=n`, by default twice the number of processors and at least 8),
  and HTTPS handshakes happen on those threads too. `Request` has `args()` and `arg(name)` for query parameters,
  `http.quote`, `http.unquote`, and `http.parse_query` match Python's `urllib.parse`, and `http.respond_json` takes any value.
- `match` on the result of a call gives the arm its own copy of the data: you can change it or move it, as in
  `match jobs.get(): Some(job): done.append(job)`. `match` on a variable still borrows.
- `examples/apps/notes` is a REST service for notes, backed by SQLite.
- **`ui.table(columns)`** shows a read-only table: `set_rows(rows)`, `selected()`, `set_selected(i)`, and `app.on_change`
  when the user selects another row.
- **Draw your own controls.** `ui.canvas()` with `app.on_draw`, `app.on_mouse`, and `app.on_key`; the `Painter` draws paths,
  rounded rectangles, circles, linear and radial gradients, and text, and can translate, rotate, scale, and clip.
- **`std/widgets`**: default controls drawn on a canvas, which you can copy and change: switches, buttons, sliders, check boxes,
  progress bars and rings, line and bar charts, cards, and badges, with light and dark themes. See `examples/ui/widgets.zm`.
- `examples/apps/ledger` is a desktop ledger: a form, a table, monthly totals, and CSV export, with amounts kept in whole cents.
- `s.ljust(width, fill)`, `s.rjust(width, fill)`, and `s.center(width, fill)` take a fill character, as in Python.
- `let x = f()` where `f` returns nothing now says so, instead of reporting a borrow error.
- Fixed on Linux: a control set as a window's child, such as a canvas, filled only its natural width; it now fills the window,
  as on Windows and macOS. Changing a table's rows or selection in code no longer calls `on_change` on Linux, as on Windows.
- Fixed: `catch` inside a call that takes a borrowed argument, such as `int(env.var("PORT") catch e: "8080")`, was rejected.
- Fixed: `datetime.now()` and other local-time functions could return a wrong offset when several threads called them at once.
- Fixed: a `CC` wrapper script that adds flags (such as `exec gcc -fsanitize=thread "$@"`) shared cached objects with plain `gcc`.
  An empty `CC` now means the default compiler, as in `make`.
- `let x: T = obj.method(...)` infers a method's type parameter from the annotation, as it already did for functions.
- Fixed: `obj.method(x)?` crashed the compiler when the method's type parameter had the same name as another type's,
  such as `T` in `Option[T]`.
- A parenthesized value can take a method call: `(a + b).upper()`.

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

- **`bytes` 类型，用来存原始数据。** `b"GIF\x89"`、`s.encode()`、`b.decode()` 与 Python 相同，下标读出
  0 到 255 的整数。`bytes` 的用法和列表一样（`len`、切片、`push`、`for b in data`），写入 0 到 255
  以外的数会让程序停下。`std/bytes` 新增 `from_hex`、`hex`、`zeros`；`std/fs` 新增 `read_bytes`、
  `write_bytes`、`append_bytes`。
- **字符串转义新增 `\xNN` 和 `\0`**，与 Python 相同。
- **方法调用也能接 `?` 和 `catch`**，例如 `text.decode()?`、`line.parse_i64()?`。
- **报错更清楚**：把列表赋给字符串变量时，现在说类型不同，而不是“不能移动被借用的值”。
- **C 句柄。** `extern type Db: "sqlite3", close = sqlite3_close` 声明一个 C 指针类型，离开作用域时
  Zamak 正好调用一次 close。C 函数以 `Option[Db]` 返回它（NULL 是 `None`），或通过输出参数
  `&mut Option[Db]` 交出它；参数写 `&Db`。
- **`[native] headers`**：按 C 库自己的头文件调用，`int`、`size_t` 参数不用再写 C 包装。
  `extern fn` 也能接收 `&bytes` 和 `&mut bytes`。
- **方法链之后也能接 `?` 和 `catch`**，例如 `buf[:n].decode() catch e: "?"`。
- **默认参数与关键字实参**，写法同 Python：`fn connect(host: &string, port: i64 = 80)`、
  `connect("a", timeout=1.5)`。默认值只能是常量或 `Option.None`，不会出现多次调用共用一个可变默认值的问题。
  方法也能用关键字实参，结构体可以写 `Point(x=1, y=2)`。
- **集合。** `{1, 2, 3}` 的类型是 `{i64}`，支持 `add`、`remove`、`in`、`union`、`intersection`、
  `difference`、`issubset` 和 `sorted(s)`。
- **新的标准库模块**，名字同 Python：`std/process`（运行程序、读取输出）、`std/json`（`loads`、`dumps`）、
  `std/re`（正则表达式，线性时间引擎，任何表达式都不会让程序卡死；不支持反向引用与环视）、
  `std/hashlib`（SHA-256、SHA-1、MD5、HMAC-SHA256）和 `std/base64`。
- **原始字符串与三引号字符串**：`r"\d+"`、`"""say "hi" now"""`、`r"""{"a": 1}"""`，以及 `\uXXXX`、
  `\UXXXXXXXX` 转义。
- 标准库的一部分改用 Zamak 编写，和你的程序一起编译。
- **接口。** `interface Shape:` 列出方法，有这些方法的类型自动满足它，不用写 `impl`。接口可以当类型，
  把不同类型的值放进同一个列表（`[Shape]`，调用时按各自的类型分派），也可以约束类型参数
  （`fn total[T: Shape]`，每种类型各生成一份代码）。
- **网络。** `std/http` 发请求的写法参考 Python 的 `requests`（`http.get`、`http.post`、`r.status`、
  `r.text()`、`r.json()`），`http.serve` 处理请求。`std/net` 提供 TCP 连接与监听，离开作用域时自动关闭。
- **HTTPS。** `https://` 地址走 TLS，用系统的根证书（或 `SSL_CERT_FILE`）校验服务器证书。`http.serve` 传
  `cert_file` 与 `key_file` 即提供 HTTPS；`std/net` 有 `conn.start_tls(host)` 与
  `listener.accept_tls(cert_file, key_file)`。TLS 来自 Mbed TLS 3.6（Apache-2.0），`zam` 自带源码，
  程序第一次用到时编译进缓存。
- `[native]` 的 C 源码并行编译；头文件里的 `__FILE__` 不再把构建机器的路径带进程序。
- **线程。** `std/thread` 用 `thread.spawn(work, 参数...)` 在新线程里运行函数，写法参考 Python 的
  `executor.submit`。参数移交给新线程，`join()` 取回返回值（或函数的错误）。线程之间不共享可以修改的数据，
  用 `thread.Queue[T]` 传递值，形状参考 Python 的 `queue.Queue`。任何线程的 `println` 都按整行输出，
  TLS 可以在多个线程里同时用。
- 类型参数只出现在返回类型里的泛型函数，按标注推断：`let q: thread.Queue[i64] = thread.queue()`。
- 字符串、浮点数、布尔值、结构体或列表组成的列表字面量不再需要标注：`let names = ["Ada", "Grace"]` 是 `[string]`。
- `[native]` 除了 C，还编译 C++（`.cpp`、`.cc`、`.cxx`）与 macOS 上的 Objective-C（`.m`），需要时链接 C++ 运行库；
  `frameworks = ["Foundation"]` 链接 macOS 的 framework。
- 只含数字、布尔值、函数值与这类类型的结构体、枚举和元组，赋值时像数字一样复制，原来的变量还能用：
  `a: Point` 时 `let b = a` 之后 `a` 照样可用。
- **窗口程序。** `std/ui` 用各系统的原生控件（Win32、AppKit、GTK 3）写桌面程序：窗口、标签、按钮、输入框、
  多行文本框、复选框、下拉框、单选按钮、滑块、数字框、进度条，纵向、横向、表单、网格、分组、选项卡布局，
  以及文件与消息对话框。程序状态是交给 `app.run` 的结构体，`app.on_click(button, clicked)` 这样的回调
  以 `&mut` 收到它。底层是 libui-ng（MIT），`zam` 自带源码，程序第一次用到时编译进缓存。
- `kept fn(...)` 标出函数会被保存下来的参数，它只收不用局部变量的函数。
- 非泛型结构体的泛型方法（如 `fn add[C: Control](&self, child: &C)`）现在能推断类型参数。
- 读取字符串元素或字段得到副本：`let arg = args[i]`、`let why = c.error` 都能写，列表与结构体不变。
- 字符串方法 `rfind`、`removeprefix`、`removesuffix`、`capitalize`、`title`、`center`、`zfill`、`isupper`、
  `islower`，以及 `s.find(sub, start)`。
- 插值里可以写任何表达式，包括字符串字面量与切片：`f"{d["key"]} {s[:5]}"`。
- `std/http` 默认跟随重定向（`follow=false` 关闭），新增 `http.urljoin`。
- `examples/apps/linkcheck`：用多个线程检查网页上的链接。
- **日期与时间。** `std/datetime` 参考 Python 的 `datetime`：`Date` 与 `DateTime`、`today()`、`now()`、解析与
  `isoformat()`、`strftime`、`add_days`、`days_since`，以及时区偏移。
- 更多日常写法可以编译：调用结果上取下标、切片和字段（`line.split(",")[0]`、`user().name`），元素方法后接 `?`
  （`args[1].parse_i64()?`），结构体字面量上调方法，嵌套的 `catch e:`，几个 `catch` 块里 `fail` 同一个值，
  以及在要值的地方使用借来的纯数据值。
- `examples/apps/todo`：命令行的待办事项，任务存成 JSON。
- **任何类型都能转成 JSON。** `json.encode(value)` 与 `json.decode(text)` 在 JSON 和结构体、列表、字典、`Option`、
  枚举之间直接转换，不用写转换代码；出错时带着路径，例如 `[0].tags[1]: the JSON value is not a string`。
  `Json` 还新增了 `as_list`、`lookup`、`is_str`。
- 从自己拥有的结构体里移出列表、字典或结构体字段，会用掉整个结构体：`let tasks = store.tasks`。
- `[]` 和 `{}` 可以当 `catch` 的回退值。
- **`std/sqlite`**：SQLite 3.54.1 编进了 `zam`。`db.execute(sql, params)`、`db.execute_named(sql, &value)` 与
  `let rows: [T] = db.query(sql, params)?` 直接存结构体、把结果行读成结构体。整数保留每一位，`BOOLEAN` 列读回 `bool`，
  放不进去的直接报错。
- **不兼容：** `Json.Number(f64)` 拆成 `Json.Int(i64)` 和 `Json.Float(f64)` 两个变体，和 Python 一样。
  `1` 与 `1.0` 经过 `loads`、`dumps` 类型不变；超出 `i64` 的整数报错。
- 列表有了 `append`，与 `push` 相同。
- **不兼容：** 列表方法和 Python 同义：`xs.remove(x)` 删掉第一个等于 `x` 的元素，`xs.pop(i)` 取走并返回第 `i` 个元素；
  以前的 `xs.remove(i)` 就是现在的 `pop(i)`。
- 模块级常量可以是列表和字典：`const NAMES = ["Food", "Home"]` 能用在 `for`、`[i]`、方法调用和 `&` 实参里。
- 多行的 `catch` 块可以以一个值结尾：前面几行先执行，这个值就是调用失败时的结果。
- **`http.serve` 用几个线程同时处理请求**（`workers=n`，默认是处理器个数的两倍，至少 8 个），HTTPS 握手也在这些线程里做。
  `Request` 有 `args()` 与 `arg(name)` 读查询参数，`http.quote`、`http.unquote`、`http.parse_query` 对应 Python 的
  `urllib.parse`，`http.respond_json` 可以直接传任何值。
- 对调用结果做 `match` 时，分支拿到的数据归自己：可以修改、移走，如 `match jobs.get(): Some(job): done.append(job)`。
  对变量做 `match` 仍然是借用。
- `examples/apps/notes`：存笔记的 REST 服务，数据在 SQLite 里。
- **`ui.table(columns)`**：只读表格，`set_rows(rows)`、`selected()`、`set_selected(i)`；用户选中别的行时触发 `app.on_change`。
- **自己画控件。** `ui.canvas()` 配合 `app.on_draw`、`app.on_mouse`、`app.on_key`；`Painter` 能画路径、圆角矩形、圆、
  线性与径向渐变和文字，还能平移、旋转、缩放、裁剪。
- **`std/widgets`**：画在画布上的默认组件，可以复制了改：开关、按钮、滑块、复选框、进度条与进度环、折线图与柱状图、
  卡片、徽章，配浅色与深色主题。见 `examples/ui/widgets.zm`。
- `examples/apps/ledger`：桌面记账本，表单、表格、按月汇总、导出 CSV，金额按整数分保存。
- `s.ljust(width, fill)`、`s.rjust(width, fill)`、`s.center(width, fill)` 可以指定填充字符，同 Python。
- `let x = f()` 而 `f` 什么也不返回时，报错直接说明这一点，不再报借用错误。
- 修复（Linux）：直接放进窗口的控件（如画布）只有自然宽度，现在和 Windows、macOS 一样铺满窗口。程序自己改表格的数据或选中行
  时，Linux 上也不再调用 `on_change`（同 Windows）。
- 修复：`catch` 放在借用参数的调用里（如 `int(env.var("PORT") catch e: "8080")`）会被拒绝。
- 修复：几个线程同时调用 `datetime.now()` 等本地时间函数时，可能得到错误的时区偏移。
- 修复：加了参数的 `CC` 包装脚本（如 `exec gcc -fsanitize=thread "$@"`）与普通 `gcc` 共用缓存的目标文件。
  `CC` 为空时和没设一样，用默认编译器（同 `make`）。
- `let x: T = obj.method(...)` 可以按标注推断方法的类型参数，和函数一样。
- 修复：方法的类型参数与别的类型的同名（如 `Option[T]` 的 `T`）时，`obj.method(x)?` 让编译器崩溃。
- 括号里的值也能接方法调用：`(a + b).upper()`。

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
