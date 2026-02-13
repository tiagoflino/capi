use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=../openvino.conf");

    let conf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Cannot find workspace root")
        .join("openvino.conf");

    let content = std::fs::read_to_string(&conf_path)
        .unwrap_or_else(|_| panic!("Cannot read {}", conf_path.display()));

    let mut vars: HashMap<String, String> = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            vars.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let version = vars.get("OPENVINO_VERSION")
        .expect("OPENVINO_VERSION not found in openvino.conf");
    let short = vars.get("OPENVINO_SHORT")
        .expect("OPENVINO_SHORT not found in openvino.conf");

    let parts: Vec<&str> = short.split('.').collect();
    let soname = format!(
        "{}{}{}",
        &parts[0][parts[0].len()-2..],
        parts[1],
        parts[2],
    );

    println!("cargo:rustc-env=OV_VERSION={}", version);
    println!("cargo:rustc-env=OV_SHORT={}", short);
    println!("cargo:rustc-env=OV_SONAME={}", soname);
}
