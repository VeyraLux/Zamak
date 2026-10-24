# ledger

`ledger` is a desktop ledger. You add income and expenses in a form, see them in a table by month, delete one, and export them to CSV. It stores everything in a SQLite database: `~/.zamak-ledger.db`, or the file that the `LEDGER_DB` environment variable names.

```bash
zam run examples/apps/ledger
```

Amounts are kept in whole cents, never as floating-point numbers, so totals are exact. An amount with more than two decimal places, such as `1.234`, is an error rather than a rounded value.

| Control | What it does |
| --- | --- |
| Date, Description, Amount, Kind, Category | The item to add. The date is `YYYY-MM-DD`, today by default. |
| Add | Adds the item and refreshes the table. |
| Month | Shows one month (`YYYY-MM`), or every month when it's empty. |
| Delete selected | Deletes the selected row. |
| Export CSV… | Saves the rows shown as a CSV file that Excel can open. |

The code is in three modules:

- `src/money.zm` reads and writes amounts in cents.
- `src/books.zm` stores, lists, and totals the items, and writes CSV. It doesn't use the window, so `zam test examples/apps/ledger` tests it without a display.
- `src/main.zm` builds the window with `std/ui`.

When `LEDGER_SMOKE=1` is set, the program fills in the form itself, adds one item, prints the totals, and exits. The automated tests use this to run the window without anyone clicking.
