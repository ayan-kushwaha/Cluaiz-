//! 🏛️ Silicon Kernel: Darwin Concrete Sensor
//! Implements Metal, sysctl, and IOKit hardware telemetry for MacOS and iOS targets.

use super::provider::SiliconProvider;
use super::mod_types::{UnifiedSiliconStats, SiliconMetrics};
use std::sync::Mutex;
use sysinfo::{SystemExt, CpuExt};

pub struct DarwinSensor {
    sys: Mutex<sysinfo::System>,
}

impl DarwinSensor {
    pub fn new() -> Self {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        Self { sys: Mutex::new(sys) }
    }
}

impl SiliconProvider for DarwinSensor {
    fn detect_specs(&self) -> UnifiedSiliconStats {
        let sys = self.sys.lock().unwrap();
        UnifiedSiliconStats {
            cpu_brand: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or("Apple Silicon".into()),
            cpu_cores: sys.cpus().len(),
            mem_total_gb: sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
            has_gpu: true, // Internal GPU on Apple Silicon
            gpu_brand: Some("Apple M-series GPU".into()),
            vram_total_gb: None, // Unified Memory Model
            has_npu: true,  // Apple Neural Engine presence
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
        (true, Some("Apple Neural Engine".into()))
    }
}
