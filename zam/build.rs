//! Windows 上给 zam.exe 写入文件信息（产品名、版本、图标）。资源管理器的“属性 → 详细信息”里能看到
//! 产品名与版本号，以后申请代码签名时也需要它们。其他平台什么也不做。
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
}
