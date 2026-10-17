# Enums, Option, and match

An enum is a type whose value is one of several variants. Each variant can carry its own data. Together with `match`, enums let you describe "one of these cases" precisely, and the compiler makes sure you handle every case.

## Declare an enum

List one variant per line. A variant can carry named data or none:

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

Create a value with the enum name, a dot, and the variant, passing the data by position: `Shape.Rect(2, 3)`.

## Read the data with match

`match` picks the arm for the variant and binds its data to names:

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

`match` borrows the value, so arms read the data without moving it. Use `_` in place of a name to skip a piece of data.

## Every case must be handled

If a `match` misses a variant, the program doesn't compile:

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

This is the main reason to use an enum instead of a string or a number: when you add a variant later, the compiler points at every `match` that needs a new arm. A final `_` arm handles all remaining variants, but then you lose that help, so use it only when the remaining cases really are all the same.

## Methods on enums

Like structs, enums can have methods. Put them after the variants:

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

Two enum values compare with `==` when they're the same variant with equal data.

## Option: a value that might be missing

Zamak has no `null`. A value that might be absent has the type `Option[T]`, with the variants `Some(value)` and `None`:

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

Because the result is an `Option[i64]`, not an `i64`, you can't use it as a number by accident. `match` makes you decide what happens when it's missing.

## Generic enums

Like structs, enums can take type parameters. `Option[T]` is one; you can declare your own:

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

For errors, Zamak has dedicated syntax that's shorter than this. See [Handle errors](errors.md).

## match on other values

`match` also works on integers, strings, characters, and booleans, as [Control flow](control-flow.md) showed. Combine alternatives with `|`:

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

## Summary

- `enum Name:` lists variants, which can carry data. Create values with `Name.Variant(...)`.
- `match` binds a variant's data, and it must cover every variant or end with `_`.
- `Option[T]` replaces null: a value is either `Some(value)` or `None`.
- Enums can have methods and type parameters.
