# Language comparison

This benchmark runs the same seven programs in Zamak, C, Rust, Go, Java, Node.js, and Python on one machine, and measures build time, executable size, run time, and peak memory. The charts in the main README come from it.

## Method

- **Programs**: [programs/](programs) has one idiomatic version of each program per language. None of them is tuned for speed, and the outputs of all languages must match.
  - Four programs from [The Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/), the set that language projects most often cite: `nbody` (floating point and struct fields, 2,000,000 steps), `spectralnorm` (floating point and function calls, n = 2,000), `fannkuch` (fannkuch-redux: integer arrays and permutations, n = 10), and `binarytrees` (allocating and freeing many small trees, depth 16). They follow the published algorithms, and their outputs match the published reference outputs.
  - `primes` counts the primes below 1,000,000 by trial division.
  - `wordfreq` reads about 12 MB of text (2,000,000 words, Zipf-distributed, fixed seed) from standard input and counts the words in a hash map.
  - `hello` prints one line. It measures executable size, startup time, and the smallest peak memory.
- **Builds**: Zamak with the GCC back end (`CC=gcc`, the same C compiler as the C baseline) and a fresh cache, so builds are cold. C with `gcc -O2`, Rust with `rustc -C opt-level=3`, Go with `go build`, and Java with `javac`. Node.js and Python run the source directly.
- **Measurements**: run time is the median wall-clock time of 5 runs (10 for `hello`; 1 for Python on the long programs, 3 on `wordfreq`), including process startup. Peak memory is the peak working set of the process, read with `GetProcessMemoryInfo`.
- **Fairness notes**: Rust and Go wrap on integer overflow in these builds, and C doesn't check it; Zamak checks overflow and bounds in every build. Zamak's range analysis removes checks it can prove redundant, and its 32-bit division fast path makes `primes` faster than the plain C version.

```sh
python bench/compare/run.py            # Windows only; writes results/compare-<time>/results.json
python bench/compare/chart.py results/compare-<time>/results.json   # writes docs/compare-*.svg
```

## Results (2026-10-17)

Windows 10, zam 0.1.0 (GCC back end), GCC 16.1.0, rustc 1.99.0, Go 1.26.4, OpenJDK 11.0.28, Node.js 26.3.0, Python 3.11.9.

| Measurement | Zamak | C | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n-body run time | 164 ms | 154 ms | 109 ms | 185 ms | 333 ms | 326 ms | 15.49 s |
| spectral-norm run time | 526 ms | 170 ms | 186 ms | 241 ms | 630 ms | 426 ms | 38.21 s |
| fannkuch-redux run time | 183 ms | 186 ms | 192 ms | 202 ms | 376 ms | 313 ms | 10.29 s |
| binary-trees run time | 884 ms | 967 ms | 960 ms | 457 ms | 369 ms | 363 ms | 2.39 s |
| primes run time | 184 ms | 495 ms | 181 ms | 506 ms | 629 ms | 290 ms | 8.05 s |
| word frequency run time | 336 ms | 102 ms | 132 ms | 160 ms | 610 ms | 755 ms | 733 ms |
| hello executable size | 56 KB | 54 KB | 128 KB | 2.3 MB | needs a JRE | needs Node.js | needs Python |
| hello startup | 15 ms | 15 ms | 15 ms | 25 ms | 139 ms | 94 ms | 142 ms |
| n-body peak memory | 3.7 MB | 3.7 MB | 3.8 MB | 5.2 MB | 51.0 MB | 66.4 MB | 12.3 MB |
| binary-trees peak memory | 31.5 MB | 11.4 MB | 11.6 MB | 18.2 MB | 364.7 MB | 374.4 MB | 20.1 MB |
| word frequency peak memory | 109.1 MB | 23.7 MB | 22.6 MB | 84.4 MB | 268.9 MB | 276.1 MB | 173.5 MB |

What this shows:

- Zamak runs in the same range as C and Rust on n-body, fannkuch-redux, binary-trees, and primes, while it checks integer overflow and bounds in every build. It's the fastest or tied for fastest on fannkuch-redux and primes.
- Zamak is about 3 times slower than C on spectral-norm. Every integer operation in the inner loop is checked, and indexing a list goes through a bounds-checked call that the range analysis can't remove yet. Proving list indexes in range is a planned improvement.
- Zamak is 2 to 3 times slower than C, Rust, and Go on word frequency. `text.words()` copies every word into its own string before counting, while the C and Rust versions count slices of the input. Counting words without copying them is a planned improvement.
- On binary-trees, the languages with a generational garbage collector (Java, Node.js, and Go) are faster, because they allocate small objects quickly. They pay in memory: Java and Node.js use more than 360 MB, and Zamak uses 32 MB.
- A Zamak hello world is about as small as C: under half the size of Rust and about 1/40 of Go. On compute-bound programs, it starts and uses memory like C.
- Zamak runs the compute-bound programs 44 to 94 times faster than Python, and binary-trees and word frequency about 2 to 3 times faster.
- Single-file builds are slower than `rustc` and GCC (about 0.5 s for `hello`), because each build also starts a C compiler. Zamak's caching targets multi-module projects and repeated edits, which this benchmark doesn't exercise.
