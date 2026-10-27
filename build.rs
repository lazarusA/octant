fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/brand/ico/octant.ico");
    // The .exe's own icon (Explorer, Start menu, installers' shortcuts); main.rs sets the window's.
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/brand/ico/octant.ico");
        if let Err(err) = res.compile() {
            println!("cargo:warning=Windows icon resource not embedded: {err}");
        }
    }
}
