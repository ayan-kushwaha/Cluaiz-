//! router.rs: The Neural Dispatcher.
//! Routes prompts to the appropriate backend based on model architecture.

use std::path::PathBuf;
use crate::utils::healer::AutoHealer;
use archer_shared::{UnifiedBackend, BackendType, SovereignContext, StructuralDNA, TemplateManager, ModelWeightsWrapper};
use crate::runtime::execution::hub::SiliconOrchestrator;
use candle_core::Device;

pub enum Backend {
    Empty(DummyBackend),
    Sovereign(ModelWeightsWrapper),
}

impl UnifiedBackend for Backend {
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        match self {
            Self::Empty(b) => b.generate(prompt, max_tokens),
            Self::Sovereign(b) => b.generate(prompt, max_tokens),
        }
    }
    fn prefill(&mut self, prompt: &str) -> anyhow::Result<()> {
        match self {
            Self::Empty(b) => b.prefill(prompt),
            Self::Sovereign(b) => b.prefill(prompt),
        }
    }

    fn evaluate_tps(&self) -> f64 {
        match self {
            Self::Empty(b) => b.evaluate_tps(),
            Self::Sovereign(b) => b.evaluate_tps(),
        }
    }
}

impl archer_shared::SovereignInference for Backend {
    fn forward_raw(&mut self, inputs: &[u32], pos: usize) -> anyhow::Result<Vec<f32>> {
        match self {
            Self::Sovereign(b) => b.forward_raw(inputs, pos),
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
            Self::Sovereign(b) => b.generate_stream(prompt, max_tokens, tokenizer, callback),
            Self::Empty(_) => Err(anyhow::anyhow!("Empty backend")),
        }
    }
}

pub struct NeuralRouter {
    pub active_backend: Backend,
    pub tokenizer: Option<tokenizers::Tokenizer>,
    pub foundry: crate::neural_foundry::NeuralFoundry,
}

impl Default for NeuralRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl NeuralRouter {
    pub fn new() -> Self {
        Self { 
            active_backend: Backend::Empty(DummyBackend),
            tokenizer: None,
            foundry: crate::neural_foundry::NeuralFoundry::new(),
        }
    }

    pub async fn load_model(path: PathBuf, runtime: BackendType, _device: &Device) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            let repo_id = path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
            let _ = AutoHealer::heal_missing_tokenizer(&repo_id, parent).await;
        }

        // [SOVEREIGN ALIGNMENT]: Bootstrapping context with default DNA and Templates
        let mut dna = StructuralDNA::default();
        dna.preferred_runtime = Some(runtime);
        
        let context = SovereignContext::boot(
            dna,
            TemplateManager::default(),
        );

        // 🚀 THE SOVEREIGN HANDSHAKE: Dispatching to the Dynamic Linker
        println!("🧬 [Router] Dispatching to SiliconOrchestrator for dynamic linkage...");
        let engine = SiliconOrchestrator::instantiate(&path.to_string_lossy(), context)
            .await
            .map_err(|e| format!("Sovereign Handshake Failure: {}", e))?;

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
            println!("🗣️ [Router] Voice initialization fail: {}", err);
        }

        let mut foundry = crate::neural_foundry::NeuralFoundry::new();
        // Load skills from a standard location (this could be configurable)
        foundry.initialize("skills");

        Ok(Self { 
            active_backend: Backend::Sovereign(engine), 
            tokenizer,
            foundry 
        })
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
        // 🧪 SOVEREIGN HANDSHAKE: Check for skills before generation
        let rt = tokio::runtime::Handle::current();
        let intent_result = rt.block_on(self.foundry.process_intent(prompt))
            .map_err(|e| format!("Skill Discovery Error: {}", e))?;

        match &mut self.active_backend {
            Backend::Sovereign(b) => {
                // If neural signals (skill souls) were identified, inject them into the kernel
                if !intent_result.signals.is_empty() {
                    println!("💉 [Router] Injecting {} neural signals into active backend...", intent_result.signals.len());
                    b.inject_signals(intent_result.signals).map_err(|e| format!("Signal Injection Failure: {}", e))?;
                }

                if let Some(ref tokenizer) = self.tokenizer {
                    b.generate_stream(prompt, max_tokens, tokenizer, callback)
                        .map_err(|e| e.to_string())
                } else {
                    Err("Tokenizer not loaded.".to_string())
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
