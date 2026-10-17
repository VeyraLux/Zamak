# Appendix C: Keywords and operators

## Keywords

These words are reserved and can't be used as names:

| Keyword | Use |
| --- | --- |
| `fn` | Declare a function or an anonymous function. |
| `let`, `mut` | Bind a name; make it mutable. |
| `const` | Declare a compile-time constant. |
| `struct`, `enum` | Declare a type. |
| `pub` | Make a declaration visible to other modules. |
| `use` | Import a module. |
| `extern` | Declare a C function. |
| `if`, `elif`, `else` | Branch. |
| `while`, `for`, `in`, `break`, `continue` | Loop. |
| `match` | Choose by pattern. |
| `return` | Leave a function. |
| `fail`, `catch` | Return an error; handle an error. |
| `assert` | Check a condition. |
| `and`, `or`, `not` | Logic. |
| `pass` | An empty block. |
| `self` | The value a method is called on. |
| `true`, `false` (also `True`, `False`) | Boolean values. |

## Operator precedence

From the loosest to the tightest binding. Operators on the same row group from left to right, except `**`, which groups from right to left.

| Operators | Meaning |
| --- | --- |
| `a if c else b` | Conditional expression |
| `or` | Logical or |
| `and` | Logical and |
| `not` | Logical not |
| `==`, `!=`, `<`, `<=`, `>`, `>=`, `in`, `not in` | Comparison and membership |
| `\|` | Bitwise or |
| `^` | Bitwise exclusive or |
| `&` | Bitwise and |
| `<<`, `>>` | Shifts |
| `+`, `-` | Addition and subtraction |
| `*`, `/`, `//`, `%` | Multiplication and division |
| `-x`, `~x` | Negation and bitwise not |
| `**` | Power |
| `x[i]`, `x[a:b]`, `x.f`, `f(...)`, `x?` | Index, slice, field, call, error propagation |

This order is the same as Python's. For example, `-2 ** 2` is `-(2 ** 2)`, and `x & 1 == 1` is `(x & 1) == 1`:

```zamak
fn main():
    println(-2 ** 2)
    println(2 ** 3 ** 2)
    println(1 + 2 * 3)
    println(5 & 1 == 1)
    println(not 1 > 2 and 3 > 2)
```

```text
-4
512
7
true
true
```

## Assignment operators

`=` assigns. The compound operators `+=`, `-=`, `*=`, `/=`, `//=`, `%=`, `&=`, `|=`, `^=`, `<<=`, and `>>=` update a variable, field, or element in place. Assignment is a statement, not an expression, so it can't appear inside a condition.

## Other symbols

| Symbol | Use |
| --- | --- |
| `:` | Ends a line that opens a block; separates a name from its type; separates a struct field from its value. |
| `->` | Introduces a function's result type. |
| `!` | In a type, separates the result from the error: `i64!string`. |
| `&`, `&mut` | In a type or argument, a read-only or mutable borrow. |
| `#` | Starts a comment. |
| `@wrapping` | Turns off overflow checks in the function that follows. |
| `..` | A range in `for i in a..b`. |
| `\|` | In a `match` pattern, separates alternatives. |
| `_` | In a pattern or unpacking, ignores a value. |
