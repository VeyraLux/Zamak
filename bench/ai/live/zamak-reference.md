# Zamak language reference

Zamak is a statically typed language compiled to native code. Blocks are defined by indentation, like Python. Statements are separated by newlines; there are no semicolons. Comments start with `#`; there are no block comments (`//` is floor division, as in Python).

## Program shape

```
use std/io
use std/string

fn main() -> !string:
    let text = io.read_all()?
    println(len(text))
```

`main` takes no parameters. It returns nothing (`fn main():`) or may fail (`fn main() -> !string:`). If `main` fails, the error message is written to standard error and the program exits with status 1. To print a line to standard error, use `io.eprintln(&text)`.

## Blocks and indentation

- A line that opens a block ends with `:` — `fn`, `struct`, `enum`, `if`, `elif`, `else`, `while`, `for`, `match`, each `match` arm, and `catch e`.
- After the `:`, either write one statement on the same line (`if n < 0: return 0`) or put the block on the following lines, indented 4 spaces deeper than the opening line. Every line of a block has the same indentation; the block ends at the first line that is indented less.
- Use spaces, not tabs. An empty block is written `pass`.
- Inside `( )`, `[ ]` and `{ }` newlines and indentation are ignored, so long calls, lists and maps can span lines.
- Braces are only used for map literals, map types and string interpolation, never for blocks.

## Values and bindings

