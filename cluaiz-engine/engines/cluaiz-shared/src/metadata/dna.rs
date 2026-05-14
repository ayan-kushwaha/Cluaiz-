use crate::backend::signature::{BackendType, KernelSignature};
use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use std::collections::HashMap;
use tracing::info;

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
    pub heterogeneous_map: Option<HashMap<String, usize>>,
    pub max_context_length: Option<usize>,
    pub eos_token: Option<String>,
    pub chat_template: Option<String>,
    pub stop_sequences: Vec<String>,
    pub inference_params: HashMap<String, String>,
    pub dynamic_attributes: HashMap<String, String>,
    // Hardware Context
    pub vram_headroom_gb: f32,
    pub ram_headroom_gb: f32,
    pub requires_gpu: bool,
    pub weights_size_gb: f32,
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
            max_context_length: None, // Must be truth-grounded
            eos_token: None,
            chat_template: None,
            stop_sequences: Vec::new(),
            inference_params: HashMap::new(),
            dynamic_attributes: HashMap::new(),
            vram_headroom_gb: 0.0,
            ram_headroom_gb: 0.0,
            requires_gpu: false,
            weights_size_gb: 0.0,
        }
    }
}

// ─── Neural Resource Constants ─────────────────────────────────────────────
const VRAM_CTX_MULTIPLIER: f32 = 4096.0;
const MIN_CONTEXT_FACTOR: usize = 4; // 25% for stability
const DEFAULT_COMPRESSION: f32 = 4.0; // Q4 Standard

