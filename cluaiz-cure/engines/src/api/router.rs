//! router.rs: The Neural Dispatcher.
//! Routes prompts to the appropriate backend based on model architecture.

use std::path::PathBuf;
use crate::utils::healer::AutoHealer;
use archer_shared::{UnifiedBackend, BackendType, SovereignContext, StructuralDNA, TemplateManager, KernelSignature};
use archer_llama::RuntimeB as RuntimeBImplementation;
use archer_candle::CandleEngine as EngineA;
use candle_core::Device;
use tracing::{info, warn, error};


pub enum Backend {
    Empty(DummyBackend),
    RuntimeA(EngineA),
    RuntimeB(RuntimeBImplementation),
    // RuntimeC: BitNet is loaded dynamically via libloading at runtime.
    // See: crate::runtime::execution::provisioner for dynamic loading.
}

impl UnifiedBackend for Backend {
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        match self {
            Self::Empty(b) => b.generate(prompt, max_tokens),
            Self::RuntimeA(b) => b.generate(prompt, max_tokens),
            Self::RuntimeB(b) => b.generate(prompt, max_tokens),
        }
    }
    fn prefill(&mut self, prompt: &str) -> anyhow::Result<()> {
        match self {
            Self::Empty(b) => b.prefill(prompt).map_err(|e| anyhow::anyhow!(e)),
            Self::RuntimeA(b) => b.prefill(prompt).map_err(|e| anyhow::anyhow!(e)),
            Self::RuntimeB(b) => b.prefill(prompt).map_err(|e| anyhow::anyhow!(e)),
        }
    }

    fn evaluate_tps(&self) -> f64 {
        match self {
            Self::Empty(b) => b.evaluate_tps(),
            Self::RuntimeA(b) => b.evaluate_tps(),
            Self::RuntimeB(b) => b.evaluate_tps(),
        }
    }
}

impl archer_shared::SovereignInference for Backend {
    fn forward_raw(&mut self, inputs: &[u32], pos: usize) -> anyhow::Result<Vec<f32>> {
        match self {
            Self::RuntimeA(b) => b.forward_raw(inputs, pos),
            Self::RuntimeB(b) => b.forward_raw(inputs, pos),
            Self::Empty(_) => Err(anyhow::anyhow!("Empty backend")),
        }
    }

    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        tokenizer: &tokenizers::Tokenizer,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> anyhow::Result<()> {
        match self {
            Self::RuntimeA(b) => b.generate_stream(prompt, max_tokens, tokenizer, callback).map_err(|e| anyhow::anyhow!(e)),
            Self::RuntimeB(b) => b.generate_stream(prompt, max_tokens, tokenizer, callback).map_err(|e| anyhow::anyhow!(e)),
            Self::Empty(_) => Err(anyhow::anyhow!("Empty backend")),
        }
    }

}

pub struct NeuralRouter {
    pub active_backend: Backend,
    pub tokenizer: Option<tokenizers::Tokenizer>,
}

impl NeuralRouter {
    pub fn new() -> Self {
        Self { 
            active_backend: Backend::Empty(DummyBackend),
            tokenizer: None,
        }
    }

