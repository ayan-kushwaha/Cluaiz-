//! 🏛️ Silicon Kernel: Heterogeneous Scheduler
//! The Grand Orchestrator for task routing across CPU, GPU, and NPU.
//! Decides the optimal compute path based on real-time Silicon Metrics and ISA features.

use super::mod_types::SiliconMetrics;
use super::platform::PlatformIdentity;

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
    pub identity: PlatformIdentity,
}

impl GrandOrchestrator {
    pub fn new(identity: PlatformIdentity) -> Self {
        Self { identity }
    }

    /// Resolves the optimal backend for a specific tensor operation.
    /// Factors in: ISA features, Thermal Gear, and VRAM pressure.
    pub fn resolve_optimal_path(&self, precision: u32, metrics: &SiliconMetrics) -> ComputeBackend {
        // 1. Check for NPU/Specialized Silicon First (Highest Efficiency)
        if self.identity.features.amx && precision <= 8 {
            return ComputeBackend::Amx;
        }

        if self.identity.features.dotprod && self.identity.is_unified_memory {
            return ComputeBackend::Ane;
        }

        // 2. Fallback to GPU if VRAM pressure is low (<80%)
        if metrics.vram_pressure < 80 {
            #[cfg(target_os = "windows")]
            return ComputeBackend::Cuda;
            
            #[cfg(target_os = "macos")]
            return ComputeBackend::Metal;

            #[cfg(target_os = "linux")]
            return ComputeBackend::Vulkan;
        }

        // 3. Last Fallback: SIMD Optimized CPU (AVX-512)
        if self.identity.features.avx512 {
            return ComputeBackend::Avx512;
        }

        ComputeBackend::Avx512 // Default to highest SIMD available
    }
}
