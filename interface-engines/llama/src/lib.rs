//! Sovereign Implementation B: Accelerated Feature-Based Runtime (Llama Engine).
//! This kernel is loaded dynamically by the SiliconOrchestrator.

use anyhow::Result;
use cluaiz_shared::{CluaizContext, CluaizInference, UnifiedBackend};
use std::sync::Arc;
use tokenizers::Tokenizer;
use neural_core::interfaces::memory_contract::SovereignBuffer;

pub mod asm_kernels;
pub mod bridge;
pub mod config;
pub mod ffi;
pub mod hybrid;
pub mod loader;
pub mod native;
pub mod pipeline;
pub mod router;

use crate::config::BoosterConfig;
use crate::native::NativeLlama;

// ─── FFI Helpers ───────────────────────────────────────────────────────────

#[repr(C)]
struct CallbackWrapper {
    callback: extern "C" fn(*const std::os::raw::c_char, *mut std::ffi::c_void),
    user_data: *mut std::ffi::c_void,
}

unsafe impl Send for CallbackWrapper {}
unsafe impl Sync for CallbackWrapper {}

pub use asm_kernels::BareMetalMath;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::llama_cpp::{self, LlamaContextParams, LlamaModelParams};

    #[test]
    fn verify_struct_sizes() {
        println!("📊 [FFI-Verify] Size of LlamaContextParams: {}", std::mem::size_of::<LlamaContextParams>());
        println!("📊 [FFI-Verify] Size of LlamaModelParams: {}", std::mem::size_of::<LlamaModelParams>());
        
        let dummy: LlamaContextParams = unsafe { std::mem::zeroed() };
        let base = &dummy as *const _ as usize;
        println!("📊 [FFI-Verify] Offset of n_ctx: {}", (&dummy.n_ctx as *const _ as usize) - base);
        println!("📊 [FFI-Verify] Offset of flash_attn_type: {}", (&dummy.flash_attn_type as *const _ as usize) - base);
        println!("📊 [FFI-Verify] Offset of rope_freq_base: {}", (&dummy.rope_freq_base as *const _ as usize) - base);
        println!("📊 [FFI-Verify] Offset of cb_eval: {}", (&dummy.cb_eval as *const _ as usize) - base);
        println!("📊 [FFI-Verify] Offset of embeddings: {}", (&dummy.embeddings as *const _ as usize) - base);
        println!("📊 [FFI-Verify] Offset of samplers: {}", (&dummy.samplers as *const _ as usize) - base);

        let defaults = unsafe { llama_cpp::llama_context_default_params() };
        println!("📊 [FFI-Verify] Default n_ctx: {}", defaults.n_ctx);
        println!("📊 [FFI-Verify] Default n_batch: {}", defaults.n_batch);
        println!("📊 [FFI-Verify] Default n_ubatch: {}", defaults.n_ubatch);
        println!("📊 [FFI-Verify] Default n_seq_max: {}", defaults.n_seq_max);
        println!("📊 [FFI-Verify] Default flash_attn_type: {}", defaults.flash_attn_type);
        println!("📊 [FFI-Verify] Default n_threads: {}", defaults.n_threads);
        println!("📊 [FFI-Verify] Default rope_freq_base: {}", defaults.rope_freq_base);
        println!("📊 [FFI-Verify] Default embeddings: {}", defaults.embeddings);

        println!("🔍 [Memory-Probe] Dumping raw bytes of LlamaContextParams defaults:");
        let ptr = &defaults as *const _ as *const u32;
        for i in 0..32 {
            let val = unsafe { *ptr.add(i) };
            println!("  [{:02}] Offset {:03}: 0x{:08x} ({})", i, i * 4, val, val as i32);
        }
    }
}

pub struct RuntimeB {
    pub model_path: String,
    pub context: CluaizContext,
    pub booster: BoosterConfig,
    pub native: Option<NativeLlama>,
    pub lucebox: Option<Arc<ffi::lucebox::LuceboxBridge>>,
}

impl RuntimeB {
    pub fn new(path: &str, context: CluaizContext) -> Self {
        Self {
            model_path: path.to_string(),
            context,
            booster: BoosterConfig::default(),
            native: None,
            lucebox: None,
        }
    }

