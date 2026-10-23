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
fn bytes_example_reads_writes_and_converts() {
    let case = Case::new("bytes", "example");
    let output = case.run(include_str!("../../examples/values/bytes.zm"));
    assert_eq!(
        stdout(&output),
        "b\"AB\\x00\\xff\\n\"\n5\n65\n10\nb\"Bb\\x00\\xff\\nC\"\nb\"b\\x00\"\n240\ntrue\n-1\n\
b\"h\\xc3\\xa9llo\"\nhéllo\n68c3a96c6c6f\nb\"\\xde\\xad\\xbe\\xef\"\ntrue\n\
Packet(kind: 1, body: b\"hi\")\n[b\"x\", b\"yz\", b\"hi\"]\nb\"xyzhi\"\n6\ninvalid UTF-8 at byte 0\n"
    );
}

#[test]
fn bytes_round_trip_through_files_and_containers() {
    let case = Case::new("bytes", "files");
    let path = case
        .path("data.bin")
        .display()
        .to_string()
        .replace('\\', "/");
    let source = format!(
        "use std/fs\nuse std/bytes\n\nfn main() -> !string:\n    let path = \"{path}\"\n    fs.write_bytes(path, b\"\\x00\\x01\\xfe\")?\n    fs.append_bytes(path, b\"!\")?\n    let data = fs.read_bytes(path)?\n    println(data)\n    let mut index: {{string: bytes}} = {{}}\n    index[\"a\"] = copy(&data)\n    println(index)\n    let mut sorted = copy(&data)\n    sorted.sort()\n    println(sorted)\n    println(max(data))\n    bytes.from_hex(\"abc\") catch e:\n        println(e)\n    bytes.from_hex(\"zz\") catch e:\n        println(e)\n    println(\"\\x41\\x42\\0!\".encode())\n"
    );
    let output = case.run(&source);
    assert_eq!(
        stdout(&output),
        "b\"\\x00\\x01\\xfe!\"\n{\"a\": b\"\\x00\\x01\\xfe!\"}\nb\"\\x00\\x01!\\xfe\"\n254\nodd number of hex digits\ninvalid hex digit at position 0\nb\"AB\\x00!\"\n"
    );
}

#[test]
fn bytes_out_of_range_stop_the_program() {
    let case = Case::new("bytes", "range");
    for body in [
        "    let mut b = b\"a\"\n    b.push(256)\n",
        "    let mut b = b\"a\"\n    b[0] = -1\n",
        "    let mut b = b\"\\xff\"\n    b[0] += 1\n",
        "    let mut b = b\"a\"\n    b.insert(0, 1000)\n",
        "    let b: bytes = [1, 2, 300]\n    println(b)\n",
    ] {
        let output = case.run(&format!("fn main():\n{body}"));
        assert!(!output.status.success(), "accepted {body}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("byte must be in range(0, 256)"),
            "{body}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn bytes_misuse_is_rejected_with_a_clear_message() {
    let case = Case::new("bytes", "errors");
    for (source, message) in [
        (
            "fn main():\n    let mut b = b\"ab\"\n    for x in &mut b:\n        x = 1\n",
            "for x in &mut over bytes is not supported",
        ),
        ("fn main():\n    let b = b\"\\xZ1\"\n", "\\x takes two hex digits"),
        ("fn main():\n    let b = b\"é\"\n", "bytes literals hold ASCII characters only"),
        (
            "fn total(xs: &[i64]) -> i64:\n    sum(xs)\n\nfn main():\n    println(total(b\"ab\"))\n",
            "b\"...\" is bytes, but &[i64] is expected",
        ),
        (
            "fn main():\n    let b = b\"a\"\n    println(b.upper())\n",
            "bytes support len, push",
        ),
        (
            "fn main():\n    let b = b\"a\"\n    let s: string = b\n",
            "type mismatch",
        ),
    ] {
        let output = case.run(source);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "accepted {source}");
        assert!(stderr.contains(message), "{source}: {stderr}");
    }
}
