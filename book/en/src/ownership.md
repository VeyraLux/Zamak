# Moves and copies

Zamak has no garbage collector, and you never free memory yourself. Instead, every string, list, map, struct, and enum has exactly one owner. When the owner goes away, Zamak frees the value. This chapter explains the rules, which are the main thing that's new if you come from Python, Go, or Java.

## Each value has one owner

A variable owns the value it holds. When the variable's block ends, the value is freed:

```zamak
fn main():
    let greeting = "hello"
    if true:
        let shout = greeting.upper()
        println(shout)
    println(greeting)
```

```text
HELLO
hello
```

`shout` is freed at the end of the `if` block, and `greeting` at the end of `main`. The compiler inserts these frees for you, so there are no leaks and no double frees.

## Assigning moves the value

Assigning a string, list, map, struct, or enum to another variable moves it. The new variable becomes the owner, and the old one can't be used anymore:

```zamak,fails
fn main():
    let name = "Ada"
    let other = name
    println(other)
    println(name)
```

```text
error[ZM301]: use after move: name
 --> main.zm:5:5
  |
5 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

Moving is cheap: it copies a pointer, not the text. And because only one variable owns the text, Zamak knows exactly when to free it.

## Passing to a function can move too

A parameter whose type has no `&` takes ownership:

```zamak,fails
fn consume(items: [i64]) -> i64:
    len(items)

fn main():
    let numbers: [i64] = [1, 2, 3]
    println(consume(numbers))
    println(numbers)
```

```text
error[ZM301]: use after move: numbers
 --> main.zm:7:5
  |
7 |     println(numbers)
  |     ^^^^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

Most functions only need to look at their argument, so they take a borrow instead: `items: &[i64]`. With a borrow, the caller keeps ownership. The next chapter, [Borrowing](borrowing.md), covers this in detail. Take ownership only when the function keeps the value, for example by storing it in a struct it returns.

## Returning gives ownership back

A function can hand ownership to its caller by returning the value:

```zamak
fn with_total(items: [i64]) -> [i64]:
    let mut result = items
    result.push(sum(result))
    result

fn main():
    let numbers: [i64] = [1, 2, 3]
    let extended = with_total(numbers)
    println(extended)
```

```text
[1, 2, 3, 6]
```

## Numbers and booleans are copied

`i64`, `f64`, and `bool` values are small, so assigning them copies the value. Both variables stay usable:

```zamak
fn main():
    let a = 10
    let b = a
    println(a + b)
```

```text
20
```

## Make an explicit copy

When you really need two independent values, copy one with `copy`. It makes a deep copy of a string, list, map, struct, or enum:

```zamak
fn main():
    let original: [i64] = [1, 2, 3]
    let mut duplicate = copy(&original)
    duplicate.push(4)
    println(original)
    println(duplicate)
```

```text
[1, 2, 3]
[1, 2, 3, 4]
```

Copying costs time and memory in proportion to the size of the value, which is why Zamak never does it silently. When you see `copy`, you know a copy happens there.

## Move after the last use

A move is only a problem if you use the old variable afterwards. Moving a value as the last thing you do with it is fine:

```zamak
struct Order:
    customer: string
    items: [string]

fn main():
    let customer = "Ada"
    let mut items: [string] = []
    items.push("notebook")
    items.push("pen")
    println(f"{customer} ordered {len(items)} items")
    let order = Order(customer: customer, items: items)
    println(order)
```

```text
Ada ordered 2 items
Order(customer: "Ada", items: ["notebook", "pen"])
```

## Why ownership

- **No garbage collector.** Memory is freed at a known point, so there are no pauses and no extra memory held for later collection.
- **No memory bugs.** Use after free and double frees can't compile.
- **No hidden costs.** Large values are never copied unless you write `copy`.

## Summary

- Every string, list, map, struct, and enum has one owner, and it's freed when the owner's block ends.
- Assigning or passing such a value moves it; using the old name afterwards is an error.
- Numbers and booleans are copied.
- `copy(&x)` makes an explicit deep copy.
