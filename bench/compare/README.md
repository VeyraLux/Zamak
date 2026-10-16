# Language comparison

This benchmark runs the same three small programs in Zamak, C, Rust, Go, Java, Node.js, and Python on one machine, and measures build time, executable size, run time, and peak memory. The charts in the main README come from it.

## Method

- **Programs**: [programs/](programs) has one idiomatic version of each task per language. None of them is tuned for speed.
  - `hello` prints one line. It measures executable size, startup time, and the smallest peak memory.
  - `primes` counts the primes below 1,000,000 by trial division. It measures integer arithmetic.
  - `wordfreq` reads about 12 MB of text (2,000,000 words, Zipf-distributed, fixed seed) from standard input and counts the words in a hash map. It measures strings, allocation, and hash maps.
- **Builds**: Zamak with the GCC back end (`CC=gcc`, the same C compiler as the C baseline) and a fresh cache, so builds are cold. C with `gcc -O2`, Rust with `rustc -C opt-level=3`, Go with `go build`, and Java with `javac`. Node.js and Python run the source directly.
- **Measurements**: run time is the median wall-clock time of 10 runs for `hello`, 5 for `primes` and `wordfreq`, and 1 or 3 for Python on the two longer tasks. Peak memory is the peak working set of the process, read with `GetProcessMemoryInfo`. The outputs of all languages must match, or the run stops.
- **Fairness notes**: Rust and Go wrap on integer overflow in these builds, and C doesn't check it; Zamak checks overflow and bounds in every build. Zamak's own range analysis removes checks it can prove redundant, and its 32-bit division fast path makes `primes` faster than the plain C version.

```sh
python bench/compare/run.py            # Windows only; writes results/compare-<time>/results.json
python bench/compare/chart.py results/compare-<time>/results.json   # writes docs/compare-*.svg
```

## Results (2026-10-17)

Windows 10, zam 0.1.0 (GCC back end), GCC 16.1.0, rustc 1.99.0, Go 1.26.4, OpenJDK 11.0.28, Node.js 26.3.0, Python 3.11.9.

| Task | Zamak | C | Rust | Go | Java | Node.js | Python |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `primes` run time | 185 ms | 503 ms | 183 ms | 510 ms | 639 ms | 305 ms | 7.82 s |
| `hello` executable size | 56 KB | 54 KB | 128 KB | 2.3 MB | needs a JRE | needs Node.js | needs Python |
| `hello` peak memory | 3.7 MB | 3.7 MB | 3.8 MB | 5.3 MB | 48.3 MB | 64.0 MB | 12.1 MB |
| `hello` startup | 15 ms | 15 ms | 16 ms | 26 ms | 140 ms | 102 ms | 148 ms |
| `wordfreq` run time | 365 ms | 126 ms | 158 ms | 176 ms | 658 ms | 831 ms | 766 ms |
| `wordfreq` peak memory | 109.0 MB | 23.7 MB | 22.6 MB | 84.7 MB | 268.9 MB | 276.2 MB | 174.0 MB |

What this shows:

- On integer code, Zamak runs as fast as Rust and about 2.7 times faster than the plain C version, while still checking overflow and bounds.
- A Zamak hello world is about as small as C: under half the size of Rust and about 1/40 of Go. It starts and uses memory like C.
- Zamak is slower on `wordfreq`. `text.words()` copies every word into its own string before counting, while the C and Rust versions count slices of the input. Zamak still runs it about twice as fast as Java, Node.js, and Python, but it uses more memory than C, Rust, and Go. Counting words without copying them is a planned improvement.
- Single-file builds are slower than `rustc` and GCC (about 0.5 s for `hello`), because each build also starts a C compiler. Zamak's caching targets multi-module projects and repeated edits, which this benchmark doesn't exercise.
