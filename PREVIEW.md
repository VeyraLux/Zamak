# Zamak 0.1.0 预览版

这是用于试用和反馈的语言原型，语法及缓存格式可能变化。尚不适合生产使用。

## 安装与工具链

Windows x64：解压预览包，将其中 `bin` 目录加入 PATH。编译 Zamak 源码还需要 Visual Studio Build Tools，安装「使用 C++ 的桌面开发」和 Windows SDK；编译器通过 `vswhere` 查找 MSVC。也可安装 MinGW GCC 并设置 `CC=gcc`，使用 GNU 原生后端；`CC` 是单个编译器程序，不接受附加参数、MSVC `cl` 或 Windows `.bat` 包装脚本。不需要 Rust。仅运行已生成的程序不需要 Zamak 或 C 编译器。

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

教程见 [TUTORIAL.md](TUTORIAL.md)，常见算法示例见 [examples/tutorial_cases/main.zm](examples/tutorial_cases/main.zm)。

## 已知限制

目前支持 i64、bool、owned string、结构体、局部定长 i64 数组、while、整数区间 for 和调用帧借用。没有浮点数、动态数组、stdin/文件读写、泛型、枚举/match、网络、线程、FFI 或包管理器。因而案例中的交互输入使用固定输入；文件、网络和浮点案例不属于本预览版已通过的范围。

升级：替换 `bin/zam.exe`；缓存格式变化时自动使用新键。卸载：移除包目录和对应 PATH 项；可按需删除用户目录 `.zam/cache`。

## 发布准备

以下命令面向源码仓库，预览包不包含编译器源码或开发脚本。在仓库根目录执行 `pwsh -File tools/package-preview.ps1`，生成本地 ZIP 与 SHA-256 清单，重新解压 ZIP 后验证编译器的新建、检查、构建、运行和格式化流程，并验证生成的程序可独立运行，以及随包算法案例的完整输出。构建产物留在忽略的 `dist/` 中。

本地 Windows 安装流程、算法功能案例和三项算法性能筛查已通过，详见 [案例验证](docs/VALIDATION.md)。该结果仅覆盖文档所列范围，预览包尚未作为 GitHub Release 发布。

GitHub 的 Preview workflow 通过手动触发构建并上传预览产物；仅在明确准备发布时创建 tag 和 Release。此文档不意味着远端已发布。
