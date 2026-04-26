use serde::{Deserialize, Serialize};

pub mod models_runner;
pub mod system_control_manager;

// 🧬 Sovereign Profile Unification: Re-exporting from archer-shared/schema
pub use archer_shared::hardware::schema::profiles::{
    SovereignProfile, 
    MemoryProfile, 
    StorageProfile, 
    ComputeProfile
};
pub use archer_shared::hardware::schema::metrics::SiliconMetrics;

pub struct HardwareDetector;
impl HardwareDetector {
    pub fn new() -> Self { Self }
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
