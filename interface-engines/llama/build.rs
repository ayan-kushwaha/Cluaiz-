use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let llama_path = Path::new(&out_dir).join("llama.cpp");

    // 🛰️ SOVEREIGN CLONE: Pulling raw code directly from official ggml-org
    if !llama_path.exists() {
        println!("cargo:warning=🔩 Cloning official ggml-org/llama.cpp source...");
        let status = Command::new("git")
            .args(&["clone", "--depth", "1", "https://github.com/ggml-org/llama.cpp", llama_path.to_str().unwrap()])
            .status()
            .expect("Failed to clone official llama.cpp repo");
        
        if !status.success() {
            panic!("Failed to clone official llama.cpp repo. Check your internet connection.");
        }
    }

    // ⚙️ SOVEREIGN COMPILATION: Direct C++ Build
    // NOTE: llama.cpp restructured — ggml sources now live under ggml/src/
    let ggml_src = llama_path.join("ggml").join("src");
    let ggml_include = llama_path.join("ggml").join("include");

    let mut build = cc::Build::new();
    build.cpp(true)
        .include(&llama_path)
        .include(llama_path.join("include"))
        .include(llama_path.join("common"))
        .include(&ggml_include)
        .include(&ggml_src)
        .file(llama_path.join("src").join("llama.cpp"))
        .file(ggml_src.join("ggml.c"))
        .file(ggml_src.join("ggml-alloc.c"))
        .file(ggml_src.join("ggml-backend.c"))
        .file(ggml_src.join("ggml-quants.c"));

    // 🧿 SILICON ACCELERATION SWITCHES
    if env::var("CARGO_FEATURE_CUDA").is_ok() {
        build.define("GGML_USE_CUDA", None);
        println!("cargo:warning=⚡ CUDA acceleration enabled.");
    }

    if env::var("CARGO_FEATURE_METAL").is_ok() {
        build.define("GGML_USE_METAL", None);
        println!("cargo:warning=⚡ Metal acceleration enabled.");
    }

    if env::var("CARGO_FEATURE_VULKAN").is_ok() {
        build.define("GGML_USE_VULKAN", None);
        println!("cargo:warning=⚡ Vulkan acceleration enabled.");
    }

    // 🔥 THE FOUNDRY STAGE: Compile the raw machine code
    build.compile("archer_llama_core");

    println!("cargo:warning=🧿 [Llama-Engine] Sovereign Source Compiled.");
}
