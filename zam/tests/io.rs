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
    let echo = "use std/env\nfn main() -> !string {\nprintln(env.arg_count())\nfor i in 0..env.arg_count() {\nlet value = env.arg(i)?\nprintln(\"[{value}]\")\n}\n}\n";
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
    let program = "use std/string\nfn show(text: &string) -> !string {\nlet value = string.parse_i64(text)?\nprintln(value)\n}\nfn try(text: string) {\nshow(&text) catch e {\nprintln(e)\nreturn\n}\n}\nfn main() {\ntry(\"9223372036854775807\")\ntry(\"-9223372036854775808\")\ntry(\"+42\")\ntry(\"9223372036854775808\")\ntry(\"-\")\ntry(\"\")\ntry(\"1 \")\nlet padded = \" \\t 中 文 \\r\\n\"\nlet trimmed = string.trim(&padded)\nprintln(\"[{trimmed}]\")\nlet text = \"a中b\"\nprintln(string.byte_at(&text, 1))\nprintln(string.slice(&text, 1, 4))\nlet needle = \"b\"\nlet missing = \"z\"\nlet empty = \"\"\nprintln(string.find(&text, &needle))\nprintln(string.find(&text, &missing))\nprintln(string.find(&text, &empty))\n}\n";
    assert_eq!(
        stdout(&sandbox.run(program, NONE, None)),
        "9223372036854775807\n-9223372036854775808\n42\ninteger out of range: 9223372036854775808\ninvalid integer: -\ninvalid integer: \ninvalid integer: 1 \n[中 文]\n228\n中\n4\n-1\n0\n"
    );
    for (source, message) in [
        (
            "use std/string\nfn main() {\nlet text = \"ab\"\nprintln(string.byte_at(&text, 2))\n}\n",
            "string index out of bounds",
        ),
        (
            "use std/string\nfn main() {\nlet text = \"中\"\nprintln(string.slice(&text, 0, 1))\n}\n",
            "string index out of bounds",
        ),
        (
            "use std/string\nfn main() {\nlet text = \"ab\"\nprintln(string.slice(&text, 2, 1))\n}\n",
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
            "use std/fs\nfn main() -> !string {\nlet p = \"x\"\nlet t = fs.read(p)?\n}\n",
            "expected explicit borrow",
        ),
        (
            "use std/fs\nfn main() {\nlet p = \"x\"\nlet t = fs.read(&p)\n}\n",
            "fallible call",
        ),
        (
            "fn main() -> !string {\nlet p = \"x\"\nlet t = fs.read(&p)?\n}\n",
            "ambiguous or missing import",
        ),
        (
            "use std/fs\nfn main() -> !string {\nlet p = \"x\"\nfs.delete(&p)?\n}\n",
            "unknown function fs.delete",
        ),
        (
            "use std/io\nfn main() -> !string {\nlet line = \"\"\nwhile io.read_line(&mut line)? {}\n}\n",
            "cannot mutably borrow immutable",
        ),
        (
            "use std/env\nfn main() {\nlet n: string = env.arg_count()\n}\n",
            "mismatch",
        ),
    ] {
        failure(&sandbox.run(source, NONE, None), message);
    }
}

#[test]
fn write_errors_are_values() {
    let sandbox = Sandbox::new("write");
    let program = "use std/env\nuse std/fs\nfn main() -> !string {\nlet path = env.arg(0)?\nlet text = \"x\"\nfs.write(&path, &text)?\nprintln(\"unreachable\")\n}\n";
    let directory: &Path = &sandbox.root;
    failure(&sandbox.run(program, &[directory], None), "cannot open ");
}
