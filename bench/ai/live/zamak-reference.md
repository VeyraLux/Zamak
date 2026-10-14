# Zamak language reference

Zamak is a statically typed language compiled to native code. Statements are separated by newlines; there are no semicolons. Comments start with `//` or `#`; there are no block comments.

## Program shape

```
use std/io
use std/string

fn main() -> !string {
    let text = io.read_all()?
    println(string.byte_len(&text))
}
```

`main` takes no parameters. It returns nothing (`fn main()`) or may fail (`fn main() -> !string`). If `main` fails, the error message is written to standard error and the program exits with status 1. To print a line to standard error, use `io.eprintln(&text)`.

## Values and bindings

- Types: `i64`, `bool`, `string` (UTF-8, owned), structs, lists `[T]`, maps `{K: V}`. There are no floats, chars, tuples, enums, generics or closures.
- `let x = 1`, `let mut x = 1`, `let mut x: i64 = 1`. Only `let mut` bindings can be reassigned. A name cannot be declared twice in the same function scope chain (no shadowing).
- Integer literals may use `_` between digits: `1_000_000`. Character literals like `'a'`, `'#'`, `'\n'`, `'\''` are `i64` Unicode code points (useful with `string.byte_at`).
- Operators: `+ - * / %`, `== != < <= > >=`, `&& || !`, unary `-`, parentheses. Compound assignment `+= -= *= /= %=` works on variables, fields and elements. On strings, `+` concatenates (`a + b` makes a new string and moves neither side) and `== != < <= > >=` compare contents. Strings and integers cannot be mixed: `"n=" + 1` is an error; use interpolation.
- Integer overflow, division by zero, and out-of-range indexes stop the program with an error.

## Strings

- Escapes: `\n \r \t \" \\`.
- Interpolation: `"{name}"` inserts a variable or a field path such as `"{p.age}"`. Any expression without string literals or braces may appear inside: `"{a + b}"`, `"{xs[i]}"`, `"{string.upper(&s)}"`. `{{` and `}}` produce literal braces.
- Build strings with `+`, interpolation or `string.join`.

## Ownership and borrowing

- Assigning or passing a `string`, struct, list or map moves it; using it afterwards is an error. `i64` and `bool` are copied.
- Parameters typed `&string`, `&[T]`, `&{K: V}` borrow read-only; `&mut string`, `&mut [T]`, `&mut {K: V}` borrow mutably. At the call site write `&x` or `&mut x` (the owner must be `let mut` for `&mut`). Borrows only exist for the duration of a call: they cannot be stored in variables, struct fields, or returned.
- A `&string` parameter also accepts a string literal or interpolation directly: `string.find(&s, ",")`.
- Inside a function, a parameter that is already a borrow is passed on without `&`: `fn f(s: &string) { g(s) }`.

## Functions

```
fn add(a: i64, b: i64) -> i64 {
    a + b
}
```

The last expression of the function body (or of the final `if`/`else` branches) is the return value; `return expr` also works. Only `pub fn` is visible from other modules (single-file programs do not need `pub`).

## Control flow

- `if cond { } else if cond { } else { }`; conditions must be `bool`. To choose a value, use an if expression: `let v = if c { a } else if d { b } else { e }`; it needs `else`, and each branch is a single expression.
- `while cond { }`, `break`, `continue`.
- `for i in start..end { }` iterates integers from `start` up to but not including `end`.
- `for x in &list { }` iterates list elements; `for k in &map { }` iterates map keys in insertion order. String elements/keys are read-only borrows. The list or map cannot be modified inside that loop; use `for i in 0..xs.len()` to modify by index.

## Errors

- A function that can fail declares `-> T!string` (returns T or an error message) or `-> !string` (only fails).
- `fail "message"` returns an error. Interpolation works: `fail "bad line {n}"`.
- `call()?` returns the error to the caller (the current function must also be declared `!string`).
- `call() catch e { ... }` handles the error in place; `e` is the message string; the block must end every path with `return` or `fail`.

