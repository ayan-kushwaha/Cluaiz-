//! 🏛️ Silicon Kernel: CPU API Wrapper
//! Agnostic interface for core/thread activity. Dispatches to the active Platform Provider.

use super::super::hal::get_provider;

pub struct CPUProbe;

impl CPUProbe {
    pub fn new() -> Self {
        Self
    }

    /// Returns usage percentage for every logical core via HAL.
    pub fn per_core_usage(&self) -> Vec<f32> {
        get_provider().per_core_usage()
    }

    /// Performs a high-level CPU audit via HAL.
    pub fn probe_specs(&self) -> String {
        get_provider().detect_specs().cpu_brand
    }
}
