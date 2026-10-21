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
  <a href="#快速运行">快速运行</a>
  &nbsp;·&nbsp;
  <a href="#为什么选择-zamak">为什么选择 Zamak</a>
  &nbsp;·&nbsp;
  <a href="book/zh-CN/src/introduction.md">语言手册</a>
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

## 快速运行

```sh
npm install -g @veyralux/zamak    # 或：npx @veyralux/zamak --version 先试一下
zam new hello                     # 生成 hello/zam.toml 与 hello/src/main.zm
zam run hello                     # 打印 hello from zamak
```

`build` 和 `run` 需要一个 C 编译器（`check` 不需要）。Windows 上装 Visual Studio Build Tools 的「使用 C++ 的桌面开发」，没有时回落到 PATH 上的 `gcc` 或 `clang`；Linux 装 `build-essential`；macOS 运行 `xcode-select --install`。想指定编译器就设 `CC`。

其他安装方式与完整的命令说明：

- [用安装脚本或从源码构建](book/zh-CN/src/installation.md)
- [每个 `zam` 子命令的细节](book/zh-CN/src/tooling.md)
- [写第一个程序](book/zh-CN/src/first-program.md)

`zam` 把构建进度写到 stderr，stdout 只留程序自己的输出。产物不需要装 Zamak 或 Rust 就能运行。
## 配合 AI 编程助手使用

```sh
zam new myapp --template ai
```

项目里会多出 `.claude/skills/zamak/SKILL.md`、`AGENTS.md` 和 `.mcp.json`，每份都带一页纸语言参考和「检查 → 修复」的循环。这些说明由编进 `zam` 的参考生成，总和你安装的版本一致。助手从 `zam check --json` 读诊断，也可以通过 `zam mcp` 直接调用 `check`、`build`、`run`、`test`、`format`、`explain` 和 `docs`。

- [配合 AI 编程助手](book/zh-CN/src/ai-agents.md)
- [AI 实测的方法与数据](bench/ai/live/README.md)

## 安装 Zamak

从源码安装需要 Rust stable 和一个 C 编译器。在仓库根目录执行：

```sh
cargo install --path zam --locked
```

`zam` 按下面的顺序找 C 编译器：

| 平台 | C 编译器 |
| --- | --- |
| Windows x64 | Visual Studio Build Tools 的「使用 C++ 的桌面开发」，通过 `vswhere` 查找。没装时依次使用 PATH 上的 `gcc`（如 MinGW-w64）或 `clang`。 |
| Linux | PATH 上的 `cc`、`clang` 或 `gcc`。Debian 和 Ubuntu 上装 `build-essential`。 |
| macOS | Xcode Command Line Tools，运行 `xcode-select --install` 安装。 |

想自己指定编译器就设 `CC`。`zam check` 不需要 C 编译器。CI 在三个平台上跑完整测试。

## 新建项目

```sh
zam new hello           # 生成 hello/zam.toml 与 hello/src/main.zm
zam run hello           # 打印 hello from zamak
zam check hello         # 只做类型与借用检查，不需要 C 编译器
zam fmt hello           # 规范化缩进
zam test hello          # 运行项目里所有 fn test_...()
zam build hello         # 写出 build/hello-<hash>(.exe)
zam run app.zm -- a b   # 给程序传参数
zam doc hello           # 从 pub 声明生成 HTML API 页面
```

`zam` 把构建进度写到 stderr，stdout 只留程序自己的输出。

## 为什么选择 Zamak

**像 Python 一样好读，像 C 一样快，无GC也安全。**

<p align="center">
  <img src="docs/demo.svg" alt="小 Ƶ 把一段 Python 函数改成 Zamak，然后运行，打印出 13.5" width="720"/>
</p>

| 与 C 一样快 | Linux 上 hello world 只有 16 KB | Java 用 365 MB 的程序它只用 20 MB | 不写生命周期标注 | 发布版也检查整数溢出 |
| :---: | :---: | :---: | :---: | :---: |

Zamak 面向这样一种工作方式：多个 AI 助手并行修改同一个项目，一天构建上千次。目标冲突时它的优先级是构建成本、内存安全、运行性能、表达力，所以没有宏，没有依赖类型，也不用写生命周期。

