use std::{fs, path::Path, process::Command};

fn zam(args: &[&str], cache: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(args)
        .env("ZAMAK_CACHE_DIR", cache)
        .output()
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

#[test]
fn zam_test_runs_each_test_in_its_own_process() {
    let root = std::env::temp_dir().join(format!("zam-testing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let cache = root.join("cache");

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/testing");
    let output = zam(&["test", example.to_str().unwrap()], &cache);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout(&output),
        "test stats.test_largest ... ok\ntest stats.test_mean ... ok\ntest stats.test_mean_of_empty_list_fails ... ok\n\ntest result: ok. 3 passed; 0 failed\n"
    );
    // 测试模式不影响普通运行：main 照常执行。
    let output = zam(&["run", example.to_str().unwrap()], &cache);
    assert_eq!(stdout(&output), "3.0\n");

    // 失败的 assert 显示条件原文、说明和两边的值；运行时错误与 `?` 传出的错误都算失败，
    // 一个测试崩溃不影响其他测试；有失败时退出码非零。
    let file = root.join("checks.zm");
    fs::write(
        &file,
        "use std/string\n\nfn test_pass():\n    assert 1 + 1 == 2\n\nfn test_assert():\n    let got = 2 + 2\n    assert got == 5, \"two plus two\"\n\nfn test_crash():\n    let xs: [i64] = []\n    println(xs[3])\n\nfn test_error() -> !string:\n    let n = string.parse_i64(\"x\")?\n    assert n == 0\n\nfn test_output():\n    println(\"shown only on failure\")\n    assert false\n",
    )
    .unwrap();
    let output = zam(&["test", file.to_str().unwrap()], &cache);
    assert!(!output.status.success());
    let text = stdout(&output);
    for line in [
        "test test_assert ... FAILED",
        "test test_crash ... FAILED",
        "test test_error ... FAILED",
        "test test_output ... FAILED",
        "test test_pass ... ok",
        ":8: assertion failed: got == 5: two plus two\n  left:  4\n  right: 5",
        "zam: list index out of bounds",
        "invalid integer: x",
        "shown only on failure",
        ":20: assertion failed: false",
        "test result: FAILED. 1 passed; 4 failed",
    ] {
        assert!(text.contains(line), "missing {line:?} in:\n{text}");
    }
    assert!(String::from_utf8_lossy(&output.stderr).contains("4 test(s) failed"));

    // 过滤：名字里含这一段的测试才运行。
    let output = zam(&["test", file.to_str().unwrap(), "pass"], &cache);
    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "test test_pass ... ok\n\ntest result: ok. 1 passed; 0 failed\n"
    );

    // 没有测试；测试函数签名不对。
    let empty = root.join("empty.zm");
    fs::write(&empty, "fn main(): pass\n").unwrap();
    let output = zam(&["test", empty.to_str().unwrap()], &cache);
    assert!(output.status.success());
    assert!(stdout(&output).contains("no tests found"));
    let bad = root.join("bad.zm");
    fs::write(&bad, "fn test_bad(x: i64): pass\n").unwrap();
    let output = zam(&["test", bad.to_str().unwrap()], &cache);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must take no parameters"));

    // assert 在普通程序里也能用：失败时写 stderr、退出码 1。
    let program = root.join("program.zm");
    fs::write(&program, "fn main():\n    let xs: [i64] = [1, 2]\n    assert xs[1] == 3\n    println(\"unreachable\")\n").unwrap();
    let output = zam(&["run", program.to_str().unwrap()], &cache);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
    assert!(
        stderr.contains("assertion failed: xs[1] == 3\n  left:  2\n  right: 3"),
        "{stderr}"
    );
    fs::remove_dir_all(&root).unwrap();
}
