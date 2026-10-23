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
    let home = env.var("HOME") catch e: "unknown"
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
| `fs.read_bytes(p)`, `fs.write_bytes(p, data)`, `fs.append_bytes(p, data)` | Read or write a file as bytes; see `bytes` below. |
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
- Convert: `parse_i64`, `parse_f64`, `codepoints`, `from_code`, `encode`, `bytes`, `from_bytes`.

## Bytes: std/bytes

The `bytes` methods `decode` and `hex` need no import. The functions that create bytes are in `std/bytes`:

```zamak
use std/bytes

fn main() -> !string:
    let key = bytes.from_hex("de ad be ef")?
    println(key)
    println(key.hex())
    println(bytes.zeros(4))
```

```text
b"\xde\xad\xbe\xef"
deadbeef
b"\x00\x00\x00\x00"
```

| Function | What it does |
| --- | --- |
| `b.decode()` | Decode UTF-8 into a string. Returns an error if the bytes aren't valid UTF-8. |
| `b.hex()` | Hexadecimal text, two lowercase digits per byte. |
| `bytes.from_hex(text)` | Parse hexadecimal text in pairs of digits, with optional spaces between pairs. |
| `bytes.zeros(n)` | `n` zero bytes. |
| `fs.read_bytes(p)`, `fs.write_bytes(p, data)`, `fs.append_bytes(p, data)` | Read and write files as bytes. |

## Run other programs: std/process

`std/process` follows Python's `subprocess`. The first item of the argument list is the program, found on `PATH`; the rest go to it as they are, with no shell:

```zamak,ignore
use std/process

fn main() -> !string:
    let result = process.run(["git", "status", "--short"])?
    println(result.code)
    println(result.stdout)
    let version = process.check_output(["git", "--version"])?
    println(version.trim())
    process.call(["git", "log", "-1"])?
```

| Function | What it does |
| --- | --- |
| `process.run(args, input="", capture=true)` | Runs the program and waits. Returns a `Completed` with `code`, `stdout`, and `stderr`. |
| `process.check_output(args, input="")` | Returns standard output. Returns an error with the standard error text when the exit code isn't 0. |
| `process.call(args)` | Uses the current terminal for input and output, and returns the exit code. |

A program that can't be found returns an error. On Windows, run `.bat` and `.cmd` scripts through `cmd /c`, as in Python.

## JSON: std/json

`json.loads` parses text into a `Json` value, and `json.dumps` writes a `Json` value as text. The accessors can fail, so you can chain them with `?`:

```zamak
use std/json

fn main() -> !string:
    let user = json.loads(r"""{"name": "Ada", "langs": ["Zamak", "C"]}""")?
    println(user.get("name")?.as_str()?)
    println(user.get("langs")?.item(0)?.as_str()?)
    println(json.dumps(user))
```

```text
Ada
Zamak
{"name": "Ada", "langs": ["Zamak", "C"]}
```

- `Json` is an enum with `Null`, `Bool`, `Number`, `Str`, `Array`, and `Object`. You can build values directly, such as `json.Json.Number(1.0)`.
- Accessors: `get(key)`, `item(index)`, `as_str()`, `as_f64()`, `as_i64()`, `as_bool()`, `is_null()`, `len()`, and `keys()`.
- Numbers are stored as `f64`, and whole numbers are written without a decimal point. `json.dumps(value, indent=2)` writes one item per line.

## Regular expressions: std/re

The functions in `std/re` have the same names and parameters as Python's `re`, and positions count characters. Write patterns as raw strings, `r"..."`, because they have many backslashes:

```zamak
use std/re

fn main() -> !string:
    let text = "order 12, order 345"
    println(re.findall(r"\d+", text)?)
    println(re.sub(r"order (\d+)", r"#\1", text)?)
    match re.search(r"(?P<id>\d{3})", text)?:
        Some(m): println(m.named("id")?)
        None: println("none")
```

```text
["12", "345"]
#12, #345
345
```

- Functions: `search`, `match`, `fullmatch`, `findall`, `finditer`, `sub`, `split`, `escape`, and `compile`. Flags: `re.IGNORECASE`, `re.MULTILINE`, and `re.DOTALL`, or `(?i)` at the start of the pattern.
- A `Match` has `group(i)`, `named(name)`, `groups()`, `start(i)`, and `end(i)`.
- Matching takes time linear in the length of the text, so a badly written pattern can't hang your program. In exchange, backreferences such as `\1` and lookaround such as `(?=...)` aren't supported and report an error.

## Hashes and base64: std/hashlib and std/base64

```zamak
use std/hashlib
use std/base64

fn main() -> !string:
    let data = "hello".encode()
    println(hashlib.sha256(data).hex())
    println(base64.b64encode(data))
    println(base64.b64decode("aGVsbG8=")?.decode()?)
```

```text
2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
aGVsbG8=
hello
```

`std/hashlib` has `sha256`, `sha1`, `md5`, and `hmac_sha256(key, data)`, which return `bytes`. `std/base64` has `b64encode`, `b64decode`, and the URL-safe `urlsafe_b64encode` and `urlsafe_b64decode`.

## Summary

- Built-in functions such as `len`, `int`, and `sorted` need no import.
- `std/io`, `std/env`, `std/fs`, and `std/path` handle input, arguments, and files; `std/bytes` handles raw bytes.
- `std/math`, `std/fmt`, `std/time`, and `std/random` handle numbers, formatting, clocks, and randomness.
- `std/process`, `std/json`, `std/re`, `std/hashlib`, and `std/base64` run programs, read and write JSON, match text, and compute hashes.
- Functions that can fail return errors that you handle with `?` or `catch`.
