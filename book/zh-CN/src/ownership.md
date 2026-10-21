# 移动与复制

Zamak 没有垃圾回收器，你也从来不用自己释放内存。取而代之的是：每个字符串、列表、字典、结构体和枚举都有且只有一个所有者，所有者不在了，Zamak 就释放这个值。这一章讲清楚其中的规则。如果你来自 Python、Go 或 Java，这是你主要需要学习的新东西。

## 每个值只有一个所有者

变量拥有它保存的值。变量所在的代码块结束时，值就被释放：

```zamak
fn main():
    let greeting = "hello"
    if true:
        let shout = greeting.upper()
        println(shout)
    println(greeting)
```

```text
HELLO
hello
```

`shout` 在 `if` 块结束时释放，`greeting` 在 `main` 结束时释放。这些释放由编译器替你插入，所以不会泄漏，也不会重复释放。

## 赋值会移动值

把字符串、列表、字典、结构体或枚举赋给另一个变量，就是移动它。新变量成为所有者，旧变量不能再用：

```zamak,fails
fn main():
    let name = "Ada"
    let other = name
    println(other)
    println(name)
```

```text
错误[ZM301]：使用已移走的值：name
 --> main.zm:5:5
  |
5 |     println(name)
  |     ^^^^^^^^^^^^^
  = 提示：借用这个值，或在最后一次使用之后再移走
```

移动的开销很小：复制的是一个指针，而不是文字本身。又因为只有一个变量拥有这段文字，Zamak 能准确知道什么时候释放它。

指针被复制，但你不会遇到两个变量共享同一份文字的情况：旧变量在移动之后就已经失效，再使用是编译错误，上面的例子就是如此。所以在这里改一个变量、另一个跟着变这种事不会发生，因为另一个已经不能用了。确实需要两个能各自修改的副本时，用下面的 `copy` 显式复制一份。

## 传给函数也可能移动

参数类型里没有 `&` 的参数会接过所有权：

```zamak,fails
fn consume(items: [i64]) -> i64:
    len(items)

fn main():
    let numbers: [i64] = [1, 2, 3]
    println(consume(numbers))
    println(numbers)
```

```text
错误[ZM301]：使用已移走的值：numbers
 --> main.zm:7:5
  |
7 |     println(numbers)
  |     ^^^^^^^^^^^^^^^^
  = 提示：借用这个值，或在最后一次使用之后再移走
```

大多数函数只需要看看参数，所以它们接收借用：`items: &[i64]`。借用时，所有权仍归调用方。下一章[借用](borrowing.md)会详细介绍。只有函数要留下这个值时（例如存进它返回的结构体），才接过所有权。

## 返回值交还所有权

函数可以通过返回值把所有权交给调用方：

```zamak
fn with_total(items: [i64]) -> [i64]:
    let mut result = items
    result.push(sum(result))
    result

fn main():
    let numbers: [i64] = [1, 2, 3]
    let extended = with_total(numbers)
    println(extended)
```

```text
[1, 2, 3, 6]
```

## 数字和布尔值会复制

`i64`、`f64` 和 `bool` 的值很小，赋值时直接复制，两个变量都能继续用：

```zamak
fn main():
    let a = 10
    let b = a
    println(a + b)
```

```text
20
```

## 显式复制

确实需要两个独立的值时，用 `copy` 复制一份。它对字符串、列表、字典、结构体或枚举做深复制：

```zamak
fn main():
    let original: [i64] = [1, 2, 3]
    let mut duplicate = copy(&original)
    duplicate.push(4)
    println(original)
    println(duplicate)
```

```text
[1, 2, 3]
[1, 2, 3, 4]
```

复制花费的时间和内存与值的大小成正比，所以 Zamak 从不悄悄复制。看到 `copy`，就知道那里发生了复制。

## 最后一次使用时再移动

只有在移动之后还用旧变量，才会出问题。把移动放在最后一次使用这个值的地方，就没有问题：

```zamak
struct Order:
    customer: string
    items: [string]

fn main():
    let customer = "Ada"
    let mut items: [string] = []
    items.push("notebook")
    items.push("pen")
    println(f"{customer} ordered {len(items)} items")
    let order = Order(customer: customer, items: items)
    println(order)
```

```text
Ada ordered 2 items
Order(customer: "Ada", items: ["notebook", "pen"])
```

## 为什么要有所有权

- **不需要垃圾回收器**：内存在确定的时刻释放，没有停顿，也不会为了以后回收而多占内存。
- **没有内存错误**：释放后使用和重复释放根本无法通过编译。
- **没有隐藏的开销**：除非你写 `copy`，大的值永远不会被复制。

## 小结

- 每个字符串、列表、字典、结构体和枚举都只有一个所有者，所有者的代码块结束时释放。
- 赋值或传参会移动这类值；之后再用旧名字是错误。
- 数字和布尔值会复制。
- `copy(&x)` 显式地做深复制。
