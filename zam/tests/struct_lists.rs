mod common;

use common::Case;
use std::{fs, process::Command};

fn stdout(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

/// 示例：读入成绩，结构体借用求平均，`&mut` 原地加分，按平均分排序，结构体值的字典分组。
#[test]
fn gradebook_example_runs() {
    let case = Case::new("struct-lists", "gradebook");
    let output = case.run_with_input(
        include_str!("../../examples/collections/gradebook.zm"),
        b"ann 88 92 95\r\nbob* 70 75 72\ncid 60 65\n\ndee 91 99 100\neve* 96 98\n",
    );
    assert_eq!(
        stdout(&output),
        "eve: 100.0 A\ndee: 96.7 A\nann: 91.7 A\nbob: 77.3 C\ncid: 62.5 D\nA: 3 (best eve)\nC: 1 (best bob)\nD: 1 (best cid)\n"
    );
}

/// 结构体元素的列表与字典：读写元素字段、借用元素与元素字段、元素字段上的方法、弹出、整体替换。
#[test]
fn struct_elements_read_write_and_borrow() {
    let case = Case::new("struct-lists", "elements");
    let output = case.run(include_str!("../../examples/structs/elements.zm"));
    assert_eq!(
        stdout(&output),
        "2\n1\n4\na=11\nbee!=2\nC=6\n19\ntrue\nC=6\nz bee! 2\nbee!\n6\nkale=5\ntrue\n0\n"
    );
}

/// 结构体经由列表递归引用自己（树），构造、递归借用遍历与释放都正常。
#[test]
fn recursive_structs_through_lists() {
    let case = Case::new("struct-lists", "tree");
    let output = case.run(include_str!("../../examples/structs/tree.zm"));
    assert_eq!(
        stdout(&output),
        "root
  a
    a1
  b
    b1
5
"
    );
}

/// 跨模块的结构体列表与结构体借用：`[shapes.Point]`、`&shapes.Point`。
#[test]
fn struct_lists_work_across_modules() {
    let case = Case::new("struct-lists", "modules");
    let root = case.path("project");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("zam.toml"),
        "[project]\nname = \"geo\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    fs::write(
        root.join("src/shapes.zm"),
        "pub struct Point:\n    x: i64\n    y: i64\npub fn norm1(p: &Point) -> i64:\n    p.x + p.y\npub fn shift(p: &mut Point):\n    p.x += 1\n",
    )
    .unwrap();
    fs::write(
        root.join("src/main.zm"),
        "use shapes\nfn total(points: &[shapes.Point]) -> i64:\n    let mut sum = 0\n    for p in &points:\n        sum += shapes.norm1(p)\n    sum\nfn main():\n    let mut points: [shapes.Point] = []\n    points.push(shapes.Point(x: 1, y: 2))\n    points.push(shapes.Point(x: 3, y: 4))\n    shapes.shift(&mut points[0])\n    println(total(&points))\n    println(points[0].x)\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&root)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "11\n2\n");
}

/// 结构体不能被复制或从元素里移出；共享借用不能写；借用不能存进变量、字段或返回值。
#[test]
fn checker_rejects_struct_list_misuse() {
    let case = Case::new("struct-lists", "reject");
    for (body, message) in [
        ("let p = items[0]", "cannot move a struct out of items[..]"),
        (
            "let n = items[0].name",
            "owned field moves are not supported",
        ),
        (
            "for it in &items:\n        it.qty = 1",
            "cannot assign immutable value it; it is a shared borrow",
        ),
        (
            "for it in &items:\n        items.push(Item(name: \"x\", qty: 1))",
            "borrowed by an enclosing for loop",
        ),
        ("let r: &Item = &items[0]", "borrow escape"),
        (
            "let fixed: [Item] = []\n    fixed[0].qty = 1",
            "cannot mutably borrow immutable value fixed",
        ),
        (
            "let fixed: [Item] = []\n    grow(&mut fixed[0])",
            "cannot mutably borrow immutable value fixed",
        ),
        ("both(&items[0], &mut items[1])", "borrow conflict"),
        ("println(items[0] < items[0])", "only support == and !="),
        ("println(items[0].missing)", "unknown field missing"),
        ("println(items[0].qty.len())", "has no method len"),
        (
            "let m: {string: Item} = {}\n    let x = m.get(\"a\", 1)",
            "struct values cannot be copied",
        ),
        ("println(items[0].tags.len())", "unknown field tags"),
    ] {
        let source = format!(
            "struct Item:\n    name: string\n    qty: i64\nfn show(item: &Item): pass\nfn grow(item: &mut Item): pass\nfn both(a: &Item, b: &mut Item): pass\nfn main():\n    let mut items: [Item] = []\n    {body}\n"
        );
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}\n{stderr}");
    }
    for (source, message) in [
        (
            "struct Item:\n    qty: i64\nfn set(item: &Item):\n    item.qty = 1\nfn main(): pass\n",
            "it is a shared borrow",
        ),
        (
            "struct Item:\n    qty: i64\nfn copy(item: &Item) -> Item:\n    item\nfn main(): pass\n",
            "cannot move borrowed value item",
        ),
        (
            "struct Item:\n    qty: i64\nstruct Holder:\n    item: &Item\nfn main(): pass\n",
            "structs cannot store references",
        ),
        (
            "struct Item:\n    qty: i64\nfn main():\n    let r: &Item = Item(qty: 1)\n",
            "borrow escape",
        ),
    ] {
        let output = case.run(source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}\n{stderr}");
    }
}

