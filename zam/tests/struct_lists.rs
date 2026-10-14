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
        ("show(items[0])", "cannot move a struct out of items[..]"),
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
        ("let r = &items[0]", "borrow escape"),
        (
            "let fixed: [Item] = []\n    fixed[0].qty = 1",
            "cannot mutably borrow immutable value fixed",
        ),
        (
            "let fixed: [Item] = []\n    grow(&mut fixed[0])",
            "cannot mutably borrow immutable value fixed",
        ),
        ("both(&items[0], &mut items[1])", "borrow conflict"),
        ("println(items[0])", "cannot print"),
        ("println(items[0].missing)", "unknown field missing"),
        ("println(items[0].qty.len())", "has no method len"),
        (
            "let m: {string: Item} = {}\n    let x = m.get(\"a\", 1)",
            "struct values cannot be copied",
        ),
        (
            "let it = Item(name: \"x\", qty: 1)\n    show(it)",
            "expected explicit borrow of it",
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
