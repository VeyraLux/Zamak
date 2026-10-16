# Zamak

[English](README.md) | 中文

面向快速构建的静态类型原生语言原型。

[![CI](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml/badge.svg)](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml)

[教程](TUTORIAL.md) · [预览版安装](PREVIEW.md) · [参与开发](CONTRIBUTING.md) · [AI 工作流](docs/AI_WORKFLOW.md) · [案例验证](docs/VALIDATION.md) · [工程指标](docs/ENGINEERING.md)

<img width="96" height="96" alt="Zamak" src="ico/Zamak_256x256.png" />

Zamak 是为频繁构建和多 worktree 协作设计的语言原型，改完代码，人和 AI 都能很快拿到检查结果。`zam` 先生成 C，再调用 MSVC、GCC 或 Clang 编译成机器码；类型与所有权由 Zamak 自己检查，生成的程序可以独立运行，不需要安装 Zamak 或 Rust。

## 特点

- **构建与缓存**：`zam check` 只做类型与所有权检查，不调用 C 编译器。同一份源码可以在不同目录和工作树之间共享缓存；改了实现只重编受影响的原生模块，改了被调用的函数接口或结构体布局，调用方随之失效。
- **类型与所有权**：静态类型检查覆盖变量、函数、结构体、列表和字典。字符串、结构体、列表和字典赋值时移动；借用只在一次调用内有效，不需要写生命周期标注。整数溢出、除零和越界在运行时检查。
- **诊断**：每条错误带稳定的错误码、位置和修改建议，`--json` 把它们写成一行 JSON 给工具读取。
- **原生执行**：生成的程序独立运行，无需安装 Zamak 或 Rust。

在 3 个模型、8 个小型命令行任务的对比实验里，模型只读了一页语言参考：24 个 Zamak 程序（缩进语法）全部第一次就通过全部测试。同样的任务用 Rust 和 Go 写，24 个也都是第一次通过，这些任务太简单，分不出语言之间的高下。方法、任务和数据见 [bench/ai/live](bench/ai/live/README.md)。

当前版本适合学习、语言实验、算法练习和命令行小工具。标准库与工具生态还不完整，不适合生产使用。

### 生态现状

| 组件 | 当前能力 |
| --- | --- |
| 命令行 | `check`、`build`、`run`、`test`、`fmt`、`new`、`add`、`update`、`demo`；检查与构建支持 JSON 诊断 |
| 包管理 | `zam.toml` 写 git 或本地路径依赖，`zam.lock` 锁定提交，依赖包里的模块按 `use 包名/模块` 导入 |
| 数值 | `i64`（溢出与除零在运行时检查）、`f64`（IEEE 754）、`bool`；`f64(n)`、`i64(x)` 显式转换 |
| 集合 | 定长 `[i64; N]`、可增长列表 `[T]`、字典 `{K: V}`；元素与值可以是标量、字符串或结构体，键为 `i64` 或 `string` |
| 标准库 | 文件读写、标准输入、命令行参数、整数与浮点数解析；`std/math` 的平方根、幂、三角、取整等函数；`fmt.fixed` 按位数格式化；字符串的长度、比较、查找、替换、大小写转换、裁剪、切片、切分与连接；`println` |
| 枚举 | `enum` 带数据的变体、`match` 语句与表达式，检查分支是否覆盖所有变体 |
| 错误处理 | `T!string`、`fail`、`?` 传播、`catch` 处理 |
| 编辑器 | VS Code 语法高亮、括号与引号配对 |

尚未支持网络 I/O、目录遍历、线程及 FFI，也还没有中心化的包仓库、语言服务器、REPL 和文档生成工具。安装方式见 [预览版说明](PREVIEW.md)。

## 快速运行

第一次使用可以先读 [入门教程](TUTORIAL.md)。Windows 预览包和各平台的工具链要求见 [安装说明](PREVIEW.md)。

从源码安装需要 Rust stable 和一个 C 编译器。在仓库根目录执行：

```powershell
cargo install --path zam --locked
```

Cargo 的安装目录（通常是用户目录下的 `.cargo/bin`）要在 PATH 里。修改编译器源码后，重新执行这条命令更新。然后创建并运行一个项目：

```powershell
zam --version
zam new hello
zam check hello
zam build hello
zam run hello
zam fmt hello
zam test hello
```

`hello` 是一个还不存在的目录。进入该目录后可以直接执行 `zam run`。新项目的输出是：

```text
hello from zamak
```

多模块、中文输出和所有权的示例见 [教程](TUTORIAL.md)。

### 命令行行为

- `build` 在 stderr 输出 `OUTPUT <path>`：Windows 上是 `build/<name>-<hash>.exe`，Unix 上是同名的无后缀可执行文件。产物可以直接启动，也可以用 `zam run <path>` 执行。
- 产物不需要安装 Zamak 或 Rust。Windows 默认的 MSVC 后端静态链接 C 运行库；其他 C 工具链的运行库依赖取决于它们的配置。
- 构建进度（`CACHE`/`OBJECT`/`LINK`/`OUTPUT`）和错误写 stderr，stdout 只留程序输出。命令失败时返回非零退出码。
- 没有路径参数时使用当前目录。源码后缀统一为 `.zm`，也可以直接检查或构建单个源码文件。
- 词法和语法错误带源码文件、行号与列号。列号从 1 开始，按 Unicode 字符计数，制表符算一个字符，EOF 使用末尾位置。词法错误指向 token 起点，语法错误指向解析器发现错误的位置。
- 语义错误带行号、列号与所属函数，位置取当前语句的起点（`missing string return value` 取函数声明处）。一个模块对应一个文件，函数前面的模块名就能定位文件。

### 诊断

不带 `--json` 时，有位置的诊断写成错误码、位置、源码行、下划线和修改建议，中文与制表符按显示宽度对齐：

