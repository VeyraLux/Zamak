use std::{fs, process::Command};

#[test]
fn english_readme_examples_run() {
    // Windows 检出可能是 CRLF，先统一换行再切代码块。
    let document = include_str!("../../README.md").replace("\r\n", "\n");
    let blocks: Vec<&str> = document
        .split("```zamak\n")
        .skip(1)
        .map(|rest| rest.split("```").next().unwrap())
        .collect();
    assert_eq!(blocks.len(), 2, "README.md 的 zamak 代码块数量变了");
    let root = std::env::temp_dir().join(format!("zam-readme-en-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for (index, block) in blocks.iter().enumerate() {
        let file = root.join(format!("example-{index}.zm"));
        fs::write(&file, block).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(&file)
            .env("ZAMAK_CACHE_DIR", root.join("cache"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"to be or not to be\n")
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "example {index}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
        if index == 0 {
            assert_eq!(stdout, "to: 2\nbe: 2\nor: 1\nnot: 1\n");
            assert!(document.contains(&format!(
                "```text\n$ echo \"to be or not to be\" | zam run count.zm\n{stdout}```"
            )));
        }
    }
    fs::remove_dir_all(&root).unwrap();
}
