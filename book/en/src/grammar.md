# Grammar at a glance

What Zamak supports today, on one page. Each section links to the chapter with the full rules and edge cases.

## Blocks

Blocks are delimited by indentation, not braces. A block header (`fn`, `struct`, `if`, `while`, `for`, `match`, `catch`) ends with `:` and the body is indented 4 spaces. Write `pass` for an empty block.

```zamak,ignore
fn greet(name: string) -> string:
    if name == "":
        return "hello"
    return "hello, {name}"
```

A header may be followed by one statement on the same line (`if n < 1: return n`). Tabs in indentation are an error; `zam fmt` fixes them. Braces appear only in dictionary literals and interpolation.

See [Variables and types](variables.md) and [Control flow](control-flow.md).

## Types

| Type | Spelling | Notes |
| --- | --- | --- |
| Integer | `i64` | Overflow and division by zero trap, even in release builds |
| Float | `f64` | IEEE 754; division by zero stops the program |
| Boolean | `bool` | `and`, `or`, `not`, short-circuiting |
| String | `string` | UTF-8, indexes count characters |
| Bytes | `bytes` | `b"..."`, indexes give integers 0–255 |
| List | `[T]` | Growable |
| Dictionary | `{K: V}` | Insertion order; keys are `i64` or `string` |
| Set | `{K}` | `{1, 2}`; values are `i64` or `string` |
| Tuple | `(i64, string)` | Multiple return values |
| Struct | `struct Point:` | Fields declared one per line |
| Enum | `enum Shape:` | Variants may carry data |
| Function | `fn(i64) -> bool` | First-class value |
| Error | `T!E`, `!E` | See below |

See [Numbers and operators](operators.md) and [Lists, maps, sets, and tuples](collections.md).

## Functions

```zamak,ignore
fn add(a: i64, b: i64) -> i64:
    return a + b

fn largest[T](items: &[T]) -> T:      # generic: one copy per type
    ...

fn parse(s: &string) -> i64!string:   # can fail
    ...
```

A parameter is a value, a borrow `&T`, or a mutable borrow `&mut T`. It can have a default, such as `port: i64 = 80`, and a call can pass it by name, such as `connect(host, timeout=1.5)`. Omitting the return type means no return value. The last expression in a body acts as the return value.

See [Functions](functions.md).

## Variables and ownership

```zamak,ignore
let n = 1              # immutable
let mut count = 0       # mutable
count += 1

let a = [1, 2, 3]
let b = a               # a is moved; unusable afterwards
let c = copy(a)         # deep copy
```

Numbers and booleans copy; strings, structs, lists, and dictionaries move.

See [Move and copy](ownership.md).

## Borrowing

A borrow lasts only for the current call and never outlives the function.

```zamak,ignore
fn total(xs: &[i64]) -> i64:     # read-only borrow
    ...

fn push_all(xs: &mut [i64]):      # mutable borrow
    ...
```

A borrow cannot be stored in a struct, returned, or sent across threads. That is why there are no lifetime annotations.

See [Borrowing](borrowing.md).

## Structs and methods

```zamak,ignore
struct Point:
    x: f64
    y: f64

    fn norm(&self) -> f64:
        return math.sqrt(self.x * self.x + self.y * self.y)

fn main():
    let p = Point(x: 3, y: 4)
    println(p.norm())        # 5.0
```

Methods live in the struct's indented block; the first parameter is `&self`, `&mut self`, or `self`. A function without `self` is an associated function, called through the type name: `Point.new(1, 2)`.

See [Structs and methods](structs.md).

## Enums and match

```zamak
use std/math

enum Shape:
    Circle(r: f64)
    Empty

fn area(s: &Shape) -> f64:
    return match s:
        Circle(r): math.pi * r * r
        Empty: 0.0

fn main():
    let c = Shape.Circle(2.0)
    let e = Shape.Empty
    println(area(&c))
    println(area(&e))
```

