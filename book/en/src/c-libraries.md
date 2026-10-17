# Call C code

Zamak compiles to C, so it can call C functions directly. Use this to reach a C library, or to reuse C code you already have.

## Declare a C function

`extern fn` declares a function that's written in C. It has no body; the call goes straight to the C function with the same name. Functions from the C standard library are available without any setup:

```zamak
extern fn hypot(x: f64, y: f64) -> f64
extern fn cbrt(x: f64) -> f64

fn main():
    println(hypot(3, 4))
    println(cbrt(27))
```

```text
5.0
3.0
```

The declaration must match the C signature. Zamak maps types as follows:

| Zamak | C |
| --- | --- |
| `i64` | `int64_t` |
| `f64` | `double` |
| `bool` | `int` |
| `&string` parameter | `const char *`, ending with `\0` |
| `string` result | `const char *`; Zamak copies the text |

Pointers and structs can't cross the boundary directly. Pass them through a small C wrapper that takes and returns these types.

## Compile your own C files

To build C files with your project, list them in a `[native]` section of `zam.toml`:

```toml,project=ffi,file=zam.toml
[project]
name = "ffi"
entry = "src/main.zm"
modules = "src"

[native]
sources = ["c/textutil.c"]
```

```c,project=ffi,file=c/textutil.c
#include <stdint.h>
#include <stdio.h>
#include <string.h>

int64_t count_vowels(const char *text) {
    int64_t count = 0;
    for (; *text; text++) {
        if (strchr("aeiouAEIOU", *text)) count++;
    }
    return count;
}

/* Zamak copies the returned text, so a static buffer is fine. */
const char *greeting(const char *name) {
    static char buffer[256];
    snprintf(buffer, sizeof buffer, "hello, %s", name);
    return buffer;
}
```

```zamak,project=ffi,file=src/main.zm
extern fn count_vowels(text: &string) -> i64
extern fn greeting(name: &string) -> string

fn main():
    println(count_vowels("Zamak is fast"))
    println(greeting("Ada"))
```

```text
4
hello, Ada
```

The `[native]` section accepts:

- `sources`: C files to compile with the project.
- `include`: folders to search for header files.
- `libraries`: libraries to link, such as `["sqlite3"]` for `-lsqlite3`.

A package can carry its own `[native]` section, so you can wrap a C library once and let other projects use it as an ordinary Zamak package.

## Safety at the boundary

Zamak's safety guarantees stop at the `extern fn` line: the compiler can't check what C code does. Keep C wrappers small, convert everything to the simple types above, and let Zamak code handle the rest.

## Summary

- `extern fn name(...)` calls a C function; the C standard library needs no setup.
- Parameters and results are `i64`, `f64`, `bool`, `&string`, or `string`.
- List C files and libraries in the `[native]` section of `zam.toml`.
