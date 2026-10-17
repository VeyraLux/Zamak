mod common;

use common::Case;

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

#[test]
fn wrapping_functions_wrap_integers_and_follow_ieee_for_floats() {
    let case = Case::new("wrapping", "semantics");
    let output = case.run(
        "@wrapping\nfn hash(seed: i64, n: i64) -> i64:\n    let mut h = seed\n    for i in range(n):\n        h = h * 1099511628211 + i\n        h += 1\n    h\n\n@wrapping\nfn edges(a: i64, b: i64) -> i64: -a + (b - 1)\n\n@wrapping\nfn ratio(a: f64, b: f64) -> f64: a / b\n\n@wrapping\nfn floor_ratio(a: f64, b: f64) -> f64: a // b + a % 3.0\n\nfn main():\n    println(hash(9223372036854775807, 3))\n    println(edges(-9223372036854775807 - 1, -9223372036854775807 - 1))\n    println(ratio(1.0, 0.0))\n    println(ratio(-1.0, 0.0))\n    println(floor_ratio(-7.0, 2.0))\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut h = i64::MAX;
    for i in 0..3_i64 {
        h = h
            .wrapping_mul(1099511628211)
            .wrapping_add(i)
            .wrapping_add(1);
    }
    let edges = i64::MIN
        .wrapping_neg()
        .wrapping_add(i64::MIN.wrapping_sub(1));
    let lines: Vec<String> = stdout(&output).lines().map(String::from).collect();
    assert_eq!(lines[0], h.to_string());
    assert_eq!(lines[1], edges.to_string());
    assert!(
        lines[2].contains("inf") && !lines[2].starts_with('-'),
        "{lines:?}"
    );
    assert!(lines[3].starts_with("-inf"), "{lines:?}");
    // -7.0 // 2.0 == -4.0，-7.0 % 3.0 == 2.0（与 Python 相同）。
    assert_eq!(lines[4], "-2.0", "{lines:?}");
}

#[test]
fn wrapping_keeps_other_checks_and_other_functions_checked() {
    let case = Case::new("wrapping", "checks");
    // 整数除以零照常报错。
    let output = case
        .run("@wrapping\nfn div(a: i64, b: i64) -> i64: a // b\nfn main(): println(div(1, 0))\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("division by zero"));
    // 下标越界照常报错。
    let output = case.run("@wrapping\nfn get(xs: &[i64], i: i64) -> i64: xs[i] + 1\nfn main():\n    let xs: [i64] = [1]\n    println(get(xs, 5))\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("out of bounds"));
    // 没有 @wrapping 的函数仍然检查溢出。
    let output = case.run("@wrapping\nfn wrap(a: i64) -> i64: a + 1\nfn add(a: i64) -> i64: a + 1\nfn main():\n    println(wrap(9223372036854775807))\n    println(add(9223372036854775807))\n");
    assert!(!output.status.success());
    assert_eq!(stdout(&output), "-9223372036854775808\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("integer arithmetic error"));
}

#[test]
fn decorators_are_validated() {
    let case = Case::new("wrapping", "errors");
    let output = case.run("@fast\nfn main(): println(1)\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown decorator @fast"));
    let output = case.run("@wrapping\nconst LIMIT = 1\nfn main(): println(LIMIT)\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("@wrapping applies only to a fn"));
}
