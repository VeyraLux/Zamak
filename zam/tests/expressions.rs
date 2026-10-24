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
        command.env("ZAMAK_LANG", "en");
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
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap()
    };
    let output = run_project("use config\nuse std/string\nconst N = 3\nfn main():\n    println(config.LIMIT + N)\n    println(string.byte_len(&config.NAME))\n    println(config.twice())\n    println(\"{config.NAME}!\")\n");
    assert_eq!(stdout(&output), "13\n5\n20\nzamak!\n");
    // 列表与字典常量：用到的地方得到一份新的（for、下标、方法、借给 & 参数、跨模块）。
    let output = case.run("const NAMES = [\"Food\", \"Home\", \"Fun\"]\nconst PRIMES = [2, 3, 5, 7]\nconst LIMITS = {\"low\": 1, \"high\": 10}\nconst SCALE = 2.5\nconst RATES = [SCALE, SCALE * 2.0]\nconst GRID = [[1, 2], [3, 4]]\n\nfn total(xs: &[i64]) -> i64:\n    let mut t = 0\n    for x in xs:\n        t += x\n    t\n\nfn main():\n    for n in NAMES:\n        println(n)\n    println(len(NAMES))\n    println(NAMES[1])\n    println(total(PRIMES))\n    println(PRIMES[3])\n    println(PRIMES.index_of(5))\n    println(LIMITS[\"high\"])\n    println(RATES)\n    let mut mine = NAMES\n    mine.append(\"Work\")\n    println(mine)\n    println(NAMES)\n    for row in GRID:\n        println(row[0])\n    println(\"Home\" in NAMES)\n");
    assert_eq!(
        stdout(&output),
        "Food\nHome\nFun\n3\nHome\n17\n7\n2\n10\n[2.5, 5.0]\n[\"Food\", \"Home\", \"Fun\", \"Work\"]\n[\"Food\", \"Home\", \"Fun\"]\n1\n3\ntrue\n"
    );
    std::fs::write(
        project.join("src/config.zm"),
        "pub const LIMIT = 10\npub const NAME = \"zamak\"\nconst SECRET = 7\npub const TAGS = [\"a\", \"b\"]\npub fn twice() -> i64: LIMIT * 2\n",
    )
    .unwrap();
    let output = run_project("use config\nfn main():\n    for t in config.TAGS:\n        println(t)\n    println(config.TAGS[1])\n");
    assert_eq!(stdout(&output), "a\nb\nb\n");
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
            "division by zero in a constant",
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
        .env("ZAMAK_LANG", "en")
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
            "let t = (1, \"x\")\n    let (a, b) = t\n    println(t.0)",
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
            .env("ZAMAK_LANG", "en")
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
        .env("ZAMAK_LANG", "en")
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
        .env("ZAMAK_LANG", "en")
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
        ("fn id[T](x: T) -> T: x\n\nfn main():\n    let f = id\n", "generic function id needs a known function type"),
        ("fn p(s: &string) -> i64!string: 1\n\nfn main():\n    let f = p\n    println(f(\"1\"))\n", "fallible call f must be handled with ? or catch"),
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
        ("fn main():\n    fn id[T](x: T) -> T: x\n    println(id(1))\n", "cannot have type parameters"),
        ("fn main():\n    fn p() -> i64!string: 1\n    println(p()?)\n", "? cannot propagate the error of p"),
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

