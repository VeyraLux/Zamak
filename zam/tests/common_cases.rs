use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);

struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zam-common-cases-{}-{}",
            std::process::id(),
            NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn source(&self) -> PathBuf {
        self.root.join("main.zm")
    }

    fn invoke(&self, command: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg(command)
            .arg(self.source())
            .env("ZAMAK_CACHE_DIR", self.root.join("cache"))
            .output()
            .unwrap()
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_source(workspace: &TempWorkspace, source: &str) {
    fs::write(workspace.source(), source).unwrap();
}

#[test]
fn tutorial_algorithms_and_native_output() {
    let workspace = TempWorkspace::new();
    write_source(
        &workspace,
        include_str!("../../examples/tutorial_cases/main.zm"),
    );
    let output = workspace.invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "1\n1\n3628800\n0\n1\n55\n6\n6\n9\n42\nfalse\ntrue\ntrue\nfalse\n0\n8\n3\n6\n2\n问候：世界\n世界\n"
    );
}

#[test]
fn loop_bound_reuse_preserves_mutations_and_control_flow() {
    let workspace = TempWorkspace::new();
    write_source(
        &workspace,
        r#"
fn boundary() -> i64:
    println(99)
    return 2
fn main():
    let seed = 3
    let mut index = 0
    while index < (seed + 2) % 7:
        index = index + 1
        if index == 2: continue
        if index == 4: break
        println(index)
    println(index)
    let mut bound = 4
    index = 0
    while index < bound + 1:
        if index == 1: bound = 1
        index = index + 1
    println(index)
    bound = 4
    index = 0
    while index < bound + 1:
        let mut once = 0
        while once < 1:
            bound = 0
            once = once + 1
        index = index + 1
    println(index)
    index = 0
    while index < boundary() + 0: index = index + 1
    println(index)
    index = 0
    while false and index < 1 / 0: index = index + 1
    println(index)
"#,
    );
    let output = workspace.invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "1\n3\n4\n2\n1\n99\n99\n99\n2\n0\n"
    );
}

#[test]
fn loop_bounds_preserve_first_evaluation_errors() {
    let workspace = TempWorkspace::new();
    for condition in ["0 < 1 // 0", "0 < 9223372036854775807 + 1"] {
        write_source(
            &workspace,
            &format!("fn main():\n    while {condition}: println(7)"),
        );
        let output = workspace.invoke("run");
        assert!(!output.status.success(), "accepted {condition}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("integer arithmetic error") || stderr.contains("division by zero"),
            "{stderr}"
        );
    }
    write_source(
        &workspace,
        r#"
fn left() -> i64!string: fail "left first"
fn main() -> !string:
    while left()? < 1 / 0: println(7)
"#,
    );
    let output = workspace.invoke("run");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("left first"), "{stderr}");
    assert!(!stderr.contains("integer arithmetic error"), "{stderr}");
}

#[test]
fn common_case_rejections_and_runtime_overflow() {
    let workspace = TempWorkspace::new();
    for source in [
        "fn main(): let values: [i64; 3] = [1, 2]",
        "fn main():\n    let values = [1, 2]\n    println(values[2])",
        "fn main():\n    let values = [1, 2]\n    println(values[-3])",
        "fn main():\n    let mut values = [1, 2]\n    values[0] = true",
        "fn main(): println(1 == true)",
    ] {
        write_source(&workspace, source);
        assert!(
            !workspace.invoke("check").status.success(),
            "accepted {source}"
        );
    }

    for expression in [
        "9223372036854775807 + 1",
        "9223372036854775807 * 2",
        "-9223372036854775808 - 1",
    ] {
        write_source(&workspace, &format!("fn main(): println({expression})"));
        let output = workspace.invoke("run");
        assert!(!output.status.success(), "accepted {expression}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("integer arithmetic error"),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
