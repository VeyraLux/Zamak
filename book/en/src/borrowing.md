# Borrowing

Moving a value into every function that reads it would be impractical. Instead, a function can borrow a value: it uses the value for the duration of the call, and the caller keeps ownership.

## Read-only borrows

A parameter type that starts with `&` borrows read-only. Pass a variable, and Zamak borrows it automatically:

```zamak
fn shout(text: &string) -> string:
    text.upper() + "!"

fn total(prices: &[f64]) -> f64:
    sum(prices)

fn main():
    let name = "ada"
    let prices: [f64] = [2.5, 4.0]
    println(shout(name))
    println(total(prices))
    println(name)
    println(prices)
```

```text
ADA!
6.5
ada
[2.5, 4.0]
```

`name` and `prices` are still usable after the calls because the functions only borrowed them. You can also write the borrow explicitly, as in `shout(&name)`; both forms mean the same thing.

A read-only parameter also accepts a literal or a field directly, such as `shout("hi")` or `shout(user.name)`.

## Mutable borrows

A parameter typed `&mut` can change the caller's value. The call site says so with `&mut`, and the variable must be declared with `let mut`:

```zamak
fn add_tax(prices: &mut [f64], rate: f64):
    for p in &mut prices:
        p = p * (1 + rate)

fn main():
    let mut prices: [f64] = [10, 20]
    add_tax(&mut prices, 0.5)
    println(prices)
```

```text
[15.0, 30.0]
```

The explicit `&mut` at the call site is deliberate: when you read `add_tax(&mut prices, 0.5)`, you know `prices` might change, without looking at the function.

## Borrows are short

A borrow passed to a function lasts for that one call. A borrow can't be stored in a struct or returned from a function:

```zamak,fails
fn first(items: &[string]) -> &string:
    items[0]

fn main():
    let names: [string] = ["Ada"]
    println(first(names))
```

```text
error[ZM422]: borrow escape: functions cannot return references
 --> main.zm:1:38
  |
1 | fn first(items: &[string]) -> &string:
  |                                      ^
  = help: keep references in call arguments; store or return an owned value
```

Return an owned value instead, such as a copy of the string. Because borrows never outlive the call or block that made them, Zamak never needs lifetime annotations, which is the hardest part of learning Rust.

## Keep a borrow in a local

To read a part of a value several times, borrow it into a local:

```zamak
struct User:
    name: string
    email: string

fn main():
    let users: [User] = [User(name: "Ada", email: "ada@example.com")]
    let first = &users[0]
    println(first.name)
    println(first.email)
```

```text
Ada
ada@example.com
```

The borrow lasts until the end of the block. Until then, the borrowed value can't be changed or moved, so the local always points at valid data.

## While something is borrowed, it can't change

The checker stops you from changing a value while another part of the program is reading it. A common case is changing a list inside a loop over the same list:

```zamak,fails
fn main():
    let mut numbers: [i64] = [1, 2, 3]
    for n in numbers:
        if n == 2:
            numbers.push(4)
```

```text
error[ZM421]: borrow conflict: numbers is borrowed by an enclosing for loop or a borrowed local such as let x = &numbers[i]
 --> main.zm:5:13
  |
5 |             numbers.push(4)
  |             ^^^^^^^^^^^^^^^
  = help: finish the loop or the borrow first: collect the changes in another list, then apply them after the loop
```

In other languages, this either crashes or silently skips elements. In Zamak, it doesn't compile. Collect the new elements in a separate list, and add them after the loop:

```zamak
fn main():
    let mut numbers: [i64] = [1, 2, 3]
    let mut extra: [i64] = []
    for n in numbers:
        if n == 2:
            extra.push(4)
    numbers.extend(extra)
    println(numbers)
```

```text
[1, 2, 3, 4]
```

## Summary

- `&T` borrows read-only. Zamak borrows a variable automatically when you pass it.
- `&mut T` borrows mutably. Write `&mut x` at the call, and declare `x` with `let mut`.
- A borrow lasts one call, or one block for a local. Borrows can't be stored or returned, so you never write lifetimes.
- While a value is borrowed, it can't be changed or moved.
