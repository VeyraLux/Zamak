# todo

`todo` keeps a task list on the command line. It stores the tasks as JSON in `~/.zamak-todo.json`, or in the file that the `TODO_FILE` environment variable names.

```bash
zam run examples/apps/todo -- add write the report --due 2026-11-01 --tag work
zam run examples/apps/todo -- add buy milk --tag home
zam run examples/apps/todo -- list
zam run examples/apps/todo -- done 2
```

```text
[ ]   1  write the report  (2026-11-01, due in 8d)  #work
[ ]   2  buy milk  #home
```

Commands:

| Command | What it does |
| --- | --- |
| `add TEXT [--due YYYY-MM-DD] [--tag TAG]...` | Adds a task. |
| `list [--all] [--tag TAG]` | Lists open tasks, or all of them, optionally with one tag. |
| `done ID` | Marks a task done. |
| `edit ID TEXT` | Changes a task's text. |
| `remove ID` | Deletes a task. |

The program uses `std/datetime` for due dates, `std/json` and `std/fs` to store the tasks, and `std/env` for its arguments and settings.
