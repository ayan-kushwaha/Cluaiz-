use engines::InferenceEngine;
use std::path::PathBuf;

#[tokio::test]
async fn test_qwen_ignition() -> Result<(), Box<dyn std::error::Error>> {
    let engine = InferenceEngine::new();
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../models/models--Qwen--Qwen2.5-0.5B-Instruct-GGUF/qwen2.5-0.5b-instruct-q4_k_m.gguf");
    
    // Check reality dynamically
    if !model_path.exists() {
        println!("⚠️ Skipping test: Model not found at {:?}", model_path);
        return Ok(());
    }

    println!("Testing Qwen Ignition on path: {:?}", model_path);
    engine.load_model(model_path).await?;
    println!("✅ Qwen loaded successfully");
    Ok(())
}

#[tokio::test]
async fn test_bonsai_ignition() -> Result<(), Box<dyn std::error::Error>> {
    let engine = InferenceEngine::new();
    let bonsai_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../models/models--prism-ml--Bonsai-4B-gguf/Bonsai-4B-Patched.gguf");
    
    if !bonsai_path.exists() {
        println!("⚠️ Skipping test: Model not found at {:?}", bonsai_path);
        return Ok(());
    }

    println!("Testing Bonsai Ignition on path: {:?}", bonsai_path);
    engine.load_model(bonsai_path).await?;
    Ok(())
}

#[tokio::test]
async fn test_gemma_ignition() -> Result<(), Box<dyn std::error::Error>> {
    let engine = InferenceEngine::new();
    let gemma_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../models/models--lmstudio-community--gemma-2-2b-it-GGUF/gemma-2-2b-it-Q4_K_M.gguf");
    
    if !gemma_path.exists() {
        println!("⚠️ Skipping test: Model not found at {:?}", gemma_path);
        return Ok(());
    }

    println!("Testing Gemma Ignition on path: {:?}", gemma_path);
    engine.load_model(gemma_path).await?;
    Ok(())
}
