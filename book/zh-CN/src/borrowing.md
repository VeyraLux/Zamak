# 借用

把值移进每一个读取它的函数是不现实的。取而代之，函数可以借用一个值：在调用期间使用它，所有权仍归调用方。

## 只读借用

以 `&` 开头的参数类型表示只读借用。传入变量时，Zamak 会自动借用：

```zamak
fn shout(text: &string) -> string:
    text.upper() + "!"

fn total(prices: &[f64]) -> f64:
    sum(prices)

fn main():
    let name = "ada"
    let prices: [f64] = [2.5, 4.0]
    println(shout(name))
    println(total(prices))
    println(name)
    println(prices)
```

```text
ADA!
6.5
ada
[2.5, 4.0]
```

调用之后 `name` 和 `prices` 仍然能用，因为函数只是借用了它们。也可以把借用写出来，例如 `shout(&name)`，两种写法意思相同。

只读参数也可以直接接收字面量或字段，例如 `shout("hi")` 或 `shout(user.name)`。

## 可变借用

类型为 `&mut` 的参数可以修改调用方的值。调用处要写 `&mut`，并且变量必须用 `let mut` 声明：

```zamak
fn add_tax(prices: &mut [f64], rate: f64):
    for p in &mut prices:
        p = p * (1 + rate)

fn main():
    let mut prices: [f64] = [10, 20]
    add_tax(&mut prices, 0.5)
    println(prices)
```

```text
[15.0, 30.0]
```

调用处显式写 `&mut` 是有意的设计：读到 `add_tax(&mut prices, 0.5)`，不用看函数定义就知道 `prices` 可能会变。

## 借用很短暂

传给函数的借用只在这一次调用中有效。借用不能存进结构体，也不能从函数返回：

```zamak,fails
fn first(items: &[string]) -> &string:
    items[0]

fn main():
    let names: [string] = ["Ada"]
    println(first(names))
```

```text
错误[ZM422]：借用逃逸：函数不能返回借用
 --> main.zm:1:38
  |
1 | fn first(items: &[string]) -> &string:
  |                                      ^
  = 提示：借用只放在调用参数里；存放或返回有所有权的值
```

改为返回一个拥有所有权的值，例如字符串的副本。因为借用永远不会比创建它的调用或代码块活得更久，Zamak 从来不需要生命周期标注，而那正是学习 Rust 时最难的部分。

## 把借用存进局部变量

要多次读取一个值的某一部分，可以把它借用到局部变量里：

```zamak
struct User:
    name: string
    email: string

fn main():
    let users: [User] = [User(name: "Ada", email: "ada@example.com")]
    let first = &users[0]
    println(first.name)
    println(first.email)
```

```text
Ada
ada@example.com
```

这个借用持续到代码块结束。在那之前，被借用的值不能修改或移动，所以这个局部变量指向的数据始终有效。

## 被借用时不能修改

有别的代码正在读取一个值时，检查器会阻止你修改它。常见的情况是在遍历列表的循环里修改同一个列表：

```zamak,fails
fn main():
    let mut numbers: [i64] = [1, 2, 3]
    for n in numbers:
        if n == 2:
            numbers.push(4)
```

```text
错误[ZM421]：借用冲突：numbers 正被外层 for 循环或借来的局部变量（例如 let x = &numbers[i]）借用
 --> main.zm:5:13
  |
5 |             numbers.push(4)
  |             ^^^^^^^^^^^^^^^
  = 提示：先结束循环或借用：把要做的修改记在另一个列表里，循环结束后再改
```

在别的语言里，这样做要么崩溃，要么悄悄跳过元素。在 Zamak 里，它根本无法通过编译。把新元素先收集到另一个列表里，循环结束后再加进去：

```zamak
fn main():
    let mut numbers: [i64] = [1, 2, 3]
    let mut extra: [i64] = []
    for n in numbers:
        if n == 2:
            extra.push(4)
    numbers.extend(extra)
    println(numbers)
```

```text
[1, 2, 3, 4]
```

## 不写生命周期的代价

不用写生命周期是有代价的。Rust 用生命周期标注告诉编译器「返回的借用和哪个参数活得一样久」，所以能安全地返回借用、把借用存进结构体。Zamak 不让借用离开创建它的那次调用或那个代码块，省掉了标注，也放弃了这些写法：

| 你想做的事 | Rust | Zamak |
| --- | --- | --- |
| 返回列表里某个元素的借用 | `fn first<'a>(xs: &'a [String]) -> &'a String` | 不支持（ZM422）。返回下标，或者返回副本 |
| 结构体里存一个借用，例如解析器借着源文本 | 带生命周期参数的结构体 | 不支持（ZM422）。让结构体拥有数据，或者只存下标 |
| 不复制地取一段 | `&xs[1..3]` | `xs[1:3]` 会复制这一段。只想读时，按下标遍历原列表 |
| 迭代器链，例如 `iter().map().filter()` | 迭代器对象借着集合 | 没有迭代器对象。用 `for` 循环，或者写一个接收函数的函数 |

最常见的替代办法是返回下标：

```zamak
fn longest(words: &[string]) -> i64:
    let mut best = 0
    for i in range(len(words)):
        if len(words[i]) > len(words[best]):
            best = i
    best

fn main():
    let words: [string] = ["fig", "banana", "kiwi"]
    let i = longest(words)
    println(f"{words[i]} is at index {i}")
```

```text
banana is at index 1
```

复杂度并没有消失，只是换了地方：从「写对生命周期标注」变成了「借用不能带出这次调用」这一条规则。这条规则更容易学，违反时编译器会指出位置（ZM421、ZM422）。代价是，有些在 Rust 里不用复制的写法，在 Zamak 里要多复制一次，或者改成传下标。如果你的程序大量依赖返回借用或不复制的视图，目前 Rust 更合适。

## 小结

- `&T` 是只读借用。传入变量时 Zamak 自动借用。
- `&mut T` 是可变借用。调用处写 `&mut x`，并用 `let mut` 声明 `x`。
- 借用只持续一次调用；存进局部变量时持续一个代码块。借用不能存储或返回，所以永远不用写生命周期。
- 值被借用期间，不能修改或移动。
- 代价是不能返回借用、不能在结构体里存借用；改为返回下标或有所有权的值。