```text
error[ZM301]: use after move: name
 --> src/main.zm:8:5
  |
8 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

`zam check --json` 与 `zam build --json` 把结果写成一行 JSON，给编辑器、CI 和 AI 工具读取：`{"command":"check","ok":false,"diagnostics":[{"file":"…","line":3,"column":9,"function":"main:main","code":"ZM100","category":"syntax","message":"…","suggestion":"…"}]}`。

- `code` 和 `category` 是稳定的机器字段，`message` 是给人看的文本；有明确修复方法时，`suggestion` 给出下一步。
- 无法定位时，`function` 和位置字段为 `null`。
- 词法、语法与模块解析错误只报第一条；类型与所有权检查逐个函数进行，每个出错的函数各报一条，`diagnostics` 按位置排序。
- 失败时 stderr 仍有人类可读的诊断，退出码与不带 `--json` 时相同。其他子命令不接受 `--json`。

### 新建项目

`zam new <directory>` 生成一个最小可运行的骨架：`zam.toml`（`name`、`entry = "src/main.zm"`、`modules = "src"`）、打印一句问候的 `src/main.zm`，以及忽略 `build/` 的 `.gitignore`。创建的文件路径以 `NEW <path>` 写到 stderr，stdout 保持为空。骨架已经格式化，可以直接 `zam run`。目标目录已存在时报错退出，不覆盖任何文件；目录名要是合法标识符，因为它会写进 `zam.toml` 的 `name`。

面向 AI 协作可以用 `zam new <directory> --template ai`，额外生成 `AI.md` 与 `README.md`，写明 check --json → 最小修复 → fmt → check → build → run 的工作循环。

### 测试

`zam test [project|file] [过滤词]` 运行所有名字以 `test_` 开头的函数（任何模块里都行，签名是 `fn test_x():` 或 `fn test_x() -> !E:`，不带参数）。测试写在被测代码旁边，普通构建不受影响。每个测试单独起一个进程运行，某个测试越界、`assert` 失败或返回错误时只算它失败，其他测试照常跑；超过 60 秒算失败。结果写 stdout：每个测试一行 `test 名字 ... ok/FAILED`，失败的测试随后列出它的输出，最后一行是 `test result: ok. N passed; M failed`；有失败时退出码非零。给出过滤词时只运行名字里含这段文字的测试。入口模块里的测试显示为 `test_x`，其他模块显示为 `模块.test_x`。测试时不要求 `fn main()`，所以库也能直接测。实现上是同一套构建，入口模块生成的 C `main` 按第一个参数的名字分派到对应的测试函数；生成的 C 不同，目标文件的缓存键自然不同，不会和普通构建混用。示例：`zam test examples/testing`。

`assert 条件` 或 `assert 条件, 说明` 在条件为假时把文件、行号、条件原文和说明写到 stderr 并以退出码 1 结束，测试和普通程序里都能用。条件是 `a == b` 或 `a != b`、两边是变量、字段、元素或字面量时，还会显示两边的值：

```text
src/stats.zm:8: assertion failed: got == 5: two plus two
  left:  4
  right: 5
```

### 格式化

`zam fmt [project|file]` 就地整理源码：缩进统一成 4 个空格（Tab 先换成空格），括号里的续行按括号层数缩进；多行形式的 `if`/`for`/`while`/`match` 前后各留一个空行（块里第一条、前一行是注释、`elif`/`else` 之前、块结束处除外），连续空行压成一个，顶层声明之间留一个空行，单行形式（`if n < 1: return n`）不加空行；去掉行尾空白，结尾统一留一个换行，并保留原有的换行风格（CRLF 文件仍是 CRLF）。它只改空白、不重建 token，注释、字符串内容和行内空格都原样保留。改写过的文件路径以 `FORMAT <path>` 写到 stderr，已经规范的文件不重写也不报告。语法有错的代码会先被拒绝（与 `check` 同一套解析）。目前不调整行内空格，也不折行。

### C 工具链

`zam` 按这个顺序选择 C 编译器：

1. 设置了 `CC` 就用它。`CC` 要指向采用 GNU 命令行的编译器程序，不能带参数，也不能是 MSVC `cl`。
2. Windows 先通过 `vswhere` 查找 Visual Studio Build Tools（「使用 C++ 的桌面开发」及 Windows SDK）里的 x64 MSVC，找不到时依次改用 PATH 上的 `gcc`（如 MinGW-w64）、`clang`。
3. Linux 与 macOS 依次使用 `cc`、`clang`、`gcc`。

都找不到时报错并说明该安装什么。`check` 不需要 C 编译器。已在 Ubuntu 26.04 + gcc 15.2，以及 WSL2 Ubuntu 24.04 + gcc 13.3 / clang 18.1 上实机跑过构建、执行和全部集成测试；macOS 由 CI 验证。旧 `.zbc` 产物已不再支持，需要重新构建。

## 编辑器与 AI 工具

- **VS Code 语法扩展**：`editors/vscode/`。纯声明式：一份 TextMate 语法加语言配置，没有 `main`、没有运行时依赖、不联网。为 `.zm` 文件提供关键字、类型、字符串插值、字符字面量和注释高亮，以及括号、引号配对；冒号结尾的行回车后自动缩进，代码块按缩进折叠。同一份 `syntaxes/zamak.tmLanguage.json` 也能给 Sublime Text、TextMate 等支持 TextMate 语法的编辑器使用。本地安装、打包 `.vsix` 与上架市场的步骤见 `editors/vscode/README.md`。
- **AI 技能**：`.agents/skills/zamak/SKILL.md`，给遵循 `.agents/skills` 约定的 AI 编程代理（DSH、Claude Code 等）使用：命令与环境变量、语法速览、常见诊断、示例与测试的位置，以及修改编译器时的验收命令和 IR/缓存版本号规则。
- **CLI**：`zam` 本身，包括 `check`/`build`/`run`/`fmt`/`new`/`demo`、`--help`、`--version`；`check`/`build` 支持 `--json` 机器可读诊断；缓存目录可用 `ZAMAK_CACHE_DIR` 覆盖。还没有包管理器。

## 项目配置

```toml
[project]
name = "hello"
entry = "src/main.zm"
modules = "src"
```

配置包含 `[project]` 和可选的 `[dependencies]`（见下文“依赖包”）；`[project]` 只有上面三个字符串字段；路径相对配置文件，不能包含 `..`，入口要在模块目录内。`zam` 递归扫描模块目录里的 `.zm` 文件，一个文件对应一个模块，例如 `src/tools/greeting.zm` 的模块名是 `tools/greeting`。暂不支持源码符号链接。

```zam
use tools/greeting
use std/io

