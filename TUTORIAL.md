# Zamak 入门教程

本教程介绍 Zamak 0.1.0 的基本语法、类型与所有权规则。完成后，你将能够创建多模块项目，编译原生程序，并使用错误值处理失败路径。示例面向初学者，不要求 Rust 或 C 编程经验。

先阅读 [安装说明](PREVIEW.md)，准备 `zam` 和平台 C 工具链。安装命令在仓库根目录执行；项目命令在包含 `zam.toml` 的目录执行，或显式指定项目、源码文件的路径。代码块后的输出表示该示例的预期结果。教程测试会提取 Zamak 代码块并运行 `zam check`；它验证编译检查能够通过，不逐项核对本页的运行输出。其他集成测试覆盖原生运行和错误路径。

Zamak 仍处于原型阶段，语法与接口可能变化。支持范围和限制见本教程最后一节及 [README](README.md)。学习顺序为：基础值与控制流 → 函数和字符串 → 数组与结构体 → 所有权 → 错误处理 → 多模块项目。

## 第 0 节：准备环境

安装完成后，确认命令可用：

```powershell
zam --version
```

```text
0.1.0
```

如果尚未安装 `zam`，在仓库根目录执行以下命令。源码安装需要 Rust 工具链；构建 Zamak 程序时，Windows 还需要 Visual Studio 的 C++ 工具，Unix 需要 `cc`：

```powershell
cargo install --path zam --locked
```

`zam` 是 Zamak 编译器的命令行工具。它将源码转换为 C 代码，再调用系统 C 编译器生成原生可执行文件。生成的程序运行时不需要安装 Zamak 或 Rust。

## 第 1 节：第一个程序

在可写目录中创建名为 `hello` 的项目，再进入项目目录运行。目标目录不能已存在：

```powershell
zam new hello
cd hello
zam run
```

```text
hello from zamak
```

上面只列出程序的标准输出（stdout）。`zam new` 的 `NEW` 行，以及构建时的 `CACHE`、`OBJECT`、`LINK`、`OUTPUT` 行写入标准错误（stderr）；实际路径、缓存哈希和命中状态取决于环境及构建历史。需要保存程序输出时，可以单独重定向 stdout。

也可以直接运行单个源码文件。在当前目录新建 `hello.zm`，写入以下内容，再执行 `zam run hello.zm`：

```zamak
fn main() {
    println("你好，Zamak")
}
```

```text
你好，Zamak
```

要点：

- 程序从 `fn main()` 开始执行。
- `println(...)` 打印一行，字符串字面量使用双引号。
- 函数体使用大括号。本教程采用 4 个空格缩进，`zam fmt` 可以整理缩进。
- 注释用 `//`（行注释），也可以用 `#`。

## 第 2 节：变量与类型

第 2 至 11 节的每个完整代码块都是独立程序。可以将当前示例保存为 `lesson.zm`，执行 `zam check lesson.zm` 检查，再执行 `zam run lesson.zm` 观察输出。切换示例时替换整个文件内容，不要将多个 `fn main()` 合并到同一文件。

```zamak
fn main() {
    let name = "Zamak"
    let mut count: i64 = 3
    count = count + 1
    let ok = true
    println(name)
    println(count)
    println(ok)
}
```

```text
Zamak
4
true
```

要点：

- `let` 声明变量，默认不可重新赋值；需要重新赋值时使用 `let mut`。
- 类型标注写在冒号后，如 `let mut count: i64 = 3`。省略类型标注时，编译器根据初值推断类型。
- 三种基本类型：`i64`（64 位整数）、`bool`（`true`/`false`）、`string`（字符串）。
- **没有变量遮蔽**：同一作用域内不能再次用 `let` 声明同名变量。
- `i64` 和 `bool` 使用复制语义，`let copy = count` 得到独立的值，`count` 仍可使用。

练习：将 `let mut count` 改为 `let count`，保存后运行 `zam check` 并指定该文件。确认编译器拒绝给不可变变量重新赋值，再恢复 `mut`。

## 第 3 节：运算符

```zamak
fn main() {
    println(7 / 2)
    println(7 % 2)
    println(2 + 3 * 4)
    println(10 - 4)
    println(1_000_000)
    println(3 > 2)
    println(!true)
}
```

```text
3
1
14
6
1000000
true
false
```

要点：

