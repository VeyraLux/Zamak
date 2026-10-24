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
        .env("ZAMAK_LANG", "en")
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
            "use std/string\nfn main():\n    let xs: [string] = [\"a\"]\n    println(string.byte_len(&xs[-2]))\n",
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
            "let xs: [i64] = [1]\n    for x in &mut xs: pass",
            "cannot mutably borrow immutable value xs",
        ),
        (
            "let xs: [i64] = [1]\n    for x in xs.len(): pass",
            "needs a list or map; to count, write a range",
        ),
        (
            "let xs: [i64] = [1]\n    xs.shuffle()",
            "unknown function xs.shuffle",
        ),
        ("let n = 1\n    println(n.len())", "has no method len"),
        (
            "let xs: [i64; 2] = [1, 2]\n    println(xs)",
            "cannot print void or a fixed array",
        ),
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
            "let xs: [string] = [\"a\"]\n    let r: &string = &xs[0]",
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

/// 列表的 sort/contains/index_of/insert/remove/reverse/slice/extend/clear 与字典的 values/clear。
#[test]
fn list_and_map_methods() {
    let case = Case::new("lists", "methods");
    let source = include_str!("../../examples/collections/listops.zm");
    assert_eq!(
        stdout(&case.run(source)),
        "sorted: 1 2 3 5 8 9\ntrue\n5\n-1\nremoved 2\nedited: 0 1 3 5 8 9 10\nreversed: 10 9 8 5 3 1 0\nmiddle: 9 8 5\n9\n0\nBob;amy;bob;mia;\ntrue\nfirst Bob\n-1.0 0.5 2.5\nages: 30 25\n0\n"
    );
    // 排序稳定：-0.0 与 0.0 保持原顺序；NaN 排在最后；字符串元素移出与清空都不泄漏。
    let output = case.run(
        "use std/math\nfn main():\n    let mut xs: [f64] = [0.0, -0.0, math.inf - math.inf, -2.0, 0.0]\n    xs.sort()\n    println(\"{xs[0]} {xs[1]} {xs[2]} {xs[3]} {xs[4]}\")\n    let mut words: [string] = [\"b\", \"a\"]\n    words.insert(1, \"c\")\n    let w = words.pop(0)\n    println(w)\n    let tail = words.slice(1, 2)\n    words.extend(tail)\n    println(\"{words[0]}{words[1]}{words[2]}\")\n    words.clear()\n    println(words.len())\n",
    );
    assert_eq!(stdout(&output), "-2.0 0.0 -0.0 0.0 NaN\nb\ncaa\n0\n");
    // `append` 是 Python 的写法，与 `push` 相同；结构体字段上的列表也一样。
    let output = case.run(
        "struct Bag:\n    items: [string]\nfn main():\n    let mut xs: [i64] = []\n    xs.append(1)\n    xs.append(2)\n    let mut bag = Bag(items: [])\n    bag.items.append(\"pen\")\n    println(xs)\n    println(bag.items)\n",
    );
    assert_eq!(stdout(&output), "[1, 2]\n[\"pen\"]\n");
    // 同 Python：remove(x) 按值删第一个，pop(i) 按下标取走；结构体自己的 pop(i) 不受影响。
    let output = case.run(
        "struct Stack:\n    n: i64\n\n    fn pop(&mut self, k: i64) -> i64:\n        self.n -= k\n        self.n\n\nfn main():\n    let mut xs: [i64] = [3, 1, 3, 2]\n    xs.remove(3)\n    println(xs)\n    println(xs.pop(-1))\n    println(xs.pop(0))\n    println(xs)\n    let mut words: [string] = [\"a\", \"b\", \"a\"]\n    words.remove(\"a\")\n    println(words)\n    let mut s = Stack(n: 10)\n    println(s.pop(3))\n",
    );
    assert_eq!(stdout(&output), "[1, 3, 2]\n2\n1\n[3]\n[\"b\", \"a\"]\n7\n");
    for (body, message) in [
        (
            "let mut xs: [i64] = [1, 2]\n    xs.remove(5)",
            "zam: list.remove(x): x is not in the list",
        ),
        (
            "let mut xs: [i64] = [1]\n    println(xs.pop(1))",
            "zam: list index out of bounds",
        ),
        // 负下标从末尾数，越过开头才越界。
        (
            "let xs: [i64] = [1]\n    println(xs[-2])",
            "zam: list index out of bounds",
        ),
    ] {
        let source = format!("fn main():\n    {body}\n");
        rejected(&case.run(&source), message, &source);
    }
    for (body, message) in [
        (
            "let mut ps: [P] = []\n    ps.sort()",
            "struct elements have no natural order",
        ),
        (
            "let m: {string: P} = {}\n    let v = m.values()",
            "struct values cannot be copied",
        ),
        (
            "let mut xs: [i64] = [1]\n    xs.shuffle()",
            "lists support len, push, pop, insert",
        ),
        (
            "let mut xs: [i64] = [1]\n    for x in &xs:\n        xs.insert(0, x)",
            "borrowed by an enclosing for loop",
        ),
        (
            "let mut xs: [string] = []\n    let s = \"a\"\n    xs.insert(0, s)\n    println(s)",
            "use after move: s",
        ),
        ("let xs: [i64] = [1]\n    xs.reverse()", "immutable"),
    ] {
        let source = format!("struct P:\n    x: i64\nfn main():\n    {body}\n");
        rejected(&case.run(&source), message, &source);
    }
}

