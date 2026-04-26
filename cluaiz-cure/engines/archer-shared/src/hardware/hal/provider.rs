//! 🏛️ Silicon Kernel: Provider Trait
//! Defines the standard interface for hardware sensing across all platforms.
//! Enables static and dynamic dispatch for zero-latency telemetry.

use super::super::schema::{SovereignProfile, SiliconMetrics, MemorySnapshot, NPUData, MobileTelemetry};

pub trait SiliconProvider: Send + Sync {
    /// Performs a full hardware detection audit.
    fn detect_specs(&self) -> SovereignProfile;
    
    /// Capture real-time metrics (VRAM, CPU, Thermal).
    fn capture_metrics(&self) -> SiliconMetrics;

    /// Returns usage percentage for every logical core.
    fn per_core_usage(&self) -> Vec<f32>;

    /// Identifies specialized accelerators (NPU/TPU).
    fn probe_accelerators(&self) -> (bool, Option<String>);

    /// Captures the OS-specific advanced memory map (Unified chunks, swap).
    fn capture_memory_state(&self) -> MemorySnapshot;

    /// Captures the edge/mobile hardware state (Battery, low-power interrupts).
    fn capture_mobile_state(&self) -> MobileTelemetry;

    /// Probes exactly NPU capabilities natively.
    fn probe_npu(&self) -> NPUData;
}