- 算术：`+` `-` `*` `/` `%`。整数除法向零截断，例如 `7 / 2` 得到 `3`，`-7 / 2` 得到 `-3`；余数的符号与被除数一致，例如 `-7 % 2` 得到 `-1`。
- 比较：`==` `!=` `<` `>` `<=` `>=`，结果是 `bool`。
- 逻辑：`!`（非）、`&&`（与）、`||`（或）。
- 整数字面量的数字之间可以用单个 `_` 分组，例如 `1_000_000` 表示一百万。
- 乘、除、取余的优先级高于加、减；可以用括号明确运算顺序。整数溢出和除零会导致运行失败。

## 第 4 节：条件判断

```zamak
fn classify(n: i64) -> string {
    if n < 0 {
        return "negative"
    } else if n == 0 {
        return "zero"
    } else {
        return "positive"
    }
}

fn main() {
    println(classify(-3))
    println(classify(0))
    println(classify(9))
}
```

```text
negative
zero
positive
```

要点：

- `if` 的条件不需要外层括号，条件后紧接代码块。
- 多个条件可以使用 `else if` 依次判断。
- 条件必须是 `bool`，Zamak 不做真假值隐式转换。
- `return` 立即从当前函数返回。本例各分支都返回 `string`，因此覆盖了函数的所有返回路径。

## 第 5 节：循环

```zamak
fn main() {
    let mut i = 0
    while i < 5 {
        i = i + 1
        if i == 2 {
            continue
        }
        if i == 5 {
            break
        }
        println(i)
    }
}
```

```text
1
3
4
```

按次数循环用 `for`：

```zamak
fn main() {
    let mut total = 0
    for i in 1..5 {
        total = total + i
    }
    println(total)

    let scores = [90, 72, 85]
    for i in 0..3 {
        println(scores[i])
    }
}
```

```text
10
90
72
85
```

要点：

- `for i in start..end` 遍历左闭右开区间：`1..5` 依次得到 1、2、3、4，不含 5；`start` 不小于 `end` 时一次也不执行。
- `start` 和 `end` 在进入循环前各求值一次，循环体里修改相关变量不会改变循环次数。
- 循环变量 `i` 是只读的 `i64`，只在循环体内可见；不能与已有变量同名。
- 目前 `for` 只能遍历整数区间，不能直接遍历数组；按下标遍历数组写 `for i in 0..长度`。
- `while` 适合按条件结束的循环，条件必须是 `bool`。
- 尚不支持 `+=` 等复合赋值，递增应写为 `i = i + 1`。
- `continue` 跳过本次循环剩下的部分，`break` 直接结束循环；两者在 `while` 和 `for` 中都可用，只影响最内层循环。

## 第 6 节：函数

```zamak
fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn greet(name: string) -> string {
    return "你好，{name}！"
}

fn main() {
    println(add(2, 3))
    let who = "Zamak"
    println(greet(who))
}
```

```text
5
你好，Zamak！
```

要点：

- 参数写为 `名称: 类型`，多个参数以逗号分隔。
- `-> 类型` 声明返回值类型；省略时表示函数不返回值。
- 函数体末尾的值表达式可以作为返回值，因此 `add` 无需显式写 `return`。也可以使用 `return` 明确返回或提前结束函数。
- 跨模块调用需要 `pub fn`（见第 12 节）。
- 当前不支持泛型、闭包和函数重载。本教程使用 `while` 讲解重复执行，不涉及递归。

练习：将 `greet` 的参数改为 `name: &string`，并使用 `println(greet(&who))` 调用。借用字符串可以用于插值；插值只读取内容，不移动或修改原字符串。调用后再加入 `println(who)`，检查借用结束后原变量仍可使用。

## 第 7 节：字符串、转义与插值

```zamak
use std/string

fn main() {
    let name = "Zamak"
    let text = "你好，{name}！"
    println(text)
    println("第一行\n第二行")
    println("字面大括号 {{x}}")
    println(string.byte_len(&text))
    println(string.char_len(&text))
    println(string.equal(&text, &text))
}
```

```text
你好，Zamak！
第一行
第二行
字面大括号 {x}
17
9
true
```

要点：