#[test]
fn equality_infers_constructors_and_generic_function_values() {
    let case = Case::new("expressions", "infer-values");
    // == / != 一边是看不出类型的构造时，按另一边的类型构造。
    let source = "enum Shape:\n    Circle(r: f64)\n    Empty\n\nfn find(xs: &[i64], n: i64) -> Option[i64]:\n    for i, x in xs:\n        if x == n: return Option.Some(i)\n    Option.None\n\nfn main():\n    let xs: [i64] = [4, 5]\n    println(find(&xs, 9) == Option.None)\n    println(Option.None == find(&xs, 5))\n    println(find(&xs, 5) == Option.Some(1))\n    println(find(&xs, 5) != Option.Some(0))\n    let s = Shape.Circle(1.0)\n    println(s == Shape.Empty)\n    let nested: Option[Option[i64]] = Option.Some(Option.None)\n    println(nested == Option.Some(Option.None))\n";
    assert_eq!(
        stdout(&case.run(source)),
        "true\nfalse\ntrue\ntrue\nfalse\ntrue\n"
    );
    let output = case.run("fn main():\n    println(Option.None == Option.None)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot infer T for Option"));

    // 泛型函数按期望的函数类型生成副本，当实参、绑定、列表元素都行。
    let source = "fn identity[T](x: T) -> T: x\n\nfn largest[T](items: &[T]) -> T:\n    let mut best = items[0]\n    for item in items:\n        if item > best: best = item\n    best\n\nfn apply(f: fn(i64) -> i64, x: i64) -> i64: f(x)\n\nfn pick(lists: &[[i64]], f: fn(&[i64]) -> i64) -> [i64]:\n    let mut out: [i64] = []\n    for list in lists:\n        out.push(f(list))\n    out\n\nfn main():\n    println(apply(identity, 7))\n    let f: fn(string) -> string = identity\n    println(f(\"hi\"))\n    let grid: [[i64]] = [[1, 9], [4, 2]]\n    println(pick(&grid, largest))\n    let steps: [fn(f64) -> f64] = [identity]\n    println(steps.len())\n";
    assert_eq!(stdout(&case.run(source)), "7\nhi\n[9, 4]\n1\n");
}

