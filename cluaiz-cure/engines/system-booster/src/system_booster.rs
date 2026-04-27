//! 🚀 System Booster Orchestrator
//! Manages neural optimizations and synchronizes booster state via the HardwareGovernor.

use crate::{BoosterControl, HardwareGovernor};

pub struct SystemBooster;

impl SystemBooster {
    /// 📡 Ignite: Initializes the booster state and synchronizes with storage.
    pub fn ignite() -> anyhow::Result<BoosterControl> {
        let mut control = HardwareGovernor::load_booster_settings().unwrap_or_default();
        
        // 🧪 TODO: Probe OS and Hardware to determine optimal Auto states
        // For now, we sync the default/loaded state
        HardwareGovernor::save_booster_settings(&control)?;
        
        Ok(control)
    }

    /// 💾 Save: Persists the booster state (Proxied to Governor).
    pub fn save(control: &BoosterControl) -> anyhow::Result<()> {
        HardwareGovernor::save_booster_settings(control)
    }

    /// 📂 Load: Retrieves the existing booster state (Proxied to Governor).
    pub fn load() -> Option<BoosterControl> {
        HardwareGovernor::load_booster_settings().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_booster_ignition() {
        let result = SystemBooster::ignite();
        assert!(result.is_ok());
        
        let control = result.unwrap();
        // Default speculative decoding should be Off as per schema
        assert_eq!(control.speculative_decoding, crate::FeatureState::Off);
    }
}
