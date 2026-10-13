use std::{fs, process::Command};

#[test]
fn syntax_diagnostics_include_source_and_line() {
    let path = std::env::temp_dir().join(format!("zam-diagnostics-{}.zm", std::process::id()));
    for (source, line, column, message) in [
        (
            "# comment\nfn main() {\nprintln(@)\n}\n",
            3,
            9,
            "unexpected character",
        ),
        (
            "fn main() {\nprintln(\"bad\\q\")\n}\n",
            2,
            9,
            "unsupported string escape",
        ),
        (
            "fn main() {\nprintln(\"open",
            2,
            9,
            "unterminated string literal",
        ),
        (
            "fn main() {\nprintln(\"你好\")\nprintln(1\n}\n",
            3,
            10,
            "expected",
        ),
        ("fn main() {\nprintln(1)\n", 3, 1, "unterminated"),
        (
            "fn main() { println(\"你好\") @ }",
            1,
            27,
            "unexpected character",
        ),
        (
            "fn main() {\r\n\tprintln(@)\r\n}",
            2,
            10,
            "unexpected character",
        ),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&path)
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success());
        assert!(error.contains(&path.display().to_string()), "{error}");
        assert!(
            error.contains(&format!("line {line}, column {column}:")),
            "{error}"
        );
        assert!(error.contains(message), "{error}");
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn semantic_diagnostics_include_line_and_column() {
    let path = std::env::temp_dir().join(format!("zam-semantic-{}.zm", std::process::id()));
    for (source, line, column, message) in [
        (
            "fn main() {\n    let a = \"x\"\n    let b = a\n    println(a)\n}\n",
            4,
            5,
            "use after move",
        ),
        (
            "fn main() {\n    let a = \"x\"\n    if true {\n        let b = a\n        println(a)\n    }\n}\n",
            5,
            9,
            "use after move",
        ),
        (
            "fn f() -> string {}\nfn main() {}\n",
            1,
            1,
            "missing string return value",
        ),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&path)
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "accepted {source}");
        assert!(
            error.contains(&format!("line {line}, column {column}:")),
            "{source}: {error}"
        );
        assert!(error.contains(message), "{source}: {error}");
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn json_diagnostics_are_one_line_and_structured() {
    let path = std::env::temp_dir().join(format!("zam-json-{}.zm", std::process::id()));
    let escaped = path.display().to_string().replace('\\', "\\\\");
    fs::write(&path, "fn main() {\nprintln(@)\n}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["check", "--json"])
        .arg(&path)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(!output.status.success());
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    for field in [
        "\"command\":\"check\"",
        "\"ok\":false",
        &format!("\"file\":\"{escaped}\""),
        "\"line\":2",
        "\"column\":9",
        "\"function\":null",
        "\"code\":\"ZM100\"",
        "\"category\":\"syntax\"",
        "\"message\":\"unexpected character '@'\"",
        "\"suggestion\":\"remove or replace the unsupported character\"",
    ] {
        assert!(stdout.contains(field), "{field} missing from {stdout}");
    }
    // 人类可读的那一行仍然在 stderr，stdout 保持是合法 JSON。
    assert!(String::from_utf8_lossy(&output.stderr).contains(&path.display().to_string()));

    fs::write(&path, "fn main() {\nprintln(1)\n}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["check", "--json"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "{\"command\":\"check\",\"ok\":true,\"diagnostics\":[]}"
    );

    // 不支持 --json 的命令必须报错，而不是静默忽略。
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["run", "--json"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--json is only supported"));

    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["check", "--json", "--json"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("may be specified once"));
    fs::remove_file(path).unwrap();
}

/// 每段都是真实源码：必须被 `check` 拒绝，并归到表里那一个诊断码。
/// 只断言单元测试里造出来的字符串不够——`classify` 匹配的是前端真实文案。
#[test]
fn json_codes_classify_real_sources() {
    let path = std::env::temp_dir().join(format!("zam-classify-{}.zm", std::process::id()));
    for (source, code, category) in [
        (
            "fn main() {\n    let x: i64 = \"s\"\n}\n",
            "ZM200",
            "type",
        ),
        ("fn main() {\n    println(nope)\n}\n", "ZM201", "name"),
        ("fn main() {\n    nope()\n}\n", "ZM201", "name"),
        (
            "struct Point {\n    x: i64\n    y: i64\n}\nfn main() {\n    let p = Point {\n        x: 1\n        y: 2\n    }\n    println(p.z)\n}\n",
            "ZM202",
            "name",
        ),
        (
            "fn main() {\n    let a = \"x\"\n    let b = a\n    println(a)\n}\n",
            "ZM301",
            "ownership",
        ),
        (
            "fn main() {\n    let x = 1\n    x = 2\n}\n",
            "ZM302",
            "ownership",
        ),
        (
            "fn f(a: &string, b: string) -> i64 {\n    return 1\n}\nfn main() {\n    let s = \"x\"\n    f(&s, s)\n}\n",
            "ZM421",
            "ownership",
        ),
        (
            "fn main() {\n    let s = \"x\"\n    let r = &s\n}\n",
            "ZM422",
            "ownership",
        ),
        (
            "fn f(a: &mut string) -> i64 {\n    return 1\n}\nfn main() {\n    let mut s = \"x\"\n    f(&s)\n}\n",
            "ZM423",
            "ownership",
        ),
        (
            "fn f(a: &mut string) -> i64 {\n    return 1\n}\nfn main() {\n    let s = \"x\"\n    f(&mut s)\n}\n",
            "ZM424",
            "ownership",
        ),
        (
            "fn f(a: &mut string) -> i64 {\n    return 1\n}\nfn main() {\n    let s = \"x\"\n    f(s)\n}\n",
            "ZM425",
            "ownership",
        ),
        (
            "fn f() -> string {}\nfn main() {}\n",
            "ZM510",
            "control_flow",
        ),
        (
            "fn main() {\n    let x: f64 = 1\n}\n",
            "ZM900",
            "unsupported",
        ),
        (
            "fn main() {\n    println(\"a\\qb\")\n}\n",
            "ZM101",
            "syntax",
        ),
        (
            "fn main() {\n    let x = 9223372036854775808\n    println(x)\n}\n",
            "ZM600",
            "arithmetic",
        ),
        (
            "fn main() {\n    let a = [1, 2, 3]\n    println(a[5])\n}\n",
            "ZM406",
            "bounds",
        ),
        (
            "fn load(p: &string) -> i64!string {\n    return 1\n}\nfn main() {\n    let v = load(\"a\")\n    println(v)\n}\n",
            "ZM500",
            "error_handling",
        ),
        (
            "fn f() -> i64!i64 {\n    return 1\n}\nfn main() {\n    println(1)\n}\n",
            "ZM509",
            "error_handling",
        ),
        (
            "fn main() {\n    println(\"hi\");\n}\n",
            "ZM110",
            "syntax",
        ),
        (
            "fn foo() -> i64 {\n    return 1\n}\n\nfn foo() -> i64 {\n    return 2\n}\n\nfn main() {\n    println(foo())\n}\n",
            "ZM204",
            "name",
        ),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .args(["check", "--json"])
            .arg(&path)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(!output.status.success(), "accepted {source}");
        assert_eq!(stdout.lines().count(), 1, "{source}: {stdout}");
        for field in [
            format!("\"code\":\"{code}\""),
            format!("\"category\":\"{category}\""),
            "\"suggestion\":\"".to_string(),
        ] {
            assert!(stdout.contains(&field), "{source}: {field} missing from {stdout}");
        }
        assert!(
            !stdout.contains("\"suggestion\":null"),
            "{source}: missing suggestion in {stdout}"
        );
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn build_json_keeps_progress_off_stdout() {
    let root = std::env::temp_dir().join(format!("zam-json-build-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("main.zm");
    fs::write(&path, "fn main() {\nprintln(1)\n}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["build", "--json"])
        .arg(&path)
        .env("ZAMAK_CACHE_DIR", root.join("cache"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(
        stdout.trim(),
        "{\"command\":\"build\",\"ok\":true,\"diagnostics\":[]}"
    );
    // CACHE/OBJECT/LINK/OUTPUT 进度挪到了 stderr，stdout 才可能是合法 JSON。
    assert!(String::from_utf8_lossy(&output.stderr).contains("CACHE MISS"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("OUTPUT "));
    fs::remove_dir_all(root).unwrap();
}

/// 模块解析期的错误（例如缺少 `use std/string`）过去只有 message，`file`/`line`/`column`
/// 全为 null，AI 修复循环只能靠正则抠模块名。位置取自 `Block` 里已有的指令位置，粒度与
/// 类型检查器一致（指向语句起始）；项目模式下 `file` 也从项目目录收紧到出错的那个模块。
#[test]
fn module_resolution_errors_include_location() {
    let source = "fn main() {\n    let s = \"hi\"\n    println(string.byte_len(&s))\n}\n";
    let root = std::env::temp_dir().join(format!("zam-module-location-{}", std::process::id()));
    fs::create_dir_all(root.join("src")).unwrap();
    let file = root.join("main.zm");
    let module = root.join("src").join("main.zm");
    fs::write(&file, source).unwrap();
    fs::write(&module, source).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"probe\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();

    for (input, reported) in [(&file, &file), (&root, &module)] {
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .args(["check", "--json"])
            .arg(input)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(!output.status.success(), "{stdout}");
        assert_eq!(stdout.lines().count(), 1, "{stdout}");
        let escaped = reported.display().to_string().replace('\\', "\\\\");
        for field in [
            format!("\"file\":\"{escaped}\""),
            "\"line\":3".to_string(),
            "\"column\":5".to_string(),
            "\"code\":\"ZM203\"".to_string(),
            "\"category\":\"name\"".to_string(),
        ] {
            assert!(stdout.contains(&field), "{field} missing from {stdout}");
        }
    }
    fs::remove_dir_all(root).unwrap();
}

/// 类型检查期的错误由 `compiler::check` 产生，message 里带的是链接后的 `模块:函数`、不含
/// 文件路径；项目模式下 `main.rs` 因此曾把 `file` 退化成项目目录。这里断言它补回了真正
/// 出错模块的路径，与解析期错误保持一致。
#[test]
fn project_check_errors_report_module_path() {
    let root = std::env::temp_dir().join(format!("zam-check-location-{}", std::process::id()));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"probe\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src").join("main.zm"),
        "use greeting\n\nfn main() {\n    println(greeting.hello())\n}\n",
    )
    .unwrap();
    let module = root.join("src").join("greeting.zm");
    fs::write(&module, "pub fn hello() -> string {\n    return 1\n}\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["check", "--json"])
        .arg(&root)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(!output.status.success(), "{stdout}");
    assert_eq!(stdout.lines().count(), 1, "{stdout}");
    let escaped = module.display().to_string().replace('\\', "\\\\");
    for field in [
        format!("\"file\":\"{escaped}\""),
        "\"line\":2".to_string(),
        "\"column\":5".to_string(),
        "\"function\":\"greeting:hello\"".to_string(),
        "\"code\":\"ZM200\"".to_string(),
        "\"category\":\"type\"".to_string(),
    ] {
        assert!(stdout.contains(&field), "{field} missing from {stdout}");
    }
    fs::remove_dir_all(root).unwrap();
}
