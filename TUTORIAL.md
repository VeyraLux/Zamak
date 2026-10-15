# Zamak 入门教程

本教程介绍 Zamak 0.1.0 的基本语法、类型与所有权规则。完成后，你将能够创建多模块项目，编译原生程序，并使用错误值处理失败路径。示例面向初学者，不要求 Rust 或 C 编程经验。

先阅读 [安装说明](PREVIEW.md)，准备 `zam` 和平台 C 工具链。安装命令在仓库根目录执行；项目命令在包含 `zam.toml` 的目录执行，或显式指定项目、源码文件的路径。代码块后的输出表示该示例的预期结果。教程测试会提取 Zamak 代码块并运行 `zam check`；它验证编译检查能够通过，不逐项核对本页的运行输出。其他集成测试覆盖原生运行和错误路径。

Zamak 仍处于原型阶段，语法与接口可能变化。支持范围和限制见本教程最后一节及 [README](README.zh-CN.md)。学习顺序为：基础值与控制流 → 函数和字符串 → 数组与结构体 → 所有权 → 错误处理 → 多模块项目。

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
fn main():
    println("你好，Zamak")
```

```text
你好，Zamak
```

要点：

- 程序从 `fn main()` 开始执行。
- `println(...)` 打印一行，字符串字面量使用双引号。
- 代码块靠缩进界定，与 Python 相同：`fn main():` 这样以冒号结尾的行开始一个块，块里的每一行比它多缩进 4 个空格，缩进退回来块就结束了。
- 块里只有一条语句时可以写在冒号后面，例如 `fn main(): println("你好")`。空块写 `pass`。
- 缩进只用空格，不用 Tab。`zam fmt` 会把 Tab 换成空格，并整理缩进和空行。
- 注释用 `//`（行注释），也可以用 `#`。

## 第 2 节：变量与类型

第 2 至 11 节的每个完整代码块都是独立程序。可以将当前示例保存为 `lesson.zm`，执行 `zam check lesson.zm` 检查，再执行 `zam run lesson.zm` 观察输出。切换示例时替换整个文件内容，不要将多个 `fn main()` 合并到同一文件。

```zamak
fn main():
    let name = "Zamak"
    let mut count: i64 = 3
    count = count + 1
    let ok = true
    println(name)
    println(count)
    println(ok)
```

```text
Zamak
4
true
```

要点：

- `let` 声明变量，默认不可重新赋值；需要重新赋值时使用 `let mut`。
- 类型标注写在冒号后，如 `let mut count: i64 = 3`。省略类型标注时，编译器根据初值推断类型。
- 四种基本类型：`i64`（64 位整数）、`f64`（64 位浮点数，见第 3 节）、`bool`（`true`/`false`）、`string`（字符串）。
- **没有变量遮蔽**：同一作用域内不能再次用 `let` 声明同名变量。
- 顶层常量写在函数外面：`const LIMIT = 100`、`const RATE: f64 = 0.25`、`const NAME = "zamak"`。常量名全大写，值在编译时算好，只能用字面量、前面声明过的常量和运算符（例如 `const CELLS = WIDTH * HEIGHT`）。常量不能重新赋值，局部变量也不能和它同名。加 `pub` 后其他模块可以写 `config.LIMIT`。
- `i64`、`f64` 和 `bool` 使用复制语义，`let copy = count` 得到独立的值，`count` 仍可使用。

练习：将 `let mut count` 改为 `let count`，保存后运行 `zam check` 并指定该文件。确认编译器拒绝给不可变变量重新赋值，再恢复 `mut`。

## 第 3 节：运算符

```zamak
fn main():
    println(7 / 2)
    println(7 % 2)
    println(2 + 3 * 4)
    println(10 - 4)
    println(1_000_000)
    println(3 > 2)
    println(not true)
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
- 逻辑：`not`（非）、`and`（与）、`or`（或），与 Python 相同。`and` 和 `or` 短路求值：左边已经决定结果时不再计算右边。
- 优先级从低到高是 `or`、`and`、`not`、比较运算，所以 `not a == b` 表示 `not (a == b)`，`a or b and c` 表示 `a or (b and c)`。
- 整数字面量的数字之间可以用单个 `_` 分组，例如 `1_000_000` 表示一百万。也可以写十六进制 `0xFF`、二进制 `0b1010`、八进制 `0o17`。
- 位运算（只用于 `i64`）：`&`（按位与）、`|`（按位或）、`^`（按位异或）、`~`（按位取反）、`<<`（左移）、`>>`（右移，负数向下取整）。移位量要在 0 到 63 之间，否则程序终止。优先级与 Python 相同：比较 < `|` < `^` < `&` < 移位 < 加减，所以 `x & 1 == 1` 表示 `(x & 1) == 1`。
- 乘、除、取余的优先级高于加、减；可以用括号明确运算顺序。整数溢出和除零会导致运行失败。

复合赋值 `+=` `-=` `*=` `/=` `%=`（以及位运算的 `&=` `|=` `^=` `<<=` `>>=`）把运算结果写回原变量：

```zamak
fn main():
    let mut score = 10
    score += 5
    score *= 2
    score -= 4
    println(score)
```

```text
26
```

- `x += e` 等价于 `x = x + e`，变量要用 `let mut` 声明。也可以用在字段（`p.age += 1`）和元素（`xs[i] += 1`、`counts["a"] += 1`）上。
- 元素的下标里不能调用函数，例如 `xs[next()] += 1` 会报错；先用 `let` 把下标算出来。

### 浮点数

带小数点或指数的字面量是 `f64`，例如 `1.5`、`0.25`、`2e-3`。数学函数在 `std/math` 里，按位数格式化用 `fmt.fixed`：

```zamak
use std/fmt
use std/math

fn main():
    let price = 19.99
    let count = 3
    let total = price * f64(count)
    println(total)
    println(fmt.fixed(total, 2))
    println(i64(total))
    println(math.sqrt(2.0))
    println(0.1 + 0.2)
    println(1.0 / 0.0)
