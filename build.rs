fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("assets/icon.ico");
    resource.set(
        "FileDescription",
        "FolioFold - PDF imposition for bookbinders",
    );
    resource.set("ProductName", "FolioFold");
    resource.set("OriginalFilename", "folio-fold.exe");
    resource.set(
        "LegalCopyright",
        "Released into the public domain under the Unlicense",
    );
    resource
        .compile()
        .expect("compile the Windows icon resource");
}
