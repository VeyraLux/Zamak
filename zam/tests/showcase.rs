use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

/// 展示程序 examples/md2html：多模块、递归、列表、字符串处理、文件与标准输入。
/// 复制到临时目录再构建，避免在仓库里留下 build/。
#[test]
fn md2html_converts_the_sample_document() {
    let root = std::env::temp_dir().join(format!("zam-showcase-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    for (name, text) in [
        ("zam.toml", include_str!("../../examples/md2html/zam.toml")),
        (
            "sample.md",
            include_str!("../../examples/md2html/sample.md"),
        ),
        (
            "src/main.zm",
            include_str!("../../examples/md2html/src/main.zm"),
        ),
        (
            "src/blocks.zm",
            include_str!("../../examples/md2html/src/blocks.zm"),
        ),
        (
            "src/inline.zm",
            include_str!("../../examples/md2html/src/inline.zm"),
        ),
        (
            "src/html.zm",
            include_str!("../../examples/md2html/src/html.zm"),
        ),
    ] {
        fs::write(root.join(name), text).unwrap();
    }
    let expected = include_str!("../../examples/md2html/sample.html").replace("\r\n", "\n");
    let run = |args: &[&str], input: &[u8]| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(&root)
            .arg("--")
            .args(args)
            .env("ZAMAK_CACHE_DIR", root.join("cache"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
    };
    let sample = root.join("sample.md");
    assert_eq!(run(&[sample.to_str().unwrap()], b""), expected);
    // 标准输入同样可以；未闭合的代码块在文末收尾，`*` 落单时原样保留。
    assert_eq!(
        run(&[], b"a * b\n```\nx < y"),
        "<p>a * b</p>\n<pre><code>x &lt; y</code></pre>\n"
    );
    fs::remove_dir_all(&root).unwrap();
}
