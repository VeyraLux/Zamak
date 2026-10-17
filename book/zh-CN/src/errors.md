# 处理错误

有些操作会因为程序控制不了的原因失败：文件不存在，或者用户在该填数字的地方输入了字母。Zamak 把这些失败写进函数的类型，让你不会忘记处理；同时提供简短的语法，把错误传下去或就地处理。

Zamak 没有异常。错误是函数返回的普通值。

## 可能失败的函数

可能失败的函数在结果类型后面写 `!` 和错误类型。`fail` 返回一个错误：

```zamak
fn parse_age(text: &string) -> i64!string:
    let n = int(text) catch e:
        fail f"not a number: {text}"
    if n < 0 or n > 150:
        fail f"out of range: {n}"
    n

fn main():
    for input in ["42", "abc", "200"]:
        let age = parse_age(input) catch e:
            println(f"error: {e}")
            continue
        println(f"age {age}")
```

```text
age 42
error: not a number: abc
error: out of range: 200
```

- `-> i64!string` 表示「返回一个 `i64`，或者以 `string` 消息失败」。不返回值但可能失败的函数写 `-> !string`。
- `fail value` 把错误返回给调用方。
- `call() catch e:` 在出错的地方就地处理错误，代码块里的 `e` 就是这个错误。调用的值要被使用时（例如 `let age = ...`），代码块必须用 `return`、`fail`、`break` 或 `continue` 离开，因为没有值可以继续往下用。

## 用 ? 把错误传下去

通常一个函数自己修不好错误，应该让调用方决定怎么办。在调用后面写 `?` 就是这个意思：成功时得到值，失败时从当前函数返回这个错误：

```zamak
fn parse_pair(text: &string) -> (i64, i64)!string:
    let parts = text.split(",")
    if len(parts) != 2:
        fail f"expected two numbers: {text}"
    let a = int(parts[0].strip())?
    let b = int(parts[1].strip())?
    (a, b)

fn main():
    for input in ["3, 4", "5", "6, x"]:
        let pair = parse_pair(input) catch e:
            println(f"skipped: {e}")
            continue
        println(pair)
```

```text
(3, 4)
skipped: expected two numbers: 5
skipped: invalid integer: x
```

`?` 只能用在以同一种错误类型失败的函数里。编译器会检查这一点，所以错误永远不会被悄悄丢掉。

## 处理错误后继续

失败的调用没有需要的值时，程序在 `catch` 块之后继续执行：

```zamak
use std/fs

fn main():
    fs.remove("does-not-exist.txt") catch e:
        println("nothing to remove")
    println("done")
```

```text
nothing to remove
done
```

## 让 main 失败

`main` 也可以失败。如果它返回错误，Zamak 把消息打印到标准错误，并以状态码 1 退出：

```zamak,check
use std/io

fn main() -> !string:
    let text = io.read_all()?
    let n = int(text.strip())?
    println(n * 2)
```

这是写小工具最简单的方式：到处用 `?`，出了任何问题都交给 `main` 报告。

## 带结构的错误

很多程序用一条 `string` 消息就够了。调用方需要区分不同的错误时，用枚举作错误类型，再用 `match` 区分：

```zamak
enum ConfigError:
    Missing(key: string)
    Invalid(key: string, value: string)

fn port(settings: &{string: string}) -> i64!ConfigError:
    if "port" not in settings:
        fail ConfigError.Missing("port")
    let text = &settings["port"]
    let n = int(text) catch e:
        fail ConfigError.Invalid("port", text)
    n

fn main():
    let good = {"port": "8080"}
    let bad = {"port": "eighty"}
    let empty: {string: string} = {}
    for settings in [good, bad, empty]:
        let p = port(settings) catch e:
            match e:
                Missing(key): println(f"missing {key}")
                Invalid(key, value): println(f"bad {key}: {value}")
            continue
        println(f"port {p}")
```

```text
port 8080
bad port: eighty
missing port
```

## 断言

`assert` 检查一个应该始终成立的条件。条件不成立时，程序停下并显示这个条件：

```zamak
fn average(xs: &[f64]) -> f64:
    assert len(xs) > 0, "average of an empty list"
    sum(xs) / len(xs)

fn main():
    println(average([1, 2, 6]))
```

```text
3.0
```

运行时可能出错的情况（例如输入不对）用错误处理；一旦发生就说明程序有 bug 的情况用断言。

## 小结

- `-> T!E` 返回 `T`，或者以 `E` 失败；`fail value` 返回错误。
- `?` 把错误交给调用方，`catch e:` 就地处理。
- `main` 可以写成 `-> !string`，出错时打印消息并以状态码 1 退出。
- 错误类型可以是字符串、结构体或枚举。
- `assert cond, "message"` 在假设不成立时让程序停下。
