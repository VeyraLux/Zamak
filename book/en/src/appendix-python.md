# Appendix A: Zamak for Python programmers

Zamak keeps Python's names and rules wherever it can. This appendix lists what stays the same, what's written differently, and what's new.

## The same as in Python

- Indentation defines blocks, and `#` starts a comment.
- `if`, `elif`, `else`, `while`, `for x in xs`, `for i in range(n)`, `break`, `continue`, and `pass`.
- `and`, `or`, `not`, `in`, `not in`, and `a if cond else b`.
- `/` always gives a float, `//` is floor division, `%` takes the sign of the divisor, and `**` is a power.
- `len`, `str`, `int`, `float`, `abs`, `min`, `max`, `sum`, and `sorted`.
- Negative indexes and slices: `xs[-1]`, `xs[1:3]`, `s[:5]`.
- `f"..."` strings, and string methods such as `strip`, `split`, `join`, `startswith`, `replace`, and `lower`.
- `True` and `False` are accepted as well as `true` and `false`.

## Written differently

| Python | Zamak | Why |
| --- | --- | --- |
| `def area(w, h):` | `fn area(w: f64, h: f64) -> f64:` | Types let the compiler check every call. |
| `x = 1` | `let x = 1` or `let mut x = 1` | `mut` marks what can change. |
| `class Point:` with `__init__` | `struct Point:` with fields, `Point(x: 1, y: 2)` | Fields are declared once; no constructor to write. |
| `def norm(self):` | `fn norm(&self) -> f64:` | `&self`, `&mut self`, or `self` says how the method uses the value. |
| `None` | `Option.None`, with type `Option[T]` | A missing value can't be used by accident. |
| `raise ValueError("bad")` | `fail "bad"`, in a function declared `-> T!string` | Failures are part of the type. |
| `try: ... except ValueError as e:` | `call() catch e:` | Handle the error right where it happens. |
| `int(s) if s.isdigit() else 0` | `int(s) catch e: 0` | Use a default value when a call fails. |
| Exceptions propagate on their own | `call()?` | Passing an error on is visible in the code. |
| `print(a, b)` | `println(f"{a} {b}")` | `println` takes one value. |
| `[1, 2, 3]` | `let xs: [i64] = [1, 2, 3]` | A list annotation is needed when you'll change the list. |
| `{"a": 1}` | `{"a": 1}`; empty: `let m: {string: i64} = {}` | Keys are `i64` or `string`. |
| `lambda x: x * 2` | `fn(x): x * 2` | |
| `import math` | `use std/math` | |

## New in Zamak

- **Ownership.** Assigning a string, list, map, or struct moves it; read-only parameters borrow automatically. See [Moves and copies](ownership.md) and [Borrowing](borrowing.md).
- **Checked integers.** Integers are 64 bits, and overflow stops the program instead of growing into a big integer.
- **Enums and match.** `enum` with data, checked for exhaustive `match`. See [Enums, Option, and match](enums.md).
- **Native executables.** `zam build` produces one small file that runs without an interpreter.

## Not available yet

Zamak doesn't have classes with inheritance, decorators other than `@wrapping`, generators, `async`, threads, or networking. Default and keyword arguments aren't supported either; pass every argument, or use a struct for many options.

## A program side by side

```python
def word_lengths(text):
    lengths = {}
    for word in text.split():
        lengths[word] = len(word)
    return lengths

for word, n in word_lengths("zamak reads like python").items():
    print(f"{word}: {n}")
```

```zamak
fn word_lengths(text: &string) -> {string: i64}:
    let mut lengths: {string: i64} = {}
    for word in text.words():
        lengths[word] = len(word)
    lengths

fn main():
    for word, n in word_lengths("zamak reads like python"):
        println(f"{word}: {n}")
```

```text
zamak: 5
reads: 5
like: 4
python: 6
```
