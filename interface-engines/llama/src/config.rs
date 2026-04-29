//! Sovereign Implementation B: Configuration & Param Resolution.

use archer_shared::neural_core::config::{NeuralConfig, ResolvedNeuralParams};
use archer_shared::metadata::dna::StructuralDNA;

pub struct RuntimeBConfig;

impl RuntimeBConfig {
    pub fn resolve(dna: &StructuralDNA) -> ResolvedNeuralParams {
        let mut params = NeuralConfig::resolve(dna);
        
        // Dynamic overrides for accelerated backends
        // Context Window
        if let Some(ctx) = dna.dynamic_attributes.get("n_ctx").and_then(|v| v.parse::<u64>().ok()) {
            params.n_ctx = ctx as u32;
        }

        params
    }
}
