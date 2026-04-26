// Real-time telemetry structures 

#[derive(Debug, Clone, Default)]
pub struct SiliconMetrics {
    pub vram_pressure: u32,
    pub cpu_thermal: i32,
    pub core_load_avg: f32, // Passed natively via atomic queues by GhostObserver
}

#[derive(Debug, Clone, Default)]
pub struct MemorySnapshot {
    pub system_total_gb: f64,
    pub system_used_gb: f64,
    pub swap_total_gb: f64,
    pub swap_used_gb: f64,
    pub is_unified: bool,
    pub shared_gpu_reserved_mb: u64,
    pub block_size_kb: u32,
    pub total_pages: u64,
    pub free_pages: u64,
}

#[derive(Debug, Clone, Default)]
pub struct MobileTelemetry {
    pub battery_level: u32,
    pub is_charging: bool,
    pub thermal_state: String,
    pub low_power_mode: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NPUData {
    pub brand: String,
    pub active_state: bool,
    pub pressure_percent: u32,
}

#[derive(Debug, Clone, Default)]
pub struct TPUData {
    pub brand: String,
    pub is_cloud_tpu: bool,
    pub activity_detected: bool,
}
