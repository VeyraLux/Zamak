use std::{fs, path::Path, process::Command};

fn format(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("fmt")
        .arg(root)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap()
}

#[test]
fn fmt_reindents_a_project_and_refuses_broken_source() {
    let root = std::env::temp_dir().join(format!("zam-fmt-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"hello\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    let entry = root.join("src/main.zm");
    // Tab 缩进、行尾空白、多余空行；Tab 缩进的文件 check 会报错，fmt 照样能整理。
    let ugly = "fn main():\n\t#保留注释\n\tlet x =1   \n\tif true:\n\t\tprintln(x)\n\n\n";
    fs::write(&entry, ugly).unwrap();

    let output = format(&root);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert!(stderr.contains("FORMAT "), "{stderr}");
    let formatted = fs::read_to_string(&entry).unwrap();
    assert_eq!(
        formatted,
        "fn main():\n    #保留注释\n    let x =1\n\n    if true:\n        println(x)\n"
    );

    // 已经格式化过：不再改写，也不再报 FORMAT。
    let again = format(&root);
    assert!(again.status.success());
    assert!(!String::from_utf8_lossy(&again.stderr).contains("FORMAT"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), formatted);

    // 语法坏掉的文件原样保留，命令失败。
    let broken = "fn main():\n    println(`)\n";
    fs::write(&entry, broken).unwrap();
    let failed = format(&root);
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("unexpected character"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), broken);

    fs::remove_dir_all(&root).unwrap();
}

/// 仓库里的示例都已经是 `zam fmt` 的输出：复制到临时目录格式化一遍，不应有文件被改写。
/// 新增示例忘了格式化时这里会失败。顺带锁住规则：连续的单行 const 之间不加空行。
#[test]
fn examples_are_already_formatted() {
    fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "zm") {
                out.push(path);
            }
        }
    }
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
    let mut files = Vec::new();
    collect(&examples, &mut files);
    assert!(files.len() >= 40, "{files:?}");
    let root = std::env::temp_dir().join(format!("zam-fmt-examples-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for (index, file) in files.iter().enumerate() {
        // Windows 检出可能是 CRLF：fmt 保留换行风格，所以比较前不用转换。
        let copy = root.join(format!("{index}.zm"));
        fs::copy(file, &copy).unwrap();
        let output = format(&copy);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{}: {stderr}", file.display());
        assert!(
            !stderr.contains("FORMAT"),
            "{} is not formatted; run zam fmt on it",
            file.display()
        );
    }
    let consts = root.join("consts.zm");
    fs::write(
        &consts,
        "const A = 1\npub const B = 2\nconst C = 3\nfn main(): pass\n",
    )
    .unwrap();
    assert!(format(&consts).status.success());
    assert_eq!(
        fs::read_to_string(&consts).unwrap(),
        "const A = 1\npub const B = 2\nconst C = 3\n\nfn main(): pass\n"
    );
    fs::remove_dir_all(&root).unwrap();
}
