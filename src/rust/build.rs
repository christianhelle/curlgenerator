fn main() {
    println!("cargo:rerun-if-changed=images/icon.ico");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Only PE executables carry an icon resource; check the target, not the host, so cross-compiles work.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("images/icon.ico")
            .compile()
            .expect("failed to embed Windows resources");
    }
}
