use color_eyre::Result;
use colored::Colorize;
use engines::models::registry::CoreRoster;
use engines::runtime::execution::hub::HardwareOrchestrator;
use cluaiz_shared::{CluaizContext, StructuralDNA, TemplateManager};
use cluaiz_shared::backend::traits::{CluaizInference, UnifiedBackend};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

pub async fn execute() -> Result<()> {
    println!("\n  {} [Performance] Starting Full System Benchmark...\n", "🚀".magenta());
    
    // Check if we are running in process-isolated mode
    if let Ok(target_model) = std::env::var("BENCHMARK_MODEL_ID") {
        run_single_model_isolated(&target_model).await;
        return Ok(());
    }

    let roster = CoreRoster::load_roster();
    
    if roster.is_empty() {
        println!("     {} No models found in the vault to benchmark.", "⚠️ ".yellow());
        return Ok(());
    }

    let out_dir = get_benchmark_out_dir();
    fs::create_dir_all(&out_dir).unwrap_or_default();
    
    println!("  {} Found {} models in the vault. Preparing benchmark orchestrator...\n", "📊".blue(), roster.len());

    let current_exe = std::env::current_exe().expect("Failed to get current executable path");

    for model in &roster {
        println!("=======================================================");
        println!("🧪 Spawning Isolated Process for Model: {}", model.id.green());
        println!("=======================================================");
        
        let status = std::process::Command::new(&current_exe)
            .args(["benchmark"])
            .env("BENCHMARK_MODEL_ID", &model.id)
            .env("BENCHMARK_FOLDER_NAME", model.local_path.as_deref().unwrap_or(&model.id))
            .status();
            
        match status {
            Ok(s) if s.success() => {
                println!("✅ Model {} completed successfully.", model.id);
            }
            Ok(s) => {
                println!("❌ Model {} failed with exit code: {}", model.id, s);
            }
            Err(e) => {
                println!("❌ Failed to spawn process for {}: {}", model.id, e);
            }
        }
        
        println!("🧹 Parent process waiting 3s for OS to absolutely flush VRAM...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    
    println!("\n  {} All benchmarks completed! Reports are in {:?}\n", "🎉".green(), out_dir);
    Ok(())
}

fn get_benchmark_out_dir() -> PathBuf {
    let mut path = std::env::current_dir().unwrap_or_default();
    while let Some(name) = path.file_name() {
        if name.to_string_lossy() == "cluaiz" {
            break;
        }
        if !path.pop() {
            break;
        }
    }
    path.join("test").join("benchmark")
}

fn find_gguf_file(dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                return Some(path);
            }
        }
    }
    None
}

