use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

struct Sandbox {
    root: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("zam-io-{name}-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    /// `zam run <file> -- args...`，可选地把 `input` 写进程序的标准输入。
    fn run<S: AsRef<OsStr>>(&self, source: &str, args: &[S], input: Option<&[u8]>) -> Output {
        let file = self.root.join("main.zm");
        fs::write(&file, source).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(&file)
            .arg("--")
            .args(args)
            .env("ZAMAK_CACHE_DIR", self.root.join("cache"))
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(input) = input {
            child.stdin.take().unwrap().write_all(input).unwrap();
        }
        child.wait_with_output().unwrap()
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn failure(output: &Output, message: &str) {
    assert!(!output.status.success(), "unexpected success");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(message), "{stderr}");
}

const NONE: &[&str] = &[];

#[test]
fn examples_read_files_stdin_and_arguments() {
    let sandbox = Sandbox::new("examples");
    let wc = include_str!("../../examples/io/wc.zm");
    let input = sandbox.path("输入 文件.txt");
    fs::write(&input, "hello world\nzamak 语言\n").unwrap();
    assert_eq!(stdout(&sandbox.run(wc, &[&input], None)), "2 4 25\n");
    assert_eq!(stdout(&sandbox.run(wc, NONE, Some(b"a b\nc"))), "1 3 5\n");
    failure(
        &sandbox.run(wc, &[sandbox.path("missing.txt")], None),
        "cannot open ",
    );

    let sum = include_str!("../../examples/io/sum.zm");
    assert_eq!(
        stdout(&sandbox.run(sum, NONE, Some(b"3\r\n 4 \n\n-10\n"))),
        "count=3 total=-3\n"
    );
    assert_eq!(
        stdout(&sandbox.run(sum, NONE, Some(b""))),
        "count=0 total=0\n"
    );
    failure(
        &sandbox.run(sum, NONE, Some(b"1\nx\n")),
        "invalid integer: x",
    );

    let notes = include_str!("../../examples/io/notes.zm");
    let output = sandbox.path("笔记.txt");
    assert_eq!(
        stdout(&sandbox.run(notes, &[&output], None)),
        "name\n扎马克\n3\n"
    );
    assert_eq!(fs::read(&output).unwrap(), "name=扎马克".as_bytes());
    failure(
        &sandbox.run(notes, NONE, None),
        "argument index out of range",
    );
}

#[test]
fn arguments_survive_quoting_and_unicode() {
    let sandbox = Sandbox::new("args");
    let echo = "use std/env\nfn main() -> !string:\n    println(env.arg_count())\n    for i in 0..env.arg_count():\n        let value = env.arg(i)?\n        println(\"[{value}]\")\n";
    let args = [
        "a b",
        "中文",
        "x\"y",
        "",
        r"back\slash",
        r"trail\",
        r#"q\"q"#,
        "--json",
        "-x",
    ];
    assert_eq!(
        stdout(&sandbox.run(echo, &args, None)),
        "9\n[a b]\n[中文]\n[x\"y]\n[]\n[back\\slash]\n[trail\\]\n[q\\\"q]\n[--json]\n[-x]\n"
    );
    assert_eq!(stdout(&sandbox.run(echo, NONE, None)), "0\n");
    let invalid = Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(["check", "x.zm", "--", "a"])
        .output()
        .unwrap();
    failure(&invalid, "only zam run accepts program arguments");
}

#[test]
fn string_helpers_parse_slice_and_search() {
    let sandbox = Sandbox::new("strings");
    let program = "use std/string\nfn show(text: &string) -> !string:\n    let value = string.parse_i64(text)?\n    println(value)\nfn try(text: string):\n    show(&text) catch e:\n        println(e)\n        return\nfn main():\n    try(\"9223372036854775807\")\n    try(\"-9223372036854775808\")\n    try(\"+42\")\n    try(\"9223372036854775808\")\n    try(\"-\")\n    try(\"\")\n    try(\"1 \")\n    let padded = \" \\t 中 文 \\r\\n\"\n    let trimmed = string.trim(&padded)\n    println(\"[{trimmed}]\")\n    let text = \"a中b\"\n    println(string.byte_at(&text, 1))\n    println(string.slice(&text, 1, 4))\n    let needle = \"b\"\n    let missing = \"z\"\n    let empty = \"\"\n    println(string.find(&text, &needle))\n    println(string.find(&text, &missing))\n    println(string.find(&text, &empty))\n";
    assert_eq!(
        stdout(&sandbox.run(program, NONE, None)),
        "9223372036854775807\n-9223372036854775808\n42\ninteger out of range: 9223372036854775808\ninvalid integer: -\ninvalid integer: \ninvalid integer: 1 \n[中 文]\n228\n中\n4\n-1\n0\n"
    );
    for (source, message) in [
        (
            "use std/string\nfn main():\n    let text = \"ab\"\n    println(string.byte_at(&text, 2))\n",
            "string index out of bounds",
        ),
        (
            "use std/string\nfn main():\n    let text = \"中\"\n    println(string.slice(&text, 0, 1))\n",
            "string index out of bounds",
        ),
        (
            "use std/string\nfn main():\n    let text = \"ab\"\n    println(string.slice(&text, 2, 1))\n",
            "string index out of bounds",
        ),
    ] {
        failure(&sandbox.run(source, NONE, None), message);
    }
}

#[test]
fn checker_rejects_misused_builtins() {
    let sandbox = Sandbox::new("reject");
    for (source, message) in [
        (
            "use std/fs\nfn main():\n    let p = \"x\"\n    let t = fs.read(&p)\n",
            "fallible call",
        ),
        (
            "fn main() -> !string:\n    let p = \"x\"\n    let t = fs.read(&p)?\n",
            "fs.read requires use std/fs",
        ),
        (
            "use std/fs\nfn main() -> !string:\n    let p = \"x\"\n    fs.delete(&p)?\n",
            "unknown function fs.delete",
        ),
        (
            "use std/io\nfn main() -> !string:\n    let line = \"\"\n    while io.read_line(&mut line)?: pass\n",
            "cannot mutably borrow immutable",
        ),
        (
            "use std/env\nfn main():\n    let n: string = env.arg_count()\n",
            "mismatch",
        ),
    ] {
        failure(&sandbox.run(source, NONE, None), message);
    }
}

#[test]
fn write_errors_are_values() {
    let sandbox = Sandbox::new("write");
    let program = "use std/env\nuse std/fs\nfn main() -> !string:\n    let path = env.arg(0)?\n    let text = \"x\"\n    fs.write(&path, &text)?\n    println(\"unreachable\")\n";
    let directory: &Path = &sandbox.root;
    failure(&sandbox.run(program, &[directory], None), "cannot open ");
}

#[test]
fn time_and_random() {
    let sandbox = Sandbox::new("time-random");
    const NONE: &[&str] = &[];

    // 同一个种子在任何平台上都得到同一串数（整数运算，不依赖 C 库的 rand）。
    let seeded = "use std/random\n\nfn main():\n    random.seed(42)\n    println(random.randint(1, 100))\n    println(random.randrange(0, 1000000))\n    println(random.random())\n    random.seed(-1)\n    println(random.randint(-9223372036854775807 - 1, 9223372036854775807))\n";
    assert_eq!(
        stdout(&sandbox.run(seeded, NONE, None)),
        "43\n543102\n0.6800434110281394\n1104825383502392584\n"
    );
    // 范围两端都取得到，取不到范围外的数。
    let bounds = "use std/random\n\nfn main():\n    let mut seen: {i64: bool} = {}\n    for i in 0..5000:\n        let n = random.randint(1, 6)\n        assert n >= 1 and n <= 6\n        seen[n] = true\n        let u = random.uniform(2.0, 3.0)\n        assert u >= 2.0 and u <= 3.0\n        assert random.randrange(5, 7) < 7\n    println(seen.len())\n";
    assert_eq!(stdout(&sandbox.run(bounds, NONE, None)), "6\n");
    failure(
        &sandbox.run(
            "use std/random\n\nfn main(): println(random.randint(5, 1))\n",
            NONE,
            None,
        ),
        "zam: random.randint needs a <= b",
    );
    failure(
        &sandbox.run(
            "use std/random\n\nfn main(): println(random.randrange(3, 3))\n",
            NONE,
            None,
        ),
        "zam: random.randrange needs start < stop",
    );

    // 时间：日期文字、睡眠、单调时钟。
    let clock = "use std/string\nuse std/time\n\nfn main():\n    println(time.iso_utc(0.0))\n    println(time.iso_utc(1760000000.9))\n    println(time.iso_utc(-1.0))\n    println(time.iso_utc(951782400.0))\n    let start = time.monotonic()\n    time.sleep(0.05)\n    time.sleep(-1.0)\n    let waited = time.monotonic() - start\n    assert waited >= 0.04 and waited < 5.0\n    assert time.time() > 1700000000.0\n    let local = time.iso_local(0.0)\n    assert string.byte_len(&local) == 25\n";
    assert_eq!(
        stdout(&sandbox.run(clock, NONE, None)),
        "1970-01-01T00:00:00Z\n2025-10-09T08:53:20Z\n1969-12-31T23:59:59Z\n2000-02-29T00:00:00Z\n"
    );
    failure(
        &sandbox.run(
            "use std/math\nuse std/time\n\nfn main(): println(time.iso_utc(math.inf - math.inf))\n",
            NONE,
            None,
        ),
        "zam: time value out of range",
    );

    // 猜数字示例：种子 7 时想的数是 95，按顺序猜 1..100 正好在第 95 次猜中。
    let guess = include_str!("../../examples/io/guess.zm");
    let input: String = (1..=100).map(|n| format!("{n}\n")).collect();
    let output = stdout(&sandbox.run(guess, &["7"], Some(input.as_bytes())));
    assert!(
        output.starts_with("Guess a number between 1 and 100.\nToo small.\n"),
        "{output}"
    );
    assert!(
        output.ends_with("Too small.\nYou got it in 95 guesses!\n"),
        "{output}"
    );
    assert_eq!(output.matches("Too small.").count(), 94);
    assert_eq!(
        stdout(&sandbox.run(guess, &["7"], Some(b"abc\n99\n"))),
        "Guess a number between 1 and 100.\nPlease type a whole number.\nToo big.\nThe number was 95.\n"
    );

    // 随机数状态整个程序共用一份：不同模块交替取数，结果与在一个模块里连续取相同。
    let project = sandbox.path("shared");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"shared\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    fs::write(
        project.join("src/dice.zm"),
        "use std/random\n\npub fn roll() -> i64: random.randint(1, 1000000)\n",
    )
    .unwrap();
    fs::write(project.join("src/main.zm"), "use std/random\nuse dice\n\nfn main():\n    random.seed(9)\n    println(random.randint(1, 1000000))\n    println(dice.roll())\n    println(random.randint(1, 1000000))\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&project)
        .env("ZAMAK_CACHE_DIR", sandbox.path("cache"))
        .output()
        .unwrap();
    let single = "use std/random\n\nfn main():\n    random.seed(9)\n    for i in 0..3:\n        println(random.randint(1, 1000000))\n";
    assert_eq!(stdout(&output), stdout(&sandbox.run(single, NONE, None)));
}

#[test]
fn file_system_and_paths() {
    let sandbox = Sandbox::new("file-system");
    let root = sandbox.path("树");
    let base = root.to_string_lossy().replace('\\', "/");

    // 创建、追加、列目录、大小、改名、删除；中文路径；错误是可以 catch 的值。
    let program = r#"use std/env
use std/fs

fn main() -> !string:
    let base = env.arg(0)?
    let deep = base + "/笔记/深层"
    fs.mkdir(&deep)?
    fs.mkdir(&deep)?
    let file = base + "/笔记/b.txt"
    fs.write(&file, "one\n")?
    fs.append(&file, "two\n")?
    println(fs.read(&file)?)
    let other = base + "/笔记/a.md"
    fs.write(&other, "x")?
    let dir = base + "/笔记"
    println(fs.list(&dir)?)
    println(fs.size(&file)?)
    println(fs.is_dir(&deep))
    println(fs.is_file(&other))
    let missing = base + "/nope"
    println(fs.exists(&missing))
    fs.rename(&other, &file)?
    println(fs.read(&file)?)
    fs.remove(&file)?
    fs.remove(&dir) catch e:
        println(e)
    fs.remove(&deep)?
    println(fs.list(&dir)?)
    fs.size(&dir) catch e:
        println("size: {e}")
    fs.list(&missing) catch e:
        println("list: {e}")
"#;
    let output = sandbox.run(program, &[&base], None);
    assert_eq!(
        stdout(&output),
        format!(
            "one\ntwo\n\n[\"a.md\", \"b.txt\", \"深层\"]\n8\ntrue\ntrue\nfalse\nx\ncannot remove directory (is it empty?) {base}/笔记\n[]\nsize: not a file: {base}/笔记\nlist: cannot list {base}/nope\n"
        )
    );

    // tree 示例：递归列目录，按名字排序。
    fs::create_dir_all(root.join("src/深")).unwrap();
    fs::write(root.join("src/main.zm"), "12345").unwrap();
    fs::write(root.join("src/深/a.txt"), "abc").unwrap();
    fs::write(root.join("README"), "").unwrap();
    let tree = include_str!("../../examples/io/tree.zm");
    assert_eq!(
        stdout(&sandbox.run(tree, &[&base], None)),
        format!(
            "{base}/\n  README (0 bytes)\n  src/\n    main.zm (5 bytes)\n    深/\n      a.txt (3 bytes)\n  笔记/\n3 directories, 3 files, 8 bytes\n"
        )
    );
    failure(
        &sandbox.run(tree, &[&format!("{base}/README")], None),
        "not a directory: ",
    );

    // std/path 只处理文字，不碰磁盘。
    let paths = r#"use std/path

fn main():
    println(path.join("a/b", "c.txt"))
    println(path.join("a/b/", "c.txt"))
    println(path.join("a", "/abs"))
    println(path.join("", "x"))
    println(path.parent("a/b/c.txt"))
    println(path.parent("c.txt") == "")
    println(path.parent("/x"))
    println(path.parent("a/b/"))
    println(path.name("a/b/c.txt"))
    println(path.name("a/b/"))
    println(path.stem("archive.tar.gz"))
    println(path.suffix("archive.tar.gz"))
    println(path.suffix(".bashrc") == "")
    println(path.stem(".bashrc"))
    println(path.suffix("dir.d/file") == "")
"#;
    assert_eq!(
        stdout(&sandbox.run(paths, &[] as &[&str], None)),
        "a/b/c.txt\na/b/c.txt\n/abs\nx\na/b\ntrue\n/\na\nc.txt\nb\narchive.tar\n.gz\ntrue\n.bashrc\ntrue\n"
    );

    // 没有成功值的调用，catch 块处理完可以接着往下走；有成功值时仍要离开。
    failure(
        &sandbox.run(
            "use std/fs\n\nfn main():\n    let n = fs.size(\"x\") catch e:\n        println(e)\n    println(n)\n",
            &[] as &[&str],
            None,
        ),
        "catch block for std/fs:size must return a value",
    );
    // 块里移走的变量之后算移走；块里的临时值在块尾释放，接着执行后面的语句。
    failure(
        &sandbox.run(
            "use std/fs

fn take(s: string): println(s)

fn main():
    let s = \"a\"
    fs.remove(\"nope\") catch e:
        take(s)
    println(s)
",
            &[] as &[&str],
            None,
        ),
        "use after move: s",
    );
    assert_eq!(
        stdout(&sandbox.run(
            "use std/fs

fn main():
    let s = \"kept\"
    fs.size(\"nope\") catch e:
        let note = \"inner \" + e
        println(note)
    println(s)
",
            &[] as &[&str],
            None,
        )),
        "inner not a file: nope
kept
"
    );
}
