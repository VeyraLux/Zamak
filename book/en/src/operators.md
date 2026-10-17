# Numbers and operators

Zamak's operators work the way they do in Python, including division. The difference is safety: when an integer result doesn't fit in 64 bits, Zamak stops the program instead of returning a wrong number.

## Arithmetic

```zamak
fn main():
    println(7 + 2)
    println(7 - 2)
    println(7 * 2)
    println(7 / 2)
    println(7 // 2)
    println(7 % 2)
    println(2 ** 10)
```

```text
9
5
14
3.5
3
1
1024
```

| Operator | Meaning |
| --- | --- |
| `+`, `-`, `*` | Add, subtract, multiply |
| `/` | Divide. The result is always an `f64`, so `7 / 2` is `3.5`. |
| `//` | Floor division: divide and round down. |
| `%` | Remainder. The result has the same sign as the divisor. |
| `**` | Power. It groups from the right: `2 ** 3 ** 2` is `2 ** 9`. |

Floor division and the remainder round toward negative infinity, as in Python, so they stay consistent for negative numbers:

```zamak
fn main():
    println(-7 // 2)
    println(-7 % 2)
    println(7.5 // 2)
```

```text
-4
1
3.0
```

## Mix integers and floats

When an operation mixes an `i64` and an `f64`, the integer becomes an `f64` first:

```zamak
fn main():
    let price = 2.5
    let count = 4
    println(price * count)
    println(count < 4.5)
```

```text
10.0
true
```

To convert explicitly, use `float(n)` or `int(x)`. `int` drops the fractional part, rounding toward zero:

```zamak
fn main():
    println(float(3))
    println(int(3.9))
    println(int(-3.9))
```

```text
3.0
3
-3
```

`f64(n)` and `i64(x)` are other names for the same conversions.

## How floats print

`println` prints the shortest text that reads back as exactly the same `f64`. A float always shows a decimal point or an exponent, so you can tell it apart from an integer:

```zamak
use std/fmt

fn main():
    println(2.0)
    println(0.1 + 0.2)
    println(1e16)
    println(fmt.fixed(3.14159, 2))
```

```text
2.0
0.30000000000000004
1e16
3.14
```

`0.1 + 0.2` isn't exactly `0.3` in any language that uses binary floating point. To print a fixed number of decimal places, use `fmt.fixed`.

## Overflow and division by zero

An `i64` holds values up to 9,223,372,036,854,775,807. When a result goes past that limit, Zamak stops the program with an error:

```zamak,panics
fn largest() -> i64: 9223372036854775807

fn main():
    println("before")
    println(largest() + 1)
    println("after")
```

```text
before
zam: integer arithmetic error
```

In C, signed overflow is undefined behavior, and Rust's release builds wrap around to a negative number; either way, the program carries on with a wrong value. Zamak checks every integer `+`, `-`, `*`, `//`, `%`, and `**`. Dividing by zero stops the program too, for integers and floats alike:

```zamak,panics
fn zero() -> i64: 0

fn main():
    println(10 // zero())
```

```text
zam: division by zero
```

These checks are cheap. The compiler proves most of them unnecessary, for example when a loop counter can't exceed a list's length, and removes them. See [Appendix B](appendix-performance.md) for measurements.

## Turn off overflow checks in one function

Some code wants integers to wrap around, such as hash functions and random number generators. Mark that function with `@wrapping`:

```zamak
@wrapping
fn hash(text: &string) -> i64:
    let mut h = 1469598103934665603
    for code in text.codepoints():
        h = (h ^ code) * 1099511628211
    h

fn main():
    println(hash("zamak"))
```

```text
3838841982605952249
```

Inside a `@wrapping` function, integer `+`, `-`, `*`, and negation wrap around. Floating-point division by zero gives infinity or NaN, as in C, instead of stopping the program. Everything else stays checked, including integer division by zero and list indexes. Only the marked function changes; the rest of the program keeps its checks.

## Comparison and logic

Comparisons give a `bool`: `==`, `!=`, `<`, `<=`, `>`, `>=`. Combine conditions with the words `and`, `or`, and `not`:

```zamak
fn main():
    let age = 30
    let member = true
    println(age >= 18 and member)
    println(not member or age > 65)
```

```text
true
false
```

`and` and `or` stop as soon as the result is known, so the right side runs only when it's needed. Their precedence follows Python: `not` binds tighter than `and`, which binds tighter than `or`.

Zamak doesn't accept `&&`, `||`, or `!` for logic; it reports an error that suggests the word to use.

## Bitwise operators

Integers support `&` (and), `|` (or), `^` (exclusive or), `~` (not), `<<` (shift left), and `>>` (shift right). Precedence follows Python, so `x & 1 == 1` means `(x & 1) == 1`:

```zamak
fn main():
    let flags = 0b1010
    println(flags & 0b0010)
    println(flags | 1)
    println(1 << 4)
    println(flags & 1 == 0)
```

```text
2
11
16
true
```

A shift count must be from 0 to 63.

## Math functions

Built-in functions that need no import include `abs`, `min`, and `max`. For square roots, powers, logarithms, trigonometry, and rounding, use `std/math`:

```zamak
use std/math

fn main():
    println(abs(-5))
    println(max(2.5, 1))
    println(math.sqrt(2.0))
    println(math.round(2.5))
    println(math.pi)
```

```text
5
2.5
1.4142135623730951
3.0
3.141592653589793
```

`math.round` rounds halfway values away from zero.

## Summary

- Division follows Python: `/` gives an `f64`, `//` and `%` round toward negative infinity.
- Mixing `i64` and `f64` converts the integer; `int` and `float` convert explicitly.
- Integer overflow and division by zero stop the program. `@wrapping` turns off overflow checks in one function.
- Logic uses the words `and`, `or`, and `not`.
