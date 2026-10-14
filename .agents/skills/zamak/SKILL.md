---
name: zamak
description: 编写、运行、调试 Zamak（.zm）程序，或修改 Zamak 编译器本身（Rust 前端、C 后端、IR/缓存版本、VS Code 语法、测试与验收）。当任务涉及 .zm 源码、zam 命令、examples/ 示例、Zamak 语法/标准库/错误值，或 zam/ 下的编译器改动时使用。
---

# Zamak

Zamak 是静态类型、编译到原生可执行文件的小语言，支持 Windows x64、Linux、macOS 三个平台，仓库位置以当前工作目录为准（本环境为 `E:\CODE\Zamak`）。`zam` 就是它自己的编译器：Rust 前端（`zam/src/compiler.rs` 词法/语法/检查）→ C11 后端（`zam/src/native.rs`）→ Windows 默认 MSVC `cl`，Linux/macOS 用 `cc`（细节见「平台差异」）。生成的产物不需要安装 Zamak 或 Rust。

## 命令

- `zam check [project|file]`、`zam build ...`、`zam run ...`、`zam fmt ...`、`zam new <directory>`、`zam demo`；无路径参数时用当前目录；源码后缀统一 `.zm`，可直接对单个源码文件操作；`run` 也能直接执行已构建的产物；`zam run <file> -- a b` 把 `--` 之后的参数原样交给程序（只有 `run` 接受 `--`）。
- `zam new <directory>` 生成最小骨架（`zam.toml` + 问候用的 `src/main.zm` + 忽略 `build/` 的 `.gitignore`），列出文件到 stderr，stdout 保持空；目录已存在就报错且不覆盖，目录名必须是合法标识符（写进 `zam.toml` 的 `name`）。
- `zam fmt` 只规范化空白：按大括号深度把每行缩进改成 4 空格的倍数、去行尾空白、结尾留一个换行、保留原换行风格（CRLF 仍是 CRLF）。它不重建 token，所以注释和字符串内容不会丢；改过的文件路径以 `FORMAT <path>` 打到 stderr，已规范的文件不重写。语法坏掉的代码先被拒绝（走 `check` 同一套解析）。不做行内空格重排与折行。
- `zam --version` / `--help`；缓存目录默认是用户主目录下的 `.zam/cache`，可用环境变量 `ZAMAK_CACHE_DIR` 覆盖。`check` 在任何平台都不需要 C 编译器。
- C 编译按模块并行，只有缓存 MISS 的模块进编译队列，默认并发度为 `min(可用核数, 8)`，可用环境变量 `ZAMAK_JOBS` 调整（设 `1` 即串行）。并行时 `OBJECT ...` 进度行的顺序不确定，解析进度请按行前缀匹配而不要依赖顺序。
- `zam check --json` / `zam build --json` 在 stdout 输出一行机器可读诊断：`{"command","ok","diagnostics":[{file,line,column,function,code,category,message,suggestion}]}`。`code`（如 `ZM100`）与 `category` 是稳定机器字段，分支判断只依赖它们；`message` 是人类可读文本，`suggestion` 在有修复路径时给出（`ZM999` 无建议，值为 `null`）；`file`/`line`/`column`/`function` 无法定位时为 `null`。**词法/语法/模块解析错误只报第一条；之后类型与所有权检查逐个函数进行，每个出错的函数各报它的第一条，`diagnostics` 按文件、行、列排序，可能有多项。** 一轮可以修完各函数的问题再检查；同一函数里的后续错误要等前一条修好才出现。 失败时 stderr 仍有一行人类可读信息、退出码非零；其他子命令不接受 `--json`。
- 构建进度（`CACHE`/`OBJECT`/`LINK`/`TIMING`/`OUTPUT`）与错误都写 **stderr**，stdout 只有程序输出或 `--json` 的那一行；`TIMING` 一行给出 `check_ms`/`key_ms`/`object_ms`/`object_prepare_ms`/`link_ms`/`total_ms`；`OUTPUT <path>` 的产物名按平台不同（见下表）。命令失败返回非零码。