```
let value = string.parse_i64(&text) catch e {
    fail "invalid number: {text}"
}
```

## Structs

```
struct Point {
    x: i64
    y: i64
}
let p = Point { x: 1 y: 2 }
```

Fields are one per line without commas; struct literals list fields without commas. Read and assign fields with `p.x`.

## Lists `[T]`

`T` is `i64`, `bool` or `string`.

- `let mut xs: [i64] = []` (an empty list needs a type annotation), `let xs: [string] = ["a", "b"]`.
- `xs.len()`, `xs.push(v)`, `xs.pop()` (removes and returns the last element), `xs.swap(i, j)`.
- `xs[i]` reads an `i64`/`bool` element and `xs[i] = v` writes. A string element cannot be taken out with `xs[i]`; pass `&xs[i]` to a `&string` parameter or copy it with `string.copy(&xs[i])`.
- Methods are called on variables or fields only, never on call results: write `let parts = string.split(&s, ",")` then `parts.len()`.

## Maps `{K: V}`

`K` is `i64` or `string`; `V` is `i64`, `bool` or `string`.

- `let mut m: {string: i64} = {}` (an empty map needs a type annotation), `let m = {"a": 1, "b": 2}`.
- `m[k]` reads (stops the program if the key is missing); `m[k] = v` inserts or replaces.
- String keys are passed as borrows: `m[&name]`, `m["literal"]`, or a borrowed loop variable directly `m[word]`.
- `m.len()`, `m.contains(k)`, `m.get(k, default)` (value or `default`), `m.remove(k)` (returns `bool`), `m.keys()` (a list of keys in insertion order).
- Counting idiom: `counts[word] = counts.get(word, 0) + 1`.

## Standard library

Import a module with `use std/<name>` and call `name.function(...)`. `println(x)` needs no import and prints one `string`, `i64` or `bool` followed by a newline.

- `std/io`: `io.print(&text)` (no newline), `io.eprintln(&text)` (standard error), `io.read_line(&mut line) -> bool!string` reads the next line into `line` without its trailing `\n` or `\r\n` and returns `false` at end of input (typical loop: `let mut line = ""` then `while io.read_line(&mut line)? { ... }`); `io.read_all() -> string!string`.
- `std/env`: `env.args() -> [string]` (program arguments, without the program name), `env.arg(i) -> string!string`, `env.arg_count() -> i64`.
- `std/fs`: `fs.read(&path) -> string!string`, `fs.write(&path, &text) -> !string`.
- `std/string` (all positions are byte offsets):
  - `byte_len(&s) -> i64`, `char_len(&s) -> i64`, `byte_at(&s, i) -> i64`
  - `equal(&a, &b) -> bool`, `starts_with(&s, &p) -> bool`, `ends_with(&s, &p) -> bool`, `contains(&s, &p) -> bool`
  - `find(&s, &needle) -> i64` and `find_from(&s, &needle, start) -> i64` (-1 when absent)
  - `slice(&s, start, end) -> string` (bytes `start..end`), `trim(&s) -> string`, `lower(&s) -> string`, `upper(&s) -> string`, `replace(&s, &from, &to) -> string`, `copy(&s) -> string`
  - `parse_i64(&s) -> i64!string` (decimal, optional sign, no surrounding spaces)
  - `split(&s, &sep) -> [string]` (keeps empty fields), `words(&s) -> [string]` (splits on whitespace), `lines(&s) -> [string]` (drops `\r`; no empty item after a final newline), `join(&list, &sep) -> string`

## Example

```
use std/io
use std/string

fn main() -> !string {
    let text = io.read_all()?
    let words = string.words(&text)
    let mut counts: {string: i64} = {}
    for word in &words {
        counts[word] = counts.get(word, 0) + 1
    }
    for word in &counts {
        let n = counts[word]
        println("{word}: {n}")
    }
}
```
