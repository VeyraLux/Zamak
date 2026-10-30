# Standard library tour

The standard library covers the everyday needs of command-line programs: input and output, files, arguments, math, time, and random numbers. Import a module with `use std/<name>`, and call its functions as `name.function(...)`.

## Built-in functions

These need no import:

| Function | What it does |
| --- | --- |
| `println(x)`, `print(x)` | Print a value and a newline. |
| `input()`, `input(prompt)` | Show `prompt` without a newline, then read one line of input without its line ending. |
| `len(x)` | Characters in a string, or items in a list or map. |
| `str(x)` | Convert a value to its text, as `"{x}"` does. |
| `int(x)`, `float(x)` | Convert numbers, or parse text (can fail). |
| `abs(x)`, `min(a, b)`, `max(a, b)` | Absolute value, smaller, larger. |
| `min(xs)`, `max(xs)`, `sum(xs)`, `sorted(xs)` | Summarize or sort a list. |
| `copy(&x)` | Deep copy. |

## Read input

`input` works as in Python. It shows a prompt, waits for a line, and returns it without the line ending:

```zamak,check
fn main() -> !string:
    let name = input("Your name: ")
    let age = int(input("Your age: "))?
    println(f"Hi {name}, next year you'll be {age + 1}.")
```

If the input ends before `input` gets a line, the program stops with an error, as an uncaught `EOFError` does in Python. To read until the input ends, use `io.read_line` from `std/io`.

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
- `env.program()` is the program's file name, without the folder or `.exe`. To read options, see [std/cli](#command-line-options-stdcli).

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

## Dates and times: std/datetime

`std/datetime` follows Python's `datetime`:

```zamak
use std/datetime

fn main() -> !string:
    let due = datetime.date_from_iso("2026-11-01")?
    println(due.add_days(30).isoformat())
    println(due.weekday())
    println(due.strftime("%A, %d %B %Y"))
    let start = datetime.date(2026, 1, 1)?
    println(due.days_since(start))
    let t = datetime.datetime_from_iso("2026-11-01T08:30:00+08:00")?
    println(t.to_utc().isoformat())
```

```text
2026-12-01
6
Sunday, 01 November 2026
304
2026-11-01T00:30:00+00:00
```

- A `Date` has `year`, `month`, and `day`. `datetime.today()` is today's date, and `datetime.date(y, m, d)` checks the values.
- A `DateTime` also has `hour`, `minute`, `second`, `microsecond`, and `offset` (seconds east of UTC). `datetime.now()` uses the local time zone, and `datetime.utcnow()` uses UTC. `datetime.from_timestamp(t)` converts a `time.time()` value.
- `add_days(n)`, `days_since(other)`, `add_seconds(s)`, and `seconds_since(other)` do arithmetic. `weekday()` is 0 for Monday.
- `isoformat()`, `strftime(format)`, `date_from_iso(text)`, and `datetime_from_iso(text)` convert to and from text.
- Dates and times hold only numbers, so they're copied like numbers.

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

- `Json` is an enum with `Null`, `Bool`, `Int`, `Float`, `Str`, `Array`, and `Object`. You can build values directly, such as `json.Json.Int(1)`.
- Accessors: `get(key)`, `item(index)`, `as_str()`, `as_f64()`, `as_i64()`, `as_bool()`, `is_null()`, `len()`, and `keys()`. `as_f64()` also accepts an `Int`. `as_i64()` accepts only an `Int`, so `1.5` and `1.0` are errors.
- Numbers work as in Python. A number without a decimal point or an exponent is an `Int` (`i64`), and any other number is a `Float` (`f64`). Each is written back the way it was read, so `1` stays `1` and `1.0` stays `1.0`. An integer that doesn't fit in `i64` is an error, not a rounded float.
- `json.dumps(value, indent=2)` writes one item per line.

### Structs to and from JSON

`json.encode(value)` writes any struct, list, map, or other value as JSON text, and `json.decode(text)` reads it back. Write the type you want in the annotation:

