# Zamak 0.2.0 安装说明

这是早期版本，1.0 之前语法及缓存格式可能变化，暂不建议用于生产。

## 安装与工具链

最省事的方式是 npm（Windows x64、Linux x64 与 arm64、macOS Apple 芯片与 Intel）：

```sh
npm install -g @veyralux/zamak    # 或 bun add -g @veyralux/zamak；不安装直接试用：npx @veyralux/zamak --version
zam upgrade             # 以后更新
```

没有 Node.js 时用安装脚本：Linux 和 macOS 运行 `curl -fsSL https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.sh | sh`，Windows 运行 `irm https://raw.githubusercontent.com/VeyraLux/Zamak/main/install.ps1 | iex`。安装脚本会校验 SHA-256。

Windows 版目前没有代码签名。用 npm 或安装脚本安装不会触发 SmartScreen；如果在浏览器里手动下载 zip，Windows 可能提示“已保护你的电脑”，点「更多信息」再点「仍要运行」即可。

手动安装 Windows x64：解压安装包，将其中 `bin` 目录加入 PATH。编译 Zamak 源码还需要一个 C 编译器：Visual Studio Build Tools（安装「使用 C++ 的桌面开发」和 Windows SDK，通过 `vswhere` 查找），或者 PATH 上的 MinGW-w64 `gcc`。没有 Visual Studio 时会自动改用 `gcc`，再试 `clang`；也可以用 `CC` 显式指定。`CC` 是单个编译器程序，不接受附加参数、MSVC `cl` 或 Windows `.bat` 包装脚本。不需要 Rust。仅运行已生成的程序不需要 Zamak 或 C 编译器。

Linux/macOS：从源码安装需要 Rust stable 和系统 `cc`：

```sh
cargo install --path zam --locked
```

Windows 包是本地验证的平台；其他系统由 CI 验证，实际 CI 运行状态请查看 GitHub Actions。下载包后可与随包的 SHA-256 校验文件比对。

## 第一个项目

在空工作目录执行：

```text
zam --version
zam new hello
zam check hello
zam build hello
zam run hello
zam fmt hello
```

预期输出包含 `hello from zamak`。`build` 的 stderr 会显示 `OUTPUT <path>`，该产物可独立运行。

教程见 [Zamak 语言手册](https://zamak.veyralux.org/zh-CN/)，常见算法示例见 [examples/tutorial_cases/main.zm](examples/tutorial_cases/main.zm)。

## 已知限制

标量支持 `i64`、`f64`、`bool` 和 `string`。复合类型支持结构体、枚举（含 `Option`/`match`）、定长数组、可增长列表、字典与元组；列表和字典可以容纳结构体，也可以嵌套。泛型函数会按实参类型生成专用副本。可用 `extern fn` 调用 C 函数。控制流支持 `if`/`elif`/`else`、`while`、整数区间 `for`、列表/字典遍历和 `for x in &items`；错误用 `?` 与 `catch` 处理。标准库覆盖整文件读写、标准输入、命令行参数、目录遍历、字符串与容器操作。

尚未实现：网络、线程、`async`，以及独立的调试器。跨语言边界只支持 C 函数，且参数与返回值限于数值、布尔和只读字符串。

升级：替换 `bin/zam.exe`；缓存格式变化时自动使用新键。卸载：移除包目录和对应 PATH 项；可按需删除用户目录 `.zam/cache`。

## 发布准备

以下命令面向源码仓库，预览包不包含编译器源码或开发脚本。在仓库根目录执行 `pwsh -File tools/package-preview.ps1`，生成本地 ZIP 与 SHA-256 清单，重新解压 ZIP 后验证编译器的新建、检查、构建、运行和格式化流程，并验证生成的程序可独立运行，以及随包算法案例的完整输出。构建产物留在忽略的 `dist/` 中。

本地 Windows 安装流程、算法功能案例和三项算法性能筛查已通过，详见 [案例验证](docs/VALIDATION.md)。该结果仅覆盖文档所列范围。

## 发布状态

v0.1.0 与 v0.1.1 已于 2026-10-19 发布为 GitHub Release，同时发布到 npm（`@veyralux/zamak`）。v0.2.0 起，推送版本标签后自动构建五个平台的安装包，并发布 GitHub Release、npm 包、VS Code 插件和 Book 网站。发布流程通过手动触发或打 tag 启动，产物包含各平台安装包与 SHA-256 清单。本文档描述的是安装与首次使用，不替代 CHANGELOG 的版本说明。
