# 调用 C 代码

Zamak 编译成 C，所以可以直接调用 C 函数。用它来使用 C 库，或者复用你已有的 C 代码。

## 声明 C 函数

`extern fn` 声明一个用 C 写的函数。它没有函数体，调用会直接转到同名的 C 函数。C 标准库里的函数不需要任何设置就能用：

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

声明必须与 C 的签名一致。类型对应关系如下：

| Zamak | C |
| --- | --- |
| `i64` | `int64_t` |
| `f64` | `double` |
| `bool` | `int` |
| `&string` 参数 | `const char *`，以 `\0` 结尾 |
| `string` 返回值 | `const char *`；Zamak 会复制这段文字 |

指针和结构体不能直接跨过这道边界。用一个小的 C 包装函数，只接收和返回上面这些类型。

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
- `libraries`：要链接的库，例如 `["sqlite3"]` 对应 `-lsqlite3`。

包也可以带自己的 `[native]` 部分，所以一个 C 库只需包装一次，别的项目就能把它当作普通的 Zamak 包来用。

## 边界上的安全

Zamak 的安全保证止步于 `extern fn` 这一行：编译器无法检查 C 代码做了什么。让 C 包装函数尽量小，把一切都转换成上面这些简单类型，其余的交给 Zamak 代码处理。

## 小结

- `extern fn name(...)` 调用 C 函数；C 标准库不需要任何设置。
- 参数和返回值是 `i64`、`f64`、`bool`、`&string` 或 `string`。
- 在 `zam.toml` 的 `[native]` 部分列出 C 文件和库。
