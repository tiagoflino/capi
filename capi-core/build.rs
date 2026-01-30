use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/cpp/bridge.h");
    println!("cargo:rerun-if-changed=src/cpp/bridge.cpp");
    println!("cargo:rerun-if-changed=src/genai_bridge.rs");

    let openvino_root = env::var("OPENVINO_ROOT").ok();

    // Link directories
    if let Some(root) = &openvino_root {
        println!("cargo:rustc-link-search=native={}/runtime/lib/intel64", root);
        println!("cargo:rustc-link-search=native={}/runtime/3rdparty/tbb/lib", root);
    } else {
        println!("cargo:rustc-link-search=native=/usr/lib");
    }

    // Link against C++ library
    println!("cargo:rustc-link-lib=openvino_genai");
    println!("cargo:rustc-link-lib=openvino");

    // Build C++ bridge with cxx
    let mut build = cxx_build::bridge("src/genai_bridge.rs");
    build.file("src/cpp/bridge.cpp");
    build.std("c++17");
    
    if let Some(root) = &openvino_root {
        build.include(format!("{}/runtime/include", root));
    }
    
    build.compile("genai_bridge");
}
