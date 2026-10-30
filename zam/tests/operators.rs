mod common;

use common::Case;
use std::process::Output;

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn rejected(output: &Output, message: &str, source: &str) {
    assert!(!output.status.success(), "accepted {source}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(message), "{source}: {stderr}");
}

#[test]
fn structs_define_operators_as_python_does() {
    let case = Case::new("operators", "example");
    let output = case.run(include_str!("../../examples/structs/operators.zm"));
    assert_eq!(
        stdout(&output),
        "Vec2(x: 4.0, y: 7.0)\nVec2(x: 2.0, y: 3.0)\nVec2(x: 8.0, y: 14.0)\nVec2(x: -1.0, y: -2.0)\nVec2(x: 4.0, y: 7.0)\n4150\ntrue\ntrue\ntrue\n¥41.50\ntaxi: ¥23.00\n[¥18.50, ¥23.00]\n"
    );
}

#[test]
fn structs_without_operator_methods_say_what_to_define() {
    let case = Case::new("operators", "rejected");
    let point = "struct P:\n    x: i64\n";
    for (body, message) in [
        (
            "    println(P(x: 1) - P(x: 2))",
            "- needs numbers; define fn __sub__(&self, other: ...) in the struct to use -",
        ),
        (
            "    println(P(x: 1) < P(x: 2))",
            "define fn __lt__(&self, other: ...) in the struct to use <",
        ),
        // 左边是数字时不找方法（Python 的 __rsub__ 不支持）：照常报类型不对。
        ("    let a = P(x: 1)\n    let b = 5 - a", "type mismatch"),
    ] {
        let source = format!("{point}fn main():\n{body}\n");
        rejected(&case.run(&source), message, &source);
    }
    // 方法的参数类型照常检查：给 &P 传了 5。
    let source = "struct P:\n    x: i64\n\n    fn __add__(&self, other: &P) -> P:\n        P(x: self.x + other.x)\nfn main():\n    println(P(x: 1) + 5)\n";
    rejected(
        &case.run(source),
        "this value is i64, but P is expected",
        source,
    );
    // 没定义 __eq__ 的结构体照旧逐项比较。
    let output = case.run("struct P:\n    x: i64\nfn main():\n    println(P(x: 1) == P(x: 1))\n    println(P(x: 1) != P(x: 1))\n");
    assert_eq!(stdout(&output), "true\nfalse\n");
}
