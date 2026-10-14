mod common;

use common::Case;

fn stdout(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

/// if 表达式、复合赋值、字符串拼接与比较、插值表达式、io.print / io.eprintln。
#[test]
fn everyday_expressions_run() {
    let case = Case::new("expressions", "run");
    let output = case.run(include_str!("../../examples/values/expressions.zm"));
    assert_eq!(
        stdout(&output),
        "7\nnegative\nzero\nbig\nseven\n5\n16\n2\n31\nZamak!\nZa\nxyz\ntrue\ntrue\ntrue\ntrue\nfalse\nal\nn=10 max=7 len=6 first=bob up=ZA\n3 20\nno newline here\nann!\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .filter(|line| line.starts_with("to stderr"))
            .collect::<Vec<_>>(),
        ["to stderr 7"]
    );
}

/// if 表达式的分支各自移动值，之后只能使用在两边都还有效的变量。
#[test]
fn if_expression_branches_merge_ownership() {
    let case = Case::new("expressions", "merge");
    let output = case.run("fn main() {\n    let s = \"x\"\n    let t = if true { \"y\" } else { \"z\" }\n    println(t)\n    println(s)\n    let u = if false { s } else { \"kept\" }\n    println(u)\n}\n");
    assert_eq!(stdout(&output), "y\nx\nkept\n");
}

#[test]
fn checker_rejects_expression_misuse() {
    let case = Case::new("expressions", "reject");
    for (body, message) in [
        ("let x = if true { 1 }", "an if expression needs an else branch"),
        ("let x = if true { 1 } else { \"a\" }", "mismatch"),
        ("let x = if true {\n        let y = 1\n        y\n    } else { 2 }", "single value"),
        ("let s = \"a\" + 1", "mismatch"),
        ("let n = 1 + \"a\"", "mismatch"),
        ("println(\"a\" < 1)", "mismatch"),
        ("let x = 1\n    x += 1", "cannot assign immutable value x"),
        ("let mut b = true\n    b += 1", "mismatch"),
        ("let mut xs: [i64] = [1]\n    xs[pick()] += 1", "unsupported compound assignment index"),
        ("pick() += 1", "expected a variable, field or index before +="),
        ("let s = \"x\"\n    let t = if true { s } else { \"y\" }\n    println(s)", "use after move: s"),
        ("let n = if true { 1 } else { 2 }\n    let r: &string = if true { \"a\" } else { \"b\" }", "borrow escape"),
    ] {
        let source = format!("fn pick() -> i64 {{\n    0\n}}\nfn main() {{\n    {body}\n}}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}
