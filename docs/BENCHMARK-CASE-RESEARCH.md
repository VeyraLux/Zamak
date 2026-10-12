# How new languages present benchmark comparisons — research notes for Zamak's benchmark documentation

Research question: how do **new** programming languages present comparison cases and benchmarks against established
languages (C, C++, Rust, Go, Python) at release time — and what can Zamak borrow, and what should it avoid?

Zamak's constraint set (the reason this report exists): Rust frontend → C11 → native; `i64` / `string` / `bool` /
`struct`, arrays, modules, error handling with `!string`. **No floats, generics, enums, closures, threads, FFI, or IO.**

Every claim below carries a URL. Where something could not be verified, it is marked **UNVERIFIED** rather than guessed.

---

## 0. Executive summary

Five patterns recur across the material reviewed:

1. **Established, careful projects publish *method* and *self-relative* trends, not cross-language victory tables.**
   The Computer Language Benchmarks Game publishes an explicit "Always look at the source code" disclaimer on every
   program page; Rust 1.0 shipped with *zero* benchmark numbers; Go's FAQ devotes a section to explaining why its own
   numbers on benchmark X are bad.
2. **The languages that published cross-language victory tables got publicly taken apart for it.** V is the clearest
   case: its 2019 site claimed a "200x faster" compiler while the comparison omitted that V merely *transpiles to C*
   and a C compiler still has to run.
3. **The strongest single statement of "language benchmarks are mostly not measuring languages" comes from a new
   language author** — Ginger Bill (Odin), in "Why I Hate Language Benchmarks".
