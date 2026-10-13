# Zamak

[English](README.en.md) | 中文

面向快速构建的静态类型原生语言原型。

[![CI](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml/badge.svg)](https://github.com/rayL-K/Zamak/actions/workflows/ci.yml)

[教程](TUTORIAL.md) · [预览版安装](PREVIEW.md) · [参与开发](CONTRIBUTING.md) · [AI 工作流](docs/AI_WORKFLOW.md) · [案例验证](docs/VALIDATION.md) · [工程指标](docs/ENGINEERING.md)

<img width="160" height="160" alt="Zamak" src="ico/Zamak_256x256.png" />

Zamak 是为频繁构建、多 worktree 协作设计的语言原型。当前 `zam` 支持多模块构建缓存、独立原生可执行文件，以及 `string` 的所有权和调用帧借用检查。原生后端先生成 C，再调用 MSVC（Windows）或 `cc`（Unix）生成机器码；安全语义由 Zamak 检查，用户程序不经过 Rust 编译器。

## 定位

- **构建与缓存**：同一份源码可以在不同目录和工作树之间共享缓存。实现变更只需重编译受影响的原生模块；使用到的函数接口或结构体布局变化会使调用方失效。
- **类型与所有权**：静态类型检查覆盖变量、函数和结构体；字符串和结构体使用移动语义，字符串支持调用期间的共享与独占借用。
- **原生执行**：生成 C 后调用平台工具链编译。生成的程序可以独立运行，无需安装 Zamak 或 Rust。

当前版本适合学习、语言实验与小规模算法程序。标准库与工具生态尚不完整，不适合生产使用。

### 生态现状

| 组件 | 当前能力 |
| --- | --- |
| 命令行 | `check`、`build`、`run`、`fmt`、`new`、`demo`；检查与构建支持 JSON 诊断 |
| 集合 | 定长 `[i64; N]`、可增长列表 `[i64]`/`[bool]`/`[string]`、字典 `{K: V}`（键为 `i64` 或 `string`） |
| 标准库 | 文件读写、标准输入、命令行参数、整数解析，字符串的长度、比较、裁剪、查找、切片、切分与连接，以及 `println` |
| 错误处理 | `T!string`、`fail`、`?` 传播、`catch` 处理 |
| 编辑器 | VS Code 语法高亮、括号与引号配对 |

尚未支持浮点数、结构体元素的列表与字典、网络 I/O、目录遍历、泛型、枚举与 `match`、闭包、线程及 FFI。包管理器、语言服务器、REPL 和文档生成工具尚未提供。安装方式见 [预览版说明](PREVIEW.md)。

## 快速运行

初次使用请阅读 [入门教程](TUTORIAL.md)。Windows 预览包和平台工具链要求见 [安装说明](PREVIEW.md)。

从源码安装需要 Rust stable 和平台 C 编译器。在仓库根目录执行：

```powershell
cargo install --path zam --locked
```

确保 Cargo 的安装目录（通常为用户目录下的 `.cargo/bin`）位于 PATH。修改编译器源码后，重新执行安装命令更新。首次使用可以创建并运行项目：

```powershell
zam --version
zam new hello
zam check hello
zam build hello
zam run hello
zam fmt hello
```

`hello` 应为尚不存在的目录。进入该目录后，可以直接执行 `zam run`。新建项目的程序输出为：

```text
hello from zamak
```

构建进度写入 stderr，程序输出写入 stdout。多模块、中文输出及所有权示例见 [教程](TUTORIAL.md)。

`build` 在 stderr 输出 `OUTPUT <path>`，Windows 上是 `build/<name>-<hash>.exe`，Unix 上是同名的无后缀可执行文件，可以直接启动，也可用 `zam run <path>` 执行。产物不需要安装 Zamak 或 Rust；Windows 默认 MSVC 后端静态链接 C 运行库，其他 C 工具链的运行库依赖取决于其配置。命令失败返回非零退出码，错误写入 stderr。词法/语法错误包含源码文件、行号与列号；列号从 1 开始，按 Unicode 字符计数，制表符算一个字符，EOF 使用末尾位置。词法错误指向 token 起点，语法错误指向解析器发现错误的位置。语义错误包含行号、列号与所属函数，位置取当前语句的起点（`missing string return value` 取函数声明处）；因为一个模块对应一个文件，函数前面的模块名即可定位文件。没有路径参数时使用当前目录；源码后缀统一为 `.zm`；可以直接检查或构建单个源码文件。构建进度（`CACHE`/`OBJECT`/`LINK`/`OUTPUT`）和错误都写 stderr，stdout 只留程序输出。

`zam check --json` 与 `zam build --json` 把结果写成一行 JSON 供编辑器、CI 和 AI 工具读取：`{"command":"check","ok":false,"diagnostics":[{"file":"…","line":3,"column":9,"function":"main:main","code":"ZM100","category":"syntax","message":"…","suggestion":"…"}]}`。`code` 和 `category` 是稳定机器字段，`message` 保持人类可读兼容；`suggestion` 在有明确修复路径时提供下一步。`function` 与位置在无法定位时为 `null`。词法、语法与模块解析错误只报第一条；类型与所有权检查逐个函数进行，每个出错的函数各报一条，`diagnostics` 按位置排序。失败时 stderr 仍有人类可读的诊断，退出码与不带 `--json` 时一致。其他子命令不接受 `--json`。

不带 `--json` 时，有位置的诊断写成错误码、位置、源码行、下划线与修改建议（中文与制表符按显示宽度对齐）：

```text
error[ZM301]: use after move: name
 --> src/main.zm:8:5
  |
8 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

`zam new <directory>` 生成一个最小可运行的项目骨架；面向 AI 协作可使用 `zam new <directory> --template ai`，额外生成 `AI.md` 与 `README.md`，固定记录“check --json → 最小修复 → fmt → check → build → run”循环。`zam.toml`（`name`/`entry = "src/main.zm"`/`modules = "src"`）、打印一句问候的 `src/main.zm`，以及忽略 `build/` 的 `.gitignore`；创建的文件路径以 `NEW <path>` 写到 stderr，stdout 保持为空。生成的骨架可以直接 `zam run`，并已完成格式化。目标目录已存在就报错退出，不覆盖任何已有文件；目录名必须同时是合法标识符（它会写进 `zam.toml` 的 `name`）。

`zam fmt [project|file]` 就地整理源码格式：按大括号深度重排每行缩进（每次 4 个空格）、去掉行尾空白、结尾统一留一个换行，并保留原有的换行风格（CRLF 文件仍是 CRLF）。它只改空白、不重建 token，所以注释、字符串内容和行内空格原样保留，不会丢注释；改写过的文件路径以 `FORMAT <path>` 写到 stderr，已经规范的文件不重写也不报告。语法不成立的代码会先被拒绝（与 `check` 同一套解析），不会被改得更难读。当前不做行内空格重排和长行折行。

C 工具链按这个顺序选择：设置了 `CC` 就用它；否则 Windows 先通过 `vswhere` 查找 Visual Studio Build Tools（「使用 C++ 的桌面开发」及 Windows SDK）里的 x64 MSVC，找不到时依次改用 PATH 上的 `gcc`（如 MinGW-w64）、`clang`；Linux 与 macOS 依次使用 `cc`、`clang`、`gcc`。都找不到时报错并说明该安装什么。`CC` 必须指向采用 GNU 命令行的编译器程序，不接受附加参数或 MSVC `cl`。已在 Ubuntu 26.04 + gcc 15.2，以及 WSL2 Ubuntu 24.04 + gcc 13.3 / clang 18.1 上实机验证构建、执行和全部集成回归；macOS 由 CI 验证。`check` 不需要 C 编译器。旧 `.zbc` 产物已不再支持，需要重新构建。

## 编辑器与 AI 工具

- **VS Code 语法扩展**：`editors/vscode/`。纯声明式（一份 TextMate 语法 + 语言配置，没有 `main`、没有运行时依赖、不联网），提供 `.zm` 文件的关键字/类型/字符串插值/注释高亮与括号、引号配对；同一份 `syntaxes/zamak.tmLanguage.json` 也可给 Sublime Text、TextMate 等支持 TextMate 语法的编辑器复用。本地安装、打包 `.vsix` 与上架市场的步骤见 `editors/vscode/README.md`。
- **AI 技能**：`.agents/skills/zamak/SKILL.md`。面向遵循 `.agents/skills` 约定的 AI 编程代理（DSH、Claude Code 等）：命令与环境变量、语法速览、常见诊断文本、示例与测试的位置，以及修改编译器本身时的验收命令和 IR/缓存版本号规则。
- **CLI**：就是 `zam` 本身（`check`/`build`/`run`/`fmt`/`new`/`demo`、`--help`、`--version`，`check`/`build` 支持 `--json` 机器可读诊断；缓存目录可用 `ZAMAK_CACHE_DIR` 覆盖）。还没有包管理器。

## 项目配置

```toml
[project]
name = "hello"
entry = "src/main.zm"
modules = "src"
```

配置必须只有 `[project]` 和上述三个字符串字段；路径相对配置文件，不能包含 `..`，入口必须在模块目录内。工具递归扫描模块目录里的 `.zm` 文件；一个文件对应一个模块，例如 `src/tools/greeting.zm` 的模块名是 `tools/greeting`。暂不支持源码符号链接。

```zam
use tools/greeting
use std/io

fn main() {
    greeting.hello()
    io.println("你好")
}
```

被导入模块里的函数必须声明为 `pub fn`。导入使用 `/`，调用使用模块路径的末段名称；有歧义的模块别名、缺失模块、私有函数调用、重复声明和模块依赖环都会报错。

## 当前语法范围

- `fn`、`pub fn`，带类型的参数：`string`、`&string`、`&mut string`、`i64`、`bool`。返回值支持 `string`、`i64`、`bool`，或省略表示无返回值。
- 错误值：签名写成 `T!E`（成功返回 T，失败返回 E）或 `!E`（只有失败）。`fail expr` 产生错误值并立即返回；`call?` 在失败时把错误传给调用方，成功时得到 T；要求当前函数签名也带 `!E`。`main` 可以是 `-> !E`，未捕获的错误写 stderr 并以退出码 1 结束。`call catch e { ... }` 就地处理错误，不向外传播：错误值是 string 并只在代码块里可见，代码块必须在所有路径上 `return`。当前只支持 `E = string`。示例：`zam run examples/values/errors.zm`、`zam run examples/values/catch.zm`。
- `let name = expr`、`let mut name = expr`、可选类型标注，以及可变绑定的重新赋值。整数和布尔值复制，字符串移动。
- 十进制整数允许在数字之间使用单个 `_` 分隔符（如 `1_000_000`），首尾或连续分隔符会报错。整数算术 `+ - * / %`、比较 `== != < <= > >=`、布尔相等比较、单目 `- !` 和括号。表达式与参数从左到右求值；溢出、除零及 `i64::MIN / -1` 在运行时返回错误。
- 本地/导入函数调用、嵌套调用、`return`；函数尾部的值表达式可作为返回值：函数体的最后一条，以及处在尾部的 `if`/`else` 分支的最后一条。循环体和非尾部分支末尾的调用只求值，不会让函数返回。
- `println(expr)` 打印字符串、整数或布尔值；打印变量时不移动变量。导入 `std/io` 或 `std/fmt` 后可使用相应的命名空间。
- 导入 `std/string` 后可用 `string.byte_len(&text) -> i64` 查询 UTF-8 字节数，`string.char_len(&text) -> i64` 按 UTF-8 码点计数（一个汉字算一个字符，续字节不计）；`string.equal(&a, &b) -> bool` 按字节比较内容，支持重复共享借用，不移动字符串；共享借用参数可直接转传。`&string` 参数接受共享借用，也接受字符串字面量与插值（借用一个只活到语句结束的临时字符串，例如 `string.split(&csv, ",")`）；不接受 owned 变量（要写 `&name`）或可变借用。示例：`zam run examples/values/strings.zm`。
- 文件、标准输入与命令行参数：`fs.read(&path) -> string!string`、`fs.write(&path, &text) -> !string`；`io.read_line(&mut line) -> bool!string`（去掉行尾 `\n`/`\r\n`，输入结束返回 `false`）、`io.read_all() -> string!string`；`env.arg_count() -> i64`、`env.arg(i) -> string!string`（不含程序名，`zam run <file> -- a b` 把 `--` 之后的参数传给程序）。字符串补充 `string.parse_i64(&s) -> i64!string`、`string.trim(&s) -> string`、`string.find(&s, &needle) -> i64`（找不到为 -1）、`string.byte_at(&s, i) -> i64` 与 `string.slice(&s, start, end) -> string`；后两者按字节计位置，越界或切在多字节字符中间时与数组越界一样终止程序。路径与参数按 UTF-8 处理，Windows 通过 `_wfopen` 与 `GetCommandLineW` 支持中文路径和参数。这些函数参与正常的类型、借用与错误值检查，C 实现只发射到用到它们的目标文件里。示例：`examples/io/wc.zm`（行、词、字节计数）、`examples/io/sum.zm`（逐行求和）、`examples/io/notes.zm`（写入、读回并拆分 `key=value`）。
- 可增长列表 `[T]`，元素为 `i64`、`bool` 或 `string`，可作局部变量、参数（`[T]` 移交所有权，`&[T]` / `&mut [T]` 调用帧借用）、返回值和结构体字段。空列表需类型标注（`let mut xs: [i64] = []`），不带标注的 `[1, 2]` 仍是定长数组。方法 `xs.len()`、`xs.push(v)`（字符串移入列表）、`xs.pop()`；`xs[i]` 读写元素，越界或空列表 `pop` 时终止程序。字符串元素不能经 `xs[i]` 移出，只能借用 `&xs[i]` 或 `string.copy(&xs[i])` 复制。`for x in &xs` 遍历：整数与布尔按值，字符串为只读借用；循环期间列表被冻结，不能修改、移动或独占借用。方法接收者只能是变量或字段路径（`bag.items.push(x)`），不支持链式调用；`a.b(...)` 中 `a` 是当前函数声明过的局部名时才是方法调用，否则按模块调用解析。配套标准库：`string.split/words/lines` 返回 `[string]`，`string.join(&list, &sep)`，`string.copy(&s)`，`env.args()`。C 侧每种元素类型生成一组 `L*` 结构与函数，只发射到用到它的目标文件。示例：`examples/collections/stats.zm`（读入整数、排序、统计）、`examples/collections/tac.zm`（倒序输出并找最长单词）。`xs.swap(i, j)` 原地交换两个元素，字符串列表靠它排序而不复制字符串。
- 字典 `{K: V}`：键为 `i64` 或 `string`，值为 `i64`、`bool` 或 `string`；可作局部、参数（`{K: V}` 移交，`&{K: V}` / `&mut {K: V}` 借用）、返回值和结构体字段。字面量 `{"amy": 30, "bob": 25}`（逗号分隔，可分行），空字典需类型标注 `let mut m: {string: i64} = {}`。`m[k]` 读取（键不存在时终止程序，`zam: key not found`），`m[k] = v` 插入或替换；字符串键以只读借用传入（`m[&name]`、`m["a"]`，或遍历得到的键 `m[k]`），新键插入时复制一份。方法 `len()`、`contains(k)`、`get(k, default)`（不存在时返回 `default`；字符串值返回副本）、`remove(k) -> bool`、`keys() -> [K]`。`for k in &m` 按插入顺序遍历键，期间字典被冻结。字符串值不能经 `m[k]` 移出，借用 `&m[k]` 或用 `get`。实现是插入顺序表加开放寻址下标表：遍历顺序确定、与哈希无关；删除留墓碑，重建下标表时压实。哈希不带随机种子，不适合直接处理不可信输入的大规模键集合。示例：`examples/collections/wordfreq.zm`（词频统计，按次数稳定排序）。
- 展示程序 `examples/md2html`：约 290 行、4 个模块的 Markdown 转 HTML 工具（标题、段落、有序/无序列表、引用、围栏代码块、分隔线，行内代码、粗体、强调、链接与 HTML 转义），`zam run examples/md2html -- examples/md2html/sample.md`；期望输出 `sample.html` 由 `zam/tests/showcase.rs` 核对。
- 字符串支持 UTF-8，以及 `\n`、`\r`、`\t`、`\"` 和 `\\` 转义。
- 字符串字面量支持插值：`"{name}"` 拼接当前作用域里的变量或字段（`{person.name}`），支持 `string`、`&string`、`&mut string`、`i64`、`bool`。插值只读取不移动，也不修改借用内容，插值后原变量仍可使用；字面大括号写作 `{{` 和 `}}`。花括号内目前只支持变量与字段路径，暂不支持调用、字面量和显式借用表达式。示例：`zam run examples/values/format.zm`。
- `if condition { ... } else { ... }`、`while condition { ... }`，条件必须是 `bool`；`for name in start..end { ... }` 遍历左闭右开的整数区间，`start` 与 `end` 进入循环前按顺序各求值一次，循环变量是只读 `i64`、只在循环体内可见（`zam run examples/control/loops.zm`）；支持 `&&`/`||` 短路求值。块内变量不能逃出块；支持 `else if` 链，复用嵌套分支的返回与所有权路径合并。支持 `break`/`continue`，跳转后同一代码块中的语句不可达，嵌套循环只影响当前循环；暂不支持遮蔽仍在作用域内的绑定。
- `struct` / `pub struct`，名称首字母大写，字段支持 `string`、`i64`、`bool` 和 owned 命名结构体，每行一个字段。用 `Person { ... }` 或 `model.Person { ... }` 按名初始化全部字段，读取整数/布尔字段，也可直接打印字符串字段（不移动）。支持可变结构体字段赋值、整体移动、整体重新赋值和字符串字段自动释放；替换字符串字段会先计算新值，再释放旧值。重复/缺失字段、借用字段和移动后使用会报错。支持 owned 结构体函数参数、返回值和变量类型标注（跨模块用 `model.Person`）；传参及返回转移整体所有权，未使用的返回值会自动释放。嵌套 owned 结构体支持构造、字段整体替换及递归释放；直接/间接递归布局会报错。支持连续字段访问（如 `a.b.c`）：读取标量、非移动打印字符串、替换可变根绑定下的字段；路径上的每一层均检查类型、字段名称和根绑定有效性。字符串字段支持调用帧借用（如 `&person.name`、`&mut person.name`，可嵌套），独占借用要求根绑定可变。冲突保守地按根结构体检查，不允许同时独占借用不同字段；借用字段时不能移动根结构体。暂不支持单独移出 owned 字段和整体结构体借用。示例：`zam run examples/structs`。
- 换行分隔语句；支持 `#` 和 `//` 单行注释。

### 所有权与借用

```zam
fn show(s: &string) { println(s) }
fn change(s: &mut string) { s = "changed" }
fn consume(s: string) { println(s) }
fn main() {
    let mut a = "hello"
    show(&a)
    change(&mut a)
    let b = a
    consume(b)
}
```

赋值和 owned 参数传递移动字符串；移动后使用会在 `check` 阶段报错。可变变量可以重新初始化。借用只能作为参数，调用返回即结束；共享借用允许多个，可变借用必须独占，且要求可变所有者。同一次调用中借用和移动同一变量会报错，嵌套调用也遵守外层已开始的借用。借用参数可以继续传给下层调用，但不能移动、返回或存进本地变量。字符串通过显式分配和作用域结束时的释放管理，不使用 GC/引用计数。

运行 `zam run examples/ownership` 查看完整示例。分支之后只允许使用所有继续路径都有效的字符串；返回分支不参与合并。循环回到下一轮前必须重新初始化已移动的外层字符串。检查保守地分析所有分支，不利用常量条件排除路径。`zam run examples/control/main.zm` 演示阶乘、GCD 和循环移动后重初始化；`zam run examples/control/jumps.zm` 演示嵌套循环中 `continue` 只跳到当前循环下一轮、`break` 只退出当前循环。`break`/`continue` 后同一代码块中的语句不可达，会在检查阶段报错。支持定长 `[i64; N]` 数组字面量、整数索引、可变元素赋值和运行时边界检查；长度限制为 1–1024，不能移动或借用；需要传递或增长的数据使用列表 `[T]`。当前尚不包含泛型、闭包、线程、unsafe 或 FFI，因此不能宣称已实现完整 Rust 级内存安全。原生程序没有解释器的步数限制；无限递归可能耗尽栈。

## 缓存行为

缓存默认放在用户目录 `.zam/cache`，可用 `ZAMAK_CACHE_DIR` 指定。模块键包含源码字节、依赖名称与递归依赖哈希、Zamak 编译器/IR 版本（当前 IR-16）、操作系统和架构，不含项目绝对路径。相同源码从不同目录构建共享模块产物。

修改依赖会使该模块和其传递依赖方失效，独立模块继续命中缓存。当前仍对全部源码执行解析和检查，并比较缓存产物内容；缓存验证以正确性为优先，尚未实现只读接口的语义增量分析。

性能验证的方法、适用范围与首轮结果见 [案例验证](docs/VALIDATION.md)，后续优化与 Rust/Go 对照见 [性能优化记录](docs/PERFORMANCE.md)。构建时间随模块规模、缓存状态、平台工具链和机器负载变化；运行性能还受整数溢出与边界检查影响。后端通过保守的整数区间分析消除可证明冗余的检查，并对有调用范围证明的同模块整数函数生成有限数量的内部版本；无法证明安全的操作保留运行时检查。GNU 风格后端使用 `-O2 -funroll-loops`，MSVC 使用 `/O2`；循环展开可能增加编译时间与代码体积，实测代价见性能记录。性能结论应与输入、检查语义和重复次数一起阅读。

Windows 工具链环境在首次调用时捕获并缓存，后续直接调用编译器与链接器。目标文件编译使用有界工作队列；默认并发度为可用核数且不超过 8，可通过 `ZAMAK_JOBS` 调整。缓存命中的模块不占用编译队列。

缓存发布过程为：同目录独占创建临时文件，写入并同步后，通过硬链接原子发布最终文件。并发构建可以重复计算，但同一缓存键只保留一个完整产物；不同内容或损坏缓存会报错。缓存所在文件系统必须支持硬链接。正常执行会清理临时文件；进程被强制终止可能留下 `.tmp` 文件。缓存清理和容量限制尚未提供子命令。

模块缓存保存确定性的 IR；原生后端为每个模块生成独立 C 和目标文件。目标文件键包含模块生成的 C（实现、实际调用函数的接口声明，以及本模块函数实际构造或实际使用函数签名中的结构体布局）、平台、架构、编译选项和本机工具链身份。无关结构体布局变更不改变目标文件或链接键；已使用布局变更会重新检查并使使用模块失效。仅修改被调用函数的实现时，调用方目标文件继续命中；修改实际使用的接口时，调用方失效或在类型检查阶段报错。函数符号由完整模块/函数名称稳定编码，增加无关函数不改变已有符号。

Windows 编译使用 `/Brepro` 与 `/d2pathmap` 将临时目录映射为固定路径，避免同一键的目标文件因构建目录不同而产生不同字节；并发回归会校验该条件。

链接键包含按模块排序的目标文件键、工具链身份和链接选项。目标文件与可执行文件都有 SHA-256 校验；输出分别显示 `OBJECT HIT/MISS` 和 `LINK HIT/MISS`。项目产物同样按内容命名和原子发布。IR 缓存仍使用递归源码依赖键，尚未实现语义检查缓存。`check` 不写缓存或构建产物。

## 验证

```powershell
cargo test --manifest-path zam/Cargo.toml --locked
cargo fmt --manifest-path zam/Cargo.toml -- --check
cargo clippy --manifest-path zam/Cargo.toml --all-targets --locked -- -D warnings
node editors/vscode/scripts/check-grammar.mjs
pwsh -File bench/ai/run.ps1 -ZamPath .\zam\target\debug\zam.exe -LatencyRepeats 3
pwsh -File tools/measure-build.ps1 -ZamPath .\zam\target\debug\zam.exe -Modules 6 -Repetitions 3
```

第 5 条校验编辑器扩展的声明式配置：三个 JSON 可解析、TextMate 语法里每条正则可编译、语言 id / 后缀 / `scopeName` / 仓库分组引用互相一致。

后两条是可重复的工程测量：离线 AI 闭环评测用固定候选和补丁验证诊断、修复、构建与输出是否全部通过；构建测量生成固定形状的多模块工程，断言冷构建全 MISS、热构建全 HIT、实现编辑只重编一个模块、接口编辑重编该模块与调用方。两者都只把结果写入被忽略的 `results/`，断言失败即报错。

集成测试覆盖原生执行、模块与缓存、并发发布、所有权与借用、错误处理、源码诊断，以及新建项目与格式化。测试同时包含成功路径和应被拒绝的输入。教程代码块会自动执行编译检查；常见算法案例另有原生运行与完整输出断言。测试方法与性能结论见 [案例验证](docs/VALIDATION.md)，开发规范见 [贡献指南](CONTRIBUTING.md)。

当前实现只是语言草案的一部分，多数功能还未实现；已在文档中标注的能力之外，语法与库都可能变化。

## 许可证

本项目按 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双许可发布，使用者可任选其一。除非另有声明，你提交的贡献同样按这两种许可证发布。