```

```text
59.97
59.97
59
1.4142135623730951
0.30000000000000004
inf
```

- `i64` 和 `f64` 之间没有隐式转换。`price * count` 会报错，要写成 `price * f64(count)`；整数字面量也要写成 `2.0` 而不是 `2`。
- `i64(x)` 向零截断，`i64(2.9)` 得到 `2`；要四舍五入就先用 `math.round(x)`。`x` 是 NaN 或超出 `i64` 范围时，程序停止运行并报错。
- `f64` 按 IEEE 754 运算，不检查溢出：除以零得到 `inf` 或 `NaN`。`%` 也能用于 `f64`，结果的符号与被除数一致。
- `println` 打印能原样读回的最短十进制：整数值也带 `.0`（如 `2.0`），绝对值很大或很小时用科学计数法（如 `1e16`、`2.5e-7`）。
- `fmt.fixed(x, n)` 保留 `n` 位小数（`n` 在 0 到 30 之间）。舍入作用在 `x` 的二进制值上，所以 `fmt.fixed(2.675, 2)` 得到 `2.67`。
- `std/math` 提供 `sqrt`、`pow`、`exp`、`log`（自然对数）、`log10`、`sin`、`cos`、`tan`、`atan`、`atan2`、`floor`、`ceil`、`round`、`trunc`、`abs`、`min`、`max`、`is_nan`，以及常量 `math.pi`、`math.e`、`math.inf`。这些函数只接受 `f64`。
- 字符串转浮点数用 `string.parse_f64(&s)`，失败时返回错误，用法和 `parse_i64` 相同。

## 第 4 节：条件判断

```zamak
fn classify(n: i64) -> string:
    if n < 0:
        return "negative"
    elif n == 0:
        return "zero"
    else:
        return "positive"

fn main():
    println(classify(-3))
    println(classify(0))
    println(classify(9))
```

```text
negative
zero
positive
```

要点：

- `if` 的条件不需要外层括号，条件后面写冒号，下面缩进的几行是代码块。
- 多个条件用 `elif` 依次判断（不写 `else if`），最后可以有 `else:`。
- 比较用 `==`。条件里写成单个 `=`（如 `if score = 60:`）会报错，提示改成 `==`。
- 条件必须是 `bool`，Zamak 不做真假值隐式转换。
- `return` 立即从当前函数返回。本例各分支都返回 `string`，因此覆盖了函数的所有返回路径。

`if` 也可以写在表达式位置，按条件选一个值：

```zamak
fn main():
    let n = -4
    let sign = if n < 0: "negative" elif n == 0: "zero" else: "positive"
    let size = if n > 100 or n < -100: "large" else: "small"
    println("{sign} {size}")
```

```text
negative small
```

- 作为值的 `if` 写在一行里：`if 条件: 值 elif 条件: 值 else: 值`。必须有 `else`，每个分支只写一个表达式，各分支的类型要相同。
- 分支里需要多步计算时，先用 `let` 算好，再放进 `if`；或者改用前面的语句写法。

## 第 5 节：循环

```zamak
fn main():
    let mut i = 0

    while i < 5:
        i = i + 1

        if i == 2:
            continue

        if i == 5:
            break

        println(i)
```

```text
1
3
4
```

按次数循环用 `for`：

```zamak
fn main():
    let mut total = 0

    for i in 1..5:
        total = total + i

    println(total)

    let scores = [90, 72, 85]

    for i in 0..3:
        println(scores[i])
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
- 递增可以写 `i += 1`（见第 3 节）。
- `continue` 跳过本次循环剩下的部分，`break` 直接结束循环；两者在 `while` 和 `for` 中都可用，只影响最内层循环。

## 第 6 节：函数

```zamak
fn add(a: i64, b: i64) -> i64:
    a + b

fn greet(name: string) -> string:
    return "你好，{name}！"

fn main():
    println(add(2, 3))
    let who = "Zamak"
    println(greet(who))
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
- 当前不支持函数重载。函数本身也能当值传，见下面的“函数值与匿名函数”。本教程使用 `while` 讲解重复执行，不涉及递归。

练习：将 `greet` 的参数改为 `name: &string`，并使用 `println(greet(&who))` 调用。借用字符串可以用于插值；插值只读取内容，不移动或修改原字符串。调用后再加入 `println(who)`，检查借用结束后原变量仍可使用。

### 泛型函数

函数名后面用方括号写类型参数，同一个函数就能处理不同类型（Python 3.12 的写法）：

```zamak
fn largest[T](items: &[T]) -> T:
    let mut best = items[0]

    for item in items:
        if item > best:
            best = item

    best

fn main():
    let numbers: [i64] = [3, 9, 4]
    println(largest(&numbers))
    let prices: [f64] = [2.5, 1.5]
    println(largest(&prices))
```

```text
9
2.5
```

要点：

- 类型参数首字母大写，可以有多个：`fn swap_pair[A, B](pair: (A, B)) -> (B, A)`。
- 调用时不用写类型，编译器按实参推断 `T`，再为每种类型生成一份专门的函数，所以运行起来和手写的一样快。`T` 至少要在一个参数的类型里出现，否则推断不出来。
- 函数体按具体类型检查：`largest` 用了 `>`，传入不能比较的类型时，报错里会写出是哪个版本（如 `largest[Bool]`）。
- `&T` 遇到 `i64`、`f64`、`bool` 时按值传。

### 泛型结构体与 Option

结构体和枚举也可以带类型参数。内置的 `Option[T]` 表示“可能没有值”：有值是 `Option.Some(值)`，没有是 `Option.None`：

```zamak
struct Stack[T]:
    items: [T]

    fn push(&mut self, item: T):
        self.items.push(item)

    fn pop(&mut self) -> Option[T]:
        if self.items.len() == 0:
            return Option.None

        Option.Some(self.items.pop())

fn main():
    let mut stack: Stack[string] = Stack(items: [])
    stack.push("a")
    println(stack.pop())
    println(stack.pop())
