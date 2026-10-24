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

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

#[test]
fn hashlib_and_base64_match_python() {
    let case = Case::new("std", "hashing");
    let output = case.run(include_str!("../../examples/values/hashing.zm"));
    assert_eq!(
        stdout(&output),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824\n\
5d41402abc4b2a76b9719d911017c592\n\
88aab3ede8d3adf94d26ab90d3bafd4a2083070c3bcce9c014ee04a443847c0b\n\
aGVsbG8=\nhello\n-_8=\n"
    );
    // 补位的边界（55、56、64 字节）与空输入，期望值来自 Python 的 hashlib。
    let output = case.run(
        "use std/hashlib\nuse std/base64\n\nfn main() -> !string:\n    println(hashlib.sha256(b\"\").hex())\n    println(hashlib.sha1(\"a\".repeat(55).encode()).hex())\n    println(hashlib.md5(\"a\".repeat(56).encode()).hex())\n    println(hashlib.sha256(\"a\".repeat(64).encode()).hex())\n    base64.b64decode(\"abc\") catch e:\n        println(e)\n    base64.b64decode(\"ab!c\") catch e:\n        println(e)\n",
    );
    assert_eq!(
        stdout(&output),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n\
c1c8bbdc22796e28c0e15163d20899b65621d65a\n\
3b0c8ac703f828b04c6c197006d17218\n\
ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb\n\
invalid base64: incorrect padding\ninvalid base64 character at position 2\n"
    );
}

#[test]
fn json_reads_writes_and_reports_positions() {
    let case = Case::new("std", "json");
    let output = case.run(include_str!("../../examples/values/json.zm"));
    assert_eq!(
        stdout(&output),
        "Ada\n37\nZamak\n[\"name\", \"langs\", \"age\", \"admin\"]\n\
{\"ok\": true, \"count\": 3, \"tags\": [\"a\", null]}\n\
{\n  \"ok\": true,\n  \"count\": 3,\n  \"tags\": [\n    \"a\",\n    null\n  ]\n}\n\
invalid JSON at position 6: expected a value\n"
    );
    let output = case.run(
        "use std/json\n\nfn main() -> !string:\n    let doc = json.loads(r\"\"\"{\"k\": \"\\u00e9\\ud83d\\ude00\\n\\\"q\\\"\", \"n\": -1.5e2, \"e\": [], \"o\": {}}\"\"\")?\n    println(json.dumps(doc))\n    println(doc.get(\"n\")?.as_f64()?)\n    for bad in [\"[1, 2,]\", \"01\", \"tru\", r\"\"\"\"abc\"\"\", \"[1] 2\", \"\"]:\n        json.loads(bad) catch e:\n            println(e)\n",
    );
    assert_eq!(
        stdout(&output),
        "{\"k\": \"é😀\\n\\\"q\\\"\", \"n\": -150.0, \"e\": [], \"o\": {}}\n-150.0\n\
invalid JSON at position 6: expected a value\n\
invalid JSON at position 1: extra data after the value\n\
invalid JSON at position 3: expected true\n\
invalid JSON at position 4: unterminated string\n\
invalid JSON at position 4: extra data after the value\n\
invalid JSON at position 0: expected a value, found the end of the text\n"
    );
}

#[test]
fn regular_expressions_follow_python_and_run_in_linear_time() {
    let case = Case::new("std", "regex");
    let output = case.run(include_str!("../../examples/values/regex.zm"));
    assert_eq!(
        stdout(&output),
        "2026-10-23: disk full\n2026-10-25: fan\n[\"2026\", \"2026\", \"2026\"]\non 23/10/2026\n\
[\"a\", \"b\", \"c\"]\nemail ada@example.com\n[\"Zamak\", \"reads\", \"Python\"]\n"
    );
    // 期望值来自 Python 的 re。最后一个表达式在回溯引擎里要指数时间。
    let output = case.run(
        "use std/re\n\nfn main() -> !string:\n    println(re.findall(r\"a*\", \"baa\")?)\n    println(re.sub(r\"x*\", \"-\", \"abc\")?)\n    println(re.split(r\"(-)\", \"1-2\")?)\n    println(re.findall(r\"\\bcat\\b\", \"Cat concat cat\", re.IGNORECASE)?)\n    println(re.findall(r\"世.\", \"你好世界\")?)\n    println(re.escape(\"a.b*c\"))\n    println(re.search(r\"(a+)+$\", \"a\".repeat(3000) + \"b\")? == Option.None)\n    for bad in [r\"(a\", r\"a\\1\", r\"(?=a)\", r\"a**\", r\"[z-a]\"]:\n        re.compile(bad) catch e:\n            println(e)\n",
    );
    assert_eq!(
        stdout(&output),
        "[\"\", \"aa\", \"\"]\n-a-b-c-\n[\"1\", \"-\", \"2\"]\n[\"Cat\", \"cat\"]\n[\"世界\"]\na\\.b\\*c\ntrue\n\
invalid regular expression at position 2: missing ) to close a group\n\
invalid regular expression at position 3: backreferences such as \\1 are not supported; Zamak matches in linear time\n\
invalid regular expression at position 3: lookahead, lookbehind and other (?...) groups are not supported\n\
invalid regular expression at position 3: multiple repeat, such as a**\n\
invalid regular expression at position 4: bad character range in a class\n"
    );
}