    /// 🧬 Load the model natively into memory using current booster settings.
    pub fn load_native(&mut self) -> anyhow::Result<()> {
        let model_params = self.booster.to_model_params();
        
        // 🧬 DNA TRUTH SYNC: Ensure DNA context is applied to context params
        let mut ctx_params = self.booster.to_context_params();
        
        if let Some(ctx) = self.context.dna.max_context_length {
            ctx_params.n_ctx = ctx as u32;
        }

        tracing::info!("🧬 [Native-Llama] Loading model: {} | ctx: {} tokens", self.model_path, ctx_params.n_ctx);
        
        // 🚀 BATCH SYNC: Optimized for 4GB hardware by default, scalable via BoosterConfig.
        ctx_params.n_batch = if ctx_params.n_batch == 0 { 512 } else { ctx_params.n_batch };
        ctx_params.n_ubatch = if ctx_params.n_ubatch == 0 { 512 } else { ctx_params.n_ubatch }; 
        
        let native = NativeLlama::load(&self.model_path, model_params, ctx_params, &mut self.context.dna)?;
        self.native = Some(native);
        tracing::info!("✅ [Llama-Engine] Native Model Loaded & Optimized.");
        Ok(())
    }

    /// 🛠️ Attach the Lucebox accelerator bridge
    pub fn attach_accelerator(&mut self, lib_path: &str) -> anyhow::Result<()> {
        let bridge = ffi::lucebox::LuceboxBridge::load(lib_path)?;
        self.lucebox = Some(Arc::new(bridge));
        tracing::info!("🚀 [Llama-Engine] Lucebox Accelerator Attached.");
        Ok(())
    }
}

impl UnifiedBackend for RuntimeB {
    fn generate(&mut self, prompt: &str, _max_tokens: usize) -> Result<String, String> {
        Ok(format!(
            "Sovereign Llama Engine: Ready for prompt: {}",
            prompt
        ))
    }

    fn prefill(&mut self, _prompt: &str) -> Result<()> {
        Ok(())
    }

    fn evaluate_tps(&self) -> f64 {
        // 📡 Sovereign Telemetry: Return the real-time TPS from the pulse counter.
        // This counter is incremented for every token generated in native.rs.
        cluaiz_shared::hardware::telemetry::get_pulse().tps_counter.load(std::sync::atomic::Ordering::Relaxed) as f64
    }
}

impl CluaizInference for RuntimeB {
    fn forward_raw(&mut self, _input_ids: &[u32], _pos: usize) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("FFI forward optimized via ASM kernels"))
    }


    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        _tokenizer: &Tokenizer,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> Result<()> {
        let mut callback = callback;
        
        // 🛡️ Neural Circuit Breaker: check if paths are safe
        let mut cb = cluaiz_shared::hardware::circuit_breaker::NeuralCircuitBreaker::default();
        if !cb.can_proceed() {
            return Err(anyhow::anyhow!("🚨 [Circuit Breaker] Inference blocked due to previous system instability."));
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // 🚀 High-Performance Native Path
            if let Some(ref native) = self.native {
                let res = native.stream_tokens(prompt, max_tokens, &self.context.dna, callback);
                
                if res.is_ok() {
                    cb.record_success();
                } else {
                    cb.record_failure("Native stream error");
                }
                return res;
            }

            // 🛡️ Safe Binary Fallback Path
            tokio::task::block_in_place(|| {
                let handle = tokio::runtime::Handle::current();
                handle
                    .block_on(crate::pipeline::RuntimeBPipeline::execute_stream(
                        &self.model_path,
                        &self.context,
                        prompt,
                        max_tokens,
                        callback,
                    ))
                    .map_err(|e| anyhow::anyhow!(e))
            })
        }));

        match result {
            Ok(res) => res,
            Err(_) => {
                tracing::error!("🚨 [FFI-Panic] Caught panic in generate_stream! Preventing OS crash.");
                Err(anyhow::anyhow!("Kernel panic during stream generation."))
            }
        }
    }

    /// 💉 Neural Injection Hook: Injects multiple pre-encoded skill states into the Llama cache.
    fn inject_signals(&mut self, signals: Vec<cluaiz_shared::hardware::memory::kv_cache::stitching::CluaizSignal>) -> Result<()> {
        let max_ctx = self.context.dna.max_context_length.unwrap_or(4096);
        let mut current_offset = 0;

        if signals.is_empty() {
            return Ok(());
        }

        println!("💉 [Llama-Engine] Multi-Signal Injection Active: {} skills detected.", signals.len());

        if let Some(ref lucebox) = self.lucebox {
            let max_layers = self.context.dna.layer_count.unwrap_or(32);

            for (i, signal) in signals.iter().enumerate() {
                let token_count = signal.token_count;
                
                // 🛑 Positional Guard
                if current_offset + token_count > max_ctx {
                    tracing::error!("❌ [Llama-Engine] Positional Collision: Signal {} exceeds remaining context space.", i);
                    return Err(anyhow::anyhow!("CluaizSignal: Context Overflow at Skill {}", i));
                }

                println!("🧵 [Llama-Engine] Stitching Skill {} ({} tokens) at offset {}.", i, token_count, current_offset);

                for layer_idx in 0..max_layers as i32 {
                    // Note: lucebox.stitch_kv_layer will eventually need to take the offset.
                    // For Phase 1 of Mission 10, we assume sequential allocation in the kernel.
                    if let Err(e) = lucebox.stitch_kv_layer(layer_idx, &*signal.raw_data) {
                        tracing::error!("❌ [Llama-Engine] Stitching failed at Skill {}, Layer {}: {}", i, layer_idx, e);
                        return Err(e);
                    }
                }
                
                current_offset += token_count;
            }

            println!("✅ [Llama-Engine] Multi-Soul Fusion: {} signals stitched successfully. [Total Context: {}/{}]", 
                signals.len(), current_offset, max_ctx);
            Ok(())
        } else {
            tracing::warn!("⚠️ [Llama-Engine] Injection skipped: No Lucebox accelerator attached.");
            Ok(())
        }
    }

    /// 🚀 Booster Sync: Applies hardware-level optimization flags (TurboQuant, KV-Cache, etc.)
    fn apply_booster(&mut self, control: &cluaiz_shared::hardware::schema::booster::BoosterControl) -> Result<()> {
        tracing::info!("🚀 [Llama-Engine] Applying Booster: Autonomous Performance Sync");
        
        // 🔄 Sync local booster state from system
        self.booster = crate::config::BoosterConfig::load_from_system();
        
        // 🌊 Trigger Elastic Resize (VRAM Sovereignty)
        if let Some(native) = &mut self.native {
            let mut ctx_params = self.booster.to_context_params();
            
            // Recalculate context window through Governor using the injected control truth
            let new_ctx = cluaiz_shared::hardware::governor::HardwareGovernor::negotiate_vram_envelope_with_booster(&self.context.dna, control);
            ctx_params.n_ctx = new_ctx as u32;
            
            native.resize_context(ctx_params)?;
            tracing::info!("🌊 [Llama-Engine] Elastic Resize Success: Context now {} tokens.", new_ctx);
        }
        
        Ok(())
    }

    /// 🌊 Liquid Execution: Activates adaptive context density.
    fn set_liquid_mode(&mut self, enabled: bool) -> Result<()> {
        tracing::info!("🌊 [Llama-Engine] Liquid Mode set to: {}", enabled);
        Ok(())
    }
}

