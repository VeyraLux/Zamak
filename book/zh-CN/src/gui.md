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
| `ui.table(columns)` | 只读的文字表格 | `set_rows(rows)`、`selected()`、`set_selected(i)`；-1 表示没有选中 |

`ui.show(c)`、`ui.hide(c)`、`ui.set_enabled(c, on)` 对所有控件都能用。

表格的数据是若干行，每行是一个字符串列表，每列一个。`table.set_rows(rows)` 一次换掉所有行；某一行的格数不对时返回错误，表格保持原样：

```zamak,ignore
let table = ui.table(["Date", "Item", "Amount"])
table.set_rows([["2026-10-24", "milk", "3.50"], ["2026-10-25", "bus", "2.00"]])?
```

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
| `app.on_change(control, f)` | 用户改了输入框、文本框、复选框、滑块、数字框、下拉框或单选按钮，或者在表格里选中了另一行。 |
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

[`examples/apps/ledger`](https://github.com/VeyraLux/Zamak/tree/main/examples/apps/ledger) 是一个完整的桌面程序：表单、表格和汇总，数据存在 SQLite 里。

## 自己画控件

`ui.canvas()` 是一块空白区域，画什么由你决定。界面需要画它时，`app.on_draw(canvas, f)` 调用 `f` 并交给它一支 `Painter`；`app.on_mouse` 与 `app.on_key` 收到鼠标和键盘事件。状态变了就调用 `canvas.redraw()`，`on_draw` 会再执行一次。

`std/widgets` 是用这种方法画出来的一套控件：开关、按钮、滑块、复选框、进度条与进度环、折线图与柱状图、卡片、徽章，配浅色和深色两套主题。每个都是普通函数，按传进来的值画，状态留在你自己的结构体里：

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

- `Painter` 能画路径（`move_to`、`line_to`、`curve_to`、`arc`、`add_rect`、`add_round_rect`、`add_circle`），用纯色、线性渐变或径向渐变填满，或者描边。`fill_rect`、`fill_round_rect`、`fill_circle`、`line`、`text`、`text_in` 一步画好。`save`、`translate`、`rotate`、`scale`、`clip`、`restore` 改变画的位置和范围。
- 颜色用 `ui.rgb(0x4F6BED)` 或 `ui.rgba(hex, alpha)` 写；`color.mix(other, t)`、`lighter`、`darker`、`alpha` 由它得到新颜色。`ui.rect(x, y, w, h)` 的 `contains(x, y)` 用来判断点到了哪里。
- `Mouse` 有 `x()`、`y()`、`down()` 与 `up()`（1 是左键）、`clicks()`、`dragging()`。`Key` 有 `name()`（如 `"a"`、`"Left"`）、`released()`、`modifiers()`。
- 每个组件在 `std/widgets` 里只有几行代码。想换个样子，就把它复制进自己的程序改；想要新组件，就写一个在 `Painter` 上画的函数。`widgets.Theme` 的字段都是公开的，`widgets.dark()` 换一个 `accent` 就是你自己的主题。
- 只在状态真的变了时重画。画文字和形状都很快，但鼠标一动就重画的画布是在白白干活。

[`examples/ui/widgets.zm`](https://github.com/VeyraLux/Zamak/tree/main/examples/ui/widgets.zm) 把所有组件放在一块画布上，深色模式开关会让整套主题渐变过去。

## 隐藏控制台窗口、设置图标

在 Windows 上，程序启动时默认带一个控制台窗口，也就是那个黑色的终端。桌面程序不想要它，还想要自己的图标。把两项都写进 `zam.toml`：

```toml
[app]
console = false
icon = "assets/app.ico"
```

也可以写在命令里，单个文件时更方便：

```sh
zam build counter.zm --noconsole --icon assets/app.ico
```

- `console = false`（或 `--noconsole`）编译出的程序，双击运行时不会打开控制台窗口。它的 `println` 没有地方写，什么都不做；从终端启动并把输出写进文件时除外，例如 `app.exe > log.txt`。
- 图标必须是 Windows 的 `.ico` 文件，里面可以放好几种尺寸。别的格式 `zam` 直接拒绝，不去猜；PNG 等图片请先转换。`zam.toml` 里的路径相对项目目录，命令行里的相对你运行 `zam` 的目录。图标会显示在文件资源管理器、任务栏和 Alt+Tab 里。
- 没有控制台的程序遇到错误停下时，没有地方显示错误信息，会静默地以退出码 1 结束。请在带控制台的状态下测试，并用 `window.error(title, text)` 把问题告诉用户。
- 这两项只在 Windows 上生效。Linux 和 macOS 上 `zam build` 会输出一行 `NOTE` 并忽略它们，所以同一份 `zam.toml` 到处都能用。macOS 的图标属于应用包，Linux 的属于 `.desktop` 文件，`zam` 还不会生成这两样。
- `zam test` 始终保留控制台。用 MinGW 而不是 MSVC 设置图标时，要装 `windres`。

## 编译与运行

- 程序第一次用到 `std/ui` 时，`zam` 把自带的 libui-ng（MIT）编译进缓存，之后直接复用。
- Windows 上用 MSVC 编译，并嵌入启用新版控件样式的清单。
- Linux 上要先装 GTK 3 和 pkg-config，例如 `sudo apt install libgtk-3-dev pkg-config`。
- 只在主线程里用 `std/ui`。耗时的工作交给 `std/thread` 的线程，再用 `app.every` 读取结果。

## 小结

- `ui.window`、`ui.button` 等函数创建控件，`ui.column`、`ui.form` 等布局负责排列。
- 程序的状态是一个结构体，交给 `app.run`；回调以 `&mut` 收到它。
- `app.on_click`、`app.on_change`、`app.every`、`app.on_close` 把事件接到回调上。
