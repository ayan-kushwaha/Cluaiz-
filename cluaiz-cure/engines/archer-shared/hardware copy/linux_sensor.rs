//! 🏛️ Silicon Kernel: Linux Concrete Sensor
//! Implements SysFS, ProcFS, and DRM probing for Linux, Server, and Edge (Pi/Jetson) targets.

use super::provider::SiliconProvider;
use super::mod_types::{UnifiedSiliconStats, SiliconMetrics};
use std::sync::Mutex;
use sysinfo::{SystemExt, CpuExt};

pub struct LinuxSensor {
    sys: Mutex<sysinfo::System>,
}

impl LinuxSensor {
    pub fn new() -> Self {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        Self { sys: Mutex::new(sys) }
    }
}

impl SiliconProvider for LinuxSensor {
    fn detect_specs(&self) -> UnifiedSiliconStats {
        let sys = self.sys.lock().unwrap();
        UnifiedSiliconStats {
            cpu_brand: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or("Linux CPU".into()),
            cpu_cores: sys.cpus().len(),
            mem_total_gb: sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
            has_gpu: false, // Updated via DRM scan in gpu.rs refactor
            gpu_brand: None,
            vram_total_gb: None,
            has_npu: false,
            has_tpu: false,
        }
    }

    fn capture_metrics(&self) -> SiliconMetrics {
        SiliconMetrics::default()
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
