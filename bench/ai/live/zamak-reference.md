# Zamak language reference

Zamak is a statically typed language compiled to native code. Blocks are defined by indentation, like Python. Statements are separated by newlines; there are no semicolons. Comments start with `//` or `#`; there are no block comments.

## Program shape

```
use std/io
use std/string

fn main() -> !string:
    let text = io.read_all()?
    println(string.byte_len(&text))
```

`main` takes no parameters. It returns nothing (`fn main():`) or may fail (`fn main() -> !string:`). If `main` fails, the error message is written to standard error and the program exits with status 1. To print a line to standard error, use `io.eprintln(&text)`.

## Blocks and indentation

- A line that opens a block ends with `:` — `fn`, `struct`, `enum`, `if`, `elif`, `else`, `while`, `for`, `match`, each `match` arm, and `catch e`.
- After the `:`, either write one statement on the same line (`if n < 0: return 0`) or put the block on the following lines, indented 4 spaces deeper than the opening line. Every line of a block has the same indentation; the block ends at the first line that is indented less.
- Use spaces, not tabs. An empty block is written `pass`.
- Inside `( )`, `[ ]` and `{ }` newlines and indentation are ignored, so long calls, lists and maps can span lines.
- Braces are only used for map literals, map types and string interpolation, never for blocks.

## Values and bindings

- Types: `i64`, `f64`, `bool`, `string` (UTF-8, owned), structs, enums, lists `[T]`, maps `{K: V}`. There are no chars, tuples, generics or closures.
- `let x = 1`, `let mut x = 1`, `let mut x: i64 = 1`. Only `let mut` bindings can be reassigned. A name cannot be declared twice in the same function scope chain (no shadowing).
- Top-level constants: `const LIMIT = 100`, `const RATE: f64 = 0.25`, `const NAME = "x"`. Names are UPPER_CASE; the value is computed at compile time from literals, earlier constants and operators. `pub const` is visible from other modules as `config.LIMIT`.
- Integer literals may use `_` between digits: `1_000_000`. Character literals like `'a'`, `'#'`, `'\n'`, `'\''` are `i64` Unicode code points (useful with `string.byte_at`).
- Operators: `+ - * / %`, `== != < <= > >=`, unary `-`, parentheses. Logical operators are the words `and`, `or`, `not`, as in Python: `if a > 0 and not done:`. They short-circuit, and their precedence from lowest to highest is `or`, `and`, `not`, comparisons. `&&`, `||` and prefix `!` are errors; `!` only appears in error types such as `i64!string`. Bitwise operators on `i64`: `& | ^ ~ << >>` with Python's precedence (`x & 1 == 1` is `(x & 1) == 1`); shift counts must be 0 to 63. Integer literals may be written `0xFF`, `0b1010`, `0o17`. Compound assignment `+= -= *= /= %= &= |= ^= <<= >>=` works on variables, fields and elements. On strings, `+` concatenates (`a + b` makes a new string and moves neither side) and `== != < <= > >=` compare contents. Strings and integers cannot be mixed: `"n=" + 1` is an error; use interpolation.
- Integer overflow, division by zero, and out-of-range indexes stop the program with an error.
- Structs, enums, lists and maps support `==` and `!=` (field by field; maps ignore order) and can be printed with `println(x)` or `"{x}"`, which shows `Point(x: 1, y: 2)`, `Shape.Circle(1.5)`, `[1, 2]`, `{"a": 1}` with strings quoted. `copy(&x)` makes a deep copy of a string, list, map, struct or enum. Comparing, printing and copying never move the value.

## Floats

- `f64` literals need a digit on both sides of the point or an exponent: `1.5`, `0.25`, `2e-3`, `1_000.5`.
- `f64` follows IEEE 754: `+ - * / %` and comparisons never stop the program; dividing by zero gives `inf` or `NaN`.
- There is no implicit conversion between `i64` and `f64`. `x * 2` is an error when `x` is `f64`; write `x * 2.0`. Convert with `f64(n)` and `i64(x)` (truncates toward zero; stops the program on `NaN` or out-of-range values). Round first with `math.round(x)` if needed.
- `println(x)` and `"{x}"` print the shortest text that reads back as the same value: `0.1`, `2.0`, `1e16`, `2.5e-7`. For a fixed number of decimals use `fmt.fixed(x, 2) -> string` (`use std/fmt`).
- `[f64]` lists and `{K: f64}` map values work like `i64` ones. Map keys cannot be `f64`.

