# Work with AI coding agents

Zamak is designed to be written by AI coding agents as well as by people. The language is small enough to describe on one page, its diagnostics are stable and machine-readable, and `zam` can set up a project so that your agent knows Zamak from the first prompt.

## Start a project for your agent

```sh
zam new myapp --template ai
```

Besides the usual `zam.toml` and `src/main.zm`, this creates:

| File | Read by |
| --- | --- |
| `.claude/skills/zamak/SKILL.md` | Claude Code, which loads it as a skill whenever the task involves Zamak. |
| `AGENTS.md` | Codex, Cursor, and other agents that read `AGENTS.md`. |
| `README.md` | People, with the commands to build and run the project. |

`SKILL.md` and `AGENTS.md` contain the complete language reference and the recommended working loop. The text is built into `zam`, so it always matches the version that created the project.

## The working loop

The files tell the agent to work in small, checked steps:

1. Run `zam check --json .` and read the diagnostics.
2. Make the smallest change that fixes a reported error.
3. Run `zam fmt .` and check again.
4. When the check passes, run `zam build .`, `zam run .`, and `zam test .`.

This loop suits agents because `zam check` takes a fraction of a second and needs no C compiler, and because each diagnostic says exactly what to change:

```json
{"file":"main.zm","line":5,"column":5,"function":"main:main","code":"ZM301","category":"ownership","message":"use after move: name","suggestion":"borrow the value or move it only after the last use"}
```

- `code` identifies the kind of error and never changes meaning.
- `category` groups codes, such as `syntax`, `type`, or `ownership`.
- `suggestion` is a concrete fix, often naming the Zamak spelling when the code used a Python one.

## Why agents write Zamak well

- **One page is enough.** In a benchmark with three models and eight command-line tasks, each model read only the one-page reference. The models solved all 24 Zamak tasks, 21 of them on the first try. The method and data are in the repository under [bench/ai/live](https://github.com/VeyraLux/Zamak/tree/main/bench/ai/live).
- **Familiar syntax.** Agents have read a lot of Python, and Zamak keeps Python's names and rules wherever it can.
- **Mistakes surface early.** Type and ownership errors appear in `zam check`, not at run time, and integer overflow and out-of-bounds indexes stop with a clear message instead of producing wrong results that an agent might not notice.
- **Errors per function.** Because `zam` reports type errors once per function, an agent can fix several of them in one edit.

## Add Zamak to an existing project

To give an agent Zamak instructions in a project that already exists, create a throwaway project with `--template ai` and copy `AGENTS.md` or `.claude/skills/zamak/` into your project.

## Coming next

A `zam mcp` command is planned. It will expose checking, building, running, and the language reference to agents through the Model Context Protocol (MCP), from the same source as the skill file.

## Summary

- `zam new NAME --template ai` sets up `SKILL.md` for Claude Code and `AGENTS.md` for other agents.
- Agents work in a loop of `zam check --json`, a small fix, `zam fmt`, and a build.
- Stable codes, categories, and suggestions make diagnostics easy for agents to act on.