/// copy(&x) 深复制、== 逐项比较、println 与插值显示结构体、枚举、列表与字典。
#[test]
fn compound_values_copy_compare_and_print() {
    let case = Case::new("struct-lists", "values");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/structs/values.zm"))),
        "true\nfalse\nPoint(x: 1, y: 2)\nmoved to Point(x: 5, y: 2)\nfalse\nTeam(name: \"红队 \\\"A\\\"\", members: [\"amy\", \"bob\"], scores: {\"amy\": 9.5})\n[\"amy\", \"bob\", \"cid\"]\n[Shape.Circle(1.5), Shape.Rect(2.0, 3.0), Shape.Empty]\ntrue\nfalse\ntrue\n{\"b\": 2, \"a\": 1}\n"
    );
    // 复制出来的值与原值互不影响；借用参数、元素与字段都能复制；空结构体与空容器也能打印。
    let output = case.run("struct E: pass\nstruct Bag:\n    items: [string]\n    e: E\nfn grow(b: &Bag) -> Bag:\n    let mut c = copy(b)\n    c.items.push(\"x\")\n    c\nfn main():\n    let bag = Bag(items: [\"a\"], e: E())\n    let big = grow(&bag)\n    println(bag)\n    println(big)\n    let bags: [Bag] = [big]\n    let first = copy(&bags[0])\n    let names = copy(&bag.items)\n    println(first == bags[0])\n    println(names)\n    let none: [i64] = []\n    let empty: {i64: bool} = {}\n    println(\"{none} {empty} {E()}\")\n    let s = \"tab\there\"\n    let t = copy(&s)\n    println(t)\n");
    assert_eq!(
        stdout(&output),
        "Bag(items: [\"a\"], e: E())\nBag(items: [\"a\", \"x\"], e: E())\ntrue\n[\"a\"]\n[] {} E()\ntab\there\n"
    );
    for (body, message) in [
        (
            "let n = 1\n    let m = copy(&n)",
            "already copied by assignment",
        ),
        (
            "let p = P(x: 1)\n    println(p == 1)",
            "compares two values of the same type",
        ),
        (
            "let p = P(x: 1)\n    println(p < p)",
            "only support == and !=",
        ),
        (
            "let p = P(x: 1)\n    let q = p\n    println(p == q)",
            "use after move: p",
        ),
        (
            "let a = [1, 2]\n    let b = copy(&a)",
            "copy is for strings, lists, maps, structs and enums",
        ),
        (
            "let p = P(x: 1)\n    let q: i64 = copy(&p)",
            "copy returns a value of the borrowed type",
        ),
    ] {
        let source = format!("struct P:\n    x: i64\nfn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
    // 用户自己定义的 copy 优先于内建的。
    let output = case.run("fn copy(n: i64) -> i64: n + 1\nfn main(): println(copy(1))\n");
    assert_eq!(stdout(&output), "2\n");
}

/// 结构体与枚举的方法：`&self`/`&mut self`/`self`、关联函数、跨模块 `pub fn`、可失败方法，以及误用。
#[test]
fn struct_and_enum_methods() {
    let case = Case::new("struct-lists", "methods");
    assert_eq!(
        stdout(&case.run(include_str!("../../examples/structs/methods.zm"))),
        "5.0\nVec2(x: 6.0, y: 8.0)\nVec2(x: 7.0, y: 9.0)\n2.0\ncircle: 3\nsquare: 9\n9.0\n70\nrefused: ada has only 70\n"
    );
    let project = case.path("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"geo\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    fs::write(
        project.join("src/geo.zm"),
        "pub struct Point:\n    x: i64\n    y: i64\n\n    pub fn new(x: i64, y: i64) -> Point: Point(x: x, y: y)\n\n    pub fn sum(&self) -> i64: self.x + self.y + self.bonus()\n\n    fn bonus(&self) -> i64: 100\n",
    )
    .unwrap();
    let run_project = |main: &str| {
        fs::write(project.join("src/main.zm"), main).unwrap();
        Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("run")
            .arg(&project)
            .env("ZAMAK_CACHE_DIR", case.path("cache"))
            .output()
            .unwrap()
    };
    let output = run_project("use geo\nfn main():\n    let p = geo.Point.new(1, 2)\n    println(p.sum())\n    let ps: [geo.Point] = [geo.Point.new(5, 5)]\n    println(ps[0].sum())\n");
    assert_eq!(stdout(&output), "103\n110\n");
    let output = run_project(
        "use geo\nfn main():\n    let p = geo.Point.new(1, 2)\n    println(p.bonus())\n",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("private method Point.bonus"), "{stderr}");
    let header = "struct V:\n    x: i64\n\n    fn new(x: i64) -> V: V(x: x)\n\n    fn get(&self) -> i64: self.x\n\n    fn bump(&mut self): self.x += 1\n\n    fn take(self) -> i64: self.x\n\n    fn check(&self) -> i64!string: self.x\n\n";
    for (body, message) in [
        (
            "let v = V.new(1)\n    v.bump()",
            "cannot mutably borrow immutable value v",
        ),
        (
            "let v = V.new(1)\n    println(v.missing())",
            "unknown method V.missing",
        ),
        (
            "let v = V.new(1)\n    let w = v.new(2)",
            "new has no self parameter",
        ),
        (
            "let v = V.new(1)\n    let n = v.take()\n    println(v.get())",
            "use after move: v",
        ),
        (
            "let vs: [V] = [V.new(1)]\n    println(vs[0].take())",
            "cannot move a struct out of vs[..]",
        ),
        (
            "let v = V.new(1)\n    println(v.check())",
            "must be handled with ?",
        ),
        ("println(V.missing(1))", "unknown function V.missing"),
    ] {
        let source = format!("{header}fn main():\n    {body}\n");
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
    for (source, message) in [
        ("fn f(self: i64): pass\nfn main(): pass\n", "self can only be the first parameter"),
        ("struct V:\n    x: i64\n\n    fn f(&self): pass\n\n    fn f(&self): pass\nfn main(): pass\n", "duplicate method V.f"),
        ("struct V:\n    x: i64\n\n    fn f(x: i64, self): pass\nfn main(): pass\n", "self can only be the first parameter"),
    ] {
        let output = case.run(source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
    // 调用结果上直接调用方法；借用参数（&V、&mut V）作为接收者转传。
    let output = case.run(&format!("{header}fn twice(v: &mut V):\n    v.bump()\n    v.bump()\nfn read(v: &V) -> i64: v.get()\nfn main():\n    println(V.new(7).get())\n    let mut v = V.new(1)\n    twice(&mut v)\n    println(read(&v))\n"));
    assert_eq!(stdout(&output), "7\n3\n");
}