```

```text
Option.Some("a")
Option.None
```

要点：

- 类型写作 `Stack[string]`、`Option[i64]`，方括号里是具体类型。
- 构造时编译器按字段的值推断类型参数，`Stack(items: ["a"])` 就是 `Stack[string]`。`Option.None`、空列表推断不出，要靠变量标注、参数类型或返回类型告诉它。
- 用 `match` 取出 `Option` 里的值：`Some(v): ...`、`None: ...`。

### 函数值与匿名函数

函数也是值：可以当实参传给别的函数。函数类型写作 `fn(参数类型) -> 返回类型`，匿名函数写作 `fn(x): 表达式`：

```zamak
use std/string

fn count_if(words: &[string], keep: fn(&string) -> bool) -> i64:
    let mut count = 0

    for word in words:
        if keep(word):
            count += 1

    count

fn is_short(word: &string) -> bool: string.byte_len(word) < 5

fn main():
    let words: [string] = ["apple", "kiwi", "fig"]
    println(count_if(&words, is_short))
    let min_len = 4
    println(count_if(&words, fn(w): string.byte_len(w) >= min_len))
```

```text
2
2
```

要点：

- 具名函数直接写名字当值传，比如 `is_short`。
- 匿名函数的冒号后面在同一行写一个表达式。参数类型能从函数类型推断时可以省略，`fn(w): ...` 里的 `w` 就是 `&string`。
- 匿名函数可以读外面的变量（这里的 `min_len`），但不能改。用到外面变量的匿名函数只能直接当实参传，不能存进变量。

## 第 7 节：字符串、转义与插值

```zamak
use std/string

fn main():
    let name = "Zamak"
    let text = "你好，{name}！"
    println(text)
    println("第一行\n第二行")
    println("字面大括号 {{x}}")
    println(string.byte_len(&text))
    println(string.char_len(&text))
    println(string.equal(&text, &text))
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
- 插值的花括号里可以写表达式，例如 `"{a + b}"`、`"{xs[0]}"`、`"{string.upper(&name)}"`；但不能再写字符串字面量或花括号。
- Unicode 码点不等同于用户看到的一个字符（字素簇）。组合附加符号或由多个码点组成的 emoji 可能显示为一个字符，`char_len` 仍逐个码点计数。常见汉字在 UTF-8 中通常占 3 字节，因此字节数与码点数不同。
- 字符字面量用单引号：`'a'`、`'#'`、`'\n'`、`'\''`，它的值是这个字符的 Unicode 码点，类型是 `i64`。和 `string.byte_at` 读到的字节比较时只用 ASCII 字符，例如 `string.byte_at(&s, 0) == '#'`；汉字的码点大于 255，不等于任何单个字节。

字符串可以用 `+` 拼接，用 `==`、`<` 等比较：

```zamak
use std/string

fn main():
    let first = "Za"
    let mut name = first + "mak"
    name += "!"
    println(name)
    println(first)
    println("apple" < "banana")
    println(name == "Zamak!")
    let scores: [i64] = [90, 72]
    println("total={scores[0] + scores[1]} length={string.char_len(&name)}")
```

```text
Zamak!
Za
true
true
total=162 length=6
```

- `a + b` 生成一个新字符串，`a` 和 `b` 都不会被移动，之后还能继续使用。
- `==` `!=` 比较内容是否相同，`<` `<=` `>` `>=` 按字节顺序比较，结果是 `bool`。
- 字符串不能和整数相加，例如 `"n=" + 1` 会报错；用插值 `"n={n}"`。

## 第 8 节：数组、列表与字典

Zamak 有两种数组：长度固定的定长数组 `[i64; N]`，以及可以增减元素的列表 `[T]`。按键存取数据用字典 `{K: V}`，见本节最后。

定长数组：

```zamak
fn main():
    let mut values: [i64; 3] = [10, 20, 30]
    values[1] = values[0] + 5
    let mut i = 0

    while i < 3:
        println(values[i])
        i = i + 1
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

列表 `[T]` 的元素可以是 `i64`、`f64`、`bool`、`string` 或结构体（见第 9 节），可以作为局部变量、函数参数、返回值和结构体字段：

```zamak
fn average(scores: &[i64]) -> i64:
    let mut sum = 0

    for score in &scores:
        sum = sum + score

    sum / scores.len()

fn main():
    let mut scores: [i64] = [90, 72]
    scores.push(85)
    scores[1] = 75
    println(scores.len())
    println(average(&scores))
    let last = scores.pop()
    println(last)
```

```text
3
83
85
```

字符串列表：

```zamak
use std/string

fn main():
    let line = "apple banana  cherry"
    let mut words = string.words(&line)
    words.push("date")

    for word in &words:
        let size = string.char_len(word)
        println("{word}: {size}")

    let first = string.copy(&words[0])
    let sep = ", "
    println(string.join(&words, &sep))
    println(first)
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
- `for x in &xs` 依次访问每个元素：整数和布尔元素按值得到，字符串元素以只读借用得到。`for i, x in &xs` 同时得到下标。循环体内不能增删、移动或独占借用这个列表。
- 要原地修改元素，写 `for x in &mut xs`（列表要是 `let mut`）：给 `x` 赋值就是改元素，例如 `x = x * 10`；字符串元素可以整体赋值，结构体元素可以改字段，如 `item.qty += 1`。
- 函数参数写 `&[T]`（只读）或 `&mut [T]`（可修改），调用时传 `&xs` 或 `&mut xs`；参数写 `[T]` 表示把列表整个移交给函数。
- `for x in xs:` 和 `for x in &xs:` 一样，都只借用列表，不会把它移走。
- 元组把几个值放在一起：类型写 `(i64, string)`，值写 `(3, "three")`，`t.0`、`t.1` 取各项。函数可以返回元组来返回多个值，`let (q, r) = divmod(17, 5)` 一次拆开，不要的项写 `_`。元组可以放进列表（`[(string, i64)]`），也能比较和打印。
- `xs.sort()` 从小到大排序（整数、浮点、布尔、字符串元素；排序稳定，NaN 排在最后），`xs.reverse()` 反转，`xs.swap(i, j)` 交换两个元素。`xs.sort_by(fn(a, b): a.age < b.age)` 按自定义规则排序，任何元素都行（包括结构体），函数返回真表示 a 排在 b 前面，排序同样稳定。
- `xs.contains(v)` 判断有没有，`xs.index_of(v)` 返回第一次出现的下标，找不到是 -1；字符串元素传借用，例如 `names.contains("amy")`。
- `xs.insert(i, v)` 插到下标 `i` 前面（`i` 可以等于长度，即追加），`xs.remove(i)` 删除并返回下标 `i` 的元素，`xs.clear()` 清空，`xs.extend(ys)` 把 `ys` 的元素整个移到末尾。
- `xs.slice(a, b)` 复制出下标 `a` 到 `b`（不含 `b`）的新列表，越界时程序终止。排序、查找和切片要比较或复制元素，结构体列表还不能用。
- 方法用在变量、字段或调用结果上，例如 `bag.items.push(x)`、`string.split(&s, ",").len()`、`xs.slice(0, 2).len()`。

