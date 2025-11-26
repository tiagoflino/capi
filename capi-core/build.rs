use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    println!("cargo:rustc-link-search=native=/usr/lib");
    println!("cargo:rustc-link-lib=openvino_genai_c");

    let bindings = bindgen::Builder::default()
        .header("/usr/include/openvino/genai/c/llm_pipeline.h")
        .header("/usr/include/openvino/genai/c/perf_metrics.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("ov_genai_.*")
        .allowlist_type("ov_genai_.*")
        .allowlist_var("OV_GENAI_.*")
        .derive_debug(true)
        .derive_default(true)
        .generate()
        .expect("Failed to generate OpenVINO GenAI bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("genai_bindings.rs"))
        .expect("Failed to write bindings");
}
