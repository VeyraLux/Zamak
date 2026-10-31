//! `zam build --noconsole --icon app.ico` 与 zam.toml 里的 `[app]`：程序不带控制台窗口、带自己的图标。
//! 这两项只在 Windows 上生效；别的系统要接受同样的设置、给出提示并照常构建，同一份 zam.toml 才能跨平台用。
mod common;

use common::Case;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// 一个 32x32、32 位色的 .ico：橙色底、中间一个深蓝方块。
fn icon() -> Vec<u8> {
    let size = 32usize;
    let mut pixels = Vec::new();
    for y in (0..size).rev() {
        for x in 0..size {
            let inside = (8..24).contains(&x) && (8..24).contains(&y);
            pixels.extend(if inside {
                [110, 40, 20, 255]
            } else {
                [40, 120, 250, 255]
            });
        }
    }
    let mask = vec![0u8; 4 * size];
    let mut image = Vec::new();
    for field in [40u32, size as u32, size as u32 * 2] {
        image.extend(field.to_le_bytes());
    }
    image.extend(1u16.to_le_bytes());
    image.extend(32u16.to_le_bytes());
    image.extend(0u32.to_le_bytes());
    image.extend(((pixels.len() + mask.len()) as u32).to_le_bytes());
    image.extend([0u8; 16]);
    image.extend(&pixels);
    image.extend(&mask);
    let mut ico = vec![0, 0, 1, 0, 1, 0, size as u8, size as u8, 0, 0, 1, 0, 32, 0];
    ico.extend((image.len() as u32).to_le_bytes());
    ico.extend(22u32.to_le_bytes());
    ico.extend(image);
    ico
}

/// 程序自己的 PE 头里写的子系统：2 是窗口程序（没有控制台），3 是控制台程序。
fn subsystem(exe: &[u8]) -> u16 {
    let pe = u32::from_le_bytes(exe[0x3C..0x40].try_into().unwrap()) as usize;
    u16::from_le_bytes(exe[pe + 24 + 68..pe + 24 + 70].try_into().unwrap())
}

fn built(output: &Output) -> PathBuf {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let line = stderr
        .lines()
        .find_map(|line| line.strip_prefix("OUTPUT "))
        .unwrap_or_else(|| panic!("no OUTPUT line in {stderr}"));
    PathBuf::from(line.trim())
}

fn zam(case: &Case, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zam"))
        .args(args)
        .current_dir(case.path("."))
        .env("ZAMAK_CACHE_DIR", case.path("cache"))
        .env("ZAMAK_LANG", "en")
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn hello(case: &Case) {
    fs::write(
        case.path("hello.zm"),
        "use std/fs\n\nfn main() -> !string:\n    println(\"hi\")\n    fs.write(\"ran.txt\", \"ok\")?\n",
    )
    .unwrap();
    fs::write(case.path("app.ico"), icon()).unwrap();
}

#[test]
fn noconsole_and_icon_build_a_window_program() {
    let case = Case::new("app", "flags");
    hello(&case);
    let output = zam(
        &case,
        &["build", "hello.zm", "--noconsole", "--icon", "app.ico"],
    );
    let exe = built(&output);
    if !cfg!(windows) {
        // 别的系统没有控制台窗口，也没有 exe 图标：照常构建，并说明这两项被忽略了。
        assert!(
            stderr(&output).contains("only apply on Windows"),
            "{}",
            stderr(&output)
        );
        return;
    }
    let bytes = fs::read(case.path(".").join(&exe)).unwrap();
    assert_eq!(subsystem(&bytes), 2);
    // 图标作为资源原样嵌在程序里。
    let ico = icon();
    let image = &ico[22..22 + 200];
    assert!(
        bytes.windows(image.len()).any(|window| window == image),
        "the icon is not in the program"
    );
    // 不加参数时仍是控制台程序，也没有这个图标。
    let plain = built(&zam(&case, &["build", "hello.zm"]));
    let plain = fs::read(case.path(".").join(plain)).unwrap();
    assert_eq!(subsystem(&plain), 3);
    assert!(!plain.windows(image.len()).any(|window| window == image));
}

#[test]
fn a_window_program_without_standard_output_still_runs() {
    // 双击启动的窗口程序没有标准输出：`println` 不能让它悄悄退出。`start /wait` 不传标准句柄。
    if !cfg!(windows) {
        return;
    }
    let case = Case::new("app", "nohandles");
    hello(&case);
    let exe = built(&zam(&case, &["build", "hello.zm", "--noconsole"]));
    let exe = case.path(".").join(exe);
    let status = Command::new("cmd")
        .args(["/C", "start", "", "/wait"])
        .arg(&exe)
        .current_dir(case.path("."))
        .status()
        .unwrap();
    assert!(status.success(), "{status}");
    assert_eq!(fs::read_to_string(case.path("ran.txt")).unwrap(), "ok");
}

#[test]
fn app_settings_come_from_zam_toml() {
    let case = Case::new("app", "manifest");
    fs::create_dir_all(case.path("src")).unwrap();
    fs::create_dir_all(case.path("assets")).unwrap();
    fs::write(case.path("assets/app.ico"), icon()).unwrap();
    fs::write(
        case.path("src/main.zm"),
        "fn main():\n    println(\"hi\")\n",
    )
    .unwrap();
    let manifest = |app: &str| {
        fs::write(
            case.path("zam.toml"),
            format!("[project]\nname = \"demo\"\nentry = \"src/main.zm\"\nmodules = \"src\"\n\n[app]\n{app}"),
        )
        .unwrap();
    };
    manifest("console = false\nicon = \"assets/app.ico\"\n");
    let exe = built(&zam(&case, &["build", "."]));
    if cfg!(windows) {
        let bytes = fs::read(case.path(".").join(exe)).unwrap();
        assert_eq!(subsystem(&bytes), 2);
    }
    // 写错的设置直接说清楚，不猜。
    for (app, message) in [
        (
            "colour = 1\n",
            "unknown app setting colour; [app] accepts console and icon",
        ),
        ("console = \"no\"\n", "app.console must be true or false"),
        ("icon = 5\n", "app.icon must be a string"),
        (
            "icon = \"../outside.ico\"\n",
            "project paths must be nonempty relative paths",
        ),
    ] {
        manifest(app);
        let output = zam(&case, &["build", "."]);
        assert!(!output.status.success(), "accepted {app}");
        assert!(
            stderr(&output).contains(message),
            "{app}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn bad_icons_and_flags_are_rejected_clearly() {
    let case = Case::new("app", "rejected");
    hello(&case);
    fs::write(
        case.path("fake.ico"),
        b"\x89PNG\r\n\x1a\n and some more bytes to be long enough",
    )
    .unwrap();
    let flags = |args: &[&str]| stderr(&zam(&case, args));
    // 不是 .ico 的文件、不存在的文件（只有 Windows 会去读图标）。
    if cfg!(windows) {
        assert!(flags(&["build", "hello.zm", "--icon", "fake.ico"])
            .contains("must be a Windows .ico file"),);
        assert!(
            flags(&["build", "hello.zm", "--icon", "missing.ico"]).contains("cannot read the icon")
        );
    }
    assert!(
        flags(&["build", "hello.zm", "--icon"]).contains("--icon needs the path of a .ico file")
    );
    for command in ["check", "test", "fmt"] {
        assert!(
            flags(&[command, "hello.zm", "--noconsole"])
                .contains("--noconsole and --icon work with zam build and zam run"),
            "{command}"
        );
    }
}
