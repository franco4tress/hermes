use std::{env, fs, path::PathBuf};

fn main() {
    // Copy memory.x to OUT_DIR so the linker can find it via cortex-m-rt's link.x
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::copy("memory.x", out.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
}