impl StructuralDNA {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read DNA: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("DNA Syntax Error: {e}"))
    }

    pub fn load_archived(path: &std::path::Path) -> Result<Self, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("Failed to read Binary DNA: {e}"))?;
        let archived = unsafe { rkyv::archived_root::<StructuralDNA>(&bytes) };
        let deserialized: StructuralDNA = archived.deserialize(&mut rkyv::Infallible).unwrap();
        Ok(deserialized)
    }

    /// 🧬 Neural Discovery: Learns model behavior and cross-references with Hardware Truth.
    pub fn discover_from_path(&mut self, model_dir: &std::path::Path) -> anyhow::Result<()> {
        println!("🧬 [DNA] Sovereign Discovery Heartbeat: Investigating -> {:?}", model_dir);
        let mut arch_limit: Option<usize> = None;
        let mut sliding_window: Option<usize> = None;

        // 🛡️ 0. Hardware Awareness (The Physical Constraints)
        use crate::hardware::governor::HardwareGovernor;
        
        let booster = HardwareGovernor::load_booster_settings().unwrap_or_default();
        let control = HardwareGovernor::load_system_control()?;

        // 🛡️ Truth Protocol: Prioritize Binary Silicon Truth
        self.vram_headroom_gb = control.silicon_truth.accelerators.gpus.iter().map(|g| g.vram_total_gb).sum::<f64>() as f32;
        self.ram_headroom_gb = control.silicon_truth.memory.available_capacity_gb as f32;

        // 🛡️ Manifest Validation: Check if model REQUIRES GPU
        let manifest_path = model_dir.join("model_manifest.json");
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                self.requires_gpu = json.get("requires_gpu").and_then(|v| v.as_bool()).unwrap_or(false);
            }
        }

        if self.requires_gpu && self.vram_headroom_gb == 0.0 {
            return Err(anyhow::anyhow!("❌ [DNA] Hardware Mismatch: This model requires a GPU but none was detected or available. Aborting to prevent freeze."));
        }

        if self.vram_headroom_gb == 0.0 && self.ram_headroom_gb == 0.0 {
            return Err(anyhow::anyhow!("❌ [DNA] Fatal: Hardware Truth Missing or Corrupted. Run 'cluaiz calibrate'."));
        }

        // 1. Read config.json (THE GOLD TRUTH - Architecture limit)
        let config_path = model_dir.join("config.json");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Look at root or inside text_config
                let text_config = json.get("text_config");
                
                if let Some(ctx) = json.get("max_position_embeddings")
                    .or_else(|| text_config.and_then(|t| t.get("max_position_embeddings")))
                    .and_then(|v| v.as_u64()) { arch_limit = Some(ctx as usize); }
                
                if let Some(sw) = json.get("sliding_window")
                    .or_else(|| text_config.and_then(|t| t.get("sliding_window")))
                    .and_then(|v| v.as_u64()) { sliding_window = Some(sw as usize); }
                
                // Architecture Consolidation (Zero-Null Enforcement)
                let target = text_config.unwrap_or(&json);
                self.layer_count = target.get("num_hidden_layers").and_then(|v| v.as_u64()).map(|v| v as usize);
                self.attention_head_count = target.get("num_attention_heads").and_then(|v| v.as_u64()).map(|v| v as usize);
                self.attention_head_count_kv = target.get("num_key_value_heads").and_then(|v| v.as_u64()).map(|v| v as usize);
                self.hidden_size = target.get("hidden_size").and_then(|v| v.as_u64()).map(|v| v as usize);
                self.intermediate_size = target.get("intermediate_size").and_then(|v| v.as_u64()).map(|v| v as usize);
                self.model_identity = json.get("model_type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();

                // 🧬 SOVEREIGN ARCHITECTURE REGISTRY: Inject missing tokens and inference stability parameters
                match self.model_identity.to_lowercase().as_str() {
                    "gemma2" | "gemma" => {
                        self.stop_sequences.push("<end_of_turn>".to_string());
                        self.stop_sequences.push("<turn>".to_string());
                        self.inference_params.insert("repetition_penalty".to_string(), "1.1".to_string());
                    },
                    "llama" => {
                        self.stop_sequences.push("<|eot_id|>".to_string());
                        self.inference_params.insert("repetition_penalty".to_string(), "1.1".to_string());
                    },
                    _ => {
                        self.stop_sequences.push("<turn>".to_string());
                        self.stop_sequences.push("<eos>".to_string());
                        // 1-bit / Low-bit Stability Guard
                        self.inference_params.insert("repetition_penalty".to_string(), "1.1".to_string());
                        self.inference_params.insert("temperature".to_string(), "0.7".to_string());
                    }
                }
            }
        }

        // 2. Read tokenizer_config.json... (Templates & EOS)
        let t_config_path = model_dir.join("tokenizer_config.json");
        if let Ok(content) = std::fs::read_to_string(&t_config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(eos) = json.get("eos_token").and_then(|v| v.as_str()) {
                    self.eos_token = Some(eos.to_string());
                    if !self.stop_sequences.contains(&eos.to_string()) { self.stop_sequences.push(eos.to_string()); }
                }
                if let Some(template) = json.get("chat_template").and_then(|v| v.as_str()) { self.chat_template = Some(template.to_string()); }
            }
        }

        // 🛠️ DEEP TRUTH RESOLUTION
        let mut final_truth = arch_limit.or(sliding_window);

        // Rule: If manual DNA exists, prioritize it but CAP by Architecture to prevent Hallucinations.
        let dna_json_path = model_dir.join("structural_dna.json");
        if let Ok(content) = std::fs::read_to_string(&dna_json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Try root max_context_length first, then dynamic_attributes
                let dna_ctx_val = json.get("max_context_length")
                    .or_else(|| json.get("dynamic_attributes").and_then(|d| d.get("context_window")));
                
                if let Some(val) = dna_ctx_val {
                    let dna_ctx = if let Some(ctx_u) = val.as_u64() {
                        ctx_u as usize
                    } else if let Some(ctx_s) = val.as_str() {
                        Self::parse_context_string(ctx_s)
                    } else { 0 };

                    if dna_ctx > 0 {
                        if let Some(arch_ctx) = final_truth {
                            final_truth = Some(dna_ctx.min(arch_ctx));
                        } else { final_truth = Some(dna_ctx); }
                    }
                }
            }
        }

        if final_truth.is_none() { return Err(anyhow::anyhow!("❌ [DNA] Fatal: Corrupted Model Metadata.")); }

        let ctx = final_truth.unwrap();
        
        // 🧬 SOVEREIGN WEIGHT DISCOVERY
        let mut model_size_gb = 0.0;
        let abs_dir = std::fs::canonicalize(model_dir).unwrap_or(model_dir.to_path_buf());
        println!("📂 [DNA] Investigating Weights in: {:?}", abs_dir);
        
        if let Ok(entries) = std::fs::read_dir(&abs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                println!("  🔍 Found File: {}", name);
                if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
                    if ext == "gguf" || ext == "bin" || ext == "safetensors" {
                        model_size_gb += entry.metadata().map(|m| m.len()).unwrap_or(0) as f64 / 1024.0 / 1024.0 / 1024.0;
                    }
                }
            }
        }
        self.weights_size_gb = model_size_gb as f32;
        println!("📊 [DNA] Weight Discovery Complete: {:.2}GB", self.weights_size_gb);

        // 🛡️ 3. Physical VRAM Arbiter (Sovereign Negotiation)
        // Delegate to Governor for real-time fitting.
        // We temporarily set max_context_length so the Governor can see the architecture cap.
        self.max_context_length = final_truth;
        let final_ctx = HardwareGovernor::negotiate_vram_envelope(&self);
        
        println!("⚖️ [DNA] Sovereign Negotiation Complete: Safely allocated {} tokens based on silicon truth.", final_ctx);

        self.max_context_length = Some(final_ctx);

        // 📊 SOVEREIGN TELEMETRY: Synchronize with Governor Truth
        self.dynamic_attributes.insert("context_window".to_string(), format!("{}k", final_ctx / 1024));
        
        // 🚀 DYNAMIC QUOTA: Mode-aware allocation (No more 75% static wall)
        let gen_headroom = match booster.mode_run {
            crate::hardware::schema::booster::BoosterMode::UltraMaxBoost | crate::hardware::schema::booster::BoosterMode::HyperCluster => 0.95, // 95% for Extreme modes
            crate::hardware::schema::booster::BoosterMode::MaxBoost => 0.90, // 90%
            _ => 0.80, // 80% Standard
        };
        
        let max_gen_tokens = (final_ctx as f64 * gen_headroom) as usize;
        self.inference_params.insert("max_tokens".to_string(), max_gen_tokens.to_string());
        self.inference_params.insert("context_length".to_string(), final_ctx.to_string());

        info!("✅ [DNA] Governor Discovery Complete: Mode {:?} | Window {}k", 
            booster.mode_run, final_ctx / 1024);

        Ok(())
    }
 
    /// Truth Protocol: Synchronizes DNA fields with actual binary metadata.
    pub fn sync_with_metadata(
        &mut self, 
        metadata: &HashMap<String, String>,
        _tensor_infos: &HashMap<String, Vec<usize>>
    ) {
        // [SOVEREIGN CLEAN]: Switched to println for better editor compatibility
        println!("🧬 [DNA] Initiating Multi-Layer Truth Protocol...");
        
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
            } else if key.contains("context_length") || key.contains("max_position_embeddings") {
                if let Ok(v) = value.parse::<usize>() { self.max_context_length = Some(v); }
            } else if key == "general.architecture" {
                self.model_identity = value.clone();
            }
        }
    }



    /// 🛠️ Parser: Converts manifest context strings (e.g., "8k", "128k") to usize.
    pub fn parse_context_string(ctx_str: &str) -> usize {
        let normalized = ctx_str.to_lowercase();
        if normalized.ends_with('k') {
            let num = normalized.trim_end_matches('k').parse::<usize>().unwrap_or(4);
            num * 1024
        } else if normalized.ends_with('m') {
            let num = normalized.trim_end_matches('m').parse::<usize>().unwrap_or(1);
            num * 1024 * 1024
        } else {
            normalized.parse::<usize>().unwrap_or(4096)
        }
    }

    /// 🛠️ Skeleton Factory: Creates a primed DNA backbone from manifest data.
    pub fn create_skeleton(
        id: String,
        has_vision: bool,
        expert_count: Option<usize>,
        bit_depth: f64,
        context_window: &str,
    ) -> Self {
        let mut signature = KernelSignature::default();
        signature.is_multimodal = has_vision;
        if expert_count.is_some() {
            signature.has_experts = true;
        }

        let mut preferred_runtime = Some(BackendType::RuntimeA); // Default: Candle
        if bit_depth < 2.0 {
            signature.is_bitnet = true;
            preferred_runtime = Some(BackendType::RuntimeB); // BitNet -> Llama.cpp
        }

        Self {
            model_identity: id,
            signature,
            preferred_runtime,
            max_context_length: Some(Self::parse_context_string(context_window)),
            ..Default::default()
        }
    }
}