fn main():
    greeting.hello()
    io.println("你好")
```

被导入模块里的函数要声明为 `pub fn`。导入用 `/`，调用用模块路径的末段名称。模块别名有歧义、模块缺失、调用私有函数、重复声明和模块依赖成环都会报错。

### 依赖包

别人写好的 Zamak 库可以直接拿来用。依赖写在 `[dependencies]` 里，来源是 git 仓库或本地目录：

```toml
[dependencies]
mathx = { git = "https://github.com/you/mathx", rev = "v1.0" }
util = { path = "../util" }
```

也可以用命令添加：`zam add mathx https://github.com/you/mathx v1.0`、`zam add util ../util`。`zam add` 修改当前目录的 `zam.toml` 并立即拉取，拉取或检查失败时把 `zam.toml` 与 `zam.lock` 恢复原样。

- **包的写法**：包就是一个普通的 Zamak 项目，`zam.toml` 里 `[project]` 的 `name` 要和依赖名相同，`entry` 可以省略（库不需要 `main`）。包可以再有自己的依赖，`zam` 会一起解析；同一个名字被要求从两个不同的来源拉取时报错。
- **导入**：包里的 `src/geo.zm` 写成 `use mathx/geo`，调用 `geo.twice(2)`；与包同名的 `src/mathx.zm` 是包的根模块，写成 `use mathx`。包内部的模块照常互相 `use geo`。只有 `pub` 的函数、结构体和常量能被项目使用；项目自己的模块不能和依赖同名。
- **锁定**：`rev` 可以是分支、标签或提交，省略时是默认分支。第一次构建把它解析成具体提交，写进 `zam.lock`（请提交到版本库），之后的构建都用锁定的提交，即使标签移动了也不变；`zam update` 忽略锁文件，把每个 `rev` 重新解析成最新的提交。路径依赖不进锁文件。
- **缓存与离线**：git 仓库的镜像与检出的源码放在缓存目录的 `packages/` 下（默认 `~/.zam/cache/packages`），按提交区分、只检出一次。锁定的提交已在缓存里时构建不访问网络。拉取 git 依赖需要 PATH 上有 `git`。
- **测试**：`zam test` 只运行项目自己的测试，不运行依赖包里的；`zam fmt` 也只整理项目自己的模块。

## 当前语法范围

