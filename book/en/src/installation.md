# Install Zamak

Zamak is a single command-line program, `zam`. It checks your code, compiles it to C, and calls a C compiler to produce a native executable. To install it, you need:

- **Rust stable**, to build `zam` itself. Install it from [rustup.rs](https://rustup.rs).
- **A C compiler**, which `zam` calls to build your programs.

## Install a C compiler

| Platform | What to install |
| --- | --- |
| Windows x64 | [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload. `zam` finds it automatically. If it isn't installed, `zam` uses `gcc` (for example, from MinGW-w64) or `clang` from `PATH`. |
| Linux | `cc`, `clang`, or `gcc`. On Debian and Ubuntu, run `sudo apt install build-essential`. |
| macOS | Xcode Command Line Tools. Run `xcode-select --install`. |

To choose a GCC-style compiler yourself, set the `CC` environment variable, such as `CC=clang`.

## Build and install zam

Clone the repository, and then install `zam` with Cargo:

```sh
git clone https://github.com/VeyraLux/Zamak.git
cd Zamak
cargo install --path zam --locked
```

Cargo puts `zam` in `~/.cargo/bin`, which rustup adds to your `PATH`.

## Check the installation

```sh
zam --version
```

```console
0.1.0
```

If your shell can't find `zam`, open a new terminal so that it picks up the updated `PATH`.

> [!NOTE]
> `zam check` doesn't need a C compiler, so you can check code and read diagnostics before you install one. `zam build` and `zam run` need it.

## Editor support

The VS Code extension in [editors/vscode](https://github.com/VeyraLux/Zamak/tree/main/editors/vscode) adds syntax highlighting and starts `zam lsp`, the Zamak language server, for diagnostics as you type, completion, hover, go to definition, and formatting. To install it, run `npm install` in that folder, link the folder into your VS Code extensions folder, and restart VS Code:

```sh
# Linux and macOS
cd /path/to/Zamak/editors/vscode && npm install
ln -s /path/to/Zamak/editors/vscode ~/.vscode/extensions/zamak-0.1.0
```

```powershell
# Windows
cd C:\path\to\Zamak\editors\vscode; npm install
New-Item -ItemType Junction -Path "$env:USERPROFILE\.vscode\extensions\zamak-0.1.0" -Target "C:\path\to\Zamak\editors\vscode"
```

The extension runs the `zam` on your `PATH`. To use another one, set **zamak.path** in the VS Code settings. Other editors that speak the Language Server Protocol can run `zam lsp` the same way.

## Update and uninstall

To update, pull the latest code and run the install command again:

```sh
git pull
cargo install --path zam --locked
```

To uninstall, run `cargo uninstall zam`. Build caches live in `~/.zam/cache`; delete that folder to remove them too.
