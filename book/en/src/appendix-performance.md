# Appendix B: Performance

Zamak compiles through C, so well-written Zamak code usually runs within a few percent of C. This appendix shows measured results, including where Zamak is slower, explains where Zamak's safety checks cost time and how the compiler removes them, and gives tips for fast code.

## Measured results

The repository's [comparison benchmark](https://github.com/VeyraLux/Zamak/tree/main/bench/compare) runs the same programs in seven languages on one machine (Windows 10, with GCC compiling both C and Zamak). Run time in milliseconds from the 2026-10-21 run (lower is better):

| Program | Zamak | C | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n-body | 208 | 201 | 118 | 254 | 404 | 418 | 18,110 |
| spectral-norm | 207 | 201 | 217 | 286 | 732 | 510 | 47,910 |
| fannkuch-redux | 209 | 213 | 221 | 257 | 453 | 371 | 13,960 |
| binary-trees | 322 | 1,040 | 1,130 | 584 | 455 | 426 | 2,410 |
| primes | 291 | 628 | 239 | 639 | 946 | 379 | 11,340 |
| word frequency | 158 | 165 | 191 | 213 | 997 | 1,040 | 819 |

Read the table with these caveats:

- **n-body**: Rust is about 40% faster than both Zamak and C.
- **binary-trees** measures allocation strategy, not code generation. The C reference calls `malloc` for every node, while Zamak reuses small list buffers. A C version with an arena allocator takes 63 ms.
- **primes**: Zamak and Rust divide in 32 bits when both numbers fit, and GCC compiles the C reference's 64-bit `%` as a full 64-bit division.
- **spectral-norm** uses `@wrapping` on its hot function; see below.
- Giving C the `-funroll-loops` flag that Zamak's back end adds doesn't make C faster.
- Results move by a few percent between runs. The benchmark's README has every run, the machine, and the commands to reproduce them.

A hello world is 16 KB on Linux, about the size of C's. On Windows, Zamak links the C runtime statically, so a hello world is about 143 KB.

## What the safety checks cost

Zamak checks integer overflow, division by zero, and list indexes. Each check is a compare and a rarely taken branch, but in a tight loop it can stop the C compiler from vectorizing or reordering code. The compiler removes checks it can prove unnecessary:

- **Range analysis** tracks the possible values of integers. In `for i in range(len(xs))`, it proves `xs[i]` is in range when `xs` doesn't change length inside the loop, and accesses the element directly.
- **Constant divisors** never need a zero check, and division by a power of two becomes a shift.
- **Small integers** that can't overflow, such as loop counters below a list's length, use plain arithmetic.

## Make a hot function faster

When a hot function does integer arithmetic that the compiler can't prove safe, and wrapping is acceptable, mark it `@wrapping`. In the spectral-norm benchmark, this one line takes the program from about 375 ms to about 200 ms:

```zamak
@wrapping
fn a(i: i64, j: i64) -> f64:
    1.0 / ((i + j) * (i + j + 1) // 2 + i + 1)

fn main():
    println(a(1, 2))
```

```text
0.125
```

Other tips:

- **Borrow instead of copying.** Take `&[T]`, `&string`, and `&Point` parameters, which cost nothing, and avoid `copy` in loops.
- **Loop with ranges over a list's length** when you need indexes: `for i in range(len(xs))` lets the compiler drop index checks.
- **Build lists with literals or `push`.** A list literal allocates exactly once, and freed small lists are reused.
- **Count with the idiom** `m[k] = m.get(k, 0) + 1`, which Zamak turns into a single lookup.
- **Loop over `text.words()` or `text.lines()` directly.** When `text` doesn't change in the loop, Zamak walks it without copying each word.

## Look at the generated C

To see exactly what Zamak produces, set `ZAMAK_DUMP_C` to a folder:

```sh
ZAMAK_DUMP_C=out zam build app.zm
```

`zam` writes one C file per module to `out`. A call such as `add(a, b)` is a checked addition, `at…(list, i)` is a checked index, and plain C operators mean the compiler proved the check unnecessary.