- 代码块：用缩进界定，与 Python 相同。块头（`fn`、`struct`、`enum`、`if`/`elif`/`else`、`while`、`for`、`match` 与每个分支、`catch e`）以 `:` 结尾；后面要么在同一行写一条语句或一个值（`if n < 1: return n`），要么换行后多缩进 4 个空格写一块，空块写 `pass`。缩进退回到哪一层，块就结束在哪一层；退到不存在的层级时报 `unindent does not match any outer indentation level`。括号里的换行和缩进不算，空行与只有注释的行跳过。缩进里有 Tab 时报错并建议运行 `zam fmt`。花括号只用于字典字面量、字典类型和插值。块头后面的一行没有缩进、且整个文件几乎没有缩进时，报错会提示“复制时可能丢了缩进”。
- 函数：`fn`、`pub fn`。参数和返回值可以是 `i64`、`f64`、`bool`、`string`、结构体、列表和字典，省略返回类型表示没有返回值；参数还可以是借用 `&string`、`&mut string`、`&[T]`、`&mut [T]`、`&{K: V}`、`&mut {K: V}`、`&Point`、`&mut Point`。
- 错误值：签名写成 `T!E`（成功返回 T，失败返回 E）或 `!E`（只有失败）。`fail expr` 产生错误值并立即返回；`call?` 在失败时把错误传给调用方，成功时得到 T，要求当前函数签名也带 `!E`。`main` 可以是 `-> !E`，未捕获的错误写 stderr 并以退出码 1 结束。`call catch e:` 加缩进块就地处理错误，不向外传播：错误值是 string，只在代码块里可见，代码块要在所有路径上 `return` 或 `fail`（在循环里也可以用 `break`、`continue` 收尾）。`E` 可以是 `string`、结构体或枚举：`fail ConfigError.Missing(key)` 构造错误，`catch e:` 里用 `match e` 分情况处理，`?` 只在同一种错误类型之间传递（不同的用 `catch` 转换），`main -> !E` 未捕获时按 `println` 的格式把错误写到 stderr。示例：`zam run examples/values/custom_errors.zm`。示例：`zam run examples/values/errors.zm`、`zam run examples/values/catch.zm`。
- 顶层常量 `const LIMIT = 100`、`const RATE: f64 = 0.25`、`pub const NAME = "x"`：名字全大写，值在编译时由字面量、前面的常量和运算符算出（溢出、除零在编译时报错），用到的地方直接换成字面量；跨模块写 `config.LIMIT`。常量不能重新赋值，局部变量不能与常量同名。示例：`zam run examples/values/constants.zm`。
- `let name = expr`、`let mut name = expr`、可选类型标注，以及可变绑定的重新赋值。整数、浮点数和布尔值复制；字符串、结构体、列表和字典移动。
- 十进制整数可以在数字之间加单个 `_` 分隔符（如 `1_000_000`），放在首尾或连续使用会报错。整数算术 `+ - * / %`、比较 `== != < <= > >=`、布尔相等比较、单目 `-` 和括号。逻辑运算是 `and`、`or`、`not`，优先级与 Python 相同（从低到高：`or`、`and`、`not`、比较），`and`/`or` 短路求值，三个词都是保留字；写 `&&`、`||` 或前缀 `!` 会报错并提示对应的单词。`!` 只出现在错误类型 `T!string`、`-> !string` 里，`!=` 照旧。整数有位运算 `&`、`|`、`^`、`~`、`<<`、`>>`，优先级与 Python 相同（比较 < `|` < `^` < `&` < 移位 < 加减）；移位量要在 0–63 之间，左移丢掉移出的位，右移是算术右移。整数字面量还可以写成 `0xFF`、`0b1010`、`0o17`。条件里写单个 `=` 时报错，提示比较要用 `==`。表达式与参数从左到右求值；溢出、除零及 `i64::MIN / -1` 在运行时返回错误。
- 浮点数 `f64`：带小数点或指数的字面量（`1.5`、`2e-3`、`1_000.5`；小数点两边都要有数字，写 `0.5` 而不是 `.5`），按 IEEE 754 做 `+ - * / %` 与比较，不检查溢出，除以零得到 `inf` 或 `NaN`。`i64` 与 `f64` 不隐式转换，混用时报错并提示改法；显式转换写 `f64(n)` 与 `i64(x)`（向零截断，NaN 或超出范围时终止程序）。`println` 与插值输出能原样读回的最短十进制（`0.1`、`2.0`、`1e16`、`2.5e-7`），`fmt.fixed(x, n)` 保留 `n` 位小数，`string.parse_f64(&s) -> f64!string` 解析十进制文本。`std/math` 提供 `sqrt`、`pow`、`exp`、`log`、`log10`、`sin`、`cos`、`tan`、`atan`、`atan2`、`floor`、`ceil`、`round`、`trunc`、`abs`、`min`、`max`、`is_nan` 和常量 `math.pi`、`math.e`、`math.inf`，参数都是 `f64`。示例：`zam run examples/values/floats.zm`。
- 复合赋值 `+= -= *= /= %= &= |= ^= <<= >>=` 可以用在可变变量、字段和元素上（`xs[i] += 1`、`counts["a"] += 1`），元素下标里不能调用函数。
- if 表达式按条件选值：`let v = if c: a elif d: b else: e`。必须有 `else`，每个分支是一个表达式，类型相同；分支各自移动的值按两边合并检查。示例：`zam run examples/values/expressions.zm`。
- 字符串用 `a + b` 拼接，生成新字符串，两边都不移动；`== != < <= > >=` 按字节比较内容。字符串不能与整数相加。
- 本地/导入函数调用、嵌套调用、`return`。函数尾部的值表达式可以作为返回值：函数体的最后一条，以及处在尾部的 `if`/`elif`/`else` 分支的最后一条。循环体和非尾部分支末尾的调用只求值，不会让函数返回。
- `println(expr)` 打印字符串、整数、浮点数或布尔值；打印变量时不移动变量。导入 `std/io` 或 `std/fmt` 后可以使用相应的命名空间。
- 导入 `std/string` 后，`string.byte_len(&text) -> i64` 返回 UTF-8 字节数，`string.char_len(&text) -> i64` 按 UTF-8 码点计数（一个汉字算一个字符，续字节不计），`string.equal(&a, &b) -> bool` 按字节比较内容；它们只做共享借用，不移动字符串，共享借用参数可以直接转传。`&string` 参数接受共享借用，也接受字符串字面量与插值（借用一个只活到语句结束的临时字符串，例如 `string.split(&csv, ",")`）；不接受 owned 变量（要写 `&name`）或可变借用。示例：`zam run examples/values/strings.zm`。
- 文件、标准输入与命令行参数：`fs.read(&path) -> string!string`、`fs.write(&path, &text) -> !string`；`io.print(&text)`（不换行）、`io.eprintln(&text)`（写到 stderr）、`io.read_line(&mut line) -> bool!string`（去掉行尾 `\n`/`\r\n`，输入结束返回 `false`）、`io.read_all() -> string!string`；`env.arg_count() -> i64`、`env.arg(i) -> string!string`（不含程序名，`zam run <file> -- a b` 把 `--` 之后的参数传给程序）、`env.var(&name) -> string!string`（读环境变量）、`env.exit(code)`（以 0–255 的退出码立即结束，`catch` 块可以用它收尾；`zam run` 原样带回程序的退出码）。不用导入的 `abs(x)`、`min(a, b)`、`max(a, b)` 对 `i64` 和 `f64` 都能用。字符串还有 `string.parse_i64(&s) -> i64!string`、`string.find_from(&s, &needle, start)`、`string.starts_with/ends_with/contains`、`string.replace(&s, &from, &to)`、`string.lower/upper`（仅 ASCII）、`string.trim(&s) -> string`、`string.find(&s, &needle) -> i64`（找不到为 -1）、`string.byte_at(&s, i) -> i64` 与 `string.slice(&s, start, end) -> string`；后两者按字节计位置，越界或切在多字节字符中间时与数组越界一样终止程序。按字符处理用 `string.chars(&s) -> [string]`（每个字符一项）、`string.codepoints(&s) -> [i64]` 与 `string.from_code(n)`，按字节用 `string.bytes(&s) -> [i64]` 与 `string.from_bytes(&list) -> string!string`（不是合法 UTF-8 时返回错误）；另有 `string.repeat(&s, n)`、`string.count(&s, &sub)`，以及按字符数补空格对齐的 `string.pad_left/pad_right(&s, width)`。路径与参数按 UTF-8 处理，Windows 通过 `_wfopen` 与 `GetCommandLineW` 支持中文路径和参数。这些函数照常参与类型、借用与错误值检查，C 实现只发射到用到它们的目标文件里。示例：`examples/io/wc.zm`（行、词、字节计数）、`examples/io/sum.zm`（逐行求和）、`examples/io/notes.zm`（写入、读回并拆分 `key=value`）。
- 可增长列表 `[T]`：元素为 `i64`、`f64`、`bool`、`string` 或结构体，可以作局部变量、参数（`[T]` 移交所有权，`&[T]` / `&mut [T]` 调用帧借用）、返回值和结构体字段。空列表需要类型标注（`let mut xs: [i64] = []`），不带标注的 `[1, 2]` 仍是定长数组。方法有 `xs.len()`、`xs.push(v)`（字符串移入列表）、`xs.pop()`、`xs.insert(i, v)`、`xs.remove(i)`（删除并返回该元素）、`xs.swap(i, j)`、`xs.sort_by(fn(a, b): a.age < b.age)`（任何元素都能按自定义规则稳定排序，函数返回真表示 a 排在前面；闭包不能读正在排序的列表）、`xs.reverse()`、`xs.clear()` 和 `xs.extend(ys)`（把 `ys` 整个移到末尾）；元素是 `i64`、`f64`、`bool`、`string` 时还有 `xs.sort()`（升序、稳定，NaN 排最后）、`xs.contains(v)`、`xs.index_of(v)`（找不到为 -1）和 `xs.slice(a, b)`（复制出下标 `a` 到 `b` 的新列表）。`insert`、`remove`、`slice` 越界时终止程序。`xs[i]` 读写元素，越界或对空列表 `pop` 时终止程序。字符串元素不能经 `xs[i]` 移出，可以借用 `&xs[i]`，或用 `string.copy(&xs[i])` 复制。`for x in &xs` 遍历时，整数与布尔按值得到，字符串是只读借用；`for i, x in &xs` 同时给出下标；循环期间列表被冻结，不能增删、移动或独占借用。`for x in &mut xs` 原地修改元素：整数、浮点、布尔元素给 `x` 赋值就是写回元素，字符串元素是 `&mut string`，结构体元素是 `&mut Point`。方法接收者可以是变量、字段路径（`bag.items.push(x)`）或调用结果（`string.split(&s, ",").len()`，结果是临时值，语句结束时释放）；`a.b(...)` 中的 `a` 是当前函数声明过的局部名时才是方法调用，否则按模块调用解析。配套标准库：`string.split/words/lines` 返回 `[string]`，`string.join(&list, &sep)`，`string.copy(&s)`，`env.args()`。C 侧每种元素类型生成一组 `L*` 结构与函数，只发射到用到它的目标文件。示例：`examples/collections/stats.zm`（读入整数、排序、统计）、`examples/collections/tac.zm`（倒序输出并找最长单词）。
- 字典 `{K: V}`：键为 `i64` 或 `string`，值为 `i64`、`f64`、`bool`、`string` 或结构体；可以作局部、参数（`{K: V}` 移交，`&{K: V}` / `&mut {K: V}` 借用）、返回值和结构体字段。字面量写作 `{"amy": 30, "bob": 25}`（逗号分隔，可分行），空字典需要类型标注 `let mut m: {string: i64} = {}`。`m[k]` 读取（键不存在时终止程序，`zam: key not found`），`m[k] = v` 插入或替换；字符串键以只读借用传入（`m[&name]`、`m["a"]`，或遍历得到的键 `m[k]`），新键插入时复制一份。方法有 `len()`、`contains(k)`、`get(k, default)`（不存在时返回 `default`；字符串值返回副本）、`remove(k) -> bool`、`keys() -> [K]`、`values() -> [V]`（结构体值没有）和 `clear()`。`for k in &m` 按插入顺序遍历键，`for k, v in &m` 同时给出值，`for k, v in &mut m` 可以改值；期间字典被冻结，不能增删条目。字符串值不能经 `m[k]` 移出，可以借用 `&m[k]` 或用 `get`。实现是插入顺序表加开放寻址下标表：遍历顺序确定，与哈希无关；删除留墓碑，重建下标表时压实。哈希不带随机种子，不适合直接处理不可信输入的大规模键集合。示例：`examples/collections/wordfreq.zm`（词频统计，按次数稳定排序）。
- 展示程序 `examples/md2html`：约 270 行、4 个模块的 Markdown 转 HTML 工具，支持标题、段落、有序/无序列表、引用、围栏代码块、分隔线，以及行内代码、粗体、强调、链接与 HTML 转义。运行 `zam run examples/md2html -- examples/md2html/sample.md`；期望输出 `sample.html` 由 `zam/tests/showcase.rs` 核对。
- 字符串支持 UTF-8，以及 `\n`、`\r`、`\t`、`\"` 和 `\\` 转义。字符字面量 `'a'`、`'\n'`、`'\''` 的值是该字符的 Unicode 码点（`i64`），可以直接和 `string.byte_at` 的结果比较 ASCII 字符。
- 字符串字面量支持插值：`"{name}"` 拼接当前作用域里的变量或字段（`{person.name}`），支持 `string`、`&string`、`&mut string`、`i64`、`f64`、`bool`。插值只读取，不移动，也不修改借用内容，插值后原变量仍可使用；字面大括号写作 `{{` 和 `}}`。花括号里也可以写表达式，例如 `"{a + b}"`、`"{xs[0]}"`、`"{string.upper(&name)}"`，从左到右求值；表达式里不能再出现字符串字面量或花括号。示例：`zam run examples/values/format.zm`。
- `if 条件:`、`elif 条件:`、`else:`（没有 `else if`，写了会提示改成 `elif`）、`while 条件:`，条件要是 `bool`；`for name in start..end:` 遍历左闭右开的整数区间，`start` 与 `end` 进入循环前按顺序各求值一次，循环变量是只读 `i64`，只在循环体内可见（`zam run examples/control/loops.zm`）；`and`/`or` 短路求值。块内变量不能逃出块；`elif` 链按嵌套分支处理返回与所有权合并。支持 `break`/`continue`，跳转后同一代码块中的语句不可达，嵌套循环只影响当前循环；暂不支持遮蔽仍在作用域内的绑定。
- `struct` / `pub struct`：名称首字母大写，字段支持 `string`、`i64`、`f64`、`bool`、owned 命名结构体、列表和字典，每行一个字段。结构体声明写成 `struct Person:`，字段缩进写在下面；`pub struct` 跨模块可见。字面量写成 `Person(name: "a", age: 1)` 或 `model.Person(...)`：逗号分隔，每个字段都写名字，与枚举变体的写法一致，可以分行写。函数名首字母大写会报错，避免和结构体字面量混淆。可以读取整数/布尔字段，也可以直接打印字符串字段（不移动）。支持可变结构体字段赋值、整体移动、整体重新赋值和字符串字段自动释放；替换字符串字段时先计算新值，再释放旧值。重复/缺失字段、借用字段和移动后使用会报错。支持 owned 结构体作函数参数、返回值和变量类型标注（跨模块用 `model.Person`）；传参及返回转移整体所有权，未使用的返回值会自动释放。嵌套 owned 结构体支持构造、字段整体替换及递归释放；直接/间接递归布局会报错。支持连续字段访问（如 `a.b.c`）：读取标量、不移动地打印字符串、替换可变根绑定下的字段；路径上的每一层都检查类型、字段名称和根绑定有效性。字符串字段支持调用帧借用（如 `&person.name`、`&mut person.name`，可嵌套），独占借用要求根绑定可变。冲突按根结构体保守检查，不允许同时独占借用不同字段；借用字段时不能移动根结构体。整个结构体也能按调用帧借用：参数写 `&Point` 或 `&mut Point`，实参写 `&p`、`&mut p`；已经是借用的参数直接转传，`&mut` 借用可以改字段或整体赋值，共享借用不能写，借用的结构体不能移出。暂不支持单独移出 owned 字段。示例：`zam run examples/structs`。
- 方法：写在结构体或枚举的缩进块里（字段或变体之后），第一个参数是 `&self`（只读）、`&mut self`（可改字段或整体赋值）或 `self`（移入）。调用写 `p.norm()`、`xs[i].norm()`、`make().norm()`，接收者按需要自动借用或移入；没有 `self` 的是关联函数，用类型名调用：`Point.new(1, 2)`、跨模块 `geo.Point.new(1, 2)`。跨模块调用要 `pub fn`。实现上方法就是函数表里叫 `Point.norm` 的普通函数，检查器与生成器经 `compiler::record_method` + `method_args` 把方法调用改写成 `Point.norm(&p, ...)`。示例：`zam run examples/structs/methods.zm`。
- 结构体列表与字典：`[Point]`、`{string: Point}`（跨模块写 `[model.Point]`），`push` 移入、`pop` 移出、`swap` 交换整个元素。`xs[i].f` 读写元素字段（可以复合赋值，下标里不能调用函数），`&xs[i]`、`&mut xs[i]` 借用元素，`&xs[i].name` 借用元素的字段，`xs[i].tags.push(v)` 调用元素字段上的容器方法，`xs[i] = Point(...)` 整体替换（先释放旧值）。`for p in &xs` 得到只读借用 `&Point`。结构体不能复制：`let p = xs[i]` 报错，结构体值的字典不支持 `get`。C 侧所有类型先前置声明，再按依赖定义（列表与字典本体只含指针，结构体按按值字段排序，字典条目最后），释放函数先给原型，所以字段写成 `children: [Node]` 的 `struct Node:` 这样经由列表的递归也能编译（`examples/structs/tree.zm`）。示例：`zam run examples/structs/elements.zm`、`zam run examples/collections/gradebook.zm < examples/collections/grades.txt`。
- 枚举：`enum Shape:` 下面每行缩进写一个变体 `Circle(r: f64)`、`Empty`，变体名首字母大写；构造按位置传值 `Shape.Circle(1.0)`、`Shape.Empty`（跨模块 `shapes.Shape.Empty`）。实现上枚举是一种结构体：字段 0 是标签，各变体的数据平铺在后面，没选中的部分保持全零，所以移动、释放、列表、字典、`&Shape` 借用都沿用结构体的规则。`match s:` 下面每行一个分支 `Circle(r): ...`、`Rect(w, _): ...`、`_: ...`，有语句和表达式两种写法：语句形式的分支是同一行的一条语句或缩进的一块，表达式形式（`let x = match s:`）的分支是一个值；分支要覆盖所有变体或以 `_` 结尾，重复分支、`_` 之后的分支、载荷个数不符都会报错（`ZM511`/`ZM201`/`ZM200`）。`match` 只借用被匹配的值：变量、字段与元素（`xs[i]`、`xs[i].status`）在匹配期间冻结，其他表达式求值成临时值；标量数据按值绑定，其他数据是只读借用。分支写成 `=>` 时报错并提示改成冒号。示例：`zam run examples/control/enums.zm`、`zam run examples/control/calc.zm < examples/control/calc.txt`。
- 复合值的复制、比较与打印：`copy(&x)` 深复制字符串、列表、字典、结构体或枚举（参数要写借用，如 `copy(&xs[i])`、`copy(&p.tags)`），改副本不影响原值。结构体、枚举、列表、字典支持 `==`/`!=` 逐项比较（字典与插入顺序无关，浮点按 IEEE 754），不支持 `<`。`println(x)` 与插值 `"{x}"` 直接显示它们，写法与构造一致：`Point(x: 1, y: 2)`、`Shape.Circle(1.5)`、`Shape.Empty`、`[1, 2]`、`{"a": 1}`，里面的字符串加引号并转义。三者都只读取，不移动变量。C 侧按类型生成 `copyX`/`eqX`/`showX`/`textX`，只在目标文件用到时发射。示例：`zam run examples/structs/values.zm`。
- 字面量 `match`：分支模式可以是整数、字符、字符串、布尔字面量，`|` 列出多个（`1 | 2 | 3: ...`、`"go" | "run": ...`），`_` 接住其余；解析时展开成 `if`/`elif` 链，被匹配值是调用时先存进隐藏变量、只求值一次。语句形式可以没有 `_`，表达式形式要有 `_` 或把 `true`、`false` 都列出。不能与枚举变体混用，字面量重复会报错。示例：`zam run examples/control/literal_match.zm`。
- 嵌套容器：列表元素与字典值也可以是列表或字典，层数不限，如 `[[i64]]`、`{string: [string]}`、`[{string: i64}]`。`grid[i][j]` 读写内层元素（两层以内；字符串元素读出副本），复合赋值 `grid[i][j] += 1` 也行；`grid[i].push(x)`、`m[k].len()` 在元素上调用方法；`for row in &grid` 得到 `&[i64]`，`for row in &mut grid` 得到 `&mut [i64]`；`&grid[i]` 借给函数。内层容器不能用 `grid[i]` 直接取出，打印、比较、`copy` 都支持。实现上内层类型是 `Elem::Nested(编号)`，编号由内层类型的结构算出（`compiler::nest`），C 名字按结构拼接，如 `LLi`、`MsLs`。示例：`zam run examples/collections/nested.zm`。
- 泛型函数：`fn largest[T](items: &[T]) -> T`（Python 3.12 的写法），调用处按实参类型推断类型参数，为每种类型生成一份专门的函数（单态化，名字如 `largest[Int]`），函数体按具体类型检查，报错会写出是哪个版本。类型参数要在某个参数类型里出现；`&T` 遇到数字或布尔按值传；跨模块调用要 `pub fn`。示例：`zam run examples/values/generics.zm`。
- 泛型结构体与枚举：`struct Pair[A, B]:`、`enum Tree[T]:`，类型写作 `Pair[i64, string]`、`Tree[string]`，字段、变体与方法里都能用类型参数。构造时按字段或载荷的值推断类型实参；推断不出时（`Option.None`、`Stack(items: [])`）取期望类型：变量标注、参数、返回值、列表元素、if/match 的分支都算，`==` 的右边不算，推断不出会报错并提示标注。方法是泛型函数，按接收者的类型实参生成副本；`[1, 2]` 填进类型参数的位置时是列表。内置 `Option[T]`，变体 `Option.Some(value)`、`Option.None`，模块自己声明了 `Option` 时用自己的。实现上每组类型实参生成一个实例结构体（如 `Option[Int]`），名字由模板名与实参拼成、编号是名字的哈希，所以不同模块用到的同一实例是同一个类型；打印时只显示模板名（`Option.Some(3)`）。示例：`zam run examples/values/generic_types.zm`。
- 函数值与匿名函数：函数类型写作 `fn(i64, &string) -> bool`（没有 `->` 表示没有返回值）。具名函数直接写名字当值传（`double`、`util.double`）；匿名函数写作 `fn(x: i64) -> i64: x * 2`，冒号后在同一行写一个表达式或一条 `println(...)`，期望的函数类型已知时参数类型和返回类型都可以省略（`map(&words, fn(w): string.upper(w))`，泛型函数先用其他实参推断出类型参数再补上）。匿名函数可以读外层函数的局部变量（闭包），只读不写：owned 值按共享借用看，数字、布尔读的是调用时的当前值；捕获了变量的闭包只能直接当实参传，参数位置的函数类型也只能调用或继续转传，所以闭包不会活得比被捕获的变量久，不需要垃圾回收。不捕获变量的函数值可以存进变量、结构体字段、枚举载荷，也能作返回值；结构体里的函数字段用方法的写法调用（`button.on_click(1)`）。泛型函数和会失败的函数暂时不能直接当值，包一层匿名函数即可。实现上函数值是 C 结构体 `F<编号>`（函数指针 + 环境指针），匿名函数体提成静态函数，捕获的变量把地址放进调用方栈上的环境结构体。示例：`zam run examples/values/functions.zm`。
- 元组：类型 `(i64, string)`，值 `(3, "three")`，`t.0`、`t.1` 读写各项；函数返回元组即返回多个值，`let (q, r) = divmod(17, 5)` 拆开（`_` 丢弃该项，元组整体被移走）。元组可以作参数、返回值、列表元素与字典值，支持比较、打印与 `copy`。实现上元组是名字由字段类型拼成的匿名结构体（如 `(Int, Owned)`），编号是名字的哈希，所以不同模块里同样的元组类型是同一个类型；检查时推断出新的元组类型会重新链接再检查。示例：`zam run examples/values/tuples.zm`。
- `for x in xs:` 与 `for x in &xs:` 相同，只借用不移走。
- 换行分隔语句；支持 `#` 和 `//` 单行注释。

