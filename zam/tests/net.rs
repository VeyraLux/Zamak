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

#[test]
fn tls_server_and_client_talk_over_loopback() {
    // TLS 握手要两边交替收发，单线程做不到：服务端与客户端各是一个 Zamak 进程，共用缓存，
    // Mbed TLS 只编译一次。服务端把端口写到 stderr（不经缓冲），客户端用 SSL_CERT_FILE 信任测试证书。
    use std::io::BufRead;
    use std::process::{Command, Stdio};
    let case = Case::new("net", "tls");
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
    let mut server = zam(
        "server.zm",
        format!(
            "use std/http\nuse std/io\nuse std/net\n\nfn hello(req: &http.Request) -> http.Response:\n    http.respond(200, f\"secure {{req.path}}\")\n\nfn main() -> !string:\n    let listener = net.listen(\"127.0.0.1\", 0)?\n    io.eprintln(f\"port {{listener.port()}}\")\n    http.serve_one(listener, hello, cert_file=\"{cert}\", key_file=\"{key}\")?\n    let conn = listener.accept_tls(\"{cert}\", \"{key}\")?\n    conn.send(conn.recv()?)?\n    listener.accept_tls(\"{cert}\", \"{key}\") catch e:\n        println(\"refused\")\n"
        ),
    )
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
    let client = zam(
        "client.zm",
        format!(
            "use std/http\nuse std/net\n\nfn main() -> !string:\n    let r = http.get(\"https://localhost:{port}/hi\")?\n    println(f\"{{r.status}} {{r.text()?}}\")\n    let mut conn = net.connect(\"127.0.0.1\", {port})?\n    conn.start_tls(\"127.0.0.1\")?\n    conn.send(b\"ping\")?\n    println(conn.recv()?)\n    let mut other = net.connect(\"127.0.0.1\", {port})?\n    other.start_tls(\"example.com\") catch e:\n        println(e)\n"
        ),
    )
    .output()
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while server.try_wait().unwrap().is_none() {
        if std::time::Instant::now() > deadline {
            let _ = server.kill();
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let served = server.wait_with_output().unwrap();
    assert_eq!(
        stdout(&client),
        "200 secure /hi\nb\"ping\"\ncannot start TLS with example.com: the server's certificate is not trusted: The certificate Common Name (CN) does not match with the expected CN\n"
    );
    assert_eq!(stdout(&served), "refused\n");
}
