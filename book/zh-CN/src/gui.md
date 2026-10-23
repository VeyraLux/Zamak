# 窗口程序

`std/ui` 用系统自己的控件写桌面程序：Windows 上是 Win32 控件，macOS 上是 AppKit，Linux 上是 GTK 3。你把按钮、文本框、列表等控件排进窗口，再写好用户点击、输入、选择时要运行的函数。

## 第一个窗口

下面的程序统计点击次数：

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

各部分的作用：

1. `ui.window`、`ui.label`、`ui.button` 创建控件。得到的值是指向控件的句柄，控件归窗口所有，所以句柄可以复制、可以存进你自己的结构体。窗口关闭时，控件一起销毁。
2. `ui.column()` 把控件从上到下排列，`window.set_child` 把它放进窗口。
3. 程序的状态是一个结构体 `Counter`，里面有点击次数，以及显示它的标签。
4. `ui.app()` 创建程序。状态的类型写在标注里：`ui.App[Counter]`。
5. `app.on_click(button, clicked)`：每次点按钮都调用 `clicked`。这个函数收到 `&mut Counter`，可以改次数、更新标签。
6. `app.run(window, state)` 显示窗口并处理事件，直到用户关闭窗口，然后返回最终的状态。

## 控件

| 函数 | 控件 | 读取与修改 |
| --- | --- | --- |
| `ui.label(text)` | 文字 | `text()`、`set_text(t)` |
| `ui.button(text)` | 按钮 | `text()`、`set_text(t)`、`app.on_click` |
| `ui.entry(text="")`、`ui.password()` | 单行输入框 | `text()`、`set_text(t)`、`set_read_only(on)` |
| `ui.text_area(wrap=true)` | 多行文本框 | `text()`、`set_text(t)`、`append(t)` |
| `ui.checkbox(text, checked=false)` | 复选框 | `checked()`、`set_checked(on)` |
| `ui.choice(items)` | 下拉框 | `selected()`、`set_selected(i)`、`add(text)` |
| `ui.radio(items)` | 单选按钮 | `selected()`、`set_selected(i)`、`add(text)` |
| `ui.slider(low, high)`、`ui.spinbox(low, high)` | 整数 | `value()`、`set_value(n)` |
| `ui.progress()` | 进度条，0 到 100 | `value()`、`set_value(n)`；-1 表示进度不确定 |
| `ui.separator(vertical=false)` | 分隔线 | |

`ui.show(c)`、`ui.hide(c)`、`ui.set_enabled(c, on)` 对所有控件都能用。

## 布局

| 函数 | 怎样排列控件 |
| --- | --- |
| `ui.column()` | 从上到下：`add(c, stretchy=false)` |
| `ui.row()` | 从左到右：`add(c, stretchy=false)` |
| `ui.form()` | 一行一项，左边是说明：`add("Name", c)` |
| `ui.grid()` | 按格子：`add(c, column, row, width=1, height=1, expand=false)` |
| `ui.group(title)` | 放在带标题的框里：`set_child(c)` |
| `ui.tabs()` | 放在选项卡里：`add("Page", c)` |

用 `stretchy=true` 加进去的控件会分走多余的空间，比如随窗口变大的文本框。

## 事件

| 方法 | 何时调用回调 |
| --- | --- |
| `app.on_click(button, f)` | 点了按钮。 |
| `app.on_change(control, f)` | 用户改了输入框、文本框、复选框、滑块、数字框、下拉框或单选按钮。 |
| `app.every(milliseconds, f)` | 每隔这么多毫秒，反复调用。 |
| `app.on_close(f)` | 窗口即将关闭。 |

回调都收到 `&mut` 状态，例如 `fn saved(app: &mut Notes)`。回调会一直保存到窗口关闭，所以要用具名函数，或者不用局部变量的匿名函数。在回调里调用 `ui.quit()` 可以关掉窗口。

用代码修改控件（比如 `entry.set_text("x")`）不会触发 `on_change`，只有用户的修改才会。

## 对话框

窗口可以弹出标准对话框：

```zamak,ignore
match app.window.open_file():
    Some(path): app.status.set_text(f"opened {path}")
    None: app.status.set_text("canceled")
```

- `window.message(title, text)` 与 `window.error(title, text)` 弹出消息。
- `window.open_file()`、`window.save_file()`、`window.open_folder()` 返回选中的路径，用户取消时是 `None`。

## 自己做控件

要做自己的控件，就把现有控件组合起来：写一个函数，把几个控件排进 `column` 或 `grid`，返回一个存着它们句柄的结构体。所有控件都有 `handle()` 方法，所以有这个方法的结构体也能像内置控件一样加进布局。

## 编译与运行

- 程序第一次用到 `std/ui` 时，`zam` 把自带的 libui-ng（MIT）编译进缓存，之后直接复用。
- Windows 上用 MSVC 编译，并嵌入启用新版控件样式的清单。
- Linux 上要先装 GTK 3 和 pkg-config，例如 `sudo apt install libgtk-3-dev pkg-config`。
- 只在主线程里用 `std/ui`。耗时的工作交给 `std/thread` 的线程，再用 `app.every` 读取结果。

## 小结

- `ui.window`、`ui.button` 等函数创建控件，`ui.column`、`ui.form` 等布局负责排列。
- 程序的状态是一个结构体，交给 `app.run`；回调以 `&mut` 收到它。
- `app.on_click`、`app.on_change`、`app.every`、`app.on_close` 把事件接到回调上。
