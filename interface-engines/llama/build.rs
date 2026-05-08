use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let llama_path = Path::new(&out_dir).join("llama.cpp");

    // ═══════════════════════════════════════════════════════════════
    // PHASE 1: SOVEREIGN CLONE
    // ═══════════════════════════════════════════════════════════════
    if !llama_path.exists() {
        println!("cargo:warning=🔩 Cloning official ggml-org/llama.cpp source...");
        let status = Command::new("git")
            .args(["clone", "--depth", "1", "https://github.com/ggml-org/llama.cpp", llama_path.to_str().unwrap()])
            .status()
            .expect("Failed to clone llama.cpp");
        if !status.success() { panic!("Clone failed"); }
    }

    // ═══════════════════════════════════════════════════════════════
    // PHASE 2: INDUSTRIAL CMAKE BUILD
    // ═══════════════════════════════════════════════════════════════
    let mut config = cmake::Config::new(&llama_path);
    
    config
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_TESTS", "OFF")
        .define("LLAMA_BUILD_SERVER", "OFF")
        .define("LLAMA_STATIC", "ON")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded") // 🏛️ Force /MT Linkage
        .profile("Release");

    // ── GPU Driver Logic (Sovereign Dispatch) ──
    if env::var("CARGO_FEATURE_CUDA").is_ok() {
        config.define("GGML_CUDA", "ON");
    } else if env::var("CARGO_FEATURE_METAL").is_ok() {
        config.define("GGML_METAL", "ON");
    }

    let dst = config.build();

    // ═══════════════════════════════════════════════════════════════
    // PHASE 3: INDUSTRIAL LINKAGE
    // ═══════════════════════════════════════════════════════════════
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display()); // Some Linux distros
    println!("cargo:rustc-link-search=native={}/build/common", dst.display());
    println!("cargo:rustc-link-search=native={}/build/src", dst.display());

    // Link core libraries produced by CMake
    println!("cargo:rustc-link-lib=static=llama");
    println!("cargo:rustc-link-lib=static=ggml");
    println!("cargo:rustc-link-lib=static=ggml-base");
    println!("cargo:rustc-link-lib=static=ggml-cpu");
    
    // Windows MSVC requires specific system libs for threading
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=ws2_32");
    }

    println!("cargo:warning=🧿 [Llama-Engine] Industrial CMake Build Complete.");
}
