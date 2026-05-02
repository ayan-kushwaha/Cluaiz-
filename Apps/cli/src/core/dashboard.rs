use crate::core::state::{ActivityBlock, AppState};
use color_eyre::Result;
use colored::Colorize;
use crossterm::event::{self, Event};
use engines::DownloadEvent;
use inquire::{
    ui::{Attributes, Color, RenderConfig, Styled},
    Select, Text,
};
use std::io::{stdout, Write};
use tokio::sync::mpsc;

// ── 📦 MODULAR APPS ──
use crate::ui::apps::registry::RegistryApp;
use engines::utils::healer::AutoHealer;

pub struct DashboardEngine;

impl DashboardEngine {
    pub fn run_native(
        state: &mut AppState,
        tx: &mpsc::Sender<DownloadEvent>,
        mode: &mut crate::app_enums::Mode,
    ) -> Result<()> {
        // ── 🔒 SOVEREIGN RENDER CONFIG ──
        let config = RenderConfig::default()
            .with_prompt_prefix(
                Styled::new(">")
                    .with_fg(Color::LightCyan)
                    .with_attr(Attributes::BOLD),
            )
            .with_answered_prompt_prefix(Styled::new(">").with_fg(Color::LightCyan));

        // ── 🧬 ATOMIC NEURAL DISCOVERY (Sovereign Startup Scan) ──
        if state.sorted_models.is_empty() {
            println!("  {} Scanning Neural Sanctum...", "🧬".cyan());
            state.sorted_models = engines::NeuralRoster::get_recommendations(
                &state.hardware.to_silicon_truth(),
                state.ram_gb,
            );
            println!(
                "  {} Discovery Complete: Found {} neural assets.",
                "✅".green(),
                state.sorted_models.len()
            );
        }

        // ── 📡 SOVEREIGN TELEMETRY IGNITION (Ghost Observer Singleton) ──
        let state_pulse = archer_shared::hardware::telemetry::get_pulse();
        let pulse_ref = state_pulse.clone();
        let start_time = std::time::Instant::now();

        // 📡 SOVEREIGN PULSE CONTROL: Visibility gate to prevent chat history bleeding
        let show_dashboard = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let show_ref = show_dashboard.clone();
 
        std::thread::spawn(move || {
            use crossterm::{
                cursor, execute,
                style::{self, Stylize},
                terminal,
            };
            let mut stdout = std::io::stdout();
            loop {
                // 🛑 ATOMIC GATE: Skip rendering during bot inference phases
                if !show_ref.load(std::sync::atomic::Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    continue;
                }
                // 🧪 MICRO-LOCKING: Copy data and drop lock immediately to unblock background updates
                let (
                    cpu,
                    cpu_temp,
                    cpu_ghz,
                    ram_used,
                    ram_pct,
                    vram_used,
                    vram_total,
                    vram_pct,
                    gpu_temp,
                    tps,
                ) = {
                    let pulse_lock = pulse_ref.pulse.read().unwrap();
                    let primary_gpu_temp = pulse_lock.gpus.get(0).map(|g| g.temperature_c).unwrap_or(0.0);
                    
                    (
                        pulse_lock.cpu.utilization_pct,
                        pulse_lock.cpu.temperature_c,
                        pulse_lock.cpu.clock_ghz,
                        pulse_lock.ram.used_gb,
                        pulse_lock.ram.utilization_pct,
                        pulse_lock.vram_used_gb,
                        pulse_lock.vram_total_gb,
                        pulse_lock.vram_pressure_pct,
                        primary_gpu_temp,
                        pulse_lock.relay_latency_ms as f64 / 10.0,
                    )
                };

                let uptime = start_time.elapsed().as_millis() / 500;
                let is_blink_on = uptime % 2 == 0;

                // 🚦 Status Dot Logic
                let status_dot = if is_blink_on { "●" } else { " " };
                let cpu_color = if cpu < 50.0 { style::Color::Green } else if cpu < 80.0 { style::Color::Yellow } else { style::Color::Red };
                let gpu_color = if (vram_pct as f32) < 50.0 { style::Color::Green } else if (vram_pct as f32) < 80.0 { style::Color::Yellow } else { style::Color::Red };
                let ram_color = if ram_pct < 50.0 { style::Color::Green } else if ram_pct < 80.0 { style::Color::Yellow } else { style::Color::Red };
                let status_color = if cpu < 50.0 && (vram_pct as f32) < 50.0 { style::Color::Green } else if cpu < 80.0 || (vram_pct as f32) < 80.0 { style::Color::Yellow } else { style::Color::Red };

                // Surgical Overwrite: Force-inject metrics into the prompt's footprint (Y+1)
                if let Ok((_x, mut y)) = cursor::position() {
                    let (_cols, rows) = terminal::size().unwrap_or((80, 24));
                    
                    if y + 1 >= rows {
                        // 🚀 NEURAL SCROLL: Force a scroll up to keep prompt and dashboard separated
                        let _ = execute!(stdout, terminal::ScrollUp(1), cursor::MoveUp(1));
                        y -= 1; 
                    }

                    let _ = execute!(
                        stdout,
                        cursor::SavePosition,
                        cursor::MoveTo(0, y + 1),
                        terminal::Clear(terminal::ClearType::CurrentLine),
                        style::Print(Stylize::bold(Stylize::dim("[ "))),
                        style::Print(Stylize::bold(status_dot.with(status_color))),
                        style::Print(Stylize::bold(Stylize::dim(" LIVE │ "))),
                        
                        // CPU Pulse: Thermal & Load Sync
                        style::Print(Stylize::dim("CPU: ")),
                        style::Print(Stylize::dim(format!("{:.0}°C ({:.0}%)", cpu_temp, cpu))),
                        style::Print(Stylize::dim(format!(" │ {:.1}GHz", cpu_ghz))),
                        style::Print(Stylize::dim(") │ ")),

                        // GPU Pulse: Silicon Pressure Audit
                        style::Print(Stylize::dim("GPU: ")),
                        style::Print(Stylize::dim(format!("{:.0}°C ({:.0}%)", gpu_temp, vram_pct))),
                        style::Print(Stylize::dim(format!(" │ {:.1}/{:.0}GB", vram_used, vram_total))),
                        style::Print(Stylize::dim(") │ ")),

                        // RAM Pulse: Neural Buffer Load
                        style::Print(Stylize::dim("RAM: ")),
                        style::Print(Stylize::dim(format!("{:.1}GB ({:.0}%)", ram_used, ram_pct))),
                        style::Print(Stylize::dim(") │ ")),

                        // TPS Pulse: Relay Latency Audit
                        style::Print(Stylize::dim("TPS: ")),
                        style::Print(Stylize::dim(format!("{:.1}", tps))),
                        style::Print(Stylize::bold(Stylize::dim(" ]"))),

                        cursor::RestorePosition
                    );
                    let _ = stdout.flush();
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });

        loop {
            show_dashboard.store(true, std::sync::atomic::Ordering::SeqCst);
            println!(); // 🧿 SOVEREIGN BUFFER: Create space for surgical anchor
            let input = Text::new("")
                .with_placeholder("Type your message or @ & / for menu")
                .with_render_config(config.clone())
                .prompt();
            
            // 🛑 ATOMIC PURGE: Immediately silence dashboard and clear its line before chat output
            show_dashboard.store(false, std::sync::atomic::Ordering::SeqCst);
            if let Ok((_x, y)) = crossterm::cursor::position() {
                let mut stdout = std::io::stdout();
                let _ = crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, y + 1), crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine));
                let _ = stdout.flush();
            }

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
                        if *mode == crate::app_enums::Mode::Quit {
                            break;
                        }
                    } else if final_message.starts_with('@') {
                        Self::handle_model_switch(state, tx, &final_message[1..])?;
                    } else {
                        // ── 👤 USER MESSAGE ──
                        use crossterm::style::Stylize;
                        let icon = Stylize::bold(Stylize::cyan("👤"));
                        println!("{} {}", icon, final_message.clone().white());
                        state.activity_stream.push(ActivityBlock::Chat(
                            "USER".to_string(),
                            final_message.to_string(),
                        ));
                        state.rendered_actions_count += 1;

                        // ── 🧿 THINKING ANIMATION ──
                        print!("{} Thinking", Stylize::cyan("🤖"));
                        let _ = std::io::Write::flush(&mut std::io::stdout());

                        // ── 🤖 REAL NEURAL STREAMING ────────────────────────
                        let full_response =
                            std::sync::Arc::new(std::sync::Mutex::new(String::new()));
                        let full_clone = full_response.clone();
                        let mut first_token = true;

                        let stream_result = tokio::task::block_in_place(|| {
                            let mut lock = state.neural_engine.router.blocking_lock();
                            lock.generate_stream(
                                &final_message,
                                256,
                                Box::new(move |token| {
                                    if first_token {
                                        print!("\r\x1B[2K");
                                        use crossterm::style::Stylize;
                                        print!("{} ", Stylize::magenta("🤖"));
                                        first_token = false;
                                    }
                                    print!("{}", token);
                                    let _ = stdout().flush();
                                    if let Ok(mut res) = full_clone.lock() {
                                        res.push_str(&token);
                                    }
                                }),
                            )
                        });

                        println!();

                        let response = if let Err(e) = stream_result {
                            use crossterm::style::Stylize;
                            let err_msg = format!("{} ERROR: {}", Stylize::red("🤖"), e);
                            println!("{}", err_msg);
                            err_msg
                        } else {
                            let res = full_response.lock().unwrap().clone();
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
        tx: &mpsc::Sender<DownloadEvent>,
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
            .with_prompt_prefix(
                Styled::new("@")
                    .with_fg(Color::LightCyan)
                    .with_attr(Attributes::BOLD),
            )
            .with_highlighted_option_prefix(Styled::new("⮞").with_fg(Color::LightCyan));

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

        let ans = Select::new("Switch to model:", options)
            .with_render_config(config)
            .with_starting_cursor(starting_index)
            .prompt()?;

        print!("\x1B[1A\x1B[2K\r");
        stdout().flush()?;

        if let Some(model) = downloaded.iter().find(|m| m.manifest.name == ans) {
            if state._active_model_id.as_ref() == Some(&model.manifest.id) {
                println!(
                    "  {} {} is already active.",
                    "ℹ️".blue(),
                    model.manifest.name.bold()
                );
                return Ok(());
            }

            println!("  {} Loading: {}", "🧬".cyan(), model.manifest.name.bold());

            if let Some(path_str) = &model.manifest.local_path {
                let path = std::path::PathBuf::from(path_str);

                // 🧬 SOVEREIGN SILICON DETECTION
                let profile = archer_shared::hardware::get_sovereign_profile();
                let device = if profile.compute.has_gpu {
                    match profile.compute.primary_driver {
                        archer_shared::hardware::schema::profiles::BackendDriver::CUDA => {
                            candle_core::Device::new_cuda(0).unwrap_or(candle_core::Device::Cpu)
                        }
                        archer_shared::hardware::schema::profiles::BackendDriver::METAL => {
                            candle_core::Device::new_metal(0).unwrap_or(candle_core::Device::Cpu)
                        }
                        _ => candle_core::Device::Cpu,
                    }
                } else {
                    candle_core::Device::Cpu
                };

                println!(
                    "  {} [Silicon Dispatch] Using device: {:?}",
                    "🧪".cyan(),
                    device
                );

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
                    match handle.block_on(engines::NeuralRouter::load_model(
                        path,
                        runtime.clone(),
                        &device,
                    )) {
                        Ok(router) => {
                            let mut lock = state.neural_engine.router.blocking_lock();
                            *lock = router;
                            Ok(())
                        }
                        Err(e) => {
                            // ⚠️ NATIVE FALLBACK: Only for standard models (Bit-depth >= 2.0)!
                            // BitNet MUST NOT use RuntimeA (Candle) as it will crash with tensor errors.
                            if runtime == archer_shared::BackendType::RuntimeB
                                && model.manifest.bit_depth >= 2.0
                            {
                                let path_inner = std::path::PathBuf::from(path_str);
                                handle
                                    .block_on(engines::NeuralRouter::load_model(
                                        path_inner,
                                        archer_shared::BackendType::RuntimeA,
                                        &device,
                                    ))
                                    .map(|router| {
                                        let mut lock = state.neural_engine.router.blocking_lock();
                                        *lock = router;
                                    })
                            } else {
                                Err(e)
                            }
                        }
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
            state
                .activity_stream
                .push(ActivityBlock::ModelMounted(model.manifest.name.clone()));
        }

        Ok(())
    }
}
