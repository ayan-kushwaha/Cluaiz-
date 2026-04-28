use crate::hardware::schema::booster::BoosterControl;
use crate::hardware::schema::profiles::SystemControl;
use crate::hardware::system_control::HardwareOrchestrator;
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// 🧠 VRAM Arbiter State: Tracks real-time resource allocations.
pub struct ArbiterState {
    pub total_vram_gb: f64,
    pub allocated_vram_gb: f64,
    pub active_allocations: HashMap<String, f64>,
}

static ARBITER: Lazy<Mutex<ArbiterState>> = Lazy::new(|| {
    Mutex::new(ArbiterState {
        total_vram_gb: 0.0,
        allocated_vram_gb: 0.0,
        active_allocations: HashMap::new(),
    })
});

#[derive(Clone, Copy, Default)]
pub struct HardwareGovernor;

impl HardwareGovernor {
    /// 🚀 Initialize the Governor and resolve hardware state.
    pub fn start() -> Self {
        Self
    }

    /// 🛡️ Checks if the 'system_control.json' fingerprint exists.
    pub fn is_ready(&self) -> bool {
        Self::resolve_base_path().join("system_control.json").exists()
    }

    /// 🔬 Deep surgical scan and persistence of silicon state.
    pub fn auto_calibrate() -> anyhow::Result<()> {
        let control = HardwareOrchestrator::start()?;
        
        // Update Arbiter with latest hardware truth
        if let Ok(mut arbiter) = ARBITER.lock() {
            let total = control.silicon_truth.accelerators.gpus.iter()
                .map(|g| g.vram_available_gb)
                .sum::<f64>();
            arbiter.total_vram_gb = total;
        }
        
        Ok(())
    }

    /// ⚖️ Request VRAM allocation for a neural engine.
    /// Prevents OOM by enforcing the sovereign memory budget.
    pub fn request_vram(engine_id: &str, required_gb: f64) -> anyhow::Result<()> {
        let mut arbiter = ARBITER.lock().map_err(|_| anyhow::anyhow!("Arbiter Lock Poisoned"))?;
        
        // If total_vram is 0, we might need a quick calibration
        if arbiter.total_vram_gb == 0.0 {
            let _ = Self::auto_calibrate();
        }

        let available = arbiter.total_vram_gb - arbiter.allocated_vram_gb;
        
        if required_gb > available {
            return Err(anyhow::anyhow!(
                "❌ [VRAM Arbiter] Out of Memory! Requested: {:.2}GB, Available: {:.2}GB (Total: {:.2}GB)",
                required_gb, available, arbiter.total_vram_gb
            ));
        }

        // Allocate
        arbiter.allocated_vram_gb += required_gb;
        arbiter.active_allocations.insert(engine_id.to_string(), required_gb);
        
        println!("✅ [VRAM Arbiter] Allocated {:.2}GB to '{}'. Current Load: {:.2}/{:.2}GB", 
                 required_gb, engine_id, arbiter.allocated_vram_gb, arbiter.total_vram_gb);
        
        Ok(())
    }

    /// 🔓 Release VRAM allocation when an engine is unloaded.
    pub fn release_vram(engine_id: &str) -> anyhow::Result<()> {
        let mut arbiter = ARBITER.lock().map_err(|_| anyhow::anyhow!("Arbiter Lock Poisoned"))?;
        
        if let Some(freed_gb) = arbiter.active_allocations.remove(engine_id) {
            arbiter.allocated_vram_gb -= freed_gb;
            println!("🔓 [VRAM Arbiter] Released {:.2}GB from '{}'. Current Load: {:.2}/{:.2}GB", 
                     freed_gb, engine_id, arbiter.allocated_vram_gb, arbiter.total_vram_gb);
        }
        
        Ok(())
    }

    /// ⚙️ Updates a specific field in the sovereign configuration.
    pub fn update_field(field: &str, value: serde_json::Value) -> anyhow::Result<()> {
        let mut control = HardwareOrchestrator::start()?;
        
        // ⚙️ Sovereign Configuration Dispatch
        match field {
            "machine_name" => {
                if let Some(s) = value.as_str() {
                    control.identity.machine_name = s.to_string();
                }
            }
            "runtime_engine.booster_flags.TurboQuant_Enable" => {
                let mut booster = Self::load_booster_settings().unwrap_or_default();
                if let Some(b) = value.as_bool() {
                    booster.turbo_quant = if b {
                        crate::hardware::schema::booster::FeatureState::On
                    } else {
                        crate::hardware::schema::booster::FeatureState::Off
                    };
                    Self::save_booster_settings(&booster)?;
                }
            }
            "runtime_engine.booster_flags.FlashAttention_v2" => {
                let mut booster = Self::load_booster_settings().unwrap_or_default();
                if let Some(b) = value.as_bool() {
                    booster.flash_attention = if b {
                        crate::hardware::schema::booster::FeatureState::On
                    } else {
                        crate::hardware::schema::booster::FeatureState::Off
                    };
                    Self::save_booster_settings(&booster)?;
                }
            }
            _ => println!("⚠️ [Governor] Field update NOT implemented: {}", field),
        }

        // Save back the updated control
        let base = Self::resolve_base_path();
        let json_data = serde_json::to_string_pretty(&control)?;
        std::fs::write(base.join("system_control.json"), json_data)?;
        
        Ok(())
    }

    /// Resolves the base AppData directory for Cluaiz configurations.
    pub fn resolve_base_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Cluaiz")
    }

    // ─── 🚀 SYSTEM CONTROL (HARDWARE TRUTH) ───

    pub fn load_system_control() -> anyhow::Result<SystemControl> {
        let path = Self::resolve_base_path().join("system_control.json");
        let data = std::fs::read_to_string(path)?;
        let control: SystemControl = serde_json::from_str(&data)?;
        Ok(control)
    }

    // ─── 🚀 BOOSTER CONTROL (USER SETTINGS) ───

    pub fn load_booster_settings() -> anyhow::Result<BoosterControl> {
        let path = Self::resolve_base_path().join("system_booster.json");
        if !path.exists() {
            return Ok(BoosterControl::default());
        }
        let data = std::fs::read_to_string(path)?;
        let control: BoosterControl = serde_json::from_str(&data)?;
        Ok(control)
    }

    pub fn save_booster_settings(control: &BoosterControl) -> anyhow::Result<()> {
        let base = Self::resolve_base_path();
        std::fs::create_dir_all(&base)?;

        // JSON for humans
        let json_data = serde_json::to_string_pretty(control)?;
        std::fs::write(base.join("system_booster.json"), json_data)?;

        // Binary for speed (Zero-copy)
        let bytes = rkyv::to_bytes::<_, 1024>(control)?;
        std::fs::write(base.join("system_booster.bin"), bytes.as_slice())?;

        Ok(())
    }
}
