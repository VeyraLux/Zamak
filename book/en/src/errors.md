# Handle errors

Some operations can fail for reasons outside your program's control: a file is missing, or a user types letters where a number belongs. Zamak makes these failures part of a function's type, so you can't forget to handle them, and it gives you short syntax to pass them on or deal with them.

Zamak has no exceptions. An error is an ordinary value that a function returns.

## Functions that can fail

A function that can fail writes `!` and an error type after its result type. `fail` returns an error:

```zamak
fn parse_age(text: &string) -> i64!string:
    let n = int(text) catch e:
        fail f"not a number: {text}"
    if n < 0 or n > 150:
        fail f"out of range: {n}"
    n

fn main():
    for input in ["42", "abc", "200"]:
        let age = parse_age(input) catch e:
            println(f"error: {e}")
            continue
        println(f"age {age}")
```

```text
age 42
error: not a number: abc
error: out of range: 200
```

- `-> i64!string` means "returns an `i64`, or fails with a `string` message". A function that returns nothing but can fail writes `-> !string`.
- `fail value` returns the error to the caller.
- `call() catch e:` handles an error where it happens. Inside the block, `e` is the error. When the call's value is used, as in `let age = ...`, the block must leave with `return`, `fail`, `break`, or `continue`, because there's no value to continue with.

## Pass errors on with ?

Usually, a function can't fix an error itself and should let its caller decide. `?` after a call does that: on success it gives the value, and on failure it returns the error from the current function:

```zamak
fn parse_pair(text: &string) -> (i64, i64)!string:
    let parts = text.split(",")
    if len(parts) != 2:
        fail f"expected two numbers: {text}"
    let a = int(parts[0].strip())?
    let b = int(parts[1].strip())?
    (a, b)

fn main():
    for input in ["3, 4", "5", "6, x"]:
        let pair = parse_pair(input) catch e:
            println(f"skipped: {e}")
            continue
        println(pair)
```

```text
(3, 4)
skipped: expected two numbers: 5
skipped: invalid integer: x
```

`?` only works inside a function that can fail with the same error type. The compiler checks this, so an error can never be silently dropped.

## Handle an error and carry on

When the failing call has no value you need, the program continues after the `catch` block:

```zamak
use std/fs

fn main():
    fs.remove("does-not-exist.txt") catch e:
        println("nothing to remove")
    println("done")
```

```text
nothing to remove
done
```

## Let main fail

`main` can fail too. If it returns an error, Zamak prints the message to standard error and exits with status 1:

```zamak,check
use std/io

fn main() -> !string:
    let text = io.read_all()?
    let n = int(text.strip())?
    println(n * 2)
```

This is the simplest way to write a small tool: use `?` everywhere, and let `main` report anything that goes wrong.

## Errors with structure

A `string` message is enough for many programs. When callers need to tell errors apart, use an enum as the error type and `match` on it:

```zamak
enum ConfigError:
    Missing(key: string)
    Invalid(key: string, value: string)

fn port(settings: &{string: string}) -> i64!ConfigError:
    if "port" not in settings:
        fail ConfigError.Missing("port")
    let text = &settings["port"]
    let n = int(text) catch e:
        fail ConfigError.Invalid("port", text)
    n

fn main():
    let good = {"port": "8080"}
    let bad = {"port": "eighty"}
    let empty: {string: string} = {}
    for settings in [good, bad, empty]:
        let p = port(settings) catch e:
            match e:
                Missing(key): println(f"missing {key}")
                Invalid(key, value): println(f"bad {key}: {value}")
            continue
        println(f"port {p}")
```

```text
port 8080
bad port: eighty
missing port
```

## Assertions

`assert` checks a condition that should always hold. If it doesn't, the program stops and shows the condition:

```zamak
fn average(xs: &[f64]) -> f64:
    assert len(xs) > 0, "average of an empty list"
    sum(xs) / len(xs)

fn main():
    println(average([1, 2, 6]))
```

```text
3.0
```

Use errors for things that can go wrong at run time, such as bad input, and assertions for things that would mean a bug in your program.

## Summary

- `-> T!E` returns a `T` or fails with an `E`; `fail value` returns the error.
- `?` passes an error to the caller; `catch e:` handles it in place.
- `main` can be `-> !string`; an error then prints and exits with status 1.
- The error type can be a string, a struct, or an enum.
- `assert cond, "message"` stops the program when an assumption is broken.
