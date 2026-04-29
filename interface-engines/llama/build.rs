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
    // PHASE 2: SOVEREIGN COMPILATION — Build C++ with cc crate
    // ═══════════════════════════════════════════════════════════════
    let ggml_src = llama_path.join("ggml").join("src");
    let ggml_include = llama_path.join("ggml").join("include");

    let mut build = cc::Build::new();
    build
        .cpp(true)
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

    // ═══════════════════════════════════════════════════════════════
    // PHASE 3: DRIVER LINKING — Connect to actual hardware drivers
    // Without this, the binary compiles but CRASHES at runtime
    // because it can't find the GPU/NPU libraries.
    // ═══════════════════════════════════════════════════════════════

    // ── NVIDIA CUDA ──
    if env::var("CARGO_FEATURE_CUDA").is_ok() {
        build.define("GGML_USE_CUDA", None);
        build.file(ggml_src.join("ggml-cuda.cu"));

        // Link the actual CUDA runtime and driver libraries
        println!("cargo:rustc-link-lib=cuda");
        println!("cargo:rustc-link-lib=cudart");
        println!("cargo:rustc-link-lib=cublas");
        println!("cargo:rustc-link-lib=cublasLt");

        // Search paths (CUDA toolkit default locations)
        if let Ok(cuda_path) = env::var("CUDA_PATH") {
            println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
            println!("cargo:rustc-link-search=native={}/lib/x64", cuda_path);
            build.include(format!("{}/include", cuda_path));
        } else {
            // Default Linux CUDA path
            println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
        }

        println!("cargo:warning=⚡ CUDA: Driver libraries linked (cuda, cudart, cublas).");
    }

    // ── Apple Metal ──
    if env::var("CARGO_FEATURE_METAL").is_ok() {
        build.define("GGML_USE_METAL", None);
        build.file(ggml_src.join("ggml-metal.m"));

        // Link Apple frameworks (Metal, Foundation, Accelerate)
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Accelerate");
        println!("cargo:rustc-link-lib=framework=MetalPerformanceShaders");

        println!("cargo:warning=⚡ Metal: Apple frameworks linked (Metal, Accelerate, MPS).");
    }

    // ── Vulkan (Cross-platform GPU) ──
    if env::var("CARGO_FEATURE_VULKAN").is_ok() {
        build.define("GGML_USE_VULKAN", None);
        build.file(ggml_src.join("ggml-vulkan.cpp"));

        // Link Vulkan SDK
        println!("cargo:rustc-link-lib=vulkan");

        if let Ok(vulkan_sdk) = env::var("VULKAN_SDK") {
            println!("cargo:rustc-link-search=native={}/lib", vulkan_sdk);
            build.include(format!("{}/include", vulkan_sdk));
        }

        println!("cargo:warning=⚡ Vulkan: SDK linked.");
    }

    // ── AMD ROCm ──
    if env::var("CARGO_FEATURE_ROCM").is_ok() {
        build.define("GGML_USE_HIPBLAS", None);

        println!("cargo:rustc-link-lib=hipblas");
        println!("cargo:rustc-link-lib=rocblas");
        println!("cargo:rustc-link-lib=amdhip64");

        if let Ok(rocm_path) = env::var("ROCM_PATH") {
            println!("cargo:rustc-link-search=native={}/lib", rocm_path);
            build.include(format!("{}/include", rocm_path));
        } else {
            println!("cargo:rustc-link-search=native=/opt/rocm/lib");
        }

        println!("cargo:warning=⚡ ROCm: AMD HIP libraries linked.");
    }

    // ── Intel OpenVINO (NPU/GPU) ──
    if env::var("CARGO_FEATURE_OPENVINO").is_ok() {
        if let Ok(ov_path) = env::var("INTEL_OPENVINO_DIR") {
            build.define("GGML_USE_OPENVINO", None);
            build.file(llama_path.join("src").join("ggml-openvino.cpp"));
            println!("cargo:rustc-link-lib=openvino");
            println!("cargo:rustc-link-lib=openvino_c");
            println!("cargo:rustc-link-search=native={}/runtime/lib/intel64", ov_path);
            build.include(format!("{}/runtime/include", ov_path));
            println!("cargo:warning=⚡ OpenVINO: Intel NPU/GPU drivers linked.");
        } else {
            println!("cargo:warning=⚠️ OpenVINO: INTEL_OPENVINO_DIR not found. Skipping NPU support.");
        }
    }

    // ── Qualcomm QNN (Snapdragon NPU) ──
    if env::var("CARGO_FEATURE_QNN").is_ok() {
        if let Ok(qnn_path) = env::var("QNN_SDK_ROOT") {
            build.define("GGML_USE_QNN", None);
            build.file(llama_path.join("src").join("ggml-qnn.cpp"));
            println!("cargo:rustc-link-lib=QnnBackend");
            println!("cargo:rustc-link-lib=QnnSystem");
            println!("cargo:rustc-link-search=native={}/lib/aarch64-android", qnn_path);
            build.include(format!("{}/include/QNN", qnn_path));
            println!("cargo:warning=⚡ QNN: Qualcomm Snapdragon NPU drivers linked.");
        } else {
            println!("cargo:warning=⚠️ QNN: QNN_SDK_ROOT not found. Skipping Snapdragon NPU support.");
        }
    }

    // ── ARM Neon (CPU Optimization) ──
    if env::var("CARGO_FEATURE_ARM_NEON").is_ok() {
        build.define("GGML_USE_ARM_NEON", None);
        println!("cargo:warning=⚡ ARM Neon: Mobile/Pi CPU optimizations enabled.");
    }

    // ═══════════════════════════════════════════════════════════════
    // PHASE 4: COMPILE — Forge the raw machine code
    // ═══════════════════════════════════════════════════════════════
    build.compile("archer_llama_core");

    println!("cargo:warning=🧿 [Llama-Engine] Sovereign Source + Drivers Compiled.");
}
