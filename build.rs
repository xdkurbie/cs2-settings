fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icons/icon.ico");
        res.set("ProductName", "CS2 Settings");
        res.set("FileDescription", "CS2 Settings — config menu & export");
        res.set("LegalCopyright", "MIT");
        if let Err(e) = res.compile() {
            println!("cargo:warning=winresource failed: {e}");
        }
    }
}