#[test]
fn processes_run_capture_and_fail_cleanly() {
    let case = Case::new("std", "process");
    // 子进程用 zam 自己：不依赖测试机上装了什么。
    let zam = env!("CARGO_BIN_EXE_zam").replace('\\', "/");
    std::fs::write(
        case.path("upper.zm"),
        "use std/io\n\nfn main() -> !string:\n    println(io.read_all()?.upper())\n",
    )
    .unwrap();
    let upper = case
        .path("upper.zm")
        .display()
        .to_string()
        .replace('\\', "/");
    let version = env!("CARGO_PKG_VERSION");
    let source = format!(
        "use std/process\n\nfn main() -> !string:\n    let version = process.run([r\"{zam}\", \"--version\"])?\n    println(version.code)\n    println(version.stdout.trim() == \"{version}\")\n    let shout = process.check_output([r\"{zam}\", \"run\", r\"{upper}\"], input=\"quiet please\")?\n    println(shout.trim())\n    process.check_output([r\"{zam}\", \"no-such-command\"]) catch e:\n        println(e.starts_with(\"{zam} exited with code\"))\n    process.run([\"zamak-no-such-program\"]) catch e:\n        println(e)\n    process.run([]) catch e:\n        println(e)\n    println(process.call([r\"{zam}\", \"--version\"])?)\n"
    );
    let output = case.run(&source);
    let text = stdout(&output);
    assert!(
        text.starts_with("0\ntrue\nQUIET PLEASE\ntrue\ncannot run zamak-no-such-program: program not found\nprocess: the argument list is empty; put the program first\n"),
        "{text}\n{}",
        stderr(&output)
    );
    assert!(text.ends_with("\n0\n"), "{text}");
}

#[test]
fn raw_triple_quoted_and_unicode_strings() {
    let case = Case::new("std", "strings");
    let output = case.run(
        "fn main():\n    println(r\"C:\\new\\{x}\")\n    println(\"\"\"say \"hi\" {1 + 1}\"\"\")\n    println(r\"\"\"{\"a\": \"\\n\"}\"\"\")\n    println(\"\\u00e9\\U0001F600\")\n",
    );
    assert_eq!(
        stdout(&output),
        "C:\\new\\{x}\nsay \"hi\" 2\n{\"a\": \"\\n\"}\né😀\n"
    );
    let output = case.run("fn main():\n    println(b\"\\u00e9\")\n");
    assert!(stderr(&output).contains("bytes literals take \\xNN escapes, not \\u"));
    let output = case.run("fn main():\n    println(\"\"\"open\n");
    assert!(stderr(&output).contains("a triple-quoted string must end on the same line"));
}

#[test]
fn datetime_matches_python() {
    // 期望值是 Python 的 datetime 对同样输入的输出（布尔值按 Zamak 的写法）。
    let case = Case::new("std", "datetime");
    let output = case.run(include_str!("../../examples/values/dates.zm"));
    assert_eq!(
        stdout(&output),
        "2026-11-01\n6\n305\n2026-12-31\n2025-09-27\n976\nThu Thursday Feb February 29/02/24 060\n1970-01-01\n1969-12-31\n-135080\n2026-11-01T08:05:09.250000+08:00\n1793491509.25\n2026-11-01T00:05:09.250000+00:00\n2026-11-01 08:05:09 08AM +0800 250000 %\n2023-11-14T22:13:20.500000+00:00\n2023-11-15T22:13:20.500000+00:00\n2026-11-01T23:59:00+00:00\nday is out of range for 2023-02: 29\nnot an ISO date (YYYY-MM-DD): 2026/11/01\n29\nfalse\n"
    );
}

