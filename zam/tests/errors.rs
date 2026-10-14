use std::{fs, process::Command};

#[test]
fn error_values() {
    let path = std::env::temp_dir().join(format!("zam-errors-{}.zm", std::process::id()));
    let invoke = |command| {
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg(command)
            .arg(&path)
            .output()
            .unwrap()
    };
    let example = include_str!("../../examples/values/errors.zm");
    let (declarations, main) = example.split_once("fn main()").unwrap();
    let success = format!("{declarations}fn main(){main}");
    // 失败版：把入口的文本换成空串，让错误穿过两层 `?` 到 main。
    let failure = success.replace("\"42\"", "\"\"");
    // 成功：`?` 把值一路带出来，main 正常退出。
    fs::write(&path, &success).unwrap();
    let output = invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("42\n"));
    // 失败：未捕获的错误写 stderr 并以 1 退出。
    fs::write(&path, &failure).unwrap();
    let output = invoke("run");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("empty input"));
    // 注意 `zam run` 把缓存进度（含哈希）也打在 stdout，这里只按行比对程序自己的输出。
    assert!(!String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.trim() == "42"));
    // `?` 与 `fail` 的规则，以及成功值类型仍然是 T。
    for (source, because) in [
        (
            format!("{declarations}fn main():\n    let text = \"42\"\n    let value = parse(&text)\n    println(value)"),
            "fallible call without ?",
        ),
        (
            format!("{declarations}fn main():\n    let text = \"42\"\n    let value = parse(&text)?\n    println(value)"),
            "? in a function without an error type",
        ),
        (
            format!("{declarations}fn main() -> !string:\n    let text = \"42\"\n    let flag: bool = parse(&text)?\n    println(flag)"),
            "success value is still i64",
        ),
        (
            "fn double(n: i64) -> i64:\n    return n\nfn main() -> !string:\n    let value = double(1)?\n    println(value)"
                .to_string(),
            "? on a call that cannot fail",
        ),
        (
            "fn main():\n    fail \"nope\"".to_string(),
            "fail in a function without an error type",
        ),
        (
            "fn f() -> i64!i64:\n    fail 1\nfn main() -> !i64:\n    let value = f()?\n    println(value)"
                .to_string(),
            "non-string error type",
        ),
    ] {
        fs::write(&path, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {because}");
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn catch_blocks() {
    let path = std::env::temp_dir().join(format!("zam-catch-{}.zm", std::process::id()));
    let invoke = |command| {
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg(command)
            .arg(&path)
            .output()
            .unwrap()
    };
    let example = include_str!("../../examples/values/catch.zm");
    let declarations = example.split_once("fn main()").unwrap().0;
    // 成功：catch 不触发，值照常带出来。
    fs::write(&path, example).unwrap();
    let output = invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("42\n"));
    // 失败：错误值绑定到 e，打印后从 main 返回，退出码仍是 0（错误已被处理）。
    let failure = example.replace("\"42\"", "\"\"");
    fs::write(&path, &failure).unwrap();
    let output = invoke("run");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("failed: empty input"));
    assert!(!String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.trim() == "42"));
    // catch 的规则。
    for (source, because) in [
        (
            format!(
                "{declarations}fn main():\n    let text = \"42\"\n    let value = parse(&text) catch e:\n        println(\"failed: {{e}}\")\n    println(value)"
            ),
            "catch block without return",
        ),
        (
            "fn double(n: i64) -> i64:\n    return n\nfn main():\n    let value = double(1) catch e:\n        println(\"nope\")\n        return\n    println(value)"
                .to_string(),
            "catch on a call that cannot fail",
        ),
        (
            format!(
                "{declarations}fn main():\n    let e = \"taken\"\n    let text = \"42\"\n    let value = parse(&text) catch e:\n        println(\"failed: {{e}}\")\n        return\n    println(value)"
            ),
            "catch binding shadows a live local",
        ),
    ] {
        fs::write(&path, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {because}");
    }
    fs::remove_file(path).unwrap();
}
