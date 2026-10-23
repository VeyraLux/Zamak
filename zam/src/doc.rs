//! `zam doc`：从项目自己的模块里取出 `pub` 声明，连同它们上方的 `#` 注释，生成一份静态
//! HTML 的 API 页面。
//!
//! 不跑检查器：只按源码的行结构读声明，所以代码写到一半也能出文档（只是可能缺条目）。
//! 注释的写法：紧贴在声明上方、`#` 开头的连续几行；第一行当摘要，其余各行是正文。

use crate::project;
use std::fmt::Write as _;

/// 一个 `pub` 声明。
#[derive(Debug, PartialEq)]
pub struct Item {
    /// `fn`、`struct`、`enum` 或 `const`。
    pub kind: &'static str,
    /// 声明本身（去掉行尾的 `:`），显示成代码。
    pub signature: String,
    /// 名字，同时用作锚点。
    pub name: String,
    /// 上方的注释，已经去掉 `#` 与首尾空白。
    pub docs: Vec<String>,
    /// 结构体的字段、枚举的变体、类型里的方法。
    pub members: Vec<Member>,
}

#[derive(Debug, PartialEq)]
pub struct Member {
    /// 方法（`fn ...`）还是字段、变体。
    pub method: bool,
    pub text: String,
    pub docs: Vec<String>,
}

/// 生成整份文档。
pub fn html(project: &project::Project) -> String {
    let (name, description) = project.summary();
    let sources = project.sources();
    let single = sources.len() == 1;
    let mut pages = Vec::new();
    for (id, source, path) in &sources {
        let title = if single {
            name.to_owned()
        } else {
            (*id).clone()
        };
        pages.push((title, path.to_path_buf(), items(source)));
    }
    render(name, description, &pages)
}

/// 从一份源码里取出所有 `pub` 声明（以及结构体、枚举里的成员）。
fn items(source: &str) -> Vec<Item> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    let mut docs: Vec<String> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start();
        // 注释：攒起来给下一个声明。块里的字段注释也一样处理。
        if let Some(text) = trimmed.strip_prefix('#') {
            docs.push(text.trim().to_owned());
            index += 1;
            continue;
        }
        if trimmed.is_empty() {
            // 空行本身不打断注释：`# doc` 与声明之间偶尔夹一个空行，注释仍属于它。
            // 隔得远的注释会在遇到别的非声明行时清掉。
            index += 1;
            continue;
        }
        let Some((kind, name)) = head(trimmed) else {
            docs.clear();
            index += 1;
            continue;
        };
        let public = trimmed.starts_with("pub ");
        if !public {
            docs.clear();
            index += 1;
            continue;
        }
        // 结构体与枚举：把下面缩进更深的成员读出来。
        let mut members = Vec::new();
        let mut next = index + 1;
        if kind == "struct" || kind == "enum" {
            let mut level: Option<usize> = None;
            let mut member_docs: Vec<String> = Vec::new();
            let mut inner = index + 1;
            while inner < lines.len() {
                let text = lines[inner];
                let trimmed = text.trim_start();
                if trimmed.is_empty() {
                    inner += 1;
                    continue;
                }
                let depth = indent(text);
                if depth == 0 {
                    break;
                }
                // 成员与第一个成员同一缩进；更深的是方法体。
                let here = *level.get_or_insert(depth);
                if depth != here {
                    inner += 1;
                    continue;
                }
                if let Some(doc) = trimmed.strip_prefix('#') {
                    member_docs.push(doc.trim().to_owned());
                    inner += 1;
                    continue;
                }
                let method = trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ");
                // 方法只留签名（去掉函数体）；字段与变体整行都是声明，`x: i64` 里的类型要留着。
                let text = if method {
                    signature(trimmed)
                } else {
                    trimmed
                        .trim_end_matches(|c: char| c == ':' || c.is_whitespace())
                        .to_owned()
                };
                members.push(Member {
                    method,
                    text,
                    docs: std::mem::take(&mut member_docs),
                });
                inner += 1;
            }
            // 成员行已经收进 `members`，外层循环从这里继续（`next` 而不是 `index`，
            // 免得把方法再当成顶层声明，也免得跳过紧跟着的下一个声明）。
            next = inner;
        }
        found.push(Item {
            kind,
            signature: signature(trimmed),
            name,
            docs: std::mem::take(&mut docs),
            members,
        });
        index = next;
    }
    found
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// 声明开头：`pub fn foo(...)` → (`fn`, `foo`)。
fn head(line: &str) -> Option<(&'static str, String)> {
    let line = line.strip_prefix("pub ").unwrap_or(line);
    let line = line.strip_prefix("extern ").unwrap_or(line);
    let (kind, rest) = [
        ("fn", "fn "),
        ("struct", "struct "),
        // `extern type Db: "sqlite3"`：C 句柄，按结构体对待。
        ("struct", "type "),
        ("enum", "enum "),
        ("const", "const "),
    ]
    .iter()
    .find_map(|(kind, prefix)| line.strip_prefix(prefix).map(|rest| (*kind, rest)))?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some((kind, name))
}

