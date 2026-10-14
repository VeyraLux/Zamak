mod common;

use common::Case;

#[test]
fn bounded_loops_short_circuit_and_scope_keep_results() {
    let case = Case::new("ranges", "bounded");
    let result = case.run(
        r#"
fn probe(n: i64) -> i64:
    if n >= 0 and n < 4: return n * 2
    if n < 0 or n >= 4: return -1
    return n + 1
fn main():
    let mut values: [i64; 4] = [0, 0, 0, 0]
    let mut i = 0
    while i < 4:
        values[i] = i * 2
        i = i + 1
    i = 0
    while i < 4:
        let current = values[i]
        i = i + 1
        if i == 2: continue
        println(current)
        if i == 3: break
    println(i)
    if i > 0:
        let temporary = i - 1
        i = temporary + 1
    println(i)
    println(probe(-1))
    println(probe(3))
    let negative = -1
    let past_end = 4
    println(false and values[negative] == 0)
    println(true or values[past_end] == 0)
"#,
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "0\n4\n3\n3\n-1\n6\nfalse\ntrue\n"
    );
}

#[test]
fn loop_backedges_and_unknown_parameters_preserve_errors() {
    let case = Case::new("ranges", "checked");
    for source in [
        "fn add(n: i64) -> i64: n + 1\nfn main(): println(add(9223372036854775807))",
        "fn main():\n    let mut n = 9223372036854775806\n    while n > 0:\n        n = n + 1",
        "fn main():\n    let mut n = 9223372036854775806\n    while n <= 9223372036854775807:\n        n = n + 1",
        "fn main():\n    let mut n = 1\n    while n > 0:\n        n = n * 2",
        "fn main():\n    let mut n = 0\n    while n < 2:\n        n = n + 1\n        if n == 1:\n            n = 9223372036854775807\n            continue\n    println(n + 1)",
        "fn main():\n    let mut n = 0\n    while n < 2:\n        n = 9223372036854775807\n        break\n    println(n + 1)",
        "fn negate(n: i64) -> i64: -n\nfn main(): println(negate(-9223372036854775808))",
    ] {
        let result = case.run(source);
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
            "{source}\n{}", String::from_utf8_lossy(&result.stderr));
    }
    for source in [
        "fn read(n: i64) -> i64:\n    let a = [1, 2]\n    a[n]\nfn main(): println(read(-1))",
        "fn main():\n    let mut a = [1, 2]\n    let mut i = 0\n    while i < 3:\n        a[i] = i\n        i = i + 1",
        "fn main():\n    let a = [1, 2]\n    let mut i = 0\n    while i < 3:\n        println(a[i])\n        i = i + 1",
        "fn main():\n    let a = [1, 2]\n    let mut i = 0\n    while i < 2:\n        i = -1\n        break\n    println(a[i])",
        "fn main():\n    let a = [1, 2]\n    let mut i = 0\n    while i < 2:\n        println(a[i])\n        i = -1\n        continue",
    ] {
        let result = case.run(source);
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("array index out of bounds"),
            "{source}\n{}", String::from_utf8_lossy(&result.stderr));
    }
}

#[test]
fn catch_handler_assignments_do_not_reach_success_path() {
    let case = Case::new("ranges", "catch");
    let result = case.run(
        r#"
fn value(failed: bool) -> i64!string:
    if failed: fail "failed"
    return 2
fn choose(failed: bool) -> i64:
    let mut n = 3
    let result = value(failed) catch error:
        n = 7
        return n + 1
    return n * result
fn main():
    println(choose(false))
    println(choose(true))
"#,
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "6\n8\n");
}

#[test]
fn finite_loop_boundaries_and_remainder_keep_checked_errors() {
    let case = Case::new("ranges", "finite-errors");
    for source in [
        // The final counter step still overflows at an inclusive maximum bound.
        "fn main():\n    let mut i = 9223372036854775806\n    while i <= 9223372036854775807:\n        i = i + 1",
        // A safe first iteration cannot authorize an overflowing later visit.
        "fn main():\n    let mut i = 0\n    let mut x = 9223372036854775806\n    while i < 2:\n        x = x + 1\n        i = i + 1",
        // The loop has zero body iterations, but its bound is still evaluated.
        "fn main():\n    let mut i = 9223372036854775807\n    let n = 9223372036854775807\n    while i < n + 1:\n        i = i + 1",
        "fn main():\n    let mut i = 0\n    while i < 1 / 0:\n        i = i + 1",
        // A break carries assignments made before the final counter step.
        "fn main():\n    let mut i = 0\n    let mut x = 0\n    while i < 2:\n        x = 9223372036854775807\n        if i == 0: break\n        i = i + 1\n    println(x + 1)",
        // Continue and a changing bound must use the conservative fallback.
        "fn main():\n    let mut i = 0\n    let mut x = 9223372036854775806\n    while i < 2:\n        x = x + 1\n        if x > 0:\n            i = i + 1\n            continue\n        i = i + 1",
        "fn main():\n    let mut bound = 1\n    let mut i = 0\n    let mut x = 9223372036854775806\n    while i < bound:\n        x = x + 1\n        bound = bound + 1\n        i = i + 1",
        "fn remainder(n: i64) -> i64: n % 0\nfn main(): println(remainder(1))",
        "fn remainder(n: i64) -> i64: n % -1\nfn main(): println(remainder(-9223372036854775808))",
    ] {
        let result = case.run(source);
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
            "{source}\n{}", String::from_utf8_lossy(&result.stderr));
    }
}

#[test]
fn finite_recurrence_unknown_bound_breaks_and_returns_keep_results() {
    let case = Case::new("ranges", "finite-results");
    let result = case.run(
        r#"
fn sequence(seed: i64) -> i64:
    let bound = seed % 40
    let mut i = 0
    let mut a = 0
    let mut b = 1
    while i < bound:
        let next = a + b
        a = b
        b = next
        i = i + 1
    return a
fn early(stop: i64) -> i64:
    let mut i = 0
    let mut total = 0
    while i <= 4:
        total = total + 3
        if i == stop: break
        if i == 2 and stop < 0: return total
        i = i + 2
    return total + i
fn varying_start(start: i64) -> i64:
    if start >= 0 and start < 4:
        let mut i = start
        let mut x = 1
        while i < 4:
            x = x * 2
            i = i + 1
        return x
    return 0
fn main():
    println(varying_start(0))
    println(varying_start(3))
    println(sequence(-1))
    println(sequence(0))
    println(sequence(39))
    println(sequence(79))
    println(early(2))
    println(early(9))
    println(early(-1))
"#,
    );
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "16\n2\n0\n0\n63245986\n63245986\n8\n15\n6\n"
    );
}
