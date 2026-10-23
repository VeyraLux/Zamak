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
| `fs.read_bytes(p)`、`fs.write_bytes(p, data)`、`fs.append_bytes(p, data)` | 按字节读写文件，见下文的 `bytes`。 |
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
- 转换：`parse_i64`、`parse_f64`、`codepoints`、`from_code`、`encode`、`bytes`、`from_bytes`。

## 字节：std/bytes

`bytes` 的方法 `decode` 和 `hex` 不用导入；创建字节串的函数在 `std/bytes` 里：

```zamak
use std/bytes

fn main() -> !string:
    let key = bytes.from_hex("de ad be ef")?
    println(key)
    println(key.hex())
    println(bytes.zeros(4))
```

```text
b"\xde\xad\xbe\xef"
deadbeef
b"\x00\x00\x00\x00"
```

| 函数 | 作用 |
| --- | --- |
| `b.decode()` | 按 UTF-8 解码成字符串，不合法时返回错误。 |
| `b.hex()` | 十六进制文本，每个字节两位小写数字。 |
| `bytes.from_hex(text)` | 解析十六进制文本，两位一组，组之间可以有空白。 |
| `bytes.zeros(n)` | n 个零字节。 |
| `fs.read_bytes(p)`、`fs.write_bytes(p, data)`、`fs.append_bytes(p, data)` | 按字节读写文件。 |

## 运行其他程序：std/process

`std/process` 的形状照 Python 的 `subprocess`。参数列表的第一项是程序（在 `PATH` 里找），其余原样传给它，不经过 shell：

```zamak,ignore
use std/process

fn main() -> !string:
    let result = process.run(["git", "status", "--short"])?
    println(result.code)
    println(result.stdout)
    let version = process.check_output(["git", "--version"])?
    println(version.trim())
    process.call(["git", "log", "-1"])?
```

| 函数 | 作用 |
| --- | --- |
| `process.run(args, input="", capture=true)` | 运行并等待，返回 `Completed`：`code`、`stdout`、`stderr`。 |
| `process.check_output(args, input="")` | 返回标准输出；退出码不是 0 时返回错误，错误里带标准错误的内容。 |
| `process.call(args)` | 输入输出都用当前终端，返回退出码。 |

程序找不到时返回错误。在 Windows 上，`.bat` 和 `.cmd` 脚本要经过 `cmd /c` 运行（同 Python）。

## JSON：std/json

`json.loads` 把文字解析成 `Json`，`json.dumps` 把 `Json` 写成文字。取值的方法都可能失败，可以用 `?` 一路接下去：

```zamak
use std/json

fn main() -> !string:
    let user = json.loads(r"""{"name": "Ada", "langs": ["Zamak", "C"]}""")?
    println(user.get("name")?.as_str()?)
    println(user.get("langs")?.item(0)?.as_str()?)
    println(json.dumps(user))
```

```text
Ada
Zamak
{"name": "Ada", "langs": ["Zamak", "C"]}
```

- `Json` 是枚举：`Null`、`Bool`、`Number`、`Str`、`Array`、`Object`，可以直接构造，例如 `json.Json.Number(1.0)`。
- 取值：`get(key)`、`item(index)`、`as_str()`、`as_f64()`、`as_i64()`、`as_bool()`、`is_null()`、`len()`、`keys()`。
- 数字按 `f64` 保存；整数值输出时不带小数点。`json.dumps(value, indent=2)` 分行缩进。

## 正则表达式：std/re

`std/re` 的函数与 Python 的 `re` 同名同参数，位置按字符计算。表达式里反斜杠多，用原始字符串 `r"..."` 写：

```zamak
use std/re

fn main() -> !string:
    let text = "order 12, order 345"
    println(re.findall(r"\d+", text)?)
    println(re.sub(r"order (\d+)", r"#\1", text)?)
    match re.search(r"(?P<id>\d{3})", text)?:
        Some(m): println(m.named("id")?)
        None: println("none")
```

```text
["12", "345"]
#12, #345
345
```

- 函数：`search`、`match`、`fullmatch`、`findall`、`finditer`、`sub`、`split`、`escape`、`compile`；标志 `re.IGNORECASE`、`re.MULTILINE`、`re.DOTALL`，也可以在表达式开头写 `(?i)`。
- 匹配结果 `Match`：`group(i)`、`named(name)`、`groups()`、`start(i)`、`end(i)`。
- 匹配时间与文字长度成线性关系，写坏的表达式也不会让程序卡死。代价是不支持反向引用 `\1` 和环视 `(?=...)`，写了会报错。

## 哈希与 base64：std/hashlib 和 std/base64

```zamak
use std/hashlib
use std/base64

fn main() -> !string:
    let data = "hello".encode()
    println(hashlib.sha256(data).hex())
    println(base64.b64encode(data))
    println(base64.b64decode("aGVsbG8=")?.decode()?)
```

```text
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
aGVsbG8=
hello
```

`std/hashlib` 有 `sha256`、`sha1`、`md5` 和 `hmac_sha256(key, data)`，结果是 `bytes`。`std/base64` 有 `b64encode`、`b64decode` 和 URL 安全的 `urlsafe_b64encode`、`urlsafe_b64decode`。

## 小结

- `len`、`int`、`sorted` 这样的内置函数不用导入。
- `std/io`、`std/env`、`std/fs`、`std/path` 处理输入、参数和文件；`std/bytes` 处理原始字节。
- `std/math`、`std/fmt`、`std/time`、`std/random` 处理数字、格式、时钟和随机数。
- `std/process`、`std/json`、`std/re`、`std/hashlib`、`std/base64` 运行程序、读写 JSON、匹配文字、计算哈希。
- 可能失败的函数返回错误，用 `?` 或 `catch` 处理。
