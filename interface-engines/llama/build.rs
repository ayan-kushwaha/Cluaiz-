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
    } else if env::var("CARGO_FEATURE_VULKAN").is_ok() {
        config.define("GGML_VULKAN", "ON");
    } else if env::var("CARGO_FEATURE_ROCM").is_ok() {
        config.define("GGML_HIPBLAS", "ON");
    } else if env::var("CARGO_FEATURE_OPENVINO").is_ok() {
        config.define("GGML_OPENVINO", "ON");
    } else if env::var("CARGO_FEATURE_SYCL").is_ok() {
        config.define("GGML_SYCL", "ON");
    } else if env::var("CARGO_FEATURE_QNN").is_ok() {
        config.define("GGML_QNN", "ON");
    } else if env::var("CARGO_FEATURE_CANN").is_ok() {
        config.define("GGML_CANN", "ON");
        config.define("SOC_TYPE", "ascend910b");
    }

    let dst = config.build();

    // ═══════════════════════════════════════════════════════════════
    // PHASE 3: INDUSTRIAL LINKAGE
    // ═══════════════════════════════════════════════════════════════
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display()); // Some Linux distros
    
    // Dynamically traverse dst to discover all compiled library folders recursively.
    // This perfectly supports multi-config targets (e.g. Xcode, MSVC), Mobile (iOS/Android),
    // and standard CMake Makefile output folders seamlessly!
    find_and_link_search_paths(&dst);

    // Link core libraries produced by CMake
    println!("cargo:rustc-link-lib=static=llama");
    println!("cargo:rustc-link-lib=static=ggml");
    println!("cargo:rustc-link-lib=static=ggml-base");
    println!("cargo:rustc-link-lib=static=ggml-cpu");
    
    // Windows MSVC requires specific system libs for threading
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=ws2_32");
    } else if target_os == "macos" {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalKit");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=dylib=c++");
        
        // 🍏 Add OpenSSL search paths from Homebrew on macOS to resolve missing link symbols
        println!("cargo:rustc-link-search=native=/usr/local/opt/openssl@3/lib");
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/openssl@3/lib");
        println!("cargo:rustc-link-lib=dylib=ssl");
        println!("cargo:rustc-link-lib=dylib=crypto");
    } else if target_os == "ios" {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=UIKit");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        // Linux / Android
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    println!("cargo:warning=🧿 [Llama-Engine] Industrial CMake Build Complete.");
}

fn find_and_link_search_paths(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut has_lib = false;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                find_and_link_search_paths(&path);
            } else if let Some(ext) = path.extension() {
                if ext == "a" || ext == "lib" {
                    has_lib = true;
                }
            }
        }
        if has_lib {
            println!("cargo:rustc-link-search=native={}", dir.display());
        }
    }
}
