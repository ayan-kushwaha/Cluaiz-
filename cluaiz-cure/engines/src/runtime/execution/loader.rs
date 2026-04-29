use std::path::Path;
use anyhow::{Result, anyhow};
use tokenizers::Tokenizer;
use crate::models::registry::Provisioner;
use crate::runtime::execution::hub::SiliconOrchestrator as NeuralHub;
use archer_shared::{ModelWeightsWrapper, SovereignContext, StructuralDNA, TemplateManager};
use archer_shared::utils::GGUFProber;

/// GGUFLoader: Lightweight orchestrator for quantized neural models.
pub struct GGUFLoader;

impl GGUFLoader {
    pub async fn load_model(path: &Path, hf_repo: &str) -> Result<(ModelWeightsWrapper, Tokenizer, Option<u32>)> {
        // 1. Detect Architecture via Native Prober (Zero Framework Bloat)
        let (metadata, _tensor_infos) = GGUFProber::probe(path)
            .map_err(|e| anyhow!("Native Probe Failure: {}", e))?;
        
        let arch = metadata.get("general.architecture")
            .ok_or_else(|| anyhow!("Registry Alert: Architecture metadata missing in GGUF file."))?;
        
        tracing::info!("🔍 Autonomous Discovery: Probed architecture '{}' via Native Prober", arch);

        // 2. Extract Special Tokens (Resilient Handshake)
        let bos_token_id = metadata.get("tokenizer.ggml.bos_token_id")
            .and_then(|v| v.parse::<u32>().ok());

        // 3. Identify Metadata Assets (Structural DNA)
        let model_dir = path.parent().ok_or_else(|| anyhow!("Invalid model path structure."))?;
        let dna_path = model_dir.join("structural_dna.json");
        let architectural_dna = StructuralDNA::load(&dna_path)
            .map_err(|load_err| anyhow!("Sovereign Boot Failure: DNA is missing. Detail: {}", load_err))?;

        let tokenizer_path = Provisioner::ensure_assets(model_dir, hf_repo, None, "tokenizer.json").await?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Neural Hardware Error: Failed to load tokenizer: {}", e))?;

        // 🧬 SOVEREIGN ACTIVATION: Dynamic Context Bootstrapping
        let sovereign_context = SovereignContext::boot(
            architectural_dna,
            TemplateManager {
                jinja_template: "".into(), // Future: Load from tokenizer_config
                is_fallback: false,
            }
        );
 
        // 4. Delegate Instantiation to the Neural Hub (Universal DNA Dispatch)
        // The Linker (driver-manager) will resolve the correct .so/.dll here.
        let model = NeuralHub::instantiate(path.to_string_lossy().as_ref(), sovereign_context).await?;
  
        Ok((model, tokenizer, bos_token_id))
    }
}
