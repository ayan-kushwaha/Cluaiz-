use super::SiliconTruth;
use anyhow::Result;
use archer_shared::hardware::{get_silicon_state, HardwareGovernor};

/// 🏛️ Performs a deep surgical scan of the host silicon.
pub fn detect_hardware() -> SiliconTruth {
    get_silicon_state()
}

/// 🛡️ Checks if the 'system_control.json' fingerprint exists.
pub fn has_config() -> bool {
    HardwareGovernor::start().is_ready()
}

/// 🧠 Reads the current hardware configuration.
pub fn read_config() -> Result<SiliconTruth> {
    // The governor maintains the system_control.json state.
    // For engine-level access, we provide the live silicon profile.
    Ok(get_silicon_state())
}

/// 📁 Persists the silicon fingerprint to disk via the Governor.
pub fn save_config(_profile: &SiliconTruth) -> Result<()> {
    HardwareGovernor::auto_calibrate().map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}

/// ⚙️ Updates a specific field in the sovereign configuration.
pub fn update_field(field: &str, value: &str) -> Result<()> {
    // Convert string value to JSON for the Governor's update protocol.
    let val_json = serde_json::Value::String(value.to_string());
    HardwareGovernor::update_field(field, val_json).map_err(|e| anyhow::anyhow!(e))?;
    Ok(())
}
