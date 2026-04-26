//! ═══════════════════════════════════════════════════════════════════════
//!  CURE Engine: Autonomous Neural Loader
//! ═══════════════════════════════════════════════════════════════════════

use std::path::Path;
use anyhow::{Result, anyhow};
use candle_core::Device;
use tokenizers::Tokenizer;
use crate::models::registry::Provisioner;
use crate::runtime::execution::hub::SiliconOrchestrator as NeuralHub;
use archer_shared::{ModelWeightsWrapper, SovereignContext, StructuralDNA, TemplateManager};

/// GGUFLoader: High-performance loader for quantized neural models with total architectural autonomy.
pub struct GGUFLoader;

impl GGUFLoader {
    pub async fn load_model(path: &Path, hf_repo: &str, device: &Device) -> Result<(ModelWeightsWrapper, Tokenizer, Device, Option<u32>)> {
        let mut file = std::fs::File::open(path)?;
        let content = candle_core::quantized::gguf_file::Content::read(&mut file)?;
        
        // 1. Detect Architecture (Single Source of Truth)
        let arch_val = content.metadata.get("general.architecture")
            .ok_or_else(|| anyhow!("Registry Alert: Architecture metadata missing in GGUF file."))?;
        
        let arch = arch_val.to_string()?.clone();
        tracing::info!("🔍 Autonomous Discovery: Probing architecture '{}'", arch);

        // 2. Extract Special Tokens (Resilient Handshake)
        let bos_token_id = content.metadata.get("tokenizer.ggml.bos_token_id")
            .and_then(|token_val| match token_val {
                candle_core::quantized::gguf_file::Value::U32(id) => Some(*id),
                candle_core::quantized::gguf_file::Value::I32(id) => Some(*id as u32),
                _ => None
            });

        // 3. Identify Metadata Assets (Structural DNA)
        let model_dir = path.parent().ok_or_else(|| anyhow!("Invalid model path structure."))?;
        let dna_path = model_dir.join("structural_dna.json");
        let architectural_dna = StructuralDNA::load(&dna_path)
            .map_err(|load_err| anyhow!("Sovereign Boot Failure: DNA is missing for Architecture. Detail: {}", load_err))?;

        let tokenizer_path = Provisioner::ensure_assets(model_dir, hf_repo, None, "tokenizer.json").await?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|tokenizer_error| anyhow!("Neural Hardware Error: Failed to load tokenizer: {}", tokenizer_error))?;

        let tok_cfg_path = model_dir.join("tokenizer_config.json");
        let chat_jinja_template = if tok_cfg_path.exists() {
            let template_raw_content = std::fs::read_to_string(&tok_cfg_path).unwrap_or_default();
            let parsed_config: serde_json::Value = serde_json::from_str(&template_raw_content).unwrap_or(serde_json::Value::Null);
            parsed_config["chat_template"].as_str().unwrap_or("").to_string()
        } else {
            "".to_string()
        };

        // 🧬 SOVEREIGN ACTIVATION: Dynamic Context Bootstrapping
        let sovereign_context = SovereignContext::boot(
            architectural_dna,
            TemplateManager {
                jinja_template: chat_jinja_template,
                is_fallback: false,
            }
        );
 
        // 4. Delegate Instantiation to the Neural Hub (Universal DNA Dispatch)
        let model = NeuralHub::instantiate(path.to_string_lossy().as_ref(), sovereign_context)?;
 
        tracing::info!("✅ Neural Activation Successful: {} is online. (BOS: {:?})", arch, bos_token_id);
        
        Ok((model, tokenizer, device.clone(), bos_token_id))

    }
}
