use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "linux" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/capi/openvino");
    }
}
