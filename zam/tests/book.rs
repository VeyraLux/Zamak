//! The Zamak Book（`book/<语言>/src/*.md`）里的每段代码都要实际编译运行。
//!
//! 围栏约定（写书时照这个标）：
//! - ```zamak：完整程序，`zam run`；紧跟着（中间只隔空行）的 ```text 块是逐字的标准输出。
//! - ```zamak,check：只 `zam check`（读标准输入、文件或参数的程序）。
//! - ```zamak,fails：`zam check` 必须失败；紧跟的 ```text 块是逐字的诊断信息。
//! - ```zamak,panics：能编译，运行时带错误停下；紧跟的 ```text 块是先打印的输出加上
//!   最后一行 `zam: ...` 错误信息。
//! - ```zamak,test：`zam test`，紧跟的 ```text 块是逐字的测试报告。
//! - ```zamak,ignore：片段，不测。
//! - ```zamak,project=名字,file=路径：同名的块合成一个项目（入口 src/main.zm），运行后核对
//!   该项目最后一块后面的 ```text 输出。
//! 中英两版同名章节的代码块数量与标记必须一致，免得两版不同步。

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug)]
struct Block {
    attrs: Vec<String>,
    code: String,
    output: Option<String>,
    line: usize,
}

impl Block {
    fn has(&self, attr: &str) -> bool {
        self.attrs.iter().any(|a| a == attr)
    }
    fn value(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find_map(|a| a.strip_prefix(key)?.strip_prefix('='))
    }
}

fn blocks(text: &str) -> Vec<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<Block> = Vec::new();
    // 上一个 zamak 块的结束行：text 块与它之间只有空行时，才是它的输出。
    let mut last_close: Option<usize> = None;
    let mut i = 0;
    while i < lines.len() {
        let Some(info) = lines[i].trim_end().strip_prefix("```") else {
            i += 1;
            continue;
        };
        let open = i;
        let mut body = String::new();
        i += 1;
        while i < lines.len() && lines[i].trim_end() != "```" {
            body.push_str(lines[i]);
            body.push('\n');
            i += 1;
        }
        let close = i;
        i += 1;
        let mut parts = info.split(',').map(str::trim);
        match parts.next() {
            Some("zamak") => {
                out.push(Block {
                    attrs: parts.map(String::from).collect(),
                    code: body,
                    output: None,
                    line: open + 1,
                });
                last_close = Some(close);
                continue;
            }
            Some("text") => {
                if let (Some(closed), Some(last)) = (last_close, out.last_mut()) {
                    if lines[closed + 1..open].iter().all(|l| l.trim().is_empty()) {
                        last.output = Some(body);
                    }
                }
            }
            _ => {}
        }
        last_close = None;
    }
    out
}

fn chapters(language: &str) -> BTreeMap<String, Vec<Block>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../book")
        .join(language)
        .join("src");
    let mut found = BTreeMap::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                let name = path
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                found.insert(name, blocks(&fs::read_to_string(&path).unwrap()));
            }
        }
    }
    found
}