```zamak
use std/json

struct Task:
    id: i64
    text: string
    tags: [string]
    due: Option[string]

fn main() -> !string:
    let tasks = [Task(id: 1, text: "write", tags: ["work"], due: Option.None)]
    let text = json.encode(tasks)
    println(text)
    let back: [Task] = json.decode(text)?
    println(back[0].text)
    let bad: [Task] = json.decode(r"""[{"id": 2, "text": "x", "tags": [7]}]""") catch e:
        println(e)
        return
```

```text
[{"id": 1, "text": "write", "tags": ["work"], "due": null}]
write
[0].tags[0]: the JSON value is not a string
```

- A struct is an object whose keys are the field names. A list is an array, a map with `string` or `i64` keys is an object, and `Option` is `null` or the value. A missing key reads as `None` for an `Option` field.
- An enum variant without data is a string, such as `"Empty"`. A variant with data is an object with a `"type"` key, such as `{"type": "Circle", "r": 1.5}`.
- When the text doesn't match the type, the error shows where: `[0].tags[0]`, `.shape.h: missing`.
- `json.to_json(value)` and `json.from_json(j)` convert to and from a `Json` value instead of text.
- Every string interpolates `{...}`, so write JSON literals as raw strings: `r"""{"id": 1}"""`.

## Databases: std/sqlite

`std/sqlite` is SQLite, built into `zam`, so you don't install anything. `sqlite.connect(path)` opens a database file and creates it when it doesn't exist, and `":memory:"` opens a database that lives only in memory. The connection closes when it goes out of scope.

```zamak
use std/sqlite

struct Note:
    id: i64
    title: string
    pinned: bool
    body: Option[string]

struct Draft:
    title: string
    pinned: bool

fn main() -> !string:
    let db = sqlite.connect(":memory:")?
    db.execute_script("CREATE TABLE notes (id INTEGER PRIMARY KEY, title TEXT NOT NULL, pinned BOOLEAN NOT NULL DEFAULT 0, body TEXT)")?
    db.execute("INSERT INTO notes (title, body) VALUES (?, ?)", [sqlite.text("milk"), sqlite.text("2 L")])?
    db.execute_named("INSERT INTO notes (title, pinned) VALUES (:title, :pinned)", &Draft(title: "rent", pinned: true))?
    let notes: [Note] = db.query("SELECT * FROM notes ORDER BY id", [])?
    for note in notes:
        println(f"{note.id} {note.title} {note.pinned} {note.body}")
```

```text
1 milk false Option.Some("2 L")
2 rent true Option.None
```

| Method | What it does |
| --- | --- |
| `db.execute(sql, params)` | Runs one statement with `?` parameters, and returns the number of rows it changed. |
| `db.execute_named(sql, &value)` | Runs one statement, and fills each `:name` parameter from the field of the same name in a struct or map. |
| `db.execute_script(sql)` | Runs several statements separated by semicolons, without parameters. |
| `let rows: [T] = db.query(sql, params)?` | Reads each result row as a `T`. Column names match field names. |
| `db.query_rows(sql, params)` | Returns each row as a list of `sqlite.Value` in column order. |
| `db.query_json(sql, params)` | Returns the rows as a `Json` array of objects. |
| `db.last_id()`, `db.changes()` | The row ID of the last `INSERT`, and the number of rows the last statement changed. |

Write parameters with `sqlite.integer(n)`, `sqlite.real(x)`, `sqlite.text(s)`, and `sqlite.null()`, which are named after SQLite's column types.

Values go in and come back exactly. Anything that doesn't fit is an error, never a silent change:

- An integer keeps every digit of its `i64`.
- A `bool` is stored as `0` or `1`, and a column declared `BOOLEAN` reads back as `true` or `false`.
- `NULL` reads as `Option.None`. For a field that isn't an `Option`, it's an error.
- A list or struct field can't go in one column. Store `json.encode(...)` in a string field, or use a second table.
- `execute` and the query methods run exactly one statement. To run several, use `execute_script`.
- While another connection is writing, a statement waits up to 5 seconds before it reports that the database is locked.

## Command-line options: std/cli

`std/cli` reads command-line options into a struct. Each field is an option, and its type decides how the option is read, so there's nothing else to declare:

