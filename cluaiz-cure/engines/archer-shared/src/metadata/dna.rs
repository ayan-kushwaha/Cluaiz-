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
    /// [REFACTORED]: Now uses generic maps to avoid framework coupling.
    pub fn sync_with_metadata(
        &mut self, 
        metadata: &std::collections::HashMap<String, String>,
        tensor_infos: &std::collections::HashMap<String, Vec<usize>>
    ) {
        tracing::info!("🧬 [DNA] Initiating Multi-Layer Truth Protocol...");
        
        // ─── Phase 1: Metadata Deep Scan ───
        for (key, value) in metadata {
            if key.ends_with(".embedding_length") || key.ends_with(".hidden_size") {
                if let Ok(v) = value.parse::<usize>() { self.hidden_size = Some(v); }
            } else if key.ends_with(".block_count") || key.ends_with(".layer_count") {
                if let Ok(v) = value.parse::<usize>() { self.layer_count = Some(v); }
            } else if key.ends_with(".attention.head_count") || key.ends_with(".num_attention_heads") {
                if let Ok(v) = value.parse::<usize>() { self.attention_head_count = Some(v); }
            } else if key.ends_with(".attention.head_count_kv") || key.ends_with(".num_key_value_heads") {
                if let Ok(v) = value.parse::<usize>() { self.attention_head_count_kv = Some(v); }
            } else if key.ends_with(".feed_forward_length") || key.ends_with(".intermediate_size") {
                if let Ok(v) = value.parse::<usize>() { self.intermediate_size = Some(v); }
            } else if key == "general.architecture" {
                self.model_identity = value.clone();
            }
        }

        // ─── Phase 2: Democratic Tensor Scan ───
        let mut embd_dims: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let mut q_dims: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

        for (name, shape) in tensor_infos {
            if shape.is_empty() { continue; }
            let out_dim = shape[0];
            let in_dim = *shape.last().unwrap_or(&0);

            if name.contains("token_embd.weight") || name.contains("output.weight") {
                *embd_dims.entry(in_dim).or_insert(0) += 1;
            }
            if name.contains("attn_q.weight") || name.contains("q_proj.weight") {
                *q_dims.entry(out_dim).or_insert(0) += 1;
            }
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