字典按键存取值，下面统计每种颜色出现的次数：

```zamak
use std/string

fn main():
    let text = "red blue red green blue red"
    let words = string.words(&text)
    let mut counts: {string: i64} = {}

    for word in &words:
        counts[word] = counts.get(word, 0) + 1

    for color in &counts:
        let n = counts[color]
        println("{color}: {n}")

    println(counts.contains("red"))
    counts.remove("blue")
    println(counts.len())

    let ages = {"amy": 30, "bob": 25}
    println(ages["amy"])
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

- 类型写成 `{键类型: 值类型}`。键可以是 `i64` 或 `string`，值可以是 `i64`、`f64`、`bool`、`string`、结构体，也可以是列表或字典。
- 容器可以嵌套：`[[i64]]` 是二维表，`{string: [string]}` 是分组，`[{string: i64}]` 是一列记录。`grid[i][j]` 读写内层元素（最多两层下标，字符串元素读出来是副本），`grid[i].push(x)`、`groups[k].len()` 在元素上调用方法，`for row in &grid` 拿到 `&[i64]`，`&grid[i]` 借给函数。内层容器不能用 `grid[i]` 直接取出来，那会把它从外层移走。
- 字面量写成 `{"amy": 30, "bob": 25}`，条目之间用逗号，可以分行写。空字典必须标注类型：`let mut m: {string: i64} = {}`。
- `m[k]` 读取，键不存在时程序终止（`zam: key not found`）；不确定键是否存在时用 `m.get(k, 默认值)` 或先 `m.contains(k)`。
- `m[k] = v` 插入或替换；`m.remove(k)` 删除并返回是否删到了；`m.len()` 返回条目数；`m.keys()` 与 `m.values()` 按插入顺序返回键和值的列表（字符串值是副本，结构体值的字典没有 `values`）；`m.clear()` 清空。
- 字符串键按只读借用传入：变量写 `m[&name]`，字面量直接写 `m["red"]`，`for` 循环得到的键直接写 `m[k]`。新键插入时字典会复制一份。
- `for k in &m` 按插入顺序遍历键，所以输出顺序是确定的；`for k, v in &m` 同时得到键和值，`for k, v in &mut m` 可以改值（键不能改）。循环体内不能增删这个字典的条目。
- 字符串值不能用 `m[k]` 取出，借用 `&m[k]`，或用 `m.get(k, "")` 得到一份副本。

## 第 9 节：结构体

```zamak
struct Point:
    x: i64
    y: i64

fn move_by(point: Point, dx: i64) -> Point:
    let mut result = point
    result.x = result.x + dx
    result

fn main():
    let mut p = Point(x: 1,
        y: 2,
    )
    println(p.x)
    p.y = p.y + 3
    println(p.y)
    let moved = p
    println(moved.x + moved.y)
    let q = move_by(moved, 10)
    println(q.x)
```

```text
1
5
6
11
```

要点：

- 结构体声明写成 `struct Point:`，下面每行缩进写一个字段。
- 构造写成 `Point(x: 1, y: 2)`：像调用一样用括号，每个字段都写名字，字段之间用逗号。字段多时可以分行写，括号里的换行和缩进不受限制。
- 结构体名首字母大写，函数名首字母小写，所以 `Point(...)` 一眼就能和函数调用区分开。
- 字段通过 `.` 访问。修改字段要求所属变量可变，如本例的 `let mut p`。
- 结构体使用移动语义：执行 `let moved = p` 后，再读取 `p` 会报 `use after move`。对于可变变量，可以通过整体重新赋值恢复其有效值，例如 `p = Point(x: 1, y: 2)`。
- 嵌套结构体字段可以连续访问，例如 `group.leader.age = group.leader.age + 1`。
- 方法写在结构体（或枚举）的缩进块里，和字段放在一起：第一个参数 `&self` 只读、`&mut self` 可以改字段、`self` 把值移进方法。调用写 `p.length()`、`p.scale(2.0)`，接收者会按需要自动借用。没有 `self` 的是关联函数，用类型名调用，比如构造函数 `Vec2.new(3.0, 4.0)`。跨模块调用方法要声明成 `pub fn`。完整例子见 `examples/structs/methods.zm`。

### 结构体借用与结构体列表

```zamak
struct Task:
    title: string
    hours: i64
    done: bool

fn summary(t: &Task) -> string:
    let mark = if t.done: "x" else: " "
    "[{mark}] {t.title} ({t.hours}h)"

fn finish(t: &mut Task):
    t.done = true

fn main():
    let mut tasks: [Task] = []
    tasks.push(Task(title: "写文档", hours: 2, done: false))
    tasks.push(Task(title: "修 bug", hours: 5, done: false))
    finish(&mut tasks[1])
    tasks[0].hours += 1
    let mut total = 0

    for t in &tasks:
        println(summary(t))
        total += t.hours

    println("合计 {total}h")
