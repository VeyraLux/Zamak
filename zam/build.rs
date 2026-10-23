//! 构建时做两件事：
//! 1. Windows 上给 zam.exe 写入文件信息（产品名、版本、图标）。资源管理器的“属性 → 详细信息”里能看到
//!    产品名与版本号，以后申请代码签名时也需要它们。
//! 2. 把 vendor/mbedtls 的源码打成一个压缩包嵌进 zam：程序用到 TLS 时，zam 解压到缓存再编译。
use std::path::Path;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let version = env!("CARGO_PKG_VERSION");
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("../ico/Zamak_256x256.ico")
            .set("ProductName", "Zamak")
            .set("FileDescription", "Zamak compiler")
            .set("CompanyName", "VeyraLux")
            .set(
                "LegalCopyright",
                "Copyright (c) 2026 VeyraLux. MIT OR Apache-2.0.",
            )
            .set("OriginalFilename", "zam.exe")
            .set("ProductVersion", version)
            .set("FileVersion", version);
        resource
            .compile()
            .expect("embed the Windows version resource");
    }
    vendor_archive();
}

/// 打包格式：每个文件依次是 路径（UTF-8）、`\0`、长度（4 字节小端）、内容；整体用 deflate 压缩。
/// 文件按路径排序，换行统一成 `\n`，所以同样的源码在任何平台上得到同样的包。
fn vendor_archive() {
    let root = Path::new("vendor/mbedtls");
    println!("cargo:rerun-if-changed=vendor/mbedtls");
    let mut files = Vec::new();
    collect(root, root, &mut files);
    files.sort();
    let mut archive = Vec::new();
    for (name, path) in files {
        let text = std::fs::read(&path).expect("read a vendored file");
        let text: Vec<u8> = text.into_iter().filter(|b| *b != b'\r').collect();
        archive.extend_from_slice(name.as_bytes());
        archive.push(0);
        archive.extend_from_slice(&(text.len() as u32).to_le_bytes());
        archive.extend_from_slice(&text);
    }
    let packed = miniz_oxide::deflate::compress_to_vec(&archive, 9);
    let out = std::env::var("OUT_DIR").expect("OUT_DIR");
    std::fs::write(Path::new(&out).join("mbedtls.bin"), packed).expect("write the vendor archive");
}

fn collect(root: &Path, dir: &Path, files: &mut Vec<(String, std::path::PathBuf)>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("read vendor/mbedtls")
        .map(|entry| entry.expect("vendor entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(root, &path, files);
        } else if path.extension().is_some_and(|ext| ext == "c" || ext == "h") {
            let name = path
                .strip_prefix(root)
                .expect("inside vendor")
                .to_string_lossy()
                .replace('\\', "/");
            files.push((name, path));
        }
    }
}