```zamak
use std/cli

struct Options:
    all: bool
    due: Option[string]
    tag: [string]
    count: i64
    args: [string]

fn main() -> !string:
    let given: [string] = ["--all", "--tag", "work", "--tag=home", "--count", "3", "report.txt"]
    let o: Options = cli.parse(given)?
    println(f"{o.all} {o.due} {o.tag} {o.count} {o.args}")
    let wrong: [string] = ["--count", "three"]
    let bad: Options = cli.parse(wrong) catch e:
        println(e)
        return
```

```text
true Option.None ["work", "home"] 3 ["report.txt"]
--count needs a whole number, not three
```

| Field type | On the command line |
| --- | --- |
| `bool` | `--all` is a switch: present means `true`. |
| `i64`, `f64`, `string` | `--count 3` or `--count=3`. Required, and a value of the wrong type is an error. |
| `Option[...]` | Optional; a missing option is `None`. |
| `[...]` | Repeatable: `--tag a --tag b`. |
| `args: [...]` | Collects the positional arguments. Without this field, a positional argument is an error. Everything after `--` is positional. |

- An underscore in a field name is a dash on the command line: `dry_run` is `--dry-run`.
- An unknown option, a missing value, or an option given twice is an error that names the option.
- `-h` and `--help` print the options and exit.
- In a program, read the real arguments with `let o: Options = cli.parse_or_exit(env.args())`. It prints the error and a hint, and exits with status 2, as Python's `argparse` does.
- For subcommands, read the first argument yourself and parse the rest: `cli.parse(args[1:])`. `examples/apps/todo` does this.
- `env.program()` is the program's file name, without the folder or `.exe`.

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

## Networking: std/net and std/http

`std/http` sends HTTP requests in the shape of Python's `requests`:

```zamak,ignore
use std/http

fn main() -> !string:
    let r = http.get("https://example.com/")?
    println(r.status)
    println(r.headers["content-type"])
    println(r.text()?)
    let reply = http.post("http://localhost:8000/notes", "buy milk")?
    println(reply.ok())
```

| Function | What it does |
| --- | --- |
| `http.get(url, params={}, headers={}, timeout=30.0)` | Sends a GET request and returns a `Response`. `params` are encoded into the query string, as in `requests`. |
| `http.post(url, body, content_type="text/plain; charset=utf-8")`, `http.post_json(url, value)` | Posts text, or any value as JSON. Both take `params` and `headers` too. |
| `http.request(method, url, body, headers)` | Any method, with your own headers and a `bytes` body. |
| `http.urljoin(base, link)` | Resolves a link against a page's URL, like Python's `urllib.parse.urljoin`. |
| `Response` | `status`, `reason`, `headers` (lowercase keys), and `body`, plus `text()`, `json()`, and `ok()`. |

On the server side, `http.serve(host, port, handler)` handles requests until the program stops. Each request goes to `handler`, and the `Response` it returns goes back to the client. Several threads handle requests at the same time, so one slow request doesn't hold up the others:

```zamak,ignore
use std/http

fn hello(req: &http.Request) -> http.Response:
    http.respond(200, f"hello from {req.path}")

fn main() -> !string:
    http.serve("127.0.0.1", 8000, hello)?
```

Requests follow redirects (301, 302, 303, 307, and 308, up to 10 times), as `requests` does; pass `follow=false` to get the redirect itself.

`http.respond(status, text)` and `http.respond_json(status, value)` build responses; `value` can be a `Json` value, a struct, a list, or anything else that converts to JSON. A `Request` has `method`, `path`, `query`, `headers`, and `body`, plus `text()`, `json()`, `args()` for the decoded query parameters, and `arg(name, default="")` for one of them.