```

```text
[ ] 写文档 (3h)
[x] 修 bug (5h)
合计 8h
```

- 参数写成 `&Task` 是只读借用，写成 `&mut Task` 是可变借用，规则与字符串借用相同：只能作参数，调用返回就结束。经由 `&mut` 可以改字段，也可以整体赋值。
- 列表和字典可以存结构体：`[Task]`、`{string: Task}`。`push` 把结构体移进列表，`pop` 把最后一个移出来。
- `tasks[i].hours` 读写元素的字段，也能用复合赋值；`&tasks[i]`、`&mut tasks[i]` 借用整个元素，`&tasks[i].title` 借用元素的字段；`tasks[i].tags.push(x)` 调用元素字段上的列表或字典方法。
- `for t in &tasks` 里的 `t` 是只读借用。要修改元素写 `for t in &mut tasks`，`t` 是 `&mut Task`，可以改字段。
- 结构体赋值是移动，所以 `let t = tasks[0]` 会报错。要一份独立的副本写 `let t = copy(&tasks[0])`；拿走元素用 `pop`，整体替换写 `tasks[0] = Task(...)`，排序用 `tasks.swap(i, j)`。结构体值的字典不支持 `get`，先用 `contains` 判断，再读 `m[k].field`。
- 元素字段里的列表不能再接下标（`tasks[0].tags[1]`），把 `&tasks[0].tags` 传给函数处理。

### 枚举与 match

```zamak
enum Shape:
    Circle(r: f64)
    Rect(w: f64, h: f64)
    Empty

fn area(s: &Shape) -> f64:
    match s:
        Circle(r): 3.14 * r * r
        Rect(w, h): w * h
        Empty: 0.0

fn main():
    let shapes: [Shape] = [Shape.Circle(1.0), Shape.Rect(2.0, 3.0), Shape.Empty]

    for s in &shapes:
        let name = match s:
            Circle(_): "circle"
            Rect(_, _): "rect"
            _: "empty"
        println("{name}: {area(s)}")
```

```text
circle: 3.14
rect: 6.0
empty: 0.0
```

- `enum` 每行声明一个变体，变体名首字母大写。带数据的变体写成 `Circle(r: f64)`，数据可以是数字、布尔、字符串、结构体、列表或字典。
- 构造时按位置传值：`Shape.Circle(1.0)`；没有数据的变体写 `Shape.Empty`。跨模块写 `shapes.Shape.Circle(1.0)`。
- `match s:` 下面每行一个分支，`Circle(r):` 按位置绑定数据，`_` 表示不绑定。分支要覆盖所有变体，或在最后写 `_:` 兜底，否则检查报 `ZM511`。
- 分支冒号后面可以写一条语句，也可以换行缩进写一块。写成 `let name = match s:` 时，每个分支冒号后面是一个值。
- `match` 不移动被匹配的值：数字和布尔数据按值绑定，字符串、结构体等数据是只读借用。分支里不能修改或移走被匹配的变量。
- 枚举和结构体一样赋值时移动，可以放进列表、字典和结构体字段，也能用 `&Shape` 借用。枚举可以用 `==` 比较（标签相同且数据相等），取出数据用 `match`。
- `match` 也能按值分支：整数、字符（如 `'a'`）、字符串、布尔字面量，`|` 列出几个值，`_` 接住其余。语句形式可以没有 `_`（都不匹配时什么也不做）；表达式形式 `let v = match n:` 要有 `_`，或者把 `true`、`false` 都列出来。`match` 后面是调用时只求值一次。
- 结构体、枚举、列表和字典都能用 `==`、`!=` 逐项比较（字典与插入顺序无关），也能直接 `println` 或放进插值，输出的写法和构造时一样，例如 `Point(x: 1, y: 2)`、`Shape.Circle(1.5)`、`["a", "b"]`、`{"amy": 30}`；里面的字符串带引号。`copy(&x)` 深复制一份，改副本不影响原值。比较、打印和复制都只读取，不移动变量。

## 第 10 节：所有权与借用

```zamak
fn show(text: &string):
    println(text)

fn update(text: &mut string):
    text = "被借用修改过"

fn consume(text: string):
    println(text)

fn main():
    let mut message = "原始内容"
    show(&message)
    update(&mut message)
    show(&message)
    let moved = message
    consume(moved)
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
- 当前 `string` 和结构体使用移动语义，`i64`、`f64` 和 `bool` 使用复制语义。字符串字段和整个结构体都可以借用（见第 9 节）；单独移出拥有所有权的字段尚不支持。

## 第 11 节：错误处理

函数在返回类型后加 `!错误类型`，表示调用可能失败；当前错误类型仅支持 `string`。

下面的 `parse` 仅用于演示错误处理：它拒绝空字符串，并对所有非空字符串返回固定值 `42`，不执行整数解析。它不是可用于输入校验的解析器；真正解析整数用 `string.parse_i64`（见第 13 节）。

```zamak
use std/string

fn parse(text: &string) -> i64!string:
    let empty = ""

    if string.equal(text, &empty):
        fail "empty input"

    return 42

fn load(text: &string) -> i64!string:
    let value = parse(text)?
    return value

fn main() -> !string:
    let text = "42"
    let value = load(&text)?
    println(value)
```

```text
42
```

错误处理语法：

- `i64!string`：成功返回 `i64`，失败返回 `string` 错误信息。错误类型也可以是自己定义的枚举或结构体，比如 `-> Config!ConfigError`，`fail ConfigError.Missing(key)` 构造错误，`catch e:` 里用 `match e` 分情况处理（见 `examples/values/custom_errors.zm`）。
- `fail "empty input"`：产生错误并立即从当前函数返回，只能在声明了错误类型的函数中使用。
- `parse(text)?`：成功时取得返回值，失败时立即从 `load` 返回同一错误。使用 `?` 的函数必须声明相同的错误类型。

