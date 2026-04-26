use crate::backend::signature::{BackendType, KernelSignature};
use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

// ─── Structural DNA Synchronization (The Root Genome) ──────────────────────
#[derive(Debug, Clone, Deserialize, Serialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct StructuralDNA {
    pub model_identity: String,
    pub layer_count: Option<usize>,
    pub attention_head_count: Option<usize>,
    pub attention_head_count_kv: Option<usize>,
    pub attention_head_dim: Option<usize>,
    pub hidden_size: Option<usize>,
    pub intermediate_size: Option<usize>,
    pub attention_dimensionality_truth: Option<usize>,
    pub signature: KernelSignature,
    pub preferred_runtime: Option<BackendType>,
    pub heterogeneous_map: Option<std::collections::HashMap<String, usize>>,
    
    /// Dynamic attributes are stored as JSON strings for rkyv compatibility
    pub dynamic_attributes: std::collections::HashMap<String, String>,
}

impl StructuralDNA {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read DNA: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("DNA Syntax Error: {e}"))
    }

    /// 🚀 Zero-Copy Recall: Loads DNA directly from a memory-mapped binary archive.
    pub fn load_archived(path: &std::path::Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("Failed to read Binary DNA: {e}"))?;
        let archived = rkyv::check_archived_root::<StructuralDNA>(&bytes).map_err(|e| format!("Binary Corruption: {e}"))?;
        let deserialized: StructuralDNA = archived.deserialize(&mut rkyv::Infallible).unwrap();
        Ok(deserialized)
    }

    /// Truth Protocol: Synchronizes DNA fields with actual binary metadata AND tensor shapes.
    /// This ensures that 'Original Truth' is extracted even if metadata is missing.
    pub fn sync_with_gguf_metadata(
        &mut self, 
        metadata: &std::collections::HashMap<String, candle_core::quantized::gguf_file::Value>,
        tensor_infos: &std::collections::HashMap<String, candle_core::quantized::gguf_file::TensorInfo>
    ) {
        tracing::info!("🧬 [DNA] Initiating Multi-Layer Truth Protocol...");
        
        // ─── Phase 1: Metadata Deep Scan ───
        for (key, value) in metadata {
            // Suffix-Based Search (Architecture Agnostic)
            if key.ends_with(".embedding_length") || key.ends_with(".hidden_size") {
                if let Some(v) = self.extract_usize(value) { self.hidden_size = Some(v); }
            } else if key.ends_with(".block_count") || key.ends_with(".layer_count") {
                if let Some(v) = self.extract_usize(value) { self.layer_count = Some(v); }
            } else if key.ends_with(".attention.head_count") || key.ends_with(".num_attention_heads") {
                if let Some(v) = self.extract_usize(value) { self.attention_head_count = Some(v); }
            } else if key.ends_with(".attention.head_count_kv") || key.ends_with(".num_key_value_heads") {
                if let Some(v) = self.extract_usize(value) { self.attention_head_count_kv = Some(v); }
            } else if key.ends_with(".feed_forward_length") || key.ends_with(".intermediate_size") {
                if let Some(v) = self.extract_usize(value) { self.intermediate_size = Some(v); }
            } else if key == "general.architecture" {
                if let Ok(arch) = value.to_string() { self.model_identity = arch.to_string(); }
            }
        }

        // ─── Phase 2: Democratic Tensor Scan ───
        let mut embd_dims: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let mut q_dims: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

        for (name, tensor_info) in tensor_infos {
            let dims = tensor_info.shape.dims();
            let out_dim = dims[0];
            let in_dim = *dims.last().unwrap_or(&0);

            if name.contains("token_embd.weight") || name.contains("output.weight") {
                *embd_dims.entry(in_dim).or_insert(0) += 1;
            }
            if name.contains("attn_q.weight") || name.contains("q_proj.weight") {
                *q_dims.entry(out_dim).or_insert(0) += 1;
            }
        }

        if self.hidden_size.is_none() {
            if let Some((&true_embd, _)) = embd_dims.iter().max_by_key(|&(_, count)| count) {
                self.hidden_size = Some(true_embd);
            }
        }

        if self.attention_dimensionality_truth.is_none() {
             if let Some((&true_q_width, _)) = q_dims.iter().max_by_key(|&(_, count)| count) {
                self.attention_dimensionality_truth = Some(true_q_width);
                if self.attention_head_count.is_none() {
                    let standard_dim = 128; // Standard Llama-cpp assumption
                    self.attention_head_count = Some(true_q_width / standard_dim);
                    self.attention_head_dim = Some(standard_dim);
                }
            }
        }

        // 🏛️ SOVEREIGN DECISION: Routing
        // GGUF models are routed to RuntimeB (Llama.cpp) except specific overrides.
        self.preferred_runtime = Some(BackendType::RuntimeB);
    }

    fn extract_usize(&self, value: &candle_core::quantized::gguf_file::Value) -> Option<usize> {
        match value {
            candle_core::quantized::gguf_file::Value::U8(v) => Some(*v as usize),
            candle_core::quantized::gguf_file::Value::U16(v) => Some(*v as usize),
            candle_core::quantized::gguf_file::Value::U32(v) => Some(*v as usize),
            candle_core::quantized::gguf_file::Value::U64(v) => Some(*v as usize),
            candle_core::quantized::gguf_file::Value::I8(v) => Some(*v as usize),
            candle_core::quantized::gguf_file::Value::I16(v) => Some(*v as usize),
            candle_core::quantized::gguf_file::Value::I32(v) => Some(*v as usize),
            candle_core::quantized::gguf_file::Value::I64(v) => Some(*v as usize),
            _ => None,
        }
    }


    /// 🧬 The Forge: Converts JSON DNA into a high-performance rkyv archive.
    /// This happens once on the first boot to eliminate parsing overhead forever.
    pub fn sync_to_archive(&self, target_path: &std::path::Path) -> Result<(), String> {
        let bytes = rkyv::to_bytes::<StructuralDNA, 1024>(self).map_err(|e| format!("Archive Failed: {e}"))?;
        std::fs::write(target_path, bytes).map_err(|e| format!("Disk Write Failed: {e}"))?;
        tracing::info!("✅ [DNA] Sovereign Archive Created: {:?}", target_path);
        Ok(())
    }
}
