# Introduction

Zamak is a statically typed programming language that reads like Python and compiles to a small native executable. It's built around three goals:

- **Clear to read.** Blocks use indentation, the operators and built-in functions follow Python, and you never write lifetime annotations.
- **Fast and small.** Zamak compiles through C to native code. There's no garbage collector and no runtime to install, and a hello world is about 56 KB.
- **Safe by default.** Ownership rules prevent use-after-free and double frees at compile time. Integer overflow, division by zero, and out-of-bounds indexes stop the program with a clear error instead of corrupting memory.

Here's a complete program:

```zamak
struct Item:
    name: string
    price: f64
    qty: i64

fn total(items: &[Item]) -> f64:
    let mut sum = 0.0
    for item in items:
        sum += item.price * item.qty
    sum

fn main():
    let items: [Item] = [
        Item(name: "notebook", price: 4.5, qty: 3),
        Item(name: "pen", price: 1.25, qty: 10),
    ]
    for item in items:
        println(f"{item.qty} x {item.name}")
    println(f"total: {total(items)}")
```

```text
3 x notebook
10 x pen
total: 26.0
```

If you know Python, most of this already looks familiar. The differences are the ones that make Zamak fast and safe: every value has a type, `let mut` marks a variable that can change, and `&[Item]` says that `total` borrows the list instead of taking it.

## Who this book is for

This book is for anyone who wants to write Zamak programs. It doesn't assume you know a particular language, but it helps if you've written some code before. If you come from Python, [Appendix A](appendix-python.md) maps what you know to Zamak.

## How to use this book

The chapters build on each other, so read them in order the first time:

- **Getting started** installs Zamak and walks you through your first programs.
- **Language basics** covers values, operators, control flow, functions, and strings.
- **Ownership** explains how Zamak manages memory without a garbage collector. It's the one idea that's new if you come from Python, Go, or Java.
- **Organize data**, **Errors and tests**, and **Projects** cover what you need for real programs.
- **Tools** covers the `zam` command and how to work with AI coding agents.

Every code example in this book is a complete program. The book's test suite compiles and runs each one, and checks that its output matches the text under it, so the examples always work with the current version of Zamak. Examples that are meant to fail show the error that `zam` reports.

## Get help

- Report problems and ask questions on [GitHub](https://github.com/VeyraLux/Zamak/issues).
- To fix a mistake in this book, select the edit icon at the top of any page.