## 平台差异（Windows / Linux / macOS）

| | Windows x64 | Linux | macOS |
|---|---|---|---|
| 默认 C 工具链 | MSVC：`vswhere` 找带 `VC.Tools.x86.x64` 组件的 VS / Build Tools，直接调 `Hostx64\x64` 下的 `cl.exe`/`link.exe`；找不到就依次用 PATH 上的 `gcc`、`clang`（本机有 MinGW gcc：`C:\ProgramData\mingw64\mingw64\bin`） | 依次 `cc`、`clang`、`gcc`（例如装 `build-essential`） | 同 Linux（Xcode Command Line Tools 的 Apple clang，`xcode-select --install`） |
| 换编译器 | 设 `CC=gcc`（MinGW）改走 GNU 后端 | 设 `CC=clang` 等 | 同 Linux |
| 产物 | `build/<name>-<hash>.exe` | `build/<name>-<hash>`，无后缀，权限 0755 | 同 Linux |
| 同键同字节 | `.obj` + `/Brepro` + `/d2pathmap` | `.o` + `-ffile-prefix-map=<临时目录>=.` | 同 Linux |
| 缓存目录 | `%USERPROFILE%\.zam\cache` | `$HOME/.zam/cache` | 同 Linux |
| 额外缓存 | MSVC 环境一次性捕获到 `%TEMP%\zam-msvc-<hash>.txt`，按 `vcvars64.bat` 的大小和修改时间失效 | 无 | 无 |
| 验证 | 本机 + CI `windows-latest` | 本机 WSL2（见下）+ CI `ubuntu-latest` | 只有 CI `macos-latest`，本机无法验证 |

- `CC` 必须是单个 GNU 风格的编译器程序：不能带参数、不能是 MSVC `cl`；在 Windows 上也不能是 `.bat`（`.bat` 会被当成 MSVC 的 vcvars 脚本）。要加 sanitizer 之类的参数，就写一个包装脚本再把 `CC` 指过去（Unix）。
- 缓存键包含 `OS` 与 `ARCH`，同一缓存目录被多个平台共用时不会混用产物。
- 选择逻辑在 `native.rs` 的 `select()`（`CC` → Windows 的 `msvc()` → PATH 回退），单元测试 `toolchain_selection_prefers_cc_then_msvc_then_path_fallbacks` 覆盖顺序；本机模拟「没有 Visual Studio」：把环境变量 `ProgramFiles(x86)` 指向空目录（`tests/toolchain.rs` 就是这么做的）。
- **本机 Linux 验证**：WSL 发行版 `Ubuntu`（24.04，默认用户 root）已装 gcc 13.3、clang 18.1、Rust 1.99（含 clippy/rustfmt）。不要直接在 `/mnt/e/...` 上跑：Windows 检出是 CRLF（与 CI 的 LF 不同），而且会和 Windows 共用 `zam/target`。`tools/wsl-verify.sh` 把仓库 clone 到 WSL 自己的文件系统，套上 Windows 侧用 `git diff --output` 导出的未提交补丁（不要用 shell 重定向，pwsh 可能改编码），依次跑 fmt、gcc 与 `CC=clang` 两套全量测试、clippy，再用 ASan/UBSan/LeakSanitizer 跑 `SANITIZE_FILES` 里的示例（正常路径，参数是一个存在的文件、stdin 是几行整数）和 `SANITIZE_FAILING` 里的示例（参数指向不存在的文件，覆盖错误冒出 `main` 的路径）。在仓库根目录用 pwsh 执行：

  ```powershell
  git diff HEAD --output=zam/target/wsl.patch
  wsl -d Ubuntu -u root -- env SANITIZE_FILES="examples/io/wc.zm examples/control/loops.zm" SANITIZE_FAILING="examples/io/wc.zm" bash tools/wsl-verify.sh
  ```

  每行输出 `passed=N failed=0`、`CLIPPY_OK`、`... sanitizer_reports=0` 才算通过。新增的未跟踪文件不会进 `git diff HEAD`，先 `git add -N <文件>` 再导出。`.sh` 由 `.gitattributes` 固定为 LF，否则 Windows 检出的 CRLF 会让 bash 报 `$'\r'`。
