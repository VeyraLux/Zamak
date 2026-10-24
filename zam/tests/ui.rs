mod common;

use common::Case;

/// 界面要有图形环境：没有显示器、Linux 上没装 GTK 3 时说明原因并跳过（返回 None）。
fn run_ui(case: &Case, source: &str) -> Option<String> {
    let output = case.run(source);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("cannot start the user interface") || stderr.contains("std/ui needs GTK 3") {
        eprintln!("skipping: no user interface here: {}", stderr.trim());
        return None;
    }
    assert!(output.status.success(), "{stderr}");
    Some(String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"))
}

#[test]
fn tables_show_rows_and_track_the_selection() {
    // 定时器换几次数据（变多、变少）、选中、取消选中；格数不对的行返回错误，表格不变。
    let case = Case::new("ui", "table");
    let Some(stdout) = run_ui(
        &case,
        r#"use std/ui

struct State:
    ticks: i64
    table: ui.Table

fn tick(s: &mut State):
    s.ticks += 1
    if s.ticks == 1:
        s.table.set_rows([["2026-10-24", "milk", "3.50"], ["2026-10-24", "rent", "900.00"], ["2026-10-25", "bus", "2.00"]]) catch e: println(e)
        println(s.table.selected())
        s.table.set_selected(1)
        println(s.table.selected())
    if s.ticks == 2:
        s.table.set_rows([["2026-10-26", "tea", "4.00"]]) catch e: println(e)
        s.table.set_rows([["a", "b"]]) catch e: println(e)
        s.table.set_selected(0)
        println(s.table.selected())
        s.table.set_selected(-1)
        println(s.table.selected())
    if s.ticks == 3:
        s.table.set_rows([["1", "2", "3"], ["4", "5", "6"], ["7", "8", "9"], ["10", "11", "12"]]) catch e: println(e)
        s.table.set_selected(3)
        println(s.table.selected())
        ui.quit()

fn main():
    let window = ui.window("Table", 400, 300)
    let table = ui.table(["Date", "Item", "Amount"])
    window.set_child(table)
    let app: ui.App[State] = ui.app()
    app.every(50, tick)
    app.on_change(table, tick)
    app.run(window, State(ticks: 0, table: table))
"#,
    ) else {
        return;
    };
    assert_eq!(
        stdout,
        "-1\n1\nrow 0 has 2 cells, but the table has 3 columns\n0\n-1\n3\n"
    );
}

#[test]
fn canvases_draw_every_widget() {
    // 画布上画出所有默认组件：画图回调真的被调用、画笔量得出文字宽度、颜色与区域的计算对。
    let case = Case::new("ui", "canvas");
    let Some(stdout) = run_ui(
        &case,
        r#"use std/ui
use std/widgets

struct S:
    canvas: ui.Canvas
    draws: i64
    ticks: i64
    width: f64

fn draw(s: &mut S, p: &ui.Painter):
    s.draws += 1
    s.width = p.width()
    let t = if s.draws % 2 == 0: widgets.dark() else: widgets.light()
    widgets.draw_background(p, t)
    let body = widgets.draw_card(p, ui.rect(10.0, 10.0, 380.0, 280.0), "Card", t)
    widgets.draw_switch_row(p, ui.rect(body.x, body.y, body.w, 30.0), "Switch", true, t, hover=true)
    widgets.draw_button(p, ui.rect(body.x, body.y + 40.0, 90.0, 30.0), "Go", t, pressed=true)
    widgets.draw_button(p, ui.rect(body.x + 100.0, body.y + 40.0, 90.0, 30.0), "Back", t, primary=false)
    widgets.draw_checkbox(p, ui.rect(body.x + 200.0, body.y + 45.0, 18.0, 18.0), true, t)
    widgets.draw_slider(p, ui.rect(body.x, body.y + 80.0, 200.0, 24.0), 0.4, t, hover=true)
    widgets.draw_progress(p, ui.rect(body.x, body.y + 115.0, 200.0, 8.0), 0.7, t)
    widgets.draw_ring(p, body.x + 290.0, body.y + 60.0, 40.0, 0.25, t, label="done")
    let values: [f64] = [3.0, 1.0, 4.0, 1.5, 5.0]
    let labels: [string] = ["a", "b"]
    widgets.draw_line_chart(p, ui.rect(body.x, body.y + 130.0, 170.0, 90.0), values, t)
    widgets.draw_bar_chart(p, ui.rect(body.x + 180.0, body.y + 130.0, 170.0, 90.0), values, labels, t)
    let empty: [f64] = []
    widgets.draw_line_chart(p, ui.rect(0.0, 0.0, 10.0, 10.0), empty, t)
    widgets.draw_badge(p, body.x, body.y + 225.0, "badge", t.good)
    p.save()
    p.translate(200.0, 150.0)
    p.rotate(0.5)
    p.begin()
    p.move_to(0.0, 0.0)
    p.curve_to(10.0, 20.0, 30.0, 20.0, 40.0, 0.0)
    p.close()
    p.fill_radial(20.0, 10.0, 20.0, t.accent, t.accent.alpha(0.0))
    p.clip()
    p.restore()

fn tick(s: &mut S):
    s.ticks += 1
    s.canvas.redraw()
    if s.ticks == 6:
        ui.quit()

fn main():
    let window = ui.window("Canvas", 400, 300)
    let canvas = ui.canvas()
    window.set_child(canvas)
    let app: ui.App[S] = ui.app()
    app.on_draw(canvas, draw)
    app.every(50, tick)
    let s = app.run(window, S(canvas: canvas, draws: 0, ticks: 0, width: 0.0))
    println(s.draws >= 2)
    println(s.width > 100.0)
    println(ui.text_width("wide text") > ui.text_width("w"))
    let c = ui.rgb(0x336699).mix(ui.rgb(0xFFFFFF), 0.5)
    println(f"{c.r:.3} {c.g:.3} {c.b:.3} {c.a}")
    let r = ui.rect(10.0, 10.0, 100.0, 50.0)
    println(r.contains(10.0, 10.0))
    println(r.contains(110.0, 30.0))
    println(r.inset(5.0).w)
    println(widgets.slider_value(109.0, ui.rect(0.0, 0.0, 218.0, 24.0)))
"#,
    ) else {
        return;
    };
    assert_eq!(
        stdout,
        "true\ntrue\ntrue\n0.600 0.700 0.800 1.0\ntrue\nfalse\n90.0\n0.5\n"
    );
}

#[test]
fn ui_examples_type_check() {
    // 示例要人去点，测试只检查它们能通过 zam check（不需要图形环境）。
    for example in ["counter", "notes", "widgets"] {
        let path = format!("../examples/ui/{example}.zm");
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
            .args(["check", &path])
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{example}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn ui_callbacks_change_state_and_controls() {
    // 程序自己用定时器操作控件，几轮之后 ui.quit()；run 返回最终状态。
    let case = Case::new("ui", "callbacks");
    let Some(stdout) = run_ui(
        &case,
        r#"use std/ui

struct State:
    ticks: i64
    window: ui.Window
    label: ui.Label
    entry: ui.Entry
    slider: ui.Slider
    check: ui.Checkbox
    choice: ui.Choice
    area: ui.TextArea

fn tick(s: &mut State):
    s.ticks += 1
    s.label.set_text(f"tick {s.ticks}")
    s.slider.set_value(s.ticks * 10)
    s.area.append(f"{s.ticks};")
    if s.ticks == 3:
        println(s.label.text())
        println(s.slider.value())
        s.entry.set_text("typed")
        println(s.entry.text())
        s.check.set_checked(true)
        println(s.check.checked())
        println(s.choice.selected())
        s.choice.set_selected(2)
        println(s.choice.selected())
        println(s.area.text())
        s.window.set_title("done")
        println(s.window.title())
        ui.quit()

fn main():
    let window = ui.window("Test", 320, 240)
    let label = ui.label("start")
    let entry = ui.entry()
    let slider = ui.slider(0, 100)
    let check = ui.checkbox("check")
    let choice = ui.choice(["a", "b", "c"])
    let area = ui.text_area()
    let form = ui.form()
    form.add("Entry", entry)
    form.add("Slider", slider)
    let column = ui.column()
    column.add(label)
    column.add(form)
    column.add(check)
    column.add(choice)
    column.add(area, stretchy=true)
    window.set_child(column)
    let app: ui.App[State] = ui.app()
    app.every(50, tick)
    let state = app.run(window, State(ticks: 0, window: window, label: label, entry: entry, slider: slider, check: check, choice: choice, area: area))
    println(f"run returned after {state.ticks} ticks")
"#,
    ) else {
        return;
    };
    assert_eq!(
        stdout,
        "tick 3\n30\ntyped\ntrue\n0\n2\n1;2;3;\ndone\nrun returned after 3 ticks\n"
    );
}

#[test]
fn ui_handlers_cannot_capture_locals() {
    // 回调会被保存下来，只收不捕获局部变量的函数：用了局部变量的匿名函数在编译时报错。
    let case = Case::new("ui", "capture");
    let output = case.run(
        "use std/ui\n\nstruct S:\n    n: i64\n\nfn main():\n    let step = 2\n    let window = ui.window(\"T\", 100, 100)\n    let button = ui.button(\"b\")\n    let app: ui.App[S] = ui.app()\n    app.on_click(button, fn(s: &mut S): println(step))\n",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(stderr.contains("borrow escape"), "{stderr}");
}
