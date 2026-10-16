mod common;

use common::Case;

#[test]
fn positive_variable_boundaries_match_checked_signed_arithmetic() {
    let case = Case::new("nonnegative-division", "boundaries");
    let mut source = String::from("fn main():\n");
    let mut expected = String::new();
    for (i, (n, d)) in [
        (0_i64, 1_i64),
        (1, i64::MAX),
        (i64::MAX, 1),
        (i64::MAX, 2),
        (i64::MAX, 3),
        (i64::MAX, i64::MAX),
        (i64::MAX - 1, i64::MAX),
    ]
    .into_iter()
    .enumerate()
    {
        source.push_str(&format!(
            "    let n{i} = {n}\n    let d{i} = {d}\n    println(n{i} // d{i})\n    println(n{i} % d{i})\n    println(n{i} // {d})\n    println(n{i} % {d})\n"
        ));
        expected.push_str(&format!(
            "{}\n{}\n{}\n{}\n",
            n.checked_div(d).unwrap(),
            n.checked_rem(d).unwrap(),
            n.checked_div(d).unwrap(),
            n.checked_rem(d).unwrap()
        ));
    }
    let result = case.run(&source);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
}

#[test]
fn negative_and_repeated_sign_changes_keep_signed_semantics() {
    let case = Case::new("nonnegative-division", "signed");
    let result = case.run("fn quotient(n: i64, d: i64) -> i64: n // d\nfn remainder(n: i64, d: i64) -> i64: n % d\nfn unknown() -> i64: -9\nfn main():\n    println(quotient(unknown(), 4))\n    println(remainder(unknown(), 4))\n    println(quotient(9, -4))\n    println(remainder(9, -4))\n    let mut n = 9\n    let mut i = 0\n    while i < 2:\n        println(n // 4)\n        println(n % 4)\n        n = -9\n        i = i + 1");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "-3\n3\n-3\n-3\n2\n1\n-3\n3\n"
    );
}

#[test]
fn zero_and_signed_overflow_keep_arithmetic_errors() {
    let case = Case::new("nonnegative-division", "errors");
    // 除以零报 division by zero；i64 最小值 // -1 溢出；与 Python 一样 i64 最小值 % -1 是 0。
    for (op, n, d, error) in [
        ("//", 1, 0, Some("division by zero")),
        ("%", 1, 0, Some("division by zero")),
        ("//", i64::MIN, -1, Some("integer arithmetic error")),
        ("%", i64::MIN, -1, None),
    ] {
        let source =
            format!("fn main():\n    let n = {n}\n    let d = {d}\n    println(n {op} d)\n");
        let result = case.run(&source);
        match error {
            Some(error) => {
                assert!(!result.status.success(), "accepted {source}");
                assert!(
                    String::from_utf8_lossy(&result.stderr).contains(error),
                    "{}",
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            None => assert_eq!(String::from_utf8_lossy(&result.stdout), "0\n"),
        }
    }
}

#[test]
fn operand_order_and_successful_result_facts_keep_operand_errors() {
    let case = Case::new("nonnegative-division", "operand-order");
    for (op, answer) in [("//", "2"), ("%", "1")] {
        let source = format!("fn first() -> i64:\n    println(1)\n    9\nfn second() -> i64:\n    println(2)\n    4\nfn main(): println(first() {op} second())\n");
        let result = case.run(&source);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            format!("1\n2\n{answer}\n")
        );
        // On success n + 1 is nonnegative, which proves unsigned lowering for
        // the outer operation. It must not authorize unchecked inner addition.
        let source = format!("fn unknown() -> i64: 9223372036854775807\nfn second() -> i64:\n    println(2)\n    4\nfn main():\n    let n = unknown()\n    if n >= 0: println((n + 1) {op} (second() * 0 + 4))\n");
        let result = case.run(&source);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
        let source = format!("fn unknown() -> i64: 9223372036854775807\nfn first() -> i64:\n    println(1)\n    9\nfn main():\n    let n = unknown()\n    if n >= 0: println((first() * 0) {op} (n + 1))\n");
        let result = case.run(&source);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "1\n");
    }
}

#[test]
fn specialized_nonnegative_recurrence_keeps_exact_results() {
    let case = Case::new("nonnegative-division", "recurrence");
    let result = case.run("fn accum(seed: i64) -> i64:\n    let mut acc = seed\n    let mut i = 0\n    while i < 512:\n        acc = (acc * 31 + i) % 1000003\n        i = i + 1\n    acc\nfn main():\n    let mut seed = 0\n    while seed < 8:\n        println(accum(seed))\n        seed = seed + 1");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut expected = String::new();
    for seed in 0_i64..8 {
        let mut acc = seed;
        for i in 0..512 {
            acc = acc
                .checked_mul(31)
                .unwrap()
                .checked_add(i)
                .unwrap()
                .checked_rem(1000003)
                .unwrap();
        }
        expected.push_str(&format!("{acc}\n"));
    }
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
}