### 所有权与借用

```zam
fn show(s: &string): println(s)

fn change(s: &mut string): s = "changed"

fn consume(s: string): println(s)

fn main():
    let mut a = "hello"
    show(&a)
    change(&mut a)
    let b = a
    consume(b)
```

赋值和 owned 参数传递会移动字符串，移动后再使用在 `check` 阶段报错。可变变量可以重新初始化。借用只能作为参数，调用返回即结束；共享借用可以有多个，可变借用要独占，所有者也要可变。同一次调用中借用和移动同一个变量会报错，嵌套调用也遵守外层已开始的借用。借用参数可以继续传给下层调用，但不能移动、返回或存进本地变量。字符串靠显式分配和作用域结束时释放来管理，不使用 GC 或引用计数。

运行 `zam run examples/ownership` 查看完整示例。分支之后只允许使用在所有继续路径上都有效的字符串；返回分支不参与合并。循环回到下一轮前，要重新初始化已移动的外层字符串。检查保守地分析所有分支，不利用常量条件排除路径。`zam run examples/control/main.zm` 演示阶乘、GCD 和循环移动后重初始化；`zam run examples/control/jumps.zm` 演示嵌套循环中 `continue` 只跳到当前循环的下一轮、`break` 只退出当前循环。`break`/`continue` 后同一代码块中的语句不可达，在检查阶段报错。支持定长 `[i64; N]` 数组字面量、整数索引、可变元素赋值和运行时边界检查；长度限制为 1–1024，不能移动或借用，需要传递或增长的数据用列表 `[T]`。当前还没有线程、unsafe 或 FFI，因此不能宣称已实现完整的 Rust 级内存安全。原生程序没有解释器的步数限制，无限递归可能耗尽栈。

