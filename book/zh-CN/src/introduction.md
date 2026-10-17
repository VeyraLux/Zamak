# 简介

Zamak 是一门静态类型的编程语言：读起来像 Python，编译出来是体积很小的本机可执行文件。它围绕三个目标设计：

- **清晰易读**：代码块靠缩进界定，运算符和内置函数与 Python 一致，不需要写生命周期标注。
- **快而小**：Zamak 经由 C 编译成本机代码，没有垃圾回收器，也不需要另装运行时。一个 hello world 约 56 KB。
- **默认安全**：所有权规则在编译时就排除了释放后使用和重复释放。整数溢出、除以零和下标越界会让程序带着清楚的错误信息停下，而不是悄悄破坏内存。

下面是一个完整的程序：

```zamak
struct Item:
    name: string
    price: f64
    qty: i64

fn total(items: &[Item]) -> f64:
    let mut sum = 0.0
    for item in items:
        sum += item.price * item.qty
    sum

fn main():
    let items: [Item] = [
        Item(name: "notebook", price: 4.5, qty: 3),
        Item(name: "pen", price: 1.25, qty: 10),
    ]
    for item in items:
        println(f"{item.qty} x {item.name}")
    println(f"total: {total(items)}")
```

```text
3 x notebook
10 x pen
total: 26.0
```

如果你熟悉 Python，这段代码大部分看起来都很眼熟。不同的地方正是 Zamak 又快又安全的原因：每个值都有类型，`let mut` 标出会改变的变量，`&[Item]` 说明 `total` 只是借用这个列表，而不是把它拿走。

## 这本书写给谁

这本书写给所有想用 Zamak 写程序的人。它不要求你熟悉某一门语言，但最好以前写过一些代码。如果你会 Python，[附录 A](appendix-python.md) 把你已经会的写法一一对应到 Zamak。

## 怎么读这本书

各章前后衔接，第一次读请按顺序来：

- **入门**：安装 Zamak，写出最初的几个程序。
- **语言基础**：值、运算符、控制流、函数和字符串。
- **所有权**：Zamak 怎样在没有垃圾回收器的情况下管理内存。如果你来自 Python、Go 或 Java，这是唯一真正陌生的概念。
- **组织数据**、**错误与测试**、**项目**：写实际程序需要的内容。
- **工具**：`zam` 命令，以及怎样配合 AI 编程助手。

书里的每个代码示例都是完整的程序。本书的测试会逐个编译运行这些示例，并核对输出与示例下方的文字一致，所以示例始终适用于当前版本的 Zamak。本来就该出错的示例会给出 `zam` 报告的错误。

## 获取帮助

- 在 [GitHub](https://github.com/VeyraLux/Zamak/issues) 上报告问题或提问。
- 发现书里的错误，可以选择任意页面顶部的编辑图标直接修改。
