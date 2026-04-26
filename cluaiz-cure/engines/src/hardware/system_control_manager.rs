use super::SovereignProfile;
use anyhow::Result;

pub fn detect_hardware() -> SovereignProfile {
    SovereignProfile::default()
}

pub fn has_config() -> bool {
    false
}

pub fn read_config() -> Result<SovereignProfile> {
    Ok(SovereignProfile::default())
}

pub fn save_config(_profile: &SovereignProfile) -> Result<()> {
    Ok(())
}

pub fn update_field(_field: &str, _value: &str) -> Result<()> {
    Ok(())
}
