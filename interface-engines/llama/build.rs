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
    // Detect target platform first — needed to set Apple cross-compile flags before cmake runs.
    let target_os   = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    let mut config = cmake::Config::new(&llama_path);

    config
        .define("LLAMA_BUILD_EXAMPLES", "OFF")
        .define("LLAMA_BUILD_TESTS",    "OFF")
        .define("LLAMA_BUILD_SERVER",   "OFF")
        .define("LLAMA_STATIC",         "ON")
        .define("BUILD_SHARED_LIBS",    "OFF")
        .define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded") // 🏛️ /MT linkage
        .profile("Release");

    // ── Apple Platform Alignment ──────────────────────────────────────
    // iOS cross-compilation requires a dedicated Xcode CMake target.  Without
    // CMAKE_SYSTEM_NAME=iOS, CMake targets the macOS host and the objects end
    // up with the wrong architecture / sysroot, causing linker failures.
    if target_os == "ios" {
        config.define("CMAKE_SYSTEM_NAME",           "iOS");
        config.define("CMAKE_OSX_SYSROOT",           "iphoneos");
        config.define("CMAKE_OSX_ARCHITECTURES",     &*target_arch); // arm64
        config.define("CMAKE_OSX_DEPLOYMENT_TARGET", "16.0");
    } else if target_os == "macos" {
        // Align with GHA runner SDK (15.x) to prevent "built for newer macOS" link errors.
        config.define("CMAKE_OSX_DEPLOYMENT_TARGET", "15.0");
    }

    // ── GPU Driver Logic (Sovereign Dispatch) ─────────────────────────
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
        // ggml-openvino's CMakeLists.txt calls find_package(OpenCL REQUIRED).
        // CI injects OpenCL_INCLUDE_DIR / OpenCL_LIBRARY so CMake can locate them.
        if let Ok(v) = env::var("OpenCL_INCLUDE_DIR") { config.define("OpenCL_INCLUDE_DIR", &*v); }
        if let Ok(v) = env::var("OpenCL_LIBRARY")     { config.define("OpenCL_LIBRARY",     &*v); }
        if let Ok(v) = env::var("OpenVINO_DIR")       { config.define("OpenVINO_DIR",       &*v); }
    } else if env::var("CARGO_FEATURE_SYCL").is_ok() {
        config.define("GGML_SYCL", "ON");
        // Intel DPC++ (icpx) must be the CXX compiler for SYCL support.
        // CI exports DPCPP_CXX / DPCPP_CC after sourcing setvars.sh.
        if let Ok(v) = env::var("DPCPP_CXX") { config.define("CMAKE_CXX_COMPILER", &*v); }
        if let Ok(v) = env::var("DPCPP_CC")  { config.define("CMAKE_C_COMPILER",   &*v); }
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
    
    // OS-specific system library linkage
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
        // NOTE: llama.cpp is built with LLAMA_BUILD_SERVER=OFF — no OpenSSL dependency.
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