/// 声明那一行到哪为止：同一行写了函数体（`fn f(): 1`）时只保留到冒号前。
fn signature(line: &str) -> String {
    let line = line.trim();
    let mut depth = 0i32;
    for (index, c) in line.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ':' if depth == 0 => return line[..index].trim_end().to_owned(),
            _ => {}
        }
    }
    line.to_owned()
}

fn render(
    name: &str,
    description: Option<&str>,
    pages: &[(String, std::path::PathBuf, Vec<Item>)],
) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{name} API</title>\n<style>{STYLE}</style>\n</head>\n<body>\n\
         <header><h1>{name}</h1>\n"
    );
    if let Some(description) = description {
        let _ = writeln!(out, "<p class=\"tagline\">{}</p>", escape(description));
    }
    let _ = write!(
        out,
        "<p class=\"tagline\">API reference</p></header>\n<nav>\n<ul>\n"
    );
    for (title, _, _) in pages {
        let _ = writeln!(
            out,
            "<li><a href=\"#{}\">{}</a></li>",
            anchor(title),
            escape(title)
        );
    }
    let _ = write!(out, "</ul>\n</nav>\n<main>\n");
    for (title, path, items) in pages {
        let _ = write!(
            out,
            "<section id=\"{}\">\n<h2>{}</h2>\n<p class=\"source\">{}</p>\n",
            anchor(title),
            escape(title),
            escape(&path.display().to_string())
        );
        if items.is_empty() {
            let _ = writeln!(
                out,
                "<p class=\"empty\">No <code>pub</code> declarations in this module.</p>"
            );
        }
        for item in items {
            let _ = write!(
                out,
                "<article id=\"{}-{}\">\n<h3><span class=\"kind\">{}</span> <code>{}</code></h3>\n",
                anchor(title),
                anchor(&item.name),
                item.kind,
                escape(&item.signature)
            );
            for line in &item.docs {
                let _ = writeln!(out, "<p>{}</p>", escape(line));
            }
            if !item.members.is_empty() {
                let _ = writeln!(out, "<dl>");
                for member in &item.members {
                    let label = if member.method { "method" } else { "member" };
                    let _ = writeln!(
                        out,
                        "<dt><code>{}</code> <span class=\"kind\">{label}</span></dt>",
                        escape(&member.text)
                    );
                    if !member.docs.is_empty() {
                        let _ = writeln!(out, "<dd>{}</dd>", escape(&member.docs.join(" ")));
                    }
                }
                let _ = writeln!(out, "</dl>");
            }
            let _ = writeln!(out, "</article>");
        }
        let _ = writeln!(out, "</section>");
    }
    let _ = write!(out, "</main>\n</body>\n</html>\n");
    out
}