- 给用户的命令按对方的 shell 写，环境变量写法不同：

| 操作 | pwsh（Windows 默认） | bash / zsh（Linux、macOS） |
|---|---|---|
| 单次设变量 | `$env:ZAMAK_JOBS='1'; zam build` | `ZAMAK_JOBS=1 zam build` |
| 换编译器 | `$env:CC='gcc'; zam build` | `CC=clang zam build` |
| 取消变量 | `Remove-Item Env:ZAMAK_JOBS` | `unset ZAMAK_JOBS` |

  注意 pwsh 的 `$env:X=...` 会留在当前会话里，后续命令都受影响，用完要取消；bash 的前缀写法只作用于这一条命令。

## 语法速览（动手前先看 `examples/`）

- 函数：`fn f(a: i64, s: &string) -> i64 { ... }`；函数尾部的值表达式即返回值（函数体最后一条，或尾部 if/else 分支的最后一条），也可写 `return`；**循环体和非尾部分支末尾的调用只求值、不返回**；跨模块可见要写 `pub fn`。
- 绑定：`let x = 1`、`let mut x = 1`、`let mut x: i64 = 1`。**不支持遮蔽**（同一作用域重名报错）；块内变量不能逃出块。
- 类型：`i64`、`f64`、`bool`、`string`、命名结构体、`[i64; N]`（定长，长度 1–1024，只能是局部值）、`[T]`（可增长列表，`T` 只能是 `i64`/`f64`/`bool`/`string`，可作局部、参数、返回值、结构体字段）、`{K: V}`（字典，`K` 是 `i64`/`string`，`V` 是 `i64`/`f64`/`bool`/`string`，用法同列表）。
- 浮点 `f64`：字面量 `1.5`、`2e-3`、`1_000.5`（小数点两边都要有数字）；IEEE 754，`+ - * / %` 与比较不检查溢出，除零得 `inf`/`NaN`。**`i64` 与 `f64` 不隐式转换**：`x * 2` 要写 `x * 2.0`，变量用 `f64(n)` / `i64(x)`（向零截断，NaN 或越界终止程序）。`println`/插值输出最短可读回十进制（`0.1`、`2.0`、`1e16`、`2.5e-7`、`NaN`、`inf`）；`fmt.fixed(x, n)`（`use std/fmt`）保留 n 位小数；`string.parse_f64(&s) -> f64!string`。字典键不能是 `f64`。
- 列表：空列表**必须标注类型** `let mut xs: [i64] = []`（不带标注的 `[1, 2]` 是定长数组）；`xs.len()`、`xs.push(v)`（字符串移入）、`xs.pop()`；`xs[i]` 读写（越界或空 `pop` 终止程序）；字符串元素**不能** `xs[i]` 取出，借用 `&xs[i]` 或 `string.copy(&xs[i])`；`for x in &xs` 遍历（字符串元素是只读借用），循环体内不能修改/移动 `xs`，要改就用 `for i in 0..xs.len()`；参数 `&[T]` / `&mut [T]`，传 `&xs` / `&mut xs`；`xs.swap(i, j)` 原地交换（字符串列表排序靠它）；方法接收者只能是变量或字段路径，**不能链式调用** `f().len()`。
- 字典：字面量 `{"a": 1, "b": 2}`（逗号分隔，可分行），空字典**必须标注类型** `let mut m: {string: i64} = {}`；`m[k]` 读（键不存在**终止程序**）、`m[k] = v` 插入或替换；字符串键按只读借用传：`m[&name]`、`m["lit"]`、遍历得到的键直接 `m[k]`（owned 变量不加 `&` 会报 `expected explicit borrow`）；`m.len()`、`m.contains(k)`、`m.get(k, default)`（缺失返回 default，字符串值返回副本）、`m.remove(k) -> bool`、`m.keys() -> [K]`；`for k in &m` 按**插入顺序**遍历键，循环体内不能改 `m`；字符串值不能 `m[k]` 取出，用 `&m[k]` 或 `get`。计数惯用法：`counts[word] = counts.get(word, 0) + 1`。
- 结构体：`struct P { name: string` 换行 `age: i64 }`（每行一个字段，无逗号）；字面量 `P { name: "a" age: 3 }`（**字段之间没有逗号**，可写一行也可分行）；字段读写 `p.name`、连续 `a.b.c`；支持整体移动、整体重新赋值、可变字段赋值；字符串字段自动释放。
- 控制：`if`/`else`/`else if`、`while`、`for i in start..end`、`break`、`continue`；条件必须是 `bool`。`for` 只遍历整数区间（左闭右开，`start`/`end` 进入循环前各求值一次，循环变量只读且不能与已有变量同名）；`for x in &list` 遍历列表；定长数组按下标写 `for i in 0..N`。
- 表达式：`+ - * / %`、`== != < <= > >=`、`&&`、`||`、前缀 `!expr`、一元 `-expr`、括号；复合赋值 `+= -= *= /= %=`（变量、字段、元素；元素下标里不能有调用）；if 表达式 `let v = if c { a } else { b }`（必须有 else，每个分支一个表达式，不能出现在插值里）；字符串 `a + b` 拼接（两边都不移动）与 `== != < <= > >=` 比较，字符串不能和整数相加；整数可用数字之间的单个 `_`（`1_000_000`）。溢出/除零在运行时返回错误。
- 字符字面量 `'a'`、`'#'`、`'\n'`、`'\''`：值是 Unicode 码点（`i64`），与 `string.byte_at` 比较只用 ASCII 字符。
- 字符串：UTF-8，转义 `\n \r \t \" \\`，插值 `"{name}"` / `"{p.name}"` / `"{a + b}"` / `"{xs[i]}"` / `"{f(x)}"`（变量、字段、元素只读不移动；花括号里不能有字符串字面量或花括号），字面大括号写 `{{` / `}}`。
- 注释：`//` 与 `#`；**没有块注释**。
- 模块：`use greeting`（同目录 `greeting.zm`，只能调用其 `pub fn`）、`use std/string`、`use std/io`、`use std/fmt`、`use std/fs`、`use std/env`、`use std/math`；调用 `greeting.hello()`、`io.println("x")`；`println(...)` 无需导入。
- 所有权：字符串赋值/传参是**移动**，移动后再用报错；借用只能作为参数（`&s`、`&mut s`），调用返回即结束；共享借用可多个，可变借用必须独占且要求可变所有者；借用参数不能移动、返回或存进局部变量。整数/布尔是拷贝。
- 错误值：签名 `fn load(p: &string) -> i64!string`（成功 T / 失败 E）或 `-> !string`（只失败）；`fail "msg"` 产生错误并立即返回；`let v = load(&p)?` 失败时向调用方传播（当前函数签名必须带相同的 `!E`）；`load(&p) catch e { ... }` 就地处理（**不传播**，错误值只在块内可见，**块必须在所有路径 `return`**）；`fn main() -> !string` 允许，未捕获错误写 stderr 并以退出码 1 结束。当前只支持 `E = string`。
- 标准库（`&string` 参数传 `&name`、已是借用的变量，或直接写字符串字面量/插值 `"x"`、`"{n}"`；owned 变量必须加 `&`）：
  - `std/string`：`byte_len(&s) -> i64`、`char_len(&s) -> i64`（按 UTF-8 码点）、`equal(&a, &b) -> bool`、`parse_i64(&s) -> i64!string`（严格十进制，可带 `+`/`-`，不允许空白）、`parse_f64(&s) -> f64!string`（十进制，可带小数点与指数，不接受空白、`inf`、`nan`）、`trim(&s) -> string`、`find(&s, &needle) -> i64`（字节位置，找不到 -1）、`byte_at(&s, i) -> i64`、`slice(&s, start, end) -> string`（字节区间，越界或切在多字节字符中间会**终止程序**，与数组越界一样）、`find_from(&s, &needle, start) -> i64`、`starts_with`/`ends_with`/`contains(&s, &p) -> bool`、`replace(&s, &from, &to) -> string`、`lower`/`upper(&s) -> string`（仅 ASCII）、`copy(&s) -> string`、`split(&s, &sep) -> [string]`（保留空段）、`words(&s) -> [string]`、`lines(&s) -> [string]`（去 `\r`，末尾换行不产生空行）、`join(&list, &sep) -> string`。
  - `std/io`：`print(&s)`（不换行）、`eprintln(&s)`（写 stderr）、`read_line(&mut line) -> bool!string`（去掉行尾 `\n`/`\r\n`，结束时返回 false；典型写法 `let mut line = ""` + `while io.read_line(&mut line)? { ... }`）、`read_all() -> string!string`。
  - `std/fs`：`read(&path) -> string!string`、`write(&path, &text) -> !string`（覆盖写）。
  - `std/env`：`arg_count() -> i64`、`arg(i) -> string!string`（不含程序名，从 0 开始）、`args() -> [string]`。
  - `std/math`（参数都是 `f64`）：`sqrt`、`pow(x, y)`、`exp`、`log`（自然对数）、`log10`、`sin`、`cos`、`tan`、`atan`、`atan2(y, x)`、`floor`、`ceil`、`round`（远离零）、`trunc`、`abs`、`min(a, b)`、`max(a, b)`、`is_nan(x) -> bool`；常量 `math.pi`、`math.e`、`math.inf`（`project.rs` 解析时替换成字面量）。
  - `std/fmt`：`fixed(x, digits) -> string`（digits 0–30，超出终止程序）；`println` 同全局。
  - `println` / `io.println` / `fmt.println` 打印字符串、整数或布尔值。
  - 可失败函数用 `?` 时当前函数要带 `!string`，所以读写文件的 `main` 一般写成 `fn main() -> !string`；`fs.write(&p, &t)?` 可以单独成一条语句。
  - 标准库实现：`zam/src/native.rs` 的 `BUILTINS` 表（签名 + C 函数体，`p0`/`p1`… 是参数）和 `STD_PRELUDE`（共用 C 辅助）；`project.rs` 把 `std/*` 调用解析成 `std/<模块>:<函数>`，检查器按普通函数签名检查。新增函数只需在表里加一项，再升 `native-c-N`。列表的 C 侧是每种元素一组 `Li`/`Lf`/`Lb`/`Ls` 结构与 `push/pop/at/len/swap/drop/take` 函数（`list_helpers`），字典是 `M<键><值>`（如 `Msi`）一组 `set/at/has/getor/del/keys/len` 与插入顺序表 + 开放寻址下标表（`MAP_TEMPLATE`）。用到哪些容器由生成器记录在 `Generator.containers`（每个 `scalar()` 都登记类型）加上签名和结构体字段，函数体先生成、容器辅助后拼在前面。容器方法签名只在 `compiler::method_signature` 一处定义，检查器与生成器共用。
