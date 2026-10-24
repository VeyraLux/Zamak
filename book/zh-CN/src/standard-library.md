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
- `env.program()` 是程序的文件名，不带目录和 `.exe`。读命令行选项见 [std/cli](#命令行参数stdcli)。

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

## 日期与时间：std/datetime

`std/datetime` 的用法参考 Python 的 `datetime`：

```zamak
use std/datetime

fn main() -> !string:
    let due = datetime.date_from_iso("2026-11-01")?
    println(due.add_days(30).isoformat())
    println(due.weekday())
    println(due.strftime("%A, %d %B %Y"))
    let start = datetime.date(2026, 1, 1)?
    println(due.days_since(start))
    let t = datetime.datetime_from_iso("2026-11-01T08:30:00+08:00")?
    println(t.to_utc().isoformat())
```

```text
2026-12-01
6
Sunday, 01 November 2026
304
2026-11-01T00:30:00+00:00
```

- `Date` 有 `year`、`month`、`day`。`datetime.today()` 是今天的日期，`datetime.date(y, m, d)` 会检查数值是否合法。
- `DateTime` 还有 `hour`、`minute`、`second`、`microsecond` 和 `offset`（比 UTC 快多少秒）。`datetime.now()` 用本地时区，`datetime.utcnow()` 用 UTC，`datetime.from_timestamp(t)` 把 `time.time()` 的值转成时间。
- `add_days(n)`、`days_since(other)`、`add_seconds(s)`、`seconds_since(other)` 做加减。`weekday()` 星期一是 0。
- `isoformat()`、`strftime(format)`、`date_from_iso(text)`、`datetime_from_iso(text)` 在时间和文字之间转换。
- 日期和时间只含数字，赋值时像数字一样复制。

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

- `Json` 是枚举：`Null`、`Bool`、`Int`、`Float`、`Str`、`Array`、`Object`，可以直接构造，例如 `json.Json.Int(1)`。
- 取值：`get(key)`、`item(index)`、`as_str()`、`as_f64()`、`as_i64()`、`as_bool()`、`is_null()`、`len()`、`keys()`。
- 数字和 Python 一样分两种：没有小数点和指数的是 `Int`（`i64`），其余是 `Float`（`f64`）。写出时原样保留，`1` 还是 `1`，`1.0` 还是 `1.0`。超出 `i64` 的整数报错，不会变成四舍五入的浮点数。`as_f64()` 也接受 `Int`；`as_i64()` 只接受 `Int`，`1.5` 和 `1.0` 都报错。
- `json.dumps(value, indent=2)` 分行缩进。

### 结构体与 JSON 互相转换

`json.encode(value)` 把任何结构体、列表、字典等值写成 JSON 文字，`json.decode(text)` 读回来。要读成什么类型，写在标注里：

```zamak
use std/json

struct Task:
    id: i64
    text: string
    tags: [string]
    due: Option[string]

fn main() -> !string:
    let tasks = [Task(id: 1, text: "write", tags: ["work"], due: Option.None)]
    let text = json.encode(tasks)
    println(text)
    let back: [Task] = json.decode(text)?
    println(back[0].text)
    let bad: [Task] = json.decode(r"""[{"id": 2, "text": "x", "tags": [7]}]""") catch e:
        println(e)
        return
```

```text
[{"id": 1, "text": "write", "tags": ["work"], "due": null}]
write
[0].tags[0]: the JSON value is not a string
```

- 结构体是对象，键就是字段名；列表是数组，键为 `string` 或 `i64` 的字典是对象，`Option` 是 `null` 或值。`Option` 字段缺少这个键时读成 `None`。
- 没有数据的枚举变体是字符串（如 `"Empty"`），带数据的是带 `"type"` 键的对象（如 `{"type": "Circle", "r": 1.5}`）。
- 文字和类型对不上时，错误里带着位置：`[0].tags[0]`、`.shape.h: missing`。
- `json.to_json(value)` 与 `json.from_json(j)` 在值和 `Json` 之间转换（不经过文字）。
- 所有字符串都会做 `{...}` 插值，所以 JSON 字面量要写成原始字符串：`r"""{"id": 1}"""`。

## 数据库：std/sqlite

`std/sqlite` 就是 SQLite，已经编进了 `zam`，不用另装。`sqlite.connect(path)` 打开数据库文件，没有就新建；`":memory:"` 打开一个只在内存里的数据库。连接离开作用域时自动关闭。

```zamak
use std/sqlite

struct Note:
    id: i64
    title: string
    pinned: bool
    body: Option[string]

struct Draft:
    title: string
    pinned: bool

fn main() -> !string:
    let db = sqlite.connect(":memory:")?
    db.execute_script("CREATE TABLE notes (id INTEGER PRIMARY KEY, title TEXT NOT NULL, pinned BOOLEAN NOT NULL DEFAULT 0, body TEXT)")?
    db.execute("INSERT INTO notes (title, body) VALUES (?, ?)", [sqlite.text("milk"), sqlite.text("2 L")])?
    db.execute_named("INSERT INTO notes (title, pinned) VALUES (:title, :pinned)", &Draft(title: "rent", pinned: true))?
    let notes: [Note] = db.query("SELECT * FROM notes ORDER BY id", [])?
    for note in notes:
        println(f"{note.id} {note.title} {note.pinned} {note.body}")
```

```text
1 milk false Option.Some("2 L")
2 rent true Option.None
```

| 方法 | 作用 |
| --- | --- |
| `db.execute(sql, params)` | 执行一条带 `?` 参数的语句，返回改动的行数。 |
| `db.execute_named(sql, &value)` | 执行一条语句，每个 `:name` 参数取结构体或字典里同名字段的值。 |
| `db.execute_script(sql)` | 执行几条用分号隔开的语句，不带参数。 |
| `let rows: [T] = db.query(sql, params)?` | 把每一行读成 `T`，列名对应字段名。 |
| `db.query_rows(sql, params)` | 每一行是按列顺序排的 `sqlite.Value` 列表。 |
| `db.query_json(sql, params)` | 结果是 `Json` 数组，每一行是一个对象。 |
| `db.last_id()`、`db.changes()` | 最近一条 `INSERT` 的行号，以及最近一条语句改动的行数。 |

参数用 `sqlite.integer(n)`、`sqlite.real(x)`、`sqlite.text(s)`、`sqlite.null()` 写，名字和 SQLite 的列类型一样。

存进去什么，读出来就是什么；放不进去的直接报错，绝不悄悄改值：

- 整数保留 `i64` 的每一位。
- `bool` 存成 `0` 或 `1`；声明成 `BOOLEAN` 的列读回 `true` 或 `false`。
- `NULL` 读成 `Option.None`；字段不是 `Option` 时报错。
- 列表或结构体字段放不进一列。把 `json.encode(...)` 存进字符串字段，或者另建一张表。
- `execute` 和各个查询方法只执行一条语句；要执行几条，用 `execute_script`。
- 另一个连接正在写时，语句最多等 5 秒，之后才报告数据库被锁住。

## 命令行参数：std/cli

`std/cli` 把命令行选项读进一个结构体。每个字段就是一个选项，字段的类型决定怎么读，不用再另外声明：

```zamak
use std/cli

struct Options:
    all: bool
    due: Option[string]
    tag: [string]
    count: i64
    args: [string]

fn main() -> !string:
    let given: [string] = ["--all", "--tag", "work", "--tag=home", "--count", "3", "report.txt"]
    let o: Options = cli.parse(given)?
    println(f"{o.all} {o.due} {o.tag} {o.count} {o.args}")
    let wrong: [string] = ["--count", "three"]
    let bad: Options = cli.parse(wrong) catch e:
        println(e)
        return
```

```text
true Option.None ["work", "home"] 3 ["report.txt"]
--count needs a whole number, not three
```

| 字段类型 | 命令行上的写法 |
| --- | --- |
| `bool` | `--all` 是开关：写了就是 `true`。 |
| `i64`、`f64`、`string` | `--count 3` 或 `--count=3`。必须写，类型不对时报错。 |
| `Option[...]` | 可以不写，没写时是 `None`。 |
| `[...]` | 可以写几次：`--tag a --tag b`。 |
| `args: [...]` | 收集位置参数。没有这个字段时，出现位置参数就报错。`--` 之后的都算位置参数。 |

- 字段名里的下划线在命令行上写成横线：`dry_run` 对应 `--dry-run`。
- 不认识的选项、缺了值、同一个选项写两次，都会报错并指出是哪个选项。
- `-h` 和 `--help` 打印所有选项后退出。
- 在程序里读真正的命令行参数，写 `let o: Options = cli.parse_or_exit(env.args())`。出错时它打印错误和提示，以状态 2 退出，与 Python 的 `argparse` 一样。
- 有子命令时，先自己读第一个参数，再解析剩下的：`cli.parse(args[1:])`。`examples/apps/todo` 就是这样做的。
- `env.program()` 是程序的文件名，不带目录和 `.exe`。

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

## 网络：std/net 和 std/http

`std/http` 发 HTTP 请求，形状参考 Python 的 `requests`：

```zamak,ignore
use std/http

fn main() -> !string:
    let r = http.get("https://example.com/")?
    println(r.status)
    println(r.headers["content-type"])
    println(r.text()?)
    let reply = http.post("http://localhost:8000/notes", "buy milk")?
    println(reply.ok())
```

| 函数 | 作用 |
| --- | --- |
| `http.get(url, timeout=30.0)` | GET 请求，返回 `Response`。 |
| `http.post(url, body, content_type="text/plain; charset=utf-8")`、`http.post_json(url, value)` | POST 文字或 JSON。 |
| `http.request(method, url, body, headers)` | 任意方法，自己给请求头与 `bytes` 请求体。 |
| `http.urljoin(base, link)` | 按网页地址补全链接，与 Python 的 `urllib.parse.urljoin` 相同。 |
| `Response` | `status`、`reason`、`headers`（键是小写）、`body`，以及 `text()`、`json()`、`ok()`。 |

服务端：`http.serve(host, port, handler)` 一直处理请求，每个请求交给 `handler`，它返回的 `Response` 发回给客户端。几个线程同时处理请求，一个慢请求不会挡住别的请求：

```zamak,ignore
use std/http

fn hello(req: &http.Request) -> http.Response:
    http.respond(200, f"hello from {req.path}")

fn main() -> !string:
    http.serve("127.0.0.1", 8000, hello)?
```

请求会跟随重定向（301、302、303、307、308，最多 10 次），与 `requests` 相同；传 `follow=false` 可以拿到重定向本身。

`http.respond(status, text)` 与 `http.respond_json(status, value)` 构造响应，`value` 可以是 `Json`，也可以是结构体、列表等任何能写成 JSON 的值。`Request` 有 `method`、`path`、`query`、`headers`、`body`，还有 `text()`、`json()`、解码好的查询参数 `args()`，以及取其中一个的 `arg(name, default="")`。

- `handler` 同时在几个线程里运行，所以它不能捕获局部变量，传一个有名字的函数。用 `workers=n` 指定线程数；默认是处理器个数的两倍，至少 8 个。
- `http.quote(text)`、`http.unquote(text)`、`http.parse_query(query)` 对应 Python 的 `urllib.parse.quote`、`unquote`、`parse_qs`。
- [`examples/apps/notes`](https://github.com/VeyraLux/Zamak/tree/main/examples/apps/notes) 是一个完整的 REST 服务，笔记存在 SQLite 里。

`std/net` 是更底层的 TCP：`net.connect(host, port)` 得到连接，`net.listen(host, port)` 得到监听器，连接有 `send`、`recv`、`recv_all`、`set_timeout`。连接与监听器离开作用域时自动关闭。

### HTTPS 与 TLS

`https://` 开头的地址自动走 TLS，并校验服务器证书：证书要由受信任的根证书签发，名字要和主机名一致，否则请求失败，错误信息会说明原因。根证书来自系统（Windows 的证书库，Linux 和 macOS 的常见证书文件）；设置环境变量 `SSL_CERT_FILE` 可以改用你自己的 PEM 文件。

服务端给 `serve` 传上证书和私钥（都是 PEM 文件）就是 HTTPS：

```zamak,ignore
http.serve("0.0.0.0", 8443, hello, cert_file="cert.pem", key_file="key.pem")?
```

在 `std/net` 里，`conn.start_tls(host)` 在已有的连接上开启 TLS（客户端），`listener.accept_tls(cert_file, key_file)` 接受一条 TLS 连接。之后照常用 `send` 和 `recv`。

TLS 由 [Mbed TLS](https://github.com/Mbed-TLS/mbedtls)（Apache-2.0）实现，`zam` 自带它的源码。程序第一次用到 `std/net` 或 `std/http` 时，`zam` 把它编译进缓存，要多花一些时间（通常十几秒）；之后直接复用。分发这样的程序时，请附上 Mbed TLS 的许可证。

## 线程与队列：std/thread

`thread.spawn(work, 参数...)` 在新线程里运行 `work`，写法同 Python 的 `executor.submit(work, *args)`。参数按值移交给新线程，之后原来的线程不能再用它们；`join()` 等线程结束并取回返回值：

```zamak,ignore
use std/thread

fn squares(start: i64, stop: i64) -> i64:
    let mut total = 0
    for i in range(start, stop):
        total += i * i
    total

fn main() -> !string:
    let mut tasks: [thread.Task[i64]] = []
    for k in range(4):
        tasks.push(thread.spawn(squares, k * 1000, (k + 1) * 1000))
    let mut total = 0
    for task in &tasks:
        total += task.join()?
    println(total)
```

`work` 有返回值时 `spawn` 返回 `Task[R]`，没有时返回 `Thread`。`work` 失败时，`join()` 以同样的错误失败。没有 `join` 的线程在句柄离开作用域时等它结束。

线程之间不共享可以修改的数据，所以不会有数据竞争。要传递数据就用队列 `Queue[T]`，它的形状参考 Python 的 `queue.Queue`：

| 写法 | 作用 |
| --- | --- |
| `let q: thread.Queue[string] = thread.queue()` | 新建队列，元素类型写在标注里。 |
| `q.put(item)?` | 放进一个元素，所有权一起移交。队列已关闭时失败。 |
| `q.get()` | 取出最早的元素；队列空时等待。关闭并取空之后返回 `Option.None`。 |
| `q.close()` | 关闭队列：不能再放，等待中的 `get` 立即醒来。 |
| `q.clone()` | 同一个队列的另一个句柄，可以交给别的线程。 |

```zamak,ignore
fn produce(q: thread.Queue[string], count: i64) -> !string:
    for i in range(count):
        q.put(f"item {i}")?
    q.close()

fn main() -> !string:
    let q: thread.Queue[string] = thread.queue()
    let worker = thread.spawn(produce, q.clone(), 3)
    while true:
        match q.get():
            Some(item): println(item)
            None: break
    worker.join()?
```

几点规则：

- 线程函数要直接写函数名，参数不能是借用（`&T`），因为新线程可能比原来线程里的值活得更久。
- 线程函数的错误要是字符串（`-> i64!string`）。
- `thread.cpu_count()` 给出逻辑处理器个数。`random` 的状态每个线程一份，`random.seed` 只影响调用它的线程。
- 多个线程的 `println` 按整行输出，不会互相插进去。

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
- `std/sqlite` 把数据存进 SQLite 数据库，查询结果直接读成结构体。
- `std/thread` 启动线程（`thread.spawn`），线程之间用 `Queue` 传递数据。
- 可能失败的函数返回错误，用 `?` 或 `catch` 处理。
