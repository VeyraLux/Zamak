# Write your first program

In this chapter, you write a program that counts the words in a text and prints the most common ones. Along the way, you meet variables, loops, functions, maps, structs, and sorting. Each idea gets its own chapter later, so don't worry about the details yet.

## Step 1: Count the words

Create a file named `words.zm`:

```zamak
fn main():
    let text = "the quick brown fox jumps over the lazy dog and the fox runs"
    let mut counts: {string: i64} = {}
    for word in text.words():
        counts[word] = counts.get(word, 0) + 1
    println(counts)
```

Run it with `zam run words.zm`:

```text
{"the": 3, "quick": 1, "brown": 1, "fox": 2, "jumps": 1, "over": 1, "lazy": 1, "dog": 1, "and": 1, "runs": 1}
```

Here's what each line does:

- `let text = "..."` binds a name to a string. Bindings made with `let` can't change.
- `let mut counts: {string: i64} = {}` creates an empty map from strings to integers. `mut` means the variable can change, and the type annotation is needed because an empty map doesn't say what it will hold.
- `text.words()` splits the text on whitespace, and `for word in ...` runs the indented block once for each word.
- `counts.get(word, 0)` reads the current count, or `0` when the word isn't in the map yet. `counts[word] = ...` stores the new count.
- `println(counts)` prints the whole map. Maps keep their keys in insertion order.

## Step 2: Move the counting into a function

As programs grow, functions keep them readable. Move the counting into `count_words`:

```zamak
fn count_words(text: &string) -> {string: i64}:
    let mut counts: {string: i64} = {}
    for word in text.words():
        counts[word] = counts.get(word, 0) + 1
    counts

fn main():
    let counts = count_words("the quick brown fox jumps over the lazy dog and the fox runs")
    println(f"{len(counts)} different words")
    let n = counts["the"]
    println(f"'the' appears {n} times")
```

```text
10 different words
'the' appears 3 times
```

- `text: &string` means `count_words` borrows the string: it can read it, but the caller still owns it. You'll learn about borrowing in [Borrowing](borrowing.md).
- `-> {string: i64}` is the return type. The last expression in a function, here `counts`, is its return value.
- `f"..."` builds a string. Each `{...}` inside it is replaced by the value of the expression. `len(counts)` is the number of entries in the map.
- `counts["the"]` reads the value for a key. If the key is missing, the program stops with an error; use `get` when a key might be missing.

## Step 3: Print the most common words

To print the top words, put each word and its count in a struct, sort the structs, and keep the first few:

```zamak
const TEXT = "the quick brown fox jumps over the lazy dog and the fox runs"

struct WordCount:
    word: string
    count: i64

fn count_words(text: &string) -> {string: i64}:
    let mut counts: {string: i64} = {}
    for word in text.words():
        counts[word] = counts.get(word, 0) + 1
    counts

fn top(counts: &{string: i64}, n: i64) -> [WordCount]:
    let mut all: [WordCount] = []
    for word, count in counts:
        all.push(WordCount(word: word, count: count))
    all.sort_by(fn(a, b): a.count > b.count)
    all[:n]

fn main():
    let counts = count_words(TEXT)
    println(f"{len(counts)} different words")
    for entry in top(counts, 3):
        println(f"{entry.word}: {entry.count}")
```

```text
10 different words
the: 3
fox: 2
quick: 1
```

The new pieces:

- `const TEXT = ...` declares a constant. Constant names are uppercase.
- `struct WordCount:` declares a type with two fields, and `WordCount(word: word, count: count)` creates a value of that type.
- `for word, count in counts:` loops over the map's keys and values together.
- `all.sort_by(fn(a, b): a.count > b.count)` sorts the list. The anonymous function returns `true` when `a` goes before `b`, so larger counts come first. The sort is stable: words with the same count keep their original order.
- `all[:n]` copies the first `n` entries, as a slice does in Python.

## Try it yourself

- Change `top(counts, 3)` to print the top five words.
- Make the count case-insensitive by counting `word.lower()` instead of `word`.
- Read the text from standard input with `io.read_all()`, as shown in [Standard library tour](standard-library.md).

## Summary

You wrote a complete program that uses most of what makes up everyday Zamak code. The next chapters cover each piece in detail, starting with [Variables and types](variables.md).