- 插值语法 `{变量}` 将值写入字符串，也支持结构体字段，例如 `"{point.x}"`。插值只读取值，不移动原变量。
- 转义：`\n` 换行、`\r` 回车、`\t` 制表、`\"` 双引号、`\\` 反斜杠。
- 输出字面大括号时使用 `{{` 和 `}}`。
- 字符串字面量必须写在一行内；字符串内容需要换行时，使用 `\n` 转义。
- 标准库 `std/string` 提供三个函数：`string.byte_len(&s)` 返回 UTF-8 字节数，`string.char_len(&s)` 返回 Unicode 码点数，`string.equal(&a, &b)` 判断内容是否相等。本例的 `你好，Zamak！` 占 17 字节，包含 9 个码点。
- Unicode 码点不等同于用户看到的一个字符（字素簇）。组合附加符号或由多个码点组成的 emoji 可能显示为一个字符，`char_len` 仍逐个码点计数。常见汉字在 UTF-8 中通常占 3 字节，因此字节数与码点数不同。

## 第 8 节：数组、列表与字典

Zamak 有两种数组：长度固定的定长数组 `[i64; N]`，以及可以增减元素的列表 `[T]`。按键存取数据用字典 `{K: V}`，见本节最后。

定长数组：

```zamak
fn main() {
    let mut values: [i64; 3] = [10, 20, 30]
    values[1] = values[0] + 5
    let mut i = 0
    while i < 3 {
        println(values[i])
        i = i + 1
    }
}
```

```text
10
15
30
```

要点：

- 类型写成 `[i64; 3]`（元素类型 + 分号 + 长度），长度必须在 1 到 1024 之间。
- 下标从 0 开始，范围为 `0` 到 `长度 - 1`。运行时会检查边界；越界访问会报错并退出。
- 定长数组元素类型仅支持 `i64`，只能作为局部值，不能用于函数参数、返回值或结构体字段；这些场合使用列表。

列表 `[T]` 的元素可以是 `i64`、`bool` 或 `string`，可以作为局部变量、函数参数、返回值和结构体字段：

```zamak
fn average(scores: &[i64]) -> i64 {
    let mut sum = 0
    for score in &scores {
        sum = sum + score
    }
    sum / scores.len()
}

fn main() {
    let mut scores: [i64] = [90, 72]
    scores.push(85)
    scores[1] = 75
    println(scores.len())
    println(average(&scores))
    let last = scores.pop()
    println(last)
}
```

```text
3
83
85
```

字符串列表：

```zamak
use std/string

fn main() {
    let line = "apple banana  cherry"
    let mut words = string.words(&line)
    words.push("date")
    for word in &words {
        let size = string.char_len(word)
        println("{word}: {size}")
    }
    let first = string.copy(&words[0])
    let sep = ", "
    println(string.join(&words, &sep))
    println(first)
}
```

```text
apple: 5
banana: 6
cherry: 6
date: 4
apple, banana, cherry, date
apple
```

要点：

- 空列表必须标注类型：`let mut xs: [i64] = []`。不写标注的 `[1, 2]` 仍然是定长数组。
- `xs.len()` 返回元素个数；`xs.push(v)` 追加到末尾，字符串会移动进列表；`xs.pop()` 移除并返回最后一个元素。修改列表需要 `let mut`。
- `xs[i]` 读写元素，下标越界时程序终止（`zam: list index out of bounds`）；`pop` 空列表同样终止。
- 字符串元素不能用 `xs[i]` 直接取出（那会把字符串移出列表）。传给函数时借用 `&xs[i]`，需要一份独立的字符串时用 `string.copy(&xs[i])`。
- `for x in &xs` 依次访问每个元素：整数和布尔元素按值得到，字符串元素以只读借用得到。循环体内不能修改、移动或独占借用这个列表；需要边遍历边修改时，用 `for i in 0..xs.len()` 按下标处理。
- 函数参数写 `&[T]`（只读）或 `&mut [T]`（可修改），调用时传 `&xs` 或 `&mut xs`；参数写 `[T]` 表示把列表整个移交给函数。
- `xs.swap(i, j)` 原地交换两个元素，排序字符串列表时用它，不用复制字符串。
- 方法只能用在变量或字段上，例如 `bag.items.push(x)`；暂不支持链式写法 `f().len()`，先用 `let` 绑定。

字典按键存取值，下面统计每种颜色出现的次数：

```zamak
use std/string

fn main() {
    let text = "red blue red green blue red"
    let words = string.words(&text)
    let mut counts: {string: i64} = {}
    for word in &words {
        counts[word] = counts.get(word, 0) + 1
    }
    for color in &counts {
        let n = counts[color]
        println("{color}: {n}")
    }
    println(counts.contains("red"))
    counts.remove("blue")
    println(counts.len())

    let ages = {"amy": 30, "bob": 25}
    println(ages["amy"])
}
```

