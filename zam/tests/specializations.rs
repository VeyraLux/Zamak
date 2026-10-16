mod common;

use common::Case;

#[test]
fn repeated_call_site_joins_all_visits_before_removing_checks() {
    let case = Case::new("specializations", "joined-overflow");
    let result = case.run("fn add(n: i64) -> i64: n + 1\nfn main():\n    let mut n = 0\n    let mut i = 0\n    while i < 2:\n        println(add(n))\n        n = 9223372036854775807\n        i = i + 1");
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "1\n");
}

#[test]
fn negative_modulus_and_nested_generic_fallback_keep_behavior() {
    let case = Case::new("specializations", "negative-unknown");
    let result = case.run("fn rem(n: i64) -> i64: (n * 31 + 1) % 7\nfn unknown() -> i64: -5\nfn nested(n: i64) -> i64: rem(n)\nfn main():\n    println(rem(-5))\n    println(rem(unknown()))\n    println(nested(-5))");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "0\n0\n0\n");
    let remainder_inputs = case.run("fn twice(n: i64) -> i64: n * 2\nfn main():\n    let mut n = -9\n    while n < 0:\n        println(twice(n % 8))\n        n = n + 1");
    assert!(
        remainder_inputs.status.success(),
        "{}",
        String::from_utf8_lossy(&remainder_inputs.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&remainder_inputs.stdout),
        "14\n0\n2\n4\n6\n8\n10\n12\n14\n"
    );
    let failed = case.run("fn unknown() -> i64: 9223372036854775807\nfn add(n: i64) -> i64: n + 1\nfn main(): println(add(unknown()))");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("integer arithmetic error"));
}

#[test]
fn specialized_call_evaluates_arguments_in_original_order() {
    let case = Case::new("specializations", "argument-order");
    let result = case.run("fn first() -> i64:\n    println(1)\n    3\nfn second() -> i64:\n    println(2)\n    4\nfn sum(a: i64, b: i64, seed: i64) -> i64: a + b + seed\nfn main(): println(sum(first(), second(), 0))");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "1\n2\n7\n");
}

#[test]
fn bounded_accumulator_and_recursive_generic_calls_keep_results() {
    let case = Case::new("specializations", "accumulator");
    let result = case.run("fn accum(seed: i64) -> i64:\n    let mut acc = seed\n    let mut i = 0\n    while i < 512:\n        acc = (acc * 31 + i) % 1000003\n        i = i + 1\n    acc\nfn recurse(n: i64) -> i64:\n    if n < 1: return 0\n    1 + recurse(n - 1)\nfn main():\n    let mut seed = 0\n    while seed < 8:\n        println(accum(seed))\n        seed = seed + 1\n    println(recurse(4))");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut expected = String::new();
    for seed in 0_i64..8 {
        let mut acc = seed;
        for i in 0..512 {
            acc = (acc * 31 + i) % 1000003;
        }
        expected.push_str(&format!("{acc}\n"));
    }
    expected.push_str("4\n");
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
}
