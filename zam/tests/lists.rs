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

/// 把 `source` 写成文件并 `zam run`，`input` 进标准输入，`args` 跟在 `--` 之后。
fn run_with(case: &Case, name: &str, source: &str, args: &[&str], input: &[u8]) -> Output {
    let file = case.path(name);
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
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn lists_grow_borrow_iterate_and_move() {
    let case = Case::new("lists", "core");
    let output = case.run("use std/string\nstruct Bag:\n    name: string\n    items: [string]\nfn total(xs: &[i64]) -> i64:\n    let mut sum = 0\n    for x in &xs:\n        sum = sum + x\n    sum\nfn double(xs: &mut [i64]):\n    for i in 0..xs.len():\n        xs[i] = xs[i] * 2\n    xs.push(100)\nfn make(n: i64) -> [i64]:\n    let mut out: [i64] = []\n    for i in 0..n:\n        out.push(i * i)\n    out\nfn shout(text: &string):\n    println(\"<{text}>\")\nfn main():\n    let mut nums = make(5)\n    println(nums.len())\n    println(total(&nums))\n    double(&mut nums)\n    println(total(&nums))\n    println(nums[5])\n    println(nums.pop())\n    let mut words: [string] = [\"alpha\", \"beta\"]\n    let w = \"gamma\"\n    words.push(w)\n    for word in &words:\n        shout(word)\n    words[0] = \"ALPHA\"\n    shout(&words[0])\n    println(string.char_len(&words[2]))\n    let last = words.pop()\n    println(last)\n    println(words.len())\n    let mut bag = Bag(name: \"box\", items: [])\n    bag.items.push(\"pen\")\n    bag.items.push(\"cup\")\n    for item in &bag.items:\n        println(item)\n    let flags: [bool] = [true, false]\n    println(flags[1])\n    let moved = nums\n    println(moved.len())\n    words = [\"reset\"]\n    println(words.len())\n");
    assert_eq!(
        stdout(&output),
        "5\n30\n160\n100\n100\n<alpha>\n<beta>\n<gamma>\n<ALPHA>\n5\ngamma\n2\npen\ncup\nfalse\n5\n1\n"
    );
}

#[test]
fn string_helpers_return_lists() {
    let case = Case::new("lists", "helpers");
    let output = run_with(
        &case,
        "main.zm",
        "use std/env\nuse std/string\nfn main():\n    let text = \"  the quick\\tbrown  fox\\n\"\n    let words = string.words(&text)\n    let comma = \", \"\n    println(string.join(&words, &comma))\n    let csv = \"a,b,,c\"\n    let sep = \",\"\n    let parts = string.split(&csv, &sep)\n    for part in &parts:\n        println(\"[{part}]\")\n    let body = \"one\\r\\ntwo\\n\\nthree\\n\"\n    let lines = string.lines(&body)\n    for line in &lines:\n        println(\"<{line}>\")\n    let empty = \"\"\n    let a = string.lines(&empty)\n    let b = string.words(&empty)\n    let c = string.split(&empty, &sep)\n    let d = string.split(&csv, &empty)\n    println(a.len())\n    println(b.len())\n    println(c.len())\n    println(d.len())\n    let none: [string] = []\n    println(\"[{string.join}]\")\n".replace("let none: [string] = []\n    println(\"[{string.join}]\")\n", "let none: [string] = []\n    let joined = string.join(&none, &comma)\n    println(\"[{joined}]\")\n    let args = env.args()\n    for arg in &args:\n        println(\"({arg})\")\n    let copied = string.copy(&args[0])\n    println(copied)\n").as_str(),
        &["x", "y z", "中文"],
        b"",
    );
    assert_eq!(
        stdout(&output),
        "the, quick, brown, fox\n[a]\n[b]\n[]\n[c]\n<one>\n<two>\n<>\n<three>\n0\n0\n1\n1\n[]\n(x)\n(y z)\n(中文)\nx\n"
    );
}

#[test]
fn examples_sort_and_reverse() {
    let case = Case::new("lists", "examples");
    let stats = include_str!("../../examples/collections/stats.zm");
    assert_eq!(
        stdout(&run_with(
            &case,
            "stats.zm",
            stats,
            &[],
            b"5\r\n3\n 9 \n\n1\n7\n"
        )),
        "count=5 min=1 max=9 median=5 total=25\n"
    );
    rejected(
        &run_with(&case, "stats.zm", stats, &[], b""),
        "no numbers on standard input",
        "empty stats input",
    );
    let tac = include_str!("../../examples/collections/tac.zm");
    let input = case.path("tac.txt");
    std::fs::write(
        &input,
        "hello zamak world\nshort\r\nthe extraordinary end\n",
    )
    .unwrap();
    assert_eq!(
        stdout(&run_with(
            &case,
            "tac.zm",
            tac,
            &[input.to_str().unwrap()],
            b""
        )),
        "3: the extraordinary end\n1: short\n3: hello zamak world\nlongest word: extraordinary\n"
    );
    assert_eq!(
        stdout(&run_with(&case, "tac.zm", tac, &[], b"a b\nc")),
        "1: c\n2: a b\nlongest word: c\n"
    );
}

#[test]
fn runtime_bounds_abort() {
    let case = Case::new("lists", "bounds");
    for (source, message) in [
        (
            "fn main():\n    let xs: [i64] = [1]\n    println(xs[1])\n",
            "zam: list index out of bounds",
        ),
        (
            "fn main():\n    let mut xs: [i64] = []\n    println(xs.pop())\n",
            "zam: pop from empty list",
        ),
        (
            "use std/string\nfn main():\n    let xs: [string] = [\"a\"]\n    println(string.byte_len(&xs[-1]))\n",
            "zam: list index out of bounds",
        ),
        (
            "fn main():\n    let mut xs: [string] = [\"a\"]\n    xs[3] = \"b\"\n",
            "zam: list index out of bounds",
        ),
    ] {
        rejected(&case.run(source), message, source);
    }
}

#[test]
fn checker_rejects_list_misuse() {
    let case = Case::new("lists", "reject");
    for (body, message) in [
        ("let xs = []", "empty list literal needs a type annotation"),
        ("let mut xs: [i64] = []\n    xs.push(\"a\")", "mismatch"),
        (
            "let xs: [i64] = []\n    xs.push(1)",
            "cannot mutably borrow immutable value xs",
        ),
        (
            "let xs: [string] = [\"a\"]\n    let s = xs[0]",
            "cannot move a string out of xs",
        ),
        (
            "let mut xs: [i64] = [1]\n    for x in &xs:\n        xs.push(x)",
            "borrowed by an enclosing for loop",
        ),
        (
            "let mut xs: [i64] = [1]\n    for x in &xs:\n        let ys = xs",
            "borrowed by an enclosing for loop",
        ),
        (
            "let mut xs: [i64] = [1]\n    for x in &xs:\n        xs = []",
            "borrowed by an enclosing for loop",
        ),
        (
            "let mut xs: [i64] = [1]\n    for x in &xs:\n        xs[0] = 2",
            "borrowed by an enclosing for loop",
        ),
        (
            "let mut xs: [i64] = [1]\n    for x in &mut xs: pass",
            "unsupported for loop over &mut",
        ),
        (
            "let xs: [i64] = [1]\n    for x in xs: pass",
            "iterate a list with for x in &items",
        ),
        (
            "let xs: [i64] = [1]\n    xs.sort()",
            "unknown function xs.sort",
        ),
        ("let n = 1\n    println(n.len())", "has no method len"),
        ("let xs: [i64] = [1]\n    println(xs)", "cannot print"),
        (
            "let xs: [i64] = [1]\n    let ys = xs\n    println(xs.len())",
            "use after move: xs",
        ),
        ("let xs: [thing] = []", "unsupported list element type"),
        (
            "let xs: [i64] = [1]\n    take(&xs[0])",
            "scalar references are not supported",
        ),
        (
            "let xs: [string] = [\"a\"]\n    let r = &xs[0]",
            "borrow escape",
        ),
        (
            "let mut xs: [i64] = [1]\n    fill(&mut xs, xs.len())",
            "borrow conflict",
        ),
        (
            "let s = \"a\"\n    fill(&s, 1)",
            "borrow type mismatch for s",
        ),
        (
            "let xs: [string] = [\"a\"]\n    let x = 1\n    x.push(1)",
            "has no method push",
        ),
    ] {
        let source = format!(
            "fn take(n: &string): pass\nfn fill(xs: &mut [i64], n: i64): pass\nfn main():\n    {body}\n"
        );
        rejected(&case.run(&source), message, &source);
    }
    for (source, message) in [
        (
            "fn f() -> &[i64]:\n    return []\nfn main(): pass\n",
            "functions cannot return references",
        ),
        (
            "struct S:\n    xs: &[i64]\nfn main(): pass\n",
            "structs cannot store references",
        ),
        (
            "fn f(xs: &[i64]):\n    xs.push(1)\nfn main(): pass\n",
            "cannot mutably borrow immutable value xs",
        ),
        (
            "fn f(xs: &[i64]):\n    xs = []\nfn main(): pass\n",
            "cannot assign immutable value xs",
        ),
        (
            "fn main():\n    missing.call()\n",
            "ambiguous or missing import for missing.call",
        ),
    ] {
        rejected(&case.run(source), message, source);
    }
}

/// 只有函数尾部的调用才是隐式返回值；循环体、非尾部分支末尾的调用只求值。
#[test]
fn trailing_calls_return_only_from_the_tail() {
    let case = Case::new("lists", "tail");
    let output = case.run("fn tick(n: i64):\n    println(n)\nfn value(n: i64) -> i64:\n    println(\"value {n}\")\n    n * 10\nfn count() -> i64:\n    let mut i = 0\n    while i < 3:\n        i = i + 1\n        value(i)\n    for j in 0..2:\n        tick(j)\n    if i > 100:\n        value(0)\n    return i\nfn pick(flag: bool) -> i64:\n    if flag:\n        value(1)\n    else:\n        value(2)\nstruct Point:\n    x: i64\n    y: i64\nfn main():\n    println(count())\n    println(pick(true))\n    println(pick(false))\n    let p = Point(x: 1, y: 2)\n    println(p.x + p.y)\n");
    assert_eq!(
        stdout(&output),
        "value 1\nvalue 2\nvalue 3\n0\n1\n3\nvalue 1\n10\nvalue 2\n20\n3\n"
    );
}
