# zam 命令

用 Zamak 做的一切都通过一个命令：`zam`。这一章列出它的子命令，说明诊断信息和缓存怎样工作，并介绍会改变它行为的环境变量。

## 子命令

大多数子命令接收一个项目目录或单个 `.zm` 文件；不写时使用当前目录。

| 命令 | 作用 |
| --- | --- |
| `zam new NAME` | 创建项目。加上 `--template ai` 会同时生成给 AI 编程助手用的文件。 |
| `zam check` | 检查类型和所有权，不编译，不需要 C 编译器。 |
| `zam build` | 编译成独立的可执行文件，放在 `build/` 里。 |
| `zam run [-- ARGS]` | 构建并运行，把 `ARGS` 传给程序。 |
| `zam test [WORD]` | 运行 `test_...` 函数；给了 `WORD` 时只运行名字里包含它的测试。 |
| `zam fmt` | 规范缩进和空行。 |
| `zam add`、`zam remove`、`zam update` | 管理依赖。 |
| `zam search WORD`、`zam publish` | 查找和发布包。 |
| `zam doc [目录]` | 从 `pub` 声明生成 HTML 的 API 页面。 |
| `zam lsp` | 运行给编辑器用的语言服务器。VS Code 扩展会自动启动它。 |
| `zam mcp` | 运行给 AI 编程助手用的 MCP 服务器，见[配合 AI 编程助手](ai-agents.md)。 |
| `zam --version` | 打印版本号。 |

`zam` 把自己的进度信息写到标准错误。标准输出里只有你的程序的输出，或者 `--json` 生成的 JSON 报告。

## 阅读诊断信息

`zam check` 发现问题时，会显示稳定的错误码、位置、源码行和修改建议：

```zamak,fails
fn main():
    let name = "Ada"
    let other = name
    println(other)
    println(name)
```

```text
错误[ZM301]：使用已移走的值：name
 --> main.zm:5:5
  |
5 |     println(name)
  |     ^^^^^^^^^^^^^
  = 提示：借用这个值，或在最后一次使用之后再移走
```

- 错误码（例如 `ZM301`）的含义永远不变，可以直接搜索。[附录 D](appendix-diagnostics.md) 列出了所有错误码。
- 语法错误在第一处就停止检查。类型和所有权错误每个函数报告一次，所以一轮可以修好好几个函数。
- 如果你按 Python 的写法写了某个东西，而 Zamak 的写法不同，提示行会给出 Zamak 的写法。
- 系统语言是中文时，`zam` 用中文显示诊断。想自己选语言，把 `ZAMAK_LANG` 设为 `zh` 或 `en`。

## JSON 格式的诊断

编辑器、脚本和 AI 助手可以要求输出一行 JSON：

```sh
zam check --json main.zm
```

```json
{"command":"check","ok":false,"diagnostics":[{"file":"main.zm","line":5,"column":5,"function":"main:main","code":"ZM301","category":"ownership","message":"use after move: name","suggestion":"borrow the value or move it only after the last use"}]}
```

`zam build --json` 以同样的方式报告构建错误。不论系统语言是什么，JSON 始终是英文，工具可以放心依赖其中的文字。

## 格式化代码

`zam fmt` 就地改写文件：把 Tab 换成空格，每层代码块缩进 4 个空格，顶层声明之间留一个空行。它只改行首的空白和空行，所以永远不会动注释、字符串或行内的空格。每改动一个文件就打印 `FORMAT path`；语法不对的文件它不会修改。

## 生成 API 页面

`zam doc` 读取项目里的 `pub` 声明，写出一张 HTML 页面：

```sh
zam doc
zam doc . site
```

第一种写法写到 `docs/index.html`；第二种自己指定输出目录。每个模块一节，每个公开的函数、结构体、枚举和常量各有一条；结构体的字段、枚举的变体和方法列在它们的类型下面。

文档写成声明正上方的 `#` 注释：第一行是摘要，其余各行是正文。

```zamak
# 平面上的一个点。
struct Point:
    # 横坐标。
    x: i64
    y: i64

    # 返回向右移动过的一个副本。
    fn right(&self, steps: i64) -> Point:
        Point(x: self.x + steps, y: self.y)

fn main():
    let p = Point(x: 1, y: 2)
    let moved = p.right(3)
    println(moved.x)
```

```text
4
```

只有 `pub` 声明会出现在页面里，所以项目内部的帮手不会露出来。页面不需要 JavaScript，从磁盘直接打开就能看，放在仓库旁边就能发布。

## 缓存怎样工作

`zam` 按源码内容（以及它依赖的模块的内容）缓存每个编译好的模块。缓存放在 `~/.zam/cache`，所有项目、同一项目的所有副本（例如 Git worktree）共用。因此：

- 同样的代码构建两次，或者在两个目录里各构建一次，只编译一遍。
- 改动一个模块，只重新构建它，以及用到它公开内容的模块。
- 在 5,000 行的项目上，`zam check` 不到 0.1 秒。

`~/.zam/cache` 随时可以删除，`zam` 会重新构建需要的部分。

## 环境变量

| 变量 | 作用 |
| --- | --- |
| `CC` | 使用的 C 编译器，例如 `gcc` 或 `clang`。默认情况下，Windows 上装了 MSVC 就用 MSVC，其他平台用 `cc`。 |
| `ZAMAK_CACHE_DIR` | 构建缓存的位置，代替 `~/.zam/cache`。 |
| `ZAMAK_JOBS` | 并行编译的模块数。默认是 CPU 核数，最多 4。 |
| `ZAMAK_INDEX` | 使用的包索引，可以是 Git 地址或本地目录。 |
| `ZAMAK_DUMP_C` | 一个目录，`zam` 会把每个模块生成的 C 代码写到这里。研究性能时很有用。 |
| `ZAMAK_LANG` | 终端里诊断的语言：`zh` 是中文，`en` 是英文。默认跟随系统语言。 |

## 小结

- `zam check`、`build`、`run`、`test`、`fmt` 覆盖日常的编辑循环；`add`、`remove`、`update`、`search`、`publish` 管理包。
- `zam doc` 把 `pub` 声明和它们的 `#` 注释变成一张 HTML 的 API 页面。
- 每条诊断都有稳定的错误码，通常还有修改建议；`--json` 把同样的信息提供给工具。
- 基于内容的缓存让重复构建和增量构建都很快。
