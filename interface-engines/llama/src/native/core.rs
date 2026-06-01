use crate::ffi::llama_cpp::{self, LlamaModelParams, LlamaContextParams};
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tracing::{info, warn};
use cluaiz_shared::StructuralDNA;

pub struct NativeLlama {
    pub model_ptr: *mut std::ffi::c_void,
    pub ctx_ptr: *mut std::ffi::c_void,
    pub interrupt_signal: Arc<AtomicBool>,
    pub n_ctx: u32,
    pub kv_cache_quantization_mode: u8,
    pub context_shifting_mode: u8,
    pub speculative_decoding_mode: u8,
}

/// 🤫 Sovereign Silence: Mute verbose native logs to prevent TUI visual noise.
#[allow(dead_code)]
extern "C" fn silent_llama_log(_level: i32, _text: *const c_char, _user_data: *mut std::ffi::c_void) {}

impl NativeLlama {
    /// 🧬 Load a model and initialize context with industrial booster params.
    pub fn load(
        model_path: &str, 
        model_params: LlamaModelParams, 
        mut ctx_params: LlamaContextParams,
        dna: &mut cluaiz_shared::metadata::dna::StructuralDNA,
        kv_cache_quantization_mode: u8,
        context_shifting_mode: u8,
        speculative_decoding_mode: u8,
    ) -> anyhow::Result<Self> {
        // ══ SOVEREIGN OPTIMIZATION (Hardware Overrides) ══
        std::env::set_var("GGML_LOG_LEVEL", "INFO");
        
        // Register default callback
        unsafe { llama_cpp::llama_log_set(None, std::ptr::null_mut()) };
        
        // 🚀 Initialize all native backends
        unsafe { 
            #[cfg(feature = "cuda")]
            {
                eprintln!("🔥 [Sovereign-Llama] Manually injecting CUDA backend...");
                let reg = llama_cpp::ggml_backend_cuda_reg();
                if !reg.is_null() {
                    llama_cpp::ggml_backend_register(reg);
                    eprintln!("🔥 [Sovereign-Llama] CUDA backend registered successfully!");
                } else {
                    eprintln!("❌ [Sovereign-Llama] CUDA reg pointer is NULL!");
                }
            }
            llama_cpp::llama_backend_init() 
        };
        
        let c_path = CString::new(model_path)?;
        
        println!("📊 [Native-Llama] FFI Parameters: n_gpu_layers = {}, use_mmap = {}, n_threads = {}, n_threads_batch = {}", model_params.n_gpu_layers, model_params.use_mmap, ctx_params.n_threads, ctx_params.n_threads_batch);
        info!("🧬 [Native-Llama] Loading model: {} | ctx: {} tokens", model_path, ctx_params.n_ctx);
        let mut model_ptr = unsafe { llama_cpp::llama_model_load_from_file(c_path.as_ptr(), model_params) };
        
        // 🔒 Mlock Graceful Fallback
        if model_ptr.is_null() && model_params.use_mlock {
            warn!("🔒 [Arbiter] mlock failed. Falling back to high-speed mmap...");
            let mut fallback_params = model_params;
            fallback_params.use_mlock = false;
            model_ptr = unsafe { llama_cpp::llama_model_load_from_file(c_path.as_ptr(), fallback_params) };
        }

        if model_ptr.is_null() {
            return Err(anyhow::anyhow!("Model Load Failure: {}", model_path));
        }

        let model_dir = std::path::Path::new(model_path).parent().unwrap_or(std::path::Path::new("."));
        eprintln!("🧬 [Native-Llama] Starting DNA Discovery for: {:?}", model_dir);
        
        dna.weights_already_loaded = true;
        
        if let Err(e) = dna.discover_from_path(model_dir) {
            eprintln!("⚠️ [Native-Llama] DNA Discovery Failed: {}", e);
        }
        
        if let Some(ctx) = dna.max_context_length {
            info!("🎯 [Native-Llama] SOVEREIGN HANDSHAKE: Setting n_ctx = {} (DNA Truth)", ctx);
            ctx_params.n_ctx = ctx as u32;
        }

        let mut speculative_decoding_mode = speculative_decoding_mode;
        if dna.model_identity.to_lowercase().contains("gemma") {
            info!("🛡️ [Native-Llama] Gemma model detected: Disabling speculative decoding.");
            speculative_decoding_mode = 0;
        }

        if speculative_decoding_mode == 1 || speculative_decoding_mode == 2 {
            std::env::set_var("GGML_CUDA_USE_GRAPHS", "0");
        } else {
            std::env::set_var("GGML_CUDA_USE_GRAPHS", "1");
        }

        let ctx_ptr = unsafe { llama_cpp::llama_init_from_model(model_ptr, ctx_params) };
        
        if ctx_ptr.is_null() {
            unsafe { llama_cpp::llama_model_free(model_ptr) };
            return Err(anyhow::anyhow!("Context Init Failure"));
        }

        Ok(Self { 
            model_ptr, 
            ctx_ptr,
            interrupt_signal: Arc::new(AtomicBool::new(false)),
            n_ctx: ctx_params.n_ctx,
            kv_cache_quantization_mode,
            context_shifting_mode,
            speculative_decoding_mode,
        })
    }

    pub fn resize_context(&mut self, ctx_params: LlamaContextParams) -> anyhow::Result<()> {
        if self.model_ptr.is_null() {
            return Err(anyhow::anyhow!("Cannot resize context: Model not loaded"));
        }
        unsafe {
            if !self.ctx_ptr.is_null() {
                llama_cpp::llama_free(self.ctx_ptr);
            }
            self.ctx_ptr = llama_cpp::llama_init_from_model(self.model_ptr, ctx_params);
            if self.ctx_ptr.is_null() {
                return Err(anyhow::anyhow!("Context Resize Failure"));
            }
            self.n_ctx = ctx_params.n_ctx;
        }
        Ok(())
    }

    pub fn stitch_signal(&self, signal_id: i32, offset: i32, length: i32) -> anyhow::Result<()> {
        unsafe {
            let memory = llama_cpp::llama_get_memory(self.ctx_ptr);
            llama_cpp::llama_memory_seq_cp(memory, signal_id, 0, 0, length);
        }
        Ok(())
    }

    pub fn stream_tokens(
        &self, 
        prompt: &str, 
        max_tokens: usize, 
        dna: &StructuralDNA,
        callback: Box<dyn FnMut(String) -> bool + Send + 'static>
    ) -> anyhow::Result<()> {
        crate::native::stream::stream_tokens(self, prompt, max_tokens, dna, callback)
    }
}

impl Drop for NativeLlama {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx_ptr.is_null() {
                llama_cpp::llama_free(self.ctx_ptr);
            }
            if !self.model_ptr.is_null() {
                llama_cpp::llama_model_free(self.model_ptr);
            }
        }
    }
}

unsafe impl Send for NativeLlama {}
unsafe impl Sync for NativeLlama {}