`main` 可以声明为 `-> !string`，表示不返回成功值，但可能失败。传播到 `main` 且未处理的错误会写入 stderr，程序以退出码 1 结束。将上例的 `"42"` 改为 `""` 后，`zam run` 在 stderr 输出下面一行，并以退出码 1 结束（`zam run` 原样带回程序的退出码）：

```text
empty input
```

使用 `catch` 可以在调用处处理错误。以下 `parse` 与前例相同，仍是返回固定值的占位函数：

```zamak
use std/string

fn parse(text: &string) -> i64!string:
    let empty = ""

    if string.equal(text, &empty):
        fail "empty input"

    return 42

fn main():
    let text = "42"
    let value = parse(&text) catch e:
        println("解析失败：{e}")
        return
    println(value)
```

成功时输出 `42`。将 `text` 的初值改为 `""` 后，错误分支输出以下内容，并从 `main` 返回；此时程序正常结束：

```text
解析失败：empty input
```

要点：

- `catch e:` 下面缩进的块里，`e` 是错误值，只在块内可见。
- `catch` 块必须在所有路径上使用 `return` 从当前函数返回；若块内有条件分支，每个分支都必须返回。它不支持用末尾表达式为 `let value` 提供恢复值。成功时才会给 `value` 赋值并继续执行后续语句。
- 错误类型可以是 `string`、结构体或枚举，不支持错误堆栈。可失败调用必须使用 `?` 传播或使用 `catch` 处理。

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

fn main():
    greeting.hello()
    io.println("你好，Zamak")
```

`src/greeting.zm`：

```zamak
pub fn hello():
    println("hello from zamak")
```

在 `hello` 项目根目录执行 `zam run`，得到以下程序输出。如果当前位于仓库根目录，也可以执行 `zam run examples/hello` 运行对应的仓库示例：

```text
hello from zamak
你好，Zamak
```

要点：

- `entry` 是入口文件，`modules` 是模块所在目录。
- **模块名就是文件名**：`greeting.zm` 用 `use greeting` 引入，调用写成 `greeting.hello()`。
- 跨模块可见的函数、结构体、常量要加 `pub`；导入后写 `config.LIMIT` 使用别的模块的常量。
- 跨模块用类型要写模块前缀：`leader: model.Person`。
- 文件后缀统一是 `.zm`；入口必须是 `.zm`（否则报 `entry must be a .zm file`）。
- 对于单文件程序，`zam check`/`build`/`run`/`fmt` 也接受单个 `.zm` 文件。

### 使用别人写的库

别人发布在 git 仓库里的 Zamak 库，可以在项目目录里用一条命令加进来：

```text
zam add mathx https://github.com/you/mathx v1.0
```

`zam.toml` 会多出一节：

```toml
[dependencies]
mathx = { git = "https://github.com/you/mathx", rev = "v1.0" }
```

然后像用自己的模块一样导入：`use mathx` 或 `use mathx/geo`，调用 `mathx.double(21)`。要点：

- `zam` 把 `v1.0` 解析成具体的提交，记在 `zam.lock` 里。把 `zam.lock` 一起提交，别人构建时用的就是同一份代码。
- 想换到最新版本时运行 `zam update`。
- 本地目录也能当依赖：`zam add util ../util`。
- 自己写库：建一个普通项目，`zam.toml` 里可以不写 `entry`，把要给别人用的函数标成 `pub`。

## 第 13 节：文件、标准输入与命令行参数

读取命令行参数指定的文件，统计后写出报告：

```zamak
use std/env
use std/fs
use std/string

fn main() -> !string:
    if env.arg_count() < 1:
        fail "usage: zam run main.zm -- <file>"

    let path = env.arg(0)?
    let text = fs.read(&path)?
    let bytes = string.byte_len(&text)
    let chars = string.char_len(&text)
    let report = "{path}: {bytes} bytes, {chars} chars"
    println(report)
    let target = "report.txt"
    fs.write(&target, &report)?
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

fn main() -> !string:
    let mut line = ""
    let mut total = 0

    while io.read_line(&mut line)?:
        let text = string.trim(&line)

        if string.byte_len(&text) > 0:
            total = total + string.parse_i64(&text)?

    println(total)
```

在 pwsh 中执行 `"10`n20`n`n-5" | zam run main.zm`，输出 `25`。

拆分 `key=value`：

```zamak
use std/string

fn main():
    let entry = "  lang=Zamak  "
    let text = string.trim(&entry)
    let separator = "="
    let at = string.find(&text, &separator)
    let key = string.slice(&text, 0, at)
    let value = string.slice(&text, at + 1, string.byte_len(&text))
    println("{key} -> {value}")
    println(string.byte_at(&text, 0))
