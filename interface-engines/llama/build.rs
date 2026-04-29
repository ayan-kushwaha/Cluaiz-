use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let llama_path = Path::new(&out_dir).join("llama.cpp");

    // ═══════════════════════════════════════════════════════════════
    // PHASE 1: SOVEREIGN CLONE — Pull official ggml-org source
    // ═══════════════════════════════════════════════════════════════
    if !llama_path.exists() {
        println!("cargo:warning=🔩 Cloning official ggml-org/llama.cpp source...");
        let status = Command::new("git")
            .args(&[
                "clone", "--depth", "1",
                "https://github.com/ggml-org/llama.cpp",
                llama_path.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to clone official llama.cpp repo");

        if !status.success() {
            panic!("Failed to clone official llama.cpp repo. Check your internet connection.");
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // PHASE 2: HYBRID COMPILATION — Split C and C++ for maximum stability
    // ═══════════════════════════════════════════════════════════════
    let ggml_src = llama_path.join("ggml").join("src");
    let ggml_include = llama_path.join("ggml").join("include");

    // Common Config
    let common_includes = [
        &llama_path,
        &llama_path.join("include"),
        &llama_path.join("common"),
        &ggml_include,
        &ggml_src,
    ];

    // 1. 🏗️ THE C-ABI CORE (Pure C)
    // Compiling ggml core files as pure C to avoid C++ strictness errors (narrowing, void*).
    let mut build_c = cc::Build::new();
    for inc in &common_includes { build_c.include(inc); }
    
    build_c
        .file(ggml_src.join("ggml.c"))
        .file(ggml_src.join("ggml-alloc.c"))
        .file(ggml_src.join("ggml-backend.c"))
        .file(ggml_src.join("ggml-quants.c"))
        .define("GGML_VERSION", "\"0.1.0\"")
        .define("GGML_COMMIT", "\"archer-sovereign\"")
        .warnings(false); // Silence noise in 3rd party code

    // 2. 🏗️ THE C++ BACKEND (Modern C++)
    // Compiling llama.cpp and driver-specific files as C++.
    let mut build_cpp = cc::Build::new();
    for inc in &common_includes { build_cpp.include(inc); }
    
    build_cpp
        .cpp(true)
        .std("c++17")
        .file(llama_path.join("src").join("llama.cpp"))
        .warnings(false);

    // Apple SDK Narrowing Fix
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "ios" {
        build_cpp.flag("-Wno-c++11-narrowing");
    }

    // ═══════════════════════════════════════════════════════════════
    // PHASE 3: DRIVER LINKING — Connect to Silicon drivers
    // ═══════════════════════════════════════════════════════════════

    // ── NVIDIA CUDA ──
    if env::var("CARGO_FEATURE_CUDA").is_ok() {
        build_cpp.define("GGML_USE_CUDA", None);
        build_cpp.file(ggml_src.join("ggml-cuda.cu"));
        println!("cargo:rustc-link-lib=cuda");
        println!("cargo:rustc-link-lib=cudart");
        println!("cargo:rustc-link-lib=cublas");
        if let Ok(path) = env::var("CUDA_PATH") {
            println!("cargo:rustc-link-search=native={}/lib64", path);
            build_cpp.include(format!("{}/include", path));
        }
    }

    // ── Apple Metal ──
    if env::var("CARGO_FEATURE_METAL").is_ok() {
        build_c.define("GGML_USE_METAL", None);
        build_cpp.define("GGML_USE_METAL", None);
        build_c.file(ggml_src.join("ggml-metal.m"));

        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");
    }

    // ── Vulkan ──
    if env::var("CARGO_FEATURE_VULKAN").is_ok() {
        build_cpp.define("GGML_USE_VULKAN", None);
        build_cpp.file(ggml_src.join("ggml-vulkan.cpp"));
        println!("cargo:rustc-link-lib=vulkan");
    }

    // ── Intel OpenVINO ──
    if env::var("CARGO_FEATURE_OPENVINO").is_ok() {
        if let Ok(ov_path) = env::var("INTEL_OPENVINO_DIR") {
            build_cpp.define("GGML_USE_OPENVINO", None);
            build_cpp.file(llama_path.join("src").join("ggml-openvino.cpp"));
            println!("cargo:rustc-link-lib=openvino");
            println!("cargo:rustc-link-search=native={}/runtime/lib/intel64", ov_path);
        }
    }

    // ── Qualcomm QNN ──
    if env::var("CARGO_FEATURE_QNN").is_ok() {
        if let Ok(qnn_path) = env::var("QNN_SDK_ROOT") {
            build_cpp.define("GGML_USE_QNN", None);
            build_cpp.file(llama_path.join("src").join("ggml-qnn.cpp"));
            println!("cargo:rustc-link-lib=QnnBackend");
            println!("cargo:rustc-link-search=native={}/lib/aarch64-android", qnn_path);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // PHASE 4: COMPILE — Forge the final binaries
    // ═══════════════════════════════════════════════════════════════
    build_c.compile("ggml_core");
    build_cpp.compile("archer_llama_core");

    println!("cargo:warning=🧿 [Llama-Engine] Hybrid Sovereign Build Complete.");
}