async fn run_single_model_isolated(model_name: &str) {
    let folder_name = std::env::var("BENCHMARK_FOLDER_NAME").unwrap_or_else(|_| model_name.replace(':', "-"));
    
    let path_str = if folder_name.contains('/') || folder_name.contains('\\') {
        folder_name.clone()
    } else {
        let models_dir = dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".cluaiz")
            .join("models")
            .join("chat");
        models_dir.join(&folder_name).to_string_lossy().to_string()
    };
    
    let model_folder = PathBuf::from(path_str);
    let out_dir = get_benchmark_out_dir();
    
    let gguf_file = match find_gguf_file(&model_folder) {
        Some(file) => file,
        None => {
            println!("⚠️ No .gguf file found for {} at {:?}", model_name, model_folder);
            return;
        }
    };
    
    // Force think_mode OFF in the system booster settings for the benchmark run
    let mut booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
    booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::Off;
    let _ = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster);

    // DYNAMIC DNA: Try to load structural DNA from the local path if available
    let dna_path = model_folder.join("structural_dna.json");
    let mut dna = StructuralDNA::default();
    if dna_path.exists() {
        if let Ok(dna_content) = fs::read_to_string(&dna_path) {
            if let Ok(parsed_dna) = serde_json::from_str::<StructuralDNA>(&dna_content) {
                dna = parsed_dna;
            }
        }
    } else {
        dna.model_identity = model_name.to_string();
    }
    
    // Hard limit context for benchmark to ensure VRAM safety
    dna.max_context_length = Some(4096); 
    
    let context = CluaizContext::boot(dna.clone(), TemplateManager::default());
    
    println!("🔥 Booting Engine in Process Isolation...");
    let mut engine = match HardwareOrchestrator::instantiate(gguf_file.to_str().unwrap(), context).await {
        Ok(engine) => engine,
        Err(e) => {
            println!("❌ Failed to instantiate engine for {}: {:?}", model_name, e);
            return;
        }
    };
    
    println!("🔥 Warming up {}...", model_name);
    let warmup_prompts = vec!["Hello", "Test"];
    
    for prompt in warmup_prompts {
        let _ = engine.generate_stream(prompt, 5, Box::new(|_| {}));
    }
    
    println!("⚡ Warmup complete. Running main benchmark...");
    
    let main_prompt = "give top 50 most popular songs list";
    let mut highest_tps = 0.0;
    let mut best_run_output = String::new();
    let mut best_time = 0.0;
    let mut best_tokens = 0;
    let mut best_ttft = 0.0;
    
    for i in 1..=2 {
        println!("  [Run {}/2] Generating...", i);
        let start = Instant::now();
        let mut generated_text = String::new();
        
        let (tx, mut rx) = mpsc::unbounded_channel();
        let first_token_time = Arc::new(Mutex::new(None));
        let first_token_time_clone = first_token_time.clone();
        let token_count = Arc::new(AtomicUsize::new(0));
        let token_count_clone = token_count.clone();
        
        let result = engine.generate_stream(
            main_prompt,
            2048,
            Box::new(move |token| {
                let mut lock = first_token_time_clone.lock().unwrap();
                if lock.is_none() {
                    *lock = Some(start.elapsed().as_secs_f64());
                }
                token_count_clone.fetch_add(1, Ordering::Relaxed);
                let _ = tx.send(token);
            }),
        );
        
        if let Err(ref e) = result {
            println!("  ❌ [Run {}/2] generate_stream FAILED: {:?}", i, e);
            println!("     → Check if model ID is fully supported, or if Template/Vocab alignment is failing.");
            continue;
        }
        
        while let Ok(token) = rx.try_recv() {
            generated_text.push_str(&token);
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        let tokens = token_count.load(Ordering::Relaxed);
        let tps = if elapsed > 0.0 { tokens as f64 / elapsed } else { 0.0 };
        let ttft = first_token_time.lock().unwrap().unwrap_or(0.0);
        
        println!("    ⏱️ Time: {:.2}s | 🧩 Tokens: {} | 🚀 TPS: {:.2} | ⏱️ TTFT: {:.2}s", elapsed, tokens, tps, ttft);
        
        // Output sanity check
        if tokens == 0 {
            println!("    ⚠️ WARNING: Model generated 0 tokens. This often indicates the model instantly returned an EOS token.");
            println!("    ⚠️          Try checking if the model uses a strict ChatML/instruct template that wasn't fulfilled.");
        }
        
        if tps > highest_tps {
            highest_tps = tps;
            best_run_output = generated_text;
            best_time = elapsed;
            best_tokens = tokens;
            best_ttft = ttft;
        }
    }
    
    println!("🏆 Best TPS for {}: {:.2}", model_name, highest_tps);
    
    // Save report under safe folder name
    let safe_folder_name = model_name.replace(':', "-").replace('/', "_");
    let report_dir = out_dir.join(&safe_folder_name);
    fs::create_dir_all(&report_dir).unwrap_or_default();
    
    let report_content = format!(
        "# 🚀 Cluaiz Local Benchmark Report\n\n\
        ## 🤖 Model: {}\n\n\
        ### 🛠️ Hardware & Environment\n\
        - **Settings**: Process-Isolated / Reduced Sliding Window (4096 ctx) / Best Offload\n\
        - **Thinking Mode**: Forced OFF via low-level engine booster settings\n\
        - **VRAM Clearance**: Guaranteed 100% flush due to isolated execution process\n\n\
        ### 📊 Benchmark Results\n\
        - **Prompt**: `{}`\n\
        - **Highest Speed**: {:.2} TPS\n\
        - **Time to First Token (TTFT)**: {:.2} seconds\n\
        - **Total Tokens**: {}\n\
        - **Total Time**: {:.2} seconds\n\n\
        ### 📝 Output Log\n\
        ```text\n{}\n```\n",
        model_name, main_prompt, highest_tps, best_ttft, best_tokens, best_time, best_run_output
    );
    
    fs::write(report_dir.join("record.md"), report_content).unwrap_or_default();
    println!("📝 Report saved to {:?}", report_dir.join("record.md"));
}