```text
red: 3
blue: 2
green: 1
true
2
30
```

要点：

- 类型写成 `{键类型: 值类型}`。键可以是 `i64` 或 `string`，值可以是 `i64`、`bool` 或 `string`。
- 字面量写成 `{"amy": 30, "bob": 25}`，条目之间用逗号，可以分行写。空字典必须标注类型：`let mut m: {string: i64} = {}`。
- `m[k]` 读取，键不存在时程序终止（`zam: key not found`）；不确定键是否存在时用 `m.get(k, 默认值)` 或先 `m.contains(k)`。
- `m[k] = v` 插入或替换；`m.remove(k)` 删除并返回是否删到了；`m.len()` 返回条目数；`m.keys()` 返回键的列表。
- 字符串键按只读借用传入：变量写 `m[&name]`，字面量直接写 `m["red"]`，`for` 循环得到的键直接写 `m[k]`。新键插入时字典会复制一份。
- `for k in &m` 按插入顺序遍历键，所以输出顺序是确定的。循环体内不能修改这个字典。
- 字符串值不能用 `m[k]` 取出，借用 `&m[k]`，或用 `m.get(k, "")` 得到一份副本。

## 第 9 节：结构体

```zamak
struct Point {
    x: i64
    y: i64
}

fn move_by(point: Point, dx: i64) -> Point {
    let mut result = point
    result.x = result.x + dx
    result
}

fn main() {
    let mut p = Point { x: 1
        y: 2
    }
    println(p.x)
    p.y = p.y + 3
    println(p.y)
    let moved = p
    println(moved.x + moved.y)
    let q = move_by(moved, 10)
    println(q.x)
}
```

```text
1
5
6
11
```

要点：

- 结构体声明每行写一个字段，字段之间不用逗号；函数参数之间仍需要逗号。
- 构造用 `Point { x: 1 y: 2 }`，字段之间同样不写逗号，可以写在一行也可以分行。
- 字段通过 `.` 访问。修改字段要求所属变量可变，如本例的 `let mut p`。
- 结构体使用移动语义：执行 `let moved = p` 后，再读取 `p` 会报 `use after move`。对于可变变量，可以通过整体重新赋值恢复其有效值，例如 `p = Point { x: 1 y: 2 }`。
- 嵌套结构体字段可以连续访问，例如 `group.leader.age = group.leader.age + 1`。

## 第 10 节：所有权与借用

```zamak
fn show(text: &string) {
    println(text)
}

fn update(text: &mut string) {
    text = "被借用修改过"
}

fn consume(text: string) {
    println(text)
}

fn main() {
    let mut message = "原始内容"
    show(&message)
    update(&mut message)
    show(&message)
    let moved = message
    consume(moved)
}
```

```text
原始内容
被借用修改过
被借用修改过
```

要点：

- `string` 使用所有权管理。赋值给另一变量，或传给 `text: string` 这样的值参数时，会转移所有权（移动），原变量随后不能再读取。本例的 `consume` 接收字符串所有权。
- 只需读取字符串时，使用共享借用：将 `&message` 传给 `text: &string`。
- 需要修改字符串时，使用可变借用：将 `&mut message` 传给 `text: &mut string`，原变量必须通过 `let mut` 声明。
- 借用仅在本次函数调用期间有效；`show` 和 `update` 返回后，`message` 仍可使用。同一次调用中，共享借用可以有多个，可变借用必须独占。借用只能用于参数，不能返回或保存在局部变量中。
- 所有权和借用检查发生在编译期，运行 `zam check` 即可检查。诊断信息示例为 `line 4, column 1: main:main: use after move: a`。
- 当前 `string` 和结构体使用移动语义，`i64` 和 `bool` 使用复制语义。字符串字段支持借用；整体结构体借用和单独移出拥有所有权的字段尚不支持。

## 第 11 节：错误处理

函数在返回类型后加 `!错误类型`，表示调用可能失败；当前错误类型仅支持 `string`。

下面的 `parse` 仅用于演示错误处理：它拒绝空字符串，并对所有非空字符串返回固定值 `42`，不执行整数解析。它不是可用于输入校验的解析器，标准库目前也未提供字符串转整数接口。

