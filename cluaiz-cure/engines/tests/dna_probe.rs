use candle_core::quantized::gguf_file;
use std::fs::File;
use std::path::PathBuf;

#[tokio::test]
async fn test_dna_probe() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../models/chat/gemma-4-E2B-it/gemma-4-E2B-it-Q4_K_M.gguf");
    
    if !model_path.exists() {
        println!("❌ Model weights not found at {:?}", model_path);
        return Ok(());
    }

    println!("🔍 [DNA Probe] Scanning GGUF Structural DNA: {:?}", model_path.file_name().unwrap_or_default());
    let mut file = File::open(&model_path)?;
    let content = gguf_file::Content::read(&mut file)?;

    println!("\n📊 --- Architectural Tensor Map ---");
    let mut tensor_metadata: Vec<_> = content.tensor_infos.keys().collect();
    tensor_metadata.sort();

    for name in tensor_metadata {
        if let Some(tensor_metadata_item) = content.tensor_infos.get(name) {
            println!("  {:<40} | Shape: {:?}", name, tensor_metadata_item.shape);
        }
    }
    println!("---------------------------------\n");
    Ok(())
}
