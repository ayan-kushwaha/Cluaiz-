//! ═══════════════════════════════════════════════════════════════════════
//!  Archer Shared: Hardware Governor (OS-Agnostic Intelligence)
//! ═══════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;


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
    pub base_clock_ghz: f64,
    pub total_threads: u32,
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

        let mut curr = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        while curr.parent().is_some() {
            if curr.join("Cargo.toml").exists() {
                return curr;
            }
            curr = curr.parent().unwrap().to_path_buf();
        }

        PathBuf::from(".")
    }

    pub fn is_ready(&self) -> bool {
        self.base_config_path.join("system_control.json").exists()
    }

    pub fn get_asset_path(&self, asset_name: &str) -> PathBuf {
        self.base_config_path.join(asset_name)
    }

    pub fn load_config() -> Result<SystemConfig, String> {
        let governor_instance = Self::start();
        let config_path = governor_instance.get_asset_path("system_control.json");

        if let Ok(config_content) = std::fs::read_to_string(config_path) {
            serde_json::from_str(&config_content).map_err(|e| format!("Config Parse Error: {e}"))
        } else {
            Err("System Control configuration missing.".into())
        }
    }

    pub fn save_config(config: &SystemConfig) -> std::io::Result<()> {
        let governor_instance = Self::start();
        let config_path = governor_instance.get_asset_path("system_control.json");
        let formatted_content = serde_json::to_string_pretty(config).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::create_dir_all(governor_instance.base_config_path)?;
        std::fs::write(config_path, formatted_content)
    }

    /// AUTO-CALIBRATE: Probes silicon and aligns system_control.json to reality.
    pub fn auto_calibrate() -> Result<SystemConfig, String> {
        use super::super::hal::detect_silicon;
        let silicon_stats = detect_silicon();
        
        let mut system_config = Self::load_config().unwrap_or_default();
        
        // 1. Map Reality to Context (Locking physical boundaries)
        let mut hardware_resources = system_config.hardware_resources.unwrap_or_default();
        hardware_resources.cpu.brand = silicon_stats.cpu_brand.clone();
        hardware_resources.cpu.total_cores = silicon_stats.cpu_cores as u32;
        hardware_resources.cpu.total_threads = silicon_stats.total_threads as u32;
        hardware_resources.cpu.base_clock_ghz = silicon_stats.base_clock_ghz;

        hardware_resources.memory.ram_total_gb = silicon_stats.mem_total_gb;
        
        hardware_resources.memory.ram_total_gb = silicon_stats.mem_total_gb;
        
        hardware_resources.gpu.has_gpu = silicon_stats.compute.has_gpu;
        if let Some(vendor) = &silicon_stats.compute.primary_vendor {
             hardware_resources.gpu.brand = format!("{:?}", vendor);
             hardware_resources.gpu.model = "Physical-Accelerator".to_string();
        }
        
        hardware_resources.gpu.vram_total_gb = silicon_stats.compute.vram_gb;

        
        system_config.hardware_resources = Some(hardware_resources);
        
        // 2. Align Identity
        let mut system_identity = system_config.system_identity.unwrap_or_default();
        system_identity.os_target = silicon_stats.platform.clone();
        system_identity.architecture = std::env::consts::ARCH.to_string();
        system_config.system_identity = Some(system_identity);

        Self::save_config(&system_config).map_err(|persist_err| persist_err.to_string())?;
        Ok(system_config)
    }
    
    /// Update a specific configuration field via dot-notation (UI Integration)
    pub fn update_field(key: &str, value: serde_json::Value) -> Result<(), String> {
        let mut config = Self::load_config().unwrap_or_default();
        let mut engine = config.runtime_engine.unwrap_or_default();
        
        match key {
            "runtime_engine.booster_flags.TurboQuant_Enable" | "runtime_engine.booster_flags.turbo_quant" => {
                engine.booster_flags.turbo_quant = value.as_bool().unwrap_or(true);
            },
            "runtime_engine.booster_flags.FlashAttention_v2" | "runtime_engine.booster_flags.flash_attention" => {
                engine.booster_flags.flash_attention = value.as_bool().unwrap_or(true);
            },
            "runtime_engine.booster_flags.AutoRound_Enable" | "runtime_engine.booster_flags.auto_round" => {
                engine.booster_flags.auto_round = value.as_bool().unwrap_or(false);
            },
            _ => return Err(format!("Key {key} not supported for direct update")),
        }

        config.runtime_engine = Some(engine);
        Self::save_config(&config).map_err(|e| e.to_string())
    }
}