```zamak
use std/string

fn parse(text: &string) -> i64!string {
    let empty = ""
    if string.equal(text, &empty) {
        fail "empty input"
    }
    return 42
}

fn load(text: &string) -> i64!string {
    let value = parse(text)?
    return value
}

fn main() -> !string {
    let text = "42"
    let value = load(&text)?
    println(value)
}
```

```text
42
```

错误处理语法：

- `i64!string`：成功返回 `i64`，失败返回 `string` 错误信息。
- `fail "empty input"`：产生错误并立即从当前函数返回，只能在声明了错误类型的函数中使用。
- `parse(text)?`：成功时取得返回值，失败时立即从 `load` 返回同一错误。使用 `?` 的函数必须声明相同的错误类型。

`main` 可以声明为 `-> !string`，表示不返回成功值，但可能失败。传播到 `main` 且未处理的错误会写入 stderr，程序以退出码 1 结束。将上例的 `"42"` 改为 `""` 后，`zam run` 的错误输出为：

```text
empty input
zam: program failed: exit code: 1
```

使用 `catch` 可以在调用处处理错误。以下 `parse` 与前例相同，仍是返回固定值的占位函数：

```zamak
use std/string

fn parse(text: &string) -> i64!string {
    let empty = ""
    if string.equal(text, &empty) {
        fail "empty input"
    }
    return 42
}

fn main() {
    let text = "42"
    let value = parse(&text) catch e {
        println("解析失败：{e}")
        return
    }
    println(value)
}
```

成功时输出 `42`。将 `text` 的初值改为 `""` 后，错误分支输出以下内容，并从 `main` 返回；此时程序正常结束：

```text
解析失败：empty input
```

要点：

- `catch e { ... }` 里的 `e` 是错误值，只在块内可见。
- `catch` 块必须在所有路径上使用 `return` 从当前函数返回；若块内有条件分支，每个分支都必须返回。它不支持用末尾表达式为 `let value` 提供恢复值。成功时才会给 `value` 赋值并继续执行后续语句。
- 当前错误类型仅支持 `string`，不支持泛型错误和错误堆栈。可失败调用必须使用 `?` 传播或使用 `catch` 处理。

## 第 12 节：多模块项目

单个源码文件可以通过 `zam run 文件名.zm` 运行。多个模块组成的项目使用 `zam.toml` 配置入口和模块目录。在第 1 节创建的 `hello` 项目中，确认配置如下，再添加下方的两个源码文件：

```toml
[project]
name = "hello"
entry = "src/main.zm"
modules = "src"
```

```text
hello/
├── zam.toml
└── src/
    ├── main.zm
    └── greeting.zm
```

`src/main.zm`：

```zamak
use greeting
use std/io

fn main() {
    greeting.hello()
    io.println("你好，Zamak")
}
```

`src/greeting.zm`：

```zamak
pub fn hello() {
    println("hello from zamak")
}
```

在 `hello` 项目根目录执行 `zam run`，得到以下程序输出。如果当前位于仓库根目录，也可以执行 `zam run examples/hello` 运行对应的仓库示例：

```text
hello from zamak
你好，Zamak
```

要点：

- `entry` 是入口文件，`modules` 是模块所在目录。
- **模块名就是文件名**：`greeting.zm` 用 `use greeting` 引入，调用写成 `greeting.hello()`。
- 跨模块可见的函数、结构体要加 `pub`。
- 跨模块用类型要写模块前缀：`leader: model.Person`。
- 文件后缀统一是 `.zm`；入口必须是 `.zm`（否则报 `entry must be a .zm file`）。
- 对于单文件程序，`zam check`/`build`/`run`/`fmt` 也接受单个 `.zm` 文件。

## 第 13 节：文件、标准输入与命令行参数

读取命令行参数指定的文件，统计后写出报告：

```zamak
use std/env
use std/fs
use std/string

fn main() -> !string {
    if env.arg_count() < 1 {
        fail "usage: zam run main.zm -- <file>"
    }
    let path = env.arg(0)?
    let text = fs.read(&path)?
    let bytes = string.byte_len(&text)
    let chars = string.char_len(&text)
    let report = "{path}: {bytes} bytes, {chars} chars"
    println(report)
    let target = "report.txt"
    fs.write(&target, &report)?
}
```

`笔记.txt` 的内容是 `name=扎马克` 时，执行 `zam run main.zm -- 笔记.txt` 输出下面一行，并把同样的内容写进 `report.txt`：

