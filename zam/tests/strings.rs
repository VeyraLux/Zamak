mod common;

use std::{fs, process::Command};

#[test]
fn borrowed_byte_length() {
    let path = std::env::temp_dir().join(format!("zam-strings-{}.zm", std::process::id()));
    let invoke = |command| {
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg(command)
            .arg(&path)
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap()
    };
    fs::write(&path, "use std/string\nfn size(s: &string) -> i64: string.byte_len(s)\nfn chars(s: &string) -> i64: string.char_len(s)\nfn main():\n    let s = \"你好\"\n    println(string.byte_len(&s))\n    println(size(&s))\n    println(string.char_len(&s))\n    println(chars(&s))\n    println(s)\n    let empty = \"\"\n    println(string.byte_len(&empty))\n    println(string.char_len(&empty))\n    let ascii = \"abc\"\n    println(string.char_len(&ascii))\n    string.byte_len(&s)\n    string.char_len(&s)\n").unwrap();
    let output = invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("6\n6\n2\n2\n你好\n0\n0\n3\n"));
    for source in [
        "fn main():\n    let s = \"x\"\n    println(string.byte_len(&s))",
        "use std/string\nfn main(): string.byte_len()",
        "use std/string\nfn main():\n    let n = 1\n    string.byte_len(&n)",
        "use std/string\nfn main():\n    let mut s = \"x\"\n    string.byte_len(&mut s)",
        "use std/string\nfn main():\n    let s = \"x\"\n    let moved = s\n    string.byte_len(&s)",
        "use std/string\nfn main():\n    let s = \"x\"\n    let b: bool = string.byte_len(&s)",
    ] {
        fs::write(&path, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {source}");
    }
    for source in [
        "fn main():\n    let s = \"x\"\n    println(string.char_len(&s))",
        "use std/string\nfn main(): string.char_len()",
        "use std/string\nfn main():\n    let n = 1\n    string.char_len(&n)",
        "use std/string\nfn main():\n    let mut s = \"x\"\n    string.char_len(&mut s)",
        "use std/string\nfn main():\n    let s = \"x\"\n    let moved = s\n    string.char_len(&s)",
        "use std/string\nfn main():\n    let s = \"x\"\n    let b: bool = string.char_len(&s)",
    ] {
        fs::write(&path, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {source}");
    }
    fs::write(&path, "use std/string\nfn same(a: &string, b: &string) -> bool: string.equal(a, b)\nfn main():\n    let a = \"你好\"\n    let b = \"你好\"\n    let c = \"您好\"\n    let empty = \"\"\n    println(same(&a, &b))\n    println(string.equal(&a, &c))\n    println(string.equal(&a, &empty))\n    println(string.equal(&empty, &empty))\n    println(string.equal(&a, &a))\n    println(a)\n    string.equal(&a, &b)\n").unwrap();
    let output = invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("true\nfalse\nfalse\ntrue\ntrue\n你好\n")
    );
    for source in [
        "use std/string\nfn main():\n    let s = \"x\"\n    string.equal(&s)",
        "use std/string\nfn main():\n    let mut s = \"x\"\n    string.equal(&s, &mut s)",
        "use std/string\nfn probe(s: &mut string, flag: bool): pass\nfn main():\n    let mut s = \"x\"\n    probe(&mut s, string.equal(&s, &s))",
        "use std/string\nfn main():\n    let s = \"x\"\n    let n: i64 = string.equal(&s, &s)",
    ] {
        fs::write(&path, source).unwrap();
        assert!(!invoke("check").status.success(), "accepted {source}");
    }
    let declarations = "use std/string\nstruct Leaf:\n    name: string\nstruct Root:\n    leaf: Leaf\nfn change(s: &mut string): s = \"updated\"\nfn consume(r: Root) -> i64: return 1\nfn hold(s: &string, n: i64): pass\nfn pair(a: &mut string, b: &string): pass\n";
    fs::write(&path, format!("{declarations}fn main():\n    let mut r = Root(leaf: Leaf(name: \"你好\"))\n    println(string.byte_len(&r.leaf.name))\n    println(string.equal(&r.leaf.name, &r.leaf.name))\n    change(&mut r.leaf.name)\n    println(&r.leaf.name)\n    let moved = r\n    println(moved.leaf.name)" )).unwrap();
    let output = invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("6\ntrue\nupdated\nupdated\n"));
    for body in [
        "hold(&r.leaf.name, consume(r))",
        "pair(&mut r.leaf.name, &r.leaf.name)",
        "let moved = r\n    string.byte_len(&r.leaf.name)",
        "let ref: &string = &r.leaf.name",
        "string.byte_len(&r.leaf.missing)",
        "string.byte_len(&r.leaf)",
    ] {
        fs::write(&path, format!("{declarations}fn main():\n    let mut r = Root(leaf: Leaf(name: \"x\"))\n    {body}" )).unwrap();
        assert!(!invoke("check").status.success(), "accepted {body}");
    }
    fs::write(&path, format!("{declarations}fn main():\n    let r = Root(leaf: Leaf(name: \"x\"))\n    change(&mut r.leaf.name)" )).unwrap();
    assert!(!invoke("check").status.success());
    fs::write(
        &path,
        "fn show(s: &string): println(\"value={s}\")\nfn edit(s: &mut string): println(\"value={s}\")\nfn main():\n    let mut name = \"before\"\n    show(&name)\n    edit(&mut name)\n    name = \"after\"\n    println(name)\n",
    )
    .unwrap();
    let output = invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("value=before\nvalue=before\nafter\n")
    );

    fs::remove_file(path).unwrap();
}