- [为什么做这门语言，以及从 Rust、Go、C、Python 各借了什么](book/zh-CN/src/why-zamak.md)
- [与热门语言相比，以及什么时候该选别的语言](book/zh-CN/src/why-zamak.md#与热门语言相比)

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

- **熟悉的语法**：`len(xs)`、`for i in range(n)`、`xs[-1]`、`xs[1:3]`、`f"..."` 和 Python 的除法规则（`7 / 2` 是 `3.5`，`7 // 2` 是 `3`），`len("你好")` 是 2。
- **不用写生命周期**：字符串、结构体、列表和字典赋值时移动；传给只读参数自动借用，`&mut x` 在调用处显式标出。借用只在一次调用内有效。
- **该检查的地方都检查**：整数溢出、除零和越界在任何构建里都会停下，编译器会去掉能证明多余的检查。
- **反馈快**：`zam check` 不需要 C 编译器，5,000 行项目不到 0.1 秒；改一行重建约 0.75 秒。
- **原生独立运行**：生成 C 再交给 MSVC、GCC 或 Clang 编译，产物不需要装 Zamak 或 Rust。

语法看[语法总览](book/zh-CN/src/grammar.md)，构建与运行数字看 [B. 性能](book/zh-CN/src/appendix-performance.md)。

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

在这些计算密集的程序上，Zamak 与 C 处在同一档，同时默认检查溢出与越界：n-body、spectral-norm、fannkuch-redux、素数统计和词频统计互有高下，差距都在几个百分点内。binary-trees 比 C 参考实现快约 3.8 倍，但这个差距来自两个 benchmark 程序本身的写法，不来自编译器：C 版本逐个节点 `malloc`，Zamak 版本复用了缓冲区。同样用 arena 分配的 C 版本只要 46 ms，而 Zamak 是 265 ms，所以这一项量的是分配策略，不是代码生成。编译参数也已对齐：给 C 加上 Zamak 后端同样使用的 `-funroll-loops` 并不能缩小差距，反而让它在其余五项上变慢。唯一的例外是 spectral-norm 的热点函数用了 `@wrapping`，只在这一个函数里关闭溢出检查。测试方法、两组对照与复现步骤见 [bench/compare](bench/compare/README.md)。

## 编辑器与 AI 工具

- **VS Code 扩展**：`editors/vscode/`。提供语法高亮，并运行 Zamak 的语言服务器 `zam lsp`：边写边报错（没保存的内容也检查）、补全、悬停查看签名、跳到定义，以及与 `zam fmt` 相同的格式化。其他支持语言服务器协议（LSP）的编辑器也能运行 `zam lsp`。同一份 `syntaxes/zamak.tmLanguage.json` 也能给 Sublime Text、TextMate 等编辑器使用。本地安装、打包 `.vsix` 与上架市场的步骤见 `editors/vscode/README.md`。
- **MCP 服务器**：`zam mcp`，给 AI 编程助手调用的工具：检查、构建、运行、测试、格式化、解释错误码、查文档。`zam new --template ai` 会在新项目里放好 Skill、`AGENTS.md` 和 `.mcp.json`。
- **AI 技能**：`.agents/skills/zamak/SKILL.md`，给修改编译器本身的 AI 编程代理（DSH、Claude Code 等）使用：命令与环境变量、语法速览、常见诊断、示例与测试的位置，以及修改编译器时的验收命令和 IR/缓存版本号规则。
- **CLI**：`zam` 本身，包括 `check`/`build`/`run`/`test`/`fmt`/`new`、包管理（`add`/`remove`/`update`/`search`/`publish`）、`lsp`、`--help`、`--version`；`check`/`build` 支持 `--json` 机器可读诊断；缓存目录可用 `ZAMAK_CACHE_DIR` 覆盖。

## 项目配置

最小配置只有三段：

```toml
[project]
name = "hello"
entry = "src/main.zm"
modules = "src"
```

一个 `.zm` 文件就是一个模块，`src/tools/greeting.zm` 的模块名是 `tools/greeting`。可选段落：`[dependencies]` 声明依赖，`[native]` 配置要链接的 C 代码，`[project]` 里还可以写发布用的 `version` 和 `description`。

- [模块与包](book/zh-CN/src/modules.md)
- [调用 C 代码](book/zh-CN/src/c-libraries.md)

## 缓存

缓存默认在 `~/.zam/cache`，可用 `ZAMAK_CACHE_DIR` 改位置。模块键取源码内容、递归依赖哈希、编译器与 IR 版本、系统和架构，**不含项目路径**，所以同一份源码从不同目录构建会共享产物。改一个模块只让它和依赖它的模块失效。

目标文件与可执行文件另有一层按内容寻址的缓存，构建输出里的 `CACHE`、`OBJECT`、`LINK`、`OUTPUT` 就是这几层的命中情况。细节见 [工具链说明](book/zh-CN/src/tooling.md)。
## 验证

提交前在仓库根目录跑这几条，必须全部通过：

```powershell
cargo test --manifest-path zam/Cargo.toml --locked
cargo fmt --manifest-path zam/Cargo.toml -- --check
cargo clippy --manifest-path zam/Cargo.toml --all-targets --locked -- -D warnings
```

开发规范、可复现的工程测量和仓库结构见 [贡献指南](CONTRIBUTING.md)，测试方法与性能结论见 [案例验证](docs/VALIDATION.md)。

Zamak 0.1.1 是早期版本，1.0 之前语法与库都可能变化。

## 许可证

本项目按 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 双许可发布，使用者可以任选其一。除非另有声明，你提交的贡献同样按这两种许可证发布。