```text
笔记.txt: 14 bytes, 8 chars
```

不传参数时程序以 `fail` 结束，stderr 输出 `usage: zam run main.zm -- <file>`，退出码为 1。

逐行读取标准输入并求和：

```zamak
use std/io
use std/string

fn main() -> !string {
    let mut line = ""
    let mut total = 0
    while io.read_line(&mut line)? {
        let text = string.trim(&line)
        if string.byte_len(&text) > 0 {
            total = total + string.parse_i64(&text)?
        }
    }
    println(total)
}
```

在 pwsh 中执行 `"10`n20`n`n-5" | zam run main.zm`，输出 `25`。

拆分 `key=value`：

```zamak
use std/string

fn main() {
    let entry = "  lang=Zamak  "
    let text = string.trim(&entry)
    let separator = "="
    let at = string.find(&text, &separator)
    let key = string.slice(&text, 0, at)
    let value = string.slice(&text, at + 1, string.byte_len(&text))
    println("{key} -> {value}")
    println(string.byte_at(&text, 0))
}
```

```text
lang -> Zamak
108
```

| 函数 | 说明 |
| --- | --- |
| `env.arg_count() -> i64` | 程序参数个数，不含程序名 |
| `env.arg(i) -> string!string` | 第 `i` 个参数，从 0 开始；越界返回错误 |
| `fs.read(&path) -> string!string` | 读取整个文件；打不开或读取失败返回错误 |
| `fs.write(&path, &text) -> !string` | 创建或覆盖文件并写入全部内容 |
| `io.read_line(&mut line) -> bool!string` | 读一行到 `line`，去掉行尾的 `\n` 或 `\r\n`；输入结束时返回 `false` 并把 `line` 置空 |
| `io.read_all() -> string!string` | 读取全部标准输入 |
| `string.parse_i64(&s) -> i64!string` | 解析十进制整数，可带 `+`/`-`；含空白或超出 `i64` 范围时返回错误 |
| `string.trim(&s) -> string` | 去掉首尾的空格、制表符和换行，返回新字符串 |
| `string.byte_at(&s, i) -> i64` | 第 `i` 个字节（0–255）；越界时程序终止 |
| `string.slice(&s, start, end) -> string` | 复制字节区间 `[start, end)`；越界或切在多字节字符中间时程序终止 |
| `string.find(&s, &needle) -> i64` | 第一次出现的字节位置；找不到返回 -1，空 `needle` 返回 0 |
| `string.copy(&s) -> string` | 复制出一份独立的字符串 |
| `string.split(&s, &sep) -> [string]` | 按分隔符切分，保留空段；空分隔符时整串作为唯一一项 |
| `string.words(&s) -> [string]` | 按空白切分，跳过连续空白 |
| `string.lines(&s) -> [string]` | 按行切分，去掉行尾 `\r`，末尾换行不产生空行 |
| `string.join(&list, &sep) -> string` | 用分隔符连接字符串列表 |
| `env.args() -> [string]` | 全部程序参数，不含程序名 |

要点：

- 使用前分别导入 `use std/env`、`use std/fs`、`use std/io`、`use std/string`。
- `zam run` 把 `--` 之后的内容原样传给程序；直接运行构建产物时照常传参。
- 路径和参数按 UTF-8 处理，Windows 上可以直接使用中文路径和中文参数。
- 标准输入按字节读取。管道和文件重定向按原样传入字节；Windows 控制台里手动输入的文字使用控制台代码页，中文输入建议改用管道或文件。
- 字符串位置都以字节计。`slice` 与数组下标一样在越界时终止程序；需要先判断的场景，先用 `string.byte_len` 或 `string.find` 检查范围。
- 目前没有目录遍历、追加写入、网络和时间函数。

## 第 14 节：命令行工具

```text
zam 0.1.0
  zam check [project|file]
  zam build [project|file]
  zam fmt [project|file]
  zam run [project|file|artifact] [-- args...]
  zam new <directory> [--template ai]
  zam check --json [project|file]
  zam build --json [project|file]
  zam demo
  zam --version
```

