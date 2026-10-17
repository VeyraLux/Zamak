# 控制流

Zamak 的控制流你在 Python 里都见过：`if`、`while`、`for`，再加上在多种情况之间做选择的 `match`。

## if、elif 和 else

```zamak
fn describe(n: i64) -> string:
    if n < 0:
        return "negative"
    elif n == 0:
        return "zero"
    else:
        return "positive"

fn main():
    println(describe(-3))
    println(describe(0))
    println(describe(8))
```

```text
negative
zero
positive
```

条件必须是 `bool`。Zamak 不会把 `0`、`""` 或空列表当成假，所以要写出你真正想要的比较，例如 `if len(items) > 0:`。

把 `==` 写成 `=` 是常见的笔误。在条件里，Zamak 会把它报告为错误：

```zamak,fails
fn main():
    let n = 3
    if n = 3:
        println("three")
```

```text
错误[ZM110]：if 的条件里比较要用 `==`；单个 `=` 是赋值
 --> main.zm:3:10
  |
3 |     if n = 3:
  |          ^
  = 提示：比较用 `==`，例如 `score == 60`；单个 `=` 只用来赋值
```

## 选择一个值

要在两个值中挑一个，用条件表达式，与 Python 相同：

```zamak
fn main():
    let score = 72
    let result = "pass" if score >= 60 else "fail"
    println(result)
```

```text
pass
```

选项多于两个时，把 `if` 写成表达式。每个分支是一个表达式，并且必须有 `else`：

```zamak
fn main():
    let temperature = 18
    let feel = if temperature < 10: "cold" elif temperature < 25: "mild" else: "hot"
    println(feel)
```

```text
mild
```

## while 循环

条件为真时，`while` 反复执行它的代码块。`break` 跳出循环，`continue` 直接进入下一轮：

```zamak
fn main():
    let mut n = 0
    while true:
        n += 1
        if n % 2 == 0:
            continue
        if n > 7:
            break
        println(n)
```

```text
1
3
5
7
```

## 遍历区间的 for 循环

`range(n)` 从 0 数到 `n`，不包括 `n`。`range(a, b)` 从 `a` 开始：

```zamak
fn main():
    for i in range(3):
        println(i)
    let mut total = 0
    for i in range(1, 11):
        total += i
    println(total)
```

```text
0
1
2
55
```

区间没有步长。要隔一个数一数，或者倒着数，用 `while`。

## 遍历集合的 for 循环

`for` 也能遍历列表、字典和字符串。多写一个名字，就能同时拿到位置，或者字典条目的值：

```zamak
fn main():
    let fruits: [string] = ["apple", "banana", "cherry"]
    for fruit in fruits:
        println(fruit)
    for i, fruit in fruits:
        println(f"{i}: {fruit}")
    let stock = {"apple": 5, "banana": 0}
    for name, count in stock:
        println(f"{name} -> {count}")
```

```text
apple
banana
cherry
0: apple
1: banana
2: cherry
apple -> 5
banana -> 0
```

`for` 循环借用集合，所以循环结束后还能继续用它。循环进行期间不能增删元素。[列表、字典与元组](collections.md)一章会讲怎样用 `for x in &mut list` 就地修改元素。

## match

`match` 把一个值依次和几个模式比较，执行第一个匹配的分支。用 `|` 分隔几种可能，用 `_` 匹配其余所有情况：

```zamak
fn kind(day: i64) -> string:
    match day:
        6 | 7: "weekend"
        1 | 2 | 3 | 4 | 5: "weekday"
        _: "not a day"

fn main():
    println(kind(3))
    println(kind(7))
    println(kind(9))
```

```text
weekday
weekend
not a day
```

`match` 可以用于整数、字符串、字符和布尔值，用在枚举上最能发挥作用，[枚举、Option 与 match](enums.md)一章会详细介绍。

## 空代码块

代码块不能是空的。需要一个空块时（例如占位），写 `pass`：

```zamak
fn main():
    for i in range(3):
        pass
    println("done")
```

```text
done
```

## 小结

- 条件必须是 `bool`，比较用 `==`。
- `a if cond else b` 和 `if c: a elif d: b else: e` 用来选择一个值。
- 条件成立时 `while` 一直循环；两种循环里都能用 `break` 和 `continue`。
- `for i in range(n)` 计数，`for x in items` 遍历列表、字典或字符串。
- `match` 选择第一个匹配的分支，`_` 匹配任何值。
