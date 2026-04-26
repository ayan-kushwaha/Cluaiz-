use tokenizers::Tokenizer;
use anyhow::Result;

/// UnifiedBackend: The foundational interface for all generation engines in the CURE system.
pub trait UnifiedBackend {
    /// Sequential generation (Legacy/Compatibility)
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> std::result::Result<String, String>;
    
    /// Prefill: Synchronous bulk processing for prompt saturation
    fn prefill(&mut self, prompt: &str) -> Result<()>;
    
    fn evaluate_tps(&self) -> f64;
}

/// SovereignInference: The advanced streaming iteration interface.
pub trait SovereignInference: Send + Sync + UnifiedBackend {
    /// Returns a generic response from a forward pass (implementation dependent)
    fn forward_raw(&mut self, input_ids: &[u32], pos: usize) -> Result<Vec<f32>>;
    
    /// The high-performance streaming protocol for Archer V6
    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        tokenizer: &Tokenizer,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> Result<()>;

    /// 🔗 Signal Injection Hook: Injects pre-encoded neural states directly into hardware cache.
    fn inject_signal(&mut self, _signal: crate::hardware::memory::kv_cache::stitching::SovereignSignal) -> Result<()> {
        tracing::warn!("⚠️ [Backend] Signal injection Not Implemented for this kernel.");
        Ok(())
    }
}

/// Dynamic trait alias bridging generic hardware kernels
pub type ModelWeightsWrapper = Box<dyn SovereignInference + Send + Sync>;


// ─── Expert Dispatcher (MoE Routing Protocol) ──────────────────────────────
pub trait ExpertDispatcher {
    fn route_token(&self, token_id: u32, experts: usize) -> u32;
    fn get_active_vram_offload(&self) -> usize;
}
