# Appendix B: Performance

Zamak compiles through C, so well-written Zamak code runs at about the speed of C. This appendix shows measured results, explains where Zamak's safety checks cost time and how the compiler removes them, and gives tips for fast code.

## Measured results

The repository's [comparison benchmark](https://github.com/VeyraLux/Zamak/tree/main/bench/compare) runs the same programs in seven languages on one machine. Run time in milliseconds (lower is better):

| Program | Zamak | C | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n-body | 212 | 222 | 129 | 208 | 361 | 356 | 16,340 |
| spectral-norm | 197 | 195 | 215 | 250 | 668 | 457 | 40,114 |
| fannkuch-redux | 197 | 200 | 207 | 214 | 391 | 336 | 10,885 |
| binary-trees | 318 | 961 | 1,041 | 473 | 405 | 402 | 2,603 |
| primes | 203 | 568 | 200 | 531 | 680 | 313 | 8,642 |
| word frequency | 154 | 160 | 191 | 208 | 742 | 972 | 880 |

A hello world is 56 KB, starts in 28 ms, and uses 3.8 MB of memory, about the same as C. The benchmark's README describes the method, the machine, and how to run it yourself.

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
