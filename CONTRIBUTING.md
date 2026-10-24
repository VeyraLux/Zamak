# Contribute to Zamak

Thanks for helping. Zamak is young, so bug reports from real programs are as valuable as code.

[中文](CONTRIBUTING.zh-CN.md)

## Report a bug

Open an issue and include:

- The smallest `.zm` file that shows the problem.
- The output of `zam --version`, your operating system, and your C compiler (MSVC, GCC, or Clang) and its version.
- The full output, both stdout and stderr.
- Whether the program failed to compile, compiled but gave a wrong result, or was slower than you expected.

If `zam` itself crashed, say so. A compiler crash is always a bug, whatever the input.

## Set up

You need Rust (stable) and a C compiler:

- **Windows**: Visual Studio Build Tools with the **Desktop development with C++** workload, which includes the Windows SDK.
- **Linux**: `gcc` or `clang`. To build programs that use `std/ui`, also install the GTK 3 development package, such as `libgtk-3-dev`.
- **macOS**: the Xcode command-line tools (`xcode-select --install`).

Then build and test from the `zam` folder:

```sh
cd zam
cargo build
cargo test --no-fail-fast
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The tests compile and run hundreds of programs and use every CPU core. To keep your machine responsive, add `-- --test-threads=4`, or run one file at a time, such as `cargo test --test lists`.

## Make a change

1. Read [AGENTS.md](AGENTS.md). It explains where the code lives, the design principles, and which docs to update with each kind of change. It applies to people and to AI coding tools alike.
2. Branch from `dev`. Changes to docs alone can target `main`.
3. Add a test. A new feature needs a runnable example, a test that checks its output, and tests for the code it rejects. A bug fix needs a test that fails without the fix.
4. Update the docs that your change affects. The table in AGENTS.md lists them.
5. Write the commit message in English as `<type>: <summary>`, such as `fix: reject a negative list index at compile time`. In the body, say what changed and why.
6. Open a pull request against `dev`. Keep it to one topic, and describe how you tested it.

## Write docs

The Zamak Book (`book/`) has an English and a Chinese edition. Change both, and run `cargo test --test book`. It compiles every code block and checks the output shown after it.

Follow the [Microsoft Writing Style Guide](https://learn.microsoft.com/style-guide/welcome/): address the reader as "you", use active voice and short sentences, and write headings in sentence case. Show a runnable example instead of describing one.

## License

You can use Zamak under either the [MIT license](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE). Unless you state otherwise, you license your contributions under the same terms.
