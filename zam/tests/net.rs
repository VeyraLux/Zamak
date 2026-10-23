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
