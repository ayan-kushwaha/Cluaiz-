use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let candle_path = Path::new(&out_dir).join("candle");

    // 🛰️ SOVEREIGN CLONE: Pulling raw code directly from official huggingface/candle
    if !candle_path.exists() {
        println!("cargo:warning=🔩 Cloning official huggingface/candle source...");
        let status = Command::new("git")
            .args(&["clone", "--depth", "1", "https://github.com/huggingface/candle", candle_path.to_str().unwrap()])
            .status()
            .expect("Failed to clone official candle repo");
        
        if !status.success() {
            panic!("Failed to clone official candle repo. Check your internet connection.");
        }
    }

    // 🔥 THE FOUNDRY STAGE: Candle is Rust-native, so we prepare the source
    // for the main Cargo build. 
    println!("cargo:warning=🧿 [Universal-Engine] Sovereign Source Ready.");
}
