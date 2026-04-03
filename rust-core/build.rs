use std::env;
use std::path::PathBuf;

fn main() {
    // Only generate C bindings when explicitly requested via env var
    // This avoids cbindgen parse errors during normal builds
    if env::var("GENERATE_C_BINDINGS").is_ok() {
        let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let config_path = PathBuf::from(&crate_dir).join("cbindgen.toml");

        cbindgen::Builder::new()
            .with_crate(&crate_dir)
            .with_config(cbindgen::Config::from_file(&config_path).unwrap_or_default())
            .with_language(cbindgen::Language::C)
            .generate()
            .expect("Unable to generate C bindings")
            .write_to_file(&format!("{}/include/claude_core.h", crate_dir));
    }
}
