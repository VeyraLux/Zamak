# Write tests

Zamak has testing built in. A test is a function whose name starts with `test_`, and `zam test` finds and runs every one of them. You don't need a framework or a configuration file.

## Write a test

Put tests next to the code they check. Each test uses `assert` to state what should be true:

```zamak,test
fn slugify(title: &string) -> string:
    title.strip().lower().replace(" ", "-")

fn test_lowercases():
    assert slugify("Hello") == "hello"

fn test_replaces_spaces():
    assert slugify("Hello World") == "hello-world"

fn test_trims():
    assert slugify("  Hi  ") == "hi"

fn main():
    println(slugify("The Zamak Book"))
```

Run them with `zam test`, passing a file or a project folder:

```sh
zam test slugify.zm
```

```text
test test_lowercases ... ok
test test_replaces_spaces ... ok
test test_trims ... ok

test result: ok. 3 passed; 0 failed
```

A test function takes no parameters and returns nothing. It can also be declared `-> !string`, so you can use `?` inside it; a test that returns an error fails.

## When a test fails

When an assertion doesn't hold, the report shows the file, the line, the condition, and your message:

```zamak,test
fn slugify(title: &string) -> string:
    title.strip().lower().replace(" ", "-")

fn test_spaces():
    assert slugify("A B") == "a_b", "spaces become underscores"

fn main():
    println(slugify("A B"))
```

```text
test test_spaces ... FAILED

---- test_spaces ----
main.zm:5: assertion failed: slugify("A B") == "a_b": spaces become underscores
(exit status 1)

test result: FAILED. 0 passed; 1 failed
```

`zam test` exits with status 1 when any test fails, so you can use it in continuous integration as is.

Each test runs in its own process. A test that crashes, for example with an integer overflow or an out-of-bounds index, fails on its own and doesn't stop the other tests.

## Run some of the tests

Add a word after the file or folder to run only the tests whose names contain it:

```sh
zam test slugify.zm trims
```

```text
test test_trims ... ok

test result: ok. 1 passed; 0 failed
```

## Test the error path

To check that a function fails when it should, call it with `catch` and record what happened:

```zamak,test
fn parse_percent(text: &string) -> i64!string:
    let n = int(text)?
    if n < 0 or n > 100:
        fail f"out of range: {n}"
    n

fn test_accepts_valid_values() -> !string:
    assert parse_percent("42")? == 42

fn test_rejects_out_of_range():
    let mut message = ""
    parse_percent("150") catch e:
        message = e
    assert message == "out of range: 150"

fn main() -> !string:
    println(parse_percent("7")?)
```

```text
test test_accepts_valid_values ... ok
test test_rejects_out_of_range ... ok

test result: ok. 2 passed; 0 failed
```

## Summary

- A test is a function named `test_...` with no parameters. It can return `!string`.
- `assert cond, "message"` states what must be true.
- `zam test` runs each test in its own process and prints a report; add a word to run only matching tests.
