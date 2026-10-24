# Appendix E: How Zamak works

This appendix explains what happens between `zam run main.zm` and your program's output. You don't need it to write Zamak, but it answers questions such as "where does memory get freed?" and "why is the second build so fast?"

## From source to executable

`zam` doesn't produce machine code itself. It translates your program into C and lets a C compiler do the rest:

1. **Parse.** `zam` reads each `.zm` file and builds a syntax tree. Indentation becomes blocks, as in Python.
2. **Check.** It infers and checks types, resolves generics, and checks ownership: who owns each value, and whether a value is used after it moved. Every error you see from `zam check` comes from this step.
3. **Generate C.** It writes one C11 file per module, plus a small runtime with strings, lists, maps, and the checks described later in this appendix.
4. **Compile and link.** It runs the C compiler it found on your system: MSVC on Windows, and GCC or Clang on Linux and macOS. The linker produces one standalone executable that doesn't need Zamak, a runtime, or a virtual machine to run.

Steps 2 to 4 are cached, as [described below](#the-build-cache), so most builds skip most of the work.

## What your code turns into

This program uses a string list, a function that can fail, a generic function, and an interface:

```zamak
interface Shape:
    fn area(&self) -> f64

struct Square:
    side: f64

    fn area(&self) -> f64:
        self.side * self.side

fn parse(text: &string) -> i64!string:
    if text == "":
        fail "empty"
    int(text)?

fn largest[T](items: &[T]) -> T:
    let mut best = items[0]
    for x in items:
        if x > best:
            best = x
    best

fn main():
    let names = ["ada", "grace"]
    let nums: [i64] = [3, 9, 4]
    let n = parse("42") catch e: 0
    let shapes: [Shape] = [Square(side: 2.0)]
    println(f"{len(names)} {n + 1} {largest(nums)} {shapes[0].area()}")
```

```text
2 43 9 4.0
```

Here's how each part looks in the generated C. The real output uses encoded names such as `f6d61696e3a7061727365` for `main:parse`; the excerpts below use readable names instead.

**A string** is a pointer to UTF-8 bytes, a length in bytes, and a length in characters. A list is a pointer, a length, and a capacity, like a Python list or a Rust `Vec`:

```c
typedef struct { unsigned char *data; size_t len; size_t chars; } S;
```

**A function that can fail** returns a small struct that holds either the value or the error. `fail` fills in the error, `?` returns early when `ok` is 0, and `catch` replaces the value:

```c
struct Result_i64 { int ok; int64_t value; S error; };

Result_i64 r = parse(&text);
if (!r.ok) {          // catch e: 0
    drop(&r.error);
    r.ok = 1;
    r.value = 0;
}
```

No exceptions or stack unwinding are involved, so an error costs the same as returning a value.

**A generic function** is compiled once for each type it's used with. `largest(nums)` calls a C function made for `i64`, named `main:largest[Int]`. Each copy is as fast as code written for that type by hand.

**An interface value** holds a pointer to the data and a pointer to a table of functions, one entry per method. `shapes[0].area()` looks up `area` in that table. Calls through an interface or a function value are the only calls that go through a pointer. Everywhere else, the C compiler sees the exact function and can inline it.

```c
struct Shape_table { void (*drop)(void *); void (*show)(S *, void *); double (*area)(void *); };
double Shape_area(Shape *p) { return p->table->area(p->data); }
```

## Memory without a garbage collector

Every value has exactly one owner: a variable, a field, or a list element. When the owner goes out of scope, `zam` inserts the code that frees it. At the end of `main` in the example, the generated C ends like this:

```c
drop_list_of_strings(&names);
drop_list_of_i64(&nums);
drop_list_of_shapes(&shapes);
```

Three rules make this safe, and the checker enforces all of them before any C is generated:

- **Assignment moves.** `let b = a` gives the value to `b`, and using `a` afterward is a compile error. So a value can't be freed twice. Numbers, booleans, and structs made only of such plain values are copied instead, because copying them is cheap.
- **A borrow lasts for one call.** A `&` or `&mut` parameter lends the value to the function, and the loan ends when the call returns. A function can't store a borrow or return one. So no pointer outlives the value it points to, and you never write a lifetime annotation.
- **Only one mutable borrow at a time.** While a function holds `&mut x`, nothing else can read or change `x`.

The cost is predictable: memory is freed at a known point, with no pauses and no background collector.

## Checks that stop the program

Integer arithmetic and indexing are checked at run time. An overflowing `+`, `-`, or `*` calls the C compiler's overflow-checking built-ins, and an index outside a list stops the program with a message:

```c
if (__builtin_add_overflow(a, b, &result)) arithmetic_error();
if (index < 0 || (uintmax_t)index >= length) { panic("list index out of bounds"); }
```

The program stops instead of continuing with a wrong number or reading memory it doesn't own. When `zam` can prove a value stays in range, it leaves the check out. In `for i in 0..4` over a four-element array, for example, neither `a[i]` nor `i + 1` is checked. To let integers wrap around in one function, such as a hash function, mark it with `@wrapping`.

## The build cache

`zam` keeps a cache in `~/.zam/cache`, shared by every project on your computer. It has three levels, and `zam` prints a line for each one:

| Line | What's cached | The key includes |
| --- | --- | --- |
| `CACHE` | A checked module | The module's source and the modules it uses |
| `OBJECT` | A compiled C object file | The generated C, the C compiler and its version, and the compiler options |
| `LINK` | The executable | All the object files and the link options |

A key is a hash of the content, not a file name or a time stamp. As a result:

- Building the same code twice, even in another folder or Git worktree, compiles it once.
- Changing one module rebuilds that module and the modules that use it, and nothing else.
- Upgrading `zam` or the C compiler changes the keys, so old results are never reused by mistake.

`zam` writes each cache entry to a temporary file and renames it into place, so two builds running at the same time never see a half-written entry. It also remembers which C compiler it found and that compiler's version, so a build where nothing changed finishes in a few milliseconds.

## Bundled C libraries

`zam` carries the source code of three C libraries inside its own executable: Mbed TLS for HTTPS, libui-ng for `std/ui`, and SQLite for `std/sqlite`. The first time your program uses one, `zam` writes its source to the cache and compiles it there. Later programs reuse the compiled objects, so only the first HTTPS program on a computer pays the cost of compiling Mbed TLS, about 13 seconds.

These libraries become part of your executable. If you distribute it, include their license notices, which are listed in `THIRD-PARTY-NOTICES.md` in the Zamak repository.

## Threads

`thread.spawn` starts an operating system thread. Its arguments move to the new thread, so after the call, the thread that called `spawn` can't touch them. Threads exchange values through a `thread.Queue`, which also moves each value. Because no two threads can reach the same changeable value, Zamak programs have no data races, and they don't need locks for your own data.
