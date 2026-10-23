mod common;

use common::Case;
use std::io::{Read, Write};
use std::net::TcpListener;

fn stdout(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

#[test]
fn tcp_connects_sends_and_receives_on_loopback() {
    // 单线程也能测：连接在内核里完成握手，数据先进缓冲区。
    let case = Case::new("net", "tcp");
    let output = case.run(
        "use std/net\n\nfn main() -> !string:\n    let server = net.listen(\"127.0.0.1\", 0)?\n    println(server.port() > 0)\n    let client = net.connect(\"127.0.0.1\", server.port())?\n    let conn = server.accept()?\n    client.send(\"ping\".encode())?\n    println(conn.recv()?.decode()?)\n    conn.send(b\"pong\")?\n    println(client.recv()?)\n    net.connect(\"127.0.0.1\", 1, timeout=2.0) catch e:\n        println(e.starts_with(\"cannot connect to 127.0.0.1:1\"))\n    net.listen(\"127.0.0.1\", 70000) catch e:\n        println(e)\n",
    );
    assert_eq!(
        stdout(&output),
        "true\nping\nb\"pong\"\ntrue\ncannot listen on 127.0.0.1:70000: the port must be between 0 and 65535 (error 0)\n"
    );
}

#[test]
fn http_client_reads_plain_chunked_and_json_responses() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let replies = [
            "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nX-Test: a\r\nX-Test: b\r\n\r\nhello",
            "HTTP/1.1 201 Created\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n6;ext=1\r\npedia!\r\n0\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"n\": 3}",
        ];
        let mut requests = Vec::new();
        for reply in replies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut seen = Vec::new();
            let mut buf = [0u8; 4096];
            // 读到头部结束，再按 Content-Length 读完请求体。
            loop {
                let n = stream.read(&mut buf).unwrap();
                seen.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&seen).to_string();
                if let Some(end) = text.find("\r\n\r\n") {
                    let length = text
                        .lines()
                        .find_map(|l| l.strip_prefix("Content-Length: "))
                        .map_or(0, |v| v.trim().parse::<usize>().unwrap());
                    if seen.len() >= end + 4 + length {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            requests.push(String::from_utf8_lossy(&seen).to_string());
            stream.write_all(reply.as_bytes()).unwrap();
        }
        requests
    });
    let case = Case::new("net", "client");
    let output = case.run(&format!(
        "use std/http\nuse std/json\n\nfn main() -> !string:\n    let a = http.get(\"http://127.0.0.1:{port}/first?q=1\")?\n    let tag = a.headers.get(\"x-test\", \"\")\n    println(f\"{{a.status}} {{a.text()?}} {{tag}}\")\n    let b = http.post(\"http://127.0.0.1:{port}/upload\", \"data!\")?\n    println(f\"{{b.status}} {{b.reason}} {{b.text()?}}\")\n    let c = http.get(\"http://127.0.0.1:{port}/json\")?\n    println(c.json()?.get(\"n\")?.as_i64()?)\n"
    ));
    assert_eq!(
        stdout(&output),
        "200 hello a, b\n201 Created Wikipedia!\n3\n"
    );
    let requests = server.join().unwrap();
    assert!(requests[0].starts_with("GET /first?q=1 HTTP/1.1\r\nHost: 127.0.0.1:"));
    assert!(requests[1].starts_with("POST /upload HTTP/1.1\r\n"));
    assert!(requests[1].contains("Content-Length: 5\r\n"));
    assert!(requests[1].ends_with("\r\n\r\ndata!"));
}

