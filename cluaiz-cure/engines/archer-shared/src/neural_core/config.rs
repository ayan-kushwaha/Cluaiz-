//! Dynamic Neural Configuration: Resolves architecture-specific parameters 
//! by reconciling Structural DNA with actual Hardware Governor limits.

use crate::metadata::dna::StructuralDNA;
use crate::hardware::governor::HardwareGovernor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedNeuralParams {
    pub n_ctx: u32,
    pub batch_size: u32,
    pub sliding_window: Option<u32>,
    pub rope_freq_base: f32,
    pub threads: u32,
    pub gpu_layers: u32,
}

pub struct NeuralConfig;

impl NeuralConfig {
    /// Reconciles model DNA with real-time hardware constraints.
    /// This eliminates hardcoded values in engine implementations.
    pub fn resolve(dna: &StructuralDNA) -> ResolvedNeuralParams {
        let sys_config = HardwareGovernor::load_config();
        
        // 1. Dynamic Context Window (Sensed from GPU VRAM in GB)
        let n_ctx = sys_config.as_ref().ok()
            .and_then(|sc| sc.hardware_resources.as_ref())
            .map(|hr| {
                if hr.gpu.vram_total_gb >= 16.0 { 32768 } // Ultra-High Performance  
                else if hr.gpu.vram_total_gb >= 8.0 { 8192 } 
                else { 4096 }
            })
            .unwrap_or(2048);

        // 2. Dynamic Batch Size (Adaptive Policy)
        let batch_size = sys_config.as_ref().ok()
            .and_then(|sc| sc.runtime_engine.as_ref())
            .map(|re| if re.booster_flags.turbo_quant { 1024 } else { 512 })
            .unwrap_or(512);

        // 3. Sliding Window (Derived from DNA or hardware sense)
        let sliding_window = dna.dynamic_attributes.get("sliding_window")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|n| n as u32);

        // 4. RoPE Frequency (Stable defaults from architecture)
        let rope_freq_base = dna.dynamic_attributes.get("rope_freq_base")
            .and_then(|v| v.parse::<f64>().ok())
            .map(|f| f as f32)
            .unwrap_or(10000.0);


        // 5. Threading (Sensed from Real CPU Cores)
        let threads = sys_config.as_ref().ok()
            .and_then(|sc| sc.hardware_resources.as_ref())
            .map(|hr| hr.cpu.total_cores)
            .unwrap_or(8);

        // 6. GPU Offloading (Full offload if TurboQuant or RTX class detected)

        // 6. GPU Offloading (Authoritative DNA or Governor override)
        let gpu_layers = dna.layer_count.map(|l| l as u32).unwrap_or(0);

        ResolvedNeuralParams {
            n_ctx,
            batch_size,
            sliding_window,
            rope_freq_base,
            threads,
            gpu_layers,
        }
    }
}
