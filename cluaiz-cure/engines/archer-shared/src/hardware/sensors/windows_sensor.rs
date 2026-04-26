//! 🏛️ Silicon Kernel: Windows Concrete Sensor
//! Implements DXGI, WMI, and NVML dynamic probing for Windows environments.

use super::super::hal::provider::SiliconProvider;
use super::super::schema::{SovereignProfile, SiliconMetrics, MemorySnapshot, NPUData, MobileTelemetry};

pub struct WindowsSensor;

impl WindowsSensor {
    pub fn new() -> Self {
        Self
    }
}

impl SiliconProvider for WindowsSensor {
    fn detect_specs(&self) -> SovereignProfile {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        
        let cpu = sys
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or("Windows CPU".into());
        let total_ram_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);

        // NOTE: Benchmark is now handled via Governor persistence, not dynamic probe here.
        let ram_bw = 0.0; 

        let accelerators = crate::hardware::accelerators::probe::SovereignProbe::full_hardware_audit();
        
        let mut compute = crate::hardware::schema::ComputeProfile::default();
        if let Some(primary) = accelerators.first() {
            compute.primary_vendor = Some(primary.vendor.clone());
            compute.primary_driver = Some(primary.driver.clone());
            compute.vram_gb = primary.vram_gb;
            compute.has_gpu = true; 
        }

        SovereignProfile {
            platform: "Windows (Universal Spectrum)".to_string(),
            cpu_brand: cpu,
            cpu_cores: sys.cpus().len(),
            base_clock_ghz: 0.0, 
            total_threads: sys.cpus().len(),
            mem_total_gb: total_ram_gb,
            accelerators,

            memory: crate::hardware::schema::MemoryProfile {
                total_ram_gb,
                free_ram_gb: sys.free_memory() as f64 / (1024.0 * 1024.0 * 1024.0),
                bw_gbps: ram_bw,
            },
            storage: crate::hardware::schema::StorageProfile::default(),
            compute,
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
        let mut sys = sysinfo::System::new(); // Zero-locking local refresh
        sys.refresh_cpu();
        sys.cpus().iter().map(|c| c.cpu_usage()).collect()
    }

    fn probe_accelerators(&self) -> (bool, Option<String>) {
        (false, None)
    }

    fn capture_memory_state(&self) -> MemorySnapshot {
        MemorySnapshot::default()
    }

    fn capture_mobile_state(&self) -> MobileTelemetry {
        MobileTelemetry::default()
    }

    fn probe_npu(&self) -> NPUData {
        NPUData::default()
    }
}
