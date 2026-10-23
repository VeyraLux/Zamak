//! examples/apps 里的真实程序：端到端运行，确认它们一直能用。
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;

fn zam(args: &[&str], cache: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(args)
        .env("ZAMAK_CACHE_DIR", cache)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap()
}

/// 一个很小的网站：首页有正常链接、404、301 跳转与重复链接，其他路径按表回复。
fn serve_site() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut seen = Vec::new();
            let mut buf = [0u8; 4096];
            while !String::from_utf8_lossy(&seen).contains("\r\n\r\n") {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => seen.extend_from_slice(&buf[..n]),
                }
            }
            let request = String::from_utf8_lossy(&seen).to_string();
            let path = request.split(' ').nth(1).unwrap_or("/").to_owned();
            let reply = match path.as_str() {
                "/" => {
                    let body = format!(
                        "<a href=\"/ok\">ok</a> <a href='docs/page.html'>page</a> <a href=\"/missing\">x</a> <a href=\"/moved\">m</a> <a href=\"http://127.0.0.1:{port}/ok\">again</a> <a href=\"mailto:a@b.c\">mail</a>"
                    );
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                }
                "/ok" | "/docs/page.html" => {
                    "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok".to_owned()
                }
                "/moved" => {
                    "HTTP/1.1 301 Moved\r\nLocation: /ok\r\nContent-Length: 0\r\n\r\n".to_owned()
                }
                _ => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_owned(),
            };
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    port
}

#[test]
fn linkcheck_finds_broken_links_and_follows_redirects() {
    let port = serve_site();
    let root = std::env::temp_dir().join(format!("zam-apps-linkcheck-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let output = zam(
        &[
            "run",
            "../examples/apps/linkcheck",
            "--",
            &format!("http://127.0.0.1:{port}/"),
            "--threads",
            "3",
        ],
        &root.join("cache"),
    );
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    assert_eq!(
        output.status.code(),
        Some(1),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.starts_with(&format!("4 links on http://127.0.0.1:{port}/\n")),
        "{stdout}"
    );
    for line in [
        format!("ok   200 http://127.0.0.1:{port}/ok\n"),
        format!("ok   200 http://127.0.0.1:{port}/docs/page.html\n"),
        format!("ok   200 http://127.0.0.1:{port}/moved\n"),
        format!("FAIL status 404 http://127.0.0.1:{port}/missing\n"),
    ] {
        assert!(stdout.contains(&line), "missing {line:?} in\n{stdout}");
    }
    assert!(stdout.ends_with("3 ok, 1 broken\n"), "{stdout}");
    let usage = zam(&["run", "../examples/apps/linkcheck"], &root.join("cache"));
    assert!(String::from_utf8_lossy(&usage.stderr).contains("usage: linkcheck <url>"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn todo_adds_lists_edits_and_saves_tasks() {
    let root = std::env::temp_dir().join(format!("zam-apps-todo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let file = root.join("todo.json");
    let todo = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_zam"))
            .args(["run", "../examples/apps/todo", "--"])
            .args(args)
            .env("ZAMAK_CACHE_DIR", root.join("cache"))
            .env("ZAMAK_LANG", "en")
            .env("TODO_FILE", &file)
            .output()
            .unwrap();
        (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
            String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        )
    };
    assert_eq!(todo(&["list"]).1, "nothing to do\n");
    assert_eq!(
        todo(&[
            "add",
            "write",
            "the",
            "report",
            "--due",
            "2999-01-01",
            "--tag",
            "work"
        ])
        .1,
        "added 1\n"
    );
    assert_eq!(
        todo(&["add", "pay", "rent", "--due", "2000-01-01"]).1,
        "added 2\n"
    );
    assert_eq!(
        todo(&["add", "buy", "milk", "--tag", "home"]).1,
        "added 3\n"
    );
    assert_eq!(todo(&["done", "3"]).1, "done 3\n");
    assert_eq!(
        todo(&["edit", "1", "write", "the", "annual", "report"]).1,
        "edited 1\n"
    );
    let (_, listed, _) = todo(&["list"]);
    let lines: Vec<&str> = listed.lines().collect();
    assert_eq!(lines.len(), 2, "{listed}");
    assert!(
        lines[0].starts_with("[ ]   1  write the annual report  (2999-01-01, due in ")
            && lines[0].ends_with("d)  #work"),
        "{listed}"
    );
    assert!(
        lines[1].starts_with("[ ]   2  pay rent  (2000-01-01, overdue "),
        "{listed}"
    );
    assert_eq!(
        todo(&["list", "--all", "--tag", "home"]).1,
        "[x]   3  buy milk  #home\n"
    );
    assert_eq!(todo(&["remove", "2"]).1, "remove 2\n");
    let (ok, _, err) = todo(&["done", "9"]);
    assert!(!ok && err.contains("no task 9"), "{err}");
    let (ok, _, err) = todo(&["add", "x", "--due", "2026-13-01"]);
    assert!(
        !ok && err.contains("month must be in 1..12, not 13"),
        "{err}"
    );
    let saved = std::fs::read_to_string(&file).unwrap();
    assert!(
        saved.contains("\"text\": \"write the annual report\""),
        "{saved}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
