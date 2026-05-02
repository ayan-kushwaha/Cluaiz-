use std::path::PathBuf;
use libloading::{Library, Symbol};
use crate::interface_engines::manager::kernel_loader::KernelLoader;
use crate::interface_engines::manager::driver_bridge::DriverBridge;
use archer_shared::hardware::schema::profiles::SystemControl;
use archer_shared::hardware::governor::HardwareGovernor;

pub mod kernel_loader;
pub mod driver_bridge;
pub mod npu_bridge;

/// Sovereign Engine Manager
/// Orchestrates pre-compiled Kernels (BitNet, Llama, Candle) and Silicon Drivers.
pub struct EngineManager {
    kernel_dir: PathBuf,
    loader: KernelLoader,
    bridge: DriverBridge,
    // 🏛️ The Soul Link: Holds the active binary in process memory
    active_lib: Option<Library>,
}

impl EngineManager {
    pub fn new(kernel_dir: PathBuf) -> Self {
        Self {
            kernel_dir: kernel_dir.clone(),
            loader: KernelLoader::new(kernel_dir),
            bridge: DriverBridge::new(),
            active_lib: None,
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

        // 🧠 Mission 12: Chronicle Neural Activity
        let _ = archer_shared::neural::graph::NeuralGraph::chronicle_pulse(
            "Silicon Handshake & Engine Preparation",
            engine_type,
            &format!("OS: {}, GPU: {:?}", os, gpu_vendor)
        );

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

        // 2. Check local presence in cluaiz/interface-engines/
        if self.loader.exists(&binary_id) {
            Ok(self.loader.resolve_path(&binary_id))
        } else {
            // If binary missing, release the reserved memory immediately
            let _ = HardwareGovernor::release_vram(&binary_id);
            Err(format!("Engine Binary Missing: Please pull the '{}' package for your {} silicon into your cluaiz/interface-engines/ folder", binary_id, os))
        }
    }

    /// Unload Engine: Release resources back to the Sovereign Governor.
    pub fn release_engine(&self, engine_type: &str) -> anyhow::Result<()> {
        // We need to know which suffix was used to reconstruct the ID
        // For simplicity in V1, we iterate and release what matches the prefix
        HardwareGovernor::release_vram(engine_type)
    }

    /// 🔗 Sovereign Linker: Maps the binary kernel to process memory and resolves symbols.
    pub fn load_and_link(&mut self, binary_path: PathBuf) -> anyhow::Result<()> {
        tracing::info!("🧬 [Linker] Mapping binary: {:?}", binary_path);
        
        unsafe {
            let lib = Library::new(&binary_path)
                .map_err(|e| anyhow::anyhow!("Binary Mapping Failed (libloading): {}", e))?;
            
            // 🎯 Phase 1: Symbol Validation
            let _init: Symbol<unsafe extern "C" fn() -> *const i8> = lib.get(b"archer_kernel_init")
                .map_err(|_| anyhow::anyhow!("Invalid Kernel: 'archer_kernel_init' symbol missing."))?;

            tracing::info!("✅ [Linker] 7ns Handshake Complete. Kernel Linked.");
            self.active_lib = Some(lib);
        }
        
        Ok(())
    }

    /// 🏛️ Neural Instantiation: Invokes the kernel's factory method to create an active execution engine.
    pub fn instantiate(&self, model_path: &str) -> anyhow::Result<()> {
        let lib = self.active_lib.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Linker Error: No active kernel linked."))?;
        
        unsafe {
            let instantiate_fn: Symbol<unsafe extern "C" fn(*const i8) -> *mut std::ffi::c_void> = 
                lib.get(b"archer_kernel_instantiate")
                .map_err(|_| anyhow::anyhow!("Invalid Kernel: 'archer_kernel_instantiate' symbol missing."))?;
            
            let c_path = std::ffi::CString::new(model_path)?;
            let _engine_ptr = instantiate_fn(c_path.as_ptr());
            
            tracing::info!("🚀 [Linker] Neural Kernel Instantiated at Bare-Metal level.");
        }
        
        Ok(())
    }

    fn get_system_control_path(&self) -> PathBuf {
        HardwareGovernor::resolve_base_path().join("interface-engines").join("system_control.json")
    }
}
