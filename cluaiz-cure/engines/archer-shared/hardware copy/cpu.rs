//! 🏛️ Silicon Kernel: CPU API Wrapper
//! Agnostic interface for core/thread activity. Dispatches to the active Platform Provider.

use super::get_provider;

pub struct CPUProbe;

impl CPUProbe {
    pub fn new() -> Self {
        Self
    }

    /// Returns usage percentage for every logical core.
    /// Dispatched via Sovereign Platform Provider (0.000ms latency).
    pub fn per_core_usage(&self) -> Vec<f32> {
        get_provider().per_core_usage()
    }

    /// Performs a high-level CPU audit.
    pub fn probe_specs(&self) -> String {
        get_provider().detect_specs().cpu_brand
    }
}
