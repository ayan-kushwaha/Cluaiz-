use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SmartState<T> {
    Static(String),
    Custom(T),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum FeatureState {
    On,
    Off,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DFlashConfig {
    pub state: String,
    pub budget: u32,
    pub asymmetric_kv: bool,
    pub draft_model_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum KvCacheQuantization {
    #[default]
    #[serde(rename = "Auto", alias = "auto")]
    Auto,
    #[serde(rename = "Kv16", alias = "kv16")]
    Kv16,
    #[serde(rename = "Kv8", alias = "kv8")]
    Kv8,
    #[serde(rename = "Kv4", alias = "kv4")]
    Kv4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ContextShiftingMode {
    #[default]
    #[serde(rename = "Auto", alias = "auto")]
    Auto,
    #[serde(rename = "Off", alias = "off")]
    Off,
    #[serde(rename = "Minimal", alias = "minimal")]
    Minimal,
    #[serde(rename = "Standard", alias = "standard", alias = "on", alias = "On")]
    Standard,
    #[serde(rename = "Aggressive", alias = "aggressive")]
    Aggressive,
    #[serde(rename = "Extreme", alias = "extreme")]
    Extreme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum BoosterMode {
    #[serde(rename = "edge")]
    Edge,
    #[default]
    #[serde(rename = "multitasking")]
    Multitasking,
    #[serde(rename = "balance")]
    Balance,
    #[serde(rename = "max_boost")]
    MaxBoost,
    #[serde(rename = "ultra_max_boost")]
    UltraMaxBoost,
    #[serde(rename = "hyper_cluster")]
    HyperCluster,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoosterControl {
    pub mode_run: BoosterMode,
    pub turbo_quant: FeatureState,
    pub flash_attention: FeatureState,
    pub speculative_decoding: FeatureState,
    pub auto_round: FeatureState,
    pub dflash: SmartState<DFlashConfig>,
    pub kv_cache_quantization: KvCacheQuantization,
    pub context_shifting: ContextShiftingMode,
    pub force_vram_reclaim: FeatureState,
    #[serde(default = "default_n_gpu_layers")]
    pub n_gpu_layers: i32,
    #[serde(default)]
    pub think_mode: FeatureState,
    #[serde(default)]
    pub think_length: String,
    #[serde(default)]
    pub answer_length: String,
    #[serde(default)]
    pub enforce_json: bool,
    #[serde(default)]
    pub force_memory_lock: FeatureState,
    #[serde(default)]
    pub moe_vram_routing: FeatureState,
}

fn default_n_gpu_layers() -> i32 { -1 }

fn main() {
    let json = r#"{
  "mode_run": "edge",
  "turbo_quant": "On",
  "flash_attention": "On",
  "speculative_decoding": "On",
  "auto_round": "On",
  "dflash": "",
  "kv_cache_quantization": "Auto",
  "context_shifting": "Auto",
  "force_vram_reclaim": "On",
  "n_gpu_layers": 0,
  "think_mode": "On",
  "think_length": "",
  "answer_length": "",
  "enforce_json": false,
  "force_memory_lock": "Off",
  "moe_vram_routing": "Off"
}"#;

    match serde_json::from_str::<BoosterControl>(json) {
        Ok(c) => println!("Success: {:?}", c.n_gpu_layers),
        Err(e) => println!("Error: {}", e),
    }
}
