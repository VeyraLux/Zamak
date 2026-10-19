//! 函数里定义的 fn 支持递归：函数体里可以直接调用自己。

use std::{fs, process::Command};

fn invoke(root: &std::path::Path, file: &std::path::Path, command: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg(command)
        .arg(file)
        .env("ZAMAK_CACHE_DIR", root.join("cache"))
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap()
}

/// 递归的嵌套 fn：带一个参数、带累加参数、捕获外层变量、两个函数互相调用。
#[test]
fn functions_declared_inside_a_function_can_call_themselves() {
    let root = std::env::temp_dir().join(format!("zam-local-recursion-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let file = root.join("main.zm");

    for (name, source, expected) in [
        (
            "factorial",
            "fn main():\n    fn fact(n: i64) -> i64:\n        if n < 2: return 1\n        return n * fact(n - 1)\n    println(fact(5))\n",
            "120\n",
        ),
        (
            "fibonacci",
            "fn main():\n    fn fib(n: i64) -> i64:\n        if n < 2: return n\n        return fib(n - 1) + fib(n - 2)\n    println(fib(10))\n",
            "55\n",
        ),
        (
            "accumulator",
            "fn main():\n    fn sum_to(n: i64, acc: i64) -> i64:\n        if n < 1: return acc\n        return sum_to(n - 1, acc + n)\n    println(sum_to(5, 0))\n",
            "15\n",
        ),
        (
            // 递归之外还读了外层变量：捕获与递归同时成立。
            "captured",
            "fn main():\n    let base = 100\n    fn add(n: i64) -> i64:\n        if n < 1: return 0\n        return base + n + add(n - 1)\n    println(add(3))\n",
            "306\n",
        ),
        (
            // 递归 + 返回字符串：返回值也走同一套。
            "strings",
            "fn main():\n    fn stars(n: i64) -> string:\n        if n < 1: return \"\"\n        return \"*\" + stars(n - 1)\n    println(stars(4))\n",
            "****\n",
        ),
    ] {
        fs::write(&file, source).unwrap();
        let checked = invoke(&root, &file, "check");
        assert!(
            checked.status.success(),
            "{name}: check failed:\n{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        let output = invoke(&root, &file, "run");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "{name}: run failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        // `zam run` 把构建进度写在 stderr，stdout 只有程序输出。
        assert_eq!(stdout, expected, "{name}");
    }

    // 递归调用自己但参数类型不对：照常按函数类型检查报错，不会因为“自己”而放过。
    fs::write(
        &file,
        "fn main():\n    fn double(n: i64) -> i64:\n        return double(\"x\")\n    println(double(2))\n",
    )
    .unwrap();
    let broken = invoke(&root, &file, "check");
    assert!(
        !broken.status.success(),
        "accepted a recursive call with the wrong argument type"
    );
    let stderr = String::from_utf8_lossy(&broken.stderr);
    assert!(stderr.contains("type mismatch"), "{stderr}");

    // 递归还要有返回类型标注：推不出来时要能指出。
    fs::write(
        &file,
        "fn main():\n    fn me(n: i64) -> i64:\n        return me(n - 1) + 1\n    println(me(2))\n",
    )
    .unwrap();
    assert!(
        invoke(&root, &file, "check").status.success(),
        "typed recursion should work"
    );

    fs::remove_dir_all(root).unwrap();
}