- `zam check`：只做词法、语法、类型与所有权检查，不生成机器码，因此不需要 C 编译器，适合在修改源码后快速验证。成功打印 `CHECK OK main`。
- `zam build`：生成可执行文件，路径以 `OUTPUT <path>` 写到 stderr，可以直接运行。
- `zam run`：先构建再运行，程序输出就是它的 stdout；`--` 之后的参数原样传给程序，例如 `zam run main.zm -- 笔记.txt`。
- `zam fmt`：就地整理缩进（4 个空格）、去行尾空白、结尾留一个换行，保留原来的 CRLF/LF。只调整空白，不重建词法单元，因此保留注释和字符串内容；语法有错的代码会被拒绝而不会被改写。修改过的文件通过 `FORMAT <path>` 写入 stderr。它不调整行内空格或自动折行。
- `zam new <directory>`：生成最小项目骨架（第 1 节）。目录名必须是合法标识符，已存在的目录会被拒绝，不会覆盖任何文件。
- `zam demo`：运行内置的双目录缓存演示，展示相同源码的缓存复用。
- `zam check --json` / `zam build --json`：将结果以一行 JSON 写入 stdout，供编辑器、持续集成（CI）或 AI 工具读取。其他子命令不接受 `--json`。第 15 节示例对应的 JSON 如下：

```text
{"command":"check","ok":false,"diagnostics":[{"file":"main.zm","line":4,"column":5,"function":"main:main","code":"ZM301","category":"ownership","message":"use after move: name","suggestion":"borrow the value or move it only after the last use"}]}
```

- 环境变量 `ZAMAK_CACHE_DIR` 可以改缓存目录。
- 出错时退出码非 0，错误写 stderr。

## 第 15 节：常见错误与解决办法

下面的程序把 `name` 移给了 `other`，之后又使用它：

```text
fn main() {
    let name = "Zamak"
    let other = name
    println(name)
}
```

`zam check main.zm` 的输出：