#[test]
fn http_server_answers_requests() {
    let case = Case::new("net", "server");
    let output = case.run(
        "use std/http\nuse std/json\nuse std/net\n\nfn handle(req: &http.Request) -> http.Response:\n    if req.path == \"/echo\":\n        let text = req.text() catch e: \"?\"\n        return http.respond(200, f\"{req.method} {req.query} {text}\")\n\n    let mut fields: {string: json.Json} = {}\n    fields[\"path\"] = json.Json.Str(copy(&req.path))\n    http.respond_json(404, json.Json.Object(fields))\n\nfn exchange(listener: &net.Listener, request: &string) -> string!string:\n    let client = net.connect(\"127.0.0.1\", listener.port())?\n    client.send(request.encode())?\n    http.serve_one(listener, handle)?\n    client.recv_all()?.decode()?\n\nfn main() -> !string:\n    let listener = net.listen(\"127.0.0.1\", 0)?\n    println(exchange(listener, \"POST /echo?x=1 HTTP/1.1\\r\\nHost: a\\r\\nContent-Length: 5\\r\\n\\r\\nhello\")?)\n    println(exchange(listener, \"GET /missing HTTP/1.1\\r\\nHost: a\\r\\n\\r\\n\")?)\n",
    );
    assert_eq!(
        stdout(&output),
        "HTTP/1.1 200 OK\nContent-Length: 14\nConnection: close\ncontent-type: text/plain; charset=utf-8\n\nPOST x=1 hello\n\
HTTP/1.1 404 Not Found\nContent-Length: 20\nConnection: close\ncontent-type: application/json\n\n{\"path\": \"/missing\"}\n"
    );
}

/// TLS 握手要两边交替收发，单线程做不到：服务端与客户端各是一个 Zamak 进程，共用缓存，
/// Mbed TLS 只编译一次。服务端把端口写到 stderr（不经缓冲），客户端用 SSL_CERT_FILE 信任测试证书。
/// `server` 拿到证书与私钥的路径，`client` 拿到端口，各自给出程序源码。返回两边的输出。
fn tls_pair(
    name: &str,
    server: impl Fn(&str, &str) -> String,
    client: impl Fn(&str) -> String,
) -> (std::process::Output, std::process::Output) {
    use std::io::BufRead;
    use std::process::{Command, Stdio};
    let case = Case::new("net", name);
    let fixture = |name: &str| {
        std::path::absolute(format!("tests/fixtures/tls/{name}"))
            .unwrap()
            .display()
            .to_string()
            .replace('\\', "/")
    };
    let (cert, key) = (fixture("cert.pem"), fixture("key.pem"));
    let zam = |file: &str, source: String| {
        let path = case.path(file);
        std::fs::write(&path, source).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_zam"));
        command
            .arg("run")
            .arg(path)
            .env("ZAMAK_CACHE_DIR", case.path("cache"))
            .env("ZAMAK_LANG", "en")
            .env("SSL_CERT_FILE", &cert);
        command
    };
    let mut server = zam("server.zm", server(&cert, &key))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut port = String::new();
    let mut log = String::new();
    for line in std::io::BufReader::new(server.stderr.take().unwrap()).lines() {
        let line = line.unwrap();
        if let Some(found) = line.strip_prefix("port ") {
            port = found.to_owned();
            break;
        }
        log += &line;
        log += "\n";
    }
    if port.is_empty() {
        // 先结束服务端，Case 才删得掉它占着的目录。
        let _ = server.kill();
        let _ = server.wait();
        panic!("the server did not start:\n{log}");
    }
    let client = zam("client.zm", client(&port)).output().unwrap();
    // 客户端没按预期连上来（比如没编译过）时服务端还在等连接：连上去再断开，让它的 accept 出错退出。
    // 只杀 `zam run` 没用：程序是它的子进程，还占着输出管道，读输出会一直等下去。
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut waited = 0;
    while server.try_wait().unwrap().is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "the server did not finish; the client said:\n{}",
            String::from_utf8_lossy(&client.stderr)
        );
        waited += 1;
        if waited > 40 {
            let _ = std::net::TcpStream::connect(format!("127.0.0.1:{port}"));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    (client, server.wait_with_output().unwrap())
}

