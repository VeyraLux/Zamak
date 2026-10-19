//! `zam doc`：从 `pub` 声明与 `#` 注释生成 API 页面。

use std::{fs, process::Command};

/// 生成到临时目录，返回 (退出码成功与否, 页面内容, stderr)。
fn generate(root: &std::path::Path, out: &std::path::Path) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("doc")
        .arg(root)
        .arg(out)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    let page = fs::read_to_string(out.join("index.html")).unwrap_or_default();
    (
        output.status.success(),
        page,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn doc_writes_an_api_page_from_public_declarations() {
    let root = std::env::temp_dir().join(format!("zam-doc-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"shapes\"\nmodules = \"src\"\ndescription = \"Geometry helpers\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/model.zm"),
        "\
# A point on the plane.
pub struct Point:
    # The horizontal coordinate.
    x: i64
    y: i64

    # Returns a copy moved to the right.
    pub fn right(&self, steps: i64) -> Point:
        Point(x: self.x + steps, y: self.y)

fn hidden() -> i64:
    1

pub const ORIGIN_X = 0
",
    )
    .unwrap();
    fs::write(root.join("src/main.zm"), "fn main():\n    println(1)\n").unwrap();

    let out = root.join("site");
    let (ok, page, stderr) = generate(&root, &out);
    assert!(ok, "{stderr}");
    // 进度打到 stderr，stdout 保持安静。
    assert!(stderr.contains("DOC "), "{stderr}");

    assert!(page.starts_with("<!doctype html>"));
    assert!(page.contains("<h1>shapes</h1>"), "{page}");
    assert!(page.contains("Geometry helpers"), "{page}");
    // 项目里的两个模块在导航里各有一项。
    assert!(page.contains("href=\"#main\""), "{page}");
    assert!(page.contains("href=\"#model\""), "{page}");
    // pub 的声明在，注释也在。
    assert!(page.contains("pub struct Point"), "{page}");
    assert!(page.contains("A point on the plane."), "{page}");
    assert!(page.contains("The horizontal coordinate."), "{page}");
    assert!(
        page.contains("Returns a copy moved to the right."),
        "{page}"
    );
    assert!(page.contains("pub const ORIGIN_X = 0"), "{page}");
    // 非 pub 的函数不出现；方法不会被当成第二个顶层声明。
    assert!(!page.contains("hidden"), "{page}");
    assert_eq!(page.matches("pub fn right").count(), 1, "{page}");
    // 没有 pub 声明的模块说清楚，而不是留空。
    assert!(page.contains("No <code>pub</code> declarations"), "{page}");

    // 输出目录不存在时自己建。
    let nested = root.join("a/b/c");
    let (ok, _, stderr) = generate(&root, &nested);
    assert!(ok, "{stderr}");
    assert!(nested.join("index.html").is_file());

    fs::remove_dir_all(root).unwrap();
}

/// 单文件项目也能出文档：标题用文件名对应的项目名，模块只有一页。
#[test]
fn doc_accepts_a_single_file() {
    let root = std::env::temp_dir().join(format!("zam-doc-file-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let file = root.join("lib.zm");
    fs::write(
        &file,
        "# Doubles a number.\npub fn double(n: i64) -> i64: n * 2\n",
    )
    .unwrap();
    let out = root.join("site");
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("doc")
        .arg(&file)
        .arg(&out)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let page = fs::read_to_string(out.join("index.html")).unwrap();
    assert!(page.contains("Doubles a number."), "{page}");
    assert!(page.contains("pub fn double(n: i64) -&gt; i64"), "{page}");
    fs::remove_dir_all(root).unwrap();
}
