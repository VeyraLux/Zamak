# Standard library tour

The standard library covers the everyday needs of command-line programs: input and output, files, arguments, math, time, and random numbers. Import a module with `use std/<name>`, and call its functions as `name.function(...)`.

## Built-in functions

These need no import:

| Function | What it does |
| --- | --- |
| `println(x)`, `print(x)` | Print a value and a newline. |
| `len(x)` | Characters in a string, or items in a list or map. |
| `str(x)` | Convert a value to its text, as `"{x}"` does. |
| `int(x)`, `float(x)` | Convert numbers, or parse text (can fail). |
| `abs(x)`, `min(a, b)`, `max(a, b)` | Absolute value, smaller, larger. |
| `min(xs)`, `max(xs)`, `sum(xs)`, `sorted(xs)` | Summarize or sort a list. |
| `copy(&x)` | Deep copy. |

## Read input: std/io

`io.read_line` reads one line at a time and returns `false` at the end of the input. This program numbers the lines it reads:

```zamak,check
use std/io

fn main() -> !string:
    let mut line = ""
    let mut n = 0
    while io.read_line(&mut line)?:
        n += 1
        println(f"{n}: {line}")
```

```sh
printf "alpha\nbeta\n" | zam run number.zm
```

```text
1: alpha
2: beta
```

- `io.read_all()` reads all of standard input into one string.
- `io.print(text)` prints without a newline, and `io.eprintln(text)` prints a line to standard error.

## Arguments and environment: std/env

```zamak,check
use std/env

fn main() -> !string:
    let args = env.args()
    if len(args) == 0:
        usage()
        env.exit(2)
    let name = env.arg(0)?
    let home = env.var("HOME") catch e:
        println(f"hello, {name}")
        return
    println(f"hello, {name}; home is {home}")

fn usage():
    println("usage: greet NAME")
```

- `env.args()` returns the arguments, without the program name. `env.arg(i)` returns one argument and fails if it's missing.
- `env.var(name)` reads an environment variable and fails if it isn't set.
- `env.exit(code)` ends the program with an exit status from 0 to 255.

## Files and paths: std/fs and std/path

```zamak
use std/fs
use std/path

fn main() -> !string:
    fs.mkdir("notes")?
    let file = path.join("notes", "todo.txt")
    fs.write(file, "buy milk\n")?
    fs.append(file, "write tests\n")?
    println(fs.read(file)?.lines())
    println(fs.exists(file))
    println(fs.list("notes")?)
    println(f"{path.name(file)} {path.stem(file)} {path.suffix(file)}")
    fs.remove(file)?
    fs.remove("notes")?
    println(fs.exists("notes"))
```

```text
["buy milk", "write tests"]
true
["todo.txt"]
todo.txt todo .txt
false
```

| Function | What it does |
| --- | --- |
| `fs.read(p)`, `fs.write(p, text)`, `fs.append(p, text)` | Read or write a whole file. |
| `fs.exists(p)`, `fs.is_file(p)`, `fs.is_dir(p)`, `fs.size(p)` | Inspect a path. |
| `fs.list(dir)` | Sorted names in a folder. |
| `fs.mkdir(p)` | Create a folder and its parents. |
| `fs.remove(p)`, `fs.rename(a, b)` | Remove a file or empty folder; rename. |
| `path.join`, `path.parent`, `path.name`, `path.stem`, `path.suffix` | Build and split paths as text. |

Every function that touches the file system can fail, so it returns an error you handle with `?` or `catch`.

## Math and formatting: std/math and std/fmt

`std/math` has `sqrt`, `pow`, `exp`, `log`, `log10`, the trigonometric functions, `floor`, `ceil`, `round`, `trunc`, and the constants `math.pi`, `math.e`, and `math.inf`. `std/fmt` has `fmt.fixed(x, digits)`:

```zamak
use std/math
use std/fmt

fn main():
    let r = 2.5
    println(fmt.fixed(math.pi * r * r, 2))
    println(math.floor(-2.5))
    println(math.ceil(-2.5))
```

```text
19.63
-3.0
-2.0
```

## Time and random numbers: std/time and std/random

```zamak
use std/time
use std/random

fn main():
    let start = time.monotonic()
    random.seed(7)
    let first = random.randint(1, 6)
    random.seed(7)
    let again = random.randint(1, 6)
    println(first == again)
    println(first >= 1 and first <= 6)
    println(time.iso_utc(0))
    println(time.monotonic() >= start)
```

```text
true
true
1970-01-01T00:00:00Z
true
```

- `time.time()` is the current time in seconds since 1970, and `time.monotonic()` measures durations. `time.sleep(seconds)` pauses. `time.iso_utc(t)` and `time.iso_local(t)` format a time.
- `random.randint(a, b)` includes both ends, and `random.randrange(a, b)` excludes `b`. `random.random()` returns an `f64` in [0, 1), and `random.uniform(a, b)` one in [a, b). `random.seed(n)` makes the sequence repeatable, which helps in tests.

## Strings: std/string

Every string function is also a method, so you rarely need to import `std/string`. [Strings](strings.md) covers the common ones. The full list includes:

- Measure and compare: `len`, `byte_len`, `starts_with`, `ends_with`, `contains`, `count`, `find`, `find_from`.
- Transform: `trim`, `lower`, `upper`, `replace`, `repeat`, `pad_left`, `pad_right`, `slice`.
- Split and join: `split`, `words`, `lines`, `join`, `chars`.
- Convert: `parse_i64`, `parse_f64`, `codepoints`, `from_code`, `bytes`, `from_bytes`.

## Summary

- Built-in functions such as `len`, `int`, and `sorted` need no import.
- `std/io`, `std/env`, `std/fs`, and `std/path` handle input, arguments, and files.
- `std/math`, `std/fmt`, `std/time`, and `std/random` handle numbers, formatting, clocks, and randomness.
- Functions that can fail return errors that you handle with `?` or `catch`.