```

```text
lang -> Zamak
108
```

| 函数 | 说明 |
| --- | --- |
| `env.arg_count() -> i64` | 程序参数个数，不含程序名 |
| `env.arg(i) -> string!string` | 第 `i` 个参数，从 0 开始；越界返回错误 |
| `env.var(&name) -> string!string` | 读取环境变量；没有设置时返回错误 |
| `env.exit(code)` | 立即结束程序并返回退出码（0–255）；写在 `catch` 块末尾时不用再写 `return` |
| `abs(x)`、`min(a, b)`、`max(a, b)` | 不用导入，整数和浮点数都能用；两个参数类型要相同 |
| `fs.read(&path) -> string!string` | 读取整个文件；打不开或读取失败返回错误 |
| `fs.write(&path, &text) -> !string` | 创建或覆盖文件并写入全部内容 |
| `io.read_line(&mut line) -> bool!string` | 读一行到 `line`，去掉行尾的 `\n` 或 `\r\n`；输入结束时返回 `false` 并把 `line` 置空 |
| `io.read_all() -> string!string` | 读取全部标准输入 |
| `string.parse_i64(&s) -> i64!string` | 解析十进制整数，可带 `+`/`-`；含空白或超出 `i64` 范围时返回错误 |
| `string.parse_f64(&s) -> f64!string` | 解析十进制浮点数，如 `-2.5`、`.5`、`1e3`；含空白、`inf`、`nan` 或超出范围时返回错误 |
| `fmt.fixed(x, digits) -> string` | 把 `f64` 格式化成 `digits` 位小数（0 到 30） |
| `string.trim(&s) -> string` | 去掉首尾的空格、制表符和换行，返回新字符串 |
| `string.byte_at(&s, i) -> i64` | 第 `i` 个字节（0–255）；越界时程序终止 |
| `string.slice(&s, start, end) -> string` | 复制字节区间 `[start, end)`；越界或切在多字节字符中间时程序终止 |
| `string.find(&s, &needle) -> i64` | 第一次出现的字节位置；找不到返回 -1，空 `needle` 返回 0 |
| `string.find_from(&s, &needle, start) -> i64` | 从字节位置 `start` 开始查找；`start` 超出 `0..=byte_len` 时程序终止 |
| `string.starts_with(&s, &p)` / `string.ends_with(&s, &p)` / `string.contains(&s, &p)` | 返回 `bool` |
| `string.replace(&s, &from, &to) -> string` | 替换所有不重叠的出现；`from` 为空时原样复制 |
| `string.lower(&s)` / `string.upper(&s) -> string` | 只转换 ASCII 字母，其他字符原样保留 |
| `string.copy(&s) -> string` | 复制出一份独立的字符串 |
| `string.split(&s, &sep) -> [string]` | 按分隔符切分，保留空段；空分隔符时整串作为唯一一项 |
| `string.words(&s) -> [string]` | 按空白切分，跳过连续空白 |
| `string.lines(&s) -> [string]` | 按行切分，去掉行尾 `\r`，末尾换行不产生空行 |
| `string.join(&list, &sep) -> string` | 用分隔符连接字符串列表 |
| `string.chars(&s) -> [string]` | 拆成字符，每个字符是一个字符串（`"你好"` 得到两项） |
| `string.codepoints(&s) -> [i64]` / `string.from_code(n) -> string` | 字符与 Unicode 码点互转；`from_code` 遇到不合法的码点时程序终止 |
| `string.bytes(&s) -> [i64]` / `string.from_bytes(&list) -> string!string` | 与 UTF-8 字节互转；字节超出 0–255 或不是合法 UTF-8 时 `from_bytes` 返回错误 |
| `string.repeat(&s, n) -> string` | 重复 `n` 次，`n` 小于 1 得到空串 |
| `string.count(&s, &sub) -> i64` | 不重叠地统计出现次数 |
| `string.pad_left(&s, width)` / `string.pad_right(&s, width) -> string` | 按字符数补空格到 `width`，用来对齐输出（中文字符算一个字符，显示时占两格） |
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
  zam test [project|file] [filter]
  zam add <name> <git-url|directory> [rev]
  zam update [project]
  zam new <directory> [--template ai]
  zam check --json [project|file]
  zam build --json [project|file]
  zam demo
  zam --version
```

- `zam check`：只做词法、语法、类型与所有权检查，不生成机器码，因此不需要 C 编译器，适合在修改源码后快速验证。成功打印 `CHECK OK main`。
- `zam build`：生成可执行文件，路径以 `OUTPUT <path>` 写到 stderr，可以直接运行。
- `zam run`：先构建再运行，程序输出就是它的 stdout；`--` 之后的参数原样传给程序，例如 `zam run main.zm -- 笔记.txt`。
- `zam fmt`：就地整理格式。缩进统一成 4 个空格（Tab 先换成空格），括号里的续行按括号层数缩进；多行的 `if`/`for`/`while`/`match` 前后各留一个空行（块里第一条、紧跟在注释后面、`elif`/`else` 之前、块的末尾除外），连续空行压成一个，顶层声明之间留一个空行；去掉行尾空白，结尾留一个换行，保留原来的 CRLF/LF。注释和字符串内容原样保留；语法有错的代码会被拒绝而不会被改写。修改过的文件通过 `FORMAT <path>` 写入 stderr。它不调整行内空格，也不自动折行。
- `zam test`：运行所有名字以 `test_` 开头的函数，每个测试单独运行，一个失败不影响其他；最后打印 `test result: ok. 3 passed; 0 failed`，有失败时退出码非 0。后面跟一段文字时只运行名字里含它的测试，例如 `zam test . parse`。测试里用 `assert 条件` 检查结果，失败时会显示条件原文；`a == b` 还会显示两边的值。示例见 `examples/testing`。
- `zam add <名字> <git 地址或目录> [rev]`：添加依赖并立即拉取；`zam update`：把每个 git 依赖更新到 `rev` 的最新提交（见第 12 节）。
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
fn main():
    let name = "Zamak"
    let other = name
    println(name)
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
| `expected ':' at the end of the if line` | 块头末尾漏了冒号 | 在 `if`、`for`、`fn` 等行末加 `:` |
| `blocks are indented, not wrapped in braces` | 用了 `{ }` 写代码块 | 行末的 `{` 换成 `:`，块缩进 4 格，删掉 `}` |
| `expected an indented block after the if line` | 冒号后面换了行，下一行却没有缩进 | 把块里的行多缩进 4 个空格 |
| `unexpected indent` | 不在块里的行多缩进了 | 和同一块的其他行对齐 |
| `unindent does not match any outer indentation level` | 缩进退回时没有对齐到外层任何一层 | 和外层某个块的行对齐 |
| `indentation contains a tab` | 缩进里有 Tab | 运行 `zam fmt` 把 Tab 换成空格 |
| `the indentation may have been lost when the code was copied` | 整个文件几乎没有缩进，多半是复制时丢了 | 按块重新缩进 |
| ``expected `elif` instead of `else if` `` | 写了 `else if` | 改成 `elif` |
| ``expected `and` instead of `&&` `` | 用了 `&&`、`\|\|` 或前缀 `!` | 改成 `and`、`or`、`not` |
| ``expected `==` to compare in the if condition`` | 条件里写了单个 `=` | 改成 `==` |
| `'(' was never closed` | 括号没有闭合 | 补上对应的右括号 |
| `use after move: a` | 变量的所有权已转移 | 用 `&a` 借用，或重新赋值 |
| `cannot interpolate this value` | 插值中使用了结构体或数组等不支持的值 | 使用字符串、借用字符串、整数或布尔字段 |
| `borrow escape: functions cannot return references` | 函数尝试返回 `&string`/`&mut string` | 返回拥有所有权的 `string` |
| `arrays are only supported as local values` | 把数组当参数或返回值 | 改成多个标量参数，或在函数内建数组 |
| `fallible call main:f must be handled with ?` | 可失败调用未被处理 | 加 `?`，或用 `catch` 就地处理 |
| `? cannot propagate the error of main:f from this function` | 当前函数未声明错误类型，或错误类型不一致 | 函数签名写 `-> T!string` |
| `fail requires a function that declares an error type` | 在不可失败的函数里用了 `fail` | 给函数加 `!string` |
| `catch block for f must return a value` | `catch` 块存在未返回的路径 | 确保所有路径都从当前函数 `return` |
| `unsupported error type` | 用了 `!i64` 之类的错误类型 | 错误类型用 `string`、结构体或枚举 |
| `entry must be a .zm file` | `zam.toml` 的入口后缀不对 | 改成 `.zm` |
| `zam.toml: 系统找不到指定的文件` | 目录内没有 `zam.toml` | 用 `zam new` 建项目，或直接传单个 `.zm` 文件 |
| `project name must be an identifier` | `zam new` 的目录名不是合法标识符 | 使用合法标识符，例如 `hello_app`；不要使用连字符或以数字开头 |
| `<dir>: already exists` | 目标目录已存在 | 使用尚不存在的目标目录 |
| 程序输出一行错误后以退出码 1 结束 | 未处理的错误传播到 `main` | 检查错误信息和输入；需要处理时在调用处使用 `catch` |
| `cannot open <path>` | `fs.read`/`fs.write` 打不开文件 | 检查路径是否存在、是否是目录、是否有权限 |
| `invalid integer: <text>` | `string.parse_i64` 的输入不是十进制整数 | 先用 `string.trim` 去掉空白，再解析 |
| `+ mixes i64 and f64` | 整数和浮点数直接相加或比较 | 用 `f64(n)` 或 `i64(x)` 转换一边，整数字面量写成 `2.0` |
| `zam: f64 value out of i64 range` | `i64(x)` 的 `x` 是 NaN 或超出 `i64` 范围 | 转换前先检查范围，或用 `math.is_nan` 排除 NaN |
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

