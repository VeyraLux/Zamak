# 语法总览

一页看清 Zamak 现在支持什么。每一行都链到详细章节；想看完整的行为说明和边界情况，点过去。

## 代码结构

块用缩进界定，不用花括号。块头（`fn`、`struct`、`if`、`while`、`for`、`match`、`catch`）以 `:` 结尾，下一行多缩进 4 个空格。空块写 `pass`。

```zamak,ignore
fn greet(name: string) -> string:
    if name == "":
        return "hello"
    return "hello, {name}"
```

块头后可以跟一条同行语句（`if n < 1: return n`）。缩进里有 Tab 会报错，`zam fmt` 能修。花括号只用于字典和插值。

详见[变量与类型](variables.md)、[控制流](control-flow.md)。

## 类型

| 类型 | 写法 | 说明 |
| --- | --- | --- |
| 整数 | `i64` | 溢出、除零在发布版也报错 |
| 浮点 | `f64` | IEEE 754，除以零终止程序 |
| 布尔 | `bool` | `and`、`or`、`not`，短路求值 |
| 字符串 | `string` | UTF-8，下标按字符算 |
| 字节 | `bytes` | `b"..."`，下标得 0–255 的整数 |
| 列表 | `[T]` | 可增长 |
| 字典 | `{K: V}` | 插入顺序，键是 `i64` 或 `string` |
| 集合 | `{K}` | `{1, 2}`，值是 `i64` 或 `string` |
| 元组 | `(i64, string)` | 多返回值 |
| 结构体 | `struct Point:` | 字段逐行声明 |
| 枚举 | `enum Shape:` | 变体可带载荷 |
| 函数 | `fn(i64) -> bool` | 可作值传递 |
| 错误 | `T!E`、`!E` | 见下 |

详见[数字与运算符](operators.md)、[列表、字典、集合与元组](collections.md)。

## 函数

```zamak,ignore
fn add(a: i64, b: i64) -> i64:
    return a + b

fn largest[T](items: &[T]) -> T:      # 泛型，每种类型各生成一份
    ...

fn parse(s: &string) -> i64!string:   # 可能失败
    ...
```

参数可以是值、借用 `&T`、可变借用 `&mut T`，可以有默认值 `port: i64 = 80`，调用时可以按名字传 `connect(host, timeout=1.5)`。省略返回类型表示没有返回值。函数体最后一条表达式可以作为返回值。

详见[函数](functions.md)。

## 变量与所有权

```zamak,ignore
let n = 1              # 不可变
let mut count = 0       # 可变
count += 1

let a = [1, 2, 3]
let b = a               # a 被移走，之后不能用
let c = copy(a)         # 深复制
```

数字、布尔复制；字符串、结构体、列表、字典移动。

详见[移动与复制](ownership.md)。

## 借用

借用只在当前函数内有效，不越过函数返回。

```zamak,ignore
fn total(xs: &[i64]) -> i64:     # 只读借用
    ...

fn push_all(xs: &mut [i64]):      # 可变借用
    ...
```

借用不能存进结构体，不能返回，不能跨线程。这是不用写生命周期标注的原因。

详见[借用](borrowing.md)。

## 结构体与方法

```zamak,ignore
struct Point:
    x: f64
    y: f64

    fn norm(&self) -> f64:
        return math.sqrt(self.x * self.x + self.y * self.y)

fn main():
    let p = Point(x: 3, y: 4)
    println(p.norm())        # 5.0
```

方法写在结构体的缩进块里，第一个参数是 `&self`、`&mut self` 或 `self`。没有 `self` 的是关联函数，用类型名调用：`Point.new(1, 2)`。

详见[结构体与方法](structs.md)。

## 枚举与 match

```zamak
use std/math

enum Shape:
    Circle(r: f64)
    Empty

fn area(s: &Shape) -> f64:
    return match s:
        Circle(r): math.pi * r * r
        Empty: 0.0

fn main():
    let c = Shape.Circle(2.0)
    let e = Shape.Empty
    println(area(&c))
    println(area(&e))
```

`match` 要用 cover 所有变体，或者以 `_` 结尾。内置 `Option[T]`，变体是 `Option.Some(v)` 和 `Option.None`。

