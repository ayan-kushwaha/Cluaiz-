//! Archer-Candle Loader: Weight initialization and DNA-driven GGUF parsing.

use anyhow::Result;
use archer_shared::metadata::dna::StructuralDNA;
use candle_core::{Device};
use candle_core::quantized::gguf_file::Content;
use std::fs::File;
use std::path::PathBuf;
use crate::SovereignModel;
use crate::config::CandleConfig;

pub struct CandleLoader;

impl CandleLoader {
    pub fn load(
        _path: &PathBuf,
        content: Content,
        file: &mut File,
        device: &Device,
        dna: Option<StructuralDNA>,
    ) -> Result<SovereignModel> {
        // 1. Resolve dynamic parameters instead of using hardcoded ones
        let mut dna_ref = dna.ok_or_else(|| anyhow::anyhow!("DNA required for Sovereign V1.0"))?;
        
        // 🏁 [Truth Protocol] Sync with binary metadata before resolution
        dna_ref.sync_with_gguf_metadata(&content.metadata, &content.tensor_infos);
        
        // 2. Routing to the correct model kernel based on "Signature" (Zero Names)
        if dna_ref.signature.is_bitnet {
            tracing::info!("🚀 [Kernel] Routine: 1-bit Neural Logic — Dispatching Variant 1 (BitNet-Agnostic).");
            let weights = candle_transformers::models::quantized_llama::ModelWeights::from_gguf(
                content, file, device,
            )?;


            Ok(SovereignModel::Variant1(weights))
        } else if dna_ref.signature.is_ssm {
            Err(anyhow::anyhow!("Mamba variant is currently disabled for architectural stabilization."))
        } else if dna_ref.signature.is_heterogeneous {
            tracing::info!("🚀 [Kernel] Routine: Heterogeneous Block Logic — Dispatching Variant 2.");
            let weights = candle_transformers::models::quantized_gemma3::ModelWeights::from_gguf(
                content, file, &device,
            )?;
            Ok(SovereignModel::Variant2(weights))
        } else {
            tracing::info!("🚀 [Kernel] Routine: Uniform GQA Logic — Dispatching Variant 1.");
            let weights = candle_transformers::models::quantized_llama::ModelWeights::from_gguf(
                content, file, &device,
            )?;
            Ok(SovereignModel::Variant1(weights))
        }
    }
}
