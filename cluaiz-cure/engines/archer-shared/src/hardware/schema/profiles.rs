use serde::{Deserialize, Serialize};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct SovereignProfile {

    pub platform: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub base_clock_ghz: f64,
    pub total_threads: usize,
    pub mem_total_gb: f64,
    pub accelerators: Vec<AcceleratorProfile>,

    // ── Nested Engine Profiles ──
    pub memory: MemoryProfile,
    pub storage: StorageProfile,
    pub compute: ComputeProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub enum HardwareVendor {
    NVIDIA,
    AMD,
    Intel,
    Apple,
    Qualcomm,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub enum BackendDriver {
    CUDA,
    METAL,
    ROCM,
    SYCL,
    OpenVINO,
    Vulkan,
    DirectML,
    OpenCL,
    NNAPI,
    Hexagon,
    QNN,

    WebGPU,
    CPU,
}

#[derive(Debug, Clone, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct AcceleratorProfile {
    pub vendor: HardwareVendor,
    pub driver: BackendDriver,
    pub vram_gb: f64,
    pub core_count: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct ComputeProfile {
    pub primary_vendor: Option<HardwareVendor>,
    pub primary_driver: Option<BackendDriver>,
    pub vram_gb: f64,
    pub has_gpu: bool,
    pub has_npu: bool,
    pub has_tpu: bool,
    pub bw_gbps: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct MemoryProfile {
    pub total_ram_gb: f64,
    pub free_ram_gb: f64,
    pub bw_gbps: f64, // Physical Bandwidth Measured by Spec
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Archive, RkyvSerialize, RkyvDeserialize)]
#[archive(check_bytes)]
pub struct StorageProfile {
    pub sequential_read_mbps: f64,
    pub is_hdd: bool,
    pub is_nvme: bool,
}


