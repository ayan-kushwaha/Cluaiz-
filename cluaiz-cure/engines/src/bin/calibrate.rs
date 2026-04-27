//! 🔬 Archer Calibration Tool
//! Surgical probe of local silicon to generate 'system_control.json'.

use archer_shared::HardwareGovernor;

fn main() -> anyhow::Result<()> {
    println!("⚔️  [CALIBRATE] Starting deep silicon probe...");
    
    match HardwareGovernor::auto_calibrate() {
        Ok(_) => {
            println!("✅ [CALIBRATE] Sovereign hardware fingerprint saved to AppData/Cluaiz/system_control.json");
            Ok(())
        },
        Err(e) => {
            eprintln!("❌ [CALIBRATE] Probe FAILED: {}", e);
            Err(e)
        }
    }
}
