<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/logo-dark.svg"/>
    <img src="docs/logo-light.svg" alt="Zamak" width="520"/>
  </picture>
</p>

<p align="center">
  <a href="./README.md">English</a>
  &nbsp;·&nbsp;
  <strong>简体中文</strong>
  &nbsp;·&nbsp;
  <a href="#为什么选择-zamak">为什么选择 Zamak</a>
  &nbsp;·&nbsp;
  <a href="https://zamak.veyralux.org/zh-CN/">语言手册</a>
  &nbsp;·&nbsp;
  <a href="./PREVIEW.md">预览版安装</a>
  &nbsp;·&nbsp;
  <a href="./CONTRIBUTING.md">参与开发</a>
  &nbsp;·&nbsp;
  <a href="./docs/AI_WORKFLOW.md">AI 工作流</a>
  &nbsp;·&nbsp;
  <a href="https://github.com/VeyraLux/zamak-index">包索引</a>
</p>

<p align="center">
  <a href="https://github.com/VeyraLux/Zamak/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/VeyraLux/Zamak/ci.yml?style=flat-square&label=ci&labelColor=161b22&logo=githubactions&logoColor=white" alt="CI"/></a>
  <a href="#许可证"><img src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-8b949e.svg?style=flat-square&labelColor=161b22" alt="许可证：MIT 或 Apache-2.0"/></a>
  <img src="https://img.shields.io/badge/version-0.1.1-f97362.svg?style=flat-square&labelColor=161b22" alt="版本 0.1.1"/>
  <img src="https://img.shields.io/badge/platforms-Windows%20%C2%B7%20Linux%20%C2%B7%20macOS-6366f1.svg?style=flat-square&labelColor=161b22" alt="平台：Windows、Linux、macOS"/>
  <a href="https://github.com/VeyraLux/Zamak/stargazers"><img src="https://img.shields.io/github/stars/VeyraLux/Zamak.svg?style=flat-square&color=06b6d4&labelColor=161b22&logo=github&logoColor=white" alt="GitHub stars"/></a>
</p>

<br/>

<p align="center"><strong>开源 · 原生代码 · 无需垃圾回收的内存管理</strong></p>
<h3 align="center">以清晰书写，以精确编译。</h3>
<p align="center">Zamak 是一门为人与 AI 共同书写而设计的静态类型语言。语法轻盈而熟悉，编译产物是独立运行的原生程序；所有权模型无需垃圾回收即可管理内存，每一个错误都附有稳定的错误码和可直接采纳的修复建议。</p>
<p align="center">作者 <strong>rayL-K</strong> · <strong>VeyraLux</strong></p>

<br/>

## 为什么选择 Zamak

**像 Python 一样好读，像 C 一样快，无GC也安全。**

<p align="center">
  <img src="docs/demo.svg" alt="小 Ƶ 把一段 Python 函数改成 Zamak，然后运行，打印出 13.5" width="720"/>
</p>

| 与 C 一样快 | Linux 上 hello world 只有 16 KB | Java 用 365 MB 的程序它只用 20 MB | 不写生命周期标注 | 发布版也检查整数溢出 |
| :---: | :---: | :---: | :---: | :---: |

### 为什么做这门语言

Zamak 诞生于一种新的工作方式：多个 AI 助手在多个工作树里并行修改同一个项目，每个工作树一天要检查、构建、测试成百上千次。在这种节奏下，编译要多久、产物多大、报错机器能不能读懂，就从「体验问题」变成了「生产力问题」。

所以 Zamak 的取舍和别的语言不太一样，冲突时按这个顺序决定：

1. **构建成本**——一次完整循环（读 → 改 → check → build → test → 读报错）的总耗时 × 每天的次数，加上磁盘占用
2. **内存安全**——默认安全，不安全代码需要显式标记
3. **运行性能**——进 Rust 那一档就够，不追求超越
4. **表达力**——易读性对标 Python，省略噪音但不省略安全

这也是为什么宏系统、依赖类型、需要看到整个程序才能定论的特性都不在计划里：它们会拖慢构建，而构建速度是这门语言的第一原则。泛型也默认不做单态化，不为每个实例生成一份代码。

