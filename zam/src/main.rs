mod compiler;
mod native;
mod project;

use std::path::Path;

fn execute() -> project::Result<()> {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let json_count = raw_args
        .iter()
        .filter(|arg| arg.as_str() == "--json")
        .count();
    let json = json_count == 1;
    let args: Vec<String> = raw_args
        .iter()
        .filter(|arg| arg.as_str() != "--json")
        .cloned()
        .collect();
    if json_count > 1 {
        return Err("usage: --json may be specified once".into());
    }
    let valid_shape = match args.first().map(String::as_str) {
        Some("check" | "build" | "fmt" | "run") => matches!(args.len(), 1..=2),
        Some("new") => {
            args.len() == 2
                || (args.len() == 4 && args[2] == "--template" && args[3] == "ai")
                || (args.len() == 3 && args[2] == "--ai")
        }
        Some("demo" | "--version" | "-V" | "--help" | "-h") => args.len() == 1,
        None => args.is_empty(),
        _ => false,
    };
    let option_as_path = args.get(1).is_some_and(|arg| arg.starts_with('-'));
    let missing_command = raw_args.first().is_some_and(|arg| arg == "--json");
    if !valid_shape || option_as_path || missing_command {
        return Err("usage: zam <check|build|run|fmt|new> [project|file] [--template ai]".into());
    }
    let input = Path::new(args.get(1).map(String::as_str).unwrap_or("."));
    if json {
        // ponytail: 只有 check/build 会产生诊断；其余命令直接拒绝，避免静默忽略。
        let outcome = match args.first().map(String::as_str) {
            Some("check") => project::Project::load(input).and_then(|project| project.check()),
            Some("build") => project::Project::load(input)
                .and_then(|project| project.build(&project::cache_dir()).map(|_| ())),
            _ => return Err("--json is only supported by check and build".into()),
        };
        println!(
            "{}",
            report(
                args.first().map(String::as_str).unwrap(),
                &input.display().to_string(),
                &outcome
            )
        );
        // 失败时 main 仍会把人类可读的一行写到 stderr，stdout 保持是合法 JSON。
        return outcome;
    }
    match args.first().map(String::as_str) {
        Some("check") => {
            let project = project::Project::load(input)?;
            project.check()?;
            println!("CHECK OK {}", project.name);
        }
        Some("build") => { project::Project::load(input)?.build(&project::cache_dir())?; }
        Some("fmt") => format_sources(input)?,
        Some("new") if args.len() == 2 => new_project(input, false)?,
        Some("new") if args.len() == 4 && args[2] == "--template" && args[3] == "ai" => {
            new_project(input, true)?
        }
        Some("new") if args.len() == 3 && args[2] == "--ai" => new_project(input, true)?,
        Some("new") => return Err("usage: zam new <directory> [--template ai]".into()),
        Some("run") => {
            // 源码后缀统一为 .zm；其余已存在的文件按原生产物直接执行。
            let source = input.extension().is_some_and(|ext| ext == "zm");
            let artifact = if input.is_file() && !source { input.into() }
                else { project::Project::load(input)?.build(&project::cache_dir())? };
            project::run_artifact(&artifact)?;
        }
        Some("demo") if args.len() == 1 => demo()?,
        Some("--version") | Some("-V") => println!("{}", env!("CARGO_PKG_VERSION")),
        None | Some("--help") | Some("-h") => println!("zam {}\n  zam check [project|file]\n  zam build [project|file]\n  zam fmt [project|file]\n  zam run [project|file|artifact]\n  zam new <directory> [--template ai]\n  zam check --json [project|file]\n  zam build --json [project|file]\n  zam demo\n  zam --version", env!("CARGO_PKG_VERSION")),
        _ => return Err("usage: zam <check|build|run|fmt|new> [project|file]".into()),
    }
    Ok(())
}

