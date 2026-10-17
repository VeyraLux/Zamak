# 安装 Zamak

Zamak 只有一个命令行程序 `zam`。它检查你的代码、把代码编译成 C，再调用 C 编译器生成本机可执行文件。安装它需要：

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
0.1.0
```

如果终端找不到 `zam`，打开一个新终端，让它读到更新后的 `PATH`。

> [!NOTE]
> `zam check` 不需要 C 编译器，所以装好编译器之前就可以检查代码、阅读诊断信息。`zam build` 和 `zam run` 需要编译器。

## 编辑器支持

[editors/vscode](https://github.com/VeyraLux/Zamak/tree/main/editors/vscode) 里的 VS Code 扩展提供语法高亮、括号匹配、以 `:` 结尾的行之后自动缩进，以及代码折叠。安装方法是把这个目录链接到 VS Code 的扩展目录，然后重启 VS Code：

```sh
# Linux 和 macOS
ln -s /path/to/Zamak/editors/vscode ~/.vscode/extensions/zamak-0.1.0
```

```powershell
# Windows
New-Item -ItemType Junction -Path "$env:USERPROFILE\.vscode\extensions\zamak-0.1.0" -Target "C:\path\to\Zamak\editors\vscode"
```

## 更新与卸载

更新时拉取最新代码，再运行一次安装命令：

```sh
git pull
cargo install --path zam --locked
```

卸载时运行 `cargo uninstall zam`。构建缓存在 `~/.zam/cache`，删除这个目录即可一并清除。
