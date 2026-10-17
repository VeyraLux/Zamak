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
- **Fairness notes**: Rust and Go wrap on integer overflow in these builds, and C doesn't check it; Zamak checks overflow and bounds by default. Zamak's range analysis removes checks it can prove redundant, and its 32-bit division fast path makes `primes` faster than the plain C version. One Zamak program opts out: the spectral-norm function `a(i, j)` is marked `@wrapping`, which turns off overflow checks in that function only (bounds stay checked). Without it, spectral-norm takes about 340 ms, about twice as long as C.

```sh
python bench/compare/run.py            # Windows only; writes results/compare-<time>/results.json
python bench/compare/chart.py results/compare-<time>/results.json   # writes docs/compare-*.svg
```

## Results (2026-10-17)

Windows 10, zam 0.1.0 (GCC back end), GCC 16.1.0, rustc 1.99.0, Go 1.26.4, OpenJDK 11.0.28, Node.js 26.3.0, Python 3.11.9.

| Measurement | Zamak | C | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n-body run time | 167 ms | 158 ms | 112 ms | 183 ms | 332 ms | 326 ms | 15.51 s |
| spectral-norm run time | 174 ms | 172 ms | 187 ms | 238 ms | 642 ms | 428 ms | 37.75 s |
| fannkuch-redux run time | 185 ms | 190 ms | 194 ms | 203 ms | 390 ms | 321 ms | 10.46 s |
| binary-trees run time | 273 ms | 864 ms | 963 ms | 466 ms | 378 ms | 380 ms | 2.51 s |
| primes run time | 183 ms | 504 ms | 184 ms | 513 ms | 642 ms | 302 ms | 8.50 s |
| word frequency run time | 137 ms | 122 ms | 190 ms | 190 ms | 658 ms | 816 ms | 785 ms |
| hello executable size | 56 KB | 54 KB | 128 KB | 2.3 MB | needs a JRE | needs Node.js | needs Python |
| hello startup | 16 ms | 16 ms | 18 ms | 26 ms | 141 ms | 103 ms | 148 ms |
| n-body peak memory | 3.7 MB | 3.7 MB | 3.8 MB | 5.2 MB | 51.0 MB | 66.0 MB | 12.3 MB |
| binary-trees peak memory | 19.9 MB | 11.4 MB | 11.6 MB | 17.2 MB | 364.6 MB | 373.9 MB | 20.1 MB |
| word frequency peak memory | 25.1 MB | 23.7 MB | 22.6 MB | 84.3 MB | 269.1 MB | 275.8 MB | 173.1 MB |

What this shows:

- Zamak runs in the same range as C on every compute-bound program, while it checks integer overflow and bounds by default. It's the fastest or tied for fastest on spectral-norm, fannkuch-redux, and primes.
- Zamak is the fastest of the seven languages on binary-trees: about 3 times faster than C and Rust, and faster than the garbage-collected languages, with a tenth of the memory of Java and Node.js. Small list buffers are reused after they're freed instead of going back to `malloc`, and a list literal allocates exactly the elements it holds.
- On word frequency, Zamak is close to C and faster than Rust and Go. `for word in text.words()` walks the input without copying the words, and `counts[word] = counts.get(word, 0) + 1` looks the word up once.
- Rust is the fastest on n-body; Zamak is about 6 percent behind C and ahead of Go.
- A Zamak hello world is about as small as C: under half the size of Rust and about 1/40 of Go. On compute-bound programs, it starts and uses memory like C.
- Zamak runs the compute-bound programs 44 to 94 times faster than Python, and binary-trees and word frequency about 2 to 3 times faster.
- Single-file builds are slower than `rustc` and GCC (about 0.5 s for `hello`), because each build also starts a C compiler. Zamak's caching targets multi-module projects and repeated edits, which this benchmark doesn't exercise.
