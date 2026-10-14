use std::{fs, process::Command};

#[test]
fn constant_divisors_preserve_signed_arithmetic() {
    let root = std::env::temp_dir().join(format!("zam-constant-divisors-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let file = root.join("main.zm");
    let divisors = [1_i64, 2, 3, -2, -3, i64::MIN, i64::MAX];
    let numerators = [
        i64::MIN,
        i64::MIN + 1,
        -7,
        -1,
        0,
        1,
        7,
        i64::MAX - 1,
        i64::MAX,
    ];
    let mut source = String::new();
    for (index, divisor) in divisors.iter().enumerate() {
        source.push_str(&format!(
            "fn quotient{index}(value: i64) -> i64: value / {divisor}\nfn remainder{index}(value: i64) -> i64: value % {divisor}\n"
        ));
    }
    source.push_str("fn probe() -> i64:\n    println(99)\n    return -7\nfn main():\n");
    let mut expected = String::new();
    for (index, divisor) in divisors.iter().enumerate() {
        for numerator in numerators {
            source.push_str(&format!(
                "    println(quotient{index}({numerator}))\n    println(remainder{index}({numerator}))\n"
            ));
            expected.push_str(&format!(
                "{}\n{}\n",
                numerator.checked_div(*divisor).unwrap(),
                numerator.checked_rem(*divisor).unwrap()
            ));
        }
    }
    source.push_str("    println(probe() / 2)\n    println(probe() % -3)\n");
    expected.push_str("99\n-3\n99\n-1\n");
    fs::write(&file, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&file)
        .env("ZAMAK_CACHE_DIR", root.join("cache"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn variable_divisors_preserve_width_and_sign_boundaries() {
    let root = std::env::temp_dir().join(format!("zam-variable-divisors-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let file = root.join("main.zm");
    let values = [
        i64::MIN,
        -4294967296,
        -7,
        -1,
        0,
        1,
        3,
        2147483648,
        4294967295,
        4294967296,
        i64::MAX,
    ];
    let mut source = String::from(
        "fn q(a: i64, b: i64) -> i64: a / b\nfn r(a: i64, b: i64) -> i64: a % b\nfn main():\n",
    );
    let mut expected = String::new();
    for a in values {
        for b in values {
            if let (Some(q), Some(r)) = (a.checked_div(b), a.checked_rem(b)) {
                source.push_str(&format!(
                    "    println(q({a}, {b}))\n    println(r({a}, {b}))\n"
                ));
                expected.push_str(&format!("{q}\n{r}\n"));
            }
        }
    }
    fs::write(&file, source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&file)
        .env("ZAMAK_CACHE_DIR", root.join("cache"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn scalar_values_and_checked_arithmetic() {
    let root = std::env::temp_dir().join(format!("zam-values-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let file = root.join("main.zm");
    let invoke = |command: &str| {
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg(command)
            .arg(&file)
            .env("ZAMAK_CACHE_DIR", root.join("cache"))
            .output()
            .unwrap()
    };
    fs::write(&file, "fn twice(x: i64) -> i64: x * 2\nfn yes(x: bool) -> bool: not x\nfn main():\n    let mut n: i64 = 20\n    let copy = n\n    n = twice(n) + 2\n    println(n)\n    println(copy)\n    println(n == 42)\n    println(yes(false))\n    println(-9223372036854775808)\n    println(7 % 3)\n").unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout)
        .ends_with("42\n20\ntrue\ntrue\n-9223372036854775808\n1\n"));
    fs::write(&file, "fn main():\n    println(1_000_000)\n    println(-9_223_372_036_854_775_808)\n    println(9_223_372_036_854_775_807)\n    println(1_2 + 3_4)\n").unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout)
        .ends_with("1000000\n-9223372036854775808\n9223372036854775807\n46\n"));
    for source in [
        "fn main(): println(1_)",
        "fn main(): println(1__2)",
        "fn main(): println(_1)",
        "fn main(): println(9_223_372_036_854_775_808)",
        "fn main(): println(-9_223_372_036_854_775_809)",
        "fn main(): let x: bool = 1",
        "fn main(): println(true + 1)",
        "fn main(): println(9223372036854775808)",
        "fn main(): let x: i64 = \"text\"",
        "fn main(): println(1 == false)",
        "fn f(x: &string): pass\nfn main():\n    let n = 1\n    f(&n)",
    ] {
        fs::write(&file, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {source}");
    }
    fs::write(
        &file,
        "fn sum(a: i64, b: i64) -> i64: a + b\nfn main():\n    println(sum(9223372036854775807, 0))\n    println(sum(-9223372036854775808, 0))\n    println(sum(9223372036854775807, -1))\n    println(sum(-9223372036854775808, 1))\n    println(sum(9223372036854775807, -9223372036854775808))\n    println(sum(-1, 1))\n",
    )
    .unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&result.stdout), "9223372036854775807\n-9223372036854775808\n9223372036854775806\n-9223372036854775807\n-1\n0\n");
    for expression in [
        "9223372036854775807 + 1",
        "-9223372036854775808 + -1",
        "-1 + -9223372036854775808",
        "1 + 9223372036854775807",
        "-9223372036854775808 - 1",
        "9223372036854775807 * 2",
        "-9223372036854775808 * -1",
        "1 / 0",
        "1 % 0",
        "-9223372036854775808 / -1",
        "-9223372036854775808 % -1",
        "-(-9223372036854775808)",
    ] {
        fs::write(&file, format!("fn main(): println({expression})")).unwrap();
        let result = invoke("run");
        assert!(!result.status.success(), "accepted {expression}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("integer arithmetic error"),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    fs::write(&file, include_str!("../../examples/values/arrays.zm")).unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).ends_with("10\n15\n30\n"));
    for source in [
        "fn main(): let a: [i64; 2] = [1]",
        "fn main(): let a: [i64; 0] = []",
        "fn main(): let a: [i64; 1025] = [1]",
        "fn main(): let a: [i64; 2] = [1, true]",
        "fn main():\n    let a = [1, 2]\n    println(a[2])",
        "fn main():\n    let a = [1, 2]\n    println(a[-1])",
        "fn main():\n    let a = [1, 2]\n    a[0] = 3",
        "fn take(a: [i64; 2]): pass\nfn main(): pass",
        "fn make() -> [i64; 2]: return [1, 2]\nfn main(): pass",
        "struct Boxed:\n    values: [i64; 2]\nfn main(): pass",
        "fn main():\n    let a = [1, 2]\n    println(a)",
        "fn main():\n    let mut a = [1, 2]\n    a = [3, 4]",
    ] {
        fs::write(&file, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {source}");
    }
    fs::write(
        &file,
        "fn main():\n    let a = [1, 2]\n    let i = 2\n    println(a[i])",
    )
    .unwrap();
    let result = invoke("run");
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr
            .lines()
            .any(|line| line == "zam: array index out of bounds"),
        "{stderr}"
    );
    assert!(!stderr.contains(r"array index out of bounds\n"), "{stderr}");
    fs::remove_dir_all(root).unwrap();
}
