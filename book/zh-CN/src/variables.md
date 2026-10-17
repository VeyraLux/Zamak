# 变量与类型

Zamak 里每个值都有类型，编译器在程序运行之前检查类型。大多数地方它会替你推断出类型，所以代码和 Python 一样简短，错误却在编译时就能发现。

## 用 let 绑定值

`let` 给一个值起名字：

```zamak
fn main():
    let name = "Ada"
    let year = 1843
    println(f"{name}, {year}")
```

```text
Ada, 1843
```

`let` 绑定的值不能改变。再给它赋值是错误，`zam check` 会在程序运行之前报告：

```zamak,fails
fn main():
    let count = 1
    count = 2
```

```text
错误[ZM302]：count 不可变，不能赋值
 --> main.zm:3:5
  |
3 |     count = 2
  |     ^^^^^^^^^
  = 提示：赋值前先用 `let mut` 声明
```

## 让变量可变

变量需要改变时，加上 `mut`：

```zamak
fn main():
    let mut count = 1
    count = count + 1
    count += 10
    println(count)
```

```text
12
```

`+=`、`-=`、`*=`、`//=` 这样的复合赋值运算符就地更新变量。

把可变性写明，读代码的人就知道哪些值会变。看到 `let`，就知道这个值在这个代码块剩下的部分里保持不变。

## 每个名字只声明一次

同一个函数里，一个名字不能声明两次，在内层代码块里也不行。这条规则避免了一类常见错误：内层变量不小心遮住了外层的同名变量。

```zamak,fails
fn main():
    let total = 0
    let total = 5
```

```text
错误[ZM204]：total 重复绑定
 --> main.zm:3:5
  |
3 |     let total = 5
  |     ^^^^^^^^^^^^^
  = 提示：删掉或改名重复的声明
```

要改变值，就用 `let mut` 声明再赋值。要存放另一个值，就起一个更具体的名字，例如 `raw_total` 和 `total`。

## 基本类型

| 类型 | 取值 | 示例 |
| --- | --- | --- |
| `i64` | 64 位有符号整数 | `42`、`-7`、`1_000_000`、`0xFF` |
| `f64` | 64 位浮点数 | `3.14`、`2.0`、`1e-9` |
| `bool` | `true` 和 `false` | `true` |
| `string` | UTF-8 文本 | `"hello"`、`"你好"` |

Zamak 根据值推断类型。要写明类型，在名字后面加类型标注：

```zamak
fn main():
    let ratio: f64 = 1
    let ready: bool = false
    let label: string = "total"
    println(f"{label}: {ratio}, ready: {ready}")
```

```text
total: 1.0, ready: false
```

值本身看不出类型时，必须写标注，例如空列表或空字典：

```zamak
fn main():
    let mut scores: [i64] = []
    scores.push(90)
    println(scores)
```

```text
[90]
```

注意 `let ratio: f64 = 1` 把整数 `1` 变成了 `1.0`。凡是需要 `f64` 的地方都可以给整数，与 Python 一样。

## 数字的写法

整数是 64 位的，范围大约是 -9.2 × 10¹⁸ 到 9.2 × 10¹⁸。整数有几种写法：

```zamak
fn main():
    println(1_000_000)
    println(0xFF)
    println(0b1010)
    println(0o17)
    println('A')
```

```text
1000000
255
10
15
65
```

- 数字之间的下划线让长数字更好读。
- `0x`、`0b`、`0o` 开头分别是十六进制、二进制和八进制。
- 单引号里的一个字符，例如 `'A'`，是这个字符的整数码点。处理字节时很方便。

浮点数字面量的小数点两边都要有数字，或者带指数：写 `0.5`，不写 `.5`。

## 常量

常量是编译时就确定的值。在文件顶层（任何函数之外）声明，名字全部大写：

```zamak
const MAX_USERS = 100
const RATE: f64 = 0.25
const GREETING = "welcome"

fn main():
    println(f"{GREETING}: up to {MAX_USERS} users at {RATE}")
```

```text
welcome: up to 100 users at 0.25
```

常量的值可以由字面量、运算符和前面的常量组成，例如 `const SECONDS_PER_DAY = 24 * 60 * 60`。值由编译器算出，所以常量在运行时没有任何开销。

## 命名规则

- 变量、函数和模块用 `lower_snake_case`：`user_count`、`read_file`。
- 类型用 `UpperCamelCase`：`WordCount`、`Shape`。
- 常量用 `UPPER_SNAKE_CASE`：`MAX_USERS`。

编译器依赖其中一部分：以大写字母开头、后面跟 `(` 的名字总是创建结构体值，而不是调用函数。

## 小结

- `let` 绑定不能改变的值，`let mut` 绑定可以改变的值。
- 一个函数里每个名字只声明一次。
- 基本类型有 `i64`、`f64`、`bool` 和 `string`。Zamak 会推断类型，值看不出类型时写标注。
- `const` 在顶层声明编译时常量。
