use rustc_version::{version, Version};

fn main() {
    if version().unwrap() < Version::parse("1.94.0-nightly").unwrap() {
        println!("cargo:rustc-cfg=nightly_old");
    }
    println!("cargo:rustc-check-cfg=cfg(nightly_old)")
}
