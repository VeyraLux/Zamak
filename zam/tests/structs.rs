use std::fs;
use std::process::Command;

#[test]
fn struct_declarations_and_rejections() {
    let path = std::env::temp_dir().join(format!("zam-structs-{}.zm", std::process::id()));
    for (source, accepted) in [
        (
            "pub struct Person:\n    name: string\n    age: i64\n    active: bool\nfn main(): pass",
            true,
        ),
        ("struct Empty: pass\nfn main(): pass", true),
        (
            "struct Person:\n    name: string\n    name: bool\nfn main(): pass",
            false,
        ),
        ("struct Person:\n    name: &string\nfn main(): pass", false),
        (
            "struct Person:\n    name: &mut string\nfn main(): pass",
            false,
        ),
        (
            "struct Person: pass\nstruct Person: pass\nfn main(): pass",
            false,
        ),
        (
            "struct Person: pass\nfn Person(): pass\nfn main(): pass",
            false,
        ),
        (
            "fn Person(): pass\nstruct Person: pass\nfn main(): pass",
            false,
        ),
        ("struct string: pass\nfn main(): pass", false),
        ("struct Person:\n    child: Person\nfn main(): pass", false),
        ("struct Person: pass\nfn main(): let p = Person()", true),
        (
            "struct Person:\n    x: i64\nfn main(): let p = Person(1)",
            false,
        ),
        (
            "struct Person:\n    x: i64\nfn main(): let p = Person(x: 1 y: 2)",
            false,
        ),
        ("fn Helper(): pass\nfn main(): pass", false),
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&path)
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            accepted,
            "{source}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let declaration = "struct Person:\n    name: string\n    age: i64\n";
    for body in [
        "let p = Person(name: \"a\")",
        "let p = Person(\n        name: \"a\",\n        age: true,\n    )",
        "let p = Person(\n        name: \"a\",\n        age: 1,\n        age: 2,\n    )",
        "let p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    let q = p\n    println(p.age)",
        "let p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    let s = p.name",
    ] {
        fs::write(&path, format!("{declaration}fn main():\n    {body}")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&path)
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {body}");
    }
    fs::write(&path, format!("{declaration}fn main():\n    let mut p = Person(\n        name: \"first\",\n        age: 41,\n    )\n    let q = p\n    println(q.age + 1)\n    p = Person(\n        name: \"second\",\n        age: 7,\n    )\n    println(p.age)" )).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&path)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("42\n7\n"));
    for body in [
        "let p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    p.age = 2",
        "let mut p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    p.age = true",
        "let mut p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    p.missing = 2",
        "let mut p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    let q = p\n    p.name = \"b\"",
        "let mut p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    let s = \"b\"\n    p.name = s\n    println(s)",
        "let mut p = Person(\n        name: \"a\",\n        age: 1,\n    )\n    p.name = p.name",
    ] {
        fs::write(&path, format!("{declaration}fn main():\n    {body}")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&path)
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {body}");
    }
    fs::write(&path, format!("{declaration}fn replacement() -> string: return \"updated\"\nfn main():\n    let mut p = Person(\n        name: \"first\",\n        age: 0,\n    )\n    println(p.name)\n    println(p.name)\n    while p.age < 3:\n        p.name = replacement()\n        p.age = p.age + 1\n    let s = \"moved\"\n    p.name = s\n    println(p.name)\n    println(p.age)\n    let q = p\n    println(q.name)" )).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&path)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("first\nfirst\nmoved\n3\nmoved\n"));
    for source in [
        "fn take(p: Missing): pass\nfn main(): pass",
        "struct Person: pass\nfn take() -> &Person: pass\nfn main(): pass",
        "struct Person: pass\nfn make() -> Person: return 1\nfn main(): pass",
        "struct Person:\n    name: string\nfn take(p: Person): pass\nfn main():\n    let p = Person(name: \"a\")\n    take(p)\n    take(p)",
        "struct Person: pass\nstruct Other: pass\nfn take(p: Person): pass\nfn main(): take(Other())",
        "struct Person: pass\nfn main(): let p: Missing = Person()",
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam")).arg("check").arg(&path).output().unwrap();
        assert!(!output.status.success(), "accepted {source}");
    }
    fs::write(&path, "struct Person:\n    name: string\nfn make() -> Person: Person(name: \"returned\")\nfn relay(p: Person) -> Person: return p\nfn consume(p: Person): println(p.name)\nfn main():\n    let p: Person = relay(make())\n    consume(p)\n    relay(make())\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&path)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("returned\n"));
    for source in [
        "struct A:\n    b: B\nstruct B:\n    a: A\nfn main(): pass",
        "struct A:\n    b: Missing\nfn main(): pass",
        "struct B:\n    s: string\nstruct A:\n    b: B\nfn main():\n    let a = A(b: B(s: \"x\"))\n    let b = a.b",
        "struct B:\n    s: string\nstruct A:\n    b: B\nfn main():\n    let a = A(b: B(s: \"x\"))\n    println(a.b < a.b)",
        "struct B:\n    s: string\nstruct A:\n    b: B\nfn main():\n    let a = A(b: B(s: \"x\"))\n    println(a == 1)",
        "struct B:\n    s: string\nstruct A:\n    b: B\nfn main():\n    let mut a = A(b: B(s: \"x\"))\n    a.b = a.b",
    ] {
        fs::write(&path, source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&path)
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {source}");
    }
    fs::write(&path, "struct Leaf:\n    name: string\nstruct Branch:\n    leaf: Leaf\nstruct Root:\n    branch: Branch\nfn leaf() -> Leaf: Leaf(name: \"nested\")\nfn root() -> Root: Root(branch: Branch(leaf: leaf()))\nfn consume(r: Root): println(8)\nfn main():\n    let mut r = root()\n    r.branch = Branch(leaf: leaf())\n    r.branch.leaf = leaf()\n    r.branch.leaf.name = \"deep\"\n    println(r.branch.leaf.name)\n    let q = r\n    consume(q)\n    r = root()\n    root()\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&path)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("deep\n8\n"));
    let nested = "struct Leaf:\n    name: string\n    age: i64\nstruct Root:\n    leaf: Leaf\n";
    for body in [
        "let r = Root(leaf: Leaf(\n            name: \"a\",\n            age: 1,\n        ))\n    r.leaf.age = 2",
        "let mut r = Root(leaf: Leaf(\n            name: \"a\",\n            age: 1,\n        ))\n    r.leaf.age = true",
        "let r = Root(leaf: Leaf(\n            name: \"a\",\n            age: 1,\n        ))\n    println(r.leaf.age.bad)",
        "let r = Root(leaf: Leaf(\n            name: \"a\",\n            age: 1,\n        ))\n    let q = r\n    println(r.leaf.name)",
        "let r = Root(leaf: Leaf(\n            name: \"a\",\n            age: 1,\n        ))\n    let s = r.leaf.name",
        "let r = Root(leaf: Leaf(\n            name: \"a\",\n            age: 1,\n        ))\n    println(r.leaf.missing)",
    ] {
        fs::write(&path, format!("{nested}fn main():\n    {body}")).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .arg("check")
            .arg(&path)
            .env("ZAMAK_LANG", "en")
            .output()
            .unwrap();
        assert!(!output.status.success(), "accepted {body}");
    }
    fs::write(&path, format!("{nested}fn main():\n    let mut r = Root(leaf: Leaf(\n            name: \"first\",\n            age: 1,\n        ))\n    println(r.leaf.name)\n    r.leaf.name = \"second\"\n    r.leaf.age = r.leaf.age + 1\n    println(r.leaf.age)\n    println(r.leaf.name)\n    let q = r\n    println(q.leaf.name)" )).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&path)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).ends_with("first\n2\nsecond\nsecond\n"));
    let legacy = path.with_extension("zam");
    fs::copy(&path, &legacy).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("check")
        .arg(&legacy)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "legacy source extension must be rejected"
    );
    fs::remove_file(legacy).unwrap();
    fs::remove_file(path).unwrap();
}
