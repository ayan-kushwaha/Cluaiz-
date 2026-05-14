//! 🚀 Sovereign Booster: Dynamic Configuration System
//! This module translates Registry-level capabilities into low-level engine parameters.

use serde::{Serialize, Deserialize};
use serde_json;
use crate::ffi::llama_cpp::{LlamaModelParams, LlamaContextParams, llama_model_default_params, llama_context_default_params};
use cluaiz_shared::hardware::schema::booster::{BoosterControl, BoosterMode, FeatureState, SmartState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoosterConfig {
    #[serde(skip_serializing)]
    pub n_gpu_layers: i32,
    pub flash_attn: bool,
    #[serde(skip_serializing)]
    pub use_mmap: bool,
    #[serde(skip_serializing)]
    pub n_ctx: u32,
    #[serde(skip_serializing)]
    pub n_threads: i32,
    pub turbo_quant: String,
    pub dflash: String, // 🏛️ Delta Flash (FlashKDA Support)
    pub speculative_decoding: String,
    pub auto_round: String,
    pub mode_run: String,
    pub force_vram_reclaim: String,
}

impl Default for BoosterConfig {
    fn default() -> Self {
        Self {
            n_gpu_layers: -1, 
            flash_attn: true,
            use_mmap: true,
            n_ctx: 0,
            n_threads: -1,
            turbo_quant: "Auto".to_string(),
            dflash: "Auto".to_string(),
            speculative_decoding: "Auto".to_string(),
            auto_round: "Auto".to_string(),
            mode_run: "balance".to_string(),
            force_vram_reclaim: "Off".to_string(),
        }
    }
}

impl BoosterConfig {
    /// 🚀 Load the booster configuration from the sovereign system control.
    pub fn load_from_system() -> Self {
        // Default to Industrial Auto standards
        let mut config = Self {
            flash_attn: true,
            use_mmap: true,
            n_gpu_layers: -1, // Full Offload
            n_ctx: 0,        // Auto-detect from model
            n_threads: -1,   // Auto-detect CPU cores
            turbo_quant: "Auto".to_string(),
            dflash: "Auto".to_string(),
            speculative_decoding: "Auto".to_string(),
            auto_round: "Auto".to_string(),
            mode_run: "balance".to_string(),
            force_vram_reclaim: "Off".to_string(),
        };
        
        // 🛡️ Sovereign Dynamic Pathing: Use cluaiz-shared to resolve the engine path universally.
        let booster_path = cluaiz_shared::hardware::governor::HardwareGovernor::resolve_engine_path()
            .join("system_booster.json");

        if let Ok(content) = std::fs::read_to_string(booster_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                // Only override if explicitly provided in JSON
                if let Some(fa) = json.get("flash_attn") {
                    config.flash_attn = fa.as_bool().unwrap_or(true);
                }
                if let Some(gl) = json.get("n_gpu_layers") {
                    config.n_gpu_layers = gl.as_i64().unwrap_or(-1) as i32;
                }
                if let Some(df) = json.get("dflash") {
                    config.dflash = df.as_str().unwrap_or("Auto").to_string();
                }
                if let Some(sd) = json.get("speculative_decoding") {
                    config.speculative_decoding = sd.as_str().unwrap_or("Auto").to_string();
                }
                if let Some(ar) = json.get("auto_round") {
                    config.auto_round = ar.as_str().unwrap_or("Auto").to_string();
                }
                if let Some(mr) = json.get("mode_run") {
                    config.mode_run = mr.as_str().unwrap_or("balance").to_string();
                }
                if let Some(fr) = json.get("force_vram_reclaim") {
                    config.force_vram_reclaim = fr.as_str().unwrap_or("Off").to_string();
                }
            }
        }
        config
    }
    /// 🛠️ Transform high-level config into raw model parameters.
    pub fn to_model_params(&self) -> LlamaModelParams {
        let mut params = unsafe { llama_model_default_params() };
        params.n_gpu_layers = self.n_gpu_layers;
        params.use_mmap = self.use_mmap;
        params
    }

    /// 🛠️ Transform high-level config into raw context parameters.
    pub fn to_context_params(&self) -> LlamaContextParams {
        let mut params = unsafe { llama_context_default_params() };
        
        // 🛡️ Sovereign Context Handshake: 
        // We use the requested context directly. The Governor's fitting loop 
        // ensures this fits in VRAM before the engine is even initialized.
        params.n_ctx = self.n_ctx;
        
        params.n_threads = self.n_threads;
        params.flash_attn_type = if self.flash_attn { 1 } else { 0 }; // 1 = LLAMA_FLASH_ATTN_TYPE_ENABLED
        
        // 🚀 TurboQuant: Inject KV-cache quantization to save memory
        // 2 = GGML_TYPE_Q4_0, 1 = GGML_TYPE_F16 (Default)
        if self.turbo_quant == "On" || self.turbo_quant == "Auto" {
            params.type_k = 2; 
            params.type_v = 2;
        }

        params
    }

    pub fn to_booster_control(&self) -> BoosterControl {
        BoosterControl {
            mode_run: match self.mode_run.to_lowercase().as_str() {
                "edge" => BoosterMode::Edge,
                "multitasking" => BoosterMode::Multitasking,
                "balance" => BoosterMode::Balance,
                "max_boost" => BoosterMode::MaxBoost,
                "ultra_max_boost" => BoosterMode::UltraMaxBoost,
                "hyper_cluster" => BoosterMode::HyperCluster,
                _ => BoosterMode::Balance,
            },
            turbo_quant: if self.turbo_quant == "On" { FeatureState::On } else if self.turbo_quant == "Off" { FeatureState::Off } else { FeatureState::Auto },
            flash_attention: if self.flash_attn { FeatureState::On } else { FeatureState::Off },
            speculative_decoding: if self.speculative_decoding == "On" { FeatureState::On } else if self.speculative_decoding == "Off" { FeatureState::Off } else { FeatureState::Auto },
            auto_round: if self.auto_round == "On" { FeatureState::On } else if self.auto_round == "Off" { FeatureState::Off } else { FeatureState::Auto },
            dflash: SmartState::Static(self.dflash.clone()),
            force_vram_reclaim: if self.force_vram_reclaim == "On" { FeatureState::On } else { FeatureState::Off },
        }
    }
}
