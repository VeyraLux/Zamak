use std::{fs, process::Command};

#[test]
fn invalid_arguments_fail_before_loading_or_writing_projects() {
    for args in [
        vec!["check", "--unknown"],
        vec!["build", "--unknown"],
        vec!["fmt", "--unknown"],
        vec!["run", "--unknown"],
        vec!["new", "--unknown"],
        vec!["check", ".", "extra"],
        vec!["new", "demo", "--template", "other"],
        vec!["--version", "extra"],
        vec!["--json", "check"],
        vec!["check", "--json", "--json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("usage:"), "{args:?}: {error}");
    }
}

/// `zam fmt` 是 AI 工作循环的第三步（见 `zam new --template ai` 生成的 AI.md），
/// 所以这里锁住它的两条承重契约：只写真正变了的模块，语法不成立时一个文件都不写。
#[test]
fn fmt_rewrites_only_the_modules_whose_indentation_changed() {
    let root = std::env::temp_dir().join(format!("zam-fmt-clean-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"fmt\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    // main.zm 缩进已经正确，greeting.zm 缩进是错的。
    let clean = "use greeting\n\nfn main() {\n    greeting.hello()\n}\n";
    let messy = "pub fn hello() -> string {\n\"hi\"\n}\n";
    fs::write(src.join("main.zm"), clean).unwrap();
    fs::write(src.join("greeting.zm"), messy).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("fmt")
        .arg(&root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // 进度行走 stderr、stdout 留空，和 build 的 CACHE MISS/OUTPUT 行一致。
    assert!(
        output.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let progress = String::from_utf8_lossy(&output.stderr);
    assert!(
        progress.contains("FORMAT") && progress.contains("greeting.zm"),
        "{progress}"
    );
    assert!(!progress.contains("main.zm"), "{progress}");
    // 没变的文件原样不动（连 mtime 都不该碰）。
    assert_eq!(fs::read_to_string(src.join("main.zm")).unwrap(), clean);
    assert_eq!(
        fs::read_to_string(src.join("greeting.zm")).unwrap(),
        "pub fn hello() -> string {\n    \"hi\"\n}\n"
    );

    // 幂等：已经格式化过的项目再跑一次不该报任何改动。
    let again = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("fmt")
        .arg(&root)
        .output()
        .unwrap();
    assert!(
        again.status.success(),
        "{}",
        String::from_utf8_lossy(&again.stderr)
    );
    assert!(
        again.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&again.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fmt_writes_nothing_when_a_module_does_not_parse() {
    let root = std::env::temp_dir().join(format!("zam-fmt-broken-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"fmt\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    // main.zm 缩进是错的（正常情况下会被改写），greeting.zm 语法不成立。
    let messy = "fn main() {\nprintln(\"hi\")\n}\n";
    fs::write(src.join("main.zm"), messy).unwrap();
    fs::write(
        src.join("greeting.zm"),
        "fn hello() -> string {\n\"unterminated\n}\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("fmt")
        .arg(&root)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    // 整个项目先解析一遍：只要有模块读不懂，就一个文件都不动。
    assert_eq!(fs::read_to_string(src.join("main.zm")).unwrap(), messy);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fmt_accepts_a_single_file_and_rejects_json() {
    let path = std::env::temp_dir().join(format!("zam-fmt-file-{}.zm", std::process::id()));
    fs::write(&path, "fn main() {\nprintln(\"hi\")\n}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("fmt")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "fn main() {\n    println(\"hi\")\n}\n"
    );
    fs::remove_file(&path).unwrap();

    // fmt 没有结构化输出；明确拒绝比静默忽略 --json 更容易被 AI 循环发现。
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["fmt", "--json", "."])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("--json is only supported by check and build"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
