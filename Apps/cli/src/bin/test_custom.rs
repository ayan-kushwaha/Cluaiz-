use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🧪 [Test] Starting Standalone Dynamic Pipeline Diagnostic...");

    let home_dir = dirs::home_dir().expect("Could not resolve Home Directory");
    let model_path = home_dir.join(".cluaiz").join("models").join("chat").join("bonsai1-8b").join("Bonsai-8B.gguf");
    
    if !model_path.exists() {
        println!("❌ Model not found at: {:?}", model_path);
        return Ok(());
    }

    // Set Permission.json text chat model to bonsai1:8b
    engines::neural_foundry::security::permission_schema::PermissionSchema::set_active_chat_model("bonsai1:8b".to_string());

    let dna = cluaiz_shared::StructuralDNA::default();
    let context = cluaiz_shared::CluaizContext::boot(dna, cluaiz_shared::TemplateManager::default());

    println!("⚙️ [Test] Instantiating Chat Engine (Bonsai)...");
    let mut engine = engines::runtime::execution::hub::HardwareOrchestrator::instantiate(
        model_path.to_str().unwrap(),
        "llama",
        context
    ).await?;

    let prompt = "Make a sad piano instrumental track with slow tempo and emotional vibe";
    println!("🚀 [Test] Triggering stream with prompt: '{}'", prompt);

    // Load active router
    let mut router = engines::api::router::CoreRouter::new();
    let skills_dir = home_dir.join(".cluaiz").join("skills");
    router.foundry.initialize(&skills_dir.to_string_lossy());
    router.active_backend = engines::api::router::Backend::Cluaiz(engine);

    // DEBUG: Let's inspect the Global Skill Router manually
    {
        let schema = engines::neural_foundry::security::permission_schema::PermissionSchema::load();
        println!("🤖 [Debug] PermissionSchema active embedding model: {:?}", schema.get_active_embedding_model());
        println!("🤖 [Debug] PermissionSchema active chat model: {:?}", schema.get_active_chat_model());

        if let Ok(mut g_router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.write() {
            println!("🤖 [Debug] Booting Global Skill Router index...");
            if let Err(e) = g_router.boot_index() {
                println!("❌ [Debug] boot_index failed: {:?}", e);
            }
            println!("🤖 [Debug] Loaded manifests: {:?}", g_router.loaded_manifests.keys());
            println!("🤖 [Debug] Keyword index size: {}", g_router.keyword_index.len());
            println!("🤖 [Debug] Loaded skill vectors: {:?}", g_router.skill_vectors.keys());
        }

        // Test embedding generation manually
        if let Some(text_model_id) = &schema.vector_models.text {
            let roster = engines::models::registry::CoreRoster::load_roster();
            if let Some(manifest) = roster.iter().find(|m| &m.id == text_model_id) {
                if let Some(local_path) = &manifest.local_path {
                    let model_dir = std::path::Path::new(local_path);
                    let model_file = model_dir.join("model.onnx");
                    let tokenizer_file = model_dir.join("tokenizer.json");
                    if model_file.exists() && tokenizer_file.exists() {
                        if let Ok(mut emb_engine) = cluaiz_onnx::engine::OnnxEngine::new() {
                            if emb_engine.load_text_model(&model_file.to_string_lossy(), &tokenizer_file.to_string_lossy()).is_ok() {
                                println!("🤖 [Debug] Embedding engine loaded successfully!");
                                use neural_core::interfaces::router_contract::EmbeddingDriver;
                                if let Ok(prompt_vec) = emb_engine.gen_embedding(prompt) {
                                    if let Ok(g_router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                                        for (path, skill_vec) in &g_router.skill_vectors {
                                            let mut dot = 0.0;
                                            let mut mag_a = 0.0;
                                            let mut mag_b = 0.0;
                                            for (a, b) in prompt_vec.iter().zip(skill_vec.iter()) {
                                                dot += a * b;
                                                mag_a += a * a;
                                                mag_b += b * b;
                                            }
                                            let score = dot / (mag_a.sqrt() * mag_b.sqrt());
                                            println!("🤖 [Debug] Similarity with {:?}: {:.4}", path.file_name().unwrap(), score);
                                        }
                                    }
                                }
                                if let Ok(unrelated_vec) = emb_engine.gen_embedding("What is the capital of France?") {
                                    if let Ok(g_router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                                        for (path, skill_vec) in &g_router.skill_vectors {
                                            let mut dot = 0.0;
                                            let mut mag_a = 0.0;
                                            let mut mag_b = 0.0;
                                            for (a, b) in unrelated_vec.iter().zip(skill_vec.iter()) {
                                                dot += a * b;
                                                mag_a += a * a;
                                                mag_b += b * b;
                                            }
                                            let score = dot / (mag_a.sqrt() * mag_b.sqrt());
                                            println!("🤖 [Debug] Unrelated Similarity with {:?}: {:.4}", path.file_name().unwrap(), score);
                                        }
                                    }
                                }
                            } else {
                                println!("❌ [Debug] Failed to load embedding model weights");
                            }
                        } else {
                            println!("❌ [Debug] Failed to instantiate OnnxEngine");
                        }
                    }
                }
            }
        }
    }

    let prompt_str = prompt.to_string();
    let res = tokio::task::block_in_place(|| {
        router.generate_stream(
            &prompt_str,
            5,
            Box::new(|token| {
                print!("{}", token);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                true
            }),
        )
    });

    if res.is_ok() {
        println!("\n✅ [Test] Generation completed!");
        
        // Let's verify that the kvcache.bin was created for the matched skill
        let cache_file = home_dir.join(".cluaiz").join("skills").join("minimax-music-gen").join(".cache").join("bonsai1-8b.kvcache.bin");
        if cache_file.exists() {
            println!("✅ [Test] VERIFIED: kvcache.bin was compiled successfully for minimax-music-gen!");
            println!("📁 Cache file location: {:?}", cache_file);
        } else {
            println!("❌ [Test] FAILED: kvcache.bin was not found!");
        }
    } else {
        println!("❌ [Test] Stream generation failed: {:?}", res.err());
    }

    Ok(())
}