`match` must cover every variant or end with `_`. `Option[T]` is built in, with variants `Option.Some(v)` and `Option.None`.

See [Enums, Option and match](enums.md).

## Strings

```zamak,ignore
let name = "world"
println("hello, {name}")          # interpolation
println("x = {count + 1}")        # an expression may go in the braces

let parts = "a,b,c".split(",")
println(parts.len())               # 3
println(name.upper())              # WORLD
```

String methods need no import. Python's names work too: `strip`, `startswith`, `endswith`.

See [Strings](strings.md).

## Lists and dictionaries

```zamak,ignore
let mut xs: [i64] = [3, 1, 2]
xs.push(4)
xs.sort()
println(xs[0])          # 1
println(xs[-1])         # 4, negative indexes count from the end
println(xs.slice(1, 3)) // [2, 3]

let mut counts: {string: i64} = {}
counts["a"] = 1
println(counts.get("b", 0))   # 0
```

An empty list or dictionary needs a type annotation. `for x in &xs` borrows without moving.

A set is written `{1, 2, 3}` and its type is `{i64}`.

See [Lists, maps, sets, and tuples](collections.md).

## Errors

```zamak,ignore
fn read(path: &string) -> string!string:
    return fs.read(path)?

fn main() -> !string:
    let text = read("a.txt")?
    println(text)
    return null
```

`T!E` means the call returns `T` or fails with `E`. `fail expr` produces an error immediately, `call?` passes it up, and `call catch e:` handles it in place. `main` may carry an error type; an uncaught error goes to stderr and exits with code 1.

See [Handling errors](errors.md).

## Function values and closures

```zamak,ignore
fn apply(f: fn(i64) -> i64, n: i64) -> i64:
    return f(n)

let double = fn(x: i64) -> i64: x * 2
println(apply(double, 5))     # 10
```

An anonymous function may read the enclosing function's locals. A closure that captures cannot be stored in a struct, returned, or outlive its variables, so no garbage collector is needed.

See [Functions](functions.md).

## Generics

```zamak,ignore
fn largest[T](items: &[T]) -> T:
    ...

struct Pair[A, B]:
    first: A
    second: B

let p = Pair(i64, string)(first: 1, second: "one")
```

Generics are monomorphized today: one copy per type. The call site infers the type arguments from the arguments.

See [Functions](functions.md).

## Calling C

```zamak,ignore
extern fn sqrt(x: f64) -> f64

fn main():
    println(sqrt(16.0))
```

`extern fn` declares a C function that you then call by name. The C standard library works with no setup; other libraries go in `zam.toml` under `[native]`, with sources, headers, and the libraries to link. `extern type Db: "sqlite3", close = sqlite3_close` declares a C handle that is freed when it goes out of scope.

See [Calling C](c-libraries.md).

## Standard library

| Module | Provides |
| --- | --- |
| `std/string` | Split, join, find, trim, case |
| `std/bytes` | Hexadecimal, zero-filled bytes |
| `std/io` | Print without newline, read a line, read all |
| `std/fs` | Read and write text or bytes, append, list, create, remove, rename |
| `std/path` | Join, parent, file name, extension |
| `std/env` | Arguments, environment variables, exit |
| `std/math` | sqrt, pow, trigonometry, rounding, constants |
| `std/time` | Current time, monotonic clock, sleep, ISO format |
| `std/random` | Integers, floats, seeding |
| `std/fmt` | Fixed decimal places |

Python builtins available without an import: `len`, `abs`, `min`, `max`, `sum`, `sorted`, `str`, `int`, `float`, `print`, `println`.

See [Standard library](standard-library.md).

## Not supported yet

- Threads and concurrency
- Networking
- Macros
- Traits, interfaces, inheritance
- Generic specialization, type-level programming
- Returning a borrow from a function (return the value instead)

`zam check` reports each of these with a specific error rather than ignoring it.