```text
error[ZM301]: use after move: name
 --> main.zm:4:5
  |
4 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

- 第一行是错误码和原因，`-->` 后面是 `文件:行:列`。行、列号从 1 开始，列号按 Unicode 码点计数。
- 接着是出错的源码行。词法和语法错误只在出错的记号下划线；类型和所有权错误指向出错语句的开头，下划线划到行尾。
- `= help` 给出修改方向。
- 每个出错的函数各报一条错误，多条错误之间空一行，最后一行是 `zam: N errors`。同一个函数里后面的错误，要等前一条修好才会出现。
- 没有位置的错误（例如找不到 `zam.toml`）仍是一行 `zam: <信息>`。

| 报错信息 | 原因 | 处理方法 |
| --- | --- | --- |
| `unexpected character '@'` | 出现了不属于 Zamak 的字符 | 删除或替换该字符 |
| `expected '{', found Some(Word("string"))` | 语法不符合要求，例如类型标注位置错误 | 对照前面的小节检查该行结构 |
| `unterminated function block` | 大括号未配对 | 补上 `}` |
| `use after move: a` | 变量的所有权已转移 | 用 `&a` 借用，或重新赋值 |
| `cannot interpolate this value` | 插值中使用了结构体或数组等不支持的值 | 使用字符串、借用字符串、整数或布尔字段 |
| `borrow escape: functions cannot return references` | 函数尝试返回 `&string`/`&mut string` | 返回拥有所有权的 `string` |
| `arrays are only supported as local values` | 把数组当参数或返回值 | 改成多个标量参数，或在函数内建数组 |
| `fallible call main:f must be handled with ?` | 可失败调用未被处理 | 加 `?`，或用 `catch` 就地处理 |
| `? cannot propagate the error of main:f from this function` | 当前函数未声明错误类型，或错误类型不一致 | 函数签名写 `-> T!string` |
| `fail requires a function that declares an error type` | 在不可失败的函数里用了 `fail` | 给函数加 `!string` |
| `catch block for f must return a value` | `catch` 块存在未返回的路径 | 确保所有路径都从当前函数 `return` |
| `only string error types are supported for now` | 用了 `!i64` 之类的错误类型 | 改成 `!string` |
| `entry must be a .zm file` | `zam.toml` 的入口后缀不对 | 改成 `.zm` |
| `zam.toml: 系统找不到指定的文件` | 目录内没有 `zam.toml` | 用 `zam new` 建项目，或直接传单个 `.zm` 文件 |
| `project name must be an identifier` | `zam new` 的目录名不是合法标识符 | 使用合法标识符，例如 `hello_app`；不要使用连字符或以数字开头 |
| `<dir>: already exists` | 目标目录已存在 | 使用尚不存在的目标目录 |
| `zam: program failed: exit code: 1` | 未处理的错误传播到 `main` | 检查错误信息和输入；需要处理时在调用处使用 `catch` |
| `cannot open <path>` | `fs.read`/`fs.write` 打不开文件 | 检查路径是否存在、是否是目录、是否有权限 |
| `invalid integer: <text>` | `string.parse_i64` 的输入不是十进制整数 | 先用 `string.trim` 去掉空白，再解析 |
| `argument index out of range` | `env.arg(i)` 的 `i` 超出参数个数 | 先用 `env.arg_count()` 检查 |
| `zam: string index out of bounds` | `byte_at`/`slice` 越界，或切在多字节字符中间 | 用 `string.byte_len`、`string.find` 先确定范围 |
| `zam: list index out of bounds` / `zam: pop from empty list` | 列表下标越界，或对空列表 `pop` | 先用 `xs.len()` 检查 |
| `an empty list literal needs a type annotation` | `let xs = []` 没写类型 | 写成 `let mut xs: [i64] = []` |
| `cannot move a string out of xs[..]` | 用 `xs[i]` 取字符串元素 | 借用 `&xs[i]`，或 `string.copy(&xs[i])` |
| `xs is borrowed by an enclosing for loop` | 在 `for x in &xs` 里修改或移动了 `xs` | 改用 `for i in 0..xs.len()` 按下标处理 |
| `zam: key not found` | `m[k]` 读取了不存在的键 | 用 `m.get(k, 默认值)`，或先 `m.contains(k)` |
| `an empty map literal needs a type annotation` | `let m = {}` 没写类型 | 写成 `let mut m: {string: i64} = {}` |
| `expected explicit borrow of k` | 字符串键用的是 owned 变量 `m[k]` | 写成 `m[&k]` |

## 第 16 节：支持范围与限制

Zamak 0.1.0 的支持范围有限。编写程序时需注意以下限制：

- 尚不支持 `switch`/`match`、枚举、泛型、闭包和 trait/接口；`for` 只支持整数区间。
- 尚不提供包管理器、交互式执行环境（REPL）、文档生成工具或重构工具。
- 标准库包括 `std/string`（`byte_len`、`char_len`、`equal`、`parse_i64`、`trim`、`byte_at`、`slice`、`find`、`copy`、`split`、`words`、`lines`、`join`）、`std/io`（`println`、`read_line`、`read_all`）、`std/fmt`（`println`）、`std/fs`（`read`、`write`）和 `std/env`（`arg_count`、`arg`、`args`），没有目录遍历、网络和时间函数。
- 错误类型只能是 `string`，没有错误堆栈。
- 定长数组仅支持 `i64` 局部值；列表 `[T]` 的元素与字典 `{K: V}` 的值只能是 `i64`、`bool` 或 `string`，字典键只能是 `i64` 或 `string`，还没有结构体元素和嵌套容器。方法不能链式调用。
- 所有权检查覆盖 `string` 和结构体；尚不支持线程、`unsafe` 和外部函数接口（FFI）。
- 编辑器支持为 VS Code 语法高亮扩展（`editors/vscode/`），没有语言服务器、补全和调试器。
- 本教程采用源码安装；安装步骤和平台工具链要求见 [安装说明](PREVIEW.md)。

## 附录：示例与测试

仓库的 `examples/` 目录提供以下可运行示例。在仓库根目录执行 `zam run` 并指定示例目录即可运行：

| 示例 | 内容 |
| --- | --- |
| `examples/hello/` | 多模块项目、`use`、`io.println` |
| `examples/ownership/` | `&string`、`&mut string`、移动 |
| `examples/structs/` | 结构体声明、字段赋值、嵌套、跨模块类型 |
| `examples/control/` | 条件、`while`、`break`/`continue`、函数 |
| `examples/values/` | 变量、运算符、字符串、数组、格式化、错误值、`catch` |

集成测试位于 `zam/tests/`。其中 `tutorial.rs` 检查本教程的 Zamak 代码块，`errors.rs` 覆盖 `T!string`、`fail`、`?`、`catch` 的成功与失败路径，`source_format.rs` 覆盖 `zam fmt` 的行为。验证新的写法时，可以先保存为独立 `.zm` 文件并运行 `zam check 文件名.zm`；需要确认运行行为时再使用 `zam run`。
