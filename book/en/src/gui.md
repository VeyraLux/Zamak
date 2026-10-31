# Windows and controls

`std/ui` builds desktop programs with the system's own controls: Win32 controls on Windows, AppKit on macOS, and GTK 3 on Linux. You arrange buttons, text boxes, lists, and other controls in a window, and you write functions that run when the user clicks, types, or picks something.

## A first window

This program counts clicks:

```zamak,ignore
use std/ui

struct Counter:
    count: i64
    label: ui.Label

fn clicked(app: &mut Counter):
    app.count += 1
    app.label.set_text(f"clicked {app.count} times")

fn main():
    let window = ui.window("Counter", 320, 160)
    let label = ui.label("not clicked yet")
    let button = ui.button("Click me")
    let column = ui.column()
    column.add(label)
    column.add(button)
    window.set_child(column)

    let app: ui.App[Counter] = ui.app()
    app.on_click(button, clicked)
    app.run(window, Counter(count: 0, label: label))
```

Here's what each part does:

1. `ui.window`, `ui.label`, and `ui.button` create controls. Each value is a handle to a control that the window owns, so you can copy a handle and keep it in your own struct. All controls close with their window.
2. `ui.column()` stacks controls from top to bottom, and `window.set_child` puts it in the window.
3. The program's state is a struct, `Counter`. It holds the click count and the label that shows it.
4. `ui.app()` creates the program. Write the state type in the annotation: `ui.App[Counter]`.
5. `app.on_click(button, clicked)` calls `clicked` whenever the button is clicked. The function receives the state as `&mut Counter`, so it can change the count and update the label.
6. `app.run(window, state)` shows the window and handles events until the user closes it. Then it returns the final state.

## Controls

| Function | Control | Read and change it |
| --- | --- | --- |
| `ui.label(text)` | Text | `text()`, `set_text(t)` |
| `ui.button(text)` | Button | `text()`, `set_text(t)`, `app.on_click` |
| `ui.entry(text="")`, `ui.password()` | One line of text | `text()`, `set_text(t)`, `set_read_only(on)` |
| `ui.text_area(wrap=true)` | Several lines of text | `text()`, `set_text(t)`, `append(t)` |
| `ui.checkbox(text, checked=false)` | Check box | `checked()`, `set_checked(on)` |
| `ui.choice(items)` | Drop-down list | `selected()`, `set_selected(i)`, `add(text)` |
| `ui.radio(items)` | Option buttons | `selected()`, `set_selected(i)`, `add(text)` |
| `ui.slider(low, high)`, `ui.spinbox(low, high)` | Whole number | `value()`, `set_value(n)` |
| `ui.progress()` | Progress bar, 0 to 100 | `value()`, `set_value(n)`; -1 shows unknown progress |
| `ui.separator(vertical=false)` | Line | |
| `ui.table(columns)` | Read-only table of text | `set_rows(rows)`, `selected()`, `set_selected(i)`; -1 means no row |

`ui.show(c)`, `ui.hide(c)`, and `ui.set_enabled(c, on)` work with any control.

A table takes a list of rows, each a list of strings, one per column. `table.set_rows(rows)` replaces all the rows at once, and returns an error, leaving the table as it was, if a row has the wrong number of cells:

```zamak,ignore
let table = ui.table(["Date", "Item", "Amount"])
table.set_rows([["2026-10-24", "milk", "3.50"], ["2026-10-25", "bus", "2.00"]])?
```

## Layout

| Function | Arranges its controls |
| --- | --- |
| `ui.column()` | From top to bottom: `add(c, stretchy=false)` |
| `ui.row()` | From left to right: `add(c, stretchy=false)` |
| `ui.form()` | As labeled rows: `add("Name", c)` |
| `ui.grid()` | In cells: `add(c, column, row, width=1, height=1, expand=false)` |
| `ui.group(title)` | In a titled frame: `set_child(c)` |
| `ui.tabs()` | On tabs: `add("Page", c)` |

A control added with `stretchy=true` takes the extra space, such as a text area that grows with the window.

## Events

| Method | Calls the handler when |
| --- | --- |
| `app.on_click(button, f)` | The button is clicked. |
| `app.on_change(control, f)` | The user changes an entry, text area, check box, slider, spin box, drop-down list, or option buttons, or selects another row of a table. |
| `app.every(milliseconds, f)` | The time passes, again and again. |
| `app.on_close(f)` | The window is about to close. |

Each handler takes `&mut` state, such as `fn saved(app: &mut Notes)`. Handlers are kept until the window closes, so a handler is a named function or an anonymous function that doesn't use local variables. Call `ui.quit()` to close the window from a handler.

Changing a control in code, such as `entry.set_text("x")`, doesn't call `on_change`. Only the user's changes do.

## Dialogs

A window can show standard dialogs:

```zamak,ignore
match app.window.open_file():
    Some(path): app.status.set_text(f"opened {path}")
    None: app.status.set_text("canceled")
```

- `window.message(title, text)` and `window.error(title, text)` show a message.
- `window.open_file()`, `window.save_file()`, and `window.open_folder()` return the chosen path, or `None` if the user cancels.

## Build your own controls

To make a control of your own, combine existing ones: write a function that builds a `column` or `grid` of controls and returns a struct that holds their handles. Every control has a `handle()` method, so a struct with one can be added to a layout like any built-in control.