#[test]
fn calling_function_elements_directly() {
    let case = Case::new("expressions", "invoke");
    // 元素是函数时直接调用；and 短路时不求后面的下标；只用到函数字典、不调用 values() 也能编译。
    let source = "fn double(x: i64) -> i64: x * 2\n\nfn shout(s: &string) -> string: s + \"!\"\n\nfn main():\n    let steps: [fn(i64) -> i64] = [double, fn(x): x + 1]\n    println(steps[0](7))\n    println(steps[1](steps[0](3)))\n    let mut commands: {string: fn(&string) -> string} = {}\n    commands[\"shout\"] = shout\n    println(commands[\"shout\"](\"hi\"))\n    let ok = false\n    if ok and steps[5](1) > 0:\n        println(\"never\")\n    steps[0](1)\n    println(\"done\")\n";
    assert_eq!(stdout(&case.run(source)), "14\n7\nhi!\ndone\n");
    let output = case.run("fn main():\n    let xs: [i64] = [1]\n    println(xs[0](2))\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("only a function value can be called"));
}

#[test]
fn fallible_function_values() {
    let case = Case::new("expressions", "fallible-values");
    let source = "use std/string\n\nfn parse(s: &string) -> i64!string: string.parse_i64(s)?\n\nfn each_number(items: &[string], convert: fn(&string) -> i64!string) -> i64!string:\n    let mut total = 0\n    for item in items:\n        total += convert(item)?\n    total\n\nfn main() -> !string:\n    let good: [string] = [\"1\", \"2\", \"3\"]\n    println(each_number(&good, parse)?)\n    let bad: [string] = [\"4\", \"x\"]\n    each_number(&bad, parse) catch e:\n        println(\"caught: {e}\")\n    let limit = 10\n    fn checked(s: &string) -> i64!string:\n        let n = string.parse_i64(s)?\n        if n > limit:\n            fail \"too big: {n}\"\n        n\n    println(each_number(&good, checked)?)\n    let big: [string] = [\"50\"]\n    each_number(&big, checked) catch e:\n        println(\"caught: {e}\")\n    let steps: [fn(&string) -> i64!string] = [parse, fn(s): string.parse_i64(s)?]\n    println(steps[1](\"7\")?)\n    steps[0](\"q\") catch e:\n        println(\"caught: {e}\")\n    let f = parse\n    let n = f(\"40\")?\n    println(n + 2)\n    let saver: fn(&string) -> !string = fn(s): fail \"cannot save {s}\"\n    saver(\"x\") catch e:\n        println(e)\n";
    assert_eq!(
        stdout(&case.run(source)),
        "6\ncaught: invalid integer: x\n6\ncaught: too big: 50\n7\ncaught: invalid integer: q\n42\ncannot save x\n"
    );
    // 错误类型是枚举时照样 match。
    let source = "enum ParseError:\n    Empty\n    Bad(text: string)\n\nfn parse(s: &string) -> i64!ParseError:\n    if s == \"\": fail ParseError.Empty\n    fail ParseError.Bad(copy(s))\n\nfn main():\n    let f: fn(&string) -> i64!ParseError = parse\n    f(\"\") catch e:\n        match e:\n            Empty: println(\"empty\")\n            Bad(t): println(\"bad {t}\")\n    f(\"z\") catch e:\n        println(e)\n";
    assert_eq!(stdout(&case.run(source)), "empty\nParseError.Bad(\"z\")\n");
    let header = "use std/string\n\nfn parse(s: &string) -> i64!string: string.parse_i64(s)?\n\nfn double(x: i64) -> i64: x * 2\n\n";
    for (body, message) in [
        ("fn main():\n    let f = parse\n    println(f(\"1\"))\n", "fallible call f must be handled with ? or catch"),
        ("fn main() -> !string:\n    let f = double\n    println(f(1)?)\n", "f cannot fail, so it needs no ? or catch"),
        ("fn run(f: fn(&string) -> i64) -> i64: f(\"1\")\n\nfn main(): println(run(parse))\n", "parse can fail, but the expected function type has no !E"),
        ("fn main():\n    let f = parse\n    println(f(\"1\")?)\n", "? cannot propagate the error of f"),
        ("fn main():\n    let fs: [fn(&string) -> i64!string] = [parse]\n    println(fs[0](\"1\"))\n", "must be handled with ? or catch"),
    ] {
        let output = case.run(&format!("{header}{body}"));
        assert!(!output.status.success(), "accepted {body}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{body}: {stderr}");
    }
}

#[test]
fn python_style_conveniences() {
    let case = Case::new("expressions", "conveniences");
    // 直接遍历调用结果与列表字面量；print 同 println（模块自己定义了 print 时用自己的）；
    // 调用结果、拼接、if 的值可以直接借给只读借用参数，`&f(x)` 等于 `f(x)`。
    let source = "use std/string\n\nstruct P:\n    name: string\n\nfn words() -> [string]: string.words(\"b a c\")\n\nfn total(xs: &[i64]) -> i64:\n    let mut sum = 0\n    for x in xs: sum += x\n    sum\n\nfn shout(s: &string) -> string: string.upper(s)\n\nfn label(p: &P) -> string: copy(&p.name)\n\nfn main() -> !string:\n    for w in words():\n        print(w)\n    for i, w in string.split(\"x,y\", \",\"):\n        print(\"{i} {w}\")\n    for n in [3, 1, 2]:\n        if n > 1:\n            for k in words():\n                print(\"{n}{k}\")\n    for name, count in {\"a\": 1, \"b\": 2}:\n        print(\"{name}={count}\")\n    print()\n    let line = \"  42 \"\n    print(string.parse_i64(string.trim(&line))?)\n    let xs: [i64] = [1, 2, 3, 4]\n    print(total(&xs.slice(0, 2)))\n    print(total(xs.slice(1, 4)))\n    print(shout(\"a\" + \"b\"))\n    print(shout(if true: \"yes\" else: \"no\"))\n    print(label(P(name: \"p\")))\n    print(string.byte_len(&string.repeat(\"ab\", 3)))\n";
    assert_eq!(
        stdout(&case.run(source)),
        "b\na\nc\n0 x\n1 y\n3b\n3a\n3c\n2b\n2a\n2c\na=1\nb=2\n\n42\n3\n9\nAB\nYES\np\n6\n"
    );
    assert_eq!(
        stdout(&case.run("fn print(x: i64): println(x * 10)\n\nfn main(): print(4)\n")),
        "40\n"
    );
    let output = case.run("fn make() -> [i64]: [1]\n\nfn grow(xs: &mut [i64]): xs.push(1)\n\nfn main(): grow(&mut make())\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot mutably borrow a call result"));
}

#[test]
fn string_methods() {
    let case = Case::new("expressions", "string-methods");
    // 字符串上的方法就是 std/string 的同名函数，不用导入；len 是字节数，char_len 按字符计。
    let source = "struct P:\n    name: string\n\nfn shout(s: &string) -> string: s.upper() + \"!\"\n\nfn main():\n    let s = \"Hello, World\"\n    println(s.len())\n    println(s.upper())\n    println(s.lower().contains(\"world\"))\n    println(s.split(\", \"))\n    for part in s.split(\", \"):\n        println(part.char_len())\n    let line = \"  42  \"\n    println(line.trim().len())\n    let p = P(name: \"zam\")\n    println(p.name.upper())\n    println(shout(&p.name))\n    let hi = \"你好\"\n    println(hi.char_len())\n    println(hi.len())\n    println(s.find(\"World\"))\n    println(s.slice(0, 5).repeat(2))\n";
    assert_eq!(
        stdout(&case.run(source)),
        "12\nHELLO, WORLD\ntrue\n[\"Hello\", \"World\"]\n5\n5\n2\nZAM\nZAM!\n2\n2\n7\nHelloHello\n"
    );
    let output = case.run("fn main():\n    let s = \"a\"\n    println(s.nope())\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown method .nope for a string"));
}

