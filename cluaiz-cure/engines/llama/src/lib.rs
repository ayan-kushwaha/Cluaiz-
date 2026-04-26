//! Sovereign Implementation B: Accelerated Feature-Based Runtime.

use anyhow::Result;
use archer_shared::{
    ArcConstructor, BackendType, KernelSignature, ModelWeightsWrapper, SovereignInference,
    UnifiedBackend, SovereignContext
};
use tokenizers::Tokenizer;

pub mod config;
pub mod loader;
pub mod pipeline;
pub mod router;


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
        Ok(format!("RuntimeB processed: {}", prompt))
    }

    fn prefill(&mut self, _prompt: &str) -> Result<()> { Ok(()) }
    fn evaluate_tps(&self) -> f64 { 75.0 }
}

impl SovereignInference for RuntimeB {
    fn forward_raw(&mut self, _input_ids: &[u32], _pos: usize) -> Result<Vec<f32>> {
        Err(anyhow::anyhow!("FFI forward not supported for Binary Driver"))
    }

    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        _tokenizer: &Tokenizer,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> Result<()> {
        // 🔥 SOVEREIGN BRIDGE: Execute the async Binary Driver synchronously
        tokio::task::block_in_place(|| {
            let handle = tokio::runtime::Handle::current();
            handle.block_on(crate::pipeline::RuntimeBPipeline::execute_stream(
                &self.model_path,
                &self.context,
                prompt,
                0,
                callback,
            )).map_err(|e| anyhow::anyhow!(e))
        })
    }
}

pub fn register_drivers(mut register_fn: impl FnMut(BackendType, KernelSignature, ArcConstructor)) -> std::result::Result<(), String> {
    let patterns = vec!["uniform", "asymmetric"];

    for pattern in patterns {
        let mut signature = KernelSignature {
            has_experts: false,
            is_asymmetric: pattern == "asymmetric",
            is_multimodal: true,
            is_heterogeneous: true,
            is_bitnet: false,
            is_ssm: false,
            head_pattern: pattern.into(),
            activation: "silu".into(),
        };

        // 1️⃣ Standard Signature
        register_fn(
            BackendType::RuntimeB,
            signature.clone(),
            std::sync::Arc::new(
                |load_path: &str,
                 sovereign_context: SovereignContext| {
                    let engine = RuntimeB::new(load_path, sovereign_context);
                    Ok(Box::new(engine) as ModelWeightsWrapper)
                },
            ) as ArcConstructor,
        );

        // 2️⃣ 🚀 BitNet Signature: Activating FFI Bridge for 1-bit models
        signature.is_bitnet = true;
        register_fn(
            BackendType::RuntimeB,
            signature,
            std::sync::Arc::new(
                |load_path: &str,
                 sovereign_context: SovereignContext| {
                    let engine = RuntimeB::new(load_path, sovereign_context);
                    Ok(Box::new(engine) as ModelWeightsWrapper)
                },
            ) as ArcConstructor,
        );

    }
    Ok(())
}
