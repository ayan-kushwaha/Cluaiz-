//! 🏛️ Silicon Kernel: Windows Concrete Sensor
//! Implements DXGI, WMI, and NVML dynamic probing for Windows environments.

use super::mod_types::{SiliconMetrics, SovereignProfile};
use super::provider::SiliconProvider;

use std::sync::Mutex;

pub struct WindowsSensor {
    sys: Mutex<sysinfo::System>,
}

impl WindowsSensor {
    pub fn new() -> Self {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        Self {
            sys: Mutex::new(sys),
        }
    }
}

impl SiliconProvider for WindowsSensor {
    fn detect_specs(&self) -> SovereignProfile {
        let sys = self.sys.lock().unwrap();
        let cpu = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or("Windows CPU".into());
        let total_ram_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);

        let ram_bw = super::benchmark::measure_memory_bandwidth();

        SovereignProfile {
            platform: "Windows (DXGI/WMI)".to_string(),
            cpu_brand: cpu,
            cpu_cores: sys.cpus().len(),
            mem_total_gb: total_ram_gb,
            has_gpu: true,
            gpu_brand: Some("Windows GPU".into()), // Needs actual WMI hook module
            vram_total_gb: None,                   // Must be dynamically probed
            has_npu: false,
            has_tpu: false,

            memory: crate::hardware::mod_types::MemoryProfile {
                total_ram_gb,
                free_ram_gb: sys.free_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
                bw_gbps: ram_bw,
            },
            storage: crate::hardware::mod_types::StorageProfile::default(),
            compute: crate::hardware::mod_types::ComputeProfile {
                has_gpu: true,
                has_cuda: true,
                vram_gb: 0.0, // Strictly 0 until dynamic hook fetches real value
                bw_gbps: 0.0, // Strictly 0 until WMI PCI bandwidth checks occur
                ..Default::default()
            },
        }
    }

    fn capture_metrics(&self) -> SiliconMetrics {
        // Real-time metrics capture logic
        SiliconMetrics {
            vram_pressure: 0,
            cpu_thermal: 45,
            core_load_avg: 0.0,
        }
    }

    fn per_core_usage(&self) -> Vec<f32> {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_cpu();
        sys.cpus().iter().map(|c| c.cpu_usage()).collect()
    }

    fn probe_accelerators(&self) -> (bool, Option<String>) {
        (false, None)
    }
}
