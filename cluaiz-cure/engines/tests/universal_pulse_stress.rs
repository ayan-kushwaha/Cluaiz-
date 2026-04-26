#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU8, Ordering};
    use std::sync::Arc;

    #[derive(Debug, PartialEq)]
    enum EngineComputeGear {
        Pulse,
        Balanced,
        Survival,
        Emergency,
    }

    impl EngineComputeGear {
        fn drop_ratio(&self) -> f64 {
            match self {
                EngineComputeGear::Pulse => 0.0,
                EngineComputeGear::Balanced => 0.125,
                EngineComputeGear::Survival => 0.25,
                EngineComputeGear::Emergency => 0.50,
            }
        }
    }

    struct LocalEngineStressMock {
        vram_pressure: AtomicU8,
        temperature: AtomicU8,
    }

    impl LocalEngineStressMock {
        fn new() -> Self {
            Self {
                vram_pressure: AtomicU8::new(0),
                temperature: AtomicU8::new(0),
            }
        }

        fn resolve_gear(&self) -> EngineComputeGear {
            let v = self.vram_pressure.load(Ordering::Relaxed);
            let t = self.temperature.load(Ordering::Relaxed);

            if t > 90 || v > 97 { return EngineComputeGear::Emergency; }
            if t > 80 || v > 92 { return EngineComputeGear::Survival; }
            if t > 70 || v > 85 { return EngineComputeGear::Balanced; }
            EngineComputeGear::Pulse
        }
    }

    #[test]
    fn test_local_gear_resolution() {
        let mock = Arc::new(LocalEngineStressMock::new());

        // Scenario 1: Nominal Power (Gear 1)
        mock.vram_pressure.store(45, Ordering::Relaxed);
        mock.temperature.store(50, Ordering::Relaxed);
        assert_eq!(mock.resolve_gear(), EngineComputeGear::Pulse);

        // Scenario 2: Balanced Load (Gear 2)
        mock.vram_pressure.store(88, Ordering::Relaxed);
        assert_eq!(mock.resolve_gear(), EngineComputeGear::Balanced);
        assert_eq!(EngineComputeGear::Balanced.drop_ratio(), 0.125);

        // Scenario 3: High Pressure (Gear 3 - Survival)
        mock.vram_pressure.store(95, Ordering::Relaxed);
        assert_eq!(mock.resolve_gear(), EngineComputeGear::Survival);
        assert_eq!(EngineComputeGear::Survival.drop_ratio(), 0.25);

        // Scenario 4: Critical Limit (Gear 4 - Emergency)
        mock.temperature.store(92, Ordering::Relaxed);
        assert_eq!(mock.resolve_gear(), EngineComputeGear::Emergency);
        assert_eq!(EngineComputeGear::Emergency.drop_ratio(), 0.50);
    }

    #[test]
    fn test_ratio_based_drop_logic() {
        let max_ctx = 32768; // 32k context
        
        let gear_balanced = EngineComputeGear::Balanced;
        let drop_balanced = (max_ctx as f64 * gear_balanced.drop_ratio()) as usize;
        assert_eq!(drop_balanced, 4096); 

        let gear_survival = EngineComputeGear::Survival;
        let drop_survival = (max_ctx as f64 * gear_survival.drop_ratio()) as usize;
        assert_eq!(drop_survival, 8192); 
    }
}
