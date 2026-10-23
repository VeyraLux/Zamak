# Lists, maps, sets, and tuples

Zamak has three ways to hold several values: lists keep values in order, maps look values up by key, and tuples group a fixed number of values that can have different types.

## Lists

A list `[T]` holds any number of values of type `T`, in order. An empty list needs a type annotation:

```zamak
fn main():
    let mut scores: [i64] = []
    scores.push(72)
    scores.push(95)
    scores.push(88)
    println(scores)
    println(len(scores))
    println(scores[0])
    println(scores[-1])
```

```text
[72, 95, 88]
3
72
88
```

Indexes start at 0, and a negative index counts from the end. An index outside the list stops the program with an error instead of reading random memory:

```zamak,panics
fn main():
    let xs: [i64] = [1, 2, 3]
    println(xs[3])
```

```text
zam: list index out of bounds
```

### Change a list

```zamak
fn main():
    let mut xs: [i64] = [5, 3, 8]
    xs[0] = 6
    xs.insert(1, 4)
    println(xs)
    let last = xs.pop()
    let first = xs.remove(0)
    println(f"{first} {last} {xs}")
    xs.extend([1, 2])
    xs.reverse()
    println(xs)
    xs.clear()
    println(len(xs))
```

```text
[6, 4, 3, 8]
6 8 [4, 3]
[2, 1, 3, 4]
0
```

| Method | What it does |
| --- | --- |
| `xs.push(v)` | Adds `v` at the end. |
| `xs.insert(i, v)` | Inserts `v` at position `i`. |
| `xs.pop()` | Removes and returns the last element. |
| `xs.remove(i)` | Removes and returns the element at `i`. |
| `xs.extend(ys)` | Moves all of `ys` to the end. |
| `xs.reverse()`, `xs.clear()`, `xs.swap(i, j)` | Reverse, empty, and swap in place. |

### Search, sort, and summarize

These work on lists of numbers, booleans, and strings:

```zamak
fn main():
    let xs: [i64] = [4, 1, 3, 1]
    println(3 in xs)
    println(xs.index_of(1))
    println(sorted(xs))
    println(f"min {min(xs)}, max {max(xs)}, sum {sum(xs)}")
    println(xs[1:3])
    let mut names: [string] = ["Linus", "Ada", "Grace"]
    names.sort()
    println(names)
```

```text
true
1
[1, 1, 3, 4]
min 1, max 4, sum 9
[1, 3]
["Ada", "Grace", "Linus"]
```

`sorted(xs)` returns a sorted copy, and `xs.sort()` sorts in place. Both are stable. A slice `xs[a:b]` copies part of the list, as in Python.

To sort any list, including a list of structs, give `sort_by` a function that returns `true` when `a` goes first:

```zamak
struct City:
    name: string
    people: i64

fn main():
    let mut cities: [City] = [
        City(name: "Lyon", people: 520000),
        City(name: "Paris", people: 2100000),
        City(name: "Nice", people: 340000),
    ]
    cities.sort_by(fn(a, b): a.people > b.people)
    for c in cities:
        println(c.name)
```

```text
Paris
Lyon
Nice
```

### Change elements in a loop

`for x in &mut xs` lets the loop change each element. Assigning to `x` writes the element:

```zamak
fn main():
    let mut prices: [f64] = [10, 20, 30]
    for p in &mut prices:
        p = p * 0.9
    println(prices)
```

```text
[9.0, 18.0, 27.0]
```

### Lists of lists

Elements can be lists or maps too. `grid[i][j]` reads and writes an inner element:

```zamak
fn main():
    let mut grid: [[i64]] = [[0, 0, 0], [0, 0, 0]]
    grid[1][2] = 7
    grid[0].push(1)
    for row in grid:
        println(row)
```

```text
[0, 0, 0, 1]
[0, 0, 7]
```

## Maps

A map `{K: V}` stores values by key. Keys are `i64` or `string`, and maps remember the order in which keys were inserted:

```zamak
fn main():
    let mut ages = {"Ada": 36, "Grace": 85}
    ages["Linus"] = 21
    ages["Ada"] = 37
    println(ages)
    println(ages["Grace"])
    println(ages.get("Guido", 0))
    println("Ada" in ages)
    println(ages.remove("Grace"))
    println(ages.keys())
    println(len(ages))
```

```text
{"Ada": 37, "Grace": 85, "Linus": 21}
85
0
true
true
["Ada", "Linus"]
2
```

- `m[k] = v` inserts or replaces.
- `m[k]` reads, and stops the program if the key is missing. `m.get(k, default)` returns `default` instead.
- `k in m` and `m.contains(k)` test for a key, and `m.remove(k)` returns whether it removed one.
- `m.keys()` and `m.values()` return lists in insertion order.

Loop over keys with `for k in m`, or over keys and values with `for k, v in m`.

### Count things

Counting is the most common use of a map. Zamak compiles this idiom into a single lookup:

```zamak
fn main():
    let mut counts: {string: i64} = {}
    for ch in "mississippi":
        counts[ch] = counts.get(ch, 0) + 1
    println(counts)
```

```text
{"m": 1, "i": 4, "s": 4, "p": 2}
```

### Group values

A map's values can be lists:

```zamak
fn main():
    let words: [string] = ["apple", "avocado", "banana", "blueberry", "cherry"]
    let mut by_letter: {string: [string]} = {}
    for w in words:
        let letter = w[0]
        if letter not in by_letter:
            by_letter[letter] = []
        by_letter[letter].push(w)
    println(by_letter)
```

```text
{"a": ["apple", "avocado"], "b": ["banana", "blueberry"], "c": ["cherry"]}
```

## Sets

A set holds values without duplicates, and checking whether a value is in it is fast. Write a literal as `{1, 2, 3}` and the type as `{i64}` or `{string}`:

```zamak
fn main():
    let mut seen = {"apple", "kiwi"}
    seen.add("apple")
    seen.add("plum")
    println(len(seen))
    println("kiwi" in seen)
    println(sorted(seen))
    let fruit = {"kiwi", "mango"}
    println(sorted(seen.intersection(fruit)))
    let empty: {i64} = {}
    println(empty)
```

```text
3
true
["apple", "kiwi", "plum"]
["kiwi"]
set()
```

- `s.add(x)` adds a value. `s.remove(x)` and `s.discard(x)` remove one and return whether it was there.
- `x in s`, `len(s)`, and `for x in s` work as they do for map keys. A set has no indexes and no order; use `sorted(s)` when you need one.
- `a.union(b)`, `a.intersection(b)`, and `a.difference(b)` return new sets, and `a.issubset(b)` tests for a subset.
- An empty set needs a type annotation: `let s: {i64} = {}`.

## Tuples

A tuple groups a fixed number of values. Write its type as `(i64, string)` and read items with `.0`, `.1`, and so on:

```zamak
fn min_max(xs: &[i64]) -> (i64, i64):
    (min(xs), max(xs))

fn main():
    let pair = (3, "three")
    println(pair.0)
    println(pair.1)
    let (low, high) = min_max([4, 9, 1])
    println(f"{low}..{high}")
    let mut scores: [(string, i64)] = []
    scores.push(("Ada", 90))
    println(scores)
```

```text
3
three
1..9
[("Ada", 90)]
```

Use a tuple for a short-lived group, such as several return values. When the group has a meaning of its own, give it a name with a struct.

## Summary

- A set `{1, 2, 3}` holds values without duplicates; its type is `{i64}`.
- `[T]` is a growable list. Indexes start at 0, negative indexes count from the end, and out-of-range indexes stop the program.
- `{K: V}` is a map with `i64` or `string` keys that keeps insertion order.
- `(A, B)` is a tuple; unpack it with `let (a, b) = ...`.
- `for x in &mut xs` changes elements in place.
