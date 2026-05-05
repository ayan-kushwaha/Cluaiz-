
pub mod models_runner;
pub mod system_control_manager;

// 🧬 Cluaiz Profile Unification: Re-exporting from archer-shared/schema
pub use archer_shared::hardware::schema::profiles::{
    HardwareTruth, 
    MemorySubsystem, 
    StorageSubsystem, 
    CpuSubsystem,
    Accelerators
};
pub use archer_shared::hardware::schema::metrics::HardwareMetrics;

pub struct HardwareDetector;
impl Default for HardwareDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareDetector {
    pub fn new() -> Self { Self }

    /// 🏛️ Executes the physical hardware detection protocol.
    pub fn detect(&self) -> HardwareTruth {
        system_control_manager::detect_hardware()
    }
}

pub enum InferenceEngine {
    Cure,
    Llama,
    Candle,
}

pub enum InferenceEvent {
    Started,
    Progress(f32),
    Completed,
    Failed(String),
}
