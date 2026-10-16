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

#[test]
fn character_literals_are_code_points() {
    let case = Case::new("text", "chars");
    let output = case.run("use std/string\nfn main():\n    println('a')\n    println('{')\n    println('\\n')\n    println('\\'')\n    println('\\\\')\n    println('中')\n    let text = \"#title\"\n    println(string.byte_at(&text, 0) == '#')\n    let digit = '7' - '0'\n    println(digit)\n");
    assert_eq!(stdout(&output), "97\n123\n10\n39\n92\n20013\ntrue\n7\n");
    for (source, message) in [
        (
            "fn main():\n    println('ab')\n",
            "character literal must contain exactly one character",
        ),
        (
            "fn main():\n    println('')\n",
            "character literal must contain exactly one character",
        ),
        (
            "fn main():\n    println('\\q')\n",
            "unsupported character escape",
        ),
        (
            "fn main():\n    println('a\n",
            "character literal must contain exactly one character",
        ),
    ] {
        let output = case.run(source);
        assert!(!output.status.success(), "accepted {source}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}

#[test]
fn string_search_replace_and_case() {
    let case = Case::new("text", "helpers");
    let output = case.run(
        "use std/string\nfn main():\n    let s = \"a-b-a-b\"\n    println(string.find_from(&s, \"a\", 1))\n    println(string.find_from(&s, \"a\", 3))\n    println(string.find_from(&s, \"z\", 0))\n    println(string.find_from(&s, \"\", 7))\n    println(string.starts_with(&s, \"a-\"))\n    println(string.starts_with(&s, \"b\"))\n    println(string.ends_with(&s, \"-b\"))\n    println(string.ends_with(\"\", \"x\"))\n    println(string.contains(&s, \"b-a\"))\n    println(string.contains(&s, \"c\"))\n    println(string.replace(&s, \"-\", \" + \"))\n    println(string.replace(&s, \"a-b\", \"\"))\n    println(string.replace(&s, \"\", \"x\"))\n    println(string.replace(\"中中\", \"中\", \"文\"))\n    println(string.lower(\"Hello, 世界 ABC\"))\n    println(string.upper(\"Hello, 世界 abc\"))\n",
    );
    assert_eq!(
        stdout(&output),
        "4\n4\n-1\n7\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\na + b + a + b\n-\na-b-a-b\n文文\nhello, 世界 abc\nHELLO, 世界 ABC\n"
    );
    // 与 Python 相同：起点超过末尾时找不到，不报错。
    let output = case.run("use std/string\nfn main():\n    let s = \"ab\"\n    println(string.find_from(&s, \"a\", 3))\n");
    assert_eq!(stdout(&output), "-1\n");
}

/// 按字符处理：chars/codepoints/from_code/bytes/from_bytes/repeat/count/pad_left/pad_right。
#[test]
fn characters_bytes_and_padding() {
    let case = Case::new("text", "characters");
    let source = include_str!("../../examples/values/chars.zm");
    assert_eq!(
        stdout(&case.run(source)),
        "9\n!好你 kamaZ\n20320 97\n好!\n195 169\né\nababab\n2\n|amy   |  30|\n|李雷    |  20|\n|bob   |  30|\n"
    );
    let output = case.run("use std/string\nfn main():\n    println(string.repeat(\"x\", -2) == \"\")\n    println(string.count(\"aaaa\", \"aa\"))\n    println(string.count(\"你好\", \"\"))\n    let empty = string.chars(\"\")\n    println(empty.len())\n    println(string.pad_left(\"long\", 2))\n");
    assert_eq!(stdout(&output), "true\n2\n3\n0\nlong\n");
    for (body, message) in [
        ("println(string.from_code(55296))", "zam: invalid code point"),
        ("println(string.from_code(-1))", "zam: invalid code point"),
        (
            "let bytes: [i64] = [256]\n    let s = string.from_bytes(&bytes) catch e:\n        println(e)\n        return\n    println(s)",
            "",
        ),
    ] {
        let source = format!("use std/string\nfn main():\n    {body}\n");
        let output = case.run(&source);
        if message.is_empty() {
            assert_eq!(stdout(&output), "byte out of range\n");
        } else {
            assert!(!output.status.success(), "accepted {source}");
            assert!(String::from_utf8_lossy(&output.stderr).contains(message));
        }
    }
    let output = case.run("use std/string\nfn main() -> !string:\n    let bytes: [i64] = [228, 189]\n    println(string.from_bytes(&bytes)?)\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid UTF-8"));
}
