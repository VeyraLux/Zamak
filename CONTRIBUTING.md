# 参与 Zamak

Zamak 目前是语言原型。提交问题时请附上最小 `.zm` 示例、`zam --version`、操作系统、C 工具链版本，以及完整 stdout/stderr。请区分编译失败、程序结果错误和性能问题。

## 本地验证

需要 Rust stable；原生构建在 Windows 需要 Visual Studio Build Tools 的「使用 C++ 的桌面开发」及 Windows SDK，在 Linux/macOS 需要 `cc`。

```text
cargo test --manifest-path zam/Cargo.toml --locked
cargo fmt --manifest-path zam/Cargo.toml -- --check
cargo clippy --manifest-path zam/Cargo.toml --all-targets --locked -- -D warnings
node editors/vscode/scripts/check-grammar.mjs
pwsh -File bench/ai/run.ps1 -ZamPath .\zam\target\debug\zam.exe -LatencyRepeats 3
pwsh -File tools/measure-build.ps1 -ZamPath .\zam\target\debug\zam.exe -Modules 6 -Repetitions 3
```

最后两条把结果写进被忽略的 `results/`：AI 闭环评测用固定候选和补丁检查诊断、修复、构建与输出；构建测量断言冷/热缓存与增量重编数量，断言失败会直接报错，而不是记录可疑数据。

新语言功能应包含可运行示例、拒绝反例和原生执行结果断言。修改缓存后，验证跨目录复用、损坏检测和并发发布。性能结论需给出源码、工具版本、输入规模、预热与重复次数，不以缓存命中代替真实重编成本。

## 仓库结构

- `zam/src`：Rust 前端、C 后端、项目与缓存、CLI。
- `zam/tests`：集成测试。
- `examples`：可运行的 Zamak 示例。
- `tools`：可复现验证与打包工具。
- `editors/vscode`：纯声明式语法高亮扩展。

本地缓存、生成物和会话恢复资料不应提交。请保持 PR 专注于一个问题，并描述实际验证结果。
