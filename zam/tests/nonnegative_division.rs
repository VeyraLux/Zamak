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
        let root = std::env::temp_dir().join(format!(
            "zam-nonnegative-division-{name}-{}",
            std::process::id()
        ));
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
fn positive_variable_boundaries_match_checked_signed_arithmetic() {
    let case = Case::new("boundaries");
    let mut source = String::from("fn main() {\n");
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
            "let n{i} = {n}\nlet d{i} = {d}\nprintln(n{i} / d{i})\nprintln(n{i} % d{i})\nprintln(n{i} / {d})\nprintln(n{i} % {d})\n"
        ));
        expected.push_str(&format!(
            "{}\n{}\n{}\n{}\n",
            n.checked_div(d).unwrap(),
            n.checked_rem(d).unwrap(),
            n.checked_div(d).unwrap(),
            n.checked_rem(d).unwrap()
        ));
    }
    source.push_str("}\n");
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
    let case = Case::new("signed");
    let result = case.run("fn quotient(n: i64, d: i64) -> i64 { n / d }\nfn remainder(n: i64, d: i64) -> i64 { n % d }\nfn unknown() -> i64 { -9 }\nfn main() {\nprintln(quotient(unknown(), 4))\nprintln(remainder(unknown(), 4))\nprintln(quotient(9, -4))\nprintln(remainder(9, -4))\nlet mut n = 9\nlet mut i = 0\nwhile i < 2 {\nprintln(n / 4)\nprintln(n % 4)\nn = -9\ni = i + 1\n}\n}");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout),
        "-2\n-1\n-2\n1\n2\n1\n-2\n-1\n"
    );
}

#[test]
fn zero_and_signed_overflow_keep_arithmetic_errors() {
    let case = Case::new("errors");
    for op in ["/", "%"] {
        for (n, d) in [(1, 0), (i64::MIN, -1)] {
            let source = format!("fn main() {{\nlet n = {n}\nlet d = {d}\nprintln(n {op} d)\n}}");
            let result = case.run(&source);
            assert!(!result.status.success(), "accepted {source}");
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}

#[test]
fn operand_order_and_successful_result_facts_keep_operand_errors() {
    let case = Case::new("operand-order");
    for (op, answer) in [("/", "2"), ("%", "1")] {
        let source = format!("fn first() -> i64 {{\nprintln(1)\n9\n}}\nfn second() -> i64 {{\nprintln(2)\n4\n}}\nfn main() {{ println(first() {op} second()) }}");
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
        let source = format!("fn unknown() -> i64 {{ 9223372036854775807 }}\nfn second() -> i64 {{\nprintln(2)\n4\n}}\nfn main() {{\nlet n = unknown()\nif n >= 0 {{ println((n + 1) {op} (second() * 0 + 4)) }}\n}}");
        let result = case.run(&source);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "");
        let source = format!("fn unknown() -> i64 {{ 9223372036854775807 }}\nfn first() -> i64 {{\nprintln(1)\n9\n}}\nfn main() {{\nlet n = unknown()\nif n >= 0 {{ println((first() * 0) {op} (n + 1)) }}\n}}");
        let result = case.run(&source);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"));
        assert_eq!(String::from_utf8_lossy(&result.stdout), "1\n");
    }
}

#[test]
fn specialized_nonnegative_recurrence_keeps_exact_results() {
    let case = Case::new("recurrence");
    let result = case.run("fn accum(seed: i64) -> i64 {\nlet mut acc = seed\nlet mut i = 0\nwhile i < 512 {\nacc = (acc * 31 + i) % 1000003\ni = i + 1\n}\nacc\n}\nfn main() {\nlet mut seed = 0\nwhile seed < 8 {\nprintln(accum(seed))\nseed = seed + 1\n}\n}");
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
