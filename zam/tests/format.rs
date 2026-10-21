use std::{fs, process::Command};

#[test]
fn string_interpolation_reads_without_moving() {
    let path = std::env::temp_dir().join(format!("zam-format-{}.zm", std::process::id()));
    let invoke = |command| {
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg(command)
            .arg(&path)
            .env("ZAMAK_LANG", "en")
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
    // 前六行是插值读取语义，后面几行是格式说明（`{x:.3}`、宽度与对齐）。
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "你好，Zamak！x=7 true 原点\n字面大括号 {x} 与 {}\nZamak\nZamak\nZamak\n比值 0.667\n[007] [   Zamak] [  42  ]\n"
    );
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

/// `f"{x:.2}"`：冒号后面是格式说明——宽度、对齐（`<`、`>`、`^`）、小数位。
#[test]
fn format_specifiers_shape_interpolated_values() {
    let root = std::env::temp_dir().join(format!("zam-format-spec-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let file = root.join("main.zm");
    let run = |source: &str| {
        fs::write(&file, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(&file)
            .env("ZAMAK_CACHE_DIR", root.join("cache"))
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap()
    };

    let output = run("\
fn main():
    let x = 3.14159
    println(f\"{x:.2}\")
    println(f\"{x:.0}\")
    println(f\"{x:.3}\")
    println(f\"{42:>5}|\")
    println(f\"{42:<5}|\")
    println(f\"{42:^7}|\")
    println(f\"{7:03}\")
    println(f\"{1.5:08.2}\")
    println(f\"{x:.2} and {42:>4}\")
    println(f\"{x}\")
    println(f\"{5:*^5}|\")
");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "3.14\n3\n3.142\n   42|\n42   |\n  42   |\n007\n00001.50\n3.14 and   42\n3.14159\n**5**|\n"
    );

    // 对齐也能用在字符串上；说明只影响这一段，别的段照常。
    let output = run("\
fn main():
    let name = \"bob\"
    let score = 7.5
    println(f\"{name:>6}|{score:.1}\")
    println(f\"{name}!\")
");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "   bob|7.5\nbob!\n"
    );

    // 说明写错时是语法错误（ZM110）并给出改法，不会落到 ZM999。
    for (source, message) in [
        (
            "fn main():\n    let n = 42\n    println(f\"{n:.2}\")\n",
            "only an f64 takes decimals",
        ),
        (
            "fn main():\n    let n = 42\n    println(f\"{n:d}\")\n",
            "unsupported format",
        ),
        (
            "fn main():\n    let x = 1.0\n    println(f\"{x:.}\")\n",
            "decimals after the dot",
        ),
        (
            "fn main():\n    let n = 4\n    println(f\"{n:名>5}\")\n",
            "fill must be an ASCII character",
        ),
    ] {
        fs::write(&file, source).unwrap();
        let checked = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&file)
            .arg("--json")
            .env("ZAMAK_CACHE_DIR", root.join("cache"))
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&checked.stdout);
        assert!(text.contains("ZM110"), "{source}: {text}");
        assert!(!text.contains("ZM999"), "{source}: {text}");
        assert!(text.contains(message), "{source}: {text}");
        assert!(text.contains("suggestion"), "{source}: {text}");
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn format_widths_count_characters_and_zero_pad_after_the_sign() {
    let root = std::env::temp_dir().join(format!("zam-format-width-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let file = root.join("main.zm");
    fs::write(
        &file,
        "fn main():\n    let e = \"é\"\n    let z = \"名字\"\n    let n = -42\n    let neg = -3.5\n    println(f\"[{e:>3}][{z:>4}][{z:*^6}]\")\n    println(f\"{n:05}|{neg:08.2}|{n:>5}\")\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&file)
        .env("ZAMAK_CACHE_DIR", root.join("cache"))
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "[  é][  名字][**名字**]\n-0042|-0003.50|  -42\n"
    );
    fs::remove_dir_all(root).unwrap();
}
