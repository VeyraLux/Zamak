mod common;

use common::Case;
use std::process::Output;

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

/// 运行源码，并用 ZAMAK_DUMP_C 取回入口模块生成的 C。
fn run_and_dump(case: &Case, source: &str) -> (Output, String) {
    let file = case.path("main.zm");
    let dump = case.path("dump");
    let _ = std::fs::remove_dir_all(&dump);
    std::fs::write(&file, source).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_zam"))
        .arg("run")
        .arg(&file)
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .env("ZAMAK_DUMP_C", &dump)
        .output()
        .unwrap();
    let c = std::fs::read_to_string(dump.join("main.c")).unwrap_or_default();
    (output, c)
}

#[test]
fn streamed_words_and_lines_match_the_list_versions() {
    // 流式遍历与先建列表再遍历的结果逐字相同：空白、空行、\r\n、末尾有无换行、break 与 continue。
    let case = Case::new("streaming", "same-results");
    let body = "    let mut out = \"\"\n    for w in text.words():\n        if w == \"skip\": continue\n        if w == \"stop\": break\n        out = out + \"[\" + w + \"]\"\n    println(out)\n    let mut n = 0\n    for line in text.lines():\n        n += 1\n        println(f\"{n}:{line}|{len(line)}\")\n";
    for text in [
        "",
        "   ",
        "one",
        "  a  b\\tc\\n",
        "x skip y\\r\\n\\r\\nz stop w\\nlast",
        "trailing\\n\\n",
        "中文 词\\n二",
    ] {
        let streamed = format!("fn main():\n    let text = \"{text}\"\n{body}");
        let (output, c) = run_and_dump(&case, &streamed);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(c.matches("// zam: streamed").count(), 2, "{text}\n{c}");
        // 循环体里提到原文时不做流式遍历：结果必须一样。
        let listed = format!("fn main():\n    let text = \"{text}\"\n    let ws = text.words()\n    let lines = text.lines()\n{}", body.replace("text.words()", "ws").replace("text.lines()", "lines"));
        let (expected, _) = run_and_dump(&case, &listed);
        assert!(
            expected.status.success(),
            "{}",
            String::from_utf8_lossy(&expected.stderr)
        );
        assert_eq!(stdout(&output), stdout(&expected), "{text:?}");
    }
}

#[test]
fn streaming_is_skipped_when_the_text_could_change() {
    let case = Case::new("streaming", "fallback");
    for source in [
        // 可变绑定：循环里可能改写原文。
        "fn main():\n    let mut text = \"a b\"\n    for w in text.words():\n        println(w)\n    text = \"c\"\n    println(text)\n",
        // 循环体提到了原文。
        "fn main():\n    let text = \"a b\"\n    for w in text.words():\n        println(f\"{w} of {text}\")\n",
        // 不是只读借用的参数。
        "fn show(text: string):\n    for w in text.words():\n        println(w)\n\nfn main(): show(\"a b\")\n",
    ] {
        let (output, c) = run_and_dump(&case, source);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(!c.contains("// zam: streamed"), "{source}\n{c}");
        assert!(stdout(&output).starts_with("a\n") || stdout(&output).starts_with("a of"), "{}", stdout(&output));
    }
    // 只读借用参数可以流式遍历。
    let (output, c) = run_and_dump(&case, "fn show(text: &string):\n    for w in text.words():\n        println(w)\n\nfn main(): show(\"a b\")\n");
    assert_eq!(stdout(&output), "a\nb\n");
    assert!(c.contains("// zam: streamed"), "{c}");
}

#[test]
fn map_updates_look_up_once_and_keep_checks() {
    let case = Case::new("streaming", "map-update");
    let (output, c) = run_and_dump(
        &case,
        "fn main():\n    let mut counts: {string: i64} = {}\n    let mut sums: {i64: f64} = {}\n    for w in \"b a b c b\".words():\n        counts[w] = counts.get(w, 0) + 1\n    let mut i = 0\n    while i < 4:\n        let k = i % 2\n        let x = 0.5\n        sums[k] = sums.get(k, 1.0) * x\n        i += 1\n    println(counts)\n    println(sums)\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        stdout(&output),
        "{\"b\": 3, \"a\": 1, \"c\": 1}\n{0: 0.25, 1: 0.25}\n"
    );
    assert!(c.contains("slotMsi(") && c.contains("slotMif("), "{c}");
    // 溢出照常检查。
    let output = case.run("fn main():\n    let mut m: {string: i64} = {\"a\": 9223372036854775807}\n    let k = \"a\"\n    m[k] = m.get(k, 0) + 1\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("integer arithmetic error"));
}
