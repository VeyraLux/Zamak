# Mbed TLS 3.6.7 (vendored)

Unmodified `library/` and `include/` from the Mbed TLS 3.6.7 release
(https://github.com/Mbed-TLS/mbedtls/releases/tag/mbedtls-3.6.7,
`mbedtls-3.6.7.tar.bz2`, SHA-256
`a7e8bcbec0e6f761b4af24f25677626b35f762f68eef79c08677a363212d11f6`).
Mbed TLS is licensed under Apache-2.0 OR GPL-2.0-or-later; Zamak uses it under Apache-2.0
(see `LICENSE`).

`zam/build.rs` compresses these files into the `zam` executable. When a program uses
`std/tls` (for example through `https://` in `std/http`), `zam` writes them to its cache
and compiles them once with the default configuration.

To update: replace `library/` and `include/` with those of a newer 3.6.x release, update
this file, and bump `native-c-N` in `zam/src/project.rs`.
