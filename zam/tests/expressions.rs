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

/// 顶层常量：编译期求值、跨模块 `pub const`、字符串常量当 `&string` 传，以及各种误用。
#[test]
fn constants_fold_and_inline() {
    let case = Case::new("expressions", "constants");
    let source = include_str!("../../examples/values/constants.zm");
    assert_eq!(
        stdout(&case.run(source)),
        "Board v1\n8x4 = 32\n64\n15\n1.0\ntrue\nlast row 3\n"
    );
    let project = case.path("project");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"consts\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/config.zm"),
        "pub const LIMIT = 10\npub const NAME = \"zamak\"\nconst SECRET = 7\npub fn twice() -> i64: LIMIT * 2\n",
    )
    .unwrap();
    let run_project = |main: &str| {
        std::fs::write(project.join("src/main.zm"), main).unwrap();
        std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(&project)
            .env("ZAMAK_CACHE_DIR", case.path("cache"))
            .output()
            .unwrap()
    };
    let output = run_project("use config\nuse std/string\nconst N = 3\nfn main():\n    println(config.LIMIT + N)\n    println(string.byte_len(&config.NAME))\n    println(config.twice())\n    println(\"{config.NAME}!\")\n");
    assert_eq!(stdout(&output), "13\n5\n20\nzamak!\n");
    let output = run_project("use config\nfn main(): println(config.SECRET)\n");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("private constant config.SECRET"),
        "{stderr}"
    );
    for (source, message) in [
        (
            "const limit = 1\nfn main(): pass",
            "constant names are written in UPPER_CASE, such as LIMIT",
        ),
        (
            "fn f() -> i64: 1\nconst A = f()\nfn main(): pass",
            "constants can only use literals",
        ),
        (
            "const A: f64 = 1\nfn main(): pass",
            "declared f64 but its value is i64",
        ),
        (
            "const A = 1 / 0\nfn main(): pass",
            "integer arithmetic error in constant",
        ),
        (
            "const A = 1 << 64\nfn main(): pass",
            "integer arithmetic error in constant",
        ),
        (
            "const A = B\nconst B = 1\nfn main(): pass",
            "unknown constant B",
        ),
        (
            "const A = 1\nconst A = 2\nfn main(): pass",
            "duplicate declaration A",
        ),
        (
            "const P = 1\nstruct P:\n    x: i64\nfn main(): pass",
            "duplicate declaration P",
        ),
        (
            "const A = 1\nfn main():\n    A = 2",
            "cannot assign to the constant A",
        ),
        (
            "const A = 1\nfn main():\n    let A = 2",
            "A is already a constant",
        ),
        ("const A = 1 + 2.0\nfn main(): pass", "mixes i64 and f64"),
        (
            "const A = 1\nfn take(s: &string): pass\nfn main(): take(&A)",
            "cannot borrow the constant A",
        ),
    ] {
        let output = case.run(source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}

/// 元组：类型、字面量、`t.0`、`let (a, b) = ...`、多返回值、元组列表，跨模块的同一个元组类型，以及误用。
#[test]
fn tuples_and_multiple_returns() {
    let case = Case::new("expressions", "tuples");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/values/tuples.zm"))),
        "3 2\ncoffee has 6 letters\n3\n4.5\n(3, 4.5)\namy: 90\nbob: 82\ncid: 85\n[(\"amy\", 90), (\"bob\", 82), (\"cid\", 85)]\ntrue\n85\n"
    );
    let project = case.path("project");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"pairs\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/geo.zm"),
        "pub struct P:\n    x: i64\npub fn pair() -> (i64, string): (7, \"seven\")\npub fn make() -> (P, i64): (P(x: 3), 4)\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/main.zm"),
        "use geo\nfn show(t: &(i64, string)): println(\"{t.0}={t.1}\")\nfn main():\n    let t: (i64, string) = geo.pair()\n    show(&t)\n    let (n, s) = geo.pair()\n    println(\"{n} {s}\")\n    let pt: (geo.P, i64) = geo.make()\n    println(pt.0.x + pt.1)\n    let (p, k) = geo.make()\n    println(p.x * k)\n",
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&project)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "7=seven\n7 seven\n7\n12\n");
    let output = case.run("fn main():\n    let xs: [i64] = [1, 2]\n    let mut total = 0\n    for x in xs:\n        total += x\n    println(total)\n");
    assert_eq!(stdout(&output), "3\n");
    for (body, message) in [
        (
            "let (a, b, c) = (1, 2)",
            "the tuple has 2 items, but the pattern names 3",
        ),
        ("let (a, b) = 5", "needs a tuple value"),
        ("let t: (i64, string) = (1, 2)", "mismatch"),
        ("let t: (i64) = 1", "at least two types"),
        ("let t = (1, 2)\n    println(t.2)", "unknown field"),
        (
            "let s = \"a\"\n    let t = (s, 1)\n    println(s)",
            "use after move: s",
        ),
        (
            "let t = (1, 2)\n    let (a, b) = t\n    println(t.0)",
            "use after move: t",
        ),
        (
            "let n = 3\n    for x in n: pass",
            "to count, write for i in 0..n",
        ),
        ("let (a, a) = (1, 2)", "duplicate binding a"),
    ] {
        let source = format!("fn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
    let output = case.run("fn f(name: &string):\n    let t = (name, 1)\nfn main(): f(\"a\")\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("tuples hold owned values"));
}

/// 泛型函数：按实参类型生成副本，`&T` 遇到数字按值传，跨模块调用，推断失败与副本里的类型错误。
#[test]
fn generic_functions() {
    let case = Case::new("expressions", "generics");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/values/generics.zm"))),
        "9\n2.5\n2\n1\n(\"one\", 1)\n[\"ab\", \"ab\", \"ab\"]\n"
    );
    let project = case.path("project");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"gen\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/util.zm"),
        "pub fn size[T](xs: &[T]) -> i64: xs.len()\npub fn pick[T](a: T, b: T, first: bool) -> T:\n    if first: return a\n    b\nfn hidden[T](x: T) -> T: x\n",
    )
    .unwrap();
    let run_project = |main: &str| {
        std::fs::write(project.join("src/main.zm"), main).unwrap();
        std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(&project)
            .env("ZAMAK_CACHE_DIR", case.path("cache"))
            .output()
            .unwrap()
    };
    let output = run_project("use util\nfn main():\n    let xs: [string] = [\"a\"]\n    println(util.size(&xs))\n    println(util.pick(\"x\", \"y\", false))\n    println(util.pick(1.5, 2.5, true))\n");
    assert_eq!(stdout(&output), "1\ny\n1.5\n");
    let output = run_project("use util\nfn main(): println(util.hidden(1))\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("private"));
    for (source, message) in [
        ("fn make[T]() -> [T]:\n    let out: [T] = []\n    out\nfn main():\n    let xs = make()\n", "cannot infer T for make"),
        ("fn add[T](a: T, b: T) -> T: a + b\nfn main(): println(add(true, false))\n", "add[Bool]"),
        ("fn f[t](x: t) -> t: x\nfn main(): pass\n", "type parameter names start with an uppercase letter"),
    ] {
        let output = case.run(source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}
