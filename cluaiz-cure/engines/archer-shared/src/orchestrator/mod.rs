use crate::backend::traits::{UnifiedBackend, SovereignInference};
use crate::backend::context::SovereignContext;
use anyhow::Result;
use tokenizers::Tokenizer;

/// SovereignLinkerPlaceholder: Used during Phase 1 to verify the Dynamic Linker Handshake.
pub struct SovereignLinkerPlaceholder;

impl UnifiedBackend for SovereignLinkerPlaceholder {
    fn generate(&mut self, _prompt: &str, _max_tokens: usize) -> std::result::Result<String, String> {
        Err("✅ SOVEREIGN LINKER: Phase 1 Handshake Success. Real inference pending Phase 2.".to_string())
    }
    fn prefill(&mut self, _prompt: &str) -> Result<()> { Ok(()) }
    fn evaluate_tps(&self) -> f64 { 0.0 }
}

impl SovereignInference for SovereignLinkerPlaceholder {
    fn forward_raw(&mut self, _input_ids: &[u32], _pos: usize) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("Handshake Placeholder"))
    }
    fn generate_stream(
        &mut self,
        _prompt: &str,
        _max_tokens: usize,
        _tokenizer: &Tokenizer,
        _callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> Result<()> {
        Err(anyhow::anyhow!("✅ SOVEREIGN LINKER: Handshake Complete. Kernel symbols resolved. Ready for Phase 2 Implementation."))
    }
}
