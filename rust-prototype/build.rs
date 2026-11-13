fn main() {
    println!("cargo:rerun-if-changed=src/music_macos.m");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
        && std::env::var_os("CARGO_FEATURE_RENDERER").is_some()
    {
        cc::Build::new()
            .file("src/music_macos.m")
            .flag("-fobjc-arc")
            .compile("firered_music");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=AVFAudio");
    }
}
