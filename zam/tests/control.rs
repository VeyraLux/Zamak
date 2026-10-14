mod common;

use common::Case;
use std::{fs, process::Command};

#[test]
fn for_ranges_run_once_per_counter_and_reject_misuse() {
    let case = Case::new("control", "for");
    let result = case.run(include_str!("../../examples/control/loops.zm"));
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout).replace("\r\n", "\n"),
        "45\n2310\n6\n-2\n-1\n0\n12\n13\nrow 1\n21\n23\nrow 2\n31\n32\nrow 3\n9223372036854775805\n9223372036854775806\n"
    );
    // Start before end, each evaluated once; owned values in the body are freed per trip,
    // including on continue, and a returning body leaves the loop early.
    let result = case.run("fn lo() -> i64:\n    println(\"lo\")\n    return 0\nfn hi() -> i64:\n    println(\"hi\")\n    return 3\nfn first(limit: i64) -> i64:\n    for i in 0..limit:\n        if i * i > 5:\n            return i\n    return -1\nfn main():\n    for i in lo()..hi():\n        let line = \"trip {i}\"\n        if i == 1:\n            continue\n        println(line)\n    println(first(10))\n    println(first(2))\n");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&result.stdout).replace("\r\n", "\n"),
        "lo\nhi\ntrip 0\ntrip 2\n3\n-1\n"
    );
    for (source, error) in [
        (
            "fn main():\n    for i in 0..3:\n        i = 1",
            "cannot assign immutable value i",
        ),
        (
            "fn main():\n    let i = 0\n    for i in 0..3: pass",
            "duplicate binding i",
        ),
        (
            "fn main():\n    for i in 0..3: pass\n    println(i)",
            "unknown variable i",
        ),
        ("fn main():\n    for i in 0..\"x\": pass", "type mismatch"),
        ("fn main():\n    for i 0..3: pass", "expected `in`"),
        ("fn main():\n    for i in 3: pass", "expected `..`"),
        (
            "fn main():\n    let s = \"x\"\n    for i in 0..3:\n        let t = s",
            "loop moves",
        ),
        (
            "fn main():\n    let s = \"x\"\n    for i in 0..3:\n        let t = s\n        break",
            "loop jump moves",
        ),
    ] {
        let result = case.run(source);
        assert!(!result.status.success(), "accepted {source}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(error),
            "{source}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn control_flow_and_path_ownership() {
    let root = std::env::temp_dir().join(format!("zam-control-{}", std::process::id()));
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
    fs::write(&file, include_str!("../../examples/control/main.zm")).unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout)
        .ends_with("3628800\n6\nbranch\nfirst\nnext\nnext\nnext\n"));
    for (source, error) in [
        ("fn main():\n    let a = \"x\"\n    if true: let b = a\n    println(a)", "use after move"),
        ("fn main():\n    let a = \"x\"\n    while true: let b = a", "loop moves"),
        ("fn main():\n    if true: let x = 1\n    println(x)", "unknown variable"),
        ("fn main():\n    if 1: pass", "type mismatch"),
        ("fn eat(s: string) -> bool: return true\nfn main():\n    let a = \"x\"\n    let flag = false and eat(a)\n    println(a)", "use after move"),
        ("fn main(): println(1 and true)", "type mismatch"),
        ("fn main():\n    if true: pass println(1)", "newline"),
        ("fn f() -> i64:\n    if true: return 1\nfn main(): pass", "missing"),
        ("fn main():\n    if true: return\n    else: return\n    println(1)", "unreachable"),
        ("fn eat(s: string) -> bool: return true\nfn main():\n    let a = \"x\"\n    while eat(a): pass", "use after move"),
    ] {
        fs::write(&file, source).unwrap(); let result = invoke("check");
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains(error), "{}", String::from_utf8_lossy(&result.stderr));
    }
    fs::write(&file, "fn f(flag: bool):\n    let a = \"live\"\n    if flag:\n        let b = a\n        return\n    println(a)\nfn main():\n    f(false)\n    f(true)\n    if true: let x = 1\n    else: let x = 2\n    let x = 3\n    println(x)").unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).ends_with("live\n3\n"));
    fs::write(&file, "fn probe() -> bool:\n    println(99)\n    return true\nfn main():\n    println(false and (1 / 0 == 0))\n    println(true or (1 / 0 == 0))\n    println(true and probe())\n    println(false or probe())\n    println(true or false and false)\n").unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout)
        .ends_with("false\ntrue\n99\ntrue\n99\ntrue\ntrue\n"));
    fs::write(&file, "fn classify(n: i64) -> string:\n    if n < 0: return \"negative\"\n    elif n == 0: return \"zero\"\n    elif n == 1: return \"one\"\n    else: return \"many\"\nfn show(n: i64):\n    let s = \"live\"\n    if n < 0:\n        let moved = s\n        return\n    elif n == 0:\n        let moved = s\n        return\n    println(s)\nfn main():\n    println(classify(-1))\n    println(classify(0))\n    println(classify(1))\n    println(classify(2))\n    show(-1)\n    show(0)\n    show(2)\n").unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).ends_with("negative\nzero\none\nmany\nlive\n"));
    for (source, error) in [
        (
            "fn main():\n    let s = \"x\"\n    if false: pass\n    elif true: let moved = s\n    println(s)",
            "use after move",
        ),
        (
            "fn f() -> i64:\n    if true: return 1\n    elif false: return 2\nfn main(): pass",
            "missing",
        ),
        ("fn main():\n    if true: pass\n    elif 1: pass", "type mismatch"),
        (
            "fn main():\n    if true: pass\n    else if false: pass",
            "expected `elif` instead of `else if`",
        ),
    ] {
        fs::write(&file, source).unwrap();
        let result = invoke("check");
        assert!(!result.status.success(), "accepted {source}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(error),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    fs::write(&file, "fn main():\n    let mut n = 0\n    while n < 5:\n        n = n + 1\n        if n == 2: continue\n        if n == 4: break\n        println(n)\n").unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).ends_with("1\n3\n"));
    fs::write(&file, "fn main():\n    let mut i = 0\n    let mut outer = \"outer\"\n    while i < 3:\n        let inner = \"inner\"\n        i = i + 1\n        if i == 1: continue\n        println(inner)\n        if i == 2: break\n    println(outer)\n").unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).ends_with("inner\nouter\n"));
    fs::write(&file, include_str!("../../examples/control/jumps.zm")).unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).ends_with("2\n3\n1\n2\n2\n2\n3\n3\n"));
    for source in [
        "fn main(): break",
        "fn main(): continue",
        "fn main():\n    let x = \"x\"\n    while true:\n        let y = x break\n        println(y)",
        "fn main():\n    while true:\n        continue println(1)",
    ] {
        fs::write(&file, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {source}");
    }
    fs::remove_dir_all(root).unwrap();
}

/// 新手常把比较写成 `if score = 60`：报错直接指出要用 `==`，位置指向那个 `=`。
#[test]
fn single_equals_in_condition_suggests_double_equals() {
    let root = std::env::temp_dir().join(format!("zam-control-equals-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let file = root.join("main.zm");
    for (source, keyword, column) in [
        (
            "fn main():\n    let score = 60\n    if score = 60:\n        println(1)\n",
            "if",
            14,
        ),
        (
            "fn main():\n    let mut n = 0\n    while n = 3:\n        n += 1\n",
            "while",
            13,
        ),
        (
            "fn main():\n    let x = 1\n    let s = if x = 1: 2 else: 3\n",
            "if",
            18,
        ),
    ] {
        fs::write(&file, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .args(["check", "--json"])
            .arg(&file)
            .output()
            .unwrap();
        let json = String::from_utf8_lossy(&output.stdout);
        assert!(!output.status.success(), "accepted {source}");
        assert!(
            json.contains(&format!(
                "expected `==` to compare in the {keyword} condition"
            )) && json.contains(&format!("\"line\":3,\"column\":{column}"))
                && json.contains("compare with `==`"),
            "{json}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}
