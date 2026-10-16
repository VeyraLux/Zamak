use std::{fs, process::Command};

#[test]
fn ownership_borrows_and_native_execution() {
    let root = std::env::temp_dir().join(format!("zam-own-{}", std::process::id()));
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
    fs::write(
        &file,
        r#"
fn view(s: &string): println(s)
fn change(s: &mut string): s = "changed"
fn consume(s: string): println(s)
fn make() -> string: return "returned"
fn forward(s: &mut string):
    view(&s)
    change(s)
fn main():
    let mut a = "世界"
    view(&a)
    forward(&mut a)
    println(a)
    let b = a
    consume(b)
    a = make()
    println(a)
    let empty = ""
    view(&empty)
    println("embedded\ttext")
    return
"#,
    )
    .unwrap();
    let result = invoke("run");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.ends_with("世界\nchanged\nchanged\nreturned\n\nembedded\ttext\n"));
    // 构建进度（含 OUTPUT）走 stderr，stdout 只剩程序输出。
    let progress = String::from_utf8_lossy(&result.stderr);
    let artifact = progress
        .lines()
        .find_map(|line| line.strip_prefix("OUTPUT "))
        .unwrap();
    let direct = Command::new(artifact).output().unwrap();
    assert!(direct.status.success());
    assert_eq!(
        String::from_utf8_lossy(&direct.stdout),
        "世界\n世界\nchanged\nchanged\nreturned\n\nembedded\ttext\n"
    );
    for (source, error) in [
        ("fn main():\n    let a = \"x\"\n    let b = a\n    println(a)", "use after move"),
        ("fn eat(x: string): pass\nfn main():\n    let a = \"x\"\n    eat(a)\n    eat(a)", "use after move"),
        ("fn bad() -> &string: return &x\nfn main(): pass", "borrow escape"),
        ("fn main():\n    let a = \"x\"\n    let r: &string = &a", "borrow escape"),
        ("fn f(x: &mut string, y: &string): pass\nfn main():\n    let mut a = \"x\"\n    f(&mut a, &a)", "borrow conflict"),
        ("fn f(x: &string, y: string): pass\nfn main():\n    let a = \"x\"\n    f(&a, a)", "borrow conflict"),
        ("fn f(x: string, y: &string): pass\nfn main():\n    let a = \"x\"\n    f(a, &a)", "use after move"),
        ("fn f(x: &mut string): pass\nfn main():\n    let a = \"x\"\n    f(&mut a)", "immutable"),
        ("fn f(x: &string): x = \"x\"\nfn main(): pass", "immutable"),
        ("fn f(x: &mut string): pass\nfn main():\n    let mut a = \"x\"\n    f(a)", "write &mut a"),
        ("fn main(x: string): pass", "main must have"),
        ("fn f(): pass\nfn main(): return f()", "void function cannot return"),
        ("fn f() -> string: pass\nfn main(): pass", "missing string return"),
        ("fn f(x: string): pass\nfn main(): f()", "argument count"),
        ("fn main():\n    let a = \"x\"\n    a = \"y\"", "immutable"),
        ("fn main():\n    return\n    println(\"x\")", "unreachable"),
        ("fn nested(x: &mut string) -> string: return \"v\"\nfn f(x: &string, y: string): pass\nfn main():\n    let mut a = \"x\"\n    f(&a, nested(&mut a))", "borrow conflict"),
    ] {
        fs::write(&file, source).unwrap();
        let result = invoke("check");
        assert!(!result.status.success(), "accepted {source}");
        assert!(String::from_utf8_lossy(&result.stderr).contains(error), "{}", String::from_utf8_lossy(&result.stderr));
    }
    fs::write(
        &file,
        "fn f(a: &string, b: &string): pass\nfn main():\n    let a = \"x\"\n    f(&a, &a)\n    f(&a, &a)",
    )
    .unwrap();
    assert!(invoke("check").status.success());
    fs::remove_dir_all(&root).unwrap();
}
