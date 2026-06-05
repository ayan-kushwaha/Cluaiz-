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
    
    fn embed(&mut self, input: &str) -> anyhow::Result<Vec<f32>> {
        match self {
            Self::Empty(b) => b.embed(input),
            Self::Cluaiz(b) => b.embed(input),
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
        callback: Box<dyn FnMut(String) -> bool + Send + 'static>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Cluaiz(b) => b.generate_stream(prompt, max_tokens, callback),
            Self::Empty(_) => Err(anyhow::anyhow!("Empty backend")),
        }
    }

    fn dump_kv_cache(&mut self, path: &str) -> anyhow::Result<()> {
        match self {
            Self::Cluaiz(b) => b.dump_kv_cache(path),
            Self::Empty(_) => Err(anyhow::anyhow!("Empty backend")),
        }
    }

    fn load_kv_cache(&mut self, path: &str) -> anyhow::Result<()> {
        match self {
            Self::Cluaiz(b) => b.load_kv_cache(path),
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
        let engine = HardwareOrchestrator::instantiate(&path.to_string_lossy(), "llama", context)
            .await
            .map_err(|e| format!("Cluaiz Handshake Failure: {}", e))?;



        let mut foundry = crate::neural_foundry::CoreFoundry::new();
        // Load skills from the global ~/.cluaiz/skills directory
        let skills_dir = dirs::home_dir().unwrap_or_default().join(".cluaiz").join("skills");
        foundry.initialize(&skills_dir.to_string_lossy());

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
        let formatted_prompt = prompt.to_string(); // Let native engines handle formatting
        self.active_backend.generate(&formatted_prompt, max_tokens)
    }

    pub fn generate_stream(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        callback: Box<dyn FnMut(String) -> bool + Send + 'static>,
    ) -> Result<(), String> {
        let mut prompt_embedding_engine = None;
        let schema = crate::neural_foundry::security::permission_schema::PermissionSchema::load();
        println!("🤖 [Debug-Router] Active embedding model ID in schema: {:?}", schema.vector_models.text);
        if let Some(text_model_id) = &schema.vector_models.text {
            let roster = crate::models::registry::CoreRoster::load_roster();
            println!("🤖 [Debug-Router] Roster size: {}, models: {:?}", roster.len(), roster.iter().map(|m| &m.id).collect::<Vec<_>>());
            if let Some(manifest) = roster.iter().find(|m| &m.id == text_model_id) {
                println!("🤖 [Debug-Router] Found manifest for embedding model. Local path: {:?}", manifest.local_path);
                if let Some(local_path) = &manifest.local_path {
                    let model_dir = std::path::Path::new(local_path);
                    let model_file = model_dir.join("model.onnx");
                    let tokenizer_file = model_dir.join("tokenizer.json");
                    println!("🤖 [Debug-Router] Checking files: model.onnx exists: {}, tokenizer.json exists: {}", model_file.exists(), tokenizer_file.exists());
                    if model_file.exists() && tokenizer_file.exists() {
                        match cluaiz_onnx::engine::OnnxEngine::new() {
                            Ok(mut engine) => {
                                match engine.load_text_model(&model_file.to_string_lossy(), &tokenizer_file.to_string_lossy()) {
                                    Ok(_) => {
                                        println!("🤖 [Debug-Router] Embedding engine loaded successfully!");
                                        prompt_embedding_engine = Some(engine);
                                    }
                                    Err(e) => println!("❌ [Debug-Router] Failed to load text model: {:?}", e),
                                }
                            }
                            Err(e) => println!("❌ [Debug-Router] Failed to instantiate OnnxEngine: {:?}", e),
                        }
                    }
                }
            } else {
                println!("❌ [Debug-Router] Embedding model ID not found in roster!");
            }
        }

        let mut matched_skill_path = None;
        if let Some(mut engine) = prompt_embedding_engine {
            // Dynamic compilation of missing or mismatched semantic vectors
            if let Ok(mut router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.write() {
                let _ = router.boot_index();
                if let Some(active_model_id) = schema.get_active_embedding_model() {
                    let safe_filename = active_model_id.replace(":", "-");
                    let mut new_vectors = Vec::new();
                    for (id, manifest) in &router.loaded_manifests {
                        let home_dir = dirs::home_dir().unwrap_or_default();
                        let skill_path = home_dir.join(".cluaiz").join("skills").join(&manifest.name);
                        let cache_dir = skill_path.join(".cache");
                        let emb_path = cache_dir.join(format!("{}.emb.bin", safe_filename));
                        let has_vector = router.skill_vectors.contains_key(&skill_path);

                        if !has_vector || !emb_path.exists() {
                            println!("⏳ [Sovereign-Ops] Mismatch detected. Generating semantic vector for skill: {}", manifest.name);
                            let skill_content = if let Some(fm) = extract_frontmatter(&skill_path) {
                                fm
                            } else {
                                let semantic_triggers = manifest.triggers.semantic.join(", ");
                                format!(
                                    "Skill Name: {}\nDescription: {}\nTriggers: {}",
                                    manifest.name, manifest.description, semantic_triggers
                                )
                            };
                            use neural_core::interfaces::router_contract::EmbeddingDriver;
                            if let Ok(vec) = engine.gen_embedding(&skill_content) {
                                let _ = std::fs::create_dir_all(&cache_dir);
                                let data_bytes = unsafe { std::slice::from_raw_parts(vec.as_ptr() as *const f32 as *const u8, vec.len() * 4) };
                                if let Ok(_) = std::fs::write(&emb_path, data_bytes) {
                                    new_vectors.push((skill_path.clone(), vec));
                                }
                            }
                        }
                    }
                    for (p, v) in new_vectors {
                        router.skill_vectors.insert(p, v);
                    }
                }
            }

            use neural_core::interfaces::router_contract::EmbeddingDriver;
            if let Ok(vector) = engine.gen_embedding(prompt) {
                if let Ok(router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                    println!("🤖 [Debug-Router] Checking semantic triggers. Active vector dim: {}, loaded skill vectors: {:?}", vector.len(), router.skill_vectors.keys());
                    for (path, skill_vec) in &router.skill_vectors {
                        let mut dot = 0.0;
                        let mut mag_a = 0.0;
                        let mut mag_b = 0.0;
                        for (a, b) in vector.iter().zip(skill_vec.iter()) {
                            dot += a * b;
                            mag_a += a * a;
                            mag_b += b * b;
                        }
                        let score = dot / (mag_a.sqrt() * mag_b.sqrt());
                        println!("🤖 [Debug-Router] Cosine similarity with {:?}: {:.4}", path.file_name().unwrap_or_default(), score);
                    }
                    matched_skill_path = router.check_semantic_trigger(&vector, 0.33); // 33% threshold for stable matching
                    println!("🤖 [Debug-Router] Matched skill path: {:?}", matched_skill_path);
                }
            }
        }

        if matched_skill_path.is_none() {
            if let Ok(router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                matched_skill_path = router.check_trigger(prompt);
            }
        }

        // 🧪 Cluaiz HANDSHAKE: Check for skills before generation
        let rt = tokio::runtime::Handle::current();
        let intent_result = rt.block_on(self.foundry.process_intent(prompt))
            .map_err(|e| format!("Skill Discovery Error: {}", e))?;

        match &mut self.active_backend {
            Backend::Cluaiz(b) => {
                // Dynamic compilation of missing or mismatched KV Cache for the matched skill
                if let Some(ref skill_path) = matched_skill_path {
                    let schema = crate::neural_foundry::security::permission_schema::PermissionSchema::load();
                    if let Some(active_chat_model) = schema.get_active_chat_model() {
                        let gen_model_safe = active_chat_model.replace(":", "-");
                        let cache_dir = skill_path.join(".cache");
                        let kv_cache_path = cache_dir.join(format!("{}.kvcache.bin", gen_model_safe));
                        if !kv_cache_path.exists() {
                            if let Some(frontmatter) = extract_frontmatter(skill_path) {
                                let prefix = format!("[System Memory Injection (Frontmatter): {}]\n", frontmatter);
                                println!("⏳ [Sovereign-Ops] First time trigger: Compiling KV Cache for skill: {}...", skill_path.file_name().unwrap_or_default().to_string_lossy());
                                use cluaiz_shared::UnifiedBackend;
                                if let Err(e) = b.prefill(&prefix) {
                                    println!("❌ [Sovereign-Ops] Failed to prefill and compile KV Cache: {}", e);
                                } else {
                                    use cluaiz_shared::CluaizInference;
                                    let path_str = kv_cache_path.to_string_lossy().to_string();
                                    let _ = std::fs::create_dir_all(&cache_dir);
                                    if let Err(e) = b.dump_kv_cache(&path_str) {
                                        println!("❌ [Sovereign-Ops] Failed to dump KV Cache: {}", e);
                                    } else {
                                        println!("✅ [Sovereign-Ops] KV Cache compiled successfully!");
                                    }
                                }
                            }
                        } else {
                            let path_str = kv_cache_path.to_string_lossy().to_string();
                            println!("🧠 [Sovereign-Ops] Restoring KV Cache for skill: {}", skill_path.file_name().unwrap_or_default().to_string_lossy());
                            if let Err(e) = b.load_kv_cache(&path_str) {
                                println!("❌ [Sovereign-Ops] Failed to load KV cache: {}", e);
                            }
                        }
                    }
                }

                // If Core signals (skill souls) were identified, inject them into the kernel
                if !intent_result.signals.is_empty() {
                    println!("💉 [Router] Injecting {} Core signals into active backend...", intent_result.signals.len());
                    b.inject_signals(intent_result.signals).map_err(|e| format!("Signal Injection Failure: {}", e))?;
                }

                // 🎭 Orchestration: Let native backend handle templating to support pivot tags
                let mut formatted_prompt = prompt.to_string();
                
                // 🧠 INJECT SKILL SYSTEM PROMPT: If the foundry provided textual skill descriptions,
                // we must append them to the system prompt so the LLM is aware of its tools.
                if !intent_result.responses.is_empty() {
                    let tool_context = intent_result.responses.join("\n\n");
                    formatted_prompt = format!(
                        "<system>\nYou have access to the following skills/tools. Read their documentation carefully and use them if needed to fulfill the user's request. Output valid JSON to use a tool if instructed by the skill documentation:\n\n{}\n</system>\n\n{}", 
                        tool_context, 
                        formatted_prompt
                    );
                    println!("🧠 [Router] Injected skill descriptions into LLM context.");
                }

                b.generate_stream(&formatted_prompt, max_tokens, callback)
                    .map_err(|e| e.to_string())
            },
            Backend::Empty(_) => Err("Core weights not loaded. Please select a model with @ or wait for the Auto-Pilot handshake to complete.".to_string()),
        }
    }
    
    pub fn embed(&mut self, input: &str) -> Result<Vec<f32>, String> {
        self.active_backend.embed(input).map_err(|e| e.to_string())
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
        _callback: Box<dyn FnMut(String) -> bool + Send + 'static>,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Dummy backend"))
    }

    fn load_kv_cache(&mut self, _path: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Dummy backend"))
    }
}

fn extract_frontmatter(skill_dir: &std::path::Path) -> Option<String> {
    let skill_md_path = skill_dir.join("SKILL.md");
    if skill_md_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&skill_md_path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut start_idx = None;
            let mut end_idx = None;
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "---" {
                    if start_idx.is_none() {
                        start_idx = Some(i);
                    } else {
                        end_idx = Some(i);
                        break;
                    }
                }
            }
            if let (Some(start), Some(end)) = (start_idx, end_idx) {
                if end > start + 1 {
                    let frontmatter_lines = &lines[start + 1..end];
                    return Some(frontmatter_lines.join("\n"));
                }
            }
        }
    }
    None
}