/// 带下标遍历、字典键值成对遍历、`&mut` 原地修改元素（标量是可赋值的别名，字符串与结构体是 `&mut` 借用）。
#[test]
fn indexed_pair_and_mutable_loops() {
    let case = Case::new("lists", "loops");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/collections/loops.zm"))),
        "0: zero\n1: one\n2: two\ntea costs 3.5\ncake costs 4.0\n{\"tea\": 7.0, \"cake\": 8.0}\n[10, 20, 30]\n[\"0-amy\", \"1-bob\"]\n[Item(name: \"pen\", qty: 2), Item(name: \"cup\", qty: 6)]\n"
    );
    // 删除过的字典项被跳过；break/continue 与 &mut 一起用；整数键的字典。
    let output = case.run("fn main():\n    let mut m = {1: 10, 2: 20, 3: 30}\n    m.remove(2)\n    for k, v in &mut m:\n        if k == 3: continue\n        v += 1\n    for k, v in &m: println(\"{k}={v}\")\n    let mut xs: [i64] = [5, 6, 7]\n    for i, x in &mut xs:\n        if i == 2: break\n        x = -x\n    println(xs)\n");
    assert_eq!(stdout(&output), "1=11\n3=30\n[-5, -6, 7]\n");
    for (body, message) in [
        (
            "let mut m = {\"a\": 1}\n    for k in &mut m: pass",
            "write for k, v in &mut m",
        ),
        (
            "for i, x in 0..3: pass",
            "two loop variables need a list or map",
        ),
        (
            "let xs: [i64] = [1]\n    for x in &xs:\n        x = 2",
            "cannot assign immutable value x",
        ),
        (
            "let mut xs: [i64] = [1]\n    for x in &mut xs:\n        xs.push(x)",
            "borrowed by an enclosing for loop",
        ),
        (
            "let xs: [i64] = [1]\n    for x, x in &xs: pass",
            "duplicate binding x",
        ),
        (
            "let m = {\"a\": 1}\n    for k, v in &m:\n        v = 2",
            "cannot assign immutable value v",
        ),
        (
            "let xs: [string] = [\"a\"]\n    for i, s in &xs:\n        i = 1",
            "cannot assign immutable value i",
        ),
    ] {
        let source = format!("fn main():\n    {body}\n");
        rejected(&case.run(&source), message, &source);
    }
}

