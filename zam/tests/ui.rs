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
fn ui_examples_type_check() {
    // 示例要人去点，测试只检查它们能通过 zam check（不需要图形环境）。
    for example in ["counter", "notes"] {
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
