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
# 全部语言、全部程序，写进 results/compare-<time>/results.json
python bench/compare/run.py

# 只跑两项对照，几分钟就能自己验证
python bench/compare/run.py --languages zamak,c,c-unroll --tasks binarytrees
python bench/compare/run.py --languages c,c-arena --tasks binarytrees

# 出图
python bench/compare/chart.py results/compare-<time>/results.json
```

Every number in this file can be reproduced with those commands. `--languages` and `--tasks` take comma-separated names; `run.py --help` lists the accepted values. A single-task comparison finishes in a couple of minutes, so there is no need to trust the tables: run the two control commands above and read `run_s` from the JSON.

The programs themselves are in [programs/](programs), one file per language per task. `programs/c/binarytrees_arena.c` is the arena variant used by `c-arena`; it is a separate file rather than a flag so the diff against `binarytrees.c` is readable.

## Results (2026-10-21)

Windows 10, zam 0.1.1 (GCC back end), GCC 16.1.0, rustc 1.99.0, Go 1.26.4, OpenJDK 11.0.28, Node.js 26.3.0, Python 3.11.9. Every program now reports its own compute time, and both C controls ran in the same batch as everything else.

| Measurement | Zamak | C | C `-funroll-loops` | C arena | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| n-body | 208 ms | 201 ms | 211 ms | — | 118 ms | 254 ms | 404 ms | 418 ms | 18.11 s |
| spectral-norm | 207 ms | 201 ms | 220 ms | — | 217 ms | 286 ms | 732 ms | 510 ms | 47.91 s |
| fannkuch-redux | 209 ms | 213 ms | 229 ms | — | 221 ms | 257 ms | 453 ms | 371 ms | 13.96 s |
| binary-trees | 322 ms | 1.04 s | 1.07 s | **63 ms** | 1.13 s | 584 ms | 455 ms | 426 ms | 2.41 s |
| primes | 291 ms | 628 ms | 681 ms | — | 239 ms | 639 ms | 946 ms | 379 ms | 11.34 s |
| word frequency | 158 ms | 165 ms | 168 ms | — | 191 ms | 213 ms | 997 ms | 1.04 s | 819 ms |

A dash means that control has no variant for that program. `c-arena` is only written for `binarytrees`, which is the one program where the allocation strategy is the question.

What the controls say:

- **`-funroll-loops` does not explain anything.** On `binarytrees` the flag leaves C where it was (1.04 s to 1.07 s) while Zamak is at 322 ms. On the other five programs it makes C slightly *slower*: 211 ms vs 201 on `n-body`, 220 vs 201 on `spectralnorm`, 229 vs 213 on `fannkuch`, 681 vs 628 on `primes`, 168 vs 165 on `wordfreq`. The flag is not a hidden advantage; on this machine it costs C time.
- **Allocation strategy is the whole story on `binarytrees`.** `c-arena` does the same tree work with nodes taken from one contiguous block instead of a `malloc` per node. It finishes in **63 ms** against Zamak's 322 ms — 5.1 times faster than Zamak, and 16.5 times faster than the `malloc` version of C. So Zamak's 3.2x lead over C there is not the compiler emitting better code: the Zamak program reuses small list buffers while the C reference calls `malloc` once per node. A C program with an arena wins outright, and by a wide margin.
- **What is left is the fair comparison.** On the four programs other than `binarytrees` and `primes`, Zamak and C are within a few percent of each other — sometimes ahead, sometimes behind. The honest summary is "same range as C", not "faster than C". The earlier README said "3 times faster than C" without that caveat; that claim is withdrawn.

Two things make the tables readable only as one picture, not row by row. Run-to-run spread is real: the 2026-10-18 run had Zamak at 265 ms on `binarytrees`, this one 322 ms, with the C and Rust numbers moving too. And machine state moves every language at once — `primes` came out 10 to 30 percent slower for all seven languages in this batch than in the previous one. Differences of a few percent between two languages in a single run are not a ranking.

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
