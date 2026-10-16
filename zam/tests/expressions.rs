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

#[test]
fn generic_structs_enums_and_option() {
    let case = Case::new("expressions", "generic-types");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/values/generic_types.zm"))),
        "Pair(first: \"one\", second: 1)\nOption.Some(\"b\")\nOption.Some(\"a\")\nOption.None\n2\nbob is at 1\nOption.None\n"
    );
    // 期望类型决定 `None` 的类型实参：赋值、push、实参、if 表达式、嵌套。
    let source = "enum Maybe[T]:\n    Some(value: T)\n    Nothing\n\n    fn or_else(self, fallback: T) -> T:\n        match self:\n            Some(v): copy(v)\n            Nothing: fallback\n\nfn show(m: Maybe[i64]): println(m.or_else(-1))\n\nfn main():\n    let mut x: Maybe[i64] = Maybe.Some(1)\n    x = Maybe.Nothing\n    println(x)\n    let mut xs: [Maybe[string]] = []\n    xs.push(Maybe.Nothing)\n    xs.push(Maybe.Some(\"a\"))\n    println(xs)\n    show(Maybe.Nothing)\n    show(Maybe.Some(7))\n    let v: Maybe[f64] = if xs.len() > 1: Maybe.Some(0.5) else: Maybe.Nothing\n    println(v)\n    let r: Maybe[Maybe[i64]] = Maybe.Some(Maybe.Nothing)\n    let want: Maybe[Maybe[i64]] = Maybe.Some(Maybe.Nothing)\n    println(r == want)\n    let o: Option[bool] = Option.Some(true)\n    println(copy(&o))\n";
    assert_eq!(
        stdout(&case.run(source)),
        "Maybe.Nothing\n[Maybe.Nothing, Maybe.Some(\"a\")]\n-1\n7\nMaybe.Some(0.5)\ntrue\nOption.Some(true)\n"
    );
    // 跨模块：泛型类型、它的方法和返回泛型类型的函数。
    let project = case.path("project");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"gen\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/box.zm"),
        "pub struct Box[T]:\n    item: T\n\n    pub fn get(&self) -> T: copy(&self.item)\n\npub fn find(xs: &[i64], n: i64) -> Option[i64]:\n    for i, x in xs:\n        if x == n: return Option.Some(i)\n    Option.None\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/main.zm"),
        "use box\n\nfn main():\n    let b = box.Box(item: \"x\")\n    println(b.get())\n    let n: box.Box[i64] = box.Box(item: 2)\n    println(n.get())\n    let xs: [i64] = [5, 6]\n    let at: Option[i64] = box.find(&xs, 6)\n    println(at)\n",
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&project)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "x\n2\nOption.Some(1)\n");
    let header = "enum Maybe[T]:\n    Some(value: T)\n    Nothing\n\nstruct Plain:\n    a: i64\n\nfn main():\n    ";
    for (body, message) in [
        (
            "let x = Maybe.Nothing",
            "cannot infer T for Maybe; annotate the type",
        ),
        (
            "let x: Maybe = Maybe.Some(1)",
            "Maybe takes 1 type argument(s); write it as Maybe[i64]",
        ),
        (
            "let x: Maybe[i64, string] = Maybe.Some(1)",
            "type argument count mismatch for Maybe: it takes 1",
        ),
        ("let p: Plain[i64] = Plain(a: 1)", "Plain is not generic"),
        ("let x: Maybe[i64] = Maybe.Some(\"s\")", "type mismatch"),
        ("let x = Maybe.Some([1, 2])\n    println(x)", ""),
    ] {
        let output = case.run(&format!("{header}{body}\n"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        if message.is_empty() {
            assert!(output.status.success(), "{body}: {stderr}");
            assert_eq!(stdout(&output), "Maybe.Some([1, 2])\n");
        } else {
            assert!(!output.status.success(), "accepted {body}");
            assert!(stderr.contains(message), "{body}: {stderr}");
        }
    }
}

#[test]
fn function_values_and_closures() {
    let case = Case::new("expressions", "functions");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/values/functions.zm"))),
        "20\n18\n42\n[5, 4, 6]\n2\n[\"APPLE!\", \"KIWI!\", \"BANANA!\"]\n42\n[\"fig\", \"kiwi\", \"banana\"]\n"
    );
    // 语句位置的调用、无返回值的函数、println 当函数体、枚举载荷、循环里的闭包、函数参数再转传。
    let source = "use std/io\n\nenum Op:\n    Unary(f: fn(i64) -> i64)\n    Nothing\n\nfn each(xs: &[string], f: fn(&string)):\n    for x in xs:\n        f(x)\n\nfn run(f: fn()): f()\n\nfn compose(f: fn(i64) -> i64, g: fn(i64) -> i64, x: i64) -> i64: g(f(x))\n\nfn relay(f: fn(i64) -> i64) -> i64: compose(f, f, 1)\n\nfn main():\n    let names: [string] = [\"a\", \"b\"]\n    let prefix = \"> \"\n    each(&names, fn(s): println(prefix + s))\n    run(fn(): io.println(\"ran\"))\n    let op = Op.Unary(fn(x: i64) -> i64: x * 10)\n    match op:\n        Unary(f): println(f(4))\n        Nothing: pass\n    let base = 7\n    println(compose(fn(x): x + base, fn(y): y * base, 1))\n    for i in 0..3:\n        println(relay(fn(x): x + i))\n";
    assert_eq!(
        stdout(&case.run(source)),
        "> a\n> b\nran\n40\n56\n1\n3\n5\n"
    );
    let source = "struct Person:\n    name: string\n    age: i64\n\nfn main():\n    let mut people: [Person] = [Person(name: \"amy\", age: 30), Person(name: \"bob\", age: 25), Person(name: \"cat\", age: 30)]\n    people.sort_by(fn(a, b): a.age < b.age)\n    for p in &people:\n        println(p.name)\n    let mut nums: [i64] = [3, 1, 2]\n    let desc = true\n    nums.sort_by(fn(a, b): if desc: a > b else: a < b)\n    println(nums)\n";
    assert_eq!(stdout(&case.run(source)), "bob\namy\ncat\n[3, 2, 1]\n");
    // 跨模块：导入模块的函数当值、接收回调的 pub fn。
    let project = case.path("project");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"fns\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/lib.zm"),
        "pub fn square(x: i64) -> i64: x * x\n\npub fn sum_by(xs: &[i64], f: fn(i64) -> i64) -> i64:\n    let mut total = 0\n    for x in xs:\n        total += f(x)\n    total\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/main.zm"),
        "use lib\n\nfn main():\n    let xs: [i64] = [1, 2, 3]\n    println(lib.sum_by(&xs, lib.square))\n    let k = 10\n    println(lib.sum_by(&xs, fn(x): x * k))\n",
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&project)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "14\n60\n");
    for (source, message) in [
        ("struct H:\n    f: fn() -> i64\n\nfn main():\n    let n = 1\n    let h = H(f: fn(): n)\n", "can only be passed directly as an argument"),
        ("struct H:\n    f: fn() -> i64\n\nfn keep(f: fn() -> i64) -> H: H(f: f)\n\nfn main(): pass\n", "may capture local variables of its caller"),
        ("fn give(f: fn() -> i64) -> fn() -> i64: f\n\nfn main(): pass\n", "may capture local variables of its caller"),
        ("fn main():\n    let f = fn(x): x\n", "cannot infer the type of parameter x"),
        ("fn run(f: fn()): f()\n\nfn main():\n    let xs: [i64] = []\n    run(fn(): xs.push(1))\n", "cannot mutably borrow immutable value xs"),
        ("fn both(xs: &mut [i64], f: fn() -> i64): pass\n\nfn main():\n    let mut xs: [i64] = []\n    both(&mut xs, fn(): xs.len())\n", "borrow conflict: xs"),
        ("fn run(f: fn(i64) -> i64) -> i64: f(1)\n\nfn main(): println(run(fn(a, b): a))\n", "the function takes 1 parameter(s)"),
        ("fn run(f: fn(i64) -> i64) -> i64: f(1)\n\nfn neg(b: bool) -> bool: not b\n\nfn main(): println(run(neg))\n", "neg has a different function type"),
        ("fn id[T](x: T) -> T: x\n\nfn main():\n    let f: fn(i64) -> i64 = id\n", "generic function id cannot be used as a value"),
        ("fn p(s: &string) -> i64!string: 1\n\nfn main():\n    let f = p\n", "function values cannot fail yet"),
        ("fn main():\n    let x = 1\n    let f = fn(x: i64) -> i64: x\n", "x is already declared"),
        ("fn main():\n    let f = fn(x: i64) -> i64:\n        x\n", "an anonymous fn holds one expression"),
        ("fn d(x: i64) -> i64: x\n\nfn main():\n    let f = d\n    println(f)\n", "function values cannot be printed"),
        ("fn main():\n    let mut xs: [i64] = [2, 1]\n    xs.sort_by(fn(a, b): a < xs.len())\n", "the fn reads xs while .sort_by changes it"),
    ] {
        let output = case.run(source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}

#[test]
fn local_functions_and_writable_captures() {
    let case = Case::new("expressions", "closures");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/values/closures.zm"))),
        "true\nfalse\n18\n[\"APPLE\", \"KIWI\", \"BANANA\"]\n[5, 4, 6, 3]\n"
    );
    // 泛型函数里的局部 fn、循环里定义的闭包、局部 fn 调用另一个、闭包里套闭包、改计数器。
    let source = "fn apply(f: fn(i64) -> i64, x: i64) -> i64: f(x)\n\nfn each(xs: &[i64], f: fn(i64)):\n    for x in xs:\n        f(x)\n\nfn twice(f: fn()):\n    f()\n    f()\n\nfn sum_by[T](items: &[T], weight: i64, f: fn(&T) -> i64) -> i64:\n    let mut total = 0\n    fn add(x: i64):\n        total += x * weight\n    for item in items:\n        add(f(item))\n    total\n\nfn main():\n    let xs: [i64] = [1, 2, 3]\n    println(sum_by(&xs, 10, fn(x): x))\n    let mut results: [i64] = []\n    for i in 0..3:\n        fn shifted(x: i64) -> i64: x + i * 100\n        results.push(apply(shifted, 1))\n    println(results)\n    let base = 5\n    fn inner(x: i64) -> i64: x + base\n    fn outer(x: i64) -> i64: inner(x) * 2\n    println(apply(outer, 1))\n    let mut log: [i64] = []\n    fn record(x: i64):\n        each(&xs, fn(y): log.push(x * y))\n    record(2)\n    record(3)\n    println(log)\n    let mut count = 0\n    fn bump():\n        count += 1\n    twice(bump)\n    bump()\n    println(count)\n    let scale = 10\n    let times = fn(x: i64) -> i64: x * scale\n    println(times(4))\n";
    assert_eq!(
        stdout(&case.run(source)),
        "60\n[1, 101, 201]\n12\n[2, 4, 6, 3, 6, 9]\n3\n40\n"
    );
    let header =
        "fn run(f: fn()): f()\n\nfn take(s: string): pass\n\nfn both(xs: &[i64], f: fn()): f()\n\n";
    for (body, message) in [
        ("fn main():\n    let total = 0\n    fn add():\n        total += 1\n    run(add)\n", "cannot assign immutable value total"),
        ("fn main():\n    let s = \"a\"\n    fn show(): println(s)\n    take(s)\n    run(show)\n", "use after move: s (the fn show uses it)"),
        ("fn main():\n    let mut xs: [i64] = []\n    fn add(): xs.push(1)\n    both(&xs, add)\n", "borrow conflict: xs (the fn add changes it)"),
        ("fn main():\n    let mut xs: [i64] = []\n    fn add(): xs.push(1)\n    fn outer(): add()\n    both(&xs, outer)\n", "borrow conflict: xs (the fn outer changes it)"),
        ("fn main():\n    let mut xs: [i64] = [1]\n    fn add(): xs.push(1)\n    for x in &xs:\n        run(add)\n", "xs is borrowed by an enclosing for loop"),
        ("fn make() -> fn() -> i64:\n    let n = 1\n    fn get() -> i64: n\n    get\n\nfn main(): pass\n", "get uses local variables of this function"),
        ("struct H:\n    f: fn()\n\nfn main():\n    let n = 1\n    fn show(): println(n)\n    let h = H(f: show)\n", "show uses local variables of this function"),
        ("fn main():\n    fn down(n: i64) -> i64:\n        if n == 0: return 0\n        down(n - 1)\n    println(down(3))\n", "down cannot call itself"),
        ("fn main():\n    fn id[T](x: T) -> T: x\n    println(id(1))\n", "cannot have type parameters"),
        ("fn main():\n    fn p() -> i64!string: 1\n", "cannot fail yet"),
        ("fn main():\n    fn f(x: i64) -> i64:\n        if x > 0: return 1\n    println(f(1))\n", "missing return"),
        ("fn main():\n    let n = 1\n    let mut f = fn(): println(n)\n", "cannot be declared mut"),
    ] {
        let source = format!("{header}{body}");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {body}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{body}: {stderr}");
    }
}

