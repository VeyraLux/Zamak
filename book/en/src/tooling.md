# The zam command

Everything you do with Zamak goes through one command, `zam`. This chapter lists its subcommands, explains how diagnostics and caching work, and describes the environment variables that change its behavior.

## Subcommands

Most subcommands take a project folder or a single `.zm` file. Without one, they use the current folder.

| Command | What it does |
| --- | --- |
| `zam new NAME` | Creates a project. Add `--template ai` to include files for AI coding agents. |
| `zam check` | Checks types and ownership without compiling. Doesn't need a C compiler. |
| `zam build` | Compiles to a standalone executable in `build/`. |
| `zam run [-- ARGS]` | Builds and runs, passing `ARGS` to the program. |
| `zam test [WORD]` | Runs the `test_...` functions, optionally only those whose names contain `WORD`. |
| `zam fmt` | Normalizes indentation and blank lines. |
| `zam add`, `zam remove`, `zam update` | Manage dependencies. |
| `zam search WORD`, `zam publish` | Find and publish packages. |
| `zam --version` | Prints the version. |

`zam` writes its own progress lines to standard error. Standard output carries only your program's output, or the JSON report of `--json`.

## Read a diagnostic

When `zam check` finds a problem, it shows a stable error code, the location, the source line, and a suggested fix:

```zamak,fails
fn main():
    let name = "Ada"
    let other = name
    println(other)
    println(name)
```

```text
error[ZM301]: use after move: name
 --> main.zm:5:5
  |
5 |     println(name)
  |     ^^^^^^^^^^^^^
  = help: borrow the value or move it only after the last use
```

- The code, such as `ZM301`, never changes meaning, so you can search for it. [Appendix D](appendix-diagnostics.md) lists the codes.
- Syntax errors stop the check at the first one. Type and ownership errors are reported once per function, so you can fix several functions in one pass.
- If you write something the Python way and Zamak spells it differently, the help line shows the Zamak spelling.
- On a system set to Chinese, `zam` shows diagnostics in Chinese. To choose the language yourself, set `ZAMAK_LANG` to `zh` or `en`.

## Diagnostics as JSON

Editors, scripts, and AI agents can ask for one line of JSON instead:

```sh
zam check --json main.zm
```

```json
{"command":"check","ok":false,"diagnostics":[{"file":"main.zm","line":5,"column":5,"function":"main:main","code":"ZM301","category":"ownership","message":"use after move: name","suggestion":"borrow the value or move it only after the last use"}]}
```

`zam build --json` reports build errors the same way. The JSON is always in English, whatever the system language, so tools can rely on the text.

## Format code

`zam fmt` rewrites files in place: it turns tabs into spaces, indents each block by 4 spaces, and puts one blank line between top-level declarations. It only changes leading whitespace and blank lines, so it never touches comments, strings, or the spacing inside a line. It prints `FORMAT path` for each file it changes and refuses to touch a file that doesn't parse.

## How the cache works

`zam` caches every compiled module by the content of its source and of the modules it depends on. The cache lives in `~/.zam/cache`, and it's shared by all projects and all copies of a project, such as Git worktrees. As a result:

- Building the same code twice, or in two folders, compiles it once.
- Changing one module rebuilds only that module and the modules that use its public items.
- `zam check` on a 5,000-line project takes less than 0.1 seconds.

You can delete `~/.zam/cache` at any time; `zam` rebuilds what it needs.

## Environment variables

| Variable | Effect |
| --- | --- |
| `CC` | The C compiler to use, such as `gcc` or `clang`. By default, `zam` uses MSVC on Windows when it's installed, and `cc` elsewhere. |
| `ZAMAK_CACHE_DIR` | Where to keep the build cache, instead of `~/.zam/cache`. |
| `ZAMAK_JOBS` | How many modules to compile in parallel. The default is the number of CPU cores, at most 4. |
| `ZAMAK_INDEX` | The package index to use, as a Git URL or a local folder. |
| `ZAMAK_DUMP_C` | A folder where `zam` writes the generated C code of each module. Useful for studying performance. |
| `ZAMAK_LANG` | The language of diagnostics in the terminal: `zh` for Chinese or `en` for English. By default, `zam` follows the system language. |

## Summary

- `zam check`, `build`, `run`, `test`, and `fmt` cover the edit loop; `add`, `remove`, `update`, `search`, and `publish` manage packages.
- Every diagnostic has a stable code and usually a fix; `--json` gives the same information to tools.
- The content-based cache makes repeated and incremental builds fast.
