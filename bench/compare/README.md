# Language comparison

This benchmark runs the same seven programs in Zamak, C, Rust, Go, Java, Node.js, and Python on one machine, and measures build time, executable size, run time, and peak memory. The charts in the main README come from it.

## Method

- **Programs**: [programs/](programs) has one idiomatic version of each program per language. None of them is tuned for speed, and the outputs of all languages must match.
  - Four programs from [The Computer Language Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/), the set that language projects most often cite: `nbody` (floating point and struct fields, 2,000,000 steps), `spectralnorm` (floating point and function calls, n = 2,000), `fannkuch` (fannkuch-redux: integer arrays and permutations, n = 10), and `binarytrees` (allocating and freeing many small trees, depth 16). They follow the published algorithms, and their outputs match the published reference outputs.
  - `primes` counts the primes below 1,000,000 by trial division.
  - `wordfreq` reads about 12 MB of text (2,000,000 words, Zipf-distributed, fixed seed) from standard input and counts the words in a hash map.
  - `hello` prints one line. It measures executable size, startup time, and the smallest peak memory.
- **Which Zamak build**: a **release build**, `zam/target/release/zam.exe`, from `cargo build --release --locked`. Build it before benchmarking, so the binary matches the source you are testing; a debug build is several times slower and must not be used for these tables. Every run records the compiler version and the git commit in `results.json` (`versions.zam`, `versions.zam_commit`), so a number can be traced back to the code that produced it.
- **Builds**: Zamak with the GCC back end (`CC=gcc`, the same C compiler as the C baseline) and a fresh cache, so builds are cold. C with `gcc -O2`, Rust with `rustc -C opt-level=3`, Go with `go build`, and Java with `javac`. Node.js and Python run the source directly.
- **Matched-flag controls**: Zamak's back end adds `-funroll-loops` on top of `-O2`, while the C baseline gets plain `-O2`. `c-unroll` rebuilds the same C sources with `gcc -O2 -funroll-loops`, so the difference between `c` and `c-unroll` is the cost of that one flag and the difference between `zamak` and `c-unroll` is closer to the effect of Zamak's code generation itself. `c-arena` compiles `programs/c/<task>_arena.c`, which only exists for `binarytrees`: it does the same tree work but allocates nodes from one contiguous block instead of calling `malloc` per node, matching the buffer-reuse strategy the Zamak version happens to have. Both controls are comparison runs: they are recorded in `results.json` but not counted in the headline table.
- **Measurements**: two clocks, because they answer different questions. The **outer** time is the median wall-clock time of 5 runs (10 for `hello`; 1 for Python on the long programs, 3 on `wordfreq`) measured around the process, so it includes startup, dynamic linking, and exit: that is what a user feels. The **inner** time is measured by each program around its compute loop and printed to stderr as `TIME <ms>`, so it excludes startup entirely. Both are recorded, as `run_s` and `inner_s`; `inner_s` is `null` where a program does not report it. All seven programs now emit `TIME`; `hello` trivially does not, because it has no compute loop.
- **Fairness notes**: Rust and Go wrap on integer overflow in these builds, and C doesn't check it; Zamak checks overflow and bounds by default. Zamak's range analysis removes checks it can prove redundant, and its 32-bit division fast path makes `primes` faster than the plain C version. One Zamak program opts out: the spectral-norm function `a(i, j)` is marked `@wrapping`, which turns off overflow checks in that function only (bounds stay checked). Without it, spectral-norm takes about 375 ms, about twice as long as C.

```sh
python bench/compare/run.py            # Windows only; writes results/compare-<time>/results.json
python bench/compare/chart.py results/compare-<time>/results.json   # writes docs/compare-*.svg
```

## Results (2026-10-21)

Windows 10, zam 0.1.1 (GCC back end), GCC 16.1.0, rustc 1.99.0, Go 1.26.4, OpenJDK 11.0.28, Node.js 26.3.0, Python 3.11.9. Every program now reports its own compute time, and the two C controls ran for the first time.

| Measurement | Zamak | C | C `-funroll-loops` | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n-body | 169 ms | 161 ms | 198 ms | 93 ms | 204 ms | 362 ms | 347 ms | 16.40 s |
| spectral-norm | 177 ms | 179 ms | 225 ms | 200 ms | 307 ms | 668 ms | 472 ms | 43.15 s |
| fannkuch-redux | 194 ms | 192 ms | 219 ms | 205 ms | 213 ms | 403 ms | 337 ms | 11.84 s |
| binary-trees | 265 ms | 997 ms | 903 ms | 1.13 s | 516 ms | 419 ms | 440 ms | 2.11 s |
| primes | 263 ms | 607 ms | 679 ms | 234 ms | 713 ms | 881 ms | 361 ms | 10.13 s |
| word frequency | 124 ms | 107 ms | 150 ms | 132 ms | 181 ms | 739 ms | 912 ms | 766 ms |
| n-body compute time | 163 ms | 158 ms | 194 ms | — | 183 ms | 192 ms | 248 ms | 16.32 s |
| binary-trees compute time | 250 ms | 945 ms | 880 ms | — | — | — | — | — |

