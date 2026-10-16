# Live AI benchmark

Can a model that has never seen Zamak write working Zamak programs from a one-page reference? This benchmark asks the same model to solve the same small command-line tasks in Zamak, Rust and Go, and records how often it succeeds and how many rounds it needs.

## Method

- **Tasks**: 8 small CLI programs in [tasks.json](tasks.json): integer sum with error reporting, word frequency, bracket matching, run-length encoding, CSV totals, prime counting, line/word reversal, and deduplication. Each task has 2 to 4 tests covering stdin, arguments, stdout and the exit status.
- **Loop**: the model gets the task and replies with a complete program. The harness builds it and runs every test. On a compile error or a failing test it sends back the first error, and the model replies with a corrected program. At most 5 rounds.
- **Prompts**: Rust and Go get one line naming the language. Zamak also gets [zamak-reference.md](zamak-reference.md) (about 11 KB), since no model has seen the language before.
- **Builds**: `zam build`, `rustc --edition 2021` on a single file (no Cargo), and `go build`. Builds run one at a time so concurrent model calls do not distort the timings.
- **Reference solutions**: [solutions/](solutions) solves every task in Zamak; `--verify-solutions` checks them, which keeps the tasks and expected outputs honest.

```sh
python bench/ai/live/run.py --verify-solutions
ZAM_LLM_BASE_URL=https://... ZAM_LLM_API_KEY=... python bench/ai/live/run.py --models <model> ...
```

Results go to the ignored `results/ai-live-<time>/`: every prompt round's source code, `trials.jsonl`, and `summary.md`.

## Results (2026-10-16)

Windows 10, zam 0.1.0 with MSVC, rustc 1.99.0, go 1.26.4. Temperature 0, OpenAI-compatible endpoint. The reference described the language as of that day, before the Python-style syntax was added.

| model | language | solved | first try | compile errors | build s / round (median) | tokens |
|---|---|---|---|---|---|---|
| claude-opus-5-5-code | Zamak | 8/8 | 5/8 | 3 | 0.58 | 128,563 |
| claude-opus-5-5-code | Rust | 8/8 | 8/8 | 0 | 0.37 | 13,138 |
| claude-opus-5-5-code | Go | 8/8 | 8/8 | 0 | 1.74 | 7,624 |
| deepseek-v4.1-flash | Zamak | 8/8 | 8/8 | 0 | 0.60 | 67,897 |
| deepseek-v4.1-flash | Rust | 8/8 | 7/8 | 1 | 0.38 | 20,770 |
| deepseek-v4.1-flash | Go | 8/8 | 8/8 | 0 | 1.75 | 12,734 |
| gemini-3.8-flash | Zamak | 8/8 | 8/8 | 0 | 0.60 | 92,310 |
| gemini-3.8-flash | Rust | 8/8 | 8/8 | 0 | 0.38 | 14,417 |
| gemini-3.8-flash | Go | 8/8 | 8/8 | 0 | 1.74 | 18,948 |

What this shows:

- Every task was solved in every language. Zamak passed 21 of 24 trials on the first try, Rust 23 and Go 24.
- Claude's three Zamak compile errors, each fixed in round 2: calling `string.codepoints` without `use std/string`, passing `env.arg(0)` without the `?` it needs, and writing `let entry = &entries[i]`, which Zamak did not allow then. Afterwards the missing-import message names the exact `use` line to add, and read-only borrows can be kept in locals. The benchmark has not been rerun since these changes or since the Python-style syntax was added.

## Results (2026-10-15)

Windows 10, zam 0.1.0 with MSVC, rustc 1.99.0, go 1.26.4. Temperature 0, OpenAI-compatible endpoint. Zamak uses indentation-based blocks and the logical operators `and`, `or`, `not`.

| model | language | solved | first try | compile errors | build s / round (median) | tokens |
|---|---|---|---|---|---|---|
| claude-opus-5-5-code | Zamak | 8/8 | 8/8 | 0 | 0.57 | 89,359 |
| claude-opus-5-5-code | Rust | 8/8 | 8/8 | 0 | 0.37 | 12,461 |
| claude-opus-5-5-code | Go | 8/8 | 8/8 | 0 | 1.73 | 9,754 |
| deepseek-v4.1-flash | Zamak | 8/8 | 8/8 | 0 | 0.54 | 50,703 |
| deepseek-v4.1-flash | Rust | 8/8 | 8/8 | 0 | 0.41 | 14,135 |
| deepseek-v4.1-flash | Go | 8/8 | 8/8 | 0 | 1.80 | 16,305 |
| gemini-3.8-flash | Zamak | 8/8 | 8/8 | 0 | 0.53 | 66,389 |
| gemini-3.8-flash | Rust | 8/8 | 8/8 | 0 | 0.35 | 12,475 |
| gemini-3.8-flash | Go | 8/8 | 8/8 | 0 | 1.70 | 14,637 |

What this shows:

- Reading only the reference, the three models passed all 24 Zamak trials on the first try with no compile errors. The previous run (2026-10-14, brace-delimited blocks and `&&`/`||`/`!`) passed 23 of 24 on the first try.
- An earlier run on 2026-10-15 accidentally sent an outdated reference that still listed `&& || !`. DeepSeek used them in 4 of 8 Zamak programs; the compiler rejected each one with ``expected `and` instead of `&&` ``, and all 4 passed in round 2. Models follow the reference closely, so it has to match the language.
- Rust and Go also passed all 24 trials on the first try. These tasks are too easy to separate the three languages by success rate.
- Zamak trials used 3 to 9 times more tokens. Most of it is the reference, now about 11 KB, sent with every request (median 3,846 prompt tokens per round versus about 130), plus longer replies (median 4,717 completion tokens versus 1,284 for Rust).
- For single-file programs, `rustc` built faster than `zam build` (0.37 s versus 0.54 s median); `go build` took about 1.74 s. Zamak's caching targets multi-module projects and repeated edits, which this benchmark does not exercise.
