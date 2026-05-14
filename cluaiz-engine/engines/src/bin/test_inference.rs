use anyhow::Result;
use engines::runtime::execution::hub::HardwareOrchestrator;
use cluaiz_shared::{StructuralDNA, CluaizContext, TemplateManager};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 [Test] Starting Raw Inference Test...");

    let home_dir = dirs::home_dir().expect("Could not resolve Home Directory");
    let model_path = home_dir.join(".cluaiz").join("models").join("chat").join("bonsai1-8b").join("Bonsai-8B.gguf");
    
    if !model_path.exists() {
        println!("❌ Model not found at: {:?}", model_path);
        return Ok(());
    }

    let dna = StructuralDNA::default();
    let context = CluaizContext::boot(dna, TemplateManager::default());

    println!("⚙️ [Test] Orchestrating Hardware...");
    let mut engine = HardwareOrchestrator::instantiate(
        model_path.to_str().unwrap(),
        context
    ).await?;

    println!("🚀 [Test] Starting Stream...");
    
    engine.generate_stream(
        "hi",
        100,
        &tokenizers::Tokenizer::from_file(home_dir.join(".cluaiz").join("models").join("chat").join("bonsai1-8b").join("tokenizer.json")).unwrap(),
        Box::new(|token| {
            print!("{}", token);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }),
    )?;

    println!("\n✅ [Test] Inference Finished.");
    Ok(())
}