#[test]
fn lists_and_maps_of_functions() {
    let case = Case::new("expressions", "function-lists");
    let source = "fn double(x: i64) -> i64: x * 2\n\nfn square(x: i64) -> i64: x * x\n\nfn main():\n    let mut steps: [fn(i64) -> i64] = [double, square]\n    steps.push(fn(x): x + 1)\n    let mut value = 3\n    for step in steps:\n        value = step(value)\n    println(value)\n    let f = steps[1]\n    println(f(5))\n    let mut commands: {string: fn(i64) -> i64} = {}\n    commands[\"double\"] = double\n    commands[\"square\"] = square\n    for name in commands:\n        let run = commands[name]\n        println(\"{name} {run(4)}\")\n    let picked = commands.get(\"cube\", double)\n    println(picked(9))\n    println(commands.values().len())\n    let copied = copy(&steps)\n    println(copied == steps)\n    println(steps)\n";
    assert_eq!(
        stdout(&case.run(source)),
        "37\n25\ndouble 8\nsquare 16\n18\n2\ntrue\n[<fn>, <fn>, <fn>]\n"
    );
    // 用到局部变量的闭包不能放进列表（列表会活得比变量久）。
    let output = case.run("fn main():\n    let k = 2\n    let mut ops: [fn(i64) -> i64] = []\n    ops.push(fn(x): x * k)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("this fn uses the local variable k"));
    // 局部变量和没导入的标准库模块同名时照常是变量。
    assert_eq!(
        stdout(&case.run("fn main():\n    let mut fs: [i64] = []\n    fs.push(1)\n    let io = \"x\"\n    println(fs)\n    println(io)\n")),
        "[1]\nx\n"
    );
}
