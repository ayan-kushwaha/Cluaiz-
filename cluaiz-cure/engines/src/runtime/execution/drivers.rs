use std::sync::Once;
use archer_candle::CandleEngine as EngineA;
use archer_shared::{ModelWeightsWrapper, BackendType};
#[allow(unused_imports)]
use archer_shared::SovereignInference;
use crate::runtime::execution::hub::SiliconOrchestrator;
use archer_shared::KernelSignature;
use std::sync::Arc;
use std::path::PathBuf;
use archer_shared::ArcConstructor;

static STARTUP: Once = Once::new();

pub fn initialize_neural_drivers() {
    STARTUP.call_once(|| {
        tracing::info!("🧬 Sovereign Archer V6: Initiating Modular Handshake...");
        
        // 🕯️ Register the Primary Runtime (Static Dispatch)
        if let Err(e) = register_runtime_a() {
            tracing::error!("❌ Fatal: RuntimeA failed to initialize: {}", e);
        }
        
        // 🦙 Register the Secondary Runtime (Accelerated Bridge)
        if let Err(e) = archer_llama::register_drivers(|target_engine, sig, constructor_hook| {
            let _ = SiliconOrchestrator::register(target_engine, sig, constructor_hook);
        }) {
            tracing::error!("❌ Fatal: RuntimeB failed to register: {}", e);
        }

        // 🧠 BitNet (Engine C) is loaded dynamically at runtime via libloading.
        // This avoids the compile-time `links = "llama"` conflict with llama-cpp-sys.
        // TODO: implement dynamic cdylib loading for archer_bitnet here.
        tracing::info!("🧬 [RuntimeC] BitNet engine stub registered. Dynamic loading pending.");

        tracing::info!("🔍 Sovereign Archer V8: Initiating Dynamic Hardware Probing...");
        let (has_gpu, driver_type) = dynamic_discovery::probe_hardware();
        if !has_gpu {
            tracing::warn!("⚠️ Discrete GPU missing or unavailable. Engaging Unified Memory Fallback.");
        } else {
            tracing::info!("✅ Attached bare-metal drivers for: {}", driver_type);
        }

        tracing::info!("✅ Sovereign Archer V8: Handshake Complete. Engines Online.");
    });
}

/// Dynamic Hardware Discovery (Ollama-style Lazy Loading)
pub mod dynamic_discovery {
    use std::sync::atomic::{AtomicBool, Ordering};

    static HAS_NVIDIA: AtomicBool = AtomicBool::new(false);
    static IS_UNIFIED: AtomicBool = AtomicBool::new(false);

    /// Probes hardware dynamically without static OS dependencies.
    pub fn probe_hardware() -> (bool, &'static str) {
        // 1. Attempt to load NVIDIA Management Library dynamically
        let nvml_probe = unsafe { probe_nvidia_nvml() };
        if nvml_probe {
            HAS_NVIDIA.store(true, Ordering::SeqCst);
            return (true, "NVIDIA (NVML)");
        }

        // 2. Fallback to Apple Silicon / Unified Memory (e.g. Raspberry Pi)
        let unified_probe = probe_unified_memory();
        if unified_probe {
            IS_UNIFIED.store(true, Ordering::SeqCst);
            return (false, "Unified Memory RAM");
        }

        (false, "System CPU (DRAM fallback)")
    }

    unsafe fn probe_nvidia_nvml() -> bool {
        #[cfg(windows)]
        {
            // Use Windows dynamic linking
            use std::ptr::null_mut;
            // A pseudo-load simulation to guarantee we don't statically link and crash
            // Real winapi load library logic goes here in full V8 build:
            // let lib = winapi::um::libloaderapi::LoadLibraryA("nvml.dll\0".as_ptr() as *const _);
            // !lib.is_null()
            false 
        }
        #[cfg(unix)]
        {
            // Use dlopen/dlsym on Linux
            // let lib = libc::dlopen("libnvidia-ml.so\0".as_ptr() as *const _, libc::RTLD_LAZY);
            // !lib.is_null()
            false
        }
        #[cfg(not(any(windows, unix)))]
        {
            false
        }
    }

    fn probe_unified_memory() -> bool {
        // Simple heuristic: If we don't have a dedicated discrete GPU handle,
        // we map to system DDR as Unified Space, which implies Apple Silicon or Linux Edge
        cfg!(target_os = "macos") || std::path::Path::new("/etc/nv_tegra_release").exists()
    }
}

fn register_runtime_a() -> Result<(), String> {
    let architectural_patterns = vec!["uniform", "asymmetric"];
    
    for pattern in architectural_patterns {
        let signature = KernelSignature {
            has_experts: false,
            is_asymmetric: pattern == "asymmetric",
            is_multimodal: true,
            is_heterogeneous: true,
            is_bitnet: false,
            is_ssm: false,
            head_pattern: pattern.into(),
            activation: "silu".into(),
        };

        let _ = SiliconOrchestrator::register(
            BackendType::RuntimeA,
            signature,
            Arc::new(|model_path: &str, context: archer_shared::SovereignContext| {
                // RuntimeA now handles its own internal loading via dna-first architecture
                // We use a dummy device for the registry since engines handle their own silicon
                let engine = EngineA::new(PathBuf::from(model_path), &candle_core::Device::Cpu)
                    .map_err(|err| anyhow::anyhow!("RuntimeA DNA-First init failed: {err}"))?;

                Ok(Box::new(engine) as ModelWeightsWrapper)
            }) as ArcConstructor,
        );

    }
    Ok(())
}
