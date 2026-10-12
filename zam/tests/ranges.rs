use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct Case {
    root: PathBuf,
}
impl Case {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("zam-ranges-{name}-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }
    fn run(&self, source: &str) -> Output {
        let file = self.root.join("main.zm");
        fs::write(&file, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(file)
            .env("ZAMAK_CACHE_DIR", self.root.join("cache"))
            .output()
            .unwrap()
    }
}
impl Drop for Case {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn bounded_loops_short_circuit_and_scope_keep_results() {
    let case = Case::new("bounded");
    let result = case.run(
        r#"
fn probe(n: i64) -> i64 {
    if n >= 0 && n < 4 { return n * 2 }
    if n < 0 || n >= 4 { return -1 }
    return n + 1
}
fn main() {
    let mut values: [i64; 4] = [0, 0, 0, 0]
    let mut i = 0
    while i < 4 {
        values[i] = i * 2
        i = i + 1
    }
    i = 0
    while i < 4 {
        let current = values[i]
        i = i + 1
        if i == 2 { continue }
        println(current)
        if i == 3 { break }
    }
    println(i)
    if i > 0 {
        let temporary = i - 1
        i = temporary + 1
    }
    println(i)
    println(probe(-1))
    println(probe(3))
    let negative = -1
    let past_end = 4
    println(false && values[negative] == 0)
    println(true || values[past_end] == 0)
}
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
    let case = Case::new("checked");
    for source in [
        "fn add(n: i64) -> i64 { n + 1 }\nfn main() { println(add(9223372036854775807)) }",
        "fn main() {\nlet mut n = 9223372036854775806\nwhile n > 0 {\nn = n + 1\n}\n}",
        "fn main() {\nlet mut n = 9223372036854775806\nwhile n <= 9223372036854775807 {\nn = n + 1\n}\n}",
        "fn main() {\nlet mut n = 1\nwhile n > 0 {\nn = n * 2\n}\n}",
        "fn main() {\nlet mut n = 0\nwhile n < 2 {\nn = n + 1\nif n == 1 { n = 9223372036854775807\ncontinue }\n}\nprintln(n + 1)\n}",
        "fn main() {\nlet mut n = 0\nwhile n < 2 {\nn = 9223372036854775807\nbreak\n}\nprintln(n + 1)\n}",
        "fn negate(n: i64) -> i64 { -n }\nfn main() { println(negate(-9223372036854775808)) }",
    ] {
        let result = case.run(source);
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
            "{source}\n{}", String::from_utf8_lossy(&result.stderr));
    }
    for source in [
        "fn read(n: i64) -> i64 {\nlet a = [1, 2]\na[n]\n}\nfn main() { println(read(-1)) }",
        "fn main() {\nlet mut a = [1, 2]\nlet mut i = 0\nwhile i < 3 {\na[i] = i\ni = i + 1\n}\n}",
        "fn main() {\nlet a = [1, 2]\nlet mut i = 0\nwhile i < 3 {\nprintln(a[i])\ni = i + 1\n}\n}",
        "fn main() {\nlet a = [1, 2]\nlet mut i = 0\nwhile i < 2 {\ni = -1\nbreak\n}\nprintln(a[i])\n}",
        "fn main() {\nlet a = [1, 2]\nlet mut i = 0\nwhile i < 2 {\nprintln(a[i])\ni = -1\ncontinue\n}\n}",
    ] {
        let result = case.run(source);
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("array index out of bounds"),
            "{source}\n{}", String::from_utf8_lossy(&result.stderr));
    }
}

#[test]
fn catch_handler_assignments_do_not_reach_success_path() {
    let case = Case::new("catch");
    let result = case.run(
        r#"
fn value(failed: bool) -> i64!string {
    if failed { fail "failed" }
    return 2
}
fn choose(failed: bool) -> i64 {
    let mut n = 3
    let result = value(failed) catch error {
        n = 7
        return n + 1
    }
    return n * result
}
fn main() {
    println(choose(false))
    println(choose(true))
}
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
    let case = Case::new("finite-errors");
    for source in [
        // The final counter step still overflows at an inclusive maximum bound.
        "fn main() {\nlet mut i = 9223372036854775806\nwhile i <= 9223372036854775807 {\ni = i + 1\n}\n}",
        // A safe first iteration cannot authorize an overflowing later visit.
        "fn main() {\nlet mut i = 0\nlet mut x = 9223372036854775806\nwhile i < 2 {\nx = x + 1\ni = i + 1\n}\n}",
        // The loop has zero body iterations, but its bound is still evaluated.
        "fn main() {\nlet mut i = 9223372036854775807\nlet n = 9223372036854775807\nwhile i < n + 1 {\ni = i + 1\n}\n}",
        "fn main() {\nlet mut i = 0\nwhile i < 1 / 0 {\ni = i + 1\n}\n}",
        // A break carries assignments made before the final counter step.
        "fn main() {\nlet mut i = 0\nlet mut x = 0\nwhile i < 2 {\nx = 9223372036854775807\nif i == 0 { break }\ni = i + 1\n}\nprintln(x + 1)\n}",
        // Continue and a changing bound must use the conservative fallback.
        "fn main() {\nlet mut i = 0\nlet mut x = 9223372036854775806\nwhile i < 2 {\nx = x + 1\nif x > 0 { i = i + 1\ncontinue }\ni = i + 1\n}\n}",
        "fn main() {\nlet mut bound = 1\nlet mut i = 0\nlet mut x = 9223372036854775806\nwhile i < bound {\nx = x + 1\nbound = bound + 1\ni = i + 1\n}\n}",
        "fn remainder(n: i64) -> i64 { n % 0 }\nfn main() { println(remainder(1)) }",
        "fn remainder(n: i64) -> i64 { n % -1 }\nfn main() { println(remainder(-9223372036854775808)) }",
    ] {
        let result = case.run(source);
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
            "{source}\n{}", String::from_utf8_lossy(&result.stderr));
    }
}

#[test]
fn finite_recurrence_unknown_bound_breaks_and_returns_keep_results() {
    let case = Case::new("finite-results");
    let result = case.run(
        r#"
fn sequence(seed: i64) -> i64 {
    let bound = seed % 40
    let mut i = 0
    let mut a = 0
    let mut b = 1
    while i < bound {
        let next = a + b
        a = b
        b = next
        i = i + 1
    }
    return a
}
fn early(stop: i64) -> i64 {
    let mut i = 0
    let mut total = 0
    while i <= 4 {
        total = total + 3
        if i == stop { break }
        if i == 2 && stop < 0 { return total }
        i = i + 2
    }
    return total + i
}
fn varying_start(start: i64) -> i64 {
    if start >= 0 && start < 4 {
        let mut i = start
        let mut x = 1
        while i < 4 {
            x = x * 2
            i = i + 1
        }
        return x
    }
    return 0
}
fn main() {
    println(varying_start(0))
    println(varying_start(3))
    println(sequence(-1))
    println(sequence(0))
    println(sequence(39))
    println(sequence(79))
    println(early(2))
    println(early(9))
    println(early(-1))
}
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
