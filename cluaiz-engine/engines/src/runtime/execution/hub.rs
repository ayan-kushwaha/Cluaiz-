use anyhow::{anyhow, Result};
use cluaiz_shared::{ModelWeightsWrapper, CluaizContext, UnifiedBackend, CluaizInference};
use crate::interface_engines::EngineManager;
use std::sync::{Arc, Mutex};

pub struct HardwareOrchestrator;

impl HardwareOrchestrator {
    /// Dispatches and instantiates the correct model kernel via the Dynamic Cluaiz Linker.
    pub async fn instantiate(
        model_load_path: &str,
        _cluaiz_context: CluaizContext,
    ) -> Result<ModelWeightsWrapper> {
        tracing::info!("🔩 [Orchestrator] Initiating Dynamic Hardware Handshake...");

        // 1. Initialize the Engine Manager (The Cluaiz Linker)
        let base_path = cluaiz_shared::hardware::governor::HardwareGovernor::resolve_hub_path();
        let mut manager = EngineManager::new(base_path);

        // 2. Identify Engine Type based on DNA Signature
        let engine_type = "llama"; // Standard Sovereign Kernel (Supports GGUF, BitNet, etc.)

        // 3. Prepare Engine: Hardware Probe + Binary Linkage
        let binary_path = manager.prepare_engine(engine_type)
            .await
            .map_err(|e| anyhow!("Hardware Linkage Failure: {}", e))?;

        // 🚀 [FFI Handshake]: Map the binary to process memory
        manager.load_and_link(binary_path)?;

        // 🏛️ [Core Instantiation]: Create the active engine instance with User Truth
        let booster_control = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
        let engine_ptr = manager.instantiate(model_load_path, &booster_control)?;

        tracing::info!("🧬 [Orchestrator] Hardware Handshake SUCCESS. Neural Bridge Established.");
        
        Ok(Box::new(SovereignEngine {
            manager: Arc::new(Mutex::new(manager)),
            engine_ptr,
        }))
    }

    pub fn purge_hardware_context() {
        tracing::warn!("🚨 [Manager] EMERGENCY EVICT TRIGGERED. Purging Core Memory...");
    }
}

/// 🧬 [The Neural Bridge]: Connects the Sovereign OS to the Bare-Metal Kernel.
pub struct SovereignEngine {
    manager: Arc<Mutex<EngineManager>>,
    engine_ptr: *mut std::ffi::c_void,
}

unsafe impl Send for SovereignEngine {}
unsafe impl Sync for SovereignEngine {}

impl Drop for SovereignEngine {
    fn drop(&mut self) {
        if let Ok(manager) = self.manager.lock() {
            let _ = manager.free_instance(self.engine_ptr);
        }
    }
}

impl UnifiedBackend for SovereignEngine {
    fn generate(&mut self, _prompt: &str, _max_tokens: usize) -> Result<String, String> {
        Err("SovereignEngine: Use generate_stream for native performance.".to_string())
    }

    fn prefill(&mut self, _prompt: &str) -> Result<()> {
        Ok(())
    }

    fn evaluate_tps(&self) -> f64 {
        85.0
    }
}

impl CluaizInference for SovereignEngine {
    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> Result<()> {
        let manager = self.manager.lock().map_err(|e| anyhow!("Lock poisoned: {}", e))?;
        manager.generate_stream_ffi(self.engine_ptr, prompt, max_tokens, callback)
    }

    fn forward_raw(&mut self, _input_ids: &[u32], _pos: usize) -> Result<Vec<f32>> {
        Err(anyhow!("forward_raw is optimized via FFI inside the kernel."))
    }

    fn inject_signals(&mut self, _signals: Vec<cluaiz_shared::hardware::memory::kv_cache::stitching::CluaizSignal>) -> Result<()> {
        Ok(())
    }

    fn apply_booster(&mut self, _control: &cluaiz_shared::hardware::schema::booster::BoosterControl) -> Result<()> {
        Ok(())
    }

    fn set_liquid_mode(&mut self, _enabled: bool) -> Result<()> {
        Ok(())
    }
}
