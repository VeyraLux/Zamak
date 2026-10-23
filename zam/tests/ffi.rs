use std::{fs, path::Path, process::Command};

fn zam(args: &[&str], cache: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(args)
        .env("ZAMAK_CACHE_DIR", cache)
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap()
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

const MANIFEST: &str = "[project]\nname = \"app\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n";

#[test]
fn extern_functions_call_c_sources_and_libraries() {
    let root = std::env::temp_dir().join(format!("zam-ffi-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let cache = root.join("cache");

    // 示例项目：C 源码与 C 标准库函数，第二次构建全部命中缓存。
    let example = root.join("example");
    fs::create_dir_all(&example).unwrap();
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/ffi"),
        &example,
    );
    let output = zam(&["run", example.to_str().unwrap()], &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), "2\nhello, zamak\n6.0\ntrue\n4.0\n");
    assert!(stderr(&output).contains("[c] called from zamak"));
    assert!(stderr(&output).contains("OBJECT MISS"));
    let again = zam(&["build", example.to_str().unwrap()], &cache);
    assert!(!stderr(&again).contains("MISS"), "{}", stderr(&again));

    // 头文件改了，用到它的 C 源码要重新编译。
    let header = example.join("c/textutil.h");
    let text = fs::read_to_string(&header).unwrap();
    fs::write(&header, text.replace("#endif", "/* changed */\n#endif")).unwrap();
    let rebuilt = zam(&["build", example.to_str().unwrap()], &cache);
    assert!(rebuilt.status.success(), "{}", stderr(&rebuilt));
    assert!(stderr(&rebuilt).contains("c/textutil.c"));
    assert!(stderr(&rebuilt)
        .lines()
        .any(|l| l.starts_with("OBJECT MISS") && l.contains("textutil")));

    // 依赖包自带 C 代码：pub extern fn 跨模块调用、当函数值传；声明了但没用的 extern fn
    // （C 里没有这个函数）不影响链接。
    let package = root.join("cmath");
    write(
        &package.join("zam.toml"),
        "[project]\nname = \"cmath\"\nmodules = \"src\"\n\n[native]\nsources = [\"native/twice.c\"]\n",
    );
    write(
        &package.join("native/twice.c"),
        "#include <stdint.h>\nint64_t cmath_twice(int64_t x) { return 2 * x; }\n",
    );
    write(
        &package.join("src/cmath.zm"),
        "pub extern fn cmath_twice(x: i64) -> i64\npub extern fn cmath_missing(x: i64) -> i64\n\npub fn quadruple(x: i64) -> i64: cmath_twice(cmath_twice(x))\n",
    );
    let app = root.join("app");
    write(
        &app.join("zam.toml"),
        &format!("{MANIFEST}\n[dependencies]\ncmath = {{ path = \"../cmath\" }}\n"),
    );
    write(
        &app.join("src/main.zm"),
        "use cmath\n\nfn apply(f: fn(i64) -> i64, x: i64) -> i64: f(x)\n\nfn main():\n    println(cmath.quadruple(3))\n    println(apply(cmath.cmath_twice, 21))\n",
    );
    let output = zam(&["run", app.to_str().unwrap()], &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), "12\n42\n");

    // libraries：链接系统里现成的库。
    let linked = root.join("linked");
    let (library, source) = if cfg!(windows) {
        (
            "kernel32",
            "extern fn sqrt(x: f64) -> f64\n\nfn main(): println(sqrt(81.0))\n",
        )
    } else {
        (
            "m",
            "extern fn sqrt(x: f64) -> f64\n\nfn main(): println(sqrt(81.0))\n",
        )
    };
    write(
        &linked.join("zam.toml"),
        &format!("{MANIFEST}\n[native]\nlibraries = [\"{library}\"]\n"),
    );
    write(&linked.join("src/main.zm"), source);
    let output = zam(&["run", linked.to_str().unwrap()], &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), "9.0\n");

    // 写错的声明在检查阶段报错，不需要 C 编译器。
    for (source, message) in [
        (
            "extern fn f(xs: &[i64]) -> i64\n\nfn main(): pass\n",
            "extern functions take i64, f64, bool, &string",
        ),
        (
            "extern fn f() -> i64!string\n\nfn main(): pass\n",
            "cannot declare an error type",
        ),
        (
            "extern fn f() -> i64: 1\n\nfn main(): pass\n",
            "extern fn f has no body",
        ),
    ] {
        let file = root.join("bad.zm");
        fs::write(&file, source).unwrap();
        let output = zam(&["check", file.to_str().unwrap()], &cache);
        assert!(!output.status.success(), "accepted {source}");
        assert!(
            stderr(&output).contains(message),
            "{source}: {}",
            stderr(&output)
        );
    }
    write(
        &linked.join("zam.toml"),
        &format!("{MANIFEST}\n[native]\nflags = [\"-O3\"]\n"),
    );
    let output = zam(&["check", linked.to_str().unwrap()], &cache);
    assert!(stderr(&output).contains("unknown native setting flags"));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn c_handles_are_closed_exactly_once() {
    let root = std::env::temp_dir().join(format!("zam-ffi-handles-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let cache = root.join("cache");

    // 示例：带头文件的 C 库，句柄自动释放、NULL 变 None、输出参数、bytes 缓冲区。
    let example = root.join("example");
    fs::create_dir_all(&example).unwrap();
    copy_dir(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/ffi_handles"),
        &example,
    );
    let output = zam(&["run", example.to_str().unwrap()], &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), "7\n-1\n1\n0\nbeta\n1\ntrue\n2\n532\n");

    // 没有头文件时按 Zamak 的签名写原型（句柄是 void *）。句柄放进列表、结构体字段，移动之后
    // 只有新的所有者释放它。
    let plain = root.join("plain");
    write(
        &plain.join("zam.toml"),
        &format!("{MANIFEST}\n[native]\nsources = [\"c/box.c\"]\n"),
    );
    write(
        &plain.join("c/box.c"),
        "#include <stdint.h>\n#include <stdlib.h>\nstatic int64_t freed;\nvoid *box_new(int64_t v) { int64_t *p = (int64_t *)malloc(sizeof *p); *p = v; return p; }\nint64_t box_get(void *p) { return *(int64_t *)p; }\nvoid box_free(void *p) { freed++; free(p); }\nint64_t box_freed(void) { return freed; }\n",
    );
    write(
        &plain.join("src/main.zm"),
        "extern type Box: \"int64_t\", close = box_free\nextern fn box_new(v: i64) -> Option[Box]\nextern fn box_get(b: &Box) -> i64\nextern fn box_freed() -> i64\n\nstruct Holder:\n    label: string\n    item: Option[Box]\n\nfn fill() -> i64:\n    let mut boxes: [Option[Box]] = []\n    for i in range(3):\n        boxes.push(box_new(i * 10))\n    let mut total = 0\n    for b in boxes:\n        match b:\n            Some(x): total += box_get(x)\n            None: pass\n    total\n\nfn keep() -> Holder:\n    let first = box_new(7)\n    let moved = first\n    Holder(label: \"kept\", item: moved)\n\nfn main():\n    println(fill())\n    println(box_freed())\n    let h = keep()\n    println(box_freed())\n    match h.item:\n        Some(x): println(box_get(x))\n        None: println(-1)\n    println(h.item)\n",
    );
    let output = zam(&["run", plain.to_str().unwrap()], &cache);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        stdout(&output),
        "30\n3\n3\n7\nOption.Some(Box(<C handle>))\n"
    );

    // 写错的地方在检查阶段报错。
    for (source, message) in [
        (
            "extern type Box: \"int64_t\", close = box_free\nextern fn box_new(v: i64) -> Option[Box]\n\nfn main():\n    let b = box_new(1)\n    let c = copy(&b)\n",
            "a C handle cannot be copied",
        ),
        (
            "extern type Box: \"int64_t\"\n\nfn main():\n    let b = Box()\n",
            "Box is a C handle; get one from an extern fn that returns Option[Box]",
        ),
        (
            "extern type Box: \"int64_t\"\nextern fn box_new(v: i64) -> Box\n\nfn main(): pass\n",
            "a C handle is returned as Option[Db]",
        ),
        (
            "struct Point:\n    x: i64\n\nextern fn draw(p: &Point)\n\nfn main(): pass\n",
            "a struct reaches C only as a C handle",
        ),
        (
            "extern type box: \"int64_t\"\n\nfn main(): pass\n",
            "C handle type names start with an uppercase letter",
        ),
        (
            "extern type Box: int64_t\n\nfn main(): pass\n",
            "expected the C type in quotes",
        ),
    ] {
        let file = root.join("bad.zm");
        fs::write(&file, source).unwrap();
        let output = zam(&["check", file.to_str().unwrap()], &cache);
        assert!(!output.status.success(), "accepted {source}");
        assert!(
            stderr(&output).contains(message),
            "{source}: {}",
            stderr(&output)
        );
    }
    write(
        &plain.join("zam.toml"),
        &format!("{MANIFEST}\n[native]\nheaders = [\"<box.h>\"]\n"),
    );
    let output = zam(&["check", plain.to_str().unwrap()], &cache);
    assert!(
        stderr(&output).contains("invalid header name"),
        "{}",
        stderr(&output)
    );

    fs::remove_dir_all(&root).unwrap();
}