#[test]
fn tls_server_and_client_talk_over_loopback() {
    let (client, served) = tls_pair(
        "tls",
        |cert, key| {
            format!(
                "use std/http\nuse std/io\nuse std/net\n\nfn hello(req: &http.Request) -> http.Response:\n    http.respond(200, f\"secure {{req.path}}\")\n\nfn main() -> !string:\n    let listener = net.listen(\"127.0.0.1\", 0)?\n    io.eprintln(f\"port {{listener.port()}}\")\n    http.serve_one(listener, hello, cert_file=\"{cert}\", key_file=\"{key}\")?\n    let conn = listener.accept_tls(\"{cert}\", \"{key}\")?\n    conn.send(conn.recv()?)?\n    listener.accept_tls(\"{cert}\", \"{key}\") catch e:\n        println(\"refused\")\n"
            )
        },
        |port| {
            format!(
                "use std/http\nuse std/net\n\nfn main() -> !string:\n    let r = http.get(\"https://localhost:{port}/hi\")?\n    println(f\"{{r.status}} {{r.text()?}}\")\n    let mut conn = net.connect(\"127.0.0.1\", {port})?\n    conn.start_tls(\"127.0.0.1\")?\n    conn.send(b\"ping\")?\n    println(conn.recv()?)\n    let mut other = net.connect(\"127.0.0.1\", {port})?\n    other.start_tls(\"example.com\") catch e:\n        println(e)\n"
            )
        },
    );
    assert_eq!(
        stdout(&client),
        "200 secure /hi\nb\"ping\"\ncannot start TLS with example.com: the server's certificate is not trusted: The certificate Common Name (CN) does not match with the expected CN\n"
    );
    assert_eq!(stdout(&served), "refused\n");
}

#[test]
fn threads_make_https_requests_at_the_same_time() {
    // 几个线程同时第一次用 TLS：Mbed TLS 的一次性准备与 PSA 加密库都要经得起并发。
    let (client, served) = tls_pair(
        "tls-threads",
        |cert, key| {
            format!(
                "use std/http\nuse std/io\nuse std/net\n\nfn hello(req: &http.Request) -> http.Response:\n    http.respond(200, f\"page {{req.path}}\")\n\nfn main() -> !string:\n    let listener = net.listen(\"127.0.0.1\", 0)?\n    io.eprintln(f\"port {{listener.port()}}\")\n    for _ in range(4):\n        http.serve_one(listener, hello, cert_file=\"{cert}\", key_file=\"{key}\")?\n    println(\"served 4\")\n"
            )
        },
        |port| {
            format!(
                "use std/http\nuse std/thread\n\nfn fetch(url: string) -> string!string:\n    let r = http.get(url)?\n    r.text()?\n\nfn main() -> !string:\n    let mut tasks: [thread.Task[string]] = []\n    for i in range(4):\n        tasks.push(thread.spawn(fetch, f\"https://localhost:{port}/n{{i}}\"))\n    for t in &tasks:\n        println(t.join()?)\n"
            )
        },
    );
    assert_eq!(stdout(&client), "page /n0\npage /n1\npage /n2\npage /n3\n");
    assert_eq!(stdout(&served), "served 4\n");
}

#[test]
fn urljoin_matches_python() {
    // 期望值是 Python 的 urllib.parse.urljoin 对同样输入的结果。
    let case = Case::new("net", "urljoin");
    let output = case.run(
        r##"use std/http

fn main():
    let base = "https://a.com/x/y/page.html?q=1#top"
    for link in ["b.html", "../c", "./d/", "/root", "//cdn.com/s.js", "?z=2", "#sec", "https://o.org/p", "../../../up", "e/../f", ""]:
        println(http.urljoin(base, link))
    println(http.urljoin("https://a.com", "x"))
"##,
    );
    assert_eq!(
        stdout(&output),
        "https://a.com/x/y/b.html\nhttps://a.com/x/c\nhttps://a.com/x/y/d/\nhttps://a.com/root\nhttps://cdn.com/s.js\nhttps://a.com/x/y/page.html?z=2\nhttps://a.com/x/y/page.html?q=1#sec\nhttps://o.org/p\nhttps://a.com/up\nhttps://a.com/x/y/f\nhttps://a.com/x/y/page.html?q=1#top\nhttps://a.com/x\n"
    );
}
