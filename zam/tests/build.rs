use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Workspace(PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("preserved failed build fixture: {}", self.0.display());
        } else {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
}

fn invoke(root: &Path, cache: &Path, command: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args([command])
        .arg(root)
        .env("ZAMAK_CACHE_DIR", cache)
        .output()
        .unwrap()
}

fn successful(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// 构建进度（CACHE/OBJECT/LINK/OUTPUT）走 stderr，stdout 只留程序输出与 --json 结果。
fn progress(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn fixture(root: &Path) {
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"hello\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/main.zm"),
        "use greet\nuse std/io\nfn main():\n    greet.hello()\n    io.println(\"世界\")\n",
    )
    .unwrap();
    fs::write(
        root.join("src/greet.zm"),
        "pub fn hello(): println(\"hello\")\n",
    )
    .unwrap();
    fs::write(
        root.join("src/unused.zm"),
        "pub struct Unused:\n    value: i64\nfn quiet(): pass\n",
    )
    .unwrap();
}

#[test]
fn bounded_call_changes_invalidate_only_their_native_unit() {
    let workspace = Workspace(std::env::temp_dir().join(format!(
        "zam-range-cache-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    let root = workspace.0.join("project");
    let cache = workspace.0.join("cache");
    fixture(&root);
    let source = |seed| {
        format!(
            "use greet\nfn twice(x: i64) -> i64: return x * 2\nfn main():\n    greet.hello()\n    println(twice({seed}))\n"
        )
    };
    fs::write(root.join("src/main.zm"), source(7)).unwrap();
    let cold = progress(&invoke(&root, &cache, "build"));
    assert_eq!(cold.matches("OBJECT MISS").count(), 3);
    assert!(cold.contains("LINK MISS"));
    let warm = progress(&invoke(&root, &cache, "build"));
    assert_eq!(warm.matches("OBJECT HIT").count(), 3);
    assert!(warm.contains("LINK HIT"));
    assert_eq!(successful(invoke(&root, &cache, "run")), "hello\n14\n");

    fs::write(root.join("src/main.zm"), source(9)).unwrap();
    let changed = progress(&invoke(&root, &cache, "build"));
    assert_eq!(changed.matches("OBJECT MISS").count(), 1);
    assert_eq!(changed.matches("OBJECT HIT").count(), 2);
    assert!(changed.contains("LINK MISS"));
    assert_eq!(successful(invoke(&root, &cache, "run")), "hello\n18\n");
    let changed_warm = progress(&invoke(&root, &cache, "build"));
    assert_eq!(changed_warm.matches("OBJECT HIT").count(), 3);
    assert!(changed_warm.contains("LINK HIT"));
}

#[test]
fn project_cache_execution_and_failures() {
    let workspace = Workspace(std::env::temp_dir().join(format!(
            "zam-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    let a = workspace.0.join("a");
    let b = workspace.0.join("b");
    let cache = workspace.0.join("cache");
    fixture(&a);
    fixture(&b);
    successful(invoke(&a, &cache, "check"));
    assert!(!cache.exists(), "check must not write cache");
    let first = progress(&invoke(&a, &cache, "build"));
    assert_eq!(first.matches("CACHE MISS").count(), 3);
    assert_eq!(first.matches("OBJECT MISS").count(), 3);
    let second = progress(&invoke(&b, &cache, "build"));
    assert_eq!(second.matches("CACHE HIT").count(), 3);
    assert_eq!(second.matches("OBJECT HIT").count(), 3);
    assert!(second.contains("LINK HIT"));
    let artifact = |text: &str| {
        PathBuf::from(
            text.lines()
                .find_map(|line| line.strip_prefix("OUTPUT "))
                .unwrap(),
        )
    };
    assert_eq!(
        fs::read(artifact(&first)).unwrap(),
        fs::read(artifact(&second)).unwrap()
    );
    let run = successful(invoke(&artifact(&second), &cache, "run"));
    assert_eq!(run, "hello\n世界\n");
    fs::write(
        b.join("src/greet.zm"),
        "pub fn hello(): println(\"changed\")\n",
    )
    .unwrap();
    let changed = progress(&invoke(&b, &cache, "build"));
    assert_eq!(changed.matches("CACHE MISS").count(), 2);
    assert_eq!(changed.matches("CACHE HIT").count(), 1);
    assert_eq!(changed.matches("OBJECT MISS").count(), 1);
    assert_eq!(changed.matches("OBJECT HIT").count(), 2);
    assert!(changed.contains("LINK MISS"));
    assert_eq!(
        successful(invoke(&artifact(&changed), &cache, "run")),
        "changed\n世界\n"
    );

    fs::write(b.join("src/greet.zm"), "pub fn hello(x: i64): println(x)\n").unwrap();
    assert!(
        !invoke(&b, &cache, "build").status.success(),
        "stale interface must not be accepted"
    );
    fs::write(
        b.join("src/main.zm"),
        "use greet\nfn main(): greet.hello(7)\n",
    )
    .unwrap();
    let interface = progress(&invoke(&b, &cache, "build"));
    assert_eq!(interface.matches("OBJECT MISS").count(), 2);
    assert_eq!(interface.matches("OBJECT HIT").count(), 1);
    assert_eq!(
        successful(invoke(&artifact(&interface), &cache, "run")),
        "7\n"
    );

    fs::write(
        b.join("src/unused.zm"),
        "pub struct Unused:\n    value: bool\nfn quiet(): pass\n",
    )
    .unwrap();
    let unrelated = progress(&invoke(&b, &cache, "build"));
    assert_eq!(unrelated.matches("OBJECT HIT").count(), 3);
    assert!(
        unrelated.contains("LINK HIT"),
        "unused layout must not change native output"
    );
    fs::write(
        b.join("src/main.zm"),
        "use unused\nfn main():\n    let p = unused.Unused(value: true)\n    println(p.value)\n",
    )
    .unwrap();
    let record = progress(&invoke(&b, &cache, "build"));
    assert_eq!(record.matches("OBJECT MISS").count(), 1);
    assert_eq!(
        successful(invoke(&artifact(&record), &cache, "run")),
        "true\n"
    );
    fs::write(
        b.join("src/unused.zm"),
        "pub struct Unused:\n    value: i64\nfn quiet(): pass\n",
    )
    .unwrap();
    assert!(
        !invoke(&b, &cache, "build").status.success(),
        "used field type changes must be checked"
    );
    fs::write(
        b.join("src/main.zm"),
        "use unused\nfn main():\n    let p = unused.Unused(value: 9)\n    println(p.value)\n",
    )
    .unwrap();
    let layout = progress(&invoke(&b, &cache, "build"));
    assert_eq!(layout.matches("OBJECT MISS").count(), 1);
    assert_eq!(layout.matches("OBJECT HIT").count(), 2);
    assert_eq!(successful(invoke(&artifact(&layout), &cache, "run")), "9\n");

    fs::write(b.join("src/unused.zm"), "pub struct Unused:\n    value: i64\npub fn make() -> Unused: Unused(value: 11)\npub fn relay(p: Unused) -> Unused: p\nfn quiet(): pass\n").unwrap();
    fs::write(b.join("src/main.zm"), "use unused\nfn main():\n    let p: unused.Unused = unused.relay(unused.make())\n    println(p.value)\n").unwrap();
    let passing = progress(&invoke(&b, &cache, "build"));
    assert_eq!(
        successful(invoke(&artifact(&passing), &cache, "run")),
        "11\n"
    );
    fs::write(b.join("src/unused.zm"), "struct Unused:\n    value: i64\npub fn make() -> Unused: Unused(value: 11)\npub fn relay(p: Unused) -> Unused: p\nfn quiet(): pass\n").unwrap();
    assert!(
        !invoke(&b, &cache, "check").status.success(),
        "private named types must not be accessible via annotations"
    );

    fs::write(b.join("src/unused.zm"), "pub struct Unused:\n    value: i64\npub fn make() -> Unused: Unused(value: 11)\nfn quiet(): pass\n").unwrap();
    fs::write(b.join("src/main.zm"), "use unused\nstruct Wrapper:\n    item: unused.Unused\nfn main():\n    let mut w = Wrapper(item: unused.make())\n    w.item = unused.make()\n    println(12)\n").unwrap();
    let nested = progress(&invoke(&b, &cache, "build"));
    assert_eq!(
        successful(invoke(&artifact(&nested), &cache, "run")),
        "12\n"
    );
    fs::write(b.join("src/unused.zm"), "pub struct Unused:\n    value: bool\npub fn make() -> Unused: Unused(value: true)\nfn quiet(): pass\n").unwrap();
    let nested_layout = progress(&invoke(&b, &cache, "build"));
    assert_eq!(nested_layout.matches("OBJECT MISS").count(), 2);
    assert_eq!(nested_layout.matches("OBJECT HIT").count(), 1);
    assert_eq!(
        successful(invoke(&artifact(&nested_layout), &cache, "run")),
        "12\n"
    );

    let concurrent_cache = workspace.0.join("concurrent-cache");
    let mut children = Vec::new();
    for index in 0..4 {
        let root = workspace.0.join(format!("parallel-{index}"));
        fixture(&root);
        children.push(
            Command::new(env!("CARGO_BIN_EXE_zam"))
                .arg("build")
                .arg(root)
                .env("ZAMAK_CACHE_DIR", &concurrent_cache)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap(),
        );
    }
    for child in children {
        progress(&child.wait_with_output().unwrap());
    }
    assert_eq!(fs::read_dir(&concurrent_cache).unwrap().count(), 11);
    let artifact_cache = fs::read_dir(&concurrent_cache)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    fs::write(&artifact_cache, "broken").unwrap();
    assert!(
        !invoke(&a, &concurrent_cache, "build").status.success(),
        "corrupt cache must be reported"
    );

    for source in [
        "fn main(): missing()",
        "fn main(): println(\"unterminated)",
        "fn main(): println(\"x\") println(\"y\")",
        "fn main(): println(\"{x}\")",
        "fn main(): let x = 1.5e",
        "fn main(): pass\nfn main(): pass",
        "fn other(): pass",
        "use missing\nfn main(): pass",
        "use std/nope\nfn main(): pass",
    ] {
        let path = workspace.0.join("bad.zm");
        fs::write(&path, source).unwrap();
        assert!(
            !invoke(&path, &cache, "check").status.success(),
            "accepted {source}"
        );
    }
    fs::write(a.join("src/greet.zm"), "fn hello(): pass\n").unwrap();
    assert!(
        !invoke(&a, &cache, "check").status.success(),
        "private function must not be exported"
    );
    fs::write(a.join("src/greet.zm"), "use main\npub fn hello(): pass\n").unwrap();
    assert!(
        !invoke(&a, &cache, "check").status.success(),
        "dependency cycles must be rejected"
    );
    fs::write(
        a.join("zam.toml"),
        "[project]\nname=\"hello\"\nentry=\"../main.zm\"\nmodules=\"src\"\n",
    )
    .unwrap();
    assert!(
        !invoke(&a, &cache, "check").status.success(),
        "manifest traversal must be rejected"
    );
    let malformed = workspace.0.join("bad.zbc");
    fs::write(
        &malformed,
        "ZAM-APP-1\nmain:main\nZAM-MODULE-1\nfn private main:main\nprint 世界\nend\n",
    )
    .unwrap();
    assert!(!invoke(&malformed, &cache, "run").status.success());
}

/// 缓存目录经过联接/符号链接（CI 的 8.3 短名 `RUNNER~1`、macOS 的 /var → /private/var 同理）时，
/// 编译器看到的工作目录写法与规范路径不同；路径映射若失配，随机临时目录名会漏进目标文件。
/// 两份独立缓存各自编译同一源码，同一个键的目标文件必须逐字节相同。
#[test]
fn object_bytes_ignore_linked_cache_paths() {
    let root = std::env::temp_dir().join(format!("zam-linked-{}", std::process::id()));
    let real = root.join("real");
    let linked = root.join("linked");
    fs::create_dir_all(&real).unwrap();
    let workspace = Workspace(root.clone());
    #[cfg(windows)]
    assert!(Command::new("cmd")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&linked)
        .arg(&real)
        .output()
        .unwrap()
        .status
        .success());
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &linked).unwrap();
    let source = root.join("main.zm");
    fs::write(&source, "fn main():\n    println(\"linked\")\n").unwrap();
    let mut objects = Vec::new();
    for name in ["first", "second"] {
        let cache = linked.join(name);
        progress(&invoke(&source, &cache, "build"));
        let mut entries: Vec<_> = fs::read_dir(&cache)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "obj"))
            .collect();
        assert_eq!(entries.len(), 1, "{entries:?}");
        let object = entries.pop().unwrap();
        objects.push((
            object.file_name().unwrap().to_owned(),
            fs::read(&object).unwrap(),
        ));
    }
    assert_eq!(objects[0].0, objects[1].0);
    assert!(
        objects[0].1 == objects[1].1,
        "object bytes depend on the scratch path"
    );
    drop(workspace);
}
