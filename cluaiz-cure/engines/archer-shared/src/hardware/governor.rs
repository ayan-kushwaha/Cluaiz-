use crate::hardware::schema::booster::BoosterControl;
use crate::hardware::schema::profiles::SystemControl;
use crate::hardware::system_control::HardwareOrchestrator;
use std::path::PathBuf;

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
        HardwareOrchestrator::start()?;
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