- `handler` runs on several threads at once, so it can't capture local variables; pass a named function. Pass `workers=n` to choose the number of threads; by default it's twice the number of processors, and at least 8.
- `http.quote(text)`, `http.unquote(text)`, and `http.parse_query(query)` work like Python's `urllib.parse.quote`, `unquote`, and `parse_qs`.
- [`examples/apps/notes`](https://github.com/VeyraLux/Zamak/tree/main/examples/apps/notes) is a complete REST service that stores notes in SQLite.

`std/net` is plain TCP underneath: `net.connect(host, port)` returns a connection, and `net.listen(host, port)` returns a listener. A connection has `send`, `recv`, `recv_all`, and `set_timeout`. Connections and listeners close when they go out of scope.

### HTTPS and TLS

URLs that start with `https://` use TLS and check the server's certificate. A trusted root must sign the certificate, and its name must match the host. Otherwise the request fails, and the error says why. Root certificates come from the system: the certificate store on Windows, and the usual certificate files on Linux and macOS. To use your own PEM file instead, set the `SSL_CERT_FILE` environment variable.

To serve HTTPS, pass a certificate and a private key, both PEM files, to `serve`:

```zamak,ignore
http.serve("0.0.0.0", 8443, hello, cert_file="cert.pem", key_file="key.pem")?
```

In `std/net`, `conn.start_tls(host)` starts TLS on an open connection as a client, and `listener.accept_tls(cert_file, key_file)` accepts a TLS connection. After that, use `send` and `recv` as usual.

TLS comes from [Mbed TLS](https://github.com/Mbed-TLS/mbedtls) (Apache-2.0), and `zam` carries its source. The first time a program uses `std/net` or `std/http`, `zam` compiles it into the cache, which usually takes 10 to 20 extra seconds. Later builds reuse it. When you distribute such a program, include the Mbed TLS license.

## Threads and queues: std/thread

`thread.spawn(work, args...)` runs `work` on a new thread, like Python's `executor.submit(work, *args)`. The arguments move to the new thread, so the starting thread can't use them afterward. `join()` waits for the thread to finish and returns its result:

```zamak,ignore
use std/thread

fn squares(start: i64, stop: i64) -> i64:
    let mut total = 0
    for i in range(start, stop):
        total += i * i
    total

fn main() -> !string:
    let mut tasks: [thread.Task[i64]] = []
    for k in range(4):
        tasks.push(thread.spawn(squares, k * 1000, (k + 1) * 1000))
    let mut total = 0
    for task in &tasks:
        total += task.join()?
    println(total)
```

When `work` returns a value, `spawn` returns a `Task[R]`. Otherwise, it returns a `Thread`. If `work` fails, `join()` fails with the same error. A thread that you don't join is waited for when its handle goes out of scope.

Threads don't share data that can change, so there are no data races. To pass data between threads, use a `Queue[T]`, which works like Python's `queue.Queue`:

| Code | What it does |
| --- | --- |
| `let q: thread.Queue[string] = thread.queue()` | Creates a queue. The annotation gives the item type. |
| `q.put(item)?` | Adds an item and moves it into the queue. Fails if the queue is closed. |
| `q.get()` | Removes the oldest item, and waits while the queue is empty. Returns `Option.None` once the queue is closed and empty. |
| `q.close()` | Closes the queue: no more items, and waiting `get` calls wake up. |
| `q.clone()` | Returns another handle to the same queue, which you can give to another thread. |

```zamak,ignore
fn produce(q: thread.Queue[string], count: i64) -> !string:
    for i in range(count):
        q.put(f"item {i}")?
    q.close()

fn main() -> !string:
    let q: thread.Queue[string] = thread.queue()
    let worker = thread.spawn(produce, q.clone(), 3)
    while true:
        match q.get():
            Some(item): println(item)
            None: break
    worker.join()?
```

Some rules:

- Pass the thread function by name. Its parameters can't be borrows (`&T`), because the new thread can outlive values in the starting thread.
- A thread function fails with a string error, such as `-> i64!string`.
- `thread.cpu_count()` returns the number of logical processors. Each thread has its own `random` state, so `random.seed` affects only the thread that calls it.
- `println` from several threads writes whole lines that don't mix.

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
- `std/sqlite` stores data in a SQLite database and reads rows back as structs.
- `std/thread` starts threads with `thread.spawn`, and threads pass data through a `Queue`.
- Functions that can fail return errors that you handle with `?` or `catch`.