fn zam(args: &[&str], dir: &Path, cache: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(args)
        .current_dir(dir)
        .env("ZAMAK_CACHE_DIR", cache)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

/// 跑一章：返回失败说明。
fn run_chapter(name: &str, blocks: &[Block], work: &Path, cache: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let mut projects: BTreeMap<String, Vec<&Block>> = BTreeMap::new();
    for (index, block) in blocks.iter().enumerate() {
        let at = format!("{name}:{}", block.line);
        if block.has("ignore") {
            continue;
        }
        if let Some(project) = block.value("project") {
            projects.entry(project.to_owned()).or_default().push(block);
            continue;
        }
        let dir = work.join(format!("{}-{index}", name.replace(['/', '.'], "_")));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("main.zm"), &block.code).unwrap();
        if block.has("fails") {
            let output = zam(&["check", "main.zm"], &dir, cache);
            let stderr = text(&output.stderr);
            if output.status.success() {
                failures.push(format!("{at}: expected a check error"));
            } else if let Some(expected) = &block.output {
                // 书里展示的诊断要与编译器的输出逐字一致（行号、列号、下划线都算）。
                if !stderr.contains(expected.trim_end()) {
                    failures.push(format!("{at}: expected\n{expected}got:\n{stderr}"));
                }
            }
            continue;
        }
        if block.has("test") {
            // `zam test` 的报告逐字核对；有失败的测试时退出码不为 0，这里不看退出码。
            let output = zam(&["test", "main.zm"], &dir, cache);
            let actual = text(&output.stdout);
            if block
                .output
                .as_ref()
                .is_some_and(|expected| &actual != expected)
            {
                failures.push(format!(
                    "{at}: zam test output differs\n{actual}{}",
                    text(&output.stderr)
                ));
            }
            continue;
        }
        if block.has("panics") {
            let output = zam(&["run", "main.zm"], &dir, cache);
            let stderr = text(&output.stderr);
            let expected = block.output.as_deref().unwrap_or_default();
            // 运行时错误前打印的内容照常核对：错误信息前面的行是标准输出。
            let (printed, error) = expected.rsplit_once("zam: ").unwrap_or(("", expected));
            if output.status.success() {
                failures.push(format!("{at}: expected the program to stop with an error"));
            } else if !stderr.contains(&format!("zam: {}", error.trim_end())) {
                failures.push(format!("{at}: expected `zam: {error}`, got:\n{stderr}"));
            } else if text(&output.stdout) != printed {
                failures.push(format!(
                    "{at}: output before the error differs:\n{}",
                    text(&output.stdout)
                ));
            }
            continue;
        }
        let command = if block.has("check") { "check" } else { "run" };
        let output = zam(&[command, "main.zm"], &dir, cache);
        if !output.status.success() {
            failures.push(format!(
                "{at}: zam {command} failed:\n{}",
                text(&output.stderr)
            ));
            continue;
        }
        if let Some(expected) = &block.output {
            let actual = text(&output.stdout);
            if command == "run" && &actual != expected {
                failures.push(format!(
                    "{at}: output differs\n--- expected\n{expected}--- actual\n{actual}"
                ));
            }
        }
    }
    for (project, files) in projects {
        let at = format!("{name}: project {project}");
        let dir = work.join(format!("{}-{project}", name.replace(['/', '.'], "_")));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("zam.toml"),
            format!(
                "[project]\nname = \"{project}\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n"
            ),
        )
        .unwrap();
        for block in &files {
            let file = dir.join(block.value("file").expect("project blocks need file="));
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, &block.code).unwrap();
        }
        let output = zam(&["run", "."], &dir, cache);
        if !output.status.success() {
            failures.push(format!("{at}: zam run failed:\n{}", text(&output.stderr)));
        } else if let Some(expected) = files.iter().rev().find_map(|b| b.output.as_ref()) {
            let actual = text(&output.stdout);
            if &actual != expected {
                failures.push(format!(
                    "{at}: output differs\n--- expected\n{expected}--- actual\n{actual}"
                ));
            }
        }
    }
    failures
}

#[test]
fn english_and_chinese_books_stay_in_sync() {
    let en = chapters("en");
    let zh = chapters("zh-CN");
    assert!(en.len() >= 2, "book/en/src is missing");
    let names = |m: &BTreeMap<String, Vec<Block>>| m.keys().cloned().collect::<Vec<_>>();
    assert_eq!(
        names(&en),
        names(&zh),
        "both books need the same chapter files"
    );
    for (name, blocks) in &en {
        let other = &zh[name];
        let shape = |b: &[Block]| {
            b.iter()
                .map(|x| (x.attrs.clone(), x.output.is_some()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            shape(blocks),
            shape(other),
            "{name}: code blocks differ between en and zh-CN"
        );
    }
}

#[test]
fn every_book_example_runs() {
    let work: PathBuf = std::env::temp_dir().join(format!("zam-book-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    let cache = work.join("cache");
    fs::create_dir_all(&cache).unwrap();
    let mut jobs: Vec<(String, Vec<Block>)> = Vec::new();
    for language in ["en", "zh-CN"] {
        for (name, blocks) in chapters(language) {
            jobs.push((format!("{language}/{name}"), blocks));
        }
    }
    // 每章一个线程：编译 C 是主要耗时，并行能把整本书压到一两分钟。
    let failures: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|(name, blocks)| {
                let (work, cache) = (&work, &cache);
                scope.spawn(move || run_chapter(name, blocks, work, cache))
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    let _ = fs::remove_dir_all(&work);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