- Types: `i64`, `f64`, `bool`, `string` (UTF-8, owned), `bytes` (raw bytes), structs, enums, lists `[T]`, maps `{K: V}`, tuples `(i64, string)`, `Option[T]`, and function types such as `fn(i64) -> bool`.
- `let x = 1`, `let mut x = 1`, `let mut x: i64 = 1`. Only `let mut` bindings can be reassigned. A name cannot be declared twice in the same function scope chain (no shadowing).
- Top-level constants: `const LIMIT = 100`, `const RATE: f64 = 0.25`, `const NAME = "x"`. Names are UPPER_CASE; the value is computed at compile time from literals, earlier constants and operators. `pub const` is visible from other modules as `config.LIMIT`.
- Integer literals may use `_` between digits: `1_000_000`. Character literals like `'a'`, `'#'`, `'\n'`, `'\''` are `i64` Unicode code points (useful with `string.byte_at`).
- Operators: `+ - * / // % **`, `== != < <= > >=`, `in`, `not in`, unary `-`, parentheses. Division follows Python: `/` always gives `f64` (`7 / 2` is `3.5`), `//` is floor division (`-7 // 2` is `-4`), `%` takes the sign of the divisor (`-7 % 2` is `1`), `**` is a power (`2 ** 10` is `1024`, right-associative). `x in xs` / `x not in xs` test membership in a list, map (keys) or string (substring). `True`/`False` are accepted for `true`/`false`, and `f"..."` is the same as `"..."`. Logical operators are the words `and`, `or`, `not`, as in Python: `if a > 0 and not done:`. They short-circuit, and their precedence from lowest to highest is `or`, `and`, `not`, comparisons. `&&`, `||` and prefix `!` are errors; `!` only appears in error types such as `i64!string`. Bitwise operators on `i64`: `& | ^ ~ << >>` with Python's precedence (`x & 1 == 1` is `(x & 1) == 1`); shift counts must be 0 to 63. Integer literals may be written `0xFF`, `0b1010`, `0o17`. Compound assignment `+= -= *= /= //= %= &= |= ^= <<= >>=` works on variables, fields and elements. On strings, `+` concatenates (`a + b` makes a new string and moves neither side) and `== != < <= > >=` compare contents. Strings and integers cannot be mixed: `"n=" + 1` is an error; use interpolation.
- Integer overflow, division by zero, and out-of-range indexes stop the program with an error.
- `@wrapping` on the line before a top-level `fn` turns off overflow checks in that function only, for hot loops such as hashes and random number generators: integer `+ - *` and unary `-` wrap around (two's complement), and `f64` `/ // %` by zero give `inf` or `NaN` instead of stopping. Integer `//` and `%` by zero, and indexes, are still checked. Use it only where wrapping is intended or values are known to be small.
- Structs, enums, lists and maps support `==` and `!=` (field by field; maps ignore order) and can be printed with `println(x)` or `"{x}"`, which shows `Point(x: 1, y: 2)`, `Shape.Circle(1.5)`, `[1, 2]`, `{"a": 1}` with strings quoted. `copy(&x)` makes a deep copy of a string, list, map, struct or enum. Comparing, printing and copying never move the value.

## Floats

- `f64` literals need a digit on both sides of the point or an exponent: `1.5`, `0.25`, `2e-3`, `1_000.5`.
- `f64` follows IEEE 754 and never overflows; dividing by zero (`/`, `//`, `%`) stops the program, as in Python. Use `math.inf` for infinity.
- Mixing `i64` and `f64` in arithmetic or comparisons converts the integer to `f64`, as in Python: `x * 2`, `1 + 0.5`, `n < 2.5`. An integer is also accepted wherever an `f64` is expected: `let x: f64 = 2`, `area(2)`, `Circle(r: 1)`, `xs.push(3)` on `[f64]`. Convert with `f64(n)`/`float(n)` and `i64(x)`/`int(x)` (truncates toward zero; stops the program on `NaN` or out-of-range values). Round first with `math.round(x)` if needed.
- `println(x)` and `"{x}"` print the shortest text that reads back as the same value: `0.1`, `2.0`, `1e16`, `2.5e-7`. For a fixed number of decimals use `fmt.fixed(x, 2) -> string` (`use std/fmt`).
- `[f64]` lists and `{K: f64}` map values work like `i64` ones. Map keys cannot be `f64`.

## Strings

- Escapes: `\n \r \t \" \\`.
- Interpolation: `"{name}"` inserts a variable or a field path such as `"{p.age}"`. Any expression without string literals or braces may appear inside: `"{a + b}"`, `"{xs[i]}"`, `"{string.upper(&s)}"`. `{{` and `}}` produce literal braces.
- Build strings with `+`, interpolation or `string.join`.
- Method form needs no import: any `std/string` function whose first parameter is `&string` can be called on a string variable, as in `s.upper()`, `line.trim().len()`, `csv.split(",")`, `s.contains("x")`. `s.len()` and `len(s)` count characters, as in Python (`len("你好")` is 2); `s.byte_len()` counts UTF-8 bytes.
- Indexes and slices count characters, as in Python: `s[0]` is the first character (a string), `s[-1]` the last, and `s[a:b]`, `s[:b]`, `s[a:]` copy a part (negative counts from the end; out-of-range bounds are clamped).
- Methods also work on literals and Python names work: `"a,b".split(",")`, `", ".join(words)`, `s.strip()`, `s.lstrip()`, `s.rstrip()`, `s.startswith(p)`, `s.endswith(p)`, `s.isdigit()`, `s.isalpha()`, `s.isalnum()`, `s.isspace()`, `s.ljust(n)`, `s.rjust(n)`.
- A borrowed string is copied automatically where an owned string is needed: `return ch`, `out.push(w)` inside `for w in words`.
- `for ch in s:` loops over the characters (each `ch` is a one-character string); `for i, ch in s:` also gives the position.

## String literals

Escapes: `\n \r \t \0 \" \\`, `\xNN`, `\uXXXX`, `\UXXXXXXXX`. `r"C:\dir\{x}"` is a raw string (no escapes, no interpolation). `"""say "hi" {n}"""` may contain double quotes; `r"""{"a": 1}"""` is both (handy for JSON). Triple-quoted strings must fit on one line for now. Ordinary strings interpolate `{...}`, so a literal brace is `{{` or `}}`.

## More standard library

- `std/process` (like `subprocess`, no shell; first item is the program): `process.run(args, input="", capture=true) -> Completed!string` with fields `code`, `stdout`, `stderr`; `process.check_output(args, input="") -> string!string` (error when the exit code is not 0); `process.call(args) -> i64!string` (inherits the terminal).
- `std/json`: `json.loads(&text) -> Json!string`, `json.dumps(&value, indent=0) -> string`. `Json` is an enum `Null | Bool(bool) | Number(f64) | Str(string) | Array([Json]) | Object({string: Json})` (write `json.Json.Number(1.0)`). Accessors return copies and can fail, so chain with `?`: `doc.get("user")?.get("name")?.as_str()?`; also `item(i)`, `as_f64()`, `as_i64()`, `as_bool()`, `is_null()`, `len()`, `keys()`.
- `std/re` (Python names, positions in characters, linear-time engine, no backreferences or lookaround): `re.search/match/fullmatch(pattern, text, flags=0) -> Option[Match]!string`, `re.findall(...) -> [string]!string` (whole matches, or group 1 when the pattern has groups), `re.finditer(...) -> [Match]!string`, `re.sub(pattern, repl, text, count=0, flags=0)` (`\1`, `\g<name>` in repl), `re.split(pattern, text, maxsplit=0, flags=0)`, `re.escape(text)`, `re.compile(pattern, flags=0) -> Pattern!string` with the same methods. Flags `re.IGNORECASE`, `re.MULTILINE`, `re.DOTALL` or `(?i)`. `Match`: `group(i=0)`, `named(name)?`, `groups()`, `start(i=0)`, `end(i=0)`. Use `match re.search(...)?: Some(m): ... None: ...`.
- `std/hashlib`: `sha256`, `sha1`, `md5(&bytes) -> bytes`, `hmac_sha256(&key, &data) -> bytes`; call `.hex()` for text. `std/base64`: `b64encode(&bytes) -> string`, `b64decode(&text) -> bytes!string`, `urlsafe_b64encode`, `urlsafe_b64decode`.

## Bytes

- `bytes` holds raw bytes, as in Python: `b"GIF\x89"` (ASCII characters and `\xNN` only), `s.encode()` (UTF-8), `b.decode() -> string!string`, `b.hex() -> string`. Indexing gives an `i64` from 0 to 255; `len`, slices, `push`, `extend`, `in` and `for b in data` work as on lists; writing a value outside 0 to 255 stops the program. `for x in &mut data` is not supported; assign `data[i] = v` instead. Parameters take `&bytes` or `&mut bytes`.
- `std/bytes`: `bytes.from_hex(&text) -> bytes!string` (pairs of hex digits, spaces allowed between pairs), `bytes.zeros(n) -> bytes`.

## Calling C

- `extern fn name(x: i64, s: &string) -> f64` declares a C function; there is no body. Parameters are `i64`, `f64`, `bool`, `&string`, `&bytes`/`&mut bytes` (C gets a pointer; pass the length separately) or a C handle; results are nothing, `i64`, `f64`, `bool`, `string` or `Option[Handle]`.
- `extern type Db: "sqlite3", close = sqlite3_close` declares a C handle. It has one owner, moves like a string, can be borrowed as `&Db`, and is closed exactly once when it goes out of scope; it cannot be copied or constructed in Zamak. A C function returning a handle is declared `-> Option[Db]` (NULL is `None`); an output parameter `sqlite3 **` is `&mut Option[Db]`. Use the handle through `match`: `match open(path): Some(db): ...`.
- `zam.toml` `[native]`: `sources` (C files), `include` (folders), `headers` (such as `["sqlite3.h"]`: Zamak then calls through the library's own prototypes), `libraries`, `library_paths`.

## Ownership and borrowing

- Assigning or passing a `string`, struct, enum, list or map moves it; using it afterwards is an error. `i64`, `f64` and `bool` are copied.
- Parameters typed `&string`, `&[T]`, `&{K: V}`, `&Point` borrow read-only, and a variable, field or element is borrowed automatically: `show(name)`, `show(p.name)`, `show(xs[i])` (writing `&name` also works). Parameters typed `&mut string`, `&mut [T]`, `&mut {K: V}`, `&mut Point` borrow mutably and need an explicit `&mut x` at the call site (the owner must be `let mut`). Borrows cannot be stored in struct fields or returned. A read-only borrow can be kept in a local without a type annotation: `let entry = &entries[i]`, `let name = &p.name`; until the end of that block the borrowed variable cannot be changed or moved. Numbers and bools are copied instead: `let n = xs[i]`.
- A `&string` parameter also accepts a string literal or interpolation directly: `string.find(&s, ",")`.
- Inside a function, a parameter that is already a borrow is passed on without `&`: `fn f(s: &string): g(s)`.

## Functions

```
fn add(a: i64, b: i64) -> i64:
    a + b

fn double(n: i64) -> i64: n * 2
```

The last expression of the function body (or of the final `if`/`elif`/`else` branches) is the return value; `return expr` also works. Function names start with a lowercase letter. Only `pub fn` is visible from other modules (single-file programs do not need `pub`).

Parameters may have defaults and calls may name arguments, as in Python: `fn connect(host: &string, port: i64 = 80, timeout: f64 = 5.0)`, called as `connect("a")`, `connect("a", 8080)` or `connect("a", timeout=1.5)`. A default is a constant (literal, `const`, or an operation on them) or `Option.None`; every parameter after a defaulted one needs a default; positional arguments come before `name=value`. Methods take keyword arguments too (`p.moved(dy=5)`), and structs can be built as `Point(x=1, y=2)`. There are no `*args` or `**kwargs`.

## Generic functions

```
fn largest[T](items: &[T]) -> T:
    let mut best = items[0]
    for item in items:
        if item > best: best = item
    best
```

Type parameters are uppercase names in brackets after the function name. Calls do not name the types (`largest(numbers)`); each type parameter must appear in a parameter type so it can be inferred. The body is checked for each concrete type used.

## Generic types and Option

```
struct Stack[T]:
    items: [T]

    fn push(&mut self, item: T):
        self.items.push(item)

fn find(names: &[string], wanted: &string) -> Option[i64]:
    for i, name in names:
        if name == wanted: return Option.Some(i)
    Option.None

fn main():
    let mut s: Stack[string] = Stack(items: [])
    s.push("a")
    match find(s.items, "a"):
        Some(i): println("at {i}")
        None: println("missing")
```

`struct Name[T]` and `enum Name[T]` take type parameters; write the type as `Stack[string]` or `Option[i64]`. A constructor infers the type arguments from its values; when it cannot (`Option.None`, `Stack(items: [])`), annotate the variable or rely on the parameter or return type. `Option[T]` is built in, with variants `Some(value)` and `None`.

## Function values

```
fn double(n: i64) -> i64: n * 2

fn count_if(xs: &[string], keep: fn(&string) -> bool) -> i64:
    let mut n = 0
    for x in xs:
        if keep(x): n += 1
    n

fn main():
    let words: [string] = ["apple", "kiwi"]
    let min_len = 5
    println(count_if(words, fn(w): len(w) >= min_len))
    let f = double
    println(f(21))
```

A function type is `fn(T1, T2) -> R` (no `-> R` means no result). Pass a named function by name (`double`, `util.double`) or write an anonymous fn `fn(x: i64) -> i64: x * 2`; its body is one expression (or one `println(...)`) on the same line, and parameter types may be omitted when the expected function type is known. For longer code, declare `fn name(x: i64) -> i64:` with an indented body inside the function. Anonymous and nested fns can read the enclosing function's variables and change those declared with `let mut`; such a closure can be called, bound with `let`, and passed as an argument, but not stored in a struct or list or returned. A function-typed parameter can only be called or passed on. Functions without captures can be stored in variables, struct fields, lists (`[fn(i64) -> i64]`) and maps, and returned. A generic function can be passed where the function type is known: `apply(identity, 7)`. Call a list element directly: `steps[0](x)`. A function type can fail: `fn(&string) -> i64!string` or `fn(&string) -> !string`; calling through it needs `?` or `catch`, and a one-line fn may use `?` or `fail` when its expected type has `!E`. A struct field of function type is called like a method: `button.on_click(1)`.

## Assertions and tests

`assert cond` or `assert cond, "message"` stops the program with exit status 1 when `cond` is false. `zam test` runs every function named `test_...` (no parameters, no result, may be `-> !string`) in its own process and reports each one.

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
- To choose a value, write `let v = a if c else b` (as in Python) or `let v = if c: a elif d: b else: e`; it needs `else`, and each branch is a single expression.
- `while cond:`, `break`, `continue`.
- `for i in range(n):`, `for i in range(a, b):` and `for i in a..b:` iterate integers up to but not including the end (no step; use `while` for one). `for x in xs:` is the same as `for x in &xs:` (it borrows, it does not move).
- `for x in &list:` iterates list elements and `for i, x in &list:` also gives the index; `for k in &map:` iterates map keys in insertion order and `for k, v in &map:` gives keys and values. String elements/keys are read-only borrows.
- `for x in &mut list:` and `for k, v in &mut map:` change elements in place: assigning `x = x * 2` writes the element, struct elements allow `x.field += 1`. Elements cannot be added or removed inside these loops.

## Errors

- A function that can fail declares `-> T!string` (returns T or an error message) or `-> !string` (only fails).
- `fail "message"` returns an error. Interpolation works: `fail "bad line {n}"`.
- `call()?` returns the error to the caller (the current function must also be declared `!string`).
- `x == Option.None` works: a constructor on one side of `==` takes the type of the other side.
- `call() catch e: value` uses `value` when the call fails, as in `let port = int(text) catch e: 8080` or `let home = env.var("HOME") catch e: "unknown"`; the value must be a single expression on the same line.
- `call() catch e:` followed by an indented block handles the error in place; `e` is the error value. When the call's value is used (`let v = f() catch e:`), the block must end every path with `return` or `fail` (or `break`/`continue` inside a loop). When the call has no value or stands alone as a statement, the program continues after the block.
- The error type can also be a struct or an enum: `fn parse(t: &string) -> i64!ParseError`, `fail ParseError.Empty`, and `match e:` inside `catch`. `?` only passes an error to a function with the same error type; convert others in a `catch` block.

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

## Interfaces

```
interface Shape:
    fn area(&self) -> f64
    fn name(&self) -> string
```

Any struct or enum with these methods (same parameters and results) satisfies `Shape`; there is no `impl` declaration. Use an interface as a type for values of different types: `let shapes: [Shape] = [Circle(r=1.0), Square(side=2.0)]`, `fn biggest(shapes: &[Shape])`, struct fields and return types; calls dispatch to each value's own method, and `&mut self` methods work through it. Or use it as a bound: `fn total[T: Shape](items: &[T]) -> f64` (one copy per type; a missing method is a compile error). Interface methods take `&self` or `&mut self` and have no body. Interface values cannot be copied (borrow them) or built directly (`Shape()` is an error).

## Tuples

- Type `(i64, string)`, value `(3, "three")`, items `t.0`, `t.1`. A function returns several values as a tuple: `fn divmod(a: i64, b: i64) -> (i64, i64): (a // b, a % b)`.
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
- `xs.len()` (or `len(xs)`), `xs.push(v)`, `xs.pop()` (removes and returns the last element), `xs.insert(i, v)` (clamps `i` like Python), `xs.remove(i)` (removes and returns element `i`), `xs.swap(i, j)`, `xs.reverse()`, `xs.clear()`, `xs.extend(ys)` (moves all of `ys` to the end).
- For `i64`, `f64`, `bool` and `string` elements: `xs.sort()` (ascending, stable, in place), `xs.sorted()` or `sorted(xs)` (a sorted copy), `xs.contains(v)` or `v in xs`, `xs.index_of(v)` (-1 when absent), `xs.min()`/`min(xs)` and `xs.max()`/`max(xs)` (stop the program on an empty list). For `i64` and `f64` elements: `xs.sum()` or `sum(xs)`.
- Indexes and slices follow Python: `xs[-1]` is the last element; `xs[a:b]`, `xs[:b]`, `xs[a:]` (and `xs.slice(a, b)`) copy a part, with negative bounds counted from the end and out-of-range bounds clamped.
- Any list: `xs.sort_by(fn(a, b): a.age < b.age)` sorts in place (stable); the fn returns true when `a` goes before `b`.
- `xs[i]` reads an `i64`/`f64`/`bool` element and `xs[i] = v` writes. A string element cannot be taken out with `xs[i]`; pass `&xs[i]` to a `&string` parameter or copy it with `string.copy(&xs[i])`.
- Struct elements: read and write fields with `xs[i].field` (also `+=`), pass `&xs[i]` / `&mut xs[i]` to `&Point` / `&mut Point` parameters, pass `&xs[i].name` to a `&string` parameter, call `xs[i].tags.push(v)`. `for p in &xs` gives a read-only `&Point`. Structs move on assignment: `let p = xs[i]` is an error; write `copy(&xs[i])` for an independent copy, `xs.pop()` to take the last one out, `xs.swap(i, j)` to reorder, and `xs[i] = Point(...)` to replace. `xs[i].tags[j]` is not supported.
- Methods can be called on variables, fields and call results: `parts.len()`, `string.split(&s, ",").len()`, `xs.slice(0, 2).len()`.

## Sets `{K}`

`{1, 2, 3}` is a set; its type is `{i64}` or `{string}` (an empty set needs an annotation: `let s: {i64} = {}`). `s.add(x)`, `s.remove(x)` / `s.discard(x)` (return whether it was there), `x in s`, `len(s)`, `for x in s`, `s.clear()`, `a.union(b)`, `a.intersection(b)`, `a.difference(b)` (new sets), `a.issubset(b)`, `sorted(s)` (a sorted list). Sets have no indexes and no order; they print as `{1, 2}` and `set()`.

## Maps `{K: V}`

`K` is `i64` or `string`; `V` is `i64`, `f64`, `bool`, `string`, a struct, an enum, a list or a map (maps of structs have no `get`; use `m.contains(k)` then `m[k].field`).

- `let mut m: {string: i64} = {}` (an empty map needs a type annotation), `let m = {"a": 1, "b": 2}`.
- `m[k]` reads (stops the program if the key is missing); `m[k] = v` inserts or replaces.
- String keys are passed as borrows: `m[&name]`, `m["literal"]`, or a borrowed loop variable directly `m[word]`.
- `m.len()`, `m.contains(k)`, `m.get(k, default)` (value or `default`), `m.remove(k)` (returns `bool`), `m.keys()` and `m.values()` (lists in insertion order; `values` is not available for struct values), `m.clear()`.
- Counting idiom: `counts[word] = counts.get(word, 0) + 1`.

## Standard library

Import a module with `use std/<name>` and call `name.function(...)`. `println(x)` and `print(x)` need no import and print one value followed by a newline (like Python's `print`); `io.print` prints without a newline. A call result can be passed to a read-only borrow parameter directly: `string.parse_i64(string.trim(&line))`. `for` can loop over a call result: `for name in fs.list(dir)?:`.

- `std/io`: `io.print(&text)` (no newline), `io.eprintln(&text)` (standard error), `io.read_line(&mut line) -> bool!string` reads the next line into `line` without its trailing `\n` or `\r\n` and returns `false` at end of input (typical loop: `let mut line = ""` then `while io.read_line(&mut line)?:`); `io.read_all() -> string!string`.
- `std/env`: `env.args() -> [string]` (program arguments, without the program name), `env.arg(i) -> string!string`, `env.arg_count() -> i64`, `env.var(&name) -> string!string` (an environment variable), `env.exit(code)` (ends the program with status 0 to 255; a `catch` block may end with it instead of `return`).
- Built-in functions need no import, as in Python: `len(x)` (string bytes, list or map size), `str(x)` (same as `"{x}"`), `int(x)` (truncates an `f64`; parses a string and can fail, so write `int(s)?` or catch it), `float(x)` (converts an `i64`; parses a string and can fail), `abs(x)`, `min(a, b)`, `max(a, b)` (both of the same type), `min(xs)`, `max(xs)`, `sum(xs)`, `sorted(xs)`.
- `std/fs`: `fs.read(&path) -> string!string`, `fs.write(&path, &text) -> !string`, `fs.append(&path, &text) -> !string`, `fs.exists(&path) -> bool`, `fs.is_file`, `fs.is_dir`, `fs.size(&path) -> i64!string`, `fs.list(&dir) -> [string]!string` (sorted names), `fs.mkdir(&path) -> !string` (creates parents), `fs.remove(&path) -> !string` (file or empty directory), `fs.rename(&from, &to) -> !string`, `fs.read_bytes(&path) -> bytes!string`, `fs.write_bytes(&path, &data) -> !string`, `fs.append_bytes(&path, &data) -> !string`.
- `std/path` (text only): `path.join(&a, &b) -> string`, `path.parent`, `path.name`, `path.stem`, `path.suffix` (`.txt`, or empty).
- `std/math` (all `f64`): `sqrt`, `pow(x, y)`, `exp`, `log` (natural), `log10`, `sin`, `cos`, `tan`, `atan`, `atan2(y, x)`, `floor`, `ceil`, `round` (half away from zero), `trunc`, `abs`, `min(a, b)`, `max(a, b)`, `is_nan(x) -> bool`; constants `math.pi`, `math.e`, `math.inf`.
- `std/fmt`: `fmt.fixed(x, digits) -> string` (0 to 30 digits).
- `std/time`: `time.time() -> f64` (seconds since 1970), `time.monotonic() -> f64` (for measuring durations), `time.sleep(seconds: f64)`, `time.iso_utc(t) -> string` (`1970-01-01T00:00:00Z`), `time.iso_local(t) -> string`.
- `std/random`: `random.randint(a, b) -> i64` (both ends included), `random.randrange(a, b) -> i64` (b excluded), `random.random() -> f64` in [0, 1), `random.uniform(a, b) -> f64`, `random.seed(n)`.
- `std/string` (positions count characters, as in Python; the `byte_*` functions use UTF-8 byte offsets):
  - `len(&s) -> i64` (characters), `byte_len(&s) -> i64`, `char_at(&s, i) -> string` (same as `s[i]`), `byte_at(&s, i) -> i64` (the byte value)
  - `equal(&a, &b) -> bool`, `starts_with(&s, &p) -> bool`, `ends_with(&s, &p) -> bool`, `contains(&s, &p) -> bool`
  - `find(&s, &needle) -> i64` and `find_from(&s, &needle, start) -> i64` (-1 when absent); `byte_find`, `byte_find_from` return byte offsets
  - `slice(&s, start, end) -> string` (same as `s[start:end]`), `byte_slice(&s, start, end) -> string` (byte offsets; stops the program inside a multi-byte character), `trim(&s) -> string`, `lower(&s) -> string`, `upper(&s) -> string`, `replace(&s, &from, &to) -> string`, `copy(&s) -> string`
  - `parse_i64(&s) -> i64!string` (decimal, optional sign, no surrounding spaces), `parse_f64(&s) -> f64!string` (decimal with optional fraction and exponent)
  - `split(&s, &sep) -> [string]` (keeps empty fields), `words(&s) -> [string]` (splits on whitespace), `lines(&s) -> [string]` (drops `\r`; no empty item after a final newline), `join(&list, &sep) -> string`
  - `chars(&s) -> [string]` (one string per character), `codepoints(&s) -> [i64]`, `from_code(n) -> string`, `bytes(&s) -> [i64]`, `from_bytes(&list) -> string!string`
  - `repeat(&s, n) -> string`, `count(&s, &sub) -> i64`, `pad_left(&s, width) -> string` and `pad_right(&s, width) -> string` (pad with spaces to `width` characters)

## Example

```
use std/io

fn main() -> !string:
    let text = io.read_all()?
    let mut counts: {string: i64} = {}

    for word in text.words():
        counts[word] = counts.get(word, 0) + 1

    for word, n in counts:
        print(f"{word}: {n}")
```
