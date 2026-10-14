mod common;

use common::Case;

fn stdout(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

/// 变体构造、语句与表达式两种 match、`_` 分支、借用载荷、枚举列表、匹配临时值与列表元素。
#[test]
fn enums_example_runs() {
    let case = Case::new("enums", "example");
    let output = case.run(include_str!("../../examples/control/enums.zm"));
    assert_eq!(
        stdout(&output),
        "circle 1.0 -> 3.00\nrect 2.0x3.5 -> 7.00\nlabel hi -> 0.00\nempty -> 0.00\n12\ncircle 2.0\nmade empty\n1.0\n2\nhi\n"
    );
}

/// 示例：把算式切成枚举记号再求值，错误经 `fail` 与 `catch` 处理。
#[test]
fn calculator_example_runs() {
    let case = Case::new("enums", "calc");
    let output = case.run_with_input(
        include_str!("../../examples/control/calc.zm"),
        b"1 + 2 * 3\r\n10 / 3 - 4\n\n2 * 3 * 4 - 5\n7 / 0\n8 +\n3 x 4\n",
    );
    assert_eq!(
        stdout(&output),
        "1 + 2 * 3 = 7\n10 / 3 - 4 = -1\n2 * 3 * 4 - 5 = 19\n7 / 0 -> error: division by zero\n8 + -> error: missing number after operator\n3 x 4 -> error: unexpected character at 2\n"
    );
}

/// 枚举放进结构体字段与字典；分支里的 break/continue/return 和所有权合并都按普通代码块处理。
#[test]
fn enums_in_fields_maps_and_control_flow() {
    let case = Case::new("enums", "fields");
    let output = case.run(
        r#"enum Status {
    Open
    Done(by: string)
}
struct Task {
    title: string
    status: Status
}
fn label(s: &Status) -> string {
    match s {
        Open => "open"
        Done(by) => "done by {by}"
    }
}
fn main() {
    let mut tasks: [Task] = []
    tasks.push(Task { title: "a" status: Status.Open })
    tasks.push(Task { title: "b" status: Status.Done("ann") })
    tasks.push(Task { title: "c" status: Status.Open })
    tasks[2].status = Status.Done("bob")
    let mut done = 0
    for i in 0..tasks.len() {
        match tasks[i].status {
            Open => {
                continue
            }
            Done(_) => {
                done += 1
            }
        }
    }
    println(done)
    for t in &tasks {
        println("{t.title}: {label(&t.status)}")
    }
    let mut owners: {string: Status} = {}
    owners["x"] = Status.Done("cid")
    println(label(&owners["x"]))
    let name = "kept"
    let moved = match owners["x"] {
        Done(by) => by == "cid"
        Open => false
    }
    println(moved)
    println(name)
    let mut n = 0
    while true {
        match Status.Open {
            Open => {
                n += 1
                if n == 3 {
                    break
                }
            }
            _ => {
                return
            }
        }
    }
    println(n)
}
"#,
    );
    assert_eq!(
        stdout(&output),
        "2\na: open\nb: done by ann\nc: done by bob\ndone by cid\ntrue\nkept\n3\n"
    );
}

/// 分支要覆盖所有变体；变体名与载荷个数要对；载荷是只读借用；匹配期间不能改被匹配的值。
#[test]
fn checker_rejects_enum_misuse() {
    let case = Case::new("enums", "reject");
    for (body, message) in [
        ("match s {\n        Circle(r) => println(r)\n    }", "match on Shape does not cover Rect, Label, Empty"),
        ("match s {\n        _ => println(1)\n        Empty => println(2)\n    }", "unreachable match arm after _"),
        ("match s {\n        Empty => println(1)\n        Empty => println(2)\n        _ => println(3)\n    }", "duplicate match arm Empty"),
        ("match s {\n        Square(x) => println(1)\n        _ => println(2)\n    }", "unknown variant Square of Shape"),
        ("match s {\n        Rect(w) => println(w)\n        _ => println(2)\n    }", "Shape.Rect has 2 field(s), but the pattern binds 1"),
        ("match s {\n        Rect(w, w) => println(w)\n        _ => println(2)\n    }", "duplicate binding w"),
        ("let r = 1\n    match s {\n        Circle(r) => println(r)\n        _ => println(2)\n    }", "duplicate binding r"),
        ("let t = Shape.Triangle(1.0)", "unknown variant Triangle of Shape"),
        ("let t = Shape.Circle()", "argument count mismatch for Shape.Circle"),
        ("let t = Shape.Circle(1)", "found the integer 1; write 1.0"),
        ("let t = Shape { r: 1.0 }", "Shape is an enum"),
        ("let t = Point.Origin", "Point is a struct, not an enum"),
        ("match 5 {\n        _ => println(1)\n    }", "match requires an enum value"),
        ("match s {\n        _ => {\n            s = Shape.Empty\n        }\n    }", "borrowed by an enclosing for loop"),
        ("match s {\n        Label(t) => {\n            let owned = t\n        }\n        _ => println(1)\n    }", "borrow escape"),
        ("println(s.r)", "unknown field r"),
        ("println(s)", "cannot print"),
        ("let a = s\n    match s {\n        _ => println(1)\n    }", "use after move: s"),
        ("let x = match s {\n        Circle(r) => {\n            r\n        }\n        _ => 0.0\n    }", "a match expression arm holds a single value"),
        ("let x = match s {\n        Circle(r) => r\n        _ => 1\n    }", "found the integer 1; write 1.0"),
    ] {
        let source = format!(
            "enum Shape {{\n    Circle(r: f64)\n    Rect(w: f64, h: f64)\n    Label(text: string)\n    Empty\n}}\nstruct Point {{\n    x: i64\n}}\nfn main() {{\n    let mut s = Shape.Circle(1.0)\n    {body}\n}}\n"
        );
        let output = case.run(&source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}\n{stderr}");
    }
    for (source, message) in [
        (
            "enum Empty {\n}\nfn main() {\n}\n",
            "expected at least one variant in enum Empty",
        ),
        (
            "enum Bad {\n    lower\n}\nfn main() {\n}\n",
            "enum variant names must start with an uppercase letter",
        ),
        (
            "enum Bad {\n    A\n    A\n}\nfn main() {\n}\n",
            "duplicate variant Bad.A",
        ),
        (
            "enum Bad {\n    A(x: &string)\n}\nfn main() {\n}\n",
            "enums cannot store references",
        ),
        (
            "enum List {\n    Cons(tail: List)\n    Nil\n}\nfn main() {\n}\n",
            "recursive struct layout",
        ),
    ] {
        let output = case.run(source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}\n{stderr}");
    }
}
