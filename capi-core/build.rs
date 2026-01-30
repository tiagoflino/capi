use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let openvino_root = env::var("OPENVINO_ROOT").ok();

    if let Some(root) = &openvino_root {
        println!("cargo:rustc-link-search=native={}/runtime/lib/intel64", root);
        println!("cargo:rustc-link-search=native={}/runtime/3rdparty/tbb/lib", root);
    } else {
        println!("cargo:rustc-link-search=native=/usr/lib");
    }

    println!("cargo:rustc-link-lib=openvino_genai_c");
    println!("cargo:rustc-link-lib=openvino_c"); // Often needed as base

    let mut builder = bindgen::Builder::default();
    
    if let Some(root) = &openvino_root {
       builder = builder
            .clang_arg(format!("-I{}/runtime/include", root))
            .clang_arg(format!("-I{}/runtime/include/openvino/genai/c", root)); // specific include might be needed
    }

    let bindings = builder
        .header(if let Some(root) = &openvino_root {
            format!("{}/runtime/include/openvino/genai/c/llm_pipeline.h", root)
        } else {
            "/usr/include/openvino/genai/c/llm_pipeline.h".to_string()
        })
        .header(if let Some(root) = &openvino_root {
            format!("{}/runtime/include/openvino/genai/c/perf_metrics.h", root)
        } else {
             "/usr/include/openvino/genai/c/perf_metrics.h".to_string()
        })
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("ov_genai_.*")
        .allowlist_function("ov_get_last_err_msg")
        .allowlist_function("ov_get_error_info")
        .allowlist_type("ov_genai_.*")
        .allowlist_type("ov_status_e")
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
