# 标准库速览

标准库覆盖了命令行程序的日常需要：输入输出、文件、参数、数学、时间和随机数。用 `use std/<name>` 导入模块，用 `name.function(...)` 调用它的函数。

## 内置函数

下面这些不用导入：

| 函数 | 作用 |
| --- | --- |
| `println(x)`、`print(x)` | 打印一个值并换行。 |
| `len(x)` | 字符串的字符数，或列表、字典的元素数。 |
| `str(x)` | 把值转成文字，与 `"{x}"` 相同。 |
| `int(x)`、`float(x)` | 转换数字，或解析文字（可能失败）。 |
| `abs(x)`、`min(a, b)`、`max(a, b)` | 绝对值、较小值、较大值。 |
| `min(xs)`、`max(xs)`、`sum(xs)`、`sorted(xs)` | 统计或排序一个列表。 |
| `copy(&x)` | 深复制。 |

## 读取输入：std/io

`io.read_line` 每次读一行，输入结束时返回 `false`。下面的程序给读到的每一行编号：

```zamak,check
use std/io

fn main() -> !string:
    let mut line = ""
    let mut n = 0
    while io.read_line(&mut line)?:
        n += 1
        println(f"{n}: {line}")
```

```sh
printf "alpha\nbeta\n" | zam run number.zm
```

```text
1: alpha
2: beta
```

- `io.read_all()` 把标准输入全部读成一个字符串。
- `io.print(text)` 打印但不换行，`io.eprintln(text)` 向标准错误打印一行。

## 参数与环境变量：std/env

```zamak,check
use std/env

fn main() -> !string:
    let args = env.args()
    if len(args) == 0:
        usage()
        env.exit(2)
    let name = env.arg(0)?
    let home = env.var("HOME") catch e: "unknown"
    println(f"hello, {name}; home is {home}")

fn usage():
    println("usage: greet NAME")
```

- `env.args()` 返回参数列表，不含程序名。`env.arg(i)` 返回一个参数，参数不存在时失败。
- `env.var(name)` 读取环境变量，没有设置时失败。
- `env.exit(code)` 以 0 到 255 之间的状态码结束程序。

## 文件与路径：std/fs 和 std/path

```zamak
use std/fs
use std/path

fn main() -> !string:
    fs.mkdir("notes")?
    let file = path.join("notes", "todo.txt")
    fs.write(file, "buy milk\n")?
    fs.append(file, "write tests\n")?
    println(fs.read(file)?.lines())
    println(fs.exists(file))
    println(fs.list("notes")?)
    println(f"{path.name(file)} {path.stem(file)} {path.suffix(file)}")
    fs.remove(file)?
    fs.remove("notes")?
    println(fs.exists("notes"))
```

```text
["buy milk", "write tests"]
true
["todo.txt"]
todo.txt todo .txt
false
```

| 函数 | 作用 |
| --- | --- |
| `fs.read(p)`、`fs.write(p, text)`、`fs.append(p, text)` | 读写整个文件。 |
| `fs.exists(p)`、`fs.is_file(p)`、`fs.is_dir(p)`、`fs.size(p)` | 查看路径的情况。 |
| `fs.list(dir)` | 目录里的名字，已排序。 |
| `fs.mkdir(p)` | 创建目录及其上级目录。 |
| `fs.remove(p)`、`fs.rename(a, b)` | 删除文件或空目录；重命名。 |
| `path.join`、`path.parent`、`path.name`、`path.stem`、`path.suffix` | 以文字方式拼接和拆分路径。 |

所有访问文件系统的函数都可能失败，所以它们会返回错误，用 `?` 或 `catch` 处理。

## 数学与格式化：std/math 和 std/fmt

`std/math` 提供 `sqrt`、`pow`、`exp`、`log`、`log10`、三角函数、`floor`、`ceil`、`round`、`trunc`，以及常量 `math.pi`、`math.e` 和 `math.inf`。`std/fmt` 提供 `fmt.fixed(x, digits)`：

```zamak
use std/math
use std/fmt

fn main():
    let r = 2.5
    println(fmt.fixed(math.pi * r * r, 2))
    println(math.floor(-2.5))
    println(math.ceil(-2.5))
```

```text
19.63
-3.0
-2.0
```

## 时间与随机数：std/time 和 std/random

```zamak
use std/time
use std/random

fn main():
    let start = time.monotonic()
    random.seed(7)
    let first = random.randint(1, 6)
    random.seed(7)
    let again = random.randint(1, 6)
    println(first == again)
    println(first >= 1 and first <= 6)
    println(time.iso_utc(0))
    println(time.monotonic() >= start)
```

```text
true
true
1970-01-01T00:00:00Z
true
```

- `time.time()` 是自 1970 年以来的秒数，`time.monotonic()` 用来测量时长，`time.sleep(seconds)` 暂停。`time.iso_utc(t)` 和 `time.iso_local(t)` 格式化时间。
- `random.randint(a, b)` 包含两端，`random.randrange(a, b)` 不包含 `b`。`random.random()` 返回 [0, 1) 之间的 `f64`，`random.uniform(a, b)` 返回 [a, b) 之间的值。`random.seed(n)` 让随机序列可以重现，写测试时很有用。

## 字符串：std/string

每个字符串函数都可以当方法用，所以很少需要导入 `std/string`。常用的见[字符串](strings.md)一章，完整列表包括：

- 测量与比较：`len`、`byte_len`、`starts_with`、`ends_with`、`contains`、`count`、`find`、`find_from`。
- 变换：`trim`、`lower`、`upper`、`replace`、`repeat`、`pad_left`、`pad_right`、`slice`。
- 切分与连接：`split`、`words`、`lines`、`join`、`chars`。
- 转换：`parse_i64`、`parse_f64`、`codepoints`、`from_code`、`bytes`、`from_bytes`。

## 小结

- `len`、`int`、`sorted` 这样的内置函数不用导入。
- `std/io`、`std/env`、`std/fs`、`std/path` 处理输入、参数和文件。
- `std/math`、`std/fmt`、`std/time`、`std/random` 处理数字、格式、时钟和随机数。
- 可能失败的函数返回错误，用 `?` 或 `catch` 处理。
