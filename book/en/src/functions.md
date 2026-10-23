# Functions

A function has a name, typed parameters, and an optional result type. Types on parameters and results are required: they document the function, and they let the compiler check every call.

## Define and call a function

```zamak
fn area(width: f64, height: f64) -> f64:
    width * height

fn greet(name: &string):
    println(f"Hello, {name}!")

fn main():
    println(area(3, 4.5))
    greet("Ada")
```

```text
13.5
Hello, Ada!
```

- `-> f64` is the result type. A function without `->` returns nothing.
- The last expression in the body is the result, so `area` doesn't need `return`. You can still write `return value`, which is handy for returning early.
- `name: &string` borrows the string instead of taking it. Most functions that only read a string, list, or map take it this way. [Borrowing](borrowing.md) explains why.

A function whose body is one expression can stay on one line:

```zamak
fn square(n: i64) -> i64: n * n

fn main():
    println(square(12))
```

```text
144
```

## Default and keyword arguments

Parameters can have default values, and you can pass arguments by name, as in Python:

```zamak
fn connect(host: &string, port: i64 = 80, timeout: f64 = 5.0) -> string:
    f"{host}:{port} ({timeout}s)"

fn main():
    println(connect("example.com"))
    println(connect("example.com", 8080))
    println(connect("example.com", timeout=1.5))
```

```text
example.com:80 (5.0s)
example.com:8080 (5.0s)
example.com:80 (1.5s)
```

- A default value is a constant (a literal, a `const`, or an operation on them) or `Option.None`. Each call gets the default afresh, so calls never share a default the way Python's mutable defaults do.
- Every parameter after one with a default must have a default too.
- In a call, write positional arguments first, then `name=value`. Methods work the same way: `p.moved(dy=5)`.
- You can build a struct the same way: `Point(x=1, y=2)` is the same as `Point(x: 1, y: 2)`.

## Return early

`return` leaves the function at once. It's useful for handling special cases first:

```zamak
fn grade(score: i64) -> string:
    if score < 0 or score > 100:
        return "invalid"
    if score >= 90:
        return "A"
    if score >= 60:
        return "pass"
    "fail"

fn main():
    println(grade(95))
    println(grade(70))
    println(grade(30))
    println(grade(120))
```

```text
A
pass
fail
invalid
```

## Return several values

To return more than one value, return a tuple and unpack it with `let`:

```zamak
fn divmod(a: i64, b: i64) -> (i64, i64):
    (a // b, a % b)

fn main():
    let (quotient, remainder) = divmod(17, 5)
    println(f"{quotient} remainder {remainder}")
```

```text
3 remainder 2
```

## Recursion

A function can call itself:

```zamak
fn factorial(n: i64) -> i64:
    if n <= 1: return 1
    n * factorial(n - 1)

fn main():
    println(factorial(20))
```

```text
2432902008176640000
```

`factorial(21)` doesn't fit in an `i64`, so it stops the program with an overflow error instead of printing a wrong number.

## Generic functions

A generic function works with many types. Put type parameters in brackets after the name:

```zamak
fn largest[T](items: &[T]) -> T:
    let mut best = items[0]
    for item in items:
        if item > best: best = item
    best

fn main():
    let numbers: [i64] = [3, 9, 4]
    let prices: [f64] = [2.5, 1.25]
    println(largest(numbers))
    println(largest(prices))
```

```text
9
2.5
```

You don't name the type at the call; Zamak infers `T` from the arguments. It then compiles a separate version of the function for each type, so generic code runs as fast as code written for one type. The compiler checks the body for each type you use, so calling `largest` on a type that doesn't support `>` is a compile-time error.

## Functions as values

A function is a value that you can pass to another function. A function type looks like `fn(i64) -> bool`:

```zamak
fn is_even(n: i64) -> bool: n % 2 == 0

fn count_if(numbers: &[i64], keep: fn(i64) -> bool) -> i64:
    let mut count = 0
    for n in numbers:
        if keep(n): count += 1
    count

fn main():
    let numbers: [i64] = [1, 2, 3, 4, 5, 6]
    println(count_if(numbers, is_even))
    println(count_if(numbers, fn(n): n > 4))
```

```text
3
2
```

`fn(n): n > 4` is an anonymous function. Its body is one expression, and Zamak infers the parameter type from where the function is used. You can also write the types: `fn(n: i64) -> bool: n > 4`.

## Closures

Anonymous functions and functions declared inside another function can use the variables around them. They can also change variables declared with `let mut`:

```zamak
fn main():
    let mut total = 0
    fn add(n: i64):
        total += n
    add(5)
    add(7)
    println(total)

    let limit = 10
    let is_small = fn(x: i64) -> bool: x < limit
    let numbers: [i64] = [3, 12, 7, 20]
    let mut kept = 0
    for n in numbers:
        if is_small(n): kept += 1
    println(kept)
```

```text
12
2
```

A function that uses variables around it is called a closure. A closure can be called, bound with `let`, and passed as an argument, but it can't be stored in a struct or list or returned from a function. That rule means a closure never outlives the variables it uses, which is how Zamak supports closures without a garbage collector. Functions that don't use outside variables have no such limits.

## Functions inside functions

A function declared inside another one can call itself, which is useful for a helper that only this function needs. Write the return type, since there is nowhere else to infer it from:

```zamak
fn main():
    fn fact(n: i64) -> i64:
        if n < 2: return 1
        return n * fact(n - 1)

    fn stars(n: i64) -> string:
        if n < 1: return ""
        return "*" + stars(n - 1)

    println(fact(5))
    println(stars(4))
```

```text
120
****
```

Recursion works the same way as any other call: the arguments are checked against the function's own parameter types, and the result has its declared type. A nested function can still use the variables around it, so a recursive helper may read them too. It cannot call a function declared after it, so order the helpers you need from the top down.

## Summary

- Parameters can have defaults (`port: i64 = 80`), and calls can pass arguments by name (`connect(host, timeout=1.5)`).
- `fn name(param: Type) -> Result:` defines a function; the last expression is the result.
- Return several values as a tuple, and unpack them with `let (a, b) = ...`.
- `fn name[T](...)` defines a generic function; Zamak infers `T` and compiles one fast version per type.
- Functions are values with types like `fn(i64) -> bool`. Closures can use and change the variables around them.
