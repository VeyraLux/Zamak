# Variables and types

Every value in Zamak has a type, and the compiler checks types before the program runs. In most places it works the types out for you, so code stays as short as Python while mistakes surface at compile time.

## Bind a value with let

`let` gives a name to a value:

```zamak
fn main():
    let name = "Ada"
    let year = 1843
    println(f"{name}, {year}")
```

```text
Ada, 1843
```

A `let` binding can't change. Assigning to it again is an error that `zam check` reports before the program runs:

```zamak,fails
fn main():
    let count = 1
    count = 2
```

```text
error[ZM302]: cannot assign immutable value count
 --> main.zm:3:5
  |
3 |     count = 2
  |     ^^^^^^^^^
  = help: declare the binding with `let mut` before assigning it
```

## Make a variable mutable

Add `mut` when a variable needs to change:

```zamak
fn main():
    let mut count = 1
    count = count + 1
    count += 10
    println(count)
```

```text
12
```

Compound assignment operators such as `+=`, `-=`, `*=`, and `//=` update a variable in place.

Making mutability explicit tells the reader which values change. When you see `let`, you know the value stays the same for the rest of the block.

## Each name is declared once

A name can't be declared twice in the same function, even in an inner block. This rule prevents a common bug: accidentally hiding an outer variable with an inner one.

```zamak,fails
fn main():
    let total = 0
    let total = 5
```

```text
error[ZM204]: duplicate binding total
 --> main.zm:3:5
  |
3 |     let total = 5
  |     ^^^^^^^^^^^^^
  = help: remove or rename the duplicate declaration
```

To change a value, declare it with `let mut` and assign to it. To hold a different value, pick a more specific name, such as `raw_total` and `total`.

## Basic types

| Type | Values | Example |
| --- | --- | --- |
| `i64` | 64-bit signed integers | `42`, `-7`, `1_000_000`, `0xFF` |
| `f64` | 64-bit floating-point numbers | `3.14`, `2.0`, `1e-9` |
| `bool` | `true` and `false` | `true` |
| `string` | UTF-8 text | `"hello"`, `"你好"` |

Zamak infers the type from the value. To state it, add a type annotation after the name:

```zamak
fn main():
    let ratio: f64 = 1
    let ready: bool = false
    let label: string = "total"
    println(f"{label}: {ratio}, ready: {ready}")
```

```text
total: 1.0, ready: false
```

An annotation is required when the value doesn't show the type, such as an empty list or map:

```zamak
fn main():
    let mut scores: [i64] = []
    scores.push(90)
    println(scores)
```

```text
[90]
```

Notice that `let ratio: f64 = 1` turns the integer `1` into `1.0`. Wherever Zamak expects an `f64`, an integer works too, as in Python.

## Numbers in detail

Integers are 64 bits wide, from about -9.2 × 10¹⁸ to 9.2 × 10¹⁸. You can write them in several ways:

```zamak
fn main():
    println(1_000_000)
    println(0xFF)
    println(0b1010)
    println(0o17)
    println('A')
```

```text
1000000
255
10
15
65
```

- An underscore between digits makes long numbers easier to read.
- `0x`, `0b`, and `0o` start hexadecimal, binary, and octal numbers.
- A character in single quotes, such as `'A'`, is the integer code point of that character. It's handy when you work with bytes.

A floating-point literal needs a digit on both sides of the point, or an exponent: write `0.5`, not `.5`.

## Constants

A constant is a value that's fixed when the program compiles. Declare it at the top level of a file, outside any function, with an uppercase name:

```zamak
const MAX_USERS = 100
const RATE: f64 = 0.25
const GREETING = "welcome"

fn main():
    println(f"{GREETING}: up to {MAX_USERS} users at {RATE}")
```

```text
welcome: up to 100 users at 0.25
```

A constant's value can use literals, operators, and earlier constants, such as `const SECONDS_PER_DAY = 24 * 60 * 60`. The compiler computes it, so a constant costs nothing at run time.

## Naming rules

- Variables, functions, and modules use `lower_snake_case`: `user_count`, `read_file`.
- Types use `UpperCamelCase`: `WordCount`, `Shape`.
- Constants use `UPPER_SNAKE_CASE`: `MAX_USERS`.

The compiler relies on part of this: a name that starts with an uppercase letter, followed by `(`, always creates a struct value, never calls a function.

## Summary

- `let` binds a value that can't change; `let mut` binds one that can.
- A name is declared once per function.
- The basic types are `i64`, `f64`, `bool`, and `string`. Zamak infers them, and you annotate when the value doesn't show the type.
- `const` declares a compile-time constant at the top level.
