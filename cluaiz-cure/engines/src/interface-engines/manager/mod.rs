use std::path::PathBuf;
use crate::interface_engines::manager::kernel_loader::KernelLoader;
use crate::interface_engines::manager::driver_bridge::DriverBridge;
use archer_shared::hardware::schema::profiles::SystemControl;
use archer_shared::hardware::governor::HardwareGovernor;

pub mod kernel_loader;
pub mod driver_bridge;

/// Sovereign Engine Manager
/// Orchestrates pre-compiled Kernels (BitNet, Llama, Candle) and Silicon Drivers.
pub struct EngineManager {
    kernel_dir: PathBuf,
    loader: KernelLoader,
    bridge: DriverBridge,
}

impl EngineManager {
    pub fn new(kernel_dir: PathBuf) -> Self {
        Self {
            kernel_dir: kernel_dir.clone(),
            loader: KernelLoader::new(kernel_dir),
            bridge: DriverBridge::new(),
        }
    }

    /// Handshake: Identify the target silicon and ensure correct kernel/driver presence.
    pub async fn prepare_engine(&self, engine_type: &str) -> Result<PathBuf, String> {
        let config_path = self.get_system_control_path();
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Hardware Config Missing: {}", e))?;
        
        let control: SystemControl = serde_json::from_str(&content)
            .map_err(|e| format!("Hardware Config Parse Error: {}", e))?;

        // 🚀 Sovereign Detection Logic: The Triple Handshake
        let os = control.identity.os_target.to_lowercase();
        let arch = control.identity.architecture.to_lowercase();
        let gpu_vendor = control.silicon_truth.accelerators.gpus.first().map(|g| g.vendor.to_lowercase());
        let has_drivers = !control.silicon_truth.active_drivers.is_empty();

        println!("🎯 Engine Prep: OS={}, Arch={}, GPU={:?}, Drivers={}", os, arch, gpu_vendor, has_drivers);

        // 1. Resolve Silicon Suffix based on Deep Probing
        let suffix = match (os.as_str(), arch.as_str()) {
            // --- Apple Silicon (Metal Mastery) ---
            ("macos", "aarch64") if gpu_vendor.as_ref().map_or(false, |v| v.contains("apple")) => "metal",
            
            // --- Linux/Windows High-Performance Targets ---
            ("linux", _) | ("windows", _) if gpu_vendor.is_some() => {
                let vendor = gpu_vendor.as_ref().unwrap();
                if vendor.contains("nvidia") && has_drivers {
                    "cuda"
                } else if vendor.contains("amd") {
                    "rocm"
                } else {
                    "vulkan"
                }
            }

            // --- ARM / Raspberry Pi Optimized ---
            ("linux", "aarch64") | ("linux", "arm") => "arm64",

            // --- Mobile Sovereign Targets ---
            ("android", _) => "android",
            ("ios", _) => "ios",

            // --- Legacy / Generic Fallback ---
            _ => "cpu",
        };

        let binary_id = format!("{}-{}", engine_type, suffix);

        // 🚀 Sovereign VRAM Handshake: Pre-Flight Check
        // Defaulting to 2.0GB for V1 Baseline. Future: Pull from model metadata.
        let required_vram = 2.0; 
        HardwareGovernor::request_vram(&binary_id, required_vram)
            .map_err(|e| format!("Memory Arbitration Failed: {}", e))?;

        // 2. Check local presence in cluaiz/kernels/
        if self.loader.exists(&binary_id) {
            Ok(self.loader.resolve_path(&binary_id))
        } else {
            // If binary missing, release the reserved memory immediately
            let _ = HardwareGovernor::release_vram(&binary_id);
            Err(format!("Engine Binary Missing: Please pull the '{}' package for your {} silicon", binary_id, os))
        }
    }

    /// Unload Engine: Release resources back to the Sovereign Governor.
    pub fn release_engine(&self, engine_type: &str) -> anyhow::Result<()> {
        // We need to know which suffix was used to reconstruct the ID
        // For simplicity in V1, we iterate and release what matches the prefix
        HardwareGovernor::release_vram(engine_type)
    }

    fn get_system_control_path(&self) -> PathBuf {
        HardwareGovernor::resolve_base_path().join("system_control.json")
    }
}
