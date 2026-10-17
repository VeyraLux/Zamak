# Strings

A `string` holds UTF-8 text. As in Python, lengths, indexes, and slices count characters, not bytes, so text in any language works the way you expect.

## Write strings

String literals use double quotes. Escape sequences start with a backslash:

```zamak
fn main():
    println("Tab:\tafter tab")
    println("Quote: \"hi\"")
    println("Backslash: \\")
    println("Two\nlines")
```

```text
Tab:	after tab
Quote: "hi"
Backslash: \
Two
lines
```

The escapes are `\n` (newline), `\r` (carriage return), `\t` (tab), `\"`, and `\\`.

## Interpolation

Braces inside a string insert the value of an expression. The `f` prefix is optional; Zamak accepts it so that Python code reads naturally:

```zamak
fn main():
    let name = "Ada"
    let items: [i64] = [3, 4]
    println("Hello, {name}!")
    println(f"{name} has {len(items)} items, total {items[0] + items[1]}")
    println("Use {{ and }} for literal braces")
```

```text
Hello, Ada!
Ada has 2 items, total 7
Use { and } for literal braces
```

An interpolated expression can't contain a string literal or braces. Bind the value to a name first when you need one.

## Length, indexes, and slices

```zamak
fn main():
    let word = "naïve 你好"
    println(len(word))
    println(word.byte_len())
    println(word[0])
    println(word[-1])
    println(word[2:5])
    println(word[:5])
    println(word[6:])
```

```text
8
13
n
好
ïve
naïve
你好
```

- `len(s)` counts characters. `s.byte_len()` counts UTF-8 bytes.
- `s[i]` is the character at position `i`, as a one-character string. A negative index counts from the end.
- `s[a:b]` copies the characters from `a` up to, but not including, `b`. Leave out either end to start at the beginning or stop at the end. Bounds out of range are clamped, as in Python.

## Loop over characters

```zamak
fn main():
    for ch in "héllo":
        print(ch)
    for i, ch in "abc":
        println(f"{i}={ch}")
```

```text
h
é
l
l
o
0=a
1=b
2=c
```

## Build and compare strings

`+` joins two strings into a new one. `==` and `<` compare contents:

```zamak
fn main():
    let first = "Grace"
    let last = "Hopper"
    let full = first + " " + last
    println(full)
    println(full == "Grace Hopper")
    println("apple" < "banana")
    let mut log = ""
    for i in range(3):
        log += f"[{i}]"
    println(log)
```

```text
Grace Hopper
true
true
[0][1][2]
```

A string and a number can't be joined with `+`; Zamak reports a type error. Use interpolation instead: `f"n = {n}"`.

## String methods

Call string functions as methods. They return new strings and never change the original:

```zamak
fn main():
    let text = "  Hello, World  "
    println(text.strip())
    println(text.lower())
    println(text.upper().strip())
    println(text.replace("World", "Zamak"))
    println(text.contains("World"))
    println(text.strip().startswith("Hello"))
    println(text.find("World"))
    println("ab".repeat(3))
```

```text
Hello, World
  hello, world  
HELLO, WORLD
  Hello, Zamak  
true
true
9
ababab
```

`find` returns the character position of the first match, or `-1`. Python's names work: `strip`, `lstrip`, `rstrip`, `startswith`, `endswith`, `isdigit`, `isalpha`, `isspace`, `ljust`, and `rjust`.

## Split and join

```zamak
fn main():
    let csv = "red,green,,blue"
    let parts = csv.split(",")
    println(parts)
    println(len(parts))
    println("one  two\tthree".words())
    println(" | ".join(parts))
    println("line 1\nline 2\n".lines())
```

```text
["red", "green", "", "blue"]
4
["one", "two", "three"]
red | green |  | blue
["line 1", "line 2"]
```

- `split(sep)` keeps empty fields, as in Python.
- `words()` splits on any run of whitespace and drops empty pieces.
- `lines()` splits on newlines, drops a trailing `\r`, and doesn't add an empty line after a final newline.
- `sep.join(list)` joins a list of strings.

## Parse numbers

`int(s)` and `float(s)` parse text. Parsing can fail, so the result must be handled; [Handle errors](errors.md) explains `?` and `catch`:

```zamak
fn main() -> !string:
    let n = int("42")?
    let x = float("2.5")?
    println(n + 1)
    println(x * 2)
```

```text
43
5.0
```

## Summary

- Strings are UTF-8, and `len`, indexes, and slices count characters.
- `"{expr}"` inserts a value; `+` joins strings.
- String functions are methods: `s.strip()`, `s.split(",")`, `s.find("x")`.
- `int(s)` and `float(s)` parse numbers and can fail.