#[test]
fn sqlite_stores_and_reads_structs_exactly() {
    let case = Case::new("std", "sqlite");
    let output = case.run(include_str!("../../examples/values/sqlite.zm"));
    assert_eq!(
        stdout(&output),
        concat!(
            "2\n",
            "1 milk pinned=false body=Option.Some(\"2 L\")\n",
            "2 rent pinned=true body=Option.None\n",
            "updated 1\n",
            "2\n",
            "9007199254740993\n",
            "error: no such table: nope\n",
            "error: wrong number of parameters: the statement takes 1, but 0 were given\n",
            "error: the SQL text has more than one statement; use execute_script for several\n",
            "error: :name: the value has no field name\n",
            "error: in the query result: [0].id: the value is not a number\n",
        ),
        "{}",
        stderr(&output)
    );
    // 列表字段没有对应的列类型：直接报错，不悄悄转成文字。
    let output = case.run("use std/sqlite\n\nstruct T:\n    tags: [string]\n\nfn main() -> !string:\n    let db = sqlite.connect(\":memory:\")?\n    db.execute_script(\"CREATE TABLE t (tags TEXT)\")?\n    db.execute_named(\"INSERT INTO t VALUES (:tags)\", &T(tags: [\"a\"])) catch e:\n        println(e)\n");
    assert_eq!(
        stdout(&output),
        "a list can't go in one column; store json.encode(...) in a string field, or use a second table\n",
        "{}",
        stderr(&output)
    );
}

#[test]
fn json_converts_structs_lists_maps_options_and_enums() {
    let case = Case::new("std", "json-structs");
    let output = case.run(include_str!("../../examples/values/json_structs.zm"));
    assert_eq!(
        stdout(&output),
        concat!(
            "[{\"id\": 1, \"text\": \"write\", \"done\": false, \"tags\": [\"work\"], \"due\": \"2026-11-01\", \"shape\": {\"type\": \"Circle\", \"r\": 1.5}, \"scores\": {\"a\": 1}}, {\"id\": 2, \"text\": \"milk\", \"done\": true, \"tags\": [], \"due\": null, \"shape\": \"Empty\", \"scores\": {}}]\n",
            "true\n",
            "Task(id: 3, text: \"x\", done: false, tags: [\"a\", \"b\"], due: Option.None, shape: Shape.Rect(2.0, 3.0), scores: {})\n",
            "error: [0].tags[1]: the JSON value is not a string\n",
            "error: [0].id: the JSON number 1.5 is not an integer\n",
            "error: [0].shape.h: missing\n",
            "error: the JSON value is not an array\n",
            "[[1, 2], [3]]\n",
            "{\n  \"1\": \"one\",\n  \"2\": \"two\"\n}\n"
        )
    );
    // 整数与浮点数分开：读回来类型不变，超出 i64 的整数报错。
    let output = case.run("use std/json\n\nstruct R:\n    n: i64\n    x: f64\n\nfn main() -> !string:\n    let r: R = json.decode(r\"\"\"{\"n\": 9007199254740993, \"x\": 3}\"\"\")?\n    println(r.n)\n    println(json.encode(r))\n    println(json.dumps(json.loads(\"[1, 1.0, 2.5e3]\")?))\n    let big = json.loads(\"99999999999999999999\") catch e:\n        println(e)\n        return\n");
    assert_eq!(
        stdout(&output),
        "9007199254740993\n{\"n\": 9007199254740993, \"x\": 3.0}\n[1, 1.0, 2500.0]\ninvalid JSON at position 20: the integer 99999999999999999999 doesn't fit in i64\n",
        "{}",
        stderr(&output)
    );
    // 函数值没法写成 JSON：在编译时报错。
    let output = case.run("use std/json\n\nstruct S:\n    f: fn(i64) -> i64\n\nfn twice(n: i64) -> i64: n * 2\n\nfn main():\n    println(json.encode(S(f: twice)))\n");
    assert!(stderr(&output).contains("cannot be converted to or from JSON"));
}