/// 锚点：只留字母、数字与 `-`，其余换成 `-`。
fn anchor(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const STYLE: &str = "\
:root { color-scheme: light dark; --fg: #1b1b1b; --muted: #666; --line: #ddd; --bg: #fff; --code: #f4f4f4; }
@media (prefers-color-scheme: dark) { :root { --fg: #e8e8e8; --muted: #999; --line: #333; --bg: #161616; --code: #222; } }
* { box-sizing: border-box; }
body { margin: 0; font: 16px/1.6 system-ui, -apple-system, Segoe UI, sans-serif; color: var(--fg); background: var(--bg); }
header { padding: 2rem 1.5rem 1rem; border-bottom: 1px solid var(--line); }
h1 { margin: 0 0 .25rem; font-size: 2rem; }
.tagline { margin: 0; color: var(--muted); }
nav { padding: 1rem 1.5rem; border-bottom: 1px solid var(--line); }
nav ul { display: flex; flex-wrap: wrap; gap: 1rem; margin: 0; padding: 0; list-style: none; }
a { color: inherit; }
main { padding: 1.5rem; max-width: 60rem; }
section { margin-bottom: 3rem; }
.source { color: var(--muted); font-size: .875rem; margin-top: -.5rem; }
article { margin: 1.5rem 0; }
h3 { margin: 0 0 .5rem; font-size: 1rem; }
code, pre { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }
code { background: var(--code); padding: .1rem .35rem; border-radius: .25rem; word-break: break-word; }
.kind { color: var(--muted); font-size: .75rem; text-transform: uppercase; letter-spacing: .04em; }
dl { margin: 0; }
dt { margin-top: .5rem; }
dd { margin: .15rem 0 .5rem 1.25rem; color: var(--muted); }
.empty { color: var(--muted); }
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_take_public_declarations_and_their_comments() {
        let source = "\
const LIMIT = 10

# Adds two numbers.
# It keeps the sum.
pub fn add(a: i64, b: i64) -> i64:
    a + b

fn hidden() -> i64:
    1

# A point.
pub struct Point:
    # The x coordinate.
    x: i64
    y: i64

    # Moves the point.
    fn moved(&self, dx: i64) -> Point:
        Point(x: self.x + dx, y: self.y)

pub enum Light:
    # Stop.
    Red
    Green
";
        let found = items(source);
        let names: Vec<&str> = found.iter().map(|item| item.name.as_str()).collect();
        // `LIMIT` 与 `hidden` 不是 pub，不该出现。
        assert_eq!(names, vec!["add", "Point", "Light"]);
        assert_eq!(found[0].kind, "fn");
        assert_eq!(
            found[0].docs,
            vec!["Adds two numbers.", "It keeps the sum."]
        );
        assert_eq!(found[0].signature, "pub fn add(a: i64, b: i64) -> i64");
        // 结构体的字段与方法都列出来，方法带自己的注释。
        let point = &found[1];
        assert_eq!(point.members.len(), 3);
        assert_eq!(point.members[0].text, "x: i64");
        assert_eq!(point.members[0].docs, vec!["The x coordinate."]);
        assert!(point.members[2].method);
        assert_eq!(point.members[2].docs, vec!["Moves the point."]);
        // 枚举变体。
        assert_eq!(found[2].members.len(), 2);
        assert_eq!(found[2].members[0].docs, vec!["Stop."]);
    }

    #[test]
    fn html_escapes_and_links_every_declaration() {
        use std::path::Path;
        let source = "pub fn less<T>(a: T, b: T) -> bool: a < b\n";
        let found = items(source);
        let pages = vec![("main".to_owned(), Path::new("main.zm").to_path_buf(), found)];
        let page = render("demo", Some("A <demo> project"), &pages);
        assert!(page.starts_with("<!doctype html>"));
        assert!(page.contains("<h1>demo</h1>"));
        // 描述里的尖括号被转义。
        assert!(page.contains("A &lt;demo&gt; project"));
        // 每个声明一个锚点；签名只到返回类型为止，函数体不进文档。
        assert!(page.contains("id=\"main-less\""));
        assert!(page.contains("less&lt;T&gt;"), "{page}");
        assert!(!page.contains("a &lt; b"), "{page}");
        // 单文件项目的页标题用项目名。
        assert!(page.contains("main.zm"));
    }
}
