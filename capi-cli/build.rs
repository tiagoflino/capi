fn main() {
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/lib");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib/capi/openvino");
}