#[test]
fn reused_list_buffers_keep_contents_separate() {
    // 小列表的缓冲区释放后会被下一次同级分配复用：内容不能串，长短混用、增长与嵌套都要对。
    let case = Case::new("lists", "buffer-reuse");
    let output = case.run(
        "struct Node:\n    children: [Node]\n\nfn make(depth: i64) -> Node:\n    if depth == 0: return Node(children: [])\n    Node(children: [make(depth - 1), make(depth - 1)])\n\nfn count(node: &Node) -> i64:\n    let mut total = 1\n    for child in node.children:\n        total += count(child)\n    total\n\nfn main():\n    let mut total = 0\n    for round in range(200):\n        let mut words: [string] = [\"a\", \"b\"]\n        for i in range(round % 9):\n            words.push(f\"w{i}\")\n        let mut xs: [i64] = [round, round + 1, round + 2]\n        xs.push(round * 2)\n        total += count(make(round % 6)) + len(words) + xs[3] - xs[0]\n        if round == 199:\n            println(words)\n            println(xs)\n    println(total)\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut total = 0_i64;
    for round in 0..200_i64 {
        total += (1 << (round % 6 + 1)) - 1 + 2 + round % 9 + round;
    }
    assert_eq!(
        stdout(&output),
        format!("[\"a\", \"b\", \"w0\"]\n[199, 200, 201, 398]\n{total}\n")
    );
}

#[test]
fn membership_in_a_literal_list_compares_each_item() {
    // `x in [1, 2, 3]` 不必先标注成列表：字面量就地展开成逐个比较。
    let case = Case::new("lists", "literal-membership");
    let output = case.run(
        "struct P:\n    name: string\n\nfn main():\n    let n = 3\n    let word = \"pear\"\n    let p = P(name: \"kiwi\")\n    println(n in [1, 2, 3])\n    println(n not in [1, 2])\n    println(word in [\"apple\", \"pear\"])\n    println(p.name in [\"apple\", \"pear\"])\n    println(2.5 in [1.5, 2.5])\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(stdout(&output), "true\ntrue\ntrue\nfalse\ntrue\n");
}

#[test]
fn struct_and_nested_lists_slice_with_deep_copies() {
    // 结构体与嵌套列表也能切片：元素逐个深复制，原列表不受影响。
    let case = Case::new("lists", "struct-slice");
    let output = case.run(
        "struct P:\n    name: string\n    tags: [string]\n\nfn main():\n    let xs: [P] = [P(name: \"a\", tags: [\"x\"]), P(name: \"b\", tags: []), P(name: \"c\", tags: [\"y\", \"z\"])]\n    let mut ys = xs[1:]\n    ys[0].tags.push(\"new\")\n    println(ys)\n    println(xs[:1])\n    println(xs[-1:])\n    println(xs.slice(5, 9))\n    let grid: [[i64]] = [[1], [2, 3]]\n    println(grid[1:])\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout(&output),
        "[P(name: \"b\", tags: [\"new\"]), P(name: \"c\", tags: [\"y\", \"z\"])]\n[P(name: \"a\", tags: [\"x\"])]\n[P(name: \"c\", tags: [\"y\", \"z\"])]\n[]\n[[2, 3]]\n"
    );
}

#[test]
fn list_literals_infer_their_element_type() {
    // 不写标注时按第一项推断：字符串、浮点数、结构体、布尔值、内层列表都得到可增长的列表。
    let case = Case::new("lists", "literals");
    let output = case.run(
        r#"struct P:
    x: i64

fn main():
    let names = ["a", "b"]
    for i, n in &names:
        println(f"{i} {n}")
    let mut fs = [1.5, 2.5]
    fs.push(3.0)
    println(fs)
    let ps = [P(x: 1), P(x: 2)]
    println(ps)
    let mut grid = [[1, 2], [3]]
    grid[1].push(4)
    println(grid)
    println([true, false])
"#,
    );
    assert_eq!(
        stdout(&output),
        "0 a\n1 b\n[1.5, 2.5, 3.0]\n[P(x: 1), P(x: 2)]\n[[1, 2], [3, 4]]\n[true, false]\n"
    );
}

#[test]
fn integer_list_literals_are_lists() {
    // 不写标注的 `[3, 9, 4]` 和 Python 一样是列表：能增长、能传给 `&[i64]` 和 `&[T]`、能用 sum/sorted。
    // 要定长数组得写 `[i64; N]`。
    let case = Case::new("lists", "integer_literals");
    let output = case.run(
        r#"fn total(items: &[i64]) -> i64:
    sum(items)

fn largest[T](items: &[T]) -> T:
    let mut best = items[0]
    for x in items:
        if x > best:
            best = x
    best

fn grow(items: &mut [i64]):
    items.append(len(items))

fn main():
    let mut xs = [3, 9, 4]
    xs.append(5)
    grow(&mut xs)
    println(xs)
    println(total(xs))
    println(largest(xs))
    println(largest([7, 2]))
    println(sorted(xs))
    let fixed: [i64; 2] = [1, 2]
    println(fixed[1])
"#,
    );
    assert_eq!(
        stdout(&output),
        "[3, 9, 4, 5, 4]\n25\n9\n7\n[3, 4, 4, 5, 9]\n2\n"
    );

    let source = "fn main():\n    let xs = [1, 2]\n    let ys = xs\n    println(xs)";
    rejected(&case.run(source), "use after move: xs", source);
}
