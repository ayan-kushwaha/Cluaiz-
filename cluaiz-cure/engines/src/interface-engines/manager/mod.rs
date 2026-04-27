use std::path::PathBuf;
use crate::interface_engines::manager::kernel_loader::KernelLoader;
use crate::interface_engines::manager::driver_bridge::DriverBridge;
use archer_shared::hardware::schema::profiles::SystemControl;

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

        // 🚀 Sovereign Detection Logic
        let os = control.identity.os_target;
        let gpu = control.silicon_truth.accelerators.gpus.first();
        let has_drivers = !control.silicon_truth.active_drivers.is_empty();

        println!("🎯 Engine Prep: OS={}, GPU={:?}, Drivers={}", os, gpu.as_ref().map(|g| &g.model), has_drivers);

        // 1. Resolve target Binary suffix based on OS/Vendor
        let binary_id = match os.as_str() {
            "Windows" | "Linux" if gpu.is_some() && has_drivers => format!("{}-cuda", engine_type),
            "Android" => format!("{}-android", engine_type),
            "iOS" => format!("{}-ios", engine_type),
            _ => format!("{}-cpu", engine_type),
        };

        // 2. Check local presence in cluaiz/kernels/
        if self.loader.exists(&binary_id) {
            Ok(self.loader.resolve_path(&binary_id))
        } else {
            Err(format!("Engine Binary Missing: Please pull the {} package for {}", binary_id, os))
        }
    }

    fn get_system_control_path(&self) -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\Aryan\\AppData\\Roaming"));
        path.push("Cluaiz");
        path.push("system_control.json");
        path
    }
}