#[test]
fn borrowed_locals() {
    let case = Case::new("expressions", "borrowed-locals");
    // `let x = &place` 把只读借用存进局部变量；到代码块结束，被借用的根不能改、不能移走。
    let source = "struct Entry:\n    word: string\n    count: i64\n\nfn show(e: &Entry): println(\"{e.word}={e.count}\")\n\nfn main():\n    let mut entries: [Entry] = [Entry(word: \"a\", count: 2), Entry(word: \"b\", count: 5)]\n    for i in 0..entries.len():\n        let entry = &entries[i]\n        println(entry.word.upper())\n        show(entry)\n        println(entry.count * 10)\n    entries.push(Entry(word: \"c\", count: 1))\n    let name = &entries[2].word\n    println(name.len())\n    let all = &entries\n    println(all.len())\n";
    assert_eq!(stdout(&case.run(source)), "A\na=2\n20\nB\nb=5\n50\n1\n3\n");
    let output = case.run("fn main():\n    let mut names: [string] = [\"a\"]\n    let n = &names[0]\n    names.push(\"b\")\n    println(n)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("borrow conflict: names is borrowed"));
    let output = case
        .run("fn get(xs: &[string]) -> string:\n    let n = &xs[0]\n    n\n\nfn main():\n    let xs: [string] = [\"k\"]\n    println(get(xs))\n    println(xs)\n");
    // 借用的字符串在要 owned 的地方复制一份（与 Python 一样按值使用），原列表不变。
    assert_eq!(stdout(&output), "k\n[\"k\"]\n");
    let output =
        case.run("fn main():\n    let xs: [i64] = [1]\n    let n = &xs[0]\n    println(n)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("scalar references are not supported"));
}

#[test]
fn python_expressions() {
    let case = Case::new("expressions", "python");
    // Python 的写法：`a if c else b`、`True`/`False`、`f"..."`、`x in xs`/`x not in xs`、`a ** b`。
    assert_eq!(
        stdout(&case.run("fn pick(n: i64) -> string: \"big\" if n > 5 else \"small\"\n\nfn main():\n    let n = 7\n    let ok = True\n    println(f\"n={n} ok={ok}\")\n    println(pick(n))\n    println(pick(1))\n    let v = n * 2 if not False else 0\n    println(v)\n    let w = 1 if n < 0 else 2 if n < 10 else 3\n    println(w)\n    if True and n > 1: println(\"yes\")\n")),
        "n=7 ok=true\nbig\nsmall\n14\n2\nyes\n"
    );
    assert_eq!(
        stdout(&case.run("struct P:\n    name: string\n\nfn main():\n    let names: [string] = [\"ann\", \"bob\"]\n    let name = \"bob\"\n    println(name in names)\n    println(\"zed\" in names)\n    println(\"zed\" not in names)\n    let nums = [3, 5]\n    let xs: [i64] = [1, 2, 3]\n    println(2 in xs)\n    println(9 not in xs)\n    let m = {\"k\": 1}\n    println(\"k\" in m)\n    let s = \"hello\"\n    println(\"ell\" in s)\n    println(name in s)\n    let p = P(name: \"x\")\n    println(p.name in names)\n    println(names[0] in names)\n    if \"ann\" in names and 1 in xs: println(\"both\")\n    println(\"a\" in \"cat\")\n    println(2 ** 10)\n    println(2 ** 3 ** 2)\n    println(2.0 ** 0.5)\n    println(10 ** 18)\n    println(name)\n")),
        "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\ntrue\nboth\ntrue\n1024\n512\n1.4142135623730951\n1000000000000000000\nbob\n"
    );
    let output = case.run("fn main():\n    let e = -1\n    println(2 ** e)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("negative exponent"));
    let output = case.run("fn main(): println(10 ** 19)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("integer arithmetic error"));
}

#[test]
fn automatic_shared_borrows() {
    let case = Case::new("expressions", "auto-borrow");
    // 只读借用参数自动借用：变量、字段、元素直接传；copy(p) 同 copy(&p)；&mut 仍要显式写。
    assert_eq!(
        stdout(&case.run("struct P:\n    name: string\n    tags: [string]\n\nfn show(s: &string): println(\"<{s}>\")\nfn count(xs: &[string]) -> i64: xs.len()\nfn first(p: &P) -> i64: p.tags.len()\nfn grow(xs: &mut [string]): xs.push(\"z\")\n\nfn main():\n    let name = \"ann\"\n    show(name)\n    show(name)\n    let mut names: [string] = [\"a\", \"b\"]\n    show(names[1])\n    println(count(names))\n    let p = P(name: \"pp\", tags: [\"t\"])\n    show(p.name)\n    println(first(p))\n    println(count(p.tags))\n    println(names.contains(name))\n    let s = \"a,b\"\n    println(s.contains(name))\n    grow(&mut names)\n    println(count(names))\n")),
        "<ann>\n<ann>\n<b>\n2\n<pp>\n1\n1\nfalse\nfalse\n3\n"
    );
    assert_eq!(
        stdout(&case.run("struct P:\n    x: i64\n    s: string\n\nfn main():\n    let p = P(x: 1, s: \"a\")\n    let q = copy(p)\n    let t = copy(p.s)\n    let xs: [string] = [\"k\"]\n    let u = copy(xs[0])\n    println(q)\n    println(t + u)\n    println(p)\n")),
        "P(x: 1, s: \"a\")\nak\nP(x: 1, s: \"a\")\n"
    );
}

#[test]
fn python_division() {
    let case = Case::new("expressions", "python-division");
    // 除法与 Python 相同：/ 得小数，// 向下取整，% 符号跟除数，整数与小数混用变小数，常量同样折叠。
    assert_eq!(
        stdout(&case.run("const HALF = 7 / 2\nconst FLOOR = -7 // 2\n\nfn main():\n    println(7 / 2)\n    println(6 / 3)\n    println(7 // 2)\n    println(-7 // 2)\n    println(7 % 3)\n    println(-7 % 3)\n    println(7 % -3)\n    println(7.5 // 2.0)\n    println(-7.5 % 2.0)\n    println(7 / 2.0)\n    println(1 + 2.5)\n    println(3 < 3.5)\n    println(HALF)\n    println(FLOOR)\n    let mut n = 17\n    n //= 5\n    println(n)\n    let xs = [10, 20, 30]\n    let i = 5\n    println(xs[i % 3])\n    let total = 7\n    let count = 2\n    let avg = total / count\n    println(avg)\n    # comment\n    println(2 ** 3 // 3)\n")),
        "3.5\n2.0\n3\n-4\n1\n2\n-2\n3.0\n0.5\n3.5\n3.5\ntrue\n3.5\n-4\n3\n30\n3.5\n2\n"
    );
}

#[test]
fn python_builtins() {
    let case = Case::new("expressions", "python-builtins");
    // Python 的内置函数：len、sum、min/max（两个数或一个列表）、sorted、str、int/float（转换或解析，解析会失败）、for i in range(...)。
    assert_eq!(
        stdout(&case.run("struct P:\n    tags: [string]\n\nfn main() -> !string:\n    let xs: [i64] = [3, 1, 2]\n    let fs: [f64] = [1.5, 2.5]\n    let words: [string] = [\"pear\", \"apple\", \"fig\"]\n    println(len(xs))\n    println(len(\"héllo\"))\n    println(len({\"a\": 1}))\n    println(sum(xs))\n    println(sum(fs))\n    println(min(xs))\n    println(max(words))\n    println(min(3, 9))\n    println(sorted(xs))\n    println(sorted(words))\n    println(xs)\n    println(str(42) + \"!\")\n    println(str(xs))\n    println(int(\"12\")? + 1)\n    println(int(3.9))\n    println(float(2))\n    println(float(\"2.5\")?)\n    let s = \"7\"\n    println(int(s)? * 2)\n    for i in range(3): print(i)\n    for i in range(2, 4): print(i)\n    let p = P(tags: [\"b\", \"a\"])\n    println(len(p.tags))\n    println(sorted(p.tags))\n    let csv = \"a,b,c\"\n    println(len(csv.split(\",\")))\n    let bad = int(\"x\") catch e:\n        println(e)\n        return\n")),
        "3\n5\n1\n6\n4.0\n1\npear\n3\n[1, 2, 3]\n[\"apple\", \"fig\", \"pear\"]\n[3, 1, 2]\n42!\n[3, 1, 2]\n13\n3\n2.0\n2.5\n14\n0\n1\n2\n2\n3\n2\n[\"a\", \"b\"]\n3\ninvalid integer: x\n"
    );
}

#[test]
fn python_indexing() {
    let case = Case::new("expressions", "python-indexing");
    // 与 Python 相同：负下标从末尾数；切片 xs[a:b]（可省略两端、可为负）越界收拢；insert 收拢位置；s[a:b] 切字符串。
    assert_eq!(
        stdout(&case.run("fn main():\n    let mut xs: [i64] = [10, 20, 30, 40, 50]\n    println(xs[-1])\n    println(xs[-5])\n    println(xs[1:3])\n    println(xs[:2])\n    println(xs[3:])\n    println(xs[-2:])\n    println(xs[:-1])\n    println(xs[10:])\n    println(xs[3:1])\n    println(xs[1:].len())\n    xs[-1] = 99\n    println(xs)\n    xs.insert(-1, 7)\n    xs.insert(100, 8)\n    xs.insert(-100, 1)\n    println(xs)\n    println(xs.pop(-1))\n    let s = \"hello\"\n    println(s[1:4])\n    println(s[-3:])\n    println(s[:100])\n    let words: [string] = [\"a\", \"b\", \"c\"]\n    println(words[-1])\n    println(words[1:])\n    let i = -2\n    println(xs[i])\n")),
        "50\n10\n[20, 30]\n[10, 20]\n[40, 50]\n[40, 50]\n[10, 20, 30, 40]\n[]\n[]\n4\n[10, 20, 30, 40, 99]\n[1, 10, 20, 30, 40, 7, 99, 8]\n8\nell\nllo\nhello\nc\n[\"b\", \"c\"]\n7\n"
    );
}

#[test]
fn python_strings() {
    let case = Case::new("expressions", "python-strings");
    // 字符串位置按字符算（与 Python 相同）：len、s[i]、s[a:b]、find；按字节用 byte_*；for ch in s 逐字符遍历，break/continue/return 都释放当前字符。
    assert_eq!(
        stdout(&case.run("fn main():\n    let s = \"你好, world\"\n    println(len(s))\n    println(s.byte_len())\n    println(s[0])\n    println(s[-1])\n    println(s[0:2])\n    println(s[4:])\n    println(s.find(\"world\"))\n    println(s.find_from(\"o\", 8))\n    println(s.byte_find(\"world\"))\n    println(s.slice(-5, 100))\n    let e = \"héllo\"\n    println(len(e))\n    println(e[1])\n    let ascii = \"abcdef\"\n    println(ascii[2:4] + ascii[-1])\n    let mut total = 0\n    for i in range(len(s)):\n        if s[i] == \"o\": total += 1\n    println(total)\n    let mut grow = \"ab\"\n    grow += \"中\"\n    println(len(grow))\n    println(grow[2])\n")),
        "9\n13\n你\nd\n你好\nworld\n4\n-1\n8\nworld\n5\né\ncdf\n1\n3\n中\n"
    );
    assert_eq!(
        stdout(&case.run("fn first_vowel(s: &string) -> string:\n    for ch in s:\n        if ch in \"aeiou\": return copy(ch)\n    return \"-\"\n\nfn main():\n    let s = \"a中b\"\n    for ch in s:\n        print(ch)\n    for i, ch in s:\n        print(\"{i}{ch}\")\n    let mut n = 0\n    for ch in \"hello world\":\n        if ch == \" \": break\n        if ch == \"l\": continue\n        n += 1\n    println(n)\n    println(first_vowel(\"xyz\"))\n    println(first_vowel(\"strong\"))\n    let words: [string] = [\"ab\", \"c\"]\n    for w in words:\n        for ch in w:\n            print(ch.upper())\n")),
        "a\n中\nb\n0a\n1中\n2b\n3\n-\no\nA\nB\nC\n"
    );
}

#[test]
fn python_string_methods() {
    let case = Case::new("expressions", "python-string-methods");
    // 字面量上调方法、sep.join(xs)、Python 的 strip/startswith/endswith/ljust/rjust/lstrip/rstrip/is*，借用的字符串在要 owned 的地方自动复制。
    assert_eq!(
        stdout(&case.run("fn first_vowel(s: &string) -> string:\n    for ch in s:\n        if ch in \"aeiou\": return ch\n    return \"-\"\n\nfn shout(s: &string) -> string: s.upper()\n\nfn keep(s: &string) -> string: s\n\nfn main():\n    println(\"a,b,c\".split(\",\"))\n    println(\", \".join([\"x\", \"y\"]))\n    let words: [string] = [\"p\", \"q\"]\n    let sep = \"-\"\n    println(sep.join(words))\n    println(\"  hi  \".strip().upper() + \"!\")\n    println(\"Hello\".lower().len())\n    println(first_vowel(\"strong\"))\n    println(shout(\"abc\"))\n    let k = keep(\"orig\")\n    println(k)\n    let mut out: [string] = []\n    for w in words:\n        out.push(w)\n    println(out)\n    println(words)\n    println(\"  x \".lstrip() + \"|\" + \" y  \".rstrip() + \"|\")\n    println(\"2024\".isdigit())\n    println(\"ab中\".isalpha())\n    println(\"a1\".isalnum())\n    println(\" \\t\".isspace())\n    println(\"file.txt\".endswith(\".txt\"))\n    println(\"ab\".rjust(4) + \"|\")\n")),
        "[\"a\", \"b\", \"c\"]\nx, y\np-q\nHI!\n5\no\nABC\norig\n[\"p\", \"q\"]\n[\"p\", \"q\"]\nx | y|\ntrue\ntrue\ntrue\ntrue\ntrue\n  ab|\n"
    );
}

#[test]
fn power_binds_tighter_than_unary_minus_as_in_python() {
    let case = Case::new("expressions", "python-power");
    // 与 Python 相同：-2 ** 2 是 -(2 ** 2)；~ 同理；有一边是浮点数就按浮点算；负的整数字面量照常。
    assert_eq!(
        stdout(&case.run("fn main():\n    println(-2 ** 2)\n    println(~2 ** 2)\n    println(2 ** -1.0)\n    println((-2) ** 2)\n    println(-9223372036854775808)\n    println(-2.0 ** 2)\n    println(2 ** 3 ** 2)\n    println(1 - -2 ** 2)\n    let x = 3\n    println(-x ** 2)\n")),
        "-4\n-5\n0.5\n4\n-9223372036854775808\n-4.0\n512\n5\n-9\n"
    );
}

#[test]
fn everyday_python_shapes_compile() {
    // 实战里遇到的写法：调用结果取下标与切片、元素方法后接 ?、字面量上调方法、
    // 嵌套的 catch e、catch 块里 fail 同一个值、借来的纯数据值当回退值、带格式说明的链式调用。
    let case = common::Case::new("expressions", "python-shapes");
    let output = case.run(
        r#"use std/env

struct P:
    x: i64

    fn q(&self) -> P:
        P(x: self.x * 2)

    fn show(&self) -> string:
        f"{self.q().double():03}"

    fn double(&self) -> i64:
        self.x * 2

fn parse(a: &string, b: &string) -> i64!string:
    let bad = "bad input"
    let x = a.parse_i64() catch e: fail bad
    let y = b.parse_i64() catch e: fail bad
    x + y

fn parse_p(text: &string) -> P!string:
    P(x: text.parse_i64()?)

fn fallback(p: &P, text: &string) -> P:
    let d = parse_p(text) catch e: p
    d

fn main() -> !string:
    let line = "a,b,c"
    println(line.split(",")[0])
    println(line.split(",")[1:])
    println(line.upper()[-1])
    let args: [string] = ["7", "x"]
    println(args[0].parse_i64()? + 1)
    println(P(x: 2).show())
    println(parse("1", "2")?)
    parse("1", "z") catch e:
        println(e)
    let home = env.var("ZAMAK_NO_SUCH_VAR") catch e: env.var("ZAMAK_NO_SUCH_VAR_2") catch e: "."
    println(home)
    println(fallback(P(x: 5), "q").x)
"#,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "a\n[\"b\", \"c\"]\nC\n8\n008\n3\nbad input\n.\n5\n"
    );
}
