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
    assert!(!output.status.success(), "the program should fail");
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

#[test]
fn threads_return_values_and_split_work() {
    let case = Case::new("threads", "split");
    let output = case.run(include_str!("../../examples/concurrency/threads.zm"));
    assert_eq!(
        stdout(&output),
        "4 threads: 18460000000\none thread: 18460000000\n"
    );
}

#[test]
fn queues_carry_structs_between_worker_threads() {
    let case = Case::new("threads", "pipeline");
    let output = case.run(include_str!("../../examples/concurrency/pipeline.zm"));
    assert_eq!(
        stdout(&output),
        "4 words: the quick brown fox\n2 words: jumps over\n3 words: the lazy dog\n1 words: a\n5 words: threads share nothing but queues\n"
    );
}

#[test]
fn failures_joins_and_closed_queues_report_errors() {
    let case = Case::new("threads", "errors");
    let output = case.run(
        r#"use std/thread

fn checked(n: i64) -> i64!string:
    if n > 2:
        fail f"too big: {n}"
    n

fn quiet(names: [string], lookup: {string: i64}) -> !string:
    if len(names) != len(lookup):
        fail "sizes differ"

fn main() -> !string:
    thread.spawn(checked, 5).join() catch e:
        println(e)
    let ok = thread.spawn(checked, 2)
    println(ok.join()?)
    ok.join() catch e:
        println(e)
    let mut lookup: {string: i64} = {}
    lookup["a"] = 1
    let names: [string] = ["a", "b"]
    thread.spawn(quiet, names, lookup).join() catch e:
        println(e)
    let q: thread.Queue[[i64]] = thread.queue()
    q.put([1, 2])?
    q.put([3])?
    println(q.len())
    q.close()
    q.put([4]) catch e:
        println(e)
    println(q.get())
    println(q.get())
    println(q.get())
    let left: thread.Queue[string] = thread.queue()
    left.put("never taken")?
    let unjoined = thread.spawn(checked, 1)
    println(thread.cpu_count() > 0)
"#,
    );
    assert_eq!(
        stdout(&output),
        "too big: 5\n2\nthis thread was already joined\nsizes differ\n2\nthe queue is closed\nOption.Some([1, 2])\nOption.Some([3])\nOption.None\ntrue\n"
    );
}

#[test]
fn many_threads_print_whole_lines_and_draw_their_own_random_numbers() {
    let case = Case::new("threads", "print");
    let output = case.run(
        r#"use std/random
use std/thread

fn talk(id: i64):
    for i in range(200):
        println(f"thread {id} line {i} ok")

fn draw(n: i64) -> i64:
    let mut total = 0
    for _ in range(n):
        total += random.randint(1, 6)
    total

fn main() -> !string:
    let mut talkers: [thread.Thread] = []
    for id in range(8):
        talkers.push(thread.spawn(talk, id))
    for t in &talkers:
        t.join()?
    let a = thread.spawn(draw, 1000)
    let b = thread.spawn(draw, 1000)
    let total = a.join()? + b.join()?
    println(total >= 2000 and total <= 12000)
"#,
    );
    let text = stdout(&output);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 8 * 200 + 1);
    for line in &lines[..lines.len() - 1] {
        let parts: Vec<&str> = line.split(' ').collect();
        assert!(
            parts.len() == 5 && parts[0] == "thread" && parts[2] == "line" && parts[4] == "ok",
            "a line was split: {line:?}"
        );
    }
    assert_eq!(lines[lines.len() - 1], "true");
}

#[test]
fn spawn_rejects_borrows_closures_and_wrong_arguments() {
    let cases = [
        (
            "fn show(name: &string):\n    println(name)\n\nfn main():\n    let name = \"a\"\n    thread.spawn(show, name)\n",
            "show borrows its parameter name; a thread function takes its arguments by value",
        ),
        (
            "fn main():\n    thread.spawn(fn(): println(1))\n",
            "thread.spawn takes the name of a function, then its arguments, such as thread.spawn(work, 1)",
        ),
        (
            "fn work(n: i64) -> i64:\n    n\n\nfn main():\n    thread.spawn(work, 1, 2)\n",
            "argument count mismatch: work takes 1 arguments",
        ),
        (
            "fn work(n: i64) -> i64:\n    n\n\nfn main():\n    thread.spawn(work, \"one\")\n",
            "type mismatch",
        ),
    ];
    for (index, (body, expected)) in cases.iter().enumerate() {
        let case = Case::new("threads", &format!("reject{index}"));
        let output = case.run(&format!("use std/thread\n\n{body}"));
        let text = stderr(&output);
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn generic_functions_infer_types_from_the_annotation() {
    let case = Case::new("threads", "annotation");
    let source = "struct Box[T]:\n    items: [T]\n\nfn empty[T]() -> Box[T]:\n    Box(items: [])\n\nfn main():\n    let mut b: Box[string] = empty()\n    b.items.push(\"x\")\n    println(b)\n    let c: Box[i64] = empty()\n    println(len(c.items))\n";
    let output = case.run(source);
    assert_eq!(stdout(&output), "Box(items: [\"x\"])\n0\n");
    let output = case.run(&format!("{source}    let d = empty()\n"));
    let text = stderr(&output);
    assert!(
        text.contains("cannot infer T for empty: T must appear in a parameter type, or in the type annotation of the variable that takes the result"),
        "{text}"
    );
}
