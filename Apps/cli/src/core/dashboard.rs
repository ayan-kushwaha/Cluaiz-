use crate::core::state::{ActivityBlock, AppState};
use color_eyre::Result;
use colored::Colorize;
use crossterm::execute;
use crossterm::cursor;
use engines::DownloadEvent;
use inquire::{
    ui::{Attributes, Color, RenderConfig, Styled},
    Select, Text,
};
use std::io::{stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use neural_core::interfaces::router_contract::EmbeddingDriver;
// ── 📦 MODULAR APPS ──
use crate::ui::apps::registry::RegistryApp;

pub struct DashboardEngine;

impl DashboardEngine {
    pub fn run_native(
        state: &mut AppState,
        tx: &mpsc::UnboundedSender<DownloadEvent>,
        rx: &mut mpsc::UnboundedReceiver<DownloadEvent>,
        mode: &mut crate::app_enums::Mode,
    ) -> Result<()> {
        // ══ 🔒 Cluaiz RENDER CONFIG ══
        let config = RenderConfig::default();

        // 🛑 GRACEFUL INTERRUPT HANDLER (Sovereign Pivot Control)
        let _ = ctrlc::set_handler(move || {
            cluaiz_shared::GLOBAL_CANCEL_SIGNAL.store(true, Ordering::SeqCst);
        });

        // ── 🧬 ATOMIC Core DISCOVERY (Cluaiz Startup Scan) ──
        if state.sorted_models.is_empty() {
            state.sorted_models = engines::CoreRoster::get_recommendations(
                &state.hardware.to_hardware_truth(),
                state.ram_gb,
            );
        }

        // ── 📡 Cluaiz TELEMETRY IGNITION (Ghost Observer Singleton) ──
        let state_pulse = cluaiz_shared::hardware::telemetry::get_pulse();
        let app_start_time = std::time::Instant::now();
        let last_inference_duration = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let last_ttft = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let peak_power = Arc::new(std::sync::atomic::AtomicU64::new(0));
        
        let ttft_ref = last_ttft.clone();
        let pwr_ref = peak_power.clone();
 
        let engine_ref = state.Core_engine.clone();
 
        // 🚀 CLUAIZ AUTO-BOOT: Activate the latest engine silently only if no model is loaded
        let is_engine_loaded = state.Core_engine.is_loaded.load(std::sync::atomic::Ordering::SeqCst);
        if state._active_model_id.is_none() && !is_engine_loaded {
            let auto_boot_name = state.sorted_models.iter().filter(|m| m.is_cached).next().map(|m| m.manifest.name.clone());
            if let Some(name) = auto_boot_name {
                println!("\n  {} Auto-Booting Neural Kernel: {}...", "🚀".magenta(), name.bold());
                let _ = Self::handle_model_switch(state, tx, rx, "", Some(&name));
            }
        }
        // 🖊️ INPUT FIX: Ensure cursor is on a fresh line before inquire renders
        println!();

        let mut last_booster_modified = std::fs::metadata(dirs::home_dir().unwrap_or_default().join(".cluaiz").join("engine").join("system_booster.json")).and_then(|m| m.modified()).ok();

        // Track global think state across pivots
        let global_think_state = Arc::new(AtomicBool::new(false));

        let mut prompt_embedding_engine: Option<cluaiz_onnx::engine::OnnxEngine> = None;
        let schema = engines::neural_foundry::security::permission_schema::PermissionSchema::load();
        if let Some(text_model_id) = schema.vector_models.text {
                let roster = engines::models::registry::CoreRoster::load_roster();
                if let Some(manifest) = roster.iter().find(|m| m.id == text_model_id) {
                    if let Some(local_path) = &manifest.local_path {
                        let model_dir = std::path::Path::new(local_path);
                        let model_file = model_dir.join("model.onnx");
                        let tokenizer_file = model_dir.join("tokenizer.json");
                        if model_file.exists() && tokenizer_file.exists() {
                            if let Ok(mut engine) = cluaiz_onnx::engine::OnnxEngine::new() {
                                if engine.load_text_model(&model_file.to_string_lossy(), &tokenizer_file.to_string_lossy()).is_ok() {
                                    prompt_embedding_engine = Some(engine);
                                }
                            }
                        }
                    }
                }
            }

        loop {
            
            let mut auto_input = None;
            if let Some(first) = state.chat_paste_buffer.first() {
                if first.starts_with("[PIVOT_CONTINUE]") {
                    auto_input = Some(state.chat_paste_buffer.remove(0));
                }
            }

            let input = if let Some(val) = auto_input {
                Ok(val)
            } else {
                // 🧹 Flush stdin buffer to clear any queued terminal input or echoed characters
                while let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(0)) {
                    let _ = crossterm::event::read();
                }

                let res = Text::new(">")
                    .with_placeholder("Type your message or @ & / for menu")
                    .with_render_config(config.clone())
                    .prompt();
                
                let (_cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
                let mut stdout = std::io::stdout();
                let _ = execute!(stdout, cursor::SavePosition, cursor::MoveTo(0, rows - 1), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine), cursor::RestorePosition);
                let _ = stdout.flush();

                if res.is_ok() {
                    print!("\x1B[1A\x1B[2K");
                    let _ = stdout.flush();
                }
                res
            };

            let now = std::time::Instant::now();
            let delta = now.duration_since(state.last_input_time).as_millis();
            state.last_input_time = now;

            match input {
                Ok(val) => {
                    let val = val.trim();
                    if val.is_empty() {
                        continue;
                    }

                    // 📋 PASTE DETECTION & MERGING
                    if delta < 50 {
                        state.chat_paste_buffer.push(val.to_string());
                        continue;
                    }

                    let final_message = if !state.chat_paste_buffer.is_empty() {
                        let mut merged = state.chat_paste_buffer.join("\n");
                        merged.push('\n');
                        merged.push_str(val);
                        state.chat_paste_buffer.clear();
                        merged
                    } else {
                        val.to_string()
                    };

                    print!("\x1B[1A\x1B[2K");
                    std::io::stdout().flush()?;

                    if final_message.starts_with('/') {
                        Self::handle_command(state, tx, mode, &final_message[1..])?;
                        if *mode == crate::app_enums::Mode::Quit {
                            break;
                        }
                    } else if final_message.starts_with('@') {
                        Self::handle_model_switch(state, tx, rx, &final_message[1..], None)?;
                    } else {
                        // show_dashboard.store(true, std::sync::atomic::Ordering::SeqCst);
                        
                        // ── 👤 USER MESSAGE ──
                        use crossterm::style::Stylize;
                        let icon = Stylize::bold(Stylize::cyan("👤"));
                        println!("{} {}", icon, final_message.clone().white());
                        state.activity_stream.push(ActivityBlock::Chat(
                            "USER".to_string(),
                            final_message.to_string(),
                        ));
                        state.rendered_actions_count += 1;

                        // ── 🧿 NEURAL DISPATCH ──
                        let _ = std::io::Write::flush(&mut std::io::stdout());

                        // ── 🤖 REAL Core STREAMING ────────────────────────
                        let full_response =
                            std::sync::Arc::new(std::sync::Mutex::new(String::new()));
                        let full_clone = full_response.clone();
                        let _first_token = true;
                        // 🧠 Think-mode state machine
                        let in_think = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                        let in_think_cb = in_think.clone();
                        // 🛑 EOS detection: stops display when model generates stop token
                        let reached_eos = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                        let eos_cb = reached_eos.clone();

                        let start_time = std::time::Instant::now();
                        let initial_tokens = state_pulse.tps_counter.load(Ordering::SeqCst);
                        
                        // Reset session metrics
                        pwr_ref.store(0.0f64.to_bits(), Ordering::SeqCst);
                        ttft_ref.store(0.0f64.to_bits(), Ordering::SeqCst);

                        let pulse_for_snapshot = state_pulse.clone();
                        let _pwr_cb = pwr_ref.clone();
                        let ttft_cb = ttft_ref.clone();
                        
                        // ── 🔥 HOT RELOAD ENGINE SETTINGS ──
                        let booster_path = dirs::home_dir().unwrap_or_default().join(".cluaiz").join("engine").join("system_booster.json");
                        if let Ok(meta) = std::fs::metadata(&booster_path) {
                            if let Ok(modified) = meta.modified() {
                                let mut needs_reload = false;
                                if let Some(last) = last_booster_modified {
                                    if modified > last {
                                        needs_reload = true;
                                    }
                                }
                                last_booster_modified = Some(modified);

                                if needs_reload {
                                    if let Some(model_id) = state._active_model_id.clone() {
                                        if let Some(model) = state.sorted_models.iter().find(|m| m.manifest.id == model_id) {
                                            if let Some(local_path) = &model.manifest.local_path {
                                                let path = std::path::PathBuf::from(local_path);
                                                println!("\r\x1B[2K\x1B[0m{} Hot-Reloading Neural Engine based on new settings...", crossterm::style::Stylize::magenta("🚀"));
                                                let rt = tokio::runtime::Handle::current();
                                                tokio::task::block_in_place(|| {
                                                    let _ = rt.block_on(state.Core_engine.load_model(path));
                                                });
                                                print!("\x1B[1A\x1B[2K\r"); // clear message
                                                let _ = std::io::stdout().flush();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Reset cancellation signal before starting
                        cluaiz_shared::GLOBAL_CANCEL_SIGNAL.store(false, Ordering::SeqCst);

                        let stream_result = tokio::task::block_in_place(|| {
                            let mut lock = state.Core_engine.router.blocking_lock();
                            
                            // 🧬 Dynamic Config Fetch: Zero Hardcoding
                            let stop_seqs = lock.get_active_dna().map(|d| d.stop_sequences.clone()).unwrap_or_default();
                            
                            // 🎭 Orchestration: native.rs handles the templating now.
                            // 🔮 SEMANTIC VECTOR ROUTING ──
                            let mut matched_skill_path = None;

                            if let Some(engine) = prompt_embedding_engine.as_mut() {
                                // Dynamic compilation of missing or mismatched semantic vectors
                                if let Ok(mut router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.write() {
                                    let _ = router.boot_index();
                                    let schema = engines::neural_foundry::security::permission_schema::PermissionSchema::load();
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
                                                println!("\r\n⏳ [Sovereign-Ops] Mismatch detected. Generating semantic vector for skill: {}", manifest.name);
                                                let skill_content = if manifest.triggers.semantic.is_empty() {
                                                    manifest.name.clone()
                                                } else {
                                                    manifest.triggers.semantic.join(", ")
                                                };
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

                                let prompt_words = final_message.trim().split_whitespace().count();
                                if prompt_words >= 3 {
                                    if let Ok(vector) = engine.gen_embedding(&final_message) {
                                        if let Ok(router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                                            matched_skill_path = router.check_semantic_trigger(&vector, 0.33); // 33% threshold for stable matching
                                        }
                                    }
                                }
                            }

                            if matched_skill_path.is_none() {
                                if let Ok(router) = cluaiz_shared::skills::router::GLOBAL_SKILL_ROUTER.read() {
                                    matched_skill_path = router.check_trigger(&final_message);
                                }
                            }

                            // Restore KV Cache if it exists, otherwise compile it dynamically on first trigger
                            let mut kv_cache_restored = false;
                            if let Some(ref skill_path) = matched_skill_path {
                                let schema = engines::neural_foundry::security::permission_schema::PermissionSchema::load();
                                if let Some(active_chat_model) = schema.get_active_chat_model() {
                                    let gen_model_safe = active_chat_model.replace(":", "-");
                                    let cache_dir = skill_path.join(".cache");
                                    let kv_cache_path = cache_dir.join(format!("{}.kvcache.bin", gen_model_safe));
                                    if kv_cache_path.exists() {
                                        let path_str = kv_cache_path.to_string_lossy().to_string();
                                        println!("\r\n🧠 [Sovereign-Ops] Restoring KV Cache for skill: {}", skill_path.file_name().unwrap_or_default().to_string_lossy());
                                        use cluaiz_shared::CluaizInference;
                                        if let Err(e) = lock.active_backend.load_kv_cache(&path_str) {
                                            println!("❌ [Sovereign-Ops] Failed to load KV cache: {}", e);
                                        } else {
                                            kv_cache_restored = true;
                                        }
                                    } else {
                                        if let Some(frontmatter) = extract_frontmatter(skill_path) {
                                            let prefix = format!("[System Memory Injection (Frontmatter): {}]\n", frontmatter);
                                            println!("\r\n⏳ [Sovereign-Ops] First time trigger: Compiling KV Cache for skill: {}...", skill_path.file_name().unwrap_or_default().to_string_lossy());
                                            use cluaiz_shared::UnifiedBackend;
                                            if let Err(e) = lock.active_backend.prefill(&prefix) {
                                                println!("❌ [Sovereign-Ops] Failed to prefill and compile KV Cache: {}", e);
                                            } else {
                                                use cluaiz_shared::CluaizInference;
                                                let path_str = kv_cache_path.to_string_lossy().to_string();
                                                let _ = std::fs::create_dir_all(&cache_dir);
                                                if let Err(e) = lock.active_backend.dump_kv_cache(&path_str) {
                                                    println!("❌ [Sovereign-Ops] Failed to dump KV Cache: {}", e);
                                                } else {
                                                    println!("✅ [Sovereign-Ops] KV Cache compiled successfully!");
                                                    kv_cache_restored = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            let mut formatted_prompt = final_message.clone();
                            if let Some(skill_path) = &matched_skill_path {
                                let skill_name = skill_path.file_name().unwrap_or_default().to_string_lossy();
                                println!("\r\n🔥 {} {}", colored::Colorize::magenta("[SOVEREIGN OPS] Semantic Skill Triggered:").bold(), colored::Colorize::yellow(&*skill_name));
                                if let Some(frontmatter) = extract_frontmatter(skill_path) {
                                    formatted_prompt = format!("[System Memory Injection (Frontmatter): {}]\n{}", frontmatter, final_message);
                                } else {
                                    formatted_prompt = format!("[System Memory Injection (Skill): {}]\n{}", skill_name, final_message);
                                }
                            }

                            // 🧬 DYNAMIC TOKEN ALLOCATION: Calculate space based on DNA Context Window
                            let ctx_window = lock.get_active_dna().and_then(|d| d.max_context_length).unwrap_or(2048);
                            let prompt_tokens = 0; // We no longer rely on external tokenizers for length prediction
                            
                            let max_t = lock.get_active_dna()
                                .and_then(|d| d.inference_params.get("max_tokens"))
                                .and_then(|v| v.parse::<usize>().ok())
                                .unwrap_or(8192); // 🚀 DYNAMIC: Allow large stream, context shifting will handle KV bounds.

                            // 🔇 SURGICAL SILENCE: freopen stderr→NUL only during inference.
                            #[cfg(windows)]
                            unsafe {
                                extern "C" { fn __acrt_iob_func(idx: u32) -> *mut libc::FILE; }
                                libc::freopen(
                                    "NUL\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    __acrt_iob_func(2),
                                );
                            }
                            #[cfg(not(windows))]
                            unsafe {
                                libc::freopen(
                                    "/dev/null\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    libc::fdopen(2, "w\0".as_ptr() as *const libc::c_char),
                                );
                            }
                            
                            let booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
                            let suppress_thinking = booster.think_mode == cluaiz_shared::hardware::schema::booster::FeatureState::Off;
                            
                            let active_model = state._active_model_id.clone().unwrap_or_default().to_lowercase();
                            let is_reasoning_model = active_model.contains("deepseek") || active_model.contains("r1") || active_model.contains("reason") || active_model.contains("bonsai") || active_model.contains("think");

                            let prompt_starts_in_think = formatted_prompt.contains("<think>") && !formatted_prompt.contains("</think>");
                            let is_pivot = formatted_prompt.starts_with("[PIVOT_CONTINUE]");

                            let mut first_token = true;
                            let pulse_clone = state_pulse.clone(); // 🧬 Clone for closure move
                            let global_think_cb = global_think_state.clone();
                            
                            // ⚡ Enable raw mode to intercept keystrokes asynchronously (like Ctrl+T)
                            let _ = crossterm::terminal::enable_raw_mode();

                            let result = lock.generate_stream(
                                &formatted_prompt, // 🚀 Pure, unadulterated prompt
                                max_t,
                                Box::new(move |token: String| -> bool {
                                    // 🛑 Stop if already past EOS or interrupted
                                    if eos_cb.load(Ordering::SeqCst) || cluaiz_shared::GLOBAL_CANCEL_SIGNAL.load(Ordering::SeqCst) { 
                                        return false; 
                                    }

                                    // ⚡ CLI Shortcut: Poll for Ctrl+T to skip thinking
                                    if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(0)) {
                                        if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                                            if key.code == crossterm::event::KeyCode::Char('t') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                                // Only skip if we are currently IN thinking mode — prevents double-fire from key-repeat
                                                if in_think_cb.load(Ordering::SeqCst) {
                                                    cluaiz_shared::GLOBAL_SKIP_THINKING_SIGNAL.store(true, Ordering::SeqCst);
                                                    // ✅ Immediately reset UI think state — don't wait for </think> text
                                                    in_think_cb.store(false, Ordering::SeqCst);
                                                    global_think_cb.store(false, Ordering::SeqCst);
                                                    print!("\x1B[0m\r\n⚡ Skipping thinking...\r\n");
                                                }
                                            }
                                            if key.code == crossterm::event::KeyCode::Char('c') && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                                cluaiz_shared::GLOBAL_CANCEL_SIGNAL.store(true, Ordering::SeqCst);
                                            }
                                        }
                                    }

                                    // 🛑 Deep-Suffix Scan
                                    if let Ok(mut res) = full_clone.lock() {
                                        let clean_res = (res.clone() + &token).replace("\n", "").replace("\r", "").replace(" ", "");
                                        if stop_seqs.iter().any(|s| {
                                            let clean_s = s.replace("\n", "").replace("\r", "").replace(" ", "");
                                            !clean_s.is_empty() && clean_res.ends_with(&clean_s)
                                        }) {
                                            eos_cb.store(true, Ordering::SeqCst);
                                            return false;
                                        }

                                        // 🤖 First-Token Handshake
                                        if first_token {
                                            let ttft = start_time.elapsed().as_secs_f64();
                                            ttft_cb.store(ttft.to_bits(), Ordering::SeqCst);

                                            let mut out = std::io::stdout();
                                            let _ = out.write_all(format!("\r\x1B[2K\x1B[0m{} ", crossterm::style::Stylize::magenta("🤖")).as_bytes());
                                            
                                            // DYNAMIC THINK INJECTION
                                            let should_start_in_think = prompt_starts_in_think || is_reasoning_model;

                                            if !suppress_thinking && should_start_in_think { 
                                                in_think_cb.store(true, Ordering::SeqCst);
                                                global_think_cb.store(true, Ordering::SeqCst);
                                                let mut out = std::io::stdout();
                                                let _ = out.write_all(b"\r\n\x1B[90m> \x1B[3m"); // Gray italics with carriage return
                                                let _ = out.flush();
                                            }
                                            
                                            let _ = out.flush();
                                            first_token = false;
                                        }

                                        // Filter tags and update state
                                        let mut display_token = token.clone();
                                        
                                        for tag in &["<turn|>", "<|im_end|>", "<end_of_turn>", "<|im_start|>", "<start_of_turn>"] {
                                            display_token = display_token.replace(tag, "");
                                        }

                                        let accumulated = res.clone() + &token;
                                        let mut just_finished_thinking = false;

                                        // ALWAYS check for </think> to hide it and cleanly exit think mode
                                        for tag in &["</think>", "</thought>", "<|thought_end|>", "<channel|>"] {
                                            if accumulated.ends_with(tag) || token.contains(tag) {
                                                in_think_cb.store(false, Ordering::SeqCst);
                                                global_think_cb.store(false, Ordering::SeqCst);
                                                display_token = display_token.replace(tag, "");
                                                just_finished_thinking = true;
                                            }
                                        }

                                        // ALWAYS check for <think> to turn ON think mode dynamically
                                        for tag in &["<think>", "<thought>", "<|thought_start|>"] {
                                            if accumulated.ends_with(tag) || token.contains(tag) {
                                                if !in_think_cb.load(Ordering::SeqCst) {
                                                    in_think_cb.store(true, Ordering::SeqCst);
                                                    global_think_cb.store(true, Ordering::SeqCst);
                                                    print!("\r\n\x1B[90m> \x1B[3m");
                                                }
                                                display_token = display_token.replace(tag, "");
                                            }
                                        }

                                        if just_finished_thinking {
                                            print!("\x1B[0m\r\n\r\n");
                                        }

                                        let currently_thinking = in_think_cb.load(Ordering::SeqCst);
                                        
                                        if !display_token.is_empty() {
                                            let display_token_raw = display_token.replace("\n", "\r\n");
                                            if currently_thinking && !suppress_thinking {
                                                 print!("\x1B[90m{}\x1B[0m", display_token_raw);
                                            } else {
                                                 print!("{}", display_token_raw);
                                            }
                                        } else if just_finished_thinking {
                                            print!("\x1B[0m"); // Even if display token is empty, we must reset
                                        }

                                        let _ = std::io::stdout().flush();
                                        res.push_str(&token); // Keep original token with tags in internal state
                                        pulse_clone.tps_counter.fetch_add(1, Ordering::SeqCst);
                                    }
                                    true
                                }),
                            );
                            
                            // ⚡ Disable raw mode safely after stream ends
                            let _ = crossterm::terminal::disable_raw_mode();
                            
                            // 🔊 RESTORE stderr → CONOUT$ (always the active Windows console)
                            #[cfg(windows)]
                            unsafe {
                                extern "C" { fn __acrt_iob_func(idx: u32) -> *mut libc::FILE; }
                                libc::freopen(
                                    "CONOUT$\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    __acrt_iob_func(2),
                                );
                            }
                            #[cfg(not(windows))]
                            unsafe {
                                libc::freopen(
                                    "/dev/tty\0".as_ptr() as *const libc::c_char,
                                    "w\0".as_ptr() as *const libc::c_char,
                                    libc::fdopen(2, "w\0".as_ptr() as *const libc::c_char),
                                );
                            }
                            result
                        });

                        let end_time = std::time::Instant::now();
                        let duration = end_time.duration_since(start_time).as_secs_f64();
                        let final_tokens = pulse_for_snapshot.tps_counter.load(Ordering::SeqCst);
                        let tokens_in_this_run = final_tokens.saturating_sub(initial_tokens);
                        let avg_tps = if duration > 0.0 { tokens_in_this_run as f64 / duration } else { 0.0 };



                        let response = if let Err(e) = stream_result {
                            use crossterm::style::Stylize;
                            let err_msg = format!("{} ERROR: {}", Stylize::red("🤖"), e);
                            println!("{}", err_msg);
                            err_msg
                        } else {
                            let res = if let Ok(lock) = full_response.lock() {
                                lock.clone()
                            } else {
                                String::new()
                            };
                            if res.is_empty() {
                                use crossterm::style::Stylize;
                                let empty_msg =
                                    format!("{} ERROR: Generated empty response.", Stylize::red("🤖"));
                                println!("{}", empty_msg);
                                empty_msg
                            } else {
                                res
                            }
                        };

                        state.activity_stream.push(ActivityBlock::Chat(
                            "ARCHER".to_string(),
                            response.to_string(),
                        ));

                        if cluaiz_shared::GLOBAL_CANCEL_SIGNAL.load(Ordering::SeqCst) {
                            println!();
                            use crossterm::style::Stylize;
                            println!("{} {}", "⏸️  Paused:".with(crossterm::style::Color::Yellow).bold(), "Engine stopped mid-generation. Context preserved in VRAM.".with(crossterm::style::Color::DarkGrey));
                            let pivot_input = Text::new("Enter mid-way instruction (or press Enter to return):")
                                .with_render_config(config.clone())
                                .prompt();
                            
                            if let Ok(instruction) = pivot_input {
                                if !instruction.trim().is_empty() {
                                    // Inject pivot into state to be processed in the next loop
                                    state.chat_paste_buffer.push(format!("[PIVOT_CONTINUE] {}", instruction.trim()));
                                }
                            }
                        }


                        
                        let ttft_secs = f64::from_bits(ttft_ref.load(Ordering::SeqCst));
                        let registry = cluaiz_shared::hardware::governor::HardwareGovernor::load_process_registry();
                        let my_pid = std::process::id().to_string();
                        let vram_used_gb = registry.get(&my_pid).map(|i| i.vram_gb).unwrap_or(0.0);

                        println!("\n  {} │ {} tokens │ {:.1} TPS │ {:.2}s │ TTFT: {:.2}s │ VRAM Used: {:.2} GB", 
                            colored::Colorize::magenta("⚡ System Benchmark"), 
                            colored::Colorize::cyan(tokens_in_this_run.to_string().as_str()), 
                            avg_tps, 
                            duration,
                            ttft_secs,
                            vram_used_gb
                        );
                        println!(); // ensure prompt starts on fresh line
                    }
                }
                Err(_) => {
                    *mode = crate::app_enums::Mode::Quit;
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_command(
        state: &mut AppState,
        tx: &mpsc::UnboundedSender<DownloadEvent>,
        mode: &mut crate::app_enums::Mode,
        cmd: &str,
    ) -> Result<()> {
        match cmd {
            "menu" | "" => {
                let config = RenderConfig::default()
                    .with_prompt_prefix(Styled::new("🏠︎").with_fg(Color::LightCyan))
                    .with_highlighted_option_prefix(
                        Styled::new("⮞")
                            .with_fg(Color::LightCyan)
                            .with_attr(Attributes::BOLD),
                    );
                let options = vec!["Model List", "Settings", "Help", "Quit"];
                let ans = Select::new("Main Menu:", options)
                    .with_render_config(config)
                    .prompt()?;

                print!("\x1B[1A\x1B[2K\r");
                stdout().flush()?;

                match ans {
                    "Model List" => RegistryApp::show(state, tx)?,
                    "Settings" => println!("  {} Settings coming soon...", "⚙️".yellow()),
                    "Help" => println!("  {} Help coming soon...", "ℹ️".blue()),
                    "Quit" => *mode = crate::app_enums::Mode::Quit,
                    _ => {}
                }
            }
            "quit" | "exit" => *mode = crate::app_enums::Mode::Quit,
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                state.printed_logo = false;
            }
            cmd if cmd.starts_with("run ") => {
                let model_id = &cmd[4..];
                println!("  {} Silicon Dispatch: '{}'...", "🧬".cyan(), model_id);
                
                let manager = engines::models::manager::ModelManager::new(
                    engines::models::registry::REGISTRY_URL.to_string(),
                    std::path::PathBuf::from("models")
                );

                tokio::task::block_in_place(|| {
                    let rt = tokio::runtime::Handle::current();
                    match rt.block_on(manager.pull_model(model_id)) {
                        Ok(_) => println!("  {} Link Established.", "✅".green()),
                        Err(e) => println!("  {} Dispatch Failed: {}", "❌".red(), e),
                    }
                });
            }
            _ => {
                println!("  {} Unknown command: /{}", "❌".red(), cmd);
            }
        }
        Ok(())
    }

    fn handle_model_switch(
        state: &mut AppState,
        _tx: &mpsc::UnboundedSender<DownloadEvent>,
        rx: &mut mpsc::UnboundedReceiver<DownloadEvent>,
        _filter: &str,
        auto_boot_target: Option<&str>,
    ) -> Result<()> {
        state.handle_events(rx);
        let config = RenderConfig::default()
            .with_prompt_prefix(
                Styled::new("@")
                    .with_fg(Color::LightCyan)
                    .with_attr(Attributes::BOLD),
            )
            .with_highlighted_option_prefix(Styled::new("⮞").with_fg(Color::LightCyan));

        let ans = if let Some(target) = auto_boot_target {
            target.to_string()
        } else {
            loop {
                let master_options = vec![
                    "🧠 Switch Model".to_string(),
                    "⚡ Engine Modes".to_string(),
                    "🚀 System Booster".to_string(),
                ];
                let master_ans = match Select::new("Action:", master_options)
                    .with_render_config(config.clone())
                    .prompt() {
                    Ok(ans) => ans,
                    Err(inquire::InquireError::OperationCanceled) | Err(inquire::InquireError::OperationInterrupted) => {
                        print!("\x1B[1A\x1B[2K\r");
                        stdout().flush()?;
                        return Ok(());
                    }
                    Err(e) => return Err(e.into()),
                };
                    
                print!("\x1B[1A\x1B[2K\r");
                stdout().flush()?;

                if master_ans.contains("Engine Modes") {
                    let modes = vec![
                        "⚡ Flash Mode (High Speed)".to_string(),
                        "🧠 Think Mode (Deep Reasoning)".to_string(),
                        "🚀 Boot Mode (Auto-Start Engine)".to_string(),
                    ];
                    let mode_ans = match Select::new("Select Mode:", modes)
                        .with_render_config(config.clone())
                        .prompt() {
                        Ok(ans) => ans,
                        Err(inquire::InquireError::OperationCanceled) | Err(inquire::InquireError::OperationInterrupted) => {
                            print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                            stdout().flush()?;
                            continue; // One step back
                        }
                        Err(e) => return Err(e.into()),
                    };
                        
                    print!("\x1B[1A\x1B[2K\r");
                    stdout().flush()?;
                    
                    let mut booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
                    if mode_ans.contains("Flash Mode") {
                        booster.mode_run = cluaiz_shared::hardware::schema::booster::BoosterMode::Edge;
                        booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::Off;
                    } else if mode_ans.contains("Think Mode") {
                        booster.mode_run = cluaiz_shared::hardware::schema::booster::BoosterMode::MaxBoost;
                        booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::On;
                    } else if mode_ans.contains("Boot Mode") {
                        booster.mode_run = cluaiz_shared::hardware::schema::booster::BoosterMode::Balance;
                        booster.think_mode = cluaiz_shared::hardware::schema::booster::FeatureState::Auto;
                    }
                    
                    let _ = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster);
                    
                    println!("  {} {} activated and saved to system_booster.json.", "✅".green(), mode_ans.bold());
                    return Ok(());
                } else if master_ans.contains("System Booster") {
                    let mut booster_path = dirs::home_dir().unwrap_or_default();
                    booster_path.push(".cluaiz");
                    booster_path.push("engine");
                    booster_path.push("system_booster.json");

                    loop {
                        let mut booster = cluaiz_shared::hardware::governor::HardwareGovernor::load_booster_settings().unwrap_or_default();
                        
                        let compute_mode_str = match booster.n_gpu_layers {
                            0 => "CPU Only".to_string(),
                            -1 => "GPU (Full Offload)".to_string(),
                            n => format!("Hybrid ({} Layers)", n),
                        };

                        let mut options = vec![
                            format!("Neural Mode (Current: {:?})", booster.mode_run),
                            format!("Compute Device (Current: {})", compute_mode_str),
                            format!("Turbo Quant (Current: {:?})", booster.turbo_quant),
                            format!("Flash Attention (Current: {:?})", booster.flash_attention),
                            format!("Speculative Decoding (Current: {:?})", booster.speculative_decoding),
                            format!("Auto Round (Current: {:?})", booster.auto_round),
                            format!("DFlash (FlashKDA) (Current: {:?})", booster.dflash),
                            format!("Context Shifting (Current: {:?})", booster.context_shifting),
                            format!("Force VRAM Reclaim (Current: {:?})", booster.force_vram_reclaim),
                            format!("KV Cache Quantization (Current: {:?})", booster.kv_cache_quantization),
                        ];
                        options.push("🔙 Back to Menu".to_string());
                        
                        let target_ans = match Select::new("Configure Setting:", options)
                            .with_render_config(config.clone())
                            .prompt() {
                            Ok(ans) => ans,
                            Err(_) => {
                                print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                                stdout().flush()?;
                                break; // One step back (returns to master menu)
                            }
                        };
                        print!("\x1B[1A\x1B[2K\r");
                        stdout().flush()?;

                        if target_ans == "🔙 Back to Menu" {
                            break; // One step back
                        }

                        let key_part = target_ans.split(" (").next().unwrap_or("").to_string();

                        // If they select Compute Device, show device sub-menu directly
                        if key_part.as_str() == "Compute Device" {
                            let device_options = vec![
                                "GPU (Full Offload)".to_string(),
                                "CPU Only".to_string(),
                                "Custom Layers".to_string(),
                            ];
                            let selected_device = match Select::new("Select Compute Device:", device_options)
                                .with_render_config(config.clone())
                                .prompt() {
                                Ok(ans) => ans,
                                Err(_) => {
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                    continue;
                                }
                            };
                            print!("\x1B[1A\x1B[2K\r");
                            stdout().flush()?;

                            match selected_device.as_str() {
                                "GPU (Full Offload)" => {
                                    booster.n_gpu_layers = -1;
                                }
                                "CPU Only" => {
                                    booster.n_gpu_layers = 0;
                                }
                                "Custom Layers" => {
                                    let layers_input = inquire::CustomType::<i32>::new("Enter number of GPU layers (-1 for full offload):")
                                        .with_default(-1)
                                        .with_render_config(config.clone())
                                        .prompt();
                                    match layers_input {
                                        Ok(layers) => {
                                            booster.n_gpu_layers = layers;
                                        }
                                        Err(_) => {
                                            print!("\x1B[1A\x1B[2K\r");
                                            stdout().flush()?;
                                            continue;
                                        }
                                    }
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                }
                                _ => {}
                            }

                            if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                                println!("  {} System Booster updated: Compute Device = {}", "✅".green(), selected_device.bold());
                            } else {
                                println!("  {} Failed to save system booster settings.", "❌".red());
                            }
                            continue;
                        }

                        // Special sub-menus for context shifting & reclaim
                        if key_part.as_str() == "Context Shifting" {
                            let shift_modes = vec![
                                "Off".to_string(),
                                "Minimal".to_string(),
                                "Standard".to_string(),
                                "Aggressive".to_string(),
                                "Extreme".to_string(),
                                "Auto".to_string(),
                            ];
                            let selected_shift = match Select::new("Select Context Shifting:", shift_modes)
                                .with_render_config(config.clone())
                                .prompt() {
                                Ok(ans) => ans,
                                Err(_) => {
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                    continue;
                                }
                            };
                            print!("\x1B[1A\x1B[2K\r");
                            stdout().flush()?;

                            booster.context_shifting = match selected_shift.as_str() {
                                "Off" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Off,
                                "Minimal" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Minimal,
                                "Standard" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Standard,
                                "Aggressive" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Aggressive,
                                "Extreme" => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Extreme,
                                _ => cluaiz_shared::hardware::schema::booster::ContextShiftingMode::Auto,
                            };

                            if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                                println!("  {} System Booster updated: Context Shifting = {}", "✅".green(), selected_shift.bold());
                            } else {
                                println!("  {} Failed to save system booster settings.", "❌".red());
                            }
                            continue;
                        }

                        if key_part.as_str() == "KV Cache Quantization" {
                            let kv_options = vec![
                                "16-bit (Lossless / High Precision)".to_string(),
                                "8-bit (50% VRAM Saving / Balanced)".to_string(),
                                "4-bit (75% VRAM Saving / High Compression)".to_string(),
                                "Auto (Dynamic Quantization)".to_string(),
                            ];
                            let selected_kv = match Select::new("Select KV Cache Quantization:", kv_options)
                                .with_render_config(config.clone())
                                .prompt() {
                                Ok(ans) => ans,
                                Err(_) => {
                                    print!("\x1B[1A\x1B[2K\r");
                                    stdout().flush()?;
                                    continue;
                                }
                            };
                            print!("\x1B[1A\x1B[2K\r");
                            stdout().flush()?;

                            booster.kv_cache_quantization = match selected_kv.as_str() {
                                s if s.starts_with("16-bit") => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Kv16,
                                s if s.starts_with("8-bit") => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Kv8,
                                s if s.starts_with("4-bit") => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Kv4,
                                _ => cluaiz_shared::hardware::schema::booster::KvCacheQuantization::Auto,
                            };

                            if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                                println!("  {} System Booster updated: KV Cache Quantization = {}", "✅".green(), selected_kv.bold());
                            } else {
                                println!("  {} Failed to save system booster settings.", "❌".red());
                            }
                            continue;
                        }

                        let values = vec!["On".to_string(), "Off".to_string(), "Auto".to_string()];
                        
                        let val_ans = match Select::new(&format!("Set {}:", key_part), values)
                            .with_render_config(config.clone())
                            .prompt() {
                            Ok(ans) => ans,
                            Err(_) => {
                                print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                                stdout().flush()?;
                                continue; // One step back (stays in System Booster list)
                            }
                        };
                        print!("\x1B[1A\x1B[2K\r");
                        stdout().flush()?;

                        let feature_state = match val_ans.as_str() {
                            "On" => cluaiz_shared::hardware::schema::booster::FeatureState::On,
                            "Off" => cluaiz_shared::hardware::schema::booster::FeatureState::Off,
                            _ => cluaiz_shared::hardware::schema::booster::FeatureState::Auto,
                        };

                        match key_part.as_str() {
                            "Neural Mode" => {
                                let mut modes = vec![
                                    "edge".to_string(), 
                                    "multitasking".to_string(), 
                                    "balance".to_string(), 
                                    "max_boost".to_string(), 
                                    "ultra_max_boost".to_string()
                                ];

                                // 🌌 VRAM GUARD: Only show HyperCluster if VRAM >= 40GB
                                let total_vram = {
                                    let pulse_lock = state.live_pulse.pulse.read().unwrap();
                                    pulse_lock.vram_total_gb
                                };
                                if total_vram >= 40.0 {
                                    modes.push("hyper_cluster".to_string());
                                }

                                let selected_mode = match Select::new("Select Neural Mode:", modes)
                                    .with_render_config(config.clone())
                                    .prompt() {
                                    Ok(ans) => ans,
                                    Err(_) => continue,
                                };
                                booster.mode_run = match selected_mode.as_str() {
                                    "edge" => cluaiz_shared::hardware::schema::booster::BoosterMode::Edge,
                                    "multitasking" => cluaiz_shared::hardware::schema::booster::BoosterMode::Multitasking,
                                    "balance" => cluaiz_shared::hardware::schema::booster::BoosterMode::Balance,
                                    "max_boost" => cluaiz_shared::hardware::schema::booster::BoosterMode::MaxBoost,
                                    "ultra_max_boost" => cluaiz_shared::hardware::schema::booster::BoosterMode::UltraMaxBoost,
                                    "hyper_cluster" => cluaiz_shared::hardware::schema::booster::BoosterMode::HyperCluster,
                                    _ => cluaiz_shared::hardware::schema::booster::BoosterMode::Balance,
                                };
                            },
                            "Turbo Quant" => booster.turbo_quant = feature_state,
                            "Flash Attention" => booster.flash_attention = feature_state,
                            "Speculative Decoding" => booster.speculative_decoding = feature_state,
                            "Auto Round" => booster.auto_round = feature_state,
                            "DFlash (FlashKDA)" => {
                                booster.dflash = match val_ans.as_str() {
                                    "On" => cluaiz_shared::hardware::schema::booster::SmartState::Static("On".to_string()),
                                    "Off" => cluaiz_shared::hardware::schema::booster::SmartState::Static("Off".to_string()),
                                    _ => cluaiz_shared::hardware::schema::booster::SmartState::Static("Auto".to_string()),
                                };
                            },
                            "Force VRAM Reclaim" => booster.force_vram_reclaim = feature_state,
                            _ => {}
                        }
                        
                        if let Ok(_) = cluaiz_shared::hardware::governor::HardwareGovernor::save_booster_settings(&booster) {
                            println!("  {} System Booster updated: {} = {}", "✅".green(), key_part.cyan(), val_ans.bold());
                        } else {
                            println!("  {} Failed to save system booster settings.", "❌".red());
                        }
                    }
                    continue; // Go back to Master Menu after exiting System Booster
                }

                let downloaded: Vec<_> = state.sorted_models.iter().filter(|m| m.is_cached).collect();

                if downloaded.is_empty() {
                    println!(
                        "  {} No downloaded models found. Install from /menu.",
                        "ℹ️".blue()
                    );
                    return Ok(());
                }

                let options: Vec<String> = downloaded.iter().map(|m| m.manifest.name.clone()).collect();

                let starting_index = if let Some(active_id) = &state._active_model_id {
                    downloaded
                        .iter()
                        .position(|m| m.manifest.id == *active_id)
                        .unwrap_or(0)
                } else {
                    0
                };

                let selection = match Select::new("Switch to model:", options)
                    .with_render_config(config.clone())
                    .with_starting_cursor(starting_index)
                    .prompt() {
                    Ok(ans) => ans,
                    Err(inquire::InquireError::OperationCanceled) | Err(inquire::InquireError::OperationInterrupted) => {
                        print!("\x1B[1A\x1B[2K\r"); // Erase <canceled>
                        stdout().flush()?;
                        continue; // One step back (goes to Action menu)
                    }
                    Err(e) => return Err(e.into()),
                };

                print!("\x1B[1A\x1B[2K\r");
                stdout().flush()?;
                break selection; // Breaks the master loop and returns the selected model
            }
        };

        let downloaded: Vec<_> = state.sorted_models.iter().filter(|m| m.is_cached).collect();

        if let Some(model) = downloaded.iter().find(|m| m.manifest.name == ans) {
            if state._active_model_id.as_ref() == Some(&model.manifest.id) {
                println!(
                    "  {} {} is already active.",
                    "ℹ️".blue(),
                    model.manifest.name.bold()
                );
                return Ok(());
            }

            println!("  {} Loading {}, please wait a moment...", "⏳".yellow(), model.manifest.name.bold());

            if let Some(path_str) = &model.manifest.local_path {
                let path = std::path::PathBuf::from(path_str);

                // 🧬 Cluaiz DISPATCH:
                // High bit-depth -> Native Rust
                // 1-bit BitNet -> MANDATORY Llama (Binary)
                let runtime = if model.manifest.bit_depth < 2.0 {
                    cluaiz_shared::BackendType::RuntimeB
                } else {
                    cluaiz_shared::BackendType::RuntimeA
                };

                let result = tokio::task::block_in_place(|| {
                    let handle = tokio::runtime::Handle::current();
                    
                    // 🛑 SURGICAL FIX: Destroy old router to free VRAM BEFORE loading new model!
                    {
                        let mut lock = state.Core_engine.router.blocking_lock();
                        *lock = engines::CoreRouter::new();
                    }
                    state.Core_engine.is_loaded.store(false, std::sync::atomic::Ordering::SeqCst);
                    
                    match handle.block_on(engines::CoreRouter::load_model(
                        path,
                        runtime.clone(),
                    )) {
                        Ok(router) => {
                            let mut lock = state.Core_engine.router.blocking_lock();
                            *lock = router;
                            state.Core_engine.is_loaded.store(true, std::sync::atomic::Ordering::SeqCst);
                            Ok(())
                        }
                        Err(e) => {
                            // ⚠️ NATIVE FALLBACK: Only for standard models (Bit-depth >= 2.0)!
                            // BitNet MUST NOT use RuntimeA (Candle) as it will crash with tensor errors.
                            if runtime == cluaiz_shared::BackendType::RuntimeB
                                && model.manifest.bit_depth >= 2.0
                            {
                                let path_inner = std::path::PathBuf::from(path_str);
                                handle
                                    .block_on(engines::CoreRouter::load_model(
                                        path_inner,
                                        cluaiz_shared::BackendType::RuntimeA
                                    ))
                                    .map(|router| {
                                        let mut lock = state.Core_engine.router.blocking_lock();
                                        *lock = router;
                                        state.Core_engine.is_loaded.store(true, std::sync::atomic::Ordering::SeqCst);
                                    })
                            } else {
                                Err(e)
                            }
                        }
                    }
                });

                match result {
                    Ok(_) => {
                        model.manifest.id.clone();
                        state._active_model_id = Some(model.manifest.id.clone());
                        println!("  {} Mounted successfully.", "✅".green());
                    }
                    Err(e) => println!("  {} Load failed: {}", "❌".red(), e),
                }
            }
            state
                .activity_stream
                .push(ActivityBlock::ModelMounted(model.manifest.name.clone()));
        }

        Ok(())
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