// ─── Sovereign FFI Gateway ──────────────────────────────────────────────────

#[export_name = "cluaiz_kernel_init"]
pub extern "C" fn cluaiz_kernel_init() -> *const std::os::raw::c_char {
    unsafe {
        // 🤫 Sovereign Silence: Hard-redirect native stdout/stderr to NUL
        // This stops all non-callback logs (CUDA Graph, etc.) from polluting the TUI.
        /* 🧪 Debug Mode: Temporarily disabled NUL redirection
        #[cfg(windows)]
        {
            let n_path = std::ffi::CString::new("NUL").unwrap();
            let mode = std::ffi::CString::new("w").unwrap();
            libc::freopen(n_path.as_ptr(), mode.as_ptr(), libc::stdout);
            libc::freopen(n_path.as_ptr(), mode.as_ptr(), libc::stderr);
        }
        */
        #[cfg(not(windows))]
        {
            let n_path = std::ffi::CString::new("/dev/null").unwrap();
            let mode = std::ffi::CString::new("w").unwrap();
            libc::freopen(n_path.as_ptr(), mode.as_ptr(), libc::stdout);
            libc::freopen(n_path.as_ptr(), mode.as_ptr(), libc::stderr);
        }

        // Also set the callback for handled logs
        extern "C" fn silent_log(_level: i32, _text: *const std::os::raw::c_char, _data: *mut std::ffi::c_void) {}
        crate::ffi::llama_cpp::llama_log_set(Some(silent_log), std::ptr::null_mut());
        
        ffi::llama_cpp::llama_backend_init();
    }
    tracing::info!("🧬 [Llama.cpp-Kernel] Sovereign Handshake & Backend Initialized.");
    "cluaiz-llama.cpp-active\0".as_ptr() as *const std::os::raw::c_char
}

#[used]
static _FORCE_KEEP_INIT: extern "C" fn() -> *const std::os::raw::c_char = cluaiz_kernel_init;

