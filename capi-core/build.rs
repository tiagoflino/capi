use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../openvino.conf");
    println!("cargo:rerun-if-changed=src/cpp/bridge.h");
    println!("cargo:rerun-if-changed=src/cpp/bridge.cpp");
    println!("cargo:rerun-if-changed=src/genai_bridge.rs");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap();
    let conf_path = workspace_root.join("openvino.conf");
    if let Ok(content) = std::fs::read_to_string(&conf_path) {
        let mut vars: HashMap<String, String> = HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((key, value)) = line.split_once('=') {
                vars.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        if let (Some(version), Some(short)) = (vars.get("OPENVINO_VERSION"), vars.get("OPENVINO_SHORT")) {
            let parts: Vec<&str> = short.split('.').collect();
            let soname = format!("{}{}{}", &parts[0][parts[0].len()-2..], parts[1], parts[2]);
            println!("cargo:rustc-env=OV_VERSION={}", version);
            println!("cargo:rustc-env=OV_SHORT={}", short);
            println!("cargo:rustc-env=OV_SONAME={}", soname);
        }
    }

    let openvino_root: Option<PathBuf> = env::var("OPENVINO_ROOT").ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
            let local_libs = manifest_dir.parent().unwrap().join("libs/openvino");
            if local_libs.exists() { Some(local_libs) } else { None }
        })
        .or_else(|| {
            let opt_ov = PathBuf::from("/opt/openvino");
            if opt_ov.exists() { Some(opt_ov) } else { None }
        })
        .map(|root| {
            if root.is_absolute() {
                root
            } else {
                let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
                manifest_dir.parent().unwrap().join(root)
            }
        });

    if let Some(root) = &openvino_root {
        println!("cargo:rustc-link-search=native={}/runtime/lib/intel64", root.display());
        println!("cargo:rustc-link-search=native={}/runtime/3rdparty/tbb/lib", root.display());
    } else {
        println!("cargo:rustc-link-search=native=/usr/lib");
    }

    println!("cargo:rustc-link-lib=openvino_genai");
    println!("cargo:rustc-link-lib=openvino");

    let mut build = cxx_build::bridge("src/genai_bridge.rs");
    build.file("src/cpp/bridge.cpp");
    build.std("c++17");
    
    if let Some(root) = &openvino_root {
        build.include(format!("{}/runtime/include", root.display()));
    }
    
    build.compile("genai_bridge");
}
