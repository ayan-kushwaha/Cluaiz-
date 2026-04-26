//! 🏛️ Silicon Kernel: Ghost Observer (Telemetry)
//! Actively monitors silicon health and manages global engine frequency states (Gears).

use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::super::hal::get_provider;
// use super::scheduler::GrandOrchestrator;

/// GlobalEngineState: Atomic synchronization for cross-thread hardware alignment.
pub static GLOBAL_ENGINE_GEAR: AtomicU32 = AtomicU32::new(1);

/// The Silicon Pulse: Global hardware state accessible by telemetry servers.
pub static SILICON_PULSE: Lazy<Arc<ObservableHardwareState>> =
    Lazy::new(|| Arc::new(ObservableHardwareState::default()));

pub fn get_pulse() -> Arc<ObservableHardwareState> {
    SILICON_PULSE.clone()
}

pub struct GhostObserver;

impl GhostObserver {
    /// Activates the background hardware telemetry loop.
    pub fn activate() {
        thread::spawn(|| {
            let provider = get_provider();
            let _profile = super::super::hal::detect_silicon();
            // let scheduler = GrandOrchestrator::new(profile);

            loop {
                // 1. Capture Real-Time Metrics (HAL dispatch)
                let metrics = provider.capture_metrics();
                
                // 2. Sample CPU Usage via thread-local fresh probe
                let cores = provider.per_core_usage();
                let cpu_p = if !cores.is_empty() {
                    cores.iter().sum::<f32>() / cores.len() as f32
                } else { 0.0 };
                
                let pulse = SILICON_PULSE.clone();
                pulse.cpu_usage_pct.store(cpu_p as u32, Ordering::Relaxed);
                pulse.vram_pressure_pct.store(metrics.vram_pressure, Ordering::Relaxed);
                pulse.temp_celsius.store(metrics.cpu_thermal as u32, Ordering::Relaxed);

                // 3. Resolve Frequency Gear (Deep Adaptive Logic)
                let target_gear = if metrics.cpu_thermal > 90 || metrics.vram_pressure > 95 {
                    4 // Emergency Throttling
                } else if metrics.cpu_thermal > 75 || metrics.vram_pressure > 80 {
                    3 // Performance Ceiling
                } else {
                    1 // Max Frequency
                };

                // 4. Adaptive Polling
                let poll_ms = if target_gear >= 3 { 50 } else { 1000 };
                GLOBAL_ENGINE_GEAR.store(target_gear, Ordering::Relaxed);

                thread::sleep(Duration::from_millis(poll_ms));
            }
        });
    }

    pub fn get_current_gear() -> u32 {
        GLOBAL_ENGINE_GEAR.load(Ordering::Relaxed)
    }
}

pub enum EngineGear {
    Pulse,
    Balanced,
    Survival,
    Emergency,
}

pub struct ObservableHardwareState {
    pub vram_pressure_pct: AtomicU32,
    pub relay_latency_ms: AtomicU32,
    pub kv_cache_footprint_mb: AtomicU32,
    pub storage_throughput_mbps: AtomicU32,
    pub per_core_usage: Vec<AtomicU32>,
    pub turbo_quant_enabled: AtomicBool,
    pub cpu_usage_pct: AtomicU32,
    pub ram_usage_mb: AtomicU32,
    pub temp_celsius: AtomicU32,
    pub current_tps: AtomicU32,
}

impl Default for ObservableHardwareState {
    fn default() -> Self {
        Self {
            vram_pressure_pct: AtomicU32::new(0),
            relay_latency_ms: AtomicU32::new(0),
            kv_cache_footprint_mb: AtomicU32::new(0),
            storage_throughput_mbps: AtomicU32::new(0),
            per_core_usage: (0..16).map(|_| AtomicU32::new(0)).collect(),
            turbo_quant_enabled: AtomicBool::new(false),
            cpu_usage_pct: AtomicU32::new(0),
            ram_usage_mb: AtomicU32::new(0),
            temp_celsius: AtomicU32::new(0),
            current_tps: AtomicU32::new(0),
        }
    }
}

impl ObservableHardwareState {
    pub fn resolve_gear(&self) -> EngineGear {
        match GhostObserver::get_current_gear() {
            1 => EngineGear::Pulse,
            2 => EngineGear::Balanced,
            3 => EngineGear::Survival,
            _ => EngineGear::Emergency,
        }
    }
}