- **尚未实现，不要写**：泛型、闭包、枚举、`match`、结构体元素的列表/字典、嵌套容器、`for k, v in &m`、方法链式调用、`for x in &mut xs`、`i64`/`f64` 隐式转换、整数版 `math.abs/min/max`、元组、目录遍历、追加写文件、网络、时间、包管理器、调试器、REPL、文档生成、线程、unsafe/FFI、块注释、`string` 以外的错误类型、整体结构体借用、单独移出 owned 字段。

## 常见诊断（照文本改代码）

- `fallible call f must be handled with ?` → 调用处漏了 `?` 或 `catch`。
- `? cannot propagate the error of f from this function` → 当前函数签名缺少 `!E`，或错误类型不同。
- `fail requires a function that declares an error type` → 在非 `!E` 函数里用了 `fail`。
- `catch block for f must return a value` → catch 代码块要在所有路径上 `return`。
- `only string error types are supported for now` → 错误类型只能是 `string`。
- 移动后使用、借用冲突/逃逸、不可变赋值、重复/缺失字段等都在 `zam check` 阶段报错。
- `an empty list literal needs a type annotation` → 给 `[]` 加类型；`cannot move a string out of xs[..]` → 用 `&xs[i]` 或 `string.copy`；`xs is borrowed by an enclosing for loop` → 循环里改了正在遍历的列表或字典；`an empty map literal needs a type annotation` → 给 `{}` 加类型。
- `type mismatch: * mixes i64 and f64` / `x is i64 but f64 is expected` → 整数字面量写成 `2.0`，变量用 `f64(n)` 或 `i64(x)` 转换；`zam: f64 value out of i64 range` → `i64(x)` 碰到 NaN 或越界。
- 运行时（stderr，退出码 1）：`zam: list index out of bounds`、`zam: pop from empty list`、`zam: key not found` 直接终止；`cannot open <path>`、`invalid integer: <text>`、`argument index out of range` 是错误值，可 `catch`；`zam: string index out of bounds`、`zam: array index out of bounds`、`zam: integer arithmetic error` 是直接终止。
- 人类可读诊断（stderr）：`error[ZM301]: <信息>`，下一行 `--> <文件>:<行>:<列>`，然后是源码行和 `^` 下划线（词法/语法错误只划记号，语义错误从语句起点划到行尾；中文按 2 格对齐），最后 `= help: <建议>`；多条之间空一行，结尾 `zam: N errors`。没有位置的错误仍是一行 `zam: <信息>`。列从 1 开始、按 Unicode 字符计数。**程序化处理一律用 `--json`**，别解析这段文本。
- 实现：编译器内部错误仍是字符串 `line L, column C: 模块:函数: 信息`，多条用 `compiler::DIAGNOSTIC_SEPARATOR`（`\u{1e}`）拼接；`main.rs` 的 `diagnostics()` 拆开排序，`diagnostic()` 反解字段，`render()`/`underline()` 渲染。

