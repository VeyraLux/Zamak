# 安装 Zamak

Zamak 只有一个命令行程序 `zam`。它检查你的代码、把代码编译成 C，再调用 C 编译器生成本机可执行文件。

## 快速安装

有 Node.js（或 Bun）的话，这是最省事的方式。它会安装预编译好的 `zam`，支持 Windows x64、Linux（x64 与 arm64）和 macOS（Apple 芯片与 Intel）：

```sh
npm install -g @veyralux/zamak      # 或：bun add -g @veyralux/zamak
zam --version
```

想先试试，可以运行 `npx @veyralux/zamak --version`，不用安装。以后更新运行 `zam upgrade`。没有 Node.js 时用安装脚本：Linux 和 macOS 运行 `curl -fsSL https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.sh | sh`，Windows 运行 `irm https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.ps1 | iex`。

`zam build` 和 `zam run` 仍然需要一个 C 编译器，见下一节。Windows 版目前没有代码签名。用 npm 或安装脚本安装不会触发 Windows SmartScreen；如果在浏览器里手动下载 zip、Windows 弹出警告，选**更多信息**，再选**仍要运行**。

想自己编译 `zam` 的话，需要 Rust stable：

- **Rust stable**：用来编译 `zam` 本身。从 [rustup.rs](https://rustup.rs) 安装。
- **C 编译器**：`zam` 用它构建你的程序。

## 安装 C 编译器

| 平台 | 需要安装 |
| --- | --- |
| Windows x64 | [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)，勾选**使用 C++ 的桌面开发**工作负载。`zam` 会自动找到它。没有安装时，`zam` 改用 `PATH` 里的 `gcc`（例如 MinGW-w64 提供的）或 `clang`。 |
| Linux | `cc`、`clang` 或 `gcc`。Debian 和 Ubuntu 上运行 `sudo apt install build-essential`。 |
| macOS | Xcode 命令行工具。运行 `xcode-select --install`。 |

要自己指定 GCC 风格的编译器，设置环境变量 `CC`，例如 `CC=clang`。

## 编译并安装 zam

克隆仓库，然后用 Cargo 安装 `zam`：

```sh
git clone https://github.com/VeyraLux/Zamak.git
cd Zamak
cargo install --path zam --locked
```

Cargo 把 `zam` 放在 `~/.cargo/bin`，rustup 已经把这个目录加进了 `PATH`。

## 检查安装

```sh
zam --version
```

```console
0.2.0
```

如果终端找不到 `zam`，打开一个新终端，让它读到更新后的 `PATH`。

> [!NOTE]
> `zam check` 不需要 C 编译器，所以装好编译器之前就可以检查代码、阅读诊断信息。`zam build` 和 `zam run` 需要编译器。

## 编辑器支持

[editors/vscode](https://github.com/VeyraLux/Zamak/tree/main/editors/vscode) 里的 VS Code 扩展提供语法高亮，并启动 Zamak 的语言服务器 `zam lsp`，带来边写边报错、补全、悬停、跳到定义和格式化。安装方法是在这个目录里运行 `npm install`，把目录链接到 VS Code 的扩展目录，然后重启 VS Code：

```sh
# Linux 和 macOS
cd /path/to/Zamak/editors/vscode && npm install
ln -s /path/to/Zamak/editors/vscode ~/.vscode/extensions/zamak-0.2.0
```

```powershell
# Windows
cd C:\path\to\Zamak\editors\vscode; npm install
New-Item -ItemType Junction -Path "$env:USERPROFILE\.vscode\extensions\zamak-0.2.0" -Target "C:\path\to\Zamak\editors\vscode"
```

扩展运行 `PATH` 里的 `zam`。想用别的 `zam`，在 VS Code 设置里修改 **zamak.path**。其他支持语言服务器协议（LSP）的编辑器也可以用同样的方式运行 `zam lsp`。

## 更新与卸载

更新时拉取最新代码，再运行一次安装命令：

```sh
git pull
cargo install --path zam --locked
```

卸载时运行 `cargo uninstall zam`。构建缓存在 `~/.zam/cache`，删除这个目录即可一并清除。
