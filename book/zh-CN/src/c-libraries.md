# 调用 C 代码

Zamak 编译成 C，所以可以直接调用 C 函数。用它来使用 C 库，或者复用你已有的 C 代码。

## 声明 C 函数

`extern fn` 声明一个用 C 写的函数。它没有函数体，调用会直接转到同名的 C 函数。C 标准库里的函数不需要任何设置就能用：

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

声明必须与 C 的签名一致。类型对应关系如下：

| Zamak | C |
| --- | --- |
| `i64` | `int64_t` |
| `f64` | `double` |
| `bool` | `int` |
| `&string` 参数 | `const char *`，以 `\0` 结尾 |
| `string` 返回值 | `const char *`；Zamak 会复制这段文字 |
| `&bytes`、`&mut bytes` 参数 | `const uint8_t *`、`uint8_t *` |
| C 句柄 `&Db` 参数 | 声明时写的指针类型，见下文 |

其他结构体不能直接跨过这道边界。用一个小的 C 包装函数，只接收和返回上面这些类型。

## 编译你自己的 C 文件

要让 C 文件和项目一起编译，在 `zam.toml` 的 `[native]` 部分列出它们：

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

/* Zamak 会复制返回的文字，所以用静态缓冲区就可以。 */
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

`[native]` 部分可以写：

- `sources`：和项目一起编译的 C 文件。
- `include`：查找头文件的目录。
- `headers`：库的头文件，例如 `["sqlite3.h"]`。见下文。
- `libraries`：要链接的库，例如 `["sqlite3"]` 对应 `-lsqlite3`。

包也可以带自己的 `[native]` 部分，所以一个 C 库只需包装一次，别的项目就能把它当作普通的 Zamak 包来用。

## C 句柄与缓冲区

很多 C 库通过指针交出一个对象，用完再调用另一个函数释放它，例如 SQLite 的 `sqlite3 *` 与 `sqlite3_close`。用 `extern type` 声明这样的句柄，Zamak 会替你释放它：

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
# Tally 是 C 的 tally *；离开作用域时 Zamak 调用 tally_free，正好一次。
extern type Tally: "tally", close = tally_free

extern fn tally_new(start: i64) -> Option[Tally]
extern fn tally_add(t: &Tally, amount: i64)
extern fn tally_get(t: &Tally) -> i64
extern fn tally_freed() -> i64
extern fn byte_sum(data: &bytes, n: i64) -> i64

# C 返回 NULL 时 match 走 None 分支。
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

- `extern type Tally: "tally", close = tally_free` 声明句柄类型。句柄和字符串一样只有一个所有者，可以移动、借用、放进列表和结构体，但不能复制，也不能在 Zamak 里构造。没有 `close` 时 Zamak 不负责释放。
- 返回句柄的 C 函数声明成返回 `Option[Tally]`：C 返回 NULL 时得到 `Option.None`。
- 参数写 `&Tally`。C 通过输出参数交出句柄时（`int open(const char *name, tally **out)`），参数写 `&mut Option[Tally]`。
- `&bytes` 和 `&mut bytes` 传给 C 的是指向第一个字节的指针，长度另外传。
- `headers` 列出库的头文件。有了它，Zamak 按库自己的原型调用，`int`、`size_t` 这类参数由 C 编译器转换，C 代码不用改成 `int64_t`。

仓库里的 `examples/ffi_handles` 还演示了输出参数和往 `bytes` 缓冲区写数据。

## 边界上的安全

Zamak 的安全保证止步于 `extern fn` 这一行：编译器无法检查 C 代码做了什么。让 C 包装函数尽量小，把一切都转换成上面这些简单类型，其余的交给 Zamak 代码处理。

## 小结

- `extern fn name(...)` 调用 C 函数；C 标准库不需要任何设置。
- 参数和返回值是 `i64`、`f64`、`bool`、`&string`、`string`、`&bytes` 或 C 句柄。
- `extern type` 声明 C 句柄，Zamak 在离开作用域时调用 `close` 释放它。
- 在 `zam.toml` 的 `[native]` 部分列出 C 文件和库。
