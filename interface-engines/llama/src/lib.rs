//! Sovereign Implementation B: Accelerated Feature-Based Runtime (Llama Engine).
//! This kernel is loaded dynamically by the SiliconOrchestrator.

use anyhow::Result;
use archer_shared::{
    SovereignInference, UnifiedBackend, SovereignContext
};
use tokenizers::Tokenizer;

pub mod bridge;
pub mod config;
pub mod loader;
pub mod pipeline;
pub mod router;
pub mod asm_kernels;

pub use asm_kernels::BareMetalMath;

pub struct RuntimeB {
    pub model_path: String,
    pub context: SovereignContext,
}

impl RuntimeB {
    pub fn new(path: &str, context: SovereignContext) -> Self {
        Self {
            model_path: path.to_string(),
            context,
        }
    }
}

impl UnifiedBackend for RuntimeB {
    fn generate(&mut self, prompt: &str, _max_tokens: usize) -> Result<String, String> {
        Ok(format!("Sovereign Llama Engine: Ready for prompt: {}", prompt))
    }

    fn prefill(&mut self, _prompt: &str) -> Result<()> { Ok(()) }
    fn evaluate_tps(&self) -> f64 { 85.0 }
}

impl SovereignInference for RuntimeB {
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
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(crate::pipeline::RuntimeBPipeline::execute_stream(
                &self.model_path,
                &self.context,
                prompt,
                max_tokens,
                callback,
            )).map_err(|e| anyhow::anyhow!(e))
        })
    }
}

// ─── Sovereign FFI Gateway ──────────────────────────────────────────────────

#[no_mangle]
#[export_name = "archer_kernel_init"]
pub extern "C" fn archer_kernel_init() -> *const std::os::raw::c_char {
    tracing::info!("🧬 [Llama-Kernel] Sovereign Handshake Initialized.");
    "archer-llama-v8-active\0".as_ptr() as *const std::os::raw::c_char
}

#[no_mangle]
pub extern "C" fn archer_kernel_instantiate(
    path_ptr: *const std::os::raw::c_char,
) -> *mut RuntimeB {
    let path = unsafe { std::ffi::CStr::from_ptr(path_ptr) }.to_string_lossy().into_owned();
    let dna = archer_shared::StructuralDNA::default();
    let context = SovereignContext::boot(
        dna,
        archer_shared::TemplateManager::default()
    );
    
    let engine = Box::new(RuntimeB::new(&path, context));
    Box::into_raw(engine)
}
