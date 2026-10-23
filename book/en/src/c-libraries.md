# Call C code

Zamak compiles to C, so it can call C functions directly. Use this to reach a C library, or to reuse C code you already have.

## Declare a C function

`extern fn` declares a function that's written in C. It has no body; the call goes straight to the C function with the same name. Functions from the C standard library are available without any setup:

```zamak
extern fn hypot(x: f64, y: f64) -> f64
extern fn fmax(a: f64, b: f64) -> f64

fn main():
    println(hypot(3, 4))
    println(fmax(2.5, 7))
```

```text
5.0
7.0
```

The declaration must match the C signature. Zamak maps types as follows:

| Zamak | C |
| --- | --- |
| `i64` | `int64_t` |
| `f64` | `double` |
| `bool` | `int` |
| `&string` parameter | `const char *`, ending with `\0` |
| `string` result | `const char *`; Zamak copies the text |
| `&bytes`, `&mut bytes` parameter | `const uint8_t *`, `uint8_t *` |
| C handle `&Db` parameter | The pointer type you declare; see below |

Other structs can't cross the boundary directly. Pass them through a small C wrapper that takes and returns these types.

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

- `sources`: files to compile with the project. C files end in `.c`, C++ files in `.cpp`, `.cc`, or `.cxx`, and Objective-C files (macOS only) in `.m`. Export C++ functions with `extern "C"` so that Zamak can call them.
- `include`: folders to search for header files.
- `headers`: the library's header files, such as `["sqlite3.h"]`. See below.
- `libraries`: libraries to link, such as `["sqlite3"]` for `-lsqlite3`.
- `library_paths`: folders to search for libraries.
- `frameworks`: macOS frameworks to link, such as `["Foundation"]`. Other systems ignore it.

A package can carry its own `[native]` section, so you can wrap a C library once and let other projects use it as an ordinary Zamak package.

## C handles and buffers

Many C libraries hand out an object through a pointer and expect you to free it with another function, such as SQLite's `sqlite3 *` and `sqlite3_close`. Declare such a handle with `extern type`, and Zamak frees it for you:

```toml,project=tally,file=zam.toml
[project]
name = "tally"
entry = "src/main.zm"
modules = "src"

[native]
sources = ["c/tally.c"]
include = ["c"]
headers = ["tally.h"]
```

```c,project=tally,file=c/tally.h
#include <stddef.h>

typedef struct tally tally;

tally *tally_new(int start);
void tally_add(tally *t, int amount);
int tally_get(const tally *t);
void tally_free(tally *t);
int tally_freed(void);
int byte_sum(const unsigned char *data, size_t n);
```

```c,project=tally,file=c/tally.c
#include "tally.h"
#include <stdlib.h>

struct tally { int value; };
static int freed;

tally *tally_new(int start) {
    tally *t = (tally *)malloc(sizeof *t);
    if (t) t->value = start;
    return t;
}
void tally_add(tally *t, int amount) { t->value += amount; }
int tally_get(const tally *t) { return t->value; }
void tally_free(tally *t) { freed++; free(t); }
int tally_freed(void) { return freed; }
int byte_sum(const unsigned char *data, size_t n) {
    int sum = 0;
    size_t i;
    for (i = 0; i < n; i++) sum += data[i];
    return sum;
}
```

```zamak,project=tally,file=src/main.zm
# Tally is C's tally *; when it goes out of scope, Zamak calls tally_free exactly once.
extern type Tally: "tally", close = tally_free

extern fn tally_new(start: i64) -> Option[Tally]
extern fn tally_add(t: &Tally, amount: i64)
extern fn tally_get(t: &Tally) -> i64
extern fn tally_freed() -> i64
extern fn byte_sum(data: &bytes, n: i64) -> i64

# When C returns NULL, match takes the None branch.
fn count() -> i64:
    match tally_new(10):
        Some(t):
            tally_add(t, 5)
            return tally_get(t)
        None: return -1

fn main():
    println(count())
    println(tally_freed())
    let data = b"abc"
    println(byte_sum(data, len(data)))
```

```text
15
1
294
```

- `extern type Tally: "tally", close = tally_free` declares the handle type. A handle has one owner, like a string: you can move it, borrow it, and keep it in lists and structs, but you can't copy it or build one in Zamak. Without `close`, Zamak doesn't free it.
- Declare a C function that returns a handle as returning `Option[Tally]`. When C returns NULL, you get `Option.None`.
- Write handle parameters as `&Tally`. When C hands out a handle through an output parameter (`int open(const char *name, tally **out)`), write the parameter as `&mut Option[Tally]`.
- `&bytes` and `&mut bytes` reach C as a pointer to the first byte; pass the length separately.
- `headers` lists the library's header files. With it, Zamak calls functions through the library's own prototypes, so the C compiler converts parameters such as `int` and `size_t`, and the C code doesn't need `int64_t`.

The repository's `examples/ffi_handles` also shows an output parameter and C writing into a `bytes` buffer.

## Safety at the boundary

Zamak's safety guarantees stop at the `extern fn` line: the compiler can't check what C code does. Keep C wrappers small, convert everything to the simple types above, and let Zamak code handle the rest.

## Summary

- `extern fn name(...)` calls a C function; the C standard library needs no setup.
- Parameters and results are `i64`, `f64`, `bool`, `&string`, `string`, `&bytes`, or C handles.
- `extern type` declares a C handle, and Zamak calls its `close` function when the handle goes out of scope.
- List C, C++, and Objective-C files and libraries in the `[native]` section of `zam.toml`.