4. **The best-documented fairness rules come from measurement tooling and compiler projects** (Google Benchmark,
   JMH, hyperfine, BenchExec/CLBG, Modular/Mojo's ten principles), not from language homepages.
5. **Nobody with a serious reputation benchmarks the way a marketing page wants to.** The defensible posture for a
   small new language is: *fixed case set, disclosed methodology, checksum-verified outputs, self-relative trend over
   time, and an explicit statement of what the numbers do not show.*

---

## 1. Case set / project → actual workloads → metrics → known criticisms

| Case set / project | Actual workloads | Metrics reported | Known criticisms |
|---|---|---|---|
| **The Computer Language Benchmarks Game** — <https://benchmarksgame-team.pages.debian.net/benchmarksgame/> | 10 programs, grouped by the site itself: "Insignificant io": `fannkuch-redux`, `n-body`, `spectral-norm`. "Significant io": `mandelbrot`, `fasta`, `k-nucleotide`, `reverse-complement`. "[ Contentious. Different approaches & libraries. ]": `binary-trees`, `pidigits`, `regex-redux`. | `secs` (lowest elapsed of 12 BenchExec measurements), `cpu secs` (95% t-score CI from 11 measurements, or the mean if the CI was miniscule), `mem` (peak), `gz` (gzipped source size in bytes), plus a separate **make-time** metric and a startup/warmup study. | Site itself: "Always look at the source code." Same-language submissions differ by >5× (mandelbrot: C++ 0.89 s vs Python 3 182.94 s; Go #4 3.77 s vs Go 5.04 s). Programs are single- *and* multi-thread mixed, un-optimised *and* heavily optimised mixed, "naked ffi" *and* library-mediated mixed. "We don't know how much work was done to optimize each program." |
| **kostya/benchmarks** — <https://github.com/kostya/benchmarks> (the suite V's community cites) | `Brainfuck` (`bench.b`, `mandel.b`), `Base64`, `Json`, `Matmul`, `Primes`. (Havlak is **not** in the current README TOC.) | Time ("loading required data and code self-testing are not measured"), memory as "base + increase" RSS, energy via `powercap` (Intel only). "All values are presented as: median ±median absolute deviation." | Criteria are aspirational rather than enforceable: "written as the average software developer would write them", "idiomatic". Energy metric is Intel-only. |
| **attractivechaos/plb2** — <https://github.com/attractivechaos/plb2> | 4 CPU-bound tasks: `nqueen` (15-queens; "nested loops and integer bit operations"), `matmul` (1500×1500), `sudoku` (4000 hard Sudokus via kudoku; "heavily uses small fixed-sized arrays with a bit complex logic"), `bedcov` (1,000,000 intervals, implicit interval trees). | Wall-clock seconds (hyperfine), 25 languages. | Author states: "Some JIT-based language runtimes take up to ~0.3 second to compile and warm-up. We are not separating out this startup time."; "As I am mostly a C programmer, implementations in other languages may be suboptimal." Does not evaluate "the performance of memory allocation and/or garbage collection." |
| **frol/completely-unscientific-benchmarks** — <https://github.com/frol/completely-unscientific-benchmarks> | One workload: a **Treap** implemented in C++, Java, Python, JS, Kotlin, Swift, Rust, Ada, C#, Crystal, D, F#, Go, Haskell, Lua, Modula-2/3, Nim, Oberon-07, Object Pascal, OCaml, PHP, Ruby. | Real time (s), Slowdown Time, Memory (MB), **Binary Size (MB)**, plus subjective "Expressiveness" and "Maintenance Complexity" scores 1–10. | Self-declared: "C++ was implemented separately, so it has a few shortcuts, and thus it might be not a completely fair comparison". Split scoreboards because "it doesn't seem fair to put them on the same scoreboard." Results span "different hardware, so the results should not be compared between platforms". |
| **TechEmpower Framework Benchmarks** — <https://github.com/TechEmpower/FrameworkBenchmarks> | Plaintext, JSON serialization, single-query, multi-query, data updates, fortunes (server-side templates), plus caching/auth. | Requests/second, latency percentiles, framework-specific. | **Not a language benchmark — a web-framework throughput benchmark.** Normalization rules ("All test types require Server and Date HTTP response headers... this roughly normalizes network load across all implementations"). Categorically inapplicable to a language with no IO/threads/HTTP. The site's own disclaimer page at <https://www.techempower.com/benchmarks/#section=about> returned an empty response on fetch — **UNVERIFIED**. |
| **programming-language-benchmarks.vercel.app** — <https://programming-language-benchmarks.vercel.app/> | Small algorithmic tasks per language pair (e.g. `/v-vs-go`, `/v-vs-rust`, `/c-vs-zig`). | Time, memory, per-language implementation listings. | Named by Ginger Bill as "One of the most egregious websites for this": "I recommend anyone to compare two languages of the same ilk and actually read the differences between the code in the tests; note how they are nothing alike most of the time. Different implementations & logic." |
| **Google Benchmark** — <https://github.com/google/benchmark> | Microbenchmark harness (not a case set). | Time, CPU time, iterations, optional perf counters; mean/stddev across repetitions. | Documents its own variance sources and requires `--benchmark_repetitions` > 1 to report dispersion at all. |
| **JMH (OpenJDK)** — <https://github.com/openjdk/jmh> | Microbenchmark harness. | Ops/time with warmup iterations and forked JVMs. | Ships an explicit sample (`JMHSample_34_SafeLooping`) demonstrating that the *obvious* ways to write a loop benchmark get optimized away. |
| **hyperfine** — <https://github.com/sharkdp/hyperfine> | Generic CLI timing harness. | wall clock, CPU time, peak RSS, optional cycles/instructions/cache refs. | Statistical outlier detection is a feature precisely because interference is normal. |

### What the launch material actually did

| Language / launch | Cases used | What they actually claimed | Criticism / caveat |
|---|---|---|---|
| **Go, 2009 launch** (<https://opensource.googleblog.com/2009/11/hey-ho-lets-go.html>, Nov 10 2009) | None named. | "Go combines the development speed of working in a dynamic language like Python with the performance and safety of a compiled language like C or C++. Typical builds feel instantaneous; even large binaries compile in just a few seconds. **And the compiled code runs close to the speed of C.**" | Qualitative launch claim; no table, no cases, no machine. |
| **Go at Google, SPLASH 2012** (<https://go.dev/talks/2012/splash.article>) | None. | **No prime-sieve / loop benchmark and no cross-language runtime numbers exist in this talk.** (grep for `benchmark|sieve|fast enough|performance` found none.) Its quantitative content is *build-time*: "The file contained about two thousand files that, if simply concatenated together, totaled 4.2 megabytes. By the time the #includes had been expanded, over 8 gigabytes were being delivered to the input of the compiler, a blow-up of 2000 bytes for every C++ source byte." Only runtime claim is self-relative: "some benchmarks show a doubling of performance with the development version today compared to the first release of Go version 1 early in 2012." | **UNVERIFIED:** any Go sieving/loop benchmark example attributed to "Go at Google". |
| **Go FAQ** (<https://go.dev/doc/faq>, "Why does Go perform badly on benchmark X?") | Names `pidigits.go` and `regex-dna.go` from `golang.org/x/exp/shootout`, and `reverse-complement.go` as the *fair* counterexample. | "If you measure truly comparable C and Go programs (reverse-complement.go is one example), you'll see the two languages are much closer in raw performance than this suite would indicate." | This is the canonical statement of *why* a language loses a benchmark for library reasons rather than language reasons. |
| **Go 1** (<https://go.dev/blog/go1>, 28 Mar 2012) | None. | **Zero performance or benchmark claims.** The post is entirely about stability and the compatibility promise. (Note: Go 1 = 2012, not the 2009 launch.) | — |
| **Rust 1.0** (<https://blog.rust-lang.org/2015/05/15/Rust-1.0/>, 15 May 2015) | None. | **No benchmark numbers, no language-vs-language comparisons.** Only qualitative/architectural claims: "Rust combines low-level control over performance with high-level convenience and safety guarantees... without requiring a garbage collector or runtime, making it possible to use Rust libraries as a 'drop-in replacement' for C." Only quantified claim is self-relative and about compile times: "the Nightly builds of Rust already demonstrate improvements to compile times (with more to come)". | Rust 1.0 deliberately shipped with **no marketing benchmark table**. |
| **Rust today** (<https://perf.rust-lang.org/>, <https://rustc-dev-guide.rust-lang.org/tests/perf.html>) | "a large collection of popular crates" compiled by the compiler itself. | Compiler self-relative performance over time. "At this time, only x86_64-unknown-linux-gnu builds are tracked." | Self-relative only; **no cross-language claims**. |
| **V / vlang.io** (<https://vlang.io/compare>) | Modern page: a concurrent Hacker-News fetcher in V vs Rust vs Go, **stdlib only, no timings** — a source-length/ergonomics comparison. | Modern: "Rust's compilation speed is slow, on par with C++. V compiles 1.2 million lines of code per cpu per second."; "Much smaller binaries (a simple web server written in V is ~600 KB vs ~7 MB in Go)". | See §1.1 — the historical benchmark claims were the criticised part, and the modern page quietly stopped publishing a timing table. |
| **V self-tracking** (<https://fast.vlang.io/>) | V compiler self-compilation. | V lines/s, self-compile time, `v.c` size, scan/parse/check/cgen times, RSS. | Self-relative only; **no other language is measured**. |
| **Zig** (<https://ziglang.org/learn/overview/>) | None cross-language. | Mechanism claims, partly comparative: "Speaking of performance, Zig is faster than C." Reasons: one compilation unit; "both signed and unsigned integers have illegal behavior on overflow, contrasted to only signed integers in C"; exposes a SIMD vector type; stdlib hash maps/array lists vs C linked lists; "Advanced CPU features are enabled by default, unless cross-compiling." Also publishes **binary size**: `zig build-exe hello.zig -O ReleaseSmall -fstrip -fsingle-threaded` → `9944 hello` (9.8 KiB); "A Windows build is even smaller, coming out to 4096 bytes". | **Publishes no language-vs-language benchmark table.** Its real contribution to fairness is making the optimisation/safety axis explicit: four build modes Debug / ReleaseSafe / ReleaseFast / ReleaseSmall with stated "Optimizations" and "Runtime Safety Checks" on/off. |
| **Gleam** (<https://gleam.run/>, <https://gleam.run/frequently-asked-questions/>) | None. | **No benchmark table, no language-vs-language timings.** Performance statements are qualitative and attributed to the BEAM: "Running on the battle-tested Erlang virtual machine that powers planet-scale systems such as WhatsApp and Ericsson, Gleam is ready for workloads of any size." On immutability: "All data structures in Gleam are immutable and are implemented using structural sharing, making them very efficient to update." | Gleam positions itself as *"the Erlang VM with a better type system"*, **not** as faster than Elixir. The FAQ (`https://gleam.run/faq/` is a 404; the real URL is `/frequently-asked-questions/`) has "How does Gleam compare to Rust?" and "Elixir also has a type system now..." but **no benchmark numbers**. |
| **Modular / Mojo** (<https://www.modular.com/blog/how-to-be-confident-in-your-performance-benchmarking>, 19 Mar 2024) | Recommends a common use case, e.g. "Matrix Multiplication". | Publishes **ten benchmarking principles** rather than a results table. | A vendor publishing methodology *because* its claims invite disbelief: "Mojo performance sounds too good to be true, so…". |
| **Carbon** (<https://raw.githubusercontent.com/carbon-language/carbon-lang/trunk/docs/project/goals.md>) | None. | Publishes **goals**, not numbers: "Performance-critical software; Software and language evolution; Code that is easy to read, understand, and write; Practical safety and testing mechanisms; Fast and scalable development; Modern OS platforms, hardware architectures, and environments; Interoperability with and migration from existing C++ code". | "Many languages share subsets of these goals, but what distinguishes Carbon is their combination." |
| **Hare** (<https://harelang.org/>, <https://lwn.net/Articles/893285/>) | None. | "Hare is a systems programming language designed to be simple, stable, and robust. Hare uses a static type system, manual memory management, and a minimal runtime." | Publishes no benchmark table at all; the LWN announcement thread is almost entirely about whether the language should exist. |
| **Odin** | None. | Its creator published the field's most explicit *anti*-benchmark methodology statement (see §1.2). | — |

### 1.1 V: the concrete mechanisms that made its benchmarks unfair

This is the single most instructive case for Zamak, because V is a *new language with a C backend* — structurally the
same position Zamak occupies.

- The 2019 site claimed a compiler "200x faster" than C compilers — while omitting that V **transpiles to C**, so a C
  compiler still has to run. Reported verbatim by an HN commenter, 1 Jul 2022
  (<https://news.ycombinator.com/item?id=31948321>): *"Back in 2019, its website looked like this:
  https://web.archive.org/web/20190303184805/https://vlang.io/ As you can see the website claimed that its compiler is
  '200x faster' than C compilers, while neglecting to mention that it merely translates V code to C, so you still have
  to run a C compiler. '400 KB compiler with zero dependencies' (apart from a C compiler and libc). 'As fast as C' - a
  lie."* — **UNVERIFIED as to the archived page itself**: web.archive.org was unreachable from this environment
  (connection failures / HTTP 429) for the whole session, so the 2019 snapshot could not be re-read directly.
- The V homepage in Feb 2019 claimed (*per* <https://justinas.org/the-bizarre-world-of-v>): *"V can translate your
  entire C/C++ project and offer you the safety, simplicity, and up to 200x compilation speed up."*
- A June 2019 HN commenter, quoted in the same article: *"Now that (part of?) the code has been released, it seems to
  be little more than a transpiler from V to C, with allowed inlined C, with most advertised features stubbed out."*
- The "as fast as C" claim was itself a *conditional* claim that got flattened. V's own docs said
  (<https://n-skvortsov-1997.github.io/reviews/>): *"As fast as C (V's main backend compiles to human readable C),
  with equivalent code. V does introduce some overhead for safety (such as array bounds checking, GC free), but these
  features can be disabled/bypassed when performance is more important."* The review's rebuttal: *"Just because you
  compile in C doesn't mean you instantly get the same performance as handwritten C code. I have already shown above
  how V carelessly works with memory; not a single experienced C developer will make such mistakes."*
- Contemporaneous HN summary of the controversy (<https://news.ycombinator.com/item?id=24482546>, 15 Sep 2020):
  *"there was a lot of controversy, partially because of the amount of donations being elicited before releasing
  anything and **overselling some benchmark results**."*
- The community's own benchmark citation list is <https://github.com/vlang/v/discussions/12970> — which points at
  `kostya/benchmarks`, `programming-language-benchmarks.vercel.app`, and TechEmpower. Note that the modern V
  comparison page (<https://vlang.io/compare>) **no longer publishes a timing table at all**: *"Initially I was going
  to compare V to all major languages, but it got repetitive pretty quickly."*

**Transferable lesson for Zamak:** the "compile time" and "run time" claims must never be merged, and a
*transpile-to-C* language must say out loud that a C compiler still runs. This is exactly the trap CLBG avoids by
reporting make-time as a **separate** metric from `secs`.

### 1.2 The strongest "fair benchmark methodology" statement from a new-language author

Ginger Bill (creator of Odin), "Why I Hate Language Benchmarks", 22 Jan 2024 —
<https://www.gingerbill.org/article/2024/01/22/comparing-language-benchmarks/>:

> "I absolutely HATE comparing programming languages with 'benchmarks'."
>
> "Language benchmarks rarely ever actually test for anything useful when comparing one language against another.
> This goes for ANY language."
>
> "Even in the best case scenario: you are comparing different compilers for the same language (and the same input).
> This means that you are just comparing how well the optimizing backends work for those compilers. Comparing
> different languages is not even in the same category. … And most benchmarks rarely use the semantically equivalent
> code to test against. The implementations vary widely. And even in the case where the input is semantically
> equivalent AND the compiler backends for each language use the same 'library' (e.g. LLVM), even then the semantics of
> each language may not allow for certain passes. LLVM is a good example which assumes C and C++ semantics. If your
> language does not adhere to C and C++ semantics, then most of the passes in LLVM cannot be used. And some compilers
> may have different default 'flags' too, which makes dumb comparisons rarely equal (e.g. native vs portable
> microarchitectures)."
>
> "a printing procedure in one language may be drastically different too. Runtime vs compile-time type information or
> none at all, flushing after each call or not, richer formatting or not, etc. Are you even comparing the same thing?"
>
> "There is also the 'idiomatic' aspect which I hate too. 'Idiomatic' in one language is a subjective and personal
> construct, and may produce very different results compared to 'non-idiomatic' code."
>
> "In summation, metrology is hard. … Comparing multivariate things against each other and going 'yep, that entire
> "language" is faster than this one' is misguided at best, and idiotic at worse. Please don't treat 'benchmarks' such
> as these as mostly pseudo-science, not science. Just because it has loads of numbers and 'measurements', does not
> make it 'scientific'."

### 1.3 Modular/Mojo's ten benchmarking principles (verbatim headings)

<https://www.modular.com/blog/how-to-be-confident-in-your-performance-benchmarking> — published because "different
communities can (and should!) show healthy skepticism of performance claims that seem to be 'too good to be true'".

1. Measure a common use case that matters. Don't cherry-pick.
2. Ensure correctness of results.
3. Make fair (apples-to-apples) comparisons.
4. Measure the parts that matter. Equally.
5. Make your benchmark reproducible.
6. Make sure code is not optimized out.
7. Collect results in a statistically correct & meaningful way.
8. Make your test system as deterministic as possible.
9. Use an appropriate measurement metric.
10. Be fair and well-intentioned!

On #3: *"One of the most important aspects of performance benchmarking when it pertains to comparison of different
implementations is making sure comparisons are fair. This is a place where most discussions occur, as deviation from
best practices can make one's performance claims easy to dismiss. For faster results of a given implementation (the
Mojo implementation in our case) to be meaningful, the comparison needs to be apples-to-apples."* and *"Ensure any
system-level settings remain the same during your benchmarking and across implementations (e.g., pinning threads,
NUMA configuration, any performance-related OS or hardware settings)."*

On #9: *"For most purposes, reporting time is a safe and straightforward choice. However, if applicable in a certain
domain, use the established metric."*

---

## 2. Recommended shortlist for Zamak

Selection criteria derived from §1: (a) expressible with `i64`/`string`/`bool`/`struct`/arrays/modules/`!string`;
(b) no floats, no IO beyond a final checksum print, no threads, no FFI; (c) the *algorithm* dominates the runtime, not
a standard library; (d) output is exactly checkable; (e) the same algorithm is naturally written in C, Rust, Go, and
Python without giving any language an unfair library advantage.

Zamak's existing suite (`bench/gen.mjs`, `bench/measure.ps1`, `bench/report.mjs`; documented in
`docs/PERFORMANCE.md`) already covers gcd, prime counting, iterative Fibonacci, 64-item insertion sort, 64-item
binary search, integer arithmetic sum, and a modular recurrence. The recommendations below are ordered by how well
they survive the criticisms in §1.

### Tier A — include, and say why

| Case | Why it is defensible | Which criticism it avoids |
|---|---|---|
| **Prime counting (sieve of Eratosthenes)** | Pure integer array work, no libraries, no allocation beyond one array. Output is a single count → exactly checkable. Already in Zamak's suite. | Avoids the "library vs library" criticism Go's FAQ makes about `pidigits`/`regex-dna`. |
| **GCD (Euclid, iterative)** | Branch-heavy integer loop; no library involvement whatsoever; trivially cross-checkable against a closed-form-independent implementation. | Same. Also immune to "optimized away" because the loop count is input-dependent. |
| **Iterative Fibonacci / modular recurrence** | Fixed-iteration integer arithmetic; deterministic; the modular recurrence in particular produces an exact residue, so an optimizer cannot replace it with a closed form without also producing the residue. | The modular form directly answers the "arithmetic sum may be folded into a closed form" caveat already noted in `docs/PERFORMANCE.md`. |
| **Insertion sort on a 64-item integer array** | In-place array mutation, `i64` only, exact output ordering. Small enough to be inspectable ("You do have time to inspect 100-line programs" — CLBG), large enough that the inner loop dominates. | Avoids "differences in libraries and approach would be hidden in the summary" — the same reason CLBG excludes `binary-trees` from its box plots. |
| **Binary search over a 64-item integer array** | Exact-match semantics; deterministic result. | **Caveat already documented by Zamak itself:** all seeds return 2080, so a matching checksum does *not* prove the loop was not eliminated. Keep it, but pair it with a case whose output varies by seed (see Tier A rows above) rather than relying on it alone. |
| **Binary-trees-style allocation/deallocation workload (integer-keyed, bounded depth)** | It is the one canonical shape that exercises *allocation* — the thing a compiled-to-C language actually has to be good at. | It is also CLBG's most criticised case ("Contentious. Different approaches & libraries"), and CLBG *excludes* it from the box-plot summary. **Only include if Zamak's implementation is visibly the same algorithm as the others** (same depth, same node count, same traversal order), and report peak memory alongside time. |
| **n-body (integer/fixed-point only)** | If implemented with scaled integers, it is deterministic and checkable. | **Only if floats are truly avoided.** Otherwise it becomes an unfair case for a language with no floats. See §3. |

### Tier B — include only as a clearly-labelled secondary metric

| Case | Note |
|---|---|
| **Self-compilation / self-transpile time** | Legitimate and V-style, but it is a *compile-time* metric. Report it in its own table, never mixed with run time — exactly as CLBG separates make-time from `secs`. For a transpile-to-C language, state that the C compiler still runs. |
| **Source size (bytes / gzipped)** | CLBG reports gzipped source size as a first-class metric, but also warns: *"There is some evidence that complexity metrics don't provide any more information than SLoC or LoC."* Report it, do not rank on it. |
| **Binary size** | Zig publishes this (`9944` bytes for a stripped ReleaseSmall hello); `frol/completely-unscientific-benchmarks` lists "Binary Size, megabytes" as a column. Safe and honest, but not a language-speed claim. |
| **Peak RSS** | Reported by CLBG (`mem`), kostya (`base + increase`), and hyperfine. Use the same definition as your comparators and say which one you used — hyperfine's own docs warn the per-process peak "is the largest per-process peak among the command and child processes, not the simultaneous total memory of the full process tree." |

### Tier C — the "checksum, not wall clock" pattern

For every case, follow kostya's Brainfuck pattern: a default mode that prints output, and a quiet mode that
"Accumulates the output using Fletcher-16 checksum, and prints it out after the benchmark"
(<https://github.com/kostya/benchmarks>). This gives a machine-checkable proof that all implementations computed the
same thing — the precondition Modular calls "Ensure correctness of results."

---

## 3. Cases to AVOID, and why

| Avoid | Why |
|---|---|
| **Anything requiring floats** — `mandelbrot`, `spectral-norm`, `n-body` in its canonical form, naive `matmul` on doubles | Zamak has no floats. Including them either forces a scaled-integer reimplementation that is *not* the canonical program (violating "same algorithm"), or guarantees a loss that measures a missing language feature rather than performance. This is structurally the same failure Go's FAQ admits for `pidigits`: *"pidigits.go depends on a multi-precision math package, and the C versions, unlike Go's, use GMP (which is written in optimized assembler)."* |
| **`pidigits` / arbitrary-precision arithmetic** | Measures the standard library, not the language. Go's FAQ says so explicitly. |
| **`regex-redux` / `k-nucleotide` / any regex- or string-library-dominated case** | Go's FAQ: *"Benchmarks that depend on regular expressions (regex-dna.go, for instance) are essentially comparing Go's native regexp package to mature, highly optimized regular expression libraries like PCRE."* A language with "no fast strings" loses for library reasons, not language reasons. |
| **Naive recursive Fibonacci** | The classic benchmark that measures nothing: exponential blowup means the timing is dominated by call overhead and the compiler's ability to memoize/inline, and different implementations silently differ in memoization. Prefer the *iterative* or *modular* form (already in Zamak's suite). **UNVERIFIED:** a specific citable source calling out naive recursive Fibonacci by name was not located in this session; treat this row as reasoned caution, not a citation. |
| **Microbenchmarks whose result is unused** | JMH's own sample demonstrates both failure modes: *"'saving' the result into a local variable would not work. A sufficiently smart compiler will inline work(), and figure out only the last work() call needs to be evaluated. Indeed, if you run it with varying $size, the score will stay the same!"* and accumulating instead triggers *"software pipelining effects... This will obliterate the benchmark setup."* Always sink results to something the compiler cannot remove. |
| **`binary-trees`, `regex-redux`, `pidigits` *as a headline aggregate*** | CLBG itself excludes exactly these three from its summary charts: *"Programs for binary-trees, pidits and regex-redux have not been included, because the differences in libraries and approach would be hidden in the summary."* |
| **Any TechEmpower-style web/JSON/ORM/database case** | Zamak has no IO, threads, or HTTP. Categorically inapplicable. |
| **Multi-threaded variants of any case** | Zamak has no threads. CLBG's disclaimer warns readers to *"compare 'secs' and 'cpu secs'"* precisely because mixing single- and multi-threaded programs misleads. |
| **`bedcov` / `sudoku`-style cases requiring large implicit data structures** | Not unfair per se, but plb2's own caveat applies: *"In most scripting languages, unfortunately, we have to put references to objects in an array at the cost of cache locality."* Only include if all implementations use the same layout. |
| **Aggregating cases into a single "Zamak is N× faster" number** | CLBG: *"We are profoundly uninterested in claims that these measurements, of a few tiny programs, somehow define the relative performance of programming languages aka Which programming language is fastest."* Ginger Bill: comparing "multivariate things against each other and going 'yep, that entire "language" is faster than this one' is misguided at best, and idiotic at worse." |
| **Publishing a timing table without publishing the source of every implementation** | CLBG's universal instruction is *"Always look at the source code."* The `frol` suite publishes a separate scoreboard for tuned solutions because *"it doesn't seem fair to put them on the same scoreboard."* |

---

## 4. Fairness rules checklist (with sources)

Each rule is stated as an actionable check for Zamak's harness, followed by the source that supports it.

**Before measuring**

- [ ] **Same algorithm and same input across languages; only the language differs.** Source: plb2 — *"all
      implementations use the same algorithm for each task and their performance bottlenecks do not fall in library
      functions"* (<https://github.com/attractivechaos/plb2>).
- [ ] **Do not cherry-pick a corner case, and if you use one, label it as a corner case.** Source: Modular principle
      #1 — *"picking a corner case with no practical utility in a domain will be less useful than a case that the
      majority cares about... As long as a corner case is not misrepresented as the common case but clearly noted as
      such, it is a valid contribution."* (<https://www.modular.com/blog/how-to-be-confident-in-your-performance-benchmarking>)
- [ ] **Use release/optimized builds for every language, never debug.** Source: kostya — *"All final binaries are
      releases (optimized for performance if possible) as debug performance may vary too much depending on the
      compiler."* (<https://github.com/kostya/benchmarks>)
- [ ] **Disclose the exact compiler version and every flag for every implementation.** Source: Ginger Bill — *"some
      compilers may have different default 'flags' too, which makes dumb comparisons rarely equal (e.g. native vs
      portable microarchitectures)"* (<https://www.gingerbill.org/article/2024/01/22/comparing-language-benchmarks/>).
- [ ] **Do not compare different optimization levels against each other** without saying so. Zamak already records
      Rust with and without overflow checks separately (`docs/PERFORMANCE.md`); keep that separation visible.
- [ ] **Declare the safety/optimization axis explicitly rather than hiding it.** Source: Zig's four build modes
      Debug / ReleaseSafe / ReleaseFast / ReleaseSmall with stated "Optimizations" and "Runtime Safety Checks"
      (<https://ziglang.org/learn/overview/>).
- [ ] **Record the machine, OS, kernel, and CPU.** Source: CLBG — *"Measured on a quad-core 3.0GHz Intel i5-3330 with
      15.8 GiB of RAM and 2TB SATA disk drive; using Ubuntu 24.04 x86_64 GNU/Linux 6.8.0-35-generic."*
      (<https://benchmarksgame-team.pages.debian.net/benchmarksgame/how-programs-are-measured.html>)

**Correctness**

- [ ] **Verify every implementation produces the identical output before timing it.** Source: CLBG measures only
      programs that "give the expected output", and reports "Bad Output" rows for those that do not
      (<https://benchmarksgame-team.pages.debian.net/benchmarksgame/performance/mandelbrot.html>). Modular principle
      #2: "Ensure correctness of results."
- [ ] **Use a checksum mode so cross-language equivalence is machine-checkable.** Source: kostya's Brainfuck quiet
      mode — *"Accumulates the output using Fletcher-16 checksum, and prints it out after the benchmark."*
      (<https://github.com/kostya/benchmarks>)
- [ ] **Do not treat a matching checksum as proof the loop was not optimized away when the output is constant across
      inputs.** Zamak already discovered this for binary search (*"二分查找各个种子的结果均为 2080，因此精确校验和验证结果一致，
      不能单独证明循环未被优化消除"*, `docs/PERFORMANCE.md`). Pair constant-output cases with input-varying cases.
- [ ] **Make sure the work cannot be optimized out — and prove it.** Sources: Modular principle #6 "Make sure code is
      not optimized out"; JMH's `JMHSample_34_SafeLooping` — *"A very straight-forward way to break the merging is to
      sink each result to Blackhole. This will force runtime to compute every work() call in full."*
      (<https://raw.githubusercontent.com/openjdk/jmh/master/jmh-samples/src/main/java/org/openjdk/jmh/samples/JMHSample_34_SafeLooping.java>)
- [ ] **Use a scaling control:** if cost does not grow linearly with task size, something is wrong. Source: JMH —
      *"Every benchmark requires control. We do a trivial control for our benchmarks by checking the benchmark costs
      are growing linearly with increased task size. If it doesn't, then something wrong is happening."*

**Measuring**

- [ ] **Warm up, and discard warmup results.** Source: Google Benchmark `--benchmark_min_warmup_time` — *"Minimum
      number of seconds a benchmark should be run before results should be taken into account... Results gathered
      within this period are discarded and not used for the reported result."*
      (<https://raw.githubusercontent.com/google/benchmark/main/docs/user_guide.md>); CLBG *"the first measurement is
      excluded from the mean and confidence intervals."*
- [ ] **Do not assume warmup always helps.** Source: CLBG — *"don't assume that after an arbitrary warmup phase JIT
      compilers must improve program performance -- It ain't necessarily so. In stark contrast to the traditional
      expectation of warmup, some benchmarks exhibit slowdown."* Also: *"Successive calculations with the
      fannkuch-redux #8 program became slower; calculations with the fannkuch-redux #3 program were stable, and after
      the first calculation the fannkuch-redux #2 program became faster."*
      (<https://benchmarksgame-team.pages.debian.net/benchmarksgame/performance/nbodyinprocess.html>)
- [ ] **Run enough repetitions to report dispersion, not a single number.** Source: Google Benchmark
      `--benchmark_repetitions` — *"The number of runs of each benchmark. If greater than 1, the mean and standard
      deviation of the runs will be reported."* Default is 1.
- [ ] **Interleave runs across implementations in a fixed random order.** Sources: Zamak's own harness already does
      this (`docs/PERFORMANCE.md`: *"按固定随机顺序交错执行各语言"*); Google Benchmark random interleaving — *"It
      randomly interleaves repetitions of a microbenchmark with repetitions from other microbenchmarks... Data shows
      it is able to lower run-to-run variance by 40% on average."*
      (<https://raw.githubusercontent.com/google/benchmark/main/docs/random_interleaving.md>)
- [ ] **Report a robust central statistic and say which one, and be aware mean vs median changes conclusions.**
      Source: CLBG energy study — *"Had the original summary presented medians rather than means, the differences
      would have been unremarkable."*
      (<https://benchmarksgame-team.pages.debian.net/benchmarksgame/energy-efficiency.html>). kostya reports
      *"median ±median absolute deviation"*.
- [ ] **Clear caches / reset state between implementations, and vary incidental environment details to defeat
      measurement bias.** Source: CLBG — *"file system caches and swap are cleared before measurements are made for
      each program. As part of that reset, the first measurement is excluded from the mean and confidence intervals.
      To ameliorate measurement bias we vary UNIX environment size."* (This is a direct response to the
      Mytkowicz et al. "Producing Wrong Data Without Doing Anything Obviously Wrong!" result.)
- [ ] **Control CPU frequency, boosting, SMT, NUMA, and affinity.** Source: Google Benchmark "Reducing Variance" —
      *"CPU scaling features that run on the CPU, like Intel's Turbo Boost and AMD Turbo Core and Precision Boost,
      can temporarily change the CPU frequency even when the using the 'performance' governor on Linux."*; mitigations
      listed include `taskset -c 0 ./mybenchmark`, `nice`/`chrt`, disabling SMT, and `echo 0 | sudo tee
      /sys/devices/system/cpu/cpufreq/boost`. Warnings emitted: *"***WARNING*** ASLR is enabled, the results may have
      unreproducible noise in them."* (<https://raw.githubusercontent.com/google/benchmark/main/docs/reducing_variance.md>)
- [ ] **Detect and handle outliers rather than silently averaging them in.** Source: hyperfine — *"Statistical
      outlier detection to detect interference from other programs and caching effects."*
      (<https://raw.githubusercontent.com/sharkdp/hyperfine/master/README.md>)
- [ ] **Subtract harness overhead.** Source: hyperfine — *"When benchmarked commands use a shell, hyperfine corrects
      for the shell spawning time. It runs the shell with an empty command multiple times to measure its startup
      time, then subtracts this time from each time measurement."*
- [ ] **Separate compile/make time from run time, and report them in different places.** Source: CLBG publishes a
      distinct make-time metric: *"There may be required (or chosen) actions before program run-time. Some languages
      are most known for their ahead-of-time language implementations and others for their run-time language
      implementations."*
- [ ] **Separate startup from steady-state, or say you did not.** Source: plb2 — *"Some JIT-based language runtimes
      take up to ~0.3 second to compile and warm-up. We are not separating out this startup time."*
- [ ] **State whether you measure wall clock or CPU time, because the ranking can differ.** Source: plb2 — *"we are
      measuring the elapsed wall-clock time because that is the time users often see. The ranking of CPU time may be
      slightly different."*; CLBG reports both `secs` and `cpu secs` and warns to compare them.
- [ ] **Report peak memory with an explicit definition.** Source: hyperfine — *"memory_peak_resident: Peak memory held
      in physical RAM, in bytes... This is the largest per-process peak among the command and child processes, not the
      simultaneous total memory of the full process tree."*
- [ ] **Report binary size separately from runtime.** Sources: Zig's `9944`-byte ReleaseSmall hello
      (<https://ziglang.org/learn/overview/>); `frol` lists "Binary Size, megabytes" as its own column
      (<https://github.com/frol/completely-unscientific-benchmarks>).

**Reporting**

- [ ] **Publish the source of every implementation.** Source: CLBG's universal per-program instruction — *"Always look
      at the source code."*
- [ ] **State that the numbers are machine- and batch-specific.** Source: `frol` — *"all of them running on different
      hardware, so the results should not be compared between platforms"*; CLBG — *"Ancient hardware... No doubt on
      current hardware all the programs would be faster. Would that make the measurements more informative or just
      different?"*
- [ ] **Say explicitly that the numbers do not define the languages.** Source: CLBG — *"We are profoundly uninterested
      in claims that these measurements, of a few tiny programs, somehow define the relative performance of
      programming languages aka Which programming language is fastest."*
- [ ] **Do not use short language labels as if they named a language rather than a specific implementation.** Source:
      CLBG index page — *"Concern has been expressed that short labels (like Java) demonstrate a confusion between
      programming language and language implementation. These short labels are just short. Each refers to a specific
      version of a programming language implementation."*
- [ ] **Do not let a benchmark name be read as a verdict.** Source: CLBG — *"The name 'benchmarks game' is just a
      name."* and *"There's a reason they call it a game..."*
- [ ] **Be fair and well-intentioned; respect the other language communities.** Source: Modular principle #10 —
      *"one should not approach performance benchmarking with any form of bias, but in an ethical and well-intentioned
      way... Different languages may excel at different use cases and more than one language has a place in a
      programmer's arsenal."*
- [ ] **When you lose, say why in library/feature terms rather than hiding it.** Source: Go's FAQ, the model to
      imitate — it explains *why* Go loses specific benchmarks and names the fair counterexample.
- [ ] **Make the whole thing reproducible, and invite re-measurement.** Source: CLBG — *"Please use krun or BenchExec
      or hyperfine, and start making the kind-of measurements you would like to see."*
- [ ] **Prefer a trend over time to a one-shot table.** Sources: Rust's `perf.rust-lang.org` (compiler self-relative,
      one platform, "a large collection of popular crates"); V's `fast.vlang.io` (self-relative, per-commit).

---

## 5. Direct quotes worth reusing (with URLs)

**On why toy programs at all** — <https://benchmarksgame-team.pages.debian.net/benchmarksgame/why-measure-toy-benchmark-programs.html>
> "You don't have time to inspect the source code of real applications to check that different implementations are
> kind-of comparable. Is it really the same program when written in different programming languages? You do have time
> to inspect 100-line programs. You do have time to write 100-line programs. You still might have something to learn
> from how other people write 100-line programs."

**On the standard accusation against toy benchmarks** — same page, quoting *Computer Architecture: A Quantitative
Approach*
> "The best choice of benchmarks to measure performance is real applications... Kernels, which are small, key pieces
> of real applications; Toy programs, which are 100-line programs from beginning programming assignments...; Synthetic
> benchmarks, which are small, fake programs invented to try to match the profile and behavior of real applications...
> All three are discredited today, usually because the compiler writer and architect can conspire to make the computer
> appear faster on these stand-in programs than on real applications."

**The per-program disclaimer** — e.g. <https://benchmarksgame-team.pages.debian.net/benchmarksgame/performance/mandelbrot.html>
> "Always look at the source code. (Lumpers and Splitters.) Some of these programs are single-thread and some are
> multi-thread: compare 'secs' and 'cpu secs'. Some of these programs are un-optimised and some are heavily optimised:
> sort-by 'gz' source code size. Some of these programs are 'naked ffi' and some put that in a library. Some of these
> programs are high-level and some are hand-written vector instructions. Always look at the source code."

**On what the numbers do not mean** — <https://benchmarksgame-team.pages.debian.net/benchmarksgame/box-plot-summary-charts.html>
> "We don't know how much work was done to optimize each program."
> "Look at the source code and see which of the faster programs use hand written vector instructions. Look and you
> might see simpler programs written in the same programming language."
> "Some-of the contributed programs were not re-written for multiprocessing, and may seem better if CPU seconds is the
> performance metric."

**On excluded programs** — same page
> "Programs for binary-trees, pidits and regex-redux have not been included, because the differences in libraries and
> approach would be hidden in the summary."

**On mean vs median** — <https://benchmarksgame-team.pages.debian.net/benchmarksgame/energy-efficiency.html>
> "Had the original summary presented medians rather than means, the differences would have been unremarkable."
> "Our analysis suggests that the choice of programming language implementation has no significant impact on energy
> consumption beyond execution time."

**On scope and reproducibility** — <https://benchmarksgame-team.pages.debian.net/benchmarksgame/sometimes-people-just-make-up-stuff.html>
> "No benchmark is perfect, but The Computer Language Benchmarks Game is a good starting point."
> "Differences in approach — to memory management, parallel programming, regex, arbitrary precision arithmetic,
> implementation technique — are part and parcel of using different programming languages."
> "So we accept something intermediate between chaos and rigidity — enough flex & slop & play to allow for Haskell
> programs that are not just mechanically translated from Fortran; enough similarity in the basic workloads & tested
> results."
> "We both accept PCRE and GMP and ... library code; and refuse custom memory pool and hash table implementations."
> "We've certainly not attempted to prove that these measurements, of a few tiny programs, are somehow representative
> of the performance of any real-world applications — not known — and in-any-case Benchmarks are a crock."
> "Please use krun or BenchExec or hyperfine, and start making the kind-of measurements you would like to see."

**Go's own admission** — <https://go.dev/doc/faq>
> "One of Go's design goals is to approach the performance of C for comparable programs, yet on some benchmarks it
> does quite poorly, including several in golang.org/x/exp/shootout. The slowest depend on libraries for which
> versions of comparable performance are not available in Go. For instance, pidigits.go depends on a multi-precision
> math package, and the C versions, unlike Go's, use GMP (which is written in optimized assembler). Benchmarks that
> depend on regular expressions (regex-dna.go, for instance) are essentially comparing Go's native regexp package to
> mature, highly optimized regular expression libraries like PCRE."
> "Benchmark games are won by extensive tuning and the Go versions of most of the benchmarks need attention. If you
> measure truly comparable C and Go programs (reverse-complement.go is one example), you'll see the two languages are
> much closer in raw performance than this suite would indicate."

**Go's 2009 launch, for contrast with modern practice** — <https://opensource.googleblog.com/2009/11/hey-ho-lets-go.html>
> "Go combines the development speed of working in a dynamic language like Python with the performance and safety of a
> compiled language like C or C++. Typical builds feel instantaneous; even large binaries compile in just a few
> seconds. And the compiled code runs close to the speed of C."

**Rust 1.0, for contrast** — <https://blog.rust-lang.org/2015/05/15/Rust-1.0/>
> "Rust combines low-level control over performance with high-level convenience and safety guarantees. Better yet, it
> achieves these goals without requiring a garbage collector or runtime, making it possible to use Rust libraries as a
> 'drop-in replacement' for C."
> (And: no benchmark table anywhere in the post.)

**The dead-code-elimination trap** — <https://raw.githubusercontent.com/openjdk/jmh/master/jmh-samples/src/main/java/org/openjdk/jmh/samples/JMHSample_34_SafeLooping.java>
> "First, the obviously wrong way: 'saving' the result into a local variable would not work. A sufficiently smart
> compiler will inline work(), and figure out only the last work() call needs to be evaluated. Indeed, if you run it
> with varying $size, the score will stay the same!"
> "Second, another wrong way: 'accumulating' the result into a local variable. While it would force the computation of
> each work() method, there are software pipelining effects in action, that can merge the operations between two
> otherwise distinct work() bodies. This will obliterate the benchmark setup."
> "Every benchmark requires control. We do a trivial control for our benchmarks by checking the benchmark costs are
> growing linearly with increased task size. If it doesn't, then something wrong is happening."

**V's own current retreat from numbers** — <https://vlang.io/compare>
> "Initially I was going to compare V to all major languages, but it got repetitive pretty quickly. The table above and
> the list of the features on the home page should give you a pretty good picture."

**Zig on explicit optimization levels** — <https://ziglang.org/learn/overview/>
> "Speaking of performance, Zig is faster than C."
> "When compiled with -O ReleaseSmall, debug symbols stripped, single-threaded mode, this produces a 9.8 KiB static
> executable for the x86_64-linux target."

**Gleam refusing to play** — <https://gleam.run/>
> "Running on the battle-tested Erlang virtual machine that powers planet-scale systems such as WhatsApp and Ericsson,
> Gleam is ready for workloads of any size."

---

## 6. Notes on provenance and gaps

- **UNVERIFIED / unreachable in this session:** web.archive.org (all snapshots; connection failures and HTTP 429), so
  V's 2019 homepage benchmark table could not be re-read directly — the "200x faster" claim is cited via a
  contemporaneous HN comment and a secondary article rather than the archived page itself. `salsa.debian.org` (CLBG's
  project repo) is behind an anti-AI-scraper challenge. `https://www.techempower.com/benchmarks/#section=about`
  returned an empty body, so TechEmpower's own disclaimers are unverified. The built-in web search/fetch tools are
  non-functional in this environment (no API key; DNS resolves to a reserved range).
- **UNVERIFIED:** a specific citable source naming *naive recursive Fibonacci* as a bad benchmark. The caution in §3
  is reasoning from the dead-code-elimination and "measure a common use case" sources, not a direct citation.
- **UNVERIFIED:** a Go prime-sieve/loop benchmark example in "Go at Google" (SPLASH 2012). No such content exists in
  that talk per a full-text grep.
- The canonical citation form for CLBG is: *Isaac Gouy. (n. d.). The Computer Language Benchmarks Game.
  <https://benchmarksgame-team.pages.debian.net/benchmarksgame/>* — reproduced verbatim in the page's own HTML
  comment and footer.
- CLBG is licensed under a Revised BSD license, Copyright © 2004-2008 Brent Fulgham, 2005-2025 Isaac Gouy, which
  forbids using the names "The Computer Language Benchmarks Game" / "The Benchmarks Game" / "The Computer Language
  Shootout Benchmarks" to endorse derived products without prior written permission
  (<https://benchmarksgame-team.pages.debian.net/benchmarksgame/license.html>). If Zamak's docs cite CLBG program
  *designs*, cite them as designs and do not imply CLBG endorses Zamak.
