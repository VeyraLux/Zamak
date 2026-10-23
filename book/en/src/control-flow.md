# Control flow

Zamak has the control flow you know from Python: `if`, `while`, and `for`, plus `match` for choosing between many cases.

## if, elif, and else

```zamak
fn describe(n: i64) -> string:
    if n < 0:
        return "negative"
    elif n == 0:
        return "zero"
    else:
        return "positive"

fn main():
    println(describe(-3))
    println(describe(0))
    println(describe(8))
```

```text
negative
zero
positive
```

A condition must be a `bool`. Zamak doesn't treat `0`, `""`, or an empty list as false, so write the comparison you mean, such as `if len(items) > 0:`.

Writing `=` where you meant `==` is a common typo. In a condition, Zamak reports it as an error:

```zamak,fails
fn main():
    let n = 3
    if n = 3:
        println("three")
```

```text
error[ZM110]: expected `==` to compare in the if condition; a single `=` assigns a value
 --> main.zm:3:10
  |
3 |     if n = 3:
  |          ^
  = help: compare with `==`, for example `score == 60`; a single `=` only assigns
```

## Choose a value

To pick one of two values, use a conditional expression, as in Python:

```zamak
fn main():
    let score = 72
    let result = "pass" if score >= 60 else "fail"
    println(result)
```

```text
pass
```

For more than two choices, write `if` as an expression. Each branch is a single expression, and `else` is required:

```zamak
fn main():
    let temperature = 18
    let feel = if temperature < 10: "cold" elif temperature < 25: "mild" else: "hot"
    println(feel)
```

```text
mild
```

## while loops

`while` repeats its block as long as the condition is true. `break` leaves the loop, and `continue` skips to the next round:

```zamak
fn main():
    let mut n = 0
    while true:
        n += 1
        if n % 2 == 0:
            continue
        if n > 7:
            break
        println(n)
```

```text
1
3
5
7
```

## for loops over ranges

`range(n)` counts from 0 up to, but not including, `n`. `range(a, b)` starts at `a`:

```zamak
fn main():
    for i in range(3):
        println(i)
    let mut total = 0
    for i in range(1, 11):
        total += i
    println(total)
```

```text
0
1
2
55
```

A range has no step. To count by twos or count down, use `while`.

## for loops over collections

`for` also walks through lists, maps, and strings. Add a second name to get the position, or the value of a map entry:

```zamak
fn main():
    let fruits: [string] = ["apple", "banana", "cherry"]
    for fruit in fruits:
        println(fruit)
    for i, fruit in fruits:
        println(f"{i}: {fruit}")
    let stock = {"apple": 5, "banana": 0}
    for name, count in stock:
        println(f"{name} -> {count}")
```

```text
apple
banana
cherry
0: apple
1: banana
2: cherry
apple -> 5
banana -> 0
```

A `for` loop borrows the collection, so you can still use it after the loop. While the loop runs, you can't add or remove elements. [Lists, maps, sets, and tuples](collections.md) shows how to change elements in place with `for x in &mut list`.

## match

`match` compares a value against several patterns and runs the first arm that fits. Separate alternatives with `|`, and use `_` for everything else:

```zamak
fn kind(day: i64) -> string:
    match day:
        6 | 7: "weekend"
        1 | 2 | 3 | 4 | 5: "weekday"
        _: "not a day"

fn main():
    println(kind(3))
    println(kind(7))
    println(kind(9))
```

```text
weekday
weekend
not a day
```

`match` works on integers, strings, characters, and booleans, and it's most useful with enums, which [Enums, Option, and match](enums.md) covers.

## Empty blocks

A block can't be empty. When you need one, such as a placeholder, write `pass`:

```zamak
fn main():
    for i in range(3):
        pass
    println("done")
```

```text
done
```

## Summary

- Conditions must be `bool`; compare with `==`.
- `a if cond else b` and `if c: a elif d: b else: e` choose a value.
- `while` loops while a condition holds; `break` and `continue` work in both kinds of loop.
- `for i in range(n)` counts, and `for x in items` walks a list, map, or string.
- `match` picks the first arm whose pattern fits, and `_` matches anything.
