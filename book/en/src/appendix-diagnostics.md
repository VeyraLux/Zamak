# Appendix D: Diagnostic codes

Every error that `zam check` and `zam build` report has a code. A code keeps its meaning across versions, so tools and AI agents can rely on it. The message text can improve over time; the code doesn't change.

| Code | Category | Meaning |
| --- | --- | --- |
| `ZM100` | syntax | A character that isn't allowed in Zamak source. |
| `ZM101` | syntax | An unsupported escape sequence in a string. |
| `ZM110` | syntax | A syntax error, including indentation problems and `=` used where `==` belongs. |
| `ZM200` | type | Type mismatch. |
| `ZM201` | name | An undefined variable, function, struct, type, or enum variant. |
| `ZM202` | name | An undefined struct field. |
| `ZM203` | name | A missing or ambiguous import. Add the `use` line that the help suggests. |
| `ZM204` | name | A duplicate declaration: import, field, function, or variable. |
| `ZM301` | ownership | A value used after it was moved. |
| `ZM302` | ownership | An assignment to a value that isn't `let mut`. |
| `ZM406` | bounds | A fixed-size array index that's out of range. |
| `ZM421` | ownership | A borrow conflict, such as changing a list while a loop reads it. |
| `ZM422` | ownership | A borrow that would outlive its value, such as returning a reference. |
| `ZM423` | ownership | A borrow of the wrong kind or type. |
| `ZM424` | ownership | A mutable borrow of a value that isn't `let mut`. |
| `ZM425` | ownership | A call that needs an explicit `&` or `&mut`. |
| `ZM500` | error_handling | A failing call that isn't handled with `?` or `catch`. |
| `ZM509` | error_handling | An error type that isn't `string`, a struct, or an enum. |
| `ZM510` | control_flow | A function that doesn't return a value on every path. |
| `ZM511` | control_flow | A `match` that doesn't cover every variant, or an arm after `_`. |
| `ZM600` | arithmetic | An integer literal or constant expression that overflows. |
| `ZM900` | unsupported | A feature that Zamak doesn't support yet. |
| `ZM999` | general | An error that isn't classified yet. Report it with a small example. |

## Run-time errors

These messages come from a running program, not from the compiler. The program prints the message to standard error and exits with status 1.

| Message | Cause |
| --- | --- |
| `zam: integer arithmetic error` | An integer operation overflowed, or `i64` minimum was divided by -1. |
| `zam: division by zero` | Division or remainder by zero. |
| `zam: list index out of bounds` | An index outside the list. |
| `zam: key not found` | `m[k]` with a key that isn't in the map. |
| `zam: pop from empty list` | `pop` on an empty list. |
| `assertion failed: ...` | An `assert` whose condition was false. |
