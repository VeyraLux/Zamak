use std::{fs, process::Command};

#[test]
fn string_interpolation_reads_without_moving() {
    let path = std::env::temp_dir().join(format!("zam-format-{}.zm", std::process::id()));
    let invoke = |command| {
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg(command)
            .arg(&path)
            .output()
            .unwrap()
    };
    fs::write(&path, include_str!("../../examples/values/format.zm")).unwrap();
    let output = invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .ends_with("你好，Zamak！x=7 true 原点\n字面大括号 {x} 与 {}\nZamak\nZamak\nZamak\n"));
    for source in [
        "fn main(): println(\"{}\")",
        "fn main(): println(\"{1 +}\")",
        "fn main():\n    let n = 1\n    println(\"{n.y}\")",
        "fn main(): println(\"a={name}\")",
        "fn main(): println(\"{name\")",
        "fn main(): println(\"a}b\")",
        "fn main(): println(\"{a{b}}\")",
        "use std/fmt\nfn main():\n    let s = \"x\"\n    let t = s\n    fmt.println(\"{s}\")",
        "fn main():\n    let a = [1, 2]\n    println(\"{a}\")",
        "fn main():\n    let s = \"x\"\n    let n: i64 = \"v={s}\"",
    ] {
        fs::write(&path, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {source}");
    }
    fs::remove_file(path).unwrap();
}
