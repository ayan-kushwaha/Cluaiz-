use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EngineAllocationMode {
    Auto,
    ForceCPU,
    ForceGPU,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserControlConfig {
    pub bitnet_mode: EngineAllocationMode,
    pub chat_mode: EngineAllocationMode,
}

impl Default for UserControlConfig {
    fn default() -> Self {
        Self {
            bitnet_mode: EngineAllocationMode::Auto,
            chat_mode: EngineAllocationMode::ForceGPU,
        }
    }
}

pub struct SystemHealthCheck;

impl SystemHealthCheck {
    /// Queries the OS for real available system memory using sysinfo.
    /// Returns available RAM in GB (not VRAM — for VRAM we rely on candle_core device).
    /// This is used as a proxy for "can we safely load this model?"
    pub fn get_available_memory_gb() -> f32 {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        // available_memory() returns bytes
        sys.available_memory() as f32 / 1_073_741_824.0
    }
}

pub struct SystemAllocator {
    config: UserControlConfig,
}

impl SystemAllocator {
    pub fn new() -> Self {
        // Dynamic path resolution via HardwareGovernor (OS-agnostic)
        let gov = crate::hardware::intelligence::HardwareGovernor::start();
        let settings_path = gov.get_asset_path("settings.json");
        let config = match fs::read_to_string(&settings_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => UserControlConfig::default(),
        };

        Self { config }
    }

    /// Determines the number of GPU layers to assign based on current system state.
    /// Returns 0 for CPU-only, or 999 (all layers) for GPU.
    /// Uses percentage-based thresholds instead of fixed GB values for scale-agnostic behavior.
    pub fn calculate_gpu_layers(&self, model_size_gb: f32, is_bitnet: bool) -> i32 {
        let mode = if is_bitnet {
            &self.config.bitnet_mode
        } else {
            &self.config.chat_mode
        };

        match mode {
            EngineAllocationMode::ForceGPU => 999,
            EngineAllocationMode::ForceCPU => 0,
            EngineAllocationMode::Auto => {
                let available_mem = SystemHealthCheck::get_available_memory_gb();
                
                // Scale-agnostic: require at least model_size + 15% headroom
                let headroom_factor = 1.15;
                let required = model_size_gb * headroom_factor;
                
                if available_mem < required {
                    tracing::warn!("[Auto-Balancer] Low memory ({:.1}GB free, {:.1}GB needed). Offloading to CPU.", available_mem, required);
                    0
                } else {
                    tracing::info!("[Auto-Balancer] Memory OK ({:.1}GB free). Allocating to GPU.", available_mem);
                    999
                }
            }
        }
    }
}
