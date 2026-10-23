# 字符串

`string` 存放 UTF-8 文本。与 Python 一样，长度、下标和切片按字符计算，而不是按字节，所以任何语言的文字都能按你预期的方式处理。

## 书写字符串

字符串字面量用双引号。转义序列以反斜杠开头：

```zamak
fn main():
    println("Tab:\tafter tab")
    println("Quote: \"hi\"")
    println("Backslash: \\")
    println("Two\nlines")
```

```text
Tab:	after tab
Quote: "hi"
Backslash: \
Two
lines
```

转义序列有 `\n`（换行）、`\r`（回车）、`\t`（制表符）、`\0`、`\"`、`\\`，以及按码点写字符的 `\xNN`、`\uXXXX` 和 `\UXXXXXXXX`，与 Python 相同。

写正则表达式、Windows 路径或 JSON 时，反斜杠和引号会很多。这时用原始字符串和三引号字符串：

```zamak
fn main():
    println(r"C:\new\{folder}")
    println("""say "hi" to {1 + 1} people""")
    println(r"""{"name": "Ada"}""")
```

```text
C:\new\{folder}
say "hi" to 2 people
{"name": "Ada"}
```

- `r"..."` 是原始字符串：反斜杠原样保留，也不做插值。
- `"""..."""` 里可以直接写双引号；`r"""..."""` 两样都有。三引号字符串目前要写在同一行里。

## 插值

字符串里的花括号会插入表达式的值。`f` 前缀可写可不写；Zamak 接受它，是为了让 Python 写法读起来自然：

```zamak
fn main():
    let name = "Ada"
    let items: [i64] = [3, 4]
    println("Hello, {name}!")
    println(f"{name} has {len(items)} items, total {items[0] + items[1]}")
    println("Use {{ and }} for literal braces")
```

```text
Hello, Ada!
Ada has 2 items, total 7
Use { and } for literal braces
```

插值的表达式里不能有字符串字面量或花括号。需要时先把值绑定到一个名字上。

### 格式说明

表达式后面紧跟一个冒号，就按说明打印这个值。说明写成 `[[填充]对齐][宽度][.小数位]`：对齐是 `<`（左）、`>`（右）或 `^`（居中），宽度是最少占几列，小数位只对 `f64` 有意义。前面写 `0` 表示补零、靠右。

```zamak
fn main():
    let ratio = 2.0 / 3.0
    let name = "Ada"
    println(f"{ratio:.3}")
    println(f"{ratio:8.2}|")
    println(f"[{name:>6}]")
    println(f"[{name:^6}]")
    println(f"[{name:<6}]")
    println(f"[{7:03}]")
```

```text
0.667
    0.67|
[   Ada]
[ Ada  ]
[Ada   ]
[007]
```

不写冒号时按值本来的样子打印。小数位只对 `f64` 有意义：给 `i64` 写精度会直接报错，而不是悄悄四舍五入；确实要那样就先写 `f64(n)` 转过去。

## 长度、下标与切片

```zamak
fn main():
    let word = "naïve 你好"
    println(len(word))
    println(word.byte_len())
    println(word[0])
    println(word[-1])
    println(word[2:5])
    println(word[:5])
    println(word[6:])
```

```text
8
13
n
好
ïve
naïve
你好
```

- `len(s)` 统计字符数，`s.byte_len()` 统计 UTF-8 字节数。
- `s[i]` 是位置 `i` 上的字符，结果是只含一个字符的字符串。负数下标从末尾数起。
- `s[a:b]` 复制从 `a` 到 `b`（不含 `b`）的字符。省略一端表示从开头开始或一直到结尾。越界的边界会被收拢，与 Python 相同。

## 遍历字符

```zamak
fn main():
    for ch in "héllo":
        print(ch)
    for i, ch in "abc":
        println(f"{i}={ch}")
```

```text
h
é
l
l
o
0=a
1=b
2=c
```

## 拼接与比较