#[test]
fn string_methods_work_on_elements_and_their_fields() {
    // `parts[0].strip()`、`users[0].name.upper()`、`m["k"].strip()`：元素借给 std/string 函数。
    let case = common::Case::new("strings", "element-methods");
    let output = case.run(
        "struct U:\n    name: string\n\nfn main() -> !string:\n    let parts = \"3, 4\".split(\",\")\n    println(int(parts[1].strip())? + 1)\n    let us: [U] = [U(name: \"ada\")]\n    println(us[0].name.upper())\n    let m = {\"k\": \" v \"}\n    println(m[\"k\"].strip())\n    println(len(parts[0]))\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "5\nADA\nv\n1\n"
    );
}

fn run_source(name: &str, source: &str) -> String {
    let case = common::Case::new("strings", name);
    let output = case.run(source);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

#[test]
fn python_string_methods_match_python() {
    // 期望值是 Python 3 对同样输入的输出。
    let out = run_source(
        "python-methods",
        r#"fn main():
    let s = "https://example.com/a/b.html"
    println(s.find("/", 8))
    println(s.rfind("/"))
    println(s.rfind("zz"))
    println(s.removeprefix("https://"))
    println(s.removesuffix(".html"))
    println(s.removesuffix(".txt"))
    println("hELLO wORLD".capitalize())
    println("hello big-world 2x".title())
    println(f"[{"ab".center(6)}] [{"a".center(4)}] [{"abc".center(2)}]")
    println("42".zfill(5))
    println("-42".zfill(5))
    println("ABC1".isupper())
    println("abc".islower())
    println("123".isupper())
"#,
    );
    assert_eq!(
        out,
        "19\n21\n-1\nexample.com/a/b.html\nhttps://example.com/a/b\nhttps://example.com/a/b.html\nHello world\nHello Big-World 2X\n[  ab  ] [ a  ] [abc]\n00042\n-0042\ntrue\ntrue\nfalse\n"
    );
}

#[test]
fn interpolation_takes_strings_slices_and_formats() {
    let out = run_source(
        "interpolation",
        r#"fn main():
    let mut d: {string: i64} = {}
    d["key"] = 7
    let s = "hello world"
    let n = 5
    let xs: [i64] = [1, 2, 3, 4]
    println(f"{d["key"]} {s.split(" ")} {{x}} {s.replace("o", "0")}")
    println(f"[{s[:n]}] [{s[6:]}] {xs[1:3]} {n:>4} {3.14159:.2}")
"#,
    );
    assert_eq!(
        out,
        "7 [\"hello\", \"world\"] {x} hell0 w0rld\n[hello] [world] [2, 3]    5 3.14\n"
    );
}

#[test]
fn string_elements_and_fields_are_read_as_copies() {
    // `let arg = args[i]`、`let why = c.error` 得到副本，原来的列表与结构体不变。
    let out = run_source(
        "copies",
        r#"struct C:
    url: string
    error: string
    n: i64

fn main():
    let args: [string] = ["a", "--b"]
    let mut i = 0
    while i < len(args):
        let arg = args[i]
        println(arg)
        i += 1
    println(args)
    let c = C(url: "u", error: "", n: 3)
    let why = if c.error != "": c.error else: f"status {c.n}"
    let u = c.url
    println(f"{why} {u} {c.url}")
    let cs = [C(url: "x", error: "e", n: 1)]
    let e = cs[0].error
    println(f"{e} {cs[0].error}")
"#,
    );
    assert_eq!(out, "a\n--b\n[\"a\", \"--b\"]\nstatus 3 u u\ne e\n");
}
