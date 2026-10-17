# Structs and methods

A struct groups related values under one name. Each value is a field with its own name and type.

## Declare and create a struct

Declare the fields one per line, and create a value by naming every field:

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

`println` shows the struct's name and fields, which makes debugging easy. Two structs compare with `==` field by field.

## Change fields

To change a field, the variable must be declared with `let mut`:

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

## Methods

Write methods inside the struct block, after the fields. The first parameter says how the method uses the value:

- `&self` reads it.
- `&mut self` changes it.
- `self` takes ownership of it.

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

- `Point.origin()` has no `self`, so you call it on the type. Functions like this are often used as constructors, such as `Point.new(...)`.
- `p.distance(...)` and `p.shift(...)` borrow `p` automatically; you don't write `&p` or `&mut p` for the receiver. `shift` changes `p`, so `p` must be `let mut`.

## Structs that hold other values

Fields can be strings, lists, maps, and other structs. The struct owns them, and they're freed together with it:

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

## Lists of structs

Read and change a field of an element with `items[i].field`, and loop with `for item in items`:

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

A struct element can't be moved out with `let x = stock[0]`, because the list would be left with a hole. Borrow it with `&stock[0]`, copy it with `copy(&stock[0])`, or take the last one out with `stock.pop()`.

## Generic structs

A struct can take type parameters, written in brackets after its name:

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

Write the type with its parameter, as in `Stack[string]`. Annotate the variable when the value doesn't show the type, as with an empty `items` list.

## Summary

- `struct Name:` declares fields one per line; `Name(field: value, ...)` creates a value.
- Methods go inside the struct and take `&self`, `&mut self`, or `self`. A function without `self` is called on the type.
- A struct owns its fields; lists of structs are read and changed through `items[i].field`.
- `struct Name[T]:` declares a generic struct.
