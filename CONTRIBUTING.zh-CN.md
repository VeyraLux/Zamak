# 参与 Zamak

谢谢你来帮忙。Zamak 还很年轻，用真实程序报告的问题和代码一样有价值。

[English](CONTRIBUTING.md)

## 报告问题

提一个 Issue，附上这些信息：

- 能复现问题的最小 `.zm` 文件。
- `zam --version` 的输出、操作系统，以及 C 编译器（MSVC、GCC 或 Clang）和它的版本。
- 完整的输出，标准输出和标准错误都要。
- 问题属于哪一种：编译不过、编译通过但结果不对，还是比你预期的慢。

如果是 `zam` 自己崩溃了，请写明。不管输入是什么，编译器崩溃都算漏洞。

## 准备环境

需要 Rust（stable）和一个 C 编译器：

- **Windows**：Visual Studio Build Tools，勾选 **使用 C++ 的桌面开发**，其中包含 Windows SDK。
- **Linux**：`gcc` 或 `clang`。要编译用到 `std/ui` 的程序，还要装 GTK 3 的开发包，例如 `libgtk-3-dev`。
- **macOS**：Xcode 命令行工具（`xcode-select --install`）。

然后在 `zam` 目录里构建和测试：

```sh
cd zam
cargo build
cargo test --no-fail-fast
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

测试会编译并运行几百个程序，用满所有 CPU 核。想让电脑保持流畅，可以加上 `-- --test-threads=4`，或者一次只跑一个文件，例如 `cargo test --test lists`。

## 提交改动

1. 先读 [AGENTS.md](AGENTS.md)。里面说明了代码放在哪里、设计原则，以及每类改动要同步哪些文档。这份规范对人和 AI 编程工具都适用。
2. 从 `dev` 拉分支。只改文档的可以直接对 `main`。
3. 加测试。新功能要有可运行的示例、检查它输出的测试，以及它应当拒绝的写法的测试。修漏洞要有一个不修就会失败的测试。
4. 更新受影响的文档，AGENTS.md 里的表格列出了要改哪些。
5. 提交信息用英文，格式是 `<type>: <summary>`，例如 `fix: reject a negative list index at compile time`。正文写清改了什么、为什么。
6. 向 `dev` 提 Pull Request。一个 PR 只做一件事，并说明你是怎么测试的。

## 写文档

Zamak Book（`book/`）有英文和中文两版，改的时候两版都要改，然后运行 `cargo test --test book`。它会编译每一段代码，并核对紧跟在后面的输出。

文风参考 [Microsoft 写作风格指南](https://learn.microsoft.com/style-guide/welcome/)：用“你”称呼读者，用主动语态和短句，直说事实，不用套话；中文和英文、数字之间留一个空格。能给出可运行的例子，就不要只用文字描述。

## 许可证

Zamak 可以按 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 任选其一使用。除非另有声明，你提交的贡献同样按这两种许可证发布。