## 缓存行为

缓存默认放在用户目录 `.zam/cache`，可用 `ZAMAK_CACHE_DIR` 指定。模块键包含源码字节、依赖名称与递归依赖哈希、Zamak 编译器/IR 版本（当前 IR-30）、操作系统和架构，不含项目绝对路径。相同源码从不同目录构建时共享模块产物。

修改依赖会使该模块和传递依赖它的模块失效，独立模块继续命中缓存。当前仍对全部源码执行解析和检查，并比较缓存产物内容；缓存验证以正确性优先，还没有实现只读接口的语义增量分析。

性能验证的方法、适用范围与首轮结果见 [案例验证](docs/VALIDATION.md)，后续优化与 Rust/Go 对照见 [性能优化记录](docs/PERFORMANCE.md)。构建时间随模块规模、缓存状态、平台工具链和机器负载变化；运行性能还受整数溢出与边界检查影响。后端用保守的整数区间分析消除可证明冗余的检查，并为有调用范围证明的同模块整数函数生成有限数量的内部版本；无法证明安全的操作保留运行时检查。GNU 风格后端使用 `-O2 -funroll-loops`，MSVC 使用 `/O2`；循环展开可能增加编译时间与代码体积，实测代价见性能记录。阅读性能结论时，请结合输入、检查语义和重复次数一起看。