The compute-time rows are the `inner_s` numbers, which exclude process startup. Only Zamak and C report them so far; the rest of the programs have no inner clock yet.

What the controls say:

- **`-funroll-loops` does not explain the gap.** On `binarytrees`, C goes from 997 ms to 903 ms with the flag, while Zamak is at 265 ms. On the other five programs the flag makes C *slower* (198 ms vs 161 ms on `nbody`, 225 vs 179 on `spectralnorm`). The flag is not a hidden advantage; if anything it costs C time on this machine.
- **Allocation strategy explains most of `binarytrees`.** `c-arena` is the same tree work with nodes taken from one contiguous block instead of a `malloc` per node. It runs in **46 ms** against Zamak's 265 ms — about 5.8 times faster than Zamak, and 22 times faster than the `malloc` version of C. So Zamak's 3.8x lead over C on this program is not the compiler generating better code; it is that the Zamak program reuses buffers while the C reference calls `malloc` per node. A C program using an arena wins outright.
- **What remains is a fair comparison.** On the five programs other than `binarytrees`, Zamak and C land within a few percent of each other with identical build flags apart from the one above. The honest summary is "same range as C", not "faster than C".

Against the 2026-10-18 run, this machine was faster across the board (most rows 10 to 30 percent lower), and not by the same factor per program, so the two tables are read as one picture rather than compared row by row. `primes` is the exception in the other direction: it went up for every language, which looks like machine state rather than a code change.

## Results (2026-10-18)

Windows 10, zam 0.1.0 (GCC back end), GCC 16.1.0, rustc 1.99.0, Go 1.26.4, OpenJDK 11.0.28, Node.js 26.3.0, Python 3.11.9.

| Measurement | Zamak | C | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n-body run time | 212 ms | 222 ms | 129 ms | 208 ms | 361 ms | 356 ms | 16.34 s |
| spectral-norm run time | 197 ms | 195 ms | 215 ms | 250 ms | 668 ms | 457 ms | 40.11 s |
| fannkuch-redux run time | 197 ms | 200 ms | 207 ms | 214 ms | 391 ms | 336 ms | 10.88 s |
| binary-trees run time | 318 ms | 961 ms | 1.04 s | 473 ms | 405 ms | 402 ms | 2.60 s |
| primes run time | 203 ms | 568 ms | 200 ms | 531 ms | 680 ms | 313 ms | 8.64 s |
| word frequency run time | 154 ms | 160 ms | 191 ms | 208 ms | 742 ms | 972 ms | 880 ms |
| hello executable size | 56 KB | 54 KB | 128 KB | 2.3 MB | needs a JRE | needs Node.js | needs Python |
| hello startup | 28 ms | 30 ms | 26 ms | 40 ms | 168 ms | 112 ms | 117 ms |
| n-body peak memory | 3.8 MB | 3.7 MB | 3.8 MB | 5.3 MB | 51.0 MB | 64.9 MB | 12.2 MB |
| binary-trees peak memory | 19.9 MB | 11.5 MB | 11.6 MB | 17.4 MB | 364.9 MB | 371.7 MB | 20.1 MB |
| word frequency peak memory | 25.1 MB | 23.7 MB | 22.6 MB | 84.1 MB | 269.0 MB | 274.4 MB | 172.4 MB |

What this run showed (superseded where the 2026-10-21 run disagrees):

- Zamak ran in the same range as C on the compute-bound programs while checking integer overflow and bounds by default.
- Rust won n-body; Zamak was third, just ahead of C and just behind Go.
- Zamak was fastest on binary-trees, about 3 times faster than C and Rust. The 2026-10-21 controls show why: the C reference allocates one node at a time while the Zamak program reuses buffers. A C version using an arena beats both by a wide margin, so this row measures allocation strategy more than code generation.
- On word frequency, Zamak is the fastest, ahead of C, Rust, and Go. `for word in text.words()` walks the input without copying the words, and `counts[word] = counts.get(word, 0) + 1` looks the word up once.
- A Zamak hello world is about as small as C: under half the size of Rust and about 1/40 of Go. On compute-bound programs, it starts and uses memory like C.
- Zamak runs the compute-bound programs 43 to 204 times faster than Python, binary-trees about 8 times faster, and word frequency about 6 times faster.
- Single-file builds are slower than `rustc` and GCC (about 0.5 s for `hello`), because each build also starts a C compiler. Zamak's caching targets multi-module projects and repeated edits, which this benchmark doesn't exercise.

## Results (2026-10-17)

The first full run, on the same machine and toolchains. Everything is within the run-to-run spread of the numbers above; that machine was a little faster across the board, including C and Rust, so the two tables are read as one picture rather than compared row by row.

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