`+` 把两个字符串拼成一个新字符串。`==` 和 `<` 比较内容：

```zamak
fn main():
    let first = "Grace"
    let last = "Hopper"
    let full = first + " " + last
    println(full)
    println(full == "Grace Hopper")
    println("apple" < "banana")
    let mut log = ""
    for i in range(3):
        log += f"[{i}]"
    println(log)
```

```text
Grace Hopper
true
true
[0][1][2]
```

字符串和数字不能用 `+` 拼接，Zamak 会报类型错误。改用插值：`f"n = {n}"`。

## 字符串方法

字符串函数可以用方法的形式调用。它们返回新字符串，从不修改原字符串：

```zamak
fn main():
    let text = "  Hello, World  "
    println(text.strip())
    println(text.lower())
    println(text.upper().strip())
    println(text.replace("World", "Zamak"))
    println(text.contains("World"))
    println(text.strip().startswith("Hello"))
    println(text.find("World"))
    println("ab".repeat(3))
```

```text
Hello, World
  hello, world  
HELLO, WORLD
  Hello, Zamak  
true
true
9
ababab
```

`find` 返回第一个匹配的字符位置，找不到时返回 `-1`。Python 的方法名都能用：`strip`、`lstrip`、`rstrip`、`startswith`、`endswith`、`isdigit`、`isalpha`、`isspace`、`ljust` 和 `rjust`。

## 切分与连接

```zamak
fn main():
    let csv = "red,green,,blue"
    let parts = csv.split(",")
    println(parts)
    println(len(parts))
    println("one  two\tthree".words())
    println(" | ".join(parts))
    println("line 1\nline 2\n".lines())
```

```text
["red", "green", "", "blue"]
4
["one", "two", "three"]
red | green |  | blue
["line 1", "line 2"]
```

- `split(sep)` 保留空字段，与 Python 相同。
- `words()` 按连续的空白切分，丢掉空的片段。
- `lines()` 按换行切分，去掉行尾的 `\r`，末尾的换行之后不会多出一个空行。
- `sep.join(list)` 连接一个字符串列表。

## 解析数字

`int(s)` 和 `float(s)` 解析文本。解析可能失败，所以必须处理结果；[处理错误](errors.md)一章会讲 `?` 和 `catch`：

```zamak
fn main() -> !string:
    let n = int("42")?
    let x = float("2.5")?
    println(n + 1)
    println(x * 2)
```

```text
43
5.0
```

## 字节：bytes

文件内容、网络数据和哈希值这类原始字节，用 `bytes` 类型保存，写法与 Python 相同：`b"..."` 是字面量，下标读出来是 0 到 255 的整数。

```zamak
fn main() -> !string:
    let mut data = b"GIF\x89"
    println(data)
    println(data[3])
    data.push(10)
    println(len(data))
    let text = "你好".encode()
    println(text)
    println(text.decode()?)
```

```text
b"GIF\x89"
137
5
b"\xe4\xbd\xa0\xe5\xa5\xbd"
你好
```

- `b"..."` 里只能写 ASCII 字符，其他字节写成 `\xNN`。
- `s.encode()` 把字符串变成 UTF-8 字节；`b.decode()` 反过来，字节不是合法的 UTF-8 时返回错误。
- `bytes` 和列表的用法一样：`len`、下标、切片、`push`、`extend`、`for b in data`。写入 0 到 255 以外的数会让程序停下。
- `b.hex()` 得到十六进制文本；`std/bytes` 和 `std/fs` 里还有更多函数，见[标准库速览](standard-library.md)。

## 小结

- 字符串是 UTF-8，`len`、下标和切片按字符计算。
- `"{expr}"` 插入值，`+` 拼接字符串。
- 字符串函数可以当方法用：`s.strip()`、`s.split(",")`、`s.find("x")`。
- `int(s)` 和 `float(s)` 解析数字，可能失败。
- `bytes` 保存原始字节：`b"..."`、`s.encode()`、`b.decode()`。
