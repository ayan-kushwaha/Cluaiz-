//! 🏛️ Silicon Kernel: Provider Trait
//! Defines the standard interface for hardware sensing across all platforms.
//! Enables static and dynamic dispatch for zero-latency telemetry.

use super::mod_types::{SovereignProfile, SiliconMetrics};

pub trait SiliconProvider: Send + Sync {
    /// Performs a full hardware detection audit.
    fn detect_specs(&self) -> SovereignProfile;
    
    /// Capture real-time metrics (VRAM, CPU, Thermal).
    fn capture_metrics(&self) -> SiliconMetrics;

    /// Returns usage percentage for every logical core.
    fn per_core_usage(&self) -> Vec<f32>;

    /// Identifies specialized accelerators (NPU/TPU).
    fn probe_accelerators(&self) -> (bool, Option<String>);
}
