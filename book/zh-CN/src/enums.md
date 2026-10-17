# 枚举、Option 与 match

枚举是这样一种类型：它的值是几个变体中的一个，每个变体可以带自己的数据。枚举配合 `match`，可以精确地表达「是这几种情况之一」，编译器还会确保你处理了每一种情况。

## 声明枚举

每行列出一个变体。变体可以带有名字的数据，也可以不带：

```zamak
enum Shape:
    Circle(r: f64)
    Rect(w: f64, h: f64)
    Empty

fn main():
    let shapes: [Shape] = [Shape.Circle(1.5), Shape.Rect(2, 3), Shape.Empty]
    for s in shapes:
        println(s)
```

```text
Shape.Circle(1.5)
Shape.Rect(2.0, 3.0)
Shape.Empty
```

创建值时写枚举名、点和变体名，数据按位置传入：`Shape.Rect(2, 3)`。

## 用 match 读取数据

`match` 选出与变体对应的分支，并把数据绑定到名字上：

```zamak
enum Shape:
    Circle(r: f64)
    Rect(w: f64, h: f64)
    Empty

fn area(s: &Shape) -> f64:
    match s:
        Circle(r): 3.14159 * r * r
        Rect(w, h): w * h
        Empty: 0.0

fn main():
    println(area(Shape.Circle(1)))
    println(area(Shape.Rect(2, 3)))
    println(area(Shape.Empty))
```

```text
3.14159
6.0
0.0
```

`match` 借用这个值，所以分支读取数据时不会把它移走。不需要的数据用 `_` 代替名字跳过。

## 每种情况都要处理

`match` 漏掉某个变体时，程序无法通过编译：

```zamak,fails
enum Light:
    Red
    Yellow
    Green

fn next(light: &Light) -> Light:
    match light:
        Red: Light.Green
        Green: Light.Yellow

fn main():
    println(next(Light.Red))
```

```text
error[ZM511]: match on Light does not cover Yellow; add an arm for each or a _ arm
 --> main.zm:7:5
  |
7 |     match light:
  |     ^^^^^^^^^^^^
  = help: add an arm for each missing variant or a final `_:` arm, and remove arms after `_`
```

这正是用枚举而不用字符串或数字的主要原因：以后新增一个变体时，编译器会指出每一个需要补分支的 `match`。最后写一个 `_` 分支可以处理剩下的所有变体，但这样就失去了这份帮助，所以只在剩下的情况确实都一样时才用。

## 枚举的方法

和结构体一样，枚举也可以有方法，写在变体之后：

```zamak
enum Light:
    Red
    Yellow
    Green

    fn next(&self) -> Light:
        match self:
            Red: Light.Green
            Green: Light.Yellow
            Yellow: Light.Red

    fn seconds(&self) -> i64:
        match self:
            Red: 30
            Yellow: 3
            Green: 25

fn main():
    let mut light = Light.Red
    for i in range(4):
        println(f"{light} for {light.seconds()}s")
        light = light.next()
```

```text
Light.Red for 30s
Light.Green for 25s
Light.Yellow for 3s
Light.Red for 30s
```

两个枚举值是同一个变体且数据相等时，用 `==` 比较结果为真。

## Option：可能不存在的值

Zamak 没有 `null`。可能不存在的值，类型是 `Option[T]`，它有两个变体：`Some(value)` 和 `None`：

```zamak
fn find(names: &[string], wanted: &string) -> Option[i64]:
    for i, name in names:
        if name == wanted: return Option.Some(i)
    Option.None

fn main():
    let names: [string] = ["Ada", "Grace", "Linus"]
    for wanted in ["Grace", "Guido"]:
        match find(names, wanted):
            Some(i): println(f"{wanted} is at {i}")
            None: println(f"{wanted} is missing")
```

```text
Grace is at 1
Guido is missing
```

因为结果是 `Option[i64]` 而不是 `i64`，你不会不小心把它当数字用。`match` 迫使你决定值不存在时该怎么办。

## 泛型枚举

和结构体一样，枚举也可以带类型参数。`Option[T]` 就是一个，你也可以声明自己的：

```zamak
enum Result[T]:
    Ok(value: T)
    Err(message: string)

fn half(n: i64) -> Result[i64]:
    if n % 2 != 0: return Result.Err(f"{n} is odd")
    Result.Ok(n // 2)

fn main():
    for n in [10, 7]:
        match half(n):
            Ok(v): println(v)
            Err(m): println(m)
```

```text
5
7 is odd
```

处理错误时，Zamak 有比这更简短的专门语法，见[处理错误](errors.md)。

## 对其他值使用 match

如[控制流](control-flow.md)一章所示，`match` 也能用于整数、字符串、字符和布尔值。用 `|` 合并几种可能：

```zamak
fn command(word: &string) -> string:
    match word:
        "quit" | "exit": "bye"
        "help": "commands: help, quit"
        _: "unknown command"

fn main():
    println(command("exit"))
    println(command("help"))
    println(command("dance"))
```

```text
bye
commands: help, quit
unknown command
```

## 小结

- `enum Name:` 列出变体，变体可以带数据。用 `Name.Variant(...)` 创建值。
- `match` 绑定变体的数据，必须覆盖每个变体，或者以 `_` 结尾。
- `Option[T]` 取代 null：值要么是 `Some(value)`，要么是 `None`。
- 枚举可以有方法和类型参数。