## 修改 Zamak 编译器本身

1. 验收（工作目录 `zam/`）：`cargo test --locked`（20 个集成套件，Windows 冷跑约 30 秒；MSVC 环境捕获会缓存在临时目录，首次多付约 4 秒）、`cargo clippy --all-targets --locked -- -D warnings`、`cargo fmt --check`；改完源码要在**仓库根目录**重新 `cargo install --path zam --locked` 才能用新的 `zam`。CI（`.github/workflows/ci.yml`）在 `ubuntu-latest`/`windows-latest`/`macos-latest` 三个平台跑同样三项；AI 评测（`bench/ai/run.ps1`）和构建体积测量（`tools/measure-build.ps1`）只在 Windows 上用 pwsh 跑。**格式以 CI 的最新稳定版 rustfmt 为准**：本机 Windows 与 WSL 截至 2026-10-14 都是 Rust 1.99，与 CI 一致；CI 升级后若本机落后，不同版本对长 `match` 臂的排版可能不同，先 `rustup update stable`，或以 `tools/wsl-verify.sh` 的 `FMT_OK` 为准。
   - 版本号：IR 编码（`Module`/`Function`/`Expr`/`Instruction` 的 Debug 输出）一变就要升 `native-c-N`，否则同一个缓存键会撞上旧内容，报 `corrupt cache entry ...zmo`（这是缓存在拦截，不是坏了）。
