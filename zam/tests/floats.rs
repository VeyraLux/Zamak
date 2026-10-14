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

/// f64 的字面量、运算、打印、结构体字段、列表、字典、转换与 std/math。
#[test]
fn floats_example_runs() {
    let case = Case::new("floats", "example");
    let output = case.run(include_str!("../../examples/values/floats.zm"));
    assert_eq!(
        stdout(&output),
        "0.30000000000000004\n3.5\n1e16\n2.5e-7\n-1.5\ninf\ntrue\narea 12.566\nmean 5.0 stddev 2.0 min 2.0\ntotal 8.00\n9\n10\n0.75\n1024.0\n-2.0\n6.02e23\ninvalid float: 1.2.3\n"
    );
}

/// 打印用能精确读回的最短十进制；指数在 [-5, 16) 内写定点，否则写科学计数法。
#[test]
fn printing_uses_shortest_round_trip_text() {
    let case = Case::new("floats", "print");
    let values = [
        ("1.0", "1.0"),
        ("-2.5", "-2.5"),
        ("0.1", "0.1"),
        ("100.0", "100.0"),
        ("123456789.125", "123456789.125"),
        ("1e15", "1000000000000000.0"),
        ("1e16", "1e16"),
        ("0.00001", "0.00001"),
        ("0.000001", "1e-6"),
        ("1.7976931348623157e308", "1.7976931348623157e308"),
        ("5e-324", "5e-324"),
        ("-0.0", "-0.0"),
        ("1_000.000_5", "1000.0005"),
        ("2.5E3", "2500.0"),
        ("0.0 / 0.0", "NaN"),
        ("-1.0 / 0.0", "-inf"),
        ("1.0 / 3.0", "0.3333333333333333"),
        ("2.0 / 3.0 * 3.0", "2.0"),
    ];
    let body: String = values
        .iter()
        .map(|(expr, _)| format!("    println({expr})\n    println(\"{{{expr}}}\")\n"))
        .collect();
    let output = case.run(&format!("fn main():\n{body}"));
    let expected: String = values
        .iter()
        .map(|(_, text)| format!("{text}\n{text}\n"))
        .collect();
    assert_eq!(stdout(&output), expected);
}

/// 比较、取负、取余、if 表达式、复合赋值与区间循环里的计数器都能和 f64 一起用。
#[test]
fn float_operators_follow_ieee() {
    let case = Case::new("floats", "operators");
    let output = case.run(
        "use std/math\nfn half(x: f64) -> f64:\n    x / 2.0\nfn main():\n    let nan = 0.0 / 0.0\n    println(nan == nan)\n    println(nan != nan)\n    println(math.is_nan(nan))\n    println(-0.0 == 0.0)\n    println(2.5 >= 2.5)\n    println(-half(5.0))\n    println(7.5 % 2.0)\n    let mut x = 1.0\n    for i in 0..4:\n        x *= 1.5\n    x -= 0.0625\n    println(x)\n    let sign = if x > 0.0: 1.0 else: -1.0\n    println(sign)\n    println(math.max(math.abs(-3.0), 2.0))\n    println(math.trunc(-2.7))\n    println(math.ceil(2.1))\n    println(math.atan2(1.0, 1.0) * 4.0)\n    println(math.log(math.e))\n    println(math.log10(1000.0))\n    println(math.exp(0.0))\n",
    );
    assert_eq!(
        stdout(&output),
        "false\ntrue\ntrue\ntrue\ntrue\n-2.5\n1.5\n5.0\n1.0\n3.0\n-2.0\n3.0\n3.141592653589793\n1.0\n3.0\n1.0\n"
    );
}

/// fmt.fixed 按位数四舍五入（作用在二进制值上），NaN 与无穷照常打印。
#[test]
fn fixed_formats_with_digits() {
    let case = Case::new("floats", "fixed");
    let output = case.run(
        "use std/fmt\nfn main():\n    println(fmt.fixed(3.14159, 2))\n    println(fmt.fixed(2.5, 0))\n    println(fmt.fixed(-1.0, 3))\n    println(fmt.fixed(1e20, 1))\n    println(fmt.fixed(0.0 / 0.0, 2))\n    println(fmt.fixed(1.0 / 0.0, 2))\n    println(fmt.fixed(1.0, 31))\n",
    );
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "3.14\n2\n-1.000\n100000000000000000000.0\nNaN\ninf\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("zam: fmt.fixed digits must be between 0 and 30"));
}

/// string.parse_f64 只认十进制写法；i64(x) 遇到 NaN 或超出范围的值在运行时报错。
#[test]
fn parsing_and_conversions_check_their_input() {
    let case = Case::new("floats", "parse");
    let output = case.run(
        "use std/string\nfn parse(text: &string):\n    let value = string.parse_f64(text) catch e:\n        println(e)\n        return\n    println(value)\nfn main():\n    let texts: [string] = [\"42\", \"-0.5\", \"+.5\", \"5.\", \"1e3\", \"2.5E-3\", \"\", \"-\", \".\", \"1e\", \" 1\", \"inf\", \"nan\", \"0x10\", \"1e999\"]\n    for text in &texts:\n        parse(text)\n    println(i64(-7.9))\n    println(i64(9.2e18))\n    println(f64(9007199254740993))\n    println(i64(1e19))\n",
    );
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "42.0\n-0.5\n0.5\n5.0\n1000.0\n0.0025\ninvalid float: \ninvalid float: -\ninvalid float: .\ninvalid float: 1e\ninvalid float:  1\ninvalid float: inf\ninvalid float: nan\ninvalid float: 0x10\nfloat out of range: 1e999\n-7\n9200000000000000000\n9007199254740992.0\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("zam: f64 value out of i64 range"));
}

/// 没有隐式转换：i64 与 f64 混用、字面量写错类型都在检查阶段报错并给出改法。
#[test]
fn checker_rejects_float_misuse() {
    let case = Case::new("floats", "reject");
    for (body, message) in [
        (
            "let x = 1.5\n    println(x * 2)",
            "write the integer 2 as 2.0",
        ),
        (
            "let n = 3\n    println(1.5 + n)",
            "convert one side with f64(n) or i64(x)",
        ),
        ("let x: f64 = 1", "found the integer 1; write 1.0"),
        (
            "let n: i64 = 2.5",
            "found an f64 literal; convert with i64(x)",
        ),
        (
            "let x = 2.5\n    let n: i64 = x",
            "x is f64 but i64 is expected",
        ),
        ("let n = 2\n    let x: f64 = n", "convert with f64(n)"),
        (
            "let n: i64 = 1.5 * 2.0",
            "this f64 expression is used as i64",
        ),
        ("println(f64(1.5))", "type mismatch"),
        ("println(i64(2))", "type mismatch"),
        ("for i in 0..2.5: pass", "mismatch"),
        ("let m: {f64: i64} = {}", "unsupported map key type"),
        ("let x = 1.5e", "expected digits in the f64 exponent"),
        ("let x = 1e400", "f64 literal is out of range"),
        (
            "let x = 1_.5",
            "integer separators must occur between digits",
        ),
        ("let x = 1.5_", "digit separators must occur between digits"),
        (
            "println(math.tau)",
            "std/math has the constants pi, e and inf",
        ),
        (
            "let xs: [f64] = [1.0]\n    take(&xs[0])",
            "scalar references are not supported",
        ),
    ] {
        let source = format!("use std/math\nfn take(s: &string): pass\nfn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}\n{stderr}");
    }
    let output = case.run("fn f64(): pass\nfn main(): pass\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("reserved type name f64"));
}
