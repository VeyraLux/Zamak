# 附录 A：写给 Python 程序员

Zamak 尽可能沿用 Python 的名字和规则。这篇附录列出哪些与 Python 相同、哪些写法不同、哪些是新东西。

## 与 Python 相同

- 缩进界定代码块，`#` 开始注释。
- `if`、`elif`、`else`、`while`、`for x in xs`、`for i in range(n)`、`break`、`continue` 和 `pass`。
- `and`、`or`、`not`、`in`、`not in` 和 `a if cond else b`。
- `/` 总是得到浮点数，`//` 是整除，`%` 的符号与除数相同，`**` 是乘方。
- `len`、`str`、`int`、`float`、`abs`、`min`、`max`、`sum` 和 `sorted`。
- 负数下标和切片：`xs[-1]`、`xs[1:3]`、`s[:5]`。
- `f"..."` 字符串，以及 `strip`、`split`、`join`、`startswith`、`replace`、`lower` 等字符串方法。
- 除了 `true` 和 `false`，也接受 `True` 和 `False`。

## 写法不同

| Python | Zamak | 原因 |
| --- | --- | --- |
| `def area(w, h):` | `fn area(w: f64, h: f64) -> f64:` | 有了类型，编译器能检查每一次调用。 |
| `x = 1` | `let x = 1` 或 `let mut x = 1` | `mut` 标出会改变的值。 |
| `class Shape(Protocol):`、抽象基类 | `interface Shape:` 列出方法 | 有这些方法的类型自动满足，不用继承。 |
| `class Point:` 加 `__init__` | `struct Point:` 列出字段，`Point(x: 1, y: 2)` | 字段只声明一次，不用写构造函数。 |
| `def norm(self):` | `fn norm(&self) -> f64:` | `&self`、`&mut self` 或 `self` 说明方法怎样使用这个值。 |
| `None` | `Option.None`，类型是 `Option[T]` | 不存在的值不会被误用。 |
| `raise ValueError("bad")` | `fail "bad"`，函数声明为 `-> T!string` | 失败是类型的一部分。 |
| `try: ... except ValueError as e:` | `call() catch e:` | 在出错的地方就地处理。 |
| `int(s) if s.isdigit() else 0` | `int(s) catch e: 0` | 调用失败时改用默认值。 |
| 异常自动向上传播 | `call()?` | 把错误传下去这件事在代码里看得见。 |
| `print(a, b)` | `println(f"{a} {b}")` | `println` 只接收一个值。 |
| `[1, 2, 3]` | `let xs: [i64] = [1, 2, 3]` | 要修改的列表需要写类型标注。 |
| `{"a": 1}` | `{"a": 1}`；空字典：`let m: {string: i64} = {}` | 键是 `i64` 或 `string`。 |
| `lambda x: x * 2` | `fn(x): x * 2` | |
| `import math` | `use std/math` | |
| `import re`、`json`、`subprocess`、`hashlib`、`base64` | `use std/re`、`std/json`、`std/process`、`std/hashlib`、`std/base64` | 函数名同 Python；`re` 不支持反向引用与环视。 |
| `r"\d+"`、`"""..."""` | 相同 | 三引号字符串目前要写在同一行。 |
| `{1, 2}`、`set()` | `{1, 2}`；空集合：`let s: {i64} = {}` | 值是 `i64` 或 `string`。 |
| `def f(x, n=1)`、`f(x, n=2)` | `fn f(x: i64, n: i64 = 1)`、`f(x, n=2)` | 默认值只能是常量。 |
| `b"\x00"`、`s.encode()`、`b.decode()` | 写法相同，类型叫 `bytes` | 下标读出 0–255 的整数。 |

## Zamak 新增的内容

- **所有权**：赋值时字符串、列表、字典和结构体会移动；只读参数自动借用。见[移动与复制](ownership.md)和[借用](borrowing.md)。
- **带检查的整数**：整数是 64 位的，溢出会让程序停下，而不是变成大整数。
- **枚举与 match**：带数据的 `enum`，`match` 会检查是否覆盖了所有情况。见[枚举、Option 与 match](enums.md)。
- **本机可执行文件**：`zam build` 生成一个小文件，不需要解释器就能运行。

## 暂时没有的功能

Zamak 还没有带继承的类、`@wrapping` 以外的装饰器、生成器、`async`、多线程和网络。参数默认值只能是常量，不支持 `*args` 和 `**kwargs`。

## 同一个程序的两种写法

```python
def word_lengths(text):
    lengths = {}
    for word in text.split():
        lengths[word] = len(word)
    return lengths

for word, n in word_lengths("zamak reads like python").items():
    print(f"{word}: {n}")
```

```zamak
fn word_lengths(text: &string) -> {string: i64}:
    let mut lengths: {string: i64} = {}
    for word in text.words():
        lengths[word] = len(word)
    lengths

fn main():
    for word, n in word_lengths("zamak reads like python"):
        println(f"{word}: {n}")
```

```text
zamak: 5
reads: 5
like: 4
python: 6
```
