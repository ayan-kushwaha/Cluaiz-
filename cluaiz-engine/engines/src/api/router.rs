//! router.rs: The Core Dispatcher.
//! Routes prompts to the appropriate backend based on model architecture.

use std::path::PathBuf;
use crate::utils::healer::AutoHealer;
use cluaiz_shared::{UnifiedBackend, BackendType, CluaizContext, StructuralDNA, TemplateManager, ModelWeightsWrapper};
use crate::runtime::execution::hub::HardwareOrchestrator;

pub enum Backend {
    Empty(DummyBackend),
    Cluaiz(ModelWeightsWrapper),
}

impl UnifiedBackend for Backend {
    fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        match self {
            Self::Empty(b) => b.generate(prompt, max_tokens),
            Self::Cluaiz(b) => b.generate(prompt, max_tokens),
        }
    }
    fn prefill(&mut self, prompt: &str) -> anyhow::Result<()> {
        match self {
            Self::Empty(b) => b.prefill(prompt),
            Self::Cluaiz(b) => b.prefill(prompt),
        }
    }

    fn evaluate_tps(&self) -> f64 {
        match self {
            Self::Empty(b) => b.evaluate_tps(),
            Self::Cluaiz(b) => b.evaluate_tps(),
        }
    }
}

impl cluaiz_shared::CluaizInference for Backend {
    fn forward_raw(&mut self, inputs: &[u32], pos: usize) -> anyhow::Result<Vec<f32>> {
        match self {
            Self::Cluaiz(b) => b.forward_raw(inputs, pos),
            Self::Empty(_) => Err(anyhow::anyhow!("Empty backend")),
        }
    }

    fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Cluaiz(b) => b.generate_stream(prompt, max_tokens, callback),
            Self::Empty(_) => Err(anyhow::anyhow!("Empty backend")),
        }
    }
}

pub struct CoreRouter {
    pub active_backend: Backend,

    pub foundry: crate::neural_foundry::CoreFoundry,
    pub active_dna: Option<cluaiz_shared::StructuralDNA>,
}

impl Default for CoreRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreRouter {
    pub fn new() -> Self {
        Self { 
            active_backend: Backend::Empty(DummyBackend),

            foundry: crate::neural_foundry::CoreFoundry::new(),
            active_dna: None,
        }
    }

    pub async fn load_model(path: PathBuf, runtime: BackendType) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            let mut repo_id = path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default().to_string();
            let manifest_path = parent.join("model_manifest.json");
            if manifest_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<crate::models::registry::ModelManifest>(&content) {
                        if manifest.download_url.contains("huggingface.co/") {
                            repo_id = manifest.download_url
                                .split("huggingface.co/")
                                .nth(1)
                                .unwrap_or("")
                                .split("/resolve")
                                .next()
                                .unwrap_or(&repo_id)
                                .to_string();
                        }
                    }
                }
            }
            let _ = AutoHealer::heal_missing_tokenizer(&repo_id, parent).await;
        }

        // [Cluaiz ALIGNMENT]: Bootstrapping context with local DNA and Templates
        let mut dna = StructuralDNA::default();
        if let Some(parent) = path.parent() {
            let dna_path = parent.join("structural_dna.json");
            if dna_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&dna_path) {
                    if let Ok(loaded_dna) = serde_json::from_str::<StructuralDNA>(&content) {
                        dna = loaded_dna;
                        println!("🧬 [Router] Neural DNA synchronized from local manifest.");
                    }
                }
            }
            // 🧬 Deep Discovery: Learn and Repair from tokenizer_config.json, etc.
            dna.discover_from_path(parent)
                .map_err(|e| format!("Neural Discovery Failure: {}", e))?;
        }
        dna.preferred_runtime = Some(runtime);
        
        let context = CluaizContext::boot(
            dna.clone(),
            TemplateManager::default(),
        );

        // 🚀 THE Cluaiz HANDSHAKE: Dispatching to the Dynamic Linker
        println!("🧬 [Router] Dispatching to HardwareOrchestrator for dynamic linkage...");
        let engine = HardwareOrchestrator::instantiate(&path.to_string_lossy(), context)
            .await
            .map_err(|e| format!("Cluaiz Handshake Failure: {}", e))?;



        let mut foundry = crate::neural_foundry::CoreFoundry::new();
        // Load skills from a standard location (this could be configurable)
        foundry.initialize("skills");

        Ok(Self { 
            active_backend: Backend::Cluaiz(engine), 

            foundry,
            active_dna: Some(dna),
        })
    }

    pub fn get_active_dna(&self) -> Option<&cluaiz_shared::StructuralDNA> {
        self.active_dna.as_ref()
    }

    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let formatted_prompt = if let Some(ref dna) = self.active_dna {
            let tm = cluaiz_shared::TemplateManager::default();
            tm.format(dna, prompt)
        } else {
            prompt.to_string()
        };
        self.active_backend.generate(&formatted_prompt, max_tokens)
    }

    pub fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> Result<(), String> {
        // 🧪 Cluaiz HANDSHAKE: Check for skills before generation
        let rt = tokio::runtime::Handle::current();
        let intent_result = rt.block_on(self.foundry.process_intent(prompt))
            .map_err(|e| format!("Skill Discovery Error: {}", e))?;

        match &mut self.active_backend {
            Backend::Cluaiz(b) => {
                // If Core signals (skill souls) were identified, inject them into the kernel
                if !intent_result.signals.is_empty() {
                    println!("💉 [Router] Injecting {} Core signals into active backend...", intent_result.signals.len());
                    b.inject_signals(intent_result.signals).map_err(|e| format!("Signal Injection Failure: {}", e))?;
                }

                // 🎭 Orchestration: Format prompt based on model DNA
                let formatted_prompt = if let Some(ref dna) = self.active_dna {
                    let tm = cluaiz_shared::TemplateManager::default();
                    tm.format(dna, prompt)
                } else {
                    prompt.to_string()
                };

                b.generate_stream(&formatted_prompt, max_tokens, callback)
                    .map_err(|e| e.to_string())
            },
            Backend::Empty(_) => Err("Core weights not loaded. Please select a model with @ or wait for the Auto-Pilot handshake to complete.".to_string()),
        }
    }
}

pub struct DummyBackend;
impl cluaiz_shared::UnifiedBackend for DummyBackend {
    fn generate(&mut self, _prompt: &str, _max_tokens: usize) -> Result<String, String> {
        Err("Core weights not loaded.".to_string())
    }
    fn prefill(&mut self, _prompt: &str) -> anyhow::Result<()> { Ok(()) }
    fn evaluate_tps(&self) -> f64 { 0.0 }
}

impl cluaiz_shared::CluaizInference for DummyBackend {
    fn forward_raw(&mut self, _inputs: &[u32], _pos: usize) -> anyhow::Result<Vec<f32>> {
        Err(anyhow::anyhow!("Dummy backend"))
    }
    fn generate_stream(
        &mut self,
        _prompt: &str,
        _max_tokens: usize,
        _callback: Box<dyn FnMut(String) + Send + 'static>,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Dummy backend"))
    }
}
