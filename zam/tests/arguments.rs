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

fn rejects(case: &Case, cases: &[(&str, &str)]) {
    for (source, message) in cases {
        let output = case.run(source);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "accepted {source}");
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}

#[test]
fn defaults_and_keyword_arguments_fill_parameters_by_name() {
    let case = Case::new("arguments", "example");
    let output = case.run(include_str!("../../examples/values/arguments.zm"));
    assert_eq!(
        stdout(&output),
        "a:80 timeout=5.0 retries=3\nb:8080 timeout=5.0 retries=3\nc:80 timeout=1.5 retries=3\n\
d:80 timeout=2.0 retries=0\nhello, Ada\nhello, Ada!\nPoint(x: 1, y: 7)\nPoint(x: 4, y: 2)\n"
    );
}

#[test]
fn defaults_cross_modules_and_evaluate_in_the_callee() {
    // 默认值 PORT 是 net 模块的私有常量：在被调函数的模块里解析，调用方看不见它也没关系。
    let case = Case::new("arguments", "modules");
    let project = case.path("app");
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("zam.toml"),
        "[project]\nname = \"app\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/net.zm"),
        "const PORT = 443\n\npub fn url(host: &string, port: i64 = PORT, secure: bool = true) -> string:\n    let scheme = if secure: \"https\" else: \"http\"\n    f\"{scheme}://{host}:{port}\"\n",
    )
    .unwrap();
    std::fs::write(
        project.join("src/main.zm"),
        "use net\n\nfn main():\n    println(net.url(\"a\"))\n    println(net.url(\"b\", secure=false, port=80))\n",
    )
    .unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&project)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap();
    assert_eq!(stdout(&output), "https://a:443\nhttp://b:80\n");
}

#[test]
fn wrong_keyword_calls_are_rejected() {
    let case = Case::new("arguments", "errors");
    let f = "fn f(a: i64, b: i64 = 2) -> i64: a + b\n\n";
    rejects(
        &case,
        &[
            (
                &format!("{f}fn main(): println(f(1, c=3))\n"),
                "f got an unexpected keyword argument c; its parameters are a, b",
            ),
            (
                &format!("{f}fn main(): println(f(1, a=3))\n"),
                "f got multiple values for argument a",
            ),
            (
                &format!("{f}fn main(): println(f(b=1, 2))\n"),
                "positional argument follows keyword argument",
            ),
            (
                &format!("{f}fn main(): println(f(b=3))\n"),
                "missing argument a in the call to f",
            ),
            (
                "fn main():\n    let g = fn(x: i64) -> i64: x\n    println(g(x=1))\n",
                "keyword argument x=... only goes in a call to a function or method",
            ),
            (
                "use std/math\nfn main(): println(math.sqrt(x=4.0))\n",
                "sqrt takes positional arguments only",
            ),
            (
                "fn h(a: i64 = 1, b: i64) -> i64: a\nfn main(): pass\n",
                "parameter b needs a default value",
            ),
            (
                "fn h(a: i64 = len(\"x\")) -> i64: a\nfn main(): pass\n",
                "a default value is a constant",
            ),
            (
                "fn h(a: i64 = \"x\") -> i64: a\nfn main(): pass\n",
                "the default value of a is not i64",
            ),
            (
                "fn h(a: &[i64] = [\"x\"]): pass\nfn main(): pass\n",
                "the default value of a is not &[i64]",
            ),
        ],
    );
}

#[test]
fn list_and_map_defaults_are_fresh_each_call() {
    // `= []`、`= {}`、`= [1, 2]`：每次调用都是新的一份（不像 Python 的可变默认值那样被共用）。
    let case = Case::new("arguments", "collection-defaults");
    let output = case.run("fn show(name: &string, tags: &[string] = [], extra: &{string: i64} = {}, base: [i64] = [1, 2]):\n    let mut mine = base\n    mine.append(len(tags))\n    println(f\"{name} {tags} {len(extra)} {mine}\")\n\nfn main():\n    show(\"a\")\n    let t: [string] = [\"x\"]\n    let e: {string: i64} = {\"k\": 1}\n    show(\"b\", t, extra=e)\n    show(\"c\")\n");
    assert_eq!(
        stdout(&output),
        "a [] 0 [1, 2, 0]\nb [\"x\"] 1 [1, 2, 1]\nc [] 0 [1, 2, 0]\n"
    );
}

#[test]
fn sets_add_test_and_combine() {
    let case = Case::new("arguments", "sets");
    let output = case.run(include_str!("../../examples/collections/sets.zm"));
    assert_eq!(
        stdout(&output),
        "5\ntrue\ntrue\ntrue\nfalse\n[3, 5, 7, 11]\n[1, 3, 5, 7, 9, 11]\n[3, 5, 7]\n[11]\ntrue\n\
{\"ada\", \"grace\"}\nset()\ntrue\n3\n"
    );
    let output = case.run(
        "fn total(s: &{i64}) -> i64:\n    let mut t = 0\n    for x in s:\n        t += x\n    t\n\nfn main():\n    let mut s = {1, 2, 3}\n    let c = copy(&s)\n    s.clear()\n    println(total(c))\n    println(s)\n    println(c == {3, 2, 1})\n",
    );
    assert_eq!(stdout(&output), "6\nset()\ntrue\n");
    rejects(
        &case,
        &[
            (
                "fn main():\n    let s = {1, 2}\n    println(s[1])\n",
                "a set has no index",
            ),
            (
                "fn main():\n    let mut s = {1, 2}\n    s[4] = 1\n",
                "a set has no index",
            ),
            (
                "fn main():\n    let s = {1, 2}\n    for k, v in &s:\n        println(k)\n",
                "a set has no values",
            ),
            (
                "fn main():\n    let s = {1, 2}\n    println(s.keys())\n",
                "sets support len, add",
            ),
            (
                "fn main():\n    let s = {1.5, 2.5}\n",
                "sets hold i64 or string values",
            ),
        ],
    );
}