Windows 工具链环境在首次调用时捕获并缓存，之后直接调用编译器与链接器。目标文件编译使用有界工作队列，默认并发度为可用核数且不超过 8，可通过 `ZAMAK_JOBS` 调整。缓存命中的模块不占用编译队列。

缓存发布过程：在同一目录独占创建临时文件，写入并同步后，通过硬链接原子发布最终文件。并发构建可能重复计算，同一缓存键只保留一个完整产物；内容不同或缓存损坏会报错。缓存所在的文件系统要支持硬链接。正常执行会清理临时文件；进程被强制终止时可能留下 `.tmp` 文件。缓存清理和容量限制还没有提供子命令。

模块缓存保存确定性的 IR；原生后端为每个模块生成独立的 C 和目标文件。目标文件键包含模块生成的 C（实现、实际调用函数的接口声明，以及本模块函数实际构造或实际使用的函数签名中的结构体布局）、平台、架构、编译选项和本机工具链身份。无关的结构体布局变更不改变目标文件键或链接键；已使用的布局变更会重新检查，并使使用它的模块失效。只修改被调用函数的实现时，调用方目标文件继续命中；修改实际使用的接口时，调用方失效或在类型检查阶段报错。函数符号由完整的模块/函数名称稳定编码，增加无关函数不改变已有符号。