## Strings

- Escapes: `\n \r \t \" \\`.
- Interpolation: `"{name}"` inserts a variable or a field path such as `"{p.age}"`. Any expression without string literals or braces may appear inside: `"{a + b}"`, `"{xs[i]}"`, `"{string.upper(&s)}"`. `{{` and `}}` produce literal braces.
- Build strings with `+`, interpolation or `string.join`.

## Ownership and borrowing

- Assigning or passing a `string`, struct, enum, list or map moves it; using it afterwards is an error. `i64`, `f64` and `bool` are copied.
- Parameters typed `&string`, `&[T]`, `&{K: V}`, `&Point` borrow read-only; `&mut string`, `&mut [T]`, `&mut {K: V}`, `&mut Point` borrow mutably. At the call site write `&x` or `&mut x` (the owner must be `let mut` for `&mut`). Borrows only exist for the duration of a call: they cannot be stored in variables, struct fields, or returned.
- A `&string` parameter also accepts a string literal or interpolation directly: `string.find(&s, ",")`.
- Inside a function, a parameter that is already a borrow is passed on without `&`: `fn f(s: &string): g(s)`.

## Functions

```
fn add(a: i64, b: i64) -> i64:
    a + b

fn double(n: i64) -> i64: n * 2
```

The last expression of the function body (or of the final `if`/`elif`/`else` branches) is the return value; `return expr` also works. Function names start with a lowercase letter. Only `pub fn` is visible from other modules (single-file programs do not need `pub`).

## Control flow

```
if n < 0:
    println("negative")
elif n == 0:
    println("zero")
else:
    println("positive")
```

- Conditions must be `bool`; compare with `==` (a single `=` in a condition is an error). There is no `else if`; write `elif`.
- To choose a value, use an if expression on one line: `let v = if c: a elif d: b else: e`; it needs `else`, and each branch is a single expression.
- `while cond:`, `break`, `continue`.
- `for i in start..end:` iterates integers from `start` up to but not including `end`. `for x in xs:` is the same as `for x in &xs:` (it borrows, it does not move).
- `for x in &list:` iterates list elements and `for i, x in &list:` also gives the index; `for k in &map:` iterates map keys in insertion order and `for k, v in &map:` gives keys and values. String elements/keys are read-only borrows.
- `for x in &mut list:` and `for k, v in &mut map:` change elements in place: assigning `x = x * 2` writes the element, struct elements allow `x.field += 1`. Elements cannot be added or removed inside these loops.

## Errors

- A function that can fail declares `-> T!string` (returns T or an error message) or `-> !string` (only fails).
- `fail "message"` returns an error. Interpolation works: `fail "bad line {n}"`.
- `call()?` returns the error to the caller (the current function must also be declared `!string`).
- `call() catch e:` handles the error in place; `e` is the message string; the block must end every path with `return` or `fail`.

```
let value = string.parse_i64(&text) catch e:
    fail "invalid number: {text}"
```

## Structs

```
struct Point:
    x: i64
    y: i64

let p = Point(x: 1, y: 2)
```

Fields are declared one per line. A struct literal names every field, separated by commas: `Point(x: 1, y: 2)`. Read and assign fields with `p.x`.

Methods go inside the struct or enum block, after the fields or variants. The first parameter is `&self` (read), `&mut self` (modify) or `self` (consume); the receiver is borrowed automatically, so call `p.norm()` or `p.scale(2.0)`. A function without `self` is called on the type, such as `Point.new(1, 2)`. Mark methods `pub fn` to call them from other modules.

```
struct Point:
    x: i64
    y: i64

    fn new(x: i64, y: i64) -> Point: Point(x: x, y: y)

    fn sum(&self) -> i64: self.x + self.y

    fn shift(&mut self, d: i64):
        self.x += d
```