/// 生成一个最小可运行的项目骨架。目标目录已存在就拒绝，不覆盖任何用户文件。
fn new_project(directory: &Path, ai_template: bool) -> project::Result<()> {
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("project name must be UTF-8")?;
    if !project::identifier(name) {
        return Err("project name must be an identifier".into());
    }
    if directory.exists() {
        return Err(format!("{}: already exists", directory.display()));
    }
    let mut files = vec![
        (
            directory.join("zam.toml"),
            format!("[project]\nname = \"{name}\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n"),
        ),
        (
            directory.join("src/main.zm"),
            "fn main() {\n    println(\"hello from zamak\")\n}\n".to_string(),
        ),
        (directory.join(".gitignore"), "build/\n".to_string()),
    ];
    if ai_template {
        files.push((
            directory.join("AI.md"),
            "# AI 工作循环\n\n1. 运行 `zam check --json <project>`。\n2. 根据 `file`、`line`、`column`、`function`、`code`、`category` 和 `suggestion` 定位最小修复。\n3. 修改后运行 `zam fmt <project>`，再重新检查。\n4. 通过后运行 `zam build <project>` 和 `zam run <project>`。\n\n保持每次修改小而明确。当前版本暂不支持浮点数、文件/网络 I/O、泛型、枚举、闭包、线程和 FFI。\n".to_string(),
        ));
        files.push((
            directory.join("README.md"),
            "# Zamak 项目\n\n```powershell\nzam check --json .\nzam fmt .\nzam build .\nzam run .\n```\n\n错误修复优先查看结构化诊断；不要猜测未支持的语言特性。\n".to_string(),
        ));
    }
    for (path, text) in files {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        eprintln!("NEW {}", path.display());
    }
    Ok(())
}

/// 就地重排缩进；只写真正变了的文件，改动过的路径打到 stderr（与构建进度一致）。
fn format_sources(input: &Path) -> project::Result<()> {
    // 先解析一遍：语法就不成立的代码不格式化，免得把错误改得更难看。
    let project = project::Project::load(input)?;
    for path in project.module_paths() {
        let source =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let formatted =
            compiler::format(&source).map_err(|e| format!("{}: {e}", path.display()))?;
        if formatted != source {
            std::fs::write(&path, &formatted).map_err(|e| format!("{}: {e}", path.display()))?;
            eprintln!("FORMAT {}", path.display());
        }
    }
    Ok(())
}

///把一次命令的结果渲染成一行 JSON，供编辑器等工具读取。
fn report(command: &str, entry: &str, outcome: &project::Result<()>) -> String {
    let items = match outcome {
        Ok(()) => String::new(),
        Err(error) => item(&diagnostic(entry, error)),
    };
    format!(
        "{{\"command\":\"{command}\",\"ok\":{},\"diagnostics\":[{items}]}}",
        outcome.is_ok()
    )
}

struct Diagnostic {
    file: String,
    line: Option<usize>,
    column: Option<usize>,
    function: Option<String>,
    code: &'static str,
    category: &'static str,
    message: String,
    suggestion: Option<&'static str>,
}

/// ponytail: 诊断目前是拼好的文本（`project::Result<T> = Result<T, String>`），这里按既有格式
/// 反解成字段；等更多消费者（LSP、CI 注解）出现时，再改成贯穿编译器的结构化 Diagnostic。
fn diagnostic(entry: &str, error: &str) -> Diagnostic {
    let (file, rest) = match error.split_once(": line ") {
        Some((head, tail)) => (head.to_string(), format!("line {tail}")),
        None => (entry.to_string(), error.to_string()),
    };
    let (position, rest) = match rest.split_once(": ") {
        Some((head, tail)) if head.starts_with("line ") => (head.to_string(), tail.to_string()),
        _ => (String::new(), rest),
    };
    let (line, column) = match position.split_once(", ") {
        Some((line, column)) => (
            line.strip_prefix("line ")
                .and_then(|n| n.trim().parse().ok()),
            column
                .strip_prefix("column ")
                .and_then(|n| n.trim().parse().ok()),
        ),
        None => (None, None),
    };
    let (function, message) = match rest.split_once(": ") {
        Some((head, tail)) => {
            let mut parts = head.split(':');
            match (parts.next(), parts.next(), parts.next()) {
                (Some(module), Some(name), None)
                    if identifier(module, true) && identifier(name, false) =>
                {
                    (Some(format!("{module}:{name}")), tail.to_string())
                }
                _ => (None, rest.clone()),
            }
        }
        None => (None, rest.clone()),
    };
    let (code, category, suggestion) = classify(&message);
    Diagnostic {
        file,
        line,
        column,
        function,
        code,
        category,
        message,
        suggestion,
    }
}

