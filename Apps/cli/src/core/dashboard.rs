use crate::app_enums::Mode;
use crate::core::state::{ActivityBlock, AppState};
use color_eyre::Result;
use colored::Colorize;
use crossterm::event::{self, Event};
use engines::DownloadEvent;
use inquire::{
    ui::{Attributes, Color, RenderConfig, Styled},
    Select, Text,
};
use rand::seq::SliceRandom;
use std::io::{stdout, Write};
use std::time::Duration;
use tokio::sync::mpsc;

// ── 📦 MODULAR APPS ──
use crate::ui::apps::registry::RegistryApp;
use engines::utils::healer::AutoHealer;

pub struct DashboardEngine;

impl DashboardEngine {
    pub fn run_native(
        state: &mut AppState,
        tx: &mpsc::Sender<DownloadEvent>,
        mode: &mut Mode,
    ) -> Result<()> {
        // ── 🔒 SOVEREIGN RENDER CONFIG ──
        let config = RenderConfig::default()
            .with_prompt_prefix(Styled::new(">").with_fg(Color::LightCyan).with_attr(Attributes::BOLD))
            .with_answered_prompt_prefix(Styled::new(">").with_fg(Color::LightCyan));

        // ── 🧬 ATOMIC NEURAL DISCOVERY (Sovereign Startup Scan) ──
        if state.sorted_models.is_empty() {
             println!("  {} Scanning Neural Sanctum...", "🧬".cyan());
             state.sorted_models = engines::NeuralRoster::get_recommendations(&state.hardware, state.ram_gb);
             println!("  {} Discovery Complete: Found {} neural assets.", "✅".green(), state.sorted_models.len());
        }

        loop {
            let pulse = archer_shared::hardware::telemetry::get_pulse();
            let cpu = pulse.cpu_usage_pct.load(std::sync::atomic::Ordering::Relaxed);
            let ram = pulse.ram_usage_mb.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1024.0;
            let vram = pulse.vram_pressure_pct.load(std::sync::atomic::Ordering::Relaxed);
            let tps = pulse.current_tps.load(std::sync::atomic::Ordering::Relaxed) as f64 / 10.0;
            let kv = pulse.kv_cache_footprint_mb.load(std::sync::atomic::Ordering::Relaxed);
            
            let telemetry_bar = format!(
                "{} {} │ {} {} │ {} {} │ {} {} │ {} {} ",
                "⏱️ CPU:".dimmed(), format!("{:>2}%", cpu).cyan(),
                "RAM:".dimmed(), format!("{:>4.1}GB", ram).cyan(),
                "VRAM:".dimmed(), format!("{:>2}%", vram).cyan(),
                "TPS:".dimmed(), format!("{:>4.1}", tps).yellow(),
                "KV:".dimmed(), format!("{:>3}MB", kv).magenta(),
            );

            let input = Text::new("")
                .with_placeholder("Type your message or @ & / for menu")
                .with_help_message(&telemetry_bar)
                .with_render_config(config)
                .prompt();
            
            if input.is_ok() {
                print!("\x1B[1A\x1B[2K"); 
                stdout().flush()?;
            }

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
                    stdout().flush()?;

                    if final_message.starts_with('/') {
                        Self::handle_command(state, tx, mode, &final_message[1..])?;
                        if *mode == Mode::Quit { break; }
                    } else if final_message.starts_with('@') {
                        Self::handle_model_switch(state, tx, &final_message[1..])?;
                    } else {
                        // ── 👤 USER MESSAGE ──
                        let icon = "👤".cyan().bold();
                        println!("{} {}", icon, final_message.white());
                        state.activity_stream.push(ActivityBlock::Chat(
                            "USER".to_string(),
                            final_message.to_string(),
                        ));
                        state.rendered_actions_count += 1;

                        // ── 🧿 THINKING ANIMATION ──
                        print!("{} Thinking", "🤖".cyan());
                        let _ = stdout().flush();

                        // ── 🤖 REAL NEURAL STREAMING ────────────────────────
                        let full_response = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
                        let full_clone = full_response.clone();
                        let mut first_token = true;

                        let stream_result = tokio::task::block_in_place(|| {
                            let mut lock = state.neural_engine.router.blocking_lock();
                            lock.generate_stream(&final_message, 256, Box::new(move |token| {
                                if first_token {
                                    print!("\r                                     \r");
                                    print!("{} ", "🤖".magenta());
                                    first_token = false;
                                }
                                print!("{}", token);
                                let _ = stdout().flush();
                                if let Ok(mut res) = full_clone.lock() {
                                    res.push_str(&token);
                                }
                            }))
                        });

                        println!();

                        let response = if let Err(e) = stream_result {
                            let err_msg = format!("{} ERROR: {}", "🤖".red(), e);
                            println!("{}", err_msg);
                            err_msg
                        } else {
                            let res = full_response.lock().unwrap().clone();
                            if res.is_empty() { 
                                let empty_msg = format!("{} ERROR: Generated empty response.", "🤖".red());
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
                    }
                }
                Err(_) => {
                    *mode = Mode::Quit;
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_command(
        state: &mut AppState,
        tx: &mpsc::Sender<DownloadEvent>,
        mode: &mut Mode,
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
                    "Quit" => *mode = Mode::Quit,
                    _ => {}
                }
            }
            "quit" | "exit" => *mode = Mode::Quit,
            "clear" => {
                print!("\x1B[2J\x1B[1;1H");
                state.printed_logo = false;
            }
            _ => {
                println!("  {} Unknown command: /{}", "❌".red(), cmd);
            }
        }
        Ok(())
    }

    fn handle_model_switch(
        state: &mut AppState,
        _tx: &mpsc::Sender<DownloadEvent>,
        _filter: &str,
    ) -> Result<()> {
         let config = RenderConfig::default()
            .with_prompt_prefix(Styled::new("@").with_fg(Color::LightCyan).with_attr(Attributes::BOLD))
            .with_highlighted_option_prefix(Styled::new("⮞").with_fg(Color::LightCyan));
            
        let downloaded: Vec<_> = state.sorted_models.iter()
            .filter(|m| m.is_cached)
            .collect();

        if downloaded.is_empty() {
             println!("  {} No downloaded models found. Install from /menu.", "ℹ️".blue());
             return Ok(());
        }

        let options: Vec<String> = downloaded.iter().map(|m| m.manifest.name.clone()).collect();
        
        let starting_index = if let Some(active_id) = &state._active_model_id {
            downloaded.iter().position(|m| m.manifest.id == *active_id).unwrap_or(0)
        } else {
            0
        };

        let ans = Select::new("Switch to model:", options)
            .with_render_config(config)
            .with_starting_cursor(starting_index)
            .prompt()?;

        print!("\x1B[1A\x1B[2K\r");
        stdout().flush()?;

        if let Some(model) = downloaded.iter().find(|m| m.manifest.name == ans) {
            if state._active_model_id.as_ref() == Some(&model.manifest.id) {
                println!("  {} {} is already active.", "ℹ️".blue(), model.manifest.name.bold());
                return Ok(());
            }

            println!("  {} Loading: {}", "🧬".cyan(), model.manifest.name.bold());
            
            if let Some(path_str) = &model.manifest.local_path {
                let path = std::path::PathBuf::from(path_str);
                let device = candle_core::Device::Cpu; 
                
                // 🧬 SOVEREIGN DISPATCH: 
                // High bit-depth -> Native Rust (Candle)
                // 1-bit BitNet -> MANDATORY Llama (Binary)
                let runtime = if model.manifest.bit_depth < 2.0 { 
                    archer_shared::BackendType::RuntimeB 
                } else { 
                    archer_shared::BackendType::RuntimeA 
                };

                let result = tokio::task::block_in_place(|| {
                    let handle = tokio::runtime::Handle::current();
                    match handle.block_on(engines::NeuralRouter::load_model(path, runtime.clone(), &device)) {
                        Ok(router) => {
                            let mut lock = state.neural_engine.router.blocking_lock();
                            *lock = router;
                            Ok(())
                        }
                        Err(e) => {
                             // ⚠️ NATIVE FALLBACK: Only for standard models (Bit-depth >= 2.0)! 
                             // BitNet MUST NOT use RuntimeA (Candle) as it will crash with tensor errors.
                             if runtime == archer_shared::BackendType::RuntimeB && model.manifest.bit_depth >= 2.0 {
                                 let path_inner = std::path::PathBuf::from(path_str);
                                 handle.block_on(engines::NeuralRouter::load_model(path_inner, archer_shared::BackendType::RuntimeA, &device))
                                     .map(|router| {
                                         let mut lock = state.neural_engine.router.blocking_lock();
                                         *lock = router;
                                     })
                             } else {
                                 Err(e)
                             }
                        },
                    }
                });



                match result {
                    Ok(_) => {
                         state.neural_engine.is_loaded = true;
                         state._active_model_id = Some(model.manifest.id.clone());
                         println!("  {} Mounted successfully.", "✅".green());
                    }
                    Err(e) => println!("  {} Load failed: {}", "❌".red(), e),
                }
            }
            state.activity_stream.push(ActivityBlock::ModelMounted(model.manifest.name.clone()));
        }

        Ok(())
    }
}
