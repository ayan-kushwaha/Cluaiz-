//! 🏛️ Silicon Kernel: Android Concrete Sensor
//! Native edge handling for Android power systems and NNAPI.

use super::super::hal::provider::SiliconProvider;
use super::super::schema::{SovereignProfile, SiliconMetrics, MemorySnapshot, MobileTelemetry, NPUData};
use std::path::Path;

pub struct AndroidSensor;

impl AndroidSensor {
    pub fn new() -> Self {
        Self
    }
}

impl SiliconProvider for AndroidSensor {
    fn detect_specs(&self) -> SovereignProfile {
        let accelerators = crate::hardware::accelerators::probe::SovereignProbe::full_hardware_audit();
        
        let mut compute = crate::hardware::schema::ComputeProfile::default();
        if let Some(primary) = accelerators.first() {
            compute.primary_vendor = Some(primary.vendor.clone());
            compute.primary_driver = Some(primary.driver.clone());
            compute.vram_gb = primary.vram_gb;
            compute.has_gpu = true; 
            compute.has_npu = true; // Android assumes NNAPI/DSP availability
        }

        SovereignProfile {
            platform: "Android (Edge/Universal)".into(),
            cpu_brand: "ARM Mobile SoC".into(),
            cpu_cores: 8,
            base_clock_ghz: 2.0,
            total_threads: 8,
            mem_total_gb: 0.0, 
            accelerators,

            memory: crate::hardware::schema::MemoryProfile {
                total_ram_gb: 0.0,
                free_ram_gb: 0.0,
                bw_gbps: 0.0,
            },
            compute,
            ..Default::default()
        }
    }


    fn capture_metrics(&self) -> SiliconMetrics {
        SiliconMetrics::default()
    }

    fn per_core_usage(&self) -> Vec<f32> {
        vec![0.0; 8] // Safe telemetry mock for Edge until native JNI hook
    }

    fn probe_accelerators(&self) -> (bool, Option<String>) {
        let npu = self.probe_npu();
        (npu.active_state, Some(npu.brand))
    }

    fn capture_memory_state(&self) -> MemorySnapshot {
        MemorySnapshot {
            is_unified: true,
            ..Default::default()
        }
    }

    fn capture_mobile_state(&self) -> MobileTelemetry {
        let mut telemetry = MobileTelemetry::default();
        
        // Android Battery Check via SysFS (Panic-Free)
        if let Ok(capacity_str) = std::fs::read_to_string("/sys/class/power_supply/battery/capacity") {
            telemetry.battery_level = capacity_str.trim().parse().unwrap_or(100);
        } else {
            telemetry.battery_level = 100; // Safe Graceful degradation
        }

        if let Ok(status_str) = std::fs::read_to_string("/sys/class/power_supply/battery/status") {
            telemetry.is_charging = status_str.trim().eq_ignore_ascii_case("Charging");
        }
        
        telemetry
    }

    fn probe_npu(&self) -> NPUData {
        let mut npu = NPUData::default();
        
        // No unwraps, grace-based file access
        if Path::new("/dev/nnapi-0").exists() || Path::new("/dev/ion").exists() {
            npu.brand = "Android NNAPI (Verified)".into();
            npu.active_state = true;
        }
        npu
    }
}
