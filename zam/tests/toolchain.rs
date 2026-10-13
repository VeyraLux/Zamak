use std::{fs, path::PathBuf, process::Command};

fn sandbox(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("zam-toolchain-{name}-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("main.zm"),
        "fn main() {\n    println(\"fallback ok\")\n}\n",
    )
    .unwrap();
    root
}

/// 找不到任何 C 编译器时，报错要说清楚该装什么，而不是只说找不到 MSVC。
#[test]
fn missing_compilers_explain_what_to_install() {
    let root = sandbox("none");
    let empty = root.join("empty");
    fs::create_dir_all(&empty).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_zam"));
    command
        .arg("build")
        .arg(root.join("main.zm"))
        .env("ZAMAK_CACHE_DIR", root.join("cache"))
        .env_remove("CC")
        .env("PATH", &empty);
    if cfg!(windows) {
        command.env("ProgramFiles(x86)", &empty);
    }
    let output = command.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "{stderr}");
    assert!(stderr.contains("no C compiler found"), "{stderr}");
    if cfg!(windows) {
        assert!(stderr.contains("Visual Studio Build Tools"), "{stderr}");
        assert!(stderr.contains("MinGW-w64 gcc"), "{stderr}");
    } else {
        assert!(stderr.contains("cc, clang, gcc"), "{stderr}");
    }
    fs::remove_dir_all(root).unwrap();
}

/// Windows 上没有 MSVC 但 PATH 里有 gcc 时自动改用 gcc。没有 gcc 的机器跳过。
#[test]
fn windows_falls_back_to_gcc_without_msvc() {
    if !cfg!(windows) || Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipped: needs Windows with gcc on PATH");
        return;
    }
    let root = sandbox("gcc");
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(root.join("main.zm"))
        .env("ZAMAK_CACHE_DIR", root.join("cache"))
        .env_remove("CC")
        .env("ProgramFiles(x86)", root.join("no-visual-studio"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "fallback ok\n"
    );
    fs::remove_dir_all(root).unwrap();
}