    pub async fn load_model(path: PathBuf, runtime: BackendType, device: &Device) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            let repo_id = path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
            let _ = AutoHealer::heal_missing_tokenizer(&repo_id, parent).await;
        }

        let context = SovereignContext::boot(
            StructuralDNA {
                model_identity: "llama-default-v3".to_string(),
                layer_count: Some(32),
                hidden_size: Some(4096),
                attention_head_count: Some(32),
                attention_head_count_kv: Some(32),
                attention_head_dim: Some(128),
                intermediate_size: Some(11008),
                attention_dimensionality_truth: Some(4096),
                signature: KernelSignature {
                    has_experts: false,
                    is_asymmetric: false,
                    is_multimodal: false,
                    is_heterogeneous: false,
                    is_bitnet: false,
                    is_ssm: false,
                    head_pattern: "uniform".to_string(),
                    activation: "silu".to_string(),
                },
                preferred_runtime: Some(runtime.clone()), 
                heterogeneous_map: None,
                dynamic_attributes: std::collections::HashMap::new(),
            },

            TemplateManager {
                jinja_template: "".to_string(),
                is_fallback: true,
            },
        );

        // 🏛️ SOVEREIGN ARCHITECTURAL OVERRIDE: 
        // 1-bit models (BitNet) MUST use RuntimeC (Engine C) for native performance.
        let final_runtime = if context.dna.signature.is_bitnet {
            info!("🧬 [Router] BitNet Signature Detected. Dispatched to Engine C (Native).");
            BackendType::RuntimeC
        } else {
            runtime
        };

        let engine = match final_runtime {
            BackendType::RuntimeA => Backend::RuntimeA(
                EngineA::from_gguf_with_dna_auto_file(path.clone(), device, Some(context.dna))
                    .map_err(|e| e.to_string())?,
            ),

            BackendType::RuntimeB | BackendType::RuntimeC => {
                // 🛰️ BINARY PRE-FLIGHT: Validate and Provision execution core on-demand
                let os = if cfg!(windows) { "windows" } else { "linux" };
                let driver = if matches!(final_runtime, BackendType::RuntimeC) {
                    archer_shared::hardware::schema::BackendDriver::CPU
                } else {
                    let truth = archer_shared::hardware::get_silicon_state();
                    if truth.active_drivers.iter().any(|d| d.driver_id == "CUDA") {
                        archer_shared::hardware::schema::BackendDriver::CUDA
                    } else if truth.active_drivers.iter().any(|d| d.driver_id == "METAL") {
                        archer_shared::hardware::schema::BackendDriver::METAL
                    } else {
                        archer_shared::hardware::schema::BackendDriver::CPU
                    }
                };

                let binary_path = tokio::task::block_in_place(|| {
                    let handle = tokio::runtime::Handle::current();
                    handle.block_on(crate::runtime::execution::provisioner::BinaryProvisioner::ensure_binary(os, &driver, &PathBuf::new()))
                });

                match binary_path {
                    Ok(_bin_p) => {
                        if matches!(final_runtime, BackendType::RuntimeB) {
                            // RuntimeB early return — it handles its own tokenization.
                            return Ok(Self { active_backend: Backend::RuntimeB(RuntimeBImplementation::new(&path.to_string_lossy(), context)), tokenizer: None });
                        }
                        // BitNet (RuntimeC): dynamic loading stub — fall through to error.
                        return Err("RuntimeC (BitNet) dynamic loading not yet provisioned.".to_string());
                    },
                    Err(e) => {
                        return Err(format!("FATAL: Neural Core could not be provisioned. Registry Error: {}", e));
                    }
                }
            }

            _ => {
                return Err(format!("Runtime {:?} is not yet implemented.", final_runtime));
            }
        };


        let (tokenizer, t_error) = if let Some(p) = path.parent() {
            let t_path = p.join("tokenizer.json");
            if t_path.exists() {
                match tokenizers::Tokenizer::from_file(&t_path) {
                    Ok(t) => (Some(t), None),
                    Err(e) => (None, Some(format!("Tokenizer found but failed to parse: {}", e))),
                }
            } else {
                (None, Some(format!("tokenizer.json missing at {:?}", t_path)))
            }
        } else {
            (None, Some("Invalid model path parent.".to_string()))
        };

        if let Some(err) = t_error {
            tracing::error!("🗣️ [Router] Voice initialization fail: {}", err);
            // We still proceed if model is loaded, but generation might fail later
        } else {
            tracing::info!("🗣️ [Router] Neural Vocal Cords (Tokenizer) mounted successfully.");
        }

        Ok(Self { active_backend: engine, tokenizer })
    }

    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        self.active_backend.generate(prompt, max_tokens)
    }

    pub fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> Result<(), String> {
        match &mut self.active_backend {
            Backend::RuntimeB(b) => {
                use archer_shared::SovereignInference;
                let dummy_tokenizer = tokenizers::Tokenizer::from_bytes(&[]).unwrap_or_else(|_| tokenizers::Tokenizer::new(tokenizers::models::bpe::BPE::default()));
                b.generate_stream(prompt, max_tokens, &dummy_tokenizer, callback)
                    .map_err(|e| e.to_string())
            },
            Backend::RuntimeA(b) => {
                if let Some(ref tokenizer) = self.tokenizer {
                    use archer_shared::SovereignInference;
                    b.generate_stream(prompt, max_tokens, tokenizer, callback)
                        .map_err(|e| e.to_string())
                } else {
                    Err("Tokenizer not loaded for RuntimeA.".to_string())
                }
            },
            Backend::Empty(_) => Err("Neural weights not loaded.".to_string()),
        }
    }
}

pub struct DummyBackend;
impl archer_shared::UnifiedBackend for DummyBackend {
    fn generate(&mut self, _prompt: &str, _max_tokens: usize) -> Result<String, String> {
        Err("Neural weights not loaded.".to_string())
    }
    fn prefill(&mut self, _prompt: &str) -> anyhow::Result<()> { Ok(()) }

    fn evaluate_tps(&self) -> f64 { 0.0 }
}

impl archer_shared::SovereignInference for DummyBackend {
    fn forward_raw(&mut self, _inputs: &[u32], _pos: usize) -> anyhow::Result<Vec<f32>> {
        Err(anyhow::anyhow!("Dummy backend"))
    }
    fn generate_stream(
        &mut self,
        _prompt: &str,
        _max_tokens: usize,
        _tokenizer: &tokenizers::Tokenizer,
        _callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Dummy backend"))
    }
}


