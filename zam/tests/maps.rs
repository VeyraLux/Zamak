mod common;

use common::Case;
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

fn stdout(output: &Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn rejected(output: &Output, message: &str, source: &str) {
    assert!(!output.status.success(), "accepted {source}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(message), "{source}: {stderr}");
}

fn run_with(case: &Case, source: &str, args: &[&str], input: &[u8]) -> Output {
    let file = case.path("main.zm");
    std::fs::write(&file, source).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&file)
        .arg("--")
        .args(args)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("ZAMAK_LANG", "en")
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn maps_count_lookup_remove_and_iterate_in_insertion_order() {
    let case = Case::new("maps", "core");
    let output = case.run(
        r#"use std/string
struct Index:
    name: string
    hits: {string: i64}
fn total(m: &{string: i64}) -> i64:
    let mut sum = 0
    for k in &m:
        sum = sum + m[k]
    sum
fn bump(m: &mut {string: i64}, word: &string):
    m[word] = m.get(word, 0) + 1
fn build() -> {i64: string}:
    let mut out: {i64: string} = {}
    for i in 0..5:
        out[i] = "n{i}"
    out
fn main():
    let text = "the cat and the dog and the bird"
    let words = string.words(&text)
    let mut counts: {string: i64} = {}
    for w in &words:
        bump(&mut counts, w)
    for k in &counts:
        let n = counts[k]
        println("{k}={n}")
    println(total(&counts))
    println(counts.contains("cat"))
    println(counts.get("cow", -1))
    println(counts.remove("cat"))
    println(counts.remove("cat"))
    println(counts.len())
    counts["the"] = 100
    println(counts["the"])
    let ages = {"amy": 30, "bob": 25}
    println(ages["bob"])
    let names = build()
    println(string.byte_len(&names[3]))
    let found = names.get(4, "none")
    let missing = names.get(9, "none")
    println("{found} {missing}")
    let keys = names.keys()
    println(keys.len())
    let flags: {i64: bool} = {
        1: true,
        2: false,
    }
    println(flags[2])
    let mut idx = Index(name: "x", hits: {})
    idx.hits["a"] = 1
    idx.hits["a"] = 2
    println(idx.hits["a"])
    let moved = counts
    println(moved.len())
"#,
    );
    assert_eq!(
        stdout(&output),
        "the=3\ncat=1\nand=2\ndog=1\nbird=1\n8\ntrue\n-1\ntrue\nfalse\n4\n100\n25\n2\nn4 none\n5\nfalse\n2\n4\n"
    );
}

/// 大量插入与删除：触发下标表重建与墓碑压实，删除后重新插入同一个键仍然正确。
#[test]
fn maps_survive_growth_and_tombstones() {
    let case = Case::new("maps", "growth");
    let output = case.run(
        "fn main():\n    let mut big: {i64: i64} = {}\n    for i in 0..5000:\n        big[i] = i * 2\n    for i in 0..5000:\n        if i % 3 == 0:\n            big.remove(i)\n    let mut check = 0\n    for k in &big:\n        check = check + big[k]\n    println(big.len())\n    println(check)\n    for i in 0..10:\n        big[i] = 7\n    println(big.len())\n    println(big[3])\n    let mut words: {string: i64} = {}\n    for round in 0..50:\n        for i in 0..20:\n            words[\"w{i}\"] = round\n        for i in 0..20:\n            words.remove(\"w{i}\")\n    println(words.len())\n    words[\"w7\"] = 1\n    println(words[\"w7\"])\n",
    );
    assert_eq!(stdout(&output), "3333\n16663334\n3337\n7\n0\n1\n");
}

#[test]
fn wordfreq_example_sorts_by_count() {
    let case = Case::new("maps", "wordfreq");
    let source = include_str!("../../examples/collections/wordfreq.zm");
    let input = case.path("words.txt");
    std::fs::write(&input, "the cat sat on the mat\nthe dog sat on the log\n").unwrap();
    let path = input.to_str().unwrap();
    assert_eq!(
        stdout(&run_with(&case, source, &[path, "3"], b"")),
        "7 distinct words\n4 the\n2 sat\n2 on\n"
    );
    assert_eq!(
        stdout(&run_with(&case, source, &[], "b a b 中 a b".as_bytes())),
        "3 distinct words\n3 b\n2 a\n1 中\n"
    );
    rejected(
        &run_with(&case, source, &[path, "x"], b""),
        "invalid integer: x",
        "bad limit",
    );
}

/// 字符串字面量与插值可以直接传给 `&string` 参数。
#[test]
fn literals_lend_to_shared_string_parameters() {
    let case = Case::new("maps", "literals");
    let output = case.run("use std/string\nfn shout(s: &string):\n    println(\"<{s}>\")\nfn main():\n    println(string.byte_len(\"中文\"))\n    let csv = \"a,b\"\n    let parts = string.split(&csv, \",\")\n    println(parts.len())\n    println(string.equal(&csv, \"a,b\"))\n    let n = 3\n    shout(\"n={n}\")\n    for i in 0..2:\n        shout(\"loop\")\n");
    assert_eq!(stdout(&output), "6\n2\ntrue\n<n=3>\n<loop>\n<loop>\n");
}

#[test]
fn missing_keys_abort() {
    let case = Case::new("maps", "missing");
    for source in [
        "fn main():\n    let m: {string: i64} = {}\n    println(m[\"x\"])\n",
        "use std/string\nfn main():\n    let m = {1: \"a\"}\n    println(string.byte_len(&m[2]))\n",
    ] {
        rejected(&case.run(source), "zam: key not found", source);
    }
}

#[test]
fn checker_rejects_map_misuse() {
    let case = Case::new("maps", "reject");
    for (body, message) in [
        ("let m = {}", "empty map literal needs a type annotation"),
        ("let m: {f64: i64} = {}", "unsupported map key type"),
        ("let m: {string: thing} = {}", "unsupported map value type"),
        ("let m = {\"a\": 1, \"b\": true}", "mismatch"),
        (
            "let m = {\"a\": \"x\"}\n    let s = m[\"a\"]",
            "cannot move a string out of m",
        ),
        (
            "let m: {string: i64} = {}\n    m[\"a\"] = 1",
            "cannot mutably borrow immutable value m",
        ),
        (
            "let mut m: {string: i64} = {\"a\": 1}\n    for k in &m:\n        m[k] = 2",
            "borrowed by an enclosing for loop",
        ),
        (
            "let mut m: {string: i64} = {\"a\": 1}\n    for k in &m:\n        m.remove(k)",
            "borrowed by an enclosing for loop",
        ),
        ("let m: {string: i64} = {}\n    println(m[1])", "mismatch"),
        (
            "let mut m: {string: i64} = {}\n    m.push(1)",
            "maps support len, contains, get, remove, keys, values and clear",
        ),
        (
            "let m: {string: i64} = {}\n    println(m < m)",
            "only support == and !=",
        ),
        (
            "let m: {string: i64} = {}\n    println(m.get(\"a\"))",
            "argument count mismatch",
        ),
        (
            "let m: {i64: i64} = {1: 2}\n    let n = 1\n    take(&m[n])",
            "scalar references are not supported",
        ),
        (
            "let m: {string: i64} = {}\n    let r: &{string: i64} = m",
            "borrow escape",
        ),
        (
            "let mut xs: [string] = [\"a\"]\n    xs.shuffle()",
            "lists support len, push, pop, insert, remove",
        ),
    ] {
        let source = format!("fn take(s: &string): pass\nfn main():\n    {body}\n");
        rejected(&case.run(&source), message, &source);
    }
    for (source, message) in [
        (
            "fn f() -> &{string: i64}:\n    return {}\nfn main(): pass\n",
            "functions cannot return references",
        ),
        (
            "fn f(m: &{string: i64}):\n    m[\"a\"] = 1\nfn main(): pass\n",
            "cannot mutably borrow immutable value m",
        ),
        (
            "use std/io\nfn main() -> !string:\n    io.read_line(\"x\")?\n",
            "mismatch",
        ),
    ] {
        rejected(&case.run(source), message, source);
    }
}

#[test]
fn lists_swap_in_place() {
    let case = Case::new("maps", "swap");
    let output = case.run("fn main():\n    let mut xs: [string] = [\"a\", \"b\", \"c\"]\n    xs.swap(0, 2)\n    for x in &xs:\n        println(x)\n    xs.swap(1, 1)\n    println(xs.len())\n");
    assert_eq!(stdout(&output), "c\nb\na\n3\n");
    rejected(
        &case.run("fn main():\n    let mut xs: [i64] = [1]\n    xs.swap(0, 1)\n"),
        "zam: list index out of bounds",
        "swap out of bounds",
    );
}

/// 嵌套容器：`[[i64]]`、`{string: [string]}`、`[{string: i64}]`，两层下标读写、元素上调方法、遍历、比较与打印。
#[test]
fn nested_containers() {
    let case = Case::new("maps", "nested");
    let output = case.run(include_str!("../../examples/collections/nested.zm"));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "[[11, 2, 3], [4, 5, 60], [7, 8]]\n60\n2\n100\na: apple,avocado\nb: banana,blueberry\nc: cherry\n[{\"a\": 1}, {\"b\": 2, \"c\": 3}]\n3\n[[\"amy\", \"bob\"], [\"solo\"]]\ntrue\n"
    );
    // 三层嵌套、&mut 遍历内层、字符串元素读出副本、弹出整行、结构体字段里的嵌套字典。
    let output = case.run("struct Index:\n    words: {string: [i64]}\nfn main():\n    let mut cube: [[[i64]]] = [[[1, 2], [3]], [[4]]]\n    for plane in &mut cube:\n        for row in &mut plane:\n            row.push(0)\n    println(cube)\n    let mut names: [[string]] = [[\"a\", \"b\"]]\n    let first = names[0][1]\n    names[0][1] = \"z\"\n    println(\"{first} {names}\")\n    let row = names.pop()\n    println(row.len())\n    let mut ix = Index(words: {})\n    ix.words[\"x\"] = [1]\n    ix.words[\"x\"].push(2)\n    println(ix)\n");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        "[[[1, 2, 0], [3, 0]], [[4, 0]]]\nb [[\"a\", \"z\"]]\n2\nIndex(words: {\"x\": [1, 2]})\n"
    );
    for (body, message) in [
        (
            "let g: [[i64]] = [[1]]\n    let r = g[0]",
            "cannot move a list or map out of g[..]",
        ),
        (
            "let g: [[[i64]]] = []\n    println(g[0][0][0])",
            "index at most two levels deep",
        ),
        (
            "let g: [[i64]] = [[1]]\n    let x = g[0][0][0]",
            "index at most two levels deep",
        ),
        (
            "let xs: [i64] = [1]\n    println(xs[0][0])",
            "cannot be indexed again",
        ),
        (
            "let g: [[i64]] = []\n    g.sort()",
            "need i64, f64, bool or string elements",
        ),
        (
            "let m: {string: [i64]} = {}\n    let v = m.get(\"a\", [])",
            "list and map values cannot be copied by get",
        ),
        (
            "let g: [[i64]] = [[1]]\n    g[0][0] = 2",
            "cannot mutably borrow immutable value g",
        ),
        (
            "let g: [[f64]] = [[1.0]]\n    let x: i64 = g[0][0]",
            "mismatch",
        ),
    ] {
        let source = format!("fn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}
