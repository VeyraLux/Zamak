# 结构体与方法

结构体把相关的值归到一个名字下。每个值是一个字段，有自己的名字和类型。

## 声明和创建结构体

每行声明一个字段；创建值时写出每个字段的名字：

```zamak
struct Point:
    x: f64
    y: f64

fn main():
    let p = Point(x: 3, y: 4)
    println(p.x)
    println(p)
```

```text
3.0
Point(x: 3.0, y: 4.0)
```

`println` 会显示结构体的名字和各个字段，调试时很方便。两个结构体用 `==` 比较时逐个字段比较。

## 修改字段

要修改字段，变量必须用 `let mut` 声明：

```zamak
struct Counter:
    name: string
    count: i64

fn main():
    let mut clicks = Counter(name: "clicks", count: 0)
    clicks.count += 1
    clicks.count += 1
    println(f"{clicks.name}: {clicks.count}")
```

```text
clicks: 2
```

## 方法

方法写在结构体的代码块里、字段之后。第一个参数说明方法怎样使用这个值：

- `&self` 读取它。
- `&mut self` 修改它。
- `self` 拿走它的所有权。

```zamak
use std/math

struct Point:
    x: f64
    y: f64

    fn origin() -> Point:
        Point(x: 0, y: 0)

    fn distance(&self, other: &Point) -> f64:
        math.sqrt((self.x - other.x) ** 2 + (self.y - other.y) ** 2)

    fn shift(&mut self, dx: f64, dy: f64):
        self.x += dx
        self.y += dy

fn main():
    let mut p = Point(x: 3, y: 4)
    println(p.distance(Point.origin()))
    p.shift(1, -1)
    println(p)
```

```text
5.0
Point(x: 4.0, y: 3.0)
```

- `Point.origin()` 没有 `self`，所以在类型上调用。这样的函数常用作构造函数，例如 `Point.new(...)`。
- `p.distance(...)` 和 `p.shift(...)` 自动借用 `p`，接收者不用写 `&p` 或 `&mut p`。`shift` 会修改 `p`，所以 `p` 必须是 `let mut`。

## 包含其他值的结构体

字段可以是字符串、列表、字典和其他结构体。结构体拥有这些字段，释放结构体时它们一起释放：

```zamak
struct Team:
    name: string
    members: [string]

    fn add(&mut self, member: &string):
        self.members.push(member)

    fn size(&self) -> i64:
        len(self.members)

fn main():
    let mut team = Team(name: "core", members: [])
    team.add("Ada")
    team.add("Grace")
    println(f"{team.name} has {team.size()} members: {team.members}")
```

```text
core has 2 members: ["Ada", "Grace"]
```

## 结构体列表

用 `items[i].field` 读取和修改元素的字段，用 `for item in items` 遍历：

```zamak
struct Item:
    name: string
    qty: i64

fn main():
    let mut stock: [Item] = [Item(name: "pen", qty: 3), Item(name: "ink", qty: 0)]
    stock[1].qty += 5
    for item in stock:
        println(f"{item.name}: {item.qty}")
```

```text
pen: 3
ink: 5
```

不能用 `let x = stock[0]` 把结构体元素移出来，否则列表里会留下一个空洞。可以用 `&stock[0]` 借用它，用 `copy(&stock[0])` 复制它，或者用 `stock.pop()` 取出最后一个。

## 泛型结构体

结构体可以带类型参数，写在名字后面的方括号里：

```zamak
struct Stack[T]:
    items: [T]

    fn push(&mut self, item: T):
        self.items.push(item)

    fn pop(&mut self) -> T:
        self.items.pop()

    fn is_empty(&self) -> bool:
        len(self.items) == 0

fn main():
    let mut s: Stack[string] = Stack(items: [])
    s.push("first")
    s.push("second")
    while not s.is_empty():
        println(s.pop())
```

```text
second
first
```

写类型时带上参数，例如 `Stack[string]`。值本身看不出类型时（例如 `items` 是空列表），给变量加上标注。

## 小结

- `struct Name:` 每行声明一个字段；`Name(field: value, ...)` 创建值。
- 方法写在结构体里，第一个参数是 `&self`、`&mut self` 或 `self`。没有 `self` 的函数在类型上调用。
- 结构体拥有它的字段；结构体列表通过 `items[i].field` 读写。
- `struct Name[T]:` 声明泛型结构体。
