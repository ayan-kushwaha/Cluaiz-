//! 🏛️ Silicon Kernel: GPU API Wrapper
//! Agnostic interface for VRAM and GPU Thermals. Dispatches to the active Platform Provider.

use super::get_provider;

pub struct GPUProbe;

impl GPUProbe {
    pub fn new() -> Self {
        Self
    }

    /// Returns GPU metrics (Pressure, Temperature) via Platform Provider.
    pub fn probe_metrics(&self) -> (u32, i32) {
        let metrics = get_provider().capture_metrics();
        (metrics.vram_pressure, metrics.cpu_thermal)
    }

    /// Performs a high-level GPU audit.
    pub fn probe_brand(&self) -> Option<String> {
        get_provider().detect_specs().gpu_brand
    }
}
