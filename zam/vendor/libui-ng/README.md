# libui-ng

This folder holds the library sources of [libui-ng](https://github.com/libui-ng/libui-ng), a portable GUI library for C with native controls on each system. `std/ui` uses it.

- Commit: `43ba1ef553c8993a43a67f1ce6e35983a2660d8c` (2025-03-15). libui-ng has no tagged release.
- Source: <https://codeload.github.com/libui-ng/libui-ng/tar.gz/43ba1ef553c8993a43a67f1ce6e35983a2660d8c>
- SHA-256 of that archive: `55315ed938d13ab82fe3fb42a6cc2eccfd3a5991750f7e00f841d4f2403e34dd`
- License: MIT. See `LICENSE.md`.

Only these parts of the archive are kept, unchanged: `ui.h`, `ui_windows.h`, `ui_unix.h`, `ui_darwin.h`, `LICENSE.md`, the `.h`, `.hpp`, and `.manifest` files in `common/`, `windows/`, `unix/`, and `darwin/`, and the `.c`, `.cpp`, and `.m` files that the folders' `meson.build` files compile. The unused `OLD_table` files are left out.

`zam` builds it as a static library: `_UI_STATIC` is defined, Windows programs embed `windows/libui.manifest` (Common Controls 6), Linux uses GTK 3 through `pkg-config`, and macOS links Foundation and AppKit.
