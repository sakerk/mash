fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        if std::path::Path::new("assets/icon.ico").exists() {
            res.set_icon("assets/icon.ico");
        }
        res.set("ProductName", "MASH");
        res.set(
            "FileDescription",
            "MASH - Media Asset Hash",
        );
        let _ = res.compile();
    }
}