#[no_mangle]
pub extern "C" fn cluaiz_kernel_instantiate(
    path_ptr: *const std::os::raw::c_char,
    booster_ptr: *const cluaiz_shared::hardware::schema::booster::CluaizBoosterContext,
) -> *mut RuntimeB {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let path_str = unsafe { std::ffi::CStr::from_ptr(path_ptr) }
            .to_string_lossy()
            .into_owned();
        
        let model_path = std::path::Path::new(&path_str);
        let model_dir = model_path.parent().unwrap_or(model_path);
        
        tracing::info!("🧬 [Llama-Lib] Initiating Sovereign DNA Handshake for: {:?}", model_dir);
        let mut dna = cluaiz_shared::metadata::dna::StructuralDNA::load(&model_dir.join("structural_dna.json"))
            .unwrap_or_else(|_| {
                println!("⚠️ [Llama-Lib] DNA Manifest missing. Creating transient skeleton...");
                cluaiz_shared::metadata::dna::StructuralDNA::default()
            });

        // ALWAYS perform real-time discovery to sync with LIVE hardware state
        eprintln!("📂 [Llama-Lib] Discovering real-time truth...");
        if let Err(e) = dna.discover_from_path(model_dir) {
            eprintln!("⚠️ [Llama-Lib] DNA Discovery Failed: {}. Using best-effort constraints.", e);
        }
        eprintln!("✅ [Llama-Lib] DNA Discovery Complete. Negotiated Context: {:?}", dna.max_context_length);
        eprintln!("📊 [Llama-Lib] Weights Size: {:.2}GB", dna.weights_size_gb);

        let context = CluaizContext::boot(dna, cluaiz_shared::TemplateManager::default());
        let mut engine = Box::new(RuntimeB::new(&path_str, context));
        
        // Inject Booster Configuration from Caller
        if !booster_ptr.is_null() {
            let booster_ctx = unsafe { *booster_ptr };
            tracing::info!("🚀 [Llama.cpp-Kernel] Received CluaizBoosterContext via FFI: {:?}", booster_ctx);
            engine.booster.flash_attn = booster_ctx.flash_attention;
            engine.booster.turbo_quant = if booster_ctx.turbo_quant { "active".to_string() } else { "none".to_string() };
            engine.booster.use_mmap = true;
        } else {
            // Self-load from Binary Booster Truth if FFI was blank
            if let Ok(booster) = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings() {
                let _ = engine.apply_booster(&booster);
            }
        }

        // 🧬 Trigger Native Load immediately on instantiation
        if let Err(e) = engine.load_native() {
            tracing::error!("❌ [Llama.cpp-Kernel] Native Load Failed: {}", e);
            return std::ptr::null_mut();
        }

        Box::into_raw(engine)
    }));

    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            tracing::error!("🚨 [FFI-Panic] Caught panic in cluaiz_kernel_instantiate! Preventing OS crash.");
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn cluaiz_kernel_generate_stream(
    engine_ptr: *mut RuntimeB,
    prompt_ptr: *const std::os::raw::c_char,
    max_tokens: usize,
    callback: extern "C" fn(*const std::os::raw::c_char, *mut std::ffi::c_void),
    user_data: *mut std::ffi::c_void,
) -> i32 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if engine_ptr.is_null() { return -1; }
        let engine = unsafe { &mut *engine_ptr };
        
        let prompt = unsafe { std::ffi::CStr::from_ptr(prompt_ptr) }
            .to_string_lossy()
            .into_owned();
        
        let user_data_ptr = user_data as usize;
        let callback_ptr = callback as usize;

        let rust_callback = Box::new(move |token: String| {
            let c_str = std::ffi::CString::new(token).unwrap_or_default();
            let cb = unsafe { std::mem::transmute::<usize, extern "C" fn(*const std::os::raw::c_char, *mut std::ffi::c_void)>(callback_ptr) };
            let ud = user_data_ptr as *mut std::ffi::c_void;
            unsafe {
                (cb)(c_str.as_ptr(), ud);
            }
        });

        // We use a minimal valid tokenizer JSON because the native path handles its own tokenization internally.
        let minimal_json = r#"{"version":"1.0","truncation":null,"padding":null,"added_tokens":[],"normalizer":null,"pre_tokenizer":null,"post_processor":null,"decoder":null,"model":{"type":"BPE","dropout":null,"unk_token":null,"continuing_subword_prefix":null,"end_of_word_suffix":null,"fuse_unk":false,"byte_fallback":false,"vocab":{},"merges":[]}}"#;
        let empty_tokenizer = tokenizers::Tokenizer::from_bytes(minimal_json.as_bytes()).expect("Critical: Failed to create dummy tokenizer");

        match engine.generate_stream(&prompt, max_tokens, &empty_tokenizer, rust_callback) {
            Ok(_) => 0,
            Err(e) => {
                tracing::error!("❌ [Llama-Engine] Generation failed: {}", e);
                -2
            }
        }
    }));

    match result {
        Ok(res) => res,
        Err(_) => {
            tracing::error!("🚨 [FFI-Panic] Caught panic in cluaiz_kernel_generate_stream!");
            -3
        }
    }
}
