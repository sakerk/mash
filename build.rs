fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string());

        let mut res = winres::WindowsResource::new();
        if std::path::Path::new("assets/icon.ico").exists() {
            res.set_icon("assets/icon.ico");
        }
        res.set("ProductName", "MASH");
        res.set("FileDescription", "MASH - Media Asset Hash");
        res.set("ProductVersion", &version);
        res.set("FileVersion", &version);
        res.set("LegalCopyright", "Copyright 2026 Saker Klippsten");
        res.set("CompanyName", "Saker Klippsten");
        res.set("OriginalFilename", "mash.exe");
        res.set("InternalName", "mash");
        res.compile().expect("Failed to compile Windows resources");
    }
}
