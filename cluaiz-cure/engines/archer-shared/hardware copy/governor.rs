//! ═══════════════════════════════════════════════════════════════════════
//!  Archer Shared: Hardware Governor (OS-Agnostic Intelligence)
//! ═══════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use crate::hardware::mod_types::SovereignProfile;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SystemConfig {
    pub system_identity: Option<SystemIdentity>,
    pub system_context: Option<SystemContext>,
    pub hardware_resources: Option<HardwareResources>,
    pub runtime_engine: Option<RuntimeEngine>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SystemIdentity {
    pub os_target: String,
    pub architecture: String,
    pub kernel_control: String,
    pub power_profile: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SystemContext {
    pub machine_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HardwareResources {
    pub gpu: GPUResources,
    pub cpu: CPUResources,
    pub memory: MemoryResources,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct GPUResources {
    pub has_gpu: bool,
    pub brand: String,
    pub model: String,
    pub vram_total_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CPUResources {
    pub brand: String,
    pub total_cores: u32,
    pub performance_cores: u32,
    pub instruction_sets: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MemoryResources {
    pub ram_total_gb: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuntimeEngine {
    pub model_run_mode: Option<String>,
    pub booster_flags: BoosterFlags,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BoosterFlags {
    #[serde(alias = "TurboQuant_Enable")]
    pub turbo_quant: bool,
    #[serde(alias = "FlashAttention_v2")]
    pub flash_attention: bool,
    #[serde(alias = "AutoRound_Enable", default)]
    pub auto_round: bool,
    #[serde(alias = "Speculative_Decoding", default)]
    pub speculative_decoding: bool,
}

#[derive(Debug, Clone)]
pub struct HardwareGovernor {
    pub platform: String,
    pub base_config_path: PathBuf,
}

impl HardwareGovernor {
    /// Detects the OS and primary hardware context dynamically
    pub fn start() -> Self {
        #[cfg(target_os = "windows")]
        let platform = "Windows".to_string();
        #[cfg(target_os = "linux")]
        let platform = "Linux".to_string();
        #[cfg(target_os = "android")]
        let platform = "Android".to_string();
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "android")))]
        let platform = "Unix-Generic".to_string();

        let base_config_path = Self::resolve_base_path();

        Self {
            platform,
            base_config_path,
        }
    }

    /// Dynamically resolves the configuration path without hardcoding.
    /// Priority: 1. Environment Var, 2. OS Default, 3. Workspace Local
    fn resolve_base_path() -> PathBuf {
        if let Ok(path) = env::var("CLUAIZ_CONFIG_HOME") {
            return PathBuf::from(path);
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = env::var("APPDATA") {
                return PathBuf::from(appdata).join("Cluaiz");
            }
        }

        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            if let Ok(home) = env::var("HOME") {
                return PathBuf::from(home).join(".cluaiz");
            }
        }

        // 3. Last Resort: Workspace Root
        let mut curr = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        while curr.parent().is_some() {
            if curr.join("Cargo.toml").exists() {
                return curr;
            }
            curr = curr.parent().unwrap().to_path_buf();
        }

        PathBuf::from(".")
    }

    /// Verifies if the hardware control system is initialized
    pub fn is_ready(&self) -> bool {
        self.base_config_path.join("system_control.json").exists()
    }

    /// Gets the absolute path for a hardwareDNA asset
    pub fn get_asset_path(&self, asset_name: &str) -> PathBuf {
        self.base_config_path.join(asset_name)
    }

    /// Loads the system_control.json from the resolved platform path
    pub fn load_config() -> Result<SystemConfig, String> {
        let governor_instance = Self::start();
        let config_path = governor_instance.get_asset_path("system_control.json");

        if let Ok(config_content) = std::fs::read_to_string(config_path) {
            serde_json::from_str(&config_content).map_err(|e| format!("Config Parse Error: {e}"))
        } else {
            Err("System Control configuration missing.".into())
        }
    }

    /// Saves the system_control.json to the resolved platform path
    pub fn save_config(config: &SystemConfig) -> std::io::Result<()> {
        let governor_instance = Self::start();
        let config_path = governor_instance.get_asset_path("system_control.json");
        let formatted_content = serde_json::to_string_pretty(config).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::create_dir_all(governor_instance.base_config_path)?;
        std::fs::write(config_path, formatted_content)
    }

    /// Updates a specific field in the config via a dot-path (e.g., "runtime_engine.booster_flags.turbo_quant")
    pub fn update_field(dot_path: &str, property_value: serde_json::Value) -> Result<(), String> {
        let mut system_config = Self::load_config().unwrap_or_default();
        
        // Simple manual routing for critical fields (Keep it high-performance/match-based)
        if dot_path == "runtime_engine.booster_flags.TurboQuant_Enable" {
            if let Some(engine_runtime) = &mut system_config.runtime_engine {
                engine_runtime.booster_flags.turbo_quant = property_value.as_bool().unwrap_or(false);
            }
        } else if dot_path == "runtime_engine.booster_flags.FlashAttention_v2" {
            if let Some(engine_runtime) = &mut system_config.runtime_engine {
                engine_runtime.booster_flags.flash_attention = property_value.as_bool().unwrap_or(false);
            }
        }

        Self::save_config(&system_config).map_err(|save_err| save_err.to_string())
    }

    /// AUTO-CALIBRATE: Probes silicon and aligns system_control.json to reality.
    pub fn auto_calibrate() -> Result<SystemConfig, String> {
        use super::hal::detect_silicon;
        let silicon_stats = detect_silicon();
        
        let mut system_config = Self::load_config().unwrap_or_default();
        
        // 1. Map Reality to Context
        let mut hardware_resources = system_config.hardware_resources.unwrap_or_default();
        hardware_resources.cpu.brand = silicon_stats.cpu_brand.clone();
        hardware_resources.cpu.total_cores = silicon_stats.cpu_cores as u32;
        hardware_resources.memory.ram_total_gb = silicon_stats.mem_total_gb;
        
        hardware_resources.gpu.has_gpu = silicon_stats.has_gpu;
        if let Some(brand_name) = silicon_stats.gpu_brand {
             hardware_resources.gpu.brand = brand_name;
             hardware_resources.gpu.model = "Generic-Accelerator".to_string();
        }
        
        hardware_resources.gpu.vram_total_gb = silicon_stats.vram_total_gb.unwrap_or(0.0);
        
        system_config.hardware_resources = Some(hardware_resources);
        
        // 2. Align Identity
        let mut system_identity = system_config.system_identity.unwrap_or_default();
        system_identity.os_target = silicon_stats.platform.clone();
        system_identity.architecture = "x86_64/arm64-Unified".to_string();
        system_config.system_identity = Some(system_identity);

        Self::save_config(&system_config).map_err(|persist_err| persist_err.to_string())?;
        Ok(system_config)
    }
}
