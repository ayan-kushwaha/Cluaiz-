use candle_core::quantized::gguf_file;
use std::fs::File;
use std::path::PathBuf;

#[tokio::test]
async fn dump_gguf_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../models/chat/gemma-4-E2B-it/gemma-4-E2B-it-Q4_K_M.gguf");
    
    // Check reality dynamically
    if !model_path.exists() {
        println!("⚠️ Skipping test: Model weights not found at {:?}", model_path);
        return Ok(());
    }

    let mut file = File::open(&model_path)?;
    let content = gguf_file::Content::read(&mut file)?;

    println!("📊 --- GGUF Metadata ---");
    for (key, value) in content.metadata.iter() {
        println!("{}: {:?}", key, value);
    }
    println!("------------------------\n");

    println!("🎨 --- Tensor Info (Sample) ---");
    for (i, (name, tensor_metadata)) in content.tensor_infos.iter().enumerate().take(10) {
        println!("Tensor {}: {} - Shape: {:?}", i, name, tensor_metadata.shape);
    }
    println!("------------------------\n");
    Ok(())
}