[`examples/apps/ledger`](https://github.com/VeyraLux/Zamak/tree/main/examples/apps/ledger) is a complete desktop program: a form, a table, and a summary, with the data in SQLite.

## Draw your own controls

`ui.canvas()` is an empty area that you draw yourself. `app.on_draw(canvas, f)` calls `f` with a `Painter` whenever the area needs drawing, and `app.on_mouse` and `app.on_key` deliver mouse and keyboard events. When your state changes, call `canvas.redraw()`, and `on_draw` runs again.

`std/widgets` is a set of controls drawn this way: switches, buttons, sliders, check boxes, progress bars and rings, line and bar charts, cards, and badges, in a light and a dark theme. Each one is an ordinary function that draws from the values you pass in, so the state stays in your own struct:

```zamak,ignore
use std/ui
use std/widgets

struct State:
    canvas: ui.Canvas
    on: bool
    hover: bool

fn switch_box() -> ui.Rect:
    ui.rect(20.0, 20.0, 44.0, 24.0)

fn draw(s: &mut State, p: &ui.Painter):
    let t = widgets.light()
    widgets.draw_background(p, t)
    widgets.draw_switch(p, switch_box(), s.on, t, hover=s.hover)
    p.text(76.0, 23.0, if s.on: "On" else: "Off", t.text)

fn mouse(s: &mut State, m: &ui.Mouse):
    let over = switch_box().contains(m.x(), m.y())
    let clicked = over and m.down() == 1
    if clicked:
        s.on = not s.on
    if clicked or over != s.hover:
        s.hover = over
        s.canvas.redraw()

fn main():
    let window = ui.window("Switch", 240, 80)
    let canvas = ui.canvas()
    window.set_child(canvas)
    let app: ui.App[State] = ui.app()
    app.on_draw(canvas, draw)
    app.on_mouse(canvas, mouse)
    app.run(window, State(canvas: canvas, on: false, hover: false))
```

- The `Painter` draws paths (`move_to`, `line_to`, `curve_to`, `arc`, `add_rect`, `add_round_rect`, `add_circle`) and fills them with a color, a linear gradient, or a radial gradient, or strokes them. `fill_rect`, `fill_round_rect`, `fill_circle`, `line`, `text`, and `text_in` do it in one call. `save`, `translate`, `rotate`, `scale`, `clip`, and `restore` change where you draw.
- Colors come from `ui.rgb(0x4F6BED)` or `ui.rgba(hex, alpha)`; `color.mix(other, t)`, `lighter`, `darker`, and `alpha` derive new ones. `ui.rect(x, y, w, h)` has `contains(x, y)` for hit testing.
- A `Mouse` has `x()`, `y()`, `down()` and `up()` (1 is the left button), `clicks()`, and `dragging()`. A `Key` has `name()`, such as `"a"` or `"Left"`, `released()`, and `modifiers()`.
- Every widget is a few lines of code in `std/widgets`. To change how one looks, copy it into your program and edit it; to add a new one, write a function that draws on a `Painter`. Fields of `widgets.Theme` are public, so `widgets.dark()` with a different `accent` is your own theme.
- Redraw only when something changed. Drawing text and shapes is fast, but a canvas that redraws on every mouse move does work for nothing.

[`examples/ui/widgets.zm`](https://github.com/VeyraLux/Zamak/tree/main/examples/ui/widgets.zm) puts every widget on one canvas, with a dark-mode switch that fades the whole theme.

## Hide the console window and set an icon

On Windows, a program starts with a console window, the black terminal, unless you ask for none. A desktop program wants no console and an icon of its own. Put both in `zam.toml`:

```toml
[app]
console = false
icon = "assets/app.ico"
```

Or pass them to the command, which is handy for a single file:

```sh
zam build counter.zm --noconsole --icon assets/app.ico
```

- `console = false`, or `--noconsole`, builds a program that opens no console window when you double-click it. Its `println` has nowhere to write and does nothing, unless you start it from a terminal with the output sent to a file, as in `app.exe > log.txt`.
- The icon must be a Windows `.ico` file, which can hold several sizes. `zam` rejects any other format instead of guessing; convert PNG and other images first. In `zam.toml` the path is relative to the project folder, and on the command line it's relative to where you run `zam`. The icon shows in File Explorer, the taskbar, and Alt+Tab.
- A program without a console that stops on an error has nowhere to show the message, so it exits with code 1 and says nothing. Test with the console on, and report problems to the user with `window.error(title, text)`.
- Both settings apply only on Windows. On Linux and macOS, `zam build` prints a `NOTE` line and ignores them, so one `zam.toml` works everywhere. On macOS, an icon belongs to an app bundle, and on Linux to a `.desktop` file; `zam` doesn't create either yet.
- `zam test` always keeps the console. To set an icon with MinGW instead of MSVC, install `windres`.

## Building and running

- The first build of a program that uses `std/ui` compiles libui-ng (MIT), which `zam` carries, into the cache. Later builds reuse it.
- On Windows, `zam` builds with MSVC and embeds the manifest that turns on the current Windows control styles.
- On Linux, install GTK 3 and pkg-config first, for example `sudo apt install libgtk-3-dev pkg-config`.
- Use `std/ui` from the main thread only. To do slow work, start a thread with `std/thread`, and read its results with `app.every`.

## Summary

- `ui.window`, `ui.button`, and the other functions create controls; layouts such as `ui.column` and `ui.form` arrange them.
- The program's state is a struct that you pass to `app.run`; handlers receive it as `&mut`.
- `app.on_click`, `app.on_change`, `app.every`, and `app.on_close` connect events to handlers.