## Tuples

- Type `(i64, string)`, value `(3, "three")`, items `t.0`, `t.1`. A function returns several values as a tuple: `fn divmod(a: i64, b: i64) -> (i64, i64): (a / b, a % b)`.
- `let (q, r) = divmod(17, 5)` unpacks a tuple (`_` skips an item); the tuple is moved. Tuples can be stored in lists (`[(string, i64)]`), compared with `==` and printed.

## Enums and match

```
enum Shape:
    Circle(r: f64)
    Rect(w: f64, h: f64)
    Empty

let s = Shape.Circle(2.0)
let area = match s:
    Circle(r): 3.14 * r * r
    Rect(w, h): w * h
    Empty: 0.0
```

- One variant per line; variant names start with an uppercase letter. Build values with positional arguments: `Shape.Rect(1.0, 2.0)`, `Shape.Empty`.
- `match` arms are `Variant(a, b): ...` (bind data by position, `_` skips one) or `_: ...` as the last arm, one arm per line. Arms must cover every variant or end with `_`.
- As a statement, an arm is one statement on the same line or an indented block; as a value (`let x = match ...`), each arm is one expression.
- `match` borrows the value: numbers and booleans are copied into the bindings, strings and structs are read-only borrows. Enums compare with `==` (same variant and equal data); use `match` to read the data.
- Enums move like structs and can be stored in lists, maps and struct fields, or borrowed as `&Shape`.
- `match` also branches on literal values: integers, characters (`'a'`), strings and booleans, with `|` for alternatives: `1 | 2: "small"`, `"quit": return`, `_: ...`. A statement match may omit `_`; a match expression needs `_` (or both `true` and `false`).

## Lists `[T]`

`T` is `i64`, `f64`, `bool`, `string`, a struct, an enum, or another list or map: `[[i64]]`, `[{string: i64}]`, `{string: [string]}`.

- Nested elements: `grid[i][j]` reads and writes an inner element (two levels at most; string elements are read as copies), `grid[i].push(x)` and `m[k].len()` call methods on an inner container, `for row in &grid:` gives `&[i64]`, and `&grid[i]` borrows a row. `grid[i]` alone cannot be taken out.

- `let mut xs: [i64] = []` (an empty list needs a type annotation), `let xs: [string] = ["a", "b"]`.
- `xs.len()`, `xs.push(v)`, `xs.pop()` (removes and returns the last element), `xs.insert(i, v)`, `xs.remove(i)` (removes and returns element `i`), `xs.swap(i, j)`, `xs.reverse()`, `xs.clear()`, `xs.extend(ys)` (moves all of `ys` to the end).
- For `i64`, `f64`, `bool` and `string` elements: `xs.sort()` (ascending, stable), `xs.contains(v)`, `xs.index_of(v)` (-1 when absent), `xs.slice(a, b)` (a new list of elements `a..b`). String arguments are borrows: `names.contains("amy")`.
- `xs[i]` reads an `i64`/`f64`/`bool` element and `xs[i] = v` writes. A string element cannot be taken out with `xs[i]`; pass `&xs[i]` to a `&string` parameter or copy it with `string.copy(&xs[i])`.
- Struct elements: read and write fields with `xs[i].field` (also `+=`), pass `&xs[i]` / `&mut xs[i]` to `&Point` / `&mut Point` parameters, pass `&xs[i].name` to a `&string` parameter, call `xs[i].tags.push(v)`. `for p in &xs` gives a read-only `&Point`. Structs move on assignment: `let p = xs[i]` is an error; write `copy(&xs[i])` for an independent copy, `xs.pop()` to take the last one out, `xs.swap(i, j)` to reorder, and `xs[i] = Point(...)` to replace. `xs[i].tags[j]` is not supported.
- Methods can be called on variables, fields and call results: `parts.len()`, `string.split(&s, ",").len()`, `xs.slice(0, 2).len()`.

## Maps `{K: V}`

`K` is `i64` or `string`; `V` is `i64`, `f64`, `bool`, `string`, a struct, an enum, a list or a map (maps of structs have no `get`; use `m.contains(k)` then `m[k].field`).