fn classify(message: &str) -> (&'static str, &'static str, Option<&'static str>) {
    if message.contains("unexpected character") {
        (
            "ZM100",
            "syntax",
            Some("remove or replace the unsupported character"),
        )
    } else if message.contains("unsupported string escape") {
        (
            "ZM101",
            "syntax",
            Some("use a supported escape such as `\\n`, `\\r`, `\\t`, `\\\"`, or `\\\\`"),
        )
    } else if message.contains("use after move") {
        (
            "ZM301",
            "ownership",
            Some("borrow the value or move it only after the last use"),
        )
    } else if message.contains("borrow escape") {
        (
            "ZM422",
            "ownership",
            Some("keep references in call arguments; store or return an owned value"),
        )
    } else if message.contains("borrow type mismatch") {
        (
            "ZM423",
            "ownership",
            Some("match the borrow type to the parameter declaration"),
        )
    } else if message.contains("cannot mutably borrow immutable") {
        (
            "ZM424",
            "ownership",
            Some("declare the owner with `let mut` before taking `&mut`"),
        )
    } else if message.contains("expected explicit borrow") {
        (
            "ZM425",
            "ownership",
            Some("pass `&value` for a shared borrow or `&mut value` for an exclusive borrow"),
        )
    } else if message.contains("borrow") || message.contains("borrowed") {
        (
            "ZM421",
            "ownership",
            Some("avoid overlapping moves or exclusive borrows in the same call"),
        )
    } else if message.contains("missing string return value") {
        (
            "ZM510",
            "control_flow",
            Some("return a string on every path or change the function result type"),
        )
    } else if message.contains("fallible call") || message.contains("propagate the error") {
        (
            "ZM500",
            "error_handling",
            Some("handle the call with `?` or a `catch` block"),
        )
    } else if message.contains("only string error types") {
        (
            "ZM509",
            "error_handling",
            Some("use `!string` or `T!string` for the error type"),
        )
    } else if message.contains("literal overflow") {
        (
            "ZM600",
            "arithmetic",
            Some("use a literal that fits in `i64` (max 9223372036854775807)"),
        )
    } else if message.contains("overflow") || message.contains("division by zero") {
        (
            "ZM600",
            "arithmetic",
            Some("check the input range before arithmetic or division"),
        )
    } else if message.contains("array index") {
        (
            "ZM406",
            "bounds",
            Some("ensure the index is within the declared array length"),
        )
    } else if message.contains("type mismatch") {
        (
            "ZM200",
            "type",
            Some("match the expression type to the declared type"),
        )
    } else if message.contains("unknown field ") {
        (
            "ZM202",
            "name",
            Some("check the field name against the struct declaration"),
        )
    } else if message.contains("unknown variable ")
        || message.contains("unknown function ")
        || message.contains("unknown struct")
        || message.contains("unknown type name")
    {
        (
            "ZM201",
            "name",
            Some("check the name, declaration, and module import"),
        )
    } else if message.starts_with("cannot assign immutable value ") {
        (
            "ZM302",
            "ownership",
            Some("declare the binding with `let mut` before assigning it"),
        )
    } else if message.contains("unsupported") {
        (
            "ZM900",
            "unsupported",
            Some("use a construct listed as supported in `zam --help` or the tutorial"),
        )
    } else if message.starts_with("expected") || message.contains("unterminated") {
        (
            "ZM110",
            "syntax",
            Some("repair the reported token or delimiter, then run `zam check` again"),
        )
    } else {
        ("ZM999", "general", None)
    }
}

/// 诊断里的 `<模块>:<函数>` 前缀只用标识符字符，磁盘路径与普通句子不会误判。
fn identifier(text: &str, slash: bool) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || (slash && c == '/'))
}

fn item(diagnostic: &Diagnostic) -> String {
    format!(
        "{{\"file\":{},\"line\":{},\"column\":{},\"function\":{},\"code\":{},\"category\":{},\"message\":{},\"suggestion\":{}}}",
        text(Some(&diagnostic.file)),
        number(diagnostic.line),
        number(diagnostic.column),
        text(diagnostic.function.as_deref()),
        text(Some(diagnostic.code)),
        text(Some(diagnostic.category)),
        text(Some(&diagnostic.message)),
        text(diagnostic.suggestion),
    )
}

fn number(value: Option<usize>) -> String {
    value.map_or_else(|| "null".to_string(), |value| value.to_string())
}

fn text(value: Option<&str>) -> String {
    value.map_or_else(
        || "null".to_string(),
        |value| format!("\"{}\"", escape(value)),
    )
}

fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn demo() -> project::Result<()> {
    let root = std::env::temp_dir().join(format!(
        "zam-demo-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let cache = root.join("cache");
    let result = (|| {
        for directory in ["a", "b"] {
            let dir = root.join(directory);
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            std::fs::write(
                dir.join("hello.zm"),
                "fn main() { println(\"hello from zamak\") }",
            )
            .map_err(|e| e.to_string())?;
            project::Project::load(&dir.join("hello.zm"))?.build(&cache)?;
        }
        let count = std::fs::read_dir(&cache)
            .map_err(|e| e.to_string())?
            .count();
        if count != 3 {
            return Err("path-independent caching failed".into());
        }
        println!("path-independent content-addressed cache verified");
        Ok(())
    })();
    std::fs::remove_dir_all(&root).map_err(|e| e.to_string())?;
    result
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("zam: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_categories_distinguish_repair_actions() {
        for (message, code) in [
            (
                "borrow escape: references cannot be stored in locals",
                "ZM422",
            ),
            ("borrow type mismatch for text", "ZM423"),
            ("cannot mutably borrow immutable value text", "ZM424"),
            ("expected explicit borrow of text", "ZM425"),
            ("borrow conflict: text", "ZM421"),
            ("string type mismatch", "ZM200"),
            ("unknown variable text", "ZM201"),
            ("entry: unknown function text", "ZM201"),
            ("unknown struct Point", "ZM201"),
            ("unknown type name Thing", "ZM201"),
            ("unknown field z", "ZM202"),
            ("cannot assign immutable value count", "ZM302"),
            ("unsupported type f64", "ZM900"),
            ("i64 literal overflow", "ZM600"),
        ] {
            let d = diagnostic("entry.zm", message);
            assert_eq!(d.code, code, "{message}");
            assert!(d.suggestion.is_some());
            assert_eq!(d.message, message);
        }
    }

    #[test]
    fn literal_overflow_suggests_a_smaller_literal() {
        let d = diagnostic("entry.zm", "i64 literal overflow");
        assert_eq!(d.code, "ZM600");
        assert_eq!(d.category, "arithmetic");
        assert_eq!(
            d.suggestion,
            Some("use a literal that fits in `i64` (max 9223372036854775807)")
        );
    }

    fn parse(entry: &str, error: &str) -> Diagnostic {
        diagnostic(entry, error)
    }

    #[test]
    fn diagnostics_parse_every_shape() {
        let with_file = parse(
            "entry.zm",
            r"C:\a\main.zm: line 3, column 9: unexpected character",
        );
        assert_eq!(with_file.file, r"C:\a\main.zm");
        assert_eq!((with_file.line, with_file.column), (Some(3), Some(9)));
        assert_eq!(with_file.function, None);
        assert_eq!(with_file.message, "unexpected character");
        assert_eq!(with_file.code, "ZM100");
        assert_eq!(with_file.category, "syntax");

        // 语义诊断由 project::check 传 <模块>:main 名字，消息里还可能出现别的冒号。
        let semantic = parse(
            "entry.zm",
            "line 6, column 5: main:main: fallible call main:f must be handled with ?",
        );
        assert_eq!(semantic.file, "entry.zm");
        assert_eq!(semantic.function.as_deref(), Some("main:main"));
        assert_eq!(
            semantic.message,
            "fallible call main:f must be handled with ?"
        );

        // 与位置无关的错误保留整段原文，位置为空。
        let plain = parse("entry.zm", "missing entry module main");
        assert_eq!(
            (plain.line, plain.column, plain.function),
            (None, None, None)
        );
        assert_eq!(plain.message, "missing entry module main");

        // 路径里的冒号不能当成 <模块>:<函数> 前缀。
        let config = parse("entry.zm", r"C:\p\zam.toml: missing string project.name");
        assert_eq!(config.file, "entry.zm");
        assert_eq!(config.function, None);
        assert_eq!(
            config.message,
            r"C:\p\zam.toml: missing string project.name"
        );
    }

    #[test]
    fn report_is_one_line_json() {
        let ok: project::Result<()> = Ok(());
        assert_eq!(
            report("build", "entry.zm", &ok),
            r#"{"command":"build","ok":true,"diagnostics":[]}"#
        );

        let failed: project::Result<()> =
            Err(r#"C:\a\main.zm: line 1, column 2: bad "quote""#.into());
        let line = report("check", "entry.zm", &failed);
        assert_eq!(line.lines().count(), 1);
        assert_eq!(
            line,
            r#"{"command":"check","ok":false,"diagnostics":[{"file":"C:\\a\\main.zm","line":1,"column":2,"function":null,"code":"ZM999","category":"general","message":"bad \"quote\"","suggestion":null}]}"#
        );
    }
}
