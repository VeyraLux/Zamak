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
    let output = case.run("fn main():\n    let s = \"x\"\n    let t = if true: \"y\" else: \"z\"\n    println(t)\n    println(s)\n    let u = if false: s else: \"kept\"\n    println(u)\n");
    assert_eq!(stdout(&output), "y\nx\nkept\n");
}

#[test]
fn checker_rejects_expression_misuse() {
    let case = Case::new("expressions", "reject");
    for (body, message) in [
        (
            "let x = if true: 1",
            "an if expression needs an else branch",
        ),
        (
            "let x = if true: 1 else if false: 2 else: 3",
            "expected `elif` instead of `else if`",
        ),
        ("let x = if true: 1 else: \"a\"", "mismatch"),
        (
            "let x = if true:\n        let y = 1\n        y\n    else: 2",
            "single value",
        ),
        ("let s = \"a\" + 1", "mismatch"),
        ("let n = 1 + \"a\"", "mismatch"),
        ("println(\"a\" < 1)", "mismatch"),
        ("let x = 1\n    x += 1", "cannot assign immutable value x"),
        ("let mut b = true\n    b += 1", "mismatch"),
        (
            "let mut xs: [i64] = [1]\n    xs[pick()] += 1",
            "unsupported compound assignment index",
        ),
        (
            "pick() += 1",
            "expected a variable, field or index before +=",
        ),
        (
            "let s = \"x\"\n    let t = if true: s else: \"y\"\n    println(s)",
            "use after move: s",
        ),
        (
            "let n = if true: 1 else: 2\n    let r: &string = if true: \"a\" else: \"b\"",
            "borrow escape",
        ),
    ] {
        let source = format!("fn pick() -> i64:\n    0\nfn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}

/// `and`、`or`、`not` 的优先级与 Python 一致：从低到高是 or、and、not、比较；and/or 短路求值。
#[test]
fn word_operators_follow_python_precedence() {
    let case = Case::new("expressions", "words");
    let output = case.run(
        "fn probe() -> bool:\n    println(\"probe\")\n    true\nfn main():\n    println(not true or false)\n    println(not 1 == 2)\n    println(true and not false)\n    println(not not true)\n    println(false and probe())\n    println(true or probe())\n    println(false or true and false)\n    println((not true) == false)\n    let n = 5\n    if n > 0 and not (n > 10):\n        println(\"in range\")\n",
    );
    assert_eq!(
        stdout(&output),
        "false\ntrue\ntrue\ntrue\nfalse\ntrue\nfalse\ntrue\nin range\n"
    );
}

/// `&&`、`||`、前缀 `!` 报错并提示对应的单词；三个词是保留字；`!` 只留给错误类型。
#[test]
fn symbol_logic_operators_point_to_words() {
    let case = Case::new("expressions", "symbols");
    for (body, message, column) in [
        (
            "println(true && false)",
            "expected `and` instead of `&&`",
            18,
        ),
        (
            "println(true || false)",
            "expected `or` instead of `||`",
            18,
        ),
        ("println(!true)", "expected `not` instead of `!`", 13),
        ("let and = 1", "reserved word and", 0),
        ("let not = 1", "reserved word not", 0),
        (
            "println(1 == not true)",
            "`not` cannot follow a comparison",
            0,
        ),
    ] {
        let source = format!("fn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
        if column > 0 {
            assert!(
                stderr.contains(&format!(":2:{column}")),
                "{source}: {stderr}"
            );
            assert!(stderr.contains("write `a and b`"), "{source}: {stderr}");
        }
    }
    let output =
        case.run("fn f() -> i64!string: 1\nfn main() -> !string:\n    println(f()? != 2)\n");
    assert_eq!(stdout(&output), "true\n");
}

/// 全局 abs/min/max 按参数类型选整数或浮点版本；同名的用户函数优先；env.var 与 env.exit。
#[test]
fn global_numeric_builtins_exit_and_env() {
    let case = Case::new("expressions", "builtins");
    let source = include_str!("../../examples/values/builtins.zm");
    let file = case.path("builtins.zm");
    std::fs::write(&file, source).unwrap();
    let run = |value: Option<&str>| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_zam"));
        command
            .arg("run")
            .arg(&file)
            .env("ZAMAK_CACHE_DIR", case.path("cache"));
        match value {
            Some(value) => command.env("ZAM_DEMO_HOME", value),
            None => command.env_remove("ZAM_DEMO_HOME"),
        };
        command.output().unwrap()
    };
    let output = run(Some("/srv/中文"));
    assert_eq!(stdout(&output), "7\n2.5\n3\n1.5\n10\nhome is /srv/中文\n");
    // env.exit 结束程序：zam run 原样带回退出码，不再加自己的提示。
    let output = run(None);
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "7\n2.5\n3\n1.5\n10\nenvironment variable not set: ZAM_DEMO_HOME\n"
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("program failed"));
    let output = case.run("fn max(a: i64, b: i64) -> i64: a\nfn main():\n    println(max(1, 2))\n    println(min(-0.5, 2.0))\n");
    assert_eq!(stdout(&output), "1\n-0.5\n");
    for (body, message) in [
        (
            "println(abs(-9223372036854775807 - 1))",
            "zam: integer arithmetic error",
        ),
        ("println(min(1, 2.0))", "f64"),
        ("env.exit(256)", "zam: exit code must be between 0 and 255"),
        ("println(abs(\"x\"))", "mismatch"),
    ] {
        let source = format!("use std/env\nfn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}

/// 位运算与进制字面量：Python 的优先级，移位量越界在运行时报错，只用于 i64。
#[test]
fn bitwise_operators_and_radix_literals() {
    let case = Case::new("expressions", "bits");
    let source = include_str!("../../examples/values/bits.zm");
    assert_eq!(
        stdout(&case.run(source)),
        "280\n4294967295\n8\n15\n6\n-1\n1024\n-4\n24\ntrue\n8\n64\n64\ntrue\n"
    );
    let output = case.run("fn main():\n    println(1 << 63)\n    println(-1 >> 63)\n    println(-9223372036854775807 - 1 >> 1)\n    println(0x7FFF_FFFF_FFFF_FFFF)\n    println(-0x8000000000000000)\n    let mut x = 5\n    x &= 4\n    x >>= 1\n    println(x)\n");
    assert_eq!(
        stdout(&output),
        "-9223372036854775808\n-1\n-4611686018427387904\n9223372036854775807\n-9223372036854775808\n2\n"
    );
    for (body, message) in [
        ("println(1 << 64)", "zam: integer arithmetic error"),
        (
            "let n = -1\n    println(8 >> n)",
            "zam: integer arithmetic error",
        ),
        ("println(true & false)", "requires i64 operands"),
        ("println(1.5 | 1)", "requires i64 operands"),
        ("println(~1.0)", "~ requires an i64 operand"),
        ("println(0x)", "invalid base-16 integer literal"),
        ("println(0b102)", "invalid base-2 integer literal"),
        (
            "println(0x_1)",
            "integer separators must occur between digits",
        ),
        ("println(0xFFFF_FFFF_FFFF_FFFF)", "i64 literal overflow"),
        ("println(true && false)", "expected `and` instead of `&&`"),
    ] {
        let source = format!("fn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}
