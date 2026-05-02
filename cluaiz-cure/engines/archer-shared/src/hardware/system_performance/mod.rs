pub mod pulse_schema;
pub mod gears;
pub mod drivers;
pub mod thermal_guard;

use std::sync::{Arc, RwLock, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
pub use pulse_schema::*;
pub use gears::*;
use drivers::HardwareDriver;
use thermal_guard::ThermalGuard;

pub struct SystemPerformanceLive {
    gpu_drivers: Vec<Box<dyn HardwareDriver>>,
    cpu_driver: Box<dyn HardwareDriver>,
    thermal_guard: ThermalGuard,
    pub state: Arc<ObservableHardwareState>,
}

impl SystemPerformanceLive {
    pub fn new() -> Self {
        // 🔗 Sovereign Linkage: Get truth from Orchestrator
        let control = crate::hardware::system_control::HardwareOrchestrator::probe();
        let mut gpu_drivers: Vec<Box<dyn HardwareDriver>> = Vec::new();

        // 🚀 Dynamic GPU/NPU/TPU Driver Provisioning
        for gpu in &control.silicon_truth.accelerators.gpus {
            if gpu.vendor.contains("NVIDIA") {
                if let Ok(d) = drivers::nvidia::NvidiaDriver::init() { gpu_drivers.push(Box::new(d)); }
            } else if gpu.vendor.contains("AMD") {
                if let Ok(d) = drivers::amd_rocm::AmdRocmDriver::init() { gpu_drivers.push(Box::new(d)); }
            } else if gpu.vendor.contains("Apple") {
                if let Ok(d) = drivers::apple_metal::AppleMetalDriver::init() { gpu_drivers.push(Box::new(d)); }
            } else if gpu.vendor.contains("Intel") {
                if let Ok(d) = drivers::openvino::OpenVinoDriver::init() { gpu_drivers.push(Box::new(d)); }
            }
        }

        for npu in &control.silicon_truth.accelerators.npus {
            if npu.model.contains("Qualcomm") || npu.model.contains("QNN") {
                if let Ok(d) = drivers::qnn::QnnDriver::init() { gpu_drivers.push(Box::new(d)); }
            }
        }

        for _tpu in &control.silicon_truth.accelerators.tpus {
            if let Ok(d) = drivers::tpu::TpuDriver::init() { gpu_drivers.push(Box::new(d)); }
        }

        let cpu_driver = Box::new(drivers::cpu_generic::CpuGenericDriver::init()) as Box<dyn HardwareDriver>;

        Self {
            gpu_drivers,
            cpu_driver,
            thermal_guard: ThermalGuard::new(85.0, 70.0),
            state: Arc::new(ObservableHardwareState {
                pulse: Arc::new(RwLock::new(LivePulse::default())),
                turbo_quant_enabled: std::sync::atomic::AtomicBool::new(false),
            }),
        }
    }

    pub fn tick(&mut self) {
        let mut pulse = LivePulse::default();
        pulse.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // 1. CPU Metrics
        if let Ok(temp) = self.cpu_driver.temperature_c() { pulse.cpu.temperature_c = temp; }
        if let Ok(util) = self.cpu_driver.utilization_pct() { pulse.cpu.utilization_pct = util; }
        if let Ok(clock) = self.cpu_driver.clock_mhz() { pulse.cpu.clock_ghz = clock as f32 / 1000.0; }

        // 2. RAM Metrics
        if let Ok(used) = self.cpu_driver.vram_used_mb() { pulse.ram.used_gb = used as f64 / 1024.0; }
        if let Ok(total) = self.cpu_driver.vram_total_mb() { 
            if total > 100 { // At least 100MB to be valid
                pulse.ram.utilization_pct = (pulse.ram.used_gb / (total as f64 / 1024.0)) as f32 * 100.0; 
            }
        }

        // 3. Accelerators (GPU/NPU/TPU)
        let mut total_vram_used = 0.0;
        let mut total_vram_max = 0.0;

        for drv in &self.gpu_drivers {
            let mut gpu_p = GpuPulse::default();
            gpu_p.name = drv.name().to_string();
            
            if let Ok(temp) = drv.temperature_c() { gpu_p.temperature_c = temp; }
            if let Ok(util) = drv.utilization_pct() { gpu_p.utilization_pct = util; }
            if let Ok(pwr) = drv.power_draw_watts() { gpu_p.power_draw_watts = pwr; }
            
            if let Ok(used) = drv.vram_used_mb() { 
                gpu_p.vram_used_gb = used as f64 / 1024.0;
                total_vram_used += gpu_p.vram_used_gb;
            }
            if let Ok(total) = drv.vram_total_mb() {
                gpu_p.vram_total_gb = total as f64 / 1024.0;
                total_vram_max += gpu_p.vram_total_gb;
            }
            
            pulse.gpus.push(gpu_p);

            // Apply Thermal Guard (Mission 13)
            let _ = self.thermal_guard.tick(drv.as_ref());
        }

        // 4. Global Aggregates (Production Safety)
        pulse.vram_used_gb = total_vram_used;
        pulse.vram_total_gb = total_vram_max;
        if pulse.vram_total_gb > 0.1 {
            pulse.vram_pressure_pct = (pulse.vram_used_gb / pulse.vram_total_gb * 100.0) as u32;
        }

        // Update Global State
        if let Ok(mut lock) = self.state.pulse.write() {
            *lock = pulse;
        }
    }

    pub fn apply_booster_profile(&self, control: &super::schema::booster::BoosterControl) -> Result<()> {
        let profile = &control.silicon;
        tracing::info!("🚀 [Kernel] Applying Silicon Booster Profile: {}", profile.performance_profile);

        for drv in &self.gpu_drivers {
            // Apply Power Limits if specified
            if let Some(limit) = profile.max_power_limit_watts {
                let _ = drv.set_power_limit_watts(limit);
            }

            // Apply Clock Boost if specified
            if let Some(clock) = profile.max_clock_mhz {
                let _ = drv.set_clock_mhz(clock);
            }

            // Profile specific logic
            if profile.performance_profile == "Eco" {
                let _ = drv.set_power_limit_watts(100); // Force low power
            }
        }
        Ok(())
    }

    pub fn start_background_stream() -> Arc<ObservableHardwareState> {
        let mut live = Self::new();
        let state_ref = live.state.clone();
        let _ = std::thread::Builder::new()
            .name("cluaiz-ghost-observer".into())
            .spawn(move || {
                loop {
                    live.tick();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            });
        state_ref
    }
}

static GLOBAL_TELEMETRY: OnceLock<Arc<ObservableHardwareState>> = OnceLock::new();

pub fn get_pulse() -> Arc<ObservableHardwareState> {
    GLOBAL_TELEMETRY.get_or_init(|| SystemPerformanceLive::start_background_stream()).clone()
}