2. 新增语言构造时按惯例升版本标记：`zam/src/compiler.rs` 的 encode 头 `ZAM-IR-N` 与 `zam/src/project.rs` 的 `VERSION`（`native-c-M`）。两者都参与缓存键，只作失效标记——缓存命中时按字节全等校验，没有 decode，旧产物只会报错不会静默错译；同时更新 `README.md` 里的 IR 号。
3. C 后端源码按 UTF-8 写出，MSVC `cl` 默认按本地代码页（936）读取，会吞掉中文注释后的下一行 → 编译命令行必须保留 `/utf-8`。`zam/src/native.rs` 的 `OPTIONS`（`-vN`）是 MSVC 与 GNU 两套命令行**共用**的一个版本标记，改动任一边（`compile`/`link` 里的 `/O2 /MT ...` 或 `-std=c11 -O2 -funroll-loops ...`）都要考虑升它——但只在**产物字节真的变了**时升版：若只是改了调用方式（例如从 `cmd /c call vcvars64.bat && cl` 改成直接调 `cl.exe`），先用旧缓存验证仍全部 `OBJECT HIT`/`LINK HIT` 再决定不升，避免无谓的缓存失效。
4. 跨平台约束：
   - 平台分支只放在 `native.rs`（`tool`/`uses_msvc`/`compile`/`link`/MSVC 环境捕获）和 `project.rs`（`cache_dir`、产物 `.exe` 后缀与 Unix 0755 权限），用 `cfg!(windows)` / `#[cfg(unix)]`，不要在前端或检查器里分平台。
   - 生成的 C 必须同时能被 MSVC 和 GCC/Clang 编译：GCC/Clang 专用写法（如 `__builtin_add_overflow`）放在 `#if defined(__GNUC__) || defined(__clang__)` 里并给 MSVC 留等价分支，见 `native.rs` 开头的运行时前导。
   - 测试和示例不要写死 `.exe`、`\` 路径分隔符或 CRLF；产物路径从 stderr 的 `OUTPUT` 行取。Windows 检出可能把示例转成 CRLF（git `autocrlf`），断言输出时不要依赖源码的换行风格。
   - macOS 本机无法验证，涉及 `cc` 命令行、链接或目标文件后缀的改动，提交前说明「macOS 待 CI 确认」，推送后看 GitHub Actions 结果。
5. 示例放 `examples/<主题>/*.zm`（多模块项目为 `<主题>/src/main.zm` + `zam.toml`），并在 `zam/tests/<主题>.rs` 里用 `include_str!(...)` 引用示例做正例与反例，真正运行产物并断言 stdout/stderr/退出码。
6. 文档边界：`README.md`（英文，GitHub 首页）与 `README.zh-CN.md`（中文，更详细）都入库，改能力范围时两份都要同步；`SYNTAX.md`、`ROADMAP.md`、`DESIGN.md`、`NAMING.md`、`MIGRATION.md` 被 `.gitignore` 忽略，只在本地维护。二者都要同步（README 如实标注缺口，ROADMAP 记进度）。
7. 提交：分支 `main`，信息用 `<type>:中文描述`（feat/fix/test/docs/chore），按主题分批，必要时 `git push origin main`。
8. 编辑器/AI 生态：`editors/vscode/` 是纯 TextMate 语法扩展（零运行时依赖），改动关键字、符号或类型时要同步 `editors/vscode/syntaxes/zamak.tmLanguage.json` 与 `language-configuration.json`。

## 参考资料

`README.md` / `README.zh-CN.md`（能力与缺口）、`TUTORIAL.md`（零基础教程，入库）、`SYNTAX.md`（语法细节）、`examples/`（可运行示例）、`ROADMAP.md`（当前进度的权威记录）、`zam/src/compiler.rs`（前端）、`zam/src/native.rs`（C 后端）、`zam/src/project.rs`（构建与缓存）。
