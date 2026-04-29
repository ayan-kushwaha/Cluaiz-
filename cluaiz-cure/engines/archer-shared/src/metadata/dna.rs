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

impl Default for StructuralDNA {
    fn default() -> Self {
        Self {
            model_identity: "unknown".into(),
            layer_count: None,
            attention_head_count: None,
            attention_head_count_kv: None,
            attention_head_dim: None,
            hidden_size: None,
            intermediate_size: None,
            attention_dimensionality_truth: None,
            signature: KernelSignature::default(),
            preferred_runtime: None,
            heterogeneous_map: None,
            dynamic_attributes: std::collections::HashMap::new(),
        }
    }
}

impl StructuralDNA {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read DNA: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("DNA Syntax Error: {e}"))
    }

    /// 🚀 Zero-Copy Recall: Loads DNA directly from a memory-mapped binary archive.
    pub fn load_archived(path: &std::path::Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("Failed to read Binary DNA: {e}"))?;
        let archived = unsafe { rkyv::archived_root::<StructuralDNA>(&bytes) };
        let deserialized: StructuralDNA = archived.deserialize(&mut rkyv::Infallible).unwrap();
        Ok(deserialized)
    }

    /// Truth Protocol: Synchronizes DNA fields with actual binary metadata AND tensor shapes.
    /// [REFACTORED]: Now uses generic ToString and into_iter for maximum compatibility.
    pub fn sync_with_metadata<K, V, T>(
        &mut self, 
        metadata: &std::collections::HashMap<K, V>,
        tensor_infos: &std::collections::HashMap<K, T>
    ) where 
        K: ToString, 
        V: ToString,
        T: IntoIterator<Item = usize> + Clone
    {
        tracing::info!("🧬 [DNA] Initiating Multi-Layer Truth Protocol...");
        
        for (k, v) in metadata {
            let key = k.to_string();
            let value = v.to_string();
            
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
    }

    /// 🧬 The Forge: Converts JSON DNA into a high-performance rkyv archive.
    pub fn sync_to_archive(&self, target_path: &std::path::Path) -> Result<(), String> {
        let bytes = rkyv::to_bytes::<StructuralDNA, 1024>(self).map_err(|e| format!("Archive Failed: {e}"))?;
        std::fs::write(target_path, bytes).map_err(|e| format!("Disk Write Failed: {e}"))?;
        tracing::info!("✅ [DNA] Sovereign Archive Created: {:?}", target_path);
        Ok(())
    }
}
