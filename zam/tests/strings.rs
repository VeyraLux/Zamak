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