- `match` 匹配枚举变体，或者整数、字符、字符串、布尔字面量（`1 | 2 | 3:`、`"go":`），还不支持区间和守卫条件；尚不支持 trait/接口；`for` 支持整数区间 `a..b` 以及遍历 `&列表`、`&字典`。
- 尚不提供包管理器、交互式执行环境（REPL）、文档生成工具或重构工具。
- 标准库包括 `std/string`（`byte_len`、`char_len`、`equal`、`parse_i64`、`parse_f64`、`trim`、`byte_at`、`slice`、`find`、`find_from`、`starts_with`、`ends_with`、`contains`、`replace`、`lower`、`upper`、`copy`、`split`、`words`、`lines`、`join`、`chars`、`codepoints`、`from_code`、`bytes`、`from_bytes`、`repeat`、`count`、`pad_left`、`pad_right`）、`std/io`（`println`、`read_line`、`read_all`）、`std/fmt`（`println`、`fixed`）、`std/math`（见第 3 节）、`std/fs`（`read`、`write`）和 `std/env`（`arg_count`、`arg`、`args`、`var`、`exit`），以及不用导入的 `abs`、`min`、`max`，没有目录遍历、网络和时间函数。
- 错误类型可以是 `string`、结构体或枚举，没有错误堆栈。
- 定长数组仅支持 `i64` 局部值；列表 `[T]` 的元素与字典 `{K: V}` 的值可以是 `i64`、`f64`、`bool`、`string` 或结构体，字典键只能是 `i64` 或 `string`；还没有嵌套容器（`[[i64]]`），元素字段里的列表不能再接下标。调用结果上可以接着调用方法。
- 所有权检查覆盖 `string`、结构体、列表和字典；尚不支持线程、`unsafe` 和外部函数接口（FFI）。
- 编辑器支持为 VS Code 扩展（`editors/vscode/`）：语法高亮、冒号后自动缩进、按缩进折叠；没有语言服务器、补全和调试器。
- 本教程采用源码安装；安装步骤和平台工具链要求见 [安装说明](PREVIEW.md)。

## 附录：示例与测试

仓库的 `examples/` 目录提供以下可运行示例。在仓库根目录执行 `zam run` 并指定示例目录即可运行：

| 示例 | 内容 |
| --- | --- |
| `examples/hello/` | 多模块项目、`use`、`io.println` |
| `examples/ownership/` | `&string`、`&mut string`、移动 |
| `examples/structs/` | 结构体声明、字段赋值、嵌套、跨模块类型、结构体列表与借用 |
| `examples/control/` | 条件、`while`、`break`/`continue`、函数、枚举与 `match` |
| `examples/values/` | 变量、运算符、浮点数与 `std/math`、字符串、数组、格式化、错误值、`catch` |

集成测试位于 `zam/tests/`。其中 `tutorial.rs` 检查本教程的 Zamak 代码块，`errors.rs` 覆盖 `T!string`、`fail`、`?`、`catch` 的成功与失败路径，`source_format.rs` 覆盖 `zam fmt` 的行为。验证新的写法时，可以先保存为独立 `.zm` 文件并运行 `zam check 文件名.zm`；需要确认运行行为时再使用 `zam run`。
