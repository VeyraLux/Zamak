# Why Zamak

**Reads like Python. Runs like C. Safe without a garbage collector.**

## Why this language exists

Zamak grew out of a new way of working: several AI agents editing the same project in parallel, across worktrees, each one checking, building, and testing hundreds or thousands of times a day. At that pace, how long a build takes, how big the artifact is, and whether a machine can read the error stop being matters of taste and become matters of throughput.

So Zamak trades differently from most languages. When goals conflict, this order decides:

1. **Build cost**: the whole loop (read, edit, check, build, test, read the error) times how often it runs per day, plus disk footprint
2. **Memory safety**: safe by default, and unsafe code has to be marked
3. **Runtime performance**: reach the Rust tier and stop, with no attempt to beat it
4. **Expressiveness**: Python-level readability, omitting noise but never safety

That is why macros, dependent types, and anything needing whole-program analysis are out of scope: they slow the build down, and build speed is the first principle here. Generics are monomorphized, so each type gets one copy of the code, which keeps the generated C predictable.

## What it borrows

Zamak does not try to be a shadow of another language. It combines the parts each language does best:

| From | What we wanted | How Zamak gets it |
| --- | --- | --- |
| Rust | Memory safety, no GC | Ownership plus call-frame borrows: values move by default, a borrow never outlives the call, and there are no lifetime annotations |
| Go | Fast builds, one file to ship | Independently compilable units, content-addressed cache, one `zam` command for check, build, test, and format |
| C | Small binaries, speed, access to existing code | Emits C and hands it to MSVC, GCC, or Clang; no runtime; `extern fn` calls C directly |
| Python | Readability, nothing to unlearn | Indentation blocks, `len`, `range`, `in`, slicing, and Python's division rules |

## What you get

- **Python's readability at C's speed.** Zamak reads like Python: `for i in range(n)`, `xs[-1]`, `f"..."`, and Python's division rules. It compiles to native code that runs within a few percent of C on n-body, spectral-norm, fannkuch-redux, and word frequency, and runs the compute-bound programs 39 to 231 times faster than Python. See [Appendix B](appendix-performance.md) for where it is slower and why.
- **Memory safety you don't have to fight.** Ownership rules out use-after-free and double frees at compile time, with no garbage collector. Read-only borrows happen automatically, a borrow lasts at most one call, and you never write lifetime annotations. When the checker does reject code, the error names the problem and suggests the fix.
- **Safer integer math than C, Rust, or Go.** Integer overflow, division by zero, and out-of-bounds indexes stop the program with a clear error in every build, including release builds. The compiler removes the checks it can prove unnecessary, so safe code still runs at C speed, and `@wrapping` turns checks off in one function when you want wrapping, such as in a hash.
- **Small programs with a small footprint.** On Linux, a Zamak hello world is 16 KB: about the size of C's, and 1/17 of the same program in Rust even after `strip` plus LTO. On Windows the binary is bigger either way (`cl` links the C runtime statically): about 143 KB with the default MSVC toolchain, against 106 KB for a maximally stripped Rust build, so there the two are in the same range. On binary-trees, Zamak is the fastest of the seven languages tested and uses 20 MB, where Java and Node.js use more than 360 MB.
- **Built for AI coding agents.** The whole language fits in a one-page reference that agents read before they write code. Every diagnostic has a stable code and a suggested fix, also as JSON. `zam new --template ai` sets up a project with a skill file for your agent, and `zam mcp` lets agents check, run, test, and look up docs through MCP.
- **Fast feedback.** `zam check` finishes in less than 0.1 seconds on a 5,000-line project, and a one-line edit rebuilds in under a second, because `zam` caches every module by content.

The numbers come from a benchmark you can run yourself. See [B. Performance](appendix-performance.md) and the [benchmark repository files](https://github.com/VeyraLux/Zamak/tree/main/bench/compare).

## Compared with popular languages

Each popular language asks you to give something up, and Zamak is designed so that you don't have to choose:

- **Python** is easy to read and write, but it's slow, it needs an interpreter on every machine, and it finds type errors only when the code runs.
- **C** is fast and small, but one wrong index or integer overflow silently corrupts memory.
- **Rust** is fast and memory safe, but lifetimes and the borrow checker take a long time to learn, and its release builds don't check integer overflow.
- **Go** is simple and safe, but it ships a garbage collector, and a hello world is 2.3 MB.
- **Java** and **Node.js** need a large runtime, and they use hundreds of megabytes on allocation-heavy work.

## When to choose something else

Zamak is young. It doesn't support threads yet, and its package ecosystem is just starting. For production services, large existing codebases, or work that depends on mature libraries, use an established language today, and try Zamak for command-line tools, data processing, learning, and code that AI agents write.
