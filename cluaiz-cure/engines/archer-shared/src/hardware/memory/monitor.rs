//! 🏛️ Silicon Kernel: Memory Monitor
//! Pure consumer of HAL memory telemetry. No OS logic allowed.

use super::super::hal::get_provider;
use super::super::schema::MemorySnapshot;

pub struct MemoryProbe;

impl MemoryProbe {
    pub fn new() -> Self {
        Self
    }

    /// Captures a granular snapshot of system memory state via HAL.
    pub fn capture_snapshot(&self) -> MemorySnapshot {
        get_provider().capture_memory_state()
    }

    pub fn get_system_pressure_percent(&self) -> u32 {
        let snapshot = self.capture_snapshot();
        if snapshot.system_total_gb > 0.0 {
            ((snapshot.system_used_gb / snapshot.system_total_gb) * 100.0) as u32
        } else {
            0
        }
    }
}
