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

`ui.show(c)`, `ui.hide(c)`, and `ui.set_enabled(c, on)` work with any control.

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
| `app.on_change(control, f)` | The user changes an entry, text area, check box, slider, spin box, drop-down list, or option buttons. |
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

## Building and running

- The first build of a program that uses `std/ui` compiles libui-ng (MIT), which `zam` carries, into the cache. Later builds reuse it.
- On Windows, `zam` builds with MSVC and embeds the manifest that turns on the current Windows control styles.
- On Linux, install GTK 3 and pkg-config first, for example `sudo apt install libgtk-3-dev pkg-config`.
- Use `std/ui` from the main thread only. To do slow work, start a thread with `std/thread`, and read its results with `app.every`.

## Summary

- `ui.window`, `ui.button`, and the other functions create controls; layouts such as `ui.column` and `ui.form` arrange them.
- The program's state is a struct that you pass to `app.run`; handlers receive it as `&mut`.
- `app.on_click`, `app.on_change`, `app.every`, and `app.on_close` connect events to handlers.
