use std::env;
use std::path::{Path, PathBuf};
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
            .args(&["clone", "--depth", "1", "https://github.com/ggml-org/llama.cpp", llama_path.to_str().unwrap()])
            .status()
            .expect("Failed to clone llama.cpp");
        if !status.success() { panic!("Clone failed"); }
    }

    // ═══════════════════════════════════════════════════════════════
    // PHASE 2: HYBRID COMPILATION — Smart Path Resolution
    // ═══════════════════════════════════════════════════════════════
    let ggml_src = llama_path.join("ggml").join("src");
    let ggml_include = llama_path.join("ggml").join("include");

    let mut build_c = cc::Build::new();
    let mut build_cpp = cc::Build::new();

    let common_includes = [&llama_path, &llama_path.join("include"), &llama_path.join("common"), &ggml_include, &ggml_src];
    for inc in &common_includes { build_c.include(inc); build_cpp.include(inc); }

    // Smart file adder helper
    let mut add_file = |builder: &mut cc::Build, base: &Path, rel_path: &str| {
        let p1 = base.join("src").join(rel_path);
        let p2 = base.join("ggml").join("src").join(rel_path);
        if p1.exists() { builder.file(p1); }
        else if p2.exists() { builder.file(p2); }
        else { println!("cargo:warning=⚠️ File not found: {}", rel_path); }
    };

    // 🏗️ C-ABI Core
    add_file(&mut build_c, &llama_path, "ggml.c");
    add_file(&mut build_c, &llama_path, "ggml-alloc.c");
    add_file(&mut build_c, &llama_path, "ggml-quants.c");
    
    // ggml-backend can be .c or .cpp depending on version
    if llama_path.join("ggml").join("src").join("ggml-backend.cpp").exists() || 
       llama_path.join("src").join("ggml-backend.cpp").exists() {
        add_file(&mut build_cpp, &llama_path, "ggml-backend.cpp");
    } else {
        add_file(&mut build_c, &llama_path, "ggml-backend.c");
    }

    build_c.define("GGML_VERSION", "\"0.1.0\"").define("GGML_COMMIT", "\"archer-sovereign\"").warnings(false);
    build_cpp.cpp(true).std("c++17").file(llama_path.join("src").join("llama.cpp")).warnings(false);

    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "ios" {
        build_cpp.flag("-Wno-c++11-narrowing");
    }

    // ── Driver Linking ──
    if env::var("CARGO_FEATURE_METAL").is_ok() {
        build_c.define("GGML_USE_METAL", None);
        build_cpp.define("GGML_USE_METAL", None);
        add_file(&mut build_c, &llama_path, "ggml-metal.m");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    // ═══════════════════════════════════════════════════════════════
    // PHASE 4: COMPILE
    // ═══════════════════════════════════════════════════════════════
    build_c.compile("ggml_core");
    build_cpp.compile("archer_llama_core");

    println!("cargo:warning=🧿 [Llama-Engine] Resilient Sovereign Build Complete.");
}