- `let mut m: {string: i64} = {}` (an empty map needs a type annotation), `let m = {"a": 1, "b": 2}`.
- `m[k]` reads (stops the program if the key is missing); `m[k] = v` inserts or replaces.
- String keys are passed as borrows: `m[&name]`, `m["literal"]`, or a borrowed loop variable directly `m[word]`.
- `m.len()`, `m.contains(k)`, `m.get(k, default)` (value or `default`), `m.remove(k)` (returns `bool`), `m.keys()` and `m.values()` (lists in insertion order; `values` is not available for struct values), `m.clear()`.
- Counting idiom: `counts[word] = counts.get(word, 0) + 1`.

## Standard library

Import a module with `use std/<name>` and call `name.function(...)`. `println(x)` needs no import and prints one `string`, `i64`, `f64` or `bool` followed by a newline.

- `std/io`: `io.print(&text)` (no newline), `io.eprintln(&text)` (standard error), `io.read_line(&mut line) -> bool!string` reads the next line into `line` without its trailing `\n` or `\r\n` and returns `false` at end of input (typical loop: `let mut line = ""` then `while io.read_line(&mut line)?:`); `io.read_all() -> string!string`.
- `std/env`: `env.args() -> [string]` (program arguments, without the program name), `env.arg(i) -> string!string`, `env.arg_count() -> i64`, `env.var(&name) -> string!string` (an environment variable), `env.exit(code)` (ends the program with status 0 to 255; a `catch` block may end with it instead of `return`).
- `abs(x)`, `min(a, b)`, `max(a, b)` need no import and work on `i64` or `f64` (both arguments of the same type).
- `std/fs`: `fs.read(&path) -> string!string`, `fs.write(&path, &text) -> !string`.
- `std/math` (all `f64`): `sqrt`, `pow(x, y)`, `exp`, `log` (natural), `log10`, `sin`, `cos`, `tan`, `atan`, `atan2(y, x)`, `floor`, `ceil`, `round` (half away from zero), `trunc`, `abs`, `min(a, b)`, `max(a, b)`, `is_nan(x) -> bool`; constants `math.pi`, `math.e`, `math.inf`.
- `std/fmt`: `fmt.fixed(x, digits) -> string` (0 to 30 digits).
- `std/string` (all positions are byte offsets):
  - `byte_len(&s) -> i64`, `char_len(&s) -> i64`, `byte_at(&s, i) -> i64`
  - `equal(&a, &b) -> bool`, `starts_with(&s, &p) -> bool`, `ends_with(&s, &p) -> bool`, `contains(&s, &p) -> bool`
  - `find(&s, &needle) -> i64` and `find_from(&s, &needle, start) -> i64` (-1 when absent)
  - `slice(&s, start, end) -> string` (bytes `start..end`), `trim(&s) -> string`, `lower(&s) -> string`, `upper(&s) -> string`, `replace(&s, &from, &to) -> string`, `copy(&s) -> string`
  - `parse_i64(&s) -> i64!string` (decimal, optional sign, no surrounding spaces), `parse_f64(&s) -> f64!string` (decimal with optional fraction and exponent)
  - `split(&s, &sep) -> [string]` (keeps empty fields), `words(&s) -> [string]` (splits on whitespace), `lines(&s) -> [string]` (drops `\r`; no empty item after a final newline), `join(&list, &sep) -> string`
  - `chars(&s) -> [string]` (one string per character), `codepoints(&s) -> [i64]`, `from_code(n) -> string`, `bytes(&s) -> [i64]`, `from_bytes(&list) -> string!string`
  - `repeat(&s, n) -> string`, `count(&s, &sub) -> i64`, `pad_left(&s, width) -> string` and `pad_right(&s, width) -> string` (pad with spaces to `width` characters)

## Example

```
use std/io
use std/string

fn main() -> !string:
    let text = io.read_all()?
    let words = string.words(&text)
    let mut counts: {string: i64} = {}

    for word in &words:
        counts[word] = counts.get(word, 0) + 1

    for word in &counts:
        let n = counts[word]
        println("{word}: {n}")
```
