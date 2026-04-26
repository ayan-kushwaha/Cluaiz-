use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SovereignProfile {
    pub platform: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub mem_total_gb: f64,
    pub has_gpu: bool,
    pub gpu_brand: Option<String>,
    pub vram_total_gb: Option<f64>,
    pub has_npu: bool,
    pub has_tpu: bool,

    // ── Nested Engine Profiles (Architectural Alignment) ──
    pub memory: MemoryProfile,
    pub storage: StorageProfile,
    pub compute: ComputeProfile,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub total_ram_gb: f64,
    pub free_ram_gb: f64,
    pub bw_gbps: f64, // True Physical Bandwidth Measured by Micro-benchmark
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StorageProfile {
    pub sequential_read_mbps: f64,
    pub is_hdd: bool,
    pub is_nvme: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputeProfile {
    pub vram_gb: f64,
    pub has_gpu: bool,
    pub has_cuda: bool,
    pub has_avx512: bool,
    pub bw_gbps: f64, // Real GPU Bandwidth
}

#[derive(Debug, Clone, Default)]
pub struct SiliconMetrics {
    pub vram_pressure: u32,
    pub cpu_thermal: i32,
    pub core_load_avg: f32,
}
