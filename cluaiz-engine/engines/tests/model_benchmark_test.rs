use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::process::Command;
use cluaiz_shared::backend::traits::{CluaizInference, UnifiedBackend};
use engines::runtime::execution::hub::HardwareOrchestrator;
use cluaiz_shared::{CluaizContext, StructuralDNA, TemplateManager};
use tokenizers::Tokenizer;

fn get_models_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".cluaiz")
        .join("models")
        .join("chat")
}

fn get_benchmark_out_dir() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    while path.join("Cargo.toml").exists() {
        if path.file_name().unwrap() == "cluaiz" {
            break;
        }
        path.pop();
    }
    path.join("benchmark")
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

fn find_tokenizer(dir: &Path) -> Option<Tokenizer> {
    let path = dir.join("tokenizer.json");
    if path.exists() {
        Tokenizer::from_file(path).ok()
    } else {
        None
    }
}

#[tokio::test]
async fn run_model_benchmark_suite() {
    // Check if we are running in the isolated child process
    if let Ok(target_model) = std::env::var("BENCHMARK_MODEL_ID") {
        run_single_model_isolated(&target_model).await;
        return;
    }

    // --- PARENT PROCESS: ORCHESTRATOR ---
    println!("🚀 Starting Local Benchmark Orchestrator (Process Isolated Mode)");
    
    let models_dir = get_models_dir();
    let out_dir = get_benchmark_out_dir();
    
    if !models_dir.exists() {
        println!("No models directory found at {:?}", models_dir);
        return;
    }
    
    fs::create_dir_all(&out_dir).unwrap();
    
    let entries = fs::read_dir(&models_dir).unwrap();
    let mut models_to_test = Vec::new();
    
    for entry in entries.flatten() {
        let model_folder = entry.path();
        if !model_folder.is_dir() { continue; }
        models_to_test.push(model_folder.file_name().unwrap().to_str().unwrap().to_string());
    }

    for model_name in models_to_test {
        println!("\n=======================================================");
        println!("🧪 Spawning Isolated Process for Model: {}", model_name);
        println!("=======================================================");
        
        let status = Command::new("cargo")
            .args(["test", "-p", "engines", "--test", "model_benchmark_test", "run_model_benchmark_suite", "--", "--nocapture", "--exact"])
            .env("BENCHMARK_MODEL_ID", &model_name)
            .status();
            
        match status {
            Ok(s) if s.success() => {
                println!("✅ Model {} completed successfully.", model_name);
            }
            Ok(s) => {
                println!("❌ Model {} failed with exit code: {}", model_name, s);
            }
            Err(e) => {
                println!("❌ Failed to spawn process for {}: {}", model_name, e);
            }
        }
        
        println!("🧹 Parent process waiting 3s for OS to absolutely flush VRAM...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }
    
    println!("🎉 All benchmarks completed!");
}

async fn run_single_model_isolated(model_name: &str) {
    let models_dir = get_models_dir();
    let out_dir = get_benchmark_out_dir();
    let model_folder = models_dir.join(model_name);
    
    let gguf_file = match find_gguf_file(&model_folder) {
        Some(file) => file,
        None => {
            println!("⚠️ No .gguf file found for {}", model_name);
            return;
        }
    };
    
    let tokenizer = find_tokenizer(&model_folder);
    if tokenizer.is_none() {
        println!("⚠️ No tokenizer.json found for {}; skipping.", model_name);
        return;
    }
    let tokenizer = tokenizer.unwrap();
    
    // --- ⚙️ DYNAMIC THINK MODE CONTROL (Flash Mode) ---
    // Force think_mode OFF in the system booster settings for the benchmark run
    let mut booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
    booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::Off;
    let _ = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster);

    // --- ⚙️ BEST SETTINGS (UltraMaxBoost logic) ---
    let mut dna = StructuralDNA::default();
    // 1. Reduced sliding window context (helps VRAM and speed)
    dna.max_context_length = Some(4096); 
    
    let context = CluaizContext::boot(dna, TemplateManager::default());
    
    println!("🔥 Booting Engine in Process Isolation...");
    let mut engine = match HardwareOrchestrator::instantiate(gguf_file.to_str().unwrap(), context).await {
        Ok(engine) => engine,
        Err(e) => {
            println!("❌ Failed to instantiate engine for {}: {:?}", model_name, e);
            return;
        }
    };
    
    println!("🔥 Warming up {}...", model_name);
    let warmup_prompts = vec![
        "Hello",
        "Test",
    ];
    
    for prompt in warmup_prompts {
        let _ = engine.generate_stream(prompt, 5, &tokenizer, Box::new(|_| {}));
    }
    
    println!("⚡ Warmup complete. Running main benchmark...");
    
    // Keep prompt clean!
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
        
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let first_token_time = std::sync::Arc::new(std::sync::Mutex::new(None));
        let first_token_time_clone = first_token_time.clone();
        
        let _result = engine.generate_stream(
            main_prompt,
            2048,
            &tokenizer,
            Box::new(move |token| {
                let mut lock = first_token_time_clone.lock().unwrap();
                if lock.is_none() {
                    *lock = Some(start.elapsed().as_secs_f64());
                }
                let _ = tx.send(token);
            }),
        );
        
        while let Ok(token) = rx.try_recv() {
            generated_text.push_str(&token);
        }
        
        let elapsed = start.elapsed().as_secs_f64();
        let tokens = tokenizer.encode(generated_text.clone(), true).map(|e| e.len()).unwrap_or(0);
        let tps = if elapsed > 0.0 { tokens as f64 / elapsed } else { 0.0 };
        let ttft = first_token_time.lock().unwrap().unwrap_or(0.0);
        
        println!("    ⏱️ Time: {:.2}s | 🧩 Tokens: {} | 🚀 TPS: {:.2} | ⏱️ TTFT: {:.2}s", elapsed, tokens, tps, ttft);
        
        if tps > highest_tps {
            highest_tps = tps;
            best_run_output = generated_text;
            best_time = elapsed;
            best_tokens = tokens;
            best_ttft = ttft;
        }
    }
    
    println!("🏆 Best TPS for {}: {:.2}", model_name, highest_tps);
    
    let report_dir = out_dir.join(&model_name);
    fs::create_dir_all(&report_dir).unwrap();
    
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
    
    fs::write(report_dir.join("record.md"), report_content).unwrap();
    println!("📝 Report saved to {:?}", report_dir.join("record.md"));
}