Windows 编译使用 `/Brepro` 与 `/d2pathmap` 把临时目录映射为固定路径，避免同一键的目标文件因构建目录不同而产生不同字节；GCC/Clang 用 `-ffile-prefix-map` 做同样的事，Windows 上的 MinGW 链接再加 `-Wl,--no-insert-timestamp`，不让链接时间写进可执行文件头。并发回归测试会校验这一点。

链接键包含按模块排序的目标文件键、工具链身份和链接选项。目标文件与可执行文件都有 SHA-256 校验，输出分别显示 `OBJECT HIT/MISS` 和 `LINK HIT/MISS`。项目产物同样按内容命名、原子发布。IR 缓存仍使用递归源码依赖键，还没有实现语义检查缓存。`check` 不写缓存或构建产物。

## 验证

```powershell
cargo test --manifest-path zam/Cargo.toml --locked
cargo fmt --manifest-path zam/Cargo.toml -- --check
cargo clippy --manifest-path zam/Cargo.toml --all-targets --locked -- -D warnings
node editors/vscode/scripts/check-grammar.mjs
pwsh -File bench/ai/run.ps1 -ZamPath .\zam\target\debug\zam.exe -LatencyRepeats 3
pwsh -File tools/measure-build.ps1 -ZamPath .\zam\target\debug\zam.exe -Modules 6 -Repetitions 3
python bench/ai/live/run.py --verify-solutions
```

`check-grammar.mjs` 校验编辑器扩展的声明式配置：三个 JSON 可以解析，TextMate 语法里每条正则可以编译，语言 id、后缀、`scopeName` 与仓库分组引用互相一致。

`bench/ai/run.ps1` 和 `tools/measure-build.ps1` 是可重复的工程测量。离线 AI 修复评测用固定候选和补丁，验证诊断、修复、构建与输出是否全部通过；构建测量生成固定形状的多模块工程，断言冷构建全 MISS、热构建全 HIT、改实现只重编一个模块、改接口重编该模块与调用方。两者都只把结果写进被忽略的 `results/`，断言失败即报错。

`bench/ai/live/run.py --verify-solutions` 检查实时 AI 对比实验的任务与参考解；完整实验要调用模型 API，用法见 [bench/ai/live](bench/ai/live/README.md)。

集成测试覆盖原生执行、模块与缓存、并发发布、所有权与借用、错误处理、源码诊断，以及新建项目与格式化，同时包含成功路径和应被拒绝的输入。教程代码块会自动编译检查；常见算法案例另有原生运行与完整输出断言。测试方法与性能结论见 [案例验证](docs/VALIDATION.md)，开发规范见 [贡献指南](CONTRIBUTING.md)。

Zamak 仍是原型，语法与库都可能变化。

## 许可证

本项目按 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双许可发布，使用者可以任选其一。除非另有声明，你提交的贡献同样按这两种许可证发布。
