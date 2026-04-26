use archer_shared::hardware::telemetry::{ObservableHardwareState, EngineGear};
use std::sync::atomic::Ordering;

#[test]
fn test_universal_gear_resolution() {
    let pulse = ObservableHardwareState::new();

    // Scenario 1: Nominal Power (Gear 1)
    pulse.vram_pressure_pct.store(45, Ordering::Relaxed);
    pulse.current_temperature.store(50, Ordering::Relaxed);
    assert_eq!(pulse.resolve_gear(), EngineGear::Pulse);

    // Scenario 2: Balanced Load (Gear 2)
    pulse.vram_pressure_pct.store(88, Ordering::Relaxed);
    assert_eq!(pulse.resolve_gear(), EngineGear::Balanced);
    assert_eq!(EngineGear::Balanced.drop_ratio(), 0.125); // 1/8th

    // Scenario 3: High Pressure (Gear 3 - Survival)
    pulse.vram_pressure_pct.store(95, Ordering::Relaxed);
    assert_eq!(pulse.resolve_gear(), EngineGear::Survival);
    assert_eq!(EngineGear::Survival.drop_ratio(), 0.25); // 1/4th

    // Scenario 4: Critical Limit (Gear 4 - Emergency)
    pulse.current_temperature.store(92, Ordering::Relaxed);
    assert_eq!(pulse.resolve_gear(), EngineGear::Emergency);
    assert_eq!(EngineGear::Emergency.drop_ratio(), 0.50); // 1/2th
}

#[test]
fn test_ratio_based_drop_math() {
    let max_ctx = 32768; // 32k context
    
    let gear_balanced = EngineGear::Balanced;
    let drop_balanced = (max_ctx as f64 * gear_balanced.drop_ratio()) as usize;
    assert_eq!(drop_balanced, 4096); // 1/8th of 32k

    let gear_survival = EngineGear::Survival;
    let drop_survival = (max_ctx as f64 * gear_survival.drop_ratio()) as usize;
    assert_eq!(drop_survival, 8192); // 1/4th of 32k
}