详见[枚举、Option 与 match](enums.md)。

## 字符串

```zamak,ignore
let name = "world"
println("hello, {name}")          # 插值
println("x = {count + 1}")        # 花括号里可写表达式

let parts = "a,b,c".split(",")
println(parts.len())               # 3
println(name.upper())              # WORLD
```

字符串方法不用导入，直接调。支持 `strip`、`startswith`、`endswith` 这些 Python 名字。

详见[字符串](strings.md)。

## 列表与字典

```zamak,ignore
let mut xs: [i64] = [3, 1, 2]
xs.push(4)
xs.sort()
println(xs[0])          # 1
println(xs[-1])         # 4，负数从末尾数
println(xs.slice(1, 3)) // [2, 3]

let mut counts: {string: i64} = {}
counts["a"] = 1
println(counts.get("b", 0))   # 0
```

空列表、空字典要写类型标注。`for x in &xs` 只借用不移走。

集合写 `{1, 2, 3}`，类型 `{i64}`。

详见[列表、字典、集合与元组](collections.md)。

## 错误处理

```zamak,ignore
fn read(path: &string) -> string!string:
    return fs.read(path)?

fn main() -> !string:
    let text = read("a.txt")?
    println(text)
    return null
```

`T!E` 表示成功返回 `T`、失败返回 `E`。`fail expr` 立刻产生错误，`call?` 向下传递错误，`call catch e:` 就地处理。`main` 可以带错误类型，未捕获的错误写到 stderr 并以退出码 1 结束。

详见[处理错误](errors.md)。

## 函数值与闭包

```zamak,ignore
fn apply(f: fn(i64) -> i64, n: i64) -> i64:
    return f(n)

let double = fn(x: i64) -> i64: x * 2
println(apply(double, 5))     # 10
```

匿名函数可以读外层局部变量。捕获变量的闭包不能存进结构体、不能返回、不能活得比变量久，所以不需要垃圾回收。

详见[函数](functions.md)。

## 泛型

```zamak,ignore
fn largest[T](items: &[T]) -> T:
    ...

struct Pair[A, B]:
    first: A
    second: B

let p = Pair(i64, string)(first: 1, second: "one")
```

泛型目前按类型各生成一份代码（单态化）。调用处按实参推断类型参数。

详见[函数](functions.md)。

## 调用 C

```zamak,ignore
extern fn sqrt(x: f64) -> f64

fn main():
    println(sqrt(16.0))
```

`extern fn` 声明后用同名 C 函数。C 标准库开箱即用，其他库在 `zam.toml` 的 `[native]` 里配置源码、头文件和要链接的库。`extern type Db: "sqlite3", close = sqlite3_close` 声明 C 句柄，离开作用域时自动释放。

详见[调用 C 代码](c-libraries.md)。

## 标准库

| 模块 | 提供 |
| --- | --- |
| `std/string` | 分割、拼接、查找、裁剪、大小写 |
| `std/bytes` | 十六进制、全零字节串 |
| `std/io` | 不换行输出、读一行、读全部 |
| `std/fs` | 读写文本或字节、追加、列目录、创建、删除、改名 |
| `std/path` | join、父目录、文件名、扩展名 |
| `std/env` | 命令行参数、环境变量、退出 |
| `std/math` | sqrt、pow、三角、取整、常量 |
| `std/time` | 当前时间、单调时钟、休眠、ISO 格式 |
| `std/random` | 整数、浮点、设置种子 |
| `std/fmt` | 保留小数位 |
| `std/process` | 运行其他程序，取得输出与退出码 |
| `std/json` | 解析与生成 JSON |
| `std/re` | 正则表达式 |
| `std/hashlib`、`std/base64` | SHA-256、SHA-1、MD5、HMAC，base64 |

不用导入的 Python 内置：`len`、`abs`、`min`、`max`、`sum`、`sorted`、`str`、`int`、`float`、`print`、`println`。

详见[标准库速览](standard-library.md)。

## 目前不支持

- 线程与并发
- 网络
- 宏
- trait / 接口 / 继承
- 泛型特化、类型级编程
- 借用跨函数返回（要靠返回值移交所有权）

`zam check` 会为这些写法给出明确报错，不会静默放过。