### 借了什么

Zamak 没想当哪门语言的影子，是把几门语言各自最擅长的部分拼在一起：

| 来源 | 想要的 | 在 Zamak 里的样子 |
| --- | --- | --- |
| Rust | 内存安全、无GC | 所有权加调用帧借用：值默认移动，借用不越过当前函数，不写生命周期标注 |
| Go | 编译快、交付一个独立文件 | 编译单元独立、按内容缓存，一个 `zam` 命令管检查、构建、测试、格式化 |
| C | 体积小、运行快、能调用现有代码 | 生成 C 再交给 MSVC / GCC / Clang，没有运行时；`extern fn` 直接调 C 函数 |
| Python | 易读、写过的人不用重新学 | 缩进分块、`len`、`range`、`in`、切片与 Python 的除法规则 |

这些特点集中在同一门语言里。下面的数字都来自你可以自己复现的测试，见[与其他语言对比](#与其他语言对比)和[特点](#特点)。

- **Python 的可读性，C 的速度**：Zamak 读起来像 Python，`for i in range(n)`、`xs[-1]`、`f"..."` 和除法规则都与 Python 相同。它编译成本机代码：在 binary-trees、fannkuch-redux、素数统计和词频统计上比 C 快，spectral-norm 只慢约 1%，计算密集的程序比 Python 快 43 到 204 倍。
- **不用和编译器较劲的内存安全**：所有权在编译时就排除了释放后使用和重复释放，不需要垃圾回收器。只读借用自动进行，借用最多持续一个代码块，永远不用写生命周期标注。代码真被拒绝时，错误信息会指出问题所在，并给出修改建议。
- **整数运算比 C、Rust、Go 更安全**：整数溢出、除以零和下标越界在任何构建里（包括发布构建）都会让程序带着清楚的错误停下。编译器会去掉能证明多余的检查，所以安全的代码照样跑出 C 的速度；哈希这类确实需要回绕的函数，用 `@wrapping` 只在这一个函数里关闭检查。
- **程序小，占用也小**：Linux 上 hello world 只有 16 KB，与 C 相当；同一个程序用 Rust 写，即使 strip 加 LTO 也有 275 KB。Windows 上两边都会因为静态链接 C 运行库变大——Zamak 默认 MSVC 约 143 KB，Rust 压到极限约 106 KB，属于同一量级。在 binary-trees 上，Zamak 是七种语言里最快的，只用 20 MB 内存，而 Java 和 Node.js 都超过 360 MB。
- **为 AI 编程助手而设计**：整门语言写得进一页参考文档，AI 写代码前读一遍就够了。每条诊断都有稳定的错误码和修改建议，也能以 JSON 输出；`zam new --template ai` 会为你的 AI 助手配好 Skill 文件，`zam mcp` 让 AI 助手通过 MCP 直接检查、运行、测试和查文档。
- **反馈快**：在 5,000 行的项目上，`zam check` 不到 0.1 秒就能完成；改一行代码，重新构建不到 1 秒，因为 `zam` 按内容缓存每个模块。

### 与热门语言相比

每一门热门语言都要你放弃一些东西，而 Zamak 的设计目标就是让你不必取舍：

- **Python** 好读好写，但运行慢，每台机器都要装解释器，类型错误要等代码跑起来才发现。
- **C** 快而小，但一个下标写错或一次整数溢出，就会悄无声息地破坏内存。
- **Rust** 快而且内存安全，但生命周期和借用检查要花很长时间才能掌握，发布构建也不检查整数溢出。
- **Go** 简单安全，但自带垃圾回收器，一个 hello world 就有 2.3 MB。
- **Java** 和 **Node.js** 需要庞大的运行时，频繁分配内存的程序要占用几百 MB。

### 什么时候该选别的语言

Zamak 还很年轻，目前不支持多线程和网络，包生态也刚刚起步。生产环境的服务、大型存量代码、依赖成熟库的工作，眼下请用成熟的语言；命令行工具、数据处理、学习，以及交给 AI 编写的代码，可以试试 Zamak。

## 示例

统计输入里每个词出现的次数：

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

Zamak 是为频繁构建和多 worktree 协作设计的语言，改完代码，人和 AI 都能很快拿到检查结果。`zam` 先生成 C，再调用 MSVC、GCC 或 Clang 编译成机器码；类型与所有权由 Zamak 自己检查，生成的程序可以独立运行，不需要安装 Zamak 或 Rust。

## 特点

- **熟悉的语法**：写过 Python 的人，读 Zamak 不需要适应。`len(xs)`、`for i in range(n)`、`x in xs`、`a if c else b`、`xs[-1]`、`xs[1:3]`、`sum(xs)`、`sorted(xs)`、`int(s)`、`str(n)`、`2 ** 10`、`f"..."` 和 `#` 注释的意思都与 Python 相同；除法也一样（`7 / 2` 是 `3.5`，`7 // 2` 是 `3`），字符串按字符计（`len("你好")` 是 2）。
- **构建与缓存**：`zam check` 只做类型与所有权检查，不调用 C 编译器。同一份源码可以在不同目录和工作树之间共享缓存；改了实现只重编受影响的原生模块，改了被调用的函数接口或结构体布局，调用方随之失效。
- **类型与所有权**：静态类型检查覆盖变量、函数、结构体、列表和字典。字符串、结构体、列表和字典赋值时移动；变量传给只读借用参数时自动借用，`&mut` 要显式写；借用只在一次调用内有效，不需要写生命周期标注。整数溢出、除零和越界在运行时检查，保守的区间分析能证明多余的检查会被去掉。哈希这类本来就要回绕的热点函数，可以在 `fn` 上一行写 `@wrapping`，只在这个函数里关闭溢出检查。
- **诊断**：每条错误带稳定的错误码、位置和修改建议，`--json` 把它们写成一行 JSON 给工具读取。
- **原生执行**：生成的程序独立运行，无需安装 Zamak 或 Rust。

在 3 个模型、8 个小型命令行任务的对比实验里，模型只读了一页语言参考：2026-10-18 重测时 24 个 Zamak 程序全部解出，其中 19 个第一次就通过。没一次通过的那些里，除了一个把 `{{` 写成了插值，其余全是漏写 `use std/string`——报错会指名要加的那一行，第二次都通过了。同样的任务 Rust 第一次通过 23 个、Go 24 个，这些任务太简单，分不出三种语言的高下。方法与数据见 [bench/ai/live](bench/ai/live/README.md)。

当前版本适合学习、语言实验、算法练习和命令行小工具。标准库与工具生态还不完整，不适合生产使用。

### 生态现状

| 组件 | 当前能力 |
| --- | --- |
| 命令行 | `check`、`build`、`run`、`test`、`fmt`、`new`、`add`、`update`、`demo`；检查与构建支持 JSON 诊断 |
| 包管理 | `zam.toml` 写 git 或本地路径依赖，`zam.lock` 锁定提交，依赖包里的模块按 `use 包名/模块` 导入 |
| 数值 | `i64`（溢出与除零在运行时检查）、`f64`（IEEE 754）、`bool`；除法与 Python 相同（`/` 得小数、`//` 整除、`%` 符号跟除数），整数与小数混用时整数变小数；`f64(n)`、`i64(x)` 显式转换 |
| 集合 | 定长 `[i64; N]`、可增长列表 `[T]`、字典 `{K: V}`；元素与值可以是标量、字符串或结构体，键为 `i64` 或 `string` |
| 标准库 | 文件读写、标准输入、命令行参数、整数与浮点数解析；`std/math` 的平方根、幂、三角、取整等函数；`std/time` 的时钟、计时、睡眠与日期文字；`std/random` 的随机整数与浮点数；`fmt.fixed` 按位数格式化；字符串的长度、比较、查找、替换、大小写转换、裁剪、切片、切分与连接；`println` |
| 枚举 | `enum` 带数据的变体、`match` 语句与表达式，检查分支是否覆盖所有变体 |
| 错误处理 | `T!string`、`fail`、`?` 传播、`catch` 处理 |
| 编辑器 | VS Code 语法高亮、括号与引号配对 |

尚未支持网络 I/O 和线程，也还没有 REPL。安装方式见 [预览版说明](PREVIEW.md)。

## 与其他语言对比

Zamak 想把 C、Rust、Go 与 Python 各自最擅长的部分融合在一起。下面的图在同一台机器上用各语言运行同样的程序：四个来自 [The Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/)，两个是日常任务。柱子越短越好。

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/compare-speed-dark.svg"/>
    <img src="docs/compare-speed-light.svg" alt="Zamak、C、Rust、Go、Java、Node.js 与 Python 运行 n-body、spectral-norm、fannkuch-redux、binary-trees、质数计数与词频统计的时间" width="920"/>
  </picture>
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/compare-footprint-dark.svg"/>
    <img src="docs/compare-footprint-light.svg" alt="Zamak、C、Rust、Go、Java、Node.js 与 Python 的可执行文件体积、启动时间与峰值内存" width="920"/>
  </picture>
</p>

| | Zamak | C | Rust | Go | Python |
| --- | :---: | :---: | :---: | :---: | :---: |
| 默认内存安全 | ✓ | ✗ | ✓ | ✓ | ✓ |
| 没有垃圾回收 | ✓ | ✓ | ✓ | ✗ | ✗ |
| 发布版检查整数溢出 | ✓ | ✗ | ✗ | ✗ | 不需要 |
| 检查数组越界 | ✓ | ✗ | ✓ | ✓ | ✓ |
| 独立的原生可执行文件 | ✓ | ✓ | ✓ | ✓ | ✗ |
| 不写生命周期标注 | ✓ | ✓ | ✗ | ✓ | ✓ |

在这些计算密集的程序上，Zamak 都与 C 处在同一档，同时默认检查溢出与越界。binary-trees 和词频统计在七种语言里最快；spectral-norm 与 C 相差 1% 以内，素数统计与 Rust 相差 2% 以内；只有 n-body 排在第三（Rust 第一、Go 第二），比 C 快 5%。唯一的例外是 spectral-norm 的热点函数用了 `@wrapping`，只在这一个函数里关闭溢出检查。测试方法、完整结果与复现步骤见 [bench/compare](bench/compare/README.md)。

## 配合 AI 编程助手使用

Zamak 把 AI 编程助手当作一等读者。新建一个助手可以马上上手的项目：

```sh
zam new hello --template ai
```

项目里会多出这些文件：

- **`.claude/skills/zamak/SKILL.md`**：Claude Code 的 Skill，读写 Zamak 代码时自动加载。
- **`AGENTS.md`**：内容相同，供 Codex、Cursor、GitHub Copilot 等读取 `AGENTS.md` 的助手使用。
- **`.mcp.json`**：给支持模型上下文协议（MCP）的助手（如 Claude Code、Cursor）启动 `zam mcp`。

两份说明都包含“检查 → 修复”的工作循环和一页纸语言参考，助手从第一行代码起就能写对。它们由编进 `zam` 的参考生成，新项目总与你安装的版本一致。

工作中，助手从 `zam check --json` 读取诊断：每条都有稳定的错误码（如 `ZM301`）、位置，通常还有修改建议。

通过 `zam mcp`，助手可以直接调用 Zamak：`check`、`build`、`run`（可传参数和标准输入）、`test`、`format`，用 `explain` 查错误码，用 `docs` 查语言参考和标准库。详见[配合 AI 编程助手](https://zamak.veyralux.org/zh-CN/ai-agents.html)。

## 快速运行

第一次使用可以先读 [Zamak 语言手册](https://zamak.veyralux.org/zh-CN/)。

安装预编译好的编译器：

```sh
npm install -g @veyralux/zamak     # 或：bun add -g @veyralux/zamak；想先试试可以用 npx @veyralux/zamak --version
zam new hello
zam run hello            # 打印 "hello from zamak"
```

以后更新运行 `zam upgrade`。没有 Node.js 的话，可以用安装脚本（Linux 和 macOS：`curl -fsSL https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.sh | sh`；Windows：`irm https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.ps1 | iex`），工具链要求见 [安装说明](PREVIEW.md)。

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
zam doc hello        # 从 pub 声明生成一张 HTML 的 API 页面
```

`hello` 是一个还不存在的目录。进入该目录后可以直接执行 `zam run`。新项目的输出是：

```text
hello from zamak
```

多模块、中文输出和所有权的示例见 [语言手册](https://zamak.veyralux.org/zh-CN/)。

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
错误[ZM301]：使用已移走的值：name
 --> src/main.zm:8:5
  |
8 |     println(name)
  |     ^^^^^^^^^^^^^
  = 提示：借用这个值，或在最后一次使用之后再移走
```

系统语言是中文时，`zam` 用中文显示诊断；把 `ZAMAK_LANG` 设为 `en` 或 `zh` 可以自己选语言。

`zam check --json` 与 `zam build --json` 把结果写成一行 JSON（始终是英文），给编辑器、CI 和 AI 工具读取：`{"command":"check","ok":false,"diagnostics":[{"file":"…","line":3,"column":9,"function":"main:main","code":"ZM100","category":"syntax","message":"…","suggestion":"…"}]}`。

- `code` 和 `category` 是稳定的机器字段，`message` 是给人看的文本；有明确修复方法时，`suggestion` 给出下一步。
- 无法定位时，`function` 和位置字段为 `null`。
- 词法、语法与模块解析错误只报第一条；类型与所有权检查逐个函数进行，每个出错的函数各报一条，`diagnostics` 按位置排序。
- 失败时 stderr 仍有人类可读的诊断，退出码与不带 `--json` 时相同。其他子命令不接受 `--json`。

### 新建项目

`zam new <directory>` 生成一个最小可运行的骨架：`zam.toml`（`name`、`entry = "src/main.zm"`、`modules = "src"`）、打印一句问候的 `src/main.zm`，以及忽略 `build/` 的 `.gitignore`。创建的文件路径以 `NEW <path>` 写到 stderr，stdout 保持为空。骨架已经格式化，可以直接 `zam run`。目标目录已存在时报错退出，不覆盖任何文件；目录名要是合法标识符，因为它会写进 `zam.toml` 的 `name`。

面向 AI 协作可以用 `zam new <directory> --template ai`，额外生成 `.claude/skills/zamak/SKILL.md`（Claude Code 的 Skill）、`AGENTS.md`（Codex、Cursor、GitHub Copilot 等读取，内容与 Skill 相同）与 `README.md`。两份说明写明 check --json → 最小修复 → fmt → check → build → run 的工作循环，并附上完整的一页纸语言参考；参考编进 `zam`（`main.rs` 的 `REFERENCE`，来源是 `bench/ai/live/zamak-reference.md`），新项目总与安装的版本一致。MCP 服务器（`zam mcp`）在计划中。

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

- **VS Code 扩展**：`editors/vscode/`。提供语法高亮，并运行 Zamak 的语言服务器 `zam lsp`：边写边报错（没保存的内容也检查）、补全、悬停查看签名、跳到定义，以及与 `zam fmt` 相同的格式化。其他支持语言服务器协议（LSP）的编辑器也能运行 `zam lsp`。同一份 `syntaxes/zamak.tmLanguage.json` 也能给 Sublime Text、TextMate 等编辑器使用。本地安装、打包 `.vsix` 与上架市场的步骤见 `editors/vscode/README.md`。
- **MCP 服务器**：`zam mcp`，给 AI 编程助手调用的工具：检查、构建、运行、测试、格式化、解释错误码、查文档。`zam new --template ai` 会在新项目里放好 Skill、`AGENTS.md` 和 `.mcp.json`。
- **AI 技能**：`.agents/skills/zamak/SKILL.md`，给修改编译器本身的 AI 编程代理（DSH、Claude Code 等）使用：命令与环境变量、语法速览、常见诊断、示例与测试的位置，以及修改编译器时的验收命令和 IR/缓存版本号规则。
- **CLI**：`zam` 本身，包括 `check`/`build`/`run`/`test`/`fmt`/`new`、包管理（`add`/`remove`/`update`/`search`/`publish`）、`lsp`、`--help`、`--version`；`check`/`build` 支持 `--json` 机器可读诊断；缓存目录可用 `ZAMAK_CACHE_DIR` 覆盖。

## 项目配置

```toml
[project]
name = "hello"
entry = "src/main.zm"
modules = "src"
```

配置包含 `[project]` 和可选的 `[dependencies]`（见下文“依赖包”）、`[native]`（见“调用 C 函数”）；`[project]` 除了上面三个字段，还可以写发布用的 `version`（如 `"1.0.0"`）和 `description`。库可以不写 `entry`：这时 `zam check` 与 `zam test` 照常可用，没有 `main` 也行，`zam build`/`zam run` 会提示它是库；路径相对配置文件，不能包含 `..`，入口要在模块目录内。`zam` 递归扫描模块目录里的 `.zm` 文件，一个文件对应一个模块，例如 `src/tools/greeting.zm` 的模块名是 `tools/greeting`。暂不支持源码符号链接。

```zam
use tools/greeting
use std/io

fn main():
    greeting.hello()
    io.println("你好")
```

被导入模块里的函数要声明为 `pub fn`。导入用 `/`，调用用模块路径的末段名称。模块别名有歧义、模块缺失、调用私有函数、重复声明和模块依赖成环都会报错。

### 依赖包

别人写好的 Zamak 库可以直接拿来用。依赖写在 `[dependencies]` 里，来源有三种：[公共包索引](https://github.com/VeyraLux/zamak-index)里的版本、git 仓库、本地目录：

```toml
[dependencies]
jsonx = "^1.2"                  # 包索引里 >=1.2.0、<2.0.0 的最高版本
mathx = { git = "https://github.com/you/mathx", rev = "v1.0" }
util = { path = "../util" }
```

也可以用命令添加：`zam add jsonx`（取最新版本，写成 `"^1.4.0"`）、`zam add jsonx "~1.2"`、`zam add mathx https://github.com/you/mathx v1.0`、`zam add util ../util`；`zam remove jsonx` 删除依赖；`zam search json` 在包索引里按名字和介绍查找。`zam add` 与 `zam remove` 修改当前目录的 `zam.toml` 并立即重新解析，失败时把 `zam.toml` 与 `zam.lock` 恢复原样。

- **版本范围**：写法同 Cargo。`^1.2`（也可以只写 `1.2`）是 `>=1.2.0, <2.0.0`，`^0.3` 是 `>=0.3.0, <0.4.0`；`~1.2` 是 `>=1.2.0, <1.3.0`；`=1.2.3` 只要这一个版本；`>=1.0, <3.0` 用逗号连接多个条件；`*` 是任何版本。`zam` 选出所有依赖方都接受的最高版本（不选已撤回的），选不出来时报错列出谁要求了什么，例如 `the project requires mathx ^2.0.0, geo 0.1.0 requires mathx >=1.1, <2`。也可以写成 `{ version = "^1.2" }`。
- **发布**：`zam.toml` 写上 `version` 和 `description`，提交后打标签 `git tag v1.0.0 && git push origin v1.0.0`，再运行 `zam publish`。它会检查包能编译、没有路径依赖、`zam.toml` 在仓库根目录、没有未提交的改动、标签已推送，然后打印索引条目；把它作为 `packages/<名字>.toml` 提 PR 到包索引仓库，自动检查会核对提交与版本号。发布过的版本不再改动，有问题的版本在索引里标 `yanked = true`。详见包索引仓库的说明。
- **换索引**：环境变量 `ZAMAK_INDEX` 可以换成别的 git 地址或本地目录；本地目录时 `zam publish` 直接把条目写进去，适合自建内部索引。

- **包的写法**：包就是一个普通的 Zamak 项目，`zam.toml` 里 `[project]` 的 `name` 要和依赖名相同，`entry` 可以省略（库不需要 `main`）。包可以再有自己的依赖，`zam` 会一起解析；同一个名字被要求从两个不同的来源拉取时报错。
- **导入**：包里的 `src/geo.zm` 写成 `use mathx/geo`，调用 `geo.twice(2)`；与包同名的 `src/mathx.zm` 是包的根模块，写成 `use mathx`。包内部的模块照常互相 `use geo`。只有 `pub` 的函数、结构体和常量能被项目使用；项目自己的模块不能和依赖同名。
- **锁定**：`rev` 可以是分支、标签或提交，省略时是默认分支。第一次构建把它解析成具体提交，包索引里的包解析成具体版本和提交，都写进 `zam.lock`（请提交到版本库）。之后的构建都用锁定的结果，即使标签移动了、索引里出了新版本也不变；锁定的版本仍满足版本范围时不读索引。`zam update` 忽略锁文件，把每个 `rev` 重新解析成最新的提交、每个包重新选最高的版本。路径依赖不进锁文件。
- **缓存与离线**：git 仓库的镜像与检出的源码放在缓存目录的 `packages/` 下（默认 `~/.zam/cache/packages`），按提交区分、只检出一次。锁定的提交已在缓存里时构建不访问网络。拉取 git 依赖需要 PATH 上有 `git`。
- **测试**：`zam test` 只运行项目自己的测试，不运行依赖包里的；`zam fmt` 也只整理项目自己的模块。

### 调用 C 函数

现成的 C 库可以直接拿来用。用 `extern fn` 声明 C 函数，调用写法和普通函数一样：

```zamak
extern fn sqrt(x: f64) -> f64

fn main():
    println(sqrt(16.0))
```

自己写的 C 代码和要链接的库写在 `zam.toml` 的 `[native]` 里：

```toml
[native]
sources = ["c/textutil.c"]   # 和项目一起编译的 C 文件
include = ["c"]               # 头文件目录
libraries = ["sqlite3"]       # 链接 libsqlite3（Windows 是 sqlite3.lib）
library_paths = ["lib"]       # 库文件所在的目录
```

- **类型**：参数可以是 `i64`（C 的 `int64_t`）、`f64`（`double`）、`bool`（`int`，非零为真）和 `&string`（复制成以 `\0` 结尾的 `const char *`，调用后释放）；返回值可以是这几种（`&string` 除外）、`string`（从 C 返回的 `const char *` 复制一份，内存仍归 C 管，NULL 当作空串）或不写。指针、结构体、数组和回调先在一个小的 C 函数里转换成这些类型。
- **名字**：Zamak 函数名要小写开头，C 函数是大写开头时写一个小写名字的 C 包装。声明 C 标准库里已有的函数时，签名要和 C 的一致（`sqrt` 可以，`abs` 是 `int abs(int)`，写成 `i64` 会和 C 的声明冲突）。
- **只链接用到的**：没被调用的 `extern fn` 不会生成调用代码，所以一个库可以声明很多 C 函数，用到哪个才需要哪个。
- **依赖包**：包可以有自己的 `[native]`，路径相对包的目录，所以 Zamak 库可以把一个 C 库包装好给别人用。
- **缓存**：C 源码按源码和 `include` 目录里所有 `.h` 文件的内容缓存，头文件改了会重新编译；`libraries` 与 `library_paths` 计入链接的缓存键。`include` 和 `library_paths` 可以是绝对路径。
- 示例：`zam run examples/ffi`。

## 语法范围

完整清单在[语法总览](https://zamak.veyralux.org/zh-CN/grammar.html)，一页看完。几个需要先知道的边界：

- 块用缩进界定，不用花括号；花括号只用于字典和插值。
- 借用不越过当前函数，所以不用写生命周期标注。代价是切片会复制，函数不能返回引用。
- 字符串、结构体、列表、字典是移动语义；数字和布尔是复制。
- `T!E` 表示可能失败，`call?` 向上传递，`call catch e:` 就地处理。
- 泛型目前按类型各生成一份代码。

暂不支持线程、网络、宏、trait 与继承。`zam check` 对这些写法给出明确报错。

## 缓存行为

缓存默认放在用户目录 `.zam/cache`，可用 `ZAMAK_CACHE_DIR` 指定。模块键包含源码字节、依赖名称与递归依赖哈希、Zamak 编译器/IR 版本（当前 IR-33）、操作系统和架构，不含项目绝对路径。相同源码从不同目录构建时共享模块产物。

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

Zamak 0.1.1 是早期版本，1.0 之前语法与库都可能变化。

## 许可证

本项目按 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双许可发布，使用者可以任选其一。除非另有声明，你提交的贡献同样按这两种许可证发布。
