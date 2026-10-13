mod common;

use common::Case;

#[test]
fn repeated_call_site_joins_all_visits_before_removing_checks() {
    let case = Case::new("specializations", "joined-overflow");
    let result = case.run("fn add(n: i64) -> i64 { n + 1 }\nfn main() {\nlet mut n = 0\nlet mut i = 0\nwhile i < 2 {\nprintln(add(n))\nn = 9223372036854775807\ni = i + 1\n}\n}");
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
    let result = case.run("fn rem(n: i64) -> i64 { (n * 31 + 1) % 7 }\nfn unknown() -> i64 { -5 }\nfn nested(n: i64) -> i64 { rem(n) }\nfn main() {\nprintln(rem(-5))\nprintln(rem(unknown()))\nprintln(nested(-5))\n}");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "0\n0\n0\n");
    let remainder_inputs = case.run("fn twice(n: i64) -> i64 { n * 2 }\nfn main() {\nlet mut n = -9\nwhile n < 0 {\nprintln(twice(n % 8))\nn = n + 1\n}\n}");
    assert!(
        remainder_inputs.status.success(),
        "{}",
        String::from_utf8_lossy(&remainder_inputs.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&remainder_inputs.stdout),
        "-2\n0\n-14\n-12\n-10\n-8\n-6\n-4\n-2\n"
    );
    let failed = case.run("fn unknown() -> i64 { 9223372036854775807 }\nfn add(n: i64) -> i64 { n + 1 }\nfn main() { println(add(unknown())) }");
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("integer arithmetic error"));
}

#[test]
fn specialized_call_evaluates_arguments_in_original_order() {
    let case = Case::new("specializations", "argument-order");
    let result = case.run("fn first() -> i64 {\nprintln(1)\n3\n}\nfn second() -> i64 {\nprintln(2)\n4\n}\nfn sum(a: i64, b: i64, seed: i64) -> i64 { a + b + seed }\nfn main() { println(sum(first(), second(), 0)) }");
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
    let result = case.run("fn accum(seed: i64) -> i64 {\nlet mut acc = seed\nlet mut i = 0\nwhile i < 512 {\nacc = (acc * 31 + i) % 1000003\ni = i + 1\n}\nacc\n}\nfn recurse(n: i64) -> i64 {\nif n < 1 { return 0 }\n1 + recurse(n - 1)\n}\nfn main() {\nlet mut seed = 0\nwhile seed < 8 {\nprintln(accum(seed))\nseed = seed + 1\n}\nprintln(recurse(4))\n}");
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
