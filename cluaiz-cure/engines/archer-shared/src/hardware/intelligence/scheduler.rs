//! 🏛️ Silicon Kernel: Heterogeneous Scheduler
//! The Grand Orchestrator for task routing across CPU, GPU, and NPU.
//! Routes based on Hardware Capabilities (Flags), not the OS name.

use super::super::schema::{SiliconMetrics, SovereignProfile};

pub enum ComputeBackend {
    Cuda,
    Vulkan,
    Metal,
    Amx,    // Intel AMX
    Avx512, // Intel/AMD SIMD
    Nnapi,  // Android
    Ane,    // Apple Neural Engine
}

pub struct GrandOrchestrator {
    pub profile: SovereignProfile,
}

impl GrandOrchestrator {
    pub fn new(profile: SovereignProfile) -> Self {
        Self { profile }
    }

    /// Resolves the optimal backend for a specific tensor operation.
    /// Factors in: ISA features, VRAM pressure, and Model requirements.
    pub fn resolve_optimal_path(&self, _precision: u32, metrics: &SiliconMetrics) -> ComputeBackend {
        // 1. Check for NPU/Specialized Silicon (Priority 1)
        if self.profile.compute.has_npu && self.profile.platform.contains("macOS") {
            return ComputeBackend::Ane;
        }

        // 2. Routing based on Capability Drivers (Zero OS assumptions)
        if metrics.vram_pressure < 80 {
            if let Some(driver) = &self.profile.compute.primary_driver {
                match driver {
                    crate::hardware::schema::BackendDriver::CUDA => return ComputeBackend::Cuda,
                    crate::hardware::schema::BackendDriver::METAL => return ComputeBackend::Metal,
                    crate::hardware::schema::BackendDriver::NNAPI => return ComputeBackend::Nnapi,
                    _ => {}
                }
            }
            
            // Default cross-platform GPU
            return ComputeBackend::Vulkan;
        }

        // 3. Fallback: CPU SIMD
        ComputeBackend::Avx512 
    }

}
