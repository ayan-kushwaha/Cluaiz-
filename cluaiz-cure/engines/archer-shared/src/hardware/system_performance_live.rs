//! 🟢 Tier 1: Sovereign Live Telemetry (Ghost Observer)
//! Single-file dynamic hardware pulse streaming.
//! Streams real-time temperatures, utilization, and power draw for CPU, GPU, NPU, TPU.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LivePulse {
    pub timestamp: u64,
    pub cpu: CpuPulse,
    pub ram: RamPulse,
    pub gpu: GpuPulse,
    pub npus: Vec<AcceleratorPulse>,
    pub tpus: Vec<AcceleratorPulse>,

    // 🏛️ Advanced Engine Metrics
    pub vram_pressure_pct: u32,
    pub vram_used_gb: f64,
    pub vram_total_gb: f64,
    pub relay_latency_ms: u64,
    pub kv_cache_footprint_mb: u64,
    pub storage_throughput_mbps: u64,
    pub per_core_usage: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CpuPulse {
    pub utilization_pct: f32,
    pub temperature_c: f32,
    pub clock_ghz: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RamPulse {
    pub used_gb: f64,
    pub utilization_pct: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GpuPulse {
    pub utilization_pct: f32,
    pub temperature_c: f32,
    pub vram_used_gb: f64,
    pub power_draw_watts: f32,
    pub fan_speed_pct: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AcceleratorPulse {
    pub utilization_pct: f32,
    pub temperature_c: f32,
    pub power_draw_watts: f32,
}

// ── ORCHESTRATOR GEARS ──

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EngineGear {
    Pulse,     // Idle/Low
    Balanced,  // Normal
    Survival,  // High pressure
    Emergency, // Critical
}

pub struct ObservableHardwareState {
    pub pulse: Arc<RwLock<LivePulse>>,
    pub turbo_quant_enabled: AtomicBool,
}

impl ObservableHardwareState {
    pub fn resolve_gear(&self) -> EngineGear {
        let p = self.pulse.read().unwrap_or_else(|e| e.into_inner());
        if p.cpu.utilization_pct > 95.0 || p.gpu.utilization_pct > 95.0 {
            EngineGear::Emergency
        } else if p.cpu.utilization_pct > 80.0 || p.gpu.utilization_pct > 80.0 {
            EngineGear::Survival
        } else if p.cpu.utilization_pct > 40.0 {
            EngineGear::Balanced
        } else {
            EngineGear::Pulse
        }
    }
}

pub struct SystemPerformanceLive {
    sys: System,
    nvml: Option<nvml_wrapper::Nvml>,
    pub state: Arc<ObservableHardwareState>,
}

impl SystemPerformanceLive {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let nvml = nvml_wrapper::Nvml::init().ok();
        Self {
            sys,
            nvml,
            state: Arc::new(ObservableHardwareState {
                pulse: Arc::new(RwLock::new(LivePulse::default())),
                turbo_quant_enabled: AtomicBool::new(false),
            }),
        }
    }

    /// Single atomic tick to read live hardware states (Zero latency goal)
    pub fn tick(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        self.sys.refresh_cpu_frequency();
        // 🧪 Wait a tiny bit for CPU delta
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.sys.refresh_cpu_usage();

        let mut pulse = LivePulse::default();
        pulse.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // --- 1. LIVE CPU METRICS ---
        let cpus = self.sys.cpus();
        let cpu_usage = if !cpus.is_empty() {
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32
        } else {
            0.0
        };
        
        let cpu_ghz = if !cpus.is_empty() {
            cpus.iter().map(|c| c.frequency()).sum::<u64>() as f32 / (cpus.len() as f32 * 1000.0)
        } else {
            0.0
        };

        let mut cpu_temp = 0.0;
        let components = sysinfo::Components::new_with_refreshed_list();
        for comp in &components {
            let label = comp.label().to_lowercase();
            // Broader search for CPU sensors
            if label.contains("cpu") || label.contains("core") || label.contains("package") || label.contains("k10temp") || label.contains("tctl") {
                if comp.temperature() > cpu_temp {
                    cpu_temp = comp.temperature();
                }
            }
        }

        pulse.cpu = CpuPulse {
            utilization_pct: cpu_usage,
            temperature_c: cpu_temp,
            clock_ghz: cpu_ghz,
        };

        // --- 2. LIVE RAM METRICS ---
        let total_ram = self.sys.total_memory() as f64 / 1_073_741_824.0;
        let used_ram = self.sys.used_memory() as f64 / 1_073_741_824.0;
        pulse.ram = RamPulse {
            used_gb: used_ram,
            utilization_pct: if total_ram > 0.0 {
                (used_ram / total_ram) as f32 * 100.0
            } else {
                0.0
            },
        };

        // --- 3. LIVE GPU METRICS ---
        if let Some(ref nvml) = self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                let util = device
                    .utilization_rates()
                    .map(|u| u.gpu as f32)
                    .unwrap_or(0.0);
                let temp = device
                    .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                    .unwrap_or(0) as f32;
                let power = device
                    .power_usage()
                    .map(|p| p as f32 / 1000.0)
                    .unwrap_or(0.0);
                let fan = device.fan_speed(0).unwrap_or(0);

                if let Ok(info) = device.memory_info() {
                    let used_gb = info.used as f64 / 1_073_741_824.0;
                    let total_gb = info.total as f64 / 1_073_741_824.0;
                    
                    pulse.vram_used_gb = used_gb;
                    pulse.vram_total_gb = total_gb;
                    pulse.vram_pressure_pct = ((used_gb / total_gb) * 100.0) as u32;

                    pulse.gpu = GpuPulse {
                        utilization_pct: util,
                        temperature_c: temp,
                        vram_used_gb: used_gb,
                        power_draw_watts: power,
                        fan_speed_pct: fan,
                    };
                } else {
                    pulse.gpu = GpuPulse {
                        utilization_pct: util,
                        temperature_c: temp,
                        vram_used_gb: 0.0,
                        power_draw_watts: power,
                        fan_speed_pct: fan,
                    };
                }
            }
        }

        // 🛠️ Placeholder for Relay/Cache/Disk metrics (Will be updated by active engines)
        pulse.relay_latency_ms = 0;
        pulse.kv_cache_footprint_mb = 0;
        pulse.storage_throughput_mbps = 0;

        // Update atomic lock for memory-streaming
        if let Ok(mut lock) = self.state.pulse.write() {
            *lock = pulse.clone();
        }

        // Write to disk for visual verification
        if let Some(base) = dirs::config_dir().map(|d| d.join("Cluaiz")) {
            let _ = std::fs::create_dir_all(&base);
            if let Ok(json) = serde_json::to_string_pretty(&pulse) {
                let _ = std::fs::write(base.join("live_pulse.json"), json);
            }
        }
    }

    /// Spawns the Ghost Observer in a background thread to continuously stream metrics
    pub fn start_background_stream() -> Arc<ObservableHardwareState> {
        let mut live = Self::new();
        let state_ref = live.state.clone();

        // 🧪 Perform first tick synchronously to ensure non-zero data on start
        live.tick();

        std::thread::spawn(move || {
            loop {
                live.tick();
                // 500ms tick rate (Zero latency updates)
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });
        state_ref
    }
}

static GLOBAL_TELEMETRY: OnceLock<Arc<ObservableHardwareState>> = OnceLock::new();

/// 📡 Helper: Quick access to the Sovereign Live Telemetry stream.
/// Returns a singleton reference to the background Ghost Observer.
pub fn get_pulse() -> Arc<ObservableHardwareState> {
    GLOBAL_TELEMETRY
        .get_or_init(|| SystemPerformanceLive::start_background_stream())
        .clone()
}
